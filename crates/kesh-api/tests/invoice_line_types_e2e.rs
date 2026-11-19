//! Story 49-1a (#1242) — T9 : l'API des factures connaît le type de ligne ;
//! Story 49-1c — T3 à T6 : elle ouvre `comment` (AC1, AC2, AC6, AC7).
//!
//! Ce que ces tests établissent à la frontière HTTP :
//!
//! - un **corps de la 0.19.0** (sans `lineType`) est accepté et rend la même
//!   réponse, au seul champ `lineType` (`"priced"`) près ;
//! - `comment` est accepté au `POST` et au `PUT` depuis la 49-1c (valeurs
//!   neutres, hors des totaux, hors de la liste blanche des taux de TVA), et
//!   un commentaire saisi par l'API s'imprime et se reprend dans l'avoir ;
//! - `heading` est refusé `400 VALIDATION_ERROR` au `POST` **et** au `PUT`,
//!   **sans aucune écriture** (nombre de factures et de lignes, version
//!   inchangés), jusqu'à la Story 49-3 ;
//! - un champ de montant manquant sur une ligne chiffrée rend `400` nommant la
//!   ligne (changement de contrat, non plus le `422` de l'extracteur) ;
//! - un `lineType` inconnu reste un rejet de l'extracteur, `422` ;
//! - les réponses de lecture d'une facture et d'un avoir portent `lineType`.

use std::net::SocketAddr;
use std::sync::Arc;

use chrono::TimeDelta;
use kesh_api::config::Config;
use kesh_api::{AppState, build_router};
use kesh_db::entities::bank_account::NewBankAccount;
use kesh_db::entities::contact::{ContactType, NewContact};
use kesh_db::repositories::{bank_accounts, contacts};
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
    /// Compte de caisse (1000) de la fixture, contrepartie d'un règlement.
    caisse: i64,
    company_id: i64,
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
        caisse: seeded.accounts["1000"],
        company_id: seeded.company_id,
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

    // Revue P1 (B1/E3/A7) : `"lineType": null` vaut l'absence.
    let mut avec_null = ligne_0_19_0();
    avec_null["lineType"] = Value::Null;
    let (status, created) = m
        .envoyer(
            reqwest::Method::POST,
            "/api/v1/invoices",
            &m.corps(json!([avec_null])),
        )
        .await;
    assert_eq!(status, 201, "lineType null : {created}");
    assert_eq!(created["lines"][0]["lineType"], "priced");
}

/// Les clés d'une ligne de facture en 0.19.0, plus la seule ajoutée par la
/// 0.20.0 : `lineType` (AC12).
const CLES_DE_LIGNE: [&str; 11] = [
    "createdAt",
    "description",
    "id",
    "invoiceId",
    "lineTotal",
    "lineType",
    "position",
    "quantity",
    "revenueAccountId",
    "unitPrice",
    "vatRate",
];

/// Contrôle une ligne rendue : exactement les clés de la 0.19.0 plus
/// `lineType`, et les valeurs du corps envoyé.
fn ligne_conforme(contexte: &str, l: &Value) {
    let mut cles: Vec<&str> = l.as_object().unwrap().keys().map(String::as_str).collect();
    cles.sort_unstable();
    assert_eq!(cles, CLES_DE_LIGNE, "{contexte} : clés de la ligne");
    assert_eq!(l["lineType"], "priced", "{contexte}");
    assert_eq!(l["description"], "Loyer", "{contexte}");
    assert_eq!(l["quantity"], "1.0000", "{contexte}");
    assert_eq!(l["unitPrice"], "1200.0000", "{contexte}");
    assert_eq!(l["vatRate"], "8.10", "{contexte}");
    assert_eq!(l["lineTotal"], "1200.0000", "{contexte}");
    assert_eq!(l["position"], 0, "{contexte}");
}

