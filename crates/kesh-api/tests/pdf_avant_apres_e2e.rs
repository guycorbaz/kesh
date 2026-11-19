//! Story 49-1b (#1242) — le PDF d'une facture et d'un avoir faits de lignes chiffrées se
//! rend, **par la base et le HTTP**, à l'identique de la 0.19.0 (AC2, D4, D7).
//!
//! Guy a déjà émis de vraies factures, remises sur papier. Une facture validée jamais
//! téléchargée se rend au premier téléchargement avec le code en fonction ; un regel après
//! un fichier perdu aussi ; un avoir n'est jamais figé. Ces trois chemins doivent rendre ce
//! que rendait la 0.19.0 pour une pièce de lignes chiffrées.
//!
//! Les références ont été capturées **avant** tout changement du rendu, dans un worktree
//! propre et détaché au commit de capture (fiche 49-1b, étape 0), par
//! `KESH_REFERENCES=ecrire cargo test -p kesh-api --test pdf_avant_apres_e2e -- --ignored capturer`,
//! puis relues par `KESH_REFERENCES=comparer`. Elles sont dans
//! `tests/fixtures/avant-0.20.0/`, avec le `sources.sha256` des fichiers de rendu compilés.
//!
//! ⛔ Un seul montage ([`derouler`]) sert à la capture et à la comparaison : les pièces,
//! leur ordre de création (identifiants, numéros, références QRR) et leurs dates ne
//! changent pas après la capture.

use std::net::SocketAddr;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use chrono::{NaiveDate, TimeDelta};
use kesh_api::config::Config;
use kesh_api::{AppState, build_router};
use kesh_db::entities::LineType;
use kesh_db::entities::bank_account::NewBankAccount;
use kesh_db::entities::contact::{ContactType, NewContact, Salutation};
use kesh_db::entities::credit_note::NewCreditNote;
use kesh_db::entities::invoice::{NewInvoice, NewInvoiceLine};
use kesh_db::repositories::{bank_accounts, contacts, credit_notes, invoices};
use kesh_db::test_fixtures::{designate_rounding_account, seed_accounting_company};
use kesh_qrbill::empreinte::{
    ModeReferences, ecrire_ou_comparer, ecrire_sans_ecraser, empreinte, mode_references,
    sources_sha256,
};
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use serde_json::json;
use sqlx::MySqlPool;

const TEST_JWT_SECRET: &[u8] = b"test-secret-32-bytes-minimum-test-secret-padding";
const TEST_ADMIN_PASSWORD: &str = "admin123";

/// Les fichiers de rendu lus par le chemin HTTP, dont le `sources.sha256` garde
/// l'empreinte (AC1 (4)) ; chemins relatifs à la racine du dépôt.
const SOURCES_DU_RENDU: &[&str] = &[
    "crates/kesh-api/src/routes/invoice_pdf_service.rs",
    "crates/kesh-api/src/routes/credit_notes.rs",
    "crates/kesh-api/src/routes/issued_invoice_pdf.rs",
    "crates/kesh-api/src/routes/invoice_pdf.rs",
    "crates/kesh-qrbill/src/pdf.rs",
    "crates/kesh-qrbill/src/types.rs",
    "crates/kesh-qrbill/src/largeurs.rs",
    "crates/kesh-qrbill/src/texte.rs",
    "crates/kesh-qrbill/src/generator.rs",
    "crates/kesh-qrbill/src/validation.rs",
    "crates/kesh-qrbill/src/parser.rs",
];

fn racine_du_depot() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("racine du dépôt")
}

fn dossier_des_references() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/avant-0.20.0")
}

struct TestApp {
    base_url: String,
    client: reqwest::Client,
    documents: PathBuf,
    token: String,
}

impl TestApp {
    fn url(&self, path: &str) -> String {
        format!("{}{}", self.base_url, path)
    }

    async fn pdf(&self, chemin: &str) -> Vec<u8> {
        let resp = self
            .client
            .get(self.url(chemin))
            .header("Authorization", format!("Bearer {}", self.token))
            .send()
            .await
            .unwrap();
        assert_eq!(resp.status(), 200, "GET {chemin}");
        resp.bytes().await.unwrap().to_vec()
    }
}

