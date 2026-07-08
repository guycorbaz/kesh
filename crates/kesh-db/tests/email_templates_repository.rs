//! Tests d'intégration pour le repository `email_templates` (Epic 20 #224,
//! Story 20-1).

use kesh_db::entities::address::StructuredAddress;
use kesh_db::entities::audit_log::ActorType;
use kesh_db::entities::{EmailTemplateType, Language, NewCompany, OrgType};
use kesh_db::errors::DbError;
use kesh_db::repositories::{audit_log, companies, email_templates};
use sqlx::MySqlPool;

async fn create_test_company(pool: &MySqlPool, name: &str) -> i64 {
    companies::create(
        pool,
        NewCompany {
            name: name.to_string(),
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
    .expect("create test company")
    .id
}

async fn create_admin_user(pool: &MySqlPool, company_id: i64) -> i64 {
    let result = sqlx::query(
        "INSERT INTO users (username, password_hash, role, active, company_id) \
         VALUES (?, ?, 'Admin', TRUE, ?)",
    )
    .bind(format!("admin_{company_id}"))
    .bind("$argon2id$v=19$m=19456,t=2,p=1$QUJDRA$YWJjZGVmZ2hpams")
    .bind(company_id)
    .execute(pool)
    .await
    .expect("create admin user for test");
    result.last_insert_id() as i64
}

async fn audit_count(pool: &MySqlPool, entity_id: i64, action: &str) -> i64 {
    let count: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM audit_log WHERE entity_type = 'email_template' AND entity_id = ? AND action = ?",
    )
    .bind(entity_id)
    .bind(action)
    .fetch_one(pool)
    .await
    .unwrap();
    count.0
}

/// Zéro-config : aucune ligne en base → `get_effective` retombe sur le
/// défaut, jamais d'erreur (AC #16).
#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn get_effective_falls_back_to_default_when_no_row(pool: MySqlPool) {
    let company_id = create_test_company(&pool, "Zero Config Co").await;

    let effective = email_templates::get_effective(
        &pool,
        company_id,
        EmailTemplateType::InvoiceSend,
        Language::Fr,
    )
    .await
    .unwrap();

    assert!(effective.is_default);
    assert_eq!(effective.version, None);
    assert!(effective.subject.contains("{invoiceNumber}"));
    assert!(!effective.allowed_variables.is_empty());
}

/// `list_effective_for_company` sur une company neuve retourne les 4
/// combinaisons type×langue, toutes en défaut (AC #16) — jamais de tableau
/// vide, jamais de 404.
#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn list_effective_returns_four_defaults_for_fresh_company(pool: MySqlPool) {
    let company_id = create_test_company(&pool, "List Co").await;

    let list = email_templates::list_effective_for_company(&pool, company_id)
        .await
        .unwrap();

    assert_eq!(list.len(), 4);
    assert!(list.iter().all(|t| t.is_default));
}

/// Création d'un override (`expected_version = None` sur ligne absente) →
/// `INSERT`, `version = 1`, audit `email_template.updated` écrit.
#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn upsert_override_creates_row_with_version_one(pool: MySqlPool) {
    let company_id = create_test_company(&pool, "Create Co").await;
    let admin_user_id = create_admin_user(&pool, company_id).await;

    let created = email_templates::upsert_override(
        &pool,
        company_id,
        EmailTemplateType::InvoiceSend,
        Language::Fr,
        None,
        admin_user_id,
        None,
        "Sujet perso {invoiceNumber}".to_string(),
        "Corps perso {amount}".to_string(),
    )
    .await
    .unwrap();

    assert_eq!(created.version, 1);
    assert_eq!(created.company_id, company_id);
    assert_eq!(created.subject, "Sujet perso {invoiceNumber}");
    assert_eq!(
        audit_count(&pool, created.id, "email_template.updated").await,
        1
    );

    let effective = email_templates::get_effective(
        &pool,
        company_id,
        EmailTemplateType::InvoiceSend,
        Language::Fr,
    )
    .await
    .unwrap();
    assert!(!effective.is_default);
    assert_eq!(effective.version, Some(1));
    assert_eq!(effective.subject, "Sujet perso {invoiceNumber}");
}

