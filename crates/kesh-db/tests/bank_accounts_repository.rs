//! Tests d'intégration pour `repositories::bank_accounts`.

use kesh_db::entities::account::AccountType;
use kesh_db::entities::address::StructuredAddress;
use kesh_db::entities::{Language, NewAccount, NewBankAccount, NewCompany, OrgType};
use kesh_db::errors::DbError;
use kesh_db::repositories::bank_accounts::{BankAccountListFilter, BankAccountPutFields};
use kesh_db::repositories::company_invoice_settings::ClaimAccounts;
use kesh_db::repositories::{accounts, bank_accounts, companies};
use sqlx::MySqlPool;

async fn create_test_company(pool: &MySqlPool) -> i64 {
    companies::create(
        pool,
        NewCompany {
            name: "Test SA".into(),
            first_name: None,
            last_name: None,
            address_structured: StructuredAddress {
                street: "Rue Test".into(),
                building: "1".into(),
                postal_code: "1000".into(),
                city: "Lausanne".into(),
                country: "CH".into(),
            },
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

#[sqlx::test(migrations = "./test-schema")]
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

#[sqlx::test(migrations = "./test-schema")]
async fn find_primary_returns_none_when_empty(pool: MySqlPool) {
    let company_id = create_test_company(&pool).await;
    let found = bank_accounts::find_primary(&pool, company_id)
        .await
        .unwrap();
    assert!(found.is_none());
}

#[sqlx::test(migrations = "./test-schema")]
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

    let list = bank_accounts::list_by_company(&pool, company_id, Default::default())
        .await
        .unwrap();
    assert_eq!(list.len(), 1);
    assert_eq!(list[0].bank_name, "UBS");
}

#[sqlx::test(migrations = "./test-schema")]
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
    let list = bank_accounts::list_by_company(&pool, company_id, Default::default())
        .await
        .unwrap();
    assert_eq!(list.len(), 1);
}

/// KF-004 : second appel `upsert_primary` avec payload identique → pas de bump
/// version, `updated_at` inchangé. Pas d'assertion audit_log : `bank_accounts`
/// n'écrit pas d'audit log v0.1.
#[sqlx::test(migrations = "./test-schema")]
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
#[sqlx::test(migrations = "./test-schema")]
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

#[sqlx::test(migrations = "./test-schema")]
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
            email: None,
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
            role: None,
            postable: true,
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
#[sqlx::test(migrations = "./test-schema")]
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
    let (updated, before) = bank_accounts::set_journal_account_id_for_company(
        &mut tx,
        company_id,
        bank_account_id,
        Some(account_id),
        pre_version,
        &ClaimAccounts::default(),
    )
    .await
    .unwrap();
    tx.commit().await.unwrap();

    assert_eq!(updated.journal_account_id, Some(account_id));
    assert_eq!(updated.version, pre_version + 1);
    assert_eq!(
        before.journal_account_id, None,
        "before snapshot pre-update"
    );
    assert_eq!(before.version, pre_version, "before version pre-bump");

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
#[sqlx::test(migrations = "./test-schema")]
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
        &ClaimAccounts::default(),
    )
    .await;
    let _ = tx.rollback().await;

    assert!(matches!(result, Err(DbError::OptimisticLockConflict)));
}

/// Multi-tenant safety : un caller `company_B` ne peut PAS modifier un
/// bank_account de `company_A` — retourne `NotFound`.
#[sqlx::test(migrations = "./test-schema")]
async fn set_journal_account_id_does_not_leak_cross_tenant(pool: MySqlPool) {
    let company_a = create_test_company(&pool).await;
    let company_b = companies::create(
        &pool,
        NewCompany {
            name: "Other SA".into(),
            first_name: None,
            last_name: None,
            address_structured: StructuredAddress {
                street: "Rue Test".into(),
                building: "1".into(),
                postal_code: "1000".into(),
                city: "Lausanne".into(),
                country: "CH".into(),
            },
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
        &ClaimAccounts::default(),
    )
    .await;
    let _ = tx.rollback().await;

    assert!(matches!(result, Err(DbError::NotFound)));
}

/// Délier (set None) un bank_account précédemment lié.
#[sqlx::test(migrations = "./test-schema")]
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
    let (linked, _before) = bank_accounts::set_journal_account_id_for_company(
        &mut tx,
        company_id,
        bank_account_id,
        Some(account_id),
        pre.version,
        &ClaimAccounts::default(),
    )
    .await
    .unwrap();
    tx.commit().await.unwrap();
    assert_eq!(linked.journal_account_id, Some(account_id));

    // Step 2 : unlink (set None).
    let mut tx = pool.begin().await.unwrap();
    let (unlinked, _before) = bank_accounts::set_journal_account_id_for_company(
        &mut tx,
        company_id,
        bank_account_id,
        None,
        linked.version,
        &ClaimAccounts::default(),
    )
    .await
    .unwrap();
    tx.commit().await.unwrap();

    assert_eq!(unlinked.journal_account_id, None);
    assert_eq!(unlinked.version, linked.version + 1);
}

/// Régression entité : `find_by_id_for_company` retourne `journal_account_id`
/// quand il est posé. Vérifie que l'extension SELECT SQL fonctionne.
#[sqlx::test(migrations = "./test-schema")]
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
    let _ = bank_accounts::set_journal_account_id_for_company(
        &mut tx,
        company_id,
        bank_account_id,
        Some(account_id),
        pre.version,
        &ClaimAccounts::default(),
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
#[sqlx::test(migrations = "./test-schema")]
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
    let (linked, _before) = bank_accounts::set_journal_account_id_for_company(
        &mut tx,
        company_id,
        bank_account_id,
        Some(account_id),
        pre.version,
        &ClaimAccounts::default(),
    )
    .await
    .unwrap();
    tx.commit().await.unwrap();
    let version_after_link = linked.version;

    // Re-call avec la même valeur → no-op short-circuit.
    let mut tx = pool.begin().await.unwrap();
    let (unchanged, before) = bank_accounts::set_journal_account_id_for_company(
        &mut tx,
        company_id,
        bank_account_id,
        Some(account_id),
        version_after_link,
        &ClaimAccounts::default(),
    )
    .await
    .unwrap();
    tx.commit().await.unwrap();

    assert_eq!(
        unchanged.version, version_after_link,
        "version doit rester inchangée"
    );
    assert_eq!(unchanged.journal_account_id, Some(account_id));
    // No-op : `before` et `updated` doivent référencer le même état.
    assert_eq!(
        before.version, unchanged.version,
        "no-op : before == updated"
    );
    assert_eq!(before.journal_account_id, unchanged.journal_account_id);
}

/// AC #75 — Pass 1 code review Sonnet 4.6 (P-M1) : la migration
/// `20260507200001_bank_account_journal_link.sql` crée la colonne
/// `journal_account_id BIGINT NULL` (sans FK DB-level). La nullabilité
/// se vérifie par INSERT direct avec valeur NULL ; l'absence de FK se
/// vérifie par INSERT direct avec un id pointant vers une row inexistante
/// du plan comptable (la défense est applicative, scopée multi-tenant
/// dans le handler — KF-002).
#[sqlx::test(migrations = "./test-schema")]
async fn migration_creates_journal_account_id_column_nullable(pool: MySqlPool) {
    let company_id = create_test_company(&pool).await;

    // (a) Introspection : la colonne existe et est nullable.
    let is_nullable: String = sqlx::query_scalar(
        "SELECT IS_NULLABLE FROM INFORMATION_SCHEMA.COLUMNS \
         WHERE TABLE_SCHEMA = DATABASE() AND TABLE_NAME = 'bank_accounts' \
         AND COLUMN_NAME = 'journal_account_id'",
    )
    .fetch_one(&pool)
    .await
    .expect("colonne journal_account_id doit exister après migration");
    assert_eq!(
        is_nullable, "YES",
        "journal_account_id doit être nullable (IS_NULLABLE = 'YES')"
    );

    // (b) NULL accepté : INSERT avec journal_account_id explicitement NULL.
    sqlx::query(
        "INSERT INTO bank_accounts (company_id, bank_name, iban, qr_iban, is_primary, journal_account_id) \
         VALUES (?, ?, ?, NULL, FALSE, NULL)",
    )
    .bind(company_id)
    .bind("BankNullExplicit")
    .bind("CH4431999123000889012")
    .execute(&pool)
    .await
    .expect("INSERT bank_account avec journal_account_id NULL doit réussir");

    let null_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM bank_accounts WHERE company_id = ? AND journal_account_id IS NULL",
    )
    .bind(company_id)
    .fetch_one(&pool)
    .await
    .unwrap();
    assert!(
        null_count >= 1,
        "au moins un bank_account avec journal_account_id=NULL doit exister"
    );

    // (c) Pas de FK DB-level : INSERT avec un id pointant vers une row
    // inexistante de la table `accounts` doit réussir au niveau DB. La
    // défense d'intégrité est applicative (handler `accounts::find_by_id_in_company`).
    sqlx::query(
        "INSERT INTO bank_accounts (company_id, bank_name, iban, qr_iban, is_primary, journal_account_id) \
         VALUES (?, ?, ?, NULL, FALSE, ?)",
    )
    .bind(company_id)
    .bind("BankUnknownAccount")
    .bind("CH1809000000306547981")
    .bind(999_999_999_i64) // id volontairement inexistant
    .execute(&pool)
    .await
    .expect(
        "INSERT bank_account avec journal_account_id pointant vers une row inexistante doit \
         réussir (pas de FK DB, défense applicative — pattern Kesh multi-tenant)",
    );
}

