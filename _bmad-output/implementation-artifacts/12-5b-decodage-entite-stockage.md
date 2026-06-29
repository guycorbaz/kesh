# Story 12.5b: Décodage serveur + entité + stockage (socle import)

Status: ready-for-dev

<!-- Sous-story 2/4 de l'umbrella 12-5 (validate convergé 6 passes). Périmètre T2 (décodage pdfium/rxing/image + Docker) + T3 (migration imported_supplier_invoices + stockage KESH_DOCUMENTS_DIR + intégration backup). Dépend de 12-5a (parse_spc_payload livré, commit 85a235b). NE COUVRE PAS : lecture inbox / rapport batch / sécurité / endpoints / complétion (12-5c), ni frontend (12-5d). -->

## Story

As a développeur de l'import de factures (12-5),
I want le **moteur de décodage QR côté serveur** (PDF via pdfium → image → rxing, images directes via rxing) et la **persistance d'une facture importée** (table `imported_supplier_invoices` + copie du fichier sur disque),
so that la couche service 12-5c dispose d'un socle décodage+entité+stockage testable en isolation, sans dépendre encore de la lecture du dossier ni des endpoints.

## Contexte & source

- Sous-story **2/4** de l'umbrella **12-5** (import répertoire factures, issue #194, cible v0.4). Cf. `12-5-import-repertoire-factures.md` (AC2, AC5, schéma, DC1/DC1-bis/DC3/DC4/DC5, Dev Notes ground-truth).
- **Dépend de 12-5a** : `parse_spc_payload` + `ScannedQrBill`/`ScannedAddress`/`ScannedReference` livrés dans `kesh-qrbill` (commit `85a235b`).
- **Débloque 12-5c** (service d'import + endpoints + complétion) qui consomme le module de décodage et le repository d'entité livrés ici.
- Introduit une **dépendance native** (`pdfium-render` + binaire `libpdfium.so`) ratifiée DC1/DC1-bis par Guy (2026-06-28/29).

## Périmètre (et hors-périmètre)

**DANS 12-5b** :
- **T2** — Module de décodage QR serveur : `rxing` (runtime) + `image` (explicite) + `pdfium-render` ajoutés à `kesh-api` ; fonctions de décodage image directe (PNG/JPG) et PDF (rendu page → image → rxing, multi-page, multi-QR par page, caps pages/dimensions) ; bundling `libpdfium.so` (amd64) dans le `Dockerfile`.
- **T3** — Entité + stockage : migration `CREATE TABLE imported_supplier_invoices` (schéma figé) ; entité `kesh-db` + repository (create staging + finders socle) ; helper de stockage `KESH_DOCUMENTS_DIR` (`{sha256hex}.{ext}` + SHA-256) ; config env `KESH_DOCUMENTS_DIR` ; intégration backup/export (TABLES_TO_TRUNCATE, manifeste export, compteurs, audit idempotence).

**HORS 12-5b** (→ 12-5c / 12-5d) :
- Lecture du dossier inbox, verrou de run, déplacement `failed/`, suppression inbox, rapport batch `{accepted, failed, warnings}` (12-5c).
- Sécurité d'ingestion : symlink, anti-traversal, magic bytes, stabilité fichier, liste blanche (12-5c — **mais** le helper de stockage 12-5b garantit déjà le nommage anti-traversal `{sha256hex}.{ext}`).
- Endpoints HTTP (liste / complétion / écarter / download), `create_in_tx` (DC6), réconciliation montant (12-5c).
- Frontend, doc utilisateur (12-5d).

## Acceptance Criteria

### Décodage serveur (T2)

1. **Dépendances** : `rxing` passe de `[dev-dependencies]` (kesh-qrbill) à **runtime** là où nécessaire, et `rxing` + `image` + `pdfium-render` sont déclarés explicitement dans `crates/kesh-api/Cargo.toml [dependencies]` (`image` est aujourd'hui transitive non déclarée — cf. Dev Notes). Versions épinglées cohérentes avec `Cargo.lock` (`rxing 0.7`, `image ~0.25`). `lopdf` n'est PAS utilisé pour le rendu (ne rastérise pas — écarté DC1).

2. **Module de décodage** (nouveau module `kesh-api`, ex. `src/qr_decode.rs` ou `src/import/decode.rs`) exposant au minimum :
   - `decode_qr_from_image(img: &image::DynamicImage) -> Vec<String>` — retourne **tous** les payloads QR décodés par `rxing` sur l'image (ordre rxing).
   - `decode_spc_from_image_bytes(bytes: &[u8]) -> Result<Option<ScannedQrBill>, DecodeError>` — décode l'image (PNG/JPG) → pour chaque QR, **garder le premier payload commençant par `SPC\n0200`** → `parse_spc_payload` (12-5a). `None` si aucun QR SPC.
   - `decode_spc_from_pdf_bytes(bytes: &[u8], cfg) -> Result<Option<ScannedQrBill>, DecodeError>` — **pdfium-render** : rendre chaque page (jusqu'au cap) → `image::DynamicImage` → `decode_qr_from_image` → 1er payload `SPC\n0200` → `parse_spc_payload`. **Multi-page** : tenter chaque page jusqu'au 1er QR SPC valide. **Plusieurs QR par page** : tenter chaque QR dans l'ordre. `None` si aucune page ne porte de QR SPC.
   - Un type d'erreur de décodage (`DecodeError` ou réutilisation d'un enum existant) couvrant au moins : `PdfRender` (pdfium échoue), `InvalidSpcPayload` (QR décodé mais `parse_spc_payload` échoue — porte `QrBillError`), `InvalidIban` (propagé du parseur). Le mapping vers les `error_code` HTTP (`PDF_RENDER_ERROR`, `INVALID_SPC_PAYLOAD`, `INVALID_IBAN`, `NO_QR_CODE_FOUND`) est **consommé en 12-5c** ; 12-5b expose une erreur typée exploitable.

3. **Robustesse rendu pdfium (caps — F4/L6)** : le rendu PDF borne (a) le **nombre de pages rendues** via `KESH_INBOX_MAX_PDF_PAGES` (défaut ex. 20 ; au-delà sans QR trouvé → erreur `PdfRender`/`PDF_RENDER_ERROR`) et (b) les **dimensions de rendu** (DPI / pixels max bornés) pour éviter un MediaBox gigapixel. **L6 documentée** : pdfium est natif in-process, un segfault sur PDF malformé tue le process (non rattrapable `catch_unwind`) — risque accepté v0.4.

4. **Packaging Docker pdfium (DC1-bis — amd64)** : le stage `runtime` du `Dockerfile` (`debian:bookworm-slim`) embarque `libpdfium.so` :
   - Téléchargé depuis une **release épinglée** de `bblanchon/pdfium-binaries` (tag `chromium/NNNN` figé, **pas `latest`**) — `pdfium-linux-x64.tgz`.
   - **Checksum SHA-256 vérifié** avant extraction.
   - Placé dans `/usr/local/lib/libpdfium.so` + `ldconfig` (chemin standard, sans `LD_LIBRARY_PATH`). Le dev confirme le nom attendu par `pdfium-render`.
   - **`linux/amd64` uniquement** (cohérent `release.yml:53`) — pas de multi-arch arm64 (**L3**). Licence pdfium (Apache-2.0/BSD-3) mentionnée dans la doc (la doc utilisateur détaillée est 12-5d ; ici, commentaire `Dockerfile` + note licence suffisent).

5. **Tests décodage** : fixtures binaires commitées — **(a)** une image **PNG** portant un QR SPC valide → `decode_spc_from_image_bytes` → `Some(ScannedQrBill)` cohérent ; **(b)** un **PDF 1 page** portant un QR SPC → `decode_spc_from_pdf_bytes` → `Some(...)` ; **(c)** un PDF **multi-page** où le QR n'est pas sur la 1ʳᵉ page → trouvé ; **(d)** image/PDF **sans QR** → `Ok(None)` (pas d'erreur) ; **(e)** PDF illisible/corrompu → `Err(PdfRender)`. Les fixtures QR SPC peuvent être générées à partir de `build_payload` (12-5a) encodé en QR (via `rxing` writer ou fixture pré-générée commitée). **Note CI** : ces tests nécessitent `libpdfium.so` présent dans l'environnement de test — si absent en CI host (hors Docker), `#[ignore]` documenté + exécution en local/Docker (décision dev à documenter, cohérent contrainte native).

### Entité + stockage (T3)

6. **Migration `CREATE TABLE imported_supplier_invoices`** — **non-breaking** (pas de bump `kesh_version_min_required`). Schéma **exactement** celui figé dans l'umbrella (§Schéma ci-dessous), conventions calquées sur `20260628000001_supplier_invoices.sql` : multi-tenant `company_id` FK `companies` RESTRICT, statut via `CHECK` texte (PAS d'enum SQLx — `feedback_sqlx_mysql_gotchas`), `version INT`, `DATETIME(3)`. Index `uq_imported_company_hash (company_id, file_hash)` UNIQUE + `idx_imported_company_status (company_id, status)` + `idx_imported_supplier_invoice (supplier_invoice_id)`.

