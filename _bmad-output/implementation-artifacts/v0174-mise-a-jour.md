# Story v0174-mise-a-jour : la 0.17.4 — Kesh sauvegarde lui-même avant d'appliquer des migrations (#847)

Status: split

*Fiche rédigée le 2026-11-05, en autonomie (Opus 5.5), worktree `/home/gcorbaz/devel/kesh-0174m`, branche
`story/v0174-mise-a-jour` partie de `7bd4d5c0` (`origin/main`, rétrospective de la série 0.17). Release **0.17.4**,
**sans migration** (C-v0174m-13). Registre : `v0174-choix-autonomes-v0174m.md` (C-v0174m-1 à 29 ; C-6, C-15, C-21 retirés). Clé courte : `v0174m`.
Issue : **#847** (P2, enhancement), en `refs` seulement dans les commits ; la fermeture se porte sur la PR de release.
Les numéros de ligne sont ceux de `7bd4d5c0`. Une fiche ne compile ni ne teste rien : tout ce qui doit être exécuté est
prescrit au développement.*

*Fiche **parente** (C-v0174m-18), réduite le 2026-11-05 par la **décision du Project Lead** (C-v0174m-30 : version
minimale de #847, après la non-convergence de la validation) : elle garde l'analyse du code ; **une seule sous-fiche se
développe**, `v0174-mise-a-jour-1.md` (la sauvegarde avant migration, version minimale) ; la sœur `v0174-mise-a-jour-2.md`
(le type « Avant mise à jour » à l'écran) est `superseded`, son contenu passe en issue. La conception commune ci-dessous
est remplacée par celle de la sous-fiche 1 là où elles diffèrent.*

## Story

En tant qu'**administrateur** d'une installation Kesh qui tient des livres réels, je veux que Kesh, au démarrage d'une
nouvelle version qui doit migrer la base, écrive **lui-même**, avant toute migration, une sauvegarde complète de l'état
d'avant — base **et** pièces —, refuse de migrer si cette sauvegarde échoue, et me dise où elle est et comment revenir
en arrière, pour que le retour à la version précédente ne dépende plus de ma seule discipline.

## Origine et contrainte

- **Motif réel** : deux mises à jour du NAS de production sur deux se sont faites sans sauvegarde préalable — de la
  0.16.1-rc1 à la 0.17.0-rc1 le 2026-10-31, puis à la 0.17.3 le 2026-11-05. Avec une migration, la version précédente
  refuse de démarrer sur la base migrée (`CLAUDE.md`, *Migration breaking policy*, P1) : sans sauvegarde, il n'y a pas de
  retour. Remarque du Project Lead (#847) : « il faut intégrer les sauvegardes dans le processus de mise à jour ».
- **Production** : la 0.17.3 tient les livres réels du Project Lead. Rien de cette story ne se joue sur le NAS ni sur ses
  données : bases `#[sqlx::test]` jetables, base de démonstration jetable pour la recette et les captures, fixtures
  synthétiques (`donnees-reelles-hors-depot-public.md`).
- **Quand le geste servira-t-il pour la première fois ?** À la première mise à jour **vers une version qui porte une
  migration**. La fiche `v0174-session` (seule story de la 0.17.4 susceptible d'en porter une) n'en prévoit **aucune**
  (`kesh-0174s/_bmad-output/implementation-artifacts/v0174-session.md`, C-v0174s-2, lu le 2026-11-05) : sauf changement à
  l'intégration, la mise à jour 0.17.3 → 0.17.4 n'écrit aucune sauvegarde — c'est voulu (C-v0174m-3) et le manuel le dit.
  Une future migration **qui crée une table applicative** est précisément le cas que la conception rend possible
  (C-v0174m-1).
- **Hors périmètre** : #848 (heure dans le nom du fichier **téléchargé** depuis l'écran) — il ne se règle pas du même
  geste (C-v0174m-13) ; bump de version Cargo, macros des manuels, tag (orchestrateur).

## Ce que le code dit (relevé le 2026-11-05, `7bd4d5c0`)

### 1. L'ordre du démarrage (`crates/kesh-api/src/main.rs`)

| Étape | Lignes | Ce qui se passe |
|---|---|---|
| 3 | `:82-108` | pool MariaDB (5 connexions) ; échec → `exit(1)` |
| 3b | `:122-154` | `check_downgrade_protection` (`kesh-db/src/version.rs:222`) : `FreshInstall` si `_kesh_version` absente (1146), `Aligned`, `BinaryAhead`, ou refus (`DowngradeRefused`, `RowMissing`) → `exit(1)` |
| 4 | `:157-161` | `kesh_db::MIGRATOR.run(&pool)` ; échec → `exit(1)` |
| 4-bis | `:169-188` | backfill Rust `client_number_canonical` |
| 4b | `:191-196` | `record_boot_version` : `kesh_version_last_applied = CARGO_PKG_VERSION`, non fatal |
| 5 | `:200-` | bootstrap admin, prompts MCP, nettoyage des jetons |
| 6 | `:376-419` | `AppState`, `auto_backup::spawn` (`:403`), `bind` puis `axum::serve` |

Rien n'écrit en base avant l'étape 4 : le pool n'a fait que des `SELECT`. **Aucune** étape ne sauvegarde. Le serveur
n'écoute qu'à l'étape 6 : tant que la sauvegarde tourne, `/health` ne répond pas (le *healthcheck* des compose,
`docker-compose.yml:165-169`, marque alors le conteneur « starting » puis « unhealthy », sans le tuer).

### 2. Savoir, avant `run`, qu'il y a des migrations à appliquer

`MIGRATOR` est `sqlx::migrate!("./migrations")` (`kesh-db/src/lib.rs:30`, 89 fichiers à `7bd4d5c0`). `sqlx-core 0.8.6`,
`Migrator::run_direct` : verrou, `ensure_migrations_table`, refus si `dirty_version()` (une ligne `success = 0`), refus si
une migration appliquée est inconnue (`validate_applied_migrations`), puis, pour chaque migration montante connue
absente de `_sqlx_migrations`, `apply`. Le plan se calcule donc **en lecture**, avant `run` : `SELECT version, success
FROM _sqlx_migrations` comparé à `MIGRATOR.iter()` (montantes seulement) — C-v0174m-3.

### 3. La sauvegarde complète d'aujourd'hui, et pourquoi elle ne peut pas tourner telle quelle avant migration

- **Lecture** : `export_installation(pool)` (`kesh-db/src/backup.rs:474`) ouvre un `ReadSnapshot`, lit `min_required`
  (`:348`) et `instance_id` (`:362`), puis **chaque table de `TABLES_TO_TRUNCATE`** (`:46`, liste écrite en dur, celle du
  binaire qui tourne) par `export_table` (`:214`), puis les pièces référencées (`read_referenced_files`, `:419` :
  `imported_supplier_invoices.storage_path`, `invoices.pdf_storage_path`). `InstallationExport.tables` est un
  `Vec<(&'static str, TableExport)>` (`:379`). Les colonnes, elles, sont lues **dynamiquement** dans
  `INFORMATION_SCHEMA` (`non_generated_columns`, `:182`), et chaque cellule est décodée par son `DATA_TYPE`.
- **Écriture** : `build_keshbackup` (`kesh-api/src/admin_backup/export.rs:148`) puis `write_keshbackup` (`:234`) :
  `data/<table>.ndjson`, `files/`, les pièces une à une, `manifest.json` (format 2, `manifest.rs:27`) ;
  `KeshBackupMeta.table_count` vaut `TABLES_TO_TRUNCATE.len()` (`export.rs:330`). Le fichier est posé par
  `write_backup_file` (`routes/admin.rs:754` : `.partial` en `create_new` `0600`, `sync_all`, renommage sans écrasement).
- **La sauvegarde automatique** (31-1c, `auto_backup.rs`) : `run_once` (`:672`) exige un `AppState` et lit
  `_kesh_backup_schedule` ; elle écrit dans `auto/` (`AUTO_SUBDIR`, `:55`), nom `kesh-auto-<UTC ms>Z.keshbackup`
  (`:581`), contrôle l'espace (`seuil_d_espace`, `:510` : `max(dernière, documents) × 1,2 + 256 Mio`, par
  `BackupEnv::free_bytes`), relit l'annuaire et le manifeste (`relire`, `:551`), applique la rétention
  (`appliquer_la_retention`, `:525`). Elle ne tourne qu'après le démarrage (`spawn`, premier tour à 5 minutes) : **la base
  est alors déjà migrée**.
- **L'import** de la version de départ exige que les tables du manifeste soient **exactement** son `TABLES_TO_TRUNCATE`
  (`admin_backup/import.rs:234-249`, dans les deux sens), puis que les colonnes de chaque table soient compatibles avec son
  schéma (`check_schema_compat`, `:579`) ; il ne contrôle la version que par `kesh_version_min_required`
  (`routes/admin.rs:223-230`), `keshVersion` n'étant qu'informatif (audit, `:254`, `:601`).

**Conséquence** : lancé avant migration, `export_installation` du binaire neuf échoue dès qu'une migration en attente
crée une table applicative (la table est dans son `TABLES_TO_TRUNCATE`, pas dans la base : `non_generated_columns` rend
une liste vide, `export_table` rend `Invariant("table '…' has no non-generated columns")`, `backup.rs:216-220`) ; et,
même quand il réussit, son manifeste ne porte l'inventaire de la version de départ que si les deux inventaires
coïncident. Exemple concret à `7bd4d5c0` : une base arrêtée avant `20261027000001_mcp_prompts` n'a ni `mcp_prompts` ni
`mcp_prompt_versions`, que `TABLES_TO_TRUNCATE` nomme.

**Ce qui rend la lecture « à l'inventaire du schéma présent » juste** (C-v0174m-1) : à chaque version, la garde
`backup_inventory_matches_schema` (`backup.rs:848-874`) impose « tables du schéma, moins `_sqlx_migrations`,
`_kesh_version`, `_kesh_backup_schedule`, `_kesh_installation_settings` = `TABLES_TO_TRUNCATE` » ; les quatre tables
système portent le préfixe `_`, aucune table de `TABLES_TO_TRUNCATE` ne le porte (lecture de la liste `:46-127`). Les
tables du schéma présent dont le nom ne commence pas par `_` sont donc exactement le `TABLES_TO_TRUNCATE` de la version
qui a créé ce schéma — celui que son import exigera.

### 4. Le dossier des sauvegardes et la liste du serveur

`KESH_ADMIN_BACKUP_DIR` (défaut `/data/backup`, `config.rs:1274` ; `./backup` avec les compose fournis,
`/volume1/docker/kesh/backup` sur Synology), créé par `ensure_backup_dir` (`routes/admin.rs:143`, `0700`). La liste
*Sauvegardes sur le serveur* (`lister_les_sauvegardes`, `auto_backup.rs:367`) connaît deux types (`BackupKind`, `:264` :
`Auto` dans `auto/`, `PreImport` à la racine), filtrés par leur expression (`NOM_AUTO` `:65`, `NOM_PRE_IMPORT` `:71`) ;
le téléchargement (`routes/backup_schedule.rs`, `download_backup`) valide `kind` et le nom puis les retrouve dans la liste,
et s'inscrit au journal d'audit (`admin.backup_downloaded`). Côté écran : `backup-schedule.api.ts:69`
(`kind: 'auto' | 'pre-import'`) et un ternaire à deux branches dans `AutoBackupSection.svelte:530-532`
(clés `admin-backup-server-kind-auto` et `-pre-import`, `messages.ftl` fr-CH `:1871-1872`).

### 5. Ce que la version de départ fait du fichier

- Elle refuse de démarrer sur la base migrée (migration appliquée qu'elle ne connaît pas ; `min_required` relevé le cas
  échéant). On ne peut donc pas importer par **son** écran sans lui rendre une base qu'elle accepte : la base se recrée
  vide, puis on restaure par l'assistant (« Restaurer une sauvegarde », Story 31-1b, manuel
  § *Restaurer sur une nouvelle machine*, `admin-manual.tex:2410`).
- Une pièce déjà présente et intacte dans le dossier des documents est laissée en place à l'import
  (`AlreadyIntact`, `document_storage.rs:248`) : le dossier des documents n'a pas à être vidé.
- Une sauvegarde de format 2 ne s'importe que dans une 0.15.0 ou plus récente (`manifest.rs:18-27`) — C-v0174m-8.
- La recréation de la base par le **compte Kesh** a un précédent joué : `scripts/synology/kesh-restore.sh` (étape 6,
  `DROP DATABASE` / `CREATE DATABASE` par `kesh-restore.cnf`) et `admin-manual.tex:2005-2010` (« il faut `ALL PRIVILEGES`
  sur la base, que porte le compte Kesh ») ; la production Synology n'a **pas** de conteneur MariaDB
  (`docker-compose.prod.yml` : aucun service `mariadb`, `mem_limit: 1g` à `:193`) — C-v0174m-19.
- `sqlx-mysql-0.8.6/src/migrate.rs`, `apply` : la ligne `success = FALSE` est insérée **dans** la transaction de la
  migration ; elle ne survit que si un DDL a provoqué un commit implicite. Une migration qui échoue avant tout DDL est
  annulée **sans** ligne « sale », et les migrations précédentes de la même mise à jour restent appliquées — C-v0174m-15 (retiré), aujourd'hui C-v0174m-25.
- `_kesh_version.kesh_version_last_applied` vaut `'0.1.0'` à la création (`20260522000001_kesh_version.sql`) et n'est mis
  à jour que par `record_boot_version`, **après** `MIGRATOR.run` (`main.rs:191`) : une base montée par migrations sans
  démarrage porte `'0.1.0'` (à poser explicitement dans les tests).

### 6. Le manuel et le CHANGELOG d'aujourd'hui

- `admin-manual.tex` § *Procédure de mise à jour standard* (`:2481`), étape 1 : « **Sauvegarder** la base de données … » ;
  le `keshwarning` qui suit : « Toujours sauvegarder **avant** la mise à jour » ; § *Passer à la 0.17.3* (`:2513`) :
  « Avant la mise à jour, une sauvegarde complète est obligatoire » ; § *Rollback en cas d'échec* (`:2976`) : arrêter,
  reprendre l'ancienne image, « si les migrations DB ont été appliquées et sont irréversibles, restaurer la sauvegarde DB
  d'avant la mise à jour » — sans dire comment avec un `.keshbackup`.
- § *La sauvegarde automatique* (`:2380`), paragraphes *Volume* et *Les sauvegardes du serveur* : deux types seulement.
- `CHANGELOG.md` : pas de section `[0.17.4]` à `7bd4d5c0` (la première est `[0.17.3] — 2026-11-05`).
- Autres sites que la story rend incomplets (validation P1, F1-5, R1-13 ; PDF identique) : `admin-manual.tex:812` et
  `:2377` (`KESH_ADMIN_BACKUP_DIR`) — **traités** par la sous-fiche 1 ; `:2282` (*Compatibilité*), `:2442` (« quatre
  approches »), `:870-871`, `:1525-1526`, `.env.example` (commentaires du dossier) et commentaires des deux compose sur
  `./backup`, `README.md:50` — **laissés** par la version minimale (C-v0174m-30) : incomplets, non faux. La variable neuve,
  elle, entre à `.env.example` et dans les compose.


## Conception retenue

Voir `v0174-mise-a-jour-1.md`, § *Ce qui est fait (et rien d'autre)* et § *Conception* : c'est la seule à jour depuis la
décision du Project Lead (C-v0174m-30). En bref : plan lu dans `_sqlx_migrations` (illisible → refus) ; `.keshbackup` de
format 2 à l'inventaire du schéma présent dans `pre-upgrade/`, nom aux deux versions ; refus sur tout échec avant
migration ; échappatoire `KESH_SKIP_PRE_UPGRADE_BACKUP=<version exacte>` ; jamais de suppression ; test d'aller-retour
discriminant.

**Alternative écartée** (C-v0174m-1) : un vidage SQL — pas de client MariaDB dans l'image, pas de pièces, un second format
à maintenir ; et le `.keshbackup` à l'inventaire du binaire neuf, qui échoue ou produit un fichier que la version de départ
refuse.

## Découpage

Une seule sous-fiche à développer : `v0174-mise-a-jour-1.md` (`kesh-db/backup`, `kesh-api`, compose et `.env.example`,
manuel, CHANGELOG — sous cinq modules). `v0174-mise-a-jour-2.md` : `superseded` (C-v0174m-30). La parente reste `split`
(le plus simple : l'analyse du code n'a pas à être recopiée, et les validations passées la citent).

## Pour l'orchestrateur

- **#847** : `closes #847` sur la PR de release, `refs #847` dans les commits.
- **Commentaire à #847** : le point de conception est tranché par C-v0174m-1 (`.keshbackup` à l'inventaire du schéma
  présent ; le vidage SQL écarté) ; le retour passe par un vidage de sécurité puis les procédures de restauration existantes (installation neuve de la version de départ, assistant « Restaurer une sauvegarde »), sans procédure neuve (C-v0174m-32).
- **Issue mémoire** (R1-14) : elle existe, **#911** (« Sauvegarde complète : toutes les tables sont chargées en mémoire
  avant l'écriture — risque d'OOM sous le mem_limit 1g de production », ouverte) ; citée au manuel et en Dev Notes. Rien à
  ouvrir.
