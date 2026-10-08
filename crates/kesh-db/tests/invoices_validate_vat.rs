//! Tests d'intégration `validate_invoice` — comptabilisation TVA aux ventes
//! (Story 18-1b, T-B4). Vérifie bout-en-bout (DB éphémère + écriture réellement
//! insérée) que l'écriture générée porte la TVA due (créance TTC, produit HT, N
//! lignes 2200 par taux), les règles de suppression F-OPUS-1, et les chemins
//! d'erreur (`ConfigurationRequired`, `InactiveOrInvalidAccounts`).
//!
//! Pré-requis : MariaDB démarré (`sqlx::test` crée une DB éphémère par test).
//! Fixture `seed_accounting_company` : 5 comptes (1100 créance, 3000 produit,
//! 2000 réutilisé en TVA due), 4 taux TVA suisses, FY 2020-2030.

use chrono::NaiveDate;
use kesh_db::entities::contact::{ContactType, NewContact};
use kesh_db::entities::{NewInvoice, NewInvoiceLine};
use kesh_db::errors::DbError;
use kesh_db::repositories::{contacts, invoices};
use kesh_db::test_fixtures::{SeededCompany, seed_accounting_company};
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use sqlx::MySqlPool;

const INVOICE_DATE: (i32, u32, u32) = (2026, 6, 15);

async fn make_contact(pool: &MySqlPool, company_id: i64, admin_id: i64) -> i64 {
    contacts::create(
        pool,
        admin_id,
        NewContact {
            company_id,
            contact_type: ContactType::Entreprise,
            name: "Client TVA".into(),
            first_name: None,
            last_name: None,
            is_client: true,
            is_supplier: false,
            address: Some("Rue 1\n1000 Lausanne".into()),
            address_street: None,
            address_building: None,
            address_postal_code: None,
            address_city: None,
            address_country: None,
            email: None,
            phone: None,
            ide_number: None,
            client_number: None,
            default_payment_terms: Some("30".into()),
            default_payment_terms_days: None,
            language: None,
            salutation: kesh_db::entities::contact::Salutation::Neutre,
        },
    )
    .await
    .expect("create contact")
    .id
}

/// Crée une facture brouillon (`(vat_rate, unit_price)` à quantité 1) et la valide.
async fn create_and_validate(
    pool: &MySqlPool,
    seeded: &SeededCompany,
    contact_id: i64,
    lines: &[(Decimal, Decimal)],
) -> Result<kesh_db::repositories::invoices::ValidatedInvoice, DbError> {
    let new = NewInvoice {
        company_id: seeded.company_id,
        contact_id,
        date: NaiveDate::from_ymd_opt(INVOICE_DATE.0, INVOICE_DATE.1, INVOICE_DATE.2).unwrap(),
        due_date: None,
        payment_terms: None,
        lines: lines
            .iter()
            .map(|(rate, price)| NewInvoiceLine {
                revenue_account_id: None,
                description: "Ligne".into(),
                quantity: dec!(1),
                unit_price: *price,
                vat_rate: *rate,
            })
            .collect(),
        project_id: None,
    };
    let (inv, _) = invoices::create(pool, seeded.admin_user_id, new)
        .await
        .expect("create invoice");
    invoices::validate_invoice(pool, seeded.company_id, inv.id, seeded.admin_user_id).await
}

fn sum_debit(je: &kesh_db::entities::JournalEntryWithLines) -> Decimal {
    je.lines.iter().map(|l| l.debit).sum()
}
fn sum_credit(je: &kesh_db::entities::JournalEntryWithLines) -> Decimal {
    je.lines.iter().map(|l| l.credit).sum()
}

/// (a) Facture mono-taux 8.1 % → écriture 3 lignes (créance TTC, produit HT, 1×TVA due) + équilibre.
#[sqlx::test(migrations = "./test-schema")]
async fn validate_single_rate_posts_vat_line(pool: MySqlPool) {
    let seeded = seed_accounting_company(&pool).await.unwrap();
    let contact = make_contact(&pool, seeded.company_id, seeded.admin_user_id).await;

    let v = create_and_validate(&pool, &seeded, contact, &[(dec!(8.10), dec!(1000.00))])
        .await
        .expect("validate");
    let je = &v.journal_entry;
    assert_eq!(je.lines.len(), 3, "créance + produit + 1 ligne TVA");

    let receivable = seeded.accounts["1100"];
    let revenue = seeded.accounts["3000"];
    let vat = seeded.accounts["2000"];

    let creance = je
        .lines
        .iter()
        .find(|l| l.account_id == receivable)
        .unwrap();
    assert_eq!(creance.debit, dec!(1081.00), "créance TTC = 1000 + 81");
    let produit = je.lines.iter().find(|l| l.account_id == revenue).unwrap();
    assert_eq!(produit.credit, dec!(1000.00), "produit HT");
    let tva = je.lines.iter().find(|l| l.account_id == vat).unwrap();
    assert_eq!(tva.credit, dec!(81.00), "TVA due 8.1 %");

    assert_eq!(sum_debit(je), sum_credit(je), "écriture équilibrée");
}

/// (b) Facture entièrement à taux 0 → 2 lignes, AUCUNE ligne sur le compte TVA due.
#[sqlx::test(migrations = "./test-schema")]
async fn validate_zero_rate_no_vat_line(pool: MySqlPool) {
    let seeded = seed_accounting_company(&pool).await.unwrap();
    let contact = make_contact(&pool, seeded.company_id, seeded.admin_user_id).await;

    let v = create_and_validate(&pool, &seeded, contact, &[(dec!(0), dec!(750.00))])
        .await
        .expect("validate");
    let je = &v.journal_entry;
    assert_eq!(je.lines.len(), 2, "créance + produit, pas de TVA");
    let vat = seeded.accounts["2000"];
    assert!(
        je.lines.iter().all(|l| l.account_id != vat),
        "aucune ligne sur le compte TVA due"
    );
    assert_eq!(sum_debit(je), sum_credit(je));
}

/// (c-bis, Story 18-1f / AC11 umbrella F-OPUS-1) Taux > 0 mais TVA **arrondie à
/// `0.00`** (HT minuscule) → AUCUNE ligne sur le compte TVA due (sinon l'INSERT
/// violerait `chk_jel_debit_credit_exclusive`, `credit = 0` interdit).
/// `line_vat_amount(0.01, 8.10) = round_half_up(0.01 × 8.10 / 100) = round_half_up(0.00081) = 0.00`.
#[sqlx::test(migrations = "./test-schema")]
async fn validate_rounds_to_zero_no_vat_line(pool: MySqlPool) {
    let seeded = seed_accounting_company(&pool).await.unwrap();
    // Story 25-4-c4-a : TTC 0.01 s'arrondirait à 0.00 et la pièce serait refusée
    // (`invoiceTotalZero`). Ce test porte sur l'absence de ligne de TVA, pas sur
    // l'arrondi : la facture est émise sans arrondi.
    kesh_db::test_fixtures::disable_rounding_to_5_centimes(&pool, seeded.company_id)
        .await
        .unwrap();
    let contact = make_contact(&pool, seeded.company_id, seeded.admin_user_id).await;

    // HT = 0.01 à 8.1 % → TVA = 0.00 après arrondi → pas de ligne 2200.
    let v = create_and_validate(&pool, &seeded, contact, &[(dec!(8.10), dec!(0.01))])
        .await
        .expect("validate");
    let je = &v.journal_entry;
    assert_eq!(
        je.lines.len(),
        2,
        "créance + produit, pas de ligne TVA (arrondi 0.00)"
    );
    let vat = seeded.accounts["2000"];
    assert!(
        je.lines.iter().all(|l| l.account_id != vat),
        "aucune ligne sur le compte TVA due quand la TVA arrondie tombe à 0.00"
    );
    // Créance TTC = HT + TVA(0.00) = 0.01 = produit HT.
    let receivable = seeded.accounts["1100"];
    let creance = je
        .lines
        .iter()
        .find(|l| l.account_id == receivable)
        .unwrap();
    assert_eq!(creance.debit, dec!(0.01), "créance = HT (TVA nulle)");
    assert_eq!(sum_debit(je), sum_credit(je), "écriture équilibrée");
}

/// (d) Facture multi-taux 8.1 % + 0 % → 1 SEULE ligne TVA due (taux > 0 uniquement).
#[sqlx::test(migrations = "./test-schema")]
async fn validate_mixed_positive_and_zero_rate(pool: MySqlPool) {
    let seeded = seed_accounting_company(&pool).await.unwrap();
    let contact = make_contact(&pool, seeded.company_id, seeded.admin_user_id).await;

    let v = create_and_validate(
        &pool,
        &seeded,
        contact,
        &[(dec!(8.10), dec!(1000.00)), (dec!(0), dec!(400.00))],
    )
    .await
    .expect("validate");
    let je = &v.journal_entry;
    assert_eq!(
        je.lines.len(),
        3,
        "créance + produit + 1 ligne TVA (taux 0 exclu)"
    );
    let vat = seeded.accounts["2000"];
    let vat_lines: Vec<_> = je.lines.iter().filter(|l| l.account_id == vat).collect();
    assert_eq!(vat_lines.len(), 1, "une seule ligne TVA");
    assert_eq!(vat_lines[0].credit, dec!(81.00));
    // créance = 1400 HT + 81 TVA.
    let receivable = seeded.accounts["1100"];
    let creance = je
        .lines
        .iter()
        .find(|l| l.account_id == receivable)
        .unwrap();
    assert_eq!(creance.debit, dec!(1481.00));
    assert_eq!(sum_debit(je), sum_credit(je));
}

