//! Tests E2E — Story 25-2-b-1 (#440) : **dévalider une facture**.
//!
//! Ce que ces tests établissent, et qu'aucun test de dépôt ne peut établir :
//!
//! - **le CYCLE complet par le vrai chemin** — valider, dévalider, revalider —
//!   et surtout que le **numéro est le même** au bout, sans que le compteur
//!   `invoice_number_sequences` ait bougé. ⛔ *C'est le test décisif de la
//!   story* : sans lui, chaque cycle brûlerait un numéro et creuserait la
//!   séquence des factures — le défaut que la 25-2-c a fermé pour les écritures.
//! - **le régime des rôles et des clés API** : la route vit dans
//!   `comptable_routes` (Administrateur **et** Comptable, arbitrage du
//!   2026-09-19), et les **clés API y passent** en écriture — « même approche
//!   que Bexio ». C'est un élargissement : jusqu'ici aucune clé ne pouvait
//!   détruire l'écriture d'une facture.
//! - **la garde d'exercice sur le `PUT`** : un brouillon qui porte un numéro ne
//!   se redate pas hors de l'exercice qui l'a émis.

use std::net::SocketAddr;
use std::sync::Arc;

use chrono::TimeDelta;
use kesh_api::config::Config;
use kesh_api::{AppState, build_router};
use kesh_db::entities::contact::{ContactType, NewContact};
use kesh_db::entities::invoice::{NewInvoice, NewInvoiceLine};
use kesh_db::repositories::{contacts, invoices};
use kesh_db::test_fixtures::seed_accounting_company;
use rust_decimal_macros::dec;
use serde_json::json;
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

async fn login(app: &TestApp, username: &str, password: &str) -> String {
    let resp = app
        .client
        .post(app.url("/api/v1/auth/login"))
        .json(&json!({ "username": username, "password": password }))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200, "login {username}");
    let body: serde_json::Value = resp.json().await.unwrap();
    body["accessToken"].as_str().unwrap().to_string()
}

async fn seed_base(pool: &MySqlPool) -> (i64, i64) {
    let seeded = seed_accounting_company(pool)
        .await
        .expect("seed_accounting_company");
    (seeded.admin_user_id, seeded.company_id)
}

