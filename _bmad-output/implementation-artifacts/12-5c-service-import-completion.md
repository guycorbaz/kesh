# Story 12.5c: Service d'import + endpoints + complétion atomique

Status: ready-for-dev

<!-- Sous-story 3/4 de l'umbrella 12-5 (validate convergé 6 passes). Périmètre T4 : service de lecture inbox + sécurité + verrou de run + rapport batch FailedProposal + refactor DC6 `create_in_tx` + endpoints liste/complétion/écarter/download. Dépend de 12-5a (parse_spc_payload) ET 12-5b (qr_decode, document_storage, entité/repo imported_supplier_invoices, KESH_DOCUMENTS_DIR). NE COUVRE PAS le frontend ni la doc (12-5d). Absorbe les 2 dettes déférées en code-review 12-5b (validation status HTTP, map_db_error 1406/1264). -->

## Story

As a comptable PME utilisant Kesh sur mon NAS,
I want déclencher l'import du dossier de factures (lecture inbox → décodage QR → staging + archivage du fichier), puis **compléter** chaque facture importée (fournisseur, date, lignes) de façon **atomique et sûre**,
so that les factures fournisseurs entrent dans la comptabilité sans ressaisie des coordonnées de paiement, sans double écriture comptable, et avec leur justificatif récupérable.

## Contexte & source

- Sous-story **3/4** de l'umbrella **12-5** (import répertoire factures, issue **#194**, cible v0.4). Cf. `12-5-import-repertoire-factures.md` (AC2/AC3/AC4/AC5/AC6/AC7/AC10, DC6, schéma, Dev Notes ground-truth).
- **Dépend de 12-5a** (`parse_spc_payload`, `ScannedQrBill` dans `kesh-qrbill`, commit `85a235b`) **et 12-5b** (décodage + entité + stockage, mergé sur la branche umbrella) : ce module **consomme** `qr_decode::{decode_spc_from_image_bytes, decode_spc_from_pdf_bytes}`, `document_storage::{store_document, read_document}`, le repository `imported_supplier_invoices` et le mapping `NewImportedSupplierInvoice::from_scanned`.
- **Débloque 12-5d** (frontend + doc) qui **consomme** les endpoints livrés ici (aucune logique transactionnelle en frontend).
- **Absorbe 2 dettes déférées de la code-review 12-5b** (cf. §Dettes héritées 12-5b).

## Périmètre (et hors-périmètre)

