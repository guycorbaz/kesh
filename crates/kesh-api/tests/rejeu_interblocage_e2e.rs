//! Rejeu sur interblocage — Story 15-5e1 (AC4, tests 1 à 5 et 7 ; #463, #491,
//! #536 pour la validation) ; Story 15-12a (tests 9 et 10 : clôture et création
//! d'un exercice ; #543).
//!
//! Chaque test « route victime » monte un interblocage **déterministe** sur le
//! patron de `accept_replays_the_batch_when_it_is_the_deadlock_victim`
//! (`reconciliation_e2e.rs`) :
//!
//! 1. une transaction de test s'**alourdit** (500 lignes insérées dans une
//!    table de lest : InnoDB choisit pour victime la transaction la plus
//!    légère), puis tient la ressource que la route demandera **en second** ;
//! 2. la route, lancée en tâche, est vue en attente sur cette ressource
//!    (`kesh_db::test_fixtures::attendre_une_requete_en_cours`) ;
//! 3. la transaction de test demande la ressource que la route tient **déjà** :
//!    cycle, InnoDB annule la route — et la transaction de test **doit** obtenir
//!    son verrou (`.expect`), sinon c'est elle la victime et le test ne prouve
//!    rien ;
//! 4. la transaction de test annule ; la route rejoue et réussit.
//!
//! **Le témoin du rejeu** (choix C74) : le code de retour et les comptes ne
//! départagent pas « annulée puis rejouée » de « réussie du premier coup ».
//! Chaque test exige donc l'événement `warn!` de `kesh_db::retry` qui porte le
//! nom de l'opération (`common::capture_rejeu`).
//!
//! ⚠️ **Harnais recopié** (`test_config`, `spawn_app`, `forge_jwt`), comme
//! dans chacun des fichiers E2E de ce répertoire : copie assumée (choix C70) —
//! factoriser le harnais de tous les fichiers est une dette antérieure.

mod common;

use std::net::SocketAddr;
use std::path::Path;
use std::sync::Arc;
use std::sync::atomic::{AtomicU32, Ordering};

use chrono::{NaiveDate, TimeDelta, Utc};
use common::capture_rejeu::CaptureRejeu;
use jsonwebtoken::{Algorithm, EncodingKey, Header};
use kesh_api::auth::jwt::Claims;
use kesh_api::config::Config;
use kesh_api::errors::AppError;
use kesh_api::{AppState, build_router};
use kesh_db::entities::{ContactType, NewContact};
use kesh_db::errors::{DbError, map_db_error};
use kesh_db::repositories::contacts;
use kesh_db::test_fixtures::{
    SeededCompany, attendre_une_requete_en_cours, designate_rounding_account,
    disable_rounding_to_5_centimes, seed_accounting_company,
};
use serde_json::{Value, json};
use sqlx::{Connection, MySql, MySqlPool, Transaction};

const TEST_JWT_SECRET: &[u8] = b"test-secret-32-bytes-minimum-test-secret-padding";
const TEST_ADMIN_PASSWORD: &str = "e2e-test-admin-password";

// ============================================================
// Harnais (recopié — patron reconciliation_e2e.rs)
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
            Err(e) => panic!("test server did not become ready within 2s: {e}"),
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

// ============================================================
// Montage commun
// ============================================================

/// Une société de `seed_accounting_company` (réglages de facturation, quatre
/// taux de TVA, cinq comptes, exercice 2020-2030, deux `Admin`), le serveur et
/// un JWT `Admin`.
struct Contexte {
    app: Arc<TestApp>,
    jwt: String,
    societe: SeededCompany,
}

impl Contexte {
    fn company_id(&self) -> i64 {
        self.societe.company_id
    }

    fn compte(&self, numero: &str) -> i64 {
        self.societe.accounts[numero]
    }
}

async fn monter(pool: &MySqlPool) -> Contexte {
    let societe = seed_accounting_company(pool)
        .await
        .expect("seed_accounting_company");
    let app = Arc::new(spawn_app(pool.clone()).await);
    let jwt = forge_jwt(societe.admin_user_id, "Admin", societe.company_id);
    Contexte { app, jwt, societe }
}

fn aujourd_hui() -> NaiveDate {
    Utc::now().date_naive()
}

/// L'exercice **ouvert qui couvre aujourd'hui**, créé s'il manque — sans date
/// codée en dur (patron `ensure_fiscal_year_today`) : l'exercice 2020-2030 de
/// `seed_accounting_company` cessera de couvrir aujourd'hui en 2031.
async fn exercice_du_jour(pool: &MySqlPool, company_id: i64) -> i64 {
    let today = aujourd_hui();
    let couvrant: Option<i64> = sqlx::query_scalar(
        "SELECT id FROM fiscal_years WHERE company_id = ? AND start_date <= ? \
         AND end_date >= ? AND status = 'Open' LIMIT 1",
    )
    .bind(company_id)
    .bind(today)
    .bind(today)
    .fetch_optional(pool)
    .await
    .unwrap();
    if let Some(id) = couvrant {
        return id;
    }
    let year = chrono::Datelike::year(&today);
    sqlx::query(
        "INSERT INTO fiscal_years (company_id, name, start_date, end_date, status) \
         VALUES (?, ?, ?, ?, 'Open')",
    )
    .bind(company_id)
    .bind(format!("Exercice {year} (rejeu)"))
    .bind(NaiveDate::from_ymd_opt(year, 1, 1).unwrap())
    .bind(NaiveDate::from_ymd_opt(year, 12, 31).unwrap())
    .execute(pool)
    .await
    .unwrap()
    .last_insert_id() as i64
}

/// Une transaction de test **alourdie** : 500 lignes insérées dans une table de
/// lest, hors de tout flux métier (le DDL se fait hors transaction).
async fn transaction_lourde(pool: &MySqlPool) -> Transaction<'static, MySql> {
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS lest_rejeu (id INT AUTO_INCREMENT PRIMARY KEY, x INT) \
         ENGINE=InnoDB",
    )
    .execute(pool)
    .await
    .unwrap();
    let mut tx = pool.begin().await.unwrap();
    for _ in 0..10 {
        sqlx::query("INSERT INTO lest_rejeu (x) SELECT seq FROM seq_1_to_50")
            .execute(&mut *tx)
            .await
            .unwrap();
    }
    tx
}

/// Lance une requête en tâche ; rend `(statut, corps)`.
fn requete_en_tache(
    ctx: &Contexte,
    methode: reqwest::Method,
    chemin: &str,
    corps: Option<Value>,
) -> tokio::task::JoinHandle<(u16, Value)> {
    let app = ctx.app.clone();
    let jwt = ctx.jwt.clone();
    let url = app.url(chemin);
    tokio::spawn(async move {
        let mut req = app.client.request(methode, url).bearer_auth(jwt);
        if let Some(c) = corps {
            req = req.json(&c);
        }
        let resp = req.send().await.unwrap();
        let status = resp.status().as_u16();
        (status, resp.json().await.unwrap_or(Value::Null))
    })
}

async fn requete(
    ctx: &Contexte,
    methode: reqwest::Method,
    chemin: &str,
    corps: Option<Value>,
) -> (u16, Value) {
    requete_en_tache(ctx, methode, chemin, corps).await.unwrap()
}

