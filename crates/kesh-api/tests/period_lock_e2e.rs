//! Tests E2E — Story 24-4c (#380) : le **verrou de période**.
//!
//! ⛔ **Ce que cette story ferme n'est PAS ce que la planification annonçait.**
//! La note du 2026-08-28 décrivait une écriture « réécrivable en décembre » ; la
//! 24-4b a supprimé `journal_entries::update` et refuse le `DELETE`, donc plus
//! rien n'est réécrivable. Ce qui restait ouvert, c'est l'**ANTIDATAGE** —
//! créer aujourd'hui une écriture datée d'un trimestre déjà déclaré, ce qui
//! change ses totaux de TVA sans que rien ne le signale, le rapport TVA se
//! recalculant à la volée.
//!
//! Couvre : la garde au point de passage unique (`create_in_tx_inner`, que la
//! contre-passation TRAVERSE et ne contourne pas), le seuil **inclusif**, les
//! deux gardes de valeur de la pose, l'asymétrie des rôles, la précédence des
//! refus, et l'invariant qui compte — **le verrou n'enferme pas**.

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

fn auth(token: &str) -> String {
    format!("Bearer {token}")
}

/// Company + admin + exercice OUVERT couvrant **aujourd'hui**.
///
/// ⚠️ L'exercice couvre le jour courant et non une date figée : la
/// contre-passation porte la date du **jour** (D4), un exercice de 2026 en dur
/// ferait donc échouer la suite en 2027 — un test qui pourrit sans qu'on l'ait
/// touché.
async fn setup(pool: &MySqlPool) -> (TestApp, String, i64, i64) {
    let app = spawn_app(pool.clone()).await;
    create_test_company(pool).await;
    ensure_admin_user(pool, &test_config()).await.unwrap();
    let token = login(&app, "admin", TEST_ADMIN_PASSWORD).await;

    let company_id: i64 = sqlx::query_scalar("SELECT id FROM companies ORDER BY id LIMIT 1")
        .fetch_one(pool)
        .await
        .unwrap();

    let today = Utc::now().date_naive();
    let fy = fiscal_years::create(
        pool,
        1,
        NewFiscalYear {
            company_id,
            name: format!("Exercice {}", today.format("%Y")),
            start_date: NaiveDate::from_ymd_opt(
                today.format("%Y").to_string().parse().unwrap(),
                1,
                1,
            )
            .unwrap(),
            end_date: NaiveDate::from_ymd_opt(
                today.format("%Y").to_string().parse().unwrap(),
                12,
                31,
            )
            .unwrap(),
        },
    )
    .await
    .expect("exercice");

    (app, token, company_id, fy.id)
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

/// Écriture manuelle à deux lignes, 100.00 au débit de `d`, au crédit de `c`.
async fn make_entry(
    pool: &MySqlPool,
    company_id: i64,
    fy_id: i64,
    d: i64,
    c: i64,
    projects: (Option<i64>, Option<i64>),
) -> i64 {
    let mut tx = pool.begin().await.unwrap();
    let created = journal_entries::create_in_tx(
        &mut tx,
        fy_id,
        1,
        NewJournalEntry {
            company_id,
            entry_date: Utc::now().date_naive(),
            journal: Journal::OD,
            description: "Écriture à corriger".into(),
            project_id: None,
            lines: vec![
                NewJournalEntryLine {
                    account_id: d,
                    debit: dec!(100.00),
                    credit: dec!(0),
                    project_id: projects.0,
                },
                NewJournalEntryLine {
                    account_id: c,
                    debit: dec!(0),
                    credit: dec!(100.00),
                    project_id: projects.1,
                },
            ],
        },
        false,
    )
    .await
    .expect("écriture");
    tx.commit().await.unwrap();
    created.entry.id
}

/// Crée un utilisateur `Consultation` et rend son jeton (AC 12).
async fn consultation_token(app: &TestApp, pool: &MySqlPool) -> String {
    use kesh_db::entities::{NewUser, Role};

    let company_id: i64 = sqlx::query_scalar("SELECT id FROM companies ORDER BY id LIMIT 1")
        .fetch_one(pool)
        .await
        .unwrap();
    let password = "consultation-test-pw-12345";
    let hash = kesh_api::auth::password::hash_password(password).expect("hash");
    kesh_db::repositories::users::create(
        pool,
        NewUser {
            username: "consultation".into(),
            password_hash: hash,
            role: Role::Consultation,
            active: true,
            company_id,
            email: None,
        },
    )
    .await
    .expect("utilisateur Consultation");
    login(app, "consultation", password).await
}

// ---------------------------------------------------------------------------
// Helpers propres au verrou
// ---------------------------------------------------------------------------

fn hier() -> chrono::NaiveDate {
    Utc::now().date_naive() - chrono::Duration::days(1)
}

async fn poser_verrou(
    app: &TestApp,
    token: &str,
    through: chrono::NaiveDate,
) -> (reqwest::StatusCode, Value) {
    let resp = app
        .client
        .post(app.url("/api/v1/companies/current/books-lock"))
        .header("Authorization", auth(token))
        .json(&json!({ "through": through.to_string() }))
        .send()
        .await
        .unwrap();
    let status = resp.status();
    (status, resp.json().await.unwrap_or(Value::Null))
}

async fn lever_verrou(
    app: &TestApp,
    token: &str,
    through: Option<chrono::NaiveDate>,
    motif: &str,
) -> (reqwest::StatusCode, Value) {
    let resp = app
        .client
        .post(app.url("/api/v1/companies/current/books-lock/release"))
        .header("Authorization", auth(token))
        .json(&json!({ "through": through.map(|d| d.to_string()), "motif": motif }))
        .send()
        .await
        .unwrap();
    let status = resp.status();
    (status, resp.json().await.unwrap_or(Value::Null))
}

/// Crée une écriture manuelle **par la route**, à la date fournie.
async fn creer_ecriture(
    app: &TestApp,
    token: &str,
    d: i64,
    c: i64,
    date: chrono::NaiveDate,
) -> (reqwest::StatusCode, Value) {
    let resp = app
        .client
        .post(app.url("/api/v1/journal-entries"))
        .header("Authorization", auth(token))
        .json(&json!({
            "entryDate": date.to_string(),
            "journal": "OD",
            "description": "Écriture de test",
            "lines": [
                { "accountId": d, "debit": "100.00", "credit": "0" },
                { "accountId": c, "debit": "0", "credit": "100.00" },
            ]
        }))
        .send()
        .await
        .unwrap();
    let status = resp.status();
    (status, resp.json().await.unwrap_or(Value::Null))
}

// ---------------------------------------------------------------------------
// AC 2 — le seuil est INCLUSIF
// ---------------------------------------------------------------------------

/// ⛔ Une borne au jour J refuse le jour J, et laisse passer J+1. L'écart d'un
/// jour est exactement le genre de défaut qu'aucun test ne rattrape s'il n'est
/// pas exercé aux DEUX bornes.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn le_seuil_du_verrou_est_inclusif(pool: MySqlPool) {
    let (app, token, company_id, _fy) = setup(&pool).await;
    let d = make_account(&pool, company_id, "6000", AccountType::Expense).await;
    let c = make_account(&pool, company_id, "1020", AccountType::Asset).await;

    let borne = Utc::now().date_naive() - chrono::Duration::days(10);
    let (st, body) = poser_verrou(&app, &token, borne).await;
    assert_eq!(st, 200, "la pose doit passer : {body}");

    // La veille de la borne : refusée.
    let (st, body) = creer_ecriture(&app, &token, d, c, borne - chrono::Duration::days(1)).await;
    assert_eq!(st, 400);
    assert_eq!(body["error"]["code"].as_str(), Some("PERIOD_LOCKED"));

    // La borne ELLE-MÊME : refusée. C'est ce que « jusqu'au 31 mars inclus » veut dire.
    let (st, body) = creer_ecriture(&app, &token, d, c, borne).await;
    assert_eq!(st, 400, "la borne elle-même doit être refusée : {body}");
    assert_eq!(body["error"]["code"].as_str(), Some("PERIOD_LOCKED"));

    // ⛔ Le message NOMME LES DEUX DATES — un refus qui ne dit pas jusqu'où les
    // livres sont fermés envoie l'utilisateur deviner.
    let message = body["error"]["message"].as_str().unwrap_or_default();
    assert!(
        message.contains(&borne.to_string()),
        "le message doit nommer la borne, or il dit : {message}"
    );

    // Le lendemain de la borne : accepté.
    let (st, body) = creer_ecriture(&app, &token, d, c, borne + chrono::Duration::days(1)).await;
    assert_eq!(st, 201, "le lendemain de la borne doit passer : {body}");
}

