//! Tests E2E — Story 15-1a-i (#518) : les routes du lettrage manuel.
//!
//! Couvre (AC6, AC10, AC11, AC12, AC14) : `POST /api/v1/letterings` → 201 et sa
//! forme (exercice par ligne) ; `GET /api/v1/letterings/{key}` par clé et par
//! code ; les références invalides → 404 ; les refus de forme (201 lignes, une
//! seule ligne) ; le RBAC (Consultation lit, ne lettre ni ne délettre) ;
//! l'anti-IDOR ; l'audit d'un `POST` par clé d'API (`actor_api_key_id`), et le
//! refus d'une clé `read` (`API_KEY_READ_ONLY`) au `POST` et au `DELETE` ; le
//! `DELETE` → 204 ; les trois champs neufs des lignes d'écriture traversant la
//! frontière HTTP ; et deux `POST` simultanés partageant une ligne — exactement
//! un 201 et un 409, jamais un 500.
//!
//! Les causes de refus de la primitive, leur ordre et la règle des périodes
//! sont éprouvés au dépôt (`crates/kesh-db/tests/letterings.rs`).

mod common;

use std::sync::Arc;

use chrono::{NaiveDate, TimeDelta, Utc};
use common::create_test_company;
use kesh_api::auth::bootstrap::ensure_admin_user;
use kesh_api::config::Config;
use kesh_api::{AppState, build_router};
use kesh_db::entities::account::AccountType;
use kesh_db::entities::journal_entry::Journal;
use kesh_db::entities::{NewAccount, NewFiscalYear, NewJournalEntry, NewJournalEntryLine};
use kesh_db::repositories::{accounts, fiscal_years, journal_entries};
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use serde_json::{Value, json};
use sqlx::MySqlPool;
use std::net::SocketAddr;

const TEST_JWT_SECRET: &[u8] = b"test-secret-32-bytes-minimum-test-secret-padding";
const TEST_ADMIN_PASSWORD: &str = "e2e-test-admin-password";

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
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind");
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

async fn login(app: &TestApp, user: &str, password: &str) -> String {
    let resp = app
        .client
        .post(app.url("/api/v1/auth/login"))
        .json(&json!({ "username": user, "password": password }))
        .send()
        .await
        .unwrap();
    let body: Value = resp.json().await.unwrap();
    body["accessToken"].as_str().unwrap().to_string()
}

/// Le monde d'un test : l'application, le jeton admin, la société, l'exercice
/// de l'année courante, un compte lettrable (actif) et sa contrepartie.
struct Monde {
    app: TestApp,
    token: String,
    company_id: i64,
    fy_id: i64,
    lettrable: i64,
    contrepartie: i64,
}

fn annee() -> i32 {
    Utc::now()
        .date_naive()
        .format("%Y")
        .to_string()
        .parse()
        .unwrap()
}

async fn make_account(pool: &MySqlPool, company_id: i64, number: &str, ty: AccountType) -> i64 {
    accounts::create(
        pool,
        1,
        NewAccount {
            company_id,
            number: number.to_string(),
            name: format!("Compte {number}"),
            account_type: ty,
            parent_id: None,
            role: None,
            postable: true,
        },
    )
    .await
    .expect("compte")
    .id
}

async fn setup(pool: &MySqlPool) -> Monde {
    let app = spawn_app(pool.clone()).await;
    create_test_company(pool).await;
    ensure_admin_user(pool, &test_config()).await.unwrap();
    let token = login(&app, "admin", TEST_ADMIN_PASSWORD).await;
    let company_id: i64 = sqlx::query_scalar("SELECT id FROM companies ORDER BY id LIMIT 1")
        .fetch_one(pool)
        .await
        .unwrap();
    let y = annee();
    let fy = fiscal_years::create(
        pool,
        1,
        NewFiscalYear {
            company_id,
            name: format!("Exercice {y}"),
            start_date: NaiveDate::from_ymd_opt(y, 1, 1).unwrap(),
            end_date: NaiveDate::from_ymd_opt(y, 12, 31).unwrap(),
        },
    )
    .await
    .expect("exercice");
    let lettrable = make_account(pool, company_id, "1100", AccountType::Asset).await;
    let contrepartie = make_account(pool, company_id, "3000", AccountType::Revenue).await;
    Monde {
        app,
        token,
        company_id,
        fy_id: fy.id,
        lettrable,
        contrepartie,
    }
}

