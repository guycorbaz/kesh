//! Tests d'intégration — TVA récupérable réelle dans le VatReport (Story 18-1d).
//!
//! Vérifie que `vat_report::generate` remplit `total_vat_recoverable` depuis le
//! solde du compte impôt préalable au grand livre (filtre `entry_date` seul, DC4),
//! recalcule `vat_balance`, et que le cas « achats seuls » (récupérable sans vente)
//! reste rendu au CSV (AC10/T-D2b).
//!
//! Le fixture `seed_accounting_company` configure
//! `default_vat_recoverable_account_id = compte 1000` (réutilisé, Story 18-1c) et
//! ne seede AUCUNE facture → `total_vat_due == 0`. Tous les tests sont donc des
//! scénarios « achats seuls » (rows vide), ce qui est le cas réel de 18-1d.
//!
//! Pré-requis : MariaDB (`sqlx::test` crée une DB éphémère par test).

use chrono::NaiveDate;
use kesh_db::entities::journal_entry::Journal;
use kesh_db::entities::{NewJournalEntry, NewJournalEntryLine};
use kesh_db::repositories::journal_entries;
use kesh_db::test_fixtures::{SeededCompany, seed_accounting_company};
use kesh_report::csv::render_vat_report_csv;
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

/// Poste une écriture équilibrée à 2 lignes (débit/crédit) sur la période.
async fn post_entry(
    pool: &MySqlPool,
    seeded: &SeededCompany,
    date: NaiveDate,
    debit_account: i64,
    credit_account: i64,
    amount: Decimal,
) {
    journal_entries::create(
        pool,
        seeded.fiscal_year_id,
        seeded.admin_user_id,
        NewJournalEntry {
            company_id: seeded.company_id,
            entry_date: date,
            journal: Journal::Achats,
            description: "Achat TVA récupérable".into(),
            project_id: None,
            lines: vec![
                NewJournalEntryLine {
                    account_id: debit_account,
                    debit: amount,
                    credit: Decimal::ZERO,
                    project_id: None,
                },
                NewJournalEntryLine {
                    account_id: credit_account,
                    debit: Decimal::ZERO,
                    credit: amount,
                    project_id: None,
                },
            ],
        },
    )
    .await
    .unwrap();
}

/// (a)+(f) Une écriture récupérable → `total_vat_recoverable` = solde 1000 ;
/// sans vente, `vat_balance` est négatif (crédit d'impôt AFC).
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn recoverable_single_entry(pool: MySqlPool) {
    let seeded = seed_accounting_company(&pool).await.unwrap();
    // D 1000 (impôt préalable) 81 / C 2000 (contrepartie ≠ 1000, garde F3) 81.
    post_entry(
        &pool,
        &seeded,
        ymd(2026, 6, 15),
        seeded.accounts["1000"],
        seeded.accounts["2000"],
        dec!(81.00),
    )
    .await;

    let report = vat_report::generate(
        &pool,
        seeded.company_id,
        &period_2026(seeded.fiscal_year_id),
    )
    .await
    .unwrap();

    assert_eq!(report.total_vat_due, dec!(0), "aucune vente");
    assert_eq!(report.total_vat_recoverable, dec!(81.00), "solde 1000 = 81");
    assert_eq!(report.vat_balance, dec!(-81.00), "0 - 81 = crédit d'impôt");
}

/// (b) Plusieurs écritures (débits + un crédit de correction) → solde net.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn recoverable_multiple_entries_net(pool: MySqlPool) {
    let seeded = seed_accounting_company(&pool).await.unwrap();
    let recoverable = seeded.accounts["1000"];
    let counter = seeded.accounts["2000"];

    post_entry(
        &pool,
        &seeded,
        ymd(2026, 3, 1),
        recoverable,
        counter,
        dec!(50.00),
    )
    .await;
    post_entry(
        &pool,
        &seeded,
        ymd(2026, 4, 1),
        recoverable,
        counter,
        dec!(31.00),
    )
    .await;
    // Correction : crédit de 10 sur 1000 (D 2000 / C 1000).
    post_entry(
        &pool,
        &seeded,
        ymd(2026, 5, 1),
        counter,
        recoverable,
        dec!(10.00),
    )
    .await;

    let report = vat_report::generate(
        &pool,
        seeded.company_id,
        &period_2026(seeded.fiscal_year_id),
    )
    .await
    .unwrap();

    // SUM(debit) - SUM(credit) = (50 + 31) - 10 = 71.
    assert_eq!(report.total_vat_recoverable, dec!(71.00));
}

