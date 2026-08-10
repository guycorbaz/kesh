//! Tests E2E pour GET /api/v1/companies/current (Story 2.4).

mod common;

use std::sync::Arc;

use chrono::TimeDelta;
use common::create_test_company;
use kesh_api::auth::bootstrap::ensure_admin_user;
use kesh_api::config::Config;
use kesh_api::{AppState, build_router};
use serde_json::json;
use sqlx::MySqlPool;
use std::net::SocketAddr;

const TEST_JWT_SECRET: &[u8] = b"test-secret-32-bytes-minimum-test-secret-padding";
const TEST_ADMIN_PASSWORD: &str = "e2e-test-admin-password";

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
    let body: serde_json::Value = resp.json().await.unwrap();
    body["accessToken"].as_str().unwrap().to_string()
}

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn companies_current_returns_company(pool: MySqlPool) {
    let app = spawn_app(pool.clone()).await;
    create_test_company(&pool).await;
    ensure_admin_user(&pool, &test_config()).await.unwrap();
    let token = login(&app).await;

    let resp = app
        .client
        .get(app.url("/api/v1/companies/current"))
        .header("Authorization", format!("Bearer {token}"))
        .send()
        .await
        .unwrap();

    assert_eq!(resp.status(), 200);
    let body: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(body["company"]["name"], "Test Company");
    assert!(body["bankAccounts"].is_array());
    assert_eq!(body["bankAccounts"].as_array().unwrap().len(), 0);
}

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn companies_current_requires_auth(pool: MySqlPool) {
    let app = spawn_app(pool).await;

    let resp = app
        .client
        .get(app.url("/api/v1/companies/current"))
        .send()
        .await
        .unwrap();

    assert_eq!(resp.status(), 401);
}

