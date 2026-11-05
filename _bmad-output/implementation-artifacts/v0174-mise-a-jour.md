# Story v0174-mise-a-jour : la 0.17.4 — Kesh sauvegarde lui-même avant d'appliquer des migrations (#847)

Status: ready-for-dev

*Fiche rédigée le 2026-11-05, en autonomie (Opus 5.5), worktree `/home/gcorbaz/devel/kesh-0174m`, branche
`story/v0174-mise-a-jour` partie de `7bd4d5c0` (`origin/main`, rétrospective de la série 0.17). Release **0.17.4**,
**sans migration** (C-v0174m-13). Registre : `v0174-choix-autonomes-v0174m.md` (C-v0174m-1 à 14). Clé courte : `v0174m`.
Issue : **#847** (P2, enhancement), en `refs` seulement dans les commits ; la fermeture se porte sur la PR de release.
Les numéros de ligne sont ceux de `7bd4d5c0`. Une fiche ne compile ni ne teste rien : tout ce qui doit être exécuté est
prescrit au développement.*

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
  migration**. Si la 0.17.4 n'en porte pas (seule `v0174-session`, #890, en est susceptible), la mise à jour 0.17.3 →
  0.17.4 n'écrit aucune sauvegarde : c'est voulu (C-v0174m-3) et le manuel le dit. Si `v0174-session` en ajoute une
  **qui crée une table applicative**, cette story est précisément ce qui rend la sauvegarde possible (C-v0174m-1).
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

### 6. Le manuel et le CHANGELOG d'aujourd'hui

- `admin-manual.tex` § *Procédure de mise à jour standard* (`:2481`), étape 1 : « **Sauvegarder** la base de données … » ;
  le `keshwarning` qui suit : « Toujours sauvegarder **avant** la mise à jour » ; § *Passer à la 0.17.3* (`:2513`) :
  « Avant la mise à jour, une sauvegarde complète est obligatoire » ; § *Rollback en cas d'échec* (`:2976`) : arrêter,
  reprendre l'ancienne image, « si les migrations DB ont été appliquées et sont irréversibles, restaurer la sauvegarde DB
  d'avant la mise à jour » — sans dire comment avec un `.keshbackup`.
- § *La sauvegarde automatique* (`:2380`), paragraphes *Volume* et *Les sauvegardes du serveur* : deux types seulement.
- `CHANGELOG.md` : pas de section `[0.17.4]` à `7bd4d5c0` (la première est `[0.17.3] — 2026-11-05`).

## Conception retenue

1. **Un module neuf** `crates/kesh-api/src/pre_upgrade_backup.rs`, appelé par `main.rs` entre l'étape 3b et l'étape 4
   (C-v0174m-2), dans ce seul ordre :
   1. si `check_downgrade_protection` a rendu `FreshInstall` → rien ;
   2. **plan** (`plan_de_migration(pool)`, partie pure `plan_depuis(appliquees, connues)`) : `Neuve` / `AJour` /
      `Interrompue { version }` / `AAppliquer { nombre, premiere }` (C-v0174m-3) ; seul `AAppliquer` continue ;
   3. **échappatoire** : `config.skip_pre_upgrade_backup == Some(CARGO_PKG_VERSION)` → avertissement, rien d'écrit
      (`Contournee`) ; une autre valeur → avertissement « ignorée », on continue (C-v0174m-4) ;
   4. **version de départ** : `_kesh_version.kesh_version_last_applied` ; illisible ou `< 0.15.0` → refus (C-v0174m-8) ;
   5. **dossier** `<backup_dir>/pre-upgrade/` créé en `0700`, qui doit se lister ; `.partial` abandonnés supprimés
      (`supprimer_les_partiels(dir, "kesh-pre-upgrade-")`) ;
   6. **espace** : `seuil_d_espace(max(tailles de auto/ et pre-upgrade/), taille_des_documents)` contre
      `BackupEnv::free_bytes` (C-v0174m-7) ;
   7. **lecture** par la fonction neuve de `kesh-db` (ci-dessous) puis **écriture** par `write_keshbackup` dans
      `write_backup_file` (nom C-v0174m-5), **relecture** par `relire` ;
   8. **rétention** : trois plus récentes, jamais la nouvelle (C-v0174m-6) ;
   9. journal (C-v0174m-9). Tout échec des étapes 4 à 7 → `Err(Refus)` ; `main.rs` écrit le message et sort en `exit(1)`
      **avant** `MIGRATOR.run`.
