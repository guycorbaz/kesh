//! Story 33-8 (#486) — un compte désigné dans les paramètres de facturation ne
//! s'archive plus sans avertissement ; et un compte déjà archivé n'y verrouille
//! plus les réglages (la seconde face de l'issue).

mod common;

use std::collections::BTreeSet;
use std::net::SocketAddr;
use std::sync::Arc;

use chrono::{NaiveDate, TimeDelta};
use common::{audit_actions, audit_details};
use kesh_api::auth::password::hash_password;
use kesh_api::config::Config;
use kesh_api::{AppState, build_router};
use kesh_db::entities::user::{NewUser, Role};
use kesh_db::entities::{
    NewInvoice, NewInvoiceLine, NewSupplierInvoice, NewSupplierInvoiceLine, SettlementChoice,
    SettlementWriteOffNature,
};
use kesh_db::errors::DbError;
use kesh_db::repositories::company_invoice_settings::DESIGNATED_ACCOUNT_COLUMNS;
use kesh_db::repositories::{
    accounts, invoice_settlements_write, invoices, supplier_invoices, users,
};
use kesh_db::test_fixtures::{SeededCompany, designate_rounding_account, seed_accounting_company};
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use serde_json::{Value, json};
use sqlx::MySqlPool;

const TEST_JWT_SECRET: &[u8] = b"test-secret-32-bytes-minimum-test-secret-padding";
const ADMIN_PASSWORD: &str = "admin123";

/// La table de l'AC-1 : la colonne, et le code stable que l'API en rend.
const TEN_SETTINGS: [(&str, &str); 10] = [
    ("default_receivable_account_id", "receivable"),
    ("default_revenue_account_id", "revenue"),
    ("default_vat_payable_account_id", "vatPayable"),
    ("default_vat_recoverable_account_id", "vatRecoverable"),
    ("default_vat_decompte_account_id", "vatDecompte"),
    ("default_payable_account_id", "payable"),
    ("default_rounding_account_id", "rounding"),
    ("default_discount_account_id", "discount"),
    ("default_bank_fees_account_id", "bankFees"),
    ("default_bad_debt_account_id", "badDebt"),
];

struct TestApp {
    base_url: String,
    client: reqwest::Client,
    token: String,
}

impl TestApp {
    fn url(&self, path: &str) -> String {
        format!("{}{}", self.base_url, path)
    }

    async fn archive(&self, id: i64, body: Value) -> reqwest::Response {
        self.client
            .put(self.url(&format!("/api/v1/accounts/{id}/archive")))
            .bearer_auth(&self.token)
            .json(&body)
            .send()
            .await
            .unwrap()
    }
}

async fn login(base_url: &str, client: &reqwest::Client, name: &str, password: &str) -> String {
    let body: Value = client
        .post(format!("{base_url}/api/v1/auth/login"))
        .json(&json!({ "username": name, "password": password }))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    body["accessToken"].as_str().unwrap().to_string()
}

async fn spawn_app(pool: &MySqlPool) -> TestApp {
    let config = Config::from_fields_for_test(
        "mysql://test:test@localhost:3306/test".to_string(),
        "admin".to_string(),
        ADMIN_PASSWORD.to_string(),
        String::from_utf8(TEST_JWT_SECRET.to_vec()).unwrap(),
        TimeDelta::minutes(15),
        TimeDelta::days(30),
        TimeDelta::minutes(15),
        100,
        TimeDelta::minutes(30),
        12,
    );
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
    let state =
        AppState::new_for_tests(pool.clone(), Arc::new(config), Arc::new(rate_limiter), i18n);
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
    let base_url = format!("http://{addr}");
    let client = reqwest::Client::new();
    let token = login(&base_url, &client, "admin", ADMIN_PASSWORD).await;
    TestApp {
        base_url,
        client,
        token,
    }
}

async fn setup(pool: &MySqlPool) -> (TestApp, SeededCompany) {
    let seeded = seed_accounting_company(pool).await.unwrap();
    // Le montage désigne d'office des comptes du seed : les tests partent de zéro
    // et désignent ce qu'ils veulent éprouver.
    clear_designations(pool, seeded.company_id).await;
    (spawn_app(pool).await, seeded)
}

async fn clear_designations(pool: &MySqlPool, company_id: i64) {
    let set = TEN_SETTINGS
        .iter()
        .map(|(c, _)| format!("{c} = NULL"))
        .collect::<Vec<_>>()
        .join(", ");
    sqlx::query(&format!(
        "UPDATE company_invoice_settings SET {set} WHERE company_id = ?"
    ))
    .bind(company_id)
    .execute(pool)
    .await
    .unwrap();
}

async fn designate(pool: &MySqlPool, company_id: i64, column: &str, account_id: i64) {
    sqlx::query(&format!(
        "UPDATE company_invoice_settings SET {column} = ? WHERE company_id = ?"
    ))
    .bind(account_id)
    .bind(company_id)
    .execute(pool)
    .await
    .unwrap();
}

/// Un compte neuf de la société, de type et de numéro donnés.
async fn new_account(pool: &MySqlPool, company_id: i64, number: &str, kind: &str) -> i64 {
    sqlx::query_scalar(
        "INSERT INTO accounts (company_id, number, name, account_type) \
         VALUES (?, ?, 'Compte 33-8', ?) RETURNING id",
    )
    .bind(company_id)
    .bind(number)
    .bind(kind)
    .fetch_one(pool)
    .await
    .unwrap()
}

