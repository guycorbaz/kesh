//! Story 35-5a (#1022, #907, #929) — la date du jour est celle de **Zurich**
//! pour ce qui écrit dans les livres ou les borne, même la nuit.
//!
//! Chaque test arrête l'horloge de l'état sur un instant de nuit
//! (`HorlogeFixe`) : `2025-12-31T23:30:00Z` est 00 h 30 le **1er janvier
//! 2026** à Zurich (heure d'hiver), `2026-03-10T23:30:00Z` 00 h 30 le
//! **11 mars**. Le code d'avant la story datait au jour UTC (le 31 décembre,
//! dans l'exercice 2025 ; le 10 mars) : ces tests rougissent sur lui.
//!
//! Montage (AC2, AC3) : la société de `seed_accounting_company`, son exercice
//! ramené à **2025** et un exercice **2026** ouvert (ou absent : le refus
//! d'AC3). Les pièces sont datées de décembre 2025 ; seule la contre-passation
//! (ou l'écriture d'ouverture) porte la date du jour. Fixtures synthétiques.
//!
//! Pré-requis : MariaDB démarré.

mod common;

use std::net::SocketAddr;
use std::str::FromStr;
use std::sync::Arc;

use chrono::{DateTime, NaiveDate, TimeDelta, Utc};
use kesh_api::config::Config;
use kesh_api::{AppState, build_router};
use kesh_db::entities::account::AccountType;
use kesh_db::entities::contact::{ContactType, NewContact};
use kesh_db::entities::journal_entry::Journal;
use kesh_db::entities::{
    AccountRole, BankImportSourceFormat, NewAccount, NewBankImport, NewBankTransaction,
    NewJournalEntry, NewJournalEntryLine, NewSupplierInvoice, NewSupplierInvoiceLine,
};
use kesh_db::repositories::{accounts, bank_imports, contacts, journal_entries, supplier_invoices};
use kesh_db::test_fixtures::{
    SeededCompany, disable_rounding_to_5_centimes, mark_onboarding_complete,
    seed_accounting_company,
};
use rust_decimal_macros::dec;
use serde_json::{Value, json};
use sqlx::MySqlPool;

const TEST_JWT_SECRET: &[u8] = b"test-secret-32-bytes-minimum-test-secret-padding";
/// Le mot de passe de l'administrateur de `seed_accounting_company`.
const ADMIN_PASSWORD: &str = "admin123";

/// 00 h 30 le 1er janvier 2026 à Zurich ; encore le 31 décembre en UTC.
const NUIT_DU_NOUVEL_AN: &str = "2025-12-31T23:30:00Z";

fn instant(s: &str) -> DateTime<Utc> {
    DateTime::parse_from_rfc3339(s).unwrap().with_timezone(&Utc)
}

fn jour(a: i32, m: u32, j: u32) -> NaiveDate {
    NaiveDate::from_ymd_opt(a, m, j).unwrap()
}

// ============================================================
// Serveur à horloge arrêtée
// ============================================================

struct TestApp {
    base_url: String,
    client: reqwest::Client,
    token: String,
}

impl TestApp {
    async fn req(
        &self,
        methode: reqwest::Method,
        chemin: &str,
        corps: Option<Value>,
    ) -> (u16, Value) {
        let mut r = self
            .client
            .request(methode, format!("{}{chemin}", self.base_url))
            .bearer_auth(&self.token);
        if let Some(c) = corps {
            r = r.json(&c);
        }
        let resp = r.send().await.unwrap();
        let status = resp.status().as_u16();
        (status, resp.json().await.unwrap_or(Value::Null))
    }

    async fn post(&self, chemin: &str, corps: Value) -> (u16, Value) {
        self.req(reqwest::Method::POST, chemin, Some(corps)).await
    }

    async fn post_vide(&self, chemin: &str) -> (u16, Value) {
        self.req(reqwest::Method::POST, chemin, None).await
    }

    async fn get(&self, chemin: &str) -> (u16, Value) {
        self.req(reqwest::Method::GET, chemin, None).await
    }
}