/// Une écriture `montant` (> 0 : débit ; < 0 : crédit) sur `compte`, contrepartie
/// sur `contre` ; rend `(id de l'écriture, id de la ligne sur compte)`.
async fn ligne(
    pool: &MySqlPool,
    company_id: i64,
    fy_id: i64,
    compte: i64,
    contre: i64,
    montant: Decimal,
) -> (i64, i64) {
    let (debit, credit) = if montant > Decimal::ZERO {
        (montant, Decimal::ZERO)
    } else {
        (Decimal::ZERO, -montant)
    };
    let mut tx = pool.begin().await.unwrap();
    let created = journal_entries::create_in_tx(
        &mut tx,
        fy_id,
        1,
        NewJournalEntry {
            company_id,
            entry_date: Utc::now().date_naive(),
            journal: Journal::OD,
            description: "Lettrage".into(),
            project_id: None,
            lines: vec![
                NewJournalEntryLine {
                    account_id: compte,
                    debit,
                    credit,
                    project_id: None,
                },
                NewJournalEntryLine {
                    account_id: contre,
                    debit: credit,
                    credit: debit,
                    project_id: None,
                },
            ],
        },
        false,
    )
    .await
    .expect("écriture");
    tx.commit().await.unwrap();
    let id = created
        .lines
        .iter()
        .find(|l| l.account_id == compte)
        .unwrap()
        .id;
    (created.entry.id, id)
}

async fn paire(pool: &MySqlPool, m: &Monde) -> (i64, i64) {
    let (_, a) = ligne(
        pool,
        m.company_id,
        m.fy_id,
        m.lettrable,
        m.contrepartie,
        dec!(100),
    )
    .await;
    let (_, b) = ligne(
        pool,
        m.company_id,
        m.fy_id,
        m.lettrable,
        m.contrepartie,
        dec!(-100),
    )
    .await;
    (a, b)
}

async fn post(app: &TestApp, token: &str, ids: &[i64]) -> (reqwest::StatusCode, Value) {
    let resp = app
        .client
        .post(app.url("/api/v1/letterings"))
        .bearer_auth(token)
        .json(&json!({ "lineIds": ids }))
        .send()
        .await
        .unwrap();
    let status = resp.status();
    (status, resp.json().await.unwrap_or(Value::Null))
}

async fn get(app: &TestApp, token: &str, reference: &str) -> (reqwest::StatusCode, Value) {
    let resp = app
        .client
        .get(app.url(&format!("/api/v1/letterings/{reference}")))
        .bearer_auth(token)
        .send()
        .await
        .unwrap();
    let status = resp.status();
    (status, resp.json().await.unwrap_or(Value::Null))
}

async fn delete(app: &TestApp, token: &str, reference: &str) -> (reqwest::StatusCode, Value) {
    let resp = app
        .client
        .delete(app.url(&format!("/api/v1/letterings/{reference}")))
        .bearer_auth(token)
        .send()
        .await
        .unwrap();
    let status = resp.status();
    (status, resp.json().await.unwrap_or(Value::Null))
}

/// AC6 — `POST` → 201 et sa forme ; `GET` par clé et par code ; `DELETE` → 204 ;
/// AC14 — les lignes d'écriture exposent la marque à travers HTTP.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn post_get_delete_round_trip(pool: MySqlPool) {
    let m = setup(&pool).await;
    let (a, b) = paire(&pool, &m).await;

    let (status, body) = post(&m.app, &m.token, &[b, a]).await;
    assert_eq!(status, 201, "corps : {body}");
    let key = body["key"].as_i64().unwrap();
    assert_eq!(key, a.min(b));
    let code = body["code"].as_str().unwrap().to_string();
    assert_eq!(code, kesh_core::lettering::code_from_key(key as u64));
    assert_eq!(body["origin"], "manual");
    assert_eq!(body["accountId"], m.lettrable);
    let lignes = body["lines"].as_array().unwrap();
    assert_eq!(lignes.len(), 2);
    for l in lignes {
        for champ in [
            "id",
            "entryId",
            "entryNumber",
            "fiscalYearId",
            "fiscalYearName",
            "date",
            "debit",
            "credit",
        ] {
            assert!(l.get(champ).is_some(), "{champ} manquant : {l}");
        }
        assert_eq!(l["fiscalYearId"], m.fy_id);
        assert_eq!(l["fiscalYearName"], format!("Exercice {}", annee()));
    }

    for reference in [key.to_string(), code.clone(), code.to_lowercase()] {
        let (status, lu) = get(&m.app, &m.token, &reference).await;
        assert_eq!(status, 200, "GET {reference}");
        assert_eq!(lu["key"], key);
        assert_eq!(lu["lines"].as_array().unwrap().len(), 2);
    }

    // AC14 — la ligne d'écriture porte la marque, la contrepartie est nulle.
    let entry_id: i64 = sqlx::query_scalar("SELECT entry_id FROM journal_entry_lines WHERE id = ?")
        .bind(a)
        .fetch_one(&pool)
        .await
        .unwrap();
    let detail: Value = m
        .app
        .client
        .get(m.app.url(&format!("/api/v1/journal-entries/{entry_id}")))
        .bearer_auth(&m.token)
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    let lignes = detail["lines"].as_array().unwrap();
    let lettree = lignes.iter().find(|l| l["id"] == a).unwrap();
    assert_eq!(lettree["letteringKey"], key);
    assert_eq!(lettree["letteringCode"], code);
    assert_eq!(lettree["letteringOrigin"], "manual");
    let ouverte = lignes.iter().find(|l| l["id"] != a).unwrap();
    for champ in ["letteringKey", "letteringCode", "letteringOrigin"] {
        assert!(ouverte.get(champ).is_some_and(Value::is_null), "{champ}");
    }

    let (status, _) = delete(&m.app, &m.token, &code).await;
    assert_eq!(status, 204);
    let (status, _) = get(&m.app, &m.token, &key.to_string()).await;
    assert_eq!(status, 404, "délettré : le groupe n'existe plus");
}