async fn account_state(pool: &MySqlPool, id: i64) -> (bool, i32) {
    sqlx::query_as("SELECT active, version FROM accounts WHERE id = ?")
        .bind(id)
        .fetch_one(pool)
        .await
        .unwrap()
}

fn settings_of(body: &Value) -> Vec<String> {
    body["error"]["details"]["settings"]
        .as_array()
        .unwrap_or_else(|| panic!("details.settings attendu : {body}"))
        .iter()
        .map(|s| s.as_str().unwrap().to_string())
        .collect()
}

/// AC-1 — chacun des dix réglages : 409, le code exact, le compte intact, aucune trace.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn each_of_the_ten_settings_stops_the_archive(pool: MySqlPool) {
    let (app, seeded) = setup(&pool).await;
    for (i, (column, code)) in TEN_SETTINGS.iter().enumerate() {
        let id = new_account(&pool, seeded.company_id, &format!("95{i:02}"), "Expense").await;
        designate(&pool, seeded.company_id, column, id).await;

        let resp = app.archive(id, json!({ "version": 1 })).await;
        assert_eq!(resp.status(), 409, "{column}");
        let body: Value = resp.json().await.unwrap();
        assert_eq!(body["error"]["code"], "ACCOUNT_DESIGNATED_IN_SETTINGS");
        assert_eq!(settings_of(&body), vec![code.to_string()], "{column}");
        assert!(
            !body["error"]["message"].as_str().unwrap().is_empty(),
            "le message est traduit"
        );

        assert_eq!(account_state(&pool, id).await, (true, 1), "{column}");
        assert!(
            audit_actions(&pool, "account", id).await.is_empty(),
            "{column} : aucune trace"
        );
    }
}

/// AC-2 — deux réglages sur le même compte : les deux codes.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn two_settings_on_one_account_are_both_named(pool: MySqlPool) {
    let (app, seeded) = setup(&pool).await;
    let id = new_account(&pool, seeded.company_id, "9501", "Expense").await;
    designate(&pool, seeded.company_id, "default_discount_account_id", id).await;
    designate(&pool, seeded.company_id, "default_bad_debt_account_id", id).await;

    let resp = app.archive(id, json!({ "version": 1 })).await;
    assert_eq!(resp.status(), 409);
    let body: Value = resp.json().await.unwrap();
    assert_eq!(settings_of(&body), vec!["discount", "badDebt"]);
}

/// AC-3 — confirmé : archivé, tracé, le réglage reste tel quel.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn a_confirmed_archive_goes_through_and_the_setting_stays(pool: MySqlPool) {
    let seeded = seed_accounting_company(&pool).await.unwrap();
    let app = spawn_app(&pool).await;
    let id = seeded.accounts["1100"];
    designate(
        &pool,
        seeded.company_id,
        "default_receivable_account_id",
        id,
    )
    .await;
    let (_, version) = account_state(&pool, id).await;

    let resp = app
        .archive(id, json!({ "version": version, "confirmDesignated": true }))
        .await;
    assert_eq!(resp.status(), 200);
    assert_eq!(resp.json::<Value>().await.unwrap()["active"], false);

    let details = audit_details(&pool, "account", id, "account.archived")
        .await
        .expect("details_json");
    assert_eq!(details["designatedSettings"], json!(["receivable"]));
    let kept: Option<i64> = sqlx::query_scalar(
        "SELECT default_receivable_account_id FROM company_invoice_settings WHERE company_id = ?",
    )
    .bind(seeded.company_id)
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(kept, Some(id), "le réglage ne bouge pas");
}

// ---------------------------------------------------------------------------
// Ce que l'archivage refuse, réglage par réglage (revue P1, A M-2, E-1, E-2, B M2) :
// le témoin avant l'archivage, puis la variante d'erreur exacte. Le dialogue n'en dit
// plus la liste (revue P2, F1, F2) : il dit la règle générale, éprouvée plus bas.
// ---------------------------------------------------------------------------

/// Un brouillon de facture : TVA et compte de produit au choix du test.
async fn draft_invoice(
    pool: &MySqlPool,
    seeded: &SeededCompany,
    price: Decimal,
    vat: Decimal,
    revenue: Option<i64>,
) -> Result<i64, DbError> {
    let contact_id: i64 = sqlx::query_scalar(
        "INSERT INTO contacts (company_id, contact_type, name, is_client) \
         VALUES (?, 'Entreprise', 'Client 33-8', TRUE) RETURNING id",
    )
    .bind(seeded.company_id)
    .fetch_one(pool)
    .await
    .unwrap();
    let date = NaiveDate::from_ymd_opt(2026, 4, 1).unwrap();
    let (invoice, _) = invoices::create(
        pool,
        seeded.admin_user_id,
        NewInvoice {
            company_id: seeded.company_id,
            contact_id,
            date,
            due_date: Some(date),
            payment_terms: None,
            project_id: None,
            lines: vec![NewInvoiceLine {
                description: "Prestation".into(),
                quantity: dec!(1),
                unit_price: price,
                vat_rate: vat,
                revenue_account_id: revenue,
            }],
        },
    )
    .await?;
    Ok(invoice.id)
}

