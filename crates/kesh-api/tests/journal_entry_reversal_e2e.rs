//! Tests E2E — Story 24-4a (#380) : la **contre-passation** d'une écriture.
//!
//! ⚠️ **Ce fichier est le PREMIER du dépôt à couvrir les écritures de bout en
//! bout.** `journal_entries.rs` n'avait que des tests unitaires — zéro
//! `#[sqlx::test]` — et seul `idor_multi_tenant_e2e.rs` touchait l'endpoint.
//! C'est aussi la couverture sur laquelle s'est appuyée la 24-4b (le gel).
//!
//! Couvre : le parcours nominal et ses invariants (I1 somme nulle compte par
//! compte, I2 les deux écritures au grand livre), les **neuf** refus — six
//! chemins de clé étrangère et non huit codes, `supplier_invoices` en portant
//! deux —, les statuts (409 pour un refus de propriété, 400 pour un compte
//! archivé ou l'absence d'exercice), le RBAC, l'IDOR, la reprise du projet
//! **par ligne**, et la suppression en masse que la clé étrangère
//! auto-référente aurait cassée de façon intermittente.
//!
//! Story 15-8a (#532) : le `PUT` rouvert — modification nominale et tracée,
//! clé d'API, refus de forme, contrôles de la saisie, période et exercices
//! clos (dont un exercice postérieur), pièces et paiement détaché, précédence,
//! et la table de correspondance entre motifs d'écran et refus d'écriture.
//!
//! Story 15-8b (#532) : le `DELETE` rouvert dans le même cadre — suppression
//! nominale tracée, clé d'API, pièces (dont le rapprochement, intact), paiement
//! détaché, exercices clos et postérieur, verrou de période, précédence, et la
//! colonne `DELETE` de la table de correspondance.

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
/// lève pas la garde `active`, qui est inconditionnelle.
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

/// Une pièce qui possède une écriture : l'écriture, le code attendu, le chemin
/// en clair, et l'identifiant de la pièce (`details.documentId`).
struct Piece {
    entry: i64,
    code: &'static str,
    quoi: &'static str,
    document_id: i64,
}

