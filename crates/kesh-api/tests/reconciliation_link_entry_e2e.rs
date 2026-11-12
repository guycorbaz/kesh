//! Tests — rattacher une transaction bancaire à une écriture **déjà passée**
//! (Story v017-3a, #671).
//!
//! Couvre, de la fiche `v017-3a-rattacher-ecriture-existante.md` :
//! - **AC-2** — la règle et la lecture, au dépôt
//!   (`kesh_db::repositories::reconciliation_link::find_entry_candidates`), une
//!   condition par test, chacun avec une écriture qui passerait sans elle ;
//! - **AC-4** — le geste (`link_entry_in_tx`), par la route qui l'appelle sous
//!   le verrou nommé du compte, et les courses T-race-1 à 3 ;
//! - **AC-5**, **AC-6**, **AC-8** — les routes `GET …/entry-candidates` et
//!   `POST …/link-entry`, leurs refus, rôles, clés, étanchéité, et les suites
//!   du rattachement ;
//! - **AC-22** — rien de ce que porte la base ne change à la lecture.
//!
//! ⚠️ **Montage** (choix C-v017-3-40) : celui de `reconciliation_link_e2e.rs`
//! (Story 29-8), recopié — chaque fichier de test est un binaire, et ce
//! montage n'est partagé par aucun module commun. Les écritures sont saisies
//! par `POST /journal-entries`, les règlements par les routes de la fiche, les
//! transactions par la fonction du dépôt que la route d'import appelle
//! (`bank_imports::create_with_transactions`). Les états qui contredisent un
//! invariant (écriture d'une autre société portant une ligne sur le compte de
//! la banque, exercice clos, compte archivé, compte clos) sont **forgés** par
//! SQL, et chaque test le dit.
//!
//! Les tests de la règle (AC-2) appellent la fonction du dépôt directement :
//! c'est elle que la route et le geste lisent ; la route n'y ajoute que le
//! motif d'annulation, testé à part (AC-5).
//!
//! Pré-requis : MariaDB démarré.
#![allow(clippy::too_many_arguments)]

mod common;

use std::net::SocketAddr;
use std::path::Path;
use std::str::FromStr;
use std::sync::Arc;

use chrono::{NaiveDate, TimeDelta, Utc};
use jsonwebtoken::{Algorithm, EncodingKey, Header};
use kesh_api::auth::jwt::Claims;
use kesh_api::auth::password::hash_password;
use kesh_api::config::Config;
use kesh_api::{AppState, build_router};
use kesh_db::entities::address::StructuredAddress;
use kesh_db::entities::contact::{ContactType, NewContact};
use kesh_db::entities::{
    BankImportSourceFormat, Language, NewBankAccount, NewBankImport, NewBankTransaction,
    NewCompany, NewSupplierInvoice, NewSupplierInvoiceLine, NewUser, OrgType, Role,
};
use kesh_db::repositories::reconciliation_link::{
    EntryCandidate, count_already_booked_for_page, find_entry_candidates, find_link_candidates,
    find_matched_elsewhere,
};
use kesh_db::repositories::{
    bank_accounts, bank_imports, companies, contacts, reconciliation, supplier_invoices, users,
};
use kesh_db::test_fixtures::{
    SeededCompany, disable_rounding_to_5_centimes, mark_onboarding_complete,
    seed_accounting_company,
};
use rust_decimal::Decimal;
use serde_json::{Value, json};
use sqlx::MySqlPool;

const TEST_JWT_SECRET: &[u8] = b"test-secret-32-bytes-minimum-test-secret-padding";
const TEST_ADMIN_PASSWORD: &str = "e2e-test-admin-password";

const IBAN_A: &str = "CH4431999123000889012";
const IBAN_B: &str = "CH9300762011623852957";
const IBAN_C: &str = "CH5604835012345678009";

