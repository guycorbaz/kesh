# Prompt — revue de code P1, Story 28-3 (la sauvegarde d'installation lit ses tables dans un instantané commun)

*Versionné le 2026-10-13. Trois lentilles (Sonnet), contexte frais chacune, lecture seule.*

Dépôt (worktree) `/home/gcorbaz/devel/kesh-28-3`, branche `story/28-3`. **Le diff à revoir** :
`git diff a4da521b 90c85099 -- . ':(exclude)_bmad-output'` (`a4da521b` = validation close de la fiche ; `90c85099` =
dernier commit de code ; les commits suivants ne portent que la fiche, le registre et ce prompt). ⚠️ Pathspec
`':(exclude)…'` et **non** `':!…'`, que le shell peut interpréter. Commits intermédiaires : `a914762b` (T1 — instantané
**dégradé**, tests rouges), `1894bf0c` (T2 — instantané réel), `c00a7f63` (T3-T4), `90c85099` (T5 — manuels, CHANGELOG,
garde G18). Fiche : `_bmad-output/implementation-artifacts/28-3-sauvegarde-instantane-commun.md` — AC1 à AC12, Tâches,
Dev Notes (inventaire clos des lectures, moteurs de stockage, DDL/durée/purge/pool, risque de montée de sqlx, faits
MariaDB), et le **Dev Agent Record** (rouge de T1 copié, preuve `git log -L` et `git diff -U0`, mutations M1-M11, faits
MariaDB tranchés, gates). Choix C-28-3-1 à 39 au registre `epic-28-choix-autonomes.md` (C-28-3-33 à 39 : développement).
Règles : `CLAUDE.md` du dépôt — § « Recompter ses propres comptes rendus », § « Propagation post-patch », § « Le prompt
d'une passe doit NOMMER le manuel », § « Une passe qui ne déclare pas ses axes exercés ne compte pas », § « Migration
breaking policy » (aucune migration attendue, AC12). Issues : #633 (la PR la fermera) ; #465 (référencée, hors
périmètre : l'export global CSV reste sans instantané).

⛔ **DONNÉES PERSONNELLES — DÉPÔT PUBLIC.** N'ouvre aucun fichier hors du dépôt (rien sous `~/kesh-test/`) ; les
données des tests sont synthétiques.

**Contexte** : story **backend et documentation**, aucun écran. `kesh-db` : module neuf `snapshot.rs`
(`ReadSnapshot` : `begin`, `begin_on` `pub(crate)`, `conn`, `finish` ; `SET TRANSACTION ISOLATION LEVEL REPEATABLE
READ` en protocole texte puis `sqlx::Transaction::begin(conn, Some("START TRANSACTION WITH CONSISTENT SNAPSHOT, READ
ONLY"))`, `close_on_drop` sur échec du `SET`, `ROLLBACK` de `finish`) ; `backup.rs` : les quatre lectures de la
sauvegarde sur `&mut ReadSnapshot`, `InstallationExport`, `export_installation`, cœur privé `export_installation_in`
avec observateur (`Etape`, alias `BoxFuture`), tests (a-défaut), (a-RC), (b), (c), (d1), (d2), (e), (f), (g)(i), (h)
et leurs helpers, adaptation de trois tests existants ; `tests/accounts_role_backfill.rs` adapté ;
`repositories/bank_imports.rs` (commentaire). `kesh-api` : `admin_backup/export.rs` (`build_keshbackup` sur
`export_installation`, en-tête, `mod tests` neuf : garde (g)(ii)) ; `routes/admin.rs` (commentaire du pré-import) ;
`exports/global.rs` (commentaire) ; `tests/textes_coherents.rs` (G18 : fragment assumé, total 16 → 17). Textes :
manuels d'administration et utilisateur FR (`.tex` **et PDF**), `CHANGELOG.md` (`## [0.13.3] — Non publié`).

⛔ **Axes communs aux trois lentilles** (chacune les déclare exercés ou non) :
1. **L'instantané, et lui seul.** Toute lecture de la sauvegarde passe-t-elle par l'instantané ? Refais l'inventaire
   clos au `HEAD` (`grep -n "fetch_\|execute(" crates/kesh-db/src/backup.rs`, `grep -rn "build_keshbackup\|export_table(\|read_min_required\|read_instance_id\|export_installation" crates --include=*.rs`) ;
   un site neuf qui lit sur le pool dans le chemin de la sauvegarde ? Les gardes lexicales (g)(i) et (g)(ii) : leur
   découpe du corps (première ligne `#[cfg(test)]`, lignes `//`, première ligne `}`) voit-elle **toute** la fonction
   et **rien d'autre** ? Un commentaire en fin de ligne contenant `pool`, une chaîne, une macro les feraient-ils
   mentir ? Les angles morts écrits à l'AC2 (alias du pool, lecture d'un appelant) sont-ils les seuls ?