async fn validate_new_invoice(
    pool: &MySqlPool,
    seeded: &SeededCompany,
    price: Decimal,
    vat: Decimal,
) -> Result<i64, DbError> {
    let revenue = Some(seeded.accounts["3000"]);
    let id = draft_invoice(pool, seeded, price, vat, revenue)
        .await
        .expect("brouillon");
    invoices::validate_invoice(pool, seeded.company_id, id, seeded.admin_user_id)
        .await
        .map(|_| id)
}

async fn validate_new_supplier_invoice(
    pool: &MySqlPool,
    seeded: &SeededCompany,
    number: &str,
    vat: Decimal,
) -> Result<i64, DbError> {
    let contact_id: i64 = sqlx::query_scalar(
        "INSERT INTO contacts (company_id, contact_type, name, is_client, is_supplier) \
         VALUES (?, 'Entreprise', ?, FALSE, TRUE) RETURNING id",
    )
    .bind(seeded.company_id)
    .bind(format!("Fournisseur {number}"))
    .fetch_one(pool)
    .await
    .unwrap();
    let draft = supplier_invoices::create_draft(
        pool,
        NewSupplierInvoice {
            company_id: seeded.company_id,
            contact_id,
            supplier_invoice_number: Some(number.into()),
            invoice_date: NaiveDate::from_ymd_opt(2026, 4, 1).unwrap(),
            due_date: None,
            creditor_iban: Some("CH9300762011623852957".into()),
            creditor_qr_iban: None,
            payment_reference: None,
            expected_payment_amount: None,
            project_id: None,
            lines: vec![NewSupplierInvoiceLine {
                description: "Achat".into(),
                quantity: dec!(1),
                unit_price: dec!(100.00),
                vat_rate: vat,
                expense_account_id: seeded.accounts["4000"],
            }],
        },
        seeded.admin_user_id,
    )
    .await
    .expect("brouillon fournisseur");
    supplier_invoices::validate(
        pool,
        seeded.company_id,
        draft.invoice.id,
        draft.invoice.version,
        seeded.admin_user_id,
    )
    .await
    .map(|_| draft.invoice.id)
}

/// Désigne un compte neuf pour `column`, puis rend de quoi l'archiver sur confirmation.
async fn designated_account(
    pool: &MySqlPool,
    seeded: &SeededCompany,
    column: &str,
    kind: &str,
) -> i64 {
    let id = new_account(pool, seeded.company_id, "9501", kind).await;
    designate(pool, seeded.company_id, column, id).await;
    id
}

async fn archive_confirmed(app: &TestApp, id: i64) {
    let resp = app
        .archive(id, json!({ "version": 1, "confirmDesignated": true }))
        .await;
    assert_eq!(resp.status(), 200);
}

#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn archived_receivable_refuses_the_validation_of_an_invoice(pool: MySqlPool) {
    let seeded = seed_accounting_company(&pool).await.unwrap();
    let app = spawn_app(&pool).await;
    let id = designated_account(&pool, &seeded, "default_receivable_account_id", "Asset").await;
    validate_new_invoice(&pool, &seeded, dec!(100.00), dec!(0))
        .await
        .expect("témoin");

    archive_confirmed(&app, id).await;

    let err = validate_new_invoice(&pool, &seeded, dec!(100.00), dec!(0))
        .await
        .expect_err("refusée");
    assert!(matches!(err, DbError::InactiveOrInvalidAccounts), "{err:?}");
}

#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn archived_vat_payable_refuses_only_an_invoice_with_vat(pool: MySqlPool) {
    let seeded = seed_accounting_company(&pool).await.unwrap();
    let app = spawn_app(&pool).await;
    let id = designated_account(
        &pool,
        &seeded,
        "default_vat_payable_account_id",
        "Liability",
    )
    .await;
    validate_new_invoice(&pool, &seeded, dec!(100.00), dec!(8.10))
        .await
        .expect("témoin");

    archive_confirmed(&app, id).await;

    let err = validate_new_invoice(&pool, &seeded, dec!(100.00), dec!(8.10))
        .await
        .expect_err("refusée");
    assert!(matches!(err, DbError::InactiveOrInvalidAccounts), "{err:?}");
    validate_new_invoice(&pool, &seeded, dec!(100.00), dec!(0))
        .await
        .expect("sans TVA, ce compte n'est pas employé");
}

#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn archived_default_revenue_refuses_a_line_without_its_own_account(pool: MySqlPool) {
    let seeded = seed_accounting_company(&pool).await.unwrap();
    let app = spawn_app(&pool).await;
    let id = designated_account(&pool, &seeded, "default_revenue_account_id", "Revenue").await;
    let validate_without_own_account = async || {
        let draft = draft_invoice(&pool, &seeded, dec!(100.00), dec!(0), None)
            .await
            .expect("le brouillon ne contrôle pas le compte par défaut");
        invoices::validate_invoice(&pool, seeded.company_id, draft, seeded.admin_user_id).await
    };
    validate_without_own_account()
        .await
        .expect("témoin : le compte par défaut s'applique");

    archive_confirmed(&app, id).await;

    let err = validate_without_own_account()
        .await
        .expect_err("refusée à la validation");
    assert!(matches!(err, DbError::InvalidRevenueAccounts(_)), "{err:?}");
    validate_new_invoice(&pool, &seeded, dec!(100.00), dec!(0))
        .await
        .expect("une ligne à son propre compte n'emploie pas le réglage");
}

