# Story 20.3a: Service PDF de facture factorisé

Status: ready-for-dev

<!-- Note: Validation is optional. Run validate-create-story for quality check before dev-story. -->

## Story

En tant que développeur du backend Kesh,
je veux extraire la génération du PDF QR-facture (chargement DB + validations + mapping + génération) du handler HTTP `get_invoice_pdf` vers un **service réutilisable** `invoice_pdf_service::render(pool, i18n, locale, company_id, invoice_id)`,
afin que la Story 20-3b (envoi de facture par e-mail) puisse produire exactement le même PDF pour la pièce jointe, sans dupliquer les 5 chargements DB, les 4 validations et le mapping `kesh-qrbill`.

**Story mécanique** (refactor sans changement de comportement fonctionnel), revue file-by-file. Aucun changement d'API HTTP, aucune migration, aucun changement frontend. Le endpoint `GET /api/v1/invoices/{id}/pdf` doit rester **strictement iso-comportement** (mêmes réponses, mêmes codes d'erreur, même Content-Disposition).

## Acceptance Criteria

**Service factorisé**

1. Nouveau module `crates/kesh-api/src/routes/invoice_pdf_service.rs` (déclaré `pub mod invoice_pdf_service;` dans `routes/mod.rs`) exposant :
   ```rust
   pub struct RenderedInvoicePdf {
       pub bytes: Vec<u8>,
       /// Nom de fichier déjà sanitizé, sans extension host — ex. "F-2026-0042"
       /// (le caller ajoute le préfixe/extension de son Content-Disposition ou
       /// de son attachment). Dérivé de `invoice.invoice_number` via `sanitize_filename`.
       pub filename_base: String,
   }

   pub async fn render(
       pool: &sqlx::MySqlPool,
       i18n: &kesh_i18n::I18nBundle,
       locale: kesh_i18n::Locale,
       company_id: i64,
       invoice_id: i64,
   ) -> Result<RenderedInvoicePdf, AppError>;
   ```
2. `render` reproduit **exactement** la séquence actuelle de `get_invoice_pdf` (lignes 57-114 de `invoice_pdf.rs`), dans cet ordre :
   - `invoices::find_by_id_with_lines(pool, company_id, invoice_id)` → `AppError::Database(DbError::NotFound)` si absent (**scoping company préservé** — anti-IDOR).
   - `invoice.status != "validated"` → `AppError::InvoiceNotValidated`.
   - `lines.len() > MAX_LINES_PER_PDF` → `AppError::InvoiceTooManyLinesForPdf(lines.len())`.
   - `contacts::find_by_id(pool, invoice.contact_id)` → `AppError::InvoiceNotPdfReady(t("invoice-pdf-error-contact-missing", …))` si absent.
   - `bank_accounts::find_primary(pool, company_id)` → `AppError::InvoiceNotPdfReady(t("invoice-pdf-error-no-primary-bank", …))` si absent.
   - `fetch_country(pool, "companies", company_id)` + `fetch_country(pool, "contacts", contact.id)`.
   - `build_qrbill_inputs(...)` (les 4 validations d'adresse/IBAN qu'il porte restent identiques).
   - `build_i18n(i18n, locale)`.
   - `kesh_qrbill::generate_qr_bill_pdf(...).map_err(map_qrbill_error)`.
   - `filename_base = sanitize_filename(invoice.invoice_number.as_deref().unwrap_or("facture"))`.
3. **La `locale` est un paramètre** (plus `state.config.locale` en dur) : `get_invoice_pdf` passe `state.config.locale` (iso-comportement) ; la Story 20-3b passera la langue du contact. Aucune régression de langue pour le endpoint existant.

**Handler thin wrapper**

