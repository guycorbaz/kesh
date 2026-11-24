//! Tests E2E — Story 49-4a (#1364) : **la capacité du PDF se contrôle à la validation**.
//!
//! Une facture dont les lignes ne tiendraient pas sur la page de son PDF est refusée par
//! `POST /api/v1/invoices/{id}/validate` (`400 INVOICE_LINES_DO_NOT_FIT_PDF`, avec les lignes
//! demandées et offertes), **sans rien écrire** — ni statut, ni numéro, ni écriture, ni
//! version, ni compteur de numérotation —, au lieu d'être validée puis refusée au premier
//! téléchargement. Les seuils exacts passent, et leur PDF se rend.
//!
//! Les brouillons se créent **par le dépôt** (`invoices::create`), qui accepte les lignes de
//! commentaire (l'API ne les ouvre qu'avec la 49-1c) ; la validation, elle, passe **par la
//! route**. La pièce (e), validée par le dépôt — sans contrôle —, montre au `GET /pdf` le
//! refus que la validation devance.

mod common;

use std::net::SocketAddr;
use std::sync::Arc;

use chrono::TimeDelta;
use kesh_api::config::Config;
use kesh_api::{AppState, build_router};
use kesh_db::entities::bank_account::NewBankAccount;
use kesh_db::entities::contact::{ContactType, NewContact};
use kesh_db::entities::invoice::{InvoiceUpdate, NewInvoice, NewInvoiceLine};
use kesh_db::repositories::{bank_accounts, contacts, invoices};
use kesh_db::test_fixtures::seed_accounting_company;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use serde_json::json;
use sqlx::MySqlPool;

const TEST_JWT_SECRET: &[u8] = b"test-secret-32-bytes-minimum-test-secret-padding";
const TEST_ADMIN_PASSWORD: &str = "admin123";

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
            std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
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

    TestApp {
        base_url: format!("http://{addr}"),
        client: reqwest::Client::new(),
    }
}

async fn login(app: &TestApp) -> String {
    let resp = app
        .client
        .post(app.url("/api/v1/auth/login"))
        .json(&json!({ "username": "admin", "password": TEST_ADMIN_PASSWORD }))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200, "login");
    let body: serde_json::Value = resp.json().await.unwrap();
    body["accessToken"].as_str().unwrap().to_string()
}

/// Société comptablement complète, un client à l'adresse structurée et un compte principal à
/// QR-IBAN : tout ce qu'il faut pour que le `GET /pdf` d'une pièce validée rende `200`.
/// Rend `(admin_id, company_id, contact_id)`.
async fn seed(pool: &MySqlPool) -> (i64, i64, i64) {
    let seeded = seed_accounting_company(pool)
        .await
        .expect("seed_accounting_company");
    let (admin_id, company_id) = (seeded.admin_user_id, seeded.company_id);
    let contact_id = contacts::create(
        pool,
        admin_id,
        NewContact {
            company_id,
            contact_type: ContactType::Personne,
            name: "Pia Rutschmann".into(),
            first_name: None,
            last_name: None,
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
            default_payment_terms: Some("30 jours net".into()),
            default_payment_terms_days: None,
            language: None,
            salutation: kesh_db::entities::contact::Salutation::Neutre,
        },
    )
    .await
    .unwrap()
    .id;
    bank_accounts::upsert_primary(
        pool,
        NewBankAccount {
            company_id,
            bank_name: "UBS".into(),
            iban: "CH9300762011623852957".into(),
            qr_iban: Some("CH4431999123000889012".into()),
            is_primary: true,
        },
    )
    .await
    .unwrap();
    (admin_id, company_id, contact_id)
}

/// Une ligne chiffrée d'une unité.
fn chiffree(prix: Decimal, taux: Decimal) -> NewInvoiceLine {
    NewInvoiceLine {
        revenue_account_id: None,
        line_type: kesh_db::entities::LineType::Priced,
        description: "Prestation".into(),
        quantity: dec!(1),
        unit_price: prix,
        vat_rate: taux,
    }
}

/// `n` lignes chiffrées de 100.00 sans TVA : total rond aux 5 centimes, **aucun
/// récapitulatif** (place 8).
fn sans_recap(n: usize) -> Vec<NewInvoiceLine> {
    (0..n).map(|_| chiffree(dec!(100.00), dec!(0))).collect()
}

/// Une ligne de commentaire, aux valeurs neutres (`chk_invoice_lines_unpriced_neutral`).
fn commentaire(texte: &str) -> NewInvoiceLine {
    NewInvoiceLine {
        revenue_account_id: None,
        line_type: kesh_db::entities::LineType::Comment,
        description: texte.into(),
        quantity: dec!(0),
        unit_price: dec!(0),
        vat_rate: dec!(0),
    }
}