/// AC12 (revue P1, A4) — à chaque réponse qui porte la facture, la ligne a
/// la forme de la 0.19.0 plus `lineType` : `POST`, `PUT` réussi, `GET`,
/// `validate`, `unvalidate`, règlement.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn every_invoice_response_carries_the_line_type(pool: MySqlPool) {
    let m = monter(&pool).await;
    let (status, created) = m
        .envoyer(
            reqwest::Method::POST,
            "/api/v1/invoices",
            &m.corps(json!([ligne_0_19_0()])),
        )
        .await;
    assert_eq!(status, 201, "{created}");
    ligne_conforme("POST", &created["lines"][0]);
    let id = created["id"].as_i64().unwrap();

    let mut put = m.corps(json!([ligne_0_19_0()]));
    put["paymentTerms"] = json!("30 jours");
    put["version"] = created["version"].clone();
    let (status, modifiee) = m
        .envoyer(
            reqwest::Method::PUT,
            &format!("/api/v1/invoices/{id}"),
            &put,
        )
        .await;
    assert_eq!(status, 200, "{modifiee}");
    assert_ne!(
        modifiee["version"], created["version"],
        "PUT réel, non no-op"
    );
    ligne_conforme("PUT", &modifiee["lines"][0]);
    ligne_conforme(
        "GET",
        &m.lire(&format!("/api/v1/invoices/{id}")).await["lines"][0],
    );

    let chemin_validation = format!("/api/v1/invoices/{id}/validate");
    let vide = json!({});
    let valider = || m.envoyer(reqwest::Method::POST, &chemin_validation, &vide);
    let (status, validee) = valider().await;
    assert_eq!(status, 200, "{validee}");
    ligne_conforme("validate", &validee["lines"][0]);
    let (status, devalidee) = m
        .envoyer(
            reqwest::Method::POST,
            &format!("/api/v1/invoices/{id}/unvalidate"),
            &json!({ "version": validee["version"] }),
        )
        .await;
    assert_eq!(status, 200, "{devalidee}");
    ligne_conforme("unvalidate", &devalidee["lines"][0]);
    let (status, validee) = valider().await;
    assert_eq!(status, 200, "{validee}");

    let (status, reglee) = m
        .envoyer(
            reqwest::Method::POST,
            &format!("/api/v1/invoices/{id}/settlements"),
            &json!({
                "settlementType": "internal_account",
                "accountId": m.caisse,
                "amount": "100.00",
                "settledOn": "2026-06-20",
                "version": validee["version"],
            }),
        )
        .await;
    assert_eq!(status, 200, "{reglee}");
    ligne_conforme("settlements", &reglee["invoice"]["lines"][0]);
}

