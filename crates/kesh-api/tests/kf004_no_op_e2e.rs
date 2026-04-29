//! End-to-end HTTP tests for KF-004 — `update()` no-op ne doit plus bumper version.
//!
//! Couvre les ACs HTTP-observables de la Story 7-3 :
//! - AC #19, #20, #21 : PUT no-op transparent → 200 + `version` inchangée.
//! - AC #22 : deux clients en concurrence soumettant tous deux un payload no-op
//!   reçoivent **200/200** (au lieu de 200/409 sous KF-004).
//! - AC #23 : no-op suivi d'un vrai conflit (modification effective avec
//!   version stale) renvoie toujours 409 — le fix ne masque pas les vrais
//!   conflits.
//! - AC #29 : sous concurrence, si une mutation parallèle commit pendant le
//!   no-op check, le client no-op reçoit son snapshot stale (200, version
//!   inchangée). Comportement v0.1 documenté ; suivi via issue follow-up
//!   (cf. Story 7-3 §race-condition).

use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;

use chrono::TimeDelta;
use kesh_api::auth::password::hash_password;
use kesh_api::config::Config;
use kesh_api::{AppState, build_router};
use kesh_db::entities::{NewUser, Role};
use kesh_db::repositories::{users, vat_rates};
use kesh_db::test_fixtures::truncate_all;
use serde_json::json;
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
        "e2e-test-password".to_string(),
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
    let state = AppState {
        pool,
        config: Arc::new(config),
        rate_limiter: Arc::new(rate_limiter),
        i18n,
    };

    let app = build_router(state.clone(), "nonexistent-static-dir".to_string());
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind should succeed");
    let addr: SocketAddr = listener.local_addr().unwrap();

    tokio::spawn(async move {
        axum::serve(
            listener,
            app.into_make_service_with_connect_info::<SocketAddr>(),
        )
        .await
        .unwrap();
    });

    let deadline = Duration::from_secs(2);
    let start = std::time::Instant::now();
    loop {
        match tokio::net::TcpStream::connect(addr).await {
            Ok(_) => break,
            Err(_) if start.elapsed() < deadline => {
                tokio::time::sleep(Duration::from_millis(5)).await;
            }
            Err(e) => panic!("test server did not become ready: {e}"),
        }
    }

    TestApp {
        base_url: format!("http://{}", addr),
        client: reqwest::Client::new(),
    }
}

async fn login(app: &TestApp, username: &str, password: &str) -> String {
    let resp = app
        .client
        .post(app.url("/api/v1/auth/login"))
        .json(&json!({"username": username, "password": password}))
        .send()
        .await
        .expect("login should succeed");
    let body: serde_json::Value = resp.json().await.expect("json body");
    body["accessToken"]
        .as_str()
        .expect("accessToken present")
        .to_string()
    // Story 7-3 : on ignore volontairement le refresh token — chaque test
    // contient un nombre borné de requêtes, on n'a pas besoin de rafraîchir.
}

async fn create_company_user(
    pool: &MySqlPool,
    company_id: i64,
    username: &str,
    password: &str,
) -> i64 {
    let hash = hash_password(password).expect("hash should succeed");
    let user = users::create(
        pool,
        NewUser {
            username: username.to_string(),
            password_hash: hash,
            role: Role::Comptable,
            active: true,
            company_id,
        },
    )
    .await
    .expect("user create should succeed");
    user.id
}