// ============================================================
// Serveur et jetons (recopiés de `reconciliation_link_e2e.rs`)
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
            Err(e) => panic!("test server not ready in 2s: {e}"),
        }
    }
    TestApp {
        base_url: format!("http://{addr}"),
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

fn d(s: &str) -> NaiveDate {
    NaiveDate::from_str(s).unwrap()
}

fn dec(s: &str) -> Decimal {
    Decimal::from_str(s).unwrap()
}

fn hash64(seed: &str) -> String {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    let mut h = DefaultHasher::new();
    seed.hash(&mut h);
    format!("{:0>64x}", h.finish())
}

fn assert_refus(status: u16, corps: &Value, attendu: u16, code: &str) {
    assert_eq!(status, attendu, "{corps}");
    assert_eq!(corps["error"]["code"], code, "{corps}");
}

fn assert_non_rattachable(status: u16, corps: &Value) {
    assert_refus(status, corps, 409, "RECONCILIATION_ENTRY_NOT_LINKABLE");
}

// ============================================================
// Contexte et montages
// ============================================================

/// Une société de `seed_accounting_company` (exercice 2020-2030 ouvert),
/// onboarding terminé, arrondi à 5 centimes désactivé, un compte 1020
/// *Banque* auquel les comptes bancaires sont liés ; le serveur et un JWT
/// `Admin`.
struct Ctx {
    pool: MySqlPool,
    app: TestApp,
    jwt: String,
    seeded: SeededCompany,
    ledger: i64,
}

async fn monter(pool: &MySqlPool) -> Ctx {
    let seeded = seed_accounting_company(pool).await.expect("seed");
    mark_onboarding_complete(pool).await.expect("onboarding");
    disable_rounding_to_5_centimes(pool, seeded.company_id)
        .await
        .expect("arrondi");
    let ledger = compte_du_grand_livre(pool, seeded.company_id, "1020", "Banque").await;
    let app = spawn_app(pool.clone()).await;
    let jwt = forge_jwt(seeded.admin_user_id, "Admin", seeded.company_id);
    Ctx {
        pool: pool.clone(),
        app,
        jwt,
        seeded,
        ledger,
    }
}

async fn compte_du_grand_livre(pool: &MySqlPool, company: i64, numero: &str, nom: &str) -> i64 {
    sqlx::query_scalar(
        "INSERT INTO accounts (company_id, number, name, account_type, active, postable) \
         VALUES (?, ?, ?, 'Asset', 1, 1) RETURNING id",
    )
    .bind(company)
    .bind(numero)
    .bind(nom)
    .fetch_one(pool)
    .await
    .unwrap()
}

/// Une seconde société, son administrateur et son JWT : `(société, utilisateur, jeton)`.
async fn autre_societe(pool: &MySqlPool, nom_admin: &str) -> (i64, i64, String) {
    let autre = companies::create(
        pool,
        NewCompany {
            name: "Autre SA".into(),
            first_name: None,
            last_name: None,
            address_structured: StructuredAddress {
                street: "Rue Autre".into(),
                building: "3".into(),
                postal_code: "1200".into(),
                city: "Genève".into(),
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
    let user = users::create(
        pool,
        NewUser {
            username: nom_admin.into(),
            password_hash: hash_password("password123").unwrap(),
            role: Role::Admin,
            active: true,
            company_id: autre,
            email: None,
        },
    )
    .await
    .unwrap()
    .id;
    (autre, user, forge_jwt(user, "Admin", autre))
}

impl Ctx {
    fn company_id(&self) -> i64 {
        self.seeded.company_id
    }

    fn compte(&self, numero: &str) -> i64 {
        self.seeded.accounts[numero]
    }

    async fn req_as(
        &self,
        jeton: &str,
        methode: reqwest::Method,
        chemin: &str,
        corps: Option<Value>,
    ) -> (u16, Value) {
        let mut r = self
            .app
            .client
            .request(methode, self.app.url(chemin))
            .bearer_auth(jeton);
        if let Some(c) = corps {
            r = r.json(&c);
        }
        let resp = r.send().await.unwrap();
        let status = resp.status().as_u16();
        (status, resp.json().await.unwrap_or(Value::Null))
    }

    async fn post(&self, chemin: &str, corps: Value) -> (u16, Value) {
        self.req_as(&self.jwt, reqwest::Method::POST, chemin, Some(corps))
            .await
    }

    async fn get(&self, chemin: &str) -> (u16, Value) {
        self.req_as(&self.jwt, reqwest::Method::GET, chemin, None)
            .await
    }

    /// Un compte bancaire non principal, lié à `ledger`.
    async fn banque_liee(&self, iban: &str, ledger: i64) -> i64 {
        let id = bank_accounts::create(
            &self.pool,
            NewBankAccount {
                company_id: self.company_id(),
                bank_name: "UBS".into(),
                iban: iban.into(),
                qr_iban: None,
                is_primary: false,
            },
        )
        .await
        .unwrap()
        .id;
        sqlx::query("UPDATE bank_accounts SET journal_account_id = ? WHERE id = ?")
            .bind(ledger)
            .bind(id)
            .execute(&self.pool)
            .await
            .unwrap();
        id
    }

    /// Un compte bancaire non principal lié au 1020.
    async fn banque(&self, iban: &str) -> i64 {
        self.banque_liee(iban, self.ledger).await
    }

    async fn version_banque(&self, id: i64) -> i64 {
        sqlx::query_scalar("SELECT version FROM bank_accounts WHERE id = ?")
            .bind(id)
            .fetch_one(&self.pool)
            .await
            .unwrap()
    }

    /// Une écriture saisie au journal **par la route** (`POST /journal-entries`),
    /// journal Banque : `lignes` = `(compte, débit, crédit)`. Rend son identifiant.
    async fn ecriture(&self, date: &str, libelle: &str, lignes: &[(i64, &str, &str)]) -> i64 {
        let lines: Vec<Value> = lignes
            .iter()
            .map(|(a, d, c)| json!({ "accountId": a, "debit": d, "credit": c }))
            .collect();
        let (s, b) = self
            .post(
                "/api/v1/journal-entries",
                json!({
                    "entryDate": date,
                    "journal": "Banque",
                    "description": libelle,
                    "lines": lines,
                }),
            )
            .await;
        assert_eq!(s, 201, "écriture : {b}");
        b["id"].as_i64().unwrap()
    }

    /// Une dépense payée par la banque : `D 4000 / C 1020`, de `montant`.
    async fn depense(&self, date: &str, montant: &str) -> i64 {
        self.ecriture(
            date,
            "Dépense payée",
            &[
                (self.compte("4000"), montant, "0"),
                (self.ledger, "0", montant),
            ],
        )
        .await
    }

    /// Un encaissement : `D 1020 / C 3000`, de `montant`.
    async fn recette(&self, date: &str, montant: &str) -> i64 {
        self.ecriture(
            date,
            "Recette encaissée",
            &[
                (self.ledger, montant, "0"),
                (self.compte("3000"), "0", montant),
            ],
        )
        .await
    }

    async fn contact(&self, nom: &str, client: bool) -> i64 {
        contacts::create(
            &self.pool,
            self.seeded.admin_user_id,
            NewContact {
                company_id: self.company_id(),
                contact_type: ContactType::Entreprise,
                name: nom.into(),
                first_name: None,
                last_name: None,
                is_client: client,
                is_supplier: !client,
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

    /// Une facture client validée par les routes (100.00 HT, 8,1 % → 108.10).
    async fn facture(&self, date: &str) -> i64 {
        let client = self.contact("Dubarde SàRL", true).await;
        let (s, b) = self
            .post(
                "/api/v1/invoices",
                json!({
                    "contactId": client,
                    "date": date,
                    "lines": [
                        {"description": "Conseil", "quantity": "1", "unitPrice": "100.00", "vatRate": "8.10"}
                    ]
                }),
            )
            .await;
        assert_eq!(s, 201, "facture : {b}");
        let id = b["id"].as_i64().unwrap();
        let (s, b) = self
            .req_as(
                &self.jwt,
                reqwest::Method::POST,
                &format!("/api/v1/invoices/{id}/validate"),
                None,
            )
            .await;
        assert_eq!(s, 200, "validation : {b}");
        id
    }

    /// Règle une facture client depuis sa fiche (`corps` : le choix de
    /// règlement) : `(règlement, écriture du règlement)`. La version courante de
    /// la facture s'y ajoute (Story 34-5, #951).
    async fn regler_avec(&self, facture: i64, mut corps: Value) -> (i64, i64) {
        corps["version"] = common::invoice_version(&self.pool, facture).await.into();
        let (s, b) = self
            .post(&format!("/api/v1/invoices/{facture}/settlements"), corps)
            .await;
        assert_eq!(s, 200, "règlement : {b}");
        let entry = b["journalEntryId"].as_i64().expect("journalEntryId");
        let sid: i64 =
            sqlx::query_scalar("SELECT id FROM invoice_settlements WHERE journal_entry_id = ?")
                .bind(entry)
                .fetch_one(&self.pool)
                .await
                .unwrap();
        (sid, entry)
    }

    /// Règle par virement sur `bank`, depuis la fiche.
    async fn regler_virement(
        &self,
        facture: i64,
        bank: i64,
        montant: &str,
        le: &str,
    ) -> (i64, i64) {
        self.regler_avec(
            facture,
            json!({
                "settlementType": "bank_transfer",
                "bankAccountId": bank,
                "amount": montant,
                "settledOn": le,
            }),
        )
        .await
    }

    /// Règle par **compte interne** `compte`, depuis la fiche.
    async fn regler_interne(
        &self,
        facture: i64,
        compte: i64,
        montant: &str,
        le: &str,
    ) -> (i64, i64) {
        self.regler_avec(
            facture,
            json!({
                "settlementType": "internal_account",
                "accountId": compte,
                "amount": montant,
                "settledOn": le,
            }),
        )
        .await
    }

    /// Une facture fournisseur ouverte de `montant` (sans TVA), IBAN du
    /// créancier posé (exigé par un lot de paiement).
    async fn facture_fournisseur(&self, numero: &str, montant: &str) -> i64 {
        // Le seed ne pose pas de compte fournisseurs par défaut : montage de
        // `supplier_settlement_cancel_e2e.rs::open_invoice`.
        sqlx::query(
            "UPDATE company_invoice_settings SET default_payable_account_id = ? WHERE company_id = ?",
        )
        .bind(self.compte("2000"))
        .bind(self.company_id())
        .execute(&self.pool)
        .await
        .unwrap();
        let fournisseur = self.contact("Fournisseur SA", false).await;
        let id = supplier_invoices::create_validated(
            &self.pool,
            NewSupplierInvoice {
                company_id: self.company_id(),
                contact_id: fournisseur,
                supplier_invoice_number: Some(numero.into()),
                invoice_date: d("2026-06-15"),
                due_date: None,
                creditor_iban: None,
                creditor_qr_iban: None,
                payment_reference: None,
                expected_payment_amount: None,
                project_id: None,
                lines: vec![NewSupplierInvoiceLine {
                    description: "Prestation".into(),
                    quantity: dec("1"),
                    unit_price: dec(montant),
                    vat_rate: dec("0"),
                    expense_account_id: self.compte("4000"),
                }],
            },
            self.seeded.admin_user_id,
        )
        .await
        .expect("facture fournisseur")
        .invoice
        .id;
        sqlx::query("UPDATE supplier_invoices SET creditor_iban = ? WHERE id = ?")
            .bind(IBAN_C)
            .bind(id)
            .execute(&self.pool)
            .await
            .unwrap();
        id
    }

    /// Paie une facture fournisseur par virement sur `bank`, **depuis sa fiche**
    /// (`POST /supplier-invoices/{id}/pay`) ; rend l'écriture de règlement.
    async fn payer_fournisseur(&self, id: i64, bank: i64, le: &str) -> i64 {
        let (s, b) = self
            .post(
                &format!("/api/v1/supplier-invoices/{id}/pay"),
                json!({ "settlementType": "bank_transfer", "bankAccountId": bank, "paymentDate": le }),
            )
            .await;
        assert_eq!(s, 200, "paiement : {b}");
        self.ecriture_de_reglement_fournisseur(id).await
    }

    async fn ecriture_de_reglement_fournisseur(&self, id: i64) -> i64 {
        sqlx::query_scalar("SELECT settlement_journal_entry_id FROM supplier_invoices WHERE id = ?")
            .bind(id)
            .fetch_one(&self.pool)
            .await
            .unwrap()
    }

    /// Un lot pain.001 des `factures`, généré puis confirmé, sur `bank`.
    async fn lot_confirme(&self, bank: i64, factures: &[i64], le: &str) {
        let (s, b) = self
            .post(
                "/api/v1/payment-batches",
                json!({
                    "bankAccountId": bank,
                    "requestedExecutionDate": le,
                    "supplierInvoiceIds": factures,
                }),
            )
            .await;
        assert_eq!(s, 200, "lot : {b}");
        let batch = b["batch"]["id"]
            .as_i64()
            .unwrap_or_else(|| panic!("lot créé : {b}"));
        let (s, b) = self
            .post(
                &format!("/api/v1/payment-batches/{batch}/confirm"),
                json!({ "paymentDate": le }),
            )
            .await;
        assert_eq!(s, 200, "confirmation : {b}");
    }

    /// Une transaction `pending` de `montant` en `devise`, datée `le`, par le
    /// dépôt que la route d'import appelle.
    async fn transaction_en(&self, bank: i64, montant: &str, devise: &str, le: &str) -> i64 {
        let jour = d(le);
        let cle = format!(
            "{bank}-{montant}-{devise}-{le}-{}",
            Utc::now().timestamp_nanos_opt().unwrap()
        );
        let mut tx = self.pool.begin().await.unwrap();
        let (_, txs) = bank_imports::create_with_transactions(
            &mut tx,
            NewBankImport {
                company_id: self.company_id(),
                bank_account_id: bank,
                filename: "releve.xml".into(),
                file_hash: hash64(&cle),
                source_format: BankImportSourceFormat::Camt053V04,
                statement_id: Some(format!("ENTRY-{cle}")),
                period_from: jour,
                period_to: jour,
                opening_balance: None,
                closing_balance: None,
                transaction_count: 1,
                imported_by_user_id: self.seeded.admin_user_id,
            },
            vec![NewBankTransaction {
                company_id: self.company_id(),
                bank_account_id: bank,
                booking_date: jour,
                value_date: Some(jour),
                amount: dec(montant),
                currency: devise.into(),
                reference: None,
                details: "Mouvement".into(),
                end_to_end_id: None,
                transaction_id: None,
                counterparty_iban: None,
                counterparty_name: Some("Contrepartie SA".into()),
            }],
        )
        .await
        .expect("import");
        tx.commit().await.unwrap();
        txs[0].id
    }

    async fn transaction(&self, bank: i64, montant: &str, le: &str) -> i64 {
        self.transaction_en(bank, montant, "CHF", le).await
    }

    /// La règle, **au dépôt** : les candidates de la transaction `tx` du compte
    /// `bank`, pour le compte du grand livre `ledger`.
    async fn regle_sur(&self, bank: i64, tx: i64, ledger: i64) -> Vec<EntryCandidate> {
        let t =
            reconciliation::find_pending_by_id_for_account(&self.pool, self.company_id(), bank, tx)
                .await
                .unwrap()
                .expect("transaction");
        find_entry_candidates(&self.pool, self.company_id(), &t, ledger)
            .await
            .unwrap()
    }

    /// Les identifiants des écritures candidates, au dépôt, sur le 1020.
    async fn regle(&self, bank: i64, tx: i64) -> Vec<i64> {
        self.regle_sur(bank, tx, self.ledger)
            .await
            .iter()
            .map(|c| c.journal_entry_id)
            .collect()
    }

    async fn candidates(&self, tx: i64) -> (u16, Value) {
        self.get(&format!(
            "/api/v1/reconciliation/transactions/{tx}/entry-candidates"
        ))
        .await
    }

    /// Les identifiants des écritures candidates, par la route, dans l'ordre.
    async fn ids_candidates(&self, tx: i64) -> Vec<i64> {
        let (s, b) = self.candidates(tx).await;
        assert_eq!(s, 200, "candidates : {b}");
        b["candidates"]
            .as_array()
            .unwrap()
            .iter()
            .map(|c| c["journalEntryId"].as_i64().unwrap())
            .collect()
    }

    async fn rattacher(&self, tx: i64, entry: i64) -> (u16, Value) {
        self.post(
            &format!("/api/v1/reconciliation/transactions/{tx}/link-entry"),
            json!({ "journalEntryId": entry }),
        )
        .await
    }

    async fn annuler_rapprochement(&self, tx: i64) -> (u16, Value) {
        self.post(
            &format!("/api/v1/reconciliation/transactions/{tx}/cancel"),
            json!({}),
        )
        .await
    }

    async fn contre_passer(&self, entry: i64) -> (u16, Value) {
        self.req_as(
            &self.jwt,
            reqwest::Method::POST,
            &format!("/api/v1/journal-entries/{entry}/reverse"),
            None,
        )
        .await
    }

    async fn etat(&self, tx: i64) -> (String, Option<i64>) {
        sqlx::query_as("SELECT status, matched_entry_id FROM bank_transactions WHERE id = ?")
            .bind(tx)
            .fetch_one(&self.pool)
            .await
            .unwrap()
    }

    async fn compter(&self, sql: &str) -> i64 {
        sqlx::query_scalar(sql)
            .bind(self.company_id())
            .fetch_one(&self.pool)
            .await
            .unwrap_or_else(|e| panic!("{sql} : {e}"))
    }

    async fn ecritures(&self) -> i64 {
        self.compter("SELECT COUNT(*) FROM journal_entries WHERE company_id = ?")
            .await
    }

    async fn liens_vers(&self, entry: i64) -> i64 {
        sqlx::query_scalar("SELECT COUNT(*) FROM bank_transactions WHERE matched_entry_id = ?")
            .bind(entry)
            .fetch_one(&self.pool)
            .await
            .unwrap()
    }

    async fn contre_passations_de(&self, entry: i64) -> i64 {
        sqlx::query_scalar("SELECT COUNT(*) FROM journal_entries WHERE reverses_entry_id = ?")
            .bind(entry)
            .fetch_one(&self.pool)
            .await
            .unwrap()
    }

    async fn clore_banque_sql(&self, bank: i64) {
        sqlx::query("UPDATE bank_accounts SET closed_on = '2026-01-31' WHERE id = ?")
            .bind(bank)
            .execute(&self.pool)
            .await
            .unwrap();
    }

    /// L'exercice du seed (2020-2030) ramené à 2020-2025 ; un exercice
    /// 2026-2030 ouvert. Rend l'identifiant du nouvel exercice.
    async fn scinder_l_exercice(&self) -> i64 {
        sqlx::query("UPDATE fiscal_years SET end_date = '2025-12-31' WHERE id = ?")
            .bind(self.seeded.fiscal_year_id)
            .execute(&self.pool)
            .await
            .unwrap();
        sqlx::query(
            "INSERT INTO fiscal_years (company_id, name, start_date, end_date, status) \
             VALUES (?, 'Exercice 2026-2030', '2026-01-01', '2030-12-31', 'Open')",
        )
        .bind(self.company_id())
        .execute(&self.pool)
        .await
        .unwrap()
        .last_insert_id() as i64
    }

    async fn clore_exercice_sql(&self, fy: i64) {
        sqlx::query("UPDATE fiscal_years SET status = 'Closed' WHERE id = ?")
            .bind(fy)
            .execute(&self.pool)
            .await
            .unwrap();
    }
}

/// L'état K2 (`PENDING_BANK_TRANSACTIONS`) de la préparation de clôture de
/// l'exercice `fy` (`GET /fiscal-years/{id}/closing-checks`) : `(statut, nombre)`.
async fn k2(ctx: &Ctx, fy: i64) -> (String, i64) {
    let (s, b) = ctx
        .get(&format!("/api/v1/fiscal-years/{fy}/closing-checks"))
        .await;
    assert_eq!(s, 200, "{b}");
    let k2 = b["checks"]
        .as_array()
        .unwrap()
        .iter()
        .find(|c| c["code"] == "PENDING_BANK_TRANSACTIONS")
        .cloned()
        .expect("K2");
    (
        k2["status"].as_str().unwrap().to_string(),
        k2["count"].as_i64().unwrap(),
    )
}

// ============================================================
// AC-2 — la règle, au dépôt
// ============================================================

/// AC-2.1 — un débit de `-250.00` et une écriture saisie `D charge / C banque`
/// de 250 : candidate, montant `-250.00`, et ses champs.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn ac2_1_un_debit_et_l_ecriture_de_la_depense(pool: MySqlPool) {
    let ctx = monter(&pool).await;
    let bank = ctx.banque(IBAN_A).await;
    let e = ctx.depense("2026-06-18", "250.00").await;
    let t = ctx.transaction(bank, "-250.00", "2026-06-20").await;
    let c = ctx.regle_sur(bank, t, ctx.ledger).await;
    assert_eq!(c.len(), 1, "{c:?}");
    assert_eq!(c[0].journal_entry_id, e);
    assert_eq!(c[0].amount.to_string(), "-250.00");
    assert_eq!(c[0].days_apart, 2);
    assert_eq!(c[0].entry_date, d("2026-06-18"));
    assert_eq!(c[0].fiscal_year_id, ctx.seeded.fiscal_year_id);
    assert_eq!(c[0].description, "Dépense payée");
    assert_eq!(c[0].journal.as_str(), "Banque");
    assert!(c[0].entry_number > 0);
}

/// AC-2.2 — le signe : un encaissement est candidat d'un crédit, jamais d'un
/// débit du même montant (tue M1).
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn ac2_2_le_signe_du_mouvement(pool: MySqlPool) {
    let ctx = monter(&pool).await;
    let bank = ctx.banque(IBAN_A).await;
    let e = ctx.recette("2026-06-18", "80.00").await;
    let credit = ctx.transaction(bank, "80.00", "2026-06-18").await;
    let debit = ctx.transaction(bank, "-80.00", "2026-06-18").await;
    let c = ctx.regle_sur(bank, credit, ctx.ledger).await;
    assert_eq!(
        c.iter().map(|c| c.journal_entry_id).collect::<Vec<_>>(),
        vec![e]
    );
    assert_eq!(c[0].amount.to_string(), "80.00");
    assert!(ctx.regle(bank, debit).await.is_empty());
}

/// AC-2.3 — un centime d'écart : absente (montant exact).
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn ac2_3_un_centime_d_ecart(pool: MySqlPool) {
    let ctx = monter(&pool).await;
    let bank = ctx.banque(IBAN_A).await;
    ctx.depense("2026-06-18", "250.00").await;
    let t = ctx.transaction(bank, "-250.01", "2026-06-18").await;
    assert!(ctx.regle(bank, t).await.is_empty());
}

/// AC-2.4 — une écriture passée sur un **autre** compte du grand livre (le
/// 1100, sans lien avec ce compte bancaire) n'est pas candidate ; ni une
/// écriture passée sur l'**ancien** compte lié (le 1020, avant le relien au
/// 1021).
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn ac2_4_un_autre_compte_du_grand_livre(pool: MySqlPool) {
    let ctx = monter(&pool).await;
    let bank = ctx.banque(IBAN_A).await;
    ctx.ecriture(
        "2026-06-18",
        "Sur un autre compte",
        &[
            (ctx.compte("4000"), "250.00", "0"),
            (ctx.compte("1100"), "0", "250.00"),
        ],
    )
    .await;
    let t = ctx.transaction(bank, "-250.00", "2026-06-18").await;
    assert!(ctx.regle(bank, t).await.is_empty());

    // L'ancien compte lié : la dépense passée sur le 1020, puis le compte
    // bancaire relié au 1021 (par la route).
    ctx.depense("2026-06-19", "75.00").await;
    let nouveau = compte_du_grand_livre(&pool, ctx.company_id(), "1021", "Banque 2").await;
    let v = ctx.version_banque(bank).await;
    let (s, b) = ctx
        .req_as(
            &ctx.jwt,
            reqwest::Method::PATCH,
            &format!("/api/v1/bank-accounts/{bank}"),
            Some(json!({ "journalAccountId": nouveau, "version": v })),
        )
        .await;
    assert_eq!(s, 200, "{b}");
    let t2 = ctx.transaction(bank, "-75.00", "2026-06-19").await;
    assert!(ctx.regle_sur(bank, t2, nouveau).await.is_empty());
}

/// AC-2.5 — une écriture déjà rapprochée d'une autre transaction n'est plus
/// candidate (tue M3). Le lien est posé par SQL : c'est l'état, non le geste,
/// qu'on éprouve ici.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn ac2_5_une_ecriture_deja_rapprochee(pool: MySqlPool) {
    let ctx = monter(&pool).await;
    let bank = ctx.banque(IBAN_A).await;
    let e = ctx.depense("2026-06-18", "250.00").await;
    let t1 = ctx.transaction(bank, "-250.00", "2026-06-18").await;
    let t2 = ctx.transaction(bank, "-250.00", "2026-06-19").await;
    assert_eq!(
        ctx.regle(bank, t2).await,
        vec![e],
        "témoin : candidate avant le lien"
    );
    sqlx::query(
        "UPDATE bank_transactions SET status = 'reconciled', matched_entry_id = ? WHERE id = ?",
    )
    .bind(e)
    .bind(t1)
    .execute(&pool)
    .await
    .unwrap();
    assert!(ctx.regle(bank, t2).await.is_empty());
}

/// AC-2.6 — une écriture contre-passée n'est plus candidate (tue M4) ; sa
/// contre-passation non plus, éprouvée contre une transaction de montant
/// **opposé**, sans quoi son mouvement de signe opposé l'écarterait seul (tue
/// M5).
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn ac2_6_contre_passee_et_contre_passation(pool: MySqlPool) {
    let ctx = monter(&pool).await;
    let bank = ctx.banque(IBAN_A).await;
    let e = ctx.depense("2026-06-18", "250.00").await;
    let (s, b) = ctx.contre_passer(e).await;
    assert_eq!(s, 201, "contre-passation : {b}");
    let r = b["id"].as_i64().unwrap();
    let debit = ctx.transaction(bank, "-250.00", "2026-06-18").await;
    let credit = ctx.transaction(bank, "250.00", "2026-06-18").await;
    assert!(
        ctx.regle(bank, debit).await.is_empty(),
        "l'écriture contre-passée"
    );
    assert!(
        !ctx.regle(bank, credit).await.contains(&r),
        "la contre-passation (D 1020 250)"
    );
}

/// AC-2.7 — un règlement client **par virement sur ce compte bancaire**, au
/// montant exact, relève de la 29-8 : absent de cette liste, présent de
/// `find_link_candidates` (tue M6). Un règlement client **par compte interne**
/// dont la contrepartie est le compte de la banque, lui, est présent.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn ac2_7_le_domaine_de_la_29_8(pool: MySqlPool) {
    let ctx = monter(&pool).await;
    let bank = ctx.banque(IBAN_A).await;
    let f1 = ctx.facture("2026-06-01").await;
    let (sid, e_virement) = ctx.regler_virement(f1, bank, "108.10", "2026-06-20").await;
    let t = ctx.transaction(bank, "108.10", "2026-06-20").await;
    assert!(!ctx.regle(bank, t).await.contains(&e_virement));
    let tt = reconciliation::find_pending_by_id_for_account(&pool, ctx.company_id(), bank, t)
        .await
        .unwrap()
        .unwrap();
    let liste_29_8 = find_link_candidates(&pool, ctx.company_id(), &tt, ctx.ledger, dec("0.05"))
        .await
        .unwrap();
    assert_eq!(
        liste_29_8
            .iter()
            .map(|c| c.settlement_id)
            .collect::<Vec<_>>(),
        vec![sid],
        "présent de la liste de la 29-8"
    );

    let f2 = ctx.facture("2026-06-02").await;
    let (_, e_interne) = ctx
        .regler_interne(f2, ctx.ledger, "54.05", "2026-06-21")
        .await;
    let t2 = ctx.transaction(bank, "54.05", "2026-06-21").await;
    assert_eq!(ctx.regle(bank, t2).await, vec![e_interne]);
}

/// AC-2.8 — un règlement fournisseur depuis la fiche (`pay_in_tx`) est
/// candidat ; un lot pain.001 d'**une** facture confirmé aussi ; un lot de
/// **deux** factures contre un débit égal à leur somme ne l'est pas — aucune
/// écriture n'égale la somme (angle mort L1, #823).
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn ac2_8_reglements_fournisseur(pool: MySqlPool) {
    let ctx = monter(&pool).await;
    let bank = ctx.banque(IBAN_A).await;

    let fiche = ctx.facture_fournisseur("FF-1", "100.00").await;
    let e_fiche = ctx.payer_fournisseur(fiche, bank, "2026-06-20").await;
    let t = ctx.transaction(bank, "-100.00", "2026-06-21").await;
    assert_eq!(ctx.regle(bank, t).await, vec![e_fiche]);

    let seule = ctx.facture_fournisseur("FF-2", "40.00").await;
    ctx.lot_confirme(bank, &[seule], "2026-07-01").await;
    let e_lot = ctx.ecriture_de_reglement_fournisseur(seule).await;
    let t_lot = ctx.transaction(bank, "-40.00", "2026-07-01").await;
    assert_eq!(ctx.regle(bank, t_lot).await, vec![e_lot]);

    let a = ctx.facture_fournisseur("FF-3", "30.00").await;
    let b = ctx.facture_fournisseur("FF-4", "20.00").await;
    ctx.lot_confirme(bank, &[a, b], "2026-07-02").await;
    let t_somme = ctx.transaction(bank, "-50.00", "2026-07-02").await;
    assert!(
        ctx.regle(bank, t_somme).await.is_empty(),
        "L1 : aucune écriture n'égale la somme"
    );
}

/// AC-2.9 — une autre société : absente (tue M18). ⚠️ **Fixture SQL
/// directe** : une écriture de la société B portant une ligne sur le compte du
/// grand livre **de A** — état qu'aucune route ne produit. Une fixture par les
/// routes n'aurait, pour B, aucune ligne sur ce compte : la condition 3
/// l'écarterait seule, et le test serait muet.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn ac2_9_une_autre_societe(pool: MySqlPool) {
    let ctx = monter(&pool).await;
    let bank = ctx.banque(IBAN_A).await;
    let e = ctx.depense("2026-06-18", "250.00").await;
    let t = ctx.transaction(bank, "-250.00", "2026-06-18").await;
    assert_eq!(ctx.regle(bank, t).await, vec![e], "témoin");
    let (autre, _, _) = autre_societe(&pool, "autre_admin").await;
    let fy_autre = sqlx::query(
        "INSERT INTO fiscal_years (company_id, name, start_date, end_date, status) \
         VALUES (?, '2026', '2026-01-01', '2026-12-31', 'Open')",
    )
    .bind(autre)
    .execute(&pool)
    .await
    .unwrap()
    .last_insert_id() as i64;
    sqlx::query("UPDATE journal_entries SET company_id = ?, fiscal_year_id = ? WHERE id = ?")
        .bind(autre)
        .bind(fy_autre)
        .bind(e)
        .execute(&pool)
        .await
        .unwrap();
    assert!(ctx.regle(bank, t).await.is_empty());
}

/// AC-2.10 — le tri : jours d'écart croissants, puis identifiant (tue M9) ;
/// et 51 candidates → 50 rendues, la plus lointaine coupée (tue M19).
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn ac2_10_le_tri_et_la_limite(pool: MySqlPool) {
    let ctx = monter(&pool).await;
    let bank = ctx.banque(IBAN_A).await;
    let a = ctx.depense("2026-06-13", "33.00").await; // 2 jours
    let b = ctx.depense("2026-05-16", "33.00").await; // 30 jours
    let c = ctx.depense("2026-06-17", "33.00").await; // 2 jours
    let t = ctx.transaction(bank, "-33.00", "2026-06-15").await;
    let rendues = ctx.regle_sur(bank, t, ctx.ledger).await;
    assert_eq!(
        rendues
            .iter()
            .map(|c| c.journal_entry_id)
            .collect::<Vec<_>>(),
        vec![a, c, b]
    );
    assert_eq!(
        rendues.iter().map(|c| c.days_apart).collect::<Vec<_>>(),
        vec![2, 2, 30]
    );

    let debut = d("2026-03-01");
    let mut ids = Vec::new();
    for k in 0..51 {
        let jour = (debut + chrono::Days::new(k)).to_string();
        ids.push(ctx.depense(&jour, "12.34").await);
    }
    let t51 = ctx.transaction(bank, "-12.34", "2026-04-20").await;
    let rendues = ctx.regle(bank, t51).await;
    assert_eq!(rendues.len(), 50);
    assert!(
        !rendues.contains(&ids[0]),
        "la plus lointaine (50 jours) est coupée"
    );
}

/// AC-2.11 — une transaction en EUR, ou de montant nul : liste vide
/// (condition 1 ; tue M27).
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn ac2_11_eur_ou_nulle(pool: MySqlPool) {
    let ctx = monter(&pool).await;
    let bank = ctx.banque(IBAN_A).await;
    ctx.depense("2026-06-18", "250.00").await;
    let eur = ctx
        .transaction_en(bank, "-250.00", "EUR", "2026-06-18")
        .await;
    assert!(ctx.regle(bank, eur).await.is_empty());
    let nulle = ctx.transaction(bank, "0.00", "2026-06-18").await;
    assert!(ctx.regle(bank, nulle).await.is_empty());
}

/// AC-2.12 — deux comptes bancaires **partageant** le compte du grand livre
/// (état admis, aucune contrainte `UNIQUE`) : une écriture non rapprochée est
/// candidate pour l'un comme pour l'autre (angle mort L3, asserté pour qu'il ne
/// change pas en silence) ; et un règlement client **par virement sur B**, au
/// montant d'une transaction de A, est candidat pour A — la condition 6
/// n'exclut que le domaine de la 29-8 **du compte de `t`** (tue M7bis).
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn ac2_12_comptes_bancaires_partageant_le_grand_livre(pool: MySqlPool) {
    let ctx = monter(&pool).await;
    let bank_a = ctx.banque(IBAN_A).await;
    let bank_b = ctx.banque(IBAN_B).await;
    let e = ctx.depense("2026-06-18", "70.00").await;
    let t_b = ctx.transaction(bank_b, "-70.00", "2026-06-18").await;
    assert_eq!(ctx.regle(bank_b, t_b).await, vec![e]);

    let f = ctx.facture("2026-06-01").await;
    let (_, e_b) = ctx.regler_virement(f, bank_b, "108.10", "2026-06-20").await;
    let t_a = ctx.transaction(bank_a, "108.10", "2026-06-20").await;
    assert_eq!(ctx.regle(bank_a, t_a).await, vec![e_b]);
}

// ============================================================
// AC-4 / AC-6 / AC-8 — le geste et ses routes
// ============================================================

/// AC-8 (a) et AC-4 nominal — une écriture saisie au journal : la liste puis
/// le geste. État final, aucune écriture, une trace et ses détails. Puis les
/// suites : la clôture du compte bancaire passe (8.1), K2 n'en compte plus
/// (8.2), la contre-passation directe est refusée (8.4), et *Annuler le
/// rapprochement* contre-passe l'écriture saisie (8.3).
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn ac8_a_une_ecriture_saisie(pool: MySqlPool) {
    let ctx = monter(&pool).await;
    let bank = ctx.banque(IBAN_A).await;
    let e = ctx.depense("2026-06-18", "250.00").await;
    let t = ctx.transaction(bank, "-250.00", "2026-06-20").await;

    let v = ctx.version_banque(bank).await;
    let (s, b) = ctx
        .post(
            &format!("/api/v1/bank-accounts/{bank}/close"),
            json!({ "version": v, "closedOn": "2026-07-31" }),
        )
        .await;
    assert_refus(s, &b, 412, "BANK_ACCOUNT_HAS_PENDING_TRANSACTIONS");
    assert_eq!(
        k2(&ctx, ctx.seeded.fiscal_year_id).await,
        ("attention".to_string(), 1)
    );

    let (s, b) = ctx.candidates(t).await;
    assert_eq!(s, 200, "{b}");
    assert_eq!(b["bankTransactionId"], t);
    let c = &b["candidates"][0];
    assert_eq!(c["journalEntryId"], e);
    assert_eq!(c["amount"], "-250.00");
    assert_eq!(c["daysApart"], 2);
    assert_eq!(c["entryDate"], "2026-06-18");
    assert_eq!(c["journal"], "Banque");
    assert_eq!(c["description"], "Dépense payée");
    assert_eq!(c["fiscalYearId"], ctx.seeded.fiscal_year_id);
    assert!(c["entryNumber"].as_i64().unwrap() > 0);
    assert!(c["cancelBlockedBy"].is_null(), "{c}");
    assert!(c["cancelBlockedLabel"].is_null(), "{c}");
    let numero = c["entryNumber"].as_i64().unwrap();

    let ecritures = ctx.ecritures().await;
    let (s, b) = ctx.rattacher(t, e).await;
    assert_eq!(s, 200, "{b}");
    assert_eq!(b, json!({ "bankTransactionId": t, "journalEntryId": e }));
    assert_eq!(ctx.etat(t).await, ("reconciled".to_string(), Some(e)));
    assert_eq!(ctx.ecritures().await, ecritures, "aucune écriture");
    let actions = common::audit_actions(&pool, "bank_transaction", t).await;
    assert_eq!(actions, vec!["reconciliation.entry_linked".to_string()]);
    let details =
        common::audit_details(&pool, "bank_transaction", t, "reconciliation.entry_linked")
            .await
            .expect("trace");
    assert_eq!(
        details,
        json!({
            "bank_transaction_id": t,
            "journal_entry_id": e,
            "entry_number": numero,
            "fiscal_year_id": ctx.seeded.fiscal_year_id,
            "amount": "-250.00",
            "was_previously_rejected": false,
        })
    );

    // 8.2 — K2.
    assert_eq!(
        k2(&ctx, ctx.seeded.fiscal_year_id).await,
        ("ok".to_string(), 0)
    );
    // 8.4 — la contre-passation directe est refusée.
    let (s, b) = ctx.contre_passer(e).await;
    assert_refus(s, &b, 409, "MATCHED_BANK_TRANSACTION");
    // La lecture du rapprochement : une écriture propre, annulable.
    let (s, b) = ctx
        .get(&format!("/api/v1/reconciliation/transactions/{t}"))
        .await;
    assert_eq!(s, 200, "{b}");
    assert_eq!(b["kind"], "entry");
    assert_eq!(b["cancellable"], true);

    // 8.1 — la clôture du compte passe.
    let v = ctx.version_banque(bank).await;
    let (s, b) = ctx
        .post(
            &format!("/api/v1/bank-accounts/{bank}/close"),
            json!({ "version": v, "closedOn": "2026-07-31" }),
        )
        .await;
    assert_eq!(s, 200, "la clôture réussit après le rattachement : {b}");
}

/// AC-8.3 (a) — *Annuler le rapprochement* d'une écriture saisie rattachée la
/// **contre-passe**, et la transaction redevient `pending` (C-v017-3-9).
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn ac8_3_a_annuler_contre_passe_l_ecriture_saisie(pool: MySqlPool) {
    let ctx = monter(&pool).await;
    let bank = ctx.banque(IBAN_A).await;
    let e = ctx.depense("2026-06-18", "250.00").await;
    let t = ctx.transaction(bank, "-250.00", "2026-06-20").await;
    let (s, b) = ctx.rattacher(t, e).await;
    assert_eq!(s, 200, "{b}");
    let (s, b) = ctx.annuler_rapprochement(t).await;
    assert_eq!(s, 200, "{b}");
    assert_eq!(ctx.etat(t).await, ("pending".to_string(), None));
    assert_eq!(ctx.contre_passations_de(e).await, 1);
}

/// AC-8 (b) — une facture fournisseur payée depuis sa fiche : la liste, le
/// geste ; l'annulation du règlement depuis la fiche est refusée (8.4) ;
/// *Annuler le rapprochement* annule le règlement, la facture redevient
/// `open` (8.3).
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn ac8_b_un_reglement_fournisseur(pool: MySqlPool) {
    let ctx = monter(&pool).await;
    let bank = ctx.banque(IBAN_A).await;
    let fi = ctx.facture_fournisseur("FF-1", "100.00").await;
    let e = ctx.payer_fournisseur(fi, bank, "2026-06-20").await;
    let t = ctx.transaction(bank, "-100.00", "2026-06-22").await;
    assert_eq!(ctx.ids_candidates(t).await, vec![e]);
    let (s, b) = ctx.rattacher(t, e).await;
    assert_eq!(s, 200, "{b}");
    assert_eq!(ctx.etat(t).await, ("reconciled".to_string(), Some(e)));

    let (s, b) = ctx
        .post(
            &format!("/api/v1/supplier-invoices/{fi}/settlement/cancel"),
            json!({}),
        )
        .await;
    assert_refus(s, &b, 409, "MATCHED_BANK_TRANSACTION");

    let (s, b) = ctx.annuler_rapprochement(t).await;
    assert_eq!(s, 200, "{b}");
    assert_eq!(b["supplierInvoiceId"], fi, "{b}");
    assert_eq!(ctx.etat(t).await, ("pending".to_string(), None));
    let statut: String = sqlx::query_scalar("SELECT status FROM supplier_invoices WHERE id = ?")
        .bind(fi)
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(statut, "open");
}

/// AC-8 (c) — un règlement client par **compte interne** dont la contrepartie
/// est le compte de la banque : la liste, le geste ; *Annuler le
/// rapprochement* retire le règlement, la facture redevient due (8.3).
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn ac8_c_un_reglement_client_par_compte_interne(pool: MySqlPool) {
    let ctx = monter(&pool).await;
    let bank = ctx.banque(IBAN_A).await;
    let f = ctx.facture("2026-06-01").await;
    let (sid, e) = ctx
        .regler_interne(f, ctx.ledger, "108.10", "2026-06-20")
        .await;
    let t = ctx.transaction(bank, "108.10", "2026-06-20").await;
    assert_eq!(ctx.ids_candidates(t).await, vec![e]);
    let (s, b) = ctx.rattacher(t, e).await;
    assert_eq!(s, 200, "{b}");
    let (s, b) = ctx.annuler_rapprochement(t).await;
    assert_eq!(s, 200, "{b}");
    assert_eq!(b["invoiceId"], f, "{b}");
    assert_eq!(ctx.etat(t).await, ("pending".to_string(), None));
    let reste: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM invoice_settlements WHERE id = ?")
        .bind(sid)
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(reste, 0, "le règlement est retiré");
    let (s, b) = ctx.get(&format!("/api/v1/invoices/{f}")).await;
    assert_eq!(s, 200, "{b}");
    assert_eq!(dec(b["amountDue"].as_str().unwrap()), dec("108.10"));
}

/// AC-4 — une transaction **rejetée** se rattache : le marqueur est remis à
/// `NULL`, la trace le dit (tue M11).
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn ac4_une_transaction_rejetee_se_rattache(pool: MySqlPool) {
    let ctx = monter(&pool).await;
    let bank = ctx.banque(IBAN_A).await;
    let e = ctx.depense("2026-06-18", "250.00").await;
    let t = ctx.transaction(bank, "-250.00", "2026-06-18").await;
    let (s, b) = ctx
        .post(
            "/api/v1/reconciliation/reject",
            json!({ "bankAccountId": bank, "bankTransactionIds": [t] }),
        )
        .await;
    assert_eq!(s, 200, "{b}");
    assert_eq!(ctx.ids_candidates(t).await, vec![e]);
    let (s, b) = ctx.rattacher(t, e).await;
    assert_eq!(s, 200, "{b}");
    let rejete: Option<chrono::NaiveDateTime> =
        sqlx::query_scalar("SELECT auto_match_rejected_at FROM bank_transactions WHERE id = ?")
            .bind(t)
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(rejete, None);
    let details =
        common::audit_details(&pool, "bank_transaction", t, "reconciliation.entry_linked")
            .await
            .unwrap();
    assert_eq!(details["was_previously_rejected"], true);
}

/// AC-4 — deux rattachements **successifs** de la même écriture à deux
/// transactions du même compte : le second est refusé, rien d'écrit (tue M3).
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn ac4_deux_rattachements_successifs(pool: MySqlPool) {
    let ctx = monter(&pool).await;
    let bank = ctx.banque(IBAN_A).await;
    let e = ctx.depense("2026-06-18", "250.00").await;
    let t1 = ctx.transaction(bank, "-250.00", "2026-06-18").await;
    let t2 = ctx.transaction(bank, "-250.00", "2026-06-19").await;
    let (s, b) = ctx.rattacher(t1, e).await;
    assert_eq!(s, 200, "{b}");
    let traces = common::audit_count(&pool).await;
    let (s, b) = ctx.rattacher(t2, e).await;
    assert_non_rattachable(s, &b);
    assert_eq!(b["error"]["details"]["bankTransactionId"], t2);
    assert_eq!(b["error"]["details"]["journalEntryId"], e);
    assert_eq!(common::audit_count(&pool).await, traces);
    assert_eq!(ctx.etat(t2).await, ("pending".to_string(), None));
    assert_eq!(ctx.liens_vers(e).await, 1);
}

/// AC-4 — deux candidates, le corps désigne la **seconde** : c'est elle qui est
/// rattachée (tue M17).
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn ac4_la_candidate_designee_est_rattachee(pool: MySqlPool) {
    let ctx = monter(&pool).await;
    let bank = ctx.banque(IBAN_A).await;
    let premiere = ctx.depense("2026-06-18", "250.00").await;
    let seconde = ctx.depense("2026-06-10", "250.00").await;
    let t = ctx.transaction(bank, "-250.00", "2026-06-18").await;
    assert_eq!(ctx.ids_candidates(t).await, vec![premiere, seconde]);
    let (s, b) = ctx.rattacher(t, seconde).await;
    assert_eq!(s, 200, "{b}");
    assert_eq!(ctx.etat(t).await, ("reconciled".to_string(), Some(seconde)));
}

/// AC-6 — chaque refus, avec son code, sans rien écrire : corps invalide ;
/// transaction absente ; transaction rapprochée ; transaction en EUR ou nulle
/// (condition 1, avant toute lecture de l'écriture) ; écriture absente ;
/// écriture non candidate (montant, condition 3).
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn ac6_les_refus(pool: MySqlPool) {
    let ctx = monter(&pool).await;
    let bank = ctx.banque(IBAN_A).await;
    let e = ctx.depense("2026-06-18", "250.00").await;
    let autre = ctx.depense("2026-06-18", "99.00").await;
    let t = ctx.transaction(bank, "-250.00", "2026-06-18").await;
    let traces = common::audit_count(&pool).await;

    for corps in [
        json!({}),
        json!({ "journalEntryId": 0 }),
        json!({ "journalEntryId": -3 }),
    ] {
        let (s, b) = ctx
            .post(
                &format!("/api/v1/reconciliation/transactions/{t}/link-entry"),
                corps,
            )
            .await;
        assert_refus(s, &b, 400, "VALIDATION_ERROR");
    }
    let (s, b) = ctx.rattacher(i64::MAX, e).await;
    assert_refus(s, &b, 404, "NOT_FOUND");
    let (s, b) = ctx.candidates(i64::MAX).await;
    assert_refus(s, &b, 404, "NOT_FOUND");
    let (s, b) = ctx.rattacher(t, i64::MAX).await;
    assert_refus(s, &b, 404, "NOT_FOUND");
    let (s, b) = ctx.rattacher(t, autre).await;
    assert_non_rattachable(s, &b);
    for (montant, devise) in [("-250.00", "EUR"), ("0.00", "CHF")] {
        let tx = ctx
            .transaction_en(bank, montant, devise, "2026-06-18")
            .await;
        let (s, b) = ctx.candidates(tx).await;
        assert_eq!(s, 200, "{b}");
        assert_eq!(b["candidates"], json!([]));
        // Condition 1 avant la lecture de l'écriture : même une écriture
        // inexistante rend « non rattachable ».
        let (s, b) = ctx.rattacher(tx, i64::MAX).await;
        assert_non_rattachable(s, &b);
    }
    assert_eq!(common::audit_count(&pool).await, traces, "rien d'écrit");
    assert_eq!(ctx.etat(t).await, ("pending".to_string(), None));

    let (s, b) = ctx.rattacher(t, e).await;
    assert_eq!(s, 200, "{b}");
    let (s, b) = ctx.candidates(t).await;
    assert_refus(s, &b, 404, "RECONCILIATION_TRANSACTION_NOT_PENDING");
    let (s, b) = ctx.rattacher(t, e).await;
    assert_refus(s, &b, 404, "RECONCILIATION_TRANSACTION_NOT_PENDING");
}

/// AC-6 — le pré-vol de la 29-8, réutilisé : compte non lié `412`, archivé
/// `404 BANK_IMPORT_BANK_ACCOUNT_NOT_FOUND`, clos `409 BANK_ACCOUNT_CLOSED` —
/// au `GET` comme au `POST` (états forgés par SQL).
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn ac6_le_pre_vol(pool: MySqlPool) {
    let ctx = monter(&pool).await;
    let bank = ctx.banque(IBAN_A).await;
    let e = ctx.depense("2026-06-18", "250.00").await;
    let t = ctx.transaction(bank, "-250.00", "2026-06-18").await;
    sqlx::query("UPDATE bank_accounts SET journal_account_id = NULL WHERE id = ?")
        .bind(bank)
        .execute(&pool)
        .await
        .unwrap();
    let (s, b) = ctx.candidates(t).await;
    assert_refus(s, &b, 412, "BANK_ACCOUNT_NOT_CONFIGURED");
    let (s, b) = ctx.rattacher(t, e).await;
    assert_refus(s, &b, 412, "BANK_ACCOUNT_NOT_CONFIGURED");
    sqlx::query("UPDATE bank_accounts SET journal_account_id = ? WHERE id = ?")
        .bind(ctx.ledger)
        .bind(bank)
        .execute(&pool)
        .await
        .unwrap();
    ctx.clore_banque_sql(bank).await;
    let (s, b) = ctx.candidates(t).await;
    assert_refus(s, &b, 409, "BANK_ACCOUNT_CLOSED");
    let (s, b) = ctx.rattacher(t, e).await;
    assert_refus(s, &b, 409, "BANK_ACCOUNT_CLOSED");
    sqlx::query("UPDATE bank_accounts SET archived = TRUE WHERE id = ?")
        .bind(bank)
        .execute(&pool)
        .await
        .unwrap();
    let (s, b) = ctx.candidates(t).await;
    assert_refus(s, &b, 404, "BANK_IMPORT_BANK_ACCOUNT_NOT_FOUND");
    let (s, b) = ctx.rattacher(t, e).await;
    assert_refus(s, &b, 404, "BANK_IMPORT_BANK_ACCOUNT_NOT_FOUND");
    assert_eq!(ctx.etat(t).await, ("pending".to_string(), None));
}

/// AC-4 — compte bancaire clos **dans** la transaction : une connexion tient
/// le verrou nommé du compte, le `POST` est vu bloqué dessus, la clôture est
/// posée par SQL, le verrou relâché : la garde de l'étape 4.3 rend `409
/// BANK_ACCOUNT_CLOSED` (tue M28). ⚠️ État forgé, comme T13 (b) de la 29-8.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn ac4_compte_clos_pendant_l_attente_du_verrou(pool: MySqlPool) {
    let ctx = monter(&pool).await;
    let bank = ctx.banque(IBAN_A).await;
    let e = ctx.depense("2026-06-18", "250.00").await;
    let t = ctx.transaction(bank, "-250.00", "2026-06-18").await;
    let db_name: String = sqlx::query_scalar("SELECT COALESCE(DATABASE(), '')")
        .fetch_one(&pool)
        .await
        .unwrap();
    let nom = format!("reconcile:{db_name}:{}:{bank}", ctx.company_id());
    let mut tenue = pool.acquire().await.unwrap();
    let pris: Option<i32> = sqlx::query_scalar("SELECT GET_LOCK(?, 60)")
        .bind(&nom)
        .fetch_one(&mut *tenue)
        .await
        .unwrap();
    assert_eq!(pris, Some(1));
    let client = ctx.app.client.clone();
    let url = ctx.app.url(&format!(
        "/api/v1/reconciliation/transactions/{t}/link-entry"
    ));
    let jwt = ctx.jwt.clone();
    let requete = tokio::spawn(async move {
        let r = client
            .post(url)
            .bearer_auth(jwt)
            .json(&json!({ "journalEntryId": e }))
            .send()
            .await
            .unwrap();
        let s = r.status().as_u16();
        (s, r.json::<Value>().await.unwrap_or(Value::Null))
    });
    let vu = kesh_db::test_fixtures::attendre_une_requete_en_cours(&pool, &["GET_LOCK"], || {
        requete.is_finished()
    })
    .await;
    assert!(vu, "le POST doit attendre le verrou du compte");
    ctx.clore_banque_sql(bank).await;
    let _: Option<i32> = sqlx::query_scalar("SELECT RELEASE_LOCK(?)")
        .bind(&nom)
        .fetch_one(&mut *tenue)
        .await
        .unwrap();
    let (s, b) = requete.await.unwrap();
    assert_refus(s, &b, 409, "BANK_ACCOUNT_CLOSED");
    assert_eq!(ctx.etat(t).await, ("pending".to_string(), None));
}

/// AC-8 — rôles et clés : Consultation `403` aux deux routes ; clé `read`
/// `403 API_KEY_READ_ONLY` au `POST` (la liste lui reste ouverte) ; clé
/// `read-write` admise et nommée à l'audit.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn ac8_roles_et_cles(pool: MySqlPool) {
    let ctx = monter(&pool).await;
    let bank = ctx.banque(IBAN_A).await;
    let e = ctx.depense("2026-06-18", "250.00").await;
    let t = ctx.transaction(bank, "-250.00", "2026-06-18").await;
    let lecteur = users::create(
        &pool,
        NewUser {
            username: "lecteur".into(),
            password_hash: hash_password("password123").unwrap(),
            role: Role::Consultation,
            active: true,
            company_id: ctx.company_id(),
            email: None,
        },
    )
    .await
    .unwrap()
    .id;
    let jwt_lecteur = forge_jwt(lecteur, "Consultation", ctx.company_id());
    let (s, _) = ctx
        .req_as(
            &jwt_lecteur,
            reqwest::Method::GET,
            &format!("/api/v1/reconciliation/transactions/{t}/entry-candidates"),
            None,
        )
        .await;
    assert_eq!(s, 403);
    let (s, _) = ctx
        .req_as(
            &jwt_lecteur,
            reqwest::Method::POST,
            &format!("/api/v1/reconciliation/transactions/{t}/link-entry"),
            Some(json!({ "journalEntryId": e })),
        )
        .await;
    assert_eq!(s, 403);

    let (_, cle_ro) =
        common::create_key_via_http(&ctx.app.client, &ctx.app.base_url, &ctx.jwt, "ro", "read")
            .await;
    let (s, b) = ctx
        .req_as(
            &cle_ro,
            reqwest::Method::POST,
            &format!("/api/v1/reconciliation/transactions/{t}/link-entry"),
            Some(json!({ "journalEntryId": e })),
        )
        .await;
    assert_refus(s, &b, 403, "API_KEY_READ_ONLY");
    let (s, b) = ctx
        .req_as(
            &cle_ro,
            reqwest::Method::GET,
            &format!("/api/v1/reconciliation/transactions/{t}/entry-candidates"),
            None,
        )
        .await;
    assert_eq!(s, 200, "{b}");
    assert_eq!(ctx.etat(t).await, ("pending".to_string(), None));

    let (cle_id, cle_rw) = common::create_key_via_http(
        &ctx.app.client,
        &ctx.app.base_url,
        &ctx.jwt,
        "rw",
        "read-write",
    )
    .await;
    let (s, b) = ctx
        .req_as(
            &cle_rw,
            reqwest::Method::POST,
            &format!("/api/v1/reconciliation/transactions/{t}/link-entry"),
            Some(json!({ "journalEntryId": e })),
        )
        .await;
    assert_eq!(s, 200, "{b}");
    let (_, api_key, _, _) =
        common::audit_actor(&pool, "bank_transaction", t, "reconciliation.entry_linked").await;
    assert_eq!(api_key, Some(cle_id), "la clé est nommée à l'audit");
}

/// AC-8 — étanchéité : la transaction d'une société vue de l'autre `404` ;
/// l'écriture de la première désignée depuis une transaction de l'autre `404`
/// (écriture d'une autre société, AC-3 étape 4.2). Rien d'écrit.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn ac8_une_autre_societe_ne_voit_rien(pool: MySqlPool) {
    let ctx = monter(&pool).await;
    let bank = ctx.banque(IBAN_A).await;
    let e = ctx.depense("2026-06-18", "250.00").await;
    let t = ctx.transaction(bank, "-250.00", "2026-06-18").await;
    let (autre, autre_user, autre_jwt) = autre_societe(&pool, "autre_admin").await;
    let autre_ledger = compte_du_grand_livre(&pool, autre, "1020", "Banque").await;
    let autre_bank = bank_accounts::create(
        &pool,
        NewBankAccount {
            company_id: autre,
            bank_name: "BCV".into(),
            iban: IBAN_C.into(),
            qr_iban: None,
            is_primary: true,
        },
    )
    .await
    .unwrap()
    .id;
    sqlx::query("UPDATE bank_accounts SET journal_account_id = ? WHERE id = ?")
        .bind(autre_ledger)
        .bind(autre_bank)
        .execute(&pool)
        .await
        .unwrap();
    let mut tr = pool.begin().await.unwrap();
    let (_, autres) = bank_imports::create_with_transactions(
        &mut tr,
        NewBankImport {
            company_id: autre,
            bank_account_id: autre_bank,
            filename: "autre.xml".into(),
            file_hash: hash64("autre-societe-entry"),
            source_format: BankImportSourceFormat::Camt053V04,
            statement_id: Some("AUTRE-ENTRY".into()),
            period_from: d("2026-06-18"),
            period_to: d("2026-06-18"),
            opening_balance: None,
            closing_balance: None,
            transaction_count: 1,
            imported_by_user_id: autre_user,
        },
        vec![NewBankTransaction {
            company_id: autre,
            bank_account_id: autre_bank,
            booking_date: d("2026-06-18"),
            value_date: Some(d("2026-06-18")),
            amount: dec("-250.00"),
            currency: "CHF".into(),
            reference: None,
            details: "Autre".into(),
            end_to_end_id: None,
            transaction_id: None,
            counterparty_iban: None,
            counterparty_name: None,
        }],
    )
    .await
    .unwrap();
    tr.commit().await.unwrap();
    let autre_tx = autres[0].id;

    let traces = common::audit_count(&pool).await;
    let (s, b) = ctx
        .req_as(
            &autre_jwt,
            reqwest::Method::GET,
            &format!("/api/v1/reconciliation/transactions/{t}/entry-candidates"),
            None,
        )
        .await;
    assert_refus(s, &b, 404, "NOT_FOUND");
    let (s, b) = ctx
        .req_as(
            &autre_jwt,
            reqwest::Method::POST,
            &format!("/api/v1/reconciliation/transactions/{t}/link-entry"),
            Some(json!({ "journalEntryId": e })),
        )
        .await;
    assert_refus(s, &b, 404, "NOT_FOUND");
    let (s, b) = ctx
        .req_as(
            &autre_jwt,
            reqwest::Method::POST,
            &format!("/api/v1/reconciliation/transactions/{autre_tx}/link-entry"),
            Some(json!({ "journalEntryId": e })),
        )
        .await;
    assert_refus(s, &b, 404, "NOT_FOUND");
    assert_eq!(common::audit_count(&pool).await, traces, "rien d'écrit");
    assert_eq!(ctx.etat(t).await, ("pending".to_string(), None));
    assert_eq!(ctx.etat(autre_tx).await, ("pending".to_string(), None));
}