- **Intégration** : si une autre story de la 0.17.4 ajoute finalement une migration, corriger la phrase du CHANGELOG
  (`v0174-mise-a-jour-1.md`) et rejouer la recette de 0.17.3 à 0.17.4.
- **Splitting** : la dérogation du premier jet (C-v0174m-12) est retirée ; une seule sous-fiche à développer depuis la
  décision du Project Lead (C-v0174m-30).
- **Issue à ouvrir** : le type « Avant mise à jour » à l'écran (contenu de la sous-fiche 2, `superseded`) — texte dans le
  rapport de l'agent.

## Change Log

- 2026-11-05 — Fiche créée (Opus 5.5, autonomie) : prémisses lues au code à `7bd4d5c0` ; conception C-v0174m-1 à 14 ;
  prochaine étape : validation P1 (Sonnet, lentilles R et F, `v0174-mise-a-jour-validate-prompt-p1.md`).
- 2026-11-05 — Validation P1 (Sonnet ; R 0/2/8/7, F 0/0/6/6) remédiée (Opus 5.5) : 29 findings, tous traités, un correctif
  proposé rejeté (R1-1, option « ne jamais réécrire un couple » : motif au registre C-v0174m-15 ; détail au Change Log de
  `v0174-mise-a-jour-1.md`) ; fiche découpée en deux sous-fiches sœurs
  (C-v0174m-18) ; choix C-v0174m-15 à 24 ajoutés, C-v0174m-1, -8, -9, -10, -12, -13 amendés ; prochaine étape :
  validation P2 (Opus, lentille complète, `v0174-mise-a-jour-validate-prompt-p2.md`), sur les trois fichiers.
