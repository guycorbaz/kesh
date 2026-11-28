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
                currency: None,
                own_fx_accounts: None,
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

    /// Story 63-1 (AC-11) — l'inverse de `entry_id` est datée **de l'origine**,
    /// dans son exercice (2025), et non du jour (le 1er janvier 2026).
    async fn assert_contre_passee_a_l_origine(&self, entry_id: i64, chemin: &str) {
        let inverse = self
            .inverse_de(entry_id)
            .await
            .unwrap_or_else(|| panic!("{chemin} : aucune contre-passation de {entry_id}"));
        let origine = self.date_et_exercice(entry_id).await;
        assert_eq!(
            origine.1, self.fy_2025,
            "{chemin} : montage, l'origine est dans 2025"
        );
        assert_eq!(
            self.date_et_exercice(inverse).await,
            origine,
            "{chemin} : la contre-passation est datée de l'origine, dans son exercice"
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
                        foreign: None,
                    },
                    NewJournalEntryLine {
                        account_id: self.s.accounts["1000"],
                        debit: dec!(0),
                        credit: dec!(100.00),
                        project_id: None,
                        foreign: None,
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
        self.regler_montant(invoice, bank, "108.10").await
    }

    /// Règle `montant` de la facture le `2025-12-20` par virement : `(règlement,
    /// écriture du règlement)`.
    async fn regler_montant(&self, invoice: i64, bank: i64, montant: &str) -> (i64, i64) {
        let (s, b) = self
            .app
            .post(
                &format!("/api/v1/invoices/{invoice}/settlements"),
                json!({
                    "settlementType": "bank_transfer",
                    "bankAccountId": bank,
                    "amount": montant,
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
                counterparty_email: None,
                provider_type: None,
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

/// AC3 — annulation d'un règlement de facture client, la nuit du Nouvel An.
///
/// Story 63-1 (#1752, AC-11) : l'annulation depuis la fiche n'est plus datée du
/// jour, mais du règlement, dans son exercice (2025) — la nuit ne change plus
/// rien à sa date. Le test ne tue plus M7 de la 35-5a0 (ce geste ne lit plus le
/// jour) ; il fixe la date de l'origine.
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
    ctx.assert_contre_passee_a_l_origine(ecriture, "règlement client")
        .await;
}

/// AC3 — annulation d'une facture fournisseur validée, la nuit du Nouvel An.
///
/// Story 63-1 (#1754, AC-11) : l'inverse de l'achat est datée de l'achat, dans
/// 2025. Ne tue plus M7 de la 35-5a0.
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
    ctx.assert_contre_passee_a_l_origine(achat.unwrap(), "facture fournisseur")
        .await;
}

/// AC3 — annulation du règlement d'une facture fournisseur, la nuit du Nouvel
/// An.
///
/// Story 63-1 (#1753, AC-11) : l'inverse est datée du paiement, dans 2025. Ne
/// tue plus M7 de la 35-5a0.
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
    ctx.assert_contre_passee_a_l_origine(reglement.unwrap(), "règlement fournisseur")
        .await;
}

/// AC3 — annulation d'un rapprochement manuel. Tue M7.
///
/// Story 53-6a (#1674) : l'annulation d'un rapprochement n'est plus datée du
/// jour, mais de l'écriture du rapprochement, dans son exercice (2025) — la nuit
/// du Nouvel An ne change plus rien à sa date ; le test fixe désormais cela.
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
    let origine = ctx.date_et_exercice(ecriture).await;
    assert_eq!(origine.1, ctx.fy_2025, "montage : l'écriture est dans 2025");
    let inverse = ctx
        .inverse_de(ecriture)
        .await
        .expect("rapprochement : une contre-passation");
    assert_eq!(
        ctx.date_et_exercice(inverse).await,
        origine,
        "rapprochement : la contre-passation est datée de l'écriture, dans son exercice"
    );
}

/// AC3 — seul l'exercice 2025 existe ; aucun ne couvre le `2026-01-01`.
///
/// Story 63-1 (#1752, AC-9, AC-11) : l'ancien refus du rang 5
/// (`FISCAL_YEAR_INVALID`, « aucun exercice ouvert ne couvre la date du jour »)
/// ne s'annonce plus depuis la fiche — l'annulation est datée du règlement, dont
/// l'exercice 2025 est ouvert : la liste ne rend aucun motif, et l'annulation
/// **passe**, datée du règlement. Tue M1 (clic) et M4 (lecture).
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn ac3_sans_exercice_au_jour_suisse_l_annulation_passe(pool: MySqlPool) {
    let ctx = monter(&pool, NUIT_DU_NOUVEL_AN, None).await;
    let bank = ctx.compte_bancaire(IBAN_A, true, ctx.ledger).await;
    let facture = ctx.facture_validee().await;
    let (reglement, ecriture) = ctx.regler(facture, bank).await;

    let (s, b) = ctx
        .app
        .get(&format!("/api/v1/invoices/{facture}/settlements"))
        .await;
    assert_eq!(s, 200, "{b}");
    assert_eq!(b[0]["cancellable"], json!(true), "{b}");
    assert_eq!(b[0]["cancelBlockedBy"], json!(null), "{b}");

    let (s, b) = ctx
        .app
        .post_vide(&format!(
            "/api/v1/invoices/{facture}/settlements/{reglement}/cancel"
        ))
        .await;
    assert_eq!(s, 200, "{b}");
    ctx.assert_contre_passee_a_l_origine(ecriture, "règlement client")
        .await;
}

/// Story 63-1 (#1753, #1754, AC-9, AC-11) — sans exercice au jour suisse, ni le
/// paiement d'une facture **fournisseur** ni la facture elle-même ne
/// s'annoncent plus refusés au rang 5 : la vue ne rend aucun motif, les deux
/// annulations passent, datées de leur origine dans 2025. Tue M2, M3 (clics),
/// M5 et M6 (lectures).
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn ac3_sans_exercice_au_jour_suisse_l_annulation_d_un_paiement_fournisseur_passe(
    pool: MySqlPool,
) {
    let ctx = monter(&pool, NUIT_DU_NOUVEL_AN, None).await;
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
    let (achat, reglement) = ctx.ecritures_fournisseur(id).await;
    let (s, vue) = ctx
        .app
        .get(&format!("/api/v1/supplier-invoices/{id}"))
        .await;
    assert_eq!(s, 200, "{vue}");
    assert_eq!(vue["settlementCancelBlockedBy"], json!(null), "{vue}");
    assert_eq!(vue["cancelBlockedBy"], json!(null), "{vue}");

    let (s, b) = ctx
        .app
        .post(
            &format!("/api/v1/supplier-invoices/{id}/settlement/cancel"),
            json!({}),
        )
        .await;
    assert_eq!(s, 200, "{b}");
    ctx.assert_contre_passee_a_l_origine(reglement.unwrap(), "règlement fournisseur")
        .await;
    let (s, b) = ctx
        .app
        .post(&format!("/api/v1/supplier-invoices/{id}/cancel"), json!({}))
        .await;
    assert_eq!(s, 200, "{b}");
    ctx.assert_contre_passee_a_l_origine(achat.unwrap(), "facture fournisseur")
        .await;
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
/// 2026), à 00 h 30 le 1er janvier à Zurich.
///
/// Story 63-1 (#1752, AC-11) : la liste des règlements ne lit plus le jour —
/// l'annulation depuis la fiche est datée du règlement (décembre, exercice 2025
/// ouvert) : `cancellable: true`, aucun motif, là où le rang 5 répondait avant.
/// Ce service ne lit donc plus le jour suisse pour ce motif : la forme
/// « services » de M7 de la 35-5a0 n'y a plus de prise (cf. Dev Agent Record de
/// la 63-1).
///
/// Story 53-6a (#1674) : la candidate au rattachement d'une transaction
/// (`entry_cancel_outlook`) ne lit plus le jour — l'annulation d'un
/// rapprochement est datée de l'écriture (décembre, exercice 2025 ouvert) :
/// aucun motif (`null`), là où le rang 5 répondait avant.
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
                    foreign: None,
                },
                NewJournalEntryLine {
                    account_id: ctx.s.accounts["3000"],
                    debit: dec!(0),
                    credit: dec!(42.00),
                    project_id: None,
                    foreign: None,
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
    assert_eq!(b[0]["cancellable"], json!(true), "{b}");
    assert_eq!(b[0]["cancelBlockedBy"], json!(null), "{b}");

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
    assert_eq!(c["cancelBlockedBy"], json!(null), "{b}");
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
                    foreign: None,
                },
                NewJournalEntryLine {
                    account_id: ctx.s.accounts["1000"],
                    debit: dec!(0),
                    credit: dec!(100.00),
                    project_id: None,
                    foreign: None,
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
                    foreign: None,
                },
                NewJournalEntryLine {
                    account_id: ctx.s.accounts["1000"],
                    debit: dec!(0),
                    credit: dec!(100.00),
                    project_id: None,
                    foreign: None,
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

// ============================================================
// Story 53-6a (#1674) — l'annulation d'un rapprochement, datée de l'écriture
// ============================================================
//
// Le montage de ce fichier est celui du défaut : l'horloge au 1er janvier 2026
// (Zurich), l'écriture rapprochée en décembre 2025, dans un exercice 2025
// encore ouvert, et un exercice 2026 ouvert où la contre-passation tombait.

impl Ctx {
    /// Une écriture OD de 42.– sur le `1020` (la banque du montage), au `date`,
    /// dans 2025 — à rattacher à une transaction d'une AUTRE date (M10).
    async fn versement(&self, date: NaiveDate) -> i64 {
        journal_entries::create(
            &self.pool,
            self.fy_2025,
            kesh_db::entities::audit_log::Acteur::utilisateur(self.s.admin_user_id),
            NewJournalEntry {
                company_id: self.s.company_id,
                entry_date: date,
                journal: Journal::OD,
                description: "Versement".into(),
                project_id: None,
                lines: vec![
                    NewJournalEntryLine {
                        account_id: self.ledger,
                        debit: dec!(42.00),
                        credit: dec!(0),
                        project_id: None,
                        foreign: None,
                    },
                    NewJournalEntryLine {
                        account_id: self.s.accounts["3000"],
                        debit: dec!(0),
                        credit: dec!(42.00),
                        project_id: None,
                        foreign: None,
                    },
                ],
            },
        )
        .await
        .expect("versement")
        .entry
        .id
    }

    /// Pose la borne du verrou de période (SQL : `lock_books` refuse une borne
    /// postérieure au jour, ce qui n'est pas le sujet ici).
    async fn verrouiller(&self, through: NaiveDate) {
        sqlx::query("UPDATE companies SET books_locked_through = ? WHERE id = ?")
            .bind(through)
            .bind(self.s.company_id)
            .execute(&self.pool)
            .await
            .unwrap();
    }

    /// Le nombre d'écritures de l'exercice `fy`.
    async fn ecritures_de(&self, fy: i64) -> i64 {
        sqlx::query_scalar("SELECT COUNT(*) FROM journal_entries WHERE fiscal_year_id = ?")
            .bind(fy)
            .fetch_one(&self.pool)
            .await
            .unwrap()
    }

    /// Une transaction de 42.– (du 20 décembre) rattachée à un versement du
    /// 15 décembre : `(transaction, écriture)`.
    async fn versement_rattache(&self) -> (i64, i64) {
        let bank = self.compte_bancaire(IBAN_A, true, self.ledger).await;
        let entry = self.versement(jour(2025, 12, 15)).await;
        let t = self.transaction(bank, "42.00").await;
        let (s, b) = self
            .app
            .post(
                &format!("/api/v1/reconciliation/transactions/{t}/link-entry"),
                json!({ "journalEntryId": entry }),
            )
            .await;
        assert_eq!(s, 200, "rattachement : {b}");
        (t, entry)
    }

    /// Un règlement client du 20 décembre 2025 rattaché à une transaction du
    /// même montant : `(transaction, facture, règlement, écriture du règlement)`.
    /// `partiel` : 42.– (sans lettrage de pièce, D5) ; sinon le solde (108.10),
    /// qui solde la facture et la lettre.
    async fn reglement_client_rattache(&self, partiel: bool) -> (i64, i64, i64, i64) {
        let bank = self.compte_bancaire(IBAN_A, true, self.ledger).await;
        let facture = self.facture_validee().await;
        let montant = if partiel { "42.00" } else { "108.10" };
        let (sid, ecriture) = self.regler_montant(facture, bank, montant).await;
        let t = self.transaction(bank, montant).await;
        let (s, b) = self
            .app
            .post(
                &format!("/api/v1/reconciliation/transactions/{t}/link"),
                json!({ "settlementId": sid }),
            )
            .await;
        assert_eq!(s, 200, "rattachement au règlement : {b}");
        (t, facture, sid, ecriture)
    }

    /// `paid_at` d'une facture client.
    async fn soldee_le(&self, facture: i64) -> Option<chrono::NaiveDateTime> {
        sqlx::query_scalar("SELECT paid_at FROM invoices WHERE id = ?")
            .bind(facture)
            .fetch_one(&self.pool)
            .await
            .unwrap()
    }

    /// Le nombre de règlements d'une facture client.
    async fn reglements_de(&self, facture: i64) -> i64 {
        sqlx::query_scalar("SELECT COUNT(*) FROM invoice_settlements WHERE invoice_id = ?")
            .bind(facture)
            .fetch_one(&self.pool)
            .await
            .unwrap()
    }

    /// Clôture puis rouvre l'exercice `fy` par le dépôt (le geste de
    /// l'utilisateur), sans rien d'autre dans les livres.
    async fn cloturer_puis_rouvrir(&self, fy: i64) {
        let (c, u) = (self.s.company_id, self.s.admin_user_id);
        kesh_db::repositories::fiscal_years::close(&self.pool, u, c, fy)
            .await
            .expect("clôture");
        kesh_db::repositories::fiscal_years::reopen(&self.pool, u, c, fy, "correction".into())
            .await
            .expect("réouverture");
    }
}

/// AC-1 (R1, D1) — **le même exercice** : une écriture du 15 décembre 2025
/// rattachée à une transaction du 20, annulée le 22 décembre 2025 (le jour dans
/// le même exercice, donc différent de la date de l'écriture) : la vue et
/// l'inverse disent la date de l'**écriture**, ni celle de la transaction ni le
/// jour ; exercice 2025. Tue M1 et M10.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn s536a_ac1_une_ecriture_du_meme_exercice_est_contre_passee_a_sa_date(pool: MySqlPool) {
    let ctx = monter(&pool, "2025-12-22T10:00:00Z", Some("Open")).await;
    let (t, entry) = ctx.versement_rattache().await;
    let (_, vue) = ctx
        .app
        .get(&format!("/api/v1/reconciliation/transactions/{t}"))
        .await;
    assert_eq!(vue["reversalDate"], json!("2025-12-15"), "{vue}");
    let (s, b) = ctx
        .app
        .post_vide(&format!("/api/v1/reconciliation/transactions/{t}/cancel"))
        .await;
    assert_eq!(s, 200, "{b}");
    let inverse = ctx.inverse_de(entry).await.expect("une contre-passation");
    assert_eq!(
        ctx.date_et_exercice(inverse).await,
        (jour(2025, 12, 15), ctx.fy_2025),
        "datée de l'écriture (ni la transaction du 20, ni le jour du 22)"
    );
}

/// AC-3 (R1) — **l'exercice de l'écriture clôturé puis rouvert**, le jour dans
/// l'exercice suivant : l'inverse est datée de l'écriture, dans 2025 ; rien
/// dans 2026 ; puis 2025 se clôture de nouveau. Tue M1.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn s536a_ac3_une_ecriture_d_un_exercice_rouvert_est_contre_passee_dans_celui_ci(
    pool: MySqlPool,
) {
    let ctx = monter(&pool, NUIT_DU_NOUVEL_AN, Some("Open")).await;
    let fy_2026 = ctx.fy_2026.unwrap();
    let (t, entry) = ctx.versement_rattache().await;
    ctx.cloturer_puis_rouvrir(ctx.fy_2025).await;
    let dans_2026 = ctx.ecritures_de(fy_2026).await;

    let (s, b) = ctx
        .app
        .post_vide(&format!("/api/v1/reconciliation/transactions/{t}/cancel"))
        .await;
    assert_eq!(s, 200, "{b}");
    let inverse = ctx.inverse_de(entry).await.expect("une contre-passation");
    assert_eq!(
        ctx.date_et_exercice(inverse).await,
        (jour(2025, 12, 15), ctx.fy_2025)
    );
    assert_eq!(ctx.ecritures_de(fy_2026).await, dans_2026, "rien dans 2026");
    kesh_db::repositories::fiscal_years::close(
        &pool,
        ctx.s.admin_user_id,
        ctx.s.company_id,
        ctx.fy_2025,
    )
    .await
    .expect("2025 se clôture de nouveau");
}

