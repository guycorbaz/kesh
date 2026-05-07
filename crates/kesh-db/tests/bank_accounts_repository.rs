//! Tests d'intégration pour `repositories::bank_accounts`.

use kesh_db::entities::account::AccountType;
use kesh_db::entities::{Language, NewAccount, NewBankAccount, NewCompany, OrgType};
use kesh_db::errors::DbError;
use kesh_db::repositories::{accounts, bank_accounts, companies};
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
    let found = bank_accounts::find_primary(&pool, company_id)
        .await
        .unwrap();
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

/// KF-004 : second appel `upsert_primary` avec payload identique → pas de bump
/// version, `updated_at` inchangé. Pas d'assertion audit_log : `bank_accounts`
/// n'écrit pas d'audit log v0.1.
#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn upsert_primary_no_op_returns_unchanged_entity(pool: MySqlPool) {
    let company_id = create_test_company(&pool).await;

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
    let version_initial = created.version;
    let updated_at_initial = created.updated_at;

    let result = bank_accounts::upsert_primary(
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

    assert_eq!(
        result.version, version_initial,
        "version doit être inchangée"
    );
    assert_eq!(
        result.updated_at, updated_at_initial,
        "updated_at doit être inchangé"
    );
    assert_eq!(result.id, created.id);
}

/// KF-004 régression : second appel avec `iban` modifié → bump version.
#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn upsert_primary_partial_change_bumps_version(pool: MySqlPool) {
    let company_id = create_test_company(&pool).await;

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
    let version_initial = created.version;

    let updated = bank_accounts::upsert_primary(
        &pool,
        NewBankAccount {
            company_id,
            bank_name: "UBS".into(),
            iban: "CH1809000000306547981".into(),
            qr_iban: None,
            is_primary: true,
        },
    )
    .await
    .unwrap();
    assert_eq!(updated.version, version_initial + 1);
    assert_eq!(updated.iban, "CH1809000000306547981");
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

// ============================================================
// Story 8-5a-zero — `set_journal_account_id_for_company` tests
// ============================================================

/// Helper : crée un user pour pouvoir appeler `accounts::create` (qui logge
/// audit_log avec un user_id réel).
async fn create_test_user(pool: &MySqlPool, company_id: i64, username: &str) -> i64 {
    use kesh_db::entities::{NewUser, Role};
    use kesh_db::repositories::users;

    users::create(
        pool,
        NewUser {
            username: username.into(),
            password_hash: "$argon2id$v=19$m=65536,t=3,p=4$abcdefgh$ijklmnop".into(),
            role: Role::Admin,
            active: true,
            company_id,
        },
    )
    .await
    .unwrap()
    .id
}

/// Helper : crée un compte du plan comptable (Asset par défaut).
async fn create_account(
    pool: &MySqlPool,
    company_id: i64,
    user_id: i64,
    number: &str,
    name: &str,
    account_type: AccountType,
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
        },
    )
    .await
    .unwrap()
    .id
}

/// Helper : crée un bank_account primary CHF.
async fn create_bank_account(pool: &MySqlPool, company_id: i64) -> i64 {
    bank_accounts::create(
        pool,
        NewBankAccount {
            company_id,
            bank_name: "UBS".into(),
            iban: "CH9300762011623852957".into(),
            qr_iban: None,
            is_primary: true,
        },
    )
    .await
    .unwrap()
    .id
}

