//! Tests d'intégration pour `repositories::bank_transactions::find_in_dedup_window`
//! (Story 8-3 T3.2).
//!
//! 4 tests : filtre fenêtre date, scoping multi-tenant `company_id`,
//! scoping `bank_account_id`, retour vide si aucune transaction dans
//! la fenêtre. Les tests utilisent `bank_imports::create_with_transactions`
//! pour seeder des transactions persistées (l'INSERT direct nécessiterait
//! de bypasser le FK constraint sur `import_id`).
//!
//! Pattern `#[sqlx::test(migrations = "./test-schema")]` — DB éphémère.

use chrono::NaiveDate;
use kesh_db::entities::{
    BankImportSourceFormat, NewBankAccount, NewBankImport, NewBankTransaction, NewUser, Role,
};
use kesh_db::repositories::{bank_accounts, bank_imports, bank_transactions, users};
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use sqlx::MySqlPool;

async fn create_test_company(pool: &MySqlPool, name: &str) -> i64 {
    let result = sqlx::query(
        "INSERT INTO companies (name, address, org_type, accounting_language, instance_language) \
         VALUES (?, ?, ?, ?, ?)",
    )
    .bind(name)
    .bind("Rue Test 1")
    .bind("Independant")
    .bind("FR")
    .bind("FR")
    .execute(pool)
    .await
    .expect("company insert");
    result.last_insert_id() as i64
}

async fn create_test_user(pool: &MySqlPool, username: &str, company_id: i64) -> i64 {
    users::create(
        pool,
        NewUser {
            username: username.into(),
            password_hash:
                "$argon2id$v=19$m=19456,t=2,p=1$dGVzdHNhbHQ$dGVzdGhhc2h0ZXN0aGFzaHRlc3RoYXNo"
                    .into(),
            role: Role::Comptable,
            active: true,
            company_id,
            email: None,
        },
    )
    .await
    .expect("user create")
    .id
}

async fn create_test_bank_account(pool: &MySqlPool, company_id: i64, iban: &str) -> i64 {
    bank_accounts::create(
        pool,
        NewBankAccount {
            company_id,
            bank_name: "UBS".into(),
            iban: iban.into(),
            qr_iban: None,
            is_primary: true,
        },
    )
    .await
    .expect("bank_account create")
    .id
}

fn make_tx(
    company_id: i64,
    bank_account_id: i64,
    booking_date: NaiveDate,
    amount: Decimal,
    reference: &str,
) -> NewBankTransaction {
    NewBankTransaction {
        company_id,
        bank_account_id,
        booking_date,
        value_date: Some(booking_date),
        amount,
        currency: "CHF".into(),
        reference: Some(reference.into()),
        details: "Test tx".into(),
        end_to_end_id: None,
        transaction_id: None,
        counterparty_iban: None,
        counterparty_name: None,
    }
}

async fn seed_import_with_txs(
    pool: &MySqlPool,
    company_id: i64,
    bank_account_id: i64,
    user_id: i64,
    file_hash: &str,
    txs: Vec<NewBankTransaction>,
) {
    let tx_count = txs.len() as i32;
    let new_import = NewBankImport {
        company_id,
        bank_account_id,
        filename: "stmt.xml".into(),
        file_hash: file_hash.into(),
        source_format: BankImportSourceFormat::Camt053V04,
        statement_id: Some("STMT-001".into()),
        period_from: NaiveDate::from_ymd_opt(2026, 5, 1).unwrap(),
        period_to: NaiveDate::from_ymd_opt(2026, 5, 31).unwrap(),
        opening_balance: Some(dec!(1000.00)),
        closing_balance: Some(dec!(1100.00)),
        transaction_count: tx_count,
        imported_by_user_id: user_id,
    };
    let mut tx = pool.begin().await.unwrap();
    bank_imports::create_with_transactions(&mut tx, new_import, txs)
        .await
        .expect("seed import");
    tx.commit().await.unwrap();
}

const HASH_A: &str = "0123456789012345678901234567890123456789012345678901234567890abc";
const HASH_B: &str = "fedcba9876543210fedcba9876543210fedcba9876543210fedcba98765432ab";

#[sqlx::test(migrations = "./test-schema")]
async fn find_in_dedup_window_returns_only_within_period(pool: MySqlPool) {
    // T3.2#1 — 3 transactions : 1 dans la fenêtre, 1 avant, 1 après → 1 retournée.
    let company_id = create_test_company(&pool, "Acme").await;
    let user_id = create_test_user(&pool, "alice", company_id).await;
    let bank_id = create_test_bank_account(&pool, company_id, "CH4431999123000889012").await;

    seed_import_with_txs(
        &pool,
        company_id,
        bank_id,
        user_id,
        HASH_A,
        vec![
            make_tx(
                company_id,
                bank_id,
                NaiveDate::from_ymd_opt(2026, 4, 30).unwrap(),
                dec!(50.00),
                "BEFORE",
            ),
            make_tx(
                company_id,
                bank_id,
                NaiveDate::from_ymd_opt(2026, 5, 15).unwrap(),
                dec!(100.00),
                "INSIDE",
            ),
            make_tx(
                company_id,
                bank_id,
                NaiveDate::from_ymd_opt(2026, 6, 1).unwrap(),
                dec!(200.00),
                "AFTER",
            ),
        ],
    )
    .await;

    let result = bank_transactions::find_in_dedup_window(
        &pool,
        company_id,
        bank_id,
        NaiveDate::from_ymd_opt(2026, 5, 1).unwrap(),
        NaiveDate::from_ymd_opt(2026, 5, 31).unwrap(),
    )
    .await
    .unwrap();

    assert_eq!(result.len(), 1);
    assert_eq!(result[0].reference.as_deref(), Some("INSIDE"));
}