2. **Dans `kesh-db/src/backup.rs`** : `export_installation_schema_present(pool)` — même `ReadSnapshot`, même
   `export_installation_in`, mais la liste des tables lue dans `information_schema.TABLES` (`TABLE_SCHEMA = DATABASE()`,
   `TABLE_TYPE = 'BASE TABLE'`, nom ne commençant pas par `_` — échapper le `_` dans le `LIKE`, ou filtrer en Rust),
   **dans l'instantané** ; `InstallationExport.tables` passe à `Vec<(String, TableExport)>` (ou `Cow<'static, str>`), et
   `KeshBackupMeta.table_count` se calcule sur `export.tables.len()`. `export_installation` (inventaire du binaire) reste
   le chemin de l'export, de l'import et de la sauvegarde automatique. Les gardes
   `build_keshbackup_ne_lit_la_base_qu_une_fois` et `export_installation_n_a_qu_un_appelant` (`export.rs:464`, `:489`)
   sont adaptées et non retirées : la seconde fonction a **un** appelant (`pre_upgrade_backup`) et partage le cœur.
   `build_keshbackup` reçoit la fonction de lecture ou un paramètre d'inventaire (au choix du développement, écrit au
   Dev Agent Record).
3. **Configuration** : `KESH_SKIP_PRE_UPGRADE_BACKUP` lue par `env_nonempty` dans `Config::from_env`
   (`skip_pre_upgrade_backup: Option<String>`), transmise par les deux compose (`${KESH_SKIP_PRE_UPGRADE_BACKUP:-}`), ligne
   commentée à `.env.example`, inscrite aux listes de `crates/kesh-api/tests/configuration_transmise.rs`
   (`EMPLACEMENTS_AUTORISES`, `VIDE_SIGNIFIANT` ou `SANS_DEFAUT` selon la forme), nommée au manuel.
4. **Lot B — l'écran** : `BackupKind::PreUpgrade` (`as_str` = `"pre-upgrade"`, dossier `pre-upgrade/`, expression
   `NOM_PRE_UPGRADE`), listé par `lister_les_sauvegardes` (un `pre-upgrade/` illisible se traite comme `auto/` : journalisé,
   la liste continue) ; `kind: 'auto' | 'pre-import' | 'pre-upgrade'` ; le ternaire devient une table de libellés à trois
   entrées ; clé `admin-backup-server-kind-pre-upgrade` dans les quatre locales.

**Alternative écartée** (C-v0174m-1) : un vidage SQL — pas de client MariaDB dans l'image, pas de pièces, un second format
à maintenir ; et le `.keshbackup` à l'inventaire du binaire neuf, qui échoue ou produit un fichier que la version de départ
refuse.

### Messages (texte exact, journal du serveur, en français comme les autres messages de démarrage)

- Plan : `Mise à jour de la base : {n} migration(s) à appliquer (de la {de} à la {vers}). Sauvegarde complète avant mise à jour en cours — base et pièces ; le serveur répondra une fois la mise à jour terminée.`
- Fin : `Sauvegarde avant mise à jour écrite : {chemin} ({taille} octets, {tables} tables, {lignes} lignes, {pieces} pièces, en {duree} s). Pour revenir à la {de}, voir le manuel d'administration, « Revenir à la version précédente ».`
- Refus : `FATAL : mise à jour refusée — la sauvegarde avant mise à jour n'a pas pu être écrite ({cause}). Aucune migration n'a été appliquée : la base est intacte et la {de} peut redémarrer dessus. Corrigez la cause, ou, en connaissance de cause, posez KESH_SKIP_PRE_UPGRADE_BACKUP={vers} pour migrer sans sauvegarde (manuel d'administration, « Sauvegarde avant mise à jour »).`
- Échappatoire utilisée : `KESH_SKIP_PRE_UPGRADE_BACKUP={vers} : migration SANS sauvegarde préalable, à la demande de l'administrateur. Retirez cette variable après la mise à jour.`
- Échappatoire ignorée : `KESH_SKIP_PRE_UPGRADE_BACKUP={valeur} ignorée : elle ne vaut que pour la version {vers}.`
- Interrompue : `Une migration interrompue ({version}) est inscrite dans la base : aucune sauvegarde avant mise à jour n'est écrite. Restaurez la sauvegarde d'avant la mise à jour (dossier {dir}) — manuel d'administration, « Revenir à la version précédente ».`

