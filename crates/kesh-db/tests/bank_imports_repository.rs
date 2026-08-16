//! Tests d'intégration pour `repositories::bank_imports` +
//! `repositories::bank_transactions` (Story 8-1b T5.6).
//!
//! 9 tests : 7 sur `bank_imports` (création atomique, rollback, multi-tenant
//! scoping, dedup hash, multi-tenant safety, bulk perf) + 2 IDOR tests sur
//! `bank_accounts::find_by_id_for_company` (T6.3 / Pass 1 H5).
//!
//! Pattern `#[sqlx::test(migrations = "./test-schema")]` — DB éphémère
//! avec migrations auto-appliquées.

use chrono::NaiveDate;
use kesh_db::entities::{
    BankImportSourceFormat, NewBankAccount, NewBankImport, NewBankTransaction, NewUser, Role,
};
use kesh_db::repositories::{bank_accounts, bank_imports, users};
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

fn make_new_import(
    company_id: i64,
    bank_account_id: i64,
    user_id: i64,
    file_hash: &str,
) -> NewBankImport {
    NewBankImport {
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
        transaction_count: 1,
        imported_by_user_id: user_id,
    }
}

fn make_new_tx(
    company_id: i64,
    bank_account_id: i64,
    amount: Decimal,
    reference: &str,
) -> NewBankTransaction {
    NewBankTransaction {
        company_id,
        bank_account_id,
        booking_date: NaiveDate::from_ymd_opt(2026, 5, 15).unwrap(),
        value_date: Some(NaiveDate::from_ymd_opt(2026, 5, 15).unwrap()),
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

const HASH_A: &str = "0123456789012345678901234567890123456789012345678901234567890abc";
const HASH_B: &str = "fedcba9876543210fedcba9876543210fedcba9876543210fedcba98765432ab";

#[sqlx::test(migrations = "./test-schema")]
async fn create_with_transactions_atomic_success(pool: MySqlPool) {
    let company_id = create_test_company(&pool, "Acme").await;
    let user_id = create_test_user(&pool, "alice", company_id).await;
    let bank_id = create_test_bank_account(&pool, company_id, "CH4431999123000889012").await;

    let mut tx = pool.begin().await.unwrap();
    let new_import = make_new_import(company_id, bank_id, user_id, HASH_A);
    let txs = vec![
        make_new_tx(company_id, bank_id, dec!(100.00), "REF-1"),
        make_new_tx(company_id, bank_id, dec!(-50.00), "REF-2"),
    ];
    let (header, inserted_txs) = bank_imports::create_with_transactions(&mut tx, new_import, txs)
        .await
        .expect("create OK");
    tx.commit().await.unwrap();

    assert!(header.id > 0);
    assert_eq!(header.company_id, company_id);
    assert_eq!(
        header.transaction_count, 1,
        "transaction_count vient de NewBankImport"
    );
    assert_eq!(inserted_txs.len(), 2);
    assert_eq!(inserted_txs[0].amount, dec!(100.00));
    assert_eq!(inserted_txs[1].amount, dec!(-50.00));
    assert_eq!(inserted_txs[0].import_id, header.id);
    assert_eq!(inserted_txs[0].status.as_str(), "pending");
}

#[sqlx::test(migrations = "./test-schema")]
async fn create_with_transactions_rolls_back_on_constraint_violation(pool: MySqlPool) {
    // AC #17 atomicité : simuler une violation FK sur la 2e transaction
    // (bank_account_id inexistant) → tx rollback, aucune ligne en DB.
    let company_id = create_test_company(&pool, "Acme").await;
    let user_id = create_test_user(&pool, "alice", company_id).await;
    let bank_id = create_test_bank_account(&pool, company_id, "CH4431999123000889012").await;

    let mut tx = pool.begin().await.unwrap();
    let new_import = make_new_import(company_id, bank_id, user_id, HASH_A);
    let txs = vec![
        make_new_tx(company_id, bank_id, dec!(100.00), "REF-1"),
        // bank_account_id 999999 inexistant → FK violation
        make_new_tx(company_id, 999_999, dec!(50.00), "REF-2"),
    ];
    let result = bank_imports::create_with_transactions(&mut tx, new_import, txs).await;
    assert!(result.is_err(), "FK violation doit échouer");
    // Pas de commit, tx drop = rollback automatique
    drop(tx);

    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM bank_imports WHERE company_id = ?")
        .bind(company_id)
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(count, 0, "rollback : aucune entête en DB");
    let tx_count: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM bank_transactions WHERE company_id = ?")
            .bind(company_id)
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(tx_count, 0, "rollback : aucune transaction en DB");
}

#[sqlx::test(migrations = "./test-schema")]
async fn find_by_company_id_only_returns_own_imports(pool: MySqlPool) {
    // AC #11 multi-tenant scoping : 2 companies, find_by_company_id(A)
    // ne retourne pas les imports de B.
    let company_a = create_test_company(&pool, "CompanyA").await;
    let company_b = create_test_company(&pool, "CompanyB").await;
    let user_a = create_test_user(&pool, "alice", company_a).await;
    let user_b = create_test_user(&pool, "bob", company_b).await;
    let bank_a = create_test_bank_account(&pool, company_a, "CH4431999123000889012").await;
    let bank_b = create_test_bank_account(&pool, company_b, "CH9300762011623852957").await;

    for (company_id, bank_id, user_id, hash) in [
        (company_a, bank_a, user_a, HASH_A),
        (company_b, bank_b, user_b, HASH_B),
    ] {
        let mut tx = pool.begin().await.unwrap();
        bank_imports::create_with_transactions(
            &mut tx,
            make_new_import(company_id, bank_id, user_id, hash),
            vec![make_new_tx(company_id, bank_id, dec!(100.00), "R")],
        )
        .await
        .unwrap();
        tx.commit().await.unwrap();
    }

    let imports_a = bank_imports::find_by_company_id(&pool, company_a, None, 100, 0)
        .await
        .unwrap();
    assert_eq!(imports_a.len(), 1);
    assert_eq!(imports_a[0].company_id, company_a);
    assert_eq!(imports_a[0].file_hash, HASH_A);
}

#[sqlx::test(migrations = "./test-schema")]
async fn find_by_company_and_hash_finds_existing(pool: MySqlPool) {
    let company_id = create_test_company(&pool, "Acme").await;
    let user_id = create_test_user(&pool, "alice", company_id).await;
    let bank_id = create_test_bank_account(&pool, company_id, "CH4431999123000889012").await;

    let mut tx = pool.begin().await.unwrap();
    bank_imports::create_with_transactions(
        &mut tx,
        make_new_import(company_id, bank_id, user_id, HASH_A),
        vec![make_new_tx(company_id, bank_id, dec!(100.00), "R")],
    )
    .await
    .unwrap();
    tx.commit().await.unwrap();

    let found = bank_imports::find_by_company_and_hash(&pool, company_id, HASH_A)
        .await
        .unwrap();
    assert!(found.is_some());
    assert_eq!(found.unwrap().file_hash, HASH_A);

    let not_found = bank_imports::find_by_company_and_hash(&pool, company_id, HASH_B)
        .await
        .unwrap();
    assert!(not_found.is_none());
}

#[sqlx::test(migrations = "./test-schema")]
async fn duplicate_company_hash_now_allowed_post_relax(pool: MySqlPool) {
    // Story 8-3 — la migration `20260507000001_bank_imports_relax_hash_unique.sql`
    // a relâché le UNIQUE en INDEX simple. Deux INSERT successifs avec
    // le même `(company_id, file_hash)` doivent désormais réussir
    // (le check applicatif transactionnel via `find_by_company_and_hash`
    // est appliqué dans le handler `bank_imports::create` selon le flag
    // multipart `confirmDuplicateFile`).
    let company_id = create_test_company(&pool, "Acme").await;
    let user_id = create_test_user(&pool, "alice", company_id).await;
    let bank_id = create_test_bank_account(&pool, company_id, "CH4431999123000889012").await;

    let mut tx = pool.begin().await.unwrap();
    bank_imports::create_with_transactions(
        &mut tx,
        make_new_import(company_id, bank_id, user_id, HASH_A),
        vec![make_new_tx(company_id, bank_id, dec!(100.00), "R")],
    )
    .await
    .unwrap();
    tx.commit().await.unwrap();

    let mut tx2 = pool.begin().await.unwrap();
    let result = bank_imports::create_with_transactions(
        &mut tx2,
        make_new_import(company_id, bank_id, user_id, HASH_A),
        vec![make_new_tx(company_id, bank_id, dec!(200.00), "R")],
    )
    .await;
    assert!(
        result.is_ok(),
        "post-relax UNIQUE, second INSERT même (company, hash) doit réussir : {result:?}"
    );
    tx2.commit().await.unwrap();
}

#[sqlx::test(migrations = "./test-schema")]
async fn unique_company_hash_allows_same_hash_across_companies(pool: MySqlPool) {
    // AC #16 multi-tenant safety : même hash sur companies différentes OK.
    let company_a = create_test_company(&pool, "CompanyA").await;
    let company_b = create_test_company(&pool, "CompanyB").await;
    let user_a = create_test_user(&pool, "alice", company_a).await;
    let user_b = create_test_user(&pool, "bob", company_b).await;
    let bank_a = create_test_bank_account(&pool, company_a, "CH4431999123000889012").await;
    let bank_b = create_test_bank_account(&pool, company_b, "CH9300762011623852957").await;

    let mut tx_a = pool.begin().await.unwrap();
    bank_imports::create_with_transactions(
        &mut tx_a,
        make_new_import(company_a, bank_a, user_a, HASH_A),
        vec![make_new_tx(company_a, bank_a, dec!(100.00), "R")],
    )
    .await
    .unwrap();
    tx_a.commit().await.unwrap();

    let mut tx_b = pool.begin().await.unwrap();
    let result = bank_imports::create_with_transactions(
        &mut tx_b,
        make_new_import(company_b, bank_b, user_b, HASH_A),
        vec![make_new_tx(company_b, bank_b, dec!(100.00), "R")],
    )
    .await;
    assert!(result.is_ok(), "même hash, company différente → OK");
    tx_b.commit().await.unwrap();
}

#[sqlx::test(migrations = "./test-schema")]
async fn bulk_insert_handles_500_transactions(pool: MySqlPool) {
    // AC #21 perf smoke : 500 transactions doivent passer en bulk INSERT
    // (chunk de 1000 max — un seul chunk ici).
    let company_id = create_test_company(&pool, "Acme").await;
    let user_id = create_test_user(&pool, "alice", company_id).await;
    let bank_id = create_test_bank_account(&pool, company_id, "CH4431999123000889012").await;

    let txs: Vec<_> = (0..500)
        .map(|i| make_new_tx(company_id, bank_id, dec!(10.00), &format!("REF-{i}")))
        .collect();

    let start = std::time::Instant::now();
    let mut tx = pool.begin().await.unwrap();
    let mut new_import = make_new_import(company_id, bank_id, user_id, HASH_A);
    new_import.transaction_count = 500;
    let (header, inserted) = bank_imports::create_with_transactions(&mut tx, new_import, txs)
        .await
        .unwrap();
    tx.commit().await.unwrap();
    let elapsed = start.elapsed();

    assert_eq!(inserted.len(), 500);
    assert_eq!(header.transaction_count, 500);
    // Smoke uniquement (pas de seuil CI strict — AC #21 dit "< 2s machine
    // de dev nominale"). On affiche le temps pour observation.
    println!("bulk_insert 500 tx en {elapsed:?}");
}

// === Tests #8 et #9 : IDOR sur bank_accounts::find_by_id_for_company ===
//
// Pass 2 M4 validate Pass 2 : ces tests vivent dans le fichier de test
// `bank_accounts` plutôt qu'ici (cohésion fonction-tests). Mais comme
// `tests/bank_accounts_repository.rs` existe déjà avec un autre style,
// on les place ici en module dédié pour conserver l'ensemble T5.6 / AC #11
// dans un seul run `#[sqlx::test]`. Le helper testé est dans
// `repositories::bank_accounts::find_by_id_for_company`.

#[sqlx::test(migrations = "./test-schema")]
async fn find_by_id_for_company_rejects_wrong_company(pool: MySqlPool) {
    // T6.3 IDOR : company_B ne peut pas lire un bank_account de company_A.
    let company_a = create_test_company(&pool, "CompanyA").await;
    let company_b = create_test_company(&pool, "CompanyB").await;
    let bank_a = create_test_bank_account(&pool, company_a, "CH4431999123000889012").await;

    let result = bank_accounts::find_by_id_for_company(&pool, company_b, bank_a)
        .await
        .unwrap();
    assert!(result.is_none(), "IDOR : pas de leak cross-tenant");
}

#[sqlx::test(migrations = "./test-schema")]
async fn find_by_id_for_company_returns_account_when_owned(pool: MySqlPool) {
    // T6.3 happy path : la company qui possède le compte le récupère.
    let company_id = create_test_company(&pool, "Acme").await;
    let bank_id = create_test_bank_account(&pool, company_id, "CH4431999123000889012").await;

    let found = bank_accounts::find_by_id_for_company(&pool, company_id, bank_id)
        .await
        .unwrap();
    assert!(found.is_some());
    let acc = found.unwrap();
    assert_eq!(acc.id, bank_id);
    assert_eq!(acc.iban, "CH4431999123000889012");
}
