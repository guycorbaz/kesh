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
#[sqlx::test(migrations = "../kesh-db/test-schema")]
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

    // ⛔ **L'origine est inchangée — les SIX champs, pas deux.** Se contenter de
    // `version` laisserait passer une réécriture qui ne bumpe pas la version, et
    // c'est précisément le geste que cette story interdit.
    let (number, date, journal, description, version, rev): (
        i64,
        NaiveDate,
        String,
        String,
        i32,
        Option<i64>,
    ) = sqlx::query_as(
        "SELECT entry_number, entry_date, journal, description, version, reverses_entry_id \
         FROM journal_entries WHERE id = ?",
    )
    .bind(origin)
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(number, 1);
    assert_eq!(date, Utc::now().date_naive());
    assert_eq!(journal, "OD");
    assert_eq!(description, "Écriture à corriger");
    assert_eq!(version, 1, "l'écriture d'origine ne doit pas être touchée");
    assert_eq!(rev, None);

    // …et ses LIGNES, que la spec nomme en premier.
    let origin_lines: Vec<(i64, rust_decimal::Decimal, rust_decimal::Decimal)> = sqlx::query_as(
        "SELECT account_id, debit, credit FROM journal_entry_lines \
         WHERE entry_id = ? ORDER BY line_order",
    )
    .bind(origin)
    .fetch_all(&pool)
    .await
    .unwrap();
    assert_eq!(
        origin_lines,
        vec![(d, dec!(100.00), dec!(0)), (c, dec!(0), dec!(100.00))],
        "les lignes de l'origine sont intactes"
    );

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
    let lines: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM journal_entry_lines WHERE entry_id IN (?, ?)")
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
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn reverse_keeps_the_project_of_each_line(pool: MySqlPool) {
    let (app, token, company_id, fy_id) = setup(&pool).await;
    let d = make_account(&pool, company_id, "6000", AccountType::Expense).await;
    let c = make_account(&pool, company_id, "1020", AccountType::Asset).await;

    let p1: i64 =
        sqlx::query("INSERT INTO projects (company_id, code, name) VALUES (?, 'P1', 'Projet 1')")
            .bind(company_id)
            .execute(&pool)
            .await
            .unwrap()
            .last_insert_id() as i64;
    let p2: i64 =
        sqlx::query("INSERT INTO projects (company_id, code, name) VALUES (?, 'P2', 'Projet 2')")
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
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn reverse_succeeds_when_a_project_was_archived_since(pool: MySqlPool) {
    let (app, token, company_id, fy_id) = setup(&pool).await;
    let d = make_account(&pool, company_id, "6000", AccountType::Expense).await;
    let c = make_account(&pool, company_id, "1020", AccountType::Asset).await;
    let p: i64 = sqlx::query(
        "INSERT INTO projects (company_id, code, name) VALUES (?, 'PA', 'Projet archivé')",
    )
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
#[sqlx::test(migrations = "../kesh-db/test-schema")]
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
#[sqlx::test(migrations = "../kesh-db/test-schema")]
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
#[sqlx::test(migrations = "../kesh-db/test-schema")]
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
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn every_document_owned_entry_is_refused(pool: MySqlPool) {
    let (app, token, company_id, fy_id) = setup(&pool).await;
    let d = make_account(&pool, company_id, "6000", AccountType::Expense).await;
    let c = make_account(&pool, company_id, "1020", AccountType::Asset).await;
    let contact = contact_id(&pool, company_id).await;
    let today = Utc::now().date_naive();

    // (1) facture client
    let e1 = make_entry(&pool, company_id, fy_id, d, c, (None, None)).await;
    let invoice_id = sqlx::query(
        "INSERT INTO invoices (company_id, contact_id, date, journal_entry_id, invoice_number) \
         VALUES (?, ?, ?, ?, 'F-2026-014')",
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
        // ⛔ Le message doit nommer le chemin de CORRECTION, pas seulement
        // interdire : un « interdit » sec n'est pas utilisable.
        assert!(
            !body["error"]["message"]
                .as_str()
                .unwrap_or_default()
                .is_empty(),
            "{quoi} : message vide"
        );
    }

    // ⚠️ **Et il nomme la PIÈCE quand elle a un numéro.** Un `documentId` brut
    // ne se comprend pas : l'utilisateur connaît le numéro de son document, pas
    // les identifiants de la base.
    let (_, body) = post_reverse(&app, &token, e1).await;
    assert!(
        body["error"]["message"]
            .as_str()
            .unwrap_or_default()
            .contains("F-2026-014"),
        "le message doit nommer la facture : {body}"
    );
    assert_eq!(
        body["error"]["details"]["documentNumber"].as_str(),
        Some("F-2026-014")
    );
}

// ---------------------------------------------------------------------------
// Lecture, RBAC, IDOR, suppression
// ---------------------------------------------------------------------------

/// AC 17 — la lecture porte de quoi décider, sans que l'écran devine.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
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
#[sqlx::test(migrations = "../kesh-db/test-schema")]
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
#[sqlx::test(migrations = "../kesh-db/test-schema")]
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

// ---------------------------------------------------------------------------
// Ajouts de la passe 1 de revue de code — ce que les trois lentilles ont trouvé
// ---------------------------------------------------------------------------

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

/// AC 12 — **Consultation ne contre-passe pas.**
///
/// ⚠️ `rbac_e2e.rs` ne couvre qu'un handler synthétique : sans ce test, un
/// futur déplacement de la route hors de `comptable_routes` passerait tous les
/// gates sans qu'aucun ne rougisse.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn consultation_cannot_reverse(pool: MySqlPool) {
    let (app, token, company_id, fy_id) = setup(&pool).await;
    let d = make_account(&pool, company_id, "6000", AccountType::Expense).await;
    let c = make_account(&pool, company_id, "1020", AccountType::Asset).await;
    let origin = make_entry(&pool, company_id, fy_id, d, c, (None, None)).await;

    let readonly = consultation_token(&app, &pool).await;
    let (status, _) = post_reverse(&app, &readonly, origin).await;
    assert_eq!(status, 403, "Consultation ne contre-passe pas");

    // …et le rôle Comptable+ passe, sur la MÊME écriture : sans cette moitié,
    // le test resterait vert si la route devenait inaccessible à tout le monde.
    let (ok, _) = post_reverse(&app, &token, origin).await;
    assert_eq!(ok, 201);
}

/// AC 12 — une écriture d'une **autre société** rend 404, jamais 403.
///
/// ⛔ Un 403 révélerait l'existence de la ressource. C'est la convention IDOR du
/// dépôt, et elle se teste sur la route réelle, pas sur une route voisine.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn another_company_entry_is_not_found(pool: MySqlPool) {
    let (app, token, company_id, fy_id) = setup(&pool).await;
    let d = make_account(&pool, company_id, "6000", AccountType::Expense).await;
    let c = make_account(&pool, company_id, "1020", AccountType::Asset).await;
    let mine = make_entry(&pool, company_id, fy_id, d, c, (None, None)).await;

    // Une seconde société, avec son exercice, ses comptes et son écriture.
    let other_company: i64 = sqlx::query(
        "INSERT INTO companies (name, address, org_type, accounting_language, instance_language) \
         SELECT CONCAT(name, ' bis'), address, org_type, accounting_language, instance_language \
         FROM companies WHERE id = ?",
    )
    .bind(company_id)
    .execute(&pool)
    .await
    .unwrap()
    .last_insert_id() as i64;

    let today = Utc::now().date_naive();
    let year: i32 = today.format("%Y").to_string().parse().unwrap();
    let other_fy = fiscal_years::create(
        &pool,
        1,
        NewFiscalYear {
            company_id: other_company,
            name: format!("Exercice {year}"),
            start_date: NaiveDate::from_ymd_opt(year, 1, 1).unwrap(),
            end_date: NaiveDate::from_ymd_opt(year, 12, 31).unwrap(),
        },
    )
    .await
    .expect("exercice de l'autre société");
    let od = make_account(&pool, other_company, "6000", AccountType::Expense).await;
    let oc = make_account(&pool, other_company, "1020", AccountType::Asset).await;
    let theirs = make_entry(&pool, other_company, other_fy.id, od, oc, (None, None)).await;

    let (status, _) = post_reverse(&app, &token, theirs).await;
    assert_eq!(
        status, 404,
        "l'écriture d'une autre société est INTROUVABLE"
    );

    // Contrôle de sanité : la mienne, elle, se contre-passe — sinon ce test
    // resterait vert avec une route cassée pour tout le monde.
    let (ok, _) = post_reverse(&app, &token, mine).await;
    assert_eq!(ok, 201);
}