#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn archived_payable_refuses_the_validation_of_a_supplier_invoice(pool: MySqlPool) {
    let seeded = seed_accounting_company(&pool).await.unwrap();
    let app = spawn_app(&pool).await;
    let id = designated_account(&pool, &seeded, "default_payable_account_id", "Liability").await;
    validate_new_supplier_invoice(&pool, &seeded, "F1", dec!(0))
        .await
        .expect("témoin");

    archive_confirmed(&app, id).await;

    let err = validate_new_supplier_invoice(&pool, &seeded, "F2", dec!(0))
        .await
        .expect_err("refusée");
    assert!(matches!(err, DbError::InactiveOrInvalidAccounts), "{err:?}");
}

#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn archived_vat_recoverable_refuses_only_a_supplier_invoice_with_vat(pool: MySqlPool) {
    let seeded = seed_accounting_company(&pool).await.unwrap();
    let app = spawn_app(&pool).await;
    let id = designated_account(
        &pool,
        &seeded,
        "default_vat_recoverable_account_id",
        "Asset",
    )
    .await;
    let payable = new_account(&pool, seeded.company_id, "9502", "Liability").await;
    designate(
        &pool,
        seeded.company_id,
        "default_payable_account_id",
        payable,
    )
    .await;
    validate_new_supplier_invoice(&pool, &seeded, "F1", dec!(8.10))
        .await
        .expect("témoin");

    archive_confirmed(&app, id).await;

    let err = validate_new_supplier_invoice(&pool, &seeded, "F2", dec!(8.10))
        .await
        .expect_err("refusée");
    assert!(matches!(err, DbError::InactiveOrInvalidAccounts), "{err:?}");
    validate_new_supplier_invoice(&pool, &seeded, "F3", dec!(0))
        .await
        .expect("sans TVA, ce compte n'est pas employé");
}

/// Le dialogue ne promet aucun refus pour le décompte TVA : aucune écriture ne le lit.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn archived_vat_decompte_refuses_nothing(pool: MySqlPool) {
    let seeded = seed_accounting_company(&pool).await.unwrap();
    let app = spawn_app(&pool).await;
    let id = designated_account(
        &pool,
        &seeded,
        "default_vat_decompte_account_id",
        "Liability",
    )
    .await;
    let payable = new_account(&pool, seeded.company_id, "9502", "Liability").await;
    designate(
        &pool,
        seeded.company_id,
        "default_payable_account_id",
        payable,
    )
    .await;
    let recoverable = new_account(&pool, seeded.company_id, "9503", "Asset").await;
    designate(
        &pool,
        seeded.company_id,
        "default_vat_recoverable_account_id",
        recoverable,
    )
    .await;

    archive_confirmed(&app, id).await;

    validate_new_invoice(&pool, &seeded, dec!(100.00), dec!(8.10))
        .await
        .expect("facture avec TVA");
    validate_new_supplier_invoice(&pool, &seeded, "F1", dec!(8.10))
        .await
        .expect("facture fournisseur avec TVA");
}

/// Une facture à arrondir, et un règlement qui laisse un écart, emploient ce compte.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn archived_rounding_refuses_what_writes_a_rounding(pool: MySqlPool) {
    let seeded = seed_accounting_company(&pool).await.unwrap();
    let app = spawn_app(&pool).await;
    let id = designate_rounding_account(&pool, seeded.company_id)
        .await
        .unwrap();
    validate_new_invoice(&pool, &seeded, dec!(100.03), dec!(0))
        .await
        .expect("témoin : l'arrondi à 5 centimes s'écrit");

    let resp = app
        .archive(id, json!({ "version": 1, "confirmDesignated": true }))
        .await;
    assert_eq!(resp.status(), 200);

    let err = validate_new_invoice(&pool, &seeded, dec!(100.03), dec!(0))
        .await
        .expect_err("refusée");
    assert!(
        matches!(err, DbError::RoundingAccountNotConfigured { .. }),
        "{err:?}"
    );
    validate_new_invoice(&pool, &seeded, dec!(100.00), dec!(0))
        .await
        .expect("un total déjà rond n'emploie pas ce compte");

    // Un règlement au centime d'un reste brut hors centime : l'autre emploi de ce compte.
    sqlx::query(
        "UPDATE company_invoice_settings SET round_to_5_centimes = FALSE WHERE company_id = ?",
    )
    .bind(seeded.company_id)
    .execute(&pool)
    .await
    .unwrap();
    let invoice = validate_new_invoice(&pool, &seeded, dec!(10.0050), dec!(0))
        .await
        .expect("hors arrondi à 5 centimes, la validation n'emploie pas ce compte");
    let err = invoice_settlements_write::settle_invoice(
        &pool,
        seeded.admin_user_id,
        seeded.company_id,
        invoice,
        SettlementChoice::InternalAccount {
            account_id: seeded.accounts["1000"],
        },
        dec!(10.01),
        NaiveDate::from_ymd_opt(2026, 4, 2).unwrap(),
    )
    .await
    .expect_err("l'écart d'arrondi n'a plus de compte");
    assert!(
        matches!(err, DbError::RoundingAccountNotConfigured { .. }),
        "{err:?}"
    );
}