/// Un serveur dont l'horloge est arrêtée sur `maintenant`, connecté en
/// administrateur (`admin` / `admin123`, `seed_accounting_company`).
async fn spawn_app(pool: &MySqlPool, maintenant: DateTime<Utc>) -> TestApp {
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
    let state = common::horloge::avec_horloge(
        AppState::new_for_tests(pool.clone(), Arc::new(config), Arc::new(rate_limiter), i18n),
        maintenant,
    );
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
    let body: Value = client
        .post(format!("{base_url}/api/v1/auth/login"))
        .json(&json!({ "username": "admin", "password": ADMIN_PASSWORD }))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    let token = body["accessToken"]
        .as_str()
        .unwrap_or_else(|| panic!("connexion : {body}"))
        .to_string();
    TestApp {
        base_url,
        client,
        token,
    }
}

// ============================================================
// Montage : exercice 2025, et 2026 ouvert, clos ou absent
// ============================================================

struct Ctx {
    pool: MySqlPool,
    app: TestApp,
    s: SeededCompany,
    fy_2025: i64,
    fy_2026: Option<i64>,
    /// 1020 *Banque*, compte lié des comptes bancaires du montage.
    ledger: i64,
}

/// La société de `seed_accounting_company`, son exercice ramené à 2025, et un
/// exercice 2026 au statut `exercice_2026` (`None` : pas d'exercice 2026).
async fn monter(pool: &MySqlPool, maintenant: &str, exercice_2026: Option<&str>) -> Ctx {
    let s = seed_accounting_company(pool).await.expect("seed");
    mark_onboarding_complete(pool).await.expect("onboarding");
    disable_rounding_to_5_centimes(pool, s.company_id)
        .await
        .expect("arrondi");
    sqlx::query(
        "UPDATE fiscal_years SET name = 'Exercice 2025', start_date = '2025-01-01', \
         end_date = '2025-12-31' WHERE id = ?",
    )
    .bind(s.fiscal_year_id)
    .execute(pool)
    .await
    .unwrap();
    let fy_2026 = match exercice_2026 {
        None => None,
        Some(statut) => Some(
            sqlx::query(
                "INSERT INTO fiscal_years (company_id, name, start_date, end_date, status) \
                 VALUES (?, 'Exercice 2026', '2026-01-01', '2026-12-31', ?)",
            )
            .bind(s.company_id)
            .bind(statut)
            .execute(pool)
            .await
            .unwrap()
            .last_insert_id() as i64,
        ),
    };
    let ledger: i64 = sqlx::query_scalar(
        "INSERT INTO accounts (company_id, number, name, account_type, active, postable) \
         VALUES (?, '1020', 'Banque', 'Asset', 1, 1) RETURNING id",
    )
    .bind(s.company_id)
    .fetch_one(pool)
    .await
    .unwrap();
    sqlx::query(
        "UPDATE company_invoice_settings SET default_payable_account_id = ? WHERE company_id = ?",
    )
    .bind(s.accounts["2000"])
    .bind(s.company_id)
    .execute(pool)
    .await
    .unwrap();
    let app = spawn_app(pool, instant(maintenant)).await;
    Ctx {
        pool: pool.clone(),
        app,
        fy_2025: s.fiscal_year_id,
        s,
        fy_2026,
        ledger,
    }
}

impl Ctx {
    /// Un compte de bilan neuf, jamais mouvementé.
    async fn compte(&self, numero: &str, nature: AccountType, role: Option<AccountRole>) -> i64 {
        accounts::create(
            &self.pool,
            self.s.admin_user_id,
            NewAccount {
                company_id: self.s.company_id,
                number: numero.into(),
                name: format!("Compte {numero}"),
                account_type: nature,
                parent_id: None,
                role,
                postable: true,
            },
        )
        .await
        .unwrap()
        .id
    }