/// D9 de la 49-1a, D2 de la 49-1c — `heading` refusé `400` au `POST` et au
/// `PUT`, sans écriture ; la version d'une facture ne bouge pas. (`comment`
/// est ouvert depuis la 49-1c : [`comment_is_accepted_at_post_and_put`].)
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn heading_is_refused_without_writing(pool: MySqlPool) {
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

    for t in ["heading"] {
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

// ---------------------------------------------------------------------------
// Story 49-1c — l'ouverture de `comment`
// ---------------------------------------------------------------------------

/// Une ligne chiffrée explicite (`lineType: "priced"`, comme l'envoie l'écran
/// depuis la 49-1c).
fn chiffree(description: &str, prix: &str, taux: &str) -> Value {
    json!({
        "lineType": "priced", "description": description,
        "quantity": "1", "unitPrice": prix, "vatRate": taux,
    })
}

/// Une ligne de commentaire au contrat de D3 : le texte seul.
fn commentaire(texte: &str) -> Value {
    json!({ "lineType": "comment", "description": texte })
}

const MERCI: &str = "Merci de votre confiance";

/// Compte bancaire principal à QR-IBAN : exigé pour rendre le PDF d'une facture.
async fn banque(pool: &MySqlPool, company_id: i64) {
    bank_accounts::upsert_primary(
        pool,
        NewBankAccount {
            company_id,
            bank_name: "UBS".into(),
            iban: "CH9300762011623852957".into(),
            qr_iban: Some("CH4431999123000889012".into()),
            is_primary: true,
        },
    )
    .await
    .unwrap();
}

/// Les sommes d'une facture rendue : HT, TTC et ventilation de TVA.
fn sommes(f: &Value) -> (Value, Value, Value) {
    (
        f["totalAmount"].clone(),
        f["totalTtc"].clone(),
        f["vatBreakdown"].clone(),
    )
}

/// Contrôle la ligne de commentaire rendue : type, texte, valeurs neutres.
fn commentaire_neutre(contexte: &str, l: &Value, texte: &str) {
    assert_eq!(l["lineType"], "comment", "{contexte} : {l}");
    assert_eq!(l["description"], texte, "{contexte}");
    assert_eq!(l["quantity"], "0.0000", "{contexte}");
    assert_eq!(l["unitPrice"], "0.0000", "{contexte}");
    assert_eq!(l["vatRate"], "0.00", "{contexte}");
    assert_eq!(l["lineTotal"], "0.0000", "{contexte}");
    assert_eq!(l["revenueAccountId"], Value::Null, "{contexte}");
}

/// Les positions des lignes rendues, dans l'ordre de la réponse.
fn positions(f: &Value) -> Vec<i64> {
    f["lines"]
        .as_array()
        .unwrap()
        .iter()
        .map(|l| l["position"].as_i64().unwrap())
        .collect()
}

/// **T3 (Story 49-1c, AC1)** — `[A, commentaire, B]` : `201`, trois lignes à
/// leur place, le commentaire aux valeurs neutres, et **les mêmes** sommes que
/// `[A, B]` ; le `PUT` qui renvoie les lignes avec leur type rend `200` et garde
/// le commentaire ; le `GET` relit le type.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn comment_is_accepted_at_post_and_put(pool: MySqlPool) {
    let m = monter(&pool).await;
    let a = || chiffree("Loyer", "1200.00", "8.10");
    let b = || chiffree("Charges", "150.00", "2.60");
    let (status, avec) = m
        .envoyer(
            reqwest::Method::POST,
            "/api/v1/invoices",
            &m.corps(json!([a(), commentaire(MERCI), b()])),
        )
        .await;
    assert_eq!(status, 201, "POST [A, commentaire, B] : {avec}");
    let (status, sans) = m
        .envoyer(
            reqwest::Method::POST,
            "/api/v1/invoices",
            &m.corps(json!([a(), b()])),
        )
        .await;
    assert_eq!(status, 201, "POST [A, B] : {sans}");

    assert_eq!(avec["lines"].as_array().unwrap().len(), 3);
    assert_eq!(positions(&avec), vec![0, 1, 2], "trois lignes à la suite");
    assert_eq!(avec["lines"][0]["description"], "Loyer");
    commentaire_neutre("POST", &avec["lines"][1], MERCI);
    assert_eq!(avec["lines"][2]["description"], "Charges");
    assert_eq!(
        sommes(&avec),
        sommes(&sans),
        "le commentaire hors des sommes"
    );
    assert_ne!(
        avec["totalAmount"], "0.0000",
        "montage : des sommes non nulles"
    );

    let id = avec["id"].as_i64().unwrap();
    let mut put = m.corps(json!([a(), commentaire("Merci, et à bientôt"), b()]));
    put["version"] = avec["version"].clone();
    let (status, modifiee) = m
        .envoyer(
            reqwest::Method::PUT,
            &format!("/api/v1/invoices/{id}"),
            &put,
        )
        .await;
    assert_eq!(status, 200, "PUT : {modifiee}");
    commentaire_neutre("PUT", &modifiee["lines"][1], "Merci, et à bientôt");
    assert_eq!(sommes(&modifiee), sommes(&sans), "PUT : sommes inchangées");

    let lu = m.lire(&format!("/api/v1/invoices/{id}")).await;
    commentaire_neutre("GET", &lu["lines"][1], "Merci, et à bientôt");
    assert_eq!(lu["lines"][0]["lineType"], "priced");
    assert_eq!(lu["lines"][2]["lineType"], "priced");
}

/// **T4 (Story 49-1c, AC2, D4)** — le taux neutre d'un commentaire ne passe pas
/// la liste blanche : dans une société dont le 0 % est **désactivé**, `POST` et
/// `PUT` d'une facture `[A à 8.10 %, commentaire]` passent ; une ligne
/// **chiffrée** à 0 % reste refusée ; une facture de commentaires seuls
/// s'enregistre en brouillon.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn comment_ignores_the_vat_rate_whitelist(pool: MySqlPool) {
    let m = monter(&pool).await;
    let desactives =
        sqlx::query("UPDATE vat_rates SET active = FALSE WHERE company_id = ? AND rate = 0")
            .bind(m.company_id)
            .execute(&pool)
            .await
            .unwrap()
            .rows_affected();
    assert_eq!(
        desactives, 1,
        "montage : le 0 % de la société est désactivé"
    );

    let lignes = json!([chiffree("Loyer", "1200.00", "8.10"), commentaire(MERCI)]);
    let (status, creee) = m
        .envoyer(
            reqwest::Method::POST,
            "/api/v1/invoices",
            &m.corps(lignes.clone()),
        )
        .await;
    assert_eq!(status, 201, "POST à commentaire, 0 % désactivé : {creee}");
    let id = creee["id"].as_i64().unwrap();
    let mut put = m.corps(lignes);
    put["paymentTerms"] = json!("30 jours");
    put["version"] = creee["version"].clone();
    let (status, modifiee) = m
        .envoyer(
            reqwest::Method::PUT,
            &format!("/api/v1/invoices/{id}"),
            &put,
        )
        .await;
    assert_eq!(status, 200, "PUT à commentaire, 0 % désactivé : {modifiee}");

    let (status, err) = m
        .envoyer(
            reqwest::Method::POST,
            "/api/v1/invoices",
            &m.corps(json!([
                chiffree("Loyer", "1200.00", "0.00"),
                commentaire(MERCI)
            ])),
        )
        .await;
    assert_eq!(status, 400, "chiffrée à 0 % désactivé : {err}");
    assert_eq!(err["error"]["code"], "VALIDATION_ERROR");
    assert!(message(&err).to_lowercase().contains("tva"), "{err}");

    let (status, seuls) = m
        .envoyer(
            reqwest::Method::POST,
            "/api/v1/invoices",
            &m.corps(json!([commentaire(MERCI), commentaire("Deuxième note")])),
        )
        .await;
    assert_eq!(status, 201, "commentaires seuls : {seuls}");
    assert_eq!(seuls["status"], "draft");
    assert_eq!(seuls["totalAmount"], "0.0000");
}

