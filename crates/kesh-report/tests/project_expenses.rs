//! Tests d'intégration — rapport « Dépenses par projet » (Story 19-6a).
//!
//! Vérifie l'agrégation par (sous-projet, compte), le rollup 2 niveaux,
//! l'exclusion des lignes de contrepartie (non-Expense) taguées, le drill-down,
//! les deux modes de période (exercice / cumulé multi-exercices), et les 404.
//!
//! Pré-requis : MariaDB (`sqlx::test` crée une DB éphémère par test).

use chrono::NaiveDate;
use kesh_db::entities::journal_entry::Journal;
use kesh_db::entities::{NewJournalEntry, NewJournalEntryLine};
use kesh_db::repositories::journal_entries;
use kesh_db::test_fixtures::{SeededCompany, seed_accounting_company};
use kesh_report::errors::ReportError;
use kesh_report::period::ReportPeriod;
use kesh_report::project_report::{ProjectPeriodMode, generate_project_expenses, resolve_scope};
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use sqlx::MySqlPool;

fn ymd(y: i32, m: u32, d: u32) -> NaiveDate {
    NaiveDate::from_ymd_opt(y, m, d).unwrap()
}

fn fy_period(fiscal_year_id: i64) -> ProjectPeriodMode {
    ProjectPeriodMode::FiscalYear {
        period: ReportPeriod {
            fiscal_year_id,
            start_date: ymd(2026, 1, 1),
            end_date: ymd(2026, 12, 31),
        },
    }
}

/// Crée un projet (racine ou sous-projet) et retourne son id.
async fn mk_project(pool: &MySqlPool, company_id: i64, code: &str, parent_id: Option<i64>) -> i64 {
    let id: i64 = sqlx::query_scalar(
        "INSERT INTO projects (company_id, parent_id, code, name, archived) \
         VALUES (?, ?, ?, ?, FALSE) RETURNING id",
    )
    .bind(company_id)
    .bind(parent_id)
    .bind(code)
    .bind(format!("Projet {code}"))
    .fetch_one(pool)
    .await
    .expect("project insert");
    id
}

/// Crée un compte et retourne son id.
async fn mk_account(
    pool: &MySqlPool,
    company_id: i64,
    number: &str,
    name: &str,
    account_type: &str,
) -> i64 {
    sqlx::query_scalar(
        "INSERT INTO accounts (company_id, number, name, account_type, active) \
         VALUES (?, ?, ?, ?, TRUE) RETURNING id",
    )
    .bind(company_id)
    .bind(number)
    .bind(name)
    .bind(account_type)
    .fetch_one(pool)
    .await
    .expect("account insert")
}

/// Poste une écriture équilibrée à 2 lignes, chaque ligne éventuellement taguée.
#[allow(clippy::too_many_arguments)]
async fn post_tagged(
    pool: &MySqlPool,
    seeded: &SeededCompany,
    fiscal_year_id: i64,
    date: NaiveDate,
    debit_account: i64,
    debit_project: Option<i64>,
    credit_account: i64,
    credit_project: Option<i64>,
    amount: Decimal,
) {
    journal_entries::create(
        pool,
        fiscal_year_id,
        seeded.admin_user_id,
        NewJournalEntry {
            company_id: seeded.company_id,
            entry_date: date,
            journal: Journal::Achats,
            description: format!("Dépense {amount}"),
            project_id: None,
            lines: vec![
                NewJournalEntryLine {
                    account_id: debit_account,
                    debit: amount,
                    credit: Decimal::ZERO,
                    project_id: debit_project,
                },
                NewJournalEntryLine {
                    account_id: credit_account,
                    debit: Decimal::ZERO,
                    credit: amount,
                    project_id: credit_project,
                },
            ],
        },
    )
    .await
    .unwrap();
}

