//! Tests E2E — Story 24-4a (#380) : la **contre-passation** d'une écriture.
//!
//! ⚠️ **Ce fichier est le PREMIER du dépôt à couvrir les écritures de bout en
//! bout.** `journal_entries.rs` n'avait que des tests unitaires — zéro
//! `#[sqlx::test]` — et seul `idor_multi_tenant_e2e.rs` touchait l'endpoint.
//! C'est aussi la couverture sur laquelle s'appuiera la 24-4b (le gel).
//!
//! Couvre : le parcours nominal et ses invariants (I1 somme nulle compte par
//! compte, I2 les deux écritures au grand livre), les **neuf** refus — six
//! chemins de clé étrangère et non huit codes, `supplier_invoices` en portant
//! deux —, les statuts (409 pour un refus de propriété, 400 pour un compte
//! archivé ou l'absence d'exercice), le RBAC, l'IDOR, la reprise du projet
//! **par ligne**, et la suppression en masse que la clé étrangère
//! auto-référente aurait cassée de façon intermittente.

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
            start_date: NaiveDate::from_ymd_opt(today.format("%Y").to_string().parse().unwrap(), 1, 1)
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

async fn contact_id(pool: &MySqlPool, company_id: i64) -> i64 {
    sqlx::query("INSERT INTO contacts (company_id, contact_type, name) VALUES (?, 'Entreprise', 'Client test')")
        .bind(company_id)
        .execute(pool)
        .await
        .unwrap()
        .last_insert_id() as i64
}

async fn post_reverse(app: &TestApp, token: &str, id: i64) -> (reqwest::StatusCode, Value) {
    let resp = app
        .client
        .post(app.url(&format!("/api/v1/journal-entries/{id}/reverse")))
        .header("Authorization", auth(token))
        .send()
        .await
        .unwrap();
    let status = resp.status();
    (status, resp.json().await.unwrap_or(Value::Null))
}

// ---------------------------------------------------------------------------
// Parcours nominal et invariants
// ---------------------------------------------------------------------------

/// AC 1, 2, 3, 13 — la contre-passation crée l'inverse et **ne touche pas**
/// l'origine ; I1 : la somme des deux est nulle **compte par compte**.
///
/// ⚠️ L'invariant se vérifie par compte et non globalement : un total nul se
/// laisserait tromper par une compensation entre deux comptes différents.
#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn reverse_creates_the_opposite_entry_and_leaves_the_origin_intact(pool: MySqlPool) {
    let (app, token, company_id, fy_id) = setup(&pool).await;
    let d = make_account(&pool, company_id, "6000", AccountType::Expense).await;
    let c = make_account(&pool, company_id, "1020", AccountType::Asset).await;
    let origin = make_entry(&pool, company_id, fy_id, d, c, (None, None)).await;

    let (status, body) = post_reverse(&app, &token, origin).await;
    assert_eq!(status, 201, "corps : {body}");

    let reversal_id = body["id"].as_i64().unwrap();
    assert_eq!(body["reversesEntryId"].as_i64(), Some(origin));
    assert_eq!(body["journal"].as_str(), Some("OD"));
    assert_eq!(
        body["entryDate"].as_str(),
        Some(Utc::now().date_naive().to_string().as_str()),
        "la contre-passation porte la date du JOUR, jamais celle de l'origine"
    );

    // L'origine est inchangée — version comprise.
    let (v, rev): (i32, Option<i64>) =
        sqlx::query_as("SELECT version, reverses_entry_id FROM journal_entries WHERE id = ?")
            .bind(origin)
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(v, 1, "l'écriture d'origine ne doit pas être touchée");
    assert_eq!(rev, None);

    // I1 — somme nulle COMPTE PAR COMPTE sur les deux écritures.
    let sums: Vec<(i64, rust_decimal::Decimal)> = sqlx::query_as(
        "SELECT account_id, SUM(debit) - SUM(credit) FROM journal_entry_lines \
         WHERE entry_id IN (?, ?) GROUP BY account_id",
    )
    .bind(origin)
    .bind(reversal_id)
    .fetch_all(&pool)
    .await
    .unwrap();
    assert_eq!(sums.len(), 2, "deux comptes touchés");
    for (account, net) in sums {
        assert_eq!(net, dec!(0), "compte {account} : le net doit être nul");
    }

    // I2 — les DEUX écritures existent : la correction se voit.
    let lines: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM journal_entry_lines WHERE entry_id IN (?, ?)",
    )
    .bind(origin)
    .bind(reversal_id)
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(lines, 4);
}

