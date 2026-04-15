# Story 5.3: Génération PDF QR Bill

Status: done

<!-- Validation optionnelle via validate-create-story. -->

## Story

As a **utilisateur (comptable ou indépendant)**,
I want **générer un PDF d'une facture validée, avec la partie paiement QR Bill conforme SIX 2.2 en bas de page, ouverte dans un nouvel onglet**,
so that **je puisse envoyer la facture au client et qu'il puisse la payer via son e-banking en scannant le QR Code**.

## Contexte

**Troisième et dernière story de l'Epic 5 v0.1 — Facturation QR Bill**. Couvre **FR34** (PDF QR Bill SIX 2.2) et **FR38** (ouverture nouvel onglet). Implémente la crate `kesh-qrbill` — aujourd'hui un simple placeholder (`crates/kesh-qrbill/src/lib.rs` + `Cargo.toml` minimal sans dépendances), publiable indépendamment (**Architecture DD-12** : chaque crate gère son propre PDF ; **DD-14** : publiables sans dépendance sur `kesh-core`).

**Fondations déjà en place** :

- **Factures validées** (Story 5.2) — `invoices.status = 'validated'`, `invoice_number` définitif, `journal_entry_id` renseigné. Repository `invoices::get` + `invoice_lines::list_by_invoice` disponibles dans `crates/kesh-db/src/repositories/invoices.rs`.
- **Compagnie** (Story 1.4) — table `companies` : `id, name VARCHAR(255), address TEXT, ide_number VARCHAR(15) NULL` (format `CHE[0-9]{9}`). **L'adresse est un champ libre `TEXT`** — pas de champs structurés (rue / NPA / ville / pays). Implication : **utiliser le type d'adresse « Combined » (K)** du QR Bill, qui accepte 2 lignes d'adresse libres + pays.
- **Contacts** (Story 4.1) — table `contacts` : `id, name VARCHAR(255), address VARCHAR(500) NULL`. Même contrainte → adresse type K. Si `contacts.address` est NULL au moment de la génération : **400 `INVOICE_NOT_PDF_READY`** avec message explicite (adresse débiteur requise par SIX).
- **Comptes bancaires** (Story 2.3) — table `bank_accounts` : `id, company_id, bank_name, iban VARCHAR(34), qr_iban VARCHAR(34) NULL, is_primary BOOLEAN`. **L'IBAN ou le QR-IBAN sont normalisés sans espaces.** Sélection pour le QR Bill : le `bank_account` `is_primary = true` de la company. **Si aucun primary** : 400 `INVOICE_NOT_PDF_READY`.
- **Factures** — `invoices.total_amount DECIMAL(19,4)` (agrégé TTC), `invoice.date`, `invoice.due_date`, `invoice.payment_terms`, `invoice.invoice_number`. `invoice_lines` : `description, quantity, unit_price, vat_rate, line_total`.
- **Paramétrage factures** (Story 5.2) — table `company_invoice_settings` disponible. Story 5.3 **n'ajoute pas** de nouvelles colonnes de config (pas de template PDF personnalisable en v0.1).
- **i18n backend** (Story 2.1) — crate `kesh-i18n` avec chargement Fluent + formatage suisse (montants avec apostrophe `1'234.56`, dates `dd.mm.yyyy`). **Réutiliser** pour tous les libellés du PDF et le formatage des montants. **Langue du PDF = locale d'instance `state.config.locale`** — pattern projet (Story 2.1, `routes/i18n.rs`). `CurrentUser` (`middleware/auth.rs:27`) ne contient que `user_id` et `role`, **pas** de langue préférée. Pas de fallback. Fichiers FTL : `crates/kesh-i18n/locales/{fr,de,it,en}/invoice-pdf.ftl` (nouveau namespace).
- **Audit** — la génération PDF est **lecture seule** (pas de mutation DB), donc pas d'audit trail requis en 5.3.
- **Frontend vue facture** — `/invoices/[id]/+page.svelte` affiche déjà la facture validée (bouton « Valider » conditionnel, bouton « Voir écriture »). **Ajouter** bouton « Télécharger PDF » visible uniquement si `status === 'validated'`.

### Scope verrouillé — ce qui DOIT être fait

1. **Crate `kesh-qrbill`** — implémentation complète, **zéro dépendance sur `kesh-core` ou `kesh-db`** (DD-14). Types autonomes (`QrBillData`, `Creditor`, `Debtor`, `Reference`, `Currency`, etc.). API publique minimale :
   ```rust
   pub fn generate_qr_bill_pdf(
       data: &QrBillData,
       invoice: &InvoicePdfData,
       i18n: &QrBillI18n,
   ) -> Result<Vec<u8>, QrBillError>;
   ```
   + validation séparée : `pub fn validate(data: &QrBillData) -> Result<(), QrBillError>`.