/// P-H2 (Pass 1 code review Sonnet 4.6) : un client avec `expected_version`
/// stale sur un no-op (target == existing.journal_account_id) DOIT
/// recevoir `OptimisticLockConflict`, pas un 200 OK silencieux. La
/// version est validée AVANT le court-circuit no-op.
#[sqlx::test(migrations = "./test-schema")]
async fn set_journal_account_id_no_op_with_stale_version_returns_conflict(pool: MySqlPool) {
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

    // Setup : link une fois pour avoir une row avec journal_account_id posé
    // et une version > 1.
    let pre = bank_accounts::find_by_id_for_company(&pool, company_id, bank_account_id)
        .await
        .unwrap()
        .unwrap();
    let mut tx = pool.begin().await.unwrap();
    let (linked, _) = bank_accounts::set_journal_account_id_for_company(
        &mut tx,
        company_id,
        bank_account_id,
        Some(account_id),
        pre.version,
        &ClaimAccounts::default(),
    )
    .await
    .unwrap();
    tx.commit().await.unwrap();
    assert_eq!(linked.version, pre.version + 1);

    // No-op (target == existing) MAIS avec une `expected_version` stale
    // (= pre.version, pas linked.version).
    let mut tx = pool.begin().await.unwrap();
    let result = bank_accounts::set_journal_account_id_for_company(
        &mut tx,
        company_id,
        bank_account_id,
        Some(account_id),
        pre.version, // version stale
        &ClaimAccounts::default(),
    )
    .await;
    let _ = tx.rollback().await;

    assert!(
        matches!(result, Err(DbError::OptimisticLockConflict)),
        "no-op avec version stale doit retourner OptimisticLockConflict, got: {result:?}"
    );
}

// ===========================================================================
// Story v014-1 — Tests contrat archived invariants (F5 Pass 1 code review,
// MANDATORY FINDING-6 Pass 3 Opus)
// ===========================================================================

/// Helper : insère un bank_account avec `archived=true` directement en DB
/// (bypasse `archive_for_company` qui exige tx). Utilisé par les tests
/// contrat pour préparer l'état "compte déjà archivé".
async fn create_archived_bank_account(pool: &MySqlPool, company_id: i64, iban: &str) -> i64 {
    let ba = bank_accounts::create(
        pool,
        NewBankAccount {
            company_id,
            bank_name: "Archived Bank".into(),
            iban: iban.into(),
            qr_iban: None,
            is_primary: false,
        },
    )
    .await
    .unwrap();
    sqlx::query("UPDATE bank_accounts SET archived = TRUE WHERE id = ?")
        .bind(ba.id)
        .execute(pool)
        .await
        .unwrap();
    ba.id
}

#[sqlx::test(migrations = "./test-schema")]
async fn archived_invariants_find_primary_excludes_archived(pool: MySqlPool) {
    let company_id = create_test_company(&pool).await;

    // Crée un primary actif.
    let primary = bank_accounts::create(
        &pool,
        NewBankAccount {
            company_id,
            bank_name: "Active Primary".into(),
            iban: "CH9300762011623852957".into(),
            qr_iban: None,
            is_primary: true,
        },
    )
    .await
    .unwrap();

    // find_primary retourne le compte actif.
    let found = bank_accounts::find_primary(&pool, company_id)
        .await
        .unwrap();
    assert!(found.is_some());
    assert_eq!(found.unwrap().id, primary.id);

    // Archive le primary (UPDATE direct car archive_for_company exige tx).
    sqlx::query("UPDATE bank_accounts SET archived = TRUE WHERE id = ?")
        .bind(primary.id)
        .execute(&pool)
        .await
        .unwrap();

    // find_primary retourne désormais None (filtre archived=FALSE — F1 Pass 3 Opus
    // → garantit que invoice_pdf.rs:83 ne renvoie jamais un primary archivé).
    let found_after = bank_accounts::find_primary(&pool, company_id)
        .await
        .unwrap();
    assert!(
        found_after.is_none(),
        "find_primary doit exclure le primary archivé (F1 Pass 3 Opus)"
    );
}

#[sqlx::test(migrations = "./test-schema")]
async fn archived_invariants_find_by_id_returns_archived(pool: MySqlPool) {
    let company_id = create_test_company(&pool).await;
    let archived_id =
        create_archived_bank_account(&pool, company_id, "CH9300762011623852957").await;

    // Contrat F6 Pass 3 Opus : find_by_id_for_company NE filtre PAS archived
    // (les call sites décident). Ce contrat est nécessaire pour que les call
    // sites de mutation puissent eux-mêmes décider de rejeter avec 404
    // anti-énumération KF-002.
    let found = bank_accounts::find_by_id_for_company(&pool, company_id, archived_id)
        .await
        .unwrap();
    assert!(
        found.is_some(),
        "find_by_id_for_company doit retourner la row même si archivée (contrat F6 Pass 3 Opus)"
    );
    assert!(found.unwrap().archived);
}

#[sqlx::test(migrations = "./test-schema")]
async fn archived_invariants_list_by_company_filters_or_includes(pool: MySqlPool) {
    let company_id = create_test_company(&pool).await;

    // Crée 1 actif + 1 archivé.
    bank_accounts::create(
        &pool,
        NewBankAccount {
            company_id,
            bank_name: "Active".into(),
            iban: "CH9300762011623852957".into(),
            qr_iban: None,
            is_primary: true,
        },
    )
    .await
    .unwrap();
    create_archived_bank_account(&pool, company_id, "CH4431999123000889012").await;

    // include_archived=false (défaut UI) : exclut.
    let active_only = bank_accounts::list_by_company(&pool, company_id, Default::default())
        .await
        .unwrap();
    assert_eq!(active_only.len(), 1);
    assert!(!active_only[0].archived);

    // include_archived=true (export ZIP souveraineté) : retourne tout.
    let all = bank_accounts::list_by_company(&pool, company_id, BankAccountListFilter::ALL)
        .await
        .unwrap();
    assert_eq!(all.len(), 2);
}

#[sqlx::test(migrations = "./test-schema")]
async fn archived_invariants_set_journal_account_id_on_archived_returns_not_found(pool: MySqlPool) {
    let company_id = create_test_company(&pool).await;
    let archived_id =
        create_archived_bank_account(&pool, company_id, "CH9300762011623852957").await;

    let mut tx = pool.begin().await.unwrap();
    let result = bank_accounts::set_journal_account_id_for_company(
        &mut tx,
        company_id,
        archived_id,
        Some(1),
        1,
        &ClaimAccounts::default(),
    )
    .await;
    let _ = tx.rollback().await;

    // F2 Pass 3 Opus : compte archivé → DbError::NotFound (anti-énumération
    // KF-002). Le handler convertit en AppError::BankAccountNotFound (404).
    assert!(
        matches!(result, Err(DbError::NotFound)),
        "set_journal_account_id sur compte archivé doit retourner NotFound (F2 Pass 3 Opus), got: {result:?}"
    );
}