/// AC6 / AC2 — toute référence invalide rend 404, au `GET` comme au `DELETE`.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn invalid_references_are_not_found(pool: MySqlPool) {
    let m = setup(&pool).await;
    let vingt_z = "Z".repeat(20);
    for reference in [
        "0",
        "-1",
        "A1",
        "%C3%84",
        "%C3%9F",
        "99999999999999999999",
        vingt_z.as_str(),
        "123456",
    ] {
        let (status, _) = get(&m.app, &m.token, reference).await;
        assert_eq!(status, 404, "GET {reference}");
        let (status, _) = delete(&m.app, &m.token, reference).await;
        assert_eq!(status, 404, "DELETE {reference}");
    }
}

/// AC6 — refus de forme : 201 lignes → 400 `LETTERING_TOO_MANY_LINES` ; une
/// ligne → 400 `LETTERING_TOO_FEW_LINES` — avant toute lecture (même pour des
/// identifiants inexistants).
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn form_refusals_are_400(pool: MySqlPool) {
    let m = setup(&pool).await;
    let trop: Vec<i64> = (1..=201).collect();
    let (status, body) = post(&m.app, &m.token, &trop).await;
    assert_eq!(status, 400);
    assert_eq!(body["error"]["code"], "LETTERING_TOO_MANY_LINES");
    // Le message lit le plafond par sa variable `{ $max }` du catalogue
    // chargé (revue P1, E-4) — une variable non passée se verrait ici.
    assert_eq!(
        body["error"]["message"],
        format!(
            "Un lettrage réunit au plus {} lignes.",
            kesh_core::lettering::MAX_LINES_PER_GROUP
        )
    );
    let (status, body) = post(&m.app, &m.token, &[999_999]).await;
    assert_eq!(status, 400);
    assert_eq!(body["error"]["code"], "LETTERING_TOO_FEW_LINES");
    let (status, body) = post(&m.app, &m.token, &[999_999, 999_998]).await;
    assert_eq!(status, 404, "{body}");
}

/// AC6, T6 — Consultation lit (200) mais ne lettre ni ne délettre (403) ; le
/// rôle Comptable+ passe sur les MÊMES lignes.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn consultation_reads_but_cannot_letter(pool: MySqlPool) {
    use kesh_db::entities::{NewUser, Role};
    let m = setup(&pool).await;
    let (a, b) = paire(&pool, &m).await;
    let password = "consultation-test-pw-12345";
    kesh_db::repositories::users::create(
        &pool,
        NewUser {
            username: "consultation".into(),
            password_hash: kesh_api::auth::password::hash_password(password).unwrap(),
            role: Role::Consultation,
            active: true,
            company_id: m.company_id,
            email: None,
        },
    )
    .await
    .unwrap();
    let lecteur = login(&m.app, "consultation", password).await;

    let (status, _) = post(&m.app, &lecteur, &[a, b]).await;
    assert_eq!(status, 403, "Consultation ne lettre pas");
    let (status, body) = post(&m.app, &m.token, &[a, b]).await;
    assert_eq!(status, 201);
    let key = body["key"].as_i64().unwrap().to_string();
    let (status, _) = get(&m.app, &lecteur, &key).await;
    assert_eq!(status, 200, "Consultation lit");
    let (status, _) = delete(&m.app, &lecteur, &key).await;
    assert_eq!(status, 403, "Consultation ne délettre pas");
    let (status, _) = delete(&m.app, &m.token, &key).await;
    assert_eq!(status, 204);
}

