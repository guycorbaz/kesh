//! Story 37-2 (AC-3) — le client HTTP de la source des cours
//! (`HttpRateSource`), contre un serveur axum **local** en HTTP : aucun appel
//! réseau. Délai, statut non 2xx, redirection non suivie, XML tronqué, corps
//! trop gros avec et sans `Content-Length`, 404 de `xmldaily` (pas encore
//! publié) contre 404 de `xmlavgmonth` (erreur), connexion refusée, chemin,
//! paramètres et `User-Agent`.

use std::net::SocketAddr;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Duration;

use axum::Router;
use axum::body::Body;
use axum::extract::{Query, RawQuery};
use axum::http::{HeaderMap, StatusCode, header};
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use chrono::NaiveDate;
use kesh_api::exchange_rates::source::{HttpRateSource, RateSource, SourceError, monthly_xml};
use kesh_core::exchange_rates::{MAX_BODY_BYTES, RatesError, parse_monthly_rates};

/// Démarre `app` sur un port local libre ; rend l'URL de base.
async fn serveur(app: Router) -> String {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr: SocketAddr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });
    format!("http://{addr}")
}

fn d(y: i32, m: u32, j: u32) -> NaiveDate {
    NaiveDate::from_ymd_opt(y, m, j).unwrap()
}

/// Succès : chemin, paramètres, `User-Agent` sans donnée personnelle, corps
/// rendu tel quel.
#[tokio::test]
async fn succes_chemin_parametres_et_user_agent() {
    let app = Router::new()
        .route(
            "/api/xmlavgmonth",
            get(|RawQuery(q): RawQuery, h: HeaderMap| async move {
                assert_eq!(q.as_deref(), Some("j=2026&m=11"));
                let ua = h[header::USER_AGENT].to_str().unwrap().to_string();
                assert_eq!(ua, concat!("Kesh/", env!("CARGO_PKG_VERSION")));
                assert!(h.get(header::COOKIE).is_none());
                monthly_xml(2026, 11, &[("USD", 1, "0.8800")])
            }),
        )
        .route(
            "/api/xmldaily",
            get(
                |Query(q): Query<std::collections::HashMap<String, String>>| async move {
                    assert_eq!(q.get("d").map(String::as_str), Some("20241231"));
                    "<wechselkurse/>"
                },
            ),
        );
    let source = HttpRateSource::with_base_url(serveur(app).await);
    let corps = source.fetch_month(2026, 11).await.unwrap();
    let m = parse_monthly_rates(corps.as_bytes()).unwrap();
    assert_eq!(m.month, d(2026, 11, 1));
    assert_eq!(
        source.fetch_day(d(2024, 12, 31)).await.unwrap(),
        "<wechselkurse/>"
    );
}

/// Délai total dépassé → `timeout`.
#[tokio::test]
async fn delai_depasse() {
    let app = Router::new().route(
        "/api/xmlavgmonth",
        get(|| async {
            tokio::time::sleep(Duration::from_secs(5)).await;
            "trop tard"
        }),
    );
    let source = HttpRateSource::with_base_url(serveur(app).await)
        .with_timeouts(Duration::from_secs(1), Duration::from_millis(300));
    let e = source.fetch_month(2026, 11).await.unwrap_err();
    assert_eq!(e, SourceError::Timeout);
    assert_eq!(e.code(), "timeout");
}

/// Statut 500 → `http_status:500`.
#[tokio::test]
async fn statut_500() {
    let app = Router::new().route(
        "/api/xmlavgmonth",
        get(|| async { (StatusCode::INTERNAL_SERVER_ERROR, "panne") }),
    );
    let source = HttpRateSource::with_base_url(serveur(app).await);
    let e = source.fetch_month(2026, 11).await.unwrap_err();
    assert_eq!(e, SourceError::HttpStatus(500));
    assert_eq!(e.code(), "http_status:500");
}

/// Redirection : jamais suivie (la cible n'est pas appelée) → `http_status:302`.
#[tokio::test]
async fn redirection_non_suivie() {
    let cible = Arc::new(AtomicUsize::new(0));
    let compteur = cible.clone();
    let app = Router::new()
        .route(
            "/api/xmlavgmonth",
            get(|| async {
                (
                    StatusCode::FOUND,
                    [(header::LOCATION, "/ailleurs")],
                    "redirection",
                )
            }),
        )
        .route(
            "/ailleurs",
            get(move || {
                compteur.fetch_add(1, Ordering::SeqCst);
                async { monthly_xml(2026, 11, &[("USD", 1, "0.8800")]) }
            }),
        );
    let source = HttpRateSource::with_base_url(serveur(app).await);
    assert_eq!(
        source.fetch_month(2026, 11).await,
        Err(SourceError::HttpStatus(302))
    );
    assert_eq!(
        cible.load(Ordering::SeqCst),
        0,
        "la redirection n'est pas suivie"
    );
}

