//! Story 34-2 (#533) — le solde d'ouverture saisi avec le compte bancaire, par
//! HTTP : `POST /api/v1/bank-accounts` (champ `openingBalance`) et `POST
//! /api/v1/bank-accounts/{id}/opening-balance`.
//!
//! Chaque test nomme l'AC qu'il tient et la mutation qu'il tue (M2 à M9 de la
//! fiche). Fixtures synthétiques seulement.

mod common;

use std::net::SocketAddr;
use std::sync::Arc;

use chrono::{Datelike, NaiveDate, TimeDelta, Utc};
use common::create_test_company;
use kesh_api::auth::bootstrap::ensure_admin_user;
use kesh_api::config::Config;
use kesh_api::{AppState, build_router};
use kesh_db::entities::account::AccountType;
use kesh_db::entities::journal_entry::Journal;
use kesh_db::entities::{
    AccountRole, NewAccount, NewFiscalYear, NewJournalEntry, NewJournalEntryLine,
};
use kesh_db::repositories::{accounts, fiscal_years, journal_entries};
use kesh_db::test_fixtures::attendre_une_requete_en_cours;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use serde_json::{Value, json};
use sqlx::MySqlPool;

const TEST_JWT_SECRET: &[u8] = b"test-secret-32-bytes-minimum-test-secret-padding";
const TEST_ADMIN_PASSWORD: &str = "e2e-test-admin-password";
const IBAN_A: &str = "CH9300762011623852957";
const IBAN_B: &str = "CH5604835012345678009";

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

async fn login(app: &TestApp) -> String {
    let resp = app
        .client
        .post(app.url("/api/v1/auth/login"))
        .json(&json!({ "username": "admin", "password": TEST_ADMIN_PASSWORD }))
        .send()
        .await
        .unwrap();
    let body: Value = resp.json().await.unwrap();
    body["accessToken"].as_str().unwrap().to_string()
}

/// La société de test, ses comptes et son exercice.
struct Ctx {
    app: TestApp,
    token: String,
    company_id: i64,
    user_id: i64,
    fy_id: i64,
    fy_start: NaiveDate,
    /// 1020 Banque (actif), jamais mouvementé.
    bank: i64,
    /// 2100 Découvert (passif), jamais mouvementé.
    overdraft: i64,
    /// 2970 report à nouveau, rôle `RetainedEarnings`.
    retained: i64,
    /// 1000 Caisse et 2000 Dettes : de quoi passer une écriture ordinaire.
    cash: i64,
    debts: i64,
}

async fn create_acc(
    pool: &MySqlPool,
    user_id: i64,
    company_id: i64,
    number: &str,
    account_type: AccountType,
    role: Option<AccountRole>,
) -> i64 {
    accounts::create(
        pool,
        user_id,
        NewAccount {
            company_id,
            number: number.into(),
            name: format!("Compte {number}"),
            account_type,
            parent_id: None,
            role,
            postable: true,
        },
    )
    .await
    .unwrap()
    .id
}

/// Montage : société, administrateur connecté, onboarding terminé, exercice de
/// l'année courante (ouvert, le jour du jour compris) et les comptes de [`Ctx`].
async fn setup(pool: &MySqlPool) -> Ctx {
    let app = spawn_app(pool.clone()).await;
    create_test_company(pool).await;
    ensure_admin_user(pool, &test_config()).await.unwrap();
    let token = login(&app).await;
    sqlx::query(
        "INSERT INTO onboarding_state (singleton, step_completed, is_demo, ui_mode) \
         VALUES (TRUE, 7, FALSE, 'guided') \
         ON DUPLICATE KEY UPDATE step_completed = 7, version = version + 1",
    )
    .execute(pool)
    .await
    .unwrap();
    let (user_id, company_id): (i64, i64) =
        sqlx::query_as("SELECT id, company_id FROM users WHERE username = 'admin'")
            .fetch_one(pool)
            .await
            .unwrap();
    let year = Utc::now().date_naive().year();
    let fy_start = NaiveDate::from_ymd_opt(year, 1, 1).unwrap();
    let fy_id = fiscal_years::create(
        pool,
        user_id,
        NewFiscalYear {
            company_id,
            name: format!("Exercice {year}"),
            start_date: fy_start,
            end_date: NaiveDate::from_ymd_opt(year, 12, 31).unwrap(),
        },
    )
    .await
    .unwrap()
    .id;
    let bank = create_acc(pool, user_id, company_id, "1020", AccountType::Asset, None).await;
    let overdraft = create_acc(
        pool,
        user_id,
        company_id,
        "2100",
        AccountType::Liability,
        None,
    )
    .await;
    let retained = create_acc(
        pool,
        user_id,
        company_id,
        "2970",
        AccountType::Liability,
        Some(AccountRole::RetainedEarnings),
    )
    .await;
    let cash = create_acc(pool, user_id, company_id, "1000", AccountType::Asset, None).await;
    let debts = create_acc(
        pool,
        user_id,
        company_id,
        "2000",
        AccountType::Liability,
        None,
    )
    .await;
    Ctx {
        app,
        token,
        company_id,
        user_id,
        fy_id,
        fy_start,
        bank,
        overdraft,
        retained,
        cash,
        debts,
    }
}