// ============================================================
// AC-5 — le motif d'annulation, au moment de la lecture
// ============================================================

/// AC-5 — une écriture d'exercice **clôturé** est candidate, avec
/// `cancelBlockedBy = "FISCAL_YEAR_CLOSED"` ; une écriture dont un compte est
/// **archivé**, `"ACCOUNT_ARCHIVED"` et le numéro du compte ; une écriture
/// d'exercice ouvert, `null` (tue M22, M26). États forgés par SQL.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn ac5_le_motif_d_annulation(pool: MySqlPool) {
    let ctx = monter(&pool).await;
    ctx.scinder_l_exercice().await;
    let bank = ctx.banque(IBAN_A).await;
    let close = ctx.depense("2025-12-20", "61.00").await;
    let ouverte = ctx.depense("2026-01-04", "61.00").await;
    let archivee = ctx
        .ecriture(
            "2026-01-05",
            "Sur un compte bientôt archivé",
            &[
                (ctx.compte("3000"), "62.00", "0"),
                (ctx.ledger, "0", "62.00"),
            ],
        )
        .await;
    ctx.clore_exercice_sql(ctx.seeded.fiscal_year_id).await;
    sqlx::query("UPDATE accounts SET active = FALSE WHERE id = ?")
        .bind(ctx.compte("3000"))
        .execute(&pool)
        .await
        .unwrap();

    let t = ctx.transaction(bank, "-61.00", "2026-01-05").await;
    let (s, b) = ctx.candidates(t).await;
    assert_eq!(s, 200, "{b}");
    let par_id = |id: i64| {
        b["candidates"]
            .as_array()
            .unwrap()
            .iter()
            .find(|c| c["journalEntryId"] == id)
            .cloned()
            .unwrap_or_else(|| panic!("{id} absente : {b}"))
    };
    assert_eq!(par_id(close)["cancelBlockedBy"], "FISCAL_YEAR_CLOSED");
    assert!(par_id(ouverte)["cancelBlockedBy"].is_null());

    let t2 = ctx.transaction(bank, "-62.00", "2026-01-05").await;
    let (s, b) = ctx.candidates(t2).await;
    assert_eq!(s, 200, "{b}");
    assert_eq!(b["candidates"][0]["journalEntryId"], archivee);
    assert_eq!(b["candidates"][0]["cancelBlockedBy"], "ACCOUNT_ARCHIVED");
    assert_eq!(b["candidates"][0]["cancelBlockedLabel"], "3000");
}