/// Un contact client (semé par le dépôt).
async fn contact(pool: &MySqlPool, ctx: &Contexte, nom: &str, fournisseur: bool) -> i64 {
    contacts::create(
        pool,
        ctx.societe.admin_user_id,
        NewContact {
            company_id: ctx.company_id(),
            contact_type: ContactType::Entreprise,
            name: nom.into(),
            first_name: None,
            last_name: None,
            is_client: !fournisseur,
            is_supplier: fournisseur,
            address: None,
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
    .expect("contact")
    .id
}

/// Une facture client brouillon, **par la route** `POST /api/v1/invoices`,
/// datée du jour, une ligne à 8,1 %. Aide locale passant par les routes — ce
/// n'est pas une copie de `seed_validated_invoice`.
async fn facture_brouillon(pool: &MySqlPool, ctx: &Contexte, prix_ht: &str) -> i64 {
    let client = contact(pool, ctx, "Client du rejeu", false).await;
    let (status, corps) = requete(
        ctx,
        reqwest::Method::POST,
        "/api/v1/invoices",
        Some(json!({
            "contactId": client,
            "date": aujourd_hui().to_string(),
            "lines": [
                {"description": "Conseil", "quantity": "1", "unitPrice": prix_ht, "vatRate": "8.10"}
            ]
        })),
    )
    .await;
    assert_eq!(status, 201, "création de la facture : {corps}");
    corps["id"].as_i64().unwrap()
}

/// Une facture client validée par les routes (création puis validation).
async fn facture_validee(pool: &MySqlPool, ctx: &Contexte, prix_ht: &str) -> i64 {
    let id = facture_brouillon(pool, ctx, prix_ht).await;
    let (status, corps) = requete(
        ctx,
        reqwest::Method::POST,
        &format!("/api/v1/invoices/{id}/validate"),
        None,
    )
    .await;
    assert_eq!(status, 200, "validation : {corps}");
    id
}

async fn compter(pool: &MySqlPool, sql: &str, id: i64) -> i64 {
    sqlx::query_scalar(sql)
        .bind(id)
        .fetch_one(pool)
        .await
        .unwrap()
}

/// Les étapes 2 à 4 du patron : attendre la route sur `motifs`, fermer le cycle
/// par `fermeture` (la ressource que la route tient déjà), annuler la
/// transaction de test, rendre la réponse de la route.
async fn victime(
    pool: &MySqlPool,
    mut lourde: Transaction<'static, MySql>,
    route: tokio::task::JoinHandle<(u16, Value)>,
    motifs: &[&str],
    fermeture: &str,
    id_ferme: i64,
) -> (u16, Value) {
    let bloquee = attendre_une_requete_en_cours(pool, motifs, || route.is_finished()).await;
    assert!(
        bloquee,
        "la route devait attendre sur {motifs:?} — le montage ne prouve rien sinon"
    );
    sqlx::query(fermeture)
        .bind(id_ferme)
        .fetch_all(&mut *lourde)
        .await
        .expect("la transaction de test doit survivre à l'interblocage");
    lourde.rollback().await.unwrap();
    route.await.unwrap()
}

// ============================================================
// Test 1 — le prédicat et l'enveloppe AppError, sur une vraie 1213
// ============================================================

/// Provoque une **vraie** 1213 sur une table sentinelle, par deux connexions
/// dédiées, et rend l'erreur de la victime passée par `map_db_error`.
async fn provoquer_un_interblocage(pool: &MySqlPool) -> DbError {
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS sentinelle_rejeu (id BIGINT PRIMARY KEY, v INT) ENGINE=InnoDB",
    )
    .execute(pool)
    .await
    .unwrap();
    sqlx::query("INSERT IGNORE INTO sentinelle_rejeu (id, v) VALUES (1, 0), (2, 0)")
        .execute(pool)
        .await
        .unwrap();
    let mut lourde = transaction_lourde(pool).await;
    sqlx::query("SELECT id FROM sentinelle_rejeu WHERE id = 1 FOR UPDATE")
        .fetch_one(&mut *lourde)
        .await
        .unwrap();
    let mut legere = pool.begin().await.unwrap();
    sqlx::query("SELECT id FROM sentinelle_rejeu WHERE id = 2 FOR UPDATE")
        .fetch_one(&mut *legere)
        .await
        .unwrap();
    let attente = tokio::spawn(async move {
        let r = sqlx::query("SELECT id FROM sentinelle_rejeu WHERE id = 1 FOR UPDATE /* legere */")
            .fetch_one(&mut *legere)
            .await;
        let _ = legere.rollback().await;
        r.map(|_| ())
    });
    let vue = attendre_une_requete_en_cours(pool, &["sentinelle_rejeu", "legere"], || {
        attente.is_finished()
    })
    .await;
    assert!(vue, "la transaction légère devait attendre la ligne 1");
    sqlx::query("SELECT id FROM sentinelle_rejeu WHERE id = 2 FOR UPDATE")
        .fetch_one(&mut *lourde)
        .await
        .expect("la transaction lourde doit survivre à l'interblocage");
    lourde.rollback().await.unwrap();
    let erreur = attente
        .await
        .unwrap()
        .expect_err("la transaction légère devait être la victime");
    map_db_error(erreur)
}

#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn the_predicates_and_the_app_envelope_recognise_a_real_deadlock(pool: MySqlPool) {
    // (a) Le prédicat, sur une vraie 1213 passée par `map_db_error`.
    let erreur = provoquer_un_interblocage(&pool).await;
    assert!(
        kesh_db::retry::is_deadlock_error(&erreur),
        "une 1213 doit rester `DbError::Sqlx` et être reconnue : {erreur:?}"
    );
    let app_err = AppError::Database(erreur);
    assert!(kesh_api::retry::is_app_deadlock(&app_err));

    // (b) Négatif : un dépassement d'attente (1205).
    let mut tient = pool.begin().await.unwrap();
    sqlx::query("SELECT id FROM sentinelle_rejeu WHERE id = 1 FOR UPDATE")
        .fetch_one(&mut *tient)
        .await
        .unwrap();
    // ⚠️ Le délai d'une seconde est une variable de SESSION : un `ROLLBACK` ne
    // la remet pas. Rendue au pool, cette connexion pourrait servir de
    // transaction légère à (d), où une attente de plus d'une seconde finirait
    // en 1205 au lieu de la 1213 attendue (revue P1, B-1). La connexion est
    // donc DÉTACHÉE du pool et fermée après usage.
    let mut attend = pool.acquire().await.unwrap();
    sqlx::query("SET SESSION innodb_lock_wait_timeout = 1")
        .execute(&mut *attend)
        .await
        .unwrap();
    sqlx::query("START TRANSACTION")
        .execute(&mut *attend)
        .await
        .unwrap();
    let delai = sqlx::query("SELECT id FROM sentinelle_rejeu WHERE id = 1 FOR UPDATE")
        .fetch_one(&mut *attend)
        .await
        .expect_err("l'attente devait dépasser une seconde");
    sqlx::query("ROLLBACK").execute(&mut *attend).await.unwrap();
    attend.detach().close().await.unwrap();
    tient.rollback().await.unwrap();
    let delai = map_db_error(delai);
    assert!(
        !kesh_db::retry::is_deadlock_error(&delai),
        "un 1205 n'est pas un interblocage : {delai:?}"
    );
    assert!(!kesh_api::retry::is_app_deadlock(&AppError::Database(
        delai
    )));

    // (c) Négatif : une erreur métier.
    assert!(!kesh_db::retry::is_deadlock_error(
        &DbError::OptimisticLockConflict
    ));
    assert!(!kesh_api::retry::is_app_deadlock(&AppError::Database(
        DbError::OptimisticLockConflict
    )));

    // (d) L'enveloppe : une vraie 1213 au premier appel, `Ok` au second.
    let appels = Arc::new(AtomicU32::new(0));
    let resultat = kesh_api::retry::retry_app_on_deadlock("test::rejeu", || {
        let appels = appels.clone();
        let pool = pool.clone();
        async move {
            if appels.fetch_add(1, Ordering::SeqCst) == 0 {
                Err(AppError::Database(provoquer_un_interblocage(&pool).await))
            } else {
                Ok(42)
            }
        }
    })
    .await;
    assert_eq!(resultat.ok(), Some(42), "la 1213 doit être rejouée");
    assert_eq!(
        appels.load(Ordering::SeqCst),
        2,
        "deux appels : la victime, le rejeu"
    );

    // (e) L'enveloppe : une erreur métier n'est pas rejouée — ni rejeu ni
    // épuisement journalisés.
    let capture = CaptureRejeu::installer();
    let appels = Arc::new(AtomicU32::new(0));
    let resultat: Result<(), AppError> =
        kesh_api::retry::retry_app_on_deadlock("test::rejeu", || {
            let appels = appels.clone();
            async move {
                appels.fetch_add(1, Ordering::SeqCst);
                Err(AppError::Database(DbError::OptimisticLockConflict))
            }
        })
        .await;
    assert!(matches!(
        resultat,
        Err(AppError::Database(DbError::OptimisticLockConflict))
    ));
    assert_eq!(
        appels.load(Ordering::SeqCst),
        1,
        "une erreur métier : un seul appel"
    );
    assert!(capture.operations().is_empty() && capture.epuisements().is_empty());

    // (f) L'enveloppe épuisée (revue P1, E-3) : une vraie 1213 à chacune des
    // trois tentatives → deux `warn!` de rejeu, puis un `error!` qui nomme
    // l'opération, et l'erreur remonte telle quelle.
    let appels = Arc::new(AtomicU32::new(0));
    let resultat: Result<(), AppError> =
        kesh_api::retry::retry_app_on_deadlock("test::epuise", || {
            let appels = appels.clone();
            let pool = pool.clone();
            async move {
                appels.fetch_add(1, Ordering::SeqCst);
                Err(AppError::Database(provoquer_un_interblocage(&pool).await))
            }
        })
        .await;
    assert!(
        matches!(&resultat, Err(e) if kesh_api::retry::is_app_deadlock(e)),
        "la dernière 1213 remonte : {resultat:?}"
    );
    assert_eq!(appels.load(Ordering::SeqCst), 3, "trois tentatives");
    assert_eq!(capture.operations(), vec!["test::epuise"; 2]);
    assert_eq!(
        capture.epuisements(),
        vec!["test::epuise"],
        "l'épuisement du rejeu doit être journalisé avec le nom de l'opération"
    );
}

