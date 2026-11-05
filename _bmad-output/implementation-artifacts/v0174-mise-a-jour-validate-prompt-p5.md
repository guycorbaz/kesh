# Validation P5 — fiche v0174-mise-a-jour-1, version minimale (Sonnet, lentille complète braquée d'abord sur la remédiation P4, lecture seule)

Tu es la lentille de validation P5 de la fiche BMAD **v0174-mise-a-jour-1** de Kesh (release 0.17.4, issue #847 : Kesh
écrit lui-même une sauvegarde complète avant d'appliquer des migrations). Modèle : **Sonnet**. Contexte frais. Lentille
**complète**. Travaille en français soigné.

**Contexte.** Les validations P1 à P3 n'ont pas convergé ; le Project Lead a décidé une **version minimale**
(C-v0174m-30, commit `a1fc8530`). La P4 (Opus) a rendu 0/2/5/7 sur la fiche réduite ; sa remédiation est le commit
`03e9032b`. Ses deux HIGH portaient sur la procédure de retour écrite par la fiche : l'orchestrateur a décidé que **la
story n'écrit aucune procédure de restauration neuve** (C-v0174m-32) — le manuel dit où est le fichier, lequel prendre,
impose un vidage de sécurité et renvoie aux procédures existantes.

**Ordre de lecture imposé** : d'abord ce que la remédiation P4 a écrit
(`git -C /home/gcorbaz/devel/kesh-0174m diff b088cb1f 03e9032b`) — la sévérité se déplace vers ce qu'on vient d'écrire —,
puis le reste de la fiche. Ne réintroduis pas ce que les décisions ont retiré (écran, règle détaillée, garde de format,
nettoyage du `.partial` sur panique, W1 à W3, seconde jambe, procédure de restauration neuve) : un finding qui le demande
n'est recevable que s'il montre qu'en son absence la version minimale **perd des données ou laisse migrer sans sauvegarde
ni refus**.

## À lire, en entier

1. La fiche : `/home/gcorbaz/devel/kesh-0174m/_bmad-output/implementation-artifacts/v0174-mise-a-jour-1.md`.
2. La parente `v0174-mise-a-jour.md` (§ *Ce que le code dit* : l'analyse du code, à `7bd4d5c0`) et le registre
   `v0174-choix-autonomes-v0174m.md` (§ *Décision du Project Lead* et § *Choix en vigueur*).
3. La décision et le mandat : le message de l'orchestrateur est consigné en C-v0174m-30 ;
   `/home/gcorbaz/devel/kesh-gate-logs/prompts/v0174-mise-a-jour-fiche.md`,
   `/home/gcorbaz/devel/kesh-gate-logs/prompts/v0174-commun.md` ; `gh issue view 847 --json title,body,comments`,
   `gh issue view 911 --json title` (lecture seule). Rapports passés, pour ne pas reproduire un défaut déjà relevé :
   `/home/gcorbaz/devel/kesh-gate-logs/v0174m-validate-p1-R.md`, `-p1-F.md`, `-p2.md`, `-p3.md`, `-p4.md` (la P4 en
   entier : juge si chacun de ses findings est fermé par le Change Log de la fiche).
4. `/home/gcorbaz/devel/kesh-0174m/CLAUDE.md` : *Review Iteration Rule*, *Migration breaking policy*, *Issue Tracking
   Rule*.
5. Le code, au commit `7bd4d5c0` (par `Read`, `git grep`, `git show 7bd4d5c0:…`) : `crates/kesh-api/src/{main,lib,config,
   auto_backup}.rs`, `crates/kesh-api/src/admin_backup/{export,import,manifest}.rs`, `crates/kesh-api/src/routes/admin.rs`
   (`write_backup_file`, `ensure_backup_dir`, `full_import`), `crates/kesh-db/src/{version,backup,snapshot}.rs`,
   `crates/kesh-api/tests/{configuration_transmise.rs,common/binaire.rs}`, `crates/kesh-db/tests/{common/mod.rs,
   test_schema_guard.rs}`, `docker-compose.yml`, `docker-compose.prod.yml`, `.env.example` ; `sqlx` :
   `~/.cargo/registry/src/index.crates.io-*/sqlx-core-0.8.6/src/migrate/migrator.rs`, `sqlx-mysql-0.8.6/src/migrate.rs`.
6. **Le manuel** : `docs/manual/fr/admin-manual.tex` (§ *Sauvegarde et restauration*, § *Mise à jour de Kesh*, § *Variables
   d'environnement complètes*, `sec:backup-dsm`, lignes 812, 2329-2333, 2377, 2410-2440, 2481-2500, 2976-2995) **et son
   PDF** `docs/manual/fr/admin-manual.pdf`, aplati avant tout grep :
   `pdftotext docs/manual/fr/admin-manual.pdf - | tr '\n' ' ' | tr -s ' ' | /usr/bin/grep -oE "…" | wc -l` — rejoue les
   comptes « avant » du § *Manuel*, point 6. `CHANGELOG.md` (tête).

⚠️ Dans ce shell, `grep` est une fonction vers `ugrep` : utilise `/usr/bin/grep` ou `git grep`.

## Ce que tu vérifies (axes)

- **A1 — Prémisses au code.** Les lignes et comportements cités (ordre du démarrage, `sqlx`, `write_backup_file`,
  `seuil_d_espace`, `BackupError::Task`, `export_installation`, `parse_and_verify`, `restore_tables_in_tx`,
  `_kesh_version` initial à `'0.1.0'`) sont-ils vrais à `7bd4d5c0` ? Copie la ligne lue pour toute affirmation contestée ou
  critique.
- **A2 — Les quatre points de la décision.** Chacun est-il prescrit sans trou : plan (états, plan illisible → refus) ;
  contenu à l'inventaire du schéma présent et importable par la version de départ ; temporaire puis renommage, refus sur
  dossier, espace, lecture, écriture, panique, **aucune migration appliquée** dans chaque cas ; échappatoire à version
  exacte évaluée en premier (C-v0174m-31) et transmise ; aucune suppression ; chemin au journal. Un chemin d'échec reste-t-il
  qui laisserait migrer sans sauvegarde ni refus ?
- **A3 — Ordre et échappatoire.** Évaluer l'échappatoire avant le plan est-il sûr (état « interrompu » refusé ensuite par
  `sqlx`) ? La variable satisfait-elle les familles L, T, V, E, F de `configuration_transmise.rs` dans `SANS_DEFAUT` ? Le
  message de refus est-il vrai pour toutes les causes ?
- **A4 — Tests et montages** (dont ceux ajoutés en P4 : T7-bis et `BackupEnv::sync_dir`, T11 et T11-bis, T15, T16 et
  `TABLES_SYSTEME`, T1-ter autonome, T2 sur le vrai `MIGRATOR`, `#![cfg(test)]`, `lire_installation` et les gardes
  lexicales récrites). Tests internes dans `crates/kesh-api/src/pre_upgrade_backup/tests.rs` incluant
  `crates/kesh-db/tests/common/mod.rs` par `#[path]` : le chemin relatif est-il juste depuis ce fichier, les dépendances
  disponibles en `[dev-dependencies]` / `[dependencies]` de `kesh-api`, `test_schema_guard` satisfaite ? T1-ter atteint-il
  `parse_and_verify_avec_inventaire` (`pub(crate)`), `check_schema_compat`, `restore_tables_in_tx_avec_inventaire` (`pub`
  dans `kesh-db`) ? Est-il discriminant (l'import à l'inventaire du binaire refuse) et son oracle indépendant ? T8
  (binaire) : environnement suffisant, port occupé, `KESH_LOCALES_DIR` ? T9 et T10 (pool fermé) discriminants de 1146 ?
- **A5 — Mutations.** M1 à M18 : chacune tuée par le test nommé ? Manque-t-il une mutation sur un point gardé (absence de
  sauvegarde sur `FreshInstall`, échappatoire évaluée trop tard, panique convertie en succès) ?
- **A6 — Retour en arrière et manuel.** Le § *Manuel* point 3 renvoie-t-il à des procédures **existantes** qui, suivies
  telles qu'écrites, ramènent la version de départ sur le fichier (§ *Restaurer sur une nouvelle machine*, § *Initialisation
  manuelle de la base*, § *Script de sauvegarde mariadb-dump*, `kesh-dump.sh`) ? Lis-les. Le vidage de sécurité vient-il
  avant tout geste destructeur ? La règle « le plus ancien fichier écrit depuis le dernier démarrage réussi de la version de
  départ » est-elle juste ? Le manuel dit-il ce que la version minimale fait, et rien de ce qui a été retiré ? Les comptes du
  point 6 se retrouvent-ils dans le PDF ? Le rejet de la prémisse V4-2 sur le manuel existant (`admin-manual.tex:2183-2186`,
  cas « base présente mais illisible ») est-il fondé ?
- **A7 — CHANGELOG et cohérence.** Le CHANGELOG est-il vrai (la 0.17.4 sans migration : `kesh-0174s/_bmad-output/
  implementation-artifacts/v0174-session.md`, C-v0174s-2, en lecture) ? La fiche, la parente, la sous-fiche 2
  (`superseded`) et le registre (choix retirés marqués, § *Choix en vigueur*) sont-ils cohérents entre eux ? Un résidu de
  la conception complète (règle RG, E1, rétention, W1-W3, T18, `pub mod admin_backup`, refus `< 0.15.0`, relecture) reste-t-il
  hors des listes « Retiré » et de l'historique daté ?
- **A8 — Périmètre.** Aucune migration ; #848 hors du geste ; modules sous le seuil de la règle de splitting.

## Sévérités

CRITICAL (la sauvegarde ne serait pas restaurable par la version de départ, une migration pourrait passer sans
sauvegarde ni refus, ou des données seraient perdues), HIGH (AC faux ou intestable, refus manquant, procédure de retour
qui n'aboutit pas), MEDIUM (prémisse fausse, test qui ne tuerait pas sa mutation, manuel faux, incohérence entre fichiers),
LOW (rédaction).

## Interdits — lecture seule stricte

Aucune commande qui écrit, compile ou exécute du code du dépôt : pas de `cargo`, `npm`, `npx`, `make`, `latexmk`,
`docker`, `curl`, `sqlx`, `mariadb` ; **pas de `scripts/prepare-release.sh`** ni d'aucun script de `scripts/` ; pas de
`git checkout` ni `git switch` (sous aucune forme), `git reset`, `git stash`, `git commit`, `git push`, `git tag`,
`git rebase`, `git worktree` ; aucune commande `gh` qui écrit (`gh issue view` en lecture seulement) ; pas de `claude`,
`pkill`, `killall`, `rm` ; pas de `sed -i`, `perl -pi`, `tee`, ni de redirection `>` vers le dépôt ou vers `/tmp`.
`pdftotext … -` vers la sortie standard est permis. **Tu n'écris qu'un seul fichier** : ton rapport. N'ouvre ni
`~/kesh/`, ni `~/kesh-test/`.

## Rapport

`/home/gcorbaz/devel/kesh-gate-logs/v0174m-validate-p5.md` (chemin absolu), en français :

1. En-tête : modèle, date, commit lu (`git -C /home/gcorbaz/devel/kesh-0174m rev-parse HEAD`).
2. Décompte `CRITICAL/HIGH/MEDIUM/LOW`, recompté depuis ta liste ; dis combien viennent de la remédiation `03e9032b`.
3. Findings numérotés `V5-n` : sévérité, axe, citation, preuve au code (ligne lue copiée : sortie `git grep -nF` ou extrait
   `Read`), correctif proposé — le plus petit qui ferme le défaut.
4. **Axes exercés et axes non exercés** (A1 à A8, un par un, avec ce que tu as réellement lu ou greppé, manuel et PDF
   compris). Un « 0 finding » sans cette liste ne compte pas comme passe.
