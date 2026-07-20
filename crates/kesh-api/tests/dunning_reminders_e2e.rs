//! E2E des routes de rappels débiteurs (Story 21-5a, #231) — liste groupée, RBAC,
//! IDOR, suspension/reprise, rappel manuel (saut de niveau), annulation Admin, gardes.

use std::net::SocketAddr;
use std::path::Path;
use std::sync::Arc;

use chrono::{Duration, TimeDelta, Utc};
use jsonwebtoken::{Algorithm, EncodingKey, Header};
use kesh_api::auth::jwt::Claims;
use kesh_api::auth::password::hash_password;
use kesh_api::config::Config;
use kesh_api::{AppState, build_router};
use kesh_db::entities::address::StructuredAddress;
use kesh_db::entities::{Language, NewCompany, NewUser, OrgType, Role};
use kesh_db::repositories::{companies, users};
use serde_json::{Value, json};
use sqlx::MySqlPool;
use std::sync::atomic::{AtomicI64, Ordering};

const TEST_JWT_SECRET: &[u8] = b"test-secret-32-bytes-minimum-test-secret-padding";
const TEST_ADMIN_PASSWORD: &str = "e2e-test-admin-password";
static SEQ: AtomicI64 = AtomicI64::new(1);

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
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(2);
    loop {
        match tokio::net::TcpStream::connect(addr).await {
            Ok(_) => break,
            Err(_) if std::time::Instant::now() < deadline => {
                tokio::time::sleep(std::time::Duration::from_millis(5)).await;
            }
            Err(e) => panic!("test server not ready: {e}"),
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

async fn create_contact(pool: &MySqlPool, company_id: i64, name: &str, email: Option<&str>) -> i64 {
    sqlx::query_scalar(
        "INSERT INTO contacts (company_id, contact_type, name, email) VALUES (?, 'Personne', ?, ?) RETURNING id",
    )
    .bind(company_id)
    .bind(name)
    .bind(email)
    .fetch_one(pool)
    .await
    .unwrap()
}

async fn create_fiscal_year(pool: &MySqlPool, company_id: i64) -> i64 {
    sqlx::query_scalar(
        "INSERT INTO fiscal_years (company_id, name, start_date, end_date) VALUES (?, '2026', '2026-01-01', '2026-12-31') RETURNING id",
    )
    .bind(company_id)
    .fetch_one(pool)
    .await
    .unwrap()
}

/// Facture validée (avec écriture liée), échéance `days_overdue` jours dans le passé.
async fn validated_invoice(
    pool: &MySqlPool,
    company_id: i64,
    contact_id: i64,
    fy_id: i64,
    days_overdue: i64,
) -> i64 {
    let n = SEQ.fetch_add(1, Ordering::SeqCst);
    let je_id: i64 = sqlx::query_scalar(
        "INSERT INTO journal_entries (company_id, fiscal_year_id, entry_number, entry_date, journal, description) \
         VALUES (?, ?, ?, '2026-06-01', 'Ventes', 'test') RETURNING id",
    )
    .bind(company_id)
    .bind(fy_id)
    .bind(n)
    .fetch_one(pool)
    .await
    .unwrap();
    let due = Utc::now().date_naive() - Duration::days(days_overdue);
    sqlx::query_scalar(
        "INSERT INTO invoices (company_id, contact_id, status, date, due_date, total_amount, journal_entry_id) \
         VALUES (?, ?, 'validated', '2026-01-01', ?, 1000.00, ?) RETURNING id",
    )
    .bind(company_id)
    .bind(contact_id)
    .bind(due)
    .bind(je_id)
    .fetch_one(pool)
    .await
    .unwrap()
}

async fn invoice_version(pool: &MySqlPool, id: i64) -> i32 {
    sqlx::query_scalar("SELECT version FROM invoices WHERE id = ?")
        .bind(id)
        .fetch_one(pool)
        .await
        .unwrap()
}

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn list_grouped_by_contact_with_has_email(pool: MySqlPool) {
    let app = spawn_app(pool.clone()).await;
    let cid = create_company(&pool, "List Co").await;
    let accountant = create_user(&pool, "compta", Role::Comptable, cid).await;
    let token = forge_jwt(accountant, "Comptable", cid);
    let fy = create_fiscal_year(&pool, cid).await;

    let c_email = create_contact(&pool, cid, "Avec Email", Some("a@example.com")).await;
    let c_noemail = create_contact(&pool, cid, "Sans Email", None).await;
    validated_invoice(&pool, cid, c_email, fy, 30).await; // échue depuis 30j → niveau 1 dû
    validated_invoice(&pool, cid, c_noemail, fy, 30).await;

    let res = app
        .client
        .get(app.url("/api/v1/dunning/reminders"))
        .bearer_auth(&token)
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 200);
    let body: Value = res.json().await.unwrap();
    let groups = body["groups"].as_array().unwrap();
    assert_eq!(groups.len(), 2, "deux contacts groupés");
    let with_email = groups.iter().find(|g| g["contactId"] == c_email).unwrap();
    assert_eq!(with_email["hasEmail"], true);
    assert_eq!(with_email["invoices"][0]["currentLevel"], 0);
    assert_eq!(with_email["invoices"][0]["nextLevel"], 1);
    let no_email = groups.iter().find(|g| g["contactId"] == c_noemail).unwrap();
    assert_eq!(no_email["hasEmail"], false);
}

/// MEDIUM-1 (code-review) : deux contacts HOMONYMES ne doivent pas fusionner ni
/// éclater — le groupement est par `contact_id`, pas par nom.
#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn list_groups_homonym_contacts_separately(pool: MySqlPool) {
    let app = spawn_app(pool.clone()).await;
    let cid = create_company(&pool, "Homonym Co").await;
    let accountant = create_user(&pool, "compta", Role::Comptable, cid).await;
    let token = forge_jwt(accountant, "Comptable", cid);
    let fy = create_fiscal_year(&pool, cid).await;

    // Deux contacts de même nom ; le premier a 2 factures échues (échéances qui
    // s'entrelacent avec celle du second), le second en a 1.
    let a = create_contact(&pool, cid, "Jean Dupont", Some("a@example.com")).await;
    let b = create_contact(&pool, cid, "Jean Dupont", Some("b@example.com")).await;
    validated_invoice(&pool, cid, a, fy, 40).await;
    validated_invoice(&pool, cid, b, fy, 35).await;
    validated_invoice(&pool, cid, a, fy, 30).await;

    let body: Value = app
        .client
        .get(app.url("/api/v1/dunning/reminders"))
        .bearer_auth(&token)
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    let groups = body["groups"].as_array().unwrap();
    // Exactement 2 groupes (un par contact_id), contact A non éclaté (2 factures).
    assert_eq!(groups.len(), 2, "un groupe par contact malgré l'homonymie");
    let ga = groups.iter().find(|g| g["contactId"] == a).unwrap();
    assert_eq!(ga["invoices"].as_array().unwrap().len(), 2);
    let gb = groups.iter().find(|g| g["contactId"] == b).unwrap();
    assert_eq!(gb["invoices"].as_array().unwrap().len(), 1);
}

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn pause_resume_toggle_and_note_reset(pool: MySqlPool) {
    let app = spawn_app(pool.clone()).await;
    let cid = create_company(&pool, "Pause Co").await;
    let accountant = create_user(&pool, "compta", Role::Comptable, cid).await;
    let token = forge_jwt(accountant, "Comptable", cid);
    let fy = create_fiscal_year(&pool, cid).await;
    let contact = create_contact(&pool, cid, "Débiteur", Some("d@example.com")).await;
    let inv = validated_invoice(&pool, cid, contact, fy, 30).await;

    // Pause avec note.
    let v0 = invoice_version(&pool, inv).await;
    let res = app
        .client
        .put(app.url(&format!("/api/v1/invoices/{inv}/dunning-pause")))
        .bearer_auth(&token)
        .json(&json!({ "version": v0, "note": "litige en cours" }))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 200);
    let body: Value = res.json().await.unwrap();
    assert!(!body["dunningPausedAt"].is_null());
    assert_eq!(body["dunningPausedNote"], "litige en cours");

    // La facture suspendue disparaît de la liste à rappeler.
    let list: Value = app
        .client
        .get(app.url("/api/v1/dunning/reminders"))
        .bearer_auth(&token)
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(list["groups"].as_array().unwrap().len(), 0);

    // Reprise → paused_at ET note remis à NULL.
    let v1 = body["version"].as_i64().unwrap() as i32;
    let res = app
        .client
        .put(app.url(&format!("/api/v1/invoices/{inv}/dunning-resume")))
        .bearer_auth(&token)
        .json(&json!({ "version": v1 }))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 200);
    let body: Value = res.json().await.unwrap();
    assert!(body["dunningPausedAt"].is_null());
    assert!(body["dunningPausedNote"].is_null());

    // Reprise d'une facture non suspendue → 422 INVOICE_NOT_PAUSED.
    let v2 = body["version"].as_i64().unwrap() as i32;
    let res = app
        .client
        .put(app.url(&format!("/api/v1/invoices/{inv}/dunning-resume")))
        .bearer_auth(&token)
        .json(&json!({ "version": v2 }))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 422);
    let body: Value = res.json().await.unwrap();
    assert_eq!(body["error"]["code"], "INVOICE_NOT_PAUSED");
}

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn manual_reminder_level_jump_and_guards(pool: MySqlPool) {
    let app = spawn_app(pool.clone()).await;
    let cid = create_company(&pool, "Manual Co").await;
    let accountant = create_user(&pool, "compta", Role::Comptable, cid).await;
    let token = forge_jwt(accountant, "Comptable", cid);
    let fy = create_fiscal_year(&pool, cid).await;
    let contact = create_contact(&pool, cid, "Débiteur", Some("d@example.com")).await;
    // Déclenche le seed lazy (3 niveaux).
    let _ = app
        .client
        .get(app.url("/api/v1/dunning/reminders"))
        .bearer_auth(&token)
        .send()
        .await
        .unwrap();
    let inv = validated_invoice(&pool, cid, contact, fy, 40).await;

    // Rappel manuel niveau 3 (saut direct, D18) → 201, current_level avance à 3.
    let sent_at = (Utc::now().naive_utc() - Duration::days(1))
        .format("%Y-%m-%dT%H:%M:%S")
        .to_string();
    let res = app
        .client
        .post(app.url(&format!("/api/v1/invoices/{inv}/reminders/manual")))
        .bearer_auth(&token)
        .json(&json!({ "levelNumber": 3, "sentAt": sent_at, "note": "recommandé envoyé" }))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 201);
    let body: Value = res.json().await.unwrap();
    assert_eq!(body["levelNumber"], 3);
    assert_eq!(body["channel"], "manual");
    assert_eq!(body["feeAmount"], "40.00"); // snapshot niveau 3

    // Historique visible (tous rôles).
    let hist: Value = app
        .client
        .get(app.url(&format!("/api/v1/invoices/{inv}/reminders")))
        .bearer_auth(&token)
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(hist.as_array().unwrap().len(), 1);

    // Garde : date future → 422.
    let future = (Utc::now().naive_utc() + Duration::days(5))
        .format("%Y-%m-%dT%H:%M:%S")
        .to_string();
    let res = app
        .client
        .post(app.url(&format!("/api/v1/invoices/{inv}/reminders/manual")))
        .bearer_auth(&token)
        .json(&json!({ "levelNumber": 1, "sentAt": future }))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 422);
    assert_eq!(
        res.json::<Value>().await.unwrap()["error"]["code"],
        "REMINDER_DATE_IN_FUTURE"
    );

    // Garde : niveau inexistant → 422.
    let res = app
        .client
        .post(app.url(&format!("/api/v1/invoices/{inv}/reminders/manual")))
        .bearer_auth(&token)
        .json(&json!({ "levelNumber": 99, "sentAt": sent_at }))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 422);
    assert_eq!(
        res.json::<Value>().await.unwrap()["error"]["code"],
        "DUNNING_LEVEL_NOT_FOUND"
    );

    // Garde : facture payée → 422 INVOICE_ALREADY_PAID (AC 20).
    let paid = validated_invoice(&pool, cid, contact, fy, 40).await;
    sqlx::query("UPDATE invoices SET paid_at = UTC_TIMESTAMP(6) WHERE id = ?")
        .bind(paid)
        .execute(&pool)
        .await
        .unwrap();
    let res = app
        .client
        .post(app.url(&format!("/api/v1/invoices/{paid}/reminders/manual")))
        .bearer_auth(&token)
        .json(&json!({ "levelNumber": 1, "sentAt": sent_at }))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 422);
    assert_eq!(
        res.json::<Value>().await.unwrap()["error"]["code"],
        "INVOICE_ALREADY_PAID"
    );
}

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn cancel_reminder_admin_only_and_soft(pool: MySqlPool) {
    let app = spawn_app(pool.clone()).await;
    let cid = create_company(&pool, "Cancel Co").await;
    let admin = create_user(&pool, "admin1", Role::Admin, cid).await;
    let accountant = create_user(&pool, "compta", Role::Comptable, cid).await;
    let admin_token = forge_jwt(admin, "Admin", cid);
    let acc_token = forge_jwt(accountant, "Comptable", cid);
    let fy = create_fiscal_year(&pool, cid).await;
    let contact = create_contact(&pool, cid, "Débiteur", Some("d@example.com")).await;
    let _ = app
        .client
        .get(app.url("/api/v1/dunning/reminders"))
        .bearer_auth(&acc_token)
        .send()
        .await
        .unwrap();
    let inv = validated_invoice(&pool, cid, contact, fy, 40).await;

    // Enregistre un rappel niveau 1.
    let sent_at = (Utc::now().naive_utc() - Duration::days(1))
        .format("%Y-%m-%dT%H:%M:%S")
        .to_string();
    let created: Value = app
        .client
        .post(app.url(&format!("/api/v1/invoices/{inv}/reminders/manual")))
        .bearer_auth(&acc_token)
        .json(&json!({ "levelNumber": 1, "sentAt": sent_at }))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    let reminder_id = created["id"].as_i64().unwrap();

    // Comptable ne peut PAS annuler (Admin requis) → 403.
    let res = app
        .client
        .post(app.url(&format!(
            "/api/v1/invoices/{inv}/reminders/{reminder_id}/cancel"
        )))
        .bearer_auth(&acc_token)
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 403);

    // Admin annule → 200, cancelled_at posé.
    let res = app
        .client
        .post(app.url(&format!(
            "/api/v1/invoices/{inv}/reminders/{reminder_id}/cancel"
        )))
        .bearer_auth(&admin_token)
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 200);
    assert!(!res.json::<Value>().await.unwrap()["cancelledAt"].is_null());
}

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn rbac_and_idor(pool: MySqlPool) {
    let app = spawn_app(pool.clone()).await;
    let cid = create_company(&pool, "Tenant A").await;
    let other = create_company(&pool, "Tenant B").await;
    let consult = create_user(&pool, "consult", Role::Consultation, cid).await;
    let accountant = create_user(&pool, "compta", Role::Comptable, cid).await;
    let intruder = create_user(&pool, "intru", Role::Comptable, other).await;
    let consult_token = forge_jwt(consult, "Consultation", cid);
    let acc_token = forge_jwt(accountant, "Comptable", cid);
    let intruder_token = forge_jwt(intruder, "Comptable", other);
    let fy = create_fiscal_year(&pool, cid).await;
    let contact = create_contact(&pool, cid, "Débiteur", Some("d@example.com")).await;
    let inv = validated_invoice(&pool, cid, contact, fy, 30).await;
    let v = invoice_version(&pool, inv).await;

    // Consultation ne peut PAS suspendre (Comptable+ requis) → 403.
    let res = app
        .client
        .put(app.url(&format!("/api/v1/invoices/{inv}/dunning-pause")))
        .bearer_auth(&consult_token)
        .json(&json!({ "version": v }))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 403);

    // Intrus d'un autre tenant → 404 (anti-IDOR, jamais 403).
    let res = app
        .client
        .put(app.url(&format!("/api/v1/invoices/{inv}/dunning-pause")))
        .bearer_auth(&intruder_token)
        .json(&json!({ "version": v }))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 404);

    // Comptable du bon tenant → OK.
    let res = app
        .client
        .put(app.url(&format!("/api/v1/invoices/{inv}/dunning-pause")))
        .bearer_auth(&acc_token)
        .json(&json!({ "version": v }))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 200);
}

