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

## Acceptance Criteria

> ACs groupés par **Partie A–F** (= frontières de split pressenties). Numérotation continue pour traçabilité.

### Partie A — Backend export (`POST /api/v1/admin/full-export`)

1. `POST /api/v1/admin/full-export` retourne un fichier `.keshbackup` (conteneur ZIP) téléchargeable, **réservé rôle Admin strict** (test : `Comptable` et `Consultation` → `403`).
2. L'endpoint est **inaccessible via clé PAT** : une requête authentifiée par `Authorization: Bearer kesh_pat_…` (même scope `read-write`) → `403` code `API_KEY_MANAGEMENT_FORBIDDEN` (cohérent Story 17-2a DC6 — opérations d'infra interdites aux PAT).
3. Le `.keshbackup` contient : (a) un fichier de données **par table applicative Kesh** (les **22 tables**, voir Dev Notes §Inventaire), (b) un `manifest.json` (métadonnées + intégrité), (c) un dossier `files/` (vide en v0.2, forward-compat). Tables **système exclues** : `_sqlx_migrations`, `_kesh_version`.
4. `manifest.json` contient au minimum : `keshVersion` (= `env!("CARGO_PKG_VERSION")` de la source), `keshVersionMinRequired` (lu de `_kesh_version`), `exportDate` (ISO 8601 UTC `…Z`), `formatVersion` (entier, ex. `1`), et par table : `rowCount` + `sha256` (hash des bytes de données décompressés de la table). Schéma JSON camelCase, pretty-printed (cohérent `crates/kesh-api/src/exports/metadata.rs`).
5. **Intégrité vérifiable** : recalculer le SHA-256 des données de chaque table doit redonner exactement la valeur stockée dans `manifest.json[tables][*].sha256`.
6. **Audit-trail** : chaque export insère une entrée `audit_log` (action `admin.full_export`, `entity_type` `installation`, `details_json` snake_case : taille fichier, nb tables, nb lignes total, version source). Discrimine l'actor JWT vs PAT via `NewAuditLogEntry::from_current_user` — mais ici toujours JWT (AC2 interdit le PAT).
7. **Maîtrise mémoire (streaming)** : l'export d'une installation volumineuse ne sature pas la RAM serveur. *(Décision d'implémentation DC8 — voir Dev Notes ; soit streaming depuis fichier temporaire, soit in-memory avec plafond documenté ; à trancher au validate.)*

### Partie B — UI admin export (`/admin/backup`)

8. Page `/admin/backup` (route `(app)/admin/backup/+page.svelte`), **visible uniquement pour rôle Admin** (gating `isAdmin` sidebar), avec un bouton « Exporter toute l'installation » qui déclenche `full-export` et **télécharge** le fichier (pattern blob → `<a download>` de `exports.api.ts`).
9. Pendant l'export : indicateur de chargement (bouton désactivé + libellé « Export en cours… »), gestion d'erreur (toast + encart), succès → toast. Le nom de fichier provient de l'en-tête `Content-Disposition` (RFC 5987/6266), fallback `kesh-installation-{YYYY-MM-DD}.keshbackup`.
10. Lien « Sauvegarde / Export installation » ajouté au groupe `administration` de la sidebar (`adminOnly`), i18n FR/DE/IT/EN.

### Partie C — Backend import (`POST /api/v1/admin/full-import`)

11. `POST /api/v1/admin/full-import` accepte un **upload multipart** (champ `file` = `.keshbackup`), **réservé rôle Admin strict** (non-Admin → `403`) et **interdit via PAT** (`403 API_KEY_MANAGEMENT_FORBIDDEN`). Limite de taille via `DefaultBodyLimit::max(...)` configurable (`KESH_ADMIN_IMPORT_MAX_MB`, pattern `bank_import_max_mib`).
12. **Validation pré-restore (ordre)** : (a) format/ZIP valide + `manifest.json` présent ; (b) **intégrité SHA-256** de chaque table re-vérifiée vs manifest → refus `400` si mismatch (tamper) ; (c) **compat version** (DC4) : refus `409`/`422` si `manifest.keshVersion` > version du binaire destination (impossible d'importer des données plus récentes dans un binaire plus ancien), ou si `manifest.keshVersionMinRequired` > version destination. Réutilise la logique SemVer de `crates/kesh-db/src/version.rs`.
13. **Backup automatique pré-import (D7/DC5)** : avant toute opération destructrice, l'état actuel est intégralement exporté (réutilise le moteur de la Partie A) vers un emplacement temporaire (`KESH_ADMIN_BACKUP_DIR` ou `/tmp`, fichier `kesh-pre-import-{timestamp}.keshbackup`). Si le restore échoue en cours de route → **rollback** depuis ce backup. Le chemin du backup est loggé + retourné dans la réponse.
14. **Restore destructeur** : truncate + re-insert de **toutes les tables applicatives** sous `SET FOREIGN_KEY_CHECKS = 0` (connexion unique), ordre enfants→parents pour le truncate (réutilise/promeut `TABLES_TO_TRUNCATE`), INSERT **paramétrés** (DC1 — pas de SQL brut concaténé) avec **liste de colonnes explicite issue du manifest** (compat source-plus-ancienne : colonnes ajoutées depuis = nullable/defaulted), réactivation `FOREIGN_KEY_CHECKS = 1`. `_sqlx_migrations` et `_kesh_version` **non touchées**.
15. **Re-run migrations** : `kesh_db::MIGRATOR.run(&pool)` est appelé après le restore (idempotent via `_sqlx_migrations`) pour aligner le schéma si la source était plus ancienne.
16. **Audit-trail import** : entrée `audit_log` (action `admin.full_import`, `entity_type` `installation`, `details_json` : version source, nb tables/lignes restaurées, chemin backup pré-import, succès/échec). L'audit de l'import est **lui-même restauré depuis le backup** (l'`audit_log` source écrase l'actuel) — documenter ce comportement (OLICo Art. 9 : l'historique source est préservé tel quel).
17. **Atomicité & cohérence post-import** : en cas d'échec partiel, l'installation est laissée soit dans l'état source complet, soit restaurée depuis le backup pré-import — **jamais** dans un état mi-truncate. Le compte admin source est fonctionnel sur la destination (login possible).

### Partie D — UI admin import (`/admin/restore`)

18. Page `/admin/restore` (route `(app)/admin/restore/+page.svelte`), **Admin only**, avec sélecteur de fichier `.keshbackup` (input file, pattern `bank-import`, FormData via `apiClient.postFormData`).
19. **Confirmation forte** : avant l'envoi, un **modal `Dialog`** (composant `lib/components/ui/dialog/`) avertit explicitement « Cette action va **remplacer TOUTES les données** de l'installation actuelle. Une sauvegarde automatique de l'état actuel sera créée avant l'import. » + double action (confirmer/annuler). Pas d'envoi sans confirmation explicite.
20. Pendant l'import : indicateur de progression (chargement), gestion d'erreurs typées (mismatch SHA → message intégrité ; incompat version → message version source/dest ; `413` taille → message limite), succès → toast + rappel du chemin du backup pré-import.
21. Lien « Restaurer / Importer installation » ajouté au groupe `administration` (`adminOnly`), i18n FR/DE/IT/EN.

### Partie E — Test E2E end-to-end (double instance)

22. Test E2E **« export A → import B → équivalence fonctionnelle »** : sur une instance source seedée (companies/users/écritures/factures), exporter ; importer le `.keshbackup` sur une instance destination distincte (DB vierge) ; vérifier l'équivalence (login admin source disponible sur destination avec même mot de passe, companies/users/écritures/factures intactes, `audit_log` préservé). *(Modalité d'orchestration double-instance à trancher au validate — voir Dev Notes §Test E2E ; candidat fort à devenir sa propre sous-story 17-3e.)*

### Partie F — Documentation

23. `docs/api-external.md` ou manuel admin LaTeX FR (`docs/manual/fr/admin-manual.tex`) : nouvelle section « Migration et restauration via l'UI Kesh » + **matrice des méthodes** (Hyper Backup DSM Story 10-4 / `mariadb-dump` CLI Story 10-4 / export-import UI Kesh) selon le use case. PDF admin régénéré (`latexmk -xelatex`).
24. `CHANGELOG.md` : entrée `Added` pour la version cible (v0.2.0) — export/import installation via UI admin.
25. `README.md` (« Feuille de route » + « Fonctionnalités ») : refléter la livraison de l'export/import installation (retirer tout `(à venir)` associé, statut Epic 17).

### Transverses (toutes parties)

26. **Sécurité** : pas de chiffrement du `.keshbackup` par défaut (responsabilité utilisateur — doc recommande GPG/age pour transit hors infra contrôlée, cohérent Epic 17 hors-scope). Le SHA-256 sert à la **détection d'altération**, pas à la confidentialité.
27. **i18n ownership** : les clés frontend respectent `lint-i18n-ownership` (feature-scoped `backup-*` / `restore-*`, ou namespace global pour UI élémentaire). `npm run lint-i18n-ownership` PASS.
28. **HTTP-LAN safe** : aucune API secure-context-only en runtime (`crypto.randomUUID`/`subtle`/`navigator.clipboard` non-gardé). Utiliser `$props.id()` pour les IDs DOM et `copyToClipboard` (fallback `execCommand`) si copie. `URL.createObjectURL` (download) est sûr en HTTP (cf. `feedback_no_secure_context_apis_http_lan`).

## Tasks / Subtasks

> Tâches groupées par Partie (A–F). Au split, chaque groupe devient une sous-story (17-3a…17-3f). Le **story-zéro** naturel est la **Partie A** (elle pose le format `.keshbackup` que tout le reste consomme).

### Partie A — Backend export (story-foundation, pose le format)

- [ ] **T-A1** Définir le format `.keshbackup` (DC1) : module `crates/kesh-api/src/admin_backup/` (format.rs/manifest.rs). Conteneur ZIP, `manifest.json` (schéma AC4, camelCase), un fichier de données par table (DC1 : **JSON/NDJSON ligne-par-ligne** recommandé pour fidélité de type + restore paramétré sans injection — alternative SQL-dump documentée), dossier `files/` vide. (AC: 3, 4)
- [ ] **T-A2** Lister/sérialiser les **22 tables applicatives** (Dev Notes §Inventaire) : récupérer toutes les lignes (pas de scope company — installation entière), sérialiser chaque ligne avec ses colonnes (liste de colonnes explicite). Réutiliser `sha2` (`sha256_hex`, `exports/metadata.rs`) pour le hash par table. (AC: 3, 5)
- [ ] **T-A3** Handler `full_export` (`crates/kesh-api/src/routes/admin.rs`) : assemble le `.keshbackup`, en-têtes `Content-Type` + `Content-Disposition` (réutiliser `util::build_content_disposition`). (AC: 1, 9)
- [ ] **T-A4** Streaming/mémoire (DC8) : implémenter selon décision validate (fichier temp + `Body::from_stream`, ou in-memory + plafond doc). (AC: 7)
- [ ] **T-A5** RBAC + anti-PAT : monter la route dans `admin_routes` (`lib.rs`, `route_layer(require_admin_role)`) + garde `ensure_not_pat`/`api_key_id.is_some()` → `403 API_KEY_MANAGEMENT_FORBIDDEN`. (AC: 1, 2)
- [ ] **T-A6** Audit `admin.full_export` via `NewAuditLogEntry::from_current_user` + `audit_log::insert_in_tx`. (AC: 6)
- [ ] **T-A7** `AppError` variant(s) (`AdminFullExportFailed`/réutiliser) + codes i18n (`errors.rs`). Tests unit (manifest shape, sha256 round-trip, RBAC, anti-PAT). (AC: 1, 2, 5)

### Partie B — UI export

- [ ] **T-B1** Feature front `lib/features/admin-backup/` : `admin-backup.api.ts` (`downloadFullExport()` via `apiClient.getBlob` + `triggerDownload`, pattern `exports.api.ts`). (AC: 8, 9)
- [ ] **T-B2** Page `(app)/admin/backup/+page.svelte` (runes Svelte 5, bouton + état chargement + toasts). (AC: 8, 9)
- [ ] **T-B3** Lien sidebar `administration.adminOnly` + i18n `backup-*` 4 locales. (AC: 10, 27)
- [ ] **T-B4** Test unit composant + `lint-i18n-ownership` PASS. (AC: 27)

### Partie C — Backend import

- [ ] **T-C1** Handler `full_import` multipart (`routes/admin.rs`), extracteur `Multipart` (pattern `bank_imports::parse_multipart`), `DefaultBodyLimit` + `KESH_ADMIN_IMPORT_MAX_MB` (`config.rs`). RBAC Admin + anti-PAT. (AC: 11)
- [ ] **T-C2** Validation : ZIP/manifest, SHA-256 par table, compat version SemVer (réutilise `version.rs`). Mapping erreurs (`400` tamper, `409`/`422` version, `413` taille). (AC: 12)
- [ ] **T-C3** Backup auto pré-import (DC5) : appelle le moteur Partie A → `KESH_ADMIN_BACKUP_DIR`/`/tmp`. Rollback depuis ce backup si restore échoue. (AC: 13, 17)
- [ ] **T-C4** Restore (DC6) : `FOREIGN_KEY_CHECKS=0`, truncate enfants→parents (promouvoir `TABLES_TO_TRUNCATE` hors test_fixtures vers module production partagé), INSERT paramétrés colonnes-explicites, `FOREIGN_KEY_CHECKS=1`. (AC: 14, 17)
- [ ] **T-C5** `MIGRATOR.run(&pool)` post-restore (idempotent). (AC: 15)
- [ ] **T-C6** Audit `admin.full_import` (note : audit_log restauré depuis la source — documenter). (AC: 16)
- [ ] **T-C7** Tests intégration DB : round-trip (export → truncate → import → équivalence row counts/clés), refus version incompat, refus SHA tamper, rollback sur échec injecté. (AC: 12, 13, 14, 17)

### Partie D — UI import

- [ ] **T-D1** `admin-restore.api.ts` (`uploadFullImport(file)` via `postFormData`). (AC: 18)
- [ ] **T-D2** Page `(app)/admin/restore/+page.svelte` : input file + **modal `Dialog` de confirmation forte** + progression + erreurs typées + rappel chemin backup. (AC: 18, 19, 20)
- [ ] **T-D3** Lien sidebar + i18n `restore-*` 4 locales. (AC: 21, 27)
- [ ] **T-D4** Test unit composant (confirmation bloque l'envoi) + `lint-i18n-ownership` PASS. (AC: 19, 27)

### Partie E — E2E

- [ ] **T-E1** Test E2E double-instance « export A → import B → équivalence » (`frontend/tests/e2e/admin-backup-restore.spec.ts` ou test d'intégration Rust selon décision validate). (AC: 22)

### Partie F — Doc

- [ ] **T-F1** Manuel admin LaTeX FR §Migration/restauration + matrice méthodes + PDF régénéré. (AC: 23)
- [ ] **T-F2** `CHANGELOG.md` (Added) + `README.md` (roadmap + fonctionnalités). (AC: 24, 25)

## Dev Notes

### Décisions de conception (DC) — à confirmer/durcir au validate

| # | Décision | Rationale | Statut |
|---|---|---|---|
| **DC1** | Format données = **JSON/NDJSON ligne-par-ligne par table**, restore via **INSERT paramétrés** | Fidélité de type (NULL vs vide, Decimal, DATETIME), **zéro risque d'injection** (pas de SQL concaténé), pas de dépendance `mariadb-dump` CLI (Alt-1 #112 rejetée). SQL-dump considéré mais escaping manuel risqué + couplage dialecte | **proposé** |
| **DC2** | Scope = **22 tables applicatives, installation entière** (PAS per-company) | D6. Distinct de 9-2b (16 tables, 1 company, exclut users/audit_log). Inclut users, audit_log, refresh_tokens, onboarding_state, api_keys… | **figé** |
| **DC3** | **Aucun fichier binaire** en v0.2 (no-op), dossier `files/` réservé forward-compat | Ground-truth : Kesh ne stocke aucun upload sur disque. NE PAS inventer de file-store | **figé** |
| **DC4** | Compat version à l'import via **SemVer** (`version.rs`) : refus si source > destination | Réutilise downgrade-protection 10-2. Empêche d'importer des données d'un Kesh plus récent dans un binaire plus ancien (schéma inconnu) | **figé** |
| **DC5** | **Backup auto pré-import** (réutilise moteur export) + rollback | D7. Safety net non-négociable (opération destructrice) | **figé** |
| **DC6** | Restore sous `FOREIGN_KEY_CHECKS=0`, truncate enfants→parents, INSERT colonnes-explicites du manifest | Tolère source-plus-ancienne (nouvelles colonnes nullable/defaulted). `_kesh_version`/`_sqlx_migrations` préservées | **figé** |
| **DC7** | Admin strict **+ anti-PAT** (`403 API_KEY_MANAGEMENT_FORBIDDEN`) | Opérations d'infra destructrices ⇒ jamais via clé API (cohérent 17-2a DC6) | **figé** |
| **DC8** | **Streaming** export/import : temp-file + `Body::from_stream` **vs** in-memory + plafond doc | 9-2b est in-memory (`Body::from(Vec<u8>)`, supposé < 5 Mo). Installation complète peut dépasser. Trancher au validate selon complexité acceptable | **à trancher** |
| **DC9** | **Aucune migration DB** | Opère sur tables existantes. Pas d'audit idempotence, pas de bump min_required | **figé** |

### Inventaire des 22 tables applicatives (DC2) — source : migrations + `TABLES_TO_TRUNCATE`

Ordre **truncate (enfants→parents)** — réutiliser/promouvoir la constante existante (`crates/kesh-db/src/test_fixtures.rs:297-320`, aujourd'hui `pub(crate)` côté test ; à exposer dans un module production, p.ex. `kesh-db/src/backup.rs`) :

```
invoice_lines, journal_entry_lines, invoices, invoice_number_sequences,
journal_entries, audit_log, api_keys, company_invoice_settings,
bank_transactions, bank_imports, bank_profiles, reconciliation_rules,
bank_accounts, accounts (FK self-ref parent_id), products, contacts,
fiscal_years, vat_rates, refresh_tokens, onboarding_state, users, companies
```

**INSERT (restore)** = ordre **inverse** (parents→enfants), OU INSERT dans n'importe quel ordre avec `FOREIGN_KEY_CHECKS=0` (plus simple). **Exclues** : `_sqlx_migrations`, `_kesh_version` (système, jamais touchées). ⚠️ `accounts` a une **FK self-référente** (`parent_id`) → `FOREIGN_KEY_CHECKS=0` indispensable pendant le restore.

### Réutilisation — moteur d'export existant (Story 9-2b) — `crates/kesh-api/src/exports/`

> **Ne pas réinventer.** Le moteur 9-2b fournit les briques, mais son **scope diffère** (per-company CSV 16 tables). Réutiliser les helpers, pas le handler.

| Brique | Chemin:ligne | Réutilisation 17-3 |
|---|---|---|
| Construction ZIP | `exports/global.rs:80-99` (`build_zip`, crate `zip` v2 deflate, `Cargo.toml:50`) | Directe |
| SHA-256 | `exports/metadata.rs:89-95` (`sha256_hex`, `sha2` 0.10, `Cargo.toml:47`) | Directe |
| Manifest JSON camelCase | `exports/metadata.rs:39-95` (`build_metadata_json`, `serde_json::to_vec_pretty`) | Adapter (champs AC4) |
| `Content-Disposition` RFC 5987 | `util::build_content_disposition` (`util.rs:104-170`) | Directe |
| Handler de référence | `routes/exports.rs:37-112` (`export_global`) | **Pattern** (scope ≠) |
| `AppError::GlobalExportFailed` | `errors.rs:236-237` → 500 | Modèle pour `AdminFullExportFailed` |
| Audit best-effort | `routes/exports.rs:151-176` (`emit_global_export_audit`) | Modèle (action `admin.full_export`) |

⚠️ Le format **CSV** de 9-2b (BOM, `;`, CRLF) est **inadapté au round-trip** (perte de type, re-parsing fragile). Pour l'import fidèle, préférer JSON/NDJSON (DC1).

### Réutilisation — version & DB — `crates/kesh-db/`

| Brique | Chemin:ligne | Usage 17-3 |
|---|---|---|
| `_kesh_version` schéma | `migrations/20260522000001_kesh_version.sql:31-41` (`id` TINYINT UNSIGNED singleton) | Lire `kesh_version_min_required` pour manifest + compat |
| Compat SemVer | `version.rs` (`check_downgrade_protection:182-223`, `VersionError::DowngradeRefused`) | **Réutiliser la comparaison** `semver::Version` pour refus import (AC12). ⚠️ détection table absente via `.number()==1146` (PAS `.code()`) |
| Version binaire | `env!("CARGO_PKG_VERSION")` (`Cargo.toml:3` = `0.1.8`), exposée `/health` `routes/health.rs:25` | Manifest source + comparaison destination |
| `MIGRATOR` | `kesh-db/src/lib.rs:22` (`sqlx::migrate!("./migrations")`), run `main.rs:137-141` | `MIGRATOR.run(&pool)` post-restore (AC15) |
| Truncate FK-safe | `test_fixtures.rs:337-371` (`truncate_all`, `SET FOREIGN_KEY_CHECKS=0/1`, **connexion unique**) | **Promouvoir** en module production |
| Pool / SQL brut | `MySqlPool`, `sqlx::query(...).bind(...).execute(...)`, settings `pool.rs:28-49` | INSERT paramétrés |
| Audit idempotence | `docs/migrations-idempotence-audit.md` (31 migrations) | **Pas de modif** (DC9) |

### Réutilisation — RBAC / audit / multipart / config — `crates/kesh-api/`

| Brique | Chemin:ligne | Usage 17-3 |
|---|---|---|
| `admin_routes` sub-router | `lib.rs:101-127` (`route_layer(require_admin_role)`) | Ajouter `/api/v1/admin/full-export` + `/full-import` |
| `require_admin_role` | `middleware/rbac.rs:31` (hiérarchie `Role` `Ord`, `entities/user.rs:13-27`) | RBAC AC1/AC11 |
| `CurrentUser` + discrimination PAT | `middleware/auth.rs:37-45` (`api_key_id: Option<i64>`) | Anti-PAT : `api_key_id.is_some()` → 403 (AC2/AC11). Vérifier helper `ensure_not_pat` existant (17-2a) |
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

L'infra E2E actuelle (Playwright, `frontend/tests/e2e/`, `playwright.config.ts`) lance **une seule** stack `kesh-api` seedée. Le test « export A → import B » exige **deux états** distincts. Options à trancher au validate :
- **(a)** Test d'**intégration Rust** (`crates/kesh-api/tests/admin_backup_e2e.rs`) : seed DB A → export en mémoire → `truncate_all` → import → assert équivalence (row counts, login admin, FKs). Plus simple, pas de double Docker. **Recommandé pour le MVP.**
- **(b)** Test Playwright **double-instance** via `docker-compose` (2 services api/db). Plus fidèle au use-case migration mais lourd (orchestration, ports, seeds). **Candidat sous-story dédiée 17-3e** si retenu.

### Standards projet (rappels CLAUDE.md)

- **Test Locally First** avant tout push : backend (`cargo fmt --all --check` + `build --workspace --all-targets` + `clippy -D warnings` + `test --workspace`) ; frontend (`npm run check` + `lint-i18n-ownership` + `test:unit` + `build`). E2E si routes/pages touchées (`PLAYWRIGHT_HOST_PLATFORM_OVERRIDE=ubuntu24.04-x64`).
- **Branche d'abord** : `git checkout main && git pull --ff-only && git checkout -b story/17-3-…` (hook pre-commit refuse `main`).
- **Commit par étape BMAD** (spec, chaque passe validate, dev, chaque passe code-review). Pas de push auto.
- **Doc dans le même commit** que le code qui la motive (Partie F).
- **Migration breaking policy** : N/A (DC9, aucune migration).

### Project Structure Notes

- **Backend** : nouveau module `crates/kesh-api/src/admin_backup/` (format/manifest/export/import) + `crates/kesh-api/src/routes/admin.rs` (handlers). Promotion d'un helper `kesh-db/src/backup.rs` (truncate + ordre tables) hors `test_fixtures`. Aligné avec la séparation existante `exports/` (logique) vs `routes/exports.rs` (handler).
- **Frontend** : `lib/features/admin-backup/` + `lib/features/admin-restore/` ; routes `(app)/admin/backup/` + `(app)/admin/restore/`. **Premier usage du préfixe `/admin/` côté front** — cohérent avec le gating `isAdmin` existant.
- **Router** : premier namespace `/api/v1/admin/*` (aujourd'hui les routes admin sont disséminées sous chemins métier). Monter dans `admin_routes` existant (RBAC déjà câblé).
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