2. **La concurrence et le pool.** Ouverture (`SET` puis `START` sur la **même** connexion, `Transaction::begin`
   `#[doc(hidden)]`), clôture (`finish` → `ROLLBACK`, erreur propagée par `export_installation`), abandon (`Drop` de
   `sqlx::Transaction` sur toute erreur après l'ouverture), échec du `SET` (`close_on_drop`), échec du `START`
   (résidu assumé, C-28-3-19/30). La connexion rendue au pool est-elle **toujours** hors transaction et avec le niveau
   d'isolation de sa session ? Pré-import (`run_backup_and_restore`) : deux connexions tenues (le `tx` du verrou et
   l'instantané) sur les 5 du pool — interblocage possible ? Le `FOR UPDATE` sur `_kesh_version` contre la lecture
   cohérente de la même ligne ? Un appelant qui tiendrait déjà la seule connexion libre ? La durée de l'instantané
   (purge InnoDB, verrous de métadonnées tenus jusqu'au `ROLLBACK` sur les tables lues) : une DDL ou une restauration
   concurrente qui attendrait sans fin ?
3. **Les tests prouvent-ils ce qu'ils nomment.** Assertions de montage (le geste concurrent a eu lieu, le helper
   `orphelins` voit la clé de l'import, la connexion réacquise est la même, `@@in_transaction` vaut 1 **dans**
   l'instantané) ; tests qui passeraient à vide ; déterminisme (synchronisation par l'observateur, jamais par
   `sleep`) ; (h) (`KILL` puis `finish`) ; la preuve « même texte, rouge puis vert » (`git log -L` par fonction,
   `exit=0`, témoin positif, `git diff -U0 a914762b 1894bf0c` en ajouts purs) — **rejoue-la** (`git log`, `git diff`
   sont autorisés).
4. **Les mutations.** Rejoue mentalement M1 à M11 (journal `/home/gcorbaz/devel/kesh-gate-logs/28-3-mutations.log`,
   sorties `28-3-mut/`) et cherche une **mutation plausible qui resterait verte** (ex. : `finish` qui ignore l'erreur
   du `ROLLBACK` ; `export_installation` qui avale l'erreur de `finish` — angle mort déclaré à l'AC1 : est-il
   vraiment inatteignable par un test ? ; le cœur qui saute une table ; `begin_on` qui pose le `SET` après le
   `START` ; une lecture déplacée hors du cœur dans `export_installation`).
5. **Format inchangé** (AC3, AC12) : ordre des entrées ZIP, NDJSON, manifeste, `KeshBackupMeta`, erreurs
   `AdminFullExportFailed` / `AdminPreImportBackupFailed` — un `.keshbackup` de la 0.13.3 s'importe-t-il dans une
   0.13.2 et réciproquement ? Ni `migrations/`, ni `test-schema/`, ni `post_restore.rs`, ni `Cargo.toml` touchés
   (`git diff --stat a4da521b 90c85099`).
6. ⛔ **LES MANUELS — `docs/manual/fr/admin-manual.tex`, `user-manual.tex` ET leurs PDF.** Le § *Exporter
   (sauvegarde complète)* dit-il **vrai** sur le code : l'entrée d'audit `admin.full_export` écrite après le fichier
   (`routes/admin.rs`, `emit_full_export_audit`) ; « un seul instantané pris au début » ; « ne bloque aucune
   saisie » ; la sauvegarde pré-import « lue de la même façon » ; la limite InnoDB ; « une table, `contact_persons` »
   (`contacts` contrainte par ses clés étrangères entrantes : vrai ?) ; les deux requêtes à base **nommée**, le
   témoin « au moins 40 », `_sqlx_migrations` ; le remède (`ALTER … ENGINE=InnoDB`, `SHOW CREATE TABLE`, recréation des
   deux clés — noms, colonnes, `ON DELETE` identiques à la migration `20260706000001_contact_persons.sql` ?) ;
   `default_storage_engine`. Le manuel utilisateur (§ *Exports de souveraineté*). **Cherche une phrase des manuels,
   du README ou du site, hors des sections touchées, que la story rend fausse** (ex. : une description de la
   sauvegarde pré-import ; « lecture seule » ailleurs ; une promesse sur l'export global ; la § *Quelle méthode de
   sauvegarde choisir ?*). Contrôle les **PDF aplatis** (`pdftotext docs/manual/fr/<manuel>.pdf - | tr '\n' ' ' | tr
   -s ' '` vers `/home/gcorbaz/devel/kesh-gate-logs/`) — l'apostrophe y est typographique, écrire `.` à sa place dans
   un motif.
7. **CHANGELOG et commentaires** : une seule section `## [0.13.3] — Non publié` ; rien qui promette au-delà du code ;
   propagation de l'AC8 (`grep -rnE "#633|lit via le pool|connexions séparées|torn-read|Lectures successives|sans instantané" crates docs CHANGELOG.md --include=*.rs --include=*.tex --include=*.md`
   et `grep -rnE "export\.rs:[0-9]+" crates docs`) : chaque ligne restante est-elle juste ?
