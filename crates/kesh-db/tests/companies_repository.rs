//! Tests d'intégration pour `repositories::companies`.
//!
//! Utilise `#[sqlx::test]` qui crée une base de données temporaire par test
//! (cloné depuis `DATABASE_URL`) et applique le migrator fourni. Nécessite
//! que l'utilisateur DB ait les droits `CREATE DATABASE` et `DROP DATABASE`.

use kesh_db::entities::address::StructuredAddress;
use kesh_db::entities::{CompanyUpdate, Language, NewCompany, OrgType};
use kesh_db::errors::DbError;
use kesh_db::repositories::companies;
use sqlx::MySqlPool;

fn sample_new_company() -> NewCompany {
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
        ide_number: Some("CHE109322551".into()),
        org_type: OrgType::Pme,
        accounting_language: Language::Fr,
        instance_language: Language::Fr,
    }
}

#[sqlx::test(migrations = "./test-schema")]
async fn create_and_find_by_id(pool: MySqlPool) {
    let created = companies::create(&pool, sample_new_company())
        .await
        .expect("create should succeed");
    assert!(created.id > 0);
    assert_eq!(created.name, "Test SA");
    assert_eq!(created.version, 1);

    let found = companies::find_by_id(&pool, created.id)
        .await
        .expect("find should succeed")
        .expect("company should exist");
    assert_eq!(found.id, created.id);
    assert_eq!(found.name, "Test SA");
    assert_eq!(found.org_type, OrgType::Pme);
    assert_eq!(found.accounting_language, Language::Fr);
}

#[sqlx::test(migrations = "./test-schema")]
async fn find_by_id_returns_none_for_missing(pool: MySqlPool) {
    let result = companies::find_by_id(&pool, 999_999).await.unwrap();
    assert!(result.is_none());
}

#[sqlx::test(migrations = "./test-schema")]
async fn update_succeeds_with_current_version(pool: MySqlPool) {
    let created = companies::create(&pool, sample_new_company())
        .await
        .unwrap();

    let changes = CompanyUpdate {
        name: "Test SA (renamed)".into(),
        first_name: None,
        last_name: None,
        address_structured: created.structured_address(),
        ide_number: created.ide_number.clone(),
        org_type: created.org_type,
        accounting_language: Language::De,
        instance_language: created.instance_language,
        email: None,
        phone: None,
        website: None,
    };

    let updated = companies::update(&pool, created.id, created.version, changes)
        .await
        .expect("update should succeed");

    assert_eq!(updated.name, "Test SA (renamed)");
    assert_eq!(updated.accounting_language, Language::De);
    assert_eq!(updated.version, created.version + 1);
}

#[sqlx::test(migrations = "./test-schema")]
async fn update_fails_on_stale_version(pool: MySqlPool) {
    let created = companies::create(&pool, sample_new_company())
        .await
        .unwrap();

    // Premier update : version 1 → 2
    let changes = CompanyUpdate {
        name: "First update".into(),
        first_name: None,
        last_name: None,
        address_structured: created.structured_address(),
        ide_number: created.ide_number.clone(),
        org_type: created.org_type,
        accounting_language: created.accounting_language,
        instance_language: created.instance_language,
        email: None,
        phone: None,
        website: None,
    };
    companies::update(&pool, created.id, 1, changes)
        .await
        .unwrap();

    // Deuxième update avec version 1 stale → conflict
    let stale_changes = CompanyUpdate {
        name: "Stale update".into(),
        first_name: None,
        last_name: None,
        address_structured: created.structured_address(),
        ide_number: created.ide_number.clone(),
        org_type: created.org_type,
        accounting_language: created.accounting_language,
        instance_language: created.instance_language,
        email: None,
        phone: None,
        website: None,
    };
    let result = companies::update(&pool, created.id, 1, stale_changes).await;
    assert!(matches!(result, Err(DbError::OptimisticLockConflict)));
}

#[sqlx::test(migrations = "./test-schema")]
async fn update_fails_on_missing_entity(pool: MySqlPool) {
    let changes = CompanyUpdate {
        name: "Ghost".into(),
        first_name: None,
        last_name: None,
        address_structured: StructuredAddress {
            street: "Nowhere".into(),
            building: String::new(),
            postal_code: "1000".into(),
            city: "Lausanne".into(),
            country: "CH".into(),
        },
        ide_number: None,
        org_type: OrgType::Pme,
        accounting_language: Language::Fr,
        instance_language: Language::Fr,
        email: None,
        phone: None,
        website: None,
    };
    let result = companies::update(&pool, 999_999, 1, changes).await;
    assert!(matches!(result, Err(DbError::NotFound)));
}