/// Monte une écriture possédée par **chacun des sept chemins** de clé
/// étrangère — facture, avoir, achat et règlement fournisseur, règlement
/// client, **solde** (`write_off`, Story 25-4-c3 — ajouté par la Story 15-8a,
/// finding R2-11), transaction bancaire rapprochée. Partagé par la
/// contre-passation (24-4a) et la modification (15-8a), qui refusent sous les
/// mêmes codes.
async fn monter_les_pieces(
    pool: &MySqlPool,
    company_id: i64,
    fy_id: i64,
    d: i64,
    c: i64,
) -> Vec<Piece> {
    let contact = contact_id(pool, company_id).await;
    let today = Utc::now().date_naive();

    // (1) facture client
    let e1 = make_entry(pool, company_id, fy_id, d, c, (None, None)).await;
    let invoice_id = sqlx::query(
        "INSERT INTO invoices (company_id, contact_id, date, journal_entry_id, invoice_number) \
         VALUES (?, ?, ?, ?, 'F-2026-014')",
    )
    .bind(company_id)
    .bind(contact)
    .bind(today)
    .bind(e1)
    .execute(pool)
    .await
    .unwrap()
    .last_insert_id() as i64;

    // (2) avoir
    let e2 = make_entry(pool, company_id, fy_id, d, c, (None, None)).await;
    let credit_note_id = sqlx::query(
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
    .execute(pool)
    .await
    .unwrap()
    .last_insert_id() as i64;

    // (3) facture fournisseur — écriture d'ACHAT
    let e3 = make_entry(pool, company_id, fy_id, d, c, (None, None)).await;
    let purchase_id = sqlx::query(
        "INSERT INTO supplier_invoices (company_id, contact_id, invoice_date, purchase_journal_entry_id) \
         VALUES (?, ?, ?, ?)",
    )
    .bind(company_id)
    .bind(contact)
    .bind(today)
    .bind(e3)
    .execute(pool)
    .await
    .unwrap()
    .last_insert_id() as i64;

    // (4) facture fournisseur — écriture de RÈGLEMENT (second chemin, même code)
    let e4a = make_entry(pool, company_id, fy_id, d, c, (None, None)).await;
    let e4b = make_entry(pool, company_id, fy_id, d, c, (None, None)).await;
    let settled_supplier_id = sqlx::query(
        "INSERT INTO supplier_invoices \
         (company_id, contact_id, invoice_date, purchase_journal_entry_id, settlement_journal_entry_id) \
         VALUES (?, ?, ?, ?, ?)",
    )
    .bind(company_id)
    .bind(contact)
    .bind(today)
    .bind(e4a)
    .bind(e4b)
    .execute(pool)
    .await
    .unwrap()
    .last_insert_id() as i64;

    // (5) règlement de facture client
    let e5 = make_entry(pool, company_id, fy_id, d, c, (None, None)).await;
    let settlement_id = sqlx::query(
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
    .execute(pool)
    .await
    .unwrap()
    .last_insert_id() as i64;

    // (5 bis) SOLDE d'une facture (`write_off`) — même table, même motif, que
    // l'ancien montage omettait.
    let e5b = make_entry(pool, company_id, fy_id, d, c, (None, None)).await;
    let write_off_id = sqlx::query(
        // ⚠️ `chk_invoice_settlements_write_off_nature` : un solde porte sa
        // nature et sa ventilation TVA (vide ici).
        "INSERT INTO invoice_settlements \
         (company_id, invoice_id, journal_entry_id, amount, settled_on, settlement_type, \
          settlement_account_id, write_off_nature, write_off_vat) \
         VALUES (?, ?, ?, 100.00, ?, 'write_off', ?, 'discount', '[]')",
    )
    .bind(company_id)
    .bind(invoice_id)
    .bind(e5b)
    .bind(today)
    .bind(c)
    .execute(pool)
    .await
    .unwrap()
    .last_insert_id() as i64;

    // (6) transaction bancaire rapprochée
    let e6 = make_entry(pool, company_id, fy_id, d, c, (None, None)).await;
    let bank_account = sqlx::query(
        "INSERT INTO bank_accounts (company_id, bank_name, iban) VALUES (?, 'Banque test', 'CH9300762011623852957')",
    )
    .bind(company_id)
    .execute(pool)
    .await
    .unwrap()
    .last_insert_id() as i64;
    let user_id: i64 = sqlx::query_scalar("SELECT id FROM users ORDER BY id LIMIT 1")
        .fetch_one(pool)
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
    .execute(pool)
    .await
    .unwrap()
    .last_insert_id() as i64;
    let bank_transaction_id = sqlx::query(
        "INSERT INTO bank_transactions \
         (company_id, import_id, bank_account_id, booking_date, amount, currency, details, matched_entry_id) \
         VALUES (?, ?, ?, ?, 100.00, 'CHF', 'test', ?)",
    )
    .bind(company_id)
    .bind(import_id)
    .bind(bank_account)
    .bind(today)
    .bind(e6)
    .execute(pool)
    .await
    .unwrap()
    .last_insert_id() as i64;

    vec![
        Piece {
            entry: e1,
            code: "OWNED_BY_INVOICE",
            quoi: "facture client",
            document_id: invoice_id,
        },
        Piece {
            entry: e2,
            code: "OWNED_BY_CREDIT_NOTE",
            quoi: "avoir",
            document_id: credit_note_id,
        },
        Piece {
            entry: e3,
            code: "OWNED_BY_SUPPLIER_INVOICE",
            quoi: "achat fournisseur",
            document_id: purchase_id,
        },
        Piece {
            entry: e4b,
            code: "OWNED_BY_SUPPLIER_INVOICE",
            quoi: "règlement fournisseur",
            document_id: settled_supplier_id,
        },
        Piece {
            entry: e5,
            code: "OWNED_BY_SETTLEMENT",
            quoi: "règlement client",
            document_id: settlement_id,
        },
        Piece {
            entry: e5b,
            code: "OWNED_BY_SETTLEMENT",
            quoi: "solde (write_off)",
            document_id: write_off_id,
        },
        Piece {
            entry: e6,
            code: "MATCHED_BANK_TRANSACTION",
            quoi: "rapprochement bancaire",
            document_id: bank_transaction_id,
        },
    ]
}

/// AC 6 (24-4a) — les **sept chemins** de clé étrangère, un cas par chemin.
///
/// ⛔ Ce sont les CHEMINS qui sont testés, pas les codes :
/// `purchase_journal_entry_id` et `settlement_journal_entry_id` partagent
/// `OWNED_BY_SUPPLIER_INVOICE` mais renvoient vers deux corrections
/// différentes, comme le règlement et le solde partagent `OWNED_BY_SETTLEMENT`
/// — un test par code laisserait le second jamais exercé.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn every_document_owned_entry_is_refused(pool: MySqlPool) {
    let (app, token, company_id, fy_id) = setup(&pool).await;
    let d = make_account(&pool, company_id, "6000", AccountType::Expense).await;
    let c = make_account(&pool, company_id, "1020", AccountType::Asset).await;
    let pieces = monter_les_pieces(&pool, company_id, fy_id, d, c).await;
    let e1 = pieces[0].entry;

    for Piece {
        entry, code, quoi, ..
    } in pieces
    {
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
    // ⛔ **Et le NUMÉRO du compte, sans quoi l'écran dit « réactivez-le » sans
    // dire lequel.** C'est la production la plus substantielle de la passe 2 de
    // revue, et elle n'était vérifiée par aucun test — un refactor du triplet de
    // `reversal_blocker` l'aurait cassée en silence. *(Passe 3 de revue.)*
    assert_eq!(
        detail["reversalBlockedLabel"].as_str(),
        Some("6000"),
        "le motif doit nommer le compte à réactiver"
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

// ---------------------------------------------------------------------------
// Story 15-8a (#532) — le `PUT` rouvert ; Story 15-8b — le `DELETE` aussi
// ---------------------------------------------------------------------------

/// `PUT` sur une écriture, avec un corps brut.
async fn put_entry(
    app: &TestApp,
    token: &str,
    id: i64,
    body: &str,
) -> (reqwest::StatusCode, Value) {
    let resp = app
        .client
        .put(app.url(&format!("/api/v1/journal-entries/{id}")))
        .header("Authorization", auth(token))
        .header("Content-Type", "application/json")
        .body(body.to_string())
        .send()
        .await
        .unwrap();
    let status = resp.status();
    (status, resp.json().await.unwrap_or(Value::Null))
}

async fn put_json(
    app: &TestApp,
    token: &str,
    id: i64,
    body: &Value,
) -> (reqwest::StatusCode, Value) {
    put_entry(app, token, id, &body.to_string()).await
}

async fn delete_entry(app: &TestApp, token: &str, id: i64) -> (reqwest::StatusCode, Value) {
    let resp = app
        .client
        .delete(app.url(&format!("/api/v1/journal-entries/{id}")))
        .header("Authorization", auth(token))
        .send()
        .await
        .unwrap();
    let status = resp.status();
    (status, resp.json().await.unwrap_or(Value::Null))
}

/// `GET /journal-entries/{id}` — le détail, motifs d'écran compris.
async fn detail(app: &TestApp, token: &str, id: i64) -> Value {
    let resp = app
        .client
        .get(app.url(&format!("/api/v1/journal-entries/{id}")))
        .header("Authorization", auth(token))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200, "détail de {id}");
    resp.json().await.unwrap()
}

/// Le corps d'un `PUT` **identique** à l'état présent, bâti depuis le détail —
/// ce qui exerce aussi l'aller-retour de la forme JSON.
async fn corps_identique(app: &TestApp, token: &str, id: i64) -> Value {
    let e = detail(app, token, id).await;
    json!({
        "entryDate": e["entryDate"],
        "journal": e["journal"],
        "description": e["description"],
        "version": e["version"],
        "lines": e["lines"].as_array().unwrap().iter().map(|l| json!({
            "accountId": l["accountId"],
            "debit": l["debit"],
            "credit": l["credit"],
            "projectId": l["projectId"],
        })).collect::<Vec<_>>(),
    })
}

/// Lit les lignes d'une écriture, triées, sous une forme comparable.
async fn lines_of(pool: &MySqlPool, id: i64) -> Vec<(i64, String, String)> {
    sqlx::query_as::<_, (i64, String, String)>(
        "SELECT account_id, CAST(debit AS CHAR), CAST(credit AS CHAR) \
         FROM journal_entry_lines WHERE entry_id = ? ORDER BY line_order",
    )
    .bind(id)
    .fetch_all(pool)
    .await
    .unwrap()
}

/// En-tête comparable : date, journal, libellé, version.
async fn header_of(pool: &MySqlPool, id: i64) -> (NaiveDate, String, String, i32) {
    sqlx::query_as(
        "SELECT entry_date, journal, description, version FROM journal_entries WHERE id = ?",
    )
    .bind(id)
    .fetch_one(pool)
    .await
    .unwrap()
}

/// Écriture manuelle à deux lignes, datée `date`, dans l'exercice `fy_id`.
async fn make_entry_on(
    pool: &MySqlPool,
    company_id: i64,
    fy_id: i64,
    d: i64,
    c: i64,
    date: NaiveDate,
) -> i64 {
    let mut tx = pool.begin().await.unwrap();
    let id = journal_entries::create_in_tx(
        &mut tx,
        fy_id,
        1,
        NewJournalEntry {
            company_id,
            entry_date: date,
            journal: Journal::OD,
            description: "Écriture à corriger".into(),
            project_id: None,
            lines: vec![
                NewJournalEntryLine {
                    account_id: d,
                    debit: dec!(100.00),
                    credit: dec!(0),
                    project_id: None,
                },
                NewJournalEntryLine {
                    account_id: c,
                    debit: dec!(0),
                    credit: dec!(100.00),
                    project_id: None,
                },
            ],
        },
        false,
    )
    .await
    .expect("écriture datée")
    .entry
    .id;
    tx.commit().await.unwrap();
    id
}

fn annee_courante() -> i32 {
    Utc::now()
        .date_naive()
        .format("%Y")
        .to_string()
        .parse()
        .unwrap()
}

/// Un exercice d'une autre année, au statut voulu (posé en SQL : `close`
/// refuserait un exercice futur, et seul l'état importe ici).
async fn exercice_de(pool: &MySqlPool, company_id: i64, annee: i32, status: &str) -> i64 {
    let id = fiscal_years::create(
        pool,
        1,
        NewFiscalYear {
            company_id,
            name: format!("Exercice {annee}"),
            start_date: NaiveDate::from_ymd_opt(annee, 1, 1).unwrap(),
            end_date: NaiveDate::from_ymd_opt(annee, 12, 31).unwrap(),
        },
    )
    .await
    .expect("exercice")
    .id;
    set_status(pool, id, status).await;
    id
}

async fn set_status(pool: &MySqlPool, fy_id: i64, status: &str) {
    sqlx::query("UPDATE fiscal_years SET status = ? WHERE id = ?")
        .bind(status)
        .bind(fy_id)
        .execute(pool)
        .await
        .unwrap();
}

async fn poser_borne(pool: &MySqlPool, company_id: i64, borne: Option<NaiveDate>) {
    sqlx::query("UPDATE companies SET books_locked_through = ? WHERE id = ?")
        .bind(borne)
        .bind(company_id)
        .execute(pool)
        .await
        .unwrap();
}

/// Pose la trace d'un **paiement détaché** (Story 25-3-c) : une facture
/// fournisseur `cancelled` et l'audit de son annulation, qui seul garde le
/// lien vers l'écriture de règlement. ⚠️ Posée par l'écrivain réel du journal
/// d'audit (`audit_log::insert_in_tx`), pas à la main ; le chemin complet
/// create → pay → cancel est tenu par `supplier_invoices_repository.rs`.
async fn detacher_un_paiement(
    pool: &MySqlPool,
    company_id: i64,
    fy_id: i64,
    (d, c): (i64, i64),
    entry: i64,
) -> i64 {
    let contact = contact_id(pool, company_id).await;
    let today = Utc::now().date_naive();
    // L'écriture d'achat, que l'annulation réelle contre-passe ; la colonne
    // est `NOT NULL`.
    let purchase = make_entry(pool, company_id, fy_id, d, c, (None, None)).await;
    let invoice_id = sqlx::query(
        "INSERT INTO supplier_invoices \
         (company_id, contact_id, invoice_date, status, supplier_invoice_number, \
          purchase_journal_entry_id) \
         VALUES (?, ?, ?, 'cancelled', 'FF-DET-1', ?)",
    )
    .bind(company_id)
    .bind(contact)
    .bind(today)
    .bind(purchase)
    .execute(pool)
    .await
    .unwrap()
    .last_insert_id() as i64;
    let mut tx = pool.begin().await.unwrap();
    kesh_db::repositories::audit_log::insert_in_tx(
        &mut tx,
        kesh_db::entities::audit_log::NewAuditLogEntry::user(
            1,
            "supplier_invoice.cancelled",
            "supplier_invoice",
            invoice_id,
            Some(json!({ "previousStatus": "paid", "settlementJournalEntryId": entry })),
        ),
    )
    .await
    .unwrap();
    tx.commit().await.unwrap();
    invoice_id
}

/// AC 1 · AC 3 · I1 — **le cas nominal** : date, journal, libellé et lignes
/// sont ceux du corps ; l'identité ne bouge pas ; `version` + 1 ; une trace
/// `journal_entry.updated` porte l'avant et l'après, lignes comprises.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn putting_an_entry_rewrites_it_and_traces_before_and_after(pool: MySqlPool) {
    let (app, token, company_id, fy_id) = setup(&pool).await;
    let d = make_account(&pool, company_id, "6000", AccountType::Expense).await;
    let c = make_account(&pool, company_id, "1020", AccountType::Asset).await;
    let id = make_entry(&pool, company_id, fy_id, d, c, (None, None)).await;
    let avant = detail(&app, &token, id).await;
    let nouvelle_date = NaiveDate::from_ymd_opt(annee_courante(), 1, 2).unwrap();

    // Le cas déclencheur, en petit : débit et crédit inversés.
    let corps = json!({
        "entryDate": nouvelle_date.to_string(),
        "journal": "Banque",
        "description": "  Écriture corrigée  ",
        "version": avant["version"],
        "lines": [
            { "accountId": d, "debit": "0", "credit": "250.00" },
            { "accountId": c, "debit": "250.00", "credit": "0", "projectId": null },
        ]
    });
    let (status, body) = put_json(&app, &token, id, &corps).await;
    assert_eq!(status, 200, "corps : {body}");
    assert_eq!(
        body["description"], "Écriture corrigée",
        "libellé taillé comme au POST"
    );
    assert_eq!(body["entryDate"], nouvelle_date.to_string());
    assert_eq!(body["journal"], "Banque");
    assert_eq!(
        body["version"].as_i64(),
        avant["version"].as_i64().map(|v| v + 1)
    );

    // I1 — l'identité ne bouge pas.
    for champ in [
        "id",
        "entryNumber",
        "fiscalYearId",
        "createdAt",
        "reversesEntryId",
    ] {
        assert_eq!(body[champ], avant[champ], "{champ} ne doit pas bouger");
    }
    assert_eq!(
        lines_of(&pool, id).await,
        vec![
            (d, "0.0000".to_string(), "250.0000".to_string()),
            (c, "250.0000".to_string(), "0.0000".to_string()),
        ]
    );

    // AC 3 — une trace, avant ET après, lignes comprises, par l'utilisateur.
    let traces: Vec<(Value, String, Option<i64>)> = sqlx::query_as(
        "SELECT details_json, actor_type, actor_api_key_id FROM audit_log \
         WHERE entity_type = 'journal_entry' AND entity_id = ? AND action = 'journal_entry.updated'",
    )
    .bind(id)
    .fetch_all(&pool)
    .await
    .unwrap();
    assert_eq!(traces.len(), 1, "une modification, une trace");
    let (details, actor_type, key) = &traces[0];
    assert_eq!(actor_type, "user");
    assert_eq!(*key, None);
    assert_eq!(details["before"]["description"], "Écriture à corriger");
    assert_eq!(details["after"]["description"], "Écriture corrigée");
    assert_eq!(details["before"]["lines"][0]["debit"], "100.0000");
    assert_eq!(details["after"]["lines"][0]["credit"], "250.0000");
    assert_eq!(details["after"]["lines"][1]["accountId"], c);
    assert_eq!(details["after"]["entryNumber"], avant["entryNumber"]);

    // AC 3 — un `PUT` identique à l'état présent : 200, ni version ni trace.
    let identique = corps_identique(&app, &token, id).await;
    let (status, body) = put_json(&app, &token, id, &identique).await;
    assert_eq!(status, 200, "corps : {body}");
    assert_eq!(body["version"], identique["version"]);
    let n: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM audit_log WHERE entity_id = ? AND action = 'journal_entry.updated'",
    )
    .bind(id)
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(n, 1, "un no-op ne laisse pas de trace");

    // I2 — sur DEUX modifications effectives : autant de traces que
    // `version − 1`, et l'« avant » de la seconde est l'« après » de la première.
    let mut seconde = corps_identique(&app, &token, id).await;
    seconde["description"] = json!("Écriture corrigée deux fois");
    let (status, body) = put_json(&app, &token, id, &seconde).await;
    assert_eq!(status, 200, "corps : {body}");
    let version = body["version"].as_i64().expect("version");
    let traces: Vec<(Value,)> = sqlx::query_as(
        "SELECT details_json FROM audit_log \
         WHERE entity_type = 'journal_entry' AND entity_id = ? AND action = 'journal_entry.updated' \
         ORDER BY id",
    )
    .bind(id)
    .fetch_all(&pool)
    .await
    .unwrap();
    assert_eq!(
        traces.len() as i64,
        version - 1,
        "I2 : nombre de traces = version − 1"
    );
    assert_eq!(traces.len(), 2);
    assert_eq!(traces[1].0["before"], traces[0].0["after"]);
    assert_eq!(
        traces[1].0["after"]["description"],
        "Écriture corrigée deux fois"
    );

    // I4 — la modification ne retire pas la contre-passation.
    let (status, _) = post_reverse(&app, &token, id).await;
    assert_eq!(status, 201);
}

/// AC 3 · D10 — une clé `read-write` modifie, et la trace porte **la clé**
/// (`actor_type = 'api_key'`, `actor_api_key_id`), pas seulement son créateur.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn a_read_write_key_modifies_and_is_traced_as_the_key(pool: MySqlPool) {
    let (app, token, company_id, fy_id) = setup(&pool).await;
    let d = make_account(&pool, company_id, "6000", AccountType::Expense).await;
    let c = make_account(&pool, company_id, "1020", AccountType::Asset).await;
    let id = make_entry(&pool, company_id, fy_id, d, c, (None, None)).await;

    let resp = app
        .client
        .post(app.url("/api/v1/settings/api-keys"))
        .bearer_auth(&token)
        .json(&json!({ "name": "integration-15-8a", "scope": "read-write" }))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 201, "création de clé");
    let cle = resp.json::<Value>().await.unwrap()["key"]
        .as_str()
        .unwrap()
        .to_string();
    let key_id: i64 =
        sqlx::query_scalar("SELECT id FROM api_keys WHERE name = 'integration-15-8a'")
            .fetch_one(&pool)
            .await
            .unwrap();

    let mut corps = corps_identique(&app, &token, id).await;
    corps["description"] = json!("Corrigée par l'intégration");
    let (status, body) = put_json(&app, &cle, id, &corps).await;
    assert_eq!(status, 200, "corps : {body}");

    let (actor_type, actor_key): (String, Option<i64>) = sqlx::query_as(
        "SELECT actor_type, actor_api_key_id FROM audit_log \
         WHERE entity_id = ? AND action = 'journal_entry.updated'",
    )
    .bind(id)
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(actor_type, "api_key");
    assert_eq!(actor_key, Some(key_id));
}