/// AC11 — deux lignes d'une autre société, une de chaque, la clé d'une autre
/// société : 404, et aucun message ne nomme une ligne étrangère.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn foreign_lines_and_groups_are_not_found(pool: MySqlPool) {
    let m = setup(&pool).await;
    let (a, _) = paire(&pool, &m).await;

    let other: i64 = sqlx::query(
        "INSERT INTO companies (name, address, org_type, accounting_language, instance_language) \
         SELECT CONCAT(name, ' bis'), address, org_type, accounting_language, instance_language \
         FROM companies WHERE id = ?",
    )
    .bind(m.company_id)
    .execute(&pool)
    .await
    .unwrap()
    .last_insert_id() as i64;
    let y = annee();
    let ofy = fiscal_years::create(
        &pool,
        1,
        NewFiscalYear {
            company_id: other,
            name: format!("Exercice {y}"),
            start_date: NaiveDate::from_ymd_opt(y, 1, 1).unwrap(),
            end_date: NaiveDate::from_ymd_opt(y, 12, 31).unwrap(),
        },
    )
    .await
    .unwrap();
    let oa = make_account(&pool, other, "1100", AccountType::Asset).await;
    let oc = make_account(&pool, other, "3000", AccountType::Revenue).await;
    let (_, x) = ligne(&pool, other, ofy.id, oa, oc, dec!(100)).await;
    let (_, z) = ligne(&pool, other, ofy.id, oa, oc, dec!(-100)).await;

    for ids in [vec![x, z], vec![a, x]] {
        let (status, body) = post(&m.app, &m.token, &ids).await;
        assert_eq!(status, 404, "{ids:?}");
        let message = body["error"]["message"].as_str().unwrap_or_default();
        assert!(!message.contains(&x.to_string()) && !message.contains(&z.to_string()));
    }

    // Un groupe de l'autre société, posé par la primitive.
    let mut tx = pool.begin().await.unwrap();
    let theirs = kesh_db::repositories::letterings::create_group_in_tx(
        &mut tx,
        other,
        &[x, z],
        kesh_db::repositories::letterings::Origin::Manual,
        kesh_db::repositories::letterings::Mode::Manual,
        kesh_db::repositories::letterings::Actor {
            user_id: 1,
            api_key_id: None,
        },
    )
    .await
    .unwrap();
    tx.commit().await.unwrap();
    for reference in [theirs.key.to_string(), theirs.code.clone()] {
        let (status, _) = get(&m.app, &m.token, &reference).await;
        assert_eq!(status, 404);
        let (status, _) = delete(&m.app, &m.token, &reference).await;
        assert_eq!(status, 404);
    }
    let reste: Option<i64> =
        sqlx::query_scalar("SELECT lettering_key FROM journal_entry_lines WHERE id = ?")
            .bind(x)
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(reste, Some(theirs.key), "le groupe étranger est intact");
}

/// Crée une clé d'API par la route de gestion (session JWT) ; rend son secret.
async fn creer_cle(m: &Monde, name: &str, scope: &str) -> String {
    let resp = m
        .app
        .client
        .post(m.app.url("/api/v1/settings/api-keys"))
        .bearer_auth(&m.token)
        .json(&json!({ "name": name, "scope": scope }))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 201, "création de clé");
    resp.json::<Value>().await.unwrap()["key"]
        .as_str()
        .unwrap()
        .to_string()
}

/// Une clé d'API `read` lit un groupe mais ne lettre ni ne délettre :
/// `403 API_KEY_READ_ONLY` au `POST` et au `DELETE`, et le groupe reste
/// intact (revue P1, E-8 — le cas négatif des routes neuves par clé).
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn a_read_only_key_reads_but_cannot_letter(pool: MySqlPool) {
    let m = setup(&pool).await;
    let (a, b) = paire(&pool, &m).await;
    let lecture = creer_cle(&m, "lecture-15-1a-i", "read").await;

    let (status, body) = post(&m.app, &lecture, &[a, b]).await;
    assert_eq!(status, 403, "{body}");
    assert_eq!(body["error"]["code"], "API_KEY_READ_ONLY");
    let ouvertes: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM journal_entry_lines WHERE id IN (?, ?) AND lettering_key IS NULL",
    )
    .bind(a)
    .bind(b)
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(ouvertes, 2, "le refus n'a rien posé");

    let (status, body) = post(&m.app, &m.token, &[a, b]).await;
    assert_eq!(status, 201, "{body}");
    let key = body["key"].as_i64().unwrap().to_string();
    let (status, lu) = get(&m.app, &lecture, &key).await;
    assert_eq!(status, 200, "une clé read lit : {lu}");
    assert_eq!(lu["key"], body["key"]);
    let (status, body) = delete(&m.app, &lecture, &key).await;
    assert_eq!(status, 403, "{body}");
    assert_eq!(body["error"]["code"], "API_KEY_READ_ONLY");
    let (status, _) = get(&m.app, &m.token, &key).await;
    assert_eq!(status, 200, "le groupe est intact");
}