/// AC-3, AC-5 (R2) — **le règlement client d'un exercice clôturé puis rouvert**,
/// le jour dans l'exercice suivant : l'inverse du règlement est datée du
/// 20 décembre, dans 2025 ; rien dans 2026 ; le règlement est retiré. Tue M2.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn s536a_ac3_un_reglement_client_d_un_exercice_rouvert_se_contre_passe_dans_celui_ci(
    pool: MySqlPool,
) {
    let ctx = monter(&pool, NUIT_DU_NOUVEL_AN, Some("Open")).await;
    let fy_2026 = ctx.fy_2026.unwrap();
    let (t, facture, _, reglement) = ctx.reglement_client_rattache(true).await;
    ctx.cloturer_puis_rouvrir(ctx.fy_2025).await;
    let dans_2026 = ctx.ecritures_de(fy_2026).await;

    let (s, b) = ctx
        .app
        .post_vide(&format!("/api/v1/reconciliation/transactions/{t}/cancel"))
        .await;
    assert_eq!(s, 200, "{b}");
    let inverse = ctx.inverse_de(reglement).await.expect("contre-passée");
    assert_eq!(
        ctx.date_et_exercice(inverse).await,
        (jour(2025, 12, 20), ctx.fy_2025)
    );
    assert_eq!(ctx.ecritures_de(fy_2026).await, dans_2026, "rien dans 2026");
    assert_eq!(
        ctx.reglements_de(facture).await,
        0,
        "le règlement est retiré"
    );
}

