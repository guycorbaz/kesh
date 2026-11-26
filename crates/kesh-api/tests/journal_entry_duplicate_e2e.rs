//! Story 53-1a (#1355) — le refus des écritures saisies en double, **par HTTP** :
//! le contrat (`409 JOURNAL_ENTRY_PROBABLE_DUPLICATE`, `details : {
//! journalEntryId, entryNumber, entryDate, confirmable }`, `confirmNotDuplicate`),
//! le message, l'audit au nom de la clé, l'ordre des refus, le `PUT` qui ignore
//! le champ, et le corps propre (`CreateJournalEntryBody`, `#[serde(flatten)]`)
//! qui laisse la conversion inchangée (D5).
//!
//! La règle elle-même est prouvée au dépôt
//! (`kesh-db/tests/journal_entry_duplicate.rs`).

use std::net::SocketAddr;
use std::sync::Arc;

use chrono::TimeDelta;
use kesh_api::config::Config;
use kesh_api::routes::journal_entries::{
    CreateConversionRequest, CreateJournalEntryBody, CreateJournalEntryRequest,
};
use kesh_api::{AppState, build_router};
use kesh_db::test_fixtures::{SeededCompany, seed_accounting_company};
use serde_json::{Value, json};
use sqlx::MySqlPool;

const TEST_JWT_SECRET: &[u8] = b"test-secret-32-bytes-minimum-test-secret-padding";
const TEST_ADMIN_PASSWORD: &str = "admin123";

/// Le texte du refus (fr-CH), tel que le catalogue le porte (D7).
const TEXTE_REFUS: &str = "Une écriture identique existe déjà à cette date (même journal, même libellé, mêmes comptes et montants). Si c’est la même, ne l’enregistrez pas une seconde fois ; sinon, confirmez que ce n’est pas un doublon.";

struct Ctx {
    seeded: SeededCompany,
    base_url: String,
    client: reqwest::Client,
    admin: String,
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

async fn setup(pool: &MySqlPool) -> Ctx {
    let seeded = seed_accounting_company(pool).await.expect("seed");
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
    let state =
        AppState::new_for_tests(pool.clone(), Arc::new(config), Arc::new(rate_limiter), i18n);
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
    let base_url = format!("http://{addr}");
    let client = reqwest::Client::new();
    let resp = client
        .post(format!("{base_url}/api/v1/auth/login"))
        .json(&json!({ "username": "admin", "password": TEST_ADMIN_PASSWORD }))
        .send()
        .await
        .unwrap();
    let body: Value = resp.json().await.unwrap();
    let admin = body["accessToken"].as_str().expect("jeton").to_string();
    Ctx {
        seeded,
        base_url,
        client,
        admin,
    }
}

async fn envoyer(
    c: &Ctx,
    methode: reqwest::Method,
    chemin: &str,
    jeton: &str,
    corps: Option<Value>,
) -> (u16, Value) {
    let mut req = c
        .client
        .request(methode, format!("{}{chemin}", c.base_url))
        .bearer_auth(jeton);
    if let Some(b) = corps {
        req = req.json(&b);
    }
    let resp = req.send().await.unwrap();
    let status = resp.status().as_u16();
    let text = resp.text().await.unwrap();
    let body = if text.is_empty() {
        Value::Null
    } else {
        serde_json::from_str(&text).unwrap_or(Value::String(text))
    };
    (status, body)
}

async fn cle_read_write(c: &Ctx) -> String {
    let (s, b) = envoyer(
        c,
        reqwest::Method::POST,
        "/api/v1/settings/api-keys",
        &c.admin,
        Some(json!({ "name": "clé 53-1a", "scope": "read-write" })),
    )
    .await;
    assert_eq!(s, 201, "{b}");
    b["key"].as_str().unwrap().to_string()
}

/// Le corps de référence : 100.00 de `1000` à `1100`, « Vente 1 », `Banque`.
fn corps(c: &Ctx) -> Value {
    json!({
        "entryDate": "2026-03-02",
        "journal": "Banque",
        "description": "Vente 1",
        "lines": [
            { "accountId": c.seeded.accounts["1000"], "debit": "100.00", "credit": "0" },
            { "accountId": c.seeded.accounts["1100"], "debit": "0", "credit": "100.00" },
        ]
    })
}

fn avec(mut corps: Value, champ: &str, valeur: Value) -> Value {
    corps[champ] = valeur;
    corps
}

async fn post(c: &Ctx, jeton: &str, corps: Value) -> (u16, Value) {
    envoyer(
        c,
        reqwest::Method::POST,
        "/api/v1/journal-entries",
        jeton,
        Some(corps),
    )
    .await
}

fn code(b: &Value) -> &str {
    b["error"]["code"].as_str().unwrap_or("")
}

async fn nombre(pool: &MySqlPool, table: &str) -> i64 {
    sqlx::query_scalar(&format!("SELECT COUNT(*) FROM {table}"))
        .fetch_one(pool)
        .await
        .unwrap()
}

// ---------------------------------------------------------------------------
// AC-1 — le refus, le contrat, le message
// ---------------------------------------------------------------------------

/// AC-1, D7 — `409 JOURNAL_ENTRY_PROBABLE_DUPLICATE`, `details` de l'existante,
/// `confirmable: true`, message français ; rien n'est écrit.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn le_doublon_est_refuse_409_avec_l_existante(pool: MySqlPool) {
    let c = setup(&pool).await;
    let (s, premiere) = post(&c, &c.admin, corps(&c)).await;
    assert_eq!(s, 201, "{premiere}");
    let (n_e, n_l, n_a) = (
        nombre(&pool, "journal_entries").await,
        nombre(&pool, "journal_entry_lines").await,
        nombre(&pool, "audit_log").await,
    );