/// AC10 — un `POST` par clé d'API `read-write` est audité **comme la clé**, avec
/// ses lignes ; la route est ouverte aux clés.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn a_read_write_key_letters_and_is_traced_as_the_key(pool: MySqlPool) {
    let m = setup(&pool).await;
    let (a, b) = paire(&pool, &m).await;
    let cle = creer_cle(&m, "integration-15-1a-i", "read-write").await;
    let key_id: i64 =
        sqlx::query_scalar("SELECT id FROM api_keys WHERE name = 'integration-15-1a-i'")
            .fetch_one(&pool)
            .await
            .unwrap();

    let (status, body) = post(&m.app, &cle, &[a, b]).await;
    assert_eq!(status, 201, "{body}");
    let key = body["key"].as_i64().unwrap();
    let (actor_type, actor_key, details): (String, Option<i64>, Value) = sqlx::query_as(
        "SELECT actor_type, actor_api_key_id, details_json FROM audit_log \
         WHERE entity_type = 'lettering' AND entity_id = ? AND action = 'lettering.created'",
    )
    .bind(key)
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(actor_type, "api_key");
    assert_eq!(actor_key, Some(key_id));
    assert_eq!(details["code"], body["code"]);
    assert_eq!(details["lines"].as_array().unwrap().len(), 2);
    assert_eq!(
        details["lines"][0]["fiscalYearName"],
        format!("Exercice {}", annee())
    );

    let (status, _) = delete(&m.app, &cle, &key.to_string()).await;
    assert_eq!(status, 204);
    let removed: (String, Option<i64>) = sqlx::query_as(
        "SELECT actor_type, actor_api_key_id FROM audit_log \
         WHERE entity_id = ? AND action = 'lettering.removed'",
    )
    .bind(key)
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(removed, ("api_key".to_string(), Some(key_id)));
}

/// Concurrence — deux `POST` simultanés partageant une ligne : exactement un
/// 201 et un 409 `LETTERING_LINE_ALREADY_LETTERED`, **jamais** un 500 (un
/// interblocage doit être absorbé par le rejeu, pas affaiblir le test).
///
/// ⚠️ **Portée** (revue P1, E-7) : ce résultat est aussi celui d'une exécution
/// sérialisée — le test prouve l'absence de 500 et de double pose sous course,
/// pas qu'une attente de verrou ait eu lieu. Aucun test n'observe l'attente de
/// la seconde création sur la lecture `FOR UPDATE` de l'acte 1 (les sondes
/// d'attente de `crates/kesh-db/tests/letterings.rs` portent sur la clôture et
/// sur l'ordre des verrous d'exercice) : angle mort assumé.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn two_simultaneous_posts_sharing_a_line(pool: MySqlPool) {
    let m = setup(&pool).await;
    for _ in 0..5 {
        let (_, commune) = ligne(
            &pool,
            m.company_id,
            m.fy_id,
            m.lettrable,
            m.contrepartie,
            dec!(100),
        )
        .await;
        let (_, x) = ligne(
            &pool,
            m.company_id,
            m.fy_id,
            m.lettrable,
            m.contrepartie,
            dec!(-100),
        )
        .await;
        let (_, y) = ligne(
            &pool,
            m.company_id,
            m.fy_id,
            m.lettrable,
            m.contrepartie,
            dec!(-100),
        )
        .await;
        let (g1, g2) = ([commune, x], [y, commune]);
        let (r1, r2) = tokio::join!(post(&m.app, &m.token, &g1), post(&m.app, &m.token, &g2));
        let mut statuts = vec![r1.0.as_u16(), r2.0.as_u16()];
        statuts.sort_unstable();
        assert_eq!(statuts, vec![201, 409], "{r1:?} / {r2:?}");
        let refus = if r1.0 == 409 { &r1.1 } else { &r2.1 };
        assert_eq!(refus["error"]["code"], "LETTERING_LINE_ALREADY_LETTERED");
    }
}

