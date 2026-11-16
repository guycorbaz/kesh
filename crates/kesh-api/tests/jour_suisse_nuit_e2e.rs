//! Story 35-5a (#1022, #907, #929) — la date du jour est celle de **Zurich**
//! pour ce qui écrit dans les livres ou les borne, même la nuit ; Story 35-5b
//! (#929), en fin de fichier — et pour ce que Kesh juge au jour (accueil,
//! échéances, rappels, noms de fichiers, défauts « aujourd'hui »).
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
            kesh_db::entities::audit_log::Acteur::utilisateur(self.s.admin_user_id),
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

/// Pose du verrou à l'instant `maintenant` : jusqu'à `aujourd_hui` (le jour
/// suisse) refusé, jusqu'à `hier` accepté.
async fn verrouiller_jusqu_a_hier(
    pool: &MySqlPool,
    maintenant: &str,
    hier: &str,
    aujourd_hui: &str,
) {
    let ctx = monter(pool, maintenant, Some("Open")).await;
    let (s, b) = ctx
        .app
        .post(
            "/api/v1/companies/current/books-lock",
            json!({ "through": aujourd_hui }),
        )
        .await;
    assert_borne_refusee(s, &b);
    let (s, b) = ctx
        .app
        .post(
            "/api/v1/companies/current/books-lock",
            json!({ "through": hier }),
        )
        .await;
    assert_eq!(
        s, 200,
        "verrou jusqu'à hier (jour suisse) à {maintenant} : {b}"
    );
}

/// AC4 — à 00 h 30 le 11 mars à Zurich (`2026-03-10T23:30:00Z`), poser le
/// verrou jusqu'au `2026-03-10` est accepté ; jusqu'au `2026-03-11`, refusé
/// (`400 INVALID_INPUT`, `settings-books-lock-bound-not-past`). Tue M4.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn ac4_verrouiller_jusqu_a_hier_la_nuit(pool: MySqlPool) {
    verrouiller_jusqu_a_hier(&pool, "2026-03-10T23:30:00Z", "2026-03-10", "2026-03-11").await;
}

/// AC4, nuit d'**été** (revue P1, E3) — à 00 h 30 le 2 juillet à Zurich
/// (`2026-07-01T22:30:00Z`, CEST) : jusqu'au 1er juillet accepté, jusqu'au 2
/// refusé. Un fuseau fixe à +1 h (M2) lit encore le 1er juillet et refuse.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn ac4_verrouiller_jusqu_a_hier_une_nuit_d_ete(pool: MySqlPool) {
    verrouiller_jusqu_a_hier(&pool, "2026-07-01T22:30:00Z", "2026-07-01", "2026-07-02").await;
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
        kesh_db::entities::audit_log::Acteur::utilisateur(ctx.s.admin_user_id),
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

// ============================================================
// Story 35-5b (#929) — ce que Kesh juge au jour, la nuit
// ============================================================
//
// Mêmes montages : l'horloge du serveur arrêtée sur un instant de nuit. Les
// instants de mars : `2026-03-10T23:30:00Z` est 00 h 30 le **11 mars** à Zurich
// (heure d'hiver), `2026-03-10T22:30:00Z` 23 h 30 le **10**. Le squelette de la
// story (le jour UTC de l'instant) rougit sur chacun des cas de nuit.

/// 00 h 30 le 11 mars 2026 à Zurich ; encore le 10 en UTC.
const NUIT_DU_11_MARS: &str = "2026-03-10T23:30:00Z";
/// 23 h 30 le 10 mars 2026 à Zurich : la veille, partout.
const SOIR_DU_10_MARS: &str = "2026-03-10T22:30:00Z";

impl TestApp {
    /// Un `GET` brut : statut, `Content-Disposition` (vide s'il manque), corps.
    async fn telecharger(&self, chemin: &str) -> (u16, String, Vec<u8>) {
        let resp = self
            .client
            .get(format!("{}{chemin}", self.base_url))
            .bearer_auth(&self.token)
            .send()
            .await
            .unwrap();
        let status = resp.status().as_u16();
        let dispo = resp
            .headers()
            .get(reqwest::header::CONTENT_DISPOSITION)
            .map(|v| v.to_str().unwrap().to_string())
            .unwrap_or_default();
        (status, dispo, resp.bytes().await.unwrap().to_vec())
    }
}

