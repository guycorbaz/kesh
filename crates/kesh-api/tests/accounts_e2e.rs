//! Story 14-3a — tests E2E HTTP pour `/api/v1/accounts`.
//!
//! Il n'existait **aucune** suite E2E dédiée aux comptes avant cette story
//! (seul `idor_multi_tenant_e2e.rs` touchait la ressource). Pattern hérité de
//! `reconciliation_rules_e2e.rs` : forge JWT directement + `spawn_app` éphémère.
//!
//! Couvre : rôles et postabilité sur les 3 verbes, conflit de rôle singleton
//! **avec le compte en conflit nommé dans `details`**, contrat full-replace du
//! PUT, cycle archive → réactivation (#269) et ses gardes, RBAC, IDOR.
#![allow(clippy::too_many_arguments)]

use std::net::SocketAddr;
use std::path::Path;
use std::sync::Arc;

use chrono::{TimeDelta, Utc};
use jsonwebtoken::{Algorithm, EncodingKey, Header};
use kesh_api::auth::jwt::Claims;
use kesh_api::auth::password::hash_password;
use kesh_api::config::Config;
use kesh_api::{AppState, build_router};
use kesh_db::entities::account::{AccountRole, AccountType};
use kesh_db::entities::address::StructuredAddress;
use kesh_db::entities::{Language, NewAccount, NewCompany, NewUser, OrgType, Role};
use kesh_db::repositories::{accounts, companies, users};
use serde_json::{Value, json};
use sqlx::MySqlPool;

const TEST_JWT_SECRET: &[u8] = b"test-secret-32-bytes-minimum-test-secret-padding";

// ============================================================
// Spawn helpers
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
        "mysql://test".into(),
        "admin".into(),
        "e2e-test-password".into(),
        std::str::from_utf8(TEST_JWT_SECRET).unwrap().to_string(),
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
    let app = build_router(state.clone(), "nonexistent-static-dir".to_string());
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

fn role_str(role: Role) -> &'static str {
    match role {
        Role::Admin => "Admin",
        Role::Comptable => "Comptable",
        Role::Consultation => "Consultation",
    }
}

// ============================================================
// Domain seed helpers
// ============================================================

