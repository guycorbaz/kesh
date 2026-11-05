# Story v0174-mise-a-jour-1 : la 0.17.4 — la sauvegarde avant migration (#847, lot A)

Status: ready-for-dev

*Sous-fiche de `v0174-mise-a-jour.md` (parente en `split`, C-v0174m-18), écrite le 2026-11-05 à la remédiation de la
validation P1, **simplifiée** à celle de la P2 (Opus 5.5, autonomie). Worktree `/home/gcorbaz/devel/kesh-0174m`, branche
`story/v0174-mise-a-jour` partie de `7bd4d5c0` ; port **3178**, bases `kesh_v0174m` / `kesh_e2e_v0174m`. Registre
`v0174-choix-autonomes-v0174m.md` (C-v0174m-1 à 29 ; C-6, C-15 et C-21 **retirés**). L'analyse du code (§ *Ce que le code
dit*) est dans la parente et n'est pas répétée ; les numéros de ligne sont ceux de `7bd4d5c0`. Se développe **avant**
`v0174-mise-a-jour-2.md`. Aucune migration (C-v0174m-13).*

## Story

En tant qu'**administrateur** d'une installation Kesh qui tient des livres réels, je veux que Kesh, au démarrage d'une
version qui doit migrer la base, écrive lui-même, avant toute migration, une sauvegarde complète de l'état d'avant — base
**et** pièces — que la version de départ sait restaurer, qu'il refuse de migrer si cette sauvegarde échoue, et qu'il me
dise où elle est et comment revenir en arrière.

## Conception (rappel ; choix au registre)

`crates/kesh-api/src/pre_upgrade_backup.rs`, `pub mod` dans `crates/kesh-api/src/lib.rs` ; `admin_backup` passe de
`pub(crate) mod` à `pub mod` (`lib.rs:14`), pour que le test d'aller-retour (T1-ter), crate externe, atteigne
`parse_and_verify_avec_inventaire` et `check_schema_compat` (C-v0174m-26 ; précédent 31-1, P6). Point d'entrée :

```rust
pub async fn sauvegarder_avant_migration(
    pool: &MySqlPool,
    config: &Config,
    controle_de_version: &DowngradeCheckOutcome,
    binaire: &str,                     // CARGO_PKG_VERSION
    environnement: Arc<dyn BackupEnv>, // free_bytes, wrap_writer (auto_backup.rs)
    now: NaiveDateTime,                // UTC, injectée (T9, T10)
) -> Result<Issue, Refus>
```

`Issue { InstallationNeuve, AJour, Interrompue { version }, Contournee, Ecrite { chemin, meta } }` ;
`Refus { Plan, VersionDeDepart, Dossier, Espace, Lecture, Ecriture, Relecture, Tache }`, chacun avec l'erreur d'origine.
Appel par `main.rs` entre l'étape 3b (`check_downgrade_protection`, `:122-154`) et l'étape 4 (`MIGRATOR.run`, `:157`) ;
doc-comment d'en-tête `main.rs:1-29` complété (étape 3c). Dans ce seul ordre :

1. `controle_de_version` = `FreshInstall` → `Issue::InstallationNeuve`, rien d'autre.
2. **Plan** (`plan_de_migration(pool)`, partie pure `plan_depuis(appliquees: &[(i64, bool)], connues: &[i64])`) :
   lecture de `_sqlx_migrations` en erreur **1146** → *neuve* ; **toute autre erreur** → `Err(Refus::Plan)`
   (C-v0174m-16) ; table vide → *neuve* ; une ligne `success = 0` → `Interrompue` ; toutes les montantes présentes →
   `AJour` ; sinon *à appliquer* `{ nombre, premiere }` (C-v0174m-3). Migration appliquée inconnue ou checksum différent :
   le plan n'en tient pas compte, `sqlx` refusera après la sauvegarde (angle mort assumé, R1-17).