impl Ctx {
    async fn post(&self, path: &str, body: Value) -> (u16, Value) {
        let resp = self
            .app
            .client
            .post(self.app.url(path))
            .header("Authorization", format!("Bearer {}", self.token))
            .json(&body)
            .send()
            .await
            .unwrap();
        let status = resp.status().as_u16();
        let text = resp.text().await.unwrap();
        (status, serde_json::from_str(&text).unwrap_or(Value::Null))
    }

    async fn create(&self, body: Value) -> (u16, Value) {
        self.post("/api/v1/bank-accounts", body).await
    }

    /// Une écriture ordinaire sur `account` (contrepartie 2000) : le compte est
    /// alors mouvementé.
    async fn move_account(&self, pool: &MySqlPool, account: i64) -> i64 {
        journal_entries::create(
            pool,
            self.fy_id,
            self.user_id,
            NewJournalEntry {
                company_id: self.company_id,
                entry_date: self.fy_start,
                journal: Journal::OD,
                description: "Écriture ordinaire".into(),
                project_id: None,
                lines: vec![
                    NewJournalEntryLine {
                        account_id: account,
                        debit: dec!(10),
                        credit: Decimal::ZERO,
                        project_id: None,
                    },
                    NewJournalEntryLine {
                        account_id: self.debts,
                        debit: Decimal::ZERO,
                        credit: dec!(10),
                        project_id: None,
                    },
                ],
            },
        )
        .await
        .unwrap()
        .entry
        .id
    }
}

/// Un compte bancaire posé directement (montage), lié ou non.
async fn insert_bank_account(
    pool: &MySqlPool,
    company_id: i64,
    iban: &str,
    is_primary: bool,
    journal_account_id: Option<i64>,
) -> i64 {
    sqlx::query(
        "INSERT INTO bank_accounts (company_id, bank_name, iban, is_primary, journal_account_id) \
         VALUES (?, 'Banque Test', ?, ?, ?)",
    )
    .bind(company_id)
    .bind(iban)
    .bind(is_primary)
    .bind(journal_account_id)
    .execute(pool)
    .await
    .unwrap()
    .last_insert_id() as i64
}

async fn count(pool: &MySqlPool, sql: &str) -> i64 {
    sqlx::query_scalar(sql).fetch_one(pool).await.unwrap()
}

async fn bank_account_count(pool: &MySqlPool) -> i64 {
    count(pool, "SELECT COUNT(*) FROM bank_accounts").await
}

async fn entry_count(pool: &MySqlPool) -> i64 {
    count(pool, "SELECT COUNT(*) FROM journal_entries").await
}

/// Lignes `(compte, débit, crédit)` d'une écriture, triées.
async fn lines_of(pool: &MySqlPool, entry_id: i64) -> Vec<(i64, Decimal, Decimal)> {
    let rows: Vec<(i64, Decimal, Decimal)> = sqlx::query_as(
        "SELECT account_id, debit, credit FROM journal_entry_lines WHERE entry_id = ? ORDER BY account_id",
    )
    .bind(entry_id)
    .fetch_all(pool)
    .await
    .unwrap();
    rows.into_iter()
        .map(|(a, d, c)| (a, d.normalize(), c.normalize()))
        .collect()
}

fn sorted(mut v: Vec<(i64, Decimal, Decimal)>) -> Vec<(i64, Decimal, Decimal)> {
    v.sort();
    v
}

fn base_body(journal_account_id: Option<i64>) -> Value {
    json!({
        "bankName": "UBS",
        "description": "Personnel",
        "iban": IBAN_A,
        "journalAccountId": journal_account_id,
    })
}

fn with_opening(mut body: Value, amount: &str, side: &str) -> Value {
    body["openingBalance"] = json!({ "amount": amount, "side": side });
    body
}

// ===========================================================================
// Création avec solde (AC-2, AC-3, AC-4)
// ===========================================================================

