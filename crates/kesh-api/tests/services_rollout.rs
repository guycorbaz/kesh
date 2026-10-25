//! Story 32-4a — le rollout de la couche de services aux lectures des
//! écritures comptables, des postes ouverts et du carnet d'adresses.
//!
//! Ce fichier porte, pour les cinq services du rollout
//! (`services::{ecritures, postes_ouverts, carnet}`) : les tests de refus et
//! leur contrôle positif (AC9), la validation et les bornes de pagination au
//! service et par la route (AC10), la lecture seule et le filtre de société
//! (AC11). La garde lexicale reste dans `tests/services_motif.rs` (AC12).
//! La 32-4b y ajoute ses tests, sans le recréer (C-32-4a-9).
//!
//! Fixtures **synthétiques** : `seed_accounting_company`, une écriture sur deux
//! comptes lettrables (1000 et 2000 — aucun compte bancaire n'y est lié dans le
//! seed), un contact ; et, pour le chemin de succès des filtres et le tri par
//! défaut du carnet (revue de code P1), trois écritures et quatre contacts que
//! chaque filtre et chaque colonne de tri séparent.

mod common;

use std::collections::BTreeSet;
use std::net::SocketAddr;
use std::path::Path;
use std::sync::Arc;

use chrono::{NaiveDate, TimeDelta, Utc};
use jsonwebtoken::{Algorithm, EncodingKey, Header};
use kesh_api::auth::jwt::Claims;
use kesh_api::config::Config;
use kesh_api::errors::AppError;
use kesh_api::middleware::auth::CurrentUser;
use kesh_api::services::acces::{ContexteAcces, Portee};
use kesh_api::services::carnet::{self, DemandeListeContacts};
use kesh_api::services::ecritures::{self, DemandeListeEcritures};
use kesh_api::services::poignees::Lecture;
use kesh_api::services::postes_ouverts::{self, DemandePostesOuverts};
use kesh_api::{AppState, build_router};
use kesh_db::entities::address::StructuredAddress;
use kesh_db::entities::invoice::{NewInvoice, NewInvoiceLine};
use kesh_db::entities::journal_entry::Journal;
use kesh_db::entities::{
    Language, NewCompany, NewJournalEntry, NewJournalEntryLine, NewSupplierInvoice,
    NewSupplierInvoiceLine, OrgType, Role, SettlementChoice,
};
use kesh_db::errors::DbError;
use kesh_db::repositories::{
    companies, invoice_settlements_write, invoices, journal_entries, supplier_invoices,
};
use kesh_db::test_fixtures::{SeededCompany, seed_accounting_company};
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use serde_json::Value;
use sqlx::MySqlPool;

use common::lecture_seule::{lecture_seule, pool_lecture_seule};

const TEST_JWT_SECRET: &[u8] = b"test-secret-32-bytes-minimum-test-secret-padding";
const TEST_ADMIN_PASSWORD: &str = "e2e-test-admin-password";

// ============================================================
// Montage HTTP (recopié de `services_motif.rs:43-141`, sans les bornes de
// période des rapports — C-32-4a-9, convention du dépôt)
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

/// `GET` authentifié : statut et corps JSON.
async fn get_json(app: &TestApp, jwt: &str, chemin: &str) -> (u16, Value) {
    let resp = app
        .client
        .get(app.url(chemin))
        .bearer_auth(jwt)
        .send()
        .await
        .unwrap();
    let statut = resp.status().as_u16();
    let corps = resp.json::<Value>().await.unwrap_or(Value::Null);
    (statut, corps)
}

// ============================================================
// Fixtures synthétiques
// ============================================================

async fn societe(pool: &MySqlPool) -> SeededCompany {
    seed_accounting_company(pool).await.expect("seed")
}

/// Les livres du rollout : la société seedée, une écriture (débit 1000, crédit
/// 2000, deux comptes lettrables), un contact ; et, pour la 32-4b (C-32-4b-15),
/// un brouillon, une facture validée partiellement réglée et une facture
/// fournisseur, posés par les dépôts.
struct Livres {
    s: SeededCompany,
    ecriture_id: i64,
    contact_id: i64,
    /// Un brouillon (le réglage d'arrondi est lu pour lui).
    brouillon_id: i64,
    /// Une facture validée portant un règlement partiel (50.00 sur 108.10).
    facture_reglee_id: i64,
    /// Une facture fournisseur ouverte.
    facture_fournisseur_id: i64,
}

impl Livres {
    fn compte(&self, numero: &str) -> i64 {
        self.s.accounts[numero]
    }
}

async fn poser_livres(pool: &MySqlPool) -> Livres {
    let s = societe(pool).await;
    let ecriture = journal_entries::create(
        pool,
        s.fiscal_year_id,
        s.admin_user_id,
        NewJournalEntry {
            company_id: s.company_id,
            entry_date: NaiveDate::from_ymd_opt(2026, 6, 1).unwrap(),
            journal: Journal::Achats,
            description: "Écriture synthétique".into(),
            project_id: None,
            lines: vec![
                NewJournalEntryLine {
                    account_id: s.accounts["1000"],
                    debit: dec!(100),
                    credit: Decimal::ZERO,
                    project_id: None,
                },
                NewJournalEntryLine {
                    account_id: s.accounts["2000"],
                    debit: Decimal::ZERO,
                    credit: dec!(100),
                    project_id: None,
                },
            ],
        },
    )
    .await
    .expect("écriture synthétique");
    let contact_id = sqlx::query(
        "INSERT INTO contacts (company_id, contact_type, name, is_client, is_supplier) \
         VALUES (?, 'Entreprise', 'Contact synthétique SA', TRUE, FALSE)",
    )
    .bind(s.company_id)
    .execute(pool)
    .await
    .expect("contact synthétique")
    .last_insert_id() as i64;
    let date = NaiveDate::from_ymd_opt(2026, 6, 1).unwrap();
    let brouillon_id = poser_facture(pool, &s, contact_id, date, date, dec!(100), false).await;
    let facture_reglee_id = poser_facture(pool, &s, contact_id, date, date, dec!(100), true).await;
    regler(pool, &s, facture_reglee_id, dec!(50.00), date).await;
    let fournisseur =
        poser_contact_fournisseur(pool, s.company_id, "Fournisseur synthétique SA").await;
    let facture_fournisseur_id =
        poser_facture_fournisseur(pool, &s, fournisseur, date, "FF-1").await;
    Livres {
        ecriture_id: ecriture.entry.id,
        contact_id,
        brouillon_id,
        facture_reglee_id,
        facture_fournisseur_id,
        s,
    }
}

/// Une facture synthétique d'une ligne (`montant` HT à 8.10 %, TTC multiple de
/// 0.05 pour les montants employés ici : le seed laisse l'arrondi à 5 centimes
/// actif sans compte d'arrondi) ; validée par le dépôt si `valider`. Rend son id.
async fn poser_facture(
    pool: &MySqlPool,
    s: &SeededCompany,
    contact_id: i64,
    date: NaiveDate,
    echeance: NaiveDate,
    montant: Decimal,
    valider: bool,
) -> i64 {
    let (facture, _) = invoices::create(
        pool,
        s.admin_user_id,
        NewInvoice {
            company_id: s.company_id,
            contact_id,
            date,
            due_date: Some(echeance),
            payment_terms: None,
            lines: vec![NewInvoiceLine {
                revenue_account_id: None,
                description: "Prestation synthétique".into(),
                quantity: dec!(1),
                unit_price: montant,
                vat_rate: dec!(8.10),
            }],
            project_id: None,
        },
    )
    .await
    .expect("facture synthétique");
    if valider {
        invoices::validate_invoice(pool, s.company_id, facture.id, s.admin_user_id)
            .await
            .expect("validation synthétique");
    }
    facture.id
}

/// Un règlement en compte interne (1000) de `montant`, au `jour`.
async fn regler(
    pool: &MySqlPool,
    s: &SeededCompany,
    facture_id: i64,
    montant: Decimal,
    jour: NaiveDate,
) {
    invoice_settlements_write::settle_invoice(
        pool,
        s.admin_user_id,
        s.company_id,
        facture_id,
        SettlementChoice::InternalAccount {
            account_id: s.accounts["1000"],
        },
        montant,
        jour,
    )
    .await
    .expect("règlement synthétique");
}

/// Un contact client synthétique ; rend son id.
async fn poser_contact_client(pool: &MySqlPool, company_id: i64, nom: &str) -> i64 {
    sqlx::query(
        "INSERT INTO contacts (company_id, contact_type, name, is_client, is_supplier) \
         VALUES (?, 'Entreprise', ?, TRUE, FALSE)",
    )
    .bind(company_id)
    .bind(nom)
    .execute(pool)
    .await
    .expect("client synthétique")
    .last_insert_id() as i64
}

/// Un contact fournisseur synthétique ; rend son id.
async fn poser_contact_fournisseur(pool: &MySqlPool, company_id: i64, nom: &str) -> i64 {
    sqlx::query(
        "INSERT INTO contacts (company_id, contact_type, name, is_client, is_supplier) \
         VALUES (?, 'Entreprise', ?, FALSE, TRUE)",
    )
    .bind(company_id)
    .bind(nom)
    .execute(pool)
    .await
    .expect("fournisseur synthétique")
    .last_insert_id() as i64
}

