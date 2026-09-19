//! Story 25-1c-a (#378) — tests HTTP des trois routes du journal d'audit.
//!
//! ⛔ **L'application est montée en `de-CH` et la société est en
//! `accounting_language` FRANÇAISE.** Ce n'est pas un détail de montage : c'est
//! le seul dispositif qui fasse rougir une implémentation qui lirait la langue
//! **comptable** au lieu de celle de l'interface. Les deux langues concordant,
//! le test passerait au vert sur un code faux.
//!
//! ⚠️ `init_error_i18n` écrit un état **global au processus** : tout test qui
//! monte une application prend `verrou_i18n()` d'abord.

use std::net::SocketAddr;
use std::sync::Arc;

use chrono::{TimeDelta, Utc};
use jsonwebtoken::{Algorithm, EncodingKey, Header};
use kesh_api::audit_labels::{ACTIONS, ENTITY_TYPES};
use kesh_api::auth::jwt::Claims;
use kesh_api::auth::password::hash_password;
use kesh_api::config::Config;
use kesh_api::{AppState, build_router};
use kesh_db::entities::address::StructuredAddress;
use kesh_db::entities::audit_log::NewAuditLogEntry;
use kesh_db::entities::{Language, NewCompany, NewUser, OrgType, Role};
use kesh_db::repositories::{audit_log, companies, users};
use serde_json::{Value, json};
use sqlx::MySqlPool;

const TEST_JWT_SECRET: &[u8] = b"test-secret-32-bytes-minimum-test-secret-padding";

static VERROU_I18N: std::sync::OnceLock<tokio::sync::Mutex<()>> = std::sync::OnceLock::new();

async fn verrou_i18n() -> tokio::sync::MutexGuard<'static, ()> {
    VERROU_I18N
        .get_or_init(|| tokio::sync::Mutex::new(()))
        .lock()
        .await
}

struct TestApp {
    base_url: String,
    client: reqwest::Client,
}

impl TestApp {
    fn url(&self, chemin: &str) -> String {
        format!("{}{}", self.base_url, chemin)
    }

    /// `GET` avec le jeton porté en `Authorization: Bearer`.
    async fn get(&self, chemin: &str, jeton: &str) -> reqwest::Response {
        self.client
            .get(self.url(chemin))
            .header("Authorization", format!("Bearer {jeton}"))
            .send()
            .await
            .expect("requête")
    }

    /// `GET` sans aucun en-tête d'autorisation.
    async fn get_anonyme(&self, chemin: &str) -> reqwest::Response {
        self.client
            .get(self.url(chemin))
            .send()
            .await
            .expect("requête")
    }
}

/// Les trois routes, pour les boucles de refus — aucune n'a le droit d'être
/// oubliée : l'export et le vocabulaire sont des chemins d'extraction au même
/// titre que la liste.
const LES_TROIS_ROUTES: [&str; 3] = [
    "/api/v1/audit-log",
    "/api/v1/audit-log/vocabulary",
    "/api/v1/audit-log/export.csv",
];