/// AC 9 · C-15-8-19 — le handler `PUT` **rejoue** réellement un interblocage
/// (revue de code P1, B-5). Le cycle projet ↔ exercice de
/// `update_and_a_reversal_of_the_same_year_can_deadlock` (kesh-db) est monté
/// ici à travers HTTP, en forçant le `PUT` à être la **victime** : B, qui tient
/// l'exercice, a d'abord écrit quelques centaines de lignes (InnoDB sacrifie la
/// transaction la plus légère — undo et verrous). Quand B obtient le projet en
/// exclusif, c'est que la transaction du `PUT` a été annulée par la 1213 : elle
/// tenait ce projet et attendait l'exercice que B tient encore. Le `PUT` doit
/// alors rendre 200, au second passage, une fois B parti.
///
/// ⛔ **Tue** « retirer l'enveloppe `retry_on_deadlock` du handler » : la 1213
/// remonte en 500.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn the_put_replays_a_deadlock_it_lost(pool: MySqlPool) {
    let (app, token, company_id, fy_id) = setup(&pool).await;
    let d = make_account(&pool, company_id, "6000", AccountType::Expense).await;
    let c = make_account(&pool, company_id, "1020", AccountType::Asset).await;
    let id = make_entry(&pool, company_id, fy_id, d, c, (None, None)).await;
    let p = sqlx::query(
        "INSERT INTO projects (company_id, code, name, archived, version) \
         VALUES (?, 'P-REJEU', 'P', FALSE, 0)",
    )
    .bind(company_id)
    .execute(&pool)
    .await
    .unwrap()
    .last_insert_id() as i64;
    let mut corps = corps_identique(&app, &token, id).await;
    corps["lines"][0]["projectId"] = json!(p);
    corps["description"] = json!("Projet posé après un interblocage");

    // B : lourde (des centaines de lignes d'undo, dans une table que le `PUT`
    // ne verrouille pas), puis l'exercice de l'écriture.
    let mut b = pool.begin().await.unwrap();
    let valeurs = vec!["(1, 'test.weight', 'test', 0)"; 500].join(", ");
    sqlx::query(&format!(
        "INSERT INTO audit_log (user_id, action, entity_type, entity_id) VALUES {valeurs}"
    ))
    .execute(&mut *b)
    .await
    .unwrap();
    sqlx::query("SELECT id FROM fiscal_years WHERE id = ? FOR UPDATE")
        .bind(fy_id)
        .execute(&mut *b)
        .await
        .unwrap();

    // A : le `PUT`, qui verrouille le projet NOUVEAU puis attend l'exercice.
    let a = {
        let app = TestApp {
            base_url: app.base_url.clone(),
            client: app.client.clone(),
        };
        let token = token.clone();
        let corps = corps.clone();
        tokio::spawn(async move { put_json(&app, &token, id, &corps).await })
    };
    assert!(
        kesh_db::test_fixtures::attendre_une_requete_en_cours(
            &pool,
            &[
                "SELECT status, start_date, end_date FROM fiscal_years",
                "FOR UPDATE"
            ],
            || a.is_finished()
        )
        .await,
        "le PUT doit attendre sur l'exercice, projet déjà tenu"
    );

    // B ferme le cycle. Obtenir le projet en EXCLUSIF prouve que la
    // transaction du `PUT`, qui le tenait, a été annulée par l'interblocage.
    sqlx::query("SELECT id FROM projects WHERE id = ? FOR UPDATE")
        .bind(p)
        .execute(&mut *b)
        .await
        .expect("B doit gagner : le PUT, plus léger, est la victime de la 1213");
    b.rollback().await.unwrap();

    let (status, body) = a.await.unwrap();
    assert_eq!(status, 200, "le PUT rejoué doit aboutir ; corps : {body}");
    assert_eq!(body["description"], "Projet posé après un interblocage");
    assert_eq!(
        body["version"].as_i64(),
        corps["version"].as_i64().map(|v| v + 1)
    );
    let n: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM audit_log WHERE entity_id = ? AND action = 'journal_entry.updated'",
    )
    .bind(id)
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(n, 1, "un seul passage a écrit : la victime n'a rien laissé");
}

/// AC 10 · AC 7 — les refus de **forme** précèdent toute lecture de la base :
/// un corps illisible rend 400/422, et un corps déséquilibré sur un `id`
/// inexistant rend `ENTRY_UNBALANCED`, pas 404 — il ne révèle rien.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn bad_forms_are_refused_before_the_base_is_read(pool: MySqlPool) {
    let (app, token, company_id, fy_id) = setup(&pool).await;
    let d = make_account(&pool, company_id, "6000", AccountType::Expense).await;
    let c = make_account(&pool, company_id, "1020", AccountType::Asset).await;
    let id = make_entry(&pool, company_id, fy_id, d, c, (None, None)).await;
    let avant = (header_of(&pool, id).await, lines_of(&pool, id).await);

    for corps in ["", "{}", "ceci n'est pas du JSON", "[1,2,3]"] {
        let (status, _) = put_entry(&app, &token, id, corps).await;
        assert!(
            status == 400 || status == 422,
            "corps {corps:?} : 400 ou 422 attendu, reçu {status}"
        );
    }

    let mut desequilibre = corps_identique(&app, &token, id).await;
    desequilibre["lines"][0]["debit"] = json!("999.00");
    for cible in [id, 999_999] {
        let (status, body) = put_json(&app, &token, cible, &desequilibre).await;
        assert_eq!(status, 400, "cible {cible} : {body}");
        assert_eq!(body["error"]["code"], "ENTRY_UNBALANCED", "cible {cible}");
    }
    assert_eq!(
        (header_of(&pool, id).await, lines_of(&pool, id).await),
        avant,
        "un refus ne touche à rien"
    );
}

