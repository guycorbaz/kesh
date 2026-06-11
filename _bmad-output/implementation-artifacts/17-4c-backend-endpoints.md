# Story 17.4c: Endpoints backend publics recovery (forgot-password / reset-password)

Status: review

<!-- Extraite de la spec parente UMBRELLA 17-4 (`17-4-recovery-mot-de-passe.md`), validate CONVERGÉ 6 passes (trend > LOW 12→1→3→2→1→0). Contenu déjà adversarialement revu (Partie C : AC12-16, T-C1..T-C4, DC3/DC4/DC5/DC6/DC8/DC9). Re-validate optionnel. -->
<!-- DÉPEND de 17-4a (DONE : table password_reset_tokens, colonne users.email, find_by_email, repo tokens) ET 17-4b (DONE : trait Mailer, AppState.mailer, PASSWORD_RESET_TTL_MINUTES, config public_base_url/forgot_password_enabled). BLOQUE 17-4d (contrat API) + 17-4e (tests). -->

## Story

As a **développeur posant la surface HTTP publique du recovery de mot de passe self-service**,
I want **les deux endpoints publics `POST /api/v1/auth/forgot-password` (génération + envoi fire-and-forget du magic-link, anti-énumération, rate-limit dédié) et `POST /api/v1/auth/reset-password` (validation token usage-unique + TTL, update Argon2id, révocation des sessions), montés conditionnellement au feature flag, plus l'exposition de `forgotPasswordEnabled` dans `/health`**,
so that **le frontend 17-4d puisse consommer un contrat API stable et qu'un utilisateur ayant oublié son mot de passe puisse en définir un nouveau via le lien reçu par email, sans intervention d'un autre admin ni accès SSH/Docker au NAS**.

## Contexte & cadrage