/// AC-1, AC-5 (R2, D1) — **le même exercice** : un règlement client du
/// 20 décembre 2025 rattaché à une transaction du même jour, annulé le
/// 22 décembre 2025 (le jour dans le même exercice, donc différent de la date
/// du règlement) : l'inverse du règlement est datée du 20 décembre, dans 2025,
/// non du jour. Tue M2 sur la date (l'exercice 2026 est ouvert : l'inverse datée
/// du jour y serait acceptée).
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn s536a_ac1_un_reglement_client_du_meme_exercice_se_contre_passe_a_sa_date(pool: MySqlPool) {
    let ctx = monter(&pool, "2025-12-22T10:00:00Z", Some("Open")).await;
    let (t, facture, _, reglement) = ctx.reglement_client_rattache(true).await;
    let (s, b) = ctx
        .app
        .post_vide(&format!("/api/v1/reconciliation/transactions/{t}/cancel"))
        .await;
    assert_eq!(s, 200, "{b}");
    let inverse = ctx.inverse_de(reglement).await.expect("contre-passée");
    assert_eq!(
        ctx.date_et_exercice(inverse).await,
        (jour(2025, 12, 20), ctx.fy_2025),
        "datée du règlement (ni le jour du 22)"
    );
    assert_eq!(
        ctx.reglements_de(facture).await,
        0,
        "le règlement est retiré"
    );
}

/// AC-4, AC-5 (R2) — **le règlement client d'un exercice clos** : la vue
/// l'annonce, le clic est refusé `409 FISCAL_YEAR_CLOSED`, rien n'est écrit, le
/// lien et le règlement restent.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn s536a_ac4_un_reglement_client_d_un_exercice_clos_refuse_l_annulation(pool: MySqlPool) {
    let ctx = monter(&pool, NUIT_DU_NOUVEL_AN, Some("Open")).await;
    let (t, facture, _, reglement) = ctx.reglement_client_rattache(true).await;
    sqlx::query("UPDATE fiscal_years SET status = 'Closed' WHERE id = ?")
        .bind(ctx.fy_2025)
        .execute(&pool)
        .await
        .unwrap();
    let (_, vue) = ctx
        .app
        .get(&format!("/api/v1/reconciliation/transactions/{t}"))
        .await;
    assert_eq!(vue["cancelBlockedBy"], json!("FISCAL_YEAR_CLOSED"), "{vue}");
    let avant = ctx.nombre_d_ecritures().await;
    let (s, b) = ctx
        .app
        .post_vide(&format!("/api/v1/reconciliation/transactions/{t}/cancel"))
        .await;
    assert_eq!(s, 409, "{b}");
    assert_eq!(b["error"]["code"], json!("FISCAL_YEAR_CLOSED"), "{b}");
    assert_eq!(ctx.nombre_d_ecritures().await, avant, "rien d'écrit");
    assert_eq!(ctx.inverse_de(reglement).await, None);
    assert_eq!(ctx.reglements_de(facture).await, 1, "le règlement reste");
    let lien: Option<i64> =
        sqlx::query_scalar("SELECT matched_entry_id FROM bank_transactions WHERE id = ?")
            .bind(t)
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(lien, Some(reglement), "le lien reste");
}

/// AC-1, AC-2, AC-10 (R1, D1, D4) — **une écriture de décembre 2025, exercice
/// encore ouvert, rapprochement annulé le 1er janvier 2026** : la vue annonce
/// la date et l'exercice de l'écriture ; l'inverse est datée du 15 décembre 2025
/// — la date de l'**écriture**, non celle de la transaction (20 décembre), ni le
/// jour —, dans 2025, sans suffixe ; rien n'est écrit dans 2026 ; la trace
/// `journal_entry.reversed` porte `"dating": "originDate"`. Tue M1 et M10.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn s536a_ac1_l_annulation_est_datee_de_l_ecriture_dans_son_exercice(pool: MySqlPool) {
    let ctx = monter(&pool, NUIT_DU_NOUVEL_AN, Some("Open")).await;
    let fy_2026 = ctx.fy_2026.unwrap();
    let (t, entry) = ctx.versement_rattache().await;

    let (s, vue) = ctx
        .app
        .get(&format!("/api/v1/reconciliation/transactions/{t}"))
        .await;
    assert_eq!(s, 200, "{vue}");
    assert_eq!(vue["cancellable"], json!(true), "{vue}");
    assert_eq!(vue["reversalDate"], json!("2025-12-15"), "{vue}");
    assert_eq!(
        vue["reversalFiscalYearName"],
        json!("Exercice 2025"),
        "{vue}"
    );

    let dans_2026 = ctx.ecritures_de(fy_2026).await;
    let (s, b) = ctx
        .app
        .post_vide(&format!("/api/v1/reconciliation/transactions/{t}/cancel"))
        .await;
    assert_eq!(s, 200, "{b}");
    let inverse = ctx.inverse_de(entry).await.expect("une contre-passation");
    assert_eq!(b["reversalJournalEntryId"], json!(inverse));
    assert_eq!(
        ctx.date_et_exercice(inverse).await,
        (jour(2025, 12, 15), ctx.fy_2025),
        "datée de l'écriture, dans 2025"
    );
    assert_eq!(ctx.ecritures_de(fy_2026).await, dans_2026, "rien dans 2026");
    let (libelle, numero): (String, i64) = sqlx::query_as(
        "SELECT i.description, o.entry_number FROM journal_entries i \
         JOIN journal_entries o ON o.id = i.reverses_entry_id WHERE i.id = ?",
    )
    .bind(inverse)
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(libelle, format!("Contre-passation écriture n° {numero}"));
    let details: Value = sqlx::query_scalar(
        "SELECT details_json FROM audit_log WHERE action = 'journal_entry.reversed' \
         AND entity_id = ? ORDER BY id DESC LIMIT 1",
    )
    .bind(entry)
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(
        details,
        json!({ "reversalJournalEntryId": inverse, "dating": "originDate" })
    );
}

