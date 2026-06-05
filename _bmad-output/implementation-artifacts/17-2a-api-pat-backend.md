# Story 17.2a: API externe à clé PAT — Backend (table, repo, middleware factorisé, audit)

Status: ready-for-dev

<!-- Issue de la scission de la story 17-2 (spec convergée 5 passes validate, 2026-06-04). 17-2a = Partie A backend (story-foundation, la plus risquée : middleware auth + ripple audit). Voir 17-2-api-pat-integrations.md pour le contexte parent complet. -->

## Story

As a **PME utilisant Kesh**,
I want **un backend qui génère et valide des clés d'accès API (lecture seule ou lecture-écriture) liées à une company, présentées par header `Authorization: Bearer kesh_pat_…`**,
so that **une IA externe (Claude API, ChatGPT, agent custom) ou un logiciel tiers (script Python, ETL, dashboard BI, ERP) puisse consommer mes données comptables via les routes `/api/v1/*` existantes sans partager mes identifiants utilisateur**.

## Contexte & provenance

- **Issue GitHub** : [#100](https://github.com/guycorbaz/kesh/issues/100) `[CR] API externes avec clé PAT (lecture / lecture-écriture) pour intégrations IA & logiciels tiers` — `enhancement` + `v0.2-milestone`.
- **Epic** : 17 « Infra & Souveraineté » — Story 17-2 (cf. `_bmad-output/planning-artifacts/epic-17.md` décisions D4/D5). Vient **après** la story-zéro sécurité 17-1 (TOCTOU, mergée PR #163).
- **Scission** : cette story **17-2a** est la **Partie A backend** issue du split de **17-2** (spec parente `17-2-api-pat-integrations.md`, **convergée en 5 passes validate** Sonnet→Haiku→Opus→Sonnet→Haiku, 0 finding > LOW, 24 patches). Le split en 17-2a/b/c est **confirmé** par le validate (règle de splitting préventif CLAUDE.md : Partie A seule > 5 modules ; ripple audit F-OPUS-1 non purement mécanique). 17-2a est la **story-foundation** (middleware auth factorisé + ripple audit cross-call-site) ; 17-2b (frontend) et 17-2c (doc) en dépendent et suivent.
- **Option retenue (D4/D5 + issue §5)** : **Option A** — middleware d'auth factorisé qui accepte SOIT le cookie session JWT (UI web) SOIT le Bearer PAT (API externe), sur les routes `/api/v1/*` existantes. Pas de routes dédiées `/external/*`.
- **Scope multi-tenant (D5 + issue §3)** : strict **1 `company_id` par clé** (cohérent multi-tenant scoping Epic 7-1 KF-002).

> ⚠️ Cette story **n'inclut PAS** le frontend (page `/settings/api-keys`, i18n de la page, E2E Playwright UI → **17-2b**) ni la documentation externe (`docs/api-external.md`, OpenAPI, synchro docs → **17-2c**). Les routes CRUD `/settings/api-keys` backend (AC7) **sont** dans 17-2a (elles sont consommées par 17-2b). Les 4 traductions Fluent **backend** des 2 codes 403 (AC11 portion backend) **sont** dans 17-2a ; les clés i18n de la **page** frontend sont dans 17-2b.

## Décisions de conception (héritées du validate parent, déjà tranchées)

Reprises verbatim de la spec parente 17-2 convergée — ces points engagent la sécurité/perf et ont survécu à 5 passes adversariales :

- **DC1 — Stockage du token = SHA-256 indexé, PAS Argon2id.** L'issue #100 §1 suggère « Argon2id ou bcrypt », mais un verify Argon2id (~50 ms, cf. `auth/password.rs`) **à chaque requête API** est inacceptable. Un PAT est un secret **aléatoire à haute entropie** (≥ 160 bits), pas un mot de passe faible — le hashing lent n'apporte rien. On stocke **`SHA-256(token)`** (hex 64 chars) avec **index UNIQUE** → lookup `O(1)` par un seul SELECT indexé (~5 ms), pattern standard industrie (GitHub PAT). Amélioration vs `refresh_tokens` qui stocke le token **en clair** (`entities/refresh_token.rs:25`). **Aucun secret en clair, aucun Argon2 per-request.**
- **DC2 — Identité effective du PAT + extension `CurrentUser`.** Le PAT authentifie **au nom de son créateur** : le middleware charge la row `api_keys` (active, non révoquée, non expirée), puis construit le `CurrentUser` avec `user_id = créateur`, `role = rôle COURANT du créateur (relu en DB)`, `company_id = company_id de la clé`. Relire le rôle/état du créateur en DB (≠ JWT qui fait confiance aux claims) garantit qu'une **désactivation du créateur ou un changement de rôle invalide immédiatement le PAT**. Coût : 1 SELECT user (jointure possible avec le lookup clé). **⚠️ Le struct `CurrentUser` (`middleware/auth.rs:33`, champs actuels `{ user_id, role, company_id, exp }`) est ÉTENDU avec `api_key_id: Option<i64>`** (chemin JWT → `None` ; chemin PAT → `Some(<id clé>)`). C'est ce champ qui permet à l'audit de distinguer `actor_type='user'` vs `'api_key'` (DC5/AC8). **Tous les sites de construction de `CurrentUser` (chemin JWT inclus) doivent fournir `api_key_id: None`.** **(S4-2)** Le champ `exp: i64` existant (consommé par `GET /auth/me` → `expires_in`) n'a pas de claim JWT sur le chemin PAT : le remplir avec `api_key.expires_at.map(|dt| dt.and_utc().timestamp()).unwrap_or(i64::MAX)` (clé permanente → `i64::MAX`). Ne pas laisser `0`.
- **DC3 — Application du scope = gate sur la méthode HTTP.** `scope='read'` → **seules GET (et HEAD/OPTIONS) autorisées** ; toute autre méthode → `403 API_KEY_READ_ONLY`. `scope='read-write'` → toutes méthodes (sous réserve du RBAC rôle existant). Permission effective = **intersection(rôle créateur, scope clé)**. Le gate s'exécute dans `require_auth` (en amont des `route_layer` RBAC). **F-OPUS-7** : ce `403 API_KEY_READ_ONLY` est un **rejet global en amont** (méthode HTTP), PAS une erreur per-proposal — sur les endpoints batch type `accept_batch` il ne s'encapsule **jamais** en `FailedProposal` (conforme CLAUDE.md §Pattern batch, exception « 403 RBAC global »).
- **DC4 — Routes de gestion gated Comptable+ (pas Admin-only).** La page `/settings/api-keys` est per-company ; un Comptable doit pouvoir gérer ses intégrations. Aligné sur le guard CRUD `bank_accounts` (`require_comptable_role`).
- **DC5 — Extension `audit_log` non-breaking + portée de l'attribution `api_key`.** Ajout `actor_type ENUM('user','api_key') NOT NULL DEFAULT 'user'` + `actor_api_key_id BIGINT NULL`. `DEFAULT 'user'` rend la migration non-breaking. ⚠️ **Impact ripple (~48 call-sites `NewAuditLogEntry { … }` dans `crates/`** : kesh-db repos ~29, kesh-api routes ~19, `kesh-api/src/auth/bootstrap.rs` 1). Mitigation = constructeurs rétro-compatibles : `::user(user_id, action, entity_type, entity_id, details)` (= sémantique ACTUELLE) et `::api_key(api_key_id, creator_user_id, action, entity_type, entity_id, details)`. **(F-OPUS-5)** Le paramètre `details` alimente le champ struct réel **`details_json: Option<serde_json::Value>`** (`entities/audit_log.rs:35/46`). **Portée de l'attribution `actor_type='api_key'` — la vraie frontière n'est PAS kesh-api/kesh-db (correction F-OPUS-1 Pass 3, ground-truth) — c'est « le call-site d'audit a-t-il `&CurrentUser` en scope, ou seulement un `user_id: i64` nu ? ». Trois catégories :**
  - **(i) Call-sites *handler* kesh-api avec `&CurrentUser` en scope** (ex. `routes/bank_accounts.rs:461/569/670/744`, `reconciliation_rules.rs:446`) : `NewAuditLogEntry::from_current_user(&current_user, action, entity_type, entity_id, details_json)` qui mappe `current_user.api_key_id` → `None`⇒`User` / `Some(id)`⇒`actor_type=ApiKey, actor_api_key_id=id, user_id=current_user.user_id`. Chemin JWT préservé (toujours `User`). **Réalise AC8(b)** pour ces routes.
  - **(ii) Call-sites *helper* kesh-api prenant `user_id: i64` nu** (vérifiés Pass 3 : `bank_accounts.rs:288 audit_primary_transition`, `exports.rs:140 emit_global_export_audit`, `reports.rs:700 emit_report_audit`, `bank_imports.rs:1435 insert_canonical_audit_log`, `reconciliation.rs:910/1220/1587 accept_one_invoice/split/rule`) : `current_user` n'y est PAS en scope. **Décision v0.2** : étendre la signature de ces helpers avec un paramètre `actor_api_key_id: Option<i64>` (passé par le handler appelant depuis `current_user.api_key_id`) → ils construisent `::user(...)` ou `::api_key(...)`. Couvre les mutations PAT passant par ces helpers **sans** toucher kesh-db. Si le coût de threading est jugé excessif au dev-story pour un helper donné, le laisser en `::user(...)` est acceptable **mais DOIT être listé explicitement** comme exception `actor_type='user'` (limitation v0.3).
  - **(iii) Call-sites internes kesh-db repos** (`user_id: i64`, aucun contexte route) : `::user(...)` → restent `actor_type='user'`. Threading jusque-là changerait N signatures de repo → **hors scope v0.2, limitation L1** (imputabilité préservée : `api_key.created` lie la clé à `created_by_user_id`). v0.3 si couverture exhaustive requise.
- **DC6 — Gestion des clés interdite via PAT (durcissement Pass 2 N4).** Les 3 routes de gestion (`list`/`create`/`revoke`) sont accessibles **uniquement par session JWT cookie (UI web)**. Un PAT — **même `read-write`** — ne peut PAS lister/créer/révoquer des clés (sinon une clé fuitée pourrait se cloner/escalader = auto-propagation). Implémentation : rejet `403 API_KEY_MANAGEMENT_FORBIDDEN` si `current_user.api_key_id.is_some()`.

## Acceptance Criteria

### Backend — table, repo, génération

1. **AC1 — Table `api_keys`** : migration `CREATE TABLE api_keys` (non-breaking, nouvelle table) avec colonnes : `id` (PK BIGINT AUTO_INCREMENT), `company_id` (BIGINT NOT NULL, FK `companies(id)` ON DELETE RESTRICT), `created_by_user_id` (BIGINT NOT NULL, FK `users(id)` ON DELETE RESTRICT), `name` (VARCHAR(255) NOT NULL, CHECK non-vide après trim), `key_hash` (CHAR(64) NOT NULL, **SHA-256 hex**, UNIQUE), `scope` (VARCHAR NOT NULL, CHECK IN `('read','read-write')`), `expires_at` (DATETIME(3) NULL), `last_used_at` (DATETIME(3) NULL), `revoked_at` (DATETIME(3) NULL), `version` (INT NOT NULL DEFAULT 1), `created_at`/`updated_at` (DATETIME(3)). Index sur `company_id`, sur `created_at DESC`, UNIQUE sur `key_hash`. Entrée ajoutée à `docs/migrations-idempotence-audit.md` (verdict `tracked-by-sqlx`). Aucun bump `kesh_version_min_required` (non-breaking, CLAUDE.md §Migration breaking policy P1).

2. **AC2 — Génération de clé sécurisée** : à la création, génération d'un secret aléatoire ≥ 160 bits via un RNG cryptographique (`OsRng`, cf. `auth/password.rs`), encodé base62, préfixé → format **`kesh_pat_<base62>`**. Le secret en clair n'est **JAMAIS** persisté : seul `SHA-256(token_complet)` est stocké en `key_hash`. Le secret en clair n'est retourné **qu'une seule fois** dans la réponse HTTP de création.

3. **AC3 — Repo `api_keys` scopé company** : fonctions `create_in_tx` (INSERT + retour `(ApiKey, token_clair)`), `list_by_company` (filtre `revoked_at IS NULL` par défaut, option `include_revoked`), `find_by_id_for_company` (multi-tenant scoping, `DbError::NotFound` si autre company — anti-énumération KF-002), `find_active_by_key_hash` (lookup auth : `key_hash = ? AND revoked_at IS NULL AND (expires_at IS NULL OR expires_at > NOW(3))`, retourne `ApiKey` + assez d'info pour construire `CurrentUser`), `revoke_for_company` (soft-delete `revoked_at = NOW(3)` + optimistic lock `version`), `touch_last_used` (UPDATE `last_used_at`, non-transactionnel, eventual consistency acceptable). Enum `ApiKeyScope { Read, ReadWrite }` mappé sqlx (`Type`/`Encode`/`Decode`, calque `Role`/`AccountType`). Struct `ApiKey` avec `Debug` masquant tout secret.

### Backend — middleware auth factorisé

4. **AC4 — `require_auth` accepte JWT cookie OU Bearer PAT** : le middleware `crates/kesh-api/src/middleware/auth.rs::require_auth` est étendu **en collant à sa structure réelle (F-OPUS-2, ground-truth `auth.rs:75-141`)** : il extrait déjà **un `token: String` unifié** depuis le cookie `kesh_access_token` OU (fallback) le header `Authorization: Bearer <v>` (scheme `bearer` matché case-INsensitive, valeur `trim()`), PUIS applique le **gate `if !users_exist → 423 SetupRequired`** (à NE PAS déplacer — flux setup v011-5), PUIS un **unique `jwt::decode(&token)`**. L'extension PAT s'insère : **après extraction de `token` ET après le gate `users_exist`**, brancher — `if token.starts_with("kesh_pat_") { validate_pat(token) } else { jwt::decode(token) }`. Le check est **case-sensitive exact** (octets) : `KESH_PAT_` / `kesh_pat ` tombent en `jwt::decode` (échec 401). ⚠️ **Ne JAMAIS** passer un `kesh_pat_…` à `jwt::decode` (fausse erreur loggée + fuite timing). Un PAT valide produit le **même** `CurrentUser` (avec `api_key_id=Some(id)`) — aucun changement aux handlers ni au router. Token absent → `401` (inchangé) ; PAT invalide/révoqué/expiré → `401 UNAUTHENTICATED`.

5. **AC5 — Identité & fraîcheur (DC2)** : la validation PAT relit l'état COURANT du créateur en DB (rôle, `active`). Créateur inactif → `401`. Le `CurrentUser` porte `user_id = créateur`, `role = rôle courant du créateur`, `company_id = company_id de la clé`. `touch_last_used` est appelé (best-effort, n'échoue pas la requête si l'UPDATE rate).

6. **AC6 — Gate de scope sur la méthode (DC3)** : pour une requête authentifiée par PAT avec `scope='read'`, toute méthode HTTP ≠ GET/HEAD/OPTIONS → `403 FORBIDDEN` (code `API_KEY_READ_ONLY`). `scope='read-write'` → toutes méthodes autorisées (le RBAC rôle existant s'applique ensuite). Le gate ne s'applique **PAS** aux requêtes authentifiées par JWT cookie (UI web inchangée).

### Backend — routes CRUD de gestion + audit

7. **AC7 — Routes `/api/v1/settings/api-keys`** (guard `require_comptable_role`, DC4) :
   - ⚠️ **DC6 — Gestion des clés interdite via PAT** : ces 3 routes sont accessibles **uniquement par session JWT cookie (UI web)**. Les handlers `list`/`create`/`revoke` rejettent en `403 API_KEY_MANAGEMENT_FORBIDDEN` si `current_user.api_key_id.is_some()`. À tester (T-A7).
   - `GET /api/v1/settings/api-keys` → liste **toutes** les clés de la company (actives **et** révoquées), triées `created_at DESC` (appeler `list_by_company(..., include_revoked=true)` — le filtre `revoked_at IS NULL` est **outrepassé ici** pour afficher l'historique). Réponse `{ id, name, scope, createdAt, lastUsedAt, revokedAt, expiresAt, version }`, **jamais** le hash ni le secret.
   - `POST /api/v1/settings/api-keys` (body `{ name, scope, expiresAt? }`) → crée la clé, retourne `{ id, name, scope, createdAt, key: "kesh_pat_…" }` (secret en clair **une seule fois**). Validation `name` non-vide + `scope ∈ {read, read-write}`.
   - `DELETE /api/v1/settings/api-keys/{id}` (body `{ version }`) → révocation soft-delete. `404` si clé absente/autre company.

8. **AC8 — Audit-trail PAT (DC5, issue §6, OLICo Art. 9)** : (a) la **création** et la **révocation** d'une clé sont auditées (`actor_type='user'` — c'est un user UI qui gère ses clés —, action `api_key.created` / `api_key.revoked`, `details_json` avec `name` + `scope`, **jamais** le secret) ; (b) les mutations **via PAT** sur les routes métier kesh-api logguent `actor_type='api_key'`, `actor_api_key_id=<id clé>`, `user_id=<créateur>`, soit via `from_current_user(&current_user, …)` (call-sites catégorie (i)), soit via le paramètre `actor_api_key_id: Option<i64>` threadé dans les helpers catégorie (ii). **Les exceptions résiduelles** (un helper (ii) laissé en `::user`, ou les call-sites kesh-db catégorie (iii)) DOIVENT être listées dans les Completion Notes comme `actor_type='user'` (limitation v0.3) — l'AC n'est PAS « 100 % des mutations PAT en `api_key` » mais « toute mutation PAT dont le call-site dispose d'un contexte d'acteur (i/ii) ». Extension `audit_log` : colonnes `actor_type` + `actor_api_key_id` (migration non-breaking, DEFAULT `'user'`), `NewAuditLogEntry` étendu avec constructeurs rétro-compatibles. **Invariant de non-régression** : sur le chemin JWT (UI web), `from_current_user` produit `actor_type='user'` à l'identique ; les sites d'audit internes kesh-db restent `actor_type='user'` (L1). Aucune entrée d'audit existante ne change de sémantique.

### Backend — i18n des erreurs

11b. **AC11b — i18n backend des 2 codes 403 (F-OPUS-4)** : les 2 nouveaux codes 403 (`API_KEY_READ_ONLY`, `API_KEY_MANAGEMENT_FORBIDDEN`) suivent le pattern `AppError` existant `build_response(status, code, t(key, default))` — la résolution se fait sur la **locale serveur globale** (`init_error_i18n`), PAS sur l'`Accept-Language` du client API. Fournir les 4 traductions Fluent (FR/DE/IT/EN) des nouvelles clés suffit ; **ne PAS** implémenter de négociation `Accept-Language` (hors pattern projet). *(Les clés i18n de la page frontend → 17-2b.)*

### Quality gate

13. **AC13 — Quality gate backend vert** : `cargo fmt --all -- --check`, `cargo build --workspace --all-targets`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace -j1 -- --test-threads=1` (touche kesh-db + tests intégration DB). Couverture : unit (génération clé, SHA-256, enum scope/actor_type, gate de scope), integration (middleware PAT auth, CRUD scopé company, audit). *(E2E Playwright du flux UI → 17-2b ; frontend checks → 17-2b.)*

## Tasks / Subtasks

- [ ] **T-A1 — Migration `api_keys` + audit idempotence** (AC: #1)
  - [ ] Créer `crates/kesh-db/migrations/<timestamp>_api_keys.sql` (calque `20260410000001_bank_accounts.sql` + `20260531000001_bank_accounts_archived.sql`). CHECK `scope IN ('read','read-write')`, CHECK name non-vide, UNIQUE `key_hash`, FK company + created_by_user ON DELETE RESTRICT, index `company_id` + `created_at DESC`.
  - [ ] Ajouter la ligne correspondante à `docs/migrations-idempotence-audit.md` (verdict `tracked-by-sqlx` + justification, CLAUDE.md P5).

- [ ] **T-A2 — Entité `ApiKey` + enum `ApiKeyScope`** (AC: #1, #3)
  - [ ] `crates/kesh-db/src/entities/api_key.rs` : structs `ApiKey` / `NewApiKey`, enum `ApiKeyScope { Read, ReadWrite }` avec `Type<MySql>`/`Encode`/`Decode`/`FromStr`/`as_str` (calque `entities/account.rs` enum `AccountType` L43-66, et `Role` `entities/user.rs`). `impl Debug` masquant tout secret (calque `RefreshToken` `entities/refresh_token.rs` L34-45).
  - [ ] Déclarer `pub mod api_key;` dans `crates/kesh-db/src/entities/mod.rs`.

- [ ] **T-A3 — Repo `api_keys`** (AC: #2, #3)
  - [ ] `crates/kesh-db/src/repositories/api_keys.rs` (calque `repositories/bank_accounts.rs`) : `create_in_tx`, `list_by_company`, `find_by_id_for_company`, `find_active_by_key_hash` (lookup auth, **idéalement JOIN users pour récupérer rôle/active du créateur en 1 requête — DC2**), `revoke_for_company` (optimistic lock `version`), `touch_last_used`.
  - [ ] Génération token : helper `generate_pat()` (RNG crypto ≥ 160 bits → encodage, préfixe `kesh_pat_`) + `sha256_hex(token)` — emplacement `crates/kesh-api/src/auth/` (nouveau `api_key.rs` ou `pat.rs`). **Dépendances (ground-truth Pass 1)** : `sha2 = "0.10"` est **DÉJÀ** dans `crates/kesh-api/Cargo.toml` (L47) → rien à ajouter pour le hash. `rand` n'est que **transitive** (via uuid) et `base62` est **absente du workspace** → à AJOUTER explicitement à `kesh-api/Cargo.toml` : `rand_core = "0.6"` + `getrandom` (ou `rand = "0.8"`) pour `OsRng`. **Encodage : base62 INLINE** (~15 lignes, alphabet `0-9A-Za-z`) — NE PAS ajouter de crate `base62`. ⚠️ L'`OsRng` de `auth/password.rs:17` vient de `argon2::password_hash::rand_core::OsRng` (chemin ré-exporté) — ne PAS supposer `rand::OsRng` accessible sans déclarer la crate. **DRY** : réutiliser le même `OsRng` que le salt Argon2. **(F-OPUS-6)** Pour remplir un buffer ≥ 20 octets (160 bits), importer `OsRng` **ET le trait `RngCore`** depuis le même `rand_core` (réexporté via `argon2::password_hash::rand_core`), puis `OsRng.fill_bytes(&mut buf)` sur `buf: [u8; 20]` — sans le trait `RngCore` en scope, `fill_bytes` est introuvable. Tester l'encodage base62 inline (roundtrip + longueur attendue pour 160+ bits).
  - [ ] Déclarer `pub mod api_keys;` dans `crates/kesh-db/src/repositories/mod.rs`.

- [ ] **T-A4 — Extension `audit_log` (actor_type / actor_api_key_id)** (AC: #8) ⚠️ **ripple — voir DC5**
  - [ ] Migration `<timestamp>_audit_log_actor.sql` : `ALTER TABLE audit_log ADD COLUMN actor_type ENUM('user','api_key') NOT NULL DEFAULT 'user'`, `ADD COLUMN actor_api_key_id BIGINT NULL`. **Non-breaking** (DEFAULT, ADD COLUMN). ⚠️ **Ne PAS toucher à `user_id`** : il reste `NOT NULL FK users(id)` (tout audit a un acteur user — même via PAT, `user_id = créateur de la clé`). Pas de FK sur `actor_api_key_id` (la clé peut être révoquée/supprimée alors que l'audit doit survivre 10 ans — pointeur logique, cohérent `entity_id` sans FK). Ajouter à `migrations-idempotence-audit.md`. Note dialecte MariaDB (`ADD COLUMN`, pas `ALTER COLUMN TYPE`).
  - [ ] Étendre `NewAuditLogEntry` / `AuditLogEntry` (`entities/audit_log.rs`) avec `actor_type: ActorType` + `actor_api_key_id: Option<i64>` + enum `ActorType { User, ApiKey }` (sqlx Type). **Constructeurs rétro-compat** : `NewAuditLogEntry::user(user_id, action, entity_type, entity_id, details)` (= sémantique actuelle) et `::api_key(api_key_id, creator_user_id, action, entity_type, entity_id, details)`.
  - [ ] Adapter `repositories/audit_log.rs::insert_in_tx` (INSERT des 2 nouvelles colonnes) **sans** changer la signature publique si possible. Mettre à jour **tous** les call-sites existants vers `NewAuditLogEntry::user(...)` (refactor mécanique). **Inventaire ground-truth (Pass 1) : ~48 occurrences `NewAuditLogEntry { … }` dans `crates/`** — ~29 kesh-db repos, ~19 kesh-api routes, **+1 dans `crates/kesh-api/src/auth/bootstrap.rs` (hors `routes/`)**. Commande : `grep -rn "NewAuditLogEntry {" crates/`.
  - [ ] Ajouter le helper `NewAuditLogEntry::from_current_user(&CurrentUser, action, entity_type, entity_id, details_json)` (DC5). **Migration par catégorie (DC5 (i)/(ii)/(iii), F-OPUS-1)** : (i) handlers kesh-api avec `&CurrentUser` → `from_current_user` ; (ii) helpers kesh-api à `user_id: i64` nu (`audit_primary_transition`, `emit_global_export_audit`, `emit_report_audit`, `insert_canonical_audit_log`, `accept_one_invoice/split/rule`) → étendre signature avec `actor_api_key_id: Option<i64>` → `::user`/`::api_key` ; (iii) repos kesh-db → `::user`. **Lister dans les Completion Notes toute mutation PAT laissée en `actor_type='user'`** (exception documentée v0.3).
  - [ ] **Cohérence INSERT/SELECT (F-OPUS-3)** : dans `repositories/audit_log.rs`, mettre à jour la const `COLUMNS` (L17 : `+ actor_type, actor_api_key_id`), binder les 2 nouvelles colonnes dans l'`INSERT` de `insert_in_tx`, et garder en **bijection** les champs de `AuditLogEntry` (FromRow) avec le `SELECT {COLUMNS}` du re-fetch (L62) ET de toute fonction de liste (L84) — sinon `sqlx` échoue runtime `ColumnNotFound`.

- [ ] **T-A5 — Middleware auth factorisé (JWT OU PAT)** (AC: #4, #5, #6)
  - [ ] Helper `validate_pat(token, &state.pool) -> Result<(CurrentUser, ApiKeyScope), AppError>` dans `auth/api_key.rs` : `sha256_hex` → `find_active_by_key_hash` → vérifier créateur actif → construire `CurrentUser` (DC2, `api_key_id = Some(id)`) → **`touch_last_used(key_hash)` best-effort** (échec loggé `warn`, n'échoue PAS la requête — eventual consistency, AC5). Retourne aussi le `scope` pour le gate.
  - [ ] Étendre `middleware/auth.rs::require_auth` (L75-141) selon la **structure réelle AC4** : conserver l'extraction du `token` unifié cookie‖bearer ET le gate `users_exist → 423` à leur place ; **après** ces deux étapes, brancher `if token.starts_with("kesh_pat_") { validate_pat } else { jwt::decode }`. Injecter `CurrentUser` (`api_key_id` selon le chemin — `None` pour JWT). Appliquer le **gate de scope** (DC3) **uniquement sur le chemin PAT** : si `scope=Read` et `req.method()` mutante → `403 API_KEY_READ_ONLY`. Chemin JWT inchangé.
  - [ ] **Étendre le struct `CurrentUser` (`middleware/auth.rs:33`) avec `api_key_id: Option<i64>`** et mettre à jour TOUS ses sites de construction (chemin JWT cookie + JWT bearer → `api_key_id: None`). Vérifier que les tests middleware existants compilent (refactor mécanique).
  - [ ] Variants `AppError` : `ApiKeyReadOnly` (403, code `API_KEY_READ_ONLY`) + `ApiKeyManagementForbidden` (403, code `API_KEY_MANAGEMENT_FORBIDDEN`, DC6). Réutiliser `Unauthenticated` (401) pour token invalide/révoqué/expiré. i18n des messages (4 langues, AC11b).

- [ ] **T-A6 — Routes CRUD `/settings/api-keys` + audit création/révocation** (AC: #7, #8)
  - [ ] `crates/kesh-api/src/routes/api_keys.rs` (calque `routes/bank_accounts.rs`) : handlers `list` / `create` / `revoke`. **Garde DC6 en tête de chaque handler** : si `current_user.api_key_id.is_some()` → `403 API_KEY_MANAGEMENT_FORBIDDEN`. `list` appelle `list_by_company(..., include_revoked=true)` (AC7). Payloads + réponses (création retourne le secret clair **une fois**). Audit `api_key.created` / `api_key.revoked` (`actor_type=user`, jamais le secret).
  - [ ] Enregistrer les routes dans `crates/kesh-api/src/lib.rs` sous `comptable_routes` (avant le `.route_layer(require_comptable_role)`, calque enregistrement `bank_accounts` ≈ lib.rs L255-263).
  - [ ] Module `pub mod api_keys;` dans `routes/mod.rs`.

- [ ] **T-A7 — Tests backend** (AC: #13)
  - [ ] Unit : `generate_pat` format + entropie, `sha256_hex` déterminisme, enum `ApiKeyScope`/`ActorType` roundtrip sqlx, gate de scope (GET autorisé / POST refusé en read).
  - [ ] Integration (`#[sqlx::test]`) : CRUD scopé company (isolation cross-company → NotFound), `find_active_by_key_hash` (révoqué/expiré exclus), middleware PAT (Bearer valide → 200 ; read + POST → 403 `API_KEY_READ_ONLY` ; créateur désactivé → 401 ; token inconnu → 401 ; **PAT read-write sur `/settings/api-keys` → 403 `API_KEY_MANAGEMENT_FORBIDDEN`**, DC6), audit `api_key.created`/`api_key.revoked` (`actor_type='user'`), et audit d'**une mutation via PAT sur une route métier kesh-api** → `actor_type='api_key'` + `actor_api_key_id` correct (via `CurrentUser.api_key_id` + `from_current_user`, DC2/DC5). Choisir une route mutante kesh-api simple comme cible (ex. POST `bank_accounts` ou équivalent disponible en read-write).

## Dev Notes

> Contexte hérité de la spec parente 17-2 (convergée 5 passes). 17-2a couvre la **Partie A backend uniquement**. Les notes frontend/doc sont volontairement omises (→ 17-2b/17-2c).

### Cartographie de réutilisation (issue de l'analyse exhaustive parente)

**Auth & middleware** :
- `crates/kesh-api/src/middleware/auth.rs:66-140` — `require_auth` : extrait JWT du cookie `kesh_access_token` ou header `Authorization: Bearer`, décode (HS256, ~1-2 ms), injecte `CurrentUser { user_id, role, company_id, exp }` (struct à `auth.rs:33`, derive à L32). **Point d'injection PAT = ici** ; struct à étendre avec `api_key_id: Option<i64>` (DC2).
- `crates/kesh-api/src/middleware/rbac.rs:19-40` — `check_role` + `require_admin_role` / `require_comptable_role` (guards réutilisables, appliqués en `route_layer` APRÈS `require_auth`).
- `crates/kesh-api/src/lib.rs` (router) — routes publiques (health/login/logout/refresh/setup) vs `protected` sous-routeur ; `comptable_routes` avec `route_layer(require_comptable_role)`. Enregistrer `/settings/api-keys` ici (calque `bank_accounts`).
- `crates/kesh-api/src/errors.rs` — `Unauthenticated(String)`→401, `Forbidden`→403, `build_response` helper. Ajouter `ApiKeyReadOnly`→403 + `ApiKeyManagementForbidden`→403.
- `crates/kesh-api/src/auth/jwt.rs` — JWT n'interroge PAS la DB par requête (claims de confiance). **DC2 diverge volontairement** pour le PAT (relire le créateur) → sécurité accrue.

**Hashing / tokens / RNG (DC1)** :
- `crates/kesh-api/src/auth/password.rs:1-210` — Argon2id (`hash_password_async`/`verify_password_async`, ~50 ms, `spawn_blocking`). **NE PAS utiliser pour le PAT**. `OsRng` (L24, via `argon2::password_hash::rand_core::OsRng`) réutilisable pour le secret.
- `crates/kesh-db/src/repositories/refresh_tokens.rs` + `migrations/20260405000001_auth_refresh_tokens.sql` — précédent « secret aléatoire à lookup rapide » : UUID v4 stocké **en clair** + UNIQUE index → `find_active_by_token`. **Le PAT améliore** : on stocke `SHA-256(token)` (jamais le clair). Reprendre le pattern lookup indexé + révocation `revoked_at` + masquage `Debug`.
- `sha2 = "0.10"` présent (`kesh-api/Cargo.toml` L47) ; `rand_core`/`getrandom` à AJOUTER ; base62 INLINE (pas de crate).

**Audit (DC5, ripple)** :
- `migrations/20260413000001_audit_log.sql` — colonnes actuelles : `id, user_id (FK NOT NULL), action, entity_type, entity_id, details_json, created_at`. **PAS** de `actor_type`/`actor_api_key_id` → à ajouter.
- `crates/kesh-db/src/entities/audit_log.rs:28-48` — `AuditLogEntry` / `NewAuditLogEntry` (tous champs obligatoires, `user_id: i64`, champ réel `details_json: Option<serde_json::Value>` L35/46). À étendre + constructeurs rétro-compat.
- `crates/kesh-db/src/repositories/audit_log.rs:17/26-69/62/84` — const `COLUMNS` L17, `insert_in_tx` L26-69, re-fetch SELECT L62, liste L84. Adapter INSERT + COLUMNS + FromRow en bijection (F-OPUS-3).
- Call-sites : `grep -rn "NewAuditLogEntry {" crates/` (~48, dont `auth/bootstrap.rs`) pour le refactor mécanique vers `::user(...)`.

**CRUD reference (pattern canonique = `bank_accounts`, Story v014-1)** :
- `entities/bank_account.rs` (struct + New), `entities/account.rs:43-66` (enum sqlx mapping), `repositories/bank_accounts.rs` (CRUD scopé company + optimistic lock + soft-delete + audit dans tx), `routes/bank_accounts.rs` (handlers, payloads, extraction `Extension<CurrentUser>`), `lib.rs` ≈ L255-263 (enregistrement bank_accounts).

### Invariants à NE PAS casser (régressions)

- **Auth JWT cookie UI** : le chemin JWT de `require_auth` doit rester strictement inchangé (cookie prioritaire, fallback `Bearer <jwt>`, claims de confiance, pas de DB par requête). Le PAT est un **chemin additionnel** activé uniquement si le bearer commence par `kesh_pat_`.
- **Gate `users_exist → 423`** : NE PAS le déplacer (flux setup v011-5). L'extension PAT s'insère **après** ce gate.
- **Tous les call-sites `NewAuditLogEntry` existants** : doivent conserver `actor_type='user'` (constructeur `::user`). Aucune entrée d'audit existante ne change de sémantique.
- **Multi-tenant scoping KF-002** : toute requête PAT est scopée au `company_id` de la clé. Jamais d'accès cross-company.
- **Migrations non-breaking** : `CREATE TABLE api_keys` + `ADD COLUMN` audit avec DEFAULT → aucun bump `kesh_version_min_required`. Confirmer au code-review (CLAUDE.md P3).

### Sécurité — points d'attention

- **Jamais de secret en clair persisté ni loggé** (`Debug` masqué, audit `details` sans secret, réponses list sans hash).
- **Rate-limiting PAT** : hors MVP (issue §7). Documenter comme limitation (risque : PAT durée illimitée si pas d'expiration). Mitigation v0.3 possible.
- **Expiration** : `expires_at` optionnel ; si absent, clé permanente jusqu'à révocation. Le lookup auth filtre `expires_at > NOW(3)`.
- **Révocation immédiate** : soft-delete `revoked_at` → `find_active_by_key_hash` exclut → 401 dès la requête suivante.
- **2FA / scope fine-grained** : hors scope v0.2 (issue §2). Documenter.

### Project Structure Notes

- Séparation respectée : persistance (`kesh-db` : entities/repos/migrations) vs HTTP (`kesh-api` : auth/middleware/routes/errors). Le PAT s'inscrit exactement dans les patterns établis (`bank_accounts` CRUD, `refresh_tokens` secret-lookup, `require_auth` middleware).
- Aucune variance de structure détectée. Le seul point d'attention structurel est le **ripple audit** (DC5/T-A4) qui touche des call-sites hors du périmètre direct de la feature → c'est précisément ce qui justifie 17-2a comme story-foundation isolée.

### Dépendances inter-stories

- **17-2a (cette story)** : aucune dépendance amont autre que 17-1 (mergée). **Bloque** 17-2b et 17-2c.
- **17-2b (frontend)** : dépend de 17-2a (routes CRUD `/settings/api-keys` + format réponse). Consommera `key: "kesh_pat_…"` one-time (AC7 POST).
- **17-2c (doc)** : dépend de 17-2a (comportement final auth Bearer + scopes + codes 403).

### References

- [Source: GitHub #100] — issue d'origine (motivation, 8 sections techniques, Option A retenue).
- [Source: _bmad-output/implementation-artifacts/17-2-api-pat-integrations.md] — **spec parente convergée (5 passes validate), source des DC1-DC6, AC, tasks Partie A et de toute la cartographie**.
- [Source: _bmad-output/planning-artifacts/epic-17.md#Story 17-2 (100-116) + D3/D4/D5] — périmètre, split.
- [Source: crates/kesh-api/src/middleware/auth.rs:66-140] — `require_auth`, point d'injection PAT.
- [Source: crates/kesh-api/src/middleware/rbac.rs:19-41] — guards de rôle.
- [Source: crates/kesh-api/src/auth/password.rs:1-210] — Argon2id (à NE PAS utiliser per-request, DC1) + `OsRng`.
- [Source: crates/kesh-db/src/repositories/refresh_tokens.rs + migrations/20260405000001_auth_refresh_tokens.sql] — précédent secret-lookup indexé.
- [Source: crates/kesh-db/migrations/20260413000001_audit_log.sql + entities/audit_log.rs + repositories/audit_log.rs] — audit à étendre (DC5).
- [Source: crates/kesh-db/src/repositories/bank_accounts.rs + routes/bank_accounts.rs + entities/account.rs:43-66] — pattern CRUD + enum sqlx de référence.
- [Source: crates/kesh-api/src/lib.rs] — câblage router (`comptable_routes`, route_layer).
- [Source: CLAUDE.md §Migration breaking policy P1/P3/P5, §Règle de splitting préventif, §Pattern batch] — contraintes process.

## Change Log — split

**Créée 2026-06-05 par scission de la story 17-2** (`bmad-create-story 17-2a`). 17-2a = **Partie A backend** (T-A1..T-A7), extraite verbatim de la spec parente `17-2-api-pat-integrations.md` **convergée en 5 passes validate** (Sonnet→Haiku→Opus→Sonnet→Haiku, 0 > LOW, 24 patches) — le contenu backend a donc déjà subi 5 passes adversariales, dont le catch-architectural Opus Pass 3 (F-OPUS-1 reframe ripple audit en 3 catégories de call-sites). Aucun re-validate n'est requis sur ce contenu déjà convergé ; un `bmad-create-story validate 17-2a` reste optionnel si l'on veut re-challenger la frontière de scope du split lui-même.

- **Scope retiré (→ 17-2b)** : AC9 (page `/settings/api-keys`), AC10 (affichage one-time + copie HTTP LAN), AC11 portion frontend (clés i18n `api-keys-*` de la page), T-B1..T-B4, E2E Playwright UI, checks frontend.
- **Scope retiré (→ 17-2c)** : AC12 (`docs/api-external.md`, OpenAPI), T-C1..T-C3, synchro docs visibles.
- **Scope conservé (17-2a)** : AC1-AC8 + AC11b (i18n backend des 2 codes 403, ex-portion backend d'AC11) + AC13 (quality gate backend). DC1-DC6 intégrales.
- **Renumérotation AC** : l'AC i18n backend (ex-AC11 portion F-OPUS-4) est renommée **AC11b** pour éviter la collision avec l'AC11 frontend de 17-2b. Les numéros AC1-AC8 et AC13 sont conservés à l'identique de la spec parente pour traçabilité.

## Dev Agent Record

### Agent Model Used

(à compléter par dev-story — **recommandation : Opus** pour le middleware auth factorisé + le ripple audit cross-call-site. Le ripple audit n'est PAS purement mécanique : F-OPUS-1 distingue 3 catégories de call-sites (i handlers `&CurrentUser` / ii helpers `user_id` nu / iii repos kesh-db). Attention particulière à T-A4 (~48 call-sites) et T-A5 (extension `CurrentUser` + insertion PAT post-gate `users_exist`).)

### Debug Log References

### Completion Notes List

### File List