/// AC-8.5 — une écriture saisie d'un exercice **clôturé**, rattachée :
/// *Annuler le rapprochement* est refusé `409 FISCAL_YEAR_CLOSED`, rien
/// d'écrit, et c'est le motif que la liste annonçait ; puis l'exercice rouvert
/// par un Administrateur, l'annulation passe — le motif était **daté**.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn ac8_5_un_motif_date_et_non_definitif(pool: MySqlPool) {
    let ctx = monter(&pool).await;
    ctx.scinder_l_exercice().await;
    let bank = ctx.banque(IBAN_A).await;
    let e = ctx.depense("2025-12-20", "61.00").await;
    ctx.clore_exercice_sql(ctx.seeded.fiscal_year_id).await;
    let t = ctx.transaction(bank, "-61.00", "2026-01-05").await;
    let (s, b) = ctx.candidates(t).await;
    assert_eq!(s, 200, "{b}");
    let annonce = b["candidates"][0]["cancelBlockedBy"].clone();
    assert_eq!(annonce, "FISCAL_YEAR_CLOSED");
    let (s, b) = ctx.rattacher(t, e).await;
    assert_eq!(s, 200, "{b}");

    let ecritures = ctx.ecritures().await;
    let (s, b) = ctx.annuler_rapprochement(t).await;
    assert_refus(s, &b, 409, "FISCAL_YEAR_CLOSED");
    assert_eq!(b["error"]["code"], annonce, "même fonction, même motif");
    assert_eq!(ctx.ecritures().await, ecritures, "rien d'écrit");
    assert_eq!(ctx.etat(t).await, ("reconciled".to_string(), Some(e)));

    let (s, b) = ctx
        .post(
            &format!("/api/v1/fiscal-years/{}/reopen", ctx.seeded.fiscal_year_id),
            json!({ "motif": "Correction d'un rapprochement" }),
        )
        .await;
    assert_eq!(s, 200, "réouverture : {b}");
    let (s, b) = ctx.annuler_rapprochement(t).await;
    assert_eq!(s, 200, "{b}");
    assert_eq!(ctx.etat(t).await, ("pending".to_string(), None));
}