/// AC-4 (R1, Q6) — **exercice de l'écriture clos** : la vue l'annonce, le clic
/// est refusé `409 FISCAL_YEAR_CLOSED` avec le texte du rapprochement, rien
/// n'est écrit, le lien reste. On ne rouvre pas d'office.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn s536a_ac4_un_exercice_clos_refuse_l_annulation(pool: MySqlPool) {
    let ctx = monter(&pool, NUIT_DU_NOUVEL_AN, Some("Open")).await;
    let (t, entry) = ctx.versement_rattache().await;
    sqlx::query("UPDATE fiscal_years SET status = 'Closed' WHERE id = ?")
        .bind(ctx.fy_2025)
        .execute(&pool)
        .await
        .unwrap();
    let (_, vue) = ctx
        .app
        .get(&format!("/api/v1/reconciliation/transactions/{t}"))
        .await;
    assert_eq!(vue["cancelBlockedBy"], json!("FISCAL_YEAR_CLOSED"), "{vue}");
    let avant = ctx.nombre_d_ecritures().await;
    let (s, b) = ctx
        .app
        .post_vide(&format!("/api/v1/reconciliation/transactions/{t}/cancel"))
        .await;
    assert_eq!(s, 409, "{b}");
    assert_eq!(b["error"]["code"], json!("FISCAL_YEAR_CLOSED"), "{b}");
    assert!(
        b["error"]["message"]
            .as_str()
            .is_some_and(|m| m.starts_with("Ce rapprochement appartient à un exercice clôturé")),
        "{b}"
    );
    assert_eq!(ctx.nombre_d_ecritures().await, avant, "rien d'écrit");
    assert_eq!(ctx.inverse_de(entry).await, None);
    let lien: Option<i64> =
        sqlx::query_scalar("SELECT matched_entry_id FROM bank_transactions WHERE id = ?")
            .bind(t)
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(lien, Some(entry), "le lien reste");
}

/// AC-9 (R1, D5, Q2) — **l'écriture sous la borne du verrou** (seuil inclusif,
/// au jour de la borne) : la vue rend `PERIOD_LOCKED` et la borne, le clic est
/// refusé par le socle en `400 PERIOD_LOCKED` (`details.lockedThrough`,
/// `details.attempted`), rien n'est écrit ; au **lendemain** de la borne,
/// l'annulation passe. Tue M6 (vue) et M11 (seuil de la vue).
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn s536a_ac9_une_ecriture_sous_le_verrou_est_refusee(pool: MySqlPool) {
    let ctx = monter(&pool, NUIT_DU_NOUVEL_AN, Some("Open")).await;
    let (t, entry) = ctx.versement_rattache().await;
    ctx.verrouiller(jour(2025, 12, 15)).await;

    let (_, vue) = ctx
        .app
        .get(&format!("/api/v1/reconciliation/transactions/{t}"))
        .await;
    assert_eq!(vue["cancellable"], json!(false), "{vue}");
    assert_eq!(vue["cancelBlockedBy"], json!("PERIOD_LOCKED"), "{vue}");
    assert_eq!(vue["cancelBlockedLabel"], json!("2025-12-15"), "{vue}");

    let avant = ctx.nombre_d_ecritures().await;
    let (s, b) = ctx
        .app
        .post_vide(&format!("/api/v1/reconciliation/transactions/{t}/cancel"))
        .await;
    assert_eq!(s, 400, "{b}");
    assert_eq!(b["error"]["code"], json!("PERIOD_LOCKED"), "{b}");
    assert_eq!(
        b["error"]["details"]["lockedThrough"],
        json!("2025-12-15"),
        "{b}"
    );
    assert_eq!(
        b["error"]["details"]["attempted"],
        json!("2025-12-15"),
        "{b}"
    );
    assert_eq!(ctx.nombre_d_ecritures().await, avant, "rien d'écrit");
    assert_eq!(ctx.inverse_de(entry).await, None);
    let lien: Option<i64> =
        sqlx::query_scalar("SELECT matched_entry_id FROM bank_transactions WHERE id = ?")
            .bind(t)
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(lien, Some(entry), "le lien reste");

    ctx.verrouiller(jour(2025, 12, 14)).await;
    let (_, vue) = ctx
        .app
        .get(&format!("/api/v1/reconciliation/transactions/{t}"))
        .await;
    assert_eq!(
        vue["cancellable"],
        json!(true),
        "au lendemain de la borne : {vue}"
    );
    let (s, b) = ctx
        .app
        .post_vide(&format!("/api/v1/reconciliation/transactions/{t}/cancel"))
        .await;
    assert_eq!(s, 200, "{b}");
}

/// AC-9 (candidates, L9) — **une écriture candidate sous la borne** : la liste
/// des écritures à rattacher l'annonce comme obstacle (`PERIOD_LOCKED`, la
/// borne en étiquette), par la même queue que la vue. Tue M6 (candidates).
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn s536a_ac9_une_candidate_sous_le_verrou_l_annonce(pool: MySqlPool) {
    let ctx = monter(&pool, NUIT_DU_NOUVEL_AN, Some("Open")).await;
    let bank = ctx.compte_bancaire(IBAN_A, true, ctx.ledger).await;
    let entry = ctx.versement(jour(2025, 12, 15)).await;
    let t = ctx.transaction(bank, "42.00").await;
    ctx.verrouiller(jour(2025, 12, 15)).await;
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
        .find(|c| c["journalEntryId"] == json!(entry))
        .unwrap_or_else(|| panic!("candidate absente : {b}"))
        .clone();
    assert_eq!(c["cancelBlockedBy"], json!("PERIOD_LOCKED"), "{b}");
    assert_eq!(c["cancelBlockedLabel"], json!("2025-12-15"), "{b}");
}

/// AC-5, AC-9 (R2) — **un règlement client partiel rattaché**, l'écriture de
/// règlement du 20 décembre 2025 : le jour de la borne, `400 PERIOD_LOCKED` du
/// socle (le règlement partiel n'a pas de lettrage de pièce, D5) ; au
/// lendemain, l'annulation passe, datée du règlement dans 2025, et le règlement
/// est retiré. Tue M2.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn s536a_ac5_un_reglement_client_rattache_se_contre_passe_a_sa_date(pool: MySqlPool) {
    let ctx = monter(&pool, NUIT_DU_NOUVEL_AN, Some("Open")).await;
    let (t, facture, _, reglement) = ctx.reglement_client_rattache(true).await;

    ctx.verrouiller(jour(2025, 12, 20)).await;
    let (_, vue) = ctx
        .app
        .get(&format!("/api/v1/reconciliation/transactions/{t}"))
        .await;
    assert_eq!(vue["kind"], json!("invoice_settlement"), "{vue}");
    assert_eq!(vue["cancelBlockedBy"], json!("PERIOD_LOCKED"), "{vue}");
    assert_eq!(vue["cancelBlockedLabel"], json!("2025-12-20"), "{vue}");
    let avant = ctx.nombre_d_ecritures().await;
    let (s, b) = ctx
        .app
        .post_vide(&format!("/api/v1/reconciliation/transactions/{t}/cancel"))
        .await;
    assert_eq!(s, 400, "{b}");
    assert_eq!(b["error"]["code"], json!("PERIOD_LOCKED"), "{b}");
    assert_eq!(
        b["error"]["details"]["lockedThrough"],
        json!("2025-12-20"),
        "{b}"
    );
    assert_eq!(
        b["error"]["details"]["attempted"],
        json!("2025-12-20"),
        "{b}"
    );
    // Rien n'est écrit, le lien et le règlement restent.
    assert_eq!(ctx.nombre_d_ecritures().await, avant, "rien d'écrit");
    assert_eq!(ctx.inverse_de(reglement).await, None);
    assert_eq!(ctx.reglements_de(facture).await, 1, "le règlement reste");
    let lien: Option<i64> =
        sqlx::query_scalar("SELECT matched_entry_id FROM bank_transactions WHERE id = ?")
            .bind(t)
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(lien, Some(reglement), "le lien reste");

    ctx.verrouiller(jour(2025, 12, 19)).await;
    let (s, b) = ctx
        .app
        .post_vide(&format!("/api/v1/reconciliation/transactions/{t}/cancel"))
        .await;
    assert_eq!(s, 200, "{b}");
    let inverse = ctx.inverse_de(reglement).await.expect("contre-passée");
    assert_eq!(
        ctx.date_et_exercice(inverse).await,
        (jour(2025, 12, 20), ctx.fy_2025)
    );
    assert_eq!(
        ctx.reglements_de(facture).await,
        0,
        "le règlement est retiré"
    );
}