/// (c) Filtre période + bornes inclusives : hors-période exclu, bornes incluses.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn recoverable_period_filter_and_borders(pool: MySqlPool) {
    let seeded = seed_accounting_company(&pool).await.unwrap();
    let recoverable = seeded.accounts["1000"];
    let counter = seeded.accounts["2000"];

    post_entry(
        &pool,
        &seeded,
        ymd(2026, 6, 15),
        recoverable,
        counter,
        dec!(20.00),
    )
    .await; // dans
    post_entry(
        &pool,
        &seeded,
        ymd(2026, 1, 1),
        recoverable,
        counter,
        dec!(5.00),
    )
    .await; // borne start
    post_entry(
        &pool,
        &seeded,
        ymd(2026, 12, 31),
        recoverable,
        counter,
        dec!(7.00),
    )
    .await; // borne end
    post_entry(
        &pool,
        &seeded,
        ymd(2025, 12, 31),
        recoverable,
        counter,
        dec!(999.00),
    )
    .await; // hors (avant)
    post_entry(
        &pool,
        &seeded,
        ymd(2027, 1, 1),
        recoverable,
        counter,
        dec!(888.00),
    )
    .await; // hors (après)

    let report = vat_report::generate(
        &pool,
        seeded.company_id,
        &period_2026(seeded.fiscal_year_id),
    )
    .await
    .unwrap();

    // 20 + 5 (start) + 7 (end) = 32 ; les hors-période 999/888 exclus.
    assert_eq!(report.total_vat_recoverable, dec!(32.00));
}

/// (d) Compte récupérable non configuré (NULL) → 0.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn recoverable_account_null_returns_zero(pool: MySqlPool) {
    let seeded = seed_accounting_company(&pool).await.unwrap();
    // Dé-configurer le compte récupérable.
    sqlx::query(
        "UPDATE company_invoice_settings SET default_vat_recoverable_account_id = NULL \
         WHERE company_id = ?",
    )
    .bind(seeded.company_id)
    .execute(&pool)
    .await
    .unwrap();
    // Une écriture sur 1000 ne doit PAS être comptée (compte non configuré).
    post_entry(
        &pool,
        &seeded,
        ymd(2026, 6, 15),
        seeded.accounts["1000"],
        seeded.accounts["2000"],
        dec!(81.00),
    )
    .await;

    let report = vat_report::generate(
        &pool,
        seeded.company_id,
        &period_2026(seeded.fiscal_year_id),
    )
    .await
    .unwrap();

    assert_eq!(report.total_vat_recoverable, dec!(0));
    assert_eq!(report.vat_balance, dec!(0));
}

/// (i) Compte configuré mais AUCUNE écriture dans la période → 0 via le helper
/// (chemin `COALESCE(SUM,0)` 100 %-NULL, distinct du cas (d) NULL qui court-circuite
/// avant le helper). Une écriture hors-période garantit que le helper est bien
/// appelé mais ne compte rien.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn recoverable_configured_no_entry_in_period_returns_zero(pool: MySqlPool) {
    let seeded = seed_accounting_company(&pool).await.unwrap();
    // Une seule écriture, HORS période (2025) → le compte est configuré (1000), le
    // helper s'exécute, mais SUM des deux côtés est NULL sur 2026 → COALESCE → 0.
    post_entry(
        &pool,
        &seeded,
        ymd(2025, 6, 15),
        seeded.accounts["1000"],
        seeded.accounts["2000"],
        dec!(81.00),
    )
    .await;

    let report = vat_report::generate(
        &pool,
        seeded.company_id,
        &period_2026(seeded.fiscal_year_id),
    )
    .await
    .unwrap();
    assert_eq!(report.total_vat_recoverable, dec!(0));
}

/// (e) Anti-IDOR (scoping company) : un `company_id` autre que celui des écritures
/// retourne 0 (le filtre `je.company_id = ?` isole la company).
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn recoverable_scoped_by_company(pool: MySqlPool) {
    let seeded = seed_accounting_company(&pool).await.unwrap();
    post_entry(
        &pool,
        &seeded,
        ymd(2026, 6, 15),
        seeded.accounts["1000"],
        seeded.accounts["2000"],
        dec!(81.00),
    )
    .await;

    // Rapport pour une autre company (inexistante) → aucune fuite.
    let other_company = seeded.company_id + 99_999;
    let report = vat_report::generate(&pool, other_company, &period_2026(seeded.fiscal_year_id))
        .await
        .unwrap();
    assert_eq!(report.total_vat_recoverable, dec!(0));
}

/// (h) « Achats seuls » : récupérable sans vente reste rendu au CSV (AC10/T-D2b).
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn recoverable_only_rendered_in_csv(pool: MySqlPool) {
    let seeded = seed_accounting_company(&pool).await.unwrap();
    post_entry(
        &pool,
        &seeded,
        ymd(2026, 6, 15),
        seeded.accounts["1000"],
        seeded.accounts["2000"],
        dec!(81.00),
    )
    .await;

    let report = vat_report::generate(
        &pool,
        seeded.company_id,
        &period_2026(seeded.fiscal_year_id),
    )
    .await
    .unwrap();
    assert!(report.rows.is_empty(), "aucune vente → rows vide");
    assert_eq!(report.total_vat_recoverable, dec!(81.00));

    // Le CSV ne doit PAS court-circuiter : le récapitulatif récupérable apparaît.
    let mut buf = Vec::new();
    render_vat_report_csv(&report, &mut buf).unwrap();
    let csv = String::from_utf8(buf).unwrap();
    assert!(
        csv.contains("TVA récupérable"),
        "le CSV doit contenir le récapitulatif récupérable malgré 0 vente : {csv}"
    );
    assert!(csv.contains("81.00"), "valeur récupérable présente : {csv}");
}
