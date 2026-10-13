# Prompt — validation P2 de la spec, Story 28-3 (la sauvegarde lit ses tables dans un instantané commun)

*Versionné le 2026-10-13. Deux lentilles, contexte frais chacune, en lecture seule. Rotation (D6, rétro 25) :
passes complètes Sonnet ↔ Opus ; Haiku réservé à une passe ciblée de fin de boucle. P1 était en Sonnet ; P2 :
**Opus** pour les deux lentilles (C-28-3-24). La remédiation de P1 touche AC1 à AC5, AC7, AC8, AC10, AC11, les
tâches et les mutations : trop large pour une passe ciblée.*

Dépôt `/home/gcorbaz/devel/kesh-28-3`, branche `story/28-3` (base `6f971d76`, = v0.13.2 + rétrospective de
l'Epic 27). Fiche : `_bmad-output/implementation-artifacts/28-3-sauvegarde-instantane-commun.md` (remédiée au commit
`f02df94c` ; état d'avant : `bf8b9624`). Le diff de la remédiation : `git diff bf8b9624 f02df94c -- _bmad-output/`.
Rapports de P1 : `/home/gcorbaz/devel/kesh-gate-logs/28-3-validate-p1-R.md` et `…-p1-F.md` (à lire **après** ta
propre lecture de la fiche, pour ne pas en hériter les angles). Epic : `epic-28-recette-v0-13-2.md` § *28-3* — à lire
dans `/home/gcorbaz/devel/kesh-28-0/_bmad-output/planning-artifacts/epic-28-recette-v0-13-2.md` (lecture seule) s'il
n'est pas encore sur la branche. Issues #633 et #465 (`gh api repos/guycorbaz/kesh/issues/633`, `…/465`, et leurs
`/comments`). Registre : `_bmad-output/implementation-artifacts/epic-28-choix-autonomes-28-3.md` (C-28-3-1 à 24 ;
C-28-3-15 à 23 sont ceux de la remédiation).
Règles : `CLAUDE.md` (§ Review Iteration Rule et ses sous-sections — le manuel nommé, les axes déclarés, les sites
NON RÉSOLUS, la propagation post-patch, le recompte des comptes rendus ; § Migration breaking policy, dont
l'exception `kesh-db` au gate ciblé ; § Règle de splitting préventif ; § Issue Tracking Rule).
Sources externes utiles, **en lecture** : sqlx 0.8.6 sous `~/.cargo/registry/src/*/sqlx-core-0.8.6/` et
`…/sqlx-mysql-0.8.6/` (`src/transaction.rs`, `src/pool/mod.rs`, `src/pool/connection.rs`, `src/testing/mod.rs`,
`src/error.rs`, `src/connection/executor.rs`).

## Axe premier (commun aux deux lentilles) — les régressions de la remédiation

*La sévérité ne stagne pas, elle se déplace vers ce qu'on vient d'écrire* (CLAUDE.md § La passe ciblée). Relis
d'abord, ligne à ligne, le diff `bf8b9624..f02df94c`, et cherche ce que la remédiation a **cassé ou laissé
incohérent** — au moins :

1. **`ReadSnapshot` dégradé à T1 (C-28-3-16)** : le même texte de test rougit-il vraiment à T1 et verdit-il à T2 sans
   modification ? (a-défaut), (a-RC) et (b) rougissent-ils tous trois sous l'intérieur dégradé (autocommit sur une
   connexion) ? T1 compile-t-il le **workspace** (adaptation de `kesh-api` et des quatre tests avancée de T3 à T1) ?
   La preuve `git diff` du module de test est-elle exécutable et discriminante ? T3 a-t-il encore un contenu cohérent ?
2. **Gardes lexicales (AC2 point 3, AC5 (g))** : la règle « `\bpool\b` exactement deux fois » est-elle juste sur le
   code **prescrit** de `export_installation` et de `build_keshbackup` (paramètre, unique appel ; une variable locale
   nommée `pool`, un `pool` dans une chaîne, un `pool.` ailleurs ?) ; la délimitation du corps (de `pub async fn …(`
   à la première `}` en colonne 0) et l'écart des lignes `//` tiennent-ils face à `rustfmt` ? Tuent-elles M10 et M11
   **sans** rougir sur le code correct ? L'angle mort écrit (lecture par un appelant passée en paramètre) est-il le
   seul ? La phrase « le cœur ne reçoit aucun pool » est-elle tenue par l'observateur du test (qui capture un pool) ?
