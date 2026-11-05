# Story v0174-mise-a-jour-1 : la 0.17.4 — la sauvegarde avant migration, version minimale (#847)

Status: ready-for-dev

*Seule sous-fiche à développer de `v0174-mise-a-jour.md` (parente en `split`, qui garde l'analyse du code ; la sœur
`v0174-mise-a-jour-2.md` est `superseded`). Récrite le 2026-11-05 en **version minimale**, sur décision du Project Lead
après la non-convergence de la validation (P1 2 HIGH → P2 1 HIGH → P3 1 HIGH, tous recyclés) — C-v0174m-30. Worktree
`/home/gcorbaz/devel/kesh-0174m`, branche `story/v0174-mise-a-jour` partie de `7bd4d5c0` ; port **3178**, bases
`kesh_v0174m` / `kesh_e2e_v0174m`. Registre `v0174-choix-autonomes-v0174m.md` (choix en vigueur : § *Choix en vigueur après
la décision du Project Lead*). Numéros de ligne de `7bd4d5c0`. Aucune migration (C-v0174m-13).*

## Story

En tant qu'**administrateur** d'une installation Kesh qui tient des livres réels, je veux que Kesh, au démarrage d'une
version qui doit migrer la base, écrive lui-même, avant toute migration, une sauvegarde complète de l'état d'avant — base
**et** pièces — que la version de départ sait restaurer, et qu'il refuse de migrer si cette sauvegarde échoue.

## Ce qui est fait (et rien d'autre)

1. **Quand** : au démarrage, après `check_downgrade_protection` (`main.rs:122-154`) et avant `MIGRATOR.run`
   (`main.rs:157`), **s'il reste des migrations à appliquer**. Le plan se lit dans `_sqlx_migrations` contre
   `MIGRATOR.iter()` (montantes) : erreur 1146 ou table vide → *neuve* ; une ligne `success = 0` → *interrompue* ; toutes
   présentes → *à jour* ; sinon *à appliquer*. Seul *à appliquer* écrit une sauvegarde. **Toute autre erreur de lecture du
   plan → refus.** `check_downgrade_protection` = `FreshInstall` → rien.
2. **Quoi** : un `.keshbackup` de format 2, pièces comprises, à l'**inventaire du schéma présent** — les tables de
   `information_schema.TABLES` de `DATABASE()` dont le nom ne commence pas par `_` (même prédicat que la garde
   `backup_inventory_matches_schema`, `backup.rs:848`), filtrées en Rust, triées (C-v0174m-1, C-v0174m-24) — pour que la
   version de départ, dont c'est exactement le `TABLES_TO_TRUNCATE`, l'importe.
3. **Où, sous quel nom** : `<KESH_ADMIN_BACKUP_DIR>/pre-upgrade/kesh-pre-upgrade-<AAAAMMJJTHHMMSSmmm>Z-<de>-to-<vers>.keshbackup`
   (UTC ; `<de>` = `_kesh_version.kesh_version_last_applied`, `<vers>` = `CARGO_PKG_VERSION` ; caractères hors
   `[0-9A-Za-z.-]` retirés, vide → `inconnue`, 40 caractères au plus). Dossier créé en `0700`.
4. **Comment** : écriture par `write_backup_file` (`routes/admin.rs:754` : `<nom>.partial` en `create_new` `0600`,
   `sync_all`, renommage sans écrasement) après un contrôle d'espace (`seuil_d_espace`, `auto_backup.rs:510`, avec la plus
   grande sauvegarde de `auto/` et de `pre-upgrade/` et la taille des documents, contre `BackupEnv::free_bytes`).
5. **Échec** : dossier non créable, espace insuffisant ou illisible, lecture de la base, écriture, panique de la tâche
   d'écriture (`BackupError::Task`) → message de refus, `exit(1)`, **aucune migration appliquée**. Un `.partial` laissé par
   une panique ou un arrêt brutal est inoffensif (jamais listé, jamais importé) : le manuel le dit, rien ne l'exige
   nettoyé.
6. **Échappatoire** : `KESH_SKIP_PRE_UPGRADE_BACKUP=<version exacte du binaire>` dans le `.env`, transmise par
   `docker-compose.yml` et `docker-compose.prod.yml` (`${KESH_SKIP_PRE_UPGRADE_BACKUP:-}`). Évaluée **en premier** (après
   `FreshInstall`) : égale → avertissement, rien d'écrit, la migration se fait ; autre valeur → avertissement « ignorée »,
   la sauvegarde se fait (C-v0174m-31).
