//! Le rapport TVA retranche la TVA des soldes — Story 25-4-d2c (#384).
//!
//! Pré-requis : MariaDB démarré.

use chrono::NaiveDate;
use kesh_db::entities::SettlementWriteOffNature as Nature;
use kesh_db::entities::contact::{ContactType, NewContact};
use kesh_db::entities::invoice::{NewInvoice, NewInvoiceLine};
use kesh_db::repositories::{contacts, invoice_settlements_write, invoices};
use kesh_db::test_fixtures::{SeededCompany, seed_accounting_company};
use kesh_report::period::ReportPeriod;
use kesh_report::vat_report;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use sqlx::MySqlPool;

fn ymd(y: i32, m: u32, d: u32) -> NaiveDate {
    NaiveDate::from_ymd_opt(y, m, d).unwrap()
}

/// Période 2026 (incluse dans l'exercice fixture 2020-2030).
fn period_2026(fiscal_year_id: i64) -> ReportPeriod {
    ReportPeriod {
        fiscal_year_id,
        start_date: ymd(2026, 1, 1),
        end_date: ymd(2026, 12, 31),
    }
}

async fn seed_contact(pool: &MySqlPool, seeded: &SeededCompany) -> i64 {
    contacts::create(
        pool,
        seeded.admin_user_id,
        NewContact {
            company_id: seeded.company_id,
            contact_type: ContactType::Personne,
            name: "Client CI".into(),
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
            default_payment_terms: None,
            default_payment_terms_days: None,
            language: None,
            salutation: kesh_db::entities::contact::Salutation::Neutre,
        },
    )
    .await
    .unwrap()
    .id
}

/// Crée une facture (lignes `(vat_rate, unit_price)` à quantité 1) puis la valide
/// via le flow normal `validate_invoice` (pose `journal_entry_id`, génère la ligne
/// TVA due sur `default_vat_payable_account_id`). Réplique le helper de
/// `vat_report_e2e.rs:144`. Renvoie l'`invoice_id`.
async fn create_validated_invoice(
    pool: &MySqlPool,
    seeded: &SeededCompany,
    contact_id: i64,
    date: NaiveDate,
    lines: &[(Decimal, Decimal)],
) -> i64 {
    let new = NewInvoice {
        company_id: seeded.company_id,
        contact_id,
        date,
        due_date: Some(date),
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
        .unwrap();
    invoices::validate_invoice(pool, seeded.company_id, inv.id, seeded.admin_user_id)
        .await
        .expect("validate_invoice")
        .invoice
        .id
}

/// Le rapport TVA de la société sur l'exercice 2026 du montage.
async fn report(pool: &MySqlPool, seeded: &SeededCompany) -> vat_report::VatReport {
    vat_report::generate(pool, seeded.company_id, &period_2026(seeded.fiscal_year_id))
        .await
        .expect("rapport")
}

/// La société de test, ses natures d'escompte et de frais sur la charge `4000` ;
/// la TVA due est le `2000` du montage.
async fn company(pool: &MySqlPool) -> SeededCompany {
    let seeded = seed_accounting_company(pool).await.unwrap();
    sqlx::query(
        "UPDATE company_invoice_settings SET default_discount_account_id = ?, \
         default_bank_fees_account_id = ? WHERE company_id = ?",
    )
    .bind(seeded.accounts["4000"])
    .bind(seeded.accounts["4000"])
    .bind(seeded.company_id)
    .execute(pool)
    .await
    .unwrap();
    seeded
}

async fn write_off(
    pool: &MySqlPool,
    seeded: &SeededCompany,
    inv: i64,
    nature: Nature,
    on: NaiveDate,
) -> i64 {
    let version: i32 = sqlx::query_scalar("SELECT version FROM invoices WHERE id = ?")
        .bind(inv)
        .fetch_one(pool)
        .await
        .unwrap();
    let out = invoice_settlements_write::write_off_invoice(
        pool,
        seeded.admin_user_id,
        seeded.company_id,
        inv,
        nature,
        on,
        version,
    )
    .await
    .expect("solde");
    sqlx::query_scalar("SELECT id FROM invoice_settlements WHERE journal_entry_id = ?")
        .bind(out.journal_entry_id)
        .fetch_one(pool)
        .await
        .unwrap()
}

/// Deux taux : 100 × 1.081 + 100 × 1.026 = 210.70 (multiple de 0.05 : pas d'arrondi).
const LINES: [(Decimal, Decimal); 2] = [(dec!(8.10), dec!(100.00)), (dec!(2.60), dec!(100.00))];

/// ⛔ **Un escompte soldé dans la période est retranché, par taux** ; la TVA due
/// nette, le solde et un écart **nul** en découlent.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn a_discount_in_the_period_is_subtracted_per_rate(pool: MySqlPool) {
    let seeded = company(&pool).await;
    let contact = seed_contact(&pool, &seeded).await;
    let inv = create_validated_invoice(&pool, &seeded, contact, ymd(2026, 3, 1), &LINES).await;
    write_off(&pool, &seeded, inv, Nature::Discount, ymd(2026, 3, 10)).await;

    let r = report(&pool, &seeded).await;
    assert_eq!(
        r.total_vat_due,
        dec!(10.70),
        "la TVA facturée est inchangée"
    );
    let rows: Vec<_> = r
        .write_off_rows
        .iter()
        .map(|w| (w.rate, w.base_ht, w.vat))
        .collect();
    assert_eq!(
        rows,
        vec![
            (dec!(2.60), dec!(100.00), dec!(2.60)),
            (dec!(8.10), dec!(100.00), dec!(8.10))
        ]
    );
    assert_eq!(r.total_vat_write_off, dec!(10.70));
    assert_eq!(r.total_vat_due_net, dec!(0.00));
    assert_eq!(r.vat_balance, r.total_vat_due_net - r.total_vat_recoverable);
    assert_eq!(
        r.reconciliation_delta,
        dec!(0.00),
        "écart nul : le grand livre net suit"
    );
    assert_eq!(r.reconciliation_status, "ok");
}