// ---------------------------------------------------------------------------
// Story 15-1c-0 (#518, AC15) — le `GET` enrichi
// ---------------------------------------------------------------------------

#[path = "../../kesh-db/tests/support/lettering_documents.rs"]
mod lettering_support;

use std::collections::BTreeSet;

fn cles(v: &Value) -> BTreeSet<String> {
    v.as_object()
        .unwrap_or_else(|| panic!("objet attendu : {v}"))
        .keys()
        .cloned()
        .collect()
}

fn ensemble(noms: &[&str]) -> BTreeSet<String> {
    noms.iter().map(|s| s.to_string()).collect()
}

/// Les clés d'une ligne de la réponse du `POST` (15-1a-i) — communes au `GET`.
const CLES_LIGNE_POST: [&str; 8] = [
    "id",
    "entryId",
    "entryNumber",
    "fiscalYearId",
    "fiscalYearName",
    "date",
    "debit",
    "credit",
];

/// Tests 1 et 8 (AC15) — **ensembles exacts** des clés : le `POST` garde sa
/// forme (aucune fuite de l'enrichissement), le `GET` porte les champs d'AC15 ;
/// valeurs d'un groupe `manual` (journal, libellé, pas de pièce, en période
/// ouverte, compte nommé, prévision nulle).
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn post_keeps_its_shape_and_get_carries_the_detail(pool: MySqlPool) {
    let m = setup(&pool).await;
    let (a, b) = paire(&pool, &m).await;

    let (status, cree) = post(&m.app, &m.token, &[a, b]).await;
    assert_eq!(status, 201);
    assert_eq!(
        cles(&cree),
        ensemble(&["key", "code", "origin", "accountId", "lines"])
    );
    for l in cree["lines"].as_array().unwrap() {
        assert_eq!(cles(l), ensemble(&CLES_LIGNE_POST), "ligne du POST : {l}");
    }

    let code = cree["code"].as_str().unwrap();
    let (status, lu) = get(&m.app, &m.token, code).await;
    assert_eq!(status, 200);
    assert_eq!(
        cles(&lu),
        ensemble(&[
            "key",
            "code",
            "origin",
            "accountId",
            "accountNumber",
            "accountName",
            "manualDissolutionBlockedBy",
            "lines",
        ])
    );
    let mut cles_ligne: Vec<&str> = CLES_LIGNE_POST.to_vec();
    cles_ligne.extend([
        "journal",
        "description",
        "document",
        "ownedByDocument",
        "inOpenPeriod",
    ]);
    for (l, base) in lu["lines"]
        .as_array()
        .unwrap()
        .iter()
        .zip(cree["lines"].as_array().unwrap())
    {
        assert_eq!(cles(l), ensemble(&cles_ligne), "ligne du GET : {l}");
        for champ in CLES_LIGNE_POST {
            assert_eq!(l[champ], base[champ], "{champ} commun au POST");
        }
        assert_eq!(l["journal"], "OD");
        assert_eq!(l["description"], "Lettrage");
        assert!(l["document"].is_null());
        assert_eq!(l["ownedByDocument"], false);
        assert_eq!(l["inOpenPeriod"], true);
    }
    assert_eq!(
        (&lu["key"], &lu["origin"], &lu["accountId"]),
        (&cree["key"], &json!("manual"), &json!(m.lettrable))
    );
    assert_eq!(lu["accountNumber"], "1100");
    assert_eq!(lu["accountName"], "Compte 1100");
    assert!(lu["manualDissolutionBlockedBy"].is_null());
}

/// Le monde des pièces : une société de `seed_accounting_company` (prête à
/// régler et à acheter), son serveur et un JWT `Admin` forgé.
struct MondePieces {
    app: TestApp,
    jwt: String,
    s: kesh_db::test_fixtures::SeededCompany,
}

async fn monde_pieces(pool: &MySqlPool) -> MondePieces {
    let s = lettering_support::societe(pool).await;
    let app = spawn_app(pool.clone()).await;
    let now = Utc::now().timestamp();
    let jwt = jsonwebtoken::encode(
        &jsonwebtoken::Header::new(jsonwebtoken::Algorithm::HS256),
        &kesh_api::auth::jwt::Claims {
            sub: s.admin_user_id.to_string(),
            role: "Admin".into(),
            company_id: s.company_id,
            iat: now,
            exp: now + 3600,
        },
        &jsonwebtoken::EncodingKey::from_secret(TEST_JWT_SECRET),
    )
    .unwrap();
    MondePieces { app, jwt, s }
}

