//! Story 33-4 (refs #606, refs #755) — les bornes d'années des dates reçues du
//! client, éprouvées à travers la frontière HTTP et par les outils MCP.
//!
//! **Un test par champ** (T1) : chaque test envoie, sur le **corps complet et
//! valide** de sa route, les quatre valeurs hors plage de l'AC-2, puis un témoin
//! de forme (une valeur valide du même type, qui doit passer : sans lui, le
//! relevé ne mesurerait rien). Chaque valeur est écrite sur la sortie d'erreur
//! (`RELEVE|…`) **avant** toute assertion : c'est ce relevé, lancé sur le code
//! d'avant le correctif, qui a classé les champs (`kesh-gate-logs/e334/`).
//!
//! Trois classes (C-e334-1) : **A**, déjà refusé en 4xx partout (rien n'est
//! posé) ; **B**, au moins un refus JSON existant, une autre valeur passe
//! (contrôle dans le handler, 400 JSON `VALIDATION_ERROR`) ; **C**, aucun refus
//! JSON (`dates_recues`, 400 en `Query`, 422 en corps).
//!
//! ⚠️ Dans une chaîne de requête, `+` se lit comme un espace : encoder `%2B`,
//! sans quoi le 400 obtenu ne prouve rien.

mod common;

use std::collections::HashSet;
use std::net::SocketAddr;
use std::path::Path;
use std::sync::Arc;
use std::sync::atomic::{AtomicI64, Ordering};

use chrono::{NaiveDate, TimeDelta, Utc};
use common::mcp::attendu_outil_en_erreur;
use common::mcp_pieges::{LECTURE, banc, cle_de, societe_piegee};
use jsonwebtoken::{Algorithm, EncodingKey, Header};
use kesh_api::auth::jwt::Claims;
use kesh_api::config::Config;
use kesh_api::mcp::textes::TEXTE_ARGUMENTS_INVALIDES;
use kesh_api::{AppState, build_router};
use kesh_db::entities::{
    BankImportSourceFormat, ContactType, NewBankAccount, NewBankImport, NewBankTransaction,
    NewContact, NewSupplierInvoice, NewSupplierInvoiceLine, SendValidation,
};
use kesh_db::repositories::{bank_accounts, bank_imports, contacts, supplier_invoices};
use kesh_db::test_fixtures::{SeededCompany, mark_onboarding_complete, seed_accounting_company};
use rust_decimal_macros::dec;
use serde_json::{Value, json};
use sqlx::MySqlPool;

const TEST_JWT_SECRET: &[u8] = b"test-secret-32-bytes-minimum-test-secret-padding";
const TEST_ADMIN_PASSWORD: &str = "e2e-test-admin-password";
const MARQUEUR: &str = "1000 à 9999";

/// Les quatre valeurs hors plage d'un `NaiveDate` (AC-2).
const HORS_PLAGE: [&str; 4] = ["+10000-01-01", "+70000-01-01", "-0001-01-01", "0999-12-31"];

static SEQ: AtomicI64 = AtomicI64::new(1);

fn seq() -> i64 {
    SEQ.fetch_add(1, Ordering::SeqCst)
}

fn aujourd_hui() -> String {
    Utc::now().date_naive().to_string()
}

/// La forme d'une valeur dans une chaîne de requête.
fn en_requete(v: &str) -> String {
    v.replace('+', "%2B")
}

// ---------------------------------------------------------------------------
// Harnais (recopié, patron `rejeu_interblocage_e2e.rs`, choix C70 de la 15-5e1)
// ---------------------------------------------------------------------------

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