Les `{cause}` nomment l'étape (dossier, espace : libre et requis en Mio, lecture de la base, écriture, relecture, version
de départ) sans jamais citer `DATABASE_URL`.

## Acceptance Criteria

**AC-1 — Le plan.** Avant `MIGRATOR.run`, après `check_downgrade_protection`, Kesh calcule le plan : `_sqlx_migrations`
absente ou vide → *neuve* ; une ligne `success = 0` → *interrompue* ; toutes les migrations montantes du `MIGRATOR`
présentes → *à jour* ; sinon *à appliquer* (nombre exact). Seul *à appliquer* déclenche une sauvegarde. (Tests T1 à T4.)

**AC-2 — Le contenu.** La sauvegarde est un `.keshbackup` de format 2 dont les tables sont **exactement** les tables de
base du schéma présent sans préfixe `_`, chacune avec ses colonnes non générées et toutes ses lignes, les pièces
référencées sous `files/`, et `kesh_version_min_required` lu dans la base. Sur une base arrêtée avant
`20261027000001_mcp_prompts`, le manifeste ne nomme ni `mcp_prompts` ni `mcp_prompt_versions`, et l'écriture réussit.
(T1, T1-bis, T11.)

**AC-3 — Emplacement et nom.** `<KESH_ADMIN_BACKUP_DIR>/pre-upgrade/kesh-pre-upgrade-<AAAAMMJJTHHMMSSmmm>Z-<de>-to-<vers>.keshbackup`,
`<de>` = `kesh_version_last_applied`, `<vers>` = `CARGO_PKG_VERSION`, réduits à `[0-9A-Za-z.-]` (sinon `inconnue`) ;
dossier `0700`, fichier `0600`, jamais d'écrasement, `.partial` pendant l'écriture. (T1, T10.)

**AC-4 — L'échec refuse la migration.** Dossier non créable ou illisible, espace sous le seuil, lecture de la base en
échec, écriture en échec, relecture en échec, version de départ illisible ou `< 0.15.0` : Kesh écrit le message de refus
et sort en code non nul **sans appliquer aucune migration** (`_sqlx_migrations` inchangée), sans laisser de `.partial`
qu'il a créé. (T5, T6, T6-bis, T8, T12.)

**AC-5 — L'échappatoire.** `KESH_SKIP_PRE_UPGRADE_BACKUP` égale à la version du binaire : aucune sauvegarde, migration
appliquée, avertissement écrit. Toute autre valeur : avertissement « ignorée », sauvegarde écrite comme sans elle.
Transmise par les deux compose, présente à `.env.example`, garde `configuration_transmise` verte. (T7, T8-bis, T14.)

**AC-6 — Pas de sauvegarde quand il n'y a rien à migrer.** *Neuve*, *à jour*, *interrompue* : aucun fichier, aucun
dossier `pre-upgrade/` créé ; *interrompue* écrit le message dédié. (T2, T3, T4.)