async fn create_company(pool: &MySqlPool, name: &str) -> i64 {
    companies::create(
        pool,
        NewCompany {
            name: name.into(),
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
    .unwrap()
    .id
}

async fn create_user(pool: &MySqlPool, username: &str, role: Role, company_id: i64) -> i64 {
    users::create(
        pool,
        NewUser {
            username: username.into(),
            password_hash: hash_password("password123").unwrap(),
            role,
            active: true,
            company_id,
            email: None,
        },
    )
    .await
    .unwrap()
    .id
}

/// Crée un compte via le repository (chemin non-HTTP, pour préparer un état).
async fn mk_account(
    pool: &MySqlPool,
    company_id: i64,
    user_id: i64,
    number: &str,
    role: Option<AccountRole>,
) -> i64 {
    accounts::create(
        pool,
        user_id,
        NewAccount::new(
            company_id,
            number,
            format!("Compte {number}"),
            AccountType::Asset,
            None,
        )
        .with_role(role, true),
    )
    .await
    .unwrap()
    .id
}

/// Contexte minimal : 1 société + 1 admin.
async fn setup(pool: &MySqlPool) -> (i64, i64, String) {
    let company_id = create_company(pool, "Acme SA").await;
    let admin_id = create_user(pool, "admin", Role::Admin, company_id).await;
    let token = forge_jwt(admin_id, role_str(Role::Admin), company_id);
    (company_id, admin_id, token)
}

// ============================================================
// Rôle & postabilité — contrat des 3 verbes
// ============================================================

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn post_and_get_expose_role_and_postable(pool: MySqlPool) {
    let app = spawn_app(pool.clone()).await;
    let (_company_id, _admin_id, token) = setup(&pool).await;

    let res = app
        .client
        .post(app.url("/api/v1/accounts"))
        .bearer_auth(&token)
        .json(&json!({
            "number": "1100",
            "name": "Débiteurs",
            "accountType": "Asset",
            "role": "Receivable",
            "postable": false
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 201);
    let body: Value = res.json().await.unwrap();
    assert_eq!(body["role"], "Receivable");
    assert_eq!(body["postable"], false);

    // GET renvoie les mêmes champs (contrat rétro-compatible : ajout de champs).
    let list: Value = app
        .client
        .get(app.url("/api/v1/accounts"))
        .bearer_auth(&token)
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(list[0]["role"], "Receivable");
    assert_eq!(list[0]["postable"], false);
}

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn post_defaults_role_null_and_postable_true(pool: MySqlPool) {
    let app = spawn_app(pool.clone()).await;
    let (_c, _a, token) = setup(&pool).await;

    let body: Value = app
        .client
        .post(app.url("/api/v1/accounts"))
        .bearer_auth(&token)
        .json(&json!({ "number": "1000", "name": "Caisse", "accountType": "Asset" }))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(body["role"], Value::Null);
    assert_eq!(body["postable"], true);
}

/// Contrat **full-replace** : omettre `role` ou `postable` est un 400, pas un
/// effacement silencieux. C'est le point de conception le plus discutable de la
/// story — il est donc testé explicitement dans les deux sens.
#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn put_requires_role_and_postable_explicitly(pool: MySqlPool) {
    let app = spawn_app(pool.clone()).await;
    let (company_id, admin_id, token) = setup(&pool).await;
    let id = mk_account(
        &pool,
        company_id,
        admin_id,
        "1100",
        Some(AccountRole::Receivable),
    )
    .await;

    // `role` omis → 400 (et surtout : le rôle N'EST PAS effacé).
    let res = app
        .client
        .put(app.url(&format!("/api/v1/accounts/{id}")))
        .bearer_auth(&token)
        .json(
            &json!({ "name": "Débiteurs", "accountType": "Asset", "postable": true, "version": 1 }),
        )
        .send()
        .await
        .unwrap();
    assert_eq!(
        res.status(),
        400,
        "omettre `role` doit être un 400 explicite"
    );

    // `postable` omis → 400.
    let res = app
        .client
        .put(app.url(&format!("/api/v1/accounts/{id}")))
        .bearer_auth(&token)
        .json(&json!({ "name": "Débiteurs", "accountType": "Asset", "role": "Receivable", "version": 1 }))
        .send()
        .await
        .unwrap();
    assert_eq!(
        res.status(),
        400,
        "omettre `postable` doit être un 400 explicite"
    );

    // Le rôle est intact après les deux refus.
    let after = accounts::find_by_id(&pool, id).await.unwrap().unwrap();
    assert_eq!(after.role, Some(AccountRole::Receivable));

    // `role: null` explicite → le rôle est bien retiré (intention du client).
    let res = app
        .client
        .put(app.url(&format!("/api/v1/accounts/{id}")))
        .bearer_auth(&token)
        .json(&json!({
            "name": "Débiteurs", "accountType": "Asset",
            "role": null, "postable": true, "version": 1
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 200);
    let after = accounts::find_by_id(&pool, id).await.unwrap().unwrap();
    assert_eq!(after.role, None);
}

// ============================================================
// Conflit de rôle singleton
// ============================================================

/// L'exigence n'est PAS seulement « 409 » : le corps doit **nommer** le compte
/// qui porte déjà le rôle, sans quoi l'utilisateur ne sait pas quoi corriger.
#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn singleton_role_conflict_names_the_holding_account(pool: MySqlPool) {
    let app = spawn_app(pool.clone()).await;
    let (company_id, admin_id, token) = setup(&pool).await;
    let holder = mk_account(
        &pool,
        company_id,
        admin_id,
        "1100",
        Some(AccountRole::Receivable),
    )
    .await;
    let other = mk_account(&pool, company_id, admin_id, "1101", None).await;

    let res = app
        .client
        .put(app.url(&format!("/api/v1/accounts/{other}")))
        .bearer_auth(&token)
        .json(&json!({
            "name": "Débiteurs bis", "accountType": "Asset",
            "role": "Receivable", "postable": true, "version": 1
        }))
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), 409);
    let body: Value = res.json().await.unwrap();
    assert_eq!(body["error"]["code"], "ACCOUNT_ROLE_ALREADY_ASSIGNED");
    assert_eq!(body["error"]["details"]["accountId"], holder);
    assert_eq!(body["error"]["details"]["accountNumber"], "1100");
    assert_eq!(body["error"]["details"]["role"], "Receivable");
    assert!(
        body["error"]["message"].as_str().unwrap().contains("1100"),
        "le message doit nommer le compte en conflit : {}",
        body["error"]["message"]
    );
}

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn multi_valued_role_accepted_twice(pool: MySqlPool) {
    let app = spawn_app(pool.clone()).await;
    let (_c, _a, token) = setup(&pool).await;

    for number in ["2850", "2860"] {
        let res = app
            .client
            .post(app.url("/api/v1/accounts"))
            .bearer_auth(&token)
            .json(&json!({
                "number": number, "name": format!("Fonds {number}"),
                "accountType": "Liability", "role": "EquityOther"
            }))
            .send()
            .await
            .unwrap();
        assert_eq!(res.status(), 201, "EquityOther n'est pas singleton");
    }
}

// ============================================================
// Réactivation (#269)
// ============================================================

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn archive_then_reactivate_round_trip(pool: MySqlPool) {
    let app = spawn_app(pool.clone()).await;
    let (company_id, admin_id, token) = setup(&pool).await;
    let id = mk_account(&pool, company_id, admin_id, "1000", None).await;

    let res = app
        .client
        .put(app.url(&format!("/api/v1/accounts/{id}/archive")))
        .bearer_auth(&token)
        .json(&json!({ "version": 1 }))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 200);

    let res = app
        .client
        .put(app.url(&format!("/api/v1/accounts/{id}/reactivate")))
        .bearer_auth(&token)
        .json(&json!({ "version": 2 }))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 200);
    let body: Value = res.json().await.unwrap();
    assert_eq!(body["active"], true);
    assert_eq!(body["version"], 3);

    // Idempotent : réactiver un compte actif ne bump pas la version.
    let body: Value = app
        .client
        .put(app.url(&format!("/api/v1/accounts/{id}/reactivate")))
        .bearer_auth(&token)
        .json(&json!({ "version": 3 }))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(body["version"], 3, "réactiver un compte actif = no-op");
}

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn reactivate_refused_when_parent_archived(pool: MySqlPool) {
    let app = spawn_app(pool.clone()).await;
    let (company_id, admin_id, token) = setup(&pool).await;

    let parent = mk_account(&pool, company_id, admin_id, "10", None).await;
    let child = accounts::create(
        &pool,
        admin_id,
        NewAccount::new(
            company_id,
            "1000",
            "Caisse",
            AccountType::Asset,
            Some(parent),
        ),
    )
    .await
    .unwrap();

    accounts::archive(&pool, child.id, child.version, admin_id)
        .await
        .unwrap();
    accounts::archive(&pool, parent, 1, admin_id).await.unwrap();

    let res = app
        .client
        .put(app.url(&format!("/api/v1/accounts/{}/reactivate", child.id)))
        .bearer_auth(&token)
        .json(&json!({ "version": 2 }))
        .send()
        .await
        .unwrap();
    assert_eq!(
        res.status(),
        409,
        "un compte actif sous un parent archivé serait incohérent"
    );
    let body: Value = res.json().await.unwrap();
    assert_eq!(
        body["error"]["code"], "ACCOUNT_PARENT_ARCHIVED",
        "code dédié, pour un message qui dit à l'utilisateur quoi corriger"
    );
    assert!(
        body["error"]["message"].as_str().unwrap().contains("10"),
        "le message doit nommer le parent archivé : {}",
        body["error"]["message"]
    );
}

