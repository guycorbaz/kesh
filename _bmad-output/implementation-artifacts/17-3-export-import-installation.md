# Story 17.3: Export/import complet d'une installation Kesh via l'UI admin

Status: ready-for-dev

<!-- Note: Validation is optional. Run validate-create-story for quality check before dev-story. -->
<!-- STORY UMBRELLA (parente) — split quasi-certain au `bmad-create-story validate` (cf. Epic 17 D3, CLAUDE.md §Règle de splitting préventif). Découpage pressenti 17-3a..17-3f aligné sur les Parties A–F ci-dessous. Le format `.keshbackup` (contrat transverse export↔import) est conçu d'un bloc dans cette spec ; les frontières sous-story se cristallisent au validate. -->

## Story

As a **administrateur d'une installation Kesh**,
I want **exporter toute l'installation (toutes companies + tous utilisateurs + données système) dans un fichier `.keshbackup` unique, et le réimporter sur une autre instance via l'interface d'administration**,
so that **je puisse migrer ou sauvegarder l'intégralité de mes données sans accès SSH/Docker/`mariadb-dump`, en conformité OLICo Art. 6 (disponibilité)**.

## Contexte & cadrage (à lire avant tout)

**Issue source :** [#112](https://github.com/guycorbaz/kesh/issues/112) (`enhancement`, `v0.2-milestone`). Provenance : demande Guy 2026-05-24 — « il doit être possible d'exporter et d'importer toutes les données d'une installation kesh dans un fichier afin de pouvoir migrer […] via l'interface d'administration ».

**Décisions Epic 17 applicables (cf. `_bmad-output/planning-artifacts/epic-17.md`) :**
- **D6** — Export/import = **format unifié auto-portant**, scope **installation complète Admin**. **PAS de chevauchement** avec l'export per-company Story 9-2b (`GET /api/v1/exports/global.zip`, CSV, 16 tables, scopé 1 company, exclut users/audit_log). 9-2b reste pour les users non-Admin ; #112 = Admin, installation entière.
- **D7** — Import = **backup automatique pré-import** (rollback safety net) **+ validation compat version** (réutilise `_kesh_version` Story 10-2). Refus si downgrade impossible. Modal de confirmation forte côté UI.

**⚠️ Découverte ground-truth critique (anti-réinvention) :** **Kesh v0.1/v0.2 ne stocke AUCUN fichier binaire uploadé sur disque.**
- Imports bancaires : seul le **hash SHA-256** est conservé en DB (`bank_imports.file_hash`), **jamais le fichier original** (cf. `crates/kesh-db/src/entities/bank_import.rs`).
- PDFs de factures : générés **on-demand en mémoire** et streamés (`crates/kesh-api/src/routes/invoice_pdf.rs`), **aucun cache disque**.
- **Aucune** variable d'env `KESH_UPLOAD_DIR` / `KESH_DATA_DIR` n'existe.
- **Conséquence :** le volet « fichiers binaires uploads » des ACs de #112 est un **no-op en v0.2**. **NE PAS** inventer de mécanisme de sauvegarde de fichiers : toute la donnée d'une installation vit en base. Le format `.keshbackup` documente ce fait et réserve (forward-compat) un dossier `files/` vide. Cela réduit drastiquement le scope réel.

**Migrations DB :** cette story **n'introduit aucune nouvelle migration** (export/import opèrent sur les tables existantes). Donc : pas de mise à jour `docs/migrations-idempotence-audit.md`, pas de bump `kesh_version_min_required` (cf. CLAUDE.md §Migration breaking policy — N/A ici).

## Décision de split (actée au validate Pass 1)

Le split **A–F est confirmé**, 6 sous-stories :
- **17-3a** backend export (`admin_backup/` + `GET /admin/full-export`) — **story-zéro** : pose le format `.keshbackup` que tout consomme. **Doit merger en premier.**
- **17-3b** UI export (`/admin/backup`) — dépend de 17-3a.
- **17-3c** backend import (`POST /admin/full-import` + restore) — dépend de 17-3a (format).
- **17-3d** UI import (`/admin/restore`) — dépend de 17-3c.
- **17-3e** E2E end-to-end (test d'intégration Rust) — dépend de 17-3a + 17-3c.
- **17-3f** doc — dépend de toutes.

**17-3b et 17-3c sont parallélisables** après le merge de 17-3a (aucune dépendance entre elles).

## Acceptance Criteria

> ACs groupés par **Partie A–F** (= frontières de split). Numérotation continue pour traçabilité.

### Partie A — Backend export (`GET /api/v1/admin/full-export`)

1. `GET /api/v1/admin/full-export` retourne un fichier `.keshbackup` (conteneur ZIP) téléchargeable (**GET** — pas de corps de requête, cohérent 9-2b `GET /exports/global.zip` et `apiClient.getBlob` qui est câblé sur GET), `Content-Type: application/octet-stream` (force le download sans réinterprétation d'extension) + `Content-Disposition: attachment; filename="kesh-installation-{YYYY-MM-DD}.keshbackup"`. **Réservé rôle Admin strict** (test : `Comptable` et `Consultation` → `403`).
2. L'endpoint est **inaccessible via clé PAT** : une requête authentifiée par `Authorization: Bearer kesh_pat_…` (même scope `read-write`) → `403` code `API_KEY_MANAGEMENT_FORBIDDEN` (cohérent Story 17-2a DC6 — opérations d'infra interdites aux PAT ; le backup contient hash de mots de passe + tokens, fuite critique si exposé via API).
3. Le `.keshbackup` contient : (a) un fichier de données NDJSON **par table applicative Kesh** (les **22 tables**, voir Dev Notes §Inventaire), (b) un `manifest.json` (métadonnées + intégrité), (c) un dossier `files/` (vide en v0.2, forward-compat). Tables **système exclues** : `_sqlx_migrations`, `_kesh_version`.
4. `manifest.json` (schéma JSON camelCase, pretty-printed) contient :
   - `formatVersion: 1` (entier **figé** = format NDJSON-per-table introduit par 17-3a ; l'import refuse `400` si `> 1`),
   - `keshVersion` (= `env!("CARGO_PKG_VERSION")` source), `keshVersionMinRequired` (lu de `_kesh_version`), `exportDate` (ISO 8601 UTC `…Z`),
   - `instanceId` (identifiant de l'installation source pour traçabilité migration : `id` de la 1ʳᵉ `company`, toujours présente),
   - par table : `rowCount` (usize), `sha256` (hash des bytes NDJSON décompressés), **`columnNames: string[]`** (liste **ordonnée et explicite** des colonnes sérialisées — **exclut les colonnes générées** type `reconciliation_rules.active_uniq` `GENERATED … VIRTUAL`, non-insérables). Source de vérité des colonnes pour l'INSERT au restore, y compris pour les tables vides (NDJSON 0 ligne).
5. **Intégrité vérifiable** : recalculer le SHA-256 du NDJSON de chaque table redonne exactement `manifest.json[tables][*].sha256`.
6. **Audit-trail** : chaque export insère une entrée `audit_log` (action `admin.full_export`, `entity_type` `installation`, `details_json` snake_case : taille fichier, nb tables, nb lignes total, version source) via `NewAuditLogEntry::from_current_user` (actor JWT — AC2 interdit le PAT).
7. **Maîtrise mémoire (DC8 figé)** : assemblage **in-memory** sous un plafond `KESH_ADMIN_EXPORT_INMEM_MB` (défaut 50, **bornes [1, 2048]**, parse+borne+warn ; log WARN si configuré > 500) ; **au-delà → fichier temporaire** (`tokio::fs`) streamé via `Body::from_stream` (`tokio_util::io::ReaderStream`). Le plafond est documenté (`.env.example` + manuel admin).

### Partie B — UI admin export (`/admin/backup`)

8. Page `/admin/backup` (route `(app)/admin/backup/+page.svelte`), **visible uniquement pour rôle Admin** (gating `isAdmin` sidebar), avec un bouton « Exporter toute l'installation » qui déclenche `GET full-export` via `apiClient.getBlob` et **télécharge** le fichier (pattern `triggerDownload` de `exports.api.ts`).
9. Pendant l'export : indicateur de chargement (bouton désactivé + libellé « Export en cours… »), gestion d'erreur (encart/toast), succès → message de succès. Le nom de fichier provient de l'en-tête `Content-Disposition`, fallback `kesh-installation-{YYYY-MM-DD}.keshbackup`. *(Aligner le pattern succès — toast `svelte-sonner` ou encart inline — sur celui retenu, cf. T-B2.)*
10. Lien « Sauvegarde / Export installation » ajouté au groupe `administration` de la sidebar (`adminOnly`), i18n FR/DE/IT/EN.

### Partie C — Backend import (`POST /api/v1/admin/full-import`)

11. `POST /api/v1/admin/full-import` accepte un **upload multipart** (champ `file` = `.keshbackup`), **réservé rôle Admin strict** (non-Admin → `403`) et **interdit via PAT** (`403 API_KEY_MANAGEMENT_FORBIDDEN`). Limite de taille via `DefaultBodyLimit::max(...)` configurable `KESH_ADMIN_IMPORT_MAX_MB` (**défaut 512**, bornes `[1, 10240]`, pattern parse+borne+warn de `bank_import_max_mib`). **Succès → HTTP 200** + corps JSON camelCase `{ backupCreated: bool, tablesRestored: number, rowsRestored: number, sourceVersion: string, sessionInvalidated: true }`.
12. **Validation pré-restore (ordre, AVANT tout DELETE)** : (a) **structure ZIP** conforme (manifest.json root + `data/<table>.ndjson` + `files/` vide) → sinon `400 INVALID_BACKUP_STRUCTURE` ; `formatVersion ≤ 1` sinon `400` ; (b) **intégrité SHA-256** de chaque table re-vérifiée vs manifest → refus `400` si mismatch (tamper) ; (c) **compat colonnes bidirectionnelle** (via `INFORMATION_SCHEMA.COLUMNS`) : (c1) chaque colonne de `columnNames` source ⊆ colonnes destination → sinon `400 IMPORT_SCHEMA_MISMATCH` (`details: { table, unknownColumns }`) ; (c2) chaque colonne destination **`NOT NULL` sans `DEFAULT`** (hors générées/auto-incr) ⊆ `columnNames` source → sinon `400 IMPORT_SCHEMA_MISMATCH` (`details: { table, missingRequiredColumns }`) ; (d) **compat version** (DC4) : refus `409` si `manifest.keshVersionMinRequired > version binaire destination` (= sémantique exacte downgrade-protection 10-2). **NE PAS** refuser sur `keshVersion > dest` seul (sur-restrictif). **Nouvelle** fonction `check_import_version_compat(manifest_min_required, binary_version)` (params strings, pas de lecture DB) réutilisant `semver::Version::parse` + compare (gère les pré-releases per semver.org, ex. `0.2.0-rc1 < 0.2.0`).
13. **Backup automatique pré-import (D7/DC5)** : avant toute opération destructrice, l'état actuel est intégralement exporté via le **cœur factorisé `build_keshbackup()`** (le même que la Partie A **mais SANS émission d'audit** `admin.full_export` — l'audit reste dans le handler `full_export` seul, sinon chaque import logue un faux export juste avant de le DELETE) ; respecte le plafond DC8 → **écrit toujours sur disque** dans `KESH_ADMIN_BACKUP_DIR` (défaut `/tmp`), fichier `kesh-pre-import-{timestamp}.keshbackup`. Répertoire créé si absent (`tokio::fs::create_dir_all`) ; échec d'écriture → `500` diagnostique (jamais d'import sans backup réussi). Si le restore échoue → **rollback** depuis ce backup. Chemin **loggé serveur** (pas exposé à l'utilisateur, cf. AC20), conservé sur disque pour diagnostic (purge = opérateur). **Concurrence** : backup (lecture) + restore (écriture) ne sont pas dans la même transaction ; sérialiser l'import via un **verrou d'installation** (`_kesh_version … FOR UPDATE` ou `GET_LOCK('kesh_full_import', N)`, pattern Story 17-1) sur toute la durée backup+restore, pour empêcher des mutations concurrentes de produire un backup incohérent.
14. **Restore destructeur** : dans **une transaction DB** (`pool.begin()`), `SET FOREIGN_KEY_CHECKS = 0` (même connexion), puis **`DELETE FROM <table>`** (DML, **pas** `TRUNCATE` qui est DDL non-rollbackable) pour les tables applicatives restaurées, puis INSERT **paramétrés** (DC1 — `query(...).bind(...)`, jamais de SQL concaténé) avec la **liste de colonnes explicite du manifest** (`columnNames`, exclut les colonnes générées), `SET FOREIGN_KEY_CHECKS = 1`, audit import (AC16) **dans la même transaction**, `COMMIT`. `_sqlx_migrations`, `_kesh_version` **et `onboarding_state`** (état local, DC11) **non touchées**. AUTO_INCREMENT : non requis (IDs réinsérés explicitement). ⚠️ **`FOREIGN_KEY_CHECKS=0` ne re-valide PAS l'intégrité référentielle au COMMIT** (MariaDB n'a pas de contraintes `DEFERRABLE`) : un dump à FK incohérente commiterait silencieusement — voir AC17 (risque assumé).
15. **Pas de re-run de migrations** : ~~`MIGRATOR.run()`~~ serait un **no-op structurel** (le restore préserve `_sqlx_migrations` destination, donc aucune migration n'est rejouée). La compat schéma source↔destination est garantie **en amont** par le double check colonnes d'AC12c : (i) `columnNames` source ⊆ colonnes destination (`IMPORT_SCHEMA_MISMATCH` si colonne inconnue) **ET** (ii) toute colonne destination **`NOT NULL` sans `DEFAULT`** (ex. `users.company_id`, `migrations/20260419000002_users_company_id.sql:18`) **doit** figurer dans `columnNames[table]` source, sinon refus `400 IMPORT_SCHEMA_MISMATCH` (`details: { table, missingRequiredColumns }`) **avant tout DELETE** — sans quoi l'INSERT planterait en cours de restore.
16. **Audit-trail import** : l'entrée `audit_log` `admin.full_import` (`entity_type` `installation` ; `details_json` snake_case : `source_kesh_version`, `source_instance_id`, `triggered_by_user` = identité de l'admin destination ayant lancé l'import [login/id pré-import, informatif], nb tables/lignes restaurées) est **insérée DANS la transaction de restore, après les INSERT et avant le COMMIT**, avec un **`user_id` garanti présent dans le dataset source restauré** = `MIN(users.id) WHERE role='Admin'` (⚠️ **PAS** `CurrentUser.user_id` destination, qui peut ne pas exister dans la source → violerait la FK `audit_log.user_id → users(id)` NOT NULL, `migrations/20260413000001_audit_log.sql:21`). Ainsi la trace d'import est **atomique avec le restore** et **toujours préservée** (OLICo Art. 9), jamais écrasée. L'`audit_log` source remplace l'historique destination (comportement migration assumé).
17. **Atomicité & cohérence post-import** : le restore + l'audit étant dans **une seule transaction** (AC14/AC16), un échec **rollback tout** → destination intacte ; **jamais d'état mi-effacé**. Le backup pré-import (AC13) couvre les échecs **hors-transaction** (crash process, OOM ; au reconnect MariaDB rollback la transaction non-commitée, le backup disque sert de filet ultime/diagnostic). **Risque assumé (dette B v0.3)** : un `.keshbackup` à intégrité référentielle corrompue mais SHA-256 valide (tamper sophistiqué recalculant le manifest, ou bug d'export d'une version future) commiterait un état FK-incohérent sans erreur — accepté car un backup produit par Kesh non-altéré est FK-cohérent par construction et le SHA-256 détecte l'altération courante. Le compte admin source est fonctionnel sur la destination (login avec credentials source ; cf. AC19/20 invalidation de session).

### Partie D — UI admin import (`/admin/restore`)

18. Page `/admin/restore` (route `(app)/admin/restore/+page.svelte`), **Admin only**, avec sélecteur de fichier `.keshbackup` (input file, pattern `bank-import`, FormData via `apiClient.postFormData`).
19. **Confirmation forte** : avant l'envoi, un **modal `Dialog`** (composant `lib/components/ui/dialog/`) avertit explicitement « Cette action va **remplacer TOUTES les données** de l'installation actuelle. Une sauvegarde de l'état actuel sera créée côté serveur avant l'import. **Vous serez déconnecté** et devrez vous reconnecter avec les identifiants de l'instance importée. » + double action (confirmer/annuler). Pas d'envoi sans confirmation explicite.
20. Pendant l'import : indicateur de progression, gestion d'erreurs typées (mismatch SHA → message intégrité ; incompat version `409` → message version source/dest ; `400 formatVersion` → format inconnu ; `413` → message limite taille). **Succès** → message + **déconnexion propre** (le corps JSON `sessionInvalidated: true`, AC11, déclenche une redirection vers `/login` ; les refresh_tokens destination ayant été remplacés, la session courante est invalide). Pas d'affichage d'un chemin de backup interne (non accessible à l'utilisateur sans SSH).
21. Lien « Restaurer / Importer installation » ajouté au groupe `administration` (`adminOnly`), i18n FR/DE/IT/EN.

### Partie E — Test end-to-end (intégration Rust)

22. **Test d'intégration Rust** (`crates/kesh-api/tests/admin_backup_e2e.rs`, DC8/F16) : seed DB source (companies/users/écritures/factures) → export en mémoire → `DELETE`-all → import → **assert équivalence** (row counts par table, login admin source possible, FKs intègres, `audit_log` source présent + entrée `admin.full_import` post-restore). Couvre aussi : refus version incompat, refus SHA tamper, rollback transactionnel sur échec injecté. *(Le test Playwright double-instance Docker Compose — fidèle au use-case migration cross-instance — est reporté **v0.3** comme dette documentée : trop lourd pour le MVP, le test d'intégration Rust couvre l'équivalence fonctionnelle.)*

### Partie F — Documentation

23. `docs/api-external.md` ou manuel admin LaTeX FR (`docs/manual/fr/admin-manual.tex`) : nouvelle section « Migration et restauration via l'UI Kesh » + **matrice des méthodes** (Hyper Backup DSM Story 10-4 / `mariadb-dump` CLI Story 10-4 / export-import UI Kesh) selon le use case. PDF admin régénéré (`latexmk -xelatex`).
24. `CHANGELOG.md` : entrée `Added` pour la version cible (v0.2.0) — export/import installation via UI admin.
25. `README.md` (« Feuille de route » + « Fonctionnalités ») : refléter la livraison de l'export/import installation (retirer tout `(à venir)` associé, statut Epic 17).

### Transverses (toutes parties)

26. **Sécurité** : pas de chiffrement du `.keshbackup` par défaut (responsabilité utilisateur — doc recommande GPG/age pour transit hors infra contrôlée, cohérent Epic 17 hors-scope). Le SHA-256 sert à la **détection d'altération**, pas à la confidentialité. ⚠️ **Le fichier contient les hash de mots de passe (`users.password_hash`) et les `refresh_tokens`** → la doc (AC23) doit avertir explicitement que le `.keshbackup` est un **secret** à manipuler comme tel. L'auth par cookie (pas de token dans un header custom) ⇒ pas de surface CSRF nouvelle (les endpoints sont POST/GET protégés par `require_admin_role` + cookie SameSite ; le GET export ne mute rien).
27. **i18n ownership** : les clés frontend respectent `lint-i18n-ownership` (feature-scoped `backup-*` / `restore-*`, ou namespace global pour UI élémentaire). `npm run lint-i18n-ownership` PASS.
28. **HTTP-LAN safe** : aucune API secure-context-only en runtime (`crypto.randomUUID`/`subtle`/`navigator.clipboard` non-gardé). Utiliser `$props.id()` pour les IDs DOM et `copyToClipboard` (fallback `execCommand`) si copie. `URL.createObjectURL` (download) est sûr en HTTP (cf. `feedback_no_secure_context_apis_http_lan`).

## Tasks / Subtasks

> Tâches groupées par Partie (A–F). Au split, chaque groupe devient une sous-story (17-3a…17-3f). Le **story-zéro** naturel est la **Partie A** (elle pose le format `.keshbackup` que tout le reste consomme).

### Partie A — Backend export (story-foundation, pose le format)

- [ ] **T-A1** Définir le format `.keshbackup` (DC1) : module `crates/kesh-api/src/admin_backup/` (`format.rs`/`manifest.rs`). Conteneur ZIP, `manifest.json` (schéma AC4 : `formatVersion=1`, `instanceId`, `columnNames` par table **excluant colonnes générées**), un fichier **NDJSON** par table, dossier `files/` vide. **Créer `build_backup_manifest_json(&BackupManifest) -> Result<Vec<u8>, AppError>`** dans `admin_backup/manifest.rs` (NE PAS modifier `exports::metadata::build_metadata_json`, signature per-company incompatible). **Factoriser le cœur `build_keshbackup(pool) -> Result<Bytes/temp-file, AppError>` SANS émission d'audit** — réutilisé par le handler export (qui ajoute l'audit) ET par le backup pré-import T-C3 (qui n'en veut pas, O-4). (AC: 3, 4)
- [ ] **T-A2** Lister/sérialiser les **22 tables applicatives** (Dev Notes §Inventaire) : récupérer toutes les lignes (installation entière, pas de scope company), sérialiser en NDJSON avec **liste de colonnes explicite excluant les colonnes générées** (`reconciliation_rules.active_uniq` VIRTUAL → exclue). Réutiliser `sha2`/`sha256_hex` (`exports/metadata.rs`) pour le hash NDJSON par table. **Promouvoir ici (17-3a, story-zéro)** la liste canonique de tables : **déplacer** (move, pas copy) `TABLES_TO_TRUNCATE` de `test_fixtures.rs` vers `kesh-db/src/backup.rs` en **`pub`** (l'export en a besoin pour énumérer les tables ; 17-3c la réutilisera). `test_fixtures.rs` **réexporte** la constante canonique (source unique). Déplacer/adapter le test de synchro `truncate_all_inventory_matches_schema` (`test_fixtures.rs:711`) pour valider la constante canonique de `backup.rs`. (AC: 3, 4, 5)
- [ ] **T-A3** Handler `full_export` (`crates/kesh-api/src/routes/admin.rs`) en **GET** : assemble le `.keshbackup`, `Content-Type: application/octet-stream` + `Content-Disposition` (réutiliser `util::build_content_disposition`). (AC: 1)
- [ ] **T-A4** Mémoire (DC8) : in-memory sous `KESH_ADMIN_EXPORT_INMEM_MB` (défaut 50, `config.rs`), au-delà fichier temp `tokio::fs` + `Body::from_stream` (`tokio_util::io::ReaderStream`). (AC: 7)
- [ ] **T-A5** RBAC + anti-PAT : monter la route GET dans `admin_routes` (`lib.rs`, `route_layer(require_admin_role)`) + garde anti-PAT. **Promouvoir `ensure_not_pat` en `pub(crate)`** dans `routes/api_keys.rs` (ou extraire dans `routes/common.rs`) pour réutilisation sans duplication. (AC: 1, 2)
- [ ] **T-A6** Audit `admin.full_export` via `NewAuditLogEntry::from_current_user` + `audit_log::insert_in_tx`. (AC: 6)
- [ ] **T-A7** `AppError` variant(s) (`AdminFullExportFailed`/réutiliser) + codes i18n (`errors.rs`). Tests unit (manifest shape `formatVersion`/`columnNames`/`instanceId`, sha256 round-trip, exclusion colonne générée, RBAC, anti-PAT). (AC: 1, 2, 4, 5)

### Partie B — UI export

- [ ] **T-B1** Feature front `lib/features/admin-backup/` : `admin-backup.api.ts` (`downloadFullExport()` via `apiClient.getBlob` sur `GET /api/v1/admin/full-export` + `triggerDownload`, pattern `exports.api.ts`). (AC: 8, 9)
- [ ] **T-B2** Page `(app)/admin/backup/+page.svelte` (runes Svelte 5, bouton + état chargement + toasts). (AC: 8, 9)
- [ ] **T-B3** Lien sidebar `administration.adminOnly` + i18n `backup-*` 4 locales. (AC: 10, 27)
- [ ] **T-B4** Test unit composant + `lint-i18n-ownership` PASS. (AC: 27)

### Partie C — Backend import

- [ ] **T-C1** Handler `full_import` multipart (`routes/admin.rs`), extracteur `Multipart` (pattern `bank_imports::parse_multipart`), `DefaultBodyLimit` + `KESH_ADMIN_IMPORT_MAX_MB` (**défaut 512, bornes [1,10240]**, `config.rs`). RBAC Admin + anti-PAT. Réponse succès **200 + JSON** `{ backupCreated, tablesRestored, rowsRestored, sourceVersion, sessionInvalidated:true }`. (AC: 11)
- [ ] **T-C2** Validation **avant tout DELETE** : structure ZIP (`400 INVALID_BACKUP_STRUCTURE`), `formatVersion ≤ 1` (`400`), SHA-256 par table (`400` tamper), **check colonnes bidirectionnel** AC12c (source⊆dest + dest-NOT-NULL-sans-default⊆source → `400 IMPORT_SCHEMA_MISMATCH`), **`check_import_version_compat(min_required, binary)`** nouvelle fn `version.rs` (`409` si `min_required > binary`, gère pré-releases). (AC: 12)
- [ ] **T-C3** Backup auto pré-import (DC5) : appelle le moteur Partie A → `KESH_ADMIN_BACKUP_DIR` (défaut `/tmp`, `create_dir_all` si absent ; échec écriture → `500` diagnostique, jamais d'import sans backup). Chemin loggé serveur. Rollback depuis ce backup si restore hors-transaction échoue. (AC: 13, 17)
- [ ] **T-C4** Restore (DC6) **transactionnel** : `pool.begin()` → `SET FOREIGN_KEY_CHECKS=0` → **`DELETE FROM`** (pas TRUNCATE) les **21 tables** (exclut `onboarding_state`, DC11) → INSERT paramétrés colonnes-explicites du manifest → audit import in-tx (T-C6) → `FOREIGN_KEY_CHECKS=1` → `COMMIT`. **Consomme** `kesh_db::backup::TABLES_TO_TRUNCATE` (promu en 17-3a/T-A2) + ajoute le helper DELETE transactionnel dans `kesh-db/src/backup.rs`. Post-restore : forcer `onboarding_state` « done » si ≥1 company non-stub + admin (DC11). (AC: 14, 17)
- [ ] **T-C5** ~~`MIGRATOR.run()`~~ supprimé (no-op structurel, O-2). La compat schéma est assurée par le double check colonnes T-C2 (AC12c). (AC: 15)
- [ ] **T-C6** Audit `admin.full_import` **dans la transaction restore** (avant COMMIT), `user_id = MIN(users.id WHERE role='Admin')` source (PAS `CurrentUser` dest → FK), `details_json` snake_case (`source_kesh_version`, `source_instance_id`, `triggered_by_user`, counts). (AC: 16)
- [ ] **T-C7** Tests intégration DB : round-trip équivalence (row counts/clés/login admin) ; **ids users source ≠ admin dest** → audit import OK (pas de viol FK, O-1) ; refus `409` version, `400` tamper/format/`IMPORT_SCHEMA_MISMATCH` (colonne manquante NOT NULL) ; **rollback transactionnel** sur échec injecté ; **import backup onboarding-incomplet sur instance onboardée → pas de catch-22** (DC11/O-3). (AC: 12, 14, 16, 17)

### Partie D — UI import

- [ ] **T-D1** `admin-restore.api.ts` (`uploadFullImport(file)` via `postFormData`). (AC: 18)
- [ ] **T-D2** Page `(app)/admin/restore/+page.svelte` : input file + **modal `Dialog` de confirmation forte** (avertit remplacement total + déconnexion) + progression + erreurs typées (`409`/`400`/`413`) + **redirection `/login`** sur succès (`sessionInvalidated`). Pas d'affichage de chemin backup interne. (AC: 18, 19, 20)
- [ ] **T-D3** Lien sidebar + i18n `restore-*` 4 locales. (AC: 21, 27)
- [ ] **T-D4** Test unit composant (confirmation bloque l'envoi) + `lint-i18n-ownership` PASS. (AC: 19, 27)

### Partie E — Test end-to-end (intégration Rust)

- [ ] **T-E1** Test d'intégration Rust `crates/kesh-api/tests/admin_backup_e2e.rs` : round-trip équivalence + refus version/tamper/format + rollback (cf. AC22). *(Playwright double-instance = dette v0.3 documentée.)* (AC: 22)

### Partie F — Doc

- [ ] **T-F1** Manuel admin LaTeX FR §Migration/restauration + matrice méthodes + PDF régénéré. (AC: 23)
- [ ] **T-F2** `CHANGELOG.md` (Added) + `README.md` (roadmap + fonctionnalités). (AC: 24, 25)

## Dev Notes

### Décisions de conception (DC) — à confirmer/durcir au validate

| # | Décision | Rationale | Statut |
|---|---|---|---|
| **DC1** | Format données = **NDJSON ligne-par-ligne par table**, restore via **INSERT paramétrés** avec liste de colonnes du manifest (`columnNames`) | Fidélité de type (NULL, `rust_decimal` via serde, `NaiveDateTime`), **zéro injection** (pas de SQL concaténé), pas de dépendance `mariadb-dump` CLI (Alt-1 #112 rejetée). SQL-dump écarté (escaping manuel risqué + couplage dialecte). `columnNames` requis car NDJSON vide n'expose pas les colonnes ; **exclut les colonnes générées** (`active_uniq` VIRTUAL, non-insérable) | **figé (Pass 1)** |
| **DC2** | Scope = **22 tables applicatives, installation entière** (PAS per-company) | D6. Distinct de 9-2b (16 tables, 1 company, exclut users/audit_log). Inclut users, audit_log, refresh_tokens, onboarding_state, api_keys… | **figé** |
| **DC3** | **Aucun fichier binaire** en v0.2 (no-op), dossier `files/` réservé forward-compat | Ground-truth : Kesh ne stocke aucun upload sur disque. NE PAS inventer de file-store | **figé** |
| **DC4** | Compat version import : refus `409` **ssi `manifest.keshVersionMinRequired > version binaire dest`** (sémantique downgrade-protection 10-2). PAS de refus sur `keshVersion > dest` seul | `min_required` est le seul critère sûr (politique migration breaking). Refuser sur `keshVersion` sur-restreint sans gain. **Nouvelle** fn `check_import_version_compat(min_required, binary)` (strings, pas de lecture DB) réutilisant le compare SemVer de `version.rs` | **figé (Pass 1)** |
| **DC5** | **Backup auto pré-import** (réutilise moteur export) → `KESH_ADMIN_BACKUP_DIR` (défaut `/tmp`, créé si absent) + rollback. Jamais d'import sans backup réussi | D7. Safety net non-négociable (opération destructrice) | **figé** |
| **DC6** | Restore **dans une transaction** : `FOREIGN_KEY_CHECKS=0` → **`DELETE FROM`** (pas TRUNCATE, DDL non-rollbackable) → INSERT colonnes-explicites → `FK=1` → COMMIT | DELETE est DML ⇒ vrai rollback DB (AC17). Tolère source-plus-ancienne (colonnes ajoutées = nullable/defaulted). `_kesh_version`/`_sqlx_migrations` préservées. ⚠️ `FOREIGN_KEY_CHECKS` est **session-scoped** ⇒ tout sur **une même connexion/transaction** | **figé (Pass 1)** |
| **DC7** | Admin strict **+ anti-PAT** (`403 API_KEY_MANAGEMENT_FORBIDDEN`) | Opérations d'infra destructrices ⇒ jamais via clé API (cohérent 17-2a DC6) | **figé** |
| **DC8** | Export **in-memory sous plafond `KESH_ADMIN_EXPORT_INMEM_MB` (défaut 50)**, au-delà → fichier temp + `Body::from_stream` | 99 % des installations PME Kesh < 50 Mo ⇒ chemin simple identique 9-2b ; les grosses ne saturent pas la RAM. Plafond documenté | **figé (Pass 1)** |
| **DC9** | **Aucune migration DB** | Opère sur tables existantes. Pas d'audit idempotence, pas de bump min_required | **figé** |
| **DC10** | `audit_log` import inséré **dans la transaction restore** (avant COMMIT) avec `user_id = MIN(users.id WHERE role='Admin')` source (PAS `CurrentUser` dest, sinon viol FK) ; `refresh_tokens` restaurés ⇒ **sessions destination invalidées** (`sessionInvalidated:true` → redirection `/login`) | Trace d'import atomique + jamais écrasée (OLICo) + FK `audit_log.user_id` respectée. Invalidation session = conséquence assumée du remplacement, signalée UI | **figé (P1, durci P3 O-1)** |
| **DC11** | **`onboarding_state` exclue du restore** (conservée côté destination, comme `_kesh_version`) ; **forcée à l'état terminé post-restore** si le dataset restauré contient ≥1 company non-stub + ≥1 admin | `onboarding_state` est un **état d'installation local**, pas une donnée métier. La restaurer depuis une source non-onboardée rouvrirait le **catch-22 #120** (corrigé v011-2). L'exclure + forcer « done » évite la régression dans les deux sens | **figé (P3 O-3)** |

### Format `.keshbackup` (spec normative — figée 17-3a, consommée par tout le reste)

> Source de vérité unique du contrat export↔import. Tout écart entre 17-3a (écriture) et 17-3c (lecture) casse l'intégrité SHA-256 → préciser ici lève l'ambiguïté.

**Structure ZIP (exacte)** — l'import refuse `400 INVALID_BACKUP_STRUCTURE` si elle diffère :
```
manifest.json              # au ROOT du ZIP
data/<table>.ndjson        # 1 par table applicative (22), nom = nom exact de la table
files/                     # dossier vide en v0.2 (forward-compat) — refus 400 si non-vide
```

**Sérialisation NDJSON** (1 objet JSON par ligne, séparés par `\n` LF) :
- Encodage **UTF-8 sans BOM**, fin de ligne **LF** (`\n`), pas de ligne finale vide superflue.
- `serde_json` standard (pas d'escaping custom). **NULL → `null` JSON** (jamais omis ni `"NULL"`).
- **Ordre des clés = `columnNames` du manifest** (déterministe, identique export/import).
- Types : `Decimal` (rust_decimal) → string JSON (fidélité), `NaiveDateTime` → ISO 8601 `"YYYY-MM-DDTHH:MM:SS"`, enums (`Role`, `ApiKeyScope`…) → leur représentation `serde` existante.
- Une table vide → fichier `data/<table>.ndjson` de **0 octet**.

**`manifest.json`** (camelCase, `serde_json::to_vec_pretty`, trailing `\n`) — exemple abrégé :
```json
{
  "formatVersion": 1,
  "keshVersion": "0.1.8",
  "keshVersionMinRequired": "0.1.0",
  "instanceId": 1,
  "exportDate": "2026-06-08T12:34:56Z",
  "tables": {
    "companies": { "rowCount": 2, "sha256": "abc…", "columnNames": ["id","name","ide_number","..."] },
    "reconciliation_rules": { "rowCount": 3, "sha256": "def…", "columnNames": ["id","company_id","match_type","match_value","active","..."] }
  }
}
```
- `formatVersion`: **1** figé. Import refuse `400` si `> 1`.
- `exportDate`: ISO 8601 UTC **précision seconde** (`Utc::now().to_rfc3339_opts(SecondsFormat::Secs, true)`, cohérent `metadata.rs:78`).
- `instanceId` (`i64`) : **informatif uniquement** (traçabilité migration) = `MIN(companies.id)` au moment de l'export (toujours ≥ 1 puisqu'une installation a au moins 1 company). Repris tel quel dans le `details_json` de l'audit import (`sourceInstanceId`). Non utilisé pour remapper quoi que ce soit.
- `columnNames` : **ordonné, explicite, exclut les colonnes générées** (`reconciliation_rules.active_uniq`). C'est la liste utilisée pour `INSERT INTO t (col1,…) VALUES (?,…)` au restore.
- `sha256` : hash des **bytes NDJSON décompressés** (post-extraction ZIP, LF). Table vide → sha256 de la chaîne vide (`e3b0c4…`).

**Vérification SHA-256 à l'import** : recalculée **avant tout DELETE** (sur les NDJSON extraits) → refus `400` (tamper) sans avoir muté la DB. Réutilise `sha2`/`sha256_hex` (`exports/metadata.rs`).

**Compat colonnes (source ≠ dest)** : l'INSERT n'utilise QUE les `columnNames` du manifest. Si une colonne du manifest **n'existe pas** dans le schéma destination (source d'une version mineure plus récente avec `ADD COLUMN` non-breaking, donc non bloquée par DC4) → l'INSERT échoue. L'import DOIT le détecter en amont (comparer `columnNames` à `INFORMATION_SCHEMA.COLUMNS` de la table) et refuser `400 IMPORT_SCHEMA_MISMATCH` (`details: { table, unknownColumns: [...] }`) plutôt que de planter en cours de restore.

**Ordre DELETE/INSERT** (sous `FOREIGN_KEY_CHECKS=0`, ordre techniquement indifférent — fixé pour lisibilité + tests) : **DELETE enfants→parents** (ordre `TABLES_TO_TRUNCATE`), **INSERT parents→enfants** (inverse).

**Dépendances Cargo** (à vérifier/ajouter) : `zip` v2 + `sha2` 0.10 (déjà `kesh-api`), `semver` "1" (déjà `kesh-db`, réutilisé pour la compat), **`tokio-util` 0.7** (à AJOUTER à `kesh-api/Cargo.toml` pour `ReaderStream` du streaming DC8 — actuellement absent).

### Inventaire des 22 tables applicatives (DC2) — source : migrations + `TABLES_TO_TRUNCATE`

Ordre **truncate (enfants→parents)** — réutiliser/promouvoir la constante existante (`crates/kesh-db/src/test_fixtures.rs:297-320`, aujourd'hui `pub(crate)` côté test ; à exposer dans un module production, p.ex. `kesh-db/src/backup.rs`) :

```
invoice_lines, journal_entry_lines, invoices, invoice_number_sequences,
journal_entries, audit_log, api_keys, company_invoice_settings,
bank_transactions, bank_imports, bank_profiles, reconciliation_rules,
bank_accounts, accounts (FK self-ref parent_id), products, contacts,
fiscal_years, vat_rates, refresh_tokens, onboarding_state, users, companies
```

**Export** : les **22 tables** sont toutes exportées (l'export est un snapshot complet, `onboarding_state` incluse). **Restore** : **21 tables** restaurées (DELETE+INSERT) — **`onboarding_state` exclue du restore** (DC11, conservée côté destination + forcée « done » post-restore). **DELETE (restore)** : dans la transaction avec `FOREIGN_KEY_CHECKS=0`, l'ordre importe peu (FK désactivées) ; conserver l'ordre enfants→parents par prudence. **INSERT** : ordre inverse (parents→enfants) OU quelconque sous `FK=0`. **Jamais touchées** (ni DELETE ni INSERT) : `_sqlx_migrations`, `_kesh_version` (système), `onboarding_state` (état local DC11). ⚠️ `accounts` a une **FK self-référente** (`parent_id`) → `FOREIGN_KEY_CHECKS=0` indispensable. ⚠️ **Colonnes générées exclues de l'INSERT** : `reconciliation_rules.active_uniq` (`GENERATED ALWAYS AS … VIRTUAL`, `migrations/20260513000001_reconciliation_rules.sql:54`) — insérer dedans = erreur SQL. La liste `columnNames` du manifest (AC4) ne doit JAMAIS contenir de colonne générée. ⚠️ Tout (DELETE+INSERT+SET) sur **une seule connexion/transaction** car `FOREIGN_KEY_CHECKS` est **session-scoped**.

### Réutilisation — moteur d'export existant (Story 9-2b) — `crates/kesh-api/src/exports/`

> **Ne pas réinventer.** Le moteur 9-2b fournit les briques, mais son **scope diffère** (per-company CSV 16 tables). Réutiliser les helpers, pas le handler.

| Brique | Chemin:ligne | Réutilisation 17-3 |
|---|---|---|
| Construction ZIP | `exports/global.rs:80-99` (`build_zip`, crate `zip` v2 deflate, `Cargo.toml:50`) | Directe |
| SHA-256 | `exports/metadata.rs:89-95` (`sha256_hex`, `sha2` 0.10, `Cargo.toml:47`) | Directe |
| Manifest JSON camelCase | `exports/metadata.rs:71` (`build_metadata_json(company:&Company, locale, tables)` — signature **per-company incompatible**) | **NE PAS réutiliser**. Créer `build_backup_manifest_json(&BackupManifest)` dans `admin_backup/manifest.rs` (champs AC4, sans company/locale) |
| `Content-Disposition` RFC 5987 | `util::build_content_disposition` (`util.rs:104-170`) | Directe |
| Handler de référence | `routes/exports.rs:37-112` (`export_global`) | **Pattern** (scope ≠) |
| `AppError::GlobalExportFailed` | `errors.rs:236-237` → 500 | Modèle pour `AdminFullExportFailed` |
| Audit best-effort | `routes/exports.rs:151-176` (`emit_global_export_audit`) | Modèle (action `admin.full_export`) |

⚠️ Le format **CSV** de 9-2b (BOM, `;`, CRLF) est **inadapté au round-trip** (perte de type, re-parsing fragile). Pour l'import fidèle, préférer JSON/NDJSON (DC1).

### Réutilisation — version & DB — `crates/kesh-db/`

| Brique | Chemin:ligne | Usage 17-3 |
|---|---|---|
| `_kesh_version` schéma | `migrations/20260522000001_kesh_version.sql:31-41` (`id` TINYINT UNSIGNED singleton) | Lire `kesh_version_min_required` pour manifest + compat |
| Compat SemVer | `version.rs` (`check_downgrade_protection:182-223` lit la DB ≠ usage import) | **Écrire une nouvelle** `pub fn check_import_version_compat(manifest_min_required:&str, binary:&str) -> Result<(), VersionError>` réutilisant le **compare `semver::Version`** (params strings, pas de lecture DB). Refus `409` ssi `min_required > binary` (DC4). ⚠️ détection table absente via `.number()==1146` (PAS `.code()`) — sans objet ici |
| Version binaire | `env!("CARGO_PKG_VERSION")` (`Cargo.toml:3` = `0.1.8`), exposée `/health` `routes/health.rs:25` | Manifest source + comparaison destination |
| `MIGRATOR` | `kesh-db/src/lib.rs:22` (`sqlx::migrate!("./migrations")`), run `main.rs:137-141` | `MIGRATOR.run(&pool)` post-restore (AC15) |
| Truncate FK-safe | `test_fixtures.rs:297-371` (`TABLES_TO_TRUNCATE` `pub(crate)` + `truncate_all`, **connexion unique**) | **Promouvoir** vers `kesh-db/src/backup.rs` en **`pub`** (cross-crate ; `pub(crate)` insuffisant pour `kesh-api`). Adapter en **DELETE transactionnel** (pas TRUNCATE) |
| Config defaults | `crates/kesh-api/src/config.rs:357` (`bank_import_max_mib:10`) | `KESH_ADMIN_IMPORT_MAX_MB` **défaut 512** [1,10240] ; `KESH_ADMIN_EXPORT_INMEM_MB` défaut 50 [1,2048] ; `KESH_ADMIN_BACKUP_DIR` défaut `/tmp` (créé si absent) |
| Pool / SQL brut | `MySqlPool`, `sqlx::query(...).bind(...).execute(...)`, settings `pool.rs:28-49` | INSERT paramétrés |
| Audit idempotence | `docs/migrations-idempotence-audit.md` (31 migrations) | **Pas de modif** (DC9) |

### Réutilisation — RBAC / audit / multipart / config — `crates/kesh-api/`

| Brique | Chemin:ligne | Usage 17-3 |
|---|---|---|
| `admin_routes` sub-router | `lib.rs:101-127` (`route_layer(require_admin_role)`) | Ajouter `/api/v1/admin/full-export` + `/full-import` |
| `require_admin_role` | `middleware/rbac.rs:31` (hiérarchie `Role` `Ord`, `entities/user.rs:13-27`) | RBAC AC1/AC11 |
| `CurrentUser` + discrimination PAT | `middleware/auth.rs:37-45` (`api_key_id: Option<i64>`) | Anti-PAT. Helper **existant** `routes/api_keys.rs:91` : `fn ensure_not_pat(&CurrentUser) -> Result<(), AppError>` (si `api_key_id.is_some()` → `Err(AppError::ApiKeyManagementForbidden)`). **Privé** → le promouvoir `pub(crate)` pour appel inline au début des handlers `full_export`/`full_import` (AC2/AC11) |
| Audit | `audit.rs:18-54` (`AuditActor::from_current_user`), `audit_log::insert_in_tx`, ex. `bank_imports.rs:1476-1487` | Actions `admin.full_export`/`admin.full_import` |
| Multipart | `routes/bank_imports.rs:348-507` (`parse_multipart`, `field.bytes().await`, garde doublons), `DefaultBodyLimit::max` `lib.rs:213-215` | Upload import (AC11) |
| Config env | `config.rs` (`bank_import_max_mib:196`, `from_env:403`, pattern parse+borne+warn) | `KESH_ADMIN_IMPORT_MAX_MB`, `KESH_ADMIN_BACKUP_DIR` |
| `AppError` + i18n | `errors.rs` (`build_response`, `t(key,default):33-39`, codes 17-2a `API_KEY_MANAGEMENT_FORBIDDEN:699-707`) | Variants + codes export/import |

### Réutilisation — Frontend — `frontend/src/`

| Brique | Chemin:ligne | Usage 17-3 |
|---|---|---|
| Page settings de référence | `routes/(app)/settings/api-keys/+page.svelte` (runes `$state`/`$derived`, toasts, erreurs) | Modèle pages backup/restore |
| `apiClient` | `lib/shared/utils/api-client.ts` (`.getBlob():527`, `.postFormData():543`) | Download export / upload import |
| Download blob | `lib/features/export/exports.api.ts:98-110` (`triggerDownload`, `URL.createObjectURL` + `<a download>` + cleanup `finally`) | Export (AC9) — **HTTP-LAN safe** |
| Upload FormData | `lib/features/bank-import/bank-import.api.ts:48-73` + `BankImportUpload.svelte` (input file, drag-drop, state machine) | Import (AC18) |
| Modal `Dialog` | `lib/components/ui/dialog/` (bits-ui : `dialog.svelte`, `dialog-content/header/footer/title`) | **Confirmation forte** import (AC19) |
| i18n | `lib/shared/utils/i18n.svelte.ts` (`i18nMsg(key, fallback, args)`), `.ftl` `crates/kesh-i18n/locales/{fr,de,it,en}-CH/messages.ftl` | Clés `backup-*`/`restore-*` |
| Lint ownership | `frontend/scripts/lint-i18n-ownership.js` (`GLOBAL_NAMESPACES:16`) | `npm run lint-i18n-ownership` PASS (AC27) |
| Sidebar admin | `routes/(app)/+layout.svelte:52-97` (`navGroups` `administration.adminOnly`, `isAdmin:43`) | Liens backup/restore Admin-only |
| HTTP-LAN safe | `lib/shared/utils/clipboard.ts:17-49` (fallback `execCommand`), `$props.id()` (ex. `ContactPicker.svelte:25`) | AC28 — **ne jamais** `crypto.randomUUID`/`navigator.clipboard` non-gardé |

> ⚠️ La page `settings/+page.svelte` importe `i18nMsg` depuis une feature (`features/onboarding`) — **anti-pattern**. Les nouvelles pages importent depuis `$lib/shared/utils/i18n.svelte` directement.

### Test E2E double-instance (AC22) — point d'attention / candidat sous-story

**Décision Pass 1 (F16) : Option (a) retenue pour le MVP.** Test d'**intégration Rust** (`crates/kesh-api/tests/admin_backup_e2e.rs`) : seed DB A → export en mémoire → `DELETE`-all → import → assert équivalence (row counts, login admin, FKs, audit_log source + entrée import post-restore). Pas de double Docker. **Option (b)** Playwright double-instance (`docker-compose` 2 stacks) = **dette v0.3 documentée** (fidèle au use-case migration cross-instance mais trop lourd pour le MVP ; à tracer en issue `v0.2-milestone`/`v0.3` au moment de la livraison).

### Standards projet (rappels CLAUDE.md)

- **Test Locally First** avant tout push : backend (`cargo fmt --all --check` + `build --workspace --all-targets` + `clippy -D warnings` + `test --workspace`) ; frontend (`npm run check` + `lint-i18n-ownership` + `test:unit` + `build`). E2E si routes/pages touchées (`PLAYWRIGHT_HOST_PLATFORM_OVERRIDE=ubuntu24.04-x64`).
- **Branche d'abord** : `git checkout main && git pull --ff-only && git checkout -b story/17-3-…` (hook pre-commit refuse `main`).
- **Commit par étape BMAD** (spec, chaque passe validate, dev, chaque passe code-review). Pas de push auto.
- **Doc dans le même commit** que le code qui la motive (Partie F).
- **Migration breaking policy** : N/A (DC9, aucune migration).
- **Pattern batch `FailedProposal`** : **NON applicable** (O-8). L'import est une opération **atomique tout-ou-rien** (une transaction unique DELETE+INSERT), pas un endpoint batch `{accepted, failed}` per-proposal. Les erreurs (`400`/`409`/`413`/`500`) sont des `AppError` globales légitimes invalidant toute la requête en amont (exactement les exceptions globales autorisées par le §Pattern batch de CLAUDE.md). Pas de `FailedProposal`.
- **Invariant `instanceId`** (O-7) : `MIN(companies.id)` est toujours défini car l'endpoint export exige un Admin authentifié, et `users.company_id` est `NOT NULL` (`migrations/20260419000002_users_company_id.sql:18`) ⇒ tout user (donc tout admin) a une company ⇒ ≥ 1 company existe. `instanceId: i64` non-nullable garanti.

### Project Structure Notes

- **Backend** : nouveau module `crates/kesh-api/src/admin_backup/` (format/manifest/export/import) + `crates/kesh-api/src/routes/admin.rs` (handlers). Promotion d'un helper `kesh-db/src/backup.rs` (truncate + ordre tables) hors `test_fixtures`. Aligné avec la séparation existante `exports/` (logique) vs `routes/exports.rs` (handler).
- **Frontend** : `lib/features/admin-backup/` + `lib/features/admin-restore/` ; routes `(app)/admin/backup/` + `(app)/admin/restore/`. **Premier usage du préfixe `/admin/` côté front** — cohérent avec le gating `isAdmin` existant.
- **Router** : premier namespace `/api/v1/admin/*` (aujourd'hui les routes admin sont disséminées : `/api/v1/users`, `/api/v1/company/invoice-settings`). Monter dans `admin_routes` existant (RBAC déjà câblé). **L1 (limitation documentée)** : les routes admin existantes ne sont **pas** renommées sous `/api/v1/admin/` en v0.2 (asymétrie API assumée ; cohérence complète reportée v0.3, hors-scope).
- **Aucun conflit** détecté avec 9-2b (chemins, format et scope distincts).

### References

- [Source: _bmad-output/planning-artifacts/epic-17.md#Story 17-3] — périmètre, D6/D7
- [Source: github #112] — ACs origine, sous-stories A–F, alternatives
- [Source: crates/kesh-api/src/exports/global.rs, metadata.rs, routes/exports.rs] — moteur export 9-2b réutilisable
- [Source: crates/kesh-db/src/version.rs, migrations/20260522000001_kesh_version.sql] — compat version 10-2
- [Source: crates/kesh-db/src/test_fixtures.rs:297-371] — `TABLES_TO_TRUNCATE` + `truncate_all` (à promouvoir)
- [Source: crates/kesh-api/src/lib.rs:101-127, middleware/rbac.rs, middleware/auth.rs] — RBAC admin + discrimination PAT
- [Source: crates/kesh-api/src/routes/bank_imports.rs:348-507] — multipart upload
- [Source: frontend/src/lib/features/export/exports.api.ts, bank-import/, components/ui/dialog/] — patterns front download/upload/modal
- [Source: CLAUDE.md] — Test Locally First, Migration breaking policy (N/A), commit/branch, i18n ownership, HTTP-LAN safe

## Dev Agent Record

### Agent Model Used

_(à compléter par dev-story)_

### Debug Log References

### Completion Notes List

### File List

### Change Log

| Date | Étape | Modèle | Résumé |
|------|-------|--------|--------|
| 2026-06-08 | create-story (umbrella) | Opus 4.8 | Spec parente 17-3 créée. Option A (spec umbrella → split au validate, choix Guy). 4 agents Explore (export 9-2b, version/DB 10-2, RBAC/audit/multipart, frontend). 28 ACs groupés Parties A–F. 9 DC (DC2/3/4/5/6/7/9 figés, DC1/DC8 à trancher validate). Découverte clé : **aucun fichier binaire stocké** (volet binaires #112 = no-op v0.2). Aucune migration (DC9). Split pressenti 17-3a..f, story-zéro = Partie A (format `.keshbackup`). |
| 2026-06-08 | validate Pass 1 | Sonnet 4.6 | 21 findings (4C+6H+7M+4L), tous ground-truthés. Patches : **F1** export→`GET` (getBlob GET-only) ; **F2** `columnNames` au manifest + exclusion colonnes générées (`active_uniq` VIRTUAL) ; **F3** restore `DELETE` transactionnel (pas TRUNCATE DDL non-rollbackable) ; **F4** `ensure_not_pat`→`pub(crate)` ; **F6** compat version = `min_required>dest` seul + nouvelle fn ; **F7/DC10** session invalidée (refresh_tokens remplacés) → `sessionInvalidated`+redirect login ; **F8** chemin backup non exposé UI ; **F9** `KESH_ADMIN_IMPORT_MAX_MB` défaut 512 ; **F10** `formatVersion=1` figé ; **F11/F16** audit import inséré APRÈS restore ; **F12** `pub` cross-crate ; **F13** Content-Type octet-stream ; **F14** réponse import 200+JSON ; **F15** `instanceId` manifest ; **F16** E2E = intégration Rust (Playwright v0.3 dette) ; **F20** namespace `/admin/` L1 doc ; **F21** `build_backup_manifest_json` nouvelle fn. **Split A–F acté**, DC1 (NDJSON) + DC8 (in-mem<50Mo sinon stream) + DC4 figés → DC10 ajouté. Prochaine : Pass 2 Haiku 4.5. |
| 2026-06-08 | validate Pass 2 | Haiku 4.5 | 16 findings (4C+5H+6M+1L, sévérités sur-cotées = sous-spécifications, pas de hallucination « patch absent »). Ajout d'une **section normative « Format `.keshbackup` »** résolvant H2-1/2/8/9/13/15/16 : structure ZIP exacte (`manifest.json` root + `data/<table>.ndjson` + `files/` vide), sérialisation NDJSON (UTF-8 LF, NULL→`null`, ordre = `columnNames`, Decimal→string), exemple `manifest.json`, `instanceId`=`MIN(companies.id)` informatif, SHA vérifié avant DELETE, compat colonnes ⊆ dest → `400 IMPORT_SCHEMA_MISMATCH`, deps Cargo (`tokio-util` à AJOUTER, `semver`/`zip`/`sha2` présents ground-truthés). Patches AC7 (bornes export [1,2048]+warn), AC12 (codes `INVALID_BACKUP_STRUCTURE`/`IMPORT_SCHEMA_MISMATCH` + pré-releases semver), AC13 (backup respecte DC8 disque), signature `ensure_not_pat` ground-truthée. **H2-14 dismiss** (octet-stream déjà tranché P1, `application/zip` 9-2b aussi valide). Prochaine : Pass 3 Opus 4.8. |
| 2026-06-08 | validate Pass 3 | Opus 4.8 | **Catch-architectural** : 10 findings (2C+3H+3M+2L), tous ground-truthés (3 fichiers), 0 recoupement P1/P2. **O-1 CRITICAL** audit import post-restore violait FK `audit_log.user_id→users` (users remplacés par la source ⇒ `CurrentUser` dest absent) → réécrit AC16 : audit **dans la transaction**, `user_id=MIN(users.id WHERE Admin)` source. **O-2 CRITICAL** `MIGRATOR.run()` = no-op structurel (`_sqlx_migrations` préservé) → AC15 supprimé + check colonnes **bidirectionnel** (dest NOT-NULL-sans-default ⊆ source, sinon `400`). **O-3 HIGH** restore `onboarding_state`/`is_stub` rouvrait catch-22 #120 → DC11 (exclue du restore + forcée « done »). **O-4 HIGH** backup pré-import = audit parasite + non-isolation → cœur `build_keshbackup()` sans audit + verrou installation (pattern 17-1). **O-5 HIGH** `FK_CHECKS=0` ne re-valide pas au COMMIT → risque assumé documenté (dette B v0.3, SHA couvre tamper). **O-6/O-10 MEDIUM** promotion `TABLES_TO_TRUNCATE` déplacée en 17-3a (export l'utilise, dépendance cachée du split) move-not-copy + test synchro. **O-7/8/9 LOW** invariant instanceId, FailedProposal N/A, préfixe crate. Prochaine : Pass 4 Sonnet 4.6. |
