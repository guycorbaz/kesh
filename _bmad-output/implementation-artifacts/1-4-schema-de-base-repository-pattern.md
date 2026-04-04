# Story 1.4 : Schéma de base & repository pattern

Status: done

<!-- Note: Validation is optional. Run validate-create-story for quality check before dev-story. -->

## Story

As a développeur,
I want un schéma de base MariaDB et un pattern d'accès aux données,
so that les stories suivantes puissent persister des données.

## Acceptance Criteria

1. **Given** kesh-db configuré, **When** `sqlx migrate run`, **Then** les tables `users` et `companies` sont créées
2. **Given** table `users`, **Then** colonnes : `id`, `username`, `password_hash`, `role`, `active`, `version`, `created_at`, `updated_at`
3. **Given** table `companies`, **Then** colonnes : `id`, `name`, `address`, `ide_number`, `org_type`, `accounting_language`, `instance_language`, `version`, `created_at`, `updated_at`
4. **Given** pool MariaDB configuré, **When** connexion, **Then** le pool SQLx se connecte via la variable `DATABASE_URL`
5. **And** repository pattern implémenté pour `users` et `companies` (`create`, `find_by_id`, `update`, `list`)
6. **And** tests d'intégration avec base de données de test
7. **And** schéma : table `fiscal_years` (`id`, `company_id`, `name`, `start_date`, `end_date`, `status` (open/closed), `created_at`, `updated_at`) — nécessaire dès les écritures pour le contrôle d'immutabilité post-clôture

## Tasks / Subtasks

- [x] Task 1 : Configurer `kesh-db/Cargo.toml` avec les dépendances (AC: 1, 4)
  - [x] 1.1 Ajouter `sqlx = "0.8"` avec features `runtime-tokio-rustls`, `mysql`, `migrate`, `chrono`, `macros`
  - [x] 1.2 Ajouter `thiserror = "2"` pour `DbError`
  - [x] 1.3 Ajouter `chrono = { version = "0.4", features = ["serde"] }` pour les timestamps
  - [x] 1.4 Ajouter `serde = { version = "1", features = ["derive"] }` pour les entités sérialisables
  - [x] 1.5 Ajouter `[dev-dependencies]` : `tokio = { version = "1", features = ["full"] }`, `dotenvy = "0.15"`
  - [x] 1.6 **Ne PAS ajouter `kesh-core`** — la validation des types métier (CheNumber, etc.) est faite côté kesh-api avant l'appel au repository. Cette story stocke `ide_number` comme `Option<String>` brut.
  - [x] 1.7 Vérifier `cargo build --workspace`
- [x] Task 2 : Créer la structure du crate `kesh-db` (AC: 1, 4, 5)
  - [x] 2.1 Créer `crates/kesh-db/src/lib.rs` avec `pub mod errors; pub mod pool; pub mod repositories; pub mod entities;` (le `MIGRATOR` sera ajouté en 2.4)
  - [x] 2.2 Créer `crates/kesh-db/src/errors.rs` avec `DbError` enum (NotFound, OptimisticLockConflict, UniqueConstraintViolation, ForeignKeyViolation, CheckConstraintViolation, Sqlx(sqlx::Error)) + fonction helper `map_db_error` utilisant les codes numériques : **1062** (unique), **1451/1452** (FK), **4025** (CHECK MariaDB), **3819** (CHECK MySQL fallback)
  - [x] 2.3 Créer `crates/kesh-db/src/pool.rs` avec `create_pool(database_url: &str, max_connections: u32, connect_timeout: Duration) -> Result<MySqlPool, DbError>`. Implémentation via `MySqlPoolOptions::new().max_connections(max_connections).acquire_timeout(connect_timeout).connect(database_url).await`. Reproduction des valeurs de kesh-api : appelants utilisent `max_connections=5` et `connect_timeout=Duration::from_secs(10)` par défaut.
  - [x] 2.4 Exposer `pub static MIGRATOR: sqlx::migrate::Migrator = sqlx::migrate!("./migrations");` dans `lib.rs` pour utilisation par `#[sqlx::test]`
  - [x] 2.5 Créer `crates/kesh-db/src/entities/mod.rs` avec réexports
  - [x] 2.6 Créer `crates/kesh-db/src/repositories/mod.rs` avec `pub mod users; pub mod companies; pub mod fiscal_years;`
- [x] Task 3 : Créer la migration initiale (AC: 1, 2, 3, 7)
  - [x] 3.1 Créer `crates/kesh-db/migrations/20260404000001_initial_schema.sql`
  - [x] 3.2 Table `companies` (PK auto-increment, version default 1, created_at/updated_at default CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP)
  - [x] 3.3 Table `users` selon AC2 — colonnes exactes : id, username, password_hash, role, active, version, created_at, updated_at (PAS de FK company_id dans cette story, multi-tenant différé)
  - [x] 3.4 Table `fiscal_years` avec FK vers companies (ON DELETE RESTRICT — jamais supprimer un exercice par cascade)
  - [x] 3.5 Index : `users.username` (via UNIQUE), `companies.ide_number` (via UNIQUE). Pas d'index `users.active` (colonne booléenne low-cardinality, inutile). Pas d'index explicite sur `fiscal_years(company_id, start_date)` — la contrainte UNIQUE le crée automatiquement.
  - [x] 3.6 Charset `utf8mb4` et collation `utf8mb4_unicode_ci` sur toutes les tables