/// Crée un **brouillon** aux lignes données, par le dépôt.
async fn brouillon(
    pool: &MySqlPool,
    (admin_id, company_id, contact_id): (i64, i64, i64),
    lines: Vec<NewInvoiceLine>,
) -> i64 {
    let (inv, _) = invoices::create(
        pool,
        admin_id,
        NewInvoice {
            company_id,
            contact_id,
            date: common::jour::aujourd_hui_test(),
            due_date: None,
            payment_terms: Some("30 jours net".into()),
            lines,
            project_id: None,
        },
    )
    .await
    .unwrap();
    inv.id
}

async fn valider(app: &TestApp, token: &str, id: i64) -> reqwest::Response {
    app.client
        .post(app.url(&format!("/api/v1/invoices/{id}/validate")))
        .bearer_auth(token)
        .send()
        .await
        .unwrap()
}

async fn pdf(app: &TestApp, token: &str, id: i64) -> reqwest::Response {
    app.client
        .get(app.url(&format!("/api/v1/invoices/{id}/pdf")))
        .bearer_auth(token)
        .send()
        .await
        .unwrap()
}

/// `(statut, numéro, version, écriture)` de la facture.
type Etat = (String, Option<String>, i32, Option<i64>);

async fn etat(pool: &MySqlPool, id: i64) -> Etat {
    sqlx::query_as(
        "SELECT status, invoice_number, version, journal_entry_id FROM invoices WHERE id = ?",
    )
    .bind(id)
    .fetch_one(pool)
    .await
    .unwrap()
}

async fn compteur(pool: &MySqlPool, company_id: i64) -> Option<i64> {
    sqlx::query_scalar("SELECT next_number FROM invoice_number_sequences WHERE company_id = ?")
        .bind(company_id)
        .fetch_optional(pool)
        .await
        .unwrap()
}

async fn ecritures(pool: &MySqlPool) -> i64 {
    sqlx::query_scalar("SELECT COUNT(*) FROM journal_entries")
        .fetch_one(pool)
        .await
        .unwrap()
}

/// La phrase fr-CH du refus (catalogue `error-invoice-lines-do-not-fit-pdf`).
fn phrase(besoin: usize, place: usize) -> String {
    format!(
        "Les lignes de cette facture ne tiennent pas sur la page de son PDF : elles demandent {besoin} lignes, la page en offre {place}. Retirez des lignes, raccourcissez les commentaires, ou répartissez les lignes sur deux factures. La facture reste un brouillon."
    )
}

/// Valide par la route une pièce qui ne tient pas : `400 INVOICE_LINES_DO_NOT_FIT_PDF`, et
/// **rien d'écrit** (état de la facture, écritures, compteur). Rend les deux nombres du
/// message.
async fn refusee(
    app: &TestApp,
    token: &str,
    pool: &MySqlPool,
    company_id: i64,
    id: i64,
    cas: &str,
) -> (usize, usize) {
    let avant = etat(pool, id).await;
    let (ecritures_avant, compteur_avant) =
        (ecritures(pool).await, compteur(pool, company_id).await);
    let resp = valider(app, token, id).await;
    assert_eq!(resp.status(), 400, "{cas} : statut");
    let body: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(
        body["error"]["code"], "INVOICE_LINES_DO_NOT_FIT_PDF",
        "{cas} : {body}"
    );
    let message = body["error"]["message"].as_str().unwrap().to_string();
    assert_eq!(etat(pool, id).await, avant, "{cas} : la facture a bougé");
    assert_eq!(avant.0, "draft", "{cas}");
    assert_eq!(
        ecritures(pool).await,
        ecritures_avant,
        "{cas} : une écriture"
    );
    assert_eq!(
        compteur(pool, company_id).await,
        compteur_avant,
        "{cas} : le compteur de numérotation a bougé"
    );
    let nombres: Vec<usize> = message
        .split(|c: char| !c.is_ascii_digit())
        .filter(|s| !s.is_empty())
        .map(|s| s.parse().unwrap())
        .collect();
    assert_eq!(nombres.len(), 2, "{cas} : {message}");
    let (besoin, place) = (nombres[0], nombres[1]);
    assert_eq!(message, phrase(besoin, place), "{cas}");
    (besoin, place)
}

/// Valide par la route une pièce qui tient : `200`, puis son PDF : `200`.
async fn acceptee(app: &TestApp, token: &str, pool: &MySqlPool, id: i64, cas: &str) {
    let resp = valider(app, token, id).await;
    assert_eq!(resp.status(), 200, "{cas} : validation");
    let (statut, numero, _, ecriture) = etat(pool, id).await;
    assert_eq!(statut, "validated", "{cas}");
    assert!(numero.is_some() && ecriture.is_some(), "{cas}");
    let resp = pdf(app, token, id).await;
    assert_eq!(resp.status(), 200, "{cas} : PDF");
    assert!(resp.bytes().await.unwrap().starts_with(b"%PDF-1."), "{cas}");
}