/// **AC8 — l'ALLER-RETOUR complet** : écrire par la route, relire par `GET`.
///
/// ⚠️ **C'est le seul test qui voit un DTO oublié.** `CompanyJson` est un
/// miroir **écrit à la main** — struct Rust, son `impl From<Company>`, et
/// l'interface TypeScript — qu'aucun compilateur ne vérifie contre l'entité.
/// Sans ce test, oublier le `From` laisse la valeur **stockée en base**,
/// **rendue sur le PDF**, et **invisible dans l'écran de réglages** : tous les
/// gates passent au vert, et le défaut ne se voit qu'à l'usage.
///
/// L'assertion porte donc sur le **corps HTTP relu**, pas sur la réponse du
/// `PUT` ni sur la base — c'est la seule chose que le frontend consomme.
#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn contact_details_survive_the_round_trip_to_the_settings_screen(pool: MySqlPool) {
    let app = spawn_app(pool.clone()).await;
    create_test_company(&pool).await;
    ensure_admin_user(&pool, &test_config()).await.unwrap();
    let token = login(&app).await;

    let current: serde_json::Value = app
        .client
        .get(app.url("/api/v1/companies/current"))
        .header("Authorization", format!("Bearer {token}"))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    let version = current["company"]["version"].as_i64().unwrap();
    // ⚠️ Les DEUX champs sont vérifiés nuls au départ, pas seulement le
    // téléphone : l'assertion finale porte sur les deux, donc si
    // `create_test_company` venait à poser un `website`, ce volet-là cesserait
    // silencieusement de mesurer l'écriture — le risque exact que ce montage
    // dit vouloir écarter. *(Passe 6 de revue.)*
    assert!(
        current["company"]["phone"].is_null(),
        "montage : la société ne doit porter aucune coordonnée au départ, \
         sinon le test ne mesure pas l'écriture"
    );
    assert!(
        current["company"]["website"].is_null(),
        "montage : idem pour le site web — sans quoi l'assertion finale sur \
         `website` passerait sans rien mesurer"
    );

    let resp = app
        .client
        .put(app.url("/api/v1/companies/current/contact-details"))
        .header("Authorization", format!("Bearer {token}"))
        .json(&json!({
            "phone": "+41 21 123 45 67",
            "website": "https://demo.ch",
            "version": version,
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200, "l'écriture doit réussir");

    // Le GET est ce que consomme l'écran de réglages — c'est LUI qui fait foi.
    let relu: serde_json::Value = app
        .client
        .get(app.url("/api/v1/companies/current"))
        .header("Authorization", format!("Bearer {token}"))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();

    assert_eq!(
        relu["company"]["phone"], "+41 21 123 45 67",
        "le téléphone doit revenir dans le GET — s'il est absent, le DTO \
         `CompanyJson` ou son `impl From<Company>` a été oublié, et l'écran de \
         réglages affichera « — » pour toujours"
    );
    assert_eq!(
        relu["company"]["website"], "https://demo.ch",
        "le site web doit revenir dans le GET — même piège que ci-dessus"
    );
}

/// Le champ vidé **efface** la valeur : la ligne disparaît du PDF (D2).
#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn empty_contact_details_clear_the_stored_values(pool: MySqlPool) {
    let app = spawn_app(pool.clone()).await;
    create_test_company(&pool).await;
    ensure_admin_user(&pool, &test_config()).await.unwrap();
    let token = login(&app).await;

    let v0 = app
        .client
        .get(app.url("/api/v1/companies/current"))
        .header("Authorization", format!("Bearer {token}"))
        .send()
        .await
        .unwrap()
        .json::<serde_json::Value>()
        .await
        .unwrap()["company"]["version"]
        .as_i64()
        .unwrap();

    let posed: serde_json::Value = app
        .client
        .put(app.url("/api/v1/companies/current/contact-details"))
        .header("Authorization", format!("Bearer {token}"))
        .json(&json!({ "phone": "+41 21 123 45 67", "website": null, "version": v0 }))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(posed["phone"], "+41 21 123 45 67", "montage : valeur posée");

    let cleared: serde_json::Value = app
        .client
        .put(app.url("/api/v1/companies/current/contact-details"))
        .header("Authorization", format!("Bearer {token}"))
        .json(&json!({
            "phone": "   ",
            "website": null,
            "version": posed["version"].as_i64().unwrap(),
        }))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();

    assert!(
        cleared["phone"].is_null(),
        "un champ vide (ou blanc) doit EFFACER la valeur, pas la conserver \
         ni stocker une chaîne vide : la ligne doit disparaître du PDF"
    );
}