#[sqlx::test(migrations = "./test-schema")]
async fn find_in_dedup_window_scopes_by_company(pool: MySqlPool) {
    // T3.2#2 / AC #25 (KF-002 Pattern 1) — multi-tenant : transactions
    // de company_B non retournées pour company_A.
    let company_a = create_test_company(&pool, "CompanyA").await;
    let company_b = create_test_company(&pool, "CompanyB").await;
    let user_a = create_test_user(&pool, "alice", company_a).await;
    let user_b = create_test_user(&pool, "bob", company_b).await;
    let bank_a = create_test_bank_account(&pool, company_a, "CH4431999123000889012").await;
    let bank_b = create_test_bank_account(&pool, company_b, "CH9300762011623852957").await;

    seed_import_with_txs(
        &pool,
        company_a,
        bank_a,
        user_a,
        HASH_A,
        vec![make_tx(
            company_a,
            bank_a,
            NaiveDate::from_ymd_opt(2026, 5, 15).unwrap(),
            dec!(100.00),
            "TX-A",
        )],
    )
    .await;
    seed_import_with_txs(
        &pool,
        company_b,
        bank_b,
        user_b,
        HASH_B,
        vec![make_tx(
            company_b,
            bank_b,
            NaiveDate::from_ymd_opt(2026, 5, 15).unwrap(),
            dec!(100.00),
            "TX-B",
        )],
    )
    .await;

    // company_A asks for txs with bank_b — devrait retourner 0 même si bank_b
    // a une transaction (cross-tenant : aucun leak). Note : bank_b
    // n'appartient pas à company_a, donc cette query simule un cas
    // d'IDOR — le filtre `company_id` doit la bloquer.
    let cross_tenant = bank_transactions::find_in_dedup_window(
        &pool,
        company_a,
        bank_b,
        NaiveDate::from_ymd_opt(2026, 5, 1).unwrap(),
        NaiveDate::from_ymd_opt(2026, 5, 31).unwrap(),
    )
    .await
    .unwrap();
    assert!(cross_tenant.is_empty(), "leak cross-tenant détecté");

    // Normal access company_a/bank_a → 1.
    let same_tenant = bank_transactions::find_in_dedup_window(
        &pool,
        company_a,
        bank_a,
        NaiveDate::from_ymd_opt(2026, 5, 1).unwrap(),
        NaiveDate::from_ymd_opt(2026, 5, 31).unwrap(),
    )
    .await
    .unwrap();
    assert_eq!(same_tenant.len(), 1);
    assert_eq!(same_tenant[0].reference.as_deref(), Some("TX-A"));
}

#[sqlx::test(migrations = "./test-schema")]
async fn find_in_dedup_window_scopes_by_bank_account(pool: MySqlPool) {
    // T3.2#3 — 2 comptes même company : seul le compte demandé retourne.
    let company_id = create_test_company(&pool, "Acme").await;
    let user_id = create_test_user(&pool, "alice", company_id).await;
    let bank_1 = create_test_bank_account(&pool, company_id, "CH4431999123000889012").await;
    let bank_2 = create_test_bank_account(&pool, company_id, "CH9300762011623852957").await;

    seed_import_with_txs(
        &pool,
        company_id,
        bank_1,
        user_id,
        HASH_A,
        vec![make_tx(
            company_id,
            bank_1,
            NaiveDate::from_ymd_opt(2026, 5, 15).unwrap(),
            dec!(100.00),
            "TX-BANK-1",
        )],
    )
    .await;
    seed_import_with_txs(
        &pool,
        company_id,
        bank_2,
        user_id,
        HASH_B,
        vec![make_tx(
            company_id,
            bank_2,
            NaiveDate::from_ymd_opt(2026, 5, 15).unwrap(),
            dec!(100.00),
            "TX-BANK-2",
        )],
    )
    .await;

    let result = bank_transactions::find_in_dedup_window(
        &pool,
        company_id,
        bank_1,
        NaiveDate::from_ymd_opt(2026, 5, 1).unwrap(),
        NaiveDate::from_ymd_opt(2026, 5, 31).unwrap(),
    )
    .await
    .unwrap();
    assert_eq!(result.len(), 1);
    assert_eq!(result[0].reference.as_deref(), Some("TX-BANK-1"));
}

#[sqlx::test(migrations = "./test-schema")]
async fn find_in_dedup_window_returns_empty_when_no_match(pool: MySqlPool) {
    // T3.2#4 — happy path empty result.
    let company_id = create_test_company(&pool, "Acme").await;
    let _user_id = create_test_user(&pool, "alice", company_id).await;
    let bank_id = create_test_bank_account(&pool, company_id, "CH4431999123000889012").await;

    let result = bank_transactions::find_in_dedup_window(
        &pool,
        company_id,
        bank_id,
        NaiveDate::from_ymd_opt(2026, 5, 1).unwrap(),
        NaiveDate::from_ymd_opt(2026, 5, 31).unwrap(),
    )
    .await
    .unwrap();
    assert!(result.is_empty());
}