// ============================================================
// Test 2 — règlement client victime (#491)
// ============================================================

#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn settlement_is_replayed_when_it_is_the_deadlock_victim(pool: MySqlPool) {
    let ctx = monter(&pool).await;
    disable_rounding_to_5_centimes(&pool, ctx.company_id())
        .await
        .unwrap();
    let capture = CaptureRejeu::installer();
    let facture = facture_validee(&pool, &ctx, "100.00").await;
    let exercice = exercice_du_jour(&pool, ctx.company_id()).await;
    let ecritures_avant = compter(
        &pool,
        "SELECT COUNT(*) FROM journal_entries WHERE company_id = ?",
        ctx.company_id(),
    )
    .await;

    // (1) La transaction de test : lourde, puis l'exercice.
    let mut lourde = transaction_lourde(&pool).await;
    sqlx::query("SELECT id FROM fiscal_years WHERE id = ? FOR UPDATE")
        .bind(exercice)
        .fetch_one(&mut *lourde)
        .await
        .unwrap();

    // (2) Le règlement par compte interne : facture, compte, puis l'exercice.
    let route = requete_en_tache(
        &ctx,
        reqwest::Method::POST,
        &format!("/api/v1/invoices/{facture}/settlements"),
        Some(json!({
            "settlementType": "internal_account",
            "accountId": ctx.compte("1000"),
            "amount": "108.10",
            "settledOn": aujourd_hui().to_string(),
        })),
    );
    // (3) La facture, que tient la route.
    let (status, corps) = victime(
        &pool,
        lourde,
        route,
        &["fiscal_years", "FOR UPDATE"],
        "SELECT id FROM invoices WHERE id = ? FOR UPDATE",
        facture,
    )
    .await;

    assert_eq!(status, 200, "⛔ l'interblocage doit être rejoué : {corps}");
    assert_eq!(
        compter(
            &pool,
            "SELECT COUNT(*) FROM invoice_settlements WHERE invoice_id = ?",
            facture
        )
        .await,
        1,
        "un seul règlement"
    );
    assert_eq!(
        compter(
            &pool,
            "SELECT COUNT(*) FROM journal_entries WHERE company_id = ?",
            ctx.company_id()
        )
        .await,
        ecritures_avant + 1,
        "une seule écriture de règlement"
    );
    capture.exiger_un_rejeu("invoices::settle");
}

// ============================================================
// Test 3 — annulation d'un règlement victime (#463)
// ============================================================

#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn settlement_cancellation_is_replayed_when_it_is_the_deadlock_victim(pool: MySqlPool) {
    let ctx = monter(&pool).await;
    disable_rounding_to_5_centimes(&pool, ctx.company_id())
        .await
        .unwrap();
    let capture = CaptureRejeu::installer();
    let facture = facture_validee(&pool, &ctx, "100.00").await;
    let (status, corps) = requete(
        &ctx,
        reqwest::Method::POST,
        &format!("/api/v1/invoices/{facture}/settlements"),
        Some(json!({
            "settlementType": "internal_account",
            "accountId": ctx.compte("1000"),
            "amount": "108.10",
            "settledOn": aujourd_hui().to_string(),
        })),
    )
    .await;
    assert_eq!(status, 200, "règlement préalable : {corps}");
    let reglement: i64 =
        sqlx::query_scalar("SELECT id FROM invoice_settlements WHERE invoice_id = ?")
            .bind(facture)
            .fetch_one(&pool)
            .await
            .unwrap();
    let exercice = exercice_du_jour(&pool, ctx.company_id()).await;
    let ecritures_avant = compter(
        &pool,
        "SELECT COUNT(*) FROM journal_entries WHERE company_id = ?",
        ctx.company_id(),
    )
    .await;

    let mut lourde = transaction_lourde(&pool).await;
    sqlx::query("SELECT id FROM fiscal_years WHERE id = ? FOR UPDATE")
        .bind(exercice)
        .fetch_one(&mut *lourde)
        .await
        .unwrap();

    // La route tient la facture et le règlement, puis attend l'exercice de
    // l'écriture de règlement (étape (2-bis) de `cancel_settlement_in_tx`).
    let route = requete_en_tache(
        &ctx,
        reqwest::Method::POST,
        &format!("/api/v1/invoices/{facture}/settlements/{reglement}/cancel"),
        None,
    );
    let (status, corps) = victime(
        &pool,
        lourde,
        route,
        &["JOIN fiscal_years fy", "FOR UPDATE"],
        "SELECT id FROM invoices WHERE id = ? FOR UPDATE",
        facture,
    )
    .await;

    assert_eq!(status, 200, "⛔ l'interblocage doit être rejoué : {corps}");
    assert_eq!(
        compter(
            &pool,
            "SELECT COUNT(*) FROM invoice_settlements WHERE invoice_id = ?",
            facture
        )
        .await,
        0,
        "le règlement est retiré, une fois"
    );
    assert_eq!(
        compter(
            &pool,
            "SELECT COUNT(*) FROM journal_entries WHERE company_id = ?",
            ctx.company_id()
        )
        .await,
        ecritures_avant + 1,
        "une seule contre-passation"
    );
    capture.exiger_un_rejeu("invoices::cancel_settlement");
}