fn forge_jwt(user_id: i64, company_id: i64) -> String {
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

/// Une société comptable (exercice 2020-2030, cinq comptes), le serveur, un JWT.
struct Monde {
    pool: MySqlPool,
    base_url: String,
    client: reqwest::Client,
    jwt: String,
    societe: SeededCompany,
}

async fn monter(pool: &MySqlPool) -> Monde {
    let societe = seed_accounting_company(pool).await.expect("société");
    mark_onboarding_complete(pool).await.expect("onboarding");
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
        .expect("i18n"),
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
    Monde {
        pool: pool.clone(),
        base_url: format!("http://{addr}"),
        client: reqwest::Client::new(),
        jwt: forge_jwt(societe.admin_user_id, societe.company_id),
        societe,
    }
}

type Reponse = (u16, String);

impl Monde {
    fn company_id(&self) -> i64 {
        self.societe.company_id
    }

    fn compte(&self, numero: &str) -> i64 {
        self.societe.accounts[numero]
    }

    async fn envoyer(
        &self,
        methode: reqwest::Method,
        chemin: &str,
        corps: Option<Value>,
    ) -> Reponse {
        let mut req = self
            .client
            .request(methode, format!("{}{}", self.base_url, chemin))
            .bearer_auth(&self.jwt);
        if let Some(c) = corps {
            req = req.json(&c);
        }
        // ⛔ Une date hors plage liée par `QueryBuilder::push_bind` **panique**
        // (`sqlx-core-0.8.6/src/query_builder.rs:158`) : aucune réponse, la
        // connexion tombe. Le relevé l'écrit 0, sans quoi le test mourrait ici.
        match req.send().await {
            Ok(resp) => {
                let status = resp.status().as_u16();
                (status, resp.text().await.unwrap_or_default())
            }
            Err(e) => (0, format!("connexion coupée : {e}")),
        }
    }

    async fn get(&self, chemin: &str) -> Reponse {
        self.envoyer(reqwest::Method::GET, chemin, None).await
    }

    async fn post(&self, chemin: &str, corps: Value) -> Reponse {
        self.envoyer(reqwest::Method::POST, chemin, Some(corps))
            .await
    }

    async fn put(&self, chemin: &str, corps: Value) -> Reponse {
        self.envoyer(reqwest::Method::PUT, chemin, Some(corps))
            .await
    }

    /// L'identifiant et la version d'une réponse de création.
    fn id_version(reponse: &Reponse, geste: &str) -> (i64, i64) {
        assert!(
            (200..300).contains(&reponse.0),
            "{geste} : {} {}",
            reponse.0,
            reponse.1
        );
        let v: Value = serde_json::from_str(&reponse.1).unwrap();
        (
            v["id"].as_i64().unwrap(),
            v["version"].as_i64().unwrap_or(0),
        )
    }

    async fn contact(&self, fournisseur: bool, delai: Option<i32>) -> i64 {
        contacts::create(
            &self.pool,
            self.societe.admin_user_id,
            NewContact {
                company_id: self.company_id(),
                contact_type: ContactType::Entreprise,
                name: format!("Contact {}", seq()),
                first_name: None,
                last_name: None,
                is_client: !fournisseur,
                is_supplier: fournisseur,
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
                default_payment_terms_days: delai,
                language: None,
                salutation: kesh_db::entities::contact::Salutation::Neutre,
            },
        )
        .await
        .expect("contact")
        .id
    }

    fn lignes_facture() -> Value {
        json!([{"description": "Conseil", "quantity": "1", "unitPrice": "100.00", "vatRate": "8.10"}])
    }

    /// Une facture client brouillon, par la route : `(id, version)`.
    async fn facture_brouillon(&self) -> (i64, i64) {
        let client = self.contact(false, None).await;
        let r = self
            .post(
                "/api/v1/invoices",
                json!({"contactId": client, "date": aujourd_hui(), "lines": Self::lignes_facture()}),
            )
            .await;
        Self::id_version(&r, "facture brouillon")
    }

    /// Une facture client validée, par les routes : `(id, version)`.
    async fn facture_validee(&self) -> (i64, i64) {
        let (id, _) = self.facture_brouillon().await;
        let r = self
            .envoyer(
                reqwest::Method::POST,
                &format!("/api/v1/invoices/{id}/validate"),
                None,
            )
            .await;
        assert_eq!(r.0, 200, "validation : {}", r.1);
        let v: Value = serde_json::from_str(&r.1).unwrap();
        let version = v["version"]
            .as_i64()
            .or_else(|| v["invoice"]["version"].as_i64())
            .expect("version");
        (id, version)
    }

    /// Une écriture équilibrée par la route : `(id, version)`.
    async fn ecriture(&self) -> (i64, i64) {
        let r = self
            .post(
                "/api/v1/journal-entries",
                json!({
                    "entryDate": aujourd_hui(), "journal": "OD", "description": "Montage",
                    "lines": self.lignes_ecriture(),
                }),
            )
            .await;
        Self::id_version(&r, "écriture")
    }

    fn lignes_ecriture(&self) -> Value {
        json!([
            {"accountId": self.compte("1000"), "debit": "100.00", "credit": "0"},
            {"accountId": self.compte("3000"), "debit": "0", "credit": "100.00"},
        ])
    }

    fn lignes_fournisseur(&self) -> Value {
        json!([{
            "description": "Fournitures", "quantity": "1", "unitPrice": "50.00", "vatRate": "0",
            "expenseAccountId": self.compte("4000"),
        }])
    }

    /// Une facture fournisseur validée, IBAN du créancier compris.
    async fn facture_fournisseur_validee(&self) -> i64 {
        let fournisseur = self.contact(true, None).await;
        sqlx::query(
            "UPDATE company_invoice_settings SET default_payable_account_id = ? WHERE company_id = ?",
        )
        .bind(self.compte("2000"))
        .bind(self.company_id())
        .execute(&self.pool)
        .await
        .unwrap();
        supplier_invoices::create_validated(
            &self.pool,
            NewSupplierInvoice {
                company_id: self.company_id(),
                contact_id: fournisseur,
                supplier_invoice_number: Some(format!("FF-{}", seq())),
                invoice_date: Utc::now().date_naive(),
                due_date: None,
                creditor_iban: Some("CH5604835012345678009".into()),
                creditor_qr_iban: None,
                payment_reference: None,
                expected_payment_amount: None,
                project_id: None,
                lines: vec![NewSupplierInvoiceLine {
                    description: "Fournitures".into(),
                    quantity: rust_decimal::Decimal::ONE,
                    unit_price: dec!(50.00),
                    vat_rate: rust_decimal::Decimal::ZERO,
                    expense_account_id: self.compte("4000"),
                }],
            },
            self.societe.admin_user_id,
        )
        .await
        .expect("facture fournisseur")
        .invoice
        .id
    }

    /// Un compte bancaire lié à un compte de banque imputable.
    async fn compte_bancaire(&self) -> i64 {
        let id = bank_accounts::create(
            &self.pool,
            NewBankAccount {
                company_id: self.company_id(),
                bank_name: "Banque des bornes".into(),
                iban: "CH4431999123000889012".into(),
                qr_iban: None,
                is_primary: false,
            },
        )
        .await
        .expect("compte bancaire")
        .id;
        let banque: i64 = sqlx::query_scalar(
            "INSERT INTO accounts (company_id, number, name, account_type, active, postable) \
             VALUES (?, '1020', 'Banque', 'Asset', 1, 1) RETURNING id",
        )
        .bind(self.company_id())
        .fetch_one(&self.pool)
        .await
        .unwrap();
        sqlx::query("UPDATE bank_accounts SET journal_account_id = ? WHERE id = ?")
            .bind(banque)
            .bind(id)
            .execute(&self.pool)
            .await
            .unwrap();
        id
    }

    /// Une transaction bancaire en attente, déposée par le dépôt.
    async fn transaction(
        &self,
        compte: i64,
        date: NaiveDate,
        montant: rust_decimal::Decimal,
    ) -> i64 {
        let n = seq();
        let mut tx = self.pool.begin().await.unwrap();
        let import = NewBankImport {
            company_id: self.company_id(),
            bank_account_id: compte,
            filename: format!("releve-{n}.xml"),
            file_hash: format!("{n:0>64}"),
            source_format: BankImportSourceFormat::Camt053V04,
            statement_id: Some(format!("STMT-{n}")),
            period_from: date,
            period_to: date,
            opening_balance: None,
            closing_balance: None,
            transaction_count: 1,
            imported_by_user_id: self.societe.admin_user_id,
        };
        let (_, inserees) = bank_imports::create_with_transactions(
            &mut tx,
            import,
            vec![NewBankTransaction {
                company_id: self.company_id(),
                bank_account_id: compte,
                booking_date: date,
                value_date: Some(date),
                amount: montant,
                currency: "CHF".into(),
                reference: Some(format!("REF-{n}")),
                details: "Bornes".into(),
                end_to_end_id: None,
                transaction_id: None,
                counterparty_iban: None,
                counterparty_name: None,
            }],
        )
        .await
        .expect("transaction");
        tx.commit().await.unwrap();
        inserees[0].id
    }

    /// Une transaction du jour, débit de 100.
    async fn transaction_du_jour(&self, compte: i64) -> i64 {
        self.transaction(compte, Utc::now().date_naive(), dec!(-100.00))
            .await
    }

    fn lignes_eclatement(&self) -> Value {
        json!([
            {"counterpartyAccountId": self.compte("4000"), "amount": "60.00", "description": "A"},
            {"counterpartyAccountId": self.compte("4000"), "amount": "40.00", "description": "B"},
        ])
    }
}

// ---------------------------------------------------------------------------
// Le relevé
// ---------------------------------------------------------------------------

/// Ce qu'un champ doit rendre aux valeurs hors plage, une fois borné.
#[derive(Clone, Copy, Debug, PartialEq)]
enum Attendu {
    /// Déjà refusé en 4xx : statut et message existants, relevés par T1.
    A,
    /// Contrôle dans le handler : 400 JSON `VALIDATION_ERROR`, marqueur.
    B,
    /// Comme B, sauf ce qui est au futur : 422 `REMINDER_DATE_IN_FUTURE`.
    BFutur,
    /// `dates_recues` sur une requête : 400 en texte, marqueur.
    CRequete,
    /// `dates_recues` sur un corps : 422 en texte, marqueur.
    CCorps,
    /// Lot `POST …/accept` (*Pattern batch*) : 200, et cette proposition dans
    /// `failed[]`, `VALIDATION_ERROR`, `details.reason`.
    ParProposition,
}

struct Releve {
    site: &'static str,
    lignes: Vec<(String, Reponse)>,
    temoin: Option<Reponse>,
    absent: Option<Reponse>,
}

fn est_refus_json(r: &Reponse) -> bool {
    (400..500).contains(&r.0) && serde_json::from_str::<Value>(&r.1).is_ok()
}

impl Releve {
    fn new(site: &'static str) -> Self {
        Self {
            site,
            lignes: Vec::new(),
            temoin: None,
            absent: None,
        }
    }

    fn note(&mut self, valeur: &str, r: Reponse) {
        self.lignes.push((valeur.to_string(), r));
    }

    /// La classe que déduit le relevé (C-e334-1).
    fn classe_deduite(&self) -> char {
        let refuses = self
            .lignes
            .iter()
            .filter(|(_, r)| (400..500).contains(&r.0))
            .count();
        if refuses == self.lignes.len() {
            'A'
        } else if self.lignes.iter().any(|(_, r)| est_refus_json(r)) {
            'B'
        } else {
            'C'
        }
    }

    /// Écrit le relevé, puis juge le champ. Le relevé part **avant** toute
    /// assertion : sur le code d'avant le correctif, c'est lui qui classe.
    fn fin(self, attendu: Attendu) {
        for (v, (s, t)) in &self.lignes {
            eprintln!(
                "RELEVE|{}|{v}|{s}|{}",
                self.site,
                t.chars().take(110).collect::<String>().replace('\n', " ")
            );
        }
        let extrait = |r: &Option<Reponse>| {
            r.as_ref()
                .map(|(s, t)| format!("{s} {}", t.chars().take(110).collect::<String>()))
                .unwrap_or_else(|| "-".into())
        };
        eprintln!("TEMOIN|{}|{}", self.site, extrait(&self.temoin));
        eprintln!("ABSENT|{}|{}", self.site, extrait(&self.absent));
        eprintln!("CLASSE|{}|{}", self.site, self.classe_deduite());

        let mut fautes = Vec::new();
        for (v, (s, t)) in &self.lignes {
            let code = serde_json::from_str::<Value>(t)
                .ok()
                .map(|j| j["error"]["code"].clone());
            let ok = match attendu {
                Attendu::A => (400..500).contains(s),
                Attendu::B => {
                    *s == 400 && code == Some(json!("VALIDATION_ERROR")) && t.contains(MARQUEUR)
                }
                Attendu::BFutur if v.starts_with('+') => {
                    *s == 422 && code == Some(json!("REMINDER_DATE_IN_FUTURE"))
                }
                Attendu::BFutur => {
                    *s == 400 && code == Some(json!("VALIDATION_ERROR")) && t.contains(MARQUEUR)
                }
                Attendu::CRequete => *s == 400 && t.contains(MARQUEUR),
                Attendu::CCorps => *s == 422 && t.contains(MARQUEUR),
                Attendu::ParProposition => {
                    let j = serde_json::from_str::<Value>(t).unwrap_or(Value::Null);
                    *s == 200
                        && j["accepted"].as_array().is_some_and(Vec::is_empty)
                        && j["failed"][0]["errorCode"] == "VALIDATION_ERROR"
                        && j["failed"][0]["details"]["reason"] == "value_date_out_of_range"
                }
            };
            if !ok {
                fautes.push(format!("{v} → {s} {t}"));
            }
        }
        // Le témoin de forme : sans lui, le relevé ne mesure rien.
        match &self.temoin {
            Some((s, t)) if !(200..300).contains(s) => fautes.push(format!("témoin → {s} {t}")),
            None => fautes.push("témoin absent".into()),
            _ => {}
        }
        // AC-6 : un `Option` reste facultatif.
        if let Some((s, t)) = &self.absent
            && !(200..300).contains(s)
        {
            fautes.push(format!("champ absent → {s} {t}"));
        }
        assert!(
            fautes.is_empty(),
            "{} ({attendu:?}) :\n{}",
            self.site,
            fautes.join("\n")
        );
    }
}

/// Éprouve un paramètre de requête : `base` porte déjà ses autres paramètres.
/// `facultatif` : le test du champ absent (AC-6).
async fn requete(
    m: &Monde,
    site: &'static str,
    base: &str,
    champ: &str,
    temoin: &str,
    attendu: Attendu,
    facultatif: bool,
) {
    let sep = if base.contains('?') { '&' } else { '?' };
    let mut r = Releve::new(site);
    for v in HORS_PLAGE {
        let chemin = format!("{base}{sep}{champ}={}", en_requete(v));
        r.note(v, m.get(&chemin).await);
    }
    r.temoin = Some(m.get(&format!("{base}{sep}{champ}={temoin}")).await);
    if facultatif {
        r.absent = Some(m.get(base).await);
    }
    r.fin(attendu);
}

// ---------------------------------------------------------------------------
// Requêtes (17 + 2 chaînes)
// ---------------------------------------------------------------------------

const J: &str = "2026-01-01";

impl Monde {
    /// Remplace `{fy}` et `{projet}` d'un chemin de requête.
    async fn gabarit(&self, g: &str) -> String {
        let mut s = g.replace("{fy}", &self.societe.fiscal_year_id.to_string());
        if s.contains("{projet}") {
            let r = self
                .post(
                    "/api/v1/projects",
                    json!({"code": format!("P{}", seq()), "name": "Projet"}),
                )
                .await;
            let (id, _) = Self::id_version(&r, "projet");
            s = s.replace("{projet}", &id.to_string());
        }
        s
    }
}

macro_rules! champ_requete {
    ($nom:ident, $gabarit:literal, $champ:literal, $attendu:expr, $facultatif:expr) => {
        #[sqlx::test(migrations = "../kesh-db/test-schema")]
        async fn $nom(pool: MySqlPool) {
            let m = monter(&pool).await;
            let base = m.gabarit($gabarit).await;
            requete(
                &m,
                concat!("GET ", $gabarit, " ", $champ),
                &base,
                $champ,
                J,
                $attendu,
                $facultatif,
            )
            .await;
        }
    };
}

use Attendu::{A, B, BFutur, CCorps, CRequete, ParProposition};

champ_requete!(
    q_factures_date_from,
    "/api/v1/invoices",
    "dateFrom",
    CRequete,
    true
);
champ_requete!(
    q_factures_date_to,
    "/api/v1/invoices",
    "dateTo",
    CRequete,
    true
);
champ_requete!(
    q_echeancier_date_from,
    "/api/v1/invoices/due-dates",
    "dateFrom",
    CRequete,
    true
);
champ_requete!(
    q_echeancier_date_to,
    "/api/v1/invoices/due-dates",
    "dateTo",
    CRequete,
    true
);
champ_requete!(
    q_echeancier_due_before,
    "/api/v1/invoices/due-dates",
    "dueBefore",
    CRequete,
    true
);
champ_requete!(
    q_bilan_debut,
    "/api/v1/reports/balance-sheet?fiscalYearId={fy}",
    "periodStart",
    A,
    true
);
champ_requete!(
    q_bilan_fin,
    "/api/v1/reports/balance-sheet?fiscalYearId={fy}",
    "periodEnd",
    A,
    true
);
champ_requete!(
    q_grand_livre_from,
    "/api/v1/reports/general-ledger?to=2026-12-31",
    "from",
    B,
    false
);
champ_requete!(
    q_grand_livre_to,
    "/api/v1/reports/general-ledger?from=2025-01-01",
    "to",
    B,
    false
);
champ_requete!(
    q_grand_livre_export_from,
    "/api/v1/reports/general-ledger/export?format=csv&to=2026-12-31",
    "from",
    B,
    false
);
champ_requete!(
    q_grand_livre_export_to,
    "/api/v1/reports/general-ledger/export?format=csv&from=2025-01-01",
    "to",
    B,
    false
);
champ_requete!(
    q_journaux_debut,
    "/api/v1/reports/journals?fiscalYearId={fy}",
    "periodStart",
    A,
    true
);
champ_requete!(
    q_journaux_fin,
    "/api/v1/reports/journals?fiscalYearId={fy}",
    "periodEnd",
    A,
    true
);
champ_requete!(
    q_export_debut,
    "/api/v1/reports/balance-sheet/export?format=csv&fiscalYearId={fy}",
    "periodStart",
    A,
    true
);
champ_requete!(
    q_export_fin,
    "/api/v1/reports/balance-sheet/export?format=csv&fiscalYearId={fy}",
    "periodEnd",
    A,
    true
);
// E-5 de la revue : `mode=cumulative` ne lit pas la période ; `fiscal_year` la lit.
champ_requete!(
    q_projet_exercice_debut,
    "/api/v1/reports/project-expenses?projectId={projet}&mode=fiscal_year&fiscalYearId={fy}",
    "periodStart",
    CRequete,
    true
);
champ_requete!(
    q_projet_exercice_fin,
    "/api/v1/reports/project-expenses?projectId={projet}&mode=fiscal_year&fiscalYearId={fy}",
    "periodEnd",
    CRequete,
    true
);
champ_requete!(
    q_projet_debut,
    "/api/v1/reports/project-expenses?projectId={projet}&mode=cumulative",
    "periodStart",
    CRequete,
    true
);
champ_requete!(
    q_projet_fin,
    "/api/v1/reports/project-expenses?projectId={projet}&mode=cumulative",
    "periodEnd",
    CRequete,
    true
);
// Chaînes analysées par `analyser_date` : 400 JSON de validation.
champ_requete!(
    q_ecritures_date_from,
    "/api/v1/journal-entries",
    "dateFrom",
    B,
    true
);
champ_requete!(
    q_ecritures_date_to,
    "/api/v1/journal-entries",
    "dateTo",
    B,
    true
);

// ---------------------------------------------------------------------------
// Corps (29)
// ---------------------------------------------------------------------------

/// Un test de corps : `$prepare` monte, pour chaque valeur (`$v`), ce qu'il faut
/// et rend `(méthode, chemin, corps)`. `$sans` : le même corps sans le champ
/// (AC-6), pour un `Option`.
macro_rules! champ_corps {
    ($nom:ident, $site:literal, $attendu:expr, $temoin:expr, |$m:ident, $v:ident| $prepare:block) => {
        #[sqlx::test(migrations = "../kesh-db/test-schema")]
        async fn $nom(pool: MySqlPool) {
            let m_ = monter(&pool).await;
            let mut r = Releve::new($site);
            for v_ in HORS_PLAGE {
                let rep = {
                    let $m = &m_;
                    let $v = Some(v_.to_string());
                    $prepare
                };
                r.note(v_, rep);
            }
            r.temoin = Some({
                let $m = &m_;
                let t: &str = $temoin;
                let $v = Some(t.to_string());
                $prepare
            });
            r.absent = None;
            r.fin($attendu);
        }
    };
    ($nom:ident, $site:literal, $attendu:expr, $temoin:expr, facultatif, |$m:ident, $v:ident| $prepare:block) => {
        #[sqlx::test(migrations = "../kesh-db/test-schema")]
        async fn $nom(pool: MySqlPool) {
            let m_ = monter(&pool).await;
            let mut r = Releve::new($site);
            for v_ in HORS_PLAGE {
                let rep = {
                    let $m = &m_;
                    let $v = Some(v_.to_string());
                    $prepare
                };
                r.note(v_, rep);
            }
            r.temoin = Some({
                let $m = &m_;
                let t: &str = $temoin;
                let $v = Some(t.to_string());
                $prepare
            });
            r.absent = Some({
                let $m = &m_;
                let $v: Option<String> = None;
                $prepare
            });
            r.fin($attendu);
        }
    };
}

/// Ajoute `cle: valeur` à un objet JSON si la valeur est présente : une clé
/// absente du corps, et non un `null` (AC-6).
fn avec(mut corps: Value, cle: &str, v: Option<String>) -> Value {
    if let Some(v) = v {
        corps[cle] = json!(v);
    }
    corps
}

/// Un taux de TVA neuf : `(société, taux, début)` est unique.
///
/// ⚠️ `SEQ` est commun au processus : sous `cargo test` (la CI), les 63 tests de ce
/// binaire le font monter au-delà de 90, et `10 + seq()` sortait de 0 à 100 (refus
/// « Le taux doit être compris entre 0 et 100. »), ce que `nextest`, un processus par
/// test, ne montre pas. Vu par `scripts/gate-ci.sh` à l'intégration de la 0.17.5.
/// Les centièmes portent le compteur : 8000 taux distincts, tous de 10,00 à 89,99.
fn taux() -> String {
    let n = seq();
    format!("{}.{:02}", 10 + (n / 100) % 80, n % 100)
}

/// Le verrou de période de la société, posé ou retiré en SQL (montage).
async fn verrou(m: &Monde, date: Option<&str>) {
    sqlx::query("UPDATE companies SET books_locked_through = ? WHERE id = ?")
        .bind(date)
        .bind(m.company_id())
        .execute(&m.pool)
        .await
        .unwrap();
}

champ_corps!(
    c_verrou_pose,
    "POST books-lock through",
    B,
    "2021-01-01",
    |m, v| {
        verrou(m, None).await;
        m.post(
            "/api/v1/companies/current/books-lock",
            avec(json!({}), "through", v),
        )
        .await
    }
);

champ_corps!(
    c_verrou_retrait,
    "POST books-lock/release through",
    B,
    "2021-01-01",
    facultatif,
    |m, v| {
        verrou(m, Some("2022-01-01")).await;
        m.post(
            "/api/v1/companies/current/books-lock/release",
            avec(json!({"motif": "Correction"}), "through", v),
        )
        .await
    }
);

champ_corps!(
    c_avoir_date,
    "POST credit-notes date",
    B,
    &aujourd_hui(),
    |m, v| {
        let (f, _) = m.facture_validee().await;
        m.post(
            "/api/v1/credit-notes",
            avec(json!({"invoiceId": f}), "date", v),
        )
        .await
    }
);

/// Les exercices créés par un test précédent de la boucle ne doivent pas faire
/// chevaucher le suivant.
async fn sans_exercices_neufs(m: &Monde) {
    sqlx::query("DELETE FROM fiscal_years WHERE company_id = ? AND name LIKE 'Borne%'")
        .bind(m.company_id())
        .execute(&m.pool)
        .await
        .unwrap();
}

champ_corps!(
    c_exercice_debut,
    "POST fiscal-years startDate",
    B,
    "1500-01-01",
    |m, v| {
        sans_exercices_neufs(m).await;
        m.post(
            "/api/v1/fiscal-years",
            avec(
                json!({"name": format!("Borne {}", seq()), "endDate": "1500-12-31"}),
                "startDate",
                v,
            ),
        )
        .await
    }
);

champ_corps!(
    c_exercice_fin,
    "POST fiscal-years endDate",
    B,
    "2035-12-31",
    |m, v| {
        sans_exercices_neufs(m).await;
        m.post(
            "/api/v1/fiscal-years",
            avec(
                json!({"name": format!("Borne {}", seq()), "startDate": "2035-01-01"}),
                "endDate",
                v,
            ),
        )
        .await
    }
);

champ_corps!(
    c_facture_date,
    "POST invoices date",
    B,
    &aujourd_hui(),
    |m, v| {
        let client = m.contact(false, None).await;
        m.post(
            "/api/v1/invoices",
            avec(
                json!({"contactId": client, "lines": Monde::lignes_facture()}),
                "date",
                v,
            ),
        )
        .await
    }
);

champ_corps!(
    c_facture_echeance,
    "POST invoices dueDate",
    B,
    "2027-01-01",
    facultatif,
    |m, v| {
        let client = m.contact(false, None).await;
        m.post(
        "/api/v1/invoices",
        avec(
            json!({"contactId": client, "date": aujourd_hui(), "lines": Monde::lignes_facture()}),
            "dueDate",
            v,
        ),
    )
    .await
    }
);

champ_corps!(
    c_facture_maj_date,
    "PUT invoices date",
    B,
    &aujourd_hui(),
    |m, v| {
        let client = m.contact(false, None).await;
        let (f, version) = m.facture_brouillon().await;
        m.put(
            &format!("/api/v1/invoices/{f}"),
            avec(
                json!({"contactId": client, "lines": Monde::lignes_facture(), "version": version}),
                "date",
                v,
            ),
        )
        .await
    }
);

champ_corps!(
    c_facture_maj_echeance,
    "PUT invoices dueDate",
    B,
    "2027-01-01",
    facultatif,
    |m, v| {
        let client = m.contact(false, None).await;
        let (f, version) = m.facture_brouillon().await;
        m.put(
        &format!("/api/v1/invoices/{f}"),
        avec(
            json!({"contactId": client, "date": aujourd_hui(), "lines": Monde::lignes_facture(), "version": version}),
            "dueDate",
            v,
        ),
    )
    .await
    }
);

champ_corps!(
    c_reglement,
    "POST settlements settledOn",
    B,
    &aujourd_hui(),
    |m, v| {
        let (f, version) = m.facture_validee().await;
        m.post(
        &format!("/api/v1/invoices/{f}/settlements"),
        avec(
            json!({"settlementType": "internal_account", "accountId": m.compte("1000"), "amount": "108.10",
                   "version": version}),
            "settledOn",
            v,
        ),
    )
    .await
    }
);

champ_corps!(
    c_solde,
    "POST write-off settledOn",
    B,
    &aujourd_hui(),
    |m, v| {
        sqlx::query(
        "UPDATE company_invoice_settings SET default_discount_account_id = ? WHERE company_id = ?",
    )
    .bind(m.compte("4000"))
    .bind(m.company_id())
    .execute(&m.pool)
    .await
    .unwrap();
        let (f, version) = m.facture_validee().await;
        let reglement = m
            .post(
                &format!("/api/v1/invoices/{f}/settlements"),
                json!({"settlementType": "internal_account", "accountId": m.compte("1000"),
                   "amount": "40.00", "settledOn": aujourd_hui(), "version": version}),
            )
            .await;
        assert_eq!(reglement.0, 200, "{}", reglement.1);
        let lu = m.get(&format!("/api/v1/invoices/{f}")).await;
        let version = serde_json::from_str::<Value>(&lu.1).unwrap()["version"]
            .as_i64()
            .unwrap();
        m.post(
            &format!("/api/v1/invoices/{f}/write-off"),
            avec(
                json!({"nature": "discount", "version": version}),
                "settledOn",
                v,
            ),
        )
        .await
    }
);

champ_corps!(
    c_ecriture_date,
    "POST journal-entries entryDate",
    B,
    &aujourd_hui(),
    |m, v| {
        m.post(
            "/api/v1/journal-entries",
            avec(
                json!({"journal": "OD", "description": "Bornes", "lines": m.lignes_ecriture()}),
                "entryDate",
                v,
            ),
        )
        .await
    }
);

champ_corps!(
    c_ecriture_maj_date,
    "PUT journal-entries entryDate",
    A,
    &aujourd_hui(),
    |m, v| {
        let (e, version) = m.ecriture().await;
        m.put(
        &format!("/api/v1/journal-entries/{e}"),
        avec(
            json!({"journal": "OD", "description": "Bornes", "lines": m.lignes_ecriture(), "version": version}),
            "entryDate",
            v,
        ),
    )
    .await
    }
);

champ_corps!(
    c_lot_execution,
    "POST payment-batches requestedExecutionDate",
    CCorps,
    &aujourd_hui(),
    |m, v| {
        let compte = compte_bancaire_unique(m).await;
        let f = m.facture_fournisseur_validee().await;
        m.post(
            "/api/v1/payment-batches",
            avec(
                json!({"bankAccountId": compte, "supplierInvoiceIds": [f]}),
                "requestedExecutionDate",
                v,
            ),
        )
        .await
    }
);

champ_corps!(
    c_lot_confirmation,
    "POST payment-batches/confirm paymentDate",
    B,
    &aujourd_hui(),
    |m, v| {
        let compte = compte_bancaire_unique(m).await;
        let f = m.facture_fournisseur_validee().await;
        let lot = m
            .post(
                "/api/v1/payment-batches",
                json!({"bankAccountId": compte, "supplierInvoiceIds": [f],
                   "requestedExecutionDate": aujourd_hui()}),
            )
            .await;
        assert_eq!(lot.0, 200, "{}", lot.1);
        let lot = serde_json::from_str::<Value>(&lot.1).unwrap()["batch"]["id"]
            .as_i64()
            .unwrap();
        m.post(
            &format!("/api/v1/payment-batches/{lot}/confirm"),
            avec(json!({}), "paymentDate", v),
        )
        .await
    }
);

champ_corps!(
    c_projet_debut,
    "POST projects startDate",
    CCorps,
    "2026-01-01",
    facultatif,
    |m, v| {
        m.post(
            "/api/v1/projects",
            avec(
                json!({"code": format!("P{}", seq()), "name": "Projet"}),
                "startDate",
                v,
            ),
        )
        .await
    }
);

champ_corps!(
    c_projet_fin,
    "POST projects endDate",
    CCorps,
    "2026-01-01",
    facultatif,
    |m, v| {
        m.post(
            "/api/v1/projects",
            avec(
                json!({"code": format!("P{}", seq()), "name": "Projet"}),
                "endDate",
                v,
            ),
        )
        .await
    }
);

async fn projet(m: &Monde) -> (i64, i64) {
    let r = m
        .post(
            "/api/v1/projects",
            json!({"code": format!("P{}", seq()), "name": "Projet"}),
        )
        .await;
    Monde::id_version(&r, "projet")
}

champ_corps!(
    c_projet_maj_debut,
    "PUT projects startDate",
    CCorps,
    "2026-01-01",
    facultatif,
    |m, v| {
        let (p, version) = projet(m).await;
        m.put(
            &format!("/api/v1/projects/{p}"),
            avec(
                json!({"code": format!("P{}", seq()), "name": "Projet", "version": version}),
                "startDate",
                v,
            ),
        )
        .await
    }
);

champ_corps!(
    c_projet_maj_fin,
    "PUT projects endDate",
    CCorps,
    "2026-01-01",
    facultatif,
    |m, v| {
        let (p, version) = projet(m).await;
        m.put(
            &format!("/api/v1/projects/{p}"),
            avec(
                json!({"code": format!("P{}", seq()), "name": "Projet", "version": version}),
                "endDate",
                v,
            ),
        )
        .await
    }
);

/// Un test de réconciliation : le compte bancaire n'est créé qu'une fois (son
/// IBAN est unique), les transactions, une par valeur.
async fn compte_bancaire_unique(m: &Monde) -> i64 {
    let existant: Option<i64> =
        sqlx::query_scalar("SELECT id FROM bank_accounts WHERE company_id = ? LIMIT 1")
            .bind(m.company_id())
            .fetch_optional(&m.pool)
            .await
            .unwrap();
    match existant {
        Some(id) => id,
        None => m.compte_bancaire().await,
    }
}

champ_corps!(
    c_rapprochement_lot,
    "POST reconciliation/accept split.valueDate",
    ParProposition,
    &aujourd_hui(),
    facultatif,
    |m, v| {
        let compte = compte_bancaire_unique(m).await;
        let tx = m.transaction_du_jour(compte).await;
        let proposition = avec(
            json!({"type": "split", "bankTransactionId": tx, "splits": m.lignes_eclatement()}),
            "valueDate",
            v,
        );
        m.post(
            "/api/v1/reconciliation/accept",
            json!({"bankAccountId": compte, "proposals": [proposition]}),
        )
        .await
    }
);

/// Le lot mixte (*Pattern batch*) : la proposition à la date hors plage tombe
/// seule, sa voisine s'écrit, le code est le même pour les quatre valeurs.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn c_rapprochement_lot_mixte(pool: MySqlPool) {
    let m = monter(&pool).await;
    let compte = compte_bancaire_unique(&m).await;
    for hors in HORS_PLAGE {
        let bonne = m.transaction_du_jour(compte).await;
        let mauvaise = m.transaction_du_jour(compte).await;
        let avant: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM journal_entries")
            .fetch_one(&pool)
            .await
            .unwrap();
        let (s, t) = m
            .post(
                "/api/v1/reconciliation/accept",
                json!({"bankAccountId": compte, "proposals": [
                    {"type": "split", "bankTransactionId": mauvaise,
                     "splits": m.lignes_eclatement(), "valueDate": hors},
                    {"type": "split", "bankTransactionId": bonne,
                     "splits": m.lignes_eclatement()},
                ]}),
            )
            .await;
        assert_eq!(s, 200, "{hors} : {t}");
        let c: Value = serde_json::from_str(&t).unwrap();
        assert_eq!(c["accepted"].as_array().unwrap().len(), 1, "{hors} : {t}");
        assert_eq!(c["accepted"][0]["bankTransactionId"], bonne, "{t}");
        assert_eq!(c["failed"].as_array().unwrap().len(), 1, "{hors} : {t}");
        assert_eq!(c["failed"][0]["bankTransactionId"], mauvaise, "{t}");
        assert_eq!(c["failed"][0]["errorCode"], "VALIDATION_ERROR", "{t}");
        assert_eq!(
            c["failed"][0]["details"]["reason"], "value_date_out_of_range",
            "{t}"
        );
        let apres: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM journal_entries")
            .fetch_one(&pool)
            .await
            .unwrap();
        assert_eq!(apres, avant + 1, "une seule écriture : {hors}");
        let statut: String =
            sqlx::query_scalar("SELECT status FROM bank_transactions WHERE id = ?")
                .bind(mauvaise)
                .fetch_one(&pool)
                .await
                .unwrap();
        assert_eq!(statut, "pending", "{hors}");
    }
}