/// Solder une facture emploie le compte de sa nature, et de celle-là seulement.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn archived_write_off_account_refuses_a_write_off_of_that_nature(pool: MySqlPool) {
    let seeded = seed_accounting_company(&pool).await.unwrap();
    let app = spawn_app(&pool).await;
    for (column, nature, kept) in [
        (
            "default_discount_account_id",
            SettlementWriteOffNature::Discount,
            SettlementWriteOffNature::BankFees,
        ),
        (
            "default_bank_fees_account_id",
            SettlementWriteOffNature::BankFees,
            SettlementWriteOffNature::BadDebt,
        ),
        (
            "default_bad_debt_account_id",
            SettlementWriteOffNature::BadDebt,
            SettlementWriteOffNature::Discount,
        ),
    ] {
        sqlx::query(
            "UPDATE company_invoice_settings SET default_discount_account_id = ?, \
             default_bank_fees_account_id = ?, default_bad_debt_account_id = ? \
             WHERE company_id = ?",
        )
        .bind(seeded.accounts["4000"])
        .bind(seeded.accounts["4000"])
        .bind(seeded.accounts["4000"])
        .bind(seeded.company_id)
        .execute(&pool)
        .await
        .unwrap();
        let number = match nature {
            SettlementWriteOffNature::Discount => "9501",
            SettlementWriteOffNature::BankFees => "9502",
            _ => "9503",
        };
        let id = new_account(&pool, seeded.company_id, number, "Expense").await;
        designate(&pool, seeded.company_id, column, id).await;
        let write_off = |nature| {
            let pool = pool.clone();
            let company_id = seeded.company_id;
            let user = seeded.admin_user_id;
            let seeded = &seeded;
            async move {
                let invoice = validate_new_invoice(&pool, seeded, dec!(100.00), dec!(0))
                    .await
                    .expect("facture validée");
                let version: i32 = sqlx::query_scalar("SELECT version FROM invoices WHERE id = ?")
                    .bind(invoice)
                    .fetch_one(&pool)
                    .await
                    .unwrap();
                invoice_settlements_write::write_off_invoice(
                    &pool,
                    user,
                    company_id,
                    invoice,
                    nature,
                    NaiveDate::from_ymd_opt(2026, 4, 2).unwrap(),
                    version,
                )
                .await
            }
        };
        write_off(nature).await.expect("témoin");

        archive_confirmed(&app, id).await;

        let err = write_off(nature).await.expect_err("refusé");
        assert!(
            matches!(err, DbError::WriteOffAccountNotConfigured { .. }),
            "{column} : {err:?}"
        );
        write_off(kept)
            .await
            .expect("une autre nature n'est pas touchée");
    }
}

// ---------------------------------------------------------------------------
// La règle que dit le dialogue (revues P2 et P3) : tant que le compte est archivé, toute
// écriture qui l'emploie est refusée, y compris sur une pièce déjà validée ; pour lever
// le blocage, le réactiver (`journal_entries.rs`, garde `active` inconditionnelle). Le
// dialogue ne promet aucun autre remède : ces deux tests ne le font pas non plus.
// ---------------------------------------------------------------------------

async fn reactivate(pool: &MySqlPool, seeded: &SeededCompany, id: i64) {
    let (_, version) = account_state(pool, id).await;
    accounts::reactivate(pool, id, version, seeded.admin_user_id, false)
        .await
        .expect("réactivé");
}

#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn archived_receivable_refuses_the_settlement_of_a_validated_invoice_until_reactivated(
    pool: MySqlPool,
) {
    let seeded = seed_accounting_company(&pool).await.unwrap();
    let app = spawn_app(&pool).await;
    let id = designated_account(&pool, &seeded, "default_receivable_account_id", "Asset").await;
    let invoice = validate_new_invoice(&pool, &seeded, dec!(100.00), dec!(0))
        .await
        .expect("validée sur ce compte");
    let settle = |amount: Decimal| {
        invoice_settlements_write::settle_invoice(
            &pool,
            seeded.admin_user_id,
            seeded.company_id,
            invoice,
            SettlementChoice::InternalAccount {
                account_id: seeded.accounts["1000"],
            },
            amount,
            NaiveDate::from_ymd_opt(2026, 4, 2).unwrap(),
        )
    };
    settle(dec!(10.00)).await.expect("témoin : un acompte");

    archive_confirmed(&app, id).await;

    let err = settle(dec!(10.00)).await.expect_err("refusé");
    assert!(matches!(err, DbError::InactiveOrInvalidAccounts), "{err:?}");
    let other = new_account(&pool, seeded.company_id, "9502", "Asset").await;
    designate(
        &pool,
        seeded.company_id,
        "default_receivable_account_id",
        other,
    )
    .await;
    let err = settle(dec!(10.00))
        .await
        .expect_err("toujours refusé, un autre compte désigné");
    assert!(matches!(err, DbError::InactiveOrInvalidAccounts), "{err:?}");

    reactivate(&pool, &seeded, id).await;
    settle(dec!(90.00))
        .await
        .expect("réactivé, le règlement passe");
}