/// AC-5 (R2) — **un règlement client qui solde la facture**, rattaché, annulé
/// le 1er janvier 2026 : l'inverse est datée du 20 décembre 2025, dans 2025 ;
/// rien dans 2026 ; le règlement est retiré et la facture **redevient à régler**
/// (`paid_at` revient à `NULL`) ; le lettrage `reversal` du règlement et de son
/// miroir est dans 2025. Tue M2.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn s536a_ac5_un_reglement_qui_solde_la_facture_la_rend_a_regler(pool: MySqlPool) {
    let ctx = monter(&pool, NUIT_DU_NOUVEL_AN, Some("Open")).await;
    let fy_2026 = ctx.fy_2026.unwrap();
    let (t, facture, _, reglement) = ctx.reglement_client_rattache(false).await;
    assert!(ctx.soldee_le(facture).await.is_some(), "facture soldée");
    let dans_2026 = ctx.ecritures_de(fy_2026).await;

    let (s, b) = ctx
        .app
        .post_vide(&format!("/api/v1/reconciliation/transactions/{t}/cancel"))
        .await;
    assert_eq!(s, 200, "{b}");
    let inverse = ctx.inverse_de(reglement).await.expect("contre-passée");
    assert_eq!(
        ctx.date_et_exercice(inverse).await,
        (jour(2025, 12, 20), ctx.fy_2025)
    );
    assert_eq!(ctx.ecritures_de(fy_2026).await, dans_2026, "rien dans 2026");
    assert_eq!(
        ctx.reglements_de(facture).await,
        0,
        "le règlement est retiré"
    );
    assert_eq!(ctx.soldee_le(facture).await, None, "facture à régler");

    // Le lettrage `reversal` : le règlement et son miroir, dans 2025.
    let lettrees: Vec<(i64, i64, Option<String>)> = sqlx::query_as(
        "SELECT l.entry_id, l.lettering_key, l.lettering_origin FROM journal_entry_lines l \
         WHERE l.entry_id IN (?, ?) AND l.lettering_key IS NOT NULL ORDER BY l.entry_id",
    )
    .bind(reglement)
    .bind(inverse)
    .fetch_all(&pool)
    .await
    .unwrap();
    assert_eq!(lettrees.len(), 2, "{lettrees:?}");
    assert_eq!(lettrees[0].1, lettrees[1].1, "une seule clé de lettrage");
    for (_, _, origine) in &lettrees {
        assert_eq!(origine.as_deref(), Some("reversal"));
    }
    assert_eq!(
        ctx.date_et_exercice(lettrees[0].0).await.1,
        ctx.fy_2025,
        "le lettrage est dans 2025"
    );
}

// ============================================================
// Story 63-1 (#1752, #1753, #1754) — les annulations depuis la fiche, datées
// de l'écriture d'origine, dans son exercice
// ============================================================
//
// Même montage que la 53-6a : l'horloge au 1er janvier 2026 (Zurich), les
// pièces en décembre 2025 dans un exercice 2025 encore ouvert, un exercice
// 2026 ouvert où la contre-passation datée du jour tombait. Le jour diffère
// toujours de la date de l'origine : un test de la date de l'origine ne passe
// pas à vide sous `Today`.

impl Ctx {
    /// Paie la facture fournisseur `id` le `date` par virement sur `bank`.
    async fn payer_fournisseur(&self, id: i64, bank: i64, date: &str) {
        let (s, b) = self
            .app
            .post(
                &format!("/api/v1/supplier-invoices/{id}/pay"),
                json!({
                    "settlementType": "bank_transfer",
                    "bankAccountId": bank,
                    "paymentDate": date
                }),
            )
            .await;
        assert_eq!(s, 200, "paiement : {b}");
    }

    /// Le détail de `journal_entry.reversed` le plus récent pour `entry_id`.
    async fn audit_contre_passation(&self, entry_id: i64) -> Value {
        sqlx::query_scalar(
            "SELECT details_json FROM audit_log WHERE action = 'journal_entry.reversed' \
             AND entity_id = ? ORDER BY id DESC LIMIT 1",
        )
        .bind(entry_id)
        .fetch_one(&self.pool)
        .await
        .unwrap()
    }

    /// Les lignes lettrées de `entry_id` : `(clé, origine du lettrage)`.
    async fn lettrage_de(&self, entry_id: i64) -> Vec<(i64, Option<String>)> {
        sqlx::query_as(
            "SELECT l.lettering_key, l.lettering_origin FROM journal_entry_lines l \
             WHERE l.entry_id = ? AND l.lettering_key IS NOT NULL ORDER BY l.id",
        )
        .bind(entry_id)
        .fetch_all(&self.pool)
        .await
        .unwrap()
    }

    /// Le refus `400 PERIOD_LOCKED` du socle, borne et date tentée.
    fn assert_periode_verrouillee(s: u16, b: &Value, borne: &str) {
        assert_eq!(s, 400, "{b}");
        assert_eq!(b["error"]["code"], json!("PERIOD_LOCKED"), "{b}");
        assert_eq!(b["error"]["details"]["lockedThrough"], json!(borne), "{b}");
        assert_eq!(b["error"]["details"]["attempted"], json!(borne), "{b}");
    }
}

/// AC-1 (#1752, R7) — **règlement client d'un exercice antérieur encore
/// ouvert**, annulé depuis la fiche le 1er janvier 2026 : l'inverse est datée du
/// règlement (20 décembre 2025), dans 2025 ; aucune écriture dans 2026 ; le
/// règlement est retiré, la facture redevient à régler ; le lettrage `reversal`
/// est dans 2025 ; `journal_entry.reversed` porte `"dating": "originDate"`.
/// Tue M1.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn s631_ac1_un_reglement_client_se_contre_passe_dans_son_exercice(pool: MySqlPool) {
    let ctx = monter(&pool, NUIT_DU_NOUVEL_AN, Some("Open")).await;
    let fy_2026 = ctx.fy_2026.unwrap();
    let bank = ctx.compte_bancaire(IBAN_A, true, ctx.ledger).await;
    let facture = ctx.facture_validee().await;
    let (reglement, ecriture) = ctx.regler(facture, bank).await;
    assert!(ctx.soldee_le(facture).await.is_some(), "facture soldée");
    let dans_2026 = ctx.ecritures_de(fy_2026).await;

    let (s, b) = ctx
        .app
        .post_vide(&format!(
            "/api/v1/invoices/{facture}/settlements/{reglement}/cancel"
        ))
        .await;
    assert_eq!(s, 200, "{b}");
    let inverse = ctx.inverse_de(ecriture).await.expect("contre-passée");
    assert_eq!(b["reversalJournalEntryId"], json!(inverse), "{b}");
    assert_eq!(
        ctx.date_et_exercice(inverse).await,
        (jour(2025, 12, 20), ctx.fy_2025),
        "datée du règlement, dans 2025 (non du jour, dans 2026)"
    );
    assert_eq!(ctx.ecritures_de(fy_2026).await, dans_2026, "rien dans 2026");
    assert_eq!(
        ctx.reglements_de(facture).await,
        0,
        "le règlement est retiré"
    );
    assert_eq!(ctx.soldee_le(facture).await, None, "facture à régler");
    let lettrage = ctx.lettrage_de(inverse).await;
    assert_eq!(lettrage.len(), 1, "{lettrage:?}");
    assert_eq!(lettrage[0].1.as_deref(), Some("reversal"));
    assert_eq!(
        ctx.audit_contre_passation(ecriture).await,
        json!({ "reversalJournalEntryId": inverse, "dating": "originDate" })
    );
}