    let (s, e) = post(&c, &c.admin, corps(&c)).await;
    assert_eq!(
        (s, code(&e)),
        (409, "JOURNAL_ENTRY_PROBABLE_DUPLICATE"),
        "{e}"
    );
    assert_eq!(e["error"]["message"], TEXTE_REFUS);
    assert_eq!(
        e["error"]["details"],
        json!({
            "journalEntryId": premiere["id"],
            "entryNumber": premiere["entryNumber"],
            "entryDate": "2026-03-02",
            "confirmable": true,
        })
    );
    assert_eq!(nombre(&pool, "journal_entries").await, n_e);
    assert_eq!(nombre(&pool, "journal_entry_lines").await, n_l);
    assert_eq!(nombre(&pool, "audit_log").await, n_a);
}

/// Le libellé est comparé après le `trim` de la route (la valeur écrite) : le
/// renvoi d'une saisie à espaces autour est un doublon.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn le_libelle_est_compare_apres_le_trim(pool: MySqlPool) {
    let c = setup(&pool).await;
    let (s, _) = post(&c, &c.admin, corps(&c)).await;
    assert_eq!(s, 201);
    let (s, e) = post(
        &c,
        &c.admin,
        avec(corps(&c), "description", json!("  Vente 1  ")),
    )
    .await;
    assert_eq!(
        (s, code(&e)),
        (409, "JOURNAL_ENTRY_PROBABLE_DUPLICATE"),
        "{e}"
    );
}

// ---------------------------------------------------------------------------
// AC-4 — la confirmation, l'audit, le champ mal typé, le PUT
// ---------------------------------------------------------------------------

/// AC-4 — `confirmNotDuplicate: true` enregistre ; l'audit
/// `journal_entry.duplicate_confirmed` est au nom de la **clé d'API** quand
/// c'est elle qui appelle, avec l'existante en détail.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn la_confirmation_s_inscrit_au_nom_de_la_cle(pool: MySqlPool) {
    let c = setup(&pool).await;
    let cle = cle_read_write(&c).await;
    let (s, premiere) = post(&c, &cle, corps(&c)).await;
    assert_eq!(s, 201, "{premiere}");

    let (s, e) = post(&c, &cle, corps(&c)).await;
    assert_eq!(
        (s, code(&e)),
        (409, "JOURNAL_ENTRY_PROBABLE_DUPLICATE"),
        "{e}"
    );
    let (s, seconde) = post(
        &c,
        &cle,
        avec(corps(&c), "confirmNotDuplicate", json!(true)),
    )
    .await;
    assert_eq!(s, 201, "{seconde}");

    let (cle_audit, details): (Option<i64>, Value) = sqlx::query_as(
        "SELECT actor_api_key_id, details_json FROM audit_log \
         WHERE action = 'journal_entry.duplicate_confirmed' AND entity_id = ?",
    )
    .bind(seconde["id"].as_i64().unwrap())
    .fetch_one(&pool)
    .await
    .unwrap();
    assert!(cle_audit.is_some(), "la confirmation est au nom de la clé");
    assert_eq!(details["journalEntryId"], premiere["id"]);
    assert_eq!(details["entryNumber"], premiere["entryNumber"]);
}