7. **Jamais de suppression automatique** : chaque tentative écrit son fichier ; ils se retirent à la main (C-v0174m-25).
8. **Journal** : une ligne au début (`Sauvegarde avant mise à jour en cours …`), une à la fin avec le **chemin** du fichier
   écrit, le refus, les deux avertissements de l'échappatoire.

**Retiré** (décision du Project Lead, C-v0174m-30) : le type « Avant mise à jour » à l'écran (sous-fiche 2, `superseded`,
issue à ouvrir) ; la règle détaillée de choix du fichier, E1 et tout message après `MIGRATOR.run` ; la garde de format
T18 / `TYPES_DECODES` ; `pub mod admin_backup` (les tests sont internes) ; le refus sur version de départ `< 0.15.0` ;
la relecture ; T6-quater et l'exigence de nettoyage du `.partial` sur panique ; les avertissements W1 à W3 ; les deux jambes
et deux topologies de recette (une seule jambe).

## Conception

- **`crates/kesh-api/src/pre_upgrade_backup.rs`**, `pub mod` dans `lib.rs` (appelé par `main.rs`, binaire distinct) ;
  point d'entrée `pub async fn sauvegarder_avant_migration(pool, &Config, &DowngradeCheckOutcome, binaire: &str,
  environnement: Arc<dyn BackupEnv>, now: NaiveDateTime) -> Result<Issue, Refus>` ;
  `Issue { InstallationNeuve, AJour, Interrompue, Contournee, Ecrite(PathBuf) }`,
  `Refus { Plan, Dossier, Espace, Lecture, Ecriture, Tache }` (chacun avec l'erreur d'origine). `main.rs` : `Err` →
  `tracing::error!` du refus, `exit(1)` ; `Ok` → étape 4. Doc-comment d'en-tête de `main.rs` complété (étape 3c).
- **`kesh-db/src/backup.rs`** : `export_installation_schema_present(pool)` — même `ReadSnapshot`, cœur
  `export_installation_in` rendu générique sur la liste des tables, liste lue **dans l'instantané** ;
  `InstallationExport.tables` et `Etape::ApresTable` à noms possédés (`String`) ; `KeshBackupMeta.table_count` =
  `export.tables.len()` (`export.rs:330`). `build_keshbackup` (`export.rs:148`) reçoit `Inventaire { Binaire,
  SchemaPresent }`. Garde lexicale `export_installation_n_a_qu_un_appelant` (`export.rs:489`) : une garde par fonction.
- **Import à inventaire paramétré** (pour le test d'aller-retour seulement) : `parse_and_verify` (`import.rs:172`) délègue
  à `pub(crate) fn parse_and_verify_avec_inventaire(reader, limits, inventaire)` ; `restore_tables_in_tx` (`backup.rs:625`)
  à `pub fn restore_tables_in_tx_avec_inventaire(tx, tables, inventaire)` (ordre de `TABLES_TO_TRUNCATE` filtré,
  exclusions `onboarding_state` et `audit_log` inchangées). Les appelants de production (`routes/admin.rs:430`) ne changent
  pas.
- **Visibilités** `pub(crate)` dans `auto_backup.rs` : `seuil_d_espace`, `taille_des_documents`, `fichiers_reguliers` ;
  `ensure_backup_dir` (`routes/admin.rs:143`) inchangée si le module crée son dossier lui-même (`DirBuilder`, `0700`).
- **Configuration** : `skip_pre_upgrade_backup: Option<String>` par `env_nonempty` ; ligne sous `environment:` de
  `kesh-api` dans les deux compose ; ligne **commentée** à `.env.example` ; `SANS_DEFAUT` et `EMPLACEMENTS_AUTORISES` de
  `crates/kesh-api/tests/configuration_transmise.rs` (vide = absent = sauvegarde faite).

### Messages (texte exact)

- **Début** (`info`) : `Mise à jour de la base : {n} migration(s) à appliquer (de la {de} à la {vers}). Sauvegarde complète avant mise à jour en cours ; le serveur répondra une fois la mise à jour terminée.`
- **Fin** (`info`) : `Sauvegarde avant mise à jour écrite : {chemin} ({taille} octets).`
- **Refus** (`error`) : `FATAL : mise à jour refusée — la sauvegarde avant mise à jour n'a pas pu être écrite ({cause}). Aucune migration n'a été appliquée par cette tentative. Corrigez la cause, ou, en connaissance de cause, posez KESH_SKIP_PRE_UPGRADE_BACKUP={vers} dans le .env pour migrer sans sauvegarde (le compose doit la transmettre ; manuel d'administration, « La sauvegarde avant mise à jour »).` — `{cause}` : `état des migrations illisible : {e}` · `dossier {dir} : {e}` · `espace insuffisant dans {dir} ({libre} Mio libres, {seuil} Mio requis)` ou `espace libre de {dir} illisible : {e}` · `lecture de la base : {e}` · `écriture de {chemin} : {e}` · `tâche interrompue : {e}`. Jamais `DATABASE_URL`.
- **Échappatoire** (`warn`) : `KESH_SKIP_PRE_UPGRADE_BACKUP={vers} : migration SANS sauvegarde préalable, à la demande de l'administrateur. Retirez cette variable après la mise à jour.` · `KESH_SKIP_PRE_UPGRADE_BACKUP={valeur} ignorée : elle ne vaut que pour la version {vers}.`

L'échappatoire étant évaluée avant le plan, le conseil du refus vaut pour toutes les causes, plan illisible compris.

## Acceptance Criteria

**AC-1 — Pas de sauvegarde sans migration.** Base neuve (`FreshInstall`, ou plan *neuve*), base *à jour*, base
*interrompue* : aucun fichier, aucun dossier `pre-upgrade/` créé. (T2, T3, T4.)

**AC-2 — Sauvegarde avant migration.** Base où reste au moins une migration : le fichier est écrit sous le nom du point 3,
ses tables sont exactement celles du schéma présent sans préfixe `_`, ses pièces y sont ; la migration suit. (T1.)

**AC-3 — Importable par la version de départ.** Sur une base arrêtée avant `20261027000001_mcp_prompts` (migration en
attente qui crée des tables applicatives), le fichier passe `parse_and_verify_avec_inventaire` avec l'inventaire de
départ **littéral**, `check_schema_compat` contre une seconde base au même état, et se restaure dans celle-ci avec les
mêmes lignes ; l'import à l'inventaire du binaire le **refuse**. (T1-ter.)

**AC-4 — Un échec refuse la migration.** Plan illisible, dossier, espace, lecture, écriture : refus, code de sortie non
nul, `_sqlx_migrations` inchangée. (T5, T6, T7, T8, T9, T10.)

**AC-5 — L'échappatoire.** Valeur égale à la version du binaire : aucun fichier, avertissement ; autre valeur :
avertissement « ignorée », fichier écrit. Transmise par les deux compose ; garde `configuration_transmise` verte. (T11,
T12.)

**AC-6 — Aucune suppression.** Deux exécutions successives : deux fichiers. (T13.)

**AC-7 — Manuel et CHANGELOG** appliqués (`.tex` et PDF régénéré) ; contrôle du § *Manuel*, point 6.

**AC-8 — Recette** (une jambe, compose standard, base de démonstration jetable avec une écriture, une facture fournisseur
à document et une facture émise à PDF figé) : `gcorbaz/kesh:0.15.0` → image de la branche (fichier écrit, migrations
appliquées, lignes de journal relevées) → `gcorbaz/kesh:0.15.0` refusée sur la base migrée → procédure du manuel → la 0.15.0
redémarre, données et pièces revenues. Commandes et sorties au Dev Agent Record. La topologie Synology n'est pas jouée : le
manuel le dit (« procédure non éprouvée sur Synology »).

**AC-9 — Aucune migration** : 89 fichiers `.sql` avant et après ; `TABLES_TO_TRUNCATE` et `min_required` inchangés.

## Tests et mutations

Rouges d'abord, relevés. **Tests internes** dans `crates/kesh-api/src/pre_upgrade_backup/tests.rs` (`#[cfg(test)] mod
tests;`), qui incluent les helpers de montage de `kesh-db` par
`#[path = "../../../kesh-db/tests/common/mod.rs"] mod montage;` (`apply_migrations_up_to`, `migrations_before` : index par
**version**, P6) ; fichier inscrit à `ALLOWED_REAL_MIGRATOR_FILES` de `crates/kesh-db/tests/test_schema_guard.rs` (la garde
balaie tout `crates/kesh-api/`, `src/` compris ; attribut `#[sqlx::test(migrations = false)]` sur une ligne). Le test
binaire T8 vit dans `crates/kesh-api/tests/sauvegarde_avant_migration.rs` (helper `common/binaire.rs`).