// ---------------------------------------------------------------------------
// Story 21-6a (#231, D10) — exposition de la suspension en lecture + filtre
// `?paused=`. Avant cette story, une facture suspendue sortait de la liste à
// rappeler sans qu'aucune surface de lecture ne la signale : elle devenait
// introuvable, donc impossible à réactiver.
// ---------------------------------------------------------------------------

/// Suspend `inv` (avec note optionnelle) via l'endpoint 21-5a et renvoie le body.
async fn pause_invoice(app: &TestApp, pool: &MySqlPool, token: &str, inv: i64, note: Option<&str>) {
    let v = invoice_version(pool, inv).await;
    let payload = match note {
        Some(n) => json!({ "version": v, "note": n }),
        None => json!({ "version": v }),
    };
    let res = app
        .client
        .put(app.url(&format!("/api/v1/invoices/{inv}/dunning-pause")))
        .bearer_auth(token)
        .json(&payload)
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 200, "la suspension doit réussir");
}

/// Récupère les `id` des items d'une réponse paginée `GET /api/v1/invoices`.
fn item_ids(body: &Value) -> Vec<i64> {
    body["items"]
        .as_array()
        .unwrap()
        .iter()
        .map(|i| i["id"].as_i64().unwrap())
        .collect()
}

/// AC 24(a) — le détail facture expose l'état de suspension (et la note).
#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn get_invoice_exposes_dunning_pause_state(pool: MySqlPool) {
    let app = spawn_app(pool.clone()).await;
    let cid = create_company(&pool, "Detail Co").await;
    let accountant = create_user(&pool, "compta", Role::Comptable, cid).await;
    let token = forge_jwt(accountant, "Comptable", cid);
    let fy = create_fiscal_year(&pool, cid).await;
    let contact = create_contact(&pool, cid, "Débiteur", Some("d@example.com")).await;
    let paused_inv = validated_invoice(&pool, cid, contact, fy, 30).await;
    let active_inv = validated_invoice(&pool, cid, contact, fy, 30).await;

    pause_invoice(&app, &pool, &token, paused_inv, Some("litige en cours")).await;

    // Facture suspendue → les 2 champs sont renseignés.
    let body: Value = app
        .client
        .get(app.url(&format!("/api/v1/invoices/{paused_inv}")))
        .bearer_auth(&token)
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert!(
        !body["dunningPausedAt"].is_null(),
        "dunningPausedAt doit être exposé sur le détail"
    );
    assert_eq!(body["dunningPausedNote"], "litige en cours");

    // Facture non suspendue → les 2 champs sont null (et présents).
    let body: Value = app
        .client
        .get(app.url(&format!("/api/v1/invoices/{active_inv}")))
        .bearer_auth(&token)
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert!(body["dunningPausedAt"].is_null());
    assert!(body["dunningPausedNote"].is_null());
}

