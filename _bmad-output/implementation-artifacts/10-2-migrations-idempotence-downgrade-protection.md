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
- FR78 « le système détecte une nouvelle version au démarrage et avertit de faire un backup » — **partiellement adressé** par cette story : la détection de version existe via `_kesh_version` + le log informatif **APRÈS** `MIGRATOR.run()` réussi (« Migrations appliquées »). Pas de warning **AVANT** `MIGRATOR.run()` (qui imposerait à l'admin d'arrêter le boot et redémarrer après backup — pattern fragile en pratique, voir Dev Notes §"Dette latente identifiée"). Le warning visuel UI utilisateur reste hors scope v0.1.0 (admin opère via SSH/Container Manager). FR78 textuel sera ré-évalué Story 10-4 si insuffisant.

## Acceptance Criteria

### Audit idempotence migrations (AC #1-3)

> **Note Pass 1 spec validate** : la stratégie initiale (commentaires `-- idempotent: ...` ajoutés in-SQL) a été abandonnée. `MIGRATOR.run()` valide le checksum SHA-384 de chaque migration tracked dans `_sqlx_migrations` (cf. `sqlx-core-0.8.6/src/migrate/migrator.rs:175-176` — retour `MigrateError::VersionMismatch` garanti si checksum diffère). Modifier les fichiers `.sql` historiques (même un commentaire) casserait toute DB déjà migrée. Plan retenu : audit dans un fichier markdown séparé `docs/migrations-idempotence-audit.md`, zéro modification des fichiers `.sql` historiques.

