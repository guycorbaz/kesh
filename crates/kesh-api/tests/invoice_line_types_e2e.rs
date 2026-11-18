//! Story 49-1a (#1242) — T9 : l'API des factures connaît le type de ligne et
//! n'en ouvre aucun de neuf (AC11, AC12, D8, D9).
//!
//! Ce que ces tests établissent à la frontière HTTP :
//!
//! - un **corps de la 0.19.0** (sans `lineType`) est accepté et rend la même
//!   réponse, au seul champ `lineType` (`"priced"`) près ;
//! - `comment` et `heading` sont refusés `400 VALIDATION_ERROR` au `POST` **et**
//!   au `PUT`, **sans aucune écriture** (nombre de factures et de lignes, version
//!   inchangés) — la garde est la seule chose qui empêche un commentaire de
//!   s'imprimer `0 × 0.00` avant la Story 49-1b ;
//! - un champ de montant manquant sur une ligne chiffrée rend `400` nommant la
//!   ligne (changement de contrat, non plus le `422` de l'extracteur) ;
//! - un `lineType` inconnu reste un rejet de l'extracteur, `422` ;
//! - les réponses de lecture d'une facture et d'un avoir portent `lineType`.

use std::net::SocketAddr;
use std::sync::Arc;

use chrono::TimeDelta;
use kesh_api::config::Config;
use kesh_api::{AppState, build_router};
use kesh_db::entities::contact::{ContactType, NewContact};
use kesh_db::repositories::contacts;
use kesh_db::test_fixtures::seed_accounting_company;
use serde_json::{Value, json};
use sqlx::MySqlPool;

const TEST_JWT_SECRET: &[u8] = b"test-secret-32-bytes-minimum-test-secret-padding";
const TEST_ADMIN_PASSWORD: &str = "admin123";

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
            std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .parent()
                .unwrap()
                .join("kesh-i18n/locales")
                .as_path(),
        )
        .expect("load test i18n"),
    );
    kesh_api::errors::init_error_i18n(i18n.clone(), config.locale);
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
    TestApp {
        base_url: format!("http://{addr}"),
        client: reqwest::Client::new(),
    }
}

async fn login(app: &TestApp) -> String {
    let resp = app
        .client
        .post(app.url("/api/v1/auth/login"))
        .json(&json!({ "username": "admin", "password": TEST_ADMIN_PASSWORD }))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200, "login");
    let body: Value = resp.json().await.unwrap();
    body["accessToken"].as_str().unwrap().to_string()
}

struct Monte {
    app: TestApp,
    token: String,
    contact_id: i64,
}

async fn monter(pool: &MySqlPool) -> Monte {
    let seeded = seed_accounting_company(pool).await.expect("seed");
    let contact_id = contacts::create(
        pool,
        seeded.admin_user_id,
        NewContact {
            company_id: seeded.company_id,
            contact_type: ContactType::Personne,
            name: "Client Types".into(),
            first_name: None,
            last_name: None,
            is_client: true,
            is_supplier: false,
            address: Some("Rue 1\n1000 Lausanne".into()),
            address_street: None,
            address_building: None,
            address_postal_code: None,
            address_city: None,
            address_country: None,
            email: None,
            phone: None,
            ide_number: None,
            client_number: None,
            default_payment_terms: None,
            default_payment_terms_days: None,
            language: None,
            salutation: kesh_db::entities::contact::Salutation::Neutre,
        },
    )
    .await
    .unwrap()
    .id;
    let app = spawn_app(pool.clone()).await;
    let token = login(&app).await;
    Monte {
        app,
        token,
        contact_id,
    }
}

impl Monte {
    async fn envoyer(&self, methode: reqwest::Method, path: &str, corps: &Value) -> (u16, Value) {
        let resp = self
            .app
            .client
            .request(methode, self.app.url(path))
            .bearer_auth(&self.token)
            .json(corps)
            .send()
            .await
            .unwrap();
        let status = resp.status().as_u16();
        let texte = resp.text().await.unwrap();
        (
            status,
            serde_json::from_str(&texte).unwrap_or(Value::String(texte)),
        )
    }

    async fn lire(&self, path: &str) -> Value {
        let resp = self
            .app
            .client
            .get(self.app.url(path))
            .bearer_auth(&self.token)
            .send()
            .await
            .unwrap();
        assert_eq!(resp.status(), 200, "GET {path}");
        resp.json().await.unwrap()
    }

    fn corps(&self, lines: Value) -> Value {
        json!({ "contactId": self.contact_id, "date": "2026-06-15", "lines": lines })
    }
}

/// Une ligne chiffrée de la forme 0.19.0 : sans `lineType`.
fn ligne_0_19_0() -> Value {
    json!({ "description": "Loyer", "quantity": "1", "unitPrice": "1200.00", "vatRate": "8.10" })
}

/// Nombre de factures et de lignes de la société (preuve de « zéro écriture »).
async fn comptes(pool: &MySqlPool) -> (i64, i64) {
    sqlx::query_as("SELECT (SELECT COUNT(*) FROM invoices), (SELECT COUNT(*) FROM invoice_lines)")
        .fetch_one(pool)
        .await
        .unwrap()
}

fn message(v: &Value) -> String {
    v["error"]["message"]
        .as_str()
        .unwrap_or_default()
        .to_string()
}

