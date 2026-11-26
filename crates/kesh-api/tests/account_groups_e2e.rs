//! Story 46-5a — `POST /api/v1/accounts/groups` à travers la frontière HTTP :
//! la réponse `200 { created, reused, moved }` au format de `GET /accounts`, la
//! fermeture à toute clé d'API, les rôles, les refus de forme et l'étape
//! (`details.groupStep`) ajoutée aux refus existants sans perdre leurs
//! propres `details`.
//!
//! Harnais repris de `accounts_e2e.rs` (JWT forgé, `spawn_app` éphémère).

mod common;

use std::net::SocketAddr;
use std::path::Path;
use std::sync::Arc;

use chrono::{Datelike, NaiveDate, TimeDelta, Utc};
use jsonwebtoken::{Algorithm, EncodingKey, Header};
use kesh_api::auth::jwt::Claims;
use kesh_api::auth::password::hash_password;
use kesh_api::config::Config;
use kesh_api::{AppState, build_router};
use kesh_db::entities::account::AccountType;
use kesh_db::entities::address::StructuredAddress;
use kesh_db::entities::journal_entry::Journal;
use kesh_db::entities::{
    Language, NewAccount, NewCompany, NewFiscalYear, NewJournalEntry, NewJournalEntryLine, NewUser,
    OrgType, Role,
};
use kesh_db::repositories::{accounts, companies, fiscal_years, journal_entries, users};
use rust_decimal_macros::dec;
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
        "mysql://test".into(),
        "admin".into(),
        "e2e-test-password".into(),
        std::str::from_utf8(TEST_JWT_SECRET).unwrap().to_string(),
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

struct Ctx {
    company_id: i64,
    admin_id: i64,
    jwt: String,
}

async fn setup(pool: &MySqlPool) -> Ctx {
    let company_id = create_company(pool, "Acme SA").await;
    let admin_id = create_user(pool, "admin", Role::Admin, company_id).await;
    let jwt = forge_jwt(admin_id, "Admin", company_id);
    Ctx {
        company_id,
        admin_id,
        jwt,
    }
}

/// Un compte d'actif créé par le dépôt, racine ou sous `parent`.
async fn compte(
    pool: &MySqlPool,
    c: &Ctx,
    company_id: i64,
    numero: &str,
    parent: Option<i64>,
) -> i64 {
    accounts::create(
        pool,
        c.admin_id,
        NewAccount::new(
            company_id,
            numero,
            format!("Compte {numero}"),
            AccountType::Asset,
            parent,
        ),
    )
    .await
    .unwrap()
    .id
}

/// `{ id, version }` d'un compte, tel que l'écran l'enverrait.
async fn membre(pool: &MySqlPool, id: i64) -> Value {
    let a = accounts::find_by_id(pool, id).await.unwrap().unwrap();
    json!({ "id": id, "version": a.version })
}

/// Une écriture du jour de l'horloge de test (lu une fois) entre `debit` et `credit`, dans un
/// exercice ouvert de l'année.
async fn ecriture(pool: &MySqlPool, c: &Ctx, debit: i64, credit: i64) {
    let jour: NaiveDate = common::jour::aujourd_hui_test();
    let year = jour.year();
    let fy = fiscal_years::create(
        pool,
        c.admin_id,
        NewFiscalYear {
            company_id: c.company_id,
            name: format!("Exercice {year}"),
            start_date: NaiveDate::from_ymd_opt(year, 1, 1).unwrap(),
            end_date: NaiveDate::from_ymd_opt(year, 12, 31).unwrap(),
        },
    )
    .await
    .expect("exercice")
    .id;
    journal_entries::create(
        pool,
        fy,
        kesh_db::entities::audit_log::Acteur::utilisateur(c.admin_id),
        NewJournalEntry {
            company_id: c.company_id,
            entry_date: jour,
            journal: Journal::OD,
            description: "Écriture de test 46-5a".into(),
            project_id: None,
            lines: vec![
                NewJournalEntryLine {
                    account_id: debit,
                    debit: dec!(100),
                    credit: dec!(0),
                    project_id: None,
                    foreign: None,
                },
                NewJournalEntryLine {
                    account_id: credit,
                    debit: dec!(0),
                    credit: dec!(100),
                    project_id: None,
                    foreign: None,
                },
            ],
        },
    )
    .await
    .expect("écriture");
}

