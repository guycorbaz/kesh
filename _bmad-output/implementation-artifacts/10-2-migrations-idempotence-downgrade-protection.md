# Story 10.2: Migrations idempotence + downgrade protection + CI MariaDB 10.11

Status: ready-for-dev

<!-- Note: Validation is optional. Run validate-create-story for quality check before dev-story. -->

## Story

As a administrateur Kesh en production sur NAS Synology,
I want que toute mise à jour de Kesh applique automatiquement les migrations DB de façon idempotente et qu'un binaire plus ancien refuse de démarrer sur une DB déjà migrée par une version plus récente,
so that je puisse updater (et même downgrader par erreur) sans risque silencieux de corruption ou de perte de données — soit ça marche, soit le binaire s'arrête tout net avec un message explicite, jamais d'état intermédiaire compromis.

## Scope

Cette story livre **trois protections complémentaires** au flow de migrations DB :

1. **Audit d'idempotence des 26 migrations existantes** (`crates/kesh-db/migrations/*.sql`) — ajout d'un commentaire `-- idempotent: yes/no + détail` en tête de chaque fichier. Le tracking sqlx via la table `_sqlx_migrations` garantit déjà qu'une migration appliquée ne ré-exécute pas, mais l'audit est une défense en profondeur : si `_sqlx_migrations` est corrompue, perdue lors d'un restore partiel, ou si un opérateur force la ré-application manuellement (e.g. exécution directe via `mariadb < migration.sql`, comme la step CI `Apply migrations to kesh DB` ligne 122-127 de `ci.yml`), on documente quel comportement attendre. Aucune modification SQL des migrations historiques n'est requise — uniquement des commentaires SQL.

