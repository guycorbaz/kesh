# Story 12.3: Paiement pain.001 (mode virement, flux deux temps)

Status: review

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
2. Chaque facture sélectionnée est validée individuellement (**pattern batch FailedProposal**, cf. CLAUDE.md) : refus per-facture (dans `failed[]`, HTTP 200) si la facture n'existe pas / pas `open` / n'a **ni** `creditor_iban` **ni** `creditor_qr_iban` / déjà engagée dans un lot `generated` / company étrangère. Si la facture a **les deux** coordonnées → on **préfère QR-IBAN** (M2-1, pas un refus). Les factures valides vont dans `accepted[]`.
3. Le lot persiste les factures acceptées (table `payment_batches` + `payment_batch_items`) au statut lot **`generated`**, avec le compte source, la date d'exécution, et le total. Les factures restent **`open`** (PAS de nouveau statut sur `supplier_invoices` — cf. DC1) mais sont **verrouillées** : une facture déjà dans un lot `generated` ne peut être ni re-sélectionnée dans un autre lot, ni réglée directement (`pay`/`cancel` de 12-2 refusent), tant que le lot n'est pas confirmé ou annulé.
4. La génération produit un **fichier XML `pain.001.001.09`** valide et bien formé (namespace SIX), téléchargeable (`GET …/pain001`, `Content-Type: application/xml`, nom de fichier `pain001-{batchId}-{date}.xml`). Structure : `GrpHdr` (MsgId unique, CreDtTm, NbOfTxs, CtrlSum, InitgPty=nom société) + un `PmtInf` (PmtInfId, PmtMtd=TRF, **NbOfTxs, CtrlSum** [M1], ReqdExctnDt, Dbtr=société, DbtrAcct=IBAN source, `DbtrAgt/Othr/Id=NOTPROVIDED`, ChrgBr=SLEV) contenant N `CdtTrfTxInf` (EndToEndId, InstdAmt Ccy=CHF, Cdtr=nom fournisseur, CdtrAcct=IBAN **ou** QR-IBAN, RmtInf : `Strd/CdtrRefInf` QRR si QR-IBAN sinon `Ustrd` référence libre). Montants en CHF (= `total_amount` TTC de chaque facture [M2-2]), format ISO `.` décimal, `NbOfTxs`/`CtrlSum` cohérents avec les transactions. **Invariant [LOW-2]** : `GrpHdr/NbOfTxs == PmtInf/NbOfTxs` et `GrpHdr/CtrlSum == PmtInf/CtrlSum` (un seul PmtInf, DC5) — assertion dans les tests du générateur.
5. Les IBAN/QR-IBAN/QRR sont validés via `kesh-qrbill::validation` (réutilisé) AVANT inclusion ; une coordonnée invalide → la facture tombe en `failed[]` (AC2), pas un XML corrompu.

**Confirmation / annulation du lot (temps 2)**

