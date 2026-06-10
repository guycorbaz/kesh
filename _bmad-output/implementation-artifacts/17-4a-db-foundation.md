# Story 17.4a: DB foundation + champ email backend (recovery — story-zéro)

Status: ready-for-dev

<!-- Extraite de la spec parente UMBRELLA 17-4 (`17-4-recovery-mot-de-passe.md`), validate CONVERGÉ 6 passes (trend > LOW 12→1→3→2→1→0). Le contenu ci-dessous est déjà adversarialement revu (dont catch-architectural Opus P3 + P4-1 stratégie new_for_tests). Re-validate optionnel. -->
<!-- STORY-ZÉRO : pose la migration `email`, la table `password_reset_tokens`, les repos, le refactor AppState→new_for_tests, le plumbing email DTO. BLOQUE 17-4b/c/d/e/f. DOIT MERGER EN PREMIER. -->

## Story

As a **développeur posant la fondation du recovery self-service Kesh**,
I want **la colonne `email` sur `users`, la table `password_reset_tokens`, les repos associés, et le plumbing email dans les DTO setup/users**,
so that **les sous-stories 17-4b (SMTP) / 17-4c (endpoints) / 17-4d (frontend) puissent s'appuyer sur un socle DB + entités stable, et que l'installation reste buildable/verte à chaque merge**.

## Contexte & cadrage