1. **Given** un nouveau fichier `docs/migrations-idempotence-audit.md`, **When** review, **Then** il contient un tableau markdown auditant chacun des 26 fichiers `crates/kesh-db/migrations/*.sql` au format :
   ```
   | Fichier | Idempotence | Justification |
   |---|---|---|
   | 20260404000001_initial_schema.sql | tracked-by-sqlx | `CREATE TABLE` sans `IF NOT EXISTS` — re-exécution manuelle hors sqlx échouerait avec erreur 1050. Le tracking `_sqlx_migrations` empêche la ré-application. |
   | ... | ... | ... |
   ```
   Verdicts admis (un seul par ligne) :
   - `yes` — re-exécution serait no-op (usage `CREATE TABLE IF NOT EXISTS`, `ALTER TABLE ... IF [NOT] EXISTS`, `CREATE INDEX IF NOT EXISTS`, `DROP INDEX IF EXISTS`).
   - `no` — re-exécution échouerait (avec code d'erreur MariaDB ciblé dans la justification : 1050 table exists, 1060 duplicate column, 1061 duplicate key, 1091 index not found).
   - `tracked-by-sqlx` — l'idempotence est garantie uniquement par le tracking `_sqlx_migrations` — cas par défaut pour la majorité des fichiers historiques sans guards `IF NOT EXISTS`.

2. **Given** les 26 fichiers `crates/kesh-db/migrations/*.sql` historiques, **When** review post-Story-10-2, **Then** **aucun fichier `.sql` historique ne reçoit de modification** (pas de commentaire ajouté, pas de refactor, pas de bytes changés). Cela évite tout `MigrateError::VersionMismatch` sur les DB existantes. La seule migration `.sql` créée par cette story est `20260522000001_kesh_version.sql` (AC #4-7). Pour la nouvelle migration, le verdict d'idempotence figure dans son commentaire d'en-tête ET dans le fichier audit.

3. **Given** la migration `20260507000001_bank_imports_relax_hash_unique.sql` (déjà documentée idempotente lignes 15-18 par Story 8-3), **When** review, **Then** son commentaire d'idempotence existant **n'est pas modifié** (cohérent AC #2). Son entrée dans `docs/migrations-idempotence-audit.md` est verdict `yes` avec justification renvoyant aux lignes 15-18 du fichier.

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

8. **Given** `crates/kesh-db/Cargo.toml`, **When** review, **Then** une nouvelle dépendance `semver = "1"` est ajoutée à la section `[dependencies]`. La feature `serde` n'est **pas** activée (pas de serde sur les structures `Version` cross-process — uniquement parsing). Note : `semver 1.0.x` est déjà résolu dans `Cargo.lock` comme dépendance transitive (cargo-metadata) — l'ajout en dépendance directe ne provoque pas de re-résolution.

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
    - **Sur DB sans table `_kesh_version`** (fresh install) — la requête `SELECT kesh_version_min_required FROM _kesh_version WHERE id=1` retourne `sqlx::Error::Database(db_err)` qui est downcastable en `sqlx::mysql::MySqlDatabaseError` dont `.number() == 1146` (ER_NO_SUCH_TABLE). **⚠️** La méthode `code()` retourne le SQLSTATE `"42S02"`, **pas** le numéro 1146 — utiliser obligatoirement le pattern `try_downcast_ref::<sqlx::mysql::MySqlDatabaseError>().map_or(false, |e| e.number() == 1146)` (référence canonique : `crates/kesh-db/src/errors.rs:150` + `crates/kesh-db/src/retry.rs:73`). **Then** la fonction retourne `Ok(DowngradeCheckOutcome::FreshInstall)` (table sera créée par `MIGRATOR.run()` ensuite).
    - **Sur DB où `_kesh_version` existe et `kesh_version_min_required = "0.1.0"` + binaire `0.1.0`** — **Then** retourne `Ok(DowngradeCheckOutcome::Aligned)`.
    - **Sur DB où `kesh_version_min_required = "0.1.0"` + binaire `"0.2.0"`** — **Then** retourne `Ok(DowngradeCheckOutcome::BinaryAhead { db_min: 0.1.0, binary: 0.2.0 })` (le binaire est plus récent que le min_required, c'est un upgrade légitime).
    - **Sur DB où `kesh_version_min_required = "0.2.0"` + binaire `"0.1.0"`** — **Then** retourne `Err(VersionError::DowngradeRefused { db_min: 0.2.0, binary: 0.1.0 })`.

12. **Given** `record_boot_version()`, **When** invoquée après `MIGRATOR.run()` réussi — la table `_kesh_version` existe nécessairement (créée par la migration `20260522000001_kesh_version.sql` au plus tard à l'instant). **Then** exécute `UPDATE _kesh_version SET kesh_version_last_applied = ?, last_boot_at = NOW() WHERE id = 1` puis vérifie défensivement `rows_affected() == 1` (si != 1 → log warning « row missing? » mais retourne quand même `Ok(())` — défense en profondeur, ne devrait jamais arriver vu la migration AC #6). Si l'UPDATE échoue (e.g. pool fermé entre-temps), retourne `Err(VersionError::Sqlx(sqlx::Error))` mais ne fait **pas** exit le binaire — c'est un log warning au caller, pas une erreur fatale (le serveur reste utilisable même si le boot version metadata n'a pas pu être enregistré).

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
    - (a) Après migration, **toutes les tables attendues** existent. Liste minimale à valider : `companies`, `users`, `fiscal_years`, `accounts`, `journal_entries`, `journal_entry_lines`, `invoices`, `invoice_lines`, `contacts`, `products`, `vat_rates`, `bank_accounts`, `bank_imports`, `bank_transactions`, `bank_profiles`, `reconciliation_rules`, `audit_log`, `refresh_tokens`, `onboarding_state`, `company_invoice_settings`, `invoice_number_sequences`, `_kesh_version`, `_sqlx_migrations` (~23 tables). Test via `SHOW TABLES` parsing.
    - (b) **Seed minimal** round-trip OK : INSERT company → INSERT user (avec `password_hash` Argon2 mock) → INSERT account (Asset class) → INSERT invoice avec validated_at NULL → INSERT journal_entry avec status='Draft' → SELECT chaque ligne avec assertion sur les colonnes clés. Aucune validation business comptable lourde (pas d'invariant partie-double sur 1 ligne) — c'est un test de schéma, pas un test métier.
    - (c) **Row initiale `_kesh_version`** : SELECT kesh_version_min_required, kesh_version_last_applied FROM _kesh_version WHERE id=1 retourne `("0.1.0", "0.1.0")`.

15. **Given** un nouveau fichier `crates/kesh-db/tests/migrations_upgrade_path.rs`, **When** `cargo test -p kesh-db --test migrations_upgrade_path`, **Then** au moins **2 tests** valident le upgrade path :
    - (a) **Cas générique** : un helper interne au test, `apply_migrations_up_to(pool, n)`, applique les `n` premières migrations via le **sub-Migrator pattern** documenté dans Dev Notes §"Pattern test migrations" (slice `&kesh_db::MIGRATOR.migrations[..n]` + construction d'un `Migrator` ad-hoc + `.run(pool)`). Le test utilise `#[sqlx::test]` **sans** l'attribut `migrator = ...` (sinon MIGRATOR complet est appliqué automatiquement avant le corps du test, ce qui invaliderait le scénario) — l'helper applique 23 migrations (= 26 historiques − 3), INSERT 1 company + 1 user + 2 accounts + 1 invoice + 1 journal_entry, puis appelle `kesh_db::MIGRATOR.run(&pool)` qui applique les 3 dernières migrations historiques + `20260522000001_kesh_version.sql` (4 migrations totales par le full run, post-story le repo contient 27 migrations). Assertion : COUNT(*) sur les 5 tables seedées **inchangé**, et SELECT sur les rows seedées retourne les mêmes valeurs (vérification quelques colonnes scalaires, pas de checksum complet — surcharge inutile pour test smoke). Plus assertion : `_kesh_version` existe avec `kesh_version_last_applied = '0.1.0'` après le test (mais `last_boot_at` reste NULL car le test n'invoque pas `record_boot_version`).
    - (b) **Cas downgrade détecté** : applique toutes les migrations (`MIGRATOR.run`), puis UPDATE `_kesh_version SET kesh_version_min_required = '0.99.0'` (simule un binaire futur qui aurait bumped le min), puis appelle `version::check_downgrade_protection(&pool, "0.1.0")` → assertion `Err(VersionError::DowngradeRefused { db_min, binary })` avec `db_min == "0.99.0"` et `binary == "0.1.0"`.

16. **Given** la suite `cargo test --workspace -j1 -- --test-threads=1` (mode CI serial), **When** exécutée après ajout des nouveaux tests, **Then** **tous les tests passent** y compris les 5 nouveaux tests (3 fresh_install + 2 upgrade_path). Aucune flakiness sur les `#[sqlx::test]` (chaque test isole sa DB via le mécanisme de DB éphémère sqlx-test).

17. **Given** un test exhibant un comportement non-déterministe (e.g. `last_boot_at` capturé in-test ≠ NOW() à l'instant de l'assertion), **When** review, **Then** le test **ne capture pas** `last_boot_at` avec assertion d'égalité exacte — soit l'assertion est `IS NOT NULL` (suffisant pour valider que l'UPDATE a eu lieu), soit l'assertion borne `last_boot_at >= test_start_time AND last_boot_at <= NOW()`. Idem pour `applied_at`.

18. **Given** un test du fichier `migrations_fresh_install.rs`, **When** review, **Then** il déclare `#[sqlx::test(migrator = "kesh_db::MIGRATOR")]` avec migrator explicite — `#[sqlx::test]` provisionne une DB éphémère puis applique le migrator complet avant le corps du test (cohérent `crates/kesh-api/src/auth/bootstrap.rs:136`, référence canonique projet).
    **Given** un test du fichier `migrations_upgrade_path.rs`, **When** review, **Then** il déclare `#[sqlx::test]` **sans** attribut `migrator` — la DB éphémère reste vide avant le corps du test, le test contrôle l'application des migrations via `apply_migrations_up_to(pool, 23)` puis `kesh_db::MIGRATOR.run(&pool)`. Cette différence est intentionnelle et critique pour la validité du scénario upgrade path (AC #15a).

### CI matrice MariaDB 10.11 (AC #19-21)

19. **Given** `.github/workflows/ci.yml`, **When** review **après cette story**, **Then** un **seul service MariaDB** est déclaré (lignes 25-41 actuelles) avec `image: mariadb:10.11` (déjà en place depuis Story 10-1 D3 — pas de changement requis cette story). **Aucune matrice `strategy.matrix.mariadb-version`** n'est introduite.

20. **Given** `docs/ci.md`, **When** review, **Then** la section existante « Décision MariaDB 10.11 (Story 10-1 D3) » (cf. story 10-1 file list `docs/ci.md`) est **complétée** par une sous-section « Justification mono-version 10.11 » (ajout ≤ 8 lignes) expliquant explicitement :
    - Pas de matrice 10.11 + 11 car la cible prod est unique (NAS Synology Package Center DSM ≥ 7.2 ne propose que MariaDB 10.x stable).
    - Une matrice 11 ferait passer des tests sur un moteur que personne ne tournera en prod ; un bug 10.11-specific masqué par une feature 11 ne serait pas détecté par la branche 11 et serait par contre détecté par 10.11 — le test 10.11 est suffisant.
    - Compat upstream MariaDB ≥ 10.6 reste documentée (migration `reconciliation_rules.sql:27-28`) pour les opérateurs qui voudraient tourner sur 10.6/10.7/10.8/10.9/10.10 hors NAS Synology, mais pas testée par la CI projet.

21. **Given** `_bmad-output/planning-artifacts/epic-10.md` ligne 360 « CI matrice MariaDB 10.11 verte sur tous les tests Rust workspace », **When** review post-Story 10-2 merged, **Then** cette ligne est **cochée** dans le checklist § "Critères de done Epic 10" (note : modification éditoriale du planning artifact à inclure dans la PR).

### Politique « migration breaking » dans CLAUDE.md (AC #22-23)

22. **Given** `CLAUDE.md`, **When** review, **Then** une nouvelle section `## Migration breaking policy` est ajoutée après la section actuelle `## Issue Tracking Rule` (avant `## Règle de commit et push`), avec exactement les 4 paragraphes ci-dessous :
    - **(P1) Définition** : Une migration est **breaking** si elle introduit un état du schéma qu'un binaire Kesh antérieur ne peut **plus** consommer correctement (ex. `DROP COLUMN` d'une colonne lue par un SELECT du binaire antérieur, `RENAME TABLE`, `MODIFY COLUMN` ou `CHANGE COLUMN` introduisant un type incompatible ex. DECIMAL → VARCHAR). La majorité des migrations (`ADD COLUMN` nullable, `ADD INDEX`, `CREATE TABLE` de nouvelle entité) sont **non-breaking** car les anciens binaires les ignorent.
    - **(P2) Procédure de bump** : Quand une migration breaking est introduite, la migration elle-même DOIT contenir, **en dernière instruction**, un `UPDATE _kesh_version SET kesh_version_min_required = '<version-de-la-PR-qui-introduit-la-migration>' WHERE id = 1;`. La version est figée dans le SQL (pas via paramètre runtime), comme la version d'origine `'0.1.0'` figée dans `20260522000001_kesh_version.sql`.
    - **(P3) Garde-fou code review** : Si une PR introduit une migration `DROP TABLE`, `DROP COLUMN`, `RENAME TABLE`, `RENAME COLUMN`, `MODIFY COLUMN <type>`, ou `CHANGE COLUMN <name> <type>` **sans** UPDATE de `kesh_version_min_required`, c'est un finding **CRITICAL** à remonter en passe `bmad-code-review`. Note dialecte : MariaDB utilise `MODIFY COLUMN <type>` ou `CHANGE COLUMN <old> <new> <type>` pour les changements de type (la syntaxe PostgreSQL `ALTER COLUMN <name> TYPE <type>` n'est **pas** supportée en MariaDB — référence locale `crates/kesh-db/migrations/20260419000002_users_company_id.sql:23` utilise bien `MODIFY COLUMN`). Le rationale : ces opérations sont celles dont l'omission du bump min_required exposerait silencieusement les utilisateurs à un downgrade corrupteur. Inversement, `ADD COLUMN nullable` / `ADD INDEX` / `CREATE TABLE` n'imposent pas de bump.
    - **(P4) Exception documentée** : Si une migration utilise une de ces opérations mais reste **techniquement compatible** avec un binaire antérieur (rare — typiquement `DROP` d'une colonne jamais lue), l'auteur de la PR doit ajouter un commentaire SQL `-- breaking-skip-bump: <justification>` dans la migration, et un Pass code-review devra confirmer la justification. Sinon par défaut → bump obligatoire.

23. **Given** la stratégie audit Pass 1 (AC #1-3 — audit dans `docs/migrations-idempotence-audit.md`, zéro modif des `.sql` historiques), **When** review, **Then** le marqueur `-- breaking-skip-bump:` (P4) **n'est introduit dans aucune migration historique** par cette story — il sera introduit dans une PR future si/quand une migration concrète déclenche le cas exception P4. La migration `20260522000001_kesh_version.sql` créée par cette story ne reçoit pas non plus de `-- breaking-skip-bump:` (c'est l'introduction du système de versioning, pas un changement breaking pré-existant — `kesh_version_min_required = '0.1.0'` figé initialement, à bumper par les futures migrations breaking via P2).

### Validation end-to-end (AC #24-26)

24. **Given** le workflow `Test Locally First` (CLAUDE.md), **When** exécuté avant push de cette story, **Then** les 4 commandes Backend Rust passent (`cargo fmt --all -- --check`, `cargo build --workspace --all-targets`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace`) **avec MariaDB 10.11 démarré localement** (les nouveaux tests `migrations_*` requièrent un service DB).

25. **Given** la CI lancée sur la PR Story 10-2, **When** le job `Backend (Rust)` exécute `cargo test --workspace -j1 -- --test-threads=1` contre `mariadb:10.11`, **Then** tous les tests Rust passent : 250+ baselines pré-existantes + 5 nouveaux tests `migrations_fresh_install` (3) + `migrations_upgrade_path` (2). Aucune flakiness sur 3 runs CI consécutifs (vérifié par re-run manuel de la CI si nécessaire).

26. **And** **0 régression** sur les baselines hors `migrations_*` : kesh-api lib (173+), frontend Vitest (253), Playwright E2E (76, à condition que le job E2E soit exécuté — non par CI principale, par `Test Locally First`).

## Tasks / Subtasks

### T1: Audit idempotence des 26 migrations historiques → fichier `docs/migrations-idempotence-audit.md` (AC #1-3)

- [ ] T1.1 — Créer `docs/migrations-idempotence-audit.md` avec en-tête (paragraphe d'intro renvoyant à Story 10-2 et au paragraphe « Note Pass 1 spec validate » d'AC #1-3) + tableau markdown 26 rows.
- [ ] T1.2 — Pour les **15 migrations** historiques utilisant `CREATE TABLE <name>` sans `IF NOT EXISTS` (`initial_schema.sql`, `auth_refresh_tokens.sql`, `onboarding_state.sql`, `bank_accounts.sql`, `accounts.sql`, `journal_entries.sql`, `audit_log.sql`, `contacts.sql`, `products.sql`, `invoices.sql`, `invoice_validation.sql`, `vat_rates.sql`, `bank_imports.sql`, `bank_profiles.sql`, `reconciliation_rules.sql`) → verdict `tracked-by-sqlx`, justification : « `CREATE TABLE` sans `IF NOT EXISTS` ; re-exécution manuelle hors sqlx échouerait avec erreur MariaDB 1050. ».
- [ ] T1.3 — Pour les **10 migrations** `ALTER TABLE` historiques (`refresh_tokens_revoked_reason.sql`, `invoice_lines_line_total_check.sql`, `invoice_validated_journal_entry_check.sql`, `country_code.sql`, `invoice_paid_at.sql`, `users_company_id.sql`, `kf005_fulltext_indexes.sql`, `bank_imports_relax_hash_unique.sql`, `reconciliation_8_4.sql`, `bank_account_journal_link.sql`), classifier par inspection :
  - Si `ADD COLUMN`/`ADD INDEX`/`ADD CONSTRAINT` sans guard → verdict `tracked-by-sqlx`, justification : « `ALTER TABLE ADD ...` sans `IF NOT EXISTS` ; re-exécution échouerait erreur 1060 (colonne) ou 1061 (index/constraint). ».
  - Si guard `IF [NOT] EXISTS` présent → verdict `yes`, justification courte (exemple `bank_imports_relax_hash_unique.sql` lignes 15-18 — verdict `yes`).
- [ ] T1.4 — Pour la **migration `company_invoice_settings.sql`** (CREATE INDEX only, pas d'ALTER TABLE) → verdict `tracked-by-sqlx`, justification : « `CREATE INDEX` sans `IF NOT EXISTS` ; re-exécution échouerait erreur 1061. ».
- [ ] T1.5 — Validation par count : `grep -c "^|" docs/migrations-idempotence-audit.md` ≥ 27 (header row + séparateur + 26 data rows). Chacun des 26 fichiers `.sql` historiques apparaît exactement une fois dans le tableau (`for f in crates/kesh-db/migrations/*.sql; do grep -q "$(basename $f)" docs/migrations-idempotence-audit.md || echo "manquant: $f"; done` retourne vide).
- [ ] T1.6 — Référencer `docs/migrations-idempotence-audit.md` depuis `crates/kesh-db/README.md` (s'il existe) OU depuis `docs/ci.md` section migrations (fallback si pas de README crate-level).

### T2: Migration `_kesh_version.sql` + boot integration (AC #4-13)

- [ ] T2.1 — Créer `crates/kesh-db/migrations/20260522000001_kesh_version.sql` avec en-tête conforme (commentaire bloc référençant Story 10-2 + AC #4-7) + `-- idempotent: tracked-by-sqlx` + `CREATE TABLE _kesh_version` schéma AC #5 + `INSERT INTO _kesh_version (...) VALUES (1, '0.1.0', '0.1.0')` AC #6.
- [ ] T2.2 — Ajouter `semver = "1"` à `crates/kesh-db/Cargo.toml` `[dependencies]` (sans feature `serde`). Vérifier `cargo build -p kesh-db` PASS.
- [ ] T2.3 — Créer `crates/kesh-db/src/version.rs` :
  - `use semver::Version; use sqlx::MySqlPool;`
  - `#[derive(Debug, thiserror::Error)] pub enum VersionError { ... }` avec variants `DowngradeRefused { db_min: Version, binary: Version }`, `Sqlx(#[from] sqlx::Error)`, `InvalidSemver(#[from] semver::Error)`.
  - `#[derive(Debug)] pub enum DowngradeCheckOutcome { FreshInstall, Aligned, BinaryAhead { db_min: Version, binary: Version } }`.
  - `pub async fn check_downgrade_protection(pool: &MySqlPool, binary_version: &str) -> Result<DowngradeCheckOutcome, VersionError>` — parse `binary_version` via `Version::parse()` (semver crate), execute `sqlx::query_scalar("SELECT kesh_version_min_required FROM _kesh_version WHERE id = 1").fetch_one(pool).await` (version dynamique sans macro `!` — évite la dépendance compile-time sur l'existence de la table en local). Détecter le cas fresh install via le pattern canonique projet : `sqlx::Error::Database(ref db_err) if db_err.try_downcast_ref::<sqlx::mysql::MySqlDatabaseError>().map_or(false, |e| e.number() == 1146)` → `Ok(FreshInstall)`. ⚠️ **NE PAS utiliser** `db_err.code() == Some("1146")` : `code()` retourne le SQLSTATE `"42S02"` (string) et **pas** le numéro MariaDB 1146. Référence canonique du pattern correct : `crates/kesh-db/src/errors.rs:150-151` et `crates/kesh-db/src/retry.rs:73-74`. Sinon parse le VARCHAR retourné en `Version`, compare via `binary.cmp(&db_min)` → 3 cas mapping vers `Aligned` / `BinaryAhead` / `Err(DowngradeRefused)`.
  - `pub async fn record_boot_version(pool: &MySqlPool, binary_version: &str) -> Result<(), VersionError>` — `let r = sqlx::query("UPDATE _kesh_version SET kesh_version_last_applied = ?, last_boot_at = NOW() WHERE id = 1").bind(binary_version).execute(pool).await?;` puis `if r.rows_affected() != 1 { tracing::warn!(rows = r.rows_affected(), "record_boot_version: row id=1 missing?"); }` puis `Ok(())`.
  - Aucun unit test interne au module — couvert par les tests d'intégration T3. Le module entier devrait faire ~80-110 lignes Rust.
- [ ] T2.4 — Exposer le module dans `crates/kesh-db/src/lib.rs` : ajouter `pub mod version;` après les autres `pub mod` existants.
- [ ] T2.5 — Modifier `crates/kesh-api/src/main.rs` selon AC #13 : insérer le bloc `check_downgrade_protection` entre l'init du pool (fin ligne 49 actuelle `tracing::info!("Base de données : connectée")`) et `MIGRATOR.run()` (ligne 62 actuelle), et `record_boot_version` après le `tracing::info!("Migrations appliquées")` (ligne 66 actuelle). Ajuster les commentaires de la docstring `//! Ordre de démarrage` en haut du fichier (lignes 1-13) pour refléter les nouveaux steps 3b et 4b.

### T3: Tests d'intégration migrations (AC #14-18)

- [ ] T3.1 — Créer `crates/kesh-db/tests/migrations_fresh_install.rs` avec 3 tests :
  - `migrations_apply_all_tables_present` (AC #14a) — `SHOW TABLES` + assertion liste tables minimale (~22 tables).
  - `migrations_minimal_seed_roundtrips` (AC #14b) — INSERT/SELECT round-trip 5 lignes minimales.
  - `migrations_kesh_version_initial_row` (AC #14c) — SELECT `_kesh_version` row 1 → assertion `(0.1.0, 0.1.0)`.
- [ ] T3.2 — Créer `crates/kesh-db/tests/migrations_upgrade_path.rs` avec 2 tests :
  - `upgrade_path_preserves_data` (AC #15a) — utilise helper `apply_migrations_up_to` (à inclure inline dans le même fichier de test).
  - `downgrade_protection_rejects_old_binary` (AC #15b) — UPDATE de `kesh_version_min_required` à `'0.99.0'` puis appel `check_downgrade_protection` avec `"0.1.0"` → assertion `Err(DowngradeRefused)`.
- [ ] T3.3 — Implémenter le helper `apply_migrations_up_to(pool, n)` inline dans `migrations_upgrade_path.rs` via la **sub-Migrator approach** (cf. Dev Notes §"Pattern test migrations" pour le code complet). Le champ `Migrator::migrations` est `pub Cow<'static, [Migration]>` (semver-exempt mais public) — slicer `&kesh_db::MIGRATOR.migrations[..n]` et construire un sub-Migrator avec ce slice puis appeler `.run(pool)`. Cela préserve les checksums SHA-384 réels et alimente correctement `_sqlx_migrations`, contrairement à toute approche d'INSERT manuel.
- [ ] T3.4 — Si l'helper T3.3 nécessite plus de 30 lignes Rust, extraire dans un sous-module `tests/common/migrations_helper.rs` (pattern Cargo `#[path = "common/migrations_helper.rs"] mod migrations_helper;` au début du fichier de test). Sinon laisser inline.
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

L'helper `apply_migrations_up_to(pool, n)` pour T3.3 utilise la **sub-Migrator approach**. Le champ `Migrator::migrations` est `pub Cow<'static, [Migration]>` dans sqlx-core 0.8.6 (cf. `src/migrate/migrator.rs:14-22` — annoté `#[doc(hidden)]` + commentaire « semver-exempt may be changed in future version », acceptable pour un test interne — alerte de breaking si bump sqlx) :

```rust
use sqlx::migrate::Migrator;
use sqlx::MySqlPool;
use std::borrow::Cow;

async fn apply_migrations_up_to(pool: &MySqlPool, n: usize) -> Result<(), sqlx::migrate::MigrateError> {
    let all = &kesh_db::MIGRATOR.migrations;
    assert!(n <= all.len(), "apply_migrations_up_to: n={} > total={}", n, all.len());
    let sub = Migrator {
        migrations: Cow::Borrowed(&all[..n]),
        ignore_missing: kesh_db::MIGRATOR.ignore_missing,
        locking: kesh_db::MIGRATOR.locking,
        no_tx: kesh_db::MIGRATOR.no_tx,
    };
    sub.run(pool).await
}
```

Avantages :
- Checksums SHA-384 réels préservés → `MIGRATOR.run()` final ne déclenche pas `MigrateError::VersionMismatch`.
- `_sqlx_migrations` correctement alimentée par `sub.run()` → les migrations restantes sont vues comme "à appliquer" par le `MIGRATOR.run()` final.
- Pas de duplication des fichiers `.sql` ni de logique custom de read+exec.

**Anti-pattern à éviter** : INSERT manuel dans `_sqlx_migrations` avec un checksum bidon (`vec![0u8; 32]` ou similaire). Le checksum sqlx est **SHA-384 = 48 bytes** (`Sha384::digest`, `sqlx-core-0.8.6/src/migrate/migration.rs:25`), pas SHA-256 ; et même avec la bonne taille un faux checksum cause `MigrateError::VersionMismatch` à l'appel `MIGRATOR.run()` final.

**Anti-pattern à éviter** : `Migration::apply(&pool)` n'existe **pas** comme méthode publique sur `Migration` en sqlx 0.8 (la struct ne contient que des données : `version`, `description`, `sql`, `checksum`, etc. — l'apply est porté par le trait `Migrate` implémenté sur les connexions, non-stable). Utiliser sub-Migrator est la voie correcte.

Note : si sqlx bumpe vers 0.9+ et que `migrations` perd la visibilité publique, le pattern devra basculer en fallback custom (read+exec+INSERT avec **vrai** checksum SHA-384 calculé runtime via `sha2 = "0.10"`). Le test échouera proprement (compile error) le cas échéant.

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
        Err(sqlx::Error::Database(ref db_err))
            if db_err
                .try_downcast_ref::<sqlx::mysql::MySqlDatabaseError>()
                .map_or(false, |e| e.number() == 1146) =>
        {
            // ER_NO_SUCH_TABLE 1146 — fresh install, the migration `_kesh_version` hasn't run yet.
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
    let result = sqlx::query(
        "UPDATE _kesh_version SET kesh_version_last_applied = ?, last_boot_at = NOW() WHERE id = 1"
    )
    .bind(binary_version)
    .execute(pool)
    .await?;
    if result.rows_affected() != 1 {
        tracing::warn!(
            rows_affected = result.rows_affected(),
            "record_boot_version: UPDATE affected unexpected number of rows (expected 1) — _kesh_version row id=1 missing?"
        );
    }
    Ok(())
}
```

### MariaDB error code 1146 (ER_NO_SUCH_TABLE) — référence

L'erreur SQLSTATE `"42S02"`, **MariaDB number 1146** « Table 'database.table' doesn't exist » est ce que retourne le SELECT contre `_kesh_version` sur fresh install. **Important** : en sqlx 0.8, `sqlx::Error::Database::code()` retourne le **SQLSTATE string** (`"42S02"`) — pas le numéro MariaDB. Pour matcher le numéro 1146, il faut downcast vers `sqlx::mysql::MySqlDatabaseError` puis appeler `.number()`. Le pattern canonique projet est implémenté dans :

- `crates/kesh-db/src/errors.rs:150-151` (matching sur deadlock, lock timeout, etc.)
- `crates/kesh-db/src/retry.rs:64-74` (detection deadlock pour retry)

Code attendu pour `version.rs` :
```rust
sqlx::Error::Database(ref db_err)
    if db_err.try_downcast_ref::<sqlx::mysql::MySqlDatabaseError>()
        .map_or(false, |e| e.number() == 1146)
```

**NE PAS utiliser** :
- `db_err.code() == Some("1146")` — `code()` retourne `Some("42S02")` (SQLSTATE), pas le numéro.
- `e.to_string().contains("1146")` — textuel fragile, dépend de la formulation du message + casse les tests si le message change.

### Dette latente identifiée pour audit cross-story

- **Story 10-3 « Résilience frontend si DB down »** dépend de `/health` qui pourrait étendre le body retourné par `record_boot_version` (e.g. `{ status: "ok", db: true, version: "0.1.0", min_required: "0.1.0" }`). Hors scope Story 10-2 mais à coordonner.
- **Story 10-4 « Manuel install Synology »** doit documenter dans la section "Update" : « après pull image + restart container, vérifier `docker logs kesh-api | grep _kesh_version` pour confirmer le bump si une migration breaking a été appliquée ». Hors scope Story 10-2 mais à mémoriser.
- **FR78 « avertir backup avant migration »** : Story 10-2 livre la **détection** (count des migrations pending via `MIGRATOR.iter()` vs `SELECT version FROM _sqlx_migrations`), mais le **log warning** « Backup recommended before applying N pending migrations » au boot est un AC borderline. Décision pre-dev : couvert par le log informatif ajouté **APRÈS** `MIGRATOR.run()` réussi (« Migrations appliquées : N nouvelles »), pas par un warning AVANT (qui imposerait à l'admin d'arrêter le boot et redémarrer après backup — pattern fragile en pratique). FR78 textuel sera ré-évalué Story 10-4 manuel install si insuffisant.

### Anti-patterns à éviter (extrait des codes review Story 10-1)

- **NE PAS modifier les fichiers `.sql` historiques** (AC #2 explicite). Risque : `MigrateError::VersionMismatch` **garanti** sur toute DB déjà migrée. Le checksum sqlx est SHA-384 du contenu complet du fichier (cf. `sqlx-core-0.8.6/src/migrate/migration.rs:25`) et le `MIGRATOR.run()` compare strictement (cf. `migrator.rs:175-176`). **Même l'ajout d'un commentaire `-- ...` casse le checksum**. C'est la raison pour laquelle Pass 1 spec validate a pivoté l'audit AC #1-3 vers un fichier markdown séparé `docs/migrations-idempotence-audit.md`.
- **NE PAS** ajouter `serde` feature à `semver` (AC #8). La struct `Version` ne traverse pas de boundary serialisée — uniquement parsée depuis VARCHAR DB et comparée. La feature `serde` augmente le scope binaire sans gain.
- **NE PAS** utiliser `.unwrap()` ou `.expect()` dans `version.rs` (clippy + CLAUDE.md). Toutes les erreurs descendent via `?` → `VersionError`. Le caller (`main.rs`) décide d'exit ou de log warn selon le type d'erreur.
- **NE PAS** utiliser `db_err.code() == Some("1146")` pour détecter ER_NO_SUCH_TABLE — `code()` retourne le SQLSTATE `"42S02"`. Pattern correct : `try_downcast_ref::<MySqlDatabaseError>().map_or(false, |e| e.number() == 1146)` (cf. `crates/kesh-db/src/errors.rs:150`).
- **NE PAS** simuler le tracking `_sqlx_migrations` par INSERT manuel avec checksum bidon dans les tests (cf. Dev Notes §"Pattern test migrations" — anti-pattern documenté). Utiliser sub-Migrator avec `&kesh_db::MIGRATOR.migrations[..n]`.

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
- `crates/kesh-db/src/errors.rs:150-151` — pattern canonique `db_err.try_downcast_ref::<sqlx::mysql::MySqlDatabaseError>().map_or(false, |e| e.number() == ...)` à reproduire dans `version.rs` (T2.3) pour matcher ER_NO_SUCH_TABLE 1146
- `crates/kesh-db/src/retry.rs:64-74` — deuxième occurrence du même pattern (deadlock detection)
- `crates/kesh-db/Cargo.toml:11` — ligne sqlx existante (référence features actives), `semver = "1"` à ajouter en `[dependencies]`
- `crates/kesh-api/src/main.rs:49` — `tracing::info!("Base de données : connectée")` — ancre AVANT laquelle insérer le bloc `check_downgrade_protection` (étape boot 3b) T2.5
- `crates/kesh-api/src/main.rs:62` — `if let Err(e) = kesh_db::MIGRATOR.run(&pool).await` — code à conserver, étape boot 4
- `crates/kesh-api/src/main.rs:66` — `tracing::info!("Migrations appliquées")` — ancre APRÈS laquelle insérer `record_boot_version` (étape boot 4b) T2.5
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

## 🚨 Questions ouvertes (résolues Pass 1)

| # | Question | Résolution Pass 1 spec validate |
|---|---|---|
| Q1 | sqlx 0.8 expose-t-il un moyen d'appliquer une sub-slice du Migrator ? | **Résolu** : oui, via `kesh_db::MIGRATOR.migrations[..n]` (champ `Cow<'static, [Migration]>`, semver-exempt mais public en 0.8.6). Sub-Migrator construit avec ce slice → `.run(pool)`. Voir Dev Notes §"Pattern test migrations" pour le code complet. |
| Q2 | Le checksum sqlx tolère-t-il l'ajout de commentaires `-- idempotent: ...` rétroactif ? | **Résolu** : non. SHA-384 comparé strictement (`sqlx-core-0.8.6/src/migrate/migrator.rs:175-176`) → `MigrateError::VersionMismatch` garanti si checksum diffère. **Plan adopté** : audit dans `docs/migrations-idempotence-audit.md` (markdown séparé, zéro modif des `.sql`). Cf. AC #1-3 réécrits + Anti-patterns. |
| Q3 | Migration `_kesh_version.sql` doit-elle être appliquée AVANT ou APRÈS le check downgrade au boot ? | **Résolu** : APRÈS (cf. AC #11 — check retourne `FreshInstall` si table absente via match `MySqlDatabaseError::number() == 1146`, puis `MIGRATOR.run` crée la table, puis `record_boot_version` UPDATE). Les tests upgrade_path AC #15a n'invoquent pas `check_downgrade_protection` avant le full `MIGRATOR.run` — pas de régression possible. |
| Q4 | Faut-il un test de **pure idempotence run twice** (`MIGRATOR.run` × 2) ? | **Résolu** : différé à code review. Couvert implicitement par `#[sqlx::test]` qui invoque `MIGRATOR` et par `_sqlx_migrations` tracking déjà éprouvé par 600+ tests existants. Non-requis par les ACs Epic. |
| Q5 | Faut-il formaliser une check CI automatique pour P3 « migration breaking » ? | **Résolu** : non en v0.1. Overhead non-justifié, exceptions P4 fragiles à grep. Laissé à la discipline code review humain + LLM (cohérent CLAUDE.md §"Migration breaking policy" P3). |

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

#### Pass 1 spec validate — Sonnet 4.6 (2026-05-21)

Verdict initial : 2 CRITICAL + 4 HIGH + 5 MEDIUM + 4 LOW = 15 findings → NEEDS PASS 2.

**Patches appliqués (15 patches, tous findings ≥ MEDIUM)** :

- **F1 (CRITICAL)** — Pattern erreur `db_err.code() == Some("1146")` (incorrect : retourne le SQLSTATE `"42S02"`) → corrigé partout pour `try_downcast_ref::<sqlx::mysql::MySqlDatabaseError>().map_or(false, |e| e.number() == 1146)`. Modifications : Dev Notes §"Pattern version.rs", Dev Notes §"MariaDB error code 1146", AC #11, T2.3, Anti-patterns. Référence canonique projet : `crates/kesh-db/src/errors.rs:150-151` (ajouté aux References).
- **F2 (CRITICAL)** — Approche B "fake checksum" (`vec![0u8; 32]`) → `MigrateError::VersionMismatch` garanti (SHA-384 réel = 48 bytes, et même la bonne taille avec faux contenu casse). `Migration::apply()` n'existe pas comme méthode publique. Solution adoptée : sub-Migrator approach via `kesh_db::MIGRATOR.migrations[..n]` (champ public en sqlx 0.8.6, doc(hidden) mais accessible). Dev Notes §"Pattern test migrations" et T3.3 entièrement réécrits.
- **F3 (HIGH)** — Anti-patterns §"sqlx tolère un checksum différent" (FAUX : VersionMismatch déterministe ligne 175-176 de migrator.rs) → réécrit pour refléter la réalité source-confirmée.
- **F4 (HIGH)** — Compteurs T1.3 « 19 » et T1.4 « 6 » incorrects → corrigés (15 CREATE TABLE + 10 ALTER + 1 CREATE INDEX = 26 fichiers vérifiés par grep). T1 entièrement réécrit avec listes correctes et `company_invoice_settings.sql` reclassifié CREATE INDEX-only.
- **F5 (HIGH)** — Table `fiscal_years` manquante dans liste minimale AC #14a → ajoutée (compte « ~22 » → « ~23 »). Vérifié par `grep -n "^CREATE TABLE" migrations/*.sql` : `initial_schema.sql:39`.
- **F6 (HIGH)** — Contradiction scope L43 « log AVANT MIGRATOR.run » vs Dev Notes L399 « APRÈS, pas AVANT » → scope harmonisé sur position APRÈS (cohérent Dev Notes + AC #13 qui n'ajoute aucun log pre-migration).
- **F7 (MEDIUM)** — `Migration::apply(&pool)` skeleton fantôme → supprimé, remplacé par sub-Migrator correct.
- **F8 (MEDIUM)** — Q2 décision architecturale : audit ne peut PAS être in-SQL (F3 le prouve). **Décision Guy** : Plan B — créer `docs/migrations-idempotence-audit.md` markdown séparé, zéro modif des `.sql` historiques. AC #1-3 + T1 entièrement réécrits.
- **F9 (MEDIUM)** — `record_boot_version` sans check `rows_affected` → ajouté défensivement (warn si != 1). AC #12 + Dev Notes §"Pattern version.rs" mis à jour.
- **F10 (MEDIUM)** — T2.5 « ligne 67 » incorrect → corrigé « ligne 66 actuelle » (grep confirmé : `tracing::info!("Migrations appliquées")` est à `main.rs:66`).
- **F11 (MEDIUM)** — Q3 « résolu spec » trop vague → reformulé précisément en termes de tests upgrade_path.
- **F12 (LOW)** — AC #12 `$1` (style PostgreSQL) → `?` (style MariaDB).
- **F13 (LOW)** — Q4 fermé (différé à code review).
- **F14 (LOW)** — AC #8 note ajoutée : `semver 1.0.x` déjà résolu dans Cargo.lock transitivement (cargo-metadata).
- **F15 (LOW)** — Dev Notes "15 min d'investigation" supprimé (API connue ground-truth).

**Vérification ground-truth orchestrateur** : tous les findings CRITICAL et HIGH ont été vérifiés par grep/Read avant patch (Bash commands : `grep -rn "try_downcast_ref" crates/kesh-db/src/`, `find sqlx-core-0.8 source for Sha384/VersionMismatch`, classification réelle des 26 migrations par `for f; do head; grep CREATE/ALTER; done`, vérification `fiscal_years` dans `initial_schema:39`).

Prochaine étape : Pass 2 spec validate avec **Haiku 4.5** (rotation cycle Sonnet → Haiku → Opus → Sonnet, conformément CLAUDE.md §"Review Iteration Rule"). Discipline grep ground-truth obligatoire pour tout finding Haiku CRITICAL/HIGH (cf. memory `feedback_haiku_review_diff_combined`).

#### Pass 2 spec validate — Haiku 4.5 (2026-05-21)

Verdict initial : 0 CRITICAL + 0 HIGH + 1 MEDIUM + 2 LOW = 3 findings → CONDITIONAL PASS (encore 1 MEDIUM bloquant convergence).

**Patches appliqués (3 findings Haiku + 2 régressions Pass 1 détectées par l'orchestrateur)** :

- **F1-P2 (MEDIUM Haiku)** — AC #22 P3 listait `ALTER COLUMN type` (syntaxe PostgreSQL invalide en MariaDB). Ground-truth verifié : la migration `users_company_id.sql:23` utilise `ALTER TABLE ... MODIFY COLUMN`. P3 corrigé pour lister `MODIFY COLUMN <type>` et `CHANGE COLUMN <old> <new> <type>` (syntaxe MariaDB) + note dialecte ajoutée. P1 (définition) aligné de même.
- **F2-P2 (LOW Haiku, escaladé à MEDIUM)** — AC #15a référençait encore `Migration::apply(&pool)` (méthode publique inexistante en sqlx 0.8) malgré la pivot sub-Migrator de Pass 1 Dev Notes. AC #15a réécrit pour pointer explicitement sur le sub-Migrator pattern, et précise que le test upgrade utilise `#[sqlx::test]` SANS attribut `migrator` (sinon MIGRATOR complet s'applique avant le corps du test, ce qui invaliderait le scénario).
- **F3-P2 (LOW Haiku)** — Math clarté AC #15a : explicité que post-story le repo contient 27 migrations totales (26 historiques + 1 nouvelle) et que `MIGRATOR.run()` final applique 4 migrations dans le test (3 historiques restantes + `_kesh_version.sql`).
- **R1-P2 (MEDIUM régression Pass 1, manquée par Haiku)** — AC #23 référençait encore « commentaire d'idempotence ajouté par AC #1 sur chaque migration historique » alors que Pass 1 a pivoté AC #1 vers `docs/migrations-idempotence-audit.md` séparé (zéro modif des `.sql`). AC #23 réécrit pour parler uniquement de la décision « pas de marker `-- breaking-skip-bump:` dans les migrations historiques » sans plus référencer l'in-SQL commentaire d'idempotence retiré.
- **R2-P2 (MEDIUM régression Pass 1, manquée par Haiku)** — Contradiction AC #15a (post-F2-P2 : « `#[sqlx::test]` sans migrator ») vs AC #18 (« chacun déclare `#[sqlx::test(migrator = ...)]` » historiquement). AC #18 amendé : `migrations_fresh_install.rs` utilise migrator (full migration avant le test), `migrations_upgrade_path.rs` utilise `#[sqlx::test]` sans migrator (le test contrôle l'application).

**Vérification ground-truth orchestrateur** :
- F1-P2 vérifié par `grep -nE "ALTER COLUMN|MODIFY COLUMN|CHANGE COLUMN" crates/kesh-db/migrations/*.sql` → `users_company_id.sql:23` confirme MariaDB syntax.
- R1-P2 vérifié par lecture directe AC #23 vs AC #1 (post-Pass-1).
- R2-P2 vérifié par contradiction logique entre AC #15a réécrit et AC #18 inchangé.

**Note sur Haiku** : conformément feedback_haiku_review_diff_combined memory, Haiku 4.5 a fait un travail solide sur la vérification ground-truth (16 vérifications listées : table count, line numbers, error patterns, etc.) mais a manqué les régressions internes entre ACs (R1, R2). Pattern attendu — Haiku est bon pour vérifier l'existence d'éléments mais moins fort sur la cohérence cross-AC du document. Pass 3 (Opus 4.7) sera utile pour boucler la cohérence globale.

Prochaine étape : Pass 3 spec validate avec **Opus 4.7** (rotation cycle).