3. **Garde de moteur à l'exécution (AC4, C-28-3-17)** : requête, filtre, `Option<String>`, compte attendu
   `TABLES_TO_TRUNCATE.len() + 1`, échec fermé ; interaction avec (b) (suppression à `Ouvert`, avant la garde), avec
   les erreurs de l'AC3 (`map_db`, `AdminFullExportFailed`, `AdminPreImportBackupFailed` — AC7), avec les suites de
   l'AC6, avec tout test existant qui exporterait une base où une table manquerait (`TABLES_TO_TRUNCATE` contre le
   squash, `_kesh_version` présente ?). (e2) est-il réalisable (`ALTER TABLE _kesh_version ENGINE=MyISAM` dans un
   `#[sqlx::test]`, contrainte `CHECK`) — **non vérifié** si cela exige une requête. Le décompte « 81 requêtes » est-il
   juste ?
4. **Ouverture (AC1, C-28-3-19, C-28-3-20)** : `conn.execute("SET TRANSACTION …")` sur un `&str` part-il bien en
   protocole texte (`executor.rs`) ? `PoolConnection::close_on_drop` existe-t-il en 0.8.6 et fait-il ce que la fiche
   dit ? Le résidu sur échec de `Transaction::begin` est-il correctement décrit ? La propagation de l'erreur de
   `finish` est-elle compatible avec (d2) et avec le `Drop` ?
5. **Observateur (AC4, C-28-3-21)** : l'alias local `BoxFuture` sans crate `futures` est-il exprimable tel qu'écrit
   (`'a` contre `'static`) ; l'observateur vide `&mut |_| Box::pin(async {})` type-t-il vers le `dyn FnMut` prescrit ;
   le futur du handler reste-t-il `Send` ?
6. **Tests (c), (d)** : le chemin `as_database_error()` → `try_downcast_ref::<MySqlDatabaseError>()` → `number()`
   existe-t-il ; le repli `INNODB_TRX` (C-28-3-22) est-il discriminant sous M6 avec la lecture ajoutée à (d1)/(d2) ?
7. **Manuel (AC10, C-28-3-18)** : la phrase de remplacement est-elle vraie au code (l'audit `admin.full_export` est
   best-effort, après le fichier ; la sauvegarde **pré-import** écrit-elle, elle aussi, quelque chose — audit,
   fichier dans `KESH_ADMIN_BACKUP_DIR` — que le manuel devrait dire ou ne pas contredire) ? La phrase sur le moteur
   non InnoDB est-elle vraie ? Les contrôles aplatis prescrits (`grep -c "aucun effet de bord"` → 0) sont-ils justes
   sur le `.tex` **et** sur le PDF, et ne rougiraient-ils pas sur une autre occurrence légitime ?
8. **Propagation (AC8)** : les deux greps prescrits rendent-ils, au sol, exactement les sites que la fiche annonce ?
   Refais-les. Le symptôme corrigé (« tenu par le compilateur », « non représentable », « octet pour octet », « API
   publique », « 41 requêtes », « au choix du dev », « aucun effet de bord ») survit-il quelque part dans la fiche, le
   registre, le prompt P1 (historique, légitime) ou ailleurs ?
9. **Recomptes** : 12 AC, T0-T6, M1-M12 (chaque mutation tuée par le test annoncé, et ce test **existe** dans la
   fiche), C-28-3-1 à 24, 41/80/81 requêtes, 39 tables, 40/40, 51/38, 13 tables sans `ENGINE`, chaque numéro de ligne
   neuf (`routes/admin.rs:44`, `:653-692`, `:936-943`, `manifest.rs:55`, `backup.rs:487-488`, `test_schema_guard.rs:64`,
   `:166`, `error.rs:15-17`, `:24-26`, `admin-manual.tex:1825`, `:1897`). Le Change Log de la remédiation est-il juste
   (19 bruts, 15 distincts, 3 MEDIUM, 16 LOW) ?

## Axes de P1, à reprendre (communs aux deux lentilles)