fn config_de_test() -> Config {
    Config::from_fields_for_test(
        "mysql://test:test@localhost:3306/test".to_string(),
        "admin".to_string(),
        "admin123".to_string(),
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

/// Monte l'application **en allemand**.
async fn spawn_app_de(pool: MySqlPool) -> TestApp {
    let config = config_de_test().with_locale(kesh_i18n::Locale::DeCh);
    let rate_limiter = kesh_api::middleware::rate_limit::RateLimiter::new(&config);
    let i18n = Arc::new(
        kesh_i18n::I18nBundle::load(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .parent()
                .unwrap()
                .join("kesh-i18n/locales")
                .as_path(),
        )
        .expect("charger les catalogues"),
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

fn forge_jwt(user_id: i64, role: &str, company_id: i64) -> String {
    let maintenant = Utc::now().timestamp();
    let claims = Claims {
        sub: user_id.to_string(),
        role: role.to_string(),
        company_id,
        iat: maintenant,
        exp: maintenant + 3600,
    };
    jsonwebtoken::encode(
        &Header::new(Algorithm::HS256),
        &claims,
        &EncodingKey::from_secret(TEST_JWT_SECRET),
    )
    .unwrap()
}

fn role_str(role: Role) -> &'static str {
    match role {
        Role::Admin => "Admin",
        Role::Comptable => "Comptable",
        Role::Consultation => "Consultation",
    }
}

struct Ctx {
    company_id: i64,
    user_id: i64,
    jeton: String,
}

/// Société + utilisateur d'un rôle donné.
///
/// ⚠️ `accounting_language` est **française** à dessein, l'interface étant
/// allemande : c'est l'écart qui rend le test capable de détecter une lecture de
/// la mauvaise langue.
async fn seed_role(pool: &MySqlPool, label: &str, role: Role) -> Ctx {
    let company_id = companies::create(
        pool,
        NewCompany {
            name: format!("CI {label}"),
            first_name: None,
            last_name: None,
            address_structured: StructuredAddress {
                street: "Rue Test".into(),
                building: "1".into(),
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
    .unwrap()
    .id;

    let user_id = users::create(
        pool,
        NewUser {
            username: format!("{label}_user"),
            password_hash: hash_password("password123").unwrap(),
            role,
            active: true,
            company_id,
            email: None,
        },
    )
    .await
    .unwrap()
    .id;

    Ctx {
        jeton: forge_jwt(user_id, role_str(role), company_id),
        company_id,
        user_id,
    }
}

/// Écrit une entrée par le chemin de production.
async fn ecrire_entree(pool: &MySqlPool, user_id: i64, action: &str, type_entite: &str, id: i64) {
    let mut tx = pool.begin().await.unwrap();
    audit_log::insert_in_tx(
        &mut tx,
        NewAuditLogEntry::user(user_id, action, type_entite, id, None),
    )
    .await
    .expect("écriture d'une entrée d'audit");
    tx.commit().await.unwrap();
}

/// Pose `actor_label` — il n'est pas choisi par l'appelant en production (c'est
/// un sous-SELECT), mais le test doit pouvoir y placer une formule.
async fn poser_actor_label(pool: &MySqlPool, id_entree: i64, etiquette: &str) {
    sqlx::query("UPDATE audit_log SET actor_label = ? WHERE id = ?")
        .bind(etiquette)
        .bind(id_entree)
        .execute(pool)
        .await
        .unwrap();
}

async fn dernier_id(pool: &MySqlPool) -> i64 {
    sqlx::query_scalar::<_, i64>("SELECT MAX(id) FROM audit_log")
        .fetch_one(pool)
        .await
        .unwrap()
}

async fn compter_entrees(pool: &MySqlPool) -> i64 {
    sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM audit_log")
        .fetch_one(pool)
        .await
        .unwrap()
}

/// Une clé API `read` créée par un **Admin** : le cas le plus favorable au
/// refus qu'on veut prouver.
async fn cle_api_read(app: &TestApp, ctx: &Ctx) -> String {
    let resp = app
        .client
        .post(app.url("/api/v1/settings/api-keys"))
        .header("Authorization", format!("Bearer {}", ctx.jeton))
        .json(&json!({ "name": "clé de test", "scope": "read" }))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 201, "création de la clé de test");
    let corps: Value = resp.json().await.unwrap();
    corps["key"].as_str().unwrap().to_string()
}

async fn code_erreur(resp: reqwest::Response) -> String {
    let statut = resp.status();
    let corps: Value = resp
        .json()
        .await
        .unwrap_or_else(|e| panic!("corps JSON attendu (statut {statut}) : {e}"));
    corps["error"]["code"]
        .as_str()
        .unwrap_or_else(|| panic!("error.code attendu, reçu {corps}"))
        .to_string()
}

// ===========================================================================
// Refus — sur les TROIS routes
// ===========================================================================

#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn sans_jeton_les_trois_routes_repondent_401(pool: MySqlPool) {
    let _verrou = verrou_i18n().await;
    let app = spawn_app_de(pool).await;

    for route in LES_TROIS_ROUTES {
        let resp = app.get_anonyme(route).await;
        assert_eq!(resp.status(), 401, "{route} sans jeton");
    }
}

#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn le_role_consultation_est_refuse_sur_les_trois_routes(pool: MySqlPool) {
    let _verrou = verrou_i18n().await;
    let ctx = seed_role(&pool, "consult", Role::Consultation).await;
    let app = spawn_app_de(pool).await;

    for route in LES_TROIS_ROUTES {
        let resp = app.get(route, &ctx.jeton).await;
        assert_eq!(
            resp.status(),
            403,
            "{route} doit refuser le rôle Consultation"
        );
    }
}

#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn une_cle_api_est_refusee_sur_les_trois_routes(pool: MySqlPool) {
    let _verrou = verrou_i18n().await;
    let ctx = seed_role(&pool, "admin_pat", Role::Admin).await;
    let app = spawn_app_de(pool).await;
    let cle = cle_api_read(&app, &ctx).await;

    for route in LES_TROIS_ROUTES {
        let resp = app.get(route, &cle).await;
        assert_eq!(resp.status(), 403, "{route} doit refuser une clé API");
        assert_eq!(
            code_erreur(resp).await,
            "API_KEY_MANAGEMENT_FORBIDDEN",
            "{route} — le code du refus"
        );
    }
}

#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn comptable_et_admin_sont_acceptes_sur_les_trois_routes(pool: MySqlPool) {
    let _verrou = verrou_i18n().await;
    let comptable = seed_role(&pool, "comptable", Role::Comptable).await;
    let admin = seed_role(&pool, "admin", Role::Admin).await;
    let app = spawn_app_de(pool).await;

    for (nom, ctx) in [("Comptable", &comptable), ("Admin", &admin)] {
        for route in LES_TROIS_ROUTES {
            let resp = app.get(route, &ctx.jeton).await;
            assert_eq!(resp.status(), 200, "{nom} sur {route}");
        }
    }
}

// ===========================================================================
// Validation des paramètres
// ===========================================================================

#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn les_parametres_invalides_sont_refuses_en_400(pool: MySqlPool) {
    let _verrou = verrou_i18n().await;
    let ctx = seed_role(&pool, "valid", Role::Comptable).await;
    let app = spawn_app_de(pool).await;

    // ⛔ Le `+` est écrit `%2B`. Écrit en clair, `form_urlencoded` le décode en
    // ESPACE : la valeur échoue alors au FORMAT, et le test ne sonde plus la
    // PLAGE — il passerait au vert sans rien prouver de la borne.
    let cas: [&str; 10] = [
        "?dateFrom=pas-une-date",
        "?dateFrom=2026-02-01&dateTo=2026-01-01",
        "?entityId=42",
        "?entityType=contact&entityId=0",
        "?action=aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
        "?dateTo=%2B262142-12-31",
        "?dateFrom=%2B262142-12-31",
        "?dateFrom=-0001-01-01",
        "?dateFrom=0999-12-31",
        "?dateTo=0999-12-31",
    ];

    for suffixe in cas {
        let resp = app
            .get(&format!("/api/v1/audit-log{suffixe}"), &ctx.jeton)
            .await;
        assert_eq!(resp.status(), 400, "attendu 400 pour {suffixe}");
        assert_eq!(
            code_erreur(resp).await,
            "VALIDATION_ERROR",
            "code du refus pour {suffixe}"
        );
    }
}