#[sqlx::test(migrations = "./test-schema")]
async fn list_with_pagination(pool: MySqlPool) {
    // Créer 5 companies
    for i in 0..5 {
        let mut new = sample_new_company();
        new.name = format!("Company {i}");
        new.ide_number = Some(format!("CHE10932255{i}")); // unique par company, format CHE+9 chiffres
        // Note : le DB valide le format REGEXP '^CHE[0-9]{9}$' mais pas le checksum
        // métier — la validation métier `CheNumber` vit dans kesh-core (story 1.3).
        companies::create(&pool, new).await.unwrap();
    }

    let page1 = companies::list(&pool, 2, 0).await.unwrap();
    assert_eq!(page1.len(), 2);

    let page2 = companies::list(&pool, 2, 2).await.unwrap();
    assert_eq!(page2.len(), 2);

    let page3 = companies::list(&pool, 2, 4).await.unwrap();
    assert_eq!(page3.len(), 1);

    let empty = companies::list(&pool, 2, 100).await.unwrap();
    assert_eq!(empty.len(), 0);

    // Vérifier l'ordre stable (par id ASC)
    assert!(page1[0].id < page1[1].id);
}

#[sqlx::test(migrations = "./test-schema")]
async fn unique_constraint_on_ide_number(pool: MySqlPool) {
    companies::create(&pool, sample_new_company())
        .await
        .unwrap();

    // Tentative de créer une seconde company avec le même IDE
    let result = companies::create(&pool, sample_new_company()).await;
    assert!(matches!(result, Err(DbError::UniqueConstraintViolation(_))));
}

#[sqlx::test(migrations = "./test-schema")]
async fn empty_name_rejected(pool: MySqlPool) {
    let mut new = sample_new_company();
    new.name = String::new();
    let result = companies::create(&pool, new).await;
    assert!(matches!(result, Err(DbError::CheckConstraintViolation(_))));
}

#[sqlx::test(migrations = "./test-schema")]
async fn empty_address_rejected(pool: MySqlPool) {
    let mut new = sample_new_company();
    new.address_structured = StructuredAddress {
        street: "   ".into(),
        building: String::new(),
        postal_code: String::new(),
        city: String::new(),
        country: "CH".into(),
    };
    let result = companies::create(&pool, new).await;
    assert!(matches!(result, Err(DbError::CheckConstraintViolation(_))));
}

#[sqlx::test(migrations = "./test-schema")]
async fn invalid_ide_format_rejected(pool: MySqlPool) {
    let mut new = sample_new_company();
    new.ide_number = Some("INVALID".into());
    let result = companies::create(&pool, new).await;
    assert!(matches!(result, Err(DbError::CheckConstraintViolation(_))));
}

#[sqlx::test(migrations = "./test-schema")]
async fn list_limit_clamped_to_max(pool: MySqlPool) {
    // Un limit très grand (i64::MAX) doit être clampé à MAX_LIST_LIMIT sans
    // provoquer d'erreur SQL — validation du clamp pre-query.
    companies::create(&pool, sample_new_company())
        .await
        .unwrap();
    let list = companies::list(&pool, i64::MAX, 0).await.unwrap();
    assert_eq!(list.len(), 1);

    // Test complémentaire : i64::MIN aussi
    let list_min = companies::list(&pool, i64::MIN, 0).await.unwrap();
    assert!(list_min.is_empty()); // limit clamped à 0
}

#[sqlx::test(migrations = "./test-schema")]
async fn list_negative_values_normalized(pool: MySqlPool) {
    companies::create(&pool, sample_new_company())
        .await
        .unwrap();
    // Limite négative → clamped à 0 → liste vide
    let empty = companies::list(&pool, -5, 0).await.unwrap();
    assert!(empty.is_empty());

    // Offset négatif → clamped à 0, limite valide → retourne les résultats
    let list = companies::list(&pool, 10, -10).await.unwrap();
    assert_eq!(list.len(), 1);
}

