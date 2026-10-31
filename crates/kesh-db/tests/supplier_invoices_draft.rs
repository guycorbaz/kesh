//! Story v017-4a (#849) — le cycle brouillon → validée → payée d'une facture
//! fournisseur, au niveau du dépôt (`kesh_db::repositories::supplier_invoices`).
//!
//! - AC-6 : créer, c'est enregistrer un brouillon — aucune écriture, total par la
//!   fonction pure de `kesh-core`, brouillon hors exercice accepté ;
//! - AC-8 : `update_draft` remplace l'en-tête et toutes les lignes ;
//! - AC-9 : `delete_draft` — document rendu « à traiter », refus nommés, et le
//!   test **piloté** de l'ordre des verrous (facture puis document, M20-quater) ;
//! - AC-10 : `validate` passe l'écriture d'achat de la rc1 (comparée à une
//!   écriture **écrite en dur**), ses refus, et la course pilotée (M20) ;
//! - AC-11 : `unvalidate` supprime l'écriture d'achat (règle de la facture
//!   client), sa précédence paire par paire ;
//! - AC-12 : `create_validated` compose les deux ;
//! - AC-13 : un brouillon ne s'annule pas.
//!
//! Décor synthétique : `seed_accounting_company` (exercice 2020-2030 ouvert,
//! comptes 1000, 1100, 2000, 3000, 4000 ; TVA récupérable au 1000) et
//! `fournisseur_rapproche::preparer` (créanciers 2000, un fournisseur, un compte
//! bancaire lié au 1100).
//!
//! Pré-requis : MariaDB démarré.

#[path = "support/fournisseur_rapproche.rs"]
mod fournisseur_rapproche;

use chrono::NaiveDate;
use fournisseur_rapproche::{Fournisseur, IBAN_CREANCIER};
use kesh_db::entities::imported_supplier_invoice::{LinkOrigin, NewImportedSupplierInvoice};
use kesh_db::entities::{
    NewPaymentBatch, NewSupplierInvoice, NewSupplierInvoiceLine, SettlementChoice,
};
use kesh_db::errors::{DbError, SettlementCancelBlocker, SupplierUnvalidationBlocker};
use kesh_db::repositories::{imported_supplier_invoices, payment_batches, supplier_invoices};
use kesh_db::test_fixtures::{
    SeededCompany, attendre_une_requete_en_cours, seed_accounting_company,
};
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use sqlx::MySqlPool;
use std::time::Duration;

const BORNE: Duration = Duration::from_secs(10);

fn jour(y: i32, m: u32, d: u32) -> NaiveDate {
    NaiveDate::from_ymd_opt(y, m, d).unwrap()
}

async fn decor(pool: &MySqlPool) -> (SeededCompany, Fournisseur) {
    let seeded = seed_accounting_company(pool).await.expect("seed");
    let f = fournisseur_rapproche::preparer(pool, &seeded).await;
    (seeded, f)
}

/// Une facture d'une ligne (`montant` HT, taux `taux`), datée `date`.
fn nouvelle(
    seeded: &SeededCompany,
    f: Fournisseur,
    numero: &str,
    date: NaiveDate,
    lignes: &[(Decimal, Decimal)],
) -> NewSupplierInvoice {
    NewSupplierInvoice {
        company_id: seeded.company_id,
        contact_id: f.supplier_id,
        supplier_invoice_number: Some(numero.into()),
        invoice_date: date,
        due_date: None,
        creditor_iban: Some(IBAN_CREANCIER.into()),
        creditor_qr_iban: None,
        payment_reference: None,
        expected_payment_amount: None,
        project_id: None,
        lines: lignes
            .iter()
            .enumerate()
            .map(|(i, (prix, taux))| NewSupplierInvoiceLine {
                description: format!("Ligne {}", i + 1),
                quantity: dec!(1),
                unit_price: *prix,
                vat_rate: *taux,
                expense_account_id: seeded.accounts["4000"],
            })
            .collect(),
    }
}

async fn brouillon(
    pool: &MySqlPool,
    seeded: &SeededCompany,
    f: Fournisseur,
    numero: &str,
    montant: Decimal,
) -> supplier_invoices::SupplierInvoiceWithLines {
    supplier_invoices::create_draft(
        pool,
        nouvelle(seeded, f, numero, jour(2026, 6, 15), &[(montant, dec!(0))]),
        seeded.admin_user_id,
    )
    .await
    .expect("brouillon")
}

async fn validee(
    pool: &MySqlPool,
    seeded: &SeededCompany,
    f: Fournisseur,
    numero: &str,
    montant: Decimal,
) -> supplier_invoices::SupplierInvoiceWithLines {
    supplier_invoices::create_validated(
        pool,
        nouvelle(seeded, f, numero, jour(2026, 6, 15), &[(montant, dec!(0))]),
        seeded.admin_user_id,
    )
    .await
    .expect("facture validée")
}

async fn nb_ecritures(pool: &MySqlPool, company_id: i64) -> i64 {
    sqlx::query_scalar("SELECT COUNT(*) FROM journal_entries WHERE company_id = ?")
        .bind(company_id)
        .fetch_one(pool)
        .await
        .unwrap()
}

async fn etat(pool: &MySqlPool, id: i64) -> (String, Option<i64>, i32) {
    sqlx::query_as(
        "SELECT status, purchase_journal_entry_id, version FROM supplier_invoices WHERE id = ?",
    )
    .bind(id)
    .fetch_one(pool)
    .await
    .unwrap()
}

async fn nb_audits(pool: &MySqlPool, action: &str, entity_id: i64) -> i64 {
    sqlx::query_scalar("SELECT COUNT(*) FROM audit_log WHERE action = ? AND entity_id = ?")
        .bind(action)
        .bind(entity_id)
        .fetch_one(pool)
        .await
        .unwrap()
}

