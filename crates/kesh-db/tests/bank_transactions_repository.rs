//! Tests d'intégration pour `repositories::bank_transactions::find_in_dedup_window`
//! (Story 8-3 T3.2).
//!
//! Tests : filtre fenêtre date, scoping multi-tenant `company_id`,
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
        counterparty_email: None,
        provider_type: None,
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

/// Story 39-11a2 — `existing_transactions_by_ids` rend une ligne par
/// transaction (un identifiant au débit et au crédit revient deux fois), avec
/// montant et date, bornée à la société et au compte ; un identifiant absent
/// ne rend rien.
#[sqlx::test(migrations = "./test-schema")]
async fn existing_transactions_by_ids_returns_each_transaction_with_its_amount(pool: MySqlPool) {
    let company = create_test_company(&pool, "Société A").await;
    let user = create_test_user(&pool, "user-a", company).await;
    let account = create_test_bank_account(&pool, company, "CH9300762011623852957").await;
    let other_account = create_test_bank_account(&pool, company, "CH4431999123000889012").await;
    let d1 = NaiveDate::from_ymd_opt(2026, 5, 10).unwrap();
    let d2 = NaiveDate::from_ymd_opt(2026, 5, 17).unwrap();
    let with_id = |date, amount, id: &str| NewBankTransaction {
        transaction_id: Some(id.into()),
        ..make_tx(company, account, date, amount, "r")
    };
    seed_import_with_txs(
        &pool,
        company,
        account,
        user,
        HASH_A,
        vec![
            with_id(d1, dec!(-32.10), "CARD-1"),
            with_id(d2, dec!(32.10), "CARD-1"),
            with_id(d2, dec!(-5.00), "CARD-2"),
        ],
    )
    .await;
    seed_import_with_txs(
        &pool,
        company,
        other_account,
        user,
        HASH_B,
        vec![NewBankTransaction {
            transaction_id: Some("CARD-1".into()),
            ..make_tx(company, other_account, d1, dec!(-1.00), "r")
        }],
    )
    .await;

    let mut conn = pool.acquire().await.unwrap();
    let ids = vec!["CARD-1".to_string(), "CARD-9".to_string()];
    let mut got =
        bank_transactions::existing_transactions_by_ids(&mut conn, company, account, &ids)
            .await
            .unwrap();
    got.sort_by_key(|t| t.booking_date);
    assert_eq!(
        got,
        vec![
            bank_transactions::ExistingTransaction {
                transaction_id: "CARD-1".into(),
                amount: dec!(-32.10),
                booking_date: d1,
            },
            bank_transactions::ExistingTransaction {
                transaction_id: "CARD-1".into(),
                amount: dec!(32.10),
                booking_date: d2,
            },
        ]
    );
    let none =
        bank_transactions::existing_transactions_by_ids(&mut conn, company + 1, account, &ids)
            .await
            .unwrap();
    assert!(none.is_empty(), "autre société : rien");
}

/// Story 39-13 — `existing_transaction_ids`, écrite sur
/// `existing_transactions_by_ids` : un identifiant au débit et au crédit est
/// rendu une fois, un identifiant absent ne l'est pas, le même identifiant sur
/// un autre compte de la société ou dans une autre société (réelle) est exclu.
/// Test de caractérisation (l'ancienne requête rendait le même ensemble).
///
/// Ordre de création voulu : société A (1), son autre compte (1) puis le
/// compte visé (2), société B (2) et son compte (3). Aucune ligne ne porte
/// donc (société = id du compte visé, compte = id de la société visée) : une
/// délégation qui permuterait les deux arguments rendrait l'ensemble vide
/// (assertion de montage ci-dessous).
#[sqlx::test(migrations = "./test-schema")]
async fn existing_transaction_ids_returns_each_id_once_within_company_and_account(pool: MySqlPool) {
    let company = create_test_company(&pool, "Société A").await;
    let user = create_test_user(&pool, "user-a", company).await;
    let other_account = create_test_bank_account(&pool, company, "CH4431999123000889012").await;
    let account = create_test_bank_account(&pool, company, "CH9300762011623852957").await;
    let other_company = create_test_company(&pool, "Société B").await;
    let other_user = create_test_user(&pool, "user-b", other_company).await;
    let foreign_account =
        create_test_bank_account(&pool, other_company, "CH9300762011623852957").await;
    assert_ne!(
        company, account,
        "montage : société et compte visés distincts"
    );

    let d1 = NaiveDate::from_ymd_opt(2026, 5, 10).unwrap();
    let d2 = NaiveDate::from_ymd_opt(2026, 5, 17).unwrap();
    let with_id = |company, account, date, amount, id: &str| NewBankTransaction {
        transaction_id: Some(id.into()),
        ..make_tx(company, account, date, amount, "r")
    };
    seed_import_with_txs(
        &pool,
        company,
        account,
        user,
        HASH_A,
        vec![
            with_id(company, account, d1, dec!(-32.10), "CARD-1"),
            with_id(company, account, d2, dec!(32.10), "CARD-1"),
            with_id(company, account, d2, dec!(-5.00), "CARD-2"),
        ],
    )
    .await;
    seed_import_with_txs(
        &pool,
        company,
        other_account,
        user,
        HASH_B,
        vec![with_id(company, other_account, d1, dec!(-1.00), "CARD-3")],
    )
    .await;
    seed_import_with_txs(
        &pool,
        other_company,
        foreign_account,
        other_user,
        HASH_A,
        vec![with_id(
            other_company,
            foreign_account,
            d1,
            dec!(-2.00),
            "CARD-4",
        )],
    )
    .await;

    let mut conn = pool.acquire().await.unwrap();
    let ids: Vec<String> = ["CARD-1", "CARD-3", "CARD-4", "CARD-9"]
        .map(String::from)
        .to_vec();
    // Montage, lu par une requête directe (non par la fonction testée) :
    // aucune ligne ne porte (société = id du compte visé, compte = id de la
    // société visée), si bien qu'une permutation des deux arguments ne rendrait rien.
    let swapped: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM bank_transactions WHERE company_id = ? AND bank_account_id = ?",
    )
    .bind(account)
    .bind(company)
    .fetch_one(&mut *conn)
    .await
    .unwrap();
    assert_eq!(
        swapped, 0,
        "montage : (compte, société) permutés ne désignent aucune ligne"
    );

    let got = bank_transactions::existing_transaction_ids(&mut conn, company, account, &ids)
        .await
        .unwrap();
    assert_eq!(
        got,
        std::collections::HashSet::from(["CARD-1".to_string()]),
        "CARD-1 une fois ; CARD-3 (autre compte), CARD-4 (autre société), CARD-9 (absent) exclus"
    );
    let other =
        bank_transactions::existing_transaction_ids(&mut conn, company, other_account, &ids)
            .await
            .unwrap();
    assert_eq!(
        other,
        std::collections::HashSet::from(["CARD-3".to_string()])
    );
    let foreign = bank_transactions::existing_transaction_ids(
        &mut conn,
        other_company,
        foreign_account,
        &ids,
    )
    .await
    .unwrap();
    assert_eq!(
        foreign,
        std::collections::HashSet::from(["CARD-4".to_string()])
    );
}