async fn post(app: &TestApp, auth: &str, body: &Value) -> (u16, Value) {
    let resp = app
        .client
        .post(app.url("/api/v1/accounts/groups"))
        .bearer_auth(auth)
        .json(body)
        .send()
        .await
        .unwrap();
    let status = resp.status().as_u16();
    (status, resp.json().await.unwrap_or(Value::Null))
}

async fn nombre_de_comptes(pool: &MySqlPool, company_id: i64) -> i64 {
    sqlx::query_scalar("SELECT COUNT(*) FROM accounts WHERE company_id = ?")
        .bind(company_id)
        .fetch_one(pool)
        .await
        .unwrap()
}

/// `200` et la forme de la réponse : `created`, `reused`, `moved` au format de
/// `GET /accounts` (`letterable`, `postable`, `version`… ; `currency` et
/// `currencyLocked` absents pour un compte CHF sans écriture).
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn cree_reutilise_et_range_en_200(pool: MySqlPool) {
    let c = setup(&pool).await;
    let app = spawn_app(pool.clone()).await;
    let a1 = compte(&pool, &c, c.company_id, "1031", None).await;
    let a2 = compte(&pool, &c, c.company_id, "1032", None).await;
    let r = compte(&pool, &c, c.company_id, "105", None).await;
    let r1 = compte(&pool, &c, c.company_id, "1051", None).await;

    let body = json!({ "groups": [
        { "kind": "create", "number": " 103 ", "name": " Banque A ", "parentId": null,
          "accounts": [membre(&pool, a1).await, membre(&pool, a2).await] },
        { "kind": "reuse", "accountId": r, "accounts": [membre(&pool, r1).await] },
    ]});
    let (status, json) = post(&app, &c.jwt, &body).await;
    assert_eq!(status, 200, "{json}");
    let created = json["created"].as_array().unwrap();
    assert_eq!(created.len(), 1);
    assert_eq!(
        created[0]["number"], "103",
        "numéro `trim`é comme POST /accounts"
    );
    assert_eq!(created[0]["name"], "Banque A");
    assert_eq!(created[0]["accountType"], "Asset");
    assert_eq!(created[0]["postable"], false);
    assert_eq!(created[0]["role"], Value::Null);
    assert_eq!(created[0]["letterable"], true);
    assert!(created[0].get("currency").is_none());
    assert!(created[0].get("currencyLocked").is_none());
    let reused = json["reused"].as_array().unwrap();
    assert_eq!(reused.len(), 1);
    assert_eq!(
        (reused[0]["id"].as_i64(), &reused[0]["postable"]),
        (Some(r), &json!(false))
    );
    let moved: Vec<i64> = json["moved"]
        .as_array()
        .unwrap()
        .iter()
        .map(|a| a["id"].as_i64().unwrap())
        .collect();
    assert_eq!(moved, vec![a1, a2, r1]);
    let g = created[0]["id"].as_i64().unwrap();
    assert_eq!(json["moved"][0]["parentId"], g);
}

/// Fermée à toute clé d'API : `read-write` → `403 API_KEY_OPERATION_FORBIDDEN`
/// (handler), `read` → `403 API_KEY_READ_ONLY` (couche d'authentification) ;
/// rien n'est écrit. Sonde : la même demande, par la session, passe.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn toute_cle_d_api_est_refusee(pool: MySqlPool) {
    let c = setup(&pool).await;
    let app = spawn_app(pool.clone()).await;
    let a1 = compte(&pool, &c, c.company_id, "1031", None).await;
    let body = json!({ "groups": [
        { "kind": "create", "number": "103", "name": "Banque A",
          "accounts": [membre(&pool, a1).await] },
    ]});
    let avant = nombre_de_comptes(&pool, c.company_id).await;

    for (scope, code) in [
        ("read-write", "API_KEY_OPERATION_FORBIDDEN"),
        ("read", "API_KEY_READ_ONLY"),
    ] {
        let create = app
            .client
            .post(app.url("/api/v1/settings/api-keys"))
            .bearer_auth(&c.jwt)
            .json(&json!({ "name": format!("cle-{scope}"), "scope": scope }))
            .send()
            .await
            .unwrap();
        assert_eq!(create.status(), 201, "montage : clé {scope}");
        let key = create.json::<Value>().await.unwrap()["key"]
            .as_str()
            .unwrap()
            .to_string();
        let (status, json) = post(&app, &key, &body).await;
        assert_eq!(status, 403, "clé {scope} : {json}");
        assert_eq!(json["error"]["code"], code, "clé {scope}");
    }
    assert_eq!(
        nombre_de_comptes(&pool, c.company_id).await,
        avant,
        "rien n'est écrit"
    );

    let (status, json) = post(&app, &c.jwt, &body).await;
    assert_eq!(status, 200, "la session passe : {json}");
}