**Issue source :** [#122](https://github.com/guycorbaz/kesh/issues/122) (recovery production-grade, `v0.2-milestone`). Épopée 17-4, scope **cœur recovery seul** (décision Guy 2026-06-10). Spec complète + DC + sécurité : voir umbrella `17-4-recovery-mot-de-passe.md`.

**Position dans le split A–F (ordre SÉRIE) :** 17-4a (**ici, story-zéro**) → 17-4b (SMTP+config) → 17-4c (endpoints publics) → 17-4d (frontend) → 17-4e (tests) → 17-4f (doc). 17-4a et 17-4b **PAS parallélisables** (P3 F2 : 18 fichiers de test partagent `NewUser {` + `AppState {`).

**Scope 17-4a (cette story) :**
- Migration `ADD COLUMN email` sur `users` + migration `CREATE TABLE password_reset_tokens`.
- Entités `User` (champ `email`) + nouvelle `PasswordResetToken`.
- Repos : `users::find_by_email`, nouveau repo `password_reset_tokens`.
- Plumbing `email` dans DTO `setup/admin` + `users` CRUD + `UserResponse` + `UserUpdate`.
- `password_reset_tokens` → `TABLES_TO_TRUNCATE` (couplage export/import 17-3).
- **Refactor de fondation** `AppState { … }` → `AppState::new_for_tests` (anti-churn pour 17-4b/c).
- 2 lignes idempotence-audit.

**Hors scope 17-4a** (sous-stories suivantes) : module `mail/` + lib `lettre` + config SMTP (17-4b) ; endpoints `forgot/reset-password` + génération/validation token + rate-limit (17-4c) ; pages frontend + champs UI email (17-4d) ; tests E2E (17-4e) ; doc (17-4f). La table `password_reset_tokens` + son repo sont **créés ici** mais **exercés** par 17-4c.

**Migrations (DC12 — non-breaking, cf. CLAUDE.md §Migration breaking policy) :**
- `ADD COLUMN email VARCHAR(255) NULL` → non-breaking (nullable). **Pas** de bump `kesh_version_min_required`.
- `CREATE TABLE password_reset_tokens` → non-breaking (nouvelle table). **Pas** de bump.
- ⇒ 2 lignes à ajouter à `docs/migrations-idempotence-audit.md` (sinon finding MEDIUM P5 code-review).
- Timestamps des migrations **> `20260605000002`** (dernière existante).

## Acceptance Criteria

1. **Migration `email`** : `ALTER TABLE users ADD COLUMN IF NOT EXISTS email VARCHAR(255) NULL` (dialecte MariaDB, style `migrations/20260419000001_invoice_paid_at.sql`). Index non-unique `idx_users_email (email)` pour le lookup recovery. **Backward-compatible** (existants → `email = NULL`). Entité `User` (`entities/user.rs:108`) étendue de `pub email: Option<String>` (`FromRow` dérivé). ⚠️ **Propagation obligatoire (validate P1 F7 + P2 F1)** : `users.rs` n'utilise PAS de constante `COLUMNS` mais liste les colonnes en dur sur **5 sites `SELECT … FROM users` retournant `User`** — TOUS doivent inclure `email` (sinon `query_as::<_, User>` échoue au runtime, colonne absente du result-set pour `FromRow`) : 3 constantes `FIND_BY_ID_SQL` (`:14`), `FIND_BY_USERNAME_SQL` (`:17`), `LIST_SQL` (`:20`) **+ 2 SELECT inline** `list_by_company` (`:228`) et `find_by_id_in_company` (`:256`). De même l'INSERT de `create_in_tx`. **MAJ aussi l'impl `Debug for User` manuelle (`entities/user.rs:123`, custom pour masquer `password_hash`) → ajouter `.field("email", …)`** (P3 F4).

2. **Migration `password_reset_tokens`** : `CREATE TABLE password_reset_tokens (id BIGINT AUTO_INCREMENT PK, user_id BIGINT NOT NULL, token_hash CHAR(64) NOT NULL, expires_at DATETIME(3) NOT NULL, used_at DATETIME(3) NULL, created_at DATETIME(3) NOT NULL DEFAULT CURRENT_TIMESTAMP(3), CONSTRAINT fk_prt_user FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE, CONSTRAINT uq_prt_token_hash UNIQUE (token_hash), INDEX idx_prt_user (user_id), INDEX idx_prt_expires (expires_at)) ENGINE=InnoDB … utf8mb4_unicode_ci` (style `migrations/20260605000001_api_keys.sql`). FK `ON DELETE CASCADE` (DC11 — tokens éphémères, supprimer un user purge ses tokens). Entité `PasswordResetToken` (`entities/password_reset_token.rs`).

3. **Repo `users`** : ajouter `find_by_email(pool, email) -> Result<Vec<User>, DbError>` (retourne **Vec** car `email` non-unique, DC6) à `crates/kesh-db/src/repositories/users.rs`. `update_password` (déjà existant, `users.rs:270`) **réutilisé tel quel** par 17-4c (pas de modif).

4. **Repo `password_reset_tokens`** (`crates/kesh-db/src/repositories/password_reset_tokens.rs`, nouveau) : `create(pool, user_id, token_hash, expires_at)`, `find_valid_by_hash(pool, token_hash) -> Option<PasswordResetToken>` (filtre `used_at IS NULL AND expires_at > NOW(3)`), `mark_used(pool, id)`, `invalidate_all_for_user(pool, user_id)` (invalider les tokens pendants d'un user). Insertions/updates **paramétrés** (sqlx `query`/`bind`). Tests unit repo (create→find_valid→mark_used→find_valid retourne None ; expiré → None ; CASCADE delete user).

5. **Plumbing `email` backend (sans UI)** : DTO `setup/admin` (`routes/setup.rs`) et `users` CRUD (`routes/users.rs` create/update) acceptent un `email: Option<String>` optionnel, validé par le helper email existant si présent (non-vide ⇒ format valide, sinon `400 VALIDATION_ERROR`). `UserResponse` expose `email`. **Le wizard onboarding pose l'email de l'admin initial** (plumbing backend ici, champ UI en 17-4d). ⚠️ **Pré-requis (validate P1 F4/F8)** : (i) `is_valid_email_simple` (`routes/contacts.rs:168`) est **privé** → le promouvoir `pub(crate)` (ou extraire `routes/common.rs`) ; (ii) `NewUser` (`entities/user.rs:148`) + `UserUpdate` (`entities/user.rs:235`, n'a que `role`/`active`) reçoivent `email` + l'`UPDATE users SET …` de `update_role_and_active` (`users.rs:303`) gère `email` (ou nouvelle fn `update_email`).

6. **`TABLES_TO_TRUNCATE`** : ajouter `"password_reset_tokens"` (`crates/kesh-db/src/backup.rs:34`, **entre `"refresh_tokens"` et `"onboarding_state"`**, autre table éphémère enfant de `users`), pour que la table soit incluse dans export/import 17-3 et que `backup_inventory_matches_schema` (`backup.rs:567`) reste vert. **Sans cet ajout, le test échoue.**

7. **Idempotence-audit** : 2 lignes ajoutées à `docs/migrations-idempotence-audit.md` (les 2 nouvelles migrations + verdict `yes` si `IF NOT EXISTS` / `tracked-by-sqlx` + justification).

### Transverses

8. **Refactor de fondation `AppState`→`new_for_tests` (P3 F2 + P4-1)** : migrer les littéraux `AppState { … }` **migrables** vers `AppState::new_for_tests` (`lib.rs:60`) — fait **ici** (avec l'ajout `email` à `NewUser`) pour éviter toute collision avec 17-4b/c. ⚠️ **2 exceptions NON-migrables** (gardent un littéral `AppState { }`) car `new_for_tests` hardcode `users_exist=true` (`lib.rs:71`) et un seul `Config` builder : `tests/setup_admin_e2e.rs:81` (a besoin de `users_exist` **variable** pour les tests gate 423) et `src/middleware/auth.rs:267` (`test_state` avec `Config::from_fields_for_test`). `src/main.rs:224` reste littéral (prod). **Stratégie anti-churn (préparée pour 17-4b/c) : les futurs champs `AppState` (mailer en 17-4b, rate_limiter_recovery en 17-4c) seront ajoutés avec un DÉFAUT dans le CORPS de `new_for_tests` (signature inchangée) → les call-sites de test ne changeront pas.** 17-4a **n'ajoute aucun nouveau champ** à `AppState` (pure migration littéral→`new_for_tests`).

9. **Build/tests verts standalone** : 17-4a est **indépendamment mergeable** — le plumbing email DTO ne référence ni `Mailer`, ni routes recovery, ni frontend ; la migration + `TABLES_TO_TRUNCATE` + idempotence-audit sont auto-contenus. `cargo build --workspace --all-targets` + `cargo test --workspace` verts (cf. Test Locally First).

## Tasks / Subtasks

- [ ] **T-A1** Migration `ADD COLUMN email` sur `users` (+ index `idx_users_email`), style `IF NOT EXISTS`. Étendre entité `User` (`pub email`) + les **5 SELECT** retournant `User` : consts `FIND_BY_ID_SQL`/`FIND_BY_USERNAME_SQL`/`LIST_SQL` (`users.rs:14/17/20`) **+ inline `list_by_company:228` + `find_by_id_in_company:256`** + INSERT `create_in_tx`. **Ajouter `email` à `NewUser` (`entities/user.rs:148`) et corriger les 35 littéraux `NewUser { … }`** = `email: None`. **MAJ `Debug for User` manuel (`:123`)** avec `email`. (AC: 1)
- [ ] **T-A2** Migration `CREATE TABLE password_reset_tokens` (DC11 CASCADE). Entité `PasswordResetToken` (`entities/password_reset_token.rs`). (AC: 2)
- [ ] **T-A3** Repo `users::find_by_email` (Vec). (AC: 3)
- [ ] **T-A4** Nouveau repo `password_reset_tokens` (create/find_valid_by_hash/mark_used/invalidate_all_for_user), paramétrés. Tests unit repo. (AC: 4)
- [ ] **T-A5** Plumbing `email` : promouvoir `is_valid_email_simple` `pub(crate)` (F4) ; `email` dans DTO `setup/admin` + `users` create/update + `UserResponse` + `UserUpdate`/`update_role_and_active` UPDATE (F8) + validation. Tests. (AC: 5)
- [ ] **T-A6** `TABLES_TO_TRUNCATE += "password_reset_tokens"` (`backup.rs:34`) + vérifier `backup_inventory_matches_schema` vert. (AC: 6)
- [ ] **T-A7** 2 lignes `docs/migrations-idempotence-audit.md`. (AC: 7)
- [ ] **T-A8** Refactor `AppState { … }` migrables → `new_for_tests` ; 2 exceptions littérales documentées ; `main.rs` reste littéral. (AC: 8)
- [ ] **T-A9** Quality gate Test Locally First backend (fmt/build/clippy -D/test workspace) + serial si touche kesh-db (`-j1 --test-threads=1`). (AC: 9)

## Dev Notes

### Ground-truth (cartographie umbrella 17-4, 5 agents Explore 2026-06-10)

**Migrations & schéma :**
- Dir `crates/kesh-db/migrations/` (31 migrations, `YYYYMMDDHHMMSS_name.sql`). Dernière = `20260605000002_audit_log_actor.sql` → nouveaux timestamps **après**.
- Template ADD COLUMN : `20260419000001_invoice_paid_at.sql:19` (`ADD COLUMN IF NOT EXISTS … NULL`). Template CREATE TABLE : `20260605000001_api_keys.sql:18` (FK/INDEX/CHECK, `token_hash CHAR(64)`, `uq_…_key_hash UNIQUE`, `ENGINE=InnoDB … utf8mb4_unicode_ci`).
- `users` actuel : `20260404000001_initial_schema.sql:24` (`id, username, password_hash, role, active, version, created_at, updated_at` + `company_id` ajouté `20260419000002`) — **pas d'`email`**.
- `_kesh_version` : `version.rs:222` `check_downgrade_protection` (avant migrate). ADD COLUMN nullable + CREATE TABLE = non-breaking → pas de bump (DC12).
- Idempotence-audit : `docs/migrations-idempotence-audit.md`, format `| Fichier | Idempotence | Justification |` (verdict `yes`/`tracked-by-sqlx`/`no`).

**Entités & repos users :**
- `User` `entities/user.rs:108` : `{ id, username, password_hash, role, active, company_id, version, created_at, updated_at }`. **`Debug` manuel `:123`** (masque `password_hash`). `NewUser` `:148`. `UserUpdate` `:235` (`role`, `active` only). `Role` enum `:18`.
- `users.rs` repo : `find_by_username` (:156), `find_by_id` (:128), `update_password` (:270, incrémente `version`), `update_role_and_active` (:303), `create_in_tx` (:50), `list_by_company` (:219 inline SELECT :228), `find_by_id_in_company` (:250 inline SELECT :256). **5 SELECT `User` à patcher** (3 consts `:14/17/20` + 2 inline `:228/256`).
- `is_valid_email_simple` `routes/contacts.rs:168` — **privé**, à promouvoir `pub(crate)`.

**Backup / couplage 17-3 :**
- `TABLES_TO_TRUNCATE` `backup.rs:34` (22 tables, enfants→parents : `… refresh_tokens, onboarding_state, users, companies`). **Insérer `password_reset_tokens` entre `refresh_tokens` et `onboarding_state`.** Test `backup_inventory_matches_schema` `backup.rs:567` auto-fail si liste ≠ schéma. L'export `.keshbackup` sérialise dynamiquement `columnNames` → la nouvelle colonne `users.email` et la nouvelle table sont prises en compte automatiquement (aucun code export à toucher).

**AppState (refactor fondation) :**
- `AppState` `lib.rs:42` : `{ pool, config, rate_limiter, i18n, users_exist }`. `new_for_tests(pool, config, rate_limiter, i18n)` `:60` → hardcode `users_exist=Arc::new(AtomicBool::new(true))` (`:71`). 40 littéraux `AppState { }` (34 test + `main.rs:224` + `auth.rs:267`). **2 non-migrables** : `setup_admin_e2e.rs:81` (users_exist variable), `auth.rs:267` (`Config::from_fields_for_test`).

### Project Structure Notes

- Nouveaux fichiers : `migrations/{ts}_users_email.sql`, `migrations/{ts}_password_reset_tokens.sql`, `crates/kesh-db/src/entities/password_reset_token.rs`, `crates/kesh-db/src/repositories/password_reset_tokens.rs`.
- Modifs : `entities/user.rs` (+ `email`, Debug, NewUser, UserUpdate), `repositories/users.rs` (5 SELECT + find_by_email + UPDATE), `backup.rs` (TABLES_TO_TRUNCATE), `routes/{setup,users,contacts}.rs` (DTO email + is_valid_email_simple pub(crate)), `lib.rs` (new_for_tests sites), `docs/migrations-idempotence-audit.md`, + ~35 fichiers de test (NewUser/AppState).
- Aucune divergence structurelle — aligné conventions existantes.

### References

- [Source: umbrella `_bmad-output/implementation-artifacts/17-4-recovery-mot-de-passe.md` — spec convergée 6 passes, DC1-12, sécurité]
- [Source: GitHub Issue #122 — recovery production-grade]
- [Source: crates/kesh-db/migrations/20260605000001_api_keys.sql:18 — template CREATE TABLE token_hash CHAR(64)]
- [Source: crates/kesh-db/src/repositories/users.rs:14/17/20/228/256 — 5 SELECT User ; :270 update_password ; :303 update_role_and_active]
- [Source: crates/kesh-db/src/entities/user.rs:108/123/148/235 — User/Debug/NewUser/UserUpdate]
- [Source: crates/kesh-db/src/backup.rs:34/567 — TABLES_TO_TRUNCATE + test schéma]
- [Source: crates/kesh-api/src/routes/contacts.rs:168 — is_valid_email_simple privé]
- [Source: crates/kesh-api/src/lib.rs:42/60/71 — AppState + new_for_tests]
- [Source: CLAUDE.md §Migration breaking policy (P1/P3/P5), §Test Locally First]

## Dev Agent Record

### Agent Model Used

(à remplir au dev-story — Opus 4.8 recommandé : refactor cross-crate + propagation 35+40 sites non-trivial)

### Debug Log References

### Completion Notes List

- Story-zéro extraite de l'umbrella 17-4 convergée 6 passes (2026-06-10). Re-validate optionnel (contenu déjà adversarialement revu). Prochaine : `bmad-dev-story 17-4a` (Opus 4.8).

### File List