/// AC 4 — les contrôles de la saisie, sans exemption (C-15-8-4, 15-5a), et la
/// nouvelle date dans l'exercice **de l'écriture** (C-15-8-3) ; rien ne bouge.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn the_put_applies_the_controls_of_the_entry_form(pool: MySqlPool) {
    let (app, token, company_id, fy_id) = setup(&pool).await;
    let x = make_account(&pool, company_id, "6100", AccountType::Expense).await;
    let y = make_account(&pool, company_id, "1030", AccountType::Asset).await;
    let id = make_entry(&pool, company_id, fy_id, x, y, (None, None)).await;
    let avant = (header_of(&pool, id).await, lines_of(&pool, id).await);
    let mut corps = corps_identique(&app, &token, id).await;
    corps["description"] = json!("Libellé seul modifié");

    // (a) Compte NON IMPUTABLE sur une ligne inchangée → nommé.
    sqlx::query("UPDATE accounts SET postable = FALSE WHERE id = ?")
        .bind(x)
        .execute(&pool)
        .await
        .unwrap();
    let (status, body) = put_json(&app, &token, id, &corps).await;
    assert_eq!(status, 400, "{body}");
    assert_eq!(body["error"]["code"], "ACCOUNT_NOT_POSTABLE");
    assert_eq!(
        body["error"]["details"]["rejected"][0]["accountNumber"],
        "6100"
    );

    // (b) Plus un compte ARCHIVÉ sur l'autre ligne → l'archivé prime (15-5a).
    sqlx::query("UPDATE accounts SET active = FALSE WHERE id = ?")
        .bind(y)
        .execute(&pool)
        .await
        .unwrap();
    let (status, body) = put_json(&app, &token, id, &corps).await;
    assert_eq!(status, 400, "{body}");
    assert_eq!(body["error"]["code"], "INACTIVE_OR_INVALID_ACCOUNTS");

    // (c) Le seul compte archivé, sur une ligne inchangée.
    sqlx::query("UPDATE accounts SET postable = TRUE WHERE id = ?")
        .bind(x)
        .execute(&pool)
        .await
        .unwrap();
    let (status, body) = put_json(&app, &token, id, &corps).await;
    assert_eq!(status, 400, "{body}");
    assert_eq!(body["error"]["code"], "INACTIVE_OR_INVALID_ACCOUNTS");
    sqlx::query("UPDATE accounts SET active = TRUE WHERE id = ?")
        .bind(y)
        .execute(&pool)
        .await
        .unwrap();

    // (d) Nouvelle date hors de l'exercice DE L'ÉCRITURE, même couverte par un
    // autre exercice ouvert.
    let suivant = annee_courante() + 1;
    exercice_de(&pool, company_id, suivant, "Open").await;
    let mut ailleurs = corps.clone();
    ailleurs["entryDate"] = json!(NaiveDate::from_ymd_opt(suivant, 2, 1).unwrap().to_string());
    let (status, body) = put_json(&app, &token, id, &ailleurs).await;
    assert_eq!(status, 400, "{body}");
    assert_eq!(body["error"]["code"], "DATE_OUTSIDE_FISCAL_YEAR");

    // (e) Déséquilibre.
    let mut desequilibre = corps.clone();
    desequilibre["lines"][0]["debit"] = json!("101.00");
    let (status, body) = put_json(&app, &token, id, &desequilibre).await;
    assert_eq!(status, 400, "{body}");
    assert_eq!(body["error"]["code"], "ENTRY_UNBALANCED");

    assert_eq!(
        (header_of(&pool, id).await, lines_of(&pool, id).await),
        avant,
        "aucun refus ne touche à l'écriture"
    );
    // Et la même requête, comptes rétablis, passe.
    let (status, body) = put_json(&app, &token, id, &corps).await;
    assert_eq!(status, 200, "{body}");
}

/// AC 5 — le verrou de période garde l'**ancienne** ET la **nouvelle** date,
/// seuil inclusif ; l'exercice clos seul rend `FISCAL_YEAR_CLOSED`.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn period_lock_and_closed_year_refuse_the_put(pool: MySqlPool) {
    let (app, token, company_id, fy_id) = setup(&pool).await;
    let d = make_account(&pool, company_id, "6000", AccountType::Expense).await;
    let c = make_account(&pool, company_id, "1020", AccountType::Asset).await;
    let annee = annee_courante();
    let date = NaiveDate::from_ymd_opt(annee, 6, 15).unwrap();
    let id = make_entry_on(&pool, company_id, fy_id, d, c, date).await;
    let mut corps = corps_identique(&app, &token, id).await;
    corps["description"] = json!("Libellé modifié");

    // Ancienne date = borne (inclusif) → refus.
    poser_borne(&pool, company_id, Some(date)).await;
    let (status, body) = put_json(&app, &token, id, &corps).await;
    assert_eq!(status, 400, "{body}");
    assert_eq!(body["error"]["code"], "PERIOD_LOCKED");

    // Ancienne date libre, NOUVELLE date = borne → refus.
    let borne = NaiveDate::from_ymd_opt(annee, 3, 31).unwrap();
    poser_borne(&pool, company_id, Some(borne)).await;
    let mut recule = corps.clone();
    recule["entryDate"] = json!(borne.to_string());
    let (status, body) = put_json(&app, &token, id, &recule).await;
    assert_eq!(status, 400, "{body}");
    assert_eq!(body["error"]["code"], "PERIOD_LOCKED");

    // Le lendemain de la borne passe.
    let mut lendemain = corps.clone();
    lendemain["entryDate"] = json!(NaiveDate::from_ymd_opt(annee, 4, 1).unwrap().to_string());
    let (status, body) = put_json(&app, &token, id, &lendemain).await;
    assert_eq!(status, 200, "{body}");
    poser_borne(&pool, company_id, None).await;

    // Exercice clos, seule cause.
    set_status(&pool, fy_id, "Closed").await;
    let corps = corps_identique(&app, &token, id).await;
    let (status, body) = put_json(&app, &token, id, &corps).await;
    assert_eq!(status, 400, "{body}");
    assert_eq!(body["error"]["code"], "FISCAL_YEAR_CLOSED");
}

/// AC 5 · C-15-8-22 — **un exercice postérieur clos fige l'écriture** : le
/// bilan est cumulatif. Le plus proche postérieur clos est nommé ; rouvert, le
/// même `PUT` passe ; un exercice ANTÉRIEUR clos ne gêne pas.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn a_closed_later_year_freezes_the_entry_until_reopened(pool: MySqlPool) {
    let (app, token, company_id, fy_id) = setup(&pool).await;
    let d = make_account(&pool, company_id, "6000", AccountType::Expense).await;
    let c = make_account(&pool, company_id, "1020", AccountType::Asset).await;
    let annee = annee_courante();
    let id = make_entry(&pool, company_id, fy_id, d, c, (None, None)).await;
    exercice_de(&pool, company_id, annee - 1, "Closed").await; // antérieur clos
    let n1 = exercice_de(&pool, company_id, annee + 1, "Closed").await;
    let n2 = exercice_de(&pool, company_id, annee + 2, "Closed").await;
    let avant = (header_of(&pool, id).await, lines_of(&pool, id).await);
    let mut corps = corps_identique(&app, &token, id).await;
    corps["description"] = json!("Libellé modifié");

    let (status, body) = put_json(&app, &token, id, &corps).await;
    assert_eq!(status, 400, "{body}");
    assert_eq!(body["error"]["code"], "LATER_FISCAL_YEAR_CLOSED");
    assert_eq!(body["error"]["details"]["fiscalYearId"].as_i64(), Some(n1));
    assert_eq!(
        body["error"]["details"]["fiscalYearName"],
        format!("Exercice {}", annee + 1),
        "le PLUS PROCHE postérieur clos est nommé"
    );
    assert!(
        body["error"]["message"]
            .as_str()
            .unwrap_or_default()
            .contains(&format!("Exercice {}", annee + 1)),
        "{body}"
    );
    let ecran = detail(&app, &token, id).await;
    assert_eq!(ecran["modifiable"], false);
    assert_eq!(ecran["modificationBlockedBy"], "LATER_FISCAL_YEAR_CLOSED");
    assert_eq!(
        ecran["modificationBlockedLabel"],
        format!("Exercice {}", annee + 1)
    );
    assert_eq!(
        (header_of(&pool, id).await, lines_of(&pool, id).await),
        avant
    );

    set_status(&pool, n2, "Open").await;
    set_status(&pool, n1, "Open").await;
    let (status, body) = put_json(&app, &token, id, &corps).await;
    assert_eq!(status, 200, "rouverts, le même PUT passe : {body}");
}

/// AC 6 · I3 — **chaque pièce gèle son écriture** : le `PUT` rend 409 sous le
/// code du motif, `details.documentId` = la pièce, et l'écriture ne bouge pas.
/// Une contre-passée rend `ENTRY_IS_REVERSED`, une contre-passation
/// `IS_A_REVERSAL`.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn every_document_owned_entry_refuses_the_put_and_stays_unchanged(pool: MySqlPool) {
    let (app, token, company_id, fy_id) = setup(&pool).await;
    let d = make_account(&pool, company_id, "6000", AccountType::Expense).await;
    let c = make_account(&pool, company_id, "1020", AccountType::Asset).await;
    let pieces = monter_les_pieces(&pool, company_id, fy_id, d, c).await;

    for piece in &pieces {
        let avant = (
            header_of(&pool, piece.entry).await,
            lines_of(&pool, piece.entry).await,
        );
        let mut corps = corps_identique(&app, &token, piece.entry).await;
        corps["description"] = json!("Tentative");
        let (status, body) = put_json(&app, &token, piece.entry, &corps).await;
        assert_eq!(status, 409, "{} — {body}", piece.quoi);
        assert_eq!(body["error"]["code"], piece.code, "{}", piece.quoi);
        assert_eq!(
            body["error"]["details"]["documentId"].as_i64(),
            Some(piece.document_id),
            "{}",
            piece.quoi
        );
        assert_eq!(
            (
                header_of(&pool, piece.entry).await,
                lines_of(&pool, piece.entry).await
            ),
            avant,
            "{} : I3, rien ne bouge",
            piece.quoi
        );
    }
    // Le message nomme la pièce, comme pour la contre-passation.
    let mut corps = corps_identique(&app, &token, pieces[0].entry).await;
    corps["description"] = json!("Tentative");
    let (_, body) = put_json(&app, &token, pieces[0].entry, &corps).await;
    assert!(
        body["error"]["message"]
            .as_str()
            .unwrap_or_default()
            .contains("F-2026-014"),
        "{body}"
    );

    // Contre-passée, et contre-passation.
    let origin = make_entry(&pool, company_id, fy_id, d, c, (None, None)).await;
    let (_, created) = post_reverse(&app, &token, origin).await;
    let reversal = created["id"].as_i64().unwrap();
    let corps = corps_identique(&app, &token, origin).await;
    let (status, body) = put_json(&app, &token, origin, &corps).await;
    assert_eq!(
        (status.as_u16(), body["error"]["code"].as_str()),
        (409, Some("ENTRY_IS_REVERSED"))
    );
    let corps = corps_identique(&app, &token, reversal).await;
    let (status, body) = put_json(&app, &token, reversal, &corps).await;
    assert_eq!(
        (status.as_u16(), body["error"]["code"].as_str()),
        (409, Some("IS_A_REVERSAL"))
    );
}

