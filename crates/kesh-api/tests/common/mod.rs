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
// Empreinte d'un jeton de renouvellement — Story v0174-session-1 (#890)
// ---------------------------------------------------------------------------

/// L'empreinte qu'un jeton de renouvellement doit porter en base, recalculée
/// **indépendamment** de `kesh_db::repositories::refresh_tokens::token_digest`
/// (C-v0174s-19) : sans quoi la mutation « empreinte = identité » passerait,
/// le test comparant le code à lui-même.
///
/// Les 16 premiers octets de SHA-256(jeton), en hexadécimal minuscule, mis en
/// forme `8-4-4-4-12`, dont le 13ᵉ chiffre (position 15 de la chaîne, tirets
/// compris) est forcé à `0` (C-v0174s-2). Vecteur : `00000000-0000-4000-8000-
/// 000000000000` ⇒ `db8055e0-e030-0d5a-016b-ec4dc338d698` (`sha256sum`).
pub fn independent_refresh_digest(clear: &str) -> String {
    use sha2::{Digest, Sha256};
    let bytes = Sha256::digest(clear.as_bytes());
    let mut hex: Vec<char> = bytes[..16]
        .iter()
        .flat_map(|b| format!("{b:02x}").chars().collect::<Vec<_>>())
        .collect();
    hex[12] = '0';
    let s: String = hex.into_iter().collect();
    format!(
        "{}-{}-{}-{}-{}",
        &s[0..8],
        &s[8..12],
        &s[12..16],
        &s[16..20],
        &s[20..32]
    )
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

/// La séquence EXACTE des actions écrites au journal d'audit, toutes entités
/// confondues, de la plus ancienne à la plus récente (`ORDER BY id`) — Story
/// 15-7a2 (F3-4), partagé avec la 15-7b1 et la 15-7b2.
///
/// ⛔ Asserter la séquence et non un nombre : un compte resterait vert sur une
/// action remplacée par une autre.
pub async fn audit_sequence(pool: &MySqlPool) -> Vec<String> {
    sqlx::query_scalar("SELECT action FROM audit_log ORDER BY id")
        .fetch_all(pool)
        .await
        .expect("lecture de la séquence d'audit")
}

/// Crée un jeton d'API par l'endpoint HTTP, **sous un JWT** (la route est
/// interdite aux jetons), et rend `(id, secret clair kesh_pat_…)` — Story
/// 15-7b1 (R2-1/F2-7 de sa validation), remonté de `api_keys_e2e.rs` pour être
/// partagé avec `onboarding_audit_e2e.rs`.
///
/// Paramétré par le client et l'URL de base, chaque fichier de test ayant son
/// propre `TestApp`. `scope` vaut `"read"` ou `"read-write"` — un jeton en
/// lecture seule prend 403 sur toute route mutante.
pub async fn create_key_via_http(
    client: &reqwest::Client,
    base_url: &str,
    jwt: &str,
    name: &str,
    scope: &str,
) -> (i64, String) {
    let resp = client
        .post(format!("{base_url}/api/v1/settings/api-keys"))
        .header("Authorization", format!("Bearer {jwt}"))
        .json(&serde_json::json!({ "name": name, "scope": scope }))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 201, "création de clé doit réussir");
    let body: serde_json::Value = resp.json().await.unwrap();
    let id = body["id"].as_i64().unwrap();
    let key = body["key"].as_str().unwrap().to_string();
    assert!(key.starts_with("kesh_pat_"), "secret au format attendu");
    (id, key)
}

/// Le nom du verrou des gestes de rapprochement (`kesh_reconciliation::mutex`,
/// `reconcile:<base>:<société>:<compte>`), tenu par une connexion de test.
///
/// Partagé par la saisie manuelle (`bank_statements_manual_e2e.rs`, v016-2a) et
/// les imports de fichier (`bank_imports_e2e.rs`, #840, v0172-correctifs) : les
/// trois chemins d'écriture d'un relevé prennent ce même verrou. Il reste tenu
/// par la session de `conn` jusqu'à un `RELEASE_LOCK` ou la fin de la connexion.
pub async fn take_account_lock(
    conn: &mut sqlx::MySqlConnection,
    company: i64,
    bank: i64,
) -> String {
    let db: String = sqlx::query_scalar("SELECT DATABASE()")
        .fetch_one(&mut *conn)
        .await
        .unwrap();
    let name = format!("reconcile:{db}:{company}:{bank}");
    let got: Option<i32> = sqlx::query_scalar("SELECT GET_LOCK(?, 5)")
        .bind(&name)
        .fetch_one(&mut *conn)
        .await
        .unwrap();
    assert_eq!(got, Some(1), "montage : verrou pris");
    name
}

// ---------------------------------------------------------------------------
// Règlement d'une facture client à sa version courante — Story 34-5 (#951)
// ---------------------------------------------------------------------------

/// La version que la facture porte en base : celle qu'exigent
/// `POST /invoices/{id}/settlements` et `invoice_settlements_write::settle_invoice`
/// (C-e343-13). Les tests du contrôle de version passent la leur à la main.
pub async fn invoice_version(pool: &MySqlPool, invoice_id: i64) -> i32 {
    sqlx::query_scalar("SELECT version FROM invoices WHERE id = ?")
        .bind(invoice_id)
        .fetch_one(pool)
        .await
        .expect("version de la facture")
}

/// `invoice_settlements_write::settle_invoice` à la version courante de la
/// facture : mêmes arguments, la version en moins.
pub async fn settle_at_current_version(
    pool: &MySqlPool,
    user_id: i64,
    company_id: i64,
    invoice_id: i64,
    choice: kesh_db::entities::SettlementChoice,
    amount: rust_decimal::Decimal,
    settled_on: chrono::NaiveDate,
) -> Result<
    kesh_db::repositories::invoice_settlements_write::SettlementOutcome,
    kesh_db::errors::DbError,
> {
    let version = invoice_version(pool, invoice_id).await;
    kesh_db::repositories::invoice_settlements_write::settle_invoice(
        pool,
        user_id,
        company_id,
        invoice_id,
        choice,
        amount,
        settled_on,
        version,
    )
    .await
}

/// Témoin du rejeu sur interblocage (Story 15-5e1, choix C74).
pub mod capture_rejeu;

/// La `Lecture` de test en lecture seule (Story 32-4a, C-32-4a-9).
pub mod lecture_seule;

/// La capture générique de `tracing`, tous niveaux et toutes cibles (Story
/// 32-6a, AC-a14).
pub mod capture_trace;

/// Le montage et les outils de test du serveur MCP (Story 32-6a).
pub mod mcp;

/// Le montage des tests des outils de lecture : sociétés piégées, témoins,
/// parcours de l'enveloppe (Story 32-9a).
pub mod mcp_pieges;