/// Race : créer avec `expected_version = None` alors qu'une ligne existe
/// déjà → `OptimisticLockConflict` (pas de crash sur violation `UNIQUE`).
#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn upsert_override_create_conflicts_when_row_already_exists(pool: MySqlPool) {
    let company_id = create_test_company(&pool, "Race Create Co").await;
    let admin_user_id = create_admin_user(&pool, company_id).await;

    email_templates::upsert_override(
        &pool,
        company_id,
        EmailTemplateType::InvoiceSend,
        Language::Fr,
        None,
        admin_user_id,
        None,
        "Sujet 1".to_string(),
        "Corps 1".to_string(),
    )
    .await
    .unwrap();

    let result = email_templates::upsert_override(
        &pool,
        company_id,
        EmailTemplateType::InvoiceSend,
        Language::Fr,
        None,
        admin_user_id,
        None,
        "Sujet 2".to_string(),
        "Corps 2".to_string(),
    )
    .await;

    assert!(matches!(result, Err(DbError::OptimisticLockConflict)));
}

/// Modification à la bonne version → `UPDATE`, `version + 1`, audit écrit.
#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn upsert_override_updates_at_correct_version(pool: MySqlPool) {
    let company_id = create_test_company(&pool, "Update Co").await;
    let admin_user_id = create_admin_user(&pool, company_id).await;

    let created = email_templates::upsert_override(
        &pool,
        company_id,
        EmailTemplateType::InvoiceSend,
        Language::Fr,
        None,
        admin_user_id,
        None,
        "Sujet initial".to_string(),
        "Corps initial".to_string(),
    )
    .await
    .unwrap();

    let updated = email_templates::upsert_override(
        &pool,
        company_id,
        EmailTemplateType::InvoiceSend,
        Language::Fr,
        Some(created.version),
        admin_user_id,
        None,
        "Sujet modifié".to_string(),
        "Corps modifié".to_string(),
    )
    .await
    .unwrap();

    assert_eq!(updated.version, 2);
    assert_eq!(updated.subject, "Sujet modifié");
    assert_eq!(
        audit_count(&pool, created.id, "email_template.updated").await,
        2
    );
}

/// Version stale (quelqu'un d'autre a déjà modifié) → `OptimisticLockConflict`.
#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn upsert_override_stale_version_conflicts(pool: MySqlPool) {
    let company_id = create_test_company(&pool, "Stale Co").await;
    let admin_user_id = create_admin_user(&pool, company_id).await;

    let created = email_templates::upsert_override(
        &pool,
        company_id,
        EmailTemplateType::InvoiceSend,
        Language::Fr,
        None,
        admin_user_id,
        None,
        "Sujet".to_string(),
        "Corps".to_string(),
    )
    .await
    .unwrap();

    // Un premier "onglet" modifie et bump la version à 2.
    email_templates::upsert_override(
        &pool,
        company_id,
        EmailTemplateType::InvoiceSend,
        Language::Fr,
        Some(created.version),
        admin_user_id,
        None,
        "Sujet v2".to_string(),
        "Corps v2".to_string(),
    )
    .await
    .unwrap();

    // Un second "onglet" resté sur la version 1 tente de modifier → conflit.
    let result = email_templates::upsert_override(
        &pool,
        company_id,
        EmailTemplateType::InvoiceSend,
        Language::Fr,
        Some(created.version),
        admin_user_id,
        None,
        "Sujet v1-stale".to_string(),
        "Corps v1-stale".to_string(),
    )
    .await;

    assert!(matches!(result, Err(DbError::OptimisticLockConflict)));
}

