//! Tests E2E HTTP Story 15-5b — `PUT /api/v1/company/invoice-settings` :
//! postabilité des six comptes historiques **à la désignation** (AC10, AC11,
//! #429) et compte créanciers préservé quand il est absent du corps (AC19,
//! #521).
//!
//! Chaque compte refusé ne diffère du compte accepté que par `postable` : même
//! société, actif, bon type. L'assertion porte sur le **code**
//! (`VALIDATION_ERROR`) **et** le message qui nomme le champ — un 400 venu
//! d'ailleurs (type, version) ne passerait pas.
//!
//! Story 15-5d (#429) — la garde **à l'usage** : la validation d'une facture
//! dont la créance désignée est devenue non imputable est refusée en 400
//! `ACCOUNT_NOT_POSTABLE`, avec un message qui renvoie à *Paramètres →
//! Facturation* et à un administrateur — servi ici à un **Comptable**.
//!
//! Pré-requis : MariaDB démarré localement.

use std::net::SocketAddr;
use std::path::Path;
use std::sync::Arc;

use chrono::{TimeDelta, Utc};
use jsonwebtoken::{Algorithm, EncodingKey, Header};
use kesh_api::auth::jwt::Claims;
use kesh_api::auth::password::hash_password;
use kesh_api::config::Config;
use kesh_api::{AppState, build_router};
use kesh_db::entities::account::AccountType;
use kesh_db::entities::address::StructuredAddress;
use kesh_db::entities::{Language, NewAccount, NewCompany, NewUser, OrgType, Role};
use kesh_db::repositories::{accounts, companies, users};
use serde_json::{Value, json};
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
        "e2e-test-admin-password".to_string(),
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
    // Story 15-5d : sans lui, le `message` d'un refus n'est pas résolu par le
    // dictionnaire (patron `invoice_unvalidate_e2e.rs`).
    kesh_api::errors::init_error_i18n(i18n.clone(), config.locale);
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
            Err(e) => panic!("test server not ready in 2s: {e}"),
        }
    }
    TestApp {
        base_url: format!("http://{}", addr),
        client: reqwest::Client::new(),
    }
}

