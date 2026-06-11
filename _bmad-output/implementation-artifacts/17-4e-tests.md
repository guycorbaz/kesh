# Story 17.4e: Tests recovery — intégration Rust + E2E Playwright

Status: ready-for-dev

<!-- Extraite de la spec parente UMBRELLA 17-4 (validate CONVERGÉ 6 passes), Partie E : AC23-24. Re-validate optionnel. -->
<!-- DÉPEND de 17-4c (endpoints DONE) + 17-4d (pages frontend DONE). Avant-dernière sous-story (reste 17-4f doc). -->

## Story

As a **mainteneur de Kesh**,
I want **une suite de tests d'intégration Rust couvrant les flux complets forgot/reset-password (happy path, expiré, réutilisé, anti-énumération, SMTP down, rate-limit, et tout le backlog de cas tracé en 17-4c/d) et des E2E Playwright sur les pages publiques (token injecté via l'API de test)**,
so that **le cœur sécurité du recovery (anti-énum DC4, usage-unique DC8, rate-limit DC5) soit verrouillé par la CI et que toute régression future soit détectée avant merge**.

## Contexte & cadrage

**Issue source :** [#122](https://github.com/guycorbaz/kesh/issues/122). Épopée 17-4, Partie E. Position : 17-4a✅ → b✅ → c✅ → d✅ → **17-4e (ici)** → 17-4f (doc).

**Contrats testés (figés 17-4c/d) :** `POST /api/v1/auth/forgot-password` `{identifier}` → toujours `200` corps vide (montée seulement si `forgot_password_enabled`, sinon 404) ; tout le travail post-match (audit `auth.password_reset_requested` + `details.recoverable`, invalidate, create token, envoi mail) en **tâche `tokio::spawn` détachée**. `POST /api/v1/auth/reset-password` `{token,newPassword}` → `200 {"status":"ok"}` | `400 INVALID_OR_EXPIRED_TOKEN` (générique : inconnu/expiré/utilisé/inactif) | `400 VALIDATION_ERROR` | `429`. Token trimé. Transaction unique mark_used+update_password+audit `auth.password_reset_completed` ; revoke refresh `"password_change"` post-commit. Rate-limit dédié partagé forgot+reset : `check_and_record` atomique, 5 req/15 min/IP, blocage 30 min. Pages frontend : testids `forgot-identifier/forgot-submit/forgot-success/forgot-error`, `reset-password/reset-password-confirm/reset-submit/reset-success/reset-invalid-link/reset-request-new-link/reset-error`, lien login `forgot-password-link`.

**Backlog de cas hérité (Change Logs 17-4c Pass 1+3 et 17-4d) :** compte inactif, email dupliqué actif/inactif, username avec `@`, double-consume, trim token, VALIDATION_ERROR ne brûle pas le token, SMTP-down → 200 quand même.

**Scope 17-4e :** T-E1 support de test backend (gated `test_mode`) ; T-E2 suite intégration `password_recovery_e2e.rs` ; T-E3 spec Playwright `password-recovery.spec.ts` ; T-E4 quality gate complet (serial + E2E live).

**Hors scope :** doc (17-4f) ; vérification de réception email réelle multi-providers (test manuel hors-CI documenté en 17-4f, dette de validation actée umbrella AC24) ; rate-limit configurable (L5 v0.2) ; XFF (#173).

## Décisions de conception

- **DE-1 — injection de token E2E via endpoint test-mode** : ajouter `POST /api/v1/_test/password-reset-token` `{username}` dans `routes/test_endpoints.rs` (router monté SEULEMENT si `config.test_mode`, pattern `/seed`+`/reset` existant) : lookup user par username, `generate_reset_token()`, `password_reset_tokens::create`, retourne `{ token: <clair> }`. C'est la voie « token injecté via seed/API de test » prévue par l'umbrella AC24 (l'email réel n'est pas vérifiable sans SMTP).
- **DE-2 — purge des rate-limiters au seed** : le limiter recovery (5 req/15 min, mémoire) est partagé entre TOUS les tests E2E d'un même backend → flaky garanti en re-runs locaux. Ajouter `RateLimiter::clear_all()` (lock + `map.clear()`, méthode triviale) et l'appeler sur `rate_limiter` + `rate_limiter_recovery` dans le `seed_handler` (`/_test/seed` est déjà invoqué par `seedTestState` en tête de spec). Effet test-mode only (le handler n'est monté qu'en test_mode).
- **DE-3 — synchronisation avec la task détachée** : les tests d'intégration attendent l'effet de la task via boucle de polling bornée (`for _ in 0..50 { if cond { break } sleep(20ms) }`) sur `MockMailer::sent()` ou sur la DB (`SELECT COUNT(*) FROM password_reset_tokens`). JAMAIS de sleep fixe seul (flaky).
- **DE-4 — isolation rate-limit en intégration** : chaque test `#[sqlx::test]` construit son propre `AppState` littéral → limiter vierge par test. Les tests fonctionnels remplacent `rate_limiter_recovery` par `RateLimiter::with_thresholds(1000, …)` (permissif) ; le test dédié 429 garde un seuil bas (ex. 3) pour déclencher vite.
- **DE-5 — config feature-on en test** : `Config::from_fields_for_test(…)` puis mutation directe des champs `pub` : `config.forgot_password_enabled = true; config.public_base_url = Some("http://127.0.0.1".into())`. PAS de SMTP nécessaire (le `state.mailer` est remplacé par `MockMailer` — le fail-fast SMTP n'existe qu'au boot `from_env`, pas dans le constructeur de test).
- **DE-6 — backend E2E avec feature on** : le run Playwright local exige désormais `KESH_FEATURE_FORGOT_PASSWORD=true` + `KESH_SMTP_*` factices complets + `KESH_PUBLIC_BASE_URL` (fail-fast boot) sur le backend test-mode. Documenter la recette dans le spec header + `docs/testing.md` (mise à jour 17-4f si besoin de plus). Les envois SMTP réels échouent en arrière-plan (loggés) — sans impact : l'E2E injecte le token via DE-1.

## Acceptance Criteria

> Numérotation umbrella (AC23-24, Partie E).

23. **Tests d'intégration Rust** (`crates/kesh-api/tests/password_recovery_e2e.rs`, `#[sqlx::test(migrator = "kesh_db::MIGRATOR")]`, pattern spawn_app de `auth_e2e.rs:92-157` adapté avec AppState littéral) couvrant AU MOINS :
    - (a) **happy path complet** : user avec email → POST forgot → 200 → polling `MockMailer.sent()` → 1 mail capturé, `reset_url` contient `?token=` → extraire le token → POST reset (nouveau mdp valide) → 200 `{"status":"ok"}` → login ancien mdp = 401, login nouveau mdp = 200 → les refresh tokens émis AVANT le reset sont révoqués (`revoked_reason="password_change"`) → audit `auth.password_reset_requested` (avec `recoverable:true` dans details) ET `auth.password_reset_completed` présents (`audit_log::find_by_entity`).
    - (b) **token expiré** : token fabriqué directement en DB (`password_reset_tokens::create` avec `expires_at` passé) → POST reset → `400 INVALID_OR_EXPIRED_TOKEN`, mot de passe inchangé.
    - (c) **token réutilisé (double-consume)** : reset OK puis 2e POST avec le même token → `400 INVALID_OR_EXPIRED_TOKEN`.
    - (d) **identifiant inexistant** : POST forgot → `200` ET, après fenêtre de polling, **0 token créé en DB + 0 mail capturé + 0 entrée audit** (anti-énum).
    - (e) **user sans email** : `200`, 0 token, 0 mail, MAIS audit `password_reset_requested` avec `recoverable:false`.
    - (f) **rate-limit** : avec un limiter à seuil bas (DE-4), la (seuil+1)e requête → `429` (et vérifier que le 429 s'applique aussi à reset-password — limiter partagé).
    - (g) **SMTP down** : `MockMailer::failing()` → POST forgot → **`200`** (jamais 500 — oracle DC4) ; le token EST créé en DB (l'échec n'est que l'envoi).
    - (h) **compte inactif** : `200`, 0 mail, audit `recoverable:false` ; ET un token encore valide émis AVANT désactivation → POST reset → `400` (re-check active 17-4c P3).
    - (i) **email dupliqué** : 2 users ACTIFS même email → `200`, 0 token/0 mail (comptage ≠ 1) ; 1 actif + 1 inactif même email → le mail part pour l'actif (retain P4).
    - (j) **username avec `@`** : identifiant `a@b` matchant un username legacy inséré direct en DB → routé lookup email → no-op silencieux `200`.
    - (k) **trim token** : POST reset avec `"  <token>  "` → `200` (trim P6).
    - (l) **VALIDATION_ERROR ne brûle pas le token** : POST reset mdp trop court → `400 VALIDATION_ERROR` → re-POST même token mdp valide → `200` (la validation précède `mark_used`).
    - (m) **feature off** : config par défaut (`forgot_password_enabled=false`) → POST forgot ET reset → `404` (routes non montées).
24. **E2E Playwright** (`frontend/tests/e2e/password-recovery.spec.ts`, backend test-mode feature-on DE-6, `seedTestState` purge les limiters DE-2) couvrant : lien « Mot de passe oublié ? » visible sur `/login` (flag on) ; parcours forgot (saisie identifiant → message générique `forgot-success`) ; parcours reset happy (token injecté DE-1 → nouveaux mdp → `reset-success` → CTA login → login avec le nouveau mdp) ; `/reset-password` sans token → `reset-invalid-link` direct ; token bidon → submit → `reset-invalid-link` + CTA `reset-request-new-link`. `clearAuthStorage` en afterEach (pattern setup.spec.ts).

### Transverses

- **Support test backend strictement gated `test_mode`** (DE-1/DE-2) — aucun nouveau chemin en prod ; `clear_all` est une méthode inerte sans appelant prod.
- Quality gate : backend **serial** (`cargo test --workspace -j1 -- --test-threads=1`, kesh-api touché + DB) + E2E live local (MariaDB up, `PLAYWRIGHT_HOST_PLATFORM_OVERRIDE=ubuntu24.04-x64`).

## Tasks / Subtasks

- [ ] **T-E1** Support test backend (gated `test_mode`) : `RateLimiter::clear_all()` (+ test unitaire) ; purge des 2 limiters dans `seed_handler` ; route `POST /_test/password-reset-token` `{username}` → `{token}` (réutilise `generate_reset_token` + `password_reset_tokens::create`, TTL standard ; 404 si user inconnu — c'est un endpoint de test, pas d'anti-énum). (AC: 24 pré-requis)
- [ ] **T-E2** Suite `crates/kesh-api/tests/password_recovery_e2e.rs` : helpers locaux (`test_config_recovery()` DE-5, `spawn_app_with_state` littéral AppState avec MockMailer + limiter permissif DE-4, `wait_for_mail`/`wait_for_token_count` DE-3, helper création user avec email via repos), puis les 13 cas AC23 a-m. (AC: 23)
- [ ] **T-E3** Spec `frontend/tests/e2e/password-recovery.spec.ts` : 5 scénarios AC24, header de recette backend (env vars DE-6), helper local `injectResetToken(username)` (POST `/_test/password-reset-token` via playwright request, pattern `seedTestState`). (AC: 24)
- [ ] **T-E4** Quality gate : fmt + build + clippy -D + `cargo test --workspace -j1 -- --test-threads=1` (serial, DB up) verts ; run E2E live local de la spec (backend feature-on) vert ; baseline E2E existante non régressée (`npm run test:e2e` complet si le temps le permet, sinon spec nouvelle + setup/auth specs). (AC: transverse)

## Dev Notes

### Ground-truth infra de test (exploration 2026-06-11, 47 lectures)

**Pattern intégration (`auth_e2e.rs:9-157`, `setup_admin_e2e.rs:56-118`) :**
- `#[sqlx::test(migrator = "kesh_db::MIGRATOR")] async fn name(pool: MySqlPool)` — pool + migrations par test, pas de truncate manuel. Exécution CI serial `--test-threads=1`.
- `spawn_app` : listener `127.0.0.1:0`, `axum::serve(listener, app.into_make_service_with_connect_info::<SocketAddr>())` (**ConnectInfo actif**, IP=127.0.0.1 partagée — d'où DE-4 limiter par test), retry TCP avant retour, `TestApp { base_url, client: reqwest::Client }`.
- `setup_admin_e2e.rs:81-91` : **précédent de littéral `AppState { … }`** avec champs custom — la voie pour injecter `mailer: Arc::new(mock.clone())` + `rate_limiter_recovery: Arc::new(RateLimiter::with_thresholds(1000, …))`.
- `Config::from_fields_for_test(11 params)` (`config.rs:382-394`) — défauts : `forgot_password_enabled=false`, `smtp_*=None`, `public_base_url=None` ; champs `pub` mutables après construction (DE-5). Builder `.with_test_mode(bool)` (`config.rs:481-489`).

**MockMailer (`mail/mod.rs:74-107`) :** `CapturedMail { to, reset_url, locale }` ; `MockMailer::new()` / `::failing()` (→ `Err(SmtpSendFailed)`) / `.sent() -> Vec<CapturedMail>` (clone thread-safe, `Arc<Mutex<Vec>>` — cloner le mock AVANT de le mettre dans l'AppState pour garder la poignée de lecture). Extraction : `reset_url.split("token=").nth(1)`.

**Repos pour fixtures/assertions :** `password_reset_tokens::{create(pool,user_id,hash,expires_at), find_valid_by_hash, mark_used}` — token expiré = `create` avec `expires_at` passé + hash connu (`crate::auth::api_key` n'est PAS accessible depuis tests/ ?… SI : `kesh_api::auth::api_key::generate_reset_token()` et `sha256_hex` sont `pub` — vérifier la visibilité du module `auth` dans lib.rs au dev ; sinon générer un token quelconque et stocker `sha256_hex(token)` via une petite copie locale du hash — préférer l'API publique). `refresh_tokens` (assert `revoked_reason`/`revoked_at` via SQL direct), `audit_log::find_by_entity(pool, "user", user_id, limit)` (`audit_log.rs:82-100`).

**E2E Playwright (`playwright.config.ts`, `helpers/test-state.ts`, `test_endpoints.rs:47-51`) :**
- Backend lancé MANUELLEMENT (pas de webServer) : `KESH_TEST_MODE=true KESH_HOST=127.0.0.1 cargo run -p kesh-api` + MariaDB Docker. `workers: 1`. `globalSetup` → `seedTestState('with-company')` fail-fast.
- `seedTestState(preset)` → `POST /api/v1/_test/seed` ; presets `fresh|post-onboarding|with-company|with-data|with-company-no-fy|setup-required`. Le preset `with-company` crée company + admin (`changeme`-style user : vérifier au dev le username/password exact du seed pour le login E2E).
- Router test : `/seed` + `/reset` (`test_endpoints.rs:47-51`), monté si `test_mode` → **ajouter `/password-reset-token` ici** (DE-1).
- ⚠️ Backend E2E feature-on : le fail-fast boot (`from_env`) exige SMTP complet quand `KESH_FEATURE_FORGOT_PASSWORD=true` → fournir `KESH_SMTP_HOST/PORT/USERNAME/PASSWORD/FROM` factices + `KESH_PUBLIC_BASE_URL=http://127.0.0.1` (DE-6). Envois réels échouent en tâche détachée (loggés) — sans impact.
- Run local : `PLAYWRIGHT_HOST_PLATFORM_OVERRIDE=ubuntu24.04-x64` requis (memory Ubuntu 26.04), `cd frontend && npm run test:e2e -- password-recovery` pour la spec seule.

**Pièges connus :**
- La task détachée : ne JAMAIS asserter immédiatement après le 200 (DE-3, polling borné).
- Le limiter recovery compte CHAQUE requête (même les 200) — un test fonctionnel qui enchaîne 6 POSTs sur le même AppState strict se 429 lui-même (DE-4).
- `find_valid_by_hash` filtre `expires_at > NOW(3)` côté MariaDB — fabriquer l'expiration avec une marge large (ex. -1h, pas -1s) pour éviter le skew (defer D3 17-4c).
- E2E : `test.afterEach(clearAuthStorage)` (pattern `setup.spec.ts`) ; ne pas dépendre de l'ordre des specs (workers=1 mais isolation par seed).
- CI principale ne lance PAS les E2E Playwright (Test Locally First : d'autant plus critique de les passer en local avant push).

### References

- [Source: umbrella `17-4-recovery-mot-de-passe.md` — AC23 (a-g) + AC24 Partie E ; backlog étendu h-m hérité des Change Logs 17-4c Pass 1+3 / 17-4d]
- [Source: `17-4c-backend-endpoints.md` + `17-4d-frontend.md` — contrats figés, testids, limitations D1-D4/L-C1..3]
- [Source: crates/kesh-api/tests/auth_e2e.rs:9-157 (spawn_app, ConnectInfo, 42 tests) ; setup_admin_e2e.rs:34-118 (config custom + littéral AppState)]
- [Source: crates/kesh-api/src/mail/mod.rs:74-107 — MockMailer/CapturedMail]
- [Source: crates/kesh-api/src/config.rs:382-489 — from_fields_for_test + with_test_mode]
- [Source: crates/kesh-api/src/lib.rs:72-105 — new_for_tests, build_recovery_rate_limiter]
- [Source: crates/kesh-api/src/routes/test_endpoints.rs:47-91 — router /seed /reset + presets]
- [Source: crates/kesh-db/src/repositories/{password_reset_tokens.rs,refresh_tokens.rs:134,audit_log.rs:82-100}]
- [Source: frontend/playwright.config.ts ; frontend/tests/e2e/helpers/test-state.ts:28-89 ; tests/e2e/setup.spec.ts]
- [Source: docs/testing.md:47-129 — recettes serial + Playwright 2 terminaux]
- [Source: CLAUDE.md §Test Locally First ; memory `reference_playwright_ubuntu26`, `reference_mariadb_docker_dev`]

## Dev Agent Record

### Agent Model Used

(à remplir au dev-story)

### Debug Log References

### Completion Notes List

### File List