/// Consultation → `403` (couche des routes Comptable+) ; Comptable passe.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn consultation_refusee_comptable_admise(pool: MySqlPool) {
    let c = setup(&pool).await;
    let app = spawn_app(pool.clone()).await;
    let a1 = compte(&pool, &c, c.company_id, "1031", None).await;
    let body = json!({ "groups": [
        { "kind": "create", "number": "103", "name": "Banque A",
          "accounts": [membre(&pool, a1).await] },
    ]});
    let lecteur = create_user(&pool, "lecteur", Role::Consultation, c.company_id).await;
    let (status, _) = post(
        &app,
        &forge_jwt(lecteur, "Consultation", c.company_id),
        &body,
    )
    .await;
    assert_eq!(status, 403);
    let comptable = create_user(&pool, "compta", Role::Comptable, c.company_id).await;
    let (status, json) = post(
        &app,
        &forge_jwt(comptable, "Comptable", c.company_id),
        &body,
    )
    .await;
    assert_eq!(status, 200, "{json}");
}

/// Refus de forme, avant la transaction : `groups` vide → `VALIDATION_ERROR` ;
/// `accountInTwoGroups`, `groupIsMember`, `groupInTwoProposals`, `empty`,
/// `tooMany` →
/// `ACCOUNT_GROUP_INVALID` avec `groupIndex` et `reason`, sans `groupStep` ;
/// un `parentId` d'une autre société → `VALIDATION_ERROR` avec `groupStep`.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn refus_de_forme(pool: MySqlPool) {
    let c = setup(&pool).await;
    let app = spawn_app(pool.clone()).await;
    let a1 = compte(&pool, &c, c.company_id, "1031", None).await;
    let a2 = compte(&pool, &c, c.company_id, "1032", None).await;
    let a3 = compte(&pool, &c, c.company_id, "105", None).await;
    let m1 = membre(&pool, a1).await;
    let m2 = membre(&pool, a2).await;

    let (status, json) = post(&app, &c.jwt, &json!({ "groups": [] })).await;
    assert_eq!(
        (status, &json["error"]["code"]),
        (400, &json!("VALIDATION_ERROR"))
    );

    let cas = [
        (
            json!([
                { "kind": "create", "number": "103", "name": "A", "accounts": [m1] },
                { "kind": "create", "number": "104", "name": "B", "accounts": [m2, m1] },
            ]),
            (1, "accountInTwoGroups", Some(a1)),
        ),
        (
            json!([
                { "kind": "create", "number": "103", "name": "A", "accounts": [m1] },
                { "kind": "reuse", "accountId": a1, "accounts": [m2] },
            ]),
            (1, "groupIsMember", Some(a1)),
        ),
        (
            json!([
                { "kind": "reuse", "accountId": a3, "accounts": [m1] },
                { "kind": "reuse", "accountId": a3, "accounts": [m2] },
            ]),
            (1, "groupInTwoProposals", Some(a3)),
        ),
        (
            json!([{ "kind": "create", "number": "103", "name": "A", "accounts": [] }]),
            (0, "empty", None),
        ),
        (
            Value::Array(
                (0..101)
                    .map(|i| {
                        json!({ "kind": "create", "number": format!("9{i}"), "name": "G",
                                     "accounts": [{ "id": 1_000_000 + i, "version": 1 }] })
                    })
                    .collect(),
            ),
            (100, "tooMany", None),
        ),
    ];
    for (groups, (index, reason, account)) in cas {
        let (status, json) = post(&app, &c.jwt, &json!({ "groups": groups })).await;
        assert_eq!(status, 400, "{reason} : {json}");
        let e = &json["error"];
        assert_eq!(e["code"], "ACCOUNT_GROUP_INVALID", "{reason}");
        assert_eq!(e["details"]["groupIndex"], index, "{reason}");
        assert_eq!(e["details"]["reason"], reason);
        assert_eq!(e["details"]["accountId"].as_i64(), account, "{reason}");
        assert!(
            e["details"].get("groupStep").is_none(),
            "{reason} : pas enveloppé"
        );
        assert!(!e["message"].as_str().unwrap().is_empty());
    }

    let autre = create_company(&pool, "Autre SA").await;
    let etranger = compte(&pool, &c, autre, "10", None).await;
    let (status, json) = post(
        &app,
        &c.jwt,
        &json!({ "groups": [
            { "kind": "create", "number": "103", "name": "A", "accounts": [m1] },
            { "kind": "create", "number": "104", "name": "B", "parentId": etranger, "accounts": [m2] },
        ]}),
    )
    .await;
    assert_eq!(status, 400, "{json}");
    assert_eq!(json["error"]["code"], "VALIDATION_ERROR");
    assert_eq!(
        json["error"]["details"]["groupStep"],
        json!({ "groupIndex": 1 })
    );
}