// ---------------------------------------------------------------------------
// AC-6 — créer un brouillon
// ---------------------------------------------------------------------------

/// AC-6 — `create_draft` n'écrit aucune écriture ; le total est HT + TVA
/// arrondie **par ligne**. Tue M1 (écriture passée) et M20-bis (total HT seul).
#[sqlx::test(migrations = "./test-schema")]
async fn a_draft_writes_no_entry_and_carries_the_ttc(pool: MySqlPool) {
    let (seeded, f) = decor(&pool).await;
    let avant = nb_ecritures(&pool, seeded.company_id).await;
    let d = supplier_invoices::create_draft(
        &pool,
        nouvelle(
            &seeded,
            f,
            "FF-B1",
            jour(2026, 6, 15),
            &[(dec!(100.05), dec!(8.1)), (dec!(10.00), dec!(2.6))],
        ),
        seeded.admin_user_id,
    )
    .await
    .expect("brouillon");
    assert_eq!(nb_ecritures(&pool, seeded.company_id).await, avant, "M1");
    assert_eq!(d.invoice.status, "draft");
    assert_eq!(d.invoice.purchase_journal_entry_id, None);
    assert_eq!(d.invoice.version, 1);
    // 100.05 × 8.1 % = 8.10405 → 8.10 ; 10 × 2.6 % = 0.26 → TTC 118.41.
    assert_eq!(d.invoice.total_amount, dec!(118.41), "M20-bis");
    assert_eq!(d.lines.len(), 2);
    let snapshot: serde_json::Value = sqlx::query_scalar(
        "SELECT details FROM audit_log WHERE action = 'supplier_invoice.created' AND entity_id = ?",
    )
    .bind(d.invoice.id)
    .fetch_one(&pool)
    .await
    .unwrap();
    assert!(
        snapshot.get("journalEntryId").is_none(),
        "un brouillon n'a pas d'écriture à tracer : {snapshot}"
    );
}

/// AC-6 — un brouillon daté hors de tout exercice s'enregistre ; sa validation
/// rend `FiscalYearInvalid` et n'écrit rien.
#[sqlx::test(migrations = "./test-schema")]
async fn a_draft_outside_any_fiscal_year_is_accepted_then_refused_at_validation(pool: MySqlPool) {
    let (seeded, f) = decor(&pool).await;
    let d = supplier_invoices::create_draft(
        &pool,
        nouvelle(
            &seeded,
            f,
            "FF-HORS",
            jour(2035, 1, 10),
            &[(dec!(50), dec!(0))],
        ),
        seeded.admin_user_id,
    )
    .await
    .expect("brouillon hors exercice");
    let avant = nb_ecritures(&pool, seeded.company_id).await;
    let err = supplier_invoices::validate(
        &pool,
        seeded.company_id,
        d.invoice.id,
        d.invoice.version,
        seeded.admin_user_id,
    )
    .await
    .expect_err("aucun exercice ne couvre la date");
    assert!(matches!(err, DbError::FiscalYearInvalid), "{err:?}");
    assert_eq!(nb_ecritures(&pool, seeded.company_id).await, avant);
    assert_eq!(etat(&pool, d.invoice.id).await.0, "draft");
}

/// AC-6 — les contrôles de saisie restent à l'enregistrement : un compte de
/// charge d'un autre type est refusé avec le code actuel.
#[sqlx::test(migrations = "./test-schema")]
async fn a_draft_refuses_a_non_expense_account(pool: MySqlPool) {
    let (seeded, f) = decor(&pool).await;
    let mut new = nouvelle(
        &seeded,
        f,
        "FF-X",
        jour(2026, 6, 15),
        &[(dec!(10), dec!(0))],
    );
    new.lines[0].expense_account_id = seeded.accounts["1100"];
    let err = supplier_invoices::create_draft(&pool, new, seeded.admin_user_id)
        .await
        .expect_err("compte d'actif");
    assert!(matches!(err, DbError::InactiveOrInvalidAccounts), "{err:?}");
}

/// AC-6 (a) — sans compte de TVA récupérable, le brouillon à 8.1 % s'enregistre
/// (TTC compris) ; sa validation rend `ConfigurationRequired`.
#[sqlx::test(migrations = "./test-schema")]
async fn a_vat_draft_without_recoverable_account_needs_it_only_at_validation(pool: MySqlPool) {
    let (seeded, f) = decor(&pool).await;
    sqlx::query(
        "UPDATE company_invoice_settings SET default_vat_recoverable_account_id = NULL \
         WHERE company_id = ?",
    )
    .bind(seeded.company_id)
    .execute(&pool)
    .await
    .unwrap();
    let d = supplier_invoices::create_draft(
        &pool,
        nouvelle(
            &seeded,
            f,
            "FF-TVA",
            jour(2026, 6, 15),
            &[(dec!(1000), dec!(8.1))],
        ),
        seeded.admin_user_id,
    )
    .await
    .expect("brouillon à TVA");
    assert_eq!(d.invoice.total_amount, dec!(1081.00));
    let err = supplier_invoices::validate(
        &pool,
        seeded.company_id,
        d.invoice.id,
        d.invoice.version,
        seeded.admin_user_id,
    )
    .await
    .expect_err("compte de TVA récupérable requis");
    assert!(
        matches!(&err, DbError::ConfigurationRequired(k) if k == "default_vat_recoverable_account_id"),
        "{err:?}"
    );
}

// ---------------------------------------------------------------------------
// AC-8 — modifier un brouillon
// ---------------------------------------------------------------------------