// ---------------------------------------------------------------------------
// AC 3 — la borne est STRICTEMENT antérieure à aujourd'hui
// ---------------------------------------------------------------------------

/// ⛔ Le « ou celle du jour » n'est pas un excès de prudence. La
/// contre-passation est datée du **jour** et le seuil est **inclusif** : une
/// borne posée à la date du jour refuserait toute correction faite le même
/// jour, en violation de l'AC 5 et de l'invariant I2.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn une_borne_future_ou_du_jour_est_refusee(pool: MySqlPool) {
    let (app, token, _company_id, _fy) = setup(&pool).await;

    let (st, _) = poser_verrou(
        &app,
        &token,
        Utc::now().date_naive() + chrono::Duration::days(1),
    )
    .await;
    assert_eq!(st, 400, "une borne future doit être refusée");

    let (st, _) = poser_verrou(&app, &token, Utc::now().date_naive()).await;
    assert_eq!(
        st, 400,
        "une borne à la date du JOUR doit être refusée aussi"
    );

    // La veille — valeur maximale admise — passe.
    let (st, body) = poser_verrou(&app, &token, hier()).await;
    assert_eq!(st, 200, "la veille est la borne maximale admise : {body}");
}

// ---------------------------------------------------------------------------
// AC 5 · I2 — LE VERROU N'ENFERME PAS
// ---------------------------------------------------------------------------