/// Le serveur, avec son propre dossier des documents (le gel y écrit), et un jeton admin.
async fn spawn_app(pool: MySqlPool) -> TestApp {
    let mut config = Config::from_fields_for_test(
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
    );
    let documents =
        std::env::temp_dir().join(format!("kesh-pdf-avant-apres-{}", uuid::Uuid::new_v4()));
    std::fs::create_dir_all(&documents).unwrap();
    config.documents_dir = documents.to_string_lossy().to_string();
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
    kesh_api::errors::init_error_i18n(i18n.clone(), config.locale);
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
    let client = reqwest::Client::new();
    let base_url = format!("http://{addr}");
    let resp = client
        .post(format!("{base_url}/api/v1/auth/login"))
        .json(&json!({ "username": "admin", "password": TEST_ADMIN_PASSWORD }))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200, "login");
    let body: serde_json::Value = resp.json().await.unwrap();
    let token = body["accessToken"].as_str().unwrap().to_string();
    TestApp {
        base_url,
        client,
        documents,
        token,
    }
}

struct Seeded {
    company_id: i64,
    admin_id: i64,
    contact_id: i64,
}

/// Société comptable, compte d'arrondi désigné, banque principale à QR-IBAN, contact à
/// adresse structurée dans la langue de l'installation.
async fn seed(pool: &MySqlPool) -> Seeded {
    let seeded = seed_accounting_company(pool).await.expect("seed");
    designate_rounding_account(pool, seeded.company_id)
        .await
        .expect("compte d'arrondi");
    bank_accounts::upsert_primary(
        pool,
        NewBankAccount {
            company_id: seeded.company_id,
            bank_name: "UBS".into(),
            iban: "CH9300762011623852957".into(),
            qr_iban: Some("CH4431999123000889012".into()),
            is_primary: true,
        },
    )
    .await
    .unwrap();
    let contact = contacts::create(
        pool,
        seeded.admin_user_id,
        NewContact {
            company_id: seeded.company_id,
            contact_type: ContactType::Personne,
            name: "Pia Rutschmann".into(),
            first_name: Some("Pia".into()),
            last_name: Some("Rutschmann".into()),
            is_client: true,
            is_supplier: false,
            address: None,
            address_street: Some("Marktgasse".into()),
            address_building: Some("28".into()),
            address_postal_code: Some("9400".into()),
            address_city: Some("Rorschach".into()),
            address_country: Some("CH".into()),
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
    .unwrap();
    Seeded {
        company_id: seeded.company_id,
        admin_id: seeded.admin_user_id,
        contact_id: contact.id,
    }
}

/// Une ligne chiffrée : `(description, quantité, prix unitaire, taux)`.
type Ligne = (&'static str, Decimal, Decimal, Decimal);

/// Crée au dépôt une facture de lignes chiffrées, puis la valide ; rend son id.
async fn facture_validee(pool: &MySqlPool, s: &Seeded, lignes: &[Ligne]) -> i64 {
    let lines = lignes
        .iter()
        .map(
            |(description, quantity, unit_price, vat_rate)| NewInvoiceLine {
                revenue_account_id: None,
                line_type: LineType::Priced,
                description: (*description).into(),
                quantity: *quantity,
                unit_price: *unit_price,
                vat_rate: *vat_rate,
            },
        )
        .collect();
    let (invoice, _) = invoices::create(
        pool,
        s.admin_id,
        NewInvoice {
            company_id: s.company_id,
            contact_id: s.contact_id,
            date: NaiveDate::from_ymd_opt(2026, 4, 14).unwrap(),
            due_date: Some(NaiveDate::from_ymd_opt(2026, 5, 14).unwrap()),
            payment_terms: Some("30 jours net".into()),
            lines,
            project_id: None,
        },
    )
    .await
    .unwrap();
    invoices::validate_invoice(pool, s.company_id, invoice.id, s.admin_id)
        .await
        .expect("validation");
    invoice.id
}

/// (a) une ligne, un taux.
const PIECE_A: &[Ligne] = &[("Loyer du mois d'avril", dec!(1), dec!(1200.00), dec!(7.70))];
/// (b) multi-taux (7.70 et 2.60) ; (b') a le même contenu.
const PIECE_B: &[Ligne] = &[
    ("Conseil", dec!(2), dec!(150.00), dec!(7.70)),
    ("Ouvrages de référence", dec!(3), dec!(40.00), dec!(2.60)),
];
/// (c) TTC 36.03 : arrondi à 5 centimes non nul (+0.02).
const PIECE_C: &[Ligne] = &[("Frais de dossier", dec!(1), dec!(33.33), dec!(8.10))];

/// Le montage unique de la capture (T0-bis) et de la comparaison (T3) : crée les quatre
/// pièces de l'AC2, les rend par le HTTP (facture jamais téléchargée, avoir), écrit ou
/// compare leurs empreintes selon `mode`, puis rend de nouveau (a) et (b) par un **regel**
/// après suppression du fichier archivé (D7 c) et les compare à la même référence. Rend
/// les écarts, par pièce.
async fn derouler(pool: MySqlPool, mode: ModeReferences) -> Vec<String> {
    let s = seed(&pool).await;
    let a = facture_validee(&pool, &s, PIECE_A).await;
    let b = facture_validee(&pool, &s, PIECE_B).await;
    let c = facture_validee(&pool, &s, PIECE_C).await;
    let b_prime = facture_validee(&pool, &s, PIECE_B).await;
    let avoir = credit_notes::create_credit_note(
        &pool,
        NewCreditNote {
            company_id: s.company_id,
            invoice_id: b_prime,
            date: NaiveDate::from_ymd_opt(2026, 4, 20).unwrap(),
        },
        s.admin_id,
    )
    .await
    .expect("avoir de (b')")
    .credit_note
    .id;
    let arrondi = invoices::find_by_id(&pool, s.company_id, c)
        .await
        .unwrap()
        .unwrap()
        .rounding_amount;
    assert!(!arrondi.is_zero(), "(c) doit porter un arrondi non nul");

    let app = spawn_app(pool.clone()).await;
    let mut ecarts = Vec::new();
    let mut comparer = |nom: &str, pdf: &[u8], mode: ModeReferences| {
        let chemin = dossier_des_references().join(format!("{nom}.empreinte"));
        for e in ecrire_ou_comparer(mode, &chemin, &empreinte(pdf)) {
            ecarts.push(format!("{nom} : {e}"));
        }
    };
    let pieces = [
        ("a-facture-un-taux", format!("/api/v1/invoices/{a}/pdf")),
        ("b-facture-multi-taux", format!("/api/v1/invoices/{b}/pdf")),
        ("c-facture-arrondi", format!("/api/v1/invoices/{c}/pdf")),
        (
            "d-avoir-multi-taux",
            format!("/api/v1/credit-notes/{avoir}/pdf"),
        ),
    ];
    for (nom, chemin) in &pieces {
        comparer(nom, &app.pdf(chemin).await, mode);
    }

    // Regel de (a) et (b) : le fichier archivé perdu, la facture se rend de nouveau.
    for (nom, id) in [("a-facture-un-taux", a), ("b-facture-multi-taux", b)] {
        let inv = invoices::find_by_id(&pool, s.company_id, id)
            .await
            .unwrap()
            .unwrap();
        let fichier = app
            .documents
            .join(inv.pdf_storage_path.as_deref().expect("facture figée"));
        std::fs::remove_file(&fichier).expect("suppression du fichier archivé");
        let resp = app
            .client
            .post(app.url(&format!("/api/v1/invoices/{id}/pdf/refreeze")))
            .header("Authorization", format!("Bearer {}", app.token))
            .send()
            .await
            .unwrap();
        assert_eq!(resp.status(), 200, "regel de {nom}");
        let pdf = app.pdf(&format!("/api/v1/invoices/{id}/pdf")).await;
        // Le regel se compare toujours à la référence, écrite plus haut en mode `ecrire`.
        comparer(nom, &pdf, ModeReferences::Comparer);
    }
    ecarts
}

/// **T0-bis** — capture (`KESH_REFERENCES=ecrire`) ou relecture (`comparer`, par défaut)
/// des références de l'AC2. Ne se lance qu'à la main, dans le worktree de capture.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
#[ignore = "capture des références de la 0.19.0 (étape 0 de la 49-1b) : KESH_REFERENCES=ecrire|comparer"]
async fn capturer_references_http_avant_0_20_0(pool: MySqlPool) {
    let mode = mode_references();
    let ecarts = derouler(pool, mode).await;
    if mode == ModeReferences::Ecrire {
        ecrire_sans_ecraser(
            &dossier_des_references().join("sources.sha256"),
            &sources_sha256(&racine_du_depot(), SOURCES_DU_RENDU),
        );
    }
    assert!(
        ecarts.is_empty(),
        "écarts à la référence :\n{}",
        ecarts.join("\n")
    );
}

/// **T3 (AC2)** — les quatre pièces chiffrées (a) à (d), créées au dépôt et rendues par le
/// HTTP (facture jamais téléchargée, avoir), puis le regel de (a) et (b), se rendent comme
/// en 0.19.0 : même montage que la capture, références relues.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn quatre_pieces_chiffrees_se_rendent_comme_en_0_19_0(pool: MySqlPool) {
    let ecarts = derouler(pool, ModeReferences::Comparer).await;
    assert!(
        ecarts.is_empty(),
        "le PDF d'une pièce chiffrée a changé depuis la 0.19.0 :\n{}",
        ecarts.join("\n")
    );
}
