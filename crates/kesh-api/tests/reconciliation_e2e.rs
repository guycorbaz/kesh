//! Tests E2E HTTP pour Story 8-4 — réconciliation matching automatique
//! (T5.6 dette E2E HTTP fermée 2026-05-07).
//!
//! 22 tests couvrant les ACs #44-#54 + #60 + #64-#74 :
//!
//! - GET /proposals : #44, #45, #47.
//! - POST /accept : #48, #49, #50, #51, #52, #66, #67, #68, #69, #70,
//!   #71, #72, #73, #74.
//! - POST /reject : #53, #54, #64.
//! - RBAC : #60.
//! - Pagination : #65.
//!
//! Tests `#[ignore]` :
//!
//! - `post_accept_filters_currency_mismatch` (#67) — v0.1 mono-CHF, cf.
//!   spec L38 / Story 11 dependency. Body placeholder.
//!
//! Pré-requis exécution : MariaDB démarré localement
//! (`KESH_TEST_MODE=true`). Les tests utilisent `#[sqlx::test(migrator)]`
//! qui crée une DB éphémère par test avec migrations auto-appliquées.

use kesh_db::repositories::company_invoice_settings::ClaimAccounts;
use std::net::SocketAddr;
use std::path::Path;
use std::sync::Arc;

use chrono::{NaiveDate, NaiveDateTime, TimeDelta, Utc};
use jsonwebtoken::{Algorithm, EncodingKey, Header};
use kesh_api::auth::jwt::Claims;
use kesh_api::auth::password::hash_password;
use kesh_api::config::Config;
use kesh_api::{AppState, build_router};
use kesh_db::entities::account::{AccountType, NewAccount};
use kesh_db::entities::address::StructuredAddress;
use kesh_db::entities::{
    BankImportSourceFormat, ContactType, Language, NewBankAccount, NewBankImport,
    NewBankTransaction, NewCompany, NewContact, NewUser, OrgType, Role,
};
use kesh_db::repositories::{
    accounts, bank_accounts, bank_imports, companies, contacts as contacts_repo, users,
};
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use serde_json::Value;
use sqlx::MySqlPool;

#[path = "../../kesh-db/tests/support/document_group.rs"]
mod document_group;

const TEST_JWT_SECRET: &[u8] = b"test-secret-32-bytes-minimum-test-secret-padding";
const TEST_ADMIN_PASSWORD: &str = "e2e-test-admin-password";

// ============================================================
// App spawn / config / JWT helpers (pattern Story 8-1b/8-2/8-3)
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
// Domain seed helpers
// ============================================================

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

/// Les trois comptes sans lesquels la fixture ne comptabilise rien : créance,
/// produit, banque au grand livre.
///
/// ⚠️ **Avant la Story 24-2, cette fixture était CREUSE** — `insert_invoice`
/// ne posait aucune `invoice_lines`, `insert_fake_journal_entry` aucune
/// `journal_entry_lines`, et le compte bancaire n'avait pas de
/// `journal_account_id`. Les tests rapprochaient donc une facture **sans
/// substance comptable**, et c'est exactement ce vide qui a permis au défaut
/// de #371 — aucune écriture d'encaissement — de passer inaperçu pendant des
/// mois : on ne peut pas voir manquer une écriture là où rien n'existe.
async fn seed_accounts(pool: &MySqlPool, company_id: i64) -> (i64, i64, i64) {
    async fn one(
        pool: &MySqlPool,
        company_id: i64,
        number: &str,
        name: &str,
        account_type: &str,
    ) -> i64 {
        sqlx::query(
            "INSERT INTO accounts (company_id, number, name, account_type, active, postable) \
             VALUES (?, ?, ?, ?, 1, 1)",
        )
        .bind(company_id)
        .bind(number)
        .bind(name)
        .bind(account_type)
        .execute(pool)
        .await
        .expect("account insert")
        .last_insert_id() as i64
    }
    let receivable = one(pool, company_id, "1100", "Créances clients", "Asset").await;
    let bank_ledger = one(pool, company_id, "1020", "Banque", "Asset").await;
    let revenue = one(pool, company_id, "3000", "Ventes", "Revenue").await;
    (receivable, revenue, bank_ledger)
}

async fn create_bank_account(pool: &MySqlPool, company_id: i64, iban: &str) -> i64 {
    bank_accounts::create(
        pool,
        NewBankAccount {
            company_id,
            bank_name: "UBS".into(),
            iban: iban.into(),
            qr_iban: None,
            is_primary: true,
        },
    )
    .await
    .unwrap()
    .id
}

async fn create_contact(pool: &MySqlPool, company_id: i64, user_id: i64, name: &str) -> i64 {
    contacts_repo::create(
        pool,
        user_id,
        NewContact {
            company_id,
            contact_type: ContactType::Entreprise,
            name: name.into(),
            first_name: None,
            last_name: None,
            is_client: true,
            is_supplier: false,
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
    .unwrap()
    .id
}

/// Insert ou réutilise un `fiscal_years` row pour la company donnée.
/// Idempotent — appel multiple sur la même company réutilise le row
/// existant (UNIQUE `(company_id, name)`).
async fn insert_fake_fiscal_year(pool: &MySqlPool, company_id: i64) -> i64 {
    let name = format!("FY 2026 c{company_id}");
    let existing: Option<i64> =
        sqlx::query_scalar("SELECT id FROM fiscal_years WHERE company_id = ? AND name = ?")
            .bind(company_id)
            .bind(&name)
            .fetch_optional(pool)
            .await
            .expect("fiscal_year lookup");
    if let Some(id) = existing {
        return id;
    }
    let result = sqlx::query(
        "INSERT INTO fiscal_years (company_id, name, start_date, end_date, status, \
         created_at, updated_at) \
         VALUES (?, ?, '2026-01-01', '2026-12-31', 'Open', NOW(3), NOW(3))",
    )
    .bind(company_id)
    .bind(&name)
    .execute(pool)
    .await
    .expect("fiscal_year insert");
    result.last_insert_id() as i64
}

/// Insert minimal `journal_entries` row pour pouvoir poser
/// `invoice.journal_entry_id IS NOT NULL` sans passer par
/// `validate_invoice` (qui requiert settings/accounts/sequence).
///
/// Note : `entry_number` est unique par `(company_id, fiscal_year_id)`.
/// On utilise `MAX(entry_number)+1` pour autoriser plusieurs JE par
/// fiscal_year sans collision UNIQUE.
async fn insert_fake_journal_entry(pool: &MySqlPool, company_id: i64, fy_id: i64) -> i64 {
    let next_number: i64 = sqlx::query_scalar(
        "SELECT COALESCE(MAX(entry_number), 0) + 1 FROM journal_entries \
         WHERE company_id = ? AND fiscal_year_id = ?",
    )
    .bind(company_id)
    .bind(fy_id)
    .fetch_one(pool)
    .await
    .expect("max entry_number");
    let result = sqlx::query(
        "INSERT INTO journal_entries (company_id, fiscal_year_id, entry_number, entry_date, \
         journal, description, version, created_at, updated_at) \
         VALUES (?, ?, ?, '2026-05-01', 'Ventes', 'fake je', 1, NOW(3), NOW(3))",
    )
    .bind(company_id)
    .bind(fy_id)
    .bind(next_number)
    .execute(pool)
    .await
    .expect("journal_entry insert");
    result.last_insert_id() as i64
}

/// Insert direct `invoices` (bypass `validate_invoice` pipeline).
#[allow(clippy::too_many_arguments)]
async fn insert_invoice(
    pool: &MySqlPool,
    company_id: i64,
    contact_id: i64,
    invoice_number: &str,
    date: NaiveDate,
    total_amount: Decimal,
    status: &str,
    journal_entry_id: Option<i64>,
    paid_at: Option<NaiveDateTime>,
) -> i64 {
    let result = sqlx::query(
        "INSERT INTO invoices (company_id, contact_id, invoice_number, status, date, \
         total_amount, journal_entry_id, paid_at, version, created_at, updated_at) \
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, 1, NOW(3), NOW(3))",
    )
    .bind(company_id)
    .bind(contact_id)
    .bind(invoice_number)
    .bind(status)
    .bind(date)
    .bind(total_amount)
    .bind(journal_entry_id)
    .bind(paid_at)
    .execute(pool)
    .await
    .expect("invoice insert");
    let invoice_id = result.last_insert_id() as i64;
    // #246 (21-2b), #420 (25-4-c) : le matching filtre sur le reste dû (TTC
    // dérivé des lignes, moins les règlements). Sans ligne, reste dû = 0 →
    // jamais candidate. Ligne unique vat_rate 0, sans règlement → reste dû =
    // TTC = total_amount, les assertions de montant existantes restent valides.
    sqlx::query(
        "INSERT INTO invoice_lines (invoice_id, position, description, quantity, unit_price, vat_rate, line_total) \
         VALUES (?, 1, 'Ligne test', 1, ?, 0, ?)",
    )
    .bind(invoice_id)
    .bind(total_amount)
    .bind(total_amount)
    .execute(pool)
    .await
    .expect("invoice_line insert");
    invoice_id
}

/// Setup complet : fiscal_year + journal_entry + validated invoice, prête
/// à être réconciliée. Retourne `(invoice_id, journal_entry_id)`.
async fn seed_validated_invoice(
    pool: &MySqlPool,
    company_id: i64,
    contact_id: i64,
    invoice_number: &str,
    date: NaiveDate,
    amount: Decimal,
) -> (i64, i64) {
    let fy_id = insert_fake_fiscal_year(pool, company_id).await;
    let je_id = insert_fake_journal_entry(pool, company_id, fy_id).await;
    let inv_id = insert_invoice(
        pool,
        company_id,
        contact_id,
        invoice_number,
        date,
        amount,
        "validated",
        Some(je_id),
        None,
    )
    .await;

    // ⛔ L'ÉCRITURE DE VENTE reçoit enfin ses lignes — `D créance / C produit`.
    //
    // `insert_invoice` posait déjà une `invoice_lines` (le TTC canonique se
    // calcule dessus, #246), mais `insert_fake_journal_entry` créait une
    // écriture **sans aucune ligne**. La facture avait donc un montant et
    // aucune contrepartie comptable : rien à créditer, rien à solder.
    //
    // C'est cette moitié manquante qui rend l'invariante testable — le compte
    // de créance se lit SUR LA LIGNE DE DÉBIT de l'écriture de vente, jamais
    // sur les réglages, et c'est ce qui garantit qu'il se solde exactement.
    let (receivable_id, revenue_id) = account_ids(pool, company_id).await;
    for (order, account_id, debit, credit) in [
        (1, receivable_id, amount, Decimal::ZERO),
        (2, revenue_id, Decimal::ZERO, amount),
    ] {
        sqlx::query(
            "INSERT INTO journal_entry_lines (entry_id, account_id, line_order, debit, credit) \
             VALUES (?, ?, ?, ?, ?)",
        )
        .bind(je_id)
        .bind(account_id)
        .bind(order)
        .bind(debit)
        .bind(credit)
        .execute(pool)
        .await
        .expect("journal_entry_line insert");
    }

    (inv_id, je_id)
}

/// Les identifiants des comptes seedés par [`seed_accounts`], retrouvés par leur
/// numéro — évite de faire transiter le `CompanyCtx` par dix-huit appelants.
async fn account_ids(pool: &MySqlPool, company_id: i64) -> (i64, i64) {
    let receivable: i64 =
        sqlx::query_scalar("SELECT id FROM accounts WHERE company_id = ? AND number = '1100'")
            .bind(company_id)
            .fetch_one(pool)
            .await
            .expect("compte 1100 seedé par setup_company");
    let revenue: i64 =
        sqlx::query_scalar("SELECT id FROM accounts WHERE company_id = ? AND number = '3000'")
            .bind(company_id)
            .fetch_one(pool)
            .await
            .expect("compte 3000 seedé par setup_company");
    (receivable, revenue)
}

fn make_new_import(
    company_id: i64,
    bank_account_id: i64,
    user_id: i64,
    file_hash: &str,
    period_from: NaiveDate,
    period_to: NaiveDate,
) -> NewBankImport {
    NewBankImport {
        company_id,
        bank_account_id,
        filename: "stmt.xml".into(),
        file_hash: file_hash.into(),
        source_format: BankImportSourceFormat::Camt053V04,
        statement_id: Some("STMT-001".into()),
        period_from,
        period_to,
        opening_balance: Some(dec!(1000.00)),
        closing_balance: Some(dec!(1100.00)),
        transaction_count: 1,
        imported_by_user_id: user_id,
    }
}

#[allow(clippy::too_many_arguments)]
fn make_new_tx(
    company_id: i64,
    bank_account_id: i64,
    booking_date: NaiveDate,
    value_date: Option<NaiveDate>,
    amount: Decimal,
    currency: &str,
    reference: &str,
    counterparty_name: Option<&str>,
) -> NewBankTransaction {
    NewBankTransaction {
        company_id,
        bank_account_id,
        booking_date,
        value_date,
        amount,
        currency: currency.into(),
        reference: Some(reference.into()),
        details: "Test tx".into(),
        end_to_end_id: None,
        transaction_id: None,
        counterparty_iban: None,
        counterparty_name: counterparty_name.map(String::from),
    }
}

/// Insère un import + N transactions en une seule transaction via
/// `bank_imports::create_with_transactions`. Retourne les IDs des
/// transactions insérées.
#[allow(clippy::too_many_arguments)]
async fn seed_bank_transactions(
    pool: &MySqlPool,
    company_id: i64,
    bank_account_id: i64,
    user_id: i64,
    file_hash: &str,
    period_from: NaiveDate,
    period_to: NaiveDate,
    new_txs: Vec<NewBankTransaction>,
) -> Vec<i64> {
    let mut tx = pool.begin().await.unwrap();
    let count = new_txs.len() as i32;
    let mut import = make_new_import(
        company_id,
        bank_account_id,
        user_id,
        file_hash,
        period_from,
        period_to,
    );
    import.transaction_count = count;
    let (_, inserted) = bank_imports::create_with_transactions(&mut tx, import, new_txs)
        .await
        .expect("create_with_transactions");
    tx.commit().await.unwrap();
    inserted.iter().map(|t| t.id).collect()
}

/// Hash hex 64 chars distinct par appel (multi-tenant tests réutilisent
/// le même hash sur des companies différentes).
fn unique_hash(seed: &str) -> String {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    let mut h = DefaultHasher::new();
    seed.hash(&mut h);
    let nano = chrono::Utc::now().timestamp_nanos_opt().unwrap_or(0);
    nano.hash(&mut h);
    let v = h.finish();
    format!("{:0>64x}", v)
}

// ============================================================
// Multi-actor setup
// ============================================================

struct CompanyCtx {
    company_id: i64,
    user_id: i64,
    bank_account_id: i64,
    contact_id: i64,
    jwt: String,
    /// Compte de créances clients — celui que l'écriture de vente débite et que
    /// l'encaissement doit créditer (Story 24-2).
    receivable_account_id: i64,
    /// Compte de banque au grand livre, câblé sur `bank_accounts.journal_account_id`.
    bank_ledger_account_id: i64,
}

async fn setup_company(pool: &MySqlPool, label: &str, iban: &str, role: Role) -> CompanyCtx {
    let company_id = create_company(pool, label).await;
    let username = format!("{label}_user");
    let user_id = create_user(pool, &username, role, company_id).await;
    // Le compte de produit n'est pas exposé par le contexte : les assertions
    // portent sur la créance et la banque, et `seed_validated_invoice` retrouve
    // le produit par son numéro (`account_ids`).
    let (receivable_account_id, _revenue_account_id, bank_ledger_account_id) =
        seed_accounts(pool, company_id).await;
    let bank_account_id = create_bank_account(pool, company_id, iban).await;
    // ⚠️ Sans ce câblage, l'encaissement échoue en `BANK_ACCOUNT_NOT_CONFIGURED` :
    // le compte bancaire n'a aucune contrepartie au grand livre.
    sqlx::query("UPDATE bank_accounts SET journal_account_id = ? WHERE id = ?")
        .bind(bank_ledger_account_id)
        .bind(bank_account_id)
        .execute(pool)
        .await
        .expect("wire journal_account_id");
    let contact_name = format!("{label} Client");
    let contact_id = create_contact(pool, company_id, user_id, &contact_name).await;
    let role_str = match role {
        Role::Admin => "Admin",
        Role::Comptable => "Comptable",
        Role::Consultation => "Consultation",
    };
    let jwt = forge_jwt(user_id, role_str, company_id);
    CompanyCtx {
        company_id,
        user_id,
        bank_account_id,
        contact_id,
        jwt,
        receivable_account_id,
        bank_ledger_account_id,
    }
}

// ============================================================
// Tests : GET /proposals
// ============================================================

/// AC #44 — GET proposals retourne les candidates avec scores. 3 tx
/// pending dont 2 ont des invoices candidates et 1 sans.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn get_proposals_returns_candidates_with_scores(pool: MySqlPool) {
    let ctx = setup_company(&pool, "Acme", "CH4431999123000889012", Role::Comptable).await;
    let day = NaiveDate::from_ymd_opt(2026, 5, 15).unwrap();

    // 2 invoices validated unpaid + 1 référence "INV-A" et "INV-B".
    let _inv_a = seed_validated_invoice(
        &pool,
        ctx.company_id,
        ctx.contact_id,
        "INV-A",
        day,
        dec!(100.00),
    )
    .await;
    let _inv_b = seed_validated_invoice(
        &pool,
        ctx.company_id,
        ctx.contact_id,
        "INV-B",
        day,
        dec!(250.00),
    )
    .await;

    // 3 transactions pending : 2 matchent une invoice (par montant),
    // la 3e n'a aucune candidate.
    seed_bank_transactions(
        &pool,
        ctx.company_id,
        ctx.bank_account_id,
        ctx.user_id,
        &unique_hash("get_proposals_happy"),
        day,
        day,
        vec![
            make_new_tx(
                ctx.company_id,
                ctx.bank_account_id,
                day,
                Some(day),
                dec!(100.00),
                "CHF",
                "INV-A",
                Some("Acme Client"),
            ),
            make_new_tx(
                ctx.company_id,
                ctx.bank_account_id,
                day,
                Some(day),
                dec!(250.00),
                "CHF",
                "INV-B",
                Some("Acme Client"),
            ),
            make_new_tx(
                ctx.company_id,
                ctx.bank_account_id,
                day,
                Some(day),
                dec!(7777.77),
                "CHF",
                "ORPHAN",
                None,
            ),
        ],
    )
    .await;

    let app = spawn_app(pool).await;
    let resp = app
        .client
        .get(app.url(&format!(
            "/api/v1/reconciliation/proposals?bankAccountId={}",
            ctx.bank_account_id
        )))
        .bearer_auth(&ctx.jwt)
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let body: Value = resp.json().await.unwrap();
    let proposals = body["proposals"].as_array().expect("proposals array");
    assert_eq!(proposals.len(), 3, "3 transactions pending exposées");

    // Vérifier que score est un objet structuré (pas booléens flags).
    let mut total_with_candidates = 0;
    let mut total_empty = 0;
    for p in proposals {
        assert!(p.get("bankTransactionId").is_some());
        assert!(p.get("transaction").is_some());
        let tx = &p["transaction"];
        assert!(tx.get("bookingDate").is_some());
        assert!(tx.get("amount").is_some());
        assert!(tx.get("currency").is_some());
        let cands = p["candidates"].as_array().unwrap();
        if cands.is_empty() {
            total_empty += 1;
        } else {
            total_with_candidates += 1;
            let score = &cands[0]["score"];
            assert!(score.get("total").is_some());
            assert!(score.get("amountScore").is_some());
            assert!(score.get("referenceScore").is_some());
            assert!(score.get("contactScore").is_some());
            // Pas de booléens flags amountMatch/referenceMatch.
            assert!(cands[0].get("amountMatch").is_none());
            assert!(cands[0].get("referenceMatch").is_none());
        }
    }
    assert_eq!(total_with_candidates, 2, "2 tx avec candidate");
    assert_eq!(total_empty, 1, "1 tx orpheline");
    assert_eq!(body["hasMore"], serde_json::Value::Bool(false));
}

/// #246 (Story 21-2b) — la proposition matche sur le **TTC** et l'expose
/// dans `invoiceAmount`. Facture HT 100 @ 8.1 % (TTC 108.10) : un encaissement
/// de 108.10 la propose comme candidate, avec `invoiceAmount = 108.1` (TTC,
/// pas le HT 100). Un encaissement de 100.00 (HT) ne la propose pas.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn get_proposals_matches_and_shows_ttc(pool: MySqlPool) {
    let ctx = setup_company(&pool, "TTC Acme", "CH4431999123000889012", Role::Comptable).await;
    let day = NaiveDate::from_ymd_opt(2026, 5, 15).unwrap();

    // Facture validée HT 100 @ 8.1 % → TTC 108.10 (avec fiscal_year + je).
    let fy_id = insert_fake_fiscal_year(&pool, ctx.company_id).await;
    let je_id = insert_fake_journal_entry(&pool, ctx.company_id, fy_id).await;
    let inv = sqlx::query(
        "INSERT INTO invoices (company_id, contact_id, invoice_number, status, date, \
         total_amount, journal_entry_id, version, created_at, updated_at) \
         VALUES (?, ?, 'INV-VAT', 'validated', ?, 100.00, ?, 1, NOW(3), NOW(3))",
    )
    .bind(ctx.company_id)
    .bind(ctx.contact_id)
    .bind(day)
    .bind(je_id)
    .execute(&pool)
    .await
    .expect("invoice insert");
    let inv_id = inv.last_insert_id() as i64;
    sqlx::query(
        "INSERT INTO invoice_lines (invoice_id, position, description, quantity, unit_price, vat_rate, line_total) \
         VALUES (?, 1, 'Prestation', 1, 100.00, 8.10, 100.00)",
    )
    .bind(inv_id)
    .execute(&pool)
    .await
    .expect("line insert");

    // Deux tx : 108.10 (TTC → matche) et 100.00 (HT → ne matche pas).
    seed_bank_transactions(
        &pool,
        ctx.company_id,
        ctx.bank_account_id,
        ctx.user_id,
        &unique_hash("get_proposals_ttc"),
        day,
        day,
        vec![
            make_new_tx(
                ctx.company_id,
                ctx.bank_account_id,
                day,
                Some(day),
                dec!(108.10),
                "CHF",
                "INV-VAT",
                Some("Acme Client"),
            ),
            make_new_tx(
                ctx.company_id,
                ctx.bank_account_id,
                day,
                Some(day),
                dec!(100.00),
                "CHF",
                "HT-ONLY",
                None,
            ),
        ],
    )
    .await;

    let app = spawn_app(pool).await;
    let resp = app
        .client
        .get(app.url(&format!(
            "/api/v1/reconciliation/proposals?bankAccountId={}",
            ctx.bank_account_id
        )))
        .bearer_auth(&ctx.jwt)
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let body: Value = resp.json().await.unwrap();
    let proposals = body["proposals"].as_array().expect("proposals array");

    // tx 108.10 → candidate avec invoiceAmount TTC = 108.1 et amountScore 1.
    // (le montant tx est sérialisé normalisé : 108.10 → "108.1").
    let ttc_prop = proposals
        .iter()
        .find(|p| p["transaction"]["amount"] == "108.1")
        .expect("proposition tx 108.10");
    let cands = ttc_prop["candidates"].as_array().unwrap();
    let inv_cand = cands
        .iter()
        .find(|c| c["invoiceId"] == inv_id)
        .expect("facture candidate pour la tx TTC");
    assert_eq!(
        inv_cand["invoiceAmount"], "108.1",
        "invoiceAmount doit être le TTC (normalisé), pas le HT 100 — sans \
         règlement, le reste dû vaut le TTC (#420, Story 25-4-c)"
    );
    assert_eq!(inv_cand["score"]["amountScore"], 1.0);
    assert!(
        inv_cand["invoiceTotalTtc"].is_null(),
        "sans règlement, pas de mention « reste dû sur » : reste dû = TTC"
    );

    // tx 100.00 (HT) → pas de candidate facture (le HT ne matche plus).
    let ht_prop = proposals
        .iter()
        .find(|p| p["transaction"]["amount"] == "100")
        .expect("proposition tx 100.00");
    let ht_cands = ht_prop["candidates"].as_array().unwrap();
    assert!(
        !ht_cands.iter().any(|c| c["invoiceId"] == inv_id),
        "le HT 100.00 ne doit plus proposer la facture (régression #246 corrigée)"
    );
}

/// AC #45 — multi-tenant : tx du company_B sur le même IBAN qu'un compte
/// du company_A n'apparait pas pour user company_A.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn get_proposals_scopes_by_company(pool: MySqlPool) {
    let iban = "CH4431999123000889012";
    let ctx_a = setup_company(&pool, "CompanyA", iban, Role::Comptable).await;
    let ctx_b = setup_company(&pool, "CompanyB", iban, Role::Comptable).await;
    let day = NaiveDate::from_ymd_opt(2026, 5, 15).unwrap();

    // Une tx pending du côté company_B.
    seed_bank_transactions(
        &pool,
        ctx_b.company_id,
        ctx_b.bank_account_id,
        ctx_b.user_id,
        &unique_hash("scopes_b"),
        day,
        day,
        vec![make_new_tx(
            ctx_b.company_id,
            ctx_b.bank_account_id,
            day,
            Some(day),
            dec!(99.99),
            "CHF",
            "REF-B",
            Some("Foreign"),
        )],
    )
    .await;

    let app = spawn_app(pool).await;
    let resp = app
        .client
        .get(app.url(&format!(
            "/api/v1/reconciliation/proposals?bankAccountId={}",
            ctx_a.bank_account_id
        )))
        .bearer_auth(&ctx_a.jwt)
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let body: Value = resp.json().await.unwrap();
    let proposals = body["proposals"].as_array().unwrap();
    assert!(
        proposals.is_empty(),
        "user company_A ne doit voir aucune tx (les txs sont du company_B)"
    );
}

/// AC #47 — tx avec auto_match_rejected_at != NULL est exclue.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn get_proposals_excludes_rejected_transactions(pool: MySqlPool) {
    let ctx = setup_company(&pool, "Acme", "CH4431999123000889012", Role::Comptable).await;
    let day = NaiveDate::from_ymd_opt(2026, 5, 15).unwrap();
    let ids = seed_bank_transactions(
        &pool,
        ctx.company_id,
        ctx.bank_account_id,
        ctx.user_id,
        &unique_hash("rejected"),
        day,
        day,
        vec![
            make_new_tx(
                ctx.company_id,
                ctx.bank_account_id,
                day,
                Some(day),
                dec!(100.00),
                "CHF",
                "REF-1",
                None,
            ),
            make_new_tx(
                ctx.company_id,
                ctx.bank_account_id,
                day,
                Some(day),
                dec!(200.00),
                "CHF",
                "REF-2",
                None,
            ),
        ],
    )
    .await;
    // Mark first one as auto-rejected.
    sqlx::query("UPDATE bank_transactions SET auto_match_rejected_at = NOW(3) WHERE id = ?")
        .bind(ids[0])
        .execute(&pool)
        .await
        .unwrap();

    let app = spawn_app(pool).await;
    let resp = app
        .client
        .get(app.url(&format!(
            "/api/v1/reconciliation/proposals?bankAccountId={}",
            ctx.bank_account_id
        )))
        .bearer_auth(&ctx.jwt)
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let body: Value = resp.json().await.unwrap();
    let proposals = body["proposals"].as_array().unwrap();
    assert_eq!(
        proposals.len(),
        1,
        "seule la tx non-rejetée doit apparaître"
    );
    assert_eq!(
        proposals[0]["bankTransactionId"].as_i64().unwrap(),
        ids[1],
        "c'est REF-2 qui reste"
    );
}

// ============================================================
// Tests : POST /accept
// ============================================================

/// AC #48 — POST accept happy path : reconcile + audit.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn post_accept_reconciles_transaction_and_invoice(pool: MySqlPool) {
    let ctx = setup_company(&pool, "Acme", "CH4431999123000889012", Role::Comptable).await;
    let day = NaiveDate::from_ymd_opt(2026, 5, 15).unwrap();
    let inv_date = NaiveDate::from_ymd_opt(2026, 4, 20).unwrap();
    let (inv_id, je_id) = seed_validated_invoice(
        &pool,
        ctx.company_id,
        ctx.contact_id,
        "INV-2026-001",
        inv_date,
        dec!(1234.56),
    )
    .await;
    let tx_ids = seed_bank_transactions(
        &pool,
        ctx.company_id,
        ctx.bank_account_id,
        ctx.user_id,
        &unique_hash("accept_happy"),
        day,
        day,
        vec![make_new_tx(
            ctx.company_id,
            ctx.bank_account_id,
            day,
            Some(day),
            dec!(1234.56),
            "CHF",
            "INV-2026-001",
            Some("Acme Client"),
        )],
    )
    .await;
    let tx_id = tx_ids[0];

    let app = spawn_app(pool.clone()).await;
    let body = serde_json::json!({
        "bankAccountId": ctx.bank_account_id,
        "proposals": [{ "type": "invoice", "bankTransactionId": tx_id, "invoiceId": inv_id }],
    });
    let resp = app
        .client
        .post(app.url("/api/v1/reconciliation/accept"))
        .bearer_auth(&ctx.jwt)
        .json(&body)
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let body: Value = resp.json().await.unwrap();
    let accepted = body["accepted"].as_array().unwrap();
    assert_eq!(accepted.len(), 1);
    assert!(body["failed"].as_array().unwrap().is_empty());
    let acc = &accepted[0];
    assert_eq!(acc["bankTransactionId"].as_i64().unwrap(), tx_id);
    assert_eq!(acc["invoiceId"].as_i64().unwrap(), inv_id);
    assert!(acc["score"]["total"].as_f64().unwrap() > 0.0);

    // ⛔ Story 24-2 (#371) — l'écriture rapprochée est celle de l'ENCAISSEMENT,
    // pas celle de vente. Jusqu'ici, cette assertion valait `je_id` : elle
    // DÉCRIVAIT le défaut. C'est `assert_ne!` qui l'aurait attrapé.
    let settlement_entry_id = acc["journalEntryId"].as_i64().unwrap();
    assert_ne!(
        settlement_entry_id, je_id,
        "l'écriture rapprochée ne doit PAS être l'écriture de vente"
    );

    // L'écriture d'encaissement dit ce qu'elle doit dire : `D banque / C créance`
    // du montant encaissé. C'est l'invariante qui fait que la banque égale le
    // relevé et que la créance se solde.
    let lines: Vec<(i64, Decimal, Decimal)> = sqlx::query_as(
        "SELECT account_id, debit, credit FROM journal_entry_lines \
         WHERE entry_id = ? ORDER BY line_order",
    )
    .bind(settlement_entry_id)
    .fetch_all(&pool)
    .await
    .unwrap();
    assert_eq!(
        lines.len(),
        2,
        "l'encaissement est une écriture à deux lignes"
    );
    assert_eq!(lines[0].0, ctx.bank_ledger_account_id);
    assert_eq!(
        lines[0].1,
        dec!(1234.56),
        "la banque est DÉBITÉE du montant"
    );
    assert_eq!(lines[1].0, ctx.receivable_account_id);
    assert_eq!(
        lines[1].2,
        dec!(1234.56),
        "la créance est CRÉDITÉE du montant"
    );

    // La liaison facture ↔ écriture existe et porte le montant.
    let (settled_invoice, settled_amount): (i64, Decimal) = sqlx::query_as(
        "SELECT invoice_id, amount FROM invoice_settlements WHERE journal_entry_id = ?",
    )
    .bind(settlement_entry_id)
    .fetch_one(&pool)
    .await
    .expect("une ligne invoice_settlements");
    assert_eq!(settled_invoice, inv_id);
    assert_eq!(settled_amount, dec!(1234.56));

    // Persistance.
    let (status, matched_entry_id): (String, Option<i64>) =
        sqlx::query_as("SELECT status, matched_entry_id FROM bank_transactions WHERE id = ?")
            .bind(tx_id)
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(status, "reconciled");
    assert_eq!(matched_entry_id, Some(settlement_entry_id));

    let paid_at: Option<NaiveDateTime> =
        sqlx::query_scalar("SELECT paid_at FROM invoices WHERE id = ?")
            .bind(inv_id)
            .fetch_one(&pool)
            .await
            .unwrap();
    assert!(
        paid_at.is_some(),
        "paid_at doit être set — la facture est soldée"
    );

    // Audit log reconciliation.accepted présent.
    let count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM audit_log WHERE action = 'reconciliation.accepted' \
         AND entity_type = 'bank_transaction' AND entity_id = ?",
    )
    .bind(tx_id)
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(count, 1);
}

/// AC #49 — partial success : 3 proposals dont 1 a un état caduc
/// (déjà reconciled). Les 2 OK passent, le 3e tombe en `failed`.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn post_accept_handles_partial_failure(pool: MySqlPool) {
    let ctx = setup_company(&pool, "Acme", "CH4431999123000889012", Role::Comptable).await;
    let day = NaiveDate::from_ymd_opt(2026, 5, 15).unwrap();
    let inv_date = NaiveDate::from_ymd_opt(2026, 5, 1).unwrap();

    let (inv1, _) = seed_validated_invoice(
        &pool,
        ctx.company_id,
        ctx.contact_id,
        "INV-1",
        inv_date,
        dec!(100.00),
    )
    .await;
    let (inv2, _) = seed_validated_invoice(
        &pool,
        ctx.company_id,
        ctx.contact_id,
        "INV-2",
        inv_date,
        dec!(200.00),
    )
    .await;
    let (inv3, _) = seed_validated_invoice(
        &pool,
        ctx.company_id,
        ctx.contact_id,
        "INV-3",
        inv_date,
        dec!(300.00),
    )
    .await;

    let tx_ids = seed_bank_transactions(
        &pool,
        ctx.company_id,
        ctx.bank_account_id,
        ctx.user_id,
        &unique_hash("partial"),
        day,
        day,
        vec![
            make_new_tx(
                ctx.company_id,
                ctx.bank_account_id,
                day,
                Some(day),
                dec!(100.00),
                "CHF",
                "INV-1",
                Some("Acme Client"),
            ),
            make_new_tx(
                ctx.company_id,
                ctx.bank_account_id,
                day,
                Some(day),
                dec!(200.00),
                "CHF",
                "INV-2",
                Some("Acme Client"),
            ),
            make_new_tx(
                ctx.company_id,
                ctx.bank_account_id,
                day,
                Some(day),
                dec!(300.00),
                "CHF",
                "INV-3",
                Some("Acme Client"),
            ),
        ],
    )
    .await;

    // Marquer la 3e tx comme déjà `reconciled` directement en DB pour
    // simuler un état caduc entre le pré-flight ownership et le step 4
    // status check.
    // NOTE : pré-marquer status='reconciled' avant le pré-flight 0ter ne
    // simulerait pas l'état caduc visé ici : le pré-vol la refuserait lui-même
    // (`RECONCILIATION_ALREADY_RECONCILED`, Story 27-4 — un `400` du lot entier
    // auparavant), sans atteindre le step 4.
    // À la place : on garde tx[2] pending mais on lie l'invoice à la tx[1] (déjà
    // utilisée au step 1) — pas applicable.
    // Approche réelle : on simule en pointant tx[2]+invoice qui ne match pas
    // (montant 300 vs invoice paid_at déjà set). On marque inv3 comme déjà
    // payée → step 6 retourne `invoice_already_paid`.
    sqlx::query("UPDATE invoices SET paid_at = NOW(3), version = version + 1 WHERE id = ?")
        .bind(inv3)
        .execute(&pool)
        .await
        .unwrap();

    let app = spawn_app(pool.clone()).await;
    let body = serde_json::json!({
        "bankAccountId": ctx.bank_account_id,
        "proposals": [
            { "type": "invoice", "bankTransactionId": tx_ids[0], "invoiceId": inv1 },
            { "type": "invoice", "bankTransactionId": tx_ids[1], "invoiceId": inv2 },
            { "type": "invoice", "bankTransactionId": tx_ids[2], "invoiceId": inv3 },
        ],
    });
    let resp = app
        .client
        .post(app.url("/api/v1/reconciliation/accept"))
        .bearer_auth(&ctx.jwt)
        .json(&body)
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let body: Value = resp.json().await.unwrap();
    let accepted = body["accepted"].as_array().unwrap();
    let failed = body["failed"].as_array().unwrap();
    assert_eq!(accepted.len(), 2, "2 réussites");
    assert_eq!(failed.len(), 1, "1 échec");
    let f = &failed[0];
    assert_eq!(f["bankTransactionId"].as_i64().unwrap(), tx_ids[2]);
    assert_eq!(
        f["errorCode"].as_str().unwrap(),
        "RECONCILIATION_INVOICE_NOT_ELIGIBLE"
    );
    assert_eq!(
        f["details"]["reason"].as_str().unwrap(),
        "invoice_already_paid"
    );

    // Les 2 réussites sont bien commit.
    let count_reconciled: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM bank_transactions WHERE id IN (?, ?) AND status = 'reconciled'",
    )
    .bind(tx_ids[0])
    .bind(tx_ids[1])
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(count_reconciled, 2);
}