- 2026-11-05 — Validation P2 (Opus, 0/1/10/9 ; 16 des 20 issus de la remédiation P1) remédiée (Opus 5.5) sur la décision
  de l'orchestrateur, **simplifier plutôt que corriger** (signal de non-convergence déclaré au Project Lead) : aucune
  suppression automatique des sauvegardes d'avant mise à jour (C-v0174m-25, retire C-6 et C-15), échappatoire par la seule
  variable du `.env` transmise par le compose (C-v0174m-28, retire C-21) ; C-v0174m-26, -27, -29 ajoutés. Les 20 findings
  traités, un correctif proposé écarté (V2-1 : protéger la plus ancienne depuis `last_boot_at`, remplacé par le retrait de
  la rétention) ; détail au Change Log de `v0174-mise-a-jour-1.md`. Prochaine étape : validation P3 (Sonnet, lentille
  complète braquée d'abord sur cette remédiation, `v0174-mise-a-jour-validate-prompt-p3.md`).
- 2026-11-05 — Validation P3 (Sonnet, 0/1/5/4) : la boucle ne converge pas (un HIGH à chaque passe, tous recyclés).
  **Décision du Project Lead : version minimale de #847** (C-v0174m-30). Sous-fiche 1 récrite ; sous-fiche 2
  `superseded` (issue demandée) ; parente gardée en `split`. Prochaine étape : validation P4 (Opus, lentille complète,
  `v0174-mise-a-jour-validate-prompt-p4.md`), sur la sous-fiche 1 et le registre.
- 2026-11-05 — Validation P4 (Opus, 0/2/5/7) remédiée : aucune procédure de restauration neuve (C-v0174m-32), renvoi aux
  procédures existantes après un vidage de sécurité ; garde de convention rétablie ; registre et parente alignés.
  Prochaine étape : validation P5 (Sonnet, `v0174-mise-a-jour-validate-prompt-p5.md`).