/// Une facture fournisseur de 100.00 (TVA 0), ouverte, au patron de
/// `supplier_settlement_cancel_e2e.rs` (compte fournisseurs par défaut posé à
/// 2000, que le seed ne désigne pas) ; rend son id.
async fn poser_facture_fournisseur(
    pool: &MySqlPool,
    s: &SeededCompany,
    fournisseur: i64,
    date: NaiveDate,
    numero: &str,
) -> i64 {
    sqlx::query(
        "UPDATE company_invoice_settings SET default_payable_account_id = ? WHERE company_id = ?",
    )
    .bind(s.accounts["2000"])
    .bind(s.company_id)
    .execute(pool)
    .await
    .expect("compte fournisseurs");
    supplier_invoices::create(
        pool,
        NewSupplierInvoice {
            company_id: s.company_id,
            contact_id: fournisseur,
            supplier_invoice_number: Some(numero.into()),
            invoice_date: date,
            due_date: None,
            creditor_iban: None,
            creditor_qr_iban: None,
            payment_reference: None,
            expected_payment_amount: None,
            project_id: None,
            lines: vec![NewSupplierInvoiceLine {
                description: "Prestation synthétique".into(),
                quantity: dec!(1),
                unit_price: dec!(100.00),
                vat_rate: dec!(0),
                expense_account_id: s.accounts["4000"],
            }],
        },
        s.admin_user_id,
    )
    .await
    .expect("facture fournisseur synthétique")
    .invoice
    .id
}

/// Une seconde société, sans utilisateur ni livres (AC11 (b)) — par
/// `companies::create`, comme `common::create_test_company` : un second
/// `seed_accounting_company` heurterait l'unicité de `users.username`.
async fn seconde_societe(pool: &MySqlPool) -> i64 {
    companies::create(
        pool,
        NewCompany {
            name: "Seconde société synthétique".into(),
            first_name: None,
            last_name: None,
            address_structured: StructuredAddress {
                street: "Rue Synthétique".into(),
                building: "2".into(),
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
    .expect("seconde société")
    .id
}

// ============================================================
// Contextes (recopiés de `services_motif.rs:830-883`, C-32-4a-9)
// ============================================================

/// Un id positif qu'aucune fixture ne pose.
const INEXISTANT: i64 = 999_999_999;

/// Les trois portées.
const TOUTES: [Portee; 3] = [
    Portee::BooksRead,
    Portee::InvoicesRead,
    Portee::ContactsRead,
];

fn mcp_societe(company_id: i64, user_id: i64, portees: &[Portee]) -> ContexteAcces {
    ContexteAcces::mcp(
        company_id,
        user_id,
        portees.iter().copied().collect::<BTreeSet<_>>(),
    )
}

fn api_societe(company_id: i64, user_id: i64, role: Role) -> ContexteAcces {
    ContexteAcces::depuis_api(&CurrentUser {
        user_id,
        role,
        company_id,
        exp: i64::MAX,
        api_key_id: None,
    })
}

// ============================================================
// AC10 (c) — les bornes et un refus, par la route
// ============================================================

/// AC10 (c) — bornes des contacts et des écritures, et un refus de validation,
/// par la route. Joué **avant** le déplacement (T1 (i)) : il décrit le code
/// d'aujourd'hui, puis tient le code déplacé.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn ac10_c_bornes_et_refus_par_la_route(pool: MySqlPool) {
    let s = societe(&pool).await;
    let app = spawn_app(pool.clone()).await;
    let jwt = forge_jwt(s.admin_user_id, "Admin", s.company_id);

    let mut echecs = Vec::new();
    for (chemin, limit, offset) in [
        ("/api/v1/contacts?limit=1000&offset=-3", 100, 0),
        ("/api/v1/contacts", 20, 0),
        ("/api/v1/journal-entries?limit=0", 1, 0),
        ("/api/v1/journal-entries", 50, 0),
    ] {
        let (statut, corps) = get_json(&app, &jwt, chemin).await;
        if statut != 200 || corps["limit"] != limit || corps["offset"] != offset {
            echecs.push(format!(
                "{chemin} : attendu 200, limit {limit}, offset {offset} ; rendu {statut}, limit {}, offset {}",
                corps["limit"], corps["offset"]
            ));
        }
    }
    assert!(
        echecs.is_empty(),
        "AC10 (c) bornes :\n{}",
        echecs.join("\n")
    );

    let (statut, corps) = get_json(&app, &jwt, "/api/v1/journal-entries?dateFrom=x").await;
    assert_eq!(statut, 400, "AC10 (c) dateFrom=x : {corps}");
    assert_eq!(corps["error"]["code"], "VALIDATION_ERROR", "{corps}");
    let message = corps["error"]["message"].as_str().unwrap_or_default();
    assert!(
        message.starts_with("dateFrom invalide ("),
        "AC10 (c) message : {message}"
    );
}

// ============================================================
// AC11 (c) — ce que la `Lecture` de test fait d'un verrou
// ============================================================

/// Numéro d'erreur MariaDB d'une erreur sqlx, s'il y en a un.
fn numero_mariadb(e: &sqlx::Error) -> Option<u16> {
    e.as_database_error()
        .and_then(|d| d.try_downcast_ref::<sqlx::mysql::MySqlDatabaseError>())
        .map(|m| m.number())
}

/// AC11 (c) — mesure, puis témoin : sur `pool_lecture_seule` (session
/// `SET SESSION TRANSACTION READ ONLY`), un `SELECT … FOR UPDATE` est refusé
/// (1792), en autocommit **et** dans `begin()` — au patron de la T0 de la
/// 29-1 (`kesh-db/tests/closing_checks.rs`), qui l'avait mesuré pour
/// `START TRANSACTION … READ ONLY`. C'est ce qui permet à l'AC11 (a) de
/// confirmer aussi l'absence de verrou sur les branches exercées.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn ac11_c_un_verrou_est_refuse_sur_la_lecture_de_test(pool: MySqlPool) {
    societe(&pool).await;
    let ro = pool_lecture_seule(&pool).await;
    let sql = "SELECT 1 FROM fiscal_years LIMIT 1 FOR UPDATE";

    let autocommit = sqlx::query(sql).fetch_optional(&ro).await;
    let mut tx = ro.begin().await.expect("begin sur la lecture seule");
    let en_transaction = sqlx::query(sql).fetch_optional(&mut *tx).await;
    let _ = tx.rollback().await;

    let autocommit = autocommit.err().as_ref().and_then(numero_mariadb);
    let en_transaction = en_transaction.err().as_ref().and_then(numero_mariadb);
    assert_eq!(
        (autocommit, en_transaction),
        (Some(1792), Some(1792)),
        "AC11 (c) : FOR UPDATE en autocommit, puis dans begin()"
    );
}

// ============================================================
// Les cinq services, joués de façon uniforme
// ============================================================

/// Les cinq services du rollout.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Svc {
    EcrituresLister,
    EcrituresDetail,
    PostesOuvertsLister,
    CarnetLister,
    CarnetDetail,
}

const SERVICES: [Svc; 5] = [
    Svc::EcrituresLister,
    Svc::EcrituresDetail,
    Svc::PostesOuvertsLister,
    Svc::CarnetLister,
    Svc::CarnetDetail,
];

/// Portée qu'exige un service.
fn portee_exigee(svc: Svc) -> Portee {
    match svc {
        Svc::CarnetLister | Svc::CarnetDetail => Portee::ContactsRead,
        _ => Portee::BooksRead,
    }
}

/// La demande d'un jeu.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Cas {
    /// Une demande qui réussit.
    Normal,
    /// Une demande qui échoue à l'analyse (`dateFrom = "x"`, `asOf = "2026-13-01"`).
    Invalide,
    /// Une demande qui nomme une entité inexistante (écriture, compte, contact).
    Inexistant,
}

/// Joue `svc` ; rend l'erreur, ou `Ok(())`.
async fn jouer(
    svc: Svc,
    acces: &ContexteAcces,
    lecture: &Lecture,
    livres: &Livres,
    cas: Cas,
) -> Result<(), AppError> {
    let id = |existant: i64| {
        if cas == Cas::Inexistant {
            INEXISTANT
        } else {
            existant
        }
    };
    match svc {
        Svc::EcrituresLister => {
            let demande = DemandeListeEcritures {
                date_from: (cas == Cas::Invalide).then(|| "x".to_string()),
                ..Default::default()
            };
            ecritures::lister(acces, lecture, demande).await.map(drop)
        }
        Svc::EcrituresDetail => ecritures::detail(acces, lecture, id(livres.ecriture_id))
            .await
            .map(drop),
        Svc::PostesOuvertsLister => {
            let demande = DemandePostesOuverts {
                account_id: id(livres.compte("1000")),
                as_of: (cas == Cas::Invalide).then(|| "2026-13-01".to_string()),
                ..Default::default()
            };
            postes_ouverts::lister(acces, lecture, demande)
                .await
                .map(drop)
        }
        Svc::CarnetLister => carnet::lister(acces, lecture, DemandeListeContacts::default())
            .await
            .map(drop),
        Svc::CarnetDetail => carnet::detail(acces, lecture, id(livres.contact_id))
            .await
            .map(drop),
    }
}