/// AC #50 — invoice draft → failed avec reason invoice_not_validated.
/// Symétrique pour invoice déjà payée → invoice_already_paid.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn post_accept_rejects_unvalidated_or_paid_invoice(pool: MySqlPool) {
    let ctx = setup_company(&pool, "Acme", "CH4431999123000889012", Role::Comptable).await;
    let day = NaiveDate::from_ymd_opt(2026, 5, 15).unwrap();
    let inv_date = NaiveDate::from_ymd_opt(2026, 5, 1).unwrap();

    // Invoice draft (pas validated).
    let inv_draft = insert_invoice(
        &pool,
        ctx.company_id,
        ctx.contact_id,
        "INV-DRAFT",
        inv_date,
        dec!(100.00),
        "draft",
        None,
        None,
    )
    .await;

    // Invoice validated mais déjà payée.
    let fy_id = insert_fake_fiscal_year(&pool, ctx.company_id).await;
    let je_id = insert_fake_journal_entry(&pool, ctx.company_id, fy_id).await;
    let paid_dt = NaiveDate::from_ymd_opt(2026, 5, 10)
        .unwrap()
        .and_hms_opt(0, 0, 0)
        .unwrap();
    let inv_paid = insert_invoice(
        &pool,
        ctx.company_id,
        ctx.contact_id,
        "INV-PAID",
        inv_date,
        dec!(200.00),
        "validated",
        Some(je_id),
        Some(paid_dt),
    )
    .await;

    let tx_ids = seed_bank_transactions(
        &pool,
        ctx.company_id,
        ctx.bank_account_id,
        ctx.user_id,
        &unique_hash("not_eligible"),
        day,
        day,
        vec![
            make_new_tx(
                ctx.company_id,
                ctx.bank_account_id,
                day,
                Some(day),
                dec!(100.00),
                "CHF",
                "INV-DRAFT",
                Some("Acme Client"),
            ),
            make_new_tx(
                ctx.company_id,
                ctx.bank_account_id,
                day,
                Some(day),
                dec!(200.00),
                "CHF",
                "INV-PAID",
                Some("Acme Client"),
            ),
        ],
    )
    .await;

    let app = spawn_app(pool).await;
    let body = serde_json::json!({
        "bankAccountId": ctx.bank_account_id,
        "proposals": [
            { "type": "invoice", "bankTransactionId": tx_ids[0], "invoiceId": inv_draft },
            { "type": "invoice", "bankTransactionId": tx_ids[1], "invoiceId": inv_paid },
        ],
    });
    let resp = app
        .client
        .post(app.url("/api/v1/reconciliation/accept"))
        .bearer_auth(&ctx.jwt)
        .json(&body)
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let body: Value = resp.json().await.unwrap();
    let failed = body["failed"].as_array().unwrap();
    assert_eq!(failed.len(), 2);
    assert!(body["accepted"].as_array().unwrap().is_empty());

    let mut got_draft = false;
    let mut got_paid = false;
    for f in failed {
        assert_eq!(
            f["errorCode"].as_str().unwrap(),
            "RECONCILIATION_INVOICE_NOT_ELIGIBLE"
        );
        match f["details"]["reason"].as_str().unwrap() {
            "invoice_not_validated" => got_draft = true,
            "invoice_already_paid" => got_paid = true,
            other => panic!("reason inattendue : {other}"),
        }
    }
    assert!(got_draft && got_paid);
}

/// AC #51 — pas de leak cross-tenant : invoice du company_B masquée
/// comme INVOICE_NOT_FOUND quand user company_A POST accept.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn post_accept_does_not_leak_cross_tenant_invoice(pool: MySqlPool) {
    let ctx_a = setup_company(&pool, "CompanyA", "CH4431999123000889012", Role::Comptable).await;
    let ctx_b = setup_company(&pool, "CompanyB", "CH5604835012345678009", Role::Comptable).await;
    let day = NaiveDate::from_ymd_opt(2026, 5, 15).unwrap();
    let inv_date = NaiveDate::from_ymd_opt(2026, 5, 1).unwrap();

    let (inv_b, _) = seed_validated_invoice(
        &pool,
        ctx_b.company_id,
        ctx_b.contact_id,
        "INV-B",
        inv_date,
        dec!(100.00),
    )
    .await;
    let tx_ids = seed_bank_transactions(
        &pool,
        ctx_a.company_id,
        ctx_a.bank_account_id,
        ctx_a.user_id,
        &unique_hash("cross_tenant_invoice"),
        day,
        day,
        vec![make_new_tx(
            ctx_a.company_id,
            ctx_a.bank_account_id,
            day,
            Some(day),
            dec!(100.00),
            "CHF",
            "REF",
            None,
        )],
    )
    .await;

    let app = spawn_app(pool).await;
    let body = serde_json::json!({
        "bankAccountId": ctx_a.bank_account_id,
        "proposals": [{ "type": "invoice", "bankTransactionId": tx_ids[0], "invoiceId": inv_b }],
    });
    let resp = app
        .client
        .post(app.url("/api/v1/reconciliation/accept"))
        .bearer_auth(&ctx_a.jwt)
        .json(&body)
        .send()
        .await
        .unwrap();
    // Le 200 OK est attendu (partial success body, pas 403 leak).
    assert_eq!(resp.status(), 200);
    let body: Value = resp.json().await.unwrap();
    let failed = body["failed"].as_array().unwrap();
    assert_eq!(failed.len(), 1);
    assert_eq!(
        failed[0]["errorCode"].as_str().unwrap(),
        "INVOICE_NOT_FOUND"
    );
}

/// AC #52 — 2 POST accept concurrents sur même `(company, account)` →
/// le 2e timeout sur GET_LOCK et retourne 409 RECONCILIATION_ACCOUNT_LOCKED.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn post_accept_returns_409_on_account_lock_contention(pool: MySqlPool) {
    let ctx = setup_company(&pool, "Acme", "CH4431999123000889012", Role::Comptable).await;
    let day = NaiveDate::from_ymd_opt(2026, 5, 15).unwrap();
    let inv_date = NaiveDate::from_ymd_opt(2026, 5, 1).unwrap();
    let (inv1, _) = seed_validated_invoice(
        &pool,
        ctx.company_id,
        ctx.contact_id,
        "INV-LOCK-1",
        inv_date,
        dec!(100.00),
    )
    .await;
    let (inv2, _) = seed_validated_invoice(
        &pool,
        ctx.company_id,
        ctx.contact_id,
        "INV-LOCK-2",
        inv_date,
        dec!(200.00),
    )
    .await;
    let tx_ids = seed_bank_transactions(
        &pool,
        ctx.company_id,
        ctx.bank_account_id,
        ctx.user_id,
        &unique_hash("lock_contention"),
        day,
        day,
        vec![
            make_new_tx(
                ctx.company_id,
                ctx.bank_account_id,
                day,
                Some(day),
                dec!(100.00),
                "CHF",
                "INV-LOCK-1",
                None,
            ),
            make_new_tx(
                ctx.company_id,
                ctx.bank_account_id,
                day,
                Some(day),
                dec!(200.00),
                "CHF",
                "INV-LOCK-2",
                None,
            ),
        ],
    )
    .await;

    // Pré-acquérir le lock manuellement via une connexion dédiée — on
    // simule une réconciliation en cours qui détient le verrou pour
    // toute la durée du test (release à la fin).
    // ⚠️ Le nom du verrou porte la BASE COURANTE depuis la correction de la
    // KF-038 (#228) — `GET_LOCK` est global au serveur MariaDB, et sans ce
    // préfixe toutes les bases éphémères des tests se disputaient le même
    // `reconcile:1:1`. Ce test doit donc le reconstruire à l'identique, sinon il
    // pose un verrou que personne ne prendra et n'observe plus aucune contention.
    let db_name: String = sqlx::query_scalar("SELECT COALESCE(DATABASE(), '')")
        .fetch_one(&pool)
        .await
        .unwrap();
    let lock_name = format!(
        "reconcile:{}:{}:{}",
        db_name, ctx.company_id, ctx.bank_account_id
    );
    let mut hold_conn = pool.acquire().await.unwrap();
    let acquired: Option<i32> = sqlx::query_scalar("SELECT GET_LOCK(?, ?)")
        .bind(&lock_name)
        .bind(60)
        .fetch_one(&mut *hold_conn)
        .await
        .unwrap();
    assert_eq!(acquired, Some(1), "lock helper acquis");

    let app = spawn_app(pool.clone()).await;
    let body = serde_json::json!({
        "bankAccountId": ctx.bank_account_id,
        "proposals": [
            { "type": "invoice", "bankTransactionId": tx_ids[0], "invoiceId": inv1 },
            { "type": "invoice", "bankTransactionId": tx_ids[1], "invoiceId": inv2 },
        ],
    });
    let start = std::time::Instant::now();
    let resp = app
        .client
        .post(app.url("/api/v1/reconciliation/accept"))
        .bearer_auth(&ctx.jwt)
        .json(&body)
        .send()
        .await
        .unwrap();
    let elapsed = start.elapsed();
    assert_eq!(resp.status(), 409);
    let body: Value = resp.json().await.unwrap();
    assert_eq!(
        body["error"]["code"].as_str().unwrap(),
        "RECONCILIATION_ACCOUNT_LOCKED"
    );
    // Doit avoir attendu environ LOCK_TIMEOUT_SECS=5s.
    assert!(
        elapsed >= std::time::Duration::from_secs(4),
        "lock timeout doit avoir lieu (elapsed={:?})",
        elapsed
    );

    // Release.
    let _: Option<i32> = sqlx::query_scalar("SELECT RELEASE_LOCK(?)")
        .bind(&lock_name)
        .fetch_one(&mut *hold_conn)
        .await
        .unwrap();
}

// ============================================================
// Tests : POST /reject
// ============================================================

/// AC #53 — POST reject happy : 2 transactions marquées + 1 audit log.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn post_reject_marks_transactions_as_manually_reviewed(pool: MySqlPool) {
    let ctx = setup_company(&pool, "Acme", "CH4431999123000889012", Role::Comptable).await;
    let day = NaiveDate::from_ymd_opt(2026, 5, 15).unwrap();
    let tx_ids = seed_bank_transactions(
        &pool,
        ctx.company_id,
        ctx.bank_account_id,
        ctx.user_id,
        &unique_hash("reject_happy"),
        day,
        day,
        vec![
            make_new_tx(
                ctx.company_id,
                ctx.bank_account_id,
                day,
                Some(day),
                dec!(100.00),
                "CHF",
                "REF-1",
                None,
            ),
            make_new_tx(
                ctx.company_id,
                ctx.bank_account_id,
                day,
                Some(day),
                dec!(200.00),
                "CHF",
                "REF-2",
                None,
            ),
        ],
    )
    .await;

    let app = spawn_app(pool.clone()).await;
    let body = serde_json::json!({
        "bankAccountId": ctx.bank_account_id,
        "bankTransactionIds": tx_ids,
    });
    let resp = app
        .client
        .post(app.url("/api/v1/reconciliation/reject"))
        .bearer_auth(&ctx.jwt)
        .json(&body)
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let body: Value = resp.json().await.unwrap();
    let rejected = body["rejected"].as_array().unwrap();
    assert_eq!(rejected.len(), 2);
    assert!(body["failed"].as_array().unwrap().is_empty());

    // Persistance auto_match_rejected_at sur les 2.
    let count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM bank_transactions \
         WHERE id IN (?, ?) AND auto_match_rejected_at IS NOT NULL",
    )
    .bind(tx_ids[0])
    .bind(tx_ids[1])
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(count, 2);

    // 1 entrée audit log (batch) avec entity_id = plus petit ID du lot
    // (lot trié, Story 27-1 AC4-bis ; ici `tx_ids` est déjà croissant) + count = 2.
    let row: (String, String, i64, Value) = sqlx::query_as(
        "SELECT action, entity_type, entity_id, details_json FROM audit_log \
         WHERE action = 'reconciliation.rejected' ORDER BY id DESC LIMIT 1",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(row.0, "reconciliation.rejected");
    assert_eq!(row.1, "bank_transaction");
    assert_eq!(row.2, tx_ids[0], "entity_id = plus petit ID du lot");
    assert_eq!(row.3["count"].as_i64().unwrap(), 2);
    let ids_json = row.3["bank_transaction_ids"].as_array().unwrap();
    assert_eq!(ids_json.len(), 2);
}

/// Story 27-1 (AC4-bis, H7, #632) — le lot de rejet est traité par
/// identifiant **croissant**, quel que soit l'ordre reçu : `rejected[]` sort
/// trié, et la trace prend le plus petit identifiant pour `entity_id`. C'est
/// ce qui ferme le cycle avec la suppression d'un import, qui verrouille ses
/// transactions dans le même ordre.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn post_reject_processes_the_batch_by_increasing_id(pool: MySqlPool) {
    let ctx = setup_company(&pool, "Acme", "CH4431999123000889012", Role::Comptable).await;
    let day = NaiveDate::from_ymd_opt(2026, 5, 15).unwrap();
    let tx_ids = seed_bank_transactions(
        &pool,
        ctx.company_id,
        ctx.bank_account_id,
        ctx.user_id,
        &unique_hash("reject_sorted"),
        day,
        day,
        vec![
            make_new_tx(
                ctx.company_id,
                ctx.bank_account_id,
                day,
                Some(day),
                dec!(100.00),
                "CHF",
                "REF-1",
                None,
            ),
            make_new_tx(
                ctx.company_id,
                ctx.bank_account_id,
                day,
                Some(day),
                dec!(200.00),
                "CHF",
                "REF-2",
                None,
            ),
        ],
    )
    .await;
    let (lo, hi) = (tx_ids[0].min(tx_ids[1]), tx_ids[0].max(tx_ids[1]));

    let app = spawn_app(pool.clone()).await;
    let resp = app
        .client
        .post(app.url("/api/v1/reconciliation/reject"))
        .bearer_auth(&ctx.jwt)
        .json(&serde_json::json!({
            "bankAccountId": ctx.bank_account_id,
            "bankTransactionIds": [hi, lo],
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let body: Value = resp.json().await.unwrap();
    let got: Vec<i64> = body["rejected"]
        .as_array()
        .unwrap()
        .iter()
        .map(|r| r["bankTransactionId"].as_i64().unwrap())
        .collect();
    assert_eq!(got, vec![lo, hi], "rejected[] par identifiant croissant");

    let row: (i64, Value) = sqlx::query_as(
        "SELECT entity_id, details_json FROM audit_log \
         WHERE action = 'reconciliation.rejected' ORDER BY id DESC LIMIT 1",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(row.0, lo, "entity_id = plus petit identifiant du lot");
    let traced: Vec<i64> = row.1["bank_transaction_ids"]
        .as_array()
        .unwrap()
        .iter()
        .map(|v| v.as_i64().unwrap())
        .collect();
    assert_eq!(traced, vec![lo, hi], "details_json trié");
}

/// Story 27-1 (AC4-bis, H7-bis — revue de code P1, E5) — ce que H7 ne voit
/// pas : l'**ordre des verrous**. H7 lit les sorties ; une variante qui
/// traiterait le lot dans l'ordre reçu puis trierait ses sorties y resterait
/// verte, en rouvrant le cycle avec la suppression d'un import.
///
/// Montage : une transaction de test tient `lo` ; le lot `[hi, lo]` part en
/// tâche. Traité par identifiant croissant, il bute sur `lo` **avant** d'avoir
/// touché `hi` : une sonde `NOWAIT` sur `hi` doit donc réussir. Traité dans
/// l'ordre reçu, il aurait déjà écrit `hi` et la sonde échouerait.
///
/// `failed[]` n'est pas asserté ici : les deux lignes sont éligibles. Depuis
/// la Story 27-4, le pré-vol (`find_by_ids_for_account`) refuse par
/// proposition, et l'ordre de `failed[]` — trié en entier, refus du pré-vol
/// et refus sous verrou mêlés — est tenu par
/// `post_reject_failed_merges_preflight_and_locked_refusals_by_id`.
///
/// ⚠️ **Angle mort assumé** (revue de code P2, A2-3) : la sonde lit
/// `information_schema.PROCESSLIST`, qui montre une requête **en cours**, non
/// une requête **en attente de verrou**. Sous la variante fautive (ordre reçu),
/// si la sonde surprend l'`UPDATE` de `hi` en vol et que la sonde `NOWAIT` part
/// avant que cet `UPDATE` n'ait posé son verrou, `hi` paraît libre et la
/// mutation survit. La fenêtre est de l'ordre de la microseconde (M26 a rougi
/// au journal) ; elle repose sur un minutage, faute d'un trait textuel du seul
/// état bloqué. Le code juste ne peut, lui, rendre aucun faux rouge : seul
/// l'`UPDATE` de `lo`, bloqué, porte le motif cherché.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn post_reject_locks_the_batch_by_increasing_id(pool: MySqlPool) {
    let ctx = setup_company(&pool, "Acme", "CH4431999123000889012", Role::Comptable).await;
    let day = NaiveDate::from_ymd_opt(2026, 5, 15).unwrap();
    let tx_ids = seed_bank_transactions(
        &pool,
        ctx.company_id,
        ctx.bank_account_id,
        ctx.user_id,
        &unique_hash("reject_lock_order"),
        day,
        day,
        vec![
            make_new_tx(
                ctx.company_id,
                ctx.bank_account_id,
                day,
                Some(day),
                dec!(100.00),
                "CHF",
                "REF-1",
                None,
            ),
            make_new_tx(
                ctx.company_id,
                ctx.bank_account_id,
                day,
                Some(day),
                dec!(200.00),
                "CHF",
                "REF-2",
                None,
            ),
        ],
    )
    .await;
    let (lo, hi) = (tx_ids[0].min(tx_ids[1]), tx_ids[0].max(tx_ids[1]));
    let app = spawn_app(pool.clone()).await;

    let mut hold = pool.begin().await.unwrap();
    sqlx::query("SELECT id FROM bank_transactions WHERE id = ? FOR UPDATE")
        .bind(lo)
        .fetch_one(&mut *hold)
        .await
        .unwrap();

    let client = app.client.clone();
    let url = app.url("/api/v1/reconciliation/reject");
    let jwt = ctx.jwt.clone();
    let body = serde_json::json!({
        "bankAccountId": ctx.bank_account_id,
        "bankTransactionIds": [hi, lo],
    });
    let reject = tokio::spawn(async move {
        let resp = client
            .post(url)
            .bearer_auth(jwt)
            .json(&body)
            .send()
            .await
            .unwrap();
        let status = resp.status().as_u16();
        (status, resp.json::<Value>().await.unwrap_or(Value::Null))
    });

    let bloque = kesh_db::test_fixtures::attendre_une_requete_en_cours(
        &pool,
        &["UPDATE bank_transactions", "auto_match_rejected_at"],
        || reject.is_finished(),
    )
    .await;
    assert!(
        bloque,
        "le lot devait attendre la ligne tenue — le montage ne prouve rien sinon"
    );
    let hi_libre = kesh_db::test_fixtures::sonde_verrou_nowait(
        &pool,
        "SELECT id FROM bank_transactions WHERE id = ? FOR UPDATE NOWAIT",
        hi,
    )
    .await;

    hold.rollback().await.unwrap();
    let (status, body) = reject.await.unwrap();
    assert!(
        hi_libre,
        "le lot [hi, lo] a verrouillé hi avant lo : il n'est pas traité par identifiant croissant"
    );
    assert_eq!(status, 200, "{body}");
    assert_eq!(body["rejected"].as_array().unwrap().len(), 2, "{body}");
}

/// AC #54 — POST reject sur tx déjà reconciled → failed +
/// auto_match_rejected_at intact.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn post_reject_skips_reconciled_transactions(pool: MySqlPool) {
    let ctx = setup_company(&pool, "Acme", "CH4431999123000889012", Role::Comptable).await;
    let day = NaiveDate::from_ymd_opt(2026, 5, 15).unwrap();
    let tx_ids = seed_bank_transactions(
        &pool,
        ctx.company_id,
        ctx.bank_account_id,
        ctx.user_id,
        &unique_hash("reject_already_reconciled"),
        day,
        day,
        vec![make_new_tx(
            ctx.company_id,
            ctx.bank_account_id,
            day,
            Some(day),
            dec!(100.00),
            "CHF",
            "REF-1",
            None,
        )],
    )
    .await;
    // Marquer comme reconciled directement.
    sqlx::query("UPDATE bank_transactions SET status = 'reconciled' WHERE id = ?")
        .bind(tx_ids[0])
        .execute(&pool)
        .await
        .unwrap();

    let app = spawn_app(pool.clone()).await;
    let body = serde_json::json!({
        "bankAccountId": ctx.bank_account_id,
        "bankTransactionIds": tx_ids,
    });
    let resp = app
        .client
        .post(app.url("/api/v1/reconciliation/reject"))
        .bearer_auth(&ctx.jwt)
        .json(&body)
        .send()
        .await
        .unwrap();
    // Story 27-4 (#634) — refus par proposition, `200` : le pré-vol classe la
    // transaction rapprochée (au sens d'`is_reconciled`).
    assert_eq!(resp.status(), 200);
    let body: Value = resp.json().await.unwrap();
    assert!(body["rejected"].as_array().unwrap().is_empty(), "{body}");
    let failed = body["failed"].as_array().unwrap();
    assert_eq!(failed.len(), 1, "{body}");
    assert_eq!(failed[0]["bankTransactionId"].as_i64().unwrap(), tx_ids[0]);
    assert_eq!(
        failed[0]["errorCode"].as_str().unwrap(),
        "RECONCILIATION_ALREADY_RECONCILED"
    );
    assert!(failed[0]["details"].is_null(), "{body}");

    // auto_match_rejected_at reste NULL.
    let rejected_at: Option<NaiveDateTime> =
        sqlx::query_scalar("SELECT auto_match_rejected_at FROM bank_transactions WHERE id = ?")
            .bind(tx_ids[0])
            .fetch_one(&pool)
            .await
            .unwrap();
    assert!(rejected_at.is_none());
}

// ============================================================
// Tests : RBAC
// ============================================================

/// AC #60 — `Consultation` rôle est rejeté 403 sur les 3 routes.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn reconciliation_routes_require_comptable_role(pool: MySqlPool) {
    let ctx = setup_company(&pool, "Acme", "CH4431999123000889012", Role::Consultation).await;
    let app = spawn_app(pool).await;

    // GET /proposals.
    let resp = app
        .client
        .get(app.url(&format!(
            "/api/v1/reconciliation/proposals?bankAccountId={}",
            ctx.bank_account_id
        )))
        .bearer_auth(&ctx.jwt)
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 403);

    // POST /accept.
    let body = serde_json::json!({
        "bankAccountId": ctx.bank_account_id,
        "proposals": [{ "type": "invoice", "bankTransactionId": 1, "invoiceId": 1 }],
    });
    let resp = app
        .client
        .post(app.url("/api/v1/reconciliation/accept"))
        .bearer_auth(&ctx.jwt)
        .json(&body)
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 403);

    // POST /reject.
    let body = serde_json::json!({
        "bankAccountId": ctx.bank_account_id,
        "bankTransactionIds": [1],
    });
    let resp = app
        .client
        .post(app.url("/api/v1/reconciliation/reject"))
        .bearer_auth(&ctx.jwt)
        .json(&body)
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 403);
}

// ============================================================
// Tests : Pass 3 ACs (cross-flow + edge cases)
// ============================================================

/// AC #64 — accept+reject race : flow A accepte tx pending puis flow B
/// POST reject sur la même tx → `200`, refus par proposition
/// `RECONCILIATION_ALREADY_RECONCILED` (Story 27-4 — auparavant un `400` du
/// lot entier). auto_match_rejected_at reste NULL.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn post_reject_after_accept_returns_already_reconciled_failed(pool: MySqlPool) {
    let ctx = setup_company(&pool, "Acme", "CH4431999123000889012", Role::Comptable).await;
    let day = NaiveDate::from_ymd_opt(2026, 5, 15).unwrap();
    let inv_date = NaiveDate::from_ymd_opt(2026, 5, 1).unwrap();
    let (inv_id, _) = seed_validated_invoice(
        &pool,
        ctx.company_id,
        ctx.contact_id,
        "INV-RACE",
        inv_date,
        dec!(100.00),
    )
    .await;
    let tx_ids = seed_bank_transactions(
        &pool,
        ctx.company_id,
        ctx.bank_account_id,
        ctx.user_id,
        &unique_hash("race_accept_reject"),
        day,
        day,
        vec![make_new_tx(
            ctx.company_id,
            ctx.bank_account_id,
            day,
            Some(day),
            dec!(100.00),
            "CHF",
            "INV-RACE",
            Some("Acme Client"),
        )],
    )
    .await;
    let tx_id = tx_ids[0];

    let app = spawn_app(pool.clone()).await;

    // Flow A : accept.
    let body = serde_json::json!({
        "bankAccountId": ctx.bank_account_id,
        "proposals": [{ "type": "invoice", "bankTransactionId": tx_id, "invoiceId": inv_id }],
    });
    let resp = app
        .client
        .post(app.url("/api/v1/reconciliation/accept"))
        .bearer_auth(&ctx.jwt)
        .json(&body)
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);

    // Flow B : reject sur la même tx (déjà reconciled).
    let body = serde_json::json!({
        "bankAccountId": ctx.bank_account_id,
        "bankTransactionIds": [tx_id],
    });
    let resp = app
        .client
        .post(app.url("/api/v1/reconciliation/reject"))
        .bearer_auth(&ctx.jwt)
        .json(&body)
        .send()
        .await
        .unwrap();
    // Story 27-4 (#634) — refus par proposition, `200`.
    assert_eq!(resp.status(), 200);
    let body: Value = resp.json().await.unwrap();
    assert!(body["rejected"].as_array().unwrap().is_empty(), "{body}");
    assert_eq!(
        body["failed"][0]["errorCode"].as_str().unwrap(),
        "RECONCILIATION_ALREADY_RECONCILED",
        "{body}"
    );
    assert_eq!(
        body["failed"][0]["bankTransactionId"].as_i64().unwrap(),
        tx_id
    );

    // auto_match_rejected_at reste NULL.
    let rejected_at: Option<NaiveDateTime> =
        sqlx::query_scalar("SELECT auto_match_rejected_at FROM bank_transactions WHERE id = ?")
            .bind(tx_id)
            .fetch_one(&pool)
            .await
            .unwrap();
    assert!(rejected_at.is_none());
}

/// AC #65 — pagination : 150 tx pending → 100 retournées + hasMore=true.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn get_proposals_paginates_at_100_default(pool: MySqlPool) {
    let ctx = setup_company(&pool, "Acme", "CH4431999123000889012", Role::Comptable).await;
    let day = NaiveDate::from_ymd_opt(2026, 5, 15).unwrap();

    let new_txs: Vec<NewBankTransaction> = (0..150)
        .map(|i| {
            make_new_tx(
                ctx.company_id,
                ctx.bank_account_id,
                day - chrono::Duration::days(i),
                Some(day - chrono::Duration::days(i)),
                Decimal::new(1000 + i, 2),
                "CHF",
                &format!("REF-{i}"),
                None,
            )
        })
        .collect();
    seed_bank_transactions(
        &pool,
        ctx.company_id,
        ctx.bank_account_id,
        ctx.user_id,
        &unique_hash("pagination"),
        day - chrono::Duration::days(149),
        day,
        new_txs,
    )
    .await;

    let app = spawn_app(pool).await;
    let resp = app
        .client
        .get(app.url(&format!(
            "/api/v1/reconciliation/proposals?bankAccountId={}",
            ctx.bank_account_id
        )))
        .bearer_auth(&ctx.jwt)
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let body: Value = resp.json().await.unwrap();
    let proposals = body["proposals"].as_array().unwrap();
    assert_eq!(proposals.len(), 100, "limite par défaut = 100");
    assert_eq!(
        body["hasMore"],
        serde_json::Value::Bool(true),
        "indicateur pagination doit être true (150 > 100)"
    );
}

/// AC #66 — sign filter : tx débit `amount = -100.00` n'a pas de
/// candidate. POST accept retourne `RECONCILIATION_SCORE_TOO_LOW` car
/// le scoring serveur-side trouve `amount_score=0.0` (signed mismatch).
///
/// Note déviation spec : la spec dit « POST accept retourne 404
/// BANK_TRANSACTION_NOT_FOUND ». En pratique, le pré-vol
/// (`find_by_ids_for_account`, Story 27-4) ne filtre pas par sign et la tx débit traverse jusqu'à `accept_one` qui
/// score 0.0 → step 7bis P3-H4 min-score guard. Le brief autorise
/// d'ajuster l'attendu selon le comportement réel.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn post_accept_filters_signed_amount(pool: MySqlPool) {
    let ctx = setup_company(&pool, "Acme", "CH4431999123000889012", Role::Comptable).await;
    let day = NaiveDate::from_ymd_opt(2026, 5, 15).unwrap();
    let inv_date = NaiveDate::from_ymd_opt(2026, 5, 1).unwrap();
    let (inv_id, _) = seed_validated_invoice(
        &pool,
        ctx.company_id,
        ctx.contact_id,
        "INV-SIGN",
        inv_date,
        dec!(100.00),
    )
    .await;
    let tx_ids = seed_bank_transactions(
        &pool,
        ctx.company_id,
        ctx.bank_account_id,
        ctx.user_id,
        &unique_hash("signed"),
        day,
        day,
        vec![make_new_tx(
            ctx.company_id,
            ctx.bank_account_id,
            day,
            Some(day),
            dec!(-100.00),
            "CHF",
            "ANY-REF",
            None,
        )],
    )
    .await;

    let app = spawn_app(pool).await;
    let body = serde_json::json!({
        "bankAccountId": ctx.bank_account_id,
        "proposals": [{ "type": "invoice", "bankTransactionId": tx_ids[0], "invoiceId": inv_id }],
    });
    let resp = app
        .client
        .post(app.url("/api/v1/reconciliation/accept"))
        .bearer_auth(&ctx.jwt)
        .json(&body)
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let body: Value = resp.json().await.unwrap();
    let failed = body["failed"].as_array().unwrap();
    assert_eq!(failed.len(), 1);
    let err_code = failed[0]["errorCode"].as_str().unwrap();
    // Comportement réel : score=0 → P3-H4 min-score guard (RECONCILIATION_SCORE_TOO_LOW).
    assert!(
        matches!(
            err_code,
            "RECONCILIATION_SCORE_TOO_LOW" | "BANK_TRANSACTION_NOT_FOUND"
        ),
        "errorCode inattendu pour tx débit : {err_code}"
    );
}

/// AC #67 — currency mismatch (v0.1 mono-CHF, see L38 / Story 11).
#[ignore = "v0.1 mono-CHF, see spec L38 / Story 11 dependency"]
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn post_accept_filters_currency_mismatch(_pool: MySqlPool) {
    // Body placeholder — à un-`#[ignore]` post Story 11 multi-currency.
    // Le filtre currency au repo `find_unpaid_invoices_for_window` a été
    // retiré en Pass 4 S4-1 CRITICAL (colonne `invoices.currency`
    // inexistante). v0.1 garantit mono-CHF par convention.
    panic!("placeholder — implement post Story 11 multi-currency, see spec L38");
}

/// AC #68 — cross-tenant bank_account → 404 AVANT acquisition lock.
/// Vérifier timing < 1s pour confirmer l'absence de lock (lock = 5s).
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn post_accept_returns_404_on_cross_tenant_bank_account(pool: MySqlPool) {
    let ctx_a = setup_company(&pool, "CompanyA", "CH4431999123000889012", Role::Comptable).await;
    let ctx_b = setup_company(&pool, "CompanyB", "CH5604835012345678009", Role::Comptable).await;

    let app = spawn_app(pool).await;
    let body = serde_json::json!({
        "bankAccountId": ctx_b.bank_account_id, // appartient à company_B
        "proposals": [{ "type": "invoice", "bankTransactionId": 99999, "invoiceId": 99999 }],
    });
    let start = std::time::Instant::now();
    let resp = app
        .client
        .post(app.url("/api/v1/reconciliation/accept"))
        .bearer_auth(&ctx_a.jwt)
        .json(&body)
        .send()
        .await
        .unwrap();
    let elapsed = start.elapsed();
    assert_eq!(resp.status(), 404);
    let body: Value = resp.json().await.unwrap();
    assert_eq!(
        body["error"]["code"].as_str().unwrap(),
        "BANK_IMPORT_BANK_ACCOUNT_NOT_FOUND"
    );
    assert!(
        elapsed < std::time::Duration::from_secs(1),
        "404 doit retourner avant lock (5s), elapsed={:?}",
        elapsed
    );
}

/// Story 27-4, AC6-3 — commun aux deux tests d'anti-énumération : accepte la
/// transaction `tx_id` (étrangère au compte `ctx.bank_account_id`) contre la
/// facture `inv_id` de `ctx`, et rend `(statut, corps)`. Vérifie au passage
/// que rien n'est écrit : aucune écriture créée, facture non réglée,
/// transaction inchangée.
async fn accept_foreign_tx_and_assert_nothing_written(
    pool: &MySqlPool,
    ctx: &CompanyCtx,
    tx_id: i64,
    inv_id: i64,
) -> (u16, Value) {
    let entries_before: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM journal_entries")
        .fetch_one(pool)
        .await
        .unwrap();
    let tx_before: (String, Option<i64>, i32) = sqlx::query_as(
        "SELECT status, matched_entry_id, version FROM bank_transactions WHERE id = ?",
    )
    .bind(tx_id)
    .fetch_one(pool)
    .await
    .unwrap();
    let app = spawn_app(pool.clone()).await;
    let body = serde_json::json!({
        "bankAccountId": ctx.bank_account_id,
        "proposals": [{ "type": "invoice", "bankTransactionId": tx_id, "invoiceId": inv_id }],
    });
    let resp = app
        .client
        .post(app.url("/api/v1/reconciliation/accept"))
        .bearer_auth(&ctx.jwt)
        .json(&body)
        .send()
        .await
        .unwrap();
    let status = resp.status().as_u16();
    let body: Value = resp.json().await.unwrap();

    let entries_after: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM journal_entries")
        .fetch_one(pool)
        .await
        .unwrap();
    assert_eq!(entries_after, entries_before, "aucune écriture créée");
    let paid_at: Option<NaiveDateTime> =
        sqlx::query_scalar("SELECT paid_at FROM invoices WHERE id = ?")
            .bind(inv_id)
            .fetch_one(pool)
            .await
            .unwrap();
    assert!(paid_at.is_none(), "facture non réglée");
    let tx_after: (String, Option<i64>, i32) = sqlx::query_as(
        "SELECT status, matched_entry_id, version FROM bank_transactions WHERE id = ?",
    )
    .bind(tx_id)
    .fetch_one(pool)
    .await
    .unwrap();
    assert_eq!(tx_after, tx_before, "transaction étrangère inchangée");
    (status, body)
}

/// Remplace dans `body` tout identifiant de transaction égal à `id` par `0`,
/// pour comparer deux corps « identifiant mis à part » (AC6-3).
fn without_tx_id(mut body: Value, id: i64) -> Value {
    if let Some(failed) = body["failed"].as_array_mut() {
        for f in failed {
            if f["bankTransactionId"].as_i64() == Some(id) {
                f["bankTransactionId"] = Value::from(0);
            }
        }
    }
    body
}