// ============================================================
// Test 4 — validation victime : le cycle de #536
// ============================================================

#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn validation_is_replayed_when_it_is_the_deadlock_victim(pool: MySqlPool) {
    let ctx = monter(&pool).await;
    // L'arrondi reste ACTIF, exprès : HT 100.10 à 8,1 % → TTC 108.21, arrondi à
    // 108.20 — la validation prend le compte d'arrondi (étape (2 bis')).
    let arrondi = designate_rounding_account(&pool, ctx.company_id())
        .await
        .unwrap();
    let designe: Option<i64> = sqlx::query_scalar(
        "SELECT default_rounding_account_id FROM company_invoice_settings WHERE company_id = ?",
    )
    .bind(ctx.company_id())
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(
        designe,
        Some(arrondi),
        "le compte d'arrondi doit être désigné (la ligne des réglages existe)"
    );
    let capture = CaptureRejeu::installer();
    let facture = facture_brouillon(&pool, &ctx, "100.10").await;
    let exercice = exercice_du_jour(&pool, ctx.company_id()).await;

    let mut lourde = transaction_lourde(&pool).await;
    sqlx::query("SELECT id FROM fiscal_years WHERE id = ? FOR UPDATE")
        .bind(exercice)
        .fetch_one(&mut *lourde)
        .await
        .unwrap();

    let route = requete_en_tache(
        &ctx,
        reqwest::Method::POST,
        &format!("/api/v1/invoices/{facture}/validate"),
        None,
    );
    // Le compte d'arrondi, que tient la route — l'ordre exercice → arrondi du
    // lot de rapprochement.
    let (status, corps) = victime(
        &pool,
        lourde,
        route,
        &["fiscal_years", "FOR UPDATE"],
        "SELECT id FROM accounts WHERE id = ? FOR UPDATE",
        arrondi,
    )
    .await;

    assert_eq!(status, 200, "⛔ l'interblocage doit être rejoué : {corps}");
    assert_eq!(corps["status"], "validated", "{corps}");
    assert_eq!(
        compter(
            &pool,
            "SELECT COUNT(*) FROM journal_entries WHERE company_id = ?",
            ctx.company_id()
        )
        .await,
        1,
        "une seule écriture de vente"
    );
    assert_eq!(
        compter(
            &pool,
            "SELECT COUNT(*) FROM journal_entry_lines WHERE account_id = ?",
            arrondi
        )
        .await,
        1,
        "l'écart d'arrondi est passé"
    );
    capture.exiger_un_rejeu("invoices::validate");
}

// ============================================================
// Test 5 — saisie fournisseur victime (choix C66)
// ============================================================

#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn supplier_invoice_is_replayed_when_it_is_the_deadlock_victim(pool: MySqlPool) {
    let ctx = monter(&pool).await;
    let fournisseur = contact(&pool, &ctx, "Fournisseur du rejeu", true).await;
    // Le compte créanciers, exigé après l'exercice.
    sqlx::query(
        "UPDATE company_invoice_settings SET default_payable_account_id = ? WHERE company_id = ?",
    )
    .bind(ctx.compte("2000"))
    .bind(ctx.company_id())
    .execute(&pool)
    .await
    .unwrap();
    let charge = ctx.compte("4000");
    let exercice = exercice_du_jour(&pool, ctx.company_id()).await;
    let capture = CaptureRejeu::installer();

    let mut lourde = transaction_lourde(&pool).await;
    sqlx::query("SELECT id FROM fiscal_years WHERE id = ? FOR UPDATE")
        .bind(exercice)
        .fetch_one(&mut *lourde)
        .await
        .unwrap();

    // La route tient la ligne des réglages et le compte de charge (ordre de
    // l'AC5), puis attend l'exercice.
    let route = requete_en_tache(
        &ctx,
        reqwest::Method::POST,
        "/api/v1/supplier-invoices",
        Some(json!({
            "contactId": fournisseur,
            "invoiceDate": aujourd_hui().to_string(),
            "lines": [{
                "description": "Fournitures",
                "quantity": "1",
                "unitPrice": "50.00",
                "vatRate": "0",
                "expenseAccountId": charge,
            }],
        })),
    );
    let (status, corps) = victime(
        &pool,
        lourde,
        route,
        &["fiscal_years", "FOR UPDATE"],
        "SELECT id FROM accounts WHERE id = ? FOR UPDATE",
        charge,
    )
    .await;

    assert_eq!(status, 201, "⛔ l'interblocage doit être rejoué : {corps}");
    assert_eq!(
        compter(
            &pool,
            "SELECT COUNT(*) FROM supplier_invoices WHERE company_id = ?",
            ctx.company_id()
        )
        .await,
        1,
        "une seule facture fournisseur"
    );
    assert_eq!(
        compter(
            &pool,
            "SELECT COUNT(*) FROM journal_entries WHERE company_id = ?",
            ctx.company_id()
        )
        .await,
        1,
        "une seule écriture d'achat"
    );
    capture.exiger_un_rejeu("supplier_invoices::create");
}

// ============================================================
// Test 7 — enregistrement des réglages de facturation victime (choix C70, C74)
// ============================================================

