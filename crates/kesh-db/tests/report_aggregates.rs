//! Tests sqlx Story 9-1 — agrégats SQL des rapports comptables.
//!
//! Niveau repository : teste directement `kesh_report::generate_*` sans HTTP,
//! pour valider les patterns SQL (COALESCE, période bounds, fy isolation,
//! multi-tenant, exercice non-calendaire).
//!
//! Complémentaire de `crates/kesh-api/tests/reports_e2e.rs` qui teste le
//! end-to-end HTTP (parsing JWT, response shape, codes erreur).
//!
//! Pattern : `#[sqlx::test]` avec migrator. Seed inline minimal.
#![allow(clippy::too_many_arguments)]

use chrono::NaiveDate;
use kesh_db::entities::account::AccountType;
use kesh_db::entities::address::StructuredAddress;
use kesh_db::entities::journal_entry::Journal;
use kesh_db::entities::{
    AccountRole, Language, NewAccount, NewCompany, NewFiscalYear, NewJournalEntry,
    NewJournalEntryLine, NewUser, OrgType, Role,
};
use kesh_db::repositories::{accounts, companies, fiscal_years, journal_entries, users};
use kesh_report::ReportPeriod;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use sqlx::MySqlPool;

// ============================================================
// Seed helpers minimal
// ============================================================

async fn create_company(pool: &MySqlPool, name: &str) -> i64 {
    companies::create(
        pool,
        NewCompany {
            name: name.into(),
            first_name: None,
            last_name: None,
            address_structured: StructuredAddress {
                street: "X".into(),
                building: String::new(),
                postal_code: "1000".into(),
                city: "Lausanne".into(),
                country: "CH".into(),
            },
            ide_number: None,
            org_type: OrgType::Independant,
            accounting_language: Language::Fr,
            instance_language: Language::Fr,
        },
    )
    .await
    .unwrap()
    .id
}

async fn create_user(pool: &MySqlPool, username: &str, company_id: i64) -> i64 {
    users::create(
        pool,
        NewUser {
            username: username.into(),
            password_hash: "$argon2id$v=19$m=19456,t=2,p=1$YWFhYWFhYWFhYWFhYWFhYQ$0000000000000000000000000000000000000000000".into(),
            role: Role::Comptable,
            active: true,
            company_id,
            email: None,
        },
    )
    .await
    .unwrap()
    .id
}

async fn create_fy(
    pool: &MySqlPool,
    user_id: i64,
    company_id: i64,
    name: &str,
    start: NaiveDate,
    end: NaiveDate,
) -> i64 {
    fiscal_years::create(
        pool,
        user_id,
        NewFiscalYear {
            company_id,
            name: name.into(),
            start_date: start,
            end_date: end,
        },
    )
    .await
    .unwrap()
    .id
}

async fn create_acc(
    pool: &MySqlPool,
    user_id: i64,
    company_id: i64,
    number: &str,
    name: &str,
    account_type: AccountType,
) -> i64 {
    create_acc_with_role(pool, user_id, company_id, number, name, account_type, None).await
}

/// Variante de [`create_acc`] posant un **rôle** explicite (Story 14-3c) — nécessaire
/// pour piloter la partition fonds propres / dettes du bilan par rôle. `create_acc`
/// délègue ici avec `role: None` (les ~21 sites d'appel existants restent inchangés).
async fn create_acc_with_role(
    pool: &MySqlPool,
    user_id: i64,
    company_id: i64,
    number: &str,
    name: &str,
    account_type: AccountType,
    role: Option<AccountRole>,
) -> i64 {
    accounts::create(
        pool,
        user_id,
        NewAccount {
            company_id,
            number: number.into(),
            name: name.into(),
            account_type,
            parent_id: None,
            role,
            postable: true,
        },
    )
    .await
    .unwrap()
    .id
}

/// Force le `role` d'un compte **après** coup, en contournant la garde de postabilité
/// (Story 14-3b) qui bloquerait une écriture sur un rôle non-postable. Modélise le
/// scénario legacy : un solde a été posté quand le compte était postable, PUIS 14-3a a
/// assigné un rôle non-postable (`CurrentYearResult`). Le rapport lit `accounts.role`
/// au moment du calcul, donc la partition par rôle s'applique au solde legacy.
async fn set_account_role(pool: &MySqlPool, account_id: i64, role: AccountRole) {
    sqlx::query("UPDATE accounts SET role = ?, postable = FALSE WHERE id = ?")
        .bind(role.as_str())
        .bind(account_id)
        .execute(pool)
        .await
        .unwrap();
}