impl Ctx {
    /// Une facture client du `date`, échéance `echeance`, validée par les
    /// routes (100.– HT, 8,1 %) : `(id, numéro)`.
    async fn facture_a_echeance(&self, client: &str, date: &str, echeance: &str) -> (i64, String) {
        let c = self.contact(client, false).await;
        let (s, b) = self
            .app
            .post(
                "/api/v1/invoices",
                json!({
                    "contactId": c,
                    "date": date,
                    "dueDate": echeance,
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
        (id, b["invoiceNumber"].as_str().unwrap().to_string())
    }
}

/// AC1 (#929) — à 00 h 30 le 1er janvier, l'accueil lit l'exercice **2026**,
/// arrêté au 1er janvier (le jour UTC lisait 2025, au 31 décembre).
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn ac1_l_accueil_la_nuit_du_nouvel_an(pool: MySqlPool) {
    let ctx = monter(&pool, NUIT_DU_NOUVEL_AN, Some("Open")).await;
    let (s, b) = ctx.app.get("/api/v1/reports/homepage-summary").await;
    assert_eq!(s, 200, "{b}");
    assert_eq!(
        (
            b["state"].clone(),
            b["asOf"].clone(),
            b["fiscalYear"]["id"].as_i64()
        ),
        (json!("open"), json!("2026-01-01"), ctx.fy_2026),
        "l'accueil lit le nouvel exercice, au jour suisse : {b}"
    );
}

/// AC1, sans exercice 2026 (remédiation P1, E3) — à 00 h 30 le 1er janvier,
/// l'exercice 2025 ne couvre plus le jour suisse : l'accueil dit
/// `noOpenFiscalYear`, arrêté au 1er janvier, sans retomber sur 2025 (C-e332-15 ;
/// le jour UTC montrait encore 2025).
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn ac1_l_accueil_la_nuit_du_nouvel_an_sans_exercice_neuf(pool: MySqlPool) {
    let ctx = monter(&pool, NUIT_DU_NOUVEL_AN, None).await;
    let (s, b) = ctx.app.get("/api/v1/reports/homepage-summary").await;
    assert_eq!(s, 200, "{b}");
    assert_eq!(
        (
            b["state"].clone(),
            b["asOf"].clone(),
            b["fiscalYear"].clone()
        ),
        (json!("noOpenFiscalYear"), json!("2026-01-01"), Value::Null),
        "{b}"
    );
}

/// Ce que chaque surface dit d'une facture, au jour du serveur.
#[derive(Debug, PartialEq)]
struct VueEcheance {
    /// `isOverdue` de la fiche.
    fiche: bool,
    /// `isOverdue` de la liste des factures.
    liste: bool,
    /// `isOverdue` de l'élément rendu par le filtre « échues » de
    /// l'échéancier ; `None` : le filtre ne la rend pas.
    filtre: Option<bool>,
    /// `summary.overdueCount` de l'échéancier.
    echues: i64,
    /// `summary.overdueTotal` de l'échéancier — le second lien du `SELECT` de
    /// `due_dates_summary` (remédiation P1, B3).
    total_echues: rust_decimal::Decimal,
    /// La facture est-elle dans l'export CSV filtré `paymentStatus=overdue`
    /// (`list_for_export`, remédiation P1, B2) ?
    export_echues: bool,
    /// Le statut de paiement de sa ligne du CSV (`paymentStatus=all`).
    statut_csv: String,
    /// Le `Content-Disposition` du CSV.
    nom_csv: String,
}

/// La ligne du CSV de l'échéancier (`paymentStatus=all`) qui commence par
/// `numero`, découpée, et le `Content-Disposition`.
async fn ligne_csv(app: &TestApp, numero: &str) -> (Vec<String>, String) {
    let (s, dispo, corps) = app
        .telecharger("/api/v1/invoices/due-dates/export.csv?paymentStatus=all")
        .await;
    assert_eq!(s, 200);
    let texte = String::from_utf8(corps).unwrap();
    let ligne = texte
        .lines()
        .find(|l| l.split(';').next() == Some(numero))
        .unwrap_or_else(|| panic!("{numero} absente du CSV : {texte}"));
    (ligne.split(';').map(str::to_string).collect(), dispo)
}

async fn vue_echeance(app: &TestApp, id: i64, numero: &str) -> VueEcheance {
    let (s, f) = app.get(&format!("/api/v1/invoices/{id}")).await;
    assert_eq!(s, 200, "{f}");
    let (s, l) = app.get("/api/v1/invoices?limit=100").await;
    assert_eq!(s, 200, "{l}");
    let liste = l["items"]
        .as_array()
        .unwrap()
        .iter()
        .find(|i| i["id"] == id)
        .expect("la facture est dans la liste")["isOverdue"]
        .as_bool()
        .unwrap();
    let (s, e) = app
        .get("/api/v1/invoices/due-dates?paymentStatus=overdue&limit=100")
        .await;
    assert_eq!(s, 200, "{e}");
    let filtre = e["items"]
        .as_array()
        .unwrap()
        .iter()
        .find(|i| i["id"] == id)
        .map(|i| i["isOverdue"].as_bool().unwrap());
    let (ligne, dispo) = ligne_csv(app, numero).await;
    let (s, _, corps) = app
        .telecharger("/api/v1/invoices/due-dates/export.csv?paymentStatus=overdue")
        .await;
    assert_eq!(s, 200);
    let export_echues = String::from_utf8(corps)
        .unwrap()
        .lines()
        .any(|l| l.split(';').next() == Some(numero));
    VueEcheance {
        fiche: f["isOverdue"].as_bool().unwrap(),
        liste,
        filtre,
        echues: e["summary"]["overdueCount"].as_i64().unwrap(),
        total_echues: e["summary"]["overdueTotal"]
            .as_str()
            .unwrap_or_else(|| panic!("overdueTotal : {e}"))
            .parse()
            .unwrap(),
        export_echues,
        statut_csv: ligne[6].clone(),
        nom_csv: dispo,
    }
}

/// AC2 — une facture échue le `2026-03-10` : à 00 h 30 le 11 mars, la fiche,
/// la liste, le filtre « échues » (élément compris), le résumé (nombre et
/// montant), l'export filtré « échues » et le CSV la disent échue, et le CSV
/// porte le nom du 11 ; à 23 h 30 le 10, non échue partout. Une même date sert
/// le filtre, le résumé et la conversion (D1, D2).
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn ac2_une_facture_echue_la_nuit(pool: MySqlPool) {
    let ctx = monter(&pool, NUIT_DU_11_MARS, Some("Open")).await;
    let (id, numero) = ctx
        .facture_a_echeance("Client Échéance SA", "2026-03-01", "2026-03-10")
        .await;
    let nuit = vue_echeance(&ctx.app, id, &numero).await;
    let soir = spawn_app(&pool, instant(SOIR_DU_10_MARS)).await;
    let veille = vue_echeance(&soir, id, &numero).await;
    assert_eq!(
        (nuit, veille),
        (
            VueEcheance {
                fiche: true,
                liste: true,
                filtre: Some(true),
                echues: 1,
                total_echues: dec!(108.10),
                export_echues: true,
                statut_csv: "En retard".into(),
                nom_csv: "attachment; filename=\"echeancier-2026-03-11.csv\"".into(),
            },
            VueEcheance {
                fiche: false,
                liste: false,
                filtre: None,
                echues: 0,
                total_echues: dec!(0),
                export_echues: false,
                statut_csv: "Impayée".into(),
                nom_csv: "attachment; filename=\"echeancier-2026-03-10.csv\"".into(),
            },
        ),
        "(00 h 30 le 11 mars, 23 h 30 le 10 mars)"
    );
}

/// AC2, minuit pile en heure d'été (remédiation P1, E6) — une facture échue le
/// `2026-07-01` : à `2026-07-01T21:59:59Z` (23 h 59 min 59 s le 1er juillet à
/// Zurich) la fiche ne la dit pas échue ; à `2026-07-01T22:00:00Z` (minuit pile
/// le 2 juillet, UTC+2) elle la dit échue.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn ac2_minuit_pile_en_heure_d_ete(pool: MySqlPool) {
    let ctx = monter(&pool, "2026-07-01T21:59:59Z", Some("Open")).await;
    let (id, _) = ctx
        .facture_a_echeance("Client Été SA", "2026-06-15", "2026-07-01")
        .await;
    let minuit = spawn_app(&pool, instant("2026-07-01T22:00:00Z")).await;
    let mut vues = Vec::new();
    for app in [&ctx.app, &minuit] {
        let (s, f) = app.get(&format!("/api/v1/invoices/{id}")).await;
        assert_eq!(s, 200, "{f}");
        vues.push(f["isOverdue"].as_bool().unwrap());
    }
    assert_eq!(
        vues,
        [false, true],
        "(23:59:59 le 1er juillet, minuit le 2)"
    );
}

/// AC2, paiement (site 16 bis, remédiation P1 E1) — `paid_at` n'est pas un
/// instant : les écrivains y posent la **date du règlement à minuit**
/// (`invoice_settlements_write.rs`, `reconciliation.rs`). Une facture réglée par
/// la route à 00 h 30 le 11 mars, `settledOn` du **10**, porte `10.03.2026` dans
/// la colonne de paiement du CSV (`paymentStatus=all`) : la date du règlement,
/// non le jour de la nuit où on l'a saisi.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn ac2_la_date_de_paiement_du_csv_est_celle_du_reglement(pool: MySqlPool) {
    let ctx = monter(&pool, NUIT_DU_11_MARS, Some("Open")).await;
    let bank = ctx.compte_bancaire(IBAN_A, true, ctx.ledger).await;
    let (id, numero) = ctx
        .facture_a_echeance("Client Payé SA", "2026-03-01", "2026-03-31")
        .await;
    let (s, b) = ctx
        .app
        .post(
            &format!("/api/v1/invoices/{id}/settlements"),
            json!({
                "settlementType": "bank_transfer",
                "bankAccountId": bank,
                "amount": "108.10",
                "settledOn": "2026-03-10",
                "version": common::invoice_version(&ctx.pool, id).await,
            }),
        )
        .await;
    assert!((200..300).contains(&s), "règlement : {b}");
    let paid_at: chrono::NaiveDateTime =
        sqlx::query_scalar("SELECT paid_at FROM invoices WHERE id = ?")
            .bind(id)
            .fetch_one(&ctx.pool)
            .await
            .unwrap();
    let (ligne, _) = ligne_csv(&ctx.app, &numero).await;
    assert_eq!(
        (paid_at.to_string(), ligne[6].as_str(), ligne[7].as_str()),
        ("2026-03-10 00:00:00".to_string(), "Payée", "10.03.2026"),
        "(paid_at écrit par le règlement, statut, colonne de paiement) : {ligne:?}"
    );
}

/// Le corps de l'aperçu du rappel de niveau 1 de `id`.
async fn apercu_du_rappel(app: &TestApp, id: i64) -> String {
    let (s, b) = app
        .get(&format!("/api/v1/invoices/{id}/reminder-preview?level=1"))
        .await;
    assert_eq!(s, 200, "{b}");
    b["body"].as_str().unwrap().to_string()
}

/// `(niveau courant, niveau suivant)` de la facture `id` dans la liste « à
/// rappeler » ; `None` : elle n'y est pas.
async fn a_rappeler(app: &TestApp, id: i64) -> Option<(i64, Option<i64>)> {
    let (s, b) = app.get("/api/v1/dunning/reminders").await;
    assert_eq!(s, 200, "{b}");
    b["groups"]
        .as_array()
        .unwrap()
        .iter()
        .flat_map(|g| g["invoices"].as_array().unwrap().iter())
        .find(|i| i["invoiceId"] == id)
        .map(|i| (i["currentLevel"].as_i64().unwrap(), i["nextLevel"].as_i64()))
}

/// AC3 — les rappels comptent en jours suisses.
///
/// - `days_overdue` d'une facture échue le `2026-03-01` vaut **9** à 23 h 30 le
///   10 mars et **10** à 00 h 30 le 11 (le jour UTC disait 9 aux deux).
/// - Dernier rappel envoyé à `2026-03-10T23:30:00Z` (le 11 à Zurich), délai du
///   niveau 2 ramené à 7 jours : à 23 h 30 le 17 mars, la facture n'est **pas**
///   à rappeler (11 + 7 = le 18) ; à 00 h 30 le 18, elle l'est, au niveau 2. Le
///   jour UTC la rappelait dès le 17 (10 + 7). Les deux jours sortent de
///   `jour_suisse` (D4). Pas d'envoi automatique : l'effet est la liste.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn ac3_les_rappels_comptent_en_jours_suisses(pool: MySqlPool) {
    let ctx = monter(&pool, SOIR_DU_10_MARS, Some("Open")).await;
    let (id, _) = ctx
        .facture_a_echeance("Client Rappel SA", "2026-02-15", "2026-03-01")
        .await;
    // Le premier accès sème les niveaux de rappel de la société.
    a_rappeler(&ctx.app, id).await;
    let nuit = spawn_app(&pool, instant(NUIT_DU_11_MARS)).await;
    let (soir_10, nuit_11) = (
        apercu_du_rappel(&ctx.app, id).await,
        apercu_du_rappel(&nuit, id).await,
    );
    assert!(
        soir_10.contains("(9 jours de retard)") && nuit_11.contains("(10 jours de retard)"),
        "jours de retard : 9 à 23 h 30 le 10 mars, 10 à 00 h 30 le 11 — {soir_10:?} / {nuit_11:?}"
    );

