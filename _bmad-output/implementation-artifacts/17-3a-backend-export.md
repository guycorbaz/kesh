# Story 17.3a: Backend export complet d'installation (`GET /api/v1/admin/full-export`)

Status: ready-for-dev

<!-- Story-zéro de l'épopée 17-3 (export/import installation, #112). Extraite de la spec umbrella `17-3-export-import-installation.md` convergée au validate en 5 passes (Sonnet→Haiku→Opus→Sonnet→Haiku, trend 21→16→10→3→0, ~50 patches, dont catch-architectural Opus P3). Contenu déjà adversarialement revu. Re-validate optionnel. -->
<!-- POSE LE FORMAT `.keshbackup` consommé par 17-3b..f. Bloque 17-3b (UI export) et 17-3c (backend import). -->

## Story

As a **administrateur d'une installation Kesh**,
I want **un endpoint backend qui exporte toute l'installation (toutes companies + tous utilisateurs + données système) dans un fichier `.keshbackup` unique, téléchargeable et auto-portant**,
so that **je dispose d'un artefact de sauvegarde/migration complet, dont le format servira aussi de base à l'import (17-3c)**.

## Contexte & cadrage

**Épopée 17-3 (#112) — split A–F :** 17-3a (cette story, backend export, **story-zéro**) → 17-3b UI export → 17-3c backend import → 17-3d UI import → 17-3e E2E → 17-3f doc. La spec **umbrella** `17-3-export-import-installation.md` reste la source de contexte complète (décisions DC1–DC11, inventaire tables, risques).

**Rôle de story-zéro :** cette story **définit et fige le format `.keshbackup`** (conteneur + manifest + sérialisation NDJSON) que toutes les autres sous-stories consomment. Elle **promeut** aussi la liste canonique des tables (`TABLES_TO_TRUNCATE`) hors du code de test vers un module de production, car l'export en a besoin pour énumérer les tables (dépendance identifiée au validate Pass 3, O-10).

**⚠️ Découverte ground-truth (anti-réinvention) :** **Kesh ne stocke AUCUN fichier binaire uploadé sur disque** (imports bancaires = hash SHA-256 seul en DB ; PDFs générés on-demand). Le dossier `files/` du `.keshbackup` est **vide en v0.2** (réservé forward-compat). NE PAS inventer de mécanisme de sauvegarde de fichiers.

**Pas de migration DB** dans cette story (export = lecture pure).

## Acceptance Criteria

1. `GET /api/v1/admin/full-export` retourne un fichier `.keshbackup` (conteneur ZIP) téléchargeable (**GET** — pas de corps de requête, cohérent 9-2b `GET /exports/global.zip` et `apiClient.getBlob` câblé sur GET), `Content-Type: application/octet-stream` (force le download sans réinterprétation d'extension) + `Content-Disposition: attachment; filename="kesh-installation-{YYYY-MM-DD}.keshbackup"` (réutiliser `util::build_content_disposition`). **Réservé rôle Admin strict** (test : `Comptable` et `Consultation` → `403`).
2. L'endpoint est **inaccessible via clé PAT** : requête `Authorization: Bearer kesh_pat_…` (même scope `read-write`) → `403` code `API_KEY_MANAGEMENT_FORBIDDEN` (le backup contient hash de mots de passe + tokens, fuite critique si exposé via API ; cohérent 17-2a DC6).
3. Le `.keshbackup` contient exactement : `manifest.json` (au ROOT), `data/<table>.ndjson` (1 par table applicative = **22 tables**, nom = nom exact de la table), un dossier `files/` (vide). Tables **système exclues** : `_sqlx_migrations`, `_kesh_version`. *(Voir §Format normatif pour la structure exacte.)*
4. `manifest.json` (camelCase, `serde_json::to_vec_pretty`, trailing `\n`) contient : `formatVersion: 1` (figé), `keshVersion` (= `env!("CARGO_PKG_VERSION")`), `keshVersionMinRequired` (lu de `_kesh_version`), `exportDate` (ISO 8601 UTC précision seconde), `instanceId` (= `MIN(companies.id)`, informatif), et par table : `rowCount`, `sha256` (hash NDJSON décompressé), **`columnNames: string[]`** (ordonné, explicite, **exclut les colonnes générées** type `reconciliation_rules.active_uniq` `GENERATED … VIRTUAL`).
5. **Intégrité vérifiable** : recalculer le SHA-256 du NDJSON de chaque table redonne exactement `manifest.json[tables][*].sha256` (réutiliser `sha2`/`sha256_hex`). Table vide → NDJSON 0 octet → sha256 de la chaîne vide.
6. **Audit-trail** : chaque export insère une entrée `audit_log` (action `admin.full_export`, `entity_type` `installation`, `details_json` snake_case : `file_size`, `table_count`, `total_rows`, `kesh_version`) via `NewAuditLogEntry::from_current_user` (actor JWT — AC2 interdit le PAT). L'audit est émis **dans le handler** `full_export`, PAS dans le cœur `build_keshbackup()` (qui sera réutilisé sans audit par le backup pré-import de 17-3c).
7. **Maîtrise mémoire (DC8)** : assemblage **in-memory** sous plafond `KESH_ADMIN_EXPORT_INMEM_MB` (défaut 50, bornes [1, 2048], parse+borne+warn ; log WARN si > 500) ; **au-delà → fichier temporaire** (`tokio::fs`) streamé via `Body::from_stream` (`tokio_util::io::ReaderStream`). Plafond documenté (`.env.example`).
8. **Sécurité doc** : le `.keshbackup` n'est **pas chiffré** (responsabilité utilisateur) et **contient `users.password_hash` + `refresh_tokens`** → c'est un **secret**. *(La doc utilisateur complète est en 17-3f ; ici, juste un commentaire `///` sur le handler/format avertissant du caractère sensible.)*

## Tasks / Subtasks

- [ ] **T-A1** Module `crates/kesh-api/src/admin_backup/` (`mod.rs` + `format.rs` + `manifest.rs`). Définir `BackupManifest`/`TableMeta` (schéma AC4). **`build_backup_manifest_json(&BackupManifest) -> Result<Vec<u8>, AppError>`** (NE PAS modifier `exports::metadata::build_metadata_json`, signature per-company incompatible). **Cœur factorisé `build_keshbackup(pool) -> Result<…, AppError>` SANS audit** (réutilisé en 17-3c par le backup pré-import). Conteneur ZIP via crate `zip` v2 (réutiliser `exports::global::build_zip`). (AC: 3, 4)
- [ ] **T-A2** Sérialiser les **22 tables** en NDJSON (installation entière, pas de scope company) avec **liste de colonnes explicite excluant les colonnes générées** (`active_uniq` VIRTUAL exclue). Hash NDJSON par table via `sha256_hex` (`exports/metadata.rs`). **Promouvoir la liste canonique de tables** : **déplacer** (move, pas copy) `TABLES_TO_TRUNCATE` de `crates/kesh-db/src/test_fixtures.rs:297` vers **`crates/kesh-db/src/backup.rs` en `pub`** ; `test_fixtures.rs` **réexporte** la constante canonique ; déplacer/adapter le test de synchro `truncate_all_inventory_matches_schema` (`test_fixtures.rs:711`) pour valider la constante de `backup.rs`. (AC: 3, 4, 5)
- [ ] **T-A3** Handler `full_export` en **GET** (`crates/kesh-api/src/routes/admin.rs`, nouveau module) : appelle `build_keshbackup`, `Content-Type: application/octet-stream` + `Content-Disposition` (réutiliser `util::build_content_disposition`). (AC: 1)
- [ ] **T-A4** Mémoire/streaming (DC8) : in-memory sous `KESH_ADMIN_EXPORT_INMEM_MB` (défaut 50, bornes [1,2048], `config.rs` pattern `bank_import_max_mib`), au-delà fichier temp `tokio::fs` + `Body::from_stream` (`tokio_util::io::ReaderStream`). **Ajouter `tokio-util` 0.7 à `crates/kesh-api/Cargo.toml`** (absent actuellement). (AC: 7)
- [ ] **T-A5** RBAC + anti-PAT : monter la route GET dans `admin_routes` (`lib.rs:101-127`, `route_layer(require_admin_role)` déjà câblé). **Promouvoir `ensure_not_pat` en `pub(crate)`** dans `routes/api_keys.rs:91` (signature `fn ensure_not_pat(&CurrentUser) -> Result<(), AppError>` : si `api_key_id.is_some()` → `Err(AppError::ApiKeyManagementForbidden)`) ; l'appeler en tête du handler. (AC: 1, 2)
- [ ] **T-A6** Audit `admin.full_export` via `NewAuditLogEntry::from_current_user` + `audit_log::insert_in_tx`, **dans le handler** (pas dans `build_keshbackup`). (AC: 6)
- [ ] **T-A7** `AppError` variant(s) (`AdminFullExportFailed` ou réutiliser `GlobalExportFailed`) + codes i18n (`errors.rs`). Doc `///` avertissant que le `.keshbackup` est un secret (AC8). **Tests** : unit (manifest shape `formatVersion`/`columnNames`/`instanceId`, sha256 round-trip, **exclusion de la colonne générée**, structure ZIP) + E2E/intégration (`crates/kesh-api/tests/admin_full_export_e2e.rs` : 200+ZIP valide, RBAC non-Admin 403, anti-PAT 403, structure 22 `.ndjson`+manifest+`files/`, intégrité SHA, audit inséré). (AC: 1, 2, 3, 4, 5, 6, 8)

## Dev Notes

### Décisions de conception (figées au validate umbrella — voir spec parente pour le détail complet)

| # | Décision (périmètre 17-3a) |
|---|---|
| **DC1** | Format données = **NDJSON ligne-par-ligne par table**. Fidélité de type, restore paramétré (17-3c) sans injection. SQL-dump écarté. `columnNames` requis (NDJSON vide n'expose pas les colonnes) + exclut les colonnes générées. |
| **DC2** | Scope = **22 tables applicatives, installation entière** (PAS per-company). Distinct de 9-2b. |
| **DC3** | **Aucun fichier binaire** (no-op v0.2), dossier `files/` vide forward-compat. |
| **DC7** | Admin strict **+ anti-PAT** (`403 API_KEY_MANAGEMENT_FORBIDDEN`). |
| **DC8** | Export in-memory < `KESH_ADMIN_EXPORT_INMEM_MB` (défaut 50), au-delà fichier temp + `Body::from_stream`. |
| **DC9** | Aucune migration DB. |

### Format `.keshbackup` (spec NORMATIVE — figée ici, story-zéro)

> Source de vérité unique du contrat export↔import. Tout écart 17-3a↔17-3c casse l'intégrité SHA.

**Structure ZIP (exacte)** :
```
manifest.json              # au ROOT
data/<table>.ndjson        # 1 par table applicative (22), nom = nom exact de la table
files/                     # dossier vide en v0.2 (forward-compat)
```

**Sérialisation NDJSON** (1 objet JSON par ligne, séparés par `\n` LF) :
- UTF-8 **sans BOM**, fin de ligne **LF** (`\n`), pas de ligne finale vide superflue.
- `serde_json` standard. **NULL → `null` JSON** (jamais omis ni `"NULL"`).
- **Ordre des clés = `columnNames` du manifest** (déterministe, identique export/import).
- `Decimal` (rust_decimal) → string JSON ; `NaiveDateTime` → `"YYYY-MM-DDTHH:MM:SS"` ; enums → leur repr `serde`.
- Table vide → fichier `data/<table>.ndjson` de **0 octet**.

**`manifest.json`** — exemple abrégé :
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
- `exportDate` : `Utc::now().to_rfc3339_opts(SecondsFormat::Secs, true)` (précision seconde, cohérent `metadata.rs:78`).
- `instanceId` (`i64`) : `MIN(companies.id)` au moment de l'export. **Invariant** : toujours ≥ 1 (l'endpoint exige un Admin authentifié et `users.company_id` est `NOT NULL` → ≥ 1 company existe). Informatif uniquement.
- `columnNames` : **ordonné, exclut les colonnes générées** (`reconciliation_rules.active_uniq` `GENERATED ALWAYS AS … VIRTUAL`, `migrations/20260513000001_reconciliation_rules.sql:54`). C'est la liste que 17-3c utilisera pour l'INSERT.
- `sha256` : hash des **bytes NDJSON décompressés** (post-extraction ZIP, LF). Table vide → `e3b0c4…` (sha256 de la chaîne vide).

### Inventaire des 22 tables applicatives (à exporter)

Source : migrations + `TABLES_TO_TRUNCATE` (`crates/kesh-db/src/test_fixtures.rs:297-320`) :
```
invoice_lines, journal_entry_lines, invoices, invoice_number_sequences,
journal_entries, audit_log, api_keys, company_invoice_settings,
bank_transactions, bank_imports, bank_profiles, reconciliation_rules,
bank_accounts, accounts, products, contacts, fiscal_years, vat_rates,
refresh_tokens, onboarding_state, users, companies
```
**Toutes exportées** (l'export est un snapshot complet, `onboarding_state` incluse). **Exclues** : `_sqlx_migrations`, `_kesh_version` (système). ⚠️ `accounts` a une FK self-référente (`parent_id`) — sans impact en lecture/export.

### Réutilisation — moteur export 9-2b (`crates/kesh-api/src/exports/`)

> **Ne pas réinventer.** Réutiliser les briques, pas le handler (scope per-company ≠ installation).

| Brique | Chemin:ligne | Réutilisation |
|---|---|---|
| Construction ZIP | `exports/global.rs:80-99` (`build_zip`, crate `zip` v2 deflate, `kesh-api/Cargo.toml:50`) | Directe |
| SHA-256 | `exports/metadata.rs:89-95` (`sha256_hex`, `sha2` 0.10, `Cargo.toml:47`) | Directe |
| Manifest JSON | `exports/metadata.rs:71` (`build_metadata_json(company:&Company, …)` — **signature per-company incompatible**) | **NE PAS réutiliser** ; créer `build_backup_manifest_json(&BackupManifest)` |
| `Content-Disposition` RFC 5987 | `util::build_content_disposition` (`util.rs:104-170`) | Directe |
| Handler de référence | `routes/exports.rs:37-112` (`export_global`) | **Pattern** (scope ≠) |
| `AppError::GlobalExportFailed` | `errors.rs:236-237` → 500 | Modèle pour `AdminFullExportFailed` |
| Audit best-effort | `routes/exports.rs:151-176` (`emit_global_export_audit`) | Modèle (action `admin.full_export`) |

⚠️ Le format **CSV** de 9-2b est **inadapté** au round-trip (perte de type). Utiliser **NDJSON** (DC1).

### Réutilisation — RBAC / audit / config / version (ground-truth)

| Brique | Chemin:ligne | Usage |
|---|---|---|
| `admin_routes` sub-router | `lib.rs:101-127` (`route_layer(require_admin_role)`) | Ajouter `GET /api/v1/admin/full-export` |
| `require_admin_role` | `middleware/rbac.rs:31` (hiérarchie `Role` `Ord`, `entities/user.rs:13-27`, `Role::Admin → "Admin"`) | RBAC AC1 |
| Anti-PAT | `routes/api_keys.rs:91` `fn ensure_not_pat(&CurrentUser) -> Result<(),AppError>` (`api_key_id.is_some()` → `ApiKeyManagementForbidden`) — **privé**, à promouvoir `pub(crate)` | AC2 |
| `CurrentUser` | `middleware/auth.rs:37-45` (`api_key_id: Option<i64>`) | Discrimination JWT/PAT |
| Audit | `audit.rs:18-54` (`AuditActor::from_current_user`), `audit_log::insert_in_tx`, ex. `bank_imports.rs:1476-1487` | Action `admin.full_export` |
| Version min_required | `migrations/20260522000001_kesh_version.sql:31-41` (`_kesh_version`, `id` TINYINT UNSIGNED) ; lire `kesh_version_min_required` | Manifest AC4 |
| Version binaire | `env!("CARGO_PKG_VERSION")` (`kesh-api/Cargo.toml:3`), `/health` `routes/health.rs:25` | Manifest `keshVersion` |
| Config | `config.rs:357` (`bank_import_max_mib:10`, pattern parse+borne+warn) | `KESH_ADMIN_EXPORT_INMEM_MB` (défaut 50 [1,2048]) |
| Colonne générée | `migrations/20260513000001_reconciliation_rules.sql:54` (`active_uniq … VIRTUAL`) | **Exclure** de `columnNames` + NDJSON |
| Tables canoniques | `test_fixtures.rs:297` (`TABLES_TO_TRUNCATE` `pub(crate)`) + test synchro `:711` | **Déplacer** vers `kesh-db/src/backup.rs` `pub` (move-not-copy) |

### Standards projet (CLAUDE.md)

- **Test Locally First** avant push qui ouvre/MAJ une PR : `cargo fmt --all --check` + `build --workspace --all-targets` + `clippy -D warnings` + `test --workspace` (mode serial `-j1 --test-threads=1` si la modif touche `kesh-db`/tests d'intégration DB — c'est le cas ici via `backup.rs` + tests E2E).
- **Migration breaking policy** : N/A (aucune migration).
- **Pattern batch `FailedProposal`** : N/A (export = lecture, pas de batch per-proposal).
- **Commit par étape BMAD**, pas de push auto (sauf demande Guy / fin epic).
- Branche déjà active : `story/17-3-export-import-installation` (stack des sous-stories 17-3 ; ou nouvelle branche `story/17-3a-backend-export` selon préférence).

### Project Structure Notes

- **Backend** : nouveau `crates/kesh-api/src/admin_backup/` (logique format/manifest/export) + `crates/kesh-api/src/routes/admin.rs` (handler). Aligné avec la séparation `exports/` (logique) vs `routes/exports.rs` (handler). Nouveau module prod `crates/kesh-db/src/backup.rs` (constante tables promue).
- **Router** : premier namespace `/api/v1/admin/*` (L1 : routes admin existantes non renommées en v0.2, asymétrie assumée). Monter dans `admin_routes` (RBAC câblé).
- **Dépendance ajoutée** : `tokio-util` 0.7 (`kesh-api/Cargo.toml`).

### References

- [Source: _bmad-output/implementation-artifacts/17-3-export-import-installation.md] — spec umbrella (DC1–DC11, §Format normatif, inventaire, risques, Change Log 5 passes)
- [Source: _bmad-output/planning-artifacts/epic-17.md#Story 17-3] — D6/D7
- [Source: github #112] — ACs origine
- [Source: crates/kesh-api/src/exports/] — moteur export réutilisable
- [Source: crates/kesh-db/src/test_fixtures.rs:297-371] — `TABLES_TO_TRUNCATE` à promouvoir
- [Source: crates/kesh-api/src/lib.rs:101-127, routes/api_keys.rs:91] — admin_routes + ensure_not_pat

## Dev Agent Record

### Agent Model Used

_(à compléter par dev-story)_

### Debug Log References

### Completion Notes List

### File List

### Change Log

| Date | Étape | Modèle | Résumé |
|------|-------|--------|--------|
| 2026-06-08 | create-story (sous-story) | Opus 4.8 | Story-zéro 17-3a extraite de l'umbrella 17-3 convergée 5 passes (contenu déjà adversarialement revu, dont catch-architectural Opus P3). Scope : module `admin_backup/` + `build_keshbackup` (sans audit) + `build_backup_manifest_json` + `GET /admin/full-export` (Admin + anti-PAT) + streaming DC8 + audit + promotion `TABLES_TO_TRUNCATE` (move-not-copy) + dep `tokio-util`. Pose le format `.keshbackup` (§normative). AC1-8. Bloque 17-3b/c. Re-validate optionnel. Prochaine : `bmad-dev-story 17-3a` (Opus). |
