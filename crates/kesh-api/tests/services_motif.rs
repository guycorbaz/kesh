//! Story 32-3 — le motif de la couche de services, sur ses deux pilotes
//! (rapports, plan comptable).
//!
//! Ce fichier fige **par la route** ce que chacun des dix-huit handlers de
//! rapport écrit au journal d'audit (AC9 (b′)) : une table unique
//! ([`TABLE_DES_TRACES`]), paramétrée par des symboles (`$fy`, `$debut`…),
//! résolus à chaque jeu.
//!
//! Fixtures **synthétiques** : `seed_accounting_company`, plus une facture
//! validée et non réglée à une ligne au taux normal, une écriture débitant le
//! compte d'impôt préalable, et un projet portant une écriture imputée.
#![allow(clippy::too_many_arguments)]

mod common;

use std::collections::BTreeSet;
use std::net::SocketAddr;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use chrono::{NaiveDate, TimeDelta, Utc};
use jsonwebtoken::{Algorithm, EncodingKey, Header};
use kesh_api::auth::jwt::Claims;
use kesh_api::config::Config;
use kesh_api::errors::AppError;
use kesh_api::middleware::auth::CurrentUser;
use kesh_api::services::acces::{ContexteAcces, Portee};
use kesh_api::services::plan_comptable;
use kesh_api::services::poignees::{Lecture, Trace};
use kesh_api::services::rapports::{
    self, Affichage, DemandeGrandLivre, DemandeJournaux, DemandePeriode, DemandeProjet, Export,
    FormatExport, ModeProjet, Rapport,
};
use kesh_api::{AppState, build_router};
use kesh_db::entities::journal_entry::Journal;
use kesh_db::entities::{NewJournalEntry, NewJournalEntryLine, Role};
use kesh_db::repositories::journal_entries;
use kesh_db::test_fixtures::{SeededCompany, seed_accounting_company};
use kesh_report::{GeneralLedger, MAX_LEDGER_LIMIT};
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use serde_json::{Value, json};
use sqlx::MySqlPool;

use common::lecture_seule::{lecture_seule, pool_lecture_seule};

const TEST_JWT_SECRET: &[u8] = b"test-secret-32-bytes-minimum-test-secret-padding";
const TEST_ADMIN_PASSWORD: &str = "e2e-test-admin-password";

/// Bornes explicites des demandes qui en portent.
const BORNE_DEBUT: &str = "2026-01-01";
const BORNE_FIN: &str = "2026-12-31";
/// Bornes de l'exercice seedé (`kesh_db::test_fixtures::seed_accounting_company`,
/// `crates/kesh-db/src/test_fixtures.rs:118`) — celles d'une demande sans bornes.
const EXERCICE_DEBUT: &str = "2020-01-01";
const EXERCICE_FIN: &str = "2030-12-31";

// ============================================================
// Montage HTTP (recopié de `reports_e2e.rs:54-80`)
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

// ============================================================
// Fixtures synthétiques
// ============================================================

/// Ce que posent les fixtures, et dont les symboles de la table ont besoin.
struct Livres {
    seeded: SeededCompany,
    /// Le projet portant une écriture imputée.
    projet_id: i64,
}

impl Livres {
    fn company_id(&self) -> i64 {
        self.seeded.company_id
    }
    fn user_id(&self) -> i64 {
        self.seeded.admin_user_id
    }
    fn fy(&self) -> i64 {
        self.seeded.fiscal_year_id
    }
    /// Le compte du filtre du grand livre (le compte bancaire seedé).
    fn compte(&self) -> i64 {
        self.seeded.accounts["1100"]
    }
}

/// Pose les livres : société seedée, facture validée et non réglée portant une
/// ligne au taux `8.10` (rapport de TVA, balance âgée), écriture débitant le
/// compte d'impôt préalable (branche récupérable de la TVA), projet portant une
/// écriture imputée (rapports de projet).
///
/// ⚠️ Pas d'`INSERT INTO company_invoice_settings` : `seed_accounting_company`
/// pose déjà la ligne (clé `company_id`).
async fn poser_livres(pool: &MySqlPool) -> Livres {
    use kesh_db::entities::contact::{ContactType, NewContact, Salutation};
    use kesh_db::entities::{NewInvoice, NewInvoiceLine};
    use kesh_db::repositories::{contacts, invoices};

    let seeded = seed_accounting_company(pool).await.unwrap();

    let contact_id = contacts::create(
        pool,
        seeded.admin_user_id,
        NewContact {
            company_id: seeded.company_id,
            contact_type: ContactType::Entreprise,
            name: "Client Synthétique SA".into(),
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
            salutation: Salutation::Neutre,
        },
    )
    .await
    .unwrap()
    .id;

    // TTC 108.10 : multiple de 0.05, l'arrondi à 5 centimes n'intervient pas.
    let date = NaiveDate::from_ymd_opt(2026, 3, 5).unwrap();
    let (facture, _) = invoices::create(
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
                revenue_account_id: None,
                description: "Prestation synthétique".into(),
                quantity: dec!(1),
                unit_price: dec!(100),
                vat_rate: dec!(8.10),
            }],
        },
    )
    .await
    .unwrap();
    invoices::validate_invoice(pool, seeded.company_id, facture.id, seeded.admin_user_id)
        .await
        .unwrap();

    // Impôt préalable : débit du compte récupérable configuré (`1000`).
    poser_ecriture(pool, &seeded, "1000", "2000", dec!(8.10), None).await;

    let projet_id: i64 = sqlx::query_scalar(
        "INSERT INTO projects (company_id, parent_id, code, name, archived) \
         VALUES (?, NULL, 'SYNTH', 'Projet synthétique', FALSE) RETURNING id",
    )
    .bind(seeded.company_id)
    .fetch_one(pool)
    .await
    .unwrap();
    poser_ecriture(pool, &seeded, "4000", "2000", dec!(120), Some(projet_id)).await;

    Livres { seeded, projet_id }
}

/// Une écriture équilibrée au journal des achats, le 2026-06-01.
async fn poser_ecriture(
    pool: &MySqlPool,
    seeded: &SeededCompany,
    debit: &str,
    credit: &str,
    montant: Decimal,
    projet: Option<i64>,
) {
    journal_entries::create(
        pool,
        seeded.fiscal_year_id,
        seeded.admin_user_id,
        NewJournalEntry {
            company_id: seeded.company_id,
            entry_date: NaiveDate::from_ymd_opt(2026, 6, 1).unwrap(),
            journal: Journal::Achats,
            description: "Écriture synthétique".into(),
            project_id: None,
            lines: vec![
                NewJournalEntryLine {
                    account_id: seeded.accounts[debit],
                    debit: montant,
                    credit: Decimal::ZERO,
                    project_id: projet,
                },
                NewJournalEntryLine {
                    account_id: seeded.accounts[credit],
                    debit: Decimal::ZERO,
                    credit: montant,
                    project_id: projet,
                },
            ],
        },
    )
    .await
    .unwrap();
}

// ============================================================
// La table des traces — une ligne par handler (AC6), constante unique
// ============================================================

/// Service de rapport visé par une ligne.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Svc {
    Bilan,
    CompteDeResultat,
    Balance,
    Journaux,
    Tva,
    GrandLivre,
    BalanceAgee,
    DepensesProjet,
    RendementProjet,
}

/// Usage d'une ligne.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Usg {
    Affichage,
    Pdf,
    Csv,
}

impl Usg {
    fn format(self) -> Option<&'static str> {
        match self {
            Usg::Affichage => None,
            Usg::Pdf => Some("pdf"),
            Usg::Csv => Some("csv"),
        }
    }
}

/// La demande d'une ligne, jouée à l'identique au service et par la route.
#[derive(Debug, Clone, Copy)]
enum Dem {
    /// Rapport à période : bornes explicites (`BORNE_*`) ou celles de l'exercice.
    Periode {
        bornes: bool,
        journal: Option<&'static str>,
    },
    /// Grand livre sur `BORNE_*`, filtré sur `$compte` ou non.
    GrandLivre {
        filtre: bool,
    },
    BalanceAgee,
    /// Rapport de projet, en mode `fiscal_year` (exercice, sans bornes) ou
    /// `cumulative`.
    Projet {
        cumule: bool,
    },
}

/// Une ligne de la table des traces.
struct Ligne {
    num: u8,
    nom: &'static str,
    svc: Svc,
    usage: Usg,
    dem: Dem,
    action: &'static str,
    /// `details_json` attendu, écrit avec les symboles `$fy`, `$debut`, `$fin`,
    /// `$du`, `$au`, `$compte`, `$pid`, `$asof`.
    details: &'static str,
}

const SANS_BORNES: Dem = Dem::Periode {
    bornes: false,
    journal: None,
};
const BORNES: Dem = Dem::Periode {
    bornes: true,
    journal: None,
};

