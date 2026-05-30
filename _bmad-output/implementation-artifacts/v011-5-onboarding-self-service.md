# Story v011.5: Onboarding self-service + recovery unifié (Issue #121, absorbe v011-3)

Status: ready-for-dev

<!-- Note: Validation is optional. Run validate-create-story for quality check before dev-story. -->

## Story

As a **nouvel utilisateur qui installe Kesh v0.1.2 sur un host vide**,
I want **créer mon compte administrateur via un formulaire web au 1er démarrage** (sans éditer `.env` au préalable, sans connaître de mot de passe technique),
so that l'expérience d'install soit conforme aux apps self-hosted modernes (Jellyfin, Bitwarden, Sonarr, Vaultwarden) — et que le **même mécanisme `.env`** serve aussi de recovery break-glass si je perds mon mot de passe administrateur.

## Scope

**Severity : amélioration UX install + recovery offline (post-v0.1.1, cible v0.1.2).** Le mécanisme `.env`-bootstrap livré par v011-2 fonctionne mais exige une édition `.env` avant le 1er `docker compose up` — pattern non-standard. v011-5 remplace ce flux par un onboarding self-service au 1er boot **et absorbe le break-glass recovery** (story v011-3 SUPERSEDED) dans un mécanisme unifié.

**Idée centrale (Guy 2026-05-30, design tranché)** : les variables `KESH_ADMIN_USERNAME`/`KESH_ADMIN_PASSWORD` deviennent **double-usage** dont le comportement au boot dépend de l'état DB :
- **DB vide + env non set** → bootstrap crée *uniquement* la company stub. Frontend détecte 423 Locked sur les routes protégées et redirige vers `/setup` (formulaire web).
- **DB vide + env set** → bootstrap crée stub + admin (≡ v011-2 actuel, conservé pour CI/Test/déploiements déclaratifs).
- **DB avec user matching username + hash diffère** → **reset password** (recovery break-glass) + révocation refresh_tokens + audit_log + `tracing::error!` rappelant de retirer les vars post-recovery.
- **DB avec user matching + hash identique** → no-op silencieux + `tracing::warn!` répété « retirer les vars de `.env` ».
- **DB avec user not matching + env set** → no-op + `tracing::warn!` explicite (« no admin matches KESH_ADMIN_USERNAME=X »).
- **DB avec users + env non set** → no-op (régime nominal post-bootstrap).

Pas de flag `KESH_ADMIN_RESET=true` explicite (élimine 1 var ; le no-op-si-hash-identique évite le piège « reset à chaque reboot tant que les vars traînent dans `.env` »).

**Dans le scope :**
- **Backend Rust** :
  - `crates/kesh-api/src/auth/bootstrap.rs` : refactor `ensure_admin_user` selon la matrice 6 cas + helper `revoke_all_refresh_tokens` (réutilisation Story 10-5) + audit_log `admin_break_glass_reset`.
  - `crates/kesh-api/src/routes/setup.rs` (**NEW**) : route ouverte `POST /api/v1/setup/admin` qui accepte `{ username, password }`, refuse si `user_count > 0` (410 Gone — auto-disable), hash le password (Argon2id, réutilise `auth::password::hash_password_async`), crée l'admin sur la company stub, renvoie les cookies HttpOnly session (réutilise le helper du login Story 10-5).
  - `crates/kesh-api/src/lib.rs:417-422` : monter la route `/api/v1/setup/admin` sur `main_router` (public, sans `route_layer(require_auth)`), avec rate-limit IP-based (réutilise `RateLimiter` middleware déjà en place sur `/auth/login`).
  - `crates/kesh-api/src/middleware/auth.rs` ou nouveau middleware : si `users` table vide → routes protégées renvoient `423 Locked` (distinct du 401 Unauthorized). Permet au frontend de distinguer « pas authentifié » de « pas encore setup ».
  - `crates/kesh-api/src/errors.rs` : nouveau variant `AppError::SetupRequired` (423) + variant `AppError::SetupAlreadyComplete` (410). I18n keys associées.

- **Frontend Svelte** :
  - `frontend/src/routes/setup/+page.svelte` (**NEW**) : écran « Bienvenue dans Kesh » avec formulaire `username` (≥ 1 char) + `password` (≥ `KESH_PASSWORD_MIN_LENGTH`, défaut 12) + confirmation password + bouton submit. Validation côté client (longueur, match confirmation). i18n FR/DE/IT/EN (clés `setup-welcome`, `setup-username`, `setup-password`, `setup-password-confirm`, `setup-submit`, `setup-success`, etc.).
  - `frontend/src/routes/setup/+layout.ts` (**NEW**) : route publique (pas de check auth). Si user déjà authentifié → redirect `/`.
  - `frontend/src/lib/app/stores/auth.svelte.ts` : `hydrate()` détecte 423 → set state `setupRequired = true` + redirect `/setup`.
  - `frontend/src/lib/shared/utils/api-client.ts` : interceptor 423 global → redirect `/setup` (cohérent avec 401 → /login existant).
  - `frontend/src/routes/+layout.ts` : si state `setupRequired` → redirect `/setup` au boot.
  - `frontend/src/lib/features/setup/setup.api.ts` (**NEW**) : `setupAdmin(username, password)` wrapper appelant `POST /api/v1/setup/admin`.
  - i18n `crates/kesh-i18n/locales/{fr,de,it,en}-CH/messages.ftl` : ~10 nouvelles clés `setup-*`.

