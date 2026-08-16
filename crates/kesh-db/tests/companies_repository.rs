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