**AC-7 — Rétention.** Après une sauvegarde réussie et relue, `pre-upgrade/` garde les trois plus récentes (par nom) des
fichiers réguliers au nom exact de l'expression, jamais la nouvelle ; un autre fichier, un lien, un `.partial` récent n'y
sont pas touchés ; un échec ne supprime rien. (T9.)

**AC-8 — Le journal.** Les lignes du § *Messages*, avec les valeurs réelles ; `warn` si des pièces référencées manquent
(la sauvegarde reste écrite, la migration se fait) ou si le fichier dépasse `KESH_ADMIN_IMPORT_MAX_MB`. (Assertions sur la sortie du binaire, T8 et T8-bis — `tracing-test` n'est pas une dépendance de `kesh-api` ; les
lignes de succès, qui exigent un démarrage complet, se contrôlent à la recette AC-11, sortie copiée au Dev Agent Record.)

**AC-9 — L'écran (lot B).** *Administration → Sauvegarde complète*, *Sauvegardes sur le serveur* : les fichiers de
`pre-upgrade/` apparaissent, type « Avant mise à jour » (quatre locales), téléchargeables par
`GET /api/v1/admin/backups/pre-upgrade/{nom}` (audit `admin.backup_downloaded`, `kind = "pre-upgrade"`) ; un nom hors
expression rend 404. (T11, T13, captures.)

**AC-10 — Le manuel et le CHANGELOG.** § *Manuel* et § *CHANGELOG* ci-dessous appliqués au `.tex` **et** au PDF
régénéré ; aucune phrase restée vraie seulement avant la story (« rien ne la fait », « sans dire comment »). (Contrôle
`pdftotext`.)

**AC-11 — La recette de restauration est jouée** (C-v0174m-10, C-v0174m-11) : base de démonstration jetable sous
`gcorbaz/kesh:0.17.2` (avec au moins une facture fournisseur à document et une facture émise à PDF figé) → image de la
branche : sauvegarde écrite, migration appliquée → `gcorbaz/kesh:0.17.2` refusée sur la base migrée (message relevé) →
procédure du manuel (vidage de sécurité, base recréée vide, `/setup`, « Restaurer une sauvegarde » avec le fichier
`pre-upgrade/…`) → la 0.17.2 redémarre, l'écriture, la facture, le document et le PDF sont revenus. Chaque commande et
chaque sortie au Dev Agent Record ; ce que la recette n'établit pas, écrit (notamment : les droits de `kesh` après
`DROP DATABASE`, à constater).

**AC-12 — Aucune migration.** Aucun fichier sous `crates/kesh-db/migrations/` ; `kesh_version_min_required` inchangé ;
inventaire `TABLES_TO_TRUNCATE` inchangé.

## Tests et mutations

Tests rouges **d'abord** (constatés et relevés au Dev Agent Record), puis le correctif. Les tests sur base partielle
suivent `crates/kesh-db/tests/common/mod.rs` (`#[sqlx::test(migrations = false)]`, `apply_migrations_up_to`, index par
**version** via `migrations_before`, garde P6) et s'inscrivent à la liste de `crates/kesh-db/tests/test_schema_guard.rs`
(ils montent le vrai `MIGRATOR`). Fichier proposé : `crates/kesh-api/tests/sauvegarde_avant_migration.rs` (+ helper
`common/binaire.rs` pour T8).

