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

use std::net::SocketAddr;
use std::path::Path;
use std::sync::Arc;

use chrono::{NaiveDate, TimeDelta, Utc};
use jsonwebtoken::{Algorithm, EncodingKey, Header};
use kesh_api::auth::jwt::Claims;
use kesh_api::config::Config;
use kesh_api::{AppState, build_router};
use kesh_db::entities::journal_entry::Journal;
use kesh_db::entities::{NewJournalEntry, NewJournalEntryLine};
use kesh_db::repositories::journal_entries;
use kesh_db::test_fixtures::{SeededCompany, seed_accounting_company};
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use serde_json::{Value, json};
use sqlx::MySqlPool;

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
        nom: "journaux_export_csv",
        svc: Svc::Journaux,
        usage: Usg::Csv,
        dem: BORNES,
        action: "report.exported",
        details: r#"{"report_type": "journals", "format": "csv", "fiscal_year_id": "$fy", "period_start": "$debut", "period_end": "$fin", "journal_filter": null}"#,
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

/// AC9 (b′) — pour chacune des dix-huit lignes, la requête de sa route, puis
/// exactement une trace neuve égale à la ligne. Seul volet qui voie un handler
/// appeler le mauvais service.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn table_des_traces_par_la_route(pool: MySqlPool) {
    let livres = poser_livres(&pool).await;
    let app = spawn_app(pool.clone()).await;
    let jwt = forge_jwt(livres.user_id(), "Admin", livres.company_id());
    assert_eq!(
        TABLE_DES_TRACES.len(),
        18,
        "dix-huit couples (service, usage)"
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