/// AC 24(b)(c)(d)(e)(g) — la liste expose l'état et le filtre `?paused=` trie
/// correctement, `total` restant cohérent avec `items` (le COUNT partage le
/// prédicat de `push_where_clauses`).
#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn list_invoices_exposes_and_filters_paused(pool: MySqlPool) {
    let app = spawn_app(pool.clone()).await;
    let cid = create_company(&pool, "List Co").await;
    let accountant = create_user(&pool, "compta", Role::Comptable, cid).await;
    let token = forge_jwt(accountant, "Comptable", cid);
    let fy = create_fiscal_year(&pool, cid).await;
    let contact = create_contact(&pool, cid, "Débiteur", Some("d@example.com")).await;
    let paused_inv = validated_invoice(&pool, cid, contact, fy, 30).await;
    let active_inv = validated_invoice(&pool, cid, contact, fy, 30).await;

    pause_invoice(&app, &pool, &token, paused_inv, Some("litige")).await;

    let fetch = async |q: &str| -> Value {
        app.client
            .get(app.url(&format!("/api/v1/invoices{q}")))
            .bearer_auth(&token)
            .send()
            .await
            .unwrap()
            .json()
            .await
            .unwrap()
    };

    // (b) + (e) — sans param : les 2 factures, et l'état est exposé sur les items.
    let body = fetch("").await;
    let ids = item_ids(&body);
    assert_eq!(ids.len(), 2, "défaut = no-op, aucune facture filtrée");
    assert!(ids.contains(&paused_inv) && ids.contains(&active_inv));
    let paused_item = body["items"]
        .as_array()
        .unwrap()
        .iter()
        .find(|i| i["id"].as_i64() == Some(paused_inv))
        .unwrap();
    assert!(
        !paused_item["dunningPausedAt"].is_null(),
        "dunningPausedAt doit être exposé sur les items de la liste"
    );
    assert_eq!(paused_item["dunningPausedNote"], "litige");

    // (c) — ?paused=paused → seule la suspendue.
    let body = fetch("?paused=paused").await;
    assert_eq!(item_ids(&body), vec![paused_inv]);
    // (g) — total cohérent avec items sous filtre.
    assert_eq!(
        body["total"].as_i64().unwrap(),
        1,
        "le COUNT doit partager le prédicat du SELECT"
    );

    // (d) — ?paused=not-paused → la suspendue est absente.
    let body = fetch("?paused=not-paused").await;
    assert_eq!(item_ids(&body), vec![active_inv]);
    assert_eq!(body["total"].as_i64().unwrap(), 1);

    // ?paused=all → explicite mais no-op.
    let body = fetch("?paused=all").await;
    assert_eq!(item_ids(&body).len(), 2);
}