/// AC-2 (#1752, Q1) — **le même exercice** : le règlement du 20 décembre 2025
/// annulé le 22 décembre 2025 : l'inverse est datée du 20, non du 22. Tue M1
/// sur la date (sous `Today`, l'exercice 2025 couvre le 22 : seule la date
/// diffère).
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn s631_ac2_un_reglement_client_du_meme_exercice_se_contre_passe_a_sa_date(pool: MySqlPool) {
    let ctx = monter(&pool, "2025-12-22T10:00:00Z", Some("Open")).await;
    let bank = ctx.compte_bancaire(IBAN_A, true, ctx.ledger).await;
    let facture = ctx.facture_validee().await;
    let (reglement, ecriture) = ctx.regler_montant(facture, bank, "42.00").await;
    let (s, b) = ctx
        .app
        .post_vide(&format!(
            "/api/v1/invoices/{facture}/settlements/{reglement}/cancel"
        ))
        .await;
    assert_eq!(s, 200, "{b}");
    let inverse = ctx.inverse_de(ecriture).await.expect("contre-passée");
    assert_eq!(
        ctx.date_et_exercice(inverse).await,
        (jour(2025, 12, 20), ctx.fy_2025),
        "datée du règlement (ni le jour du 22)"
    );
}

/// AC-4 (#1753, R8) — **le paiement d'une facture fournisseur** du 20 décembre
/// 2025, annulé depuis la fiche le 1er janvier 2026 : l'inverse est datée du
/// paiement, dans 2025 ; rien dans 2026 ; la facture redevient `open`, ses
/// colonnes de règlement à `null` ; audit `"dating": "originDate"`. Tue M2.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn s631_ac4_un_paiement_fournisseur_se_contre_passe_dans_son_exercice(pool: MySqlPool) {
    let ctx = monter(&pool, NUIT_DU_NOUVEL_AN, Some("Open")).await;
    let fy_2026 = ctx.fy_2026.unwrap();
    let bank = ctx.compte_bancaire(IBAN_A, true, ctx.ledger).await;
    let id = ctx.facture_fournisseur().await;
    ctx.payer_fournisseur(id, bank, "2025-12-20").await;
    let (_, reglement) = ctx.ecritures_fournisseur(id).await;
    let reglement = reglement.expect("écriture de règlement");
    let dans_2026 = ctx.ecritures_de(fy_2026).await;

    let (s, b) = ctx
        .app
        .post(
            &format!("/api/v1/supplier-invoices/{id}/settlement/cancel"),
            json!({}),
        )
        .await;
    assert_eq!(s, 200, "{b}");
    let inverse = ctx.inverse_de(reglement).await.expect("contre-passée");
    assert_eq!(
        ctx.date_et_exercice(inverse).await,
        (jour(2025, 12, 20), ctx.fy_2025),
        "datée du paiement, dans 2025"
    );
    assert_eq!(ctx.ecritures_de(fy_2026).await, dans_2026, "rien dans 2026");
    assert_eq!(b["reversalJournalEntryId"], json!(inverse), "{b}");
    assert_eq!(b["invoice"]["status"], json!("open"), "{b}");
    assert_eq!(b["invoice"]["settlementJournalEntryId"], json!(null), "{b}");
    assert_eq!(b["invoice"]["paidAt"], json!(null), "{b}");
    assert_eq!(
        ctx.audit_contre_passation(reglement).await,
        json!({ "reversalJournalEntryId": inverse, "dating": "originDate" })
    );
}

/// AC-5 (#1754, R9) — **une facture fournisseur validée**, annulée le
/// 1er janvier 2026 : l'inverse de l'achat est datée de l'achat (15 décembre
/// 2025), dans 2025 ; la charge nette à zéro dans 2025 ; rien dans 2026 ; la
/// facture est `cancelled`. Tue M3.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn s631_ac5_une_facture_fournisseur_validee_se_contre_passe_a_la_date_de_l_achat(
    pool: MySqlPool,
) {
    let ctx = monter(&pool, NUIT_DU_NOUVEL_AN, Some("Open")).await;
    let fy_2026 = ctx.fy_2026.unwrap();
    let id = ctx.facture_fournisseur().await;
    let (achat, _) = ctx.ecritures_fournisseur(id).await;
    let achat = achat.expect("écriture d'achat");
    let dans_2026 = ctx.ecritures_de(fy_2026).await;

    let (s, b) = ctx
        .app
        .post(&format!("/api/v1/supplier-invoices/{id}/cancel"), json!({}))
        .await;
    assert_eq!(s, 200, "{b}");
    assert_eq!(b["status"], json!("cancelled"), "{b}");
    let inverse = ctx.inverse_de(achat).await.expect("contre-passée");
    assert_eq!(
        ctx.date_et_exercice(inverse).await,
        (jour(2025, 12, 15), ctx.fy_2025),
        "datée de l'achat, dans 2025"
    );
    assert_eq!(ctx.ecritures_de(fy_2026).await, dans_2026, "rien dans 2026");
    let charge_2025: rust_decimal::Decimal = sqlx::query_scalar(
        "SELECT COALESCE(SUM(l.debit - l.credit), 0) FROM journal_entry_lines l \
         JOIN journal_entries e ON e.id = l.entry_id \
         WHERE l.account_id = ? AND e.fiscal_year_id = ?",
    )
    .bind(ctx.s.accounts["4000"])
    .bind(ctx.fy_2025)
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(charge_2025, dec!(0), "la charge nette à zéro dans 2025");
    assert_eq!(
        ctx.audit_contre_passation(achat).await,
        json!({ "reversalJournalEntryId": inverse, "dating": "originDate" })
    );
}

/// AC-5 (#1754, P7) — **une facture fournisseur payée**, annulée le
/// 1er janvier 2026 : l'inverse de l'achat est datée de l'achat, dans 2025 ; le
/// règlement reste au grand livre, à sa date, **détaché**
/// (`settlementJournalEntryId` à `null`), nommé par l'audit du geste. Tue M3.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn s631_ac5_une_facture_fournisseur_payee_se_contre_passe_a_la_date_de_l_achat(
    pool: MySqlPool,
) {
    let ctx = monter(&pool, NUIT_DU_NOUVEL_AN, Some("Open")).await;
    let bank = ctx.compte_bancaire(IBAN_A, true, ctx.ledger).await;
    let id = ctx.facture_fournisseur().await;
    ctx.payer_fournisseur(id, bank, "2025-12-20").await;
    let (achat, reglement) = ctx.ecritures_fournisseur(id).await;
    let (achat, reglement) = (achat.unwrap(), reglement.unwrap());

    let (s, b) = ctx
        .app
        .post(&format!("/api/v1/supplier-invoices/{id}/cancel"), json!({}))
        .await;
    assert_eq!(s, 200, "{b}");
    assert_eq!(b["status"], json!("cancelled"), "{b}");
    assert_eq!(b["settlementJournalEntryId"], json!(null), "{b}");
    let inverse = ctx.inverse_de(achat).await.expect("contre-passée");
    assert_eq!(
        ctx.date_et_exercice(inverse).await,
        (jour(2025, 12, 15), ctx.fy_2025),
        "datée de l'achat, dans 2025"
    );
    assert_eq!(
        ctx.inverse_de(reglement).await,
        None,
        "règlement non contre-passé"
    );
    assert_eq!(
        ctx.date_et_exercice(reglement).await,
        (jour(2025, 12, 20), ctx.fy_2025),
        "le règlement reste, à sa date"
    );
    let details: Value = sqlx::query_scalar(
        "SELECT details_json FROM audit_log WHERE action = 'supplier_invoice.cancelled' \
         AND entity_id = ? ORDER BY id DESC LIMIT 1",
    )
    .bind(id)
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(
        details["settlementJournalEntryId"],
        json!(reglement),
        "{details}"
    );
    assert_eq!(
        details["reversalJournalEntryId"],
        json!(inverse),
        "{details}"
    );
}