/// **Le full-replace, dans les DEUX directions** — revue de code, passe 2.
///
/// ⚠️ `companies::update` remplace **toutes** les colonnes. Les deux routes
/// `PUT /companies/current/*` ne modifient qu'un sous-ensemble de champs et
/// **reportent le reste à l'identique** ; si l'un de ces reports disparaissait,
/// la valeur non visée serait **effacée en silence**, en `200`, avec bump de
/// `version` et une entrée d'audit d'apparence normale.
///
/// Le Dev Agent Record nommait ce piège sans le tester. Ce test le ferme dans
/// les deux sens : éditer les coordonnées ne doit pas perdre l'e-mail, et
/// éditer l'e-mail ne doit pas perdre les coordonnées.
#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn each_route_preserves_the_fields_it_does_not_touch(pool: MySqlPool) {
    let app = spawn_app(pool.clone()).await;
    create_test_company(&pool).await;
    ensure_admin_user(&pool, &test_config()).await.unwrap();
    let token = login(&app).await;

    let read = |token: String| {
        let app = &app;
        async move {
            app.client
                .get(app.url("/api/v1/companies/current"))
                .header("Authorization", format!("Bearer {token}"))
                .send()
                .await
                .unwrap()
                .json::<serde_json::Value>()
                .await
                .unwrap()["company"]
                .clone()
        }
    };

    // 1. Poser un e-mail par sa route dédiée.
    let v0 = read(token.clone()).await["version"].as_i64().unwrap();
    let after_email: serde_json::Value = app
        .client
        .put(app.url("/api/v1/companies/current/email"))
        .header("Authorization", format!("Bearer {token}"))
        .json(&json!({ "email": "reply@demo.ch", "version": v0 }))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(after_email["email"], "reply@demo.ch", "montage");

    // 2. Éditer les COORDONNÉES — l'e-mail ne doit pas bouger.
    let after_contact: serde_json::Value = app
        .client
        .put(app.url("/api/v1/companies/current/contact-details"))
        .header("Authorization", format!("Bearer {token}"))
        .json(&json!({
            "phone": "+41 21 123 45 67",
            "website": "https://demo.ch",
            "version": after_email["version"].as_i64().unwrap(),
        }))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(
        after_contact["email"], "reply@demo.ch",
        "éditer les coordonnées ne doit PAS effacer l'e-mail — si cette \
         assertion tombe, le report `email: company.email.clone()` a disparu \
         de `update_company_contact_details` et l'adresse de réponse des \
         factures est perdue en silence"
    );

    // 3. Rééditer l'E-MAIL — les coordonnées ne doivent pas bouger.
    let after_email2: serde_json::Value = app
        .client
        .put(app.url("/api/v1/companies/current/email"))
        .header("Authorization", format!("Bearer {token}"))
        .json(&json!({
            "email": "autre@demo.ch",
            "version": after_contact["version"].as_i64().unwrap(),
        }))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(
        after_email2["phone"], "+41 21 123 45 67",
        "éditer l'e-mail ne doit PAS effacer le téléphone — c'est le piège que \
         le Dev Agent Record nommait sans le tester"
    );
    assert_eq!(
        after_email2["website"], "https://demo.ch",
        "ni le site web, pour la même raison"
    );
}

/// La borne de longueur est **refusée par le backend**, pas seulement par le
/// `maxlength` du navigateur (revue de code, passe 2).
#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn overlong_contact_details_are_rejected_by_the_api(pool: MySqlPool) {
    let app = spawn_app(pool.clone()).await;
    create_test_company(&pool).await;
    ensure_admin_user(&pool, &test_config()).await.unwrap();
    let token = login(&app).await;

    // ⚠️ La borne se teste des DEUX CÔTÉS. Ne vérifier que le rejet au-delà
    // laisse passer un `>` changé en `>=` : des valeurs parfaitement légales
    // seraient refusées avec un message « trop long » faux, et la suite
    // resterait verte. *(Relevé aux passes 4 et 5 de revue.)*
    for (champ, valeur) in [("phone", "0".repeat(50)), ("website", "x".repeat(255))] {
        // ⚠️ Relire la version À CHAQUE tour : une acceptation la bumpe, et
        // réutiliser la précédente rend un 409 qu'on prendrait pour un rejet de
        // longueur. (Première version de ce test : `v0` figé, 409 au 2e tour.)
        let v = app
            .client
            .get(app.url("/api/v1/companies/current"))
            .header("Authorization", format!("Bearer {token}"))
            .send()
            .await
            .unwrap()
            .json::<serde_json::Value>()
            .await
            .unwrap()["company"]["version"]
            .as_i64()
            .unwrap();
        let resp = app
            .client
            .put(app.url("/api/v1/companies/current/contact-details"))
            .header("Authorization", format!("Bearer {token}"))
            .json(&json!({ champ: valeur, "version": v }))
            .send()
            .await
            .unwrap();
        assert_eq!(
            resp.status(),
            200,
            "un `{champ}` d'exactement la longueur maximale DOIT être accepté — \
             la colonne l'accepte, le refuser serait un faux positif"
        );

        // ⚠️ Le 200 ne suffit PAS. Une régression de `normalize_contact_field`
        // rendant `Ok(None)` au lieu d'`Err` pour une valeur pile à la borne
        // laisserait ce test vert : le statut est là, et la valeur est
        // silencieusement perdue. C'est la valeur relue qui fait foi.
        // *(Passe 6 de revue.)*
        let corps: serde_json::Value = resp.json().await.unwrap();
        assert_eq!(
            corps[champ], valeur,
            "un `{champ}` d'exactement la longueur maximale doit être STOCKÉ, \
             pas seulement accepté — un 200 qui perd la valeur est un défaut \
             muet"
        );
    }

    // Relire la version : les acceptations ci-dessus l'ont bumpée.
    let v0 = app
        .client
        .get(app.url("/api/v1/companies/current"))
        .header("Authorization", format!("Bearer {token}"))
        .send()
        .await
        .unwrap()
        .json::<serde_json::Value>()
        .await
        .unwrap()["company"]["version"]
        .as_i64()
        .unwrap();

    for (champ, valeur) in [("phone", "0".repeat(51)), ("website", "x".repeat(256))] {
        let resp = app
            .client
            .put(app.url("/api/v1/companies/current/contact-details"))
            .header("Authorization", format!("Bearer {token}"))
            .json(&json!({ champ: valeur, "version": v0 }))
            .send()
            .await
            .unwrap();
        assert_eq!(
            resp.status(),
            400,
            "un `{champ}` d'un caractère au-delà de la borne DOIT être refusé \
             par l'API — le `maxlength` du navigateur ne protège pas un appel \
             direct, et MariaDB tronquerait en silence"
        );
    }
}