/// AC 11 + AC 17 — **le compte archivé se voit AVANT le clic.**
///
/// ⛔ Le défaut que ce test ferme : le recensement des empêchements ignorait
/// l'archivage, si bien que la fiche affichait un bouton « Contre-passer » qui
/// échouait en 400 une fois cliqué. Relevé en passe 1 de revue de code.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn detail_reports_an_archived_account_before_the_click(pool: MySqlPool) {
    let (app, token, company_id, fy_id) = setup(&pool).await;
    let d = make_account(&pool, company_id, "6000", AccountType::Expense).await;
    let c = make_account(&pool, company_id, "1020", AccountType::Asset).await;
    let origin = make_entry(&pool, company_id, fy_id, d, c, (None, None)).await;

    sqlx::query("UPDATE accounts SET active = FALSE WHERE id = ?")
        .bind(d)
        .execute(&pool)
        .await
        .unwrap();

    let detail: Value = app
        .client
        .get(app.url(&format!("/api/v1/journal-entries/{origin}")))
        .header("Authorization", auth(&token))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(
        detail["reversable"].as_bool(),
        Some(false),
        "le bouton doit être masqué AVANT le clic"
    );
    assert_eq!(
        detail["reversalBlockedBy"].as_str(),
        Some("ACCOUNT_ARCHIVED")
    );
}