/// Deux soldes de deux factures, au même taux, dans la période : **une** ligne
/// par taux, montants additionnés (revue P1, lentille B).
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn write_offs_of_several_invoices_merge_per_rate(pool: MySqlPool) {
    let seeded = company(&pool).await;
    let contact = seed_contact(&pool, &seeded).await;
    let a = create_validated_invoice(&pool, &seeded, contact, ymd(2026, 3, 1), &LINES).await;
    // 50 × 1.081 = 54.05 (multiple de 0.05 : pas d'arrondi).
    let b = create_validated_invoice(
        &pool,
        &seeded,
        contact,
        ymd(2026, 4, 1),
        &[(dec!(8.10), dec!(50.00))],
    )
    .await;
    write_off(&pool, &seeded, a, Nature::Discount, ymd(2026, 3, 10)).await;
    write_off(&pool, &seeded, b, Nature::Discount, ymd(2026, 4, 10)).await;

    let r = report(&pool, &seeded).await;
    let rows: Vec<_> = r
        .write_off_rows
        .iter()
        .map(|w| (w.rate, w.base_ht, w.vat))
        .collect();
    assert_eq!(
        rows,
        vec![
            (dec!(2.60), dec!(100.00), dec!(2.60)),
            (dec!(8.10), dec!(150.00), dec!(12.15))
        ]
    );
    assert_eq!(r.total_vat_write_off, dec!(14.75));
    assert_eq!(r.reconciliation_delta, dec!(0.00));
}

/// Un solde de la période sur une facture d'une période ANTÉRIEURE : la correction
/// est retranchée, et l'écart reste nul (la TVA facturée manque des deux côtés).
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn a_write_off_of_an_earlier_invoice_keeps_the_delta_null(pool: MySqlPool) {
    let seeded = company(&pool).await;
    let contact = seed_contact(&pool, &seeded).await;
    let inv = create_validated_invoice(&pool, &seeded, contact, ymd(2025, 11, 1), &LINES).await;
    write_off(&pool, &seeded, inv, Nature::Discount, ymd(2026, 2, 1)).await;
    let r = report(&pool, &seeded).await;
    assert_eq!(r.total_vat_due, dec!(0));
    assert_eq!(r.total_vat_write_off, dec!(10.70));
    assert_eq!(r.total_vat_due_net, dec!(-10.70));
    assert_eq!(r.reconciliation_delta, dec!(0.00));
}

