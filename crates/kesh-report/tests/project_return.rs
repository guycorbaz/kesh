//! Tests d'intégration — rapport « Rendement par projet » (Story 19-6b).
//!
//! Vérifie coût investi (Expense + Asset hors trésorerie 10xx, DC8), revenus,
//! résultat net, rendement %, rollup, mode cumulé, scope vide.

use chrono::NaiveDate;
use kesh_db::entities::journal_entry::Journal;
use kesh_db::entities::{NewJournalEntry, NewJournalEntryLine};
use kesh_db::repositories::journal_entries;
use kesh_db::test_fixtures::{SeededCompany, seed_accounting_company};
use kesh_report::period::ReportPeriod;
use kesh_report::project_report::{ProjectPeriodMode, generate_project_return, resolve_scope};
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use sqlx::MySqlPool;

fn ymd(y: i32, m: u32, d: u32) -> NaiveDate {
    NaiveDate::from_ymd_opt(y, m, d).unwrap()
}

fn fy_mode(fiscal_year_id: i64) -> ProjectPeriodMode {
    ProjectPeriodMode::FiscalYear {
        period: ReportPeriod {
            fiscal_year_id,
            start_date: ymd(2026, 1, 1),
            end_date: ymd(2026, 12, 31),
        },
    }
}

async fn mk_project(pool: &MySqlPool, company_id: i64, code: &str, parent: Option<i64>) -> i64 {
    sqlx::query_scalar(
        "INSERT INTO projects (company_id, parent_id, code, name, archived) \
         VALUES (?, ?, ?, ?, FALSE) RETURNING id",
    )
    .bind(company_id)
    .bind(parent)
    .bind(code)
    .bind(format!("Projet {code}"))
    .fetch_one(pool)
    .await
    .unwrap()
}

async fn mk_account(pool: &MySqlPool, company_id: i64, number: &str, ty: &str) -> i64 {
    sqlx::query_scalar(
        "INSERT INTO accounts (company_id, number, name, account_type, active) \
         VALUES (?, ?, ?, ?, TRUE) RETURNING id",
    )
    .bind(company_id)
    .bind(number)
    .bind(format!("Compte {number}"))
    .bind(ty)
    .fetch_one(pool)
    .await
    .unwrap()
}

/// Poste une écriture équilibrée 2 lignes, chaque ligne taguée `project`.
#[allow(clippy::too_many_arguments)]
async fn post(
    pool: &MySqlPool,
    seeded: &SeededCompany,
    fy: i64,
    date: NaiveDate,
    debit_acc: i64,
    credit_acc: i64,
    amount: Decimal,
    project: i64,
) {
    journal_entries::create(
        pool,
        fy,
        seeded.admin_user_id,
        NewJournalEntry {
            company_id: seeded.company_id,
            entry_date: date,
            journal: Journal::OD,
            description: "mvt".into(),
            project_id: None,
            lines: vec![
                NewJournalEntryLine {
                    account_id: debit_acc,
                    debit: amount,
                    credit: Decimal::ZERO,
                    project_id: Some(project),
                },
                NewJournalEntryLine {
                    account_id: credit_acc,
                    debit: Decimal::ZERO,
                    credit: amount,
                    project_id: Some(project),
                },
            ],
        },
    )
    .await
    .unwrap();
}

/// (a) — coût investi = Expense + Asset immobilisé (1500), EXCLUT la banque 1020
/// (trésorerie 10xx, DC8) ; résultat net = revenus − charges ; rendement % correct.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn return_excludes_treasury_and_computes_rendement(pool: MySqlPool) {
    let seeded = seed_accounting_company(&pool).await.unwrap();
    let proj = mk_project(&pool, seeded.company_id, "INVEST", None).await;
    let charge = seeded.accounts["4000"]; // Expense
    let immo = mk_account(&pool, seeded.company_id, "1500", "Asset").await; // actif immobilisé
    let banque = seeded.accounts["1100"]; // Asset 11xx — NON exclu (seul 10xx exclu)
    let tresorerie = mk_account(&pool, seeded.company_id, "1020", "Asset").await; // 10xx exclu
    let revenue = seeded.accounts["3000"]; // Revenue
    let capital = seeded.accounts["2000"]; // Liability (contrepartie neutre)

    // Charge 100 (D 4000 / C 2000).
    post(
        &pool,
        &seeded,
        seeded.fiscal_year_id,
        ymd(2026, 3, 1),
        charge,
        capital,
        dec!(100.00),
        proj,
    )
    .await;
    // Immobilisation 300 (D 1500 / C 2000) → compte dans coût investi.
    post(
        &pool,
        &seeded,
        seeded.fiscal_year_id,
        ymd(2026, 3, 2),
        immo,
        capital,
        dec!(300.00),
        proj,
    )
    .await;
    // Banque 11xx 50 (D 1100 / C 2000) → 1100 est Asset non-10xx → COMPTE dans coût.
    post(
        &pool,
        &seeded,
        seeded.fiscal_year_id,
        ymd(2026, 3, 3),
        banque,
        capital,
        dec!(50.00),
        proj,
    )
    .await;
    // Trésorerie 1020 999 (D 1020 / C 2000) → 10xx → EXCLU du coût.
    post(
        &pool,
        &seeded,
        seeded.fiscal_year_id,
        ymd(2026, 3, 4),
        tresorerie,
        capital,
        dec!(999.00),
        proj,
    )
    .await;
    // Revenu 200 (D 2000 / C 3000).
    post(
        &pool,
        &seeded,
        seeded.fiscal_year_id,
        ymd(2026, 4, 1),
        capital,
        revenue,
        dec!(200.00),
        proj,
    )
    .await;

    let scope = resolve_scope(&pool, seeded.company_id, proj).await.unwrap();
    let report = generate_project_return(
        &pool,
        seeded.company_id,
        &scope,
        &fy_mode(seeded.fiscal_year_id),
    )
    .await
    .unwrap();

    // Coût investi = charges 100 + immo 300 + banque11xx 50 = 450 (1020 exclu).
    assert_eq!(
        report.totals.cout_investi,
        dec!(450.00),
        "1020 (10xx) doit être exclu du coût investi"
    );
    assert_eq!(report.totals.revenus, dec!(200.00));
    // Résultat net = revenus 200 − charges 100 = 100.
    assert_eq!(report.totals.resultat_net, dec!(100.00));
    // Rendement % = 200 / 450 * 100 = 44.44.
    assert_eq!(report.totals.rendement_pct, Some(dec!(44.44)));
}

