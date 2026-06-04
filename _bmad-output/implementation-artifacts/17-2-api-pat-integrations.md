# Story 17.2: API externe à clé PAT (Personal Access Token) pour intégrations IA & logiciels tiers

Status: ready-for-dev

<!-- Note: Validation is optional. Run validate-create-story for quality check before dev-story. -->

## Story

As a **PME utilisant Kesh**,
I want **générer des clés d'accès API (lecture seule ou lecture-écriture) liées à une company, présentées par header `Authorization: Bearer kesh_pat_…`**,
so that **une IA externe (Claude API, ChatGPT, agent custom) ou un logiciel tiers (script Python, ETL, dashboard BI, ERP) puisse consommer mes données comptables sans partager mes identifiants utilisateur**.

## Contexte & provenance

- **Issue GitHub** : [#100](https://github.com/guycorbaz/kesh/issues/100) `[CR] API externes avec clé PAT (lecture / lecture-écriture) pour intégrations IA & logiciels tiers` — labellisée `enhancement` + `v0.2-milestone`.
- **Epic** : 17 « Infra & Souveraineté » — Story 17-2 (cf. `_bmad-output/planning-artifacts/epic-17.md` décisions D4/D5). Vient **après** la story-zéro sécurité 17-1 (TOCTOU, mergée PR #163) qui a sécurisé le bootstrap.
- **Option retenue (D5 + issue §5)** : **Option A** — réutiliser les routes `/api/v1/*` existantes via un middleware d'auth factorisé qui accepte SOIT le cookie session JWT (UI web) SOIT le Bearer PAT (API externe). Pas de routes dédiées `/external/*` (évite la duplication).
- **Scope multi-tenant (D + issue §3)** : strict **1 `company_id` par clé** (cohérent multi-tenant scoping Epic 7-1 KF-002).

> ⚠️ **Story « split quasi-certain »** (epic-17.md D3, split pressenti 17-2a backend / 17-2b frontend / 17-2c doc/OpenAPI). Décision de split à trancher au `bmad-create-story validate` selon la règle de splitting préventif CLAUDE.md (> 5 modules OU > 4 passes validate). Les tâches ci-dessous sont **groupées par partie** (A/B/C) pour matérialiser le découpage probable. Voir §« Découpage proposé ».

## Décisions de conception à confirmer au validate

Ces points engagent la sécurité/perf ; ils sont **pré-tranchés ici avec justification** mais ouverts au challenge adversarial du validate :

- **DC1 — Stockage du token = SHA-256 indexé, PAS Argon2id.** L'issue #100 §1 suggère « Argon2id ou bcrypt », mais un verify Argon2id (~50 ms, cf. `auth/password.rs`) **à chaque requête API** est inacceptable (50 ms × N req/s). Un PAT est un secret **aléatoire à haute entropie** (≥ 160 bits), pas un mot de passe utilisateur faible — le hashing lent (anti-bruteforce) n'apporte rien. On stocke **`SHA-256(token)`** (hex 64 chars) avec **index UNIQUE** → lookup `O(1)` par un seul SELECT indexé (~5 ms), pattern standard industrie (GitHub PAT). C'est une amélioration de sécurité vs le précédent `refresh_tokens` qui stocke le token **en clair** (`entities/refresh_token.rs:25`, `pub token: String`) : ici jamais de secret réversible en DB. **Aucun secret en clair, aucun Argon2 per-request.**
- **DC2 — Identité effective du PAT + extension `CurrentUser`.** Le PAT authentifie **au nom de son créateur** : le middleware charge la row `api_keys` (active, non révoquée, non expirée), puis construit le `CurrentUser` avec `user_id = créateur`, `role = rôle COURANT du créateur (relu en DB)`, `company_id = company_id de la clé`. Relire le rôle/état du créateur en DB (≠ JWT qui fait confiance aux claims) garantit qu'une **désactivation du créateur ou un changement de rôle invalide immédiatement le PAT** (sécurité). Coût : 1 SELECT user supplémentaire (jointure possible avec le lookup clé). **⚠️ Le struct `CurrentUser` (`middleware/auth.rs:33`, champs actuels `{ user_id, role, company_id, exp }`) est ÉTENDU avec `api_key_id: Option<i64>`** (chemin JWT → `None` ; chemin PAT → `Some(<id clé>)`). C'est ce champ qui permet à l'audit des routes métier de distinguer `actor_type='user'` vs `'api_key'` (cf. DC5/AC8) — sans lui, les handlers n'ont aucun moyen de savoir qu'ils sont appelés via PAT. **Tous les sites de construction de `CurrentUser` (chemin JWT inclus) doivent fournir `api_key_id: None`.**
- **DC3 — Application du scope = gate sur la méthode HTTP.** `scope='read'` → **seules les méthodes GET (et HEAD/OPTIONS) sont autorisées** ; toute autre méthode (POST/PUT/PATCH/DELETE) → `403 FORBIDDEN`. `scope='read-write'` → toutes méthodes (sous réserve du RBAC rôle existant). La permission effective = **intersection(rôle du créateur, scope de la clé)** : les guards de rôle existants (`require_admin_role`/`require_comptable_role`) continuent de s'appliquer normalement.
- **DC4 — Routes de gestion gated Comptable+ (pas Admin-only).** La page `/settings/api-keys` est per-company ; un Comptable doit pouvoir gérer ses intégrations. Aligné sur le guard CRUD existant des `bank_accounts` (`require_comptable_role`). **À confirmer au validate** (Admin-only est une alternative plus stricte défendable).
- **DC5 — Extension `audit_log` non-breaking + portée de l'attribution `api_key`.** Ajout `actor_type ENUM('user','api_key') NOT NULL DEFAULT 'user'` + `actor_api_key_id BIGINT NULL`. `DEFAULT 'user'` rend la migration non-breaking pour les lignes existantes. ⚠️ **Impact ripple (mesuré : ~48 call-sites `NewAuditLogEntry { … }` dans `crates/`, dont kesh-db repos ~29, kesh-api routes ~19, `kesh-api/src/auth/bootstrap.rs` 1)** : l'ajout de champs casserait la compilation de chaque call-site. Mitigation = constructeurs rétro-compatibles sur `NewAuditLogEntry` : `::user(user_id, action, entity_type, entity_id, details)` (= sémantique ACTUELLE, `actor_type=User`, `actor_api_key_id=None`) et `::api_key(api_key_id, creator_user_id, action, entity_type, entity_id, details)`. **Portée de l'attribution `actor_type='api_key'` (résout la contradiction F1 Pass 1)** :
  - **Sites au niveau route kesh-api** (qui ont `Extension<CurrentUser>` en scope) : remplacés par un helper `NewAuditLogEntry::from_current_user(&current_user, action, entity_type, entity_id, details)` qui mappe `current_user.api_key_id` → `None`⇒`actor_type=User` / `Some(id)`⇒`actor_type=ApiKey, actor_api_key_id=id, user_id=current_user.user_id`. Comportement **préservé** pour le chemin JWT (toujours `User`). C'est ce qui réalise AC8(b) pour les mutations métier via PAT.
  - **Sites internes kesh-db repos** (qui reçoivent un `user_id: i64` sans contexte d'acteur) : migrés mécaniquement vers `::user(...)` → restent `actor_type='user'`. Propager le contexte PAT jusqu'à ces couches profondes nécessiterait de changer N signatures de repo → **hors scope v0.2, limitation L1 documentée** (l'imputabilité reste assurée : l'événement `api_key.created` lie la clé à son `created_by_user_id`, et les mutations métier via PAT au niveau route portent bien `actor_type='api_key'`). À réévaluer v0.3 si couverture exhaustive requise.

## Acceptance Criteria

### Backend — table, repo, génération

1. **AC1 — Table `api_keys`** : migration `CREATE TABLE api_keys` (non-breaking, nouvelle table) avec colonnes : `id` (PK BIGINT AUTO_INCREMENT), `company_id` (BIGINT NOT NULL, FK `companies(id)` ON DELETE RESTRICT), `created_by_user_id` (BIGINT NOT NULL, FK `users(id)` ON DELETE RESTRICT — responsabilité conservée), `name` (VARCHAR(255) NOT NULL, CHECK non-vide après trim), `key_hash` (CHAR(64) NOT NULL, **SHA-256 hex**, UNIQUE), `scope` (VARCHAR NOT NULL, CHECK IN `('read','read-write')`), `expires_at` (DATETIME(3) NULL), `last_used_at` (DATETIME(3) NULL), `revoked_at` (DATETIME(3) NULL), `version` (INT NOT NULL DEFAULT 1), `created_at`/`updated_at` (DATETIME(3)). Index sur `company_id`, sur `created_at DESC`, UNIQUE sur `key_hash`. Entrée ajoutée à `docs/migrations-idempotence-audit.md` (verdict `tracked-by-sqlx`). Aucun bump `kesh_version_min_required` (non-breaking, cf. CLAUDE.md §Migration breaking policy P1).

2. **AC2 — Génération de clé sécurisée** : à la création, génération d'un secret aléatoire ≥ 160 bits via un RNG cryptographique (`OsRng`/`rand`, cf. crate déjà présente pour le salt Argon2 `auth/password.rs`), encodé base62, préfixé → format **`kesh_pat_<base62>`**. Le secret en clair n'est **JAMAIS** persisté : seul `SHA-256(token_complet)` est stocké en `key_hash`. Le secret en clair n'est retourné **qu'une seule fois** dans la réponse HTTP de création.

3. **AC3 — Repo `api_keys` scopé company** : fonctions `create_in_tx` (INSERT + retour `(ApiKey, token_clair)`), `list_by_company` (filtre `revoked_at IS NULL` par défaut, option `include_revoked`), `find_by_id_for_company` (multi-tenant scoping, `DbError::NotFound` si autre company — anti-énumération KF-002), `find_active_by_key_hash` (lookup auth : `key_hash = ? AND revoked_at IS NULL AND (expires_at IS NULL OR expires_at > NOW(3))`, retourne `ApiKey` + assez d'info pour construire `CurrentUser`), `revoke_for_company` (soft-delete `revoked_at = NOW(3)` + optimistic lock `version`), `touch_last_used` (UPDATE `last_used_at`, non-transactionnel, eventual consistency acceptable). Enum `ApiKeyScope { Read, ReadWrite }` mappé sqlx (`Type`/`Encode`/`Decode`, calque `Role`/`AccountType`). Struct `ApiKey` avec `Debug` masquant tout secret.

### Backend — middleware auth factorisé

4. **AC4 — `require_auth` accepte JWT cookie OU Bearer PAT** : le middleware `crates/kesh-api/src/middleware/auth.rs::require_auth` est étendu. **Ordre de discrimination non ambigu (résout F2 Pass 1)** : (1) cookie `kesh_access_token` présent → `jwt::decode` (chemin actuel, prioritaire, inchangé) ; sinon (2) header `Authorization: Bearer <v>` présent : **inspecter le préfixe AVANT tout decode** — si `<v>` commence par `kesh_pat_` → `validate_pat(<v>)` (chemin PAT) ; sinon → `jwt::decode(<v>)` (chemin JWT bearer actuel inchangé) ; sinon (3) aucun credential → `401`. ⚠️ **Ne JAMAIS** passer un token `kesh_pat_…` à `jwt::decode` (évite une fausse erreur JWT loggée + fuite de timing). Un PAT valide produit le **même** `CurrentUser` (avec `api_key_id=Some(id)`, DC2) injecté dans les extensions — aucun changement aux handlers ni au router. Token absent/invalide/révoqué/expiré → `401 UNAUTHENTICATED`.

5. **AC5 — Identité & fraîcheur (DC2)** : la validation PAT relit l'état COURANT du créateur en DB (rôle, `active`). Créateur inactif → `401`. Le `CurrentUser` porte `user_id = créateur`, `role = rôle courant du créateur`, `company_id = company_id de la clé`. `touch_last_used` est appelé (best-effort, n'échoue pas la requête si l'UPDATE rate).

6. **AC6 — Gate de scope sur la méthode (DC3)** : pour une requête authentifiée par PAT avec `scope='read'`, toute méthode HTTP ≠ GET/HEAD/OPTIONS → `403 FORBIDDEN` (code `API_KEY_READ_ONLY`). `scope='read-write'` → toutes méthodes autorisées (le RBAC rôle existant s'applique ensuite). Le gate ne s'applique **PAS** aux requêtes authentifiées par JWT cookie (UI web inchangée).

### Backend — routes CRUD de gestion + audit

7. **AC7 — Routes `/api/v1/settings/api-keys`** (guard `require_comptable_role`, DC4) :
   - `GET /api/v1/settings/api-keys` → liste **toutes** les clés de la company (actives **et** révoquées), triées `created_at DESC`, pour que l'utilisateur voie l'historique (le champ `revokedAt` est alors signifiant). Le filtre `revoked_at IS NULL` de `list_by_company` est donc **outrepassé ici** (appeler avec `include_revoked=true`). Réponse `{ id, name, scope, createdAt, lastUsedAt, revokedAt, expiresAt, version }`, **jamais** le hash ni le secret.
   - `POST /api/v1/settings/api-keys` (body `{ name, scope, expiresAt? }`) → crée la clé, retourne `{ id, name, scope, createdAt, key: "kesh_pat_…" }` (secret en clair **une seule fois**). Validation `name` non-vide + `scope ∈ {read, read-write}`.
   - `DELETE /api/v1/settings/api-keys/{id}` (body `{ version }`) → révocation soft-delete. `404` si clé absente/autre company.

8. **AC8 — Audit-trail PAT (DC5, issue §6, OLICo Art. 9)** : (a) la **création** et la **révocation** d'une clé sont auditées (`actor_type='user'` — c'est un user UI qui gère ses clés —, action `api_key.created` / `api_key.revoked`, `details_json` avec `name` + `scope`, **jamais** le secret) ; (b) toute action **mutante via PAT** sur une route métier kesh-api logge `actor_type='api_key'`, `actor_api_key_id=<id clé>`, `user_id=<créateur>` (responsabilité conservée), via le helper `NewAuditLogEntry::from_current_user(&current_user, …)` (DC5) — possible car `CurrentUser.api_key_id = Some(id)` sur le chemin PAT (DC2). Extension `audit_log` : colonnes `actor_type` + `actor_api_key_id` ajoutées (migration non-breaking, DEFAULT `'user'`), `NewAuditLogEntry` étendu avec constructeurs rétro-compatibles (`::user(...)` = comportement actuel + `::api_key(...)` + `::from_current_user(...)`). **Invariant de non-régression** : sur le chemin JWT (UI web), `from_current_user` produit `actor_type='user'` à l'identique de l'actuel ; les sites d'audit internes kesh-db (sans contexte d'acteur) restent `actor_type='user'` (limitation L1 v0.3, DC5). Aucune entrée d'audit existante ne change de sémantique.

### Frontend — page de gestion

9. **AC9 — Page `/settings/api-keys`** (per-company) : liste (nom, scope, créée le, dernier accès, statut révoqué), bouton « Créer une clé » → modal (nom + select scope read/read-write + expiration optionnelle), bouton « Révoquer » avec confirmation forte. Lien d'accès depuis `/settings`.

10. **AC10 — Affichage one-time + copie sûre HTTP LAN** : après création, la clé `kesh_pat_…` est affichée **une seule fois** avec bouton « Copier » et avertissement « copiez-la maintenant, elle ne sera plus jamais affichée ». ⚠️ **Pas d'API secure-context-only** : la copie utilise `navigator.clipboard.writeText()` **avec fallback** (sélection textarea + `execCommand` ou affichage pour copie manuelle) car `navigator.clipboard` est `undefined` en HTTP LAN (cf. memory `feedback_no_secure_context_apis_http_lan`, bugs #143/#145). Pas de `crypto.randomUUID()` côté front — IDs DOM via `$props.id()` Svelte 5.

11. **AC11 — i18n 4 langues** : toutes les chaînes de la page + messages d'erreur backend internationalisés FR/DE/IT/EN, préfixe feature `api-key-` exclusif (respect `lint-i18n-ownership`).

### Documentation

12. **AC12 — `docs/api-external.md`** : doc d'usage de l'API externe (auth Bearer PAT, scopes read/read-write, scoping company, gestion des clés) avec exemples **`curl`**, **Python** (`requests`), **JavaScript** (`fetch`), et **config MCP server** (Claude API). Mention de la limitation v0.2 (scope binaire global, pas de fine-grained per-ressource). OpenAPI (`utoipa`) **si faisable sans refactor lourd** — sinon documenté comme limitation L-suivante v0.3.

### Quality gate

13. **AC13 — Quality gate vert** : `cargo fmt --all -- --check`, `cargo build --workspace --all-targets`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace -j1 -- --test-threads=1` (touche kesh-db + tests intégration DB). Frontend : `npm run check`, `npm run lint-i18n-ownership`, `npm run test:unit`, `npm run build`. E2E Playwright pour le flux création→usage PAT→révocation. Couverture : unit (génération clé, SHA-256, enum scope, gate de scope), integration (middleware PAT auth, CRUD scopé company, audit), E2E (page + flux Bearer réel).

## Tasks / Subtasks

> Groupées par partie pour matérialiser le split probable. Si le validate confirme le split, ces groupes deviennent 17-2a / 17-2b / 17-2c.

### Partie A — Backend (17-2a pressenti)

- [ ] **T-A1 — Migration `api_keys` + audit idempotence** (AC: #1)
  - [ ] Créer `crates/kesh-db/migrations/<timestamp>_api_keys.sql` (calque `20260410000001_bank_accounts.sql` + `20260531000001_bank_accounts_archived.sql`). CHECK `scope IN ('read','read-write')`, CHECK name non-vide, UNIQUE `key_hash`, FK company + created_by_user ON DELETE RESTRICT, index `company_id` + `created_at DESC`.
  - [ ] Ajouter la ligne correspondante à `docs/migrations-idempotence-audit.md` (verdict `tracked-by-sqlx` + justification, cf. CLAUDE.md P5).

- [ ] **T-A2 — Entité `ApiKey` + enum `ApiKeyScope`** (AC: #1, #3)
  - [ ] `crates/kesh-db/src/entities/api_key.rs` : structs `ApiKey` / `NewApiKey`, enum `ApiKeyScope { Read, ReadWrite }` avec `Type<MySql>`/`Encode`/`Decode`/`FromStr`/`as_str` (calque `entities/account.rs` enum `AccountType` L43-66, et `Role` `entities/user.rs`). `impl Debug` masquant tout secret (calque `RefreshToken` `entities/refresh_token.rs` L34-45).
  - [ ] Déclarer `pub mod api_key;` dans `crates/kesh-db/src/entities/mod.rs`.

- [ ] **T-A3 — Repo `api_keys`** (AC: #2, #3)
  - [ ] `crates/kesh-db/src/repositories/api_keys.rs` (calque `repositories/bank_accounts.rs`) : `create_in_tx`, `list_by_company`, `find_by_id_for_company`, `find_active_by_key_hash` (lookup auth, **idéalement JOIN users pour récupérer rôle/active du créateur en 1 requête — DC2**), `revoke_for_company` (optimistic lock `version`), `touch_last_used`.
  - [ ] Génération token : helper `generate_pat()` (RNG crypto ≥ 160 bits → encodage, préfixe `kesh_pat_`) + `sha256_hex(token)` — emplacement `crates/kesh-api/src/auth/` (nouveau `api_key.rs` ou `pat.rs`). **Dépendances (vérifié ground-truth Pass 1)** : `sha2 = "0.10"` est **DÉJÀ** dans `crates/kesh-api/Cargo.toml` (L47) → rien à ajouter pour le hash. `rand` n'est que **transitive** (via uuid) et `base62` est **absente du workspace** → à AJOUTER explicitement à `kesh-api/Cargo.toml` : `rand_core = "0.6"` + `getrandom` (ou `rand = "0.8"`) pour `OsRng`, et soit `base62 = "2"`, soit **encoder en base62 inline** (~15 lignes, alphabet `0-9A-Za-z`) pour éviter une dépendance d'un mainteneur unique. ⚠️ L'`OsRng` de `auth/password.rs:17` vient de `argon2::password_hash::rand_core::OsRng` (chemin ré-exporté) — ne PAS supposer `rand::OsRng` accessible sans déclarer la crate. **DRY** : réutiliser le même `OsRng` que le salt Argon2.
  - [ ] Déclarer `pub mod api_keys;` dans `crates/kesh-db/src/repositories/mod.rs`.

- [ ] **T-A4 — Extension `audit_log` (actor_type / actor_api_key_id)** (AC: #8) ⚠️ **ripple — voir DC5**
  - [ ] Migration `<timestamp>_audit_log_actor.sql` : `ALTER TABLE audit_log ADD COLUMN actor_type ENUM('user','api_key') NOT NULL DEFAULT 'user'`, `ADD COLUMN actor_api_key_id BIGINT NULL`. **Non-breaking** (DEFAULT). Ajouter à `migrations-idempotence-audit.md`. Note dialecte MariaDB (`ADD COLUMN`, pas `ALTER COLUMN TYPE`).
  - [ ] Étendre `NewAuditLogEntry` / `AuditLogEntry` (`entities/audit_log.rs`) avec `actor_type: ActorType` + `actor_api_key_id: Option<i64>` + enum `ActorType { User, ApiKey }` (sqlx Type). **Constructeurs rétro-compat** : `NewAuditLogEntry::user(user_id, action, entity_type, entity_id, details)` (= sémantique actuelle, `actor_type=User`, `actor_api_key_id=None`) et `::api_key(api_key_id, creator_user_id, action, entity_type, entity_id, details)`.
  - [ ] Adapter `repositories/audit_log.rs::insert_in_tx` (INSERT des 2 nouvelles colonnes) **sans** changer la signature publique si possible (le `NewAuditLogEntry` porte déjà les nouveaux champs). Mettre à jour **tous** les call-sites existants vers `NewAuditLogEntry::user(...)` (refactor mécanique). **Inventaire ground-truth (Pass 1) : ~48 occurrences `NewAuditLogEntry { … }` dans `crates/`** — ~29 kesh-db repos (fiscal_years, journal_entries, invoices, contacts, products, bank_profiles, accounts, reconciliation_rules + tests), ~19 kesh-api routes, **+1 dans `crates/kesh-api/src/auth/bootstrap.rs` (hors `routes/` — ne pas l'oublier)**. Commande : `grep -rn "NewAuditLogEntry {" crates/`.
  - [ ] Ajouter le helper `NewAuditLogEntry::from_current_user(&CurrentUser, action, entity_type, entity_id, details)` (DC5) pour les call-sites au niveau route kesh-api ; migrer ces ~19 sites vers ce helper (réalise AC8(b)). Les ~29 sites kesh-db repos → `::user(...)` (limitation L1, DC5).

- [ ] **T-A5 — Middleware auth factorisé (JWT OU PAT)** (AC: #4, #5, #6)
  - [ ] Helper `validate_pat(token, &state.pool) -> Result<(CurrentUser, ApiKeyScope), AppError>` dans `auth/api_key.rs` : `sha256_hex` → `find_active_by_key_hash` → vérifier créateur actif → construire `CurrentUser` (DC2). Retourne aussi le `scope` pour le gate.
  - [ ] Étendre `middleware/auth.rs::require_auth` (L66-140) selon l'**ordre de discrimination AC4** : cookie → JWT ; bearer préfixé `kesh_pat_` → `validate_pat` (sans passer par `jwt::decode`) ; bearer non préfixé → JWT. Injecter `CurrentUser` (avec `api_key_id` renseigné selon le chemin — `None` pour JWT). Appliquer le **gate de scope** (DC3) : si PAT `scope=Read` et `req.method()` mutante (≠ GET/HEAD/OPTIONS) → `403 API_KEY_READ_ONLY`. Chemin JWT (cookie ou bearer non-PAT) → comportement strictement inchangé.
  - [ ] **Étendre le struct `CurrentUser` (`middleware/auth.rs:33`) avec `api_key_id: Option<i64>`** et mettre à jour TOUS ses sites de construction (chemin JWT cookie + JWT bearer → `api_key_id: None`). Vérifier que les tests middleware existants compilent (refactor mécanique).
  - [ ] Variants `AppError` : `ApiKeyReadOnly` (403). Réutiliser `Unauthenticated` (401) pour token invalide. i18n des messages.

- [ ] **T-A6 — Routes CRUD `/settings/api-keys` + audit création/révocation** (AC: #7, #8)
  - [ ] `crates/kesh-api/src/routes/api_keys.rs` (calque `routes/bank_accounts.rs`) : handlers `list` / `create` / `revoke`. Payloads + réponses (création retourne le secret clair **une fois**). Audit `api_key.created` / `api_key.revoked` (`actor_type=user`, jamais le secret).
  - [ ] Enregistrer les routes dans `crates/kesh-api/src/lib.rs` sous `comptable_routes` (avant le `.route_layer(require_comptable_role)`, calque enregistrement `bank_accounts` (≈ lib.rs L255-263)).
  - [ ] Module `pub mod api_keys;` dans `routes/mod.rs`.

- [ ] **T-A7 — Tests backend** (AC: #13)
  - [ ] Unit : `generate_pat` format + entropie, `sha256_hex` déterminisme, enum `ApiKeyScope`/`ActorType` roundtrip sqlx, gate de scope (GET autorisé / POST refusé en read).
  - [ ] Integration (`#[sqlx::test]`) : CRUD scopé company (isolation cross-company → NotFound), `find_active_by_key_hash` (révoqué/expiré exclus), middleware PAT (Bearer valide → 200 ; read + POST → 403 `API_KEY_READ_ONLY` ; créateur désactivé → 401 ; token inconnu → 401), audit `api_key.created`/`api_key.revoked` (`actor_type='user'`), et audit d'**une mutation via PAT sur une route métier kesh-api** → `actor_type='api_key'` + `actor_api_key_id` correct (implémentable via `CurrentUser.api_key_id` + `from_current_user`, DC2/DC5). Choisir une route mutante kesh-api simple comme cible (ex. POST `bank_accounts` ou équivalent disponible en read-write).

### Partie B — Frontend (17-2b pressenti)

- [ ] **T-B1 — Feature `api-keys` (API client + types)** (AC: #9, #11)
  - [ ] `frontend/src/lib/features/api-keys/` : `api-keys.api.ts` (`listApiKeys`/`createApiKey`/`revokeApiKey`), `api-keys.types.ts` (calque `features/bank-accounts/`).

- [ ] **T-B2 — Page `/settings/api-keys` + modal + révocation** (AC: #9, #10)
  - [ ] `frontend/src/routes/(app)/settings/api-keys/+page.svelte` (calque `bank-accounts/+page.svelte` + structure `settings/+page.svelte`) : liste, modal création, affichage one-time de la clé + bouton copier (fallback HTTP LAN, AC10), confirmation de révocation.
  - [ ] Lien « Clés API » depuis `frontend/src/routes/(app)/settings/+page.svelte`.

- [ ] **T-B3 — i18n FR/DE/IT/EN** (AC: #11)
  - [ ] Clés `api-key-*` dans les 4 locales (`kesh-i18n` + locales frontend selon convention). Respect `lint-i18n-ownership` (préfixe feature exclusif).

- [ ] **T-B4 — E2E Playwright** (AC: #13)
  - [ ] Scénario : login → `/settings/api-keys` → créer clé (read-write) → copier → utiliser le token réel sur un `GET /api/v1/...` (200) et un `POST` (selon scope) → révoquer → réutilisation → 401. (Pré-requis Playwright Ubuntu cf. memory.)

### Partie C — Documentation (17-2c pressenti)

- [ ] **T-C1 — `docs/api-external.md`** (AC: #12)
  - [ ] Auth Bearer, scopes, scoping company, gestion des clés, exemples curl/Python/JS/MCP. Limitations v0.2.
- [ ] **T-C2 — OpenAPI `utoipa` (si faisable)** (AC: #12)
  - [ ] Évaluer le coût d'annoter les routes externes avec `utoipa`. Si > effort raisonnable → documenter en limitation v0.3 plutôt que forcer.
- [ ] **T-C3 — Synchro docs visibles** (CLAUDE.md §Synchroniser TOUTES les docs)
  - [ ] CHANGELOG § Added (API externe PAT). README « Fonctionnalités » si la feature y figure. Manuel admin LaTeX si la config/sécurité change (probable : section « Clés API »).

## Découpage proposé (à trancher au validate)

| Sous-story | Périmètre | Modules touchés |
|---|---|---|
| **17-2a** backend | T-A1..T-A7 : table, repo, middleware factorisé, CRUD, audit, tests | kesh-db (migrations, entities, repos ×2 + audit), kesh-api (auth, middleware, routes, errors), kesh-i18n |
| **17-2b** frontend | T-B1..T-B4 : feature, page, i18n, E2E | frontend (features, routes, i18n) |
| **17-2c** doc | T-C1..T-C3 : api-external.md, OpenAPI, synchro docs | docs, README, manuels |

La règle de splitting préventif CLAUDE.md (> 5 modules) est **dépassée par la Partie A seule** (kesh-db audit + entities + repos + kesh-api auth/middleware/routes). Le pattern recommandé « story-zéro pose le pattern + rollout » suggère : **17-2a comme story foundation** (la plus risquée : middleware auth + audit ripple), puis 17-2b/17-2c en rollout mécanique. **Recommandation forte : splitter en 17-2a/b/c au validate.**

## Dev Notes

### Cartographie de réutilisation (issue de l'analyse exhaustive)

**Auth & middleware** :
- `crates/kesh-api/src/middleware/auth.rs:66-140` — `require_auth` : extrait JWT du cookie `kesh_access_token` ou header `Authorization: Bearer`, décode (HS256, ~1-2 ms), injecte `CurrentUser { user_id, role, company_id, exp }` (struct à `auth.rs:33`, derive à L32) dans les extensions. **Point d'injection PAT = ici** ; struct à étendre avec `api_key_id: Option<i64>` (DC2).
- `crates/kesh-api/src/middleware/rbac.rs:19-40` — `check_role` + `require_admin_role` / `require_comptable_role` (guards réutilisables, appliqués en `route_layer` APRÈS `require_auth`).
- `crates/kesh-api/src/lib.rs` (router) — routes publiques (health/login/logout/refresh/setup) vs `protected` sous-routeur ; `comptable_routes` avec `route_layer(require_comptable_role)`. Enregistrer `/settings/api-keys` ici (calque `bank_accounts`).
- `crates/kesh-api/src/errors.rs` — `Unauthenticated(String)`→401, `Forbidden`→403, `build_response` helper. Ajouter `ApiKeyReadOnly`→403.
- `crates/kesh-api/src/auth/jwt.rs` — JWT n'interroge PAS la DB par requête (claims de confiance). **DC2 diverge volontairement** pour le PAT (relire le créateur) → sécurité accrue.

**Hashing / tokens / RNG (DC1)** :
- `crates/kesh-api/src/auth/password.rs:1-210` — Argon2id (`hash_password_async`/`verify_password_async`, ~50 ms, `spawn_blocking`). **NE PAS utiliser pour le PAT** (trop lent par requête). `OsRng` (L24) réutilisable pour le secret.
- `crates/kesh-db/src/repositories/refresh_tokens.rs` + `migrations/20260405000001_auth_refresh_tokens.sql` — précédent « secret aléatoire à lookup rapide » : UUID v4 stocké **en clair** + UNIQUE index → `find_active_by_token` (SELECT indexé). **Le PAT améliore** : on stocke `SHA-256(token)` (jamais le clair). Reprendre le pattern lookup indexé + révocation `revoked_at` + masquage `Debug`.
- Crate `uuid` présente ; `sha2` à vérifier/ajouter ; encodage base62 (vérifier crate ou implémenter simple).

**Audit (DC5, ripple)** :
- `migrations/20260413000001_audit_log.sql` — colonnes actuelles : `id, user_id (FK NOT NULL), action, entity_type, entity_id, details_json, created_at`. **PAS** de `actor_type`/`actor_api_key_id` → à ajouter.
- `crates/kesh-db/src/entities/audit_log.rs:28-48` — `AuditLogEntry` / `NewAuditLogEntry` (tous champs obligatoires, `user_id: i64`). À étendre + constructeurs rétro-compat.
- `crates/kesh-db/src/repositories/audit_log.rs:26-69` — `insert_in_tx` (transactionnel, pas de delete — conservation 10 ans CO 957-964). Adapter l'INSERT.
- Call-sites existants : `routes/reconciliation.rs`, `repositories/fiscal_years.rs`, etc. — `grep "NewAuditLogEntry"` pour le refactor mécanique vers `::user(...)`.

**CRUD reference (pattern canonique = `bank_accounts`, Story v014-1)** :
- `entities/bank_account.rs` (struct + New), `entities/account.rs:43-66` (enum sqlx mapping), `repositories/bank_accounts.rs` (CRUD scopé company + optimistic lock + soft-delete `archived`/`revoked_at` + audit dans tx), `routes/bank_accounts.rs` (handlers, payloads, extraction `Extension<CurrentUser>`), `lib.rs` ≈ L255-263 (enregistrement bank_accounts).

**Frontend** :
- `frontend/src/lib/features/bank-accounts/` (api + types), `frontend/src/routes/(app)/bank-accounts/+page.svelte` (liste + modales), `frontend/src/routes/(app)/settings/+page.svelte` (structure sections + lien).
- `frontend/scripts/lint-i18n-ownership.js` — préfixe feature `api-key-` exclusif au dossier `api-keys/`.
- ⚠️ **HTTP LAN** : `navigator.clipboard`/`crypto.randomUUID` `undefined` en contexte non-sécurisé (memory `feedback_no_secure_context_apis_http_lan`). Copie clé = `navigator.clipboard.writeText().catch(fallback)` ; IDs DOM = `$props.id()`.

### Invariants à NE PAS casser (régressions)

- **Auth JWT cookie UI** : le chemin JWT de `require_auth` doit rester strictement inchangé en comportement (cookie prioritaire, fallback `Bearer <jwt>`, claims de confiance, pas de DB par requête). Le PAT est un **chemin additionnel** activé uniquement si le bearer commence par `kesh_pat_`.
- **Tous les call-sites `NewAuditLogEntry` existants** : doivent conserver `actor_type='user'` (constructeur `::user`). Aucune entrée d'audit existante ne doit changer de sémantique.
- **Multi-tenant scoping KF-002** : toute requête PAT est scopée au `company_id` de la clé. Jamais d'accès cross-company.
- **Migrations non-breaking** : `CREATE TABLE api_keys` + `ADD COLUMN` audit avec DEFAULT → aucun bump `kesh_version_min_required`. Confirmer au code-review (CLAUDE.md P3).

### Sécurité — points d'attention

- **Jamais de secret en clair persisté ni loggé** (`Debug` masqué, audit `details` sans secret, réponses list sans hash).
- **Rate-limiting PAT** : hors MVP (issue §7). Documenter comme limitation (risque : PAT durée illimitée si pas d'expiration → plus exposé à l'abus qu'un JWT 15 min). Mitigation v0.3 possible (rate-limit per-token).
- **Expiration** : `expires_at` optionnel ; si absent, clé permanente jusqu'à révocation. Le lookup auth filtre `expires_at > NOW(3)`.
- **Révocation immédiate** : soft-delete `revoked_at` → `find_active_by_key_hash` exclut → 401 dès la requête suivante.
- **2FA / scope fine-grained** : hors scope v0.2 (issue §2). Documenter.

### Project Structure Notes

- Séparation respectée : persistance (`kesh-db` : entities/repos/migrations) vs HTTP (`kesh-api` : auth/middleware/routes/errors) vs UI (`frontend`). Le PAT s'inscrit exactement dans les patterns établis (`bank_accounts` CRUD, `refresh_tokens` secret-lookup, `require_auth` middleware).
- Aucune variance de structure détectée. Le seul point d'attention structurel est le **ripple audit** (DC5/T-A4) qui touche des call-sites hors du périmètre direct de la feature.

### References

- [Source: GitHub #100] — issue d'origine (motivation, 8 sections techniques, Option A retenue).
- [Source: _bmad-output/planning-artifacts/epic-17.md#Story 17-2 (100-116) + D3/D4/D5] — périmètre, split pressenti.
- [Source: crates/kesh-api/src/middleware/auth.rs:66-140] — `require_auth`, point d'injection PAT.
- [Source: crates/kesh-api/src/middleware/rbac.rs:19-41] — guards de rôle.
- [Source: crates/kesh-api/src/auth/password.rs:1-210] — Argon2id (à NE PAS utiliser per-request, DC1) + `OsRng`.
- [Source: crates/kesh-db/src/repositories/refresh_tokens.rs + migrations/20260405000001_auth_refresh_tokens.sql] — précédent secret-lookup indexé.
- [Source: crates/kesh-db/migrations/20260413000001_audit_log.sql + entities/audit_log.rs + repositories/audit_log.rs] — audit à étendre (DC5).
- [Source: crates/kesh-db/src/repositories/bank_accounts.rs + routes/bank_accounts.rs + entities/account.rs:43-66] — pattern CRUD + enum sqlx de référence.
- [Source: crates/kesh-api/src/lib.rs] — câblage router (`comptable_routes`, route_layer).
- [Source: frontend/src/lib/features/bank-accounts/ + routes/(app)/settings/+page.svelte + scripts/lint-i18n-ownership.js] — pattern frontend + i18n.
- [Source: memory feedback_no_secure_context_apis_http_lan] — clipboard/crypto HTTP LAN.
- [Source: CLAUDE.md §Migration breaking policy P1/P3/P5, §Règle de splitting préventif, §Synchroniser TOUTES les docs] — contraintes process.

## Dev Agent Record

### Agent Model Used

(à compléter par dev-story — recommandation : Opus pour le middleware auth factorisé + le ripple audit cross-call-site ; story large → split 17-2a/b/c quasi-certain au validate)

### Debug Log References

### Completion Notes List

### File List