async fn post_entry(
    pool: &MySqlPool,
    user_id: i64,
    fy_id: i64,
    company_id: i64,
    date: NaiveDate,
    journal: Journal,
    debit_account: i64,
    credit_account: i64,
    amount: Decimal,
) {
    journal_entries::create(
        pool,
        fy_id,
        user_id,
        NewJournalEntry {
            company_id,
            entry_date: date,
            journal,
            description: "test".into(),
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

// ============================================================
// Tests 1-7
// ============================================================

/// Test 1 : balance_sheet sur 0 écriture → COALESCE retourne 0, équation tient
#[sqlx::test(migrations = "./test-schema")]
async fn balance_sheet_empty_returns_zero_with_equation(pool: MySqlPool) {
    let cid = create_company(&pool, "co1").await;
    let uid = create_user(&pool, "u1", cid).await;
    let fy = create_fy(
        &pool,
        uid,
        cid,
        "FY",
        NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
        NaiveDate::from_ymd_opt(2026, 12, 31).unwrap(),
    )
    .await;

    let period = ReportPeriod::resolve(&pool, cid, fy, None, None)
        .await
        .unwrap();
    let bs = kesh_report::generate_balance_sheet(&pool, cid, &period)
        .await
        .unwrap();
    assert_eq!(bs.total_assets, Decimal::ZERO);
    assert_eq!(bs.total_liabilities, Decimal::ZERO);
    assert_eq!(bs.equity_result, Decimal::ZERO);
    assert!(bs.equation_holds);
}

/// Test 2 : fiscal_year non-calendaire (juillet → juin), isolation correcte
#[sqlx::test(migrations = "./test-schema")]
async fn fiscal_year_non_calendar_isolation(pool: MySqlPool) {
    let cid = create_company(&pool, "co2").await;
    let uid = create_user(&pool, "u2", cid).await;
    let fy = create_fy(
        &pool,
        uid,
        cid,
        "FY",
        NaiveDate::from_ymd_opt(2026, 7, 1).unwrap(),
        NaiveDate::from_ymd_opt(2027, 6, 30).unwrap(),
    )
    .await;
    let asset = create_acc(&pool, uid, cid, "1000", "Banque", AccountType::Asset).await;
    let revenue = create_acc(&pool, uid, cid, "4000", "Ventes", AccountType::Revenue).await;
    // Écriture au milieu (calendrier qui chevauche 2 années civiles)
    post_entry(
        &pool,
        uid,
        fy,
        cid,
        NaiveDate::from_ymd_opt(2027, 1, 15).unwrap(),
        Journal::Banque,
        asset,
        revenue,
        dec!(500),
    )
    .await;

    let period = ReportPeriod::resolve(&pool, cid, fy, None, None)
        .await
        .unwrap();
    let tb = kesh_report::generate_trial_balance(&pool, cid, &period)
        .await
        .unwrap();
    assert!(tb.balanced);
    assert_eq!(tb.total_debit, dec!(500));
}

/// Test 3 : multi-tenant strict cross-company aggregation = 0
#[sqlx::test(migrations = "./test-schema")]
async fn cross_tenant_aggregation_returns_zero(pool: MySqlPool) {
    let cid_a = create_company(&pool, "co3a").await;
    let cid_b = create_company(&pool, "co3b").await;
    let uid_a = create_user(&pool, "ua", cid_a).await;
    let uid_b = create_user(&pool, "ub", cid_b).await;
    let fy_a = create_fy(
        &pool,
        uid_a,
        cid_a,
        "FYA",
        NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
        NaiveDate::from_ymd_opt(2026, 12, 31).unwrap(),
    )
    .await;
    let fy_b = create_fy(
        &pool,
        uid_b,
        cid_b,
        "FYB",
        NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
        NaiveDate::from_ymd_opt(2026, 12, 31).unwrap(),
    )
    .await;
    let asset_b = create_acc(&pool, uid_b, cid_b, "1000", "Banque B", AccountType::Asset).await;
    let revenue_b = create_acc(
        &pool,
        uid_b,
        cid_b,
        "4000",
        "Ventes B",
        AccountType::Revenue,
    )
    .await;
    post_entry(
        &pool,
        uid_b,
        fy_b,
        cid_b,
        NaiveDate::from_ymd_opt(2026, 5, 15).unwrap(),
        Journal::Banque,
        asset_b,
        revenue_b,
        dec!(1234),
    )
    .await;

    // User A fait trial_balance sur fy_a (sans écritures, pas d'accounts non plus)
    let period = ReportPeriod::resolve(&pool, cid_a, fy_a, None, None)
        .await
        .unwrap();
    let tb = kesh_report::generate_trial_balance(&pool, cid_a, &period)
        .await
        .unwrap();
    assert_eq!(tb.total_debit, Decimal::ZERO);
    assert_eq!(tb.total_credit, Decimal::ZERO);
}

/// Test 4 : période bounds inclusives (start et end exactement sur fy bornes)
#[sqlx::test(migrations = "./test-schema")]
async fn period_bounds_inclusive(pool: MySqlPool) {
    let cid = create_company(&pool, "co4").await;
    let uid = create_user(&pool, "u4", cid).await;
    let fy = create_fy(
        &pool,
        uid,
        cid,
        "FY",
        NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
        NaiveDate::from_ymd_opt(2026, 12, 31).unwrap(),
    )
    .await;
    let asset = create_acc(&pool, uid, cid, "1000", "B", AccountType::Asset).await;
    let revenue = create_acc(&pool, uid, cid, "4000", "V", AccountType::Revenue).await;
    // Écriture pile sur le 1er jour
    post_entry(
        &pool,
        uid,
        fy,
        cid,
        NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
        Journal::Banque,
        asset,
        revenue,
        dec!(100),
    )
    .await;
    // Écriture pile sur le dernier jour
    post_entry(
        &pool,
        uid,
        fy,
        cid,
        NaiveDate::from_ymd_opt(2026, 12, 31).unwrap(),
        Journal::Banque,
        asset,
        revenue,
        dec!(200),
    )
    .await;

    let period = ReportPeriod::resolve(&pool, cid, fy, None, None)
        .await
        .unwrap();
    let tb = kesh_report::generate_trial_balance(&pool, cid, &period)
        .await
        .unwrap();
    // Les 2 écritures sont incluses → total 300 débit + 300 crédit
    assert_eq!(tb.total_debit, dec!(300));
    assert_eq!(tb.total_credit, dec!(300));
}

/// Test 5 : partial period exclut les écritures hors bornes
#[sqlx::test(migrations = "./test-schema")]
async fn partial_period_excludes_outside_entries(pool: MySqlPool) {
    let cid = create_company(&pool, "co5").await;
    let uid = create_user(&pool, "u5", cid).await;
    let fy = create_fy(
        &pool,
        uid,
        cid,
        "FY",
        NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
        NaiveDate::from_ymd_opt(2026, 12, 31).unwrap(),
    )
    .await;
    let asset = create_acc(&pool, uid, cid, "1000", "B", AccountType::Asset).await;
    let revenue = create_acc(&pool, uid, cid, "4000", "V", AccountType::Revenue).await;
    post_entry(
        &pool,
        uid,
        fy,
        cid,
        NaiveDate::from_ymd_opt(2026, 3, 1).unwrap(),
        Journal::Banque,
        asset,
        revenue,
        dec!(100),
    )
    .await;
    post_entry(
        &pool,
        uid,
        fy,
        cid,
        NaiveDate::from_ymd_opt(2026, 9, 1).unwrap(),
        Journal::Banque,
        asset,
        revenue,
        dec!(200),
    )
    .await;

    // Période Q2 (avril-juin) : exclut les 2 écritures
    let period = ReportPeriod::resolve(
        &pool,
        cid,
        fy,
        Some(NaiveDate::from_ymd_opt(2026, 4, 1).unwrap()),
        Some(NaiveDate::from_ymd_opt(2026, 6, 30).unwrap()),
    )
    .await
    .unwrap();
    let tb = kesh_report::generate_trial_balance(&pool, cid, &period)
        .await
        .unwrap();
    assert_eq!(tb.total_debit, Decimal::ZERO);
}

/// Test 6 : balance_sheet **partitionne** le compte 2979 (rôle `CurrentYearResult`)
/// dans la section **Capitaux propres**, pas dans les Passifs (Story 14-3c).
///
/// Historique : l'ancien `balance_sheet_excludes_2979_from_liabilities` (hardcode
/// numéro, retiré 14-1) comptait 2979 dans `liabilities` ; 14-1
/// (`balance_sheet_counts_2979_in_liabilities`) l'y laissait faute de rôle. Depuis
/// 14-3c la classification se fait **par rôle** : un compte de rôle equity va dans
/// `equity`, jamais dans `liabilities`. La partition reste indépendante du numéro
/// (le fixture pose `role: Some(CurrentYearResult)`, sinon `role: None` le laisserait
/// en dettes et le test échouerait pour la mauvaise raison — Piège #6).
#[sqlx::test(migrations = "./test-schema")]
async fn balance_sheet_counts_2979_in_equity(pool: MySqlPool) {
    let cid = create_company(&pool, "co6").await;
    let uid = create_user(&pool, "u6", cid).await;
    let fy = create_fy(
        &pool,
        uid,
        cid,
        "FY",
        NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
        NaiveDate::from_ymd_opt(2026, 12, 31).unwrap(),
    )
    .await;
    let asset = create_acc(&pool, uid, cid, "1000", "Banque", AccountType::Asset).await;
    // Solde legacy non-nul : posté quand 2979 était postable (rôle None), PUIS 14-3a a
    // assigné le rôle non-postable CurrentYearResult (garde postabilité 14-3b bloquerait
    // un posting direct sur ce rôle — cf. `set_account_role`).
    let acc_2979 = create_acc(
        &pool,
        uid,
        cid,
        "2979",
        "Report à nouveau",
        AccountType::Liability,
    )
    .await;
    post_entry(
        &pool,
        uid,
        fy,
        cid,
        NaiveDate::from_ymd_opt(2026, 5, 1).unwrap(),
        Journal::OD,
        asset,
        acc_2979,
        dec!(500),
    )
    .await;
    set_account_role(&pool, acc_2979, AccountRole::CurrentYearResult).await;

    let period = ReportPeriod::resolve(&pool, cid, fy, None, None)
        .await
        .unwrap();
    let bs = kesh_report::generate_balance_sheet(&pool, cid, &period)
        .await
        .unwrap();
    // Le compte 2979 DOIT apparaître dans `equity` (rôle equity), PAS dans `liabilities`.
    assert!(
        bs.equity.iter().any(|a| a.account_number == "2979"),
        "Compte 2979 (rôle CurrentYearResult) doit être partitionné dans equity (14-3c)"
    );
    assert!(
        !bs.liabilities.iter().any(|a| a.account_number == "2979"),
        "Compte 2979 ne doit PLUS figurer dans liabilities (partition par rôle 14-3c)"
    );
    assert_eq!(bs.total_liabilities, Decimal::ZERO);
    assert_eq!(bs.total_equity, dec!(500));
    assert_eq!(bs.total_assets, dec!(500));
    assert_eq!(bs.retained_earnings, Decimal::ZERO);
    assert_eq!(bs.equity_result, Decimal::ZERO);
    assert!(bs.equation_holds);
}

/// Test 8 : report à-nouveau virtuel cross-exercice (Story 14-1 AC-A/B, fixture AC-I).
///
/// FY2025 : actifs 15 000 / passifs 10 000 / résultat net 5 000 (reste vivant dans les
/// comptes Revenue/Expense — AUCUNE écriture de clôture). FY2026 : une écriture +200 de
/// produit (débit actif 200 / crédit produit 200). Attendus au bilan FY2026 : actifs
/// cumulés **15 200**, `retained_earnings` **5 000**, `equity_result` **200**, équation
/// `15 200 == 10 000 + 5 000 + 200`.
#[sqlx::test(migrations = "./test-schema")]
async fn balance_sheet_virtual_carryforward_cross_fiscal_year(pool: MySqlPool) {
    let cid = create_company(&pool, "co8").await;
    let uid = create_user(&pool, "u8", cid).await;
    let fy2025 = create_fy(
        &pool,
        uid,
        cid,
        "FY2025",
        NaiveDate::from_ymd_opt(2025, 1, 1).unwrap(),
        NaiveDate::from_ymd_opt(2025, 12, 31).unwrap(),
    )
    .await;
    let fy2026 = create_fy(
        &pool,
        uid,
        cid,
        "FY2026",
        NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
        NaiveDate::from_ymd_opt(2026, 12, 31).unwrap(),
    )
    .await;
    let asset = create_acc(&pool, uid, cid, "1000", "Banque", AccountType::Asset).await;
    let liab = create_acc(&pool, uid, cid, "2000", "Dettes", AccountType::Liability).await;
    let revenue = create_acc(&pool, uid, cid, "3000", "Ventes", AccountType::Revenue).await;

    // FY2025 : passifs 10 000 (actif 10 000 en contrepartie)
    post_entry(
        &pool,
        uid,
        fy2025,
        cid,
        NaiveDate::from_ymd_opt(2025, 3, 1).unwrap(),
        Journal::OD,
        asset,
        liab,
        dec!(10000),
    )
    .await;
    // FY2025 : résultat net 5 000 (actif +5 000 / produit 5 000)
    post_entry(
        &pool,
        uid,
        fy2025,
        cid,
        NaiveDate::from_ymd_opt(2025, 6, 1).unwrap(),
        Journal::Ventes,
        asset,
        revenue,
        dec!(5000),
    )
    .await;
    // FY2026 : +200 de produit
    post_entry(
        &pool,
        uid,
        fy2026,
        cid,
        NaiveDate::from_ymd_opt(2026, 2, 1).unwrap(),
        Journal::Ventes,
        asset,
        revenue,
        dec!(200),
    )
    .await;

    // Bilan FY2026
    let period = ReportPeriod::resolve(&pool, cid, fy2026, None, None)
        .await
        .unwrap();
    let bs = kesh_report::generate_balance_sheet(&pool, cid, &period)
        .await
        .unwrap();
    assert_eq!(
        bs.total_assets,
        dec!(15200),
        "actifs cumulés depuis l'origine"
    );
    assert_eq!(bs.total_liabilities, dec!(10000), "passifs cumulés");
    assert_eq!(
        bs.retained_earnings,
        dec!(5000),
        "résultat reporté = P&L des exercices antérieurs"
    );
    assert_eq!(
        bs.equity_result,
        dec!(200),
        "résultat de l'exercice courant"
    );
    assert!(bs.equation_holds, "15200 == 10000 + 5000 + 200");

    // Bilan FY2025 (1er exercice) : retained 0, equity 5000 — AC-D régression zéro
    let period25 = ReportPeriod::resolve(&pool, cid, fy2025, None, None)
        .await
        .unwrap();
    let bs25 = kesh_report::generate_balance_sheet(&pool, cid, &period25)
        .await
        .unwrap();
    assert_eq!(bs25.total_assets, dec!(15000));
    assert_eq!(bs25.total_liabilities, dec!(10000));
    assert_eq!(bs25.retained_earnings, Decimal::ZERO, "1er exercice → 0");
    assert_eq!(bs25.equity_result, dec!(5000));
    assert!(bs25.equation_holds);
}

/// Test 9 : résultat reporté **négatif** (pertes cumulées) + exercice N+1 vide qui
/// équilibre sans écriture (Story 14-1 AC-C + AC-I cas b).
#[sqlx::test(migrations = "./test-schema")]
async fn balance_sheet_negative_retained_and_empty_next_year(pool: MySqlPool) {
    let cid = create_company(&pool, "co9").await;
    let uid = create_user(&pool, "u9", cid).await;
    let fy2025 = create_fy(
        &pool,
        uid,
        cid,
        "FY2025",
        NaiveDate::from_ymd_opt(2025, 1, 1).unwrap(),
        NaiveDate::from_ymd_opt(2025, 12, 31).unwrap(),
    )
    .await;
    let fy2026 = create_fy(
        &pool,
        uid,
        cid,
        "FY2026",
        NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
        NaiveDate::from_ymd_opt(2026, 12, 31).unwrap(),
    )
    .await;
    let asset = create_acc(&pool, uid, cid, "1000", "Banque", AccountType::Asset).await;
    let liab = create_acc(&pool, uid, cid, "2000", "Dettes", AccountType::Liability).await;
    let expense = create_acc(&pool, uid, cid, "4000", "Charges", AccountType::Expense).await;

    // FY2025 : actif 10 000 / dette 10 000
    post_entry(
        &pool,
        uid,
        fy2025,
        cid,
        NaiveDate::from_ymd_opt(2025, 3, 1).unwrap(),
        Journal::OD,
        asset,
        liab,
        dec!(10000),
    )
    .await;
    // FY2025 : perte 3 000 (charge 3 000 / actif -3 000)
    post_entry(
        &pool,
        uid,
        fy2025,
        cid,
        NaiveDate::from_ymd_opt(2025, 8, 1).unwrap(),
        Journal::OD,
        expense,
        asset,
        dec!(3000),
    )
    .await;

    // Bilan FY2026 SANS aucune écriture 2026 → équilibre + reflète soldes clôture N
    let period = ReportPeriod::resolve(&pool, cid, fy2026, None, None)
        .await
        .unwrap();
    let bs = kesh_report::generate_balance_sheet(&pool, cid, &period)
        .await
        .unwrap();
    assert_eq!(bs.total_assets, dec!(7000), "10000 - 3000 cumulés");
    assert_eq!(bs.total_liabilities, dec!(10000));
    assert_eq!(bs.retained_earnings, dec!(-3000), "perte reportée");
    assert_eq!(bs.equity_result, Decimal::ZERO, "aucune écriture 2026");
    assert!(bs.equation_holds, "7000 == 10000 + (-3000) + 0");
}

/// Test 10 : `period_start` **sans effet** sur le bilan cumulatif (Story 14-1 AC-A).
///
/// Deux appels à la même date d'arrêté (mi-année) mais avec des `period_start`
/// différents (défaut fy_start vs mi-mars) donnent des soldes et un split fonds propres
/// **identiques** — l'ancrage est `fy_start`, jamais `period_start` (AC-I cas d).
#[sqlx::test(migrations = "./test-schema")]
async fn balance_sheet_period_start_has_no_effect(pool: MySqlPool) {
    let cid = create_company(&pool, "co10").await;
    let uid = create_user(&pool, "u10", cid).await;
    let fy2025 = create_fy(
        &pool,
        uid,
        cid,
        "FY2025",
        NaiveDate::from_ymd_opt(2025, 1, 1).unwrap(),
        NaiveDate::from_ymd_opt(2025, 12, 31).unwrap(),
    )
    .await;
    let fy2026 = create_fy(
        &pool,
        uid,
        cid,
        "FY2026",
        NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
        NaiveDate::from_ymd_opt(2026, 12, 31).unwrap(),
    )
    .await;
    let asset = create_acc(&pool, uid, cid, "1000", "Banque", AccountType::Asset).await;
    let revenue = create_acc(&pool, uid, cid, "3000", "Ventes", AccountType::Revenue).await;
    // FY2025 résultat 5000
    post_entry(
        &pool,
        uid,
        fy2025,
        cid,
        NaiveDate::from_ymd_opt(2025, 6, 1).unwrap(),
        Journal::Ventes,
        asset,
        revenue,
        dec!(5000),
    )
    .await;
    // FY2026 : produit 100 en février, 100 en avril
    post_entry(
        &pool,
        uid,
        fy2026,
        cid,
        NaiveDate::from_ymd_opt(2026, 2, 1).unwrap(),
        Journal::Ventes,
        asset,
        revenue,
        dec!(100),
    )
    .await;
    post_entry(
        &pool,
        uid,
        fy2026,
        cid,
        NaiveDate::from_ymd_opt(2026, 4, 1).unwrap(),
        Journal::Ventes,
        asset,
        revenue,
        dec!(100),
    )
    .await;

    let arrete = NaiveDate::from_ymd_opt(2026, 6, 30).unwrap();
    // (a) period_start = défaut (fy_start)
    let p_default = ReportPeriod::resolve(&pool, cid, fy2026, None, Some(arrete))
        .await
        .unwrap();
    let bs_default = kesh_report::generate_balance_sheet(&pool, cid, &p_default)
        .await
        .unwrap();
    // (b) period_start = mi-mars (après la 1ère écriture de fév.)
    let p_mid = ReportPeriod::resolve(
        &pool,
        cid,
        fy2026,
        Some(NaiveDate::from_ymd_opt(2026, 3, 15).unwrap()),
        Some(arrete),
    )
    .await
    .unwrap();
    let bs_mid = kesh_report::generate_balance_sheet(&pool, cid, &p_mid)
        .await
        .unwrap();

    // period_start sans effet : soldes et split fonds propres identiques
    assert_eq!(bs_default.total_assets, bs_mid.total_assets);
    assert_eq!(bs_default.total_liabilities, bs_mid.total_liabilities);
    assert_eq!(bs_default.retained_earnings, bs_mid.retained_earnings);
    assert_eq!(bs_default.equity_result, bs_mid.equity_result);
    // Valeurs attendues : retained 5000 (FY2025), equity_result 200 (YTD 2026), actifs 5200
    assert_eq!(bs_default.retained_earnings, dec!(5000));
    assert_eq!(bs_default.equity_result, dec!(200));
    assert_eq!(bs_default.total_assets, dec!(5200));
    assert!(bs_default.equation_holds);
}

/// Test 11 : garde défensive `create_in_tx` — une écriture datée hors des bornes de son
/// exercice est **rejetée** (Story 14-1 Dev Note 4, invariant dont dépend l'équation).
#[sqlx::test(migrations = "./test-schema")]
async fn journal_entry_out_of_fiscal_year_bounds_rejected(pool: MySqlPool) {
    use kesh_db::entities::{NewJournalEntry, NewJournalEntryLine};
    let cid = create_company(&pool, "co11").await;
    let uid = create_user(&pool, "u11", cid).await;
    let fy = create_fy(
        &pool,
        uid,
        cid,
        "FY2026",
        NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
        NaiveDate::from_ymd_opt(2026, 12, 31).unwrap(),
    )
    .await;
    let asset = create_acc(&pool, uid, cid, "1000", "Banque", AccountType::Asset).await;
    let revenue = create_acc(&pool, uid, cid, "3000", "Ventes", AccountType::Revenue).await;

    // entry_date 2027-01-15 hors de [2026-01-01, 2026-12-31]
    let err = journal_entries::create(
        &pool,
        fy,
        uid,
        NewJournalEntry {
            company_id: cid,
            entry_date: NaiveDate::from_ymd_opt(2027, 1, 15).unwrap(),
            journal: Journal::OD,
            description: "hors bornes".into(),
            project_id: None,
            lines: vec![
                NewJournalEntryLine {
                    account_id: asset,
                    debit: dec!(100),
                    credit: Decimal::ZERO,
                    project_id: None,
                },
                NewJournalEntryLine {
                    account_id: revenue,
                    debit: Decimal::ZERO,
                    credit: dec!(100),
                    project_id: None,
                },
            ],
        },
    )
    .await
    .unwrap_err();
    assert!(
        matches!(err, kesh_db::errors::DbError::DateOutsideFiscalYear),
        "écriture hors bornes d'exercice doit être rejetée, got: {err:?}"
    );
}

/// Test 7 : ReportPeriod::resolve cross-tenant returns FiscalYearNotFound
#[sqlx::test(migrations = "./test-schema")]
async fn report_period_resolve_cross_tenant_returns_fy_not_found(pool: MySqlPool) {
    let cid_a = create_company(&pool, "co7a").await;
    let cid_b = create_company(&pool, "co7b").await;
    let uid_b = create_user(&pool, "u7b", cid_b).await;
    let fy_b = create_fy(
        &pool,
        uid_b,
        cid_b,
        "FY",
        NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
        NaiveDate::from_ymd_opt(2026, 12, 31).unwrap(),
    )
    .await;

    // Tenter de résoudre fy_b depuis company_a
    let err = ReportPeriod::resolve(&pool, cid_a, fy_b, None, None)
        .await
        .unwrap_err();
    assert!(
        matches!(err, kesh_report::ReportError::FiscalYearNotFound { .. }),
        "expected FiscalYearNotFound on cross-tenant fy lookup, got: {err:?}"
    );
}

// ============================================================
// Story 14-3c — présentation des fonds propres PAR RÔLE au bilan
// ============================================================

/// AC-A/G : la partition fonds propres / dettes se fait **par rôle**. Un compte de
/// rôle equity va dans `equity`, un compte de rôle non-equity (`Payable`) OU de rôle
/// NULL reste dans `liabilities`. `total_liabilities + total_equity` = ancien total.
#[sqlx::test(migrations = "./test-schema")]
async fn balance_sheet_partitions_equity_by_role(pool: MySqlPool) {
    let cid = create_company(&pool, "co143c1").await;
    let uid = create_user(&pool, "u143c1", cid).await;
    let fy = create_fy(
        &pool,
        uid,
        cid,
        "FY",
        NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
        NaiveDate::from_ymd_opt(2026, 12, 31).unwrap(),
    )
    .await;
    let asset = create_acc(&pool, uid, cid, "1000", "Banque", AccountType::Asset).await;
    let payable = create_acc_with_role(
        &pool,
        uid,
        cid,
        "2000",
        "Fournisseurs",
        AccountType::Liability,
        Some(AccountRole::Payable),
    )
    .await;
    let capital = create_acc_with_role(
        &pool,
        uid,
        cid,
        "2800",
        "Capital",
        AccountType::Liability,
        Some(AccountRole::EquityCapital),
    )
    .await;
    // Compte de passif de rôle NULL → reste dans les dettes (invariant : jamais equity).
    let other_liab = create_acc(&pool, uid, cid, "2100", "Emprunt", AccountType::Liability).await;

    let d = |m, dd| NaiveDate::from_ymd_opt(2026, m, dd).unwrap();
    post_entry(
        &pool,
        uid,
        fy,
        cid,
        d(5, 1),
        Journal::OD,
        asset,
        payable,
        dec!(300),
    )
    .await;
    post_entry(
        &pool,
        uid,
        fy,
        cid,
        d(5, 2),
        Journal::OD,
        asset,
        capital,
        dec!(1000),
    )
    .await;
    post_entry(
        &pool,
        uid,
        fy,
        cid,
        d(5, 3),
        Journal::OD,
        asset,
        other_liab,
        dec!(200),
    )
    .await;

    let period = ReportPeriod::resolve(&pool, cid, fy, None, None)
        .await
        .unwrap();
    let bs = kesh_report::generate_balance_sheet(&pool, cid, &period)
        .await
        .unwrap();

    // equity : seulement le capital (rôle equity).
    assert!(bs.equity.iter().any(|a| a.account_number == "2800"));
    assert!(!bs.equity.iter().any(|a| a.account_number == "2000"));
    assert!(!bs.equity.iter().any(|a| a.account_number == "2100"));
    // liabilities : la dette Payable ET la dette de rôle NULL — PAS le capital.
    assert!(bs.liabilities.iter().any(|a| a.account_number == "2000"));
    assert!(bs.liabilities.iter().any(|a| a.account_number == "2100"));
    assert!(!bs.liabilities.iter().any(|a| a.account_number == "2800"));

    assert_eq!(bs.total_equity, dec!(1000));
    assert_eq!(bs.total_liabilities, dec!(500));
    // Le déplacement est neutre : liabilities + equity = ancien total des passifs.
    assert_eq!(bs.total_liabilities + bs.total_equity, dec!(1500));
    assert_eq!(bs.total_assets, dec!(1500));
    assert!(bs.equation_holds);
}

/// P1-F2 : l'ordre de la section equity suit le **rang de rôle** (CO 959a al. 2),
/// pas le numéro de compte. Plan renuméroté : `EquityOther` sur un numéro INFÉRIEUR
/// à `EquityCapital` → le capital doit quand même sortir en premier.
#[sqlx::test(migrations = "./test-schema")]
async fn balance_sheet_equity_order_by_role_on_renumbered_plan(pool: MySqlPool) {
    let cid = create_company(&pool, "co143c2").await;
    let uid = create_user(&pool, "u143c2", cid).await;
    let fy = create_fy(
        &pool,
        uid,
        cid,
        "FY",
        NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
        NaiveDate::from_ymd_opt(2026, 12, 31).unwrap(),
    )
    .await;
    let asset = create_acc(&pool, uid, cid, "1000", "Banque", AccountType::Asset).await;
    // EquityOther sur 2100 (numéro INFÉRIEUR au capital 2800) — plan non standard.
    let other = create_acc_with_role(
        &pool,
        uid,
        cid,
        "2100",
        "Réserves",
        AccountType::Liability,
        Some(AccountRole::EquityOther),
    )
    .await;
    let capital = create_acc_with_role(
        &pool,
        uid,
        cid,
        "2800",
        "Capital",
        AccountType::Liability,
        Some(AccountRole::EquityCapital),
    )
    .await;
    let d = |dd| NaiveDate::from_ymd_opt(2026, 5, dd).unwrap();
    post_entry(
        &pool,
        uid,
        fy,
        cid,
        d(1),
        Journal::OD,
        asset,
        other,
        dec!(500),
    )
    .await;
    post_entry(
        &pool,
        uid,
        fy,
        cid,
        d(2),
        Journal::OD,
        asset,
        capital,
        dec!(1000),
    )
    .await;

    let period = ReportPeriod::resolve(&pool, cid, fy, None, None)
        .await
        .unwrap();
    let bs = kesh_report::generate_balance_sheet(&pool, cid, &period)
        .await
        .unwrap();

    assert_eq!(bs.equity.len(), 2);
    // Malgré le numéro 2800 > 2100, le CAPITAL (rang 0) sort AVANT les autres FP (rang 1).
    assert_eq!(bs.equity[0].role, Some(AccountRole::EquityCapital));
    assert_eq!(bs.equity[0].account_number, "2800");
    assert_eq!(bs.equity[1].role, Some(AccountRole::EquityOther));
    assert_eq!(bs.equity[1].account_number, "2100");
}

/// D1 : un compte physique de rôle `RetainedEarnings` (report d'ouverture d'un migrant)
/// et la ligne CALCULÉE `retained_earnings` (cumul P&L antérieur) sont **deux grandeurs
/// distinctes**, jamais fusionnées.
#[sqlx::test(migrations = "./test-schema")]
async fn balance_sheet_distinguishes_physical_and_calculated_retained(pool: MySqlPool) {
    let cid = create_company(&pool, "co143c3").await;
    let uid = create_user(&pool, "u143c3", cid).await;
    let fy2025 = create_fy(
        &pool,
        uid,
        cid,
        "FY2025",
        NaiveDate::from_ymd_opt(2025, 1, 1).unwrap(),
        NaiveDate::from_ymd_opt(2025, 12, 31).unwrap(),
    )
    .await;
    let fy2026 = create_fy(
        &pool,
        uid,
        cid,
        "FY2026",
        NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
        NaiveDate::from_ymd_opt(2026, 12, 31).unwrap(),
    )
    .await;
    let asset = create_acc(&pool, uid, cid, "1000", "Banque", AccountType::Asset).await;
    let revenue = create_acc(&pool, uid, cid, "3000", "Ventes", AccountType::Revenue).await;
    let retained_acc = create_acc_with_role(
        &pool,
        uid,
        cid,
        "2970",
        "Bénéfice reporté (compte)",
        AccountType::Liability,
        Some(AccountRole::RetainedEarnings),
    )
    .await;

    // FY2025 : résultat net 5 000 (reste vivant dans Revenue — pas de clôture) → devient
    // le report CALCULÉ pour FY2026.
    post_entry(
        &pool,
        uid,
        fy2025,
        cid,
        NaiveDate::from_ymd_opt(2025, 6, 1).unwrap(),
        Journal::Ventes,
        asset,
        revenue,
        dec!(5000),
    )
    .await;
    // Report d'ouverture PHYSIQUE de 50 000 sur le compte 2970 (migration d'un migrant).
    post_entry(
        &pool,
        uid,
        fy2025,
        cid,
        NaiveDate::from_ymd_opt(2025, 1, 1).unwrap(),
        Journal::OD,
        asset,
        retained_acc,
        dec!(50000),
    )
    .await;

    let period = ReportPeriod::resolve(&pool, cid, fy2026, None, None)
        .await
        .unwrap();
    let bs = kesh_report::generate_balance_sheet(&pool, cid, &period)
        .await
        .unwrap();

    // Ligne PHYSIQUE : compte 2970 itemisé dans equity à 50 000.
    let physical = bs
        .equity
        .iter()
        .find(|a| a.account_number == "2970")
        .expect("compte physique 2970 dans equity");
    assert_eq!(physical.balance, dec!(50000));
    assert_eq!(physical.role, Some(AccountRole::RetainedEarnings));
    // Ligne CALCULÉE : distincte, valeur 5 000 (≠ 50 000 → aucune fusion).
    assert_eq!(bs.retained_earnings, dec!(5000));
    assert_ne!(bs.retained_earnings, physical.balance);
    assert_eq!(bs.total_equity, dec!(50000));
    assert!(bs.equation_holds);
}

/// AC-G : un compte de rôle `CurrentYearResult` à **solde nul** (postings s'annulant)
/// est absent de la section (via `HAVING balance != 0`) — pas de garde spéciale par rôle.
#[sqlx::test(migrations = "./test-schema")]
async fn balance_sheet_current_year_result_zero_balance_absent(pool: MySqlPool) {
    let cid = create_company(&pool, "co143c4").await;
    let uid = create_user(&pool, "u143c4", cid).await;
    let fy = create_fy(
        &pool,
        uid,
        cid,
        "FY",
        NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
        NaiveDate::from_ymd_opt(2026, 12, 31).unwrap(),
    )
    .await;
    let asset = create_acc(&pool, uid, cid, "1000", "Banque", AccountType::Asset).await;
    // Créé postable (rôle None) le temps de poster les écritures qui s'annulent, puis
    // basculé en rôle non-postable CurrentYearResult (scénario legacy — cf. 2979_in_equity).
    let cyr = create_acc(
        &pool,
        uid,
        cid,
        "2979",
        "Résultat exercice",
        AccountType::Liability,
    )
    .await;
    let d = |dd| NaiveDate::from_ymd_opt(2026, 5, dd).unwrap();
    // Deux écritures opposées → solde net 0 sur 2979.
    post_entry(
        &pool,
        uid,
        fy,
        cid,
        d(1),
        Journal::OD,
        asset,
        cyr,
        dec!(300),
    )
    .await;
    post_entry(
        &pool,
        uid,
        fy,
        cid,
        d(2),
        Journal::OD,
        cyr,
        asset,
        dec!(300),
    )
    .await;
    set_account_role(&pool, cyr, AccountRole::CurrentYearResult).await;

    let period = ReportPeriod::resolve(&pool, cid, fy, None, None)
        .await
        .unwrap();
    let bs = kesh_report::generate_balance_sheet(&pool, cid, &period)
        .await
        .unwrap();
    assert!(
        !bs.equity.iter().any(|a| a.account_number == "2979"),
        "2979 à solde nul doit être exclu (HAVING balance != 0)"
    );
    assert!(!bs.liabilities.iter().any(|a| a.account_number == "2979"));
    assert_eq!(bs.total_equity, Decimal::ZERO);
}

/// P1-F1 (intégration) : un reclassement pur entre deux comptes de fonds propres
/// (aucun actif/passif, virtuels nuls, `total_equity` net 0 mais `equity` peuplé) →
/// le bilan n'est PAS vide et l'équation tient.
#[sqlx::test(migrations = "./test-schema")]
async fn balance_sheet_pure_equity_reclass_is_not_empty(pool: MySqlPool) {
    let cid = create_company(&pool, "co143c5").await;
    let uid = create_user(&pool, "u143c5", cid).await;
    let fy = create_fy(
        &pool,
        uid,
        cid,
        "FY",
        NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
        NaiveDate::from_ymd_opt(2026, 12, 31).unwrap(),
    )
    .await;
    let capital = create_acc_with_role(
        &pool,
        uid,
        cid,
        "2800",
        "Capital",
        AccountType::Liability,
        Some(AccountRole::EquityCapital),
    )
    .await;
    let reserves = create_acc_with_role(
        &pool,
        uid,
        cid,
        "2900",
        "Réserves",
        AccountType::Liability,
        Some(AccountRole::EquityOther),
    )
    .await;
    // Reclassement : débit Réserves / crédit Capital 1 000 (aucun actif/passif touché).
    // Capital (crédit−débit) = +1000 ; Réserves = −1000 ; somme nette 0.
    post_entry(
        &pool,
        uid,
        fy,
        cid,
        NaiveDate::from_ymd_opt(2026, 6, 1).unwrap(),
        Journal::OD,
        reserves,
        capital,
        dec!(1000),
    )
    .await;

    let period = ReportPeriod::resolve(&pool, cid, fy, None, None)
        .await
        .unwrap();
    let bs = kesh_report::generate_balance_sheet(&pool, cid, &period)
        .await
        .unwrap();

    assert_eq!(bs.equity.len(), 2, "les 2 comptes non nuls doivent figurer");
    assert_eq!(bs.total_equity, Decimal::ZERO);
    assert!(bs.assets.is_empty());
    assert!(bs.liabilities.is_empty());
    assert!(
        !bs.is_empty(),
        "equity peuplé ⇒ rapport NON vide (garde P1-F1) — ne pas masquer la section"
    );
    assert!(bs.equation_holds);
}

/// L3 (non-régression, validate P1-F7) : `accepts_account_type` autorise **techniquement**
/// un rôle equity sur un compte de type `Asset` (les 4 rôles equity acceptent
/// `Asset|Liability`). La partition ne scanne que la section `Liability`, donc un tel
/// compte reste dans **Actifs** et n'apparaît **jamais** dans « Capitaux propres » —
/// arithmétiquement sain (compté une fois dans `total_assets`), problème de présentation
/// seul, cas rare. Ce test **documente et verrouille** ce comportement : un futur refactor
/// de la partition (ex. scan aussi de `assets`) le ferait échouer, signalant la régression.
#[sqlx::test(migrations = "./test-schema")]
async fn balance_sheet_equity_role_on_asset_stays_in_assets(pool: MySqlPool) {
    let cid = create_company(&pool, "co143c6").await;
    let uid = create_user(&pool, "u143c6", cid).await;
    let fy = create_fy(
        &pool,
        uid,
        cid,
        "FY",
        NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
        NaiveDate::from_ymd_opt(2026, 12, 31).unwrap(),
    )
    .await;
    // Compte mal typé : type Asset MAIS rôle EquityCapital (autorisé par
    // accepts_account_type qui accepte Asset|Liability pour les rôles equity).
    let asset_equity = create_acc_with_role(
        &pool,
        uid,
        cid,
        "1090",
        "Compte courant associé (équité mal typée)",
        AccountType::Asset,
        Some(AccountRole::EquityCapital),
    )
    .await;
    let liab = create_acc_with_role(
        &pool,
        uid,
        cid,
        "2000",
        "Fournisseurs",
        AccountType::Liability,
        Some(AccountRole::Payable),
    )
    .await;
    // Débit du compte Asset mal typé / crédit d'une dette → Asset (débit−crédit) = +1000.
    post_entry(
        &pool,
        uid,
        fy,
        cid,
        NaiveDate::from_ymd_opt(2026, 5, 1).unwrap(),
        Journal::OD,
        asset_equity,
        liab,
        dec!(1000),
    )
    .await;

    let period = ReportPeriod::resolve(&pool, cid, fy, None, None)
        .await
        .unwrap();
    let bs = kesh_report::generate_balance_sheet(&pool, cid, &period)
        .await
        .unwrap();

    // Le compte reste dans Actifs (partition ne scanne que les Passifs).
    assert!(
        bs.assets.iter().any(|a| a.account_number == "1090"),
        "un rôle equity sur un compte Asset reste dans Actifs (L3)"
    );
    // Il n'est JAMAIS dans la section Capitaux propres.
    assert!(
        !bs.equity.iter().any(|a| a.account_number == "1090"),
        "un compte Asset ne doit jamais apparaître dans equity (L3)"
    );
    // Compté une fois dans total_assets, pas de double-comptage → équation tient.
    assert_eq!(bs.total_assets, dec!(1000));
    assert_eq!(bs.total_equity, Decimal::ZERO);
    assert_eq!(bs.total_liabilities, dec!(1000));
    assert!(bs.equation_holds);
}