/// AC-8 — `update_draft` remplace en-tête et lignes (trois puis une), `version
/// + 1`, aucune écriture, audit avant/après ; puis la validation porte le total
/// corrigé (AC-6 (b) : `total_amount` = crédit des créanciers).
#[sqlx::test(migrations = "./test-schema")]
async fn update_draft_replaces_everything_and_validation_follows(pool: MySqlPool) {
    let (seeded, f) = decor(&pool).await;
    let d = supplier_invoices::create_draft(
        &pool,
        nouvelle(
            &seeded,
            f,
            "FF-U",
            jour(2026, 6, 15),
            &[
                (dec!(10), dec!(0)),
                (dec!(20), dec!(0)),
                (dec!(30), dec!(0)),
            ],
        ),
        seeded.admin_user_id,
    )
    .await
    .unwrap();
    let avant = nb_ecritures(&pool, seeded.company_id).await;
    let mut corrige = nouvelle(
        &seeded,
        f,
        "FF-U2",
        jour(2026, 6, 20),
        &[(dec!(100.20), dec!(8.1))],
    );
    corrige.due_date = Some(jour(2026, 7, 20));
    let u = supplier_invoices::update_draft(
        &pool,
        seeded.company_id,
        d.invoice.id,
        d.invoice.version,
        corrige,
        seeded.admin_user_id,
    )
    .await
    .expect("brouillon modifié");
    assert_eq!(u.invoice.version, d.invoice.version + 1);
    assert_eq!(u.invoice.supplier_invoice_number.as_deref(), Some("FF-U2"));
    assert_eq!(u.invoice.invoice_date, jour(2026, 6, 20));
    assert_eq!(u.invoice.due_date, Some(jour(2026, 7, 20)));
    assert_eq!(u.invoice.total_amount, dec!(108.32));
    assert_eq!(u.lines.len(), 1, "lignes orphelines absentes");
    assert_eq!(u.lines[0].position, 0);
    let nb_lignes: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM supplier_invoice_lines WHERE supplier_invoice_id = ?",
    )
    .bind(d.invoice.id)
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(nb_lignes, 1);
    assert_eq!(nb_ecritures(&pool, seeded.company_id).await, avant);
    assert_eq!(
        nb_audits(&pool, "supplier_invoice.updated", d.invoice.id).await,
        1
    );

    let v = supplier_invoices::validate(
        &pool,
        seeded.company_id,
        d.invoice.id,
        u.invoice.version,
        seeded.admin_user_id,
    )
    .await
    .expect("validation");
    let credit: Decimal = sqlx::query_scalar(
        "SELECT credit FROM journal_entry_lines WHERE entry_id = ? AND credit > 0",
    )
    .bind(v.invoice.purchase_journal_entry_id.unwrap())
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(credit, v.invoice.total_amount);
}

/// AC-8 — refus : facture validée (`IllegalStateTransition`, M8), version
/// périmée (`OptimisticLockConflict`), facture inconnue (`NotFound`).
#[sqlx::test(migrations = "./test-schema")]
async fn update_draft_refusals(pool: MySqlPool) {
    let (seeded, f) = decor(&pool).await;
    let v = validee(&pool, &seeded, f, "FF-V", dec!(10)).await;
    let err = supplier_invoices::update_draft(
        &pool,
        seeded.company_id,
        v.invoice.id,
        v.invoice.version,
        nouvelle(
            &seeded,
            f,
            "FF-V",
            jour(2026, 6, 15),
            &[(dec!(11), dec!(0))],
        ),
        seeded.admin_user_id,
    )
    .await
    .expect_err("validée");
    assert!(
        matches!(err, DbError::IllegalStateTransition(_)),
        "M8 : {err:?}"
    );

    let d = brouillon(&pool, &seeded, f, "FF-D", dec!(10)).await;
    let err = supplier_invoices::update_draft(
        &pool,
        seeded.company_id,
        d.invoice.id,
        d.invoice.version + 7,
        nouvelle(
            &seeded,
            f,
            "FF-D",
            jour(2026, 6, 15),
            &[(dec!(11), dec!(0))],
        ),
        seeded.admin_user_id,
    )
    .await
    .expect_err("version périmée");
    assert!(matches!(err, DbError::OptimisticLockConflict), "{err:?}");

    let err = supplier_invoices::update_draft(
        &pool,
        seeded.company_id + 1000,
        d.invoice.id,
        d.invoice.version,
        nouvelle(
            &seeded,
            f,
            "FF-D",
            jour(2026, 6, 15),
            &[(dec!(11), dec!(0))],
        ),
        seeded.admin_user_id,
    )
    .await
    .expect_err("autre société");
    assert!(matches!(err, DbError::NotFound), "{err:?}");
}

// ---------------------------------------------------------------------------
// AC-10 — valider
// ---------------------------------------------------------------------------

/// AC-10 — l'écriture d'achat est celle qu'aurait passée la création de la rc1,
/// **écrite en dur** : D 4000 1000.00, D 1000 (TVA récupérable) 81.00,
/// C 2000 1081.00, journal Achats, date de facture, libellé. Tue M2.
#[sqlx::test(migrations = "./test-schema")]
async fn validation_passes_the_rc1_purchase_entry(pool: MySqlPool) {
    let (seeded, f) = decor(&pool).await;
    let d = supplier_invoices::create_draft(
        &pool,
        nouvelle(
            &seeded,
            f,
            "FF-G",
            jour(2026, 6, 15),
            &[(dec!(1000), dec!(8.1))],
        ),
        seeded.admin_user_id,
    )
    .await
    .unwrap();
    let v = supplier_invoices::validate(
        &pool,
        seeded.company_id,
        d.invoice.id,
        d.invoice.version,
        seeded.admin_user_id,
    )
    .await
    .expect("validation");
    assert_eq!(v.invoice.status, "open");
    assert_eq!(v.invoice.version, d.invoice.version + 1);
    let je = v
        .invoice
        .purchase_journal_entry_id
        .expect("M2 : écriture posée");
    let (journal, date, libelle): (String, NaiveDate, String) =
        sqlx::query_as("SELECT journal, entry_date, description FROM journal_entries WHERE id = ?")
            .bind(je)
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(journal, "Achats");
    assert_eq!(date, jour(2026, 6, 15));
    assert_eq!(
        libelle,
        "Facture fournisseur FF-G - Fournisseur rapproché SA"
    );
    let lignes: Vec<(i64, Decimal, Decimal, Option<i64>)> = sqlx::query_as(
        "SELECT account_id, debit, credit, project_id FROM journal_entry_lines \
         WHERE entry_id = ? ORDER BY line_order",
    )
    .bind(je)
    .fetch_all(&pool)
    .await
    .unwrap();
    assert_eq!(
        lignes,
        vec![
            (seeded.accounts["4000"], dec!(1000.0000), dec!(0.0000), None),
            (seeded.accounts["1000"], dec!(81.0000), dec!(0.0000), None),
            (seeded.accounts["2000"], dec!(0.0000), dec!(1081.0000), None),
        ]
    );
    assert_eq!(
        nb_audits(&pool, "supplier_invoice.validated", d.invoice.id).await,
        1
    );
}