/// La table de l'AC9 (b), relue par (b′) — **constante unique**.
const TABLE_DES_TRACES: &[Ligne] = &[
    Ligne {
        num: 1,
        nom: "bilan_affichage",
        svc: Svc::Bilan,
        usage: Usg::Affichage,
        dem: SANS_BORNES,
        action: "report.generated",
        details: r#"{"report_type": "balance-sheet", "fiscal_year_id": "$fy", "period_start": "$debut", "period_end": "$fin", "journal_filter": null}"#,
    },
    Ligne {
        num: 2,
        nom: "bilan_export_pdf",
        svc: Svc::Bilan,
        usage: Usg::Pdf,
        dem: BORNES,
        action: "report.exported",
        details: r#"{"report_type": "balance-sheet", "format": "pdf", "fiscal_year_id": "$fy", "period_start": "$debut", "period_end": "$fin", "journal_filter": null}"#,
    },
    Ligne {
        num: 3,
        nom: "compte_de_resultat_affichage",
        svc: Svc::CompteDeResultat,
        usage: Usg::Affichage,
        dem: BORNES,
        action: "report.generated",
        details: r#"{"report_type": "income-statement", "fiscal_year_id": "$fy", "period_start": "$debut", "period_end": "$fin", "journal_filter": null}"#,
    },
    Ligne {
        num: 4,
        nom: "compte_de_resultat_export_csv",
        svc: Svc::CompteDeResultat,
        usage: Usg::Csv,
        dem: BORNES,
        action: "report.exported",
        details: r#"{"report_type": "income-statement", "format": "csv", "fiscal_year_id": "$fy", "period_start": "$debut", "period_end": "$fin", "journal_filter": null}"#,
    },
    Ligne {
        num: 5,
        nom: "balance_affichage",
        svc: Svc::Balance,
        usage: Usg::Affichage,
        dem: BORNES,
        action: "report.generated",
        details: r#"{"report_type": "trial-balance", "fiscal_year_id": "$fy", "period_start": "$debut", "period_end": "$fin", "journal_filter": null}"#,
    },
    Ligne {
        num: 6,
        nom: "balance_export_pdf",
        svc: Svc::Balance,
        usage: Usg::Pdf,
        dem: BORNES,
        action: "report.exported",
        details: r#"{"report_type": "trial-balance", "format": "pdf", "fiscal_year_id": "$fy", "period_start": "$debut", "period_end": "$fin", "journal_filter": null}"#,
    },
    Ligne {
        num: 7,
        nom: "journaux_affichage_filtre_achats",
        svc: Svc::Journaux,
        usage: Usg::Affichage,
        dem: Dem::Periode {
            bornes: true,
            journal: Some("Achats"),
        },
        action: "report.generated",
        details: r#"{"report_type": "journals", "fiscal_year_id": "$fy", "period_start": "$debut", "period_end": "$fin", "journal_filter": "Achats"}"#,
    },
    Ligne {
        num: 8,
        // Filtrée (revue de code P1, E2) : seule ligne qui voie le filtre de
        // journal d'un **export** tracé ; les exports 2, 4, 6 et 10 tiennent
        // le `null`.
        nom: "journaux_export_csv_filtre_achats",
        svc: Svc::Journaux,
        usage: Usg::Csv,
        dem: Dem::Periode {
            bornes: true,
            journal: Some("Achats"),
        },
        action: "report.exported",
        details: r#"{"report_type": "journals", "format": "csv", "fiscal_year_id": "$fy", "period_start": "$debut", "period_end": "$fin", "journal_filter": "Achats"}"#,
    },
    Ligne {
        num: 9,
        nom: "tva_affichage",
        svc: Svc::Tva,
        usage: Usg::Affichage,
        dem: BORNES,
        action: "report.generated",
        details: r#"{"report_type": "vat", "fiscal_year_id": "$fy", "period_start": "$debut", "period_end": "$fin", "journal_filter": null}"#,
    },
    Ligne {
        num: 10,
        nom: "tva_export_pdf",
        svc: Svc::Tva,
        usage: Usg::Pdf,
        dem: BORNES,
        action: "report.exported",
        details: r#"{"report_type": "vat", "format": "pdf", "fiscal_year_id": "$fy", "period_start": "$debut", "period_end": "$fin", "journal_filter": null}"#,
    },
    Ligne {
        num: 11,
        nom: "grand_livre_affichage_filtre_compte",
        svc: Svc::GrandLivre,
        usage: Usg::Affichage,
        dem: Dem::GrandLivre { filtre: true },
        action: "report.generated",
        details: r#"{"report_type": "general-ledger", "period_start": "$du", "period_end": "$au", "account_ids": ["$compte"]}"#,
    },
    // #729 : l'export du grand livre est tracé en consultation, sans format —
    // comportement d'aujourd'hui, conservé ; son correctif modifiera cette ligne
    // en connaissance de cause.
    Ligne {
        num: 12,
        nom: "grand_livre_export_pdf_export_grand_livre_trace_en_consultation_729",
        svc: Svc::GrandLivre,
        usage: Usg::Pdf,
        dem: Dem::GrandLivre { filtre: false },
        action: "report.generated",
        details: r#"{"report_type": "general-ledger", "period_start": "$du", "period_end": "$au", "account_ids": null}"#,
    },
    Ligne {
        num: 13,
        nom: "balance_agee_affichage",
        svc: Svc::BalanceAgee,
        usage: Usg::Affichage,
        dem: Dem::BalanceAgee,
        action: "report.generated",
        details: r#"{"report_type": "aged-receivables", "as_of": "$asof"}"#,
    },
    Ligne {
        num: 14,
        nom: "balance_agee_export_csv",
        svc: Svc::BalanceAgee,
        usage: Usg::Csv,
        dem: Dem::BalanceAgee,
        action: "report.exported",
        details: r#"{"report_type": "aged-receivables", "format": "csv", "as_of": "$asof"}"#,
    },
    Ligne {
        num: 15,
        nom: "depenses_projet_affichage_fiscal_year",
        svc: Svc::DepensesProjet,
        usage: Usg::Affichage,
        dem: Dem::Projet { cumule: false },
        action: "report.generated",
        details: r#"{"report_type": "project-expenses", "project_id": "$pid", "mode": "fiscal_year"}"#,
    },
    Ligne {
        num: 16,
        nom: "depenses_projet_export_pdf_fiscal_year",
        svc: Svc::DepensesProjet,
        usage: Usg::Pdf,
        dem: Dem::Projet { cumule: false },
        action: "report.exported",
        details: r#"{"report_type": "project-expenses", "format": "pdf", "project_id": "$pid", "mode": "fiscal_year"}"#,
    },
    Ligne {
        num: 17,
        nom: "rendement_projet_affichage_cumulative",
        svc: Svc::RendementProjet,
        usage: Usg::Affichage,
        dem: Dem::Projet { cumule: true },
        action: "report.generated",
        details: r#"{"report_type": "project-return", "project_id": "$pid", "mode": "cumulative"}"#,
    },
    Ligne {
        num: 18,
        nom: "rendement_projet_export_csv_cumulative",
        svc: Svc::RendementProjet,
        usage: Usg::Csv,
        dem: Dem::Projet { cumule: true },
        action: "report.exported",
        details: r#"{"report_type": "project-return", "format": "csv", "project_id": "$pid", "mode": "cumulative"}"#,
    },
    Ligne {
        num: 19,
        // Story 32-4a (#738 P2-2) : seule ligne qui trace les journaux SANS
        // filtre — un filtre absent tracé comme présent n'était vu par aucune
        // ligne ; la 7 reste filtrée (seule à exercer « affichage avec
        // filtre »). Deuxième ligne du couple (Journaux, Affichage).
        nom: "journaux_affichage_sans_filtre",
        svc: Svc::Journaux,
        usage: Usg::Affichage,
        dem: BORNES,
        action: "report.generated",
        details: r#"{"report_type": "journals", "fiscal_year_id": "$fy", "period_start": "$debut", "period_end": "$fin", "journal_filter": null}"#,
    },
];

/// Les valeurs des symboles d'une ligne.
struct Symboles {
    fy: i64,
    debut: String,
    fin: String,
    du: String,
    au: String,
    compte: i64,
    pid: i64,
    asof: String,
}

impl Symboles {
    fn valeur(&self, symbole: &str) -> Value {
        match symbole {
            "$fy" => json!(self.fy),
            "$debut" => json!(self.debut),
            "$fin" => json!(self.fin),
            "$du" => json!(self.du),
            "$au" => json!(self.au),
            "$compte" => json!(self.compte),
            "$pid" => json!(self.pid),
            "$asof" => json!(self.asof),
            autre => panic!("symbole inconnu dans la table : {autre}"),
        }
    }
}

/// Résout le gabarit `details` d'une ligne : chaque chaîne `$…` est remplacée
/// par la valeur de son symbole.
fn resoudre(gabarit: &str, symboles: &Symboles) -> Value {
    fn remplacer(v: &mut Value, s: &Symboles) {
        match v {
            Value::String(x) if x.starts_with('$') => *v = s.valeur(x),
            Value::Array(a) => a.iter_mut().for_each(|e| remplacer(e, s)),
            Value::Object(o) => o.values_mut().for_each(|e| remplacer(e, s)),
            _ => {}
        }
    }
    let mut v: Value = serde_json::from_str(gabarit).expect("gabarit JSON valide");
    remplacer(&mut v, symboles);
    v
}

/// Une trace lue au journal d'audit.
#[derive(Debug)]
struct LigneAudit {
    action: String,
    entity_type: String,
    entity_id: i64,
    details: Value,
}

/// Plus grand `id` de `audit_log` (0 si vide) — repère avant un appel.
async fn repere_audit(pool: &MySqlPool) -> i64 {
    sqlx::query_scalar::<_, Option<i64>>("SELECT MAX(id) FROM audit_log")
        .fetch_one(pool)
        .await
        .unwrap()
        .unwrap_or(0)
}

/// Les traces de l'utilisateur écrites après le repère.
async fn traces_depuis(pool: &MySqlPool, user_id: i64, repere: i64) -> Vec<LigneAudit> {
    let rows: Vec<(String, String, i64, Option<Vec<u8>>)> = sqlx::query_as(
        "SELECT action, entity_type, entity_id, details_json FROM audit_log \
         WHERE user_id = ? AND id > ? ORDER BY id",
    )
    .bind(user_id)
    .bind(repere)
    .fetch_all(pool)
    .await
    .unwrap();
    rows.into_iter()
        .map(|(action, entity_type, entity_id, details)| LigneAudit {
            action,
            entity_type,
            entity_id,
            details: details
                .map(|b| serde_json::from_slice(&b).expect("details_json JSON valide"))
                .unwrap_or(Value::Null),
        })
        .collect()
}

/// Vérifie qu'un appel a écrit **exactement une** trace, égale à la ligne de la
/// table (pour l'un des `attendus` — plusieurs seulement pour `$asof`, accepté
/// à la date d'avant ou d'après l'appel). Rend le message d'échec, nommé.
fn verifier_trace(
    volet: &str,
    ligne: &Ligne,
    traces: &[LigneAudit],
    attendus: &[Value],
) -> Result<(), String> {
    let nom = format!("{volet}#{} {}", ligne.num, ligne.nom);
    if traces.len() != 1 {
        return Err(format!(
            "{nom} : {} trace(s) neuve(s), une attendue : {traces:?}",
            traces.len()
        ));
    }
    let t = &traces[0];
    if t.action != ligne.action {
        return Err(format!(
            "{nom} : action {:?}, attendue {:?}",
            t.action, ligne.action
        ));
    }
    if t.entity_type != "report" || t.entity_id != 0 {
        return Err(format!(
            "{nom} : entité ({:?}, {}), attendue (\"report\", 0)",
            t.entity_type, t.entity_id
        ));
    }
    if !attendus.contains(&t.details) {
        return Err(format!(
            "{nom} : details_json {}, attendu {}",
            t.details,
            attendus
                .iter()
                .map(Value::to_string)
                .collect::<Vec<_>>()
                .join(" ou ")
        ));
    }
    Ok(())
}

// ============================================================
// AC9 (b′) — la même table, par la route
// ============================================================

/// La requête HTTP d'une ligne (route du tableau de l'AC6).
fn requete(ligne: &Ligne, livres: &Livres) -> String {
    let base = |chemin: &str| {
        let export = if ligne.usage == Usg::Affichage {
            ""
        } else {
            "/export"
        };
        format!("/api/v1/reports/{chemin}{export}")
    };
    let mut params: Vec<String> = Vec::new();
    let chemin = match ligne.dem {
        Dem::Periode { bornes, journal } => {
            params.push(format!("fiscalYearId={}", livres.fy()));
            if bornes {
                params.push(format!("periodStart={BORNE_DEBUT}&periodEnd={BORNE_FIN}"));
            }
            if let Some(j) = journal {
                params.push(format!("journal={j}"));
            }
            base(match ligne.svc {
                Svc::Bilan => "balance-sheet",
                Svc::CompteDeResultat => "income-statement",
                Svc::Balance => "trial-balance",
                Svc::Journaux => "journals",
                Svc::Tva => "vat",
                autre => panic!("{autre:?} n'est pas un rapport à période"),
            })
        }
        Dem::GrandLivre { filtre } => {
            params.push(format!("from={BORNE_DEBUT}&to={BORNE_FIN}"));
            if filtre {
                params.push(format!("accountIds={}", livres.compte()));
            }
            base("general-ledger")
        }
        Dem::BalanceAgee => base("aged-receivables"),
        Dem::Projet { cumule } => {
            params.push(format!("projectId={}", livres.projet_id));
            if cumule {
                params.push("mode=cumulative".into());
            } else {
                params.push(format!("mode=fiscal_year&fiscalYearId={}", livres.fy()));
            }
            base(match ligne.svc {
                Svc::DepensesProjet => "project-expenses",
                Svc::RendementProjet => "project-return",
                autre => panic!("{autre:?} n'est pas un rapport de projet"),
            })
        }
    };
    if let Some(f) = ligne.usage.format() {
        params.push(format!("format={f}"));
    }
    if params.is_empty() {
        chemin
    } else {
        format!("{chemin}?{}", params.join("&"))
    }
}