| Test | Montage | Assertion |
|---|---|---|
| **T1** | base arrêtée avant `20261027000001` (version résolue par `migrations_before`), peuplée en SQL brut : société, utilisateur, compte, une écriture, une `imported_supplier_invoices` dont le document existe dans un dossier temporaire, une facture à `pdf_storage_path` ; `KESH_ADMIN_BACKUP_DIR` temporaire | **montage** : le plan rend `AAppliquer` avec un nombre > 0 ; l'issue est `Ecrite` ; le fichier existe sous `pre-upgrade/`, son nom satisfait l'expression et porte les deux versions ; le manifeste : ensemble des tables = tables sans `_` de `information_schema` de cette base, sans `mcp_prompts` ni `mcp_prompt_versions`, `rowCount` exacts, `files` porte les deux pièces ; puis `MIGRATOR.run` réussit |
| **T1-bis** | même base | `export_installation` (inventaire du binaire) y échoue en `Invariant` : documente la prémisse ; **mutation M4** le prouve côté production |
| **T2** | base entièrement migrée | `AJour`, aucun fichier, `pre-upgrade/` non créé |
| **T3** | base vide (aucune table) | `Neuve`, aucun fichier |
| **T4** | base partielle + une ligne `_sqlx_migrations` `success = 0` insérée | `Interrompue`, aucun fichier |
| **T5** | `KESH_ADMIN_BACKUP_DIR` désigne un fichier régulier | `Err(Refus::Dossier)`, rien d'écrit |
| **T6** | `BackupEnv` simulé, espace libre sous le seuil | `Err(Refus::Espace { libre, seuil })`, aucun `.partial` |
| **T6-bis** | `BackupEnv` dont l'écrivain tronque l'archive | `Err(Refus::Relecture)`, fichier supprimé |
| **T7** | skip = version du binaire / skip = `0.0.1` | `Contournee` sans fichier / `Ecrite` |
| **T8** (binaire) | base partielle ; `KESH_ADMIN_BACKUP_DIR` = fichier ; `KESH_PORT` occupé par un `TcpListener` du test (pour que le binaire sorte même si le refus manque) ; `DATABASE_URL` = celle des tests sur la base éphémère (`SELECT DATABASE()`) | code non nul ; la sortie contient `mise à jour refusée` ; `SELECT COUNT(*) FROM _sqlx_migrations` **inchangé** |
| **T8-bis** (binaire) | même montage, skip = version du binaire | la sortie contient l'avertissement ; `_sqlx_migrations` porte toutes les migrations ; sortie par l'échec de `bind` |
| **T9** | `pre-upgrade/` avec quatre sauvegardes au nom valide, un fichier quelconque, un lien | après une écriture : trois au nom valide (dont la nouvelle), le fichier et le lien intacts ; écriture en échec : rien supprimé |
| **T10** | fonction pure du nom | format exact ; `0.17.0-rc1` gardé ; `1.0.0+build/x` réduit ; vide → `inconnue` |
| **T11** | `lister_les_sauvegardes`, route de téléchargement (`admin_auto_backup_e2e.rs`) | le type `pre-upgrade` listé et téléchargé, audit `kind = "pre-upgrade"` ; nom hors expression → 404 ; `pre-upgrade/` illisible → liste des autres servie |
| **T12** | `last_applied` = `0.14.0` / `pas-semver` | `Err(Refus::VersionDeDepart)` |
| **T13** (Vitest) | `AutoBackupSection` | une ligne `kind = 'pre-upgrade'` affiche « Avant mise à jour » |
| **T14** | `configuration_transmise` | verte avec la variable neuve (compose, `.env.example`, manuel) |
| **T15** | garde de convention (`kesh-db`, base squash) | toute table de `information_schema` hors `TABLES_TO_TRUNCATE` commence par `_` ; aucune de `TABLES_TO_TRUNCATE` ne commence par `_` |

Mutations (jouées sous verrou, restaurées par `git checkout --` puis `touch`, `mutation-restauree-binaire-perime.md`) :