/// Hors période, frais bancaires (sans TVA), autre société : rien n'est retranché.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn out_of_period_bank_fees_and_other_companies_are_ignored(pool: MySqlPool) {
    let seeded = company(&pool).await;
    let contact = seed_contact(&pool, &seeded).await;
    // Hors période : un solde daté de 2027.
    let a = create_validated_invoice(&pool, &seeded, contact, ymd(2026, 3, 1), &LINES).await;
    write_off(&pool, &seeded, a, Nature::Discount, ymd(2027, 1, 15)).await;
    // Frais bancaires : ventilation `[]`.
    let b = create_validated_invoice(&pool, &seeded, contact, ymd(2026, 3, 1), &LINES).await;
    write_off(&pool, &seeded, b, Nature::BankFees, ymd(2026, 3, 10)).await;
    // Autre société : un solde rattaché à une autre société.
    let c = create_validated_invoice(&pool, &seeded, contact, ymd(2026, 3, 1), &LINES).await;
    let sid = write_off(&pool, &seeded, c, Nature::Discount, ymd(2026, 3, 10)).await;
    let other = sqlx::query(
        "INSERT INTO companies (name, address, org_type, accounting_language, instance_language) \
         VALUES ('Autre société', 'Rue 2\n1000 Lausanne', 'Pme', 'FR', 'FR')",
    )
    .execute(&pool)
    .await
    .unwrap()
    .last_insert_id() as i64;
    sqlx::query("UPDATE invoice_settlements SET company_id = ? WHERE id = ?")
        .bind(other)
        .bind(sid)
        .execute(&pool)
        .await
        .unwrap();

    let r = report(&pool, &seeded).await;
    assert!(r.write_off_rows.is_empty(), "{:?}", r.write_off_rows);
    assert_eq!(r.total_vat_write_off, dec!(0));
    assert_eq!(r.total_vat_due_net, r.total_vat_due);
}

/// Un solde annulé disparaît du rapport — et de l'écart : ni son écriture ni sa
/// contre-passation ne sont jointes.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn a_cancelled_write_off_disappears(pool: MySqlPool) {
    let seeded = company(&pool).await;
    let contact = seed_contact(&pool, &seeded).await;
    let inv = create_validated_invoice(&pool, &seeded, contact, ymd(2026, 3, 1), &LINES).await;
    let sid = write_off(&pool, &seeded, inv, Nature::Discount, ymd(2026, 3, 10)).await;
    invoice_settlements_write::cancel_settlement(
        &pool,
        seeded.admin_user_id,
        seeded.company_id,
        inv,
        sid,
    )
    .await
    .expect("annulation");
    let r = report(&pool, &seeded).await;
    assert!(r.write_off_rows.is_empty());
    assert_eq!(r.total_vat_due_net, dec!(10.70));
    assert_eq!(r.reconciliation_delta, dec!(0.00));
}

/// Une ventilation de forme fausse est une erreur, jamais un zéro silencieux.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn a_malformed_breakdown_is_an_error(pool: MySqlPool) {
    let seeded = company(&pool).await;
    let contact = seed_contact(&pool, &seeded).await;
    let inv = create_validated_invoice(&pool, &seeded, contact, ymd(2026, 3, 1), &LINES).await;
    let sid = write_off(&pool, &seeded, inv, Nature::Discount, ymd(2026, 3, 10)).await;
    // JSON valide, forme fausse : un taux en nombre au lieu d'une chaîne.
    sqlx::query(
        "UPDATE invoice_settlements SET write_off_vat = '[{\"ratePercent\": 8.1, \"baseHt\": \"1\", \"vatAmount\": \"1\"}]' WHERE id = ?",
    )
    .bind(sid)
    .execute(&pool)
    .await
    .unwrap();
    let err = vat_report::generate(
        &pool,
        seeded.company_id,
        &period_2026(seeded.fiscal_year_id),
    )
    .await
    .unwrap_err();
    assert!(
        matches!(err, kesh_report::errors::ReportError::CorruptData(_)),
        "{err:?}"
    );
}