- **Tests** :
  - `crates/kesh-api/src/auth/bootstrap.rs` (unit `#[sqlx::test]`) : **6 cas matrice** + race admin existant.
  - `crates/kesh-api/tests/setup_admin_e2e.rs` (**NEW**) : POST `/setup/admin` happy path → cookies HttpOnly + 200 + redirect onboarding ; POST sur DB avec user existant → 410 Gone ; GET routes protégées DB vide → 423 Locked ; rate-limit IP brute-force.
  - `crates/kesh-api/tests/auth_recovery_e2e.rs` (**NEW**) ou extension `auth_e2e.rs` : recovery path (boot avec env vars + user existant + hash diff → password reset + tokens révoqués + audit log entry + login avec nouveau mdp OK + ancien refresh token rejeté).
  - `frontend/src/routes/setup/+page.svelte.test.ts` ou `setup.svelte.test.ts` (**NEW**) : validation client + submit success/error.
  - `frontend/tests/e2e/setup.spec.ts` (**NEW**) : DB vide + preset `fresh` modifié → écran `/setup` → submit → wizard onboarding → app.
  - `crates/kesh-db/src/test_fixtures.rs` : adapter le preset `fresh` post-v011-5 (ajouter un preset `setup-required` qui ne crée PAS d'admin, ne crée QUE la stub company) OU passer un paramètre booléen `create_admin` à `seed_changeme_user_only`. Préférence : nouveau preset distinct pour éviter de casser les tests existants.

- **Doc & Ops** :
  - `docs/manual/fr/admin-manual.tex` (+ PDF régénéré) :
    - Section « Premier démarrage » (lignes ~785-825 actuelles, déjà réécrites v011-2 commit `d36953a`) : **réécriture complète** pour décrire le flux setup-UI au lieu du `.env`-bootstrap. L'admin du `.env` devient *optionnel* (legacy/CI/Test/déclaratif).
    - **Nouvelle sous-section** « J'ai oublié mon mot de passe administrateur » : procédure step-by-step recovery (1. stop container, 2. setter `KESH_ADMIN_USERNAME=admin` + `KESH_ADMIN_PASSWORD=<nouveau>` dans `.env`, 3. restart, 4. login avec nouveau mdp, 5. changer le mdp via UI ou conserver, 6. retirer les vars de `.env`, 7. restart). Warning explicite sur le risque sécurité de laisser les vars actives.
    - **Warning explicite** dans la section setup-UI : « Avant le 1er démarrage, bloquer l'accès réseau public — qui touche `/api/v1/setup/admin` en premier devient admin. Recommandé : binder loopback `127.0.0.1` ou LAN privé en attendant la création du compte. »
    - KF-035 (#127) partiellement adressée (section onboarding admin du manuel devient ground-truth).
  - `.env.example` : section « Compte admin initial » réécrite pour clarifier le double-usage (vars optionnelles bootstrap + procédure recovery). Variables conservées avec exemples commentés.
  - `CHANGELOG.md` `[0.1.2]` : entrée `Modifié` (mécanisme onboarding `.env` → web setup-UI ; `.env` admin devient optionnel) + entrée `Ajout` (recovery break-glass via `.env` + restart). Procédure migration pour utilisateurs v0.1.1 existants (ils ont déjà un admin → comportement no-op + warning vars à retirer).
  - Story file `v011-4-default-port-80.md` bump status `review → done` (PR #132 mergée 2026-05-30, pattern avoid-parallel-prs).
  - Issue GitHub `#121` (break-glass) fermée au merge via commit message.

**Hors scope :**
- **Multi-user setup** : v011-5 ne couvre QUE la création du *premier* admin (gate `user_count == 0`). La gestion utilisateurs post-setup reste via les routes admin existantes.
- **Password reset email** : pas de flow self-service email/SMS pour les utilisateurs lambda. Recovery uniquement via `.env` (opérateur server-side).
- **Multi-tenant** : `users` table single-tenant v0.1, la matrice 6 cas raisonne sur un seul admin. Multi-admin scenario (ex. plusieurs admins après setup, recovery cible le *premier* matching) : warning + cas de bordure documenté, pas de design profond.
- **CLI `kesh-cli admin reset`** : référencé par l'ancien manuel fictif et la KF-035 — **pas implémenté**. Recovery 100% via `.env` (suffisant pour l'opérateur Docker).
- **Persistance du « state setup-required »** : le frontend lit 423 dynamiquement à chaque boot. Pas de localStorage flag (le state DB est source de vérité).
- **HTTPS forcé avant setup** : v0.1 ne force pas HTTPS (reverse proxy externe en charge). Documentation prévient l'opérateur du risque MITM si setup-UI exposé en HTTP non-loopback.

## Contexte technique (ground-truth post-v011-2/v011-4, vérifié 2026-05-30)

### Backend Rust — `crates/kesh-api/src/auth/bootstrap.rs`

État actuel (post-v011-2, lignes ~40-170) : `ensure_admin_user(pool, config)` lit `company_count` + `user_count`, court-circuite si `user_count > 0`, puis branche selon `company_count` (0 → INSERT stub + admin, >0 → admin sur company existante). Tolérance race `UniqueConstraintViolation` admin → cleanup orphan stub (Story v011-2 Pass 1 code-review). Constantes `STUB_COMPANY_NAME = "(en cours de configuration)"` + `STUB_COMPANY_ADDRESS = "-"` partagées avec `onboarding.rs`.

**Refactor v011-5** :
- Détecter env vars : `let has_admin_env = !config.admin_username.is_empty() && !config.admin_password.is_empty();`. ⚠️ Vérifier que `Config::from_env` accepte des `KESH_ADMIN_*` vides ou absents sans fail-fast (cf. `config.rs:388-405` actuel — peut nécessiter relâcher la validation). Vérifier en spec validate / dev-story.
- 6 branches selon la matrice :
  1. `users` vide + `!has_admin_env` → ne créer QUE la company stub (skip création admin). Log info « setup-required : créer l'admin via POST /api/v1/setup/admin ».
  2. `users` vide + `has_admin_env` → comportement v011-2 actuel (stub + admin créé déclarativement).
  3. `users` non vide + `!has_admin_env` → no-op (régime nominal).
  4. `users` non vide + `has_admin_env` + match `username` + hash identique → no-op + `tracing::warn!` répété « retirer les vars ».
  5. `users` non vide + `has_admin_env` + match `username` + hash diffère → **recovery** : UPDATE password_hash + `refresh_tokens::revoke_all_for_user(pool, user_id, "admin_break_glass_reset")` + `audit_log::create` event `admin_break_glass_reset` + `tracing::error!` « Recovery effectué — RETIRER LES VARS DE .ENV ».
  6. `users` non vide + `has_admin_env` + no match → no-op + `tracing::warn!` « no user matches KESH_ADMIN_USERNAME=<x>, recovery skipped ».

### Backend Rust — `crates/kesh-api/src/routes/setup.rs` (NEW)

Référence pattern : `routes/auth.rs:162` (`fn login`) pour la création de cookies HttpOnly post-création. Réutiliser :
- `auth::password::hash_password_async` pour le hash.
- `kesh_db::repositories::users::create(pool, NewUser { ... })` (helper existant).
- Le helper de set-cookie session (cf. `auth.rs` post-Story 10-5) — extraire en helper public `set_session_cookies(jar, user_id, role, ...)` si pas déjà fait.
- Body request : `SetupAdminRequest { username: String, password: String }` (serde camelCase via `#[serde(rename_all = "camelCase")]`). Validation : `username.trim()` non-vide ; `password.len() >= config.password_min_length` (cf. `Config::password_min_length` Story 10-1 hardening).
- Body response : identique à `LoginResponse` (avec `userId`/`username`/`role`/`expiresIn`).
- Erreurs :
  - `user_count > 0` → `AppError::SetupAlreadyComplete` (410 Gone) avec message i18n « Le compte administrateur a déjà été créé. Cet endpoint est désactivé. ».
  - `companies` vide → `AppError::Internal` (« company stub introuvable au setup — bootstrap a échoué silencieusement »). Théoriquement impossible car bootstrap crée toujours la stub si DB vide, mais défense en profondeur.
  - validation échouée → `AppError::Validation` (400) avec champ précis (username vide, password trop court).

### Backend Rust — `crates/kesh-api/src/lib.rs` (mounting)

Ajouter dans `main_router` (l.417-422, public/no-auth) :
```rust
.route("/api/v1/setup/admin", post(routes::setup::create_admin))
```
Rate-limit : envelopper la route avec le `RateLimiter` existant (5 tentatives/15 min/IP, cohérent `/auth/login`). Vérifier que le `RateLimiter` est appliquable au niveau d'une route individuelle (pas seulement global ; cf. `middleware/rate_limit.rs:35+`).

### Backend Rust — `crates/kesh-api/src/middleware/auth.rs` (423 Locked gate)

Approche minimaliste : dans `require_auth`, avant la vérification JWT, faire un `SELECT EXISTS(SELECT 1 FROM users LIMIT 1)`. Si users vide → retourner `AppError::SetupRequired` (423) au lieu de continuer la vérif JWT (qui retournerait 401 confusément).

**Perf** : query par requête. Acceptable en mode setup-required (transition courte). Alternative : cacher l'état « users vide » dans `AppState` et invalider sur création réussie via setup-admin. Plus complexe ; commencer simple.

### Backend Rust — `crates/kesh-api/src/errors.rs` (variants)

Ajouter :
```rust
#[error(...)] SetupRequired,           // 423 Locked
#[error(...)] SetupAlreadyComplete,    // 410 Gone
```
Mapping HTTP + clés i18n :
- `SetupRequired` → 423 Locked, code `SETUP_REQUIRED`, message FR « Configuration initiale requise. Créer le compte administrateur via /setup. »
- `SetupAlreadyComplete` → 410 Gone, code `SETUP_ALREADY_COMPLETE`, message FR « Le compte administrateur a déjà été créé. ».
- i18n keys dans `crates/kesh-i18n/locales/{fr,de,it,en}-CH/messages.ftl`.

### Frontend Svelte — `frontend/src/routes/setup/`

Nouvelle route SvelteKit (cohérent structure `routes/onboarding/`). `+page.svelte` avec formulaire (utilise `Input` + `Button` shared components — cf. `frontend/src/lib/components/ui/`). Validation côté client : password ≥ 12 chars + confirmation match. Au submit, appel `setup.api.ts::setupAdmin(username, password)`. Success → cookies set (HttpOnly invisible à JS, mais redirect server-side response) + redirect `/onboarding` (le wizard).

`+layout.ts` (route guard) : pas d'auth check. Si user déjà authentifié → redirect `/`. Sinon → afficher.

### Frontend Svelte — `frontend/src/lib/app/stores/auth.svelte.ts`

Le `hydrate()` actuel (Story 10-5) fait un `GET /api/v1/auth/me` au boot pour restaurer l'identité depuis le cookie. v011-5 :
- Si `me()` retourne 423 → set state `setupRequired = true`, ne pas set `_currentUser`.
- Le store expose un getter `isSetupRequired` (boolean).
- `+layout.ts` au boot : si `auth.isSetupRequired` → goto `/setup`.

### Frontend Svelte — `frontend/src/lib/shared/utils/api-client.ts`

Interceptor 423 global (cohérent avec interceptor 401 existant qui redirige vers /login post-Story 10-5). Si réponse 423 → goto `/setup` (sauf si déjà sur `/setup`, pour éviter boucle).

### Tests

**Backend unit `bootstrap.rs` (`#[sqlx::test]`)** — 6 cas + race :
1. `bootstrap_db_empty_no_env_creates_stub_only` (cas 1) : créer la stub, vérifier `users` reste vide, log « setup-required ».
2. `bootstrap_db_empty_with_env_creates_stub_and_admin` (cas 2) : v011-2 actuel renommé/préservé.
3. `bootstrap_users_exist_no_env_noop` (cas 3) : `bootstrap_skips_if_users_already_exist` existant renommé/préservé.
4. `bootstrap_recovery_same_hash_noop` (cas 4) : créer admin avec hash X, set `KESH_ADMIN_PASSWORD` qui produit X, vérifier password_hash inchangé + log warn.
5. `bootstrap_recovery_diff_hash_resets` (cas 5) : créer admin avec hash X, set `KESH_ADMIN_PASSWORD` qui produit Y, vérifier password_hash devient Y + refresh_tokens user révoqués + audit_log event présent.
6. `bootstrap_recovery_no_match_username_warns` (cas 6) : créer admin avec username `alice`, set `KESH_ADMIN_USERNAME=bob` → no-op + log warn.

**Backend intégration** :
- `crates/kesh-api/tests/setup_admin_e2e.rs` (NEW) : preset DB minimal (juste la stub company, pas d'admin), spawn_app, POST /setup/admin avec creds valides → 200 + Set-Cookie ; second POST → 410 ; GET route protégée pré-setup → 423 ; rate-limit IP brute-force.
- `crates/kesh-api/tests/auth_recovery_e2e.rs` (NEW) ou `bootstrap_e2e.rs` : full recovery flow via `cargo run` simulé + restart.

**Frontend** :
- `setup.svelte.test.ts` (vitest) : validation password match, longueur, soumission API mock.
- `tests/e2e/setup.spec.ts` (Playwright) : preset DB vide custom, navigate root → redirect /setup → submit form → land on /onboarding wizard → complete → app.

**Adapter `test_fixtures.rs`** : nouveau preset `setup-required` qui ne crée que la stub company (pas de user). Le preset `fresh` actuel (post-v011-2, crée stub + changeme user) reste pour les tests onboarding existants — `setup-required` est nouveau pour v011-5.

### Doc admin LaTeX (`docs/manual/fr/admin-manual.tex`)

Section actuelle « Premier démarrage » (lignes ~785-825 post-v011-2 commit `d36953a`) décrit le flux `.env`-bootstrap. **Réécriture complète** :
1. Préambule : « Au tout premier démarrage de Kesh sur une base de données vide, l'écran de setup web vous accueille pour créer le compte administrateur. Pas d'édition de `.env` requise. »
2. Procédure step-by-step :
   - Démarrer la stack (`docker compose up -d`).
   - Ouvrir `https://kesh.local` dans un navigateur.
   - L'écran « Bienvenue dans Kesh » apparaît automatiquement.
   - Saisir username + password (≥ 12 chars) + confirmation.
   - Cliquer « Créer le compte administrateur ».
   - Le wizard d'onboarding démarre automatiquement (langue, mode, coords company, banque).
3. Warning sécurité : « ⚠️ Avant le 1er démarrage, bloquer l'accès réseau public — qui touche `/api/v1/setup/admin` en premier devient admin. Recommandé : binder loopback `127.0.0.1` ou LAN privé en attendant la création du compte. »
4. **Nouvelle sous-section** « J'ai oublié mon mot de passe administrateur (recovery break-glass) » :
   - Stopper le container (`docker compose stop kesh-api`).
   - Éditer `.env` : décommenter et renseigner `KESH_ADMIN_USERNAME=admin` (le username de votre compte) et `KESH_ADMIN_PASSWORD=<nouveau-mot-de-passe>` (≥ 12 chars).
   - Redémarrer (`docker compose up -d kesh-api`).
   - Les logs affichent « Recovery effectué — RETIRER LES VARS DE .ENV ».
   - Se connecter avec username + nouveau password.
   - **Retirer les vars `KESH_ADMIN_*` du `.env`** (ou les recommenter) — sinon chaque restart resetera le password (no-op si hash identique, mais warning persistant).
   - Redémarrer une dernière fois pour confirmer le warning a disparu.
5. Warning sécurité recovery : « ⚠️ Tant que `KESH_ADMIN_PASSWORD` est non-vide dans `.env`, toute personne y ayant accès peut reset le mdp. Restreindre `chmod 600 .env`. »

PDF régénéré avec `latexmk -xelatex docs/manual/fr/admin-manual.tex`.

### CHANGELOG `[0.1.2]`

Section déjà créée par v011-4 (CHANGELOG section `## [0.1.2] — Non publié`). v011-5 **ajoute** dans cette section :
- `### Modifié` (extension de l'existant) : « **Onboarding self-service** : le compte administrateur initial se crée désormais via un formulaire web (`/setup`) au 1er démarrage, plus besoin d'éditer `.env` avant `docker compose up`. Les variables `KESH_ADMIN_USERNAME` et `KESH_ADMIN_PASSWORD` deviennent optionnelles et conservent un double-usage : (a) **bootstrap déclaratif** (CI, Test, déploiements automatisés) si renseignées sur DB vide, (b) **recovery break-glass** si un admin existe avec le username configuré mais un mot de passe différent (cf. manuel admin section « J'ai oublié mon mot de passe administrateur »). »
- `### Ajout` : « **Recovery break-glass** : si vous perdez votre mot de passe administrateur, renseigner `KESH_ADMIN_USERNAME`/`KESH_ADMIN_PASSWORD` dans `.env` puis redémarrer le container reset le hash de l'admin matching. Refresh tokens révoqués, audit log entry créé. Procédure complète dans le manuel admin. »
- Migration utilisateurs v0.1.1 existants : « Aucune action requise. Votre admin existe déjà (créé par v0.1.0/v0.1.1 bootstrap). Les variables `KESH_ADMIN_*` de votre `.env` peuvent rester sans effet (no-op si hash identique) ; les retirer fait disparaître le warning de log persistant. »

## Acceptance Criteria

### Backend bootstrap matrice 6 cas (AC #1-7)

- [ ] **AC #1** `crates/kesh-api/src/auth/bootstrap.rs::ensure_admin_user` refactoré pour respecter exactement la matrice 6 cas (cf. Story Scope). Détection `has_admin_env = !username.is_empty() && !password.is_empty()` au début de la fonction.
- [ ] **AC #2** Cas 1 (DB vide + no env) : INSERT company stub uniquement, **pas** d'INSERT user. Log info `setup-required: créer l'admin via POST /api/v1/setup/admin`. Vérifié par test `bootstrap_db_empty_no_env_creates_stub_only`.
- [ ] **AC #3** Cas 2 (DB vide + env set) : comportement v011-2 strictement préservé (stub + admin déclaratif). Vérifié par tests bootstrap existants (renommés/conservés).
- [ ] **AC #4** Cas 5 (recovery, hash diff) : UPDATE password_hash + `refresh_tokens::revoke_all_for_user(pool, user_id, "admin_break_glass_reset")` + `audit_log::create` avec event `admin_break_glass_reset` + `tracing::error!` rappelant de retirer les vars. Vérifié par test `bootstrap_recovery_diff_hash_resets`.
- [ ] **AC #5** Cas 4 (recovery, hash identique) : **no-op silencieux** sur `users.password_hash` (pas d'UPDATE), pas de révocation tokens, pas d'audit log. `tracing::warn!` répété « retirer les vars de .env ». Vérifié par test `bootstrap_recovery_same_hash_noop`. Garantit l'idempotence sur reboots avec `.env` non purgé.
- [ ] **AC #6** Cas 6 (env set, no match username) : no-op + `tracing::warn!` « no user matches KESH_ADMIN_USERNAME=<x>, recovery skipped ». Vérifié par test `bootstrap_recovery_no_match_username_warns`.
- [ ] **AC #7** Tolérance race admin (Story v011-2 Pass 1) préservée : cleanup orphan stub sur `UniqueConstraintViolation` (cas 2 race) conservé.

### Setup endpoint (AC #8-12)

- [ ] **AC #8** Route `POST /api/v1/setup/admin` montée dans `main_router` (`lib.rs:417-422`, public sans `require_auth`), avec rate-limit `RateLimiter` (5 tentatives/15 min/IP, réutilise config existante).
- [ ] **AC #9** Body request `{ username, password }` (serde camelCase). Validation : `username.trim()` non-vide ; `password.len() >= config.password_min_length` (≥ 12 par défaut).
- [ ] **AC #10** Gate `user_count > 0` → `AppError::SetupAlreadyComplete` (410 Gone) avec code `SETUP_ALREADY_COMPLETE`. Vérifié par test intégration.
- [ ] **AC #11** Happy path : hash password (Argon2id), INSERT user `role=Admin` attaché à la company stub existante (SELECT id FROM companies ORDER BY id LIMIT 1), set cookies HttpOnly session (réutilise helper `auth.rs` login), renvoie body `LoginResponse` (userId/username/role/expiresIn). Vérifié par test E2E.
- [ ] **AC #12** Aucun side-effect supplémentaire en cas d'échec : pas de user créé partiellement, pas de cookies set, transaction safe.

### Middleware 423 Locked (AC #13-14)

- [ ] **AC #13** `crates/kesh-api/src/middleware/auth.rs::require_auth` : avant la vérif JWT, si `SELECT EXISTS(SELECT 1 FROM users LIMIT 1)` retourne `false` → `AppError::SetupRequired` (423 Locked) avec code `SETUP_REQUIRED`. Acceptable perf : query par requête en mode setup uniquement, transition courte.
- [ ] **AC #14** Route `/api/v1/setup/admin` elle-même PAS gated par ce middleware (elle est publique). `/health` non gated non plus.

### Frontend setup screen (AC #15-19)

- [ ] **AC #15** Route `/setup` créée (`frontend/src/routes/setup/+page.svelte` + `+layout.ts`). `+layout.ts` route publique sans auth check ; redirige `/` si user déjà authentifié.
- [ ] **AC #16** Formulaire avec champs `username` + `password` + `password-confirm` + submit. Validation client : password ≥ 12 chars (afficher message d'erreur i18n) + match confirmation. Le bouton submit reste désactivé tant que validation invalide.
- [ ] **AC #17** Sur submit : appel `POST /api/v1/setup/admin` via `setup.api.ts`. Success → goto `/onboarding`. Error 410 → afficher « Compte admin déjà créé, redirection... » + goto `/login`. Error 400 (validation backend) → afficher le message d'erreur backend (réutilise pattern existant). Error 429 (rate-limit) → afficher « Trop de tentatives, réessayer dans X minutes ».
- [ ] **AC #18** `auth.svelte.ts::hydrate()` détecte 423 sur `/me` → `_setupRequired = true` (pas d'erreur fatale, juste set state). Le store expose un getter `isSetupRequired`.
- [ ] **AC #19** `api-client.ts` interceptor 423 global → goto `/setup` (sauf si déjà sur `/setup` pour éviter boucle). `+layout.ts` racine au boot : si `auth.isSetupRequired` → goto `/setup`.

### i18n (AC #20)

- [ ] **AC #20** ~10 nouvelles clés `setup-*` dans les 4 locales `crates/kesh-i18n/locales/{fr,de,it,en}-CH/messages.ftl` : `setup-welcome` (titre), `setup-intro` (texte explicatif), `setup-username-label`, `setup-username-placeholder`, `setup-password-label`, `setup-password-min` (`Au moins 12 caractères`), `setup-password-confirm-label`, `setup-password-mismatch`, `setup-submit`, `setup-error-already-complete`, `setup-error-rate-limit`. `npm run lint-i18n-ownership` PASS.

### Tests (AC #21-24)

- [ ] **AC #21** Tests unitaires bootstrap : 6 nouveaux cas matrice (`bootstrap_db_empty_no_env_creates_stub_only`, `bootstrap_db_empty_with_env_creates_stub_and_admin` renommé, `bootstrap_users_exist_no_env_noop`, `bootstrap_recovery_same_hash_noop`, `bootstrap_recovery_diff_hash_resets`, `bootstrap_recovery_no_match_username_warns`). Tests existants v011-2 renommés/conservés. Total ≥ 8 tests bootstrap verts.
- [ ] **AC #22** Test intégration `setup_admin_e2e.rs` : happy path 200 + Set-Cookie ; 410 si user existe ; 423 sur route protégée pré-setup ; 429 rate-limit après N tentatives.
- [ ] **AC #23** Test E2E Playwright `setup.spec.ts` : nouveau preset `setup-required` (stub company seule, pas d'admin) → navigation root → redirect `/setup` → submit form → land on `/onboarding` → wizard prod → app.
- [ ] **AC #24** `test_fixtures.rs` : nouveau preset `setup-required` ajouté ; preset `fresh` (post-v011-2, crée changeme user) **inchangé** pour préserver compat tests existants. Endpoint `_test/seed?preset=setup-required` câblé.

### Doc & CHANGELOG (AC #25-27)

- [ ] **AC #25** `docs/manual/fr/admin-manual.tex` : section « Premier démarrage » réécrite complète (setup-UI + warning bloquage réseau public). Nouvelle sous-section « J'ai oublié mon mot de passe administrateur » (procédure recovery 7 étapes). PDF régénéré (`latexmk -xelatex`). Adresse partiellement KF-035 #127.
- [ ] **AC #26** `.env.example` section « Compte admin initial » réécrite pour clarifier double-usage (optionnel bootstrap déclaratif / obligatoire recovery). Pas de modification fonctionnelle des vars (compat v011-2 préservée).
- [ ] **AC #27** `CHANGELOG.md` `[0.1.2]` : étendre section `### Modifié` avec onboarding self-service + ajouter section `### Ajout` avec recovery break-glass + note migration utilisateurs v0.1.1 existants (no-op + warning vars).

### Quality gate (AC #28-29)

- [ ] **AC #28** Série Test Locally First complète verte : `cargo fmt + clippy --workspace --all-targets -- -D warnings + build + test --workspace -j1 -- --test-threads=1` (DB seedée open FY) ; `npm run check + lint-i18n-ownership + test:unit + build` ; E2E Playwright `setup.spec.ts` PASS avec backend up + preset `setup-required` seedé.
- [ ] **AC #29** Sprint-status : v011-4 bumped `review → done` (PR #132 mergée, pattern avoid-parallel-prs) ; v011-5 `in-progress → review` au commit final dev-story. Story file v011-4 Change Log entry « MERGED PR #132 squash 4c9558b ».

## Tasks / Subtasks

- [ ] **T1 — Bootstrap matrice 6 cas** (AC #1-7)
  - [ ] Refactor `ensure_admin_user` selon la matrice. Détection `has_admin_env`.
  - [ ] Vérifier `Config::from_env` tolère `KESH_ADMIN_*` absents/vides (adapter `config.rs:388-405` si fail-fast actuel).
  - [ ] Renommer/réécrire les 5 tests bootstrap existants pour mapper les 6 cas matrice. Ajouter les 3 cas manquants (recovery diff hash, same hash, no match).
- [ ] **T2 — Setup endpoint** (AC #8-12)
  - [ ] Nouveau fichier `crates/kesh-api/src/routes/setup.rs` avec handler `create_admin`.
  - [ ] Extraire helper `set_session_cookies` depuis `auth.rs::login` si pas déjà public (rendre `pub(crate)`).
  - [ ] Monter la route dans `lib.rs:417-422` (avant `.merge(protected)`) + rate-limit wrapper.
  - [ ] Variants `AppError::SetupAlreadyComplete` + i18n keys (4 locales).
- [ ] **T3 — Middleware 423 Locked** (AC #13-14)
  - [ ] Ajouter le SELECT EXISTS users en début de `require_auth`. Retourner `SetupRequired` si vide.
  - [ ] Variant `AppError::SetupRequired` + i18n keys (4 locales).
  - [ ] Vérifier que `/api/v1/setup/admin` et `/health` ne sont pas gated.
- [ ] **T4 — Frontend setup screen** (AC #15-19, AC #20)
  - [ ] Nouvelle route `routes/setup/+page.svelte` + `+layout.ts`.
  - [ ] Nouveau wrapper API `lib/features/setup/setup.api.ts`.
  - [ ] Extension `auth.svelte.ts::hydrate()` : détecter 423.
  - [ ] Extension `api-client.ts` : interceptor 423 → /setup.
  - [ ] Extension `routes/+layout.ts` : redirect /setup si setupRequired.
  - [ ] 10 nouvelles clés i18n FR/DE/IT/EN.
- [ ] **T5 — Tests** (AC #21-24)
  - [ ] 6 tests unitaires bootstrap matrice + adapter les tests v011-2 existants.
  - [ ] Nouveau `crates/kesh-api/tests/setup_admin_e2e.rs` (4 scénarios : happy / 410 / 423 / 429).
  - [ ] Preset `setup-required` dans `test_fixtures.rs` + câblage `_test/seed`.
  - [ ] Spec E2E Playwright `frontend/tests/e2e/setup.spec.ts`.
  - [ ] Frontend unit test `setup.svelte.test.ts` (validation client).
- [ ] **T6 — Doc admin + CHANGELOG** (AC #25-27)
  - [ ] Réécriture section « Premier démarrage » `admin-manual.tex` (setup-UI + warning bloquage réseau).
  - [ ] Nouvelle sous-section « J'ai oublié mon mot de passe administrateur » (procédure recovery 7 étapes + warning).
  - [ ] PDF régénéré.
  - [ ] `.env.example` section « Compte admin initial » double-usage.
  - [ ] CHANGELOG `[0.1.2]` : extension `Modifié` + nouveau `Ajout`.
- [ ] **T7 — Quality gate + sprint status** (AC #28-29)
  - [ ] Série Test Locally First complète.
  - [ ] Sprint-status : bump v011-4 review→done + v011-5 in-progress→review.
  - [ ] Story file v011-4 Change Log entry « MERGED PR #132 squash 4c9558b ».
  - [ ] Commit + push.

## Dev Notes

### Patterns à respecter (ground-truth code)

- **Constantes stub partagées** (Story v011-2) : `STUB_COMPANY_NAME` / `STUB_COMPANY_ADDRESS` dans `auth/bootstrap.rs:22-23` (pub(crate) const) réutilisées par `onboarding.rs:808-809`. Préserver le pattern.
- **Tolérance race admin** (Story v011-2 Pass 1 patches) : sur `UniqueConstraintViolation` lors de l'INSERT user, si on a créé un stub ce boot (`company_count==0`), DELETE l'orphan stub. Préserver pour le cas 2.
- **Helper revocation tokens** : `refresh_tokens::revoke_all_for_user(pool, user_id, reason: &str) -> Result<u64, DbError>` (Story 1.6 / Story 10-5). Réutiliser tel quel pour le cas 5 recovery. Reason = `"admin_break_glass_reset"`.
- **Audit log create** : `audit_log::create` ou via la helper `NewAuditLogEntry` dans `crates/kesh-db/src/entities/audit_log.rs`. Vérifier la signature exacte au dev-story. Event name = `admin_break_glass_reset`. Details JSON = `{ "username": "<x>", "trigger": "env_vars_present_hash_diff" }`.
- **Cookies HttpOnly** (Story 10-5) : pattern `set_session_cookies(jar, access_token, refresh_token, expires)` dans `auth.rs`. Extraire helper public si nécessaire. **Ne pas réimplémenter** la logique cookies.
- **i18n FR/DE/IT/EN** (CLAUDE.md) : toute clé `setup-*` doit avoir les 4 traductions. `npm run lint-i18n-ownership` enforce le préfixe `setup-*` pour la feature `setup`.

### Sécurité — gate setup ouverte au 1er boot

L'endpoint `POST /api/v1/setup/admin` est **publique sans auth** tant que `user_count == 0`. C'est le compromis classique des apps self-hosted (Jellyfin, Bitwarden). Risques mitigés :

1. **Course "first-to-setup" attacker** : qui touche `/setup/admin` en premier devient admin. Atténuation : (a) warning manuel d'admin de bloquer réseau public avant le 1er boot, (b) rate-limit IP brute-force, (c) auto-disable au 1er succès (410 Gone). **Pas de protection serveur-side automatique** (pas de fenêtre temporelle, pas de token offline). Acceptable v0.1, à reconsidérer v0.2 si retour terrain.
2. **Brute-force du formulaire** : rate-limit IP 5/15min/30min block (cohérent `/auth/login`). À documenter dans le manuel.
3. **MITM si HTTP non-TLS** : v0.1 ne force pas HTTPS (reverse proxy externe en charge). Documenter dans le manuel le risque MITM si setup-UI exposé HTTP non-loopback.

### Sécurité — recovery break-glass (cas 5)

L'endpoint « recovery » n'est pas une route HTTP — c'est un side-effect du **boot avec `.env` vars présentes**. Risques :

1. **Accès à `.env`** : quiconque a accès au fichier `.env` peut reset le mdp admin. Atténuation : `chmod 600 .env` (documenté), warning manuel.
2. **Reset persistant si vars oubliées** : no-op-si-hash-identique évite l'écrasement perpétuel, mais le `tracing::warn!` répété alerte l'opérateur de retirer les vars.
3. **Logs leak password ?** : le `password_hash` est dans la DB, jamais loggé. Le `tracing::error!` recovery loggue seulement le username et le rappel d'action — pas le password en clair. À vérifier dans le code.
4. **Audit log immuable** : l'event `admin_break_glass_reset` est dans `audit_log` (table protégée RESTRICT FK, conservation OLICo 10 ans). Permet investigation post-incident.

### Race condition `user_count` (TOCTOU)

Le `POST /setup/admin` lit `user_count` puis INSERT user. Race théorique : 2 requêtes concurrentes lisent `user_count == 0` simultanément, les deux INSERT — on aurait 2 admins. Atténuation : `uq_users_username` UNIQUE constraint catch le 2e INSERT (UniqueConstraintViolation). Si les 2 requêtes ont des username DIFFÉRENTS, les 2 réussissent — race réelle, mais le 1er successful set la session cookie ; le 2e obtient aussi un succès mais perdre la course UI. Bug acceptable v0.1 (race extrêmement étroite + rate-limit IP réduit la fenêtre).

Alternative robuste : transaction `SELECT ... FOR UPDATE` sur une row sentinelle. Sur-ingénierie pour v0.1.

### Compatibilité v0.1.1 → v0.1.2 (utilisateurs existants)

Les utilisateurs qui ont installé v0.1.1 ont déjà :
- Un admin créé par bootstrap v011-2 (KESH_ADMIN_USERNAME/PASSWORD du `.env`).
- Une company complète (renommée via wizard).
- Potentiellement des données.

Au boot v0.1.2 :
- Cas 3 (users > 0, env présent) si `.env` traîné de v0.1.1 → comportement dépend du hash :
  - Si l'utilisateur n'a pas changé le mdp depuis v0.1.1 → hash identique → cas 4 → no-op + warning « retirer les vars ».
  - Si l'utilisateur a changé le mdp via UI → hash diffère → cas 5 → **RECOVERY déclenché silencieusement, ce qui resetterait le mdp via UI au mdp de `.env` !** 🚨

C'est un piège ! Pour les utilisateurs v0.1.1 qui ont changé leur mdp via UI mais ont gardé `KESH_ADMIN_PASSWORD` dans `.env`, le boot v0.1.2 va resetter leur mdp au mdp de `.env`.

**Mitigation à trancher en spec validate :**
- Option A : ajouter une migration ou un flag DB (`recovery_enabled_at` timestamp ?) pour distinguer « 1er boot post-upgrade » vs « recovery intentionnel ». Complexe.
- Option B : documenter agressivement dans CHANGELOG + warning de boot très visible — l'utilisateur doit RETIRER `KESH_ADMIN_PASSWORD` de `.env` avant d'upgrader v0.1.1→v0.1.2. Pragmatique.
- Option C : ajouter un opt-in `KESH_ADMIN_RECOVERY=true` flag explicite pour activer le cas 5 — revient à la design v011-3 original. Casserait l'unification.

→ Option B retenue par défaut. Question ouverte si Guy préfère A ou C.

### `Config::from_env` — vars optionnelles

Actuellement `crates/kesh-api/src/config.rs:388-405` (post-v011-2) :
```rust
let admin_username = env::var("KESH_ADMIN_USERNAME")
    .map_err(|_| ConfigError::MissingVar("KESH_ADMIN_USERNAME".into()))?;
```
Probable fail-fast sur var manquante. v011-5 doit relâcher : `unwrap_or_default()` ou `Option<String>`. Adapter Config struct + tests config. **Vérifier en dev-story le comportement exact + impact downstream**.

### Frontend hydrate flow — détection 423

Actuel `hydrate()` (Story 10-5) :
```ts
const me = await api.get('/api/v1/auth/me'); // 200 = authentifié, 401 = pas authentifié (= pas logged in)
```
v011-5 ajoute une 3e branche : 423 = setup-required. Le store gère `_currentUser = null` + `_setupRequired = true` au lieu d'erreur fatale. Le boot layout regarde `auth.isSetupRequired` et redirect.

Attention : `/auth/me` est dans `protected` (auth required) → si users vide, le middleware retourne 423 AVANT JWT check. Coherent.

### Migration breaking policy (CLAUDE.md)

v011-5 ne touche **aucune migration** (pas de schema change). Politique P3/P5 → N/A. Aucun audit `docs/migrations-idempotence-audit.md` à ajouter.

### Règle de splitting préventif (CLAUDE.md)

Story touche **~12-15 fichiers** (bootstrap.rs, setup.rs NEW, lib.rs, middleware/auth.rs, errors.rs, config.rs, 4 i18n .ftl, 4 frontend NEW/MODIFIED, 2 tests NEW, test_fixtures.rs, admin-manual.tex, .env.example, CHANGELOG). Au-dessus du seuil > 5 modules. **Cohésion forte** (single feature unifié), pas mécanique find-replace mais logique métier coordonnée.

→ Maintenue en story unique. **Soupape** : si `bmad-create-story validate` boucle > 4 passes sans converger, splitter en v011-5a (backend bootstrap matrice + setup endpoint + middleware 423 + tests backend) / v011-5b (frontend setup screen + i18n + E2E + doc). Frontière nette à la frontière backend/frontend.

### Test Locally First (CLAUDE.md)

- Backend : `cargo fmt + clippy --workspace --all-targets -- -D warnings + build + test --workspace -j1 -- --test-threads=1` (DB seedée open FY pour les tests `test_pool()`).
- Frontend : `npm run check + lint-i18n-ownership + test:unit + build`.
- E2E Playwright : `setup.spec.ts` nouveau, requiert preset `setup-required` côté backend + kesh-api lancé `KESH_TEST_MODE=true`. Suite path-b existante (post-v011-2 fixme) doit rester verte.

### Convention `.env` utilisateur (v0.1.1 → v0.1.2)

Le `.env` du dev (Guy) actuel contient `KESH_ADMIN_USERNAME=admin` + `KESH_ADMIN_PASSWORD=<hash actuel ?>`. Au boot v0.1.2 :
- Si Guy n'a pas changé son mdp via UI → no-op + warning.
- Si Guy a changé via UI → recovery déclenché (cas 5) ! 🚨

**Action manuelle requise avant tag v0.1.2** : Guy doit retirer `KESH_ADMIN_PASSWORD` de son `.env` local (ou le commenter) AVANT le merge de v011-5, pour éviter le piège lors de son prochain `docker compose up`. À mentionner dans la CHANGELOG migration note.

### Questions ouvertes (à trancher en spec validate)

- **Q1 — Compatibilité v0.1.1 → v0.1.2 piège recovery** : option A (flag DB), B (doc + warning agressif, default), C (opt-in flag `KESH_ADMIN_RECOVERY=true`) ?
- **Q2 — Bind loopback obligatoire avant setup** : forcer `KESH_HOST=127.0.0.1` tant que `users` vide (sécurité dur), ou rely on warning manuel uniquement (default actuel) ?
- **Q3 — Rate-limit setup-admin** : 5 tentatives/15 min IP (cohérent /auth/login) ou plus strict (3 tentatives / window plus large) ? L'auto-disable au 1er succès limite la valeur d'un rate-limit strict.
- **Q4 — Audit log details** : minimal `{ username, trigger }` ou inclure aussi `client_ip` + `user_agent` du `.env`-recovery (mais ce sont pas applicables car c'est un boot side-effect, pas une requête HTTP) ?
- **Q5 — Frontend `/setup` accessible directement par URL** : si user navigue `/setup` alors qu'un admin existe déjà → redirect `/login` (proposition) ou afficher un message d'erreur permanent ?

## Change Log

### Create-story (2026-05-30)

Story créée par `bmad-create-story v011-5` (Opus 4.7) à partir du planning epic Hotfix v0.1.1 (section v011-5 ajoutée post-release v0.1.1 par Guy). Analyse ground-truth exhaustive :
- Lecture `bootstrap.rs` post-v011-2 (3-branche actuelle).
- Lecture `lib.rs:417-422` (main_router public).
- Lecture `middleware/auth.rs` (require_auth pattern).
- Lecture `auth.svelte.ts` (hydrate Story 10-5).
- Lecture `auth.rs:162` (login pattern cookies HttpOnly).
- Lecture `refresh_tokens.rs:111-134` (revoke helpers).

Story unifie v011-3 break-glass (SUPERSEDED) dans la matrice 6 cas via le double-usage `KESH_ADMIN_USERNAME/PASSWORD`. 29 ACs sur 7 sections (bootstrap matrice, setup endpoint, middleware 423, frontend, i18n, tests, doc).

5 questions ouvertes (Q1 piège recovery v0.1.1→v0.1.2, Q2 bind loopback, Q3 rate-limit, Q4 audit details, Q5 /setup URL direct) à trancher en spec validate.

Status `ready-for-dev`. Prochaine étape : `bmad-create-story validate v011-5` (boucle Sonnet → Haiku → ... jusqu'à 0 > LOW).

## Dev Agent Record

### Agent Model Used

_(à remplir au dev-story)_

### Debug Log References

_(à remplir au dev-story)_

### Completion Notes List

_(à remplir au dev-story)_

### File List

_(à remplir au dev-story)_