/// KF-004 : payload identique à l'état persisté → pas de bump version,
/// `updated_at` inchangé. Pas d'assertion audit_log : `companies::update`
/// n'écrit pas d'audit log v0.1.
#[sqlx::test(migrations = "./test-schema")]
async fn update_no_op_returns_unchanged_entity(pool: MySqlPool) {
    let created = companies::create(&pool, sample_new_company())
        .await
        .unwrap();
    let version_initial = created.version;
    let updated_at_initial = created.updated_at;

    let identical = CompanyUpdate {
        name: created.name.clone(),
        first_name: None,
        last_name: None,
        address_structured: created.structured_address(),
        ide_number: created.ide_number.clone(),
        org_type: created.org_type,
        accounting_language: created.accounting_language,
        instance_language: created.instance_language,
        email: None,
        phone: None,
        website: None,
    };

    let result = companies::update(&pool, created.id, version_initial, identical)
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
    assert_eq!(result.name, created.name);
}

/// KF-004 régression : modifier `name` → bump version.
#[sqlx::test(migrations = "./test-schema")]
async fn update_partial_change_bumps_version(pool: MySqlPool) {
    let created = companies::create(&pool, sample_new_company())
        .await
        .unwrap();
    let version_initial = created.version;

    let changes = CompanyUpdate {
        name: "Test SA Renommée".into(),
        first_name: None,
        last_name: None,
        address_structured: created.structured_address(),
        ide_number: created.ide_number.clone(),
        org_type: created.org_type,
        accounting_language: created.accounting_language,
        instance_language: created.instance_language,
        email: None,
        phone: None,
        website: None,
    };
    let result = companies::update(&pool, created.id, version_initial, changes)
        .await
        .unwrap();
    assert_eq!(result.version, version_initial + 1);
    assert_eq!(result.name, "Test SA Renommée");
}

#[sqlx::test(migrations = "./test-schema")]
async fn multiple_companies_without_ide(pool: MySqlPool) {
    // Plusieurs companies sans IDE (NULL) doivent être acceptées
    // — UNIQUE n'applique pas aux NULL en MariaDB.
    let mut c1 = sample_new_company();
    c1.ide_number = None;
    c1.name = "Company A".into();
    companies::create(&pool, c1).await.unwrap();

    let mut c2 = sample_new_company();
    c2.ide_number = None;
    c2.name = "Company B".into();
    companies::create(&pool, c2).await.unwrap();
}

/// Story 16-3a, passe 6 de revue — **une société antérieure à la migration
/// `structured_addresses` (#213, v0.5.0) doit pouvoir être mise à jour.**
///
/// Ces sociétés-là ont leurs quatre colonnes structurées à `''` : la migration
/// les a ajoutées en `NOT NULL DEFAULT ''` **sans backfill**, et aucune
/// migration du dépôt ne fait `UPDATE companies`. Leur adresse ne vit que dans
/// la colonne `address`, en texte libre.
///
/// Toute route en full-replace — `update_company_email` (20-3b1) comme
/// `update_company_contact_details` (16-3a) — reconstruit `CompanyUpdate` par
/// `company.structured_address()`. Sur ces lignes, `combined()` rend `""`, et
/// sans garde l'`UPDATE` écrit `address = ''`, que
/// `chk_companies_address_nonempty` rejette : **500** en voulant simplement
/// renseigner un téléphone.
///
/// ⚠️ Le montage vide les colonnes structurées **en SQL direct** : aucune
/// fixture du dépôt ne produit cet état — `test_fixtures.rs` les peuple
/// toujours — et c'est précisément pourquoi aucun gate ne voyait le défaut.
#[sqlx::test(migrations = "./test-schema")]
async fn update_preserves_address_when_structured_columns_are_empty(pool: MySqlPool) {
    let created = companies::create(&pool, sample_new_company())
        .await
        .unwrap();

    // Ramener la ligne à l'état d'une société pré-#213 : adresse en texte
    // libre, colonnes structurées vides.
    sqlx::query(
        "UPDATE companies SET address = ?, address_street = '', address_building = '',
         address_postal_code = '', address_city = '' WHERE id = ?",
    )
    .bind("Ancienne Rue 3\n1200 Genève")
    .bind(created.id)
    .execute(&pool)
    .await
    .expect("le montage doit pouvoir simuler une société pré-#213");

    let before = companies::find_by_id(&pool, created.id)
        .await
        .unwrap()
        .expect("la société doit exister");
    assert_eq!(
        before.structured_address().combined(),
        "",
        "montage invalide : les colonnes structurées doivent être vides, \
         sans quoi ce test ne mesure PAS le cas qu'il prétend couvrir"
    );

    // Le geste de l'utilisateur : renseigner son téléphone, rien d'autre.
    let changes = CompanyUpdate {
        name: before.name.clone(),
        first_name: before.first_name.clone(),
        last_name: before.last_name.clone(),
        address_structured: before.structured_address(),
        ide_number: before.ide_number.clone(),
        org_type: before.org_type,
        accounting_language: before.accounting_language,
        instance_language: before.instance_language,
        email: before.email.clone(),
        phone: Some("+41 21 123 45 67".into()),
        website: before.website.clone(),
    };

    let updated = companies::update(&pool, before.id, before.version, changes)
        .await
        .expect("renseigner un téléphone ne doit pas échouer sur une société pré-#213");

    assert_eq!(
        updated.phone.as_deref(),
        Some("+41 21 123 45 67"),
        "le téléphone doit avoir été écrit"
    );
    assert_eq!(
        updated.address, "Ancienne Rue 3\n1200 Genève",
        "l'adresse en texte libre doit être PRÉSERVÉE, ni vidée ni recomposée"
    );
}