    /// Un compte bancaire posé directement, lié à `journal_account_id`.
    async fn compte_bancaire(&self, iban: &str, principal: bool, journal_account_id: i64) -> i64 {
        sqlx::query(
            "INSERT INTO bank_accounts (company_id, bank_name, iban, is_primary, journal_account_id) \
             VALUES (?, 'Banque Test', ?, ?, ?)",
        )
        .bind(self.s.company_id)
        .bind(iban)
        .bind(principal)
        .bind(journal_account_id)
        .execute(&self.pool)
        .await
        .unwrap()
        .last_insert_id() as i64
    }

    /// `(date, exercice)` d'une écriture.
    async fn date_et_exercice(&self, entry_id: i64) -> (NaiveDate, i64) {
        sqlx::query_as("SELECT entry_date, fiscal_year_id FROM journal_entries WHERE id = ?")
            .bind(entry_id)
            .fetch_one(&self.pool)
            .await
            .unwrap()
    }

    /// L'écriture qui contre-passe `entry_id`, s'il y en a une.
    async fn inverse_de(&self, entry_id: i64) -> Option<i64> {
        sqlx::query_scalar("SELECT id FROM journal_entries WHERE reverses_entry_id = ?")
            .bind(entry_id)
            .fetch_optional(&self.pool)
            .await
            .unwrap()
    }

    async fn nombre_d_ecritures(&self) -> i64 {
        sqlx::query_scalar("SELECT COUNT(*) FROM journal_entries")
            .fetch_one(&self.pool)
            .await
            .unwrap()
    }

    /// L'inverse de `entry_id` est datée du `2026-01-01`, dans l'exercice 2026.
    async fn assert_contre_passee_le_1er_janvier(&self, entry_id: i64, chemin: &str) {
        let inverse = self
            .inverse_de(entry_id)
            .await
            .unwrap_or_else(|| panic!("{chemin} : aucune contre-passation de {entry_id}"));
        assert_eq!(
            self.date_et_exercice(inverse).await,
            (jour(2026, 1, 1), self.fy_2026.unwrap()),
            "{chemin} : la contre-passation est datée du jour suisse, dans l'exercice 2026"
        );
    }

    /// Une écriture OD de 100.– au `2025-06-01`, passée par le dépôt.
    async fn ecriture(&self) -> i64 {
        journal_entries::create(
            &self.pool,
            self.fy_2025,
            self.s.admin_user_id,
            NewJournalEntry {
                company_id: self.s.company_id,
                entry_date: jour(2025, 6, 1),
                journal: Journal::OD,
                description: "Écriture à contre-passer".into(),
                project_id: None,
                lines: vec![
                    NewJournalEntryLine {
                        account_id: self.s.accounts["4000"],
                        debit: dec!(100.00),
                        credit: dec!(0),
                        project_id: None,
                    },
                    NewJournalEntryLine {
                        account_id: self.s.accounts["1000"],
                        debit: dec!(0),
                        credit: dec!(100.00),
                        project_id: None,
                    },
                ],
            },
        )
        .await
        .expect("écriture d'origine")
        .entry
        .id
    }

