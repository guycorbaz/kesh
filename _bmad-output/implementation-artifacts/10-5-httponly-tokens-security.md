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
   - `kesh_refresh_token=<uuid>; HttpOnly; Secure; SameSite=Strict; Path=/api/v1/auth; Max-Age=<refresh_token_max_lifetime_seconds>` (**Pass 3 F-COOKIE-LIFETIME-P3-1** : typiquement 2592000s = 30 jours = `state.config.refresh_token_max_lifetime`, **PAS** `state.config.refresh_inactivity` 15 min sliding — qui est la fenêtre DB `expires_at` étendue à chaque /refresh, distincte de la durée hard ceiling browser cookie ; aligné avec epic-10.md ligne 283 `Max-Age=2592000`).
   - Note `Path=/api/v1/auth` restreint le refresh cookie aux endpoints auth (le browser ne l'enverra pas sur `/api/v1/invoices/*` etc.) — réduit la surface d'attaque cookie leak.

2. **Given** réponse `login` ci-dessus, **When** inspection DevTools `Application > Cookies`, **Then** flag **HttpOnly = ☑** présent sur les 2 cookies, et `document.cookie` exécuté en console JavaScript retourne soit une chaîne vide soit une chaîne ne contenant **pas** les valeurs des tokens.

3. **Given** `POST /api/v1/auth/refresh` (sans body, ou avec body vide `{}`), **When** le browser envoie automatiquement le cookie `kesh_refresh_token` (Path=/api/v1/auth), **Then** le backend lit ce cookie, valide le refresh token via la logique existante (rotation, revocation, sliding expiry), et émet **2 nouveaux `Set-Cookie`** (access + refresh — rotation des deux tokens, refresh re-émis avec `Max-Age=<refresh_token_max_lifetime_seconds>` 30 jours cohérent AC #1). Body de réponse peut rester `{}` ou contenir uniquement `{ "expiresIn": <seconds> }` pour UX refresh proactif.

4. **Given** `POST /api/v1/auth/logout` (sans body), **When** la réponse est émise, **Then** elle contient **2 headers `Set-Cookie`** avec `Max-Age=0` (invalidation immédiate browser-side) : `kesh_access_token=; Max-Age=0; Path=/; HttpOnly; Secure; SameSite=Strict` et `kesh_refresh_token=; Max-Age=0; Path=/api/v1/auth; HttpOnly; Secure; SameSite=Strict`. Le refresh_token correspondant est aussi révoqué en DB (logique Story 1-6 préservée — `refresh_tokens::revoke_by_token` avec reason `"logout"`).

5. **Given** un nouvel endpoint `GET /api/v1/auth/me`, **When** appelé avec le cookie `kesh_access_token` valide, **Then** retourne `200 { "userId": "...", "username": "...", "role": "Admin|...", "expiresIn": <seconds_remaining> }` permettant au frontend de restaurer son état auth sans avoir besoin de lire le JWT (qui n'est plus accessible côté JS). `username` provient d'un lookup DB sur `user_id` (les claims JWT n'incluent pas `username`).

6. **Given** middleware `require_auth` (`crates/kesh-api/src/middleware/auth.rs`), **When** une requête arrive, **Then** le middleware lit le JWT en priorité depuis le **cookie `kesh_access_token`**, et en fallback depuis le header `Authorization: Bearer ...` (pour préserver la compatibilité ascendante avec les tests existants `crates/kesh-api/tests/*_e2e.rs` qui injectent l'Authorization header explicitement, et avec l'éventuel mode API pour automation hors-browser).

### Frontend retrait localStorage (AC #7-12)

7. **Given** `frontend/src/lib/app/stores/auth.svelte.ts`, **When** review post-Story, **Then** **aucun** `window.localStorage.setItem(STORAGE_KEY_ACCESS_TOKEN, ...)` ni `setItem(STORAGE_KEY_REFRESH_TOKEN, ...)` ni `setItem(STORAGE_KEY_EXPIRES_IN, ...)` n'est appelé (Pass 1 F-L-P1-13 : les **3 keys** ACCESS_TOKEN + REFRESH_TOKEN + **EXPIRES_IN** doivent toutes être retirées — `grep -nF "localStorage.setItem" auth.svelte.ts` retourne 0 résultats pour les 3 keys). Les constantes `STORAGE_KEY_*` peuvent rester exportées si d'autres modules y font référence (mais leur valeur n'est jamais écrite ni lue par auth-store post-Story).

8. **Given** `frontend/src/lib/shared/utils/api-client.ts:buildHeaders`, **When** une requête `apiClient.get/post/...` est émise, **Then** **aucun** header `Authorization: Bearer <token>` n'est ajouté (le browser envoie automatiquement le cookie `kesh_access_token` via l'option `credentials: 'include'` sur le `fetch`).

9. **Given** `fetchWithRetry` et `fetchWithTimeout` (`api-client.ts:203,255` — **Pass 4 F-AC9-LINEREF-STALE-P4-6** : lignes mises à jour post-Story 10-3 refactor), **When** appelés, **Then** ils incluent **`credentials: 'include'`** dans les options `fetch()` pour permettre l'envoi automatique des cookies cross-origin (utile en dev local quand le frontend Vite tourne sur `:5173` et l'API sur `:3000` — `SameSite=Strict` empêche le cross-origin, mais `same-site` reste OK).

10. **Given** `auth.svelte.ts:hydrate()` appelée au démarrage de l'app, **When** elle s'exécute, **Then** **au lieu** de lire `localStorage.getItem(STORAGE_KEY_ACCESS_TOKEN)`, elle fait `await fetch('/api/v1/auth/me', { credentials: 'include' })`. Si la réponse est 200 → restauration `authState.currentUser = { userId, role }` + `_expiresIn` depuis le body. Si 401 → état non-auth (utilisateur doit se relogger). Si erreur réseau → silently swallow (état non-auth, comportement actuel préservé).

11. **Given** `auth.svelte.ts:login()`, **When** elle reçoit la réponse `POST /login`, **Then** elle **n'extrait plus** `accessToken` ni `refreshToken` du body (qui peut continuer à les retourner pour rétro-compat, ou ne plus les retourner — choix backend AC #1). Elle lit seulement `expiresIn` pour pile-bumping refresh, et fait éventuellement immediately `await fetch('/api/v1/auth/me')` pour récupérer `userId`, `role`, `username`. Le state `authState.currentUser` est rempli depuis `/me`, pas depuis les claims JWT (qui ne sont plus accessibles).

12. **Given** `auth.svelte.ts:logout()`, **When** appelée, **Then** elle appelle `POST /api/v1/auth/logout` avec `credentials: 'include'` (sans body refresh_token car le backend lit le cookie). Elle clear `_accessToken = null`, `_refreshToken = null`, etc. Le cleanup `localStorage` peut être retiré (les keys ne sont plus écrites donc plus à clean) OU conservé comme defensive cleanup pour les utilisateurs migrant depuis une session pre-Story 10-5 (à documenter dans le code).

### Defense-in-depth CSP + tests XSS (AC #13-15)

13. **Given** middleware HTTP backend (`crates/kesh-api/src/middleware/`), **When** une réponse HTML est émise (typiquement le fallback `ServeDir` qui sert `index.html`), **Then** elle contient un header **`Content-Security-Policy`** restrictif. **Valeur minimale recommandée** (Pass 1 F-AC-P1-3 — vérifié ground-truth que `frontend/build/index.html` contient un script inline d'amorçage SvelteKit, donc `script-src 'self'` strict bloque l'app entière en whitepage) : `default-src 'self'; script-src 'self' 'unsafe-inline'; style-src 'self' 'unsafe-inline'; img-src 'self' data:; font-src 'self' data:; connect-src 'self'; frame-ancestors 'none'; base-uri 'self'; form-action 'self'`. Les directives `'unsafe-inline'` sont tolérées en v0.1 car SvelteKit emit (a) un script inline d'amorçage `Promise.all([import(...)]).then(...)` non-déterministe au build, et (b) des styles scoped inline. Migration vers `'sha256-...'` ou `'nonce-...'` envisageable v0.2 (cf. limitation L2). **Cible** : aucune CSP violation console au boot d'une page authentifiée standard (login, dashboard, contacts, journal-entries, reports), **et** l'app fonctionne normalement (pas de whitepage).

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

17. **Given** la suite de tests E2E `npm run test:e2e`, **When** exécutée, **Then** **0 régression** sur les 91 baselines actuelles (status Story 10-3 post-merge). Les tests qui font `page.evaluate(() => localStorage.getItem('kesh:auth:accessToken'))` (e.g. `frontend/tests/e2e/bank-import-confirms.spec.ts:50-53` `authHeaders` helper) doivent être adaptés pour utiliser `credentials: 'include'` au lieu de lire le token + l'injecter en Authorization header. **Pass 1 F-AH-P1-11 + F-AH-P1-6 scope élargi** : la liste exhaustive inclut OBLIGATOIREMENT `frontend/tests/e2e/helpers/test-state.ts` (modification CENTRALE impactant 27+ spec files via `clearAuthStorage()` et 7 spec files via `authedApiContext()`). Le `grep -rn "localStorage.getItem.*kesh:auth"` retourne seulement 3 fichiers — sous-ensemble du vrai impact. Commandes ground-truth complètes : `grep -rln "authedApiContext\|clearAuthStorage" frontend/tests/e2e/` (27+7 = 34 fichiers impactés via helper cascade).

18. **Given** `CHANGELOG.md` racine (créé Story 10-4), **When** Story 10-5 mergée, **Then** entrée `[0.1.0]` section **Security** (nouvelle sous-section) ajoutée : "**Sécurité durcie** : les tokens d'authentification sont maintenant stockés en cookies `HttpOnly` + `Secure` + `SameSite=Strict`, inaccessibles au JavaScript du navigateur. Élimine la possibilité de vol immédiat des tokens via une faille XSS hypothétique. Nouveau endpoint `GET /api/v1/auth/me` permet au frontend de restaurer l'état utilisateur sans lire les tokens. Headers CSP défensifs ajoutés sur les réponses HTML (defense-in-depth)." **Et** issue [#41](https://github.com/guycorbaz/kesh/issues/41) fermée avec `closes #41` dans le message du commit final (squash-merge sur main).

## Tasks / Subtasks

### T1: Dépendance Cargo `axum-extra` cookie feature + `reqwest` cookies (AC #1-6, AC #15)

- [ ] **T1.1** : ajouter `axum-extra = { version = "0.12", features = ["typed-header", "cookie"] }` à `crates/kesh-api/Cargo.toml` dans `[dependencies]`. **Pass 1 F-L-P1-12** : version `0.12` (et non `0.10`) — compatible `axum ^0.8.4` actuellement dans le projet (ground-truth crates.io). Le crate `axum-extra` fournit `CookieJar` extractor et `PrivateCookieJar` (avec signing si on veut ajouter SHA d'intégrité — pas requis Story 10-5, simple `CookieJar` suffit).
- [ ] **T1.2** : `cargo build -p kesh-api` PASS après ajout (vérification compile).
- [ ] **T1.3** : **Pass 1 F-T1-P1-7** — mettre à jour `crates/kesh-api/Cargo.toml` section `[dev-dependencies]` : ajouter la feature `cookies` à `reqwest` : `reqwest = { version = "0.12", features = ["json", "multipart", "cookies"] }`. Sans cette feature, l'API `reqwest::Client::builder().cookie_store(true).build()` utilisée dans T9.2 ne compile pas (`method not found` ou feature-gated).

### T2: Backend `auth.rs` — Set-Cookie sur login + refresh + logout (AC #1, #3, #4)

- [ ] **T2.1** : refactor `login` handler (`crates/kesh-api/src/routes/auth.rs:85-180`) pour retourner `(CookieJar, Json<LoginResponse>)` (tuple Axum standard avec `IntoResponse`). Construire les 2 cookies via **Pass 1 F-T2-P1-8** API `cookie ^0.18` correcte : `Cookie::build(("kesh_access_token", access_token)).http_only(true).secure(!state.config.test_mode).same_site(SameSite::Strict).path("/").max_age(time::Duration::seconds(state.config.jwt_expiry.num_seconds())).build()` — **noter les parenthèses enveloppantes `("name", "value")` (tuple)** et le `time::Duration` (pas `chrono`). **Pass 3 F-COOKIE-DEV-LOCAL-NO-HTTPS-P3-7 + F-CHANGELOG-DEV-FLAG-P3-8** : `.secure(!state.config.test_mode)` (au lieu de hardcode `true`) permet aux tests E2E Playwright en HTTP local et au dev mode `KESH_TEST_MODE=true` de fonctionner (cookies sans flag Secure tolérés sur localhost HTTP par Chromium/Firefox/Safari). En prod `test_mode=false` → Secure=true (sécurité préservée). L'API à 2 args séparés `Cookie::build("name", "value")` est obsolète (`cookie ^0.17`) et ne compile pas avec `axum-extra 0.12`. **Pass 2 F-REFRESH-EXPIRY-UNIT-P2-10 note unit** : `state.config.jwt_expiry` et `state.config.refresh_token_max_lifetime` sont des `chrono::TimeDelta` (cf. `config.rs:135,137`). Pour convertir en `time::Duration` du cookie crate, utiliser explicitement `.num_seconds()` (returns `i64`) puis `time::Duration::seconds(...)`. Ne PAS confondre avec `.num_minutes()` ou `.num_milliseconds()` qui causeraient un mismatch d'ordre de grandeur. **Pour kesh_refresh_token (Pass 3 F-COOKIE-LIFETIME-P3-1)** : utiliser `max_age(time::Duration::seconds(state.config.refresh_token_max_lifetime.num_seconds()))` (30 jours hard ceiling browser, **PAS** `refresh_inactivity` qui est la sliding window DB côté `expires_at` extended à chaque /refresh). Sans ce patch, le cookie est purgé par le browser après 15 min d'inactivité → user déconnecté silencieusement malgré la sliding logique DB attendant 30 jours d'usage actif. `let jar = jar.add(access_cookie).add(refresh_cookie); Ok((jar, Json(response)))`. Body `LoginResponse` étendu (**D1 + D6 actés**) : `{ access_token, refresh_token, expires_in, user_id, username, role }` — `user_id/username/role` permettent au frontend d'éviter le round-trip `/me` post-login (D6), `access_token/refresh_token` conservés pour rétro-compat tests historiques (D1 Option A).
- [ ] **T2.2** : refactor `refresh` handler (`auth.rs:228-316`) : extraire `jar: CookieJar` du request (Axum extractor), lire `jar.get("kesh_refresh_token").map(|c| c.value())` pour obtenir le refresh_token. Si absent ou vide → fallback lecture body `req.refresh_token` (rétro-compat tests). Émettre nouveaux cookies via la même logique T2.1 (notamment `Max-Age=<refresh_token_max_lifetime>` 30 jours pour refresh cookie, cohérent Pass 3 F-COOKIE-LIFETIME-P3-1). Body `RefreshResponse` conserve aussi `access_token + refresh_token + expires_in` (cohérent D1 Option A).

- [ ] **T2.4** : **Pass 3 F-EXPIRES-IN-CHANGE-PASSWORD-P3-15** — refactor `change_password` handler (`auth.rs:341-407`) : émettre les 2 nouveaux Set-Cookie (access + refresh rotation) cohérent avec `login` T2.1 et `refresh` T2.2. Le handler retourne déjà `Json<RefreshResponse>` (body avec `access_token + refresh_token + expires_in`) mais sans Set-Cookie post-Story 10-5 → le user qui change son password reste sur l'ancien cookie expiré → déconnexion silencieuse au prochain refresh proactif. Patch obligatoire pour ne pas laisser un handler oublié hors du flux cookie.
- [ ] **T2.3** : refactor `logout` handler (`auth.rs:188-196`) : **Pass 1 F-T2-P1-4** — extraire `jar: CookieJar` du request, lire `kesh_refresh_token` en priorité depuis le cookie, fallback sur le body `Option<Json<LogoutRequest>>` si cookie absent. Changer `LogoutRequest.refresh_token` de required à optionnel : soit `Option<String>`, soit transformer la signature en `Json(req): Option<Json<LogoutRequest>>`. Résolution : `let rt = cookie_rt.or_else(|| body_rt);` puis si `Some(rt)` → révocation DB via `refresh_tokens::revoke_by_token(&state.pool, &rt, "logout")` (logique Story 1-6 préservée). Si `None` → émettre quand même les cookies expirés, retourner 204 (logout sans session valide reste idempotent). **Émettre cookies expirés** via `Cookie::build(("kesh_access_token", "")).max_age(time::Duration::seconds(0)).path("/").http_only(true).secure(!state.config.test_mode).same_site(SameSite::Strict).build()` (Set-Cookie avec Max-Age=0 = invalidation immédiate browser-side). **Pass 4 F-SECURE-FLAG-INCONSISTENCY-P4-2** : `.secure(!state.config.test_mode)` cohérent avec T2.1/T2.2/T2.4 — sinon en `test_mode=true` (CI Playwright HTTP localhost), le browser ignorerait le Set-Cookie avec `Secure` flag → cookie d'access_token persisterait silencieusement → test `logout_invalidates_cookie` (AC #4 / T9.1) faussement vert. Tests existants `logout_revokes_refresh_token` + `logout_idempotent` + `logout_unknown_token_returns_204` (`auth_e2e.rs:606,647,677` envoient `refreshToken` dans le body JSON) restent verts grâce au fallback body.

### T3: Backend `middleware/auth.rs` — cookie-first avec fallback header (AC #6)

- [ ] **T3.1** : modifier `require_auth` (`crates/kesh-api/src/middleware/auth.rs:61`) pour extraire `jar: CookieJar` en plus de `request: Request`. Lire `jar.get("kesh_access_token").map(|c| c.value())` en priorité. Si absent → fallback sur le code actuel `Authorization: Bearer ...` (lignes 73-75+). Le reste de la logique (decode JWT, validate exp, attacher `CurrentUser` au request) reste inchangée.
- [ ] **T3.2** : **Pass 1 F-L-P1-15** — adapter les tests inline dans `crates/kesh-api/src/middleware/auth.rs` (bloc `#[cfg(test)] mod tests` lignes 106-324, **pas** dans `tests/middleware/auth.rs` qui n'existe pas) : ajouter au moins 2 nouveaux tests : (a) `require_auth_accepts_cookie_no_authorization` et (b) `require_auth_prefers_cookie_over_header_when_both_present`. Les tests existants `require_auth_accepts_valid_authorization` doivent rester verts (fallback header préservé).

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
  **Pass 1 F-L-P1-14 ground-truth** : `CurrentUser` (`middleware/auth.rs:29`) contient actuellement `user_id`, `role`, `company_id` — **PAS de `exp`**. Donc T4.1 doit obligatoirement (a) ajouter `pub exp: i64` à la struct `CurrentUser` ligne 29, et (b) l'initialiser lors de l'extraction JWT dans `require_auth` (ligne ~85) depuis `claims.exp` (cf. `crates/kesh-api/src/auth/jwt.rs` Claims struct). Sans cette modification, le handler `me` ne compile pas (accès à `current_user.exp` inexistant).
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
              "default-src 'self'; script-src 'self' 'unsafe-inline'; style-src 'self' 'unsafe-inline'; img-src 'self' data:; font-src 'self' data:; connect-src 'self'; frame-ancestors 'none'; base-uri 'self'; form-action 'self'".parse().unwrap(),  // Pass 4 F-CSP-CODE-INCONSISTENCY-P4-1 — script-src 'unsafe-inline' obligatoire (D3 acté), sinon SvelteKit inline script bloqué → whitepage
          );
      }
      response
  }
  ```
- [ ] **T5.2** : monter le middleware dans `crates/kesh-api/src/lib.rs`. **Pass 3 F-CSP-API-ATTACK-SURFACE-P3-3 architectural Axum 0.8 gotcha** : le layer DOIT être appliqué **APRÈS** `.fallback_service(fallback)` ligne 432, sinon il n'enveloppe PAS le `ServeDir` fallback (en Axum 0.7+, un `.layer()` AVANT `.fallback_service()` ne traverse pas le fallback qui est un `Service` séparé). Pattern correct :
  ```rust
  main_router
      .fallback_service(fallback)
      .layer(axum::middleware::from_fn(crate::middleware::csp::csp_html))
      .with_state(state)
  ```
  Cela enveloppe TOUTES les réponses (routes API + ServeDir fallback). Le filter `content_type.starts_with("text/html")` dans `csp_html` empêche le CSP de s'appliquer sur les réponses JSON `/api/v1/*` (inoffensif). Sans le `.layer()` APRÈS `fallback_service`, le middleware n'enveloppe pas le ServeDir → AC #13 silencieusement faux (header CSP absent sur `/login`, `/`, etc.).
- [ ] **T5.3** : validation manuelle renforcée Pass 3 F-CSP-API-ATTACK-SURFACE-P3-3 :
  - `curl -I http://localhost:3000/` (root → index.html via ServeDir fallback) → header `Content-Security-Policy: ...` **présent** (vérifie que le layer enveloppe bien ServeDir, pas juste les routes Axum natives).
  - `curl -I http://localhost:3000/login` → CSP présent (path géré côté SPA via ServeDir fallback to index.html).
  - `curl -I http://localhost:3000/_app/immutable/start-XXXX.js` (asset statique) → header `Content-Type` est `application/javascript` ou similaire, donc CSP **absent** (filtre content-type correct).
  - `curl -I http://localhost:3000/api/v1/auth/me` (sans auth) → 401 JSON, header CSP **absent** (filtre OK).
  - `curl -I http://localhost:3000/inexistant` (404 Axum default) → text/plain, CSP **absent** (acceptable, pages d'erreur n'ont pas de JS à protéger).
- [ ] **T5.4** : optionnel — valider via DevTools console qu'aucune CSP violation ne fire au boot d'une page SvelteKit normale (login, dashboard, etc.). Si SvelteKit emit des styles inline qui violent `'unsafe-inline'` ou des scripts inline qui violent `'self'`, ajuster la directive (e.g. `style-src 'self' 'unsafe-inline'` est déjà tolérant ; `script-src 'self'` strict — si violation, considérer `'nonce-...'` ou `'sha256-...'`).

### T6: Frontend `auth.svelte.ts` — retrait localStorage + consumer `/me` (AC #7, #10, #11, #12)

- [ ] **T6.1** : retirer les 3 `window.localStorage.setItem(STORAGE_KEY_*, ...)` dans `login()` (lignes 82-86). Le `_accessToken`, `_refreshToken`, `_expiresIn` `$state` peuvent rester (utiles pour `isAuthenticated` getter et refresh proactif), mais NE plus persister en localStorage.
- [ ] **T6.2** : retirer les 3 `window.localStorage.removeItem(STORAGE_KEY_*, ...)` dans `clearSession()` (lignes 100-104) ET dans `logout()` (lignes 126-130) — ou les laisser comme defensive cleanup migration (cf. AC #12) avec un commentaire `// Pass 10-5 : defensive cleanup pour utilisateurs migrant depuis localStorage`.
- [ ] **T6.3** : refactor `hydrate()` (lignes 140-185) : **remplacer entièrement** la logique `localStorage.getItem(...)` par `await fetch('/api/v1/auth/me', { credentials: 'include' })`. Si réponse 200 → parse body, set `_currentUser = { userId: response.userId, role: response.role }`, `_expiresIn = response.expiresIn` (cohérent **D5 acté** ci-dessous : `_accessToken` reste `null` post-Story 10-5 car le JWT est dans le cookie HttpOnly inaccessible côté JS, et `isAuthenticated` getter T6.5 dépendra de `_currentUser !== null`). Si réponse 401 → state non-auth (`_currentUser` reste `null`). Si réseau fail → catch silencieux, state non-auth. **Pass 1 F-AH-P1-1 critique** : `hydrate()` devient une fonction `async` (return `Promise<void>`). Son appelant `frontend/src/hooks.client.ts:17` (`try { authState.hydrate(); } catch...`) DOIT être adapté pour `await` la promesse — sinon `load()` functions s'exécutent avant que `_currentUser` soit peuplé, et `+layout.ts:10` (`if (browser && !authState.isAuthenticated) throw redirect(302, '/login')`) redirige systématiquement vers `/login` pour tous les utilisateurs authentifiés (régression UX catastrophique). **Patch obligatoire** dans T6.3.bis ci-dessous.
- [ ] **T6.3bis** : **Pass 1 F-AH-P1-1** — modifier `frontend/src/hooks.client.ts` : remplacer le bloc actuel (ligne ~17) `try { authState.hydrate(); } catch (e) { console.error(...); }` par le pattern SvelteKit `init` hook async :
  ```typescript
  export const init = async () => {
      try {
          await authState.hydrate();
      } catch (e) {
          console.error('[auth] Hydration failed:', e);
      }
  };
  ```
  Le hook `init` est garanti exécuté par SvelteKit AVANT toutes les `load()` functions. Sans ce changement, la régression "redirect /login systématique" est garantie. Vérifier la version SvelteKit utilisée : `init` hook async supporté depuis SvelteKit 2.x (déjà en place per `frontend/package.json`).
- [ ] **T6.4** : adapter `login()` : après le `POST /login`, le body de réponse contient désormais `{ access_token, refresh_token, expires_in, user_id, username, role }` (D1 + **D6 actés Pass 2 F-E2E-LOGIN-RESPONSE-USERID-P2-9**). **Pass 3 F-LOGIN-PAGE-DECODES-JWT-P3-4 — refactor signature obligatoire** : changer la signature de `authState.login()` de `(accessToken: string, refreshToken: string, expiresIn: number)` à `(payload: { userId: string; username: string; role: string; expiresIn: number })`. Set directement `_currentUser = { userId: payload.userId, role: payload.role }` + `_expiresIn = payload.expiresIn` depuis le body — **pas de second round-trip `/me` nécessaire**. **Supprimer le décodage JWT côté JS** (`decodeJwtPayload` lignes 32-51 devient mort code post-Story 10-5, retirer aussi). Le navigateur a déjà set les cookies HttpOnly via les headers Set-Cookie (T2.1) avant que `login()` lise le body. Si la session est restorée au boot (pas après un login), c'est `hydrate()` (T6.3) qui appelle `/me` à la place.

- [ ] **T6.4bis** : **Pass 3 F-LOGIN-PAGE-DECODES-JWT-P3-4 caller adaptation** — adapter `frontend/src/routes/login/+page.svelte:33` qui appelle actuellement `authState.login(data.accessToken, data.refreshToken, data.expiresIn)`. Post-Story 10-5 : `authState.login({ userId: data.userId.toString(), username: data.username, role: data.role, expiresIn: data.expiresIn })`. Les tokens `data.accessToken`/`data.refreshToken` du body restent disponibles (D1 Option A pour rétro-compat tests) mais sont **ignorés** par le frontend (cookies HttpOnly déjà set par browser). Sans cette adaptation, l'ancien `authState.login(accessToken, ...)` continue à décoder le JWT côté JS, contradiction silencieuse D5 (`_currentUser` set depuis JWT decode au lieu du body) + D6 (les champs `user_id/username/role` du body ignorés).
- [ ] **T6.5** : **Pass 1 F-AC-P1-9 D5 acté** — `isAuthenticated` getter (`auth.svelte.ts:70-72`) : changer la dépendance de `_accessToken !== null` à `_currentUser !== null`. Pendant qu'on y est, **supprimer aussi le getter public `accessToken`** (lignes 58-60) ou le faire retourner toujours `null` (post-Story 10-5 il n'est plus disponible en cookie scenario). Adapter `frontend/src/lib/shared/utils/api-client.ts:141` (guard `authState.accessToken` actuel) — retirer entièrement (T7.2). Adapter `frontend/src/routes/+layout.svelte:49` (`authState.accessToken !== null`) → `authState.isAuthenticated` (T7.4). Adapter `frontend/src/lib/app/stores/auth.svelte.test.ts` (si existe — lignes 23,34,88,105 typiquement assertent `expect(authState.accessToken).toBe(...)` — ces tests doivent être réécrits pour asserter `authState.currentUser` ou `authState.isAuthenticated`). Liste exhaustive des consumers `authState.accessToken` à identifier via `grep -rn "authState.accessToken\|\\.accessToken" frontend/src/`.
- [ ] **T6.6** : `_hydrated` guard (ligne 142) reste utile pour éviter les double-fetches concurrents `/me`.
- [ ] **T6.7** : **Pass 4 F-TASK-MISSING-UPDATEEXPIRESIN-P4-4** — ajouter `updateExpiresIn(expiresIn: number)` dans `authState` (`auth.svelte.ts`) : méthode minimale `_expiresIn = expiresIn;` appelée par `doRefresh()` post-refresh réussi (T7.3d) pour bumper la fenêtre de refresh proactif sans toucher `_currentUser` (qui reste inchangé puisque même utilisateur, juste JWT refreshed).

### T7: Frontend `api-client.ts` — credentials include + retrait Authorization header (AC #8, #9)

- [ ] **T7.1** : modifier `fetchWithTimeout` (`frontend/src/lib/shared/utils/api-client.ts:192-199`) pour ajouter `credentials: 'include'` à l'objet options passé à `fetch()` : `return await fetch(url, { ...init, credentials: 'include', signal: controller.signal })`.
- [ ] **T7.2** : modifier `buildHeaders` (`api-client.ts:124-146`) — retirer entièrement le bloc lignes 141-143 qui ajoute `Authorization: Bearer ${authState.accessToken}`. **Garder** la logique `AUTH_EXCLUDED_URLS` (login/logout/refresh) pour information, mais comme aucun header Authorization n'est jamais ajouté, son rôle change : elle ne sert plus à rien pour le frontend post-Story 10-5. On peut soit la retirer entièrement (cleanup), soit la laisser commentée pour traçabilité. **Recommandation** : retirer pour clarté.
- [ ] **T7.3** : modifier `api-client.ts:doRefresh` (lignes 57-104) — modifications complètes Pass 1 F-AH-P1-2 + F-T7-P1-5 :
  - **(a)** Supprimer la garde lignes 58-63 (`const currentRefreshToken = authState.refreshToken; if (!currentRefreshToken) { authState.clearSession(); window.location.replace('/login?reason=session_expired'); return false; }`) — post-Story 10-5, `_refreshToken` est toujours `null` (T6.1 supprime le setItem) donc la garde déclencherait clearSession + redirect login systématiquement sur chaque 401. Le refresh via cookie ne serait jamais tenté. **Sans ce patch**, l'utilisateur est déconnecté toutes les 15 minutes (expiration JWT).
  - **(b)** Modifier le `fetch('/api/v1/auth/refresh', ...)` lignes 67-71 — retirer le body JSON `{refreshToken: ...}` (le browser envoie le cookie automatiquement). Garder `method: 'POST'`, `credentials: 'include'` (ajouté T7.1), pas de body (ou body vide `''`).
  - **(c)** Adapter la validation de la réponse lignes 87-96 selon **D1 Option A acté T2.1** : si `RefreshResponse` body conserve `accessToken + refreshToken + expiresIn` pour rétro-compat tests, la validation actuelle (typeof checks) reste valide. **Mais Pass 3 F-LOGIN-PAGE-DECODES-JWT-P3-4** : le `RefreshResponse` ne contient PAS `user_id/username/role` (ces champs sont uniquement dans `LoginResponse` extended D6). Donc post-refresh, on **ne peut pas** appeler la nouvelle signature `authState.login({ userId, username, role, expiresIn })`. **Solution** : ne PAS appeler `authState.login()` du tout après un refresh réussi — juste update `_expiresIn = data.expiresIn` directement (les cookies sont déjà rotation-set par le browser via Set-Cookie de T2.2, `_currentUser` reste inchangé puisque c'est le même utilisateur, juste un JWT refreshed). Documenter explicitement dans T7.3(c).
  - **(d)** **Pass 3 F-LOGIN-PAGE-DECODES-JWT-P3-4 sub-d** : retirer le call `authState.login(data.accessToken, data.refreshToken, data.expiresIn)` ligne 97 actuelle — il décodait le JWT côté JS via `decodeJwtPayload` (anti-pattern D5). Remplacer par : `authState.updateExpiresIn(data.expiresIn)` (nouvelle méthode minimale dans le store qui set `_expiresIn` sans toucher à `_currentUser`). Ajouter cette méthode dans `auth.svelte.ts` (T6.x).
- [ ] **T7.4** : adapter `+layout.svelte` onMount (`frontend/src/routes/+layout.svelte:30-51`) — l'auth gate `if (authState.accessToken !== null)` (ligne 50) doit être adaptée pour `if (authState.isAuthenticated)` (cohérent avec T6.5) puisque `_accessToken` n'est plus défini en mode cookie.

### T8: Frontend tests Vitest — adaptation (AC #7, #11)

- [ ] **T8.1** : `frontend/src/lib/app/stores/auth.svelte.test.ts` (**Pass 3 F-AUTH-SVELTE-TEST-RECONNECT-P3-9 ground-truth : le fichier EXISTE**, contrairement à la formule conditionnelle initiale) — refactor MASSIF requis : (a) 6+ assertions `expect(authState.accessToken).toBe(...)` à supprimer ou réécrire vers `authState.currentUser` (D5 acté : getter `accessToken` retiré ou nullified). (b) Les tests qui validaient le décodage JWT côté JS via `decodeJwtPayload` (typiquement lignes 40-90) deviennent **obsolètes** post-Pass 3 F-LOGIN-PAGE-DECODES-JWT-P3-4 (le décodage JWT est retiré du store). (c) Réécrire la suite pour mocker `fetch('/api/v1/auth/me')` via `vi.stubGlobal('fetch', vi.fn().mockResolvedValue(mockResponse(200, { userId: '42', username: 'alice', role: 'Admin', expiresIn: 900 })))` et asseroitr que `hydrate()` peuple `_currentUser` depuis le body. Adapter aussi la nouvelle signature `authState.login({ userId, username, role, expiresIn })` (sans tokens). Effort estimé : ~30-50 lignes net (vs flou « adaptation » initial).
- [ ] **T8.2** : `frontend/src/lib/shared/utils/api-client.test.ts` — adapter les tests existants si certains injectent `authState.accessToken` directement (via `authState.login(...)` ligne ~556) — vérifier que `mockFetch` reçoit bien `credentials: 'include'` dans les options.

### T9: Test intégration backend `tests/auth_cookies_e2e.rs` (AC #15)

- [ ] **T9.1** : créer `crates/kesh-api/tests/auth_cookies_e2e.rs` (nouveau fichier ~150-200 lignes) avec 4 tests `#[sqlx::test(migrator = "kesh_db::MIGRATOR")]` :
  - **`login_sets_two_httponly_cookies`** (AC #1) : POST /login → assertions sur les headers `Set-Cookie` (pattern `assert!(set_cookie_header.contains("HttpOnly")); assert!(set_cookie_header.contains("SameSite=Strict")); ...`). **Pass 4 F-TEST-SECURE-ASSERTION-P4-5 note** : le flag `Secure` sera **absent** des cookies en mode test (`.secure(!test_mode)` → false en CI Playwright HTTP). Ne PAS asserter `contains("Secure")` dans ces tests d'intégration backend — l'assertion `Secure` est réservée aux tests E2E Playwright T10 (qui tournent en browser context et peuvent inspecter le flag via DevTools Cookies inspector ou Playwright `page.context().cookies()`).
  - **`authenticated_request_with_cookie_only`** (AC #6) : login → extract cookies → request `/api/v1/companies/current` avec cookie seul (sans header Authorization) → 200 OK.
  - **`authenticated_request_with_authorization_only`** (AC #6 fallback) : login → extract access_token from body OR via cookie parsing → request avec `Authorization: Bearer ...` sans cookie → 200 OK.
  - **`logout_invalidates_cookie`** (AC #4) : login → logout → assertions sur Set-Cookie `Max-Age=0` + DB check `refresh_tokens.revoked_at IS NOT NULL`.
- [ ] **T9.2** : pattern test reqwest comme `i18n_e2e.rs` ou `health_endpoint.rs` (les 19+ tests intégration `tests/*_e2e.rs`). **Pass 1 F-T9-P1-10** : créer un helper distinct **`spawn_app_with_cookie_jar(pool)`** dans `auth_cookies_e2e.rs` (local au fichier, pas dans `common/`) qui retourne un `TestApp` avec `client: reqwest::Client::builder().cookie_store(true).build().unwrap()`. **NE PAS modifier** `spawn_app()` global dans `auth_e2e.rs` ou `tests/common/` — risque de breaker l'isolation de session des 19+ tests existants qui utilisent `Authorization: Bearer` explicite (les Set-Cookie du login seraient maintenant capturés et renvoyés automatiquement sur les requêtes subséquentes, changeant le comportement). Le helper local évite cette interférence cross-tests. Nécessite la feature `cookies` de `reqwest` activée par T1.3.

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
- [ ] **T12.3** : E2E Playwright : `npm run test:e2e -- xss-token-protection.spec.ts` 3/3 PASS, **et** `npm run test:e2e` suite complète → 0 régression sur les 91 baselines actuelles (Story 10-3 état). Tests `bank-import-confirms.spec.ts:50-53` (`authHeaders` helper) et autres consumers `localStorage.getItem('kesh:auth:accessToken')` à adapter (cf. AC #17 et T8.2). Liste exhaustive à identifier via : `grep -rln "authedApiContext\|clearAuthStorage" frontend/tests/e2e/` (la commande `localStorage.getItem` seule sous-estime — voir F-AH-P1-11).
- [ ] **T12.3.a** : **Pass 1 F-AH-P1-6 + F-AH-P1-11 + Pass 3 F-PLAYWRIGHT-COOKIE-CROSS-CONTEXT-P3-2** — refactorer le helper central `frontend/tests/e2e/helpers/test-state.ts` (modification CENTRALE impactant 27+ spec files via `clearAuthStorage()` et 7 spec files via `authedApiContext()`) :
  - **`authedApiContext(page)`** (ligne ~150) lit actuellement `localStorage.getItem('kesh:auth:accessToken')` et l'injecte en `Authorization: Bearer`. **Pass 3 architectural gotcha Playwright** : `playwrightRequest.newContext()` actuellement utilisé NE partage PAS le cookie jar HttpOnly du browser context (il crée un `APIRequestContext` complètement isolé). 2 options post-Story 10-5 :
    - **Option (a-i) simple mais piège dispose** : retourner directement `page.request` (l'`APIRequestContext` scoped au page browser context, qui inclut les cookies HttpOnly). MAIS : les 7 callers utilisent un pattern `try/finally { ...; disposeContextSafe(ctx); }` — appeler `.dispose()` sur `page.request` invaliderait le request context du page entier (crash tests subsequents). Donc **modifier le pattern caller** : retirer `disposeContextSafe(ctx)` partout.
    - **Option (a-ii) recommandée — preserve dispose-safe pattern** : conserver `playwrightRequest.newContext()` MAIS passer `storageState: await page.context().storageState()` pour cloner le cookie jar du browser context dans le nouveau APIRequestContext isolé. `dispose()` reste safe (clos uniquement le context cloné). Change minimal de signature, callers existants (7 spec files) intacts. **Choix recommandé : (a-ii)**.
  - **Implémentation (a-ii) `authedApiContext(page)`** :
    ```typescript
    export async function authedApiContext(page: Page): Promise<APIRequestContext> {
        const storageState = await page.context().storageState(); // ← cookies HttpOnly inclus
        return playwrightRequest.newContext({
            baseURL: resolveBackendUrl(),
            storageState, // ← clone cookie jar (browser → API request)
            // PAS de extraHTTPHeaders Authorization Bearer — les cookies sont dans storageState
        });
    }
    ```
  - **`readAccessTokenFromStorage(page)`** (helper interne) : retirer ou marquer dépréqué (puisque le token n'est plus en localStorage). Les 3 spec files qui le consomment via `authedApiContext` ne le voient plus.
  - **`clearAuthStorage(page)`** : remplacer le clear localStorage par `page.context().clearCookies()` (pour invalider le cookie HttpOnly entre tests). Impact sur 27+ spec files qui l'invoquent en `afterEach`.
  - **Test unitaire Vitest** `frontend/tests/e2e/helpers/test-state.test.ts:71` qui assert "throw 'no accessToken in localStorage'" devient obsolète → supprimer ou réécrire pour asserter "throw 'no cookie kesh_access_token in browser context'" (cohérent D5 acté).
- [ ] **T12.4** : commit unique (ou commits par task selon préférence) sur branche `story/10-5-httponly-tokens-security`. Status sprint-status `10-5-httponly-tokens-security: ready-for-dev → in-progress` avant 1er commit puis `in-progress → review` au push de fin de dev-story. Bump status `10-4 review → done` au démarrage de Story 10-5 (pattern `feedback_avoid_parallel_prs`).
- [ ] **T12.5** : message du commit final inclut `closes #41` pour fermer automatiquement l'issue GitHub [KF-002] à la fusion sur main (AC #18).

## Dev Notes

### Architecture patterns à respecter

- **Pattern Axum CookieJar** : utiliser l'extractor `axum_extra::extract::CookieJar` (re-export depuis `axum-extra`) avec la signature `async fn handler(jar: CookieJar, ...) -> impl IntoResponse`. Retourner `(jar, body)` tuple pour émettre les Set-Cookie headers. Référence canonique : [axum-extra cookie docs](https://docs.rs/axum-extra/latest/axum_extra/extract/cookie/index.html).
- **Pattern Cookie Builder** : `Cookie::build(("name", "value")).http_only(true).secure(!state.config.test_mode).same_site(SameSite::Strict).path("/").max_age(time::Duration::seconds(N)).build()`. Note : `axum-extra` utilise le crate `cookie` qui dépend de `time` (pas `chrono`) pour `Duration` — attention à l'unit mismatch. **Pass 4 F-SECURE-FLAG-INCONSISTENCY-P4-2** : `.secure(!state.config.test_mode)` cohérent T2.1/T2.2/T2.3/T2.4 — `Secure=true` hardcodé bloquerait les cookies sur HTTP local en CI test_mode.
- **Pattern middleware Axum** : `async fn middleware_fn(request: Request, next: Next) -> Response` monté via `.layer(axum::middleware::from_fn(middleware_fn))`. Référence : `crates/kesh-api/src/middleware/auth.rs` (require_auth pattern).
- **Pattern test intégration** : `#[sqlx::test(migrator = "kesh_db::MIGRATOR")]` + `spawn_app(pool).await` + `reqwest::Client::builder().cookie_store(true).build()` pour gérer les cookies. Cohérent avec les 19+ tests `tests/*_e2e.rs`.

### Intelligence Story 10-4 (dernière story Epic 10 livrée)

- **Branche actuelle main** : `d250672` post-merge PR #111 Story 10-4.
- **Sprint-status** : `10-4-...: review` (sera bumpé `done` au démarrage 10-5 dans le 1er commit), `10-5-...: backlog → ready-for-dev` après création de cette spec.
- **CHANGELOG.md** : existe (Story 10-4), contient `[0.1.0]` avec section `Multi-utilisateurs et sécurité` — Story 10-5 doit ajouter une sous-section dédiée `Sécurité` ou enrichir la section existante.
- **Manuel admin** : `docs/manual/fr/admin-manual.tex` 204+ KB PDF, §"Sécurité" existe (§"Authentification JWT + refresh tokens") à enrichir T11.2.
- **Pattern code-review** : 4-pass adversarial Sonnet → Haiku → Opus → Sonnet (cf. CLAUDE.md §"Review Iteration Rule"). Sur Story 10-3, Pass 3 Opus a catché 3 HIGH architecturaux ratés par Sonnet+Haiku — anticiper le même type de findings pour Story 10-5 (race conditions cookie/header dans middleware fallback, scope path cookie trop large/étroit, CSP violations imprévues côté SvelteKit hydration, etc.).

### Décisions ouvertes à clarifier lors de l'implémentation

- **D1 — `LoginResponse` body : tokens en clair ou retirés** : **ACTÉ Option A (Pass 1 F-T7-P1-5)** — tokens conservés en body (`accessToken + refreshToken + expiresIn`) pour rétro-compat tests 19+ `*_e2e.rs` + simplicité validation `doRefresh()` `api-client.ts:87-96` (validation actuelle reste valide). Le passage à Option B (body sans tokens) serait propre mais nécessite refactor non-trivial des 19+ tests + nouvelle fonction `authState.setAuthenticated(expiresIn)` distincte de `login(...)`. Reporté v0.2.
- **D2 — `CurrentUser` middleware structure** : **ACTÉ — ajout obligatoire** (Pass 1 F-L-P1-14) : ajouter `pub exp: i64` à `CurrentUser` (`middleware/auth.rs:29`) initialisé depuis `claims.exp` lors de l'extraction JWT dans `require_auth`. Sans, le handler `me` (T4.1) ne compile pas.
- **D3 — CSP `script-src` strict vs `'unsafe-inline'`** : **ACTÉ `'unsafe-inline'` v0.1 (Pass 1 F-AC-P1-3)** — vérifié ground-truth que SvelteKit emit un `<script>` inline d'amorçage dans `frontend/build/index.html` (Promise.all([import(...)]).then(...)). `script-src 'self'` strict bloquerait l'app en whitepage. Migration vers `'sha256-...'` ou `'nonce-...'` reportée v0.2 (limitation L2).
- **D4 — Tests E2E adaptation** : **ACTÉ Option (a-ii) après Pass 3 F-PLAYWRIGHT-COOKIE-CROSS-CONTEXT-P3-2** — refactor du helper central `frontend/tests/e2e/helpers/test-state.ts` (impact cascade 27+ spec files via `clearAuthStorage` + 7 via `authedApiContext`). **Conserver `playwrightRequest.newContext()`** MAIS passer `storageState: await page.context().storageState()` pour cloner cookie jar du browser context dans un APIRequestContext isolé — dispose-safe et préserve le pattern try/finally des 7 callers. Option (a-i) `page.request` direct rejetée car appel `.dispose()` casserait le request context partagé du page. Voir T12.3.a pour snippet implémentation.
- **D5 — `isAuthenticated` getter (Pass 1 F-AC-P1-9)** : **ACTÉ** — convertir `isAuthenticated` getter pour dépendre de `_currentUser !== null` (pas `_accessToken !== null` qui restera `null` post-Story 10-5 en cookie scenario). Supprimer aussi le getter public `accessToken` (le JWT est inaccessible côté JS). Adaptation des consumers : `api-client.ts:141` (retirer guard), `+layout.svelte:49` (utiliser `isAuthenticated`), tests `auth.svelte.test.ts` (réécrire `expect(authState.accessToken)` → `expect(authState.currentUser)`). Voir T6.5.

- **D6 — `LoginResponse` body : ajouter `userId` + `username` + `role` pour éviter round-trip `/me` post-login (Pass 2 F-E2E-LOGIN-RESPONSE-USERID-P2-9)** : **ACTÉ** — étendre `LoginResponse` struct (`auth.rs:46-52`) pour inclure `user_id: i64`, `username: String`, `role: String` (en plus de `access_token + refresh_token + expires_in` conservés via D1 Option A). Le frontend `auth.svelte.ts:login()` peut directement set `_currentUser` depuis la réponse `/login` sans déclencher un second fetch `/api/v1/auth/me` (qui serait redondant). `/me` reste utile uniquement pour `hydrate()` au boot (quand on n'a pas la réponse `/login` en mémoire). Évite 1 round-trip ↔ 1 ms latence × N users + 1 query DB users (déjà faite dans login pour Argon2 verify). Économie réelle. Adapter T6.4 : retirer le "second fetch /me après login" — utiliser les champs `userId/username/role` du body de réponse directement.

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
- **Pass 1 F-L-P1-16 nuance Path** : `Path=/api/v1/auth` est un **préfixe** (browser cookie path matching), le refresh_token est donc aussi transmis aux handlers `/api/v1/auth/me` et `/api/v1/auth/logout`. Comportement normal — les handlers `/me` et `/logout` ignorent ou consomment le refresh_token selon leur logique. Pour une restriction exacte sur `/api/v1/auth/refresh` uniquement, il faudrait `Path=/api/v1/auth/refresh` strict, mais alors `/logout` n'aurait plus accès au cookie pour la révocation DB. Compromis choisi : `Path=/api/v1/auth` préfixe pour couvrir refresh + logout + (accidentellement) me.
- `Secure` : envoyé uniquement sur HTTPS. **En dev local** (Vite `:5173` + kesh-api `:3000` en HTTP), Chrome/Firefox autorisent `Secure` cookies sur `localhost` (exception). En prod, le reverse proxy HTTPS (Nginx/Caddy/Traefik/Synology Portail — cf. Story 10-4 §4.4/§4.5) doit terminer TLS pour que les cookies soient acceptés.

### Pass 3 Opus — edge cases LOW acceptés v0.1 (notes consolidées)

Findings LOW Pass 3 documentés comme **acceptés v0.1** sans modification de code Story 10-5 (à reconsidérer v0.2 si scope croît) :

- **F-AC-TRACEABILITY-GAP-P3-5** : AC #18 « `closes #41` dans commit final » est une discipline humaine pre-push sans test/hook. Sanity check manuel pre-push : `git log -1 --format=%B | grep -q "closes #41" || echo "WARNING: closes #41 absent du HEAD commit message"`. Acceptable v0.1 — pattern récurrent stories Kesh.
- **F-CSP-MIDDLEWARE-ALL-RESPONSES-P3-6** : filtre `content_type.starts_with("text/html")` est case-sensitive (Rust) mais tower-http émet en lowercase. Si une future version tower-http capitalise (`Text/Html`), le filtre échoue silencieusement. Acceptable v0.1.
- **F-COOKIE-NAME-HOST-PREFIX-P3-10** : préfixe `__Host-kesh_access_token` (force Secure + no Domain + Path=/) défense in-depth contre sub-domain cookie tossing. Non adopté v0.1 (Kesh single-user pas exposé sub-domain risque). `kesh_refresh_token` Path=/api/v1/auth incompatible `__Host-` (qui exige Path=/). **Limitation L6 v0.2-milestone** à créer en KF GitHub si déploiement multi-tenant/sub-domain envisagé v0.2.
- **F-DUMMY-VERIFY-LOGIN-TIMING-P3-11** : D6 (`LoginResponse` body étendu avec username variable length) théoriquement expose un length-side-channel via TLS frame size. Très théorique (TLS padding + HTTP/2 framing brouillent), pas vector v0.1 single-user. Si paranoïa v0.2 : revert D6 → `/login` body length-fixed + frontend appelle `/me` post-login.
- **F-CSP-FRAME-ANCESTORS-NONE-IFRAME-P3-12** : `frame-ancestors 'none'` interdit toute incrustation iframe — si une Story future ajoute preview PDF via iframe (`<embed>` ou `<iframe>`), basculer vers `'self'`.
- **F-CSRF-DEFENSE-DEPTH-P3-13** : L3 (CSRF protection v0.2) — créer une issue GitHub `[CR-XXX] CSRF token defense-in-depth` avec labels `enhancement` + `v0.2-milestone` au merge de Story 10-5 (cohérent CLAUDE.md §"Tech debt management — zero carry-forward policy").
- **F-COOKIE-CRATE-VERSION-PIN-P3-14** : `axum-extra = "0.12"` Cargo semver autorise minor uplift qui peut bumper `cookie` crate transitive de `^0.18` à `^0.19` (breaking API). Vérification post-`cargo build` : `cargo tree -p kesh-api -i cookie` doit retourner `cookie v0.18.x`. Si Cargo.lock change la version au prochain `cargo update`, re-vérifier l'API `Cookie::build(("name", "value"))` syntax.

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
| `frontend/src/lib/app/stores/auth.svelte.ts` | M | +20/-30 lignes (retrait localStorage 3 keys, refactor hydrate via /me, login adapté, isAuthenticated dépend currentUser, accessToken getter retiré) |
| `frontend/src/hooks.client.ts` | M | **Pass 1 F-AH-P1-1** : remplacer `try { authState.hydrate(); }` par `export const init = async () => { await authState.hydrate(); }` (pattern SvelteKit init hook — critique pour éviter régression redirect /login systématique) |
| `frontend/src/lib/shared/utils/api-client.ts` | M | +5/-15 lignes (credentials include, retrait Authorization header builder, retrait guard doRefresh `!currentRefreshToken`, retrait body refresh) |
| `frontend/src/lib/shared/utils/api-client.test.ts` | M | adaptation tests existants |
| `frontend/src/lib/app/stores/auth.svelte.test.ts` | M (si existe) | adaptation tests `expect(authState.accessToken).toBe(...)` → `expect(authState.currentUser).toEqual(...)` |
| `frontend/src/routes/+layout.svelte` | M | +1 ligne (isAuthenticated au lieu de accessToken) |
| `frontend/tests/e2e/security/xss-token-protection.spec.ts` | A | ~100 lignes (3 scénarios) |
| `frontend/tests/e2e/helpers/test-state.ts` | M | **Pass 1 F-AH-P1-6 CENTRAL** : refactor `authedApiContext` + `clearAuthStorage` + `readAccessTokenFromStorage` — impact cascade 27+ spec files via `clearAuthStorage` + 7 via `authedApiContext` |
| `frontend/tests/e2e/bank-import*.spec.ts` + ~25 autres spec files | M | adaptation transparente via helpers `test-state.ts` (la majorité passe sans modif individuelle), sauf ceux qui ont leur propre `authHeaders` inline (`bank-import.spec.ts`, `bank-import-confirms.spec.ts`, `bank-account-journal-link.spec.ts`) |
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