#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn une_date_hors_plage_est_refusee_pour_la_plage_et_non_le_format(pool: MySqlPool) {
    let _verrou = verrou_i18n().await;
    let ctx = seed_role(&pool, "plage", Role::Comptable).await;
    let app = spawn_app_de(pool).await;

    // `0999-12-31` est une date PARFAITEMENT valide : si le refus parlait du
    // format, il porterait sur autre chose que ce qu'on veut interdire.
    let resp = app
        .get("/api/v1/audit-log?dateFrom=0999-12-31", &ctx.jeton)
        .await;
    assert_eq!(resp.status(), 400);
    let corps: Value = resp.json().await.unwrap();
    let message = corps["error"]["message"].as_str().unwrap_or_default();
    assert!(
        message.contains("1000-01-01") && message.contains("9999-12-31"),
        "le message doit nommer la PLAGE, reçu : {message}"
    );
}

#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn la_borne_haute_9999_est_acceptee_et_rend_bien_l_entree(pool: MySqlPool) {
    let _verrou = verrou_i18n().await;
    let ctx = seed_role(&pool, "borne", Role::Comptable).await;
    ecrire_entree(&pool, ctx.user_id, "contact.created", "contact", 1).await;
    let id = dernier_id(&pool).await;
    sqlx::query("UPDATE audit_log SET created_at = '9999-12-31 23:59:59.999' WHERE id = ?")
        .bind(id)
        .execute(&pool)
        .await
        .unwrap();
    let app = spawn_app_de(pool).await;

    let resp = app
        .get("/api/v1/audit-log?dateTo=9999-12-31", &ctx.jeton)
        .await;
    assert_eq!(resp.status(), 200);
    let corps: Value = resp.json().await.unwrap();
    // ⛔ Un 200 à liste vide masquerait exactement le défaut que la borne
    // inclusive supprime.
    assert!(
        !corps["items"].as_array().unwrap().is_empty(),
        "l'entrée du 9999-12-31 doit être rendue, reçu : {corps}"
    );
}