/// Le message d'une date mal formée sur ce lot parle d'un champ mal formé, sans le
/// nommer, et non plus du seul `type` (`docs/api-external.md`, § 5).
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn c_rapprochement_lot_date_mal_formee(pool: MySqlPool) {
    let m = monter(&pool).await;
    let compte = compte_bancaire_unique(&m).await;
    let tx = m.transaction_du_jour(compte).await;
    let (s, t) = m
        .post(
            "/api/v1/reconciliation/accept",
            json!({"bankAccountId": compte, "proposals": [
                {"type": "split", "bankTransactionId": tx,
                 "splits": m.lignes_eclatement(), "valueDate": "2026-13-01"},
            ]}),
        )
        .await;
    assert_eq!(s, 400, "{t}");
    assert!(
        t.contains("VALIDATION_ERROR") && t.contains("champ mal formé"),
        "{t}"
    );
}

/// E-8 — `?dateFrom=` (vide) reste le refus d'une date mal formée, sans marqueur :
/// `Option::<NaiveDate>::deserialize` est ce qu'appelait le dérivé.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn q_date_vide_reste_mal_formee(pool: MySqlPool) {
    let m = monter(&pool).await;
    let (s, t) = m.get("/api/v1/invoices?dateFrom=").await;
    assert_eq!(s, 400, "{t}");
    assert!(!t.contains(MARQUEUR), "{t}");
}

