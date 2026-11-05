# Validation P1 — fiche v0174-mise-a-jour (Sonnet, lentilles R et F, lecture seule)

Tu es une lentille de validation P1 de la fiche BMAD **v0174-mise-a-jour** de Kesh (release 0.17.4, issue #847 : Kesh
écrit lui-même une sauvegarde complète avant d'appliquer des migrations). Modèle : **Sonnet**. Contexte frais. Travaille
en français soigné. Deux lentilles, lancées séparément sur ce même prompt :

- **R (prémisses et faisabilité)** : axes A1 à A7 en priorité ;
- **F (fermeture : AC, tests, mutations, manuel, registre)** : axes A6 à A12 en priorité.

Chaque lentille exerce **tous** les axes qu'elle peut, et déclare les autres non exercés.

## À lire, en entier

1. La fiche : `/home/gcorbaz/devel/kesh-0174m/_bmad-output/implementation-artifacts/v0174-mise-a-jour.md`.
2. Le registre : `/home/gcorbaz/devel/kesh-0174m/_bmad-output/implementation-artifacts/v0174-choix-autonomes-v0174m.md`
   (C-v0174m-1 à 14).
3. Le mandat : `/home/gcorbaz/devel/kesh-gate-logs/prompts/v0174-mise-a-jour-fiche.md` et
   `/home/gcorbaz/devel/kesh-gate-logs/prompts/v0174-commun.md` ; l'issue : `gh issue view 847 --json title,body,comments`
   (lecture seule) et `gh issue view 848 --json title,body`.
4. `/home/gcorbaz/devel/kesh-0174m/CLAUDE.md` : *Review Iteration Rule*, *Règle de splitting préventif*, *Migration
   breaking policy* (P1, P6, P7, P8), *Issue Tracking Rule*, *Inventorier les sites non résolus*.
5. Le code, au commit `7bd4d5c0` (worktree `/home/gcorbaz/devel/kesh-0174m`, par `Read`, `git grep`, `git show 7bd4d5c0:…`) :
   `crates/kesh-api/src/main.rs` ; `crates/kesh-db/src/version.rs` ; `crates/kesh-db/src/lib.rs` ;
   `crates/kesh-db/src/backup.rs` (`TABLES_TO_TRUNCATE`, `non_generated_columns`, `export_table`,
   `read_referenced_files`, `export_installation`, `export_installation_in`, `backup_inventory_matches_schema`,
   `restore_body`) ; `crates/kesh-db/src/snapshot.rs` ; `crates/kesh-api/src/admin_backup/{export,import,manifest}.rs` ;
   `crates/kesh-api/src/auto_backup.rs` ; `crates/kesh-api/src/routes/admin.rs` (`ensure_backup_dir`,
   `write_pre_import_backup`, `write_backup_file`, `full_import`) ; `crates/kesh-api/src/routes/backup_schedule.rs` ;
   `crates/kesh-api/src/config.rs` ; `crates/kesh-api/src/document_storage.rs` ;
   `crates/kesh-api/tests/configuration_transmise.rs` (familles L, T, E, F, D/G16) ; `crates/kesh-api/tests/common/binaire.rs` ;
   `crates/kesh-db/tests/common/mod.rs` ; `crates/kesh-db/tests/test_schema_guard.rs` ;
   `frontend/src/lib/features/admin-backup/{AutoBackupSection.svelte,backup-schedule.api.ts}` ;
   `crates/kesh-i18n/locales/*/messages.ftl` ; `Dockerfile` ; `docker-compose.yml` ; `docker-compose.prod.yml` ;
   `.env.example` ; `crates/kesh-db/migrations/` (noms, tables système `_…`). Pour `sqlx` :
   `~/.cargo/registry/src/index.crates.io-*/sqlx-core-0.8.6/src/migrate/migrator.rs` et l'implémentation MySQL
   (`sqlx-mysql-0.8.6/src/migrate.rs` : `list_applied_migrations`, `dirty_version`, `apply`).
6. **Le manuel** : `docs/manual/fr/admin-manual.tex` (§ *Sauvegarde et restauration*, dont *La sauvegarde automatique*,
   *Restaurer sur une nouvelle machine*, *Script de sauvegarde mariadb-dump*, *backup-dsm* ; § *Mise à jour de Kesh*,
   dont *Procédure de mise à jour standard*, *Passer à la 0.17.3*, *Rollback en cas d'échec* ; § *Variables
   d'environnement complètes*) **et son PDF** `docs/manual/fr/admin-manual.pdf`, aplati avant tout grep :
   `pdftotext docs/manual/fr/admin-manual.pdf - | tr '\n' ' ' | tr -s ' ' | /usr/bin/grep -o "…"`.
   `CHANGELOG.md` (tête).

⚠️ Dans ce shell, `grep` est une fonction vers `ugrep` : utilise `/usr/bin/grep` ou `git grep`.

## Ce que tu vérifies (axes)

- **A1 — Prémisses au code.** Chaque affirmation du § *Ce que le code dit* (lignes, ordre du démarrage, comportement de
  `sqlx` 0.8.6, échec d'`export_table` sur une table absente, contrôles de l'import, liste du serveur) est-elle vraie à
  `7bd4d5c0` ? Copie la ligne lue pour toute affirmation contestée **ou** confirmée comme critique.
- **A2 — La conception de l'inventaire (C-v0174m-1).** « Tables de base sans préfixe `_` du schéma présent =
  `TABLES_TO_TRUNCATE` de la version qui a créé ce schéma » : vrai pour **chaque** version publiée de 0.15.0 à 0.17.3 ?
  (Rejoue `git show v0.15.0:crates/kesh-db/src/backup.rs`, `v0.16.0`, `v0.17.0`, `v0.17.3` : la garde
  `backup_inventory_matches_schema` et sa liste d'exclusions, et cherche une table système sans `_` ou une table
  applicative avec `_`.) Une vue, une table temporaire, une table créée hors migration (par MariaDB, par un test) peut-elle
  s'y glisser ? L'import d'une 0.17.3 (ou 0.15.0) accepterait-il réellement le fichier (`parse_and_verify`,
  `check_schema_compat`, `check_import_version_compat`, `verifier_coherence`, `formatVersion`) ? Le `keshVersion` du
  manifeste (version d'arrivée) gêne-t-il quoi que ce soit chez la version de départ (C-v0174m-14) ?
- **A3 — Le plan (C-v0174m-3).** `neuve` / `interrompue` / `à jour` / `à appliquer` couvrent-ils tous les états réels de
  `_sqlx_migrations` (table absente, vide, ligne `success = 0`, migration appliquée inconnue, checksum différent,
  migrations descendantes) ? Le plan lu correspond-il à ce que `run_direct` fera ? L'ordre « après
  `check_downgrade_protection`, avant `run` » est-il le bon (C-v0174m-2), et l'outcome `FreshInstall` suffit-il à couvrir
  la base vide ?
- **A4 — Lecture sur l'ancien schéma.** `read_min_required`, `read_instance_id`, `read_referenced_files` (colonnes
  `imported_supplier_invoices.storage_path`, `file_hash`, `invoices.pdf_storage_path`, `pdf_sha256`) existent-elles dans
  toute base de 0.15.0 à 0.17.3 ? La borne « version de départ ≥ 0.15.0 » (C-v0174m-8) est-elle la bonne, et
  `kesh_version_last_applied` est-il un indicateur fiable de cette version (`record_boot_version` non fatal, ordre dans
  `main.rs`) ?
- **A5 — Échecs et échappatoire (C-v0174m-4, -7).** Chaque échec énuméré refuse-t-il bien **avant** toute écriture en
  base ? Un échec oublié (dossier `pre-upgrade/` qui est un lien, droits, `relire`, panique dans le `spawn_blocking`,
  `ReadSnapshot::finish`) ? L'échappatoire à version exacte est-elle sûre et utilisable — en particulier pour un compose
  local **antérieur** à la 0.17.4 qui ne transmet pas la variable (`configuration_transmise`, famille T) ? Le message
  « la {de} peut redémarrer dessus » est-il vrai dans tous les cas de refus ?
- **A6 — Rétention, emplacement, nom (C-v0174m-5, -6).** L'expression du nom accepte-t-elle les versions réelles
  (`0.17.0-rc1`) et refuse-t-elle tout chemin ? La rétention peut-elle supprimer la seule sauvegarde utile (boucle de
  redémarrage, horloge déréglée, migration « sale ») ? La liste et le téléchargement (lot B) restent-ils sûrs (lien,
  `O_NOFOLLOW`, 404) ?
- **A7 — La restauration (C-v0174m-10, -11).** La procédure prescrite (vidage de sécurité, `DROP DATABASE` /
  `CREATE DATABASE`, image précédente, `/setup`, assistant) aboutit-elle ? Les droits de `kesh` survivent-ils à
  `DROP DATABASE` (fiche : « à constater en recette » — est-ce le bon traitement) ? Les pièces, le plafond d'import
  (`KESH_ADMIN_IMPORT_MAX_MB`), la sauvegarde pré-import de l'assistant ? La recette 0.17.2 → 0.17.4 exerce-t-elle
  réellement le chemin (la 0.17.3 ajoute-t-elle bien une migration que la 0.17.2 ne connaît pas) ?
- **A8 — AC décidables et tests.** Chaque AC a-t-il un test qui rougirait si l'AC était faux ? T8 (binaire, port occupé)
  est-il montable (URL de la base éphémère, environnement vidé par `lancer_binaire`, variables obligatoires de
  `Config::from_env`) ? Les tests sur base partielle respectent-ils P6 (version, non position) et `test_schema_guard` ?
- **A9 — Mutations.** M1 à M14 sont-elles chacune tuées par le test nommé ? Une mutation importante manque-t-elle
  (ex. sauvegarde écrite **après** `run`, `.partial` non nettoyé, seuil calculé sur la mauvaise taille) ?
- **A10 — Manuel, CHANGELOG et doc.** Le § *Manuel* de la fiche corrige-t-il **toutes** les phrases du `.tex` **et du PDF**
  que la story rendrait fausses (greppe « sauvegarder », « Toujours sauvegarder », « Rollback », « deux types »,
  « sauvegardes pré-import », « Sauvegardes sur le serveur ») et seulement celles-là (les § *Passer à la 0.x* passés ne
  doivent pas être réécrits) ? Le texte proposé est-il exact au regard du code prescrit ? Le CHANGELOG est-il vrai, y
  compris la phrase conditionnelle ? `README.md`, `website/`, `docs/` portent-ils une phrase à corriger ?
- **A11 — Migrations et périmètre.** « Aucune migration » est-il vrai pour tout ce que la fiche prescrit ? #848 est-il bien
  hors du même geste ? Le découpage en deux lots et la dérogation (C-v0174m-12) sont-ils justifiés, ou faut-il deux
  fiches ?
- **A12 — Registre.** Chaque C-v0174m-k a contexte, retenu, écartées, réversibilité ; aucune décision de la fiche sans
  choix ; cohérence fiche ↔ registre (noms, variable, dossier, nombre de sauvegardes gardées, lignes citées).

## Sévérités

CRITICAL (la sauvegarde ne serait pas restaurable par la version de départ, ou une migration pourrait passer sans
sauvegarde ni refus, ou des données seraient perdues), HIGH (AC faux ou intestable, refus manquant, procédure de retour
qui n'aboutit pas), MEDIUM (prémisse fausse, inventaire incomplet, test qui ne tuerait pas sa mutation, manuel faux ou
incomplet), LOW (rédaction).

## Interdits — lecture seule stricte

Aucune commande qui écrit, compile ou exécute : pas de `cargo`, `npm`, `npx`, `make`, `latexmk`, `docker`, `curl`,
`sqlx`, `mariadb` ; **pas de `scripts/prepare-release.sh`** ni d'aucun script de `scripts/` ; pas de `git checkout` ni
`git switch` (sous aucune forme), `git reset`, `git stash`, `git commit`, `git push`, `git tag`, `git rebase`, `git
worktree` ; aucune commande `gh` qui écrit (`gh issue view` en lecture seulement) ; pas de `claude`, `pkill`, `killall`,
`rm` ; pas de `sed -i`, `perl -pi`, `tee`, ni de redirection `>` vers le dépôt. **Tu n'écris qu'un seul fichier** : ton
rapport. N'ouvre ni `~/kesh/`, ni `~/kesh-test/`.

## Rapport

Lentille R : `/home/gcorbaz/devel/kesh-gate-logs/v0174m-validate-p1-R.md` ; lentille F :
`/home/gcorbaz/devel/kesh-gate-logs/v0174m-validate-p1-F.md` (chemins absolus), en français :

1. En-tête : lentille, modèle, date, commit lu (`git -C /home/gcorbaz/devel/kesh-0174m rev-parse HEAD`).
2. Décompte `CRITICAL/HIGH/MEDIUM/LOW`, recompté depuis ta liste.
3. Findings numérotés `R1-n` ou `F1-n` : sévérité, axe, citation de la fiche ou du registre, preuve au code (ligne lue
   copiée : sortie `git grep -nF` ou extrait `Read`), correctif proposé.
4. **Axes exercés et axes non exercés** (A1 à A12, un par un, avec ce que tu as réellement lu ou greppé, manuel et PDF
   compris). Un « 0 finding » sans cette liste ne compte pas comme passe.