/// Les symboles d'une ligne jouée par la route : `debut`/`fin` sont ceux de la
/// demande, ou ceux de l'exercice seedé pour la ligne sans bornes.
fn symboles_route(ligne: &Ligne, livres: &Livres, asof: NaiveDate) -> Symboles {
    let (debut, fin) = match ligne.dem {
        Dem::Periode { bornes: false, .. } => (EXERCICE_DEBUT, EXERCICE_FIN),
        _ => (BORNE_DEBUT, BORNE_FIN),
    };
    Symboles {
        fy: livres.fy(),
        debut: debut.into(),
        fin: fin.into(),
        du: BORNE_DEBUT.into(),
        au: BORNE_FIN.into(),
        compte: livres.compte(),
        pid: livres.projet_id,
        asof: asof.format("%Y-%m-%d").to_string(),
    }
}

/// AC9 (b′) — pour chacune des dix-neuf lignes, la requête de sa route, puis
/// exactement une trace neuve égale à la ligne. Seul volet qui voie un handler
/// appeler le mauvais service.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn table_des_traces_par_la_route(pool: MySqlPool) {
    let livres = poser_livres(&pool).await;
    let app = spawn_app(pool.clone()).await;
    let jwt = forge_jwt(livres.user_id(), "Admin", livres.company_id());
    assert_eq!(
        TABLE_DES_TRACES.len(),
        19,
        "dix-neuf lignes pour dix-huit couples (service, usage)"
    );

    let mut echecs = Vec::new();
    for ligne in TABLE_DES_TRACES {
        let avant = Utc::now().naive_utc().date();
        let repere = repere_audit(&pool).await;
        let url = requete(ligne, &livres);
        let resp = app
            .client
            .get(app.url(&url))
            .bearer_auth(&jwt)
            .send()
            .await
            .unwrap();
        let apres = Utc::now().naive_utc().date();
        if resp.status() != 200 {
            echecs.push(format!(
                "route#{} {} : {url} → {}",
                ligne.num,
                ligne.nom,
                resp.status()
            ));
            continue;
        }
        let traces = traces_depuis(&pool, livres.user_id(), repere).await;
        let attendus: Vec<Value> = [avant, apres]
            .iter()
            .map(|d| resoudre(ligne.details, &symboles_route(ligne, &livres, *d)))
            .collect();
        if let Err(e) = verifier_trace("route", ligne, &traces, &attendus) {
            echecs.push(e);
        }
    }
    assert!(echecs.is_empty(), "AC9 (b′) :\n{}", echecs.join("\n"));
}

// ============================================================
// Au niveau du service : contextes, poignées, demandes
// ============================================================

/// Les neuf services de rapport.
const SERVICES: [Svc; 9] = [
    Svc::Bilan,
    Svc::CompteDeResultat,
    Svc::Balance,
    Svc::Journaux,
    Svc::Tva,
    Svc::GrandLivre,
    Svc::BalanceAgee,
    Svc::DepensesProjet,
    Svc::RendementProjet,
];

/// Un id positif qu'aucune fixture ne pose.
const INEXISTANT: i64 = 999_999_999;

// `pool_lecture_seule` et `lecture_seule` (la `Lecture` du chapeau de l'AC9) :
// `tests/common/lecture_seule.rs`, partagées avec `services_rollout.rs`
// (Story 32-4a, C-32-4a-9).

fn mcp(livres: &Livres, portees: &[Portee]) -> ContexteAcces {
    ContexteAcces::mcp(
        livres.company_id(),
        livres.user_id(),
        portees.iter().copied().collect::<BTreeSet<_>>(),
    )
}

fn mcp_societe(company_id: i64, user_id: i64, portees: &[Portee]) -> ContexteAcces {
    ContexteAcces::mcp(company_id, user_id, portees.iter().copied().collect())
}

const TOUTES: [Portee; 3] = [
    Portee::BooksRead,
    Portee::InvoicesRead,
    Portee::ContactsRead,
];

fn api(livres: &Livres, role: Role) -> ContexteAcces {
    api_societe(livres.company_id(), livres.user_id(), role)
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

/// Portée qu'exige l'affichage d'un service.
fn portee_exigee(svc: Svc) -> Portee {
    match svc {
        Svc::BalanceAgee => Portee::InvoicesRead,
        _ => Portee::BooksRead,
    }
}

/// L'autre portée — insuffisante seule.
fn autre_portee(svc: Svc) -> Portee {
    match portee_exigee(svc) {
        Portee::BooksRead => Portee::InvoicesRead,
        Portee::InvoicesRead => Portee::BooksRead,
        // Aucun rapport n'exige la portée du carnet (Story 32-4a) : bras posé
        // pour l'exhaustivité, jamais atteint par `portee_exigee`.
        Portee::ContactsRead => panic!("aucun service de rapport n'exige ContactsRead"),
    }
}

/// Demande par défaut d'un service.
fn dem_defaut(svc: Svc) -> Dem {
    match svc {
        Svc::GrandLivre => Dem::GrandLivre { filtre: false },
        Svc::BalanceAgee => Dem::BalanceAgee,
        Svc::DepensesProjet | Svc::RendementProjet => Dem::Projet { cumule: false },
        _ => BORNES,
    }
}

/// Usage d'export d'un service (la balance âgée n'exporte qu'en CSV).
fn usage_export(svc: Svc) -> Usg {
    match svc {
        Svc::BalanceAgee => Usg::Csv,
        _ => Usg::Pdf,
    }
}

/// Paramètres des demandes — modifiés par les cas (c) de l'AC8.
#[derive(Debug, Clone, Copy)]
struct Params {
    fy: i64,
    pid: i64,
    compte: i64,
    /// Grand livre : bornes inversées (`from > to`).
    gl_inverse: bool,
}

impl Params {
    fn de(livres: &Livres) -> Self {
        Self {
            fy: livres.fy(),
            pid: livres.projet_id,
            compte: livres.compte(),
            gl_inverse: false,
        }
    }
}

fn date(s: &str) -> NaiveDate {
    NaiveDate::parse_from_str(s, "%Y-%m-%d").unwrap()
}

fn demande_periode(dem: Dem, p: &Params) -> DemandePeriode {
    let bornes = matches!(dem, Dem::Periode { bornes: true, .. });
    DemandePeriode {
        fiscal_year_id: p.fy,
        period_start: bornes.then(|| date(BORNE_DEBUT)),
        period_end: bornes.then(|| date(BORNE_FIN)),
    }
}

fn demande_journaux(dem: Dem, p: &Params) -> DemandeJournaux {
    let journal = match dem {
        Dem::Periode {
            journal: Some("Achats"),
            ..
        } => Some(Journal::Achats),
        Dem::Periode {
            journal: Some(autre),
            ..
        } => panic!("journal non prévu par la table : {autre}"),
        _ => None,
    };
    DemandeJournaux {
        periode: demande_periode(dem, p),
        journal,
    }
}

fn demande_grand_livre(dem: Dem, p: &Params) -> DemandeGrandLivre {
    let filtre = matches!(dem, Dem::GrandLivre { filtre: true });
    let (from, to) = if p.gl_inverse {
        (date(BORNE_FIN), date(BORNE_DEBUT))
    } else {
        (date(BORNE_DEBUT), date(BORNE_FIN))
    };
    DemandeGrandLivre {
        from,
        to,
        account_ids: filtre.then(|| p.compte.to_string()),
        include_zero: false,
        offset: None,
        limit: None,
    }
}

fn demande_projet(dem: Dem, p: &Params) -> DemandeProjet {
    let cumule = matches!(dem, Dem::Projet { cumule: true });
    DemandeProjet {
        project_id: p.pid,
        mode: if cumule {
            ModeProjet::Cumule
        } else {
            ModeProjet::Exercice
        },
        fiscal_year_id: (!cumule).then_some(p.fy),
        period_start: None,
        period_end: None,
    }
}

/// Ce qu'expose un `Rapport` — relevé **avant** `afficher`/`exporter`.
#[derive(Debug, Default)]
struct Releve {
    periode: Option<(NaiveDate, NaiveDate)>,
    bornes_gl: Option<(NaiveDate, NaiveDate)>,
    as_of: Option<NaiveDate>,
    /// `AgedReceivables.as_of` du contenu (balance âgée seulement).
    as_of_contenu: Option<NaiveDate>,
}

fn relever<T, U>(r: &Rapport<T, U>) -> Releve {
    Releve {
        periode: r.periode().map(|p| (p.start_date, p.end_date)),
        bornes_gl: r.bornes_grand_livre().map(|p| (p.from, p.to)),
        as_of: r.as_of(),
        as_of_contenu: None,
    }
}

/// Ce que fait le test du `Rapport` rendu par le service.
#[derive(Clone, Copy)]
enum Fin<'a> {
    /// L'abandonner sans tracer.
    Aucune,
    /// `afficher`/`exporter` sur cette `Trace`, rendu réussi.
    Tracer(&'a Trace),
    /// `exporter` avec un rendu qui échoue (en affichage : comme `Tracer`).
    RenduEnEchec(&'a Trace),
}

impl<'a> Fin<'a> {
    fn trace(self) -> Option<&'a Trace> {
        match self {
            Fin::Aucune => None,
            Fin::Tracer(t) | Fin::RenduEnEchec(t) => Some(t),
        }
    }
}

fn format_de(usage: Usg) -> FormatExport {
    match usage {
        Usg::Pdf => FormatExport::Pdf,
        Usg::Csv => FormatExport::Csv,
        Usg::Affichage => panic!("pas de format en affichage"),
    }
}

fn rendu_en_echec() -> AppError {
    AppError::Internal("rendu en échec (test)".into())
}

/// Appelle un service générique sur l'usage, puis applique la `Fin`.
macro_rules! servir {
    ($usage:expr, $fin:expr, |$u:ident| $appel:expr) => {{
        match $usage {
            Usg::Affichage => {
                let $u = Affichage;
                let r = $appel.await?;
                let rel = relever(&r);
                if let Some(t) = $fin.trace() {
                    let _ = r.afficher(t).await;
                }
                rel
            }
            Usg::Pdf | Usg::Csv => {
                let $u = Export(format_de($usage));
                let r = $appel.await?;
                let rel = relever(&r);
                match $fin {
                    Fin::Aucune => {}
                    Fin::Tracer(t) => {
                        r.exporter(t, |_| Ok(Vec::new())).await?;
                    }
                    Fin::RenduEnEchec(t) => {
                        r.exporter(t, |_| Err(rendu_en_echec())).await?;
                    }
                }
                rel
            }
        }
    }};
}

/// Joue un service de rapport (service, usage, demande) dans un contexte et
/// sur une `Lecture`, puis applique la `Fin`.
async fn jouer(
    svc: Svc,
    usage: Usg,
    dem: Dem,
    p: &Params,
    a: &ContexteAcces,
    l: &Lecture,
    fin: Fin<'_>,
) -> Result<Releve, AppError> {
    let rel = match svc {
        Svc::Bilan => servir!(usage, fin, |u| rapports::bilan(
            a,
            l,
            demande_periode(dem, p),
            u
        )),
        Svc::CompteDeResultat => servir!(usage, fin, |u| rapports::compte_de_resultat(
            a,
            l,
            demande_periode(dem, p),
            u
        )),
        Svc::Balance => servir!(usage, fin, |u| rapports::balance(
            a,
            l,
            demande_periode(dem, p),
            u
        )),
        Svc::Journaux => servir!(usage, fin, |u| rapports::journaux(
            a,
            l,
            demande_journaux(dem, p),
            u
        )),
        Svc::Tva => servir!(usage, fin, |u| rapports::tva(
            a,
            l,
            demande_periode(dem, p),
            u
        )),
        Svc::GrandLivre => servir!(usage, fin, |u| rapports::grand_livre(
            a,
            l,
            demande_grand_livre(dem, p),
            u
        )),
        Svc::DepensesProjet => servir!(usage, fin, |u| rapports::depenses_projet(
            a,
            l,
            demande_projet(dem, p),
            u
        )),
        Svc::RendementProjet => servir!(usage, fin, |u| rapports::rendement_projet(
            a,
            l,
            demande_projet(dem, p),
            u
        )),
        Svc::BalanceAgee => match usage {
            Usg::Affichage => {
                let r = rapports::balance_agee(a, l, Affichage).await?;
                let mut rel = relever(&r);
                if let Some(t) = fin.trace() {
                    rel.as_of_contenu = Some(r.afficher(t).await.as_of);
                }
                rel
            }
            Usg::Pdf | Usg::Csv => {
                let r = rapports::balance_agee(a, l, Export(format_de(usage))).await?;
                let mut rel = relever(&r);
                let mut vu = None;
                match fin {
                    Fin::Aucune => {}
                    Fin::Tracer(t) => {
                        r.exporter(t, |c| {
                            vu = Some(c.as_of);
                            Ok(Vec::new())
                        })
                        .await?;
                    }
                    Fin::RenduEnEchec(t) => {
                        r.exporter(t, |_| Err(rendu_en_echec())).await?;
                    }
                }
                rel.as_of_contenu = vu;
                rel
            }
        },
    };
    Ok(rel)
}

/// Résumé d'un résultat pour les messages d'échec.
fn resume(r: &Result<Releve, AppError>) -> String {
    match r {
        Ok(_) => "Ok".into(),
        Err(e) => format!("Err({e:?})"),
    }
}

// ============================================================
// AC8 — un test de refus par service, et son contrôle positif
// ============================================================

/// AC8 (a) — canal MCP sans la portée exigée (portées vides, puis l'autre
/// portée seule) → `Forbidden`, pour les dix services.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn ac8_a_refus_mcp_sans_la_portee(pool: MySqlPool) {
    let livres = poser_livres(&pool).await;
    let lecture = Lecture::new(pool.clone());
    let p = Params::de(&livres);
    let mut echecs = Vec::new();
    for svc in SERVICES {
        for portees in [vec![], vec![autre_portee(svc)]] {
            let r = jouer(
                svc,
                Usg::Affichage,
                dem_defaut(svc),
                &p,
                &mcp(&livres, &portees),
                &lecture,
                Fin::Aucune,
            )
            .await;
            if !matches!(r, Err(AppError::Forbidden)) {
                echecs.push(format!(
                    "AC8 (a) {svc:?} portées {portees:?} : {}",
                    resume(&r)
                ));
            }
        }
    }
    for portees in [vec![], vec![Portee::InvoicesRead]] {
        let r = plan_comptable::lister(&mcp(&livres, &portees), &lecture, false).await;
        if !matches!(r, Err(AppError::Forbidden)) {
            echecs.push(format!(
                "AC8 (a) lister portées {portees:?} : {:?}",
                r.err()
            ));
        }
    }
    assert!(echecs.is_empty(), "{}", echecs.join("\n"));
}