/// Le témoin du lot : un `200` n'y prouve rien, `failed[]` en rend un aussi. La
/// proposition doit être **acceptée**, avec la date comme sans elle.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn c_rapprochement_lot_temoin_accepte(pool: MySqlPool) {
    let m = monter(&pool).await;
    let compte = compte_bancaire_unique(&m).await;
    for v in [Some(aujourd_hui()), None] {
        let tx = m.transaction_du_jour(compte).await;
        let proposition = avec(
            json!({"type": "split", "bankTransactionId": tx, "splits": m.lignes_eclatement()}),
            "valueDate",
            v,
        );
        let (s, t) = m
            .post(
                "/api/v1/reconciliation/accept",
                json!({"bankAccountId": compte, "proposals": [proposition]}),
            )
            .await;
        assert_eq!(s, 200, "{t}");
        let corps: Value = serde_json::from_str(&t).unwrap();
        assert_eq!(corps["accepted"].as_array().unwrap().len(), 1, "{t}");
        assert_eq!(corps["failed"].as_array().unwrap().len(), 0, "{t}");
    }
}

champ_corps!(
    c_rapprochement_manuel,
    "POST reconciliation/manual valueDate",
    B,
    &aujourd_hui(),
    facultatif,
    |m, v| {
        let compte = compte_bancaire_unique(m).await;
        let tx = m.transaction_du_jour(compte).await;
        m.post(
            "/api/v1/reconciliation/manual",
            avec(
                json!({"bankAccountId": compte, "bankTransactionId": tx,
                   "counterpartyAccountId": m.compte("4000"), "description": "Bornes"}),
                "valueDate",
                v,
            ),
        )
        .await
    }
);