// ===========================================================================
// Forme de la réponse, et langue
// ===========================================================================

#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn la_reponse_porte_le_code_de_base_et_jamais_la_societe(pool: MySqlPool) {
    let _verrou = verrou_i18n().await;
    let ctx = seed_role(&pool, "forme", Role::Comptable).await;
    ecrire_entree(&pool, ctx.user_id, "invoice.validated", "invoice", 7).await;
    let app = spawn_app_de(pool).await;

    let corps: Value = app
        .get("/api/v1/audit-log", &ctx.jeton)
        .await
        .json()
        .await
        .unwrap();
    let entree = &corps["items"][0];

    // ⛔ `ActorType` sérialiserait « User » ; la valeur de la base est « user »,
    // et c'est la seule stable.
    assert_eq!(entree["actorType"], "user");
    assert!(
        entree["createdAt"].as_str().unwrap().ends_with('Z'),
        "createdAt doit être un UTC explicite, reçu {}",
        entree["createdAt"]
    );
    assert!(
        entree.get("companyId").is_none(),
        "le filtre étant strict, companyId n'apprendrait rien"
    );
}

#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn les_libelles_suivent_la_langue_de_l_interface_et_non_la_comptable(pool: MySqlPool) {
    let _verrou = verrou_i18n().await;
    let ctx = seed_role(&pool, "langue", Role::Comptable).await;
    ecrire_entree(&pool, ctx.user_id, "invoice.validated", "invoice", 7).await;
    let app = spawn_app_de(pool).await;

    let corps: Value = app
        .get("/api/v1/audit-log", &ctx.jeton)
        .await
        .json()
        .await
        .unwrap();
    let entree = &corps["items"][0];

    // La société est en FRANÇAIS, l'interface en ALLEMAND : une implémentation
    // qui lirait `accounting_language` rendrait « Facture validée » et rougirait.
    assert_eq!(entree["actionLabel"], "Rechnung validiert");
    assert_eq!(entree["entityTypeLabel"], "Rechnung");
}