/// ⛔ **L'invariant qui compte.** La contre-passation d'une écriture d'une
/// période verrouillée doit aboutir — sinon le verrou rend les livres
/// incorrigibles, exactement le mode d'échec que l'ordre 24-4a → 24-4b existait
/// pour éviter.
///
/// ⚠️ **La borne est posée À LA VEILLE**, valeur maximale admise. Écrit avec
/// une borne franchement passée, ce test ne verrait pas le défaut d'un jour.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn la_contre_passation_traverse_le_verrou(pool: MySqlPool) {
    let (app, token, company_id, fy_id) = setup(&pool).await;
    let d = make_account(&pool, company_id, "6000", AccountType::Expense).await;
    let c = make_account(&pool, company_id, "1020", AccountType::Asset).await;

    // Une écriture datée d'avant la future borne.
    let origine = make_entry(&pool, company_id, fy_id, d, c, (None, None)).await;
    sqlx::query("UPDATE journal_entries SET entry_date = ? WHERE id = ?")
        .bind(Utc::now().date_naive() - chrono::Duration::days(5))
        .bind(origine)
        .execute(&pool)
        .await
        .unwrap();

    let (st, _) = poser_verrou(&app, &token, hier()).await;
    assert_eq!(st, 200);

    // La contre-passation est datée du JOUR, donc après la borne : elle passe.
    let resp = app
        .client
        .post(app.url(&format!("/api/v1/journal-entries/{origine}/reverse")))
        .header("Authorization", auth(&token))
        .send()
        .await
        .unwrap();
    let st = resp.status();
    let body: Value = resp.json().await.unwrap_or(Value::Null);
    assert_eq!(
        st, 201,
        "geler sans laisser corriger enfermerait l'utilisateur : {body}"
    );
}