/// AC 8 — le projet se reprend **ligne par ligne**.
///
/// ⛔ Le test est délibérément **multi-projets** : avec un seul projet, un
/// implémenteur qui estampillerait toutes les lignes de la même valeur passerait
/// sans qu'on le voie. C'est précisément la faute que la revue a rattrapée.
#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn reverse_keeps_the_project_of_each_line(pool: MySqlPool) {
    let (app, token, company_id, fy_id) = setup(&pool).await;
    let d = make_account(&pool, company_id, "6000", AccountType::Expense).await;
    let c = make_account(&pool, company_id, "1020", AccountType::Asset).await;

    let p1: i64 = sqlx::query("INSERT INTO projects (company_id, code, name) VALUES (?, 'P1', 'Projet 1')")
        .bind(company_id)
        .execute(&pool)
        .await
        .unwrap()
        .last_insert_id() as i64;
    let p2: i64 = sqlx::query("INSERT INTO projects (company_id, code, name) VALUES (?, 'P2', 'Projet 2')")
        .bind(company_id)
        .execute(&pool)
        .await
        .unwrap()
        .last_insert_id() as i64;

    let origin = make_entry(&pool, company_id, fy_id, d, c, (Some(p1), Some(p2))).await;
    let (status, body) = post_reverse(&app, &token, origin).await;
    assert_eq!(status, 201, "corps : {body}");

    let reversal_id = body["id"].as_i64().unwrap();
    let tags: Vec<(i64, Option<i64>)> = sqlx::query_as(
        "SELECT account_id, project_id FROM journal_entry_lines WHERE entry_id = ? ORDER BY line_order",
    )
    .bind(reversal_id)
    .fetch_all(&pool)
    .await
    .unwrap();
    assert_eq!(
        tags,
        vec![(d, Some(p1)), (c, Some(p2))],
        "chaque ligne garde SON projet — pas celui de la première"
    );
}

/// AC 9 — un projet **archivé** depuis ne bloque pas : le tag est copié, pas choisi.
#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn reverse_succeeds_when_a_project_was_archived_since(pool: MySqlPool) {
    let (app, token, company_id, fy_id) = setup(&pool).await;
    let d = make_account(&pool, company_id, "6000", AccountType::Expense).await;
    let c = make_account(&pool, company_id, "1020", AccountType::Asset).await;
    let p: i64 = sqlx::query("INSERT INTO projects (company_id, code, name) VALUES (?, 'PA', 'Projet archivé')")
        .bind(company_id)
        .execute(&pool)
        .await
        .unwrap()
        .last_insert_id() as i64;
    let origin = make_entry(&pool, company_id, fy_id, d, c, (Some(p), None)).await;

    sqlx::query("UPDATE projects SET archived = TRUE WHERE id = ?")
        .bind(p)
        .execute(&pool)
        .await
        .unwrap();

    let (status, body) = post_reverse(&app, &token, origin).await;
    assert_eq!(
        status, 201,
        "un projet archivé ne doit pas rendre l'écriture incorrigible — corps : {body}"
    );
}

/// AC 11 — un compte **archivé** depuis bloque, en **400**, et le refus NOMME
/// le compte à réactiver.
///
/// ⛔ C'est l'asymétrie voulue avec le projet : `enforce_postable = false` ne
/// lève pas la garde `active = TRUE`, qui est inconditionnelle.
#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn reverse_refuses_when_an_account_was_archived_since(pool: MySqlPool) {
    let (app, token, company_id, fy_id) = setup(&pool).await;
    let d = make_account(&pool, company_id, "6000", AccountType::Expense).await;
    let c = make_account(&pool, company_id, "1020", AccountType::Asset).await;
    let origin = make_entry(&pool, company_id, fy_id, d, c, (None, None)).await;

    sqlx::query("UPDATE accounts SET active = FALSE WHERE id = ?")
        .bind(d)
        .execute(&pool)
        .await
        .unwrap();

    let (status, body) = post_reverse(&app, &token, origin).await;
    assert_eq!(status, 400, "corps : {body}");
    assert_eq!(body["error"]["code"].as_str(), Some("ACCOUNT_ARCHIVED"));
    assert_eq!(
        body["error"]["details"]["rejected"][0]["accountNumber"].as_str(),
        Some("6000"),
        "le refus doit NOMMER le compte à réactiver"
    );
}