    async fn contact(&self, nom: &str, fournisseur: bool) -> i64 {
        contacts::create(
            &self.pool,
            self.s.admin_user_id,
            NewContact {
                company_id: self.s.company_id,
                contact_type: ContactType::Entreprise,
                name: nom.into(),
                first_name: None,
                last_name: None,
                is_client: !fournisseur,
                is_supplier: fournisseur,
                address: Some("Rue 2\n1000 Lausanne".into()),
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

    /// Une facture client du `2025-12-15`, validée par les routes (100.– HT,
    /// 8,1 % → 108.10).
    async fn facture_validee(&self) -> i64 {
        let client = self.contact("Client SA", false).await;
        let (s, b) = self
            .app
            .post(
                "/api/v1/invoices",
                json!({
                    "contactId": client,
                    "date": "2025-12-15",
                    "lines": [
                        {"description": "Conseil", "quantity": "1", "unitPrice": "100.00", "vatRate": "8.10"}
                    ]
                }),
            )
            .await;
        assert_eq!(s, 201, "facture : {b}");
        let id = b["id"].as_i64().unwrap();
        let (s, b) = self
            .app
            .post_vide(&format!("/api/v1/invoices/{id}/validate"))
            .await;
        assert_eq!(s, 200, "validation : {b}");
        id
    }

    /// Règle la facture (108.10) le `2025-12-20` par virement : `(règlement,
    /// écriture du règlement)`.
    async fn regler(&self, invoice: i64, bank: i64) -> (i64, i64) {
        let (s, b) = self
            .app
            .post(
                &format!("/api/v1/invoices/{invoice}/settlements"),
                json!({
                    "settlementType": "bank_transfer",
                    "bankAccountId": bank,
                    "amount": "108.10",
                    "settledOn": "2025-12-20",
                    "version": common::invoice_version(&self.pool, invoice).await,
                }),
            )
            .await;
        assert!((200..300).contains(&s), "règlement : {b}");
        sqlx::query_as(
            "SELECT id, journal_entry_id FROM invoice_settlements WHERE invoice_id = ? \
             ORDER BY id DESC LIMIT 1",
        )
        .bind(invoice)
        .fetch_one(&self.pool)
        .await
        .unwrap()
    }

    /// Une facture fournisseur validée du `2025-12-15`, 100.– sans TVA.
    async fn facture_fournisseur(&self) -> i64 {
        let fournisseur = self.contact("Fournisseur SA", true).await;
        supplier_invoices::create_validated(
            &self.pool,
            NewSupplierInvoice {
                company_id: self.s.company_id,
                contact_id: fournisseur,
                supplier_invoice_number: Some("FF-NUIT".into()),
                invoice_date: jour(2025, 12, 15),
                due_date: None,
                creditor_iban: None,
                creditor_qr_iban: None,
                payment_reference: None,
                expected_payment_amount: None,
                project_id: None,
                lines: vec![NewSupplierInvoiceLine {
                    description: "Prestation".into(),
                    quantity: dec!(1),
                    unit_price: dec!(100.00),
                    vat_rate: dec!(0),
                    expense_account_id: self.s.accounts["4000"],
                }],
            },
            self.s.admin_user_id,
        )
        .await
        .expect("facture fournisseur")
        .invoice
        .id
    }

    /// `(écriture d'achat, écriture du règlement)` d'une facture fournisseur.
    async fn ecritures_fournisseur(&self, id: i64) -> (Option<i64>, Option<i64>) {
        sqlx::query_as(
            "SELECT purchase_journal_entry_id, settlement_journal_entry_id \
             FROM supplier_invoices WHERE id = ?",
        )
        .bind(id)
        .fetch_one(&self.pool)
        .await
        .unwrap()
    }

    /// Une transaction `pending` de `montant`, le `2025-12-20`, importée par le
    /// dépôt : son identifiant.
    async fn transaction(&self, bank: i64, montant: &str) -> i64 {
        let d = jour(2025, 12, 20);
        let mut tx = self.pool.begin().await.unwrap();
        let (_, txs) = bank_imports::create_with_transactions(
            &mut tx,
            NewBankImport {
                company_id: self.s.company_id,
                bank_account_id: bank,
                filename: "releve.xml".into(),
                file_hash: format!("{:0>64}", "nuit"),
                source_format: BankImportSourceFormat::Camt053V04,
                statement_id: Some("NUIT-1".into()),
                period_from: d,
                period_to: d,
                opening_balance: None,
                closing_balance: None,
                transaction_count: 1,
                imported_by_user_id: self.s.admin_user_id,
            },
            vec![NewBankTransaction {
                company_id: self.s.company_id,
                bank_account_id: bank,
                booking_date: d,
                value_date: Some(d),
                amount: rust_decimal::Decimal::from_str(montant).unwrap(),
                currency: "CHF".into(),
                reference: Some("REF-NUIT".into()),
                details: "Mouvement".into(),
                end_to_end_id: None,
                transaction_id: None,
                counterparty_iban: None,
                counterparty_name: None,
            }],
        )
        .await
        .expect("import");
        tx.commit().await.unwrap();
        txs[0].id
    }
}

const IBAN_A: &str = "CH9300762011623852957";
const IBAN_B: &str = "CH5604835012345678009";

// ============================================================
// AC2 — solde d'ouverture et complément des soldes de départ (#1022)
// ============================================================

/// AC2 — exercice 2025 premier, son premier jour verrouillé (borne au
/// `2025-06-30`, posée en SQL) : la branche (b) date du **jour**. À 00 h 30 le
/// 1er janvier à Zurich, le solde d'ouverture d'un compte créé, celui d'un
/// compte existant, le statut et le complément des *Soldes de départ* sont
/// datés du `2026-01-01`, dans l'exercice 2026 (le jour UTC les mettait au
/// 31 décembre, dans 2025). Tue M1, M3.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn ac2_solde_d_ouverture_la_nuit_du_nouvel_an(pool: MySqlPool) {
    let ctx = monter(&pool, NUIT_DU_NOUVEL_AN, Some("Open")).await;
    let fy_2026 = ctx.fy_2026.unwrap();
    ctx.compte(
        "2970",
        AccountType::Liability,
        Some(AccountRole::RetainedEarnings),
    )
    .await;
    sqlx::query("UPDATE companies SET books_locked_through = '2025-06-30' WHERE id = ?")
        .bind(ctx.s.company_id)
        .execute(&pool)
        .await
        .unwrap();

    // Création d'un compte bancaire avec son solde (site 1).
    let (s, b) = ctx
        .app
        .post(
            "/api/v1/bank-accounts",
            json!({
                "bankName": "UBS",
                "iban": IBAN_A,
                "journalAccountId": ctx.ledger,
                "openingBalance": { "amount": "500.00", "side": "debit" },
            }),
        )
        .await;
    assert_eq!(s, 201, "{b}");
    assert_eq!(b["openingEntry"]["entryDate"], json!("2026-01-01"), "{b}");
    assert_eq!(b["openingEntry"]["fiscalYearId"], json!(fy_2026), "{b}");

    // Solde d'un compte existant (site 2).
    let lie = ctx.compte("1021", AccountType::Asset, None).await;
    let existant = ctx.compte_bancaire(IBAN_B, false, lie).await;
    let (s, b) = ctx
        .app
        .post(
            &format!("/api/v1/bank-accounts/{existant}/opening-balance"),
            json!({ "amount": "75.25", "side": "debit" }),
        )
        .await;
    assert_eq!(s, 201, "{b}");
    assert_eq!(b["entryDate"], json!("2026-01-01"), "{b}");
    assert_eq!(b["fiscalYearId"], json!(fy_2026), "{b}");

    // Statut du complément (site 3).
    let (s, b) = ctx.app.get("/api/v1/opening-balances/status").await;
    assert_eq!(s, 200, "{b}");
    assert_eq!(b["complementDate"], json!("2026-01-01"), "{b}");
    assert_eq!(b["complementDateKind"], json!("TODAY"), "{b}");

    // Complément (site 4) : la caisse 1000, jamais mouvementée.
    let (s, b) = ctx
        .app
        .post(
            "/api/v1/opening-balances/complete",
            json!({ "lines": [ { "accountId": ctx.s.accounts["1000"], "debit": "5", "credit": "0" } ] }),
        )
        .await;
    assert_eq!(s, 201, "{b}");
    let id = b["id"].as_i64().unwrap_or_else(|| panic!("{b}"));
    assert_eq!(
        ctx.date_et_exercice(id).await,
        (jour(2026, 1, 1), fy_2026),
        "complément : {b}"
    );
}

// ============================================================
// AC3 — contre-passation et annulations la nuit (#929)
// ============================================================

/// AC3 — `POST /journal-entries/{id}/reverse` : l'inverse est datée du
/// `2026-01-01` dans l'exercice 2026 (site 5).
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn ac3_contre_passation_la_nuit_du_nouvel_an(pool: MySqlPool) {
    let ctx = monter(&pool, NUIT_DU_NOUVEL_AN, Some("Open")).await;
    let origine = ctx.ecriture().await;
    let (s, b) = ctx
        .app
        .post(
            &format!("/api/v1/journal-entries/{origine}/reverse"),
            json!({}),
        )
        .await;
    assert_eq!(s, 201, "{b}");
    assert_eq!(b["entryDate"], json!("2026-01-01"), "{b}");
    ctx.assert_contre_passee_le_1er_janvier(origine, "reverse")
        .await;
}

/// AC3 — annulation d'un règlement de facture client (rang 5 jugé au jour
/// suisse, puis la contre-passation datée du même jour). Tue M7.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn ac3_annulation_d_un_reglement_client_la_nuit(pool: MySqlPool) {
    let ctx = monter(&pool, NUIT_DU_NOUVEL_AN, Some("Open")).await;
    let bank = ctx.compte_bancaire(IBAN_A, true, ctx.ledger).await;
    let facture = ctx.facture_validee().await;
    let (reglement, ecriture) = ctx.regler(facture, bank).await;
    let (s, b) = ctx
        .app
        .post_vide(&format!(
            "/api/v1/invoices/{facture}/settlements/{reglement}/cancel"
        ))
        .await;
    assert_eq!(s, 200, "{b}");
    ctx.assert_contre_passee_le_1er_janvier(ecriture, "règlement client")
        .await;
}

/// AC3 — annulation d'une facture fournisseur validée. Tue M7.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn ac3_annulation_d_une_facture_fournisseur_la_nuit(pool: MySqlPool) {
    let ctx = monter(&pool, NUIT_DU_NOUVEL_AN, Some("Open")).await;
    let id = ctx.facture_fournisseur().await;
    let (achat, _) = ctx.ecritures_fournisseur(id).await;
    let (s, b) = ctx
        .app
        .post(&format!("/api/v1/supplier-invoices/{id}/cancel"), json!({}))
        .await;
    assert_eq!(s, 200, "{b}");
    ctx.assert_contre_passee_le_1er_janvier(achat.unwrap(), "facture fournisseur")
        .await;
}