/// ⚠️ **Le cycle de ce montage est celui de MariaDB 10.11, la version
/// épinglée** : un `S` tenu par le test, l'`X` de la route en attente — depuis
/// la Story 15-6c, celui du `SELECT … FOR UPDATE` de `before`, et non plus de
/// l'`UPDATE` —, le test qui demande l'`X`. Sur MySQL ≥ 8.0.18 et MariaDB ≥ 11.4.5
/// (MDEV-34877), le détenteur du `S` reçoit l'`X` sans attendre ; le cycle ne
/// s'y forme que par le verrou partagé que l'`INSERT IGNORE` de la route pose
/// sur la ligne existante (R3-7, mesurée vraie sur 10.11.16 — Story 15-5e1,
/// T0). Un rouge du **témoin** après une montée de version dit que le montage
/// ne forme plus son cycle, non que le rejeu est cassé.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn invoice_settings_update_is_replayed_when_it_is_the_deadlock_victim(pool: MySqlPool) {
    let ctx = monter(&pool).await;
    let capture = CaptureRejeu::installer();
    // La ligne des réglages existe (semée) ; la version courante, lue avant la
    // transaction de test.
    let (status, mut corps) = requete(
        &ctx,
        reqwest::Method::GET,
        "/api/v1/company/invoice-settings",
        None,
    )
    .await;
    assert_eq!(status, 200, "{corps}");
    let version = corps["version"].as_i64().unwrap();
    // Un corps qui CHANGE un réglage : un corps identique est un no-op
    // court-circuité avant l'UPDATE, et la route n'attendrait rien.
    corps["invoiceNumberFormat"] = json!("R-{YEAR}-{SEQ:04}");
    let audits_avant: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM audit_log WHERE action = 'company_invoice_settings.updated'",
    )
    .fetch_one(&pool)
    .await
    .unwrap();

    let mut lourde = transaction_lourde(&pool).await;
    sqlx::query(
        "SELECT company_id FROM company_invoice_settings WHERE company_id = ? LOCK IN SHARE MODE",
    )
    .bind(ctx.company_id())
    .fetch_one(&mut *lourde)
    .await
    .unwrap();

    let route = requete_en_tache(
        &ctx,
        reqwest::Method::PUT,
        "/api/v1/company/invoice-settings",
        Some(corps),
    );
    let (status, corps) = victime(
        &pool,
        lourde,
        route,
        // Story 15-6c (AC2) : `before` est lu en `FOR UPDATE` — c'est là, et non
        // plus à l'`UPDATE`, que la route attend le `S` du test (promotion S → X
        // après son `INSERT IGNORE`). Même cycle, autre point d'attente : le motif
        // suit la forme du verrou.
        &["FROM company_invoice_settings", "FOR UPDATE"],
        "SELECT company_id FROM company_invoice_settings WHERE company_id = ? FOR UPDATE",
        ctx.company_id(),
    )
    .await;

    assert_eq!(status, 200, "⛔ l'interblocage doit être rejoué : {corps}");
    assert_eq!(
        corps["version"].as_i64(),
        Some(version + 1),
        "la version incrémentée une seule fois"
    );
    assert_eq!(corps["invoiceNumberFormat"], "R-{YEAR}-{SEQ:04}");
    let audits_apres: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM audit_log WHERE action = 'company_invoice_settings.updated'",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(audits_apres, audits_avant + 1, "une seule entrée d'audit");
    capture.exiger_un_rejeu("company_invoice_settings::update");
}

// ============================================================
// Test 8 — rapprochement manuel victime (Story 15-5e2, AC2 ; choix C86)
// ============================================================

/// Un compte bancaire lié au compte de grand livre `1100` de la société, et
/// une transaction bancaire `pending` datée du jour, de `montant` CHF.
async fn transaction_bancaire_en_attente(
    pool: &MySqlPool,
    ctx: &Contexte,
    montant: &str,
) -> (i64, i64) {
    use kesh_db::entities::{
        BankImportSourceFormat, NewBankAccount, NewBankImport, NewBankTransaction,
    };
    use kesh_db::repositories::{bank_accounts, bank_imports};
    use std::str::FromStr;

    let compte_bancaire = bank_accounts::create(
        pool,
        NewBankAccount {
            company_id: ctx.company_id(),
            bank_name: "UBS".into(),
            iban: "CH4431999123000889013".into(),
            qr_iban: None,
            is_primary: true,
        },
    )
    .await
    .expect("compte bancaire")
    .id;
    sqlx::query(
        "UPDATE bank_accounts SET journal_account_id = ?, version = version + 1 WHERE id = ?",
    )
    .bind(ctx.compte("1100"))
    .bind(compte_bancaire)
    .execute(pool)
    .await
    .unwrap();

    let jour = aujourd_hui();
    let montant = rust_decimal::Decimal::from_str(montant).unwrap();
    let mut tx = pool.begin().await.unwrap();
    let (_, inserees) = bank_imports::create_with_transactions(
        &mut tx,
        NewBankImport {
            company_id: ctx.company_id(),
            bank_account_id: compte_bancaire,
            filename: "releve.xml".into(),
            file_hash: format!("{:0>64}", "rejeu-manuel"),
            source_format: BankImportSourceFormat::Camt053V04,
            statement_id: Some("REJEU-8".into()),
            period_from: jour,
            period_to: jour,
            opening_balance: None,
            closing_balance: None,
            transaction_count: 1,
            imported_by_user_id: ctx.societe.admin_user_id,
        },
        vec![NewBankTransaction {
            company_id: ctx.company_id(),
            bank_account_id: compte_bancaire,
            booking_date: jour,
            value_date: Some(jour),
            amount: montant,
            currency: "CHF".into(),
            reference: Some("REJEU-MANUEL".into()),
            details: "Frais du rejeu".into(),
            end_to_end_id: None,
            transaction_id: None,
            counterparty_iban: None,
            counterparty_name: None,
        }],
    )
    .await
    .expect("import bancaire");
    tx.commit().await.unwrap();
    (compte_bancaire, inserees[0].id)
}