champ_corps!(
    c_rapprochement_eclate,
    "POST reconciliation/split valueDate",
    B,
    &aujourd_hui(),
    facultatif,
    |m, v| {
        let compte = compte_bancaire_unique(m).await;
        let tx = m.transaction_du_jour(compte).await;
        m.post(
        "/api/v1/reconciliation/split",
        avec(
            json!({"bankAccountId": compte, "bankTransactionId": tx, "splits": m.lignes_eclatement()}),
            "valueDate",
            v,
        ),
    )
    .await
    }
);

champ_corps!(
    c_fournisseur_date,
    "POST supplier-invoices invoiceDate",
    B,
    &aujourd_hui(),
    |m, v| {
        let f = m.contact(true, None).await;
        m.post(
            "/api/v1/supplier-invoices",
            avec(
                json!({"contactId": f, "lines": m.lignes_fournisseur()}),
                "invoiceDate",
                v,
            ),
        )
        .await
    }
);

champ_corps!(
    c_fournisseur_echeance,
    "POST supplier-invoices dueDate",
    B,
    "2027-01-01",
    facultatif,
    |m, v| {
        let f = m.contact(true, None).await;
        m.post(
        "/api/v1/supplier-invoices",
        avec(
            json!({"contactId": f, "invoiceDate": aujourd_hui(), "lines": m.lignes_fournisseur()}),
            "dueDate",
            v,
        ),
    )
    .await
    }
);