/// Une erreur existante garde son code et ses `details`, augmentés de
/// `groupStep` : `ACCOUNT_PARENT_HAS_ENTRIES` d'un parent mouvementé (garde
/// `parentNumber`, étape sans compte), `OPTIMISTIC_LOCK_CONFLICT` d'un membre
/// périmé et `NOT_FOUND` d'un membre d'une autre société (étape avec compte).
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn erreur_existante_portee_avec_son_etape(pool: MySqlPool) {
    let c = setup(&pool).await;
    let app = spawn_app(pool.clone()).await;
    let parent = compte(&pool, &c, c.company_id, "10", None).await;
    let caisse = compte(&pool, &c, c.company_id, "1000", None).await;
    let a1 = compte(&pool, &c, c.company_id, "1031", None).await;
    let a2 = compte(&pool, &c, c.company_id, "1032", None).await;
    ecriture(&pool, &c, parent, caisse).await;
    let m1 = membre(&pool, a1).await;

    let (status, json) = post(
        &app,
        &c.jwt,
        &json!({ "groups": [
            { "kind": "create", "number": "103", "name": "A", "parentId": parent, "accounts": [m1] },
        ]}),
    )
    .await;
    assert_eq!(status, 409, "{json}");
    let e = &json["error"];
    assert_eq!(e["code"], "ACCOUNT_PARENT_HAS_ENTRIES");
    assert_eq!(
        e["details"]["parentNumber"], "10",
        "ses propres `details` gardés"
    );
    assert_eq!(e["details"]["parentAccountId"], parent);
    assert_eq!(e["details"]["groupStep"], json!({ "groupIndex": 0 }));

    let (status, json) = post(
        &app,
        &c.jwt,
        &json!({ "groups": [
            { "kind": "create", "number": "103", "name": "A", "accounts": [m1] },
            { "kind": "create", "number": "104", "name": "B",
              "accounts": [{ "id": a2, "version": 99 }] },
        ]}),
    )
    .await;
    assert_eq!(status, 409, "{json}");
    assert_eq!(json["error"]["code"], "OPTIMISTIC_LOCK_CONFLICT");
    assert_eq!(
        json["error"]["details"]["groupStep"],
        json!({ "groupIndex": 1, "accountId": a2 })
    );

    let autre = create_company(&pool, "Autre SA").await;
    let etranger = compte(&pool, &c, autre, "1033", None).await;
    let (status, json) = post(
        &app,
        &c.jwt,
        &json!({ "groups": [
            { "kind": "create", "number": "103", "name": "A",
              "accounts": [m1, { "id": etranger, "version": 1 }] },
        ]}),
    )
    .await;
    assert_eq!(status, 404, "{json}");
    assert_eq!(json["error"]["code"], "NOT_FOUND");
    assert_eq!(
        json["error"]["details"]["groupStep"],
        json!({ "groupIndex": 0, "accountId": etranger })
    );
    assert_eq!(
        nombre_de_comptes(&pool, c.company_id).await,
        4,
        "aucun groupe créé par les trois refus"
    );
}