/// AC3 — annulation du règlement d'une facture fournisseur. Tue M7.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn ac3_annulation_d_un_reglement_fournisseur_la_nuit(pool: MySqlPool) {
    let ctx = monter(&pool, NUIT_DU_NOUVEL_AN, Some("Open")).await;
    let bank = ctx.compte_bancaire(IBAN_A, true, ctx.ledger).await;
    let id = ctx.facture_fournisseur().await;
    let (s, b) = ctx
        .app
        .post(
            &format!("/api/v1/supplier-invoices/{id}/pay"),
            json!({
                "settlementType": "bank_transfer",
                "bankAccountId": bank,
                "paymentDate": "2025-12-20"
            }),
        )
        .await;
    assert_eq!(s, 200, "paiement : {b}");
    let (_, reglement) = ctx.ecritures_fournisseur(id).await;
    let (s, b) = ctx
        .app
        .post(
            &format!("/api/v1/supplier-invoices/{id}/settlement/cancel"),
            json!({}),
        )
        .await;
    assert_eq!(s, 200, "{b}");
    ctx.assert_contre_passee_le_1er_janvier(reglement.unwrap(), "règlement fournisseur")
        .await;
}

/// AC3 — annulation d'un rapprochement manuel. Tue M7.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn ac3_annulation_d_un_rapprochement_la_nuit(pool: MySqlPool) {
    let ctx = monter(&pool, NUIT_DU_NOUVEL_AN, Some("Open")).await;
    let bank = ctx.compte_bancaire(IBAN_A, true, ctx.ledger).await;
    let t = ctx.transaction(bank, "-100.00").await;
    let (s, b) = ctx
        .app
        .post(
            "/api/v1/reconciliation/manual",
            json!({
                "bankAccountId": bank,
                "bankTransactionId": t,
                "counterpartyAccountId": ctx.s.accounts["4000"],
                "description": "Frais",
            }),
        )
        .await;
    assert_eq!(s, 200, "rapprochement : {b}");
    let ecriture: i64 =
        sqlx::query_scalar("SELECT matched_entry_id FROM bank_transactions WHERE id = ?")
            .bind(t)
            .fetch_one(&pool)
            .await
            .unwrap();
    let (s, b) = ctx
        .app
        .post_vide(&format!("/api/v1/reconciliation/transactions/{t}/cancel"))
        .await;
    assert_eq!(s, 200, "{b}");
    ctx.assert_contre_passee_le_1er_janvier(ecriture, "rapprochement")
        .await;
}