// ---------------------------------------------------------------------------
// AC 6 — la garde de VALEUR, sans laquelle la garde de RÔLE est contournable
// ---------------------------------------------------------------------------

/// ⛔ Rien, dans une séparation de rôles, n'empêche d'appeler l'endpoint
/// « avancer » avec une date antérieure. La borne reculerait sans motif, sans
/// rôle Admin, sous une entrée d'audit `books.locked` **mensongère** — un
/// retrait maquillé en pose.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn l_endpoint_d_avancement_ne_peut_pas_reculer_la_borne(pool: MySqlPool) {
    let (app, token, _company_id, _fy) = setup(&pool).await;

    let borne = Utc::now().date_naive() - chrono::Duration::days(10);
    let (st, _) = poser_verrou(&app, &token, borne).await;
    assert_eq!(st, 200);

    // Reculer par l'endpoint de POSE : refusé.
    let (st, body) = poser_verrou(&app, &token, borne - chrono::Duration::days(5)).await;
    assert_eq!(
        st, 409,
        "reculer par l'endpoint d'avancement doit être refusé : {body}"
    );

    // La même valeur : refusée aussi (avancer veut dire AVANCER).
    let (st, _) = poser_verrou(&app, &token, borne).await;
    assert_eq!(st, 409);

    // Avancer : passe.
    let (st, body) = poser_verrou(&app, &token, borne + chrono::Duration::days(1)).await;
    assert_eq!(st, 200, "avancer doit passer : {body}");
}

/// ⚠️ **La garde de valeur se TAIT quand la borne est `NULL`** — c'est le seul
/// cas où elle doit se taire, et c'est la première pose.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn la_garde_de_valeur_se_tait_a_la_premiere_pose(pool: MySqlPool) {
    let (app, token, _company_id, _fy) = setup(&pool).await;
    let (st, body) = poser_verrou(
        &app,
        &token,
        Utc::now().date_naive() - chrono::Duration::days(365),
    )
    .await;
    assert_eq!(
        st, 200,
        "aucune borne courante : la garde ne s'applique pas : {body}"
    );
}

/// Le motif est obligatoire au déverrouillage, et les blancs ne comptent pas.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn le_deverrouillage_exige_un_motif_non_blanc(pool: MySqlPool) {
    let (app, token, _company_id, _fy) = setup(&pool).await;
    let (st, _) = poser_verrou(&app, &token, hier()).await;
    assert_eq!(st, 200);

    for motif in ["", "   ", "\t"] {
        let (st, _) = lever_verrou(&app, &token, None, motif).await;
        assert_eq!(st, 400, "motif {motif:?} : doit être refusé");
    }

    // ⛔ Le refus du motif porte lui aussi un CODE, et il doit traverser le
    // dispatch. ⚠️ Le patch de la passe 2 en a ajouté DEUX à la liste blanche et
    // n'en testait qu'UN : le jumeau serait resté muet, arrivant à l'écran en
    // « Entrée invalide » sans que rien ne rougisse. *Trouvé en grepant chaque
    // correction dans les tests avant d'écrire le journal — le geste que la
    // passe 2 avait recommandé.*
    let (_st, body) = lever_verrou(&app, &token, None, "").await;
    let message = body["error"]["message"].as_str().unwrap_or_default();
    assert!(
        message.to_lowercase().contains("motif"),
        "le refus doit nommer le motif, pas dire « Entrée invalide » : {message}"
    );

    let (st, body) = lever_verrou(&app, &token, None, "erreur de saisie sur le T1").await;
    assert_eq!(st, 200, "avec motif, le déverrouillage passe : {body}");
    assert!(
        body["booksLockedThrough"].is_null(),
        "le verrou doit être retiré : {body}"
    );
}

// ---------------------------------------------------------------------------
// AC 7 — RBAC
// ---------------------------------------------------------------------------