6. Un Comptable+ peut **confirmer** un lot `generated` (`POST …/confirm`, avec une date de règlement effective `paymentDate`). Pour chaque facture du lot : poste l'écriture de règlement `D 2000 / C journal_account_id du compte source` via la logique virement de 12-2, passe la facture `paid`, renseigne les champs settlement (`settlement_type='bank_transfer'`, `settlement_bank_account_id`, `settlement_journal_entry_id`, `paid_at`). Le lot passe **`confirmed`**. Atomique : soit toutes les écritures du lot sont postées, soit aucune (rollback) — un lot confirmé est un tout.
7. Un Comptable+ peut **annuler** un lot `generated` (`POST …/cancel`) → lot **`cancelled`**, les factures sont **déverrouillées** (redeviennent librement réglables/annulables/re-sélectionnables). Aucune écriture comptable (rien n'avait été posté).
8. Un lot `confirmed` ou `cancelled` est **immuable** (pas de re-confirmation/ré-annulation). Refus en erreur métier (pas 500).
9. Refus de confirmation (erreur métier, HTTP 422 `{ error, invoiceId }` [M4]) si la date de règlement tombe hors exercice ouvert, si le **compte bancaire source est archivé** (`bank_account.archived = true`) ou si son `journal_account_id` pointe vers un compte `active = false` [M5]. ⚠️ `find_by_id_for_company` ne filtre PAS `archived` → vérif explicite dans la tx.

**CRUD / liste / cohérence**

10. Liste paginée des lots (scopée société, anti-IDOR) : id, date de création, date d'exécution souhaitée, compte source, nb de factures, total, statut. Détail d'un lot : en-tête + factures incluses + (si confirmé) liens vers les écritures de règlement + re-téléchargement du XML.
11. Toutes les mutations (génération, confirmation, annulation) écrivent un **audit log** (`payment_batch.generated` / `.confirmed` / `.cancelled`).
12. **i18n** : libellés FR + clés FTL (FR/DE/IT/EN) au moins titres + nav. Item sidebar **« Paiements fournisseurs »** dans le groupe « Quotidien », **après « Factures fournisseurs »** (cohérence 12-2) [M7].
13. **Tests** : unitaires générateur pain.001 (structure XML, NbOfTxs/CtrlSum, QR-IBAN→QRR vs IBAN→Ustrd, échappement XML) + golden file pain.001 ; intégration repo (génération lot + verrouillage factures + confirmation poste les écritures + annulation déverrouille + refus batch) ; E2E (sélection→génération→download→confirmation→paid) ; quality gate vert (workspace serial + frontend + i18n). **Vérifier les compteurs en dur** `admin_full_export_e2e` (data_count) + `migrations_upgrade_path` (count + fenêtre) — 2 nouvelles tables + 1 migration (cf. leçon 12-2 `feedback_cargo_test_pipe_masks_exit`).

## Points de décision (DC) — à figer au `validate`

- **DC1 — Modélisation du statut « pending » : NON-BREAKING via lot** [✅ FIGÉ Guy 2026-06-28] : **NE PAS** ajouter de statut `payment_pending` à `supplier_invoices` (modifier la contrainte `chk_supplier_invoices_status` = migration **breaking** → bump `kesh_version_min_required`, dangereux car **la prod NAS de Guy tourne déjà v0.3.2 avec données**). À la place : la facture reste `open` ; l'état « en cours de paiement » = **appartenance à un lot `generated`** (JOIN `payment_batch_items`). Le verrouillage (AC3) empêche double-paiement. Avantage : migration non-breaking (CREATE TABLE seul), pas de bump, pattern propre. *Inconvénient : les guards de `pay`/`cancel` 12-2 doivent vérifier l'absence de lot actif (ajout cross-table).* À confirmer Guy.
- **DC2 — Variante pain.001 & namespace** [✅ FIGÉ Guy 2026-06-28] : `pain.001.001.09`, namespace `urn:iso:std:iso:20022:tech:xsd:pain.001.001.09`. La « ch.03 » = version des Swiss Implementation Guidelines (pas un namespace distinct). Portée v0.4 : types de paiement **1** (IBAN domestique CH/LI, RmtInf Ustrd) et **3** (QR-IBAN + référence QRR, RmtInf Strd) **uniquement**. Types 2/2.2 (ESR/IS, étranger SEPA/SWIFT) **hors scope** (documenter limitation).
- **DC3 — `DbtrAgt`/`CdtrAgt` (BIC/Clearing)** [✅ FIGÉ P1-C2] : émettre `DbtrAgt/FinInstnId/Othr/Id=NOTPROVIDED` (SIX autorise pour le domestique sans BIC) et **omettre `CdtrAgt`** (déductible de l'IBAN créancier en domestique CH/LI). Pas de résolution BIC.
- **DC4 — Génération XML** [✅ FIGÉ P1-C2] : **`quick-xml::Writer`** (échappement automatique, cohérence projet avec kesh-import) dans `kesh-payment::pain001`. **JAMAIS** de concat de `String` (injection/échappement). **Déclaration XML obligatoire** en premier événement : `<?xml version="1.0" encoding="UTF-8"?>` via `Event::Decl(BytesDecl::new("1.0", Some("UTF-8"), None))` [M2].
- **DC5 — Un PmtInf vs N** [✅ FIGÉ P1-C2] : **un seul `PmtInf`** par lot (même débiteur + même date d'exécution pour tout le lot). `BtchBookg` omis. `NbOfTxs` + `CtrlSum` présents au niveau **GrpHdr ET PmtInf** (obligatoires pain.001.001.09) [M1].
- **DC6 — Confirmation ATOMIQUE + extraction `pay_in_tx`** [✅ FIGÉ P1-C1/C2, Option A] : la confirmation poste N écritures **dans UNE transaction unique (tout ou rien)** — ≠ génération (batch FailedProposal). ⚠️ **`supplier_invoices::pay` ouvre sa propre tx (`pool.begin`, ligne 429) → réutilisation directe N fois = N transactions = PAS atomique**. Donc : **extraire `pay_in_tx(tx: &mut Transaction<'_, MySql>, company_id, id, choice, payment_date, user_id)` depuis `pay()`** dans `supplier_invoices.rs` (12-2) ; `pay()` devient un wrapper `pool.begin() → pay_in_tx → commit`. `confirm_batch` ouvre **une** tx et appelle `pay_in_tx` N fois. ⚠️ Modif d'un fichier 12-2 mergé → File List + **test de non-régression 12-2** (les 12 tests intégration `pay` doivent rester verts). Sur échec d'une facture (exercice clos / compte inactif) → rollback total, lot reste `generated`, HTTP 422 `{ error: <CODE>, invoiceId }` [M4].
- **DC7 — Emplacement & couche de validation** [✅ FIGÉ P1-C2/M3] : générateur pain.001 **pur (zéro I/O DB)** dans **`kesh-payment::pain001`** (structs d'entrée déjà validées) ; **validation IBAN/QR-IBAN/QRR dans `kesh-db::repositories::payment_batches::create_batch`** (repo layer, AVANT appel au générateur) via `kesh-qrbill::validation` → `kesh-payment` ne dépend PAS de kesh-qrbill (reste pur). Deps `kesh-payment/Cargo.toml` à ajouter : `quick-xml = "0.39"` (= kesh-import), `rust_decimal`, `chrono`, `thiserror` (workspace). Entités/repo lot dans `kesh-db` ; routes dans `kesh-api`.

## Tasks / Subtasks (provisoire — à affiner au validate, split probable)

- [x] **T1 — Générateur pain.001 (kesh-payment)** (AC: 4 ; DC3,DC4,DC5,DC7) : créer `src/pain001/mod.rs` + `pub mod pain001;` dans `src/lib.rs` [L1] ; deps `Cargo.toml` : `quick-xml="0.39"`, `rust_decimal`, `chrono`, `thiserror` (workspace) [L2]. Structs d'entrée pures (`Pain001Batch { msg_id, creation_dt, initiating_party, debtor_name, debtor_iban, requested_execution_date, transactions: Vec<Pain001Tx> }`, `Pain001Tx { end_to_end_id: String, amount: Decimal, creditor_name, creditor_account (enum IBAN|QrIban), reference (enum Qrr(String)|Unstructured(String)) }`) + `generate_pain001(batch) -> Result<String, PaymentError>` via `quick-xml::Writer` : **déclaration XML d'abord** (`Event::Decl`, UTF-8) [M2], `NbOfTxs`+`CtrlSum` au niveau GrpHdr ET PmtInf [M1], `DbtrAgt/Othr/Id=NOTPROVIDED` + CdtrAgt omis [DC3], `ChrgBr=SLEV`. **Contraintes ISO [L-F1]** : `MsgId`/`PmtInfId`/`EndToEndId` ≤ 35 car. (`PAY-{batch:08}-{inv:08}`=21, OK) ; `Cdtr/Nm`/`Dbtr/Nm` ≤ 70 (tronquer sinon XSD reject) ; `Ustrd` ≤ 140. **`CtrlSum` [L-F2]** = Σ exacte des `InstdAmt`, **2 décimales, `.` décimal, sans séparateur de milliers** (format `{:.2}` cohérent avec `InstdAmt`). **Ordre XSD strict [L-F3]** (quick-xml ne le valide pas) : GrpHdr(MsgId,CreDtTm,NbOfTxs,CtrlSum,InitgPty) → PmtInf(PmtInfId,PmtMtd,NbOfTxs,CtrlSum,ReqdExctnDt,Dbtr,DbtrAcct,DbtrAgt,ChrgBr,CdtTrfTxInf) → CdtTrfTxInf(PmtId,Amt,Cdtr,CdtrAcct,RmtInf). Helper local `el(writer,name,text)` anti-verbosité conseillé [L-F4]. Tests unitaires (structure, NbOfTxs/CtrlSum, QR-IBAN→Strd/QRR vs IBAN→Ustrd, échappement, longueurs) + golden file `tests/fixtures/pain001_*.xml` (idéalement validé contre XSD SIX).
- [x] **T2 — Migration & entités lot (kesh-db)** (AC: 3,7 ; DC1) : migration `payment_batches` (company_id FK, bank_account_id FK, status CHECK('generated','confirmed','cancelled'), requested_execution_date, total_amount, msg_id, created_at…) + `payment_batch_items` (payment_batch_id FK CASCADE, supplier_invoice_id FK RESTRICT, **end_to_end_id VARCHAR(35) persisté** [H2], position). ⚠️ **PAS de UNIQUE SQL sur supplier_invoice_id** [H1] (facture réutilisable après cancel) — le verrou « 1 facture dans 1 seul lot actif » est applicatif (`SELECT … FOR UPDATE` + JOIN sur lots `generated`). **Non-breaking** (CREATE TABLE seul). Audit idempotence + `TABLES_TO_TRUNCATE` (`payment_batch_items` AVANT `payment_batches`) + **compteurs en dur** [M6] : `admin_full_export_e2e` data_count **28→30**, `migrations_upgrade_path` total **37→38** + fenêtre **`total-14`→`total-15`**.
- [x] **T3 — Repo lot (kesh-db) + extraction pay_in_tx** (AC: 1,2,3,6,7,8,9 ; DC1,DC6) :
  - [x] **Extraire `supplier_invoices::pay_in_tx(tx, …)`** depuis `pay()` (12-2, corps l.431-603 extractible tel quel, `pay()`→wrapper) [C1/DC6]. ⚠️ **`pay_in_tx` est GUARD-FREE pour la lot-membership** [H-FRESH-1 passe 3] : il garde uniquement le verrou `FOR UPDATE` + statut `open` (l.433 migre dedans). **Le guard « facture dans lot generated » va dans le wrapper `pay()` + dans `cancel()`, JAMAIS dans `pay_in_tx`** — sinon `confirm_batch` (qui appelle `pay_in_tx` pendant que le lot est encore `generated`) se self-bloque et aucune confirmation n'aboutit. **Test de non-régression : les 12 tests intégration 12-2 restent verts** ; **+ test `confirm_batch` end-to-end OBLIGATOIRE** (génération→verrou→confirm poste N écritures→`paid`) — seul ce test révèle un éventuel self-block (les 12 tests n'ont pas de lot).
  - [x] `create_batch` : pour chaque facture, `SELECT supplier_invoices … FOR UPDATE` [H1] + guards (existe/`open`/a IBAN **ou** QR-IBAN/pas dans lot `generated`/company-scoped) + **validation `kesh-qrbill` IBAN/QR-IBAN/QRR** [M3/AC5], erreurs per-facture en `PaymentBatchFailedItem { supplier_invoice_id, error_code, details }` (pattern `FailedProposal` `reconciliation.rs:152`, struct DÉDIÉE pas import) [H3]. Calcule `end_to_end_id` (ex. `PAY-{batch_id:08}-{invoice_id:08}`, ≤35) persisté [H2].
    - **Sérialisation concurrente [P2-clarif passe 2]** : le `FOR UPDATE` sur la **ligne `supplier_invoices`** EST le point de sérialisation — deux `create_batch` concurrents sur la même facture verrouillent la **même** ligne ; le 2e bloque jusqu'au commit du 1er, puis son guard « pas dans lot generated » (JOIN `payment_batch_items`+`payment_batches`) voit l'insert du 1er et rejette en `failed[]`. Guard OBLIGATOIREMENT dans la même tx, APRÈS le `FOR UPDATE`. (MariaDB n'a pas d'index UNIQUE partiel filtré → pas de UNIQUE SQL ; `FOR UPDATE` suffit, cf. `acquire_company_sentinel_lock` Epic 8.)
    - **Sélection coordonnée [M2-1]** : facture avec **QR-IBAN** → `CdtrAcct`=QR-IBAN + `RmtInf/Strd/CdtrRefInf` QRR (depuis `payment_reference`) ; sinon **IBAN** → `CdtrAcct`=IBAN + `RmtInf/Ustrd`. Si **les deux** présents → **préférer QR-IBAN**. Au moins un requis (AC2).
    - **Montant [M2-2]** : `amount` pain.001 = **`total_amount` (TTC)** (cohérent avec l'écriture de règlement que la confirmation poste, qui débite 2000 de `total_amount`). `expected_payment_amount` reste informatif, NON utilisé (évite tout écart transfert≠règlement).
  - [x] `confirm_batch` : UNE tx ; **pré-check batch-level AVANT la boucle [M-FRESH-1 passe 3]** : `find_by_id_for_company` du compte source → `422` si `bank_account.archived == true` (`{error:"BANK_ACCOUNT_ARCHIVED", invoiceId:null}`) — `pay_in_tx` ne porte PAS ce check (sa branche BankTransfer ne vérifie que `journal_account_id` NON NULL, pas `archived`). Le compte source étant unique pour le lot (DC5), check une seule fois. Puis `pay_in_tx` per-facture (atomique), lot→`confirmed` ; rollback total sur échec (le sous-cas `journal_account_id`→compte `active=false` est aussi rattrapé défensivement par `create_in_tx WHERE active=TRUE`, mais le pré-check donne le code propre).
  - [x] `cancel_batch` [LOW-1 figé] : lot→`cancelled` par **UPDATE status** (les `payment_batch_items` sont **conservés** pour l'audit/historique) ; déverrouillage = le guard ne matche que les lots `status='generated'`, donc un lot `cancelled` ne verrouille plus ses factures.
  - [x] `get`/`list`. **Guards cross-table dans `supplier_invoices::pay`/`cancel` 12-2** : refus si facture dans lot `generated` (JOIN `payment_batch_items`+`payment_batches status='generated'`). **Placement [P4-M1 passe 4, cohérent H-FRESH-1]** : dans **`cancel()`** (monolithique, pas extrait) → APRÈS son propre `FOR UPDATE` l.633 ; dans **`pay()`** → dans le **wrapper**, AVANT l'appel à `pay_in_tx` (le `FOR UPDATE` l.433 a migré DANS `pay_in_tx` qui reste guard-free — **NE PAS** y mettre le guard, sinon `confirm_batch` self-bloque). ⚠️ régression possible 12-2 (les 12 tests restent verts car sans lot ; test `confirm_batch` e2e prouve l'absence de self-block).
- [x] **T4 — Routes API (kesh-api)** (AC: 1,4,6,7,8,10,11) : `POST /api/v1/payment-batches` (créer, `{accepted, failed}`) + `GET …/{id}/pain001` (download XML) + `POST …/{id}/confirm` + `POST …/{id}/cancel` + `GET …` liste + `GET …/{id}` détail. **RBAC [LOW-3 figé]** : POST create/confirm/cancel = `comptable_routes` (require_comptable_role) ; GET liste/détail/**pain001** = `authenticated_routes` (tout rôle authentifié, comme les autres lectures). Cohérent 12-2.
- [x] **T5 — Frontend (kesh frontend)** (AC: 1,4,6,7,10,12) : feature `payment-batches` + page de sélection des factures `open` virement-réglables + choix compte source + génération + download + page liste/détail lot + actions confirmer/annuler + sidebar + i18n.
- [x] **T6 — Tests + doc** (AC: 13) : intégration repo + E2E + golden pain.001 + CHANGELOG/README + manuel. Quality gate Test Locally First (exit code vérifié, PAS de pipe masquant — leçon 12-2).

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
- `crates/kesh-api/src/routes/reconciliation.rs:152` — `FailedProposal { bank_transaction_id, error_code, details }` + `accept_batch`→`{accepted, failed}` : **modèle canonique** pour `POST /payment-batches` (AC2). Adapter l'identifiant business → `supplier_invoice_id`.
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

Opus 4.8 (1M context) — run autonome bout-en-bout (livraison inline 12-3a..e).

### Debug Log References

- Clippy « very complex type » sur le tuple de lecture facture → alias `InvoiceCoordsRow`.
- Champ `recoverable_id` non lu dans le test Ctx → retiré.

### Completion Notes List

- **12-3a** Générateur pain.001 (kesh-payment) : Cargo deps (quick-xml/rust_decimal/chrono/thiserror), `src/pain001/mod.rs` (structs pures + `generate_pain001` via quick-xml Writer : decl XML, ordre XSD, NbOfTxs/CtrlSum GrpHdr+PmtInf, DbtrAgt NOTPROVIDED, type 1 IBAN→Ustrd / type 3 QR-IBAN→Strd/QRR, longueurs ISO 35/70/140). 9 tests (8 unit + golden file) + example gen_golden.
- **12-3b** Migration + entités (kesh-db) : `payment_batches` + `payment_batch_items` (DC1 non-breaking, UNIQUE(batch,invoice) per-lot). Entité `PaymentBatch/Item/NewPaymentBatch`. `TABLES_TO_TRUNCATE` + compteurs (export 28→30, migration 37→38, fenêtre total-15) + audit idempotence.
- **12-3c** Repo + extraction `pay_in_tx` : `supplier_invoices::pay_in_tx` GUARD-FREE (H-FRESH-1) extrait de `pay()`, guard lot-membership dans wrapper `pay()` + `cancel()` (12 tests 12-2 non-régression verts). Repo `payment_batches` : create_batch (FailedProposal per-facture, FOR UPDATE, validation kesh-qrbill, préférence QR-IBAN, montant=total_amount) + confirm_batch (atomique, pay_in_tx N fois, pré-check archivé) + cancel_batch + get/list + generate_pain001_xml. 7 tests dont `confirm_batch_posts_settlements_no_self_block` (prouve H-FRESH-1, solde 2000=0).
- **12-3d** Routes : POST create/confirm/cancel (comptable_routes) + GET liste/détail/pain001 (authenticated_routes). DTO camelCase, download XML.
- **12-3e** Frontend : feature payment-batches (types/api/helpers + test) + page liste/génération (sélection factures + résultat {batch, failed}) + page détail (items, download, confirmer/annuler) + sidebar. i18n 4 locales. E2E spec.
- **12-3f** Doc : CHANGELOG [Non publié] + README. Quality gate Test Locally First exit-code vérifié (fmt + clippy workspace + test workspace serial + frontend check/lint/340 unit/build).

### File List

**kesh-payment** : `Cargo.toml` (M), `src/lib.rs` (M), `src/pain001/mod.rs` (N), `examples/gen_golden.rs` (N), `tests/fixtures/pain001_sample.xml` (N).
**kesh-db** : `Cargo.toml` (M), `migrations/20260628000002_payment_batches.sql` (N), `src/entities/payment_batch.rs` (N), `src/entities/mod.rs` (M), `src/repositories/payment_batches.rs` (N), `src/repositories/mod.rs` (M), `src/repositories/supplier_invoices.rs` (M — extraction pay_in_tx + guards), `src/backup.rs` (M), `tests/payment_batches_repository.rs` (N), `tests/migrations_upgrade_path.rs` (M).
**kesh-api** : `src/routes/payment_batches.rs` (N), `src/routes/mod.rs` (M), `src/lib.rs` (M), `tests/admin_full_export_e2e.rs` (M — data_count 30).
**i18n** : `crates/kesh-i18n/locales/{fr,de,it,en}-CH/messages.ftl` (M).
**Frontend** : `src/lib/features/payment-batches/{payment-batches.types.ts,payment-batches.api.ts,payment-batch-helpers.ts,payment-batch-helpers.test.ts}` (N), `src/routes/(app)/payment-batches/{+page.svelte,[id]/+page.svelte}` (N), `src/routes/(app)/+layout.svelte` (M), `tests/e2e/payment-batches.spec.ts` (N).
**Docs** : `CHANGELOG.md` (M), `README.md` (M), `docs/migrations-idempotence-audit.md` (M).

### Change Log (dev)

- run autonome Opus 4.8 — livraison inline 12-3a→12-3f. Commits : `8babbf3` (a générateur) + `f474569` (b migration+entités) + `16dfa7f` (c repo+pay_in_tx) + `614e322` (d routes) + `ac5a4cf` (e frontend) + doc-sync (f). Extraction `pay_in_tx` GUARD-FREE (H-FRESH-1) sans régression 12-2 (12 tests verts). Quality gate Test Locally First exit-code vérifié.

## Change Log (validate)

Boucle de validation adversariale (Review Iteration Rule) — rotation LLM, contexte frais, ground-truth grep.

- **Passe 1 — Sonnet 4.6** : 15 findings (2 CRITICAL / 3 HIGH / 7 MEDIUM / 3 LOW), tous ground-truthés. **C1** : `pay()` ouvre sa propre tx (`pool.begin` l.429) → confirmation atomique impossible par réutilisation directe → extraire `pay_in_tx` (DC6 Option A, modif fichier 12-2 + test non-régression). **C2** : DC3-DC7 non figés → tous figés (DC3 NOTPROVIDED, DC4 quick-xml Writer + decl XML, DC5 un PmtInf + NbOfTxs/CtrlSum GrpHdr&PmtInf, DC6 atomique+pay_in_tx, DC7 validation en kesh-db pas kesh-payment). **H1** verrou facture (PAS de UNIQUE SQL, `SELECT FOR UPDATE` + JOIN lots generated, guard post-FOR-UPDATE dans pay/cancel). **H2** EndToEndId format `PAY-{batch:08}-{inv:08}` persisté. **H3** struct dédiée `PaymentBatchFailedItem` (pattern FailedProposal `reconciliation.rs:152`). **M1** NbOfTxs/CtrlSum PmtInf. **M2** déclaration XML. **M3** validation en kesh-db (kesh-payment pur). **M4** confirm error 422 `{error,invoiceId}`. **M5** archived+active check. **M6** compteurs cibles (data_count 28→30, migration 37→38, fenêtre total-15). **M7** sidebar « Paiements fournisseurs » après Factures fournisseurs. **L1** mod.rs+pub mod. **L2** deps Cargo. **L3** ref 150→152. → 15 patches. Split 12-3a..e validé (séquencement : a/b parallèles → c [touche 12-2] → d → e).
- **Passe 2 — Haiku 4.5** : 1 CRITICAL + 2 MEDIUM + 3 LOW. Garde-fou Haiku : **CRITICAL-P2 (race facture dans 2 lots)** reclassé en **clarification** — le design passe 1 (`SELECT supplier_invoices FOR UPDATE` + guard dans la même tx) sérialise déjà les `create_batch` concurrents sur la même facture (verrou même ligne) ; le scénario de Haiku suppose à tort une lecture pré-commit. Sérialisation rendue **explicite** dans T3 (pas de changement structurel). MEDIUM réels : M2-1 sélection coordonnée (les deux présents → préférer QR-IBAN, pas un refus) + M2-2 montant = `total_amount` TTC (pas `expected_payment_amount`, évite écart transfert≠règlement). LOW : cancel = UPDATE status (items conservés audit), invariant NbOfTxs GrpHdr==PmtInf, RBAC explicite (mutations comptable / lectures+pain001 tout rôle). Patches passe 1 confirmés sains (C1 l.429, H3 l.152, M6 28/37 ground-truth). → 5 patches (+1 clarif).
- **Passe 3 — Opus 4.8** (catch-architectural) : 1 HIGH + 1 MEDIUM + 4 LOW, ratés par Sonnet+Haiku. **H-FRESH-1** : le guard lot-membership « après FOR UPDATE l.433 » tombe DANS `pay_in_tx` → `confirm_batch` (appelle `pay_in_tx` quand lot encore `generated`) se SELF-BLOQUE, confirmation jamais possible ; les 12 tests non-régression ne le voient pas. Fix : `pay_in_tx` GUARD-FREE, guard dans wrapper `pay()`+`cancel()` SEULEMENT + test `confirm_batch` end-to-end obligatoire. **M-FRESH-1** : refus « compte source archivé » (AC9) non porté par `pay_in_tx` → pré-check batch-level avant la boucle de `confirm_batch`. LOW : longueurs ISO (≤35/70/140), format CtrlSum 2-déc, ordre XSD strict, helper anti-verbosité. **Réfutés (gain passe)** : numérotation séquentielle en boucle SAINE (read-own-writes même tx), extraction `pay_in_tx` faisable, PAS de double-source montant (total_amount=ligne crédit achat par construction), compta deux-temps cohérente (2000 soldé), split sans cycle, compteurs M6 EXACTS (37→38, total-14→15, 28→30 ground-truthés). → 6 patches. Trend >LOW : p1=12 → p2=4 → p3=2.
- **Passe 4 — Sonnet 4.6** (convergence) : confirme passes 1-3 SAINES (9 vérifs ground-truth EXACTES : pay l.429, FOR UPDATE l.433, FailedProposal l.152, data_count 28, total 37, total-14, 12 tests, find_by_id_for_company archived, kesh-payment placeholder). 1 MEDIUM **P4-M1** : contradiction résiduelle — la note `[H1]` du dernier bullet T3 (« guard après FOR UPDATE l.433 ») non mise à jour passe 3 → post-extraction tombe dans `pay_in_tx`, contredit H-FRESH-1. Corrigé : guard dans `cancel()` après l.633, dans `pay()` dans le wrapper avant `pay_in_tx`. P4-L1 LOW (fenêtre race pré-check archived) acceptable PME. → 1 patch. Trend >LOW : p1=12 → p2=4 → p3=2 → p4=1.
- **Passe 5 — Haiku 4.5** (confirmation) : **CONVERGÉ — 0 finding > LOW**. 9 vérifs ground-truth confirment les patches passes 1-4 sains (P4-M1 guard non-contradictoire, atomicité confirm_batch, compteurs 28→30/37→38/total-15, FailedProposal l.152, sélection QR-IBAN, invariant NbOfTxs). 0 hallucination.

### Verdict de convergence

**✅ CONVERGÉ — 0 finding > LOW réel** (passe 5). Cycle complet 5 passes **Sonnet→Haiku→Opus→Sonnet→Haiku**. Trend réel >LOW : **12 → 4 → 2 → 1 → 0**. ~27 patches. DC1-DC7 figés (DC1/DC2 par Guy). Catch architectural majeur **H-FRESH-1 (Opus)** : `pay_in_tx` GUARD-FREE sinon `confirm_batch` self-bloque — raté par Sonnet+Haiku. **SPLIT 12-3a..e validé** (a générateur kesh-payment / b migration+entités lot / c repo+extraction pay_in_tx+guards [touche 12-2] / d routes / e frontend+tests ; a,b parallèles → c → d → e). Spec **ready-for-dev**.