/// AC 6 · C-15-8-20 — **le paiement détaché** d'une facture fournisseur
/// annulée ne se modifie pas (sortie de banque réelle), mais reste
/// contre-passable — comme le manuel le promet.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn a_detached_supplier_payment_refuses_the_put_but_stays_reversable(pool: MySqlPool) {
    let (app, token, company_id, fy_id) = setup(&pool).await;
    let d = make_account(&pool, company_id, "2000", AccountType::Liability).await;
    let c = make_account(&pool, company_id, "1020", AccountType::Asset).await;
    let paiement = make_entry(&pool, company_id, fy_id, d, c, (None, None)).await;
    let facture = detacher_un_paiement(&pool, company_id, fy_id, (d, c), paiement).await;

    let mut corps = corps_identique(&app, &token, paiement).await;
    corps["description"] = json!("Tentative");
    let (status, body) = put_json(&app, &token, paiement, &corps).await;
    assert_eq!(status, 409, "{body}");
    assert_eq!(body["error"]["code"], "DETACHED_SUPPLIER_SETTLEMENT");
    assert_eq!(
        body["error"]["details"]["documentId"].as_i64(),
        Some(facture)
    );
    assert_eq!(body["error"]["details"]["documentNumber"], "FF-DET-1");
    assert!(
        body["error"]["message"]
            .as_str()
            .unwrap_or_default()
            .contains("FF-DET-1"),
        "{body}"
    );

    let ecran = detail(&app, &token, paiement).await;
    assert_eq!(
        ecran["modificationBlockedBy"],
        "DETACHED_SUPPLIER_SETTLEMENT"
    );
    assert_eq!(
        ecran["reversable"], true,
        "la contre-passation reste offerte"
    );
    let (status, _) = post_reverse(&app, &token, paiement).await;
    assert_eq!(status, 201);
}

/// AC 7 · AC 8 — **la précédence**, chaque paire montée avec ses DEUX causes.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn the_precedence_of_the_put_refusals_is_fixed(pool: MySqlPool) {
    let (app, token, company_id, fy_id) = setup(&pool).await;
    let d = make_account(&pool, company_id, "6000", AccountType::Expense).await;
    let c = make_account(&pool, company_id, "1020", AccountType::Asset).await;
    let pieces = monter_les_pieces(&pool, company_id, fy_id, d, c).await;
    let doc = pieces[0].entry; // facture client
    let reversed = make_entry(&pool, company_id, fy_id, d, c, (None, None)).await;
    let (s, _) = post_reverse(&app, &token, reversed).await;
    assert_eq!(s, 201);
    let plain = make_entry(&pool, company_id, fy_id, d, c, (None, None)).await;

    async fn perime(app: &TestApp, token: &str, id: i64) -> Value {
        let mut corps = corps_identique(app, token, id).await;
        corps["version"] = json!(corps["version"].as_i64().unwrap() + 7);
        corps["description"] = json!("Tentative");
        corps
    }
    async fn code(app: &TestApp, token: &str, id: i64, corps: &Value) -> (u16, String) {
        let (status, body) = put_json(app, token, id, corps).await;
        (
            status.as_u16(),
            body["error"]["code"]
                .as_str()
                .unwrap_or_default()
                .to_string(),
        )
    }

    // AC 8 — version périmée seule.
    let avant = (header_of(&pool, plain).await, lines_of(&pool, plain).await);
    let corps = perime(&app, &token, plain).await;
    assert_eq!(
        code(&app, &token, plain, &corps).await,
        (409, "OPTIMISTIC_LOCK_CONFLICT".into())
    );
    assert_eq!(
        (header_of(&pool, plain).await, lines_of(&pool, plain).await),
        avant
    );

    // Contre-passée + version périmée → le motif réel prime.
    let corps = perime(&app, &token, reversed).await;
    assert_eq!(
        code(&app, &token, reversed, &corps).await,
        (409, "ENTRY_IS_REVERSED".into())
    );
    // Pièce + version périmée → code de la pièce.
    let corps = perime(&app, &token, doc).await;
    assert_eq!(
        code(&app, &token, doc, &corps).await,
        (409, "OWNED_BY_INVOICE".into())
    );
    // Pièce + période verrouillée → code de la pièce.
    poser_borne(&pool, company_id, Some(Utc::now().date_naive())).await;
    let corps = corps_identique(&app, &token, doc).await;
    assert_eq!(
        code(&app, &token, doc, &corps).await,
        (409, "OWNED_BY_INVOICE".into())
    );
    poser_borne(&pool, company_id, None).await;

    // Exercice postérieur clos + contre-passée, + pièce → LATER.
    let later = exercice_de(&pool, company_id, annee_courante() + 1, "Closed").await;
    for id in [reversed, doc] {
        let corps = corps_identique(&app, &token, id).await;
        assert_eq!(
            code(&app, &token, id, &corps).await,
            (400, "LATER_FISCAL_YEAR_CLOSED".into())
        );
    }
    // Exercice clos + exercice postérieur clos → FISCAL_YEAR_CLOSED.
    set_status(&pool, fy_id, "Closed").await;
    let corps = corps_identique(&app, &token, plain).await;
    assert_eq!(
        code(&app, &token, plain, &corps).await,
        (400, "FISCAL_YEAR_CLOSED".into())
    );
    // Exercice clos + contre-passée (postérieur rouvert) → FISCAL_YEAR_CLOSED.
    set_status(&pool, later, "Open").await;
    let corps = corps_identique(&app, &token, reversed).await;
    assert_eq!(
        code(&app, &token, reversed, &corps).await,
        (400, "FISCAL_YEAR_CLOSED".into())
    );
}

/// AC 12 — **la table de correspondance**, écrite en dur : pour chacun des
/// onze codes d'écran, un montage qui ne porte QUE cette cause ; le détail rend
/// le code, et le `PUT` identique rend le refus associé — et (Story 15-8b, AC 8)
/// le `DELETE` **le même**. `ALREADY_REVERSED` ↔
/// `ENTRY_IS_REVERSED` est le seul écart de nom, voulu. Un compte archivé ou
/// non imputable ne rend PAS l'écriture non modifiable.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn each_screen_code_maps_to_its_put_and_delete_refusal(pool: MySqlPool) {
    let (app, token, company_id, fy_id) = setup(&pool).await;
    let d = make_account(&pool, company_id, "6000", AccountType::Expense).await;
    let c = make_account(&pool, company_id, "1020", AccountType::Asset).await;
    let annee = annee_courante();

    // N-2 ouvert suivi de N-1 clos : LATER pour N-2, FISCAL_YEAR_CLOSED pour N-1.
    let n2 = exercice_de(&pool, company_id, annee - 2, "Open").await;
    let n1 = exercice_de(&pool, company_id, annee - 1, "Open").await;
    let e_later = make_entry_on(
        &pool,
        company_id,
        n2,
        d,
        c,
        NaiveDate::from_ymd_opt(annee - 2, 5, 1).unwrap(),
    )
    .await;
    let e_closed = make_entry_on(
        &pool,
        company_id,
        n1,
        d,
        c,
        NaiveDate::from_ymd_opt(annee - 1, 5, 1).unwrap(),
    )
    .await;
    set_status(&pool, n1, "Closed").await;

    let pieces = monter_les_pieces(&pool, company_id, fy_id, d, c).await;
    let origin = make_entry(&pool, company_id, fy_id, d, c, (None, None)).await;
    let (_, created) = post_reverse(&app, &token, origin).await;
    let reversal = created["id"].as_i64().unwrap();
    let paiement = make_entry(&pool, company_id, fy_id, d, c, (None, None)).await;
    detacher_un_paiement(&pool, company_id, fy_id, (d, c), paiement).await;
    let verrouillee = make_entry_on(
        &pool,
        company_id,
        fy_id,
        d,
        c,
        NaiveDate::from_ymd_opt(annee, 1, 1).unwrap(),
    )
    .await;

    let mut cas: Vec<(i64, &str, u16, &str)> = vec![
        (e_closed, "FISCAL_YEAR_CLOSED", 400, "FISCAL_YEAR_CLOSED"),
        (
            e_later,
            "LATER_FISCAL_YEAR_CLOSED",
            400,
            "LATER_FISCAL_YEAR_CLOSED",
        ),
        (reversal, "IS_A_REVERSAL", 409, "IS_A_REVERSAL"),
        (origin, "ALREADY_REVERSED", 409, "ENTRY_IS_REVERSED"),
        (
            paiement,
            "DETACHED_SUPPLIER_SETTLEMENT",
            409,
            "DETACHED_SUPPLIER_SETTLEMENT",
        ),
    ];
    for p in &pieces {
        cas.push((p.entry, p.code, 409, p.code));
    }
    let mut vus: std::collections::BTreeSet<&str> = std::collections::BTreeSet::new();
    for (id, ecran, statut, put_code) in &cas {
        let e = detail(&app, &token, *id).await;
        assert_eq!(e["modifiable"], false, "{ecran}");
        assert_eq!(e["modificationBlockedBy"], *ecran);
        let corps = corps_identique(&app, &token, *id).await;
        let (status, body) = put_json(&app, &token, *id, &corps).await;
        assert_eq!(
            (status.as_u16(), body["error"]["code"].as_str()),
            (*statut, Some(*put_code)),
            "{ecran}"
        );
        let (status, body) = delete_entry(&app, &token, *id).await;
        assert_eq!(
            (status.as_u16(), body["error"]["code"].as_str()),
            (*statut, Some(*put_code)),
            "DELETE — {ecran}"
        );
        vus.insert(ecran);
    }
    let e = detail(&app, &token, pieces[0].entry).await;
    assert_eq!(e["modificationBlockedLabel"], "F-2026-014");

    // PERIOD_LOCKED, en dernier : la borne gèlerait les autres montages.
    let borne = NaiveDate::from_ymd_opt(annee, 1, 1).unwrap();
    poser_borne(&pool, company_id, Some(borne)).await;
    let e = detail(&app, &token, verrouillee).await;
    assert_eq!(e["modificationBlockedBy"], "PERIOD_LOCKED");
    assert_eq!(e["modificationBlockedLabel"], borne.to_string());
    let corps = corps_identique(&app, &token, verrouillee).await;
    let (status, body) = put_json(&app, &token, verrouillee, &corps).await;
    assert_eq!(
        (status.as_u16(), body["error"]["code"].as_str()),
        (400, Some("PERIOD_LOCKED"))
    );
    let (status, body) = delete_entry(&app, &token, verrouillee).await;
    assert_eq!(
        (status.as_u16(), body["error"]["code"].as_str()),
        (400, Some("PERIOD_LOCKED")),
        "DELETE — PERIOD_LOCKED"
    );
    vus.insert("PERIOD_LOCKED");
    poser_borne(&pool, company_id, None).await;
    assert_eq!(
        vus.len(),
        11,
        "les onze codes d'écran sont exercés : {vus:?}"
    );

    // `null` : modifiable ; le PUT identique rend 200.
    let libre = make_entry(&pool, company_id, fy_id, d, c, (None, None)).await;
    let e = detail(&app, &token, libre).await;
    assert_eq!(
        (
            e["modifiable"].as_bool(),
            e["modificationBlockedBy"].is_null()
        ),
        (Some(true), true)
    );
    let corps = corps_identique(&app, &token, libre).await;
    assert_eq!(put_json(&app, &token, libre, &corps).await.0, 200);
    // `modifiable = true` → le DELETE rend 204.
    assert_eq!(delete_entry(&app, &token, libre).await.0, 204);

    // Compte archivé / non imputable : modifiable reste VRAI, le PUT rend le 400.
    for (colonne, attendu) in [
        ("active", "INACTIVE_OR_INVALID_ACCOUNTS"),
        ("postable", "ACCOUNT_NOT_POSTABLE"),
    ] {
        let x = make_account(
            &pool,
            company_id,
            if colonne == "active" { "6200" } else { "6300" },
            AccountType::Expense,
        )
        .await;
        let id = make_entry(&pool, company_id, fy_id, x, c, (None, None)).await;
        sqlx::query(&format!(
            "UPDATE accounts SET {colonne} = FALSE WHERE id = ?"
        ))
        .bind(x)
        .execute(&pool)
        .await
        .unwrap();
        let e = detail(&app, &token, id).await;
        assert_eq!(e["modifiable"], true, "{colonne}");
        let corps = corps_identique(&app, &token, id).await;
        let (status, body) = put_json(&app, &token, id, &corps).await;
        assert_eq!(
            (status.as_u16(), body["error"]["code"].as_str()),
            (400, Some(attendu)),
            "{colonne}"
        );
    }
}