fn forge_admin_jwt(user_id: i64, company_id: i64) -> String {
    let now = Utc::now().timestamp();
    let claims = Claims {
        sub: user_id.to_string(),
        role: "Admin".to_string(),
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

/// Un des six champs historiques : clé JSON, type de compte attendu, libellé
/// du message, numéros du compte en place et du compte cible.
struct Field {
    key: &'static str,
    account_type: AccountType,
    label: &'static str,
    current_number: &'static str,
    target_number: &'static str,
}

const FIELDS: [Field; 6] = [
    Field {
        key: "defaultReceivableAccountId",
        account_type: AccountType::Asset,
        label: "Compte créance",
        current_number: "1100",
        target_number: "1101",
    },
    Field {
        key: "defaultRevenueAccountId",
        account_type: AccountType::Revenue,
        label: "Compte produit",
        current_number: "3000",
        target_number: "3001",
    },
    Field {
        key: "defaultVatPayableAccountId",
        account_type: AccountType::Liability,
        label: "Compte TVA due",
        current_number: "2200",
        target_number: "2201",
    },
    Field {
        key: "defaultVatRecoverableAccountId",
        account_type: AccountType::Asset,
        label: "Compte TVA récupérable",
        current_number: "1170",
        target_number: "1171",
    },
    Field {
        key: "defaultVatDecompteAccountId",
        account_type: AccountType::Liability,
        label: "Compte décompte TVA",
        current_number: "2201",
        target_number: "2202",
    },
    Field {
        key: "defaultPayableAccountId",
        account_type: AccountType::Liability,
        label: "Compte créanciers",
        current_number: "2000",
        target_number: "2001",
    },
];

struct Ctx {
    company_id: i64,
    jwt: String,
    /// `(compte en place, compte cible)` par champ, dans l'ordre de `FIELDS`.
    accounts: Vec<(i64, i64)>,
}

async fn create_account(
    pool: &MySqlPool,
    company_id: i64,
    user_id: i64,
    number: &str,
    account_type: AccountType,
) -> i64 {
    // Un même numéro peut servir deux champs (2201) : on le réutilise.
    if let Some(id) =
        sqlx::query_scalar::<_, i64>("SELECT id FROM accounts WHERE company_id = ? AND number = ?")
            .bind(company_id)
            .bind(number)
            .fetch_optional(pool)
            .await
            .unwrap()
    {
        return id;
    }
    accounts::create(
        pool,
        user_id,
        NewAccount {
            company_id,
            number: number.into(),
            name: format!("Compte {number}"),
            account_type,
            parent_id: None,
            role: None,
            postable: true,
        },
    )
    .await
    .unwrap()
    .id
}

async fn set_postable(pool: &MySqlPool, account_id: i64, postable: bool) {
    sqlx::query("UPDATE accounts SET postable = ?, version = version + 1 WHERE id = ?")
        .bind(postable)
        .bind(account_id)
        .execute(pool)
        .await
        .expect("set postable");
}

/// Société + Admin + les comptes des six champs, puis un premier PUT qui
/// désigne les comptes « en place », tous imputables.
async fn setup(pool: &MySqlPool, app: &TestApp) -> Ctx {
    let company_id = companies::create(
        pool,
        NewCompany {
            name: "Réglages SA".into(),
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
    .id;
    let user_id = users::create(
        pool,
        NewUser {
            username: "reglages_admin".into(),
            password_hash: hash_password("password123").unwrap(),
            role: Role::Admin,
            active: true,
            company_id,
            email: None,
        },
    )
    .await
    .unwrap()
    .id;
    let mut ids = Vec::new();
    for f in &FIELDS {
        let current =
            create_account(pool, company_id, user_id, f.current_number, f.account_type).await;
        let target =
            create_account(pool, company_id, user_id, f.target_number, f.account_type).await;
        ids.push((current, target));
    }
    let ctx = Ctx {
        company_id,
        jwt: forge_admin_jwt(user_id, company_id),
        accounts: ids,
    };
    let mut body = current_body(app, &ctx).await;
    for (i, f) in FIELDS.iter().enumerate() {
        body[f.key] = json!(ctx.accounts[i].0);
    }
    let resp = put(app, &ctx, &body).await;
    assert_eq!(resp.status(), 200, "{}", resp.text().await.unwrap());
    ctx
}

/// Le corps d'un PUT qui renvoie les réglages tels qu'ils sont.
async fn current_body(app: &TestApp, ctx: &Ctx) -> Value {
    let resp = app
        .client
        .get(app.url("/api/v1/company/invoice-settings"))
        .bearer_auth(&ctx.jwt)
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let s: Value = resp.json().await.unwrap();
    json!({
        "invoiceNumberFormat": s["invoiceNumberFormat"],
        "defaultReceivableAccountId": s["defaultReceivableAccountId"],
        "defaultRevenueAccountId": s["defaultRevenueAccountId"],
        "defaultVatPayableAccountId": s["defaultVatPayableAccountId"],
        "defaultVatRecoverableAccountId": s["defaultVatRecoverableAccountId"],
        "defaultVatDecompteAccountId": s["defaultVatDecompteAccountId"],
        "defaultSalesJournal": s["defaultSalesJournal"],
        "journalEntryDescriptionTemplate": s["journalEntryDescriptionTemplate"],
        "defaultPayableAccountId": s["defaultPayableAccountId"],
        "version": s["version"],
    })
}

async fn put(app: &TestApp, ctx: &Ctx, body: &Value) -> reqwest::Response {
    app.client
        .put(app.url("/api/v1/company/invoice-settings"))
        .bearer_auth(&ctx.jwt)
        .json(body)
        .send()
        .await
        .unwrap()
}

async fn stored_payable(pool: &MySqlPool, company_id: i64) -> Option<i64> {
    sqlx::query_scalar(
        "SELECT default_payable_account_id FROM company_invoice_settings WHERE company_id = ?",
    )
    .bind(company_id)
    .fetch_one(pool)
    .await
    .unwrap()
}

/// AC10 — pour **chacun** des six champs, désigner un compte non imputable du
/// bon type est refusé : 400 `VALIDATION_ERROR`, message qui nomme le champ
/// et les trois sortes de comptes non imputables (choix C19).
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn each_historical_field_rejects_a_new_non_postable_account(pool: MySqlPool) {
    let app = spawn_app(pool.clone()).await;
    let ctx = setup(&pool, &app).await;
    for (i, f) in FIELDS.iter().enumerate() {
        let target = ctx.accounts[i].1;
        set_postable(&pool, target, false).await;
        let mut body = current_body(&app, &ctx).await;
        body[f.key] = json!(target);
        let resp = put(&app, &ctx, &body).await;
        assert_eq!(resp.status(), 400, "{}", f.key);
        let err: Value = resp.json().await.unwrap();
        assert_eq!(err["error"]["code"], "VALIDATION_ERROR", "{}", f.key);
        assert_eq!(
            err["error"]["message"],
            format!(
                "{} : compte non imputable (compte de regroupement, de résultat ou de clôture)",
                f.label
            ),
            "{}",
            f.key
        );
        // Le compte cible redevient imputable : il peut servir de valeur en
        // place à un autre champ (2201).
        set_postable(&pool, target, true).await;
    }
}

/// AC11 — exemption « inchangé » : les six comptes en place deviennent non
/// imputables, et un PUT qui les renvoie tels quels passe.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn unchanged_non_postable_accounts_do_not_block_saving(pool: MySqlPool) {
    let app = spawn_app(pool.clone()).await;
    let ctx = setup(&pool, &app).await;
    for (current, _) in &ctx.accounts {
        set_postable(&pool, *current, false).await;
    }
    let mut body = current_body(&app, &ctx).await;
    body["journalEntryDescriptionTemplate"] = json!("{YEAR}-{INVOICE_NUMBER}");
    let resp = put(&app, &ctx, &body).await;
    assert_eq!(resp.status(), 200, "{}", resp.text().await.unwrap());
}

/// AC19 (#521) — `defaultPayableAccountId` **absent** du corps : le compte
/// créanciers en place est conservé (relu après).
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn absent_payable_account_is_preserved(pool: MySqlPool) {
    let app = spawn_app(pool.clone()).await;
    let ctx = setup(&pool, &app).await;
    let payable = ctx.accounts[5].0;
    assert_eq!(stored_payable(&pool, ctx.company_id).await, Some(payable));

    let mut body = current_body(&app, &ctx).await;
    body.as_object_mut()
        .unwrap()
        .remove("defaultPayableAccountId");
    let resp = put(&app, &ctx, &body).await;
    assert_eq!(resp.status(), 200, "{}", resp.text().await.unwrap());
    assert_eq!(
        stored_payable(&pool, ctx.company_id).await,
        Some(payable),
        "le compte créanciers n'est plus effacé (#521)"
    );
}

/// AC19 — absent du corps et **devenu non imputable** : préservé sans
/// contrôle (on ne refuse pas un enregistrement pour un champ non envoyé).
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn absent_payable_account_is_preserved_without_control(pool: MySqlPool) {
    let app = spawn_app(pool.clone()).await;
    let ctx = setup(&pool, &app).await;
    let payable = ctx.accounts[5].0;
    sqlx::query("UPDATE accounts SET active = FALSE, version = version + 1 WHERE id = ?")
        .bind(payable)
        .execute(&pool)
        .await
        .unwrap();
    let mut body = current_body(&app, &ctx).await;
    body.as_object_mut()
        .unwrap()
        .remove("defaultPayableAccountId");
    let resp = put(&app, &ctx, &body).await;
    assert_eq!(resp.status(), 200, "{}", resp.text().await.unwrap());
    assert_eq!(stored_payable(&pool, ctx.company_id).await, Some(payable));
}

/// AC19 — `null` efface ; une autre valeur valide remplace.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn null_clears_and_value_replaces_payable_account(pool: MySqlPool) {
    let app = spawn_app(pool.clone()).await;
    let ctx = setup(&pool, &app).await;
    let other = ctx.accounts[5].1;

    let mut body = current_body(&app, &ctx).await;
    body["defaultPayableAccountId"] = json!(other);
    let resp = put(&app, &ctx, &body).await;
    assert_eq!(resp.status(), 200, "{}", resp.text().await.unwrap());
    assert_eq!(stored_payable(&pool, ctx.company_id).await, Some(other));

    let mut body = current_body(&app, &ctx).await;
    body["defaultPayableAccountId"] = Value::Null;
    let resp = put(&app, &ctx, &body).await;
    assert_eq!(resp.status(), 200, "{}", resp.text().await.unwrap());
    assert_eq!(stored_payable(&pool, ctx.company_id).await, None);
}

// ---------------------------------------------------------------------------
// Story 15-5d (#429) — la garde À L'USAGE, par la route de validation
// ---------------------------------------------------------------------------

/// La validation d'une facture dont la **créance désignée** est devenue non
/// imputable : 400 `ACCOUNT_NOT_POSTABLE`, `details.rejected`, un message qui
/// renvoie à *Paramètres → Facturation*, nomme le numéro et dit qu'un
/// administrateur doit agir — servi à un **Comptable**, qui n'a pas accès à la
/// page des réglages (choix C36). La facture reste brouillon.
///
/// Montage (findings F5-4 = R5-3) : `setup` ne suffit pas à valider une facture
/// (ni exercice, ni contact, ni taux) ; il est remplacé par
/// `seed_accounting_company` (exercice, comptes, réglages : créance `1100`, TVA
/// due `2000`, taux), l'arrondi à 5 centimes désactivé, un contact client et un
/// utilisateur Comptable connecté par la vraie route de login.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn validation_refuses_a_designated_receivable_made_non_postable(pool: MySqlPool) {
    use kesh_db::entities::contact::{ContactType, NewContact, Salutation};
    use kesh_db::entities::{NewInvoice, NewInvoiceLine};
    use kesh_db::repositories::{contacts, invoices};
    use rust_decimal_macros::dec;

    let app = spawn_app(pool.clone()).await;
    let seeded = kesh_db::test_fixtures::seed_accounting_company(&pool)
        .await
        .unwrap();
    kesh_db::test_fixtures::disable_rounding_to_5_centimes(&pool, seeded.company_id)
        .await
        .unwrap();
    let contact_id = contacts::create(
        &pool,
        seeded.admin_user_id,
        NewContact {
            company_id: seeded.company_id,
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
            salutation: Salutation::Neutre,
        },
    )
    .await
    .unwrap()
    .id;
    let (invoice, _) = invoices::create(
        &pool,
        seeded.admin_user_id,
        NewInvoice {
            company_id: seeded.company_id,
            contact_id,
            date: chrono::NaiveDate::from_ymd_opt(2026, 6, 15).unwrap(),
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
        },
    )
    .await
    .unwrap();

    // La créance, DÉSIGNÉE (par le seed), puis rendue non imputable.
    let receivable = seeded.accounts["1100"];
    set_postable(&pool, receivable, false).await;

    users::create(
        &pool,
        NewUser {
            username: "comptable".into(),
            password_hash: hash_password("comptable123").unwrap(),
            role: Role::Comptable,
            active: true,
            company_id: seeded.company_id,
            email: None,
        },
    )
    .await
    .unwrap();
    let login = app
        .client
        .post(app.url("/api/v1/auth/login"))
        .json(&json!({ "username": "comptable", "password": "comptable123" }))
        .send()
        .await
        .unwrap();
    assert_eq!(login.status(), 200, "login du Comptable");
    let token = login.json::<Value>().await.unwrap()["accessToken"]
        .as_str()
        .unwrap()
        .to_string();

    let resp = app
        .client
        .post(app.url(&format!("/api/v1/invoices/{}/validate", invoice.id)))
        .bearer_auth(&token)
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 400);
    let body: Value = resp.json().await.unwrap();
    assert_eq!(body["error"]["code"], "ACCOUNT_NOT_POSTABLE", "{body}");
    assert_eq!(
        body["error"]["details"]["rejected"],
        json!([{ "accountId": receivable, "accountNumber": "1100" }]),
        "{body}"
    );
    let message = body["error"]["message"].as_str().unwrap();
    for attendu in ["Paramètres → Facturation", "1100", "administrateur"] {
        assert!(
            message.contains(attendu),
            "« {attendu} » absent : {message}"
        );
    }

    let status: String = sqlx::query_scalar("SELECT status FROM invoices WHERE id = ?")
        .bind(invoice.id)
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(status, "draft", "la facture doit rester brouillon");
}