**Issue source :** [#122](https://github.com/guycorbaz/kesh/issues/122) (recovery production-grade, `v0.2-milestone`). Épopée 17-4, scope **cœur recovery seul** (décision Guy 2026-06-10). Spec complète + DC + sécurité : voir umbrella `17-4-recovery-mot-de-passe.md`.

**Position dans le split A–F (ordre SÉRIE) :** 17-4a (DONE) → 17-4b (DONE) → **17-4c (ici)** → 17-4d (frontend) → 17-4e (tests) → 17-4f (doc). 17-4c **dépend de 17-4a** (DB foundation : `password_reset_tokens`, `users.email`, `find_by_email`, repo tokens avec garde TOCTOU) **et 17-4b** (couche mail : trait `Mailer`, `AppState.mailer`, `PASSWORD_RESET_TTL_MINUTES = 30`, config `smtp_*`/`public_base_url`/`forgot_password_enabled`). **BLOQUE 17-4d** (qui consomme le contrat des 2 endpoints) **et 17-4e** (tests d'intégration des flux).

**Ce qui est DÉJÀ posé par 17-4a + 17-4b (ne PAS réinventer — vérifié ground-truth 2026-06-11) :**
- Table `password_reset_tokens` + entité `PasswordResetToken` + repo `password_reset_tokens` complet : `create(pool, user_id, token_hash, expires_at)`, `find_valid_by_hash` (filtre `used_at IS NULL AND expires_at > NOW(3)`), `mark_used(pool, id)` (**garde TOCTOU `AND used_at IS NULL` → 0 ligne = `DbError::NotFound`**), `invalidate_all_for_user(pool, user_id)`.
- `users.email` (nullable, NON-unique) + `users::find_by_email(pool, email) -> Vec<User>` + `users::update_password(pool, user_id, hash)`.
- Couche mail : trait `Mailer::send_password_reset(&self, to, reset_url, locale: Locale) -> MailFuture` ; `AppState.mailer: Arc<dyn Mailer>` (défaut `NoopMailer`, `pub` pour mutation test) ; `MockMailer`/`MockMailer::failing()` ; `SmtpMailer` construit dans `main.rs` selon feature.
- Const partagée `PASSWORD_RESET_TTL_MINUTES: i64 = 30` (`crates/kesh-api/src/mail/mod.rs:30`) — **à consommer pour le calcul `expires_at` ET implicitement remontée dans l'email via le placeholder `{ $ttlMinutes }`** (déjà câblé côté `SmtpMailer`).
- Config : `config.forgot_password_enabled: bool`, `config.public_base_url: Option<String>` (déjà `.trim_end_matches('/')` appliqué au boot, P4-4), `config.locale`, `config.password_min_length`. Fail-fast boot déjà en place (feature on ⟹ SMTP + `public_base_url` complets).
- `AppError::SmtpSendFailed(String)` (→ 500, loggé `tracing::error!`, jamais exposé) + clés FTL `email-password-reset-subject`/`-body` ×4.

**Scope 17-4c (cette story) :**
- **T-C1** — `POST /api/v1/auth/forgot-password` (public, gated feature) : champ `rate_limiter_recovery` sur `AppState` (DC5), rate-limit `check`+`record` **inconditionnel**, lookup DC6, génération token base62 (générateur PAT généralisé, DC3) + SHA-256, `tokio::spawn` détaché pour l'envoi (DC4), `200` anti-énum **toujours**, audit `auth.password_reset_requested` **si match**.
- **T-C2** — `POST /api/v1/auth/reset-password` (public, gated) : hash + `find_valid_by_hash`, validation politique mdp, transaction (`mark_used` + `update_password` + audit `auth.password_reset_completed`), révocation refresh post-commit.
- **T-C3** — révocation refresh : **réutiliser `"password_change"`** (DC8/AC14, aucune migration CHECK).
- **T-C4** — `forgotPasswordEnabled` dans `/health` (DC9, deux branches 200/503) + montage **conditionnel** des routes selon `config.forgot_password_enabled` (`lib.rs`).

**Hors scope 17-4c** (sous-stories suivantes) : pages frontend `/forgot-password` + `/reset-password` + lien login conditionnel + champ email UI (17-4d) ; tests d'intégration des flux + E2E Playwright (17-4e) ; doc SMTP/recovery + `.env.example` + CHANGELOG/README (17-4f). 17-4c **n'ajoute que les tests unitaires de proximité** (rate-limiter recovery, helper de génération token) ; les tests d'intégration end-to-end des endpoints (happy/expiré/réutilisé/inexistant/no-email/SMTP-down/rate-limit) sont la **responsabilité de 17-4e** (AC23).

## Décisions de conception applicables (figées à l'umbrella, 6 passes)

- **DC3 — token SHA-256 hashé, générateur PAT réutilisé** (calque 17-2a). **Réutiliser/généraliser le générateur `crates/kesh-api/src/auth/api_key.rs`** (OsRng + base62 inline, ≥160 bits). Préférer **généraliser la fn** (param `prefix: &str` ou nouvelle `generate_reset_token()` partageant `base62_fixed_width`/`base62_encode` promus `pub(crate)`) — **PAS** ajouter `base64`/`base64url`, PAS réinventer. Stockage **uniquement** `SHA-256` hex (`sha256_hex`, déjà `pub`). Le brut ne vit que dans l'URL de l'email. **Token reset SANS préfixe `kesh_pat_`** (ce n'est pas une clé PAT ; le préfixe PAT route vers `validate_pat` côté middleware — un token reset ne doit jamais matcher ce chemin).
- **DC4 — anti-énumération : `forgot-password` retourne TOUJOURS `200`** corps générique, que l'utilisateur existe ou non. Aucun signal distinguant existant/inexistant (status, timing grossier, message). Envoi **fire-and-forget** via `tokio::spawn` détaché → l'échec SMTP n'est que loggé serveur, **jamais** propagé au client (un `500` créerait un oracle d'énumération, AC23-g). `reset-password` est symétrique : token-inexistant / token-expiré / token-utilisé → **même** `400 INVALID_OR_EXPIRED_TOKEN` générique.
- **DC5 — rate-limiting : instance `RateLimiter` dédiée recovery**. Champ `rate_limiter_recovery: Arc<RateLimiter>` sur `AppState` (distinct de `rate_limiter` login), **seuils hardcodés** (5 req / fenêtre 15 min / blocage 30 min — config par env var = L5 v0.2+). ⚠️ **Sémantique always-200 (catch P3 Opus F1)** : `RateLimiter` n'incrémente QUE via `record_failed_attempt` et `reset()` au succès. Comme forgot-password retourne toujours `200`, un simple `check_rate_limit` serait **inerte** (compteur jamais incrémenté). **MUST**, sur chaque requête forgot-password ET reset-password : (1) `check_rate_limit(ip)` → `429` si bloqué, puis (2) `record_failed_attempt(ip)` **inconditionnellement** (chaque requête consomme un slot), et (3) **NE JAMAIS** appeler `reset(ip)`.
- **DC6 — match utilisateur**. `users.email` nullable + NON-unique. Règle : (a) input **sans** `@` → `find_by_username` (UNIQUE, ≤1) ; (b) input **avec** `@` → `find_by_email` (Vec) ; si **exactement 1** match → procéder, si **0 ou >1** → traiter comme « pas de match » (aucun email, mais `200` anti-énum). Un compte sans email renseigné est non-recouvrable par ce flux (→ break-glass #121).
- **DC8 — token usage unique + TTL 30 min + révocation refresh**. `expires_at = Utc::now() + TimeDelta::minutes(PASSWORD_RESET_TTL_MINUTES)` (`.naive_utc()`, pattern `auth.rs:253`). Au reset réussi : `mark_used(token.id)` (garde TOCTOU) + `update_password` + audit `auth.password_reset_completed` **dans une transaction** ; **après commit** : `refresh_tokens::revoke_all_for_user(user_id, "password_change")` (raison **existante** réutilisée). Token `used_at IS NOT NULL` OU `expires_at < now` → `400 INVALID_OR_EXPIRED_TOKEN`.
- **DC9 — exposition feature-flag via `GET /health`**. Ajouter `forgotPasswordEnabled: bool` (config-dérivé `config.forgot_password_enabled`) **dans les deux branches** (200 OK et 503 degraded) de `health.rs` — indépendant de l'état DB. PAS de nouvel endpoint `/api/v1/config`.

## Acceptance Criteria

> Numérotation continue de l'umbrella (AC12-16, Partie C). Les AC8-11 (Partie B) sont DONE ; AC17-32 sont hors scope (17-4d/e/f).

12. **`POST /api/v1/auth/forgot-password`** (route **publique**, montée à côté de `/api/v1/auth/login` `lib.rs:503`, **seulement si `config.forgot_password_enabled == true`**) : corps `{ identifier: string }` (username ou email). **Rate-limited par IP (DC5 — `check_rate_limit` PUIS `record_failed_attempt` inconditionnel, JAMAIS `reset`)** → `429` (`AppError::RateLimited`) si dépassement, via `rate_limiter_recovery`. Lookup DC6 ; si match unique → génère token brut (**générateur PAT réutilisé, base62, DC3 — PAS de préfixe `kesh_pat_`, PAS base64url**), stocke `SHA-256` (`password_reset_tokens::create`) **avant** l'envoi, construit `reset_url = {config.public_base_url}/reset-password?token={brut}`, envoie via `state.mailer.send_password_reset(...)` en **tâche détachée `tokio::spawn`** (timing constant + ne bloque pas la réponse ; échec loggé serveur, jamais propagé — AC23-g). **Retourne TOUJOURS `200`** corps générique (DC4), que le user existe ou non. Audit `auth.password_reset_requested` (`entity_type` `user`, `entity_id` = user_id, `details_json` interne ex. `{ "identifier_kind": "username"|"email" }`) **uniquement si match** (un non-match ne crée aucune entrée — `audit_log.user_id` est NOT NULL).
13. **`POST /api/v1/auth/reset-password`** (route **publique**, même gating, **même rate-limit recovery** check+record inconditionnel DC5) : corps `{ token: string, new_password: string }`. `sha256_hex(token)` → `find_valid_by_hash`. Si absent/expiré/utilisé → `400 INVALID_OR_EXPIRED_TOKEN` (générique, pas de fuite). Valide le nouveau mot de passe (`password::validate_password(&new_password, config.password_min_length)`). Hash via `password::hash_password_async`. **Dans une transaction** : `mark_used(token.id)` (garde TOCTOU — si 0 ligne / `NotFound` → mapper `400 INVALID_OR_EXPIRED_TOKEN`, course concurrente) + `update_password(user_id, hash)` + `audit_log::insert_in_tx(NewAuditLogEntry::user(user_id, "auth.password_reset_completed", "user", user_id, details))` ; **après commit** : `refresh_tokens::revoke_all_for_user(user_id, "password_change")` (**hors transaction par design** — `revoke_all_for_user` prend `&MySqlPool` `refresh_tokens.rs:134`, best-effort/loggée-si-échec, pattern identique à `change_password` `auth.rs:514` et break-glass `bootstrap.rs:289`). Retourne `200` succès (corps minimal, ex. `{ "status": "ok" }`).
14. **Raison de révocation : RÉUTILISER `"password_change"`** — **pas de nouvelle valeur, donc pas de migration de contrainte CHECK** (`chk_refresh_tokens_revoked_reason`, migration `20260406000001`, inchangée). Le distinguo recovery-self-service ↔ change-UI ↔ break-glass est porté par `audit_log.action` (`auth.password_reset_completed` vs `admin_break_glass_reset`), pas par `revoked_reason`. (Précédent confirmé : `bootstrap.rs:289` + `auth.rs:514` réutilisent déjà `"password_change"`.)
15. **Exposition feature-flag (DC9)** : `GET /health` expose `forgotPasswordEnabled: bool` (= `config.forgot_password_enabled`, config-dérivé) présent dans les **deux** branches 200/503 de `health.rs` (indépendant de l'état DB), pour que 17-4d conditionne l'affichage du lien.
16. **Sécurité endpoints** : routes **hors** middleware `require_auth` (publiques, pré-login — montées dans le bloc public de `build_router`, AVANT `.merge(protected)`). Pas de scope PAT (routes publiques non authentifiées). Anti-énum DC4 cohérent sur les **deux** endpoints (reset-password ne distingue pas token-inexistant de token-expiré de token-utilisé). Token reset SANS préfixe `kesh_pat_` (DC3). Les corps de requête masquent le secret en `Debug` manuel (`new_password`/`token` → `***`, calque `ChangePasswordRequest` `auth.rs:468`).

### Transverses

- **Sécurité** : (a) token bearer hashé DC3, jamais en clair en DB ; (b) anti-énumération DC4 sur les 2 endpoints ; (c) rate-limit DC5 (instance dédiée, check+record inconditionnel) ; (d) pas de fuite du token en logs (ne jamais logger `reset_url` ni `token` brut — logger `user_id` au plus) ; (e) `Debug` manuel sur les DTO de requête. (f) Pas de CSRF nouveau : endpoints publics non authentifiés, pas de cookie de session impliqué.
- **Build/tests verts standalone** : 17-4c est mergeable — le `tokio::spawn` détaché + `NoopMailer`/`MockMailer` par défaut → aucun envoi réel. Quality gate Test Locally First backend vert (T-C5).

## Tasks / Subtasks

- [x] **T-C1** `POST /api/v1/auth/forgot-password` (public, gated feature). (AC: 12, 16)
  - Ajouter le champ `rate_limiter_recovery: Arc<RateLimiter>` à `AppState` (`lib.rs:43`), **défauté dans le CORPS de `new_for_tests`** (signature inchangée, P4-1 — construire un `RateLimiter::new(&config)` ou un `RateLimiter` à seuils recovery ; cf. note dev sur les seuils hardcodés) + ajouté manuellement aux 3 sites littéraux préservés : `main.rs:271` (vrai limiter recovery), `middleware/auth.rs:267` (`test_state`), `tests/setup_admin_e2e.rs:81`.
  - Construire l'instance recovery dans `main.rs` avec les **seuils hardcodés 5/15min/30min** (cf. note dev : soit en clonant `config` avec ces 3 champs surchargés avant `RateLimiter::new`, soit via un constructeur dédié — figer au dev).
  - Handler : extraire `ip` via `ConnectInfo<SocketAddr>` (`auth.rs:181`) ; `rate_limiter_recovery.check_rate_limit(ip)` → `429` ; **puis `record_failed_attempt(ip)` inconditionnel** (jamais `reset`). Lookup DC6 (`@` ⇒ `find_by_email` Vec, sinon `find_by_username`). Si match **unique** : générer token base62 (DC3), `create` SHA-256, construire `reset_url`, `tokio::spawn` détaché de `state.mailer.send_password_reset(email, &reset_url, config.locale)` (cloner les `Arc`/`String` nécessaires dans la closure). Audit `auth.password_reset_requested` si match. **Toujours `200`** générique. (DC4/DC6)
- [x] **T-C2** `POST /api/v1/auth/reset-password` (public, gated). (AC: 13, 16)
  - Même rate-limit recovery (check+record inconditionnel, DC5). `sha256_hex(token)` → `find_valid_by_hash` → `None` ⇒ `400 INVALID_OR_EXPIRED_TOKEN`. `validate_password` + `hash_password_async`. Transaction : `mark_used` (mapper `DbError::NotFound` → `400 INVALID_OR_EXPIRED_TOKEN`) + `update_password` + `audit_log::insert_in_tx`. Commit. Post-commit best-effort : `revoke_all_for_user(user_id, "password_change")` (loggé si échec). `200`.

### Review Findings

> Code review Pass 1 (Fable 5, 2026-06-11) — 3 couches parallèles (Blind Hunter / Edge Case Hunter / Acceptance Auditor), 30 findings bruts → dédupliqués : 0 CRITICAL/HIGH, 8 MEDIUM, 12 LOW. Triage : 10 patch, 4 defer, 3 dismiss.

- [x] [Review][Patch] P1 (MEDIUM, blind+edge+auditor) Oracle timing + oracle 500 + audit/token orphelin — déplacer TOUT le travail post-lookup de `forgot_password` (audit, invalidate, create, mail) dans la tâche `tokio::spawn` détachée ; réponse `200` immédiate dans les 2 branches ; erreurs DB loggées jamais propagées ; audit écrit dès le match (avec `recoverable: bool`) ; résoudre `public_base_url` AVANT `create` [crates/kesh-api/src/routes/auth.rs:705-775]
- [x] [Review][Patch] P2 (MEDIUM, blind+edge+auditor) Atomicité + DRY transaction reset — ajouter `password_reset_tokens::mark_used_in_tx` et `users::update_password_in_tx` (garde `rows_affected`) dans kesh-db ; transaction unique mark_used + update_password + audit [crates/kesh-api/src/routes/auth.rs:828-868]
- [x] [Review][Patch] P3 (MEDIUM, blind+edge+auditor) `reset_password` ne re-vérifie pas `user.active` — charger le user, `!active` → même `400 INVALID_OR_EXPIRED_TOKEN` générique (cohérence gel admin, symétrie avec le gate à l'émission) [crates/kesh-api/src/routes/auth.rs:828]
- [x] [Review][Patch] P4 (MEDIUM, edge) Comptage `len == 1` email inclut les comptes inactifs — `candidates.retain(|u| u.active)` avant le comptage (un doublon désactivé prive un actif du recovery) [crates/kesh-api/src/routes/auth.rs:687-693]
- [x] [Review][Patch] P5 (MEDIUM, edge) Username contenant `@` structurellement non-recouvrable (DC6) — interdire `@` à la création/édition d'username (garde validation) ; legacy → break-glass #121 [crates/kesh-api/src/routes/users.rs:164-181]
- [x] [Review][Patch] P6 (LOW, edge) Token non trimé dans `reset_password` — `req.token.trim()` avant `sha256_hex` (copier-coller email avec espace/retour-ligne) [crates/kesh-api/src/routes/auth.rs:810]
- [x] [Review][Patch] P7 (LOW, blind) Double binding `let mut main_router` redondant après le bloc conditionnel — nettoyer [crates/kesh-api/src/lib.rs:553]
- [x] [Review][Patch] P8 (LOW, blind) `check_rate_limit` + `record_failed_attempt` non-atomiques sous burst concurrent — méthode `check_and_record` sous un seul lock [crates/kesh-api/src/middleware/rate_limit.rs]
- [x] [Review][Patch] P9 (LOW, blind+edge) Commentaire « anti-accumulation » sur-vend l'invariant — invalidate+create non-transactionnels = best-effort, corriger le wording [crates/kesh-api/src/routes/auth.rs:714]
- [x] [Review][Patch] P10 (MEDIUM, auditor, process) Story file non à jour — statut, checkboxes, Agent Record, File List, Change Log
- [x] [Review][Defer] D1 (MEDIUM, edge) `ConnectInfo` derrière reverse proxy → IP partagée → DoS global du recovery (record inconditionnel DC5 amplifie) — DC5 figé + pattern pré-existant login ; documenté §Limitations ci-dessous + issue GitHub enhancement trusted-proxy XFF
- [x] [Review][Defer] D2 (LOW, edge) Lockout utilisateur légitime à mi-flux (limiter partagé forgot+reset, blocage 30 min = TTL token) — lié L5 (seuils configurables v0.2+) ; documenté §Limitations
- [x] [Review][Defer] D3 (LOW, blind+edge) `expires_at` horloge app vs `NOW(3)` horloge MariaDB (skew/TZ) — pattern pré-existant identique `refresh_tokens` ; exigence NTP/UTC à documenter manuel admin (17-4f)
- [x] [Review][Defer] D4 (LOW, edge) `tokio::spawn` fire-and-forget perdu au shutdown (email jamais envoyé, zéro trace) — acceptable v0.1, documenté §Limitations

Dismissed (3) : casse email (réfuté ground-truth — collation `_ci`, `users.rs:174`) ; token en query param (design figé AC12, magic-link standard, doc 17-4f) ; bypass rate-limit par body JSON malformé (pattern identique login in-handler, accepté).

### Limitations documentées (17-4c)

- **L-C1 (D1)** : derrière un reverse proxy, le rate-limit recovery voit l'IP du proxy pour tous les clients ; 5 requêtes recovery quelconques / 15 min bloquent le flux 30 min pour toute l'installation. Remédiation : support `X-Forwarded-For` opt-in (env var trusted proxy) — **issue [#173](https://github.com/guycorbaz/kesh/issues/173)** (`enhancement` + `v0.2-milestone`).
- **L-C2 (D2)** : le blocage (30 min) égale le TTL du token ; un utilisateur qui consomme ses 5 slots à mi-flux doit recommencer. Remédiation : seuils configurables (L5 umbrella, v0.2+).
- **L-C3 (D4)** : l'envoi d'email détaché n'est pas drainé au shutdown (redeploy Docker pendant l'envoi = email perdu sans trace). Acceptable v0.1 ; piste `TaskTracker` si récurrent.
- [x] **T-C3** Révocation refresh : **réutiliser `"password_change"`** (DC8/AC14) — **aucune migration de contrainte CHECK**. (AC: 14)
- [x] **T-C4** `forgotPasswordEnabled` dans `/health` (DC9, **deux branches** 200/503). Montage **conditionnel** des routes `/api/v1/auth/forgot-password` + `/reset-password` dans le bloc public de `build_router` (`lib.rs:501-512`) selon `state.config.forgot_password_enabled` (si `false` → routes non montées → `404`). (AC: 15, 16)
- [x] **T-C5** Quality gate Test Locally First backend (fmt/build/clippy -D/test). Les endpoints touchent `kesh-api` uniquement (pas `kesh-db`) ; mode parallèle suffit pour les tests unitaires. Si des tests d'intégration DB sont ajoutés ici (proximité), lancer aussi le **mode serial** `cargo test --workspace -j1 -- --test-threads=1`. (AC: transverse build vert)

## Dev Notes

### Ground-truth (état réel post-17-4a + 17-4b, vérifié 2026-06-11)

**Token & hash (DC3 — `crates/kesh-api/src/auth/api_key.rs`) :**
- `pub fn generate_pat() -> (String, String)` (`:107`) = `(token_clair, key_hash)` ; **hardcode le préfixe `kesh_pat_`** (`:110`) → **NE PAS** réutiliser tel quel pour le reset (le préfixe route vers `validate_pat` côté middleware). `pub fn sha256_hex(token: &str) -> String` (`:91`) **déjà `pub`, réutilisable directement**.
- Helpers `base62_encode` (`:48`) + `base62_fixed_width` (`:73`, `debug_assert_eq!` longueur 20 octets) + const `PAT_ENTROPY_BYTES = 20` (160 bits, `:33`) + `BASE62_ALPHABET` (`:41`) sont **privés** au module. RNG : `OsRng.fill_bytes` (`:108-109`, `argon2::password_hash::rand_core`, aucune dép nouvelle).
- **Décision dev (DC3, P4-3)** : généraliser proprement. Option recommandée — ajouter `pub fn generate_reset_token() -> (String, String)` dans `api_key.rs` (ou un module `auth/recovery_token.rs`) qui fait `OsRng.fill_bytes(&mut [0u8; PAT_ENTROPY_BYTES])` + `base62_fixed_width` (promu `pub(crate)`) **sans préfixe**, puis `sha256_hex`. Évite la duplication des ~25 lignes base62. Le token reset = 27 chars base62 (URL-safe : `0-9A-Za-z`, aucun escaping nécessaire dans le query param).

**Repo tokens (DONE 17-4a — `crates/kesh-db/src/repositories/password_reset_tokens.rs`) :**
- `create(pool, user_id, token_hash: &str, expires_at: NaiveDateTime) -> Result<PasswordResetToken, DbError>` (`:24`).
- `find_valid_by_hash(pool, token_hash: &str) -> Result<Option<PasswordResetToken>, DbError>` (`:63`, filtre `used_at IS NULL AND expires_at > NOW(3)`).
- `mark_used(pool, id: i64) -> Result<(), DbError>` (`:88`) — **garde TOCTOU `AND used_at IS NULL`** : 0 ligne affectée ⇒ `DbError::NotFound` (course concurrente / double-consume). **Prend `&MySqlPool` (PAS `&mut Tx`)** → dans la transaction reset, soit l'appeler avec `&state.pool` (la garde SQL ferme la fenêtre, suffisant), soit ouvrir une tx pour `update_password`+audit et appeler `mark_used` séparément. Vérifier au dev si une variante `_in_tx` est requise pour l'atomicité stricte ; le pattern existant (mark_used hors-tx + garde SQL) est acceptable (cf. revue 17-4a Pass 2 « garde TOCTOU mark_used »).
- `invalidate_all_for_user(pool, user_id)` (`:108`) — optionnel : invalider les tokens pendants à la création d'un nouveau (anti-accumulation). À décider au dev (non bloquant).

**Users & password (DONE 17-4a / existant) :**
- `users::find_by_email(pool, email: &str) -> Result<Vec<User>, DbError>` (`users.rs:174`, collation case-insensitive, retourne **Vec** car non-unique DC6).
- `users::find_by_username(pool, username) -> Option<User>` (`users.rs:156`).
- `users::update_password(pool, user_id, hash)` (`users.rs:291`, incrémente `version`, **ne révoque PAS** — c'est au caller).
- `password::validate_password(pwd, min_length) -> Result<(), AppError>` (`auth/password.rs:113`) ; `password::hash_password_async(String) -> Result<String, AppError>` (`:49`). Politique : `config.password_min_length`.
- Référence de flux complet (à calquer SANS l'auth admin) : `change_password` handler `auth.rs:481-559` (validate → find user → hash → `update_password` → `revoke_all_for_user(_, _, "password_change")` `auth.rs:514`) et break-glass `bootstrap.rs:255-302` (audit `NewAuditLogEntry::user` + commit tx + post-commit `revoke_all_for_user(pool, u.id, "password_change")` best-effort loggé).

**Mailer & TTL (DONE 17-4b) :**
- `state.mailer: Arc<dyn mail::Mailer>` (`lib.rs:52`, `pub`). `Mailer::send_password_reset(&self, to: &str, reset_url: &str, locale: Locale) -> MailFuture` (`mail/mod.rs:48`). Le `locale` est l'enum `kesh_i18n::Locale` ; passer `config.locale` (DC10, pas de locale per-user).
- `pub const PASSWORD_RESET_TTL_MINUTES: i64 = 30` (`mail/mod.rs:30`) — utiliser pour `expires_at`. Calcul : `(chrono::Utc::now() + chrono::TimeDelta::minutes(PASSWORD_RESET_TTL_MINUTES)).naive_utc()` (pattern `auth.rs:253` / `jwt.rs` `TimeDelta`).
- Le `tokio::spawn` détaché : cloner `state.mailer` (`Arc`), `email`, `reset_url`, `config.locale` (`Copy`/`Clone`) dans la closure `async move`. Logger `tracing::error!` si `Err` (jamais propager). `tokio` `["full"]` présent (`Cargo.toml:22`).

**Config (DONE 17-4b — `crates/kesh-api/src/config.rs`) :**
- `config.forgot_password_enabled: bool` (`:298`), `config.public_base_url: Option<String>` (`:293`, déjà `.trim_end_matches('/')` au boot). Fail-fast garantit que si `forgot_password_enabled`, `public_base_url` est `Some` non-vide → safe d'`unwrap`/`expect` au montage des routes (ou garder un `if let` défensif). `config.locale`, `config.password_min_length` existants.

**RateLimiter (existant — `crates/kesh-api/src/middleware/rate_limit.rs`) :**
- `RateLimiter::new(config: &Config) -> Self` (`:44`) lit `config.rate_limit_max_attempts` / `rate_limit_window` / `rate_limit_block_duration`. API : `check_rate_limit(ip) -> Result<(), RateLimitReject>` (`:65`), `record_failed_attempt(ip)` (`:127`), `reset(ip)` (`:148`). **Seuils recovery hardcodés 5/15min/30min** : au dev, construire un `Config` cloné avec ces 3 champs surchargés OU un petit constructeur `RateLimiter::with_thresholds(max, window, block)` (préféré — 1 fn, évite de muter un Config). `AppState` derive `Clone` ; le limiter est `Arc`. Pattern de mapping `RateLimitReject` → `AppError::RateLimited { retry_after }` : `auth.rs:188-193`.

**AppState & anti-churn (`crates/kesh-api/src/lib.rs`) :**
- `AppState` (`:43`) actuel = `{ pool, config, rate_limiter, i18n, users_exist, mailer }` (6 champs). `new_for_tests(pool, config, rate_limiter, i18n)` (`:65`) défaute `users_exist`/`mailer` dans le corps. **Ajouter `rate_limiter_recovery` en le défautant dans le corps de `new_for_tests`** (signature inchangée, P4-1) → les ~33 call-sites de test 17-4a restent intacts. **3 sites littéraux à patcher manuellement** : `main.rs:271` (vrai limiter recovery), `middleware/auth.rs:267` (`test_state`), `tests/setup_admin_e2e.rs:81`.

**Routeur public (`crates/kesh-api/src/lib.rs:501-512`) :**
- Bloc public `main_router` : `/health`, `/api/v1/auth/{login,logout,refresh}`, `/api/v1/setup/admin`, **AVANT** `.merge(protected)` (lui sous `require_auth`). **Ajouter `/api/v1/auth/forgot-password` + `/reset-password` ici, conditionnellement** : `let mut main_router = ...; if state.config.forgot_password_enabled { main_router = main_router.route("/api/v1/auth/forgot-password", post(...)).route("/api/v1/auth/reset-password", post(...)); }` AVANT le `.merge(protected)` (vérifier que `main_router` reste mutable au bon endroit ; cf. pattern conditionnel `test_mode` `lib.rs:519-524`). Routes publiques confirmées : pas de `route_layer(require_auth)`.

**Audit (existant) :**
- `audit_log::insert_in_tx(tx: &mut Transaction<MySql>, new: NewAuditLogEntry) -> Result<AuditLogEntry, DbError>` (`repositories/audit_log.rs:29`). Constructeur `NewAuditLogEntry::user(user_id, action, entity_type, entity_id, details_json)` (`entities/audit_log.rs:131`) — **utiliser celui-ci** (pas `from_current_user`, car les endpoints publics n'ont AUCUN `CurrentUser` en scope — pré-auth). `actor_type = User`, `actor_api_key_id = None` (sémantique correcte : recovery self-service). `audit_log.user_id` est **NOT NULL** → pour forgot-password non-matché, **ne créer aucune entrée**. Actions = strings libres : `auth.password_reset_requested` / `auth.password_reset_completed`.

**AppError (existant — `crates/kesh-api/src/errors.rs`) :**
- `AppError::Validation(String)` → `400 VALIDATION_ERROR` (`:716`) — utilisé par `validate_password`. `AppError::RateLimited { retry_after }` → `429` + `Retry-After` (`:792`). `AppError::Database(DbError)` standard.
- **Pas de variant `INVALID_OR_EXPIRED_TOKEN` existant** → en créer un nouveau (`AppError::InvalidOrExpiredToken`, mappé `400` code `INVALID_OR_EXPIRED_TOKEN` + clé FTL `error-invalid-or-expired-token` ×4 locales, pattern enum + `IntoResponse` `errors.rs`). Anti-fuite : message générique « Lien de réinitialisation invalide ou expiré ». **Ne PAS** réutiliser `Validation` (sémantique 400 mais code distinct attendu par le frontend 17-4d). Le mapping `DbError::NotFound` de `mark_used` (course) → même `InvalidOrExpiredToken`.

### Project Structure Notes

- **Nouveaux fichiers / fonctions** : handlers `forgot_password` + `reset_password` dans `crates/kesh-api/src/routes/auth.rs` (étendu — DTO `ForgotPasswordRequest { identifier }`, `ResetPasswordRequest { token, new_password }` avec `Debug` manuel masquant le secret). Générateur `generate_reset_token()` dans `auth/api_key.rs` (ou `auth/recovery_token.rs`).
- **Modifs** : `lib.rs` (champ `AppState.rate_limiter_recovery` + `new_for_tests` corps + montage conditionnel routes), `main.rs:271` (construire le limiter recovery + champ AppState), `middleware/auth.rs:267` + `tests/setup_admin_e2e.rs:81` (littéraux), `routes/health.rs` (`forgotPasswordEnabled` ×2 branches), `errors.rs` (variant `InvalidOrExpiredToken`), `crates/kesh-i18n/locales/{fr,de,it,en}-CH/messages.ftl` (clé `error-invalid-or-expired-token` ×4).
- Aucune migration DB (17-4a/b ont tout posé). Aucune divergence structurelle — aligné conventions (token PAT 17-2a, RateLimiter login, audit string-action `insert_in_tx`, public-route login, révocation `"password_change"`, fire-and-forget `tokio::spawn`).

### References

- [Source: umbrella `_bmad-output/implementation-artifacts/17-4-recovery-mot-de-passe.md` — Partie C AC12-16, T-C1..T-C4, DC3/DC4/DC5/DC6/DC8/DC9, convergée 6 passes (P3 Opus catch RateLimiter inerte)]
- [Source: GitHub Issue #122 — recovery production-grade ; #121 break-glass conservé]
- [Source: crates/kesh-api/src/auth/api_key.rs:48,73,91,107 — base62/sha256_hex/generate_pat (DC3, à généraliser sans préfixe)]
- [Source: crates/kesh-db/src/repositories/password_reset_tokens.rs:24,63,88,108 — create/find_valid_by_hash/mark_used (garde TOCTOU)/invalidate_all_for_user (17-4a DONE)]
- [Source: crates/kesh-db/src/repositories/users.rs:156,174,291 — find_by_username/find_by_email(Vec)/update_password]
- [Source: crates/kesh-api/src/auth/password.rs:49,113 — hash_password_async/validate_password]
- [Source: crates/kesh-api/src/routes/auth.rs:181,188,253,468,481,514 — ConnectInfo IP, rate-limit→429, expires_at TimeDelta, Debug masqué, change_password flux, revoke "password_change"]
- [Source: crates/kesh-api/src/auth/bootstrap.rs:255-302 — break-glass : audit user + commit tx + post-commit revoke best-effort "password_change"]
- [Source: crates/kesh-api/src/mail/mod.rs:30,48 — PASSWORD_RESET_TTL_MINUTES, Mailer::send_password_reset (17-4b DONE)]
- [Source: crates/kesh-api/src/config.rs:293,298 — public_base_url (trim trailing slash), forgot_password_enabled (17-4b DONE)]
- [Source: crates/kesh-api/src/middleware/rate_limit.rs:44,65,127,148 — RateLimiter::new/check/record/reset (DC5 instance dédiée)]
- [Source: crates/kesh-api/src/lib.rs:43,65,501-512,519-524 — AppState, new_for_tests, bloc routes public, montage conditionnel (pattern test_mode)]
- [Source: crates/kesh-api/src/main.rs:223,271 — RateLimiter::new + struct literal AppState prod]
- [Source: crates/kesh-api/src/middleware/auth.rs:267 ; tests/setup_admin_e2e.rs:81 — 2 littéraux AppState à patcher]
- [Source: crates/kesh-db/src/repositories/audit_log.rs:29 ; crates/kesh-db/src/entities/audit_log.rs:131 — insert_in_tx, NewAuditLogEntry::user]
- [Source: crates/kesh-api/src/routes/health.rs — shape {status,db,version} à étendre forgotPasswordEnabled (DC9)]
- [Source: crates/kesh-api/src/errors.rs:716,792 — Validation 400, RateLimited 429 (pattern variant + IntoResponse pour InvalidOrExpiredToken)]
- [Source: CLAUDE.md §Test Locally First, §Issue Tracking]

## Dev Agent Record

### Agent Model Used

Dev-story : session interrompue avant commit (code retrouvé complet dans le working tree le 2026-06-11, T-C1..T-C4 implémentés, fmt + build verts). Finalisation + code review Pass 1 : Claude Fable 5.

### Debug Log References

### Completion Notes List

- T-C1..T-C4 implémentés conformément aux DC3/DC4/DC5/DC6/DC8/DC9 ; tests unitaires de proximité présents (format/unicité token reset, `with_thresholds` rate-limiter).
- Choix dev documentés : `mark_used` initialement hors-tx (Dev Notes l'autorisaient) — revu en Pass 1 (P2) vers variantes `_in_tx` ; UPDATE password initialement SQL inline — revu en Pass 1 (P2) vers `update_password_in_tx` ; gate `user.active` à l'émission (hors-spec, décision sécurité documentée, symétrie ajoutée côté reset en P3).
- Quality gate T-C5 : exécuté après les patches Pass 1 (cf. Change Log).

### File List

- crates/kesh-api/src/auth/api_key.rs — `generate_reset_token()` + `base62_*` promus `pub(crate)` + 2 tests
- crates/kesh-api/src/errors.rs — variant `AppError::InvalidOrExpiredToken` (400 `INVALID_OR_EXPIRED_TOKEN`)
- crates/kesh-api/src/lib.rs — champ `AppState.rate_limiter_recovery`, `build_recovery_rate_limiter()`, montage conditionnel routes
- crates/kesh-api/src/main.rs — construction limiter recovery prod
- crates/kesh-api/src/middleware/auth.rs — littéral test_state
- crates/kesh-api/src/middleware/rate_limit.rs — `RateLimiter::with_thresholds` + test
- crates/kesh-api/src/routes/auth.rs — handlers `forgot_password` + `reset_password` + DTOs + `enforce_recovery_rate_limit`
- crates/kesh-api/src/routes/health.rs — `forgotPasswordEnabled` ×2 branches
- crates/kesh-api/tests/setup_admin_e2e.rs — littéral E2E
- crates/kesh-i18n/locales/{fr,de,it,en}-CH/messages.ftl — clé `error-invalid-or-expired-token` ×4

## Change Log

### Pass 1 code-review (Fable 5, 2026-06-11)

- **Setup** : diff = working tree non commité (dev-story interrompue avant commit ; snapshot vérifié fmt+build verts puis commité `feat` avant patches). 3 couches parallèles même modèle (Blind Hunter diff-only / Edge Case Hunter + accès projet / Acceptance Auditor + spec + umbrella).
- **Findings** : 30 bruts → dédup → 0 CRITICAL/HIGH, 8 MEDIUM, 12 LOW → triage 10 patch / 4 defer / 3 dismiss. Forte convergence inter-couches (timing oracle ×2, mark_used hors-tx ×3, gate `active` ×3).
- **Patches appliqués** :
  - **P1** (MEDIUM) `forgot_password` restructuré : TOUT le travail post-match (audit, invalidate, create, envoi) déplacé dans la tâche `tokio::spawn` détachée via `process_forgot_password_match` — supprime l'oracle de timing (~6 round-trips DB synchrones seulement côté match) ET l'oracle 500 (`?` propagés seulement côté match) ET le token-créé-sans-audit (base résolue avant `create`). Audit écrit pour TOUT match (AC12) avec `details_json.recoverable`.
  - **P2** (MEDIUM) Atomicité reset : nouvelles variantes `password_reset_tokens::mark_used_in_tx` + `users::update_password_in_tx` (kesh-db, SQL partagé par const avec les variantes pool, garde TOCTOU/rows_affected identiques) ; transaction unique mark_used + update_password + audit — un échec rollback tout, le token n'est plus brûlé sans changement de mot de passe. Remplace aussi l'UPDATE SQL inline (DRY, garde NotFound restaurée).
  - **P3** (MEDIUM) `reset_password` re-vérifie `users.active` à la consommation (`find_by_id` + filter) → même `400` générique (gel admin respecté pendant le TTL).
  - **P4** (MEDIUM) `candidates.retain(|u| u.active)` avant le comptage « exactement 1 » — un doublon email désactivé ne prive plus le compte actif du recovery.
  - **P5** (MEDIUM) Garde `username.contains('@')` → 400 à la création (routes/users.rs i18n `error-username-contains-at` ×4 + setup.rs) — protège l'invariant d'aiguillage DC6. Legacy/bootstrap env-var : break-glass #121.
  - **P6** (LOW) `req.token.trim()` avant `sha256_hex` (URLs wrappées par les clients mail).
  - **P7** (LOW) lib.rs : re-binding `let mut main_router` redondant → simple réassignation.
  - **P8** (LOW) `RateLimiter::check_and_record` atomique sous un seul lock (refactor helpers privés `purge_expired`/`check_locked`/`record_locked`, méthodes existantes inchangées) + test `check_and_record_atomic_blocks_at_threshold` ; `enforce_recovery_rate_limit` migré.
  - **P9** (LOW) Commentaire anti-accumulation requalifié « best-effort » (invariant non tenu sous concurrence, impact nul — même boîte mail).
  - **P10** (process) Story file complété : statut, checkboxes, Agent Record, File List, Review Findings, Limitations L-C1..L-C3, ce Change Log.
- **Defers documentés** (§Limitations + deferred-work.md) : D1 MEDIUM reclassé dette documentée (XFF trusted-proxy, **issue #173** `enhancement`+`v0.2-milestone`) ; D2/D3/D4 LOW.
- **Dismiss** (3) : casse email (réfuté ground-truth collation `_ci`), token en query param (design AC12), bypass rate-limit body malformé (pattern login identique).
- **Décisions documentées** : gate `user.active` à l'émission = ajout hors-spec conservé (défendable sécurité) + symétrie ajoutée côté reset (P3) ; cas de test à couvrir en 17-4e (compte inactif, email dupliqué actif/inactif, username avec `@`, double-consume, trim token).
- **Prochaine passe** : Pass 2 LLM différent (≥1 MEDIUM trouvé → Review Iteration Rule), diff aplati HEAD vs main.

### Pass 2 code-review (Sonnet 4.6, 2026-06-11)

- **Setup** : diff unique aplati `64c8050..HEAD` (1081 lignes, feat + fix Pass 1), 3 couches Sonnet contexte frais.
- **Findings bruts** : BH 1 HIGH + 5 MEDIUM + 3 LOW ; ECH 1 MEDIUM + 3 LOW ; AA 2 LOW (24 points conformes vérifiés ground-truth).
- **Réfutés grep ground-truth (4)** : BH2-HIGH `ConnectInfo` non câblé — faux positif, `main.rs:304` utilise `into_make_service_with_connect_info::<SocketAddr>` ; BH2-LOW test combo `with_thresholds`+`check_and_record` non couverte — faux, `rate_limit.rs:326` la couvre exactement ; BH2-LOW slash trailing `public_base_url` — strippé au boot (17-4b P4-4) ; BH2-MEDIUM timeout SMTP manquant — `lettre::AsyncSmtpTransport` applique un timeout défaut 60 s au niveau builder, et le rate-limit borne le débit de tasks.
- **Patches appliqués** :
  - **PP1** (MEDIUM, BH2-M6) Lookup DC6 déplacé dans la tâche détachée (`process_forgot_password_request`) : le handler `forgot_password` ne touche plus du tout la DB → timing strictement constant (Pass 1 avait détaché les écritures, la latence du lookup présent-vs-absent restait théoriquement observable). Inclut l'early-return identifiant vide (ECH2-L1, no-op sans spawn) et le commentaire d'asymétrie username/inactif (BH2-M3 downgradé doc).
  - **PP2** (MEDIUM, ECH2-M1) `Instant::checked_sub` dans `check_locked`/`record_locked` — panique latente `now - window` si la machine a booté il y a moins de 15 min (autostart Docker post-reboot NAS) ; pré-existante côté login, étendue à `check_and_record` par la Pass 1, corrigée pour les deux.
  - **PP3** (process, AA2-L1) T-C5 coché + statut `review`.
- **Dismiss (hors grep-réfutés)** : DoS NAT/IP partagée (= D1, issue #173) ; couplage `sha256_hex` (nit) ; invalidation best-effort (documentée P9, fail-closed dégraderait l'UX) ; `@` seul (no-op anti-énum) ; garde `TimeDelta::minutes` (spéculatif, const 30).
- **Defer** : AA2-L2 garde `@` non-i18n dans `setup.rs` — cohérent avec le pattern pré-existant du fichier (`"username must be non-empty"` hardcodé) ; rejoint le cleanup i18n setup.rs déjà tracé dans deferred-work.md (BH2-4 v011-5).
- **Trend >LOW : Pass 1 = 8 MEDIUM → Pass 2 = 2 MEDIUM réels (+1 MEDIUM et 1 HIGH réfutés)** → Pass 3 requise (LLM différent : Haiku, garde-fous grep ground-truth obligatoires).
