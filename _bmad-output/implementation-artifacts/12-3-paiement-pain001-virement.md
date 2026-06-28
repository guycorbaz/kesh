# Story 12.3: Paiement pain.001 (mode virement, flux deux temps)

Status: ready-for-dev

<!-- Note: Validation is optional. Run validate-create-story for quality check before dev-story. -->
<!-- SPEC UMBRELLA — candidate split 12-3a..e (cf. §"Découpage / split candidate"). Patron : épopées 12-1 / 12-2 / 17-3 / 17-4 / 18-1. -->

## Story

As a comptable PME utilisant Kesh,
I want sélectionner plusieurs factures fournisseurs ouvertes à régler par virement, générer un fichier de paiement **pain.001** que j'importe dans l'e-banking de ma banque, puis confirmer le lot une fois le virement exécuté,
so that je paie mes fournisseurs sans ressaisir les IBAN/montants dans l'e-banking, et la comptabilité enregistre automatiquement les règlements à la confirmation.

## Contexte & source

- Issue GitHub **#191** « Factures fournisseurs & paiements ». 12-3 = volet **pain.001 (mode virement)**.
- Suite directe de **12-2** (factures fournisseurs & règlement binaire, mergée PR #192, squash `90f8847`). 12-2 a livré l'entité `supplier_invoices` (statut `open`/`paid`/`cancelled`), le règlement binaire immédiat, et `bank_accounts` (Epic 8) comme compte source de virement.
- Mémoire de design : `project_epic_12_supplier_invoices_design.md` (Guy 2026-06-27) — « pain.001.001.09.ch.03 = export du seul virement bancaire » ; « flux deux temps sélection→lot→payé ».
- Cible : **v0.4**. Le scan QR-facture (12-4) est **hors scope**.

## Objet métier

Standard **ISO 20022 `pain.001.001.09`** (CustomerCreditTransferInitiation), variante suisse **Swiss Payment Standards / SIX Implementation Guidelines** (dite « ch.03 », même namespace XSD `urn:iso:std:iso:20022:tech:xsd:pain.001.001.09`). Le fichier décrit un ordre de virement groupé :
- **Débiteur** = la société (compte bancaire source choisi : `bank_account.iban`).
- **Créanciers** = chaque facture fournisseur sélectionnée : IBAN **ou** QR-IBAN du créancier + référence (QRR si QR-IBAN, sinon libre) + montant TTC.

**Flux deux temps** (décision Guy) :
1. **Génération du lot** : l'utilisateur sélectionne N factures fournisseurs `open` réglables par virement (ayant des coordonnées de paiement IBAN/QR-IBAN), choisit le **compte bancaire source** débiteur et une date d'exécution souhaitée → Kesh crée un **lot de paiement** et produit le **fichier XML pain.001** téléchargeable. **Aucune écriture comptable n'est postée à ce stade** (le virement n'est pas encore exécuté).
2. **Confirmation du lot** : après import dans l'e-banking et exécution, l'utilisateur **confirme** le lot dans Kesh → pour chaque facture du lot, l'écriture de règlement `D 2000 créanciers / C compte bancaire source` est postée (réutilise la logique `supplier_invoices::pay` mode virement) et la facture passe `paid`. Le lot peut aussi être **annulé** avant confirmation (les factures redeviennent librement réglables).

## Acceptance Criteria

**Sélection & génération du lot (temps 1)**