    sqlx::query(
        "UPDATE dunning_levels SET delay_days = 7 WHERE company_id = ? AND level_number = 2",
    )
    .bind(ctx.s.company_id)
    .execute(&pool)
    .await
    .unwrap();
    sqlx::query(
        "INSERT INTO invoice_reminders (company_id, invoice_id, level_number, fee_amount, \
         sent_at, channel, subject, body) \
         VALUES (?, ?, 1, 0.00, '2026-03-10 23:30:00', 'manual', 'Rappel', 'Rappel')",
    )
    .bind(ctx.s.company_id)
    .bind(id)
    .execute(&pool)
    .await
    .unwrap();
    let le_17 = spawn_app(&pool, instant("2026-03-17T22:30:00Z")).await;
    let le_18 = spawn_app(&pool, instant("2026-03-17T23:30:00Z")).await;
    assert_eq!(
        (a_rappeler(&le_17, id).await, a_rappeler(&le_18, id).await),
        (None, Some((1, Some(2)))),
        "à rappeler : pas le 17 à 23 h 30, le 18 à 00 h 30 (rappel du 11 + 7 jours)"
    );
}

/// AC4 — à 00 h 30 le 1er janvier, les noms des fichiers exportés portent le
/// `2026-01-01` : sauvegarde, export comptable complet, journal d'audit.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn ac4_les_noms_de_fichiers_la_nuit_du_nouvel_an(pool: MySqlPool) {
    let ctx = monter(&pool, NUIT_DU_NOUVEL_AN, Some("Open")).await;
    let mut noms = Vec::new();
    for chemin in [
        "/api/v1/admin/full-export",
        "/api/v1/exports/global.zip",
        "/api/v1/audit-log/export.csv",
    ] {
        let (s, dispo, _) = ctx.app.telecharger(chemin).await;
        assert_eq!(s, 200, "{chemin}");
        noms.push(dispo);
    }
    assert!(
        noms[0].contains("filename=\"kesh-installation-2026-01-01.keshbackup\"")
            && noms[1].contains("-2026-01-01.zip\"")
            && noms[2].contains("-2026-01-01.csv\""),
        "{noms:?}"
    );
}