/// AC3 — le refus : seul l'exercice 2025 existe ; aucun ne couvre le
/// `2026-01-01`. L'annulation du règlement est refusée au rang 5 par son refus
/// nommé, et rien n'est écrit (le jour UTC acceptait et contre-passait au
/// 31 décembre).
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn ac3_sans_exercice_au_jour_suisse_l_annulation_est_refusee(pool: MySqlPool) {
    let ctx = monter(&pool, NUIT_DU_NOUVEL_AN, None).await;
    let bank = ctx.compte_bancaire(IBAN_A, true, ctx.ledger).await;
    let facture = ctx.facture_validee().await;
    let (reglement, ecriture) = ctx.regler(facture, bank).await;

    // Le motif annoncé par la liste (AC7 en partie) : le rang 5.
    let (s, b) = ctx
        .app
        .get(&format!("/api/v1/invoices/{facture}/settlements"))
        .await;
    assert_eq!(s, 200, "{b}");
    assert_eq!(b[0]["cancelBlockedBy"], json!("FISCAL_YEAR_INVALID"), "{b}");

    let avant = ctx.nombre_d_ecritures().await;
    let (s, b) = ctx
        .app
        .post_vide(&format!(
            "/api/v1/invoices/{facture}/settlements/{reglement}/cancel"
        ))
        .await;
    // Le refus du rang 5 au clic (`docs/api-external.md`, table de l'annulation) :
    // `400 FISCAL_YEAR_INVALID`.
    assert_eq!(
        (s, b["error"]["code"].clone()),
        (400, json!("FISCAL_YEAR_INVALID")),
        "refus du rang 5 attendu : {b}"
    );
    assert_eq!(ctx.nombre_d_ecritures().await, avant, "rien n'est écrit");
    assert_eq!(ctx.inverse_de(ecriture).await, None);
}