/// AC-10 — refus : non brouillon, version périmée ; un refus n'écrit rien.
#[sqlx::test(migrations = "./test-schema")]
async fn validation_refusals_write_nothing(pool: MySqlPool) {
    let (seeded, f) = decor(&pool).await;
    let v = validee(&pool, &seeded, f, "FF-V", dec!(10)).await;
    let err = supplier_invoices::validate(
        &pool,
        seeded.company_id,
        v.invoice.id,
        v.invoice.version,
        seeded.admin_user_id,
    )
    .await
    .expect_err("déjà validée");
    assert!(matches!(err, DbError::IllegalStateTransition(_)), "{err:?}");

    let d = brouillon(&pool, &seeded, f, "FF-D", dec!(10)).await;
    let avant = nb_ecritures(&pool, seeded.company_id).await;
    let err = supplier_invoices::validate(
        &pool,
        seeded.company_id,
        d.invoice.id,
        d.invoice.version + 1,
        seeded.admin_user_id,
    )
    .await
    .expect_err("version périmée");
    assert!(matches!(err, DbError::OptimisticLockConflict), "{err:?}");
    assert_eq!(nb_ecritures(&pool, seeded.company_id).await, avant);
    assert_eq!(etat(&pool, d.invoice.id).await, ("draft".into(), None, 1));
}

/// AC-10 — **course pilotée** (M20) : A valide le brouillon sans valider sa
/// transaction ; B, seconde validation, attend la facture ; A valide ; B relit
/// `open` sous le verrou et rend `IllegalStateTransition`. Sans le verrou
/// initial, B aurait lu `draft` par une lecture ordinaire et rendrait
/// `OptimisticLockConflict`.
#[sqlx::test(migrations = "./test-schema")]
async fn concurrent_validations_the_loser_reads_validated_under_the_lock(pool: MySqlPool) {
    let (seeded, f) = decor(&pool).await;
    let d = brouillon(&pool, &seeded, f, "FF-C", dec!(40)).await;
    let (c, id, v, u) = (
        seeded.company_id,
        d.invoice.id,
        d.invoice.version,
        seeded.admin_user_id,
    );
    let mut a = pool.begin().await.unwrap();
    supplier_invoices::validate_in_tx(&mut a, c, id, v, u)
        .await
        .expect("A valide");
    let pool_b = pool.clone();
    let b = tokio::spawn(async move { supplier_invoices::validate(&pool_b, c, id, v, u).await });
    let vu = attendre_une_requete_en_cours(&pool, &["FOR UPDATE"], || b.is_finished()).await;
    assert!(vu, "B devait attendre un verrou tenu par A");
    a.commit().await.unwrap();
    let b = tokio::time::timeout(BORNE, b)
        .await
        .expect("B aboutit")
        .unwrap();
    let err = b.expect_err("B perd la course");
    assert!(
        matches!(err, DbError::IllegalStateTransition(_)),
        "M20 : le perdant doit relire `open` sous le verrou : {err:?}"
    );
    let n: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM journal_entries WHERE company_id = ? AND journal = 'Achats'",
    )
    .bind(c)
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(n, 1, "une seule écriture d'achat");
}

// ---------------------------------------------------------------------------
// AC-11 — dévalider
// ---------------------------------------------------------------------------

/// AC-11 — l'écriture d'achat est **supprimée** (−1 écriture, aucune
/// contre-passation), la facture redevient brouillon, `version + 1`, audits
/// `journal_entry.deleted` et `supplier_invoice.unvalidated`. Tue M5 et M19.
#[sqlx::test(migrations = "./test-schema")]
async fn unvalidation_deletes_the_purchase_entry(pool: MySqlPool) {
    let (seeded, f) = decor(&pool).await;
    let v = validee(&pool, &seeded, f, "FF-DV", dec!(70)).await;
    let je = v.invoice.purchase_journal_entry_id.unwrap();
    let avant = nb_ecritures(&pool, seeded.company_id).await;
    let u = supplier_invoices::unvalidate(
        &pool,
        seeded.company_id,
        v.invoice.id,
        v.invoice.version,
        seeded.admin_user_id,
    )
    .await
    .expect("dévalidation");
    assert_eq!(u.invoice.status, "draft");
    assert_eq!(u.invoice.purchase_journal_entry_id, None);
    assert_eq!(u.invoice.version, v.invoice.version + 1);
    assert_eq!(
        nb_ecritures(&pool, seeded.company_id).await,
        avant - 1,
        "M5"
    );
    let reste: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM journal_entries WHERE id = ?")
        .bind(je)
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(reste, 0);
    assert_eq!(nb_audits(&pool, "journal_entry.deleted", je).await, 1);
    let details: serde_json::Value = sqlx::query_scalar(
        "SELECT details FROM audit_log WHERE action = 'supplier_invoice.unvalidated' \
         AND entity_id = ?",
    )
    .bind(v.invoice.id)
    .fetch_one(&pool)
    .await
    .expect("M19 : audit de la dévalidation");
    assert_eq!(details["purchaseJournalEntryId"], je, "{details}");
}