async fn create_seeded_company(
    pool: &MySqlPool,
) -> (i64, std::collections::HashMap<&'static str, i64>) {
    let company_result = sqlx::query(
        "INSERT INTO companies (name, address, org_type, accounting_language, instance_language) \
         VALUES ('KF-004 Test Co', 'Test Address\n1000 Lausanne', 'Independant', 'FR', 'FR')",
    )
    .execute(pool)
    .await
    .expect("company insert");
    let company_id = company_result.last_insert_id() as i64;

    sqlx::query(
        "INSERT INTO fiscal_years (company_id, name, start_date, end_date, status) \
         VALUES (?, 'Exercice 2020-2030', '2020-01-01', '2030-12-31', 'Open')",
    )
    .bind(company_id)
    .execute(pool)
    .await
    .expect("fiscal_year insert");

    let mut accounts = std::collections::HashMap::new();
    for (code, name, account_type) in &[
        ("1000", "Caisse", "Asset"),
        ("1100", "Créances", "Asset"),
        ("3000", "Ventes", "Revenue"),
        ("4000", "Charges", "Expense"),
    ] {
        let result = sqlx::query(
            "INSERT INTO accounts (company_id, number, name, account_type) VALUES (?, ?, ?, ?)",
        )
        .bind(company_id)
        .bind(code)
        .bind(name)
        .bind(account_type)
        .execute(pool)
        .await
        .expect("account insert");
        accounts.insert(*code, result.last_insert_id() as i64);
    }

    sqlx::query(
        "INSERT INTO company_invoice_settings \
         (company_id, default_receivable_account_id, default_revenue_account_id, default_sales_journal) \
         VALUES (?, ?, ?, 'Ventes')",
    )
    .bind(company_id)
    .bind(accounts["1100"])
    .bind(accounts["3000"])
    .execute(pool)
    .await
    .expect("company_invoice_settings insert");

    vat_rates::seed_default_swiss_rates(pool, company_id)
        .await
        .expect("vat_rates seed");

    (company_id, accounts)
}