// ============================================================
// AC4 — verrou de période (#907)
// ============================================================

/// AC4 — à 00 h 30 le 11 mars à Zurich (`2026-03-10T23:30:00Z`), poser le
/// verrou jusqu'au `2026-03-10` est accepté ; jusqu'au `2026-03-11`, refusé
/// (`400 INVALID_INPUT`, `settings-books-lock-bound-not-past`). Tue M4.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn ac4_verrouiller_jusqu_a_hier_la_nuit(pool: MySqlPool) {
    let ctx = monter(&pool, "2026-03-10T23:30:00Z", Some("Open")).await;
    let (s, b) = ctx
        .app
        .post(
            "/api/v1/companies/current/books-lock",
            json!({ "through": "2026-03-11" }),
        )
        .await;
    assert_borne_refusee(s, &b);
    let (s, b) = ctx
        .app
        .post(
            "/api/v1/companies/current/books-lock",
            json!({ "through": "2026-03-10" }),
        )
        .await;
    assert_eq!(s, 200, "verrou jusqu'à hier (jour suisse) : {b}");
}

/// AC4 — `unlock_books`, borne courante au `2026-03-10` **posée en SQL** :
/// la cible `2026-03-10` (retrait sans effet, mais tracé) est acceptée ; la
/// cible `2026-03-11` refusée par la même réponse. Tue M4.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn ac4_lever_le_verrou_jusqu_a_hier_la_nuit(pool: MySqlPool) {
    let ctx = monter(&pool, "2026-03-10T23:30:00Z", Some("Open")).await;
    sqlx::query("UPDATE companies SET books_locked_through = '2026-03-10' WHERE id = ?")
        .bind(ctx.s.company_id)
        .execute(&pool)
        .await
        .unwrap();
    let lever = |cible: &'static str| {
        let app = &ctx.app;
        async move {
            app.post(
                "/api/v1/companies/current/books-lock/release",
                json!({ "through": cible, "motif": "Correction d'une pièce" }),
            )
            .await
        }
    };
    let (s, b) = lever("2026-03-11").await;
    assert_borne_refusee(s, &b);
    let (s, b) = lever("2026-03-10").await;
    assert_eq!(s, 200, "cible égale au jour suisse moins un : {b}");
    let traces: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM audit_log WHERE action = 'books.unlocked'")
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(traces, 1, "le retrait sans effet est tracé");
}