/// (c) Multi-taux 8.1 % + 2.6 % → 2 lignes TVA distinctes + équilibre (1594 = 1500 + 13 + 81).
#[sqlx::test(migrations = "./test-schema")]
async fn validate_multi_rate_two_vat_lines(pool: MySqlPool) {
    let seeded = seed_accounting_company(&pool).await.unwrap();
    let contact = make_contact(&pool, seeded.company_id, seeded.admin_user_id).await;

    let v = create_and_validate(
        &pool,
        &seeded,
        contact,
        &[(dec!(8.10), dec!(1000.00)), (dec!(2.60), dec!(500.00))],
    )
    .await
    .expect("validate");
    let je = &v.journal_entry;
    assert_eq!(je.lines.len(), 4, "créance + produit + 2 lignes TVA");
    let vat = seeded.accounts["2000"];
    let vat_total: Decimal = je
        .lines
        .iter()
        .filter(|l| l.account_id == vat)
        .map(|l| l.credit)
        .sum();
    assert_eq!(vat_total, dec!(94.00), "81.00 + 13.00");
    let receivable = seeded.accounts["1100"];
    let creance = je
        .lines
        .iter()
        .find(|l| l.account_id == receivable)
        .unwrap();
    assert_eq!(creance.debit, dec!(1594.00));
    assert_eq!(sum_debit(je), sum_credit(je));
}

/// (f) TVA > 0 mais `default_vat_payable_account_id` NULL → `ConfigurationRequired`.
#[sqlx::test(migrations = "./test-schema")]
async fn validate_vat_without_account_returns_config_required(pool: MySqlPool) {
    let seeded = seed_accounting_company(&pool).await.unwrap();
    let contact = make_contact(&pool, seeded.company_id, seeded.admin_user_id).await;

    // Retirer la config TVA due (la fixture l'a posée à 2000).
    sqlx::query("UPDATE company_invoice_settings SET default_vat_payable_account_id = NULL WHERE company_id = ?")
        .bind(seeded.company_id)
        .execute(&pool)
        .await
        .unwrap();

    let err = create_and_validate(&pool, &seeded, contact, &[(dec!(8.10), dec!(1000.00))])
        .await
        .expect_err("doit échouer sans compte TVA due");
    match err {
        DbError::ConfigurationRequired(field) => {
            assert_eq!(field, "default_vat_payable_account_id");
        }
        other => panic!("attendu ConfigurationRequired, reçu {other:?}"),
    }

    // Sans compte TVA due mais facture SANS TVA → la validation passe.
    let v = create_and_validate(&pool, &seeded, contact, &[(dec!(0), dec!(500.00))]).await;
    assert!(
        v.is_ok(),
        "facture sans TVA ne requiert pas le compte TVA due"
    );
}

/// (f2) Compte TVA due configuré mais ARCHIVÉ → `InactiveOrInvalidAccounts` (pas ConfigurationRequired).
#[sqlx::test(migrations = "./test-schema")]
async fn validate_vat_with_archived_account_returns_inactive(pool: MySqlPool) {
    let seeded = seed_accounting_company(&pool).await.unwrap();
    let contact = make_contact(&pool, seeded.company_id, seeded.admin_user_id).await;

    // Archiver le compte TVA due (2000) — la config le référence toujours.
    sqlx::query("UPDATE accounts SET active = FALSE WHERE id = ?")
        .bind(seeded.accounts["2000"])
        .execute(&pool)
        .await
        .unwrap();

    let err = create_and_validate(&pool, &seeded, contact, &[(dec!(8.10), dec!(1000.00))])
        .await
        .expect_err("doit échouer sur compte TVA due archivé");
    assert!(
        matches!(err, DbError::InactiveOrInvalidAccounts),
        "attendu InactiveOrInvalidAccounts, reçu {err:?}"
    );
}

/// (h) Immunité au changement de taux (F-OPUS-6) : la validation comptabilise la TVA
/// sur le `vat_rate` **snapshoté dans `invoice_lines`** (8.10 % figé à la création),
/// JAMAIS sur un re-lookup de la config `vat_rates`. On mute la config `vat_rates`
/// **entre la création et la validation** : si le code re-lookupait le taux courant,
/// l'écriture porterait 90.00 (9 %) ; comme il lit le snapshot ligne, elle porte 81.00.
#[sqlx::test(migrations = "./test-schema")]
async fn validate_uses_line_rate_snapshot_immune_to_vat_rates_change(pool: MySqlPool) {
    let seeded = seed_accounting_company(&pool).await.unwrap();
    let contact = make_contact(&pool, seeded.company_id, seeded.admin_user_id).await;

    // Facture brouillon : la ligne fige `vat_rate = 8.10` au moment de la création.
    let new = NewInvoice {
        company_id: seeded.company_id,
        contact_id: contact,
        date: NaiveDate::from_ymd_opt(INVOICE_DATE.0, INVOICE_DATE.1, INVOICE_DATE.2).unwrap(),
        due_date: None,
        payment_terms: None,
        lines: vec![NewInvoiceLine {
            revenue_account_id: None,
            description: "Ligne".into(),
            quantity: dec!(1),
            unit_price: dec!(1000.00),
            vat_rate: dec!(8.10),
        }],
        project_id: None,
    };
    let (inv, _) = invoices::create(&pool, seeded.admin_user_id, new)
        .await
        .expect("create invoice");

    // Changement de config APRÈS création, AVANT validation : taux normal 8.10 → 9.00.
    sqlx::query(
        "UPDATE vat_rates SET rate = 9.00 WHERE company_id = ? AND label = 'product-vat-normal'",
    )
    .bind(seeded.company_id)
    .execute(&pool)
    .await
    .unwrap();

    let v = invoices::validate_invoice(&pool, seeded.company_id, inv.id, seeded.admin_user_id)
        .await
        .expect("validate");
    let je = &v.journal_entry;

    let vat = seeded.accounts["2000"];
    let tva = je.lines.iter().find(|l| l.account_id == vat).unwrap();
    assert_eq!(
        tva.credit,
        dec!(81.00),
        "TVA basée sur le snapshot 8.10 % (1000 × 8.1 %), immune au changement config 9.00 %"
    );
    let receivable = seeded.accounts["1100"];
    let creance = je
        .lines
        .iter()
        .find(|l| l.account_id == receivable)
        .unwrap();
    assert_eq!(
        creance.debit,
        dec!(1081.00),
        "créance TTC = 1000 + 81 (snapshot, pas 1090)"
    );
    assert_eq!(sum_debit(je), sum_credit(je), "écriture équilibrée");
}

// ---------------------------------------------------------------------------
// Story 19-4 — tag analytique document-level (Epic 19)
// ---------------------------------------------------------------------------

/// Helper Story 19-4 : insère un projet analytique et retourne son id (calque
/// le helper 19-3 de `supplier_invoices_repository.rs`).
async fn make_project(pool: &MySqlPool, company_id: i64, code: &str, archived: bool) -> i64 {
    sqlx::query(
        "INSERT INTO projects (company_id, code, name, archived, version) VALUES (?, ?, ?, ?, 0)",
    )
    .bind(company_id)
    .bind(code)
    .bind(code)
    .bind(archived)
    .execute(pool)
    .await
    .unwrap()
    .last_insert_id() as i64
}

fn draft_with_project(
    seeded: &SeededCompany,
    contact_id: i64,
    project_id: Option<i64>,
) -> NewInvoice {
    NewInvoice {
        company_id: seeded.company_id,
        contact_id,
        date: NaiveDate::from_ymd_opt(INVOICE_DATE.0, INVOICE_DATE.1, INVOICE_DATE.2).unwrap(),
        due_date: None,
        payment_terms: None,
        project_id,
        lines: vec![NewInvoiceLine {
            revenue_account_id: None,
            description: "Ligne".into(),
            quantity: dec!(1),
            unit_price: dec!(1000.00),
            vat_rate: dec!(8.10),
        }],
    }
}

/// (19-4 a) Le projet de la facture est propagé sur TOUTES les lignes de
/// l'écriture de vente à la validation (créance, produit, TVA comprises).
#[sqlx::test(migrations = "./test-schema")]
async fn validate_with_project_tags_all_sale_lines(pool: MySqlPool) {
    let seeded = seed_accounting_company(&pool).await.unwrap();
    let contact = make_contact(&pool, seeded.company_id, seeded.admin_user_id).await;
    let project = make_project(&pool, seeded.company_id, "RENDEMENT", false).await;

    let (inv, _) = invoices::create(
        &pool,
        seeded.admin_user_id,
        draft_with_project(&seeded, contact, Some(project)),
    )
    .await
    .expect("create");
    assert_eq!(inv.project_id, Some(project));

    let v = invoices::validate_invoice(&pool, seeded.company_id, inv.id, seeded.admin_user_id)
        .await
        .expect("validate");
    assert!(!v.journal_entry.lines.is_empty());
    assert!(
        v.journal_entry
            .lines
            .iter()
            .all(|l| l.project_id == Some(project)),
        "toutes les lignes de vente doivent porter project_id={project}, got {:?}",
        v.journal_entry
            .lines
            .iter()
            .map(|l| l.project_id)
            .collect::<Vec<_>>()
    );
}

/// (19-4 b) Un projet archivé est refusé à la CRÉATION du brouillon.
#[sqlx::test(migrations = "./test-schema")]
async fn create_with_archived_project_is_rejected(pool: MySqlPool) {
    let seeded = seed_accounting_company(&pool).await.unwrap();
    let contact = make_contact(&pool, seeded.company_id, seeded.admin_user_id).await;
    let archived = make_project(&pool, seeded.company_id, "OLD", true).await;

    let err = invoices::create(
        &pool,
        seeded.admin_user_id,
        draft_with_project(&seeded, contact, Some(archived)),
    )
    .await
    .unwrap_err();
    assert!(
        matches!(err, DbError::IllegalStateTransition(_)),
        "got {err:?}"
    );
}