/// Story 15-8b (#532), AC 1 — **supprimer une écriture manuelle** : 204,
/// l'écriture et ses lignes disparaissent, une trace `journal_entry.deleted`
/// porte l'instantané complet (lignes comprises) et l'acteur ; la création
/// suivante ne reprend **pas** le numéro (compteur de la 25-2-c).
///
/// ⛔ Avant la 15-8b, ce test s'appelait `deleting_a_posted_entry_is_refused`
/// et prouvait le gel (un 409 inconditionnel) ; il est **inversé**, non supprimé.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn deleting_a_manual_entry_removes_it_traces_it_and_never_reuses_its_number(pool: MySqlPool) {
    let (app, token, company_id, fy_id) = setup(&pool).await;
    let d = make_account(&pool, company_id, "6000", AccountType::Expense).await;
    let c = make_account(&pool, company_id, "1020", AccountType::Asset).await;
    let id = make_entry(&pool, company_id, fy_id, d, c, (None, None)).await;
    let numero: i64 = sqlx::query_scalar("SELECT entry_number FROM journal_entries WHERE id = ?")
        .bind(id)
        .fetch_one(&pool)
        .await
        .unwrap();

    let (status, body) = delete_entry(&app, &token, id).await;
    assert_eq!(status, 204, "{body}");

    let (entetes, lignes): (i64, i64) = sqlx::query_as(
        "SELECT (SELECT COUNT(*) FROM journal_entries WHERE id = ?), \
                (SELECT COUNT(*) FROM journal_entry_lines WHERE entry_id = ?)",
    )
    .bind(id)
    .bind(id)
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(
        (entetes, lignes),
        (0, 0),
        "l'écriture et ses lignes ont disparu"
    );

    let (actor_type, details): (String, Option<Value>) = sqlx::query_as(
        "SELECT actor_type, details_json FROM audit_log \
         WHERE entity_type = 'journal_entry' AND entity_id = ? AND action = 'journal_entry.deleted'",
    )
    .bind(id)
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(actor_type, "user");
    let details = details.expect("instantané");
    assert_eq!(details["entryNumber"].as_i64(), Some(numero));
    assert_eq!(
        details["lines"].as_array().map(Vec::len),
        Some(2),
        "l'instantané porte les lignes : {details}"
    );

    let suivante = make_entry(&pool, company_id, fy_id, d, c, (None, None)).await;
    let suivant: i64 = sqlx::query_scalar("SELECT entry_number FROM journal_entries WHERE id = ?")
        .bind(suivante)
        .fetch_one(&pool)
        .await
        .unwrap();
    assert!(
        suivant > numero,
        "le numéro {numero} ne doit jamais être réattribué ; reçu {suivant}"
    );
}

/// Story 15-8b, AC 1 · C-15-8-8 — une clé `read-write` supprime, et la trace
/// porte **la clé** (`actor_type = 'api_key'`, `actor_api_key_id`).
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn a_read_write_key_deletes_and_is_traced_as_the_key(pool: MySqlPool) {
    let (app, token, company_id, fy_id) = setup(&pool).await;
    let d = make_account(&pool, company_id, "6000", AccountType::Expense).await;
    let c = make_account(&pool, company_id, "1020", AccountType::Asset).await;
    let id = make_entry(&pool, company_id, fy_id, d, c, (None, None)).await;

    let resp = app
        .client
        .post(app.url("/api/v1/settings/api-keys"))
        .bearer_auth(&token)
        .json(&json!({ "name": "integration-15-8b", "scope": "read-write" }))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 201, "création de clé");
    let cle = resp.json::<Value>().await.unwrap()["key"]
        .as_str()
        .unwrap()
        .to_string();
    let key_id: i64 =
        sqlx::query_scalar("SELECT id FROM api_keys WHERE name = 'integration-15-8b'")
            .fetch_one(&pool)
            .await
            .unwrap();

    let (status, body) = delete_entry(&app, &cle, id).await;
    assert_eq!(status, 204, "{body}");

    let (actor_type, actor_key): (String, Option<i64>) = sqlx::query_as(
        "SELECT actor_type, actor_api_key_id FROM audit_log \
         WHERE entity_id = ? AND action = 'journal_entry.deleted'",
    )
    .bind(id)
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(actor_type, "api_key");
    assert_eq!(actor_key, Some(key_id));
}

/// Story 15-8b, AC 2 · AC 4 — **chaque pièce retient son écriture** : le
/// `DELETE` rend 409 sous le code du motif, `details.documentId` = la pièce, et
/// l'écriture reste, lignes et en-tête identiques. ⛔ **L'écriture rapprochée** :
/// la clé `matched_entry_id` est en `ON DELETE SET NULL` — sans la garde, la
/// suppression réussirait et laisserait la transaction bancaire `reconciled`
/// sans lien ; la ligne `bank_transactions` est donc assertée intacte. Une
/// contre-passée rend `ENTRY_IS_REVERSED`, une contre-passation `IS_A_REVERSAL`
/// (la clé `RESTRICT` ne protège que l'origine), le paiement détaché
/// `DETACHED_SUPPLIER_SETTLEMENT` (aucune colonne ne le référence).
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn every_document_owned_entry_refuses_the_delete_and_stays_intact(pool: MySqlPool) {
    let (app, token, company_id, fy_id) = setup(&pool).await;
    let d = make_account(&pool, company_id, "6000", AccountType::Expense).await;
    let c = make_account(&pool, company_id, "1020", AccountType::Asset).await;
    let pieces = monter_les_pieces(&pool, company_id, fy_id, d, c).await;

    async fn transaction_bancaire(pool: &MySqlPool, id: i64) -> (Option<i64>, String) {
        sqlx::query_as("SELECT matched_entry_id, status FROM bank_transactions WHERE id = ?")
            .bind(id)
            .fetch_one(pool)
            .await
            .unwrap()
    }

    for piece in &pieces {
        let avant = (
            header_of(&pool, piece.entry).await,
            lines_of(&pool, piece.entry).await,
        );
        let banque_avant = if piece.code == "MATCHED_BANK_TRANSACTION" {
            Some(transaction_bancaire(&pool, piece.document_id).await)
        } else {
            None
        };
        let (status, body) = delete_entry(&app, &token, piece.entry).await;
        assert_eq!(status, 409, "{} — {body}", piece.quoi);
        assert_eq!(body["error"]["code"], piece.code, "{}", piece.quoi);
        assert_eq!(
            body["error"]["details"]["documentId"].as_i64(),
            Some(piece.document_id),
            "{}",
            piece.quoi
        );
        assert_eq!(
            (
                header_of(&pool, piece.entry).await,
                lines_of(&pool, piece.entry).await
            ),
            avant,
            "{} : rien ne bouge",
            piece.quoi
        );
        if let Some(banque_avant) = banque_avant {
            let banque_apres = transaction_bancaire(&pool, piece.document_id).await;
            assert_eq!(
                banque_apres, banque_avant,
                "la transaction bancaire est intacte"
            );
            assert_eq!(banque_apres.0, Some(piece.entry));
        }
    }

    // Contre-passée, et contre-passation.
    let origin = make_entry(&pool, company_id, fy_id, d, c, (None, None)).await;
    let (_, created) = post_reverse(&app, &token, origin).await;
    let reversal = created["id"].as_i64().unwrap();
    let (status, body) = delete_entry(&app, &token, origin).await;
    assert_eq!(
        (status.as_u16(), body["error"]["code"].as_str()),
        (409, Some("ENTRY_IS_REVERSED"))
    );
    let avant = (
        header_of(&pool, reversal).await,
        lines_of(&pool, reversal).await,
    );
    let (status, body) = delete_entry(&app, &token, reversal).await;
    assert_eq!(
        (status.as_u16(), body["error"]["code"].as_str()),
        (409, Some("IS_A_REVERSAL"))
    );
    assert_eq!(
        (
            header_of(&pool, reversal).await,
            lines_of(&pool, reversal).await
        ),
        avant
    );

    // Le paiement détaché d'une facture fournisseur annulée.
    let paiement = make_entry(&pool, company_id, fy_id, d, c, (None, None)).await;
    let facture = detacher_un_paiement(&pool, company_id, fy_id, (d, c), paiement).await;
    let (status, body) = delete_entry(&app, &token, paiement).await;
    assert_eq!(status, 409, "{body}");
    assert_eq!(body["error"]["code"], "DETACHED_SUPPLIER_SETTLEMENT");
    assert_eq!(
        body["error"]["details"]["documentId"].as_i64(),
        Some(facture)
    );
    assert!(
        body["error"]["message"]
            .as_str()
            .unwrap_or_default()
            .contains("FF-DET-1"),
        "{body}"
    );
}