/// ⛔ **Le seul test dynamique de la famille `AppError` avec verrou nommé**
/// (Story 15-5e2, finding F6-1) : 1213 levé sous le verrou nommé du compte
/// bancaire, converti par le `match` de `post_manual_once`, `RELEASE_LOCK`
/// par `with_account_lock`, `rollback`, nouvelle tentative.
///
/// Montage **avec projet** : la route prend l'exercice `FOR UPDATE` (étape 6)
/// **puis** la sentinelle `companies` (étape 6bis, `validate_taggable_in_tx`).
/// La transaction de test, lourde, tient la sentinelle ; la route bloque sur
/// elle ; la transaction de test demande l'exercice : cycle, la route est la
/// victime. Le verrou nommé, hors du graphe d'InnoDB, ne participe pas au
/// cycle — la transaction de test ne le demande pas. Cycle formé à la main
/// sur MariaDB 10.11.16 en T0 (Dev Agent Record de la 15-5e2).
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn manual_match_is_replayed_when_it_is_the_deadlock_victim(pool: MySqlPool) {
    let ctx = monter(&pool).await;
    let exercice = exercice_du_jour(&pool, ctx.company_id()).await;
    let (compte_bancaire, transaction) =
        transaction_bancaire_en_attente(&pool, &ctx, "-150.00").await;
    let projet = sqlx::query(
        "INSERT INTO projects (company_id, parent_id, code, name, archived) \
         VALUES (?, NULL, 'REJEU-8', 'Projet du rejeu', FALSE)",
    )
    .bind(ctx.company_id())
    .execute(&pool)
    .await
    .unwrap()
    .last_insert_id() as i64;
    let ecritures_avant = compter(
        &pool,
        "SELECT COUNT(*) FROM journal_entries WHERE company_id = ?",
        ctx.company_id(),
    )
    .await;
    let capture = CaptureRejeu::installer();

    let mut lourde = transaction_lourde(&pool).await;
    sqlx::query("SELECT id FROM companies WHERE id = ? FOR UPDATE")
        .bind(ctx.company_id())
        .fetch_one(&mut *lourde)
        .await
        .unwrap();

    // La route tient l'exercice (étape 6), puis attend la sentinelle (6bis).
    let route = requete_en_tache(
        &ctx,
        reqwest::Method::POST,
        "/api/v1/reconciliation/manual",
        Some(json!({
            "bankAccountId": compte_bancaire,
            "bankTransactionId": transaction,
            "counterpartyAccountId": ctx.compte("4000"),
            "description": "Frais du rejeu",
            "projectId": projet,
        })),
    );
    // Revue P1 de la 15-5e2 (L-1/B-2) : la route a fait ses pré-lectures et
    // attend la sentinelle ; on marque la transaction bancaire comme rejetée
    // (validée, hors de toute transaction tenue) AVANT de fermer le cycle. La
    // tentative rejouée doit lire cette valeur — l'audit en écrivait une
    // pré-lue, périmée, tant qu'elle était capturée avant la première
    // tentative.
    assert!(
        attendre_une_requete_en_cours(&pool, &["companies", "FOR UPDATE"], || route.is_finished())
            .await,
        "la route devait attendre la sentinelle"
    );
    sqlx::query("UPDATE bank_transactions SET auto_match_rejected_at = NOW(3) WHERE id = ?")
        .bind(transaction)
        .execute(&pool)
        .await
        .unwrap();
    let (status, corps) = victime(
        &pool,
        lourde,
        route,
        &["companies", "FOR UPDATE"],
        "SELECT id FROM fiscal_years WHERE id = ? FOR UPDATE",
        exercice,
    )
    .await;

    assert_eq!(status, 200, "⛔ l'interblocage doit être rejoué : {corps}");
    assert_eq!(
        compter(
            &pool,
            "SELECT COUNT(*) FROM journal_entries WHERE company_id = ?",
            ctx.company_id()
        )
        .await,
        ecritures_avant + 1,
        "une seule écriture créée"
    );
    let (statut, rapprochee_par): (String, Option<i64>) =
        sqlx::query_as("SELECT status, matched_entry_id FROM bank_transactions WHERE id = ?")
            .bind(transaction)
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(
        statut, "reconciled",
        "la transaction bancaire est rapprochée"
    );
    assert_eq!(
        rapprochee_par,
        corps["journalEntryId"].as_i64(),
        "rapprochée une seule fois, par l'écriture rendue"
    );
    assert_eq!(
        compter(
            &pool,
            "SELECT COUNT(*) FROM audit_log WHERE action = 'reconciliation.manual_matched' \
             AND entity_id = ?",
            transaction
        )
        .await,
        1,
        "une seule entrée d'audit"
    );
    let rejetee_avant: Option<String> = sqlx::query_scalar(
        "SELECT CAST(JSON_EXTRACT(details_json, '$.was_previously_rejected') AS CHAR) FROM audit_log \
         WHERE action = 'reconciliation.manual_matched' AND entity_id = ?",
    )
    .bind(transaction)
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(
        rejetee_avant.as_deref(),
        Some("true"),
        "⛔ l'audit écrit ce que la tentative rejouée a lu, pas une pré-lecture périmée"
    );
    capture.exiger_un_rejeu("reconciliation::manual");
}

// ============================================================
// Story 15-6a (test 15) — l'avoir victime sur l'attente neuve de ses comptes
// ============================================================

/// Le rejeu de `POST /api/v1/credit-notes` (posé par la Story 15-5e2) couvre
/// l'attente **neuve** que la Story 15-6a crée : le verrou partagé des comptes
/// de l'avoir, pris avant l'exercice (AC6).
///
/// Montage déterministe (patron du fichier) : la transaction de test, alourdie
/// (table de lest hors de tout flux métier — un lest dans une table applicative
/// poserait des verrous de clé étrangère sur `accounts` et changerait le cycle),
/// tient la créance 1100 en exclusif ; l'avoir, qui tient déjà la facture et les
/// réglages, est vu en attente de son `LOCK IN SHARE MODE` sur `accounts` ; la
/// transaction de test demande alors la facture → cycle, InnoDB annule l'avoir
/// (la plus légère), la transaction de test annule, et le rejeu repart à neuf :
/// 201, un seul avoir, facture annulée, premier numéro d'avoir.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn credit_note_route_replays_when_it_is_the_deadlock_victim(pool: MySqlPool) {
    let ctx = monter(&pool).await;
    disable_rounding_to_5_centimes(&pool, ctx.company_id())
        .await
        .unwrap();
    let capture = CaptureRejeu::installer();
    let facture = facture_validee(&pool, &ctx, "100.00").await;

    // (1) La transaction de test : lourde, puis la créance de la vente.
    let mut lourde = transaction_lourde(&pool).await;
    sqlx::query("SELECT id FROM accounts WHERE id = ? FOR UPDATE")
        .bind(ctx.compte("1100"))
        .fetch_one(&mut *lourde)
        .await
        .unwrap();

    // (2) L'avoir : facture, réglages, puis le verrou partagé de ses comptes.
    let route = requete_en_tache(
        &ctx,
        reqwest::Method::POST,
        "/api/v1/credit-notes",
        Some(json!({ "invoiceId": facture, "date": aujourd_hui().to_string() })),
    );
    // (3) La facture, que tient l'avoir.
    let (status, corps) = victime(
        &pool,
        lourde,
        route,
        &["FROM accounts", "LOCK IN SHARE MODE"],
        "SELECT id FROM invoices WHERE id = ? FOR UPDATE",
        facture,
    )
    .await;

    assert_eq!(status, 201, "⛔ l'interblocage doit être rejoué : {corps}");
    assert_eq!(
        compter(
            &pool,
            "SELECT COUNT(*) FROM credit_notes WHERE invoice_id = ?",
            facture
        )
        .await,
        1,
        "un seul avoir"
    );
    let statut: String = sqlx::query_scalar("SELECT status FROM invoices WHERE id = ?")
        .bind(facture)
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(statut, "cancelled");
    // Premier numéro : la tentative annulée n'a rien consommé (le compteur se
    // rembobine au rollback), la séquence n'a servi qu'une fois.
    assert_eq!(
        compter(
            &pool,
            "SELECT CAST(COALESCE(SUM(next_number - 1), 0) AS SIGNED) FROM credit_note_number_sequences \
             WHERE company_id = ?",
            ctx.company_id()
        )
        .await,
        1,
        "un seul numéro d'avoir tiré : {corps}"
    );
    capture.exiger_un_rejeu("credit_notes::create");
}

// ============================================================
// Tests 9 et 10 — clôture et création d'un exercice victimes (Story 15-12a,
// AC 6 ; #543)
// ============================================================

/// Insère un exercice `année` (1er janvier – 31 décembre) de statut `statut`,
/// par SQL : le montage pose un état, il ne teste pas la création.
async fn exercice_sql(pool: &MySqlPool, company_id: i64, annee: i32, statut: &str) -> i64 {
    sqlx::query(
        "INSERT INTO fiscal_years (company_id, name, start_date, end_date, status) \
         VALUES (?, ?, ?, ?, ?)",
    )
    .bind(company_id)
    .bind(format!("Exercice {annee} (rejeu)"))
    .bind(NaiveDate::from_ymd_opt(annee, 1, 1).unwrap())
    .bind(NaiveDate::from_ymd_opt(annee, 12, 31).unwrap())
    .bind(statut)
    .execute(pool)
    .await
    .unwrap()
    .last_insert_id() as i64
}