/// AC8 (b) — contrôle positif : canal MCP avec la portée exigée → `Ok`.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn ac8_b_controle_positif_mcp_avec_la_portee(pool: MySqlPool) {
    let livres = poser_livres(&pool).await;
    let lecture = Lecture::new(pool.clone());
    let p = Params::de(&livres);
    let mut echecs = Vec::new();
    for svc in SERVICES {
        let r = jouer(
            svc,
            Usg::Affichage,
            dem_defaut(svc),
            &p,
            &mcp(&livres, &[portee_exigee(svc)]),
            &lecture,
            Fin::Aucune,
        )
        .await;
        if r.is_err() {
            echecs.push(format!("AC8 (b) {svc:?} : {}", resume(&r)));
        }
    }
    let r = plan_comptable::lister(&mcp(&livres, &[Portee::BooksRead]), &lecture, false).await;
    if let Err(e) = r {
        echecs.push(format!("AC8 (b) lister : {e:?}"));
    }
    assert!(echecs.is_empty(), "{}", echecs.join("\n"));
}

/// Un cas (c) : sans portée → `Forbidden` ; avec la portée (contrôle) → une
/// erreur qui n'est pas `Forbidden` — la demande échoue bien sans le contrôle.
async fn cas_c(
    volet: &str,
    svc: Svc,
    p: &Params,
    livres: &Livres,
    lecture: &Lecture,
    echecs: &mut Vec<String>,
) {
    let dem = dem_defaut(svc);
    let refuse = jouer(
        svc,
        Usg::Affichage,
        dem,
        p,
        &mcp(livres, &[]),
        lecture,
        Fin::Aucune,
    )
    .await;
    if !matches!(refuse, Err(AppError::Forbidden)) {
        echecs.push(format!(
            "AC8 ({volet}) {svc:?} sans portée : {}",
            resume(&refuse)
        ));
    }
    let controle = jouer(
        svc,
        Usg::Affichage,
        dem,
        p,
        &mcp(livres, &[portee_exigee(svc)]),
        lecture,
        Fin::Aucune,
    )
    .await;
    if !matches!(controle, Err(ref e) if !matches!(e, AppError::Forbidden)) {
        echecs.push(format!(
            "AC8 ({volet}) {svc:?} contrôle avec portée — la demande devait échouer : {}",
            resume(&controle)
        ));
    }
}

/// AC8 (c-val) — le refus précède la **validation** : `fiscalYearId = 0`
/// (rapports à période), période inversée (grand livre), `projectId = 0`
/// (rapports de projet). Sans objet pour la balance âgée.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn ac8_c_val_refus_avant_la_validation(pool: MySqlPool) {
    let livres = poser_livres(&pool).await;
    let lecture = Lecture::new(pool.clone());
    let base = Params::de(&livres);
    let mut echecs = Vec::new();
    for svc in SERVICES {
        let p = match svc {
            Svc::BalanceAgee => continue,
            Svc::GrandLivre => Params {
                gl_inverse: true,
                ..base
            },
            Svc::DepensesProjet | Svc::RendementProjet => Params { pid: 0, ..base },
            _ => Params { fy: 0, ..base },
        };
        cas_c("c-val", svc, &p, &livres, &lecture, &mut echecs).await;
    }
    assert!(echecs.is_empty(), "{}", echecs.join("\n"));
}

/// AC8 (c-lec) — le refus précède la **lecture** : exercice inexistant
/// (rapports à période), projet inexistant (rapports de projet), société
/// inexistante (`lister`). Le grand livre ne lit, avant sa génération, aucune
/// entité que nomme la demande, et la demande de la balance âgée ne porte rien :
/// pas de cas pour eux.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn ac8_c_lec_refus_avant_la_lecture(pool: MySqlPool) {
    let livres = poser_livres(&pool).await;
    let lecture = Lecture::new(pool.clone());
    let base = Params::de(&livres);
    let mut echecs = Vec::new();
    for svc in SERVICES {
        let p = match svc {
            Svc::BalanceAgee | Svc::GrandLivre => continue,
            Svc::DepensesProjet | Svc::RendementProjet => Params {
                pid: INEXISTANT,
                ..base
            },
            _ => Params {
                fy: INEXISTANT,
                ..base
            },
        };
        cas_c("c-lec", svc, &p, &livres, &lecture, &mut echecs).await;
    }
    let refuse = plan_comptable::lister(
        &mcp_societe(INEXISTANT, livres.user_id(), &[]),
        &lecture,
        false,
    )
    .await;
    if !matches!(refuse, Err(AppError::Forbidden)) {
        echecs.push(format!(
            "AC8 (c-lec) lister sans portée : {:?}",
            refuse.err()
        ));
    }
    let controle = plan_comptable::lister(
        &mcp_societe(INEXISTANT, livres.user_id(), &[Portee::BooksRead]),
        &lecture,
        false,
    )
    .await;
    if !matches!(controle, Err(AppError::Internal(_))) {
        echecs.push(format!(
            "AC8 (c-lec) lister contrôle — Internal attendu : {:?}",
            controle.err()
        ));
    }
    assert!(echecs.is_empty(), "{}", echecs.join("\n"));
}

/// AC8 (d) — les exports ne sont jamais servis au MCP, même avec toutes les
/// portées.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn ac8_d_exports_jamais_servis_au_mcp(pool: MySqlPool) {
    let livres = poser_livres(&pool).await;
    let lecture = Lecture::new(pool.clone());
    let p = Params::de(&livres);
    let mut echecs = Vec::new();
    for svc in SERVICES {
        let r = jouer(
            svc,
            usage_export(svc),
            dem_defaut(svc),
            &p,
            &mcp(&livres, &TOUTES),
            &lecture,
            Fin::Aucune,
        )
        .await;
        if !matches!(r, Err(AppError::Forbidden)) {
            echecs.push(format!("AC8 (d) {svc:?} export : {}", resume(&r)));
        }
    }
    assert!(echecs.is_empty(), "{}", echecs.join("\n"));
}

/// AC8 (e) — canal API : l'export de la balance âgée exige `Comptable` ;
/// l'affichage de chaque rapport, les autres exports et `lister` sont servis à
/// `Consultation`.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn ac8_e_canal_api(pool: MySqlPool) {
    let livres = poser_livres(&pool).await;
    let lecture = Lecture::new(pool.clone());
    let p = Params::de(&livres);
    let mut echecs = Vec::new();

    let agee = |role| {
        let a = api(&livres, role);
        let lecture = lecture.clone();
        async move {
            jouer(
                Svc::BalanceAgee,
                Usg::Csv,
                Dem::BalanceAgee,
                &p,
                &a,
                &lecture,
                Fin::Aucune,
            )
            .await
        }
    };
    let r = agee(Role::Consultation).await;
    if !matches!(r, Err(AppError::Forbidden)) {
        echecs.push(format!(
            "AC8 (e) balance_agee export Consultation : {}",
            resume(&r)
        ));
    }
    let r = agee(Role::Comptable).await;
    if r.is_err() {
        echecs.push(format!(
            "AC8 (e) balance_agee export Comptable : {}",
            resume(&r)
        ));
    }

    let consultation = api(&livres, Role::Consultation);
    for svc in SERVICES {
        let mut usages = vec![Usg::Affichage];
        if svc != Svc::BalanceAgee {
            usages.push(usage_export(svc));
        }
        for usage in usages {
            let r = jouer(
                svc,
                usage,
                dem_defaut(svc),
                &p,
                &consultation,
                &lecture,
                Fin::Aucune,
            )
            .await;
            if r.is_err() {
                echecs.push(format!(
                    "AC8 (e) {svc:?} {usage:?} Consultation : {}",
                    resume(&r)
                ));
            }
        }
    }
    if let Err(e) = plan_comptable::lister(&consultation, &lecture, false).await {
        echecs.push(format!("AC8 (e) lister Consultation : {e:?}"));
    }
    assert!(echecs.is_empty(), "{}", echecs.join("\n"));
}