async fn seed_contact(pool: &MySqlPool, company_id: i64, admin_id: i64) -> i64 {
    contacts::create(
        pool,
        admin_id,
        NewContact {
            company_id,
            contact_type: ContactType::Personne,
            name: "Client X".into(),
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
    .id
}

/// Crée une facture **brouillon** datée de `date`, sans la valider.
async fn create_draft(
    pool: &MySqlPool,
    company_id: i64,
    contact_id: i64,
    admin_id: i64,
    date: chrono::NaiveDate,
) -> i64 {
    let new = NewInvoice {
        company_id,
        contact_id,
        date,
        due_date: None,
        payment_terms: None,
        lines: vec![NewInvoiceLine {
            revenue_account_id: None,
            description: "Prestation".into(),
            quantity: dec!(1),
            unit_price: dec!(100.00),
            vat_rate: dec!(8.10),
        }],
        project_id: None,
    };
    let (inv, _) = invoices::create(pool, admin_id, new).await.unwrap();
    inv.id
}

async fn etat(pool: &MySqlPool, invoice_id: i64) -> (String, Option<String>, i32, Option<i64>) {
    sqlx::query_as(
        "SELECT status, invoice_number, version, journal_entry_id FROM invoices WHERE id = ?",
    )
    .bind(invoice_id)
    .fetch_one(pool)
    .await
    .unwrap()
}

async fn compteur(pool: &MySqlPool, company_id: i64) -> Option<i64> {
    sqlx::query_scalar("SELECT next_number FROM invoice_number_sequences WHERE company_id = ?")
        .bind(company_id)
        .fetch_optional(pool)
        .await
        .unwrap()
}

// --- Tests -------------------------------------------------------------------

/// ⛔ **LE TEST DÉCISIF DE LA STORY.** Valider → dévalider → revalider rend le
/// **même numéro**, et le compteur n'a **pas bougé** entre les deux
/// validations.
///
/// Sans cela, la story transporterait sur la numérotation des **factures** la
/// maladie que la 25-2-c vient de fermer sur celle des **écritures** : un
/// compteur qui ne redescend pas, et un trou à chaque cycle.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn le_cycle_complet_rend_le_meme_numero_sans_consommer_le_compteur(pool: MySqlPool) {
    let app = spawn_app(pool.clone()).await;
    let (admin_id, company_id) = seed_base(&pool).await;
    let contact_id = seed_contact(&pool, company_id, admin_id).await;
    let invoice_id = create_draft(
        &pool,
        company_id,
        contact_id,
        admin_id,
        chrono::Utc::now().date_naive(),
    )
    .await;
    let token = login(&app, "admin", TEST_ADMIN_PASSWORD).await;

    // (1) Validation : le numéro est tiré du compteur.
    let resp = app
        .client
        .post(app.url(&format!("/api/v1/invoices/{invoice_id}/validate")))
        .bearer_auth(&token)
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let (statut, numero_initial, version, je_id) = etat(&pool, invoice_id).await;
    assert_eq!(statut, "validated");
    let numero_initial = numero_initial.expect("une facture validée porte un numéro");
    let compteur_apres_validation = compteur(&pool, company_id).await;
    let je_id = je_id.expect("une facture validée porte une écriture");

    // (2) Dévalidation : brouillon, numéro conservé, écriture supprimée.
    let resp = app
        .client
        .post(app.url(&format!("/api/v1/invoices/{invoice_id}/unvalidate")))
        .bearer_auth(&token)
        .json(&json!({ "version": version }))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200, "dévalidation refusée");
    // ⛔ **Le CORPS, et pas seulement la base.** Le handler reconstruit la
    // réponse après le commit ; rien d'autre ne vérifie qu'elle porte bien la
    // facture dévalidée AVEC ses lignes, forme que le frontend consomme déjà.
    let corps: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(corps["status"].as_str(), Some("draft"));
    assert!(
        corps["journalEntryId"].is_null(),
        "l'écriture est détachée dans le corps rendu, pas seulement en base"
    );
    assert_eq!(
        corps["lines"].as_array().map(|l| l.len()),
        Some(1),
        "la réponse porte les lignes ; obtenu {corps}"
    );
    let (statut, numero, version, je) = etat(&pool, invoice_id).await;
    assert_eq!(statut, "draft");
    assert_eq!(numero.as_deref(), Some(numero_initial.as_str()));
    assert!(je.is_none(), "`journal_entry_id` doit être NULL");
    let reste: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM journal_entries WHERE id = ?")
        .bind(je_id)
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(reste, 0, "l'écriture comptable doit avoir disparu");
    let _ = version;

    // (3) Revalidation : MÊME numéro, compteur INCHANGÉ.
    let resp = app
        .client
        .post(app.url(&format!("/api/v1/invoices/{invoice_id}/validate")))
        .bearer_auth(&token)
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let (statut, numero_final, _, je_final) = etat(&pool, invoice_id).await;
    assert_eq!(statut, "validated");
    assert_eq!(
        numero_final.as_deref(),
        Some(numero_initial.as_str()),
        "la revalidation doit REPRENDRE le numéro, non en tirer un neuf"
    );
    assert_eq!(
        compteur(&pool, company_id).await,
        compteur_apres_validation,
        "le compteur ne doit PAS avoir bougé : sinon chaque cycle brûle un numéro"
    );
    assert!(
        je_final.is_some_and(|id| id != je_id),
        "l'écriture, elle, est NEUVE — ce n'est pas la même, et c'est voulu"
    );
}