/// AC-4 — `confirmNotDuplicate: true` **sans doublon** n'écrit rien de plus.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn la_confirmation_sans_doublon_n_ecrit_pas_d_audit(pool: MySqlPool) {
    let c = setup(&pool).await;
    let (s, b) = post(
        &c,
        &c.admin,
        avec(corps(&c), "confirmNotDuplicate", json!(true)),
    )
    .await;
    assert_eq!(s, 201, "{b}");
    assert_eq!(
        nombre(
            &pool,
            "audit_log WHERE action = 'journal_entry.duplicate_confirmed'"
        )
        .await,
        0
    );
}

/// AC-4, D5 — les codes rendus pour un `confirmNotDuplicate` mal typé, `null`
/// et un champ inconnu (constatés ; l'extracteur les rend, aucun n'écrit).
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn le_champ_mal_type_est_refuse_par_l_extracteur(pool: MySqlPool) {
    let c = setup(&pool).await;
    let (s, b) = post(
        &c,
        &c.admin,
        avec(corps(&c), "confirmNotDuplicate", json!("oui")),
    )
    .await;
    assert_eq!(s, 422, "mal typé : {b}");
    assert_eq!(nombre(&pool, "journal_entries").await, 0);

    let (s_null, b_null) = post(
        &c,
        &c.admin,
        avec(corps(&c), "confirmNotDuplicate", Value::Null),
    )
    .await;
    assert_eq!(s_null, 422, "null : {b_null} (un booléen n'est pas nul)");
    assert_eq!(nombre(&pool, "journal_entries").await, 0);

    // Un champ inconnu est ignoré en silence, comme avant la story.
    let (s, b) = post(&c, &c.admin, avec(corps(&c), "champInconnu", json!(1))).await;
    assert_eq!(s, 201, "{b}");
}

/// AC-4 — au `PUT`, `confirmNotDuplicate` est **ignoré** : la modification
/// d'une écriture en une écriture identique à une autre n'est pas contrôlée
/// (#1581), avec ou sans le champ.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn le_put_ignore_le_champ(pool: MySqlPool) {
    let c = setup(&pool).await;
    let (s, premiere) = post(&c, &c.admin, corps(&c)).await;
    assert_eq!(s, 201, "{premiere}");
    let (s, autre) = post(
        &c,
        &c.admin,
        avec(corps(&c), "description", json!("Vente 2")),
    )
    .await;
    assert_eq!(s, 201, "{autre}");

    // La seconde devient identique à la première : accepté, avec le champ.
    let mut put = corps(&c);
    put["version"] = autre["version"].clone();
    put["confirmNotDuplicate"] = json!(false);
    let (s, r) = envoyer(
        &c,
        reqwest::Method::PUT,
        &format!("/api/v1/journal-entries/{}", autre["id"]),
        &c.admin,
        Some(put),
    )
    .await;
    assert_eq!(s, 200, "{r}");
}

// ---------------------------------------------------------------------------
// AC-5 — l'ordre des refus, par HTTP
// ---------------------------------------------------------------------------