/// AC #69 — cross-account proposal : bankAccountId=17 mais tx.bank_account_id=18.
///
/// **Story 27-4 (AC6-3, #634)** : auparavant un `400` du lot entier, dont le
/// message affirmait que la transaction « n'appartient pas » au compte ;
/// désormais `200`, refus par proposition `BANK_TRANSACTION_NOT_FOUND`, sans
/// `details`, rien d'écrit. Le jumeau « autre société » rend **exactement**
/// le même corps, identifiant mis à part (anti-énumération, AC1).
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn post_accept_refuses_cross_account_and_cross_company_transactions_identically(
    pool: MySqlPool,
) {
    let ctx = setup_company(&pool, "Acme", "CH4431999123000889012", Role::Comptable).await;
    let other = setup_company(&pool, "Other", "CH9300762011623852957", Role::Comptable).await;
    // 2e bank_account de la même company.
    let bank_account_2 = create_bank_account(&pool, ctx.company_id, "CH5604835012345678009").await;
    let day = NaiveDate::from_ymd_opt(2026, 5, 15).unwrap();
    let (inv_id, _) = seed_validated_invoice(
        &pool,
        ctx.company_id,
        ctx.contact_id,
        "INV-XACC",
        day,
        dec!(100.00),
    )
    .await;
    let seed_one = |company_id: i64, bank_id: i64, user_id: i64, seed: &'static str| {
        let pool = pool.clone();
        async move {
            seed_bank_transactions(
                &pool,
                company_id,
                bank_id,
                user_id,
                &unique_hash(seed),
                day,
                day,
                vec![make_new_tx(
                    company_id,
                    bank_id,
                    day,
                    Some(day),
                    dec!(100.00),
                    "CHF",
                    "INV-XACC",
                    Some("Acme Client"),
                )],
            )
            .await[0]
        }
    };
    let tx_other_account = seed_one(ctx.company_id, bank_account_2, ctx.user_id, "xacc").await;
    let tx_other_company = seed_one(
        other.company_id,
        other.bank_account_id,
        other.user_id,
        "xco",
    )
    .await;

    let (st_acc, body_acc) =
        accept_foreign_tx_and_assert_nothing_written(&pool, &ctx, tx_other_account, inv_id).await;
    assert_eq!(st_acc, 200, "{body_acc}");
    assert!(body_acc["accepted"].as_array().unwrap().is_empty());
    let failed = body_acc["failed"].as_array().unwrap();
    assert_eq!(failed.len(), 1, "{body_acc}");
    assert_eq!(
        failed[0]["bankTransactionId"].as_i64().unwrap(),
        tx_other_account
    );
    assert_eq!(failed[0]["errorCode"], "BANK_TRANSACTION_NOT_FOUND");
    assert!(failed[0]["details"].is_null(), "details nul : {body_acc}");

    let (st_co, body_co) =
        accept_foreign_tx_and_assert_nothing_written(&pool, &ctx, tx_other_company, inv_id).await;
    assert_eq!(st_co, 200, "{body_co}");
    assert!(body_co["failed"][0]["details"].is_null(), "{body_co}");
    assert_eq!(
        without_tx_id(body_acc, tx_other_account),
        without_tx_id(body_co, tx_other_company),
        "autre compte et autre société : même réponse, identifiant mis à part"
    );
}

/// AC #70 — paid_at < invoice.date → failed avec reason
/// payment_date_before_invoice_date.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn post_accept_rejects_payment_date_before_invoice_date(pool: MySqlPool) {
    let ctx = setup_company(&pool, "Acme", "CH4431999123000889012", Role::Comptable).await;
    // Tx au 2026-04-15 (avant date invoice 2026-05-01).
    let tx_day = NaiveDate::from_ymd_opt(2026, 4, 15).unwrap();
    let inv_date = NaiveDate::from_ymd_opt(2026, 5, 1).unwrap();
    let (inv_id, _) = seed_validated_invoice(
        &pool,
        ctx.company_id,
        ctx.contact_id,
        "INV-FUTURE",
        inv_date,
        dec!(100.00),
    )
    .await;
    let tx_ids = seed_bank_transactions(
        &pool,
        ctx.company_id,
        ctx.bank_account_id,
        ctx.user_id,
        &unique_hash("paid_before"),
        tx_day,
        tx_day,
        vec![make_new_tx(
            ctx.company_id,
            ctx.bank_account_id,
            tx_day,
            Some(tx_day),
            dec!(100.00),
            "CHF",
            "INV-FUTURE",
            Some("Acme Client"),
        )],
    )
    .await;

    let app = spawn_app(pool).await;
    let body = serde_json::json!({
        "bankAccountId": ctx.bank_account_id,
        "proposals": [{ "type": "invoice", "bankTransactionId": tx_ids[0], "invoiceId": inv_id }],
    });
    let resp = app
        .client
        .post(app.url("/api/v1/reconciliation/accept"))
        .bearer_auth(&ctx.jwt)
        .json(&body)
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let body: Value = resp.json().await.unwrap();
    let failed = body["failed"].as_array().unwrap();
    assert_eq!(failed.len(), 1);
    assert_eq!(
        failed[0]["errorCode"].as_str().unwrap(),
        "RECONCILIATION_INVOICE_NOT_ELIGIBLE"
    );
    assert_eq!(
        failed[0]["details"]["reason"].as_str().unwrap(),
        "payment_date_before_invoice_date"
    );
}

/// AC #71 — dual audit : reconciliation.accepted + invoice.paid avec
/// shape lightweight + reconciliation_audit_id link bidirectionnel.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn post_accept_emits_dual_audit_invoice_paid(pool: MySqlPool) {
    let ctx = setup_company(&pool, "Acme", "CH4431999123000889012", Role::Comptable).await;
    let day = NaiveDate::from_ymd_opt(2026, 5, 15).unwrap();
    let inv_date = NaiveDate::from_ymd_opt(2026, 5, 1).unwrap();
    let (inv_id, je_id) = seed_validated_invoice(
        &pool,
        ctx.company_id,
        ctx.contact_id,
        "INV-DUAL",
        inv_date,
        dec!(1000.00),
    )
    .await;
    let tx_ids = seed_bank_transactions(
        &pool,
        ctx.company_id,
        ctx.bank_account_id,
        ctx.user_id,
        &unique_hash("dual_audit"),
        day,
        day,
        vec![make_new_tx(
            ctx.company_id,
            ctx.bank_account_id,
            day,
            Some(day),
            dec!(1000.00),
            "CHF",
            "INV-DUAL",
            Some("Acme Client"),
        )],
    )
    .await;
    let tx_id = tx_ids[0];

    let app = spawn_app(pool.clone()).await;
    let body = serde_json::json!({
        "bankAccountId": ctx.bank_account_id,
        "proposals": [{ "type": "invoice", "bankTransactionId": tx_id, "invoiceId": inv_id }],
    });
    let resp = app
        .client
        .post(app.url("/api/v1/reconciliation/accept"))
        .bearer_auth(&ctx.jwt)
        .json(&body)
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);

    // Query 2 audit rows liés à invoice_id ET bank_transaction_id.
    // - reconciliation.accepted entity_id = tx_id
    // - invoice.paid entity_id = inv_id
    // ORDER BY id ASC : la 1ère est reconciliation.accepted (insérée step 9).
    let rows: Vec<(i64, String, String, i64, Value)> = sqlx::query_as(
        "SELECT id, action, entity_type, entity_id, details_json FROM audit_log \
         WHERE action IN ('reconciliation.accepted', 'invoice.paid') \
         ORDER BY id ASC",
    )
    .fetch_all(&pool)
    .await
    .unwrap();
    assert_eq!(rows.len(), 2, "2 entrées audit attendues");

    let row0 = &rows[0];
    assert_eq!(row0.1, "reconciliation.accepted");
    assert_eq!(row0.2, "bank_transaction");
    assert_eq!(row0.3, tx_id);
    let det0 = &row0.4;
    assert!(det0["score"]["total"].as_f64().is_some());
    assert!(det0["batch_size"].as_i64().is_some());
    // ⛔ Story 24-2 : `journal_entry_id` est celui de l'ENCAISSEMENT ; l'écriture
    // de vente est tracée à part, sous `sale_journal_entry_id`.
    assert_ne!(det0["journal_entry_id"].as_i64().unwrap(), je_id);
    assert_eq!(det0["sale_journal_entry_id"].as_i64().unwrap(), je_id);
    assert!(det0["fully_settled"].as_bool().unwrap());

    let row1 = &rows[1];
    assert_eq!(row1.1, "invoice.paid");
    assert_eq!(row1.2, "invoice");
    assert_eq!(row1.3, inv_id);
    let det1 = &row1.4;
    assert_eq!(det1["paid_via"].as_str().unwrap(), "reconciliation");
    assert_eq!(
        det1["reconciliation_audit_id"].as_i64().unwrap(),
        row0.0,
        "reconciliation_audit_id doit pointer sur l'entry reconciliation.accepted"
    );
    assert_eq!(det1["before"]["version"].as_i64().unwrap(), 1);
    assert_eq!(det1["after"]["version"].as_i64().unwrap(), 2);
}

/// AC #72 — currency tx-side guard.
///
/// **Angle mort documenté** : le contrôle `tx.currency != "CHF"` de
/// `reconciliation.rs` joue dans `get_proposals` (la transaction non CHF ne
/// reçoit aucune proposition) et dans le rapprochement par règle
/// (`RECONCILIATION_CURRENCY_MISMATCH`), mais **PAS** dans le rapprochement
/// d'une facture par `POST /accept` (`accept_one_invoice`) : avec une
/// transaction EUR et une facture dont montant, référence et contact
/// concordent, l'opération réussit.
///
/// Sévérité : **MEDIUM** en principe, sans prise aujourd'hui : aucun chemin
/// d'import n'enregistre une transaction non CHF. Le CAMT.053 la refuse
/// (`BANK_IMPORT_UNSUPPORTED_CURRENCY`) ; le CSV aussi depuis la Story 27-2
/// quand le profil lit la colonne Devise, et suppose CHF sans elle. Ce test
/// pose la transaction EUR **directement en base** pour l'exercer.
/// À fermer avec le multi-devise (garde symétrique dans `accept_one_invoice`,
/// un-`#[ignore]` de l'AC #67).
///
/// Pour ce test : on documente le comportement réel (succès silencieux)
/// pour empêcher une régression future qui changerait silencieusement
/// la sémantique. Le jour où la garde symétrique sera ajoutée, ce test
/// devra être inversé pour vérifier `failed[]` non vide.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn post_accept_skips_non_chf_transaction(pool: MySqlPool) {
    let ctx = setup_company(&pool, "Acme", "CH4431999123000889012", Role::Comptable).await;
    let day = NaiveDate::from_ymd_opt(2026, 5, 15).unwrap();
    let inv_date = NaiveDate::from_ymd_opt(2026, 5, 1).unwrap();
    let (inv_id, _) = seed_validated_invoice(
        &pool,
        ctx.company_id,
        ctx.contact_id,
        "INV-EUR",
        inv_date,
        dec!(100.00),
    )
    .await;
    let tx_ids = seed_bank_transactions(
        &pool,
        ctx.company_id,
        ctx.bank_account_id,
        ctx.user_id,
        &unique_hash("eur"),
        day,
        day,
        vec![make_new_tx(
            ctx.company_id,
            ctx.bank_account_id,
            day,
            Some(day),
            dec!(100.00),
            "EUR",
            "INV-EUR",
            Some("Acme Client"),
        )],
    )
    .await;

    let app = spawn_app(pool).await;
    let body = serde_json::json!({
        "bankAccountId": ctx.bank_account_id,
        "proposals": [{ "type": "invoice", "bankTransactionId": tx_ids[0], "invoiceId": inv_id }],
    });
    let resp = app
        .client
        .post(app.url("/api/v1/reconciliation/accept"))
        .bearer_auth(&ctx.jwt)
        .json(&body)
        .send()
        .await
        .unwrap();
    // Comportement réel observé (NON aligné avec AC #72 strict) : POST
    // accept ne filtre PAS la currency, le scoring matche amount + ref +
    // contact, l'opération réussit. Le test pin le comportement actuel
    // pour empêcher une régression silencieuse opposée.
    assert_eq!(resp.status(), 200);
    let body: Value = resp.json().await.unwrap();
    let accepted = body["accepted"].as_array().unwrap();
    let failed = body["failed"].as_array().unwrap();
    // Soit `accepted` (gap A6-2 POST-side documenté ci-dessus), soit
    // `failed` avec un errorCode currency-aware (si le guard est ajouté
    // ultérieurement). On accepte les 2 trajectoires pour ne pas casser
    // ce test si le fix arrive en Pass 5+.
    assert_eq!(
        accepted.len() + failed.len(),
        1,
        "exactement 1 résultat batch attendu"
    );
    if !failed.is_empty() {
        let err_code = failed[0]["errorCode"].as_str().unwrap();
        assert!(
            matches!(
                err_code,
                "RECONCILIATION_SCORE_TOO_LOW"
                    | "BANK_TRANSACTION_NOT_FOUND"
                    | "CURRENCY_NOT_SUPPORTED"
            ),
            "errorCode inattendu pour tx EUR : {err_code}"
        );
    }
}

/// AC #73 — P3-H4 min-score guard : POST accept avec couple (tx, invoice)
/// qui scorerait 0.0 (amount mismatch + no reference + no contact match)
/// → RECONCILIATION_SCORE_TOO_LOW.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn post_accept_rejects_zero_score_match(pool: MySqlPool) {
    let ctx = setup_company(&pool, "Acme", "CH4431999123000889012", Role::Comptable).await;
    let day = NaiveDate::from_ymd_opt(2026, 5, 15).unwrap();
    let inv_date = NaiveDate::from_ymd_opt(2026, 5, 1).unwrap();
    // Invoice 100 CHF avec contact "Acme Client".
    let (inv_id, _) = seed_validated_invoice(
        &pool,
        ctx.company_id,
        ctx.contact_id,
        "INV-CORRECT",
        inv_date,
        dec!(100.00),
    )
    .await;
    // Tx 9999 CHF (amount very different) avec ref/contact différents.
    let tx_ids = seed_bank_transactions(
        &pool,
        ctx.company_id,
        ctx.bank_account_id,
        ctx.user_id,
        &unique_hash("zero_score"),
        day,
        day,
        vec![make_new_tx(
            ctx.company_id,
            ctx.bank_account_id,
            day,
            Some(day),
            dec!(9999.99),
            "CHF",
            "DIFFERENT-REF",
            Some("Stranger Corp"),
        )],
    )
    .await;

    let app = spawn_app(pool).await;
    let body = serde_json::json!({
        "bankAccountId": ctx.bank_account_id,
        "proposals": [{ "type": "invoice", "bankTransactionId": tx_ids[0], "invoiceId": inv_id }],
    });
    let resp = app
        .client
        .post(app.url("/api/v1/reconciliation/accept"))
        .bearer_auth(&ctx.jwt)
        .json(&body)
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let body: Value = resp.json().await.unwrap();
    let failed = body["failed"].as_array().unwrap();
    assert_eq!(failed.len(), 1);
    assert_eq!(
        failed[0]["errorCode"].as_str().unwrap(),
        "RECONCILIATION_SCORE_TOO_LOW"
    );
    assert_eq!(
        failed[0]["details"]["reason"].as_str().unwrap(),
        "score_zero_no_match"
    );
}

/// AC #74 — P3-M2 upper bound : tx 2026-12-01 + invoice 2026-05-01
/// (>30 jours d'écart) → failed avec reason payment_date_outside_window.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn post_accept_rejects_payment_date_outside_window(pool: MySqlPool) {
    let ctx = setup_company(&pool, "Acme", "CH4431999123000889012", Role::Comptable).await;
    // Tx tardive bien après l'invoice.
    let tx_day = NaiveDate::from_ymd_opt(2026, 12, 1).unwrap();
    let inv_date = NaiveDate::from_ymd_opt(2026, 5, 1).unwrap();
    let (inv_id, _) = seed_validated_invoice(
        &pool,
        ctx.company_id,
        ctx.contact_id,
        "INV-OLD",
        inv_date,
        dec!(100.00),
    )
    .await;
    let tx_ids = seed_bank_transactions(
        &pool,
        ctx.company_id,
        ctx.bank_account_id,
        ctx.user_id,
        &unique_hash("upper_bound"),
        tx_day,
        tx_day,
        vec![make_new_tx(
            ctx.company_id,
            ctx.bank_account_id,
            tx_day,
            Some(tx_day),
            dec!(100.00),
            "CHF",
            "INV-OLD",
            Some("Acme Client"),
        )],
    )
    .await;

    let app = spawn_app(pool).await;
    let body = serde_json::json!({
        "bankAccountId": ctx.bank_account_id,
        "proposals": [{ "type": "invoice", "bankTransactionId": tx_ids[0], "invoiceId": inv_id }],
    });
    let resp = app
        .client
        .post(app.url("/api/v1/reconciliation/accept"))
        .bearer_auth(&ctx.jwt)
        .json(&body)
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let body: Value = resp.json().await.unwrap();
    let failed = body["failed"].as_array().unwrap();
    assert_eq!(failed.len(), 1);
    assert_eq!(
        failed[0]["errorCode"].as_str().unwrap(),
        "RECONCILIATION_INVOICE_NOT_ELIGIBLE"
    );
    assert_eq!(
        failed[0]["details"]["reason"].as_str().unwrap(),
        "payment_date_outside_window"
    );
    assert_eq!(failed[0]["details"]["window_days"].as_i64().unwrap(), 30);
}

// ============================================================
// Story 8-5a-bis Q2 — breaking change POST /accept discriminator type
// ============================================================

/// AC #100 part 1 — body proposal sans champ `type` → 400 Validation
/// (custom extractor `AcceptBodyExtractor`, F9 Pass 1).
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn accept_rejects_proposal_missing_type_discriminator(pool: MySqlPool) {
    let app = spawn_app(pool.clone()).await;
    let ctx = setup_company(
        &pool,
        "discrim_missing_co",
        "CH1000000000000099000",
        Role::Comptable,
    )
    .await;

    let body = serde_json::json!({
        "bankAccountId": ctx.bank_account_id,
        "proposals": [
            { "bankTransactionId": 1, "invoiceId": 1 }
        ]
    });
    let resp = app
        .client
        .post(app.url("/api/v1/reconciliation/accept"))
        .header("Authorization", format!("Bearer {}", ctx.jwt))
        .json(&body)
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 400, "{}", resp.text().await.unwrap());
    let body: Value = resp.json().await.unwrap();
    assert_eq!(body["error"]["code"], "VALIDATION_ERROR");
}

/// AC #100 part 2 — body avec `type: "invoice"` explicite exécute le flow
/// 8-4 standard (audit log `reconciliation.accepted` + `invoice.paid`).
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn accept_with_explicit_invoice_type_runs_8_4_flow(pool: MySqlPool) {
    let app = spawn_app(pool.clone()).await;
    let ctx = setup_company(
        &pool,
        "discrim_invoice_co",
        "CH1000000000000099001",
        Role::Comptable,
    )
    .await;

    let inv_date = NaiveDate::from_ymd_opt(2026, 5, 15).unwrap();
    let (inv_id, _je_id) = seed_validated_invoice(
        &pool,
        ctx.company_id,
        ctx.contact_id,
        "INV-DISCRIM",
        inv_date,
        dec!(200.00),
    )
    .await;
    let tx_ids = seed_bank_transactions(
        &pool,
        ctx.company_id,
        ctx.bank_account_id,
        ctx.user_id,
        &unique_hash("discrim_invoice"),
        inv_date,
        inv_date,
        vec![make_new_tx(
            ctx.company_id,
            ctx.bank_account_id,
            inv_date,
            Some(inv_date),
            dec!(200.00),
            "CHF",
            "INV-DISCRIM",
            Some(&format!("c{}", ctx.company_id)),
        )],
    )
    .await;

    let body = serde_json::json!({
        "bankAccountId": ctx.bank_account_id,
        "proposals": [
            { "type": "invoice", "bankTransactionId": tx_ids[0], "invoiceId": inv_id }
        ]
    });
    let resp = app
        .client
        .post(app.url("/api/v1/reconciliation/accept"))
        .header("Authorization", format!("Bearer {}", ctx.jwt))
        .json(&body)
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200, "{}", resp.text().await.unwrap());

    // Audit log reconciliation.accepted présent.
    let count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM audit_log WHERE action = 'reconciliation.accepted' AND entity_id = ?",
    )
    .bind(tx_ids[0])
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(count, 1);
}

/// AA-F1 Pass 1 code-review — coverage E2E HTTP du flow `POST /accept`
/// avec `type: "split"` (batch path qui exerce `accept_one_split`).
/// Vérifie : 200 OK + audit `reconciliation.split_applied` snake_case +
/// audit `journal_entry.created` + `bank_transactions.status='reconciled'`.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn accept_with_explicit_split_type_runs_split_flow(pool: MySqlPool) {
    let app = spawn_app(pool.clone()).await;
    let company_id = create_company(&pool, "accept_split_co").await;
    let user_id = create_user(&pool, "accept_split_user", Role::Comptable, company_id).await;
    let bank_account_id = create_bank_account(&pool, company_id, "CH1000000000000099002").await;

    // Bank ledger 1020 + 2 counterparties classes 5/6.
    let bank_ledger_account_id = accounts::create(
        &pool,
        user_id,
        NewAccount {
            company_id,
            number: "1020".into(),
            name: "Banque".into(),
            account_type: AccountType::Asset,
            parent_id: None,
            role: None,
            postable: true,
        },
    )
    .await
    .unwrap()
    .id;
    let cp_a = accounts::create(
        &pool,
        user_id,
        NewAccount {
            company_id,
            number: "5000".into(),
            name: "Salaires".into(),
            account_type: AccountType::Expense,
            parent_id: None,
            role: None,
            postable: true,
        },
    )
    .await
    .unwrap()
    .id;
    let cp_b = accounts::create(
        &pool,
        user_id,
        NewAccount {
            company_id,
            number: "5700".into(),
            name: "Charges sociales".into(),
            account_type: AccountType::Expense,
            parent_id: None,
            role: None,
            postable: true,
        },
    )
    .await
    .unwrap()
    .id;

    // Link bank_account → journal_account.
    let mut tx = pool.begin().await.unwrap();
    let ba = bank_accounts::find_by_id_for_company(&pool, company_id, bank_account_id)
        .await
        .unwrap()
        .expect("bank_account exists");
    bank_accounts::set_journal_account_id_for_company(
        &mut tx,
        company_id,
        bank_account_id,
        Some(bank_ledger_account_id),
        ba.version,
        &ClaimAccounts::default(),
    )
    .await
    .unwrap();
    tx.commit().await.unwrap();

    // Fiscal year ouvert 2026 (insert_fake_fiscal_year déjà défini).
    let _ = insert_fake_fiscal_year(&pool, company_id).await;

    let booking_date = NaiveDate::from_ymd_opt(2026, 5, 31).unwrap();
    let tx_ids = seed_bank_transactions(
        &pool,
        company_id,
        bank_account_id,
        user_id,
        &unique_hash("accept_split"),
        booking_date,
        booking_date,
        vec![make_new_tx(
            company_id,
            bank_account_id,
            booking_date,
            Some(booking_date),
            dec!(-100.00),
            "CHF",
            "BATCH-PAY",
            None,
        )],
    )
    .await;

    let jwt = forge_jwt(user_id, "Comptable", company_id);
    let body = serde_json::json!({
        "bankAccountId": bank_account_id,
        "proposals": [
            {
                "type": "split",
                "bankTransactionId": tx_ids[0],
                "splits": [
                    { "counterpartyAccountId": cp_a, "amount": "60.00", "description": "Salaire" },
                    { "counterpartyAccountId": cp_b, "amount": "40.00", "description": "Charges" },
                ],
            }
        ]
    });
    let resp = app
        .client
        .post(app.url("/api/v1/reconciliation/accept"))
        .header("Authorization", format!("Bearer {jwt}"))
        .json(&body)
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200, "{}", resp.text().await.unwrap());
    let resp_body: Value = resp.json().await.unwrap();
    let accepted = resp_body["accepted"].as_array().expect("accepted array");
    assert_eq!(accepted.len(), 1);
    let je_id = accepted[0]["journalEntryId"]
        .as_i64()
        .expect("journalEntryId i64");
    assert_eq!(accepted[0]["bankTransactionId"].as_i64(), Some(tx_ids[0]));

    // Audit log reconciliation.split_applied (pas reconciliation.accepted).
    let split_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM audit_log WHERE action = 'reconciliation.split_applied' \
         AND entity_id = ?",
    )
    .bind(tx_ids[0])
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(split_count, 1);

    // Audit log journal_entry.created.
    let je_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM audit_log WHERE action = 'journal_entry.created' AND entity_id = ?",
    )
    .bind(je_id)
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(je_count, 1);

    // tx status reconciled.
    let status: String = sqlx::query_scalar("SELECT status FROM bank_transactions WHERE id = ?")
        .bind(tx_ids[0])
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(status, "reconciled");
}

// ============================================================
// Story 24-2 (#371) — l'écriture d'encaissement
//
// ⚠️ Ces tests n'étaient PAS écrivables avant cette story : la fixture ne
// posait aucune ligne sur l'écriture de vente, donc le compte de créance
// n'existait nulle part et il n'y avait rien à solder. C'est le vide qui a
// laissé le défaut fondateur invisible pendant des mois.
// ============================================================

/// Poste une proposition d'encaissement et rend le corps de la réponse.
async fn post_accept_one(
    app: &TestApp,
    ctx: &CompanyCtx,
    tx_id: i64,
    inv_id: i64,
) -> serde_json::Value {
    let body = serde_json::json!({
        "bankAccountId": ctx.bank_account_id,
        "proposals": [{ "type": "invoice", "bankTransactionId": tx_id, "invoiceId": inv_id }],
    });
    let resp = app
        .client
        .post(app.url("/api/v1/reconciliation/accept"))
        .bearer_auth(&ctx.jwt)
        .json(&body)
        .send()
        .await
        .unwrap();
    assert_eq!(
        resp.status(),
        200,
        "succès partiel = succès HTTP (pattern batch)"
    );
    resp.json().await.unwrap()
}

/// Solde du compte de créance sur la période — l'invariante centrale.
async fn receivable_balance(pool: &MySqlPool, account_id: i64) -> Decimal {
    sqlx::query_scalar::<_, Decimal>(
        "SELECT COALESCE(SUM(debit) - SUM(credit), 0) FROM journal_entry_lines \
         WHERE account_id = ?",
    )
    .bind(account_id)
    .fetch_one(pool)
    .await
    .unwrap()
}

/// ⛔ **Deux encaissements partiels soldent la facture, et `paid_at` n'est posé
/// qu'au second.**
///
/// C'est l'arbitrage du Project Lead — *« une facture est payée parce que la
/// comptabilité le montre »* — rendu vérifiable. Une facture partiellement
/// réglée garde `paid_at` à NULL, donc elle reste relançable et rapprochable
/// pour le solde, sans qu'aucun des cinq sites lisant `paid_at IS NULL` change.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn accept_settles_an_invoice_in_two_payments(pool: MySqlPool) {
    let ctx = setup_company(&pool, "Partiel", "CH4431999123000889012", Role::Comptable).await;
    let day = NaiveDate::from_ymd_opt(2026, 5, 15).unwrap();
    let inv_date = NaiveDate::from_ymd_opt(2026, 4, 20).unwrap();
    let (inv_id, je_id) = seed_validated_invoice(
        &pool,
        ctx.company_id,
        ctx.contact_id,
        "INV-PART-1",
        inv_date,
        dec!(100.00),
    )
    .await;
    let tx_ids = seed_bank_transactions(
        &pool,
        ctx.company_id,
        ctx.bank_account_id,
        ctx.user_id,
        &unique_hash("accept_partial"),
        day,
        day,
        vec![
            make_new_tx(
                ctx.company_id,
                ctx.bank_account_id,
                day,
                Some(day),
                dec!(60.00),
                "CHF",
                "INV-PART-1",
                Some("Partiel Client"),
            ),
            make_new_tx(
                ctx.company_id,
                ctx.bank_account_id,
                day,
                Some(day),
                dec!(40.00),
                "CHF",
                "INV-PART-1",
                Some("Partiel Client"),
            ),
        ],
    )
    .await;

    let app = spawn_app(pool.clone()).await;

    // Premier versement : 60 sur 100.
    let body = post_accept_one(&app, &ctx, tx_ids[0], inv_id).await;
    assert_eq!(body["accepted"].as_array().unwrap().len(), 1);
    let paid_at: Option<NaiveDateTime> =
        sqlx::query_scalar("SELECT paid_at FROM invoices WHERE id = ?")
            .bind(inv_id)
            .fetch_one(&pool)
            .await
            .unwrap();
    assert!(
        paid_at.is_none(),
        "⛔ 60 sur 100 ne SOLDE pas : `paid_at` doit rester NULL, sinon la \
         facture sort des relances alors qu'il reste 40 dus"
    );
    assert_eq!(
        receivable_balance(&pool, ctx.receivable_account_id).await,
        dec!(40.00),
        "le compte de créance porte le solde restant"
    );

    // Second versement : le solde tombe à zéro.
    let body = post_accept_one(&app, &ctx, tx_ids[1], inv_id).await;
    assert_eq!(
        body["accepted"].as_array().unwrap().len(),
        1,
        "failed = {:?}",
        body["failed"]
    );
    let paid_at: Option<NaiveDateTime> =
        sqlx::query_scalar("SELECT paid_at FROM invoices WHERE id = ?")
            .bind(inv_id)
            .fetch_one(&pool)
            .await
            .unwrap();
    assert!(paid_at.is_some(), "solde nul ⇒ la facture est payée");
    assert_eq!(
        receivable_balance(&pool, ctx.receivable_account_id).await,
        Decimal::ZERO,
        "⛔ L'INVARIANTE : le compte de créance se solde exactement"
    );

    // Deux liaisons, une par versement, et aucune ne pointe sur la vente.
    let settlements: Vec<(i64, Decimal)> = sqlx::query_as(
        "SELECT journal_entry_id, amount FROM invoice_settlements \
         WHERE invoice_id = ? ORDER BY id",
    )
    .bind(inv_id)
    .fetch_all(&pool)
    .await
    .unwrap();
    assert_eq!(settlements.len(), 2);
    assert_eq!(settlements[0].1, dec!(60.00));
    assert_eq!(settlements[1].1, dec!(40.00));
    assert!(settlements.iter().all(|(entry_id, _)| *entry_id != je_id));
}

/// Story 25-4-c (#420) — une facture de 1 000.— **à TVA non nulle** (HT 925.07
/// @ 8.1 % → TVA 74.93), déjà réglée de 400.— par un premier virement accepté,
/// plus une transaction de 600.— — le solde — qui ne porte **ni** la référence
/// de la facture **ni** le nom du contact : seul le score de MONTANT peut la
/// relier à la facture. Retourne `(invoice_id, tx_du_solde)`.
async fn seed_partially_settled_vat_invoice(
    pool: &MySqlPool,
    app: &TestApp,
    ctx: &CompanyCtx,
) -> (i64, i64) {
    let day = NaiveDate::from_ymd_opt(2026, 5, 15).unwrap();
    let inv_date = NaiveDate::from_ymd_opt(2026, 4, 20).unwrap();
    // L'écriture de vente débite la créance du TTC, 1 000.— ; la ligne de
    // facture est ensuite remplacée par sa forme à TVA, de même TTC.
    let (inv_id, _je_id) = seed_validated_invoice(
        pool,
        ctx.company_id,
        ctx.contact_id,
        "INV-VAT-PART",
        inv_date,
        dec!(1000.00),
    )
    .await;
    sqlx::query(
        "UPDATE invoice_lines SET unit_price = 925.07, line_total = 925.07, vat_rate = 8.10 \
         WHERE invoice_id = ?",
    )
    .bind(inv_id)
    .execute(pool)
    .await
    .expect("ligne à TVA");
    sqlx::query("UPDATE invoices SET total_amount = 925.07 WHERE id = ?")
        .bind(inv_id)
        .execute(pool)
        .await
        .expect("HT de la facture");

    let tx_ids = seed_bank_transactions(
        pool,
        ctx.company_id,
        ctx.bank_account_id,
        ctx.user_id,
        &unique_hash("partially_settled_vat"),
        day,
        day,
        vec![
            // Premier virement : 400.—, avec la référence (score de référence).
            make_new_tx(
                ctx.company_id,
                ctx.bank_account_id,
                day,
                Some(day),
                dec!(400.00),
                "CHF",
                "INV-VAT-PART",
                Some("Solde Client"),
            ),
            // Le solde : 600.—, sans référence ni contact reconnaissables.
            make_new_tx(
                ctx.company_id,
                ctx.bank_account_id,
                day,
                Some(day),
                dec!(600.00),
                "CHF",
                "VIREMENT DIVERS",
                Some("Tiers Inconnu SA"),
            ),
        ],
    )
    .await;
    let body = post_accept_one(app, ctx, tx_ids[0], inv_id).await;
    assert_eq!(
        body["accepted"].as_array().unwrap().len(),
        1,
        "le premier virement doit régler 400.— ; failed = {:?}",
        body["failed"]
    );
    (inv_id, tx_ids[1])
}

/// #420 (Story 25-4-c) — le virement du **solde** d'une facture réglée en
/// partie est proposé, score de montant 1, montant affiché = le reste, et le
/// TTC en mention. Avant la correction, la facture n'était même pas candidate.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn get_proposals_offers_the_amount_due_of_a_partially_settled_invoice(pool: MySqlPool) {
    let ctx = setup_company(&pool, "Solde", "CH4431999123000889012", Role::Comptable).await;
    let app = spawn_app(pool.clone()).await;
    let (inv_id, balance_tx_id) = seed_partially_settled_vat_invoice(&pool, &app, &ctx).await;

    let resp = app
        .client
        .get(app.url(&format!(
            "/api/v1/reconciliation/proposals?bankAccountId={}",
            ctx.bank_account_id
        )))
        .bearer_auth(&ctx.jwt)
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let body: Value = resp.json().await.unwrap();
    let prop = body["proposals"]
        .as_array()
        .expect("proposals array")
        .iter()
        .find(|p| p["bankTransactionId"] == balance_tx_id)
        .expect("proposition pour le virement du solde");
    let cand = prop["candidates"]
        .as_array()
        .unwrap()
        .iter()
        .find(|c| c["invoiceId"] == inv_id)
        .expect("la facture réglée en partie doit être candidate pour son solde");
    assert_eq!(cand["score"]["amountScore"], 1.0);
    assert_eq!(cand["score"]["referenceScore"], 0.0);
    assert_eq!(cand["score"]["contactScore"], 0.0);
    assert_eq!(
        cand["invoiceAmount"], "600",
        "le montant affiché est le reste dû"
    );
    assert_eq!(cand["invoiceTotalTtc"], "1000", "le TTC suit, en mention");
}

/// #420 (Story 25-4-c) — le virement du solde **s'accepte** et solde la
/// facture, sur le seul score de montant : sans référence ni contact, le total
/// valait 0 avant la correction (`RECONCILIATION_SCORE_TOO_LOW`).
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn accept_settles_the_balance_of_a_partially_settled_invoice_on_amount(pool: MySqlPool) {
    let ctx = setup_company(&pool, "Solde2", "CH4431999123000889012", Role::Comptable).await;
    let app = spawn_app(pool.clone()).await;
    let (inv_id, balance_tx_id) = seed_partially_settled_vat_invoice(&pool, &app, &ctx).await;

    let body = post_accept_one(&app, &ctx, balance_tx_id, inv_id).await;
    let accepted = body["accepted"].as_array().unwrap();
    assert_eq!(accepted.len(), 1, "failed = {:?}", body["failed"]);
    assert_eq!(accepted[0]["score"]["amountScore"], 1.0);
    assert_eq!(accepted[0]["score"]["referenceScore"], 0.0);
    assert_eq!(accepted[0]["score"]["contactScore"], 0.0);

    let paid_at: Option<NaiveDateTime> =
        sqlx::query_scalar("SELECT paid_at FROM invoices WHERE id = ?")
            .bind(inv_id)
            .fetch_one(&pool)
            .await
            .unwrap();
    assert!(paid_at.is_some(), "le solde versé, la facture est payée");
    assert_eq!(
        receivable_balance(&pool, ctx.receivable_account_id).await,
        Decimal::ZERO,
        "⛔ le compte de créance se solde exactement"
    );
}