champ_corps!(
    c_fournisseur_maj_date,
    "PUT supplier-invoices invoiceDate",
    B,
    &aujourd_hui(),
    |m, v| {
        let f = m.contact(true, None).await;
        let cree = m
        .post(
            "/api/v1/supplier-invoices",
            json!({"contactId": f, "invoiceDate": aujourd_hui(), "lines": m.lignes_fournisseur()}),
        )
        .await;
        let (id, version) = Monde::id_version(&cree, "brouillon");
        m.put(
            &format!("/api/v1/supplier-invoices/{id}"),
            avec(
                json!({"contactId": f, "lines": m.lignes_fournisseur(), "version": version}),
                "invoiceDate",
                v,
            ),
        )
        .await
    }
);

champ_corps!(
    c_fournisseur_maj_echeance,
    "PUT supplier-invoices dueDate",
    B,
    "2027-01-01",
    facultatif,
    |m, v| {
        let f = m.contact(true, None).await;
        let cree = m
        .post(
            "/api/v1/supplier-invoices",
            json!({"contactId": f, "invoiceDate": aujourd_hui(), "lines": m.lignes_fournisseur()}),
        )
        .await;
        let (id, version) = Monde::id_version(&cree, "brouillon");
        m.put(
        &format!("/api/v1/supplier-invoices/{id}"),
        avec(
            json!({"contactId": f, "invoiceDate": aujourd_hui(), "lines": m.lignes_fournisseur(), "version": version}),
            "dueDate",
            v,
        ),
    )
    .await
    }
);

champ_corps!(
    c_fournisseur_reglement,
    "POST supplier-invoices/pay paymentDate",
    B,
    &aujourd_hui(),
    |m, v| {
        let f = m.facture_fournisseur_validee().await;
        m.post(
            &format!("/api/v1/supplier-invoices/{f}/pay"),
            avec(
                json!({"settlementType": "internal_account", "accountId": m.compte("1000")}),
                "paymentDate",
                v,
            ),
        )
        .await
    }
);