/// Le pendant : quand les colonnes structurées SONT renseignées, `address`
/// reste bien dérivée d'elles. Sans ce test, remplacer la garde par un
/// « ne jamais toucher à `address` » passerait inaperçu.
#[sqlx::test(migrations = "./test-schema")]
async fn update_recomposes_address_when_structured_columns_are_filled(pool: MySqlPool) {
    let created = companies::create(&pool, sample_new_company())
        .await
        .unwrap();

    let changes = CompanyUpdate {
        name: created.name.clone(),
        first_name: None,
        last_name: None,
        address_structured: StructuredAddress {
            street: "Avenue Neuve".into(),
            building: "12".into(),
            postal_code: "1204".into(),
            city: "Genève".into(),
            country: "CH".into(),
        },
        ide_number: created.ide_number.clone(),
        org_type: created.org_type,
        accounting_language: created.accounting_language,
        instance_language: created.instance_language,
        email: None,
        phone: None,
        website: None,
    };

    let updated = companies::update(&pool, created.id, created.version, changes)
        .await
        .unwrap();

    assert_eq!(
        updated.address, "Avenue Neuve 12\n1204 Genève",
        "adresse structurée renseignée → `address` doit être recomposée depuis elle"
    );
}

async fn stub_flag_and_version(pool: &MySqlPool, id: i64) -> (bool, i32) {
    sqlx::query_as("SELECT is_stub, version FROM companies WHERE id = ?")
        .bind(id)
        .fetch_one(pool)
        .await
        .unwrap()
}

/// Test 6 (Story 15-7a1, AC 6) — `clear_stub_in_tx` lève le drapeau d'une
/// société provisoire et bumpe `version` ; rien sur un second appel ni sur une
/// société non provisoire.
#[sqlx::test(migrations = "./test-schema")]
async fn clear_stub_in_tx_clears_only_a_stub_and_bumps_version(pool: MySqlPool) {
    let stub = companies::create(&pool, sample_new_company())
        .await
        .unwrap()
        .id;
    // Aucune fonction de kesh-db n'insère de stub : montage en SQL brut.
    sqlx::query("UPDATE companies SET is_stub = TRUE WHERE id = ?")
        .bind(stub)
        .execute(&pool)
        .await
        .unwrap();
    let mut other_company = sample_new_company();
    other_company.name = "Non provisoire SA".into();
    other_company.ide_number = None;
    let regular = companies::create(&pool, other_company).await.unwrap().id;

    let (flag, v0) = stub_flag_and_version(&pool, stub).await;
    assert!(flag);

    let mut tx = pool.begin().await.unwrap();
    assert!(companies::clear_stub_in_tx(&mut tx, stub).await.unwrap());
    tx.commit().await.unwrap();
    assert_eq!(stub_flag_and_version(&pool, stub).await, (false, v0 + 1));

    let mut tx = pool.begin().await.unwrap();
    assert!(!companies::clear_stub_in_tx(&mut tx, stub).await.unwrap());
    tx.commit().await.unwrap();
    assert_eq!(
        stub_flag_and_version(&pool, stub).await,
        (false, v0 + 1),
        "second appel : version inchangée"
    );

    let before = stub_flag_and_version(&pool, regular).await;
    let mut tx = pool.begin().await.unwrap();
    assert!(!companies::clear_stub_in_tx(&mut tx, regular).await.unwrap());
    tx.commit().await.unwrap();
    assert_eq!(stub_flag_and_version(&pool, regular).await, before);
}

