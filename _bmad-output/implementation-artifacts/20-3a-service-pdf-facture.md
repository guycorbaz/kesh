# Story 20.3a: Service PDF de facture factorisé

Status: done

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

- [x] **T1 — Créer le module service** (AC: #1, #2, #3, #6, #7)
  - [x] T1.1 `crates/kesh-api/src/routes/invoice_pdf_service.rs` + `pub mod invoice_pdf_service;` dans `routes/mod.rs`
  - [x] T1.2 Déplacer dans le service : `MAX_LINES_PER_PDF`, `build_qrbill_inputs`, `build_i18n`, `map_qrbill_error`, `sanitize_filename`, `split_lines`, `fetch_country` (+ le test unitaire `sanitize_filename_replaces_non_alphanumeric`)
  - [x] T1.3 Écrire `render(pool, i18n, locale, company_id, invoice_id) -> Result<RenderedInvoicePdf, AppError>` reproduisant la séquence exacte (AC #2), `locale` en paramètre (AC #3)
  - [x] T1.4 Définir `pub struct RenderedInvoicePdf { bytes, filename_base }`

- [x] **T2 — Réduire le handler à un wrapper** (AC: #4, #5)
  - [x] T2.1 `get_invoice_pdf` : `get_company_for` + `tracing::info!` + `render(...)` + Response (headers identiques)
  - [x] T2.2 Retirer de `invoice_pdf.rs` tout ce qui a déménagé ; importer depuis `invoice_pdf_service` ce que le handler référence encore (aucun si tout est dans render, sinon `RenderedInvoicePdf`)

- [x] **T3 — Mettre à jour les call-sites externes** (AC: #6, #9)
  - [x] T3.1 `credit_notes.rs` imports → `invoice_pdf_service`
  - [x] T3.2 `errors.rs:955` chemin `MAX_LINES_PER_PDF` + doc-comment `:210`

- [x] **T4 — Test Locally First & commit** (AC: #8)
  - [x] T4.1 `cargo fmt --all -- --check`, `cargo build --workspace --all-targets`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace` (kesh-api en série `--test-threads=1` recommandé si les tests d'intégration DB touchés) — le test `invoice_pdf_e2e` DOIT passer inchangé (iso-comportement)
  - [x] T4.2 Commit sur `story/20-1-envoi-factures-email`

### Review Findings

**Pass 1 (2026-07-09, Sonnet 5 × 3 reviewers : Blind Hunter + Edge Case Hunter + Acceptance Auditor)** — 8 findings bruts → dédup en 2 patch + 1 dismiss.

- [x] [Review][Patch] P1 (MEDIUM, blind+edge+auditor — fusion BH-1/BH-2/BH-3/ECH-1/ECH-2/AA-1) : reload `Company` dans `render` — requête SQL redondante + branche `NotFound` inatteignable (code mort) + fenêtre TOCTOU (snapshot company pris ~5 requêtes DB plus tard que dans le handler historique) + doc-comment « reproduit exactement » inexact + déviation AC #2. Fix racine : signature `render(pool, i18n, locale, company: &Company, invoice_id)` — supprime le reload, rend la séquence exactement identique à l'historique, explicite le contrat d'autorisation (le caller DOIT fournir une `Company` résolue via `get_company_for`). Refinement AC #1 documenté au Change Log (AC #1 et AC #2 étaient mutuellement incompatibles — constat AA-1). [crates/kesh-api/src/routes/invoice_pdf_service.rs:63-116]
- [x] [Review][Patch] P2 (LOW, blind BH-4) : doc `filename_base` « sans extension host » ambigu → « sans extension ni chemin ». [crates/kesh-api/src/routes/invoice_pdf_service.rs:48]

Dismiss (1) : BH-5 (duplication doc de module entre `invoice_pdf.rs` et `invoice_pdf_service.rs` sans lien Rustdoc) — intention « thin wrapper » assumée, renvoi en prose déjà présent dans les deux en-têtes.

**Pass 2 (2026-07-09, Haiku 4.5 × 3 reviewers, contexte frais, diff aplati unique `370a9b08..HEAD`)** — **0 > LOW, convergence.** AA2 : 9/9 AC conformes (refinement AC #1 Pass 1 vérifié tracé), 0 finding. ECH2 : 0 finding (24 vérifications outillées : call-sites résiduels, séquence, visibilités, chemins d'erreur, edge cases filename/locale/TOCTOU). BH2 : 3 LOW informatifs, tous dismiss — BH2-1 (commentaire laconique errors.rs : non-issue), BH2-2 (« get_company_for non visible » : **réfuté**, le handler l'appelle bien — artefact de la vue diff-only du Blind Hunter, pas d'hallucination d'indexation), BH2-3 (locale paramétrable : design intentionnel documenté). 0 hallucination Haiku sur ce cycle.

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

Claude Fable 5 (claude-fable-5) — run unique 2026-07-09.

### Debug Log References

- Gate Test Locally First : `cargo fmt` ✅, `cargo build --workspace --all-targets` ✅ (1m17s), `cargo clippy --workspace --all-targets -- -D warnings` ✅ (0 warning), `cargo test --workspace -j1 -- --test-threads=1` ✅ exit 0 — **91 suites, 0 échec** (log : scratchpad `test-workspace.log`). Flake KF-038 (#228) non réapparu (mode série).
- `invoice_pdf_e2e.rs` : **11/11 verts, fichier non modifié** — preuve d'iso-comportement (AC #8).
- `sanitize_filename_replaces_non_alphanumeric` passe à son nouvel emplacement (`routes::invoice_pdf_service::tests`).

### Completion Notes List

- **T1** : `invoice_pdf_service.rs` créé — `render()` reproduit la séquence exacte du handler historique (AC #2), `locale` en paramètre (AC #3), `RenderedInvoicePdf { bytes, filename_base }` (AC #1). Les 7 helpers déplacés à visibilité identique (`pub const` / `pub(crate)` / privé pour `build_qrbill_inputs`), doc-comments conservés verbatim (dont le calcul géométrique de `MAX_LINES_PER_PDF` et la note anti-injection de `fetch_country`).
- **Décision d'implémentation (déviation mineure vs AC #2, comportement inchangé)** : la spec fixe la signature `render(pool, i18n, locale, company_id, invoice_id)` mais `build_qrbill_inputs` a besoin de l'entité `Company` complète (adresse/nom/IDE), que le handler tenait de `get_company_for`. Le service recharge donc la company via `companies::find_by_id(pool, company_id)` (même requête que `get_company_for`) — coût : 1 SELECT PK supplémentaire par rendu ; bénéfice : le service reste autonome pour 20-3b, signature spec respectée. Company absente → `AppError::Database(DbError::NotFound)` (cas inatteignable via le handler qui a déjà validé la company).
- **T2** : `get_invoice_pdf` réduit au thin wrapper (auth + `tracing::info!` conservé + `render` + Response). Content-Disposition strictement identique : `inline; filename="facture-{base}.pdf"` avec fallback `HeaderValue::from_static("inline")`.
- **T3** : 3 call-sites re-routés (`credit_notes.rs:26`, `errors.rs:955` + doc `:210`) + 1 doc-comment supplémentaire hors spec (`errors.rs:710`, citait le mapping `QrBillError::PdfGeneration` « dans invoice_pdf.rs » — chemin mis à jour pour exactitude).
- **T4** : gate complet vert (cf. Debug Log). Aucun nouveau test requis (story mécanique, E2E existant = filet), conformément aux Testing standards de la spec.
- Frontières respectées : 0 migration, 0 frontend, 0 changement `kesh-qrbill`, aucune logique modifiée dans `build_qrbill_inputs`.

### File List

- `crates/kesh-api/src/routes/invoice_pdf_service.rs` — **nouveau** : service `render` + `RenderedInvoicePdf` + helpers déplacés (`MAX_LINES_PER_PDF`, `build_qrbill_inputs`, `split_lines`, `build_i18n`, `map_qrbill_error`, `sanitize_filename`, `fetch_country`) + test unitaire déplacé.
- `crates/kesh-api/src/routes/invoice_pdf.rs` — **modifié** : réduit au thin wrapper HTTP (handler `get_invoice_pdf` seul).
- `crates/kesh-api/src/routes/mod.rs` — **modifié** : `pub mod invoice_pdf_service;`.
- `crates/kesh-api/src/routes/credit_notes.rs` — **modifié** : import des 5 helpers depuis `invoice_pdf_service`.
- `crates/kesh-api/src/errors.rs` — **modifié** : chemin `MAX_LINES_PER_PDF` (`:955`) + 2 doc-comments (`:210`, `:710`).
- `_bmad-output/implementation-artifacts/20-3a-service-pdf-facture.md` — **modifié** : story file (tasks, Dev Agent Record, status).
- `_bmad-output/implementation-artifacts/sprint-status.yaml` — **modifié** : 20-3a in-progress → review.

## Change Log

- 2026-07-09 — **Code-review CONVERGÉ en 2 passes** — trend findings > LOW : Pass 1 (Sonnet 5 × 3 reviewers) **5 MEDIUM** (dédupliqués en 1 cause racine reload `Company`) + 3 LOW → patches P1+P2 (`3180e903`) → Pass 2 (Haiku 4.5 × 3, contexte frais, diff aplati) **0** (3 LOW dismiss, 0 hallucination). Modèles : implémentation Fable 5, Pass 1 Sonnet 5, Pass 2 Haiku 4.5 (rotation conforme). Reclassement : refinement AC #1 (signature `&Company`) décidé et documenté en Pass 1, validé conforme par l'AA Pass 2. Status → done.
- 2026-07-09 — `bmad-code-review` **Pass 1** (reviewers Sonnet 5 × 3 : Blind Hunter + Edge Case Hunter + Acceptance Auditor, orchestration Fable 5) : 8 findings bruts → **2 patch + 1 dismiss** (0 CRITICAL/HIGH ; 5 MEDIUM dédupliqués en 1 cause racine). **P1 appliqué (refinement AC #1)** : signature `render(..., company: &Company, invoice_id)` au lieu de `company_id: i64` — l'Acceptance Auditor a démontré qu'AC #1 (signature id) et AC #2 (séquence exacte, qui suppose l'entité déjà chargée) étaient mutuellement incompatibles ; le fix racine supprime le reload `companies::find_by_id` (requête redondante, branche NotFound morte, fenêtre TOCTOU) et rend le doc-comment « séquence exacte » vrai ; contrat d'autorisation explicité (le caller DOIT fournir une `Company` issue de `get_company_for`). AC #4 : le handler passe `&company` (déjà chargée). P2 appliqué : doc `filename_base` clarifiée. La déviation « reload Company » notée en dev-story est ainsi résolue à la racine plutôt que reclassée en dette.
- 2026-07-09 — `bmad-dev-story` (Claude Fable 5) : implémentation complète T1→T4 en run unique. Refactor mécanique : extraction du service `invoice_pdf_service::render` depuis `get_invoice_pdf` (thin wrapper), déplacement des 7 helpers partagés, re-routage de 3 call-sites (+1 doc-comment hors spec `errors.rs:710`). Déviation documentée : rechargement de la `Company` dans `render` (signature spec `company_id: i64` vs besoin entité complète de `build_qrbill_inputs`). Gate Test Locally First vert : fmt/build/clippy 0 warning, tests workspace série 91 suites 0 échec, `invoice_pdf_e2e` 11/11 **inchangé** (iso-comportement prouvé), KF-038 non réapparu. Status → review.