2. **Table `_kesh_version` + downgrade protection** :
   - Nouvelle migration `20260522000001_kesh_version.sql` qui crée `_kesh_version` (single-row config table).
   - Nouveau module `crates/kesh-db/src/version.rs` qui expose `check_downgrade_protection(pool, &binary_version)` et `record_boot_version(pool, &binary_version)`.
   - Intégration dans `crates/kesh-api/src/main.rs` :
     - **AVANT** `MIGRATOR.run()` → `check_downgrade_protection()` : si la table existe et `binary_version < kesh_version_min_required`, exit non-zero + log « FATAL: Database migrated by Kesh vX, current binary vY cannot downgrade safely ». Si la table n'existe pas (fresh install), continue silencieusement.
     - **APRÈS** `MIGRATOR.run()` → `record_boot_version()` : UPDATE de `kesh_version_last_applied` + `last_boot_at`. La table aura été créée par la nouvelle migration `20260522000001_kesh_version.sql` au plus tard ici.
   - Version binaire lue via `env!("CARGO_PKG_VERSION")` (résout Q7 epic-10.md — déjà pattern établi dans `crates/kesh-api/src/exports/metadata.rs:77` Story 9-2b).
   - Comparaison sémantique via le crate `semver` 1.x à ajouter à `crates/kesh-db/Cargo.toml` (NIH absolu sinon — parsing manuel semver = source d'edge-cases insolubles).

3. **Tests d'intégration migrations** (deux fichiers nouveaux dans `crates/kesh-db/tests/`) :
   - `migrations_fresh_install.rs` — DB vierge → `MIGRATOR.run()` → vérification structurelle (toutes les tables attendues existent) + seed minimal (1 company + 1 invoice + 1 journal_entry) qui round-trip OK.
   - `migrations_upgrade_path.rs` — applique migrations[..N-3] manuellement, insère sample data (rows dans tables historiques antérieures à `bank_imports`, e.g. `companies` + `users` + `accounts` + `invoices`), puis `MIGRATOR.run()` applique les 3+ migrations restantes (`bank_imports_relax_hash_unique`, `reconciliation_8_4`, `bank_account_journal_link`, `reconciliation_rules`), assertion que la sample data est préservée (COUNT(*) + checksum déterministe sur quelques colonnes).

**CI matrice MariaDB 10.11** (décision D3 epic-10.md déjà appliquée Story 10-1) :

- `ci.yml` utilise déjà `mariadb:10.11` depuis Story 10-1 (commit `b5a156b`). Cette story **documente la décision** dans `docs/ci.md` §"Décision MariaDB 10.11" et ajoute un AC explicite vérifiant qu'**aucune** matrice multi-version n'est introduite (la matrice serait contre-productive — pas de valeur de tester `mariadb:11.x` puisque la cible prod est 10.11 et qu'une feature 11 passant sur 11 mais cassant sur 10.11 ne serait pas détectée par 11).
- Aucune modification CI workflow effective n'est requise (l'alignement a été fait Story 10-1).

**Hors scope** (couverts par d'autres stories Epic 10) :

- Bumping de `kesh_version_min_required` à chaque release future qui introduit une migration breaking → **politique** à documenter dans `CLAUDE.md` §"Migration breaking policy" en fin de cette story, **mécanique** triviale (UPDATE inline dans la migration breaking elle-même quand elle apparaîtra).
- Résilience frontend si DB inaccessible — Story 10-3.
- Manuel install Synology mentionnant la procédure update + check `_kesh_version` — Story 10-4 (section "Update").
- Tokens cookies httpOnly — Story 10-5.
- FR78 « le système détecte une nouvelle version au démarrage et avertit de faire un backup » — **partiellement adressé** par cette story (la détection de version existe via `_kesh_version`, le log au boot mentionne « New migrations pending, backup recommended » avant `MIGRATOR.run()`). Le warning visuel UI utilisateur reste hors scope v0.1.0 (admin opère via SSH/Container Manager, le log est suffisant).

## Acceptance Criteria

### Audit idempotence migrations (AC #1-3)

1. **Given** chacun des 26 fichiers `crates/kesh-db/migrations/*.sql`, **When** review, **Then** chaque fichier porte un commentaire d'idempotence explicite **après le commentaire d'en-tête existant** et **avant la première instruction DDL/DML**, au format strict :
   - `-- idempotent: yes` (si re-exécution serait no-op : usage `CREATE TABLE IF NOT EXISTS`, `ALTER TABLE ... IF [NOT] EXISTS`, `CREATE INDEX IF NOT EXISTS`)
   - `-- idempotent: no — re-running fails with <error code/condition>` (si re-exécution échouerait : usage `CREATE TABLE <name>` sans `IF NOT EXISTS` → erreur 1050 ; `ALTER TABLE ADD COLUMN` sans `IF NOT EXISTS` → erreur 1060)
   - `-- idempotent: tracked-by-sqlx` (si l'idempotence est garantie uniquement par le tracking `_sqlx_migrations` — cas par défaut pour la majorité des 26 fichiers historiques, qui n'utilisent pas les guards `IF NOT EXISTS`).

2. **Given** le commentaire d'idempotence d'une migration historique, **When** review, **Then** **aucune migration historique** ne reçoit de modification SQL effective — uniquement l'ajout du commentaire. Pas de refactor de `CREATE TABLE foo (...)` en `CREATE TABLE IF NOT EXISTS foo (...)` (changement de comportement non-trivial qui pourrait masquer un bug futur où une migration croit faire un fresh create alors qu'elle hérite d'une table préexistante).

3. **Given** la migration `20260507000001_bank_imports_relax_hash_unique.sql` (déjà documentée idempotente lignes 15-18 par Story 8-3), **When** review, **Then** son commentaire d'idempotence existant est **conservé tel quel** (mention « L4 (Pass 1 review) — idempotency: support re-application or partial-state re-runs without crashing on MariaDB error 1091 (index not found) or 1061 (duplicate key) »). L'ajout du marqueur `-- idempotent: yes` au-dessus de ce commentaire historique est suffisant pour normaliser au format AC #1.

### Migration `_kesh_version` (AC #4-7)

4. **Given** le répertoire `crates/kesh-db/migrations/`, **When** review, **Then** une **nouvelle migration** `20260522000001_kesh_version.sql` existe avec une en-tête conforme au pattern projet (commentaire d'introduction expliquant le contexte Story 10-2 + decision references + `-- idempotent: tracked-by-sqlx`).

5. **Given** `20260522000001_kesh_version.sql`, **When** appliquée sur une DB vierge, **Then** crée la table `_kesh_version` avec le schéma :
   ```sql
   CREATE TABLE _kesh_version (
       id TINYINT UNSIGNED NOT NULL PRIMARY KEY DEFAULT 1,
       kesh_version_min_required VARCHAR(20) NOT NULL,
       kesh_version_last_applied VARCHAR(20) NOT NULL,
       applied_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
       last_boot_at DATETIME NULL,
       CONSTRAINT chk_kesh_version_single_row CHECK (id = 1)
   );
   ```
   Le `CHECK (id = 1)` enforce le singleton-row pattern (une seule row utile, AUTO_INCREMENT inutile). VARCHAR(20) couvre largement SemVer (`major.minor.patch[-pre][+build]` typiquement ≤ 20 chars pour Kesh — pas de pré-release `1.0.0-rc1.20260522.canary` envisagée).

6. **Given** `20260522000001_kesh_version.sql` appliquée, **When** review, **Then** la migration **insère la row initiale** : `INSERT INTO _kesh_version (id, kesh_version_min_required, kesh_version_last_applied) VALUES (1, '0.1.0', '0.1.0')`. La version `0.1.0` est figée dans le SQL — c'est la version Kesh courante au moment où cette migration est créée (cf. `crates/kesh-api/Cargo.toml:3`). L'`UPDATE` du `kesh_version_last_applied` par le boot logic (AC #11) écrasera ce default au prochain démarrage.

7. **Given** la migration `20260522000001_kesh_version.sql`, **When** review, **Then** elle est **idempotente en pratique sous tracking sqlx** : le `CREATE TABLE` n'utilise pas `IF NOT EXISTS` (cohérent avec les 19 autres migrations `CREATE TABLE` non-guarded historiques) et l'`INSERT` initial ne porte pas de `INSERT IGNORE` ni `ON DUPLICATE KEY UPDATE`. La ré-exécution manuelle hors sqlx échouerait avec erreur 1050 (table existe) — c'est intentionnel et documenté par `-- idempotent: tracked-by-sqlx` au format AC #1.

### Module `kesh-db/src/version.rs` + boot integration (AC #8-13)

8. **Given** `crates/kesh-db/Cargo.toml`, **When** review, **Then** une nouvelle dépendance `semver = "1"` est ajoutée à la section `[dependencies]`. La feature `serde` n'est **pas** activée (pas de serde sur les structures `Version` cross-process — uniquement parsing).

9. **Given** `crates/kesh-db/src/lib.rs`, **When** review, **Then** un nouveau module public `pub mod version;` est exposé (cohérent avec le pattern `pub mod repositories;` existant).

10. **Given** le nouveau fichier `crates/kesh-db/src/version.rs`, **When** review, **Then** il expose deux fonctions publiques :
    ```rust
    pub async fn check_downgrade_protection(
        pool: &MySqlPool,
        binary_version: &str,  // e.g. env!("CARGO_PKG_VERSION") = "0.1.0"
    ) -> Result<DowngradeCheckOutcome, VersionError>

    pub async fn record_boot_version(
        pool: &MySqlPool,
        binary_version: &str,
    ) -> Result<(), VersionError>
    ```
    avec un enum `DowngradeCheckOutcome { FreshInstall, Aligned, BinaryAhead { db_min: Version, binary: Version } }` (3 variants couvrant les 3 états possibles : table n'existe pas / binaire == ou > min_required / binaire > min_required quand min_required est juste informationnel d'une version antérieure). Le 4e cas `BinaryBehind { db_min, binary }` n'est **pas** un variant `Outcome` — il est **converti en `VersionError::DowngradeRefused { db_min, binary }`** car c'est le seul cas qui doit faire échouer le boot.

11. **Given** `check_downgrade_protection()`, **When** invoquée :
    - **Sur DB sans table `_kesh_version`** (fresh install) — la requête `SELECT kesh_version_min_required FROM _kesh_version WHERE id=1` retourne `sqlx::Error::Database(...)` avec MariaDB error code `1146` (ER_NO_SUCH_TABLE). **Then** la fonction retourne `Ok(DowngradeCheckOutcome::FreshInstall)` (table sera créée par `MIGRATOR.run()` ensuite).
    - **Sur DB où `_kesh_version` existe et `kesh_version_min_required = "0.1.0"` + binaire `0.1.0`** — **Then** retourne `Ok(DowngradeCheckOutcome::Aligned)`.
    - **Sur DB où `kesh_version_min_required = "0.1.0"` + binaire `"0.2.0"`** — **Then** retourne `Ok(DowngradeCheckOutcome::BinaryAhead { db_min: 0.1.0, binary: 0.2.0 })` (le binaire est plus récent que le min_required, c'est un upgrade légitime).
    - **Sur DB où `kesh_version_min_required = "0.2.0"` + binaire `"0.1.0"`** — **Then** retourne `Err(VersionError::DowngradeRefused { db_min: 0.2.0, binary: 0.1.0 })`.

12. **Given** `record_boot_version()`, **When** invoquée après `MIGRATOR.run()` réussi — la table `_kesh_version` existe nécessairement (créée par la migration `20260522000001_kesh_version.sql` au plus tard à l'instant). **Then** exécute `UPDATE _kesh_version SET kesh_version_last_applied = $1, last_boot_at = NOW() WHERE id = 1` puis retourne `Ok(())`. Si l'UPDATE échoue (e.g. pool fermé entre-temps), retourne `Err(VersionError::Sqlx(sqlx::Error))` mais ne fait **pas** exit le binaire — c'est un log warning au caller, pas une erreur fatale (le serveur reste utilisable même si le boot version metadata n'a pas pu être enregistré).

13. **Given** `crates/kesh-api/src/main.rs`, **When** review, **Then** l'ordre de boot est étendu autour de l'appel actuel `MIGRATOR.run()` (ligne 62 main actuelle) :
    ```rust
    // (existant) 3. Pool MariaDB
    // (existant) ligne 61 — tracing::info!(« Base de données : connectée »);

    // NOUVEAU 3b. Downgrade protection
    match kesh_db::version::check_downgrade_protection(&pool, env!("CARGO_PKG_VERSION")).await {
        Ok(outcome) => tracing::info!("Database version check: {:?}", outcome),
        Err(kesh_db::version::VersionError::DowngradeRefused { db_min, binary }) => {
            tracing::error!(
                "FATAL: Database was migrated by Kesh v{}, current binary v{} cannot downgrade safely. Restore a backup compatible with v{} or upgrade the binary.",
                db_min, binary, binary
            );
            std::process::exit(1);
        }
        Err(e) => {
            tracing::error!("Database version check failed (refusing to boot to avoid data corruption risk): {}", e);
            std::process::exit(1);
        }
    }

    // (existant) 4. Migrations
    if let Err(e) = kesh_db::MIGRATOR.run(&pool).await { ... }

    // NOUVEAU 4b. Record boot version
    if let Err(e) = kesh_db::version::record_boot_version(&pool, env!("CARGO_PKG_VERSION")).await {
        tracing::warn!("Failed to record boot version metadata (non-fatal): {}", e);
    }

    // (existant) 5. Bootstrap admin (...)
    ```

### Tests d'intégration migrations (AC #14-18)

14. **Given** un nouveau fichier `crates/kesh-db/tests/migrations_fresh_install.rs`, **When** `cargo test -p kesh-db --test migrations_fresh_install`, **Then** au moins **3 tests `#[sqlx::test(migrator = "kesh_db::MIGRATOR")]`** vérifient :
    - (a) Après migration, **toutes les tables attendues** existent. Liste minimale à valider : `companies`, `users`, `accounts`, `journal_entries`, `journal_entry_lines`, `invoices`, `invoice_lines`, `contacts`, `products`, `vat_rates`, `bank_accounts`, `bank_imports`, `bank_transactions`, `bank_profiles`, `reconciliation_rules`, `audit_log`, `refresh_tokens`, `onboarding_state`, `company_invoice_settings`, `invoice_number_sequences`, `_kesh_version`, `_sqlx_migrations` (~22 tables). Test via `SHOW TABLES` parsing.
    - (b) **Seed minimal** round-trip OK : INSERT company → INSERT user (avec `password_hash` Argon2 mock) → INSERT account (Asset class) → INSERT invoice avec validated_at NULL → INSERT journal_entry avec status='Draft' → SELECT chaque ligne avec assertion sur les colonnes clés. Aucune validation business comptable lourde (pas d'invariant partie-double sur 1 ligne) — c'est un test de schéma, pas un test métier.
    - (c) **Row initiale `_kesh_version`** : SELECT kesh_version_min_required, kesh_version_last_applied FROM _kesh_version WHERE id=1 retourne `("0.1.0", "0.1.0")`.

15. **Given** un nouveau fichier `crates/kesh-db/tests/migrations_upgrade_path.rs`, **When** `cargo test -p kesh-db --test migrations_upgrade_path`, **Then** au moins **2 tests** valident le upgrade path :
    - (a) **Cas générique** : un helper interne au test, `apply_migrations_up_to(pool, n)`, applique manuellement les `n` premières migrations (utilise `kesh_db::MIGRATOR.iter()` pour récupérer la liste ordonnée, applique chacune via `Migration::apply(&pool)` ou équivalent sqlx 0.8 API ; cf. Dev Notes §"Pattern test migrations" pour l'implémentation exacte). Le test : applique 23 migrations (= 26 historiques − 3), INSERT 1 company + 1 user + 2 accounts + 1 invoice + 1 journal_entry, puis appelle `MIGRATOR.run(&pool)` qui applique les 3 dernières historiques + `20260522000001_kesh_version.sql`. Assertion : COUNT(*) sur les 5 tables seedées **inchangé**, et SELECT sur les rows seedées retourne les mêmes valeurs (vérification quelques colonnes scalaires, pas de checksum complet — surcharge inutile pour test smoke). Plus assertion : `_kesh_version` existe avec `kesh_version_last_applied = '0.1.0'` après le test (mais `last_boot_at` reste NULL car le test n'invoque pas `record_boot_version`).
    - (b) **Cas downgrade détecté** : applique toutes les migrations (`MIGRATOR.run`), puis UPDATE `_kesh_version SET kesh_version_min_required = '0.99.0'` (simule un binaire futur qui aurait bumped le min), puis appelle `version::check_downgrade_protection(&pool, "0.1.0")` → assertion `Err(VersionError::DowngradeRefused { db_min, binary })` avec `db_min == "0.99.0"` et `binary == "0.1.0"`.

16. **Given** la suite `cargo test --workspace -j1 -- --test-threads=1` (mode CI serial), **When** exécutée après ajout des nouveaux tests, **Then** **tous les tests passent** y compris les 5 nouveaux tests (3 fresh_install + 2 upgrade_path). Aucune flakiness sur les `#[sqlx::test]` (chaque test isole sa DB via le mécanisme de DB éphémère sqlx-test).

17. **Given** un test exhibant un comportement non-déterministe (e.g. `last_boot_at` capturé in-test ≠ NOW() à l'instant de l'assertion), **When** review, **Then** le test **ne capture pas** `last_boot_at` avec assertion d'égalité exacte — soit l'assertion est `IS NOT NULL` (suffisant pour valider que l'UPDATE a eu lieu), soit l'assertion borne `last_boot_at >= test_start_time AND last_boot_at <= NOW()`. Idem pour `applied_at`.

18. **Given** un test du fichier `migrations_fresh_install.rs` ou `migrations_upgrade_path.rs`, **When** review, **Then** chacun déclare `#[sqlx::test(migrator = "kesh_db::MIGRATOR")]` avec migrator explicite (pas le default qui chercherait `./migrations/`). Le pattern est cohérent avec `crates/kesh-api/src/auth/bootstrap.rs:136` (référence canonique projet).

### CI matrice MariaDB 10.11 (AC #19-21)

19. **Given** `.github/workflows/ci.yml`, **When** review **après cette story**, **Then** un **seul service MariaDB** est déclaré (lignes 25-41 actuelles) avec `image: mariadb:10.11` (déjà en place depuis Story 10-1 D3 — pas de changement requis cette story). **Aucune matrice `strategy.matrix.mariadb-version`** n'est introduite.

20. **Given** `docs/ci.md`, **When** review, **Then** la section existante « Décision MariaDB 10.11 (Story 10-1 D3) » (cf. story 10-1 file list `docs/ci.md`) est **complétée** par une sous-section « Justification mono-version 10.11 » (ajout ≤ 8 lignes) expliquant explicitement :
    - Pas de matrice 10.11 + 11 car la cible prod est unique (NAS Synology Package Center DSM ≥ 7.2 ne propose que MariaDB 10.x stable).
    - Une matrice 11 ferait passer des tests sur un moteur que personne ne tournera en prod ; un bug 10.11-specific masqué par une feature 11 ne serait pas détecté par la branche 11 et serait par contre détecté par 10.11 — le test 10.11 est suffisant.
    - Compat upstream MariaDB ≥ 10.6 reste documentée (migration `reconciliation_rules.sql:27-28`) pour les opérateurs qui voudraient tourner sur 10.6/10.7/10.8/10.9/10.10 hors NAS Synology, mais pas testée par la CI projet.

21. **Given** `_bmad-output/planning-artifacts/epic-10.md` ligne 360 « CI matrice MariaDB 10.11 verte sur tous les tests Rust workspace », **When** review post-Story 10-2 merged, **Then** cette ligne est **cochée** dans le checklist § "Critères de done Epic 10" (note : modification éditoriale du planning artifact à inclure dans la PR).

### Politique « migration breaking » dans CLAUDE.md (AC #22-23)

22. **Given** `CLAUDE.md`, **When** review, **Then** une nouvelle section `## Migration breaking policy` est ajoutée après la section actuelle `## Issue Tracking Rule` (avant `## Règle de commit et push`), avec exactement les 4 paragraphes ci-dessous :
    - **(P1) Définition** : Une migration est **breaking** si elle introduit un état du schéma qu'un binaire Kesh antérieur ne peut **plus** consommer correctement (ex. DROP COLUMN d'une colonne lue par un SELECT du binaire antérieur, RENAME TABLE, changement de type INCOMPATIBLE Decimal → VARCHAR). La majorité des migrations (ADD COLUMN nullable, ADD INDEX, CREATE TABLE de nouvelle entité) sont **non-breaking** car les anciens binaires les ignorent.
    - **(P2) Procédure de bump** : Quand une migration breaking est introduite, la migration elle-même DOIT contenir, **en dernière instruction**, un `UPDATE _kesh_version SET kesh_version_min_required = '<version-de-la-PR-qui-introduit-la-migration>' WHERE id = 1;`. La version est figée dans le SQL (pas via paramètre runtime), comme la version d'origine `'0.1.0'` figée dans `20260522000001_kesh_version.sql`.
    - **(P3) Garde-fou code review** : Si une PR introduit une migration `DROP TABLE`, `DROP COLUMN`, `RENAME TABLE`, `RENAME COLUMN`, ou `ALTER COLUMN type` **sans** UPDATE de `kesh_version_min_required`, c'est un finding **CRITICAL** à remonter en passe `bmad-code-review`. Le rationale : ce sont les opérations dont l'omission du bump min_required exposera silencieusement les utilisateurs à un downgrade silencieux corrupteur. Inversement, ADD COLUMN nullable / ADD INDEX / CREATE TABLE n'imposent pas de bump.
    - **(P4) Exception documentée** : Si une migration utilise une de ces opérations mais reste **techniquement compatible** avec un binaire antérieur (rare — typiquement DROP d'une colonne jamais lue), l'auteur de la PR doit ajouter un commentaire SQL `-- breaking-skip-bump: <justification>` dans la migration, et un Pass code-review devra confirmer la justification. Sinon par défaut → bump obligatoire.

23. **Given** le nouveau commentaire d'idempotence ajouté par AC #1 sur chaque migration historique, **When** review, **Then** la **mention `-- idempotent: ...`** est l'**unique métadonnée commentée standardisée** par cette story sur les migrations historiques. Le marqueur `-- breaking-skip-bump:` (P4) sera introduit dans une PR future si/quand une migration concrète déclenche le cas exception. Les migrations historiques n'introduisent **pas** rétroactivement de marker `-- breaking-skip-bump:` (pas de bump pour la migration `_kesh_version.sql` elle-même non plus — c'est l'introduction du système, pas un changement breaking pré-existant).

### Validation end-to-end (AC #24-26)

24. **Given** le workflow `Test Locally First` (CLAUDE.md), **When** exécuté avant push de cette story, **Then** les 4 commandes Backend Rust passent (`cargo fmt --all -- --check`, `cargo build --workspace --all-targets`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace`) **avec MariaDB 10.11 démarré localement** (les nouveaux tests `migrations_*` requièrent un service DB).

25. **Given** la CI lancée sur la PR Story 10-2, **When** le job `Backend (Rust)` exécute `cargo test --workspace -j1 -- --test-threads=1` contre `mariadb:10.11`, **Then** tous les tests Rust passent : 250+ baselines pré-existantes + 5 nouveaux tests `migrations_fresh_install` (3) + `migrations_upgrade_path` (2). Aucune flakiness sur 3 runs CI consécutifs (vérifié par re-run manuel de la CI si nécessaire).

26. **And** **0 régression** sur les baselines hors `migrations_*` : kesh-api lib (173+), frontend Vitest (253), Playwright E2E (76, à condition que le job E2E soit exécuté — non par CI principale, par `Test Locally First`).

## Tasks / Subtasks

### T1: Audit idempotence des 26 migrations historiques (AC #1-3)

- [ ] T1.1 — Pour chacun des 26 fichiers `crates/kesh-db/migrations/*.sql`, ajouter un commentaire `-- idempotent: ...` immédiatement après le commentaire d'en-tête et avant la première instruction DDL/DML.
- [ ] T1.2 — Pour la migration `20260507000001_bank_imports_relax_hash_unique.sql` (déjà documentée idempotente lignes 15-18), ajouter `-- idempotent: yes` au-dessus du bloc existant **sans le modifier**.
- [ ] T1.3 — Pour les 19 migrations historiques utilisant `CREATE TABLE <name>` sans `IF NOT EXISTS`, le marqueur est `-- idempotent: tracked-by-sqlx`. Liste : `initial_schema.sql`, `auth_refresh_tokens.sql`, `onboarding_state.sql`, `bank_accounts.sql`, `accounts.sql`, `journal_entries.sql`, `audit_log.sql`, `contacts.sql`, `products.sql`, `invoices.sql`, `invoice_validation.sql`, `vat_rates.sql`, `bank_imports.sql`, `bank_profiles.sql`, `reconciliation_rules.sql`.
- [ ] T1.4 — Pour les 6 migrations `ALTER TABLE` historiques (`refresh_tokens_revoked_reason.sql`, `country_code.sql`, `invoice_paid_at.sql`, `users_company_id.sql`, `company_invoice_settings.sql`, `invoice_lines_line_total_check.sql`, `invoice_validated_journal_entry_check.sql`, `kf005_fulltext_indexes.sql`, `reconciliation_8_4.sql`, `bank_account_journal_link.sql`), classifier par inspection : si `ADD COLUMN`/`ADD INDEX`/`ADD CONSTRAINT` sans guard → `-- idempotent: tracked-by-sqlx`. Si guard `IF NOT EXISTS` présent → `-- idempotent: yes`.
- [ ] T1.5 — Validation par grep : `grep -l "^-- idempotent:" crates/kesh-db/migrations/*.sql | wc -l` doit retourner `26` (toutes auditées). Si < 26, identifier les manquantes et compléter.

### T2: Migration `_kesh_version.sql` + boot integration (AC #4-13)

- [ ] T2.1 — Créer `crates/kesh-db/migrations/20260522000001_kesh_version.sql` avec en-tête conforme (commentaire bloc référençant Story 10-2 + AC #4-7) + `-- idempotent: tracked-by-sqlx` + `CREATE TABLE _kesh_version` schéma AC #5 + `INSERT INTO _kesh_version (...) VALUES (1, '0.1.0', '0.1.0')` AC #6.
- [ ] T2.2 — Ajouter `semver = "1"` à `crates/kesh-db/Cargo.toml` `[dependencies]` (sans feature `serde`). Vérifier `cargo build -p kesh-db` PASS.
- [ ] T2.3 — Créer `crates/kesh-db/src/version.rs` :
  - `use semver::Version; use sqlx::MySqlPool;`
  - `#[derive(Debug, thiserror::Error)] pub enum VersionError { ... }` avec variants `DowngradeRefused { db_min: Version, binary: Version }`, `Sqlx(#[from] sqlx::Error)`, `InvalidSemver(#[from] semver::Error)`.
  - `#[derive(Debug)] pub enum DowngradeCheckOutcome { FreshInstall, Aligned, BinaryAhead { db_min: Version, binary: Version } }`.
  - `pub async fn check_downgrade_protection(pool: &MySqlPool, binary_version: &str) -> Result<DowngradeCheckOutcome, VersionError>` — parse `binary_version` via `Version::parse()` (semver crate), execute `sqlx::query_scalar!("SELECT kesh_version_min_required FROM _kesh_version WHERE id = 1").fetch_one(pool).await`, match sur l'erreur `sqlx::Error::Database(db_err) if db_err.code().as_deref() == Some("1146")` → `Ok(FreshInstall)`, parse le VARCHAR retourné en `Version`, compare via `binary.cmp(&db_min)` → 3 cas mapping vers `Aligned` / `BinaryAhead` / `Err(DowngradeRefused)`. ⚠️ `sqlx::query_scalar!` macro vérifie le schema à compile-time ; si la table `_kesh_version` n'existe pas encore en local au moment du build, soit utiliser `sqlx::query_scalar` (sans `!`) dynamique pour éviter la pré-compilation, soit s'assurer que `cargo sqlx prepare` est exécuté après création de la migration en local. **Recommandé Story 10-2** : utiliser `sqlx::query_scalar` (sans macro `!`) pour éviter cette dépendance ordonnée — perte de check compile-time tolérable pour 1 requête trivialement testée par les tests d'intégration.
  - `pub async fn record_boot_version(pool: &MySqlPool, binary_version: &str) -> Result<(), VersionError>` — `sqlx::query("UPDATE _kesh_version SET kesh_version_last_applied = ?, last_boot_at = NOW() WHERE id = 1").bind(binary_version).execute(pool).await` → `Ok(())` ou `Err(Sqlx)`.
  - Aucun unit test interne au module — couvert par les tests d'intégration T4. Le module entier devrait faire ~70-100 lignes Rust.
- [ ] T2.4 — Exposer le module dans `crates/kesh-db/src/lib.rs` : ajouter `pub mod version;` après les autres `pub mod` existants.
- [ ] T2.5 — Modifier `crates/kesh-api/src/main.rs` selon AC #13 : insérer le bloc `check_downgrade_protection` entre l'init du pool (fin ligne 61 actuelle) et `MIGRATOR.run()` (ligne 62 actuelle), et `record_boot_version` après le `tracing::info!("Migrations appliquées")` (ligne 67 actuelle). Adjuster les commentaires de la docstring `//! Ordre de démarrage` en haut du fichier (lignes 1-13) pour refléter les nouveaux steps 3b et 4b.

### T3: Tests d'intégration migrations (AC #14-18)

- [ ] T3.1 — Créer `crates/kesh-db/tests/migrations_fresh_install.rs` avec 3 tests :
  - `migrations_apply_all_tables_present` (AC #14a) — `SHOW TABLES` + assertion liste tables minimale (~22 tables).
  - `migrations_minimal_seed_roundtrips` (AC #14b) — INSERT/SELECT round-trip 5 lignes minimales.
  - `migrations_kesh_version_initial_row` (AC #14c) — SELECT `_kesh_version` row 1 → assertion `(0.1.0, 0.1.0)`.
- [ ] T3.2 — Créer `crates/kesh-db/tests/migrations_upgrade_path.rs` avec 2 tests :
  - `upgrade_path_preserves_data` (AC #15a) — utilise helper `apply_migrations_up_to` (à inclure inline dans le même fichier de test).
  - `downgrade_protection_rejects_old_binary` (AC #15b) — UPDATE de `kesh_version_min_required` à `'0.99.0'` puis appel `check_downgrade_protection` avec `"0.1.0"` → assertion `Err(DowngradeRefused)`.
- [ ] T3.3 — Implémenter le helper `apply_migrations_up_to(pool, n)` inline dans `migrations_upgrade_path.rs`. Pattern proposé Dev Notes §"Pattern test migrations". Si l'API sqlx 0.8 ne permet pas d'appliquer une sub-slice via `Migrator::iter()`, fallback : exécution manuelle des fichiers SQL via `std::fs::read_to_string` sur les 23 premiers fichiers tri-alpha, exécution séquentielle via `sqlx::raw_sql` + INSERT manuel d'une row par migration dans `_sqlx_migrations` (pattern visible dans `.github/workflows/ci.yml:122-127` mais sans le tracking, à adapter).
- [ ] T3.4 — Si l'helper T3.3 nécessite plus de 50 lignes Rust, extraire dans un sous-module `tests/common/migrations_helper.rs` (pattern Cargo `#[path = "common/migrations_helper.rs"] mod migrations_helper;` au début du fichier de test). Sinon laisser inline.
- [ ] T3.5 — Vérifier que les 5 nouveaux tests passent en local : `cargo test -p kesh-db --test migrations_fresh_install --test migrations_upgrade_path` avec MariaDB 10.11 démarré sur 127.0.0.1:3306 (kesh-mariadb container projet).

### T4: CI matrice MariaDB 10.11 doc + planning artifact sync (AC #19-21)

- [ ] T4.1 — Vérifier que `ci.yml:29` contient bien `image: mariadb:10.11` (déjà aligné Story 10-1). **No-op** Rust/YAML.
- [ ] T4.2 — Compléter `docs/ci.md` section « Décision MariaDB 10.11 (Story 10-1 D3) » par une sous-section « Justification mono-version 10.11 » ≤ 8 lignes selon AC #20.
- [ ] T4.3 — Cocher `[x]` la ligne 360 de `_bmad-output/planning-artifacts/epic-10.md` (« CI matrice MariaDB 10.11 verte sur tous les tests Rust workspace ») dans le checklist § "Critères de done Epic 10" — uniquement après que la CI Story 10-2 PR ait passé verte (vérification fonctionnelle effective, pas juste l'alignement statique de Story 10-1).

### T5: Politique migration breaking dans CLAUDE.md (AC #22-23)

- [ ] T5.1 — Ajouter la section `## Migration breaking policy` à `CLAUDE.md` selon AC #22 (4 paragraphes P1-P4), positionnée après `## Issue Tracking Rule` et avant `## Règle de commit et push`.
- [ ] T5.2 — Vérifier qu'aucune migration historique du repo ne reçoit rétroactivement de marker `-- breaking-skip-bump:` (cf. AC #23). Le seul marker méta-ajouté à 26 migrations historiques est `-- idempotent: ...` (T1).

### T6: Validation end-to-end (AC #24-26)

- [ ] T6.1 — Exécuter `Test Locally First` complet Backend (cf. CLAUDE.md §"Test Locally First / Backend (Rust)") avec MariaDB 10.11 local actif :
  - `cargo fmt --all -- --check` PASS
  - `cargo build --workspace --all-targets` PASS
  - `cargo clippy --workspace --all-targets -- -D warnings` PASS (en particulier vérifier que `version.rs` ne déclenche pas de warning clippy — pas de `unwrap()`, pas de `expect()` injustifié, error pattern via `?`)
  - `cargo test --workspace` PASS (mode parallèle local OK ; ne pas serializer en local sauf si flakiness observée)
- [ ] T6.2 — Exécuter Frontend `Test Locally First` (les 4 commandes habituelles) — **devrait être no-op fonctionnel** puisque Story 10-2 ne touche pas le frontend. Validation que rien n'a régressé par effet de bord (e.g. un fichier `.gitignore` ou `package.json` non-prévu).
- [ ] T6.3 — Push branche `chore/story-10-2-spec` (ou `story/10-2-...` après commit dev-story) et vérifier CI verte sur les 3 jobs (Backend Rust + Frontend + Docker sanity).
- [ ] T6.4 — Si CI verte → status sprint-status `10-2-...: review` puis `bmad-code-review 10-2` (LLM rotation cohérent avec `feedback_haiku_review_diff_combined` : commencer Sonnet 4.6, puis Haiku, etc. jusqu'à CONVERGED).

## Dev Notes

### Pattern test migrations

L'helper `apply_migrations_up_to(pool, n)` pour T3.3 — deux approches selon ce que l'API sqlx 0.8 permet :

**Approche A (préférée si sqlx 0.8 expose `.iter()` itérable)** :
```rust
use sqlx::migrate::Migrator;
use sqlx::MySqlPool;

async fn apply_migrations_up_to(pool: &MySqlPool, n: usize) -> sqlx::Result<()> {
    let migrator: &Migrator = &kesh_db::MIGRATOR;
    for (i, migration) in migrator.iter().enumerate() {
        if i >= n { break; }
        // sqlx 0.8 may expose Migration::apply directly, or via Migrator internals.
        // Verify via `cargo doc -p sqlx --open` chap. migrate::Migrator before relying on this.
        // Fallback if not exposed: switch to Approche B.
    }
    Ok(())
}
```

**Approche B (fallback robuste — utilisée par CI step `Apply migrations to kesh DB` ligne 122-127)** :
```rust
async fn apply_migrations_up_to(pool: &MySqlPool, n: usize) -> sqlx::Result<()> {
    let mut entries: Vec<_> = std::fs::read_dir("./migrations")?
        .collect::<Result<_, _>>()?;
    entries.sort_by_key(|e| e.path()); // sort alpha = sort by timestamp prefix
    for entry in entries.into_iter().take(n) {
        let sql = std::fs::read_to_string(entry.path())?;
        sqlx::raw_sql(&sql).execute(pool).await?;
        // Insert tracking row for this migration so MIGRATOR.run() later skips it.
        let version = filename_to_version(entry.file_name());
        sqlx::query(
            "INSERT INTO _sqlx_migrations (version, description, installed_on, success, checksum, execution_time) \
             VALUES (?, ?, NOW(), TRUE, ?, 0)"
        )
        .bind(version)
        .bind("backfill via apply_migrations_up_to")
        .bind(vec![0u8; 32])
        .execute(pool).await?;
    }
    Ok(())
}
```

⚠️ L'approche B requiert que le path `./migrations` soit relatif au manifest dir de `kesh-db` au moment du test — utiliser `env!("CARGO_MANIFEST_DIR")` pour résoudre absolu. Et la table `_sqlx_migrations` doit exister, ce qui est le cas dès qu'**une** migration sqlx a été exécutée — donc il faut soit pré-exécuter `MIGRATOR.run` sur les 0 migrations (no-op qui crée `_sqlx_migrations`), soit créer manuellement la table.

**Approche recommandée Story 10-2** : essayer Approche A d'abord (15 min d'investigation API sqlx 0.8). Si bloqué > 1h, basculer Approche B.

### Décision Q7 (epic-10.md) — résolue

`env!("CARGO_PKG_VERSION")` retenu pour lire la version binaire au runtime. Pattern déjà établi dans `crates/kesh-api/src/exports/metadata.rs:77` (Story 9-2b export ZIP). La version est figée au build (pas runtime via `KESH_VERSION` env var) — c'est intentionnel : un binaire est associé à une seule version, l'env override serait une porte ouverte à des inconsistences.

`crates/kesh-api/Cargo.toml:3` = `version = "0.1.0"` à la date de cette story. Cette version sera bumpée à `0.2.0` au kickoff Epic v0.2 (cf. memory `project_prod_deployment_gating`).

### Schéma `_kesh_version` — discussion

| Colonne | Type | Raison |
|---|---|---|
| `id` | `TINYINT UNSIGNED PRIMARY KEY DEFAULT 1` | Singleton row. CHECK constraint `id = 1` enforce. Tinyint suffit largement (0-255). |
| `kesh_version_min_required` | `VARCHAR(20) NOT NULL` | SemVer string, bumpé par migrations breaking (cf. CLAUDE.md §"Migration breaking policy"). |
| `kesh_version_last_applied` | `VARCHAR(20) NOT NULL` | Updated au boot par `record_boot_version()`. Informationnel — pour debug/audit, pas utilisé par downgrade check. |
| `applied_at` | `DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP` | Timestamp de l'INSERT initial par la migration. Référence historique première install. |
| `last_boot_at` | `DATETIME NULL` | Updated par `record_boot_version()`. NULL avant le premier boot effectif post-migration. |

Le pattern singleton avec `CHECK (id = 1)` est cohérent avec MariaDB 10.11 (le CHECK est enforce contrairement à versions très antérieures). Si pour une raison X le CHECK pose problème, l'alternative `UNIQUE KEY (id) + INSERT ... ON DUPLICATE KEY UPDATE` est équivalente fonctionnellement.

### Pattern `crates/kesh-db/src/version.rs` complet

Esquisse de ~90 lignes Rust (à raffiner pendant dev-story) :

```rust
//! Version tracking + downgrade protection for Kesh DB schema.
//! Story 10-2 — cf. _bmad-output/implementation-artifacts/10-2-...md

use semver::Version;
use sqlx::MySqlPool;

#[derive(Debug, thiserror::Error)]
pub enum VersionError {
    #[error("Database was migrated by Kesh v{db_min}, current binary v{binary} cannot downgrade safely. Restore a backup compatible with v{binary} or upgrade the binary.")]
    DowngradeRefused { db_min: Version, binary: Version },

    #[error("Database error during version check: {0}")]
    Sqlx(#[from] sqlx::Error),

    #[error("Invalid semver string: {0}")]
    InvalidSemver(#[from] semver::Error),
}

#[derive(Debug, PartialEq, Eq)]
pub enum DowngradeCheckOutcome {
    FreshInstall,
    Aligned,
    BinaryAhead { db_min: Version, binary: Version },
}

pub async fn check_downgrade_protection(
    pool: &MySqlPool,
    binary_version: &str,
) -> Result<DowngradeCheckOutcome, VersionError> {
    let binary = Version::parse(binary_version)?;

    let row: Result<String, sqlx::Error> = sqlx::query_scalar(
        "SELECT kesh_version_min_required FROM _kesh_version WHERE id = 1"
    )
    .fetch_one(pool)
    .await;

    match row {
        Err(sqlx::Error::Database(db_err)) if db_err.code().as_deref() == Some("1146") => {
            // ER_NO_SUCH_TABLE — fresh install, the migration `_kesh_version` hasn't run yet.
            Ok(DowngradeCheckOutcome::FreshInstall)
        }
        Err(e) => Err(VersionError::Sqlx(e)),
        Ok(db_min_str) => {
            let db_min = Version::parse(&db_min_str)?;
            match binary.cmp(&db_min) {
                std::cmp::Ordering::Less => {
                    Err(VersionError::DowngradeRefused { db_min, binary })
                }
                std::cmp::Ordering::Equal => Ok(DowngradeCheckOutcome::Aligned),
                std::cmp::Ordering::Greater => {
                    Ok(DowngradeCheckOutcome::BinaryAhead { db_min, binary })
                }
            }
        }
    }
}

pub async fn record_boot_version(
    pool: &MySqlPool,
    binary_version: &str,
) -> Result<(), VersionError> {
    Version::parse(binary_version)?; // validate semver but don't store the Version struct
    sqlx::query(
        "UPDATE _kesh_version SET kesh_version_last_applied = ?, last_boot_at = NOW() WHERE id = 1"
    )
    .bind(binary_version)
    .execute(pool)
    .await?;
    Ok(())
}
```

### MariaDB error code 1146 (ER_NO_SUCH_TABLE) — référence

L'erreur SQLSTATE 42S02, MariaDB code 1146 « Table 'database.table' doesn't exist » est ce que retourne le SELECT contre `_kesh_version` sur fresh install. Vérification API sqlx 0.8 : `sqlx::Error::Database` exposait `code()` en 0.7, conservé en 0.8 (cf. `cargo doc -p sqlx --open`). Si l'API évolue (peu probable patch-level), fallback : `e.to_string().contains("1146")` (textuel, plus fragile mais robuste contre refactor type-level).

### Dette latente identifiée pour audit cross-story

- **Story 10-3 « Résilience frontend si DB down »** dépend de `/health` qui pourrait étendre le body retourné par `record_boot_version` (e.g. `{ status: "ok", db: true, version: "0.1.0", min_required: "0.1.0" }`). Hors scope Story 10-2 mais à coordonner.
- **Story 10-4 « Manuel install Synology »** doit documenter dans la section "Update" : « après pull image + restart container, vérifier `docker logs kesh-api | grep _kesh_version` pour confirmer le bump si une migration breaking a été appliquée ». Hors scope Story 10-2 mais à mémoriser.
- **FR78 « avertir backup avant migration »** : Story 10-2 livre la **détection** (count des migrations pending via `MIGRATOR.iter()` vs `SELECT version FROM _sqlx_migrations`), mais le **log warning** « Backup recommended before applying N pending migrations » au boot est un AC borderline. Décision pre-dev : couvert par le log informatif ajouté **APRÈS** `MIGRATOR.run()` réussi (« Migrations appliquées : N nouvelles »), pas par un warning AVANT (qui imposerait à l'admin d'arrêter le boot et redémarrer après backup — pattern fragile en pratique). FR78 textuel sera ré-évalué Story 10-4 manuel install si insuffisant.

### Anti-patterns à éviter (extrait des codes review Story 10-1)

- **NE PAS faire** : modifier rétroactivement le SQL des migrations historiques (T1.2 explicite). Risque : rupture d'idempotence du hash sqlx `_sqlx_migrations.checksum`, lequel ferait échouer `MIGRATOR.run` sur toute install existante avec « migration X checksum mismatch ». Le seul changement autorisé est l'ajout d'un **commentaire** (`-- ...`) qui modifie le checksum mais sqlx tolère un checksum différent pour les migrations déjà tracked **si on lance via `MIGRATOR.run` sur une DB ayant le tracking** — à vérifier en local. Si checksum mismatch bloque : se contenter de marquer une seule migration historique (e.g. l'`initial_schema.sql`) avec le commentaire d'idempotence — l'audit reste informationnel et n'est pas mass-éditable rétroactivement.
- **NE PAS faire** : ajouter `serde` feature à `semver` (AC #8). La struct `Version` ne traverse pas de boundary serialisée — uniquement parsée depuis VARCHAR DB et comparée. La feature `serde` augmente le scope binaire sans gain.
- **NE PAS faire** : utiliser `.unwrap()` ou `.expect()` dans `version.rs` (clippy + CLAUDE.md). Toutes les erreurs descendent via `?` → `VersionError`. Le caller (`main.rs`) décide d'exit ou de log warn selon le type d'erreur.

### Test Locally First — MariaDB 10.11 requis

Les nouveaux tests `migrations_*` requièrent un service MariaDB joignable sur `DATABASE_URL`. Le container projet `kesh-mariadb` (actuellement `mariadb:11-jammy` selon session-state 2026-05-20) doit être restart sur `mariadb:10.11` pour parité prod **pour cette story** :

```sh
docker stop kesh-mariadb && docker rm kesh-mariadb
docker run -d --name kesh-mariadb \
  -e MARIADB_ROOT_PASSWORD=kesh_root \
  -e MARIADB_DATABASE=kesh \
  -e MARIADB_USER=kesh \
  -e MARIADB_PASSWORD=kesh_dev \
  -p 3306:3306 \
  mariadb:10.11
```

Note pré-dev : à confirmer si Guy souhaite restart son container local maintenant ou continuer à dev sur 11-jammy en local et compter sur la CI (qui tourne déjà 10.11 depuis Story 10-1). Les 5 nouveaux tests passeront sur les deux versions tant que les features SQL utilisées sont compat 10.6+ — ce qui est le cas (pas de window function avancée, pas de feature 11-only).

### Splitting check (CLAUDE.md §"Règle de splitting préventif")

Story 10-2 touche **2 crates** :
- `crates/kesh-db/` — Cargo.toml + lib.rs + version.rs (nouveau) + migrations/ (1 nouveau + 26 commentaires) + tests/ (2 nouveaux)
- `crates/kesh-api/` — main.rs (boot logic insert)

Plus 3 fichiers infra/doc : `docs/ci.md`, `CLAUDE.md`, `_bmad-output/planning-artifacts/epic-10.md`.

**Total : 2 crates + 3 docs. Largement sous le seuil 5 modules**. Pas de split nécessaire.

### Issue GitHub fermée

Aucune Issue spécifique fermée par cette story (KF Epic 7/9 toutes closes par Epic 9.5). Si un comportement d'auto-migration ou de downgrade est découvert défectueux pendant dev-story, ouvrir une Issue `bug` taggée Epic 10 (cf. CLAUDE.md §"Issue Tracking Rule").

## References

### Sources spec

- `_bmad-output/planning-artifacts/epic-10.md:145-181` — Story 10-2 ACs source (épic), périmètre, effort estimé
- `_bmad-output/planning-artifacts/epic-10.md:359-360` — Critères de done Epic 10 (lignes à cocher post-merge)
- `_bmad-output/planning-artifacts/epic-10.md:377` — Q7 « env!("CARGO_PKG_VERSION") » à confirmer (résolu cette story)
- `_bmad-output/planning-artifacts/epic-10.md:394` — référence « 26 migrations à auditer pour idempotence »
- `_bmad-output/planning-artifacts/epics.md:1174-1186` — Story 9.2 legacy mapping (numérotation pré-Epic 9.5 retro, redirigée vers 10-2 par PR #101)
- `_bmad-output/planning-artifacts/prd.md:496-497` — FR78 (détection version + warning backup) + FR79 (migrations auto)
- `_bmad-output/planning-artifacts/prd.md:228` — NFR-REL-5 (migrations préservent intégrité données exercices passés)
- `_bmad-output/planning-artifacts/architecture.md:214` — décision archi « Migrations: sqlx migrate, fichiers versionnés dans crates/kesh-db/migrations/ »
- `_bmad-output/planning-artifacts/architecture.md:276` — ARCH-12 « Migrations SQLx dans crates/kesh-db/migrations/ — fichiers versionnés, zéro perte de données »

### Code existant à toucher / référencer

- `crates/kesh-db/src/lib.rs:21` — `pub static MIGRATOR: sqlx::migrate::Migrator = sqlx::migrate!("./migrations");` — point d'entrée à conserver, ajout `pub mod version;` ici
- `crates/kesh-db/Cargo.toml:11` — ligne sqlx existante (référence features actives), `semver = "1"` à ajouter en `[dependencies]`
- `crates/kesh-api/src/main.rs:62` — `if let Err(e) = kesh_db::MIGRATOR.run(&pool).await` — code à étendre avant + après par les nouveaux blocs T2.5
- `crates/kesh-api/src/exports/metadata.rs:77` — `env!("CARGO_PKG_VERSION")` pattern de référence (résout Q7)
- `crates/kesh-api/Cargo.toml:3` — `version = "0.1.0"` source de la string passée à `check_downgrade_protection`
- `crates/kesh-api/src/auth/bootstrap.rs:136,170,202,254` — pattern `#[sqlx::test(migrator = "kesh_db::MIGRATOR")]` à reproduire dans T3.1 + T3.2
- `crates/kesh-db/migrations/20260507000001_bank_imports_relax_hash_unique.sql:15-18` — commentaire idempotence existant à conserver (T1.2)
- `crates/kesh-db/migrations/20260513000001_reconciliation_rules.sql:27-28` — référence MariaDB 10.6+ compat (cohérence cross-doc)
- `.github/workflows/ci.yml:25-41` — service MariaDB 10.11 (référence T4.1, aucune modif)
- `.github/workflows/ci.yml:122-127` — step `Apply migrations to kesh DB` (pattern fallback Approche B helper T3.3)
- `docs/ci.md` — section « Décision MariaDB 10.11 (Story 10-1 D3) » à étendre T4.2

### Documents projet

- `CLAUDE.md` §"Test Locally First" — checks Backend obligatoires (T6.1)
- `CLAUDE.md` §"Issue Tracking Rule" — pas d'Issue à fermer cette story
- `CLAUDE.md` §"Règle de splitting préventif" — Story 10-2 sous le seuil (cf. Dev Notes)
- `CLAUDE.md` §"Review Iteration Rule" — cycle spec validate + code review jusqu'à CONVERGED LOW
- Memory `project_session_state_2026_05_21_end` — état pré-Story 10-2 (Story 10-1 done, PRs #103 + #105 merged)
- Memory `feedback_haiku_review_diff_combined` — grep ground-truth obligatoire pendant code review (T6.4)
- Memory `feedback_avoid_parallel_prs` — éventuelle retro Epic 10 groupée dans PR de Story 10-5 (pas immédiat Story 10-2)
- Memory `feedback_subagent_unauthorized_commits` — surveiller commits Agent tool pendant subagent reviews

### Décisions pré-figées

- D3 epic-10.md — MariaDB 10.11 partout (appliqué Story 10-1, justifié docs/ci.md T4.2)
- D7 epic-10.md — Continuité données garantie dès v0.1.0 (livré cette story par downgrade protection + tests upgrade path)
- D10 epic-10.md — Bootstrap admin idempotent (déjà vérifié Story 10-1)
- D11 epic-10.md — Pas d'autre install Kesh, breaking change OK (donc pas de souci si min_required = 0.1.0 hardcoded — Guy = single user)

## 🚨 Questions ouvertes (à clarifier en spec validate Pass 1)

| # | Question | Hypothèse Story 10-2 ready-for-dev | À résoudre par |
|---|---|---|---|
| Q1 | sqlx 0.8 expose-t-il `Migration::apply` publiquement pour Approche A T3.3 ? | Inconnu — à vérifier par `cargo doc -p sqlx --open` au début dev-story | dev-story T3.3 (15 min investigation, fallback approche B définie) |
| Q2 | Le checksum sqlx tolère-t-il l'ajout de commentaires `-- idempotent: ...` rétroactif sans break du `MIGRATOR.run` sur DB existante ? | Inconnu — à vérifier par smoke test local : modif d'1 migration historique commentaire + run kesh-api boot, observer log. Si checksum mismatch → fallback Plan B (cf. Dev Notes anti-patterns) | dev-story T1 (smoke test 5 min avant T1.1-T1.5) |
| Q3 | Migration `20260522000001_kesh_version.sql` doit-elle être appliquée AVANT ou APRÈS le check downgrade au boot ? | **APRÈS** (cf. AC #11 — check retourne `FreshInstall` si table absente, puis `MIGRATOR.run` crée la table, puis `record_boot_version` UPDATE). Le risque : sur un upgrade de Kesh v0.0.x (jamais existé) → v0.1.0, la première fois où le check tournerait, la table n'existerait pas. C'est OK car v0.0.x n'existe pas en pratique. | (résolu spec) |
| Q4 | Faut-il un test de **pure idempotence run twice** (`MIGRATOR.run` + `MIGRATOR.run` second appel = no-op) ? | Pas en AC #14-15 mais bon à avoir comme test 6e (e.g. `migrations_idempotent_double_run`). À ajouter si dev-story le souhaite, sinon différer à code review pass. | spec validate Pass 1 ou dev-story optionnel |
| Q5 | Le commentaire P3 de CLAUDE.md §"Migration breaking policy" (AC #22) mentionne « finding CRITICAL » en code review — doit-on aussi formaliser une **check automatique** (script grep dans CI) ? | Pas immédiatement (overhead — un check `grep -lE "DROP TABLE|DROP COLUMN|RENAME ..." migrations/*.sql | xargs grep -L "UPDATE _kesh_version SET kesh_version_min_required"` serait fragile pour les exceptions P4). Laissé à la discipline code review humain + LLM. | (résolu spec — pas de check auto en v0.1) |

## Dev Agent Record

### Agent Model Used

(à compléter par `bmad-dev-story 10-2`)

### Debug Log References

(à compléter pendant dev-story)

### Completion Notes List

(à compléter pendant dev-story)

### File List

(à compléter pendant dev-story)

### Change Log

(à compléter — entrée par passe spec validate puis dev-story puis code review)
