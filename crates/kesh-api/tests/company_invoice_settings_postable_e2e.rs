//! Tests E2E HTTP Story 15-5b — `PUT /api/v1/company/invoice-settings` :
//! postabilité des six comptes historiques **à la désignation** (AC10, AC11,
//! #429) et compte créanciers préservé quand il est absent du corps (AC19,
//! #521).
//!
//! Chaque compte refusé ne diffère du compte accepté que par `postable` : même
//! société, actif, bon type. L'assertion porte sur le **code**
//! (`VALIDATION_ERROR`) **et** le message qui nomme le champ — un 400 venu
//! d'ailleurs (type, version) ne passerait pas.
//!
//! Pré-requis : MariaDB démarré localement.

use std::net::SocketAddr;
use std::path::Path;
use std::sync::Arc;

use chrono::{TimeDelta, Utc};
use jsonwebtoken::{Algorithm, EncodingKey, Header};
use kesh_api::auth::jwt::Claims;
use kesh_api::auth::password::hash_password;
use kesh_api::config::Config;
use kesh_api::{AppState, build_router};
use kesh_db::entities::account::AccountType;
use kesh_db::entities::address::StructuredAddress;
use kesh_db::entities::{Language, NewAccount, NewCompany, NewUser, OrgType, Role};
use kesh_db::repositories::{accounts, companies, users};
use serde_json::{Value, json};
use sqlx::MySqlPool;

const TEST_JWT_SECRET: &[u8] = b"test-secret-32-bytes-minimum-test-secret-padding";

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
        "e2e-test-admin-password".to_string(),
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