4. `get_invoice_pdf` (handler HTTP, reste dans `invoice_pdf.rs`) devient un thin wrapper : `get_company_for` + `tracing::info!` (conservé) + appel `invoice_pdf_service::render(&state.pool, &state.i18n, state.config.locale, company.id, id)` + construction de la `Response` (headers `Content-Type: application/pdf` + `Content-Disposition: inline; filename="facture-{filename_base}.pdf"`). Le format exact du Content-Disposition (`inline; filename="facture-<base>.pdf"`) est **inchangé**.
5. Toutes les erreurs remontent inchangées (mêmes variantes `AppError`, mêmes codes HTTP). Le mapping `map_qrbill_error` est réutilisé tel quel.

**Helpers partagés — préserver les call-sites existants**

6. Les helpers actuellement dans `invoice_pdf.rs` et **importés ailleurs** — `MAX_LINES_PER_PDF`, `build_i18n`, `map_qrbill_error`, `sanitize_filename`, `split_lines` — **déménagent dans `invoice_pdf_service.rs`** (logique PDF partagée) et y restent `pub(crate)`/`pub const`. Les 3 call-sites externes sont mis à jour :
   - `crates/kesh-api/src/routes/credit_notes.rs:26-27` : `use crate::routes::invoice_pdf::{MAX_LINES_PER_PDF, build_i18n, map_qrbill_error, sanitize_filename, split_lines};` → `use crate::routes::invoice_pdf_service::{...};`.
   - `crates/kesh-api/src/errors.rs:955` : `crate::routes::invoice_pdf::MAX_LINES_PER_PDF` → `crate::routes::invoice_pdf_service::MAX_LINES_PER_PDF`.
   - Doc-comment `errors.rs:210` (`routes::invoice_pdf::MAX_LINES_PER_PDF`) : mettre à jour le chemin cité.
7. `build_qrbill_inputs` (privé, utilisé seulement par `render`) et `fetch_country` (utilisé seulement par `render`) déménagent aussi dans `invoice_pdf_service.rs`. `build_qrbill_inputs` peut rester privé (`fn`) au module service ; `fetch_country` reste `pub(crate)` (aucun autre call-site actuel mais utilitaire).

**Iso-comportement vérifié**

8. Le test E2E existant `crates/kesh-api/tests/invoice_pdf_e2e.rs` passe **inchangé** (aucune modification de test nécessaire) — c'est la preuve d'iso-comportement du endpoint. Le test unitaire `sanitize_filename_replaces_non_alphanumeric` déménage avec `sanitize_filename` dans `invoice_pdf_service.rs` (ou reste testé là où la fonction vit).
9. Aucune régression sur les avoirs : le PDF d'avoir (`credit_notes.rs`, `generate_credit_note_pdf`) continue d'utiliser `build_i18n`/`map_qrbill_error`/`sanitize_filename`/`split_lines`/`MAX_LINES_PER_PDF` depuis leur nouvel emplacement, comportement identique.

## Tasks / Subtasks