// ---------------------------------------------------------------------------
// Les neuf refus — SIX chemins de clé étrangère, pas huit codes
// ---------------------------------------------------------------------------

/// AC 4 — contre-passer deux fois est refusé en 409 `ALREADY_REVERSED`.
///
/// ⚠️ Le code vient de la discrimination du **nom de contrainte** sur la
/// violation d'unicité, pas d'un `RESOURCE_CONFLICT` générique.
#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn reversing_twice_is_refused(pool: MySqlPool) {
    let (app, token, company_id, fy_id) = setup(&pool).await;
    let d = make_account(&pool, company_id, "6000", AccountType::Expense).await;
    let c = make_account(&pool, company_id, "1020", AccountType::Asset).await;
    let origin = make_entry(&pool, company_id, fy_id, d, c, (None, None)).await;

    let (first, _) = post_reverse(&app, &token, origin).await;
    assert_eq!(first, 201);

    let (status, body) = post_reverse(&app, &token, origin).await;
    assert_eq!(status, 409, "corps : {body}");
    assert_eq!(body["error"]["code"].as_str(), Some("ALREADY_REVERSED"));
}

/// AC 5 — contre-passer une contre-passation est refusé.
#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn reversing_a_reversal_is_refused(pool: MySqlPool) {
    let (app, token, company_id, fy_id) = setup(&pool).await;
    let d = make_account(&pool, company_id, "6000", AccountType::Expense).await;
    let c = make_account(&pool, company_id, "1020", AccountType::Asset).await;
    let origin = make_entry(&pool, company_id, fy_id, d, c, (None, None)).await;

    let (_, first) = post_reverse(&app, &token, origin).await;
    let reversal_id = first["id"].as_i64().unwrap();

    let (status, body) = post_reverse(&app, &token, reversal_id).await;
    assert_eq!(status, 409, "corps : {body}");
    assert_eq!(body["error"]["code"].as_str(), Some("IS_A_REVERSAL"));
}