/// Résumé d'un résultat, pour les messages.
fn resume(r: &Result<(), AppError>) -> String {
    match r {
        Ok(()) => "Ok".into(),
        Err(e) => format!("Err({e:?})"),
    }
}

fn est_interdit(r: &Result<(), AppError>) -> bool {
    matches!(r, Err(AppError::Forbidden))
}

fn est_introuvable(r: &Result<(), AppError>) -> bool {
    matches!(r, Err(AppError::Database(DbError::NotFound)))
}

fn est_interne(r: &Result<(), AppError>) -> bool {
    matches!(r, Err(AppError::Internal(_)))
}

fn est_validation(r: &Result<(), AppError>) -> bool {
    matches!(r, Err(AppError::Validation(_)))
}

// ============================================================
// AC9 — refus, contrôle positif, canal API
// ============================================================

/// AC9 (a), (b), (e) — pour chacun des cinq services : MCP sans portée, puis
/// avec chacune des deux autres portées seule → `Forbidden` ; MCP avec la
/// portée exigée → `Ok` ; API `Consultation` → `Ok`.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn ac9_a_b_e_refus_et_controle_positif(pool: MySqlPool) {
    let livres = poser_livres(&pool).await;
    let lecture = Lecture::new(pool.clone());
    let (c, u) = (livres.s.company_id, livres.s.admin_user_id);
    let mut echecs = Vec::new();
    for svc in SERVICES {
        let exigee = portee_exigee(svc);
        let r = jouer(svc, &mcp_societe(c, u, &[]), &lecture, &livres, Cas::Normal).await;
        if !est_interdit(&r) {
            echecs.push(format!("AC9 (a) {svc:?} sans portée : {}", resume(&r)));
        }
        for autre in TOUTES.into_iter().filter(|p| *p != exigee) {
            let r = jouer(
                svc,
                &mcp_societe(c, u, &[autre]),
                &lecture,
                &livres,
                Cas::Normal,
            )
            .await;
            if !est_interdit(&r) {
                echecs.push(format!(
                    "AC9 (a) {svc:?} avec {autre:?} seule : {}",
                    resume(&r)
                ));
            }
        }
        let r = jouer(
            svc,
            &mcp_societe(c, u, &[exigee]),
            &lecture,
            &livres,
            Cas::Normal,
        )
        .await;
        if r.is_err() {
            echecs.push(format!("AC9 (b) {svc:?} avec {exigee:?} : {}", resume(&r)));
        }
        let r = jouer(
            svc,
            &api_societe(c, u, Role::Consultation),
            &lecture,
            &livres,
            Cas::Normal,
        )
        .await;
        if r.is_err() {
            echecs.push(format!("AC9 (e) {svc:?} API Consultation : {}", resume(&r)));
        }
    }
    assert!(echecs.is_empty(), "{}", echecs.join("\n"));
}

/// AC9 (c) — le refus précède la validation (c-val) et la lecture (c-lec) :
/// MCP sans portée → `Forbidden` ; le même cas avec la portée rend l'erreur
/// attendue (témoin du cas). Sans objet : (c-lec) pour `ecritures::lister`
/// (aucune entité nommée par la demande n'est lue avant la page), (c-val) pour
/// `ecritures::detail`, `carnet::lister`, `carnet::detail` (leur demande ne
/// porte rien qui échoue à l'analyse).
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn ac9_c_le_refus_precede_validation_et_lecture(pool: MySqlPool) {
    let livres = poser_livres(&pool).await;
    let lecture = Lecture::new(pool.clone());
    let (c, u) = (livres.s.company_id, livres.s.admin_user_id);
    type Attendu = fn(&Result<(), AppError>) -> bool;
    // (nom, service, société du contexte, cas, erreur attendue avec la portée)
    let table: [(&str, Svc, i64, Cas, Attendu); 8] = [
        (
            "c-val ecritures::lister dateFrom=x",
            Svc::EcrituresLister,
            c,
            Cas::Invalide,
            est_validation,
        ),
        (
            "c-val postes_ouverts::lister asOf=2026-13-01",
            Svc::PostesOuvertsLister,
            c,
            Cas::Invalide,
            est_validation,
        ),
        (
            "c-lec ecritures::detail id inexistant",
            Svc::EcrituresDetail,
            c,
            Cas::Inexistant,
            est_introuvable,
        ),
        (
            "c-lec carnet::detail id inexistant",
            Svc::CarnetDetail,
            c,
            Cas::Inexistant,
            est_introuvable,
        ),
        (
            "c-lec postes_ouverts::lister compte inexistant",
            Svc::PostesOuvertsLister,
            c,
            Cas::Inexistant,
            est_introuvable,
        ),
        (
            "c-lec ecritures::detail société inexistante",
            Svc::EcrituresDetail,
            INEXISTANT,
            Cas::Normal,
            est_interne,
        ),
        (
            "c-lec postes_ouverts::lister société inexistante",
            Svc::PostesOuvertsLister,
            INEXISTANT,
            Cas::Normal,
            est_interne,
        ),
        (
            "c-lec carnet::lister société inexistante",
            Svc::CarnetLister,
            INEXISTANT,
            Cas::Normal,
            est_interne,
        ),
    ];
    let mut echecs = Vec::new();
    for (nom, svc, societe, cas, attendu) in table {
        let r = jouer(svc, &mcp_societe(societe, u, &[]), &lecture, &livres, cas).await;
        if !est_interdit(&r) {
            echecs.push(format!("AC9 {nom}, sans portée : {}", resume(&r)));
        }
        let exigee = portee_exigee(svc);
        let r = jouer(
            svc,
            &mcp_societe(societe, u, &[exigee]),
            &lecture,
            &livres,
            cas,
        )
        .await;
        if !attendu(&r) {
            echecs.push(format!(
                "AC9 {nom}, témoin avec {exigee:?} : {}",
                resume(&r)
            ));
        }
    }
    assert!(echecs.is_empty(), "{}", echecs.join("\n"));
}

/// AC9 — l'ordre de `carnet::detail` : le contact est lu avant la société.
/// Société inexistante **et** contact inexistant → `NotFound`, quand
/// `carnet::lister` sur la même société rend `Internal`.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn ac9_carnet_detail_lit_le_contact_avant_la_societe(pool: MySqlPool) {
    let livres = poser_livres(&pool).await;
    let lecture = Lecture::new(pool.clone());
    let acces = mcp_societe(INEXISTANT, livres.s.admin_user_id, &[Portee::ContactsRead]);
    let detail = jouer(
        Svc::CarnetDetail,
        &acces,
        &lecture,
        &livres,
        Cas::Inexistant,
    )
    .await;
    assert!(
        est_introuvable(&detail),
        "AC9 ordre carnet::detail : contact d'abord → NotFound, rendu {}",
        resume(&detail)
    );
    let liste = jouer(Svc::CarnetLister, &acces, &lecture, &livres, Cas::Normal).await;
    assert!(
        est_interne(&liste),
        "AC9 ordre carnet::lister : société d'abord → Internal, rendu {}",
        resume(&liste)
    );
}

// ============================================================
// AC10 — la validation et les bornes, au service
// ============================================================

/// Le message d'une erreur de validation, ou le résumé de ce qui a été rendu.
fn message_de_validation<T: std::fmt::Debug>(r: &Result<T, AppError>) -> Result<String, String> {
    match r {
        Err(AppError::Validation(m)) => Ok(m.clone()),
        autre => Err(format!("{autre:?}")),
    }
}