/// (19-4 c) Projet inexistant → NotFound (idem cross-company : scoping company
/// dans validate_taggable_in_tx, couvert par les tests 19-2 du helper).
#[sqlx::test(migrations = "./test-schema")]
async fn create_with_unknown_project_is_rejected(pool: MySqlPool) {
    let seeded = seed_accounting_company(&pool).await.unwrap();
    let contact = make_contact(&pool, seeded.company_id, seeded.admin_user_id).await;

    let err = invoices::create(
        &pool,
        seeded.admin_user_id,
        draft_with_project(&seeded, contact, Some(999_999_999)),
    )
    .await
    .unwrap_err();
    assert!(matches!(err, DbError::NotFound), "got {err:?}");
}

/// (19-4 d) Grandfathering à l'édition du brouillon : un tag INCHANGÉ sur un
/// projet archivé après coup ne bloque pas l'édition ; un CHANGEMENT de projet
/// est validé (archivé refusé).
#[sqlx::test(migrations = "./test-schema")]
async fn update_grandfathers_unchanged_project(pool: MySqlPool) {
    use kesh_db::entities::InvoiceUpdate;

    let seeded = seed_accounting_company(&pool).await.unwrap();
    let contact = make_contact(&pool, seeded.company_id, seeded.admin_user_id).await;
    let p = make_project(&pool, seeded.company_id, "GRAND", false).await;
    let q = make_project(&pool, seeded.company_id, "GRAND-Q", true).await;

    let (inv, _) = invoices::create(
        &pool,
        seeded.admin_user_id,
        draft_with_project(&seeded, contact, Some(p)),
    )
    .await
    .expect("create");

    // Archiver P après la pose du tag.
    sqlx::query("UPDATE projects SET archived = TRUE WHERE id = ?")
        .bind(p)
        .execute(&pool)
        .await
        .unwrap();

    let mk_changes = |project_id: Option<i64>| InvoiceUpdate {
        contact_id: contact,
        date: inv.date,
        due_date: inv.due_date,
        payment_terms: Some("60".into()),
        project_id,
        lines: vec![NewInvoiceLine {
            revenue_account_id: None,
            description: "Ligne".into(),
            quantity: dec!(1),
            unit_price: dec!(1000.00),
            vat_rate: dec!(8.10),
        }],
    };

    // (i) Tag inchangé (P archivé) + édition d'un autre champ → OK.
    let (after, _) = invoices::update(
        &pool,
        seeded.company_id,
        inv.id,
        inv.version,
        seeded.admin_user_id,
        mk_changes(Some(p)),
    )
    .await
    .expect("le tag inchangé archivé doit être toléré à l'édition");
    assert_eq!(after.project_id, Some(p));

    // (ii) Changement vers un AUTRE projet archivé (Q) → refus.
    let err = invoices::update(
        &pool,
        seeded.company_id,
        inv.id,
        after.version,
        seeded.admin_user_id,
        mk_changes(Some(q)),
    )
    .await
    .unwrap_err();
    assert!(
        matches!(err, DbError::IllegalStateTransition(_)),
        "changement vers projet archivé doit être refusé, got {err:?}"
    );
}

/// (19-4 e) Re-validation AU POSTING : un projet archivé entre le brouillon et
/// la validation est refusé (toute nouvelle entrée au grand livre = projet actif).
#[sqlx::test(migrations = "./test-schema")]
async fn validate_rejects_project_archived_after_draft(pool: MySqlPool) {
    let seeded = seed_accounting_company(&pool).await.unwrap();
    let contact = make_contact(&pool, seeded.company_id, seeded.admin_user_id).await;
    let p = make_project(&pool, seeded.company_id, "LATE-ARCH", false).await;

    let (inv, _) = invoices::create(
        &pool,
        seeded.admin_user_id,
        draft_with_project(&seeded, contact, Some(p)),
    )
    .await
    .expect("create");

    sqlx::query("UPDATE projects SET archived = TRUE WHERE id = ?")
        .bind(p)
        .execute(&pool)
        .await
        .unwrap();

    let err = invoices::validate_invoice(&pool, seeded.company_id, inv.id, seeded.admin_user_id)
        .await
        .unwrap_err();
    assert!(
        matches!(err, DbError::IllegalStateTransition(_)),
        "posting sur projet archivé doit être refusé, got {err:?}"
    );
}

// ---------------------------------------------------------------------------
// Story 25-4-c4-a (#494) — l'arrondi à 5 centimes, figé à la validation
// ---------------------------------------------------------------------------

mod arrondi_5_centimes {
    use super::*;
    use kesh_db::entities::{NewCreditNote, SettlementChoice};
    use kesh_db::errors::RoundingContext;
    use kesh_db::repositories::{credit_notes, invoice_settlements, invoice_settlements_write};
    use kesh_db::test_fixtures::{designate_rounding_account, disable_rounding_to_5_centimes};

    /// `(compte, débit, crédit)` des lignes d'une écriture, dans leur ordre.
    fn lignes(je: &kesh_db::entities::JournalEntryWithLines) -> Vec<(i64, Decimal, Decimal)> {
        let mut l: Vec<_> = je.lines.iter().collect();
        l.sort_by_key(|x| x.line_order);
        l.iter()
            .map(|x| (x.account_id, x.debit, x.credit))
            .collect()
    }

    async fn setup(pool: &MySqlPool) -> (SeededCompany, i64, i64) {
        let seeded = seed_accounting_company(pool).await.unwrap();
        let contact = make_contact(pool, seeded.company_id, seeded.admin_user_id).await;
        let rounding = designate_rounding_account(pool, seeded.company_id)
            .await
            .unwrap();
        (seeded, contact, rounding)
    }

    /// ⛔ 123.44 → +0.01 : créance 123.45, écart au CRÉDIT en ligne finale, reste
    /// dû 123.45 ; l'arrondi est figé sur la facture.
    #[sqlx::test(migrations = "./test-schema")]
    async fn positive_rounding_is_credited_last(pool: MySqlPool) {
        let (seeded, contact, rounding) = setup(&pool).await;
        let v = create_and_validate(&pool, &seeded, contact, &[(dec!(0), dec!(123.44))])
            .await
            .expect("validate");
        assert_eq!(v.invoice.rounding_amount, dec!(0.01));
        assert_eq!(
            lignes(&v.journal_entry),
            vec![
                (seeded.accounts["1100"], dec!(123.45), dec!(0)),
                (seeded.accounts["3000"], dec!(0), dec!(123.44)),
                (rounding, dec!(0), dec!(0.01)),
            ]
        );
        assert_eq!(
            invoice_settlements::amount_due(&pool, v.invoice.id)
                .await
                .unwrap(),
            dec!(123.45)
        );
    }

    /// ⛔ 234.52 → −0.02 : l'écart au DÉBIT vient APRÈS la créance, qui reste la
    /// première ligne au débit (lecteurs `ORDER BY jel.id LIMIT 1`).
    #[sqlx::test(migrations = "./test-schema")]
    async fn negative_rounding_is_debited_after_the_receivable(pool: MySqlPool) {
        let (seeded, contact, rounding) = setup(&pool).await;
        let v = create_and_validate(&pool, &seeded, contact, &[(dec!(0), dec!(234.52))])
            .await
            .expect("validate");
        assert_eq!(v.invoice.rounding_amount, dec!(-0.02));
        assert_eq!(
            lignes(&v.journal_entry),
            vec![
                (seeded.accounts["1100"], dec!(234.50), dec!(0)),
                (seeded.accounts["3000"], dec!(0), dec!(234.52)),
                (rounding, dec!(0.02), dec!(0)),
            ]
        );
        let first_debit: i64 = sqlx::query_scalar(
            "SELECT account_id FROM journal_entry_lines WHERE entry_id = ? AND debit > 0 \
             ORDER BY id LIMIT 1",
        )
        .bind(v.journal_entry.entry.id)
        .fetch_one(&pool)
        .await
        .unwrap();
        assert_eq!(first_debit, seeded.accounts["1100"], "la créance d'abord");
    }

    /// Réglage désactivé : aucun arrondi, deux lignes, aucun compte exigé.
    #[sqlx::test(migrations = "./test-schema")]
    async fn disabled_setting_leaves_the_total_unrounded(pool: MySqlPool) {
        let seeded = seed_accounting_company(&pool).await.unwrap();
        let contact = make_contact(&pool, seeded.company_id, seeded.admin_user_id).await;
        disable_rounding_to_5_centimes(&pool, seeded.company_id)
            .await
            .unwrap();
        let v = create_and_validate(&pool, &seeded, contact, &[(dec!(0), dec!(123.44))])
            .await
            .expect("validate sans compte d'arrondi");
        assert_eq!(v.invoice.rounding_amount, dec!(0));
        assert_eq!(v.journal_entry.lines.len(), 2);
    }

    /// ⛔ Compte absent, puis archivé : refus au contexte ÉMISSION, la facture
    /// reste brouillon. Un TTC déjà rond n'exige aucun compte.
    #[sqlx::test(migrations = "./test-schema")]
    async fn a_gap_without_a_usable_account_refuses_validation(pool: MySqlPool) {
        let seeded = seed_accounting_company(&pool).await.unwrap();
        let contact = make_contact(&pool, seeded.company_id, seeded.admin_user_id).await;

        let err = create_and_validate(&pool, &seeded, contact, &[(dec!(0), dec!(123.44))])
            .await
            .expect_err("pas de compte d'arrondi");
        assert!(
            matches!(
                err,
                DbError::RoundingAccountNotConfigured {
                    context: RoundingContext::Issuance
                }
            ),
            "got {err:?}"
        );
        let drafts: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM invoices WHERE company_id = ? AND status = 'draft' \
             AND rounding_amount = 0",
        )
        .bind(seeded.company_id)
        .fetch_one(&pool)
        .await
        .unwrap();
        assert_eq!(drafts, 1, "rien d'écrit, la facture reste brouillon");