// ============================================================
// Courses — T-race-1 à 3
// ============================================================

/// T-race-1 — deux `link-entry` simultanés de la même écriture à deux
/// transactions du même compte : exactement un `200`, l'autre `409
/// RECONCILIATION_ENTRY_NOT_LINKABLE`, un seul lien.
///
/// ⚠️ **Ce qu'une course gagnée ne prouve pas** (réserve de `t16`/`t17` de la
/// 29-8, C-29-8-25) : sans chevauchement réel, le second voit le premier par
/// simple succession ; et le verrou nommé du compte sérialise déjà deux gestes
/// du même compte — ce test ne prouve ni l'attente sur la ligne de l'écriture
/// ni l'ordre d'acquisition. C'est T-race-3 qui force l'entrelacement.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn t_race_1_double_rattachement(pool: MySqlPool) {
    let ctx = monter(&pool).await;
    let bank = ctx.banque(IBAN_A).await;
    let e = ctx.depense("2026-06-18", "250.00").await;
    let t1 = ctx.transaction(bank, "-250.00", "2026-06-18").await;
    let t2 = ctx.transaction(bank, "-250.00", "2026-06-19").await;
    let ((s1, b1), (s2, b2)) = tokio::join!(ctx.rattacher(t1, e), ctx.rattacher(t2, e));
    let (gagnant, perdant, bp) = match (s1, s2) {
        (200, _) => (t1, t2, (s2, b2)),
        (_, 200) => (t2, t1, (s1, b1)),
        _ => panic!("aucun 200 : {s1} {b1} / {s2} {b2}"),
    };
    assert_non_rattachable(bp.0, &bp.1);
    assert_eq!(ctx.etat(gagnant).await, ("reconciled".to_string(), Some(e)));
    assert_eq!(ctx.etat(perdant).await, ("pending".to_string(), None));
    assert_eq!(ctx.liens_vers(e).await, 1);
}

/// T-race-2 — `link-entry` contre `POST /journal-entries/{id}/reverse` sur la
/// même écriture saisie : exactement l'un réussit ; rattachée et
/// contre-passation refusée `409 MATCHED_BANK_TRANSACTION`, ou contre-passée et
/// rattachement refusé `409 RECONCILIATION_ENTRY_NOT_LINKABLE`. ⚠️ Même
/// réserve que T-race-1 : une course gagnée ne prouve pas l'attente.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn t_race_2_rattachement_et_contre_passation(pool: MySqlPool) {
    let ctx = monter(&pool).await;
    let bank = ctx.banque(IBAN_A).await;
    let e = ctx.depense("2026-06-18", "250.00").await;
    let t = ctx.transaction(bank, "-250.00", "2026-06-18").await;
    let ((sl, bl), (sr, br)) = tokio::join!(ctx.rattacher(t, e), ctx.contre_passer(e));
    match (sl, sr) {
        (200, _) => {
            assert_refus(sr, &br, 409, "MATCHED_BANK_TRANSACTION");
            assert_eq!(ctx.etat(t).await, ("reconciled".to_string(), Some(e)));
            assert_eq!(ctx.contre_passations_de(e).await, 0);
        }
        (_, 201) => {
            assert_non_rattachable(sl, &bl);
            assert_eq!(ctx.etat(t).await, ("pending".to_string(), None));
            assert_eq!(ctx.contre_passations_de(e).await, 1);
        }
        _ => panic!("aucun des deux n'a réussi : {sl} {bl} / {sr} {br}"),
    }
}