/// Code review 14-3a (HIGH) : le conflit de rôle singleton doit être détecté au
/// `POST`, avec le compte détenteur nommé — pas remonté en `RESOURCE_CONFLICT`
/// générique que le formulaire traduirait par « ce numéro existe déjà ».
#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn create_names_the_holder_on_singleton_role_conflict(pool: MySqlPool) {
    let app = spawn_app(pool.clone()).await;
    let (company_id, admin_id, token) = setup(&pool).await;
    let holder = mk_account(
        &pool,
        company_id,
        admin_id,
        "1100",
        Some(AccountRole::Receivable),
    )
    .await;

    let res = app
        .client
        .post(app.url("/api/v1/accounts"))
        .bearer_auth(&token)
        .json(&json!({
            "number": "1101", "name": "Débiteurs bis", "accountType": "Asset",
            "role": "Receivable"
        }))
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), 409);
    let body: Value = res.json().await.unwrap();
    assert_eq!(body["error"]["code"], "ACCOUNT_ROLE_ALREADY_ASSIGNED");
    assert_eq!(body["error"]["details"]["accountId"], holder);
    assert_eq!(body["error"]["details"]["accountNumber"], "1100");
    assert!(
        body["error"]["message"].as_str().unwrap().contains("1100"),
        "le message doit nommer le compte détenteur : {}",
        body["error"]["message"]
    );
}