        let rounding = designate_rounding_account(&pool, seeded.company_id)
            .await
            .unwrap();
        sqlx::query("UPDATE accounts SET active = FALSE WHERE id = ?")
            .bind(rounding)
            .execute(&pool)
            .await
            .unwrap();
        let err = create_and_validate(&pool, &seeded, contact, &[(dec!(0), dec!(123.44))])
            .await
            .expect_err("compte archivé");
        assert!(
            matches!(err, DbError::RoundingAccountNotConfigured { .. }),
            "got {err:?}"
        );

        create_and_validate(&pool, &seeded, contact, &[(dec!(0), dec!(100.00))])
            .await
            .expect("TTC rond : aucun compte exigé");
    }

    /// Ordre de l'AC 3 : un total ARRONDI nul est refusé comme pièce à zéro,
    /// avant de réclamer un compte d'arrondi.
    #[sqlx::test(migrations = "./test-schema")]
    async fn a_total_rounded_to_zero_is_refused_before_the_account(pool: MySqlPool) {
        let seeded = seed_accounting_company(&pool).await.unwrap();
        let contact = make_contact(&pool, seeded.company_id, seeded.admin_user_id).await;
        let err = create_and_validate(&pool, &seeded, contact, &[(dec!(0), dec!(0.02))])
            .await
            .expect_err("0.02 → 0.00");
        assert!(
            matches!(&err, DbError::InvalidInput(c) if c == "invoiceTotalZero"),
            "got {err:?}"
        );
    }

    /// Un règlement du TTC arrondi solde en DEUX lignes : le chemin d'écart au
    /// centime (25-4-c3-b) n'est pas pris.
    #[sqlx::test(migrations = "./test-schema")]
    async fn settling_the_rounded_total_takes_two_lines(pool: MySqlPool) {
        let (seeded, contact, _rounding) = setup(&pool).await;
        let v = create_and_validate(&pool, &seeded, contact, &[(dec!(0), dec!(123.44))])
            .await
            .expect("validate");
        let out = invoice_settlements_write::settle_invoice(
            &pool,
            seeded.admin_user_id,
            seeded.company_id,
            v.invoice.id,
            SettlementChoice::InternalAccount {
                account_id: seeded.accounts["1000"],
            },
            dec!(123.45),
            NaiveDate::from_ymd_opt(INVOICE_DATE.0, INVOICE_DATE.1, INVOICE_DATE.2).unwrap(),
        )
        .await
        .expect("settle");
        assert!(out.fully_settled);
        let n: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM journal_entry_lines WHERE entry_id = ?")
                .bind(out.journal_entry_id)
                .fetch_one(&pool)
                .await
                .unwrap();
        assert_eq!(n, 2);
    }

    /// ⛔ L'avoir recopie l'arrondi et l'annule en miroir : reste dû 0.
    #[sqlx::test(migrations = "./test-schema")]
    async fn a_credit_note_mirrors_the_rounding(pool: MySqlPool) {
        let (seeded, contact, rounding) = setup(&pool).await;
        let v = create_and_validate(&pool, &seeded, contact, &[(dec!(0), dec!(234.52))])
            .await
            .expect("validate");
        let cn = credit_notes::create_credit_note(
            &pool,
            NewCreditNote {
                company_id: seeded.company_id,
                invoice_id: v.invoice.id,
                date: NaiveDate::from_ymd_opt(INVOICE_DATE.0, INVOICE_DATE.1, INVOICE_DATE.2)
                    .unwrap(),
            },
            seeded.admin_user_id,
        )
        .await
        .expect("avoir");
        assert_eq!(cn.credit_note.rounding_amount, dec!(-0.02));
        assert_eq!(
            lignes(&cn.journal_entry),
            vec![
                (seeded.accounts["1100"], dec!(0), dec!(234.50)),
                (seeded.accounts["3000"], dec!(234.52), dec!(0)),
                (rounding, dec!(0), dec!(0.02)),
            ]
        );
        assert_eq!(
            invoice_settlements::amount_due(&pool, v.invoice.id)
                .await
                .unwrap(),
            dec!(0)
        );
    }

    /// ⛔ Un arrondi NUL n'exige aucun compte pour l'avoir — factures émises sans
    /// arrondi, antérieures comprises (validation P3).
    #[sqlx::test(migrations = "./test-schema")]
    async fn a_credit_note_of_an_unrounded_invoice_needs_no_account(pool: MySqlPool) {
        let seeded = seed_accounting_company(&pool).await.unwrap();
        let contact = make_contact(&pool, seeded.company_id, seeded.admin_user_id).await;
        let v = create_and_validate(&pool, &seeded, contact, &[(dec!(0), dec!(100.00))])
            .await
            .expect("validate");
        let cn = credit_notes::create_credit_note(
            &pool,
            NewCreditNote {
                company_id: seeded.company_id,
                invoice_id: v.invoice.id,
                date: NaiveDate::from_ymd_opt(INVOICE_DATE.0, INVOICE_DATE.1, INVOICE_DATE.2)
                    .unwrap(),
            },
            seeded.admin_user_id,
        )
        .await
        .expect("avoir sans compte d'arrondi");
        assert_eq!(cn.journal_entry.lines.len(), 2);
    }

    /// ⛔ Avoir d'une facture ARRONDIE dont le compte d'arrondi a été archivé
    /// depuis (#486) : refus, et RIEN d'écrit — ni avoir, ni écriture, ni numéro
    /// consommé. Revue de code P1, lentille B.
    ///
    /// **Modifié délibérément par la Story 15-6a** (AC6, choix C-15-6-25) : le
    /// refus était `RoundingAccountNotConfigured { Issuance }`, dont le message
    /// renvoie à *Paramètres → Facturation* — remède faux, l'avoir ne lisant plus
    /// ce réglage (#523). Il est désormais `CreditNoteAccountsArchived`, qui
    /// NOMME le compte à réactiver — celui que la vente a mouvementé. Le refus
    /// vient du verrou des comptes de l'avoir, AVANT la séquence : aucun numéro
    /// n'est tiré. L'assertion « premier numéro après réactivation » fige donc
    /// l'ORDRE (refus avant la séquence), et non plus le rollback du compteur.
    #[sqlx::test(migrations = "./test-schema")]
    async fn a_credit_note_is_refused_when_the_rounding_account_was_archived(pool: MySqlPool) {
        let (seeded, contact, rounding) = setup(&pool).await;
        let v = create_and_validate(&pool, &seeded, contact, &[(dec!(0), dec!(123.44))])
            .await
            .expect("validate");
        sqlx::query("UPDATE accounts SET active = FALSE WHERE id = ?")
            .bind(rounding)
            .execute(&pool)
            .await
            .unwrap();
        let entries_before: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM journal_entries")
            .fetch_one(&pool)
            .await
            .unwrap();
        let new = || NewCreditNote {
            company_id: seeded.company_id,
            invoice_id: v.invoice.id,
            date: NaiveDate::from_ymd_opt(INVOICE_DATE.0, INVOICE_DATE.1, INVOICE_DATE.2).unwrap(),
        };
        let err = credit_notes::create_credit_note(&pool, new(), seeded.admin_user_id)
            .await
            .expect_err("compte d'arrondi archivé");
        match &err {
            DbError::CreditNoteAccountsArchived(archived) => assert_eq!(
                archived,
                &vec![kesh_db::errors::ArchivedAccount {
                    account_id: rounding,
                    account_number: Some("6940".into()),
                }]
            ),
            other => panic!("attendu CreditNoteAccountsArchived(6940), obtenu {other:?}"),
        }
        let notes: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM credit_notes")
            .fetch_one(&pool)
            .await
            .unwrap();
        let entries_after: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM journal_entries")
            .fetch_one(&pool)
            .await
            .unwrap();
        assert_eq!((notes, entries_after), (0, entries_before), "rien d'écrit");

        // Le compte réactivé, l'avoir passe et prend le PREMIER numéro : le refus
        // n'en a consommé aucun.
        sqlx::query("UPDATE accounts SET active = TRUE WHERE id = ?")
            .bind(rounding)
            .execute(&pool)
            .await
            .unwrap();
        let cn = credit_notes::create_credit_note(&pool, new(), seeded.admin_user_id)
            .await
            .expect("avoir");
        assert!(
            cn.credit_note
                .credit_note_number
                .as_deref()
                .is_some_and(|n| n.ends_with("0001")),
            "numéro {:?}",
            cn.credit_note.credit_note_number
        );
    }

    /// Avoir émis par le dépôt, à la date de la facture.
    async fn avoir(
        pool: &MySqlPool,
        seeded: &SeededCompany,
        invoice_id: i64,
    ) -> Result<credit_notes::IssuedCreditNote, DbError> {
        credit_notes::create_credit_note(
            pool,
            NewCreditNote {
                company_id: seeded.company_id,
                invoice_id,
                date: NaiveDate::from_ymd_opt(INVOICE_DATE.0, INVOICE_DATE.1, INVOICE_DATE.2)
                    .unwrap(),
            },
            seeded.admin_user_id,
        )
        .await
    }

    /// Un second compte de charge `6941`, désigné comme compte d'arrondi —
    /// `designate_rounding_account` insère toujours `6940` et ne peut pas
    /// servir deux fois.
    async fn redesignate_rounding(pool: &MySqlPool, company_id: i64) -> i64 {
        let id = sqlx::query(
            "INSERT INTO accounts (company_id, number, name, account_type) \
             VALUES (?, '6941', 'Différences d''arrondi (nouveau)', 'Expense')",
        )
        .bind(company_id)
        .execute(pool)
        .await
        .unwrap()
        .last_insert_id() as i64;
        sqlx::query(
            "UPDATE company_invoice_settings SET default_rounding_account_id = ? \
             WHERE company_id = ?",
        )
        .bind(id)
        .bind(company_id)
        .execute(pool)
        .await
        .unwrap();
        id
    }

    /// Solde (débit − crédit) d'un compte sur toutes les écritures.
    async fn solde(pool: &MySqlPool, account_id: i64) -> Decimal {
        sqlx::query_scalar(
            "SELECT COALESCE(SUM(debit) - SUM(credit), 0) FROM journal_entry_lines \
             WHERE account_id = ?",
        )
        .bind(account_id)
        .fetch_one(pool)
        .await
        .unwrap()
    }

    /// Story 15-6a (test 5) — sur une facture à arrondi NÉGATIF, l'écriture de
    /// vente porte deux lignes de débit (créance, puis arrondi) : le lecteur de
    /// créance rend la créance, pas le compte d'arrondi.
    #[sqlx::test(migrations = "./test-schema")]
    async fn sale_receivable_reader_skips_a_negative_rounding_line(pool: MySqlPool) {
        let (seeded, contact, _rounding) = setup(&pool).await;
        let v = create_and_validate(&pool, &seeded, contact, &[(dec!(0), dec!(234.52))])
            .await
            .expect("validate");
        assert_eq!(v.invoice.rounding_amount, dec!(-0.02));
        let mut conn = pool.acquire().await.unwrap();
        let got = invoice_settlements::sale_receivable_account(
            &mut conn,
            seeded.company_id,
            v.journal_entry.entry.id,
        )
        .await
        .unwrap();
        assert_eq!(got, Some(seeded.accounts["1100"]));
    }

    /// Story 15-6a (test 6, #523) — l'avoir contre-passe l'arrondi sur le compte
    /// que la VENTE a mouvementé (6940), même après qu'un autre compte (6941) a
    /// été désigné : solde de 6940 à zéro, aucune ligne sur 6941.
    #[sqlx::test(migrations = "./test-schema")]
    async fn a_credit_note_reverses_the_sale_rounding_after_redesignation(pool: MySqlPool) {
        let (seeded, contact, rounding) = setup(&pool).await;
        let v = create_and_validate(&pool, &seeded, contact, &[(dec!(0), dec!(123.44))])
            .await
            .expect("validate");
        assert_eq!(v.invoice.rounding_amount, dec!(0.01));
        let other = redesignate_rounding(&pool, seeded.company_id).await;

        let cn = avoir(&pool, &seeded, v.invoice.id).await.expect("avoir");
        assert_eq!(
            lignes(&cn.journal_entry),
            vec![
                (seeded.accounts["1100"], dec!(0), dec!(123.45)),
                (seeded.accounts["3000"], dec!(123.44), dec!(0)),
                (rounding, dec!(0.01), dec!(0)),
            ]
        );
        assert_eq!(solde(&pool, rounding).await, dec!(0), "6940 revient à zéro");
        let on_other: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM journal_entry_lines WHERE account_id = ?")
                .bind(other)
                .fetch_one(&pool)
                .await
                .unwrap();
        assert_eq!(on_other, 0, "aucune ligne sur le compte désigné depuis");
    }

    /// Story 15-6a (test 7, #523) — le réglage d'arrondi VIDÉ après la validation
    /// (SQL direct) n'empêche plus l'avoir : la ligne d'arrondi vise 6940.
    #[sqlx::test(migrations = "./test-schema")]
    async fn a_credit_note_needs_no_rounding_setting(pool: MySqlPool) {
        let (seeded, contact, rounding) = setup(&pool).await;
        let v = create_and_validate(&pool, &seeded, contact, &[(dec!(0), dec!(123.44))])
            .await
            .expect("validate");
        sqlx::query(
            "UPDATE company_invoice_settings SET default_rounding_account_id = NULL \
             WHERE company_id = ?",
        )
        .bind(seeded.company_id)
        .execute(&pool)
        .await
        .unwrap();

        let cn = avoir(&pool, &seeded, v.invoice.id)
            .await
            .expect("avoir sans réglage d'arrondi");
        assert_eq!(
            lignes(&cn.journal_entry).last(),
            Some(&(rounding, dec!(0.01), dec!(0)))
        );
        assert_eq!(solde(&pool, rounding).await, dec!(0));
    }

    /// Story 15-6a (test 8) — le lecteur d'arrondi RECOUPE la dernière ligne avec
    /// l'arrondi de la facture : appelé avec le signe opposé, il refuse
    /// (`Invariant`) au lieu de rendre une ligne en silence. Le bon signe rend
    /// bien le compte (témoin).
    #[sqlx::test(migrations = "./test-schema")]
    async fn sale_rounding_reader_refuses_a_mismatched_line(pool: MySqlPool) {
        let (seeded, contact, rounding) = setup(&pool).await;
        let v = create_and_validate(&pool, &seeded, contact, &[(dec!(0), dec!(123.44))])
            .await
            .expect("validate");
        let entry = v.journal_entry.entry.id;
        let mut conn = pool.acquire().await.unwrap();
        let ok = invoice_settlements::sale_rounding_account(
            &mut conn,
            seeded.company_id,
            entry,
            dec!(0.01),
        )
        .await
        .unwrap();
        assert_eq!(ok, rounding, "témoin : le bon signe rend le compte");
        for wrong in [dec!(-0.01), dec!(0.02)] {
            let err = invoice_settlements::sale_rounding_account(
                &mut conn,
                seeded.company_id,
                entry,
                wrong,
            )
            .await
            .expect_err("forme différente");
            assert!(matches!(err, DbError::Invariant(_)), "{wrong} : {err:?}");
        }
    }

    /// La dévalidation remet l'arrondi figé à zéro.
    #[sqlx::test(migrations = "./test-schema")]
    async fn unvalidation_resets_the_rounding(pool: MySqlPool) {
        let (seeded, contact, _rounding) = setup(&pool).await;
        let v = create_and_validate(&pool, &seeded, contact, &[(dec!(0), dec!(123.44))])
            .await
            .expect("validate");
        let (inv, _) = invoices::unvalidate(
            &pool,
            seeded.company_id,
            v.invoice.id,
            seeded.admin_user_id,
            v.invoice.version,
        )
        .await
        .expect("unvalidate");
        assert_eq!(inv.rounding_amount, dec!(0));
    }
}