/// T-race-3 — **entrelacement forcé** (patron `t17bis` de la 29-8) : une
/// connexion `tenue` prend la ligne de l'écriture `FOR UPDATE` ; le `POST` de
/// `t2` est **vu en attente** sur ce verrou (sonde `PROCESSLIST`, jamais
/// `INNODB_TRX`) ; `tenue` lie `t1` à l'écriture et **commite**. Le `POST` doit
/// rendre `409 RECONCILIATION_ENTRY_NOT_LINKABLE` : sa vue naît **après**
/// l'attente (étape 4.3), et l'étape 4.4 voit le lien commité. Un seul lien.
///
/// Tue M10 (la garde lue avant le verrou de l'écriture fait naître la vue
/// trop tôt : `200`, deux liens) et M20 (sans `FOR UPDATE`, le `POST`
/// n'attend jamais : la sonde ne le voit pas). L'invariant « une écriture, une
/// transaction » n'a pas de contrainte `UNIQUE` : ce test est sa garde
/// dynamique.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn t_race_3_le_second_attend_le_verrou_de_l_ecriture(pool: MySqlPool) {
    let ctx = monter(&pool).await;
    let bank = ctx.banque(IBAN_A).await;
    let e = ctx.depense("2026-06-18", "250.00").await;
    let t1 = ctx.transaction(bank, "-250.00", "2026-06-18").await;
    let t2 = ctx.transaction(bank, "-250.00", "2026-06-19").await;

    let mut tenue = pool.begin().await.unwrap();
    let pris: Option<i64> =
        sqlx::query_scalar("SELECT id FROM journal_entries WHERE id = ? FOR UPDATE")
            .bind(e)
            .fetch_optional(&mut *tenue)
            .await
            .unwrap();
    assert_eq!(pris, Some(e));

    let client = ctx.app.client.clone();
    let url = ctx.app.url(&format!(
        "/api/v1/reconciliation/transactions/{t2}/link-entry"
    ));
    let jwt = ctx.jwt.clone();
    let requete = tokio::spawn(async move {
        let r = client
            .post(url)
            .bearer_auth(jwt)
            .json(&json!({ "journalEntryId": e }))
            .send()
            .await
            .unwrap();
        let st = r.status().as_u16();
        (st, r.json::<Value>().await.unwrap_or(Value::Null))
    });
    let vu = kesh_db::test_fixtures::attendre_une_requete_en_cours(
        &pool,
        &["journal_entries", "FOR UPDATE"],
        || requete.is_finished(),
    )
    .await;
    if !vu {
        let (st, b) = requete.await.unwrap();
        panic!("le POST n'a jamais attendu le verrou de l'écriture : rendu {st} {b}");
    }
    let fait = sqlx::query(
        "UPDATE bank_transactions SET status = 'reconciled', matched_entry_id = ? WHERE id = ?",
    )
    .bind(e)
    .bind(t1)
    .execute(&mut *tenue)
    .await
    .unwrap();
    assert_eq!(fait.rows_affected(), 1);
    tenue.commit().await.unwrap();

    let (st, b) = requete.await.unwrap();
    assert_non_rattachable(st, &b);
    assert_eq!(ctx.etat(t2).await, ("pending".to_string(), None));
    assert_eq!(ctx.liens_vers(e).await, 1);
}

// ============================================================
// AC-22 — les données existantes ne bougent pas à la lecture
// ============================================================

/// AC-22 — fixture « base 0.16.x » : une transaction **rejetée** et une facture
/// fournisseur payée depuis la fiche, par les fonctions de la 0.16. La page de
/// propositions et la liste des candidates ne changent **aucune** ligne ; le
/// rattachement ne se fait que par le `POST`.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn ac22_la_lecture_ne_change_rien(pool: MySqlPool) {
    let ctx = monter(&pool).await;
    let bank = ctx.banque(IBAN_A).await;
    let fi = ctx.facture_fournisseur("FF-1", "100.00").await;
    let e = ctx.payer_fournisseur(fi, bank, "2026-06-20").await;
    let t = ctx.transaction(bank, "-100.00", "2026-06-21").await;
    let (s, b) = ctx
        .post(
            "/api/v1/reconciliation/reject",
            json!({ "bankAccountId": bank, "bankTransactionIds": [t] }),
        )
        .await;
    assert_eq!(s, 200, "{b}");

    type Ligne = (
        String,
        Option<i64>,
        Option<chrono::NaiveDateTime>,
        i32,
        chrono::NaiveDateTime,
    );
    let lire_t = || async {
        sqlx::query_as::<_, Ligne>(
            "SELECT status, matched_entry_id, auto_match_rejected_at, version, updated_at \
             FROM bank_transactions WHERE id = ?",
        )
        .bind(t)
        .fetch_one(&pool)
        .await
        .unwrap()
    };
    let lire_f = || async {
        sqlx::query_as::<_, (String, Option<i64>, i32)>(
            "SELECT status, settlement_journal_entry_id, version FROM supplier_invoices WHERE id = ?",
        )
        .bind(fi)
        .fetch_one(&pool)
        .await
        .unwrap()
    };
    let (avant_t, avant_f, avant_audit) = (
        lire_t().await,
        lire_f().await,
        common::audit_count(&pool).await,
    );

    let (s, b) = ctx
        .get(&format!(
            "/api/v1/reconciliation/proposals?bankAccountId={bank}"
        ))
        .await;
    assert_eq!(s, 200, "{b}");
    let (s, b) = ctx.candidates(t).await;
    assert_eq!(s, 200, "{b}");
    assert_eq!(b["candidates"][0]["journalEntryId"], e);

    assert_eq!(lire_t().await, avant_t, "transaction inchangée");
    assert_eq!(lire_f().await, avant_f, "facture inchangée");
    assert_eq!(
        common::audit_count(&pool).await,
        avant_audit,
        "aucune trace"
    );
    assert_eq!(ctx.etat(t).await, ("pending".to_string(), None));

    let (s, b) = ctx.rattacher(t, e).await;
    assert_eq!(s, 200, "{b}");
    assert_eq!(ctx.etat(t).await, ("reconciled".to_string(), Some(e)));
}

// ============================================================
// AC-5 — la mesure de coût (ignorée par défaut)
// ============================================================

/// AC-5, mesure — `GET …/entry-candidates` rend **50** candidates en moins
/// d'une seconde, trois mesures, pour chacun des deux sens :
/// - un **débit** : 25 écritures saisies (bras « écriture propre ») et 25
///   règlements fournisseur payés depuis la fiche (bras le plus coûteux) ;
/// - un **crédit** : 25 écritures saisies et 25 règlements client par compte
///   interne sur le compte de la banque.
///
/// ⚠️ Les trois sortes ne peuvent pas être candidates d'**une même**
/// transaction : un règlement fournisseur débite la banque, un règlement client
/// la crédite — d'où deux montages. ⚠️ **Borne basse** : la base de test est
/// en tmpfs, la mesure ne dit rien du NAS. Montage synthétique, base
/// éphémère du serveur de dev. Ignorée par défaut (une mesure de durée n'est
/// pas un garde de CI) : `cargo nextest run --run-ignored ignored-only -E
/// 'test(mesure_cout)'`. Les durées s'impriment (`--no-capture`).
#[sqlx::test(migrations = "../kesh-db/test-schema")]
#[ignore = "mesure de coût, AC-5 de la fiche v017-3a — à lancer à la main"]
async fn mesure_cout_50_candidates(pool: MySqlPool) {
    let ctx = monter(&pool).await;
    let bank = ctx.banque(IBAN_A).await;
    for k in 0..25u64 {
        let jour = (d("2026-03-01") + chrono::Days::new(k)).to_string();
        ctx.depense(&jour, "100.00").await;
        let fi = ctx.facture_fournisseur(&format!("FF-{k}"), "100.00").await;
        ctx.payer_fournisseur(fi, bank, &jour).await;
        ctx.recette(&jour, "108.10").await;
        let f = ctx.facture("2026-02-01").await;
        ctx.regler_interne(f, ctx.ledger, "108.10", &jour).await;
    }
    for (sens, montant) in [("débit", "-100.00"), ("crédit", "108.10")] {
        let t = ctx.transaction(bank, montant, "2026-04-15").await;
        for essai in 1..=3 {
            let debut = std::time::Instant::now();
            let (s, b) = ctx.candidates(t).await;
            let duree = debut.elapsed();
            assert_eq!(s, 200, "{b}");
            assert_eq!(b["candidates"].as_array().unwrap().len(), 50, "{sens}");
            println!("MESURE {sens} essai {essai} : {} ms", duree.as_millis());
            assert!(
                duree < std::time::Duration::from_secs(1),
                "{sens} : {duree:?}"
            );
        }
    }
}

// ============================================================
// Story 34-4 (#828), AC-1 à AC-3 — le champ `alreadyBooked` des propositions
// ============================================================

impl Ctx {
    /// `alreadyBooked` de la transaction `tx` dans `GET /reconciliation/proposals`
    /// du compte `bank` (`Value::Null` pour un champ nul ; panique s'il manque).
    async fn deja_comptabilisee(&self, bank: i64, tx: i64) -> Value {
        let (s, b) = self
            .get(&format!(
                "/api/v1/reconciliation/proposals?bankAccountId={bank}&limit=500"
            ))
            .await;
        assert_eq!(s, 200, "propositions : {b}");
        let p = b["proposals"]
            .as_array()
            .unwrap()
            .iter()
            .find(|p| p["bankTransactionId"].as_i64() == Some(tx))
            .unwrap_or_else(|| panic!("transaction {tx} absente : {b}"))
            .clone();
        p.as_object()
            .unwrap()
            .get("alreadyBooked")
            .unwrap_or_else(|| panic!("champ alreadyBooked absent : {p}"))
            .clone()
    }

    /// Les longueurs des deux listes du dialogue, par leurs routes :
    /// `(link-candidates, entry-candidates)`.
    async fn longueurs_du_dialogue(&self, tx: i64) -> (i64, i64) {
        let (s, b) = self
            .get(&format!(
                "/api/v1/reconciliation/transactions/{tx}/link-candidates"
            ))
            .await;
        assert_eq!(s, 200, "link-candidates : {b}");
        let reglements = b["candidates"].as_array().unwrap().len() as i64;
        (reglements, self.ids_candidates(tx).await.len() as i64)
    }

    /// Asserte que le champ égale les longueurs des listes du dialogue (AC-1),
    /// et rend `(settlementCount, entryCount, matchedElsewhereCount)`.
    async fn champ_egal_au_dialogue(&self, bank: i64, tx: i64) -> (i64, i64, i64) {
        let champ = self.deja_comptabilisee(bank, tx).await;
        let n = |k: &str| {
            champ[k]
                .as_i64()
                .unwrap_or_else(|| panic!("{k} absent : {champ}"))
        };
        let (reglements, ecritures) = self.longueurs_du_dialogue(tx).await;
        assert_eq!(
            n("settlementCount"),
            reglements,
            "settlementCount : {champ}"
        );
        assert_eq!(n("entryCount"), ecritures, "entryCount : {champ}");
        (
            n("settlementCount"),
            n("entryCount"),
            n("matchedElsewhereCount"),
        )
    }
}

/// AC-1 — le champ égale la longueur des deux listes du dialogue, mêmes
/// fonctions, même tolérance : règlement client au montant exact, règlement
/// inférieur de 5 centimes (compté), de 6 centimes (non compté), écriture
/// saisie au journal (crédit et débit), et rien (tue M1, M2).
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn s34_4_ac1_le_champ_egale_les_listes_du_dialogue(pool: MySqlPool) {
    let ctx = monter(&pool).await;
    let bank = ctx.banque(IBAN_A).await;

    let f1 = ctx.facture("2026-03-01").await;
    ctx.regler_virement(f1, bank, "108.10", "2026-03-10").await;
    let exact = ctx.transaction(bank, "108.10", "2026-03-10").await;

    let f2 = ctx.facture("2026-03-01").await;
    ctx.regler_virement(f2, bank, "103.05", "2026-03-10").await;
    let moins5 = ctx.transaction(bank, "103.10", "2026-03-10").await;

    let f3 = ctx.facture("2026-03-01").await;
    ctx.regler_virement(f3, bank, "104.04", "2026-03-10").await;
    let moins6 = ctx.transaction(bank, "104.10", "2026-03-10").await;

    ctx.recette("2026-03-12", "250.00").await;
    let ecriture_credit = ctx.transaction(bank, "250.00", "2026-03-12").await;
    ctx.depense("2026-03-13", "60.00").await;
    let ecriture_debit = ctx.transaction(bank, "-60.00", "2026-03-13").await;

    let rien = ctx.transaction(bank, "777.77", "2026-03-14").await;

    assert_eq!(ctx.champ_egal_au_dialogue(bank, exact).await, (1, 0, 0));
    assert_eq!(ctx.champ_egal_au_dialogue(bank, moins5).await, (1, 0, 0));
    assert_eq!(ctx.champ_egal_au_dialogue(bank, moins6).await, (0, 0, 0));
    assert_eq!(
        ctx.champ_egal_au_dialogue(bank, ecriture_credit).await,
        (0, 1, 0)
    );
    assert_eq!(
        ctx.champ_egal_au_dialogue(bank, ecriture_debit).await,
        (0, 1, 0)
    );
    assert_eq!(ctx.champ_egal_au_dialogue(bank, rien).await, (0, 0, 0));
}

