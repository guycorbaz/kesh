//! Story 10-5 — Middleware Content-Security-Policy défensif sur réponses HTML.
//!
//! Émet un header `Content-Security-Policy` sur les réponses dont le
//! `Content-Type` commence par `text/html` (typiquement le fallback `ServeDir`
//! servant `frontend/build/index.html`). Les réponses JSON `/api/v1/*` ne
//! reçoivent pas le header (filtre content-type).
//!
//! Pass 1 F-AC-P1-3 / D3 acté : `script-src 'unsafe-inline'` obligatoire car
//! SvelteKit emit un script inline d'amorçage non-déterministe dans
//! `frontend/build/index.html` (`Promise.all([import(...)]).then(...)`) —
//! `script-src 'self'` strict bloquerait l'app en whitepage. Migration vers
//! `'sha256-...'` ou `'nonce-...'` envisageable v0.2 (limitation L2).
//!
//! Pass 3 F-CSP-API-ATTACK-SURFACE-P3-3 architectural Axum 0.8 gotcha : le
//! `.layer()` qui monte ce middleware DOIT être appliqué APRÈS
//! `.fallback_service(fallback)` dans `lib.rs`, sinon il n'enveloppe pas le
//! ServeDir → header CSP absent sur `/login`, `/`, etc.

use axum::extract::Request;
use axum::http::HeaderValue;
use axum::middleware::Next;
use axum::response::Response;

const CSP_VALUE: &str = "default-src 'self'; \
    script-src 'self' 'unsafe-inline'; \
    style-src 'self' 'unsafe-inline'; \
    img-src 'self' data:; \
    font-src 'self' data:; \
    connect-src 'self'; \
    frame-ancestors 'none'; \
    base-uri 'self'; \
    form-action 'self'";

/// Middleware Axum qui ajoute `Content-Security-Policy` aux réponses HTML.
///
/// Filter `content-type.starts_with("text/html")` — couvre `text/html` et
/// `text/html; charset=utf-8` (tower-http émet en lowercase). Pas appliqué
/// aux réponses JSON `/api/v1/*` (inoffensif) ni aux assets statiques
/// (`application/javascript`, `image/*`, etc.).
pub async fn csp_html(request: Request, next: Next) -> Response {
    let mut response = next.run(request).await;
    let content_type = response
        .headers()
        .get("content-type")
        .and_then(|h| h.to_str().ok())
        .unwrap_or("");
    if content_type.starts_with("text/html") {
        response.headers_mut().insert(
            "Content-Security-Policy",
            HeaderValue::from_static(CSP_VALUE),
        );
    }
    response
}