// ---------------------------------------------------------------------------
// Story 25-4-e (#495) — le montant minimum d'une facture
// ---------------------------------------------------------------------------

mod montant_minimum {
    use super::*;
    use kesh_db::entities::NewCreditNote;
    use kesh_db::repositories::credit_notes;
    use kesh_db::test_fixtures::designate_rounding_account;

    async fn set_minimum(pool: &MySqlPool, company_id: i64, minimum: Option<Decimal>) {
        sqlx::query(
            "UPDATE company_invoice_settings SET minimum_invoice_amount = ? WHERE company_id = ?",
        )
        .bind(minimum)
        .bind(company_id)
        .execute(pool)
        .await
        .unwrap();
    }

    async fn setup(pool: &MySqlPool) -> (SeededCompany, i64) {
        let seeded = seed_accounting_company(pool).await.unwrap();
        let contact = make_contact(pool, seeded.company_id, seeded.admin_user_id).await;
        designate_rounding_account(pool, seeded.company_id)
            .await
            .unwrap();
        (seeded, contact)
    }

    /// ⛔ Sous le seuil : refus dédié, nommant les deux montants, la facture reste
    /// brouillon ; égal au seuil : accepté.
    #[sqlx::test(migrations = "./test-schema")]
    async fn below_the_minimum_is_refused_equal_is_accepted(pool: MySqlPool) {
        let (seeded, contact) = setup(&pool).await;
        set_minimum(&pool, seeded.company_id, Some(dec!(5.00))).await;

        let err = create_and_validate(&pool, &seeded, contact, &[(dec!(0), dec!(4.50))])
            .await
            .expect_err("4.50 < 5.00");
        assert!(
            matches!(err, DbError::InvoiceBelowMinimum { total, minimum }
                if total == dec!(4.50) && minimum == dec!(5.00)),
            "got {err:?}"
        );
        let drafts: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM invoices WHERE company_id = ? AND status = 'draft'",
        )
        .bind(seeded.company_id)
        .fetch_one(&pool)
        .await
        .unwrap();
        assert_eq!(drafts, 1, "rien d'écrit, la facture reste brouillon");