fn bloque(err: &DbError) -> Option<SupplierUnvalidationBlocker> {
    match err {
        DbError::SupplierInvoiceNotUnvalidatable { blocker, .. } => Some(*blocker),
        _ => None,
    }
}

/// AC-11 — statut : payée → `SUPPLIER_INVOICE_PAID` (M6) ; brouillon et
/// annulée → `IllegalStateTransition` ; puis la version (après le statut).
#[sqlx::test(migrations = "./test-schema")]
async fn unvalidation_status_refusals(pool: MySqlPool) {
    let (seeded, f) = decor(&pool).await;
    let c = seeded.company_id;
    let u = seeded.admin_user_id;
    let (paid, _, _) = fournisseur_rapproche::facture_payee(
        &pool,
        &seeded,
        f,
        "FF-P",
        jour(2026, 6, 15),
        dec!(30),
    )
    .await;
    let version: i32 = sqlx::query_scalar("SELECT version FROM supplier_invoices WHERE id = ?")
        .bind(paid)
        .fetch_one(&pool)
        .await
        .unwrap();
    let err = supplier_invoices::unvalidate(&pool, c, paid, version, u)
        .await
        .expect_err("payée");
    assert_eq!(
        bloque(&err),
        Some(SupplierUnvalidationBlocker::Paid),
        "M6 : {err:?}"
    );
    // Le statut parle avant la version.
    let err = supplier_invoices::unvalidate(&pool, c, paid, version + 3, u)
        .await
        .expect_err("payée, version périmée");
    assert_eq!(
        bloque(&err),
        Some(SupplierUnvalidationBlocker::Paid),
        "{err:?}"
    );

    let d = brouillon(&pool, &seeded, f, "FF-D", dec!(10)).await;
    let err = supplier_invoices::unvalidate(&pool, c, d.invoice.id, d.invoice.version, u)
        .await
        .expect_err("brouillon");
    assert!(matches!(err, DbError::IllegalStateTransition(_)), "{err:?}");

    let x = validee(&pool, &seeded, f, "FF-X", dec!(10)).await;
    supplier_invoices::cancel(&pool, c, x.invoice.id, u)
        .await
        .unwrap();
    let err = supplier_invoices::unvalidate(&pool, c, x.invoice.id, x.invoice.version + 1, u)
        .await
        .expect_err("annulée");
    assert!(matches!(err, DbError::IllegalStateTransition(_)), "{err:?}");

    let o = validee(&pool, &seeded, f, "FF-O", dec!(10)).await;
    let err = supplier_invoices::unvalidate(&pool, c, o.invoice.id, o.invoice.version + 1, u)
        .await
        .expect_err("version périmée");
    assert!(matches!(err, DbError::OptimisticLockConflict), "{err:?}");
}

/// Place la facture dans un lot `generated` ; rend l'id du lot.
async fn en_lot(pool: &MySqlPool, seeded: &SeededCompany, f: Fournisseur, id: i64) -> i64 {
    let outcome = payment_batches::create_batch(
        pool,
        NewPaymentBatch {
            company_id: seeded.company_id,
            bank_account_id: f.bank_account_id,
            requested_execution_date: jour(2026, 6, 25),
            supplier_invoice_ids: vec![id],
        },
        seeded.admin_user_id,
    )
    .await
    .expect("lot");
    assert!(outcome.failed.is_empty(), "{:?}", outcome.failed);
    outcome.batch.expect("lot créé").batch.id
}

/// Rapproche l'écriture d'achat d'un débit ; rend la transaction.
async fn rapprochee(pool: &MySqlPool, seeded: &SeededCompany, f: Fournisseur, je: i64) -> i64 {
    fournisseur_rapproche::debit_rapproche(pool, seeded, f, je, jour(2026, 6, 15), dec!(1)).await
}

/// AC-11 — précédence, paire par paire : version → lot `generated` (M7 : la
/// paire « version périmée et lot » rend la version ; M4 : le lot) → écriture
/// rapprochée (M3 : le lien reste) → exercice clos.
#[sqlx::test(migrations = "./test-schema")]
async fn unvalidation_precedence_version_batch_match_fiscal_year(pool: MySqlPool) {
    let (seeded, f) = decor(&pool).await;
    let c = seeded.company_id;
    let u = seeded.admin_user_id;
    let v = validee(&pool, &seeded, f, "FF-PR", dec!(90)).await;
    let id = v.invoice.id;
    let je = v.invoice.purchase_journal_entry_id.unwrap();
    en_lot(&pool, &seeded, f, id).await;
    let version: i32 = sqlx::query_scalar("SELECT version FROM supplier_invoices WHERE id = ?")
        .bind(id)
        .fetch_one(&pool)
        .await
        .unwrap();

    let err = supplier_invoices::unvalidate(&pool, c, id, version + 1, u)
        .await
        .expect_err("version périmée et lot");
    assert!(
        matches!(err, DbError::OptimisticLockConflict),
        "M7 : {err:?}"
    );

    let tx = rapprochee(&pool, &seeded, f, je).await;
    let err = supplier_invoices::unvalidate(&pool, c, id, version, u)
        .await
        .expect_err("lot et rapprochement");
    assert_eq!(
        bloque(&err),
        Some(SupplierUnvalidationBlocker::InPaymentBatch),
        "M4 : {err:?}"
    );
    sqlx::query("UPDATE payment_batches SET status = 'cancelled' WHERE company_id = ?")
        .bind(c)
        .execute(&pool)
        .await
        .unwrap();

    sqlx::query("UPDATE fiscal_years SET status = 'Closed' WHERE id = ?")
        .bind(seeded.fiscal_year_id)
        .execute(&pool)
        .await
        .unwrap();
    let err = supplier_invoices::unvalidate(&pool, c, id, version, u)
        .await
        .expect_err("rapprochement et exercice clos");
    assert_eq!(
        bloque(&err),
        Some(SupplierUnvalidationBlocker::MatchedBankTransaction),
        "M3 : {err:?}"
    );
    let lien: Option<i64> =
        sqlx::query_scalar("SELECT matched_entry_id FROM bank_transactions WHERE id = ?")
            .bind(tx)
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(lien, Some(je), "la transaction garde son lien");

    sqlx::query(
        "UPDATE bank_transactions SET matched_entry_id = NULL, status = 'unmatched' WHERE id = ?",
    )
    .bind(tx)
    .execute(&pool)
    .await
    .unwrap();
    let err = supplier_invoices::unvalidate(&pool, c, id, version, u)
        .await
        .expect_err("exercice clos");
    assert!(matches!(err, DbError::FiscalYearClosed), "{err:?}");
}