/// AC-6 (#1754, Q6) — **l'exercice de l'achat clôturé puis rouvert** :
/// l'annulation se date dans 2025 rouvert ; rien dans 2026 ; 2025 se reclôture
/// ensuite sans obstacle nouveau. Tue M3.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn s631_ac6_une_facture_fournisseur_d_un_exercice_rouvert_s_y_contre_passe(pool: MySqlPool) {
    let ctx = monter(&pool, NUIT_DU_NOUVEL_AN, Some("Open")).await;
    let fy_2026 = ctx.fy_2026.unwrap();
    let id = ctx.facture_fournisseur().await;
    let (achat, _) = ctx.ecritures_fournisseur(id).await;
    ctx.cloturer_puis_rouvrir(ctx.fy_2025).await;
    let dans_2026 = ctx.ecritures_de(fy_2026).await;

    let (s, b) = ctx
        .app
        .post(&format!("/api/v1/supplier-invoices/{id}/cancel"), json!({}))
        .await;
    assert_eq!(s, 200, "{b}");
    let inverse = ctx.inverse_de(achat.unwrap()).await.expect("contre-passée");
    assert_eq!(
        ctx.date_et_exercice(inverse).await,
        (jour(2025, 12, 15), ctx.fy_2025)
    );
    assert_eq!(ctx.ecritures_de(fy_2026).await, dans_2026, "rien dans 2026");
    kesh_db::repositories::fiscal_years::close(
        &pool,
        ctx.s.admin_user_id,
        ctx.s.company_id,
        ctx.fy_2025,
    )
    .await
    .expect("2025 se clôture de nouveau");
}

/// AC-7 (Q6, P5) — **l'exercice de l'origine clos**, pour chacun des trois
/// gestes : la vue l'annonce (`FISCAL_YEAR_CLOSED`), le clic est refusé
/// `409 FISCAL_YEAR_CLOSED`, rien n'est écrit. Inchangé par la story (rang 2).
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn s631_ac7_un_exercice_d_origine_clos_refuse_les_trois_gestes(pool: MySqlPool) {
    let ctx = monter(&pool, NUIT_DU_NOUVEL_AN, Some("Open")).await;
    let bank = ctx.compte_bancaire(IBAN_A, true, ctx.ledger).await;
    let facture = ctx.facture_validee().await;
    let (reglement, _) = ctx.regler_montant(facture, bank, "42.00").await;
    let payee = ctx.facture_fournisseur().await;
    ctx.payer_fournisseur(payee, bank, "2025-12-20").await;
    sqlx::query("UPDATE fiscal_years SET status = 'Closed' WHERE id = ?")
        .bind(ctx.fy_2025)
        .execute(&pool)
        .await
        .unwrap();

    let (_, b) = ctx
        .app
        .get(&format!("/api/v1/invoices/{facture}/settlements"))
        .await;
    assert_eq!(b[0]["cancelBlockedBy"], json!("FISCAL_YEAR_CLOSED"), "{b}");
    let (_, vue) = ctx
        .app
        .get(&format!("/api/v1/supplier-invoices/{payee}"))
        .await;
    assert_eq!(
        vue["settlementCancelBlockedBy"],
        json!("FISCAL_YEAR_CLOSED"),
        "{vue}"
    );
    assert_eq!(vue["cancelBlockedBy"], json!("FISCAL_YEAR_CLOSED"), "{vue}");

    let avant = ctx.nombre_d_ecritures().await;
    for (chemin, corps) in [
        (
            format!("/api/v1/invoices/{facture}/settlements/{reglement}/cancel"),
            None,
        ),
        (
            format!("/api/v1/supplier-invoices/{payee}/settlement/cancel"),
            Some(json!({})),
        ),
        (
            format!("/api/v1/supplier-invoices/{payee}/cancel"),
            Some(json!({})),
        ),
    ] {
        let (s, b) = match corps {
            None => ctx.app.post_vide(&chemin).await,
            Some(c) => ctx.app.post(&chemin, c).await,
        };
        assert_eq!(
            (s, b["error"]["code"].clone()),
            (409, json!("FISCAL_YEAR_CLOSED")),
            "{chemin} : {b}"
        );
    }
    assert_eq!(ctx.nombre_d_ecritures().await, avant, "rien n'est écrit");
}

/// AC-8 (a, c, d) (#1752) — **un règlement client partiel sous la borne**
/// (seuil inclusif, au jour de la borne) : la liste des règlements rend
/// `cancellable: false`, `PERIOD_LOCKED` et la borne ; le clic forcé est refusé
/// par le socle en `400 PERIOD_LOCKED`, rien n'est écrit ; au **lendemain** de
/// la borne, la liste l'annonce possible et l'annulation passe, datée du
/// règlement. Tue M1 (clic) et M4 (lecture).
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn s631_ac8_un_reglement_client_sous_le_verrou_est_annonce_et_refuse(pool: MySqlPool) {
    let ctx = monter(&pool, NUIT_DU_NOUVEL_AN, Some("Open")).await;
    let bank = ctx.compte_bancaire(IBAN_A, true, ctx.ledger).await;
    let facture = ctx.facture_validee().await;
    let (reglement, ecriture) = ctx.regler_montant(facture, bank, "42.00").await;
    ctx.verrouiller(jour(2025, 12, 20)).await;

    let (s, b) = ctx
        .app
        .get(&format!("/api/v1/invoices/{facture}/settlements"))
        .await;
    assert_eq!(s, 200, "{b}");
    assert_eq!(b[0]["cancellable"], json!(false), "{b}");
    assert_eq!(b[0]["cancelBlockedBy"], json!("PERIOD_LOCKED"), "{b}");
    assert_eq!(b[0]["cancelBlockedLabel"], json!("2025-12-20"), "{b}");

    let avant = ctx.nombre_d_ecritures().await;
    let (s, b) = ctx
        .app
        .post_vide(&format!(
            "/api/v1/invoices/{facture}/settlements/{reglement}/cancel"
        ))
        .await;
    Ctx::assert_periode_verrouillee(s, &b, "2025-12-20");
    assert_eq!(ctx.nombre_d_ecritures().await, avant, "rien d'écrit");
    assert_eq!(ctx.inverse_de(ecriture).await, None);
    assert_eq!(ctx.reglements_de(facture).await, 1, "le règlement reste");

    ctx.verrouiller(jour(2025, 12, 19)).await;
    let (_, b) = ctx
        .app
        .get(&format!("/api/v1/invoices/{facture}/settlements"))
        .await;
    assert_eq!(b[0]["cancellable"], json!(true), "au lendemain : {b}");
    assert_eq!(b[0]["cancelBlockedBy"], json!(null), "{b}");
    let (s, b) = ctx
        .app
        .post_vide(&format!(
            "/api/v1/invoices/{facture}/settlements/{reglement}/cancel"
        ))
        .await;
    assert_eq!(s, 200, "{b}");
    let inverse = ctx.inverse_de(ecriture).await.expect("contre-passée");
    assert_eq!(
        ctx.date_et_exercice(inverse).await,
        (jour(2025, 12, 20), ctx.fy_2025)
    );
}

/// AC-8 (b, c, d) (#1753, D4) — **le paiement fournisseur sous la borne,
/// l'achat au-dessus** : la facture validée au 15 décembre, payée au 10 (`pay`
/// l'accepte), **puis** le verrou posé jusqu'au 10 — le groupe {achat,
/// paiement} n'est pas figé (l'achat est en période ouverte), le rang 2 bis ne
/// répond pas. La vue rend `settlementCancelBlockedBy: "PERIOD_LOCKED"` et la
/// borne ; le clic forcé est refusé `400 PERIOD_LOCKED` ; au lendemain, il
/// passe, daté du paiement. Tue M2 (clic) et M5 (lecture).
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn s631_ac8_un_paiement_fournisseur_sous_le_verrou_est_annonce_et_refuse(pool: MySqlPool) {
    let ctx = monter(&pool, NUIT_DU_NOUVEL_AN, Some("Open")).await;
    let bank = ctx.compte_bancaire(IBAN_A, true, ctx.ledger).await;
    let id = ctx.facture_fournisseur().await;
    ctx.payer_fournisseur(id, bank, "2025-12-10").await;
    let (_, reglement) = ctx.ecritures_fournisseur(id).await;
    let reglement = reglement.unwrap();
    ctx.verrouiller(jour(2025, 12, 10)).await;

    let (s, vue) = ctx
        .app
        .get(&format!("/api/v1/supplier-invoices/{id}"))
        .await;
    assert_eq!(s, 200, "{vue}");
    assert_eq!(vue["settlementCancellable"], json!(false), "{vue}");
    assert_eq!(
        vue["settlementCancelBlockedBy"],
        json!("PERIOD_LOCKED"),
        "{vue}"
    );
    assert_eq!(
        vue["settlementCancelBlockedLabel"],
        json!("2025-12-10"),
        "{vue}"
    );

    let avant = ctx.nombre_d_ecritures().await;
    let (s, b) = ctx
        .app
        .post(
            &format!("/api/v1/supplier-invoices/{id}/settlement/cancel"),
            json!({}),
        )
        .await;
    Ctx::assert_periode_verrouillee(s, &b, "2025-12-10");
    assert_eq!(ctx.nombre_d_ecritures().await, avant, "rien d'écrit");
    assert_eq!(ctx.inverse_de(reglement).await, None);

    ctx.verrouiller(jour(2025, 12, 9)).await;
    let (_, vue) = ctx
        .app
        .get(&format!("/api/v1/supplier-invoices/{id}"))
        .await;
    assert_eq!(
        vue["settlementCancellable"],
        json!(true),
        "au lendemain : {vue}"
    );
    let (s, b) = ctx
        .app
        .post(
            &format!("/api/v1/supplier-invoices/{id}/settlement/cancel"),
            json!({}),
        )
        .await;
    assert_eq!(s, 200, "{b}");
    let inverse = ctx.inverse_de(reglement).await.expect("contre-passée");
    assert_eq!(
        ctx.date_et_exercice(inverse).await,
        (jour(2025, 12, 10), ctx.fy_2025)
    );
}

