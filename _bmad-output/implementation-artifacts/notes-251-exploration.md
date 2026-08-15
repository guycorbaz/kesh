# Notes d'exploration — issue #251 (suite de tests lente)

*Exploration ground-truth du 2026-08-14/15, lecture seule (sources sqlx 0.8.6 du registry,
kesh-db, scripts, compose, CI). Matière première de la future spec — l'engagement de Guy
est au sprint-status. Chaque affirmation porte son fichier:ligne dans le rapport source ;
l'essentiel décisionnel est ici.*

## Ce que sqlx 0.8.6 NE permet PAS (contraintes dures, vérifiées dans les sources)

1. `#[sqlx::test]` fait `CREATE DATABASE` **vide** puis `migrator.run_direct` — aucun
   clonage, aucun hook, aucune variable d'env pour fournir un schéma pré-migré
   (`sqlx-mysql-0.8.6/src/testing/mod.rs:169`, `sqlx-core-0.8.6/src/testing/mod.rs:245-274`).
2. `TestSupport` est un impl unique global pour `MySql` — non surchargeable sans fork.
3. `snapshot()`/fixtures capturées : `todo!()` pour MySQL.
4. MariaDB n'a NI `CREATE DATABASE … TEMPLATE` (Postgres) NI plugin `CLONE` (MySQL 8).
5. Les pools de test n'ont pas d'`after_connect` : la durabilité relâchée doit être un
   flag SERVEUR (command du conteneur), pas une SET SESSION.
6. Seuls points d'extension per-test : `migrator = "<path Rust>"`, `migrations = "<dir>"|false`,
   `fixtures(...)` (rejouées APRÈS le migrator — n'évite rien).

## Le dessin qui en découle (volet squash)

Pas de clonage possible → **le template devient un MIGRATOR DE SQUASH** :

- générer `schema-squash.sql` = dump DDL (sans données) d'une base entièrement migrée ;
- exposer dans kesh-db un `TEST_MIGRATOR` : un `Migrator` à UNE migration portant ce dump
  (le patron `Migrator { migrations: Cow<[Migration]> … }` est déjà exercé deux fois dans
  le dépôt — `migrations_upgrade_path.rs`, `tests/common/mod.rs`) ;
- basculer les attributs `migrator = "kesh_db::MIGRATOR"` des tests éphémères vers
  `TEST_MIGRATOR` (1092 sites, substitution mécanique) — 1 exécution DDL par test au lieu
  du cycle INSERT/DDL/UPDATE × 61 ;
- **garde-fou anti-dérive OBLIGATOIRE** : un test qui monte les DEUX migrators sur deux
  bases éphémères et diffe `information_schema` (tables, colonnes, index, contraintes,
  collations) — le squash périmé doit rougir, jamais dériver en silence. Script de
  régénération du dump fourni (mysqldump --no-data) + consigne P8-like : le squash se
  REGÉNÈRE, ne s'édite pas.
- Les tests qui ont BESOIN du chemin réel des migrations (upgrade_path, fresh_install,
  backfills à fenêtre, triage P7) restent sur `kesh_db::MIGRATOR` — liste à figer en spec.

## Volet tmpfs / durabilité (dev seulement en l'état)

- `docker-compose.dev.yml` : volume nommé classique, AUCUN `command:` — tout est à poser :
  `tmpfs: /var/lib/mysql` + `command: --innodb_flush_log_at_trx_commit=0 --sync_binlog=0
  --innodb-doublewrite=0` (base JETABLE uniquement — la base dev `kesh` y perd sa
  persistance au restart : à documenter, seed à rejouer).
- Précédent : Guy a déjà monté un MariaDB tmpfs manuel port 3307 pour la 14-3a
  (`14-3a-socle-roles-comptes.md:266,485,526`) — jamais formalisé.
- **CI** : `services:` GitHub Actions ne passe pas de `command:` mariadbd — MAIS `options:`
  va à `docker create`, qui accepte `--tmpfs` : **à vérifier en spec** (le rapport
  d'exploration le classait contrainte dure ; `--tmpfs /var/lib/mysql` via options est
  plausiblement possible). CI n'utilise par ailleurs PAS nextest (`cargo test -j1
  --test-threads=1`) — tout gain CI passe par le squash, pas par les threads.

## Chiffres frais (recomptés, périment ceux de nextest.toml)

- 1142 `#[sqlx::test]` (718 kesh-api, 393 kesh-db, 31 kesh-report), dont 1092 `migrator=`,
  17 `migrations = false` ; 0 `fixtures(...)`.
- 154 `#[tokio::test]` sur base PARTAGÉE (sérialisés par `shared-db-serial`) — le « 84 »
  du commentaire CI est périmé ; 47 `#[tokio::test]` sans I/O.
- `.config/nextest.toml` cite encore « ~894 tests / 51 migrations » (2026-07-13) — périmé,
  à rafraîchir dans la story.
- Un test qui PANIQUE laisse sa base orpheline (`cleanup` seulement si succès) — ménage au
  run suivant, comportement sqlx existant.

## Gains attendus (à MESURER, pas à croire)

- Squash : le coût dominant (cycle de 61 migrations par base) devient 1 batch DDL.
- tmpfs+durabilité : accélère le DDL restant ET peut débloquer le plafond de 6 threads
  (posé pour contention DDL — re-mesurer avant de le bouger).
- Cible évoquée en discussion : ~40 min → 10-15 min. La story devra publier ses mesures
  avant/après sur la même machine.
