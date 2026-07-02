# Story 12.4: Scan / import du QR-facture (pré-remplissage)

Status: done

<!-- Note: Validation is optional. Run validate-create-story for quality check before dev-story. -->
<!-- SPEC UMBRELLA — candidate split 12-4a..c (cf. §"Découpage"). Patron : épopées 12-1 / 12-2 / 12-3. DERNIER volet de #191. -->

## Story

As a comptable PME utilisant Kesh,
I want scanner le QR-code d'une facture fournisseur reçue (QR-bill suisse) pour pré-remplir automatiquement les coordonnées de paiement d'une nouvelle facture fournisseur,
so that je ne ressaisis pas l'IBAN, la référence et le montant à la main (source d'erreurs), et je gagne du temps à l'enregistrement.

## Contexte & source

- Issue GitHub **#191** « Factures fournisseurs & paiements » — **dernier volet** (12-2 factures + règlement et 12-3 pain.001 déjà mergées : PR #192 `90f8847`, PR #193 `27e7d8e`). **12-4 conclut #191.**
- Mémoire de design : `project_epic_12_supplier_invoices_design.md` — « (B) scan/import du QR-facture voulu rapidement après (réduire erreurs de saisie) ».
- Cible : **v0.4**.

## Objet métier

Le **Swiss QR Code** (SPC — Swiss Payments Code) imprimé sur les factures suisses (QR-bill SIX 2.2) encode un payload texte structuré (en-tête `SPC\n0200\n1\n` + champs). Kesh **génère** déjà ce payload (`kesh-qrbill::generator`) ; 12-4 ajoute le **parseur inverse** (texte SPC → coordonnées) et l'utilise pour **pré-remplir** le formulaire d'enregistrement de facture fournisseur (12-2).

**Méthode d'entrée figée (Guy 2026-06-28)** : **upload d'une image** du QR (photo/capture) → **décodage dans le navigateur** (lib JS `jsQR` sur `<canvas>`, PAS la caméra) → le **texte SPC** décodé est envoyé au backend qui le **parse**. ⚠️ Pas de caméra (`getUserMedia` est secure-context-only → KO sur le NAS HTTP de Guy, cf. `feedback_no_secure_context_apis_http_lan`). Pas de décodage image côté backend (`rxing` reste dev-only).

## Acceptance Criteria

**Parseur SPC (kesh-qrbill)**