/// AC 17 — **la précédence est figée** : un motif de propriété passe devant le
/// compte archivé, parce que réactiver le compte ne rendrait pas l'écriture
/// contre-passable pour autant.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn ownership_outranks_the_archived_account(pool: MySqlPool) {
    let (app, token, company_id, fy_id) = setup(&pool).await;
    let d = make_account(&pool, company_id, "6000", AccountType::Expense).await;
    let c = make_account(&pool, company_id, "1020", AccountType::Asset).await;
    let contact = contact_id(&pool, company_id).await;
    let entry = make_entry(&pool, company_id, fy_id, d, c, (None, None)).await;

    sqlx::query(
        "INSERT INTO invoices (company_id, contact_id, date, journal_entry_id) VALUES (?, ?, ?, ?)",
    )
    .bind(company_id)
    .bind(contact)
    .bind(Utc::now().date_naive())
    .bind(entry)
    .execute(&pool)
    .await
    .unwrap();
    sqlx::query("UPDATE accounts SET active = FALSE WHERE id = ?")
        .bind(d)
        .execute(&pool)
        .await
        .unwrap();

    let detail: Value = app
        .client
        .get(app.url(&format!("/api/v1/journal-entries/{entry}")))
        .header("Authorization", auth(&token))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(
        detail["reversalBlockedBy"].as_str(),
        Some("OWNED_BY_INVOICE"),
        "les deux causes coexistent : c'est la propriété qui doit être annoncée"
    );
}

/// AC 13 — l'audit porte le lien vers la contre-passation.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn reverse_writes_an_audit_entry(pool: MySqlPool) {
    let (app, token, company_id, fy_id) = setup(&pool).await;
    let d = make_account(&pool, company_id, "6000", AccountType::Expense).await;
    let c = make_account(&pool, company_id, "1020", AccountType::Asset).await;
    let origin = make_entry(&pool, company_id, fy_id, d, c, (None, None)).await;

    let (_, created) = post_reverse(&app, &token, origin).await;
    let reversal_id = created["id"].as_i64().unwrap();

    // ⚠️ `details_json` est stocké en JSON (BLOB au décodage) : on le rend en
    // texte côté SQL plutôt que de deviner un type Rust.
    let (action, target, details): (String, i64, Option<String>) = sqlx::query_as(
        "SELECT action, entity_id, CAST(details_json AS CHAR) FROM audit_log \
         WHERE action = 'journal_entry.reversed' ORDER BY id DESC LIMIT 1",
    )
    .fetch_one(&pool)
    .await
    .expect("une entrée d'audit doit exister");
    assert_eq!(action, "journal_entry.reversed");
    assert_eq!(
        target, origin,
        "l'audit vise l'ORIGINE, pas la contre-passation"
    );
    assert!(
        details
            .unwrap_or_default()
            .contains(&reversal_id.to_string()),
        "l'audit porte `reversalJournalEntryId`"
    );
}

/// AC 7 — sans exercice ouvert couvrant **aujourd'hui**, le refus est un **400**.
///
/// ⚠️ Et non un 409 : le mappage de `FiscalYearInvalid` est partagé par tous les
/// flux du dépôt. Un test qui exigerait 409 pousserait à le changer pour eux tous.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn reverse_without_an_open_fiscal_year_is_refused(pool: MySqlPool) {
    let (app, token, company_id, fy_id) = setup(&pool).await;
    let d = make_account(&pool, company_id, "6000", AccountType::Expense).await;
    let c = make_account(&pool, company_id, "1020", AccountType::Asset).await;
    let origin = make_entry(&pool, company_id, fy_id, d, c, (None, None)).await;

    // L'exercice est clos APRÈS la création : l'origine reste, la cible manque.
    sqlx::query("UPDATE fiscal_years SET status = 'Closed' WHERE id = ?")
        .bind(fy_id)
        .execute(&pool)
        .await
        .unwrap();

    let (status, body) = post_reverse(&app, &token, origin).await;
    assert_eq!(status, 400, "corps : {body}");
    assert_eq!(body["error"]["code"].as_str(), Some("FISCAL_YEAR_INVALID"));
}