#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn consultation_ne_pose_ni_ne_leve_le_verrou(pool: MySqlPool) {
    let (app, _token, _company_id, _fy) = setup(&pool).await;
    let lecteur = consultation_token(&app, &pool).await;

    let (st, _) = poser_verrou(&app, &lecteur, hier()).await;
    assert_eq!(st, 403);
    let (st, _) = lever_verrou(&app, &lecteur, None, "peu importe").await;
    assert_eq!(st, 403);
}

// ---------------------------------------------------------------------------
// AC 8 — l'audit, et son producteur unique
// ---------------------------------------------------------------------------

#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn poser_et_lever_ecrivent_deux_actions_distinctes(pool: MySqlPool) {
    let (app, token, _company_id, _fy) = setup(&pool).await;

    let (st, _) = poser_verrou(&app, &token, hier()).await;
    assert_eq!(st, 200);
    let (st, _) = lever_verrou(&app, &token, None, "clôture annulée par le fiduciaire").await;
    assert_eq!(st, 200);

    let actions: Vec<String> =
        sqlx::query_scalar("SELECT action FROM audit_log WHERE action LIKE 'books.%' ORDER BY id")
            .fetch_all(&pool)
            .await
            .unwrap();
    assert_eq!(
        actions,
        vec!["books.locked".to_string(), "books.unlocked".to_string()],
        "deux actions DISTINCTES : confondre les deux rendrait le filtre d'audit inutilisable"
    );

    // ⛔ Le motif est dans la trace du déverrouillage — c'est ce qui le rend
    // opposable.
    let details: String = sqlx::query_scalar(
        "SELECT CAST(details_json AS CHAR) FROM audit_log WHERE action = 'books.unlocked'",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert!(
        details.contains("fiduciaire"),
        "le motif doit être tracé : {details}"
    );
}

// ---------------------------------------------------------------------------
// AC 9 — la précédence, telle que la ROUTE la rend
// ---------------------------------------------------------------------------

/// ⛔ `DATE_OUTSIDE_FISCAL_YEAR` est **inatteignable par cette route** : le
/// handler résout d'abord l'exercice couvrant la date, et une date qui ne tombe
/// dans aucun exercice rend `NO_FISCAL_YEAR`. Un test qui attendrait l'autre
/// code serait « ajusté jusqu'à passer ».
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn le_verrou_parle_en_dernier(pool: MySqlPool) {
    let (app, token, company_id, fy_id) = setup(&pool).await;
    let d = make_account(&pool, company_id, "6000", AccountType::Expense).await;
    let c = make_account(&pool, company_id, "1020", AccountType::Asset).await;

    let (st, _) = poser_verrou(&app, &token, hier()).await;
    assert_eq!(st, 200);

    // Hors de tout exercice, ET sous la borne : c'est NO_FISCAL_YEAR qui répond.
    let (st, body) = creer_ecriture(
        &app,
        &token,
        d,
        c,
        NaiveDate::from_ymd_opt(1990, 6, 15).unwrap(),
    )
    .await;
    assert_eq!(st, 400);
    assert_eq!(
        body["error"]["code"].as_str(),
        Some("NO_FISCAL_YEAR"),
        "hors exercice prime sur le verrou : {body}"
    );

    // Exercice CLOS, et sous la borne : c'est FISCAL_YEAR_CLOSED qui répond.
    sqlx::query("UPDATE fiscal_years SET status = 'Closed' WHERE id = ?")
        .bind(fy_id)
        .execute(&pool)
        .await
        .unwrap();
    let (st, body) = creer_ecriture(
        &app,
        &token,
        d,
        c,
        Utc::now().date_naive() - chrono::Duration::days(5),
    )
    .await;
    assert_eq!(st, 400);
    assert_eq!(
        body["error"]["code"].as_str(),
        Some("FISCAL_YEAR_CLOSED"),
        "exercice clos prime sur le verrou : {body}"
    );
}

// ---------------------------------------------------------------------------
// AC 4 · I1 — la garde tient sur les flux AUTOMATIQUES aussi
// ---------------------------------------------------------------------------