7. **Entité + repository** (`crates/kesh-db/src/entities/` + `repositories/`) :
   - Struct lecture `ImportedSupplierInvoice` (tous les champs) + `NewImportedSupplierInvoice` (champs d'insertion : `company_id` + colonnes document + colonnes QR ; `status` défaut `to_complete`, `supplier_invoice_id` = `None`).
   - Repository socle : `create(&pool, NewImportedSupplierInvoice) -> Result<ImportedSupplierInvoice, DbError>` (INSERT staging) ; `find_by_id_scoped(company_id, id)` ; `find_by_company_hash(company_id, file_hash) -> Option<...>` (utilisé par 12-5c pour l'idempotence/réactivation) ; `list_by_status(company_id, status)`. **Multi-tenant** : tout finder est scopé `company_id`.
   - **Mapping `ScannedQrBill` → `NewImportedSupplierInvoice`** : helper (ou `From`/constructeur) qui projette `creditor_iban`, `is_qr_iban`, `creditor.address_type`/`name`/`street_or_line1`→`creditor_line1`/`building_or_line2`→`creditor_line2`/`postal_code`/`town`/`country`, `amount`, `currency`, `reference` (→ `reference_type` ∈ {QRR,SCOR,NON} + `reference_value`), `unstructured_message`, `billing_information` + les métadonnées document. **Aucun champ orphelin** (cf. umbrella Pass 6).
   - **NE PAS** implémenter ici la logique de réactivation `discarded`→`to_complete` ni la complétion (`create_in_tx`) — **12-5c**.

8. **Helper de stockage `KESH_DOCUMENTS_DIR`** :
   - Nouvelle config env `KESH_DOCUMENTS_DIR` (pattern `from_env` comme `KESH_ADMIN_BACKUP_DIR` `config.rs:896`) — **défaut `/data/documents`** (PAS `/tmp` : perte au redémarrage conteneur). Champ ajouté à la struct `Config`.
   - `store_document(documents_dir, src_bytes_or_path, ext) -> Result<StoredDocument, _>` : calcule **SHA-256 hex**, écrit sous `{documents_dir}/{sha256hex}.{ext}`, retourne `{ storage_path (relatif), sha256, byte_size, mime_type }`. **Anti-traversal garanti** : le nom du fichier archivé est dérivé du hash, **jamais** du nom d'origine. `original_filename` n'est utilisé qu'en colonne DB (affichage).
   - `read_document(documents_dir, storage_path) -> Result<Vec<u8>, _>` distinguant **`ENOENT` (fichier absent)** d'une autre erreur I/O — exploité en 12-5c pour renvoyer 404/410 plutôt que 500 (F7). 12-5b expose la distinction ; le mapping HTTP est 12-5c.
   - SHA-256 via une crate déjà au workspace si disponible (`sha2`) — vérifier `Cargo.lock` avant d'ajouter.

9. **Intégration backup/export & compteurs** (la nouvelle table entre dans le périmètre `.keshbackup` — DC5 métadonnées seules) :
   - Ajouter `"imported_supplier_invoices"` à **`TABLES_TO_TRUNCATE`** (`crates/kesh-db/src/backup.rs:34`) **avant `supplier_invoices`** (l.41) et **avant `companies`** (l.64) — zone enfants. Restore insère en ordre inverse (`.rev()` l.432) → la table est insérée **après** ses parents (FK `company_id` RESTRICT satisfaite ; `supplier_invoice_id` SET NULL nullable moins contraignant). Le test `backup_inventory_matches_schema` (`backup.rs:570`) attrape l'oubli de table mais **PAS** un mauvais ordre FK → vérifier l'ordre à la main.
   - Ajouter la table au **manifeste d'export** (`crates/kesh-api/src/admin_backup/export.rs`) si la liste est explicite.
   - Bumper les **compteurs en dur** des tests : `admin_full_export_e2e` (`data_count`) + `migrations_upgrade_path` (`crates/kesh-api/tests/admin_backup_e2e.rs` + `admin_full_export_e2e.rs`).
   - Ajouter la ligne d'**audit idempotence** dans `docs/migrations-idempotence-audit.md` (verdict pour la nouvelle migration — `tracked-by-sqlx` ou justification).
   - **L1 (DC5)** : seules les **métadonnées** (`storage_path`/`sha256`/…) entrent au backup ; le **binaire** de `KESH_DOCUMENTS_DIR` reste hors `.keshbackup` (limitation documentée, remédiation Epic 14).

10. **Tests T3** : migration applique proprement (sqlx) ; `create` + finders scopés `company_id` (intégration DB, isolement multi-tenant : company A ne voit pas le staging de company B) ; `UNIQUE (company_id, file_hash)` rejette un doublon **même company** mais accepte le **même hash sur 2 companies** ; mapping `ScannedQrBill`→`NewImportedSupplierInvoice` (round-trip champ par champ) ; helper stockage (`store_document` écrit `{sha256hex}.{ext}`, `read_document` distingue ENOENT) ; `backup_inventory_matches_schema` reste vert.

### Quality gate (T2+T3)

11. **Test Locally First — exit code vérifié** (PAS `cargo test | grep`) : `cargo fmt --all --check` + `cargo clippy --workspace --all-targets -- -D warnings` + `cargo build --workspace --all-targets` + `cargo test --workspace -j1 -- --test-threads=1`. 0 régression. Documenter le sort des tests pdfium en CI host sans `libpdfium.so` (cf. AC5 note).

## Schéma `imported_supplier_invoices` (figé umbrella)

Migration `CREATE TABLE` **non-breaking**.

| Colonne | Type | Null | Notes |
|---|---|---|---|
| `id` | BIGINT PK AUTO_INCREMENT | non | |
| `company_id` | BIGINT | non | FK `companies(id)` ON DELETE RESTRICT — multi-tenant |
| `status` | VARCHAR(16) DEFAULT `'to_complete'` | non | CHECK IN (`to_complete`,`completed`,`discarded`) |
| `supplier_invoice_id` | BIGINT | **oui** | FK `supplier_invoices(id)` ON DELETE SET NULL — renseigné à la complétion (12-5c) |
| `file_hash` | CHAR(64) | non | SHA-256 hex (idempotence) |
| `storage_path` | VARCHAR(512) | non | relatif à `KESH_DOCUMENTS_DIR`, = `{sha256hex}.{ext}` |
| `original_filename` | VARCHAR(255) | non | nom d'origine (affichage seul) |
| `mime_type` | VARCHAR(100) | non | |
| `byte_size` | BIGINT | non | |
| `creditor_iban` | VARCHAR(34) | non | IBAN ou QR-IBAN |
| `is_qr_iban` | BOOLEAN | non | IID 30000–31999 |
| `creditor_address_type` | CHAR(1) | non | CHECK IN (`'K'`,`'S'`) |
| `creditor_name` | VARCHAR(70) | non | |
| `creditor_line1` | VARCHAR(70) | oui | line1 (K) / street (S) |
| `creditor_line2` | VARCHAR(70) | oui | line2 (K) / building_no (S) |
| `creditor_postal_code` | VARCHAR(16) | oui | type S |
| `creditor_town` | VARCHAR(35) | oui | type S |
| `creditor_country` | CHAR(2) | non | |
| `reference_type` | VARCHAR(8) | non | CHECK IN (`QRR`,`SCOR`,`NON`) |
| `reference_value` | VARCHAR(40) | oui | vide si `NON` |
| `amount` | DECIMAL(19,4) | **oui** | SPC autorise montant vide |
| `currency` | VARCHAR(3) | non | validée CHF/EUR à la complétion (12-5c) |
| `unstructured_message` | VARCHAR(140) | oui | |
| `billing_information` | VARCHAR(140) | oui | |
| `version` | INT DEFAULT 1 | non | |
| `created_at` | DATETIME(3) DEFAULT CURRENT_TIMESTAMP(3) | non | |
| `updated_at` | DATETIME(3) … ON UPDATE | non | |

## Tasks / Subtasks

- [ ] **T2.1** — Déplacer `rxing` dev→runtime + déclarer `rxing`/`image`/`pdfium-render` dans `kesh-api/Cargo.toml` (versions cohérentes Cargo.lock).
- [ ] **T2.2** — Module `qr_decode` : `decode_qr_from_image`, `decode_spc_from_image_bytes`, `decode_spc_from_pdf_bytes` (multi-page, multi-QR, 1er `SPC\n0200`) + type `DecodeError`. Caps pdfium (pages `KESH_INBOX_MAX_PDF_PAGES`, dimensions).
- [ ] **T2.3** — `Dockerfile` : bundle `libpdfium.so` (tag épinglé + checksum SHA-256 + `/usr/local/lib` + `ldconfig`, amd64). Commentaire licence.
- [ ] **T2.4** — Tests décodage (fixtures PNG/PDF 1p/PDF multipage/sans-QR/corrompu) + gestion `libpdfium.so` absent en CI host (`#[ignore]` documenté si besoin).
- [ ] **T3.1** — Migration `CREATE TABLE imported_supplier_invoices` (schéma figé, non-breaking, CHECK texte) + ligne audit idempotence.
- [ ] **T3.2** — Entité `ImportedSupplierInvoice`/`NewImportedSupplierInvoice` + repository (`create`, `find_by_id_scoped`, `find_by_company_hash`, `list_by_status`) + mapping `ScannedQrBill`→`New…`.
- [ ] **T3.3** — Config `KESH_DOCUMENTS_DIR` (défaut `/data/documents`) + helper stockage `store_document`/`read_document` (SHA-256, `{sha256hex}.{ext}`, distinction ENOENT).
- [ ] **T3.4** — Backup/export : `TABLES_TO_TRUNCATE` (ordre FK), manifeste export, compteurs `admin_full_export_e2e`/`migrations_upgrade_path`, audit idempotence.
- [ ] **T3.5** — Tests T3 (migration, repo scopé, UNIQUE company/hash, mapping, stockage, `backup_inventory_matches_schema`).

## Dev Notes

### Ground-truth (vérifié 2026-06-29)
- `crates/kesh-api/Cargo.toml` : `[dependencies]` l.7, `[dev-dependencies]` l.64 — **ni `rxing`, ni `pdfium-render`, ni `image`** déclarés → à ajouter.
- `crates/kesh-qrbill/Cargo.toml:17` : `rxing = "0.7"` en **`[dev-dependencies]`** → passer runtime côté kesh-api. `image` transitive (~0.25.9) non déclarée → déclarer explicitement.
- `Dockerfile` : builder `rust:1.85-bookworm` (l.2), runtime `debian:bookworm-slim` (l.18), `apt-get install ca-certificates curl` (l.20), `COPY target/release/kesh-api` (l.23). **Aucune lib native** → ajouter pdfium dans le stage runtime.
- `crates/kesh-api/src/config.rs` : pattern `env::var("KESH_ADMIN_BACKUP_DIR").unwrap_or_else(|_| "/tmp")` (l.896), champ `admin_backup_dir` (l.249/1042), `from_env` (l.517). Reproduire pour `KESH_DOCUMENTS_DIR` **avec défaut `/data/documents`** (PAS `/tmp`).
- `crates/kesh-db/src/backup.rs:34` `TABLES_TO_TRUNCATE` : ordre enfants→parents — `credit_notes` (l.40), `supplier_invoices` (l.41), `payment_batches` (l.42), … `companies` (l.64). Restore `.rev()` (l.432). Test `backup_inventory_matches_schema` (l.570) = filet table-présente (pas l'ordre FK).
- Compteurs : `crates/kesh-api/tests/admin_backup_e2e.rs` (`migrations_upgrade_path` + `data_count`), `crates/kesh-api/tests/admin_full_export_e2e.rs`.
- `crates/kesh-db/src/entities/supplier_invoice.rs` + `repositories/supplier_invoices.rs` : conventions entité/repo + `20260628000001_supplier_invoices.sql` (multi-tenant `company_id`, CHECK texte, `version`, `DATETIME(3)`) — **modèle pour la nouvelle migration/entité**.
- `crates/kesh-qrbill` (12-5a, `85a235b`) : `parse_spc_payload`, `ScannedQrBill`/`ScannedAddress`/`ScannedReference` exportés `lib.rs`. **Réutiliser** (le décodage 12-5b appelle `parse_spc_payload`).
- `pdfium-binaries` : https://github.com/bblanchon/pdfium-binaries — binaire natif `libpdfium.so`, licence Apache-2.0/BSD-3.

### Conventions
- **Migration policy** : `CREATE TABLE` = non-breaking (pas de bump min_required) ; audit idempotence + compteurs obligatoires (CLAUDE.md).
- **SQLx MySQL** : pas d'enum SQLx, statut via `CHECK` texte (cf. `feedback_sqlx_mysql_gotchas`). `DATETIME(3)` + `%.f` à la sérialisation si exposé.
- **Multi-tenant** : tout finder scopé `company_id` (défense IDOR, KF-002).
- **Anti-traversal** : nom archivé dérivé du hash, jamais du nom d'origine (déjà au niveau du helper de stockage 12-5b).
- **Test Locally First** exit code vérifié (PAS `cargo test | grep`, cf. `feedback_cargo_test_pipe_masks_exit`). Ajout de table → maj compteur `admin_full_export_e2e` `data_count`.
- **Branche** : `story/12-5-import-repertoire-factures` (umbrella).
- **Dérogation/split (F8)** : T2 (pdfium/Docker) et T3 (migration/entité/stockage) ont des modes d'échec disjoints. Si la passe de code-review devient hétérogène, split possible 12-5b1 (T3) / 12-5b2 (T2). Sinon gardés ensemble (assumé).

### Limitations (héritées umbrella)
- **L1** — binaires justificatifs hors `.keshbackup` (métadonnées seules, DC5). Remédiation Epic 14.
- **L3** — amd64 uniquement (pas d'image arm64). Follow-up.
- **L6** — pdfium natif in-process non sandboxé ; segfault PDF malformé tue le process (caps pages/dimensions en atténuation). Risque accepté v0.4.

### References
- [Source: 12-5-import-repertoire-factures.md] — AC2/AC5, §Schéma, DC1/DC1-bis/DC3/DC4/DC5, Dev Notes ground-truth, Découpage (F8).
- [Source: Cargo.toml kesh-api/kesh-qrbill] — deps à ajouter. [Dockerfile] — runtime pdfium. [config.rs:896] — pattern env dir.
- [Source: backup.rs:34/432/570] — TABLES_TO_TRUNCATE + restore + test inventaire.
- [Source: supplier_invoices.sql + entity/repo 12-2] — modèle migration/entité.

## Dev Agent Record

### Agent Model Used

### Debug Log References

### Completion Notes List

### File List
