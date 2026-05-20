//! End-to-end HTTP tests for KF-004 — `update()` no-op ne doit plus bumper version.
//!
//! Couvre les ACs HTTP-observables de la Story 7-3 :
//! - AC #19, #20, #21 : PUT no-op transparent → 200 + `version` inchangée.
//! - AC #22 : deux clients en concurrence soumettant tous deux un payload no-op
//!   reçoivent **200/200** (au lieu de 200/409 sous KF-004).
//! - AC #23 : no-op suivi d'un vrai conflit (modification effective avec
//!   version stale) renvoie toujours 409 — le fix ne masque pas les vrais
//!   conflits.
//! - AC #29 : sous concurrence, comportement initial v0.1 (snapshot stale
//!   `200 OK` possible) **fermé par KF-020 (#49) 2026-05-02** — `invoices::update`
//!   utilise maintenant `SELECT ... FOR UPDATE` (cf. test concurrent
//!   `test_update_concurrent_no_op_vs_mutation_no_stale_snapshot_kf020` dans
//!   `crates/kesh-db/src/repositories/invoices.rs`). Le test concurrent E2E
//!   `no_op_with_parallel_mutation_returns_409_under_concurrency` ci-dessous
//!   (closure KF-021 #50 2026-05-20) couvre la même invariant au niveau
//!   HTTP/API : sous `tokio::join!` réel sur deux requêtes parallèles, le
//!   X-lock `FOR UPDATE` sérialise les deux PUT et la 2ᵉ reçoit 409 stale
//!   (au lieu du `200 stale` que la spec v0.1 d'origine documentait pour
//!   l'absence de FOR UPDATE).

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