/// AC8 — l'aide `identite_societe` : jamais servie au MCP (le refus précède la
/// lecture, même sur une société inexistante) ; servie à `Consultation`.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn ac8_identite_societe(pool: MySqlPool) {
    let livres = poser_livres(&pool).await;
    let lecture = Lecture::new(pool.clone());
    let mut echecs = Vec::new();

    let r = rapports::identite_societe(&mcp(&livres, &TOUTES), &lecture).await;
    if !matches!(r, Err(AppError::Forbidden)) {
        echecs.push(format!(
            "AC8 identite_societe MCP toutes portées : {:?}",
            r.err()
        ));
    }
    let r = rapports::identite_societe(
        &mcp_societe(INEXISTANT, livres.user_id(), &TOUTES),
        &lecture,
    )
    .await;
    if !matches!(r, Err(AppError::Forbidden)) {
        echecs.push(format!(
            "AC8 identite_societe MCP société inexistante : {:?}",
            r.err()
        ));
    }
    match rapports::identite_societe(&api(&livres, Role::Consultation), &lecture).await {
        Ok((nom, _)) if nom == "CI Test Company" => {}
        autre => echecs.push(format!("AC8 identite_societe API Consultation : {autre:?}")),
    }
    // Contrôle : sans refus, la société inexistante échoue bien à la lecture.
    let r = rapports::identite_societe(
        &api_societe(INEXISTANT, livres.user_id(), Role::Consultation),
        &lecture,
    )
    .await;
    if !matches!(r, Err(ref e) if !matches!(e, AppError::Forbidden)) {
        echecs.push(format!(
            "AC8 identite_societe contrôle société inexistante : {r:?}"
        ));
    }
    assert!(echecs.is_empty(), "{}", echecs.join("\n"));
}

// ============================================================
// AC9 — la trace s'écrit, et sur `Trace`
// ============================================================

/// Témoin du chapeau de l'AC9 : la `Lecture` de test refuse bien l'écriture
/// (1792) — sans quoi M1 serait vert.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn ac9_temoin_la_lecture_de_test_refuse_l_ecriture(pool: MySqlPool) {
    let livres = poser_livres(&pool).await;
    let ro = pool_lecture_seule(&pool).await;
    let e = sqlx::query(
        "INSERT INTO audit_log (user_id, action, entity_type, entity_id) \
         VALUES (?, 'report.generated', 'report', 0)",
    )
    .bind(livres.user_id())
    .execute(&ro)
    .await
    .expect_err("l'INSERT doit échouer sur la connexion en lecture seule");
    let numero = e
        .as_database_error()
        .and_then(|d| d.try_downcast_ref::<sqlx::mysql::MySqlDatabaseError>())
        .map(|m| m.number());
    assert_eq!(numero, Some(1792), "erreur attendue 1792 : {e:?}");
}

/// AC9 (a) — les neuf services (affichage et export), `lister` et
/// `identite_societe` réussissent sur une `Lecture` en lecture seule.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn ac9_a_les_services_lisent_sur_une_lecture_en_lecture_seule(pool: MySqlPool) {
    let livres = poser_livres(&pool).await;
    let lecture = lecture_seule(&pool).await;
    let trace = Trace::new(pool.clone());
    let admin = api(&livres, Role::Admin);
    let p = Params::de(&livres);
    let mut echecs = Vec::new();
    for svc in SERVICES {
        for usage in [Usg::Affichage, usage_export(svc)] {
            let r = jouer(
                svc,
                usage,
                dem_defaut(svc),
                &p,
                &admin,
                &lecture,
                Fin::Tracer(&trace),
            )
            .await;
            if r.is_err() {
                echecs.push(format!("AC9 (a) {svc:?} {usage:?} : {}", resume(&r)));
            }
        }
    }
    match plan_comptable::lister(&admin, &lecture, true).await {
        Ok(comptes) if comptes.len() == 5 => {}
        autre => echecs.push(format!("AC9 (a) lister : {:?}", autre.map(|c| c.len()))),
    }
    if let Err(e) = rapports::identite_societe(&admin, &lecture).await {
        echecs.push(format!("AC9 (a) identite_societe : {e:?}"));
    }
    // Les fixtures atteignent bien les branches visées : une ligne de TVA, une
    // ligne de balance âgée.
    match rapports::tva(&admin, &lecture, demande_periode(BORNES, &p), Affichage).await {
        Ok(r) => {
            if r.afficher(&trace).await.rows.is_empty() {
                echecs.push("AC9 (a) tva : aucune ligne — fixture inopérante".into());
            }
        }
        Err(e) => echecs.push(format!("AC9 (a) tva : {e:?}")),
    }
    match rapports::balance_agee(&admin, &lecture, Affichage).await {
        Ok(r) => {
            if r.afficher(&trace).await.rows.is_empty() {
                echecs.push("AC9 (a) balance_agee : aucune ligne — fixture inopérante".into());
            }
        }
        Err(e) => echecs.push(format!("AC9 (a) balance_agee : {e:?}")),
    }
    assert!(echecs.is_empty(), "{}", echecs.join("\n"));
}

/// AC9 (b) — la table des traces au niveau du service : chaque appel reçoit la
/// `Lecture` **en lecture seule**, `afficher`/`exporter` la `Trace` du pool
/// ordinaire ; exactement une trace neuve, égale à la ligne.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn ac9_b_table_des_traces_au_service(pool: MySqlPool) {
    let livres = poser_livres(&pool).await;
    let lecture = lecture_seule(&pool).await;
    let trace = Trace::new(pool.clone());
    let admin = api(&livres, Role::Admin);
    let p = Params::de(&livres);
    let mut echecs = Vec::new();
    for ligne in TABLE_DES_TRACES {
        let repere = repere_audit(&pool).await;
        let rel = match jouer(
            ligne.svc,
            ligne.usage,
            ligne.dem,
            &p,
            &admin,
            &lecture,
            Fin::Tracer(&trace),
        )
        .await
        {
            Ok(rel) => rel,
            Err(e) => {
                echecs.push(format!("service#{} {} : {e:?}", ligne.num, ligne.nom));
                continue;
            }
        };
        if ligne.svc == Svc::BalanceAgee && rel.as_of != rel.as_of_contenu {
            echecs.push(format!(
                "service#{} {} : as_of du rapport {:?} ≠ du contenu {:?}",
                ligne.num, ligne.nom, rel.as_of, rel.as_of_contenu
            ));
        }
        let jour = |d: Option<NaiveDate>| {
            d.map(|d| d.format("%Y-%m-%d").to_string())
                .unwrap_or_default()
        };
        let symboles = Symboles {
            fy: p.fy,
            debut: jour(rel.periode.map(|x| x.0)),
            fin: jour(rel.periode.map(|x| x.1)),
            du: jour(rel.bornes_gl.map(|x| x.0)),
            au: jour(rel.bornes_gl.map(|x| x.1)),
            compte: p.compte,
            pid: p.pid,
            asof: jour(rel.as_of),
        };
        let traces = traces_depuis(&pool, livres.user_id(), repere).await;
        if let Err(e) = verifier_trace(
            "service",
            ligne,
            &traces,
            &[resoudre(ligne.details, &symboles)],
        ) {
            echecs.push(e);
        }
    }
    assert!(echecs.is_empty(), "AC9 (b) :\n{}", echecs.join("\n"));
}

/// AC9 (c) — `exporter` avec un rendu qui échoue → `Err`, et **aucune** trace.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn ac9_c_rendu_en_echec_aucune_trace(pool: MySqlPool) {
    let livres = poser_livres(&pool).await;
    let lecture = lecture_seule(&pool).await;
    let trace = Trace::new(pool.clone());
    let admin = api(&livres, Role::Admin);
    let p = Params::de(&livres);
    let mut echecs = Vec::new();
    for svc in SERVICES {
        let repere = repere_audit(&pool).await;
        let r = jouer(
            svc,
            usage_export(svc),
            dem_defaut(svc),
            &p,
            &admin,
            &lecture,
            Fin::RenduEnEchec(&trace),
        )
        .await;
        if !matches!(r, Err(AppError::Internal(_))) {
            echecs.push(format!("AC9 (c) {svc:?} : {}", resume(&r)));
        }
        let traces = traces_depuis(&pool, livres.user_id(), repere).await;
        if !traces.is_empty() {
            echecs.push(format!("AC9 (c) {svc:?} : trace écrite {traces:?}"));
        }
    }
    assert!(echecs.is_empty(), "{}", echecs.join("\n"));
}

/// AC9 (d) — un service qui échoue (période hors exercice) n'écrit aucune
/// trace.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn ac9_d_service_en_echec_aucune_trace(pool: MySqlPool) {
    let livres = poser_livres(&pool).await;
    let lecture = lecture_seule(&pool).await;
    let admin = api(&livres, Role::Admin);
    let repere = repere_audit(&pool).await;
    let demande = DemandePeriode {
        fiscal_year_id: livres.fy(),
        period_start: Some(date("2019-01-01")),
        period_end: None,
    };
    let r = rapports::bilan(&admin, &lecture, demande, Affichage).await;
    assert!(
        r.is_err(),
        "période hors exercice : le service doit échouer"
    );
    let traces = traces_depuis(&pool, livres.user_id(), repere).await;
    assert!(traces.is_empty(), "aucune trace attendue : {traces:?}");
}

// ============================================================
// Les deux régimes du grand livre (revue de code P1, E1)
// ============================================================

/// Les identifiants des lignes de la section du compte `compte`, et son
/// `line_count` (avant pagination).
fn lignes_du_compte(gl: &GeneralLedger, compte: i64) -> (Vec<i64>, i64) {
    let section = gl
        .sections
        .iter()
        .find(|s| s.account_id == compte)
        .expect("la section du compte demandé");
    (
        section.lines.iter().map(|l| l.line_id).collect(),
        section.line_count,
    )
}