**DANS 12-5c (T4)** :
- **Config** : `KESH_INBOX_DIR` (défaut `/data/inbox`) + caps `KESH_INBOX_MAX_FILE_BYTES` (défaut 25 Mo), `KESH_INBOX_MAX_FILES_PER_RUN` (défaut 200), `KESH_INBOX_MAX_PDF_PAGES` (défaut 20, câblé à `DecodeConfig.max_pages` 12-5b).
- **Service d'import** : lecture inbox sur déclenchement manuel (DC2) + **verrou de run** (sérialisation, F6) + court-circuit doublon hash (réactive `discarded`, F5) + boucle décodage (12-5b) + création staging + archivage fichier (12-5b) + suppression inbox au succès / déplacement `failed/` à l'échec + rapport batch `{accepted, failed, warnings}` (HTTP 200, pattern FailedProposal).
- **Sécurité d'ingestion** (AC4) : symlink rejeté, anti-traversal (canonicalisation sous racine), liste blanche d'extensions + **magic bytes**, taille max, stabilité fichier (en cours d'écriture), caps pdfium.
- **Refactor DC6** : extraire `supplier_invoices::create_in_tx(&mut tx, …)` (miroir de `pay_in_tx`) sans changer le comportement de `create` (pool-owning) existant.
- **Endpoints** (Comptable+, company-scopés) : `POST .../inbox-import`, `GET .../imported-supplier-invoices?status=`, `POST .../imported-supplier-invoices/{id}/complete` (siège DC6 + réconciliation F2 + routage IBAN F9 + validation devise), `POST .../imported-supplier-invoices/{id}/discard`, `GET .../imported-supplier-invoices/{id}/source-document`, `GET .../supplier-invoices/{id}/source-document`.
- **Dettes 12-5b absorbées** : validation `status` à la frontière HTTP (liste) ; mapping `map_db_error` des codes 1406/1264 → variant typé pour échec par-fichier propre.
- **Compteurs/tests** : aucune nouvelle table (la table est livrée 12-5b) ; tests d'intégration import + complétion + sécurité + download.

**HORS 12-5c** (→ 12-5d) :
- Tout le frontend (écran import, rapport UI, liste/complétion/écarter UI, lien « Voir la facture d'origine »).
- Doc utilisateur/admin, `.env.example`, `docker-compose.yml` volumes, CHANGELOG, README (12-5d, sauf si une env var introduite ici impose une note minimale — alors la note minimale `.env.example` est tolérée ici).
- E2E Playwright (12-5d, après l'UI).

## Acceptance Criteria

### Service d'import & rapport batch

1. **Config inbox** : `KESH_INBOX_DIR` (pattern `from_env` comme `KESH_DOCUMENTS_DIR` 12-5b / `KESH_ADMIN_BACKUP_DIR`, défaut **`/data/inbox`**) + `KESH_INBOX_MAX_FILE_BYTES` (défaut 25*1024*1024), `KESH_INBOX_MAX_FILES_PER_RUN` (défaut 200), `KESH_INBOX_MAX_PDF_PAGES` (défaut 20). Champs ajoutés à `Config`, lus dans `from_env`. Le wiring `KESH_INBOX_MAX_PDF_PAGES` → `DecodeConfig { max_pages, .. }` (12-5b) est fait **ici** (AC3/L6 12-5b délégait ce câblage à 12-5c).

2. **`POST /api/v1/inbox-import`** (Comptable+) : lit `KESH_INBOX_DIR`, traite chaque entrée, retourne **HTTP 200** avec body `{ accepted: [...], failed: [...], warnings: [...] }` (pattern FailedProposal CLAUDE.md — **aucune** erreur per-fichier n'escalade en `AppError` global ; exceptions globales = 401/403 RBAC, 409 run déjà en cours, 500 catastrophe DB/IO).
   - `accepted[]` : `{ imported_supplier_invoice_id: i64, file_name: String }` (staging créé).
   - `failed[]` : `{ file_name: String, error_code: String (constante), details: Option<serde_json::Value> }` — **identifiant business = `file_name`**, JAMAIS d'index positionnel ; `error_code` = constante canonique (jamais `format!`).
   - `warnings: Vec<String>` : avertissements non liés à un fichier précis (troncature `MAX_FILES_PER_RUN`, fichiers instables ignorés ce tour).
   - **Catalogue `error_code`** (constantes) : `UNSUPPORTED_FILE_TYPE`, `FILE_TOO_LARGE`, `SYMLINK_REJECTED`, `DUPLICATE`, `NO_QR_CODE_FOUND`, `INVALID_SPC_PAYLOAD`, `INVALID_IBAN`, `PDF_RENDER_ERROR`, `FILE_READ_ERROR`. Mapping `DecodeError` (12-5b) → `error_code` : `ImageDecode`→`FILE_READ_ERROR` ou `UNSUPPORTED_FILE_TYPE` (selon contexte), `PdfRender`→`PDF_RENDER_ERROR`, `InvalidSpcPayload`→`INVALID_SPC_PAYLOAD`, `InvalidIban`→`INVALID_IBAN` ; absence de QR SPC (`Ok(None)`)→`NO_QR_CODE_FOUND`.

3. **Boucle de traitement par fichier** : pour chaque entrée d'inbox (jusqu'à `MAX_FILES_PER_RUN`) :
   1. **Symlink** : `symlink_metadata().is_symlink()` → `SYMLINK_REJECTED` (jamais suivi/ouvert), déplacé `failed/`.
   2. **Taille** : > `MAX_FILE_BYTES` → `FILE_TOO_LARGE`.
   3. **Stabilité** : taille + mtime identiques sur 2 lectures espacées d'un court délai ; si instable → **ignoré ce tour** (ni `accepted` ni `failed`, warning), retenté au prochain déclenchement. Lecture impossible → `FILE_READ_ERROR`.
   4. **Type** : extension ∈ liste blanche (`pdf`,`png`,`jpg`,`jpeg`) **ET** magic bytes cohérents (PDF `%PDF`, PNG/JPEG signatures) ; sinon `UNSUPPORTED_FILE_TYPE`.
   5. **Hash SHA-256** calculé **avant** décodage → court-circuit doublon (AC6) : si `(company_id, file_hash)` matche une row `to_complete`/`completed` → `DUPLICATE` (déplacé `failed/`) ; si matche une row `discarded` → **réactivation `to_complete`** (F5, `accepted`, fichier inbox supprimé).
   6. **Décodage** (12-5b) : PDF → `decode_spc_from_pdf_bytes(bytes, DecodeConfig{ max_pages: KESH_INBOX_MAX_PDF_PAGES, .. })` ; image → `decode_spc_from_image_bytes(bytes)`. `Ok(Some)` → suite ; `Ok(None)` → `NO_QR_CODE_FOUND` ; `Err(DecodeError)` → `error_code` mappé.
   7. **Archivage** : `store_document(documents_dir, bytes, ext, original_filename, mime)` (12-5b) → `DocumentMeta`.
   8. **Staging** : `NewImportedSupplierInvoice::from_scanned(company_id, &scanned, doc)` → `imported_supplier_invoices::create`. Violation `UNIQUE (company_id, file_hash)` (race) → `DUPLICATE` per-fichier (HTTP 200, **PAS** d'`AppError` 500).
   9. **Succès** → suppression du fichier inbox (tolérer `ENOENT`, idempotent). **Échec** → déplacement `failed/` (anti-traversal sur le nom).

4. **Sécurité** (AC4 umbrella) : chemins via env (admin) ; canonicalisation `KESH_INBOX_DIR`/`failed/`/`KESH_DOCUMENTS_DIR` (chemin résolu reste sous la racine) ; jamais d'écriture hors `KESH_DOCUMENTS_DIR`/`failed/` ; nom archivé = `{sha256hex}.{ext}` (12-5b, jamais le nom d'origine). **Caps pdfium** (12-5b expose `DecodeConfig`) : `max_pages` câblé ici ; **L6** (segfault natif tue le process) acceptée v0.4.

5. **Verrou de run (F6)** : deux déclenchements concurrents (2 onglets / double-clic) → sérialisés. Implémentation : `GET_LOCK`/`RELEASE_LOCK` MariaDB **OU** flag global `AtomicBool` (documenter le choix). Si un import est déjà en cours → **HTTP 409** (ou warning + no-op selon choix documenté), pas de double traitement.

### Refactor DC6 & complétion atomique

6. **Refactor `supplier_invoices::create_in_tx`** : extraire le cœur de `create` (`repositories/supplier_invoices.rs:212-406`, qui `pool.begin()` l.239 / `tx.commit()` l.406) en **`create_in_tx(tx: &mut Transaction, company_id, user_id, new: NewSupplierInvoice) -> Result<SupplierInvoiceResponse, DbError>`** — **exactement** le pattern `pay`/`pay_in_tx` (l.457/490). `create` (pool-owning) **conserve son comportement** (appelle `create_in_tx` puis `commit`, comme `pay` appelle `pay_in_tx`). Aucune régression des tests 12-2.

7. **`POST /api/v1/imported-supplier-invoices/{id}/complete`** (Comptable+, company-scopé) — corps JSON camelCase :
   ```json
   { "contactId": i64, "invoiceDate": "YYYY-MM-DD",
     "supplierInvoiceNumber": "string|null", "dueDate": "YYYY-MM-DD|null",
     "lines": [{ "description": "string", "quantity": "Decimal",
                 "unitPrice": "Decimal (HT)", "vatRate": "Decimal", "expenseAccountId": i64 }] }
   ```
   **Complétion atomique (DC6)** dans **une seule transaction** :
   1. `SELECT … FROM imported_supplier_invoices WHERE id=? AND company_id=? FOR UPDATE` (garde anti-double + anti-IDOR).
   2. Si `status != 'to_complete'` → abandon (409/erreur métier explicite, pas d'écriture).
   3. **Validation devise (P4-M4)** : `staging.currency ∈ {CHF, EUR}` sinon erreur « Devise non supportée (CHF ou EUR) ». **NE PAS** ajouter de champ `currency` à `NewSupplierInvoice`/migration 12-2 (L7).
   4. **Routage IBAN (F9)** : `staging.is_qr_iban=true` → `NewSupplierInvoice.creditor_qr_iban` + **exiger `staging.reference_type='QRR'`** (sinon rejet, cohérent `payment_batches.rs:613`) ; `false` → `creditor_iban`.
   5. **Mapping `payment_reference` (C5-1)** : `staging.reference_value` si `reference_type ∈ {QRR,SCOR}`, sinon `None`.
   6. **Réconciliation montant (F2)** : si `staging.amount` présent → exiger `round2(Σ lignes TTC, HALF_UP) == round2(staging.amount, HALF_UP)` sinon bloquer « Le total des lignes (X) ne correspond pas au montant du QR (Y) ». Renseigner `NewSupplierInvoice.expected_payment_amount = staging.amount`. Si `staging.amount` absent → pas de contrôle.
   7. `create_in_tx(&mut tx, …)` (AC6) → `UPDATE imported_supplier_invoices SET status='completed', supplier_invoice_id=? WHERE id=?` → `commit`.
   - Retourne **`SupplierInvoiceResponse`** (identique à `POST /api/v1/supplier-invoices`).
   - **Échec de `create_in_tx`** : rollback → staging **reste `to_complete`**, message UX explicite, **aucune écriture comptable partielle**. `DbError::FiscalYearInvalid` → « Aucun exercice fiscal ouvert couvrant cette date » ; montant ≤ 0 → « Le montant doit être positif ».

8. **`POST /api/v1/imported-supplier-invoices/{id}/discard`** (Comptable+, company-scopé) : transition `to_complete` → `discarded` (FOR UPDATE, garde statut). Le fichier archivé est **conservé** (pas de suppression du justificatif v0.4).

### Endpoints liste & download

9. **`GET /api/v1/imported-supplier-invoices?status={to_complete|completed|discarded}`** (Comptable+, company-scopé) : liste filtrée par statut (défaut/use-case principal `to_complete`). **Validation du `status`** à la frontière HTTP (dette 12-5b absorbée) : un `status` hors-domaine → **400 Bad Request** (`AppError::Validation` ou équivalent), PAS une liste vide silencieuse. Consomme `repositories::imported_supplier_invoices::list_by_status` (scopé `company_id`).

10. **Download du justificatif** (Comptable+, company-scopé, anti-IDOR `WHERE company_id = {current}`) :
    - `GET /api/v1/imported-supplier-invoices/{id}/source-document` — avant complétion (résout par `id` du staging).
    - `GET /api/v1/supplier-invoices/{id}/source-document` — après complétion (résout `imported_supplier_invoices WHERE supplier_invoice_id = {id} AND company_id = {current}`).
    - Lit via `document_storage::read_document(documents_dir, storage_path)` (12-5b). Headers `Content-Type` (= `mime_type` stocké) + `Content-Disposition: attachment; filename="{original_filename}"`.
    - **`ReadDocumentError::NotFound` / row absente → 404** (ou 410 Gone) « justificatif non disponible » (F7/L1/L5), **JAMAIS 500**. `ReadDocumentError::InvalidPath` → 500 (corruption interne, ne devrait pas arriver). Test dédié (métadonnée sans fichier disque → 404, pas 500).

### Qualité

11. **Tests** (intégration `#[sqlx::test]` + service) : import (dossier temp avec fixtures PNG/PDF → staging + fichiers archivés + rapport ; échecs `failed[]` par catégorie) ; sécurité (symlink, traversal, type refusé/magic-bytes, doublon hash scopé company, réactivation `discarded`, taille max) ; verrou de run (2e appel concurrent → 409/no-op) ; **complétion atomique** (`to_complete`→`completed` + facture `open` + `supplier_invoice_id` lié ; échec exercice fermé → reste `to_complete`, pas d'écriture ; réconciliation montant KO → bloqué ; routage QR-IBAN⇒QRR ; devise non supportée) ; `discard` ; download (404 fichier absent, anti-IDOR cross-company, Content-Disposition). Refactor `create_in_tx` : 0 régression des tests 12-2.

12. **Quality gate Test Locally First — exit code vérifié** (PAS `cargo test | grep`) : `cargo fmt --all --check` + `cargo clippy --workspace --all-targets -- -D warnings` + `cargo build --workspace --all-targets` + `cargo test --workspace -j1 -- --test-threads=1`. 0 régression.

## Dettes héritées 12-5b (à absorber ici)

- **D1 — validation `status` (list_by_status)** : le repository `list_by_status` (12-5b) ne valide pas le `status` (retourne liste vide sur valeur hors-domaine). **Ici (AC9)** : valider le query param à la frontière HTTP → 400 sur valeur invalide. (Contrat documenté en doc de `list_by_status` 12-5b.)
- **D2 — `map_db_error` 1406/1264** : un champ QR tiers sur-long (nom > 70, message > 140, etc.) fait échouer l'INSERT staging avec un `DbError` générique opaque (codes MariaDB **1406** Data too long, **1264** Out of range non gérés par `map_db_error`, `errors.rs:151`). **Ici** : ajouter un variant typé (ex. `DbError::DataLengthOrRange(String)`) mappé depuis 1406/1264, exploité par le service d'import pour un **échec par-fichier propre** (`failed[]` avec un `error_code` dédié, ex. `INVALID_SPC_PAYLOAD` ou un nouveau `FIELD_TOO_LONG`) au lieu d'un 500 global. **Garde-fou** : vérifier qu'aucun test existant n'assume ces codes en `DbError::Other`.

## DC6 — Complétion atomique (rappel umbrella, figé)

`repositories::supplier_invoices::create` **possède** le pool et **commit sa propre transaction** → une complétion naïve `create()` (COMMIT) puis `UPDATE staging` laisse une fenêtre non-atomique (crash entre les deux → double facture + double écriture comptable à la re-complétion). **Solution figée** : `create_in_tx(&mut tx, …)` (refactor, AC6) exécuté dans la même transaction que le `SELECT … FOR UPDATE` du staging + l'`UPDATE status='completed'`. Atomique : pas de double facture possible. `create` (pool-owning) reste pour les appels directs 12-2.

## Tasks / Subtasks

- [ ] **T4.1 — Config inbox** : `KESH_INBOX_DIR` (`/data/inbox`) + `KESH_INBOX_MAX_FILE_BYTES`/`MAX_FILES_PER_RUN`/`MAX_PDF_PAGES` dans `Config` + `from_env` (pattern `documents_dir` 12-5b). Câbler `MAX_PDF_PAGES` → `DecodeConfig.max_pages`.
- [ ] **T4.2 — Refactor DC6** : extraire `supplier_invoices::create_in_tx(&mut tx, …)` (miroir `pay_in_tx`), `create` délègue + commit. 0 régression tests 12-2.
- [ ] **T4.3 — Dette D2** : `DbError` variant pour MariaDB 1406/1264 + mapping `map_db_error` ; vérifier non-régression.
- [ ] **T4.4 — Service d'import** : module `inbox_import` (kesh-api) — lecture inbox, verrou run, sécurité (symlink/taille/stabilité/type+magic), hash+doublon (réactive `discarded`), décodage 12-5b, archivage 12-5b, staging, suppression/`failed/`, rapport `{accepted, failed, warnings}`. Constantes `error_code`.
- [ ] **T4.5 — Endpoints** : `routes/imported_supplier_invoices.rs` (nouveau) — `POST /inbox-import`, `GET /imported-supplier-invoices?status=` (valide status, dette D1), `POST /{id}/complete` (DC6 + F2 + F9 + devise), `POST /{id}/discard`, `GET /{id}/source-document` ; + `GET /supplier-invoices/{id}/source-document` (routes/supplier_invoices.rs). Enregistrer dans `routes/mod.rs`. RBAC Comptable+ + company-scope partout.
- [ ] **T4.6 — Tests** : intégration import + sécurité + verrou + complétion atomique + réconciliation + download (404/IDOR) + non-régression `create_in_tx`. Quality gate exit-code vérifié.

## Dev Notes

### Ground-truth (vérifié 2026-06-29)

**API 12-5b consommée** (déjà mergée sur la branche umbrella) :
- `crates/kesh-api/src/qr_decode.rs` : `decode_spc_from_image_bytes(bytes: &[u8]) -> Result<Option<ScannedQrBill>, DecodeError>` ; `decode_spc_from_pdf_bytes(bytes: &[u8], cfg: DecodeConfig) -> Result<…>` ; `DecodeConfig { max_pages, max_dimension }` (Default 20/2000) ; `DecodeError::{ImageDecode(String), PdfRender(String), InvalidSpcPayload(QrBillError), InvalidIban(String)}`. **Sérialisation pdfium déjà gérée** (`PDFIUM_LOCK` statique) — mais le **verrou de run** AC5 reste nécessaire (sémantique métier : un seul import à la fois, pas seulement un seul appel pdfium).
- `crates/kesh-api/src/document_storage.rs` : `store_document(documents_dir: &Path, bytes: &[u8], ext: &str, original_filename: &str, mime_type: &str) -> io::Result<DocumentMeta>` (refuse 0-octet + ext non sûre) ; `read_document(documents_dir, storage_path) -> Result<Vec<u8>, ReadDocumentError>` (`NotFound`/`InvalidPath`/`Io`) ; `mime_for_ext(ext) -> &'static str`.
- `crates/kesh-db/src/entities/imported_supplier_invoice.rs` : `NewImportedSupplierInvoice::from_scanned(company_id, &ScannedQrBill, DocumentMeta)` (mapping complet, `address_type` normalisé {K,S}) ; `DocumentMeta { storage_path, original_filename, sha256, mime_type, byte_size }`.
- `crates/kesh-db/src/repositories/imported_supplier_invoices.rs` : `create`, `find_by_id_scoped(company_id, id)`, `find_by_company_hash(company_id, file_hash)`, `list_by_status(company_id, status)` — tous scopés `company_id`. **`list_by_status` ne valide pas `status`** (dette D1 à traiter à la frontière HTTP).
- `crates/kesh-api/src/config.rs` : `documents_dir` (défaut `/data/documents`) ajouté 12-5b — reproduire le pattern pour `KESH_INBOX_DIR` (défaut `/data/inbox`).

**Cibles de refactor / extension** :
- `crates/kesh-db/src/repositories/supplier_invoices.rs:212-406` `create` (pool-owning, `pool.begin()` l.239, `tx.commit()` l.406) → extraire `create_in_tx`. **Patron exact** : `pay` (l.457) délègue à `pay_in_tx` (l.490). `find_open_covering_date(&mut tx, …)` l.296, `company_invoice_settings::get_or_create_default_in_tx` l.302 déjà tx-aware.
- `crates/kesh-db/src/errors.rs:151` `map_db_error` `match my_err.number()` gère 1062/1451/1452 → ajouter 1406/1264 (dette D2).
- `crates/kesh-api/src/routes/supplier_invoices.rs` : handlers `list/get/create/pay/cancel` (l.216/234/247/286/310) — **aucun handler complétion ni download** → ajouter `source-document` ici + nouveau module `imported_supplier_invoices`.
- `crates/kesh-api/src/routes/mod.rs:28` `pub mod supplier_invoices` → ajouter `pub mod imported_supplier_invoices` + nest la route.
- `crates/kesh-db/src/entities/supplier_invoice.rs` : `NewSupplierInvoice { company_id, contact_id, supplier_invoice_number, invoice_date: NaiveDate, due_date, creditor_iban, creditor_qr_iban, payment_reference, expected_payment_amount, lines: Vec<NewSupplierInvoiceLine> }`. `NewSupplierInvoiceLine { description (NOT NULL non-vide), quantity, unit_price, vat_rate, expense_account_id }`. **Aucun champ `currency`** (L7 — devise validée service-level).

### Conventions
- **Pattern batch FailedProposal** (CLAUDE.md) : `{accepted, failed, warnings}`, HTTP 200, `file_name` identifiant business, `error_code` constante (jamais `format!`), `details` JSON. Exceptions globales : 401/403 RBAC, 409 run en cours, 500 catastrophe.
- **Migration policy** : aucune nouvelle migration ici (table livrée 12-5b). Si un `DbError` variant est ajouté (D2), pas d'impact migration.
- **Sécurité filesystem** : env (admin), canonicalisation sous racine, symlink rejeté, liste blanche + magic bytes, taille max, stabilité fichier.
- **Multi-tenant** : tout finder/endpoint scopé `company_id` du user courant (anti-IDOR, KF-002).
- **Test Locally First** exit code vérifié (PAS `cargo test | grep`, cf. `feedback_cargo_test_pipe_masks_exit`).
- **Branche** : `story/12-5-import-repertoire-factures` (umbrella).
- **Contrat d'autonomie** : STOP si modification architecturale (le refactor `create_in_tx` est prévu par DC6 figé, donc dans le périmètre).

### Limitations (héritées umbrella)
- **L1** — binaires justificatifs hors `.keshbackup` (métadonnées seules, DC5). Le download après restore métadonnée-seule → 404 (F7). Epic 14.
- **L4** — création inline fournisseur hors scope (sélection contact `is_supplier` existant). UI 12-5d.
- **L5** — seules les factures importées via 12-5 ont un justificatif → `GET .../source-document` 404 pour une facture créée directement 12-2. Epic 14.
- **L6** — pdfium natif non sandboxé ; segfault PDF malformé tue le process (caps en atténuation). Accepté v0.4.
- **L7** — devise non persistée sur `supplier_invoices` (validée service-level CHF/EUR). Évolution 12-2 future.

### References
- [Source: 12-5-import-repertoire-factures.md] — AC2/AC3/AC4/AC5/AC6/AC7/AC10, DC6, schéma, Dev Notes, §Découpage.
- [Source: 12-5b-decodage-entite-stockage.md] — API qr_decode/document_storage/entité/repo consommée + dettes déférées D1/D2.
- [Source: supplier_invoices.rs:212/457/490] — `create` owns-tx, patron `pay_in_tx` pour `create_in_tx`. [errors.rs:151] — map_db_error.
- [Source: routes/supplier_invoices.rs / routes/mod.rs] — enregistrement endpoints.

## Change Log

## Dev Agent Record

### Agent Model Used

### Debug Log References

### Completion Notes List

### File List
