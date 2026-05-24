# Story 10.5: httpOnly tokens (sécurité — Option A)

Status: ready-for-dev

<!-- Validate optional via `bmad-create-story validate 10-5` avant `bmad-dev-story 10-5` -->

## Story

**As a** utilisateur Kesh (fiduciaire, PME, indépendant),
**I want** que mes tokens d'authentification soient inaccessibles au JavaScript du navigateur (cookies `HttpOnly` + `Secure` + `SameSite=Strict`),
**So that** une faille XSS éventuelle ne permette pas le vol immédiat de mon `access_token` / `refresh_token` depuis `document.cookie` ou `localStorage`.

**Provenance** : GitHub Issue [#41 [KF-002]](https://github.com/guycorbaz/kesh/issues/41) (catégorie A confirmée pre-flight Epic 10 décision D4). Story épic-10.md §"Story 10-5 : httpOnly tokens (sécurité — Option A)".

**Constat technique pré-Story** :

- `frontend/src/lib/app/stores/auth.svelte.ts:74-86` persiste `access_token` + `refresh_token` + `expires_in` dans `window.localStorage` via `setItem(STORAGE_KEY_ACCESS_TOKEN, ...)` etc.
- `crates/kesh-api/src/routes/auth.rs` retourne `Json<LoginResponse>` / `Json<RefreshResponse>` avec tokens en body — **aucun `Set-Cookie`** sur les 407 lignes du fichier (`grep -c "Set-Cookie\|set_cookie\|TypedHeader<Cookie>" routes/auth.rs` = 0).
- `crates/kesh-api/src/middleware/auth.rs:75` lit exclusivement `Authorization: Bearer <token>` header (`BEARER_PREFIX_LEN: usize = 7`). Aucune lecture de cookie.
- Risque XSS : un script malveillant injecté (via dépendance npm compromise, faille de rendu Svelte hypothétique, etc.) peut lire `window.localStorage.getItem('kesh:auth:accessToken')` et exfiltrer le JWT vers un endpoint externe — vol immédiat avec ~15 minutes d'usage avant expiration.

## Acceptance Criteria

### Backend cookies sécurisés (AC #1-6)

1. **Given** `POST /api/v1/auth/login` avec credentials valides, **When** la réponse est émise, **Then** elle contient **2 headers `Set-Cookie`** :
   - `kesh_access_token=<jwt>; HttpOnly; Secure; SameSite=Strict; Path=/; Max-Age=<jwt_expiry_seconds>` (typiquement 900s = 15 min, valeur lue depuis `state.config.jwt_expiry`).
   - `kesh_refresh_token=<uuid>; HttpOnly; Secure; SameSite=Strict; Path=/api/v1/auth; Max-Age=<refresh_inactivity_seconds>` (typiquement 900s = 15 min sliding, valeur lue depuis `state.config.refresh_inactivity`).
   - Note `Path=/api/v1/auth` restreint le refresh cookie aux endpoints auth (le browser ne l'enverra pas sur `/api/v1/invoices/*` etc.) — réduit la surface d'attaque cookie leak.

2. **Given** réponse `login` ci-dessus, **When** inspection DevTools `Application > Cookies`, **Then** flag **HttpOnly = ☑** présent sur les 2 cookies, et `document.cookie` exécuté en console JavaScript retourne soit une chaîne vide soit une chaîne ne contenant **pas** les valeurs des tokens.

3. **Given** `POST /api/v1/auth/refresh` (sans body, ou avec body vide `{}`), **When** le browser envoie automatiquement le cookie `kesh_refresh_token` (Path=/api/v1/auth), **Then** le backend lit ce cookie, valide le refresh token via la logique existante (rotation, revocation, sliding expiry), et émet **2 nouveaux `Set-Cookie`** (access + refresh — rotation des deux tokens). Body de réponse peut rester `{}` ou contenir uniquement `{ "expiresIn": <seconds> }` pour UX refresh proactif.

4. **Given** `POST /api/v1/auth/logout` (sans body), **When** la réponse est émise, **Then** elle contient **2 headers `Set-Cookie`** avec `Max-Age=0` (invalidation immédiate browser-side) : `kesh_access_token=; Max-Age=0; Path=/; HttpOnly; Secure; SameSite=Strict` et `kesh_refresh_token=; Max-Age=0; Path=/api/v1/auth; HttpOnly; Secure; SameSite=Strict`. Le refresh_token correspondant est aussi révoqué en DB (logique Story 1-6 préservée — `refresh_tokens::revoke_by_token` avec reason `"logout"`).

5. **Given** un nouvel endpoint `GET /api/v1/auth/me`, **When** appelé avec le cookie `kesh_access_token` valide, **Then** retourne `200 { "userId": "...", "username": "...", "role": "Admin|...", "expiresIn": <seconds_remaining> }` permettant au frontend de restaurer son état auth sans avoir besoin de lire le JWT (qui n'est plus accessible côté JS). `username` provient d'un lookup DB sur `user_id` (les claims JWT n'incluent pas `username`).

6. **Given** middleware `require_auth` (`crates/kesh-api/src/middleware/auth.rs`), **When** une requête arrive, **Then** le middleware lit le JWT en priorité depuis le **cookie `kesh_access_token`**, et en fallback depuis le header `Authorization: Bearer ...` (pour préserver la compatibilité ascendante avec les tests existants `crates/kesh-api/tests/*_e2e.rs` qui injectent l'Authorization header explicitement, et avec l'éventuel mode API pour automation hors-browser).

### Frontend retrait localStorage (AC #7-12)

7. **Given** `frontend/src/lib/app/stores/auth.svelte.ts`, **When** review post-Story, **Then** **aucun** `window.localStorage.setItem(STORAGE_KEY_ACCESS_TOKEN, ...)` ni `setItem(STORAGE_KEY_REFRESH_TOKEN, ...)` n'est appelé (`grep -nF "localStorage.setItem" auth.svelte.ts` retourne 0 résultats pour les 2 keys ACCESS_TOKEN + REFRESH_TOKEN). Les constantes `STORAGE_KEY_ACCESS_TOKEN` et `STORAGE_KEY_REFRESH_TOKEN` peuvent rester exportées si d'autres modules y font référence (mais leur valeur n'est jamais écrite ni lue par auth-store post-Story).

8. **Given** `frontend/src/lib/shared/utils/api-client.ts:buildHeaders`, **When** une requête `apiClient.get/post/...` est émise, **Then** **aucun** header `Authorization: Bearer <token>` n'est ajouté (le browser envoie automatiquement le cookie `kesh_access_token` via l'option `credentials: 'include'` sur le `fetch`).

9. **Given** `fetchWithRetry` et `fetchWithTimeout` (`api-client.ts:192,244`), **When** appelés, **Then** ils incluent **`credentials: 'include'`** dans les options `fetch()` pour permettre l'envoi automatique des cookies cross-origin (utile en dev local quand le frontend Vite tourne sur `:5173` et l'API sur `:3000` — `SameSite=Strict` empêche le cross-origin, mais `same-site` reste OK).

10. **Given** `auth.svelte.ts:hydrate()` appelée au démarrage de l'app, **When** elle s'exécute, **Then** **au lieu** de lire `localStorage.getItem(STORAGE_KEY_ACCESS_TOKEN)`, elle fait `await fetch('/api/v1/auth/me', { credentials: 'include' })`. Si la réponse est 200 → restauration `authState.currentUser = { userId, role }` + `_expiresIn` depuis le body. Si 401 → état non-auth (utilisateur doit se relogger). Si erreur réseau → silently swallow (état non-auth, comportement actuel préservé).

11. **Given** `auth.svelte.ts:login()`, **When** elle reçoit la réponse `POST /login`, **Then** elle **n'extrait plus** `accessToken` ni `refreshToken` du body (qui peut continuer à les retourner pour rétro-compat, ou ne plus les retourner — choix backend AC #1). Elle lit seulement `expiresIn` pour pile-bumping refresh, et fait éventuellement immediately `await fetch('/api/v1/auth/me')` pour récupérer `userId`, `role`, `username`. Le state `authState.currentUser` est rempli depuis `/me`, pas depuis les claims JWT (qui ne sont plus accessibles).

12. **Given** `auth.svelte.ts:logout()`, **When** appelée, **Then** elle appelle `POST /api/v1/auth/logout` avec `credentials: 'include'` (sans body refresh_token car le backend lit le cookie). Elle clear `_accessToken = null`, `_refreshToken = null`, etc. Le cleanup `localStorage` peut être retiré (les keys ne sont plus écrites donc plus à clean) OU conservé comme defensive cleanup pour les utilisateurs migrant depuis une session pre-Story 10-5 (à documenter dans le code).

### Defense-in-depth CSP + tests XSS (AC #13-15)

13. **Given** middleware HTTP backend (`crates/kesh-api/src/middleware/`), **When** une réponse HTML est émise (typiquement le fallback `ServeDir` qui sert `index.html`), **Then** elle contient un header **`Content-Security-Policy`** restrictif. **Valeur minimale recommandée** : `default-src 'self'; script-src 'self'; style-src 'self' 'unsafe-inline'; img-src 'self' data:; font-src 'self' data:; connect-src 'self'; frame-ancestors 'none'; base-uri 'self'; form-action 'self'`. La directive `'unsafe-inline'` sur `style-src` est tolérée car SvelteKit emit des styles inline pour les scoped components — à valider lors de l'implémentation (si SvelteKit hash-based CSP est faisable, préférer `'sha256-...'` ou `'nonce-...'`). **Cible** : aucune CSP violation console au boot d'une page authentifiée standard (login, dashboard, contacts, journal-entries, reports).

14. **Given** test E2E Playwright `frontend/tests/e2e/security/xss-token-protection.spec.ts` (nouveau fichier), **When** exécuté, **Then** il vérifie 3 invariants :
    - **(a)** Après login, `document.cookie` lu en console JavaScript retourne une chaîne qui ne contient **pas** le JWT ni le refresh UUID (assertion `expect(cookie).not.toContain(token)`).
    - **(b)** Après login, `localStorage.getItem('kesh:auth:accessToken')` retourne `null` (assertion `expect(value).toBeNull()`).
    - **(c)** Un payload XSS simulé injecté via `page.evaluate()` qui tente `fetch('/api/v1/auth/me', { credentials: 'omit' })` reçoit **401** (cookie non-envoyé sans `credentials: include`), prouvant que le cookie reste protégé même si du JS hostile s'exécute dans la page.

15. **Given** `crates/kesh-api/tests/auth_cookies_e2e.rs` (nouveau fichier d'intégration), **When** exécuté via `cargo test -p kesh-api --test auth_cookies_e2e -- --test-threads=1`, **Then** il valide les 4 scénarios :
    - **(a)** Login → réponse contient `Set-Cookie: kesh_access_token=...; HttpOnly; Secure; SameSite=Strict; Path=/; Max-Age=...` + idem pour `kesh_refresh_token` Path=/api/v1/auth.
    - **(b)** Requête authentifiée avec cookie seul (sans Authorization header) → 200 OK.
    - **(c)** Requête authentifiée avec Authorization header seul (sans cookie) → 200 OK (fallback préservé).
    - **(d)** Logout → réponse contient `Set-Cookie: kesh_access_token=; Max-Age=0` + refresh idem, et requête authentifiée subséquente avec l'ancien cookie → 401 (révoqué côté DB ET expiré côté browser).

### Compatibilité, breaking, documentation (AC #16-18)

16. **Given** la suite de tests backend `cargo test --workspace -j1 -- --test-threads=1`, **When** exécutée, **Then** **0 régression** sur les tests `auth_e2e.rs` existants (rate-limit, refresh rotation, password change, dummy_verify timing, etc. — Stories 1.5, 1.6). Le fallback `Authorization: Bearer` du middleware (AC #6) garantit qu'aucun test existant n'a besoin de modification.

17. **Given** la suite de tests E2E `npm run test:e2e`, **When** exécutée, **Then** **0 régression** sur les 91 baselines actuelles (status Story 10-3 post-merge). Les tests qui font `page.evaluate(() => localStorage.getItem('kesh:auth:accessToken'))` (e.g. `frontend/tests/e2e/bank-import-confirms.spec.ts:50-53` `authHeaders` helper) doivent être adaptés pour utiliser `credentials: 'include'` au lieu de lire le token + l'injecter en Authorization header. Liste exhaustive des tests à adapter à identifier ground-truth via `grep -rn "localStorage.getItem.*kesh:auth" frontend/tests/e2e/`.

18. **Given** `CHANGELOG.md` racine (créé Story 10-4), **When** Story 10-5 mergée, **Then** entrée `[0.1.0]` section **Security** (nouvelle sous-section) ajoutée : "**Sécurité durcie** : les tokens d'authentification sont maintenant stockés en cookies `HttpOnly` + `Secure` + `SameSite=Strict`, inaccessibles au JavaScript du navigateur. Élimine la possibilité de vol immédiat des tokens via une faille XSS hypothétique. Nouveau endpoint `GET /api/v1/auth/me` permet au frontend de restaurer l'état utilisateur sans lire les tokens. Headers CSP défensifs ajoutés sur les réponses HTML (defense-in-depth)." **Et** issue [#41](https://github.com/guycorbaz/kesh/issues/41) fermée avec `closes #41` dans le message du commit final (squash-merge sur main).

## Tasks / Subtasks

### T1: Dépendance Cargo `axum-extra` cookie feature (AC #1-6)

- [ ] **T1.1** : ajouter `axum-extra = { version = "0.10", features = ["typed-header", "cookie"] }` à `crates/kesh-api/Cargo.toml` dans `[dependencies]`. Le crate `axum-extra` fournit `CookieJar` extractor et `PrivateCookieJar` (avec signing si on veut ajouter SHA d'intégrité — pas requis Story 10-5, simple `CookieJar` suffit).
- [ ] **T1.2** : `cargo build -p kesh-api` PASS après ajout (vérification compile).

### T2: Backend `auth.rs` — Set-Cookie sur login + refresh + logout (AC #1, #3, #4)

- [ ] **T2.1** : refactor `login` handler (`crates/kesh-api/src/routes/auth.rs:85-180`) pour retourner `(CookieJar, Json<LoginResponse>)` (tuple Axum standard avec `IntoResponse`). Construire les 2 cookies via `Cookie::build("kesh_access_token", access_token).http_only(true).secure(true).same_site(SameSite::Strict).path("/").max_age(Duration::seconds(state.config.jwt_expiry.num_seconds())).build()`. Idem pour `kesh_refresh_token` avec `path("/api/v1/auth")` et `max_age` depuis `state.config.refresh_inactivity`. `let jar = jar.add(access_cookie).add(refresh_cookie); Ok((jar, Json(response)))`. Body `LoginResponse` peut conserver les champs `access_token` + `refresh_token` pour rétro-compat tests (AC #6 fallback), OU les retirer si on décide breaking total — à décider lors de l'implémentation selon le coût d'adapter les tests.
- [ ] **T2.2** : refactor `refresh` handler (`auth.rs:228-316`) : extraire `jar: CookieJar` du request (Axum extractor), lire `jar.get("kesh_refresh_token").map(|c| c.value())` pour obtenir le refresh_token. Si absent ou vide → fallback lecture body `req.refresh_token` (rétro-compat tests). Émettre nouveaux cookies via la même logique T2.1.
- [ ] **T2.3** : refactor `logout` handler (`auth.rs:188-196`) : émettre cookies expirés (`Cookie::build(...).max_age(Duration::seconds(0)).build()` ou méthode dédiée `Cookie::named("kesh_access_token")` puis `.remove()` selon API axum-extra). Conserver la révocation DB du refresh_token (logique Story 1-6 préservée).

### T3: Backend `middleware/auth.rs` — cookie-first avec fallback header (AC #6)

- [ ] **T3.1** : modifier `require_auth` (`crates/kesh-api/src/middleware/auth.rs:61`) pour extraire `jar: CookieJar` en plus de `request: Request`. Lire `jar.get("kesh_access_token").map(|c| c.value())` en priorité. Si absent → fallback sur le code actuel `Authorization: Bearer ...` (lignes 73-75+). Le reste de la logique (decode JWT, validate exp, attacher `CurrentUser` au request) reste inchangée.
- [ ] **T3.2** : adapter `tests/middleware/auth.rs` (lignes ~200-320 — tests `require_auth_*`) : ajouter au moins 2 nouveaux tests : (a) `require_auth_accepts_cookie_no_authorization` et (b) `require_auth_prefers_cookie_over_header_when_both_present`. Les tests existants `require_auth_accepts_valid_authorization` doivent rester verts (fallback header préservé).

### T4: Backend nouveau endpoint `GET /api/v1/auth/me` (AC #5)

- [ ] **T4.1** : ajouter handler `me` dans `crates/kesh-api/src/routes/auth.rs` :
  ```rust
  #[derive(Debug, Serialize)]
  #[serde(rename_all = "camelCase")]
  pub struct MeResponse {
      pub user_id: i64,
      pub username: String,
      pub role: String,
      pub expires_in: i64, // seconds remaining until JWT exp
  }
  pub async fn me(
      State(state): State<AppState>,
      Extension(current_user): Extension<CurrentUser>,
  ) -> Result<Json<MeResponse>, AppError> {
      let user = users::find_by_id(&state.pool, current_user.user_id).await?
          .ok_or(AppError::Internal("current user not found in DB".into()))?;
      let expires_in = (current_user.exp - chrono::Utc::now().timestamp()).max(0);
      Ok(Json(MeResponse { user_id: user.id, username: user.username, role: format!("{:?}", user.role), expires_in }))
  }
  ```
  Verifier que `CurrentUser` (ligne 17 import) expose bien `user_id`, `role`, et `exp` (sinon adapter la structure pour exposer `exp` — actuellement `crates/kesh-api/src/middleware/auth.rs` constate ce qui est exposé). Si `exp` n'est pas dans `CurrentUser`, on peut soit l'ajouter (changement bénin), soit le récupérer en re-décodant le JWT (anti-pattern, préférer ajout).
- [ ] **T4.2** : monter la route dans `crates/kesh-api/src/lib.rs` : ajouter `.route("/api/v1/auth/me", get(routes::auth::me))` dans le bloc `require_auth` (ligne ~270 — pas dans le bloc `require_role` puisque tous les rôles sont autorisés à connaître leur identité).
- [ ] **T4.3** : test intégration dans `crates/kesh-api/tests/auth_cookies_e2e.rs` (créé T9) : login → GET /me → assert body shape et valeurs.

### T5: Backend CSP headers défensifs (AC #13)

- [ ] **T5.1** : créer middleware `crates/kesh-api/src/middleware/csp.rs` (nouveau fichier ~30 lignes) qui ajoute un header `Content-Security-Policy` sur les réponses **dont le `Content-Type` est `text/html`** (ne pas appliquer aux réponses JSON `/api/v1/*` — inutile, et risque de break si CSP réflexive). Pattern Axum :
  ```rust
  use axum::middleware::Next;
  use axum::response::Response;
  pub async fn csp_html(request: axum::extract::Request, next: Next) -> Response {
      let mut response = next.run(request).await;
      let content_type = response.headers().get("content-type").and_then(|h| h.to_str().ok()).unwrap_or("");
      if content_type.starts_with("text/html") {
          response.headers_mut().insert(
              "Content-Security-Policy",
              "default-src 'self'; script-src 'self'; style-src 'self' 'unsafe-inline'; img-src 'self' data:; font-src 'self' data:; connect-src 'self'; frame-ancestors 'none'; base-uri 'self'; form-action 'self'".parse().unwrap(),
          );
      }
      response
  }
  ```
- [ ] **T5.2** : monter le middleware dans `crates/kesh-api/src/lib.rs` (ligne ~430 où sont les `.layer(middleware::from_fn(...))`). Ordre : juste avant ou après le layer `require_auth` — peu importe puisque CSP s'applique uniquement sur réponses `text/html` (donc routes publiques `/` ServeDir).
- [ ] **T5.3** : validation manuelle : `curl -I http://localhost:3000/login` → vérifier que header `Content-Security-Policy: ...` est présent. `curl -I http://localhost:3000/api/v1/auth/me` (sans auth) → 401, **sans** header CSP (puisque body JSON pas HTML).
- [ ] **T5.4** : optionnel — valider via DevTools console qu'aucune CSP violation ne fire au boot d'une page SvelteKit normale (login, dashboard, etc.). Si SvelteKit emit des styles inline qui violent `'unsafe-inline'` ou des scripts inline qui violent `'self'`, ajuster la directive (e.g. `style-src 'self' 'unsafe-inline'` est déjà tolérant ; `script-src 'self'` strict — si violation, considérer `'nonce-...'` ou `'sha256-...'`).

### T6: Frontend `auth.svelte.ts` — retrait localStorage + consumer `/me` (AC #7, #10, #11, #12)

- [ ] **T6.1** : retirer les 3 `window.localStorage.setItem(STORAGE_KEY_*, ...)` dans `login()` (lignes 82-86). Le `_accessToken`, `_refreshToken`, `_expiresIn` `$state` peuvent rester (utiles pour `isAuthenticated` getter et refresh proactif), mais NE plus persister en localStorage.
- [ ] **T6.2** : retirer les 3 `window.localStorage.removeItem(STORAGE_KEY_*, ...)` dans `clearSession()` (lignes 100-104) ET dans `logout()` (lignes 126-130) — ou les laisser comme defensive cleanup migration (cf. AC #12) avec un commentaire `// Pass 10-5 : defensive cleanup pour utilisateurs migrant depuis localStorage`.
- [ ] **T6.3** : refactor `hydrate()` (lignes 140-185) : **remplacer entièrement** la logique `localStorage.getItem(...)` par `await fetch('/api/v1/auth/me', { credentials: 'include' })`. Si réponse 200 → parse body, set `_accessToken = '<not-accessible>'` (placeholder string non-vide pour que `isAuthenticated` getter retourne `true`, ou plus propre : convertir `isAuthenticated` getter pour qu'il dépende de `_currentUser !== null` à la place de `_accessToken !== null`). Set `_currentUser = { userId: response.userId, role: response.role }`, `_expiresIn = response.expiresIn`. Si réponse 401 → state non-auth (les getters retournent null). Si réseau fail → catch silencieux, state non-auth.
- [ ] **T6.4** : adapter `login()` : après le `POST /login` qui retourne `expiresIn` (le browser a déjà set le cookie HttpOnly), faire un second `await fetch('/api/v1/auth/me', { credentials: 'include' })` pour récupérer `userId`, `role` (qu'on ne peut plus extraire du JWT côté JS). **OU** : modifier `LoginResponse` backend pour inclure `{ userId, role, expiresIn }` (sans tokens) — choix architectural à acter lors de l'implémentation (préférer la 2e option : 1 round-trip au lieu de 2).
- [ ] **T6.5** : `isAuthenticated` getter (`auth.svelte.ts:70-72`) — adapter pour dépendre de `_currentUser !== null` au lieu de `_accessToken !== null` (puisque `_accessToken` n'est plus accessible dans le cookie scenario).
- [ ] **T6.6** : `_hydrated` guard (ligne 142) reste utile pour éviter les double-fetches concurrents `/me`.

### T7: Frontend `api-client.ts` — credentials include + retrait Authorization header (AC #8, #9)

- [ ] **T7.1** : modifier `fetchWithTimeout` (`frontend/src/lib/shared/utils/api-client.ts:192-199`) pour ajouter `credentials: 'include'` à l'objet options passé à `fetch()` : `return await fetch(url, { ...init, credentials: 'include', signal: controller.signal })`.
- [ ] **T7.2** : modifier `buildHeaders` (`api-client.ts:124-146`) — retirer entièrement le bloc lignes 141-143 qui ajoute `Authorization: Bearer ${authState.accessToken}`. **Garder** la logique `AUTH_EXCLUDED_URLS` (login/logout/refresh) pour information, mais comme aucun header Authorization n'est jamais ajouté, son rôle change : elle ne sert plus à rien pour le frontend post-Story 10-5. On peut soit la retirer entièrement (cleanup), soit la laisser commentée pour traçabilité. **Recommandation** : retirer pour clarté.
- [ ] **T7.3** : modifier `api-client.ts:doRefresh` (lignes 57-104) — le `fetch('/api/v1/auth/refresh', { method: 'POST', headers: { 'Content-Type': 'application/json' }, body: JSON.stringify({ refreshToken: currentRefreshToken }) })` (lignes 67-71) : retirer le body (le browser envoie le cookie) ou laisser un body vide `'{}'`. Adapter aussi la lecture de la réponse — selon ce que le backend retourne post-AC #3 (juste `expiresIn` ou tokens en body fallback).
- [ ] **T7.4** : adapter `+layout.svelte` onMount (`frontend/src/routes/+layout.svelte:30-51`) — l'auth gate `if (authState.accessToken !== null)` (ligne 50) doit être adaptée pour `if (authState.isAuthenticated)` (cohérent avec T6.5) puisque `_accessToken` n'est plus défini en mode cookie.

### T8: Frontend tests Vitest — adaptation (AC #7, #11)

- [ ] **T8.1** : `frontend/src/lib/app/stores/auth.svelte.test.ts` (si existe) — adapter les tests `login`, `hydrate`, `logout` pour le nouveau flow (`/me` fetch mocké via `vi.stubGlobal('fetch', ...)`) au lieu de `localStorage`.
- [ ] **T8.2** : `frontend/src/lib/shared/utils/api-client.test.ts` — adapter les tests existants si certains injectent `authState.accessToken` directement (via `authState.login(...)` ligne ~556) — vérifier que `mockFetch` reçoit bien `credentials: 'include'` dans les options.

### T9: Test intégration backend `tests/auth_cookies_e2e.rs` (AC #15)

- [ ] **T9.1** : créer `crates/kesh-api/tests/auth_cookies_e2e.rs` (nouveau fichier ~150-200 lignes) avec 4 tests `#[sqlx::test(migrator = "kesh_db::MIGRATOR")]` :
  - **`login_sets_two_httponly_cookies`** (AC #1) : POST /login → assertions sur les headers `Set-Cookie` (pattern `assert!(set_cookie_header.contains("HttpOnly")); assert!(set_cookie_header.contains("SameSite=Strict")); ...`).
  - **`authenticated_request_with_cookie_only`** (AC #6) : login → extract cookies → request `/api/v1/companies/current` avec cookie seul (sans header Authorization) → 200 OK.
  - **`authenticated_request_with_authorization_only`** (AC #6 fallback) : login → extract access_token from body OR via cookie parsing → request avec `Authorization: Bearer ...` sans cookie → 200 OK.
  - **`logout_invalidates_cookie`** (AC #4) : login → logout → assertions sur Set-Cookie `Max-Age=0` + DB check `refresh_tokens.revoked_at IS NOT NULL`.
- [ ] **T9.2** : pattern test reqwest comme `i18n_e2e.rs` ou `health_endpoint.rs` (les 19+ tests intégration `tests/*_e2e.rs`). `spawn_app(pool).await` + `reqwest::Client::builder().cookie_store(true).build()` pour activer le cookie jar reqwest.

### T10: Test E2E Playwright XSS protection (AC #14)

- [ ] **T10.1** : créer dossier `frontend/tests/e2e/security/` (nouveau) + fichier `xss-token-protection.spec.ts` (~80-120 lignes) avec 3 scénarios :
  - **`(a) document.cookie does not expose tokens`** : login via UI → `page.evaluate(() => document.cookie)` → assert `not.toContain(jwt_pattern)` et `not.toContain(uuid_pattern)`.
  - **`(b) localStorage does not contain tokens`** : login via UI → `page.evaluate(() => localStorage.getItem('kesh:auth:accessToken'))` → assert `toBeNull()`.
  - **`(c) XSS-simulated fetch without credentials fails`** : login → `page.evaluate(() => fetch('/api/v1/auth/me', { credentials: 'omit' }).then(r => r.status))` → assert `toBe(401)`.
- [ ] **T10.2** : `PLAYWRIGHT_HOST_PLATFORM_OVERRIDE=ubuntu24.04-x64 npm run test:e2e -- xss-token-protection.spec.ts` PASS (3 scénarios verts).

### T11: Documentation — CHANGELOG.md + manuel admin (AC #18)

- [ ] **T11.1** : éditer `CHANGELOG.md` (créé Story 10-4) — ajouter une nouvelle sous-section `### Sécurité` (ou `### Security`) dans l'entrée `[0.1.0]`, après la sous-section `Multi-utilisateurs et sécurité` actuelle. Contenu : description du durcissement httpOnly + Secure + SameSite, mention nouveau endpoint `/me`, mention CSP headers défensifs, mention breaking change accepté (single-user pre-prod = pas de migration utilisateur nécessaire).
- [ ] **T11.2** : éditer `docs/manual/fr/admin-manual.tex` §"Sécurité" (existante, ~ligne 1071+ pour `\subsection{Authentification JWT + refresh tokens}`) — ajouter mention "Story 10-5 : tokens stockés en cookies `HttpOnly` + `Secure` + `SameSite=Strict`, inaccessibles au JavaScript ; nouveau endpoint `GET /api/v1/auth/me` pour restoration d'état frontend ; headers CSP défensifs sur réponses HTML." Régénérer PDF via `latexmk -xelatex`.

### T12: Test Locally First + commit + sprint-status (AC #16, #17)

- [ ] **T12.1** : Test Locally First Backend : `cargo fmt --all -- --check`, `cargo build --workspace --all-targets`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace -j1 -- --test-threads=1` — tous PASS. Vérifier en particulier 0 régression sur `tests/auth_e2e.rs` existants (rate-limit, refresh rotation, change_password, dummy_verify) — le fallback Authorization header (AC #6) doit garantir ça.
- [ ] **T12.2** : Test Locally First Frontend : `npm run check`, `npm run lint-i18n-ownership`, `npm run test:unit -- --run`, `npm run build` — tous PASS.
- [ ] **T12.3** : E2E Playwright : `npm run test:e2e -- xss-token-protection.spec.ts` 3/3 PASS, **et** `npm run test:e2e` suite complète → 0 régression sur les 91 baselines actuelles (Story 10-3 état). Tests `bank-import-confirms.spec.ts:50-53` (`authHeaders` helper) et autres consumers `localStorage.getItem('kesh:auth:accessToken')` à adapter (cf. AC #17 et T8.2). Liste exhaustive à identifier via `grep -rn "localStorage.getItem.*kesh:auth" frontend/tests/e2e/` et `grep -rn "Authorization.*Bearer.*token" frontend/tests/e2e/`.
- [ ] **T12.4** : commit unique (ou commits par task selon préférence) sur branche `story/10-5-httponly-tokens-security`. Status sprint-status `10-5-httponly-tokens-security: ready-for-dev → in-progress` avant 1er commit puis `in-progress → review` au push de fin de dev-story. Bump status `10-4 review → done` au démarrage de Story 10-5 (pattern `feedback_avoid_parallel_prs`).
- [ ] **T12.5** : message du commit final inclut `closes #41` pour fermer automatiquement l'issue GitHub [KF-002] à la fusion sur main (AC #18).

## Dev Notes

### Architecture patterns à respecter

- **Pattern Axum CookieJar** : utiliser l'extractor `axum_extra::extract::CookieJar` (re-export depuis `axum-extra`) avec la signature `async fn handler(jar: CookieJar, ...) -> impl IntoResponse`. Retourner `(jar, body)` tuple pour émettre les Set-Cookie headers. Référence canonique : [axum-extra cookie docs](https://docs.rs/axum-extra/latest/axum_extra/extract/cookie/index.html).
- **Pattern Cookie Builder** : `Cookie::build(("name", "value")).http_only(true).secure(true).same_site(SameSite::Strict).path("/").max_age(time::Duration::seconds(N)).build()`. Note : `axum-extra` utilise le crate `cookie` qui dépend de `time` (pas `chrono`) pour `Duration` — attention à l'unit mismatch.
- **Pattern middleware Axum** : `async fn middleware_fn(request: Request, next: Next) -> Response` monté via `.layer(axum::middleware::from_fn(middleware_fn))`. Référence : `crates/kesh-api/src/middleware/auth.rs` (require_auth pattern).
- **Pattern test intégration** : `#[sqlx::test(migrator = "kesh_db::MIGRATOR")]` + `spawn_app(pool).await` + `reqwest::Client::builder().cookie_store(true).build()` pour gérer les cookies. Cohérent avec les 19+ tests `tests/*_e2e.rs`.

### Intelligence Story 10-4 (dernière story Epic 10 livrée)

- **Branche actuelle main** : `d250672` post-merge PR #111 Story 10-4.
- **Sprint-status** : `10-4-...: review` (sera bumpé `done` au démarrage 10-5 dans le 1er commit), `10-5-...: backlog → ready-for-dev` après création de cette spec.
- **CHANGELOG.md** : existe (Story 10-4), contient `[0.1.0]` avec section `Multi-utilisateurs et sécurité` — Story 10-5 doit ajouter une sous-section dédiée `Sécurité` ou enrichir la section existante.
- **Manuel admin** : `docs/manual/fr/admin-manual.tex` 204+ KB PDF, §"Sécurité" existe (§"Authentification JWT + refresh tokens") à enrichir T11.2.
- **Pattern code-review** : 4-pass adversarial Sonnet → Haiku → Opus → Sonnet (cf. CLAUDE.md §"Review Iteration Rule"). Sur Story 10-3, Pass 3 Opus a catché 3 HIGH architecturaux ratés par Sonnet+Haiku — anticiper le même type de findings pour Story 10-5 (race conditions cookie/header dans middleware fallback, scope path cookie trop large/étroit, CSP violations imprévues côté SvelteKit hydration, etc.).

### Décisions ouvertes à clarifier lors de l'implémentation

- **D1 — `LoginResponse` body : tokens en clair ou retirés** : Option A (tokens conservés en body pour rétro-compat tests + automation API) vs Option B (tokens retirés pour breaking complet, tests adaptés). Recommandation B si effort d'adaptation tests < 1h, sinon A. À acter T2.1.
- **D2 — `CurrentUser` middleware structure** : exposer `exp` (JWT expiration) en plus de `user_id` + `role` + `company_id` pour permettre à `me` handler de calculer `expires_in`. Sinon, re-decoder le JWT (anti-pattern). Recommandation : exposer `exp`. À acter T4.1.
- **D3 — CSP `script-src` strict vs `'unsafe-inline'`** : SvelteKit peut emit des scripts inline si certaines features sont activées. Tester la directive stricte d'abord, fallback `'unsafe-inline'` ou `'nonce-...'` si violations. À acter T5.4.
- **D4 — Tests E2E adaptation** : `bank-import-confirms.spec.ts` `authHeaders` helper et autres consumers à adapter (cf. AC #17). Choix entre (a) refactor du helper pour utiliser `credentials: 'include'` dans `page.request.post(...)` au lieu d'inject Authorization header, ou (b) garder le helper et faire un login API dans le test pour récupérer la cookie. Recommandation (a). À acter T12.3.

### Impact `Authorization: Bearer` fallback (AC #6)

Préserver le fallback header est **essentiel** pour ne pas casser les tests existants qui injectent explicitement `Authorization: Bearer <token>` :
- `crates/kesh-api/tests/*_e2e.rs` (19+ fichiers) utilisent tous le pattern `reqwest::Client::new() ... .bearer_auth(token)` ou équivalent.
- `frontend/tests/e2e/bank-import*.spec.ts` `authHeaders()` helper lit le token depuis localStorage et l'injecte en header.

Sans fallback, Story 10-5 deviendrait une refonte massive (>>1-2 jours estimés). Avec fallback, l'effort reste contenu sur la modification du middleware + nouveau endpoint + nouveau test cookie, sans toucher les tests historiques.

**Plan de retrait du fallback** (hors scope Story 10-5) : story dédiée v0.2 ou v0.3 quand tous les tests E2E auront migré vers le pattern cookie. Tracer comme limitation Story 10-5 catégorie B v0.2 (cf. §"Limitations" ci-dessous).

### Path cookie scope et défense en profondeur

- `kesh_access_token` `Path=/` : envoyé sur **toutes** les requêtes vers le domaine Kesh (browser default). Nécessaire car le middleware `require_auth` est appliqué sur la majorité des routes `/api/v1/*` (cf. `crates/kesh-api/src/lib.rs:268-365`).
- `kesh_refresh_token` `Path=/api/v1/auth` : envoyé **uniquement** sur les endpoints `/api/v1/auth/*` (login, logout, refresh, me, change-password). Réduit la surface d'attaque : si un endpoint hors auth a une faille de réflexion d'header (CRLF injection hypothétique, log leak via WAF mis-configuré, etc.), le refresh token n'est pas exposé.
- `SameSite=Strict` : empêche l'envoi du cookie en cross-site (e.g. lien depuis un email externe vers `https://kesh.exemple.ch/api/v1/...` ne portera pas le cookie → user devra se relogger). **Trade-off UX** : strict mais cohérent avec le profil d'usage Kesh (app interne fiduciaire, pas de deeplinks externes attendus).
- `Secure` : envoyé uniquement sur HTTPS. **En dev local** (Vite `:5173` + kesh-api `:3000` en HTTP), Chrome/Firefox autorisent `Secure` cookies sur `localhost` (exception). En prod, le reverse proxy HTTPS (Nginx/Caddy/Traefik/Synology Portail — cf. Story 10-4 §4.4/§4.5) doit terminer TLS pour que les cookies soient acceptés.

### Limitations identifiées (catégorie B v0.2 — à tracer si non-traitées Story 10-5)

- **L1 — Fallback Authorization header conservé** : permet aux tests E2E historiques de passer sans modification massive, mais la surface d'attaque reste théoriquement présente. À retirer en v0.2 (story dédiée "Backend cookie-only auth").
- **L2 — CSP `style-src 'unsafe-inline'`** : nécessaire pour SvelteKit scoped styles. Migration vers hash-based CSP (`'sha256-...'`) ou nonce-based en v0.2 — nécessite probablement une refonte du build SvelteKit pour collecter les hashes.
- **L3 — CSRF protection non-explicite** : `SameSite=Strict` couvre la majorité des cas CSRF, mais defense-in-depth via token CSRF (double-submit cookie ou synchronizer pattern) serait recommandée. Hors scope Story 10-5, à étudier v0.2.
- **L4 — Cookie size limit 4 KB** : un JWT typique HS256 fait ~500-800 bytes selon claims. Largement sous la limite. À surveiller si des claims supplémentaires sont ajoutés ultérieurement (e.g. permissions list, multi-tenant claims étendus).

### Project Structure Notes

| Fichier | Type | Lignes estimées |
|---------|------|-----------------|
| `crates/kesh-api/Cargo.toml` | M | +1 ligne dep axum-extra cookie |
| `crates/kesh-api/src/routes/auth.rs` | M | +60 lignes (login refactor + refresh refactor + logout refactor + me handler nouveau) |
| `crates/kesh-api/src/middleware/auth.rs` | M | +15 lignes (cookie-first fallback header) |
| `crates/kesh-api/src/middleware/csp.rs` | A | ~30 lignes (nouveau middleware CSP) |
| `crates/kesh-api/src/middleware/mod.rs` | M | +1 ligne export csp |
| `crates/kesh-api/src/lib.rs` | M | +2-3 lignes (route /me + layer csp_html) |
| `crates/kesh-api/tests/auth_cookies_e2e.rs` | A | ~150-200 lignes (4 tests intégration) |
| `frontend/src/lib/app/stores/auth.svelte.ts` | M | +20/-30 lignes (retrait localStorage, refactor hydrate via /me, login adapté) |
| `frontend/src/lib/shared/utils/api-client.ts` | M | +5/-10 lignes (credentials include, retrait Authorization header builder) |
| `frontend/src/lib/shared/utils/api-client.test.ts` | M | adaptation tests existants |
| `frontend/src/lib/app/stores/auth.svelte.test.ts` | M (si existe) | adaptation tests |
| `frontend/src/routes/+layout.svelte` | M | +1 ligne (isAuthenticated au lieu de accessToken) |
| `frontend/tests/e2e/security/xss-token-protection.spec.ts` | A | ~100 lignes (3 scénarios) |
| `frontend/tests/e2e/bank-import*.spec.ts` + autres | M | adaptation `authHeaders` helper pour `credentials: 'include'` |
| `CHANGELOG.md` | M | +10 lignes (sous-section Sécurité dans `[0.1.0]`) |
| `docs/manual/fr/admin-manual.tex` | M | +5 lignes (mention Story 10-5 dans §"Sécurité") |
| `docs/manual/fr/admin-manual.pdf` | M | régénéré |
| `_bmad-output/implementation-artifacts/sprint-status.yaml` | M | bump 10-4 done + 10-5 in-progress puis review |

Total estimé : ~14 fichiers M + 3 A, ~350-450 lignes net. Cohérent avec l'effort 1-2 jours annoncé epic-10.md.

### Testing standards summary

- **Backend** : cohérent §"Pattern test intégration" ci-dessus. Tests `#[sqlx::test(migrator = ...)]` avec `spawn_app(pool).await` + `reqwest::Client::builder().cookie_store(true).build()`. Baseline 14 tests Story 10-3 + 4 nouveaux Story 10-5 = 18 nouveaux tests intégration Epic 10.
- **Frontend Vitest** : adaptation des tests `auth.svelte.test.ts` et `api-client.test.ts` existants. Pas de nouveaux tests Vitest spécifiques Story 10-5 (la valeur testable est dans le E2E XSS).
- **E2E Playwright** : 3 nouveaux scénarios `xss-token-protection.spec.ts` + 0 régression sur les 91 baselines actuelles (Story 10-3). KFs ouvertes (#107 KF-030 bank-* + #108 KF-031 axe race + #96 KF-028 + #97 KF-029) restent ouvertes — Story 10-5 n'aggrave ni n'améliore leur statut.

### References

- [Epic 10 planning](../planning-artifacts/epic-10.md#story-10-5--httponly-tokens-securite-option-a) (lignes 265-317)
- [GitHub Issue #41 [KF-002]](https://github.com/guycorbaz/kesh/issues/41) — checklist détaillée
- [Architecture decision D4 epic-10.md ligne 76](../planning-artifacts/epic-10.md) — Tokens auth en cookies httpOnly + Secure + SameSite=Strict
- [Architecture decision D11 epic-10.md ligne 83](../planning-artifacts/epic-10.md) — Breaking change cookies confirmé acceptable
- [Story 10-3 résilience frontend DB inaccessible](./10-3-resilience-frontend-db-inaccessible.md) — pattern code-review 4-pass appliqué et CYCLE CONVERGED
- [Story 10-4 manuel install Synology backup CHANGELOG](./10-4-manuel-install-synology-backup-dsm-changelog.md) — CHANGELOG.md créé, manuel admin §"Sécurité" à enrichir
- [CLAUDE.md §"Review Iteration Rule"](../../CLAUDE.md#review-iteration-rule) — discipline cycle review obligatoire
- [CLAUDE.md §"Synchroniser TOUTES les docs avant tout push / création de release"](../../CLAUDE.md) — règle ajoutée Story PR #110 docs/rule-doc-sync-on-push-release
- [axum-extra cookie docs](https://docs.rs/axum-extra/latest/axum_extra/extract/cookie/index.html) — pattern CookieJar Axum
- [OWASP Session Management Cheat Sheet](https://cheatsheetseries.owasp.org/cheatsheets/Session_Management_Cheat_Sheet.html#cookies) — best practices HttpOnly + Secure + SameSite
- [MDN Set-Cookie SameSite](https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Set-Cookie/SameSite) — comportement Strict vs Lax vs None

## Dev Agent Record

### Agent Model Used

_(à remplir au démarrage `bmad-dev-story 10-5`)_

### Debug Log References

_(à remplir pendant l'implémentation)_

### Completion Notes List

_(à remplir au fil des tâches T1-T12)_

### File List

_(à remplir au commit final — récapitulatif des fichiers A/M)_

## Change Log

_(à remplir aux passes spec validate et code-review)_