1. Un Comptable+ peut **créer un lot de paiement** en fournissant : le `bankAccountId` source (compte société à débiter, doit avoir un `journal_account_id` configuré — cf. 12-2), une date d'exécution souhaitée (`requestedExecutionDate`), et une liste de `supplierInvoiceId` à régler par virement.
2. Chaque facture sélectionnée est validée individuellement (**pattern batch FailedProposal**, cf. CLAUDE.md) : refus per-facture (dans `failed[]`, HTTP 200) si la facture n'existe pas / pas `open` / n'a **ni** `creditor_iban` **ni** `creditor_qr_iban` / déjà engagée dans un autre lot non confirmé / company étrangère. Les factures valides vont dans `accepted[]`.
3. Le lot persiste les factures acceptées (table `payment_batches` + `payment_batch_items`) au statut lot **`generated`**, avec le compte source, la date d'exécution, et le total. Les factures restent **`open`** (PAS de nouveau statut sur `supplier_invoices` — cf. DC1) mais sont **verrouillées** : une facture déjà dans un lot `generated` ne peut être ni re-sélectionnée dans un autre lot, ni réglée directement (`pay`/`cancel` de 12-2 refusent), tant que le lot n'est pas confirmé ou annulé.
4. La génération produit un **fichier XML `pain.001.001.09`** valide et bien formé (namespace SIX), téléchargeable (`GET …/pain001`, `Content-Type: application/xml`, nom de fichier `pain001-{batchId}-{date}.xml`). Structure : `GrpHdr` (MsgId unique, CreDtTm, NbOfTxs, CtrlSum, InitgPty=nom société) + un `PmtInf` (PmtMtd=TRF, ReqdExctnDt, Dbtr=société, DbtrAcct=IBAN source, ChrgBr=SLEV) contenant N `CdtTrfTxInf` (EndToEndId, InstdAmt Ccy=CHF, Cdtr=nom fournisseur, CdtrAcct=IBAN **ou** QR-IBAN, RmtInf : `Strd/CdtrRefInf` QRR si QR-IBAN sinon `Ustrd` référence libre). Montants en CHF, format ISO `.` décimal, `NbOfTxs`/`CtrlSum` cohérents avec les transactions.
5. Les IBAN/QR-IBAN/QRR sont validés via `kesh-qrbill::validation` (réutilisé) AVANT inclusion ; une coordonnée invalide → la facture tombe en `failed[]` (AC2), pas un XML corrompu.

**Confirmation / annulation du lot (temps 2)**