/// AC 24(f) — INVARIANT ANTI-DISSIMULATION D10 (test anti-régression clé).
///
/// Une facture suspendue sort de la liste « à rappeler » et de **nulle part
/// ailleurs**. `push_where_clauses` étant partagé, un défaut de filtre non
/// no-op — ou un `build_due_dates_query` qui cesserait de poser `paused: None`
/// — la ferait disparaître de l'échéancier : le défaut même que 21-6a ferme,
/// réintroduit par sa correction.
#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn paused_invoice_stays_visible_in_due_dates(pool: MySqlPool) {
    let app = spawn_app(pool.clone()).await;
    let cid = create_company(&pool, "Echeancier Co").await;
    let accountant = create_user(&pool, "compta", Role::Comptable, cid).await;
    let token = forge_jwt(accountant, "Comptable", cid);
    let fy = create_fiscal_year(&pool, cid).await;
    let contact = create_contact(&pool, cid, "Débiteur", Some("d@example.com")).await;
    let paused_inv = validated_invoice(&pool, cid, contact, fy, 30).await;

    pause_invoice(&app, &pool, &token, paused_inv, Some("litige")).await;

    // Elle a bien disparu de la liste à rappeler (comportement 21-5a).
    let reminders: Value = app
        .client
        .get(app.url("/api/v1/dunning/reminders"))
        .bearer_auth(&token)
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(reminders["groups"].as_array().unwrap().len(), 0);

    // …mais elle RESTE dans l'échéancier, avec son état de suspension exposé.
    let body: Value = app
        .client
        .get(app.url("/api/v1/invoices/due-dates"))
        .bearer_auth(&token)
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    let ids = item_ids(&body);
    assert!(
        ids.contains(&paused_inv),
        "INVARIANT D10 VIOLÉ : une facture suspendue a disparu de l'échéancier"
    );
    let item = body["items"]
        .as_array()
        .unwrap()
        .iter()
        .find(|i| i["id"].as_i64() == Some(paused_inv))
        .unwrap();
    assert!(!item["dunningPausedAt"].is_null());
}

/// AC 24(h) — valeur inconnue rejetée par serde, aucune validation handler.
#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn list_invoices_rejects_unknown_paused_value(pool: MySqlPool) {
    let app = spawn_app(pool.clone()).await;
    let cid = create_company(&pool, "Bogus Co").await;
    let accountant = create_user(&pool, "compta", Role::Comptable, cid).await;
    let token = forge_jwt(accountant, "Comptable", cid);

    let res = app
        .client
        .get(app.url("/api/v1/invoices?paused=bogus"))
        .bearer_auth(&token)
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 400, "?paused=bogus doit être rejeté");
}
