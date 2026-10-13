# Prompt — validation P1 de la spec, Story 28-3 (la sauvegarde lit ses tables dans un instantané commun)

*Versionné le 2026-10-13. Deux lentilles, contexte frais chacune, en lecture seule. Rotation (D6, rétro 25) :
passes complètes Sonnet ↔ Opus ; Haiku réservé à une passe ciblée de fin de boucle. P1 : **Sonnet** pour les deux
lentilles (la fiche a été écrite par Opus).*

Dépôt `/home/gcorbaz/devel/kesh-28-3`, branche `story/28-3` (base `6f971d76`, = v0.13.2 + rétrospective de
l'Epic 27). Fiche : `_bmad-output/implementation-artifacts/28-3-sauvegarde-instantane-commun.md` (commit
`bf8b9624`). Epic : `epic-28-recette-v0-13-2.md` § *28-3* — **pas encore mergé** : à lire dans le worktree du
kickoff, `/home/gcorbaz/devel/kesh-28-0/_bmad-output/planning-artifacts/epic-28-recette-v0-13-2.md` (lecture seule).
Issues #633 et #465 (`gh api repos/guycorbaz/kesh/issues/633`, `…/465`, et leurs `/comments`). Registre :
`_bmad-output/implementation-artifacts/epic-28-choix-autonomes-28-3.md` (C-28-3-1 à 14).
Règles : `CLAUDE.md` (§ Review Iteration Rule et ses sous-sections — le manuel nommé, les axes déclarés, les sites
NON RÉSOLUS, la propagation post-patch, le recompte des comptes rendus ; § Migration breaking policy, dont
l'exception `kesh-db` au gate ciblé ; § Règle de splitting préventif ; § Issue Tracking Rule).
Sources externes utiles, **en lecture** : sqlx 0.8.6 sous `~/.cargo/registry/src/*/sqlx-core-0.8.6/` et
`…/sqlx-mysql-0.8.6/` (`src/transaction.rs`, `src/pool/mod.rs`, `src/testing/mod.rs`).

⚠️ La fiche **sépare #465** de la story (C-28-3-1) et le motive par un décompte (29 lectures, 24 modules). Vérifie le
décompte et le raisonnement toi-même avant d'en tirer quoi que ce soit — dans un sens comme dans l'autre.

## Axes prioritaires (communs aux deux lentilles)

1. **Inventaire clos des lectures** (Dev Notes § *Inventaire clos*) : refais les `grep` copiés dans la fiche, et
   **élargis-les** (un détecteur part du symptôme, pas d'une liste) — toute lecture de la base faite pendant la
   construction d'un `.keshbackup`, sous toutes formes : `sqlx::query*`, `query_scalar`, `QueryBuilder`, `format!`
   de SQL, appels de dépôt, `INFORMATION_SCHEMA`, lectures dans `manifest.rs`, dans le handler `full_export` et dans
   `run_backup_and_restore` **entre** l'ouverture et la fin de la sauvegarde, `kesh-seed`, `test_endpoints.rs`, tout
   autre producteur de `.keshbackup` éventuel. Une lecture de la sauvegarde qui échapperait au `ReadSnapshot` est au
   moins **HIGH**.
2. **Manuel contre code** (obligatoire) : `docs/manual/fr/admin-manual.tex` § *Exporter (sauvegarde complète)*
   (l. 1823 et suivantes), § *Importer* et la sauvegarde de sécurité pré-import (l. 1831-1900), § *Quelle méthode de
   sauvegarde choisir ?* (l. 1901) ; `docs/manual/fr/user-manual.tex` § *Exports de souveraineté* (l. 2519-2600). Ce
   que les manuels disent **aujourd'hui** est-il vrai au code ? Ce que l'AC10 prescrit d'écrire sera-t-il vrai au code
   que la fiche prescrit — y compris « ne bloque aucune saisie » et « la sauvegarde de sécurité est lue de la même
   façon » ? Contrôle aussi les **PDF**, aplatis : `pdftotext docs/manual/fr/admin-manual.pdf - | tr '\n' ' ' | tr -s ' '`.
   Une promesse que le code ne tiendrait pas est au moins MEDIUM.
3. **Recomptes** : chaque nombre de la fiche et du registre se recompte depuis la source — 41 requêtes et 80
   requêtes, 39 tables, 40 tables InnoDB, 51/38 `CREATE TABLE`/`ENGINE=InnoDB` des migrations, 29 lectures et 24
   modules de l'export global, 12 AC, T0-T6, M1-M10, C-28-3-1 à 14, chaque numéro de ligne cité (`sed -n`, `grep -nF`).

## Lentilles

- **R — Relecture contre le code au sol (auditeur d'acceptation)** : `.claude/skills/bmad-create-story/checklist.md` ;
  chaque phrase de l'issue #633 et du § *28-3* de l'epic (dont « effet sur la durée de verrouillage et sur le
  streaming », « trancher le sort de #465 », « comment prouver l'instantané par un test ») couverte par un AC
  testable ; chaque AC par une tâche ; chaque AC par un test **qui rougirait** ; chaque fait, chemin et numéro de
  ligne cité par la fiche **revérifié** ; les textes prescrits (manuels, CHANGELOG, commentaires) confrontés à ce que
  le code prescrit fera réellement ; les gardes existantes qui changeront de sens ou de compte — la garde lexicale
  `avant_sauvegarde_branchee_aux_appels_de_l_import` (`routes/admin.rs`), `test_schema_guard.rs` (un test
  `#[sqlx::test]` neuf sur le squash doit-il y être inscrit ?), `textes_coherents.rs`, `admin_full_export_e2e.rs`,
  `admin_full_import_e2e.rs`, `admin_backup_e2e.rs` — la fiche les nomme-t-elle toutes ? La séparation de #465
  (C-28-3-1) respecte-t-elle la règle de splitting et le § *Priorités des défauts* (la règle du décompte des bugs de
  l'epic) ? Les mots-clés de la PR (`closes #633`, `refs #465`) sont-ils justes ?
- **F — Faisabilité, sémantique MariaDB/sqlx et tests (adversaire de la conception)** : le mécanisme de l'AC1
  est-il **correct sous MariaDB 10.11 et sqlx 0.8.6** — syntaxe `START TRANSACTION WITH CONSISTENT SNAPSHOT,
  READ ONLY`, portée de `SET TRANSACTION` sans `SESSION`, comportement sous `READ COMMITTED`, `Transaction::begin`
  sur une `PoolConnection` (visibilité publique, contrôle `in_transaction()`, `Drop` et `start_rollback`), protocole
  texte ou préparé pour ces instructions ? Le type `ReadSnapshot` de l'AC2 est-il réalisable (durée de vie
  `'static`, accès à l'exécuteur `&mut MySqlConnection`, `Send` pour un `axum` handler) ? L'observateur de l'AC4
  est-il exprimable sans `feature` ni coût en production, et le cœur reste-t-il le chemin réellement emprunté ? Les
  tests de l'AC5 sont-ils **déterministes** et font-ils rougir les mutations annoncées — en particulier : (a) rougit-il
  vraiment sur le code actuel (ordre de `TABLES_TO_TRUNCATE`, cascade `fk_bank_transactions_import`, l'écrivain
  `delete_unreconciled_in_tx` et ses verrous `FOR UPDATE` contre la lecture cohérente — peut-il **pendre**) ; (b) tue-t-il
  M2 ; (a-RC) tue-t-il M3 ; le pool d'une connexion de (d)/(f) est-il constructible depuis le pool de `#[sqlx::test]`
  et `@@in_transaction` / `@@tx_isolation` disent-ils ce que la fiche affirme ; M6 et M10 sont-ils bien posés ?
  Croise l'instantané avec **chaque** écrivain concurrent possible (saisies, suppression d'import, restauration,
  remise à zéro `kesh-seed`, création paresseuse de `company_invoice_settings` déclenchée par un écran) et avec le
  verrou `FOR UPDATE` du pré-import : un interblocage, une attente ou une vue antérieure au verrou sont-ils possibles ?
  Le raisonnement DDL (C-28-3-8), durée et purge, pool (5 connexions) et mémoire (C-28-3-5) tient-il ? La restauration
  est-elle vraiment hors de cause (C-28-3-10) ?

## Ce que tu rends

Findings numérotés (`R1-…` ou `F1-…`) avec sévérité **CRITICAL / HIGH / MEDIUM / LOW**, `fichier:ligne`,
**preuve** (commande exécutée et sortie copiée), correction proposée. Toute affirmation de présence ou d'absence =
sortie de `grep -nF` copiée. Un fait MariaDB que tu ne peux établir qu'en exécutant une requête — interdit ici — se
signale comme **non vérifié**, avec le test de la fiche qui le trancherait ; il ne se tranche pas de mémoire.
⛔ **La liste des axes exercés ET non exercés** — un « 0 finding » sans elle ne compte pas comme passe. Rapport dans
`/home/gcorbaz/devel/kesh-gate-logs/28-3-validate-p1-R.md` (lentille R) ou
`/home/gcorbaz/devel/kesh-gate-logs/28-3-validate-p1-F.md` (lentille F) ; dernier message : le chemin du rapport, le
bilan par sévérité, une ligne par finding MEDIUM ou plus.

## Interdits

⛔ N'écris aucun fichier hors `/home/gcorbaz/devel/kesh-gate-logs/`. Aucune commande qui écrit, compile ou
exécute : `scripts/*` (dont **`scripts/prepare-release.sh`**, `scripts/test-fast.sh`, `scripts/mem-guard.sh`,
`scripts/regen-test-schema.sh` et les scripts Synology), `make`, `latexmk`, `xelatex`, `git commit`/`add`/`checkout`/
`switch`/`stash`/`fetch`/`pull`/`reset`/`rebase`/`tag`/`push`, `cargo`, `rustc`, `npm`, `npx`, `node`, `docker`,
**`mariadb`, `mysql`, `mariadb-dump`** et toute requête SQL, **tout interpréteur** (`python`, `python3`, `perl`,
`ruby`, `bash`/`sh` sur un script, `awk` qui écrirait un fichier), `gh` en écriture. Rien de `~/kesh-test/` (relevé
réel de Guy) ne se lit ni ne se cite. Autorisés : lecture, `grep`, `sed -n`, `wc`, `sort`, `git log`/`show`/`diff`/
`blame`, `gh api` en lecture, `pdftotext` vers la sortie standard.