/// Forme décomposée (NFD) : `e` suivi de l'accent aigu combinant U+0301.
const CAFE_NFD: &str = "Cafe\u{0301} offert";
const CAFE_NFC: &str = "Caf\u{00E9} offert";
const THE_NFD: &str = "The\u{0301} offert";
const THE_NFC: &str = "Th\u{00E9} offert";

/// **T5 (Story 49-1c, AC7, D9 — #1401)** — un commentaire et un libellé chiffré
/// saisis en NFD sont relus en NFC au `POST`, puis au `PUT` ; la facture validée
/// imprime `Café offert` pour l'un et l'autre.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn nfd_comment_prints_with_its_accent(pool: MySqlPool) {
    assert_ne!(CAFE_NFD, CAFE_NFC, "montage : les deux formes diffèrent");
    assert_eq!(CAFE_NFD.chars().count(), CAFE_NFC.chars().count() + 1);
    let m = monter(&pool).await;
    banque(&pool, m.company_id).await;

    let (status, creee) = m
        .envoyer(
            reqwest::Method::POST,
            "/api/v1/invoices",
            &m.corps(json!([
                chiffree(CAFE_NFD, "100.00", "8.10"),
                commentaire(CAFE_NFD),
            ])),
        )
        .await;
    assert_eq!(status, 201, "{creee}");
    for i in 0..2 {
        let d = creee["lines"][i]["description"].as_str().unwrap();
        assert_eq!(d, CAFE_NFC, "POST, ligne {i} relue en NFC");
        assert_eq!(d.chars().count(), CAFE_NFC.chars().count());
    }

    let id = creee["id"].as_i64().unwrap();
    let mut put = m.corps(json!([
        chiffree(CAFE_NFD, "100.00", "8.10"),
        commentaire(CAFE_NFD),
        commentaire(THE_NFD),
    ]));
    put["version"] = creee["version"].clone();
    let (status, modifiee) = m
        .envoyer(
            reqwest::Method::PUT,
            &format!("/api/v1/invoices/{id}"),
            &put,
        )
        .await;
    assert_eq!(status, 200, "{modifiee}");
    assert_eq!(
        modifiee["lines"][0]["description"], CAFE_NFC,
        "PUT, chiffrée"
    );
    assert_eq!(
        modifiee["lines"][1]["description"], CAFE_NFC,
        "PUT, commentaire"
    );
    assert_eq!(
        modifiee["lines"][2]["description"], THE_NFC,
        "PUT, second texte"
    );

    let (status, validee) = m
        .envoyer(
            reqwest::Method::POST,
            &format!("/api/v1/invoices/{id}/validate"),
            &json!({}),
        )
        .await;
    assert_eq!(status, 200, "{validee}");
    let pdf = pdf_200(&m, &format!("/api/v1/invoices/{id}/pdf")).await;
    let textes = kesh_qrbill::empreinte::textes(&pdf);
    let cafe = textes.iter().filter(|(_, _, t)| t == CAFE_NFC).count();
    assert_eq!(
        cafe, 2,
        "« Café offert » imprimé pour le libellé chiffré et le commentaire : {textes:#?}"
    );
    assert!(textes.iter().any(|(_, _, t)| t == THE_NFC), "{textes:#?}");
}