3. **Échappatoire** : `config.skip_pre_upgrade_backup == Some(binaire)` → W4, `Issue::Contournee` ; **évaluée avant** la
   version de départ, **après** le plan (un plan illisible n'est pas contourné : C-v0174m-27). Toute autre valeur → W5, on
   continue (C-v0174m-4).
4. **Version de départ** : `_kesh_version.kesh_version_last_applied` ; illisible, absente ou `< 0.15.0` →
   `Err(Refus::VersionDeDepart)` (C-v0174m-8).
5. **Dossier** `<KESH_ADMIN_BACKUP_DIR>/pre-upgrade/` (constante `PRE_UPGRADE_SUBDIR`) créé en `0700`, qui doit **se
   lister** (`read_dir`, sinon `Refus::Dossier`) ; `.partial` abandonnés du préfixe supprimés
   (`supprimer_les_partiels(dir, "kesh-pre-upgrade-")`, `auto_backup.rs:448`).
6. **Espace** : `seuil_d_espace(max(taille des sauvegardes de auto/ et de pre-upgrade/), taille_des_documents)`
   (`auto_backup.rs:510`, `:496`) contre `BackupEnv::free_bytes` (C-v0174m-7) ; dessous → `Refus::Espace { libre, seuil }` ;
   `free_bytes` en erreur → `Refus::Espace` portant l'erreur (comme la sauvegarde automatique, qui refuse).
7. **Lecture** par `kesh_db::backup::export_installation_schema_present(pool)`, **écriture** par `write_keshbackup` dans
   `write_backup_file` (`routes/admin.rs:754`), sous le nom du § *Nom*, avec l'écrivain de `wrap_writer` ; **relecture** par
   `relire` (`auto_backup.rs:551`). `Refus::Lecture` (base), `Refus::Ecriture` (classe `disk_full` par
   `classify_write_error`, `:250`), `Refus::Relecture` (fichier supprimé), `Refus::Tache` (panique dans le
   `spawn_blocking`, `BackupError::Task`, `export.rs:148`).
8. **Aucune suppression** de sauvegarde avant mise à jour, jamais (C-v0174m-25) : chaque tentative écrit la sienne ; elles
   s'accumulent dans `pre-upgrade/` et se retirent à la main (manuel).
9. **Journal** (§ *Messages*) ; W1 si des pièces manquent (la migration se fait, C-v0174m-20), W2 si le fichier dépasse
   `KESH_ADMIN_IMPORT_MAX_MB`, W3 si une table lue n'est pas dans le `TABLES_TO_TRUNCATE` du binaire (C-v0174m-23).

`main.rs` : `Err(refus)` → `tracing::error!` du message de refus, `exit(1)` ; `Ok(issue)` → étape 4 ; si
`MIGRATOR.run` échoue **après** `Issue::Ecrite`, la ligne d'erreur existante (`main.rs:158`) est suivie de **E1**
(§ *Messages*), qui nomme le fichier de cette tentative et rappelle la règle (C-v0174m-25, V2-7).

**Dans `kesh-db/src/backup.rs`** : `export_installation_schema_present(pool)` — même `ReadSnapshot`, même cœur
`export_installation_in` (générique sur la liste des tables), la liste lue **dans l'instantané** par
`SELECT TABLE_NAME FROM information_schema.TABLES WHERE TABLE_SCHEMA = DATABASE()` (sans filtre `TABLE_TYPE`, comme la
garde `backup_inventory_matches_schema`), filtrée en Rust par `!nom.starts_with('_')`, triée (C-v0174m-24).
`InstallationExport.tables` et `Etape::ApresTable` passent à des noms possédés (`String`) ; `KeshBackupMeta.table_count`
= `export.tables.len()` (`export.rs:330`). `export_installation` reste le chemin de l'export, de l'import et de la
sauvegarde automatique. `build_keshbackup` (`export.rs:148`) reçoit `enum Inventaire { Binaire, SchemaPresent }`.

**Gardes lexicales** : `export_installation_n_a_qu_un_appelant` (`export.rs:489`) cherche `export_installation(`, que
`export_installation_schema_present(` ne contient pas : **une garde par fonction** ; `build_keshbackup_ne_lit_la_base_qu_une_fois`
(`:464`) couvre les deux inventaires ; la garde d'instantané de `backup.rs` s'étend à la fonction neuve.

**Import à inventaire paramétré** (C-v0174m-17) : `parse_and_verify` (`import.rs:172`) et `restore_tables_in_tx`
(`backup.rs:625`) délèguent à `parse_and_verify_avec_inventaire(reader, limits, inventaire: &[&str])` et
`restore_tables_in_tx_avec_inventaire(tx, tables, inventaire)` ; les fonctions d'aujourd'hui passent `TABLES_TO_TRUNCATE` et
restent les seules appelées en production (`routes/admin.rs:430` inchangé) ; la restauration parcourt l'inventaire dans
l'ordre de `TABLES_TO_TRUNCATE` (filtre), exclusions `onboarding_state` et `audit_log` (fusionnée) inchangées.

**Configuration** (C-v0174m-4, C-v0174m-28) : `KESH_SKIP_PRE_UPGRADE_BACKUP` par `env_nonempty` dans `Config::from_env`
(`skip_pre_upgrade_backup: Option<String>`) ; **une seule voie** : la ligne
`KESH_SKIP_PRE_UPGRADE_BACKUP: ${KESH_SKIP_PRE_UPGRADE_BACKUP:-}` sous `environment:` de `kesh-api` dans
`docker-compose.yml` **et** `docker-compose.prod.yml`, la valeur posée dans `.env` ; ligne **commentée** à `.env.example`
(« version exacte ; vide ou absente : la sauvegarde se fait ; à retirer après la mise à jour ») ; rangée dans
**`SANS_DEFAUT`** de `crates/kesh-api/tests/configuration_transmise.rs` (`:132` : forme `${NOM:-}` exigée, lignes de
`.env.example` commentées ; vide = absent = sauvegarde faite — **pas** `VIDE_SIGNIFIANT`, réservé à un vide qui a un sens
distinct de l'absence, `:122`) et dans `EMPLACEMENTS_AUTORISES` ; doc-comment de `SANS_DEFAUT` complété de la raison ;
manuel § *Variables d'environnement*.

### Nom (C-v0174m-5)

`kesh-pre-upgrade-<AAAAMMJJTHHMMSSmmm>Z-<de>-to-<vers>.keshbackup` (horodatage = `now`, UTC), expression exacte :

```text
NOM_PRE_UPGRADE = ^kesh-pre-upgrade-[0-9]{8}T[0-9]{9}Z-[0-9A-Za-z.-]{1,40}-to-[0-9A-Za-z.-]{1,40}\.keshbackup$
```

`<de>` et `<vers>` : chaque caractère hors `[0-9A-Za-z.-]` est **retiré** ; vide après réduction → `inconnue` ; tronqué à
40 caractères.

### Messages (texte exact, journal du serveur)

Produits par des fonctions **pures** (`message_*`, `message_de_refus(&Refus, &Contexte)`), testées par T17.
`Contexte { de: Option<String>, vers: String, dir: PathBuf, fichier: Option<PathBuf> }` — `de` absent tant que l'étape 4
n'est pas atteinte (plan) ou si la version est illisible ; `fichier` = celui de **cette** tentative, s'il a été écrit.

Succès (`info`) :

- **I1, plan** : `Mise à jour de la base : {n} migration(s) à appliquer (de la {de} à la {vers}). Sauvegarde complète avant mise à jour en cours — base et pièces ; le serveur répondra une fois la mise à jour terminée.`
- **I2, base lue** : `Sauvegarde avant mise à jour : {tables} tables, {lignes} lignes lues en {duree} s.`
- **I3, pièces** : `Sauvegarde avant mise à jour : {pieces} pièces emportées ({octets} octets), {manquantes} non incluses.`
- **I4, fin** : `Sauvegarde avant mise à jour écrite : {chemin} ({taille} octets, en {duree} s). Pour revenir à la {de} : manuel d'administration, « Revenir à la version précédente ».`

La **règle** (texte **RG**, repris par R-b, E1 et W6, et mot pour mot au manuel) :
`Si une tentative précédente de cette mise à jour a déjà appliqué une partie des migrations, restaurez non pas ce fichier mais la plus ancienne sauvegarde de {dir} écrite après le dernier démarrage réussi de la {de}.`
(sans `{de}` : « … de la version de départ »).

Avertissements : **W1** pièces (`{manquantes} pièce(s) référencée(s) absente(s), altérée(s) ou illisible(s) ne sont pas dans la sauvegarde avant mise à jour (le journal les nomme) ; la mise à jour continue. Vérifiez le montage du dossier des documents.`) ;
**W2** taille (`La sauvegarde avant mise à jour ({taille} Mio) dépasse KESH_ADMIN_IMPORT_MAX_MB ({plafond} Mio) : pour la restaurer par l'écran, la version de départ devra relever ce plafond.`) ;
**W3** table étrangère (`Table « {table} » inconnue de Kesh {vers} : elle est dans la sauvegarde avant mise à jour, qui risque de ne pas se restaurer.`) ;
**W4** échappatoire utilisée (`KESH_SKIP_PRE_UPGRADE_BACKUP={vers} : migration SANS sauvegarde préalable, à la demande de l'administrateur. Retirez cette variable après la mise à jour.`) ;
**W5** échappatoire ignorée (`KESH_SKIP_PRE_UPGRADE_BACKUP={valeur} ignorée : elle ne vaut que pour la version {vers}.`) ;
**W6** interrompue (`Une migration interrompue ({version}) est inscrite dans la base : aucune sauvegarde avant mise à jour n'est écrite. Pour revenir en arrière, restaurez la plus ancienne sauvegarde de {dir} écrite après le dernier démarrage réussi de la version de départ — manuel d'administration, « Revenir à la version précédente ».`).

**E1**, après l'échec de `MIGRATOR.run` qui suit une sauvegarde écrite :
`La sauvegarde avant mise à jour de cette tentative est {fichier}. Pour revenir à la {de}, restaurez-la — manuel d'administration, « Revenir à la version précédente ». {RG}`

Refus (`error`, puis `exit(1)`), trois phrases :

- **R-a** : `FATAL : mise à jour refusée — {cause}. Cette tentative n'a appliqué aucune migration.`
- **R-b** : si `de` est connu → ` Si aucune tentative précédente n'a migré, la {de} redémarre sur cette base. {RG}` ;
  `Refus::Plan` → ` L'état des migrations n'a pas pu être lu.` ; version illisible → ` La version de départ n'est pas connue. {RG sans {de}}`.
- **R-c** : `Refus::Plan` → ` Corrigez la cause : connexion à la base ou droits de lecture de _sqlx_migrations.` (aucune
  échappatoire : elle ne contourne pas le plan) ; autres → ` Corrigez la cause, ou, en connaissance de cause, migrez sans sauvegarde : posez KESH_SKIP_PRE_UPGRADE_BACKUP={vers} dans le .env — votre compose doit la transmettre sous environment: du service kesh-api (ceux de la 0.17.4 le font ; un compose antérieur, non : ajoutez-y la ligne) ; manuel d'administration, « La sauvegarde avant mise à jour ».`

`{cause}` par variante : `lecture de l'état des migrations impossible ({erreur})` ;
`version de départ {valeur} : la sauvegarde avant mise à jour ne se restaure que dans une 0.15.0 ou plus récente — faites un vidage de la base` ;
`dossier {dir} : {erreur}` ; `espace insuffisant dans {dir} : {libre} Mio libres, {seuil} Mio requis` (ou `espace libre de {dir} illisible : {erreur}`) ;
`lecture de la base : {erreur}` ; `écriture de {chemin} : {erreur}` ; `relecture de {chemin} : {erreur} (fichier supprimé)` ;
`tâche interrompue : {erreur}`. Aucun ne cite `DATABASE_URL`.

## Acceptance Criteria

**AC-1 — Le plan.** 1146 ou table vide → *neuve* ; une ligne `success = 0` → *interrompue* ; toutes les montantes
présentes → *à jour* ; sinon *à appliquer* avec le **nombre exact** ; toute autre erreur de lecture → refus sans
échappatoire. Seul *à appliquer* déclenche une sauvegarde. (T1, T2, T3, T4, T16.)

**AC-2 — Le contenu.** Un `.keshbackup` de format 2 dont les tables sont exactement les tables du schéma présent sans
préfixe `_` (colonnes non générées, toutes les lignes), les pièces référencées sous `files/`, `kesh_version_min_required` de
la base. Sur une base arrêtée avant `20261027000001_mcp_prompts` : ensemble des tables **égal** à `TABLES_TO_TRUNCATE` privé
de `mcp_prompt_versions` et `mcp_prompts` (oracle littéral). (T1, T1-bis.)

**AC-3 — Importable par la version de départ.** Le fichier de T1 passe `parse_and_verify_avec_inventaire` avec
l'inventaire de départ, `check_schema_compat` contre une seconde base au même état, et sa restauration y redonne les mêmes
lignes ; la garde T18 fige le format **et l'encodage**. (T1-ter, T18.)

**AC-4 — Emplacement et nom.** § *Nom* ; dossier `0700`, fichier `0600`, jamais d'écrasement, `.partial` pendant
l'écriture ; `.partial` abandonnés du préfixe supprimés, récents laissés. (T1, T9, T10.)

**AC-5 — L'échec refuse la migration.** Plan illisible, version de départ illisible ou `< 0.15.0`, dossier non créable ou
non listable, espace sous le seuil ou illisible, lecture, écriture, relecture, tâche interrompue : message de refus,
code non nul, aucune migration appliquée (`_sqlx_migrations` inchangée), aucun `.partial` de la tentative laissé.
(T5, T5-bis, T6, T6-bis, T6-ter, T6-quater, T6-quinquies, T12, T16, T19, T8.)

**AC-6 — L'échappatoire.** Égale à la version du binaire : aucune sauvegarde, migration appliquée, W4 — y compris avec une
version de départ `< 0.15.0` ; sans effet sur un plan illisible. Autre valeur : W5, sauvegarde écrite. Une seule voie :
transmise par les deux compose, ligne commentée à `.env.example`, garde `configuration_transmise` verte. (T7, T8-bis, T14,
T16.)

**AC-7 — Rien à migrer, rien d'écrit.** *Neuve*, *à jour*, *interrompue* : aucun fichier, aucun dossier `pre-upgrade/`
créé ; *interrompue* écrit W6. (T2, T3, T4.)

**AC-8 — Aucune suppression.** Kesh ne supprime aucun fichier `kesh-pre-upgrade-*.keshbackup` : après n tentatives, les n
fichiers sont présents ; les sauvegardes de `auto/` et pré-import ne sont pas touchées par ce chemin. (T9.)

**AC-9 — Le journal.** I1 à I4, W1 à W6, E1 et le refus, avec les valeurs réelles, par les fonctions pures ; W1, W2, W3
**déclenchés** par leur condition. (T17, T20 ; I1 à I4 et E1 relevés en recette.)

**AC-10 — Le manuel et le CHANGELOG.** § *Manuel* et § *CHANGELOG* appliqués au `.tex` et au PDF ; le contrôle du point 9
relève, pour chaque jeton, le compte **avant** (≥ 1, sonde positive) et **après**, au Dev Agent Record.

**AC-11 — La recette** (C-v0174m-19, C-v0174m-22), base de démonstration jetable (une écriture, une facture fournisseur à
document, une facture émise à PDF figé, un réglage de sauvegarde automatique modifié) :

- **Jambe 1, 0.15.0 → branche, compose standard** : sauvegarde écrite (I1 à I4 relevés), migrations appliquées →
  `gcorbaz/kesh:0.15.0` refusée sur la base migrée (message relevé) → procédure du manuel (commande de recréation exacte du
  § *Manuel* point 3) → la 0.15.0 redémarre, données et pièces revenues, réglage de sauvegarde revenu aux défauts. Puis,
  sur la même installation, l'échappatoire : `.env` + ligne du compose 0.17.4, migration sans sauvegarde, W4 relevé.
- **Jambe 2, 0.17.2 → branche, topologie de production** (MariaDB autonome sur un réseau jetable, conteneur Kesh seul,
  `--memory 1g`, compte Kesh sans root), même enchaînement. Si elle ne peut pas se monter sur la station : le dire au Dev
  Agent Record, et le manuel marque la procédure Synology « non éprouvée ».
- Une migration en échec après sauvegarde (migration de test ajoutée **à une copie jetable de l'image**, jamais au dépôt) :
  E1 relevé — ou, si ce montage est trop coûteux, l'écrire comme non joué.
- Chaque commande et chaque sortie au Dev Agent Record ; ce que la recette n'établit pas, écrit.

**AC-12 — Aucune migration.** `ls crates/kesh-db/migrations/*.sql | wc -l` = 89 avant et après ;
`kesh_version_min_required` et `TABLES_TO_TRUNCATE` inchangés.

## Tests et mutations

Rouges **d'abord**, constatés et relevés. Fichier : `crates/kesh-api/tests/sauvegarde_avant_migration.rs`, qui inclut les
helpers de `kesh-db` par `#[path = "../../kesh-db/tests/common/mod.rs"] mod montage;` (`apply_migrations_up_to`,
`migrations_before` : index par **version**, P6 ; dépendances `sqlx` et `kesh_db` seulement) et s'inscrit à
`ALLOWED_REAL_MIGRATOR_FILES` de `crates/kesh-db/tests/test_schema_guard.rs` (attribut
`#[sqlx::test(migrations = false)]` sur une seule ligne).

**Montage commun des bases partielles** : `apply_migrations_up_to(&pool, migrations_before(20261027000001, "mcp_prompts"))`,
puis `UPDATE _kesh_version SET kesh_version_last_applied = '0.17.3' WHERE id = 1` (sinon `'0.1.0'` et tout test rend
`Refus::VersionDeDepart`), puis les données en SQL brut ; assertion de montage : le plan rend
`MIGRATOR.migrations.len() − n` (recalculé). Appels avec `controle_de_version = DowngradeCheckOutcome::Aligned` sauf T3.

| Test | Montage | Assertion |
|---|---|---|
| **T1** | base partielle ; société, utilisateur, compte, une écriture, une `imported_supplier_invoices` dont le document existe dans un dossier temporaire, une facture à `pdf_storage_path` | `Ecrite` ; fichier sous `pre-upgrade/`, nom = `NOM_PRE_UPGRADE`, `0.17.3-to-<binaire>` ; tables du manifeste = `TABLES_TO_TRUNCATE \ {mcp_prompt_versions, mcp_prompts}` ; aucune `_…` ; `rowCount` exacts ; `files` porte les deux pièces ; puis `MIGRATOR.run` réussit |
| **T1-bis** | même base | `export_installation` (inventaire du binaire) y échoue en `Invariant` |
| **T1-ter** | fichier de T1 ; seconde base au même état | `parse_and_verify_avec_inventaire(…, inventaire littéral)` OK ; `check_schema_compat` OK ; restauration par `restore_tables_in_tx_avec_inventaire` ; chaque table de l'inventaire littéral : `SELECT *` brut identique aux deux bases (`onboarding_state` exclue, `audit_log` comparée sans `id`) ; `parse_and_verify` (inventaire du binaire) **refuse** ce fichier : `table 'mcp_prompt_versions' absente du manifeste` |
| **T2** | base entièrement migrée | `AJour`, aucun fichier, `pre-upgrade/` non créé |
| **T3** | base vide | `plan_de_migration` → *neuve* (appel direct) ; point d'entrée avec `Aligned` → `InstallationNeuve`, aucun fichier ; point d'entrée avec `FreshInstall` → idem |
| **T4** | base partielle + ligne `success = 0` | `Interrompue`, aucun fichier |
| **T5** | `KESH_ADMIN_BACKUP_DIR` = fichier régulier | `Refus::Dossier` |
| **T5-bis** | `pre-upgrade/` en `0o300` | `Refus::Dossier` |
| **T6** | libre = seuil − 1 avec une sauvegarde de 600 Mio (fichier épars) dans `pre-upgrade/` seulement, puis dans `auto/` seulement ; libre = seuil | `Refus::Espace` (deux variantes) ; écrit |
| **T6-bis** | écrivain qui tronque | `Refus::Relecture`, fichier supprimé |
| **T6-ter** | écrivain `StorageFull` | `Refus::Ecriture` (`disk_full`), aucun `.partial` |
| **T6-quater** | `wrap_writer` qui panique | `Refus::Tache`, aucun `.partial` |
| **T6-quinquies** | `free_bytes` en `Err` | `Refus::Espace` portant l'erreur, rien d'écrit |
| **T7** | skip = binaire / `0.0.1` / binaire et `last_applied = '0.14.0'` | `Contournee` / `Ecrite` / `Contournee` |
| **T8** (binaire) | base partielle ; dossier = fichier ; `KESH_PORT` occupé ; `DATABASE_URL` (base éphémère, `SELECT DATABASE()`), `KESH_JWT_SECRET`, `KESH_HOST=127.0.0.1`, `KESH_PORT`, `KESH_ADMIN_BACKUP_DIR`, `KESH_LOCALES_DIR` = `concat!(env!("CARGO_MANIFEST_DIR"), "/../kesh-i18n/locales")` | code non nul ; sortie contient `mise à jour refusée — dossier`, pas `version de départ` ; `_sqlx_migrations` inchangée |
| **T8-bis** (binaire) | même montage, dossier valide, skip = binaire | sortie contient W4 et `Impossible de bind` ; `_sqlx_migrations` complète |
| **T9** | cinq exécutions de suite (`now` avancé d'une milliseconde) ; `pre-upgrade/` portant aussi un `.partial` récent et un `.partial` vieilli de plus de dix minutes (`File::set_modified`), un fichier quelconque, et une sauvegarde dans `auto/` | cinq fichiers `kesh-pre-upgrade-*` présents ; le `.partial` vieilli supprimé, le récent, le fichier quelconque et `auto/` intacts |
| **T10** | fonction pure du nom | format exact ; `0.17.0-rc1` gardé ; `1.0.0+build/x` → `1.0.0buildx` ; `+++` → `inconnue` ; 41 caractères → 40 ; chaque nom satisfait `NOM_PRE_UPGRADE` |
| **T12** | `last_applied` = `0.14.0` / `pas-semver` | `Refus::VersionDeDepart`, aucun dossier créé |
| **T14** | `configuration_transmise` | verte : variable dans `SANS_DEFAUT`, forme `${NOM:-}` dans les deux compose, ligne commentée à `.env.example` |
| **T15** | garde de convention (base squash) | toute table hors `TABLES_TO_TRUNCATE` commence par `_` ; aucune de `TABLES_TO_TRUNCATE` ne commence par `_` |
| **T16** | plan sur un pool fermé ; puis avec skip = binaire | `Refus::Plan` dans les deux cas (l'échappatoire ne le contourne pas) |
| **T17** | fonctions pures | I1, I4, W1 à W6, E1, RG et le refus pour chaque variante de `Refus` : textes exacts ; `Plan` : R-b « L'état des migrations n'a pas pu être lu », R-c sans `KESH_SKIP_PRE_UPGRADE_BACKUP` ; version illisible : aucun `{de}` ; aucune variante ne cite `DATABASE_URL` |
| **T18** | garde de format | `BACKUP_FORMAT_VERSION == 2` ; clés du manifeste à **tous** les niveaux (`formatVersion`, `keshVersion`, `keshVersionMinRequired`, `instanceId`, `exportDate`, `tables.*.{rowCount, sha256, columnNames}`, `files[].{path, sha256, byteSize}`, `filesNotIncluded[].{path, reason}`) et valeurs de `NotIncludedReason` = listes figées ; **gabarit d'octets** : une table de test (base éphémère) portant une colonne de chaque type que `decode_cell` distingue (`bigint`, `int`, `tinyint`, `decimal`, `double`, `date`, `datetime(3)`, `varchar`, `text`, `json`, `NULL`), exportée par `export_table`, NDJSON comparé à une chaîne figée ; `decode_cell` lit ses bras dans une constante `TYPES_DECODES` que le test énumère ; message : « changer ceci rend les sauvegardes inimportables par les versions antérieures : consigner au manuel et au CHANGELOG » |
| **T19** | étape d'écriture (`ecrire_la_sauvegarde`, `pub`) appelée avec le pool fermé après un plan réussi | `Refus::Lecture`, aucun `.partial`, aucun fichier |
| **T20** | déclencheurs : une pièce référencée absente ; plafond d'import posé à 0 Mio ; une table `zz_etrangere` créée dans la base partielle | `Ecrite` avec W1 / W2 / W3 rendus par l'issue (`Issue::Ecrite` porte la liste des avertissements émis, que le test lit) |

Mutations (sous verrou, restaurées par `git checkout --` puis `touch`) :

| Mutation | Tuée par |
|---|---|
| **M1** l'appel retiré de `main.rs` | T8 |
| **M2** `main.rs` journalise le refus et continue | T8 |
| **M3** le plan rend toujours `AJour` | T1 |
| **M4** l'inventaire lu dans `TABLES_TO_TRUNCATE` | T1, T1-ter |
| **M5** *neuve* (plan) traitée comme *à appliquer* | T3 (appel direct et `Aligned`) |
| **M6** la branche `Interrompue` retirée | T4 |
| **M7** l'échappatoire acceptée sur `is_some()` | T7 (`0.0.1`) |
| **M9** contrôle d'espace retiré | T6 |
| **M9-bis** seuil sans `pre-upgrade/` (ou sans `auto/`) | T6 |
| **M10** relecture retirée | T6-bis |
| **M13** contrôle de version de départ retiré | T12 |
| **M14** préfixe `_` non exclu | T1 |
| **M15** `Err(Write)` converti en `Ecrite` | T6-ter |
| **M16** plan en erreur traité comme *neuve* | T16 |
| **M17** échappatoire évaluée après la version de départ | T7 (troisième cas) |
| **M17-bis** échappatoire évaluée avant le plan | T16 (second cas) |
| **M19** `supprimer_les_partiels` non appelé | T9 |
| **M20** dossier non listable accepté | T5-bis |
| **M21** `Err(Db)` de la lecture ignoré | T19 |
| **M23** `restore_tables_in_tx_avec_inventaire` ignore l'inventaire | T1-ter (`DELETE FROM mcp_prompt_versions` échoue) |
| **M24** `Err(Task)` converti en `Ecrite` | T6-quater |
| **M25** `free_bytes` en erreur traité comme espace suffisant | T6-quinquies |
| **M26** R-c avec l'échappatoire sur `Refus::Plan` | T17 |
| **M27** une suppression de sauvegarde avant mise à jour réintroduite (rétention) | T9 |
| **M28** W3 jamais émis | T20 |
| **M29** encodage d'un `datetime` changé (`%.3f` → secondes) | T18 (gabarit) |

(M8, M8-bis, M18, M22 sont retirées avec la rétention, C-v0174m-25 ; M11 et M12 appartiennent à la sous-fiche 2.)

## Tâches

1. **Étape 0** : relire les prémisses de la parente au code (lignes citées au Change Log) ; recréer `kesh_v0174m` /
   `kesh_e2e_v0174m` à neuf (migrations du worktree, `scripts/seed-dev-db.sql`).
2. T1 à T20 en rouge ; relever les rouges.
3. `kesh-db` : inventaire du schéma présent, noms possédés, import à inventaire paramétré, `TYPES_DECODES`, T15, T18.
4. `kesh-api` : module `pre_upgrade_backup` ; `pub mod admin_backup` ; `Config::skip_pre_upgrade_backup` ; appel, E1 et
   doc-comment dans `main.rs` ; visibilités `pub(crate)` : `seuil_d_espace`, `taille_des_documents`, `relire`,
   `fichiers_reguliers`, `supprimer_les_partiels`, `classify_write_error` (`auto_backup.rs`), `ensure_backup_dir`
   (`routes/admin.rs:143`) ; gardes lexicales adaptées. (Aucune rétention à généraliser.)
5. Compose (deux fichiers), `.env.example`, `configuration_transmise.rs` (`SANS_DEFAUT`).
6. Mutations du tableau.
7. Manuel `.tex` + PDF (`make fr` sous les deux verrous) ; contrôle du point 9.
8. CHANGELOG ; `README.md:50` ; `docs/testing.md` (poste de dev).
9. Recette AC-11 (images Docker sous les deux verrous, réseaux et volumes jetables, jamais `kesh-mariadb-dev`).
10. Gate complet (`test-fast.sh --ci`), Vitest, build, E2E complet au dernier commit de code.

## Manuel

`docs/manual/fr/admin-manual.tex`, puis PDF. Titres **exacts** (cibles des messages) : *La sauvegarde avant mise à jour*
(`sec:sauvegarde-avant-maj`) et *Revenir à la version précédente* (`sec:revenir-version-precedente`).

1. **Nouvelle sous-section** *La sauvegarde avant mise à jour*, après § *La sauvegarde automatique* : depuis la 0.17.4, au
   démarrage d'une version qui doit migrer la base, Kesh écrit d'abord une sauvegarde complète (base et pièces, format 2)
   dans `pre-upgrade/` du dossier de sauvegarde (`./backup/pre-upgrade`, `/volume1/docker/kesh/backup/pre-upgrade`),
   `kesh-pre-upgrade-AAAAMMJJTHHMMSSmmmZ-<de>-to-<vers>.keshbackup` (UTC). À dire :
   - ce qu'elle contient : les tables du schéma de la version de départ — elle se restaure dans cette version ; la version
     d'arrivée la refuse si la mise à jour ajoute une table applicative (§ *Compatibilité*) ;
   - quand elle ne s'écrit pas : installation neuve ; aucune migration à appliquer (la plupart des redémarrages, toute version
     sans migration, **même si elle répare des données au démarrage**) ; migration interrompue ;
   - pendant l'écriture, le serveur ne répond pas (`/health` muet, conteneur « unhealthy » un moment) : attendre la ligne
     « Sauvegarde avant mise à jour écrite » du journal ;
   - taille (≈ base + documents), espace requis (le seuil de la sauvegarde automatique, qui ne mesure pas la base) ;
     mémoire : la base est lue en mémoire — sous le `mem_limit: 1g` du compose de production, une très grosse base fait
     tuer le conteneur (« Killed », code 137) en boucle, sans migration (issue #911) ;
   - prérequis à vérifier **avant** la mise à jour : dossier de sauvegarde **monté**, dossier des documents monté (W1) ;
   - **Kesh n'en supprime aucune** : chaque tentative de mise à jour écrit la sienne, elles s'accumulent (quelques-unes par
     an) ; **retirez les anciennes à la main**, une fois la mise à jour éprouvée — en gardant au moins la plus récente ;
   - quel fichier sert au retour : celui que nomme le journal de la tentative ; et la **règle RG**, mot pour mot ;
   - la relecture ne contrôle que l'annuaire du ZIP et le manifeste ; le test de restauration périodique reste la seule
     preuve ; c'est un **secret** ; même disque que Kesh : la copier ailleurs ; ni le `.env` ;
   - **si elle échoue**, Kesh ne migre pas et le dit ; l'échappatoire `KESH_SKIP_PRE_UPGRADE_BACKUP=<version exacte>`, à
     poser dans le `.env` ; les compose de la 0.17.4 la transmettent ; **un compose antérieur ne la transmet pas : ajoutez
     sous `environment:` du service `kesh-api` la ligne `KESH_SKIP_PRE_UPGRADE_BACKUP: ${KESH_SKIP_PRE_UPGRADE_BACKUP:-}`**,
     puis `docker compose up -d` ; à retirer du `.env` ensuite ; elle ne contourne pas un état des migrations illisible ;
     depuis une version antérieure à la 0.15.0 : refus, faire un vidage de la base, puis l'échappatoire ;
   - la liste *Sauvegardes sur le serveur* la montre (type « Avant mise à jour ») — **phrase écrite avec la sous-fiche 2** ;
     sans elle, la sauvegarde se récupère dans le dossier.
2. **§ Procédure de mise à jour standard** : étape 1 — la sauvegarde manuelle reste recommandée (hors du disque, avec le
   `.env`) ; depuis la 0.17.4, Kesh écrit lui-même la sauvegarde avant migration (renvoi). Étapes 5 et 6 : le serveur ne
   répond pas pendant la sauvegarde ; lire la ligne de fin. Le `keshwarning` récrit dans ce sens. Les § *Passer à la 0.x*
   existants ne sont pas réécrits (`:2518`, `:2606` restent vrais).
3. **§ Rollback en cas d'échec** → **Revenir à la version précédente** (C-v0174m-19) : (1) récupérer le fichier : avec la
   sous-fiche 2, par *Sauvegardes sur le serveur* **avant** d'arrêter Kesh si l'écran répond ; sinon par SSH (`sudo cp` +
   `chown`, comme pour la pré-import, `:2329-2333`) ; **quel fichier** : celui que nomme le journal, et la règle RG ;
   (2) `docker compose stop kesh-api` ; (3) vidage de sécurité de la base migrée ; (4) recréer la base vide **par le compte
   Kesh** — compose standard, variables développées **dans** le conteneur (non dans le shell de l'hôte, qui ne lit pas
   `.env`) :
   ```text
   docker compose exec mariadb sh -c 'mariadb -u"$MARIADB_USER" -p"$MARIADB_PASSWORD" -e "DROP DATABASE \`$MARIADB_DATABASE\`; CREATE DATABASE \`$MARIADB_DATABASE\` CHARACTER SET utf8mb4 COLLATE utf8mb4_unicode_ci"'
   ```
   (forme exacte jouée en jambe 1) ; **Synology** : renvoi à une sous-section placée **dans** `sec:backup-dsm`
   (C-v0174m-29), qui recrée la base par le client d'un conteneur jetable lisant `kesh-restore.cnf`, comme
   `kesh-restore.sh`, vidage de sécurité par `kesh-dump.sh` d'abord — sous les gardes G14 à G16 ; (5) remettre dans le
   compose l'image `<de>` du nom du fichier, **épinglée**, `docker compose up -d` ; (6) `/setup` (ou connexion avec
   `KESH_ADMIN_USERNAME` s'il est posé), puis « Restaurer une sauvegarde » de l'assistant (§ *Restaurer sur une nouvelle
   machine*, plafond d'import) ; le dossier des documents reste en place. **Ce que le retour perd** : la saisie faite depuis
   la mise à jour ; les réglages hors fichier (planification de la sauvegarde automatique, durée d'inactivité) reviennent
   aux défauts. Garder le paragraphe sur la réparation de l'installation. Si la jambe 2 n'a pu se jouer : « procédure
   Synology non éprouvée ». **Angle mort** : si un démarrage précédent n'a pas pu écrire sa version (`record_boot_version`
   non fatal, `main.rs:191-196`), `<de>` du nom peut être plus ancien que le schéma sauvegardé.
4. **§ Compatibilité** (`:2282`) : une phrase sur la sauvegarde avant mise à jour (importable par la version de départ,
   refusée par l'arrivée si la mise à jour ajoute une table applicative ; tant que le format ne change pas, ce qu'une release
   qui le changerait dirait).
5. **§ La sauvegarde automatique**, *Volume* et *Les sauvegardes du serveur* : `pre-upgrade/` s'ajoute au dossier, jamais
   purgé (le type à l'écran avec la sous-fiche 2).
6. **`KESH_ADMIN_BACKUP_DIR` et `./backup`** : `:812`, `:2377` (variables) ; `:870-871` (« Les deux compose montent en
   outre `./backup` sur `/data/backup` (sauvegarde de sécurité pré-import, …) ») ; `:1525` (*Stratégie recommandée*,
   « Automatique, par Kesh ») et `:1526` (« Avec le répertoire du compose : le dossier `./backup`, où Kesh écrit … ») —
   ajouter la sauvegarde avant mise à jour et `pre-upgrade/`.
7. **§ Quelle méthode choisir ?** : « quatre approches » → « cinq approches » (`:2442`), une ligne au tableau.
8. **§ Variables d'environnement complètes** : `KESH_SKIP_PRE_UPGRADE_BACKUP`.
9. **Contrôle** (PDF aplati : `pdftotext docs/manual/fr/admin-manual.pdf - | tr '\n' ' ' | tr -s ' '`), compte **avant**
   (à `7bd4d5c0`, doit être ≥ 1 : sonde positive) et **après**, au Dev Agent Record. Jetons, écrits pour l'apostrophe
   typographique du PDF (U+2019) par un `.` : `quatre approches` (avant 1, après 0) ; `Toujours sauvegarder` (avant 1,
   après : récrit) ; `Rollback en cas d.échec` (avant 3 — titre, table des matières, renvoi —, après 0 ; **jamais** l'apostrophe droite,
   qui rend 0 avant comme après) ; `sauvegarde de sécurité pré-import, des sauvegardes automatiques` (avant 1 dans le PDF :
   le site `:2377` ; le site `:812`, cellule de tableau coupée dans le PDF, se contrôle dans le `.tex` par
   `git grep -n "sauvegardes automatiques (sous-dossier" docs/manual/fr/admin-manual.tex`, avant 2 lignes) — après,
   chaque site nomme `pre-upgrade/` ; `où Kesh écrit la sauvegarde de sécurité` (`:1526`, avant 1) ; et les deux titres
   exacts (avant 0, après ≥ 1). Comptes « avant » mesurés à `7bd4d5c0` à la remédiation P2 (PDF du dépôt). Le manuel utilisateur n'est pas concerné (`git grep -n "Avant import" docs/manual/fr/user-manual.tex` vide).
10. **Autres fichiers** : `.env.example` (dossier de sauvegarde, variable neuve), commentaires des deux compose sur
    `./backup`, `README.md:50`, `docs/testing.md` § *Base de dev jetable* (poste de dev, Dev Notes).

## CHANGELOG

Sous `## [0.17.4] — Non publié` (créée en tête si absente ; union avec les autres stories), `### Ajouté` :

- **Kesh se sauvegarde lui-même avant de mettre la base à jour** ([#847](https://github.com/guycorbaz/kesh/issues/847)) :
  au démarrage d'une version qui doit migrer la base, une sauvegarde complète — base et pièces — est écrite dans
  `backup/pre-upgrade/` avant toute migration, et se restaure dans la version de départ (procédure au manuel
  d'administration, *Revenir à la version précédente*). Pendant qu'elle s'écrit, le serveur ne répond pas. Si elle échoue
  — dossier, espace disque, ou version de départ antérieure à la 0.15.0 —, Kesh ne migre pas et le dit. Kesh n'en supprime
  aucune : retirez les anciennes à la main une fois la mise à jour éprouvée. Pour migrer sans elle, en connaissance de
  cause : `KESH_SKIP_PRE_UPGRADE_BACKUP=<version>` dans le `.env`, qui ne vaut que pour cette version — les compose de la
  0.17.4 la transmettent ; **un compose antérieur ne la transmet pas** : reprenez-en la ligne dans le vôtre. La mise à jour
  0.17.3 → 0.17.4 n'en écrit pas, la 0.17.4 n'ayant pas de migration *(à confirmer à l'intégration)*. La sauvegarde
  manuelle avant mise à jour reste recommandée (hors du disque, avec le `.env`).

## Dev Notes

- **Verrous de station** : toute commande lourde par
  `flock /tmp/station-heavy.lock flock /home/gcorbaz/devel/kesh-gate-logs/.gate.lock scripts/mem-guard.sh …`, une par
  prise, `free -g` avant (sous 12 Go disponibles : attendre).
- **Boucle de redémarrage** : avec `restart: unless-stopped`, un refus fait redémarrer le conteneur en boucle. Si l'échec
  est en aval de la sauvegarde (migration qui échoue **sans** ligne « sale », avant tout DDL), chaque tentative écrit une
  sauvegarde, dont les suivantes portent un schéma à moitié migré : rien n'est supprimé (C-v0174m-25), le journal nomme le
  fichier de chaque tentative (E1) et la règle RG désigne la bonne (la plus ancienne depuis le dernier démarrage réussi de
  la version de départ). Le disque se remplit au rythme de la boucle : le manuel dit de l'arrêter (`docker compose stop`).
- **Mémoire** : une seule lecture ; même coût que la sauvegarde automatique (#911).
- **Mono-instance** : rien ne tourne encore à ce stade (pas d'`AppState`, pas de `backup_lock`).
- **Poste de dev** : une base montée par `sqlx migrate run` + seed porte `last_applied = '0.1.0'` ; le premier `cargo run`
  d'une branche qui **ajoute** une migration sur une telle base refuse (version de départ `< 0.15.0`) — poser
  l'échappatoire, ou mettre `last_applied` à jour à la main ; hors Docker, `KESH_ADMIN_BACKUP_DIR` doit être inscriptible
  (l'E2E local pose `/tmp/kesh-e2e/backup`). À écrire dans `docs/testing.md` § *Base de dev jetable*.
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

## Dev Agent Record

### Agent Model Used

### Debug Log References

### Completion Notes List

### File List