/// Epic 46, garde du mainteneur du 2026-11-26 : un compte désigné par un compte
/// bancaire ne devient pas groupe — `409 ACCOUNT_PARENT_BANK_LINKED`, ses
/// `details` (compte et compte bancaire) augmentés de `groupStep`, message qui
/// nomme le compte et le compte bancaire ; rien n'est écrit.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn compte_d_un_compte_bancaire_refuse_en_groupe(pool: MySqlPool) {
    let c = setup(&pool).await;
    let app = spawn_app(pool.clone()).await;
    let lie = compte(&pool, &c, c.company_id, "102", None).await;
    let a1 = compte(&pool, &c, c.company_id, "1021", None).await;
    let banque = sqlx::query(
        "INSERT INTO bank_accounts (company_id, bank_name, description, iban, journal_account_id) \
         VALUES (?, 'Banque Test', 'Courant', 'CH9300762011623852957', ?)",
    )
    .bind(c.company_id)
    .bind(lie)
    .execute(&pool)
    .await
    .unwrap()
    .last_insert_id() as i64;
    let m1 = membre(&pool, a1).await;
    let avant = nombre_de_comptes(&pool, c.company_id).await;

    let (status, json) = post(
        &app,
        &c.jwt,
        &json!({ "groups": [ { "kind": "reuse", "accountId": lie, "accounts": [m1] } ] }),
    )
    .await;
    assert_eq!(status, 409, "{json}");
    let e = &json["error"];
    assert_eq!(e["code"], "ACCOUNT_PARENT_BANK_LINKED");
    assert_eq!(e["details"]["parentAccountId"], lie);
    assert_eq!(e["details"]["parentNumber"], "102");
    assert_eq!(e["details"]["bankAccountId"], banque);
    assert_eq!(e["details"]["bankName"], "Banque Test — Courant");
    assert_eq!(
        e["details"]["groupStep"],
        json!({ "groupIndex": 0, "accountId": a1 })
    );
    let message = e["message"].as_str().unwrap();
    assert!(
        message.contains("102") && message.contains("Banque Test — Courant"),
        "{message}"
    );
    assert_eq!(nombre_de_comptes(&pool, c.company_id).await, avant);
    let relu = accounts::find_by_id(&pool, lie).await.unwrap().unwrap();
    assert!(relu.postable, "rien n'est écrit");
}

/// Revue d'intégration de la PR de l'Epic 46 (H1) : `PUT /accounts/{id}/reactivate`
/// d'un sous-compte archivé dont le parent, repassé imputable, est désigné par un
/// compte bancaire — `409 ACCOUNT_PARENT_BANK_LINKED`, les `details` du
/// changement de parent (sans `groupStep`), un message propre à la réactivation
/// qui nomme le sous-compte, le parent et le compte bancaire ; rien n'est écrit.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn reactivation_sous_un_compte_bancaire_refusee(pool: MySqlPool) {
    let c = setup(&pool).await;
    let app = spawn_app(pool.clone()).await;
    let groupe = compte(&pool, &c, c.company_id, "102", None).await;
    let enfant = compte(&pool, &c, c.company_id, "1021", Some(groupe)).await;
    let v = accounts::find_by_id(&pool, enfant)
        .await
        .unwrap()
        .unwrap()
        .version;
    accounts::archive(&pool, enfant, v, c.admin_id)
        .await
        .unwrap();
    sqlx::query("UPDATE accounts SET postable = TRUE WHERE id = ?")
        .bind(groupe)
        .execute(&pool)
        .await
        .unwrap();
    let banque = sqlx::query(
        "INSERT INTO bank_accounts (company_id, bank_name, description, iban, journal_account_id) \
         VALUES (?, 'Banque Test', 'Courant', 'CH9300762011623852957', ?)",
    )
    .bind(c.company_id)
    .bind(groupe)
    .execute(&pool)
    .await
    .unwrap()
    .last_insert_id() as i64;
    let archive = accounts::find_by_id(&pool, enfant).await.unwrap().unwrap();

    let resp = app
        .client
        .put(app.url(&format!("/api/v1/accounts/{enfant}/reactivate")))
        .bearer_auth(&c.jwt)
        .json(&json!({ "version": archive.version }))
        .send()
        .await
        .unwrap();
    let status = resp.status().as_u16();
    let json: Value = resp.json().await.unwrap_or(Value::Null);
    assert_eq!(status, 409, "{json}");
    let e = &json["error"];
    assert_eq!(e["code"], "ACCOUNT_PARENT_BANK_LINKED");
    assert_eq!(
        e["details"],
        json!({
            "parentAccountId": groupe,
            "parentNumber": "102",
            "bankAccountId": banque,
            "bankName": "Banque Test — Courant",
        })
    );
    let message = e["message"].as_str().unwrap();
    assert!(
        message.contains("1021")
            && message.contains("102")
            && message.contains("Banque Test — Courant")
            && message.contains("reste archivé"),
        "{message}"
    );
    let relu = accounts::find_by_id(&pool, enfant).await.unwrap().unwrap();
    assert!(!relu.active, "le sous-compte reste archivé");
    assert_eq!(relu.version, archive.version, "rien n'est écrit");
}