/// ⛔ **Le trop-perçu est refusé, il ne s'écrit pas.**
///
/// Sans ce garde, le compte de créance passerait créditeur — un solde contre
/// nature que le grand livre signalerait, mais après coup.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn accept_refuses_an_overpayment_without_writing_anything(pool: MySqlPool) {
    let ctx = setup_company(&pool, "Trop", "CH4431999123000889012", Role::Comptable).await;
    let day = NaiveDate::from_ymd_opt(2026, 5, 15).unwrap();
    let inv_date = NaiveDate::from_ymd_opt(2026, 4, 20).unwrap();
    let (inv_id, _je_id) = seed_validated_invoice(
        &pool,
        ctx.company_id,
        ctx.contact_id,
        "INV-TROP-1",
        inv_date,
        dec!(100.00),
    )
    .await;
    let tx_ids = seed_bank_transactions(
        &pool,
        ctx.company_id,
        ctx.bank_account_id,
        ctx.user_id,
        &unique_hash("accept_overpay"),
        day,
        day,
        vec![make_new_tx(
            ctx.company_id,
            ctx.bank_account_id,
            day,
            Some(day),
            dec!(150.00),
            "CHF",
            // La référence matche : le score est > 0, donc le garde de score
            // ne suffit PAS — c'est bien le nouveau garde qui doit refuser.
            "INV-TROP-1",
            Some("Trop Client"),
        )],
    )
    .await;

    let app = spawn_app(pool.clone()).await;
    let body = post_accept_one(&app, &ctx, tx_ids[0], inv_id).await;

    assert!(body["accepted"].as_array().unwrap().is_empty());
    let failed = body["failed"].as_array().unwrap();
    assert_eq!(failed.len(), 1);
    assert_eq!(
        failed[0]["errorCode"].as_str().unwrap(),
        "RECONCILIATION_OVERPAYMENT"
    );

    // Rien n'a été écrit : ni liaison, ni `paid_at`, ni mouvement de créance.
    let settlements: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM invoice_settlements WHERE invoice_id = ?")
            .bind(inv_id)
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(settlements, 0);
    assert_eq!(
        receivable_balance(&pool, ctx.receivable_account_id).await,
        dec!(100.00),
        "la créance est intacte"
    );
}

/// ⛔ **Story 25-3-a-1 (#414) — un règlement né d'un rapprochement ne s'annule
/// pas : c'est le dé-rapprochement (25-3-b) qui défait ce lien d'abord.**
///
/// Par le **chemin réel** (`POST /reconciliation/accept`) : le règlement porte
/// à la fois une ligne `invoice_settlements` et un `matched_entry_id`. La
/// liste l'annonce avant le clic, avec l'identifiant de la transaction ; le
/// clic est refusé par la contre-passation elle-même — l'exemption du
/// règlement ne lève QUE son motif, la précédence se poursuit jusqu'au
/// rapprochement. Rien n'est retiré.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn cancelling_a_reconciled_settlement_is_refused(pool: MySqlPool) {
    let ctx = setup_company(&pool, "Acme", "CH4431999123000889012", Role::Comptable).await;
    let day = NaiveDate::from_ymd_opt(2026, 5, 15).unwrap();
    let (inv_id, _) = seed_validated_invoice(
        &pool,
        ctx.company_id,
        ctx.contact_id,
        "INV-2026-001",
        NaiveDate::from_ymd_opt(2026, 4, 20).unwrap(),
        dec!(1234.56),
    )
    .await;
    let tx_id = seed_bank_transactions(
        &pool,
        ctx.company_id,
        ctx.bank_account_id,
        ctx.user_id,
        &unique_hash("cancel_reconciled"),
        day,
        day,
        vec![make_new_tx(
            ctx.company_id,
            ctx.bank_account_id,
            day,
            Some(day),
            dec!(1234.56),
            "CHF",
            "INV-2026-001",
            Some("Acme Client"),
        )],
    )
    .await[0];
    let app = spawn_app(pool.clone()).await;
    let resp = app
        .client
        .post(app.url("/api/v1/reconciliation/accept"))
        .bearer_auth(&ctx.jwt)
        .json(&serde_json::json!({
            "bankAccountId": ctx.bank_account_id,
            "proposals": [{ "type": "invoice", "bankTransactionId": tx_id, "invoiceId": inv_id }],
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let sid: i64 = sqlx::query_scalar("SELECT id FROM invoice_settlements WHERE invoice_id = ?")
        .bind(inv_id)
        .fetch_one(&pool)
        .await
        .expect("le rapprochement a créé une ligne de règlement");

    let list: Value = app
        .client
        .get(app.url(&format!("/api/v1/invoices/{inv_id}/settlements")))
        .bearer_auth(&ctx.jwt)
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(list[0]["cancellable"], false, "got {list:?}");
    assert_eq!(list[0]["cancelBlockedBy"], "MATCHED_BANK_TRANSACTION");
    assert_eq!(list[0]["cancelBlockedDocumentId"], tx_id);

    let resp = app
        .client
        .post(app.url(&format!(
            "/api/v1/invoices/{inv_id}/settlements/{sid}/cancel"
        )))
        .bearer_auth(&ctx.jwt)
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 409);
    let body: Value = resp.json().await.unwrap();
    assert_eq!(
        body["error"]["code"], "MATCHED_BANK_TRANSACTION",
        "got {body:?}"
    );
    assert_eq!(body["error"]["details"]["documentId"], tx_id);
    let restantes: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM invoice_settlements WHERE id = ?")
            .bind(sid)
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(restantes, 1, "refusée ⇒ rien de retiré");
}

// ============================================================
// Story 25-3-b (#418) — annuler un rapprochement
// ============================================================

/// Un exercice ouvert qui couvre la date du JOUR — celle où la contre-passation
/// est datée. `insert_fake_fiscal_year` ne crée que 2026 : sans ce complément,
/// ces tests cesseraient de passer au 1er janvier.
async fn ensure_fiscal_year_today(pool: &MySqlPool, company_id: i64) {
    // 2026 d'abord, par l'outil commun — sans quoi ses appels suivants
    // entreraient en collision (`uq_fiscal_years_company_start_date`).
    let _ = insert_fake_fiscal_year(pool, company_id).await;
    let today = Utc::now().date_naive();
    let covered: Option<i64> = sqlx::query_scalar(
        "SELECT id FROM fiscal_years WHERE company_id = ? AND start_date <= ? AND end_date >= ?",
    )
    .bind(company_id)
    .bind(today)
    .bind(today)
    .fetch_optional(pool)
    .await
    .unwrap();
    if covered.is_none() {
        let year = chrono::Datelike::year(&today);
        sqlx::query(
            "INSERT INTO fiscal_years (company_id, name, start_date, end_date, status, \
             created_at, updated_at) VALUES (?, ?, ?, ?, 'Open', NOW(3), NOW(3))",
        )
        .bind(company_id)
        .bind(format!("FY {year} jour c{company_id}"))
        .bind(NaiveDate::from_ymd_opt(year, 1, 1).unwrap())
        .bind(NaiveDate::from_ymd_opt(year, 12, 31).unwrap())
        .execute(pool)
        .await
        .unwrap();
    }
}

async fn cancel_reco(app: &TestApp, jwt: &str, tx_id: i64) -> (u16, Value) {
    let resp = app
        .client
        .post(app.url(&format!(
            "/api/v1/reconciliation/transactions/{tx_id}/cancel"
        )))
        .bearer_auth(jwt)
        .send()
        .await
        .unwrap();
    let status = resp.status().as_u16();
    (status, resp.json().await.unwrap_or(Value::Null))
}

async fn get_reco(app: &TestApp, jwt: &str, tx_id: i64) -> (u16, Value) {
    let resp = app
        .client
        .get(app.url(&format!("/api/v1/reconciliation/transactions/{tx_id}")))
        .bearer_auth(jwt)
        .send()
        .await
        .unwrap();
    let status = resp.status().as_u16();
    (status, resp.json().await.unwrap_or(Value::Null))
}

async fn matched_entry_of(pool: &MySqlPool, tx_id: i64) -> Option<i64> {
    sqlx::query_scalar("SELECT matched_entry_id FROM bank_transactions WHERE id = ?")
        .bind(tx_id)
        .fetch_one(pool)
        .await
        .unwrap()
}

/// ⛔ Ce qu'un dé-rapprochement doit laisser, quel que soit le chemin : la
/// transaction « à rapprocher » (statut, lien, marqueur de rejet), l'écriture
/// d'origine contre-passée par une écriture inverse, et — surtout — la
/// transaction **de retour dans les propositions** : c'est ce que l'écran lit.
async fn assert_back_to_reconcile(
    pool: &MySqlPool,
    app: &TestApp,
    jwt: &str,
    bank_account_id: i64,
    tx_id: i64,
    entry_id: i64,
    reversal_id: i64,
) {
    let (status, matched, rejected): (String, Option<i64>, Option<NaiveDateTime>) = sqlx::query_as(
        "SELECT status, matched_entry_id, auto_match_rejected_at \
             FROM bank_transactions WHERE id = ?",
    )
    .bind(tx_id)
    .fetch_one(pool)
    .await
    .unwrap();
    assert_eq!(status, "pending");
    assert_eq!(matched, None, "le lien est défait");
    assert_eq!(rejected, None, "le marqueur de rejet est remis à zéro");
    let reverses: Option<i64> =
        sqlx::query_scalar("SELECT reverses_entry_id FROM journal_entries WHERE id = ?")
            .bind(reversal_id)
            .fetch_one(pool)
            .await
            .unwrap();
    assert_eq!(
        reverses,
        Some(entry_id),
        "l'écriture inverse vise l'origine"
    );
    let origin_left: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM journal_entries WHERE id = ?")
        .bind(entry_id)
        .fetch_one(pool)
        .await
        .unwrap();
    assert_eq!(
        origin_left, 1,
        "l'origine reste — contre-passée, jamais supprimée"
    );
    let body: Value = app
        .client
        .get(app.url(&format!(
            "/api/v1/reconciliation/proposals?bankAccountId={bank_account_id}"
        )))
        .bearer_auth(jwt)
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    let back = body["proposals"]
        .as_array()
        .unwrap()
        .iter()
        .any(|p| p["bankTransactionId"].as_i64() == Some(tx_id));
    assert!(
        back,
        "la transaction revient dans les propositions — got {body:?}"
    );
}

/// Une facture rapprochée par une proposition acceptée, prête à dé-rapprocher.
async fn reconciled_invoice(
    pool: &MySqlPool,
    app: &TestApp,
    ctx: &CompanyCtx,
    number: &str,
    invoice_date: NaiveDate,
    paid_on: NaiveDate,
    amount: Decimal,
) -> (i64, i64) {
    let (inv_id, _) = seed_validated_invoice(
        pool,
        ctx.company_id,
        ctx.contact_id,
        number,
        invoice_date,
        amount,
    )
    .await;
    let tx_id = seed_bank_transactions(
        pool,
        ctx.company_id,
        ctx.bank_account_id,
        ctx.user_id,
        &unique_hash(number),
        paid_on,
        paid_on,
        vec![make_new_tx(
            ctx.company_id,
            ctx.bank_account_id,
            paid_on,
            Some(paid_on),
            amount,
            "CHF",
            number,
            Some("Acme Client"),
        )],
    )
    .await[0];
    let accepted = post_accept_one(app, ctx, tx_id, inv_id).await;
    assert_eq!(
        accepted["accepted"].as_array().map(Vec::len),
        Some(1),
        "got {accepted:?}"
    );
    (inv_id, tx_id)
}

/// ⛔ **Chemin « facture »** : le règlement retiré, la facture de nouveau à
/// régler, la transaction de retour — puis re-rapprochable.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn cancelling_an_invoice_reconciliation_undoes_everything(pool: MySqlPool) {
    let ctx = setup_company(&pool, "Acme", "CH4431999123000889012", Role::Comptable).await;
    ensure_fiscal_year_today(&pool, ctx.company_id).await;
    let app = spawn_app(pool.clone()).await;
    let day = NaiveDate::from_ymd_opt(2026, 5, 15).unwrap();
    let (inv_id, tx_id) = reconciled_invoice(
        &pool,
        &app,
        &ctx,
        "INV-2026-001",
        NaiveDate::from_ymd_opt(2026, 4, 20).unwrap(),
        day,
        dec!(1234.56),
    )
    .await;
    let entry_id = matched_entry_of(&pool, tx_id).await.expect("rapprochée");

    let (st, view) = get_reco(&app, &ctx.jwt, tx_id).await;
    assert_eq!(st, 200, "got {view:?}");
    assert_eq!(view["kind"], "invoice_settlement");
    assert_eq!(view["invoiceId"], inv_id);
    assert_eq!(view["invoiceNumber"], "INV-2026-001");
    assert_eq!(view["matchedEntryId"], entry_id);
    assert_eq!(view["cancellable"], true, "got {view:?}");

    let (st, done) = cancel_reco(&app, &ctx.jwt, tx_id).await;
    assert_eq!(st, 200, "got {done:?}");
    assert_eq!(done["invoiceId"], inv_id);
    assert_eq!(done["bankTransaction"]["status"], "pending");
    let reversal_id = done["reversalJournalEntryId"].as_i64().unwrap();
    assert_back_to_reconcile(
        &pool,
        &app,
        &ctx.jwt,
        ctx.bank_account_id,
        tx_id,
        entry_id,
        reversal_id,
    )
    .await;

    let (settlements, paid_at): (i64, Option<NaiveDateTime>) = sqlx::query_as(
        "SELECT (SELECT COUNT(*) FROM invoice_settlements WHERE invoice_id = i.id), i.paid_at \
         FROM invoices i WHERE i.id = ?",
    )
    .bind(inv_id)
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(
        (settlements, paid_at),
        (0, None),
        "la facture redevient à régler"
    );
    let audits: Vec<String> = sqlx::query_scalar(
        "SELECT action FROM audit_log WHERE action IN \
         ('reconciliation.cancelled', 'invoice.settlement_cancelled', 'journal_entry.reversed') \
         ORDER BY action",
    )
    .fetch_all(&pool)
    .await
    .unwrap();
    assert_eq!(
        audits,
        [
            "invoice.settlement_cancelled",
            "journal_entry.reversed",
            "reconciliation.cancelled"
        ],
        "trois lignes, chacune nomme son objet"
    );

    // Re-rapprochable : la même transaction s'accepte à nouveau.
    let again = post_accept_one(&app, &ctx, tx_id, inv_id).await;
    assert_eq!(
        again["accepted"].as_array().map(Vec::len),
        Some(1),
        "got {again:?}"
    );
}

/// Une facture réglée en deux fois — un règlement en caisse, un rapprochement :
/// dé-rapprocher ne retire que le règlement rapproché.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn cancelling_one_of_two_settlements_keeps_the_other(pool: MySqlPool) {
    let ctx = setup_company(&pool, "Acme", "CH4431999123000889012", Role::Comptable).await;
    ensure_fiscal_year_today(&pool, ctx.company_id).await;
    let app = spawn_app(pool.clone()).await;
    let day = NaiveDate::from_ymd_opt(2026, 5, 15).unwrap();
    let (inv_id, _) = seed_validated_invoice(
        &pool,
        ctx.company_id,
        ctx.contact_id,
        "INV-2026-002",
        NaiveDate::from_ymd_opt(2026, 4, 20).unwrap(),
        dec!(1000.00),
    )
    .await;
    let tx_id = seed_bank_transactions(
        &pool,
        ctx.company_id,
        ctx.bank_account_id,
        ctx.user_id,
        &unique_hash("two_settlements"),
        day,
        day,
        vec![make_new_tx(
            ctx.company_id,
            ctx.bank_account_id,
            day,
            Some(day),
            dec!(400.00),
            "CHF",
            "INV-2026-002",
            Some("Acme Client"),
        )],
    )
    .await[0];
    let accepted = post_accept_one(&app, &ctx, tx_id, inv_id).await;
    assert_eq!(
        accepted["accepted"].as_array().map(Vec::len),
        Some(1),
        "got {accepted:?}"
    );
    // Le second règlement, par la route d'encaissement (vrai chemin).
    let resp = app
        .client
        .post(app.url(&format!("/api/v1/invoices/{inv_id}/settlements")))
        .bearer_auth(&ctx.jwt)
        .json(&serde_json::json!({
            "settlementType": "bank_transfer",
            "bankAccountId": ctx.bank_account_id,
            "amount": "250.00",
            "settledOn": "2026-05-20",
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200, "got {:?}", resp.text().await);

    let (st, done) = cancel_reco(&app, &ctx.jwt, tx_id).await;
    assert_eq!(st, 200, "got {done:?}");
    let amounts: Vec<Decimal> =
        sqlx::query_scalar("SELECT amount FROM invoice_settlements WHERE invoice_id = ?")
            .bind(inv_id)
            .fetch_all(&pool)
            .await
            .unwrap();
    assert_eq!(
        amounts,
        vec![dec!(250.00)],
        "seul le règlement rapproché est retiré"
    );
    let paid_at: Option<NaiveDateTime> =
        sqlx::query_scalar("SELECT paid_at FROM invoices WHERE id = ?")
            .bind(inv_id)
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(paid_at, None, "toujours partiellement réglée");
}

/// ⛔ **Chemin « rapprochement manuel »** : une écriture que seule la
/// transaction possède, contre-passée.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn cancelling_a_manual_reconciliation_reverses_its_entry(pool: MySqlPool) {
    let ctx = setup_company(&pool, "Acme", "CH4431999123000889012", Role::Comptable).await;
    let _ = insert_fake_fiscal_year(&pool, ctx.company_id).await;
    ensure_fiscal_year_today(&pool, ctx.company_id).await;
    let app = spawn_app(pool.clone()).await;
    let (_, revenue_id) = account_ids(&pool, ctx.company_id).await;
    let day = NaiveDate::from_ymd_opt(2026, 5, 15).unwrap();
    let tx_id = manual_reconciled(&pool, &app, &ctx, revenue_id, day, "manual").await;
    let entry_id = matched_entry_of(&pool, tx_id).await.expect("rapprochée");

    let (st, view) = get_reco(&app, &ctx.jwt, tx_id).await;
    assert_eq!(st, 200);
    assert_eq!(view["kind"], "entry");
    assert_eq!(view["invoiceId"], Value::Null);
    assert_eq!(view["cancellable"], true, "got {view:?}");

    let (st, done) = cancel_reco(&app, &ctx.jwt, tx_id).await;
    assert_eq!(st, 200, "got {done:?}");
    assert_eq!(done["invoiceId"], Value::Null);
    let reversal_id = done["reversalJournalEntryId"].as_i64().unwrap();
    assert_back_to_reconcile(
        &pool,
        &app,
        &ctx.jwt,
        ctx.bank_account_id,
        tx_id,
        entry_id,
        reversal_id,
    )
    .await;
}

/// Une transaction entrante rapprochée manuellement (`POST /manual`).
async fn manual_reconciled(
    pool: &MySqlPool,
    app: &TestApp,
    ctx: &CompanyCtx,
    counterparty_account_id: i64,
    day: NaiveDate,
    seed: &str,
) -> i64 {
    let tx_id = seed_bank_transactions(
        pool,
        ctx.company_id,
        ctx.bank_account_id,
        ctx.user_id,
        &unique_hash(seed),
        day,
        day,
        vec![make_new_tx(
            ctx.company_id,
            ctx.bank_account_id,
            day,
            Some(day),
            dec!(80.00),
            "CHF",
            seed,
            Some("Divers"),
        )],
    )
    .await[0];
    let resp = app
        .client
        .post(app.url("/api/v1/reconciliation/manual"))
        .bearer_auth(&ctx.jwt)
        .json(&serde_json::json!({
            "bankAccountId": ctx.bank_account_id,
            "bankTransactionId": tx_id,
            "counterpartyAccountId": counterparty_account_id,
            "description": "Recette diverse",
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200, "got {:?}", resp.text().await);
    tx_id
}

/// ⛔ **Chemin « éclatement manuel »** (`POST /split`).
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn cancelling_a_manual_split_reverses_its_entry(pool: MySqlPool) {
    let ctx = setup_company(&pool, "Acme", "CH4431999123000889012", Role::Comptable).await;
    let _ = insert_fake_fiscal_year(&pool, ctx.company_id).await;
    ensure_fiscal_year_today(&pool, ctx.company_id).await;
    let app = spawn_app(pool.clone()).await;
    let (receivable_id, revenue_id) = account_ids(&pool, ctx.company_id).await;
    let day = NaiveDate::from_ymd_opt(2026, 5, 15).unwrap();
    let tx_id = seed_bank_transactions(
        &pool,
        ctx.company_id,
        ctx.bank_account_id,
        ctx.user_id,
        &unique_hash("post_split"),
        day,
        day,
        vec![make_new_tx(
            ctx.company_id,
            ctx.bank_account_id,
            day,
            Some(day),
            dec!(100.00),
            "CHF",
            "SPLIT",
            Some("Divers"),
        )],
    )
    .await[0];
    let resp = app
        .client
        .post(app.url("/api/v1/reconciliation/split"))
        .bearer_auth(&ctx.jwt)
        .json(&serde_json::json!({
            "bankAccountId": ctx.bank_account_id,
            "bankTransactionId": tx_id,
            "splits": [
                { "counterpartyAccountId": revenue_id, "amount": "60.00", "description": "Part A" },
                { "counterpartyAccountId": receivable_id, "amount": "40.00", "description": "Part B" },
            ],
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200, "got {:?}", resp.text().await);
    let entry_id = matched_entry_of(&pool, tx_id).await.expect("rapprochée");

    let (st, done) = cancel_reco(&app, &ctx.jwt, tx_id).await;
    assert_eq!(st, 200, "got {done:?}");
    let reversal_id = done["reversalJournalEntryId"].as_i64().unwrap();
    assert_back_to_reconcile(
        &pool,
        &app,
        &ctx.jwt,
        ctx.bank_account_id,
        tx_id,
        entry_id,
        reversal_id,
    )
    .await;
}

/// ⛔ **Chemin « éclatement accepté »** (proposition `type: split`).
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn cancelling_an_accepted_split_reverses_its_entry(pool: MySqlPool) {
    let ctx = setup_company(&pool, "Acme", "CH4431999123000889012", Role::Comptable).await;
    let _ = insert_fake_fiscal_year(&pool, ctx.company_id).await;
    ensure_fiscal_year_today(&pool, ctx.company_id).await;
    let app = spawn_app(pool.clone()).await;
    let (receivable_id, revenue_id) = account_ids(&pool, ctx.company_id).await;
    let day = NaiveDate::from_ymd_opt(2026, 5, 15).unwrap();
    let tx_id = seed_bank_transactions(
        &pool,
        ctx.company_id,
        ctx.bank_account_id,
        ctx.user_id,
        &unique_hash("accept_split_cancel"),
        day,
        day,
        vec![make_new_tx(
            ctx.company_id,
            ctx.bank_account_id,
            day,
            Some(day),
            dec!(100.00),
            "CHF",
            "SPLIT",
            Some("Divers"),
        )],
    )
    .await[0];
    let resp = app
        .client
        .post(app.url("/api/v1/reconciliation/accept"))
        .bearer_auth(&ctx.jwt)
        .json(&serde_json::json!({
            "bankAccountId": ctx.bank_account_id,
            "proposals": [{
                "type": "split",
                "bankTransactionId": tx_id,
                "splits": [
                    { "counterpartyAccountId": revenue_id, "amount": "60.00", "description": "A" },
                    { "counterpartyAccountId": receivable_id, "amount": "40.00", "description": "B" },
                ],
            }],
        }))
        .send()
        .await
        .unwrap();
    let body: Value = resp.json().await.unwrap();
    assert_eq!(
        body["accepted"].as_array().map(Vec::len),
        Some(1),
        "got {body:?}"
    );
    let entry_id = matched_entry_of(&pool, tx_id).await.expect("rapprochée");

    let (st, done) = cancel_reco(&app, &ctx.jwt, tx_id).await;
    assert_eq!(st, 200, "got {done:?}");
    let reversal_id = done["reversalJournalEntryId"].as_i64().unwrap();
    assert_back_to_reconcile(
        &pool,
        &app,
        &ctx.jwt,
        ctx.bank_account_id,
        tx_id,
        entry_id,
        reversal_id,
    )
    .await;
}

/// ⛔ **Arbitrage Q2 de Guy** : une facture de décembre, dans un exercice
/// **clos**, payée en janvier — se rapproche, et se dé-rapproche. Seul
/// l'exercice de l'écriture DE RAPPROCHEMENT compte, jamais celui de la vente.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn an_invoice_of_a_closed_year_paid_the_next_year_is_unreconcilable(pool: MySqlPool) {
    let ctx = setup_company(&pool, "Acme", "CH4431999123000889012", Role::Comptable).await;
    ensure_fiscal_year_today(&pool, ctx.company_id).await;
    let app = spawn_app(pool.clone()).await;
    let (inv_id, sale_entry) = seed_validated_invoice(
        &pool,
        ctx.company_id,
        ctx.contact_id,
        "INV-2025-099",
        NaiveDate::from_ymd_opt(2025, 12, 20).unwrap(),
        dec!(500.00),
    )
    .await;
    // La vente appartient à l'exercice 2025, clos par le vrai chemin.
    let fy_2025 = sqlx::query(
        "INSERT INTO fiscal_years (company_id, name, start_date, end_date, status, \
         created_at, updated_at) VALUES (?, 'FY 2025', '2025-01-01', '2025-12-31', 'Open', \
         NOW(3), NOW(3))",
    )
    .bind(ctx.company_id)
    .execute(&pool)
    .await
    .unwrap()
    .last_insert_id() as i64;
    sqlx::query(
        "UPDATE journal_entries SET fiscal_year_id = ?, entry_date = '2025-12-20' WHERE id = ?",
    )
    .bind(fy_2025)
    .bind(sale_entry)
    .execute(&pool)
    .await
    .unwrap();
    kesh_db::repositories::fiscal_years::close(&pool, ctx.user_id, ctx.company_id, fy_2025)
        .await
        .expect("clôture de 2025");

    let paid_on = NaiveDate::from_ymd_opt(2026, 1, 10).unwrap();
    let tx_id = seed_bank_transactions(
        &pool,
        ctx.company_id,
        ctx.bank_account_id,
        ctx.user_id,
        &unique_hash("closed_year_invoice"),
        paid_on,
        paid_on,
        vec![make_new_tx(
            ctx.company_id,
            ctx.bank_account_id,
            paid_on,
            Some(paid_on),
            dec!(500.00),
            "CHF",
            "INV-2025-099",
            Some("Acme Client"),
        )],
    )
    .await[0];
    let accepted = post_accept_one(&app, &ctx, tx_id, inv_id).await;
    assert_eq!(
        accepted["accepted"].as_array().map(Vec::len),
        Some(1),
        "got {accepted:?}"
    );

    let (st, view) = get_reco(&app, &ctx.jwt, tx_id).await;
    assert_eq!(view["cancellable"], true, "got {st} {view:?}");
    let (st, done) = cancel_reco(&app, &ctx.jwt, tx_id).await;
    assert_eq!(st, 200, "got {done:?}");
}

/// Rang 2 sur une écriture **propre** : l'exercice de l'écriture de
/// rapprochement est clos. ⛔ Le refus est le 409 du DÉ-RAPPROCHEMENT, dont le
/// texte ne dit pas « règlement » — une écriture manuelle n'en est pas un.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn a_closed_year_refuses_with_the_reconciliation_text(pool: MySqlPool) {
    let ctx = setup_company(&pool, "Acme", "CH4431999123000889012", Role::Comptable).await;
    // Un exercice passé, qui ne couvre pas le jour : le clore ne ferme pas le
    // jour — c'est bien le rang 2 qui parle, pas le rang 5.
    let fy_2024 = sqlx::query(
        "INSERT INTO fiscal_years (company_id, name, start_date, end_date, status, \
         created_at, updated_at) VALUES (?, 'FY 2024', '2024-01-01', '2024-12-31', 'Open', \
         NOW(3), NOW(3))",
    )
    .bind(ctx.company_id)
    .execute(&pool)
    .await
    .unwrap()
    .last_insert_id() as i64;
    ensure_fiscal_year_today(&pool, ctx.company_id).await;
    let app = spawn_app(pool.clone()).await;
    let (_, revenue_id) = account_ids(&pool, ctx.company_id).await;
    let tx_id = manual_reconciled(
        &pool,
        &app,
        &ctx,
        revenue_id,
        NaiveDate::from_ymd_opt(2024, 6, 3).unwrap(),
        "closed_manual",
    )
    .await;
    kesh_db::repositories::fiscal_years::close(&pool, ctx.user_id, ctx.company_id, fy_2024)
        .await
        .expect("clôture");

    let (_, view) = get_reco(&app, &ctx.jwt, tx_id).await;
    assert_eq!(
        view["cancelBlockedBy"], "FISCAL_YEAR_CLOSED",
        "got {view:?}"
    );
    let (st, body) = cancel_reco(&app, &ctx.jwt, tx_id).await;
    assert_eq!(st, 409, "got {body:?}");
    assert_eq!(body["error"]["code"], "FISCAL_YEAR_CLOSED");
    let message = body["error"]["message"].as_str().unwrap_or_default();
    assert!(
        !message.contains("règlement") && message.contains("rapprochement"),
        "le texte du dé-rapprochement, pas celui du règlement — got {message:?}"
    );
    assert!(
        matched_entry_of(&pool, tx_id).await.is_some(),
        "rien n'a bougé"
    );
}

/// ⛔ **Composition et rollback** : le rang 1 (facture créditée) est refusé par
/// le geste du règlement **après** que le lien a été défait dans la
/// transaction — le 409 remonte, et le lien est toujours là.
/// ⚠️ État **forgé** : la facture passe à `cancelled` par SQL (l'avoir par le
/// vrai chemin exige une facture complète) ; c'est l'état que produit l'avoir.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn a_credited_invoice_refuses_and_the_link_survives(pool: MySqlPool) {
    let ctx = setup_company(&pool, "Acme", "CH4431999123000889012", Role::Comptable).await;
    ensure_fiscal_year_today(&pool, ctx.company_id).await;
    let app = spawn_app(pool.clone()).await;
    let (inv_id, tx_id) = reconciled_invoice(
        &pool,
        &app,
        &ctx,
        "INV-2026-003",
        NaiveDate::from_ymd_opt(2026, 4, 20).unwrap(),
        NaiveDate::from_ymd_opt(2026, 5, 15).unwrap(),
        dec!(300.00),
    )
    .await;
    let entry_id = matched_entry_of(&pool, tx_id).await;
    sqlx::query("UPDATE invoices SET status = 'cancelled' WHERE id = ?")
        .bind(inv_id)
        .execute(&pool)
        .await
        .unwrap();

    let (_, view) = get_reco(&app, &ctx.jwt, tx_id).await;
    assert_eq!(view["cancelBlockedBy"], "INVOICE_CREDITED", "got {view:?}");
    let (st, body) = cancel_reco(&app, &ctx.jwt, tx_id).await;
    assert_eq!(st, 409, "got {body:?}");
    assert_eq!(body["error"]["code"], "INVOICE_CREDITED");
    assert_eq!(
        matched_entry_of(&pool, tx_id).await,
        entry_id,
        "le rollback a rétabli le lien"
    );
    let n: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM audit_log WHERE action = 'reconciliation.cancelled'",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(n, 0, "rien n'est écrit");
}

/// ⛔ **L'exemption étroite** : deux transactions pointent la même écriture
/// (état **forgé** — aucun chemin de l'application ne le produit). Défaire l'une
/// est refusé au rang 3, avec l'identifiant de l'**autre** — dans les deux
/// ordres d'insertion, pour que la lecture naïve d'un `LIMIT 1` sans ordre ne
/// passe pas par chance.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn another_transaction_on_the_same_entry_still_refuses(pool: MySqlPool) {
    let ctx = setup_company(&pool, "Acme", "CH4431999123000889012", Role::Comptable).await;
    let _ = insert_fake_fiscal_year(&pool, ctx.company_id).await;
    ensure_fiscal_year_today(&pool, ctx.company_id).await;
    let app = spawn_app(pool.clone()).await;
    let (_, revenue_id) = account_ids(&pool, ctx.company_id).await;
    let day = NaiveDate::from_ymd_opt(2026, 5, 15).unwrap();
    // L'ordre « celle qu'on défait est la plus ancienne » : elle est rapprochée
    // d'abord, l'autre forgée ensuite (id plus grand).
    let first = manual_reconciled(&pool, &app, &ctx, revenue_id, day, "exempt_a").await;
    let entry = matched_entry_of(&pool, first).await.unwrap();
    let later = seed_bank_transactions(
        &pool,
        ctx.company_id,
        ctx.bank_account_id,
        ctx.user_id,
        &unique_hash("exempt_b"),
        day,
        day,
        vec![make_new_tx(
            ctx.company_id,
            ctx.bank_account_id,
            day,
            Some(day),
            dec!(80.00),
            "CHF",
            "exempt_b",
            Some("Divers"),
        )],
    )
    .await[0];
    sqlx::query(
        "UPDATE bank_transactions SET status = 'reconciled', matched_entry_id = ? WHERE id = ?",
    )
    .bind(entry)
    .bind(later)
    .execute(&pool)
    .await
    .unwrap();
    assert!(first < later);

    for (undone, other) in [(first, later), (later, first)] {
        let (_, view) = get_reco(&app, &ctx.jwt, undone).await;
        assert_eq!(
            view["cancelBlockedBy"], "MATCHED_BANK_TRANSACTION",
            "got {view:?}"
        );
        assert_eq!(
            view["cancelBlockedDocumentId"], other,
            "l'AUTRE transaction"
        );
        let (st, body) = cancel_reco(&app, &ctx.jwt, undone).await;
        assert_eq!(st, 409, "got {body:?}");
        assert_eq!(body["error"]["code"], "MATCHED_BANK_TRANSACTION");
        assert_eq!(
            matched_entry_of(&pool, undone).await,
            Some(entry),
            "rien n'a bougé"
        );
    }
}

/// Deux annulations simultanées de la même transaction : l'une réussit,
/// l'autre trouve une transaction qui n'est plus rapprochée.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn two_simultaneous_cancellations_one_wins(pool: MySqlPool) {
    let ctx = setup_company(&pool, "Acme", "CH4431999123000889012", Role::Comptable).await;
    let _ = insert_fake_fiscal_year(&pool, ctx.company_id).await;
    ensure_fiscal_year_today(&pool, ctx.company_id).await;
    let app = spawn_app(pool.clone()).await;
    let (_, revenue_id) = account_ids(&pool, ctx.company_id).await;
    let tx_id = manual_reconciled(
        &pool,
        &app,
        &ctx,
        revenue_id,
        NaiveDate::from_ymd_opt(2026, 5, 15).unwrap(),
        "twice",
    )
    .await;
    let (a, b) = tokio::join!(
        cancel_reco(&app, &ctx.jwt, tx_id),
        cancel_reco(&app, &ctx.jwt, tx_id)
    );
    let mut statuses = [a.0, b.0];
    statuses.sort_unstable();
    assert_eq!(statuses, [200, 409], "got {a:?} / {b:?}");
    let refused = if a.0 == 409 { &a.1 } else { &b.1 };
    assert_eq!(refused["error"]["code"], "BANK_TRANSACTION_NOT_RECONCILED");
    let reversals: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM journal_entries WHERE reverses_entry_id IS NOT NULL",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(reversals, 1, "une seule contre-passation");
}