/// AC-2, AC-3, AC-4 — société sans écriture : le compte et son écriture
/// d'ouverture sont créés ensemble ; l'écriture est datée du premier jour du
/// premier exercice, journal OD, libellé « Solde d'ouverture — UBS —
/// Personnel », 1020 au débit et le report au crédit ; `bank_account.created`
/// nomme l'écriture ; la génération générale est ensuite refusée. Tue M3
/// (*Avoir* envoyé en crédit) et M4 (contrepartie sur le compte lié).
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn creation_avec_solde_passe_l_ecriture_d_ouverture(pool: MySqlPool) {
    let ctx = setup(&pool).await;
    assert_eq!(
        entry_count(&pool).await,
        0,
        "montage : société sans écriture"
    );

    let (status, body) = ctx
        .create(with_opening(base_body(Some(ctx.bank)), "1234.50", "debit"))
        .await;
    assert_eq!(status, 201, "{body}");
    let opening = &body["openingEntry"];
    let entry_id = opening["id"].as_i64().expect("openingEntry.id");
    assert_eq!(opening["entryDate"], json!(ctx.fy_start.to_string()));
    assert_eq!(opening["fiscalYearId"], json!(ctx.fy_id));
    assert!(opening["entryNumber"].as_i64().is_some());
    assert_eq!(body["bankName"], json!("UBS"), "le compte reste aplati");

    let (journal, description): (String, String) =
        sqlx::query_as("SELECT journal, description FROM journal_entries WHERE id = ?")
            .bind(entry_id)
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(journal, "OD");
    assert_eq!(description, "Solde d'ouverture — UBS — Personnel");
    assert_eq!(
        lines_of(&pool, entry_id).await,
        sorted(vec![
            (ctx.bank, dec!(1234.5), Decimal::ZERO),
            (ctx.retained, Decimal::ZERO, dec!(1234.5)),
        ])
    );

    let bank_account_id = body["id"].as_i64().unwrap();
    let details = common::audit_details(
        &pool,
        "bank_account",
        bank_account_id,
        "bank_account.created",
    )
    .await
    .expect("details");
    assert_eq!(details["opening_entry_id"], json!(entry_id));
    assert_eq!(
        count(
            &pool,
            "SELECT COUNT(*) FROM audit_log WHERE action = 'journal_entry.created'"
        )
        .await,
        1
    );

    // AC-4 — l'écran *Soldes de départ* passe en mode *Compléter*, sans le compte
    // de la banque (mouvementé) ; la génération générale est refusée.
    let status: Value = ctx
        .app
        .client
        .get(ctx.app.url("/api/v1/opening-balances/status"))
        .header("Authorization", format!("Bearer {}", ctx.token))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(status["reason"], json!("ALREADY_HAS_ENTRIES"), "{status}");
    assert_eq!(status["completeReason"], json!("READY"), "{status}");
    let completable: Vec<i64> = status["completableAccounts"]
        .as_array()
        .unwrap()
        .iter()
        .map(|a| a["id"].as_i64().unwrap())
        .collect();
    assert!(!completable.contains(&ctx.bank), "{status}");
    assert!(completable.contains(&ctx.cash), "{status}");
    let (s, b) = ctx
        .post(
            "/api/v1/opening-balances",
            json!({ "lines": [
                { "accountId": ctx.cash, "debit": "5", "credit": "0" },
                { "accountId": ctx.retained, "debit": "0", "credit": "5" },
            ]}),
        )
        .await;
    assert_eq!(
        (s, b["error"]["code"].clone()),
        (409, json!("ILLEGAL_STATE_TRANSITION"))
    );
}

/// C-e341-7 — *Dette* sur un compte de passif : le compte lié au crédit, le
/// report au débit.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn creation_avec_solde_crediteur_sur_un_passif(pool: MySqlPool) {
    let ctx = setup(&pool).await;
    let (status, body) = ctx
        .create(with_opening(
            base_body(Some(ctx.overdraft)),
            "300",
            "credit",
        ))
        .await;
    assert_eq!(status, 201, "{body}");
    let entry_id = body["openingEntry"]["id"].as_i64().unwrap();
    assert_eq!(
        lines_of(&pool, entry_id).await,
        sorted(vec![
            (ctx.overdraft, Decimal::ZERO, dec!(300)),
            (ctx.retained, dec!(300), Decimal::ZERO),
        ])
    );
}

/// AC-2 — sans `openingBalance`, ou avec un montant vide ou nul : la réponse
/// d'avant (aucune clé `openingEntry`), aucune écriture, la trace d'avant.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn creation_sans_solde_comportement_d_avant(pool: MySqlPool) {
    let ctx = setup(&pool).await;
    let bodies = [
        base_body(Some(ctx.bank)),
        json!({ "bankName": "UBS", "iban": IBAN_B, "openingBalance": null }),
        with_opening(
            json!({ "bankName": "PostFinance", "iban": IBAN_A }),
            "",
            "debit",
        ),
        with_opening(json!({ "bankName": "BCV", "iban": IBAN_B }), "0", "credit"),
    ];
    for body in bodies {
        let (status, resp) = ctx.create(body).await;
        assert_eq!(status, 201, "{resp}");
        assert!(
            resp.get("openingEntry").is_none(),
            "aucune clé openingEntry : {resp}"
        );
        let id = resp["id"].as_i64().unwrap();
        // Revue P1 (B-8) — la réponse est, au champ près, la sérialisation du
        // compte seul (celle d'avant la 34-2) : aucune clé de plus ni de moins.
        let account =
            kesh_db::repositories::bank_accounts::find_by_id_for_company(&pool, ctx.company_id, id)
                .await
                .unwrap()
                .expect("compte créé");
        assert_eq!(resp, serde_json::to_value(&account).unwrap());
        // Revue P2 (B2-8) — et contre une liste écrite à la main, indépendante
        // du `Serialize` de `BankAccount` : les treize clés du compte d'avant la
        // 34-2, ni plus ni moins — quinze depuis la 34-11a1 (#972), qui ajoute
        // `accountKind` et `cardLast4` à tout compte (intégration de la 0.17.6 :
        // les deux stories, écrites en parallèle, se rencontrent ici).
        let mut cles: Vec<&str> = resp
            .as_object()
            .unwrap()
            .keys()
            .map(String::as_str)
            .collect();
        cles.sort_unstable();
        assert_eq!(
            cles,
            [
                "accountKind",
                "archived",
                "bankName",
                "cardLast4",
                "closedOn",
                "companyId",
                "createdAt",
                "description",
                "iban",
                "id",
                "isPrimary",
                "journalAccountId",
                "qrIban",
                "updatedAt",
                "version",
            ]
        );
        let details = common::audit_details(&pool, "bank_account", id, "bank_account.created")
            .await
            .expect("details");
        assert!(details.get("opening_entry_id").is_none(), "{details}");
    }
    assert_eq!(entry_count(&pool).await, 0);
}