| Mutation | Tuée par |
|---|---|
| **M1** l'appel retiré de `main.rs` | T8 (`_sqlx_migrations` change) |
| **M2** `main.rs` journalise le refus et continue | T8 |
| **M3** le plan rend toujours `AJour` | T1 |
| **M4** l'inventaire lu dans `TABLES_TO_TRUNCATE` | T1 (échec `Invariant`) |
| **M5** `Neuve` traitée comme `AAppliquer` | T3 |
| **M6** la branche `Interrompue` retirée | T4 |
| **M7** l'échappatoire acceptée sur `is_some()` | T7 (valeur `0.0.1`) |
| **M8** rétention à quatre / qui supprime la nouvelle | T9 |
| **M9** contrôle d'espace retiré | T6 |
| **M10** relecture retirée | T6-bis |
| **M11** `PreUpgrade` absent de la liste | T11 |
| **M12** le libellé retombe sur « Avant import » | T13 |
| **M13** contrôle de version de départ retiré | T12 |
| **M14** préfixe `_` non exclu de l'inventaire | T1 (`_kesh_version` au manifeste) |

## Tâches

### Lot A — la sauvegarde avant migration

1. **Étape 0** : relire les prémisses du § *Ce que le code dit* au code (lignes citées au Change Log) ; recréer
   `kesh_v0174m` / `kesh_e2e_v0174m` à neuf (migrations du worktree, `scripts/seed-dev-db.sql`).
2. T1 à T10, T12, T15 en rouge ; relever les rouges.
3. `kesh-db` : `export_installation_schema_present`, `InstallationExport.tables` à noms possédés, T15.
4. `kesh-api` : module `pre_upgrade_backup` (plan, échappatoire, version de départ, dossier, espace, lecture, écriture,
   relecture, rétention, journal), `Config::skip_pre_upgrade_backup`, appel dans `main.rs` (doc-comment d'en-tête
   `main.rs:1-29` mis à jour : étape 3c).
5. Compose (deux fichiers), `.env.example`, `configuration_transmise.rs`.
6. Mutations M1 à M10, M13, M14.

### Lot B — l'écran

7. `BackupKind::PreUpgrade`, `NOM_PRE_UPGRADE`, liste ; T11, M11.
8. Frontend (`backup-schedule.api.ts`, `AutoBackupSection.svelte`) ; clé i18n quatre locales (replis = FTL fr-CH) ;
   T13, M12 ; `npm run lint-i18n-ownership`.
9. Captures (thème clair, base de démonstration jetable) : la liste avec une ligne « Avant mise à jour » ; dans
   `/home/gcorbaz/devel/kesh-gate-logs/v0174m/captures/`.

### Commun

10. Manuel d'administration `.tex` + PDF (`make fr` sous les deux verrous) ; contrôle `pdftotext` aplati.
11. CHANGELOG `[0.17.4] — Non publié`, *Ajouté*.
12. Recette AC-11 (images Docker sous les deux verrous, `docker run` sur un réseau et des volumes jetables, jamais
    `kesh-mariadb-dev`).
13. Gate complet (`test-fast.sh --ci`), Vitest, build, E2E complet au dernier commit de code ; grep du symptôme :
    `git grep -nE "rien ne la fait|discipline de l'administrateur|sans dire comment"` sur docs et code.

## Manuel

`docs/manual/fr/admin-manual.tex`, puis PDF. Le texte ci-dessous est la substance ; la forme suit le manuel.

