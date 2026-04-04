//! Tests d'intégration pour `repositories::companies`.
//!
//! Utilise `#[sqlx::test]` qui crée une base de données temporaire par test
//! (cloné depuis `DATABASE_URL`) et applique le migrator fourni. Nécessite
//! que l'utilisateur DB ait les droits `CREATE DATABASE` et `DROP DATABASE`.

use kesh_db::entities::{CompanyUpdate, Language, NewCompany, OrgType};
use kesh_db::errors::DbError;
use kesh_db::repositories::companies;
use sqlx::MySqlPool;

fn sample_new_company() -> NewCompany {
    NewCompany {
        name: "Test SA".into(),
        address: "Rue Test 1, 1000 Lausanne".into(),
        ide_number: Some("CHE109322551".into()),
        org_type: OrgType::Pme,
        accounting_language: Language::Fr,
        instance_language: Language::Fr,
    }
}

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
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

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn find_by_id_returns_none_for_missing(pool: MySqlPool) {
    let result = companies::find_by_id(&pool, 999_999).await.unwrap();
    assert!(result.is_none());
}

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn update_succeeds_with_current_version(pool: MySqlPool) {
    let created = companies::create(&pool, sample_new_company()).await.unwrap();

    let changes = CompanyUpdate {
        name: "Test SA (renamed)".into(),
        address: created.address.clone(),
        ide_number: created.ide_number.clone(),
        org_type: created.org_type,
        accounting_language: Language::De,
        instance_language: created.instance_language,
    };

    let updated = companies::update(&pool, created.id, created.version, changes)
        .await
        .expect("update should succeed");

    assert_eq!(updated.name, "Test SA (renamed)");
    assert_eq!(updated.accounting_language, Language::De);
    assert_eq!(updated.version, created.version + 1);
}

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn update_fails_on_stale_version(pool: MySqlPool) {
    let created = companies::create(&pool, sample_new_company()).await.unwrap();

    // Premier update : version 1 → 2
    let changes = CompanyUpdate {
        name: "First update".into(),
        address: created.address.clone(),
        ide_number: created.ide_number.clone(),
        org_type: created.org_type,
        accounting_language: created.accounting_language,
        instance_language: created.instance_language,
    };
    companies::update(&pool, created.id, 1, changes).await.unwrap();

    // Deuxième update avec version 1 stale → conflict
    let stale_changes = CompanyUpdate {
        name: "Stale update".into(),
        address: created.address.clone(),
        ide_number: created.ide_number.clone(),
        org_type: created.org_type,
        accounting_language: created.accounting_language,
        instance_language: created.instance_language,
    };
    let result = companies::update(&pool, created.id, 1, stale_changes).await;
    assert!(matches!(result, Err(DbError::OptimisticLockConflict)));
}

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn update_fails_on_missing_entity(pool: MySqlPool) {
    let changes = CompanyUpdate {
        name: "Ghost".into(),
        address: "Nowhere".into(),
        ide_number: None,
        org_type: OrgType::Pme,
        accounting_language: Language::Fr,
        instance_language: Language::Fr,
    };
    let result = companies::update(&pool, 999_999, 1, changes).await;
    assert!(matches!(result, Err(DbError::NotFound)));
}

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
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

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn unique_constraint_on_ide_number(pool: MySqlPool) {
    companies::create(&pool, sample_new_company()).await.unwrap();

    // Tentative de créer une seconde company avec le même IDE
    let result = companies::create(&pool, sample_new_company()).await;
    assert!(matches!(result, Err(DbError::UniqueConstraintViolation(_))));
}

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn empty_name_rejected(pool: MySqlPool) {
    let mut new = sample_new_company();
    new.name = String::new();
    let result = companies::create(&pool, new).await;
    assert!(matches!(result, Err(DbError::CheckConstraintViolation(_))));
}

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn empty_address_rejected(pool: MySqlPool) {
    let mut new = sample_new_company();
    new.address = "   ".into();
    let result = companies::create(&pool, new).await;
    assert!(matches!(result, Err(DbError::CheckConstraintViolation(_))));
}

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn invalid_ide_format_rejected(pool: MySqlPool) {
    let mut new = sample_new_company();
    new.ide_number = Some("INVALID".into());
    let result = companies::create(&pool, new).await;
    assert!(matches!(result, Err(DbError::CheckConstraintViolation(_))));
}

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn list_limit_clamped_to_max(pool: MySqlPool) {
    // Un limit très grand (i64::MAX) doit être clampé à MAX_LIST_LIMIT sans
    // provoquer d'erreur SQL — validation du clamp pre-query.
    companies::create(&pool, sample_new_company()).await.unwrap();
    let list = companies::list(&pool, i64::MAX, 0).await.unwrap();
    assert_eq!(list.len(), 1);

    // Test complémentaire : i64::MIN aussi
    let list_min = companies::list(&pool, i64::MIN, 0).await.unwrap();
    assert!(list_min.is_empty()); // limit clamped à 0
}

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn list_negative_values_normalized(pool: MySqlPool) {
    companies::create(&pool, sample_new_company()).await.unwrap();
    // Limite négative → clamped à 0 → liste vide
    let empty = companies::list(&pool, -5, 0).await.unwrap();
    assert!(empty.is_empty());

    // Offset négatif → clamped à 0, limite valide → retourne les résultats
    let list = companies::list(&pool, 10, -10).await.unwrap();
    assert_eq!(list.len(), 1);
}

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
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
