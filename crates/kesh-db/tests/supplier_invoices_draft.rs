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
        "SELECT details_json FROM audit_log WHERE action = 'supplier_invoice.created' AND entity_id = ?",
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
        "Facture fournisseur FF-2020-001 (FF-G) - Fournisseur rapproché SA"
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

/// Insère un projet analytique (non archivé) et rend son id.
async fn projet(pool: &MySqlPool, company_id: i64, code: &str) -> i64 {
    sqlx::query(
        "INSERT INTO projects (company_id, code, name, archived, version) VALUES (?, ?, ?, FALSE, 0)",
    )
    .bind(company_id)
    .bind(code)
    .bind(code)
    .execute(pool)
    .await
    .unwrap()
    .last_insert_id() as i64
}

/// Valide le brouillon `d` et attend un refus ; vérifie que rien n'est écrit.
async fn validation_refusee(
    pool: &MySqlPool,
    seeded: &SeededCompany,
    d: &supplier_invoices::SupplierInvoiceWithLines,
) -> DbError {
    let avant = nb_ecritures(pool, seeded.company_id).await;
    let err = supplier_invoices::validate(
        pool,
        seeded.company_id,
        d.invoice.id,
        d.invoice.version,
        seeded.admin_user_id,
    )
    .await
    .expect_err("validation refusée");
    assert_eq!(
        nb_ecritures(pool, seeded.company_id).await,
        avant,
        "aucune écriture"
    );
    assert_eq!(etat(pool, d.invoice.id).await, ("draft".into(), None, 1));
    assert_eq!(
        nb_audits(pool, "supplier_invoice.validated", d.invoice.id).await,
        0
    );
    err
}

/// Pose `colonne` (`active` ou `postable`) du compte `compte`.
async fn poser_compte(pool: &MySqlPool, compte: i64, colonne: &str, valeur: bool) {
    sqlx::query(&format!("UPDATE accounts SET {colonne} = ? WHERE id = ?"))
        .bind(valeur)
        .bind(compte)
        .execute(pool)
        .await
        .unwrap();
}

/// AC-10, C-v017-4-48 (revue P1, A-3) — la validation **refait sous verrou** les
/// contrôles de saisie : un compte de charge devenu inactif, puis non imputable,
/// puis un projet archivé **depuis l'enregistrement du brouillon** sont refusés à
/// la validation, qui n'écrit rien. Sans ce re-contrôle, le brouillon passerait
/// avec un compte ou un projet invalide (mutations M-A3a et M-A3b).
#[sqlx::test(migrations = "./test-schema")]
async fn validation_rechecks_accounts_and_project_changed_since_the_draft(pool: MySqlPool) {
    let (seeded, f) = decor(&pool).await;
    let compte = seeded.accounts["4000"];

    // (a) compte de charge devenu inactif.
    let d = brouillon(&pool, &seeded, f, "FF-INACTIF", dec!(10)).await;
    poser_compte(&pool, compte, "active", false).await;
    let err = validation_refusee(&pool, &seeded, &d).await;
    assert!(matches!(err, DbError::InactiveOrInvalidAccounts), "{err:?}");
    poser_compte(&pool, compte, "active", true).await;

    // (b) compte de charge devenu non imputable.
    let d = brouillon(&pool, &seeded, f, "FF-NON-IMPUTABLE", dec!(10)).await;
    poser_compte(&pool, compte, "postable", false).await;
    let err = validation_refusee(&pool, &seeded, &d).await;
    match &err {
        DbError::AccountsNotPostable(liste) => assert!(
            format!("{liste:?}").contains("4000"),
            "le compte 4000 doit être nommé : {liste:?}"
        ),
        autre => panic!("AccountsNotPostable attendu : {autre:?}"),
    }
    poser_compte(&pool, compte, "postable", true).await;

    // (c) projet archivé depuis l'enregistrement.
    let p = projet(&pool, seeded.company_id, "P-ARCHIVE").await;
    let mut nouveau = nouvelle(
        &seeded,
        f,
        "FF-PROJET",
        jour(2026, 6, 15),
        &[(dec!(10), dec!(0))],
    );
    nouveau.project_id = Some(p);
    let d = supplier_invoices::create_draft(&pool, nouveau, seeded.admin_user_id)
        .await
        .expect("brouillon avec projet");
    sqlx::query("UPDATE projects SET archived = TRUE WHERE id = ?")
        .bind(p)
        .execute(&pool)
        .await
        .unwrap();
    let err = validation_refusee(&pool, &seeded, &d).await;
    assert!(matches!(err, DbError::IllegalStateTransition(_)), "{err:?}");
}