1. **Nouvelle sous-section** sous § *Sauvegarde et restauration*, après § *La sauvegarde automatique* :
   `\subsubsection{La sauvegarde avant mise à jour}\label{sec:sauvegarde-avant-maj}` — Depuis la 0.17.4, au démarrage
   d'une version qui doit migrer la base, Kesh écrit d'abord une sauvegarde complète (base et pièces, format 2) dans
   `pre-upgrade/` du dossier de sauvegarde (`./backup/pre-upgrade`, `/volume1/docker/kesh/backup/pre-upgrade`), nommée
   `kesh-pre-upgrade-AAAAMMJJTHHMMSSmmmZ-<de>-to-<vers>.keshbackup` (heure UTC). Ce qu'elle contient (les tables du schéma
   **de la version de départ**, importable par elle) ; quand elle ne s'écrit pas (installation neuve, aucune migration —
   la plupart des redémarrages et les versions sans migration —, migration interrompue) ; pendant l'écriture, le serveur
   ne répond pas (`/health`, conteneur « unhealthy » un moment : attendre le journal) ; durée et taille (≈ base +
   documents ; l'espace requis comme la sauvegarde automatique) ; rétention (les trois plus récentes) ; la liste du
   serveur (type « Avant mise à jour », téléchargeable) ; ce qu'elle ne remplace pas (même disque : la copier ailleurs ;
   ni le `.env`) ; **si elle échoue**, Kesh ne migre pas et le dit — la base est intacte, l'ancienne image redémarre ;
   l'échappatoire `KESH_SKIP_PRE_UPGRADE_BACKUP=<version exacte>`, à poser dans le `.env` **et** sous `environment:` du
   compose si le vôtre est antérieur à la 0.17.4, à retirer ensuite ; depuis une version antérieure à la 0.15.0, refus
   (vidage manuel de la base, puis échappatoire).
2. **§ Procédure de mise à jour standard**, étape 1 : la sauvegarde manuelle reste recommandée (elle est hors du disque
   si on la copie ailleurs ; elle couvre le `.env`) ; depuis la 0.17.4, Kesh écrit lui-même la sauvegarde avant
   migration (renvoi). Le `keshwarning` est récrit dans ce sens. **Ne pas réécrire les § *Passer à la 0.x* existants** :
   ils décrivent des versions qui n'ont pas cette sauvegarde (une mise à jour **vers** la 0.17.3 ou avant n'en écrit pas).
3. **§ Rollback en cas d'échec** → renommé ou complété *Revenir à la version précédente*
   (`\label{sec:revenir-version-precedente}`, cible des messages du journal) : (1) `docker compose stop kesh-api` ;
   (2) copier le fichier `pre-upgrade/…` hors du dossier ; (3) **vidage de sécurité** de la base migrée (Synology : `kesh-dump.sh`, § *backup-dsm* ; compose standard :
   `mariadb-dump` par le conteneur, § *Script de sauvegarde mariadb-dump*, `sec:script-sauvegarde`) — la garde G16 de
   `configuration_transmise` l'exige des sections Synology avant tout rechargement ; (4) recréer la base vide par le compte
   root du conteneur MariaDB (`DROP DATABASE kesh; CREATE DATABASE kesh CHARACTER SET utf8mb4 COLLATE
   utf8mb4_unicode_ci;` — commande exacte et effet sur les droits de `kesh` **constatés en recette**) ; (5) remettre
   l'image précédente dans le compose, `docker compose up -d` ; (6) `/setup`, puis « Restaurer une sauvegarde » de
   l'assistant avec le fichier (renvoi § *Restaurer sur une nouvelle machine*, dont le plafond d'import) ; le dossier des
   documents reste en place. Garder le paragraphe existant sur la réparation de l'installation.
4. **§ La sauvegarde automatique**, paragraphes *Volume* et *Les sauvegardes du serveur* : trois types désormais ; *Quelle
   méthode choisir ?* : une ligne au tableau.
5. **§ Variables d'environnement complètes** (`sec:env-vars`) : `KESH_SKIP_PRE_UPGRADE_BACKUP`.
6. Le manuel utilisateur n'est pas concerné (écran d'administration) ; le vérifier par
   `git grep -n "Avant import" docs/manual/fr/user-manual.tex`.

## CHANGELOG

Sous `## [0.17.4] — Non publié` (à créer en tête si absente ; union avec les deux autres stories), `### Ajouté` :