champ_corps!(
    c_tva_debut,
    "POST vat-rates validFrom",
    B,
    "2030-01-01",
    |m, v| {
        m.post(
            "/api/v1/vat-rates",
            avec(
                json!({"category": format!("cat{}", seq()), "rate": taux()}),
                "validFrom",
                v,
            ),
        )
        .await
    }
);

champ_corps!(
    c_tva_fin,
    "POST vat-rates validTo",
    B,
    "2031-01-01",
    facultatif,
    |m, v| {
        m.post(
        "/api/v1/vat-rates",
        avec(
            json!({"category": format!("cat{}", seq()), "rate": taux(), "validFrom": "2030-01-01"}),
            "validTo",
            v,
        ),
    )
    .await
    }
);

champ_corps!(
    c_tva_maj_fin,
    "PUT vat-rates validTo",
    B,
    "2031-01-01",
    facultatif,
    |m, v| {
        let r = m
        .post(
            "/api/v1/vat-rates",
            json!({"category": format!("cat{}", seq()), "rate": taux(), "validFrom": "2030-01-01"}),
        )
        .await;
        let (id, version) = Monde::id_version(&r, "taux");
        m.put(
            &format!("/api/v1/vat-rates/{id}"),
            avec(json!({"active": true, "version": version}), "validTo", v),
        )
        .await
    }
);

/// `sent_at` (`NaiveDateTime`) : valeurs de son type (AC-2).
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn c_rappel_manuel_sent_at(pool: MySqlPool) {
    let m = monter(&pool).await;
    sqlx::query(
        "INSERT INTO dunning_levels (company_id, level_number, delay_days, fee_amount, version) \
         VALUES (?, 1, 10, 0, 0)",
    )
    .bind(m.company_id())
    .execute(&pool)
    .await
    .unwrap();
    let mut r = Releve::new("POST reminders/manual sentAt");
    for v in [
        "+10000-01-01T00:00:00",
        "+70000-01-01T00:00:00",
        "-0001-01-01T00:00:00",
        "0999-12-31T00:00:00",
    ] {
        let (f, _) = m.facture_validee().await;
        let rep = m
            .post(
                &format!("/api/v1/invoices/{f}/reminders/manual"),
                json!({"levelNumber": 1, "sentAt": v}),
            )
            .await;
        r.note(v, rep);
    }
    let (f, _) = m.facture_validee().await;
    let sent = (Utc::now().naive_utc() - TimeDelta::days(1))
        .format("%Y-%m-%dT%H:%M:%S")
        .to_string();
    r.temoin = Some(
        m.post(
            &format!("/api/v1/invoices/{f}/reminders/manual"),
            json!({"levelNumber": 1, "sentAt": sent}),
        )
        .await,
    );
    r.fin(BFutur);
}

// ---------------------------------------------------------------------------
// #755 (AC-8) — l'expiration d'une clé d'API
// ---------------------------------------------------------------------------

async fn nombre_de_cles(pool: &MySqlPool) -> i64 {
    sqlx::query_scalar("SELECT COUNT(*) FROM api_keys")
        .fetch_one(pool)
        .await
        .unwrap()
}

/// `9999-12-31T23:59:59-05:00` est en l'an 10000 en UTC : sans garde, la clé
/// voulue temporaire devenait permanente (#755).
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn cle_api_expiration_hors_plage_refusee(pool: MySqlPool) {
    let m = monter(&pool).await;
    let avant = nombre_de_cles(&pool).await;
    let (s, t) = m
        .post(
            "/api/v1/settings/api-keys",
            json!({"name": "Temporaire", "scope": "read", "expiresAt": "9999-12-31T23:59:59-05:00"}),
        )
        .await;
    eprintln!("RELEVE|POST api-keys expiresAt|9999-12-31T23:59:59-05:00|{s}|{t}");
    assert_eq!(s, 400, "{t}");
    assert_eq!(nombre_de_cles(&pool).await, avant, "rien n'est créé");

    let (s, t) = m
        .post(
            "/api/v1/settings/api-keys",
            json!({"name": "Temoin", "scope": "read", "expiresAt": "9999-12-31T23:59:59Z"}),
        )
        .await;
    eprintln!("TEMOIN|POST api-keys expiresAt|{s} {t}");
    assert_eq!(s, 201, "{t}");
    // La réponse de création ne porte pas l'expiration : la colonne fait foi.
    let v: Value = serde_json::from_str(&t).unwrap();
    let ecrite: Option<chrono::NaiveDateTime> =
        sqlx::query_scalar("SELECT expires_at FROM api_keys WHERE id = ?")
            .bind(v["id"].as_i64().unwrap())
            .fetch_one(&pool)
            .await
            .unwrap();
    assert!(ecrite.is_some(), "expires_at non nul : {t}");
}

// ---------------------------------------------------------------------------
// Date calculée : la fenêtre de rapprochement (validation P3, L-R4-2)
// ---------------------------------------------------------------------------

/// `reconciliation.rs:527-528` lie `d ± N jours` en SQL. Une transaction au
/// `9999-12-31`, qu'admettent le CAMT, le CSV et la saisie, donne une borne en
/// l'an 10000 ; une au `1000-01-01`, une borne en 0999. ⚠️ Ce test ne prouve que
/// l'absence de refus et de panique : il ne dit rien des candidats (C-e334-35).
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn fenetre_de_rapprochement_aux_bornes(pool: MySqlPool) {
    let m = monter(&pool).await;
    let compte = m.compte_bancaire().await;
    let _ = m.facture_validee().await;
    let _ = m.facture_fournisseur_validee().await;
    for (jour, montant) in [
        (NaiveDate::from_ymd_opt(9999, 12, 31).unwrap(), dec!(108.10)),
        (NaiveDate::from_ymd_opt(9999, 12, 30).unwrap(), dec!(-50.00)),
        (NaiveDate::from_ymd_opt(1000, 1, 1).unwrap(), dec!(108.10)),
        (NaiveDate::from_ymd_opt(1000, 1, 2).unwrap(), dec!(-50.00)),
    ] {
        m.transaction(compte, jour, montant).await;
    }
    let (s, t) = m
        .get(&format!(
            "/api/v1/reconciliation/proposals?bankAccountId={compte}"
        ))
        .await;
    eprintln!(
        "RELEVE|GET reconciliation/proposals|9999-12-31|{s}|{}",
        t.chars().take(200).collect::<String>()
    );
    assert_eq!(s, 200, "{t}");
}

// ---------------------------------------------------------------------------
// MCP (AC-5)
// ---------------------------------------------------------------------------