/// ⛔ La garde vit dans `create_in_tx_inner`, point de passage de **tous** les
/// chemins de création. Une garde posée aux routes en laisserait onze ouverts —
/// et valider une facture antidatée produirait exactement l'écriture que la
/// story interdit.
///
/// ⚠️ Ce test l'exerce par le chemin le plus court qui traverse `create_in_tx` :
/// l'écriture d'**ouverture** d'exercice, qui n'est pas une écriture manuelle.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn la_garde_tient_sous_les_flux_automatiques(pool: MySqlPool) {
    let (app, token, company_id, fy_id) = setup(&pool).await;
    let d = make_account(&pool, company_id, "1020", AccountType::Asset).await;
    let c = make_account(&pool, company_id, "2800", AccountType::Liability).await;

    let (st, _) = poser_verrou(&app, &token, hier()).await;
    assert_eq!(st, 200);

    let err = journal_entries::create_opening_entry(
        &pool,
        company_id,
        fy_id,
        1,
        NewJournalEntry {
            company_id,
            entry_date: Utc::now().date_naive() - chrono::Duration::days(3),
            journal: Journal::OD,
            description: "Soldes de départ antidatés".into(),
            project_id: None,
            lines: vec![
                NewJournalEntryLine {
                    account_id: d,
                    debit: dec!(5000.00),
                    credit: dec!(0),
                    project_id: None,
                },
                NewJournalEntryLine {
                    account_id: c,
                    debit: dec!(0),
                    credit: dec!(5000.00),
                    project_id: None,
                },
            ],
        },
    )
    .await
    .expect_err("un flux automatique ne contourne pas le verrou");

    assert!(
        matches!(err, kesh_db::errors::DbError::PeriodLocked { .. }),
        "le refus doit être PeriodLocked, or : {err:?}"
    );

    // I1 — rien n'est entré sous la borne.
    let sous_la_borne: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM journal_entries je JOIN companies co ON co.id = je.company_id \
         WHERE co.books_locked_through IS NOT NULL AND je.entry_date <= co.books_locked_through",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(
        sous_la_borne, 0,
        "aucune écriture ne doit exister sous la borne"
    );
}

// ---------------------------------------------------------------------------
// AC 10 — verrou de période et clôture annuelle sont INDÉPENDANTS
// ---------------------------------------------------------------------------

#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn la_cloture_ne_touche_pas_la_borne(pool: MySqlPool) {
    let (app, token, _company_id, fy_id) = setup(&pool).await;
    let borne = hier();
    let (st, _) = poser_verrou(&app, &token, borne).await;
    assert_eq!(st, 200);

    sqlx::query("UPDATE fiscal_years SET status = 'Closed' WHERE id = ?")
        .bind(fy_id)
        .execute(&pool)
        .await
        .unwrap();

    let apres: Option<chrono::NaiveDate> =
        sqlx::query_scalar("SELECT books_locked_through FROM companies ORDER BY id LIMIT 1")
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(apres, Some(borne), "clôturer ne doit pas toucher la borne");
}

// ---------------------------------------------------------------------------
// AC 3 · AC 6 — l'endpoint de LEVÉE porte les mêmes gardes de valeur
// ---------------------------------------------------------------------------

/// ⛔ **La garde de date manquait sur `unlock_books`, et rien ne l'exerçait.**
/// Un Admin pouvait y poser une borne **future** — d'un clic dans le formulaire
/// de déverrouillage, qui n'a aucun garde-fou visuel — et refuser du même coup
/// toute création d'écriture datée d'aujourd'hui, **contre-passation comprise**.
/// C'est-à-dire casser l'invariant I2, « le verrou n'enferme pas », que toute la
/// vague 24-4a → 24-4c existe pour tenir.
///
/// ⚠️ Ce test existe parce que la garde a été **livrée sans lui** en passe 1 de
/// revue, et que le journal de cette passe déclarait deux réserves qui portaient
/// sur autre chose. *Une garde qu'aucun test n'exerce se supprime au prochain
/// refactor sans que rien ne rougisse.*
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn le_deverrouillage_refuse_une_borne_future(pool: MySqlPool) {
    let (app, token, _company_id, _fy) = setup(&pool).await;
    let (st, _) = poser_verrou(&app, &token, hier()).await;
    assert_eq!(st, 200);

    for cible in [
        Utc::now().date_naive() + chrono::Duration::days(1),
        Utc::now().date_naive(),
    ] {
        let (st, _) = lever_verrou(&app, &token, Some(cible), "motif valable").await;
        assert_eq!(
            st, 400,
            "une borne {cible} posée par la LEVÉE doit être refusée comme par la pose"
        );
    }
}