#[sqlx::test(migrations = "./test-schema")]
async fn archived_invariants_update_for_company_on_archived_returns_not_found(pool: MySqlPool) {
    let company_id = create_test_company(&pool).await;
    let archived_id =
        create_archived_bank_account(&pool, company_id, "CH9300762011623852957").await;

    let new = NewBankAccount {
        company_id,
        bank_name: "Updated".into(),
        iban: "CH4431999123000889012".into(),
        qr_iban: None,
        is_primary: false,
    };
    let mut tx = pool.begin().await.unwrap();
    let result = bank_accounts::update_for_company(
        &mut tx,
        company_id,
        archived_id,
        &new,
        BankAccountPutFields {
            journal_account_id: None,
            description: None,
        },
        1,
        &ClaimAccounts::default(),
    )
    .await;
    let _ = tx.rollback().await;

    assert!(
        matches!(result, Err(DbError::NotFound)),
        "update_for_company sur compte archivé doit retourner NotFound (F6 Pass 3 Opus), got: {result:?}"
    );
}

#[sqlx::test(migrations = "./test-schema")]
async fn archived_invariants_archive_for_company_on_already_archived_returns_not_found(
    pool: MySqlPool,
) {
    let company_id = create_test_company(&pool).await;
    let archived_id =
        create_archived_bank_account(&pool, company_id, "CH9300762011623852957").await;

    let mut tx = pool.begin().await.unwrap();
    let result = bank_accounts::archive_for_company(&mut tx, company_id, archived_id, 1).await;
    let _ = tx.rollback().await;

    // Cohérent avec F6 Pass 3 Opus : idempotence non-supportée v0.1 (L1) —
    // un DELETE sur compte déjà archivé retourne 404 plutôt que 200 idempotent.
    assert!(
        matches!(result, Err(DbError::NotFound)),
        "archive_for_company sur compte déjà archivé doit retourner NotFound (F6 Pass 3 Opus), got: {result:?}"
    );
}

// ---------------------------------------------------------------------------
// Story 15-5b (AC12, #427) — `DbError::AccountsNotPostable` rendue par le
// dépôt, sous le verrou de la ligne, seulement si le compte lié change.
// ---------------------------------------------------------------------------

async fn set_not_postable(pool: &MySqlPool, account_id: i64) {
    sqlx::query("UPDATE accounts SET postable = FALSE, version = version + 1 WHERE id = ?")
        .bind(account_id)
        .execute(pool)
        .await
        .expect("set not postable");
}

fn assert_not_postable<T: std::fmt::Debug>(res: Result<T, DbError>, account_id: i64, number: &str) {
    match res {
        Err(DbError::AccountsNotPostable(list)) => {
            let v: Vec<(i64, String)> = list
                .iter()
                .map(|a| (a.account_id, a.account_number.clone()))
                .collect();
            assert_eq!(v, vec![(account_id, number.to_string())]);
        }
        other => panic!("attendu AccountsNotPostable, obtenu {other:?}"),
    }
}

fn bank_payload(company_id: i64) -> NewBankAccount {
    NewBankAccount {
        company_id,
        bank_name: "UBS".into(),
        iban: "CH9300762011623852957".into(),
        qr_iban: None,
        is_primary: true,
    }
}

/// AC12 — `set_journal_account_id_for_company` : un nouveau compte non
/// imputable est refusé ; le même compte, devenu non imputable une fois lié,
/// court-circuite sans contrôle.
#[sqlx::test(migrations = "./test-schema")]
async fn set_journal_account_id_guards_only_a_changed_account(pool: MySqlPool) {
    let company_id = create_test_company(&pool).await;
    let user_id = create_test_user(&pool, company_id, "admin").await;
    let linked = create_account(
        &pool,
        company_id,
        user_id,
        "1020",
        "Banque",
        AccountType::Asset,
    )
    .await;
    let target = create_account(
        &pool,
        company_id,
        user_id,
        "1021",
        "Banque 2",
        AccountType::Asset,
    )
    .await;
    let bank_account_id = create_bank_account(&pool, company_id).await;

    let mut tx = pool.begin().await.unwrap();
    let (after, _) = bank_accounts::set_journal_account_id_for_company(
        &mut tx,
        company_id,
        bank_account_id,
        Some(linked),
        1,
        &ClaimAccounts::default(),
    )
    .await
    .unwrap();
    tx.commit().await.unwrap();
    set_not_postable(&pool, linked).await;
    set_not_postable(&pool, target).await;

    let mut tx = pool.begin().await.unwrap();
    let res = bank_accounts::set_journal_account_id_for_company(
        &mut tx,
        company_id,
        bank_account_id,
        Some(target),
        after.version,
        &ClaimAccounts::default(),
    )
    .await;
    drop(tx);
    assert_not_postable(res, target, "1021");

    let mut tx = pool.begin().await.unwrap();
    bank_accounts::set_journal_account_id_for_company(
        &mut tx,
        company_id,
        bank_account_id,
        Some(linked),
        after.version,
        &ClaimAccounts::default(),
    )
    .await
    .expect("inchangé : no-op, sans contrôle");
}

/// AC12 — `update_for_company` : idem, et la version périmée passe avant.
#[sqlx::test(migrations = "./test-schema")]
async fn update_for_company_guards_only_a_changed_account(pool: MySqlPool) {
    let company_id = create_test_company(&pool).await;
    let user_id = create_test_user(&pool, company_id, "admin").await;
    let linked = create_account(
        &pool,
        company_id,
        user_id,
        "1020",
        "Banque",
        AccountType::Asset,
    )
    .await;
    let target = create_account(
        &pool,
        company_id,
        user_id,
        "1021",
        "Banque 2",
        AccountType::Asset,
    )
    .await;
    let bank_account_id = create_bank_account(&pool, company_id).await;

    let mut tx = pool.begin().await.unwrap();
    let (after, _) = bank_accounts::update_for_company(
        &mut tx,
        company_id,
        bank_account_id,
        &bank_payload(company_id),
        BankAccountPutFields {
            journal_account_id: Some(linked),
            description: None,
        },
        1,
        &ClaimAccounts::default(),
    )
    .await
    .unwrap();
    tx.commit().await.unwrap();
    set_not_postable(&pool, linked).await;
    set_not_postable(&pool, target).await;

    // Version périmée : 409 d'abord.
    let mut tx = pool.begin().await.unwrap();
    let res = bank_accounts::update_for_company(
        &mut tx,
        company_id,
        bank_account_id,
        &bank_payload(company_id),
        BankAccountPutFields {
            journal_account_id: Some(target),
            description: None,
        },
        after.version + 7,
        &ClaimAccounts::default(),
    )
    .await;
    drop(tx);
    assert!(
        matches!(res, Err(DbError::OptimisticLockConflict)),
        "attendu OptimisticLockConflict, obtenu {res:?}"
    );

    let mut tx = pool.begin().await.unwrap();
    let res = bank_accounts::update_for_company(
        &mut tx,
        company_id,
        bank_account_id,
        &bank_payload(company_id),
        BankAccountPutFields {
            journal_account_id: Some(target),
            description: None,
        },
        after.version,
        &ClaimAccounts::default(),
    )
    .await;
    drop(tx);
    assert_not_postable(res, target, "1021");

    let mut tx = pool.begin().await.unwrap();
    bank_accounts::update_for_company(
        &mut tx,
        company_id,
        bank_account_id,
        &bank_payload(company_id),
        BankAccountPutFields {
            journal_account_id: Some(linked),
            description: None,
        },
        after.version,
        &ClaimAccounts::default(),
    )
    .await
    .expect("compte inchangé : accepté");
}

