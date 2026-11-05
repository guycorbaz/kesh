# Story v0174-mise-a-jour-1 : la 0.17.4 — la sauvegarde avant migration (#847, lot A)

Status: ready-for-dev

*Sous-fiche de `v0174-mise-a-jour.md` (parente en `split`, C-v0174m-18), écrite le 2026-11-05 à la remédiation de la
validation P1 (Opus 5.5, autonomie). Worktree `/home/gcorbaz/devel/kesh-0174m`, branche `story/v0174-mise-a-jour` partie
de `7bd4d5c0` ; port **3178**, bases `kesh_v0174m` / `kesh_e2e_v0174m`. Registre `v0174-choix-autonomes-v0174m.md`
(C-v0174m-1 à 24). L'analyse du code (§ *Ce que le code dit*) est dans la parente et n'est pas répétée ; les numéros de
ligne sont ceux de `7bd4d5c0`. Se développe **avant** `v0174-mise-a-jour-2.md`. Aucune migration (C-v0174m-13).*

## Story

En tant qu'**administrateur** d'une installation Kesh qui tient des livres réels, je veux que Kesh, au démarrage d'une
version qui doit migrer la base, écrive lui-même, avant toute migration, une sauvegarde complète de l'état d'avant — base
**et** pièces — que la version de départ sait restaurer, qu'il refuse de migrer si cette sauvegarde échoue, et qu'il me
dise où elle est et comment revenir en arrière.

## Conception (rappel ; choix au registre)

`crates/kesh-api/src/pre_upgrade_backup.rs`, `pub mod` dans `crates/kesh-api/src/lib.rs`, point d'entrée et types `pub`
(`Plan`, `Issue`, `Refus`, `sauvegarder_avant_migration`) : les tests sont un crate externe, `main.rs` un binaire distinct
(précédent 31-1, P6 : un `pub(crate)` invisible des tests d'intégration). Appel par `main.rs` entre l'étape 3b
(`check_downgrade_protection`, `:122-154`) et l'étape 4 (`MIGRATOR.run`, `:157`) ; doc-comment d'en-tête `main.rs:1-29`
complété (étape 3c). Ordre, dans ce seul ordre :

1. `check_downgrade_protection` a rendu `FreshInstall` → `Issue::InstallationNeuve`, rien d'autre.
2. **Plan** (`plan_de_migration(pool)`, partie pure `plan_depuis(appliquees: &[(i64, bool)], connues: &[i64])`) :
   - lecture de `_sqlx_migrations` en erreur **1146** → `Plan::Neuve` ; **toute autre erreur** → `Err(Refus::Plan)`
     (C-v0174m-16) ;
   - table vide → `Neuve` ; une ligne `success = 0` → `Interrompue { version }` ; toutes les migrations montantes de
     `MIGRATOR.iter()` présentes → `AJour` ; sinon `AAppliquer { nombre, premiere }` (C-v0174m-3). Une migration appliquée
     inconnue ou un checksum différent ne changent pas le plan : `sqlx` refusera après la sauvegarde (angle mort assumé :
     une sauvegarde inutile, jamais dangereuse — R1-17).
   - Seul `AAppliquer` continue.
3. **Échappatoire** : `config.skip_pre_upgrade_backup == Some(CARGO_PKG_VERSION)` → avertissement, `Issue::Contournee`,
   rien d'écrit ; **évaluée avant** le contrôle de version de départ (le manuel promet « vidage manuel puis échappatoire »
   depuis une 0.14.x). Toute autre valeur → avertissement « ignorée », on continue (C-v0174m-4).
4. **Version de départ** : `_kesh_version.kesh_version_last_applied` ; illisible, absente ou `< 0.15.0` →
   `Err(Refus::VersionDeDepart)` (C-v0174m-8).
5. **Dossier** `<KESH_ADMIN_BACKUP_DIR>/pre-upgrade/` créé en `0700` (`DirBuilder` récursif, comme `ecrire`,
   `auto_backup.rs:590`), qui doit **se lister** (`read_dir`, sinon refus : un dossier `0o300` accepterait l'écriture sans
   que la rétention puisse le lister) ; `.partial` abandonnés supprimés (`supprimer_les_partiels(dir, "kesh-pre-upgrade-")`,
   `auto_backup.rs:448`).
6. **Espace** : `seuil_d_espace(max(taille des sauvegardes de auto/ et de pre-upgrade/), taille_des_documents)`
   (`auto_backup.rs:510`, `:496`) contre `BackupEnv::free_bytes` (C-v0174m-7) ; dessous → `Err(Refus::Espace { libre,
   seuil })`.
7. **Lecture** par `kesh_db::backup::export_installation_schema_present(pool)` (ci-dessous), **écriture** par
   `write_keshbackup` dans `write_backup_file` (`routes/admin.rs:754`), sous le nom de C-v0174m-5, avec l'écrivain de
   `BackupEnv::wrap_writer` ; **relecture** par `relire` (`auto_backup.rs:551`). Erreurs : `Refus::Lecture` (base),
   `Refus::Ecriture` (avec la classe `disk_full` de `classify_write_error`, `:250`), `Refus::Relecture` (fichier supprimé),
   `Refus::Tache` (panique du `spawn_blocking`).