/// AC #76 — happy path : `set_journal_account_id_for_company` met à jour la
/// colonne et bumpe la version.
#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn set_journal_account_id_updates_column_and_bumps_version(pool: MySqlPool) {
    let company_id = create_test_company(&pool).await;
    let user_id = create_test_user(&pool, company_id, "admin").await;
    let account_id = create_account(
        &pool,
        company_id,
        user_id,
        "1020",
        "Caisse banque",
        AccountType::Asset,
    )
    .await;
    let bank_account_id = create_bank_account(&pool, company_id).await;

    let pre = bank_accounts::find_by_id_for_company(&pool, company_id, bank_account_id)
        .await
        .unwrap()
        .expect("bank_account exists");
    assert_eq!(pre.journal_account_id, None);
    let pre_version = pre.version;

    let mut tx = pool.begin().await.unwrap();
    let updated = bank_accounts::set_journal_account_id_for_company(
        &mut tx,
        company_id,
        bank_account_id,
        Some(account_id),
        pre_version,
    )
    .await
    .unwrap();
    tx.commit().await.unwrap();

    assert_eq!(updated.journal_account_id, Some(account_id));
    assert_eq!(updated.version, pre_version + 1);

    // Régression DB : la row est bien mise à jour après commit.
    let post = bank_accounts::find_by_id_for_company(&pool, company_id, bank_account_id)
        .await
        .unwrap()
        .expect("bank_account exists");
    assert_eq!(post.journal_account_id, Some(account_id));
    assert_eq!(post.version, pre_version + 1);
}

/// AC #77 — optimistic lock : version mismatch retourne
/// `OptimisticLockConflict`.
#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn set_journal_account_id_returns_optimistic_lock_conflict_on_version_mismatch(
    pool: MySqlPool,
) {
    let company_id = create_test_company(&pool).await;
    let user_id = create_test_user(&pool, company_id, "admin").await;
    let account_id = create_account(
        &pool,
        company_id,
        user_id,
        "1020",
        "Caisse banque",
        AccountType::Asset,
    )
    .await;
    let bank_account_id = create_bank_account(&pool, company_id).await;

    let pre = bank_accounts::find_by_id_for_company(&pool, company_id, bank_account_id)
        .await
        .unwrap()
        .unwrap();

    let mut tx = pool.begin().await.unwrap();
    let result = bank_accounts::set_journal_account_id_for_company(
        &mut tx,
        company_id,
        bank_account_id,
        Some(account_id),
        pre.version + 99, // version mismatch volontaire
    )
    .await;
    let _ = tx.rollback().await;

    assert!(matches!(result, Err(DbError::OptimisticLockConflict)));
}

/// Multi-tenant safety : un caller `company_B` ne peut PAS modifier un
/// bank_account de `company_A` — retourne `NotFound`.
#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn set_journal_account_id_does_not_leak_cross_tenant(pool: MySqlPool) {
    let company_a = create_test_company(&pool).await;
    let company_b = companies::create(
        &pool,
        NewCompany {
            name: "Other SA".into(),
            address: "Rue Other 1".into(),
            ide_number: None,
            org_type: OrgType::Pme,
            accounting_language: Language::Fr,
            instance_language: Language::Fr,
        },
    )
    .await
    .unwrap()
    .id;

    let user_a = create_test_user(&pool, company_a, "admin_a").await;
    let account_a = create_account(
        &pool,
        company_a,
        user_a,
        "1020",
        "Caisse banque",
        AccountType::Asset,
    )
    .await;
    let bank_a = create_bank_account(&pool, company_a).await;

    let pre = bank_accounts::find_by_id_for_company(&pool, company_a, bank_a)
        .await
        .unwrap()
        .unwrap();

    // Caller = company_b, target = bank_account de company_a → 404 NotFound.
    let mut tx = pool.begin().await.unwrap();
    let result = bank_accounts::set_journal_account_id_for_company(
        &mut tx,
        company_b,
        bank_a,
        Some(account_a),
        pre.version,
    )
    .await;
    let _ = tx.rollback().await;

    assert!(matches!(result, Err(DbError::NotFound)));
}