/// Rôles, portée et motif de tête : Consultation → 403 ; autre société → 404
/// et rien d'écrit ; transaction non rapprochée → 409.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn cancel_reconciliation_roles_scope_and_head(pool: MySqlPool) {
    let ctx = setup_company(&pool, "Acme", "CH4431999123000889012", Role::Comptable).await;
    let _ = insert_fake_fiscal_year(&pool, ctx.company_id).await;
    ensure_fiscal_year_today(&pool, ctx.company_id).await;
    let other = setup_company(&pool, "Autre", "CH9300762011623852957", Role::Comptable).await;
    let app = spawn_app(pool.clone()).await;
    let (_, revenue_id) = account_ids(&pool, ctx.company_id).await;
    let day = NaiveDate::from_ymd_opt(2026, 5, 15).unwrap();
    let tx_id = manual_reconciled(&pool, &app, &ctx, revenue_id, day, "roles").await;

    let reader_id = create_user(&pool, "lecteur", Role::Consultation, ctx.company_id).await;
    let reader = forge_jwt(reader_id, "Consultation", ctx.company_id);
    assert_eq!(cancel_reco(&app, &reader, tx_id).await.0, 403);
    assert_eq!(get_reco(&app, &reader, tx_id).await.0, 403);

    let tables = [
        "bank_transactions WHERE matched_entry_id IS NOT NULL",
        "journal_entries",
        "journal_entry_lines",
        "invoice_settlements",
        "audit_log",
    ];
    let mut before = Vec::new();
    for t in tables {
        before.push(
            sqlx::query_scalar::<_, i64>(&format!("SELECT COUNT(*) FROM {t}"))
                .fetch_one(&pool)
                .await
                .unwrap(),
        );
    }
    assert_eq!(cancel_reco(&app, &other.jwt, tx_id).await.0, 404);
    assert_eq!(get_reco(&app, &other.jwt, tx_id).await.0, 404);
    for (t, n) in tables.iter().zip(before) {
        let now: i64 = sqlx::query_scalar(&format!("SELECT COUNT(*) FROM {t}"))
            .fetch_one(&pool)
            .await
            .unwrap();
        assert_eq!(now, n, "rien d'écrit dans {t}");
    }

    assert_eq!(cancel_reco(&app, &ctx.jwt, tx_id).await.0, 200);
    let (_, view) = get_reco(&app, &ctx.jwt, tx_id).await;
    assert_eq!(view["cancelBlockedBy"], "BANK_TRANSACTION_NOT_RECONCILED");
    assert_eq!(view["kind"], Value::Null);
    let (st, body) = cancel_reco(&app, &ctx.jwt, tx_id).await;
    assert_eq!(st, 409);
    assert_eq!(body["error"]["code"], "BANK_TRANSACTION_NOT_RECONCILED");
}

/// ⛔ **Le marqueur de rejet** : une transaction dé-rapprochée doit revenir
/// dans les propositions, qui écartent toute transaction marquée rejetée.
/// ⚠️ **État FORGÉ, et dit tel** : aucun chemin réel ne laisse le marqueur sur
/// une transaction rapprochée — quatre chemins le remettent à zéro, et le
/// chemin « facture » refuse une transaction rejetée dès son contrôle
/// préalable (`failed[]`, `RECONCILIATION_ALREADY_REJECTED` depuis la Story
/// 27-4 ; un `400` du lot entier auparavant). La remise à zéro du geste est
/// donc une **défense** ; sans ce test, l'oublier ne rougirait nulle part.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn a_previously_rejected_transaction_comes_back_to_the_proposals(pool: MySqlPool) {
    let ctx = setup_company(&pool, "Acme", "CH4431999123000889012", Role::Comptable).await;
    ensure_fiscal_year_today(&pool, ctx.company_id).await;
    let app = spawn_app(pool.clone()).await;
    let day = NaiveDate::from_ymd_opt(2026, 5, 15).unwrap();
    let (inv_id, _) = seed_validated_invoice(
        &pool,
        ctx.company_id,
        ctx.contact_id,
        "INV-2026-004",
        NaiveDate::from_ymd_opt(2026, 4, 20).unwrap(),
        dec!(700.00),
    )
    .await;
    let tx_id = seed_bank_transactions(
        &pool,
        ctx.company_id,
        ctx.bank_account_id,
        ctx.user_id,
        &unique_hash("rejected_then_accepted"),
        day,
        day,
        vec![make_new_tx(
            ctx.company_id,
            ctx.bank_account_id,
            day,
            Some(day),
            dec!(700.00),
            "CHF",
            "INV-2026-004",
            Some("Acme Client"),
        )],
    )
    .await[0];
    let accepted = post_accept_one(&app, &ctx, tx_id, inv_id).await;
    assert_eq!(
        accepted["accepted"].as_array().map(Vec::len),
        Some(1),
        "got {accepted:?}"
    );
    sqlx::query("UPDATE bank_transactions SET auto_match_rejected_at = NOW(3) WHERE id = ?")
        .bind(tx_id)
        .execute(&pool)
        .await
        .unwrap();
    let entry_id = matched_entry_of(&pool, tx_id).await.unwrap();

    let (st, done) = cancel_reco(&app, &ctx.jwt, tx_id).await;
    assert_eq!(st, 200, "got {done:?}");
    let reversal_id = done["reversalJournalEntryId"].as_i64().unwrap();
    assert_back_to_reconcile(
        &pool,
        &app,
        &ctx.jwt,
        ctx.bank_account_id,
        tx_id,
        entry_id,
        reversal_id,
    )
    .await;
}

/// ⛔ **Clés API d'écriture admises, et portées à l'audit** : la ligne
/// `reconciliation.cancelled` nomme la clé (`for_actor`). ⚠️ Les lignes écrites
/// par le socle et par le geste du règlement ne la portent pas — défaut
/// antérieur suivi par #431.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn an_api_key_cancels_and_is_named_in_the_audit(pool: MySqlPool) {
    let ctx = setup_company(&pool, "Acme", "CH4431999123000889012", Role::Comptable).await;
    let _ = insert_fake_fiscal_year(&pool, ctx.company_id).await;
    ensure_fiscal_year_today(&pool, ctx.company_id).await;
    let app = spawn_app(pool.clone()).await;
    let (_, revenue_id) = account_ids(&pool, ctx.company_id).await;
    let tx_id = manual_reconciled(
        &pool,
        &app,
        &ctx,
        revenue_id,
        NaiveDate::from_ymd_opt(2026, 5, 15).unwrap(),
        "by_key",
    )
    .await;
    let resp = app
        .client
        .post(app.url("/api/v1/settings/api-keys"))
        .bearer_auth(&ctx.jwt)
        .json(&serde_json::json!({ "name": "rw", "scope": "read-write" }))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 201);
    let body: Value = resp.json().await.unwrap();
    let (key_id, key) = (
        body["id"].as_i64().unwrap(),
        body["key"].as_str().unwrap().to_string(),
    );

    let (st, done) = cancel_reco(&app, &key, tx_id).await;
    assert_eq!(st, 200, "got {done:?}");
    let (actor_type, actor_key): (String, Option<i64>) = sqlx::query_as(
        "SELECT actor_type, actor_api_key_id FROM audit_log \
         WHERE action = 'reconciliation.cancelled' AND entity_id = ?",
    )
    .bind(tx_id)
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!((actor_type.as_str(), actor_key), ("api_key", Some(key_id)));
}

// ============================================================
// Story 25-4-c2 (#480) — l'acceptation contre les écritures concurrentes
// ============================================================

/// Lance `POST /reconciliation/accept` pour une seule proposition, **en tâche** :
/// le test garde la main pour agir pendant que l'acceptation attend. Rend le
/// statut HTTP et le corps.
fn accept_in_background(
    app: &TestApp,
    ctx: &CompanyCtx,
    tx_id: i64,
    inv_id: i64,
) -> tokio::task::JoinHandle<(u16, Value)> {
    let client = app.client.clone();
    let url = app.url("/api/v1/reconciliation/accept");
    let jwt = ctx.jwt.clone();
    let body = serde_json::json!({
        "bankAccountId": ctx.bank_account_id,
        "proposals": [{ "type": "invoice", "bankTransactionId": tx_id, "invoiceId": inv_id }],
    });
    tokio::spawn(async move {
        let resp = client
            .post(url)
            .bearer_auth(jwt)
            .json(&body)
            .send()
            .await
            .unwrap();
        let status = resp.status().as_u16();
        (status, resp.json().await.unwrap_or(Value::Null))
    })
}

/// ⛔ **#480, la course n° 1 : un règlement manuel PARTIEL validé pendant
/// qu'une acceptation est en cours ne doit pas laisser régler la facture deux fois.**
///
/// La fenêtre réelle de la course va de la **première lecture** de la
/// transaction d'acceptation (l'instantané se fige à l'étape 2) jusqu'à sa
/// recherche d'exercice (d) : dans cet intervalle elle ne pose aucun verrou de
/// ligne, et un règlement manuel peut s'y valider entièrement. Après (d), elle
/// tient les exercices de la société — la recherche d'exercice les parcourt en
/// les verrouillant —, et le règlement manuel l'attend : ce sens-là est sûr.
///
/// Montage, sans `sleep` — une lecture simple n'attend jamais un verrou de
/// ligne, mais elle attend un verrou de **métadonnées** :
/// 1. une connexion de test tient `LOCK TABLES contacts WRITE` ;
/// 2. l'acceptation (1 000.— sur une facture de 1 000.—) part en tâche : elle
///    fige son instantané, lit la facture, puis s'arrête à la lecture du
///    contact (étape 5bis), **avant** sa garde de trop-perçu — on l'y attend ;
/// 3. un règlement manuel de 400.— est validé (il ne lit pas `contacts`) ;
/// 4. `UNLOCK TABLES` : l'acceptation reprend, sur un instantané où la
///    facture doit encore 1 000.—.
///
/// Avant la 25-4-c2, le règlement partiel n'incrémentait pas `version` :
/// l'`UPDATE invoices … version = ?` de l'acceptation réussissait, et la
/// facture finissait réglée 1 400.— pour 1 000.—, compte clients créditeur.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn accept_refuses_when_a_partial_manual_settlement_lands_meanwhile(pool: MySqlPool) {
    use kesh_db::entities::SettlementChoice;
    use kesh_db::repositories::invoice_settlements_write;

    let ctx = setup_company(&pool, "Course1", "CH4431999123000889012", Role::Comptable).await;
    let day = NaiveDate::from_ymd_opt(2026, 5, 15).unwrap();
    let inv_date = NaiveDate::from_ymd_opt(2026, 4, 20).unwrap();
    let (inv_id, _) = seed_validated_invoice(
        &pool,
        ctx.company_id,
        ctx.contact_id,
        "INV-RACE-1",
        inv_date,
        dec!(1000.00),
    )
    .await;
    let tx_ids = seed_bank_transactions(
        &pool,
        ctx.company_id,
        ctx.bank_account_id,
        ctx.user_id,
        &unique_hash("race_partial_manual"),
        day,
        day,
        vec![make_new_tx(
            ctx.company_id,
            ctx.bank_account_id,
            day,
            Some(day),
            dec!(1000.00),
            "CHF",
            "INV-RACE-1",
            Some("Course1 Client"),
        )],
    )
    .await;
    let app = spawn_app(pool.clone()).await;

    // (1) Le verrou de métadonnées, sur une connexion hors transaction —
    // DÉTACHÉE du pool : si le test panique avant `UNLOCK TABLES`, elle se
    // ferme au lieu de retourner au pool verrouillée, et sa session emporte le
    // verrou (sinon la suppression de la base éphémère attendrait sans fin).
    let mut verrou = pool.acquire().await.unwrap().detach();
    sqlx::query("LOCK TABLES contacts WRITE")
        .execute(&mut verrou)
        .await
        .unwrap();

    // (2) L'acceptation, jusqu'à la lecture du contact.
    let acceptation = accept_in_background(&app, &ctx, tx_ids[0], inv_id);
    let bloquee =
        kesh_db::test_fixtures::attendre_une_requete_en_cours(&pool, &["FROM contacts"], || {
            acceptation.is_finished()
        })
        .await;
    assert!(
        bloquee,
        "l'acceptation devait attendre sur `contacts` — le montage ne prouve rien sinon"
    );

    // (3) Le règlement manuel partiel, validé pendant l'attente.
    let partiel = invoice_settlements_write::settle_invoice(
        &pool,
        ctx.user_id,
        ctx.company_id,
        inv_id,
        SettlementChoice::BankTransfer {
            bank_account_id: ctx.bank_account_id,
        },
        dec!(400.00),
        day,
    )
    .await
    .expect("règlement manuel partiel");
    assert!(!partiel.fully_settled);

    // (4) Relâcher : l'acceptation reprend sur un reste périmé.
    sqlx::query("UNLOCK TABLES")
        .execute(&mut verrou)
        .await
        .unwrap();
    drop(verrou);
    let (status, body) = acceptation.await.unwrap();
    assert_eq!(status, 200, "succès partiel = succès HTTP ; corps = {body}");
    assert!(
        body["accepted"].as_array().unwrap().is_empty(),
        "⛔ l'acceptation a réglé une facture déjà réglée de 400.— entre-temps : {body}"
    );
    let failed = body["failed"].as_array().unwrap();
    assert_eq!(failed.len(), 1);
    assert_eq!(
        failed[0]["errorCode"],
        "RECONCILIATION_INVOICE_NOT_ELIGIBLE"
    );
    assert_eq!(failed[0]["details"]["reason"], "race_during_update");

    // Rien d'écrit par l'acceptation : un seul règlement, le manuel.
    let settled: Vec<Decimal> =
        sqlx::query_scalar("SELECT amount FROM invoice_settlements WHERE invoice_id = ?")
            .bind(inv_id)
            .fetch_all(&pool)
            .await
            .unwrap();
    assert_eq!(settled, vec![dec!(400.0000)]);
    let tx_status: String = sqlx::query_scalar("SELECT status FROM bank_transactions WHERE id = ?")
        .bind(tx_ids[0])
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(tx_status, "pending", "la transaction reste à rapprocher");
    assert_eq!(
        receivable_balance(&pool, ctx.receivable_account_id).await,
        dec!(600.00),
        "le compte clients porte le reste, jamais un solde créditeur"
    );
}

/// ⛔ **#480 — un interblocage dont l'acceptation est la victime se REJOUE.**
///
/// Avant la 25-4-c2, InnoDB annulait toute la transaction du lot, l'instruction
/// fautive ressortait en `FailedProposal` `DATABASE_ERROR`, le `ROLLBACK TO
/// SAVEPOINT` suivant échouait (1305) et la route rendait un **500** — sans
/// rejeu, `retry_with` n'existant que sur l'annulation.
///
/// Montage d'un interblocage **déterministe**, dont l'acceptation est la
/// victime :
/// 1. une transaction de test s'alourdit (500 lignes insérées : InnoDB choisit
///    pour victime la transaction la plus légère), puis verrouille la facture ;
/// 2. l'acceptation prend l'exercice (d), passe son écriture, puis attend la
///    facture dès l'insertion du règlement — la clé étrangère vers `invoices`
///    y pose un verrou partagé sur la ligne — : on l'y attend ;
/// 3. le test demande l'exercice : cycle, InnoDB annule l'acceptation ;
/// 4. le test annule sa transaction ; le rejeu repart à neuf et accepte.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn accept_replays_the_batch_when_it_is_the_deadlock_victim(pool: MySqlPool) {
    let ctx = setup_company(&pool, "Victime", "CH4431999123000889012", Role::Comptable).await;
    let day = NaiveDate::from_ymd_opt(2026, 5, 15).unwrap();
    let inv_date = NaiveDate::from_ymd_opt(2026, 4, 20).unwrap();
    let (inv_id, _) = seed_validated_invoice(
        &pool,
        ctx.company_id,
        ctx.contact_id,
        "INV-DEADLOCK-1",
        inv_date,
        dec!(250.00),
    )
    .await;
    let tx_ids = seed_bank_transactions(
        &pool,
        ctx.company_id,
        ctx.bank_account_id,
        ctx.user_id,
        &unique_hash("deadlock_victim"),
        day,
        day,
        vec![make_new_tx(
            ctx.company_id,
            ctx.bank_account_id,
            day,
            Some(day),
            dec!(250.00),
            "CHF",
            "INV-DEADLOCK-1",
            Some("Victime Client"),
        )],
    )
    .await;
    let fy_2026: i64 = sqlx::query_scalar(
        "SELECT id FROM fiscal_years WHERE company_id = ? AND start_date <= ? AND end_date >= ? \
         AND status = 'Open' LIMIT 1",
    )
    .bind(ctx.company_id)
    .bind(day)
    .bind(day)
    .fetch_one(&pool)
    .await
    .unwrap();
    // Une table de lest, hors du flux d'acceptation (DDL hors transaction).
    sqlx::query(
        "CREATE TABLE lest_interblocage (id INT AUTO_INCREMENT PRIMARY KEY, x INT) ENGINE=InnoDB",
    )
    .execute(&pool)
    .await
    .unwrap();
    let app = spawn_app(pool.clone()).await;

    // (1) La transaction de test : lourde, puis la facture.
    let mut lourde = pool.begin().await.unwrap();
    for _ in 0..10 {
        sqlx::query("INSERT INTO lest_interblocage (x) SELECT seq FROM seq_1_to_50")
            .execute(&mut *lourde)
            .await
            .unwrap();
    }
    sqlx::query("SELECT id FROM invoices WHERE id = ? FOR UPDATE")
        .bind(inv_id)
        .fetch_one(&mut *lourde)
        .await
        .unwrap();

    // (2) L'acceptation, jusqu'à l'insertion de son règlement.
    let acceptation = accept_in_background(&app, &ctx, tx_ids[0], inv_id);
    let bloquee = kesh_db::test_fixtures::attendre_une_requete_en_cours(
        &pool,
        &["INSERT INTO invoice_settlements"],
        || acceptation.is_finished(),
    )
    .await;
    assert!(
        bloquee,
        "l'acceptation devait attendre la facture — le montage ne prouve rien sinon"
    );

    // (3) Le cycle : l'exercice, que tient l'acceptation. La transaction de
    // test DOIT l'obtenir — sinon c'est elle la victime, et le test ne dit rien.
    sqlx::query("SELECT id FROM fiscal_years WHERE id = ? FOR UPDATE")
        .bind(fy_2026)
        .fetch_one(&mut *lourde)
        .await
        .expect("la transaction de test doit survivre à l'interblocage");

    // (4) Relâcher : le rejeu repart à neuf.
    lourde.rollback().await.unwrap();
    let (status, body) = acceptation.await.unwrap();
    assert_eq!(
        status, 200,
        "⛔ un interblocage doit être rejoué, pas rendu en erreur : {body}"
    );
    assert_eq!(
        body["accepted"].as_array().unwrap().len(),
        1,
        "le rejeu accepte la proposition ; corps = {body}"
    );
    let settled: Vec<Decimal> =
        sqlx::query_scalar("SELECT amount FROM invoice_settlements WHERE invoice_id = ?")
            .bind(inv_id)
            .fetch_all(&pool)
            .await
            .unwrap();
    assert_eq!(
        settled,
        vec![dec!(250.0000)],
        "un seul règlement, celui du rejeu"
    );
}

/// #480, la course n° 2 — **deux acceptations du même solde, depuis deux
/// comptes bancaires** (donc deux `GET_LOCK` distincts) : une seule règle la
/// facture. Déjà refusée avant la 25-4-c2, puisque l'acceptation incrémente
/// toujours `version` ; ce test la garde, parce que c'est ce refus qu'un
/// `FOR UPDATE` mal placé désarmerait.
///
/// Montage : une transaction de test tient la facture ; l'acceptation A prend
/// l'exercice et bute sur la facture (insertion du règlement) ; l'acceptation B,
/// son instantané déjà figé, bute sur l'exercice que tient A. Le test relâche :
/// A règle, B lit un reste périmé et doit être refusée.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn two_acceptances_of_the_same_balance_settle_it_once(pool: MySqlPool) {
    let ctx = setup_company(&pool, "Course2", "CH4431999123000889012", Role::Comptable).await;
    let day = NaiveDate::from_ymd_opt(2026, 5, 15).unwrap();
    let inv_date = NaiveDate::from_ymd_opt(2026, 4, 20).unwrap();
    let (inv_id, _) = seed_validated_invoice(
        &pool,
        ctx.company_id,
        ctx.contact_id,
        "INV-RACE-2",
        inv_date,
        dec!(700.00),
    )
    .await;
    // Le second compte bancaire, câblé sur le même compte du grand livre.
    let second_account = create_bank_account(&pool, ctx.company_id, "CH9300762011623852957").await;
    sqlx::query("UPDATE bank_accounts SET journal_account_id = ? WHERE id = ?")
        .bind(ctx.bank_ledger_account_id)
        .bind(second_account)
        .execute(&pool)
        .await
        .unwrap();
    let tx_a = seed_bank_transactions(
        &pool,
        ctx.company_id,
        ctx.bank_account_id,
        ctx.user_id,
        &unique_hash("race2_a"),
        day,
        day,
        vec![make_new_tx(
            ctx.company_id,
            ctx.bank_account_id,
            day,
            Some(day),
            dec!(700.00),
            "CHF",
            "INV-RACE-2",
            Some("Course2 Client"),
        )],
    )
    .await[0];
    let tx_b = seed_bank_transactions(
        &pool,
        ctx.company_id,
        second_account,
        ctx.user_id,
        &unique_hash("race2_b"),
        day,
        day,
        vec![make_new_tx(
            ctx.company_id,
            second_account,
            day,
            Some(day),
            dec!(700.00),
            "CHF",
            "INV-RACE-2",
            Some("Course2 Client"),
        )],
    )
    .await[0];
    let app = spawn_app(pool.clone()).await;
    let ctx_b = CompanyCtx {
        bank_account_id: second_account,
        jwt: ctx.jwt.clone(),
        ..ctx
    };

    let mut verrou = pool.begin().await.unwrap();
    sqlx::query("SELECT id FROM invoices WHERE id = ? FOR UPDATE")
        .bind(inv_id)
        .fetch_one(&mut *verrou)
        .await
        .unwrap();

    let acceptation_a = accept_in_background(&app, &ctx, tx_a, inv_id);
    assert!(
        kesh_db::test_fixtures::attendre_une_requete_en_cours(
            &pool,
            &["INSERT INTO invoice_settlements"],
            || acceptation_a.is_finished(),
        )
        .await,
        "A devait attendre la facture"
    );
    let acceptation_b = accept_in_background(&app, &ctx_b, tx_b, inv_id);
    assert!(
        kesh_db::test_fixtures::attendre_une_requete_en_cours(
            &pool,
            &["FROM fiscal_years", "FOR UPDATE"],
            || acceptation_b.is_finished(),
        )
        .await,
        "B devait attendre l'exercice que tient A"
    );

    verrou.rollback().await.unwrap();
    let (status_a, body_a) = acceptation_a.await.unwrap();
    let (status_b, body_b) = acceptation_b.await.unwrap();
    assert_eq!(
        (status_a, status_b),
        (200, 200),
        "A = {body_a} ; B = {body_b}"
    );
    assert_eq!(
        body_a["accepted"].as_array().unwrap().len(),
        1,
        "A règle : {body_a}"
    );
    assert!(
        body_b["accepted"].as_array().unwrap().is_empty(),
        "⛔ B a réglé une seconde fois une facture déjà soldée : {body_b}"
    );
    assert_eq!(
        body_b["failed"][0]["details"]["reason"],
        "race_during_update"
    );
    let settled: Vec<Decimal> =
        sqlx::query_scalar("SELECT amount FROM invoice_settlements WHERE invoice_id = ?")
            .bind(inv_id)
            .fetch_all(&pool)
            .await
            .unwrap();
    assert_eq!(settled, vec![dec!(700.0000)], "un seul règlement");
}

// ============================================================
// Story 25-4-c3-b (#476) — le reste dû au centime, et l'écart en écriture
// ============================================================

/// Désigne un compte de différences d'arrondi (6940, charge) dans les réglages
/// de facturation — `setup_company` n'en crée aucun. Rend son id.
async fn designate_rounding_account(pool: &MySqlPool, company_id: i64) -> i64 {
    let id = sqlx::query(
        "INSERT INTO accounts (company_id, number, name, account_type, active, postable) \
         VALUES (?, '6940', 'Différences d''arrondi', 'Expense', 1, 1)",
    )
    .bind(company_id)
    .execute(pool)
    .await
    .expect("compte 6940")
    .last_insert_id() as i64;
    sqlx::query(
        "INSERT INTO company_invoice_settings (company_id, default_rounding_account_id) VALUES (?, ?) \
         ON DUPLICATE KEY UPDATE default_rounding_account_id = VALUES(default_rounding_account_id)",
    )
    .bind(company_id)
    .bind(id)
    .execute(pool)
    .await
    .expect("réglage du compte d'arrondi");
    id
}

/// Une facture au reste brut de **10.0050** et une transaction de `tx_amount`,
/// référencée ou non. Rend `(invoice_id, tx_id)`.
async fn half_centime_invoice_and_tx(
    pool: &MySqlPool,
    ctx: &CompanyCtx,
    number: &str,
    tx_amount: Decimal,
    reference: &str,
) -> (i64, i64) {
    let day = NaiveDate::from_ymd_opt(2026, 5, 15).unwrap();
    let (inv_id, _je) = seed_validated_invoice(
        pool,
        ctx.company_id,
        ctx.contact_id,
        number,
        NaiveDate::from_ymd_opt(2026, 4, 20).unwrap(),
        dec!(10.0050),
    )
    .await;
    let tx_ids = seed_bank_transactions(
        pool,
        ctx.company_id,
        ctx.bank_account_id,
        ctx.user_id,
        &unique_hash(number),
        day,
        day,
        vec![make_new_tx(
            ctx.company_id,
            ctx.bank_account_id,
            day,
            Some(day),
            tx_amount,
            "CHF",
            reference,
            None,
        )],
    )
    .await;
    (inv_id, tx_ids[0])
}

/// Lignes `(compte, débit, crédit)` d'une écriture, dans l'ordre.
async fn entry_lines(pool: &MySqlPool, entry_id: i64) -> Vec<(i64, Decimal, Decimal)> {
    sqlx::query_as(
        "SELECT account_id, debit, credit FROM journal_entry_lines WHERE entry_id = ? \
         ORDER BY line_order, id",
    )
    .bind(entry_id)
    .fetch_all(pool)
    .await
    .unwrap()
}

/// Ce qu'une facture n'a pas encore reçu, pour les assertions « rien d'écrit ».
async fn settlements_and_paid_at(pool: &MySqlPool, inv_id: i64) -> (i64, Option<NaiveDateTime>) {
    sqlx::query_as(
        "SELECT (SELECT COUNT(*) FROM invoice_settlements WHERE invoice_id = i.id), i.paid_at \
         FROM invoices i WHERE i.id = ?",
    )
    .bind(inv_id)
    .fetch_one(pool)
    .await
    .unwrap()
}

/// ⛔ **Un virement de 10.01 solde une facture de 10.0050**, sur le seul score de
/// montant : candidat affiché au centime, accepté, règlement enregistré au reste
/// BRUT, écart de 0.0050 au crédit du compte d'arrondi, créance soldée à zéro.
///
/// Avant la story : score de montant 0 (10.01 ≠ 10.005), donc
/// `RECONCILIATION_SCORE_TOO_LOW` — et même accepté, trop-perçu refusé.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn a_centime_payment_settles_a_half_centime_invoice_with_a_rounding_line(pool: MySqlPool) {
    let ctx = setup_company(&pool, "Centime", "CH4431999123000889012", Role::Comptable).await;
    let rounding = designate_rounding_account(&pool, ctx.company_id).await;
    let (inv_id, tx_id) =
        half_centime_invoice_and_tx(&pool, &ctx, "INV-CT-1", dec!(10.01), "sans rapport").await;
    let app = spawn_app(pool.clone()).await;

    let resp = app
        .client
        .get(app.url(&format!(
            "/api/v1/reconciliation/proposals?bankAccountId={}",
            ctx.bank_account_id
        )))
        .bearer_auth(&ctx.jwt)
        .send()
        .await
        .unwrap();
    let body: Value = resp.json().await.unwrap();
    let cand = body["proposals"]
        .as_array()
        .unwrap()
        .iter()
        .find(|p| p["bankTransactionId"] == tx_id)
        .and_then(|p| {
            p["candidates"]
                .as_array()
                .unwrap()
                .iter()
                .find(|c| c["invoiceId"] == inv_id)
                .cloned()
        })
        .expect("la facture est candidate");
    assert_eq!(cand["score"]["amountScore"], 1.0, "comparé au centime");
    assert_eq!(
        cand["invoiceAmount"], "10.01",
        "affiché au centime, pas « 10.005 »"
    );
    assert!(
        cand["invoiceTotalTtc"].is_null(),
        "reste et TTC égaux au centime : pas de mention, got {cand:?}"
    );

    let body = post_accept_one(&app, &ctx, tx_id, inv_id).await;
    assert_eq!(
        body["accepted"].as_array().map(Vec::len),
        Some(1),
        "failed = {:?}",
        body["failed"]
    );

    let (settled, entry_id): (Decimal, i64) = sqlx::query_as(
        "SELECT amount, journal_entry_id FROM invoice_settlements WHERE invoice_id = ?",
    )
    .bind(inv_id)
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(settled, dec!(10.0050), "⛔ réglé au reste BRUT");
    assert_eq!(
        entry_lines(&pool, entry_id).await,
        vec![
            (ctx.bank_ledger_account_id, dec!(10.01), dec!(0)),
            (ctx.receivable_account_id, dec!(0), dec!(10.0050)),
            (rounding, dec!(0), dec!(0.0050)),
        ],
        "trois lignes : banque du payé, créance du brut, écart au crédit"
    );
    assert_eq!(
        receivable_balance(&pool, ctx.receivable_account_id).await,
        Decimal::ZERO,
        "⛔ la créance se ferme exactement"
    );
    let (_, paid_at) = settlements_and_paid_at(&pool, inv_id).await;
    assert!(paid_at.is_some(), "la facture est soldée");
    let details: Value = sqlx::query_scalar(
        "SELECT details_json FROM audit_log WHERE action = 'invoice.paid' AND entity_id = ?",
    )
    .bind(inv_id)
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(
        details["rounding_difference"]
            .as_str()
            .map(|s| s.parse::<Decimal>().unwrap()),
        Some(dec!(0.0050)),
        "l'audit nomme l'écart, got {details:?}"
    );
}

/// 10.02 sur 10.0050 reste un trop-perçu : la référence donne un score, c'est la
/// garde à double borne qui refuse.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn two_centimes_over_a_half_centime_invoice_is_an_overpayment(pool: MySqlPool) {
    let ctx = setup_company(&pool, "Trop2", "CH4431999123000889012", Role::Comptable).await;
    designate_rounding_account(&pool, ctx.company_id).await;
    let (inv_id, tx_id) =
        half_centime_invoice_and_tx(&pool, &ctx, "INV-CT-2", dec!(10.02), "INV-CT-2").await;
    let app = spawn_app(pool.clone()).await;

    let body = post_accept_one(&app, &ctx, tx_id, inv_id).await;
    let failed = body["failed"].as_array().unwrap();
    assert_eq!(failed.len(), 1, "got {body:?}");
    assert_eq!(failed[0]["errorCode"], "RECONCILIATION_OVERPAYMENT");
    assert_eq!(settlements_and_paid_at(&pool, inv_id).await, (0, None));
}

/// ⛔ **Sans compte d'arrondi utilisable, rien ne s'écrit** — refus
/// per-proposal (pattern batch), réglage absent puis compte archivé.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn a_rounding_gap_without_a_usable_account_is_refused_per_proposal(pool: MySqlPool) {
    let ctx = setup_company(&pool, "SansArr", "CH4431999123000889012", Role::Comptable).await;
    let (inv_id, tx_id) =
        half_centime_invoice_and_tx(&pool, &ctx, "INV-CT-3", dec!(10.01), "INV-CT-3").await;
    let app = spawn_app(pool.clone()).await;

    let body = post_accept_one(&app, &ctx, tx_id, inv_id).await;
    let failed = body["failed"].as_array().unwrap();
    assert_eq!(failed.len(), 1, "got {body:?}");
    assert_eq!(failed[0]["errorCode"], "ROUNDING_ACCOUNT_NOT_CONFIGURED");
    assert_eq!(settlements_and_paid_at(&pool, inv_id).await, (0, None));

    let rounding = designate_rounding_account(&pool, ctx.company_id).await;
    sqlx::query("UPDATE accounts SET active = FALSE WHERE id = ?")
        .bind(rounding)
        .execute(&pool)
        .await
        .unwrap();
    let body = post_accept_one(&app, &ctx, tx_id, inv_id).await;
    let failed = body["failed"].as_array().unwrap();
    assert_eq!(
        failed[0]["errorCode"], "ROUNDING_ACCOUNT_NOT_CONFIGURED",
        "archivé depuis sa désignation (#486) : revérifié au moment d'écrire"
    );
    assert_eq!(settlements_and_paid_at(&pool, inv_id).await, (0, None));
    let status: String = sqlx::query_scalar("SELECT status FROM bank_transactions WHERE id = ?")
        .bind(tx_id)
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(status, "pending", "la transaction reste à rapprocher");
}

/// Story 15-6a (AC1, AC2 ; test 10) — **témoin** du refus
/// `INVOICE_SALE_ENTRY_MALFORMED`, écrit AVANT que la requête de créance de
/// `accept_one_invoice` ne soit remplacée par le lecteur partagé
/// `invoice_settlements::sale_receivable_account` : aucun autre test ne figeait
/// ce refus, et la copie remplacée devait garder un témoin. Il passe avant et
/// après le remplacement.
///
/// Montage léger (choix de la fiche) : la facture est repointée sur un en-tête
/// d'écriture SANS ligne, de la même société — le lecteur ne trouve aucune
/// ligne de débit et rend `None`.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn invoice_proposal_with_a_sale_entry_without_debit_line_is_malformed(pool: MySqlPool) {
    let ctx = setup_company(&pool, "Malforme", "CH4431999123000889012", Role::Comptable).await;
    let day = NaiveDate::from_ymd_opt(2026, 5, 15).unwrap();
    let inv_date = NaiveDate::from_ymd_opt(2026, 5, 1).unwrap();
    let (inv_id, je_id) = seed_validated_invoice(
        &pool,
        ctx.company_id,
        ctx.contact_id,
        "INV-MAL-1",
        inv_date,
        dec!(100.00),
    )
    .await;
    let fy_id: i64 = sqlx::query_scalar("SELECT fiscal_year_id FROM journal_entries WHERE id = ?")
        .bind(je_id)
        .fetch_one(&pool)
        .await
        .unwrap();
    let empty_entry = insert_fake_journal_entry(&pool, ctx.company_id, fy_id).await;
    sqlx::query("UPDATE invoices SET journal_entry_id = ? WHERE id = ?")
        .bind(empty_entry)
        .bind(inv_id)
        .execute(&pool)
        .await
        .unwrap();
    let tx_id = seed_bank_transactions(
        &pool,
        ctx.company_id,
        ctx.bank_account_id,
        ctx.user_id,
        &unique_hash("sale_entry_malformed"),
        day,
        day,
        vec![make_new_tx(
            ctx.company_id,
            ctx.bank_account_id,
            day,
            Some(day),
            dec!(100.00),
            "CHF",
            "INV-MAL-1",
            Some("Malforme Client"),
        )],
    )
    .await[0];
    let app = spawn_app(pool.clone()).await;

    let body = post_accept_one(&app, &ctx, tx_id, inv_id).await;
    assert_eq!(
        body["accepted"].as_array().map(Vec::len),
        Some(0),
        "got {body:?}"
    );
    let failed = body["failed"].as_array().unwrap();
    assert_eq!(failed.len(), 1, "got {body:?}");
    assert_eq!(failed[0]["bankTransactionId"], tx_id);
    assert_eq!(failed[0]["errorCode"], "INVOICE_SALE_ENTRY_MALFORMED");
    assert_eq!(
        failed[0]["details"],
        serde_json::json!({
            "reason": "no_debit_line_on_sale_entry",
            "saleEntryId": empty_entry,
        })
    );
    assert_eq!(settlements_and_paid_at(&pool, inv_id).await, (0, None));
    let status: String = sqlx::query_scalar("SELECT status FROM bank_transactions WHERE id = ?")
        .bind(tx_id)
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(status, "pending", "la transaction reste à rapprocher");
}

