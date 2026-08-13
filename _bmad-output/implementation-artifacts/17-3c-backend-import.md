# Story 17.3c: Backend import complet d'installation (`POST /api/v1/admin/full-import`)

Status: done

<!-- Sous-story de l'épopée 17-3 (export/import installation, #112). Extraite de la spec umbrella `17-3-export-import-installation.md` (Partie C), convergée au validate en 5 passes (Sonnet→Haiku→Opus→Sonnet→Haiku, trend 21→16→10→3→0, ~50 patches, dont catch-architectural Opus P3 O-1/O-2/O-3). Contenu déjà adversarialement revu. Re-validate optionnel. -->
<!-- CONSOMME le format `.keshbackup` posé par 17-3a (DONE, `fb43669`). Sous-story la plus risquée (O-1 FK audit, O-2 MIGRATOR no-op, O-5 FK_CHECKS). Dépend de 17-3a (format + cœur `build_keshbackup`). Parallélisable avec 17-3b (UI export). Bloque 17-3d (UI import) + 17-3e (E2E). -->

## Story

As a **administrateur d'une installation Kesh**,
I want **un endpoint backend qui réimporte un fichier `.keshbackup` (produit par l'export 17-3a) sur l'instance courante : validation préalable, backup automatique de l'état actuel, puis restauration destructrice transactionnelle de toutes les données d'installation**,
so that **je puisse migrer ou restaurer une installation complète sans accès SSH/Docker/`mariadb-dump`, avec un filet de sécurité (rollback) en cas d'échec**.

## Contexte & cadrage

**Épopée 17-3 (#112) — split A–F :** 17-3a (backend export, **DONE** `fb43669`) → 17-3b UI export → **17-3c (cette story, backend import)** → 17-3d UI import → 17-3e E2E → 17-3f doc. La spec **umbrella** `17-3-export-import-installation.md` reste la source de contexte complète (décisions DC1–DC11, §Format normatif, risques).

**Rôle :** cette story est la **contrepartie lecture** du format `.keshbackup` figé en 17-3a. Elle lit le manifeste + les NDJSON, valide (structure / SHA / colonnes / version), sauvegarde l'état courant, puis remplace toutes les tables applicatives dans **une transaction**. C'est la sous-story **la plus risquée** de l'épopée (3 catches architecturaux Opus au validate : O-1 FK audit, O-2 MIGRATOR no-op, O-5 FK_CHECKS non re-validées au COMMIT).

**⚠️ Découverte ground-truth (anti-réinvention) :** **Kesh ne stocke AUCUN fichier binaire uploadé sur disque.** Le dossier `files/` du `.keshbackup` est **vide en v0.2** (l'import vérifie qu'il est présent et vide, refus `400` sinon). NE PAS inventer de restauration de fichiers.

**Pas de migration DB** dans cette story (l'import opère sur les tables existantes par DELETE+INSERT ; aucune `.sql` ajoutée → pas d'audit idempotence, pas de bump `kesh_version_min_required`).

## Acceptance Criteria

> Numérotation continue avec l'umbrella (Partie C = AC11–17, + transverses applicables).

1. **(AC11)** `POST /api/v1/admin/full-import` accepte un **upload multipart** (champ `file` = `.keshbackup`), **réservé rôle Admin strict** (non-Admin → `403`) et **interdit via PAT** (`403 API_KEY_MANAGEMENT_FORBIDDEN`, réutiliser `ensure_not_pat` promu `pub(crate)` en 17-3a). Limite de taille via `DefaultBodyLimit::max(...)` configurable `KESH_ADMIN_IMPORT_MAX_MB` (**défaut 512**, bornes `[1, 10240]`, pattern parse+borne+warn de `bank_import_max_mib`). **Succès → HTTP 200** + corps JSON camelCase `{ backupCreated: bool, tablesRestored: number, rowsRestored: number, sourceVersion: string, sessionInvalidated: true }`.

**→ Code renommé par la story 22-4 (#167)** : cette route rend désormais `403 API_KEY_ADMIN_FORBIDDEN` — la couche d'`admin_routes` répond avant le handler, dont le `ensure_not_pat` subsiste (D3 de 22-4a). Énoncé d'origine conservé verbatim.

2. **(AC12)** **Validation pré-restore (ordre strict, AVANT tout DELETE)** :
   - **(a) structure ZIP** conforme (`manifest.json` au root + `data/<table>.ndjson` + dossier `files/` présent et **vide**) → sinon `400 INVALID_BACKUP_STRUCTURE` ; `manifest.formatVersion ≤ 1` sinon `400` (code `INVALID_BACKUP_STRUCTURE` ou dédié) ;
   - **(b) intégrité SHA-256** de chaque table re-calculée sur le NDJSON extrait et comparée à `manifest.tables[t].sha256` → refus `400` (tamper) si mismatch, **sans avoir muté la DB** ;
   - **(c) compat colonnes bidirectionnelle** (via `INFORMATION_SCHEMA.COLUMNS`) :
     - **(c1)** chaque colonne de `columnNames` source ⊆ colonnes destination → sinon `400 IMPORT_SCHEMA_MISMATCH` (`details: { table, unknownColumns: [...] }`) ;
     - **(c2)** chaque colonne destination **`NOT NULL` sans `DEFAULT`** (hors colonnes générées + auto-increment) ⊆ `columnNames` source → sinon `400 IMPORT_SCHEMA_MISMATCH` (`details: { table, missingRequiredColumns: [...] }`) ;
   - **(d) compat version (DC4)** : refus `409` si `manifest.keshVersionMinRequired > version binaire destination` (sémantique exacte downgrade-protection 10-2). **NE PAS** refuser sur `keshVersion > dest` seul. **Nouvelle** fonction `check_import_version_compat(manifest_min_required: &str, binary: &str) -> Result<(), VersionError>` dans `version.rs` (params strings, **pas de lecture DB**) réutilisant le compare `semver::Version` (gère les pré-releases per semver.org, ex. `0.2.0-rc1 < 0.2.0`).

3. **(AC13)** **Backup automatique pré-import (D7/DC5)** : avant toute opération destructrice :
   - **acquérir d'abord un verrou d'installation** (`SELECT … FROM _kesh_version WHERE id = 1 FOR UPDATE` ou `GET_LOCK('kesh_full_import', N)`, pattern Story 17-1) tenu sur **toute la durée backup+restore** (empêche des mutations concurrentes de produire un backup incohérent), relâché après COMMIT/rollback ;
   - exporter l'état actuel via le **cœur factorisé `build_keshbackup(pool)`** (réutilisé tel quel de 17-3a — il **n'émet PAS d'audit**, O-4) ;
   - écrire le `.keshbackup` **sur disque** dans `KESH_ADMIN_BACKUP_DIR` (défaut `/tmp`, créé si absent via `tokio::fs::create_dir_all`), fichier `kesh-pre-import-{timestamp}.keshbackup`. Échec d'écriture → `500` diagnostique (**jamais d'import sans backup réussi**) ;
   - chemin **loggé serveur uniquement** (jamais exposé à l'utilisateur, cf. AC UI 17-3d), conservé sur disque pour diagnostic (purge = opérateur). Si le restore échoue hors-transaction → **rollback** depuis ce backup.

4. **(AC14)** **Restore destructeur transactionnel (DC6)** : dans **une transaction DB** (`pool.begin()`, **une seule connexion** car `FOREIGN_KEY_CHECKS` est session-scoped) :
   - `SET FOREIGN_KEY_CHECKS = 0` ;
   - **`DELETE FROM <table>`** (DML, **PAS** `TRUNCATE` qui est DDL non-rollbackable) pour les **21 tables** applicatives restaurées (ordre `TABLES_TO_TRUNCATE` enfants→parents, indifférent sous FK=0) ;
   - INSERT **paramétrés** (DC1 — `query(...).bind(...)`, **jamais** de SQL concaténé pour les valeurs) avec la **liste de colonnes explicite du manifeste** (`columnNames`, exclut les colonnes générées) ;
   - `SET FOREIGN_KEY_CHECKS = 1` ;
   - audit import (AC16) **dans la même transaction** ;
   - `COMMIT`.
   - **`_sqlx_migrations`, `_kesh_version` et `onboarding_state` ne sont JAMAIS touchées** (ni DELETE ni INSERT). AUTO_INCREMENT non requis (IDs réinsérés explicitement).

5. **(AC15)** **Pas de re-run de migrations** : `MIGRATOR.run()` serait un **no-op structurel** (le restore préserve `_sqlx_migrations` destination ⇒ aucune migration rejouée) — **ne pas l'appeler** (O-2). La compat schéma source↔destination est garantie **en amont** par le double check colonnes d'AC12c.

6. **(AC16)** **Audit-trail import** : l'entrée `audit_log` (`action='admin.full_import'`, `entity_type='installation'` ; `details_json` snake_case : `source_kesh_version`, `source_instance_id`, `triggered_by_user` [identité de l'admin destination ayant lancé l'import, login/id pré-import, **informatif**], `tables_restored`, `rows_restored`) est **insérée DANS la transaction de restore, après les INSERT et avant le COMMIT**, avec un **`user_id` garanti présent dans le dataset source restauré** = **`MIN(users.id) WHERE role='Admin'`** (⚠️ **PAS** `CurrentUser.user_id` destination, qui peut ne pas exister dans la source → violerait la FK `audit_log.user_id → users(id)` NOT NULL). Construire via **`NewAuditLogEntry::user(min_admin_id_source, "admin.full_import", "installation", AUDIT_ENTITY_ID_NONE, Some(details))`** + `audit_log::insert_in_tx(&mut tx, …)`. *(Cas dégénéré : dump altéré sans aucun admin → `MIN(...)` = NULL → INSERT audit viole NOT NULL → rollback `500`. Correct ; impossible pour un dump authentique, et le SHA-256 l'aurait détecté en amont.)*

7. **(AC17)** **Atomicité & cohérence post-import** : restore + audit dans **une seule transaction** ⇒ un échec **rollback tout** → destination intacte, **jamais d'état mi-effacé**. Le backup pré-import (AC13) couvre les échecs **hors-transaction** (crash process/OOM ; au reconnect MariaDB rollback la transaction non-commitée, le backup disque sert de filet ultime). **Post-restore : forcer `onboarding_state` à l'état terminé** (DC11) si le dataset restauré contient ≥1 company non-stub + ≥1 admin (évite la réouverture du catch-22 #120 ; `onboarding_state` étant exclue du restore, elle reste celle de la destination — la forcer « done » couvre le cas d'une destination fraîche/non-onboardée). **Risque assumé (dette B v0.3)** : un `.keshbackup` à intégrité référentielle corrompue mais SHA-256 valide (tamper recalculant le manifeste, ou bug d'export futur) commiterait un état FK-incohérent sans erreur car `FOREIGN_KEY_CHECKS=0` ne re-valide PAS au COMMIT (MariaDB n'a pas de contraintes `DEFERRABLE`, O-5) — accepté car un backup produit par Kesh non-altéré est FK-cohérent par construction.

### Transverses applicables

8. **(AC26 — sécurité)** Le `.keshbackup` n'est **pas chiffré** ; il **contient `users.password_hash` + `refresh_tokens`** → c'est un **secret**. L'import remplace les `refresh_tokens` destination ⇒ **les sessions destination sont invalidées** (`sessionInvalidated: true` dans la réponse, conséquence assumée signalée à l'UI 17-3d). L'auth par cookie (pas de token header custom) ⇒ pas de surface CSRF nouvelle (endpoint POST protégé par `require_admin_role` + cookie SameSite). Doc `///` sur le handler avertissant du caractère sensible.

9. **(Pattern batch — NON applicable, O-8)** L'import est une opération **atomique tout-ou-rien** (une transaction DELETE+INSERT), **pas** un endpoint batch `{accepted, failed}`. Les erreurs (`400`/`409`/`413`/`500`) sont des `AppError` **globales** légitimes invalidant toute la requête en amont — exactement les exceptions globales autorisées par le §Pattern batch de CLAUDE.md. **Pas de `FailedProposal`.**

## Tasks / Subtasks

- [x] **T-C1** Handler `full_import` multipart (`crates/kesh-api/src/routes/admin.rs`, à côté de `full_export`). Extracteur `axum::extract::Multipart` (pattern `bank_imports::parse_multipart`, `field.bytes().await`, garde champ absent/dupliqué). `DefaultBodyLimit::max` sur la route + config `KESH_ADMIN_IMPORT_MAX_MB` (défaut 512, bornes [1,10240], `config.rs` pattern `admin_export_inmem_mib`/`bank_import_max_mib`). RBAC Admin (sub-router `admin_routes`) + **anti-PAT** (`ensure_not_pat` en tête). Réponse succès **200 + JSON** `{ backupCreated, tablesRestored, rowsRestored, sourceVersion, sessionInvalidated: true }` (struct `#[serde(rename_all = "camelCase")]`). (AC: 11)

- [x] **T-C2** Lecture + validation **avant tout DELETE** :
  - parser le ZIP en mémoire (crate `zip` v2, déjà dépendance) : extraire `manifest.json` + chaque `data/<table>.ndjson` + vérifier présence dossier `files/` **vide** → `400 INVALID_BACKUP_STRUCTURE` sinon ;
  - **désérialiser le manifeste** : ajouter `#[derive(Deserialize)]` à `BackupManifest`/`BackupTableMeta` dans `admin_backup/manifest.rs` (actuellement `Serialize` seul — extension anticipée par la spec « 17-3c relira ce manifeste ») ; refus `400` si `formatVersion > 1` ;
  - **SHA-256** par table re-calculé (`exports::metadata::sha256_hex`) vs manifeste → `400` (tamper) si mismatch ;
  - **check colonnes bidirectionnel** AC12c via nouvelle requête `INFORMATION_SCHEMA.COLUMNS` (la fn existante `backup::non_generated_columns` ne lit pas la nullabilité/défaut → **nouveau helper** `kesh_db::backup::column_constraints(pool, table) -> Vec<{name, is_nullable, has_default, extra}>` ou équivalent) : source⊆dest (`unknownColumns`) + dest-NOT-NULL-sans-default-non-générée-non-autoinc⊆source (`missingRequiredColumns`) → `400 IMPORT_SCHEMA_MISMATCH` ;
  - **`check_import_version_compat(min_required, binary)`** nouvelle fn `version.rs` (`409` si `min_required > binary`, gère pré-releases). (AC: 12)

- [x] **T-C3** Backup auto pré-import (DC5) : **acquérir d'abord le verrou d'installation** (`SELECT … FROM _kesh_version WHERE id=1 FOR UPDATE` dans la même tx/connexion que le restore, OU `GET_LOCK`) avant backup+restore ; puis `build_keshbackup(pool)` (réutilisé tel quel, sans audit, O-4) → écrire dans `KESH_ADMIN_BACKUP_DIR` (défaut `/tmp`, `create_dir_all` si absent ; échec écriture → `500`, jamais d'import sans backup). Chemin loggé serveur (`tracing::info!`, jamais dans la réponse). Rollback depuis ce backup si restore hors-transaction échoue. Config `KESH_ADMIN_BACKUP_DIR` (`config.rs`). (AC: 13, 17)

- [x] **T-C4** Restore transactionnel (DC6) : `pool.begin()` → `SET FOREIGN_KEY_CHECKS=0` → **`DELETE FROM`** (pas TRUNCATE) les **21 tables** (`TABLES_TO_TRUNCATE` exclut déjà les tables système ; **exclure aussi `onboarding_state`**, DC11) → INSERT paramétrés colonnes-explicites du manifeste (depuis le NDJSON parsé en `Vec<serde_json::Value>` par ligne, bind par type) → audit import in-tx (T-C6) → `SET FOREIGN_KEY_CHECKS=1` → `COMMIT`. **Ajouter le helper restore transactionnel dans `kesh-db/src/backup.rs`** (`restore_table_in_tx(&mut tx, table, &column_names, &ndjson_rows)` + orchestrateur, consommant `TABLES_TO_TRUNCATE`). ⚠️ tout sur **une seule connexion** (FK_CHECKS session-scoped). Bind des valeurs JSON → SQL : `null`→NULL, string→bind string (MariaDB recoerce DECIMAL/DATETIME/etc. depuis la représentation string fidèle de l'export), nombre→bind i64/f64. (AC: 14, 17)

- [x] **T-C5** **NE PAS appeler `MIGRATOR.run()`** (no-op structurel, O-2). Confirmer par commentaire `///` que la compat schéma est assurée par le double check colonnes T-C2. (AC: 15)

- [x] **T-C6** Audit `admin.full_import` **dans la transaction restore** (après les INSERT, avant COMMIT) : `user_id = MIN(users.id) WHERE role='Admin'` **lu sur le dataset déjà restauré** (donc requête après les INSERT, dans la tx), via `NewAuditLogEntry::user(min_admin_id, "admin.full_import", "installation", AUDIT_ENTITY_ID_NONE, Some(json!({... snake_case ...})))` + `audit_log::insert_in_tx`. PAS `from_current_user` (O-1, FK). `details_json` : `source_kesh_version`, `source_instance_id`, `triggered_by_user`, `tables_restored`, `rows_restored`. (AC: 16)

- [x] **T-C7** Tests intégration DB (`crates/kesh-api/tests/admin_full_import_e2e.rs` ou helpers dans le e2e roundtrip 17-3e ; au minimum couvrir ici les invariants backend) :
  - **round-trip** : seed → export (`build_keshbackup`) → DELETE-all → import → équivalence row counts + login admin source possible + FK intègres ;
  - **O-1** : ids users source ≠ admin dest → audit import OK (pas de viol FK `audit_log.user_id`) ;
  - refus `409` version incompatible, `400` SHA tamper, `400` format inconnu (`formatVersion=2`), `400 IMPORT_SCHEMA_MISMATCH` (colonne dest NOT NULL absente de la source / colonne source inconnue de la dest) ;
  - **rollback transactionnel** sur échec injecté (destination intacte) ;
  - **DC11/O-3** : import d'un backup onboarding-incomplet sur instance onboardée → `onboarding_state` reste « done » (pas de catch-22) ;
  - RBAC non-Admin `403`, anti-PAT `403`. (AC: 12, 14, 16, 17)

## Dev Notes

### Décisions de conception (figées au validate umbrella — voir spec parente pour le détail complet)

| # | Décision (périmètre 17-3c) |
|---|---|
| **DC1** | Restore via **INSERT paramétrés** avec `columnNames` du manifeste (jamais de SQL concaténé pour les valeurs). NDJSON re-parsé ligne→`serde_json::Value`, bind par type. |
| **DC2** | Scope = **22 tables exportées, 21 restaurées** (`onboarding_state` exclue du restore, DC11). |
| **DC4** | Compat version : refus `409` **ssi `manifest.keshVersionMinRequired > version binaire dest`**. Nouvelle fn `check_import_version_compat` (strings, pas de lecture DB). |
| **DC5** | Backup auto pré-import (réutilise `build_keshbackup`) → `KESH_ADMIN_BACKUP_DIR` + verrou installation. Jamais d'import sans backup réussi. |
| **DC6** | Restore **dans une transaction** : `FK_CHECKS=0` → **`DELETE FROM`** (pas TRUNCATE) → INSERT colonnes-explicites → `FK=1` → COMMIT. Tout sur **une connexion**. |
| **DC7** | Admin strict **+ anti-PAT** (`403 API_KEY_MANAGEMENT_FORBIDDEN`). |
| **DC9** | Aucune migration DB. |
| **DC10** | Audit import **dans la tx**, `user_id = MIN(admin)` source (PAS CurrentUser dest, FK). `refresh_tokens` restaurés ⇒ sessions dest invalidées (`sessionInvalidated: true`). |
| **DC11** | `onboarding_state` **exclue du restore** ; **forcée « done »** post-restore si ≥1 company non-stub + ≥1 admin (anti catch-22 #120). |

**→ Code renommé par la story 22-4 (#167)** : `DC7` reste vrai dans son principe — ces opérations d'infra restent interdites aux clés API —, mais le code rendu est désormais `403 API_KEY_ADMIN_FORBIDDEN`, la couche d'`admin_routes` répondant avant le handler. Le `ensure_not_pat` du handler subsiste (décision D3 de 22-4a). Tableau conservé verbatim.

### Format `.keshbackup` (contrat figé par 17-3a — à RELIRE exactement)

> Source de vérité : `17-3a-backend-export.md` §Format normatif + `crates/kesh-api/src/admin_backup/`. Tout écart de lecture casse l'intégrité SHA.

**Structure ZIP** : `manifest.json` (root) + `data/<table>.ndjson` (1 par table, nom = nom exact table) + `files/` (vide). Refus `400 INVALID_BACKUP_STRUCTURE` si différent.

**`manifest.json`** (camelCase) — schéma réel produit par 17-3a (`admin_backup/manifest.rs`) :
```rust
struct BackupManifest {           // #[serde(rename_all = "camelCase")] — AJOUTER Deserialize
    format_version: u32,          // figé 1 ; import refuse 400 si > 1
    kesh_version: String,
    kesh_version_min_required: String,  // → check_import_version_compat
    instance_id: i64,             // informatif → details_json.source_instance_id
    export_date: String,
    tables: BTreeMap<String, BackupTableMeta>,
}
struct BackupTableMeta {          // #[serde(rename_all = "camelCase")] — AJOUTER Deserialize
    row_count: usize,
    sha256: String,               // hash des bytes NDJSON décompressés (LF)
    column_names: Vec<String>,    // ordonné, exclut colonnes générées → liste INSERT
}
```

**NDJSON** : 1 objet JSON par ligne (UTF-8, LF), ordre des clés = `column_names`. `null`→NULL, `Decimal`→string, `NaiveDateTime`→`"YYYY-MM-DDTHH:MM:SS[.mmm]"` (précision sub-seconde `DATETIME(3)`), entiers→nombre. Table vide → fichier 0 octet → sha256 de la chaîne vide (`e3b0c4…`). **Le bind à l'INSERT re-bind la string telle quelle** (MariaDB coerce DECIMAL/DATETIME depuis string fidèle).

**SHA vérifié AVANT tout DELETE** (sur NDJSON extraits) → `400` tamper sans muter la DB.

### Inventaire tables — réutilisation `kesh_db::backup` (posé par 17-3a, ground-truth `crates/kesh-db/src/backup.rs`)

`pub const TABLES_TO_TRUNCATE: &[&str]` (22 entrées, ordre enfants→parents) est **déjà `pub`** dans `kesh-db/src/backup.rs:32`. Restore : DELETE dans l'ordre, **exclure `onboarding_state`** (DC11) → 21 tables. ⚠️ `accounts` a une FK self-référente (`parent_id`) → `FK_CHECKS=0` indispensable. ⚠️ Colonnes générées (`reconciliation_rules.active_uniq` VIRTUAL) **jamais** dans `columnNames` (garanti par l'export 17-3a) → l'INSERT ne les touche pas.

### Réutilisation — briques 17-3a déjà en place (ground-truth)

| Brique | Chemin:ligne | Usage 17-3c |
|---|---|---|
| Cœur backup sans audit | `admin_backup/export.rs:40` `build_keshbackup(pool) -> Result<(Vec<u8>, KeshBackupMeta), AppError>` | **Directe** (T-C3 backup pré-import) — retourne `Vec<u8>` in-mem, à écrire sur disque |
| Manifeste | `admin_backup/manifest.rs:18` `BackupManifest` / `:30` `BackupTableMeta` (`Serialize` seul) | **AJOUTER `Deserialize`** pour relire ; `BACKUP_FORMAT_VERSION` const `:15` |
| Tables canoniques | `kesh-db/src/backup.rs:32` `pub TABLES_TO_TRUNCATE` | Énumération DELETE/INSERT |
| Sérialiseur colonnes | `kesh-db/src/backup.rs:75` `non_generated_columns` (privé, (name, data_type) **sans** nullabilité) | **Insuffisant** → nouveau helper nullabilité-aware pour AC12c |
| SHA-256 | `exports/metadata.rs:89` `sha256_hex` (`sha2` 0.10) | Vérif intégrité (T-C2) |
| Anti-PAT | `routes/api_keys.rs` `pub(crate) ensure_not_pat(&CurrentUser)` (promu 17-3a) | **Directe** en tête handler (AC11) |
| Audit explicit user_id | `entities/audit_log.rs:136` `NewAuditLogEntry::user(user_id, …)` + `repositories/audit_log.rs:29` `insert_in_tx(&mut tx, …)` | **O-1** : user_id = MIN(admin) source, PAS from_current_user |
| `admin_routes` sub-router | `lib.rs:101-127` `route_layer(require_admin_role)` ; route GET full-export déjà montée | Ajouter `POST /api/v1/admin/full-import` + `DefaultBodyLimit` |
| Compat SemVer | `version.rs:182` `check_downgrade_protection` (lit la DB, ≠ usage) + `VersionError` enum `:29` | **Écrire** `check_import_version_compat(min_required:&str, binary:&str)` réutilisant `semver::Version` compare (pas de lecture DB) ; mapper en `409` |
| Config | `config.rs` `admin_export_inmem_mib` (posé 17-3a) + `bank_import_max_mib:196` pattern parse+borne+warn | `KESH_ADMIN_IMPORT_MAX_MB` (512, [1,10240]) + `KESH_ADMIN_BACKUP_DIR` (`/tmp`) |
| Multipart | `routes/bank_imports.rs:348-507` `parse_multipart` (`field.bytes().await`, gardes) ; `DefaultBodyLimit::max` `lib.rs:213-215` | Upload import (AC11) |
| `AppError` | `errors.rs` : `AdminFullExportFailed:244`→500, `ApiKeyManagementForbidden:100`→403 | **Ajouter** variants import : `AdminFullImportFailed`(→500), `InvalidBackupStructure`(→400), `ImportSchemaMismatch{details}`(→400), `ImportVersionIncompatible`(→409). Codes i18n FR/DE/IT/EN |

### Construction des erreurs typées (AC12)

- `400 INVALID_BACKUP_STRUCTURE` — ZIP malformé, manifest absent, `files/` non-vide, `formatVersion > 1`, NDJSON manquant pour une table du manifeste.
- `400 IMPORT_SCHEMA_MISMATCH` — `details: { table, unknownColumns }` (c1) **ou** `{ table, missingRequiredColumns }` (c2).
- `400` (tamper SHA) — réutiliser `INVALID_BACKUP_STRUCTURE` avec `details: { table, expectedSha, actualSha }` ou un code dédié `BACKUP_INTEGRITY_MISMATCH` (au choix dev, documenter).
- `409 IMPORT_VERSION_INCOMPATIBLE` — `details: { sourceMinRequired, binaryVersion }`.
- `413` — géré par `DefaultBodyLimit` (Axum renvoie `PAYLOAD_TOO_LARGE` automatiquement).
- `500 ADMIN_FULL_IMPORT_FAILED` — échec backup pré-import, échec DB transaction, MIN(admin)=NULL (dump sans admin).

### Standards projet (CLAUDE.md)

- **Test Locally First** (modif touche `kesh-db` + tests d'intégration → **mode serial**) : `cargo fmt --all --check` + `cargo build --workspace --all-targets` + `cargo clippy --workspace --all-targets -D warnings` + `cargo test --workspace -j1 -- --test-threads=1`. (DATABASE_URL via env settings → conteneur `kesh-mariadb` local.)
- **Migration breaking policy** : N/A (DC9, aucune migration).
- **Pattern batch `FailedProposal`** : **N/A** (O-8, opération atomique tout-ou-rien ; erreurs globales légitimes).
- **Commit par étape BMAD**, pas de push auto avant fin de cycle.
- Branche active : `story/17-3-export-import-installation` (stack des sous-stories 17-3, où 17-3a est déjà mergée).
- **Sécurité** : ne jamais logger le contenu du backup (secrets) ni exposer le chemin du backup pré-import dans la réponse HTTP.

### Project Structure Notes

- **Backend** : handler `full_import` ajouté à `crates/kesh-api/src/routes/admin.rs` (à côté de `full_export`). Logique de restore + helpers colonnes/INSERT dans `crates/kesh-db/src/backup.rs` (co-localisé avec `TABLES_TO_TRUNCATE` + `export_table`). Validation/orchestration import éventuellement dans `crates/kesh-api/src/admin_backup/import.rs` (symétrique de `export.rs`).
- **Router** : `POST /api/v1/admin/full-import` monté dans `admin_routes` (RBAC câblé), avec `DefaultBodyLimit` spécifique.
- **Aucun conflit** avec 9-2b (per-company) ni 17-3b (UI export, parallèle).

### References

- [Source: _bmad-output/implementation-artifacts/17-3-export-import-installation.md] — spec umbrella (DC1–DC11, Partie C AC11-17, §Format normatif, risques O-1..O-10, Change Log 5 passes)
- [Source: _bmad-output/implementation-artifacts/17-3a-backend-export.md] — story-zéro (format figé, cœur `build_keshbackup`, promotion `TABLES_TO_TRUNCATE`)
- [Source: crates/kesh-api/src/admin_backup/export.rs, manifest.rs] — `build_keshbackup`, `BackupManifest`/`BackupTableMeta`
- [Source: crates/kesh-db/src/backup.rs] — `TABLES_TO_TRUNCATE` (pub), `export_table`, `non_generated_columns`, `read_min_required`, `read_instance_id`
- [Source: crates/kesh-db/src/version.rs:182] — `check_downgrade_protection` + `VersionError` (modèle pour `check_import_version_compat`)
- [Source: crates/kesh-db/src/entities/audit_log.rs:136] — `NewAuditLogEntry::user` (user_id explicite)
- [Source: crates/kesh-api/src/routes/bank_imports.rs:348-507] — multipart upload pattern
- [Source: crates/kesh-api/src/routes/admin.rs] — handler `full_export` (modèle handler + anti-PAT + audit)
- [Source: CLAUDE.md] — Test Locally First, Migration breaking policy (N/A), Pattern batch (N/A), commit/branch

## Dev Agent Record

### Agent Model Used

Opus 4.8 (claude-opus-4-8[1m]) — single-pass orchestré T-C1→T-C7.

### Debug Log References

Quality gate (DATABASE_URL local `kesh-mariadb` Docker) : `cargo fmt --all --check` OK, `cargo clippy --workspace --all-targets -D warnings` 0 warning, tests ciblés verts (kesh-db backup/version, kesh-api admin_backup unit, **admin_full_import_e2e 9/9**), `cargo test --workspace -j1 --test-threads=1` (non-régression — voir Completion Notes).

### Completion Notes List

- **Couche kesh-db (`backup.rs`)** : ajout symétrique de l'export — `parse_ndjson_rows` (NDJSON → lignes ordonnées par `column_names`, clé absente → null), `column_constraints` + `ColumnConstraint::is_required()` (NOT NULL sans défaut, non-générée, non-auto-incr) pour le check AC12c, `restore_tables_in_tx` (DELETE+INSERT paramétrés sous `FOREIGN_KEY_CHECKS=0`, **`onboarding_state` exclue** DC11, **rétablissement systématique de FK=1** même sur erreur via capture du résultat), `bind_json_value` (fidélité de type DC1 : null/bool/i64/u64/f64/string ; objet→re-string défensif), `force_onboarding_done_if_eligible` (DC11 : `step_completed=8` si ≥1 company non-stub + ≥1 admin).
- **`version.rs`** : `check_import_version_compat(min_required, binary)` **pure** (pas de DB), refus `DowngradeRefused` ssi `min_required > binary` (DC4), gère pré-releases SemVer. 4 tests unit.
- **`admin_backup/import.rs`** (nouveau) : `parse_and_verify` (structure ZIP stricte — entrée inattendue refusée, `files/` non-vide refusé, manifeste désérialisé, `formatVersion ≤ 1`, couverture des 22 tables, **SHA-256 par table avant tout DELETE**, rowCount cohérent) + `check_schema_compat` (bidirectionnel c1/c2 → `IMPORT_SCHEMA_MISMATCH`). 6 tests unit.
- **`manifest.rs`** : `Deserialize` ajouté à `BackupManifest`/`BackupTableMeta` (relecture import).
- **Handler `routes/admin.rs::full_import`** : multipart (champ `file`, dup refusé) → anti-PAT → `parse_and_verify` → compat version (409 / 400 si SemVer illisible) → `check_schema_compat` (400) → **verrou `GET_LOCK('kesh_full_import', 10)`** sur connexion dédiée (sérialise backup+restore, relâché tous chemins) → **backup pré-import** (`build_keshbackup` sans audit → `KESH_ADMIN_BACKUP_DIR`, échec=500) → **restore transactionnel** + **garde de cohérence** (rows insérées = rows backup hors `onboarding_state`) + **audit in-tx** `user_id=MIN(admin) source` (O-1) + DC11 + COMMIT. Réponse 200 `{ backupCreated, tablesRestored, rowsRestored, sourceVersion, sessionInvalidated:true }`.
- **`config.rs`** : `admin_import_max_mib` (512, [1,10240]) + `admin_backup_dir` (`/tmp`) + parsing env + 3 builders.
- **`errors.rs`** : 4 variants (`AdminFullImportFailed`→500, `InvalidBackupStructure`→400, `ImportSchemaMismatch{details}`→400, `ImportVersionIncompatible{details}`→409) + i18n FR/DE/IT/EN ×4 clés.
- **`lib.rs`** : route `POST /api/v1/admin/full-import` dans `admin_routes` + `DefaultBodyLimit` propre.
- **Tests E2E** (`admin_full_import_e2e.rs`, 9/9) : round-trip remplacement + **O-1** (audit user_id = admin source ≠ caller, pas de viol FK), RBAC non-Admin 403, anti-PAT 403, refus 409 version / 400 SHA tamper / 400 format / 400 IMPORT_SCHEMA_MISMATCH, **rollback** (ide invalide → 500 → destination intacte), DC11 onboarding forcé done.
- **Pattern batch `FailedProposal`** : N/A (O-8, opération atomique tout-ou-rien).
- **Aucune migration DB** (DC9). **Note v0.3** : INSERT ligne-par-ligne (pas de batch multi-VALUES) — acceptable PME, optimisation batch documentée pour les grosses installs.

### File List

**Nouveaux fichiers :**
- `crates/kesh-api/src/admin_backup/import.rs` — `parse_and_verify` + `check_schema_compat` + `ParsedBackup` + tests unit.
- `crates/kesh-api/tests/admin_full_import_e2e.rs` — 9 tests E2E HTTP.

**Fichiers modifiés :**
- `crates/kesh-db/src/backup.rs` — `parse_ndjson_rows`, `column_constraints`/`ColumnConstraint`, `TableRestore`, `restore_tables_in_tx`/`restore_body`/`bind_json_value`, `force_onboarding_done_if_eligible` + tests.
- `crates/kesh-db/src/version.rs` — `check_import_version_compat` + 4 tests.
- `crates/kesh-api/src/admin_backup/mod.rs` — `pub mod import;`.
- `crates/kesh-api/src/admin_backup/manifest.rs` — `Deserialize` sur `BackupManifest`/`BackupTableMeta`.
- `crates/kesh-api/src/routes/admin.rs` — handler `full_import` + `read_upload`/`run_backup_and_restore`/`write_pre_import_backup`.
- `crates/kesh-api/src/config.rs` — `admin_import_max_mib` + `admin_backup_dir` + parsing + builders.
- `crates/kesh-api/src/errors.rs` — 4 variants import + arms.
- `crates/kesh-api/src/lib.rs` — route `POST /api/v1/admin/full-import` + `DefaultBodyLimit`.
- `crates/kesh-i18n/locales/{fr,de,en,it}-CH/messages.ftl` — 4 clés d'erreur import.

### Change Log

| Date | Étape | Modèle | Résumé |
|------|-------|--------|--------|
| 2026-06-09 | create-story (sous-story) | Opus 4.8 | Story 17-3c (backend import) extraite de l'umbrella 17-3 Partie C (convergée 5 passes, contenu déjà adversarialement revu dont catches Opus P3 O-1/O-2/O-3/O-5). Ancrée sur le code réel de 17-3a (DONE) : `build_keshbackup` (Vec<u8> sans audit) réutilisé pour backup pré-import, `BackupManifest` à étendre `Deserialize`, `TABLES_TO_TRUNCATE` pub, `NewAuditLogEntry::user` pour audit user_id-explicite (O-1 FK), `check_downgrade_protection` modèle pour `check_import_version_compat`. AC11-17 + transverses. T-C1..T-C7. Validation pré-restore stricte (structure/SHA/colonnes bidir/version), restore transactionnel DELETE (pas TRUNCATE), audit in-tx, DC11 onboarding. Re-validate optionnel. Prochaine : `bmad-dev-story 17-3c` (Opus recommandé — restore transactionnel + bind dynamique 21 tables). |
| 2026-06-09 | code-review (cycle) | Sonnet→Haiku→Opus→Sonnet→Haiku | **CYCLE CONVERGÉ en 5 passes**, trend > LOW **~8→~6→1→2→0**. **P1 Sonnet** (3 reviewers, 0 CRITICAL) : verrou GET_LOCK→`_kesh_version FOR UPDATE` in-tx (fuite panic + 2 conn), u64>i64::MAX rejeté, `force_onboarding` UPDATE→upsert `ON DUPLICATE KEY GREATEST` (couvre row absente = catch-22 #120 + step<terminal), check couverture réciproque, `files/` présence+sous-dossier, garde backtick colonnes, +2 tests E2E (c2 missingRequiredColumns + login admin source). **P2 Haiku** (0 hallucination, 2 HIGH grep-vérifiés) : FOR UPDATE row absente→refus, f64 `unwrap_or(0.0)`→erreur, collision timestamp backup→compteur atomique+pid. **P3 Opus catch-architectural** (EdgeCase+Acceptance 0>LOW, Blind 1 MEDIUM) : **fidélité DECIMAL** — nombre non-entier bindé en string décimale exacte (évite arrondi binaire f64 sur colonne DECIMAL), `is_demo=FALSE` à l'upsert onboarding, skip-sans-incrément tables_restored, saturating_mul body-limit. **P4 Sonnet** (EdgeCase 0>LOW, Blind+Acceptance MEDIUM convergent) : **backup pré-import déplacé APRÈS le verrou FOR UPDATE** (AC13 « acquérir d'abord » — élimine torn-read inter-imports), skip `onboarding_state` dans `check_schema_compat`, doc GET_LOCK→FOR UPDATE. **P5 Haiku** (Blind 0>LOW, Acceptance 0>LOW READY, EdgeCase 1 HIGH **réfuté grep ground-truth** : skip schéma onboarding_state sans risque — table jamais restaurée [backup.rs:411/425], `force_onboarding` colonnes hardcodées [:544], columnNames manifeste jamais consommées — faux-positif auto-contradictoire). ~13 patches. Quality gate final : fmt OK, clippy 0, `cargo test --workspace -j1 --test-threads=1` 0 régression. Status → done. |
| 2026-06-09 | dev-story | Opus 4.8 | Implémentation single-pass T-C1→T-C7. Couche kesh-db (`parse_ndjson_rows`/`column_constraints`/`restore_tables_in_tx` avec FK=1 garanti/`bind_json_value`/`force_onboarding_done_if_eligible`) + `check_import_version_compat` (pure). Module `admin_backup/import.rs` (`parse_and_verify` validation pré-DELETE stricte + SHA + couverture + `check_schema_compat` bidirectionnel). Handler `full_import` (multipart, anti-PAT, GET_LOCK serialise backup+restore, backup pré-import disque, restore transactionnel + garde cohérence rows + audit in-tx O-1 + DC11). 4 variants AppError + i18n ×4. 2 nouveaux fichiers + 9 modifiés. **Quality gate** : fmt OK, clippy 0 warning, **admin_full_import_e2e 9/9** (round-trip+O-1, RBAC, anti-PAT, refus 409/400×3, rollback, DC11), kesh-db backup/version + import unit verts, `cargo test --workspace -j1 --test-threads=1` non-régression. 2 bugs auto-détectés+corrigés pendant le dev (garde cohérence comptait `onboarding_state` exclue ; test rollback déclenchait IMPORT_SCHEMA_MISMATCH avant l'INSERT). Status review. Prochaine : `bmad-code-review 17-3c` (Sonnet 4.6, LLM différent). |
