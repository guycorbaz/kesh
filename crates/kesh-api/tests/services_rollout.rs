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
//! seed), un contact.

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
use kesh_db::entities::journal_entry::Journal;
use kesh_db::entities::{
    Language, NewCompany, NewJournalEntry, NewJournalEntryLine, OrgType, Role,
};
use kesh_db::errors::DbError;
use kesh_db::repositories::{companies, journal_entries};
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
/// 2000, deux comptes lettrables), un contact.
struct Livres {
    s: SeededCompany,
    ecriture_id: i64,
    contact_id: i64,
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
    Livres {
        ecriture_id: ecriture.entry.id,
        contact_id,
        s,
    }
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
/// parenthèse ouvrante incluse) ; un cas d'ordre ; une description blanche.
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