/// AC-11 — précédence dans `delete_in_tx` : exercice postérieur clos, puis
/// période verrouillée ; chaque refus laisse la facture validée.
#[sqlx::test(migrations = "./test-schema")]
async fn unvalidation_precedence_later_year_then_period_lock(pool: MySqlPool) {
    let (seeded, f) = decor(&pool).await;
    let c = seeded.company_id;
    let u = seeded.admin_user_id;
    let v = validee(&pool, &seeded, f, "FF-L", dec!(15)).await;
    let id = v.invoice.id;
    // Exercice postérieur clos.
    sqlx::query(
        "INSERT INTO fiscal_years (company_id, name, start_date, end_date, status) \
         VALUES (?, 'Exercice 2031', '2031-01-01', '2031-12-31', 'Closed')",
    )
    .bind(c)
    .execute(&pool)
    .await
    .unwrap();
    sqlx::query("UPDATE companies SET books_locked_through = '2026-12-31' WHERE id = ?")
        .bind(c)
        .execute(&pool)
        .await
        .unwrap();
    let err = supplier_invoices::unvalidate(&pool, c, id, v.invoice.version, u)
        .await
        .expect_err("exercice postérieur clos");
    assert!(
        matches!(err, DbError::LaterFiscalYearClosed { .. }),
        "{err:?}"
    );
    sqlx::query("DELETE FROM fiscal_years WHERE company_id = ? AND name = 'Exercice 2031'")
        .bind(c)
        .execute(&pool)
        .await
        .unwrap();
    let err = supplier_invoices::unvalidate(&pool, c, id, v.invoice.version, u)
        .await
        .expect_err("période verrouillée");
    assert!(matches!(err, DbError::PeriodLocked { .. }), "{err:?}");
    assert_eq!(etat(&pool, id).await.0, "open");
}

/// AC-11 — de bout en bout : payée, règlement annulé, dévalidée, modifiée,
/// revalidée.
#[sqlx::test(migrations = "./test-schema")]
async fn paid_then_settlement_cancelled_then_unvalidated(pool: MySqlPool) {
    let (seeded, f) = decor(&pool).await;
    let c = seeded.company_id;
    let u = seeded.admin_user_id;
    let (id, _, _) = fournisseur_rapproche::facture_payee(
        &pool,
        &seeded,
        f,
        "FF-BEB",
        jour(2026, 6, 15),
        dec!(55),
    )
    .await;
    let annule = supplier_invoices::cancel_settlement(&pool, c, id, u)
        .await
        .expect("règlement annulé");
    let d = supplier_invoices::unvalidate(&pool, c, id, annule.invoice.invoice.version, u)
        .await
        .expect("dévalidée");
    let m = supplier_invoices::update_draft(
        &pool,
        c,
        id,
        d.invoice.version,
        nouvelle(
            &seeded,
            f,
            "FF-BEB",
            jour(2026, 6, 15),
            &[(dec!(50), dec!(0))],
        ),
        u,
    )
    .await
    .expect("modifiée");
    let r = supplier_invoices::validate(&pool, c, id, m.invoice.version, u)
        .await
        .expect("revalidée");
    assert_eq!(r.invoice.status, "open");
    assert_eq!(r.invoice.total_amount, dec!(50));
}

// ---------------------------------------------------------------------------
// AC-9 — supprimer un brouillon
// ---------------------------------------------------------------------------

async fn document(pool: &MySqlPool, company_id: i64, hash: &str) -> i64 {
    imported_supplier_invoices::create(
        pool,
        &NewImportedSupplierInvoice {
            company_id,
            file_hash: hash.into(),
            storage_path: format!("{hash}.pdf"),
            original_filename: format!("{hash}.pdf"),
            mime_type: "application/pdf".into(),
            byte_size: 10,
            creditor_iban: None,
            is_qr_iban: None,
            creditor_address_type: None,
            creditor_name: None,
            creditor_line1: None,
            creditor_line2: None,
            creditor_postal_code: None,
            creditor_town: None,
            creditor_country: None,
            reference_type: None,
            reference_value: None,
            amount: None,
            currency: None,
            unstructured_message: None,
            billing_information: None,
        },
    )
    .await
    .expect("document")
    .id
}

async fn joindre(pool: &MySqlPool, company_id: i64, doc: i64, invoice: i64) {
    let mut tx = pool.begin().await.unwrap();
    imported_supplier_invoices::attach_in_tx(&mut tx, company_id, doc, invoice)
        .await
        .unwrap()
        .expect("rattachement");
    tx.commit().await.unwrap();
}