/// AC-1 — `null` quand la transaction n'a rien à quoi se rattacher (autre
/// devise, montant nul) ou que le compte bancaire n'a pas de compte du grand
/// livre ; jamais `null` pour une transaction rattachable (zéros compris).
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn s34_4_ac1_null_hors_chf_montant_nul_ou_sans_grand_livre(pool: MySqlPool) {
    let ctx = monter(&pool).await;
    let bank = ctx.banque(IBAN_A).await;
    let eur = ctx.transaction_en(bank, "20.00", "EUR", "2026-03-10").await;
    let nul = ctx.transaction(bank, "0.00", "2026-03-10").await;
    let chf = ctx.transaction(bank, "20.00", "2026-03-10").await;
    assert_eq!(ctx.deja_comptabilisee(bank, eur).await, Value::Null);
    assert_eq!(ctx.deja_comptabilisee(bank, nul).await, Value::Null);
    assert_eq!(
        ctx.deja_comptabilisee(bank, chf).await,
        json!({ "settlementCount": 0, "entryCount": 0, "matchedElsewhereCount": 0 })
    );

    let sans_grand_livre = bank_accounts::create(
        &ctx.pool,
        NewBankAccount {
            company_id: ctx.company_id(),
            bank_name: "Postfinance".into(),
            iban: IBAN_B.into(),
            qr_iban: None,
            is_primary: false,
        },
    )
    .await
    .unwrap()
    .id;
    let t = ctx
        .transaction(sans_grand_livre, "20.00", "2026-03-10")
        .await;
    assert_eq!(
        ctx.deja_comptabilisee(sans_grand_livre, t).await,
        Value::Null
    );

    // Revue P1 (B1-3) : un compte bancaire **clos** — état forgé, un compte clos
    // ne porte en principe aucune transaction en attente — rend `null` : le
    // dialogue de rattachement y est refusé (409 `BANK_ACCOUNT_CLOSED`), un
    // avertissement qui y renverrait mentirait.
    ctx.recette("2026-03-10", "20.00").await;
    assert_eq!(
        ctx.deja_comptabilisee(bank, chf).await["entryCount"],
        1,
        "ouvert : l'écriture compte"
    );
    ctx.clore_banque_sql(bank).await;
    assert_eq!(ctx.deja_comptabilisee(bank, chf).await, Value::Null);
}

/// Revue P1 (E1-1), choix C-e344-11 — le second côté ne dépend pas du lien
/// **actuel** d'un compte bancaire à son grand livre. A et C partagent le 1020 ;
/// une recette de +400 est rapprochée du crédit de C ; C est ensuite relié au
/// 1022 (puis délié). Un crédit **distinct** de +400 sur A n'est pas l'autre
/// côté d'un virement : la transaction rapprochée sur C va dans le **même** sens.
/// Aucun conseil de rejet (`matchedElsewhereCount = 0`) ; la recette, elle, est
/// rapprochée : elle n'est pas candidate non plus. État du relien **forgé** par
/// SQL (la route l'autorise, le montage est plus court).
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn s34_4_ac2_compte_relie_ailleurs_ne_fait_pas_rejeter(pool: MySqlPool) {
    let ctx = monter(&pool).await;
    let ledger_1022 = compte_du_grand_livre(&pool, ctx.company_id(), "1022", "Banque 3").await;
    let a = ctx.banque(IBAN_A).await;
    let c = ctx.banque(IBAN_C).await;
    let credit_c = ctx.transaction(c, "400.00", "2026-05-02").await;
    let recette = ctx.recette("2026-05-02", "400.00").await;
    let (s, corps) = ctx.rattacher(credit_c, recette).await;
    assert_eq!(s, 200, "{corps}");
    let credit_a = ctx.transaction(a, "400.00", "2026-05-03").await;
    for relien in [Some(ledger_1022), None] {
        sqlx::query("UPDATE bank_accounts SET journal_account_id = ? WHERE id = ?")
            .bind(relien)
            .bind(c)
            .execute(&pool)
            .await
            .unwrap();
        assert_eq!(
            ctx.deja_comptabilisee(a, credit_a).await,
            json!({ "settlementCount": 0, "entryCount": 0, "matchedElsewhereCount": 0 }),
            "C relié à {relien:?}"
        );
    }
}

/// AC-2 — le second côté d'un virement : le compte A (1020) paie le compte B
/// (1021) ; le débit de A est rapproché d'une écriture `D 1021 / C 1020` ; le
/// crédit de B porte `matchedElsewhereCount = 1`. N'asserte que ce nombre
/// (la 34-11b rendra cette écriture candidate).
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn s34_4_ac2_le_second_cote_d_un_virement(pool: MySqlPool) {
    let ctx = monter(&pool).await;
    let ledger_b = compte_du_grand_livre(&pool, ctx.company_id(), "1021", "Banque 2").await;
    let a = ctx.banque(IBAN_A).await;
    let b = ctx.banque_liee(IBAN_B, ledger_b).await;
    let debit_a = ctx.transaction(a, "-500.00", "2026-05-02").await;
    let credit_b = ctx.transaction(b, "500.00", "2026-05-02").await;
    let virement = ctx
        .ecriture(
            "2026-05-02",
            "Virement vers l'épargne",
            &[(ledger_b, "500.00", "0"), (ctx.ledger, "0", "500.00")],
        )
        .await;
    assert_eq!(
        ctx.deja_comptabilisee(b, credit_b).await["matchedElsewhereCount"],
        0,
        "avant le rattachement du côté A"
    );
    let (s, corps) = ctx.rattacher(debit_a, virement).await;
    assert_eq!(s, 200, "{corps}");
    assert_eq!(
        ctx.deja_comptabilisee(b, credit_b).await["matchedElsewhereCount"],
        1
    );
}

/// AC-2, revue P2 (E2-4) — le second côté d'un virement vu d'un **débit** : B
/// (1021) paie A (1020) ; le crédit de A est rapproché de l'écriture
/// `D 1020 / C 1021` ; le débit de B porte `matchedElsewhereCount = 1` (la
/// règle de sens contraire vaut dans les deux sens).
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn s34_4_ac2_le_second_cote_d_un_debit(pool: MySqlPool) {
    let ctx = monter(&pool).await;
    let ledger_b = compte_du_grand_livre(&pool, ctx.company_id(), "1021", "Banque 2").await;
    let a = ctx.banque(IBAN_A).await;
    let b = ctx.banque_liee(IBAN_B, ledger_b).await;
    let credit_a = ctx.transaction(a, "300.00", "2026-05-04").await;
    let debit_b = ctx.transaction(b, "-300.00", "2026-05-04").await;
    let virement = ctx
        .ecriture(
            "2026-05-04",
            "Virement depuis l'épargne",
            &[(ctx.ledger, "300.00", "0"), (ledger_b, "0", "300.00")],
        )
        .await;
    let (s, corps) = ctx.rattacher(credit_a, virement).await;
    assert_eq!(s, 200, "{corps}");
    assert_eq!(
        ctx.deja_comptabilisee(b, debit_b).await["matchedElsewhereCount"],
        1
    );
}

/// AC-2, revue P2 (B2-1, A2-4) — une transaction rapprochée sur le **même**
/// compte bancaire ne compte jamais comme l'autre côté (tue M10, prédicat
/// `bt.bank_account_id <> ?`). État **forgé** par SQL : A, relié au 1020, a son
/// débit de −500 rapproché de `D 1021 / C 1020` ; A est ensuite relié au 1021 —
/// sur ce grand livre, l'écriture a un mouvement de +500, opposé au débit
/// rapproché ; un crédit de +500 sur A n'est pas l'autre côté de son propre
/// débit.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn s34_4_ac2_rapprochee_sur_le_meme_compte_ne_compte_pas(pool: MySqlPool) {
    let ctx = monter(&pool).await;
    let ledger_1021 = compte_du_grand_livre(&pool, ctx.company_id(), "1021", "Banque 2").await;
    let a = ctx.banque(IBAN_A).await;
    let debit_a = ctx.transaction(a, "-500.00", "2026-05-02").await;
    let ecriture = ctx
        .ecriture(
            "2026-05-02",
            "Virement vers l'épargne",
            &[(ledger_1021, "500.00", "0"), (ctx.ledger, "0", "500.00")],
        )
        .await;
    let (s, corps) = ctx.rattacher(debit_a, ecriture).await;
    assert_eq!(s, 200, "{corps}");
    sqlx::query("UPDATE bank_accounts SET journal_account_id = ? WHERE id = ?")
        .bind(ledger_1021)
        .bind(a)
        .execute(&pool)
        .await
        .unwrap();
    let credit_a = ctx.transaction(a, "500.00", "2026-05-03").await;
    assert_eq!(
        ctx.deja_comptabilisee(a, credit_a).await["matchedElsewhereCount"],
        0
    );
}

/// AC-2 — comptes partagés : deux comptes bancaires sur le **même** compte du
/// grand livre, l'un rapproché ; l'autre porte `0` (tue M3).
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn s34_4_ac2_comptes_partages_ne_comptent_pas(pool: MySqlPool) {
    let ctx = monter(&pool).await;
    let a = ctx.banque(IBAN_A).await;
    let c = ctx.banque(IBAN_C).await;
    let credit_a = ctx.transaction(a, "400.00", "2026-05-02").await;
    let credit_c = ctx.transaction(c, "400.00", "2026-05-02").await;
    let recette = ctx.recette("2026-05-02", "400.00").await;
    let (s, corps) = ctx.rattacher(credit_a, recette).await;
    assert_eq!(s, 200, "{corps}");
    assert_eq!(
        ctx.deja_comptabilisee(c, credit_c).await["matchedElsewhereCount"],
        0
    );
}

/// AC-2 — une écriture contre-passée ne compte pas (tue M4). État **forgé** par
/// SQL : une écriture rapprochée ne se contre-passe pas par l'API ; on
/// contre-passe d'abord, puis on pose le rapprochement du côté A.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn s34_4_ac2_ecriture_contre_passee_ne_compte_pas(pool: MySqlPool) {
    let ctx = monter(&pool).await;
    let ledger_b = compte_du_grand_livre(&pool, ctx.company_id(), "1021", "Banque 2").await;
    let a = ctx.banque(IBAN_A).await;
    let b = ctx.banque_liee(IBAN_B, ledger_b).await;
    let debit_a = ctx.transaction(a, "-600.00", "2026-05-02").await;
    let credit_b = ctx.transaction(b, "600.00", "2026-05-02").await;
    let virement = ctx
        .ecriture(
            "2026-05-02",
            "Virement contre-passé",
            &[(ledger_b, "600.00", "0"), (ctx.ledger, "0", "600.00")],
        )
        .await;
    let (s, corps) = ctx.contre_passer(virement).await;
    assert!(s == 200 || s == 201, "contre-passation : {s} {corps}");
    sqlx::query(
        "UPDATE bank_transactions SET status = 'reconciled', matched_entry_id = ? WHERE id = ?",
    )
    .bind(virement)
    .bind(debit_a)
    .execute(&pool)
    .await
    .unwrap();
    assert_eq!(
        ctx.deja_comptabilisee(b, credit_b).await["matchedElsewhereCount"],
        0
    );
}

/// AC-3 — la lecture n'écrit rien : la page relue deux fois rend le même champ,
/// et aucune trace d'audit, aucune écriture ne naît.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn s34_4_ac3_la_lecture_n_ecrit_rien(pool: MySqlPool) {
    let ctx = monter(&pool).await;
    let bank = ctx.banque(IBAN_A).await;
    ctx.recette("2026-03-12", "250.00").await;
    let t = ctx.transaction(bank, "250.00", "2026-03-12").await;
    let (audit, ecritures) = (common::audit_count(&pool).await, ctx.ecritures().await);
    let premier = ctx.deja_comptabilisee(bank, t).await;
    assert_eq!(ctx.deja_comptabilisee(bank, t).await, premier);
    assert_eq!(common::audit_count(&pool).await, audit);
    assert_eq!(ctx.ecritures().await, ecritures);
    assert_eq!(ctx.etat(t).await, ("pending".to_string(), None));
}