/// (a)+(c)+(d) — racine + sous-projet, 2 comptes de charge, lignes de
/// contrepartie taguées exclues, sous-totaux + total + drill-down.
#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn expenses_rollup_sections_and_drilldown(pool: MySqlPool) {
    let seeded = seed_accounting_company(&pool).await.unwrap();
    let root = mk_project(&pool, seeded.company_id, "RENOV", None).await;
    let child = mk_project(&pool, seeded.company_id, "RENOV-CHALET", Some(root)).await;
    let charge2 = mk_account(&pool, seeded.company_id, "4100", "Matériaux", "Expense").await;
    let charge1 = seeded.accounts["4000"];
    let counterparty = seeded.accounts["2000"]; // Liability — DOIT être exclu

    // Racine : 100 sur 4000 + 40 sur 4100. Contrepartie 2000 taguée root (exclue).
    post_tagged(
        &pool,
        &seeded,
        seeded.fiscal_year_id,
        ymd(2026, 3, 1),
        charge1,
        Some(root),
        counterparty,
        Some(root),
        dec!(100.00),
    )
    .await;
    post_tagged(
        &pool,
        &seeded,
        seeded.fiscal_year_id,
        ymd(2026, 4, 1),
        charge2,
        Some(root),
        counterparty,
        Some(root),
        dec!(40.00),
    )
    .await;
    // Sous-projet : 60 sur 4000.
    post_tagged(
        &pool,
        &seeded,
        seeded.fiscal_year_id,
        ymd(2026, 5, 1),
        charge1,
        Some(child),
        counterparty,
        Some(child),
        dec!(60.00),
    )
    .await;

    // Rapport RACINE : agrège root + child.
    let scope = resolve_scope(&pool, seeded.company_id, root).await.unwrap();
    assert_eq!(scope.project_ids.len(), 2, "racine + 1 enfant");
    let report = generate_project_expenses(
        &pool,
        seeded.company_id,
        &scope,
        &fy_period(seeded.fiscal_year_id),
    )
    .await
    .unwrap();

    assert_eq!(
        report.grand_total,
        dec!(200.00),
        "100+40+60, contreparties exclues"
    );
    assert_eq!(report.sections.len(), 2, "section root + section child");
    let root_sec = report.sections.iter().find(|s| s.is_root).unwrap();
    assert_eq!(root_sec.subtotal, dec!(140.00));
    assert_eq!(root_sec.rows.len(), 2, "comptes 4000 + 4100");
    // Drill-down : le compte 4000 racine a 1 écriture de 100.
    let row_4000 = root_sec
        .rows
        .iter()
        .find(|r| r.account_number == "4000")
        .unwrap();
    assert_eq!(row_4000.amount, dec!(100.00));
    assert_eq!(row_4000.entries.len(), 1);
    assert_eq!(row_4000.entries[0].amount, dec!(100.00));
    assert_eq!(row_4000.entries[0].entry_date, ymd(2026, 3, 1));

    let child_sec = report.sections.iter().find(|s| !s.is_root).unwrap();
    assert_eq!(child_sec.subtotal, dec!(60.00));

    // Rapport SOUS-PROJET ciblé : lui seul.
    let scope_child = resolve_scope(&pool, seeded.company_id, child)
        .await
        .unwrap();
    assert_eq!(scope_child.project_ids, vec![child]);
    let report_child = generate_project_expenses(
        &pool,
        seeded.company_id,
        &scope_child,
        &fy_period(seeded.fiscal_year_id),
    )
    .await
    .unwrap();
    assert_eq!(report_child.grand_total, dec!(60.00));
    assert_eq!(report_child.sections.len(), 1);
}

/// (b) — mode cumulé traverse deux exercices ; le mode exercice n'en voit qu'un.
#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn expenses_cumulative_crosses_fiscal_years(pool: MySqlPool) {
    let seeded = seed_accounting_company(&pool).await.unwrap();
    let proj = mk_project(&pool, seeded.company_id, "INVEST", None).await;
    let charge = seeded.accounts["4000"];
    let cp = seeded.accounts["2000"];

    // Un 2e exercice 2027 (le fixture couvre large ; on crée une FY dédiée 2027).
    let fy2027: i64 = sqlx::query_scalar(
        "INSERT INTO fiscal_years (company_id, name, start_date, end_date, status, created_at, updated_at) \
         VALUES (?, 'FY 2027', '2027-01-01', '2027-12-31', 'Open', NOW(3), NOW(3)) RETURNING id",
    )
    .bind(seeded.company_id)
    .fetch_one(&pool)
    .await
    .unwrap();

    post_tagged(
        &pool,
        &seeded,
        seeded.fiscal_year_id,
        ymd(2026, 6, 1),
        charge,
        Some(proj),
        cp,
        None,
        dec!(100.00),
    )
    .await;
    post_tagged(
        &pool,
        &seeded,
        fy2027,
        ymd(2027, 6, 1),
        charge,
        Some(proj),
        cp,
        None,
        dec!(70.00),
    )
    .await;

    let scope = resolve_scope(&pool, seeded.company_id, proj).await.unwrap();

    // Mode exercice 2026 → 100 seulement.
    let r_fy = generate_project_expenses(
        &pool,
        seeded.company_id,
        &scope,
        &fy_period(seeded.fiscal_year_id),
    )
    .await
    .unwrap();
    assert_eq!(r_fy.grand_total, dec!(100.00), "exercice 2026 seul");

    // Mode cumulé jusqu'à fin 2027 → 100 + 70.
    let cum = ProjectPeriodMode::Cumulative {
        start: None,
        end: ymd(2027, 12, 31),
    };
    let r_cum = generate_project_expenses(&pool, seeded.company_id, &scope, &cum)
        .await
        .unwrap();
    assert_eq!(r_cum.grand_total, dec!(170.00), "cumulé 2026+2027");
}

/// (e) — projet inconnu / cross-company → ProjectNotFound.
#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn expenses_unknown_project_is_not_found(pool: MySqlPool) {
    let seeded = seed_accounting_company(&pool).await.unwrap();
    let err = resolve_scope(&pool, seeded.company_id, 999_999)
        .await
        .unwrap_err();
    assert!(matches!(err, ReportError::ProjectNotFound { .. }));
}

/// (f) — scope sans ligne taguée → rapport vide (sections vides, total 0), pas d'erreur.
#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn expenses_empty_scope_is_empty_report(pool: MySqlPool) {
    let seeded = seed_accounting_company(&pool).await.unwrap();
    let proj = mk_project(&pool, seeded.company_id, "VIDE", None).await;
    let scope = resolve_scope(&pool, seeded.company_id, proj).await.unwrap();
    let report = generate_project_expenses(
        &pool,
        seeded.company_id,
        &scope,
        &fy_period(seeded.fiscal_year_id),
    )
    .await
    .unwrap();
    assert_eq!(report.grand_total, Decimal::ZERO);
    assert!(report.sections.is_empty());
    assert_eq!(report.project.code, "VIDE");
}
