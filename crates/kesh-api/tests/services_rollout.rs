//! Story 32-4a — le rollout de la couche de services aux lectures des
//! écritures comptables, des postes ouverts et du carnet d'adresses.
//!
//! Ce fichier porte, pour les cinq services du rollout
//! (`services::{ecritures, postes_ouverts, carnet}`) : les tests de refus et
//! leur contrôle positif (AC9), la validation et les bornes de pagination au
//! service et par la route (AC10), la lecture seule et le filtre de société
//! (AC11). La garde lexicale reste dans `tests/services_motif.rs` (AC12).
//! La 32-4b y ajoute ses tests, sans le recréer (C-32-4a-9).
//!
//! Fixtures **synthétiques** : `seed_accounting_company`, une écriture sur deux
//! comptes lettrables (1000 et 2000 — aucun compte bancaire n'y est lié dans le
//! seed), un contact.

mod common;

use std::net::SocketAddr;
use std::path::Path;
use std::sync::Arc;

use chrono::{TimeDelta, Utc};
use jsonwebtoken::{Algorithm, EncodingKey, Header};
use kesh_api::auth::jwt::Claims;
use kesh_api::config::Config;
use kesh_api::{AppState, build_router};
use kesh_db::test_fixtures::{SeededCompany, seed_accounting_company};
use serde_json::Value;
use sqlx::MySqlPool;

use common::lecture_seule::pool_lecture_seule;

const TEST_JWT_SECRET: &[u8] = b"test-secret-32-bytes-minimum-test-secret-padding";
const TEST_ADMIN_PASSWORD: &str = "e2e-test-admin-password";

// ============================================================
// Montage HTTP (recopié de `services_motif.rs:43-141`, sans les bornes de
// période des rapports — C-32-4a-9, convention du dépôt)
// ============================================================

struct TestApp {
    base_url: String,
    client: reqwest::Client,
}

impl TestApp {
    fn url(&self, path: &str) -> String {
        format!("{}{}", self.base_url, path)
    }
}

fn test_config() -> Config {
    Config::from_fields_for_test(
        "mysql://test:test@localhost:3306/test".to_string(),
        "admin".to_string(),
        TEST_ADMIN_PASSWORD.to_string(),
        String::from_utf8(TEST_JWT_SECRET.to_vec()).unwrap(),
        TimeDelta::minutes(15),
        TimeDelta::days(30),
        TimeDelta::minutes(15),
        TimeDelta::minutes(15),
        100,
        TimeDelta::minutes(30),
        12,
    )
}

async fn spawn_app(pool: MySqlPool) -> TestApp {
    let config = test_config();
    let rate_limiter = kesh_api::middleware::rate_limit::RateLimiter::new(&config);
    let i18n = Arc::new(
        kesh_i18n::I18nBundle::load(
            Path::new(env!("CARGO_MANIFEST_DIR"))
                .parent()
                .unwrap()
                .join("kesh-i18n/locales")
                .as_path(),
        )
        .expect("load test i18n"),
    );
    let state = AppState::new_for_tests(pool, Arc::new(config), Arc::new(rate_limiter), i18n);
    let app = build_router(state, "nonexistent-static-dir".to_string());
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr: SocketAddr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        axum::serve(
            listener,
            app.into_make_service_with_connect_info::<SocketAddr>(),
        )
        .await
        .unwrap();
    });
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(2);
    loop {
        match tokio::net::TcpStream::connect(addr).await {
            Ok(_) => break,
            Err(_) if std::time::Instant::now() < deadline => {
                tokio::time::sleep(std::time::Duration::from_millis(5)).await;
            }
            Err(e) => panic!("test server not ready in 2s: {e}"),
        }
    }
    TestApp {
        base_url: format!("http://{}", addr),
        client: reqwest::Client::new(),
    }
}

fn forge_jwt(user_id: i64, role: &str, company_id: i64) -> String {
    let now = Utc::now().timestamp();
    let claims = Claims {
        sub: user_id.to_string(),
        role: role.to_string(),
        company_id,
        iat: now,
        exp: now + 3600,
    };
    jsonwebtoken::encode(
        &Header::new(Algorithm::HS256),
        &claims,
        &EncodingKey::from_secret(TEST_JWT_SECRET),
    )
    .unwrap()
}