/// Story 15-8b, AC 4 — le verrou de période (seuil inclusif), l'exercice clos
/// seul, et **l'exercice postérieur clos seul** (C-15-8-22) : 400 nommant N+1,
/// puis 204 une fois N+1 rouvert. Après chaque refus, l'écriture est intacte.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn period_lock_closed_year_and_later_year_refuse_the_delete(pool: MySqlPool) {
    let (app, token, company_id, fy_id) = setup(&pool).await;
    let d = make_account(&pool, company_id, "6000", AccountType::Expense).await;
    let c = make_account(&pool, company_id, "1020", AccountType::Asset).await;
    let annee = annee_courante();
    let date = NaiveDate::from_ymd_opt(annee, 6, 15).unwrap();
    let id = make_entry_on(&pool, company_id, fy_id, d, c, date).await;
    let avant = (header_of(&pool, id).await, lines_of(&pool, id).await);

    // Date = borne (inclusif) → refus.
    poser_borne(&pool, company_id, Some(date)).await;
    let (status, body) = delete_entry(&app, &token, id).await;
    poser_borne(&pool, company_id, None).await;
    assert_eq!(status, 400, "{body}");
    assert_eq!(body["error"]["code"], "PERIOD_LOCKED");
    assert_eq!(
        (header_of(&pool, id).await, lines_of(&pool, id).await),
        avant
    );

    // Exercice postérieur clos, seule cause → nommé ; rouvert → 204.
    let n1 = exercice_de(&pool, company_id, annee + 1, "Closed").await;
    let (status, body) = delete_entry(&app, &token, id).await;
    assert_eq!(status, 400, "{body}");
    assert_eq!(body["error"]["code"], "LATER_FISCAL_YEAR_CLOSED");
    assert_eq!(body["error"]["details"]["fiscalYearId"].as_i64(), Some(n1));
    assert_eq!(
        body["error"]["details"]["fiscalYearName"],
        format!("Exercice {}", annee + 1)
    );
    assert_eq!(
        (header_of(&pool, id).await, lines_of(&pool, id).await),
        avant
    );

    // Exercice clos, seule cause (le postérieur rouvert).
    set_status(&pool, n1, "Open").await;
    set_status(&pool, fy_id, "Closed").await;
    let (status, body) = delete_entry(&app, &token, id).await;
    assert_eq!(status, 400, "{body}");
    assert_eq!(body["error"]["code"], "FISCAL_YEAR_CLOSED");
    assert_eq!(
        (header_of(&pool, id).await, lines_of(&pool, id).await),
        avant
    );

    // Tout rouvert, borne posée la VEILLE de la date (l'écriture est le
    // lendemain de la borne) : la suppression passe — c'est l'autre côté du
    // seuil inclusif testé plus haut, fixé ici sur le chemin de la route
    // (revue P1, A3) et non seulement en `mod tests` de `kesh-db`.
    set_status(&pool, fy_id, "Open").await;
    poser_borne(&pool, company_id, Some(date.pred_opt().unwrap())).await;
    let (status, body) = delete_entry(&app, &token, id).await;
    poser_borne(&pool, company_id, None).await;
    assert_eq!(
        status, 204,
        "rouverts, borne la veille, le même DELETE passe : {body}"
    );
}

/// Story 15-8b, AC 4-bis — **la précédence du `DELETE`**, chaque paire montée
/// avec ses DEUX causes (le pendant de `the_precedence_of_the_put_refusals_is_fixed`).
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn the_precedence_of_the_delete_refusals_is_fixed(pool: MySqlPool) {
    let (app, token, company_id, fy_id) = setup(&pool).await;
    let d = make_account(&pool, company_id, "6000", AccountType::Expense).await;
    let c = make_account(&pool, company_id, "1020", AccountType::Asset).await;
    let pieces = monter_les_pieces(&pool, company_id, fy_id, d, c).await;
    let doc = pieces[0].entry; // facture client
    let reversed = make_entry(&pool, company_id, fy_id, d, c, (None, None)).await;
    let (s, created) = post_reverse(&app, &token, reversed).await;
    assert_eq!(s, 201);
    let reversal = created["id"].as_i64().unwrap();
    let plain = make_entry(&pool, company_id, fy_id, d, c, (None, None)).await;

    async fn code(app: &TestApp, token: &str, id: i64) -> (u16, String) {
        let (status, body) = delete_entry(app, token, id).await;
        (
            status.as_u16(),
            body["error"]["code"]
                .as_str()
                .unwrap_or_default()
                .to_string(),
        )
    }

    // Pièce + période verrouillée → code de la pièce ; contre-passation +
    // période verrouillée → IS_A_REVERSAL.
    poser_borne(&pool, company_id, Some(Utc::now().date_naive())).await;
    let piece_sous_borne = code(&app, &token, doc).await;
    let contre_passation_sous_borne = code(&app, &token, reversal).await;
    poser_borne(&pool, company_id, None).await;
    assert_eq!(piece_sous_borne, (409, "OWNED_BY_INVOICE".into()));
    assert_eq!(contre_passation_sous_borne, (409, "IS_A_REVERSAL".into()));

    // Exercice postérieur clos + contre-passée, + pièce → LATER.
    let later = exercice_de(&pool, company_id, annee_courante() + 1, "Closed").await;
    for id in [reversed, doc] {
        assert_eq!(
            code(&app, &token, id).await,
            (400, "LATER_FISCAL_YEAR_CLOSED".into())
        );
    }
    // Exercice clos + exercice postérieur clos → FISCAL_YEAR_CLOSED.
    set_status(&pool, fy_id, "Closed").await;
    assert_eq!(
        code(&app, &token, plain).await,
        (400, "FISCAL_YEAR_CLOSED".into())
    );
    // Exercice clos + pièce, exercice clos + contre-passée (postérieur rouvert)
    // → FISCAL_YEAR_CLOSED.
    set_status(&pool, later, "Open").await;
    for id in [doc, reversed] {
        assert_eq!(
            code(&app, &token, id).await,
            (400, "FISCAL_YEAR_CLOSED".into())
        );
    }
}

/// AC 2, 3 — un `id` inconnu **ou d'une autre société** rend 404, jamais 409.
///
/// ⛔ Le 409 révélerait l'existence de la ressource : c'est la convention IDOR
/// du dépôt, et elle prime sur le refus d'état.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn put_and_delete_never_leak_the_existence_of_a_foreign_entry(pool: MySqlPool) {
    let (app, token, company_id, fy_id) = setup(&pool).await;
    let d = make_account(&pool, company_id, "6000", AccountType::Expense).await;
    let c = make_account(&pool, company_id, "1020", AccountType::Asset).await;
    let _mien = make_entry(&pool, company_id, fy_id, d, c, (None, None)).await;

    // Une écriture d'une AUTRE société, bien réelle.
    // ⚠️ Même gabarit que `another_company_entry_is_not_found` : la table
    // `companies` a des colonnes obligatoires sans défaut, on recopie donc la
    // société existante plutôt que d'énumérer ses champs à la main.
    let autre_company: i64 = sqlx::query(
        "INSERT INTO companies (name, address, org_type, accounting_language, instance_language) \
         SELECT CONCAT(name, ' ter'), address, org_type, accounting_language, instance_language \
         FROM companies WHERE id = ?",
    )
    .bind(company_id)
    .execute(&pool)
    .await
    .unwrap()
    .last_insert_id() as i64;
    let today = Utc::now().date_naive();
    let autre_fy = fiscal_years::create(
        &pool,
        1,
        NewFiscalYear {
            company_id: autre_company,
            name: "Exercice autre".into(),
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
    .expect("exercice de l'autre société");
    let ad = make_account(&pool, autre_company, "6000", AccountType::Expense).await;
    let ac = make_account(&pool, autre_company, "1020", AccountType::Asset).await;
    let etranger = make_entry(&pool, autre_company, autre_fy.id, ad, ac, (None, None)).await;

    // ⚠️ Corps VALIDE (Story 15-8a) : le 404 doit venir de l'appartenance, pas
    // d'un refus de forme.
    let corps = corps_identique(&app, &token, _mien).await;
    for id in [999_999_i64, etranger] {
        let (status, _) = put_json(&app, &token, id, &corps).await;
        assert_eq!(status, 404, "PUT sur l'id {id} doit rendre 404");
        let (status, _) = delete_entry(&app, &token, id).await;
        assert_eq!(status, 404, "DELETE sur l'id {id} doit rendre 404");
    }
}

/// AC 5 (24-4b) · Story 15-8b, AC 4 — une écriture **contre-passée** répond
/// `ENTRY_IS_REVERSED` au `PUT` **et** au `DELETE`, le code de la 24-4a.
///
/// ⛔ Avant la 15-8b, ce test s'appelait `a_reversed_entry_answers_reversed_not_posted`
/// et distinguait ce code de celui du gel ; le gel retiré, il ne restait rien à
/// distinguer — il est **réécrit** pour tenir le code sur les deux verbes.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn a_reversed_entry_answers_reversed_on_put_and_delete(pool: MySqlPool) {
    let (app, token, company_id, fy_id) = setup(&pool).await;
    let d = make_account(&pool, company_id, "6000", AccountType::Expense).await;
    let c = make_account(&pool, company_id, "1020", AccountType::Asset).await;
    let origin = make_entry(&pool, company_id, fy_id, d, c, (None, None)).await;
    let (created, _) = post_reverse(&app, &token, origin).await;
    assert_eq!(created, 201);

    let (status, body) = delete_entry(&app, &token, origin).await;
    assert_eq!(status, 409);
    assert_eq!(
        body["error"]["code"].as_str(),
        Some("ENTRY_IS_REVERSED"),
        "DELETE sur une écriture contre-passée : {body}"
    );
    let corps = corps_identique(&app, &token, origin).await;
    let (status, body) = put_json(&app, &token, origin, &corps).await;
    assert_eq!(status, 409);
    assert_eq!(
        body["error"]["code"].as_str(),
        Some("ENTRY_IS_REVERSED"),
        "PUT sur une écriture contre-passée : {body}"
    );
}