#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn archived_payable_refuses_the_payment_of_a_validated_supplier_invoice_until_reactivated(
    pool: MySqlPool,
) {
    let seeded = seed_accounting_company(&pool).await.unwrap();
    let app = spawn_app(&pool).await;
    let id = designated_account(&pool, &seeded, "default_payable_account_id", "Liability").await;
    let witness = validate_new_supplier_invoice(&pool, &seeded, "F1", dec!(0))
        .await
        .expect("validée sur ce compte");
    let invoice = validate_new_supplier_invoice(&pool, &seeded, "F2", dec!(0))
        .await
        .expect("validée sur ce compte");
    let pay = |invoice: i64| {
        supplier_invoices::pay(
            &pool,
            seeded.company_id,
            invoice,
            SettlementChoice::InternalAccount {
                account_id: seeded.accounts["1000"],
            },
            NaiveDate::from_ymd_opt(2026, 4, 2).unwrap(),
            seeded.admin_user_id,
        )
    };
    pay(witness).await.expect("témoin");

    archive_confirmed(&app, id).await;

    let err = pay(invoice).await.expect_err("refusé");
    assert!(matches!(err, DbError::InactiveOrInvalidAccounts), "{err:?}");
    let other = new_account(&pool, seeded.company_id, "9502", "Liability").await;
    designate(
        &pool,
        seeded.company_id,
        "default_payable_account_id",
        other,
    )
    .await;
    let err = pay(invoice)
        .await
        .expect_err("toujours refusé, un autre compte désigné");
    assert!(matches!(err, DbError::InactiveOrInvalidAccounts), "{err:?}");

    reactivate(&pool, &seeded, id).await;
    pay(invoice).await.expect("réactivé, le paiement passe");
}

/// Revue P1 (E-8) — la confirmation explicitement fausse vaut son absence ; sur un
/// compte non désigné, elle ne change rien à la trace.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn explicit_confirmation_values_change_nothing_else(pool: MySqlPool) {
    let (app, seeded) = setup(&pool).await;
    let designated = new_account(&pool, seeded.company_id, "9501", "Expense").await;
    designate(
        &pool,
        seeded.company_id,
        "default_discount_account_id",
        designated,
    )
    .await;
    let resp = app
        .archive(
            designated,
            json!({ "version": 1, "confirmDesignated": false }),
        )
        .await;
    assert_eq!(resp.status(), 409);

    let plain = new_account(&pool, seeded.company_id, "9502", "Expense").await;
    let resp = app
        .archive(plain, json!({ "version": 1, "confirmDesignated": true }))
        .await;
    assert_eq!(resp.status(), 200);
    let details = audit_details(&pool, "account", plain, "account.archived")
        .await
        .expect("details_json");
    assert!(details.get("designatedSettings").is_none(), "{details}");
}

/// Revue P1 (E-8) — une société sans ligne de réglages ne désigne rien.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn a_company_without_settings_designates_nothing(pool: MySqlPool) {
    let (app, seeded) = setup(&pool).await;
    sqlx::query("DELETE FROM company_invoice_settings WHERE company_id = ?")
        .bind(seeded.company_id)
        .execute(&pool)
        .await
        .unwrap();
    let id = new_account(&pool, seeded.company_id, "9501", "Expense").await;
    assert_eq!(app.archive(id, json!({ "version": 1 })).await.status(), 200);
}

/// AC-4 — non désigné : 200 sans confirmation, audit exactement inchangé (M6).
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn an_undesignated_account_archives_as_before(pool: MySqlPool) {
    let (app, seeded) = setup(&pool).await;
    let id = new_account(&pool, seeded.company_id, "9501", "Expense").await;

    let resp = app.archive(id, json!({ "version": 1 })).await;
    assert_eq!(resp.status(), 200);

    let details = audit_details(&pool, "account", id, "account.archived")
        .await
        .expect("details_json");
    assert!(
        details.get("designatedSettings").is_none(),
        "le snapshot reste exactement celui d'avant : {details}"
    );
}

/// AC-4 — un sous-compte actif prime sur la désignation.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn active_children_are_refused_first(pool: MySqlPool) {
    let (app, seeded) = setup(&pool).await;
    let parent = new_account(&pool, seeded.company_id, "9500", "Expense").await;
    let child = new_account(&pool, seeded.company_id, "9501", "Expense").await;
    sqlx::query("UPDATE accounts SET parent_id = ? WHERE id = ?")
        .bind(parent)
        .bind(child)
        .execute(&pool)
        .await
        .unwrap();
    designate(
        &pool,
        seeded.company_id,
        "default_discount_account_id",
        parent,
    )
    .await;

    // ⚠️ Sans confirmation (revue P1, M1) : confirmée, la désignation laisserait
    // passer, et les deux ordres rendraient le même refus.
    let resp = app.archive(parent, json!({ "version": 1 })).await;
    assert_eq!(resp.status(), 409);
    let body: Value = resp.json().await.unwrap();
    assert_eq!(body["error"]["code"], "ILLEGAL_STATE_TRANSITION", "{body}");
    assert_eq!(account_state(&pool, parent).await, (true, 1));
}