- **Kesh se sauvegarde lui-même avant de mettre la base à jour** ([#847](https://github.com/guycorbaz/kesh/issues/847)) :
  au démarrage d'une version qui doit migrer la base, une sauvegarde complète — base et pièces — est écrite dans
  `backup/pre-upgrade/` avant toute migration, importable par la version de départ ; si elle échoue, Kesh ne migre pas
  et le dit (la base reste intacte). Les trois plus récentes sont gardées ; elles se téléchargent depuis *Administration
  → Sauvegarde complète*. La procédure de retour à la version précédente est au manuel d'administration. Cette
  sauvegarde ne s'écrit qu'à une mise à jour **vers** une version qui porte une migration : celle de la 0.17.3 vers la
  0.17.4 n'en écrit [pas | une, si la 0.17.4 porte une migration — à fixer à l'intégration]. La sauvegarde manuelle avant
  mise à jour reste recommandée (hors du disque, avec le `.env`).

## Captures

Thème clair seulement, base de démonstration jetable : la liste *Sauvegardes sur le serveur* avec une ligne « Avant mise
à jour » à côté d'une automatique et d'une pré-import (fr-CH ; une seconde en de-CH si `KESH_LANG=de` est simple à
poser). Jointes au Dev Agent Record et au prompt de la revue de code.

## Dev Notes

- **Verrous de station** : toute commande lourde (`test-fast.sh`, build, nextest, clippy du workspace, Vitest, `npm run
  build`, E2E, `make` des manuels, `docker run` d'une image, mutations) par
  `flock /tmp/station-heavy.lock flock /home/gcorbaz/devel/kesh-gate-logs/.gate.lock scripts/mem-guard.sh …`, une
  commande par prise, `free -g` avant (sous 12 Go disponibles : attendre).
- **Mémoire** : `export_installation` charge toutes les tables en mémoire avant l'écriture (angle mort existant de toutes
  les sauvegardes, non traité ici) ; la sauvegarde avant migration a le même coût que la sauvegarde automatique. Ne pas
  l'aggraver (une seule lecture).
- **Mono-instance** : comme la sauvegarde automatique, rien ne protège deux processus Kesh sur une même base ; le verrou
  `backup_lock` n'existe pas encore à ce stade (pas d'`AppState`) et n'est pas nécessaire (rien d'autre ne tourne).
- **Boucle de redémarrage** : avec `restart: unless-stopped`, un refus fait redémarrer le conteneur en boucle, chaque
  tentative refaisant la sauvegarde puis échouant ; c'est attendu, le journal le dit, et un échec ne laisse pas de
  `.partial` créé par lui (`write_backup_file`). Une migration qui échoue après une sauvegarde réussie laisse une ligne
  « sale » : la tentative suivante n'écrit plus de sauvegarde (AC-6), si bien que la rétention ne peut pas évincer la
  bonne.
- **Splitting** : six modules au sens de la règle ; dérogation C-v0174m-12, deux lots, déclarée au Project Lead.
- **Données réelles** : aucune ; `~/kesh/`, `~/kesh-test/`, le NAS interdits.

## Pour l'orchestrateur

- **Issue à ouvrir** (on trace toujours) : aucune à la rédaction. Angle mort existant rappelé, sans issue nouvelle
  demandée ici : la sauvegarde charge les tables en mémoire (à vérifier : une issue le porte-t-elle déjà ?).
- **#847** : `closes #847` sur la PR de release, `refs #847` dans les commits.
- **Commentaire à #847** : le point de conception est tranché par C-v0174m-1 (`.keshbackup` à l'inventaire du schéma
  présent ; le vidage SQL écarté) ; la restauration passe par une base recréée vide et l'assistant.
- **Intégration** : si `v0174-session` ajoute une migration, fixer la phrase entre crochets du CHANGELOG et rejouer la
  recette de 0.17.3 à 0.17.4.

## Change Log

- 2026-11-05 — Fiche créée (Opus 5.5, autonomie) : prémisses lues au code à `7bd4d5c0` ; conception C-v0174m-1 à 14 ;
  prochaine étape : validation P1 (Sonnet, lentilles R et F, `v0174-mise-a-jour-validate-prompt-p1.md`).

## Dev Agent Record

### Agent Model Used

### Debug Log References

### Completion Notes List

### File List