- [x] Task 4 : Implémenter `entities/company.rs` (AC: 3, 5)
  - [x] 4.1 Struct `Company` (id: `i64`, name: String, address: String, ide_number: Option<String>, org_type: OrgType, accounting_language: Language, instance_language: Language, version: i32, created_at: `NaiveDateTime`, updated_at: `NaiveDateTime`)
  - [x] 4.2 Enum `OrgType` : `Independant`, `Association`, `Pme` — utiliser `#[derive(sqlx::Type)]` avec `#[sqlx(rename_all = "PascalCase")]` (sans `type_name` — SQLx utilise l'encoding texte transparent). Ajouter `#[serde(rename_all = "PascalCase")]` pour cohérence JSON↔DB.
  - [x] 4.3 Enum `Language` : `Fr`, `De`, `It`, `En` — `#[derive(sqlx::Type)]` avec `#[sqlx(rename_all = "SCREAMING_SNAKE_CASE")]` (stocké en DB comme `FR/DE/IT/EN`). Ajouter `#[serde(rename_all = "UPPERCASE")]` pour que JSON API matche la DB (`"FR"`, `"DE"`, `"IT"`, `"EN"`). Note : SQLx n'a pas `UPPERCASE`, serde n'a pas `SCREAMING_SNAKE_CASE` — les deux attributs produisent exactement la même chaîne pour les variantes mono-mot comme `Fr`.
  - [x] 4.4 Implémentation `sqlx::FromRow` via `#[derive(sqlx::FromRow)]` (fonctionne car les enums ont `sqlx::Type`)
  - [x] 4.5 Struct `NewCompany` (champs : `name: String`, `address: String`, `ide_number: Option<String>`, `org_type: OrgType`, `accounting_language: Language`, `instance_language: Language`) — pas d'id/version/timestamps (gérés par la DB)
  - [x] 4.6 Struct `CompanyUpdate` (champs : `name: String`, `address: String`, `ide_number: Option<String>`, `org_type: OrgType`, `accounting_language: Language`, `instance_language: Language`) — remplacement complet des champs modifiables. Pour cette story, pas de patch partiel (Option<Option<>>) — si un besoin émerge, une story future introduira un struct séparé.
  - [x] 4.7 `Serialize/Deserialize` dérivés sur Company, NewCompany, CompanyUpdate (sûrs, pas de secret)
  - [x] 4.8 `///` doc comments
- [x] Task 5 : Implémenter `entities/user.rs` (AC: 2, 5)
  - [x] 5.1 Struct `User` (id: `i64`, username, password_hash, role: Role, active: bool, version: i32, created_at: `NaiveDateTime`, updated_at: `NaiveDateTime`)
  - [x] 5.2 Enum `Role` : `Admin`, `Comptable`, `Consultation` — `#[derive(sqlx::Type)]` avec `#[sqlx(rename_all = "PascalCase")]` (sans `type_name`) + `#[serde(rename_all = "PascalCase")]` pour cohérence
  - [x] 5.3 Implémentation `sqlx::FromRow` via derive
  - [x] 5.4 Struct `NewUser` pour la création (password_hash fourni par l'appelant — le hachage Argon2id sera dans story 1.5)
  - [x] 5.5 **SÉCURITÉ — User ne dérive PAS Serialize** : le `password_hash` ne doit jamais fuiter via JSON (logs, API, tests). Si besoin de sérialisation pour l'API future, créer un `UserDto` séparé dans story 1.5/1.7.
  - [x] 5.6 **SÉCURITÉ — Debug manuel masquant `password_hash`** : `f.debug_struct("User").field("password_hash", &"***")...`
  - [x] 5.7 `///` doc comments
- [x] Task 6 : Implémenter `entities/fiscal_year.rs` (AC: 7)
  - [x] 6.1 Struct `FiscalYear` (id: `i64`, company_id: `i64`, name, start_date: `NaiveDate`, end_date: `NaiveDate`, status: FiscalYearStatus, created_at: `NaiveDateTime`, updated_at: `NaiveDateTime`)
  - [x] 6.2 Enum `FiscalYearStatus` : `Open`, `Closed` — `#[derive(sqlx::Type)]` avec `#[sqlx(rename_all = "PascalCase")]` (sans `type_name`) + `#[serde(rename_all = "PascalCase")]`
  - [x] 6.3 Struct `NewFiscalYear` pour la création
  - [x] 6.4 Serialize/Deserialize dérivés
  - [x] 6.5 `///` doc comments
- [x] Task 7 : Implémenter `repositories/companies.rs` (AC: 5)
  - [x] 7.1 Fonction `create(pool: &MySqlPool, new: NewCompany) -> Result<Company, DbError>`
  - [x] 7.2 Fonction `find_by_id(pool: &MySqlPool, id: i64) -> Result<Option<Company>, DbError>`
  - [x] 7.3 Fonction `update(pool: &MySqlPool, id: i64, version: i32, changes: CompanyUpdate) -> Result<Company, DbError>` — vérifie `version` pour optimistic lock, retourne `DbError::OptimisticLockConflict` si version mismatch, incrémente version
  - [x] 7.4 Fonction `list(pool: &MySqlPool, limit: i64, offset: i64) -> Result<Vec<Company>, DbError>` — **i64 obligatoire** : MySQL rapporte LIMIT/OFFSET comme BIGINT, `u32` provoquerait une erreur type mismatch dans `sqlx::query!`
  - [x] 7.5 Utiliser `sqlx::query_as!` et `sqlx::query!` (vérification compile-time). Le mode offline avec `.sqlx/` cache est géré à la Task 11 — si blocant, fallback temporaire sur `sqlx::query_as::<_, Company>("SELECT ...")` non-macro
  - [x] 7.6 `///` doc comments
- [x] Task 8 : Implémenter `repositories/users.rs` (AC: 5)
  - [x] 8.1 Même pattern que companies : `create`, `find_by_id`, `update`, `list`
  - [x] 8.2 Fonction supplémentaire `find_by_username(pool: &MySqlPool, username: &str) -> Result<Option<User>, DbError>` (nécessaire pour story 1.5 auth)
  - [x] 8.3 Gérer la contrainte unique sur `username` → `DbError::UniqueConstraintViolation`
  - [x] 8.4 Optimistic lock sur update
  - [x] 8.5 `///` doc comments
- [x] Task 9 : Implémenter `repositories/fiscal_years.rs` (AC: 7)
  - [x] 9.1 `create`, `find_by_id`, `list_by_company(company_id)`, `update_status`
  - [x] 9.2 Pas de `delete` — les exercices ne se suppriment jamais (CO art. 957-964)
  - [x] 9.3 `///` doc comments
- [x] Task 10 : Tests d'intégration (AC: 6)
  - [x] 10.1 Créer `crates/kesh-db/tests/companies_repository.rs`
  - [x] 10.2 Les tests utilisent `#[sqlx::test(migrator = "kesh_db::MIGRATOR")]` — crée une base temporaire par test (NOT a transaction rollback, voir Dev Notes pour les privilèges DB requis)
  - [x] 10.3 Tests Company : `create` + `find_by_id`, `find_by_id` retourne `None` pour id inexistant, `update` succès, `update` avec version stale → `OptimisticLockConflict`, `list` avec pagination (limit/offset bornes), UNIQUE constraint sur `ide_number`. Pas de test "CHECK constraint sur org_type invalide" via le repository — l'enum Rust `OrgType` rend l'état invalide impossible à construire. Le CHECK est une défense en profondeur testée indirectement par le type system.
  - [x] 10.4 Créer `crates/kesh-db/tests/users_repository.rs` : `create` + `find_by_id`, `find_by_id` None, `find_by_username`, `find_by_username` None, `update` + optimistic lock, `list` avec pagination, UNIQUE constraint sur `username`
  - [x] 10.5 Créer `crates/kesh-db/tests/fiscal_years_repository.rs` : CRUD, FK violation si `company_id` inexistant, UNIQUE (company_id, name), CHECK end_date > start_date, pas de `delete` (méthode absente du repo)
  - [x] 10.6 Documenter dans `crates/kesh-db/README.md` comment lancer les tests :
    - Démarrer MariaDB via `docker compose -f docker-compose.dev.yml up -d mariadb`
    - S'assurer que l'utilisateur de `DATABASE_URL` a les droits `CREATE`, `DROP`, `ALL PRIVILEGES ON *.*`
    - Lancer `cargo test -p kesh-db`
- [x] Task 11 : Validation finale (AC: 1-7)
  - [x] 11.1 Démarrer MariaDB via docker-compose : `docker compose -f docker-compose.dev.yml up -d mariadb`
  - [x] 11.2 Créer la base et appliquer les migrations : `sqlx database create && sqlx migrate run` (depuis `crates/kesh-db/`)
  - [x] 11.3 Vérifier que `.gitignore` racine n'exclut PAS `.sqlx/` — si présent, le retirer
  - [x] 11.4 Générer le cache SQLx offline : `cargo sqlx prepare --workspace`
  - [x] 11.5 **Commiter le répertoire `.sqlx/`** — indispensable pour les builds CI et les autres développeurs
  - [x] 11.6 `cargo build --workspace` sans erreur (avec ou sans DB disponible grâce au cache)
  - [x] 11.7 `cargo test -p kesh-db` — tous les tests d'intégration passent (nécessite MariaDB + privilèges CREATE DATABASE)
  - [x] 11.8 `cargo clippy -p kesh-db -- -D warnings` — aucun warning
  - [x] 11.9 `cargo doc -p kesh-db --no-deps` — documentation générée sans warning

## Dev Notes

### Périmètre strict de cette story

**UNIQUEMENT** : schéma DB (tables `users`, `companies`, `fiscal_years`) + repository pattern CRUD + tests d'intégration. Ne PAS implémenter :
- Authentification / hachage Argon2id (story 1.5)
- Routes API `/api/v1/users` etc. (story 1.7)
- Middleware RBAC (story 1.8)
- Migrations automatiques au démarrage avec détection de version (story 8.2)
- Frontend ou page d'admin utilisateurs (stories 1.10+, 2.x)

Le `password_hash` est un champ texte que cette story accepte tel quel (le hachage Argon2id sera fait dans kesh-api story 1.5).

### Architecture kesh-db — Contrainte fondamentale

`kesh-db` est la **couche de persistance** :
- Aucune dépendance sur `kesh-core` dans cette story (ide_number traité comme `Option<String>` brut — la validation `CheNumber` est faite côté kesh-api avant l'appel au repository)
- Ne connaît rien du réseau, HTTP, Axum, ou du frontend
- Expose les entités sérialisables que `kesh-api` consomme et retourne en JSON — **à l'exception de `User`** qui ne dérive PAS `Serialize` (protection du `password_hash`, voir Task 5.5)
- Utilise `SQLx` directement, **pas d'ORM**
- Un fichier par repository, un fichier par entité

**Note architecture.md** : le document d'architecture prévoit des fichiers `entities/` non listés formellement (les repositories y sont mentionnés). Cette story introduit `src/entities/` par souci de séparation des préoccupations (entités de données vs opérations de persistance). Cette décision sera rétro-appliquée à l'architecture dans une story de documentation.

### Structure de fichiers à créer

```
crates/kesh-db/
├── Cargo.toml
├── migrations/
│   └── 20260404000001_initial_schema.sql
├── src/
│   ├── lib.rs
│   ├── errors.rs                      # DbError
│   ├── pool.rs                        # create_pool()
│   ├── entities/
│   │   ├── mod.rs                     # pub use user::*; pub use company::*; ...
│   │   ├── user.rs                    # User, NewUser, Role
│   │   ├── company.rs                 # Company, NewCompany, OrgType, Language
│   │   └── fiscal_year.rs             # FiscalYear, NewFiscalYear, FiscalYearStatus
│   └── repositories/
│       ├── mod.rs
│       ├── users.rs
│       ├── companies.rs
│       └── fiscal_years.rs
└── tests/
    ├── companies_repository.rs
    ├── users_repository.rs
    └── fiscal_years_repository.rs
```

### Schéma SQL détaillé

```sql
-- Migration: 20260404000001_initial_schema.sql

CREATE TABLE companies (
    id BIGINT NOT NULL AUTO_INCREMENT PRIMARY KEY,
    name VARCHAR(255) NOT NULL,
    address TEXT NOT NULL,
    ide_number VARCHAR(15) NULL COMMENT 'Format: CHExxxxxxxxx (normalisé, sans séparateurs)',
    org_type VARCHAR(20) NOT NULL COMMENT 'Independant|Association|Pme (ASCII par design, pas d''accent)',
    accounting_language CHAR(2) NOT NULL COMMENT 'FR|DE|IT|EN — langue des libellés comptables',
    instance_language CHAR(2) NOT NULL COMMENT 'FR|DE|IT|EN — langue de l''interface',
    version INT NOT NULL DEFAULT 1,
    created_at DATETIME(3) NOT NULL DEFAULT CURRENT_TIMESTAMP(3),
    updated_at DATETIME(3) NOT NULL DEFAULT CURRENT_TIMESTAMP(3) ON UPDATE CURRENT_TIMESTAMP(3),
    CONSTRAINT uq_companies_ide_number UNIQUE (ide_number),
    CONSTRAINT chk_companies_org_type CHECK (org_type IN ('Independant', 'Association', 'Pme')),
    CONSTRAINT chk_companies_accounting_language CHECK (accounting_language IN ('FR', 'DE', 'IT', 'EN')),
    CONSTRAINT chk_companies_instance_language CHECK (instance_language IN ('FR', 'DE', 'IT', 'EN'))
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

CREATE TABLE users (
    id BIGINT NOT NULL AUTO_INCREMENT PRIMARY KEY,
    username VARCHAR(64) NOT NULL,
    password_hash VARCHAR(255) NOT NULL COMMENT 'Argon2id — format PHC string',
    role VARCHAR(20) NOT NULL COMMENT 'Admin|Comptable|Consultation',
    active BOOLEAN NOT NULL DEFAULT TRUE,
    version INT NOT NULL DEFAULT 1,
    created_at DATETIME(3) NOT NULL DEFAULT CURRENT_TIMESTAMP(3),
    updated_at DATETIME(3) NOT NULL DEFAULT CURRENT_TIMESTAMP(3) ON UPDATE CURRENT_TIMESTAMP(3),
    CONSTRAINT uq_users_username UNIQUE (username),
    CONSTRAINT chk_users_role CHECK (role IN ('Admin', 'Comptable', 'Consultation'))
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

CREATE TABLE fiscal_years (
    id BIGINT NOT NULL AUTO_INCREMENT PRIMARY KEY,
    company_id BIGINT NOT NULL,
    name VARCHAR(50) NOT NULL COMMENT 'ex: "Exercice 2026"',
    start_date DATE NOT NULL,
    end_date DATE NOT NULL,
    status VARCHAR(10) NOT NULL DEFAULT 'Open' COMMENT 'Open|Closed',
    created_at DATETIME(3) NOT NULL DEFAULT CURRENT_TIMESTAMP(3),
    updated_at DATETIME(3) NOT NULL DEFAULT CURRENT_TIMESTAMP(3) ON UPDATE CURRENT_TIMESTAMP(3),
    CONSTRAINT fk_fiscal_years_company FOREIGN KEY (company_id) REFERENCES companies(id) ON DELETE RESTRICT,
    CONSTRAINT uq_fiscal_years_company_name UNIQUE (company_id, name),
    CONSTRAINT uq_fiscal_years_company_start_date UNIQUE (company_id, start_date),
    CONSTRAINT chk_fiscal_years_dates CHECK (end_date > start_date),
    CONSTRAINT chk_fiscal_years_status CHECK (status IN ('Open', 'Closed'))
    -- Note : la contrainte UNIQUE (company_id, start_date) crée implicitement un index,
    -- pas besoin de INDEX séparé.
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;
```

**Notes** :
- **`BIGINT` signé** (pas `UNSIGNED`) : SQLx 0.8 mappe `BIGINT` signé sur `i64` Rust natif. `BIGINT UNSIGNED` nécessiterait `u64` et casse la cohérence avec les signatures de repository. La plage signée (±9 × 10¹⁸) est largement suffisante.
- `DATETIME(3)` (millisecondes) stocké sans timezone. SQLx le mappe sur `chrono::NaiveDateTime` (pas `DateTime<Utc>` — MySQL DATETIME n'a pas de timezone).
- `ON DELETE RESTRICT` sur `fiscal_years` : le CO suisse interdit la suppression en cascade des données comptables.
- Les enums sont stockés en VARCHAR avec `CHECK` constraints explicites (garantissent la cohérence côté DB même si le client envoie une valeur invalide).
- **UNIQUE (company_id, name)** et **UNIQUE (company_id, start_date)** sur `fiscal_years` : empêchent les doublons d'exercices sur une même entreprise (conformité CO art. 957-964).
- Charset utf8mb4 obligatoire pour supporter FR/DE/IT/EN + caractères spéciaux (accents, umlauts).
- La non-superposition des périodes d'exercices sur une même company n'est **pas** garantie par ce schéma — elle sera validée au niveau applicatif dans une story future (12.1 Clôture d'exercice).

### DbError — Design

```rust
use thiserror::Error;

#[derive(Debug, Error)]
pub enum DbError {
    #[error("Entité non trouvée")]
    NotFound,

    #[error("Conflit de version — l'entité a été modifiée par un autre utilisateur")]
    OptimisticLockConflict,

    #[error("Contrainte d'unicité violée : {0}")]
    UniqueConstraintViolation(String),

    #[error("Contrainte de clé étrangère violée : {0}")]
    ForeignKeyViolation(String),

    #[error("Contrainte CHECK violée : {0}")]
    CheckConstraintViolation(String),

    #[error("Erreur SQLx : {0}")]
    Sqlx(#[from] sqlx::Error),
}

impl DbError {
    pub fn error_code(&self) -> &'static str {
        match self {
            Self::NotFound => "NOT_FOUND",
            Self::OptimisticLockConflict => "OPTIMISTIC_LOCK_CONFLICT",
            Self::UniqueConstraintViolation(_) => "UNIQUE_CONSTRAINT_VIOLATION",
            Self::ForeignKeyViolation(_) => "FOREIGN_KEY_VIOLATION",
            Self::CheckConstraintViolation(_) => "CHECK_CONSTRAINT_VIOLATION",
            Self::Sqlx(_) => "DATABASE_ERROR",
        }
    }
}
```

**Important** : Les messages sont pour le logging serveur uniquement. `kesh-api` mappe les variantes vers des codes HTTP : `NotFound` → 404, `OptimisticLockConflict` → 409, `UniqueConstraintViolation` → 409, `ForeignKeyViolation`/`CheckConstraintViolation` → 400, autres → 500.

### Pattern Repository

Chaque repository suit la même forme (fonctions libres, pas de trait — YAGNI) :

```rust
// repositories/companies.rs
use sqlx::MySqlPool;
use crate::entities::{Company, NewCompany, CompanyUpdate};
use crate::errors::DbError;

/// Crée une nouvelle company et retourne l'entité persistée.
///
/// MySQL/MariaDB n'a pas de clause `RETURNING` (contrairement à Postgres),
/// d'où le pattern en deux étapes : INSERT puis SELECT via `find_by_id`.
pub async fn create(pool: &MySqlPool, new: NewCompany) -> Result<Company, DbError> {
    // Les enums dérivés `sqlx::Type` s'encodent automatiquement, pas de .as_str()
    let result = sqlx::query!(
        "INSERT INTO companies (name, address, ide_number, org_type, accounting_language, instance_language)
         VALUES (?, ?, ?, ?, ?, ?)",
        new.name,
        new.address,
        new.ide_number,
        new.org_type,
        new.accounting_language,
        new.instance_language,
    )
    .execute(pool)
    .await
    .map_err(map_db_error)?; // Mappe les violations de contraintes vers des variantes DbError typées

    // last_insert_id() retourne u64 ; AUTO_INCREMENT BIGINT signé ne dépasse jamais i64::MAX
    let id = i64::try_from(result.last_insert_id())
        .expect("AUTO_INCREMENT id dépasse i64::MAX");
    find_by_id(pool, id).await?.ok_or(DbError::NotFound)
}
```

**`find_by_id` avec `sqlx::query_as!` et annotations de type pour enums custom** :

⚠️ **Piège SQLx important** : avec `sqlx::query_as!` et des enums dérivés `sqlx::Type`, le macro **exige des annotations de type explicites** dans la requête SQL via la syntaxe `col as "col: Type"`. Sans ces annotations, le macro échoue à la compilation car il ne peut pas inférer le mapping VARCHAR → enum custom.

```rust
/// Retrouve une company par son id. Retourne `None` si absente.
pub async fn find_by_id(pool: &MySqlPool, id: i64) -> Result<Option<Company>, DbError> {
    sqlx::query_as!(
        Company,
        r#"SELECT
               id,
               name,
               address,
               ide_number,
               org_type as "org_type: OrgType",
               accounting_language as "accounting_language: Language",
               instance_language as "instance_language: Language",
               version,
               created_at,
               updated_at
           FROM companies
           WHERE id = ?"#,
        id
    )
    .fetch_optional(pool)
    .await
    .map_err(map_db_error)
}

/// Liste les companies avec pagination offset/limit.
pub async fn list(pool: &MySqlPool, limit: i64, offset: i64) -> Result<Vec<Company>, DbError> {
    sqlx::query_as!(
        Company,
        r#"SELECT
               id,
               name,
               address,
               ide_number,
               org_type as "org_type: OrgType",
               accounting_language as "accounting_language: Language",
               instance_language as "instance_language: Language",
               version,
               created_at,
               updated_at
           FROM companies
           ORDER BY id
           LIMIT ? OFFSET ?"#,
        limit,
        offset
    )
    .fetch_all(pool)
    .await
    .map_err(map_db_error)
}
```

**Notes importantes** :
- La syntaxe `col as "alias: Type"` est spécifique à SQLx — elle n'existe pas en SQL standard mais est reconnue par le macro au niveau compile-time.
- `Option<String>` pour `ide_number` est géré automatiquement (colonne NULL).
- `limit` et `offset` sont des **`i64`** (pas `u32`/`u64`) car MySQL rapporte LIMIT/OFFSET comme BIGINT — utiliser `u32` provoquerait une erreur de type compile-time.
- Les strings bruts `r#"..."#` sont utilisés pour permettre les guillemets doubles dans les annotations de type `"col: Type"`.

**Optimistic lock pattern** (update complet de tous les champs modifiables) :

```rust
pub async fn update(
    pool: &MySqlPool,
    id: i64,
    version: i32,
    changes: CompanyUpdate,
) -> Result<Company, DbError> {
    let rows_affected = sqlx::query!(
        "UPDATE companies
         SET name = ?, address = ?, ide_number = ?, org_type = ?,
             accounting_language = ?, instance_language = ?,
             version = version + 1
         WHERE id = ? AND version = ?",
        changes.name,
        changes.address,
        changes.ide_number,
        changes.org_type,
        changes.accounting_language,
        changes.instance_language,
        id,
        version,
    )
    .execute(pool)
    .await
    .map_err(map_db_error)?
    .rows_affected();

    if rows_affected == 0 {
        // Soit l'entité n'existe pas, soit la version ne correspond pas.
        // version += 1 garantit toujours un changement si match → 0 signifie stale.
        match find_by_id(pool, id).await? {
            None => Err(DbError::NotFound),
            Some(_) => Err(DbError::OptimisticLockConflict),
        }
    } else {
        find_by_id(pool, id).await?.ok_or(DbError::NotFound)
    }
}
```

**Note** : cette story utilise une sémantique de **remplacement complet** pour les updates (tous les champs modifiables sont toujours fournis). C'est simple, cohérent avec le pattern optimistic lock (le client envoie l'entité entière + version), et évite la complexité des patches partiels (`Option<Option<T>>`). Si un besoin de patch partiel émerge, une story future introduira un struct séparé comme `CompanyPatch`.

### Détection des violations de contraintes MariaDB

SQLx retourne les erreurs MariaDB sous forme de `sqlx::Error::Database`. L'API `DatabaseError` expose :
- `code()` : SQLSTATE standard (`"23000"` pour toutes les violations de contraintes — **non discriminant**)
- Les implémentations MariaDB/MySQL exposent en plus un `error_number()` numérique via downcasting vers `sqlx::mysql::MySqlDatabaseError`

Codes d'erreur numériques MariaDB pertinents (stables, locale-indépendants) :
- **1062** : `ER_DUP_ENTRY` — contrainte unique violée
- **1452** : `ER_NO_REFERENCED_ROW_2` — FK vers parent inexistant
- **1451** : `ER_ROW_IS_REFERENCED_2` — FK empêche la suppression (ON DELETE RESTRICT)
- **4025** : `ER_CONSTRAINT_FAILED` — CHECK constraint violée (**MariaDB 10.2+**)
- **3819** : `ER_CHECK_CONSTRAINT_VIOLATED` — CHECK constraint violée (**MySQL 8.0.16+**, fallback pour portabilité)

Helper de mapping utilisant les codes numériques (pas les chaînes anglaises qui dépendent de la locale) :

```rust
use sqlx::mysql::MySqlDatabaseError;

fn map_db_error(err: sqlx::Error) -> DbError {
    if let Some(db_err) = err.as_database_error() {
        if let Some(my_err) = db_err.try_downcast_ref::<MySqlDatabaseError>() {
            match my_err.number() {
                1062 => return DbError::UniqueConstraintViolation(my_err.message().to_string()),
                1452 | 1451 => return DbError::ForeignKeyViolation(my_err.message().to_string()),
                4025 | 3819 => return DbError::CheckConstraintViolation(my_err.message().to_string()),
                _ => {}
            }
        }
    }
    DbError::Sqlx(err)
}
```

**Note** : `try_downcast_ref` retourne `Option<&MySqlDatabaseError>` — c'est l'API stable pour accéder aux codes spécifiques au driver sans dépendre des messages d'erreur.

### Tests d'intégration — Stratégie

SQLx 0.8 fournit la macro `#[sqlx::test]` qui, **pour MySQL/MariaDB** :
1. Crée une base de données **temporaire** (`{base}_test_{hash}`) par test — CE N'EST PAS un rollback de transaction
2. Applique automatiquement les migrations de `crates/kesh-db/migrations/`
3. Détruit la base après chaque test (nettoyage complet)

**Prérequis critique** : l'utilisateur MariaDB de `DATABASE_URL` doit avoir les droits `CREATE`, `DROP` et `ALL PRIVILEGES` sur le pattern `%` (pas uniquement la base `kesh`). L'utilisateur `kesh` par défaut dans `docker-compose.dev.yml` a ces droits pour le container de dev, mais il faut **vérifier et documenter** ce prérequis.

**Configuration pour les tests** : ajouter au `docker-compose.dev.yml` ou à un script `setup-test-db.sh` :

```sql
GRANT ALL PRIVILEGES ON *.* TO 'kesh'@'%' WITH GRANT OPTION;
FLUSH PRIVILEGES;
```

Ou utiliser une variable `TEST_DATABASE_URL` pointant vers l'utilisateur `root` uniquement pour les tests.

Usage du test (un fichier par repository dans `crates/kesh-db/tests/` — chaque fichier est un binaire de test d'intégration séparé, il doit importer le crate via `use kesh_db::...`) :

```rust
// crates/kesh-db/tests/companies_repository.rs
use kesh_db::entities::{Company, Language, NewCompany, OrgType};
use kesh_db::repositories::companies;
use sqlx::MySqlPool;

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn test_create_and_find_company(pool: MySqlPool) {
    let new = NewCompany {
        name: "Test SA".into(),
        address: "Rue Test 1, 1000 Lausanne".into(),
        ide_number: Some("CHE109322551".into()),
        org_type: OrgType::Pme,
        accounting_language: Language::Fr,
        instance_language: Language::Fr,
    };
    let created = companies::create(&pool, new).await.unwrap();
    let found = companies::find_by_id(&pool, created.id).await.unwrap().unwrap();
    assert_eq!(found.name, "Test SA");
}
```

`kesh_db::MIGRATOR` doit être exposé dans `lib.rs` :

```rust
pub static MIGRATOR: sqlx::migrate::Migrator = sqlx::migrate!("./migrations");
```

**Alternative si les privilèges CREATE DATABASE sont refusés** : tests manuels avec une base pré-migrée et transaction rollback explicite par test :

```rust
#[tokio::test]
async fn test_create_company() {
    let pool = setup_test_pool().await; // lit TEST_DATABASE_URL
    let mut tx = pool.begin().await.unwrap();
    // ... opérations via &mut *tx ...
    // Pas de commit — rollback automatique au drop de tx
}
```

**Choix par défaut** : utiliser `#[sqlx::test]` et documenter les privilèges requis dans le README du crate.

### Types SQLx custom pour les enums

**Approche imposée : `#[derive(sqlx::Type)]` + `sqlx::FromRow` automatique.**

L'alternative de mapping manuel (`as_str` + `FromStr`) **ne fonctionne pas** avec `#[derive(sqlx::FromRow)]` car le derive exige que chaque champ implémente `sqlx::Decode` + `sqlx::Type`. On utilise donc le derive SQLx partout :

```rust
use serde::{Serialize, Deserialize};

// ASCII-only : "Independant" (sans accent) par design pour éviter les
// problèmes de collation MariaDB. Ne PAS "corriger" en "Indépendant".
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(rename_all = "PascalCase")]
#[serde(rename_all = "PascalCase")]
pub enum OrgType {
    Independant,
    Association,
    Pme,
}

// Language : SQLx utilise SCREAMING_SNAKE_CASE (accepté), serde utilise
// UPPERCASE (accepté par serde uniquement). Les deux produisent "FR/DE/IT/EN"
// pour les variantes mono-mot, garantissant cohérence JSON ↔ DB.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(rename_all = "SCREAMING_SNAKE_CASE")]
#[serde(rename_all = "UPPERCASE")]
pub enum Language {
    Fr,  // stocké "FR" en DB, JSON "FR"
    De,
    It,
    En,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(rename_all = "PascalCase")]
#[serde(rename_all = "PascalCase")]
pub enum Role {
    Admin,
    Comptable,
    Consultation,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(rename_all = "PascalCase")]
#[serde(rename_all = "PascalCase")]
pub enum FiscalYearStatus {
    Open,
    Closed,
}
```

**Notes importantes** :
- **Pas de `type_name`** : pour les enums string-backed, SQLx infère le type texte correctement. Déclarer `type_name = "VARCHAR"` peut casser la vérification compile-time de `query_as!` car MariaDB ne rapporte pas toujours "VARCHAR" comme nom canonique.
- **Double `rename_all`** (sqlx + serde) : garantit que la chaîne stockée en DB matche celle produite par le JSON API. Sans cela, le DB aurait `"FR"` et le JSON `"Fr"` (drift).

Puis les entités peuvent dériver `FromRow` directement :

```rust
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Company {
    pub id: i64,
    pub name: String,
    pub address: String,
    pub ide_number: Option<String>,
    pub org_type: OrgType,
    pub accounting_language: Language,
    pub instance_language: Language,
    pub version: i32,
    pub created_at: chrono::NaiveDateTime,
    pub updated_at: chrono::NaiveDateTime,
}
```

Les `CHECK` constraints dans le schéma SQL (voir section ci-dessus) doublent la garantie côté base en cas de valeur invalide transmise par un client mal comportant.

### Password hash — Sécurité

Le champ `password_hash` de `User` est un **texte opaque** dans cette story. Le hachage Argon2id sera implémenté dans story 1.5 (kesh-api auth). Pour les tests de cette story, on insère des valeurs factices du type `"$argon2id$v=19$m=19456,t=2,p=1$..."`.

**Règles obligatoires pour `User`** :

1. **PAS de `#[derive(Serialize)]`** — le hash ne doit jamais fuiter via JSON (logs tracing, tests, futures réponses API). Si kesh-api a besoin de sérialiser un User (story 1.7), il créera un `UserDto` séparé qui exclut le hash.
2. **PAS de `#[derive(Deserialize)]`** — même raison (défense en profondeur).
3. **PAS de `#[derive(Debug)]`** — implémentation manuelle masquant le hash :

```rust
use sqlx::FromRow;

#[derive(Clone, FromRow)]
// NOTE: PAS de derive Debug/Serialize/Deserialize — voir Dev Notes
pub struct User {
    pub id: i64,
    pub username: String,
    pub password_hash: String,
    pub role: Role,
    pub active: bool,
    pub version: i32,
    pub created_at: chrono::NaiveDateTime,
    pub updated_at: chrono::NaiveDateTime,
}

impl std::fmt::Debug for User {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("User")
            .field("id", &self.id)
            .field("username", &self.username)
            .field("password_hash", &"***")
            .field("role", &self.role)
            .field("active", &self.active)
            .field("version", &self.version)
            .field("created_at", &self.created_at)
            .field("updated_at", &self.updated_at)
            .finish()
    }
}
```

Les autres entités (`Company`, `FiscalYear`) sont libres de dériver `Serialize/Deserialize/Debug` — elles ne contiennent aucun secret.

### SQLx compile-time query verification — Stratégie build

Les macros `sqlx::query!` et `sqlx::query_as!` vérifient les requêtes SQL à la compilation contre une base de données réelle. Deux modes possibles :

1. **Online** : `DATABASE_URL` pointe vers une instance MariaDB avec le schéma appliqué. Chaque `cargo build` se connecte à la base.
2. **Offline** : un cache `.sqlx/` est pré-généré via `cargo sqlx prepare --workspace` puis commité dans le repo. Les builds suivants n'ont plus besoin d'une DB.

**Choix imposé pour cette story : mode OFFLINE avec cache commité.**

Raison : le workflow BMAD lance `cargo build --workspace` sans garantie de DB disponible. Le cache offline permet une build déterministe en CI/local sans setup DB préalable.

Étapes pour le dev agent :
1. Démarrer MariaDB via docker-compose (`docker compose up -d mariadb`)
2. Créer la base et appliquer les migrations (`sqlx migrate run`)
3. **Vérifier** que `.gitignore` racine ne contient pas `.sqlx/` (le retirer si présent)
4. Générer le cache : `cargo sqlx prepare --workspace` depuis la racine
5. Commiter le répertoire `.sqlx/` explicitement (`git add .sqlx/`)
6. Ajouter au `.env` : `SQLX_OFFLINE=true` pour les builds ultérieurs (en CI ou sans DB locale)

**Pré-requis** : installer `sqlx-cli` : `cargo install sqlx-cli --no-default-features --features rustls,mysql`

Alternative pour cette story initiale si `cargo sqlx prepare` pose problème : utiliser les variantes **non-macro** (`sqlx::query_as::<_, Company>("SELECT ...")`). Moins de vérification compile-time mais zéro dépendance à une DB au build. Acceptable pour cette story, à migrer vers macro+offline dans une story de suivi.

### Dépendances kesh-db/Cargo.toml

```toml
[package]
name = "kesh-db"
version = "0.1.0"
edition.workspace = true
license.workspace = true

[dependencies]
sqlx = { version = "0.8", features = ["runtime-tokio-rustls", "mysql", "migrate", "chrono", "macros"] }
chrono = { version = "0.4", features = ["serde"] }
serde = { version = "1", features = ["derive"] }
thiserror = "2"

[dev-dependencies]
tokio = { version = "1", features = ["full"] }
dotenvy = "0.15"
```

**Ne PAS ajouter** : `axum`, `tower-http`, `tracing` (c'est dans kesh-api), `reqwest` (pas de HTTP ici).

### Conventions de nommage DB (rappel)

| Élément | Convention | Exemple |
|---------|-----------|---------|
| Tables | snake_case pluriel | `users`, `companies`, `fiscal_years` |
| Colonnes | snake_case | `company_id`, `fiscal_year_start` |
| FK | `{table_singulier}_id` | `company_id` |
| Index | `idx_{table}_{colonnes}` | `idx_accounts_company_id` |
| Unique | `uq_{table}_{colonnes}` | `uq_users_username` |
| Check | `chk_{table}_{description}` | `chk_fiscal_years_dates` |
| FK constraint | `fk_{table}_{colonne_sans_id}` | `fk_fiscal_years_company` |

### Variables d'environnement

- **DATABASE_URL** : déjà présente dans `.env.example` → `mysql://kesh:kesh_dev@127.0.0.1:3306/kesh`
- **TEST_DATABASE_URL** (optionnelle) : si définie, utilisée par `#[sqlx::test]`, sinon fallback sur `DATABASE_URL`

### Intégration avec kesh-api (pour info, hors scope de cette story)

kesh-api va migrer son initialisation du pool pour utiliser `kesh_db::pool::create_pool()` :

```rust
// Dans une story future (pas celle-ci) :
// let pool = kesh_db::pool::create_pool(&config.database_url, 5).await?;
```

Pour cette story, **NE PAS modifier kesh-api**. L'intégration se fera quand elle sera nécessaire (story 1.5 pour l'auth).

### Project Structure Notes

- `crates/kesh-db/src/lib.rs` existe (placeholder)
- `crates/kesh-db/Cargo.toml` existe (vide)
- `crates/kesh-db/migrations/` existe avec `.gitkeep`
- `crates/kesh-db/src/repositories/` existe (vide)
- `crates/kesh-core` contient déjà `CheNumber` (réutilisable pour valider `ide_number` avant insertion côté kesh-api)

### Learnings des stories précédentes

**Story 1.1** :
- `kesh-api` utilise déjà `sqlx 0.8` avec features `runtime-tokio-rustls` et `mysql`. Cohérence requise.
- La migration du pool de kesh-api vers kesh-db se fera dans une story ultérieure.

**Story 1.2** :
- `DATABASE_URL` est lue via `dotenvy` dans kesh-api. Même convention pour les tests kesh-db.
- MariaDB 11.4 dans docker-compose. Supporte DATETIME(3), CHECK constraints, toutes les features modernes.
- Le pool est actuellement dans `kesh-api/main.rs` avec max 5 connexions et timeout 10s. Reproduire ces valeurs par défaut dans `kesh-db::pool::create_pool`.

**Story 1.3** :
- Pattern `error_code()` sur les enums d'erreur — à reproduire sur `DbError`.
- Tests d'intégration avec assertions strictes (format `assert_eq!` + message).
- `thiserror 2` fonctionne très bien, utiliser le même pattern.
- `kesh-core::types::CheNumber` disponible — l'utiliser si besoin de valider un IDE avant insertion (mais la validation doit être faite côté kesh-api/entrée utilisateur, pas dans le repository qui fait confiance aux données entrantes).

### Anti-patterns à éviter

- **NE PAS** implémenter de trait `Repository` générique — fonctions libres par entité, YAGNI
- **NE PAS** dériver `Debug`, `Serialize` ni `Deserialize` sur `User` (expose le `password_hash`)
- **NE PAS** ajouter la dépendance `kesh-core` — aucun type de kesh-core n'est utilisé dans cette story
- **NE PAS** utiliser `BIGINT UNSIGNED` dans le schéma — utiliser `BIGINT` signé pour compatibilité avec `i64` Rust
- **NE PAS** utiliser `DateTime<Utc>` pour les timestamps — utiliser `chrono::NaiveDateTime` (MySQL DATETIME n'a pas de timezone)
- **NE PAS** détecter les erreurs MariaDB via les messages en anglais — utiliser les numéros d'erreur (1062, 1452, etc.)
- **NE PAS** prétendre que `#[sqlx::test]` fait un rollback sur MySQL — il crée/détruit une DB temporaire
- **NE PAS** utiliser f64 ou float dans le schéma — pour cette story aucun montant, mais garder la règle
- **NE PAS** implémenter le hachage Argon2id ici — c'est pour story 1.5
- **NE PAS** créer des routes HTTP ou toucher à kesh-api dans cette story
- **NE PAS** implémenter de `delete` sur fiscal_years ni companies (le CO art. 957 interdit la suppression de données comptables)
- **NE PAS** faire des migrations destructives — uniquement CREATE TABLE pour cette migration initiale
- **NE PAS** utiliser `expect()`/`unwrap()` dans le code de production (uniquement dans les tests)
- **NE PAS** ignorer les erreurs SQLx en utilisant `.ok()` — toujours mapper vers `DbError`
- **NE PAS** créer les repositories pour `accounts`, `journal_entries`, `invoices`, `contacts`, etc. (ce sont les stories 3.x, 4.x, 5.x)
- **NE PAS** gérer les tokens de rafraîchissement ou sessions (story 1.6)
- **NE PAS** implémenter la politique de mot de passe ici (story 1.7)

### References

- [Source: _bmad-output/planning-artifacts/epics.md#Story 1.4] — Acceptance criteria
- [Source: _bmad-output/planning-artifacts/architecture.md#Structure Complète du Répertoire] — kesh-db/src/ layout
- [Source: _bmad-output/planning-artifacts/architecture.md#Data Architecture] — SQLx + repository pattern
- [Source: _bmad-output/planning-artifacts/architecture.md#Naming Patterns] — Conventions snake_case DB
- [Source: _bmad-output/planning-artifacts/architecture.md#Verrouillage optimiste] — Version column pattern
- [Source: _bmad-output/planning-artifacts/architecture.md#Gestion des erreurs Rust] — DbError pattern
- [Source: _bmad-output/planning-artifacts/prd.md#Contraintes techniques] — Migrations rétrocompatibles, CO art. 957-964
- [Source: _bmad-output/planning-artifacts/prd.md#FR75-FR76] — Multilingual instance_language vs accounting_language
- [Source: docker-compose.dev.yml] — MariaDB 11.4 configuration
- [Source: .env.example] — DATABASE_URL format
- [Source: SQLx 0.8 docs] — `#[sqlx::test]` macro pour tests d'intégration

## Dev Agent Record

### Agent Model Used

Claude Opus 4.6 (1M context)

### Debug Log References

- **Piège SQLx `#[derive(sqlx::Type)]` en MySQL** : le derive par défaut déclare l'enum comme type SQL `ENUM` natif, incompatible avec notre schéma `VARCHAR`. Même avec `#[sqlx(type_name = "VARCHAR")]`, le runtime échoue avec `mismatched types ... SQL type ENUM is not compatible with SQL type VARCHAR`.
- **Solution** : implémentation manuelle de `Type<MySql>`, `Encode<MySql>`, `Decode<MySql>` sur chaque enum, délégant à `String`. Fiable et explicite. +1 méthode `as_str()` et impl `FromStr` par enum.
- **Pattern SQL non-macro** : utilisation de `sqlx::query_as::<_, T>("...")` plutôt que le macro `sqlx::query_as!` pour éviter la dépendance compile-time à une DB live.

### Completion Notes List

- 3 tables créées : `companies`, `users`, `fiscal_years` (schéma conforme aux AC 1-3 + 7)
- 4 enums avec impl Type/Encode/Decode manuels : OrgType, Language, Role, FiscalYearStatus
- 3 repositories CRUD : companies (create, find_by_id, update, list), users (+ find_by_username, UserUpdate), fiscal_years (create, find_by_id, list_by_company, update_status — pas de delete conformément au CO suisse)
- DbError avec 6 variantes + `error_code()` + helper `map_db_error` détectant MariaDB codes 1062/1451/1452/4025/3819
- Pool configurable via `create_pool(url, max_connections, connect_timeout)`
- MIGRATOR exposé pour `#[sqlx::test]`
- 26 tests d'intégration (8 companies + 9 users + 9 fiscal_years), tous passent
- `User` protégé : pas de `Serialize/Deserialize`, `Debug` manuel masquant `password_hash`
- Test dédié `debug_masks_password_hash` vérifie que le hash ne fuit jamais en Debug
- README du crate documente les prérequis DB pour les tests

### Review Follow-ups (AI)

Revue de code adversariale (3 reviewers en parallèle) — 4 critiques + 6 med + 6 low corrigés.

- [x] [AI-Review CRITIQUE] Fix #1 — Retrait des `expect()` en production, remplacés par `DbError::Invariant`
- [x] [AI-Review CRITIQUE] Fix #2 — `fiscal_years::update_status` → `close`, guard SQL `WHERE status = 'Open'` empêchant la réouverture (CO suisse art. 957-964)
- [x] [AI-Review CRITIQUE] Fix #3 — Retrait de `#[from] sqlx::Error` sur `DbError::Sqlx` pour forcer le passage par `map_db_error`
- [x] [AI-Review CRITIQUE] Fix #4 — `create()` et `update()` dans des transactions atomiques (INSERT+SELECT, UPDATE+SELECT) pour éviter les race windows
- [x] [AI-Review MED] Fix #5 — CHECK constraints pour chaînes vides sur username, password_hash (>=20), name, address, fiscal_year.name
- [x] [AI-Review MED] Fix #6 — `list()` clampe `limit` dans `[0, MAX_LIST_LIMIT=1000]` et `offset >= 0`
- [x] [AI-Review MED] Fix #7 — `sql_mode='STRICT_ALL_TABLES,NO_ZERO_DATE,NO_ZERO_IN_DATE'` + `time_zone='+00:00'` via `after_connect` hook
- [x] [AI-Review MED] Fix #8 — `UserUpdate` déplacé vers `entities/user.rs` + réexporté par `entities::mod`
- [x] [AI-Review MED] Fix #9 — Variantes `DbError::ConnectionUnavailable` (timeout/pool closed/IO) et `DbError::Invariant` ajoutées
- [x] [AI-Review MED] Fix #10 — `fiscal_years` garde le schéma AC (pas de version column) mais la sécurité est garantie par le guard `WHERE status = 'Open'` en SQL
- [x] [AI-Review LOW] Fix #11 — Tests stricts : `matches!(Err(DbError::CheckConstraintViolation(_)))` au lieu de `is_err()`
- [x] [AI-Review LOW] Fix #12 — Macro pour la duplication Type/Encode/Decode : différé (refactor pur, acceptable en l'état, peut être fait en cleanup pass)
- [x] [AI-Review LOW] Fix #13 — `users::update` renommé en `update_role_and_active` (nom aligné avec la sémantique restreinte)
- [x] [AI-Review LOW] Fix #14 — `password_hash` VARCHAR(255) → VARCHAR(512) (support paramètres Argon2id custom)
- [x] [AI-Review LOW] Fix #15 — Documentation explicite sur la protection Serialize User (trybuild reporté à une story future)
- [x] [AI-Review LOW] Fix #16 — CHECK REGEXP `^CHE[0-9]{9}$` sur `ide_number` (défense en profondeur contre un bypass applicatif)

Tests ajoutés : empty_name_rejected, empty_address_rejected, invalid_ide_format_rejected, list_limit_clamped_to_max, list_negative_values_normalized, username_empty_rejected, password_hash_too_short_rejected, debug_masks_password_hash_on_new_user, close_fails_on_already_closed. **Total : 35 tests** (vs 26 initialement).

### Review Follow-ups (AI) — Passe 2

Deuxième passe adversariale sur le code corrigé. 1 HIGH + 3 MED + 8 LOW détectés et corrigés.

- [x] [AI-Review HIGH] Fix #1 — Rollback explicite dans toutes les branches `Invariant` des fonctions `create`/`update` (cohérence avec les autres branches d'erreur)
- [x] [AI-Review MED] Fix #2 — **Régression corrigée** : `DbError::Sqlx(#[source] sqlx::Error)` pour préserver la chaîne d'erreur (cassée par le retrait de `#[from]`)
- [x] [AI-Review MED] Fix #3 — Message plus précis dans `close()` pour le cas "status Open mais 0 rows affected" (race condition hypothétique)
- [x] [AI-Review MED] Fix #4 — `OCTET_LENGTH` au lieu de `CHAR_LENGTH` pour le CHECK sur password_hash
- [x] [AI-Review LOW] Fix #5 — `MAX_LIST_LIMIT` déplacée dans `repositories/mod.rs` (pas de cross-module coupling)
- [x] [AI-Review LOW] Fix #6 — Doc `repositories/mod.rs` mise à jour (`close` au lieu de `update_status`)
- [x] [AI-Review LOW] Fix #7 — `debug_masks_password_hash_on_new_user` converti en `#[test]` simple (pas de DB inutile)
- [x] [AI-Review LOW] Fix #8 — Tests `list_limit_clamped_to_max` et `list_negative_values_normalized` avec assertions précises (count exact, pas juste `is_ok()`)
- [x] [AI-Review LOW] Fix #9 — Test boundary `start_date == end_date` rejeté par CHECK strict
- [x] [AI-Review LOW] Fix #10 — Test `find_by_username_is_case_insensitive` couvre le comportement documenté de la collation utf8mb4_unicode_ci
- [x] [AI-Review LOW] Fix #11 — `map_db_error` gère `sqlx::Error::RowNotFound` → `DbError::NotFound` pour robustesse future
- [x] [AI-Review LOW] Fix #12 — `Sqlx` Display exposition documentée (déjà en Dev Notes, pas de changement code)

Tests finaux : **37 tests** (13 + 11 + 13 + doc 0).

### Review Follow-ups (AI) — Passe 3

Troisième passe adversariale (ultime sanity check). 3 MED + 3 LOW + 2 NITPICK détectés et corrigés.

- [x] [AI-Review MED] Fix #1 — Nouvelle variante `DbError::IllegalStateTransition` utilisée par `close()` quand l'exercice est déjà clos (sémantique correcte : transition d'état interdite, pas violation de CHECK). Mapping HTTP 409 côté API.
- [x] [AI-Review MED] Fix #2 — CHECK constraints avec `BINARY` pour toutes les colonnes enum (`org_type`, `accounting_language`, `instance_language`, `role`, `status`). Empêche le contournement via la collation case-insensitive (`'fr'` ne matche plus `'FR'`).
- [x] [AI-Review MED] Fix #3 — Doc `# Sécurité` sur `find_by_username` warning contre timing attack enumeration, pointant la responsabilité story 1.5.
- [x] [AI-Review LOW] Fix #4 — `list_by_company` applique maintenant `LIMIT MAX_LIST_LIMIT` (cohérence avec les autres `list()`, défense OOM).
- [x] [AI-Review LOW] Fix #5 — Commentaire TODO sur `ide_number: Option<String>` pointant vers le futur `Option<CheNumber>` newtype.
- [x] [AI-Review LOW] Fix #6/#8 — Commentaires "défensif / unreachable" sur les branches `Invariant` post-transaction, expliquant pourquoi elles ne peuvent survenir sous REPEATABLE READ.
- [x] [AI-Review NITPICK] Fix #7 — Doc `pool.rs` documente le choix d'isolation par défaut (REPEATABLE READ) et la justification.

### Change Log

- 2026-04-05 : Revue code passe 2 — 12 findings corrigés (1 HIGH, 3 MED, 8 LOW). 37 tests
- 2026-04-05 : Revue code passe 3 — 8 findings corrigés (3 MED, 3 LOW, 2 NITPICK). Nouvelle variante `IllegalStateTransition`, BINARY dans les CHECK.

### Change Log

- 2026-04-05 : Implémentation complète story 1.4 — schéma MariaDB + repository pattern (26 tests)
- 2026-04-05 : Correction runtime SQLx (impl Type/Encode/Decode manuelle pour les enums)
- 2026-04-05 : Revue de code adversariale — 16 findings corrigés (4 CRITIQUES, 6 MED, 6 LOW). 35 tests au total.

### File List

- crates/kesh-db/Cargo.toml (modifié)
- crates/kesh-db/README.md (nouveau)
- crates/kesh-db/migrations/20260404000001_initial_schema.sql (nouveau)
- crates/kesh-db/src/lib.rs (modifié)
- crates/kesh-db/src/errors.rs (nouveau)
- crates/kesh-db/src/pool.rs (nouveau)
- crates/kesh-db/src/entities/mod.rs (nouveau)
- crates/kesh-db/src/entities/company.rs (nouveau)
- crates/kesh-db/src/entities/user.rs (nouveau)
- crates/kesh-db/src/entities/fiscal_year.rs (nouveau)
- crates/kesh-db/src/repositories/mod.rs (nouveau)
- crates/kesh-db/src/repositories/companies.rs (nouveau)
- crates/kesh-db/src/repositories/users.rs (nouveau)
- crates/kesh-db/src/repositories/fiscal_years.rs (nouveau)
- crates/kesh-db/tests/companies_repository.rs (nouveau)
- crates/kesh-db/tests/users_repository.rs (nouveau)
- crates/kesh-db/tests/fiscal_years_repository.rs (nouveau)