// ---------------------------------------------------------------------------
// Story 15-6c (#474) — un compte lié à un compte bancaire non archivé ne se
// désigne ni comme compte débiteurs ni comme compte créanciers :
// 400 `CLAIM_ACCOUNT_LINKED_TO_BANK_ACCOUNT`, contrôlé dans la transaction du
// dépôt, après le 409, seulement si la valeur change.
// ---------------------------------------------------------------------------

/// Un compte bancaire lié en SQL au compte `account_id` (contourne la garde du
/// lien : c'est l'état qu'un lien antérieur ou concurrent laisse en base).
async fn bank_linked_to(
    pool: &MySqlPool,
    company_id: i64,
    name: &str,
    iban: &str,
    account_id: i64,
    archived: bool,
) -> i64 {
    sqlx::query(
        "INSERT INTO bank_accounts (company_id, bank_name, iban, is_primary, journal_account_id, archived) \
         VALUES (?, ?, ?, FALSE, ?, ?)",
    )
    .bind(company_id)
    .bind(name)
    .bind(iban)
    .bind(account_id)
    .bind(archived)
    .execute(pool)
    .await
    .unwrap()
    .last_insert_id() as i64
}

async fn assert_linked_refusal(
    resp: reqwest::Response,
    account_id: i64,
    number: &str,
    claim: &str,
    bank_account_id: i64,
    bank_name: &str,
) -> Value {
    let status = resp.status();
    let err: Value = resp.json().await.unwrap();
    assert_eq!(status, 400, "{err}");
    assert_eq!(
        err["error"]["code"], "CLAIM_ACCOUNT_LINKED_TO_BANK_ACCOUNT",
        "{err}"
    );
    assert_eq!(
        err["error"]["details"],
        json!({
            "accountId": account_id,
            "accountNumber": number,
            "claim": claim,
            "bankAccountId": bank_account_id,
            "bankName": bank_name,
        })
    );
    err
}