/// AC-2 — un solde sans compte lié : `400 VALIDATION_ERROR`, rien de créé.
/// Tue M5.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn creation_avec_solde_sans_compte_lie_refusee(pool: MySqlPool) {
    let ctx = setup(&pool).await;
    let (status, body) = ctx
        .create(with_opening(base_body(None), "100", "debit"))
        .await;
    assert_eq!(status, 400, "{body}");
    assert_eq!(body["error"]["code"], json!("VALIDATION_ERROR"));
    assert_eq!(bank_account_count(&pool).await, 0);
    assert_eq!(entry_count(&pool).await, 0);
}

/// AC-2 — contrôles de forme du montant (ceux du complément) et sens exigé.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn creation_avec_solde_mal_forme_refusee(pool: MySqlPool) {
    let ctx = setup(&pool).await;
    for amount in ["-5", "abc", "1.00001", "1000000000000000"] {
        let (status, body) = ctx
            .create(with_opening(base_body(Some(ctx.bank)), amount, "debit"))
            .await;
        assert_eq!(status, 400, "{amount} : {body}");
        assert_eq!(
            body["error"]["code"],
            json!("OPENING_COMPLEMENT_INVALID_AMOUNT"),
            "{amount}"
        );
    }
    let mut sans_sens = base_body(Some(ctx.bank));
    sans_sens["openingBalance"] = json!({ "amount": "10" });
    let (status, body) = ctx.create(sans_sens).await;
    assert_eq!(
        (status, body["error"]["code"].clone()),
        (400, json!("VALIDATION_ERROR"))
    );
    let (status, body) = ctx
        .create(with_opening(base_body(Some(ctx.bank)), "10", "sideways"))
        .await;
    assert_eq!(
        (status, body["error"]["code"].clone()),
        (400, json!("VALIDATION_ERROR")),
        "sens inconnu : {body}"
    );
    // Revue P1 (E-1) — un nom de banque plus long que la colonne : `400`, et non
    // plus un `500` de l'`INSERT` après le complément.
    let mut trop_long = with_opening(base_body(Some(ctx.bank)), "10", "debit");
    trop_long["bankName"] = json!("B".repeat(256));
    let (status, body) = ctx.create(trop_long).await;
    assert_eq!(
        (status, body["error"]["code"].clone()),
        (400, json!("VALIDATION_ERROR")),
        "nom trop long : {body}"
    );
    assert_eq!(bank_account_count(&pool).await, 0);
    assert_eq!(entry_count(&pool).await, 0);
}

/// Revue P2 (B2-3 = E2-5) — le `PUT` borne le nom de la banque comme la
/// création : 256 caractères de deux octets → `400 VALIDATION_ERROR`, le compte
/// inchangé ; 255 passent.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn put_borne_le_nom_de_banque(pool: MySqlPool) {
    let ctx = setup(&pool).await;
    let id = insert_bank_account(&pool, ctx.company_id, IBAN_A, true, None).await;
    let put = |name: String| {
        let client = ctx.app.client.clone();
        let url = ctx.app.url(&format!("/api/v1/bank-accounts/{id}"));
        let token = ctx.token.clone();
        async move {
            let resp = client
                .put(url)
                .header("Authorization", format!("Bearer {token}"))
                .json(&json!({
                    "bankName": name,
                    "description": null,
                    "iban": IBAN_A,
                    "isPrimary": true,
                    "version": 1,
                }))
                .send()
                .await
                .unwrap();
            let status = resp.status().as_u16();
            (status, resp.json::<Value>().await.unwrap_or(Value::Null))
        }
    };
    let (status, body) = put("é".repeat(256)).await;
    assert_eq!(
        (status, body["error"]["code"].clone()),
        (400, json!("VALIDATION_ERROR")),
        "{body}"
    );
    let name: String = sqlx::query_scalar("SELECT bank_name FROM bank_accounts WHERE id = ?")
        .bind(id)
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(name, "Banque Test", "compte inchangé");
    let (status, body) = put("é".repeat(255)).await;
    assert_eq!(status, 200, "255 caractères, à la borne : {body}");
}

/// Revue P1 (E-1) — la borne est celle de la colonne, en caractères : 255
/// caractères de deux octets passent, avec leur solde.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn nom_de_banque_a_la_borne_accepte(pool: MySqlPool) {
    let ctx = setup(&pool).await;
    let mut body = with_opening(base_body(Some(ctx.bank)), "10", "debit");
    body["bankName"] = json!("é".repeat(255));
    body["description"] = json!(null);
    let (status, resp) = ctx.create(body).await;
    assert_eq!(status, 201, "{resp}");
    assert!(resp["openingEntry"]["id"].as_i64().is_some(), "{resp}");
}

// ===========================================================================
// Refus du complément : rien d'écrit (AC-4, AC-5)
// ===========================================================================