/// KF-004 : payload identique à l'override persisté → pas de bump version,
/// **aucune** nouvelle entrée audit.
#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn upsert_override_no_op_does_not_bump_version_or_audit(pool: MySqlPool) {
    let company_id = create_test_company(&pool, "NoOp Co").await;
    let admin_user_id = create_admin_user(&pool, company_id).await;

    let created = email_templates::upsert_override(
        &pool,
        company_id,
        EmailTemplateType::InvoiceSend,
        Language::Fr,
        None,
        admin_user_id,
        None,
        "Sujet stable".to_string(),
        "Corps stable".to_string(),
    )
    .await
    .unwrap();
    assert_eq!(
        audit_count(&pool, created.id, "email_template.updated").await,
        1
    );

    let result = email_templates::upsert_override(
        &pool,
        company_id,
        EmailTemplateType::InvoiceSend,
        Language::Fr,
        Some(created.version),
        admin_user_id,
        None,
        "Sujet stable".to_string(),
        "Corps stable".to_string(),
    )
    .await
    .unwrap();

    assert_eq!(result.version, created.version);
    // No-op : toujours 1 seule entrée audit (celle de la création).
    assert_eq!(
        audit_count(&pool, created.id, "email_template.updated").await,
        1
    );
}

/// `restore_default` supprime l'override ; un `get_effective` suivant
/// retombe sur le défaut. Audit `email_template.restored_default` écrit.
#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn restore_default_deletes_override_and_falls_back(pool: MySqlPool) {
    let company_id = create_test_company(&pool, "Restore Co").await;
    let admin_user_id = create_admin_user(&pool, company_id).await;

    let created = email_templates::upsert_override(
        &pool,
        company_id,
        EmailTemplateType::InvoiceSend,
        Language::Fr,
        None,
        admin_user_id,
        None,
        "Sujet override".to_string(),
        "Corps override".to_string(),
    )
    .await
    .unwrap();

    email_templates::restore_default(
        &pool,
        company_id,
        EmailTemplateType::InvoiceSend,
        Language::Fr,
        admin_user_id,
        None,
    )
    .await
    .unwrap();

    let effective = email_templates::get_effective(
        &pool,
        company_id,
        EmailTemplateType::InvoiceSend,
        Language::Fr,
    )
    .await
    .unwrap();
    assert!(effective.is_default);
    assert_eq!(
        audit_count(&pool, created.id, "email_template.restored_default").await,
        1
    );
}

/// `restore_default` sans override existant : idempotent, pas d'erreur,
/// pas d'audit.
#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn restore_default_is_idempotent_when_no_override(pool: MySqlPool) {
    let company_id = create_test_company(&pool, "Idempotent Restore Co").await;
    let admin_user_id = create_admin_user(&pool, company_id).await;

    // Aucun override créé — ne doit pas échouer.
    let result = email_templates::restore_default(
        &pool,
        company_id,
        EmailTemplateType::InvoiceSend,
        Language::Fr,
        admin_user_id,
        None,
    )
    .await;
    assert!(result.is_ok());

    let count: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM audit_log WHERE entity_type = 'email_template' AND action = 'email_template.restored_default'",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(count.0, 0);
}

/// `UNIQUE(company_id, template_type, language)` — un `INSERT` direct en
/// doublon est rejeté par la contrainte (contrôle schéma, indépendant du
/// repository qui l'évite déjà via `FOR UPDATE`).
#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn unique_constraint_rejects_duplicate_row(pool: MySqlPool) {
    let company_id = create_test_company(&pool, "Unique Co").await;

    sqlx::query(
        "INSERT INTO email_templates (company_id, template_type, language, subject, body) \
         VALUES (?, 'invoice_send', 'FR', 'S1', 'B1')",
    )
    .bind(company_id)
    .execute(&pool)
    .await
    .unwrap();

    let dup = sqlx::query(
        "INSERT INTO email_templates (company_id, template_type, language, subject, body) \
         VALUES (?, 'invoice_send', 'FR', 'S2', 'B2')",
    )
    .bind(company_id)
    .execute(&pool)
    .await;

    assert!(dup.is_err());
}