        create_and_validate(&pool, &seeded, contact, &[(dec!(0), dec!(5.00))])
            .await
            .expect("égal au seuil : accepté");
    }

    /// ⛔ Le seuil se compare au total ARRONDI : 4.98 brut, arrondi à 5.00, passe un
    /// seuil de 5.00.
    #[sqlx::test(migrations = "./test-schema")]
    async fn the_minimum_compares_to_the_rounded_total(pool: MySqlPool) {
        let (seeded, contact) = setup(&pool).await;
        set_minimum(&pool, seeded.company_id, Some(dec!(5.00))).await;
        let v = create_and_validate(&pool, &seeded, contact, &[(dec!(0), dec!(4.98))])
            .await
            .expect("4.98 → 5.00 : au seuil");
        assert_eq!(v.invoice.rounding_amount, dec!(0.02));
    }

    /// ⛔ Arrondi désactivé, un total à quatre décimales est comparé AU CENTIME :
    /// 4.995 vaut 5.00 et passe un seuil de 5.00 ; 4.994 vaut 4.99 et est refusé, le
    /// message nommant 4.99 (revue de code P1, lentille A).
    #[sqlx::test(migrations = "./test-schema")]
    async fn the_minimum_compares_at_the_centime(pool: MySqlPool) {
        let (seeded, contact) = setup(&pool).await;
        kesh_db::test_fixtures::disable_rounding_to_5_centimes(&pool, seeded.company_id)
            .await
            .unwrap();
        set_minimum(&pool, seeded.company_id, Some(dec!(5.00))).await;
        create_and_validate(&pool, &seeded, contact, &[(dec!(0), dec!(4.995))])
            .await
            .expect("4.995 → 5.00 au centime : au seuil");
        let err = create_and_validate(&pool, &seeded, contact, &[(dec!(0), dec!(4.994))])
            .await
            .expect_err("4.994 → 4.99");
        assert!(
            matches!(err, DbError::InvoiceBelowMinimum { total, .. } if total == dec!(4.99)),
            "got {err:?}"
        );
    }

    /// Aucun seuil (le défaut) : une facture de 0.05 se valide.
    #[sqlx::test(migrations = "./test-schema")]
    async fn no_minimum_by_default(pool: MySqlPool) {
        let (seeded, contact) = setup(&pool).await;
        create_and_validate(&pool, &seeded, contact, &[(dec!(0), dec!(0.05))])
            .await
            .expect("aucun seuil");
    }

    /// L'avoir n'est pas soumis au seuil : une facture émise avant qu'un seuil plus
    /// haut soit fixé se crédite.
    #[sqlx::test(migrations = "./test-schema")]
    async fn a_credit_note_ignores_the_minimum(pool: MySqlPool) {
        let (seeded, contact) = setup(&pool).await;
        let v = create_and_validate(&pool, &seeded, contact, &[(dec!(0), dec!(3.00))])
            .await
            .expect("validate");
        set_minimum(&pool, seeded.company_id, Some(dec!(10.00))).await;
        credit_notes::create_credit_note(
            &pool,
            NewCreditNote {
                company_id: seeded.company_id,
                invoice_id: v.invoice.id,
                date: NaiveDate::from_ymd_opt(INVOICE_DATE.0, INVOICE_DATE.1, INVOICE_DATE.2)
                    .unwrap(),
            },
            seeded.admin_user_id,
        )
        .await
        .expect("avoir sous le seuil : accepté");
    }
}

// ---------------------------------------------------------------------------
// Story 15-5d (#429) — la garde À L'USAGE des comptes de réglage, côté vente
// ---------------------------------------------------------------------------

/// La créance et la TVA due désignées dans les réglages, contrôlées au moment où
/// la validation y écrit : `DesignatedAccountsNotPostable` (AC1, AC7).
///
/// Montage commun (finding R6-1, choix C88) : `seed_accounting_company` (créance
/// `1100`, TVA due `2000`, exercice 2020-2030), **puis l'arrondi à 5 centimes
/// désactivé** — sans quoi une facture au TTC non multiple de 0.05 réclamerait un
/// compte d'arrondi que ce montage ne désigne pas, et s'arrêterait AVANT
/// l'accesseur. Chaque compte de test est **désigné avant** d'être rendu non
/// imputable : le chemin réel (l'exemption « inchangé » de la 15-5b l'a laissé en
/// place).
mod garde_usage_comptes_reglage {
    use super::*;
    use kesh_db::entities::NewCreditNote;
    use kesh_db::errors::RevenueAccountRejection;
    use kesh_db::repositories::credit_notes;
    use kesh_db::test_fixtures::{
        attendre_une_requete_en_cours, disable_rounding_to_5_centimes, sonde_verrou_nowait,
    };

    /// Motifs de la requête VERROUILLANTE de l'accesseur
    /// (`lock_designated_accounts_in_tx`) ; la lecture préalable des identifiants
    /// de la société ne porte pas `LOCK IN SHARE MODE`.
    const ACCESSEUR: &[&str] = &[
        "FROM accounts FORCE INDEX (PRIMARY) WHERE company_id",
        "LOCK IN SHARE MODE",
    ];
    /// Motifs de `fiscal_years::find_open_covering_date`.
    const EXERCICE: &[&str] = &["FROM fiscal_years", "FOR UPDATE"];
    const SONDE_EXERCICE: &str = "SELECT id FROM fiscal_years WHERE id = ? FOR UPDATE NOWAIT";
    const SONDE_COMPTE: &str = "SELECT id FROM accounts WHERE id = ? FOR UPDATE NOWAIT";

    async fn setup(pool: &MySqlPool) -> (SeededCompany, i64) {
        let seeded = seed_accounting_company(pool).await.unwrap();
        disable_rounding_to_5_centimes(pool, seeded.company_id)
            .await
            .unwrap();
        let contact = make_contact(pool, seeded.company_id, seeded.admin_user_id).await;
        (seeded, contact)
    }

    /// Une facture brouillon, `(taux, prix unitaire, compte de produit)` à
    /// quantité 1.
    async fn draft(
        pool: &MySqlPool,
        seeded: &SeededCompany,
        contact_id: i64,
        lines: &[(Decimal, Decimal, Option<i64>)],
    ) -> i64 {
        let new = NewInvoice {
            company_id: seeded.company_id,
            contact_id,
            date: NaiveDate::from_ymd_opt(INVOICE_DATE.0, INVOICE_DATE.1, INVOICE_DATE.2).unwrap(),
            due_date: None,
            payment_terms: None,
            lines: lines
                .iter()
                .map(|(rate, price, account)| NewInvoiceLine {
                    revenue_account_id: *account,
                    description: "Ligne".into(),
                    quantity: dec!(1),
                    unit_price: *price,
                    vat_rate: *rate,
                })
                .collect(),
            project_id: None,
        };
        invoices::create(pool, seeded.admin_user_id, new)
            .await
            .expect("create invoice")
            .0
            .id
    }

    async fn validate(
        pool: &MySqlPool,
        seeded: &SeededCompany,
        invoice_id: i64,
    ) -> Result<invoices::ValidatedInvoice, DbError> {
        invoices::validate_invoice(pool, seeded.company_id, invoice_id, seeded.admin_user_id).await
    }

    async fn set_postable(pool: &MySqlPool, account_id: i64, postable: bool) {
        sqlx::query("UPDATE accounts SET postable = ? WHERE id = ?")
            .bind(postable)
            .bind(account_id)
            .execute(pool)
            .await
            .unwrap();
    }

    /// `(id, numéro)` des comptes nommés par un `DesignatedAccountsNotPostable`.
    fn designated_rejected(err: DbError) -> Vec<(i64, String)> {
        match err {
            DbError::DesignatedAccountsNotPostable(list) => list
                .iter()
                .map(|a| (a.account_id, a.account_number.clone()))
                .collect(),
            other => panic!("attendu DesignatedAccountsNotPostable, obtenu {other:?}"),
        }
    }

    /// La facture est restée brouillon et la société n'a aucune écriture.
    async fn assert_nothing_written(pool: &MySqlPool, seeded: &SeededCompany, invoice_id: i64) {
        let status: String = sqlx::query_scalar("SELECT status FROM invoices WHERE id = ?")
            .bind(invoice_id)
            .fetch_one(pool)
            .await
            .unwrap();
        assert_eq!(status, "draft", "la facture doit rester brouillon");
        let entries: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM journal_entries WHERE company_id = ?")
                .bind(seeded.company_id)
                .fetch_one(pool)
                .await
                .unwrap();
        assert_eq!(entries, 0, "aucune écriture ne doit avoir été passée");
    }

    /// Créance non imputable → validation refusée, le compte nommé `(id, n°)`.
    #[sqlx::test(migrations = "./test-schema")]
    async fn receivable_not_postable_refuses_validation(pool: MySqlPool) {
        let (seeded, contact) = setup(&pool).await;
        let id = draft(&pool, &seeded, contact, &[(dec!(8.10), dec!(100.00), None)]).await;
        let receivable = seeded.accounts["1100"];
        set_postable(&pool, receivable, false).await;

        let err = validate(&pool, &seeded, id).await.expect_err("refusée");
        assert_eq!(err.error_code(), "ACCOUNT_NOT_POSTABLE");
        assert_eq!(
            designated_rejected(err),
            vec![(receivable, "1100".to_string())]
        );
        assert_nothing_written(&pool, &seeded, id).await;
    }

    /// TVA due non imputable : avec TVA → refusée ; sans TVA → validée ; à taux
    /// positif dont la TVA arrondit à zéro → validée (le prédicat est
    /// `total_vat > 0`, non « une ligne à taux > 0 »).
    #[sqlx::test(migrations = "./test-schema")]
    async fn vat_payable_not_postable_refuses_only_when_vat_is_written(pool: MySqlPool) {
        let (seeded, contact) = setup(&pool).await;
        let vat = seeded.accounts["2000"];
        let with_vat = draft(&pool, &seeded, contact, &[(dec!(8.10), dec!(100.00), None)]).await;
        let without_vat = draft(&pool, &seeded, contact, &[(dec!(0), dec!(100.00), None)]).await;
        // HT 0.01 à 8.1 % : TVA arrondie à 0.00, aucune ligne de TVA due.
        let rounds_to_zero =
            draft(&pool, &seeded, contact, &[(dec!(8.10), dec!(0.01), None)]).await;
        set_postable(&pool, vat, false).await;

        let err = validate(&pool, &seeded, with_vat)
            .await
            .expect_err("avec TVA : refusée");
        assert_eq!(designated_rejected(err), vec![(vat, "2000".to_string())]);

        validate(&pool, &seeded, without_vat)
            .await
            .expect("sans TVA : la TVA due n'est pas écrite, la validation passe");
        validate(&pool, &seeded, rounds_to_zero)
            .await
            .expect("TVA arrondie à zéro : la TVA due n'est pas écrite, la validation passe");
    }