/// AC-8 — le remède : la lecture **par page**
/// (`count_already_booked_for_page`) rend, pour chacune de 50 transactions
/// variées, **les mêmes nombres** que les trois fonctions par transaction —
/// celles du dialogue (`find_link_candidates`, `find_entry_candidates`) et
/// `find_matched_elsewhere`. Variété : crédits et débits ; écriture rapprochée
/// sur ce compte, sur un autre compte dans le même sens, sur un autre compte en
/// sens contraire (virement) ; écriture contre-passée ; règlements client à 0,
/// −5 et −6 centimes ; 55 écritures, 55 règlements et 55 écritures rapprochées
/// ailleurs d'un même montant chacun (les trois plafonds de 50, revue P1
/// A1-8) ; une écriture forgée d'une **autre société** sur le compte de la
/// banque (revue P1 E1-5) ; montant sans rien ; montant nul et autre devise
/// (absents de la table).
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn s34_4_ac8_la_lecture_par_page_egale_les_fonctions_du_dialogue(pool: MySqlPool) {
    let ctx = monter(&pool).await;
    let ledger_b = compte_du_grand_livre(&pool, ctx.company_id(), "1021", "Banque 2").await;
    let a = ctx.banque(IBAN_A).await;
    let b = ctx.banque_liee(IBAN_B, ledger_b).await;
    let c = ctx.banque(IBAN_C).await;

    // Écritures non rapprochées.
    ctx.recette("2026-04-01", "100.00").await;
    ctx.recette("2026-04-02", "100.00").await;
    ctx.depense("2026-04-03", "50.00").await;
    // Contre-passée : ne compte nulle part.
    let contre = ctx.depense("2026-04-04", "75.00").await;
    let (s, corps) = ctx.contre_passer(contre).await;
    assert!(s == 200 || s == 201, "contre-passation : {s} {corps}");
    // Rapprochée sur ce compte (A) : ne compte nulle part pour A.
    let r200 = ctx.recette("2026-04-05", "200.00").await;
    let t200 = ctx.transaction(a, "200.00", "2026-04-05").await;
    assert_eq!(ctx.rattacher(t200, r200).await.0, 200);
    // Rapprochée sur B, en sens contraire (un virement) : le second côté pour A.
    let v400 = ctx
        .ecriture(
            "2026-04-06",
            "Virement depuis l'épargne",
            &[(ctx.ledger, "400.00", "0"), (ledger_b, "0", "400.00")],
        )
        .await;
    let t400b = ctx.transaction(b, "-400.00", "2026-04-06").await;
    assert_eq!(ctx.rattacher(t400b, v400).await.0, 200);
    // Rapprochée sur C, dans le même sens que la transaction de A (même grand
    // livre) : ne compte pas pour A.
    let r500 = ctx.recette("2026-04-07", "500.00").await;
    let t500c = ctx.transaction(c, "500.00", "2026-04-07").await;
    assert_eq!(ctx.rattacher(t500c, r500).await.0, 200);
    // Règlements client par virement sur A.
    let f1 = ctx.facture("2026-03-01").await;
    ctx.regler_virement(f1, a, "108.10", "2026-04-08").await;
    let f2 = ctx.facture("2026-03-01").await;
    ctx.regler_virement(f2, a, "103.05", "2026-04-08").await;
    let f3 = ctx.facture("2026-03-01").await;
    ctx.regler_virement(f3, a, "104.04", "2026-04-08").await;
    // 55 écritures d'un même montant, forgées.
    forger_historique(&ctx, 10_001, 10_055, Some("999.00")).await;
    // 55 règlements client par virement sur A, de 555.00, forgés sur la facture f1.
    forger_historique(&ctx, 20_001, 20_055, Some("555.00")).await;
    sqlx::query(
        "INSERT INTO invoice_settlements (company_id, invoice_id, journal_entry_id, amount, \
         settled_on, settlement_type, settlement_bank_account_id) \
         SELECT e.company_id, ?, e.id, 555.00, '2026-04-08', 'bank_transfer', ? \
           FROM journal_entries e \
          WHERE e.company_id = ? AND e.entry_number BETWEEN 20001 AND 20055",
    )
    .bind(f1)
    .bind(a)
    .bind(ctx.company_id())
    .execute(&pool)
    .await
    .unwrap();
    // 55 écritures de 444.00 rapprochées, en sens contraire, sur B.
    forger_historique(&ctx, 30_001, 30_055, Some("444.00")).await;
    rapprocher_ailleurs(&ctx, b, 30_001, 30_055, "TRUE").await;
    // Une autre société : une écriture forgée de 777.77 sur le compte de la
    // banque de A (état contraire aux invariants, forgé) — elle ne compte jamais.
    let (autre, _, _) = autre_societe(&pool, "admin_autre_34_4").await;
    let etrangere: i64 = sqlx::query_scalar(
        "INSERT INTO journal_entries (company_id, fiscal_year_id, entry_number, entry_date, \
         journal, description) VALUES (?, ?, 1, '2026-04-09', 'Banque', 'Autre société') RETURNING id",
    )
    .bind(autre)
    .bind(ctx.seeded.fiscal_year_id)
    .fetch_one(&pool)
    .await
    .unwrap();
    sqlx::query(
        "INSERT INTO journal_entry_lines (entry_id, account_id, line_order, debit, credit) \
         VALUES (?, ?, 1, 777.77, 0), (?, ?, 2, 0, 777.77)",
    )
    .bind(etrangere)
    .bind(ctx.ledger)
    .bind(etrangere)
    .bind(ctx.compte("3000"))
    .execute(&pool)
    .await
    .unwrap();

    let montants = [
        "100.00", "200.00", "-50.00", "-75.00", "400.00", "500.00", "108.10", "103.10", "104.10",
        "999.00", "777.77", "0.00", "-100.00", "555.00", "444.00",
    ];
    let mut ids = Vec::new();
    for k in 0..48 {
        ids.push(
            ctx.transaction(a, montants[k % montants.len()], "2026-04-15")
                .await,
        );
    }
    ids.push(ctx.transaction_en(a, "100.00", "EUR", "2026-04-15").await);
    ids.push(ctx.transaction_en(a, "400.00", "EUR", "2026-04-15").await);
    let mut transactions = Vec::new();
    for id in &ids {
        transactions.push(
            reconciliation::find_pending_by_id_for_account(&pool, ctx.company_id(), a, *id)
                .await
                .unwrap()
                .expect("transaction"),
        );
    }
    assert_eq!(transactions.len(), 50);

    let tolerance = dec("0.05");
    let page = count_already_booked_for_page(
        &pool,
        ctx.company_id(),
        a,
        ctx.ledger,
        &transactions,
        tolerance,
    )
    .await
    .unwrap();
    let (mut reglements, mut ecritures, mut ailleurs, mut absentes) = (0, 0, 0, 0);
    for t in &transactions {
        let attendu_reglements =
            find_link_candidates(&pool, ctx.company_id(), t, ctx.ledger, tolerance)
                .await
                .unwrap()
                .len();
        let attendu_ecritures = find_entry_candidates(&pool, ctx.company_id(), t, ctx.ledger)
            .await
            .unwrap()
            .len();
        let attendu_ailleurs = find_matched_elsewhere(&pool, ctx.company_id(), t, ctx.ledger)
            .await
            .unwrap()
            .len();
        match page.get(&t.id) {
            None => {
                assert!(
                    t.amount.is_zero() || t.currency != "CHF",
                    "absente de la page : {} {}",
                    t.amount,
                    t.currency
                );
                absentes += 1;
            }
            Some(n) => {
                assert_eq!(
                    (n.settlement_count, n.entry_count, n.matched_elsewhere_count),
                    (attendu_reglements, attendu_ecritures, attendu_ailleurs),
                    "transaction de {} {}",
                    t.amount,
                    t.currency
                );
                reglements += n.settlement_count;
                ecritures += n.entry_count;
                ailleurs += n.matched_elsewhere_count;
            }
        }
    }
    // Le jeu n'est pas vide : chaque nombre est exercé, et le plafond atteint.
    assert!(reglements > 0 && ecritures > 0 && ailleurs > 0 && absentes > 0);
    let t999 = transactions
        .iter()
        .find(|t| t.amount == dec("999.00"))
        .unwrap();
    assert_eq!(page[&t999.id].entry_count, 50);
    let de = |montant: &str| {
        let t = transactions
            .iter()
            .find(|t| t.amount == dec(montant) && t.currency == "CHF")
            .unwrap();
        page[&t.id]
    };
    assert_eq!(de("555.00").settlement_count, 50, "plafond des règlements");
    assert_eq!(
        de("444.00").matched_elsewhere_count,
        50,
        "plafond du second côté"
    );
    assert_eq!(
        (
            de("777.77").settlement_count,
            de("777.77").entry_count,
            de("777.77").matched_elsewhere_count
        ),
        (0, 0, 0),
        "l'écriture d'une autre société ne compte pas"
    );
}

// ============================================================
// Story 34-4 (#828), AC-8 — le banc de coût de la page des propositions
// ============================================================

/// Rapproche, **par SQL forgé**, chaque écriture numérotée `premier..=dernier`
/// qui satisfait `filtre` (SQL sur `e`) d'une transaction `reconciled` du compte
/// bancaire `bank`, de montant **opposé** au mouvement de l'écriture sur le
/// compte de la banque de `ctx` — l'autre côté d'un virement (revue P1, A1-8 et
/// E1-9). L'import qui les porte naît d'une transaction de −0,01 laissée en
/// attente sur `bank`.
async fn rapprocher_ailleurs(ctx: &Ctx, bank: i64, premier: i64, dernier: i64, filtre: &str) {
    let porteuse = ctx.transaction(bank, "-0.01", "2026-04-15").await;
    let import: i64 = sqlx::query_scalar("SELECT import_id FROM bank_transactions WHERE id = ?")
        .bind(porteuse)
        .fetch_one(&ctx.pool)
        .await
        .unwrap();
    sqlx::query(&format!(
        "INSERT INTO bank_transactions (company_id, import_id, bank_account_id, booking_date, \
         amount, currency, details, status, matched_entry_id) \
         SELECT e.company_id, ?, ?, e.entry_date, \
                -(SELECT SUM(l.debit) - SUM(l.credit) FROM journal_entry_lines l \
                   WHERE l.entry_id = e.id AND l.account_id = ?), \
                'CHF', 'Rapprochée ailleurs (forgé)', 'reconciled', e.id \
           FROM journal_entries e \
          WHERE e.company_id = ? AND e.entry_number BETWEEN ? AND ? AND {filtre}"
    ))
    .bind(import)
    .bind(bank)
    .bind(ctx.ledger)
    .bind(ctx.company_id())
    .bind(premier)
    .bind(dernier)
    .execute(&ctx.pool)
    .await
    .unwrap();
}

/// Ajoute au compte du grand livre `ledger` les écritures numérotées
/// `premier..=dernier` — **SQL forgé** (un banc de 20 000 écritures par la
/// route coûterait des minutes) : deux lignes chacune, `D ledger / C 3000` pour
/// un numéro pair, `D 4000 / C ledger` pour un impair, d'un montant de
/// `(numéro mod 997) + 1` francs. Aucune n'est contre-passée ni, à la
/// création, rapprochée ([`rapprocher_ailleurs`] en rapproche ensuite).
///
/// `montant_fixe` : toutes au débit de la banque (`D ledger / C 3000`), de ce
/// montant (test d'égalité de l'AC-8, au-delà du plafond de 50).
async fn forger_historique(ctx: &Ctx, premier: i64, dernier: i64, montant_fixe: Option<&str>) {
    let (sens_banque_au_debit, montant) = match montant_fixe {
        Some(m) => ("TRUE".to_string(), m.to_string()),
        None => (
            "MOD(e.entry_number, 2) = 0".to_string(),
            "MOD(e.entry_number, 997) + 1".to_string(),
        ),
    };
    let mut n = premier;
    while n <= dernier {
        let fin = (n + 999).min(dernier);
        let valeurs: Vec<String> = (n..=fin)
            .map(|k| {
                let jour = d("2021-01-01") + chrono::Days::new((k % 3000) as u64);
                format!(
                    "({}, {}, {k}, '{jour}', 'Banque', 'Banc 34-4')",
                    ctx.company_id(),
                    ctx.seeded.fiscal_year_id
                )
            })
            .collect();
        sqlx::query(&format!(
            "INSERT INTO journal_entries (company_id, fiscal_year_id, entry_number, entry_date, \
             journal, description) VALUES {}",
            valeurs.join(", ")
        ))
        .execute(&ctx.pool)
        .await
        .unwrap();
        n = fin + 1;
    }
    // Ligne 1 au débit (le compte de la banque pour un numéro pair, 4000 sinon),
    // ligne 2 au crédit (3000 pour un numéro pair, le compte de la banque sinon).
    for (ordre, compte_pair, compte_impair, colonne) in [
        (1, ctx.ledger, ctx.compte("4000"), "debit"),
        (2, ctx.compte("3000"), ctx.ledger, "credit"),
    ] {
        sqlx::query(&format!(
            "INSERT INTO journal_entry_lines (entry_id, account_id, line_order, {colonne}) \
             SELECT e.id, IF({sens_banque_au_debit}, ?, ?), {ordre}, {montant} \
               FROM journal_entries e \
              WHERE e.company_id = ? AND e.fiscal_year_id = ? AND e.entry_number BETWEEN ? AND ?"
        ))
        .bind(compte_pair)
        .bind(compte_impair)
        .bind(ctx.company_id())
        .bind(ctx.seeded.fiscal_year_id)
        .bind(premier)
        .bind(dernier)
        .execute(&ctx.pool)
        .await
        .unwrap();
    }
}

/// AC-8 de la Story 34-4 (#828) — le coût de `GET /reconciliation/proposals`
/// à `limit=500`, sur **500 transactions en attente** et un historique de
/// **1 000, 5 000 puis 20 000 écritures** sur le compte du grand livre de la
/// banque, trois mesures par palier. Le même banc se joue **avant** le champ
/// `alreadyBooked` et **après** : le coût ajouté est la différence.
///
/// ⚠️ **Plancher** : la base de dev est en tmpfs, le NAS lit sur disque. Les
/// montants des transactions (`(13 k mod 997) + 1`, signe alterné) et ceux de
/// l'historique se recoupent : la plupart des transactions ont des candidates.
/// Depuis la revue P1 (E1-9), la **moitié** de l'historique est rapprochée
/// (`rapprocher_ailleurs`, numéros `≡ 0, 1 mod 4`) de transactions de sens
/// contraire d'un second compte bancaire : comme dans des livres tenus, où la
/// plupart des écritures de la banque sont rapprochées, le `EXISTS` du second
/// côté trouve une ligne. (Les mesures d'avant la revue P1, sans écriture
/// rapprochée, restent au Change Log de la fiche.) Ignoré par défaut ; à jouer
/// en `--release`, durées imprimées (`--nocapture`), sans seuil asserté (la
/// pente se juge au Change Log).
#[sqlx::test(migrations = "../kesh-db/test-schema")]
#[ignore = "banc de coût (AC-8, Story 34-4) : à jouer à la main, en --release"]
async fn banc_cout_propositions_deja_comptabilisees(pool: MySqlPool) {
    let ctx = monter(&pool).await;
    let bank = ctx.banque(IBAN_A).await;
    let ledger_b = compte_du_grand_livre(&pool, ctx.company_id(), "1021", "Banque 2").await;
    let autre_banque = ctx.banque_liee(IBAN_B, ledger_b).await;
    let jour = d("2026-04-15");
    let transactions: Vec<NewBankTransaction> = (0..500i64)
        .map(|k| {
            let montant = Decimal::from((13 * k) % 997 + 1);
            NewBankTransaction {
                company_id: ctx.company_id(),
                bank_account_id: bank,
                booking_date: jour,
                value_date: Some(jour),
                amount: if k % 2 == 0 { montant } else { -montant },
                currency: "CHF".into(),
                reference: None,
                details: format!("Banc {k}"),
                end_to_end_id: None,
                transaction_id: None,
                counterparty_iban: None,
                counterparty_name: Some(format!("Contrepartie {k}")),
            }
        })
        .collect();
    let mut tx = ctx.pool.begin().await.unwrap();
    bank_imports::create_with_transactions(
        &mut tx,
        NewBankImport {
            company_id: ctx.company_id(),
            bank_account_id: bank,
            filename: "banc.xml".into(),
            file_hash: hash64("banc-34-4"),
            source_format: BankImportSourceFormat::Camt053V04,
            statement_id: Some("BANC-34-4".into()),
            period_from: jour,
            period_to: jour,
            opening_balance: None,
            closing_balance: None,
            transaction_count: 500,
            imported_by_user_id: ctx.seeded.admin_user_id,
        },
        transactions,
    )
    .await
    .expect("import");
    tx.commit().await.unwrap();

    let mut deja = 0i64;
    for palier in [1_000i64, 5_000, 20_000] {
        forger_historique(&ctx, deja + 1, palier, None).await;
        rapprocher_ailleurs(
            &ctx,
            autre_banque,
            deja + 1,
            palier,
            "MOD(e.entry_number, 4) < 2",
        )
        .await;
        deja = palier;
        for essai in 1..=3 {
            let debut = std::time::Instant::now();
            let (s, b) = ctx
                .get(&format!(
                    "/api/v1/reconciliation/proposals?bankAccountId={bank}&limit=500"
                ))
                .await;
            let duree = debut.elapsed();
            assert_eq!(s, 200, "{b}");
            assert_eq!(b["proposals"].as_array().unwrap().len(), 500);
            println!(
                "BANC 34-4 palier {palier} écritures, essai {essai} : {} ms",
                duree.as_millis()
            );
        }
    }
}