/// AC10 (a) — les neuf refus de `ecritures::lister`, dans l'ordre d'avant la
/// 32-4a, messages en entier (les quatre erreurs d'analyse comparées jusqu'à la
/// parenthèse ouvrante incluse) ; deux cas d'ordre (date avant `accountId`,
/// montants négatifs avant le contrôle croisé) ; un `accountId` négatif ; une
/// description blanche. Le chemin de succès des filtres et l'égalité des
/// bornes : `filtres_des_ecritures_par_la_route`.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn ac10_a_les_neuf_refus_de_la_liste_des_ecritures(pool: MySqlPool) {
    let livres = poser_livres(&pool).await;
    let lecture = Lecture::new(pool.clone());
    let acces = api_societe(
        livres.s.company_id,
        livres.s.admin_user_id,
        Role::Consultation,
    );
    let s = |v: &str| Some(v.to_string());
    let d = DemandeListeEcritures::default;
    // (nom, demande, message attendu, comparaison par préfixe)
    let table: Vec<(&str, DemandeListeEcritures, &str, bool)> = vec![
        (
            "1 dateFrom",
            DemandeListeEcritures {
                date_from: s("x"),
                ..d()
            },
            "dateFrom invalide (",
            true,
        ),
        (
            "2 dateTo",
            DemandeListeEcritures {
                date_to: s("x"),
                ..d()
            },
            "dateTo invalide (",
            true,
        ),
        (
            "3 amountMin",
            DemandeListeEcritures {
                amount_min: s("x"),
                ..d()
            },
            "amountMin invalide (",
            true,
        ),
        (
            "4 amountMax",
            DemandeListeEcritures {
                amount_max: s("x"),
                ..d()
            },
            "amountMax invalide (",
            true,
        ),
        (
            "5 amountMin négatif",
            DemandeListeEcritures {
                amount_min: s("-1"),
                ..d()
            },
            "amountMin ne peut pas être négatif",
            false,
        ),
        (
            "6 amountMax négatif",
            DemandeListeEcritures {
                amount_max: s("-1"),
                ..d()
            },
            "amountMax ne peut pas être négatif",
            false,
        ),
        (
            "7 amountMin > amountMax",
            DemandeListeEcritures {
                amount_min: s("10"),
                amount_max: s("5"),
                ..d()
            },
            "amountMin doit être inférieur ou égal à amountMax",
            false,
        ),
        (
            "8 dateFrom > dateTo",
            DemandeListeEcritures {
                date_from: s("2026-02-01"),
                date_to: s("2026-01-01"),
                ..d()
            },
            "dateFrom doit être inférieur ou égal à dateTo",
            false,
        ),
        (
            "9 accountId",
            DemandeListeEcritures {
                account_id: Some(0),
                ..d()
            },
            "accountId doit être un identifiant positif",
            false,
        ),
        (
            "ordre : dateFrom=x et accountId=0",
            DemandeListeEcritures {
                date_from: s("x"),
                account_id: Some(0),
                ..d()
            },
            "dateFrom invalide (",
            true,
        ),
        // Revue de code P1, B2 : un identifiant négatif (pas seulement 0), et
        // les montants négatifs contrôlés avant le contrôle croisé.
        (
            "9 bis accountId négatif",
            DemandeListeEcritures {
                account_id: Some(-5),
                ..d()
            },
            "accountId doit être un identifiant positif",
            false,
        ),
        (
            "ordre : amountMin=10 et amountMax=-5",
            DemandeListeEcritures {
                amount_min: s("10"),
                amount_max: s("-5"),
                ..d()
            },
            "amountMax ne peut pas être négatif",
            false,
        ),
    ];
    let mut echecs = Vec::new();
    for (nom, demande, attendu, prefixe) in table {
        let r = ecritures::lister(&acces, &lecture, demande).await;
        match message_de_validation(&r) {
            Ok(m) if (prefixe && m.starts_with(attendu)) || (!prefixe && m == attendu) => {}
            Ok(m) => echecs.push(format!(
                "AC10 (a) {nom} : attendu « {attendu} », rendu « {m} »"
            )),
            Err(autre) => echecs.push(format!(
                "AC10 (a) {nom} : attendu Validation, rendu {autre}"
            )),
        }
    }
    let sans_filtre = ecritures::lister(&acces, &lecture, d())
        .await
        .unwrap()
        .total;
    let blanche = ecritures::lister(
        &acces,
        &lecture,
        DemandeListeEcritures {
            description: s("   "),
            ..d()
        },
    )
    .await
    .unwrap()
    .total;
    if sans_filtre < 1 || blanche != sans_filtre {
        echecs.push(format!(
            "AC10 (a) description blanche : {blanche} écritures pour {sans_filtre} sans filtre (fixture : au moins une)"
        ));
    }
    assert!(echecs.is_empty(), "{}", echecs.join("\n"));
}

/// Une ligne de la table des bornes : (limit, offset) demandés, (limit, offset)
/// rendus.
type LigneDeBornes = ((Option<i64>, Option<i64>), (i64, i64));

/// Les trois couples (limit, offset) demandés de l'AC10 (b), et leur rendu
/// attendu en fonction du défaut et du plafond.
fn table_des_bornes(defaut: i64, plafond: i64) -> [LigneDeBornes; 3] {
    [
        ((None, None), (defaut, 0)),
        ((Some(0), Some(-3)), (1, 0)),
        ((Some(10_000), Some(0)), (plafond, 0)),
    ]
}

/// AC10 (b) — les bornes des trois services paginés, au service : défauts et
/// plafonds — écritures 50 / 500, postes ouverts 50 / 500, contacts 20 / 100.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn ac10_b_les_bornes_des_trois_listes(pool: MySqlPool) {
    let livres = poser_livres(&pool).await;
    let lecture = Lecture::new(pool.clone());
    let acces = api_societe(
        livres.s.company_id,
        livres.s.admin_user_id,
        Role::Consultation,
    );
    let mut echecs = Vec::new();
    for ((limit, offset), attendu) in table_des_bornes(50, 500) {
        let r = ecritures::lister(
            &acces,
            &lecture,
            DemandeListeEcritures {
                limit,
                offset,
                ..Default::default()
            },
        )
        .await
        .map(|p| (p.limit, p.offset));
        if r.as_ref().ok() != Some(&attendu) {
            echecs.push(format!(
                "AC10 (b) écritures ({limit:?}, {offset:?}) : attendu {attendu:?}, rendu {r:?}"
            ));
        }
    }
    for ((limit, offset), attendu) in table_des_bornes(50, 500) {
        let r = postes_ouverts::lister(
            &acces,
            &lecture,
            DemandePostesOuverts {
                account_id: livres.compte("1000"),
                as_of: None,
                limit,
                offset,
            },
        )
        .await
        .map(|p| (p.limit, p.offset));
        if r.as_ref().ok() != Some(&attendu) {
            echecs.push(format!(
                "AC10 (b) postes ouverts ({limit:?}, {offset:?}) : attendu {attendu:?}, rendu {r:?}"
            ));
        }
    }
    for ((limit, offset), attendu) in table_des_bornes(20, 100) {
        let r = carnet::lister(
            &acces,
            &lecture,
            DemandeListeContacts {
                limit,
                offset,
                ..Default::default()
            },
        )
        .await
        .map(|p| (p.page.limit, p.page.offset));
        if r.as_ref().ok() != Some(&attendu) {
            echecs.push(format!(
                "AC10 (b) contacts ({limit:?}, {offset:?}) : attendu {attendu:?}, rendu {r:?}"
            ));
        }
    }
    assert!(echecs.is_empty(), "{}", echecs.join("\n"));
}

// ============================================================
// AC11 — lecture seule, et filtre de société
// ============================================================

/// AC11 (a) — les cinq services réussissent sur une `Lecture` en lecture seule
/// (le détail d'écriture : `BEGIN`, quatre lectures, `ROLLBACK` ; les postes
/// ouverts : leur transaction propre), et les fixtures atteignent bien les
/// branches visées (une écriture, un poste ouvert, un contact). Avec
/// l'AC11 (c), confirme l'absence d'écriture **et** de verrou sur les branches
/// exercées.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn ac11_a_les_cinq_services_lisent_en_lecture_seule(pool: MySqlPool) {
    let livres = poser_livres(&pool).await;
    let lecture = lecture_seule(&pool).await;
    let acces = api_societe(
        livres.s.company_id,
        livres.s.admin_user_id,
        Role::Consultation,
    );
    let mut echecs = Vec::new();
    match ecritures::lister(&acces, &lecture, DemandeListeEcritures::default()).await {
        Ok(p) if p.total >= 1 => {}
        autre => echecs.push(format!(
            "AC11 (a) ecritures::lister : {:?}",
            autre.map(|p| p.total)
        )),
    }
    match ecritures::detail(&acces, &lecture, livres.ecriture_id).await {
        Ok(d) if d.ecriture.entry.id == livres.ecriture_id => {}
        autre => echecs.push(format!(
            "AC11 (a) ecritures::detail : {:?}",
            autre.map(|d| d.ecriture.entry.id)
        )),
    }
    let demande = DemandePostesOuverts {
        account_id: livres.compte("1000"),
        ..Default::default()
    };
    match postes_ouverts::lister(&acces, &lecture, demande).await {
        Ok(p) if p.total >= 1 => {}
        autre => echecs.push(format!(
            "AC11 (a) postes_ouverts::lister : {:?}",
            autre.map(|p| p.total)
        )),
    }
    match carnet::lister(&acces, &lecture, DemandeListeContacts::default()).await {
        Ok(p) if p.page.total >= 1 => {}
        autre => echecs.push(format!(
            "AC11 (a) carnet::lister : {:?}",
            autre.map(|p| p.page.total)
        )),
    }
    match carnet::detail(&acces, &lecture, livres.contact_id).await {
        Ok(f) if f.contact.id == livres.contact_id => {}
        autre => echecs.push(format!(
            "AC11 (a) carnet::detail : {:?}",
            autre.map(|f| f.contact.id)
        )),
    }
    assert!(echecs.is_empty(), "{}", echecs.join("\n"));
}