/// Le Comptable dévalide — arbitrage du 2026-09-19, « qui peut valider peut
/// dévalider ». ⚠️ C'est un **élargissement** : la suppression, elle, reste
/// réservée à l'Administrateur.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn le_comptable_peut_devalider(pool: MySqlPool) {
    let app = spawn_app(pool.clone()).await;
    let (admin_id, company_id) = seed_base(&pool).await;
    let contact_id = seed_contact(&pool, company_id, admin_id).await;
    let invoice_id = create_draft(
        &pool,
        company_id,
        contact_id,
        admin_id,
        chrono::Utc::now().date_naive(),
    )
    .await;
    kesh_db::repositories::invoices::validate_invoice(&pool, company_id, invoice_id, admin_id)
        .await
        .expect("validate_invoice");
    let (_, _, version, _) = etat(&pool, invoice_id).await;

    kesh_db::repositories::users::create(
        &pool,
        kesh_db::entities::user::NewUser {
            username: "comptable".into(),
            password_hash: kesh_api::auth::password::hash_password("comptable123").unwrap(),
            role: kesh_db::entities::user::Role::Comptable,
            active: true,
            company_id,
            email: None,
        },
    )
    .await
    .expect("create comptable");
    let token = login(&app, "comptable", "comptable123").await;

    let resp = app
        .client
        .post(app.url(&format!("/api/v1/invoices/{invoice_id}/unvalidate")))
        .bearer_auth(&token)
        .json(&json!({ "version": version }))
        .send()
        .await
        .unwrap();

    assert_eq!(resp.status(), 200, "le Comptable doit pouvoir dévalider");
    assert_eq!(etat(&pool, invoice_id).await.0, "draft");
}

/// ⛔ **Un brouillon NUMÉROTÉ ne change pas d'exercice.** Le numéro vient du
/// compteur de l'exercice qui couvre la date ; le déplacer lui ferait porter le
/// numéro d'une autre séquence, sans que rien ne le signale.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn un_brouillon_numerote_ne_change_pas_d_exercice(pool: MySqlPool) {
    let app = spawn_app(pool.clone()).await;
    let (admin_id, company_id) = seed_base(&pool).await;
    let contact_id = seed_contact(&pool, company_id, admin_id).await;
    let aujourd_hui = chrono::Utc::now().date_naive();
    let invoice_id = create_draft(&pool, company_id, contact_id, admin_id, aujourd_hui).await;
    kesh_db::repositories::invoices::validate_invoice(&pool, company_id, invoice_id, admin_id)
        .await
        .expect("validate_invoice");
    let (_, _, version, _) = etat(&pool, invoice_id).await;
    let token = login(&app, "admin", TEST_ADMIN_PASSWORD).await;
    app.client
        .post(app.url(&format!("/api/v1/invoices/{invoice_id}/unvalidate")))
        .bearer_auth(&token)
        .json(&json!({ "version": version }))
        .send()
        .await
        .unwrap();

    // Un second exercice, contigu, pour y pousser la facture.
    let (_, _, version, _) = etat(&pool, invoice_id).await;
    let (debut, fin): (chrono::NaiveDate, chrono::NaiveDate) = sqlx::query_as(
        "SELECT start_date, end_date FROM fiscal_years WHERE company_id = ? ORDER BY start_date LIMIT 1",
    )
    .bind(company_id)
    .fetch_one(&pool)
    .await
    .unwrap();
    let hors_exercice = fin + TimeDelta::days(1);
    sqlx::query(
        "INSERT INTO fiscal_years (company_id, name, start_date, end_date, status) \
         VALUES (?, 'Exercice suivant', ?, ?, 'Open')",
    )
    .bind(company_id)
    .bind(hors_exercice)
    .bind(hors_exercice + TimeDelta::days(364))
    .execute(&pool)
    .await
    .unwrap();
    let _ = debut;

    let resp = app
        .client
        .put(app.url(&format!("/api/v1/invoices/{invoice_id}")))
        .bearer_auth(&token)
        .json(&json!({
            "contactId": contact_id,
            "date": hors_exercice,
            "version": version,
            "lines": [{
                "description": "Prestation",
                "quantity": "1",
                "unitPrice": "100.00",
                "vatRate": "8.10"
            }]
        }))
        .send()
        .await
        .unwrap();

    assert_eq!(
        resp.status(),
        409,
        "la redatation hors exercice doit être refusée"
    );
    let body: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(
        body["error"]["code"].as_str(),
        Some("INVOICE_NUMBER_FISCAL_YEAR_MISMATCH")
    );
    assert!(
        etat(&pool, invoice_id).await.1.is_some(),
        "le refus ne touche à rien : la facture garde son numéro"
    );
}