/// La clé portée par la ligne de `entry_id` sur `account_id`.
async fn cle_de_ligne(pool: &MySqlPool, entry_id: i64, account_id: i64) -> i64 {
    let cle: Option<i64> = sqlx::query_scalar(
        "SELECT lettering_key FROM journal_entry_lines WHERE entry_id = ? AND account_id = ?",
    )
    .bind(entry_id)
    .bind(account_id)
    .fetch_one(pool)
    .await
    .unwrap();
    cle.expect("montage : ligne lettrée")
}

/// Test 5 (AC15) à HTTP — **la prévision égale le `DELETE`**, par une table :
/// groupe `manual` (204), `document` (409 `LETTERING_IS_DOCUMENT`), paire
/// `reversal` possédée par une facture fournisseur (409
/// `LETTERING_LINE_OWNED_BY_DOCUMENT`, mêmes `details` et même message suffixé
/// qu'avant la refonte — et sans suffixe pour une facture sans numéro), paire
/// d'un règlement client annulé (204), groupe tout
/// en période verrouillée (409 `LETTERING_ALL_LINES_IN_CLOSED_PERIODS`).
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn forecast_equals_delete_over_http(pool: MySqlPool) {
    use kesh_db::repositories::{invoice_settlements_write, supplier_invoices};
    use lettering_support::*;
    let mp = monde_pieces(&pool).await;
    let s = &mp.s;
    let creance = s.accounts["1100"];
    let passif = s.accounts["2000"];

    // Groupe `document` : facture réglée en entier.
    let inv = facture(&pool, s, dec!(100.00), jours_avant(100)).await;
    regler(&pool, s, inv, dec!(100.00), jours_avant(90)).await;
    let document = cle_de(&pool, inv).await.expect("lettrée");
    // Paire `reversal` possédée : facture fournisseur validée puis annulée.
    let achats = achats(&pool, s).await;
    let si = facture_fournisseur(
        &pool,
        s,
        &achats,
        dec!(60.00),
        jours_avant(80),
        Some("FF-12"),
    )
    .await;
    supplier_invoices::cancel(&pool, s.company_id, si, s.admin_user_id)
        .await
        .expect("annulation");
    let possedee = cle_de_ligne(&pool, achat(&pool, si).await, achats.payable).await;
    // La même, sans numéro de facture (revue P1, E LOW-3) : message sans suffixe.
    let si_sans = facture_fournisseur(&pool, s, &achats, dec!(30.00), jours_avant(80), None).await;
    supplier_invoices::cancel(&pool, s.company_id, si_sans, s.admin_user_id)
        .await
        .expect("annulation");
    let possedee_sans = cle_de_ligne(&pool, achat(&pool, si_sans).await, achats.payable).await;
    // Paire `reversal` libre : règlement client annulé.
    let inv2 = facture(&pool, s, dec!(40.00), jours_avant(70)).await;
    let (sid, reglement) = regler(&pool, s, inv2, dec!(40.00), jours_avant(60)).await;
    invoice_settlements_write::cancel_settlement(&pool, s.admin_user_id, s.company_id, inv2, sid)
        .await
        .expect("annulation du règlement");
    let libre = cle_de_ligne(&pool, reglement, creance).await;
    // Deux groupes `manual` sur la créance, l'un ouvert, l'autre à verrouiller.
    let mut manuels = Vec::new();
    for (d1, d2) in [(30, 29), (20, 19)] {
        let (_, l1, _) =
            ecriture_manuelle(&pool, s, creance, passif, dec!(7), jours_avant(d1)).await;
        let (_, _, l2) =
            ecriture_manuelle(&pool, s, passif, creance, dec!(7), jours_avant(d2)).await;
        let (status, g) = post(&mp.app, &mp.jwt, &[l1, l2]).await;
        assert_eq!(status, 201, "{g}");
        manuels.push(g["key"].as_i64().unwrap());
    }

    let cas = [
        (manuels[0], None),
        (document, Some("LETTERING_IS_DOCUMENT")),
        (possedee, Some("LETTERING_LINE_OWNED_BY_DOCUMENT")),
        (possedee_sans, Some("LETTERING_LINE_OWNED_BY_DOCUMENT")),
        (libre, None),
    ];
    for (key, attendu) in cas {
        let (status, lu) = get(&mp.app, &mp.jwt, &key.to_string()).await;
        assert_eq!(status, 200);
        assert_eq!(
            lu["manualDissolutionBlockedBy"].as_str(),
            attendu,
            "prévision du groupe {key} : {lu}"
        );
        let (status, refus) = delete(&mp.app, &mp.jwt, &key.to_string()).await;
        match attendu {
            None => assert_eq!(status, 204, "groupe {key} : {refus}"),
            Some(code) => {
                assert_eq!(status, 409, "groupe {key}");
                assert_eq!(refus["error"]["code"], code, "groupe {key}");
            }
        }
        if key == possedee {
            assert_eq!(
                refus["error"]["details"],
                json!({ "documentId": si, "documentNumber": "FF-12" })
            );
            assert_eq!(
                refus["error"]["message"],
                "Une de ces lignes appartient à une pièce : elle ne se lettre ni ne se délettre \
                 à la main. (FF-12)"
            );
        }
        if key == possedee_sans {
            assert_eq!(
                refus["error"]["details"],
                json!({ "documentId": si_sans, "documentNumber": null })
            );
            assert_eq!(
                refus["error"]["message"],
                "Une de ces lignes appartient à une pièce : elle ne se lettre ni ne se délettre \
                 à la main."
            );
        }
    }

    // Le verrou de période couvre le second groupe manuel (et tout le reste).
    sqlx::query("UPDATE companies SET books_locked_through = ? WHERE id = ?")
        .bind(jours_avant(1))
        .bind(s.company_id)
        .execute(&pool)
        .await
        .unwrap();
    let (_, lu) = get(&mp.app, &mp.jwt, &manuels[1].to_string()).await;
    assert_eq!(
        lu["manualDissolutionBlockedBy"],
        "LETTERING_ALL_LINES_IN_CLOSED_PERIODS"
    );
    assert!(
        lu["lines"]
            .as_array()
            .unwrap()
            .iter()
            .all(|l| l["inOpenPeriod"] == false)
    );
    let (status, refus) = delete(&mp.app, &mp.jwt, &manuels[1].to_string()).await;
    assert_eq!(status, 409);
    assert_eq!(
        refus["error"]["code"],
        "LETTERING_ALL_LINES_IN_CLOSED_PERIODS"
    );
}

