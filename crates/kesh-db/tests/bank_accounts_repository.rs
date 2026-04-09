//! Tests d'intégration pour `repositories::bank_accounts`.

use kesh_db::entities::{Language, NewBankAccount, NewCompany, OrgType};
use kesh_db::repositories::{bank_accounts, companies};
use sqlx::MySqlPool;

async fn create_test_company(pool: &MySqlPool) -> i64 {
    companies::create(
        pool,
        NewCompany {
            name: "Test SA".into(),
            address: "Rue Test 1".into(),
            ide_number: None,
            org_type: OrgType::Pme,
            accounting_language: Language::Fr,
            instance_language: Language::Fr,
        },
    )
    .await
    .unwrap()
    .id
}

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn create_and_find_primary(pool: MySqlPool) {
    let company_id = create_test_company(&pool).await;

    let created = bank_accounts::create(
        &pool,
        NewBankAccount {
            company_id,
            bank_name: "UBS".into(),
            iban: "CH9300762011623852957".into(),
            qr_iban: None,
            is_primary: true,
        },
    )
    .await
    .unwrap();

    assert!(created.id > 0);
    assert_eq!(created.bank_name, "UBS");
    assert!(created.is_primary);

    let found = bank_accounts::find_primary(&pool, company_id)
        .await
        .unwrap();
    assert!(found.is_some());
    assert_eq!(found.unwrap().iban, "CH9300762011623852957");
}

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn find_primary_returns_none_when_empty(pool: MySqlPool) {
    let company_id = create_test_company(&pool).await;
    let found = bank_accounts::find_primary(&pool, company_id).await.unwrap();
    assert!(found.is_none());
}

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn list_by_company(pool: MySqlPool) {
    let company_id = create_test_company(&pool).await;

    bank_accounts::create(
        &pool,
        NewBankAccount {
            company_id,
            bank_name: "UBS".into(),
            iban: "CH9300762011623852957".into(),
            qr_iban: None,
            is_primary: true,
        },
    )
    .await
    .unwrap();

    let list = bank_accounts::list_by_company(&pool, company_id)
        .await
        .unwrap();
    assert_eq!(list.len(), 1);
    assert_eq!(list[0].bank_name, "UBS");
}

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn upsert_primary_creates_then_updates(pool: MySqlPool) {
    let company_id = create_test_company(&pool).await;

    // First call creates
    let created = bank_accounts::upsert_primary(
        &pool,
        NewBankAccount {
            company_id,
            bank_name: "UBS".into(),
            iban: "CH9300762011623852957".into(),
            qr_iban: None,
            is_primary: true,
        },
    )
    .await
    .unwrap();
    assert_eq!(created.bank_name, "UBS");

    // Second call updates
    let updated = bank_accounts::upsert_primary(
        &pool,
        NewBankAccount {
            company_id,
            bank_name: "PostFinance".into(),
            iban: "CH1809000000306547981".into(),
            qr_iban: None,
            is_primary: true,
        },
    )
    .await
    .unwrap();
    assert_eq!(updated.bank_name, "PostFinance");
    assert_eq!(updated.id, created.id); // Same row updated

    // Only one account in DB
    let list = bank_accounts::list_by_company(&pool, company_id)
        .await
        .unwrap();
    assert_eq!(list.len(), 1);
}

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn fk_constraint_rejects_missing_company(pool: MySqlPool) {
    let result = bank_accounts::create(
        &pool,
        NewBankAccount {
            company_id: 999_999,
            bank_name: "Test".into(),
            iban: "CH9300762011623852957".into(),
            qr_iban: None,
            is_primary: true,
        },
    )
    .await;
    assert!(result.is_err());
}