/// Revue P1 (E-1, A LOW-3) — un échec **après** un complément réussi : l'écriture
/// est déjà dans la transaction quand l'`INSERT` du compte échoue (déclencheur
/// de test, déterministe). Rien ne doit rester : ni compte, ni écriture, ni
/// trace, et l'ancien principal, démoté dans la transaction, l'est toujours.
/// Tue M9 (commit sur erreur) — la ligne « ancien principal » vue rouge.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn un_echec_apres_le_complement_n_ecrit_rien(pool: MySqlPool) {
    let ctx = setup(&pool).await;
    let primary = insert_bank_account(&pool, ctx.company_id, IBAN_B, true, None).await;
    sqlx::raw_sql(
        "CREATE TRIGGER t_echec_insert_bank_account BEFORE INSERT ON bank_accounts \
         FOR EACH ROW BEGIN \
           IF NEW.bank_name = 'Echec apres complement' THEN \
             SIGNAL SQLSTATE '45000' SET MESSAGE_TEXT = 'echec de test'; \
           END IF; \
         END",
    )
    .execute(&pool)
    .await
    .unwrap();
    let (banks, entries, traces) = (
        bank_account_count(&pool).await,
        entry_count(&pool).await,
        common::audit_count(&pool).await,
    );

    let mut body = with_opening(base_body(Some(ctx.bank)), "50", "debit");
    body["bankName"] = json!("Echec apres complement");
    body["isPrimary"] = json!(true);
    let (status, resp) = ctx.create(body).await;
    assert_eq!(status, 500, "{resp}");

    // Les quatre constats sont relevés avant d'être assertés, pour que M9 rende
    // rouge chacun d'eux et non le premier seul (revue P1, A LOW-3).
    let still_primary: bool =
        sqlx::query_scalar("SELECT is_primary FROM bank_accounts WHERE id = ?")
            .bind(primary)
            .fetch_one(&pool)
            .await
            .unwrap();
    let constats = (
        bank_account_count(&pool).await == banks,
        entry_count(&pool).await == entries,
        common::audit_count(&pool).await == traces,
        still_primary,
    );
    assert_eq!(
        constats,
        (true, true, true, true),
        "(aucun compte, aucune écriture orpheline, aucune trace, ancien principal intact)"
    );
}

/// AC-5 (revue P1 A LOW-2) — date verrouillée et compte de report non imputable,
/// sur la route bancaire : le refus sort tel quel, rien d'écrit.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn creation_date_verrouillee_ou_report_non_imputable_refusee(pool: MySqlPool) {
    let ctx = setup(&pool).await;
    // Premier jour et jour même dans la période verrouillée.
    sqlx::query("UPDATE companies SET books_locked_through = ? WHERE id = ?")
        .bind(Utc::now().date_naive())
        .bind(ctx.company_id)
        .execute(&pool)
        .await
        .unwrap();
    let (status, body) = ctx
        .create(with_opening(base_body(Some(ctx.bank)), "50", "debit"))
        .await;
    assert_eq!(
        (status, body["error"]["code"].clone()),
        (409, json!("OPENING_COMPLEMENT_DATE_LOCKED")),
        "{body}"
    );
    sqlx::query("UPDATE companies SET books_locked_through = NULL WHERE id = ?")
        .bind(ctx.company_id)
        .execute(&pool)
        .await
        .unwrap();
    sqlx::query("UPDATE accounts SET postable = FALSE WHERE id = ?")
        .bind(ctx.retained)
        .execute(&pool)
        .await
        .unwrap();
    let (status, body) = ctx
        .create(with_opening(base_body(Some(ctx.bank)), "50", "debit"))
        .await;
    assert_eq!(
        (status, body["error"]["code"].clone()),
        (
            409,
            json!("OPENING_COMPLEMENT_RETAINED_EARNINGS_NOT_POSTABLE")
        ),
        "{body}"
    );
    assert_eq!(bank_account_count(&pool).await, 0);
    assert_eq!(entry_count(&pool).await, 0);
}

/// AC-5 — compte lié déjà mouvementé : `409 OPENING_COMPLEMENT_ACCOUNT_MOVED`
/// tel quel ; ni compte bancaire, ni écriture ; l'ancien principal n'est pas
/// démoté. Tue M2 (refus du complément avalé : compte créé sans écriture). M9
/// (commit sur erreur) n'est pas tuée ici — le refus sort avant toute écriture —
/// mais par `un_echec_apres_le_complement_n_ecrit_rien`.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn creation_sur_un_compte_mouvemente_n_ecrit_rien(pool: MySqlPool) {
    let ctx = setup(&pool).await;
    let primary = insert_bank_account(&pool, ctx.company_id, IBAN_B, true, None).await;
    ctx.move_account(&pool, ctx.bank).await;
    let (banks, entries) = (bank_account_count(&pool).await, entry_count(&pool).await);

    let mut body = with_opening(base_body(Some(ctx.bank)), "50", "debit");
    body["isPrimary"] = json!(true);
    let (status, resp) = ctx.create(body).await;
    assert_eq!(status, 409, "{resp}");
    assert_eq!(
        resp["error"]["code"],
        json!("OPENING_COMPLEMENT_ACCOUNT_MOVED")
    );
    assert!(
        resp["error"]["message"].as_str().unwrap().contains("1020"),
        "le message nomme le compte : {resp}"
    );
    assert_eq!(bank_account_count(&pool).await, banks, "aucun compte créé");
    assert_eq!(entry_count(&pool).await, entries, "aucune écriture");
    let still_primary: bool =
        sqlx::query_scalar("SELECT is_primary FROM bank_accounts WHERE id = ?")
            .bind(primary)
            .fetch_one(&pool)
            .await
            .unwrap();
    assert!(still_primary, "l'ancien principal n'est pas démoté");
}