/// Test 6 (AC15) à HTTP — **même source que la vue** : la vente d'une facture
/// réglée après `X` est ouverte à `X` ; dans la vue et dans son groupe, ses
/// `document`, `inOpenPeriod`, `journal` et `description` sont égaux — avant et
/// après un verrou posé entre la vente et le règlement.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn detail_agrees_with_open_items_over_http(pool: MySqlPool) {
    use lettering_support::{facture, jours_avant, regler};
    let mp = monde_pieces(&pool).await;
    let s = &mp.s;
    let inv = facture(&pool, s, dec!(100.00), jours_avant(100)).await;
    regler(&pool, s, inv, dec!(100.00), jours_avant(50)).await;
    let x = jours_avant(70);

    for verrou in [false, true] {
        if verrou {
            sqlx::query("UPDATE companies SET books_locked_through = ? WHERE id = ?")
                .bind(jours_avant(60))
                .bind(s.company_id)
                .execute(&pool)
                .await
                .unwrap();
        }
        let resp = mp
            .app
            .client
            .get(mp.app.url(&format!(
                "/api/v1/accounts/{}/open-items?asOf={x}",
                s.accounts["1100"]
            )))
            .bearer_auth(&mp.jwt)
            .send()
            .await
            .unwrap();
        assert_eq!(resp.status(), 200);
        let vue: Value = resp.json().await.unwrap();
        let item = vue["items"]
            .as_array()
            .unwrap()
            .iter()
            .find(|i| i["document"]["type"] == "invoice")
            .unwrap_or_else(|| panic!("la vente est ouverte à X : {vue}"))
            .clone();
        assert_eq!(item["reason"], "letteredAfterAsOf");
        let code = item["letteringCode"].as_str().unwrap();
        let (status, groupe) = get(&mp.app, &mp.jwt, code).await;
        assert_eq!(status, 200);
        let ligne = groupe["lines"]
            .as_array()
            .unwrap()
            .iter()
            .find(|l| l["id"] == item["lineId"])
            .expect("la ligne est dans son groupe");
        for champ in ["document", "inOpenPeriod", "journal", "description"] {
            assert_eq!(ligne[champ], item[champ], "{champ}, verrou {verrou}");
        }
        assert_eq!(ligne["inOpenPeriod"], !verrou, "montage : période");
    }
}