// ===========================================================================
// Story 15-7b2 — la règle des principaux orphelins (AC 4, test 6f)
// ===========================================================================

/// Société inexistante que désignent les principaux orphelins du montage.
const DEAD_COMPANY: i64 = 987_654;

/// Les ids du montage : société vivante, utilisateur sain, utilisateur
/// orphelin, clé active orpheline, clé déjà révoquée orpheline, clé saine.
struct Orphans {
    company: i64,
    orphan_user: i64,
    active_orphan_key: i64,
    revoked_orphan_key: i64,
    sane_key: i64,
}

/// Montage sur une connexion **détachée du pool** (`FOREIGN_KEY_CHECKS=0`
/// n'y retourne jamais) : un utilisateur et deux clés désignent
/// `DEAD_COMPANY` — l'état qu'a laissé la remise à zéro v0.12.x (#528).
async fn mount_orphans(pool: &MySqlPool) -> Orphans {
    use sqlx::Connection;
    let company = companies::create(pool, sample_new_company())
        .await
        .unwrap()
        .id;
    let mut conn = pool.acquire().await.unwrap().detach();
    sqlx::query("SET FOREIGN_KEY_CHECKS = 0")
        .execute(&mut conn)
        .await
        .unwrap();
    let insert_user = |name: &'static str, company_id: i64| {
        sqlx::query(
            "INSERT INTO users (username, password_hash, role, active, company_id) \
             VALUES (?, 'argon2id-hash-placeholder-long-enough', 'Admin', TRUE, ?)",
        )
        .bind(name)
        .bind(company_id)
    };
    let sane_user = insert_user("sain", company)
        .execute(&mut conn)
        .await
        .unwrap()
        .last_insert_id() as i64;
    let orphan_user = insert_user("orphelin", DEAD_COMPANY)
        .execute(&mut conn)
        .await
        .unwrap()
        .last_insert_id() as i64;
    let mut insert_key = async |name: &str, company_id: i64, revoked: bool| -> i64 {
        sqlx::query(
            "INSERT INTO api_keys (company_id, created_by_user_id, name, key_hash, scope, \
                                   last_used_at, revoked_at, version) \
             VALUES (?, ?, ?, SHA2(?, 256), 'read-write', '2026-01-02 03:04:05.678', \
                     IF(?, NOW(3), NULL), IF(?, 2, 1))",
        )
        .bind(company_id)
        .bind(sane_user)
        .bind(name)
        .bind(name)
        .bind(revoked)
        .bind(revoked)
        .execute(&mut conn)
        .await
        .unwrap()
        .last_insert_id() as i64
    };
    let active_orphan_key = insert_key("active orpheline", DEAD_COMPANY, false).await;
    let revoked_orphan_key = insert_key("révoquée orpheline", DEAD_COMPANY, true).await;
    let sane_key = insert_key("saine", company, false).await;
    conn.close().await.unwrap();
    Orphans {
        company,
        orphan_user,
        active_orphan_key,
        revoked_orphan_key,
        sane_key,
    }
}

/// `(company_id, revoked_at IS NOT NULL, version)` d'une clé.
async fn key_row(pool: &MySqlPool, id: i64) -> (i64, bool, i32) {
    sqlx::query_as("SELECT company_id, revoked_at IS NOT NULL, version FROM api_keys WHERE id = ?")
        .bind(id)
        .fetch_one(pool)
        .await
        .unwrap()
}

async fn user_company(pool: &MySqlPool, id: i64) -> i64 {
    sqlx::query_scalar("SELECT company_id FROM users WHERE id = ?")
        .bind(id)
        .fetch_one(pool)
        .await
        .unwrap()
}

/// Appelle la règle dans une transaction qui tient `companies` (pré-condition),
/// puis commite.
async fn reattach(pool: &MySqlPool, target: Option<i64>) -> companies::OrphanPrincipals {
    let mut tx = pool.begin().await.unwrap();
    sqlx::query("SELECT id FROM companies ORDER BY id FOR UPDATE")
        .fetch_all(&mut *tx)
        .await
        .unwrap();
    let out = companies::reattach_orphan_principals_in_tx(&mut tx, target)
        .await
        .unwrap();
    tx.commit().await.unwrap();
    out
}