async fn statut_document(pool: &MySqlPool, doc: i64) -> (String, Option<i64>, Option<String>) {
    sqlx::query_as(
        "SELECT status, supplier_invoice_id, link_origin FROM imported_supplier_invoices \
         WHERE id = ?",
    )
    .bind(doc)
    .fetch_one(pool)
    .await
    .unwrap()
}

/// AC-9 — un brouillon se supprime : facture et lignes parties, document rendu
/// « à traiter », lien et origine nuls, audits `detached` puis `deleted`. Tue M9.
#[sqlx::test(migrations = "./test-schema")]
async fn deleting_a_draft_releases_its_document(pool: MySqlPool) {
    let (seeded, f) = decor(&pool).await;
    let c = seeded.company_id;
    let d = brouillon(&pool, &seeded, f, "FF-S", dec!(12)).await;
    let doc = document(&pool, c, "s1").await;
    joindre(&pool, c, doc, d.invoice.id).await;
    assert_eq!(
        statut_document(&pool, doc).await,
        (
            "completed".into(),
            Some(d.invoice.id),
            Some("attachment".into())
        ),
        "AC-7 : le rattachement pose `attachment`"
    );
    supplier_invoices::delete_draft(&pool, c, d.invoice.id, seeded.admin_user_id)
        .await
        .expect("suppression");
    let reste: i64 = sqlx::query_scalar(
        "SELECT (SELECT COUNT(*) FROM supplier_invoices WHERE id = ?) + \
                (SELECT COUNT(*) FROM supplier_invoice_lines WHERE supplier_invoice_id = ?)",
    )
    .bind(d.invoice.id)
    .bind(d.invoice.id)
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(reste, 0);
    assert_eq!(
        statut_document(&pool, doc).await,
        ("to_complete".into(), None, None),
        "M9"
    );
    let actions: Vec<String> = sqlx::query_scalar(
        "SELECT action FROM audit_log WHERE action IN \
         ('imported_supplier_invoice.detached', 'supplier_invoice.deleted') ORDER BY id",
    )
    .fetch_all(&pool)
    .await
    .unwrap();
    assert_eq!(
        actions,
        vec![
            "imported_supplier_invoice.detached".to_string(),
            "supplier_invoice.deleted".to_string()
        ]
    );
}

/// AC-9 — refus : validée → `MustBeUnvalidatedFirst`, le document reste lié ;
/// payée, annulée → `IllegalStateTransition` ; brouillon à historique de lot →
/// `HasPaymentBatchHistory`, jamais une erreur de clé étrangère (M10).
#[sqlx::test(migrations = "./test-schema")]
async fn deleting_refusals(pool: MySqlPool) {
    let (seeded, f) = decor(&pool).await;
    let c = seeded.company_id;
    let u = seeded.admin_user_id;
    let v = validee(&pool, &seeded, f, "FF-O", dec!(12)).await;
    let doc = document(&pool, c, "o1").await;
    joindre(&pool, c, doc, v.invoice.id).await;
    let err = supplier_invoices::delete_draft(&pool, c, v.invoice.id, u)
        .await
        .expect_err("validée");
    assert!(
        matches!(err, DbError::SupplierInvoiceMustBeUnvalidatedFirst),
        "{err:?}"
    );
    assert_eq!(statut_document(&pool, doc).await.0, "completed");
    assert_eq!(statut_document(&pool, doc).await.1, Some(v.invoice.id));

    let (paid, _, _) =
        fournisseur_rapproche::facture_payee(&pool, &seeded, f, "FF-P", jour(2026, 6, 15), dec!(8))
            .await;
    let err = supplier_invoices::delete_draft(&pool, c, paid, u)
        .await
        .expect_err("payée");
    assert!(matches!(err, DbError::IllegalStateTransition(_)), "{err:?}");

    let x = validee(&pool, &seeded, f, "FF-X", dec!(9)).await;
    supplier_invoices::cancel(&pool, c, x.invoice.id, u)
        .await
        .unwrap();
    let err = supplier_invoices::delete_draft(&pool, c, x.invoice.id, u)
        .await
        .expect_err("annulée");
    assert!(matches!(err, DbError::IllegalStateTransition(_)), "{err:?}");

    // Validée, placée dans un lot, lot annulé, puis dévalidée (limite L8).
    let l = validee(&pool, &seeded, f, "FF-L8", dec!(7)).await;
    let lot = en_lot(&pool, &seeded, f, l.invoice.id).await;
    payment_batches::cancel_batch(&pool, c, lot, u)
        .await
        .unwrap();
    let version: i32 = sqlx::query_scalar("SELECT version FROM supplier_invoices WHERE id = ?")
        .bind(l.invoice.id)
        .fetch_one(&pool)
        .await
        .unwrap();
    supplier_invoices::unvalidate(&pool, c, l.invoice.id, version, u)
        .await
        .expect("dévalidée, lot annulé");
    let err = supplier_invoices::delete_draft(&pool, c, l.invoice.id, u)
        .await
        .expect_err("historique de lot");
    assert!(
        matches!(err, DbError::SupplierInvoiceHasPaymentBatchHistory { payment_batch_id } if payment_batch_id == lot),
        "M10 : {err:?}"
    );
}

