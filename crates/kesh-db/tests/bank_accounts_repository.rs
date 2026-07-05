//! Tests d'intégration pour `repositories::bank_accounts`.

use kesh_db::entities::account::AccountType;
use kesh_db::entities::address::StructuredAddress;
use kesh_db::entities::{Language, NewAccount, NewBankAccount, NewCompany, OrgType};
use kesh_db::errors::DbError;
use kesh_db::repositories::{accounts, bank_accounts, companies};
use sqlx::MySqlPool;

async fn create_test_company(pool: &MySqlPool) -> i64 {
    companies::create(
        pool,
        NewCompany {
            name: "Test SA".into(),
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

    let list = bank_accounts::list_by_company(&pool, company_id, /*include_archived=*/ false)
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
    let list = bank_accounts::list_by_company(&pool, company_id, /*include_archived=*/ false)
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
    let (updated, before) = bank_accounts::set_journal_account_id_for_company(
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
    let (linked, _before) = bank_accounts::set_journal_account_id_for_company(
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
    let (unlinked, _before) = bank_accounts::set_journal_account_id_for_company(
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
    let _ = bank_accounts::set_journal_account_id_for_company(
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
    let (linked, _before) = bank_accounts::set_journal_account_id_for_company(
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
    let (unchanged, before) = bank_accounts::set_journal_account_id_for_company(
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
#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
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
#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
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

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
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

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
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

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
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
    let active_only = bank_accounts::list_by_company(&pool, company_id, false)
        .await
        .unwrap();
    assert_eq!(active_only.len(), 1);
    assert!(!active_only[0].archived);

    // include_archived=true (export ZIP souveraineté) : retourne tout.
    let all = bank_accounts::list_by_company(&pool, company_id, true)
        .await
        .unwrap();
    assert_eq!(all.len(), 2);
}

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
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

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
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
    let result =
        bank_accounts::update_for_company(&mut tx, company_id, archived_id, &new, None, 1).await;
    let _ = tx.rollback().await;

    assert!(
        matches!(result, Err(DbError::NotFound)),
        "update_for_company sur compte archivé doit retourner NotFound (F6 Pass 3 Opus), got: {result:?}"
    );
}

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
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