/// AC #21 — PUT /api/v1/invoices/{id} avec body identique (header + lignes)
/// → 200 OK + version inchangée + updatedAt inchangé.
///
/// Couvre le cas le plus complexe (replace-all sur les lignes) : le no-op
/// check doit comparer correctement le header ET les lignes ligne-à-ligne
/// (description, quantity, unitPrice, vatRate) pour court-circuiter sans
/// DELETE+INSERT du sous-table `invoice_lines`.
#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn put_invoice_no_op_returns_200_unchanged_version(pool: MySqlPool) {
    truncate_all(&pool).await.expect("truncate");
    let (company_id, _) = create_seeded_company(&pool).await;
    create_company_user(&pool, company_id, "alice", "password123").await;

    let app = spawn_app(pool.clone()).await;
    let token = login(&app, "alice", "password123").await;

    // Setup : créer un contact pour la facture.
    let contact_resp = app
        .client
        .post(app.url("/api/v1/contacts"))
        .header("Authorization", format!("Bearer {token}"))
        .json(&json!({
            "contactType": "Entreprise",
            "name": "Acme Invoice Co",
            "isClient": true,
            "isSupplier": false,
            "address": "Rue 1\n1000 Lausanne",
            "email": null,
            "phone": null,
            "ideNumber": null,
            "defaultPaymentTerms": null
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(contact_resp.status(), 201);
    let contact: serde_json::Value = contact_resp.json().await.unwrap();
    let contact_id = contact["id"].as_i64().unwrap();

    // Créer une facture brouillon avec 2 lignes.
    let create_resp = app
        .client
        .post(app.url("/api/v1/invoices"))
        .header("Authorization", format!("Bearer {token}"))
        .json(&json!({
            "contactId": contact_id,
            "date": "2026-04-29",
            "dueDate": "2026-05-29",
            "paymentTerms": "30 jours net",
            "lines": [
                {
                    "description": "Conseil",
                    "quantity": "2",
                    "unitPrice": "150.00",
                    "vatRate": "8.10"
                },
                {
                    "description": "Frais",
                    "quantity": "1",
                    "unitPrice": "50.00",
                    "vatRate": "8.10"
                }
            ]
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(create_resp.status(), 201);
    let invoice: serde_json::Value = create_resp.json().await.unwrap();
    let id = invoice["id"].as_i64().unwrap();
    let version_initial = invoice["version"].as_i64().unwrap();
    let updated_at_initial = invoice["updatedAt"].as_str().unwrap().to_string();
    let line_ids_initial: Vec<i64> = invoice["lines"]
        .as_array()
        .unwrap()
        .iter()
        .map(|l| l["id"].as_i64().unwrap())
        .collect();
    assert_eq!(line_ids_initial.len(), 2, "facture créée avec 2 lignes");

    // PUT body strictement identique (header + lignes même ordre, mêmes
    // valeurs métier) → no-op.
    let put_resp = app
        .client
        .put(app.url(&format!("/api/v1/invoices/{id}")))
        .header("Authorization", format!("Bearer {token}"))
        .json(&json!({
            "contactId": contact_id,
            "date": "2026-04-29",
            "dueDate": "2026-05-29",
            "paymentTerms": "30 jours net",
            "lines": [
                {
                    "description": "Conseil",
                    "quantity": "2",
                    "unitPrice": "150.00",
                    "vatRate": "8.10"
                },
                {
                    "description": "Frais",
                    "quantity": "1",
                    "unitPrice": "50.00",
                    "vatRate": "8.10"
                }
            ],
            "version": version_initial
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(
        put_resp.status(),
        200,
        "PUT invoice no-op doit renvoyer 200 (KF-004 fix, AC #21)"
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

    // AC #5 (sqlx) → AC #21 (E2E) : les lignes doivent conserver leurs IDs
    // d'origine (pas de DELETE+INSERT) — invariant observable côté API qui
    // valide que le no-op court-circuite avant la phase replace-all.
    let line_ids_after: Vec<i64> = body["lines"]
        .as_array()
        .unwrap()
        .iter()
        .map(|l| l["id"].as_i64().unwrap())
        .collect();
    assert_eq!(
        line_ids_after, line_ids_initial,
        "les IDs de lignes doivent être préservés sur no-op (pas de churn DELETE+INSERT)"
    );
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
    let updated_at_initial = contact["updatedAt"].as_str().unwrap().to_string();

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
    assert_eq!(
        body_a["updatedAt"].as_str().unwrap(),
        updated_at_initial,
        "no-op A : updatedAt doit être inchangé"
    );

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
    assert_eq!(
        body_b["updatedAt"].as_str().unwrap(),
        updated_at_initial,
        "no-op B : updatedAt doit être inchangé (les 2 invariants AC #22 — version ET updatedAt)"
    );
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

/// KF-021 (closes #50) regression detector for KF-020 SELECT FOR UPDATE (closes #49).
///
/// Test concurrent E2E de la fenêtre de race REPEATABLE READ documentée dans
/// l'issue [#49 KF-020](https://github.com/guycorbaz/kesh/issues/49) sur
/// `invoices::update`. Sous l'ancien comportement (plain `SELECT` sans
/// `FOR UPDATE`), deux PUT concurrents — l'un modification effective, l'autre
/// no-op avec snapshot stale — pouvaient produire `200 + body v=N stale` pour
/// le no-op au lieu du `409 OPTIMISTIC_LOCK_CONFLICT` attendu (cf. spec
/// originale issue #50 « comportement v0.1 attendu `200 OK + body snapshot stale` »).
///
/// **Depuis le fix #49** (commit `ebdea4b` `fix(db): KF-020 SELECT FOR UPDATE
/// in invoices::update`), le `SELECT … FOR UPDATE` étape 1 de `update()`
/// sérialise les deux transactions au niveau du X-lock InnoDB : si la mutation
/// gagne la course au X-lock et commit v=N+1 avant que le no-op n'acquière le
/// lock, le no-op ré-SELECT post-lock voit v=N+1, déclenche la version-check
/// applicative et retourne **409**.
///
/// **Régression detector** : si un futur refactor retire accidentellement le
/// `FOR UPDATE` de `invoices::update` (cf. `crates/kesh-db/src/repositories/invoices.rs:674`),
/// le no-op concurrent reviendrait à `200 stale` et ce test échouerait
/// (le compte de `409` post-mutation sur N itérations tomberait à 0) →
/// red signal en CI.
///
/// Cible **entité `invoices`** spécifiquement (PAS `contacts` ni `products`)
/// car le `FOR UPDATE` est appliqué uniquement à `invoices::update` (cf. issue
/// #49 §"Remediation story" — les autres entités variant A étaient hors scope).
///
/// **Approche choisie : stress loop N=20** (Approche 3 du spec 9-5-1d §"Approche
/// concurrence à privilégier"). Approche 1 (`tokio::join!` simple) testée
/// initialement mais non-déterministe : la course est symétrique (si le no-op
/// gagne le X-lock en premier, il commit v=N inchangée, puis la mutation
/// commit v=N+1 → 200/200 légitime sans race-condition observable).
/// Le stress loop N=20 vise un taux de détection ≥ 99% en cumulant des
/// itérations indépendantes (probabilité « jamais aucune mutation-avant-no-op »
/// sur 20 itérations ≈ négligeable). On asserte **au moins 1 cas 200/409**
/// (mutation puis no-op avec X-lock + version-check = 409) sur l'ensemble.
#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn no_op_with_parallel_mutation_returns_409_under_concurrency(pool: MySqlPool) {
    truncate_all(&pool).await.expect("truncate");
    let (company_id, _) = create_seeded_company(&pool).await;
    create_company_user(&pool, company_id, "alice", "password123").await;
    create_company_user(&pool, company_id, "bob", "password123").await;

    let app = spawn_app(pool.clone()).await;
    let token_a = login(&app, "alice", "password123").await;
    let token_b = login(&app, "bob", "password123").await;

    // Setup : créer un contact + une facture brouillon v=N avec 1 ligne.
    // Pattern réutilisé de `put_invoice_no_op_returns_200_unchanged_version` (ligne 345).
    let contact_resp = app
        .client
        .post(app.url("/api/v1/contacts"))
        .header("Authorization", format!("Bearer {token_a}"))
        .json(&json!({
            "contactType": "Entreprise",
            "name": "Race Invoice Co",
            "isClient": true,
            "isSupplier": false,
            "address": "Rue 1\n1000 Lausanne",
            "email": null,
            "phone": null,
            "ideNumber": null,
            "defaultPaymentTerms": null
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(contact_resp.status(), 201);
    let contact: serde_json::Value = contact_resp.json().await.unwrap();
    let contact_id = contact["id"].as_i64().unwrap();

    let create_resp = app
        .client
        .post(app.url("/api/v1/invoices"))
        .header("Authorization", format!("Bearer {token_a}"))
        .json(&json!({
            "contactId": contact_id,
            "date": "2026-04-29",
            "dueDate": "2026-05-29",
            "paymentTerms": "30 jours net",
            "lines": [
                {
                    "description": "Conseil",
                    "quantity": "2",
                    "unitPrice": "150.00",
                    "vatRate": "8.10"
                }
            ]
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(create_resp.status(), 201);
    let invoice: serde_json::Value = create_resp.json().await.unwrap();
    let id = invoice["id"].as_i64().unwrap();

    // Stress loop N=20 : à chaque itération, on (a) ré-récupère la version
    // courante via GET (l'invoice peut être en v=N+k après les itérations
    // précédentes), puis (b) lance `tokio::join!` sur PUT mutation + PUT no-op,
    // les 2 avec la même version stale. La fenêtre de race est exercée si la
    // mutation gagne le X-lock en premier : la 2ᵉ tx (no-op) ré-SELECT post-lock
    // voit la version bumped et retourne 409 (version-check applicative).
    //
    // Anti-flake : si le no-op gagne le X-lock en premier (race symétrique),
    // les 2 retournent 200/200 (commit v=N inchangée puis v=N+1) sans test
    // de la propriété cible. C'est légitime — on cumule 20 itérations pour
    // garantir ≥ 1 cas mutation-en-premier (probabilité d'échec total ≈ 1/2^20
    // sous distribution équiprobable, en pratique encore plus faible vu la
    // serialization MySQL X-lock).
    const N_ITERATIONS: u32 = 20;
    let url = app.url(&format!("/api/v1/invoices/{id}"));
    let client = app.client.clone();
    let mut mutation_409_count = 0u32;
    let mut both_200_count = 0u32;
    let mut other_count = 0u32;

    for iteration in 0..N_ITERATIONS {
        // Récupérer version courante (peut avoir bougé entre itérations).
        let get_resp = client
            .get(&url)
            .header("Authorization", format!("Bearer {token_a}"))
            .send()
            .await
            .unwrap();
        assert_eq!(get_resp.status(), 200, "GET initial iteration {iteration}");
        let current: serde_json::Value = get_resp.json().await.unwrap();
        let v_current = current["version"].as_i64().unwrap();
        let current_unit_price = current["lines"][0]["unitPrice"]
            .as_str()
            .unwrap()
            .to_string();

        // Payload no-op : identique au snapshot courant v=v_current.
        let no_op_body = json!({
            "contactId": contact_id,
            "date": "2026-04-29",
            "dueDate": "2026-05-29",
            "paymentTerms": "30 jours net",
            "lines": [
                {
                    "description": "Conseil",
                    "quantity": "2",
                    "unitPrice": current_unit_price,
                    "vatRate": "8.10"
                }
            ],
            "version": v_current
        });

        // Payload mutation : changement réel sur `unitPrice` (incrément à
        // chaque itération pour éviter no-op si on tombe sur l'ancienne valeur
        // — `total_amount` est server-computed, hors `UpdateInvoiceRequest`).
        let mutated_price = format!("{}.00", 200 + iteration);
        let mutation_body = json!({
            "contactId": contact_id,
            "date": "2026-04-29",
            "dueDate": "2026-05-29",
            "paymentTerms": "30 jours net",
            "lines": [
                {
                    "description": "Conseil",
                    "quantity": "2",
                    "unitPrice": mutated_price,
                    "vatRate": "8.10"
                }
            ],
            "version": v_current
        });

        let url_a = url.clone();
        let url_b = url.clone();
        let client_a = client.clone();
        let client_b = client.clone();
        let token_a_clone = token_a.clone();
        let token_b_clone = token_b.clone();

        let tx_a = async move {
            client_a
                .put(&url_a)
                .header("Authorization", format!("Bearer {token_a_clone}"))
                .json(&mutation_body)
                .send()
                .await
                .unwrap()
        };
        let tx_b = async move {
            client_b
                .put(&url_b)
                .header("Authorization", format!("Bearer {token_b_clone}"))
                .json(&no_op_body)
                .send()
                .await
                .unwrap()
        };

        let (resp_a, resp_b) = tokio::join!(tx_a, tx_b);
        let status_a = resp_a.status();
        let status_b = resp_b.status();

        // Classification des outcomes :
        // - mutation_409 : tx_a=200 (mutation gagne X-lock, commit v+1) + tx_b=409
        //   (no-op perd, ré-SELECT voit v+1, version-check rejette) — cas cible
        //   qui prouve le fix KF-020.
        // - both_200 : tx_b=200 (no-op gagne X-lock, commit v inchangée) + tx_a=200
        //   (mutation post-no-op, commit v+1) — race symétrique légitime.
        // - other : tout autre combinaison (e.g. 409/200, 500/X, etc.) — anomalie.
        match (status_a.as_u16(), status_b.as_u16()) {
            (200, 409) => mutation_409_count += 1,
            (200, 200) => both_200_count += 1,
            _ => other_count += 1,
        }
    }

    // Sans le `SELECT FOR UPDATE` du fix #49, le no-op verrait toujours v=N
    // (snapshot REPEATABLE READ pré-lock), court-circuiterait via
    // `is_no_op_change` et retournerait `200 stale` — `mutation_409_count`
    // resterait à 0 sur les 20 itérations. **C'est l'invariant testé**.
    assert!(
        mutation_409_count >= 1,
        "Régression KF-020 SELECT FOR UPDATE (#49) probable : 0 cas 200/409 sur {N_ITERATIONS} \
         itérations stress loop. Counts: mutation_409={mutation_409_count}, \
         both_200={both_200_count}, other={other_count}. \
         Vérifier `crates/kesh-db/src/repositories/invoices.rs:674` (SELECT … FOR UPDATE)."
    );
    assert_eq!(
        other_count, 0,
        "Anomalie : status combinations inattendues détectées ({other_count}/{N_ITERATIONS} itérations). \
         mutation_409={mutation_409_count}, both_200={both_200_count}, other={other_count}."
    );
}