1. Une fonction `parse_spc_payload(text) -> Result<ScannedQrBill, QrBillError>` parse un payload SPC : valide l'en-tête (`SPC` ligne 0, version `0200` ligne 1), extrait **IBAN/QR-IBAN** (ligne 3), **nom créancier** (ligne 5) + adresse, **montant** (ligne 18, peut être vide = montant ouvert), **devise** (ligne 19, CHF/EUR), **type de référence** (ligne 27 : `QRR`/`SCOR`/`NON`) + **valeur de référence** (ligne 28), **message non structuré** (ligne 29). Robuste aux champs de fin absents (`AltPmtInf` omis). Inverse exact de `generator.rs` (mêmes index de lignes).
2. Le parseur **valide** les coordonnées via les fonctions existantes : `validate_iban`/`validate_qr_iban` (selon que l'IBAN est un QR-IBAN, plage IID 30000-31999) et `validate_qrr` si type `QRR`. Un payload mal formé / en-tête invalide / IBAN invalide → `QrBillError` explicite (pas de panic).

**Endpoint de pré-remplissage (kesh-api)**

3. `POST /api/v1/supplier-invoices/scan-qr` (Comptable+) prend `{ spcText: String }` et retourne les coordonnées extraites pour pré-remplir : `{ creditorIban | creditorQrIban (selon plage QR-IID), paymentReference, expectedPaymentAmount, currency, creditorName, creditorAddress, unstructuredMessage }`. **Ne crée RIEN** (pure transformation, lecture seule). Erreur métier 4xx (pas 500) si le SPC est invalide.
4. Mapping → champs `supplier_invoices` (12-2) : QR-IBAN (plage 30000-31999) → `creditor_qr_iban` + `payment_reference`=QRR ; IBAN classique → `creditor_iban` + `payment_reference`=référence libre/SCOR ; montant → `expected_payment_amount` (informatif, TTC du QR). **Le QR ne contient PAS** le compte de charge ni la ventilation TVA → ces champs restent à saisir manuellement (le pré-remplissage couvre la partie error-prone : IBAN + référence + montant).

**Frontend (pré-remplissage du formulaire)**

5. Le formulaire d'enregistrement de facture fournisseur (12-2, `/supplier-invoices`) gagne un bouton **« Scanner un QR-facture »** → ouvre un sélecteur de fichier image (PNG/JPG). À la sélection : décodage `jsQR` sur `<canvas>` (navigateur) → si un QR est trouvé, POST `scan-qr` → **pré-remplit** les champs coordonnées (IBAN/QR-IBAN, référence, montant attendu) + affiche le **nom du créancier** détecté (indice pour sélectionner/créer le fournisseur). L'utilisateur complète les lignes comptables (compte de charge, HT, TVA) puis enregistre via le flux 12-2 existant.
6. Gestion d'erreur : image sans QR détectable → message « Aucun QR-code détecté » ; QR non-SPC / invalide → message backend. Aucun crash, aucune dépendance secure-context.

**Tests / qualité**

7. **Tests** : unitaires parseur SPC (round-trip generator↔parseur : générer un payload puis le re-parser et vérifier l'égalité des champs ; cas QRR vs SCOR vs NON ; en-tête invalide ; IBAN invalide ; montant ouvert vide) ; intégration endpoint scan-qr (SPC valide → coordonnées, SPC invalide → 4xx) ; unit frontend (helper de mapping) ; E2E optionnel (upload fixture image → pré-remplissage). Quality gate vert. **Pas de nouvelle table ni migration** → compteurs en dur `admin_full_export_e2e`/`migrations_upgrade_path` **inchangés** (vérifier qu'ils ne bougent pas). Test Locally First **exit code vérifié** (pas de `cargo test | grep`, cf. `feedback_cargo_test_pipe_masks_exit`).

## Points de décision (DC)

- **DC1 — Méthode d'entrée** [✅ FIGÉ Guy 2026-06-28] : upload image + décodage navigateur `jsQR` → texte SPC → parseur backend. Pas de caméra (HTTP LAN), pas de `rxing` runtime.
- **DC2 — Dépendance frontend `jsQR`** [✅ FIGÉ par DC1] : ajouter `jsqr` (npm, pur JS, ~30KB, pas de WASM, MIT) au frontend. Décodage local `ImageData`→QR text, aucune API caméra. *(Le dev-story HALTe normalement sur nouvelle dépendance — approbation Guy actée ici.)*
- **DC3 — Emplacement parseur** [proposition] : `kesh-qrbill::parser` (nouveau module `src/parser.rs`), symétrique de `generator.rs`, réutilise `validation.rs`. `ScannedQrBill` = struct allégée (champs nécessaires au pré-remplissage) OU réutiliser `QrBillData` (parse complet). *Proposition : struct dédiée `ScannedQrBill` (le pré-remplissage n'a pas besoin du debtor ni du billing).* À confirmer au validate.
- **DC4 — Création du fournisseur** [proposition] : v0.4 ne crée PAS automatiquement le contact fournisseur depuis le nom du QR (risque de doublons). On **affiche** le nom détecté + on laisse l'utilisateur sélectionner un fournisseur existant ou en créer un. *Auto-match/auto-create = story de suivi.* À confirmer.
- **DC5 — Pré-remplissage des lignes comptables** [proposition] : le QR donne un montant TTC sans ventilation TVA ni compte de charge → on **ne crée PAS de ligne** automatiquement (HT≠TTC, compte inconnu). On pré-remplit `expected_payment_amount` (informatif) + coordonnées. *Une ligne pré-remplie au montant TTC induirait une écriture fausse.* À confirmer.

## Alignement post-12-5 (2026-07-01)

⚠️ **T1 déjà livré par la story 12-5a** (branche `story/12-5-...`, commit `85a235b`) :
`kesh-qrbill::parser::parse_spc_payload(&str) -> Result<ScannedQrBill, QrBillError>`
existe, avec `ScannedQrBill { creditor_iban, is_qr_iban, creditor: ScannedAddress,
amount: Option<Decimal>, currency, reference: ScannedReference (Qrr/Scor/None),
unstructured_message, billing_information }`. C'est exactement la « struct dédiée »
proposée en **DC3 → RÉSOLU (réutilisation, pas de nouveau code parseur)**. 12-4 se
réduit donc à **T2 (endpoint wrapper) + T3 (frontend jsQR) + T4 (doc)**.

**DC3** ✅ réutiliser `ScannedQrBill` (12-5a). **DC4** ✅ pas d'auto-création du
fournisseur (affiche le nom détecté). **DC5** ✅ pas de ligne comptable
pré-remplie (montant TTC informatif seul). Tous confirmés — pas de nouvelle
table/migration. Développé sur la branche `story/12-5-...` (chevauchement de
fichiers avec 12-5 → même PR #200, cf. `feedback_pr_grouping`).

Status: in-progress → review → done

## Tasks / Subtasks

- [x] **T1 — Parseur SPC (kesh-qrbill)** (AC: 1,2 ; DC3) : **LIVRÉ par 12-5a** — `parse_spc_payload` + `ScannedQrBill` présents. 12-4 vérifie/réutilise (aucun nouveau code parseur).
- [x] **T2 — Endpoint scan-qr (kesh-api)** (AC: 3,4) : `POST /api/v1/supplier-invoices/scan-qr` dans `routes/supplier_invoices.rs` (Comptable+), DTO `ScanQrRequest { spc_text }` → `ScanQrResponse { creditorIban/creditorQrIban, paymentReference, expectedPaymentAmount, currency, creditorName, … }`. Lecture seule. Montage `lib.rs`. Tests intégration.
- [x] **T3 — Frontend (pré-remplissage)** (AC: 5,6 ; DC2) : dép `jsqr` ; sur le formulaire `/supplier-invoices`, bouton « Scanner un QR-facture » + `<input type=file accept=image/*>` → décodage `jsQR` (canvas) → `scanQr(spcText)` api → pré-remplit les champs coordonnées + affiche nom créancier. Helper de mapping + test unit. Gestion d'erreur (pas de QR / SPC invalide). i18n FR + clés.
- [x] **T4 — Doc** : CHANGELOG `[Non publié]` (conclut la feature factures fournisseurs & paiements) + README (retirer « scan QR à venir »). Quality gate Test Locally First exit-code vérifié.

## Découpage

**Candidate split 12-4a (parseur kesh-qrbill) / 12-4b (endpoint) / 12-4c (frontend+doc)** — story plus petite que 12-2/12-3 (pas de DB, pas de flux d'état). Le `validate` tranchera (probablement une seule story livrable inline).

## Dev Notes

### Ground-truth réutilisable
- `crates/kesh-qrbill/src/generator.rs:18-90` — **ordre exact des lignes du payload SPC** (en-tête, IBAN ligne 3, créancier type K lignes 4-10, ultimate creditor 7 lignes vides 11-17, montant ligne 18, devise ligne 19, debtor lignes 20-26, référence type/valeur lignes 27-28, message ligne 29, `EPD` ligne 30, billing ligne 31). Le parseur lit ces mêmes index.
- `crates/kesh-qrbill/src/types.rs:13` — `QrBillData { creditor_iban, creditor: Address{name,line1,line2,country}, amount: Option<Decimal>, currency, reference, unstructured_message, … }`. `Reference` enum + `tp_code()` (QRR/SCOR/NON) + `ref_value()`.
- `crates/kesh-qrbill/src/validation.rs` — `validate_iban`/`validate_qr_iban`/`validate_qrr`/`normalize_iban`. QR-IBAN = plage IID 30000-31999 (positions 4-8).
- `crates/kesh-qrbill/Cargo.toml` — `rxing` est **dev-dependency uniquement** (décodage QR image en test) → NE PAS le passer runtime (DC1).
- **Frontend 12-2** : `frontend/src/routes/(app)/supplier-invoices/+page.svelte` (formulaire d'enregistrement inline) — c'est là qu'on ajoute le bouton scan + pré-remplissage. `frontend/src/lib/features/supplier-invoices/{supplier-invoices.api.ts,.types.ts}` — ajouter `scanQr` + types.
- **Entité cible** `supplier_invoices` (12-2) : champs `creditor_iban`/`creditor_qr_iban`/`payment_reference`/`expected_payment_amount` — déjà présents (ajoutés 12-2). 12-4 ne touche PAS la DB.

### Conventions projet
- **Pas de migration / pas de nouvelle table** → les compteurs `admin_full_export_e2e` (data_count 30) et `migrations_upgrade_path` (total 38) **ne doivent PAS changer**. Vérifier qu'aucun n'est touché.
- **Test Locally First** : exit code vérifié (redirection fichier + `$?`, PAS `cargo test | grep`).
- **Issue Tracking** : 12-4 conclut #191 — le commit final peut `closes #191` (avec 12-2/12-3 déjà mergées, 12-4 termine le sous-système).
- **Branche** : `story/12-4-scan-qr-facture` (créée). Commit après chaque étape BMAD.
- **HTTP LAN** : aucune API secure-context (caméra/clipboard) — décodage `jsQR` sur canvas via `<input type=file>` fonctionne en HTTP.

### References
- [Source: issue #191] — dernier volet.
- [Source: crates/kesh-qrbill/src/generator.rs:18] — structure payload SPC (miroir).
- [Source: crates/kesh-qrbill/src/validation.rs] — validations IBAN/QR-IBAN/QRR.
- [Source: frontend/.../supplier-invoices/+page.svelte] — formulaire à enrichir.
- [Source: memory feedback_no_secure_context_apis_http_lan] — pas de caméra sur HTTP NAS.

## Dev Agent Record

### Agent Model Used

(à remplir au dev-story)

### Debug Log References

### Completion Notes List

### File List

## Change Log — code-review

**Dev** : T1 réutilisé (parseur `ScannedQrBill` livré par 12-5a), T2 endpoint `scan-qr` (4 unit + 4 intégration), T3 frontend jsQR + pré-remplissage (helper testé + wiring), T4 doc, E2E réel 1/1 (décodage jsQR navigateur → parse → prefill ; a exposé un bug préexistant `each_key_duplicate` vatRates corrigé `(r.id)`).

**Code-review — CONVERGÉ 2 passes (Sonnet → Haiku)** :
- **Pass 1 (Sonnet, 2 couches)** : 1 MEDIUM + 3 LOW.
  - MEDIUM (correctness) : édition manuelle de l'IBAN après un scan QR-IBAN laissait `fPaymentReference` (QRR) périmé → record incohérent (IBAN classique + réf QRR, pain.001 aval faux). Fix : `oninput` vide QRR + devise + créancier **uniquement** si on quitte le mode QR-IBAN (une réf saisie à la main n'est jamais effacée).
  - LOW-1 devise (EUR/CHF) exposée + affichée ; LOW-2 nom créancier périmé vidé ; LOW-3 garde-fou taille image 15 Mo.
- **Pass 2 (Haiku, 2 couches)** : **0 > LOW**. Tous AC1-7 + DC1-5 vérifiés ground-truth, edge cases couverts (no-QR, non-image, oversize, SPC invalide, édition manuelle, EUR, re-scan, reset), fixes Pass 1 confirmés présents. Aucune hallucination.

**Trend > LOW : 1 (Sonnet) → 0 (Haiku).** Gate : fmt + clippy workspace + suite kesh-api exit-0 (0 régression) + front check 0/lint PASS/343 unit/build + E2E scan 1/1 réel. Développé sur branche `story/12-5` (chevauchement fichiers → PR #200, `feedback_pr_grouping`). **Épopée 12-4 conclut #191 (factures fournisseurs & paiements).**