/// **Le cas positif de la garde d'exercice**, et il est indispensable : sans
/// lui, une garde qui refuserait TOUTE redatation passerait le test du refus
/// sans que rien ne le dise. Elle ne compare pas deux dates, elle compare deux
/// **exercices couvrants**.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn un_brouillon_numerote_se_redate_dans_son_propre_exercice(pool: MySqlPool) {
    let app = spawn_app(pool.clone()).await;
    let (admin_id, company_id) = seed_base(&pool).await;
    let contact_id = seed_contact(&pool, company_id, admin_id).await;
    let aujourd_hui = chrono::Utc::now().date_naive();
    let invoice_id = create_draft(&pool, company_id, contact_id, admin_id, aujourd_hui).await;
    kesh_db::repositories::invoices::validate_invoice(&pool, company_id, invoice_id, admin_id)
        .await
        .expect("validate_invoice");
    let (_, _, version, _) = etat(&pool, invoice_id).await;
    let token = login(&app, "admin", TEST_ADMIN_PASSWORD).await;
    app.client
        .post(app.url(&format!("/api/v1/invoices/{invoice_id}/unvalidate")))
        .bearer_auth(&token)
        .json(&json!({ "version": version }))
        .send()
        .await
        .unwrap();

    // Une autre date du MÊME exercice : la veille, sauf au premier jour.
    let (debut, _fin): (chrono::NaiveDate, chrono::NaiveDate) = sqlx::query_as(
        "SELECT start_date, end_date FROM fiscal_years WHERE company_id = ? ORDER BY start_date LIMIT 1",
    )
    .bind(company_id)
    .fetch_one(&pool)
    .await
    .unwrap();
    let dans_exercice = if aujourd_hui > debut {
        aujourd_hui - TimeDelta::days(1)
    } else {
        aujourd_hui + TimeDelta::days(1)
    };
    let (_, numero_avant, version, _) = etat(&pool, invoice_id).await;

    let resp = app
        .client
        .put(app.url(&format!("/api/v1/invoices/{invoice_id}")))
        .bearer_auth(&token)
        .json(&json!({
            "contactId": contact_id,
            "date": dans_exercice,
            "version": version,
            "lines": [{
                "description": "Prestation",
                "quantity": "1",
                "unitPrice": "100.00",
                "vatRate": "8.10"
            }]
        }))
        .send()
        .await
        .unwrap();

    assert_eq!(
        resp.status(),
        200,
        "une date du même exercice doit être acceptée"
    );
    let (_, numero_apres, _, _) = etat(&pool, invoice_id).await;
    assert_eq!(
        numero_apres, numero_avant,
        "la redatation ne touche pas au numéro"
    );
}

/// Crée une clé API par l'endpoint HTTP et rend le secret clair.
async fn creer_cle(app: &TestApp, token: &str, nom: &str, scope: &str) -> String {
    let resp = app
        .client
        .post(app.url("/api/v1/settings/api-keys"))
        .bearer_auth(token)
        .json(&json!({ "name": nom, "scope": scope }))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 201, "création de clé");
    let body: serde_json::Value = resp.json().await.unwrap();
    body["key"].as_str().unwrap().to_string()
}

/// ⚠️ **C'est un ÉLARGISSEMENT, et il est voulu** — « même approche que
/// Bexio », dont l'API publique porte `POST /2.0/kb_invoice/{id}/revert_issue`.
/// Jusqu'ici aucune clé API ne pouvait détruire l'écriture d'une facture.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn une_cle_read_write_devalide_une_cle_read_est_refusee(pool: MySqlPool) {
    let app = spawn_app(pool.clone()).await;
    let (admin_id, company_id) = seed_base(&pool).await;
    let contact_id = seed_contact(&pool, company_id, admin_id).await;
    let aujourd_hui = chrono::Utc::now().date_naive();
    let token = login(&app, "admin", TEST_ADMIN_PASSWORD).await;
    let cle_rw = creer_cle(&app, &token, "rw", "read-write").await;
    let cle_ro = creer_cle(&app, &token, "ro", "read").await;

    // 1. La clé en lecture seule est refusée — et la facture ne bouge pas.
    let premiere = create_draft(&pool, company_id, contact_id, admin_id, aujourd_hui).await;
    kesh_db::repositories::invoices::validate_invoice(&pool, company_id, premiere, admin_id)
        .await
        .expect("validate_invoice");
    let (_, _, version, _) = etat(&pool, premiere).await;
    let resp = app
        .client
        .post(app.url(&format!("/api/v1/invoices/{premiere}/unvalidate")))
        .bearer_auth(&cle_ro)
        .json(&json!({ "version": version }))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 403, "clé `read` sur une mutation → 403");
    let body: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(
        body["error"]["code"].as_str(),
        Some("API_KEY_READ_ONLY"),
        "le refus est celui du scope, pas un refus de route"
    );
    assert_eq!(
        etat(&pool, premiere).await.0,
        "validated",
        "le refus ne touche à rien"
    );

    // 2. La clé en écriture dévalide.
    let seconde = create_draft(&pool, company_id, contact_id, admin_id, aujourd_hui).await;
    kesh_db::repositories::invoices::validate_invoice(&pool, company_id, seconde, admin_id)
        .await
        .expect("validate_invoice");
    let (_, _, version, _) = etat(&pool, seconde).await;
    let resp = app
        .client
        .post(app.url(&format!("/api/v1/invoices/{seconde}/unvalidate")))
        .bearer_auth(&cle_rw)
        .json(&json!({ "version": version }))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200, "clé `read-write` → dévalidation admise");
    let (statut, _, _, je) = etat(&pool, seconde).await;
    assert_eq!(statut, "draft");
    assert!(je.is_none(), "l'écriture est détachée puis supprimée");
}