/// `GET` d'un PDF : `200` et ses octets.
async fn pdf_200(m: &Monte, chemin: &str) -> Vec<u8> {
    let resp = m
        .app
        .client
        .get(m.app.url(chemin))
        .bearer_auth(&m.token)
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200, "GET {chemin}");
    resp.bytes().await.unwrap().to_vec()
}

/// Colonnes chiffrées du PDF de facture (mm), comme `invoice_pdf_e2e.rs` (49-1b).
const COLONNES_CHIFFREES: [f32; 4] = [110.0, 130.0, 160.0, 180.0];

/// Le commentaire est dessiné à `x = 20` mm, sans texte aux colonnes chiffrées
/// à sa hauteur (49-1b AC3).
fn commentaire_pleine_largeur(contexte: &str, pdf: &[u8], texte: &str) {
    let textes = kesh_qrbill::empreinte::textes(pdf);
    let (x, y, _) = textes
        .iter()
        .find(|(_, _, t)| t == texte)
        .unwrap_or_else(|| panic!("{contexte} : commentaire non imprimé : {textes:#?}"))
        .clone();
    assert!((x - 20.0).abs() < 0.01, "{contexte} : x = {x}");
    for (tx, _, t) in textes.iter().filter(|(_, ty, _)| (ty - y).abs() < 0.01) {
        assert!(
            COLONNES_CHIFFREES.iter().all(|c| (tx - c).abs() >= 0.01),
            "{contexte} : {t:?} à {tx}, à la hauteur du commentaire"
        );
    }
}