/// Les arguments directs : `ArgumentsInvalides` à message fixe, qui est aussi
/// celui d'un champ manquant : le jeu est donc complet et valide hors la seule
/// date fautive, et le témoin de forme passe.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn mcp_arguments_directs(pool: MySqlPool) {
    let a = societe_piegee(&pool, "Société bornes", 'A').await;
    let jeton = cle_de(&pool, &a, &LECTURE, SendValidation::Exempt).await;
    let b = banc(&pool);
    let fy = a.p.exercice_id;
    let cpt = a.p.compte_k1;
    let debut = a.p.exercice_debut.to_string();
    let fin = a.p.exercice_fin.to_string();
    let cas: Vec<(&str, &str, Value)> = vec![
        ("kesh_factures", "dateFrom", json!({})),
        ("kesh_factures", "dateTo", json!({})),
        (
            "kesh_echeancier",
            "dateFrom",
            json!({"paymentStatus": "all"}),
        ),
        ("kesh_echeancier", "dateTo", json!({"paymentStatus": "all"})),
        (
            "kesh_echeancier",
            "dueBefore",
            json!({"paymentStatus": "all"}),
        ),
        (
            "kesh_grand_livre",
            "from",
            json!({"accountId": cpt, "to": fin}),
        ),
        (
            "kesh_grand_livre",
            "to",
            json!({"accountId": cpt, "from": debut}),
        ),
        ("kesh_bilan", "periodStart", json!({"fiscalYearId": fy})),
        ("kesh_bilan", "periodEnd", json!({"fiscalYearId": fy})),
        ("kesh_balance", "periodStart", json!({"fiscalYearId": fy})),
        ("kesh_balance", "periodEnd", json!({"fiscalYearId": fy})),
    ];
    let attendu = attendu_outil_en_erreur(1, TEXTE_ARGUMENTS_INVALIDES);
    let mut fautes = Vec::new();
    let mut vus = HashSet::new();
    for (outil, champ, base) in cas {
        vus.insert((outil, champ));
        for v in HORS_PLAGE {
            let mut args = base.clone();
            args[champ] = json!(v);
            let r = b.appeler(&jeton, outil, args).await;
            eprintln!(
                "RELEVE|MCP {outil} {champ}|{v}|-|{}",
                r.to_string().chars().take(140).collect::<String>()
            );
            if r != attendu {
                fautes.push(format!("{outil} {champ} {v} : {r}"));
            }
        }
        let mut args = base.clone();
        args[champ] = json!(if champ == "from" {
            debut.clone()
        } else {
            fin.clone()
        });
        let r = b.appeler(&jeton, outil, args).await;
        eprintln!(
            "TEMOIN|MCP {outil} {champ}|{}",
            r.to_string().chars().take(140).collect::<String>()
        );
        if r["result"]["isError"] == json!(true) || r.get("error").is_some() {
            fautes.push(format!("témoin {outil} {champ} : {r}"));
        }
    }
    assert_eq!(vus.len(), 11, "les onze arguments directs");
    assert!(fautes.is_empty(), "{}", fautes.join("\n"));
}

/// `kesh_ecritures` et `kesh_postes_ouverts` : le refus du service, que
/// `depuis_app_error` traduit (`mcp/textes.rs:315`, `AppError::Validation` →
/// arguments invalides, texte fixe). Le témoin de forme passe sur les mêmes
/// arguments : le refus vient bien de la date.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn mcp_arguments_par_le_service(pool: MySqlPool) {
    let a = societe_piegee(&pool, "Société bornes", 'A').await;
    let jeton = cle_de(&pool, &a, &LECTURE, SendValidation::Exempt).await;
    let b = banc(&pool);
    let cpt = a.p.compte_k1;
    let mut fautes = Vec::new();
    for (outil, champ, base) in [
        ("kesh_ecritures", "dateFrom", json!({"accountId": cpt})),
        ("kesh_ecritures", "dateTo", json!({"accountId": cpt})),
        ("kesh_postes_ouverts", "asOf", json!({"accountId": cpt})),
    ] {
        for v in HORS_PLAGE {
            let mut args = base.clone();
            args[champ] = json!(v);
            let r = b.appeler(&jeton, outil, args).await;
            eprintln!(
                "RELEVE|MCP {outil} {champ}|{v}|-|{}",
                r.to_string().chars().take(160).collect::<String>()
            );
            if r != attendu_outil_en_erreur(1, TEXTE_ARGUMENTS_INVALIDES) {
                fautes.push(format!("{outil} {champ} {v} : {r}"));
            }
        }
        let mut args = base.clone();
        args[champ] = json!(a.p.exercice_fin.to_string());
        let r = b.appeler(&jeton, outil, args).await;
        if r["result"]["isError"] == json!(true) || r.get("error").is_some() {
            fautes.push(format!("témoin {outil} {champ} : {r}"));
        }
    }
    assert!(fautes.is_empty(), "{}", fautes.join("\n"));
}

// ---------------------------------------------------------------------------
// AC-4 — les deux bornes ne sont pas refusées pour leur plage, un site par canal
// ---------------------------------------------------------------------------

#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn bornes_acceptees_sur_chaque_canal(pool: MySqlPool) {
    let m = monter(&pool).await;
    for jour in ["1000-01-01", "9999-12-31"] {
        // Requête, classe C.
        let (s, t) = m.get(&format!("/api/v1/invoices?dateFrom={jour}")).await;
        assert_eq!(s, 200, "{jour} : {t}");
        // Requête, chaîne analysée.
        let (s, t) = m
            .get(&format!("/api/v1/journal-entries?dateTo={jour}"))
            .await;
        assert_eq!(s, 200, "{jour} : {t}");
        // Corps, classe C.
        let (s, t) = m
            .post(
                "/api/v1/projects",
                json!({"code": format!("P{}", seq()), "name": "Projet", "startDate": jour}),
            )
            .await;
        assert_eq!(s, 201, "{jour} : {t}");
        // Corps, classe B : la date passe la borne ; le refus qui reste n'est
        // pas celui de la plage.
        let (s, t) = m
            .post(
                "/api/v1/journal-entries",
                json!({"entryDate": jour, "journal": "OD", "description": "Bornes",
                       "lines": m.lignes_ecriture()}),
            )
            .await;
        assert!(!t.contains(MARQUEUR), "{jour} : {s} {t}");
        assert_eq!(s, 400, "pas d'exercice à cette date : {t}");
        assert!(t.contains("NO_FISCAL_YEAR"), "{t}");
    }
    // MCP, argument direct : le filtre lit et ne refuse pas.
    let a = societe_piegee(&pool, "Société bornes", 'A').await;
    let jeton = cle_de(&pool, &a, &LECTURE, SendValidation::Exempt).await;
    let b = banc(&pool);
    for jour in ["1000-01-01", "9999-12-31"] {
        let r = b
            .appeler(&jeton, "kesh_factures", json!({"dateFrom": jour}))
            .await;
        assert_ne!(r["result"]["isError"], json!(true), "{jour} : {r}");
    }
}

/// Le test croisé (C-e334-2) : `kesh-import`, qui n'a aucune dépendance interne,
/// porte sa propre copie de la plage ; les deux ne divergent pas.
#[test]
fn la_plage_de_kesh_import_est_celle_de_kesh_core() {
    assert_eq!(kesh_import::ANNEE_MIN, kesh_core::dates::ANNEE_MIN);
    assert_eq!(kesh_import::ANNEE_MAX, kesh_core::dates::ANNEE_MAX);
}

// ---------------------------------------------------------------------------
// AC-13 — l'échéance calculée
// ---------------------------------------------------------------------------

/// `9999-12-31` + le délai du contact tomberait en l'an 10000, que
/// `checked_add_signed` laisse passer et que la base écrirait `NULL`.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn echeance_calculee_hors_plage(pool: MySqlPool) {
    let m = monter(&pool).await;
    let avec_delai = m.contact(false, Some(30)).await;
    let avant: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM invoices")
        .fetch_one(&pool)
        .await
        .unwrap();
    let (s, t) = m
        .post(
            "/api/v1/invoices",
            json!({"contactId": avec_delai, "date": "9999-12-31", "lines": Monde::lignes_facture()}),
        )
        .await;
    eprintln!("RELEVE|POST invoices échéance calculée|9999-12-31|{s}|{t}");
    assert_eq!(s, 400, "{t}");
    assert!(
        t.contains("VALIDATION_ERROR") && t.contains(MARQUEUR),
        "{t}"
    );
    let apres: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM invoices")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(avant, apres, "rien d'écrit");

    // Sans délai au contact, la même date n'est pas refusée pour sa plage.
    let sans_delai = m.contact(false, None).await;
    let (s, t) = m
        .post(
            "/api/v1/invoices",
            json!({"contactId": sans_delai, "date": "9999-12-31", "lines": Monde::lignes_facture()}),
        )
        .await;
    assert_eq!(s, 201, "{t}");
}