/// Test 3 (Story 15-7a1, AC 3) — `upsert_primary_in_tx` rend ce qu'il a
/// réellement fait : `Created`, `Updated { before, after }`, `Unchanged` (sans
/// annuler : la transaction reste utilisable) ; et ne commite jamais.
#[sqlx::test(migrations = "./test-schema")]
async fn upsert_primary_in_tx_reports_created_updated_unchanged(pool: MySqlPool) {
    use bank_accounts::UpsertPrimaryOutcome;

    let company_id = create_test_company(&pool).await;
    let payload = |bank: &str| NewBankAccount {
        company_id,
        bank_name: bank.into(),
        iban: "CH9300762011623852957".into(),
        qr_iban: None,
        is_primary: true,
    };

    // Rollback après `Created` ⇒ aucune ligne (aucun commit interne).
    let mut tx = pool.begin().await.unwrap();
    let outcome = bank_accounts::upsert_primary_in_tx(&mut tx, payload("UBS"))
        .await
        .unwrap();
    assert!(
        matches!(outcome, UpsertPrimaryOutcome::Created(_)),
        "{outcome:?}"
    );
    tx.rollback().await.unwrap();
    assert!(
        bank_accounts::find_primary(&pool, company_id)
            .await
            .unwrap()
            .is_none(),
        "rollback après Created : aucune ligne"
    );

    // Created, commité.
    let mut tx = pool.begin().await.unwrap();
    let created = match bank_accounts::upsert_primary_in_tx(&mut tx, payload("UBS"))
        .await
        .unwrap()
    {
        UpsertPrimaryOutcome::Created(a) => a,
        other => panic!("Created attendu, obtenu {other:?}"),
    };
    tx.commit().await.unwrap();
    assert_eq!(created.bank_name, "UBS");

    // Updated { before, after }.
    let mut tx = pool.begin().await.unwrap();
    let (before, after) = match bank_accounts::upsert_primary_in_tx(&mut tx, payload("PostFinance"))
        .await
        .unwrap()
    {
        UpsertPrimaryOutcome::Updated { before, after } => (before, after),
        other => panic!("Updated attendu, obtenu {other:?}"),
    };
    tx.commit().await.unwrap();
    assert_eq!(before.bank_name, "UBS", "before = valeurs d'avant");
    assert_eq!(before.version, created.version);
    assert_eq!(after.bank_name, "PostFinance");
    assert_eq!(after.version, before.version + 1);

    // Unchanged : version inchangée, transaction toujours utilisable.
    let mut tx = pool.begin().await.unwrap();
    let unchanged = match bank_accounts::upsert_primary_in_tx(&mut tx, payload("PostFinance"))
        .await
        .unwrap()
    {
        UpsertPrimaryOutcome::Unchanged(a) => a,
        other => panic!("Unchanged attendu, obtenu {other:?}"),
    };
    assert_eq!(
        unchanged.version, after.version,
        "no-op : version inchangée"
    );
    let still_usable: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM bank_accounts WHERE company_id = ?")
            .bind(company_id)
            .fetch_one(&mut *tx)
            .await
            .expect("transaction toujours utilisable après Unchanged");
    assert_eq!(still_usable, 1);
    tx.rollback().await.unwrap();
}

/// Revue P1 (B-3) — `upsert_primary_in_tx` ne commite pas non plus sur
/// `Updated` : un rollback de l'appelant rend le compte d'avant, version
/// comprise. (Le test 3 ne le prouvait que pour `Created`.)
#[sqlx::test(migrations = "./test-schema")]
async fn upsert_primary_in_tx_updated_is_not_committed(pool: MySqlPool) {
    use bank_accounts::UpsertPrimaryOutcome;

    let company_id = create_test_company(&pool).await;
    let payload = |bank: &str| NewBankAccount {
        company_id,
        bank_name: bank.into(),
        iban: "CH9300762011623852957".into(),
        qr_iban: None,
        is_primary: true,
    };
    let original = bank_accounts::upsert_primary(&pool, payload("UBS"))
        .await
        .expect("compte principal initial");

    let mut tx = pool.begin().await.unwrap();
    let outcome = bank_accounts::upsert_primary_in_tx(&mut tx, payload("PostFinance"))
        .await
        .unwrap();
    assert!(
        matches!(outcome, UpsertPrimaryOutcome::Updated { .. }),
        "{outcome:?}"
    );
    tx.rollback().await.unwrap();

    let post = bank_accounts::find_primary(&pool, company_id)
        .await
        .unwrap()
        .expect("compte principal toujours présent");
    assert_eq!(
        post.bank_name, "UBS",
        "rollback après Updated : valeurs d'avant"
    );
    assert_eq!(post.version, original.version, "rollback : version d'avant");
}

/// Revue P1 (B-3) — l'enveloppe pool `upsert_primary`, sur erreur, rend
/// l'erreur d'origine (rollback *best-effort*) et n'écrit rien. L'erreur est
/// provoquée par une société inexistante (clé étrangère, à l'`INSERT`).
///
/// ⚠️ Angle mort assumé (revue P1, A-2) : la branche
/// `OptimisticLockConflict` de `upsert_primary_in_tx` n'est exercée par aucun
/// test. Elle suit un `SELECT … FOR UPDATE` dans la même transaction, qui
/// tient le X-lock sur la ligne : la version ne peut pas changer entre la
/// lecture et l'`UPDATE`, et la branche n'est pas atteignable depuis un test
/// sans modifier le code. Son contrat (« sans annuler ») est tenu par lecture.
#[sqlx::test(migrations = "./test-schema")]
async fn upsert_primary_returns_original_error_and_writes_nothing(pool: MySqlPool) {
    let missing_company = i64::MAX - 7;
    let err = bank_accounts::upsert_primary(
        &pool,
        NewBankAccount {
            company_id: missing_company,
            bank_name: "UBS".into(),
            iban: "CH9300762011623852957".into(),
            qr_iban: None,
            is_primary: true,
        },
    )
    .await
    .expect_err("société inexistante : clé étrangère");
    assert!(
        matches!(err, DbError::ForeignKeyViolation(_)),
        "erreur d'origine attendue, obtenu {err:?}"
    );
    let n: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM bank_accounts WHERE company_id = ?")
        .bind(missing_company)
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(n, 0);
}

// ---------------------------------------------------------------------------
// Story 15-6c (#474) — un compte bancaire ne se lie pas au compte de créance
// désigné (`DbError::BankAccountLedgerIsClaimAccount`), sous le verrou de la
// ligne, après la postabilité, seulement si le compte lié change.
// ---------------------------------------------------------------------------

/// Test 5 (AC4) — `set_journal_account_id_for_company` avec des
/// `ClaimAccounts` désignant la cible : refus, rien d'écrit, `version`
/// inchangée — côté débiteurs comme côté créanciers. Avec des `claims` vides,
/// la même cible est acceptée.
#[sqlx::test(migrations = "./test-schema")]
async fn set_journal_account_id_refuses_a_designated_claim_account(pool: MySqlPool) {
    use kesh_db::errors::ClaimSide;
    let company_id = create_test_company(&pool).await;
    let user_id = create_test_user(&pool, company_id, "admin").await;
    let receivable = create_account(
        &pool,
        company_id,
        user_id,
        "1100",
        "Débiteurs",
        AccountType::Asset,
    )
    .await;
    let payable = create_account(
        &pool,
        company_id,
        user_id,
        "2000",
        "Créanciers",
        AccountType::Liability,
    )
    .await;
    let bank_account_id = create_bank_account(&pool, company_id).await;
    let claims = ClaimAccounts {
        receivable: Some(receivable),
        payable: Some(payable),
    };

    for (target, number, side) in [
        (receivable, "1100", ClaimSide::Receivable),
        (payable, "2000", ClaimSide::Payable),
    ] {
        let mut tx = pool.begin().await.unwrap();
        let res = bank_accounts::set_journal_account_id_for_company(
            &mut tx,
            company_id,
            bank_account_id,
            Some(target),
            1,
            &claims,
        )
        .await;
        drop(tx);
        match res {
            Err(DbError::BankAccountLedgerIsClaimAccount {
                account_id,
                account_number,
                claim,
            }) => {
                assert_eq!(account_id, target);
                assert_eq!(account_number.as_deref(), Some(number));
                assert_eq!(claim, side);
            }
            other => panic!("attendu BankAccountLedgerIsClaimAccount, obtenu {other:?}"),
        }
        let row = bank_accounts::find_by_id_for_company(&pool, company_id, bank_account_id)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(row.journal_account_id, None, "rien n'est écrit");
        assert_eq!(row.version, 1, "version inchangée");
    }

    // Mêmes cibles, `claims` vides : acceptées (la garde ne joue que sur les
    // comptes que la route a lus dans les réglages).
    let mut tx = pool.begin().await.unwrap();
    let (linked, _) = bank_accounts::set_journal_account_id_for_company(
        &mut tx,
        company_id,
        bank_account_id,
        Some(receivable),
        1,
        &ClaimAccounts::default(),
    )
    .await
    .unwrap();
    tx.commit().await.unwrap();
    assert_eq!(linked.journal_account_id, Some(receivable));
}