/// AC 6 (24-4b) · Story 15-8b, AC 4-bis — l'exercice clos précède **tout**
/// conflit : écriture clos ET contre-passée, et écriture clos ET de pièce,
/// `PUT` et `DELETE` → 400 `FISCAL_YEAR_CLOSED`.
///
/// ⚠️ Le cas est atteignable : la 24-4a autorise de contre-passer une écriture
/// d'un exercice clos, la contre-passation tombant dans l'exercice courant.
/// ⛔ Avant la 15-8b, ce test s'appelait `a_closed_fiscal_year_answers_before_both_conflicts`
/// (« les deux 409 ») ; l'un des deux n'existe plus.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn a_closed_fiscal_year_answers_before_any_conflict(pool: MySqlPool) {
    let (app, token, company_id, fy_id) = setup(&pool).await;
    let d = make_account(&pool, company_id, "6000", AccountType::Expense).await;
    let c = make_account(&pool, company_id, "1020", AccountType::Asset).await;
    let pieces = monter_les_pieces(&pool, company_id, fy_id, d, c).await;
    let doc = pieces[0].entry;
    let origin = make_entry(&pool, company_id, fy_id, d, c, (None, None)).await;
    let (created, _) = post_reverse(&app, &token, origin).await;
    assert_eq!(created, 201);

    sqlx::query("UPDATE fiscal_years SET status = 'Closed' WHERE id = ?")
        .bind(fy_id)
        .execute(&pool)
        .await
        .unwrap();

    for (id, quoi) in [(origin, "contre-passée"), (doc, "de pièce")] {
        let (status, body) = delete_entry(&app, &token, id).await;
        assert_eq!(status, 400, "DELETE, exercice clos et {quoi} : {body}");
        assert_eq!(body["error"]["code"], "FISCAL_YEAR_CLOSED");
        let corps = corps_identique(&app, &token, id).await;
        let (status, body) = put_json(&app, &token, id, &corps).await;
        assert_eq!(status, 400, "PUT, exercice clos et {quoi} : {body}");
        assert_eq!(body["error"]["code"], "FISCAL_YEAR_CLOSED");
    }
}

/// AC 7 — Consultation reçoit 403 sur les deux verbes, **avant** tout refus
/// d'état : le rôle se juge avant l'écriture.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn consultation_can_neither_rewrite_nor_delete(pool: MySqlPool) {
    let (app, _token, company_id, fy_id) = setup(&pool).await;
    let d = make_account(&pool, company_id, "6000", AccountType::Expense).await;
    let c = make_account(&pool, company_id, "1020", AccountType::Asset).await;
    let id = make_entry(&pool, company_id, fy_id, d, c, (None, None)).await;

    let lecteur = consultation_token(&app, &pool).await;
    let (status, _) = put_entry(&app, &lecteur, id, "{}").await;
    assert_eq!(status, 403);
    let (status, _) = delete_entry(&app, &lecteur, id).await;
    assert_eq!(status, 403);
}

/// AC 2 (15-8a) · I3 — **l'écriture d'ouverture se modifie** : débit et
/// crédit inversés, c'est le cas déclencheur de #532. La contre-passation reste
/// offerte ; et — Story 15-8b, AC 7 — elle **se supprime** (le statut des soldes
/// de départ et le numéro 2 sont tenus par `opening_balances_e2e.rs`).
///
/// ⛔ Avant la 15-8a, ce test s'appelait `the_opening_entry_is_frozen_but_still_correctable`
/// et prouvait le gel ; inversé pour le `PUT` par la 15-8a, puis pour le
/// `DELETE` par la 15-8b — jamais supprimé.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn the_opening_entry_is_modifiable_reversable_and_deletable(pool: MySqlPool) {
    let (app, token, company_id, fy_id) = setup(&pool).await;
    let d = make_account(&pool, company_id, "1020", AccountType::Asset).await;
    let c = make_account(&pool, company_id, "2800", AccountType::Liability).await;

    let ouverture = journal_entries::create_opening_entry(
        &pool,
        company_id,
        fy_id,
        1,
        NewJournalEntry {
            company_id,
            entry_date: Utc::now().date_naive(),
            journal: Journal::OD,
            description: "Soldes de départ".into(),
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
    .expect("écriture d'ouverture")
    .entry
    .id;

    // Débit et crédit inversés.
    let mut corps = corps_identique(&app, &token, ouverture).await;
    corps["lines"][0]["debit"] = json!("0");
    corps["lines"][0]["credit"] = json!("5000.00");
    corps["lines"][1]["debit"] = json!("5000.00");
    corps["lines"][1]["credit"] = json!("0");
    let (status, body) = put_json(&app, &token, ouverture, &corps).await;
    assert_eq!(status, 200, "{body}");
    assert_eq!(
        lines_of(&pool, ouverture).await,
        vec![
            (d, "0.0000".to_string(), "5000.0000".to_string()),
            (c, "5000.0000".to_string(), "0.0000".to_string()),
        ]
    );

    // I4 — la porte de la contre-passation reste ouverte.
    let ecran = detail(&app, &token, ouverture).await;
    assert_eq!(ecran["reversable"], true, "{ecran}");

    // Story 15-8b, AC 7 — et la suppression aboutit.
    let (status, body) = delete_entry(&app, &token, ouverture).await;
    assert_eq!(status, 204, "{body}");
    let reste: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM journal_entries WHERE id = ?")
        .bind(ouverture)
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(reste, 0);
}

/// I3 — une écriture d'un exercice **clos** reste contre-passable ; le `PUT` y
/// est refusé (Story 15-8a).
///
/// ⚠️ Second cas où l'enfermement serait le plus coûteux : le `PUT` et le
/// `DELETE` y étaient déjà refusés avant cette story, mais c'est la
/// contre-passation qui doit continuer d'aboutir — dans l'exercice courant.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn an_entry_of_a_closed_year_stays_correctable(pool: MySqlPool) {
    let (app, token, company_id, fy_id) = setup(&pool).await;
    let d = make_account(&pool, company_id, "6000", AccountType::Expense).await;
    let c = make_account(&pool, company_id, "1020", AccountType::Asset).await;

    let today = Utc::now().date_naive();
    let annee: i32 = today.format("%Y").to_string().parse().unwrap();
    let passe = fiscal_years::create(
        &pool,
        1,
        NewFiscalYear {
            company_id,
            name: format!("Exercice {}", annee - 1),
            start_date: NaiveDate::from_ymd_opt(annee - 1, 1, 1).unwrap(),
            end_date: NaiveDate::from_ymd_opt(annee - 1, 12, 31).unwrap(),
        },
    )
    .await
    .expect("exercice antérieur");

    let mut tx = pool.begin().await.unwrap();
    let ancienne = journal_entries::create_in_tx(
        &mut tx,
        passe.id,
        1,
        NewJournalEntry {
            company_id,
            entry_date: NaiveDate::from_ymd_opt(annee - 1, 6, 15).unwrap(),
            journal: Journal::OD,
            description: "Écriture de l'exercice précédent".into(),
            project_id: None,
            lines: vec![
                NewJournalEntryLine {
                    account_id: d,
                    debit: dec!(100.00),
                    credit: dec!(0),
                    project_id: None,
                },
                NewJournalEntryLine {
                    account_id: c,
                    debit: dec!(0),
                    credit: dec!(100.00),
                    project_id: None,
                },
            ],
        },
        false,
    )
    .await
    .expect("écriture antérieure")
    .entry
    .id;
    tx.commit().await.unwrap();

    sqlx::query("UPDATE fiscal_years SET status = 'Closed' WHERE id = ?")
        .bind(passe.id)
        .execute(&pool)
        .await
        .unwrap();

    // Story 15-8a — le `PUT` y est refusé (exercice clos)…
    let corps = corps_identique(&app, &token, ancienne).await;
    let (status, body) = put_json(&app, &token, ancienne, &corps).await;
    assert_eq!(status, 400, "{body}");
    assert_eq!(body["error"]["code"], "FISCAL_YEAR_CLOSED");
    // … Story 15-8b — le `DELETE` aussi …
    let (status, body) = delete_entry(&app, &token, ancienne).await;
    assert_eq!(status, 400, "{body}");
    assert_eq!(body["error"]["code"], "FISCAL_YEAR_CLOSED");

    // … et la contre-passation y reste la correction.
    let (status, _) = post_reverse(&app, &token, ancienne).await;
    assert_eq!(status, 201);

    // La contre-passation tombe dans l'exercice OUVERT, pas dans le clos.
    let fy_contre: i64 = sqlx::query_scalar(
        "SELECT fiscal_year_id FROM journal_entries WHERE reverses_entry_id = ?",
    )
    .bind(ancienne)
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(fy_contre, fy_id);
}