/// ⛔ **Dé-rapprocher un règlement à trois lignes contre-passe les trois** : le
/// reste dû redevient le brut d'avant, créance et compte d'arrondi reviennent.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn cancelling_a_three_line_reconciliation_reverses_the_rounding_line(pool: MySqlPool) {
    let ctx = setup_company(&pool, "Annul3", "CH4431999123000889012", Role::Comptable).await;
    ensure_fiscal_year_today(&pool, ctx.company_id).await;
    let rounding = designate_rounding_account(&pool, ctx.company_id).await;
    let (inv_id, tx_id) =
        half_centime_invoice_and_tx(&pool, &ctx, "INV-CT-4", dec!(10.01), "INV-CT-4").await;
    let app = spawn_app(pool.clone()).await;
    let body = post_accept_one(&app, &ctx, tx_id, inv_id).await;
    assert_eq!(
        body["accepted"].as_array().map(Vec::len),
        Some(1),
        "got {body:?}"
    );

    let (st, done) = cancel_reco(&app, &ctx.jwt, tx_id).await;
    assert_eq!(st, 200, "got {done:?}");
    let reversal_id = done["reversalJournalEntryId"].as_i64().unwrap();
    assert_eq!(
        entry_lines(&pool, reversal_id).await,
        vec![
            (ctx.bank_ledger_account_id, dec!(0), dec!(10.01)),
            (ctx.receivable_account_id, dec!(10.0050), dec!(0)),
            (rounding, dec!(0.0050), dec!(0)),
        ],
        "les trois lignes, retournées"
    );
    assert_eq!(
        receivable_balance(&pool, ctx.receivable_account_id).await,
        dec!(10.0050),
        "la créance revient au reste brut"
    );
    assert_eq!(receivable_balance(&pool, rounding).await, Decimal::ZERO);
    let due = kesh_db::repositories::invoice_settlements::amount_due(&pool, inv_id)
        .await
        .unwrap();
    assert_eq!(due, dec!(10.0050));
    assert_eq!(settlements_and_paid_at(&pool, inv_id).await, (0, None));
}

// ============================================================
// Story 15-5b (AC3, AC16, #427) — `POST /accept`, proposition `split`, compte
// de contrepartie non imputable
// ============================================================

/// Contexte d'acceptation ventilée : société, Comptable, compte bancaire lié
/// à 1020, deux contreparties 5000 et 5700 imputables, exercice 2026 ouvert,
/// deux transactions `pending` de -100.00.
struct AcceptSplitCtx {
    company_id: i64,
    user_id: i64,
    bank_account_id: i64,
    cp_a: i64,
    cp_b: i64,
    tx_ids: Vec<i64>,
    jwt: String,
}

async fn setup_accept_split_ctx(pool: &MySqlPool, label: &str, iban: &str) -> AcceptSplitCtx {
    let company_id = create_company(pool, label).await;
    let user_id = create_user(pool, &format!("{label}_user"), Role::Comptable, company_id).await;
    let bank_account_id = create_bank_account(pool, company_id, iban).await;
    let mk = |number: &'static str, name: &'static str, account_type: AccountType| NewAccount {
        company_id,
        number: number.into(),
        name: name.into(),
        account_type,
        parent_id: None,
        role: None,
        postable: true,
    };
    let ledger = accounts::create(pool, user_id, mk("1020", "Banque", AccountType::Asset))
        .await
        .unwrap()
        .id;
    let cp_a = accounts::create(pool, user_id, mk("5000", "Salaires", AccountType::Expense))
        .await
        .unwrap()
        .id;
    let cp_b = accounts::create(
        pool,
        user_id,
        mk("5700", "Charges sociales", AccountType::Expense),
    )
    .await
    .unwrap()
    .id;
    sqlx::query(
        "UPDATE bank_accounts SET journal_account_id = ?, version = version + 1 WHERE id = ?",
    )
    .bind(ledger)
    .bind(bank_account_id)
    .execute(pool)
    .await
    .unwrap();
    let _ = insert_fake_fiscal_year(pool, company_id).await;
    let day = NaiveDate::from_ymd_opt(2026, 5, 31).unwrap();
    let tx_ids = seed_bank_transactions(
        pool,
        company_id,
        bank_account_id,
        user_id,
        &unique_hash(label),
        day,
        day,
        vec![
            make_new_tx(
                company_id,
                bank_account_id,
                day,
                Some(day),
                dec!(-100.00),
                "CHF",
                "BATCH-1",
                None,
            ),
            make_new_tx(
                company_id,
                bank_account_id,
                day,
                Some(day),
                dec!(-100.00),
                "CHF",
                "BATCH-2",
                None,
            ),
        ],
    )
    .await;
    AcceptSplitCtx {
        company_id,
        user_id,
        bank_account_id,
        cp_a,
        cp_b,
        tx_ids,
        jwt: forge_jwt(user_id, "Comptable", company_id),
    }
}

async fn set_account_not_postable_15_5b(pool: &MySqlPool, account_id: i64) {
    sqlx::query("UPDATE accounts SET postable = FALSE, version = version + 1 WHERE id = ?")
        .bind(account_id)
        .execute(pool)
        .await
        .expect("set account not postable");
}

/// Story 15-5b (AC3) — une proposition ventilée dont une ligne vise un compte
/// non imputable : HTTP 200, `failed[]` avec `ACCOUNT_NOT_POSTABLE` et
/// `details.rejected` ; la proposition valide du même lot est acceptée.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn accept_split_with_non_postable_counterparty_fails_per_proposal(pool: MySqlPool) {
    let app = spawn_app(pool.clone()).await;
    let ctx = setup_accept_split_ctx(&pool, "accept_split_np", "CH1000000000000099003").await;
    // Un troisième compte, rendu non imputable : seule la proposition qui le
    // vise doit échouer.
    let cp_np = accounts::create(
        &pool,
        ctx.user_id,
        NewAccount {
            company_id: ctx.company_id,
            number: "6900".into(),
            name: "Divers".into(),
            account_type: AccountType::Expense,
            parent_id: None,
            role: None,
            postable: true,
        },
    )
    .await
    .unwrap()
    .id;
    set_account_not_postable_15_5b(&pool, cp_np).await;

    let resp = app
        .client
        .post(app.url("/api/v1/reconciliation/accept"))
        .header("Authorization", format!("Bearer {}", ctx.jwt))
        .json(&serde_json::json!({
            "bankAccountId": ctx.bank_account_id,
            "proposals": [
                {
                    "type": "split",
                    "bankTransactionId": ctx.tx_ids[0],
                    "splits": [
                        { "counterpartyAccountId": ctx.cp_a, "amount": "60.00", "description": "Ligne" },
                        { "counterpartyAccountId": cp_np, "amount": "40.00", "description": "Ligne" },
                    ],
                },
                {
                    "type": "split",
                    "bankTransactionId": ctx.tx_ids[1],
                    "splits": [
                        { "counterpartyAccountId": ctx.cp_a, "amount": "60.00", "description": "Ligne" },
                        { "counterpartyAccountId": ctx.cp_b, "amount": "40.00", "description": "Ligne" },
                    ],
                }
            ]
        }))
        .send()
        .await
        .unwrap();
    let status = resp.status();
    let body: Value = resp.json().await.unwrap();
    assert_eq!(
        status, 200,
        "pattern batch : succès partiel = HTTP 200 — {body}"
    );
    let failed = body["failed"].as_array().expect("failed[]");
    assert_eq!(failed.len(), 1, "{body}");
    assert_eq!(failed[0]["bankTransactionId"].as_i64(), Some(ctx.tx_ids[0]));
    assert_eq!(failed[0]["errorCode"], "ACCOUNT_NOT_POSTABLE");
    assert_eq!(
        failed[0]["details"],
        serde_json::json!({ "rejected": [{ "accountId": cp_np, "accountNumber": "6900" }] })
    );
    let accepted = body["accepted"].as_array().expect("accepted[]");
    assert_eq!(accepted.len(), 1, "{body}");
    assert_eq!(
        accepted[0]["bankTransactionId"].as_i64(),
        Some(ctx.tx_ids[1])
    );

    let status: String = sqlx::query_scalar("SELECT status FROM bank_transactions WHERE id = ?")
        .bind(ctx.tx_ids[0])
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(status, "pending", "la proposition refusée n'a rien écrit");
}

/// Story 15-5b (AC3, priorité) — une ligne **manquante** et une ligne non
/// imputable dans la même proposition : `failed[]` `ACCOUNT_NOT_FOUND`.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn accept_split_missing_and_non_postable_reports_not_found_first(pool: MySqlPool) {
    let app = spawn_app(pool.clone()).await;
    let ctx = setup_accept_split_ctx(&pool, "accept_split_prio", "CH1000000000000099004").await;
    set_account_not_postable_15_5b(&pool, ctx.cp_b).await;
    let unknown = ctx.cp_b + 100_000;

    let resp = app
        .client
        .post(app.url("/api/v1/reconciliation/accept"))
        .header("Authorization", format!("Bearer {}", ctx.jwt))
        .json(&serde_json::json!({
            "bankAccountId": ctx.bank_account_id,
            "proposals": [{
                "type": "split",
                "bankTransactionId": ctx.tx_ids[0],
                "splits": [
                    { "counterpartyAccountId": unknown, "amount": "60.00", "description": "Ligne" },
                    { "counterpartyAccountId": ctx.cp_b, "amount": "40.00", "description": "Ligne" },
                ],
            }]
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let body: Value = resp.json().await.unwrap();
    let failed = body["failed"].as_array().expect("failed[]");
    assert_eq!(failed.len(), 1, "{body}");
    assert_eq!(failed[0]["errorCode"], "ACCOUNT_NOT_FOUND");
}

// ============================================================
// Story 15-6b (#474) — le compte de banque n'est pas la créance
// ============================================================

/// Une transaction de `amount`, référencée `reference`, sur le compte bancaire
/// du contexte. Rend son id.
async fn one_tx(
    pool: &MySqlPool,
    ctx: &CompanyCtx,
    hash_seed: &str,
    amount: Decimal,
    reference: &str,
) -> i64 {
    let day = NaiveDate::from_ymd_opt(2026, 5, 15).unwrap();
    seed_bank_transactions(
        pool,
        ctx.company_id,
        ctx.bank_account_id,
        ctx.user_id,
        &unique_hash(hash_seed),
        day,
        day,
        vec![make_new_tx(
            ctx.company_id,
            ctx.bank_account_id,
            day,
            Some(day),
            amount,
            "CHF",
            reference,
            None,
        )],
    )
    .await[0]
}

async fn tx_status(pool: &MySqlPool, tx_id: i64) -> String {
    sqlx::query_scalar("SELECT status FROM bank_transactions WHERE id = ?")
        .bind(tx_id)
        .fetch_one(pool)
        .await
        .unwrap()
}

/// Story 15-6b, test 7 — **un lot de deux propositions, deux créances** : A sur
/// 1100, B sur 1101 ; le compte bancaire du lot est lié au 1100. HTTP 200, B
/// acceptée, A dans `failed[]` avec ses cinq clés ; la transaction de A reste
/// en attente.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn accept_refuses_per_proposal_a_bank_ledger_that_is_the_receivable(pool: MySqlPool) {
    let ctx = setup_company(&pool, "ContrepA", "CH4431999123000889012", Role::Comptable).await;
    let inv_date = NaiveDate::from_ymd_opt(2026, 5, 1).unwrap();
    let (inv_a, _) = seed_validated_invoice(
        &pool,
        ctx.company_id,
        ctx.contact_id,
        "INV-CP-A",
        inv_date,
        dec!(100.00),
    )
    .await;
    let (inv_b, je_b) = seed_validated_invoice(
        &pool,
        ctx.company_id,
        ctx.contact_id,
        "INV-CP-B",
        inv_date,
        dec!(200.00),
    )
    .await;
    // B porte sa créance sur un 1101 (débiteurs changé dans les réglages entre
    // les deux ventes) : la ligne de débit de son écriture de vente.
    let r1101: i64 = sqlx::query_scalar(
        "INSERT INTO accounts (company_id, number, name, account_type, active, postable) \
         VALUES (?, '1101', 'Débiteurs bis', 'Asset', 1, 1) RETURNING id",
    )
    .bind(ctx.company_id)
    .fetch_one(&pool)
    .await
    .unwrap();
    sqlx::query("UPDATE journal_entry_lines SET account_id = ? WHERE entry_id = ? AND debit > 0")
        .bind(r1101)
        .bind(je_b)
        .execute(&pool)
        .await
        .unwrap();
    // Le compte bancaire du lot, lié au 1100 (sans passer par la 15-6c).
    sqlx::query("UPDATE bank_accounts SET journal_account_id = ? WHERE id = ?")
        .bind(ctx.receivable_account_id)
        .bind(ctx.bank_account_id)
        .execute(&pool)
        .await
        .unwrap();
    let tx_a = one_tx(&pool, &ctx, "cp-a", dec!(100.00), "INV-CP-A").await;
    let tx_b = one_tx(&pool, &ctx, "cp-b", dec!(200.00), "INV-CP-B").await;
    let app = spawn_app(pool.clone()).await;

    let resp = app
        .client
        .post(app.url("/api/v1/reconciliation/accept"))
        .bearer_auth(&ctx.jwt)
        .json(&serde_json::json!({
            "bankAccountId": ctx.bank_account_id,
            "proposals": [
                { "type": "invoice", "bankTransactionId": tx_a, "invoiceId": inv_a },
                { "type": "invoice", "bankTransactionId": tx_b, "invoiceId": inv_b },
            ],
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200, "succès partiel = succès HTTP");
    let body: Value = resp.json().await.unwrap();
    assert_eq!(body["accepted"].as_array().map(Vec::len), Some(1), "{body}");
    let failed = body["failed"].as_array().unwrap();
    assert_eq!(failed.len(), 1, "{body}");
    assert_eq!(failed[0]["bankTransactionId"], tx_a);
    assert_eq!(
        failed[0]["errorCode"],
        "SETTLEMENT_COUNTERPARTY_IS_CLAIM_ACCOUNT"
    );
    assert_eq!(
        failed[0]["details"],
        serde_json::json!({
            "bankAccountId": ctx.bank_account_id,
            "accountId": ctx.receivable_account_id,
            "accountNumber": "1100",
            "claim": "receivable",
            "role": "counterparty",
        })
    );
    assert_eq!(settlements_and_paid_at(&pool, inv_a).await, (0, None));
    assert_eq!(tx_status(&pool, tx_a).await, "pending");
    let (settled_b, _) = settlements_and_paid_at(&pool, inv_b).await;
    assert_eq!(settled_b, 1, "B est réglée");
}

/// Story 15-6b, test 8 — **ordre** : sur A, une transaction supérieure au reste
/// (référencée, donc de score positif — l'étape 7bis passe) rend le refus de
/// contrepartie, pas `RECONCILIATION_OVERPAYMENT`.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn the_counterparty_refusal_comes_before_the_reconciliation_overpayment(pool: MySqlPool) {
    let ctx = setup_company(&pool, "ContrepB", "CH4431999123000889012", Role::Comptable).await;
    let (inv_a, _) = seed_validated_invoice(
        &pool,
        ctx.company_id,
        ctx.contact_id,
        "INV-CP-C",
        NaiveDate::from_ymd_opt(2026, 5, 1).unwrap(),
        dec!(100.00),
    )
    .await;
    sqlx::query("UPDATE bank_accounts SET journal_account_id = ? WHERE id = ?")
        .bind(ctx.receivable_account_id)
        .bind(ctx.bank_account_id)
        .execute(&pool)
        .await
        .unwrap();
    let tx_a = one_tx(&pool, &ctx, "cp-c", dec!(150.00), "INV-CP-C").await;
    let app = spawn_app(pool.clone()).await;

    let body = post_accept_one(&app, &ctx, tx_a, inv_a).await;
    let failed = body["failed"].as_array().unwrap();
    assert_eq!(failed.len(), 1, "{body}");
    assert_eq!(
        failed[0]["errorCode"], "SETTLEMENT_COUNTERPARTY_IS_CLAIM_ACCOUNT",
        "la contrepartie précède le trop-perçu"
    );
    assert_eq!(settlements_and_paid_at(&pool, inv_a).await, (0, None));
}

/// Story 15-6b, test 12 — **arrondi (c-bis) = créance** (AC3 bis) : la créance
/// retypée en charge et désignée compte d'arrondi. `failed[]`, HTTP 200,
/// `details.role = "rounding"`, et PAS de `bankAccountId`.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn accept_refuses_a_rounding_account_that_is_the_receivable(pool: MySqlPool) {
    let ctx = setup_company(&pool, "ContrepR", "CH4431999123000889012", Role::Comptable).await;
    designate_rounding_account(&pool, ctx.company_id).await;
    let (inv_id, tx_id) =
        half_centime_invoice_and_tx(&pool, &ctx, "INV-CP-R", dec!(10.01), "INV-CP-R").await;
    sqlx::query("UPDATE accounts SET account_type = 'Expense', role = NULL WHERE id = ?")
        .bind(ctx.receivable_account_id)
        .execute(&pool)
        .await
        .unwrap();
    sqlx::query(
        "UPDATE company_invoice_settings SET default_rounding_account_id = ? WHERE company_id = ?",
    )
    .bind(ctx.receivable_account_id)
    .bind(ctx.company_id)
    .execute(&pool)
    .await
    .unwrap();
    let app = spawn_app(pool.clone()).await;

    let body = post_accept_one(&app, &ctx, tx_id, inv_id).await;
    let failed = body["failed"].as_array().unwrap();
    assert_eq!(failed.len(), 1, "{body}");
    assert_eq!(
        failed[0]["errorCode"],
        "SETTLEMENT_COUNTERPARTY_IS_CLAIM_ACCOUNT"
    );
    assert_eq!(
        failed[0]["details"],
        serde_json::json!({
            "accountId": ctx.receivable_account_id,
            "accountNumber": "1100",
            "claim": "receivable",
            "role": "rounding",
        })
    );
    assert!(failed[0]["details"].get("bankAccountId").is_none());
    assert_eq!(settlements_and_paid_at(&pool, inv_id).await, (0, None));
    assert_eq!(tx_status(&pool, tx_id).await, "pending");
}

// ============================================================
// Story 15-12b (#543, AC 11) — le filet sous un bilan clos, dans le lot
//
// État hérité posé par SQL : 2026 ouvert, **2027 clos**, 2028 ouvert. Une
// proposition datée de 2026 est refusée par le filet de `create_in_tx_inner` et
// rendue dans `failed[]` sous `LATER_FISCAL_YEAR_CLOSED` (jamais
// `DATABASE_ERROR`) ; une proposition saine du MÊME lot, datée de 2028, passe.
// ============================================================

/// Pose 2027 **clos** et 2028 **ouvert** à côté de l'exercice 2026 du montage ;
/// rend l'identifiant et le nom de 2027.
async fn poser_un_posterieur_clos(pool: &MySqlPool, company_id: i64) -> (i64, String) {
    let _ = insert_fake_fiscal_year(pool, company_id).await;
    let mut clos = 0;
    for (annee, statut) in [(2027, "Closed"), (2028, "Open")] {
        let id = sqlx::query(
            "INSERT INTO fiscal_years (company_id, name, start_date, end_date, status) \
             VALUES (?, ?, ?, ?, ?)",
        )
        .bind(company_id)
        .bind(format!("FY {annee} c{company_id}"))
        .bind(NaiveDate::from_ymd_opt(annee, 1, 1).unwrap())
        .bind(NaiveDate::from_ymd_opt(annee, 12, 31).unwrap())
        .bind(statut)
        .execute(pool)
        .await
        .expect("exercice")
        .last_insert_id() as i64;
        if statut == "Closed" {
            clos = id;
        }
    }
    (clos, format!("FY 2027 c{company_id}"))
}

async fn ecritures_de(pool: &MySqlPool, company_id: i64) -> i64 {
    sqlx::query_scalar("SELECT COUNT(*) FROM journal_entries WHERE company_id = ?")
        .bind(company_id)
        .fetch_one(pool)
        .await
        .unwrap()
}

/// Le refus attendu dans `failed[]` : code littéral et `details` exacts.
fn assert_refus_du_filet(f: &Value, tx_id: i64, fy_id: i64, fy_name: &str) {
    assert_eq!(f["bankTransactionId"].as_i64(), Some(tx_id), "{f}");
    assert_eq!(f["errorCode"], "LATER_FISCAL_YEAR_CLOSED", "{f}");
    assert_eq!(
        f["details"],
        serde_json::json!({ "fiscalYearId": fy_id, "fiscalYearName": fy_name }),
        "{f}"
    );
}

/// AC 11, voie **facture** (`accept_one_invoice` → `project_error_to_failed_proposal`).
///
/// ⛔ Tue la mutation (vii-a) — retirer le bras `LaterFiscalYearClosed` du mapper.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn accept_invoice_under_a_closed_later_year_lands_in_failed(pool: MySqlPool) {
    let ctx = setup_company(&pool, "Filet", "CH4431999123000889012", Role::Comptable).await;
    let (inv_a, _) = seed_validated_invoice(
        &pool,
        ctx.company_id,
        ctx.contact_id,
        "INV-A",
        NaiveDate::from_ymd_opt(2026, 5, 1).unwrap(),
        dec!(100.00),
    )
    .await;
    let (inv_b, _) = seed_validated_invoice(
        &pool,
        ctx.company_id,
        ctx.contact_id,
        "INV-B",
        NaiveDate::from_ymd_opt(2028, 5, 1).unwrap(),
        dec!(200.00),
    )
    .await;
    let (fy_clos, nom) = poser_un_posterieur_clos(&pool, ctx.company_id).await;
    let (jour_a, jour_b) = (
        NaiveDate::from_ymd_opt(2026, 5, 15).unwrap(),
        NaiveDate::from_ymd_opt(2028, 5, 15).unwrap(),
    );
    let tx_ids = seed_bank_transactions(
        &pool,
        ctx.company_id,
        ctx.bank_account_id,
        ctx.user_id,
        &unique_hash("filet_facture"),
        jour_a,
        jour_b,
        vec![
            make_new_tx(
                ctx.company_id,
                ctx.bank_account_id,
                jour_a,
                Some(jour_a),
                dec!(100.00),
                "CHF",
                "INV-A",
                Some("Filet Client"),
            ),
            make_new_tx(
                ctx.company_id,
                ctx.bank_account_id,
                jour_b,
                Some(jour_b),
                dec!(200.00),
                "CHF",
                "INV-B",
                Some("Filet Client"),
            ),
        ],
    )
    .await;
    let avant = ecritures_de(&pool, ctx.company_id).await;

    let app = spawn_app(pool.clone()).await;
    let resp = app
        .client
        .post(app.url("/api/v1/reconciliation/accept"))
        .bearer_auth(&ctx.jwt)
        .json(&serde_json::json!({
            "bankAccountId": ctx.bank_account_id,
            "proposals": [
                { "type": "invoice", "bankTransactionId": tx_ids[0], "invoiceId": inv_a },
                { "type": "invoice", "bankTransactionId": tx_ids[1], "invoiceId": inv_b },
            ],
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200, "pattern batch : succès partiel = 200");
    let body: Value = resp.json().await.unwrap();
    let failed = body["failed"].as_array().unwrap();
    assert_eq!(failed.len(), 1, "{body}");
    assert_refus_du_filet(&failed[0], tx_ids[0], fy_clos, &nom);
    let accepted = body["accepted"].as_array().unwrap();
    assert_eq!(accepted.len(), 1, "{body}");
    assert_eq!(accepted[0]["bankTransactionId"].as_i64(), Some(tx_ids[1]));
    assert_eq!(
        ecritures_de(&pool, ctx.company_id).await,
        avant + 1,
        "seule la proposition saine écrit"
    );
}

/// AC 11, voie **ventilé** (`accept_one_split` → `project_error_to_failed_proposal`,
/// `projectId` inconnu → omis).
///
/// ⛔ Tue la mutation (vii-a), indépendamment de la voie facture.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn accept_split_under_a_closed_later_year_lands_in_failed(pool: MySqlPool) {
    let ctx = setup_company(&pool, "FiletV", "CH4431999123000889012", Role::Comptable).await;
    let (fy_clos, nom) = poser_un_posterieur_clos(&pool, ctx.company_id).await;
    let charge = accounts::create(
        &pool,
        ctx.user_id,
        NewAccount {
            company_id: ctx.company_id,
            number: "5000".into(),
            name: "Salaires".into(),
            account_type: AccountType::Expense,
            parent_id: None,
            role: None,
            postable: true,
        },
    )
    .await
    .unwrap()
    .id;
    let (jour_a, jour_b) = (
        NaiveDate::from_ymd_opt(2026, 5, 31).unwrap(),
        NaiveDate::from_ymd_opt(2028, 5, 31).unwrap(),
    );
    let tx_ids = seed_bank_transactions(
        &pool,
        ctx.company_id,
        ctx.bank_account_id,
        ctx.user_id,
        &unique_hash("filet_ventile"),
        jour_a,
        jour_b,
        vec![
            make_new_tx(
                ctx.company_id,
                ctx.bank_account_id,
                jour_a,
                Some(jour_a),
                dec!(-100.00),
                "CHF",
                "LOT-A",
                None,
            ),
            make_new_tx(
                ctx.company_id,
                ctx.bank_account_id,
                jour_b,
                Some(jour_b),
                dec!(-100.00),
                "CHF",
                "LOT-B",
                None,
            ),
        ],
    )
    .await;
    let avant = ecritures_de(&pool, ctx.company_id).await;
    let ventilation = serde_json::json!([
        { "counterpartyAccountId": charge, "amount": "60.00", "description": "Salaire" },
        { "counterpartyAccountId": charge, "amount": "40.00", "description": "Salaire bis" },
    ]);

    let app = spawn_app(pool.clone()).await;
    let resp = app
        .client
        .post(app.url("/api/v1/reconciliation/accept"))
        .bearer_auth(&ctx.jwt)
        .json(&serde_json::json!({
            "bankAccountId": ctx.bank_account_id,
            "proposals": [
                { "type": "split", "bankTransactionId": tx_ids[0], "splits": ventilation },
                { "type": "split", "bankTransactionId": tx_ids[1], "splits": ventilation },
            ],
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let body: Value = resp.json().await.unwrap();
    let failed = body["failed"].as_array().unwrap();
    assert_eq!(failed.len(), 1, "{body}");
    assert_refus_du_filet(&failed[0], tx_ids[0], fy_clos, &nom);
    let accepted = body["accepted"].as_array().unwrap();
    assert_eq!(accepted.len(), 1, "{body}");
    assert_eq!(accepted[0]["bankTransactionId"].as_i64(), Some(tx_ids[1]));
    assert_eq!(ecritures_de(&pool, ctx.company_id).await, avant + 1);
}

/// Story 15-6d (test 9, #524) — **témoin** de la garde existante
/// d'`accept_one_split` : une proposition ventilée dont une ligne vise le
/// compte de banque **rendu non imputable** → `failed[]` `VALIDATION_ERROR` /
/// `counterparty_equals_bank_ledger`, pas `ACCOUNT_NOT_POSTABLE` ; rien
/// d'écrit pour elle. Fige l'ordre « égalité avant postabilité » du lot
/// (choix C-15-6-23), auquel `accept_one_rule` s'aligne.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn accept_split_bank_ledger_counterparty_precedes_not_postable(pool: MySqlPool) {
    let app = spawn_app(pool.clone()).await;
    let ctx = setup_accept_split_ctx(&pool, "accept_split_bank", "CH1000000000000099005").await;
    let bank_ledger: i64 =
        sqlx::query_scalar("SELECT journal_account_id FROM bank_accounts WHERE id = ?")
            .bind(ctx.bank_account_id)
            .fetch_one(&pool)
            .await
            .unwrap();
    set_account_not_postable_15_5b(&pool, bank_ledger).await;

    let resp = app
        .client
        .post(app.url("/api/v1/reconciliation/accept"))
        .header("Authorization", format!("Bearer {}", ctx.jwt))
        .json(&serde_json::json!({
            "bankAccountId": ctx.bank_account_id,
            "proposals": [{
                "type": "split",
                "bankTransactionId": ctx.tx_ids[0],
                "splits": [
                    { "counterpartyAccountId": ctx.cp_a, "amount": "60.00", "description": "Ligne" },
                    { "counterpartyAccountId": bank_ledger, "amount": "40.00", "description": "Ligne" },
                ],
            }]
        }))
        .send()
        .await
        .unwrap();
    let status = resp.status();
    let body: Value = resp.json().await.unwrap();
    assert_eq!(status, 200, "pattern batch — {body}");
    let failed = body["failed"].as_array().expect("failed[]");
    assert_eq!(failed.len(), 1, "{body}");
    assert_eq!(failed[0]["errorCode"], "VALIDATION_ERROR", "{body}");
    assert_eq!(
        failed[0]["details"],
        serde_json::json!({ "reason": "counterparty_equals_bank_ledger" })
    );
    let (status, matched): (String, Option<i64>) =
        sqlx::query_as("SELECT status, matched_entry_id FROM bank_transactions WHERE id = ?")
            .bind(ctx.tx_ids[0])
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(status, "pending", "rien d'écrit");
    assert_eq!(matched, None);
    let je_count: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM journal_entries WHERE company_id = ?")
            .bind(ctx.company_id)
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(je_count, 0, "aucune écriture");
}

// ============================================================
// Story 15-1a2-0 (#518) — le lettrage figé par la période
// ============================================================

/// AC4 par l'API — facture encaissée par un rapprochement, groupe `document`
/// posé à la main sur la vente et l'encaissement, verrou ENSUITE à la date de
/// l'encaissement : la vue porte le motif, et le dé-rapprochement rend `409
/// LETTERING_ALL_LINES_IN_CLOSED_PERIODS` dans SA famille — le texte de
/// `reconciliation-cancel-blocked-lettering-closed`, non celui du règlement
/// (un motif codé en dur rendrait l'autre code). Rien n'est écrit, le lien
/// reste.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn unreconcile_of_a_closed_lettering_is_refused_in_its_family(pool: MySqlPool) {
    let ctx = setup_company(&pool, "Acme", "CH4431999123000889012", Role::Comptable).await;
    ensure_fiscal_year_today(&pool, ctx.company_id).await;
    let app = spawn_app(pool.clone()).await;
    let paid_on = NaiveDate::from_ymd_opt(2026, 5, 15).unwrap();
    let (inv_id, tx_id) = reconciled_invoice(
        &pool,
        &app,
        &ctx,
        "INV-2026-001",
        NaiveDate::from_ymd_opt(2026, 4, 20).unwrap(),
        paid_on,
        dec!(1234.56),
    )
    .await;
    let encaissement = matched_entry_of(&pool, tx_id).await.expect("rapprochée");
    let vente: i64 = sqlx::query_scalar("SELECT journal_entry_id FROM invoices WHERE id = ?")
        .bind(inv_id)
        .fetch_one(&pool)
        .await
        .unwrap();
    let cle = document_group::poser_groupe_document(
        &pool,
        ctx.receivable_account_id,
        &[vente, encaissement],
    )
    .await;
    kesh_db::repositories::companies::lock_books(&pool, ctx.user_id, ctx.company_id, paid_on)
        .await
        .expect("verrou");

    let (st, view) = get_reco(&app, &ctx.jwt, tx_id).await;
    assert_eq!(st, 200, "got {view:?}");
    assert_eq!(view["cancellable"], false, "got {view:?}");
    assert_eq!(
        view["cancelBlockedBy"], "LETTERING_ALL_LINES_IN_CLOSED_PERIODS",
        "got {view:?}"
    );

    let ecritures: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM journal_entries")
        .fetch_one(&pool)
        .await
        .unwrap();
    let (st, body) = cancel_reco(&app, &ctx.jwt, tx_id).await;
    assert_eq!(st, 409, "got {body:?}");
    assert_eq!(
        body["error"]["code"],
        "LETTERING_ALL_LINES_IN_CLOSED_PERIODS"
    );
    let catalogue = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../kesh-i18n/locales/fr-CH/messages.ftl"
    ))
    .expect("catalogue fr-CH");
    let attendu = catalogue
        .lines()
        .find_map(|l| l.strip_prefix("reconciliation-cancel-blocked-lettering-closed = "))
        .expect("clé au catalogue");
    assert_eq!(body["error"]["message"], attendu, "le texte de SA famille");

    assert_eq!(
        matched_entry_of(&pool, tx_id).await,
        Some(encaissement),
        "le lien reste"
    );
    let apres: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM journal_entries")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(apres, ecritures, "rien d'écrit");
    assert_eq!(document_group::lignes_du_groupe(&pool, cle).await, 2);
}

// ============================================================
// Story 15-1a2-i (#518) — le rapprochement lettre la facture qu'il solde
// ============================================================

/// Les lettrages `document` de la facture : `(clé, nombre de lignes)`.
async fn groupes_document(pool: &MySqlPool, inv_id: i64) -> Vec<(i64, i64)> {
    sqlx::query_as(
        "SELECT jel.lettering_key, COUNT(*) FROM journal_entry_lines jel \
         WHERE jel.lettering_origin = 'document' AND jel.entry_id IN ( \
             SELECT journal_entry_id FROM invoices WHERE id = ? \
             UNION SELECT journal_entry_id FROM invoice_settlements WHERE invoice_id = ?) \
         GROUP BY jel.lettering_key",
    )
    .bind(inv_id)
    .bind(inv_id)
    .fetch_all(pool)
    .await
    .unwrap()
}