/// Test 6 (AC5) — compte débiteurs déplacé vers un compte lié à un compte
/// bancaire → 400 nommant ce compte bancaire ; le même compte lié à un compte
/// bancaire **archivé** → accepté ; valeur **inchangée** alors qu'un lien
/// fautif existe → accepté (patron C4 de la 15-5b).
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn receivable_linked_to_a_bank_account_is_refused_when_it_changes(pool: MySqlPool) {
    let app = spawn_app(pool.clone()).await;
    let ctx = setup(&pool, &app).await;
    let target = ctx.accounts[0].1;
    let bank = bank_linked_to(
        &pool,
        ctx.company_id,
        "UBS courant",
        "CH9300762011623852957",
        target,
        false,
    )
    .await;

    let mut body = current_body(&app, &ctx).await;
    body["defaultReceivableAccountId"] = json!(target);
    let resp = put(&app, &ctx, &body).await;
    assert_linked_refusal(resp, target, "1101", "receivable", bank, "UBS courant").await;

    // Archivé : le serveur ne refuse que le lien à un compte bancaire actif.
    sqlx::query("UPDATE bank_accounts SET archived = TRUE WHERE id = ?")
        .bind(bank)
        .execute(&pool)
        .await
        .unwrap();
    let resp = put(&app, &ctx, &body).await;
    assert_eq!(resp.status(), 200, "{}", resp.text().await.unwrap());

    // Inchangé : un lien fautif (antérieur) sur la valeur en place ne bloque
    // pas l'enregistrement des autres réglages.
    bank_linked_to(
        &pool,
        ctx.company_id,
        "PostFinance",
        "CH3908704016075473007",
        target,
        false,
    )
    .await;
    let mut body = current_body(&app, &ctx).await;
    assert_eq!(body["defaultReceivableAccountId"], json!(target));
    body["journalEntryDescriptionTemplate"] = json!("{YEAR}-{INVOICE_NUMBER}");
    let resp = put(&app, &ctx, &body).await;
    assert_eq!(resp.status(), 200, "{}", resp.text().await.unwrap());
}