/// AC4 — à 00 h 30 le 1er janvier, les défauts « aujourd'hui » valent le
/// `2026-01-01` : la date de clôture d'un compte bancaire sans `closedOn`,
/// l'`asOf` des postes ouverts et de la balance âgée ; le rapport de projet
/// cumulé d'un projet sans date de fin inclut l'écriture du 1er janvier (le
/// jour UTC l'arrêtait au 31 décembre).
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn ac4_les_defauts_du_jour_la_nuit_du_nouvel_an(pool: MySqlPool) {
    let ctx = monter(&pool, NUIT_DU_NOUVEL_AN, Some("Open")).await;

    ctx.compte_bancaire(IBAN_A, true, ctx.ledger).await;
    let second = ctx.compte_bancaire(IBAN_B, false, ctx.ledger).await;
    let version: i32 = sqlx::query_scalar("SELECT version FROM bank_accounts WHERE id = ?")
        .bind(second)
        .fetch_one(&pool)
        .await
        .unwrap();
    let (s, b) = ctx
        .app
        .post(
            &format!("/api/v1/bank-accounts/{second}/close"),
            json!({ "version": version }),
        )
        .await;
    assert_eq!(s, 200, "{b}");
    let cloture = b["closedOn"].clone();

    let (s, postes) = ctx
        .app
        .get(&format!(
            "/api/v1/accounts/{}/open-items",
            ctx.s.accounts["1100"]
        ))
        .await;
    assert_eq!(s, 200, "{postes}");
    let (s, agee) = ctx.app.get("/api/v1/reports/aged-receivables").await;
    assert_eq!(s, 200, "{agee}");

    let projet = sqlx::query(
        "INSERT INTO projects (company_id, code, name) VALUES (?, 'NUIT', 'Projet de nuit')",
    )
    .bind(ctx.s.company_id)
    .execute(&pool)
    .await
    .unwrap()
    .last_insert_id() as i64;
    journal_entries::create(
        &pool,
        ctx.fy_2026.unwrap(),
        kesh_db::entities::audit_log::Acteur::utilisateur(ctx.s.admin_user_id),
        NewJournalEntry {
            company_id: ctx.s.company_id,
            entry_date: jour(2026, 1, 1),
            journal: Journal::OD,
            description: "Charge du 1er janvier".into(),
            project_id: Some(projet),
            lines: vec![
                NewJournalEntryLine {
                    account_id: ctx.s.accounts["4000"],
                    debit: dec!(100.00),
                    credit: dec!(0),
                    project_id: Some(projet),
                },
                NewJournalEntryLine {
                    account_id: ctx.s.accounts["1000"],
                    debit: dec!(0),
                    credit: dec!(100.00),
                    project_id: None,
                },
            ],
        },
    )
    .await
    .expect("écriture du 1er janvier");
    let (s, rapport) = ctx
        .app
        .get(&format!(
            "/api/v1/reports/project-expenses?projectId={projet}&mode=cumulative"
        ))
        .await;
    assert_eq!(s, 200, "{rapport}");
    let total: rust_decimal::Decimal = rapport["grandTotal"]
        .as_str()
        .unwrap_or_else(|| panic!("{rapport}"))
        .parse()
        .unwrap();

    assert_eq!(
        (cloture, postes["asOf"].clone(), agee["asOf"].clone(), total),
        (
            json!("2026-01-01"),
            json!("2026-01-01"),
            json!("2026-01-01"),
            dec!(100)
        ),
        "(closedOn, asOf des postes ouverts, asOf de la balance âgée, total du projet cumulé)"
    );
}