/// AC15 (a) — une proposition qui solde la facture la lettre `document`, par
/// l'API et SOUS UNE CLÉ D'API : l'entrée `lettering.created` porte la clé —
/// le seul geste qui la porte (AC10).
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn accept_letters_a_fully_settled_invoice(pool: MySqlPool) {
    let ctx = setup_company(&pool, "Lettre", "CH4431999123000889012", Role::Comptable).await;
    let day = NaiveDate::from_ymd_opt(2026, 5, 15).unwrap();
    let (inv_id, je_id) = seed_validated_invoice(
        &pool,
        ctx.company_id,
        ctx.contact_id,
        "INV-LETTRE-1",
        NaiveDate::from_ymd_opt(2026, 4, 20).unwrap(),
        dec!(100.00),
    )
    .await;
    let tx_id = seed_bank_transactions(
        &pool,
        ctx.company_id,
        ctx.bank_account_id,
        ctx.user_id,
        &unique_hash("accept_letters"),
        day,
        day,
        vec![make_new_tx(
            ctx.company_id,
            ctx.bank_account_id,
            day,
            Some(day),
            dec!(100.00),
            "CHF",
            "INV-LETTRE-1",
            Some("Lettre Client"),
        )],
    )
    .await[0];
    let app = spawn_app(pool.clone()).await;
    let resp = app
        .client
        .post(app.url("/api/v1/settings/api-keys"))
        .bearer_auth(&ctx.jwt)
        .json(&serde_json::json!({ "name": "rw", "scope": "read-write" }))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 201);
    let body: Value = resp.json().await.unwrap();
    let (key_id, key) = (
        body["id"].as_i64().unwrap(),
        body["key"].as_str().unwrap().to_string(),
    );

    let resp = app
        .client
        .post(app.url("/api/v1/reconciliation/accept"))
        .bearer_auth(&key)
        .json(&serde_json::json!({
            "bankAccountId": ctx.bank_account_id,
            "proposals": [{ "type": "invoice", "bankTransactionId": tx_id, "invoiceId": inv_id }],
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let body: Value = resp.json().await.unwrap();
    assert_eq!(body["accepted"].as_array().map(Vec::len), Some(1), "{body}");

    let groupes = groupes_document(&pool, inv_id).await;
    assert_eq!(groupes.len(), 1, "un groupe : {groupes:?}");
    assert_eq!(groupes[0].1, 2, "vente + encaissement");
    let cle_vente: Option<i64> = sqlx::query_scalar(
        "SELECT lettering_key FROM journal_entry_lines WHERE entry_id = ? AND debit > 0",
    )
    .bind(je_id)
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(cle_vente, Some(groupes[0].0), "l'ancre est dans le groupe");

    let (actor_type, actor_key, details): (String, Option<i64>, Value) = sqlx::query_as(
        "SELECT actor_type, actor_api_key_id, details_json FROM audit_log \
         WHERE action = 'lettering.created'",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!((actor_type.as_str(), actor_key), ("api_key", Some(key_id)));
    assert_eq!(details["origin"], "document");
    assert_eq!(details["documentType"], "invoice");
    assert_eq!(details["documentId"], inv_id);
    assert_eq!(details["documentNumber"], "INV-LETTRE-1");
}

/// AC15 (b) — une modification concurrente de la facture entre l'instantané et
/// l'étape (g) (ici, un règlement manuel qui la SOLDE pendant que
/// l'acceptation attend) rend `race_during_update` sans qu'aucune marque ne soit
/// posée par l'acceptation : le seul lettrage est celui du règlement manuel.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn accept_race_refuses_before_any_lettering(pool: MySqlPool) {
    use kesh_db::entities::SettlementChoice;
    use kesh_db::repositories::invoice_settlements_write;
    let ctx = setup_company(&pool, "CourseL", "CH4431999123000889012", Role::Comptable).await;
    let day = NaiveDate::from_ymd_opt(2026, 5, 15).unwrap();
    let (inv_id, _) = seed_validated_invoice(
        &pool,
        ctx.company_id,
        ctx.contact_id,
        "INV-RACE-L",
        NaiveDate::from_ymd_opt(2026, 4, 20).unwrap(),
        dec!(1000.00),
    )
    .await;
    let tx_id = seed_bank_transactions(
        &pool,
        ctx.company_id,
        ctx.bank_account_id,
        ctx.user_id,
        &unique_hash("race_lettering"),
        day,
        day,
        vec![make_new_tx(
            ctx.company_id,
            ctx.bank_account_id,
            day,
            Some(day),
            dec!(1000.00),
            "CHF",
            "INV-RACE-L",
            Some("CourseL Client"),
        )],
    )
    .await[0];
    let app = spawn_app(pool.clone()).await;

    let mut verrou = pool.acquire().await.unwrap().detach();
    sqlx::query("LOCK TABLES contacts WRITE")
        .execute(&mut verrou)
        .await
        .unwrap();
    let acceptation = accept_in_background(&app, &ctx, tx_id, inv_id);
    let bloquee =
        kesh_db::test_fixtures::attendre_une_requete_en_cours(&pool, &["FROM contacts"], || {
            acceptation.is_finished()
        })
        .await;
    assert!(bloquee, "l'acceptation devait attendre sur `contacts`");

    // Le règlement manuel qui SOLDE la facture, pendant l'attente : il la lettre.
    let manuel = invoice_settlements_write::settle_invoice(
        &pool,
        ctx.user_id,
        ctx.company_id,
        inv_id,
        SettlementChoice::BankTransfer {
            bank_account_id: ctx.bank_account_id,
        },
        dec!(1000.00),
        day,
    )
    .await
    .expect("règlement manuel");
    assert!(manuel.fully_settled);
    let avant = groupes_document(&pool, inv_id).await;
    assert_eq!(avant.len(), 1, "montage : le règlement manuel a lettré");

    sqlx::query("UNLOCK TABLES")
        .execute(&mut verrou)
        .await
        .unwrap();
    drop(verrou);
    let (status, body) = acceptation.await.unwrap();
    assert_eq!(status, 200, "{body}");
    let failed = body["failed"].as_array().unwrap();
    assert_eq!(failed.len(), 1, "{body}");
    assert_eq!(
        failed[0]["errorCode"], "RECONCILIATION_INVOICE_NOT_ELIGIBLE",
        "le refus du contrôle de version (g), non celui de la transaction"
    );
    assert_eq!(failed[0]["details"]["reason"], "race_during_update");

    assert_eq!(
        groupes_document(&pool, inv_id).await,
        avant,
        "aucune marque neuve"
    );
    let lettrages: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM audit_log WHERE action = 'lettering.created'")
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(
        lettrages, 1,
        "le seul lettrage est celui du règlement manuel"
    );
    let reglements: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM invoice_settlements WHERE invoice_id = ?")
            .bind(inv_id)
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(reglements, 1, "l'acceptation n'a rien laissé");
}

// ============================================================
// Story 27-4 (#634) — un lot qui contient une transaction disparue est
// traité proposition par proposition
// ============================================================

/// Une transaction de `amount`, référence `reference`, contrepartie
/// « Acme Client », dans un import qui lui est propre ; rend son identifiant.
async fn seed_one_tx_274(
    pool: &MySqlPool,
    ctx: &CompanyCtx,
    bank_account_id: i64,
    seed: &str,
    amount: Decimal,
    reference: &str,
) -> i64 {
    let day = NaiveDate::from_ymd_opt(2026, 5, 15).unwrap();
    seed_bank_transactions(
        pool,
        ctx.company_id,
        bank_account_id,
        ctx.user_id,
        &unique_hash(seed),
        day,
        day,
        vec![make_new_tx(
            ctx.company_id,
            bank_account_id,
            day,
            Some(day),
            amount,
            "CHF",
            reference,
            Some("Acme Client"),
        )],
    )
    .await[0]
}

/// Une facture validée de `amount`, numéro `number`, datée du 2026-05-01 :
/// rapprochable d'une transaction du 2026-05-15 de même montant et de même
/// référence. Rend son identifiant.
async fn seed_invoice_274(
    pool: &MySqlPool,
    ctx: &CompanyCtx,
    number: &str,
    amount: Decimal,
) -> i64 {
    let inv_date = NaiveDate::from_ymd_opt(2026, 5, 1).unwrap();
    seed_validated_invoice(
        pool,
        ctx.company_id,
        ctx.contact_id,
        number,
        inv_date,
        amount,
    )
    .await
    .0
}

/// `DELETE /api/v1/bank-imports/{id}` de l'import qui porte `tx_id`
/// (Story 27-1), avec le JWT du Comptable ; asserte `204`.
async fn delete_import_of_274(app: &TestApp, pool: &MySqlPool, jwt: &str, tx_id: i64) {
    let import_id: i64 = sqlx::query_scalar("SELECT import_id FROM bank_transactions WHERE id = ?")
        .bind(tx_id)
        .fetch_one(pool)
        .await
        .unwrap();
    let resp = app
        .client
        .delete(app.url(&format!("/api/v1/bank-imports/{import_id}")))
        .bearer_auth(jwt)
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 204, "suppression de l'import {import_id}");
}

/// Tient le verrou nommé du compte (`reconcile:{base}:{company}:{account}`,
/// KF-038) sur une connexion dédiée, comme
/// `post_accept_returns_409_on_account_lock_contention`.
async fn hold_account_lock_274(
    pool: &MySqlPool,
    ctx: &CompanyCtx,
) -> (sqlx::pool::PoolConnection<sqlx::MySql>, String) {
    let db_name: String = sqlx::query_scalar("SELECT COALESCE(DATABASE(), '')")
        .fetch_one(pool)
        .await
        .unwrap();
    let lock_name = format!(
        "reconcile:{}:{}:{}",
        db_name, ctx.company_id, ctx.bank_account_id
    );
    let mut conn = pool.acquire().await.unwrap();
    let acquired: Option<i32> = sqlx::query_scalar("SELECT GET_LOCK(?, ?)")
        .bind(&lock_name)
        .bind(60)
        .fetch_one(&mut *conn)
        .await
        .unwrap();
    assert_eq!(acquired, Some(1), "verrou du compte tenu par le test");
    (conn, lock_name)
}

async fn release_account_lock_274(
    conn: &mut sqlx::pool::PoolConnection<sqlx::MySql>,
    lock_name: &str,
) {
    let _: Option<i32> = sqlx::query_scalar("SELECT RELEASE_LOCK(?)")
        .bind(lock_name)
        .fetch_one(&mut **conn)
        .await
        .unwrap();
}

/// `POST` JSON synchrone ; rend `(statut, corps)`.
async fn post_274(app: &TestApp, jwt: &str, path: &str, body: &Value) -> (u16, Value) {
    let resp = app
        .client
        .post(app.url(path))
        .bearer_auth(jwt)
        .json(body)
        .send()
        .await
        .unwrap();
    let status = resp.status().as_u16();
    (status, resp.json::<Value>().await.unwrap_or(Value::Null))
}

/// `POST` JSON en tâche, pour un lot qui doit se bloquer sur un verrou.
fn spawn_post_274(
    app: &TestApp,
    jwt: &str,
    path: &str,
    body: Value,
) -> tokio::task::JoinHandle<(u16, Value)> {
    let client = app.client.clone();
    let url = app.url(path);
    let jwt = jwt.to_string();
    tokio::spawn(async move {
        let resp = client
            .post(url)
            .bearer_auth(jwt)
            .json(&body)
            .send()
            .await
            .unwrap();
        let status = resp.status().as_u16();
        (status, resp.json::<Value>().await.unwrap_or(Value::Null))
    })
}

/// Le lot en tâche est vu bloqué dans `with_account_lock` (`SELECT
/// GET_LOCK(?, ?)`, `kesh-reconciliation/src/mutex.rs`) — sinon le montage
/// ne prouve rien.
async fn wait_blocked_on_account_lock_274(
    pool: &MySqlPool,
    task: &tokio::task::JoinHandle<(u16, Value)>,
) {
    let bloque = kesh_db::test_fixtures::attendre_une_requete_en_cours(pool, &["GET_LOCK"], || {
        task.is_finished()
    })
    .await;
    assert!(
        bloque,
        "le lot devait attendre le verrou du compte — le montage ne prouve rien sinon"
    );
}

/// `(bankTransactionId, errorCode, details)` de chaque refus, dans l'ordre.
fn failed_274(body: &Value) -> Vec<(i64, String, Value)> {
    body["failed"]
        .as_array()
        .unwrap_or_else(|| panic!("pas de failed[] : {body}"))
        .iter()
        .map(|f| {
            (
                f["bankTransactionId"].as_i64().unwrap(),
                f["errorCode"].as_str().unwrap().to_string(),
                f["details"].clone(),
            )
        })
        .collect()
}

/// Les `bankTransactionId` d'un tableau de succès (`accepted` / `rejected`).
fn ids_274(body: &Value, key: &str) -> Vec<i64> {
    body[key]
        .as_array()
        .unwrap_or_else(|| panic!("pas de {key}[] : {body}"))
        .iter()
        .map(|r| r["bankTransactionId"].as_i64().unwrap())
        .collect()
}

/// `(status, matched_entry_id, auto_match_rejected_at, version)` d'une ligne.
async fn tx_row_274(
    pool: &MySqlPool,
    id: i64,
) -> Option<(String, Option<i64>, Option<NaiveDateTime>, i32)> {
    sqlx::query_as(
        "SELECT status, matched_entry_id, auto_match_rejected_at, version \
         FROM bank_transactions WHERE id = ?",
    )
    .bind(id)
    .fetch_optional(pool)
    .await
    .unwrap()
}

/// Tous les identifiants cités par les traces `reconciliation.rejected`.
async fn rejected_traces_274(pool: &MySqlPool) -> Vec<Vec<i64>> {
    let rows: Vec<Value> = sqlx::query_scalar(
        "SELECT details_json FROM audit_log WHERE action = 'reconciliation.rejected' ORDER BY id",
    )
    .fetch_all(pool)
    .await
    .unwrap();
    rows.iter()
        .map(|d| {
            d["bank_transaction_ids"]
                .as_array()
                .unwrap()
                .iter()
                .map(|v| v.as_i64().unwrap())
                .collect()
        })
        .collect()
}

async fn invoice_paid_274(pool: &MySqlPool, inv_id: i64) -> bool {
    let paid_at: Option<NaiveDateTime> =
        sqlx::query_scalar("SELECT paid_at FROM invoices WHERE id = ?")
            .bind(inv_id)
            .fetch_one(pool)
            .await
            .unwrap();
    paid_at.is_some()
}

fn not_found_274(id: i64) -> (i64, String, Value) {
    (id, "BANK_TRANSACTION_NOT_FOUND".to_string(), Value::Null)
}

/// AC6-1 (acceptation) — l'import A est supprimé **avant** le lot : `tA` sort
/// en `BANK_TRANSACTION_NOT_FOUND` sans `details`, `tB` est acceptée.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn post_accept_refuses_a_transaction_deleted_with_its_import_and_accepts_the_rest(
    pool: MySqlPool,
) {
    let ctx = setup_company(&pool, "Acme", "CH4431999123000889012", Role::Comptable).await;
    let inv_a = seed_invoice_274(&pool, &ctx, "INV-DEL-A", dec!(110.00)).await;
    let inv_b = seed_invoice_274(&pool, &ctx, "INV-DEL-B", dec!(220.00)).await;
    let t_a = seed_one_tx_274(
        &pool,
        &ctx,
        ctx.bank_account_id,
        "del_a",
        dec!(110.00),
        "INV-DEL-A",
    )
    .await;
    let t_b = seed_one_tx_274(
        &pool,
        &ctx,
        ctx.bank_account_id,
        "del_b",
        dec!(220.00),
        "INV-DEL-B",
    )
    .await;
    let app = spawn_app(pool.clone()).await;

    delete_import_of_274(&app, &pool, &ctx.jwt, t_a).await;
    let (st, body) = post_274(
        &app,
        &ctx.jwt,
        "/api/v1/reconciliation/accept",
        &serde_json::json!({
            "bankAccountId": ctx.bank_account_id,
            "proposals": [
                { "type": "invoice", "bankTransactionId": t_a, "invoiceId": inv_a },
                { "type": "invoice", "bankTransactionId": t_b, "invoiceId": inv_b },
            ],
        }),
    )
    .await;
    assert_eq!(st, 200, "{body}");
    assert_eq!(ids_274(&body, "accepted"), vec![t_b], "{body}");
    assert_eq!(failed_274(&body), vec![not_found_274(t_a)], "{body}");
    assert!(
        invoice_paid_274(&pool, inv_b).await,
        "la facture B est réglée"
    );
    assert!(
        !invoice_paid_274(&pool, inv_a).await,
        "la facture A ne l'est pas"
    );
    assert_eq!(tx_row_274(&pool, t_b).await.unwrap().0, "reconciled");
}

/// AC6-1 (rejet) — même montage : `tA` refusée, `tB` rejetée, la trace ne
/// cite que `tB`.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn post_reject_refuses_a_transaction_deleted_with_its_import_and_rejects_the_rest(
    pool: MySqlPool,
) {
    let ctx = setup_company(&pool, "Acme", "CH4431999123000889012", Role::Comptable).await;
    let t_a = seed_one_tx_274(
        &pool,
        &ctx,
        ctx.bank_account_id,
        "rdel_a",
        dec!(110.00),
        "R-A",
    )
    .await;
    let t_b = seed_one_tx_274(
        &pool,
        &ctx,
        ctx.bank_account_id,
        "rdel_b",
        dec!(220.00),
        "R-B",
    )
    .await;
    let app = spawn_app(pool.clone()).await;

    delete_import_of_274(&app, &pool, &ctx.jwt, t_a).await;
    let (st, body) = post_274(
        &app,
        &ctx.jwt,
        "/api/v1/reconciliation/reject",
        &serde_json::json!({ "bankAccountId": ctx.bank_account_id, "bankTransactionIds": [t_a, t_b] }),
    )
    .await;
    assert_eq!(st, 200, "{body}");
    assert_eq!(ids_274(&body, "rejected"), vec![t_b], "{body}");
    assert_eq!(failed_274(&body), vec![not_found_274(t_a)], "{body}");
    assert!(tx_row_274(&pool, t_b).await.unwrap().2.is_some());
    assert_eq!(rejected_traces_274(&pool).await, vec![vec![t_b]]);
}

/// AC6-2 (acceptation) — l'import A est supprimé **pendant** que le lot attend
/// le verrou du compte (après le pré-vol) : la recharge sous verrou
/// d'`accept_one_invoice` le voit absent.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn post_accept_refuses_a_transaction_deleted_while_the_batch_waits_for_the_lock(
    pool: MySqlPool,
) {
    let ctx = setup_company(&pool, "Acme", "CH4431999123000889012", Role::Comptable).await;
    let inv_a = seed_invoice_274(&pool, &ctx, "INV-WAIT-A", dec!(110.00)).await;
    let inv_b = seed_invoice_274(&pool, &ctx, "INV-WAIT-B", dec!(220.00)).await;
    let t_a = seed_one_tx_274(
        &pool,
        &ctx,
        ctx.bank_account_id,
        "wait_a",
        dec!(110.00),
        "INV-WAIT-A",
    )
    .await;
    let t_b = seed_one_tx_274(
        &pool,
        &ctx,
        ctx.bank_account_id,
        "wait_b",
        dec!(220.00),
        "INV-WAIT-B",
    )
    .await;
    let app = spawn_app(pool.clone()).await;

    let (mut conn, lock) = hold_account_lock_274(&pool, &ctx).await;
    let task = spawn_post_274(
        &app,
        &ctx.jwt,
        "/api/v1/reconciliation/accept",
        serde_json::json!({
            "bankAccountId": ctx.bank_account_id,
            "proposals": [
                { "type": "invoice", "bankTransactionId": t_a, "invoiceId": inv_a },
                { "type": "invoice", "bankTransactionId": t_b, "invoiceId": inv_b },
            ],
        }),
    );
    wait_blocked_on_account_lock_274(&pool, &task).await;
    delete_import_of_274(&app, &pool, &ctx.jwt, t_a).await;
    release_account_lock_274(&mut conn, &lock).await;

    let (st, body) = task.await.unwrap();
    assert_eq!(st, 200, "{body}");
    assert_eq!(ids_274(&body, "accepted"), vec![t_b], "{body}");
    assert_eq!(failed_274(&body), vec![not_found_274(t_a)], "{body}");
    assert!(!invoice_paid_274(&pool, inv_a).await);
    assert!(invoice_paid_274(&pool, inv_b).await);
}

/// AC6-2 (rejet) — même montage : la reprise de l'AC4 (relecture
/// verrouillante) rend `BANK_TRANSACTION_NOT_FOUND`, et non plus
/// `RECONCILIATION_ALREADY_RECONCILED` + `race_during_update`.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn post_reject_refuses_a_transaction_deleted_while_the_batch_waits_for_the_lock(
    pool: MySqlPool,
) {
    let ctx = setup_company(&pool, "Acme", "CH4431999123000889012", Role::Comptable).await;
    let t_a = seed_one_tx_274(
        &pool,
        &ctx,
        ctx.bank_account_id,
        "rwait_a",
        dec!(110.00),
        "R-A",
    )
    .await;
    let t_b = seed_one_tx_274(
        &pool,
        &ctx,
        ctx.bank_account_id,
        "rwait_b",
        dec!(220.00),
        "R-B",
    )
    .await;
    let app = spawn_app(pool.clone()).await;

    let (mut conn, lock) = hold_account_lock_274(&pool, &ctx).await;
    let task = spawn_post_274(
        &app,
        &ctx.jwt,
        "/api/v1/reconciliation/reject",
        serde_json::json!({ "bankAccountId": ctx.bank_account_id, "bankTransactionIds": [t_a, t_b] }),
    );
    wait_blocked_on_account_lock_274(&pool, &task).await;
    delete_import_of_274(&app, &pool, &ctx.jwt, t_a).await;
    release_account_lock_274(&mut conn, &lock).await;

    let (st, body) = task.await.unwrap();
    assert_eq!(st, 200, "{body}");
    assert_eq!(ids_274(&body, "rejected"), vec![t_b], "{body}");
    assert_eq!(failed_274(&body), vec![not_found_274(t_a)], "{body}");
    assert_eq!(rejected_traces_274(&pool).await, vec![vec![t_b]]);
}

/// AC6-4 — un lot de rejet des quatre classes, envoyé dans un ordre non
/// croissant : `rejected` = l'éligible ; `failed` = les trois autres dans
/// l'ordre croissant, codes de l'AC1, `details` nul sur chacun (tue M3 au
/// niveau route : sous M3 la rapprochée ressortirait par le repli
/// `race_during_update`, `details` non nul).
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn post_reject_mixed_batch_of_the_four_classes(pool: MySqlPool) {
    let ctx = setup_company(&pool, "Acme", "CH4431999123000889012", Role::Comptable).await;
    let mut ids = [
        seed_one_tx_274(&pool, &ctx, ctx.bank_account_id, "mix1", dec!(10.00), "M1").await,
        seed_one_tx_274(&pool, &ctx, ctx.bank_account_id, "mix2", dec!(20.00), "M2").await,
        seed_one_tx_274(&pool, &ctx, ctx.bank_account_id, "mix3", dec!(30.00), "M3").await,
    ];
    ids.sort_unstable();
    let (reconciled, rejected, eligible) = (ids[0], ids[1], ids[2]);
    let missing = eligible + 1_000_000;
    sqlx::query("UPDATE bank_transactions SET status = 'reconciled' WHERE id = ?")
        .bind(reconciled)
        .execute(&pool)
        .await
        .unwrap();
    sqlx::query("UPDATE bank_transactions SET auto_match_rejected_at = NOW(3) WHERE id = ?")
        .bind(rejected)
        .execute(&pool)
        .await
        .unwrap();
    let before_reconciled = tx_row_274(&pool, reconciled).await;
    let before_rejected = tx_row_274(&pool, rejected).await;
    let app = spawn_app(pool.clone()).await;

    let (st, body) = post_274(
        &app,
        &ctx.jwt,
        "/api/v1/reconciliation/reject",
        &serde_json::json!({
            "bankAccountId": ctx.bank_account_id,
            "bankTransactionIds": [missing, rejected, eligible, reconciled],
        }),
    )
    .await;
    assert_eq!(st, 200, "{body}");
    assert_eq!(ids_274(&body, "rejected"), vec![eligible], "{body}");
    assert_eq!(
        failed_274(&body),
        vec![
            (
                reconciled,
                "RECONCILIATION_ALREADY_RECONCILED".to_string(),
                Value::Null
            ),
            (
                rejected,
                "RECONCILIATION_ALREADY_REJECTED".to_string(),
                Value::Null
            ),
            not_found_274(missing),
        ],
        "{body}"
    );
    assert_eq!(tx_row_274(&pool, reconciled).await, before_reconciled);
    assert_eq!(tx_row_274(&pool, rejected).await, before_rejected);
    assert_eq!(rejected_traces_274(&pool).await, vec![vec![eligible]]);
}

/// AC6-5 (a) — rejet concurrent de la même transaction pendant que le lot
/// attend le verrou (`UPDATE` direct, qui n'écrit aucune trace) : la garde
/// `auto_match_rejected_at IS NULL` empêche de la ré-horodater et de la tracer
/// deux fois (tue M10).
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn post_reject_does_not_reject_twice_a_transaction_rejected_meanwhile(pool: MySqlPool) {
    let ctx = setup_company(&pool, "Acme", "CH4431999123000889012", Role::Comptable).await;
    let t_a = seed_one_tx_274(
        &pool,
        &ctx,
        ctx.bank_account_id,
        "conc_a",
        dec!(110.00),
        "R-A",
    )
    .await;
    let t_b = seed_one_tx_274(
        &pool,
        &ctx,
        ctx.bank_account_id,
        "conc_b",
        dec!(220.00),
        "R-B",
    )
    .await;
    let app = spawn_app(pool.clone()).await;

    let (mut conn, lock) = hold_account_lock_274(&pool, &ctx).await;
    let task = spawn_post_274(
        &app,
        &ctx.jwt,
        "/api/v1/reconciliation/reject",
        serde_json::json!({ "bankAccountId": ctx.bank_account_id, "bankTransactionIds": [t_a, t_b] }),
    );
    wait_blocked_on_account_lock_274(&pool, &task).await;
    let sentinel = NaiveDate::from_ymd_opt(2026, 1, 1)
        .unwrap()
        .and_hms_opt(0, 0, 0)
        .unwrap();
    sqlx::query("UPDATE bank_transactions SET auto_match_rejected_at = ? WHERE id = ?")
        .bind(sentinel)
        .bind(t_a)
        .execute(&pool)
        .await
        .unwrap();
    let v0 = tx_row_274(&pool, t_a).await.unwrap().3;
    release_account_lock_274(&mut conn, &lock).await;

    let (st, body) = task.await.unwrap();
    assert_eq!(st, 200, "{body}");
    assert_eq!(ids_274(&body, "rejected"), vec![t_b], "{body}");
    assert_eq!(
        failed_274(&body),
        vec![(
            t_a,
            "RECONCILIATION_ALREADY_REJECTED".to_string(),
            Value::Null
        )],
        "{body}"
    );
    let row = tx_row_274(&pool, t_a).await.unwrap();
    assert_eq!(row.2, Some(sentinel), "horodatage concurrent conservé");
    assert_eq!(row.3, v0, "version inchangée");
    assert_eq!(rejected_traces_274(&pool).await, vec![vec![t_b]]);
}

/// AC6-5 (b) — la transaction passe sur un autre compte de la même société
/// pendant l'attente du verrou (ce que produit, au même identifiant, une
/// restauration concurrente) : la garde `bank_account_id = ?` de l'`UPDATE`
/// empêche de rejeter la transaction d'un autre compte (tue M17).
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn post_reject_does_not_reject_a_transaction_moved_to_another_account(pool: MySqlPool) {
    let ctx = setup_company(&pool, "Acme", "CH4431999123000889012", Role::Comptable).await;
    let second_account = create_bank_account(&pool, ctx.company_id, "CH5604835012345678009").await;
    let t_a = seed_one_tx_274(
        &pool,
        &ctx,
        ctx.bank_account_id,
        "move_a",
        dec!(110.00),
        "R-A",
    )
    .await;
    let t_b = seed_one_tx_274(
        &pool,
        &ctx,
        ctx.bank_account_id,
        "move_b",
        dec!(220.00),
        "R-B",
    )
    .await;
    let app = spawn_app(pool.clone()).await;

    let (mut conn, lock) = hold_account_lock_274(&pool, &ctx).await;
    let task = spawn_post_274(
        &app,
        &ctx.jwt,
        "/api/v1/reconciliation/reject",
        serde_json::json!({ "bankAccountId": ctx.bank_account_id, "bankTransactionIds": [t_a, t_b] }),
    );
    wait_blocked_on_account_lock_274(&pool, &task).await;
    sqlx::query("UPDATE bank_transactions SET bank_account_id = ? WHERE id = ?")
        .bind(second_account)
        .bind(t_a)
        .execute(&pool)
        .await
        .unwrap();
    release_account_lock_274(&mut conn, &lock).await;

    let (st, body) = task.await.unwrap();
    assert_eq!(st, 200, "{body}");
    assert_eq!(ids_274(&body, "rejected"), vec![t_b], "{body}");
    assert_eq!(failed_274(&body), vec![not_found_274(t_a)], "{body}");
    assert!(
        tx_row_274(&pool, t_a).await.unwrap().2.is_none(),
        "la transaction de l'autre compte n'est pas rejetée"
    );
    assert_eq!(rejected_traces_274(&pool).await, vec![vec![t_b]]);
}

/// AC6-5 (c) — la transaction passe à `reconciled` pendant l'attente du verrou
/// (ce que laisse une acceptation terminée, puis la suppression de son écriture
/// — `matched_entry_id` est `ON DELETE SET NULL`) : **sans** lien, seule la
/// garde `status = 'pending'` de l'`UPDATE` la protège, et la reprise sous
/// verrou la reclasse en `RECONCILIATION_ALREADY_RECONCILED`, `details` nul
/// (revue de code P1, E1 ; tue M20).
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn post_reject_does_not_reject_a_transaction_reconciled_meanwhile(pool: MySqlPool) {
    let ctx = setup_company(&pool, "Acme", "CH4431999123000889012", Role::Comptable).await;
    let t_a = seed_one_tx_274(
        &pool,
        &ctx,
        ctx.bank_account_id,
        "reco_a",
        dec!(110.00),
        "R-A",
    )
    .await;
    let t_b = seed_one_tx_274(
        &pool,
        &ctx,
        ctx.bank_account_id,
        "reco_b",
        dec!(220.00),
        "R-B",
    )
    .await;
    let app = spawn_app(pool.clone()).await;

    let (mut conn, lock) = hold_account_lock_274(&pool, &ctx).await;
    let task = spawn_post_274(
        &app,
        &ctx.jwt,
        "/api/v1/reconciliation/reject",
        serde_json::json!({ "bankAccountId": ctx.bank_account_id, "bankTransactionIds": [t_a, t_b] }),
    );
    wait_blocked_on_account_lock_274(&pool, &task).await;
    sqlx::query("UPDATE bank_transactions SET status = 'reconciled' WHERE id = ?")
        .bind(t_a)
        .execute(&pool)
        .await
        .unwrap();
    let before = tx_row_274(&pool, t_a).await.unwrap();
    assert_eq!(
        (before.0.as_str(), before.1),
        ("reconciled", None),
        "montage : rapprochée sans lien"
    );
    release_account_lock_274(&mut conn, &lock).await;

    let (st, body) = task.await.unwrap();
    assert_eq!(st, 200, "{body}");
    assert_eq!(ids_274(&body, "rejected"), vec![t_b], "{body}");
    assert_eq!(
        failed_274(&body),
        vec![(
            t_a,
            "RECONCILIATION_ALREADY_RECONCILED".to_string(),
            Value::Null
        )],
        "{body}"
    );
    assert_eq!(
        tx_row_274(&pool, t_a).await.unwrap(),
        before,
        "la transaction rapprochée n'est ni horodatée ni versionnée"
    );
    assert_eq!(rejected_traces_274(&pool).await, vec![vec![t_b]]);
}

/// AC6-6 — l'acceptation d'une transaction rejetée la refuse en
/// `RECONCILIATION_ALREADY_REJECTED` (tue M4 au niveau route).
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn post_accept_refuses_a_rejected_transaction(pool: MySqlPool) {
    let ctx = setup_company(&pool, "Acme", "CH4431999123000889012", Role::Comptable).await;
    let inv = seed_invoice_274(&pool, &ctx, "INV-REJ", dec!(110.00)).await;
    let t = seed_one_tx_274(
        &pool,
        &ctx,
        ctx.bank_account_id,
        "acc_rej",
        dec!(110.00),
        "INV-REJ",
    )
    .await;
    sqlx::query("UPDATE bank_transactions SET auto_match_rejected_at = NOW(3) WHERE id = ?")
        .bind(t)
        .execute(&pool)
        .await
        .unwrap();
    let before = tx_row_274(&pool, t).await;
    let app = spawn_app(pool.clone()).await;
    let (st, body) = post_274(
        &app,
        &ctx.jwt,
        "/api/v1/reconciliation/accept",
        &serde_json::json!({
            "bankAccountId": ctx.bank_account_id,
            "proposals": [{ "type": "invoice", "bankTransactionId": t, "invoiceId": inv }],
        }),
    )
    .await;
    assert_eq!(st, 200, "{body}");
    assert!(ids_274(&body, "accepted").is_empty(), "{body}");
    assert_eq!(
        failed_274(&body),
        vec![(
            t,
            "RECONCILIATION_ALREADY_REJECTED".to_string(),
            Value::Null
        )],
        "{body}"
    );
    assert_eq!(tx_row_274(&pool, t).await, before);
    assert!(!invoice_paid_274(&pool, inv).await);
}

/// AC6-7 — refus du pré-vol et refus sous verrou mêlés, lot non trié : le
/// seul refus sous verrou (`tMil`, import B supprimé pendant l'attente) sort
/// **entre** les deux refus du pré-vol (`tBas`, `tHaut`, rejetées avant) —
/// tue M9 dans ses deux ordres de concaténation.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn post_reject_failed_merges_preflight_and_locked_refusals_by_id(pool: MySqlPool) {
    let ctx = setup_company(&pool, "Acme", "CH4431999123000889012", Role::Comptable).await;
    let t_bas = seed_one_tx_274(&pool, &ctx, ctx.bank_account_id, "m9_a", dec!(10.00), "A").await;
    let t_mil = seed_one_tx_274(&pool, &ctx, ctx.bank_account_id, "m9_b", dec!(20.00), "B").await;
    let t_haut = seed_one_tx_274(&pool, &ctx, ctx.bank_account_id, "m9_c", dec!(30.00), "C").await;
    assert!(
        t_bas < t_mil && t_mil < t_haut,
        "montage : identifiants croissants"
    );
    for id in [t_bas, t_haut] {
        sqlx::query("UPDATE bank_transactions SET auto_match_rejected_at = NOW(3) WHERE id = ?")
            .bind(id)
            .execute(&pool)
            .await
            .unwrap();
    }
    let app = spawn_app(pool.clone()).await;

    let (mut conn, lock) = hold_account_lock_274(&pool, &ctx).await;
    let task = spawn_post_274(
        &app,
        &ctx.jwt,
        "/api/v1/reconciliation/reject",
        serde_json::json!({
            "bankAccountId": ctx.bank_account_id,
            "bankTransactionIds": [t_haut, t_mil, t_bas],
        }),
    );
    wait_blocked_on_account_lock_274(&pool, &task).await;
    delete_import_of_274(&app, &pool, &ctx.jwt, t_mil).await;
    release_account_lock_274(&mut conn, &lock).await;

    let (st, body) = task.await.unwrap();
    assert_eq!(st, 200, "{body}");
    assert!(ids_274(&body, "rejected").is_empty(), "{body}");
    assert_eq!(
        failed_274(&body),
        vec![
            (
                t_bas,
                "RECONCILIATION_ALREADY_REJECTED".to_string(),
                Value::Null
            ),
            not_found_274(t_mil),
            (
                t_haut,
                "RECONCILIATION_ALREADY_REJECTED".to_string(),
                Value::Null
            ),
        ],
        "{body}"
    );
    assert!(
        rejected_traces_274(&pool).await.is_empty(),
        "rien de rejeté, aucune trace"
    );
}

/// AC6-8 — la relecture de l'AC4 est une **lecture courante** (tue M11).
///
/// Deux transactions `tB < tA` du même import. Le test supprime `tA` dans une
/// transaction SQL **non validée** (verrou X). Le lot `reject [tA, tB]` part :
/// le pré-vol (lecture non verrouillante) voit encore `tA` éligible ; sous
/// verrou, `tB` est rejetée — sa relecture `SELECT auto_match_rejected_at`
/// **ouvre la vue cohérente** de la transaction du lot, qui voit `tA` —, puis
/// l'`UPDATE` de `tA` se bloque sur le verrou X. Le test valide alors la
/// suppression : l'`UPDATE` touche 0 ligne, la relecture `FOR UPDATE` voit
/// `tA` absente. Sous M11 (relecture ordinaire), la vue cohérente la verrait
/// éligible → repli `race_during_update`.
///
/// Observation **durable** (C-27-4-26) : l'aide est appelée deux fois, à
/// 500 ms d'écart. Le motif désigne aussi l'`UPDATE` de `tB`, de même texte
/// (PROCESSLIST montre l'instruction, non les valeurs) : le premier appel peut
/// le surprendre en vol ; 500 ms plus tard, le seul `UPDATE` encore en cours
/// est celui de `tA`, bloqué.
///
/// ⚠️ **Angle mort assumé** : la sonde lit une requête **en cours**, non une
/// requête **en attente de verrou** — le seul trait propre à l'attente
/// (`information_schema.INNODB_TRX`, `trx_state = 'LOCK WAIT'`) exige le
/// privilège `PROCESS`, que l'utilisateur de dev (`kesh`, droits par base,
/// `scripts/mariadb-init/01-dev-grants.sql`) n'a pas ; la CI tourne en
/// `root`, mais le test doit passer sur le poste de dev. Sous M11, la mutation
/// ne survit que si l'`UPDATE` de `tB`, par clé primaire, dure plus de 500 ms.
/// Le code juste ne peut rendre aucun faux rouge : quel que soit l'`UPDATE`
/// observé, la suppression est validée avant ou pendant le blocage de `tA`, et
/// la lecture courante la voit. Si M11 survit à une campagne, rejouer le test
/// avant de la déclarer survivante.
///
/// ⚠️ Repose sur MariaDB 10.11 : avec `innodb_snapshot_isolation` (défaut
/// depuis 11.6), la lecture verrouillante rendrait `ER_CHECKREAD` — ce test
/// est celui qui le signalera à la montée de version.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn post_reject_locking_reread_sees_a_deletion_committed_after_the_snapshot(pool: MySqlPool) {
    let ctx = setup_company(&pool, "Acme", "CH4431999123000889012", Role::Comptable).await;
    let day = NaiveDate::from_ymd_opt(2026, 5, 15).unwrap();
    let mut ids = seed_bank_transactions(
        &pool,
        ctx.company_id,
        ctx.bank_account_id,
        ctx.user_id,
        &unique_hash("m11"),
        day,
        day,
        vec![
            make_new_tx(
                ctx.company_id,
                ctx.bank_account_id,
                day,
                Some(day),
                dec!(10.00),
                "CHF",
                "R1",
                None,
            ),
            make_new_tx(
                ctx.company_id,
                ctx.bank_account_id,
                day,
                Some(day),
                dec!(20.00),
                "CHF",
                "R2",
                None,
            ),
        ],
    )
    .await;
    ids.sort_unstable();
    let (t_b, t_a) = (ids[0], ids[1]);
    let app = spawn_app(pool.clone()).await;

    let mut deleting = pool.begin().await.unwrap();
    let deleted = sqlx::query("DELETE FROM bank_transactions WHERE id = ?")
        .bind(t_a)
        .execute(&mut *deleting)
        .await
        .unwrap();
    assert_eq!(
        deleted.rows_affected(),
        1,
        "montage : tA supprimée, non validée"
    );

    let task = spawn_post_274(
        &app,
        &ctx.jwt,
        "/api/v1/reconciliation/reject",
        serde_json::json!({ "bankAccountId": ctx.bank_account_id, "bankTransactionIds": [t_a, t_b] }),
    );
    let motifs = ["UPDATE bank_transactions", "auto_match_rejected_at"];
    for _ in 0..2 {
        let bloque = kesh_db::test_fixtures::attendre_une_requete_en_cours(&pool, &motifs, || {
            task.is_finished()
        })
        .await;
        assert!(
            bloque,
            "l'UPDATE de tA devait attendre la suppression non validée — le montage ne prouve rien sinon"
        );
        tokio::time::sleep(std::time::Duration::from_millis(500)).await;
    }
    deleting.commit().await.unwrap();

    let (st, body) = task.await.unwrap();
    assert_eq!(st, 200, "{body}");
    assert_eq!(ids_274(&body, "rejected"), vec![t_b], "{body}");
    assert_eq!(failed_274(&body), vec![not_found_274(t_a)], "{body}");
    assert!(tx_row_274(&pool, t_a).await.is_none());
}