/// **Un champ OMIS du payload efface la valeur** — comportement épinglé, pas
/// défendu (revue de code, passe 3).
///
/// ⚠️ `#[serde(default)]` rend l'**absence** d'une clé indistinguable de `null`,
/// et `companies::update` est un full-replace : envoyer `{"phone": …, "version": …}`
/// **sans** `website` efface le site web, en `200`, avec bump de `version`.
///
/// Le frontend envoie toujours les deux champs, ce qui borne le risque aux
/// clients API — mais le doc-comment du DTO ne disait que « `null`/vide =
/// effacer », jamais « absent aussi ». Ce test rend le comportement visible :
/// s'il rougit, la sémantique a changé et le CHANGELOG doit suivre.
#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn an_omitted_field_clears_it_just_like_null(pool: MySqlPool) {
    let app = spawn_app(pool.clone()).await;
    create_test_company(&pool).await;
    ensure_admin_user(&pool, &test_config()).await.unwrap();
    let token = login(&app).await;

    let v0 = app
        .client
        .get(app.url("/api/v1/companies/current"))
        .header("Authorization", format!("Bearer {token}"))
        .send()
        .await
        .unwrap()
        .json::<serde_json::Value>()
        .await
        .unwrap()["company"]["version"]
        .as_i64()
        .unwrap();

    // Poser les DEUX champs.
    let posed: serde_json::Value = app
        .client
        .put(app.url("/api/v1/companies/current/contact-details"))
        .header("Authorization", format!("Bearer {token}"))
        .json(&json!({ "phone": "+41 21 123 45 67", "website": "https://demo.ch", "version": v0 }))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(posed["website"], "https://demo.ch", "montage");

    // N'envoyer QUE `phone` — `website` est ABSENT, pas à `null`.
    let after: serde_json::Value = app
        .client
        .put(app.url("/api/v1/companies/current/contact-details"))
        .header("Authorization", format!("Bearer {token}"))
        .json(
            &json!({ "phone": "+41 21 999 88 77", "version": posed["version"].as_i64().unwrap() }),
        )
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();

    assert!(
        after["website"].is_null(),
        "une clé ABSENTE efface la valeur, exactement comme `null` — c'est le \
         full-replace hérité du patron e-mail. Si cette assertion rougit, la \
         sémantique a changé : mettre à jour le doc-comment du DTO et le CHANGELOG."
    );
}