/// AC-4 — sans aucun exercice : `409 OPENING_COMPLEMENT_NO_OPEN_FISCAL_YEAR`,
/// rien d'écrit. Tue M7 au niveau HTTP.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn creation_sans_exercice_refusee(pool: MySqlPool) {
    let ctx = setup(&pool).await;
    sqlx::query("DELETE FROM fiscal_years")
        .execute(&pool)
        .await
        .unwrap();
    let (status, body) = ctx
        .create(with_opening(base_body(Some(ctx.bank)), "50", "debit"))
        .await;
    assert_eq!(status, 409, "{body}");
    assert_eq!(
        body["error"]["code"],
        json!("OPENING_COMPLEMENT_NO_OPEN_FISCAL_YEAR")
    );
    assert_eq!(bank_account_count(&pool).await, 0);
    assert_eq!(entry_count(&pool).await, 0);
}

/// AC-5 — compte de report absent : le refus du complément sort tel quel.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn creation_sans_compte_de_report_refusee(pool: MySqlPool) {
    let ctx = setup(&pool).await;
    sqlx::query("UPDATE accounts SET role = NULL WHERE id = ?")
        .bind(ctx.retained)
        .execute(&pool)
        .await
        .unwrap();
    let (status, body) = ctx
        .create(with_opening(base_body(Some(ctx.bank)), "50", "debit"))
        .await;
    assert_eq!(status, 409, "{body}");
    assert_eq!(
        body["error"]["code"],
        json!("OPENING_COMPLEMENT_NO_RETAINED_EARNINGS")
    );
    assert_eq!(bank_account_count(&pool).await, 0);
}

/// AC-10 — une écriture **croisée** sur le compte lié, commitée pendant la
/// création : la création la voit (lecture « jamais mouvementé » après ses
/// verrous) et rend `ACCOUNT_MOVED`, sans compte créé.
///
/// Montage : la ligne est ajoutée à une écriture **existante** (une
/// modification d'écriture ne prend pas `companies`, contrairement à une
/// création, que la sentinelle sérialiserait avec la création du compte) ; la
/// création est **principale au-dessus d'un principal existant**, seul cas où la
/// démotion relit sans verrou. Tue M8 (complément après la démotion et
/// l'`INSERT` : l'instantané, ouvert par la relecture de la démotion, ne voit
/// pas la ligne commitée ensuite).
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn ecriture_croisee_pendant_la_creation_est_vue(pool: MySqlPool) {
    let ctx = setup(&pool).await;
    insert_bank_account(&pool, ctx.company_id, IBAN_B, true, None).await;
    let existing = ctx.move_account(&pool, ctx.cash).await;
    let banks = bank_account_count(&pool).await;

    // La transaction croisée : une ligne sur 1020 ajoutée à l'écriture
    // existante, non commitée (verrou partagé de clé étrangère sur 1020).
    let mut tx = pool.begin().await.unwrap();
    sqlx::query(
        "INSERT INTO journal_entry_lines (entry_id, account_id, line_order, debit, credit) \
         VALUES (?, ?, 99, 5, 0)",
    )
    .bind(existing)
    .bind(ctx.bank)
    .execute(&mut *tx)
    .await
    .unwrap();

    let task = {
        let client = ctx.app.client.clone();
        let url = ctx.app.url("/api/v1/bank-accounts");
        let token = ctx.token.clone();
        let mut body = with_opening(base_body(Some(ctx.bank)), "50", "debit");
        body["isPrimary"] = json!(true);
        tokio::spawn(async move {
            let resp = client
                .post(url)
                .header("Authorization", format!("Bearer {token}"))
                .json(&body)
                .send()
                .await
                .unwrap();
            let status = resp.status().as_u16();
            (status, resp.json::<Value>().await.unwrap())
        })
    };
    assert!(
        attendre_une_requete_en_cours(&pool, &["FROM accounts WHERE id IN", "FOR UPDATE"], || {
            task.is_finished()
        })
        .await,
        "la création doit attendre sur le verrou du compte lié"
    );
    tx.commit().await.unwrap();

    let (status, body) = task.await.unwrap();
    assert_eq!(status, 409, "{body}");
    assert_eq!(
        body["error"]["code"],
        json!("OPENING_COMPLEMENT_ACCOUNT_MOVED")
    );
    assert_eq!(bank_account_count(&pool).await, banks, "aucun compte créé");
}

// ===========================================================================
// Carte de crédit et solde d'ouverture — raccords d'intégration de la 0.17.6
// (fiche 34-11a2, C-e34a2-1)
// ===========================================================================

fn card_body(journal_account_id: i64) -> Value {
    json!({
        "bankName": "Visa",
        "accountKind": "credit_card",
        "journalAccountId": journal_account_id,
    })
}

/// Raccord (2) — une carte, liée à un passif, se crée avec son solde
/// d'ouverture au sens `credit` (*Dette*) : le compte de dette au crédit, le
/// report au débit.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn carte_creee_avec_son_solde_d_ouverture(pool: MySqlPool) {
    let ctx = setup(&pool).await;
    let (status, body) = ctx
        .post(
            "/api/v1/bank-accounts",
            with_opening(card_body(ctx.overdraft), "120.50", "credit"),
        )
        .await;
    assert_eq!(status, 201, "{body}");
    assert_eq!(body["accountKind"], json!("credit_card"));
    let entry_id = body["openingEntry"]["id"].as_i64().expect("openingEntry");
    assert_eq!(
        lines_of(&pool, entry_id).await,
        sorted(vec![
            (ctx.overdraft, Decimal::ZERO, dec!(120.50)),
            (ctx.retained, dec!(120.50), Decimal::ZERO),
        ])
    );
}