/// Story 15-6c (AC4 ; revue de code P1, A5) — **exemption « inchangé »**, au
/// niveau du dépôt : un compte bancaire **déjà** lié au compte débiteurs
/// (donnée antérieure, posée ici avec des `claims` vides) reste modifiable.
/// Le remplacement qui renvoie le même compte lié (`update_for_company`) passe
/// et écrit les autres champs ; le lien au même compte
/// (`set_journal_account_id_for_company`) court-circuite en no-op — alors même
/// que `claims` désigne ce compte.
#[sqlx::test(migrations = "./test-schema")]
async fn already_linked_claim_account_is_exempt_in_the_repository(pool: MySqlPool) {
    let company_id = create_test_company(&pool).await;
    let user_id = create_test_user(&pool, company_id, "admin").await;
    let receivable = create_account(
        &pool,
        company_id,
        user_id,
        "1100",
        "Débiteurs",
        AccountType::Asset,
    )
    .await;
    let bank_account_id = create_bank_account(&pool, company_id).await;
    // Donnée antérieure : le lien fautif, posé sans la garde.
    let mut tx = pool.begin().await.unwrap();
    bank_accounts::set_journal_account_id_for_company(
        &mut tx,
        company_id,
        bank_account_id,
        Some(receivable),
        1,
        &ClaimAccounts::default(),
    )
    .await
    .unwrap();
    tx.commit().await.unwrap();
    let claims = ClaimAccounts {
        receivable: Some(receivable),
        payable: None,
    };

    // Remplacement : autre nom, même compte lié → accepté.
    let mut payload = bank_payload(company_id);
    payload.bank_name = "Raiffeisen".into();
    let mut tx = pool.begin().await.unwrap();
    let (after, _) = bank_accounts::update_for_company(
        &mut tx,
        company_id,
        bank_account_id,
        &payload,
        BankAccountPutFields {
            journal_account_id: Some(receivable),
            description: None,
        },
        2,
        &claims,
    )
    .await
    .expect("le compte lié inchangé n'est pas contrôlé");
    tx.commit().await.unwrap();
    assert_eq!(after.bank_name, "Raiffeisen");
    assert_eq!(after.journal_account_id, Some(receivable));
    assert_eq!(after.version, 3);

    // Lien au même compte → no-op, sans refus ni nouvelle version.
    let mut tx = pool.begin().await.unwrap();
    let (updated, before) = bank_accounts::set_journal_account_id_for_company(
        &mut tx,
        company_id,
        bank_account_id,
        Some(receivable),
        3,
        &claims,
    )
    .await
    .expect("le lien inchangé court-circuite avant la garde");
    tx.commit().await.unwrap();
    assert_eq!(updated.version, before.version, "no-op : version inchangée");
    assert_eq!(updated.journal_account_id, Some(receivable));
}

/// Test 11 (AC2) — **sérialisation, côté compte bancaire**. Une transaction
/// tenue à la main verrouille la ligne des réglages (`FOR UPDATE`) et y désigne
/// le compte X sans valider ; le lien démarre, lit les réglages en
/// `LOCK IN SHARE MODE` et **attend**. Une fois la désignation validée, il lit
/// X et refuse.
///
/// ⚠️ L'attente est prouvée par `attendre_une_requete_en_cours` sur le texte
/// `FROM company_invoice_settings` + `LOCK IN SHARE MODE` — couplé à la forme
/// du verrou de `claim_accounts_in_share_mode` : changer l'une, c'est changer
/// l'autre. Sans ce verrou (lecture simple), le lien lirait l'ancien réglage,
/// passerait, et la désignation concurrente aussi.
#[sqlx::test(migrations = "./test-schema")]
async fn bank_link_waits_for_a_concurrent_receivable_designation(pool: MySqlPool) {
    use kesh_db::repositories::company_invoice_settings;
    let company_id = create_test_company(&pool).await;
    let user_id = create_test_user(&pool, company_id, "admin").await;
    let x = create_account(
        &pool,
        company_id,
        user_id,
        "1100",
        "Débiteurs",
        AccountType::Asset,
    )
    .await;
    let bank_account_id = create_bank_account(&pool, company_id).await;
    sqlx::query("INSERT IGNORE INTO company_invoice_settings (company_id) VALUES (?)")
        .bind(company_id)
        .execute(&pool)
        .await
        .unwrap();

    // (1) La désignation concurrente, non validée, qui tient la ligne.
    let mut concurrent = pool.begin().await.unwrap();
    sqlx::query("SELECT company_id FROM company_invoice_settings WHERE company_id = ? FOR UPDATE")
        .bind(company_id)
        .execute(&mut *concurrent)
        .await
        .unwrap();
    sqlx::query(
        "UPDATE company_invoice_settings SET default_receivable_account_id = ? WHERE company_id = ?",
    )
    .bind(x)
    .bind(company_id)
    .execute(&mut *concurrent)
    .await
    .unwrap();

    // (2) Le lien démarre et attend.
    let p = pool.clone();
    let lien = tokio::spawn(async move {
        let mut tx = p.begin().await.unwrap();
        let claims = company_invoice_settings::claim_accounts_in_share_mode(&mut tx, company_id)
            .await
            .unwrap();
        bank_accounts::set_journal_account_id_for_company(
            &mut tx,
            company_id,
            bank_account_id,
            Some(x),
            1,
            &claims,
        )
        .await
        .map(|(updated, _)| updated.journal_account_id)
    });
    let vue = kesh_db::test_fixtures::attendre_une_requete_en_cours(
        &pool,
        &["FROM company_invoice_settings", "LOCK IN SHARE MODE"],
        || lien.is_finished(),
    )
    .await;
    if !vue {
        panic!("le lien a fini sans attendre le verrou : {:?}", lien.await);
    }

    // (3) La désignation est validée.
    concurrent.commit().await.unwrap();

    let result = lien.await.expect("tâche du lien");
    assert!(
        matches!(
            result,
            Err(DbError::BankAccountLedgerIsClaimAccount { account_id, .. }) if account_id == x
        ),
        "le lien devait attendre la désignation puis refuser — reçu {result:?}"
    );
}

// ===========================================================================
// Story 26-2a (#622) — description optionnelle du compte bancaire.
// ===========================================================================

/// Pose une description **par SQL**, hors de tout chemin du dépôt.
async fn set_description_sql(pool: &MySqlPool, id: i64, description: &str) {
    sqlx::query("UPDATE bank_accounts SET description = ? WHERE id = ?")
        .bind(description)
        .bind(id)
        .execute(pool)
        .await
        .unwrap();
}

async fn description_of(pool: &MySqlPool, id: i64) -> Option<String> {
    sqlx::query_scalar("SELECT description FROM bank_accounts WHERE id = ?")
        .bind(id)
        .fetch_one(pool)
        .await
        .unwrap()
}

/// AC2 — toutes les lectures hydratent la description (une seule liste de
/// colonnes, macro `columns!`) : `find_primary`, `find_by_id_for_company`,
/// `list_by_company` (deux variantes), `find_by_id_in_tx`.
#[sqlx::test(migrations = "./test-schema")]
async fn description_hydratee_par_toutes_les_lectures(pool: MySqlPool) {
    let company_id = create_test_company(&pool).await;
    let id = create_bank_account(&pool, company_id).await;
    set_description_sql(&pool, id, "Personnel").await;
    let attendu = Some("Personnel".to_string());

    let primary = bank_accounts::find_primary(&pool, company_id)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(primary.description, attendu);
    let by_id = bank_accounts::find_by_id_for_company(&pool, company_id, id)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(by_id.description, attendu);
    for filter in [BankAccountListFilter::default(), BankAccountListFilter::ALL] {
        let list = bank_accounts::list_by_company(&pool, company_id, filter)
            .await
            .unwrap();
        assert_eq!(list[0].description, attendu);
    }
    let mut tx = pool.begin().await.unwrap();
    let in_tx = bank_accounts::find_by_id_in_tx(&mut tx, company_id, id)
        .await
        .unwrap();
    assert_eq!(in_tx.description, attendu);
    assert_eq!(in_tx.display_name(), "UBS — Personnel");
}

/// AC2 (C-26-2-20) — `find_by_id_in_tx` est scopée par société : un id d'une
/// autre société → `NotFound`, comme un id inconnu.
#[sqlx::test(migrations = "./test-schema")]
async fn find_by_id_in_tx_est_scopee_par_societe(pool: MySqlPool) {
    let company_a = create_test_company(&pool).await;
    let company_b = create_test_company(&pool).await;
    let id_a = create_bank_account(&pool, company_a).await;

    let mut tx = pool.begin().await.unwrap();
    let res = bank_accounts::find_by_id_in_tx(&mut tx, company_b, id_a).await;
    assert!(matches!(res, Err(DbError::NotFound)), "obtenu {res:?}");
    let res = bank_accounts::find_by_id_in_tx(&mut tx, company_a, id_a + 10_000).await;
    assert!(matches!(res, Err(DbError::NotFound)), "obtenu {res:?}");
    assert!(
        bank_accounts::find_by_id_in_tx(&mut tx, company_a, id_a)
            .await
            .is_ok()
    );
}

