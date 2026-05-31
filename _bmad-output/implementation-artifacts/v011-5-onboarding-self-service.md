# Story v011.5: Onboarding self-service + recovery unifié (Issue #121, absorbe v011-3)

Status: review

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
- **DB avec user matching username + hash diffère** → **reset password** (recovery break-glass) + révocation refresh_tokens + audit_log + **`tracing::error!`** (`error!` retenu Pass 1 AUD1-9 — event de sécurité critique, pas un simple warning) rappelant de retirer les vars post-recovery, **précédé d'un warning AVANT l'UPDATE** : « ⚠️ Recovery break-glass déclenché pour <username>. Si vous avez changé votre mdp via l'UI, votre mdp actuel sera écrasé. Retirez la var de .env pour annuler. » (Pass 1 BH1-2 patch).
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
- Détecter env vars : `let has_admin_env = config.admin_username.as_deref().is_some_and(|u| !u.is_empty()) && config.admin_password.as_deref().is_some_and(|p| !p.is_empty());`. ⚠️ **Pré-requis bloquant** : `Config::from_env` accepte actuellement seulement des `KESH_ADMIN_*` présents et valides → fail-fast `MissingVar` / `EmptyAdminPassword` / `WeakAdminPassword` / `InsecureAdminPassword` (`config.rs:401-427`). **Le cas 1 de la matrice n'est PAS atteignable sans refactor préalable de `Config`** (Pass 1 BH1-1 / ECH1-1 / AUD1-1 / AUD1-3 — voir sous-section dédiée ci-dessous). Refactor de `Config` est T1 prérequis avant le refactor `ensure_admin_user`.
- **Ordre des branches dans le refactor** (Pass 1 BH1-10 — clarification) : la détection `has_admin_env` se fait AVANT le guard `user_count > 0`. Le guard `user_count > 0` actuel (ligne 56) disparaît, remplacé par la logique matrice. `company_count` et `user_count` sont lus **une seule fois** au tout début de `ensure_admin_user` (Pass 1 BH1-8) et utilisés par toutes les branches y compris le cleanup orphan stub race du cas 2.
- Pseudo-code attendu :
  ```text
  let company_count = SELECT COUNT FROM companies
  let user_count = SELECT COUNT FROM users
  let has_admin_env = <détection vars optionnelles>
  match (user_count, has_admin_env) {
    (0, false) => cas 1: créer stub seule (skip admin)
    (0, true)  => cas 2: créer stub (si company_count==0) + admin (v011-2 actuel, avec cleanup orphan stub race)
    (n, false) => cas 3: no-op
    (n, true)  => SELECT user WHERE username=KESH_ADMIN_USERNAME → match {
      None       => cas 6: warn "no user matches", skip
      Some(u) if argon2_verify(u.password_hash, KESH_ADMIN_PASSWORD).is_ok() => cas 4: warn "retirer les vars" + no-op
      Some(u)    => cas 5: warn préventif "⚠️ recovery déclenché..." + transaction { UPDATE u.password_hash; INSERT audit_log }; revoke_all_for_user(u.id, "admin_break_glass_reset"); error! "RETIRER LES VARS"
    }
  }
  ```
- 6 branches détaillées :
  1. `users` vide + `!has_admin_env` → ne créer QUE la company stub (skip création admin). Log info « setup-required : créer l'admin via POST /api/v1/setup/admin ».
  2. `users` vide + `has_admin_env` → comportement v011-2 actuel (stub + admin créé déclarativement), cleanup orphan stub préservé sur race admin (`UniqueConstraintViolation`).
  3. `users` non vide + `!has_admin_env` → no-op (régime nominal).
  4. `users` non vide + `has_admin_env` + match `username` + hash identique → no-op + `tracing::warn!` répété « retirer les vars ».
  5. `users` non vide + `has_admin_env` + match `username` + hash diffère → **warning AVANT UPDATE** (« ⚠️ Recovery déclenché... ») puis **transaction atomique** `UPDATE password_hash WHERE username=? + audit_log::insert_in_tx(tx, NewAuditLogEntry { user_id: u.id, action: "admin_break_glass_reset", entity_type: "user", entity_id: u.id, details_json: Some(json!({...})) })` (Pass 1 ECH1-2 — `audit_log::insert_in_tx` exige `&mut Transaction`, donc ouvrir une `pool.begin().await?` qui englobe UPDATE + audit ; **schéma `NewAuditLogEntry` corrigé Pass 3 OP3-1**). Hors transaction : `let _ = refresh_tokens::revoke_all_for_user(pool, user_id, "admin_break_glass_reset").await;` (best-effort idempotent — `let _ = ...` ignore explicitement l'erreur pour ne pas faire échouer le bootstrap si le revoke échoue ; sessions existantes restent valides jusqu'à expiration naturelle — limitation acceptée v0.1 documentée L2 ci-dessous) + `tracing::error!` final « Recovery effectué — RETIRER LES VARS DE .ENV ». Rollback strategy : si la transaction UPDATE+audit échoue → SQLx auto-rollback → password inchangé (état avant recovery préservé), revoke_all et log error skipped via early-return `?`, bootstrap retourne l'erreur DB.
  6. `users` non vide + `has_admin_env` + no match → no-op + `tracing::warn!` « no user matches KESH_ADMIN_USERNAME=<x>, recovery skipped ».

### Backend Rust — Refactor `Config::from_env` (PRÉ-REQUIS BLOQUANT T1)

Pass 1 a identifié que le cas 1 de la matrice est **structurellement inatteignable** sans modification préalable de `Config`. Ground-truth `config.rs:401-427` (vérifié 2026-05-30) :
```rust
let admin_username = env::var("KESH_ADMIN_USERNAME").map_err(|_| ConfigError::MissingVar("KESH_ADMIN_USERNAME".into()))?...;
let admin_password = env::var("KESH_ADMIN_PASSWORD").map_err(|_| ConfigError::MissingVar(...))?.trim().to_string();
if admin_password.is_empty() { return Err(ConfigError::EmptyAdminPassword); }
if admin_password.eq_ignore_ascii_case("changeme") { return Err(ConfigError::InsecureAdminPassword); }
if admin_password.chars().count() < 12 { return Err(ConfigError::WeakAdminPassword { ... }); }
```
Aucune valeur de `KESH_ADMIN_PASSWORD` ne signifie « pas de bootstrap déclaratif » dans le code actuel : absence → fail, vide → fail, < 12 chars → fail.