/// ⛔ **Reculer la borne d'un cran** — la moitié de `unlock_books` que rien
/// n'exerçait : les quatre appels du fichier passaient tous `None`, donc seule
/// la suppression totale était testée, jamais le recul.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn le_deverrouillage_recule_effectivement_la_borne(pool: MySqlPool) {
    let (app, token, _company_id, _fy) = setup(&pool).await;
    let haute = Utc::now().date_naive() - chrono::Duration::days(10);
    let basse = Utc::now().date_naive() - chrono::Duration::days(40);

    let (st, _) = poser_verrou(&app, &token, haute).await;
    assert_eq!(st, 200);

    let (st, body) = lever_verrou(&app, &token, Some(basse), "le T2 doit être rouvert").await;
    assert_eq!(st, 200, "reculer d'un cran doit passer : {body}");
    assert_eq!(
        body["booksLockedThrough"].as_str(),
        Some(basse.to_string().as_str()),
        "la borne doit avoir effectivement reculé : {body}"
    );
}

/// ⛔ **L'endpoint de LEVÉE ne peut pas AVANCER la borne** — sinon le verrou
/// avancerait sous une entrée d'audit `books.unlocked`, et le doc-comment de la
/// fonction affirmerait faux : ce verbe a **un seul producteur**, le
/// déverrouillage délibéré.
///
/// ⚠️ Pas de faille de droits ici — la route est Admin seule. Ce qui se
/// corrompt, c'est la **trace** : le réviseur qui filtre « qui a déverrouillé »
/// lirait une pose.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn le_deverrouillage_ne_peut_pas_avancer_la_borne(pool: MySqlPool) {
    let (app, token, _company_id, _fy) = setup(&pool).await;
    let basse = Utc::now().date_naive() - chrono::Duration::days(40);
    let (st, _) = poser_verrou(&app, &token, basse).await;
    assert_eq!(st, 200);

    let (st, body) = lever_verrou(
        &app,
        &token,
        Some(Utc::now().date_naive() - chrono::Duration::days(10)),
        "tentative d'avancée par le mauvais verbe",
    )
    .await;
    assert_eq!(st, 409, "avancer par la levée doit être refusé : {body}");

    // La borne n'a pas bougé, et aucune entrée `books.unlocked` mensongère.
    let apres: Option<chrono::NaiveDate> =
        sqlx::query_scalar("SELECT books_locked_through FROM companies ORDER BY id LIMIT 1")
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(apres, Some(basse));
    let mensonges: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM audit_log WHERE action = 'books.unlocked'")
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(mensonges, 0, "un refus n'écrit aucune trace");
}

/// AC 3 — ⚠️ **le refus porte un CODE, pas une phrase.** `DbError::InvalidInput`
/// est confronté à une liste blanche stricte côté API ; un code absent du
/// dispatch retombe sur « Entrée invalide », sans date ni raison — sur le geste
/// même que la garde existe pour rattraper.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn le_refus_de_borne_dit_pourquoi(pool: MySqlPool) {
    let (app, token, _company_id, _fy) = setup(&pool).await;
    let (_st, body) = poser_verrou(&app, &token, Utc::now().date_naive()).await;
    let message = body["error"]["message"].as_str().unwrap_or_default();
    assert!(
        message.to_lowercase().contains("antérieure")
            || message.to_lowercase().contains("aujourd'hui"),
        "le refus doit expliquer, pas dire « Entrée invalide » : {message}"
    );
}