/// Test 9 — `POST /fiscal-years/{N}/close`, victime, est **rejouée**.
///
/// Montage : M **clos**, N **ouvert**, M < N (antérieurs à l'exercice
/// 2020-2030 du semis). La transaction lourde tient N ; la clôture, lancée en
/// tâche, verrouille M **par sa clé primaire** à l'étape (b') — explicitement,
/// quel que soit son statut — puis est vue en attente de N à l'étape (c)
/// (motif de `LOCK_IN_COMPANY_SQL` ; l'exécution étant séquentielle, voir (c)
/// en cours prouve que (b') a rendu et tient M). La transaction demande M :
/// cycle ; InnoDB annule la clôture, plus légère ; la transaction obtient M
/// (`.expect`), annule ; la route rejoue et rend **200**, N clos.
///
/// ⛔ **Tue la mutation (ix)** (appeler `fiscal_years::close` hors de son
/// enveloppe dans la route) : 500, et pas de témoin de rejeu.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn fiscal_year_close_is_replayed_when_it_is_the_deadlock_victim(pool: MySqlPool) {
    let ctx = monter(&pool).await;
    let capture = CaptureRejeu::installer();
    let m = exercice_sql(&pool, ctx.company_id(), 2010, "Closed").await;
    let n = exercice_sql(&pool, ctx.company_id(), 2015, "Open").await;

    let mut lourde = transaction_lourde(&pool).await;
    sqlx::query("SELECT id FROM fiscal_years WHERE id = ? FOR UPDATE")
        .bind(n)
        .fetch_one(&mut *lourde)
        .await
        .unwrap();

    let route = requete_en_tache(
        &ctx,
        reqwest::Method::POST,
        &format!("/api/v1/fiscal-years/{n}/close"),
        None,
    );
    let (status, corps) = victime(
        &pool,
        lourde,
        route,
        &["SELECT id, company_id", "WHERE id = ", "FOR UPDATE"],
        "SELECT id FROM fiscal_years WHERE id = ? FOR UPDATE",
        m,
    )
    .await;

    assert_eq!(status, 200, "⛔ l'interblocage doit être rejoué : {corps}");
    assert_eq!(corps["status"], "Closed");
    let audits: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM audit_log WHERE action = 'fiscal_year.closed' AND entity_id = ?",
    )
    .bind(n)
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(audits, 1, "une seule entrée d'audit");
    capture.exiger_un_rejeu("fiscal_years::close");
}

/// Test 10 — `POST /fiscal-years`, victime, est **rejouée**.
///
/// Montage (mesuré en T0, Story 15-12a) : M **clos**, Y **ouvert**, M < X < Y.
/// La transaction lourde tient Y ; la création de X, lancée en tâche, lit — donc
/// verrouille — M dans son pré-contrôle de chevauchement, et bute sur Y (borne
/// de ce parcours, ou sa garde `find_later_closed_in_tx` : le motif tolère les
/// deux, seule la création s'exécute pendant l'attente). La transaction demande
/// M : cycle ; la création, plus légère, est annulée ; la transaction obtient M,
/// annule ; la route rejoue et rend **201**.
///
/// ⛔ **Tue la mutation (x)** (appeler `fiscal_years::create` hors de son
/// enveloppe dans la route) : 500, et pas de témoin de rejeu.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn fiscal_year_creation_is_replayed_when_it_is_the_deadlock_victim(pool: MySqlPool) {
    let ctx = monter(&pool).await;
    let capture = CaptureRejeu::installer();
    let m = exercice_sql(&pool, ctx.company_id(), 2010, "Closed").await;
    let y = exercice_sql(&pool, ctx.company_id(), 2015, "Open").await;

    let mut lourde = transaction_lourde(&pool).await;
    sqlx::query("SELECT id FROM fiscal_years WHERE id = ? FOR UPDATE")
        .bind(y)
        .fetch_one(&mut *lourde)
        .await
        .unwrap();

    let route = requete_en_tache(
        &ctx,
        reqwest::Method::POST,
        "/api/v1/fiscal-years",
        Some(json!({
            "name": "Exercice 2012",
            "startDate": "2012-01-01",
            "endDate": "2012-12-31"
        })),
    );
    let (status, corps) = victime(
        &pool,
        lourde,
        route,
        &["fiscal_years", "FOR UPDATE"],
        "SELECT id FROM fiscal_years WHERE id = ? FOR UPDATE",
        m,
    )
    .await;

    assert_eq!(status, 201, "⛔ l'interblocage doit être rejoué : {corps}");
    assert_eq!(corps["name"], "Exercice 2012");
    let n: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM fiscal_years WHERE company_id = ? AND name = 'Exercice 2012'",
    )
    .bind(ctx.company_id())
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(n, 1, "un seul exercice créé");
    capture.exiger_un_rejeu("fiscal_years::create");
}

// ============================================================
// Story 15-1a2-i (#518) — AC15 (c) : rapprochement ‖ annulation d'un
// règlement de la même facture
// ============================================================

#[path = "../../kesh-db/tests/support/lettering_documents.rs"]
mod lettering_support;

/// Un compte bancaire lié à un compte de banque (1020, distinct de la
/// créance) et une transaction en attente par montant, datées du jour.
async fn transactions_en_attente(
    pool: &MySqlPool,
    ctx: &Contexte,
    reference: &str,
    montants: &[&str],
) -> (i64, Vec<i64>) {
    use kesh_db::entities::{
        BankImportSourceFormat, NewBankAccount, NewBankImport, NewBankTransaction,
    };
    use kesh_db::repositories::{bank_accounts, bank_imports};
    use std::str::FromStr;

    let banque = sqlx::query(
        "INSERT INTO accounts (company_id, number, name, account_type) \
         VALUES (?, '1020', 'Banque du rapprochement', 'Asset')",
    )
    .bind(ctx.company_id())
    .execute(pool)
    .await
    .unwrap()
    .last_insert_id() as i64;
    let compte_bancaire = bank_accounts::create(
        pool,
        NewBankAccount {
            company_id: ctx.company_id(),
            bank_name: "UBS".into(),
            iban: "CH4431999123000889013".into(),
            qr_iban: None,
            is_primary: true,
        },
    )
    .await
    .expect("compte bancaire")
    .id;
    sqlx::query(
        "UPDATE bank_accounts SET journal_account_id = ?, version = version + 1 WHERE id = ?",
    )
    .bind(banque)
    .bind(compte_bancaire)
    .execute(pool)
    .await
    .unwrap();
    let jour = aujourd_hui();
    let mut tx = pool.begin().await.unwrap();
    let (_, inserees) = bank_imports::create_with_transactions(
        &mut tx,
        NewBankImport {
            company_id: ctx.company_id(),
            bank_account_id: compte_bancaire,
            filename: "releve.xml".into(),
            file_hash: format!("{:0>64}", "lettrage-course"),
            source_format: BankImportSourceFormat::Camt053V04,
            statement_id: Some("LETTRAGE".into()),
            period_from: jour,
            period_to: jour,
            opening_balance: None,
            closing_balance: None,
            transaction_count: montants.len() as i32,
            imported_by_user_id: ctx.societe.admin_user_id,
        },
        montants
            .iter()
            .map(|m| NewBankTransaction {
                company_id: ctx.company_id(),
                bank_account_id: compte_bancaire,
                booking_date: jour,
                value_date: Some(jour),
                amount: rust_decimal::Decimal::from_str(m).unwrap(),
                currency: "CHF".into(),
                reference: Some(reference.into()),
                details: "Encaissement".into(),
                end_to_end_id: None,
                transaction_id: None,
                counterparty_iban: None,
                counterparty_name: Some("Client du rejeu".into()),
            })
            .collect(),
    )
    .await
    .expect("import bancaire");
    tx.commit().await.unwrap();
    (compte_bancaire, inserees.iter().map(|t| t.id).collect())
}