**Montage des bases partielles** : `apply_migrations_up_to(&pool, migrations_before(20261027000001, "mcp_prompts"))`, puis
`UPDATE _kesh_version SET kesh_version_last_applied = '0.17.3' WHERE id = 1` (la base porte sinon `'0.1.0'`), puis les
données en SQL brut ; assertion de montage : le plan rend *à appliquer* avec `MIGRATOR.migrations.len() − n` migrations.

| Test | Montage | Assertion |
|---|---|---|
| **T1** | base partielle ; société, utilisateur, compte, une écriture, une `imported_supplier_invoices` dont le document existe dans un dossier temporaire, une facture à `pdf_storage_path` | `Ecrite(chemin)` ; nom conforme, `0.17.3-to-<binaire>` ; tables du manifeste = `TABLES_TO_TRUNCATE` privé de `mcp_prompt_versions` et `mcp_prompts` ; les deux pièces dans `files` ; puis `MIGRATOR.run` réussit |
| **T1-ter** | fichier de T1 ; seconde base au même état partiel | `parse_and_verify_avec_inventaire(…, inventaire littéral)` OK ; `check_schema_compat` OK ; restauration par `restore_tables_in_tx_avec_inventaire` ; chaque table de l'inventaire : `SELECT *` brut identique aux deux bases (`onboarding_state` exclue, `audit_log` sans `id`) ; `parse_and_verify` (inventaire du binaire) refuse : `table 'mcp_prompt_versions' absente du manifeste` |
| **T2** | base entièrement migrée | `AJour`, `pre-upgrade/` absent |
| **T3** | base vide : `plan_de_migration` direct, puis point d'entrée avec `Aligned` et avec `FreshInstall` | *neuve* ; `InstallationNeuve` deux fois ; rien d'écrit |
| **T4** | base partielle + ligne `success = 0` | `Interrompue`, rien d'écrit |
| **T5** | `KESH_ADMIN_BACKUP_DIR` = fichier régulier | `Refus::Dossier` |
| **T6** | `BackupEnv` : espace libre sous le seuil | `Refus::Espace`, rien d'écrit |
| **T7** | écrivain qui rend `StorageFull` | `Refus::Ecriture` |
| **T8** (binaire) | base partielle ; dossier = fichier ; `KESH_PORT` occupé ; `DATABASE_URL` (base éphémère), `KESH_JWT_SECRET`, `KESH_HOST=127.0.0.1`, `KESH_PORT`, `KESH_ADMIN_BACKUP_DIR`, `KESH_LOCALES_DIR` absolu | code non nul ; sortie contient `mise à jour refusée` et `dossier` ; `COUNT(*)` de `_sqlx_migrations` inchangé |
| **T9** | plan sur un pool fermé | `Refus::Plan` |
| **T10** | étape d'écriture (`ecrire_la_sauvegarde`) appelée avec le pool fermé après un plan réussi | `Refus::Lecture` |
| **T11** | skip = version du binaire / `0.0.1` | `Contournee` sans fichier / `Ecrite` |
| **T12** | `configuration_transmise` | verte (variable dans `SANS_DEFAUT`, forme `${NOM:-}` dans les deux compose, ligne commentée) |
| **T13** | deux appels successifs (`now` avancé d'une milliseconde) | deux fichiers présents |
| **T14** | fonction pure du nom | format ; `0.17.0-rc1` gardé ; `1.0.0+build/x` → `1.0.0buildx` ; `+++` → `inconnue` ; 41 caractères → 40 |

| Mutation | Tuée par |
|---|---|
| **M1** l'appel retiré de `main.rs` | T8 (`_sqlx_migrations` change) |
| **M2** `main.rs` journalise le refus et continue | T8 |
| **M3** le plan rend toujours *à jour* | T1 |
| **M4** l'inventaire lu dans `TABLES_TO_TRUNCATE` | T1, T1-ter |
| **M5** préfixe `_` non exclu | T1 |
| **M6** *neuve* traitée comme *à appliquer* | T3 |
| **M7** la branche *interrompue* retirée | T4 |
| **M8** plan en erreur traité comme *neuve* | T9 |
| **M9** contrôle d'espace retiré | T6 |
| **M10** `Err(Write)` converti en `Ecrite` | T7 |
| **M11** `Err(Db)` de la lecture ignoré | T10 |
| **M12** l'échappatoire acceptée sur `is_some()` | T11 (`0.0.1`) |
| **M13** `restore_tables_in_tx_avec_inventaire` ignore l'inventaire | T1-ter |
| **M14** une suppression de sauvegarde ajoutée après l'écriture | T13 |

## Tâches

1. Étape 0 : relire les prémisses de la parente (lignes citées au Change Log) ; recréer `kesh_v0174m` / `kesh_e2e_v0174m`.
2. Tests en rouge ; relever.
3. `kesh-db` : inventaire du schéma présent, noms possédés, `restore_tables_in_tx_avec_inventaire`.
4. `kesh-api` : module, `parse_and_verify_avec_inventaire`, `Config`, `main.rs`, visibilités, garde lexicale.
5. Compose (deux), `.env.example`, `configuration_transmise.rs`.
6. Mutations M1 à M14 (sous verrou, restaurées par `git checkout --` puis `touch`).
7. Manuel `.tex` + PDF (`make fr` sous les deux verrous) ; CHANGELOG.
8. Recette AC-8 (images sous les deux verrous, réseau et volumes jetables, jamais `kesh-mariadb-dev`).
9. Gate complet, Vitest, build, E2E complet au dernier commit de code.

## Manuel

`docs/manual/fr/admin-manual.tex`, puis PDF.

1. **Nouvelle `\subsubsection{La sauvegarde avant mise à jour}`** (`sec:sauvegarde-avant-maj`), après *La sauvegarde
   automatique* : depuis la 0.17.4, au démarrage d'une version qui doit migrer la base, Kesh écrit d'abord une sauvegarde
   complète (base et pièces) dans `pre-upgrade/` du dossier de sauvegarde (`./backup/pre-upgrade`,
   `/volume1/docker/kesh/backup/pre-upgrade`), au nom `kesh-pre-upgrade-AAAAMMJJTHHMMSSmmmZ-<de>-to-<vers>.keshbackup` ;
   le journal en donne le chemin. Elle se restaure dans la version de départ (une 0.15.0 ou plus récente). Pas de
   sauvegarde sans migration à appliquer (la plupart des redémarrages, toute version sans migration) ni sur une
   installation neuve. Pendant l'écriture le serveur ne répond pas. Si elle échoue, Kesh ne migre pas et le dit ;
   échappatoire `KESH_SKIP_PRE_UPGRADE_BACKUP=<version exacte>` dans le `.env`, transmise par les compose de la 0.17.4 —
   **un compose antérieur ne la transmet pas : ajoutez sous `environment:` du service `kesh-api` la ligne
   `KESH_SKIP_PRE_UPGRADE_BACKUP: ${KESH_SKIP_PRE_UPGRADE_BACKUP:-}`** ; à retirer après. **Kesh n'en supprime aucune** :
   retirez les anciennes à la main une fois la mise à jour réussie et éprouvée — tant qu'elle n'a pas réussi, n'en
   supprimez aucune. Après un refus ou une migration en échec, le conteneur (`restart: unless-stopped`) redémarre en
   boucle et réessaie à chaque tour : arrêtez-le (`docker compose stop kesh-api`) le temps de corriger (V3-4). Un fichier `.partial` laissé par un arrêt brutal n'est
   pas une sauvegarde : il peut être supprimé. C'est un secret, sur le même disque que Kesh : copiez-la ailleurs.
2. **§ Procédure de mise à jour standard**, étape 1 et `keshwarning` : la sauvegarde manuelle reste recommandée ; depuis
   la 0.17.4, Kesh écrit en plus la sienne (renvoi). Les § *Passer à la 0.x* existants ne sont pas réécrits.
3. **§ Rollback en cas d'échec** → **Revenir à la version précédente** (`sec:revenir-version-precedente`) :
   (1) récupérer le fichier `pre-upgrade/…` (SSH, `sudo cp` + `chown`, comme pour la pré-import, `:2329-2333`) — **si une
   tentative précédente a déjà migré en partie, restaurer le plus ancien fichier écrit pour cette mise à jour** ;
   (2) `docker compose stop kesh-api` ; (3) recréer la base vide par le **compte Kesh** — compose standard :
   ```text
   docker compose exec mariadb sh -c 'mariadb -u"$MARIADB_USER" -p"$MARIADB_PASSWORD" -e "DROP DATABASE \`$MARIADB_DATABASE\`; CREATE DATABASE \`$MARIADB_DATABASE\` CHARACTER SET utf8mb4 COLLATE utf8mb4_unicode_ci"'
   ```
   (forme jouée en recette) ; sur Synology, recréer la base comme le fait déjà § *backup-dsm* (`kesh-restore.cnf`,
   `DROP DATABASE`), par renvoi, sans nouvelle commande ici ; (4) remettre dans le compose l'image `<de>` du nom du fichier,
   épinglée, `docker compose up -d` ; (5) `/setup` (ou connexion si `KESH_ADMIN_USERNAME` est posé), puis « Restaurer une
   sauvegarde » de l'assistant (§ *Restaurer sur une nouvelle machine*) ; le dossier des documents reste en place. Ce que le
   retour perd : la saisie faite depuis la mise à jour ; les réglages hors fichier reviennent aux défauts. « Procédure non
   éprouvée sur Synology. » Garder le paragraphe sur la réparation de l'installation.
4. **`KESH_ADMIN_BACKUP_DIR`** (`:812`, `:2377`) : ajouter `pre-upgrade/`.
5. **§ Variables d'environnement complètes** : `KESH_SKIP_PRE_UPGRADE_BACKUP`.
6. **Contrôle** (PDF aplati : `pdftotext docs/manual/fr/admin-manual.pdf - | tr '\n' ' ' | tr -s ' '`), comptes avant
   (mesurés à `7bd4d5c0`) et après au Dev Agent Record : `Rollback en cas d.échec` (avant 3, après 0 ; le `.` pour
   l'apostrophe typographique) ; `La sauvegarde avant mise à jour` et `Revenir à la version précédente` (avant 0, après
   ≥ 1) ; `git grep -n "sauvegardes automatiques (sous-dossier" docs/manual/fr/admin-manual.tex` (avant 2 ; après, chaque
   ligne nomme aussi `pre-upgrade/`).

## CHANGELOG

Sous `## [0.17.4] — Non publié`, `### Ajouté` :

- **Kesh se sauvegarde lui-même avant de mettre la base à jour** ([#847](https://github.com/guycorbaz/kesh/issues/847)) :
  au démarrage d'une version qui doit migrer la base, une sauvegarde complète — base et pièces — est écrite dans
  `backup/pre-upgrade/` avant toute migration ; elle se restaure dans la version de départ (manuel d'administration,
  *Revenir à la version précédente*). Si elle échoue, Kesh ne migre pas et le dit ; pour migrer quand même, en
  connaissance de cause : `KESH_SKIP_PRE_UPGRADE_BACKUP=<version>` dans le `.env` (les compose de la 0.17.4 la
  transmettent ; un compose antérieur, non : reprenez-en la ligne). Kesh n'en supprime aucune. La 0.17.4 n'ayant pas de
  migration, la mise à jour depuis la 0.17.3 n'en écrit pas.

## Dev Notes

- **Verrous de station** : toute commande lourde par
  `flock /tmp/station-heavy.lock flock /home/gcorbaz/devel/kesh-gate-logs/.gate.lock scripts/mem-guard.sh …`, une par prise,
  `free -g` avant.
- **Mémoire** : la lecture charge les tables en mémoire, comme toute sauvegarde (#911).
- **Poste de dev** : une base montée par `sqlx migrate run` porte `last_applied = '0.1.0'` ; le nom du fichier dira alors
  `0.1.0-to-…` — sans effet sur la sauvegarde. Hors Docker, `KESH_ADMIN_BACKUP_DIR` doit être inscriptible.
- **Données réelles** : aucune ; `~/kesh/`, `~/kesh-test/` et le NAS interdits.

## Change Log

- 2026-11-05 — Sous-fiche créée à la remédiation de la validation P1 (Opus 5.5) de la fiche `v0174-mise-a-jour`
  (Sonnet, R 0/2/8/7, F 0/0/6/6). Traitement des findings (prémisse de chacun relue au code) :
  - **R1-1 / F1-3 (HIGH / MEDIUM)** : prémisse confirmée (`sqlx-mysql-0.8.6` `apply`, INSERT dans la transaction) ;
    traité par C-v0174m-15 (plus ancienne du couple protégée, message R-b, T9-bis, M18, M22). Correctif « ne jamais
    réécrire un couple » **rejeté** : la sauvegarde gardée serait périmée si la version de départ a servi entre deux
    tentatives (motif au registre).
  - **R1-2 (HIGH)** : confirmé (`docker-compose.prod.yml` sans service `mariadb`, `admin-manual.tex:2005-2010`,
    `kesh-restore.sh` étape 6) ; compte Kesh dans les deux topologies, recette en topologie de production (C-v0174m-19).
  - **R1-3 / F1-2 (plan)** : C-v0174m-16, T16, M16. **F1-2 (lecture, écriture)** et **R1-5** : T5-bis, T6-ter, T19, M15,
    M20, M21.
  - **R1-4 / F1-4** : C-v0174m-17 (garde T18, import à inventaire paramétré, T1-ter, M23) et C-v0174m-22 (jambe
    0.15.0, qui seule a un inventaire différent : prémisse vérifiée, 0.17.2 → 0.17.3 n'ajoute qu'une table système).
  - **R1-6** : T6 à deux variantes (M9-bis), T7 troisième cas (M17), M8-bis.
  - **R1-7 / F1-1** : confirmés (`main.rs:248-258` locales relatives au répertoire courant ; `'0.1.0'` initial) :
    montage commun (`UPDATE _kesh_version`), `KESH_LOCALES_DIR` absolu, T8 asserte la cause.
  - **R1-8, F1-11** : § *Manuel* point 3 (obtention du fichier, ce qui est perdu, image épinglée, `/setup`).
  - **R1-9 / F1-6** : C-v0174m-21 ; message R-c, CHANGELOG, manuel.
  - **R1-10** : C-v0174m-18 — découpage en deux sous-fiches.
  - **R1-11** : motif retiré de C-v0174m-8. **R1-12 / F1-8** : quatre lignes `info` écrites (I1 à I4) ; C-v0174m-20
    (pièces manquantes) ; titres exacts fixés ; C-v0174m-13 « écartées » rempli.
  - **R1-13 / F1-5** : § *Manuel* points 4 à 10, grep de contrôle sur des jetons qui rendent quelque chose.
  - **R1-14** : manuel (seuil, mémoire) ; issue demandée à l'orchestrateur (parente).
  - **R1-15** : oracle littéral dans T1 ; C-v0174m-23 (W3) ; gardes lexicales et `Etape` précisées.
  - **R1-16 / F1-10** : `#[path]` vers les helpers de `kesh-db`, visibilités nommées, `pub mod`.
  - **R1-17** : angle mort écrit (Conception, point 2).
  - **F1-7** : nombre exact (AC-1), W5 et W1/W2 par T17, `.partial` récent et vieilli (T9, M19).
  - **F1-9** : C-v0174m-24 (prédicat de la garde), `NOM_PRE_UPGRADE` écrit, « réduits » = retirés.
  - **F1-12** : manuel (versions sans migration qui réparent), R-b sans `{de}`, Dev Notes du poste de dev.
  Prochaine étape : validation P2 (Opus, lentille complète, `v0174-mise-a-jour-validate-prompt-p2.md`).

- 2026-11-05 — Remédiation de la validation P2 (Opus, 0/1/10/9 ; 16 findings sur 20 issus de la remédiation P1) par
  Opus 5.5. **Signal de non-convergence** (la rétention a produit un HIGH en P1 et un autre en P2 ; l'échappatoire par
  `docker compose run` un MEDIUM à chaque passe), déclaré au Project Lead par l'orchestrateur ; décision : **simplifier
  plutôt que corriger**. Prémisse de chaque finding relue au code. Traitement :
  - **V2-1 (HIGH)** : prémisse confirmée (`record_boot_version`, `version.rs:325`, à chaque démarrage réussi ; séquences A et
    B exactes). Fermé **par retrait** : plus aucune suppression automatique (C-v0174m-25 retire C-6 et C-15 ; T9-bis, M8,
    M8-bis, M18, M22 retirés ; T9 récrit, M27). Le correctif proposé (protéger la plus ancienne depuis `last_boot_at`) est
    **écarté** : un mécanisme de plus, dans la zone qui recycle.
  - **V2-7** : E1 après l'échec de `MIGRATOR.run`, règle RG reprise par R-b, W6 et le manuel ; chaque message nomme le
    fichier de **cette** tentative.
  - **V2-8** : la voie `docker compose run --rm -e …` est **retirée** (C-v0174m-28 remplace C-21) : une seule voie, la
    variable dans `.env` transmise par le compose ; pour un compose antérieur, ajouter la ligne.
  - **V2-2** : confirmé (`configuration_transmise.rs:122`, `:132`) ; `SANS_DEFAUT`, non `VIDE_SIGNIFIANT`.
  - **V2-3** : confirmé (`lib.rs:14` `pub(crate) mod admin_backup`) ; `pub mod admin_backup` (C-v0174m-26).
  - **V2-4** : T18 récrit (clés à tous les niveaux, `NotIncludedReason`, gabarit d'octets, constante `TYPES_DECODES`), M29.
  - **V2-5** : `Contexte` défini ; `Refus::Plan` sans `{de}` ni échappatoire (C-v0174m-27), M26, T16 second cas, M17-bis.
  - **V2-6** : T6-quater (`Refus::Tache`, M24), T6-quinquies (`free_bytes` en erreur, M25).
  - **V2-9** : confirmé (`admin-manual.tex:2930`, patron `printenv`) ; commande développée dans le conteneur.
  - **V2-10** : confirmé (U+2019 dans le PDF) ; jeton `Rollback en cas d.échec`, comptes avant/après exigés.
  - **V2-11** : `:870-871`, `:1525-1526` ajoutés (point 6) et aux jetons.
  - **LOW** : V2-12 (`mcp_prompt_versions` premier manquant, comparaison par `SELECT` brut), V2-13 (T3 en trois appels),
    V2-14 (`now` injecté au point d'entrée), V2-15 (registre et parente à jour, **#911** citée), V2-16 (tableau de la
    parente corrigé, téléchargement conditionné à la 2), V2-17 (sous-fiche 2, clé d'API), V2-18 (T20 : déclencheurs ;
    « tentative n° » retiré), V2-19 (procédure Synology placée dans `sec:backup-dsm`, C-v0174m-29), V2-20 (angle mort écrit,
    manuel point 3).
  Prochaine étape : validation P3 (Sonnet, lentille complète braquée d'abord sur cette remédiation,
  `v0174-mise-a-jour-validate-prompt-p3.md`).

- 2026-11-05 — **Version minimale**, décision du Project Lead (C-v0174m-30) après la non-convergence de la validation
  (P1 2 HIGH → P2 1 HIGH → P3 1 HIGH, tous recyclés ; P3 : 0/1/5/4, `kesh-gate-logs/v0174m-validate-p3.md`). La fiche est
  récrite autour de quatre points (plan et refus ; `.keshbackup` à l'inventaire du schéma présent ; échappatoire à version
  exacte ; jamais de suppression). **Gardés** : plan lu dans `_sqlx_migrations`, plan illisible refusé, inventaire du schéma
  présent, nom aux deux versions, seuil d'espace, refus sur tout échec avant migration, échappatoire (évaluée en premier,
  C-v0174m-31), aucune suppression, test d'aller-retour discriminant (T1-ter), absence de sauvegarde sans migration ou sur
  base neuve, recette d'une jambe. **Retirés** : liste à l'écran (sous-fiche 2 `superseded`, issue demandée), règle RG, E1
  et messages après `MIGRATOR.run`, T18 et `TYPES_DECODES`, `pub mod admin_backup` (tests internes), refus de version de
  départ `< 0.15.0`, relecture, T6-quater et nettoyage sur panique, W1 à W3, seconde jambe et seconde topologie, procédure
  Synology neuve (renvoi à `sec:backup-dsm`). Findings P3 : V3-3 (HIGH, nettoyage sur panique) fermé par retrait de
  l'exigence ; V3-1, V3-2, V3-8, V3-9, V3-10 sans objet (retirés) ; V3-4 : une phrase au manuel (point 1 : la boucle, l'arrêt, ne rien supprimer avant la réussite) ; V3-5 et V3-7 (registre) traités par les
  annotations du registre ; V3-6 sans objet (aucune procédure Synology neuve). Tests renumérotés T1 à T14, mutations M1 à
  M14. Prochaine étape : validation P4 (Opus, lentille complète, `v0174-mise-a-jour-validate-prompt-p4.md`).

## Dev Agent Record

### Agent Model Used

### Debug Log References

### Completion Notes List

### File List