/// AC-4 — version périmée, avec confirmation : le 409 de version.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn a_stale_version_with_confirmation_is_a_version_conflict(pool: MySqlPool) {
    let (app, seeded) = setup(&pool).await;
    let id = new_account(&pool, seeded.company_id, "9501", "Expense").await;
    designate(&pool, seeded.company_id, "default_discount_account_id", id).await;

    let resp = app
        .archive(id, json!({ "version": 99, "confirmDesignated": true }))
        .await;
    assert_eq!(resp.status(), 409);
    let body: Value = resp.json().await.unwrap();
    assert_eq!(body["error"]["code"], "OPTIMISTIC_LOCK_CONFLICT");
    assert_eq!(account_state(&pool, id).await, (true, 1));
}

/// AC-4, C-e334-29 — version périmée, sans confirmation : la désignation, lue sur
/// l'état courant, parle d'abord ; sa confirmation à la même version reçoit le 409
/// de version. Le compte n'a bougé à aucun des deux (M7).
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn a_stale_version_without_confirmation_hits_the_designation_first(pool: MySqlPool) {
    let (app, seeded) = setup(&pool).await;
    let id = new_account(&pool, seeded.company_id, "9501", "Expense").await;
    designate(&pool, seeded.company_id, "default_discount_account_id", id).await;

    let resp = app.archive(id, json!({ "version": 99 })).await;
    assert_eq!(resp.status(), 409);
    let body: Value = resp.json().await.unwrap();
    assert_eq!(body["error"]["code"], "ACCOUNT_DESIGNATED_IN_SETTINGS");
    assert_eq!(account_state(&pool, id).await, (true, 1));

    let resp = app
        .archive(id, json!({ "version": 99, "confirmDesignated": true }))
        .await;
    assert_eq!(resp.status(), 409);
    let body: Value = resp.json().await.unwrap();
    assert_eq!(body["error"]["code"], "OPTIMISTIC_LOCK_CONFLICT");
    assert_eq!(account_state(&pool, id).await, (true, 1));
}

/// Le refus ne consomme pas la version : la confirmation part de celle de l'écran.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn the_refusal_does_not_consume_the_version(pool: MySqlPool) {
    let (app, seeded) = setup(&pool).await;
    let id = new_account(&pool, seeded.company_id, "9501", "Expense").await;
    designate(&pool, seeded.company_id, "default_discount_account_id", id).await;

    assert_eq!(app.archive(id, json!({ "version": 1 })).await.status(), 409);
    let resp = app
        .archive(id, json!({ "version": 1, "confirmDesignated": true }))
        .await;
    assert_eq!(resp.status(), 200);
    assert_eq!(account_state(&pool, id).await, (false, 2));
}

/// Le Comptable, qui ne règle pas ces paramètres, n'est pas enfermé : il confirme.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn a_comptable_may_confirm(pool: MySqlPool) {
    let (mut app, seeded) = setup(&pool).await;
    users::create(
        &pool,
        NewUser {
            username: "compta338".into(),
            password_hash: hash_password("password-e2e-33-8").unwrap(),
            role: Role::Comptable,
            active: true,
            company_id: seeded.company_id,
            email: None,
        },
    )
    .await
    .unwrap();
    app.token = login(&app.base_url, &app.client, "compta338", "password-e2e-33-8").await;
    let id = new_account(&pool, seeded.company_id, "9501", "Expense").await;
    designate(&pool, seeded.company_id, "default_discount_account_id", id).await;

    assert_eq!(app.archive(id, json!({ "version": 1 })).await.status(), 409);
    assert_eq!(
        app.archive(id, json!({ "version": 1, "confirmDesignated": true }))
            .await
            .status(),
        200
    );
}

/// AC-5 — une désignation d'une autre société ne déclenche rien (M2).
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn another_companys_designation_is_ignored(pool: MySqlPool) {
    let (app, seeded) = setup(&pool).await;
    let other: i64 = sqlx::query_scalar(
        "INSERT INTO companies (name, address, org_type, accounting_language, instance_language) \
         VALUES ('Autre 33-8', 'Rue 1', 'Pme', 'FR', 'FR') RETURNING id",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    // Le même numéro de compte dans l'autre société, et une désignation forcée en
    // SQL du compte de la première depuis les réglages de la seconde.
    new_account(&pool, other, "9501", "Expense").await;
    let id = new_account(&pool, seeded.company_id, "9501", "Expense").await;
    sqlx::query(
        "INSERT INTO company_invoice_settings (company_id, default_discount_account_id) \
         VALUES (?, ?)",
    )
    .bind(other)
    .bind(id)
    .execute(&pool)
    .await
    .unwrap();

    let resp = app.archive(id, json!({ "version": 1 })).await;
    assert_eq!(resp.status(), 200, "{}", resp.text().await.unwrap());
}

/// AC-6 — la liste égale les colonnes réelles : un onzième réglage ne se tait pas.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn the_inventory_equals_the_real_columns(pool: MySqlPool) {
    // Revue P1 (A L-7) : par la clé étrangère vers `accounts`, non par le nom de la
    // colonne, qu'un onzième réglage pourrait ne pas suivre.
    let columns: Vec<String> = sqlx::query_scalar(
        "SELECT CAST(COLUMN_NAME AS CHAR) FROM information_schema.KEY_COLUMN_USAGE \
         WHERE TABLE_SCHEMA = DATABASE() AND TABLE_NAME = 'company_invoice_settings' \
         AND REFERENCED_TABLE_NAME = 'accounts'",
    )
    .fetch_all(&pool)
    .await
    .unwrap();
    assert!(
        columns.len() >= 10,
        "la requête voit les colonnes : {columns:?}"
    );
    let real: BTreeSet<&str> = columns.iter().map(String::as_str).collect();
    let listed: BTreeSet<&str> = DESIGNATED_ACCOUNT_COLUMNS.iter().map(|(c, _)| *c).collect();
    assert_eq!(listed, real);
    assert_eq!(
        DESIGNATED_ACCOUNT_COLUMNS.to_vec(),
        TEN_SETTINGS.to_vec(),
        "les codes de l'API sont stables"
    );
}