fn forge_admin_jwt(user_id: i64, company_id: i64) -> String {
    let now = Utc::now().timestamp();
    let claims = Claims {
        sub: user_id.to_string(),
        role: "Admin".to_string(),
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

/// Un des six champs historiques : clé JSON, type de compte attendu, libellé
/// du message, numéros du compte en place et du compte cible.
struct Field {
    key: &'static str,
    account_type: AccountType,
    label: &'static str,
    current_number: &'static str,
    target_number: &'static str,
}

const FIELDS: [Field; 6] = [
    Field {
        key: "defaultReceivableAccountId",
        account_type: AccountType::Asset,
        label: "Compte créance",
        current_number: "1100",
        target_number: "1101",
    },
    Field {
        key: "defaultRevenueAccountId",
        account_type: AccountType::Revenue,
        label: "Compte produit",
        current_number: "3000",
        target_number: "3001",
    },
    Field {
        key: "defaultVatPayableAccountId",
        account_type: AccountType::Liability,
        label: "Compte TVA due",
        current_number: "2200",
        target_number: "2201",
    },
    Field {
        key: "defaultVatRecoverableAccountId",
        account_type: AccountType::Asset,
        label: "Compte TVA récupérable",
        current_number: "1170",
        target_number: "1171",
    },
    Field {
        key: "defaultVatDecompteAccountId",
        account_type: AccountType::Liability,
        label: "Compte décompte TVA",
        current_number: "2201",
        target_number: "2202",
    },
    Field {
        key: "defaultPayableAccountId",
        account_type: AccountType::Liability,
        label: "Compte créanciers",
        current_number: "2000",
        target_number: "2001",
    },
];

struct Ctx {
    company_id: i64,
    jwt: String,
    /// `(compte en place, compte cible)` par champ, dans l'ordre de `FIELDS`.
    accounts: Vec<(i64, i64)>,
}

async fn create_account(
    pool: &MySqlPool,
    company_id: i64,
    user_id: i64,
    number: &str,
    account_type: AccountType,
) -> i64 {
    // Un même numéro peut servir deux champs (2201) : on le réutilise.
    if let Some(id) =
        sqlx::query_scalar::<_, i64>("SELECT id FROM accounts WHERE company_id = ? AND number = ?")
            .bind(company_id)
            .bind(number)
            .fetch_optional(pool)
            .await
            .unwrap()
    {
        return id;
    }
    accounts::create(
        pool,
        user_id,
        NewAccount {
            company_id,
            number: number.into(),
            name: format!("Compte {number}"),
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

async fn set_postable(pool: &MySqlPool, account_id: i64, postable: bool) {
    sqlx::query("UPDATE accounts SET postable = ?, version = version + 1 WHERE id = ?")
        .bind(postable)
        .bind(account_id)
        .execute(pool)
        .await
        .expect("set postable");
}

/// Société + Admin + les comptes des six champs, puis un premier PUT qui
/// désigne les comptes « en place », tous imputables.
async fn setup(pool: &MySqlPool, app: &TestApp) -> Ctx {
    let company_id = companies::create(
        pool,
        NewCompany {
            name: "Réglages SA".into(),
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
    .id;
    let user_id = users::create(
        pool,
        NewUser {
            username: "reglages_admin".into(),
            password_hash: hash_password("password123").unwrap(),
            role: Role::Admin,
            active: true,
            company_id,
            email: None,
        },
    )
    .await
    .unwrap()
    .id;
    let mut ids = Vec::new();
    for f in &FIELDS {
        let current =
            create_account(pool, company_id, user_id, f.current_number, f.account_type).await;
        let target =
            create_account(pool, company_id, user_id, f.target_number, f.account_type).await;
        ids.push((current, target));
    }
    let ctx = Ctx {
        company_id,
        jwt: forge_admin_jwt(user_id, company_id),
        accounts: ids,
    };
    let mut body = current_body(app, &ctx).await;
    for (i, f) in FIELDS.iter().enumerate() {
        body[f.key] = json!(ctx.accounts[i].0);
    }
    let resp = put(app, &ctx, &body).await;
    assert_eq!(resp.status(), 200, "{}", resp.text().await.unwrap());
    ctx
}

/// Le corps d'un PUT qui renvoie les réglages tels qu'ils sont.
async fn current_body(app: &TestApp, ctx: &Ctx) -> Value {
    let resp = app
        .client
        .get(app.url("/api/v1/company/invoice-settings"))
        .bearer_auth(&ctx.jwt)
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let s: Value = resp.json().await.unwrap();
    json!({
        "invoiceNumberFormat": s["invoiceNumberFormat"],
        "defaultReceivableAccountId": s["defaultReceivableAccountId"],
        "defaultRevenueAccountId": s["defaultRevenueAccountId"],
        "defaultVatPayableAccountId": s["defaultVatPayableAccountId"],
        "defaultVatRecoverableAccountId": s["defaultVatRecoverableAccountId"],
        "defaultVatDecompteAccountId": s["defaultVatDecompteAccountId"],
        "defaultSalesJournal": s["defaultSalesJournal"],
        "journalEntryDescriptionTemplate": s["journalEntryDescriptionTemplate"],
        "defaultPayableAccountId": s["defaultPayableAccountId"],
        "version": s["version"],
    })
}

async fn put(app: &TestApp, ctx: &Ctx, body: &Value) -> reqwest::Response {
    app.client
        .put(app.url("/api/v1/company/invoice-settings"))
        .bearer_auth(&ctx.jwt)
        .json(body)
        .send()
        .await
        .unwrap()
}

async fn stored_payable(pool: &MySqlPool, company_id: i64) -> Option<i64> {
    sqlx::query_scalar(
        "SELECT default_payable_account_id FROM company_invoice_settings WHERE company_id = ?",
    )
    .bind(company_id)
    .fetch_one(pool)
    .await
    .unwrap()
}

/// AC10 — pour **chacun** des six champs, désigner un compte non imputable du
/// bon type est refusé : 400 `VALIDATION_ERROR`, message qui nomme le champ
/// et les trois sortes de comptes non imputables (choix C19).
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn each_historical_field_rejects_a_new_non_postable_account(pool: MySqlPool) {
    let app = spawn_app(pool.clone()).await;
    let ctx = setup(&pool, &app).await;
    for (i, f) in FIELDS.iter().enumerate() {
        let target = ctx.accounts[i].1;
        set_postable(&pool, target, false).await;
        let mut body = current_body(&app, &ctx).await;
        body[f.key] = json!(target);
        let resp = put(&app, &ctx, &body).await;
        assert_eq!(resp.status(), 400, "{}", f.key);
        let err: Value = resp.json().await.unwrap();
        assert_eq!(err["error"]["code"], "VALIDATION_ERROR", "{}", f.key);
        assert_eq!(
            err["error"]["message"],
            format!(
                "{} : compte non imputable (compte de regroupement, de résultat ou de clôture)",
                f.label
            ),
            "{}",
            f.key
        );
        // Le compte cible redevient imputable : il peut servir de valeur en
        // place à un autre champ (2201).
        set_postable(&pool, target, true).await;
    }
}

/// AC11 — exemption « inchangé » : les six comptes en place deviennent non
/// imputables, et un PUT qui les renvoie tels quels passe.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn unchanged_non_postable_accounts_do_not_block_saving(pool: MySqlPool) {
    let app = spawn_app(pool.clone()).await;
    let ctx = setup(&pool, &app).await;
    for (current, _) in &ctx.accounts {
        set_postable(&pool, *current, false).await;
    }
    let mut body = current_body(&app, &ctx).await;
    body["journalEntryDescriptionTemplate"] = json!("{YEAR}-{INVOICE_NUMBER}");
    let resp = put(&app, &ctx, &body).await;
    assert_eq!(resp.status(), 200, "{}", resp.text().await.unwrap());
}

/// AC19 (#521) — `defaultPayableAccountId` **absent** du corps : le compte
/// créanciers en place est conservé (relu après).
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn absent_payable_account_is_preserved(pool: MySqlPool) {
    let app = spawn_app(pool.clone()).await;
    let ctx = setup(&pool, &app).await;
    let payable = ctx.accounts[5].0;
    assert_eq!(stored_payable(&pool, ctx.company_id).await, Some(payable));

    let mut body = current_body(&app, &ctx).await;
    body.as_object_mut()
        .unwrap()
        .remove("defaultPayableAccountId");
    let resp = put(&app, &ctx, &body).await;
    assert_eq!(resp.status(), 200, "{}", resp.text().await.unwrap());
    assert_eq!(
        stored_payable(&pool, ctx.company_id).await,
        Some(payable),
        "le compte créanciers n'est plus effacé (#521)"
    );
}

/// AC19 — absent du corps et **devenu non imputable** : préservé sans
/// contrôle (on ne refuse pas un enregistrement pour un champ non envoyé).
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn absent_payable_account_is_preserved_without_control(pool: MySqlPool) {
    let app = spawn_app(pool.clone()).await;
    let ctx = setup(&pool, &app).await;
    let payable = ctx.accounts[5].0;
    sqlx::query("UPDATE accounts SET active = FALSE, version = version + 1 WHERE id = ?")
        .bind(payable)
        .execute(&pool)
        .await
        .unwrap();
    let mut body = current_body(&app, &ctx).await;
    body.as_object_mut()
        .unwrap()
        .remove("defaultPayableAccountId");
    let resp = put(&app, &ctx, &body).await;
    assert_eq!(resp.status(), 200, "{}", resp.text().await.unwrap());
    assert_eq!(stored_payable(&pool, ctx.company_id).await, Some(payable));
}

/// AC19 — `null` efface ; une autre valeur valide remplace.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn null_clears_and_value_replaces_payable_account(pool: MySqlPool) {
    let app = spawn_app(pool.clone()).await;
    let ctx = setup(&pool, &app).await;
    let other = ctx.accounts[5].1;

    let mut body = current_body(&app, &ctx).await;
    body["defaultPayableAccountId"] = json!(other);
    let resp = put(&app, &ctx, &body).await;
    assert_eq!(resp.status(), 200, "{}", resp.text().await.unwrap());
    assert_eq!(stored_payable(&pool, ctx.company_id).await, Some(other));

    let mut body = current_body(&app, &ctx).await;
    body["defaultPayableAccountId"] = Value::Null;
    let resp = put(&app, &ctx, &body).await;
    assert_eq!(resp.status(), 200, "{}", resp.text().await.unwrap());
    assert_eq!(stored_payable(&pool, ctx.company_id).await, None);
}