/// Les lignes de l'écriture d'une facture validée : `(compte, débit, crédit)`.
async fn ecriture(pool: &MySqlPool, facture: &Value) -> Vec<(i64, String, String)> {
    let entry_id = facture["journalEntryId"]
        .as_i64()
        .expect("facture validée : écriture");
    sqlx::query_as(
        "SELECT account_id, CAST(debit AS CHAR), CAST(credit AS CHAR) \
         FROM journal_entry_lines WHERE entry_id = ? ORDER BY line_order",
    )
    .bind(entry_id)
    .fetch_all(pool)
    .await
    .unwrap()
}

/// **T6 (Story 49-1c, AC6)** — un commentaire saisi **par l'API** entre deux
/// lignes chiffrées : la facture validée l'imprime pleine largeur ; son
/// écriture est celle de `[A, B]` ; l'avoir le reprend à la même position, et
/// son PDF l'imprime.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn comment_typed_at_the_api_reaches_the_pdf_and_the_credit_note(pool: MySqlPool) {
    const NOTE: &str = "Appartement 3 pieces, 2e etage";
    let m = monter(&pool).await;
    banque(&pool, m.company_id).await;
    let a = || chiffree("Loyer", "100.00", "8.10");
    let b = || chiffree("Charges", "50.00", "8.10");

    let mut ids = Vec::new();
    for lignes in [json!([a(), commentaire(NOTE), b()]), json!([a(), b()])] {
        let (status, creee) = m
            .envoyer(reqwest::Method::POST, "/api/v1/invoices", &m.corps(lignes))
            .await;
        assert_eq!(status, 201, "{creee}");
        ids.push(creee["id"].as_i64().unwrap());
    }
    let mut validees = Vec::new();
    for id in &ids {
        let (status, v) = m
            .envoyer(
                reqwest::Method::POST,
                &format!("/api/v1/invoices/{id}/validate"),
                &json!({}),
            )
            .await;
        assert_eq!(status, 200, "{v}");
        validees.push(v);
    }
    let (id_avec, avec, sans) = (ids[0], &validees[0], &validees[1]);
    assert_eq!(avec["lines"][1]["lineType"], "comment");

    let pdf = pdf_200(&m, &format!("/api/v1/invoices/{id_avec}/pdf")).await;
    commentaire_pleine_largeur("facture", &pdf, NOTE);

    let ecriture_avec = ecriture(&pool, avec).await;
    assert!(!ecriture_avec.is_empty(), "montage : écriture lue");
    assert_eq!(
        ecriture_avec,
        ecriture(&pool, sans).await,
        "écriture de [A, B]"
    );

    let (status, avoir) = m
        .envoyer(
            reqwest::Method::POST,
            "/api/v1/credit-notes",
            &json!({ "invoiceId": id_avec, "date": "2026-07-01" }),
        )
        .await;
    assert!(status == 200 || status == 201, "{status} {avoir}");
    let lignes = avoir["lines"].as_array().expect("lignes de l'avoir");
    assert_eq!(lignes.len(), 3, "{avoir}");
    assert_eq!(lignes[1]["lineType"], "comment", "{avoir}");
    assert_eq!(lignes[1]["description"], NOTE);
    assert_eq!(lignes[1]["position"], avec["lines"][1]["position"]);
    let avoir_id = avoir["id"].as_i64().expect("id de l'avoir");
    let pdf_avoir = pdf_200(&m, &format!("/api/v1/credit-notes/{avoir_id}/pdf")).await;
    assert!(
        kesh_qrbill::empreinte::textes(&pdf_avoir)
            .iter()
            .any(|(x, _, t)| t == NOTE && (x - 20.0).abs() < 0.01),
        "le PDF de l'avoir imprime le commentaire à x = 20"
    );
}