/// Raccord (4) — le complément de la 34-2 passe **après** la relecture du type
/// (`recheck_ledger_type_in_tx`) : une carte dont le compte lié devient un actif
/// pendant la création (retypage commité pendant que la création attend la
/// ligne `accounts`) est refusée `CREDIT_CARD_LEDGER_MUST_BE_LIABILITY` par la
/// relecture, et **aucune écriture d'ouverture** n'est écrite, ni compte.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn carte_refusee_par_la_relecture_du_type_n_ecrit_aucune_ouverture(pool: MySqlPool) {
    let ctx = setup(&pool).await;
    let banks = bank_account_count(&pool).await;
    let entries = entry_count(&pool).await;

    // Le retypage : la ligne du compte lié tenue `FOR UPDATE`, son type changé
    // en actif, non commité. Le pré-vol de la création lit le pool sans verrou
    // (il voit encore un passif) ; la relecture attend la ligne.
    let mut tx = pool.begin().await.unwrap();
    sqlx::query("SELECT id FROM accounts WHERE id = ? FOR UPDATE")
        .bind(ctx.overdraft)
        .execute(&mut *tx)
        .await
        .unwrap();
    sqlx::query("UPDATE accounts SET account_type = 'Asset' WHERE id = ?")
        .bind(ctx.overdraft)
        .execute(&mut *tx)
        .await
        .unwrap();

    let task = {
        let client = ctx.app.client.clone();
        let url = ctx.app.url("/api/v1/bank-accounts");
        let token = ctx.token.clone();
        let body = with_opening(card_body(ctx.overdraft), "50", "credit");
        tokio::spawn(async move {
            let resp = client
                .post(url)
                .header("Authorization", format!("Bearer {token}"))
                .json(&body)
                .send()
                .await
                .unwrap();
            let status = resp.status().as_u16();
            (status, resp.json::<Value>().await.unwrap())
        })
    };
    assert!(
        attendre_une_requete_en_cours(
            &pool,
            &["account_type, active FROM accounts", "LOCK IN SHARE MODE"],
            || task.is_finished(),
        )
        .await,
        "la création doit attendre la ligne du compte lié à la relecture du type"
    );
    tx.commit().await.unwrap();

    let (status, body) = task.await.unwrap();
    assert_eq!(status, 400, "{body}");
    assert_eq!(
        body["error"]["code"],
        json!("CREDIT_CARD_LEDGER_MUST_BE_LIABILITY")
    );
    assert_eq!(
        entry_count(&pool).await,
        entries,
        "aucune écriture d'ouverture"
    );
    assert_eq!(bank_account_count(&pool).await, banks, "aucun compte créé");
}

// ===========================================================================
// Compte existant : POST /bank-accounts/{id}/opening-balance (AC-6)
// ===========================================================================

fn opening_path(id: i64) -> String {
    format!("/api/v1/bank-accounts/{id}/opening-balance")
}

/// AC-6 — un compte existant, lié, jamais mouvementé : `201` avec l'écriture ;
/// une seconde saisie est refusée `ACCOUNT_MOVED` (le compte l'est désormais).
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn compte_existant_recoit_son_solde(pool: MySqlPool) {
    let ctx = setup(&pool).await;
    let id = insert_bank_account(&pool, ctx.company_id, IBAN_A, true, Some(ctx.bank)).await;

    let (status, body) = ctx
        .post(
            &opening_path(id),
            json!({ "amount": "75.25", "side": "debit" }),
        )
        .await;
    assert_eq!(status, 201, "{body}");
    let entry_id = body["id"].as_i64().unwrap();
    assert_eq!(body["entryDate"], json!(ctx.fy_start.to_string()));
    assert_eq!(body["fiscalYearId"], json!(ctx.fy_id));
    let description: String =
        sqlx::query_scalar("SELECT description FROM journal_entries WHERE id = ?")
            .bind(entry_id)
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(description, "Solde d'ouverture — Banque Test");
    assert_eq!(
        lines_of(&pool, entry_id).await,
        sorted(vec![
            (ctx.bank, dec!(75.25), Decimal::ZERO),
            (ctx.retained, Decimal::ZERO, dec!(75.25)),
        ])
    );
    // Revue P1 (B-4) — la trace relie l'écriture au compte bancaire.
    let details = common::audit_details(
        &pool,
        "bank_account",
        id,
        "bank_account.opening_balance_entered",
    )
    .await
    .expect("details");
    assert_eq!(details["opening_entry_id"], json!(entry_id), "{details}");
    assert_eq!(details["journal_account_id"], json!(ctx.bank), "{details}");

    let (status, body) = ctx
        .post(&opening_path(id), json!({ "amount": "1", "side": "debit" }))
        .await;
    assert_eq!(status, 409, "{body}");
    assert_eq!(
        body["error"]["code"],
        json!("OPENING_COMPLEMENT_ACCOUNT_MOVED")
    );
    assert_eq!(entry_count(&pool).await, 1);
}