- [ ] **T1 — Créer le module service** (AC: #1, #2, #3, #6, #7)
  - [ ] T1.1 `crates/kesh-api/src/routes/invoice_pdf_service.rs` + `pub mod invoice_pdf_service;` dans `routes/mod.rs`
  - [ ] T1.2 Déplacer dans le service : `MAX_LINES_PER_PDF`, `build_qrbill_inputs`, `build_i18n`, `map_qrbill_error`, `sanitize_filename`, `split_lines`, `fetch_country` (+ le test unitaire `sanitize_filename_replaces_non_alphanumeric`)
  - [ ] T1.3 Écrire `render(pool, i18n, locale, company_id, invoice_id) -> Result<RenderedInvoicePdf, AppError>` reproduisant la séquence exacte (AC #2), `locale` en paramètre (AC #3)
  - [ ] T1.4 Définir `pub struct RenderedInvoicePdf { bytes, filename_base }`

- [ ] **T2 — Réduire le handler à un wrapper** (AC: #4, #5)
  - [ ] T2.1 `get_invoice_pdf` : `get_company_for` + `tracing::info!` + `render(...)` + Response (headers identiques)
  - [ ] T2.2 Retirer de `invoice_pdf.rs` tout ce qui a déménagé ; importer depuis `invoice_pdf_service` ce que le handler référence encore (aucun si tout est dans render, sinon `RenderedInvoicePdf`)

- [ ] **T3 — Mettre à jour les call-sites externes** (AC: #6, #9)
  - [ ] T3.1 `credit_notes.rs` imports → `invoice_pdf_service`
  - [ ] T3.2 `errors.rs:955` chemin `MAX_LINES_PER_PDF` + doc-comment `:210`

- [ ] **T4 — Test Locally First & commit** (AC: #8)
  - [ ] T4.1 `cargo fmt --all -- --check`, `cargo build --workspace --all-targets`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace` (kesh-api en série `--test-threads=1` recommandé si les tests d'intégration DB touchés) — le test `invoice_pdf_e2e` DOIT passer inchangé (iso-comportement)
  - [ ] T4.2 Commit sur `story/20-1-envoi-factures-email`

## Dev Notes

### Nature de la story — refactor mécanique, zéro changement fonctionnel

C'est une extraction pure. Le critère de succès : **le comportement observable du endpoint `GET /api/v1/invoices/{id}/pdf` est identique bit-pour-bit** (mêmes bytes PDF, mêmes erreurs, même header). Le test E2E `invoice_pdf_e2e.rs` non modifié qui reste vert est la preuve. Ne PAS en profiter pour « améliorer » la logique (pas de refactor de `build_qrbill_inputs`, pas de changement des messages d'erreur, pas de nouveau validation).

### Refinement du contrat planning (`Vec<u8>` → struct)

Le planning epic-20 §14 esquisse la signature `(company_id, invoice_id, locale) → Result<Vec<u8>, AppError>`. **Refinement assumé** : `render` retourne `RenderedInvoicePdf { bytes, filename_base }` plutôt que `Vec<u8>` nu. Raison : le handler (Content-Disposition) ET la Story 20-3b (nom de la pièce jointe e-mail) ont tous deux besoin du nom de fichier dérivé de `invoice.invoice_number`. Re-charger la facture juste pour le numéro serait un gaspillage ; le service a déjà l'invoice en main, il retourne donc le `filename_base` sanitizé avec les bytes. `locale` reste un paramètre d'entrée (comme le planning le demande explicitement).

### État actuel de `invoice_pdf.rs` (fichier à refactorer — lire intégralement avant de commencer)

- `get_invoice_pdf` (L44-127) : handler actuel = chargement + validations + mapping + generate + Response. C'est ce corps qui se scinde entre `render` (tout sauf la Response) et le wrapper (Response).
- `build_qrbill_inputs` (L130-245, privé) : mapping entités→`QrBillData`/`InvoicePdfData` + 4 validations (adresse créancier NPA/ville, adresse débiteur requise+complète, QR-IBAN→QRR, IBAN simple). **Ne pas toucher la logique.**
- `split_lines` (L249, `pub(crate)`) — utilisé par credit_notes.
- `build_i18n` (L258, `pub(crate)`) — utilisé par credit_notes.
- `map_qrbill_error` (L270, `pub(crate)`) — utilisé par credit_notes.
- `sanitize_filename` (L297, `pub(crate)`) — utilisé par credit_notes + le handler.
- `fetch_country` (L316, `pub(crate)`) — utilisé seulement par le handler ; note SQL anti-injection (littéral validé "companies"/"contacts") à préserver telle quelle.
- `MAX_LINES_PER_PDF` (L41, `pub const`) — utilisé par credit_notes + errors.rs. Doc-comment géométrique important à conserver verbatim.
- test `sanitize_filename_replaces_non_alphanumeric` (L343).

### Couplage existant à préserver (ne PAS casser)

`grep` confirme 3 dépendances externes aux helpers, toutes à re-router vers `invoice_pdf_service` :
- `credit_notes.rs:26-27` (import de 5 symboles).
- `errors.rs:955` (`MAX_LINES_PER_PDF` dans le mapping de `InvoiceTooManyLinesForPdf`).
- `errors.rs:210` (doc-comment citant le chemin).
La compilation `cargo build --workspace` échouera immédiatement si un chemin est oublié — c'est le filet de sécurité mécanique de cette story.

### Signatures repository (déjà en place, ne rien changer)

- `invoices::find_by_id_with_lines(pool, company_id, id) -> Result<Option<(Invoice, Vec<InvoiceLine>)>, DbError>` (`repositories/invoices.rs:432`).
- `bank_accounts::find_primary(pool, company_id) -> Result<Option<BankAccount>, DbError>` (`repositories/bank_accounts.rs:72`).
- `contacts::find_by_id(pool, id) -> Result<Option<Contact>, DbError>`.

### Pourquoi un module `invoice_pdf_service` (et pas une fonction dans `invoice_pdf.rs`)

Le planning nomme explicitement `invoice_pdf_service::render`. Au-delà du nom : (1) 20-3b consommera `invoice_pdf_service::render` — un import depuis un module « service » lit mieux que depuis un module route ; (2) `credit_notes.rs` couple déjà à la logique PDF de `invoice_pdf` — déplacer les helpers partagés dans le service assainit ce couplage (route→service au lieu de route→route) ; (3) sépare la logique réutilisable (service) de la couche HTTP (handler). Le déplacement des 5 helpers + mise à jour de 3 imports est mécanique et file-by-file reviewable.

### Frontières de scope

- **Aucune migration, aucun frontend, aucun nouveau endpoint.** Story purement backend-refactor.
- **Ne PAS** implémenter l'envoi e-mail, le champ `contacts.language`, ni quoi que ce soit de 20-3b. La `locale` paramétrable est le seul « hook » posé pour 20-3b.
- **Ne PAS** modifier `kesh-qrbill` (crate pur, `generate_qr_bill_pdf` inchangé).

### Testing standards summary

- **Iso-comportement** : `crates/kesh-api/tests/invoice_pdf_e2e.rs` reste **inchangé** et vert — c'est le test de non-régression central. Ne pas le réécrire.
- **Test unitaire** : `sanitize_filename_replaces_non_alphanumeric` suit `sanitize_filename` dans le service.
- **Optionnel (nice-to-have, non bloquant)** : un test unitaire léger sur `render` nécessiterait un pool DB (`#[sqlx::test]`) — l'E2E existant couvre déjà le chemin complet via HTTP, donc pas de nouveau test d'intégration requis pour cette story mécanique.
- **Test Locally First (CLAUDE.md)** : 4 checks backend. `cargo build --workspace` est le garde-fou principal (tout import oublié casse la compilation). Story 100% Rust → pas de check frontend.

### References

- [Source: `_bmad-output/planning-artifacts/epic-20-envoi-factures-email.md`] — décision #14 (service factorisé), découpage 20-3a, §contexte technique PDF (`generate_qr_bill_pdf` pur, handler L44-265).
- [Source: `crates/kesh-api/src/routes/invoice_pdf.rs`] — fichier à refactorer (lu intégralement, cf. Dev Notes).
- [Source: `crates/kesh-api/src/routes/credit_notes.rs:26-27,195,246,249,257,272,274`] — call-sites des helpers partagés à re-router.
- [Source: `crates/kesh-api/src/errors.rs:210,955`] — `MAX_LINES_PER_PDF` call-site + doc.
- [Source: `crates/kesh-api/tests/invoice_pdf_e2e.rs`] — test d'iso-comportement à garder vert inchangé.
- [Source: `crates/kesh-api/src/lib.rs:493-494`] — enregistrement route (inchangé).
- [Source: CLAUDE.md §Test Locally First] — 4 checks backend.

## Dev Agent Record

### Agent Model Used

### Debug Log References

### Completion Notes List

### File List