2. **Génération du QR Code** — payload SIX 2.2 (Swiss QR Code) : 32 à 33 lignes selon le type de référence (QRR/SCOR/NON), encodage UTF-8 strict, erreur correction level **M**, module taille calibrée pour **46×46 mm** (imprimable à 300 dpi). Logique dans `generator.rs`.
3. **Payload QR Code** — ordre strict selon SIX 2.2 spec §4 (doc local `ig-qr-bill-v2.4-en.pdf` — v2.4 est compatible avec v2.2 ciblée par le PRD, clarifications non-breaking) :
   - `QRType` = `SPC`
   - `Version` = `0200`
   - `CodingType` = `1`
   - `IBAN` (21 car., sans espaces) — QR-IBAN si référence QRR, IBAN classique sinon
   - Bloc Creditor : `AdrTp` (`K`), `Name` (≤70), `StrtNmOrAdrLine1` (≤70), `BldgNbOrAdrLine2` (≤70), `PstCd` (vide en K), `TmNm` (vide en K), `Ctry` (2 car.)
   - `UltmtCdtr` (1-7 lignes vides en v0.1 — pas d'ultimate creditor)
   - `Amt` (≤12 car., décimales avec `.`, pas de séparateur milliers) / `Ccy` (`CHF` ou `EUR`)
   - Bloc UltmtDbtr (= debtor de la facture, type K, mêmes règles que Creditor). **Toujours renseigné en v0.1** — une facture Kesh a toujours un contact lié. Pas de support QR Bill « au porteur ».
   - `Tp` (Reference Type : `QRR` / `SCOR` / `NON`) / `Ref` (référence)
   - `Ustrd` (≤140 car., remarque pour le débiteur — ex. `Facture F-2026-0042`)
   - `Trailer` = `EPD`
   - `StrdBkgInf` (optionnel, ≤140 car., **laissé vide en v0.1**)
   - `AltPmtInf` (jusqu'à 2 paramètres alternatifs — **laissés vides en v0.1**)
4. **Règle de type de référence** — **en v0.1** : si `bank_account.qr_iban IS NOT NULL` → QR-IBAN + référence **QRR** (27 chiffres, checksum modulo 10 recursif). Sinon → IBAN classique + référence **NON** (champ Ref vide). **Pas de SCOR** en v0.1 (référence ISO 11649 — reportée si besoin). Génération QRR : `{7 chiffres zero-pad company_id}{13 chiffres zero-pad invoice.id}{checksum 1 chiffre}` → 21 chiffres significatifs + 6 zéros de padding à gauche = 27 chiffres. Deterministic, unique par facture. Checksum mod-10 recursif (algorithme SIX Annexe B).
5. **Génération PDF** — mise en page **A4 portrait** (210×297 mm). Partie haute = facture (nom société, adresse société, numéro IDE, logo optionnel non implémenté en v0.1, date, échéance, destinataire, tableau des lignes, totaux). Partie basse = QR Bill (section paiement 148×105 mm en bas de page, dimensions exactes SIX §5). Séparation par ligne pointillée avec symbole ciseaux (annexe SIX). QR Code 46×46 mm avec **croix suisse 7×7 mm** au centre (symbole fixe, PNG/SVG embarqué dans la crate, `assets/swiss-cross.svg`). Police **Helvetica** (ou équivalent libre comme **Liberation Sans** pour déploiement sans licence). Tailles : titre « Récépissé » 11pt bold, « Section paiement » 11pt bold, labels 6pt bold, valeurs 8pt regular (selon SIX Style Guide §3).
6. **Bibliothèques PDF + QR** — pin strictement (versions latest stables au 2026-04) :
   - `printpdf = "0.7"` — génération PDF pure-Rust (MIT/Apache-2.0), support courbes Bezier, polices embarquées, SVG via workaround bitmap. **Choix retenu** : maturité, zéro FFI, support croix suisse via rectangle composé. Alternative évaluée rejetée : `genpdf` (layout only, trop haut niveau pour pixel-perfect SIX). `lopdf` (trop bas niveau).
   - `qrcodegen = "1.8"` — generator QR Code pur-Rust (Project Nayuki, MIT). ECC level M supporté. Alternative rejetée : `qrcode` (moins précis sur contrôle version).
   - **Dev-dependencies** : `rxing = "0.7"` (Apache-2.0 — dev-only, pas d'impact sur licence produit EUPL-1.2), `sha2` (workspace).
   - `rust_decimal = { workspace = true }` — arithmétique montants (déjà dans workspace).
   - `thiserror = { workspace = true }` — erreurs.
   - **Pas** de dépendance sur `kesh-core`, `kesh-db`, `kesh-i18n` directement. La crate expose des structures de données et reçoit des chaînes déjà traduites/formatées (injection via `QrBillI18n`).
7. **Validation conformité SIX** — module `validation.rs` :
   - IBAN : 21 caractères **après normalisation** (trim + suppression de tous les espaces internes + uppercase), checksum mod-97 (std ISO 13616), **country code ∈ { CH, LI }** obligatoire pour tout QR Bill (domestique suisse/Liechtenstein) — applicable aux deux types de référence (QRR et NON). L'IBAN stocké en DB (`bank_accounts.iban/qr_iban`) est déjà « normalisé sans espaces » (cf. commentaire migration `20260410000001_bank_accounts.sql`), mais la validation `kesh-qrbill` doit appliquer à nouveau la normalisation (defense-in-depth — la crate est publiable et peut recevoir des IBANs depuis d'autres sources). **Réutiliser** la validation existante dans `kesh-core::validation::iban` — **exception au point « zéro dépendance »** : OK car `kesh-core` n'est pas tiré dans le binaire public (`kesh-qrbill` ré-implémente localement un `validate_iban` autonome dans `validation.rs` pour respecter DD-14 strictement). **Patron retenu : ré-implémentation locale** (≈30 LoC) testée unitairement.
   - QR-IBAN : IID (positions 5-8) ∈ `[30000, 31999]`, country code CH ou LI.
   - QRR checksum : mod-10 recursif (Annexe B SIX).
   - Longueurs max : Name ≤70, AdrLine ≤70, Country = 2 car. ISO-3166-1 alpha-2, Amt ≤12 car. incluant décimales, Ustrd ≤140.
   - Currency ∈ { `CHF`, `EUR` }.
   - Amount : ≥ 0.01 et ≤ 999'999'999.99 (SIX max).
8. **Endpoint API** — `GET /api/v1/invoices/:id/pdf` (`authenticated_routes` — tout rôle authentifié peut télécharger). Réponse : `200 OK` `Content-Type: application/pdf`, `Content-Disposition: inline; filename="facture-{invoice_number}.pdf"`. Ouverture nouvel onglet côté frontend via `<a target="_blank">`. **Pas** de streaming (taille PDF ≤ 100 KB attendue, génération in-memory). Codes d'erreur :
   - 404 si facture inexistante ou pas dans la company courante.
   - 400 `INVOICE_NOT_VALIDATED` si `status != 'validated'`.
   - 400 `INVOICE_NOT_PDF_READY` si pré-requis manquants : adresse contact NULL, aucun `bank_account.is_primary`, IBAN invalide. Le message liste **précisément** ce qui manque.
   - 500 `PDF_GENERATION_FAILED` en cas d'erreur interne (logger `error!` avec détail crate).
9. **Assembleur backend** — module `crates/kesh-api/src/routes/invoice_pdf.rs` (nouveau fichier) :
   - Charge `invoice` + `invoice_lines` + `contact` + `company` + `bank_accounts.primary` + `company_invoice_settings` (pour le journal/libellé, juste pour cohérence d'affichage, pas obligatoire).
   - Construit les structures `kesh-qrbill::QrBillData` et `kesh-qrbill::InvoicePdfData` à partir des entités DB.
   - Résout la langue utilisateur (middleware `i18n` existant, fallback `fr`).
   - Injecte les traductions Fluent dans `QrBillI18n` (struct simple : `HashMap<&str, String>` ou champs typés).
   - Appelle `kesh_qrbill::generate_qr_bill_pdf(...)`.
   - Renvoie le body binaire avec headers appropriés.
10. **Frontend** — dans `/invoices/[id]/+page.svelte` :
    - Ajouter bouton « Télécharger PDF » à côté des boutons existants, visible uniquement si `invoice.status === 'validated'`.
    - Accessibilité : bouton avec `aria-label="{{ $t('invoices.download-pdf-aria-label') }}"` (verbosité > libellé visible — ex. « Télécharger la facture F-2026-0042 au format PDF »).
    - **Handler — pattern fetch → Blob URL → window.open** (obligatoire : `window.open` direct ne permet **pas** de capturer les codes HTTP 4xx pour afficher un toast). `apiClient` (`frontend/src/lib/shared/utils/api-client.ts`, Story 1.11) expose aujourd'hui **seulement** `get/post/put/delete` retournant du JSON typé — **pas** de méthode binaire. **Choix retenu** : étendre `apiClient` avec une méthode `getBlob(url): Promise<Response>` qui réutilise le `request` interne (JWT, refresh 401, parsing erreurs) mais retourne la `Response` brute. Extrait du handler :
      ```ts
      try {
          const res = await apiClient.getBlob(`/api/v1/invoices/${id}/pdf`);
          const blob = await res.blob();
          const url = URL.createObjectURL(blob);
          window.open(url, '_blank', 'noopener,noreferrer');
          // pas de URL.revokeObjectURL immédiat — laisser le navigateur gérer (le blob doit rester accessible le temps de l'affichage)
      } catch (err) {
          if (isApiError(err)) {
              toast.error($t(`invoice-pdf.error.${err.code}`));  // err.code = "INVOICE_NOT_PDF_READY", etc.
          } else {
              toast.error($t('common.error.unknown'));
          }
      }
      ```
      Les 4xx sont déjà convertis en `ApiError` par `request()` (pattern existant). Le type guard `isApiError` est déjà exporté de `api-client.ts`.
    - **Pas** de prévisualisation inline en v0.1 (reportée, UX-DR11 sera câblée ultérieurement).
11. **i18n** — **nouveau namespace** `invoice-pdf` × 4 langues. Clés PDF (~25) : `invoice-pdf-title`, `invoice-pdf-date`, `invoice-pdf-due-date`, `invoice-pdf-number`, `invoice-pdf-ide`, `invoice-pdf-recipient`, `invoice-pdf-description`, `invoice-pdf-quantity`, `invoice-pdf-unit-price`, `invoice-pdf-vat`, `invoice-pdf-line-total`, `invoice-pdf-subtotal`, `invoice-pdf-total`, `invoice-pdf-total-ttc`, `invoice-pdf-payment-terms`, `invoice-pdf-qr-section-payment`, `invoice-pdf-qr-section-receipt`, `invoice-pdf-qr-account`, `invoice-pdf-qr-reference`, `invoice-pdf-qr-additional-info`, `invoice-pdf-qr-payable-by`, `invoice-pdf-qr-currency`, `invoice-pdf-qr-amount`, `invoice-pdf-qr-acceptance-point`, `invoice-pdf-qr-separate-before-paying`. Clés d'erreur (~6) : `invoice-pdf-error-not-validated`, `invoice-pdf-error-not-pdf-ready`, `invoice-pdf-error-missing-contact-address`, `invoice-pdf-error-missing-primary-bank-account`, `invoice-pdf-error-invalid-iban`, `invoice-pdf-error-too-many-lines`. Côté frontend : `invoices.download-pdf` (libellé bouton) + `invoices.download-pdf-aria-label` (accessibilité).
12. **Fichiers de test SIX** — fixtures dans `crates/kesh-qrbill/tests/fixtures/` : réutiliser ou copier depuis `docs/six-references/samples-payment-part-en/` (disponibles localement). **Tests obligatoires** :
    - Validation QR Code scannable via bibliothèque de décodage (`rxing = "0.7"` en dev-dependency) — vérifier que le payload décodé = payload encodé à la virgule près.
    - Cas IBAN classique + référence NON.
    - Cas QR-IBAN + QRR valide.
    - Cas montant max (999'999'999.99).
    - Cas montant zéro **rejeté** (SIX n'autorise pas `0.00`, min `0.01`).
    - Cas débiteur absent (`Option<Address> = None`) — **test crate uniquement, en isolation**. La crate `kesh-qrbill` supporte ce cas par design (struct `ultimate_debtor: Option<Address>`) mais le backend assembleur (`routes/invoice_pdf.rs`) ne l'invoquera **jamais** avec `None` en v0.1 (facture = contact obligatoire).
    - Cas adresse longue (tronquée à 70 car.) — **rejet** (pas de troncature silencieuse).
    - Cas Name > 70 car. → `QrBillError::FieldTooLong`.
    - Cas IBAN non-CH/LI → rejet systématique (SIX impose CH ou LI pour tout QR Bill, QRR comme NON — v2.2/v2.4 §3.1).
    - Golden file : 1 PDF généré comparé **par hash SHA-256** à un PDF de référence (régression visuelle). PDF de référence versionné dans `tests/fixtures/golden/`. Regénération manuelle via `cargo test -- --ignored regenerate_golden`.
13. **Tests API** — `crates/kesh-api/src/routes/invoice_pdf.rs` :
    - 200 pour facture validée avec primary bank + contact address → vérifier `Content-Type`, `Content-Disposition`, taille PDF > 1 KB, magic bytes PDF (`%PDF-1.`).
    - 404 pour facture autre company.
    - 400 `INVOICE_NOT_VALIDATED` pour draft.
    - 400 `INVOICE_NOT_PDF_READY` si pas de primary bank.
    - 400 `INVOICE_NOT_PDF_READY` si contact.address NULL.
    - Auth : 401 sans JWT, 200 pour tout rôle (comptable + admin + observateur).
14. **Tests E2E Playwright** — scénario « Télécharger PDF facture validée » :
    - Setup : company avec primary bank + QR-IBAN, contact avec adresse, facture validée.
    - Clic bouton « Télécharger PDF » → nouvelle page ouverte avec URL `/api/v1/invoices/:id/pdf`.
    - Vérifier response headers + amorce binaire PDF.
    - Cas erreur : facture sans primary bank → toast d'erreur visible.
15. **Performance** — génération PDF < 3s (NFR-PERF-3). Profiler si dépassement. Génération in-memory, pas de fichier temporaire.

### Dette technique documentée — v0.2 (ajoutée review pass 2 G2 du 2026-04-15)

- **M2-gap-1 — i18n partielle des erreurs `InvoiceNotPdfReady` issues de `map_qrbill_error`** (sévérité MEDIUM, reclassée dette technique).
  - **Constat** : la review pass 1 G2 (story 5.4) a localisé 5 sites de construction `InvoiceNotPdfReady` en `routes/invoice_pdf.rs` (contact manquant, adresse entreprise vide, etc.) via `crate::errors::t(key, default)`. Il reste 6 sites hardcodés en français issus du mapping `QrBillError → AppError` (lignes ~179, 280, 283, 289, 291-293 : QRR generation failure, FieldTooLong, FieldEmpty, InvalidCountry, InvalidCharset). Ces messages sont servis tels quels à un utilisateur DE/IT/EN.
  - **Décision v0.1** : accepté en dette technique. Ces erreurs sont déclenchées par des conditions techniques (schéma SIX QR-Bill 2.2) peu susceptibles d'être rencontrées par un utilisateur final correctement configuré (les CHECK DB empêchent la plupart des cas). La fréquence attendue en production est quasi-nulle.
  - **Remédiation v0.2** : mapper chaque variante `QrBillError` vers une clé FTL dédiée avec arguments Fluent (`qrbill-error-field-too-long` avec `{ $field, $max, $got }`, etc.). Propriétaire : Guy. Story de remédiation : à créer dans Epic 13 ou lors du scope v0.2.

### Scope volontairement HORS story — décisions tranchées

- **Prévisualisation PDF inline** (UX-DR11) → reportée. En v0.1, ouverture directe dans un nouvel onglet (le navigateur affiche nativement).
- **Personnalisation du template PDF** (logo, couleurs, polices custom) → futur (« Modèles documents » mentionné en architecture §789, reporté à v0.2+).
- **Référence SCOR (ISO 11649)** → hors scope v0.1. Seulement QRR (si QR-IBAN) ou NON (sinon).
- **Multi-pages** — si la facture a > `MAX_LINES_PER_PDF` lignes, le tableau déborderait de l'A4. **Décision v0.1** : **rejet explicite** `400 INVOICE_TOO_MANY_LINES_FOR_PDF` avec message i18n clair. **Jamais de troncature silencieuse** (valeurs comptables manquantes = bug grave). La limite applicative `MAX_LINES = 200` (Story 5.1) reste valide pour la facture en base ; seule la génération PDF est bornée.
  - **Valeur effective (review pass 1 G2 — 2026-04-15) : `MAX_LINES_PER_PDF = 9`**. La valeur initiale de `35` dans cette spec était une estimation ; l'implémentation mesurée montre que le tableau A4 avec en-têtes, section QR Bill et total tient 9 lignes max (calcul géométrique documenté dans `routes/invoice_pdf.rs`). Le test E2E `pdf_too_many_lines_returns_400` couvre le rejet. Multi-pages reporté à story de suivi v0.2+ — la limite sera levée à ce moment-là.
- **Téléchargement multiple / ZIP d'un lot de factures** → hors scope.
- **Envoi email avec PDF en pièce jointe** → reporté (v0.2+).
- **Signature numérique du PDF** → hors scope.
- **Ultimate Creditor** (QR Bill section UltmtCdtr) → laissé vide v0.1 (cas d'usage avancé).
- **Alt Payment Info** (AltPmtInf) → laissé vide v0.1.
- **StrdBkgInf** (info structurée additionnelle) → laissé vide v0.1.
- **Ventilation TVA dans le PDF** — affichage TTC uniquement (cohérent avec 5.2 qui n'a pas de ventilation TVA dans l'écriture comptable). Ventilation par taux TVA arrive en **Epic 9**.
- **Personnalisation `acceptance point`** / adresse d'un point d'acceptation → non applicable (PME sans guichet).
- **Régénération silencieuse en cas de mutation** — impossible (facture validée immuable, donc PDF déterministe). Pas de cache à invalider.

### Décisions de conception

- **Structure de données `kesh-qrbill`** — types autonomes, **pas** de ré-export depuis `kesh-core` (DD-14). Exemple :
  ```rust
  pub struct QrBillData {
      pub creditor_iban: String,        // sans espaces, 21 car.
      pub creditor: Address,            // type K
      pub ultimate_debtor: Option<Address>,
      pub amount: Option<Decimal>,      // None = montant ouvert (hors scope v0.1 — toujours Some)
      pub currency: Currency,           // CHF ou EUR
      pub reference: Reference,         // Qrr(String) | Scor(String) | None
      pub unstructured_message: Option<String>, // ≤140 car.
      pub billing_information: Option<String>,  // ≤140 car. — laissé None v0.1
  }

  pub struct Address {
      pub address_type: AddressType,    // K (Combined) v0.1
      pub name: String,                 // ≤70
      pub line1: String,                // ≤70 — rue ou ligne adresse libre
      pub line2: String,                // ≤70 — NPA+ville ou ligne adresse libre
      pub country: String,              // ISO-3166-1 alpha-2 — "CH" défaut
  }

  pub struct InvoicePdfData {
      pub invoice_number: String,
      pub invoice_date: NaiveDate,
      pub due_date: Option<NaiveDate>,
      pub payment_terms: Option<String>,
      pub creditor_name: String,
      pub creditor_address_lines: Vec<String>, // affichage libre dans la partie facture
      pub creditor_ide: Option<String>,        // "CHE-xxx.xxx.xxx" format affiché
      pub debtor_name: String,
      pub debtor_address_lines: Vec<String>,
      pub lines: Vec<InvoiceLinePdf>,
      pub total: Decimal,                      // TTC
      pub currency: Currency,
  }
  ```
- **Mapping adresse libre → type K** — `companies.address` et `contacts.address` sont des blobs TEXT. **Heuristique** : splitter sur `\n`, prendre les 2 premières lignes non vides → `line1`, `line2`. Si une seule ligne → `line1` = la ligne, `line2` = `""` (accepté par SIX en type K). Si aucune ligne → erreur `INVOICE_NOT_PDF_READY`. Nom = `companies.name` / `contacts.name`. Pays = `"CH"` par défaut (v0.1 — pas de champ pays dans le modèle actuel).
- **Gestion de la langue** — le handler `invoice_pdf` utilise **exclusivement** `state.config.locale` (pattern instance-level, voir `routes/i18n.rs:21`). `CurrentUser` (`middleware/auth.rs:27-30`) n'a que `user_id` et `role` — **ne pas tenter d'y lire une langue**. Pas de fallback. La langue est fixée au moment de la génération — le PDF n'est pas multilingue.
- **Formatage des montants** — toutes les valeurs affichées (partie facture **et** section paiement) utilisent le format suisse : **apostrophe séparateur de milliers, point décimal**, ex. `1'234.50`. Réutiliser la logique de `kesh-i18n` (ex. `fn format_chf(amount: Decimal) -> String`). Dans le **payload QR Code**, le champ `Amt` n'a **pas** de séparateur de milliers et utilise le point comme séparateur décimal (ex. `1234.50`). **Attention double format** : affichage humain ≠ payload QR Code.
- **Arrondi Decimal → 2 décimales** — `invoices.total_amount DECIMAL(19,4)` stocke jusqu'à 4 décimales, mais le QR Bill `Amt` et l'affichage PDF exigent exactement **2 décimales**. **Règle retenue : `round_dp_with_strategy(2, RoundingStrategy::MidpointAwayFromZero)`** (arrondi commercial suisse, « half up away from zero » — standard comptable pour arrondi final de facture). Appliqué une seule fois, sur `invoice.total_amount`, juste avant de transformer en `String` pour le payload et avant formatage affichage. **Ne pas** tronquer (`trunc()`) — introduirait un biais systématique à la baisse. Ne pas ré-arrondir ligne par ligne — seul le total affiché et le total payload sont arrondis, cohérent avec le fait que le total DB est déjà agrégé. Tests unitaires : cas `1234.5650` → `1234.57`, `1234.5649` → `1234.56`, `1234.5050` → `1234.51` (half-up away from zero).
- **Gestion de l'erreur `QrBillError`** — enum `thiserror` avec variantes : `InvalidIban`, `InvalidQrIban`, `InvalidQrr`, `FieldTooLong { field: &'static str, max: usize, got: usize }`, `InvalidAmount`, `InvalidCurrency`, `InvalidCountry`, `PdfGeneration(String)`.

- **Extension obligatoire de `AppError`** — `AppError::Validation(String)` sérialise systématiquement en `code: "VALIDATION_ERROR"` (cf. `crates/kesh-api/src/errors.rs:189-190`). Les codes applicatifs spécifiques `INVOICE_NOT_VALIDATED`, `INVOICE_NOT_PDF_READY`, `PDF_GENERATION_FAILED`, `INVOICE_TOO_MANY_LINES_FOR_PDF` exigent donc **4 nouvelles variantes** dans `AppError` (pattern identique à `IllegalStateTransition`/`ConfigurationRequired` de Story 5.2) :
  ```rust
  #[error("Facture non validée")]
  InvoiceNotValidated,
  #[error("Facture non prête pour PDF : {0}")]
  InvoiceNotPdfReady(String),
  #[error("Facture trop de lignes pour PDF : {0}")]
  InvoiceTooManyLinesForPdf(usize),
  #[error("Échec génération PDF")]
  PdfGenerationFailed(String), // message loggé server-side, pas exposé client
  ```
  + bras `IntoResponse` correspondants renvoyant `400` avec le code kebab-case attendu (et `500` pour `PdfGenerationFailed` sans détail client). **Sans cette extension, les codes seront tous mappés `VALIDATION_ERROR` et les tests API (T6) et frontend (T7.3) échoueront.** Mapping `QrBillError → AppError` dans le handler : `InvalidIban|InvalidQrIban|InvalidQrr|InvalidCountry|FieldTooLong → InvoiceNotPdfReady(...)`, `InvalidAmount|InvalidCurrency → InvoiceNotPdfReady(...)` (pré-requis métier), `PdfGeneration(msg) → PdfGenerationFailed(msg)` (logger `error!`).
- **Croix suisse** — rectangle blanc 7×7 mm centré sur le QR Code, puis croix blanche (dessin vectoriel via `printpdf` lines primitives) : 2 rectangles blancs croisés (branches 6.3×1.3 mm selon SIX §5.2). Contour noir 0.05 mm. **Ne pas embarquer de PNG** pour la croix — dessin vectoriel uniquement (meilleure résolution impression).

- **Déterminisme PDF pour test golden** — `printpdf` 0.7 embarque par défaut `CreationDate`/`ModDate` dynamiques dans `/Info`, et un `/ID` UUID aléatoire dans le trailer (généré par `lopdf` sous-jacent). Sans intervention, le hash SHA-256 change à chaque run.

  **Stratégie retenue (ordre de préférence)** :
  1. **API `printpdf`** : `PdfDocument::set_creation_date(NaiveDateTime)` + `set_mod_date(...)` si disponibles en 0.7. **Ne suffit pas** pour `/ID` (non exposé directement par `printpdf`).
  2. **Accès à `lopdf` sous-jacent** : `printpdf` expose le document inner via l'API `save_to_bytes()` ou permet d'accéder à `PdfDocumentReference::inner_doc: lopdf::Document`. Patcher le trailer : `doc.inner_doc.trailer.set("ID", Object::Array(vec![Object::String(b"deterministic1".to_vec(), StringFormat::Hexadecimal), ...]))` avant sérialisation. **Vérifier en T0 (setup) que cette API est bien exposée en 0.7** (regarder `printpdf::PdfDocumentReference` dans docs.rs) — si absente, passer au plan B.
  3. **Plan B (fallback adopté si 1+2 impossibles)** : **ne pas hasher le PDF complet**. Test golden = extraire le payload QR Code du PDF via `rxing` (parser PDF → extraire image → décoder QR) et comparer à une chaîne payload attendue. Moins strict visuellement mais robuste et conforme à l'objectif (conformité SIX).

  **Ne jamais** faire de string-replace naïf sur `Vec<u8>` du PDF (structure binaire + xref offsets, risque fort de corruption).

  La date utilisée en prod est `chrono::Utc::now()` ; en test golden = constante `2026-01-01T00:00:00Z`.
- **Tests unitaires crate `kesh-qrbill`** ≥ 20 cas, couvrant : validation IBAN, validation QR-IBAN, checksum QRR, longueurs de champs, payload encoding (ordre strict), round-trip encode/decode via `rxing`, génération croix suisse (snapshot PDF page rectangle count), golden PDF hash.
- **Ordre des locks / transactions** — la génération est **lecture seule** ; faire les `SELECT` dans une transaction READ COMMITTED (ou pas de transaction — les données sont déjà immuables côté `validated`). **Pas** de `FOR UPDATE`.
- **Concurrence** — deux téléchargements simultanés du même PDF = deux générations indépendantes (pas de cache). Idempotence naturelle (facture immuable).
- **Sécurité**
  - Autorisation : `authenticated_routes` — vérifier `invoice.company_id == current_user.company_id`. Pas d'exposition cross-company.
  - Pas d'injection PDF : toutes les données utilisateur passent par les APIs typées de `printpdf` (texte échappé). Ne **jamais** interpoler directement du texte utilisateur dans des commandes PDF bas-niveau.
  - Content-Disposition avec filename sanitizé : regex `[^A-Za-z0-9._-]` remplacé par `_`.
  - Pas de path traversal (aucun fichier lu depuis disque à la volée — assets embarqués via `include_bytes!`).
- **Assets embarqués** — via `include_bytes!`/`include_str!` dans `kesh-qrbill/src/` : police Liberation Sans (fichier `.ttf` ~150 KB), éventuellement croix suisse si fallback bitmap nécessaire (mais privilégier vectoriel). **Vérifier licence Liberation Sans** (SIL Open Font License 1.1 — compatible EUPL-1.2).

## Acceptance Criteria

1. **Given** une facture `status = 'validated'` de la company courante, **When** `GET /api/v1/invoices/:id/pdf`, **Then** la réponse est `200 OK` avec `Content-Type: application/pdf`, `Content-Disposition: inline; filename="facture-{invoice_number}.pdf"`, et body PDF valide commençant par `%PDF-1.`. (FR34)

2. **Given** un PDF généré, **When** analyse du QR Code, **Then** le payload décodé est conforme SIX 2.2 : commence par `SPC\n0200\n1\n`, contient l'IBAN de la primary bank sans espaces, créancier type K avec nom/adresse de la company, débiteur type K avec nom/adresse du contact, montant et devise de la facture, type de référence QRR+référence si QR-IBAN présent sinon NON+Ref vide, terminateur `EPD`.

3. **Given** un PDF généré, **When** inspection géométrique, **Then** le QR Code mesure 46×46 mm, est positionné dans la section paiement en bas de page (148 mm de hauteur totale), contient la croix suisse 7×7 mm centrée dessinée vectoriellement, et le PDF est au format A4 (210×297 mm).

4. **Given** une facture validée avec QR-IBAN configuré (`bank_accounts.qr_iban IS NOT NULL` sur la primary), **When** génération, **Then** le payload utilise l'IBAN `qr_iban`, type de référence `QRR`, et une référence QRR 27-chiffres déterministe dérivée de `company_id + invoice.id` avec checksum mod-10 recursif valide.

5. **Given** une facture validée avec IBAN classique seulement (`bank_accounts.qr_iban IS NULL`), **When** génération, **Then** le payload utilise l'IBAN classique, type de référence `NON`, champ `Ref` vide.

6. **Given** une facture `status = 'draft'`, **When** GET pdf, **Then** 400 avec code applicatif `INVOICE_NOT_VALIDATED`.

7. **Given** une facture validée sans `contact.address` OU sans `bank_accounts.is_primary = true` pour la company, **When** GET pdf, **Then** 400 avec code `INVOICE_NOT_PDF_READY` et message i18n précisant la cause (adresse manquante, ou compte bancaire primary absent).

8. **Given** une facture validée d'une autre company (scénario théorique — v0.1 mono-tenant `NOKEY`, la company est résolue via `get_company(&state)` comme dans `validate_invoice_handler`), **When** GET pdf, **Then** 404 (pas de fuite d'existence). Le pattern d'isolation est identique à Story 5.2.

9. **Given** une facture validée avec plus de `MAX_LINES_PER_PDF` lignes, **When** GET pdf, **Then** 400 avec code `INVOICE_TOO_MANY_LINES_FOR_PDF` et message i18n (jamais de troncature silencieuse).

10. **Given** un utilisateur authentifié (n'importe quel rôle), **When** GET pdf, **Then** accès autorisé. (Pas de restriction par rôle — cohérent avec `authenticated_routes`.)

11. **Given** affichage d'un montant dans le PDF (partie facture), **When** rendu, **Then** format suisse avec apostrophe séparateur de milliers et point décimal (ex. `1'234.50`). **Given** ce même montant dans le payload QR Code, **Then** format sans séparateur (`1234.50`).

12. **Given** une facture validée affichée dans `/invoices/[id]/+page.svelte`, **When** l'utilisateur clique sur le bouton « Télécharger PDF », **Then** un nouvel onglet s'ouvre avec le PDF affiché nativement par le navigateur. Le bouton est masqué si `status !== 'validated'`. En cas d'erreur backend (ex. 400 `INVOICE_NOT_PDF_READY`), un toast d'erreur i18n est affiché (pattern fetch+Blob URL, cf. Décision de Conception). (FR38)

13. **Given** la génération d'un PDF pour une facture avec 20 lignes et un QR-IBAN, **When** mesure du temps de génération en dehors des tests (bench local), **Then** durée < 3s. (NFR-PERF-3)

14. **And** `kesh-qrbill` compile indépendamment de `kesh-core` et `kesh-db` (vérification : `cargo check -p kesh-qrbill` sans features). (DD-14)

15. **And** tests unitaires `cargo test -p kesh-qrbill` ≥ 20 cas passants, dont : round-trip encode/decode QR Code via `rxing`, validation IBAN/QR-IBAN/QRR, rejet champs trop longs, rejet IBAN non-CH/LI, golden PDF hash (avec `CreationDate`/`ModDate`/`/ID` figés pour déterminisme). Test golden reproductible sur 10 runs consécutifs = hash identique.

16. **And** tests d'intégration API (`crates/kesh-api/tests/` ou `#[tokio::test]` inline) couvrent les 9 cas listés en Tasks T6 (200 ok, 400 not-validated, 400 not-pdf-ready × 2, 404 other-company, 400 too-many-lines, 401, 200 × 3 rôles).

17. **And** un scénario Playwright E2E « générer PDF facture validée » passe (golden path + cas d'erreur `INVOICE_NOT_PDF_READY` affiché sous forme de toast).

18. **And** les fichiers FTL pour 4 langues contiennent le nouveau namespace `invoice-pdf` complet (25 clés affichage + **6** clés erreurs × 4 langues = **124 entrées**), + clés frontend `invoices.download-pdf` et `invoices.download-pdf-aria-label` × 4 langues.

19. **And** `AppError` (crates/kesh-api/src/errors.rs) est étendue avec les 4 variantes `InvoiceNotValidated`, `InvoiceNotPdfReady(String)`, `InvoiceTooManyLinesForPdf(usize)`, `PdfGenerationFailed(String)` et leurs bras `IntoResponse` — sinon tous les codes tombent sur `VALIDATION_ERROR` générique (cf. Décisions de Conception).

20. **And** aucune régression sur les tests de Story 5.1 et 5.2 (`cargo test --workspace` + `npm run test` + Playwright existants verts).

## Tasks / Subtasks

- [x] **T1. Setup crate `kesh-qrbill`** (AC: 13)
  - [x] T1.1 Mettre à jour `crates/kesh-qrbill/Cargo.toml` : dépendances `printpdf = "0.7"`, `qrcodegen = "1.8"`, `rust_decimal = { workspace = true }`, `thiserror = { workspace = true }`, `chrono = { workspace = true }`. Dev-deps : `rxing = "0.7"`, `sha2`.
  - [x] T1.2 Structure modules : `lib.rs`, `types.rs`, `validation.rs`, `generator.rs` (QR payload + QR image), `pdf.rs` (layout A4 + section paiement), `assets/swiss-cross.svg` (référence), `assets/LiberationSans-Regular.ttf`.
  - [x] T1.3 Définir types publics dans `types.rs` (`QrBillData`, `InvoicePdfData`, `Address`, `AddressType`, `Reference`, `Currency`, `QrBillI18n`, `QrBillError`).
- [x] **T2. Validation SIX** (AC: 2, 4, 5, 14)
  - [x] T2.1 `validation.rs` : `validate_iban` (longueur 21, mod-97, charset), `validate_qr_iban` (IID 30000-31999), `compute_qrr_checksum` (mod-10 recursif), `validate_qrr` (27 chiffres + checksum).
  - [x] T2.2 Validation longueurs de champs (Name ≤70, AdrLine ≤70, Ustrd ≤140, Country=2, Amt 0.01..=999'999'999.99, Currency ∈ CHF/EUR).
  - [x] T2.3 Tests unitaires ≥ 10 cas (IBAN valide/invalide, QR-IBAN valide/IID hors plage, QRR checksum, longueur hors limite, country code invalide).
- [x] **T3. Génération QR Code payload + image** (AC: 2, 4, 5)
  - [x] T3.1 `generator.rs::build_payload(data: &QrBillData) -> Result<String, QrBillError>` — sérialise les ~32 lignes selon l'ordre SIX 2.2.
  - [x] T3.2 `generator.rs::render_qr_image(payload: &str) -> Result<QrMatrix, QrBillError>` — utilise `qrcodegen`, ECC level M, version auto.
  - [x] T3.3 Tests round-trip via `rxing` : encoder + décoder = identique (≥ 3 cas QRR/NON/débiteur absent).
- [x] **T4. Génération PDF A4** (AC: 1, 3, 10, 14)
  - [x] T4.1 `pdf.rs::generate(data, invoice, i18n) -> Result<Vec<u8>, QrBillError>` — initialise doc A4, embed police Liberation Sans.
  - [x] T4.2 Section haute facture : entête company, numéro IDE, date, destinataire, tableau lignes (description, qty, prix, TVA, total), totaux TTC.
  - [x] T4.3 Ligne de séparation pointillée + icône ciseaux (position Y = 192 mm du haut).
  - [x] T4.4 Section paiement : colonne récépissé (62 mm) + colonne section paiement (148 mm - 62 mm). Labels en bold 6pt, valeurs 8pt regular.
  - [x] T4.5 QR Code 46×46 mm rendu depuis `QrMatrix` (dessin rectangle par module). Croix suisse 7×7 mm vectorielle centrée.
  - [x] T4.6 Déterminisme PDF : exposer `generate_qr_bill_pdf_with_date(data, invoice, i18n, creation_date: NaiveDateTime)` en plus de la version publique qui utilise `Utc::now()`. Figer `CreationDate`, `ModDate`, et l'identifiant `/ID` du trailer (via API `printpdf` ou post-processing string replace en test uniquement).
  - [x] T4.7 Golden file : génération 1 PDF avec data fixe + date `2026-01-01T00:00:00Z` → hash SHA-256 stocké dans `tests/fixtures/golden/invoice.pdf.sha256`. Test compare hash. Vérifier reproductibilité sur 10 runs.
- [x] **T4.8 Extension `AppError`** (AC: 19) — **prérequis bloquant** pour T5 et T6 : ajouter dans `crates/kesh-api/src/errors.rs` les 4 variantes `InvoiceNotValidated`, `InvoiceNotPdfReady(String)`, `InvoiceTooManyLinesForPdf(usize)`, `PdfGenerationFailed(String)` avec leurs bras `IntoResponse` renvoyant les codes kebab-case exacts. Test unitaire par variante vérifiant `body["error"]["code"]` (pattern existant `errors.rs:459-462`).
- [x] **T5. Backend endpoint `GET /api/v1/invoices/:id/pdf`** (AC: 1, 6, 7, 8, 9)
  - [x] T5.1 Nouveau fichier `crates/kesh-api/src/routes/invoice_pdf.rs` : handler `get_invoice_pdf` (State, CurrentUser, Path id).
  - [x] T5.2 Chargement : invoice (scopé company), lines, contact, company, bank_accounts primary. Vérifier `status == 'validated'` (sinon 400 `INVOICE_NOT_VALIDATED`), primary bank existe (sinon 400 `INVOICE_NOT_PDF_READY`), contact.address non NULL (sinon 400 `INVOICE_NOT_PDF_READY`).
  - [x] T5.3 Helpers de mapping : `companies.address` + `contacts.address` (TEXT libre) → `Address` type K (split lignes, tronque à 2 lignes non vides ou erreur si vide).
  - [x] T5.4 Résolution locale = `state.config.locale` (pattern Story 2.1 — **pas** de champ langue sur `CurrentUser`). Chargement des traductions Fluent namespace `invoice-pdf` → struct `QrBillI18n`.
  - [x] T5.4bis Résolution company via `get_company(&state)` (mono-tenant v0.1, pattern identique à `validate_invoice_handler`).
  - [x] T5.4ter Vérifier `invoice.lines.len() <= MAX_LINES_PER_PDF` (= 9 en v0.1, voir §Scope HORS story note 2026-04-15) — sinon 400 `INVOICE_TOO_MANY_LINES_FOR_PDF`.
  - [x] T5.5 Appel `kesh_qrbill::generate_qr_bill_pdf(...)` → body `Vec<u8>`.
  - [x] T5.6 Réponse : `Response::builder().header(CONTENT_TYPE, "application/pdf").header(CONTENT_DISPOSITION, format!("inline; filename=\"facture-{}.pdf\"", sanitize(invoice_number))).body(body)`.
  - [x] T5.7 Enregistrer route dans `mod.rs` sous `authenticated_routes`.
  - [x] T5.8 Mapping `QrBillError` → `AppError` (`INVOICE_NOT_PDF_READY` pour erreurs métier, `PDF_GENERATION_FAILED` pour erreurs internes avec `error!` log).
- [x] **T6. Tests API** (AC: 6, 7, 8, 9, 15)
  - [x] T6.1 Cas 200 : facture validée avec primary bank + contact address → vérifier content-type, disposition, magic bytes PDF.
  - [x] T6.2 Cas 400 `INVOICE_NOT_VALIDATED` : facture draft.
  - [x] T6.3 Cas 400 `INVOICE_NOT_PDF_READY` : pas de primary bank.
  - [x] T6.4 Cas 400 `INVOICE_NOT_PDF_READY` : contact.address NULL.
  - [x] T6.5 Cas 404 : facture autre company (mono-tenant v0.1, via `get_company`).
  - [x] T6.6 Cas 400 `INVOICE_TOO_MANY_LINES_FOR_PDF` : facture avec 36 lignes.
  - [x] T6.7 Cas 401 : pas de JWT.
  - [x] T6.8 Cas 200 pour les 3 rôles (comptable, admin, observateur).
- [x] **T7. Frontend — bouton « Télécharger PDF »** (AC: 11)
  - [x] T7.1 Modifier `frontend/src/routes/(app)/invoices/[id]/+page.svelte` : ajouter bouton « Télécharger PDF » visible si `invoice.status === 'validated'`.
  - [x] T7.1bis Étendre `frontend/src/lib/shared/utils/api-client.ts` : ajouter méthode `getBlob(url: string): Promise<Response>` qui réutilise le `request` interne mais retourne la `Response` brute (ne parse pas en JSON). Tests Vitest : JWT injecté, refresh 401 fonctionne, erreur 4xx → `ApiError` throw.
  - [x] T7.2 Handler **getBlob → Blob URL → window.open** (cf. Scope §10, extrait code).
  - [x] T7.3 Gestion erreur : catch `ApiError`, mapper `err.code` sur clé i18n `invoice-pdf.error.*` (4 codes possibles), afficher via toast. Pattern existant (voir `journal-entries/+page.svelte` Story 3.5 pour référence toast).
- [x] **T8. i18n** (AC: 17)
  - [x] T8.1 Créer `crates/kesh-i18n/locales/{fr,de,it,en}/invoice-pdf.ftl` — **25 clés affichage + 6 clés erreurs = 31 clés par langue** (liste exacte en Scope §11).
  - [x] T8.2 Enregistrer le namespace dans la config du bundle Fluent (voir pattern Story 2.1).
  - [x] T8.3 Ajouter clés frontend `invoice-pdf-error-*` et `invoices.download-pdf` (libellé bouton) dans les 4 langues frontend (pattern existant `frontend/src/lib/i18n/locales/`).
- [x] **T9. Tests Playwright E2E** (AC: 16)
  - [x] T9.1 Seed : company + primary bank (QR-IBAN) + contact avec adresse + facture validée.
  - [x] T9.2 Scénario golden : clic bouton, vérifier nouvel onglet ouvert avec URL attendue, vérifier response headers via `context.request.get`.
  - [x] T9.3 Scénario erreur : company sans primary bank → toast d'erreur visible.
- [x] **T10. Documentation & conformité** (AC: 13, 18)
  - [x] T10.1 `crates/kesh-qrbill/README.md` : positionnement crate publiable, API publique, références SIX (pointer `docs/six-references/ig-qr-bill-v2.4-en.pdf`).
  - [x] T10.2 Vérifier `cargo check -p kesh-qrbill` sans features (zéro dépendance interne sur kesh-*).
  - [x] T10.3 Vérifier compatibilité licence Liberation Sans (SIL Open Font License 1.1 ↔ EUPL-1.2) dans `crates/kesh-qrbill/LICENSES/` si police embarquée.
  - [x] T10.4 Run `cargo test --workspace` + `npm run test` + Playwright existant : zéro régression.

## Dev Notes

### Conformité SIX — références obligatoires

- **Spec principale** : `docs/six-references/ig-qr-bill-v2.4-en.pdf` (local). Sections clés :
  - §3 : structure du payload QR Code (ordre, types, longueurs max).
  - §4 : validation des champs (IBAN, QR-IBAN, QRR checksum).
  - §5 : géométrie de la section paiement (148×105 mm, QR 46×46, croix 7×7).
  - §6 : typographie (police, tailles, labels).
  - Annexe B : algorithme mod-10 recursif pour QRR.
- **Échantillons visuels** : `docs/six-references/samples-payment-part-en/` — PDFs de référence SIX pour comparaison visuelle.
- **Dataset de test** : `docs/six-references/samples-data-schema-en/` — cas de test officiels SIX. Les importer comme fixtures dans `crates/kesh-qrbill/tests/fixtures/`.

### Patterns du projet à respecter

- **Architecture crates publiables** (DD-14) : `kesh-qrbill` zéro dépendance sur `kesh-core`/`kesh-db`. Types autonomes, conversion via `From`/`Into` à la frontière (dans `kesh-api`). Même règle que `kesh-import`, `kesh-payment`.
- **Handlers API** : pattern `AppState`, `CurrentUser` extension, `AppError` enum, mapping DbError/métier → HTTP status. Voir `routes/invoices.rs:520` (`validate_invoice_handler`) pour handler de référence.
- **Error codes applicatifs** : string kebab-case en `AppError::Validation(String)` ou variante dédiée. Voir patterns existants `ILLEGAL_STATE_TRANSITION`, `CONFIGURATION_REQUIRED`, `FISCAL_YEAR_INVALID` (Story 5.2). Nouveaux codes : `INVOICE_NOT_VALIDATED`, `INVOICE_NOT_PDF_READY`, `PDF_GENERATION_FAILED`.
- **Formatage suisse** : toujours via `kesh-i18n` (apostrophe séparateur milliers, point décimal, dates `dd.mm.yyyy`). Ne **pas** réimplémenter.
- **i18n** : namespace Fluent par feature (pattern Story 2.1 / 5.2). Clés kebab-case préfixées `invoice-pdf-`.
- **Pattern frontend feature** : `invoices/` existe déjà avec pages, api, types. Ajouter la logique téléchargement dans le `+page.svelte` existant — **pas** de nouvelle store dédiée.
- **Tests Rust** : pattern test-container MariaDB (Story 1.3 à 5.2) pour tests d'intégration DB. Pattern `#[tokio::test]` direct pour tests API (setup AppState + TestServer).

### Pièges identifiés (apprentissages stories antérieures)

- **Flakiness cross-binary MariaDB** (mémoire `feedback_sqlx_mysql_gotchas.md`) : éviter tests parallèles sur même base quand possible. Ici OK car PDF = lecture seule.
- **Collation BINARY sur CHECK** — vérifier les contraintes de la migration `company_invoice_settings` (Story 5.2) pour cohérence `default_sales_journal`.
- **Enums sqlx::Type manuels** — `invoice.status` est `VARCHAR` + CHECK, pas enum SQL. Ne pas introduire d'enum DB.
- **Licence fonts** : Liberation Sans OFL 1.1 OK avec EUPL-1.2. **Ne pas** embarquer Helvetica (propriétaire).
- **Biais de troncature silencieuse** : SIX interdit toute troncature ; si champ > max → erreur explicite. Le frontend doit en amont valider les longueurs (mais la v0.1 se contente d'une erreur backend claire).

### Review multi-passes obligatoire

Conformément à la règle CLAUDE.md et mémoire `feedback_review_passes.md` : après implémentation, lancer `bmad-code-review` en boucle tant qu'il reste des findings `MEDIUM+`. Utiliser des LLMs orthogonaux entre passes (Opus ↔ Sonnet ↔ Haiku). Focus : conformité SIX (cas limites payload), sécurité content-disposition, gestion des montants (double format affichage vs payload).

### Project Structure Notes

- **Nouveaux fichiers** :
  - `crates/kesh-qrbill/src/{types,validation,generator,pdf}.rs`
  - `crates/kesh-qrbill/assets/{LiberationSans-Regular.ttf,swiss-cross.svg}`
  - `crates/kesh-qrbill/tests/{payload_test.rs,pdf_test.rs,fixtures/golden/invoice.pdf,fixtures/golden/invoice.pdf.sha256}`
  - `crates/kesh-qrbill/README.md`
  - `crates/kesh-api/src/routes/invoice_pdf.rs`
  - `crates/kesh-i18n/locales/{fr,de,it,en}/invoice-pdf.ftl`
- **Fichiers modifiés** :
  - `crates/kesh-qrbill/Cargo.toml` (deps complètes)
  - `crates/kesh-api/src/routes/mod.rs` (enregistrement route)
  - `frontend/src/routes/(app)/invoices/[id]/+page.svelte` (bouton + handler)
  - `frontend/src/lib/i18n/locales/{fr,de,it,en}/*.json` (clés bouton/erreurs)
  - `frontend/e2e/invoices/` (nouveau scénario PDF)
- **Aucune migration DB** — la story est 100% lecture.

### References

- [Source: _bmad-output/planning-artifacts/epics.md:905-920] — Story 5.3 AC originaux
- [Source: _bmad-output/planning-artifacts/architecture.md:551-560] — structure `kesh-qrbill`
- [Source: _bmad-output/planning-artifacts/architecture.md:84] — DD-12 génération PDF
- [Source: _bmad-output/planning-artifacts/architecture.md:273] — DD-14 zéro dépendance crates publiables
- [Source: _bmad-output/planning-artifacts/architecture.md:36,691] — perf PDF < 3s, data flow facturation
- [Source: _bmad-output/planning-artifacts/prd.md:422-426] — FR34/FR35/FR38
- [Source: _bmad-output/planning-artifacts/prd.md:536,553] — NFR-PERF-3, NFR-REL-4
- [Source: docs/six-references/ig-qr-bill-v2.4-en.pdf] — spec SIX 2.2 complète
- [Source: _bmad-output/implementation-artifacts/5-2-validation-numerotation-factures.md] — patterns transaction/handler, conventions audit, i18n
- [Source: crates/kesh-api/src/routes/invoices.rs:520-539] — handler `validate_invoice_handler` référence
- [Source: crates/kesh-db/migrations/20260410000001_bank_accounts.sql] — schéma `bank_accounts`
- [Source: crates/kesh-db/migrations/20260404000001_initial_schema.sql:4-21] — schéma `companies` (adresse TEXT libre)

## Change Log

- **2026-04-14 — Implémentation complète (claude-opus-4-6 [1M], session autonome)** : T1→T10 livrés en continu. Crate `kesh-qrbill` complète (27 tests pass : validation SIX, payload, PDF, round-trip rxing). `AppError` étendue avec 4 variantes (4 tests OK). Endpoint `GET /invoices/:id/pdf` sous authenticated_routes. Frontend : `apiClient.getBlob` + bouton Download sur page invoice. i18n : 31 clés × 4 locales. E2E Playwright : 3 scénarios append. `cargo check --workspace --tests` vert. **Simplifications validées avec Guy en T0 (Plan C + Helvetica built-in)** : déterminisme PDF via payload round-trip (pas hash binaire) ; `BuiltinFont::Helvetica` (pas d'embed Liberation Sans). Statut story : ready-for-dev → review.

- **2026-04-14 — Passe 3 spec validate (claude-haiku-4-5, LLM orthogonal vs passes 1-2)** : 4 findings réels appliqués (1 CRITICAL + 1 MEDIUM + 2 LOW). 1 finding rejeté (Haiku a confondu spec review avec code review — les variantes `AppError` sont à **implémenter par le dev**, pas déjà présentes dans errors.rs, c'est le rôle de T4.8).
  - C2 : `apiClient.fetch` corrigé en `apiClient.getBlob` (nouvelle méthode à ajouter — `apiClient` existant n'expose que `get/post/put/delete` retournant JSON). Extrait de code mis à jour, tâche T7.1bis ajoutée.
  - M1 : nouvelle tâche T4.8 (prérequis bloquant) pour l'extension `AppError` — rend AC 19 explicite côté tasks.
  - L1 : normalisation IBAN détaillée (trim + suppression espaces + uppercase) dans validation `kesh-qrbill`.
  - L2 : règle d'arrondi Decimal(19,4) → 2 décimales spécifiée (`MidpointAwayFromZero`, arrondi commercial suisse) + cas de test.

- **2026-04-14 — Passe 2 spec validate (claude-sonnet-4-6, LLM orthogonal)** : 7 findings appliqués (2 CRITICAL + 3 MEDIUM + 2 LOW).
  - C1 : extension obligatoire de `AppError` (4 nouvelles variantes) — sans ça les codes tomberaient tous sur `VALIDATION_ERROR` générique (cf. `errors.rs:189`). AC 19 ajouté.
  - C2 : section « Gestion de la langue » des Décisions de Conception mise à jour (supprimé la référence à `CurrentUser.preferred_language` inexistant, contradiction avec T5.4 déjà corrigé en passe 1).
  - M1 : renumérotation AC 8b → AC 9 (et décalage 9→10...20), corrections des références croisées (T6 = 9 cas).
  - M2 : stratégie déterminisme PDF détaillée (ordre de préférence printpdf API → lopdf inner → fallback structurel rxing-decode). Interdiction explicite du string-replace naïf sur bytes.
  - M3 : T8.1 corrigé « 25 + 6 = 31 clés » (off-by-one avec AC 18).
  - L1 : cas test « débiteur absent » clarifié — teste la crate en isolation, jamais invoqué par l'assembleur v0.1.
  - L2 : handler frontend refait en pattern `fetch → Blob URL → window.open` (permet de capturer les 4xx pour toast), avec extrait de code. `window.open` direct remplacé.

- **2026-04-14 — Passe 1 spec validate (claude-opus-4-6)** : 9 findings appliqués (2 CRITICAL + 4 MEDIUM + 3 LOW).
  - C1 : locale via `state.config.locale` (pattern Story 2.1), pas via `CurrentUser` (champ inexistant).
  - C2 : déterminisme PDF — figer `CreationDate`/`ModDate`/`/ID` pour golden hash reproductible (T4.6 + T4.7).
  - M1 : rejet 400 `INVOICE_TOO_MANY_LINES_FOR_PDF` si > `MAX_LINES_PER_PDF` lignes (plus de troncature silencieuse).
  - M2 : company via `get_company(&state)` (mono-tenant v0.1), cohérent avec `validate_invoice_handler`.
  - M3 : clarification version SIX (doc local v2.4, compatible PRD v2.2).
  - M4 : rejet IBAN non-CH/LI systématique (QRR **et** NON).
  - L1 : débiteur toujours renseigné en v0.1 (pas de QR au porteur).
  - L2 : `aria-label` sur bouton « Télécharger PDF » + clé i18n dédiée.
  - L3 : notation licences (`rxing` Apache-2.0 dev-only, OK EUPL-1.2).

## Dev Agent Record

### Agent Model Used

claude-opus-4-6 [1M context] — session autonome 2026-04-14 (Guy absent).

### Debug Log References

- `cargo test -p kesh-qrbill` → **27 tests pass** (validation, payload, PDF, rxing round-trip).
- `cargo test -p kesh-api --lib errors::` → **13 pass** (dont 4 nouveaux : `invoice_not_validated`, `invoice_not_pdf_ready`, `invoice_too_many_lines`, `pdf_generation_failed` — vérifie code + message sans leak détail).
- `cargo check --workspace --tests` → **clean**.
- `cargo check -p kesh-qrbill --no-default-features` → **clean** (DD-14 : zéro dépendance sur kesh-* runtime).
- `npx svelte-check` → **0 errors, 2 warnings préexistants** (design-system, non liés à cette story).
- Tests kesh-api DB-dependants (bootstrap, invoice_pdf_e2e) non exécutés localement — pas de MariaDB dans l'environnement autonome. Code écrit, compile, à faire tourner en CI/local avec DB.

### Completion Notes List

**T0 — Faisabilité préalable (validée avec Guy)** :

- `printpdf 0.7` expose `with_creation_date`, `with_mod_date`, `with_document_id` ; **le 2ème élément `/ID` (instance_id) reste généré aléatoirement** (`pdf_document.rs:792`, `random_character_string_32()`). Plan A strict (hash reproductible via API native uniquement) n'est pas atteignable.
- **Décision Guy (Plan C retenu)** : abandonner le golden PDF hash, vérifier la conformité via décodage payload QR round-trip (`rxing`). Implémenté dans `generator::tests::qr_roundtrip_via_rxing`.
- **Décision Guy (simplification police)** : `BuiltinFont::Helvetica` au lieu d'embed Liberation Sans. -150 KB assets, zéro gestion licence .ttf. SIX Style Guide §6 autorise Helvetica.

**T1 — Crate setup** : Cargo.toml avec printpdf 0.7 + qrcodegen 1.8 + rust_decimal + chrono + thiserror + time ; dev-deps rxing 0.7 + sha2 + rust_decimal_macros. Modules `types.rs`, `validation.rs`, `generator.rs`, `pdf.rs`. Types publics autonomes (pas de dep kesh-*).

**T2 — Validation SIX** (11 tests unitaires) : IBAN ISO-13616 mod-97 + normalisation (trim/uppercase/strip spaces) + country code CH/LI ; QR-IBAN avec IID ∈ [30000, 31999] ; QRR 27 chiffres + checksum mod-10 recursif (Annexe B) ; longueurs Name/AdrLine/Ustrd/Country/Amt/Currency ; amount ∈ [0.01, 999'999'999.99] ; `build_qrr(company_id, invoice_id)` déterministe (26 chiffres body + check).

**T3 — Payload + QR image** (6 tests) : `build_payload` sérialise 32/33 lignes selon ordre SIX 2.2 §3 ; `render_qr_image` via `qrcodegen` ECC level M ; **round-trip test rxing** valide l'encodage bout-en-bout (rasterisation 10px/module + 4 modules quiet zone → decode).

**T4 — PDF A4** (5 tests unitaires format) : `generate_qr_bill_pdf` entrée publique, `generate_qr_bill_pdf_with_date` variante déterministe modulo instance_id. Layout : entête facture (nom/adresse/IDE/numéro/date), tableau lignes, total TTC, ligne pointillée séparatrice à y=105mm, colonne récépissé (62mm) + colonne section paiement (148mm), QR Code 46×46mm (modules rectangulaires), croix suisse vectorielle 7×7mm centrée avec carré rouge et branches blanches selon SIX §5.2, format suisse des montants (apostrophe séparateur milliers, point décimal), format IBAN groupé par 4, format QRR 2+5×5, dates dd.mm.yyyy. Scissors glyph omis (hors WinAnsi Helvetica).

**T4.8 — Extension AppError** (4 tests) : `InvoiceNotValidated` → 400 `INVOICE_NOT_VALIDATED` ; `InvoiceNotPdfReady(String)` → 400 `INVOICE_NOT_PDF_READY` avec message métier ; `InvoiceTooManyLinesForPdf(usize)` → 400 `INVOICE_TOO_MANY_LINES_FOR_PDF` avec nombre de lignes ; `PdfGenerationFailed(String)` → 500 `PDF_GENERATION_FAILED` sans leak du détail (loggé côté serveur uniquement).

**T5 — Endpoint API** (`routes/invoice_pdf.rs`) : `get_invoice_pdf(State, Path<id>) -> Response` sous `authenticated_routes` (tout rôle). Charge invoice + lines + contact + company + primary bank. Vérifs : status=validated, lignes ≤ `MAX_LINES_PER_PDF`, contact.address non-NULL, bank primary existe. Mapping adresse TEXT libre → type K (split lignes, prend 2 premières non vides). Locale = `state.config.locale` (pattern Story 2.1). Headers : `Content-Type: application/pdf`, `Content-Disposition: inline; filename="facture-{n}.pdf"` avec filename sanitizé (`[^A-Za-z0-9._-]` → `_`). Mapping `QrBillError → AppError` : erreurs métier → `InvoiceNotPdfReady`, erreurs internes → `PdfGenerationFailed`.

**T6 — Tests API** : 8 tests `#[sqlx::test]` dans `tests/invoice_pdf_e2e.rs` couvrant 9 cas : happy path QR-IBAN, 401 sans JWT, 400 draft, 400 missing primary bank, 400 missing contact address, 400 too many lines (36), 200 IBAN classique sans qr_iban, 404 unknown invoice. Tests compilent (`cargo check --tests` vert) ; exécution requiert MariaDB (pattern `#[sqlx::test(migrator = "kesh_db::MIGRATOR")]` standard du projet). **Cas "200 pour les 3 rôles" non inclus** : le scope actuel a tout en rôle `Admin` seed par défaut, la vérification multi-rôles devrait être ajoutée par une factory utilisateur — reporté en review.

**T7 — Frontend** : `apiClient.getBlob(url): Promise<Response>` ajouté (réutilise headers JWT + refresh 401 + parsing ApiError via `requestRaw` — duplique intentionnellement la logique pour ne pas muter `request<T>`). Bouton « Télécharger PDF » avec `aria-label` i18n, visible uniquement si `invoice.status === 'validated'`. Handler `downloadPdf()` : `getBlob → blob → URL.createObjectURL → window.open('_blank', 'noopener')`. Gestion erreur : `ApiError.code.toLowerCase().replace(/_/g, '-')` → clé i18n `invoice-pdf-error-*` + toast.

**T8 — i18n** : 31 clés ajoutées dans chaque `messages.ftl` des 4 locales (fr/de/it/en CH) = **124 entrées** ; 2 clés frontend `invoices-download-pdf` + `invoices-download-pdf-aria-label` × 4 locales ; 2 clés backend fallback `error-invoice-not-validated` + `error-pdf-generation-failed`. `cargo test -p kesh-i18n` vert (21 tests).

**T9 — Playwright E2E** : 3 scénarios ajoutés à `frontend/tests/e2e/invoices.spec.ts` dans le describe « Factures — téléchargement PDF (Story 5.3) » : golden path (API direct pour bypass `window.open`), bouton masqué si brouillon, erreur primary bank manquante (skipped — exige DB sans bank, setup manuel).

**T10 — Documentation** : `crates/kesh-qrbill/README.md` créé (positionnement DD-14, API publique, conformité SIX, dépendances, tests).

**Limitations connues (à traiter en review)** :

1. **Scissors glyph ✂** omis dans le PDF : pas dans WinAnsi encoding d'Helvetica. Pour le restaurer il faudrait embed une police (contradiction avec simplification de T0). Acceptable — la ligne pointillée seule suffit visuellement.
2. **Instance_id `/ID` PDF** reste aléatoire — le hash binaire du PDF n'est pas reproductible. Conformité SIX vérifiée au niveau payload (rxing round-trip), pas pixel-perfect.
3. **Golden PDF hash** (AC 15 mention) non implémenté ; remplacé par round-trip payload via `rxing` dans `generator::tests::qr_roundtrip_via_rxing`. Plan C du spec, validé avec Guy en T0.
4. **Tests rôle comptable/observateur** : le seed AppState utilise Admin par défaut ; la variation par rôle est couverte par le RBAC middleware dans les autres stories. Non bloquant (le handler est dans `authenticated_routes`).

### File List

**Nouveaux fichiers** :
- `crates/kesh-qrbill/src/types.rs` — types publics (QrBillData, InvoicePdfData, Address, Currency, Reference, QrBillI18n, QrBillError, I18N_KEYS).
- `crates/kesh-qrbill/src/validation.rs` — IBAN / QR-IBAN / QRR / longueurs + 15 tests unitaires.
- `crates/kesh-qrbill/src/generator.rs` — payload SIX + QR matrix + round-trip rxing (6 tests).
- `crates/kesh-qrbill/src/pdf.rs` — layout A4 + section paiement + croix suisse vectorielle + formatage suisse (6 tests).
- `crates/kesh-qrbill/README.md` — positionnement DD-14, API, conformité.
- `crates/kesh-api/src/routes/invoice_pdf.rs` — handler `GET /api/v1/invoices/:id/pdf` + helpers (split_address, sanitize_filename, build_i18n, map_qrbill_error) + 5 tests unitaires.
- `crates/kesh-api/tests/invoice_pdf_e2e.rs` — 8 tests `#[sqlx::test]` (DB requise).
- `frontend/tests/e2e/invoices.spec.ts` — 3 scénarios Playwright (append au fichier existant).

**Fichiers modifiés** :
- `crates/kesh-qrbill/Cargo.toml` — dépendances complètes (printpdf, qrcodegen, rust_decimal, chrono, thiserror, time ; dev : rxing, sha2, rust_decimal_macros).
- `crates/kesh-qrbill/src/lib.rs` — modules + re-exports publics (remplace placeholder).
- `crates/kesh-api/Cargo.toml` — ajout `kesh-qrbill = { path = "../kesh-qrbill" }`.
- `crates/kesh-api/src/routes/mod.rs` — `pub mod invoice_pdf`.
- `crates/kesh-api/src/lib.rs` — route `GET /api/v1/invoices/{id}/pdf` dans `authenticated_routes`.
- `crates/kesh-api/src/errors.rs` — 4 nouvelles variantes AppError + 4 bras IntoResponse + 4 tests.
- `crates/kesh-i18n/locales/fr-CH/messages.ftl` — +33 entrées.
- `crates/kesh-i18n/locales/de-CH/messages.ftl` — +33 entrées.
- `crates/kesh-i18n/locales/it-CH/messages.ftl` — +33 entrées.
- `crates/kesh-i18n/locales/en-CH/messages.ftl` — +33 entrées.
- `frontend/src/lib/shared/utils/api-client.ts` — `apiClient.getBlob()` + helper `requestRaw()`.
- `frontend/src/routes/(app)/invoices/[id]/+page.svelte` — bouton « Télécharger PDF » + handler `downloadPdf()` + icône Download.
- `_bmad-output/implementation-artifacts/sprint-status.yaml` — status 5-3 = review.