/// Une facture validée de 100.— HT (108.10 TTC) réglée de 60.— en espèces par
/// la route ; rend `(facture, règlement)`. Le rapprochement de 48.10 la
/// soldera.
async fn facture_reglee_de_60(pool: &MySqlPool, ctx: &Contexte) -> (i64, i64) {
    let facture = facture_validee(pool, ctx, "100.00").await;
    let (status, corps) = requete(
        ctx,
        reqwest::Method::POST,
        &format!("/api/v1/invoices/{facture}/settlements"),
        Some(json!({
            "settlementType": "internal_account",
            "accountId": ctx.compte("1000"),
            "amount": "60.00",
            "settledOn": aujourd_hui().to_string(),
        })),
    )
    .await;
    assert_eq!(status, 200, "règlement partiel : {corps}");
    let reglement: i64 =
        sqlx::query_scalar("SELECT id FROM invoice_settlements WHERE invoice_id = ?")
            .bind(facture)
            .fetch_one(pool)
            .await
            .unwrap();
    (facture, reglement)
}

fn accepter(
    ctx: &Contexte,
    compte_bancaire: i64,
    tx_id: i64,
    facture: i64,
) -> tokio::task::JoinHandle<(u16, Value)> {
    requete_en_tache(
        ctx,
        reqwest::Method::POST,
        "/api/v1/reconciliation/accept",
        Some(json!({
            "bankAccountId": compte_bancaire,
            "proposals": [{ "type": "invoice", "bankTransactionId": tx_id, "invoiceId": facture }],
        })),
    )
}

/// L'issue d'un `accept` : `true` accepté, `false` refusé per-proposal — et
/// jamais un code d'erreur de lettrage (un refus per-proposal est toujours un
/// refus du rapprochement, jamais `INTERNAL_ERROR`).
fn issue_du_rapprochement(status: u16, corps: &Value) -> bool {
    assert_eq!(status, 200, "pattern batch : {corps}");
    let acceptes = corps["accepted"].as_array().map(Vec::len).unwrap_or(0);
    let refuses = corps["failed"].as_array().cloned().unwrap_or_default();
    assert_eq!(acceptes + refuses.len(), 1, "{corps}");
    for f in &refuses {
        assert_ne!(f["errorCode"], "INTERNAL_ERROR", "{corps}");
    }
    acceptes == 1
}

/// AC15 (c) — entrelacement *rapprochement ‖ annulation d'un règlement de la
/// même facture* : (1) déterministe — le rapprochement attend sur `contacts`
/// (avant toute écriture) pendant que l'annulation valide, puis reprend sur un
/// reste périmé ; (2) cinq courses libres. Les deux gestes finissent toujours
/// (succès, refus per-proposal, refus de la file ou rejeu), et l'état final
/// satisfait AC5 sur toute la société.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn accept_and_settlement_cancel_interleave(pool: MySqlPool) {
    let ctx = monter(&pool).await;
    disable_rounding_to_5_centimes(&pool, ctx.company_id())
        .await
        .unwrap();
    let _ = exercice_du_jour(&pool, ctx.company_id()).await;
    // Six factures entrelacées, plus une septième, TÉMOIN : rapprochée seule,
    // elle doit être acceptée et lettrée — sans quoi un rapprochement toujours
    // refusé (score, fenêtre) laisserait ce test vert à vide.
    let n = 7;
    let factures_reglements = {
        let mut v = Vec::new();
        for _ in 0..n {
            v.push(facture_reglee_de_60(&pool, &ctx).await);
        }
        v
    };
    let (compte_bancaire, txs) =
        transactions_en_attente(&pool, &ctx, "REJEU", &vec!["48.10"; n]).await;

    // (1) Déterministe.
    let (facture, reglement) = factures_reglements[0];
    let mut verrou = pool.acquire().await.unwrap().detach();
    sqlx::query("LOCK TABLES contacts WRITE")
        .execute(&mut verrou)
        .await
        .unwrap();
    let rapprochement = accepter(&ctx, compte_bancaire, txs[0], facture);
    let attend =
        attendre_une_requete_en_cours(&pool, &["FROM contacts"], || rapprochement.is_finished())
            .await;
    assert!(attend, "le rapprochement devait attendre sur `contacts`");
    let (status, corps) = requete(
        &ctx,
        reqwest::Method::POST,
        &format!("/api/v1/invoices/{facture}/settlements/{reglement}/cancel"),
        None,
    )
    .await;
    assert_eq!(
        status, 200,
        "l'annulation passe pendant l'attente : {corps}"
    );
    sqlx::query("UNLOCK TABLES")
        .execute(&mut verrou)
        .await
        .unwrap();
    drop(verrou);
    let (status, corps) = rapprochement.await.unwrap();
    assert!(
        !issue_du_rapprochement(status, &corps),
        "reste périmé : refusé per-proposal ({corps})"
    );

    // Témoin.
    let (temoin, _) = factures_reglements[n - 1];
    let (status, corps) = accepter(&ctx, compte_bancaire, txs[n - 1], temoin)
        .await
        .unwrap();
    assert!(
        issue_du_rapprochement(status, &corps),
        "témoin accepté : {corps}"
    );
    assert!(
        lettering_support::lettree_document(&pool, temoin).await,
        "témoin lettré par le rapprochement"
    );
    // … puis l'annulation de son règlement de 60.— DISSOUT le groupe posé par le
    // rapprochement (revue P2, A2-7 : la partie déterministe ne passait pas par
    // la dissolution).
    let (_, reglement_temoin) = factures_reglements[n - 1];
    let (status, corps) = requete(
        &ctx,
        reqwest::Method::POST,
        &format!("/api/v1/invoices/{temoin}/settlements/{reglement_temoin}/cancel"),
        None,
    )
    .await;
    assert_eq!(status, 200, "annulation après le rapprochement : {corps}");
    assert!(
        !lettering_support::lettree_document(&pool, temoin).await,
        "le groupe du témoin est dissous"
    );
    let dissous: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM audit_log WHERE action = 'lettering.removed' \
         AND JSON_EXTRACT(details_json, '$.documentId') = ?",
    )
    .bind(temoin)
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(dissous, 1, "une dissolution tracée pour le témoin");

    // (2) Courses libres.
    for (i, (facture, reglement)) in factures_reglements.iter().enumerate().take(n - 1).skip(1) {
        let rapprochement = accepter(&ctx, compte_bancaire, txs[i], *facture);
        let annulation = requete_en_tache(
            &ctx,
            reqwest::Method::POST,
            &format!("/api/v1/invoices/{facture}/settlements/{reglement}/cancel"),
            None,
        );
        let (s1, c1) = rapprochement.await.unwrap();
        let (s2, c2) = annulation.await.unwrap();
        issue_du_rapprochement(s1, &c1);
        assert!(
            s2 == 200 || s2 == 409,
            "l'annulation finit (succès, ou refusée parce que le règlement est rapproché \
             ou retiré) : {s2} {c2}"
        );
    }

    let fautes = lettering_support::divergences_ac5(&pool, ctx.company_id(), &[]).await;
    assert!(
        fautes.is_empty(),
        "⛔ AC5 après les entrelacements :\n{}",
        fautes.join("\n")
    );
}