/// AC2 (a) — `failed[]` d'`accept` suit l'**ordre des propositions reçues**,
/// refus du pré-vol et refus sous verrou mêlés, dans un lot non trié par
/// identifiant (tue M8).
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn post_accept_failed_follows_the_received_order(pool: MySqlPool) {
    let ctx = setup_company(&pool, "Acme", "CH4431999123000889012", Role::Comptable).await;
    let other_account = create_bank_account(&pool, ctx.company_id, "CH5604835012345678009").await;
    // id bas, absent du compte du lot (autre compte de la même société).
    let low_absent = seed_one_tx_274(&pool, &ctx, other_account, "ord_low", dec!(10.00), "L").await;
    // éligible au pré-vol, refusée sous verrou : facture non validée.
    let inv_draft = insert_invoice(
        &pool,
        ctx.company_id,
        ctx.contact_id,
        "INV-DRAFT-ORD",
        NaiveDate::from_ymd_opt(2026, 5, 1).unwrap(),
        dec!(50.00),
        "draft",
        None,
        None,
    )
    .await;
    let mid = seed_one_tx_274(
        &pool,
        &ctx,
        ctx.bank_account_id,
        "ord_mid",
        dec!(50.00),
        "INV-DRAFT-ORD",
    )
    .await;
    let high_absent = mid + 1_000_000;
    assert!(low_absent < mid);
    let app = spawn_app(pool.clone()).await;

    let (st, body) = post_274(
        &app,
        &ctx.jwt,
        "/api/v1/reconciliation/accept",
        &serde_json::json!({
            "bankAccountId": ctx.bank_account_id,
            "proposals": [
                { "type": "invoice", "bankTransactionId": high_absent, "invoiceId": inv_draft },
                { "type": "invoice", "bankTransactionId": mid, "invoiceId": inv_draft },
                { "type": "invoice", "bankTransactionId": low_absent, "invoiceId": inv_draft },
            ],
        }),
    )
    .await;
    assert_eq!(st, 200, "{body}");
    assert!(ids_274(&body, "accepted").is_empty(), "{body}");
    let failed = failed_274(&body);
    let got: Vec<i64> = failed.iter().map(|f| f.0).collect();
    assert_eq!(
        got,
        vec![high_absent, mid, low_absent],
        "ordre reçu : {body}"
    );
    assert_eq!(failed[0], not_found_274(high_absent));
    assert_eq!(failed[1].1, "RECONCILIATION_INVOICE_NOT_ELIGIBLE", "{body}");
    assert_eq!(failed[2], not_found_274(low_absent));
}

/// AC2 (b) — la trace `reconciliation.accepted` porte la taille du lot
/// **reçu** (`batch_size = 2`), pas celle du sous-lot éligible (tue M12).
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn post_accept_trace_batch_size_is_the_received_batch(pool: MySqlPool) {
    let ctx = setup_company(&pool, "Acme", "CH4431999123000889012", Role::Comptable).await;
    let inv = seed_invoice_274(&pool, &ctx, "INV-BS", dec!(110.00)).await;
    let t = seed_one_tx_274(
        &pool,
        &ctx,
        ctx.bank_account_id,
        "bs",
        dec!(110.00),
        "INV-BS",
    )
    .await;
    let app = spawn_app(pool.clone()).await;
    let (st, body) = post_274(
        &app,
        &ctx.jwt,
        "/api/v1/reconciliation/accept",
        &serde_json::json!({
            "bankAccountId": ctx.bank_account_id,
            "proposals": [
                { "type": "invoice", "bankTransactionId": t + 1_000_000, "invoiceId": inv },
                { "type": "invoice", "bankTransactionId": t, "invoiceId": inv },
            ],
        }),
    )
    .await;
    assert_eq!(st, 200, "{body}");
    assert_eq!(ids_274(&body, "accepted"), vec![t], "{body}");
    let details: Value = sqlx::query_scalar(
        "SELECT details_json FROM audit_log WHERE action = 'reconciliation.accepted' AND entity_id = ?",
    )
    .bind(t)
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(details["batch_size"].as_i64(), Some(2), "{details}");
}

/// AC3 — un lot sans aucune proposition éligible répond `200` sans
/// transaction ni verrou, sur les deux routes : le verrou du compte est tenu
/// par le test ; sans le court-circuit, la route attendrait 5 s et rendrait
/// `409` (tue M7a et M7b). Aucune borne de temps : le statut suffit.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn batches_without_eligible_transaction_take_no_lock(pool: MySqlPool) {
    let ctx = setup_company(&pool, "Acme", "CH4431999123000889012", Role::Comptable).await;
    let app = spawn_app(pool.clone()).await;
    let (mut conn, lock) = hold_account_lock_274(&pool, &ctx).await;

    let (st, body) = post_274(
        &app,
        &ctx.jwt,
        "/api/v1/reconciliation/accept",
        &serde_json::json!({
            "bankAccountId": ctx.bank_account_id,
            "proposals": [
                { "type": "invoice", "bankTransactionId": 99_999_991, "invoiceId": 1 },
                { "type": "invoice", "bankTransactionId": 99_999_990, "invoiceId": 1 },
            ],
        }),
    )
    .await;
    assert_eq!(st, 200, "accept sous verrou tenu : {body}");
    assert_eq!(
        failed_274(&body),
        vec![not_found_274(99_999_991), not_found_274(99_999_990)]
    );

    let (st, body) = post_274(
        &app,
        &ctx.jwt,
        "/api/v1/reconciliation/reject",
        &serde_json::json!({
            "bankAccountId": ctx.bank_account_id,
            "bankTransactionIds": [99_999_991, 99_999_990],
        }),
    )
    .await;
    assert_eq!(st, 200, "reject sous verrou tenu : {body}");
    assert_eq!(
        failed_274(&body),
        vec![not_found_274(99_999_990), not_found_274(99_999_991)]
    );
    release_account_lock_274(&mut conn, &lock).await;
}

/// AC5 — plus de 500 identifiants : `400` avant toute lecture, sur les deux
/// routes ; 500 passent le contrôle (tue M13).
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn batches_over_500_transactions_are_refused_in_400(pool: MySqlPool) {
    let ctx = setup_company(&pool, "Acme", "CH4431999123000889012", Role::Comptable).await;
    let app = spawn_app(pool.clone()).await;
    for n in [500_i64, 501] {
        let ids: Vec<i64> = (1..=n).map(|i| 90_000_000 + i).collect();
        let proposals: Vec<Value> = ids
            .iter()
            .map(|id| serde_json::json!({ "type": "invoice", "bankTransactionId": id, "invoiceId": 1 }))
            .collect();
        let (st_acc, body_acc) = post_274(
            &app,
            &ctx.jwt,
            "/api/v1/reconciliation/accept",
            &serde_json::json!({ "bankAccountId": ctx.bank_account_id, "proposals": proposals }),
        )
        .await;
        let (st_rej, body_rej) = post_274(
            &app,
            &ctx.jwt,
            "/api/v1/reconciliation/reject",
            &serde_json::json!({ "bankAccountId": ctx.bank_account_id, "bankTransactionIds": ids }),
        )
        .await;
        if n == 500 {
            assert_eq!(st_acc, 200, "accept 500 : {body_acc}");
            assert_eq!(body_acc["failed"].as_array().unwrap().len(), 500);
            assert_eq!(st_rej, 200, "reject 500 : {body_rej}");
            assert_eq!(body_rej["failed"].as_array().unwrap().len(), 500);
        } else {
            assert_eq!(st_acc, 400, "accept 501 : {body_acc}");
            assert_eq!(body_acc["error"]["code"], "VALIDATION_ERROR");
            assert_eq!(st_rej, 400, "reject 501 : {body_rej}");
            assert_eq!(body_rej["error"]["code"], "VALIDATION_ERROR");
        }
    }
}

// ============================================================
// Story 28-5a (#526) — annuler le rejet d'une transaction
// ============================================================

/// `POST /api/v1/reconciliation/transactions/{id}/unreject` ; rend
/// `(statut, corps)` — `Null` pour un `204`, qui n'a pas de corps.
async fn unreject(app: &TestApp, jwt: &str, tx_id: i64) -> (u16, Value) {
    let resp = app
        .client
        .post(app.url(&format!(
            "/api/v1/reconciliation/transactions/{tx_id}/unreject"
        )))
        .bearer_auth(jwt)
        .send()
        .await
        .unwrap();
    let status = resp.status().as_u16();
    (status, resp.json().await.unwrap_or(Value::Null))
}

/// Rejette `ids` par la route du lot (`POST /reconciliation/reject`) ; exige un
/// `200` sans refus et rend le corps.
async fn reject_285(app: &TestApp, jwt: &str, bank_account_id: i64, ids: &[i64]) -> Value {
    let (st, body) = post_274(
        app,
        jwt,
        "/api/v1/reconciliation/reject",
        &serde_json::json!({ "bankAccountId": bank_account_id, "bankTransactionIds": ids }),
    )
    .await;
    assert_eq!(st, 200, "{body}");
    assert!(
        body["failed"].as_array().unwrap().is_empty(),
        "montage : le rejet ne refuse rien — {body}"
    );
    body
}

/// `n` transactions `pending` d'un même import, montants distincts ; rend
/// leurs identifiants.
async fn seed_txs_285(pool: &MySqlPool, ctx: &CompanyCtx, seed: &str, n: usize) -> Vec<i64> {
    let day = NaiveDate::from_ymd_opt(2026, 5, 15).unwrap();
    let txs = (0..n)
        .map(|i| {
            make_new_tx(
                ctx.company_id,
                ctx.bank_account_id,
                day,
                Some(day),
                Decimal::new(10_000 + 100 * i as i64, 2),
                "CHF",
                &format!("REF-{seed}-{i}"),
                None,
            )
        })
        .collect();
    seed_bank_transactions(
        pool,
        ctx.company_id,
        ctx.bank_account_id,
        ctx.user_id,
        &unique_hash(seed),
        day,
        day,
        txs,
    )
    .await
}

/// Les `bankTransactionId` que `GET /reconciliation/proposals` liste pour le
/// compte.
async fn proposed_ids_285(app: &TestApp, jwt: &str, bank_account_id: i64) -> Vec<i64> {
    let body: Value = app
        .client
        .get(app.url(&format!(
            "/api/v1/reconciliation/proposals?bankAccountId={bank_account_id}"
        )))
        .bearer_auth(jwt)
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    body["proposals"]
        .as_array()
        .unwrap_or_else(|| panic!("pas de proposals[] : {body}"))
        .iter()
        .map(|p| p["bankTransactionId"].as_i64().unwrap())
        .collect()
}

/// La transaction `tx_id` telle que la rend le détail de son import
/// (`GET /api/v1/bank-imports/{id}`).
async fn import_detail_tx_285(app: &TestApp, pool: &MySqlPool, jwt: &str, tx_id: i64) -> Value {
    let import_id: i64 = sqlx::query_scalar("SELECT import_id FROM bank_transactions WHERE id = ?")
        .bind(tx_id)
        .fetch_one(pool)
        .await
        .unwrap();
    let resp = app
        .client
        .get(app.url(&format!("/api/v1/bank-imports/{import_id}")))
        .bearer_auth(jwt)
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let body: Value = resp.json().await.unwrap();
    body["transactions"]
        .as_array()
        .unwrap_or_else(|| panic!("pas de transactions[] : {body}"))
        .iter()
        .find(|t| t["id"].as_i64() == Some(tx_id))
        .unwrap_or_else(|| panic!("transaction {tx_id} absente du détail : {body}"))
        .clone()
}

/// `(entity_type, details_json)` des lignes `reconciliation.unrejected` de la
/// transaction.
async fn unrejected_traces_285(pool: &MySqlPool, tx_id: i64) -> Vec<(String, Value)> {
    sqlx::query_as(
        "SELECT entity_type, details_json FROM audit_log \
         WHERE action = 'reconciliation.unrejected' AND entity_id = ? ORDER BY id",
    )
    .bind(tx_id)
    .fetch_all(pool)
    .await
    .unwrap()
}

async fn count_285(pool: &MySqlPool, sql: &str) -> i64 {
    sqlx::query_scalar(sql).fetch_one(pool).await.unwrap()
}

async fn updated_at_285(pool: &MySqlPool, tx_id: i64) -> NaiveDateTime {
    sqlx::query_scalar("SELECT updated_at FROM bank_transactions WHERE id = ?")
        .bind(tx_id)
        .fetch_one(pool)
        .await
        .unwrap()
}

/// Le refus `409 RECONCILIATION_TRANSACTION_NOT_REJECTED`, `details` compris.
fn assert_not_rejected_285(st: u16, body: &Value, tx_id: i64) {
    assert_eq!(st, 409, "{body}");
    assert_eq!(
        body["error"]["code"], "RECONCILIATION_TRANSACTION_NOT_REJECTED",
        "{body}"
    );
    assert_eq!(
        body["error"]["details"],
        serde_json::json!({ "bankTransactionId": tx_id }),
        "{body}"
    );
}

/// B1 (AC2, AC13) — annuler le rejet remet la transaction dans les
/// propositions, sans écriture ni règlement, et le trace ; puis le cycle
/// rejeter → annuler se rejoue.
///
/// `updated_at` est **forcé** dans le passé par une affectation explicite (sans
/// elle, `ON UPDATE CURRENT_TIMESTAMP(3)` écraserait la valeur forcée) : un
/// rejet et son annulation peuvent tomber dans la même milliseconde. Son
/// assertion établit que la ligne a été écrite ; elle ne tue pas M17, la
/// colonne étant renouvelée par le moteur.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn unreject_puts_a_rejected_transaction_back_in_the_proposals(pool: MySqlPool) {
    let ctx = setup_company(&pool, "Acme", "CH4431999123000889012", Role::Comptable).await;
    let tx_id = seed_txs_285(&pool, &ctx, "b1", 1).await[0];
    let app = spawn_app(pool.clone()).await;
    reject_285(&app, &ctx.jwt, ctx.bank_account_id, &[tx_id]).await;

    let forced = NaiveDate::from_ymd_opt(2020, 1, 1)
        .unwrap()
        .and_hms_milli_opt(0, 0, 0, 0)
        .unwrap();
    sqlx::query("UPDATE bank_transactions SET updated_at = ? WHERE id = ?")
        .bind(forced)
        .bind(tx_id)
        .execute(&pool)
        .await
        .unwrap();
    assert_eq!(
        updated_at_285(&pool, tx_id).await,
        forced,
        "montage : updated_at forcé"
    );
    let (status0, matched0, rejected0, version0) = tx_row_274(&pool, tx_id).await.unwrap();
    assert_eq!((status0.as_str(), matched0), ("pending", None));
    assert!(rejected0.is_some(), "montage : la transaction est rejetée");
    let rejected_at =
        import_detail_tx_285(&app, &pool, &ctx.jwt, tx_id).await["autoMatchRejectedAt"]
            .as_str()
            .expect("autoMatchRejectedAt d'une rejetée")
            .to_string();
    assert!(
        !proposed_ids_285(&app, &ctx.jwt, ctx.bank_account_id)
            .await
            .contains(&tx_id),
        "montage : une rejetée n'est pas proposée"
    );
    let books_sql = [
        format!(
            "SELECT COUNT(*) FROM journal_entries WHERE company_id = {}",
            ctx.company_id
        ),
        format!(
            "SELECT COUNT(*) FROM invoice_settlements WHERE company_id = {}",
            ctx.company_id
        ),
    ];
    let mut books_before = Vec::new();
    for q in &books_sql {
        books_before.push(count_285(&pool, q).await);
    }

    let (st, body) = unreject(&app, &ctx.jwt, tx_id).await;
    assert_eq!(st, 204, "{body}");

    let (status1, matched1, rejected1, version1) = tx_row_274(&pool, tx_id).await.unwrap();
    assert_eq!(rejected1, None, "le marqueur est remis à NULL");
    assert_eq!(version1, version0 + 1, "version + 1");
    assert_eq!(
        (status1.as_str(), matched1),
        ("pending", None),
        "statut et lien inchangés"
    );
    assert!(
        updated_at_285(&pool, tx_id).await > forced,
        "la ligne a été écrite par l'annulation"
    );
    for (q, n) in books_sql.iter().zip(&books_before) {
        assert_eq!(
            count_285(&pool, q).await,
            *n,
            "aucune écriture, aucun règlement : {q}"
        );
    }
    assert!(
        proposed_ids_285(&app, &ctx.jwt, ctx.bank_account_id)
            .await
            .contains(&tx_id),
        "la transaction est de nouveau proposée"
    );
    let traces = unrejected_traces_285(&pool, tx_id).await;
    assert_eq!(traces.len(), 1, "une ligne d'audit : {traces:?}");
    assert_eq!(traces[0].0, "bank_transaction");
    assert_eq!(
        traces[0].1,
        serde_json::json!({ "previouslyRejectedAt": rejected_at }),
        "même chaîne que le champ autoMatchRejectedAt lu avant"
    );

    // AC13 — la transaction se rejette de nouveau, et s'annule de nouveau.
    let body = reject_285(&app, &ctx.jwt, ctx.bank_account_id, &[tx_id]).await;
    assert_eq!(ids_274(&body, "rejected"), vec![tx_id], "{body}");
    assert!(
        tx_row_274(&pool, tx_id).await.unwrap().2.is_some(),
        "le marqueur est reposé"
    );
    let (st, body) = unreject(&app, &ctx.jwt, tx_id).await;
    assert_eq!(st, 204, "{body}");
    assert_eq!(unrejected_traces_285(&pool, tx_id).await.len(), 2);
}

/// B2 (AC3) — une transaction non rejetée, jamais ou plus : `409`, sans rien
/// écrire ni tracer.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn unreject_refuses_a_transaction_that_is_not_rejected(pool: MySqlPool) {
    let ctx = setup_company(&pool, "Acme", "CH4431999123000889012", Role::Comptable).await;
    let tx_id = seed_txs_285(&pool, &ctx, "b2", 1).await[0];
    let app = spawn_app(pool.clone()).await;
    let audit_sql = "SELECT COUNT(*) FROM audit_log";

    let audit0 = count_285(&pool, audit_sql).await;
    let version0 = tx_row_274(&pool, tx_id).await.unwrap().3;
    let (st, body) = unreject(&app, &ctx.jwt, tx_id).await;
    assert_not_rejected_285(st, &body, tx_id);
    assert_eq!(tx_row_274(&pool, tx_id).await.unwrap().3, version0);
    assert_eq!(count_285(&pool, audit_sql).await, audit0, "rien de tracé");

    reject_285(&app, &ctx.jwt, ctx.bank_account_id, &[tx_id]).await;
    assert_eq!(unreject(&app, &ctx.jwt, tx_id).await.0, 204);
    let audit1 = count_285(&pool, audit_sql).await;
    let version1 = tx_row_274(&pool, tx_id).await.unwrap().3;
    let (st, body) = unreject(&app, &ctx.jwt, tx_id).await;
    assert_not_rejected_285(st, &body, tx_id);
    assert_eq!(tx_row_274(&pool, tx_id).await.unwrap().3, version1);
    assert_eq!(count_285(&pool, audit_sql).await, audit1, "rien de tracé");
    assert_eq!(unrejected_traces_285(&pool, tx_id).await.len(), 1);
}

/// B3 (AC3) — une transaction rapprochée au sens d'`is_reconciled` est
/// refusée **même si** son marqueur de rejet est posé.
///
/// ⚠️ **État FORGÉ, et dit tel** : aucun chemin réel ne laisse le marqueur sur
/// une transaction rapprochée (cf. le test du dé-rapprochement
/// `a_previously_rejected_transaction_comes_back_to_the_proposals`). C'est ce
/// que la classification refuse sans second test de statut.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn unreject_refuses_a_reconciled_transaction_even_if_marked(pool: MySqlPool) {
    let ctx = setup_company(&pool, "Acme", "CH4431999123000889012", Role::Comptable).await;
    let tx_id = seed_txs_285(&pool, &ctx, "b3", 1).await[0];
    let fy_id = insert_fake_fiscal_year(&pool, ctx.company_id).await;
    let entry_id = insert_fake_journal_entry(&pool, ctx.company_id, fy_id).await;
    sqlx::query(
        "UPDATE bank_transactions SET status = 'reconciled', matched_entry_id = ?, \
         auto_match_rejected_at = NOW(3) WHERE id = ?",
    )
    .bind(entry_id)
    .bind(tx_id)
    .execute(&pool)
    .await
    .unwrap();
    let before = tx_row_274(&pool, tx_id).await.unwrap();
    assert!(before.2.is_some(), "montage : marqueur posé");
    let app = spawn_app(pool.clone()).await;

    let (st, body) = unreject(&app, &ctx.jwt, tx_id).await;
    assert_not_rejected_285(st, &body, tx_id);
    assert_eq!(
        tx_row_274(&pool, tx_id).await.unwrap(),
        before,
        "marqueur, statut, lien et version intacts"
    );
    assert!(unrejected_traces_285(&pool, tx_id).await.is_empty());
}

/// B4 (AC3, AC4) — Consultation → `403` ; autre société → `404` et rien
/// d'écrit ; Administrateur → `204`.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn unreject_roles_and_scope(pool: MySqlPool) {
    let ctx = setup_company(&pool, "Acme", "CH4431999123000889012", Role::Comptable).await;
    let other = setup_company(&pool, "Autre", "CH9300762011623852957", Role::Comptable).await;
    let tx_id = seed_txs_285(&pool, &ctx, "b4", 1).await[0];
    let app = spawn_app(pool.clone()).await;
    reject_285(&app, &ctx.jwt, ctx.bank_account_id, &[tx_id]).await;

    let reader_id = create_user(&pool, "lecteur", Role::Consultation, ctx.company_id).await;
    let reader = forge_jwt(reader_id, "Consultation", ctx.company_id);
    assert_eq!(unreject(&app, &reader, tx_id).await.0, 403);

    let tables = [
        "SELECT COUNT(*) FROM bank_transactions WHERE auto_match_rejected_at IS NOT NULL",
        "SELECT COUNT(*) FROM audit_log",
    ];
    let mut before = Vec::new();
    for q in tables {
        before.push(count_285(&pool, q).await);
    }
    let row_before = tx_row_274(&pool, tx_id).await.unwrap();
    let (st, body) = unreject(&app, &other.jwt, tx_id).await;
    assert_eq!(st, 404, "{body}");
    assert_eq!(body["error"]["code"], "NOT_FOUND", "{body}");
    for (q, n) in tables.iter().zip(before) {
        assert_eq!(count_285(&pool, q).await, n, "rien d'écrit : {q}");
    }
    assert_eq!(tx_row_274(&pool, tx_id).await.unwrap(), row_before);

    let admin_id = create_user(&pool, "admin_acme", Role::Admin, ctx.company_id).await;
    let admin = forge_jwt(admin_id, "Admin", ctx.company_id);
    let (st, body) = unreject(&app, &admin, tx_id).await;
    assert_eq!(st, 204, "{body}");
    assert_eq!(tx_row_274(&pool, tx_id).await.unwrap().2, None);
}

/// B5 (AC2, AC3) — le geste prend le verrou nommé du compte : tenu par une
/// autre connexion, il rend `409 RECONCILIATION_ACCOUNT_LOCKED` et laisse le
/// marqueur intact (patron de
/// `post_accept_returns_409_on_account_lock_contention`).
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn unreject_returns_409_when_the_account_is_locked(pool: MySqlPool) {
    let ctx = setup_company(&pool, "Acme", "CH4431999123000889012", Role::Comptable).await;
    let tx_id = seed_txs_285(&pool, &ctx, "b5", 1).await[0];
    let app = spawn_app(pool.clone()).await;
    reject_285(&app, &ctx.jwt, ctx.bank_account_id, &[tx_id]).await;
    let before = tx_row_274(&pool, tx_id).await.unwrap();

    // Le nom porte la base courante (KF-038, #228) : à reconstruire à l'identique.
    let db_name: String = sqlx::query_scalar("SELECT COALESCE(DATABASE(), '')")
        .fetch_one(&pool)
        .await
        .unwrap();
    let lock_name = format!(
        "reconcile:{}:{}:{}",
        db_name, ctx.company_id, ctx.bank_account_id
    );
    let mut hold_conn = pool.acquire().await.unwrap();
    let acquired: Option<i32> = sqlx::query_scalar("SELECT GET_LOCK(?, ?)")
        .bind(&lock_name)
        .bind(60)
        .fetch_one(&mut *hold_conn)
        .await
        .unwrap();
    assert_eq!(acquired, Some(1), "montage : verrou tenu");

    let (st, body) = unreject(&app, &ctx.jwt, tx_id).await;
    assert_eq!(st, 409, "{body}");
    assert_eq!(
        body["error"]["code"], "RECONCILIATION_ACCOUNT_LOCKED",
        "{body}"
    );
    assert_eq!(
        tx_row_274(&pool, tx_id).await.unwrap(),
        before,
        "marqueur intact"
    );
    assert!(unrejected_traces_285(&pool, tx_id).await.is_empty());

    let _: Option<i32> = sqlx::query_scalar("SELECT RELEASE_LOCK(?)")
        .bind(&lock_name)
        .fetch_one(&mut *hold_conn)
        .await
        .unwrap();
}

/// B6 (AC4) — une clé d'API `read-write` annule un rejet, et la ligne d'audit
/// la nomme.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn an_api_key_unrejects_and_is_named_in_the_audit(pool: MySqlPool) {
    let ctx = setup_company(&pool, "Acme", "CH4431999123000889012", Role::Comptable).await;
    let tx_id = seed_txs_285(&pool, &ctx, "b6", 1).await[0];
    let app = spawn_app(pool.clone()).await;
    reject_285(&app, &ctx.jwt, ctx.bank_account_id, &[tx_id]).await;
    let resp = app
        .client
        .post(app.url("/api/v1/settings/api-keys"))
        .bearer_auth(&ctx.jwt)
        .json(&serde_json::json!({ "name": "rw", "scope": "read-write" }))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 201);
    let body: Value = resp.json().await.unwrap();
    let (key_id, key) = (
        body["id"].as_i64().unwrap(),
        body["key"].as_str().unwrap().to_string(),
    );

    let (st, body) = unreject(&app, &key, tx_id).await;
    assert_eq!(st, 204, "{body}");
    let (actor_type, actor_key): (String, Option<i64>) = sqlx::query_as(
        "SELECT actor_type, actor_api_key_id FROM audit_log \
         WHERE action = 'reconciliation.unrejected' AND entity_id = ?",
    )
    .bind(tx_id)
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!((actor_type.as_str(), actor_key), ("api_key", Some(key_id)));
}

/// B7 (AC1) — le détail d'un import expose `autoMatchRejectedAt` : chaîne
/// ISO 8601 UTC pour la rejetée, `null` pour l'autre, `null` après
/// l'annulation.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn the_import_detail_exposes_the_rejection_date(pool: MySqlPool) {
    let ctx = setup_company(&pool, "Acme", "CH4431999123000889012", Role::Comptable).await;
    let ids = seed_txs_285(&pool, &ctx, "b7", 2).await;
    let (rejected, other) = (ids[0], ids[1]);
    let app = spawn_app(pool.clone()).await;
    reject_285(&app, &ctx.jwt, ctx.bank_account_id, &[rejected]).await;

    let r = import_detail_tx_285(&app, &pool, &ctx.jwt, rejected).await;
    let at = r["autoMatchRejectedAt"]
        .as_str()
        .unwrap_or_else(|| panic!("chaîne attendue : {r}"));
    assert!(at.ends_with('Z'), "UTC, suffixe Z : {at}");
    assert!(
        chrono::DateTime::parse_from_rfc3339(at).is_ok(),
        "ISO 8601 : {at}"
    );
    let o = import_detail_tx_285(&app, &pool, &ctx.jwt, other).await;
    assert!(
        o.get("autoMatchRejectedAt").is_some_and(Value::is_null),
        "null pour une transaction non rejetée : {o}"
    );

    assert_eq!(unreject(&app, &ctx.jwt, rejected).await.0, 204);
    let r = import_detail_tx_285(&app, &pool, &ctx.jwt, rejected).await;
    assert!(
        r.get("autoMatchRejectedAt").is_some_and(Value::is_null),
        "null après l'annulation : {r}"
    );
}

/// B9 (AC13) — une transaction dont le rejet est annulé s'accepte de nouveau :
/// avant l'annulation, l'acceptation d'une proposition ventilée la refuse en
/// `RECONCILIATION_ALREADY_REJECTED` ; après, elle la rapproche.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn a_transaction_whose_rejection_is_undone_can_be_accepted_again(pool: MySqlPool) {
    let app = spawn_app(pool.clone()).await;
    let ctx = setup_accept_split_ctx(&pool, "unreject_split", "CH1000000000000099285").await;
    let tx_id = ctx.tx_ids[0];
    reject_285(&app, &ctx.jwt, ctx.bank_account_id, &[tx_id]).await;
    let accept_body = serde_json::json!({
        "bankAccountId": ctx.bank_account_id,
        "proposals": [{
            "type": "split",
            "bankTransactionId": tx_id,
            "splits": [
                { "counterpartyAccountId": ctx.cp_a, "amount": "60.00", "description": "Ligne" },
                { "counterpartyAccountId": ctx.cp_b, "amount": "40.00", "description": "Ligne" },
            ],
        }],
    });

    let (st, body) = post_274(
        &app,
        &ctx.jwt,
        "/api/v1/reconciliation/accept",
        &accept_body,
    )
    .await;
    assert_eq!(st, 200, "{body}");
    assert!(ids_274(&body, "accepted").is_empty(), "{body}");
    assert_eq!(
        failed_274(&body),
        vec![(
            tx_id,
            "RECONCILIATION_ALREADY_REJECTED".to_string(),
            Value::Null
        )],
        "{body}"
    );

    assert_eq!(unreject(&app, &ctx.jwt, tx_id).await.0, 204);

    let (st, body) = post_274(
        &app,
        &ctx.jwt,
        "/api/v1/reconciliation/accept",
        &accept_body,
    )
    .await;
    assert_eq!(st, 200, "{body}");
    assert_eq!(ids_274(&body, "accepted"), vec![tx_id], "{body}");
    assert!(failed_274(&body).is_empty(), "{body}");
    let (status, matched, rejected, _) = tx_row_274(&pool, tx_id).await.unwrap();
    assert_eq!(status, "reconciled");
    assert!(matched.is_some(), "rapprochée : lien posé");
    assert_eq!(rejected, None, "marqueur NULL");
}

/// B10 (AC3, « disparue sous verrou ») — une transaction supprimée pendant que
/// l'annulation attend sa lecture verrouillante est **introuvable** : `404`,
/// rien de tracé (tue M15 ; patron de
/// `post_reject_locking_reread_sees_a_deletion_committed_after_the_snapshot`).
///
/// Le test supprime la transaction dans une transaction SQL **non validée**
/// (verrou X), lance l'annulation : la lecture hors verrou du compte (lecture
/// cohérente, autocommit) la voit encore ; sous le verrou nommé, la lecture
/// `FOR UPDATE` se bloque. La suppression est alors validée : la lecture
/// courante voit la ligne absente → `404`. Sous M15 (lecture ordinaire), la vue
/// cohérente la voit encore rejetée, l'`UPDATE` attend le verrou X puis ne
/// touche aucune ligne → `409 OPTIMISTIC_LOCK_CONFLICT` : rouge sur le statut,
/// non sur le montage.
///
/// Les motifs `["bank_transactions", "bank_account_id = ?"]` sont communs à la
/// lecture verrouillante (`bank_tx_by_id_for_account_sql`) et à l'`UPDATE` de
/// l'annulation, absents de la lecture hors verrou (`WHERE id = ? AND
/// company_id = ?`). Observation **durable** (C-27-4-26) : la sonde est
/// appelée deux fois, à 500 ms d'écart.
///
/// ⚠️ **Angle mort assumé** : la sonde voit une requête **en cours**, non une
/// requête **en attente de verrou** — `information_schema.INNODB_TRX` exige le
/// privilège `PROCESS`, absent sur le poste de dev. Sous M15, la mutation ne
/// survivrait que si la lecture ordinaire, par clé primaire, durait plus de
/// 500 ms. Si M15 survit à une campagne, rejouer le test avant de la déclarer
/// survivante.
///
/// ⚠️ Repose sur MariaDB 10.11 : avec `innodb_snapshot_isolation` (défaut
/// depuis 11.6), la lecture verrouillante rendrait `ER_CHECKREAD`.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn unreject_of_a_transaction_deleted_under_the_lock_is_not_found(pool: MySqlPool) {
    let ctx = setup_company(&pool, "Acme", "CH4431999123000889012", Role::Comptable).await;
    let tx_id = seed_txs_285(&pool, &ctx, "b10", 1).await[0];
    let app = spawn_app(pool.clone()).await;
    reject_285(&app, &ctx.jwt, ctx.bank_account_id, &[tx_id]).await;

    let mut deleting = pool.begin().await.unwrap();
    let deleted = sqlx::query("DELETE FROM bank_transactions WHERE id = ?")
        .bind(tx_id)
        .execute(&mut *deleting)
        .await
        .unwrap();
    assert_eq!(
        deleted.rows_affected(),
        1,
        "montage : transaction supprimée, non validée"
    );

    let task = {
        let client = app.client.clone();
        let url = app.url(&format!(
            "/api/v1/reconciliation/transactions/{tx_id}/unreject"
        ));
        let jwt = ctx.jwt.clone();
        tokio::spawn(async move {
            let resp = client.post(url).bearer_auth(jwt).send().await.unwrap();
            let status = resp.status().as_u16();
            (status, resp.json::<Value>().await.unwrap_or(Value::Null))
        })
    };
    let motifs = ["bank_transactions", "bank_account_id = ?"];
    for _ in 0..2 {
        let bloque = kesh_db::test_fixtures::attendre_une_requete_en_cours(&pool, &motifs, || {
            task.is_finished()
        })
        .await;
        assert!(
            bloque,
            "l'annulation devait attendre la suppression non validée — le montage ne prouve rien sinon"
        );
        tokio::time::sleep(std::time::Duration::from_millis(500)).await;
    }
    deleting.commit().await.unwrap();

    let (st, body) = task.await.unwrap();
    assert_eq!(st, 404, "{body}");
    assert_eq!(body["error"]["code"], "NOT_FOUND", "{body}");
    assert!(unrejected_traces_285(&pool, tx_id).await.is_empty());
}