    /// Créance ET TVA due non imputables, sur deux comptes DISTINCTS → un seul
    /// refus qui les nomme tous deux (`count = 2`, pluriel).
    #[sqlx::test(migrations = "./test-schema")]
    async fn receivable_and_vat_payable_are_named_in_one_refusal(pool: MySqlPool) {
        let (seeded, contact) = setup(&pool).await;
        let id = draft(&pool, &seeded, contact, &[(dec!(8.10), dec!(100.00), None)]).await;
        let (receivable, vat) = (seeded.accounts["1100"], seeded.accounts["2000"]);
        set_postable(&pool, receivable, false).await;
        set_postable(&pool, vat, false).await;

        let err = validate(&pool, &seeded, id).await.expect_err("refusée");
        assert_eq!(
            designated_rejected(err),
            vec![(receivable, "1100".to_string()), (vat, "2000".to_string())]
        );
        assert_nothing_written(&pool, &seeded, id).await;
    }

    /// Un même compte désigné pour DEUX rôles (créance et TVA due), rendu non
    /// imputable, facture avec TVA → nommé UNE fois (`count = 1`, singulier) :
    /// le dédoublonnage de `NonPostableAccounts::new` est atteint par l'accesseur
    /// (choix C40).
    #[sqlx::test(migrations = "./test-schema")]
    async fn one_account_for_two_roles_is_named_once(pool: MySqlPool) {
        let (seeded, contact) = setup(&pool).await;
        let receivable = seeded.accounts["1100"];
        sqlx::query(
            "UPDATE company_invoice_settings SET default_vat_payable_account_id = ? \
             WHERE company_id = ?",
        )
        .bind(receivable)
        .bind(seeded.company_id)
        .execute(&pool)
        .await
        .unwrap();
        let id = draft(&pool, &seeded, contact, &[(dec!(8.10), dec!(100.00), None)]).await;
        set_postable(&pool, receivable, false).await;

        let err = validate(&pool, &seeded, id).await.expect_err("refusée");
        let DbError::DesignatedAccountsNotPostable(list) = err else {
            panic!("attendu DesignatedAccountsNotPostable, obtenu {err:?}");
        };
        assert_eq!(list.len(), 1, "le compte commun doit être nommé une fois");
        assert_eq!(list.details()["rejected"].as_array().unwrap().len(), 1);
        assert_eq!(list.numbers(), vec!["1100"]);
    }

    /// Ordre des refus — le compte de produit EXPLICITE d'une ligne non
    /// imputable passe avant : `InvalidRevenueAccounts`, raison `NotPostable`
    /// (Story 16-1a ; finding F5-2).
    #[sqlx::test(migrations = "./test-schema")]
    async fn line_revenue_account_refusal_comes_first(pool: MySqlPool) {
        let (seeded, contact) = setup(&pool).await;
        let services: i64 = sqlx::query(
            "INSERT INTO accounts (company_id, number, name, account_type) \
             VALUES (?, '3200', 'Prestations', 'Revenue')",
        )
        .bind(seeded.company_id)
        .execute(&pool)
        .await
        .unwrap()
        .last_insert_id() as i64;
        let id = draft(
            &pool,
            &seeded,
            contact,
            &[(dec!(8.10), dec!(100.00), Some(services))],
        )
        .await;
        set_postable(&pool, services, false).await;
        set_postable(&pool, seeded.accounts["1100"], false).await;

        let err = validate(&pool, &seeded, id).await.expect_err("refusée");
        match err {
            DbError::InvalidRevenueAccounts(rejected) => {
                assert_eq!(rejected[0].reason, RevenueAccountRejection::NotPostable);
            }
            other => panic!("attendu InvalidRevenueAccounts, obtenu {other:?}"),
        }
    }

    /// Ordre des refus — TVA due ABSENTE (et créance non imputable) →
    /// `ConfigurationRequired` du générateur, avant la garde (finding F4-5).
    #[sqlx::test(migrations = "./test-schema")]
    async fn missing_vat_payable_comes_before_the_guard(pool: MySqlPool) {
        let (seeded, contact) = setup(&pool).await;
        sqlx::query(
            "UPDATE company_invoice_settings SET default_vat_payable_account_id = NULL \
             WHERE company_id = ?",
        )
        .bind(seeded.company_id)
        .execute(&pool)
        .await
        .unwrap();
        let id = draft(&pool, &seeded, contact, &[(dec!(8.10), dec!(100.00), None)]).await;
        set_postable(&pool, seeded.accounts["1100"], false).await;

        let err = validate(&pool, &seeded, id).await.expect_err("refusée");
        assert!(
            matches!(&err, DbError::ConfigurationRequired(f) if f == "default_vat_payable_account_id"),
            "attendu ConfigurationRequired, obtenu {err:?}"
        );
    }

    /// Ordre des refus — aucun exercice ouvert ET créance non imputable →
    /// `FiscalYearInvalid` : le verrou des comptes précède l'exercice, leur
    /// refus non (finding F2-1, C43).
    #[sqlx::test(migrations = "./test-schema")]
    async fn no_open_fiscal_year_comes_before_the_guard(pool: MySqlPool) {
        let (seeded, contact) = setup(&pool).await;
        let id = draft(&pool, &seeded, contact, &[(dec!(8.10), dec!(100.00), None)]).await;
        sqlx::query("UPDATE fiscal_years SET status = 'Closed' WHERE id = ?")
            .bind(seeded.fiscal_year_id)
            .execute(&pool)
            .await
            .unwrap();
        set_postable(&pool, seeded.accounts["1100"], false).await;

        let err = validate(&pool, &seeded, id).await.expect_err("refusée");
        assert!(
            matches!(err, DbError::FiscalYearInvalid),
            "attendu FiscalYearInvalid, obtenu {err:?}"
        );
    }

    /// Compte de réglage ARCHIVÉ → `InactiveOrInvalidAccounts`, rendu par
    /// l'accesseur (C49) — la variante neuve n'est pas émise.
    ///
    /// ⚠️ Ce test prouve le **résultat**, non **qui** le produit : sans la course
    /// d'archivage entre l'instantané et le verrou, le contrôle non verrouillant
    /// de `create_in_tx` rendrait le même refus. Le refus de l'accesseur est
    /// vérifié à la lecture (Dev Agent Record) ; sa mutation n'est pas attendue
    /// rouge.
    #[sqlx::test(migrations = "./test-schema")]
    async fn archived_designated_account_is_inactive_or_invalid(pool: MySqlPool) {
        let (seeded, contact) = setup(&pool).await;
        let id = draft(&pool, &seeded, contact, &[(dec!(8.10), dec!(100.00), None)]).await;
        sqlx::query("UPDATE accounts SET active = FALSE, postable = FALSE WHERE id = ?")
            .bind(seeded.accounts["1100"])
            .execute(&pool)
            .await
            .unwrap();

        let err = validate(&pool, &seeded, id).await.expect_err("refusée");
        assert!(
            matches!(err, DbError::InactiveOrInvalidAccounts),
            "attendu InactiveOrInvalidAccounts, obtenu {err:?}"
        );
        assert_nothing_written(&pool, &seeded, id).await;
    }

    /// Priorité des refus **en mélange** (C49 ; revue de code P1, E2) :
    /// créance non imputable ET TVA due archivée →
    /// `InactiveOrInvalidAccounts`, et non la variante qui nommerait la
    /// créance. Le contrôle juge « absent ou inactif » sur TOUS les rôles écrits
    /// avant de nommer un compte non imputable, quel que soit l'ordre des rôles
    /// (la créance précède la TVA due).
    #[sqlx::test(migrations = "./test-schema")]
    async fn archived_account_wins_over_a_non_postable_one(pool: MySqlPool) {
        let (seeded, contact) = setup(&pool).await;
        let id = draft(&pool, &seeded, contact, &[(dec!(8.10), dec!(100.00), None)]).await;
        set_postable(&pool, seeded.accounts["1100"], false).await;
        sqlx::query("UPDATE accounts SET active = FALSE WHERE id = ?")
            .bind(seeded.accounts["2000"])
            .execute(&pool)
            .await
            .unwrap();

        let err = validate(&pool, &seeded, id).await.expect_err("refusée");
        assert!(
            matches!(err, DbError::InactiveOrInvalidAccounts),
            "attendu InactiveOrInvalidAccounts, obtenu {err:?}"
        );
        assert_nothing_written(&pool, &seeded, id).await;
    }

    /// L'AVOIR ne contrôle pas l'imputabilité de ce qu'il contre-passe
    /// (choix C-15-6-2, C-15-6-7 ; Story 15-6a) : la créance **de la vente**,
    /// rendue non imputable après la validation, reste celle que l'avoir
    /// crédite, et l'avoir est émis — « mêmes comptes que l'origine, seule
    /// l'inactivité bloque ».
    ///
    /// **Changé de sens par la Story 15-6a** : ce test figeait l'exemption de
    /// l'avoir à la garde à l'usage (C35), quand l'avoir relisait la créance dans
    /// les réglages. Il ne la lit plus là : sans ce ré-ancrage, le test serait
    /// resté vert pour une autre raison, sans plus rien figer.
    #[sqlx::test(migrations = "./test-schema")]
    async fn credit_note_credits_a_non_postable_sale_receivable(pool: MySqlPool) {
        let (seeded, contact) = setup(&pool).await;
        let id = draft(&pool, &seeded, contact, &[(dec!(8.10), dec!(100.00), None)]).await;
        validate(&pool, &seeded, id).await.expect("validée");
        set_postable(&pool, seeded.accounts["1100"], false).await;

        let cn = credit_notes::create_credit_note(
            &pool,
            NewCreditNote {
                company_id: seeded.company_id,
                invoice_id: id,
                date: NaiveDate::from_ymd_opt(INVOICE_DATE.0, INVOICE_DATE.1, INVOICE_DATE.2)
                    .unwrap(),
            },
            seeded.admin_user_id,
        )
        .await
        .expect("l'avoir est émis malgré la créance non imputable");
        let creance = cn
            .journal_entry
            .lines
            .iter()
            .find(|l| l.credit > Decimal::ZERO)
            .expect("ligne de crédit");
        assert_eq!(
            (creance.account_id, creance.credit),
            (seeded.accounts["1100"], dec!(108.10)),
            "le crédit vise la créance de la vente"
        );
    }