/// AC-10 — **course pilotée** (M20) : A valide le brouillon sans valider sa
/// transaction ; B, seconde validation, attend la facture ; A valide ; B relit
/// `open` sous le verrou et rend `IllegalStateTransition`.
///
/// Le brouillon porte un **projet** (revue P1, E-3) : sous M20 (pas de verrou
/// initial sur la facture), B lit `draft` par une lecture ordinaire, puis attend
/// la sentinelle `companies … FOR UPDATE` de l'étape (0), que A tient — la sonde
/// la voit ; A valide ; B rend alors `OptimisticLockConflict`, et c'est
/// l'assertion de code qui rougit. Sans projet, B bloquait plus tôt, sur
/// l'`INSERT IGNORE` des réglages, que la sonde ne reconnaît pas : la mutation
/// n'était tuée que par le délai de la sonde.
#[sqlx::test(migrations = "./test-schema")]
async fn concurrent_validations_the_loser_reads_validated_under_the_lock(pool: MySqlPool) {
    let (seeded, f) = decor(&pool).await;
    let p = projet(&pool, seeded.company_id, "P-COURSE").await;
    let mut nouveau = nouvelle(
        &seeded,
        f,
        "FF-C",
        jour(2026, 6, 15),
        &[(dec!(40), dec!(0))],
    );
    nouveau.project_id = Some(p);
    let d = supplier_invoices::create_draft(&pool, nouveau, seeded.admin_user_id)
        .await
        .expect("brouillon avec projet");
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
        "SELECT details_json FROM audit_log WHERE action = 'supplier_invoice.unvalidated' \
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

/// AC-11 — statut : payée → `SUPPLIER_INVOICE_PAID` (M6) ; brouillon →
/// `IllegalStateTransition` ; puis la version (après le statut). Story
/// v0172-annulees (#868, AC-1) : une **annulée** n'est plus refusée par son
/// statut — elle repasse en brouillon ; le brouillon reste le seul
/// `IllegalStateTransition`.
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
    // Le refus nomme la pièce par son numéro interne (Story 33-1b, #926) ; un numéro
    // blanc vaut absent (revue P1, E-6) — ni « () » au message, ni `documentNumber: ""`.
    assert!(
        matches!(&err, DbError::SupplierInvoiceNotUnvalidatable { document_label: Some(n), .. } if n.starts_with("FF-20")),
        "{err:?}"
    );
    sqlx::query("UPDATE supplier_invoices SET internal_number = NULL WHERE id = ?")
        .bind(paid)
        .execute(&pool)
        .await
        .unwrap();
    sqlx::query("UPDATE supplier_invoices SET supplier_invoice_number = '  ' WHERE id = ?")
        .bind(paid)
        .execute(&pool)
        .await
        .unwrap();
    let err = supplier_invoices::unvalidate(&pool, c, paid, version, u)
        .await
        .expect_err("payée, numéro blanc");
    assert!(
        matches!(
            &err,
            DbError::SupplierInvoiceNotUnvalidatable {
                document_label: None,
                ..
            }
        ),
        "E-6 : un numéro blanc vaut absent : {err:?}"
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
    // Story v0172-annulees (AC-1) : version périmée → la version parle (le
    // statut `cancelled` passe) ; à jour → brouillon.
    let err = supplier_invoices::unvalidate(&pool, c, x.invoice.id, x.invoice.version + 7, u)
        .await
        .expect_err("annulée, version périmée");
    assert!(matches!(err, DbError::OptimisticLockConflict), "{err:?}");
    let r = supplier_invoices::unvalidate(&pool, c, x.invoice.id, x.invoice.version + 1, u)
        .await
        .expect("annulée → brouillon");
    assert_eq!(r.invoice.status, "draft");

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
        "UPDATE bank_transactions SET matched_entry_id = NULL, status = 'pending' WHERE id = ?",
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

// ---------------------------------------------------------------------------
// Story v0172-annulees (#868) — repasser en brouillon une facture annulée
// ---------------------------------------------------------------------------
//
// Topologie des exercices (fiche, AC-5) : le décor n'a qu'un exercice,
// 2020-2030, ouvert, et la contre-passation est datée **du jour** (horloge
// réelle). Les tests à deux exercices le **scindent en SQL** relativement à
// l'année du jour `Y` : `A` = `Y-2`, `B` = `Y-1`, `C` = [`Y`-01-01,
// 2030-12-31] (l'exercice du décor, raccourci).

fn aujourdhui() -> NaiveDate {
    chrono::Utc::now().date_naive()
}

/// Les trois exercices de la scission.
#[derive(Debug, Clone, Copy)]
struct Exercices {
    y: i32,
    a: i64,
    b: i64,
}

async fn scinder(pool: &MySqlPool, seeded: &SeededCompany) -> Exercices {
    use chrono::Datelike;
    let y = aujourdhui().year();
    assert!(
        y <= 2029,
        "montage : l'exercice C finit le 2030-12-31 (Y = {y})"
    );
    let done =
        sqlx::query("UPDATE fiscal_years SET start_date = ? WHERE id = ? AND company_id = ?")
            .bind(jour(y, 1, 1))
            .bind(seeded.fiscal_year_id)
            .bind(seeded.company_id)
            .execute(pool)
            .await
            .unwrap();
    assert_eq!(done.rows_affected(), 1, "montage : l'exercice du décor");
    let mut ids = Vec::new();
    for an in [y - 2, y - 1] {
        let id = sqlx::query(
            "INSERT INTO fiscal_years (company_id, name, start_date, end_date, status) \
             VALUES (?, ?, ?, ?, 'Open')",
        )
        .bind(seeded.company_id)
        .bind(format!("Exercice {an}"))
        .bind(jour(an, 1, 1))
        .bind(jour(an, 12, 31))
        .execute(pool)
        .await
        .unwrap()
        .last_insert_id() as i64;
        ids.push(id);
    }
    Exercices {
        y,
        a: ids[0],
        b: ids[1],
    }
}

async fn clore(pool: &MySqlPool, fy: i64) {
    sqlx::query("UPDATE fiscal_years SET status = 'Closed' WHERE id = ?")
        .bind(fy)
        .execute(pool)
        .await
        .unwrap();
}

async fn verrouiller(pool: &MySqlPool, company_id: i64, au: NaiveDate) {
    sqlx::query("UPDATE companies SET books_locked_through = ? WHERE id = ?")
        .bind(au)
        .bind(company_id)
        .execute(pool)
        .await
        .unwrap();
}

async fn existe(pool: &MySqlPool, entry: i64) -> bool {
    let n: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM journal_entries WHERE id = ?")
        .bind(entry)
        .fetch_one(pool)
        .await
        .unwrap();
    n == 1
}

/// Une facture annulée : son achat (conservé) et sa contre-passation.
#[derive(Debug, Clone, Copy)]
struct Annulee {
    id: i64,
    achat: i64,
    cp: i64,
    version: i32,
}

async fn annuler(pool: &MySqlPool, seeded: &SeededCompany, id: i64) -> Annulee {
    let x = supplier_invoices::cancel(pool, seeded.company_id, id, seeded.admin_user_id)
        .await
        .expect("annulation");
    assert_eq!(x.invoice.status, "cancelled");
    let achat = x
        .invoice
        .purchase_journal_entry_id
        .expect("l'annulée garde son achat");
    let cp: i64 = sqlx::query_scalar("SELECT id FROM journal_entries WHERE reverses_entry_id = ?")
        .bind(achat)
        .fetch_one(pool)
        .await
        .expect("contre-passation");
    Annulee {
        id,
        achat,
        cp,
        version: x.invoice.version,
    }
}

/// Une facture validée de `lignes`, datée `date`, puis annulée.
async fn annulee(
    pool: &MySqlPool,
    seeded: &SeededCompany,
    f: Fournisseur,
    numero: &str,
    date: NaiveDate,
    lignes: &[(Decimal, Decimal)],
) -> Annulee {
    let v = supplier_invoices::create_validated(
        pool,
        nouvelle(seeded, f, numero, date, lignes),
        seeded.admin_user_id,
    )
    .await
    .expect("facture validée");
    annuler(pool, seeded, v.invoice.id).await
}

async fn repasser(
    pool: &MySqlPool,
    seeded: &SeededCompany,
    a: Annulee,
) -> Result<supplier_invoices::SupplierInvoiceWithLines, DbError> {
    supplier_invoices::unvalidate(
        pool,
        seeded.company_id,
        a.id,
        a.version,
        seeded.admin_user_id,
    )
    .await
}

async fn nb_action(pool: &MySqlPool, action: &str) -> i64 {
    sqlx::query_scalar("SELECT COUNT(*) FROM audit_log WHERE action = ?")
        .bind(action)
        .fetch_one(pool)
        .await
        .unwrap()
}

/// L'état que tout refus doit laisser intact : facture, deux écritures,
/// lettrage, audit.
async fn photo(
    pool: &MySqlPool,
    a: Annulee,
) -> (String, Option<i64>, i32, bool, bool, i64, i64, i64, i64) {
    let (status, je, version) = etat(pool, a.id).await;
    let lettrees: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM journal_entry_lines WHERE entry_id IN (?, ?) \
         AND lettering_key IS NOT NULL",
    )
    .bind(a.achat)
    .bind(a.cp)
    .fetch_one(pool)
    .await
    .unwrap();
    (
        status,
        je,
        version,
        existe(pool, a.achat).await,
        existe(pool, a.cp).await,
        lettrees,
        nb_action(pool, "lettering.removed").await,
        nb_action(pool, "journal_entry.deleted").await,
        nb_action(pool, "supplier_invoice.unvalidated").await,
    )
}

async fn refus_sans_ecrire(pool: &MySqlPool, seeded: &SeededCompany, a: Annulee) -> DbError {
    let avant = photo(pool, a).await;
    assert_eq!(avant.0, "cancelled", "montage");
    let err = repasser(pool, seeded, a).await.expect_err("refus attendu");
    assert_eq!(
        photo(pool, a).await,
        avant,
        "un refus n'écrit rien : {err:?}"
    );
    err
}

fn refus(
    err: &DbError,
) -> (
    SupplierUnvalidationBlocker,
    Option<i64>,
    kesh_db::errors::SupplierUnvalidationHint,
) {
    match err {
        DbError::SupplierInvoiceNotUnvalidatable {
            blocker,
            document_id,
            hint,
            ..
        } => (*blocker, *document_id, hint.clone()),
        other => panic!("refus propre à la pièce attendu : {other:?}"),
    }
}

async fn solde_au(
    pool: &MySqlPool,
    company_id: i64,
    account_id: i64,
    au: Option<NaiveDate>,
) -> Decimal {
    sqlx::query_scalar::<_, Decimal>(
        "SELECT COALESCE(SUM(jel.debit - jel.credit), 0) FROM journal_entry_lines jel \
         JOIN journal_entries je ON je.id = jel.entry_id \
         WHERE je.company_id = ? AND jel.account_id = ? AND (? IS NULL OR je.entry_date <= ?)",
    )
    .bind(company_id)
    .bind(account_id)
    .bind(au)
    .bind(au)
    .fetch_one(pool)
    .await
    .unwrap()
}

/// `(compte, Σ débit − crédit)` des lignes d'une écriture.
async fn effet(pool: &MySqlPool, entry: i64) -> Vec<(i64, Decimal)> {
    sqlx::query_as(
        "SELECT account_id, SUM(debit - credit) FROM journal_entry_lines WHERE entry_id = ? \
         GROUP BY account_id ORDER BY account_id",
    )
    .bind(entry)
    .fetch_all(pool)
    .await
    .unwrap()
}

async fn cles_reversal(pool: &MySqlPool, a: Annulee) -> Vec<i64> {
    sqlx::query_scalar(
        "SELECT DISTINCT lettering_key FROM journal_entry_lines WHERE entry_id IN (?, ?) \
         AND lettering_origin = 'reversal' ORDER BY lettering_key",
    )
    .bind(a.achat)
    .bind(a.cp)
    .fetch_all(pool)
    .await
    .unwrap()
}

/// T1 (AC-1) — une annulée repasse en brouillon : les deux écritures et leurs
/// lignes partent, les paires `reversal` sont dissoutes (une trace chacune),
/// la facture garde ses lignes ; soldes cumulés identiques, soldes de la
/// période de l'achat diminués de l'achat (V2-5). Tue M2, M4, M8.
#[sqlx::test(migrations = "./test-schema")]
async fn cancelled_invoice_returns_to_draft(pool: MySqlPool) {
    let (seeded, f) = decor(&pool).await;
    let c = seeded.company_id;
    let date = jour(2026, 6, 15);
    let a = annulee(
        &pool,
        &seeded,
        f,
        "FF-AN1",
        date,
        &[(dec!(100), dec!(8.1)), (dec!(50), dec!(0))],
    )
    .await;
    let veille = aujourdhui().pred_opt().unwrap();
    assert!(
        veille >= date,
        "montage : l'achat précède la veille de l'annulation"
    );
    let effet_achat = effet(&pool, a.achat).await;
    assert_eq!(
        effet_achat.len(),
        3,
        "montage : charge, impôt préalable, créanciers : {effet_achat:?}"
    );
    assert!(
        effet_achat.iter().any(|(_, m)| *m == dec!(8.10)),
        "montage : une ligne d'impôt préalable : {effet_achat:?}"
    );
    let cles = cles_reversal(&pool, a).await;
    assert!(
        !cles.is_empty(),
        "montage : l'annulation a lettré des paires `reversal`"
    );
    let mut soldes_avant = Vec::new();
    for (compte, _) in &effet_achat {
        soldes_avant.push((
            solde_au(&pool, c, *compte, None).await,
            solde_au(&pool, c, *compte, Some(veille)).await,
        ));
    }
    let lignes_avant: Vec<(String, Decimal, Decimal, Decimal, i64)> = sqlx::query_as(
        "SELECT description, quantity, unit_price, vat_rate, expense_account_id \
         FROM supplier_invoice_lines WHERE supplier_invoice_id = ? ORDER BY id",
    )
    .bind(a.id)
    .fetch_all(&pool)
    .await
    .unwrap();
    let entete = |pool: MySqlPool| async move {
        sqlx::query_as::<
            _,
            (
                Option<String>,
                i64,
                NaiveDate,
                Option<NaiveDate>,
                Option<i64>,
            ),
        >(
            "SELECT supplier_invoice_number, contact_id, invoice_date, due_date, project_id \
             FROM supplier_invoices WHERE id = ?",
        )
        .bind(a.id)
        .fetch_one(&pool)
        .await
        .unwrap()
    };
    let entete_avant = entete(pool.clone()).await;
    let removed_avant = nb_action(&pool, "lettering.removed").await;

    let r = repasser(&pool, &seeded, a)
        .await
        .expect("annulée → brouillon");

    assert_eq!(r.invoice.status, "draft");
    assert_eq!(r.invoice.purchase_journal_entry_id, None);
    assert_eq!(r.invoice.version, a.version + 1, "M8");
    assert_eq!(
        etat(&pool, a.id).await,
        ("draft".into(), None, a.version + 1)
    );
    assert!(!existe(&pool, a.achat).await, "l'achat est supprimé");
    assert!(
        !existe(&pool, a.cp).await,
        "M2 : la contre-passation est supprimée"
    );
    let lignes: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM journal_entry_lines WHERE entry_id IN (?, ?)")
            .bind(a.achat)
            .bind(a.cp)
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(lignes, 0, "les lignes des deux écritures sont parties");
    assert_eq!(
        nb_action(&pool, "lettering.removed").await - removed_avant,
        cles.len() as i64,
        "M4 : une trace `lettering.removed` par paire dissoute"
    );
    for cle in &cles {
        let reste: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM journal_entry_lines WHERE lettering_key = ?")
                .bind(cle)
                .fetch_one(&pool)
                .await
                .unwrap();
        assert_eq!(reste, 0, "aucune ligne ne garde la marque {cle}");
    }
    for ((compte, montant), (cumul, periode)) in effet_achat.iter().zip(soldes_avant) {
        assert_eq!(
            solde_au(&pool, c, *compte, None).await,
            cumul,
            "compte {compte} : la paire était nette à zéro au cumul"
        );
        assert_eq!(
            solde_au(&pool, c, *compte, Some(veille)).await,
            periode - *montant,
            "compte {compte} : la période de l'achat perd l'achat"
        );
    }
    let lignes_apres: Vec<(String, Decimal, Decimal, Decimal, i64)> = sqlx::query_as(
        "SELECT description, quantity, unit_price, vat_rate, expense_account_id \
         FROM supplier_invoice_lines WHERE supplier_invoice_id = ? ORDER BY id",
    )
    .bind(a.id)
    .fetch_all(&pool)
    .await
    .unwrap();
    assert_eq!(lignes_apres, lignes_avant);
    assert_eq!(entete(pool.clone()).await, entete_avant);
}

/// T2 (AC-2) — le brouillon issu d'une annulée vit comme tout brouillon :
/// modifié puis revalidé ; ou, document joint, supprimé (document rendu « à
/// traiter »).
#[sqlx::test(migrations = "./test-schema")]
async fn a_cancelled_then_redrafted_invoice_is_edited_validated_then_deleted(pool: MySqlPool) {
    let (seeded, f) = decor(&pool).await;
    let c = seeded.company_id;
    let u = seeded.admin_user_id;
    // Revalidée.
    let a = annulee(
        &pool,
        &seeded,
        f,
        "FF-AN2",
        jour(2026, 6, 15),
        &[(dec!(40), dec!(0))],
    )
    .await;
    let d = repasser(&pool, &seeded, a).await.expect("brouillon");
    let m = supplier_invoices::update_draft(
        &pool,
        c,
        a.id,
        d.invoice.version,
        nouvelle(
            &seeded,
            f,
            "FF-AN2",
            jour(2026, 6, 15),
            &[(dec!(45), dec!(0))],
        ),
        u,
    )
    .await
    .expect("modifié");
    let r = supplier_invoices::validate(&pool, c, a.id, m.invoice.version, u)
        .await
        .expect("revalidé");
    assert_eq!(r.invoice.status, "open");
    assert_eq!(r.invoice.total_amount, dec!(45));
    let neuf = r
        .invoice
        .purchase_journal_entry_id
        .expect("écriture d'achat neuve");
    assert_ne!(neuf, a.achat);

    // Supprimée, document joint.
    let v = validee(&pool, &seeded, f, "FF-AN2B", dec!(12)).await;
    let doc = document(&pool, c, "an2b").await;
    joindre(&pool, c, doc, v.invoice.id).await;
    let b = annuler(&pool, &seeded, v.invoice.id).await;
    repasser(&pool, &seeded, b).await.expect("brouillon");
    assert_eq!(
        statut_document(&pool, doc).await.1,
        Some(b.id),
        "le document reste joint au brouillon"
    );
    supplier_invoices::delete_draft(&pool, c, b.id, u)
        .await
        .expect("suppression");
    assert_eq!(
        statut_document(&pool, doc).await,
        ("to_complete".into(), None, None)
    );
}

/// T3a (AC-4 a) — exercice de l'achat clos, contre-passation dans un exercice
/// ouvert → `FISCAL_YEAR_CLOSED`, rien d'écrit.
#[sqlx::test(migrations = "./test-schema")]
async fn cancelled_redraft_refused_when_purchase_year_closed(pool: MySqlPool) {
    let (seeded, f) = decor(&pool).await;
    let x = scinder(&pool, &seeded).await;
    let a = annulee(
        &pool,
        &seeded,
        f,
        "FF-3A",
        jour(x.y - 2, 6, 15),
        &[(dec!(20), dec!(0))],
    )
    .await;
    clore(&pool, x.a).await;
    let err = refus_sans_ecrire(&pool, &seeded, a).await;
    assert!(matches!(err, DbError::FiscalYearClosed), "{err:?}");
}

/// T3b (AC-4 b) — exercice clos **entre** l'achat et la contre-passation (état
/// hérité d'avant la v0.13.0, posé en SQL) → l'achat rend
/// `LATER_FISCAL_YEAR_CLOSED`, qui nomme l'exercice.
#[sqlx::test(migrations = "./test-schema")]
async fn cancelled_redraft_refused_when_a_later_year_is_closed(pool: MySqlPool) {
    let (seeded, f) = decor(&pool).await;
    let x = scinder(&pool, &seeded).await;
    let a = annulee(
        &pool,
        &seeded,
        f,
        "FF-3B",
        jour(x.y - 2, 6, 15),
        &[(dec!(20), dec!(0))],
    )
    .await;
    clore(&pool, x.b).await;
    let err = refus_sans_ecrire(&pool, &seeded, a).await;
    match err {
        DbError::LaterFiscalYearClosed {
            fiscal_year_id,
            fiscal_year_name,
        } => {
            assert_eq!(fiscal_year_id, x.b);
            assert_eq!(fiscal_year_name, format!("Exercice {}", x.y - 1));
        }
        other => panic!("{other:?}"),
    }
}

/// T3c (AC-4 c) — la **contre-passation** sous le verrou de période, l'achat
/// (daté après son annulation) hors du verrou → `PERIOD_LOCKED` sur la date de
/// la contre-passation. Tue M1 (suppression brute de la contre-passation).
#[sqlx::test(migrations = "./test-schema")]
async fn cancelled_redraft_refused_when_the_reversal_is_locked(pool: MySqlPool) {
    let (seeded, f) = decor(&pool).await;
    let x = scinder(&pool, &seeded).await;
    let a = annulee(
        &pool,
        &seeded,
        f,
        "FF-3C",
        jour(x.y + 1, 3, 1),
        &[(dec!(20), dec!(0))],
    )
    .await;
    verrouiller(&pool, seeded.company_id, aujourdhui()).await;
    let err = refus_sans_ecrire(&pool, &seeded, a).await;
    match err {
        DbError::PeriodLocked { attempted, .. } => assert_eq!(attempted, aujourdhui(), "M1"),
        other => panic!("M1 : {other:?}"),
    }
}

/// T3c' (AC-4 c') — l'achat sous le verrou → `PERIOD_LOCKED` sur sa date.
#[sqlx::test(migrations = "./test-schema")]
async fn cancelled_redraft_refused_when_the_purchase_is_locked(pool: MySqlPool) {
    let (seeded, f) = decor(&pool).await;
    let x = scinder(&pool, &seeded).await;
    let date = jour(x.y - 2, 6, 15);
    let a = annulee(&pool, &seeded, f, "FF-3CP", date, &[(dec!(20), dec!(0))]).await;
    verrouiller(&pool, seeded.company_id, jour(x.y - 2, 12, 31)).await;
    let err = refus_sans_ecrire(&pool, &seeded, a).await;
    match err {
        DbError::PeriodLocked { attempted, .. } => assert_eq!(attempted, date),
        other => panic!("{other:?}"),
    }
}

/// Lettre « à la main » (état posé en SQL — aucune route ne lettre une ligne
/// d'une pièce, R5) les lignes `ids` en un groupe `manual`.
async fn lettrer_a_la_main(pool: &MySqlPool, ids: &[i64]) {
    let cle = *ids.iter().min().unwrap();
    for id in ids {
        let done = sqlx::query(
            "UPDATE journal_entry_lines SET lettering_key = ?, lettering_origin = 'manual' \
             WHERE id = ? AND lettering_key IS NULL",
        )
        .bind(cle)
        .bind(id)
        .execute(pool)
        .await
        .unwrap();
        assert_eq!(done.rows_affected(), 1, "montage : ligne {id} libre");
    }
}

/// La ligne de charge (compte 4000) d'une écriture.
async fn ligne_de_charge(pool: &MySqlPool, seeded: &SeededCompany, entry: i64) -> i64 {
    sqlx::query_scalar("SELECT id FROM journal_entry_lines WHERE entry_id = ? AND account_id = ?")
        .bind(entry)
        .bind(seeded.accounts["4000"])
        .fetch_one(pool)
        .await
        .unwrap()
}

/// T3d (AC-4 d) — une ligne lettrée **hors** des paires `reversal` de
/// l'annulation (groupe `manual` posé en SQL : la route ne le produit pas) →
/// `ENTRY_LETTERED` ; la dissolution ne touche pas ce groupe. Tue M5.
#[sqlx::test(migrations = "./test-schema")]
async fn cancelled_redraft_refused_when_a_line_is_lettered_outside_the_pairs(pool: MySqlPool) {
    let (seeded, f) = decor(&pool).await;
    let a = annulee(
        &pool,
        &seeded,
        f,
        "FF-3D",
        jour(2026, 6, 15),
        &[(dec!(20), dec!(0))],
    )
    .await;
    let l1 = ligne_de_charge(&pool, &seeded, a.achat).await;
    let l2 = ligne_de_charge(&pool, &seeded, a.cp).await;
    lettrer_a_la_main(&pool, &[l1, l2]).await;
    let err = refus_sans_ecrire(&pool, &seeded, a).await;
    match &err {
        DbError::EntryNotModifiable(g) => assert_eq!(g.code(), "ENTRY_LETTERED", "M5"),
        other => panic!("M5 : {other:?}"),
    }
}

/// T3e (AC-4 e) — l'achat, puis la contre-passation, rapprochés (lien posé
/// après l'annulation) → `MATCHED_BANK_TRANSACTION`, transaction et date au
/// détail ; le lien reste. Tue M9 (variante contre-passation).
#[sqlx::test(migrations = "./test-schema")]
async fn cancelled_redraft_refused_when_an_entry_is_reconciled(pool: MySqlPool) {
    let (seeded, f) = decor(&pool).await;
    for (numero, sur_cp, jour_tx) in [
        ("FF-3E1", false, jour(2026, 7, 2)),
        ("FF-3E2", true, jour(2026, 7, 3)),
    ] {
        let a = annulee(
            &pool,
            &seeded,
            f,
            numero,
            jour(2026, 6, 15),
            &[(dec!(20), dec!(0))],
        )
        .await;
        let cible = if sur_cp { a.cp } else { a.achat };
        let tx =
            fournisseur_rapproche::debit_rapproche(&pool, &seeded, f, cible, jour_tx, dec!(20))
                .await;
        let err = refus_sans_ecrire(&pool, &seeded, a).await;
        let (b, doc, hint) = refus(&err);
        assert_eq!(
            b,
            SupplierUnvalidationBlocker::MatchedBankTransaction,
            "{numero} (M9)"
        );
        assert_eq!(doc, Some(tx));
        assert_eq!(hint.transaction_date, Some(jour_tx));
        let lien: Option<i64> =
            sqlx::query_scalar("SELECT matched_entry_id FROM bank_transactions WHERE id = ?")
                .bind(tx)
                .fetch_one(&pool)
                .await
                .unwrap();
        assert_eq!(lien, Some(cible), "le lien reste");
    }
}

/// T4 P1 (AC-5) — version périmée + paiement détaché → la version parle.
#[sqlx::test(migrations = "./test-schema")]
async fn cancelled_redraft_precedence_version_before_detached_payment(pool: MySqlPool) {
    let (seeded, f) = decor(&pool).await;
    let (id, _, _) = fournisseur_rapproche::facture_payee(
        &pool,
        &seeded,
        f,
        "FF-P1",
        jour(2026, 6, 15),
        dec!(30),
    )
    .await;
    let a = annuler(&pool, &seeded, id).await;
    let err = refus_sans_ecrire(
        &pool,
        &seeded,
        Annulee {
            version: a.version + 1,
            ..a
        },
    )
    .await;
    assert!(matches!(err, DbError::OptimisticLockConflict), "{err:?}");
}

/// T4 P2 — paiement détaché + exercice de l'achat clos → le paiement parle.
#[sqlx::test(migrations = "./test-schema")]
async fn cancelled_redraft_precedence_detached_payment_before_closed_year(pool: MySqlPool) {
    let (seeded, f) = decor(&pool).await;
    let x = scinder(&pool, &seeded).await;
    let (id, _, _) = fournisseur_rapproche::facture_payee(
        &pool,
        &seeded,
        f,
        "FF-P2",
        jour(x.y - 2, 6, 15),
        dec!(30),
    )
    .await;
    let a = annuler(&pool, &seeded, id).await;
    clore(&pool, x.a).await;
    let err = refus_sans_ecrire(&pool, &seeded, a).await;
    assert_eq!(
        refus(&err).0,
        SupplierUnvalidationBlocker::DetachedPayment,
        "{err:?}"
    );
}

/// T4 P3 — rapprochement de l'achat + exercice de l'achat clos → le
/// rapprochement parle.
#[sqlx::test(migrations = "./test-schema")]
async fn cancelled_redraft_precedence_match_before_closed_year(pool: MySqlPool) {
    let (seeded, f) = decor(&pool).await;
    let x = scinder(&pool, &seeded).await;
    let a = annulee(
        &pool,
        &seeded,
        f,
        "FF-P3",
        jour(x.y - 2, 6, 15),
        &[(dec!(20), dec!(0))],
    )
    .await;
    fournisseur_rapproche::debit_rapproche(
        &pool,
        &seeded,
        f,
        a.achat,
        jour(x.y - 2, 6, 20),
        dec!(20),
    )
    .await;
    clore(&pool, x.a).await;
    let err = refus_sans_ecrire(&pool, &seeded, a).await;
    assert_eq!(
        refus(&err).0,
        SupplierUnvalidationBlocker::MatchedBankTransaction,
        "{err:?}"
    );
}

/// T4 P4 — exercice de l'achat clos + contre-passation sous le verrou de
/// période → `FISCAL_YEAR_CLOSED` (pré-contrôle). Tue M12.
#[sqlx::test(migrations = "./test-schema")]
async fn cancelled_redraft_precedence_closed_year_before_locked_reversal(pool: MySqlPool) {
    let (seeded, f) = decor(&pool).await;
    let x = scinder(&pool, &seeded).await;
    let a = annulee(
        &pool,
        &seeded,
        f,
        "FF-P4",
        jour(x.y - 2, 6, 15),
        &[(dec!(20), dec!(0))],
    )
    .await;
    clore(&pool, x.a).await;
    verrouiller(&pool, seeded.company_id, aujourdhui()).await;
    let err = refus_sans_ecrire(&pool, &seeded, a).await;
    assert!(matches!(err, DbError::FiscalYearClosed), "M12 : {err:?}");
}

/// T4 P5 — exercice clos entre l'achat et la contre-passation + contre-passation
/// sous le verrou → la contre-passation parle d'abord (`PERIOD_LOCKED`) ;
/// conséquence assumée de la précédence par écriture (C-v0172a-19).
#[sqlx::test(migrations = "./test-schema")]
async fn cancelled_redraft_precedence_reversal_speaks_first(pool: MySqlPool) {
    let (seeded, f) = decor(&pool).await;
    let x = scinder(&pool, &seeded).await;
    let a = annulee(
        &pool,
        &seeded,
        f,
        "FF-P5",
        jour(x.y - 2, 6, 15),
        &[(dec!(20), dec!(0))],
    )
    .await;
    clore(&pool, x.b).await;
    verrouiller(&pool, seeded.company_id, aujourdhui()).await;
    let err = refus_sans_ecrire(&pool, &seeded, a).await;
    match err {
        DbError::PeriodLocked { attempted, .. } => assert_eq!(attempted, aujourdhui()),
        other => panic!("{other:?}"),
    }
}

/// T4 P6 — verrou sur l'achat + ligne de l'achat lettrée hors paire → le
/// verrou parle (3-quater avant 3-quinquies), la contre-passation ayant passé.
#[sqlx::test(migrations = "./test-schema")]
async fn cancelled_redraft_precedence_period_lock_before_lettering(pool: MySqlPool) {
    let (seeded, f) = decor(&pool).await;
    let x = scinder(&pool, &seeded).await;
    let date = jour(x.y - 2, 6, 15);
    let a = annulee(&pool, &seeded, f, "FF-P6", date, &[(dec!(20), dec!(0))]).await;
    verrouiller(&pool, seeded.company_id, jour(x.y - 2, 12, 31)).await;
    let l = ligne_de_charge(&pool, &seeded, a.achat).await;
    sqlx::query(
        "UPDATE journal_entry_lines SET lettering_key = ?, lettering_origin = 'manual' WHERE id = ?",
    )
    .bind(l)
    .bind(l)
    .execute(&pool)
    .await
    .unwrap();
    let err = refus_sans_ecrire(&pool, &seeded, a).await;
    match err {
        DbError::PeriodLocked { attempted, .. } => assert_eq!(attempted, date),
        other => panic!("{other:?}"),
    }
}

/// T5a (AC-6) — payée puis annulée : le paiement détaché existe, non
/// contre-passé → `SUPPLIER_INVOICE_DETACHED_PAYMENT`, l'écriture nommée,
/// aucune date de transaction. Tue M6.
#[sqlx::test(migrations = "./test-schema")]
async fn cancelled_paid_invoice_redraft_refused_while_payment_stands(pool: MySqlPool) {
    let (seeded, f) = decor(&pool).await;
    let (id, _, reglement) = fournisseur_rapproche::facture_payee(
        &pool,
        &seeded,
        f,
        "FF-5A",
        jour(2026, 6, 15),
        dec!(30),
    )
    .await;
    let a = annuler(&pool, &seeded, id).await;
    let numero: i64 = sqlx::query_scalar("SELECT entry_number FROM journal_entries WHERE id = ?")
        .bind(reglement)
        .fetch_one(&pool)
        .await
        .unwrap();
    let err = refus_sans_ecrire(&pool, &seeded, a).await;
    let (b, doc, hint) = refus(&err);
    assert_eq!(b, SupplierUnvalidationBlocker::DetachedPayment, "M6");
    assert_eq!(doc, Some(reglement));
    assert_eq!(hint.entry_id, Some(reglement));
    assert_eq!(hint.entry_number, Some(numero));
    assert_eq!(hint.entry_date, Some(jour(2026, 6, 15)));
    assert_eq!(hint.transaction_date, None);
    assert!(existe(&pool, reglement).await);
}

/// T5b (AC-6) — idem, paiement rapproché → refus, date de la transaction.
#[sqlx::test(migrations = "./test-schema")]
async fn cancelled_paid_invoice_redraft_refused_with_the_transaction_date(pool: MySqlPool) {
    let (seeded, f) = decor(&pool).await;
    let r = fournisseur_rapproche::reglement_rapproche(
        &pool,
        &seeded,
        f,
        "FF-5B",
        jour(2026, 6, 16),
        dec!(30),
    )
    .await;
    let a = annuler(&pool, &seeded, r.supplier_invoice_id).await;
    let err = refus_sans_ecrire(&pool, &seeded, a).await;
    let (b, _, hint) = refus(&err);
    assert_eq!(b, SupplierUnvalidationBlocker::DetachedPayment);
    assert_eq!(hint.entry_id, Some(r.settlement_entry_id));
    assert_eq!(hint.transaction_date, Some(jour(2026, 6, 16)));
}

/// T5c (AC-6) — paiement contre-passé depuis sa fiche → brouillon ; le paiement
/// et sa contre-passation restent, inchangés. Tue M6b.
#[sqlx::test(migrations = "./test-schema")]
async fn cancelled_paid_invoice_redrafts_once_the_payment_is_reversed(pool: MySqlPool) {
    let (seeded, f) = decor(&pool).await;
    let (id, _, reglement) = fournisseur_rapproche::facture_payee(
        &pool,
        &seeded,
        f,
        "FF-5C",
        jour(2026, 6, 15),
        dec!(30),
    )
    .await;
    let a = annuler(&pool, &seeded, id).await;
    let inverse = kesh_db::repositories::journal_entries::reverse(
        &pool,
        seeded.company_id,
        reglement,
        seeded.admin_user_id,
    )
    .await
    .expect("paiement contre-passé")
    .entry
    .id;
    let effets = (effet(&pool, reglement).await, effet(&pool, inverse).await);
    let r = repasser(&pool, &seeded, a).await.expect("M6b : brouillon");
    assert_eq!(r.invoice.status, "draft");
    assert!(existe(&pool, reglement).await && existe(&pool, inverse).await);
    assert_eq!(
        (effet(&pool, reglement).await, effet(&pool, inverse).await),
        effets
    );
}

/// T5d (AC-6) — le chemin de T5b : *Annuler le rapprochement* contre-passe le
/// paiement → brouillon.
#[sqlx::test(migrations = "./test-schema")]
async fn cancelled_paid_invoice_redrafts_after_unreconciliation(pool: MySqlPool) {
    let (seeded, f) = decor(&pool).await;
    let r = fournisseur_rapproche::reglement_rapproche(
        &pool,
        &seeded,
        f,
        "FF-5D",
        jour(2026, 6, 16),
        dec!(30),
    )
    .await;
    let a = annuler(&pool, &seeded, r.supplier_invoice_id).await;
    refus_sans_ecrire(&pool, &seeded, a).await;
    kesh_db::repositories::reconciliation_cancel::cancel(
        &pool,
        seeded.company_id,
        r.bank_transaction_id,
        seeded.admin_user_id,
    )
    .await
    .expect("rapprochement annulé");
    let r2 = repasser(&pool, &seeded, a).await.expect("brouillon");
    assert_eq!(r2.invoice.status, "draft");
    assert!(
        existe(&pool, r.settlement_entry_id).await,
        "le paiement reste"
    );
    let inverse: Option<i64> =
        sqlx::query_scalar("SELECT id FROM journal_entries WHERE reverses_entry_id = ?")
            .bind(r.settlement_entry_id)
            .fetch_optional(&pool)
            .await
            .unwrap();
    assert!(inverse.is_some(), "et sa contre-passation");
}

/// T5e (AC-6) — deux cycles d'annulation : le premier paiement contre-passé, le
/// second non → le refus nomme le second. Tue M6c.
#[sqlx::test(migrations = "./test-schema")]
async fn cancelled_redraft_examines_every_cancellation_trace(pool: MySqlPool) {
    let (seeded, f) = decor(&pool).await;
    let c = seeded.company_id;
    let u = seeded.admin_user_id;
    let (id, _, s1) = fournisseur_rapproche::facture_payee(
        &pool,
        &seeded,
        f,
        "FF-5E",
        jour(2026, 6, 15),
        dec!(30),
    )
    .await;
    let a = annuler(&pool, &seeded, id).await;
    kesh_db::repositories::journal_entries::reverse(&pool, c, s1, u)
        .await
        .expect("premier paiement contre-passé");
    let d = repasser(&pool, &seeded, a).await.expect("premier cycle");
    let v = supplier_invoices::validate(&pool, c, id, d.invoice.version, u)
        .await
        .expect("revalidée");
    let p = supplier_invoices::pay(
        &pool,
        c,
        id,
        SettlementChoice::BankTransfer {
            bank_account_id: f.bank_account_id,
        },
        jour(2026, 6, 18),
        u,
    )
    .await
    .expect("payée de nouveau");
    assert_eq!(v.invoice.id, id);
    let s2 = p
        .invoice
        .settlement_journal_entry_id
        .expect("second paiement");
    let b = annuler(&pool, &seeded, id).await;
    let err = refus_sans_ecrire(&pool, &seeded, b).await;
    let (blocker, doc, hint) = refus(&err);
    assert_eq!(blocker, SupplierUnvalidationBlocker::DetachedPayment, "M6c");
    assert_eq!(
        (doc, hint.entry_id),
        (Some(s2), Some(s2)),
        "M6c : le second paiement"
    );
}

/// T6 (AC-7) — payée, règlement rapproché → `SUPPLIER_INVOICE_PAID_RECONCILED`,
/// transaction et date ; le statut parle avant la version. Tue M7.
#[sqlx::test(migrations = "./test-schema")]
async fn paid_reconciled_invoice_redraft_names_the_reconciliation(pool: MySqlPool) {
    let (seeded, f) = decor(&pool).await;
    let r = fournisseur_rapproche::reglement_rapproche(
        &pool,
        &seeded,
        f,
        "FF-6",
        jour(2026, 6, 17),
        dec!(30),
    )
    .await;
    let version: i32 = sqlx::query_scalar("SELECT version FROM supplier_invoices WHERE id = ?")
        .bind(r.supplier_invoice_id)
        .fetch_one(&pool)
        .await
        .unwrap();
    for v in [version, version + 3] {
        let err = supplier_invoices::unvalidate(
            &pool,
            seeded.company_id,
            r.supplier_invoice_id,
            v,
            seeded.admin_user_id,
        )
        .await
        .expect_err("payée rapprochée");
        let (b, doc, hint) = refus(&err);
        assert_eq!(b, SupplierUnvalidationBlocker::PaidReconciled, "M7");
        assert_eq!(doc, Some(r.bank_transaction_id));
        assert_eq!(hint.transaction_date, Some(jour(2026, 6, 17)));
    }
    assert_eq!(etat(&pool, r.supplier_invoice_id).await.0, "paid");
}

/// T9 (AC-9) — la trace du geste : `supplier_invoice.unvalidated` avec
/// `previousStatus`, les deux écritures, l'instantané ; `journal_entry.deleted`
/// pour chacune ; `lettering.removed` pour chaque paire. Tue M3, M3b.
#[sqlx::test(migrations = "./test-schema")]
async fn cancelled_redraft_is_audited(pool: MySqlPool) {
    let (seeded, f) = decor(&pool).await;
    let a = annulee(
        &pool,
        &seeded,
        f,
        "FF-9",
        jour(2026, 6, 15),
        &[(dec!(100), dec!(8.1))],
    )
    .await;
    let cles = cles_reversal(&pool, a).await;
    repasser(&pool, &seeded, a).await.expect("brouillon");
    let details: serde_json::Value = sqlx::query_scalar(
        "SELECT details_json FROM audit_log WHERE action = 'supplier_invoice.unvalidated' AND entity_id = ?",
    )
    .bind(a.id)
    .fetch_one(&pool)
    .await
    .expect("M3 : trace du geste");
    assert_eq!(details["previousStatus"], "cancelled", "M3b : {details}");
    assert_eq!(details["purchaseJournalEntryId"], a.achat, "{details}");
    assert_eq!(details["reversalJournalEntryId"], a.cp, "M3b : {details}");
    assert_eq!(
        details["supplierInvoice"]["status"], "cancelled",
        "{details}"
    );
    assert_eq!(nb_audits(&pool, "journal_entry.deleted", a.achat).await, 1);
    assert_eq!(nb_audits(&pool, "journal_entry.deleted", a.cp).await, 1);
    for cle in &cles {
        assert_eq!(
            nb_audits(&pool, "lettering.removed", *cle).await,
            1,
            "paire {cle}"
        );
    }
}

/// T9b (AC-9) — la dévalidation d'une validée écrit `previousStatus: "open"`
/// et aucun `reversalJournalEntryId`.
#[sqlx::test(migrations = "./test-schema")]
async fn validated_redraft_audit_says_open(pool: MySqlPool) {
    let (seeded, f) = decor(&pool).await;
    let v = validee(&pool, &seeded, f, "FF-9B", dec!(10)).await;
    supplier_invoices::unvalidate(
        &pool,
        seeded.company_id,
        v.invoice.id,
        v.invoice.version,
        seeded.admin_user_id,
    )
    .await
    .expect("brouillon");
    let details: serde_json::Value = sqlx::query_scalar(
        "SELECT details_json FROM audit_log WHERE action = 'supplier_invoice.unvalidated' AND entity_id = ?",
    )
    .bind(v.invoice.id)
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(details["previousStatus"], "open", "{details}");
    assert!(details.get("reversalJournalEntryId").is_none(), "{details}");
}

/// T10 (AC-10) — payée par la réconciliation : refus `PaidReconciled` →
/// *Annuler le rapprochement* → validée → brouillon → supprimée ; le règlement
/// et sa contre-passation restent. Tue M7.
#[sqlx::test(migrations = "./test-schema")]
async fn a_reconciled_paid_invoice_goes_back_to_draft_through_unreconciliation(pool: MySqlPool) {
    let (seeded, f) = decor(&pool).await;
    let c = seeded.company_id;
    let u = seeded.admin_user_id;
    let r = fournisseur_rapproche::reglement_rapproche(
        &pool,
        &seeded,
        f,
        "FF-10",
        jour(2026, 6, 15),
        dec!(30),
    )
    .await;
    let version = etat(&pool, r.supplier_invoice_id).await.2;
    let err = supplier_invoices::unvalidate(&pool, c, r.supplier_invoice_id, version, u)
        .await
        .expect_err("payée rapprochée");
    assert_eq!(
        refus(&err).0,
        SupplierUnvalidationBlocker::PaidReconciled,
        "M7"
    );
    kesh_db::repositories::reconciliation_cancel::cancel(&pool, c, r.bank_transaction_id, u)
        .await
        .expect("rapprochement annulé");
    let (status, _, version) = etat(&pool, r.supplier_invoice_id).await;
    assert_eq!(status, "open");
    let d = supplier_invoices::unvalidate(&pool, c, r.supplier_invoice_id, version, u)
        .await
        .expect("brouillon");
    assert_eq!(d.invoice.status, "draft");
    supplier_invoices::delete_draft(&pool, c, r.supplier_invoice_id, u)
        .await
        .expect("supprimée");
    let reste: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM supplier_invoices WHERE id = ?")
        .bind(r.supplier_invoice_id)
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(reste, 0);
    assert!(
        !existe(&pool, r.purchase_entry_id).await,
        "l'achat est supprimé"
    );
    assert!(
        existe(&pool, r.settlement_entry_id).await,
        "le règlement n'est jamais supprimé"
    );
    let inverse: Option<i64> =
        sqlx::query_scalar("SELECT id FROM journal_entries WHERE reverses_entry_id = ?")
            .bind(r.settlement_entry_id)
            .fetch_optional(&pool)
            .await
            .unwrap();
    assert!(inverse.is_some(), "et sa contre-passation");
}

/// T11 (AC-11) — une annulée qui a figuré dans un lot (annulé) repasse en
/// brouillon, mais le brouillon ne se supprime pas.
#[sqlx::test(migrations = "./test-schema")]
async fn a_redrafted_cancelled_invoice_keeps_its_batch_history(pool: MySqlPool) {
    let (seeded, f) = decor(&pool).await;
    let c = seeded.company_id;
    let u = seeded.admin_user_id;
    let v = validee(&pool, &seeded, f, "FF-11", dec!(7)).await;
    let lot = en_lot(&pool, &seeded, f, v.invoice.id).await;
    payment_batches::cancel_batch(&pool, c, lot, u)
        .await
        .unwrap();
    let a = annuler(&pool, &seeded, v.invoice.id).await;
    repasser(&pool, &seeded, a).await.expect("brouillon");
    let err = supplier_invoices::delete_draft(&pool, c, a.id, u)
        .await
        .expect_err("historique de lot");
    assert!(
        matches!(err, DbError::SupplierInvoiceHasPaymentBatchHistory { payment_batch_id } if payment_batch_id == lot),
        "{err:?}"
    );
}

/// T14 (C-v0172a-16) — une annulée sans contre-passation liée (lien retiré en
/// SQL de test : aucun geste ne produit cet état) → `Invariant`, rien d'écrit.
#[sqlx::test(migrations = "./test-schema")]
async fn cancelled_redraft_without_a_linked_reversal_is_an_invariant(pool: MySqlPool) {
    let (seeded, f) = decor(&pool).await;
    let a = annulee(
        &pool,
        &seeded,
        f,
        "FF-14",
        jour(2026, 6, 15),
        &[(dec!(20), dec!(0))],
    )
    .await;
    sqlx::query("UPDATE journal_entries SET reverses_entry_id = NULL WHERE id = ?")
        .bind(a.cp)
        .execute(&pool)
        .await
        .unwrap();
    let err = refus_sans_ecrire(&pool, &seeded, a).await;
    assert!(matches!(err, DbError::Invariant(_)), "{err:?}");
}