8. **Rétention** (C-v0174m-6, C-v0174m-15) : après succès seulement, `pre-upgrade/` garde, parmi les fichiers réguliers au
   nom exact de `NOM_PRE_UPGRADE`, les **trois plus récents** et le **plus ancien du couple courant** `<de>-to-<vers>` ;
   jamais le nouveau ; ni lien, ni autre fichier, ni `.partial`.
9. **Journal** (§ *Messages*, C-v0174m-9) ; `warn` si des pièces manquent (la migration se fait, C-v0174m-20), si le
   fichier dépasse `KESH_ADMIN_IMPORT_MAX_MB`, si une table lue n'est pas dans le `TABLES_TO_TRUNCATE` du binaire
   (C-v0174m-23).

`main.rs` : `Err(refus)` → `tracing::error!` du message de refus, `exit(1)` ; `Ok(_)` → étape 4.

**Dans `kesh-db/src/backup.rs`** : `export_installation_schema_present(pool)` — même `ReadSnapshot`, même cœur
`export_installation_in` (rendu générique sur la liste des tables), la liste lue **dans l'instantané** par
`SELECT TABLE_NAME FROM information_schema.TABLES WHERE TABLE_SCHEMA = DATABASE()` (**sans** filtre `TABLE_TYPE`, comme
la garde `backup_inventory_matches_schema`), filtrée en Rust par `!nom.starts_with('_')` et triée (C-v0174m-24).
`InstallationExport.tables` passe à des noms possédés (`String`), comme `Etape::ApresTable` (son `&'static str` ne
convient plus) ; `KeshBackupMeta.table_count` se calcule sur `export.tables.len()` (`export.rs:330`).
`export_installation` reste le chemin de l'export, de l'import et de la sauvegarde automatique. `build_keshbackup`
(`export.rs:148`) reçoit l'inventaire (`enum Inventaire { Binaire, SchemaPresent }`) plutôt qu'une seconde fonction
d'assemblage.

**Gardes lexicales à adapter** : `export_installation_n_a_qu_un_appelant` (`export.rs:489`) cherche la sous-chaîne
`export_installation(` — que `export_installation_schema_present(` ne contient pas : écrire **une garde par fonction**
(chacune un seul appelant de production) ; `build_keshbackup_ne_lit_la_base_qu_une_fois` (`:464`) couvre les deux
inventaires ; la garde d'instantané de `backup.rs` (« le pool exactement deux fois ») s'étend à la fonction neuve.