/// Les deux régimes de `rapports::grand_livre` : l'affichage lit `offset` et
/// borne `limit` (défaut et plafond `MAX_LEDGER_LIMIT`) ; l'export rend le livre
/// **entier**, quels que soient `offset` et `limit`.
///
/// ⚠️ Le générateur borne **par compte** (`fetch_lines` : `LIMIT ? OFFSET ?`
/// sur les lignes d'un seul compte, `limit` écrêté à `[1, MAX_LEDGER_LIMIT]`) :
/// la fixture pose donc `MAX_LEDGER_LIMIT + 1` lignes sur **un** compte — une
/// écriture synthétique de `MAX_LEDGER_LIMIT + 2` lignes. C'est le seul moyen de
/// distinguer « aucune borne » du plafond.
///
/// Mutations que ce test tue (jouées à la remédiation de la P1) : l'export
/// tronqué à `MAX_LEDGER_LIMIT` ; l'export qui lit `offset` ; l'affichage sans
/// plafond par défaut ; l'affichage qui ne lit pas `offset`.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn grand_livre_les_deux_regimes(pool: MySqlPool) {
    let seeded = seed_accounting_company(&pool).await.unwrap();
    let compte = seeded.accounts["1100"];
    let plafond = usize::try_from(MAX_LEDGER_LIMIT).unwrap();
    let total = plafond + 1;
    let attendu = i64::try_from(total).unwrap();

    let mut lines: Vec<NewJournalEntryLine> = (0..total)
        .map(|_| NewJournalEntryLine {
            account_id: compte,
            debit: dec!(1),
            credit: Decimal::ZERO,
            project_id: None,
        })
        .collect();
    lines.push(NewJournalEntryLine {
        account_id: seeded.accounts["2000"],
        debit: Decimal::ZERO,
        credit: Decimal::from(attendu),
        project_id: None,
    });
    journal_entries::create(
        &pool,
        seeded.fiscal_year_id,
        seeded.admin_user_id,
        NewJournalEntry {
            company_id: seeded.company_id,
            entry_date: date("2026-06-01"),
            journal: Journal::Achats,
            description: "Écriture synthétique à nombreuses lignes".into(),
            project_id: None,
            lines,
        },
    )
    .await
    .unwrap();

    let lecture = lecture_seule(&pool).await;
    let trace = Trace::new(pool.clone());
    let admin = api_societe(seeded.company_id, seeded.admin_user_id, Role::Admin);
    let demande = |offset: Option<i64>, limit: Option<i64>| DemandeGrandLivre {
        from: date(BORNE_DEBUT),
        to: date(BORNE_FIN),
        account_ids: Some(compte.to_string()),
        include_zero: false,
        offset,
        limit,
    };

    // Export sans paramètre : le livre entier, au-delà du plafond d'écran.
    let mut vu = None;
    rapports::grand_livre(
        &admin,
        &lecture,
        demande(None, None),
        Export(FormatExport::Pdf),
    )
    .await
    .unwrap()
    .exporter(&trace, |gl| {
        vu = Some(lignes_du_compte(gl, compte));
        Ok(Vec::new())
    })
    .await
    .unwrap();
    let (entier, n) = vu.take().unwrap();
    assert_eq!(n, attendu, "line_count de la fixture");
    assert_eq!(
        entier.len(),
        total,
        "export : le livre entier ({total} lignes), jamais tronqué au plafond {plafond}"
    );

    // Export avec `offset` et `limit` : ignorés, le même livre entier.
    rapports::grand_livre(
        &admin,
        &lecture,
        demande(Some(1), Some(1)),
        Export(FormatExport::Csv),
    )
    .await
    .unwrap()
    .exporter(&trace, |gl| {
        vu = Some(lignes_du_compte(gl, compte));
        Ok(Vec::new())
    })
    .await
    .unwrap();
    let (export_pagine, _) = vu.take().unwrap();
    assert_eq!(
        export_pagine, entier,
        "export : `offset` et `limit` sont ignorés — le livre entier, depuis la première ligne"
    );

    // Affichage sans paramètre : le plafond par défaut, depuis la première ligne.
    let gl = rapports::grand_livre(&admin, &lecture, demande(None, None), Affichage)
        .await
        .unwrap()
        .afficher(&trace)
        .await;
    let (ecran, n) = lignes_du_compte(&gl, compte);
    assert_eq!(n, attendu, "line_count avant pagination");
    assert_eq!(
        ecran,
        entier[..plafond],
        "affichage : le défaut est le plafond ({plafond} lignes), depuis la première"
    );

    // Affichage paginé : `offset` et `limit` lus.
    let gl = rapports::grand_livre(&admin, &lecture, demande(Some(1), Some(1)), Affichage)
        .await
        .unwrap()
        .afficher(&trace)
        .await;
    let (page, _) = lignes_du_compte(&gl, compte);
    assert_eq!(
        page,
        entier[1..2],
        "affichage : `offset=1&limit=1` rend la deuxième ligne seule"
    );
}

// ============================================================
// AC10 — garde lexicale de la frontière
// ============================================================

/// Exceptions écrites de la garde (v-1) : (fichier relatif à `src/`, ligne
/// réduite de ses blancs de tête et de fin). **Vide en 32-3** ; un faux rouge
/// s'y inscrit, jamais par un motif affaibli.
const EXCEPTIONS_V1: &[(&str, &str)] = &[];

/// Les deux fabriques de poignées, au câblage de `lib.rs` (v-2).
const FABRIQUES_V2: &[(&str, &str)] = &[
    (
        "lib.rs",
        "services::poignees::Lecture::new(self.pool.clone())",
    ),
    (
        "lib.rs",
        "services::poignees::Trace::new(self.pool.clone())",
    ),
];

/// Les deux lignes de code de `services/` qui nomment `axum`, `AppState` ou
/// `CurrentUser` (AC1, (vi)).
const RESIDUS_VI: &[(&str, &str)] = &[
    (
        "services/acces.rs",
        "use crate::middleware::auth::CurrentUser;",
    ),
    (
        "services/acces.rs",
        "pub fn depuis_api(u: &CurrentUser) -> Self {",
    ),
];

const MOTIF_V1: &str = r"\.\s*pool\s*\(|::\s*pool\b";
const MOTIF_V2: &str = r"\bLecture::new\(|\bTrace::new\(";
const MOTIF_VI: &str = r"axum|AppState|CurrentUser";

fn racine_src() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("src")
}

/// Tous les `.rs` de `dir`, récursivement, triés.
fn fichiers_rs(dir: &Path) -> Vec<PathBuf> {
    let mut v = Vec::new();
    for e in std::fs::read_dir(dir).expect("répertoire lisible") {
        let p = e.unwrap().path();
        if p.is_dir() {
            v.extend(fichiers_rs(&p));
        } else if p.extension().is_some_and(|x| x == "rs") {
            v.push(p);
        }
    }
    v.sort();
    v
}

/// Chemin relatif à `src/`, séparateur `/`.
fn relatif(p: &Path) -> String {
    p.strip_prefix(racine_src())
        .unwrap()
        .to_string_lossy()
        .replace('\\', "/")
}

/// Ligne de commentaire au sens de l'AC1 : son premier caractère non blanc
/// ouvre `//` (`///` et `//!` compris).
fn est_commentaire(ligne: &str) -> bool {
    ligne.trim_start().starts_with("//")
}

/// Les lignes de code d'un texte : (numéro 1-based, ligne).
fn lignes_de_code(texte: &str) -> Vec<(usize, &str)> {
    texte
        .lines()
        .enumerate()
        .filter(|(_, l)| !est_commentaire(l))
        .map(|(i, l)| (i + 1, l))
        .collect()
}

/// Les sources de `src/` : (chemin relatif, texte). Plancher de fichiers lus.
fn sources() -> Vec<(String, String)> {
    let v: Vec<(String, String)> = fichiers_rs(&racine_src())
        .into_iter()
        .map(|p| {
            let t = std::fs::read_to_string(&p).unwrap();
            (relatif(&p), t)
        })
        .collect();
    assert!(
        v.len() >= 80,
        "balayage de src/ : {} fichiers lus, plancher 80 — chemin faux ?",
        v.len()
    );
    v
}

/// Les couples (fichier, ligne réduite) des lignes de code qui satisfont `motif`.
fn couples(sources: &[(String, String)], motif: &regex::Regex) -> Vec<(String, String)> {
    let mut v = Vec::new();
    for (f, t) in sources {
        for (_, l) in lignes_de_code(t) {
            if motif.is_match(l) {
                v.push((f.clone(), l.trim().to_string()));
            }
        }
    }
    v.sort();
    v
}

fn attendus(liste: &[(&str, &str)]) -> Vec<(String, String)> {
    let mut v: Vec<(String, String)> = liste
        .iter()
        .map(|(f, l)| (f.to_string(), l.to_string()))
        .collect();
    v.sort();
    v
}

/// Ôte d'un corps ses commentaires : blocs `/* … */` (sur le corps entier,
/// plusieurs lignes comprises), lignes de commentaire, fin de ligne après `//`.
///
/// ⚠️ **Limite** (revue de code P1 de la 32-4a, B4) : l'aide ne connaît pas
/// les littéraux. Un `//` **dans une chaîne** (`"https://…"`) coupe la ligne
/// comme un commentaire, si bien qu'un `state.pool` écrit après lui sur la même
/// ligne échappe au détecteur ; de même, un `/*` dans une chaîne ouvrirait un
/// bloc, qui avalerait tout jusqu'au `*/` suivant, sur plusieurs lignes. Aucun
/// des quatre fichiers lus par les gardes (ii) et (iii) — `routes/accounts.rs`,
/// `journal_entries.rs`, `letterings.rs`, `contacts.rs` — ne porte aujourd'hui
/// de chaîne contenant `//` ou `/*` (relevé de la revue P1 sur trois fichiers
/// et le seul `//`, étendu à `accounts.rs` et à `/*` en P2) ; la limite est
/// écrite aussi à la doc de `ac12_iii_inventaire_des_sites_non_resolus`.
fn sans_commentaires(corps: &str) -> String {
    let blocs = regex::Regex::new(r"(?s)/\*.*?\*/").unwrap();
    let sans_blocs = blocs.replace_all(corps, " ");
    sans_blocs
        .lines()
        .filter(|l| !est_commentaire(l))
        .map(|l| l.split("//").next().unwrap_or(""))
        .collect::<Vec<_>>()
        .join("\n")
}

/// Les corps des `pub async fn <prefixe>…` d'un texte : de la ligne d'ouverture
/// à la première ligne égale à `}`.
fn corps_des_handlers<'a>(texte: &'a str, prefixe: &str) -> Vec<(String, Vec<&'a str>)> {
    let lignes: Vec<&str> = texte.lines().collect();
    let ouverture = format!("pub async fn {prefixe}");
    let mut v = Vec::new();
    let mut i = 0;
    while i < lignes.len() {
        if lignes[i].starts_with(&ouverture) {
            let nom = lignes[i]
                .trim_start_matches("pub async fn ")
                .split('(')
                .next()
                .unwrap()
                .to_string();
            let fin = (i..lignes.len())
                .find(|&j| lignes[j] == "}")
                .expect("corps de handler fermé");
            v.push((nom, lignes[i..=fin].to_vec()));
            i = fin;
        }
        i += 1;
    }
    v
}

/// Témoins : chaque motif est d'abord trouvé dans un texte témoin — une faute de
/// frappe ne rend pas la garde vide.
#[test]
fn ac10_temoins_des_motifs() {
    let v1 = regex::Regex::new(MOTIF_V1).unwrap();
    for forme in ["lecture.pool()", "Lecture::pool(&l)", ".map(Lecture::pool)"] {
        assert!(v1.is_match(forme), "(v-1) doit voir {forme}");
    }
    assert!(
        !v1.is_match("state.pool"),
        "(v-1) ne doit pas voir le champ"
    );
    let v2 = regex::Regex::new(MOTIF_V2).unwrap();
    assert!(v2.is_match("Lecture::new(p)") && v2.is_match("x::Trace::new(p)"));
    assert!(
        !v2.is_match("DetailsTrace::new(p)"),
        "(v-2) : frontière de mot"
    );
    let vi = regex::Regex::new(MOTIF_VI).unwrap();
    assert!(
        vi.is_match("use axum::Router;") && vi.is_match("&AppState") && vi.is_match("CurrentUser")
    );
    assert!(
        est_commentaire("   /// doc") && est_commentaire("//! m") && !est_commentaire("x // y")
    );
    let temoin = "pub async fn export_x() {\n    // rapport.exporter(&t)\n    /*\n    rapport.exporter(&t)\n    */\n    let a = 1; // r.exporter(\n}\n";
    assert!(
        !sans_commentaires(temoin).contains(".exporter("),
        "(vii) : un commentaire ne compte pas"
    );
    assert!(sans_commentaires("    r.exporter(&t, f)").contains(".exporter("));
    let corps = corps_des_handlers(temoin, "export_");
    assert_eq!(corps.len(), 1);
    assert_eq!(corps[0].0, "export_x");
}