/// Code review 14-3a (D1) : la frontière bilan / résultat est validée au POST
/// comme au PUT — un rôle de bilan sur une charge est un 400 typé.
#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn create_rejects_role_on_incompatible_type(pool: MySqlPool) {
    let app = spawn_app(pool.clone()).await;
    let (_company_id, _admin_id, token) = setup(&pool).await;

    let res = app
        .client
        .post(app.url("/api/v1/accounts"))
        .bearer_auth(&token)
        .json(&json!({
            "number": "6500", "name": "Frais admin", "accountType": "Expense",
            "role": "Payable"
        }))
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), 400);
    let body: Value = res.json().await.unwrap();
    assert_eq!(body["error"]["code"], "ACCOUNT_ROLE_INVALID_FOR_TYPE");
}

/// Code review 14-3a (HIGH) : `PUT /accounts/{id}` doit refuser un compte d'une
/// autre société (IDOR) — la garde existait sur archive/reactivate mais pas ici,
/// et la story y fait désormais transiter `role`/`postable`.
#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn update_is_scoped_to_company(pool: MySqlPool) {
    let app = spawn_app(pool.clone()).await;
    let (_company_a, _admin_a, token_a) = setup(&pool).await;

    // Compte appartenant à une AUTRE société.
    let company_b = create_company(&pool, "Autre SA").await;
    let admin_b = create_user(&pool, "admin_b", Role::Admin, company_b).await;
    let victim = mk_account(&pool, company_b, admin_b, "1100", None).await;

    let res = app
        .client
        .put(app.url(&format!("/api/v1/accounts/{victim}")))
        .bearer_auth(&token_a)
        .json(&json!({
            "name": "Détourné", "accountType": "Asset",
            "role": "Receivable", "postable": true, "version": 1
        }))
        .send()
        .await
        .unwrap();

    assert_eq!(
        res.status(),
        404,
        "IDOR : modifier le compte d'une autre société doit renvoyer 404"
    );

    // Et le compte victime est intact.
    let still = accounts::find_by_id(&pool, victim).await.unwrap().unwrap();
    assert_eq!(still.name, "Compte 1100");
    assert_eq!(still.role, None);
}

/// Code review 14-3a (D4) : `clearRole: true` réactive un compte dont le rôle a
/// été repris, en le laissant sans rôle plutôt qu'en échouant.
#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn reactivate_with_clear_role_succeeds_after_conflict(pool: MySqlPool) {
    let app = spawn_app(pool.clone()).await;
    let (company_id, admin_id, token) = setup(&pool).await;

    let a = accounts::create(
        &pool,
        admin_id,
        NewAccount::new(company_id, "1100", "Débiteurs", AccountType::Asset, None)
            .with_role(Some(AccountRole::Receivable), true),
    )
    .await
    .unwrap();
    let a = accounts::archive(&pool, a.id, a.version, admin_id)
        .await
        .unwrap();
    // B reprend le rôle.
    mk_account(
        &pool,
        company_id,
        admin_id,
        "1101",
        Some(AccountRole::Receivable),
    )
    .await;

    // Sans le drapeau : 409.
    let res = app
        .client
        .put(app.url(&format!("/api/v1/accounts/{}/reactivate", a.id)))
        .bearer_auth(&token)
        .json(&json!({ "version": a.version }))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 409);
    assert_eq!(
        res.json::<Value>().await.unwrap()["error"]["code"],
        "ACCOUNT_ROLE_ALREADY_ASSIGNED"
    );

    // Avec clearRole : 200, sans rôle.
    let res = app
        .client
        .put(app.url(&format!("/api/v1/accounts/{}/reactivate", a.id)))
        .bearer_auth(&token)
        .json(&json!({ "version": a.version, "clearRole": true }))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 200);
    let body: Value = res.json().await.unwrap();
    assert_eq!(body["active"], true);
    assert_eq!(body["role"], Value::Null, "le rôle doit avoir été retiré");
}