/// **T1 (AC1)** — la validation refuse les pièces qui ne tiennent pas, aux deux nombres justes,
/// sans rien écrire ; le compteur de numérotation n'a pas bougé (la validation suivante d'une
/// pièce qui tient prend le numéro qui suit) ; la pièce (e), validée sans contrôle, est refusée
/// au rendu ; (c) passe, arrondi désactivé.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn la_validation_refuse_une_facture_qui_ne_tient_pas(pool: MySqlPool) {
    let app = spawn_app(pool.clone()).await;
    let ids = seed(&pool).await;
    let (admin_id, company_id, _) = ids;
    let token = login(&app).await;

    // Une première pièce qui tient : le compteur part.
    let premiere = brouillon(&pool, ids, sans_recap(1)).await;
    acceptee(&app, &token, &pool, premiere, "première").await;
    let numero_premiere = etat(&pool, premiere).await.1.unwrap();
    let compteur_initial = compteur(&pool, company_id).await;

    // (a) Une ligne chiffrée à 8.10 % (récapitulatif « sous-total, TVA », place 6) et un
    // commentaire de mille caractères.
    let mille: String = "Commentaire de mille caracteres. ".repeat(30);
    let mille = format!("{mille}{}", "x".repeat(1000 - mille.chars().count()));
    assert_eq!(mille.chars().count(), 1000);
    let a = brouillon(
        &pool,
        ids,
        vec![chiffree(dec!(100.00), dec!(8.10)), commentaire(&mille)],
    )
    .await;
    let (besoin, place) = refusee(&app, &token, &pool, company_id, a, "(a)").await;
    assert_eq!(place, 6, "(a)");
    assert!(
        besoin > 1 + 6,
        "(a) : le commentaire compte ses lignes imprimées ({besoin})"
    );

    // (b) Huit lignes chiffrées à deux taux (récapitulatif de trois lignes, place 5) ; total
    // 842.80, rond aux 5 centimes : pas d'arrondi.
    let mut lignes_b: Vec<_> = (0..4).map(|_| chiffree(dec!(100.00), dec!(8.10))).collect();
    lignes_b.extend((0..4).map(|_| chiffree(dec!(100.00), dec!(2.60))));
    let b = brouillon(&pool, ids, lignes_b).await;
    assert_eq!(
        refusee(&app, &token, &pool, company_id, b, "(b)").await,
        (8, 5)
    );

    // (c) Sept lignes chiffrées sans TVA à 10.02 (total 70.14, arrondi +0.01 : récapitulatif
    // « sous-total, arrondi », place 6).
    let lignes_c: Vec<_> = (0..7).map(|_| chiffree(dec!(10.02), dec!(0))).collect();
    let c = brouillon(&pool, ids, lignes_c).await;
    assert_eq!(
        refusee(&app, &token, &pool, company_id, c, "(c)").await,
        (7, 6)
    );

    // (d) Neuf lignes chiffrées courtes, sans récapitulatif (place 8).
    let d = brouillon(&pool, ids, sans_recap(9)).await;
    assert_eq!(
        refusee(&app, &token, &pool, company_id, d, "(d)").await,
        (9, 8)
    );

    // (e) Huit lignes chiffrées et deux commentaires « Ω » (rien d'imprimable : zéro ligne
    // imprimée) : le contrôle géométrique accepte (8), le pré-filtre refuse (10 lignes de la
    // pièce, 9 au plus).
    let mut lignes_e = sans_recap(8);
    lignes_e.insert(3, commentaire("Ω"));
    lignes_e.push(commentaire("Ω"));
    let e = brouillon(&pool, ids, lignes_e.clone()).await;
    assert_eq!(
        refusee(&app, &token, &pool, company_id, e, "(e)").await,
        (10, 9)
    );

    // (f) Douze lignes chiffrées : le contrôle géométrique passe d'abord (12 et 8, non 9).
    let f = brouillon(&pool, ids, sans_recap(12)).await;
    assert_eq!(
        refusee(&app, &token, &pool, company_id, f, "(f)").await,
        (12, 8)
    );

    // Le compteur n'a pas bougé : la pièce suivante qui tient prend le numéro qui suit.
    assert_eq!(compteur(&pool, company_id).await, compteur_initial);
    let suivante = brouillon(&pool, ids, sans_recap(2)).await;
    acceptee(&app, &token, &pool, suivante, "suivante").await;
    let numero_suivante = etat(&pool, suivante).await.1.unwrap();
    assert_ne!(numero_suivante, numero_premiere);
    assert_eq!(
        compteur(&pool, company_id).await,
        compteur_initial.map(|n| n + 1),
        "un seul numéro tiré pour la pièce suivante, aucun pour les six refus"
    );

    // La pièce (e), validée par le dépôt (sans contrôle), est refusée au rendu : c'est le
    // refus que la validation devance.
    let e_sans_controle = brouillon(&pool, ids, lignes_e).await;
    invoices::validate_invoice(&pool, company_id, e_sans_controle, admin_id)
        .await
        .expect("validate_invoice par le dépôt");
    let resp = pdf(&app, &token, e_sans_controle).await;
    assert_eq!(resp.status(), 400);
    let body: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(body["error"]["code"], "INVOICE_TOO_MANY_LINES_FOR_PDF");

    // (c) passe arrondi désactivé : plus de récapitulatif, place 8.
    sqlx::query(
        "UPDATE company_invoice_settings SET round_to_5_centimes = FALSE WHERE company_id = ?",
    )
    .bind(company_id)
    .execute(&pool)
    .await
    .unwrap();
    acceptee(&app, &token, &pool, c, "(c) arrondi désactivé").await;
}