/// AC4, projet à date de fin (remédiation P1, E4) — en mode cumulé, une date de
/// fin donnée l'emporte sur la date du jour : à 00 h 30 le 1er janvier 2026, un
/// projet clos le `2026-06-30` compte son écriture du `2026-06-01`, postérieure
/// au jour (une fin prise à « aujourd'hui » l'exclurait).
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn ac4_la_fin_donnee_d_un_projet_l_emporte_sur_le_jour(pool: MySqlPool) {
    let ctx = monter(&pool, NUIT_DU_NOUVEL_AN, Some("Open")).await;
    let projet = sqlx::query(
        "INSERT INTO projects (company_id, code, name, end_date) \
         VALUES (?, 'FIN', 'Projet à fin donnée', '2026-06-30')",
    )
    .bind(ctx.s.company_id)
    .execute(&pool)
    .await
    .unwrap()
    .last_insert_id() as i64;
    journal_entries::create(
        &pool,
        ctx.fy_2026.unwrap(),
        kesh_db::entities::audit_log::Acteur::utilisateur(ctx.s.admin_user_id),
        NewJournalEntry {
            company_id: ctx.s.company_id,
            entry_date: jour(2026, 6, 1),
            journal: Journal::OD,
            description: "Charge de juin".into(),
            project_id: Some(projet),
            lines: vec![
                NewJournalEntryLine {
                    account_id: ctx.s.accounts["4000"],
                    debit: dec!(100.00),
                    credit: dec!(0),
                    project_id: Some(projet),
                },
                NewJournalEntryLine {
                    account_id: ctx.s.accounts["1000"],
                    debit: dec!(0),
                    credit: dec!(100.00),
                    project_id: None,
                },
            ],
        },
    )
    .await
    .expect("écriture de juin");
    let (s, rapport) = ctx
        .app
        .get(&format!(
            "/api/v1/reports/project-expenses?projectId={projet}&mode=cumulative"
        ))
        .await;
    assert_eq!(s, 200, "{rapport}");
    let total: rust_decimal::Decimal = rapport["grandTotal"]
        .as_str()
        .unwrap_or_else(|| panic!("{rapport}"))
        .parse()
        .unwrap();
    assert_eq!(total, dec!(100), "{rapport}");
}