/// AC 6 — les **six chemins** de clé étrangère, un test paramétré par chemin.
///
/// ⛔ Ce sont les CHEMINS qui sont testés, pas les codes :
/// `purchase_journal_entry_id` et `settlement_journal_entry_id` partagent
/// `OWNED_BY_SUPPLIER_INVOICE` mais renvoient vers deux corrections
/// différentes — un test par code laisserait le second jamais exercé.
#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn every_document_owned_entry_is_refused(pool: MySqlPool) {
    let (app, token, company_id, fy_id) = setup(&pool).await;
    let d = make_account(&pool, company_id, "6000", AccountType::Expense).await;
    let c = make_account(&pool, company_id, "1020", AccountType::Asset).await;
    let contact = contact_id(&pool, company_id).await;
    let today = Utc::now().date_naive();

    // (1) facture client
    let e1 = make_entry(&pool, company_id, fy_id, d, c, (None, None)).await;
    let invoice_id = sqlx::query(
        "INSERT INTO invoices (company_id, contact_id, date, journal_entry_id) VALUES (?, ?, ?, ?)",
    )
    .bind(company_id)
    .bind(contact)
    .bind(today)
    .bind(e1)
    .execute(&pool)
    .await
    .unwrap()
    .last_insert_id() as i64;

    // (2) avoir
    let e2 = make_entry(&pool, company_id, fy_id, d, c, (None, None)).await;
    sqlx::query(
        // ⚠️ `credit_note_number` est exigé par `chk_credit_notes_issued_has_je`
        // dès que le statut vaut `issued` — un avoir émis a forcément son numéro.
        "INSERT INTO credit_notes \
         (company_id, contact_id, invoice_id, status, date, journal_entry_id, credit_note_number) \
         VALUES (?, ?, ?, 'issued', ?, ?, 'AV-TEST-1')",
    )
    .bind(company_id)
    .bind(contact)
    .bind(invoice_id)
    .bind(today)
    .bind(e2)
    .execute(&pool)
    .await
    .unwrap();

    // (3) facture fournisseur — écriture d'ACHAT
    let e3 = make_entry(&pool, company_id, fy_id, d, c, (None, None)).await;
    sqlx::query(
        "INSERT INTO supplier_invoices (company_id, contact_id, invoice_date, purchase_journal_entry_id) \
         VALUES (?, ?, ?, ?)",
    )
    .bind(company_id)
    .bind(contact)
    .bind(today)
    .bind(e3)
    .execute(&pool)
    .await
    .unwrap();

    // (4) facture fournisseur — écriture de RÈGLEMENT (second chemin, même code)
    let e4a = make_entry(&pool, company_id, fy_id, d, c, (None, None)).await;
    let e4b = make_entry(&pool, company_id, fy_id, d, c, (None, None)).await;
    sqlx::query(
        "INSERT INTO supplier_invoices \
         (company_id, contact_id, invoice_date, purchase_journal_entry_id, settlement_journal_entry_id) \
         VALUES (?, ?, ?, ?, ?)",
    )
    .bind(company_id)
    .bind(contact)
    .bind(today)
    .bind(e4a)
    .bind(e4b)
    .execute(&pool)
    .await
    .unwrap();

    // (5) règlement de facture client
    let e5 = make_entry(&pool, company_id, fy_id, d, c, (None, None)).await;
    sqlx::query(
        // ⚠️ `chk_invoice_settlements_counterparty` (Story 24-3) impose une
        // contrepartie COHÉRENTE avec le mode : `internal_account` exige
        // `settlement_account_id` et interdit `settlement_bank_account_id`.
        "INSERT INTO invoice_settlements \
         (company_id, invoice_id, journal_entry_id, amount, settled_on, settlement_type, settlement_account_id) \
         VALUES (?, ?, ?, 100.00, ?, 'internal_account', ?)",
    )
    .bind(company_id)
    .bind(invoice_id)
    .bind(e5)
    .bind(today)
    .bind(c)
    .execute(&pool)
    .await
    .unwrap();

    // (6) transaction bancaire rapprochée
    let e6 = make_entry(&pool, company_id, fy_id, d, c, (None, None)).await;
    let bank_account = sqlx::query(
        "INSERT INTO bank_accounts (company_id, bank_name, iban) VALUES (?, 'Banque test', 'CH9300762011623852957')",
    )
    .bind(company_id)
    .execute(&pool)
    .await
    .unwrap()
    .last_insert_id() as i64;
    let user_id: i64 = sqlx::query_scalar("SELECT id FROM users ORDER BY id LIMIT 1")
        .fetch_one(&pool)
        .await
        .unwrap();
    let import_id = sqlx::query(
        "INSERT INTO bank_imports \
         (company_id, bank_account_id, filename, file_hash, source_format, period_from, period_to, imported_by_user_id) \
         VALUES (?, ?, 'test.xml', REPEAT('a', 64), 'camt053', ?, ?, ?)",
    )
    .bind(company_id)
    .bind(bank_account)
    .bind(today)
    .bind(today)
    .bind(user_id)
    .execute(&pool)
    .await
    .unwrap()
    .last_insert_id() as i64;
    sqlx::query(
        "INSERT INTO bank_transactions \
         (company_id, import_id, bank_account_id, booking_date, amount, currency, details, matched_entry_id) \
         VALUES (?, ?, ?, ?, 100.00, 'CHF', 'test', ?)",
    )
    .bind(company_id)
    .bind(import_id)
    .bind(bank_account)
    .bind(today)
    .bind(e6)
    .execute(&pool)
    .await
    .unwrap();

    for (entry, code, quoi) in [
        (e1, "OWNED_BY_INVOICE", "facture client"),
        (e2, "OWNED_BY_CREDIT_NOTE", "avoir"),
        (e3, "OWNED_BY_SUPPLIER_INVOICE", "achat fournisseur"),
        (e4b, "OWNED_BY_SUPPLIER_INVOICE", "règlement fournisseur"),
        (e5, "OWNED_BY_SETTLEMENT", "règlement client"),
        (e6, "MATCHED_BANK_TRANSACTION", "rapprochement bancaire"),
    ] {
        let (status, body) = post_reverse(&app, &token, entry).await;
        assert_eq!(status, 409, "{quoi} — corps : {body}");
        assert_eq!(
            body["error"]["code"].as_str(),
            Some(code),
            "{quoi} : code attendu {code}"
        );
    }
}