**Import à inventaire paramétré** (C-v0174m-17) : `parse_and_verify` (`import.rs:172`) et `restore_tables_in_tx`
(`backup.rs:625`) délèguent à `parse_and_verify_avec_inventaire(reader, limits, inventaire: &[&str])` et
`restore_tables_in_tx_avec_inventaire(tx, tables, inventaire)` ; les deux fonctions publiques d'aujourd'hui passent
`TABLES_TO_TRUNCATE` et restent les seules appelées en production (`routes/admin.rs:430` inchangé). La restauration
parcourt l'inventaire **dans l'ordre de `TABLES_TO_TRUNCATE`** (filtre, l'ordre enfants → parents est préservé).

**Configuration** (C-v0174m-4, C-v0174m-21) : `KESH_SKIP_PRE_UPGRADE_BACKUP` par `env_nonempty` dans `Config::from_env`
(`skip_pre_upgrade_backup: Option<String>`) ; `KESH_SKIP_PRE_UPGRADE_BACKUP: ${KESH_SKIP_PRE_UPGRADE_BACKUP:-}` sous
`environment:` de `kesh-api` dans `docker-compose.yml` et `docker-compose.prod.yml` ; ligne commentée à `.env.example`
(avec la règle « version exacte, à retirer après ») ; listes de `crates/kesh-api/tests/configuration_transmise.rs`
(`EMPLACEMENTS_AUTORISES`, et `VIDE_SIGNIFIANT` : vide = sauvegarde faite) ; manuel § *Variables d'environnement*.

### Nom (C-v0174m-5)

`kesh-pre-upgrade-<AAAAMMJJTHHMMSSmmm>Z-<de>-to-<vers>.keshbackup`, expression **exacte** :

```text
NOM_PRE_UPGRADE = ^kesh-pre-upgrade-[0-9]{8}T[0-9]{9}Z-[0-9A-Za-z.-]{1,40}-to-[0-9A-Za-z.-]{1,40}\.keshbackup$
```

`<de>` et `<vers>` sont **réduits** : chaque caractère hors `[0-9A-Za-z.-]` est **retiré** ; une chaîne vide après
réduction devient `inconnue` ; tronquée à 40 caractères. Le **couple** est le suffixe `-<de>-to-<vers>.keshbackup`.

### Messages (texte exact, journal du serveur)

Quatre lignes `info` au succès (C-v0174m-9) :

- **I1, plan** : `Mise à jour de la base : {n} migration(s) à appliquer (de la {de} à la {vers}){tentative}. Sauvegarde complète avant mise à jour en cours — base et pièces ; le serveur répondra une fois la mise à jour terminée.` — `{tentative}` = ` — tentative n° {k} de cette mise à jour` si `pre-upgrade/` porte déjà `k−1` fichiers du couple, vide sinon.
- **I2, base lue** : `Sauvegarde avant mise à jour : {tables} tables, {lignes} lignes lues en {duree} s.`
- **I3, pièces** : `Sauvegarde avant mise à jour : {pieces} pièces emportées ({octets} octets), {manquantes} non incluses.`
- **I4, fin** : `Sauvegarde avant mise à jour écrite : {chemin} ({taille} octets, en {duree} s). Pour revenir à la {de}, voir le manuel d'administration, « Revenir à la version précédente ».`

Avertissements :

- **W1, pièces** (si `{manquantes}` > 0) : `{manquantes} pièce(s) référencée(s) absente(s), altérée(s) ou illisible(s) ne sont pas dans la sauvegarde avant mise à jour (le journal les nomme) ; la mise à jour continue. Vérifiez le montage du dossier des documents.`
- **W2, taille** : `La sauvegarde avant mise à jour ({taille} Mio) dépasse KESH_ADMIN_IMPORT_MAX_MB ({plafond} Mio) : pour la restaurer par l'écran, la version de départ devra relever ce plafond.`
- **W3, table étrangère** : `Table « {table} » inconnue de Kesh {vers} : elle est dans la sauvegarde avant mise à jour, qui risque de ne pas se restaurer.`
- **W4, échappatoire utilisée** : `KESH_SKIP_PRE_UPGRADE_BACKUP={vers} : migration SANS sauvegarde préalable, à la demande de l'administrateur. Retirez cette variable après la mise à jour.`
- **W5, échappatoire ignorée** : `KESH_SKIP_PRE_UPGRADE_BACKUP={valeur} ignorée : elle ne vaut que pour la version {vers}.`
- **W6, interrompue** : `Une migration interrompue ({version}) est inscrite dans la base : aucune sauvegarde avant mise à jour n'est écrite. Restaurez la sauvegarde d'avant la mise à jour (dossier {dir}) — manuel d'administration, « Revenir à la version précédente ».`

Refus (`error`, puis `exit(1)`), en trois phrases assemblées :

- **R-a** : `FATAL : mise à jour refusée — {cause}. Cette tentative n'a appliqué aucune migration.`
- **R-b**, selon `pre-upgrade/` : sans fichier du couple → ` La base est dans l'état où la {de} l'a laissée : elle peut redémarrer dessus.` ; avec un fichier du couple → ` Une tentative précédente de cette mise à jour a écrit {plus_ancien} : si elle a appliqué des migrations, la {de} ne redémarrera pas sur la base ; restaurez alors ce fichier (manuel d'administration, « Revenir à la version précédente »).` ; version de départ illisible → ` La version de départ n'est pas connue.` (aucun `{de}`).
- **R-c** : ` Corrigez la cause, ou, en connaissance de cause, migrez sans sauvegarde avec KESH_SKIP_PRE_UPGRADE_BACKUP={vers} — sous environment: du service kesh-api (votre compose doit la transmettre : un compose antérieur à la 0.17.4 ne le fait pas), ou par docker compose run --rm -e KESH_SKIP_PRE_UPGRADE_BACKUP={vers} kesh-api (manuel d'administration, « La sauvegarde avant mise à jour »).`

Les `{cause}` (une par variante de `Refus`) : `lecture de l'état des migrations impossible ({erreur})` ;
`version de départ {valeur} : la sauvegarde avant mise à jour ne se restaure que dans une 0.15.0 ou plus récente — faites un vidage de la base`
(variante « illisible ») ; `dossier {dir} : {erreur}` ; `espace insuffisant dans {dir} : {libre} Mio libres, {seuil} Mio requis` ;
`lecture de la base : {erreur}` ; `écriture de {chemin} : {erreur}` ; `relecture de {chemin} : {erreur} (fichier supprimé)` ;
`tâche interrompue : {erreur}`. Aucun ne cite `DATABASE_URL`. Les messages sont produits par **une fonction pure**
`message_de_refus(&Refus, &Contexte) -> String` (et `message_*` pour I1 à W6), testée (T17) : c'est ce qui rend AC-8
vérifiable sans capturer le journal.

## Acceptance Criteria

**AC-1 — Le plan.** Avant `MIGRATOR.run`, après `check_downgrade_protection` : 1146 ou table vide → *neuve* ; une ligne
`success = 0` → *interrompue* ; toutes les migrations montantes présentes → *à jour* ; sinon *à appliquer* avec le
**nombre exact** de migrations absentes ; toute autre erreur de lecture → refus. Seul *à appliquer* déclenche une
sauvegarde. (T1, T2, T3, T4, T16.)

**AC-2 — Le contenu.** Un `.keshbackup` de format 2 dont les tables sont exactement les tables du schéma présent sans
préfixe `_`, chacune avec ses colonnes non générées et toutes ses lignes, les pièces référencées sous `files/`, et
`kesh_version_min_required` lu dans la base. Sur une base arrêtée avant `20261027000001_mcp_prompts`, l'ensemble des
tables est **égal** à `TABLES_TO_TRUNCATE` privé de `mcp_prompts` et `mcp_prompt_versions` (oracle littéral, indépendant
du prédicat de lecture), et l'écriture réussit. (T1, T1-bis.)

**AC-3 — Importable par la version de départ.** Le fichier produit sur cette base passe
`parse_and_verify_avec_inventaire` avec l'inventaire de départ, `check_schema_compat` contre une seconde base au même
état, et sa restauration y redonne, table par table, les mêmes lignes que la base source ; la garde de format rougit à tout
changement de `BACKUP_FORMAT_VERSION`, des clés du manifeste ou des types décodés. (T1-ter, T18.)

**AC-4 — Emplacement et nom.** § *Nom* ; dossier `0700`, fichier `0600`, jamais d'écrasement, `.partial` pendant
l'écriture, `.partial` abandonnés du préfixe supprimés, `.partial` récents laissés. (T1, T10, T9.)

**AC-5 — L'échec refuse la migration.** Plan illisible, version de départ illisible ou `< 0.15.0`, dossier non créable
ou non listable, espace sous le seuil, lecture de la base, écriture, relecture, tâche interrompue : message de refus
(§ *Messages*), code de sortie non nul, **aucune migration appliquée** (`_sqlx_migrations` inchangée), aucun `.partial`
créé par la tentative laissé. (T5, T5-bis, T6, T6-bis, T6-ter, T12, T16, T19, T8.)

**AC-6 — L'échappatoire.** Égale à la version du binaire : aucune sauvegarde, migration appliquée, W4 — y compris avec
une version de départ `< 0.15.0`. Toute autre valeur : W5, sauvegarde écrite comme sans elle. Transmise par les deux
compose, présente à `.env.example`, garde `configuration_transmise` verte. (T7, T8-bis, T14.)

**AC-7 — Rien à migrer, rien d'écrit.** *Neuve*, *à jour*, *interrompue* : aucun fichier, aucun dossier `pre-upgrade/`
créé ; *interrompue* écrit W6. (T2, T3, T4.)

**AC-8 — Rétention et tentatives répétées.** Après une sauvegarde réussie et relue : les trois plus récentes du nom exact
et la plus ancienne du couple courant sont gardées, jamais la nouvelle ; un autre fichier, un lien, un `.partial` récent
ne sont pas touchés ; un échec ne supprime rien. Deux tentatives de suite du même couple, puis trois autres : la première
survit. Le message de refus suit R-b selon la présence d'un fichier du couple. (T9, T9-bis, T17.)

**AC-9 — Le journal.** I1 à I4, W1 à W6 et le refus, avec les valeurs réelles, produits par les fonctions pures de
§ *Messages*. (T17 ; I1 à I4 relevés en recette, sortie copiée.)

**AC-10 — Le manuel et le CHANGELOG.** § *Manuel* et § *CHANGELOG* appliqués au `.tex` et au PDF régénéré ; les jetons
anciens du § *Manuel*, point 9, ne se retrouvent plus dans le PDF aplati là où la story les rend faux.

**AC-11 — La recette** (C-v0174m-19, C-v0174m-22), base de démonstration jetable, au moins une écriture, une facture
fournisseur à document, une facture émise à PDF figé, et un réglage de sauvegarde automatique modifié :

- **Jambe 1, 0.15.0 → branche, compose standard** : `gcorbaz/kesh:0.15.0` → image de la branche (sauvegarde écrite,
  I1 à I4 relevés, migrations appliquées) → `gcorbaz/kesh:0.15.0` refusée sur la base migrée (message relevé) → procédure
  du manuel → la 0.15.0 redémarre, données et pièces revenues ; le réglage de sauvegarde est revenu aux défauts (dit au
  manuel).
- **Jambe 2, 0.17.2 → branche, topologie de production** : conteneur MariaDB autonome sur un réseau jetable, conteneur
  Kesh seul (`--memory 1g`), `DATABASE_URL` sur le compte Kesh (sans root), même enchaînement, la base recréée par le
  compte Kesh. Si cette topologie ne peut pas se monter sur la station, le Dev Agent Record le dit et le manuel marque la
  procédure Synology « non éprouvée ».
- Chaque commande et chaque sortie au Dev Agent Record ; ce que la recette n'établit pas, écrit.

**AC-12 — Aucune migration.** Aucun fichier sous `crates/kesh-db/migrations/` (`ls … | wc -l` = 89 avant et après) ;
`kesh_version_min_required` et `TABLES_TO_TRUNCATE` inchangés.

## Tests et mutations

Rouges **d'abord**, constatés et relevés. Fichier : `crates/kesh-api/tests/sauvegarde_avant_migration.rs`. Il inclut les
helpers de montage de `kesh-db` par `#[path = "../../kesh-db/tests/common/mod.rs"] mod montage;` (comme
`common/binaire.rs` l'est par `#[path]`) — `apply_migrations_up_to`, `migrations_before` (index par **version**, P6) —, et
s'inscrit à `ALLOWED_REAL_MIGRATOR_FILES` de `crates/kesh-db/tests/test_schema_guard.rs` (chemin `crates/kesh-api/tests/…`,
attribut `#[sqlx::test(migrations = false)]` sur une seule ligne). Si l'inclusion `#[path]` tire des dépendances absentes
de `kesh-api`, déplacer les deux fonctions dans un module partagé plutôt que les dupliquer (DRY), et l'écrire.

**Montage commun des bases partielles** : `apply_migrations_up_to(&pool, migrations_before(20261027000001, "mcp_prompts"))`,
puis **`UPDATE _kesh_version SET kesh_version_last_applied = '0.17.3' WHERE id = 1`** (sans quoi la base porte `'0.1.0'`
et tout test rend `Refus::VersionDeDepart`), puis les données en SQL brut. **Assertion de montage** dans chaque test :
le plan rend le nombre attendu (`MIGRATOR.migrations.len() − n`, recalculé, jamais écrit en dur).

| Test | Montage | Assertion |
|---|---|---|
| **T1** | base partielle ; société, utilisateur, compte, une écriture, une `imported_supplier_invoices` dont le document existe dans un dossier temporaire, une facture à `pdf_storage_path` ; `KESH_ADMIN_BACKUP_DIR` temporaire | `AAppliquer { nombre }` exact ; `Issue::Ecrite` ; fichier sous `pre-upgrade/`, nom = `NOM_PRE_UPGRADE`, couple `0.17.3-to-<CARGO_PKG_VERSION>` ; tables du manifeste **=** `TABLES_TO_TRUNCATE \ {mcp_prompts, mcp_prompt_versions}` ; aucune table `_…` ; `rowCount` exacts ; `files` porte les deux pièces ; puis `MIGRATOR.run` réussit |
| **T1-bis** | même base | `export_installation` (inventaire du binaire) y échoue en `Invariant` : documente la prémisse |
| **T1-ter** | le fichier de T1 ; une **seconde** base au même état partiel | `parse_and_verify_avec_inventaire(…, inventaire de départ littéral)` OK ; `check_schema_compat` OK ; restauration par `restore_tables_in_tx_avec_inventaire` ; chaque table : mêmes lignes que la base source (comparaison par export des deux bases) ; et `parse_and_verify` (inventaire du binaire) **refuse** ce fichier (`table 'mcp_prompts' absente du manifeste`) : preuve que l'inventaire diffère réellement |
| **T2** | base entièrement migrée | `AJour`, aucun fichier, `pre-upgrade/` non créé |
| **T3** | base vide | `Neuve`, aucun fichier |
| **T4** | base partielle + ligne `success = 0` | `Interrompue`, aucun fichier |
| **T5** | `KESH_ADMIN_BACKUP_DIR` = fichier régulier | `Refus::Dossier`, rien d'écrit |
| **T5-bis** | `pre-upgrade/` existant en `0o300` | `Refus::Dossier` (non listable), rien d'écrit |
| **T6** | `BackupEnv` : libre = seuil − 1 octet, le seuil étant calculé avec une sauvegarde de 600 Mio (fichier épars) **dans `pre-upgrade/` seulement** ; puis variante avec le fichier dans `auto/` seulement | `Refus::Espace` dans les deux variantes ; avec libre = seuil : écrit |
| **T6-bis** | écrivain qui tronque l'archive | `Refus::Relecture`, fichier supprimé |
| **T6-ter** | écrivain qui rend `StorageFull` | `Refus::Ecriture` (classe `disk_full`), aucun `.partial`, rétention non exécutée |
| **T7** | skip = version du binaire / `0.0.1` / skip = version du binaire **et** `last_applied = '0.14.0'` | `Contournee` sans fichier / `Ecrite` / `Contournee` |
| **T8** (binaire) | base partielle ; `KESH_ADMIN_BACKUP_DIR` = fichier ; `KESH_PORT` occupé par un `TcpListener` du test ; environnement posé : `DATABASE_URL` (celle des tests, base `SELECT DATABASE()`), `KESH_JWT_SECRET`, `KESH_HOST=127.0.0.1`, `KESH_PORT`, `KESH_ADMIN_BACKUP_DIR`, `KESH_LOCALES_DIR` (absolu : `concat!(env!("CARGO_MANIFEST_DIR"), "/../kesh-i18n/locales")`) | code non nul ; la sortie contient `mise à jour refusée — dossier` (la **cause**, pas seulement le préfixe) et **ne contient pas** `version de départ` ; `COUNT(*)` de `_sqlx_migrations` inchangé |
| **T8-bis** (binaire) | même montage, dossier valide, skip = version du binaire | la sortie contient W4 et `Impossible de bind` ; `_sqlx_migrations` porte toutes les migrations |
| **T9** | `pre-upgrade/` : quatre sauvegardes valides d'autres couples, un fichier quelconque, un lien, un `.partial` récent et un `.partial` vieilli de plus de dix minutes (mtime posé par `File::set_modified`) | après une écriture : les trois plus récentes valides (dont la nouvelle), le fichier, le lien, le `.partial` récent intacts, le `.partial` vieilli supprimé ; écriture en échec : rien supprimé |
| **T9-bis** | même base, cinq exécutions de suite du même couple (l'horloge injectée avance d'une milliseconde) | la **première** et les trois dernières subsistent, la deuxième est supprimée |
| **T10** | fonction pure du nom | format exact ; `0.17.0-rc1` gardé ; `1.0.0+build/x` → `1.0.0buildx` ; `+++` → `inconnue` ; 41 caractères → tronqué à 40 ; chaque nom produit satisfait `NOM_PRE_UPGRADE` |
| **T12** | `last_applied` = `0.14.0` / `pas-semver` | `Refus::VersionDeDepart`, aucun dossier créé |
| **T14** | `configuration_transmise` | verte avec la variable neuve |
| **T15** | garde de convention (base squash) | toute table de `information_schema.TABLES` hors `TABLES_TO_TRUNCATE` commence par `_` ; aucune de `TABLES_TO_TRUNCATE` ne commence par `_` (même prédicat que la lecture) |
| **T16** | plan sur un pool fermé (`pool.close().await`) | `Refus::Plan` |
| **T17** | fonctions pures des messages | I1 (avec et sans tentative), I4, W1 à W6, et le refus pour chaque variante de `Refus` × présence d'un fichier du couple : textes exacts du § *Messages* ; « peut redémarrer dessus » absent quand un fichier du couple existe ; aucun `{de}` quand la version est illisible |
| **T18** | garde de format | `BACKUP_FORMAT_VERSION == 2` ; clés du manifeste sérialisé = liste figée ; types de `decode_cell` = liste figée ; message : « changer ceci rend les sauvegardes avant mise à jour inimportables par les versions antérieures : consigner au manuel et au CHANGELOG » |
| **T19** | lecture de la base en échec : l'étape d'écriture (`ecrire_la_sauvegarde(pool, …)`, `pub` pour ce test) appelée, après un plan réussi, avec le pool **fermé** (`pool.close().await`) | `Refus::Lecture`, aucun `.partial`, aucun fichier |

Mutations (sous verrou, restaurées par `git checkout --` puis `touch` ; `mutation-restauree-binaire-perime.md`) :

| Mutation | Tuée par |
|---|---|
| **M1** l'appel retiré de `main.rs` | T8 (`_sqlx_migrations` change) |
| **M2** `main.rs` journalise le refus et continue | T8 |
| **M3** le plan rend toujours `AJour` | T1 |
| **M4** l'inventaire lu dans `TABLES_TO_TRUNCATE` | T1, T1-ter |
| **M5** `Neuve` traitée comme `AAppliquer` | T3 |
| **M6** la branche `Interrompue` retirée | T4 |
| **M7** l'échappatoire acceptée sur `is_some()` | T7 (`0.0.1`) |
| **M8** rétention à quatre récentes / qui supprime la nouvelle | T9 |
| **M8-bis** la rétention supprime aussi un fichier hors expression ou un lien | T9 |
| **M9** contrôle d'espace retiré | T6 |
| **M9-bis** seuil calculé sans `pre-upgrade/` (ou sans `auto/`) | T6 (variante correspondante) |
| **M10** relecture retirée | T6-bis |
| **M13** contrôle de version de départ retiré | T12 |
| **M14** préfixe `_` non exclu | T1 (`_kesh_version` au manifeste) |
| **M15** `Err(Write)` converti en `Ecrite` | T6-ter |
| **M16** plan en erreur traité comme *neuve* | T16 |
| **M17** échappatoire évaluée après la version de départ | T7 (troisième cas) |
| **M18** la plus ancienne du couple non protégée | T9-bis |
| **M19** `supprimer_les_partiels` non appelé | T9 (`.partial` vieilli) |
| **M20** dossier non listable accepté | T5-bis |
| **M21** `Err(Db)` de la lecture converti en `Ecrite` ou ignoré | T19 |
| **M22** R-b « peut redémarrer » écrit même avec un fichier du couple | T17 |
| **M23** `restore_tables_in_tx_avec_inventaire` ignore l'inventaire (parcourt `TABLES_TO_TRUNCATE`) | T1-ter (`DELETE FROM mcp_prompts` échoue) |

(M11 et M12 appartiennent à `v0174-mise-a-jour-2.md`.)

## Tâches

1. **Étape 0** : relire les prémisses de la parente au code (lignes citées au Change Log) ; recréer `kesh_v0174m` /
   `kesh_e2e_v0174m` à neuf (migrations du worktree, `scripts/seed-dev-db.sql`).
2. Tests T1 à T19 en rouge ; relever les rouges.
3. `kesh-db` : inventaire du schéma présent, noms possédés, import à inventaire paramétré, T15, T18.
4. `kesh-api` : module `pre_upgrade_backup` ; `Config::skip_pre_upgrade_backup` ; appel et doc-comment de `main.rs` ;
   visibilités : `seuil_d_espace`, `taille_des_documents`, `relire`, `fichiers_reguliers`, `supprimer_les_partiels`,
   `classify_write_error` (`auto_backup.rs`) et `ensure_backup_dir` (`routes/admin.rs:143`) passent en `pub(crate)`, la
   rétention se généralise (`appliquer_la_retention(dir, expression, garder, proteges)`) au lieu d'une seconde fonction ;
   gardes lexicales adaptées.
5. Compose (deux fichiers), `.env.example`, `configuration_transmise.rs`.
6. Mutations M1 à M23 (hors M11, M12).
7. Manuel `.tex` + PDF (`make fr` sous les deux verrous) ; contrôle `pdftotext` aplati (§ *Manuel*, point 9).
8. CHANGELOG.
9. Recette AC-11 (images Docker sous les deux verrous, réseaux et volumes jetables, jamais `kesh-mariadb-dev`).
10. Gate complet (`test-fast.sh --ci`), Vitest, build, E2E complet au dernier commit de code.

## Manuel

`docs/manual/fr/admin-manual.tex`, puis PDF. Titres **exacts** (cibles des messages) : *La sauvegarde avant mise à jour*
(`sec:sauvegarde-avant-maj`) et *Revenir à la version précédente* (`sec:revenir-version-precedente`).

1. **Nouvelle sous-section** *La sauvegarde avant mise à jour*, après § *La sauvegarde automatique* : depuis la 0.17.4,
   au démarrage d'une version qui doit migrer la base, Kesh écrit d'abord une sauvegarde complète (base et pièces,
   format 2) dans `pre-upgrade/` du dossier de sauvegarde (`./backup/pre-upgrade`, `/volume1/docker/kesh/backup/pre-upgrade`),
   nommée `kesh-pre-upgrade-AAAAMMJJTHHMMSSmmmZ-<de>-to-<vers>.keshbackup` (UTC). À dire :
   - ce qu'elle contient : les tables du schéma **de la version de départ** — elle se restaure dans cette version ; si la
     mise à jour ajoute une table applicative, la version **d'arrivée**, elle, la refuse (§ *Compatibilité*) ;
   - quand elle ne s'écrit pas : installation neuve ; aucune migration à appliquer — la plupart des redémarrages, et toute
     version sans migration, **même si elle répare des données au démarrage** (réparation de l'installation, rattrapages) ;
     migration interrompue ;
   - pendant l'écriture, le serveur ne répond pas : `/health` muet, conteneur « unhealthy » un moment ; attendre la ligne
     « Sauvegarde avant mise à jour écrite » du journal ;
   - durée, taille (≈ base + documents), espace requis (le seuil de la sauvegarde automatique, qui ne mesure pas la base :
     sans sauvegarde antérieure, un disque trop petit fait échouer l'écriture, donc refuser la mise à jour) ; mémoire : la
     base est lue en mémoire — sous le `mem_limit: 1g` du compose de production, une base très grosse fait tuer le conteneur
     (« Killed », code 137) en boucle, sans migration ;
   - prérequis à vérifier **avant** la mise à jour : dossier de sauvegarde **monté** (sinon le fichier vit dans le conteneur
     et disparaît à sa recréation) et dossier des documents monté (sinon les pièces manquent, W1) ;
   - rétention : les trois plus récentes, plus la plus ancienne de la mise à jour en cours (tentatives répétées après un
     échec) ;
   - la relecture ne contrôle que l'annuaire du ZIP et le manifeste ; le test de restauration périodique reste la seule
     preuve ; c'est un **secret**, comme tout `.keshbackup` ; même disque que Kesh : la copier ailleurs ; ni le `.env` ;
   - **si elle échoue**, Kesh ne migre pas et le dit ; l'échappatoire `KESH_SKIP_PRE_UPGRADE_BACKUP=<version exacte>` :
     sous `environment:` du service `kesh-api` (un compose antérieur à la 0.17.4 ne la transmet pas : y ajouter la ligne),
     ou `docker compose run --rm -e KESH_SKIP_PRE_UPGRADE_BACKUP=<version> kesh-api` puis `docker compose up -d` ; à retirer
     ensuite ; depuis une version antérieure à la 0.15.0 : refus, faire un vidage de la base, puis l'échappatoire ;
   - la liste *Sauvegardes sur le serveur* la montre (type « Avant mise à jour ») — **à écrire avec la sous-fiche 2** ; si la
     2 n'est pas livrée, dire qu'elle se récupère dans le dossier.
2. **§ Procédure de mise à jour standard** : étape 1 — la sauvegarde manuelle reste recommandée (hors du disque, avec le
   `.env`) ; depuis la 0.17.4, Kesh écrit lui-même la sauvegarde avant migration (renvoi). Étapes 5 et 6 (« Redémarrer »,
   « Vérifier les logs ») : le serveur ne répond pas pendant la sauvegarde ; lire la ligne de fin. Le `keshwarning` est
   récrit dans ce sens. **Ne pas réécrire les § *Passer à la 0.x*** existants (une mise à jour **vers** la 0.17.3 ou avant
   n'écrit pas cette sauvegarde : `:2518`, `:2606` restent vrais).
3. **§ Rollback en cas d'échec** → **Revenir à la version précédente** (C-v0174m-19) : (1) **avant** d'arrêter Kesh, si
   l'écran répond encore, télécharger le fichier depuis *Sauvegardes sur le serveur* ; sinon le copier par SSH
   (`sudo cp` + `chown`, comme pour la pré-import, `:2329-2333`) ; quel fichier : la plus récente du couple si la version
   de départ a servi depuis la première tentative, sinon la plus ancienne ; (2) `docker compose stop kesh-api` ;
   (3) vidage de sécurité de la base migrée (Synology : `kesh-dump.sh` ; compose standard : `mariadb-dump`,
   § *Script de sauvegarde mariadb-dump*) ; (4) recréer la base vide **par le compte Kesh** (`DROP DATABASE` /
   `CREATE DATABASE <MARIADB_DATABASE> CHARACTER SET utf8mb4 COLLATE utf8mb4_unicode_ci`) : Synology, par le client d'un
   conteneur jetable lisant `kesh-restore.cnf`, comme `kesh-restore.sh` ; compose standard,
   `docker compose exec mariadb mariadb -u"$MARIADB_USER" -p` — pas de `docker compose exec` dans les sections Synology
   (gardes G14 à G16) ; (5) remettre dans le compose l'image `<de>` du nom du fichier, **épinglée** (pas `:latest`),
   `docker compose up -d` ; (6) `/setup` (ou connexion avec `KESH_ADMIN_USERNAME` s'il est posé : pas de `/setup`), puis
   « Restaurer une sauvegarde » de l'assistant (§ *Restaurer sur une nouvelle machine*, plafond d'import) ; le dossier des
   documents reste en place. **Ce que le retour perd** : tout ce qui a été saisi depuis la mise à jour ; les réglages hors
   fichier — planification de la sauvegarde automatique et durée d'inactivité de la session — reviennent aux défauts.
   Garder le paragraphe existant sur la réparation de l'installation. Si la jambe 2 de la recette n'a pu se jouer :
   « procédure Synology non éprouvée ».
4. **§ Compatibilité** (`:2282`) : une phrase sur la sauvegarde avant mise à jour (importable par la version de départ,
   refusée par l'arrivée quand la mise à jour ajoute une table applicative ; importable tant que le format ne change pas,
   ce qu'une release qui le changerait dirait).
5. **§ La sauvegarde automatique**, *Volume* et *Les sauvegardes du serveur* : `pre-upgrade/` s'ajoute au dossier (le
   type à l'écran avec la sous-fiche 2).
6. **`KESH_ADMIN_BACKUP_DIR`** : `:812` (tableau des variables) et `:2377` — ajouter `pre-upgrade/`.
7. **§ Quelle méthode de sauvegarde choisir ?** : « quatre approches » → « cinq approches » (`:2442`) et une ligne au
   tableau.
8. **§ Variables d'environnement complètes** : `KESH_SKIP_PRE_UPGRADE_BACKUP`.
9. **Contrôle** (PDF aplati, `pdftotext docs/manual/fr/admin-manual.pdf - | tr '\n' ' ' | tr -s ' '`), jetons **réels**,
   qui rendent quelque chose à `7bd4d5c0` : `quatre approches` (doit disparaître), `Toujours sauvegarder` (doit être
   récrit), `Rollback en cas d'échec` (doit disparaître), `sous-dossier` près de `auto/` aux deux sites des variables
   (doit nommer `pre-upgrade/`), et les deux titres exacts (doivent apparaître). Le manuel utilisateur n'est pas concerné
   (`git grep -n "Avant import" docs/manual/fr/user-manual.tex` vide).
10. **Autres fichiers** : `.env.example` (commentaires du dossier de sauvegarde et la variable neuve), commentaires des
    deux compose sur `./backup`, `README.md:50` (fonctionnalité *Export/import d'installation* : une phrase sur la
    sauvegarde avant mise à jour).

## CHANGELOG

Sous `## [0.17.4] — Non publié` (créée en tête si absente ; union avec les autres stories), `### Ajouté` :

- **Kesh se sauvegarde lui-même avant de mettre la base à jour** ([#847](https://github.com/guycorbaz/kesh/issues/847)) :
  au démarrage d'une version qui doit migrer la base, une sauvegarde complète — base et pièces — est écrite dans
  `backup/pre-upgrade/` avant toute migration, et se restaure dans la version de départ (procédure au manuel
  d'administration, *Revenir à la version précédente*). Pendant qu'elle s'écrit, le serveur ne répond pas. Si elle échoue
  — dossier, espace disque, ou version de départ antérieure à la 0.15.0 —, Kesh ne migre pas et le dit. Pour migrer quand
  même, en connaissance de cause : `KESH_SKIP_PRE_UPGRADE_BACKUP=<version>`, qui ne vaut que pour cette version — **un
  compose antérieur à la 0.17.4 ne la transmet pas** : reprenez dès maintenant la ligne du nouveau compose dans le vôtre.
  Les trois plus récentes sont gardées, ainsi que la plus ancienne de la mise à jour en cours. La mise à jour 0.17.3 → 0.17.4 n'en écrit pas, la 0.17.4 n'ayant pas de migration
  *(à confirmer à l'intégration)*. La sauvegarde manuelle avant mise à jour reste recommandée (hors du disque, avec le
  `.env`).

## Dev Notes

- **Verrous de station** : toute commande lourde par
  `flock /tmp/station-heavy.lock flock /home/gcorbaz/devel/kesh-gate-logs/.gate.lock scripts/mem-guard.sh …`, une par
  prise, `free -g` avant (sous 12 Go disponibles : attendre).
- **Boucle de redémarrage** (C-v0174m-15) : avec `restart: unless-stopped`, un refus fait redémarrer le conteneur en
  boucle, chaque tentative refaisant le plan et, si l'échec est en aval de la sauvegarde, une sauvegarde ; une migration
  qui échoue **sans** ligne « sale » (avant tout DDL) laisse les précédentes appliquées et produit, aux tentatives
  suivantes, des sauvegardes d'un schéma à moitié migré. La protection de la plus ancienne du couple garde celle d'avant
  la première tentative ; une migration qui échoue **avec** ligne « sale » arrête les sauvegardes (W6).
- **Mémoire** : une seule lecture ; la sauvegarde avant migration a le coût de la sauvegarde automatique (issue demandée,
  parente § *Pour l'orchestrateur*).
- **Mono-instance** : rien ne tourne encore à ce stade (pas d'`AppState`, pas de `backup_lock`).
- **Poste de dev** : une base montée par `sqlx migrate run` + seed porte `last_applied = '0.1.0'` ; sur une base à jour,
  rien ne change ; mais le premier `cargo run` d'une branche qui **ajoute** une migration sur une telle base refuse
  (version de départ `< 0.15.0`) — poser l'échappatoire, ou mettre `last_applied` à jour à la main ; hors Docker,
  `KESH_ADMIN_BACKUP_DIR` doit être inscriptible (l'E2E local pose `/tmp/kesh-e2e/backup`). À écrire aussi dans
  `docs/testing.md` § *Base de dev jetable*.
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

## Dev Agent Record

### Agent Model Used

### Debug Log References

### Completion Notes List

### File List