/// XML tronqué : le client rend le corps, l'analyse le refuse (`malformed`).
#[tokio::test]
async fn xml_tronque() {
    let app = Router::new().route(
        "/api/xmlavgmonth",
        get(|| async {
            let complet = monthly_xml(2026, 11, &[("USD", 1, "0.8800")]);
            complet[..complet.len() / 2].to_string()
        }),
    );
    let source = HttpRateSource::with_base_url(serveur(app).await);
    let corps = source.fetch_month(2026, 11).await.unwrap();
    let e = parse_monthly_rates(corps.as_bytes()).unwrap_err();
    assert!(matches!(e, RatesError::Malformed(_)));
    assert_eq!(e.code(), "malformed");
}

/// Corps trop gros annoncé par `Content-Length` → `too_large`.
#[tokio::test]
async fn corps_trop_gros_avec_content_length() {
    let app = Router::new().route(
        "/api/xmlavgmonth",
        get(|| async { vec![b'a'; MAX_BODY_BYTES + 1] }),
    );
    let source = HttpRateSource::with_base_url(serveur(app).await);
    assert_eq!(
        source.fetch_month(2026, 11).await,
        Err(SourceError::TooLarge)
    );
}

/// Corps trop gros **sans** `Content-Length` (flux par morceaux, la forme de
/// la source réelle) → `too_large`, à la lecture bornée.
#[tokio::test]
async fn corps_trop_gros_sans_content_length() {
    let app = Router::new().route(
        "/api/xmlavgmonth",
        get(|| async {
            // Un flux de 20 × 64 Kio écrit dans un tube : corps sans longueur.
            let (mut ecrivain, lecteur) = tokio::io::duplex(64 * 1024);
            tokio::spawn(async move {
                use tokio::io::AsyncWriteExt;
                for _ in 0..20 {
                    if ecrivain.write_all(&[b'a'; 64 * 1024]).await.is_err() {
                        break;
                    }
                }
            });
            let reponse: Response =
                Body::from_stream(tokio_util::io::ReaderStream::new(lecteur)).into_response();
            assert!(reponse.headers().get(header::CONTENT_LENGTH).is_none());
            reponse
        }),
    );
    let source = HttpRateSource::with_base_url(serveur(app).await);
    assert_eq!(
        source.fetch_month(2026, 11).await,
        Err(SourceError::TooLarge)
    );
}

/// 404 de `xmldaily` (corps JSON, relevé du 2026-11-14) → pas encore publié.
#[tokio::test]
async fn quatre_cent_quatre_journalier_pas_encore_publie() {
    let app = Router::new().route(
        "/api/xmldaily",
        get(|| async {
            (
                StatusCode::NOT_FOUND,
                [(header::CONTENT_TYPE, "application/json")],
                r#"{"error":"Aucune donnée disponible","message":"Aucun taux de change trouvé pour la période demandée"}"#,
            )
        }),
    );
    let source = HttpRateSource::with_base_url(serveur(app).await);
    assert_eq!(
        source.fetch_day(d(2030, 12, 31)).await,
        Err(SourceError::NotPublished)
    );
}

/// 404 de `xmlavgmonth` → une erreur (`http_status:404`), jamais « pas encore
/// publié ».
#[tokio::test]
async fn quatre_cent_quatre_mensuel_est_une_erreur() {
    let app = Router::new().route("/api/xmlavgmonth", get(|| async { StatusCode::NOT_FOUND }));
    let source = HttpRateSource::with_base_url(serveur(app).await);
    assert_eq!(
        source.fetch_month(2026, 11).await,
        Err(SourceError::HttpStatus(404))
    );
}

/// Connexion refusée (aucun serveur sur le port) → `connect`.
#[tokio::test]
async fn connexion_refusee() {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    drop(listener);
    let source = HttpRateSource::with_base_url(format!("http://{addr}"));
    assert_eq!(
        source.fetch_month(2026, 11).await,
        Err(SourceError::Connect)
    );
}