/// Test 6f (i) — avec une cible : l'utilisateur orphelin est rattaché ; la clé
/// active orpheline est **révoquée** (`version` + 1) **puis** repointée ; la
/// clé déjà révoquée est repointée, `revoked_at` et `version` inchangés ; la
/// clé saine est intacte. `api_keys_revoked` dit la seule clé active
/// orpheline, nom et dates exacts.
#[sqlx::test(migrations = "./test-schema")]
async fn reattach_orphans_with_a_target_revokes_then_repoints(pool: MySqlPool) {
    let m = mount_orphans(&pool).await;
    let sane_before = key_row(&pool, m.sane_key).await;
    let revoked_before = key_row(&pool, m.revoked_orphan_key).await;
    assert_eq!(revoked_before, (DEAD_COMPANY, true, 2), "montage");
    let (created_at, last_used_at): (chrono::NaiveDateTime, Option<chrono::NaiveDateTime>) =
        sqlx::query_as("SELECT created_at, last_used_at FROM api_keys WHERE id = ?")
            .bind(m.active_orphan_key)
            .fetch_one(&pool)
            .await
            .unwrap();
    let created_by: i64 =
        sqlx::query_scalar("SELECT created_by_user_id FROM api_keys WHERE id = ?")
            .bind(m.active_orphan_key)
            .fetch_one(&pool)
            .await
            .unwrap();

    let out = reattach(&pool, Some(m.company)).await;

    assert_eq!(out.user_ids, vec![m.orphan_user]);
    assert_eq!(user_company(&pool, m.orphan_user).await, m.company);
    assert_eq!(
        out.api_keys_revoked,
        vec![companies::RevokedApiKey {
            id: m.active_orphan_key,
            name: "active orpheline".into(),
            created_by_user_id: created_by,
            created_at,
            last_used_at,
        }]
    );
    assert!(last_used_at.is_some(), "montage : date d'usage posée");
    assert_eq!(out.api_keys_repointed, 2);
    assert_eq!(
        key_row(&pool, m.active_orphan_key).await,
        (m.company, true, 2),
        "clé active orpheline : révoquée, version + 1, repointée"
    );
    assert_eq!(
        key_row(&pool, m.revoked_orphan_key).await,
        (m.company, true, 2),
        "clé déjà révoquée : repointée, version inchangée"
    );
    assert_eq!(
        key_row(&pool, m.sane_key).await,
        sane_before,
        "clé saine intacte"
    );
}

/// Test 6f (ii) — sans cible : la clé active orpheline est révoquée et **non**
/// repointée ; l'utilisateur n'est pas rattaché.
#[sqlx::test(migrations = "./test-schema")]
async fn reattach_orphans_without_target_only_revokes(pool: MySqlPool) {
    let m = mount_orphans(&pool).await;
    let out = reattach(&pool, None).await;
    assert!(out.user_ids.is_empty());
    assert_eq!(out.api_keys_repointed, 0);
    assert_eq!(
        out.api_keys_revoked
            .iter()
            .map(|k| k.id)
            .collect::<Vec<_>>(),
        vec![m.active_orphan_key]
    );
    assert_eq!(
        key_row(&pool, m.active_orphan_key).await,
        (DEAD_COMPANY, true, 2)
    );
    assert_eq!(user_company(&pool, m.orphan_user).await, DEAD_COMPANY);
}

/// Test 6f (iii) — aucun orphelin : `is_empty()`, et rien n'est écrit.
#[sqlx::test(migrations = "./test-schema")]
async fn reattach_without_orphans_writes_nothing(pool: MySqlPool) {
    let m = mount_orphans(&pool).await;
    // On rattache d'abord, puis on rejoue : plus aucun orphelin.
    reattach(&pool, Some(m.company)).await;
    let snapshot = |pool: MySqlPool| async move {
        sqlx::query_as::<_, (i64, i64, i32, Option<chrono::NaiveDateTime>)>(
            "SELECT id, company_id, version, revoked_at FROM api_keys ORDER BY id",
        )
        .fetch_all(&pool)
        .await
        .unwrap()
    };
    let before = snapshot(pool.clone()).await;
    let out = reattach(&pool, Some(m.company)).await;
    assert!(out.is_empty(), "{out:?}");
    assert_eq!(snapshot(pool.clone()).await, before);
    assert_eq!(user_company(&pool, m.orphan_user).await, m.company);
}