/// Test 7 (AC2) — **deux** comptes bancaires non archivés liés au même compte :
/// le refus nomme le **premier par `id`**.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn claim_refusal_names_the_first_linked_bank_account_by_id(pool: MySqlPool) {
    let app = spawn_app(pool.clone()).await;
    let ctx = setup(&pool, &app).await;
    let target = ctx.accounts[0].1;
    let first = bank_linked_to(
        &pool,
        ctx.company_id,
        "Premier",
        "CH9300762011623852957",
        target,
        false,
    )
    .await;
    let second = bank_linked_to(
        &pool,
        ctx.company_id,
        "Second",
        "CH3908704016075473007",
        target,
        false,
    )
    .await;
    assert!(first < second);

    let mut body = current_body(&app, &ctx).await;
    body["defaultReceivableAccountId"] = json!(target);
    let resp = put(&app, &ctx, &body).await;
    assert_linked_refusal(resp, target, "1101", "receivable", first, "Premier").await;
}

/// Test 8 (AC5) — `defaultPayableAccountId` **présent** vers un compte de
/// passif lié → `claim: "payable"` ; **absent** du corps → valeur en place
/// préservée, aucun contrôle, même liée (AC19 de la 15-5b).
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn payable_linked_to_a_bank_account_is_refused_only_when_sent_and_changed(pool: MySqlPool) {
    let app = spawn_app(pool.clone()).await;
    let ctx = setup(&pool, &app).await;
    let (current, target) = ctx.accounts[5];
    let bank = bank_linked_to(
        &pool,
        ctx.company_id,
        "Crédit",
        "CH9300762011623852957",
        target,
        false,
    )
    .await;

    let mut body = current_body(&app, &ctx).await;
    body["defaultPayableAccountId"] = json!(target);
    let resp = put(&app, &ctx, &body).await;
    assert_linked_refusal(resp, target, "2001", "payable", bank, "Crédit").await;

    bank_linked_to(
        &pool,
        ctx.company_id,
        "Débit",
        "CH3908704016075473007",
        current,
        false,
    )
    .await;
    let mut body = current_body(&app, &ctx).await;
    body.as_object_mut()
        .unwrap()
        .remove("defaultPayableAccountId");
    body["journalEntryDescriptionTemplate"] = json!("{YEAR}-{INVOICE_NUMBER}");
    let resp = put(&app, &ctx, &body).await;
    assert_eq!(resp.status(), 200, "{}", resp.text().await.unwrap());
    assert_eq!(stored_payable(&pool, ctx.company_id).await, Some(current));
}