    /// Un compte d'une AUTRE société posé comme TVA due des réglages
    /// (choix C51, C88 ; findings F3-5, F6-5).
    ///
    /// (1) **Il n'est jamais verrouillé** : la validation est tenue en attente sur
    /// l'exercice — donc PASSÉE l'accesseur, ses verrous posés — et une sonde
    /// `FOR UPDATE NOWAIT` sur la ligne étrangère réussit. Sous la mutation qui
    /// retire le patron `owned_account_ids` (verrou posé directement sur
    /// `company_id = ? AND id IN (…)`), la ligne étrangère est verrouillée avant
    /// que le filtre ne l'écarte, et la sonde échoue (`1205`).
    ///
    /// (2) **Il est refusé** `InactiveOrInvalidAccounts` (branche « absent de
    /// l'instantané »). ⚠️ Même limite que le test « archivé » : sans cette
    /// branche, le contrôle non verrouillant de `create_in_tx` rendrait le même
    /// refus — le test prouve le résultat, non qui le produit.
    #[sqlx::test(migrations = "./test-schema")]
    async fn foreign_account_is_never_locked_and_is_refused(pool: MySqlPool) {
        let (seeded, contact) = setup(&pool).await;
        let other_company: i64 = sqlx::query(
            "INSERT INTO companies (name, address, org_type, accounting_language, \
             instance_language) VALUES ('Autre SA', 'Rue 3\n1000 Lausanne', 'Independant', \
             'FR', 'FR')",
        )
        .execute(&pool)
        .await
        .unwrap()
        .last_insert_id() as i64;
        let foreign: i64 = sqlx::query(
            "INSERT INTO accounts (company_id, number, name, account_type) \
             VALUES (?, '2200', 'TVA due étrangère', 'Liability')",
        )
        .bind(other_company)
        .execute(&pool)
        .await
        .unwrap()
        .last_insert_id() as i64;
        sqlx::query(
            "UPDATE company_invoice_settings SET default_vat_payable_account_id = ? \
             WHERE company_id = ?",
        )
        .bind(foreign)
        .bind(seeded.company_id)
        .execute(&pool)
        .await
        .unwrap();
        let id = draft(&pool, &seeded, contact, &[(dec!(8.10), dec!(100.00), None)]).await;

        // La bloqueuse tient l'exercice : la validation s'y arrêtera, après
        // l'accesseur.
        let mut bloqueuse = pool.begin().await.unwrap();
        sqlx::query("SELECT id FROM fiscal_years WHERE id = ? FOR UPDATE")
            .bind(seeded.fiscal_year_id)
            .fetch_all(&mut *bloqueuse)
            .await
            .unwrap();
        let (p, c, u) = (pool.clone(), seeded.company_id, seeded.admin_user_id);
        let tache = tokio::spawn(async move { invoices::validate_invoice(&p, c, id, u).await });
        let vue = attendre_une_requete_en_cours(&pool, EXERCICE, || tache.is_finished()).await;
        assert!(
            vue,
            "la validation a fini sans attendre l'exercice : {:?}",
            tache.await.map(|r| r.map(|_| ()))
        );
        assert!(
            sonde_verrou_nowait(&pool, SONDE_COMPTE, foreign).await,
            "le compte d'une autre société ne doit pas être verrouillé par l'accesseur"
        );
        // Témoin : la créance, elle, est tenue (en partagé) par l'accesseur.
        assert!(
            !sonde_verrou_nowait(&pool, SONDE_COMPTE, seeded.accounts["1100"]).await,
            "la créance de la société doit être verrouillée par l'accesseur"
        );
        bloqueuse.rollback().await.unwrap();

        let err = tache.await.expect("tâche").expect_err("refusée");
        assert!(
            matches!(err, DbError::InactiveOrInvalidAccounts),
            "attendu InactiveOrInvalidAccounts, obtenu {err:?}"
        );
        assert_nothing_written(&pool, &seeded, id).await;
    }

    /// **Test de place 1** — vente : le verrou des comptes désignés précède
    /// l'exercice, et il lit la ligne FRAÎCHE (C43, C87).
    ///
    /// La bloqueuse exécute l'`UPDATE accounts SET postable = FALSE` de la créance
    /// sans conclure (le geste réel d'un passage à non imputable : verrou
    /// exclusif). La validation est vue en attente **sur l'accesseur** ; une
    /// sonde sur l'exercice réussit : la validation ne le tient pas encore. La
    /// bloqueuse valide ; la validation lit la ligne fraîche et refuse.
    ///
    /// Sous l'ordre fautif (verrou après l'exercice), la sonde échoue ; sous un
    /// accesseur sans verrou, la tâche n'est jamais vue en attente sur
    /// l'accesseur — le test rougit dans les deux cas.
    #[sqlx::test(migrations = "./test-schema")]
    async fn place_1_sale_lock_precedes_fiscal_year_and_reads_fresh(pool: MySqlPool) {
        let (seeded, contact) = setup(&pool).await;
        let id = draft(&pool, &seeded, contact, &[(dec!(8.10), dec!(100.00), None)]).await;
        let receivable = seeded.accounts["1100"];

        let mut bloqueuse = pool.begin().await.unwrap();
        sqlx::query("UPDATE accounts SET postable = FALSE WHERE id = ?")
            .bind(receivable)
            .execute(&mut *bloqueuse)
            .await
            .unwrap();
        let (p, c, u) = (pool.clone(), seeded.company_id, seeded.admin_user_id);
        let tache = tokio::spawn(async move { invoices::validate_invoice(&p, c, id, u).await });
        let vue = attendre_une_requete_en_cours(&pool, ACCESSEUR, || tache.is_finished()).await;
        assert!(
            vue,
            "la validation n'a pas été vue en attente sur l'accesseur : {:?}",
            tache.await.map(|r| r.map(|_| ()))
        );
        assert!(
            sonde_verrou_nowait(&pool, SONDE_EXERCICE, seeded.fiscal_year_id).await,
            "la validation ne doit pas encore tenir l'exercice"
        );
        bloqueuse.commit().await.unwrap();

        let err = tache.await.expect("tâche").expect_err("refusée");
        assert_eq!(
            designated_rejected(err),
            vec![(receivable, "1100".to_string())]
        );
        assert_nothing_written(&pool, &seeded, id).await;
    }

    /// **Test de mode 4** — vente : le verrou des comptes désignés est PARTAGÉ
    /// (C87 ; le seul test qui rougit si l'accesseur repasse en `FOR UPDATE`).
    ///
    /// La bloqueuse reproduit ce que tient un règlement client à l'insertion de
    /// ses lignes : l'exercice en exclusif, la créance et la TVA due en partagé.
    /// La validation est vue en attente **sur l'exercice** — donc passée
    /// l'accesseur malgré les partagés de la bloqueuse. Sous un accesseur en
    /// `FOR UPDATE`, elle attendrait sur les comptes, et
    /// `attendre_une_requete_en_cours` paniquerait au bout de dix secondes.
    #[sqlx::test(migrations = "./test-schema")]
    async fn mode_4_sale_lock_is_shared(pool: MySqlPool) {
        let (seeded, contact) = setup(&pool).await;
        let id = draft(&pool, &seeded, contact, &[(dec!(8.10), dec!(100.00), None)]).await;
        let (receivable, vat) = (seeded.accounts["1100"], seeded.accounts["2000"]);

        let mut bloqueuse = pool.begin().await.unwrap();
        sqlx::query("SELECT id FROM fiscal_years WHERE id = ? FOR UPDATE")
            .bind(seeded.fiscal_year_id)
            .fetch_all(&mut *bloqueuse)
            .await
            .unwrap();
        // ⛔ `name` est lu À DESSEIN : `SELECT id … LOCK IN SHARE MODE` est
        // couvert par un index secondaire (`fk_accounts_parent`, mesuré par
        // `EXPLAIN` sur ce montage à cinq comptes), et un verrou PARTAGÉ posé
        // par un index secondaire couvrant ne verrouille PAS la ligne de la clé
        // primaire — celle que prennent `fk_jel_account` et l'accesseur. La
        // bloqueuse ne tiendrait alors rien, et le test passerait sous un
        // accesseur en `FOR UPDATE` (constaté au développement : mutation
        // restée verte). `name` n'est dans aucun index secondaire.
        sqlx::query("SELECT id, name FROM accounts WHERE id IN (?, ?) LOCK IN SHARE MODE")
            .bind(receivable)
            .bind(vat)
            .fetch_all(&mut *bloqueuse)
            .await
            .unwrap();
        // Le montage lui-même est vérifié : les deux lignes sont bien tenues.
        for compte in [receivable, vat] {
            assert!(
                !sonde_verrou_nowait(&pool, SONDE_COMPTE, compte).await,
                "la bloqueuse doit tenir le compte {compte} en partagé"
            );
        }
        let (p, c, u) = (pool.clone(), seeded.company_id, seeded.admin_user_id);
        let tache = tokio::spawn(async move { invoices::validate_invoice(&p, c, id, u).await });
        let vue = attendre_une_requete_en_cours(&pool, EXERCICE, || tache.is_finished()).await;
        assert!(
            vue,
            "la validation a fini sans attendre l'exercice : {:?}",
            tache.await.map(|r| r.map(|_| ()))
        );
        bloqueuse.rollback().await.unwrap();

        tache
            .await
            .expect("tâche")
            .expect("la validation réussit une fois l'exercice rendu");
    }
}