/// **T1 (AC2)** — les seuils exacts passent, et leur PDF se rend : (a) huit lignes chiffrées
/// sans récapitulatif ; (b) cinq lignes chiffrées et un récapitulatif de **quatre** lignes
/// (trois taux : la cinquième ligne tombe à l'égalité exacte du plancher, 139 mm) — et une
/// sixième est refusée ; (c) cinq lignes chiffrées et un commentaire de trois lignes imprimées.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn les_seuils_exacts_passent_et_le_pdf_se_rend(pool: MySqlPool) {
    let app = spawn_app(pool.clone()).await;
    let ids = seed(&pool).await;
    let company_id = ids.1;
    let token = login(&app).await;

    let a = brouillon(&pool, ids, sans_recap(8)).await;
    acceptee(&app, &token, &pool, a, "(a)").await;

    // Trois taux, total 525.20 (rond aux 5 centimes) : récapitulatif de quatre lignes.
    let trois_taux = || {
        vec![
            chiffree(dec!(100.00), dec!(8.10)),
            chiffree(dec!(100.00), dec!(2.60)),
            chiffree(dec!(100.00), dec!(3.80)),
            chiffree(dec!(100.00), dec!(8.10)),
            chiffree(dec!(100.00), dec!(2.60)),
        ]
    };
    let b = brouillon(&pool, ids, trois_taux()).await;
    acceptee(&app, &token, &pool, b, "(b)").await;
    let mut six = trois_taux();
    six.push(chiffree(dec!(100.00), dec!(3.80)));
    let b6 = brouillon(&pool, ids, six).await;
    assert_eq!(
        refusee(&app, &token, &pool, company_id, b6, "(b) + 1").await,
        (6, 5)
    );

    let mut lignes_c = sans_recap(5);
    lignes_c.insert(
        2,
        commentaire("Premier paragraphe\nDeuxième paragraphe\nTroisième"),
    );
    let c = brouillon(&pool, ids, lignes_c).await;
    acceptee(&app, &token, &pool, c, "(c)").await;
}

/// **T1 (AC2)** — une facture dévalidée puis allongée au-delà de la place est refusée à la
/// revalidation, sans rien écrire : elle reste un brouillon, garde le numéro qu'elle porte,
/// sans écriture, à la même version ; le compteur ne bouge pas.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn la_revalidation_d_une_facture_devalidee_est_controlee(pool: MySqlPool) {
    let app = spawn_app(pool.clone()).await;
    let ids = seed(&pool).await;
    let (admin_id, company_id, contact_id) = ids;
    let token = login(&app).await;

    let id = brouillon(&pool, ids, sans_recap(8)).await;
    acceptee(&app, &token, &pool, id, "validation").await;
    let (_, numero, version, _) = etat(&pool, id).await;
    let resp = app
        .client
        .post(app.url(&format!("/api/v1/invoices/{id}/unvalidate")))
        .bearer_auth(&token)
        .json(&json!({ "version": version }))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200, "dévalidation");
    let (_, _, version, _) = etat(&pool, id).await;
    invoices::update(
        &pool,
        company_id,
        id,
        version,
        admin_id,
        InvoiceUpdate {
            contact_id,
            date: common::jour::aujourd_hui_test(),
            due_date: None,
            payment_terms: Some("30 jours net".into()),
            project_id: None,
            lines: sans_recap(9),
        },
    )
    .await
    .expect("brouillon allongé");

    assert_eq!(
        refusee(&app, &token, &pool, company_id, id, "revalidation").await,
        (9, 8)
    );
    let (statut, numero_apres, _, ecriture) = etat(&pool, id).await;
    assert_eq!(statut, "draft");
    assert_eq!(
        numero_apres, numero,
        "le numéro gardé par la dévalidation reste"
    );
    assert!(ecriture.is_none());
}