/// Conséquence directe du `active AND` de la colonne générée : le rôle est
/// libéré à l'archivage, donc réactiver après reprise doit échouer **proprement**
/// (409 nommant le repreneur) et jamais en 500 sur le 1062 brut.
#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn reactivate_refused_when_singleton_role_was_taken(pool: MySqlPool) {
    let app = spawn_app(pool.clone()).await;
    let (company_id, admin_id, token) = setup(&pool).await;

    let a = mk_account(
        &pool,
        company_id,
        admin_id,
        "1100",
        Some(AccountRole::Receivable),
    )
    .await;
    accounts::archive(&pool, a, 1, admin_id).await.unwrap();
    let b = mk_account(
        &pool,
        company_id,
        admin_id,
        "1101",
        Some(AccountRole::Receivable),
    )
    .await;

    let res = app
        .client
        .put(app.url(&format!("/api/v1/accounts/{a}/reactivate")))
        .bearer_auth(&token)
        .json(&json!({ "version": 2 }))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 409);
    let body: Value = res.json().await.unwrap();
    assert_eq!(body["error"]["code"], "ACCOUNT_ROLE_ALREADY_ASSIGNED");
    assert_eq!(body["error"]["details"]["accountId"], b);
}

// ============================================================
// RBAC & IDOR
// ============================================================

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn consultation_role_cannot_write(pool: MySqlPool) {
    let app = spawn_app(pool.clone()).await;
    let company_id = create_company(&pool, "Acme SA").await;
    let admin_id = create_user(&pool, "admin", Role::Admin, company_id).await;
    let viewer_id = create_user(&pool, "viewer", Role::Consultation, company_id).await;
    let viewer_token = forge_jwt(viewer_id, role_str(Role::Consultation), company_id);
    let id = mk_account(&pool, company_id, admin_id, "1000", None).await;

    // Lecture autorisée…
    let res = app
        .client
        .get(app.url("/api/v1/accounts"))
        .bearer_auth(&viewer_token)
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 200);

    // …écriture refusée, réactivation comprise.
    for (method, path) in [
        ("POST", "/api/v1/accounts".to_string()),
        ("PUT", format!("/api/v1/accounts/{id}/reactivate")),
    ] {
        let req = if method == "POST" {
            app.client.post(app.url(&path))
        } else {
            app.client.put(app.url(&path))
        };
        let res = req
            .bearer_auth(&viewer_token)
            .json(&json!({ "number": "9", "name": "X", "accountType": "Asset", "version": 1 }))
            .send()
            .await
            .unwrap();
        assert_eq!(
            res.status(),
            403,
            "{method} {path} doit être refusé au rôle Consultation"
        );
    }
}

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn reactivate_is_scoped_to_company(pool: MySqlPool) {
    let app = spawn_app(pool.clone()).await;
    let (_c1, _a1, token) = setup(&pool).await;

    // Compte appartenant à une AUTRE société.
    let other_company = create_company(&pool, "Autre SA").await;
    let other_admin = create_user(&pool, "other-admin", Role::Admin, other_company).await;
    let foreign = mk_account(&pool, other_company, other_admin, "1000", None).await;
    accounts::archive(&pool, foreign, 1, other_admin)
        .await
        .unwrap();

    let res = app
        .client
        .put(app.url(&format!("/api/v1/accounts/{foreign}/reactivate")))
        .bearer_auth(&token)
        .json(&json!({ "version": 2 }))
        .send()
        .await
        .unwrap();
    assert_eq!(
        res.status(),
        404,
        "IDOR : un compte d'une autre société doit être invisible"
    );
}