/// AC-8 (b, c, d) (#1754, D4, Q-A) — **une facture fournisseur validée, non
/// payée, dont l'achat est sous la borne** (au jour de la borne) : la vue rend
/// `cancelBlockedBy: "PERIOD_LOCKED"` et la borne ; le clic forcé est refusé
/// `400 PERIOD_LOCKED`, rien n'est écrit, la facture reste validée ; au
/// lendemain de la borne, l'annulation passe, datée de l'achat. Tue M3 (clic)
/// et M6 (lecture).
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn s631_ac8_une_facture_fournisseur_sous_le_verrou_est_annoncee_et_refusee(pool: MySqlPool) {
    let ctx = monter(&pool, NUIT_DU_NOUVEL_AN, Some("Open")).await;
    let id = ctx.facture_fournisseur().await;
    let (achat, _) = ctx.ecritures_fournisseur(id).await;
    let achat = achat.unwrap();
    ctx.verrouiller(jour(2025, 12, 15)).await;

    let (s, vue) = ctx
        .app
        .get(&format!("/api/v1/supplier-invoices/{id}"))
        .await;
    assert_eq!(s, 200, "{vue}");
    assert_eq!(vue["cancellable"], json!(false), "{vue}");
    assert_eq!(vue["cancelBlockedBy"], json!("PERIOD_LOCKED"), "{vue}");
    assert_eq!(vue["cancelBlockedLabel"], json!("2025-12-15"), "{vue}");

    let avant = ctx.nombre_d_ecritures().await;
    let (s, b) = ctx
        .app
        .post(&format!("/api/v1/supplier-invoices/{id}/cancel"), json!({}))
        .await;
    Ctx::assert_periode_verrouillee(s, &b, "2025-12-15");
    assert_eq!(ctx.nombre_d_ecritures().await, avant, "rien d'écrit");
    assert_eq!(ctx.inverse_de(achat).await, None);
    let statut: String = sqlx::query_scalar("SELECT status FROM supplier_invoices WHERE id = ?")
        .bind(id)
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(statut, "open", "la facture reste validée");

    ctx.verrouiller(jour(2025, 12, 14)).await;
    let (_, vue) = ctx
        .app
        .get(&format!("/api/v1/supplier-invoices/{id}"))
        .await;
    assert_eq!(vue["cancellable"], json!(true), "au lendemain : {vue}");
    let (s, b) = ctx
        .app
        .post(&format!("/api/v1/supplier-invoices/{id}/cancel"), json!({}))
        .await;
    assert_eq!(s, 200, "{b}");
    let inverse = ctx.inverse_de(achat).await.expect("contre-passée");
    assert_eq!(
        ctx.date_et_exercice(inverse).await,
        (jour(2025, 12, 15), ctx.fy_2025)
    );
}

/// AC-3 (#1752, D6) — **un escompte** (solde `write_off` avec correction de
/// TVA) du 20 décembre 2025, annulé depuis la fiche le 1er janvier 2026 :
/// l'inverse, **ligne de TVA comprise**, est datée du solde, dans 2025 ; rien
/// dans 2026 ; dans 2025, les comptes du solde (nature, TVA due, débiteurs)
/// nettent à zéro entre le solde et son inverse. Tue M1 (pour le solde).
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn s631_ac3_un_escompte_se_contre_passe_tva_comprise_dans_sa_periode(pool: MySqlPool) {
    let ctx = monter(&pool, NUIT_DU_NOUVEL_AN, Some("Open")).await;
    let fy_2026 = ctx.fy_2026.unwrap();
    sqlx::query(
        "UPDATE company_invoice_settings SET default_discount_account_id = ? WHERE company_id = ?",
    )
    .bind(ctx.s.accounts["4000"])
    .bind(ctx.s.company_id)
    .execute(&pool)
    .await
    .unwrap();
    let facture = ctx.facture_validee().await;
    let (s, b) = ctx
        .app
        .post(
            &format!("/api/v1/invoices/{facture}/write-off"),
            json!({
                "nature": "discount",
                "settledOn": "2025-12-20",
                "version": common::invoice_version(&pool, facture).await,
            }),
        )
        .await;
    assert!((200..300).contains(&s), "escompte : {b}");
    let (solde, ecriture): (i64, i64) = sqlx::query_as(
        "SELECT id, journal_entry_id FROM invoice_settlements WHERE invoice_id = ? \
         AND settlement_type = 'write_off'",
    )
    .bind(facture)
    .fetch_one(&pool)
    .await
    .unwrap();
    let tva_du_solde: rust_decimal::Decimal = sqlx::query_scalar(
        "SELECT COALESCE(SUM(debit - credit), 0) FROM journal_entry_lines \
         WHERE entry_id = ? AND account_id = ?",
    )
    .bind(ecriture)
    .bind(ctx.s.accounts["2000"])
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(
        tva_du_solde,
        dec!(8.10),
        "montage : la correction de TVA de l'escompte"
    );
    let dans_2026 = ctx.ecritures_de(fy_2026).await;

    let (s, b) = ctx
        .app
        .post_vide(&format!(
            "/api/v1/invoices/{facture}/settlements/{solde}/cancel"
        ))
        .await;
    assert_eq!(s, 200, "{b}");
    let inverse = ctx.inverse_de(ecriture).await.expect("contre-passée");
    assert_eq!(
        ctx.date_et_exercice(inverse).await,
        (jour(2025, 12, 20), ctx.fy_2025),
        "datée du solde, dans 2025"
    );
    assert_eq!(ctx.ecritures_de(fy_2026).await, dans_2026, "rien dans 2026");
    // Par compte, le solde et son inverse nettent à zéro **dans 2025** : la somme est
    // bornée à l'exercice (revue P1 de la 63-1, L8) — sans borne, elle serait nulle sous
    // toute datation, l'inverse du jour comprise.
    let nets: Vec<(i64, rust_decimal::Decimal)> = sqlx::query_as(
        "SELECT l.account_id, SUM(l.debit - l.credit) FROM journal_entry_lines l \
         JOIN journal_entries je ON je.id = l.entry_id \
         WHERE l.entry_id IN (?, ?) AND je.entry_date BETWEEN '2025-01-01' AND '2025-12-31' \
         GROUP BY l.account_id ORDER BY l.account_id",
    )
    .bind(ecriture)
    .bind(inverse)
    .fetch_all(&pool)
    .await
    .unwrap();
    assert!(
        nets.iter().any(|(a, _)| *a == ctx.s.accounts["2000"]),
        "la ligne de TVA est contre-passée : {nets:?}"
    );
    assert!(
        nets.iter().all(|(_, n)| n.is_zero()),
        "solde et inverse nettent à zéro : {nets:?}"
    );
}