fn assert_borne_refusee(s: u16, b: &Value) {
    assert_eq!(s, 400, "{b}");
    assert_eq!(b["error"]["code"], json!("INVALID_INPUT"), "{b}");
    assert!(
        b["error"]["message"]
            .as_str()
            .is_some_and(|m| m.contains("antérieure à aujourd'hui")),
        "message de settings-books-lock-bound-not-past attendu : {b}"
    );
}

// ============================================================
// AC7 — les services lisent le jour suisse (`Lecture`)
// ============================================================

/// AC7 — **aucun exercice ouvert ne couvre le jour suisse** (pas d'exercice
/// 2026) : à 00 h 30 le 1er janvier à Zurich, le règlement de décembre ne
/// s'annule pas (rang 5), et la liste des règlements le dit ; de même la
/// candidate au rattachement d'une transaction (`entry_cancel_outlook`). Le
/// jour UTC (31 décembre, exercice 2025 ouvert) n'annonçait aucun motif.
///
/// ⚠️ Un exercice 2026 **clos** ne convient pas : il fige 2025, qu'il suit
/// (`LATER_FISCAL_YEAR_CLOSED` à l'écriture, `LETTERING_ALL_LINES_IN_CLOSED_PERIODS`
/// — rang 2 bis — à la lecture, quel que soit le jour). C-e355a-3.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn ac7_les_services_jugent_au_jour_suisse(pool: MySqlPool) {
    let ctx = monter(&pool, NUIT_DU_NOUVEL_AN, None).await;
    let bank = ctx.compte_bancaire(IBAN_A, true, ctx.ledger).await;
    let facture = ctx.facture_validee().await;
    ctx.regler(facture, bank).await;

    // Candidate au rattachement : une écriture manuelle sur le 1020, du même
    // montant qu'une transaction en attente.
    let t = ctx.transaction(bank, "42.00").await;
    let candidate = journal_entries::create(
        &pool,
        ctx.fy_2025,
        ctx.s.admin_user_id,
        NewJournalEntry {
            company_id: ctx.s.company_id,
            entry_date: jour(2025, 12, 20),
            journal: Journal::OD,
            description: "Versement".into(),
            project_id: None,
            lines: vec![
                NewJournalEntryLine {
                    account_id: ctx.ledger,
                    debit: dec!(42.00),
                    credit: dec!(0),
                    project_id: None,
                },
                NewJournalEntryLine {
                    account_id: ctx.s.accounts["3000"],
                    debit: dec!(0),
                    credit: dec!(42.00),
                    project_id: None,
                },
            ],
        },
    )
    .await
    .unwrap()
    .entry
    .id;

    let (s, b) = ctx
        .app
        .get(&format!("/api/v1/invoices/{facture}/settlements"))
        .await;
    assert_eq!(s, 200, "{b}");
    assert_eq!(b[0]["cancellable"], json!(false), "{b}");
    assert_eq!(b[0]["cancelBlockedBy"], json!("FISCAL_YEAR_INVALID"), "{b}");

    let (s, b) = ctx
        .app
        .get(&format!(
            "/api/v1/reconciliation/transactions/{t}/entry-candidates"
        ))
        .await;
    assert_eq!(s, 200, "{b}");
    let c = b["candidates"]
        .as_array()
        .unwrap_or_else(|| panic!("{b}"))
        .iter()
        .find(|c| c["journalEntryId"] == json!(candidate))
        .unwrap_or_else(|| panic!("candidate absente : {b}"))
        .clone();
    assert_eq!(c["cancelBlockedBy"], json!("FISCAL_YEAR_INVALID"), "{b}");
}