/// AC11 (b) — filtre de société : un contexte MCP d'une seconde société,
/// toutes portées, ne lit rien de la première.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn ac11_b_aucune_lecture_d_une_autre_societe(pool: MySqlPool) {
    let livres = poser_livres(&pool).await;
    let seconde = seconde_societe(&pool).await;
    let lecture = Lecture::new(pool.clone());
    let acces = mcp_societe(seconde, livres.s.admin_user_id, &TOUTES);
    let mut echecs = Vec::new();
    let r = ecritures::detail(&acces, &lecture, livres.ecriture_id)
        .await
        .map(drop);
    if !est_introuvable(&r) {
        echecs.push(format!("AC11 (b) ecritures::detail : {}", resume(&r)));
    }
    let r = carnet::detail(&acces, &lecture, livres.contact_id)
        .await
        .map(drop);
    if !est_introuvable(&r) {
        echecs.push(format!("AC11 (b) carnet::detail : {}", resume(&r)));
    }
    let demande = DemandePostesOuverts {
        account_id: livres.compte("1000"),
        ..Default::default()
    };
    let r = postes_ouverts::lister(&acces, &lecture, demande)
        .await
        .map(drop);
    if !est_introuvable(&r) {
        echecs.push(format!("AC11 (b) postes_ouverts::lister : {}", resume(&r)));
    }
    match ecritures::lister(&acces, &lecture, DemandeListeEcritures::default()).await {
        Ok(p) if p.total == 0 && p.items.is_empty() => {}
        autre => echecs.push(format!(
            "AC11 (b) ecritures::lister : {:?}",
            autre.map(|p| p.total)
        )),
    }
    match carnet::lister(&acces, &lecture, DemandeListeContacts::default()).await {
        Ok(p) if p.page.total == 0 && p.page.items.is_empty() => {}
        autre => echecs.push(format!(
            "AC11 (b) carnet::lister : {:?}",
            autre.map(|p| p.page.total)
        )),
    }
    assert!(echecs.is_empty(), "{}", echecs.join("\n"));
}

/// C-32-4a-6 — `ecritures::lister` ne lit **pas** la société : un contexte
/// d'une société qui n'existe pas rend une page vide, non une erreur
/// (`ecritures::detail`, `postes_ouverts::lister` et `carnet::lister`, qui la
/// lisent d'abord ou seule, rendent `Internal` — `ac9_c` ; `carnet::detail` la
/// lit après le contact et rend `NotFound` —
/// `ac9_carnet_detail_lit_le_contact_avant_la_societe`). Revue de code P1, A3 = E3 : sans ce
/// test, un `company_for_ids` ajouté en tête de `lister` laissait tout vert.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn ac11_b_la_liste_des_ecritures_ne_lit_pas_la_societe(pool: MySqlPool) {
    let livres = poser_livres(&pool).await;
    let lecture = Lecture::new(pool.clone());
    let acces = mcp_societe(INEXISTANT, livres.s.admin_user_id, &TOUTES);
    match ecritures::lister(&acces, &lecture, DemandeListeEcritures::default()).await {
        Ok(p) if p.total == 0 && p.items.is_empty() => {}
        autre => panic!(
            "ecritures::lister sur une société inexistante : attendu une page vide, rendu {:?}",
            autre.map(|p| p.total)
        ),
    }
}

// ============================================================
// Revue de code P1 (B1 = E1 = E2, A4) — le chemin de succès des filtres
// et le tri par défaut, par la route
// ============================================================

/// Une écriture synthétique à deux lignes : `montant` au débit de `debit`, au
/// crédit de `credit`.
#[allow(clippy::too_many_arguments)]
async fn poser_ecriture(
    pool: &MySqlPool,
    s: &SeededCompany,
    date: (i32, u32, u32),
    journal: Journal,
    description: &str,
    debit: &str,
    credit: &str,
    montant: Decimal,
) {
    journal_entries::create(
        pool,
        s.fiscal_year_id,
        s.admin_user_id,
        NewJournalEntry {
            company_id: s.company_id,
            entry_date: NaiveDate::from_ymd_opt(date.0, date.1, date.2).unwrap(),
            journal,
            description: description.into(),
            project_id: None,
            lines: vec![
                NewJournalEntryLine {
                    account_id: s.accounts[debit],
                    debit: montant,
                    credit: Decimal::ZERO,
                    project_id: None,
                },
                NewJournalEntryLine {
                    account_id: s.accounts[credit],
                    debit: Decimal::ZERO,
                    credit: montant,
                    project_id: None,
                },
            ],
        },
    )
    .await
    .expect("écriture synthétique");
}

/// Les libellés d'une page de `GET /api/v1/journal-entries`, en ensemble.
fn libelles(corps: &Value) -> BTreeSet<String> {
    corps["items"]
        .as_array()
        .map(|v| {
            v.iter()
                .filter_map(|e| e["description"].as_str().map(str::to_string))
                .collect()
        })
        .unwrap_or_default()
}

/// Chacun des filtres que `ecritures::lister` recopie vers
/// `JournalEntryListQuery` — `description`, `accountId`, `amountMin`,
/// `amountMax`, `dateFrom`, `dateTo`, `journal` —, seul, puis les bornes
/// d'égalité (`dateFrom == dateTo`, `amountMin == amountMax`, acceptées) et
/// l'inclusion des deux bornes, par la route : le handler, le service et le
/// dépôt à la fois. Puis, en ordre rendu (revue de code P2, P2-1), `sortBy`,
/// `sortDir` et un `offset` positif par la route, et le tri par défaut au
/// service, demande vide (`EntryDate`, `Desc` : ce que reçoit le MCP).
///
/// Fixture synthétique — trois écritures que chaque filtre sépare :
/// « Loyer mars » (10.03.2026, Achats, 100, 4000 / 1000), « Vente comptoir »
/// (20.03.2026, Ventes, 250, 1100 / 3000), « Frais bancaires » (05.04.2026,
/// Banque, 40, 4000 / 1100). Chaque ligne de la table est choisie pour qu'une
/// recopie perdue (`None`) ou deux bornes permutées rendent un autre ensemble,
/// et qu'un contrôle croisé devenu strict (`>=`) rende un 400.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn filtres_des_ecritures_par_la_route(pool: MySqlPool) {
    let s = societe(&pool).await;
    poser_ecriture(
        &pool,
        &s,
        (2026, 3, 10),
        Journal::Achats,
        "Loyer mars",
        "4000",
        "1000",
        dec!(100),
    )
    .await;
    poser_ecriture(
        &pool,
        &s,
        (2026, 3, 20),
        Journal::Ventes,
        "Vente comptoir",
        "1100",
        "3000",
        dec!(250),
    )
    .await;
    poser_ecriture(
        &pool,
        &s,
        (2026, 4, 5),
        Journal::Banque,
        "Frais bancaires",
        "4000",
        "1100",
        dec!(40),
    )
    .await;
    let app = spawn_app(pool.clone()).await;
    let jwt = forge_jwt(s.admin_user_id, "Consultation", s.company_id);

    const L: &str = "Loyer mars";
    const V: &str = "Vente comptoir";
    const F: &str = "Frais bancaires";
    let compte_ventes = s.accounts["3000"];
    let table: Vec<(String, Vec<&str>)> = vec![
        (String::new(), vec![L, V, F]),
        ("description=Loyer".into(), vec![L]),
        (format!("accountId={compte_ventes}"), vec![V]),
        ("amountMin=200".into(), vec![V]),
        ("amountMax=50".into(), vec![F]),
        ("amountMin=250&amountMax=250".into(), vec![V]),
        ("amountMin=40&amountMax=100".into(), vec![L, F]),
        ("dateFrom=2026-03-15".into(), vec![V, F]),
        ("dateTo=2026-03-15".into(), vec![L]),
        ("dateFrom=2026-03-20&dateTo=2026-03-20".into(), vec![V]),
        ("dateFrom=2026-03-10&dateTo=2026-03-20".into(), vec![L, V]),
        ("journal=Ventes".into(), vec![V]),
    ];
    let mut echecs = Vec::new();
    for (filtre, attendu) in table {
        let chemin = format!("/api/v1/journal-entries?{filtre}");
        let (statut, corps) = get_json(&app, &jwt, &chemin).await;
        let attendu: BTreeSet<String> = attendu.iter().map(|x| x.to_string()).collect();
        let rendu = libelles(&corps);
        if statut != 200 || rendu != attendu || corps["total"] != attendu.len() {
            echecs.push(format!(
                "{chemin} : attendu 200 {attendu:?} ; rendu {statut} {rendu:?}, total {} — {}",
                corps["total"], corps["error"]
            ));
        }
    }

    // Revue de code P2, P2-1 — le tri et le décalage, en ordre rendu : défaut
    // (`EntryDate`, `Desc`), `sortDir`, `sortBy` (un ordre distinct des deux
    // autres), et un `offset` positif. Le total reste celui sans pagination.
    let ordonnee: Vec<(&str, Vec<&str>)> = vec![
        ("", vec![F, V, L]),
        ("sortDir=Asc", vec![L, V, F]),
        ("sortBy=Description&sortDir=Asc", vec![F, L, V]),
        ("limit=1&offset=1", vec![V]),
    ];
    for (filtre, attendu) in ordonnee {
        let chemin = format!("/api/v1/journal-entries?{filtre}");
        let (statut, corps) = get_json(&app, &jwt, &chemin).await;
        let rendu: Vec<String> = corps["items"]
            .as_array()
            .map(|v| {
                v.iter()
                    .filter_map(|e| e["description"].as_str().map(str::to_string))
                    .collect()
            })
            .unwrap_or_default();
        if statut != 200 || rendu != attendu || corps["total"] != 3 {
            echecs.push(format!(
                "{chemin} (ordonné) : attendu 200 {attendu:?}, total 3 ; rendu {statut} {rendu:?}, total {} — {}",
                corps["total"], corps["error"]
            ));
        }
    }

    // Au service, demande vide : le tri par défaut que reçoit l'outil MCP de
    // la 32-9a.
    let lecture = Lecture::new(pool.clone());
    let acces = mcp_societe(s.company_id, s.admin_user_id, &[Portee::BooksRead]);
    match ecritures::lister(&acces, &lecture, DemandeListeEcritures::default()).await {
        Ok(p) => {
            let rendu: Vec<&str> = p
                .items
                .iter()
                .map(|e| e.entry.description.as_str())
                .collect();
            if rendu != [F, V, L] {
                echecs.push(format!(
                    "service, demande vide : attendu {:?}, rendu {rendu:?}",
                    [F, V, L]
                ));
            }
        }
        Err(e) => echecs.push(format!("service, demande vide : {e:?}")),
    }
    assert!(echecs.is_empty(), "{}", echecs.join("\n"));
}