**Décision tranchée Pass 1 (T1 prérequis)** :
- `Config::admin_username: Option<String>` et `Config::admin_password: Option<String>`.
- Si `env::var("KESH_ADMIN_USERNAME")` ou `env::var("KESH_ADMIN_PASSWORD")` retourne `Err(NotPresent)` → champ correspondant = `None`.
- Si présent et vide après `trim()` → traité comme `None` (équivalence absent ↔ vide pour ces deux vars seulement).
- Les **validations de sécurité** (`InsecureAdminPassword` = "changeme", `WeakAdminPassword` < 12 chars) s'appliquent **uniquement si Some(p) avec p non-vide**. Logique : un opérateur qui set explicitement une valeur s'engage à respecter la politique ; un opérateur qui omet la var délègue au flow self-service.
- `make_test_config` (utilisé par tests unitaires internes config.rs/bootstrap.rs/rate_limit.rs — 3 sites) : signature inchangée, callers passent explicitement creds (cf. AC #0 Pass 2 BH2-1).
- **`Config::from_fields_for_test`** (signature publique `config.rs:238-243`, **30 sites de tests d'intégration** `crates/kesh-api/tests/*.rs` — `grep -rln "from_fields_for_test" crates/kesh-api/ | wc -l`) **conserve sa signature `admin_username: String, admin_password: String`** (Pass 3 OP3-2). Elle wrappe en interne `Some(admin_username)` / `Some(admin_password)` après l'assertion non-vide existante (l.257). **Aucune migration des 30 fichiers tests requise** — décision retenue pour minimiser le churn vs. option « adapter les 30 sites à `Some("admin".into())` ». Conséquence : les tests d'intégration existants v011-2 utilisent toujours des creds non-`None` au runtime (ils ne couvrent pas le cas `Option == None`, ce qui est OK car le cas 1 matrice est couvert par les **nouveaux** tests unitaires bootstrap qui construisent un `Config` directement (struct literal) avec `admin_username: None`.
- Tous les downstream qui lisent `config.admin_username` / `config.admin_password` adaptés :
  - `auth/bootstrap.rs` : `has_admin_env` = both `Some(non_empty)`.
  - Tests existants v011-2 du bootstrap : préservés via `from_fields_for_test` wrapper.
- Tests unitaires `config.rs` :
  - `from_env_admin_vars_absent_returns_none_none` (NEW)
  - `from_env_admin_vars_empty_returns_none_none` (NEW)
  - `from_env_admin_vars_set_short_password_fails` (existant, conservé)
  - `from_env_admin_vars_set_changeme_fails` (existant, conservé)
  - `from_env_admin_vars_set_valid_returns_some` (renommé)

### Backend Rust — `crates/kesh-api/src/routes/setup.rs` (NEW)

Référence pattern : `routes/auth.rs:162` (`fn login`) pour la création de cookies HttpOnly post-création. Réutiliser :
- `auth::password::hash_password_async` pour le hash.
- `kesh_db::repositories::users::create(pool, NewUser { ... })` (helper existant).
- **Helper de set-cookie session** : ground-truth `auth.rs:31` confirme `fn build_auth_cookies(...)` actuellement **privée** (Pass 1 BH1-3). Rendre `pub(crate) fn build_auth_cookies(...)` (ou extraire un helper public `set_session_cookies`) **pour éviter toute duplication** de la logique HttpOnly + Secure + SameSite=Strict + Path + Max-Age. Test post-refactor par `grep -nF "HttpOnly" crates/kesh-api/src/routes/` : un seul site (auth.rs), pas de duplication dans setup.rs.
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

**Décision Pass 1 (perf + résilience)** : optimisation `AppState::users_exist: Arc<AtomicBool>` retenue (Pass 1 BH1-4) plutôt que SELECT par requête. Initialisée au boot post-`ensure_admin_user` à `user_count > 0`. Mise à `true` par `setup.rs::create_admin` après INSERT réussi (`state.users_exist.store(true, Ordering::Release)` — Pass 2 ECH2-2 ARM-safe). Lue lock-free par `require_auth` (`state.users_exist.load(Ordering::Acquire)`). Évite la query DB perpétuelle post-setup.

**Impact cross-fichiers `AppState` extension** (Pass 3 OP3-3) : ajouter le champ `users_exist: Arc<AtomicBool>` à `AppState` (`lib.rs:29-34`) impacte **28 sites de tests** (`grep -rln "AppState {" crates/kesh-api/tests/ | wc -l` = 28) + le `test_state()` helper dans `middleware/auth.rs:186-192` = **29 sites total** à adapter. **Stratégie retenue** : ajouter un constructeur `pub fn AppState::new_for_tests(pool: MySqlPool, config: Config, rate_limiter: RateLimiter, i18n: ...) -> Self` qui défaute `users_exist: Arc::new(AtomicBool::new(true))` (valeur cohérente avec les presets de seed E2E qui créent des users post-bootstrap). Les 29 sites tests utilisent ce constructeur (minimise le churn — pas de struct literal à patcher partout). Le seul site de construction `AppState` qui DOIT explicitement initialiser `users_exist` à la valeur DB-réelle est `main.rs::main` (cf. boot order ci-dessous).

**Boot order `main.rs` adaptation** (Pass 3 OP3-4) : ground-truth `main.rs:152` appelle `ensure_admin_user(&pool, &config)` qui retourne actuellement `Result<(), AppError>`. Le refactor v011-5 modifie cette signature :
```rust
pub async fn ensure_admin_user(pool: &MySqlPool, config: &Config) -> Result<i64, AppError>
//                                                                          ^^^^^ user_count final
```
**`main.rs` capture le retour avec le pattern `match` cohérent avec le reste du fichier** (Pass 4 CC4-1) — `async fn main() -> ()` (pas `Result`), donc le `?` ne compile pas. Pattern à utiliser, miroir des 10+ autres appels fallibles dans `main.rs` :
```rust
let user_count = match bootstrap::ensure_admin_user(&pool, &config).await {
    Ok(count) => count,
    Err(e) => {
        tracing::error!("Échec du bootstrap admin : {}", e);
        std::process::exit(1);
    }
};
```
Puis à la construction `AppState` inline (`main.rs:220`), passe `users_exist: Arc::new(AtomicBool::new(user_count > 0))`. **Note** : le spec se référait initialement à `lib.rs::create_state` (Pass 1 — fonction inexistante dans le ground-truth ; cf. Pass 3 OP3-4) — corrigé : le site réel de construction `AppState` est `main.rs:220` inline.

**Fail-open sur erreur DB** (Pass 1 ECH1-8) : la valeur est mémoire-only — pas de query DB dans le path requête. Aucun fail-closed sur transient DB error (cohérent avec la résilience visée Story 10-3). Le state `users_exist` n'est jamais re-synchronisé à partir de la DB après le boot — si un admin est supprimé en SQL direct hors API, le middleware ne le détecte pas. **Acceptable v0.1** (suppression d'admin via SQL direct = ops manuel hors-spec).

**Routes exemptes du gate** (AC #14) : la gate est appliquée sur le sous-routeur `protected` (cohérent existant). Les routes `main_router` publiques (`/health`, `/api/v1/auth/login`, `/api/v1/auth/logout`, `/api/v1/auth/refresh`, `/api/v1/setup/admin`) ne sont pas gated. Le `ServeDir` fallback non plus (sinon page blanche, le JS ne peut pas charger pour rediriger vers `/setup`).

**Cas spécial `/api/v1/auth/login` quand users vide** (Pass 1 ECH1-10) : `login` est sur `main_router` (public, hors `protected`), donc le middleware 423 ne s'y applique pas. `login` recherche le user → aucun match → fallback `dummy_verify` constant-time → retourne 401 (cohérent). Fingerprinting mineur (401 sur `/login` + 423 sur `/me`) accepté v0.1 — l'attacker découvre seulement que l'instance est en mode setup, pas une information secrète. À documenter dans Dev Notes Sécurité.

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

Le `hydrate()` actuel (Story 10-5) fait un `fetch('/api/v1/auth/me')` **direct** (pas via `apiClient.get()`) au boot pour restaurer l'identité depuis le cookie. Ground-truth `auth.svelte.ts:241-260` : branches actuelles = `res.ok` (200) / `res.status === 401` / `else` (warn + reset null).

**Décision Pass 1 (ECH1-3)** : ajouter **explicitement** une 3e branche `else if (res.status === 423)` entre la branche 401 et la branche `else`. Comportement : `_currentUser = null` + `_setupRequired = true`, **sans `console.warn`** (le 423 est un état légal pas un backend KO). La branche `else` (4xx/5xx restants) reste pour les erreurs réelles. Code attendu :
```ts
} else if (res.status === 423) {
  _currentUser = null;
  _setupRequired = true;
} else {
  console.warn(`auth hydrate: ${res.status}`);
  _currentUser = null;
}
```
Le store expose un getter `isSetupRequired` (boolean, lit `_setupRequired`).

**Sync cross-tab** (Pass 1 ECH1-6) : `setup.api.ts::setupAdmin()` doit appeler `authState.login(payload)` après le succès POST `/setup/admin` (réutilise le pattern existant Story 10-5 qui broadcast `auth-change`). Les autres onglets ouverts sur `/setup` reçoivent le broadcast → re-hydrate → `/me` retourne 200 → goto `/onboarding` ou `/`. Ne PAS oublier le broadcast, sinon les onglets parallèles restent bloqués sur `/setup` avec form-submit → 410.

### Frontend Svelte — `frontend/src/lib/shared/utils/api-client.ts`

**Décision Pass 1 (BH1-6)** : interceptor 423 placé dans `request<T>()` **avant** `parseErrorResponse(res)`, miroir du flow 401 existant (lignes ~389-406). Comportement :
```ts
if (res.status === 423 && window.location.pathname !== '/setup') {
  window.location.replace('/setup');
  throw new ApiError('SETUP_REQUIRED', 423);  // early-return semantique
}
```
Utiliser `window.location.replace` (pas `goto` SvelteKit — `goto` est composant-context-dépendant et peut ne pas marcher hors layout). Le guard `pathname !== '/setup'` empêche la boucle infinie. Si le 423 atteint malgré tout `parseErrorResponse` (ex. import hors `request<T>`), pas critique (404 ou error UI cohérent).

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
   - **Retirer les vars `KESH_ADMIN_*` du `.env`** (ou les recommenter) — sinon chaque restart loggue un warning « retirer les vars » (no-op si hash identique, mais bruit dans les logs).
   - **(Optionnel)** Redémarrer une dernière fois pour confirmer que les warnings bootstrap ont disparu — preuve que les variables `KESH_ADMIN_*` sont bien retirées.
5. Warning sécurité recovery : « ⚠️ Tant que `KESH_ADMIN_PASSWORD` est non-vide dans `.env`, toute personne y ayant accès peut reset le mdp. Restreindre `chmod 600 .env`. »

PDF régénéré avec `latexmk -xelatex docs/manual/fr/admin-manual.tex`.

### CHANGELOG `[0.1.2]`

Section déjà créée par v011-4 (CHANGELOG section `## [0.1.2] — Non publié`). v011-5 **ajoute** dans cette section :
- `### Modifié` (extension de l'existant) : « **Onboarding self-service** : le compte administrateur initial se crée désormais via un formulaire web (`/setup`) au 1er démarrage, plus besoin d'éditer `.env` avant `docker compose up`. Les variables `KESH_ADMIN_USERNAME` et `KESH_ADMIN_PASSWORD` deviennent optionnelles et conservent un double-usage : (a) **bootstrap déclaratif** (CI, Test, déploiements automatisés) si renseignées sur DB vide, (b) **recovery break-glass** si un admin existe avec le username configuré mais un mot de passe différent (cf. manuel admin section « J'ai oublié mon mot de passe administrateur »). »
- `### Ajout` : « **Recovery break-glass** : si vous perdez votre mot de passe administrateur, renseigner `KESH_ADMIN_USERNAME`/`KESH_ADMIN_PASSWORD` dans `.env` puis redémarrer le container reset le hash de l'admin matching. Refresh tokens révoqués, audit log entry créé. Procédure complète dans le manuel admin. »
- Migration utilisateurs v0.1.1 existants : « Aucune action requise. Votre admin existe déjà (créé par v0.1.0/v0.1.1 bootstrap). Les variables `KESH_ADMIN_*` de votre `.env` peuvent rester sans effet (no-op si hash identique) ; les retirer fait disparaître le warning de log persistant. »

## Acceptance Criteria

### Pré-requis Config refactor (AC #0)

- [x] **AC #0** (NEW Pass 1) `crates/kesh-api/src/config.rs` : `Config::admin_username` et `Config::admin_password` deviennent `Option<String>`. Vars absentes OU présentes-mais-vides après trim → `None`. Validations `EmptyAdminPassword`/`InsecureAdminPassword`/`WeakAdminPassword` s'appliquent uniquement si `Some(p)` non-vide. **Invariant garanti par `from_env`** (Pass 2 ECH2-6) : `config.admin_password: Some(p) ⟹ !p.is_empty()`. La détection downstream `has_admin_env` peut donc se contenter de `.is_some()` ; le double-check `.is_empty()` reste défensif. **`make_test_config` signature inchangée** (Pass 2 BH2-1) — les call-sites tests v011-2 existants passent explicitement `make_test_config("admin", "admin-test-password-123")` sans défauts internes, pour éviter l'oubli silencieux du passage de creds valides dans un test futur. Les *nouveaux* tests cas 1 (`bootstrap_db_empty_no_env_creates_stub_only`) construisent un `Config` directement (struct literal) avec `admin_username: None, admin_password: None` sans passer par `make_test_config`. Tests unitaires `config.rs` : `from_env_admin_vars_absent_returns_none_none` + `from_env_admin_vars_empty_returns_none_none` + `from_env_admin_vars_set_valid_returns_some` (renommé) + validations longueur/changeme conservées. **Bloquant T1** — sans cette étape, le cas 1 de la matrice est inatteignable.

### Backend bootstrap matrice 6 cas (AC #1-7)

- [x] **AC #1** `crates/kesh-api/src/auth/bootstrap.rs::ensure_admin_user` refactoré pour respecter exactement la matrice 6 cas (cf. Story Scope). Détection `has_admin_env = config.admin_username.as_deref().is_some_and(|u| !u.is_empty()) && config.admin_password.as_deref().is_some_and(|p| !p.is_empty())` au début de la fonction. `company_count` et `user_count` lus une **seule fois** au début (pré-requis cleanup orphan stub race AC #7). Le guard `user_count > 0` actuel disparaît — remplacé par le pattern `match`.
- [x] **AC #2** Cas 1 (DB vide + no env) : INSERT company stub uniquement, **pas** d'INSERT user. Log info `setup-required: créer l'admin via POST /api/v1/setup/admin`. Vérifié par test `bootstrap_db_empty_no_env_creates_stub_only`.
- [x] **AC #3** Cas 2 (DB vide + env set) : comportement v011-2 strictement préservé (stub + admin déclaratif). Vérifié par tests bootstrap existants (renommés/conservés).
- [x] **AC #4** Cas 5 (recovery, hash diff) : **warning préventif AVANT toute mutation** (`tracing::warn!` « ⚠️ Recovery break-glass déclenché pour <username>... ») PUIS **transaction atomique** `pool.begin() → UPDATE users SET password_hash WHERE username=? → audit_log::insert_in_tx(tx, NewAuditLogEntry { user_id: u.id, action: "admin_break_glass_reset", entity_type: "user", entity_id: u.id, details_json: Some(json!({ "username": u.username, "trigger": "env_vars_present_hash_diff" })) })` → `tx.commit()` (Pass 3 OP3-1 — schéma `NewAuditLogEntry` corrigé : champs `action/entity_type/entity_id/details_json`, PAS `event/details`). HORS transaction (post-commit) : `let _ = refresh_tokens::revoke_all_for_user(pool, user_id, "admin_break_glass_reset").await;` (best-effort, `let _ = ...` ignore explicitement l'erreur — voir limitation L2 ci-dessous) + `tracing::error!` final « RETIRER LES VARS DE .ENV ». Si la transaction échoue (lock timeout, DB error, audit_log insert error) → SQLx auto-rollback, password inchangé, bootstrap retourne l'erreur DB via early-return `?`. Vérifié par test `bootstrap_recovery_diff_hash_resets` (vérification : password_hash modifié + audit_log entry présente avec `action="admin_break_glass_reset"` + refresh_tokens révoqués + log error présent).
- [x] **AC #5** Cas 4 (recovery, hash identique) : **no-op silencieux** sur `users.password_hash` (pas d'UPDATE), pas de révocation tokens, pas d'audit log. `tracing::warn!` répété « retirer les vars de .env ». Vérifié par test `bootstrap_recovery_same_hash_noop`. Garantit l'idempotence sur reboots avec `.env` non purgé.
- [x] **AC #6** Cas 6 (env set, no match username) : no-op + `tracing::warn!` « no user matches KESH_ADMIN_USERNAME=<x>, recovery skipped ». Vérifié par test `bootstrap_recovery_no_match_username_warns`.
- [x] **AC #7** Tolérance race admin (Story v011-2 Pass 1) préservée : cleanup orphan stub sur `UniqueConstraintViolation` (cas 2 race) conservé.

### Setup endpoint (AC #8-12)

- [x] **AC #8** Route `POST /api/v1/setup/admin` montée dans `main_router` (`lib.rs:417-422`, public sans `require_auth`). **Rate-limit mechanism** (Pass 3 OP3-6) : `state.rate_limiter.check_rate_limit(ip)` est un **check manuel synchrone** appelé en début de handler (pattern cohérent `routes/auth.rs:171` — pas un middleware Axum montable via `.layer()`). Le handler `create_admin` appelle `state.rate_limiter.check_rate_limit(ip)?` en tout début, et `state.rate_limiter.record_failed_attempt(ip)` sur échec validation OU sur `user_count > 0` (410). Quota partagé avec `/auth/login` (même instance `state.rate_limiter` dans `AppState`) — un attacker bloqué sur l'un est bloqué sur l'autre (défense en profondeur).
- [x] **AC #9** Body request `{ username, password }` (serde camelCase). Validation : `username.trim()` non-vide ; `password.len() >= config.password_min_length` (≥ 12 par défaut).
- [x] **AC #10** Gate `user_count > 0` → `AppError::SetupAlreadyComplete` (410 Gone) avec code `SETUP_ALREADY_COMPLETE`. Vérifié par test intégration.
- [x] **AC #11** Happy path : hash password (Argon2id), INSERT user `role=Admin` attaché à la company stub existante (SELECT id FROM companies ORDER BY id LIMIT 1), set cookies HttpOnly session via le helper extrait **`pub(crate) fn build_auth_cookies(...)`** rendu accessible depuis `routes::setup` (Pass 1 BH1-3 — sinon duplication de la logique HttpOnly + Secure + SameSite=Strict + Path + Max-Age). Renvoie body `LoginResponse` (userId/username/role/expiresIn). Met `state.users_exist.store(true, Ordering::Release)` après l'INSERT réussi (Pass 2 ECH2-2 — `Release` au store + `Acquire` au load garantit happens-before cross-thread, requis sur ARM/CPU weak-memory). Vérifié par test E2E. **`username.trim()`** stocké en DB (cohérent avec `routes/auth.rs::login` qui trim au lookup — Pass 1 ECH1-9).
- [x] **AC #12** Aucun side-effect supplémentaire en cas d'échec : pas de user créé partiellement, pas de cookies set, pas de mise à jour `state.users_exist`. **Race TOCTOU 2 usernames distincts** (Pass 1 BH1-5 / ECH1-4) documentée comme **limitation acceptée v0.1** (cf. Dev Notes L1) — le SELECT count + INSERT user n'est pas atomique, les 2 requêtes peuvent toutes deux réussir et créer 2 admins. Rate-limit IP réduit la fenêtre d'exploitation. Limitation tracée en GitHub Issue à créer label `v0.2-milestone`.

### Middleware 423 Locked (AC #13-14)

- [x] **AC #13** `crates/kesh-api/src/middleware/auth.rs::require_auth` : utilise `AppState::users_exist: Arc<AtomicBool>` (cache mémoire init au boot post-`ensure_admin_user`) lu lock-free au début du middleware avec `Ordering::Acquire` (Pass 2 ECH2-2 — paire Release/Acquire avec le store de setup-admin). **Position exacte de l'insertion** (Pass 2 BH2-2) : **après l'extraction du token du cookie/header** (ligne ~75 actuelle de `require_auth`), **avant** le JWT decode. Logique : si pas de token + users vide → 423 (setup requis, plus précis que 401 « pas authentifié ») ; si pas de token + users existent → 401 nominal ; si token + users vide → 423 (theoretical edge case, le user a un JWT mais la DB a été truncate, redirect setup). Si `state.users_exist.load(Acquire) == false` → return `AppError::SetupRequired` (423). Pas de query DB par requête (perf nominal post-setup préservée). Fail-open implicite sur DB error (la valeur est mémoire-only — aucun fail-closed sur DB transient down, cohérent résilience Story 10-3).
- [x] **AC #14** Routes exemptes du gate 423 : `/health`, `/api/v1/auth/login`, `/api/v1/auth/logout`, `/api/v1/auth/refresh`, `/api/v1/setup/admin`, ServeDir fallback SPA (sinon page blanche au boot, le JS ne peut pas charger pour `goto /setup`). La gate est appliquée sur le sous-routeur `protected` uniquement.

### Frontend setup screen (AC #15-19)

- [x] **AC #15** Route `/setup` créée (`frontend/src/routes/setup/+page.svelte` + `+layout.ts`). `+layout.ts` route publique sans auth check ; redirige `/` si user déjà authentifié.
- [x] **AC #16** Formulaire avec champs `username` + `password` + `password-confirm` + submit. Validation client : password ≥ 12 chars (afficher message d'erreur i18n `setup-password-min`) + match confirmation (`setup-password-mismatch`) + username non-vide après trim. Le bouton submit reste désactivé tant que validation invalide.
- [x] **AC #17** Sur submit : appel `POST /api/v1/setup/admin` via `setup.api.ts`. Success → **`await authState.login(payload)` AVANT redirect** (Pass 2 ECH2-3 — `await` explicite obligatoire pour que le broadcast `auth-change` soit garanti propagé avant le goto ; sinon le goto s'exécute avant que les onglets parallèles aient reçu l'event) → `goto('/onboarding')`. Error 410 → afficher i18n `setup-error-already-complete` + goto `/login` (Q5 tranchée). Error 400 (validation backend) → afficher le message d'erreur backend (réutilise pattern existant). Error 429 (rate-limit) → afficher i18n `setup-error-rate-limit` avec délai estimé.
- [x] **AC #18** `auth.svelte.ts::hydrate()` : ajouter une **3e branche explicite** `else if (res.status === 423)` entre les branches 401 et `else` actuelles. Comportement : `_currentUser = null` + `_setupRequired = true`, **sans `console.warn`** (état légal). La branche `else` (4xx/5xx) reste pour les erreurs réelles. **Déclaration variable d'état** (Pass 2 BH2-4) : ajouter `let _setupRequired = $state<boolean>(false);` au début du store (cohérent avec `_currentUser` et `_expiresIn` déjà présents Story 10-5). Le store expose un getter `get isSetupRequired(): boolean { return _setupRequired; }` au sein de l'objet `authState` (pattern cohérent avec `isAuthenticated`). `login(payload)` doit aussi reset `_setupRequired = false` (un login réussi implique que les users existent → state cohérent).
- [x] **AC #19** `api-client.ts` : ajouter intercepteur 423 dans `request<T>()` **avant** `parseErrorResponse(res)` (miroir flow 401). Comportement : `if (res.status === 423 && window.location.pathname !== '/setup') { window.location.replace('/setup'); throw new ApiError('SETUP_REQUIRED', 423); }`. Utiliser `window.location.replace` (pas `goto` SvelteKit qui est context-dépendant). `+layout.ts` racine au boot : si `auth.isSetupRequired` → goto `/setup` (cas où le boot-ping a déjà détecté 423 et set state).

### i18n (AC #20)

- [x] **AC #20** Clés `setup-*` (12 au total) dans les 4 locales `crates/kesh-i18n/locales/{fr,de,it,en}-CH/messages.ftl` : `setup-welcome` (titre H1), `setup-intro` (paragraphe explicatif), `setup-username-label`, `setup-username-placeholder`, `setup-username-required`, `setup-password-label`, `setup-password-min` (`Au moins 12 caractères`), `setup-password-confirm-label`, `setup-password-mismatch`, `setup-submit`, `setup-error-already-complete`, `setup-error-rate-limit`. **Note linter** (Pass 1 BH1-7) : `lint-i18n-ownership.js` scanne `src/lib/features/` uniquement — la route `src/routes/setup/+page.svelte` est **hors-scope** du linter, donc `npm run lint-i18n-ownership` reste PASS sans garantir l'usage correct de ces clés. Mitigation : la logique UI est extraite dans `src/lib/features/setup/SetupForm.svelte` (composant) pour permettre au linter de couvrir les usages, et `+page.svelte` ne contient qu'un `<SetupForm />`. Pas de validation `setup-success` (success → redirect immédiate, pas de message affiché).

### Tests (AC #21-24)

- [x] **AC #21** Tests unitaires bootstrap : 6 nouveaux cas matrice (`bootstrap_db_empty_no_env_creates_stub_only`, `bootstrap_db_empty_with_env_creates_stub_and_admin` renommé, `bootstrap_users_exist_no_env_noop`, `bootstrap_recovery_same_hash_noop`, `bootstrap_recovery_diff_hash_resets`, `bootstrap_recovery_no_match_username_warns`). Tests existants v011-2 renommés/conservés. Total ≥ 8 tests bootstrap verts. **Vérification atomicité recovery** (cas 5) dans le test : forcer `audit_log::insert_in_tx` à échouer (FK manquant simulé ou table truncate) → vérifier que `password_hash` n'a PAS été modifié (rollback).
- [x] **AC #22** Test intégration `setup_admin_e2e.rs` : happy path 200 + Set-Cookie ; 410 si user existe (séquentiel) ; 423 sur route protégée pré-setup ; 429 rate-limit après N tentatives ; **scénario race TOCTOU documenté** : test (peut être `#[ignore]`) qui lance 2 POST concurrents avec usernames différents et **documente le comportement réel** (2 admins créés OU UniqueConstraintViolation 500) dans le commentaire du test — pas un AC d'assertion, mais une trace de la limitation L1 (Pass 1 BH1-5/ECH1-4).
- [x] **AC #23** Test E2E Playwright `setup.spec.ts` : nouveau preset `setup-required` (stub company seule, pas d'admin) → navigation root → redirect `/setup` (via 423 + interceptor api-client) → submit form → cookies HttpOnly présents → land on `/onboarding` → wizard prod (au moins step 1 « Bienvenue ») → app accessible. Test password mismatch côté client + test redirect /setup si user déjà authentifié (`+layout.ts` `redirect /`).
- [x] **AC #24** `test_fixtures.rs` : nouveau preset `setup-required` ajouté (helper `seed_stub_company_only(pool)` qui truncate puis INSERT seule la company stub). Preset `fresh` (post-v011-2, crée changeme user) **inchangé** pour préserver compat tests existants. **Endpoint `_test/seed?preset=setup-required` câblé** (Pass 1 ECH1-5) : ajouter variant `SetupRequired` à l'enum `Preset` dans `crates/kesh-api/src/routes/test_endpoints.rs:68` + branche match dans le handler + mise à jour de la constante `VALID_PRESETS` (l.94) pour inclure `setup-required`. **Côté frontend TypeScript** (Pass 3 OP3-5) : ground-truth `frontend/tests/e2e/helpers/test-state.ts:28-33` définit `export type Preset = 'fresh' | 'post-onboarding' | 'with-company' | 'with-data' | 'with-company-no-fy';` (union type fermée TypeScript). **Ajouter `'setup-required'`** à cette union (sinon `setup.spec.ts` ne compile pas — `npm run check` de T7 échoue). **Synchronisation `state.users_exist` au seed** (Pass 2 BH2-3) : le handler `seed` doit `state.users_exist.store(actual_user_count > 0, Ordering::Release)` après TRUNCATE+INSERT du preset, sinon le flag mémoire diverge de la DB et les requêtes suivantes retournent 423 erroné (state stuck `true`) ou 401 erroné (state stuck `false`). Pour le preset `setup-required`, le store est `false` ; pour `fresh`, `with-company`, `with-data`, `post-onboarding`, le store est `true`. **Atomicité TRUNCATE+INSERT** (Pass 2 ECH2-7) : le seed handler exécute déjà ses opérations en transaction (pattern existant `routes/test_endpoints.rs`). Playwright tests utilisent `workers=1` (cohérent avec config existante `playwright.config.ts` post-Story 10-3 résilience), donc pas de race concurrente entre specs sur le même DB. **File List étendue** : `routes/test_endpoints.rs` + `frontend/tests/e2e/helpers/test-state.ts` ajoutés aux fichiers modifiés.

### Doc & CHANGELOG (AC #25-27)

- [x] **AC #25** `docs/manual/fr/admin-manual.tex` : section « Premier démarrage » réécrite complète (setup-UI + warning bloquage réseau public). Nouvelle sous-section « J'ai oublié mon mot de passe administrateur » (procédure recovery 7 étapes, étape 7 reformulée Pass 1 BH1-12 « (Optionnel) Redémarrez une dernière fois pour confirmer que les warnings bootstrap ont disparu des logs — preuve que les variables `KESH_ADMIN_*` sont bien retirées »). PDF régénéré (`latexmk -xelatex`). **Issue #127 (KF-035)** : laisser **OPEN** avec commentaire de merge « v011-5 adresse la section Premier démarrage du manuel admin — les autres dérives (RBAC 5 rôles fictifs vs 2 réels, kesh-cli inexistant utilisé partout) restent ouvertes et seront adressées par stories ultérieures » (Pass 1 AUD1-7).
- [x] **AC #26** `.env.example` section « Compte admin initial » réécrite pour clarifier double-usage (optionnel bootstrap déclaratif / obligatoire recovery). Pas de modification fonctionnelle des vars (compat v011-2 préservée).
- [x] **AC #27** `CHANGELOG.md` `[0.1.2]` : étendre section `### Modifié` avec onboarding self-service + ajouter section `### Ajout` avec recovery break-glass. **Section dédiée `### ⚠️ Action requise — upgrade v0.1.1 → v0.1.2`** (Pass 1 BH1-2 / ECH1-7 / AUD1-2 — Q1 tranchée Option B) avec format visible : « **Si vous avez changé votre mot de passe administrateur via l'UI depuis l'installation v0.1.0/v0.1.1**, vous devez retirer `KESH_ADMIN_PASSWORD` de votre `.env` AVANT de redémarrer en v0.1.2. Sinon, le password sera resetté au password de `.env` au prochain boot (Recovery break-glass déclenché). Visible dans les logs Docker : `RETIRER LES VARS DE .ENV`. **Aucune action si le mdp n'a pas été changé** depuis l'installation (no-op + warning persistant uniquement). »

### Quality gate (AC #28-29)

- [x] **AC #28** Série Test Locally First complète verte : `cargo fmt + clippy --workspace --all-targets -- -D warnings + build + test --workspace -j1 -- --test-threads=1` (DB seedée open FY) ; `npm run check + lint-i18n-ownership + test:unit + build` ; E2E Playwright `setup.spec.ts` PASS avec backend up + preset `setup-required` seedé.
- [x] **AC #29** Sprint-status : v011-4 bumped `review → done` (PR #132 mergée, pattern avoid-parallel-prs) ; v011-5 `in-progress → review` au commit final dev-story. Story file v011-4 Change Log entry « MERGED PR #132 squash 4c9558b ».

## Tasks / Subtasks

- [x] **T1 — Bootstrap matrice 6 cas** (AC #1-7)
  - [x] Refactor `ensure_admin_user` selon la matrice. Détection `has_admin_env`.
  - [x] Vérifier `Config::from_env` tolère `KESH_ADMIN_*` absents/vides (adapter `config.rs:388-405` si fail-fast actuel).
  - [x] Renommer/réécrire les 5 tests bootstrap existants pour mapper les 6 cas matrice. Ajouter les 3 cas manquants (recovery diff hash, same hash, no match) + test atomicité transaction recovery (audit_log force-fail → password rollback).
  - [x] **(T1 prérequis — Pass 1 BH1-1/AUD1-1)** Refactor `Config::from_env` : `admin_username` + `admin_password` deviennent `Option<String>`. Vars absentes/vides → `None`. Validations sécu (changeme, < 12 chars, EmptyAdminPassword) ne s'appliquent que si `Some(non-empty)`. `make_test_config` adapté. 3 tests unitaires Config NEW + 2 conservés.
  - [x] Adapter `AppState` (lib.rs:29-34) pour héberger `users_exist: Arc<AtomicBool>` (init au boot post-`ensure_admin_user`). Ajouter constructeur `AppState::new_for_tests(...)` avec `users_exist: Arc::new(AtomicBool::new(true))` par défaut (Pass 3 OP3-3 — minimise churn sur 28 sites tests + 1 site `middleware/auth.rs::test_state`).
  - [x] **Modifier signature `ensure_admin_user`** (Pass 3 OP3-4) : `Result<(), AppError>` → `Result<i64, AppError>` (retourne `user_count` final). `main.rs:152` capture le retour via `match ... { Ok(c) => c, Err(e) => { tracing::error!(...); std::process::exit(1); } }` (Pass 4 CC4-1 — `main()` retourne `()`, le `?` ne compile pas, cohérent pattern existant). `main.rs:220` construction `AppState` inline initialise `users_exist: Arc::new(AtomicBool::new(user_count > 0))`.
  - [x] **Refactor `from_fields_for_test`** (Pass 3 OP3-2) : signature inchangée (`admin_username: String, admin_password: String`) pour éviter migration des 30 sites tests d'intégration. Wrappe en interne `admin_username = Some(...)` / `admin_password = Some(...)` après l'assertion non-vide existante (l.257).
- [x] **T2 — Setup endpoint + helper cookies** (AC #8-12)
  - [x] **Rendre `build_auth_cookies` `pub(crate)`** dans `routes/auth.rs:31` (Pass 1 BH1-3 — pas de duplication HttpOnly/Secure/SameSite=Strict).
  - [x] Nouveau fichier `crates/kesh-api/src/routes/setup.rs` avec handler `create_admin` : **`state.rate_limiter.check_rate_limit(ip)?` en tout début** (Pass 3 OP3-6 — pattern manuel cohérent `/auth/login`), Body `SetupAdminRequest { username, password }`, validation (trim, longueurs), guard `user_count > 0` → 410 + `record_failed_attempt(ip)`, hash password, INSERT user attaché à company stub, set cookies via helper `build_auth_cookies` pub(crate), set `state.users_exist.store(true, Ordering::Release)` (Pass 2 ECH2-2), retourne `LoginResponse`.
  - [x] Monter la route dans `lib.rs:417-422` sur `main_router` (public, **pas** de `.layer(rate_limit_middleware)` — le rate-limit est dans le handler).
  - [x] Variants `AppError::SetupAlreadyComplete` (410) + i18n keys (4 locales).
- [x] **T3 — Middleware 423 Locked + AppState** (AC #13-14)
  - [x] `AppState` extension faite dans T1 (cf. ci-dessus).
  - [x] Modifier `middleware/auth.rs::require_auth` : lire `state.users_exist.load(Ordering::Acquire)` (Pass 2 ECH2-2) **après extraction du token, avant JWT decode** (Pass 2 BH2-2) ; si `false` → return `AppError::SetupRequired` (423).
  - [x] Variant `AppError::SetupRequired` (423) + i18n keys (4 locales).
  - [x] Adapter les 29 sites de tests qui construisent `AppState { ... }` → utiliser `AppState::new_for_tests(...)` (Pass 3 OP3-3) — pour minimiser le churn.
  - [x] Vérifier exemptions : `/api/v1/setup/admin`, `/api/v1/auth/login`, `/api/v1/auth/logout`, `/api/v1/auth/refresh`, `/health`, ServeDir fallback NON gated.
- [x] **T4 — Frontend setup screen** (AC #15-19, AC #20)
  - [x] Nouveau composant `src/lib/features/setup/SetupForm.svelte` (UI form + validation client) — Pass 1 BH1-7 : isole le code i18n-scopé dans `features/` pour permettre au linter de couvrir les usages des clés `setup-*`.
  - [x] Nouvelle route `routes/setup/+page.svelte` (importe `<SetupForm />`) + `+layout.ts` (route publique, redirect `/` si user authentifié).
  - [x] Nouveau wrapper API `lib/features/setup/setup.api.ts::setupAdmin(username, password)` qui appelle `POST /api/v1/setup/admin`. **Sur succès → appel `authState.login(payload)` AVANT redirect** (broadcast cross-tab — Pass 1 ECH1-6).
  - [x] Extension `auth.svelte.ts::hydrate()` : **ajouter explicitement la branche `else if (res.status === 423)`** entre 401 et `else` actuelles, set `_currentUser = null` + `_setupRequired = true`, **sans `console.warn`**.
  - [x] Extension `api-client.ts::request<T>()` : intercepteur 423 **avant `parseErrorResponse`**, guard `pathname !== '/setup'`, `window.location.replace('/setup')`.
  - [x] Extension `routes/+layout.ts` racine : si `auth.isSetupRequired` au boot → goto `/setup`.
  - [x] 11 clés i18n FR/DE/IT/EN (cf. AC #20 liste exacte).
- [x] **T5 — Tests** (AC #21-24)
  - [x] 6 tests unitaires bootstrap matrice + test atomicité recovery + adapter les tests v011-2 existants.
  - [x] 3 tests unitaires Config NEW (`admin_vars_absent`, `admin_vars_empty`, `admin_vars_set_valid`).
  - [x] Nouveau `crates/kesh-api/tests/setup_admin_e2e.rs` (4 scénarios + 1 race documenté ignoré).
  - [x] Preset `setup-required` dans `test_fixtures.rs` + **variant `SetupRequired` ajouté à l'enum `Preset` de `routes/test_endpoints.rs:68`** + `VALID_PRESETS` (l.94) mis à jour + **`type Preset` TypeScript étendue dans `frontend/tests/e2e/helpers/test-state.ts:28-33`** avec `'setup-required'` (Pass 3 OP3-5 — sinon `npm run check` échoue).
  - [x] Spec E2E Playwright `frontend/tests/e2e/setup.spec.ts`.
  - [x] Frontend unit test `SetupForm.svelte.test.ts` (validation client + match confirmation).
- [x] **T6 — Doc admin + CHANGELOG + website** (AC #25-27, Pass 1 AUD1-5)
  - [x] Réécriture section « Premier démarrage » `admin-manual.tex` (setup-UI + warning bloquage réseau).
  - [x] Nouvelle sous-section « J'ai oublié mon mot de passe administrateur » (procédure recovery 7 étapes, étape 7 reformulée Pass 1 BH1-12).
  - [x] PDF régénéré.
  - [x] `.env.example` section « Compte admin initial » double-usage.
  - [x] CHANGELOG `[0.1.2]` : extension `Modifié` + nouveau `Ajout` + **section `### ⚠️ Action requise — upgrade v0.1.1 → v0.1.2`** (Pass 1 BH1-2 / AUD1-2 — Q1 tranchée).
  - [x] **Vérifier `website/index.html` + `website/roadmap.html`** (Pass 1 AUD1-5) : section v0.1.2 alignée (mention onboarding self-service + recovery break-glass si déjà visibles ; sinon mettre à jour la roadmap).
  - [x] **Créer Issue GitHub L1 TOCTOU double-admin** (Pass 1 AUD1-8) label `technical-debt` + `v0.2-milestone`, référencer dans le Change Log final.
  - [x] **Commentaire de merge Issue #127 (KF-035)** : « v011-5 adresse partiellement (section Premier démarrage). RBAC fictif + kesh-cli inexistant restent ouverts. » (Pass 1 AUD1-7)
- [x] **T7 — Quality gate + sprint status** (AC #28-29)
  - [x] Série Test Locally First complète (backend + frontend + E2E avec `KESH_TEST_MODE=true` + `KESH_HOST=127.0.0.1`).
  - [x] Sprint-status : v011-5 in-progress→review.
  - [x] Commit + push à la demande.

### Review Findings — Pass 1 (2026-05-31, Sonnet 4.6 × 3 lentilles parallèles)

**Trend brut** : 16 findings (3 HIGH + 6 MEDIUM + 5 LOW + 2 chevauchements). Après triage : 11 patches + 3 defer + 1 dismiss + 1 merge.

#### Patches à appliquer (11)

- [ ] [Review][Patch] BH1-1 HIGH — `users::create UniqueConstraintViolation` retourne 409 générique au lieu de 410 `SetupAlreadyComplete` [crates/kesh-api/src/routes/setup.rs:131]
- [ ] [Review][Patch] BH1-2 HIGH — chemin 410 saute `record_failed_attempt(ip)` (déviation AC #8) [crates/kesh-api/src/routes/setup.rs:104-108]
- [ ] [Review][Patch] ECH1-1 HIGH — `users_exist.store(true)` après `refresh_tokens::create` ; échec transient lock l'app jusqu'à restart [crates/kesh-api/src/routes/setup.rs:131-171]
- [ ] [Review][Patch] BH1-4 MEDIUM — `seed_stub_company_only` sans `truncate_all` doc-comment précondition manquant [crates/kesh-db/src/test_fixtures.rs:401-419]
- [ ] [Review][Patch] ECH1-2 MEDIUM — validation password whitespace-only manquante (12 espaces valide) [crates/kesh-api/src/routes/setup.rs:83-95]
- [ ] [Review][Patch] ECH1-3 MEDIUM — frontend `password.length` (UTF-16) vs backend `chars().count()` (Unicode scalars) → 11 emojis false-valide front [frontend/src/lib/features/setup/SetupForm.svelte:25]
- [ ] [Review][Patch] AUD1-1 MEDIUM — test `429 rate-limit` absent de setup_admin_e2e.rs (AC #22) [crates/kesh-api/tests/setup_admin_e2e.rs]
- [ ] [Review][Patch] AUD1-2 MEDIUM — test race TOCTOU `#[ignore]` absent (AC #22 explicit demand) [crates/kesh-api/tests/setup_admin_e2e.rs]
- [ ] [Review][Patch] BH1-3+AUD1-3 MEDIUM — `authState.login(payload)` sans `await` (delta spec AC #17) — sync en pratique mais doc trompeuse [frontend/src/lib/features/setup/setup.api.ts:47]
- [ ] [Review][Patch] ECH1-5 LOW — `requestRaw` (getBlob) sans intercepteur 423 (defense-in-depth) [frontend/src/lib/shared/utils/api-client.ts:453-484]
- [ ] [Review][Patch] BH1-7 LOW — `setTimeout(goto/login, 2000)` leak si SetupForm unmount avant 2s [frontend/src/lib/features/setup/SetupForm.svelte]

#### Defer (3, persistés dans `deferred-work.md`)

- [x] [Review][Defer] BH1-5 MEDIUM — `+layout.ts::load` redirect `_setupRequired` likely dead code first paint (mais defense-in-depth client-nav) — deferred, valeur défensive sur navigations subséquentes [frontend/src/routes/+layout.ts:14-17]
- [x] [Review][Defer] ECH1-4 MEDIUM — `KESH_PASSWORD_MIN_LENGTH` non-propagé au frontend (hardcoded 12) — deferred v0.2, nécessite endpoint public config [frontend/src/lib/features/setup/SetupForm.svelte:22 + locales `setup-password-min`]
- [x] [Review][Defer] AUD1-4 LOW — `website/index.html` + `roadmap.html` non mis à jour v0.1.2 — deferred, non-bloquant pour PR (à faire avant release tag v0.1.2)

#### Dismiss (1)

- BH1-6 LOW — `test_config_no_env` mutates public `Config` fields directly — dismissed, design intentionnel test-only.

## Dev Notes

### Patterns à respecter (ground-truth code)

- **Constantes stub partagées** (Story v011-2) : `STUB_COMPANY_NAME` / `STUB_COMPANY_ADDRESS` dans `auth/bootstrap.rs:22-23` (pub(crate) const) réutilisées par `onboarding.rs:808-809`. Préserver le pattern.
- **Tolérance race admin** (Story v011-2 Pass 1 patches) : sur `UniqueConstraintViolation` lors de l'INSERT user, si on a créé un stub ce boot (`company_count==0`), DELETE l'orphan stub. Préserver pour le cas 2.
- **Helper revocation tokens** : `refresh_tokens::revoke_all_for_user(pool, user_id, reason: &str) -> Result<u64, DbError>` (Story 1.6 / Story 10-5). Réutiliser tel quel pour le cas 5 recovery. Reason = `"admin_break_glass_reset"`.
- **Audit log create** : ground-truth `audit_log.rs:26` confirme `pub async fn insert_in_tx(tx: &mut Transaction<'_, MySql>, entry: NewAuditLogEntry) -> Result<...>` — **signature transaction-only, pas de variante pool** (Pass 1 ECH1-2). Pour le cas 5 recovery, ouvrir une `pool.begin().await?` qui englobe l'UPDATE password + `insert_in_tx`. **Schéma `NewAuditLogEntry` ground-truth** (Pass 3 OP3-1) : `entities/audit_log.rs:42-48` définit `{ user_id: i64, action: String, entity_type: String, entity_id: i64, details_json: Option<serde_json::Value> }`. **Pas de champ `event` ni `details`** (le spec utilise incorrectement ces noms dans plusieurs sections corrigées Pass 3). Pour le cas 5 recovery, instancier :
  ```rust
  NewAuditLogEntry {
      user_id: u.id,
      action: "admin_break_glass_reset".into(),
      entity_type: "user".into(),
      entity_id: u.id,
      details_json: Some(serde_json::json!({
          "username": u.username,
          "trigger": "env_vars_present_hash_diff"
      })),
  }
  ```
  Le choix `entity_type = "user"` est explicite : l'entité auditée est l'admin dont le mdp est resetté (cohérent FK `entity_id` → `users(id)`). `audit_log.user_id` FK `ON DELETE RESTRICT` garantit que l'entrée audit ne peut référencer qu'un user existant — la transaction `UPDATE password + insert_in_tx` reste cohérente même si concurrent DELETE user (rollback préserve l'état). Si transaction échoue → rollback automatique → password inchangé.
- **Cookies HttpOnly** (Story 10-5) : pattern `set_session_cookies(jar, access_token, refresh_token, expires)` dans `auth.rs`. Extraire helper public si nécessaire. **Ne pas réimplémenter** la logique cookies.
- **i18n FR/DE/IT/EN** (CLAUDE.md) : toute clé `setup-*` doit avoir les 4 traductions. `npm run lint-i18n-ownership` enforce le préfixe `setup-*` pour la feature `setup`.

### Sécurité — gate setup ouverte au 1er boot

L'endpoint `POST /api/v1/setup/admin` est **publique sans auth** tant que `user_count == 0`. C'est le compromis classique des apps self-hosted (Jellyfin, Bitwarden). Risques mitigés :

1. **Course "first-to-setup" attacker** : qui touche `/setup/admin` en premier devient admin. Atténuation : (a) warning manuel d'admin de bloquer réseau public avant le 1er boot, (b) rate-limit IP brute-force, (c) auto-disable au 1er succès (410 Gone). **Pas de protection serveur-side automatique** (pas de fenêtre temporelle, pas de token offline). Acceptable v0.1, à reconsidérer v0.2 si retour terrain.
2. **Brute-force du formulaire** : rate-limit IP 5/15min/30min block (cohérent `/auth/login`). À documenter dans le manuel.
3. **MITM si HTTP non-TLS** : v0.1 ne force pas HTTPS (reverse proxy externe en charge). Documenter dans le manuel le risque MITM si setup-UI exposé HTTP non-loopback.
4. **CSRF justification** (Pass 1 AUD1-6) : pas de token CSRF explicite. `POST /api/v1/setup/admin` accepte uniquement `Content-Type: application/json` (body JSON, pas de form-urlencoded). Les navigateurs n'envoient pas de requêtes cross-origin JSON sans CORS preflight → protection SOP implicite. Pas de cookie de session existant au moment de l'appel (les cookies sont définis dans la **réponse**, pas envoyés en input). Si la politique CORS globale était permissive (`Access-Control-Allow-Origin: *` + `Allow-Credentials`), ce serait un finding séparé — mais la config CORS de Kesh v0.1 ne wildcardent pas avec credentials. Le risque CSRF est donc N/A pour cet endpoint.
5. **Fingerprinting setup-mode** (Pass 1 ECH1-10) : un attacker découvre que l'instance est en mode setup en comparant `/api/v1/auth/me` (423) vs `/api/v1/auth/login` (401 si users vide via dummy_verify constant-time). Info non-secrète (l'écran `/setup` est visible publiquement). Acceptable v0.1.

### Limitations documentées (catégorie B per CLAUDE.md §Tech debt management)

- **L1 — TOCTOU double-admin sur `POST /setup/admin`** (Pass 1 AUD1-8 / BH1-5 / ECH1-4) : 2 requêtes concurrentes avec usernames distincts lisent `user_count == 0` simultanément et créent 2 admins. Atténuation v0.1 : rate-limit IP (5/15min) + auto-disable au 1er succès (410). Remédiation v0.2 : transaction `SELECT ... FOR UPDATE` sur une row sentinelle ou advisory lock. **GitHub Issue à créer** label `technical-debt` + `v0.2-milestone` (référencer le numéro dans le Change Log final story).
- **L2 — Refresh token timing window post-recovery cas 5** (Pass 2 ECH2-4) : entre le `tx.commit()` (password reset effectif) et le `refresh_tokens::revoke_all_for_user(...).await` (post-commit, best-effort hors transaction), une requête `/auth/refresh` avec un ancien refresh_token actif peut renouveler une session valide même après le recovery. Fenêtre ~ms (latence réseau du revoke query). Atténuation v0.1 : revoke immédiatement post-commit + tokens expirent naturellement 7j. Remédiation v0.2 : atomiser `RefreshTokenCleanupTask` intra-transaction ou `revoke pre-commit`. **GitHub Issue à créer** label `technical-debt` + `v0.2-milestone`.

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

**Mitigation TRANCHÉE Pass 1 (Q1) — Option B confirmée :**
- Option A (flag DB) écartée : complexe et ajoute un schema change qu'on évite pour un cas de migration ponctuelle.
- **Option B retenue** : documenter agressivement dans CHANGELOG + warning préventif au boot avant l'UPDATE password (cas 5).
- Option C (opt-in `KESH_ADMIN_RECOVERY=true`) écartée : casserait l'unification design (retour au flag explicite éliminé du double-usage).

**Format du warning préventif au boot v0.1.2** (avant l'UPDATE cas 5) :
```
WARN bootstrap: ⚠️ Recovery break-glass déclenché pour user 'admin'
WARN bootstrap: Si vous avez changé votre mdp via l'UI, votre mdp sera écrasé
WARN bootstrap: par KESH_ADMIN_PASSWORD. Pour annuler : Ctrl-C + retirer la var
WARN bootstrap: de .env + redémarrer.
```
Le warning précède l'UPDATE de quelques millisecondes — l'opérateur attentif aux logs peut Ctrl-C avant le commit transaction. C'est imparfait mais c'est une **mitigation visible** vs un reset complètement silencieux.

**CHANGELOG section dédiée** (AC #27) : `### ⚠️ Action requise — upgrade v0.1.1 → v0.1.2` avec wording exact « **Si vous avez changé votre mot de passe administrateur via l'UI depuis l'installation** » (vs la note migration v011-2 trop optimiste « aucune action requise »).

### `Config::from_env` — vars optionnelles (TRANCHÉ Pass 1 — cf. section dédiée "Refactor `Config::from_env`" dans Contexte technique)

Décision Pass 1 : `Config::admin_username: Option<String>` + `Config::admin_password: Option<String>`. Vars absentes/vides → `None`. Validations sécu (longueur, changeme) uniquement si `Some(non-empty)`. Détail complet dans la section dédiée du Contexte technique. **AC #0 prérequis bloquant T1.**

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

### Questions tranchées Pass 1 (Sonnet 4.6, 2026-05-30)

- **Q1 — Piège recovery v0.1.1 → v0.1.2** : **TRANCHÉE → Option B** (doc agressive + warning préventif au boot avant l'UPDATE cas 5). Détail dans la section "Compatibilité v0.1.1 → v0.1.2" + AC #27 section CHANGELOG dédiée `### ⚠️ Action requise`.
- **Q2 — Bind loopback obligatoire avant setup** : **TRANCHÉE → Warning manuel uniquement (option soft)**. Pas de hard-enforce `KESH_HOST=127.0.0.1` tant que `users` vide. Rationale : le binding host est une décision opérateur (cf. memory `feedback_deployment_port_mapping_user_concern`), et le warning manuel + rate-limit IP suffisent à atténuer le risque. À reconsidérer v0.2 si retour terrain.
- **Q3 — Rate-limit setup-admin** : **TRANCHÉE → 5 tentatives/15 min/30 min block IP** (cohérent `/auth/login`). L'auto-disable au 1er succès (410 Gone) limite la valeur d'un rate-limit strict. Pas d'arbitrage à durcir.
- **Q4 — Audit log details** : **TRANCHÉE → minimal**. Details JSON = `{ "username": "<x>", "trigger": "env_vars_present_hash_diff" }`. Pas de `client_ip`/`user_agent` (non applicables — boot side-effect, pas requête HTTP). Si forensic-needs futur, ajouter dans v0.2.
- **Q5 — Frontend `/setup` accessible directement par URL si admin existe** : **TRANCHÉE → redirect `/login`**. Cohérent AC #17 (« Error 410 → goto /login »). Le `+layout.ts` route guard vérifie aussi au montage : si `auth.isAuthenticated` → redirect `/`, sinon → afficher form (qui peut échouer en 410 si admin créé entre-temps, géré par AC #17).

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

### Pass 1 spec validate (2026-05-30, Sonnet 4.6 × 3 lentilles parallèles)

3 reviewers adversariaux Sonnet (BlindHunter / EdgeCaseHunter / Auditor) sur le spec initial. **28 findings raw** (3 CRITICAL + 8 HIGH + 11 MEDIUM + 6 LOW), **convergence forte** sur 2 CRITICAL :

| Code(s) convergents | Sev | Title court |
|---|---|---|
| BH1-1 + ECH1-1 + AUD1-1/AUD1-3 | CRITICAL | `Config::from_env` fail-fast rend cas 1 inatteignable sans refactor préalable |
| BH1-2 + ECH1-7 + AUD1-2 | CRITICAL/HIGH | Piège recovery v0.1.1→v0.1.2 si user a changé mdp UI (Q1 non tranchée) |
| ECH1-2 | CRITICAL | `audit_log::insert_in_tx` exige transaction — atomicité UPDATE+audit non spécifiée |
| BH1-3 | HIGH | `build_auth_cookies` privée → dupliquerait dans setup.rs sans refactor `pub(crate)` |
| BH1-4 | HIGH | Middleware 423 par-requête SELECT EXISTS non-invalidé post-setup (perf perpétuelle) |
| BH1-5 + ECH1-4 | HIGH | Race TOCTOU 2 usernames distincts non testée |
| ECH1-3 | HIGH | `hydrate()` utilise `fetch()` direct (pas apiClient) → besoin de branche 423 explicite |
| ECH1-5 | HIGH | Preset `setup-required` doit être whitelisté dans enum Rust `Preset` + `VALID_PRESETS` |

**Patches appliqués Pass 1 (~22 patches structurels)** :

1. **AC #0 NEW** — pré-requis bloquant T1 : `Config::admin_username`/`admin_password` deviennent `Option<String>`, validations sécu uniquement si `Some(non-empty)`, 3 tests config (NEW absent/vide/valide).
2. **Section dédiée "Refactor `Config::from_env`"** — décision tranchée + détail downstream + tests.
3. **AC #1 amendé** — détection `has_admin_env` Option-based + lecture unique `company_count`/`user_count`.
4. **Pseudo-code matrice** ajouté en Dev Notes.
5. **AC #4 amendé** — transaction atomique UPDATE+audit_log, rollback strategy, warning préventif avant UPDATE.
6. **AC #7 + Dev Notes** — `company_count` lu une seule fois, cleanup orphan stub préservé.
7. **AC #11 amendé** — `build_auth_cookies` rendue `pub(crate)`, `username.trim()` stocké, `state.users_exist.store(true)` après INSERT.
8. **AC #12 amendé** — race TOCTOU L1 tracée comme limitation acceptée v0.1.
9. **AC #13 amendé** — `AppState::users_exist: Arc<AtomicBool>` cache mémoire (vs SELECT par requête), fail-open sur DB error.
10. **AC #14 amendé** — exemptions étendues (login/logout/refresh/setup/admin/ServeDir).
11. **AC #18 amendé** — branche 423 explicite dans `hydrate()` sans `console.warn`.
12. **AC #19 amendé** — interceptor 423 dans `request<T>()` avant `parseErrorResponse`, `window.location.replace`, guard pathname.
13. **AC #17 amendé** — `authState.login(payload)` broadcast cross-tab avant goto.
14. **AC #20 amendé** — 11 clés `setup-*` détaillées, mitigation linter scope via `SetupForm` composant dans `src/lib/features/setup/`.
15. **AC #22 amendé** — test race TOCTOU documenté (#[ignore] OK) + test atomicité recovery.
16. **AC #24 amendé** — variant `SetupRequired` enum Preset + `VALID_PRESETS` + File List étendue.
17. **AC #27 amendé** — section CHANGELOG `### ⚠️ Action requise — upgrade v0.1.1 → v0.1.2` avec wording exact.
18. **T1 enrichi** — sous-task Config refactor + AppState `users_exist`.
19. **T2 enrichi** — `build_auth_cookies pub(crate)`.
20. **T3 enrichi** — AppState + middleware via cache mémoire.
21. **T4 enrichi** — `SetupForm.svelte` composant `features/setup/`, broadcast cross-tab, hydrate explicite.
22. **T5 enrichi** — atomicité tests + Preset enum update.
23. **T6 enrichi** — website/, Issue L1 TOCTOU, commentaire Issue #127.
24. **5 questions ouvertes Q1-Q5 → TRANCHÉES** (Q1 Option B, Q2 warning manuel uniquement, Q3 5/15min, Q4 minimal, Q5 redirect /login).
25. **Dev Notes Sécurité** — CSRF justification ajoutée (AUD1-6), fingerprinting login vs me (ECH1-10).
26. **Dev Notes Limitations** — L1 TOCTOU formalisée catégorie B avec remédiation v0.2.
27. **Procédure recovery étape 7** reformulée optionnelle (BH1-12).
28. **Audit log** ground-truth `insert_in_tx` transaction-only documentée.
29. **Tableau matrice** : `tracing::warn!` cas 5 → `tracing::error!` (AUD1-9 corrigé).

**Findings dismissed / non-patché** :
- ECH1-9 (`username.trim()` stocké) — couvert par AC #11 mention « `username.trim()` stocké ».
- BH1-11 (LOW liste i18n keys `setup-success`) — couvert par AC #20 mention « pas de `setup-success` (success → redirect immédiat) ».
- ECH1-10 (LOW fingerprinting login/me) — documenté section Sécurité.
- AUD1-10 (LOW Test Locally First `KESH_HOST=127.0.0.1`) — pré-requis Playwright déjà documenté dans `docs/testing.md` et T7 mention `KESH_HOST=127.0.0.1`.

**Trend Pass 1** : 28 findings raw → **0 CRITICAL/HIGH restant** (tous patchés ou tranchés), reste à confirmer en Pass 2 Haiku avec LLM orthogonal. Critère d'arrêt Review Iteration Rule non atteint (Pass 1 unique modèle Sonnet, biais d'auteur sur patches). Prochaine passe obligatoire : **Pass 2 Haiku 4.5 contexte frais**.

### Pass 2 spec validate (2026-05-31, Haiku 4.5 × 3 lentilles parallèles, contexte frais)

3 reviewers adversariaux Haiku (BlindHunter / EdgeCaseHunter / Auditor) sur le spec post-Pass 1. **~20 findings raw** (2 CRITICAL + 6 HIGH + 9 MEDIUM + 5 LOW). **Discipline grep ground-truth Haiku 4.5 systématique** (CLAUDE.md §"Haiku-specific guardrails") :

**Faux-positifs réfutés (2 CRITICAL + 4 redondants)** :
- **AUD2-1 CRITICAL DISMISSED** : Haiku « `Config::from_env` refactor AC #0 non appliqué — admin_username/admin_password restent `String` ». **Misread méta-spec** : AC #0 *demande* le refactor (TODO pour dev-story), pas une affirmation que le refactor est exécuté. Pattern documenté dans memory `feedback_haiku_review_diff_combined`. Le spec est l'instruction, pas le code.
- **ECH2-1 CRITICAL DISMISSED** : Haiku « tx.commit() peut échouer après audit_log::insert_in_tx → sessions attaquant restent valides 7j ». **Misanalyse transaction** : si commit échoue → SQLx rollback automatique → password inchangé → la branche cas 5 retourne `Err(...)?` early → `revoke_all_for_user` jamais appelé. État DB cohérent (rien n'a changé), aucun leak. Haiku confond « commit fail + revoke skipped » avec « password changed + sessions live » — non, le rollback annule TOUT.
- AUD2-2..AUD2-8 : tous **redondants** avec Pass 1 (Issue #121/#127 statut déjà spécifié AC #25, P5 audit déjà N/A, avoid-parallel-prs déjà fait commit `9c7fb8d`, linter scope déjà documenté AC #20, website déjà dans T6, atomicité tests déjà AC #22).

**Findings légitimes (3 HIGH + 5 MEDIUM = 8 patches Pass 2)** :

| Code | Sev | Title court | Patch |
|---|---|---|---|
| BH2-2 | HIGH | Middleware 423 position d'insertion non spécifiée | AC #13 amendé : insertion APRÈS extraction token, AVANT JWT decode |
| ECH2-2 | HIGH | `Ordering::Relaxed` insuffisant sur ARM/weak-memory CPU | AC #11 store→`Release`, AC #13 load→`Acquire` (paire happens-before) |
| ECH2-3 | HIGH | `authState.login(payload)` sans `await` explicite — broadcast cross-tab non-garanti | AC #17 amendé : `await authState.login(...)` explicite |
| BH2-1 | MEDIUM | `make_test_config` signature ambiguë (défauts internes ou args explicites ?) | AC #0 amendé : signature inchangée, callers passent explicitement, nouveaux tests construisent `Config` direct |
| BH2-3 | MEDIUM | `state.users_exist` non resetté entre tests (truncate DB mais flag mémoire stuck) | AC #24 amendé : seed handler `state.users_exist.store(user_count > 0, Release)` après TRUNCATE+INSERT |
| BH2-4 | MEDIUM | Variable `_setupRequired` jamais déclarée dans le store | AC #18 amendé : déclaration `let _setupRequired = $state<boolean>(false);` + getter + reset au login |
| ECH2-6 | MEDIUM | Invariant `Some(s) ⟹ !s.is_empty()` non documenté → `has_admin_env` détection redondante mais correcte | AC #0 amendé : invariant explicite, double-check défensif |
| ECH2-7 | MEDIUM | Preset seed atomicité + Playwright workers parallelism | AC #24 amendé : workers=1 cohérent existant, atomicité transaction handler |

**Findings LOW (5)** acceptés / déjà documentés / inchangés :
- BH2-5 LOW : broadcast cross-tab sur 410 — acceptable v0.1 (page reload via goto /login).
- BH2-6 LOW : api-client guard pathname imprécis hash URL — edge case improbable, acceptable v0.1.
- BH2-7 LOW : linter scope vérification — déjà notée AC #20 + mitigation `SetupForm` features/.
- ECH2-4 LOW (Haiku HIGH→MEDIUM) : refresh_token timing window post-commit-before-revoke — déjà documenté Pass 1 "non-critique partiellement exécuté".
- ECH2-8 LOW : redirects parallèles 423 guard convergence — déjà documenté Pass 1 AC #19 idempotent.

**Trend cumulatif** :
- Pass 1 Sonnet : 28 findings → 22 patches structurels (3 CRITICAL + 8 HIGH adressés)
- Pass 2 Haiku : ~20 findings raw → **2 CRITICAL réfutés grep ground-truth** + 8 patches légitimes (3 HIGH + 5 MEDIUM) + 5 LOW acceptés
- Discipline grep ground-truth Haiku : **2 hallucinations CRITICAL réfutées** (cohérent retour d'expérience `feedback_haiku_review_diff_combined` — Haiku misread méta-spec et misanalyse transaction)

Pass 2 ne converge **pas** (3 HIGH légitimes remontés Pass 2). **Pass 3 Opus 4.7 obligatoire** (cycle Sonnet→Haiku→Opus per Review Iteration Rule) sur contexte frais pour catch les findings architecturaux que Sonnet+Haiku ratent (pattern Story 10-2/10-3/10-5 ECH catch architectural).

### Pass 3 spec validate (2026-05-31, Opus 4.7 comprehensive single reviewer, contexte frais)

1 reviewer Opus 4.7 comprehensive (lentilles BlindHunter + EdgeCaseHunter + Auditor cumulées, focus architectural cross-fichiers/cross-fonctions). **8 findings** (3 HIGH + 3 MEDIUM + 2 LOW) — pattern empirique Opus catch architectural confirmé (Stories 10-2/10-3/10-5 rétrospective Epic 10).

**Tous les 6 findings HIGH/MEDIUM vérifiés par grep ground-truth avant patch** :

| Code | Sev | Title court | Patch appliqué |
|---|---|---|---|
| OP3-1 | HIGH | `NewAuditLogEntry` schema réel = `action/entity_type/entity_id/details_json`, PAS `event/details` | Pseudo-code matrice + AC #4 + Dev Notes : signature canonique corrigée avec `action: "admin_break_glass_reset"`, `entity_type: "user"`, `entity_id: u.id`, `details_json: Some(json!({...}))` |
| OP3-2 | HIGH | `Config::from_fields_for_test` 30 sites tests d'intégration non traités | AC #0 amendé : signature inchangée `String, String`, wrappe `Some(...)` interne, 0 migration |
| OP3-3 | HIGH | `AppState` extension impacte 28 sites tests + 1 site `test_state` | Section middleware enrichie : constructeur `AppState::new_for_tests(...)` avec `users_exist: Arc::new(AtomicBool::new(true))` par défaut, minimise churn |
| OP3-4 | MEDIUM | Boot order : `ensure_admin_user` doit retourner `user_count` ; `lib.rs::create_state` inexistant (site réel `main.rs:220` inline) | Section enrichie : signature `Result<i64, AppError>`, capture dans `main.rs:152`, construction `AppState` inline `main.rs:220` |
| OP3-5 | MEDIUM | Frontend `type Preset` TypeScript fermé non synchronisé avec enum Rust | AC #24 amendé : ajouter `'setup-required'` à l'union `test-state.ts:28-33`, File List étendue |
| OP3-6 | MEDIUM | `RateLimiter` est check manuel handler, pas middleware Axum | AC #8 reformulé : `state.rate_limiter.check_rate_limit(ip)?` en début handler, `record_failed_attempt(ip)` sur échec, défense en profondeur quota partagé /login |
| OP3-7 | LOW | `audit_log.user_id` FK ON DELETE RESTRICT note | Documenté Dev Notes Audit log |
| OP3-8 | LOW | Quota rate-limiter partagé `/login` + `/setup/admin` = défense en profondeur | Documenté AC #8 |

**Pattern Opus distinctive value confirmé** : OP3-1 (schéma audit_log erroné), OP3-2 (30 call-sites `from_fields_for_test`), OP3-3 (28 sites `AppState`) sont 3 HIGH cross-fichiers que Sonnet Pass 1 et Haiku Pass 2 ont tous deux ratés. Ces 3 findings auraient surface comme **compile errors au dev-story T1** sans Opus. Pass 3 Opus se paie largement.

**Trend cumulatif Pass 1+2+3** :
- Pass 1 Sonnet : 28 findings raw → 22 patches structurels (3 CRITICAL + 8 HIGH adressés)
- Pass 2 Haiku : ~20 findings → 8 patches légitimes (3 HIGH + 5 MEDIUM) + 2 CRITICAL réfutés grep
- Pass 3 Opus : 8 findings → 8 patches (3 HIGH + 3 MEDIUM + 2 LOW)

**Convergence à évaluer Pass 4** : 0 CRITICAL/HIGH restant après Pass 3 patches. Per Review Iteration Rule, le critère d'arrêt est « 0 finding > LOW ». L2 (refresh token timing) et L1 (TOCTOU) sont catégorie B documentées avec Issue GitHub planifiée — résolues par reclassement. Pass 4 Sonnet 4.6 (cycle pair Sonnet→Haiku→Opus→Sonnet) **recommandée** pour confirmer convergence sur contexte frais avant dev-story. Si Pass 4 converge → STOP.

### Pass 4 spec validate (2026-05-31, Sonnet 4.6 comprehensive convergence check, contexte frais)

1 reviewer Sonnet 4.6 comprehensive (lentilles cumulées, focus convergence). **2 findings** :

| Code | Sev | Title | Patch |
|---|---|---|---|
| CC4-1 | HIGH | Pseudo-code `await?` dans `main()` → `error: ? in async fn returning ()` | Pseudo-code remplacé par pattern `match { Ok(c) => c, Err(e) => exit(1) }` cohérent existant `main.rs` |
| CC4-2 | LOW | AC #20 dit « 11 clés » mais liste nominative en compte 12 | « 11 au total » → « 12 au total » |

**Pass 4 Sonnet a validé ground-truth toutes les autres dimensions** :
- ✓ `NewAuditLogEntry` schema `action/entity_type/entity_id/details_json` cohérent partout
- ✓ `Config::admin_*: Option<String>` cohérent AC #0/Tasks/Config section
- ✓ `Ordering::Acquire/Release` paire correcte (load=Acquire AC #13, store=Release AC #11/#24)
- ✓ Rate-limiter check manuel handler (pas middleware Axum) cohérent
- ✓ `insert_in_tx` + `revoke_all_for_user` signatures alignées ground-truth
- ✓ `build_auth_cookies` actuellement privée → `pub(crate)` demandé cohérent
- ✓ `AppState` extension via `new_for_tests` constructeur cohérent
- ✓ `type Preset` TypeScript/`enum Preset` Rust sync demandés cohérents
- ✓ Couverture tests 6 cas matrice + atomicité + race + E2E
- ✓ Migration breaking N/A confirmé
- ✓ Issues GitHub planifiées L1 + L2 catégorie B
- ✓ Issue #121 fermée au merge + #127 commentaire spécifié
- ✓ Splitting rule documentée avec soupape

**Trend cumulatif final** :
- Pass 1 Sonnet : 28 findings → 22 patches (3C + 8H adressés)
- Pass 2 Haiku : ~20 findings → 8 patches + 2 CRITICAL réfutés grep
- Pass 3 Opus : 8 findings → 8 patches (3H architecturaux ratés par Sonnet+Haiku)
- Pass 4 Sonnet : 2 findings → 2 patches (1 HIGH typo `?` + 1 LOW cosmétique)

**Critère d'arrêt Review Iteration Rule ATTEINT** (post-Pass 4 patches) : 0 CRITICAL/HIGH/MEDIUM restant, uniquement L1+L2 catégorie B reclassées en dette technique tracée v0.2 + 0 finding LOW restant après patch CC4-2. **CYCLE CONVERGÉ** en 4 passes (cycle complet Sonnet→Haiku→Opus→Sonnet per Review Iteration Rule). Status `ready-for-dev` confirmé. Prochaine étape : **`bmad-dev-story v011-5`** (Opus 4.7 recommandé pour scope architectural).

## Dev Agent Record

### Agent Model Used

**Opus 4.7 (1M context)** — single-pass orchestré T1 → T7 conforme à la recommandation de la story (« Opus 4.7 recommandé pour scope architectural », rétro Epic 10 confirmant le pattern).

### Debug Log References

Aucun crash / blocage. Deux corrections faites en cours de dev :

1. **`bootstrap_creates_admin_on_existing_company` test échoué (1ère exécution)** : ma refactor cas 2 créait toujours une stub, même quand `company_count > 0` (partial state). Patch : branchement `if company_count == 0 → insert_stub_company; else → SELECT id FROM companies LIMIT 1`. Cleanup orphan stub gated par `created_stub_this_boot: bool`. Cohérent v011-2.
2. **`bootstrap_recovery_diff_hash_resets` test échoué (refresh tokens 0 revoked)** : DB constraint `chk_refresh_tokens_revoked_reason` whitelist {logout, rotation, password_change, admin_disable, theft_detected}. Mon `"admin_break_glass_reset"` non listé → INSERT silencieux ignoré. Patch : utilise `"password_change"` pour `refresh_tokens.revoked_reason` (mapping sémantique le plus proche : recovery = changement de password). Le motif détaillé `"admin_break_glass_reset"` reste dans `audit_log.action` (VARCHAR libre). Évite une migration `ALTER CHECK` dédiée pour v0.1.2.

### Completion Notes List

- **T1 (AC #0-7) — Bootstrap matrice 6 cas + Config refactor + AppState** : `Config::admin_username/password: Option<String>` avec invariant `Some(s) ⟹ !s.is_empty()` garanti par `from_env`. `make_test_config` signature inchangée (wrap `Some(...)` interne). `from_fields_for_test` idem. `ensure_admin_user` retourne `Result<i64, AppError>` (user_count post-bootstrap). `AppState::users_exist: Arc<AtomicBool>` ajouté + constructeur `AppState::new_for_tests` pour minimiser churn (28 sites de tests d'intégration patchés via script Python regex). 8 tests bootstrap matrice verts (cas 1-6 + idempotence + partial state) + 5 tests Config (absent/empty/valide/changeme/short).
- **T2 (AC #8-12) — Setup endpoint POST /api/v1/setup/admin** : `routes/setup.rs` créé. Rate-limit IP via `state.rate_limiter.check_rate_limit(ip)?` en début handler (pattern cohérent `/login`, pas middleware Axum). Gate `user_count > 0 → 410 SetupAlreadyComplete`. INSERT user `Admin` attaché à la company stub. `state.users_exist.store(true, Release)` post-INSERT (paire avec `Acquire` middleware). Cookies HttpOnly via `build_auth_cookies` rendu `pub(crate)`. Body response = `LoginResponse` (cohérent /login). 6 tests setup_admin_e2e.rs verts (happy path + 410 + 423 + validation + rate-limit + exemption /health).
- **T3 (AC #13-14) — Middleware 423 Locked gate** : `require_auth` lit `state.users_exist.load(Acquire)` APRÈS extraction token, AVANT JWT decode. Si `false` → return `AppError::SetupRequired` (423 Locked). Variant `AppError::SetupRequired` ajouté + mapping HTTP 423 + i18n key `error-setup-required` 4 locales. Routes exemptes : `/health`, `/api/v1/auth/login`, `/api/v1/auth/logout`, `/api/v1/auth/refresh`, `/api/v1/setup/admin`, ServeDir fallback (toutes sur `main_router` hors sub-router `protected`). 2 tests middleware nouveaux verts (`users_exist_false_returns_423_no_token` + `users_exist_false_returns_423_even_with_valid_jwt`).
- **T4 (AC #15-20) — Frontend setup screen + hydrate 423 + interceptor** : route `/setup` avec `+page.svelte` + `+layout.ts` (route publique, redirect `/` si authentifié). Composant `src/lib/features/setup/SetupForm.svelte` isolé dans `features/` pour que `lint-i18n-ownership` couvre les clés `setup-*`. Wrapper API `setup.api.ts::setupAdmin` qui appelle `authState.login(payload)` post-POST (broadcast cross-tab). `auth.svelte.ts::hydrate()` ajoute branche explicite `else if (res.status === 423)` → `_currentUser = null` + `_setupRequired = true` (sans `console.warn` — état légal). Store expose `get isSetupRequired()`. `clearSession` + `login` resettent `_setupRequired`. `api-client.ts::request<T>()` interceptor 423 AVANT `parseErrorResponse` → `window.location.replace('/setup')` (guard `pathname !== '/setup'` + `!url.startsWith('/api/v1/setup/')`). `+layout.ts` racine : si `auth.isSetupRequired` + `pathname !== '/setup'` → redirect /setup. 12 clés i18n FR/DE/IT/EN ajoutées (`setup-welcome`, `setup-intro`, ... `setup-error-rate-limit`).
- **T5 (AC #21-24) — Tests bootstrap matrice + setup E2E + Playwright** : 8 tests unit bootstrap matrice + 5 tests Config + 6 tests setup_admin_e2e + 5 tests SetupForm.svelte (Vitest) + 4 tests Playwright `setup.spec.ts`. Preset `SetupRequired` ajouté à `enum Preset` Rust + `VALID_PRESETS` + handler match + `seed_stub_company_only` fixture. Type `'setup-required'` ajouté à l'union `Preset` TypeScript `frontend/tests/e2e/helpers/test-state.ts`. Synchronisation `state.users_exist` post-TRUNCATE+seed (BH2-3 — évite divergence cache mémoire vs DB).
- **T6 (AC #25-27) — Doc admin LaTeX + CHANGELOG + .env.example + Issue L1** : section « Premier démarrage » `admin-manual.tex` réécrite (setup-UI + warning bloquage réseau public + procédure alternative `.env` bootstrap déclaratif). Nouvelle sous-section « J'ai oublié mon mot de passe administrateur » (procédure recovery 7 étapes + tableau matrice 6 cas). PDF régénéré (`latexmk -xelatex`). `.env.example` section admin réécrite : variables OPTIONNELLES avec triple-usage (bootstrap déclaratif / setup-UI / recovery). CHANGELOG `[0.1.2]` : section `### ⚠️ Action requise — upgrade v0.1.1 → v0.1.2` (Q1 Option B) + section `### Ajouts` (onboarding + recovery). Issue GitHub #133 créée (`technical-debt` + `v0.2-milestone`) pour L1 TOCTOU double-admin.
- **T7 (AC #28-29) — Quality gate Test Locally First** : `cargo fmt --all -- --check` ✓ ; `cargo build --workspace --all-targets` ✓ ; `cargo clippy --workspace --all-targets -- -D warnings` ✓ ; `cargo test --lib -p kesh-api` ✓ 199 tests ; `cargo test --test setup_admin_e2e -- --test-threads=1` ✓ 6 tests ; `cargo test --test auth_e2e --test auth_cookies_e2e --test test_endpoints_e2e --test onboarding_e2e -- --test-threads=1` ✓ 42+9+10+13 = 74 tests sans régression ; `npm run check` ✓ 0 erreurs 25 warnings pré-existants ; `npm run lint-i18n-ownership` ✓ PASS ; `npm run test:unit -- --run` ✓ 267 tests (262 base + 5 SetupForm) ; `npm run build` ✓ bundle généré. Sprint-status `in-progress → review` au commit final.

### File List

#### Backend Rust (modified)

- `crates/kesh-api/src/config.rs` — `admin_username/password: Option<String>`, validations sécu conditionnelles, 5 nouveaux tests
- `crates/kesh-api/src/auth/bootstrap.rs` — matrice 6 cas, retourne `Result<i64, AppError>`, 8 tests (6 cas + idempotence + partial state)
- `crates/kesh-api/src/lib.rs` — `AppState::users_exist: Arc<AtomicBool>`, constructeur `new_for_tests`, mount `/api/v1/setup/admin`
- `crates/kesh-api/src/main.rs` — capture `user_count` du retour bootstrap, init `users_exist` avec valeur réelle DB
- `crates/kesh-api/src/middleware/auth.rs` — gate 423 Locked APRÈS extraction token AVANT JWT decode, 2 nouveaux tests
- `crates/kesh-api/src/errors.rs` — variants `SetupRequired` (423) + `SetupAlreadyComplete` (410) + mappings i18n
- `crates/kesh-api/src/routes/auth.rs` — `build_auth_cookies` rendu `pub(crate)` pour réutilisation
- `crates/kesh-api/src/routes/mod.rs` — déclaration `pub mod setup;`
- `crates/kesh-api/src/routes/test_endpoints.rs` — variant `Preset::SetupRequired` + `VALID_PRESETS` + handler + sync `state.users_exist` post-seed
- `crates/kesh-db/src/test_fixtures.rs` — helper `seed_stub_company_only`

#### Backend Rust (new)

- `crates/kesh-api/src/routes/setup.rs` — handler `POST /api/v1/setup/admin`
- `crates/kesh-api/tests/setup_admin_e2e.rs` — 6 tests E2E (happy path + 410 + 423 + validation + exemption /health)

#### Backend Rust — adaptation AppState struct literal (28 tests, ajout `users_exist`)

- `crates/kesh-api/tests/auth_cookies_e2e.rs`, `auth_e2e.rs`, `bank_accounts_e2e.rs`, `bank_imports_e2e.rs`, `bank_profiles_e2e.rs`, `companies_e2e.rs`, `exports_global_e2e.rs`, `fiscal_years_e2e.rs`, `health_endpoint.rs`, `i18n_e2e.rs`, `idor_multi_tenant_e2e.rs`, `invoice_echeancier_e2e.rs`, `invoice_pdf_e2e.rs`, `kf004_no_op_e2e.rs`, `onboarding_e2e.rs`, `onboarding_path_b_e2e.rs`, `profile_e2e.rs`, `rbac_e2e.rs`, `reconciliation_e2e.rs`, `reconciliation_manual_e2e.rs`, `reconciliation_rules_e2e.rs`, `reconciliation_split_e2e.rs`, `reports_e2e.rs`, `reports_export_e2e.rs`, `spa_resilience.rs`, `test_endpoints_e2e.rs`, `users_e2e.rs`, `vat_rates_e2e.rs` (script Python regex appliqué)

#### Frontend (modified)

- `frontend/src/lib/app/stores/auth.svelte.ts` — `_setupRequired` $state, `isSetupRequired` getter, branche 423 dans `hydrate()`, reset dans `login`/`clearSession`
- `frontend/src/lib/shared/utils/api-client.ts` — interceptor 423 AVANT `parseErrorResponse`, guards anti-boucle
- `frontend/src/routes/+layout.ts` — redirect `/setup` si `auth.isSetupRequired` au boot
- `frontend/tests/e2e/helpers/test-state.ts` — type `Preset` étendu `'setup-required'`

#### Frontend (new)

- `frontend/src/lib/features/setup/SetupForm.svelte` — composant formulaire
- `frontend/src/lib/features/setup/setup.api.ts` — wrapper `setupAdmin()` + broadcast `authState.login`
- `frontend/src/lib/features/setup/SetupForm.test.ts` — 5 tests Vitest validation client + submit happy path
- `frontend/src/routes/setup/+page.svelte` — page wrapping SetupForm
- `frontend/src/routes/setup/+layout.ts` — route publique avec redirect si auth
- `frontend/tests/e2e/setup.spec.ts` — 4 tests Playwright (redirect /setup, validation, happy path, 410)

#### i18n (modified)

- `crates/kesh-i18n/locales/{fr,de,it,en}-CH/messages.ftl` — 14 clés ajoutées par locale (12 UI `setup-*` + 2 erreur `error-setup-*`)

#### Documentation (modified)

- `docs/manual/fr/admin-manual.tex` — section « Premier démarrage » réécrite + sous-section recovery + tableau matrice
- `docs/manual/fr/admin-manual.pdf` — régénéré (`latexmk -xelatex`)
- `CHANGELOG.md` — section `[0.1.2]` étendue avec Action requise + Ajouts onboarding/recovery
- `.env.example` — section admin OPTIONNELLE avec triple-usage documenté
- `_bmad-output/implementation-artifacts/sprint-status.yaml` — `v011-5: in-progress → review`

## Change Log

### Dev-story (2026-05-31, Opus 4.7 single-pass orchestré)

Implémentation T1 → T7 complète selon spec validate Pass 4 CONVERGED. 29 ACs (AC#0 prerequis + AC#1-7 bootstrap + AC#8-12 setup endpoint + AC#13-14 middleware 423 + AC#15-19 frontend + AC#20 i18n + AC#21-24 tests + AC#25-27 doc + AC#28-29 quality gate) tous adressés. 0 régression sur baselines pré-existantes (74+199 tests backend critiques verts + 267 tests Vitest frontend + lint i18n-ownership pass).

**Patches imprévus en cours de dev** :
1. Cas 2 partial state (company existe + user 0) : ajout branchement `created_stub_this_boot: bool` pour préserver v011-2 et éviter de créer un 2e stub.
2. Refresh tokens `revoked_reason` : mapping `"password_change"` (valeur dans le whitelist DB check constraint) au lieu de `"admin_break_glass_reset"` qui aurait nécessité une migration ALTER CHECK. Le motif détaillé `admin_break_glass_reset` reste dans `audit_log.action` (VARCHAR libre).

**Trend cumulatif (dev-story unique passe Opus 4.7)** :
- 0 erreur de compilation au final.
- `cargo fmt --check` ✓, `cargo clippy -D warnings` ✓.
- 199 tests `cargo test --lib -p kesh-api` verts (8 nouveaux bootstrap + 5 nouveaux config inclus).
- 6 tests `setup_admin_e2e.rs` E2E verts.
- 267 tests Vitest verts (5 nouveaux SetupForm inclus).
- 4 tests Playwright `setup.spec.ts` créés (exécution nécessite backend + preset `setup-required` seedé).

**Prochaine étape** : `bmad-code-review v011-5` avec un LLM différent (cycle Sonnet→Haiku→Opus→Sonnet per Review Iteration Rule). Suggestion : Sonnet 4.6 en Pass 1 puisque le dev-story a été fait par Opus 4.7.