8. **Décomptes** : tout nombre du Dev Agent Record (tests ajoutés, nextest, Vitest, E2E, mutations, choix, occurrences
   G18) recompté depuis la source, avec son périmètre.

## Lentilles

- **B — Blind Hunter : bugs et états**, sans présupposer que la fiche a raison. `snapshot.rs` ligne à ligne (que se
  passe-t-il si `acquire` échoue, si le `SET` échoue, si le `START` échoue, si une lecture échoue, si `finish` échoue,
  si le futur est annulé — client HTTP qui coupe pendant l'export : le `Drop` d'une `Transaction` hors runtime ?) ;
  `export_installation` / `export_installation_in` (erreur au milieu de la boucle) ; `build_keshbackup` (mémoire : les
  39 NDJSON tenus en même temps, angle mort écrit aux Dev Notes) ; `Send` du futur du handler ; `pub(crate)` de
  `begin_on`, visibilité de `Etape`.
- **E — Edge Case Hunter : bords, et les tests prouvent-ils ce qu'ils nomment.** Axes 2, 3, 4 en profondeur : base
  vide (aucune société : `read_instance_id` à 0), table vide, une table non InnoDB (le test (e) et M9), serveur en
  `READ COMMITTED` global, `autocommit=0` sur la session, connexion du pool déjà porteuse d'un `SET TRANSACTION`
  résiduel, `max_connections` du pool épuisé à l'import. Les helpers de test (`orphelins` : clés composites, clé
  auto-référente, table référencée absente du fichier) ; l'observateur (panique dans le futur).
- **A — Acceptance Auditor : conformité, décomptes, documentation.** Chaque AC (1 à 12) contre le code : satisfait,
  partiel, absent. Chaque test nommé à l'AC5 existe-t-il, sous le nom et la forme prescrits, et prouve-t-il ce qu'il
  nomme ? T0-T6 cochées à raison ? Les faits MariaDB déclarés « tranchés » au Dev Agent Record le sont-ils
  réellement par un test ou une mutation exécutés (et non par raisonnement) ? Les choix C-28-3-33 à 39 : justifiés, ou
  défaut ? Les écarts à la fiche (signature d'`orphelins`, commit T3-T4 unique, G18) sont-ils écrits ? Décomptes
  recomptés (axe 8).

## Ce que tu rends

Ton rapport dans `/home/gcorbaz/devel/kesh-gate-logs/28-3-review-p1-<B|E|A>.md` (ta lettre). Findings avec sévérité
(CRITICAL/HIGH/MEDIUM/LOW), `fichier:ligne`, **preuve** (commande et sortie, ou code relu), correction proposée. Pour
tout finding affirmant qu'un code ou un texte est absent ou présent : la sortie d'un `grep -nF` copiée.
⛔ **La liste des axes exercés ET non exercés** (les huit axes communs et ceux de ta lentille) — un « 0 finding » sans
elle ne compte pas.

## Interdits

⛔ N'écris aucun fichier hors de ton rapport dans `/home/gcorbaz/devel/kesh-gate-logs/` (et le texte aplati des PDF au
même endroit) ; aucune commande qui écrit dans le dépôt ou dans une base, ni qui compile ou exécute : `scripts/*` (dont
`scripts/prepare-release.sh`, `scripts/test-fast.sh`, `scripts/mem-guard.sh`), `make`, `latexmk`, `xelatex`,
`git commit`/`add`/`checkout`/`switch`/`stash`/`reset`/`rebase`/`apply`, `sqlx`, `cargo` (aucune sous-commande), `npm`,
`npx`, `node`, `docker`, `mariadb`, `mysql`, et **tout interpréteur** (`python`, `python3`, `perl`, `ruby`, `bash -c`
d'un script, `awk` qui écrit), `gh` en écriture, aucune requête SQL ni HTTP. Aucun fichier hors du dépôt. Autorisés :
lecture, `grep`, `sed -n`, `wc`, `git log`/`show`/`diff`, `gh issue view`, `pdftotext` vers
`/home/gcorbaz/devel/kesh-gate-logs/`.
