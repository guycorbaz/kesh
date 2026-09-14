//! Shared test utilities and fixtures for E2E tests
//!
//! ⚠️ **`dead_code` est autorisé ici, et ce n'est pas une négligence** : ce
//! module est compilé SÉPARÉMENT dans chaque binaire de test, si bien qu'un
//! helper employé par dix fichiers sur quarante est « jamais utilisé » dans les
//! trente autres. Sans cet attribut, le gate `clippy -D warnings` échouerait sur
//! des fonctions pourtant appelées. *(Constaté à la Story 25-1b, en ajoutant les
//! quatre helpers d'audit ci-dessous.)*
#![allow(dead_code)]

use kesh_db::entities::address::StructuredAddress;
use kesh_db::entities::{Language, NewCompany, OrgType};
use kesh_db::repositories::companies;
use sqlx::MySqlPool;

/// Create a test company with default values (required by Story 6.2 for users.company_id FK).
/// Used across E2E tests: onboarding, profile, rbac, users, companies, i18n, etc.
pub async fn create_test_company(pool: &MySqlPool) {
    companies::create(
        pool,
        NewCompany {
            name: "Test Company".into(),
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
            org_type: OrgType::Independant,
            accounting_language: Language::Fr,
            instance_language: Language::Fr,
        },
    )
    .await
    .expect("create test company");
}

// ---------------------------------------------------------------------------
// Assertions d'audit — Story 25-1b
// ---------------------------------------------------------------------------
//
// ⚠️ **Pourquoi ces helpers vivent ici** : avant cette story, chaque fichier de
// test réécrivait son propre `sqlx::query_scalar` sur `audit_log`. La story en
// ajoute quatorze ; les laisser diverger aurait été quatorze occasions
// d'asserter la mauvaise chose.
//
// ⚠️ **`details_json` se lit en `Option<Vec<u8>>`, jamais en `Value`** : MariaDB
// le stocke en blob binaire. C'est le piège que `reports_e2e` a documenté après
// s'être fait prendre.

/// Les actions écrites pour une entité, de la plus ancienne à la plus récente.
pub async fn audit_actions(pool: &MySqlPool, entity_type: &str, entity_id: i64) -> Vec<String> {
    sqlx::query_scalar(
        "SELECT action FROM audit_log WHERE entity_type = ? AND entity_id = ? ORDER BY id",
    )
    .bind(entity_type)
    .bind(entity_id)
    .fetch_all(pool)
    .await
    .expect("lecture des actions d'audit")
}

/// L'attribution de la DERNIÈRE entrée écrite pour une action donnée :
/// `(actor_type, actor_api_key_id, user_id, actor_label)`.
///
/// C'est le patron d'assertion à employer dès qu'on vérifie **qui** a agi —
/// `audit_actions` ne dit que **quoi**.
pub async fn audit_actor(
    pool: &MySqlPool,
    entity_type: &str,
    entity_id: i64,
    action: &str,
) -> (String, Option<i64>, i64, String) {
    sqlx::query_as(
        "SELECT actor_type, actor_api_key_id, user_id, actor_label FROM audit_log \
         WHERE entity_type = ? AND entity_id = ? AND action = ? ORDER BY id DESC LIMIT 1",
    )
    .bind(entity_type)
    .bind(entity_id)
    .bind(action)
    .fetch_one(pool)
    .await
    .expect("lecture de l'attribution d'audit")
}

/// Le `details_json` de la dernière entrée écrite pour une action donnée.
///
/// Rend `None` si la colonne est `NULL` — ce qui est un cas LÉGITIME et non un
/// défaut : `user.password_reset` l'écrit ainsi délibérément.
pub async fn audit_details(
    pool: &MySqlPool,
    entity_type: &str,
    entity_id: i64,
    action: &str,
) -> Option<serde_json::Value> {
    let raw: Option<Vec<u8>> = sqlx::query_scalar(
        "SELECT details_json FROM audit_log \
         WHERE entity_type = ? AND entity_id = ? AND action = ? ORDER BY id DESC LIMIT 1",
    )
    .bind(entity_type)
    .bind(entity_id)
    .bind(action)
    .fetch_one(pool)
    .await
    .expect("lecture des détails d'audit");
    raw.map(|bytes| serde_json::from_slice(&bytes).expect("details_json doit être du JSON valide"))
}

/// Le nombre total d'entrées d'audit — utile pour prouver qu'un refus n'écrit
/// **rien**, ce qu'aucune assertion positive ne peut établir.
pub async fn audit_count(pool: &MySqlPool) -> i64 {
    sqlx::query_scalar("SELECT COUNT(*) FROM audit_log")
        .fetch_one(pool)
        .await
        .expect("comptage des entrées d'audit")
}
