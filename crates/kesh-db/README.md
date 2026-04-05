# kesh-db

Couche de persistance MariaDB de Kesh. Schéma, entités, repository pattern.

## Structure

```
src/
├── lib.rs           # Expose les modules + MIGRATOR
├── errors.rs        # DbError enum + map_db_error helper
├── pool.rs          # create_pool(url, max_conn, timeout)
├── entities/        # Structs de données (Company, User, FiscalYear, ...)
└── repositories/    # CRUD par entité (companies, users, fiscal_years)
migrations/          # Migrations SQLx (sqlx migrate run)
tests/               # Tests d'intégration (#[sqlx::test])
```

## Tests d'intégration

Les tests utilisent `#[sqlx::test(migrator = "kesh_db::MIGRATOR")]` qui crée
une **base de données temporaire par test** (ce n'est pas un rollback de
transaction). La base est clonée puis détruite automatiquement.

### Prérequis

1. MariaDB en cours d'exécution :
   ```bash
   docker compose -f docker-compose.dev.yml up -d mariadb
   ```

2. L'utilisateur configuré dans `DATABASE_URL` doit avoir les droits
   `CREATE DATABASE` et `DROP DATABASE` (nécessaire pour `#[sqlx::test]`).
   Dans le container de dev, exécuter une fois :
   ```bash
   docker exec kesh-mariadb-dev mariadb -u root -pkesh_dev_root \
     -e "GRANT ALL PRIVILEGES ON *.* TO 'kesh'@'%' WITH GRANT OPTION; FLUSH PRIVILEGES;"
   ```

3. `DATABASE_URL` configurée :
   ```bash
   export DATABASE_URL='mysql://kesh:kesh_dev@127.0.0.1:3306/kesh'
   ```

4. Appliquer la migration initiale (une fois, pour créer le schéma cible) :
   ```bash
   docker exec -i kesh-mariadb-dev mariadb -u kesh -pkesh_dev kesh \
     < crates/kesh-db/migrations/20260404000001_initial_schema.sql
   ```

### Lancer les tests

```bash
DATABASE_URL='mysql://kesh:kesh_dev@127.0.0.1:3306/kesh' cargo test -p kesh-db
```

### Flakiness connue — `#[sqlx::test]` en mode workspace parallèle

Sur MySQL/MariaDB, `#[sqlx::test]` crée une **base de données temporaire
par test** via un meta-système (`_sqlx_test`). Quand `cargo test --workspace`
lance plusieurs binaires de tests en parallèle (unit tests de `kesh-db`,
tests d'intégration `kesh-db/tests/*.rs`, tests de `kesh-api`, E2E
`kesh-api/tests/auth_e2e.rs`), tous ces binaires partagent le même meta-système
SQLx et peuvent entrer en contention sur la création/destruction des DBs
temporaires.

**Symptôme observé** : 1 échec intermittent sur ~4 runs de
`cargo test --workspace` (reproductible < 25 % du temps), typiquement
sur un test de bootstrap ou d'un autre test qui fait `SELECT COUNT(*)`
puis lit/écrit immédiatement. Les runs isolés
(`cargo test -p kesh-db` ou `cargo test -p kesh-api`) ne montrent
**jamais** cette flakiness.

**Contournement** si le flake bloque un CI :

```bash
# Sérialiser la création/exécution des test binaries
DATABASE_URL='...' cargo test --workspace -- --test-threads=1
# OU limiter le parallélisme côté cargo lui-même
DATABASE_URL='...' cargo test --workspace -j1
```

**Ne pas** chercher à corriger le code applicatif sur cette base — c'est
un problème connu du test runner `sqlx::test` sur MySQL, pas un bug
produit. À réévaluer si SQLx ajoute un support transactionnel pour
MySQL (comme sur Postgres).

## Types enum stockés en VARCHAR

Les enums (`OrgType`, `Language`, `Role`, `FiscalYearStatus`) implémentent
manuellement `sqlx::Type<MySql>`, `Encode<MySql>` et `Decode<MySql>` en
déléguant à `String`. Le derive `sqlx::Type` par défaut traite les enums
string-backed comme colonne `ENUM` native MySQL, incompatible avec notre
schéma qui utilise `VARCHAR` + `CHECK` constraints pour la portabilité.

## Verrouillage optimiste

Chaque entité modifiable possède un champ `version: i32`. Les fonctions
`update()` comparent la version fournie avec celle en base :

- Version match → UPDATE réussit, version incrémentée
- Version mismatch → `DbError::OptimisticLockConflict` (409 côté API)
- Entité absente → `DbError::NotFound` (404 côté API)