/// AC2/AC4 — `update_for_company` écrit la description (`PUT`), et une
/// description `None` l'**efface** (remplacement complet, C-26-2-8).
#[sqlx::test(migrations = "./test-schema")]
async fn update_for_company_ecrit_puis_efface_la_description(pool: MySqlPool) {
    let company_id = create_test_company(&pool).await;
    let id = create_bank_account(&pool, company_id).await;

    let mut tx = pool.begin().await.unwrap();
    let (after, before) = bank_accounts::update_for_company(
        &mut tx,
        company_id,
        id,
        &bank_payload(company_id),
        BankAccountPutFields {
            journal_account_id: None,
            description: Some("Épargne"),
        },
        1,
        &ClaimAccounts::default(),
    )
    .await
    .unwrap();
    tx.commit().await.unwrap();
    assert_eq!(before.description, None);
    assert_eq!(after.description.as_deref(), Some("Épargne"));
    assert_eq!(description_of(&pool, id).await.as_deref(), Some("Épargne"));

    let mut tx = pool.begin().await.unwrap();
    let (after, _) = bank_accounts::update_for_company(
        &mut tx,
        company_id,
        id,
        &bank_payload(company_id),
        BankAccountPutFields {
            journal_account_id: None,
            description: None,
        },
        after.version,
        &ClaimAccounts::default(),
    )
    .await
    .unwrap();
    tx.commit().await.unwrap();
    assert_eq!(after.description, None);
    assert_eq!(description_of(&pool, id).await, None);
}

/// AC2 — l'upsert de l'onboarding (défense en profondeur, C-26-2-22) : un
/// changement de banque rend `Updated` et **conserve** la description (cible
/// de M5).
#[sqlx::test(migrations = "./test-schema")]
async fn upsert_primary_conserve_la_description(pool: MySqlPool) {
    let company_id = create_test_company(&pool).await;
    let id = create_bank_account(&pool, company_id).await;
    set_description_sql(&pool, id, "Personnel").await;

    let mut tx = pool.begin().await.unwrap();
    let outcome = bank_accounts::upsert_primary_in_tx(
        &mut tx,
        NewBankAccount {
            company_id,
            bank_name: "PostFinance".into(),
            iban: "CH9300762011623852957".into(),
            qr_iban: None,
            is_primary: true,
        },
    )
    .await
    .unwrap();
    tx.commit().await.unwrap();
    let bank_accounts::UpsertPrimaryOutcome::Updated { before, after } = outcome else {
        panic!("attendu Updated, obtenu {outcome:?}");
    };
    assert_eq!(before.description.as_deref(), Some("Personnel"));
    assert_eq!(after.bank_name, "PostFinance");
    assert_eq!(after.description.as_deref(), Some("Personnel"));
    assert_eq!(
        description_of(&pool, id).await.as_deref(),
        Some("Personnel")
    );
}

/// AC2 — le `PATCH` (lien au grand livre) ne touche pas la description.
#[sqlx::test(migrations = "./test-schema")]
async fn set_journal_account_id_conserve_la_description(pool: MySqlPool) {
    let company_id = create_test_company(&pool).await;
    let user_id = create_test_user(&pool, company_id, "u26").await;
    let ledger = create_account(
        &pool,
        company_id,
        user_id,
        "1020",
        "Banque",
        AccountType::Asset,
    )
    .await;
    let id = create_bank_account(&pool, company_id).await;
    set_description_sql(&pool, id, "Personnel").await;

    let mut tx = pool.begin().await.unwrap();
    let (after, _) = bank_accounts::set_journal_account_id_for_company(
        &mut tx,
        company_id,
        id,
        Some(ledger),
        1,
        &ClaimAccounts::default(),
    )
    .await
    .unwrap();
    tx.commit().await.unwrap();
    assert_eq!(after.journal_account_id, Some(ledger));
    assert_eq!(after.description.as_deref(), Some("Personnel"));
    assert_eq!(
        description_of(&pool, id).await.as_deref(),
        Some("Personnel")
    );
}

/// AC2 — l'archivage (`DELETE`) ne touche pas la description.
#[sqlx::test(migrations = "./test-schema")]
async fn archive_for_company_conserve_la_description(pool: MySqlPool) {
    let company_id = create_test_company(&pool).await;
    let id = create_bank_account(&pool, company_id).await;
    set_description_sql(&pool, id, "Personnel").await;

    let mut tx = pool.begin().await.unwrap();
    let (after, _) = bank_accounts::archive_for_company(&mut tx, company_id, id, 1)
        .await
        .unwrap();
    tx.commit().await.unwrap();
    assert!(after.archived);
    assert_eq!(after.description.as_deref(), Some("Personnel"));
    assert_eq!(
        description_of(&pool, id).await.as_deref(),
        Some("Personnel")
    );
}

/// AC2 — `bank_accounts::create` (création par le dépôt) ne connaît pas la
/// description : le compte créé n'en a pas, et un compte décrit voisin garde
/// la sienne.
#[sqlx::test(migrations = "./test-schema")]
async fn create_ne_touche_pas_la_description(pool: MySqlPool) {
    let company_id = create_test_company(&pool).await;
    let voisin = create_bank_account(&pool, company_id).await;
    set_description_sql(&pool, voisin, "Personnel").await;

    let cree = bank_accounts::create(
        &pool,
        NewBankAccount {
            company_id,
            bank_name: "UBS".into(),
            iban: "CH5604835012345678009".into(),
            qr_iban: None,
            is_primary: false,
        },
    )
    .await
    .unwrap();
    assert_eq!(cree.description, None);
    assert_eq!(description_of(&pool, cree.id).await, None);
    assert_eq!(
        description_of(&pool, voisin).await.as_deref(),
        Some("Personnel")
    );
}

/// AC2 — la démotion de l'ancien principal, par appel **direct** de
/// `flip_primary_off_for_company` (cible de M12) : relu `is_primary = false`,
/// description intacte.
#[sqlx::test(migrations = "./test-schema")]
async fn flip_primary_off_conserve_la_description(pool: MySqlPool) {
    let company_id = create_test_company(&pool).await;
    let ancien = create_bank_account(&pool, company_id).await;
    set_description_sql(&pool, ancien, "Personnel").await;

    let mut tx = pool.begin().await.unwrap();
    let (updated, before) = bank_accounts::flip_primary_off_for_company(&mut tx, company_id, -1)
        .await
        .unwrap()
        .expect("un principal à démoter");
    tx.commit().await.unwrap();
    assert_eq!(before.id, ancien);
    assert!(!updated.is_primary);
    assert_eq!(updated.description.as_deref(), Some("Personnel"));
    let relu = bank_accounts::find_by_id_for_company(&pool, company_id, ancien)
        .await
        .unwrap()
        .unwrap();
    assert!(!relu.is_primary);
    assert_eq!(relu.description.as_deref(), Some("Personnel"));
}

// ===========================================================================
// Story 28-4a (#642) — le compte bancaire clos
// ===========================================================================

mod compte_clos {
    use super::*;
    use chrono::NaiveDate;
    use kesh_db::entities::{BankImportSourceFormat, NewBankImport, NewBankTransaction};
    use kesh_db::repositories::bank_accounts::{ClosureOutcome, ClosureRefusal};
    use kesh_db::repositories::bank_imports;

    fn ymd(y: i32, m: u32, d: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(y, m, d).unwrap()
    }

    /// Un compte bancaire non principal (IBAN distinct de `create_bank_account`).
    async fn compte_secondaire(pool: &MySqlPool, company_id: i64, iban: &str) -> i64 {
        bank_accounts::create(
            pool,
            NewBankAccount {
                company_id,
                bank_name: "PostFinance".into(),
                iban: iban.into(),
                qr_iban: None,
                is_primary: false,
            },
        )
        .await
        .unwrap()
        .id
    }

    /// Pose `closed_on` par SQL direct (montage).
    async fn clore_sql(pool: &MySqlPool, id: i64) {
        sqlx::query("UPDATE bank_accounts SET closed_on = '2026-01-31' WHERE id = ?")
            .bind(id)
            .execute(pool)
            .await
            .unwrap();
    }

    async fn version(pool: &MySqlPool, id: i64) -> i32 {
        sqlx::query_scalar("SELECT version FROM bank_accounts WHERE id = ?")
            .bind(id)
            .fetch_one(pool)
            .await
            .unwrap()
    }