/// (b) — rendement null si coût investi = 0 (que des revenus).
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn return_rendement_null_when_no_cost(pool: MySqlPool) {
    let seeded = seed_accounting_company(&pool).await.unwrap();
    let proj = mk_project(&pool, seeded.company_id, "REVONLY", None).await;
    let revenue = seeded.accounts["3000"];
    let capital = seeded.accounts["2000"];
    post(
        &pool,
        &seeded,
        seeded.fiscal_year_id,
        ymd(2026, 4, 1),
        capital,
        revenue,
        dec!(500.00),
        proj,
    )
    .await;

    let scope = resolve_scope(&pool, seeded.company_id, proj).await.unwrap();
    let report = generate_project_return(
        &pool,
        seeded.company_id,
        &scope,
        &fy_mode(seeded.fiscal_year_id),
    )
    .await
    .unwrap();
    assert_eq!(report.totals.cout_investi, Decimal::ZERO);
    assert_eq!(report.totals.revenus, dec!(500.00));
    assert_eq!(
        report.totals.rendement_pct, None,
        "coût investi 0 → rendement null"
    );
}

/// (c) — rollup racine + sous-projet.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn return_rollup_root_and_child(pool: MySqlPool) {
    let seeded = seed_accounting_company(&pool).await.unwrap();
    let root = mk_project(&pool, seeded.company_id, "ROOT", None).await;
    let child = mk_project(&pool, seeded.company_id, "ROOT-A", Some(root)).await;
    let charge = seeded.accounts["4000"];
    let capital = seeded.accounts["2000"];
    post(
        &pool,
        &seeded,
        seeded.fiscal_year_id,
        ymd(2026, 3, 1),
        charge,
        capital,
        dec!(100.00),
        root,
    )
    .await;
    post(
        &pool,
        &seeded,
        seeded.fiscal_year_id,
        ymd(2026, 3, 2),
        charge,
        capital,
        dec!(60.00),
        child,
    )
    .await;

    let scope = resolve_scope(&pool, seeded.company_id, root).await.unwrap();
    let report = generate_project_return(
        &pool,
        seeded.company_id,
        &scope,
        &fy_mode(seeded.fiscal_year_id),
    )
    .await
    .unwrap();
    assert_eq!(report.sections.len(), 2);
    assert_eq!(report.totals.cout_investi, dec!(160.00), "rollup 100 + 60");
}

/// (d) — mode cumulé traverse 2 exercices.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn return_cumulative_crosses_fy(pool: MySqlPool) {
    let seeded = seed_accounting_company(&pool).await.unwrap();
    let proj = mk_project(&pool, seeded.company_id, "CUM", None).await;
    let charge = seeded.accounts["4000"];
    let capital = seeded.accounts["2000"];
    let fy2027: i64 = sqlx::query_scalar(
        "INSERT INTO fiscal_years (company_id, name, start_date, end_date, status, created_at, updated_at) \
         VALUES (?, 'FY 2027', '2027-01-01', '2027-12-31', 'Open', NOW(3), NOW(3)) RETURNING id",
    )
    .bind(seeded.company_id)
    .fetch_one(&pool)
    .await
    .unwrap();
    post(
        &pool,
        &seeded,
        seeded.fiscal_year_id,
        ymd(2026, 6, 1),
        charge,
        capital,
        dec!(100.00),
        proj,
    )
    .await;
    post(
        &pool,
        &seeded,
        fy2027,
        ymd(2027, 6, 1),
        charge,
        capital,
        dec!(40.00),
        proj,
    )
    .await;

    let scope = resolve_scope(&pool, seeded.company_id, proj).await.unwrap();
    let cum = ProjectPeriodMode::Cumulative {
        start: None,
        end: ymd(2027, 12, 31),
    };
    let report = generate_project_return(&pool, seeded.company_id, &scope, &cum)
        .await
        .unwrap();
    assert_eq!(report.totals.cout_investi, dec!(140.00), "cumulé 100 + 40");
}

/// (e) — scope vide → totaux 0, rendement null, pas d'erreur.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn return_empty_scope(pool: MySqlPool) {
    let seeded = seed_accounting_company(&pool).await.unwrap();
    let proj = mk_project(&pool, seeded.company_id, "VIDE", None).await;
    let scope = resolve_scope(&pool, seeded.company_id, proj).await.unwrap();
    let report = generate_project_return(
        &pool,
        seeded.company_id,
        &scope,
        &fy_mode(seeded.fiscal_year_id),
    )
    .await
    .unwrap();
    assert!(report.sections.is_empty());
    assert_eq!(report.totals.cout_investi, Decimal::ZERO);
    assert_eq!(report.totals.rendement_pct, None);
}
