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

/// Politique des pages HTML.
///
/// Story 30-2a (AC6, C-30-2-6) — l'aperçu d'un document lit ses octets par
/// `apiClient.getBlob` (qui gère le rafraîchissement du jeton) et les affiche
/// par une URL `blob:` : en `<img>` (PNG, JPEG, d'où `img-src … blob:`) ou en
/// `<iframe>` (PDF, d'où `frame-src 'self' blob:`). `object-src 'self' blob:`
/// est **explicite** et non hérité de `default-src` : le document `blob:` d'une
/// `<iframe>` hérite de la CSP de la page, et la visionneuse PDF du navigateur
/// peut y passer par un objet embarqué. `default-src`, `script-src` et
/// `frame-ancestors` ne changent pas.
const CSP_VALUE: &str = "default-src 'self'; \
    script-src 'self' 'unsafe-inline'; \
    style-src 'self' 'unsafe-inline'; \
    img-src 'self' data: blob:; \
    frame-src 'self' blob:; \
    object-src 'self' blob:; \
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

#[cfg(test)]
mod tests {
    use super::CSP_VALUE;

    /// Directive `name` de la politique, valeurs comprises (`None` si absente).
    fn directive(name: &str) -> Option<&'static str> {
        CSP_VALUE
            .split(';')
            .map(str::trim)
            .find(|d| d.split_whitespace().next() == Some(name))
    }

    /// Story 30-2a (AC6) — la valeur est figée directive par directive.
    /// L'aperçu d'un document passe par une URL `blob:` : `img-src` (PNG, JPEG),
    /// `frame-src` (PDF dans une `<iframe>`) et `object-src` (visionneuse PDF du
    /// navigateur, objet embarqué du document `blob:` qui hérite de la CSP de la
    /// page) l'autorisent **explicitement** ; `default-src`, `script-src` et
    /// `frame-ancestors` ne changent pas (bornes d'AC6-bis). Tue M-a8.
    #[test]
    fn csp_value_is_pinned_and_allows_blob_previews() {
        assert_eq!(directive("default-src"), Some("default-src 'self'"));
        assert_eq!(
            directive("script-src"),
            Some("script-src 'self' 'unsafe-inline'")
        );
        assert_eq!(directive("img-src"), Some("img-src 'self' data: blob:"));
        assert_eq!(directive("frame-src"), Some("frame-src 'self' blob:"));
        assert_eq!(directive("object-src"), Some("object-src 'self' blob:"));
        assert_eq!(directive("frame-ancestors"), Some("frame-ancestors 'none'"));
        assert_eq!(
            CSP_VALUE
                .split(';')
                .map(str::trim)
                .filter(|d| !d.is_empty())
                .count(),
            11,
            "onze directives : toute directive ajoutée ou retirée se déclare ici"
        );
    }
}