1. **Inventaire clos des lectures** (Dev Notes § *Inventaire clos*) : refais les `grep` copiés dans la fiche, et
   **élargis-les** (un détecteur part du symptôme, pas d'une liste) — toute lecture de la base faite pendant la
   construction d'un `.keshbackup`, sous toutes formes : `sqlx::query*`, `query_scalar`, `QueryBuilder`, `format!`
   de SQL, appels de dépôt, `INFORMATION_SCHEMA`, lectures dans `manifest.rs`, dans le handler `full_export` et dans
   `run_backup_and_restore` **entre** l'ouverture et la fin de la sauvegarde, `kesh-seed`, `test_endpoints.rs`, tout
   autre producteur de `.keshbackup`. Une lecture de la sauvegarde qui échapperait au `ReadSnapshot` est au moins
   **HIGH**. La formule « aucun site non résolu dans les formes couvertes » est-elle exacte ?
2. **Manuel contre code** (obligatoire) : `docs/manual/fr/admin-manual.tex` § *Exporter (sauvegarde complète)*
   (l. 1823 et suivantes), § *Importer* et la sauvegarde de sécurité pré-import, § *Quelle méthode de sauvegarde
   choisir ?* (l. 1901), variables `KESH_ADMIN_*` (l. 1895-1898) ; `docs/manual/fr/user-manual.tex` § *Exports de
   souveraineté*. Ce que les manuels disent **aujourd'hui** est-il vrai au code ? Ce que l'AC10 prescrit d'écrire
   sera-t-il vrai au code prescrit ? Contrôle aussi les **PDF**, aplatis :
   `pdftotext docs/manual/fr/admin-manual.pdf - | tr '\n' ' ' | tr -s ' '`. Une promesse que le code ne tiendrait pas
   est au moins MEDIUM.
3. **Couverture** (lentille R surtout) : chaque phrase de #633 et du § *28-3* de l'epic par un AC testable ; chaque
   AC par une tâche ; chaque AC par un test qui rougirait ; gardes existantes (`avant_sauvegarde_branchee_aux_appels_de_l_import`,
   `test_schema_guard.rs`, `textes_coherents.rs`, `admin_full_export_e2e.rs`, `admin_full_import_e2e.rs`,
   `admin_backup_e2e.rs`) ; séparation de #465 (C-28-3-1) contre la règle de splitting et le § *Priorités des
   défauts* ; mots-clés de PR (`closes #633`, `refs #465`).
4. **Sémantique MariaDB/sqlx et concurrence** (lentille F surtout) : mécanisme de l'AC1 sous MariaDB 10.11 et
   sqlx 0.8.6 ; `ReadSnapshot` (`'static`, `Send`, accès `&mut MySqlConnection`) ; déterminisme de chaque test de
   l'AC5 ; chaque mutation M1-M12 tuée ; croisement avec **chaque** écrivain concurrent (saisies, suppression
   d'import, restauration, remise à zéro `kesh-seed`, création paresseuse de `company_invoice_settings`) et avec le
   `FOR UPDATE` du pré-import : interblocage, attente, vue antérieure au verrou ; DDL (C-28-3-8), durée, purge, pool,
   mémoire (C-28-3-5 et l'angle mort écrit).

## Lentilles

- **R — Relecture contre le code au sol (auditeur d'acceptation)** : `.claude/skills/bmad-create-story/checklist.md` ;
  axe premier, puis axes 1 à 3 de P1 ; chaque fait, chemin et numéro de ligne cité par la fiche **revérifié** ;
  textes prescrits (manuels, CHANGELOG, commentaires) confrontés au code prescrit.
- **F — Faisabilité, sémantique MariaDB/sqlx et tests (adversaire de la conception)** : axe premier (points 1 à 6
  en priorité), puis axe 4 de P1 ; lis le code de sqlx 0.8.6 pour chaque appel que la fiche prescrit, au lieu de
  t'en remettre à ce que la fiche ou les rapports de P1 en disent.

## Ce que tu rends

Findings numérotés (`R2-…` ou `F2-…`) avec sévérité **CRITICAL / HIGH / MEDIUM / LOW**, `fichier:ligne`,
**preuve** (commande exécutée et sortie copiée), correction proposée ; pour chaque finding, dire s'il est **né de la
remédiation** (diff `bf8b9624..f02df94c`) ou **d'origine**. Toute affirmation de présence ou d'absence = sortie de
`grep -nF` copiée. Un fait MariaDB que tu ne peux établir qu'en exécutant une requête — interdit ici — se signale
comme **non vérifié**, avec le test de la fiche qui le trancherait ; il ne se tranche pas de mémoire.
⛔ **La liste des axes exercés ET non exercés** — un « 0 finding » sans elle ne compte pas comme passe. Rapport dans
`/home/gcorbaz/devel/kesh-gate-logs/28-3-validate-p2-R.md` (lentille R) ou
`/home/gcorbaz/devel/kesh-gate-logs/28-3-validate-p2-F.md` (lentille F) ; dernier message : le chemin du rapport, le
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