/// ⛔ **Le refus, par la couche HTTP.** Les tests de dépôt établissent le
/// `DbError` ; **aucun** n'établissait le statut, le code ni le `details` que
/// l'appelant reçoit réellement. Une correspondance HTTP fausse — mauvais
/// statut, code interpolé, `details` vide — passait inaperçue.
///
/// *(Trou trouvé en passe 1 de revue, lentille C.)*
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn un_refus_de_devalidation_rend_409_et_son_code(pool: MySqlPool) {
    let app = spawn_app(pool.clone()).await;
    let (admin_id, company_id) = seed_base(&pool).await;
    let contact_id = seed_contact(&pool, company_id, admin_id).await;
    let invoice_id = create_draft(
        &pool,
        company_id,
        contact_id,
        admin_id,
        chrono::Utc::now().date_naive(),
    )
    .await;
    kesh_db::repositories::invoices::validate_invoice(&pool, company_id, invoice_id, admin_id)
        .await
        .expect("validate_invoice");
    sqlx::query("UPDATE invoices SET emailed_at = NOW(3), emailed_to = ? WHERE id = ?")
        .bind("client@example.invalid")
        .bind(invoice_id)
        .execute(&pool)
        .await
        .unwrap();
    let (_, _numero, version, _) = etat(&pool, invoice_id).await;
    let token = login(&app, "admin", TEST_ADMIN_PASSWORD).await;

    let resp = app
        .client
        .post(app.url(&format!("/api/v1/invoices/{invoice_id}/unvalidate")))
        .bearer_auth(&token)
        .json(&json!({ "version": version }))
        .send()
        .await
        .unwrap();

    assert_eq!(resp.status(), 409, "un empêchement rend 409");
    let corps: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(corps["error"]["code"].as_str(), Some("INVOICE_EMAILED"));
    // ⚠️ **`documentNumber` ne porte PAS un numéro pour ce motif** : il porte
    // l'adresse du destinataire, seule information utile pour comprendre le
    // refus. Le champ est générique et son contenu dépend du motif — piège
    // relevé en écrivant ce test, et écrit dans `docs/api-external.md`.
    assert_eq!(
        corps["error"]["details"]["documentNumber"].as_str(),
        Some("client@example.invalid"),
        "le `details` nomme le destinataire ; obtenu {corps}"
    );
    assert_eq!(
        etat(&pool, invoice_id).await.0,
        "validated",
        "le refus ne touche à rien"
    );

    // Et le conflit de version, par la même couche.
    sqlx::query("UPDATE invoices SET emailed_at = NULL WHERE id = ?")
        .bind(invoice_id)
        .execute(&pool)
        .await
        .unwrap();
    let resp = app
        .client
        .post(app.url(&format!("/api/v1/invoices/{invoice_id}/unvalidate")))
        .bearer_auth(&token)
        .json(&json!({ "version": version - 1 }))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 409, "version périmée → 409");
    let corps: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(
        corps["error"]["code"].as_str(),
        Some("OPTIMISTIC_LOCK_CONFLICT")
    );
}