#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn un_code_inconnu_sort_en_code_dans_la_liste_et_dans_le_csv(pool: MySqlPool) {
    let _verrou = verrou_i18n().await;
    let ctx = seed_role(&pool, "inconnu", Role::Comptable).await;
    ecrire_entree(&pool, ctx.user_id, "zz.unknown_code", "zz_unknown", 3).await;
    let app = spawn_app_de(pool).await;

    let corps: Value = app
        .get("/api/v1/audit-log", &ctx.jeton)
        .await
        .json()
        .await
        .unwrap();
    let entree = &corps["items"][0];
    assert_eq!(entree["actionLabel"], "zz.unknown_code");
    assert_eq!(entree["entityTypeLabel"], "zz_unknown");

    let csv = app
        .get("/api/v1/audit-log/export.csv", &ctx.jeton)
        .await
        .text()
        .await
        .unwrap();
    assert!(
        csv.contains("zz.unknown_code") && !csv.contains("audit-log-action-zz"),
        "le CSV doit porter le code, jamais la clé"
    );
}

// ===========================================================================
// Vocabulaire
// ===========================================================================

#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn le_vocabulaire_rend_les_deux_listes_completes_et_traduites(pool: MySqlPool) {
    let _verrou = verrou_i18n().await;
    let ctx = seed_role(&pool, "vocab", Role::Comptable).await;
    let app = spawn_app_de(pool).await;

    let corps: Value = app
        .get("/api/v1/audit-log/vocabulary", &ctx.jeton)
        .await
        .json()
        .await
        .unwrap();

    let types = corps["entityTypes"].as_array().unwrap();
    let actions = corps["actions"].as_array().unwrap();
    // ⛔ Les longueurs sont LUES du module, jamais recopiées : un nombre écrit
    // ici deviendrait faux en silence au premier code ajouté.
    assert_eq!(types.len(), ENTITY_TYPES.len());
    assert_eq!(actions.len(), ACTIONS.len());

    // L'ordre est celui des constantes, par code.
    assert_eq!(types[0]["code"], ENTITY_TYPES[0]);
    assert_eq!(actions[0]["code"], ACTIONS[0]);

    let facture = types
        .iter()
        .find(|t| t["code"] == "invoice")
        .expect("le type invoice");
    assert_eq!(facture["label"], "Rechnung", "libellé en allemand");
}

// ===========================================================================
// Export CSV
// ===========================================================================

#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn l_export_porte_son_bom_ses_entetes_traduits_et_son_nom_de_fichier(pool: MySqlPool) {
    let _verrou = verrou_i18n().await;
    let ctx = seed_role(&pool, "export", Role::Comptable).await;
    ecrire_entree(&pool, ctx.user_id, "invoice.validated", "invoice", 7).await;
    let app = spawn_app_de(pool).await;

    let resp = app.get("/api/v1/audit-log/export.csv", &ctx.jeton).await;
    assert_eq!(resp.status(), 200);
    assert_eq!(
        resp.headers().get("content-type").unwrap(),
        "text/csv; charset=utf-8"
    );
    let disposition = resp
        .headers()
        .get("content-disposition")
        .unwrap()
        .to_str()
        .unwrap()
        .to_string();
    assert!(
        disposition.contains("filename*=UTF-8'de-CH'"),
        "le tag de langue doit être celui de l'interface, reçu : {disposition}"
    );
    assert!(
        disposition.contains("kesh-journal-audit-"),
        "nom de fichier attendu, reçu : {disposition}"
    );

    let octets = resp.bytes().await.unwrap();
    assert_eq!(&octets[..3], &[0xEF, 0xBB, 0xBF], "BOM UTF-8 attendu");

    let texte = String::from_utf8(octets[3..].to_vec()).unwrap();
    let premiere = texte.lines().next().unwrap();
    assert_eq!(
        premiere.split(';').count(),
        10,
        "dix colonnes attendues, reçu : {premiere}"
    );
    assert!(
        premiere.starts_with("Nr.;"),
        "en-têtes en allemand attendus, reçu : {premiere}"
    );
    assert!(
        texte.contains("Rechnung validiert"),
        "la cellule action doit être traduite"
    );
    assert!(
        texte.contains("Benutzer"),
        "la cellule type d'auteur doit être traduite"
    );
}