/// Cross-tenant : deux companies ont des overrides indépendants pour la
/// même combinaison type×langue.
#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn cross_tenant_overrides_are_independent(pool: MySqlPool) {
    let company_a = create_test_company(&pool, "Tenant A").await;
    let company_b = create_test_company(&pool, "Tenant B").await;
    let user_a = create_admin_user(&pool, company_a).await;

    email_templates::upsert_override(
        &pool,
        company_a,
        EmailTemplateType::InvoiceSend,
        Language::Fr,
        None,
        user_a,
        None,
        "Sujet A".to_string(),
        "Corps A".to_string(),
    )
    .await
    .unwrap();

    let effective_a = email_templates::get_effective(
        &pool,
        company_a,
        EmailTemplateType::InvoiceSend,
        Language::Fr,
    )
    .await
    .unwrap();
    let effective_b = email_templates::get_effective(
        &pool,
        company_b,
        EmailTemplateType::InvoiceSend,
        Language::Fr,
    )
    .await
    .unwrap();

    assert!(!effective_a.is_default);
    assert_eq!(effective_a.subject, "Sujet A");
    assert!(effective_b.is_default); // company B jamais touchée
}

/// Code-review Pass 1 (AC #11) : `upsert_override` doit threader le PAT
/// (`for_actor`) pour que l'audit distingue une action via clé API d'une
/// action UI web — pas `NewAuditLogEntry::user` seul.
#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn upsert_override_threads_actor_api_key_id_into_audit(pool: MySqlPool) {
    let company_id = create_test_company(&pool, "PAT Actor Co").await;
    let admin_user_id = create_admin_user(&pool, company_id).await;
    let fake_api_key_id = 4242i64;

    let created = email_templates::upsert_override(
        &pool,
        company_id,
        EmailTemplateType::InvoiceSend,
        Language::Fr,
        None,
        admin_user_id,
        Some(fake_api_key_id),
        "Sujet PAT".to_string(),
        "Corps PAT".to_string(),
    )
    .await
    .unwrap();

    let entries = audit_log::find_by_entity(&pool, "email_template", created.id, 1)
        .await
        .unwrap();
    let entry = entries.first().expect("audit entry attendue");
    assert_eq!(entry.actor_type, ActorType::ApiKey);
    assert_eq!(entry.actor_api_key_id, Some(fake_api_key_id));
}

/// Code-review Pass 1 (Blind Hunter #3/#4) : concurrence RÉELLE (pas
/// seulement séquentielle) sur deux créations simultanées de la même
/// combinaison type×langue → exactement une réussit, l'autre reçoit
/// `OptimisticLockConflict` — jamais un `UniqueConstraintViolation` brut
/// remonté par la contrainte `UNIQUE` (remap explicite ajouté en review).
///
/// `#[sqlx::test]` (pas une connexion directe `DATABASE_URL` façon
/// `invoices::tests::test_mark_as_paid_concurrent_one_succeeds_other_409`) :
/// le pool éphémère fourni par la macro garantit un schéma à jour (chaîne de
/// migrations complète rejouée), et son clone partage le même pool de
/// connexions sous-jacent — suffisant pour deux opérations concurrentes.
#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn upsert_override_true_concurrent_create_yields_exactly_one_conflict(pool: MySqlPool) {
    let company_id = create_test_company(&pool, "True Concurrency Co").await;
    let admin_user_id = create_admin_user(&pool, company_id).await;

    let pool_a = pool.clone();
    let pool_b = pool.clone();

    let (res_a, res_b) = tokio::join!(
        async move {
            email_templates::upsert_override(
                &pool_a,
                company_id,
                EmailTemplateType::InvoiceSend,
                Language::Fr,
                None,
                admin_user_id,
                None,
                "Sujet concurrent A".to_string(),
                "Corps concurrent A".to_string(),
            )
            .await
        },
        async move {
            email_templates::upsert_override(
                &pool_b,
                company_id,
                EmailTemplateType::InvoiceSend,
                Language::Fr,
                None,
                admin_user_id,
                None,
                "Sujet concurrent B".to_string(),
                "Corps concurrent B".to_string(),
            )
            .await
        },
    );

    let successes = [&res_a, &res_b].iter().filter(|r| r.is_ok()).count();
    assert_eq!(
        successes, 1,
        "exactement une création concurrente doit réussir"
    );
    let conflicts = [&res_a, &res_b]
        .iter()
        .filter(|r| matches!(r, Err(DbError::OptimisticLockConflict)))
        .count();
    assert_eq!(
        conflicts, 1,
        "l'autre doit recevoir OptimisticLockConflict (jamais un UniqueConstraintViolation brut)"
    );
}