/// Test 9 (AC6, ordre) — `version` périmée **et** compte débiteurs lié : **409**
/// (le contrôle de version précède le refus).
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn stale_version_precedes_the_linked_claim_refusal(pool: MySqlPool) {
    let app = spawn_app(pool.clone()).await;
    let ctx = setup(&pool, &app).await;
    let target = ctx.accounts[0].1;
    bank_linked_to(
        &pool,
        ctx.company_id,
        "UBS",
        "CH9300762011623852957",
        target,
        false,
    )
    .await;

    let mut body = current_body(&app, &ctx).await;
    body["defaultReceivableAccountId"] = json!(target);
    body["version"] = json!(body["version"].as_i64().unwrap() - 1);
    let resp = put(&app, &ctx, &body).await;
    assert_eq!(resp.status(), 409, "{}", resp.text().await.unwrap());
}

/// Test 10 (AC1) — le message **français** du refus nomme le compte et le
/// compte bancaire (langue globale au processus, `init_error_i18n` du
/// montage ; la parité des clés dans les autres locales est contrôlée par le
/// test de `kesh-i18n`).
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn linked_claim_refusal_message_is_french(pool: MySqlPool) {
    let app = spawn_app(pool.clone()).await;
    let ctx = setup(&pool, &app).await;
    let target = ctx.accounts[0].1;
    let bank = bank_linked_to(
        &pool,
        ctx.company_id,
        "UBS",
        "CH9300762011623852957",
        target,
        false,
    )
    .await;

    let mut body = current_body(&app, &ctx).await;
    body["defaultReceivableAccountId"] = json!(target);
    let resp = put(&app, &ctx, &body).await;
    let err = assert_linked_refusal(resp, target, "1101", "receivable", bank, "UBS").await;
    assert_eq!(
        err["error"]["message"],
        "Le compte 1101 est lié au compte bancaire UBS : il ne peut pas servir de compte débiteurs."
    );
}