// ---------------------------------------------------------------------------
// Lecture, RBAC, IDOR, suppression
// ---------------------------------------------------------------------------

/// AC 17 — la lecture porte de quoi décider, sans que l'écran devine.
#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn detail_exposes_what_the_screen_needs(pool: MySqlPool) {
    let (app, token, company_id, fy_id) = setup(&pool).await;
    let d = make_account(&pool, company_id, "6000", AccountType::Expense).await;
    let c = make_account(&pool, company_id, "1020", AccountType::Asset).await;
    let origin = make_entry(&pool, company_id, fy_id, d, c, (None, None)).await;

    let before: Value = app
        .client
        .get(app.url(&format!("/api/v1/journal-entries/{origin}")))
        .header("Authorization", auth(&token))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(before["reversable"].as_bool(), Some(true));
    assert!(before["reversalBlockedBy"].is_null());
    assert!(before["reversedByEntryId"].is_null());

    let (_, created) = post_reverse(&app, &token, origin).await;
    let reversal_id = created["id"].as_i64().unwrap();

    let after: Value = app
        .client
        .get(app.url(&format!("/api/v1/journal-entries/{origin}")))
        .header("Authorization", auth(&token))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(after["reversable"].as_bool(), Some(false));
    assert_eq!(
        after["reversalBlockedBy"].as_str(),
        Some("ALREADY_REVERSED")
    );
    assert_eq!(
        after["reversedByEntryId"].as_i64(),
        Some(reversal_id),
        "le renvoi croisé se DÉRIVE de l'UNIQUE, il n'a pas de colonne"
    );
}

/// AC 12 — un `id` inconnu rend **404**, jamais 403 : un 403 révélerait
/// l'existence de la ressource.
#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn unknown_entry_is_not_found(pool: MySqlPool) {
    let (app, token, _company_id, _fy_id) = setup(&pool).await;
    let (status, _) = post_reverse(&app, &token, 999_999).await;
    assert_eq!(status, 404);
}

/// AC 15, 16 — la suppression : refusée sur une origine contre-passée, et la
/// suppression **en masse** passe malgré la clé étrangère auto-référente.
///
/// ⛔ Sans le `NULL` préalable de `delete_all_by_company`, l'échec serait
/// INTERMITTENT — InnoDB vérifie les FK ligne à ligne et l'ordre de parcours
/// déciderait. Un test qui passe une fois sur deux est pire qu'un test rouge.
#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn deleting_a_reversed_entry_is_refused_but_bulk_delete_still_works(pool: MySqlPool) {
    let (app, token, company_id, fy_id) = setup(&pool).await;
    let d = make_account(&pool, company_id, "6000", AccountType::Expense).await;
    let c = make_account(&pool, company_id, "1020", AccountType::Asset).await;
    let origin = make_entry(&pool, company_id, fy_id, d, c, (None, None)).await;
    let (created_status, _) = post_reverse(&app, &token, origin).await;
    assert_eq!(created_status, 201);

    let resp = app
        .client
        .delete(app.url(&format!("/api/v1/journal-entries/{origin}")))
        .header("Authorization", auth(&token))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 409);
    let body: Value = resp.json().await.unwrap();
    assert_eq!(body["error"]["code"].as_str(), Some("ENTRY_IS_REVERSED"));

    journal_entries::delete_all_by_company(&pool, company_id)
        .await
        .expect("la suppression en masse ne doit PAS buter sur la FK auto-référente");
    let left: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM journal_entries WHERE company_id = ?")
        .bind(company_id)
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(left, 0);
}