/// Un contact synthétique ; dates de création et de modification posées, pour
/// que les trois colonnes de tri rendent trois ordres distincts.
#[allow(clippy::too_many_arguments)]
async fn poser_contact(
    pool: &MySqlPool,
    company_id: i64,
    nom: &str,
    type_: &str,
    client: bool,
    fournisseur: bool,
    actif: bool,
    cree: &str,
    modifie: &str,
) {
    sqlx::query(
        "INSERT INTO contacts (company_id, contact_type, name, is_client, is_supplier, active, \
         created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(company_id)
    .bind(type_)
    .bind(nom)
    .bind(client)
    .bind(fournisseur)
    .bind(actif)
    .bind(cree)
    .bind(modifie)
    .execute(pool)
    .await
    .expect("contact synthétique");
}

/// Les noms d'une page de `GET /api/v1/contacts`, dans l'ordre rendu.
fn noms(corps: &Value) -> Vec<String> {
    corps["items"]
        .as_array()
        .map(|v| {
            v.iter()
                .filter_map(|c| c["name"].as_str().map(str::to_string))
                .collect()
        })
        .unwrap_or_default()
}

/// Le tri par défaut du carnet (AC6 : `sort_by` absent → `Name`, `sort_direction`
/// absent → `Asc`, ce que reçoit le MCP), au service **et** par la route, puis
/// la recopie, par le handler, de chacun des paramètres de `GET /api/v1/contacts`
/// — `search`, `contactType`, `isClient`, `isSupplier`, `includeArchived`,
/// `sortBy`, `sortDirection` — et d'un `offset` positif (revue de code P2).
///
/// Fixture synthétique — quatre contacts, dont l'ordre se distingue selon la
/// colonne : nom (Alpha, Archivé, Mu, Zeta), création (Zeta, Mu, Alpha,
/// Archivé), modification (Mu, Alpha, Zeta, Archivé) ; « Archivé SA » est
/// inactif, client et fournisseur.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn tri_par_defaut_et_filtres_du_carnet(pool: MySqlPool) {
    let s = societe(&pool).await;
    let c = s.company_id;
    poser_contact(
        &pool,
        c,
        "Alpha SA",
        "Entreprise",
        true,
        false,
        true,
        "2026-03-01 00:00:00",
        "2026-01-20 00:00:00",
    )
    .await;
    poser_contact(
        &pool,
        c,
        "Mu Personne",
        "Personne",
        false,
        false,
        true,
        "2026-02-01 00:00:00",
        "2026-01-10 00:00:00",
    )
    .await;
    poser_contact(
        &pool,
        c,
        "Zeta SA",
        "Entreprise",
        false,
        true,
        true,
        "2026-01-01 00:00:00",
        "2026-01-30 00:00:00",
    )
    .await;
    poser_contact(
        &pool,
        c,
        "Archivé SA",
        "Entreprise",
        true,
        true,
        false,
        "2026-04-01 00:00:00",
        "2026-04-01 00:00:00",
    )
    .await;

    const A: &str = "Alpha SA";
    const M: &str = "Mu Personne";
    const Z: &str = "Zeta SA";
    const X: &str = "Archivé SA";
    let mut echecs = Vec::new();

    // Au service, demande vide : ce que reçoit l'outil MCP de la 32-9b.
    let lecture = Lecture::new(pool.clone());
    let acces = mcp_societe(c, s.admin_user_id, &[Portee::ContactsRead]);
    match carnet::lister(&acces, &lecture, DemandeListeContacts::default()).await {
        Ok(p) => {
            let rendu: Vec<&str> = p.page.items.iter().map(|x| x.name.as_str()).collect();
            if rendu != [A, M, Z] {
                echecs.push(format!(
                    "service, demande vide : attendu {:?}, rendu {rendu:?}",
                    [A, M, Z]
                ));
            }
        }
        Err(e) => echecs.push(format!("service, demande vide : {e:?}")),
    }

    let app = spawn_app(pool.clone()).await;
    let jwt = forge_jwt(s.admin_user_id, "Consultation", c);
    let table: Vec<(&str, Vec<&str>)> = vec![
        ("", vec![A, M, Z]),
        ("sortDirection=Desc", vec![Z, M, A]),
        ("sortBy=CreatedAt", vec![Z, M, A]),
        ("sortBy=CreatedAt&sortDirection=Desc", vec![A, M, Z]),
        ("sortBy=UpdatedAt", vec![M, A, Z]),
        ("isClient=true", vec![A]),
        ("isClient=false", vec![M, Z]),
        ("isSupplier=true", vec![Z]),
        ("contactType=Personne", vec![M]),
        ("search=Zeta", vec![Z]),
        ("includeArchived=true", vec![A, X, M, Z]),
    ];
    for (filtre, attendu) in table {
        let chemin = format!("/api/v1/contacts?{filtre}");
        let (statut, corps) = get_json(&app, &jwt, &chemin).await;
        let rendu = noms(&corps);
        if statut != 200 || rendu != attendu || corps["total"] != attendu.len() {
            echecs.push(format!(
                "{chemin} : attendu 200 {attendu:?} ; rendu {statut} {rendu:?}, total {}",
                corps["total"]
            ));
        }
    }

    // Revue de code P2, P2-1 — un `offset` positif, recopié par le handler :
    // la deuxième ligne de l'ordre par défaut, total inchangé.
    let chemin = "/api/v1/contacts?limit=1&offset=1";
    let (statut, corps) = get_json(&app, &jwt, chemin).await;
    let rendu = noms(&corps);
    if statut != 200 || rendu != [M] || corps["total"] != 3 || corps["offset"] != 1 {
        echecs.push(format!(
            "{chemin} : attendu 200 [{M:?}], total 3, offset 1 ; rendu {statut} {rendu:?}, total {}, offset {}",
            corps["total"], corps["offset"]
        ));
    }
    assert!(echecs.is_empty(), "{}", echecs.join("\n"));
}

// ============================================================
// Story 32-4b — les factures, par la route (AC11 (c) à (g), T1 (i) : jouées
// sur le code d'avant le déplacement, elles décrivent le comportement
// d'aujourd'hui, puis tiennent le code déplacé)
// ============================================================

/// Les factures de l'AC11 (d) à (g) — fixture synthétique, au patron des trois
/// écritures de `filtres_des_ecritures_par_la_route` : quatre factures validées
/// (F1 à F4) et un brouillon (F5), chacune d'un client distinct, dont les cinq
/// colonnes de tri rendent cinq ordres distincts deux à deux, dans les deux sens
/// (dix ordres en tout, chacun différent des neuf autres) :
///
/// | | date | échéance | HT | client | créée |
/// |---|---|---|---|---|---|
/// | F1 | 10.01.2021 | 31.05.2021 (échue) | 200 | Bravo SA | 03.01.2026 |
/// | F2 | 10.02.2021 | 30.09.2030 | 400 | Alpha SA | 02.01.2026 |
/// | F3 | 10.03.2021 | 20.03.2021 (échue) | 300 | Delta SA | 01.01.2026 |
/// | F4 | 10.04.2021 | 30.06.2030 | 100 | Charlie SA | 04.01.2026 |
/// | F5 (brouillon) | 10.06.2021 | 30.06.2021 | 50 | Echo SA | 05.01.2026 |
///
/// F1 porte un règlement partiel (50.00 sur 216.20), F4 est **payée** (108.10),
/// F2 a ses rappels **suspendus**. `created_at` est posé par un `UPDATE`
/// explicite, distinct pour chacune (validation P1, F5). Dates indépendantes de
/// l'horloge : 2021 est échu, 2030 à échoir.
struct Factures {
    s: SeededCompany,
    /// Les ids de F1 à F5.
    ids: [i64; 5],
    /// Les clients de F1 à F5.
    clients: [i64; 5],
    /// Le numéro de F1 (pour la recherche par numéro).
    numero_f1: String,
}

impl Factures {
    /// Le libellé (`"F1"` … `"F5"`) d'un id rendu, `"?"` s'il est étranger.
    fn libelle(&self, id: i64) -> String {
        self.ids
            .iter()
            .position(|x| *x == id)
            .map_or("?".into(), |i| format!("F{}", i + 1))
    }