/// AC-9 — **test piloté** de l'ordre des verrous (M20-quater ; C-v017-4-38,
/// 41). A rejoue à la main les étapes (1) à (3) de `attach_in_tx` — document
/// neuf `FOR UPDATE`, facture `FOR UPDATE`, sentinelle — sans `mark_completed`.
/// B = `delete_draft` du **dépôt**, sans enveloppe de rejeu, dans une tâche ;
/// B attend la facture ; A pose alors le lien et valide. Sous l'ordre prescrit
/// (facture puis document), aucun interblocage : A aboutit, B relit la facture
/// et la supprime en rendant le document lié par A « à traiter ». Sous
/// M20-quater, B tient l'intervalle de `uq_imported_supplier_invoice` avant
/// d'attendre la facture, et `mark_completed` de A l'attend : interblocage,
/// quelle que soit la victime.
#[sqlx::test(migrations = "./test-schema")]
async fn deleting_a_draft_during_an_attachment_does_not_deadlock(pool: MySqlPool) {
    let (seeded, f) = decor(&pool).await;
    let c = seeded.company_id;
    let d = brouillon(&pool, &seeded, f, "FF-PIL", dec!(5)).await;
    let id = d.invoice.id;
    let doc = document(&pool, c, "pil").await;

    let mut a = pool.begin().await.unwrap();
    let _: Option<i64> = sqlx::query_scalar(
        "SELECT id FROM imported_supplier_invoices WHERE id = ? AND company_id = ? FOR UPDATE",
    )
    .bind(doc)
    .bind(c)
    .fetch_optional(&mut *a)
    .await
    .unwrap();
    let _: Option<i64> = sqlx::query_scalar(
        "SELECT id FROM supplier_invoices WHERE id = ? AND company_id = ? FOR UPDATE",
    )
    .bind(id)
    .bind(c)
    .fetch_optional(&mut *a)
    .await
    .unwrap();
    let _: Option<i64> = sqlx::query_scalar("SELECT id FROM companies WHERE id = ? FOR UPDATE")
        .bind(c)
        .fetch_optional(&mut *a)
        .await
        .unwrap();

    let pool_b = pool.clone();
    let user = seeded.admin_user_id;
    let b =
        tokio::spawn(async move { supplier_invoices::delete_draft(&pool_b, c, id, user).await });
    let vu = attendre_une_requete_en_cours(
        &pool,
        &["FROM supplier_invoices WHERE id", "FOR UPDATE"],
        || b.is_finished(),
    )
    .await;
    assert!(vu, "B devait attendre la facture tenue par A");

    let a_result =
        imported_supplier_invoices::mark_completed(&mut a, c, doc, id, LinkOrigin::Attachment)
            .await;
    assert!(
        a_result.is_ok(),
        "A ne doit subir aucun interblocage : {a_result:?}"
    );
    a.commit().await.unwrap();
    let b = tokio::time::timeout(BORNE, b)
        .await
        .expect("B aboutit")
        .unwrap();
    assert!(b.is_ok(), "B ne doit subir aucun interblocage : {b:?}");
    assert_eq!(
        statut_document(&pool, doc).await,
        ("to_complete".into(), None, None),
        "le document lié par A est rendu « à traiter » par la suppression"
    );
    let reste: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM supplier_invoices WHERE id = ?")
        .bind(id)
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(reste, 0);
}

// ---------------------------------------------------------------------------
// AC-12, AC-13
// ---------------------------------------------------------------------------

/// AC-12 — `create_validated` : un brouillon puis sa validation, dans une
/// transaction ; version 2, écriture posée.
#[sqlx::test(migrations = "./test-schema")]
async fn create_validated_is_the_composition(pool: MySqlPool) {
    let (seeded, f) = decor(&pool).await;
    let v = validee(&pool, &seeded, f, "FF-CV", dec!(33)).await;
    assert_eq!(v.invoice.status, "open");
    assert_eq!(v.invoice.version, 2);
    assert!(v.invoice.purchase_journal_entry_id.is_some());
    assert_eq!(
        nb_audits(&pool, "supplier_invoice.created", v.invoice.id).await,
        1
    );
    assert_eq!(
        nb_audits(&pool, "supplier_invoice.validated", v.invoice.id).await,
        1
    );
}

/// AC-13 — un brouillon ne s'annule pas (rang 1), rien n'est écrit ; la vue le
/// dit. Tue M18.
#[sqlx::test(migrations = "./test-schema")]
async fn a_draft_cannot_be_cancelled(pool: MySqlPool) {
    let (seeded, f) = decor(&pool).await;
    let d = brouillon(&pool, &seeded, f, "FF-NC", dec!(6)).await;
    let avant = nb_ecritures(&pool, seeded.company_id).await;
    let err =
        supplier_invoices::cancel(&pool, seeded.company_id, d.invoice.id, seeded.admin_user_id)
            .await
            .expect_err("brouillon");
    assert!(
        matches!(
            err,
            DbError::SupplierInvoiceNotCancellable {
                blocker: SettlementCancelBlocker::SupplierInvoiceDraft
            }
        ),
        "M18 : {err:?}"
    );
    assert_eq!(nb_ecritures(&pool, seeded.company_id).await, avant);
    assert_eq!(etat(&pool, d.invoice.id).await.0, "draft");
    let vue = supplier_invoices::get_settlement_view(&pool, seeded.company_id, d.invoice.id)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(
        vue.invoice_cancel_blocker.map(|h| h.0),
        Some(SettlementCancelBlocker::SupplierInvoiceDraft)
    );
}

/// AC-5 (5) — `pay` d'un brouillon : `IllegalStateTransition`, rien d'écrit.
#[sqlx::test(migrations = "./test-schema")]
async fn a_draft_cannot_be_paid(pool: MySqlPool) {
    let (seeded, f) = decor(&pool).await;
    let d = brouillon(&pool, &seeded, f, "FF-NP", dec!(6)).await;
    let err = supplier_invoices::pay(
        &pool,
        seeded.company_id,
        d.invoice.id,
        SettlementChoice::BankTransfer {
            bank_account_id: f.bank_account_id,
        },
        jour(2026, 6, 20),
        seeded.admin_user_id,
    )
    .await
    .expect_err("brouillon");
    match err {
        DbError::IllegalStateTransition(msg) => {
            assert!(msg.contains("validée"), "message : {msg}")
        }
        other => panic!("{other:?}"),
    }
}