/// AC-7 — la seconde face : un compte archivé par confirmation n'empêche plus
/// d'enregistrer un autre réglage (M5) ; passer **à** un compte archivé reste refusé.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn an_archived_designation_no_longer_locks_the_settings(pool: MySqlPool) {
    let seeded = seed_accounting_company(&pool).await.unwrap();
    let app = spawn_app(&pool).await;
    let vat_payable = new_account(&pool, seeded.company_id, "9501", "Liability").await;
    designate(
        &pool,
        seeded.company_id,
        "default_vat_payable_account_id",
        vat_payable,
    )
    .await;
    assert_eq!(
        app.archive(
            vat_payable,
            json!({ "version": 1, "confirmDesignated": true })
        )
        .await
        .status(),
        200
    );

    let path = "/api/v1/company/invoice-settings";
    let current: Value = app
        .client
        .get(app.url(path))
        .bearer_auth(&app.token)
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(current["defaultVatPayableAccountId"], json!(vat_payable));

    let mut body = current.clone();
    body["invoiceNumberFormat"] = json!("F-{YEAR}-{SEQ:04}");
    let resp = app
        .client
        .put(app.url(path))
        .bearer_auth(&app.token)
        .json(&body)
        .send()
        .await
        .unwrap();
    let status = resp.status();
    let saved: Value = resp.json().await.unwrap();
    assert_eq!(status, 200, "{saved}");
    assert_eq!(saved["invoiceNumberFormat"], "F-{YEAR}-{SEQ:04}");

    // Passer À un autre compte archivé reste refusé.
    let other = new_account(&pool, seeded.company_id, "9502", "Liability").await;
    sqlx::query("UPDATE accounts SET active = FALSE WHERE id = ?")
        .bind(other)
        .execute(&pool)
        .await
        .unwrap();
    let mut body = saved.clone();
    body["defaultVatPayableAccountId"] = json!(other);
    let resp = app
        .client
        .put(app.url(path))
        .bearer_auth(&app.token)
        .json(&body)
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 400);
    let refused: Value = resp.json().await.unwrap();
    assert!(
        refused["error"]["message"]
            .as_str()
            .unwrap()
            .contains("archivé"),
        "{refused}"
    );
}

/// Revue P1 (B M3) — **le type reste vérifié** pour les six champs historiques, même
/// reconduit et même archivé (`validate_account`, à chaque enregistrement). Les quatre
/// comptes d'écart ne sont revalidés, type compris, que s'ils changent
/// (`resolve_designated_account`) : asymétrie antérieure, que ce test épingle.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn the_type_stays_checked_for_a_historical_field_only(pool: MySqlPool) {
    let seeded = seed_accounting_company(&pool).await.unwrap();
    let app = spawn_app(&pool).await;
    let path = "/api/v1/company/invoice-settings";
    let vat = new_account(&pool, seeded.company_id, "9501", "Liability").await;
    designate(
        &pool,
        seeded.company_id,
        "default_vat_payable_account_id",
        vat,
    )
    .await;
    let rounding = new_account(&pool, seeded.company_id, "9502", "Expense").await;
    designate(
        &pool,
        seeded.company_id,
        "default_rounding_account_id",
        rounding,
    )
    .await;

    // Le compte d'écart retypé en actif, archivé, reconduit : rien ne le revalide.
    sqlx::query("UPDATE accounts SET active = FALSE, account_type = 'Asset' WHERE id = ?")
        .bind(rounding)
        .execute(&pool)
        .await
        .unwrap();
    let body: Value = app
        .client
        .get(app.url(path))
        .bearer_auth(&app.token)
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    let resp = app
        .client
        .put(app.url(path))
        .bearer_auth(&app.token)
        .json(&body)
        .send()
        .await
        .unwrap();
    assert_eq!(
        resp.status(),
        200,
        "témoin : {}",
        resp.text().await.unwrap()
    );

    // Le champ historique retypé et archivé, reconduit à l'identique : le type parle.
    sqlx::query("UPDATE accounts SET active = FALSE, account_type = 'Asset' WHERE id = ?")
        .bind(vat)
        .execute(&pool)
        .await
        .unwrap();
    let body: Value = app
        .client
        .get(app.url(path))
        .bearer_auth(&app.token)
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    let resp = app
        .client
        .put(app.url(path))
        .bearer_auth(&app.token)
        .json(&body)
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 400);
    let refused: Value = resp.json().await.unwrap();
    assert!(
        refused["error"]["message"]
            .as_str()
            .unwrap()
            .contains("type de compte incompatible"),
        "{refused}"
    );
}