6. Un Comptable+ peut **confirmer** un lot `generated` (`POST …/confirm`, avec une date de règlement effective `paymentDate`). Pour chaque facture du lot : poste l'écriture de règlement `D 2000 / C journal_account_id du compte source` via la logique virement de 12-2, passe la facture `paid`, renseigne les champs settlement (`settlement_type='bank_transfer'`, `settlement_bank_account_id`, `settlement_journal_entry_id`, `paid_at`). Le lot passe **`confirmed`**. Atomique : soit toutes les écritures du lot sont postées, soit aucune (rollback) — un lot confirmé est un tout.
7. Un Comptable+ peut **annuler** un lot `generated` (`POST …/cancel`) → lot **`cancelled`**, les factures sont **déverrouillées** (redeviennent librement réglables/annulables/re-sélectionnables). Aucune écriture comptable (rien n'avait été posté).
8. Un lot `confirmed` ou `cancelled` est **immuable** (pas de re-confirmation/ré-annulation). Refus en erreur métier (pas 500).
9. Refus de confirmation (erreur métier) si la date de règlement tombe hors exercice ouvert, ou si un compte cible est devenu inactif.

**CRUD / liste / cohérence**

10. Liste paginée des lots (scopée société, anti-IDOR) : id, date de création, date d'exécution souhaitée, compte source, nb de factures, total, statut. Détail d'un lot : en-tête + factures incluses + (si confirmé) liens vers les écritures de règlement + re-téléchargement du XML.
11. Toutes les mutations (génération, confirmation, annulation) écrivent un **audit log** (`payment_batch.generated` / `.confirmed` / `.cancelled`).
12. **i18n** : libellés FR + clés FTL (FR/DE/IT/EN) au moins titres + nav. Item sidebar « Paiements » (ou sous « Factures fournisseurs »).
13. **Tests** : unitaires générateur pain.001 (structure XML, NbOfTxs/CtrlSum, QR-IBAN→QRR vs IBAN→Ustrd, échappement XML) + golden file pain.001 ; intégration repo (génération lot + verrouillage factures + confirmation poste les écritures + annulation déverrouille + refus batch) ; E2E (sélection→génération→download→confirmation→paid) ; quality gate vert (workspace serial + frontend + i18n). **Vérifier les compteurs en dur** `admin_full_export_e2e` (data_count) + `migrations_upgrade_path` (count + fenêtre) — 2 nouvelles tables + 1 migration (cf. leçon 12-2 `feedback_cargo_test_pipe_masks_exit`).

## Points de décision (DC) — à figer au `validate`

- **DC1 — Modélisation du statut « pending » : NON-BREAKING via lot** [✅ FIGÉ Guy 2026-06-28] : **NE PAS** ajouter de statut `payment_pending` à `supplier_invoices` (modifier la contrainte `chk_supplier_invoices_status` = migration **breaking** → bump `kesh_version_min_required`, dangereux car **la prod NAS de Guy tourne déjà v0.3.2 avec données**). À la place : la facture reste `open` ; l'état « en cours de paiement » = **appartenance à un lot `generated`** (JOIN `payment_batch_items`). Le verrouillage (AC3) empêche double-paiement. Avantage : migration non-breaking (CREATE TABLE seul), pas de bump, pattern propre. *Inconvénient : les guards de `pay`/`cancel` 12-2 doivent vérifier l'absence de lot actif (ajout cross-table).* À confirmer Guy.
- **DC2 — Variante pain.001 & namespace** [✅ FIGÉ Guy 2026-06-28] : `pain.001.001.09`, namespace `urn:iso:std:iso:20022:tech:xsd:pain.001.001.09`. La « ch.03 » = version des Swiss Implementation Guidelines (pas un namespace distinct). Portée v0.4 : types de paiement **1** (IBAN domestique CH/LI, RmtInf Ustrd) et **3** (QR-IBAN + référence QRR, RmtInf Strd) **uniquement**. Types 2/2.2 (ESR/IS, étranger SEPA/SWIFT) **hors scope** (documenter limitation).
- **DC3 — `DbtrAgt`/`CdtrAgt` (BIC/Clearing)** : SIX autorise `DbtrAgt` avec `FinInstnId/Othr/Id=«NOTPROVIDED»` quand l'IBAN suffit (pas de BIC requis pour le domestique). *Proposition : émettre `Othr/Id=NOTPROVIDED` pour DbtrAgt et omettre CdtrAgt (déductible de l'IBAN créancier en domestique).* À valider vs XSD.
- **DC4 — Génération XML : quick-xml Writer vs string templating** : `quick-xml 0.39` est déjà au workspace (kesh-import, en Reader). *Proposition : utiliser `quick-xml::Writer` (échappement correct, cohérence projet) dans `kesh-payment::pain001`. PAS de concat de String (risque d'injection/échappement).* À confirmer.
- **DC5 — Un PmtInf vs N** : un seul `PmtInf` (même date d'exécution + même débiteur pour tout le lot) suffit en v0.4 (`BtchBookg` omis ou `false`). À confirmer.
- **DC6 — Confirmation atomique vs per-facture** : la confirmation poste N écritures. *Proposition : atomique (tout ou rien) — un lot confirmé est un événement unique, contrairement à la génération (batch FailedProposal per-facture). Si une écriture échoue (ex. exercice clos), rollback total + erreur métier identifiant la facture fautive.* À confirmer (alternative : confirmation partielle avec `failed[]`).
- **DC7 — Emplacement** : générateur pain.001 pur (sans I/O DB) dans **`kesh-payment::pain001`** (crate pré-structurée, `tests/fixtures/` pour golden) ; entités/repo lot dans `kesh-db` ; routes dans `kesh-api`. `kesh-payment` dépendra de `rust_decimal`, `quick-xml`, `chrono` (+ `kesh-qrbill` si réutilise la validation IBAN, sinon valider côté kesh-db avant).

## Tasks / Subtasks (provisoire — à affiner au validate, split probable)

- [ ] **T1 — Générateur pain.001 (kesh-payment)** (AC: 4,5 ; DC2,DC3,DC4,DC5,DC7) : structs d'entrée pures (`Pain001Batch { msg_id, creation_dt, initiating_party, debtor_name, debtor_iban, requested_execution_date, transactions: Vec<Pain001Tx> }`, `Pain001Tx { end_to_end_id, amount, creditor_name, creditor_account (IBAN|QR-IBAN), reference (QRR|Free) }`) + `generate_pain001(batch) -> Result<String, PaymentError>` via `quick-xml::Writer` ; calcul NbOfTxs/CtrlSum ; tests unitaires + golden file `tests/fixtures/pain001_*.xml`.
- [ ] **T2 — Migration & entités lot (kesh-db)** (AC: 3,7 ; DC1) : migration `payment_batches` (company_id FK, bank_account_id FK, status CHECK('generated','confirmed','cancelled'), requested_execution_date, total_amount, msg_id, created_at…) + `payment_batch_items` (payment_batch_id FK CASCADE, supplier_invoice_id FK RESTRICT, UNIQUE(supplier_invoice_id) **partiel/applicatif** pour le verrouillage). **Non-breaking** (CREATE TABLE seul). Audit idempotence + `TABLES_TO_TRUNCATE` + compteurs en dur (AC13).
- [ ] **T3 — Repo lot (kesh-db)** (AC: 1,2,3,6,7,8,9 ; DC1,DC6) : `create_batch` (valide chaque facture, FailedProposal per-facture, verrou) + `confirm_batch` (atomique, réutilise la logique virement `pay`) + `cancel_batch` (déverrouille) + `get`/`list`. Guards cross-table dans `supplier_invoices::pay`/`cancel` (refus si facture dans lot `generated`).
- [ ] **T4 — Routes API (kesh-api)** (AC: 1,4,6,7,8,10,11) : `POST /api/v1/payment-batches` (créer, `{accepted, failed}`) + `GET …/{id}/pain001` (download XML) + `POST …/{id}/confirm` + `POST …/{id}/cancel` + `GET …` liste + `GET …/{id}` détail. Comptable+ mutations / lectures tout rôle.
- [ ] **T5 — Frontend (kesh frontend)** (AC: 1,4,6,7,10,12) : feature `payment-batches` + page de sélection des factures `open` virement-réglables + choix compte source + génération + download + page liste/détail lot + actions confirmer/annuler + sidebar + i18n.
- [ ] **T6 — Tests + doc** (AC: 13) : intégration repo + E2E + golden pain.001 + CHANGELOG/README + manuel. Quality gate Test Locally First (exit code vérifié, PAS de pipe masquant — leçon 12-2).

## Découpage / split candidate

**Story umbrella — CANDIDATE SPLIT 12-3a..e** (générateur XML kesh-payment / migration+entités lot / repo flux deux temps + guards / routes / frontend+tests). Le `validate` tranchera (pattern 12-2/17-3/18-1).

## Dev Notes

### Réutilisation 12-2 (mergée, ground-truth)
- `crates/kesh-db/src/entities/supplier_invoice.rs` — `SupplierInvoice` (champs `creditor_iban`/`creditor_qr_iban`/`payment_reference`/`expected_payment_amount`/`total_amount` TTC/`status`) + `SettlementChoice::BankTransfer { bank_account_id }`.
- `crates/kesh-db/src/repositories/supplier_invoices.rs` — `pay(pool, company_id, id, choice, payment_date, user_id)` : la **logique virement** (D 2000 relu de la ligne crédit achat / C `bank_account.journal_account_id`, solde 2000=0) est exactement ce que `confirm_batch` doit réutiliser per-facture. ⚠️ Ajouter un guard « facture dans lot `generated` » dans `pay`/`cancel`.
- `crates/kesh-db/migrations/20260628000001_supplier_invoices.sql` — contrainte `chk_supplier_invoices_status CHECK (status IN ('open','paid','cancelled'))`. **NE PAS la modifier** (DC1).

### pain.001 — structure normative (pain.001.001.09, SIX)
```
Document(xmlns=urn:iso:std:iso:20022:tech:xsd:pain.001.001.09)
 └ CstmrCdtTrfInitn
    ├ GrpHdr { MsgId, CreDtTm, NbOfTxs, CtrlSum, InitgPty{Nm} }
    └ PmtInf { PmtInfId, PmtMtd=TRF, NbOfTxs, CtrlSum, ReqdExctnDt{Dt},
               Dbtr{Nm}, DbtrAcct{Id{IBAN}}, DbtrAgt{FinInstnId{Othr{Id=NOTPROVIDED}}}, ChrgBr=SLEV,
               CdtTrfTxInf[1..n] {
                  PmtId{EndToEndId}, Amt{InstdAmt Ccy=CHF},
                  Cdtr{Nm}, CdtrAcct{Id{IBAN|QR-IBAN}},
                  RmtInf{ Strd{CdtrRefInf{Tp{CdOrPrtry{Prtry=QRR}},Ref}} | Ustrd } } }
```
- Type 3 (QR-IBAN) → `RmtInf/Strd/CdtrRefInf` avec QRR 27 chiffres (validé `kesh-qrbill::validation::validate_qrr`).
- Type 1 (IBAN) → `RmtInf/Ustrd` (référence libre, ≤140).

### Réutilisation infra
- `crates/kesh-import/src/camt053/mod.rs` — usage `quick-xml` (Reader/NsReader). Pour 12-3 : **Writer** (`quick_xml::Writer`, `events::{BytesStart, BytesEnd, BytesText, Event}`). Pas de Writer existant → premier du projet (DC4).
- `crates/kesh-qrbill/src/validation.rs` — `validate_iban`/`validate_qr_iban`/`normalize_iban`/`validate_qrr` réutilisables pour valider les coordonnées avant génération (AC5).
- `crates/kesh-api/src/routes/reconciliation.rs:150` — `FailedProposal { bank_transaction_id, error_code, details }` + `accept_batch`→`{accepted, failed}` : **modèle canonique** pour `POST /payment-batches` (AC2). Adapter l'identifiant business → `supplier_invoice_id`.
- `crates/kesh-db/src/repositories/bank_accounts.rs:101` — `find_by_id_for_company` → `bank_account.iban` (débiteur) + `journal_account_id` (contrepartie règlement, requis).
- `crates/kesh-payment/` — crate **pré-structurée** : `src/pain001/` (placeholder) + `tests/fixtures/` (golden). Deps à ajouter : `rust_decimal`, `quick-xml`, `chrono`, `thiserror` (+ `kesh-qrbill` éventuel).

### Conventions projet
- **Pattern batch FailedProposal** (CLAUDE.md) : génération du lot = N factures, chacune peut échouer → `{accepted, failed}` HTTP 200. AppError global réservé 401/403/400/500.
- **Migration non-breaking** (DC1) : CREATE TABLE seul → pas de bump `kesh_version_min_required`. Audit idempotence obligatoire.
- **Compteurs en dur** (leçon 12-2) : nouvelle migration → bumper `migrations_upgrade_path` (count + fenêtre `total-N`) ; nouvelles tables → bumper `admin_full_export_e2e` (`data_count`) + `TABLES_TO_TRUNCATE`.
- **Test Locally First** : exit code vérifié explicitement (redirection fichier, PAS `cargo test | grep` — cf. `feedback_cargo_test_pipe_masks_exit`).
- **Branche** : `story/12-3-paiement-pain001-virement` (déjà créée). Commit après chaque étape BMAD. `(refs #191)`.

### References
- [Source: issue #191] + [memory project_epic_12_supplier_invoices_design].
- [Source: crates/kesh-db/src/repositories/supplier_invoices.rs] — `pay` virement réutilisé par `confirm_batch`.
- [Source: crates/kesh-api/src/routes/reconciliation.rs:150] — FailedProposal.
- [Source: crates/kesh-qrbill/src/validation.rs] — validation IBAN/QR-IBAN/QRR.
- [Source: crates/kesh-import/src/camt053/mod.rs] — pattern quick-xml.
- ISO 20022 pain.001.001.09 + Swiss Payment Standards (SIX Implementation Guidelines Credit Transfer).

## Dev Agent Record

### Agent Model Used

(à remplir au dev-story — Opus 4.8 recommandé : XSD/standard externe + scope cross-stack)

### Debug Log References

### Completion Notes List

### File List