#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn l_export_neutralise_une_formule_dans_le_nom_de_l_auteur(pool: MySqlPool) {
    let _verrou = verrou_i18n().await;
    let ctx = seed_role(&pool, "inject", Role::Comptable).await;
    ecrire_entree(&pool, ctx.user_id, "contact.created", "contact", 1).await;
    let id = dernier_id(&pool).await;
    poser_actor_label(&pool, id, "=HYPERLINK(\"x\")").await;
    let app = spawn_app_de(pool).await;

    let octets = app
        .get("/api/v1/audit-log/export.csv", &ctx.jeton)
        .await
        .bytes()
        .await
        .unwrap();

    // ⛔ Relire par un VRAI lecteur CSV et comparer la CELLULE exacte. Le writer
    // double les guillemets (`"'=HYPERLINK(""x"")"`) : une assertion par
    // sous-chaîne rougirait à tort, et une assertion négative resterait verte
    // sous la mutation qu'on veut détecter.
    let mut lecteur = csv::ReaderBuilder::new()
        .delimiter(b';')
        .from_reader(&octets[3..]);
    let ligne = lecteur
        .records()
        .next()
        .expect("une ligne de données")
        .expect("ligne lisible");
    assert_eq!(
        &ligne[2], "'=HYPERLINK(\"x\")",
        "la cellule doit être préfixée d'une apostrophe"
    );
}

#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn au_dela_du_plafond_l_export_refuse_en_allemand(pool: MySqlPool) {
    let _verrou = verrou_i18n().await;
    let ctx = seed_role(&pool, "plafond", Role::Comptable).await;

    // 10 001 entrées par UN SEUL INSERT — dix mille appels HTTP coûteraient des
    // minutes pour la même assertion.
    sqlx::query(
        "INSERT INTO audit_log \
         (user_id, actor_label, action, entity_type, entity_id, actor_type, company_id, created_at) \
         SELECT ?, 'CI', 'contact.created', 'contact', 1, 'user', ?, NOW(3) \
         FROM seq_1_to_10001",
    )
    .bind(ctx.user_id)
    .bind(ctx.company_id)
    .execute(&pool)
    .await
    .expect("semis des 10 001 entrées");

    let app = spawn_app_de(pool).await;
    let resp = app.get("/api/v1/audit-log/export.csv", &ctx.jeton).await;
    assert_eq!(resp.status(), 400);
    let corps: Value = resp.json().await.unwrap();
    assert_eq!(corps["error"]["code"], "RESULT_TOO_LARGE");
    let message = corps["error"]["message"].as_str().unwrap_or_default();
    assert!(
        message.starts_with("Zu viele Ergebnisse"),
        "le refus doit être dans la langue de l'interface, reçu : {message}"
    );
}

// ===========================================================================
// La lecture ne s'écrit pas elle-même
// ===========================================================================

#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn consulter_le_journal_n_y_ecrit_aucune_entree(pool: MySqlPool) {
    let _verrou = verrou_i18n().await;
    let ctx = seed_role(&pool, "silence", Role::Comptable).await;
    ecrire_entree(&pool, ctx.user_id, "contact.created", "contact", 1).await;
    let app = spawn_app_de(pool.clone()).await;

    let avant = compter_entrees(&pool).await;
    for route in LES_TROIS_ROUTES {
        let resp = app.get(route, &ctx.jeton).await;
        assert_eq!(resp.status(), 200, "{route}");
    }
    let apres = compter_entrees(&pool).await;

    // Chaque page d'écran en écrirait une, et le journal enflerait de sa propre
    // lecture jusqu'à devenir illisible.
    assert_eq!(avant, apres, "la consultation ne doit rien écrire");
}