/// AC-5 — identique **et** période verrouillée, compte non imputable, exercice
/// clos : le refus de doublon vient après.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn les_autres_refus_precedent_le_doublon(pool: MySqlPool) {
    let c = setup(&pool).await;
    let (s, b) = post(&c, &c.admin, corps(&c)).await;
    assert_eq!(s, 201, "{b}");

    sqlx::query("UPDATE accounts SET postable = FALSE WHERE id = ?")
        .bind(c.seeded.accounts["1000"])
        .execute(&pool)
        .await
        .unwrap();
    let (s, e) = post(&c, &c.admin, corps(&c)).await;
    assert_eq!((s, code(&e)), (400, "ACCOUNT_NOT_POSTABLE"), "{e}");
    sqlx::query("UPDATE accounts SET postable = TRUE WHERE id = ?")
        .bind(c.seeded.accounts["1000"])
        .execute(&pool)
        .await
        .unwrap();

    sqlx::query("UPDATE companies SET books_locked_through = '2026-03-31' WHERE id = ?")
        .bind(c.seeded.company_id)
        .execute(&pool)
        .await
        .unwrap();
    let (s, e) = post(&c, &c.admin, corps(&c)).await;
    assert_eq!(code(&e), "PERIOD_LOCKED", "{s} {e}");
    sqlx::query("UPDATE companies SET books_locked_through = NULL WHERE id = ?")
        .bind(c.seeded.company_id)
        .execute(&pool)
        .await
        .unwrap();

    sqlx::query("UPDATE fiscal_years SET status = 'Closed' WHERE id = ?")
        .bind(c.seeded.fiscal_year_id)
        .execute(&pool)
        .await
        .unwrap();
    let (s, e) = post(&c, &c.admin, corps(&c)).await;
    assert_eq!(code(&e), "FISCAL_YEAR_CLOSED", "{s} {e}");
}

// ---------------------------------------------------------------------------
// D5 — le corps propre : `flatten` à l'identique, conversion inchangée
// ---------------------------------------------------------------------------

/// D5, V3-2 — un **corps complet** (ligne en devise avec `foreign`, `projectId`,
/// énumération `journal`, date) se désérialise à l'identique par le chemin
/// `flatten` et par `CreateJournalEntryRequest` seul (le chemin d'avant).
#[test]
fn un_corps_complet_se_deserialise_a_l_identique() {
    let complet = json!({
        "entryDate": "2026-03-02",
        "journal": "Achats",
        "description": "Achat en dollars",
        "lines": [
            {
                "accountId": 7, "debit": "90.00", "credit": "0", "projectId": 3,
                "foreign": { "currency": "USD", "amount": "100", "rate": "0.90",
                             "rateSource": "manual", "side": "debit" }
            },
            { "accountId": 8, "debit": "0", "credit": "90.00" }
        ]
    });
    let avant: CreateJournalEntryRequest = serde_json::from_value(complet.clone()).unwrap();
    let apres: CreateJournalEntryBody = serde_json::from_value(complet.clone()).unwrap();
    assert_eq!(format!("{avant:?}"), format!("{:?}", apres.entry));
    assert!(!apres.confirm_not_duplicate, "absent = faux");

    let mut confirme = complet;
    confirme["confirmNotDuplicate"] = json!(true);
    let apres: CreateJournalEntryBody = serde_json::from_value(confirme).unwrap();
    assert!(apres.confirm_not_duplicate);
}

/// AC-4 (M13c) — la conversion garde son comportement d'avant : un
/// `entries[0].confirmNotDuplicate: "oui"` ne devient pas un refus de
/// désérialisation (le champ n'est **pas** dans `CreateJournalEntryRequest`).
#[test]
fn la_conversion_ignore_le_champ_dans_ses_entrees() {
    let entree = |extra: Value| {
        let mut e = json!({
            "entryDate": "2026-03-02", "journal": "OD", "description": "Conversion",
            "lines": [
                { "accountId": 1, "debit": "10.00", "credit": "0" },
                { "accountId": 2, "debit": "0", "credit": "10.00" }
            ]
        });
        e["confirmNotDuplicate"] = extra;
        e
    };
    let corps = json!({ "entries": [entree(json!("oui")), entree(json!(true))] });
    let r: CreateConversionRequest =
        serde_json::from_value(corps).expect("le champ est ignoré par la conversion");
    assert_eq!(r.entries.len(), 2);
}