/// AC11, AC12 — un corps de la 0.19.0 est accepté ; la réponse porte
/// `lineType: "priced"` et les mêmes montants ; `GET` aussi.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn body_0_19_0_is_accepted_and_reads_back_priced(pool: MySqlPool) {
    let m = monter(&pool).await;
    let (status, created) = m
        .envoyer(
            reqwest::Method::POST,
            "/api/v1/invoices",
            &m.corps(json!([ligne_0_19_0()])),
        )
        .await;
    assert_eq!(status, 201, "{created}");
    let ligne = &created["lines"][0];
    assert_eq!(ligne["lineType"], "priced");
    assert_eq!(ligne["quantity"], "1.0000");
    assert_eq!(ligne["lineTotal"], "1200.0000");
    let id = created["id"].as_i64().unwrap();
    let lu = m.lire(&format!("/api/v1/invoices/{id}")).await;
    assert_eq!(lu["lines"][0]["lineType"], "priced");
}

/// D9 — `comment` et `heading` refusés `400` au `POST` et au `PUT`, sans
/// écriture ; la version d'une facture ne bouge pas.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn unpriced_types_are_refused_without_writing(pool: MySqlPool) {
    let m = monter(&pool).await;
    let (status, created) = m
        .envoyer(
            reqwest::Method::POST,
            "/api/v1/invoices",
            &m.corps(json!([ligne_0_19_0()])),
        )
        .await;
    assert_eq!(status, 201, "{created}");
    let id = created["id"].as_i64().unwrap();
    let version = created["version"].as_i64().unwrap();
    let avant = comptes(&pool).await;

    for t in ["comment", "heading"] {
        let lignes = json!([
            ligne_0_19_0(),
            { "lineType": t, "description": "Note" },
        ]);
        let (status, err) = m
            .envoyer(
                reqwest::Method::POST,
                "/api/v1/invoices",
                &m.corps(lignes.clone()),
            )
            .await;
        assert_eq!(status, 400, "POST {t} : {err}");
        assert_eq!(err["error"]["code"], "VALIDATION_ERROR");
        assert_eq!(
            message(&err),
            format!("Ligne 2 : le type de ligne {t} n'est pas encore disponible")
        );

        let mut put = m.corps(lignes);
        put["version"] = json!(version);
        let (status, err) = m
            .envoyer(
                reqwest::Method::PUT,
                &format!("/api/v1/invoices/{id}"),
                &put,
            )
            .await;
        assert_eq!(status, 400, "PUT {t} : {err}");
        assert!(
            message(&err).contains("n'est pas encore disponible"),
            "{err}"
        );
    }

    assert_eq!(comptes(&pool).await, avant, "aucune écriture");
    let lu = m.lire(&format!("/api/v1/invoices/{id}")).await;
    assert_eq!(
        lu["version"].as_i64().unwrap(),
        version,
        "version inchangée"
    );
}

/// D8 — un montant manquant (ou `null`) sur une ligne chiffrée : `400`
/// nommant la ligne et le champ ; un `lineType` inconnu : `422`.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn missing_amount_is_400_and_unknown_type_is_422(pool: MySqlPool) {
    let m = monter(&pool).await;
    let avant = comptes(&pool).await;
    for (champ, attendu) in [
        ("quantity", "la quantité"),
        ("unitPrice", "le prix unitaire"),
        ("vatRate", "le taux de TVA"),
    ] {
        for absent in [false, true] {
            let mut l = ligne_0_19_0();
            if absent {
                l.as_object_mut().unwrap().remove(champ);
            } else {
                l[champ] = Value::Null;
            }
            let (status, err) = m
                .envoyer(
                    reqwest::Method::POST,
                    "/api/v1/invoices",
                    &m.corps(json!([ligne_0_19_0(), l])),
                )
                .await;
            assert_eq!(status, 400, "{champ} (absent : {absent}) : {err}");
            assert_eq!(err["error"]["code"], "VALIDATION_ERROR");
            assert_eq!(
                message(&err),
                format!("Ligne 2 : {attendu} est obligatoire")
            );
        }
    }
    let mut inconnu = ligne_0_19_0();
    inconnu["lineType"] = json!("subtotal");
    let (status, _) = m
        .envoyer(
            reqwest::Method::POST,
            "/api/v1/invoices",
            &m.corps(json!([inconnu])),
        )
        .await;
    assert_eq!(status, 422, "type inconnu : rejet de l'extracteur");
    assert_eq!(comptes(&pool).await, avant, "aucune écriture");
}

/// AC12 — la réponse de l'avoir porte `lineType` sur chaque ligne.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn credit_note_response_carries_line_type(pool: MySqlPool) {
    let m = monter(&pool).await;
    let (status, created) = m
        .envoyer(
            reqwest::Method::POST,
            "/api/v1/invoices",
            &m.corps(json!([ligne_0_19_0()])),
        )
        .await;
    assert_eq!(status, 201, "{created}");
    let id = created["id"].as_i64().unwrap();
    let (status, validee) = m
        .envoyer(
            reqwest::Method::POST,
            &format!("/api/v1/invoices/{id}/validate"),
            &json!({}),
        )
        .await;
    assert_eq!(status, 200, "{validee}");
    assert_eq!(validee["lines"][0]["lineType"], "priced", "{validee}");
    let (status, avoir) = m
        .envoyer(
            reqwest::Method::POST,
            "/api/v1/credit-notes",
            &json!({ "invoiceId": id, "date": "2026-07-01" }),
        )
        .await;
    assert!(status == 200 || status == 201, "{status} {avoir}");
    let lignes = avoir["lines"].as_array().expect("lignes de l'avoir");
    assert!(!lignes.is_empty());
    assert!(lignes.iter().all(|l| l["lineType"] == "priced"), "{avoir}");
}