    /// Les libellés d'une page de `GET /api/v1/invoices` ou de l'échéancier,
    /// dans l'ordre rendu.
    fn page(&self, corps: &Value) -> Vec<String> {
        corps["items"]
            .as_array()
            .map(|v| {
                v.iter()
                    .filter_map(|e| e["id"].as_i64())
                    .map(|id| self.libelle(id))
                    .collect()
            })
            .unwrap_or_default()
    }
}

async fn poser_factures(pool: &MySqlPool) -> Factures {
    let s = societe(pool).await;
    let jour = |a, m, j| NaiveDate::from_ymd_opt(a, m, j).unwrap();
    let mut clients = [0; 5];
    for (i, nom) in ["Bravo SA", "Alpha SA", "Delta SA", "Charlie SA", "Echo SA"]
        .iter()
        .enumerate()
    {
        clients[i] = poser_contact_client(pool, s.company_id, nom).await;
    }
    // (date, échéance, HT, validée, créée)
    let fiches = [
        (
            jour(2021, 1, 10),
            jour(2021, 5, 31),
            dec!(200),
            true,
            "2026-01-03 00:00:00",
        ),
        (
            jour(2021, 2, 10),
            jour(2030, 9, 30),
            dec!(400),
            true,
            "2026-01-02 00:00:00",
        ),
        (
            jour(2021, 3, 10),
            jour(2021, 3, 20),
            dec!(300),
            true,
            "2026-01-01 00:00:00",
        ),
        (
            jour(2021, 4, 10),
            jour(2030, 6, 30),
            dec!(100),
            true,
            "2026-01-04 00:00:00",
        ),
        (
            jour(2021, 6, 10),
            jour(2021, 6, 30),
            dec!(50),
            false,
            "2026-01-05 00:00:00",
        ),
    ];
    let mut ids = [0; 5];
    for (i, (date, echeance, ht, valider, cree)) in fiches.into_iter().enumerate() {
        ids[i] = poser_facture(pool, &s, clients[i], date, echeance, ht, valider).await;
        sqlx::query("UPDATE invoices SET created_at = ? WHERE id = ?")
            .bind(cree)
            .bind(ids[i])
            .execute(pool)
            .await
            .expect("created_at synthétique");
    }
    regler(pool, &s, ids[0], dec!(50.00), jour(2021, 6, 15)).await;
    regler(pool, &s, ids[3], dec!(108.10), jour(2021, 6, 15)).await;
    sqlx::query("UPDATE invoices SET dunning_paused_at = UTC_TIMESTAMP(6) WHERE id = ?")
        .bind(ids[1])
        .execute(pool)
        .await
        .expect("suspension synthétique");
    let numero_f1: String = sqlx::query_scalar("SELECT invoice_number FROM invoices WHERE id = ?")
        .bind(ids[0])
        .fetch_one(pool)
        .await
        .expect("numéro de F1");
    Factures {
        s,
        ids,
        clients,
        numero_f1,
    }
}

/// Les dix ordres de tri (colonne, sens) de l'AC11 (e), attendus sur F1 à F4.
const ORDRES_DES_FACTURES: [(&str, &str, [&str; 4]); 10] = [
    ("Date", "Asc", ["F1", "F2", "F3", "F4"]),
    ("Date", "Desc", ["F4", "F3", "F2", "F1"]),
    ("DueDate", "Asc", ["F3", "F1", "F4", "F2"]),
    ("DueDate", "Desc", ["F2", "F4", "F1", "F3"]),
    ("TotalAmount", "Asc", ["F4", "F1", "F3", "F2"]),
    ("TotalAmount", "Desc", ["F2", "F3", "F1", "F4"]),
    ("ContactName", "Asc", ["F2", "F1", "F4", "F3"]),
    ("ContactName", "Desc", ["F3", "F4", "F1", "F2"]),
    ("CreatedAt", "Asc", ["F3", "F2", "F1", "F4"]),
    ("CreatedAt", "Desc", ["F4", "F1", "F2", "F3"]),
];

/// AC11 (c) — les bornes et les refus des trois listes, par la route : les
/// listes clientes **refusent** hors bornes (400, `VALIDATION_ERROR`, message
/// du code), la liste fournisseur **écrête**.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn ac324b_c_bornes_et_refus_des_factures_par_la_route(pool: MySqlPool) {
    let s = societe(&pool).await;
    let app = spawn_app(pool.clone()).await;
    let jwt = forge_jwt(s.admin_user_id, "Consultation", s.company_id);
    let mut echecs = Vec::new();
    for (chemin, message) in [
        (
            "/api/v1/invoices?limit=101",
            "limit doit être compris entre 1 et 100",
        ),
        (
            "/api/v1/invoices?limit=0",
            "limit doit être compris entre 1 et 100",
        ),
        (
            "/api/v1/invoices?offset=-1",
            "offset doit être positif ou nul",
        ),
        (
            "/api/v1/invoices/due-dates?offset=1000001",
            "offset doit être ≤ 1000000",
        ),
    ] {
        let (statut, corps) = get_json(&app, &jwt, chemin).await;
        if statut != 400
            || corps["error"]["code"] != "VALIDATION_ERROR"
            || corps["error"]["message"] != message
        {
            echecs.push(format!(
                "{chemin} : attendu 400 VALIDATION_ERROR « {message} » ; rendu {statut} {corps}"
            ));
        }
    }
    for (chemin, limit, offset) in [
        ("/api/v1/invoices", 20, 0),
        ("/api/v1/supplier-invoices?limit=1000&offset=-3", 200, 0),
        ("/api/v1/supplier-invoices", 50, 0),
    ] {
        let (statut, corps) = get_json(&app, &jwt, chemin).await;
        if statut != 200 || corps["limit"] != limit || corps["offset"] != offset {
            echecs.push(format!(
                "{chemin} : attendu 200, limit {limit}, offset {offset} ; rendu {statut} {corps}"
            ));
        }
    }
    let (statut, corps) = get_json(&app, &jwt, "/api/v1/invoices?status=cancelled").await;
    if statut != 200 {
        echecs.push(format!(
            "?status=cancelled : attendu 200 ; rendu {statut} {corps}"
        ));
    }
    assert!(echecs.is_empty(), "AC11 (c) :\n{}", echecs.join("\n"));
}

/// AC11 (d), (e), (f) — la liste des factures, par la route : chaque filtre
/// seul (`search` par client et par numéro, `status`, `contactId`, `dateFrom`,
/// `dateTo` bornes incluses, `paused` sous ses trois valeurs), un `offset`
/// positif, chaque colonne et chaque sens du tri en ordre rendu (sur F1 à F4,
/// `status=validated`), et le tri par défaut (`Date`, `Desc`).
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn ac324b_d_e_f_la_liste_des_factures_par_la_route(pool: MySqlPool) {
    let f = poser_factures(&pool).await;
    let app = spawn_app(pool.clone()).await;
    let jwt = forge_jwt(f.s.admin_user_id, "Consultation", f.s.company_id);
    let mut echecs = Vec::new();
    let table: Vec<(String, Vec<&str>)> = vec![
        (String::new(), vec!["F1", "F2", "F3", "F4", "F5"]),
        ("search=Charlie".into(), vec!["F4"]),
        (format!("search={}", f.numero_f1), vec!["F1"]),
        ("status=draft".into(), vec!["F5"]),
        ("status=validated".into(), vec!["F1", "F2", "F3", "F4"]),
        (format!("contactId={}", f.clients[0]), vec!["F1"]),
        ("dateFrom=2021-02-10".into(), vec!["F2", "F3", "F4", "F5"]),
        ("dateTo=2021-02-10".into(), vec!["F1", "F2"]),
        ("paused=paused".into(), vec!["F2"]),
        ("paused=not-paused".into(), vec!["F1", "F3", "F4", "F5"]),
        ("paused=all".into(), vec!["F1", "F2", "F3", "F4", "F5"]),
    ];
    for (filtre, attendu) in table {
        let chemin = format!("/api/v1/invoices?{filtre}");
        let (statut, corps) = get_json(&app, &jwt, &chemin).await;
        let rendu: BTreeSet<String> = f.page(&corps).into_iter().collect();
        let attendu: BTreeSet<String> = attendu.iter().map(|x| x.to_string()).collect();
        if statut != 200 || rendu != attendu || corps["total"] != attendu.len() {
            echecs.push(format!(
                "{chemin} : attendu 200 {attendu:?} ; rendu {statut} {rendu:?}, total {} — {}",
                corps["total"], corps["error"]
            ));
        }
    }
    let mut ordonnee: Vec<(String, Vec<&str>, usize)> = vec![
        (String::new(), vec!["F5", "F4", "F3", "F2", "F1"], 5),
        ("limit=1&offset=1".into(), vec!["F4"], 5),
    ];
    for (colonne, sens, attendu) in ORDRES_DES_FACTURES {
        ordonnee.push((
            format!("status=validated&sortBy={colonne}&sortDirection={sens}"),
            attendu.to_vec(),
            4,
        ));
    }
    for (filtre, attendu, total) in ordonnee {
        let chemin = format!("/api/v1/invoices?{filtre}");
        let (statut, corps) = get_json(&app, &jwt, &chemin).await;
        let rendu = f.page(&corps);
        if statut != 200 || rendu != attendu || corps["total"] != total {
            echecs.push(format!(
                "{chemin} (ordonné) : attendu 200 {attendu:?}, total {total} ; rendu {statut} {rendu:?}, total {} — {}",
                corps["total"], corps["error"]
            ));
        }
    }
    assert!(echecs.is_empty(), "{}", echecs.join("\n"));
}