    /// Un import d'une transaction `pending` sur `bank_account_id`, dans `tx`.
    async fn importer(
        tx: &mut sqlx::Transaction<'_, sqlx::MySql>,
        company_id: i64,
        bank_account_id: i64,
        user_id: i64,
        hash: &str,
    ) -> Result<(), DbError> {
        bank_imports::create_with_transactions(
            tx,
            NewBankImport {
                company_id,
                bank_account_id,
                filename: "releve.xml".into(),
                file_hash: format!("{hash:0>64}"),
                source_format: BankImportSourceFormat::Camt053V04,
                statement_id: Some("CLOS-1".into()),
                period_from: ymd(2026, 1, 1),
                period_to: ymd(2026, 1, 31),
                opening_balance: None,
                closing_balance: None,
                transaction_count: 1,
                imported_by_user_id: user_id,
            },
            vec![NewBankTransaction {
                company_id,
                bank_account_id,
                booking_date: ymd(2026, 1, 15),
                value_date: Some(ymd(2026, 1, 15)),
                amount: rust_decimal::Decimal::new(-1500, 2),
                currency: "CHF".into(),
                reference: None,
                details: "Frais".into(),
                end_to_end_id: None,
                transaction_id: None,
                counterparty_iban: None,
                counterparty_name: None,
            }],
        )
        .await
        .map(|_| ())
    }

    async fn compter(pool: &MySqlPool, sql: &str, id: i64) -> i64 {
        sqlx::query_scalar(sql)
            .bind(id)
            .fetch_one(pool)
            .await
            .unwrap()
    }

    /// T-a1 (AC1) — la contrainte `chk_bank_accounts_closed_not_primary` :
    /// un principal ne se clôt pas, un clos ne devient pas principal (SQL
    /// direct, sans passer par le dépôt).
    #[sqlx::test(migrations = "./test-schema")]
    async fn la_contrainte_interdit_un_principal_clos(pool: MySqlPool) {
        let company_id = create_test_company(&pool).await;
        let principal = create_bank_account(&pool, company_id).await;
        let err = sqlx::query("UPDATE bank_accounts SET closed_on = '2026-01-31' WHERE id = ?")
            .bind(principal)
            .execute(&pool)
            .await
            .expect_err("un principal ne se clôt pas");
        assert!(
            err.to_string()
                .contains("chk_bank_accounts_closed_not_primary"),
            "{err}"
        );
        let autre = compte_secondaire(&pool, company_id, "CH4431999123000889012").await;
        clore_sql(&pool, autre).await;
        let err = sqlx::query("UPDATE bank_accounts SET is_primary = TRUE WHERE id = ?")
            .bind(autre)
            .execute(&pool)
            .await
            .expect_err("un clos ne devient pas principal");
        assert!(
            err.to_string()
                .contains("chk_bank_accounts_closed_not_primary"),
            "{err}"
        );
    }

    /// T-a2 à T-a5 (AC2) — les quatre combinaisons du filtre de liste, sur un
    /// compte ouvert, un clos, un archivé et un archivé clos.
    #[sqlx::test(migrations = "./test-schema")]
    async fn le_filtre_de_liste_et_ses_quatre_combinaisons(pool: MySqlPool) {
        let company_id = create_test_company(&pool).await;
        let ouvert = create_bank_account(&pool, company_id).await;
        let clos = compte_secondaire(&pool, company_id, "CH4431999123000889012").await;
        clore_sql(&pool, clos).await;
        let archive =
            create_archived_bank_account(&pool, company_id, "CH5604835012345678009").await;
        let archive_clos =
            create_archived_bank_account(&pool, company_id, "CH9300762011623852957").await;
        clore_sql(&pool, archive_clos).await;

        let ids = |filter: BankAccountListFilter| {
            let pool = pool.clone();
            async move {
                let mut v: Vec<i64> = bank_accounts::list_by_company(&pool, company_id, filter)
                    .await
                    .unwrap()
                    .into_iter()
                    .map(|b| b.id)
                    .collect();
                v.sort_unstable();
                v
            }
        };
        let trie = |mut v: Vec<i64>| {
            v.sort_unstable();
            v
        };
        // T-a2 — défaut : ni archivés ni clos.
        assert_eq!(ids(BankAccountListFilter::default()).await, vec![ouvert]);
        // T-a3 — + clos non archivés.
        assert_eq!(
            ids(BankAccountListFilter {
                include_archived: false,
                include_closed: true
            })
            .await,
            trie(vec![ouvert, clos])
        );
        // T-a4 — + archivés, clos ou non.
        assert_eq!(
            ids(BankAccountListFilter {
                include_archived: true,
                include_closed: false
            })
            .await,
            trie(vec![ouvert, archive, archive_clos])
        );
        // T-a5 — tout.
        assert_eq!(
            ids(BankAccountListFilter::ALL).await,
            trie(vec![ouvert, clos, archive, archive_clos])
        );
        // `closed_on` hydraté par la macro `columns!`.
        let relu = bank_accounts::find_by_id_for_company(&pool, company_id, clos)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(relu.closed_on, Some(ymd(2026, 1, 31)));
    }

    /// T-a8 (AC2) — un compte clos ne compte pas parmi les « autres comptes
    /// actifs » : il n'empêche plus l'archivage du principal.
    #[sqlx::test(migrations = "./test-schema")]
    async fn un_compte_clos_n_empeche_pas_d_archiver_le_principal(pool: MySqlPool) {
        let company_id = create_test_company(&pool).await;
        let principal = create_bank_account(&pool, company_id).await;
        let autre = compte_secondaire(&pool, company_id, "CH4431999123000889012").await;
        let mut tx = pool.begin().await.unwrap();
        assert_eq!(
            bank_accounts::count_other_active_for_company(&mut tx, company_id, principal)
                .await
                .unwrap(),
            1
        );
        tx.rollback().await.unwrap();
        clore_sql(&pool, autre).await;
        let mut tx = pool.begin().await.unwrap();
        assert_eq!(
            bank_accounts::count_other_active_for_company(&mut tx, company_id, principal)
                .await
                .unwrap(),
            0,
            "le compte clos ne compte plus"
        );
    }

    /// T-a9 (AC2, C-28-4-7) — le compte comptable d'un compte clos reste « lié
    /// à une banque » : `first_active_bank_account_linked_to` le voit.
    #[sqlx::test(migrations = "./test-schema")]
    async fn le_compte_lie_d_un_compte_clos_reste_lie(pool: MySqlPool) {
        let company_id = create_test_company(&pool).await;
        let user_id = create_test_user(&pool, company_id, "u").await;
        let ledger = create_account(
            &pool,
            company_id,
            user_id,
            "1020",
            "Banque",
            AccountType::Asset,
        )
        .await;
        let clos = compte_secondaire(&pool, company_id, "CH4431999123000889012").await;
        sqlx::query("UPDATE bank_accounts SET journal_account_id = ? WHERE id = ?")
            .bind(ledger)
            .bind(clos)
            .execute(&pool)
            .await
            .unwrap();
        clore_sql(&pool, clos).await;
        let mut tx = pool.begin().await.unwrap();
        let lie = bank_accounts::first_active_bank_account_linked_to(&mut tx, company_id, ledger)
            .await
            .unwrap();
        assert_eq!(lie.map(|(id, _)| id), Some(clos));
    }