/// AC-6 — les refus propres à la route, dans l'ordre : forme, `404` (absent,
/// archivé, d'une autre société), `409` clos (tue M6), `412` sans compte lié.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn compte_existant_refus_de_la_route(pool: MySqlPool) {
    let ctx = setup(&pool).await;
    let body = json!({ "amount": "10", "side": "debit" });

    // Montant nul : rien à saisir.
    let linked = insert_bank_account(&pool, ctx.company_id, IBAN_A, true, Some(ctx.bank)).await;
    let (s, b) = ctx
        .post(
            &opening_path(linked),
            json!({ "amount": "0", "side": "debit" }),
        )
        .await;
    assert_eq!(
        (s, b["error"]["code"].clone()),
        (400, json!("OPENING_COMPLEMENT_INVALID_AMOUNT"))
    );

    // Absent.
    let (s, _) = ctx.post(&opening_path(999_999), body.clone()).await;
    assert_eq!(s, 404);

    // Archivé.
    let archived =
        insert_bank_account(&pool, ctx.company_id, IBAN_B, false, Some(ctx.overdraft)).await;
    sqlx::query("UPDATE bank_accounts SET archived = TRUE WHERE id = ?")
        .bind(archived)
        .execute(&pool)
        .await
        .unwrap();
    let (s, _) = ctx.post(&opening_path(archived), body.clone()).await;
    assert_eq!(s, 404, "archivé");

    // D'une autre société.
    let other_company: i64 = {
        create_test_company(&pool).await;
        sqlx::query_scalar("SELECT MAX(id) FROM companies")
            .fetch_one(&pool)
            .await
            .unwrap()
    };
    let foreign = insert_bank_account(&pool, other_company, IBAN_B, true, None).await;
    let (s, _) = ctx.post(&opening_path(foreign), body.clone()).await;
    assert_eq!(s, 404, "autre société");

    // Clos.
    let closed =
        insert_bank_account(&pool, ctx.company_id, IBAN_B, false, Some(ctx.overdraft)).await;
    sqlx::query("UPDATE bank_accounts SET closed_on = ? WHERE id = ?")
        .bind(ctx.fy_start)
        .bind(closed)
        .execute(&pool)
        .await
        .unwrap();
    let (s, b) = ctx.post(&opening_path(closed), body.clone()).await;
    assert_eq!(
        (s, b["error"]["code"].clone()),
        (409, json!("BANK_ACCOUNT_CLOSED"))
    );

    // Sans compte lié.
    let unlinked = insert_bank_account(&pool, ctx.company_id, IBAN_B, false, None).await;
    let (s, b) = ctx.post(&opening_path(unlinked), body.clone()).await;
    assert_eq!(
        (s, b["error"]["code"].clone()),
        (412, json!("BANK_ACCOUNT_NOT_CONFIGURED"))
    );

    assert_eq!(entry_count(&pool).await, 0, "aucun refus n'écrit");
}

/// Revue P1 (E-2) — un compte bancaire lié au compte débiteurs désigné (lien
/// antérieur à la 15-6c) ne reçoit pas de solde par la route de ligne :
/// `400 BANK_ACCOUNT_LEDGER_IS_CLAIM_ACCOUNT`, comme à la création ; rien d'écrit.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn compte_existant_lie_au_compte_de_creance_refuse(pool: MySqlPool) {
    let ctx = setup(&pool).await;
    let receivable = create_acc(
        &pool,
        ctx.user_id,
        ctx.company_id,
        "1100",
        AccountType::Asset,
        None,
    )
    .await;
    let id = insert_bank_account(&pool, ctx.company_id, IBAN_A, true, Some(receivable)).await;
    sqlx::query(
        "INSERT INTO company_invoice_settings (company_id, default_receivable_account_id) \
         VALUES (?, ?) \
         ON DUPLICATE KEY UPDATE default_receivable_account_id = VALUES(default_receivable_account_id)",
    )
    .bind(ctx.company_id)
    .bind(receivable)
    .execute(&pool)
    .await
    .unwrap();
    let (status, body) = ctx
        .post(
            &opening_path(id),
            json!({ "amount": "10", "side": "debit" }),
        )
        .await;
    assert_eq!(
        (status, body["error"]["code"].clone()),
        (400, json!("BANK_ACCOUNT_LEDGER_IS_CLAIM_ACCOUNT")),
        "{body}"
    );
    assert_eq!(entry_count(&pool).await, 0);
}

/// AC-6 — une clé d'API `read-write` est admise.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn compte_existant_cle_d_api_read_write_admise(pool: MySqlPool) {
    let ctx = setup(&pool).await;
    let id = insert_bank_account(&pool, ctx.company_id, IBAN_A, true, Some(ctx.overdraft)).await;
    let (_, key) = common::create_key_via_http(
        &ctx.app.client,
        &ctx.app.base_url,
        &ctx.token,
        "solde",
        "read-write",
    )
    .await;
    let resp = ctx
        .app
        .client
        .post(ctx.app.url(&opening_path(id)))
        .header("Authorization", format!("Bearer {key}"))
        .json(&json!({ "amount": "20", "side": "credit" }))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 201);
    let body: Value = resp.json().await.unwrap();
    assert_eq!(
        lines_of(&pool, body["id"].as_i64().unwrap()).await,
        sorted(vec![
            (ctx.overdraft, Decimal::ZERO, dec!(20)),
            (ctx.retained, dec!(20), Decimal::ZERO),
        ])
    );
}