/// AC11 (d), (e), (f) — l'échéancier, par la route : chaque filtre seul
/// (`search`, `contactId`, `dateFrom`, `dateTo`, `dueBefore`, `paymentStatus`
/// sous ses quatre valeurs), un `offset` positif ; les constantes — un
/// brouillon n'y paraît jamais, même avec `paymentStatus=all` (M21a) ; F2
/// suspendue **et** F1, F3 non suspendues y paraissent (M21b) ; F4 payée n'y
/// paraît pas sans `paymentStatus` (M21c) — ; le résumé, le même pour
/// `paymentStatus=paid` et sans paramètre (contrat de la route ; le dépôt ignore
/// lui-même `payment_status`, M21d équivalente) ; chaque colonne et chaque sens
/// du tri sur F1 à F4 (`paymentStatus=all`) ; le tri par défaut (`DueDate`,
/// `Asc`).
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn ac324b_d_e_f_l_echeancier_par_la_route(pool: MySqlPool) {
    let f = poser_factures(&pool).await;
    let app = spawn_app(pool.clone()).await;
    let jwt = forge_jwt(f.s.admin_user_id, "Consultation", f.s.company_id);
    let mut echecs = Vec::new();
    let table: Vec<(String, Vec<&str>)> = vec![
        (String::new(), vec!["F1", "F2", "F3"]),
        ("paymentStatus=all".into(), vec!["F1", "F2", "F3", "F4"]),
        ("paymentStatus=paid".into(), vec!["F4"]),
        ("paymentStatus=unpaid".into(), vec!["F1", "F2", "F3"]),
        ("paymentStatus=overdue".into(), vec!["F1", "F3"]),
        ("search=Delta".into(), vec!["F3"]),
        (format!("contactId={}", f.clients[0]), vec!["F1"]),
        ("dateFrom=2021-02-10".into(), vec!["F2", "F3"]),
        ("dateTo=2021-02-10".into(), vec!["F1", "F2"]),
        ("dueBefore=2021-05-31".into(), vec!["F1", "F3"]),
    ];
    for (filtre, attendu) in table {
        let chemin = format!("/api/v1/invoices/due-dates?{filtre}");
        let (statut, corps) = get_json(&app, &jwt, &chemin).await;
        let rendu: BTreeSet<String> = f.page(&corps).into_iter().collect();
        let attendu: BTreeSet<String> = attendu.iter().map(|x| x.to_string()).collect();
        if statut != 200 || rendu != attendu || corps["total"] != attendu.len() {
            echecs.push(format!(
                "{chemin} : attendu 200 {attendu:?} ; rendu {statut} {rendu:?}, total {} — {}",
                corps["total"], corps["error"]
            ));
        }
    }
    let mut ordonnee: Vec<(String, Vec<&str>, usize)> = vec![
        (String::new(), vec!["F3", "F1", "F2"], 3),
        ("limit=1&offset=1".into(), vec!["F1"], 3),
    ];
    for (colonne, sens, attendu) in ORDRES_DES_FACTURES {
        ordonnee.push((
            format!("paymentStatus=all&sortBy={colonne}&sortDirection={sens}"),
            attendu.to_vec(),
            4,
        ));
    }
    for (filtre, attendu, total) in ordonnee {
        let chemin = format!("/api/v1/invoices/due-dates?{filtre}");
        let (statut, corps) = get_json(&app, &jwt, &chemin).await;
        let rendu = f.page(&corps);
        if statut != 200 || rendu != attendu || corps["total"] != total {
            echecs.push(format!(
                "{chemin} (ordonné) : attendu 200 {attendu:?}, total {total} ; rendu {statut} {rendu:?}, total {} — {}",
                corps["total"], corps["error"]
            ));
        }
    }
    let (_, sans) = get_json(&app, &jwt, "/api/v1/invoices/due-dates").await;
    let (_, payees) = get_json(&app, &jwt, "/api/v1/invoices/due-dates?paymentStatus=paid").await;
    if sans["summary"].is_null() || sans["summary"] != payees["summary"] {
        echecs.push(format!(
            "résumé : sans paramètre {} ; paymentStatus=paid {}",
            sans["summary"], payees["summary"]
        ));
    }
    assert!(echecs.is_empty(), "{}", echecs.join("\n"));
}

/// Les numéros d'une page de `GET /api/v1/supplier-invoices`, dans l'ordre
/// rendu.
fn numeros_fournisseur(corps: &Value) -> Vec<String> {
    corps["items"]
        .as_array()
        .map(|v| {
            v.iter()
                .filter_map(|e| e["supplierInvoiceNumber"].as_str().map(str::to_string))
                .collect()
        })
        .unwrap_or_default()
}

/// Trois factures fournisseur posées dans l'ordre FA (01.03.2021), FB
/// (01.01.2021), FC (01.02.2021) : l'ordre du dépôt (`invoice_date DESC,
/// id DESC`) rend FA, FC, FB — distinct de l'ordre des ids, dans un sens comme
/// dans l'autre.
async fn poser_factures_fournisseur(pool: &MySqlPool) -> SeededCompany {
    let s = societe(pool).await;
    let fournisseur = poser_contact_fournisseur(pool, s.company_id, "Fournisseur SA").await;
    for (numero, mois) in [("FA", 3), ("FB", 1), ("FC", 2)] {
        poser_facture_fournisseur(
            pool,
            &s,
            fournisseur,
            NaiveDate::from_ymd_opt(2021, mois, 1).unwrap(),
            numero,
        )
        .await;
    }
    s
}

/// AC11 (d), (f) — la liste fournisseur, par la route : l'ordre du dépôt
/// (`invoice_date DESC, id DESC`, non déplacé) et un `offset` positif.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn ac324b_d_f_la_liste_fournisseur_par_la_route(pool: MySqlPool) {
    let s = poser_factures_fournisseur(&pool).await;
    let app = spawn_app(pool.clone()).await;
    let jwt = forge_jwt(s.admin_user_id, "Consultation", s.company_id);
    let mut echecs = Vec::new();
    for (chemin, attendu) in [
        ("/api/v1/supplier-invoices", vec!["FA", "FC", "FB"]),
        ("/api/v1/supplier-invoices?limit=1&offset=1", vec!["FC"]),
    ] {
        let (statut, corps) = get_json(&app, &jwt, chemin).await;
        let rendu = numeros_fournisseur(&corps);
        if statut != 200 || rendu != attendu || corps["total"] != 3 {
            echecs.push(format!(
                "{chemin} : attendu 200 {attendu:?}, total 3 ; rendu {statut} {rendu:?}, total {}",
                corps["total"]
            ));
        }
    }
    assert!(echecs.is_empty(), "{}", echecs.join("\n"));
}

/// Un montant décimal d'un corps JSON (rendu en chaîne).
fn montant(v: &Value) -> Option<Decimal> {
    v.as_str().and_then(|s| s.parse().ok())
}

/// AC11 (g) — le détail, par la route : F1, partiellement réglée →
/// `amountSettled` 50.00 et `amountDue` 166.20 ; F5, brouillon →
/// `roundingIsPreview: true` (le réglage est lu) ; F2, validée →
/// `roundingIsPreview: false`.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn ac324b_g_le_detail_d_une_facture_par_la_route(pool: MySqlPool) {
    let f = poser_factures(&pool).await;
    let app = spawn_app(pool.clone()).await;
    let jwt = forge_jwt(f.s.admin_user_id, "Consultation", f.s.company_id);
    let mut echecs = Vec::new();
    let (statut, corps) = get_json(&app, &jwt, &format!("/api/v1/invoices/{}", f.ids[0])).await;
    let (regle, du) = (
        montant(&corps["amountSettled"]),
        montant(&corps["amountDue"]),
    );
    if statut != 200 || regle != Some(dec!(50.00)) || du != Some(dec!(166.20)) {
        echecs.push(format!(
            "F1 : attendu 200, réglé 50.00, dû 166.20 ; rendu {statut}, {regle:?}, {du:?}"
        ));
    }
    let (statut, corps) = get_json(&app, &jwt, &format!("/api/v1/invoices/{}", f.ids[4])).await;
    if statut != 200 || corps["roundingIsPreview"] != true {
        echecs.push(format!(
            "F5 brouillon : attendu roundingIsPreview true ; rendu {statut} {}",
            corps["roundingIsPreview"]
        ));
    }
    let (statut, corps) = get_json(&app, &jwt, &format!("/api/v1/invoices/{}", f.ids[1])).await;
    if statut != 200 || corps["roundingIsPreview"] != false {
        echecs.push(format!(
            "F2 validée : attendu roundingIsPreview false ; rendu {statut} {}",
            corps["roundingIsPreview"]
        ));
    }
    assert!(echecs.is_empty(), "{}", echecs.join("\n"));
}