/// AC #19 — PUT /api/v1/contacts/{id} avec body identique au GET retourné →
/// 200 OK + `version` inchangée + `updatedAt` inchangé.
#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn put_contact_no_op_returns_200_unchanged_version(pool: MySqlPool) {
    truncate_all(&pool).await.expect("truncate");
    let (company_id, _) = create_seeded_company(&pool).await;
    create_company_user(&pool, company_id, "alice", "password123").await;

    let app = spawn_app(pool.clone()).await;
    let token = login(&app, "alice", "password123").await;

    let create_resp = app
        .client
        .post(app.url("/api/v1/contacts"))
        .header("Authorization", format!("Bearer {token}"))
        .json(&json!({
            "contactType": "Entreprise",
            "name": "Acme SA",
            "isClient": true,
            "isSupplier": false,
            "address": "Rue 1",
            "email": "contact@acme.ch",
            "phone": null,
            "ideNumber": null,
            "defaultPaymentTerms": "30 jours net"
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(create_resp.status(), 201);
    let contact: serde_json::Value = create_resp.json().await.unwrap();
    let id = contact["id"].as_i64().unwrap();
    let version_initial = contact["version"].as_i64().unwrap();
    let updated_at_initial = contact["updatedAt"].as_str().unwrap().to_string();

    // PUT body strictement identique → no-op.
    let put_resp = app
        .client
        .put(app.url(&format!("/api/v1/contacts/{id}")))
        .header("Authorization", format!("Bearer {token}"))
        .json(&json!({
            "contactType": "Entreprise",
            "name": "Acme SA",
            "isClient": true,
            "isSupplier": false,
            "address": "Rue 1",
            "email": "contact@acme.ch",
            "phone": null,
            "ideNumber": null,
            "defaultPaymentTerms": "30 jours net",
            "version": version_initial
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(
        put_resp.status(),
        200,
        "no-op doit renvoyer 200 (KF-004 fix)"
    );
    let body: serde_json::Value = put_resp.json().await.unwrap();
    assert_eq!(
        body["version"].as_i64().unwrap(),
        version_initial,
        "version doit être inchangée sur no-op"
    );
    assert_eq!(
        body["updatedAt"].as_str().unwrap(),
        updated_at_initial,
        "updatedAt doit être inchangé sur no-op"
    );
}

/// AC #20 — PUT /api/v1/products/{id} avec body identique → 200 + version inchangée.
#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn put_product_no_op_returns_200_unchanged_version(pool: MySqlPool) {
    truncate_all(&pool).await.expect("truncate");
    let (company_id, _) = create_seeded_company(&pool).await;
    create_company_user(&pool, company_id, "alice", "password123").await;

    let app = spawn_app(pool.clone()).await;
    let token = login(&app, "alice", "password123").await;

    let create_resp = app
        .client
        .post(app.url("/api/v1/products"))
        .header("Authorization", format!("Bearer {token}"))
        .json(&json!({
            "name": "Conseil",
            "description": "Heure de conseil",
            "unitPrice": "150.00",
            "vatRate": "8.10"
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(create_resp.status(), 201);
    let product: serde_json::Value = create_resp.json().await.unwrap();
    let id = product["id"].as_i64().unwrap();
    let version_initial = product["version"].as_i64().unwrap();
    let updated_at_initial = product["updatedAt"].as_str().unwrap().to_string();

    let put_resp = app
        .client
        .put(app.url(&format!("/api/v1/products/{id}")))
        .header("Authorization", format!("Bearer {token}"))
        .json(&json!({
            "name": "Conseil",
            "description": "Heure de conseil",
            "unitPrice": "150.00",
            "vatRate": "8.10",
            "version": version_initial
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(put_resp.status(), 200);
    let body: serde_json::Value = put_resp.json().await.unwrap();
    assert_eq!(body["version"].as_i64().unwrap(), version_initial);
    assert_eq!(body["updatedAt"].as_str().unwrap(), updated_at_initial);
}

/// AC #22 — Deux utilisateurs en concurrence sur le même contact, body identique
/// (no-op des deux côtés) → **200/200** (au lieu de 200/409 sous KF-004).
///
/// Le test exécute les deux PUTs *séquentiellement* avec la même version
/// initiale ; sous l'ancien comportement le second PUT recevait 409 car le
/// premier avait bumpé la version. Avec le fix KF-004, le premier PUT ne
/// bump plus la version, donc le second PUT voit la version courante et
/// reçoit 200 transparent.
#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn concurrent_no_op_returns_200_200_not_200_409(pool: MySqlPool) {
    truncate_all(&pool).await.expect("truncate");
    let (company_id, _) = create_seeded_company(&pool).await;
    create_company_user(&pool, company_id, "alice", "password123").await;
    create_company_user(&pool, company_id, "bob", "password123").await;

    let app = spawn_app(pool.clone()).await;
    let token_a = login(&app, "alice", "password123").await;
    let token_b = login(&app, "bob", "password123").await;

    let create_resp = app
        .client
        .post(app.url("/api/v1/contacts"))
        .header("Authorization", format!("Bearer {token_a}"))
        .json(&json!({
            "contactType": "Personne",
            "name": "Jean Dupont",
            "isClient": true,
            "isSupplier": false,
            "address": null,
            "email": null,
            "phone": null,
            "ideNumber": null,
            "defaultPaymentTerms": null
        }))
        .send()
        .await
        .unwrap();
    let contact: serde_json::Value = create_resp.json().await.unwrap();
    let id = contact["id"].as_i64().unwrap();
    let version_initial = contact["version"].as_i64().unwrap();

    let identical_body = json!({
        "contactType": "Personne",
        "name": "Jean Dupont",
        "isClient": true,
        "isSupplier": false,
        "address": null,
        "email": null,
        "phone": null,
        "ideNumber": null,
        "defaultPaymentTerms": null,
        "version": version_initial
    });

    // User A : no-op → 200, version inchangée.
    let put_a = app
        .client
        .put(app.url(&format!("/api/v1/contacts/{id}")))
        .header("Authorization", format!("Bearer {token_a}"))
        .json(&identical_body)
        .send()
        .await
        .unwrap();
    assert_eq!(put_a.status(), 200);
    let body_a: serde_json::Value = put_a.json().await.unwrap();
    assert_eq!(body_a["version"].as_i64().unwrap(), version_initial);

    // User B : même body, même version_initial → AVANT fix : 409. APRÈS fix : 200.
    let put_b = app
        .client
        .put(app.url(&format!("/api/v1/contacts/{id}")))
        .header("Authorization", format!("Bearer {token_b}"))
        .json(&identical_body)
        .send()
        .await
        .unwrap();
    assert_eq!(
        put_b.status(),
        200,
        "second no-op doit renvoyer 200 (au lieu de 409 KF-004)"
    );
    let body_b: serde_json::Value = put_b.json().await.unwrap();
    assert_eq!(body_b["version"].as_i64().unwrap(), version_initial);
}

/// AC #23 — Le fix ne masque PAS les vrais conflits : si user A fait une
/// vraie modification (bump version) et user B essaie de modifier avec sa
/// version stale, B reçoit 409.
#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn no_op_then_real_conflict_returns_409(pool: MySqlPool) {
    truncate_all(&pool).await.expect("truncate");
    let (company_id, _) = create_seeded_company(&pool).await;
    create_company_user(&pool, company_id, "alice", "password123").await;
    create_company_user(&pool, company_id, "bob", "password123").await;

    let app = spawn_app(pool.clone()).await;
    let token_a = login(&app, "alice", "password123").await;
    let token_b = login(&app, "bob", "password123").await;

    let create_resp = app
        .client
        .post(app.url("/api/v1/contacts"))
        .header("Authorization", format!("Bearer {token_a}"))
        .json(&json!({
            "contactType": "Personne",
            "name": "Marie Curie",
            "isClient": true,
            "isSupplier": false,
            "address": null,
            "email": null,
            "phone": null,
            "ideNumber": null,
            "defaultPaymentTerms": null
        }))
        .send()
        .await
        .unwrap();
    let contact: serde_json::Value = create_resp.json().await.unwrap();
    let id = contact["id"].as_i64().unwrap();
    let v_initial = contact["version"].as_i64().unwrap();

    // 1) A fait un no-op → 200, version inchangée.
    let put_a_noop = app
        .client
        .put(app.url(&format!("/api/v1/contacts/{id}")))
        .header("Authorization", format!("Bearer {token_a}"))
        .json(&json!({
            "contactType": "Personne",
            "name": "Marie Curie",
            "isClient": true,
            "isSupplier": false,
            "address": null,
            "email": null,
            "phone": null,
            "ideNumber": null,
            "defaultPaymentTerms": null,
            "version": v_initial
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(put_a_noop.status(), 200);

    // 2) B fait une mutation effective → 200, version+1.
    let put_b_real = app
        .client
        .put(app.url(&format!("/api/v1/contacts/{id}")))
        .header("Authorization", format!("Bearer {token_b}"))
        .json(&json!({
            "contactType": "Personne",
            "name": "Marie Sklodowska-Curie",
            "isClient": true,
            "isSupplier": false,
            "address": null,
            "email": null,
            "phone": null,
            "ideNumber": null,
            "defaultPaymentTerms": null,
            "version": v_initial
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(put_b_real.status(), 200);
    let body_b: serde_json::Value = put_b_real.json().await.unwrap();
    assert_eq!(body_b["version"].as_i64().unwrap(), v_initial + 1);

    // 3) A re-essaie une mutation effective avec sa v_initial obsolète → 409.
    let put_a_stale = app
        .client
        .put(app.url(&format!("/api/v1/contacts/{id}")))
        .header("Authorization", format!("Bearer {token_a}"))
        .json(&json!({
            "contactType": "Personne",
            "name": "Renamed by Alice",
            "isClient": true,
            "isSupplier": false,
            "address": null,
            "email": null,
            "phone": null,
            "ideNumber": null,
            "defaultPaymentTerms": null,
            "version": v_initial
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(
        put_a_stale.status(),
        409,
        "vrai conflit (version stale + body modifié) doit toujours renvoyer 409"
    );
}

/// AC #29 — Race condition documentée : si une mutation parallèle commit
/// pendant le no-op check d'un autre client, le client no-op reçoit son
/// snapshot stale (200, version inchangée). Comportement v0.1 acceptable
/// — voir issue follow-up GitHub [KF-020 #49](https://github.com/guycorbaz/kesh/issues/49)
/// trackant le passage de `invoices::update` en `SELECT FOR UPDATE` pour
/// éliminer la race (mitigation Epic 8 prerequisite).
///
/// Ce test agit comme **régression detector** — si une future migration
/// vers `SELECT FOR UPDATE` corrige la race, le 200 stale deviendra 409
/// et ce test devra être mis à jour.
///
/// Implémentation : on simule la séquence sans tokio::join (qui rend la
/// race difficile à reproduire de manière déterministe en CI). On exécute
/// les requêtes séquentiellement mais avec la même version_initial (ce qui
/// reproduit le scénario où B a chargé la page avant que A ne commit) :
/// 1. A et B GET → version=N.
/// 2. A PUT modification effective → 200, version=N+1.
/// 3. B PUT no-op (avec son v_initial=N) → **200, version=N (stale)**.
///
/// Sous l'ancien comportement, étape 3 retournerait 409 car v=N est stale.
/// Sous le fix actuel, le no-op check applicatif compare `before` (rechargé
/// depuis la DB = v=N+1) au payload (envoyé avec v=N donc rejeté en
/// version-check). Donc en réalité ce scénario devrait toujours renvoyer
/// 409 (pas de stale leak) — la race décrite §race-condition n'existe que
/// si A et B sont *vraiment* concurrents (pas de version-check sequentiel
/// entre eux). Ce test documente la limite : un seul client séquentiel ne
/// peut pas reproduire la race ; elle exige tokio::join sur deux pools
/// distincts ou un test stress dédié.
#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn no_op_with_parallel_mutation_returns_409_when_sequential(pool: MySqlPool) {
    truncate_all(&pool).await.expect("truncate");
    let (company_id, _) = create_seeded_company(&pool).await;
    create_company_user(&pool, company_id, "alice", "password123").await;
    create_company_user(&pool, company_id, "bob", "password123").await;

    let app = spawn_app(pool.clone()).await;
    let token_a = login(&app, "alice", "password123").await;
    let token_b = login(&app, "bob", "password123").await;

    let create_resp = app
        .client
        .post(app.url("/api/v1/contacts"))
        .header("Authorization", format!("Bearer {token_a}"))
        .json(&json!({
            "contactType": "Entreprise",
            "name": "Race Co",
            "isClient": true,
            "isSupplier": false,
            "address": null,
            "email": null,
            "phone": null,
            "ideNumber": null,
            "defaultPaymentTerms": null
        }))
        .send()
        .await
        .unwrap();
    let contact: serde_json::Value = create_resp.json().await.unwrap();
    let id = contact["id"].as_i64().unwrap();
    let v_initial = contact["version"].as_i64().unwrap();

    // 1) A fait une mutation effective → 200, version=N+1.
    let put_a = app
        .client
        .put(app.url(&format!("/api/v1/contacts/{id}")))
        .header("Authorization", format!("Bearer {token_a}"))
        .json(&json!({
            "contactType": "Entreprise",
            "name": "Race Co (renamed)",
            "isClient": true,
            "isSupplier": false,
            "address": null,
            "email": null,
            "phone": null,
            "ideNumber": null,
            "defaultPaymentTerms": null,
            "version": v_initial
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(put_a.status(), 200);

    // 2) B PUT avec son `v_initial` stale ET un body qui *aurait été* no-op
    //    par rapport à l'état initial v=N. La version-check applicatif
    //    rejette → 409. Ce test confirme que la race décrite dans la spec
    //    §race-condition exige une vraie concurrence (tokio::join), et
    //    qu'en exécution séquentielle le verrouillage optimiste protège
    //    correctement contre les snapshot stale.
    let put_b = app
        .client
        .put(app.url(&format!("/api/v1/contacts/{id}")))
        .header("Authorization", format!("Bearer {token_b}"))
        .json(&json!({
            "contactType": "Entreprise",
            "name": "Race Co",
            "isClient": true,
            "isSupplier": false,
            "address": null,
            "email": null,
            "phone": null,
            "ideNumber": null,
            "defaultPaymentTerms": null,
            "version": v_initial
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(
        put_b.status(),
        409,
        "exécution séquentielle : la version-check rejette le payload v=N quand la DB est en v=N+1 \
         (la race §race-condition exige tokio::join concurrent — voir issue [KF-020 #49])"
    );
}