/// AC10 (i) et (ii) — `routes/reports.rs` sans `state.pool` ni `MySqlPool` ;
/// le corps de `list_accounts` et des cinq lectures de la 32-4a
/// ([`HANDLERS_MIGRES`]) sans `state.pool`, par le détecteur aplati
/// ([`MOTIF_POOL`], AC12 de la 32-4a ; témoins :
/// `ac12_temoins_du_detecteur_du_pool`).
#[test]
fn ac10_i_ii_handlers_sans_le_pool() {
    let src = racine_src();
    let reports = std::fs::read_to_string(src.join("routes/reports.rs")).unwrap();
    let fautes: Vec<String> = lignes_de_code(&reports)
        .into_iter()
        .filter(|(_, l)| l.contains("state.pool") || l.contains("MySqlPool"))
        .map(|(n, l)| format!("routes/reports.rs:{n}: {l}"))
        .collect();
    assert!(fautes.is_empty(), "AC10 (i) :\n{}", fautes.join("\n"));

    // (ii) — étendue par la Story 32-4a (AC12) aux cinq lectures du rollout,
    // puis par la 32-4b (AC13) aux six lectures des factures : le corps de
    // chaque handler migré ne lit pas `state.pool`.
    let mut fautes = Vec::new();
    for (fichier, handler) in HANDLERS_MIGRES {
        let texte = code_avant_les_tests(&src.join(fichier));
        let corps = corps_du_handler(&texte, handler)
            .unwrap_or_else(|| panic!("{fichier} : {handler} introuvable"));
        assert!(
            corps.len() > 3,
            "{fichier} : corps de {handler} non vide ({} lignes)",
            corps.len()
        );
        let n = occurrences_du_pool(&corps.join("\n"));
        if n > 0 {
            fautes.push(format!("{fichier} : {handler} lit state.pool ({n} fois)"));
        }
    }
    assert!(fautes.is_empty(), "AC10 (ii) :\n{}", fautes.join("\n"));
}

/// Les handlers dont la lecture passe par un service : `list_accounts` (32-3),
/// les cinq lectures de la 32-4a et les six de la 32-4b (factures clientes et
/// fournisseur).
const HANDLERS_MIGRES: &[(&str, &str)] = &[
    ("routes/accounts.rs", "list_accounts"),
    ("routes/journal_entries.rs", "list_journal_entries"),
    ("routes/journal_entries.rs", "get_journal_entry"),
    ("routes/letterings.rs", "get_open_items"),
    ("routes/contacts.rs", "list_contacts"),
    ("routes/contacts.rs", "get_contact"),
    ("routes/invoices.rs", "list_invoices"),
    ("routes/invoices.rs", "get_invoice"),
    ("routes/invoices.rs", "list_due_dates_handler"),
    ("routes/invoices.rs", "list_invoice_settlements_handler"),
    ("routes/supplier_invoices.rs", "list_supplier_invoices"),
    ("routes/supplier_invoices.rs", "get_supplier_invoice"),
];

/// AC12 (iii) — l'inventaire clos des sites non résolus des fichiers du
/// rollout : (fichier, nombre de `pub async fn`, nombre d'`async fn` toutes
/// visibilités, handlers qui lisent encore `state.pool`, **aides hors corps**
/// qui lisent `state.pool`) — les dix mutants de la 32-4a (C-32-0-4), les deux
/// lectures de lettrage qu'aucun outil ne sert (C-32-4a-1) ; pour la 32-4b,
/// les douze mutants des factures, l'export CSV de l'échéancier (C-32-4b-1) et
/// les deux aides des mutants de `invoices.rs` (AC13 (iii)).
const SITES_NON_RESOLUS: &[(&str, usize, usize, &[&str], &[&str])] = &[
    (
        "routes/journal_entries.rs",
        7,
        7,
        &[
            "create_journal_entry",
            "delete_journal_entry",
            "reverse_journal_entry",
            "reverse_journal_entry_at_next_fiscal_year_start",
            "update_journal_entry",
        ],
        &[],
    ),
    (
        "routes/letterings.rs",
        5,
        5,
        &[
            "create_lettering",
            "delete_lettering",
            "get_lettering",
            "get_lettering_proposals",
        ],
        &[],
    ),
    (
        "routes/contacts.rs",
        5,
        5,
        &["archive_contact", "create_contact", "update_contact"],
        &[],
    ),
    (
        "routes/invoices.rs",
        13,
        15,
        &[
            "cancel_invoice_settlement_handler",
            "create_invoice",
            "delete_invoice",
            "export_due_dates_csv_handler",
            "settle_invoice_handler",
            "unvalidate_invoice_handler",
            "update_invoice",
            "validate_invoice_handler",
            "write_off_invoice_handler",
        ],
        &[
            "ensure_contact_belongs_to_company",
            "with_draft_rounding_preview",
        ],
    ),
    (
        "routes/supplier_invoices.rs",
        7,
        8,
        &[
            "cancel_supplier_invoice",
            "cancel_supplier_invoice_settlement",
            "create_supplier_invoice",
            "pay_supplier_invoice",
        ],
        &[],
    ),
];

/// Le détecteur commun de (ii) et (iii) (Story 32-4a, AC12) : `state.pool` sur
/// un texte **aplati** et sans commentaires — `\s` traverse les fins de ligne,
/// si bien que la forme coupée par rustfmt (`state` / `.pool`) compte aussi
/// (#738 B4, pour cette seule forme).
const MOTIF_POOL: &str = r"\bstate\s*\.\s*pool\b";

/// Nombre d'occurrences de [`MOTIF_POOL`] dans `texte`, commentaires ôtés.
fn occurrences_du_pool(texte: &str) -> usize {
    regex::Regex::new(MOTIF_POOL)
        .unwrap()
        .find_iter(&sans_commentaires(texte))
        .count()
}

/// Le texte d'un fichier de `src/`, tronqué à sa première ligne `#[cfg(test)]`.
fn code_avant_les_tests(chemin: &Path) -> String {
    std::fs::read_to_string(chemin)
        .unwrap_or_else(|e| panic!("{} illisible : {e}", chemin.display()))
        .lines()
        .take_while(|l| l.trim() != "#[cfg(test)]")
        .collect::<Vec<_>>()
        .join("\n")
}

/// Le corps du handler de nom **exact** `nom` (définition de
/// [`corps_des_handlers`]) — le préfixe `get_lettering` rendrait aussi
/// `get_lettering_proposals`.
fn corps_du_handler<'a>(texte: &'a str, nom: &str) -> Option<Vec<&'a str>> {
    corps_des_handlers(texte, nom)
        .into_iter()
        .find(|(n, _)| n == nom)
        .map(|(_, corps)| corps)
}

/// Témoins du détecteur commun (AC12 (ii), #738 A5) : il trouve `state.pool`
/// en une ligne, coupé par rustfmt, et dans le corps réel de `get_lettering` ;
/// il ne le trouve ni dans un commentaire ni dans `lecture.pool()`.
#[test]
fn ac12_temoins_du_detecteur_du_pool() {
    assert_eq!(occurrences_du_pool("let c = state.pool.acquire();"), 1);
    assert_eq!(
        occurrences_du_pool("let mut conn = state\n        .pool\n        .acquire()"),
        1,
        "forme coupée par rustfmt"
    );
    assert_eq!(occurrences_du_pool("    // state.pool"), 0, "commentaire");
    assert_eq!(occurrences_du_pool("lecture.pool()"), 0, "poignée");
    let letterings = code_avant_les_tests(&racine_src().join("routes/letterings.rs"));
    let corps = corps_du_handler(&letterings, "get_lettering").expect("get_lettering");
    assert!(
        occurrences_du_pool(&corps.join("\n")) >= 1,
        "corps réel de get_lettering"
    );
}

/// Les corps des **aides** d'un texte (Story 32-4b, AC13 (iii)) : les fonctions
/// de premier niveau dont la ligne d'ouverture **commence** par `fn ` ou
/// `async fn ` (non `pub`), de la ligne d'ouverture à la première ligne égale à
/// `}` — la définition du corps d'un handler ([`corps_des_handlers`]).
fn corps_des_aides(texte: &str) -> Vec<(String, Vec<&str>)> {
    let lignes: Vec<&str> = texte.lines().collect();
    let mut v = Vec::new();
    let mut i = 0;
    while i < lignes.len() {
        let reste = lignes[i]
            .strip_prefix("async fn ")
            .or_else(|| lignes[i].strip_prefix("fn "));
        if let Some(reste) = reste {
            let nom = reste.split(['(', '<']).next().unwrap().trim().to_string();
            let fin = (i..lignes.len())
                .find(|&j| lignes[j] == "}")
                .expect("corps d'aide fermé");
            v.push((nom, lignes[i..=fin].to_vec()));
            i = fin;
        }
        i += 1;
    }
    v
}

