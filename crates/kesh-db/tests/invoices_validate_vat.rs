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
            is_client: true,
            is_supplier: false,
            address: Some("Rue 1\n1000 Lausanne".into()),
            email: None,
            phone: None,
            ide_number: None,
            default_payment_terms: Some("30".into()),
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
                description: "Ligne".into(),
                quantity: dec!(1),
                unit_price: *price,
                vat_rate: *rate,
            })
            .collect(),
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
#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
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
#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
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
/// `line_vat_amount(0.01, 8.10) = round_half_up(0.00081) = 0.00`.
#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn validate_rounds_to_zero_no_vat_line(pool: MySqlPool) {
    let seeded = seed_accounting_company(&pool).await.unwrap();
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
#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
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
#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
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
#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
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
#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
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
#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
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
            description: "Ligne".into(),
            quantity: dec!(1),
            unit_price: dec!(1000.00),
            vat_rate: dec!(8.10),
        }],
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