/// `GET` authentifié : statut et corps JSON.
async fn get_json(app: &TestApp, jwt: &str, chemin: &str) -> (u16, Value) {
    let resp = app
        .client
        .get(app.url(chemin))
        .bearer_auth(jwt)
        .send()
        .await
        .unwrap();
    let statut = resp.status().as_u16();
    let corps = resp.json::<Value>().await.unwrap_or(Value::Null);
    (statut, corps)
}

// ============================================================
// Fixtures synthétiques
// ============================================================

async fn societe(pool: &MySqlPool) -> SeededCompany {
    seed_accounting_company(pool).await.expect("seed")
}

// ============================================================
// AC10 (c) — les bornes et un refus, par la route
// ============================================================

/// AC10 (c) — bornes des contacts et des écritures, et un refus de validation,
/// par la route. Joué **avant** le déplacement (T1 (i)) : il décrit le code
/// d'aujourd'hui, puis tient le code déplacé.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn ac10_c_bornes_et_refus_par_la_route(pool: MySqlPool) {
    let s = societe(&pool).await;
    let app = spawn_app(pool.clone()).await;
    let jwt = forge_jwt(s.admin_user_id, "Admin", s.company_id);

    let mut echecs = Vec::new();
    for (chemin, limit, offset) in [
        ("/api/v1/contacts?limit=1000&offset=-3", 100, 0),
        ("/api/v1/contacts", 20, 0),
        ("/api/v1/journal-entries?limit=0", 1, 0),
        ("/api/v1/journal-entries", 50, 0),
    ] {
        let (statut, corps) = get_json(&app, &jwt, chemin).await;
        if statut != 200 || corps["limit"] != limit || corps["offset"] != offset {
            echecs.push(format!(
                "{chemin} : attendu 200, limit {limit}, offset {offset} ; rendu {statut}, limit {}, offset {}",
                corps["limit"], corps["offset"]
            ));
        }
    }
    assert!(
        echecs.is_empty(),
        "AC10 (c) bornes :\n{}",
        echecs.join("\n")
    );

    let (statut, corps) = get_json(&app, &jwt, "/api/v1/journal-entries?dateFrom=x").await;
    assert_eq!(statut, 400, "AC10 (c) dateFrom=x : {corps}");
    assert_eq!(corps["error"]["code"], "VALIDATION_ERROR", "{corps}");
    let message = corps["error"]["message"].as_str().unwrap_or_default();
    assert!(
        message.starts_with("dateFrom invalide ("),
        "AC10 (c) message : {message}"
    );
}

// ============================================================
// AC11 (c) — ce que la `Lecture` de test fait d'un verrou
// ============================================================

/// Numéro d'erreur MariaDB d'une erreur sqlx, s'il y en a un.
fn numero_mariadb(e: &sqlx::Error) -> Option<u16> {
    e.as_database_error()
        .and_then(|d| d.try_downcast_ref::<sqlx::mysql::MySqlDatabaseError>())
        .map(|m| m.number())
}

/// AC11 (c) — mesure, puis témoin : sur `pool_lecture_seule` (session
/// `SET SESSION TRANSACTION READ ONLY`), un `SELECT … FOR UPDATE` est refusé
/// (1792), en autocommit **et** dans `begin()` — au patron de la T0 de la
/// 29-1 (`kesh-db/tests/closing_checks.rs`), qui l'avait mesuré pour
/// `START TRANSACTION … READ ONLY`. C'est ce qui permet à l'AC11 (a) de
/// confirmer aussi l'absence de verrou sur les branches exercées.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn ac11_c_un_verrou_est_refuse_sur_la_lecture_de_test(pool: MySqlPool) {
    societe(&pool).await;
    let ro = pool_lecture_seule(&pool).await;
    let sql = "SELECT 1 FROM fiscal_years LIMIT 1 FOR UPDATE";

    let autocommit = sqlx::query(sql).fetch_optional(&ro).await;
    let mut tx = ro.begin().await.expect("begin sur la lecture seule");
    let en_transaction = sqlx::query(sql).fetch_optional(&mut *tx).await;
    let _ = tx.rollback().await;

    let autocommit = autocommit.err().as_ref().and_then(numero_mariadb);
    let en_transaction = en_transaction.err().as_ref().and_then(numero_mariadb);
    assert_eq!(
        (autocommit, en_transaction),
        (Some(1792), Some(1792)),
        "AC11 (c) : FOR UPDATE en autocommit, puis dans begin()"
    );
}