/// Délier (set None) un bank_account précédemment lié.
#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn set_journal_account_id_to_null_unlinks_successfully(pool: MySqlPool) {
    let company_id = create_test_company(&pool).await;
    let user_id = create_test_user(&pool, company_id, "admin").await;
    let account_id = create_account(
        &pool,
        company_id,
        user_id,
        "1020",
        "Caisse banque",
        AccountType::Asset,
    )
    .await;
    let bank_account_id = create_bank_account(&pool, company_id).await;

    let pre = bank_accounts::find_by_id_for_company(&pool, company_id, bank_account_id)
        .await
        .unwrap()
        .unwrap();

    // Step 1 : link.
    let mut tx = pool.begin().await.unwrap();
    let linked = bank_accounts::set_journal_account_id_for_company(
        &mut tx,
        company_id,
        bank_account_id,
        Some(account_id),
        pre.version,
    )
    .await
    .unwrap();
    tx.commit().await.unwrap();
    assert_eq!(linked.journal_account_id, Some(account_id));

    // Step 2 : unlink (set None).
    let mut tx = pool.begin().await.unwrap();
    let unlinked = bank_accounts::set_journal_account_id_for_company(
        &mut tx,
        company_id,
        bank_account_id,
        None,
        linked.version,
    )
    .await
    .unwrap();
    tx.commit().await.unwrap();

    assert_eq!(unlinked.journal_account_id, None);
    assert_eq!(unlinked.version, linked.version + 1);
}

/// Régression entité : `find_by_id_for_company` retourne `journal_account_id`
/// quand il est posé. Vérifie que l'extension SELECT SQL fonctionne.
#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn find_by_id_for_company_returns_journal_account_id_when_set(pool: MySqlPool) {
    let company_id = create_test_company(&pool).await;
    let user_id = create_test_user(&pool, company_id, "admin").await;
    let account_id = create_account(
        &pool,
        company_id,
        user_id,
        "1030",
        "Banque",
        AccountType::Asset,
    )
    .await;
    let bank_account_id = create_bank_account(&pool, company_id).await;

    let pre = bank_accounts::find_by_id_for_company(&pool, company_id, bank_account_id)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(pre.journal_account_id, None);

    let mut tx = pool.begin().await.unwrap();
    bank_accounts::set_journal_account_id_for_company(
        &mut tx,
        company_id,
        bank_account_id,
        Some(account_id),
        pre.version,
    )
    .await
    .unwrap();
    tx.commit().await.unwrap();

    let post = bank_accounts::find_by_id_for_company(&pool, company_id, bank_account_id)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(post.journal_account_id, Some(account_id));
}

/// KF-004 court-circuit no-op : si le `journal_account_id` ne change pas,
/// la fonction retourne l'entité inchangée sans bumper version.
#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn set_journal_account_id_no_op_short_circuits_without_bump(pool: MySqlPool) {
    let company_id = create_test_company(&pool).await;
    let user_id = create_test_user(&pool, company_id, "admin").await;
    let account_id = create_account(
        &pool,
        company_id,
        user_id,
        "1020",
        "Caisse banque",
        AccountType::Asset,
    )
    .await;
    let bank_account_id = create_bank_account(&pool, company_id).await;

    // Setup : link une première fois.
    let pre = bank_accounts::find_by_id_for_company(&pool, company_id, bank_account_id)
        .await
        .unwrap()
        .unwrap();
    let mut tx = pool.begin().await.unwrap();
    let linked = bank_accounts::set_journal_account_id_for_company(
        &mut tx,
        company_id,
        bank_account_id,
        Some(account_id),
        pre.version,
    )
    .await
    .unwrap();
    tx.commit().await.unwrap();
    let version_after_link = linked.version;

    // Re-call avec la même valeur → no-op short-circuit.
    let mut tx = pool.begin().await.unwrap();
    let unchanged = bank_accounts::set_journal_account_id_for_company(
        &mut tx,
        company_id,
        bank_account_id,
        Some(account_id),
        version_after_link,
    )
    .await
    .unwrap();
    tx.commit().await.unwrap();

    assert_eq!(
        unchanged.version, version_after_link,
        "version doit rester inchangée"
    );
    assert_eq!(unchanged.journal_account_id, Some(account_id));
}