    /// Clôture et réouverture au dépôt : l'ordre des refus, et l'effet (rien
    /// d'autre que `closed_on` et `version`).
    #[sqlx::test(migrations = "./test-schema")]
    async fn cloture_et_reouverture_au_depot(pool: MySqlPool) {
        let company_id = create_test_company(&pool).await;
        let principal = create_bank_account(&pool, company_id).await;
        let autre = compte_secondaire(&pool, company_id, "CH4431999123000889012").await;

        // Le principal : refusé avant tout décompte.
        let mut tx = pool.begin().await.unwrap();
        let r = bank_accounts::close_in_tx(
            &mut tx,
            company_id,
            principal,
            version(&pool, principal).await,
            ymd(2026, 1, 31),
        )
        .await
        .unwrap();
        assert!(matches!(
            r,
            ClosureOutcome::Refused(ClosureRefusal::Primary)
        ));
        tx.rollback().await.unwrap();

        // Version périmée.
        let mut tx = pool.begin().await.unwrap();
        let err = bank_accounts::close_in_tx(&mut tx, company_id, autre, 99, ymd(2026, 1, 31))
            .await
            .expect_err("version");
        assert!(matches!(err, DbError::OptimisticLockConflict));
        tx.rollback().await.unwrap();

        // Rouvrir un compte ouvert.
        let mut tx = pool.begin().await.unwrap();
        let r =
            bank_accounts::reopen_in_tx(&mut tx, company_id, autre, version(&pool, autre).await)
                .await
                .unwrap();
        assert!(matches!(
            r,
            ClosureOutcome::Refused(ClosureRefusal::NotClosed)
        ));
        tx.rollback().await.unwrap();

        // Clore.
        let v = version(&pool, autre).await;
        let mut tx = pool.begin().await.unwrap();
        let ClosureOutcome::Done { after, before } =
            bank_accounts::close_in_tx(&mut tx, company_id, autre, v, ymd(2026, 1, 31))
                .await
                .unwrap()
        else {
            panic!("clôture attendue");
        };
        tx.commit().await.unwrap();
        assert_eq!(before.closed_on, None);
        assert_eq!(after.closed_on, Some(ymd(2026, 1, 31)));
        assert_eq!(after.version, v + 1);
        assert!(!after.archived && !after.is_primary);
        assert_eq!(after.journal_account_id, before.journal_account_id);

        // Déjà clos.
        let mut tx = pool.begin().await.unwrap();
        let err = bank_accounts::close_in_tx(&mut tx, company_id, autre, v + 1, ymd(2026, 2, 1))
            .await
            .expect_err("déjà clos");
        assert!(matches!(
            err,
            DbError::BankAccountClosed { bank_account_id } if bank_account_id == autre
        ));
        tx.rollback().await.unwrap();

        // Rouvrir : `closed_on` remis à NULL, le compte ne devient pas principal.
        let mut tx = pool.begin().await.unwrap();
        let ClosureOutcome::Done { after, .. } =
            bank_accounts::reopen_in_tx(&mut tx, company_id, autre, v + 1)
                .await
                .unwrap()
        else {
            panic!("réouverture attendue");
        };
        tx.commit().await.unwrap();
        assert_eq!(after.closed_on, None);
        assert!(!after.is_primary);
        assert_eq!(after.version, v + 2);
    }

    /// Les décomptes de la clôture : une transaction `pending` (rejetée
    /// comprise) et un lot `generated` la refusent.
    #[sqlx::test(migrations = "./test-schema")]
    async fn la_cloture_refuse_les_pending_et_les_lots_ouverts(pool: MySqlPool) {
        let company_id = create_test_company(&pool).await;
        let user_id = create_test_user(&pool, company_id, "u").await;
        let autre = compte_secondaire(&pool, company_id, "CH4431999123000889012").await;
        let mut tx = pool.begin().await.unwrap();
        importer(&mut tx, company_id, autre, user_id, "a")
            .await
            .unwrap();
        tx.commit().await.unwrap();
        // Rejetée : toujours `pending`.
        sqlx::query("UPDATE bank_transactions SET auto_match_rejected_at = NOW(3)")
            .execute(&pool)
            .await
            .unwrap();
        let mut tx = pool.begin().await.unwrap();
        let r = bank_accounts::close_in_tx(
            &mut tx,
            company_id,
            autre,
            version(&pool, autre).await,
            ymd(2026, 1, 31),
        )
        .await
        .unwrap();
        assert!(matches!(
            r,
            ClosureOutcome::Refused(ClosureRefusal::PendingTransactions { pending_count: 1 })
        ));
        tx.rollback().await.unwrap();

        sqlx::query("UPDATE bank_transactions SET status = 'reconciled'")
            .execute(&pool)
            .await
            .unwrap();
        sqlx::query(
            "INSERT INTO payment_batches (company_id, bank_account_id, status, \
             requested_execution_date, msg_id, payment_info_id) \
             VALUES (?, ?, 'generated', '2026-02-01', 'M', 'P')",
        )
        .bind(company_id)
        .bind(autre)
        .execute(&pool)
        .await
        .unwrap();
        let mut tx = pool.begin().await.unwrap();
        let r = bank_accounts::close_in_tx(
            &mut tx,
            company_id,
            autre,
            version(&pool, autre).await,
            ymd(2026, 1, 31),
        )
        .await
        .unwrap();
        assert!(matches!(
            r,
            ClosureOutcome::Refused(ClosureRefusal::OpenPaymentBatches { batch_count: 1 })
        ));
    }

    /// T-a46 (AC6-bis) — l'import, au dépôt : un compte **archivé** rend
    /// `NotFound`, un compte **clos** `BankAccountClosed` ; aucune ligne écrite.
    #[sqlx::test(migrations = "./test-schema")]
    async fn l_import_refuse_l_archive_et_le_clos_au_depot(pool: MySqlPool) {
        let company_id = create_test_company(&pool).await;
        let user_id = create_test_user(&pool, company_id, "u").await;
        let archive =
            create_archived_bank_account(&pool, company_id, "CH5604835012345678009").await;
        let clos = compte_secondaire(&pool, company_id, "CH4431999123000889012").await;
        clore_sql(&pool, clos).await;

        let mut tx = pool.begin().await.unwrap();
        let err = importer(&mut tx, company_id, archive, user_id, "a")
            .await
            .expect_err("archivé");
        assert!(matches!(err, DbError::NotFound), "{err:?}");
        tx.rollback().await.unwrap();

        let mut tx = pool.begin().await.unwrap();
        let err = importer(&mut tx, company_id, clos, user_id, "b")
            .await
            .expect_err("clos");
        assert!(
            matches!(err, DbError::BankAccountClosed { bank_account_id } if bank_account_id == clos),
            "{err:?}"
        );
        tx.rollback().await.unwrap();

        for table in ["bank_imports", "bank_transactions"] {
            let n: i64 = sqlx::query_scalar(&format!("SELECT COUNT(*) FROM {table}"))
                .fetch_one(&pool)
                .await
                .unwrap();
            assert_eq!(n, 0, "{table}");
        }
    }

    /// T-a31 (AC7, premier volet) — **déterministe** : un import a inséré son
    /// entête et sa transaction `pending` sans commiter ; il tient, par la clé
    /// étrangère, un verrou partagé sur la ligne du compte. La clôture lancée
    /// en parallèle **attend** sur sa première instruction (`FOR UPDATE`), puis,
    /// au commit de l'import, ses décomptes — lus APRÈS le verrou — voient la
    /// transaction et refusent. Si la prémisse InnoDB (verrou partagé du parent
    /// à l'`INSERT` d'un enfant) était fausse, la sonde paniquerait au bout de
    /// dix secondes : le test la vérifie aussi. Tue M6 (décompte lu avant le
    /// verrou : la vue naîtrait avant l'attente et ne verrait pas l'import).
    #[sqlx::test(migrations = "./test-schema")]
    async fn une_cloture_attend_un_import_en_cours_puis_le_voit(pool: MySqlPool) {
        let company_id = create_test_company(&pool).await;
        let user_id = create_test_user(&pool, company_id, "u").await;
        let autre = compte_secondaire(&pool, company_id, "CH4431999123000889012").await;
        let v = version(&pool, autre).await;

        let mut import = pool.begin().await.unwrap();
        importer(&mut import, company_id, autre, user_id, "a")
            .await
            .unwrap();

        let p = pool.clone();
        let cloture = tokio::spawn(async move {
            let mut tx = p.begin().await.unwrap();
            let r =
                bank_accounts::close_in_tx(&mut tx, company_id, autre, v, ymd(2026, 1, 31)).await;
            if matches!(r, Ok(ClosureOutcome::Done { .. })) {
                tx.commit().await.unwrap();
            }
            r
        });
        let vue = kesh_db::test_fixtures::attendre_une_requete_en_cours(
            &pool,
            &["FROM bank_accounts", "archived = FALSE FOR UPDATE"],
            || cloture.is_finished(),
        )
        .await;
        if !vue {
            panic!(
                "la clôture a fini sans attendre l'import : {:?}",
                cloture.await.map(|r| r.map(|_| ()))
            );
        }
        import.commit().await.unwrap();

        let r = cloture.await.expect("tâche de clôture").expect("clôture");
        assert!(
            matches!(
                r,
                ClosureOutcome::Refused(ClosureRefusal::PendingTransactions { pending_count: 1 })
            ),
            "la clôture devait voir l'import commité — reçu {r:?}"
        );
        assert_eq!(
            compter(
                &pool,
                "SELECT COUNT(*) FROM bank_accounts WHERE id = ? AND closed_on IS NULL",
                autre
            )
            .await,
            1,
            "le compte reste ouvert"
        );
    }
}