/// AC 10 — un compte devenu **non postable** ne bloque pas.
///
/// ⛔ Distinct de l'archivage : exiger la postabilité rendrait l'écriture
/// incorrigible à cause d'un changement de configuration POSTÉRIEUR.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn reverse_succeeds_when_an_account_became_non_postable(pool: MySqlPool) {
    let (app, token, company_id, fy_id) = setup(&pool).await;
    let d = make_account(&pool, company_id, "6000", AccountType::Expense).await;
    let c = make_account(&pool, company_id, "1020", AccountType::Asset).await;
    let origin = make_entry(&pool, company_id, fy_id, d, c, (None, None)).await;

    sqlx::query("UPDATE accounts SET postable = FALSE WHERE id = ?")
        .bind(d)
        .execute(&pool)
        .await
        .unwrap();

    let (status, body) = post_reverse(&app, &token, origin).await;
    assert_eq!(status, 201, "corps : {body}");
}

/// **Invariant I3** — aucune pièce ne référence une écriture contre-passée.
///
/// ⚠️ Cet invariant était annoncé « écrit » par le compte rendu de la story et
/// ne l'était pas. C'est le mode d'échec que le `CLAUDE.md` nomme : le compte
/// rendu devient le lieu du défaut. Il l'est désormais.
///
/// ⚠️ **Ce que la boucle ajoute VRAIMENT** : la garde est tenue par le `409`
/// asserté juste au-dessus ; la boucle ne peut donc rougir que dans un seul cas
/// — une contre-passation **committée malgré le refus**, c'est-à-dire une fuite
/// de rollback. C'est peu, et c'est exactement ce qu'aucun autre test ne
/// couvre. *(Portée précisée en passe 2 : une assertion dont on surestime la
/// portée est un test qu'on croit plus fort qu'il n'est.)*
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn no_document_ever_points_at_a_reversed_entry(pool: MySqlPool) {
    let (app, token, company_id, fy_id) = setup(&pool).await;
    let d = make_account(&pool, company_id, "6000", AccountType::Expense).await;
    let c = make_account(&pool, company_id, "1020", AccountType::Asset).await;
    let contact = contact_id(&pool, company_id).await;
    let today = Utc::now().date_naive();

    // Une écriture libre, contre-passée ; une écriture possédée par une facture,
    // dont la contre-passation est refusée. Après quoi l'invariant doit tenir.
    let free = make_entry(&pool, company_id, fy_id, d, c, (None, None)).await;
    let owned = make_entry(&pool, company_id, fy_id, d, c, (None, None)).await;
    sqlx::query(
        "INSERT INTO invoices (company_id, contact_id, date, journal_entry_id) VALUES (?, ?, ?, ?)",
    )
    .bind(company_id)
    .bind(contact)
    .bind(today)
    .bind(owned)
    .execute(&pool)
    .await
    .unwrap();

    assert_eq!(post_reverse(&app, &token, free).await.0, 201);
    assert_eq!(post_reverse(&app, &token, owned).await.0, 409);

    // ⛔ Le contrôle porte sur les CINQ tables qui possèdent une écriture.
    for (table, colonne) in [
        ("invoices", "journal_entry_id"),
        ("credit_notes", "journal_entry_id"),
        ("supplier_invoices", "purchase_journal_entry_id"),
        ("supplier_invoices", "settlement_journal_entry_id"),
        ("invoice_settlements", "journal_entry_id"),
        ("bank_transactions", "matched_entry_id"),
    ] {
        let n: i64 = sqlx::query_scalar(&format!(
            "SELECT COUNT(*) FROM {table} t \
             JOIN journal_entries r ON r.reverses_entry_id = t.{colonne} \
             WHERE t.company_id = ?"
        ))
        .bind(company_id)
        .fetch_one(&pool)
        .await
        .unwrap();
        assert_eq!(
            n, 0,
            "{table}.{colonne} pointe une écriture contre-passée — la pièce et les livres divergent"
        );
    }
}