/// AC12 (iii) — dans les fichiers du rollout, l'ensemble des handlers dont
/// le corps lit `state.pool` est exactement [`SITES_NON_RESOLUS`] ; le nombre
/// de `pub async fn` et d'`async fn` toutes visibilités est figé ; et, au
/// niveau du fichier, aucune occurrence hors d'un corps (le compte du fichier
/// égale la somme des corps des handlers **et des aides**, Story 32-4b) — ce
/// qui voit une fonction asynchrone non `pub` ou une aide synchrone posée entre
/// deux handlers ; l'ensemble des aides ([`corps_des_aides`]) qui lisent
/// `state.pool` est figé lui aussi (cinquième champ).
///
/// **Limites** (#738 B4, hors périmètre) : le champ lu sous un autre nom que
/// `state` — déstructuration `State(AppState { pool, .. })`, liaison renommée
/// (`s.pool`, `let st = &state;` puis `st.pool`), `self.pool` dans un
/// `impl AppState`, `(&state).pool`, `state.clone().pool` —, une macro à
/// paramètre ident ; et, dans `accounts.rs`, une aide appelée par
/// `list_accounts` (ce fichier porte légitimement `state.pool` hors de tout
/// corps : il n'est pas compté au niveau du fichier) ; enfin, une chaîne qui
/// contient `//` ou `/*`, que [`sans_commentaires`] prend pour un commentaire —
/// elle masque un `state.pool` de la même ligne pour `//`, ou de toute ligne
/// jusqu'au `*/` suivant pour `/*` (revue de code P1 de la 32-4a, B4).
#[test]
fn ac12_iii_inventaire_des_sites_non_resolus() {
    let async_fn = regex::Regex::new(r"^\s*(pub(\([^)]*\))?\s+)?async\s+fn\b").unwrap();
    let mut fautes = Vec::new();
    for (fichier, nb_pub, nb_async, attendus, aides_attendues) in SITES_NON_RESOLUS {
        let texte = code_avant_les_tests(&racine_src().join(fichier));
        let corps = corps_des_handlers(&texte, "");
        if corps.len() != *nb_pub {
            fautes.push(format!(
                "{fichier} : {} pub async fn, {nb_pub} attendus",
                corps.len()
            ));
        }
        let n_async = lignes_de_code(&texte)
            .iter()
            .filter(|(_, l)| async_fn.is_match(l))
            .count();
        if n_async != *nb_async {
            fautes.push(format!(
                "{fichier} : {n_async} async fn (toutes visibilités), {nb_async} attendues"
            ));
        }
        let mut lisent: Vec<&str> = Vec::new();
        let mut somme = 0;
        for (nom, lignes) in &corps {
            let n = occurrences_du_pool(&lignes.join("\n"));
            somme += n;
            if n > 0 {
                lisent.push(nom.as_str());
            }
        }
        lisent.sort();
        let mut attendus = attendus.to_vec();
        attendus.sort();
        if lisent != attendus {
            fautes.push(format!(
                "{fichier} : handlers sur state.pool {lisent:?}, attendus {attendus:?}"
            ));
        }
        // Story 32-4b — les aides hors corps qui lisent `state.pool`.
        let mut aides: Vec<&str> = Vec::new();
        let mut somme_aides = 0;
        let corps_aides = corps_des_aides(&texte);
        for (nom, lignes) in &corps_aides {
            let n = occurrences_du_pool(&lignes.join("\n"));
            somme_aides += n;
            if n > 0 {
                aides.push(nom.as_str());
            }
        }
        aides.sort();
        let mut aides_attendues = aides_attendues.to_vec();
        aides_attendues.sort();
        if aides != aides_attendues {
            fautes.push(format!(
                "{fichier} : aides sur state.pool {aides:?}, attendues {aides_attendues:?}"
            ));
        }
        let fichier_entier = occurrences_du_pool(&texte);
        if fichier_entier != somme + somme_aides {
            fautes.push(format!(
                "{fichier} : {fichier_entier} occurrences de state.pool dans le fichier, {somme} dans les corps des handlers et {somme_aides} dans ceux des aides — une occurrence hors d'un handler ou d'une aide"
            ));
        }
    }
    assert!(fautes.is_empty(), "AC12 (iii) :\n{}", fautes.join("\n"));
}

/// Les formes permises de `state` dans le corps d'un handler migré (Story
/// 32-4b, AC13 (ix)) : l'extracteur `State(state)`, et les poignées
/// `state.lecture()`, `state.trace()` et le bundle `state.i18n` — blancs
/// tolérés (rustfmt peut couper).
const FORMES_PERMISES_DE_STATE: [&str; 3] = [
    r"State\s*\(\s*state\s*\)",
    r"\bstate\s*\.\s*(lecture|trace)\s*\(\s*\)",
    r"\bstate\s*\.\s*i18n\b",
];

/// Ce que (ix) reproche à un corps de handler migré, aplati et sans
/// commentaires : l'absence de `State(state)`, ou une occurrence de `state` qui
/// n'est aucune des [`FORMES_PERMISES_DE_STATE`] (le compte des `\bstate\b`
/// doit égaler la somme des comptes des formes permises).
fn usages_interdits_de_state(corps: &str) -> Option<String> {
    let texte = sans_commentaires(corps);
    let extracteur = regex::Regex::new(FORMES_PERMISES_DE_STATE[0]).unwrap();
    if !extracteur.is_match(&texte) {
        return Some("pas d'extracteur State(state)".into());
    }
    let permis: usize = FORMES_PERMISES_DE_STATE
        .iter()
        .map(|m| regex::Regex::new(m).unwrap().find_iter(&texte).count())
        .sum();
    let tous = regex::Regex::new(r"\bstate\b")
        .unwrap()
        .find_iter(&texte)
        .count();
    (tous != permis)
        .then(|| format!("{tous} occurrences de state, {permis} sous une forme permise"))
}

/// AC13 (ix) — Story 32-4b (C-32-4b-9) : dans le corps de chacun des
/// [`HANDLERS_MIGRES`], les **seuls** usages de `state` sont `State(state)`,
/// `state.lecture()`, `state.trace()` et `state.i18n`. Ferme, pour les handlers
/// migrés, les formes de #738 B4 que (ii) ne voit pas : une aide appelée avec
/// `&state` (le trou de `get_invoice`, qui passait `&state` à une aide lisant le
/// pool), une liaison renommée (`let st = &state;`), un `state.clone()`, et une
/// déstructuration (`State(AppState { pool, .. })`, faute de `State(state)`).
/// Ne lit que les corps des handlers migrés : une aide appelée **depuis une
/// aide** n'est vue que par (iii).
#[test]
fn ac13_ix_les_seuls_usages_de_state_d_un_handler_migre() {
    // Témoins : chaque forme interdite est vue, chaque forme permise acceptée.
    let ouvre = "pub async fn h(\n    State(state): State<AppState>,\n) -> R {\n";
    for interdite in [
        "    f(&state, x).await?;",
        "    let p = &state.pool;",
        "    let st = &state;",
        "    let s = state.clone();",
    ] {
        assert!(
            usages_interdits_de_state(&format!("{ouvre}{interdite}\n}}")).is_some(),
            "(ix) doit voir {interdite}"
        );
    }
    assert!(
        usages_interdits_de_state(
            "pub async fn h(\n    State(AppState { pool, .. }): State<AppState>,\n) -> R {\n    f(&pool)\n}"
        )
        .is_some(),
        "(ix) doit refuser un corps sans State(state)"
    );
    for permise in [
        "    s(&state.lecture()).await?;",
        "    s(&state . lecture ()).await?;",
        "    let t = state\n        .trace();",
        "    let m = state.i18n.format(&l, k, None);",
        "    // f(&state, x) : un commentaire ne compte pas",
    ] {
        assert_eq!(
            usages_interdits_de_state(&format!("{ouvre}{permise}\n}}")),
            None,
            "(ix) doit accepter {permise}"
        );
    }

    let src = racine_src();
    let mut fautes = Vec::new();
    for (fichier, handler) in HANDLERS_MIGRES {
        let texte = code_avant_les_tests(&src.join(fichier));
        let corps = corps_du_handler(&texte, handler)
            .unwrap_or_else(|| panic!("{fichier} : {handler} introuvable"));
        if let Some(faute) = usages_interdits_de_state(&corps.join("\n")) {
            fautes.push(format!("{fichier} : {handler} — {faute}"));
        }
    }
    assert!(fautes.is_empty(), "AC13 (ix) :\n{}", fautes.join("\n"));
}

/// Les cinq modules de services du rollout (AC12 (viii)) : trois de la 32-4a,
/// deux de la 32-4b.
const SERVICES_DU_ROLLOUT: [&str; 5] = [
    "services/ecritures.rs",
    "services/postes_ouverts.rs",
    "services/carnet.rs",
    "services/factures.rs",
    "services/factures_fournisseur.rs",
];

const MOTIF_VIII: &str = r"\bTrace\b|\btrace\b";

/// AC12 (viii) — les services du rollout ne tracent rien : leurs lignes de code
/// ne nomment ni `Trace` ni `trace`. Témoins : le motif voit `trace: &Trace`,
/// pas `tracing::warn!`.
#[test]
fn ac12_viii_les_services_du_rollout_ne_tracent_pas() {
    let motif = regex::Regex::new(MOTIF_VIII).unwrap();
    assert!(
        motif.is_match("trace: &Trace"),
        "(viii) doit voir trace: &Trace"
    );
    assert!(
        !motif.is_match("tracing::warn!(\"x\")"),
        "(viii) ne doit pas voir tracing"
    );
    let sources: Vec<(String, String)> = sources()
        .into_iter()
        .filter(|(f, _)| SERVICES_DU_ROLLOUT.contains(&f.as_str()))
        .collect();
    assert_eq!(sources.len(), 5, "les cinq services du rollout lus");
    let trouves = couples(&sources, &motif);
    assert!(
        trouves.is_empty(),
        "AC12 (viii) : un service du rollout nomme la trace : {trouves:?}"
    );
}

/// AC10 (v-1) — aucun appel ni chemin de l'accesseur `pool` hors de `services/`.
#[test]
fn ac10_v1_aucun_acces_au_pool_hors_des_services() {
    let motif = regex::Regex::new(MOTIF_V1).unwrap();
    let hors: Vec<(String, String)> = sources()
        .into_iter()
        .filter(|(f, _)| !f.starts_with("services/"))
        .collect();
    let trouves = couples(&hors, &motif);
    assert_eq!(
        trouves,
        attendus(EXCEPTIONS_V1),
        "AC10 (v-1) : accès au pool d'une poignée hors de services/"
    );
}

/// AC10 (v-2) — `Lecture::new(`/`Trace::new(` aux deux seules lignes du câblage
/// de `lib.rs`.
#[test]
fn ac10_v2_les_poignees_ne_se_fabriquent_qu_au_cablage() {
    let motif = regex::Regex::new(MOTIF_V2).unwrap();
    assert_eq!(
        couples(&sources(), &motif),
        attendus(FABRIQUES_V2),
        "AC10 (v-2) : fabrique de poignée hors du câblage de lib.rs"
    );
}

/// AC10 (vi) — la règle de l'AC1 : dans `services/`, exactement deux lignes de
/// code nomment `axum`, `AppState` ou `CurrentUser`.
#[test]
fn ac10_vi_services_sans_axum_ni_app_state() {
    let motif = regex::Regex::new(MOTIF_VI).unwrap();
    let services: Vec<(String, String)> = sources()
        .into_iter()
        .filter(|(f, _)| f.starts_with("services/"))
        .collect();
    assert_eq!(
        services.len(),
        10,
        "services/ : dix fichiers attendus (cinq de la 32-3, trois de la 32-4a, deux de la 32-4b), {:?}",
        services.iter().map(|(f, _)| f).collect::<Vec<_>>()
    );
    assert_eq!(
        couples(&services, &motif),
        attendus(RESIDUS_VI),
        "AC10 (vi) : type d'Axum ou AppState dans le code de services/"
    );
}

/// AC10 (vii) — le bon usage à chaque handler : neuf `export_…` qui appellent
/// `.exporter(` et jamais `.afficher(`, neuf `get_…` l'inverse.
#[test]
fn ac10_vii_le_bon_usage_a_chaque_handler() {
    let reports = std::fs::read_to_string(racine_src().join("routes/reports.rs")).unwrap();
    let mut fautes = Vec::new();
    for (prefixe, present, absent) in [
        ("export_", ".exporter(", ".afficher("),
        ("get_", ".afficher(", ".exporter("),
    ] {
        let corps = corps_des_handlers(&reports, prefixe);
        assert_eq!(corps.len(), 9, "neuf handlers {prefixe}… attendus");
        for (nom, lignes) in corps {
            let texte = lignes.join("\n");
            if !sans_commentaires(&texte).contains(present) {
                fautes.push(format!("{nom} : n'appelle pas {present}"));
            }
            if lignes
                .iter()
                .any(|l| !est_commentaire(l) && l.contains(absent))
            {
                fautes.push(format!("{nom} : appelle {absent}"));
            }
        }
    }
    assert!(fautes.is_empty(), "AC10 (vii) :\n{}", fautes.join("\n"));
}
