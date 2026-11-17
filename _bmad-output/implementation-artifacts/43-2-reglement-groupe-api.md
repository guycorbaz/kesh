# Story 43-2 : un règlement pour plusieurs factures — l'API de la réconciliation (#823)

Status: ready-for-dev

*Fiche rédigée le 2026-11-17 par l'agent de rédaction de l'Epic 43 (Opus 5.5), workflow `bmad-create-story`, code lu à
`8b50fd195` ; révisée le soir même (décisions de Guy), puis **remédiée après sa validation P1**
(`kesh-gate-logs/e43/validation-43-2-p1.md`, 0/2/7/8 : H1, H2, M1-M8 appliqués ; LOW d'une ligne appliqués, les autres en
Dev Notes, étape 0). Release **0.19.0**. **Part de la 43-1a à son commit T3** ; se fusionne après elle. Aucune migration.
Décisions : C-43-1 à C-43-23 (fichier d'épique), qui font foi. Ce que cette fiche impose à la 43-1a : § *Exigences sur la
43-1a*. Validation : passe 2 à lancer.*

**Issues** : `refs #823` ; aucun mot-clé de fermeture dans un commit.

## Story

En tant qu'intégration (clé d'API) ou écran de réconciliation, je veux accepter **une** transaction contre **plusieurs**
factures, chercher les factures candidates par texte, et voir puis annuler le rapprochement qui en résulte — avec les
refus par proposition du *Pattern batch*.

## Ce qui se lit au code (`8b50fd195`)

- `AcceptProposalInput` (`routes/reconciliation.rs:855`), `bank_transaction_id()` (:903), message de l'extracteur (:961,
  quatre types) ; pré-vol de `post_accept` qui refuse en `400` `invoiceId <= 0`, `supplierInvoiceId <= 0`, etc. (:1605-1690) ;
  `accept_batch` (:1967) : point de sauvegarde par proposition, `ROLLBACK TO SAVEPOINT` après un refus, interblocage 1213
  absorbé puis lot rejoué (`is_savepoint_lost` :2046, `retry_with` près de :1791, :2015-2025) ; `accept_one` (:2057) rend
  `Result<AcceptedProposal, FailedProposal>` : **aucune** `AppError` ne peut en sortir.
- L'unitaire convertit **tout** `DbError` en `FailedProposal` : `DATABASE_ERROR` avec `details.message` (une quinzaine de
  sites de :2408 à :2872), `INTERNAL_ERROR` pour un défaut structurel (:2208, :2803) ; refus de compte de créance par
  `claim_account_failed_proposal` (:2144, code `SETTLEMENT_COUNTERPARTY_IS_CLAIM_ACCOUNT`) ; projet par
  `project_error_to_failed_proposal` (:743 : `PROJECT_ARCHIVED`, `PROJECT_NOT_FOUND`) ; période par
  `period_locked_failed_proposal` (:603), `later_fiscal_year_closed_failed_proposal` (:644),
  `non_postable_failed_proposal` (:682).
- `AcceptedProposal` (:557-567) : `bank_transaction_id`, `invoice_id: i64` (sentinelle `0` hors facture client),
  `supplier_invoice_id: Option<i64>`, `journal_entry_id`, `score: MatchScore` **obligatoire** ; lu tel quel par
  `frontend/src/lib/features/reconciliation/reconciliation.types.ts:134-141` ; décrit par `docs/api-external.md:679`.
- Clés d'audit de l'unitaire, en **snake_case** : `bank_transaction_id`, `invoice_id`, `score`, `batch_size` (taille du lot
  reçu, Story 27-4, :1706-1710), `journal_entry_id`, `sale_journal_entry_id`, `settled_amount`, `paid_amount`,
  `rounding_difference` (près de :3003-3075).
- Aucune garde de devise dans `accept_one_invoice` ni `accept_one_supplier_invoice` ; la garde CHF n'est que dans les
  propositions (:506, :1101) et le rattachement (`transaction_can_be_linked_to_entry`,
  `repositories/reconciliation_link.rs:535` : montant non nul et `currency == "CHF"` ; miroir écran `canLink`).
- Recherche : `docs/search-patterns.md` (§ 1-4 et 6 : `contacts.name` en FULLTEXT `ft_contacts_name`, jetons de 3
  caractères au moins, préfixe seulement ; numéros et références en `LIKE … ESCAPE` ; disjonction dans les deux branches) ;
  `crates/kesh-db/src/util/search.rs` (`escape_like` :24, `escape_boolean_ft` :121).
- Vue `ReconciliationTransactionResponse` (:5821) : `(invoice_id, supplier_invoice_id)` par un `match` exhaustif sur
  `ReconciliationKind` (:5842-5849) ; `CancelReconciliationResponse` (:5896-5905) : **ni `kind` ni liste** —
  `bank_transaction`, `reversal_journal_entry_id`, `invoice_id`, `supplier_invoice_id`. Routes sœurs :
  `/reconciliation/transactions/{id}/link-candidates` et `…/entry-candidates` (`lib.rs:886-943`).

## Décisions propres à la story

- **D1** — Code neuf dans `crates/kesh-api/src/routes/reconciliation_group.rs` (module déclaré dans `routes/mod.rs`) ;
  `reconciliation.rs` ne gagne que les variantes, le pré-vol, l'aiguillage, le texte de l'extracteur et les champs de la vue.
- **D2 — Aucune erreur d'un groupe ne sort au lot (H1).** Les refus de `GroupRefusal` deviennent des `FailedProposal` par
  **une seule** fonction, `group_refusal_to_failed_proposal(bank_transaction_id, GroupRefusal)` ; tout `Err(DbError)` des
  fonctions de groupe, par **une seule** fonction, `group_error_to_failed_proposal(bank_transaction_id, DbError)` :
  `INTERNAL_ERROR` pour `DbError::Invariant` (défaut structurel), `DATABASE_ERROR` avec `details.message` sinon — jamais
  une `AppError`. L'interblocage (1213) est ainsi absorbé comme dans l'unitaire et le lot rejoué par `accept_batch`. Un
  corps illisible reste un `400` du lot.
- **D3 — Audit écrit par le dépôt** (C-43-9) : la route n'écrit **aucune** trace pour un groupe.
- **D4 — Type fournisseur retirable** (C-43-18) : la variante `SupplierInvoiceGroup`, sa branche du pré-vol, du `match` de
  `bank_transaction_id()` et de l'aiguillage, le **message de l'extracteur passé de cinq à six types**, la famille
  fournisseur de la recherche et de la vue, et leurs tests vivent dans **un seul commit** à part, retirable si la 43-1b
  sort ; un test fixe le texte du message à cinq types avant ce commit, à six après.
- **D5 — Montants bruts de bout en bout (H2, C-43-21).** Un reste dû peut porter quatre décimales (10.0050) ; le serveur
  calcule l'écart sur le brut. Le contrat rend donc le **brut** (`amountDueRaw`, `difference`, `settledInvoices[].amount`,
  chaînes à quatre décimales) **et** le centime pour l'affichage (`amountDue`). L'écran calcule en entiers de 1/10 000 sur le
  brut (43-3). La différence `d = |montant| − Σ apports bruts` (valeur absolue du montant de la transaction, comme aux lignes de `group_difference` plus bas et à D4 bis de la 43-1a) se décompose (D8) en fractions de centime (compte d'arrondi,
  au brut, comme la troisième ligne de l'unitaire, `settlement_journal_lines`) et en frais au centime ; seuls les frais se
  comparent au plafond.
- **D6 — Garde CHF (M6, C-43-22).** Une transaction dont la devise n'est pas `CHF`, ou de montant nul, est refusée par
  proposition (`RECONCILIATION_GROUP_INVALID`, `reason: "currency"` ou `"zero_amount"`) **avant** tout verrou ; la
  recherche rend `409 RECONCILIATION_GROUP_UNSUPPORTED_TRANSACTION` pour la même transaction. La condition est la fonction
  existante `transaction_can_be_linked_to_entry` (une seule définition), appelée par les deux.
- **D7 — Plafond réglable** (décision de Guy, Q4 ; C-43-24). La valeur est le réglage
  `company_invoice_settings.group_settlement_max_difference`, lu au moment du geste (43-1a AC3 bis) ; la constante
  `DEFAULT_GROUP_DIFFERENCE_MAX` n'en est que la valeur par défaut. La recherche rend le réglage (`maxDifference`), les
  refus le portent (`details.maxDifference`), `docs/api-external.md` le nomme sans recopier de valeur figée hors de la
  mention « 5.00 par défaut » ; un test règle 1.00 et vérifie `maxDifference` et le refus.
- **D8 — Différences (C-43-16, proposition de l'orchestrateur en attente de Guy)** : fractions de centime au compte
  d'arrondi, sans plafond ; frais (client : moins reçu ; fournisseur : plus payé) au compte de frais bancaires, dans la
  limite du plafond ; l'inverse refusé (`RECONCILIATION_OVERPAYMENT`, `RECONCILIATION_UNDERPAYMENT`). Aucun refus de projet
  (C-43-20).

## Contrat

### `POST /api/v1/reconciliation/accept` — deux types neufs

```json
{ "bankAccountId": 3, "proposals": [
  { "type": "invoice_group", "bankTransactionId": 41, "invoiceIds": [12, 15, 19] },
  { "type": "supplier_invoice_group", "bankTransactionId": 42, "supplierInvoiceIds": [7, 8] }
] }
```

`paymentEntryIds` (paiements existants) s'ajoute avec la 43-5, champ facultatif.

**Pré-vol (M7)**, dans `post_accept`, comme les types existants : un identifiant ≤ 0 dans `invoiceIds` /
`supplierInvoiceIds` → `400 VALIDATION_ERROR` du lot ; la taille du corps reste bornée par la limite existante. Puis, **par
proposition, avant la lecture de la transaction et tout verrou**, dans cet ordre : tableau vide ou d'un élément, plus de
50, doublon → `RECONCILIATION_GROUP_INVALID` (`reason: "too_few" | "too_many" | "duplicate"`).

**Élément de `accepted[]` pour un groupe (M1)** — la structure complète :

```json
{ "bankTransactionId": 41, "invoiceId": 0, "supplierInvoiceId": null, "journalEntryId": 905,
  "score": { "...": "sous-scores à 0, total conventionnel 1.0, comme le type rule" },
  "difference": "-0.4000", "roundingDifference": "0.0000", "bankFees": "0.40",
  "settledInvoices": [ { "invoiceId": 12, "amount": "500.0000" }, { "invoiceId": 15, "amount": "300.0000" } ] }
```

Une seule définition pour les deux familles, celle de `group_difference` (43-1a D4 bis) : `difference = |montant| − Σ
apports bruts` (signé, brut : négatif quand moins reçu, positif quand plus payé) ; `roundingDifference` = les fractions de
centime (signé, brut) ; `bankFees = |e|` (≥ 0, au centime, `"0.00"` sans frais) (C-43-16).

`invoiceId` vaut `0` (sentinelle documentée, `docs/api-external.md:679`), `supplierInvoiceId` `null` ; pour un groupe
fournisseur, `settledInvoices[].supplierInvoiceId`. `journalEntryId` existe déjà ; seuls `difference`, `roundingDifference`,
`bankFees` et `settledInvoices` sont neufs (additifs, absents pour les autres types). Montants au brut (D5). Le type TypeScript `AcceptedProposal` gagne
deux champs facultatifs ; un test de la 43-3 désérialise cet élément.

**Refus (`failed[]`, `200`) — table complète variante de `GroupRefusal` → code (M8)**, construite par
`group_refusal_to_failed_proposal` seule. `details` porte toujours `invoiceId` ou `supplierInvoiceId` quand une facture est
en cause, et **`invoiceNumber`** (numéro affichable : `invoice_number` client, numéro interne ou à défaut numéro du
fournisseur) pour que l'écran la nomme (43-3).

| Variante (43-1a / 43-1b) | `errorCode` | `details` |
|---|---|---|
| `TooFew`, `TooMany`, `Duplicate` | `RECONCILIATION_GROUP_INVALID` | `reason` : `too_few`, `too_many`, `duplicate` (+ `invoiceId`) |
| `UnsupportedTransaction` (devise, montant nul) | `RECONCILIATION_GROUP_INVALID` | `reason` : `currency`, `zero_amount` |
| `TransactionNotFound` | `BANK_TRANSACTION_NOT_FOUND` | — |
| `TransactionNotPending` | `RECONCILIATION_ALREADY_RECONCILED` | — |
| `NotCredit` / `NotDebit` | `RECONCILIATION_TRANSACTION_NOT_CREDIT` / `…_NOT_DEBIT` | — |
| `InvoiceNotFound` | `INVOICE_NOT_FOUND` / `SUPPLIER_INVOICE_NOT_FOUND` | id |
| `InvoiceNotEligible { reason }` | `RECONCILIATION_INVOICE_NOT_ELIGIBLE` | `reason` : `invoice_not_validated`, `invoice_already_paid`, `invoice_journal_entry_not_set`, `nothing_due`, `supplier_invoice_not_open`, `supplier_invoice_in_payment_batch`, `race_during_update` (version) |
| `HeldByPaymentBatch` | `RECONCILIATION_SUPPLIER_PAYMENT_HOLD` | `reason: "other_invoice_in_payment_batch"`, ids |
| `SaleEntryMalformed` | `INVOICE_SALE_ENTRY_MALFORMED` | `reason: "no_debit_line_on_sale_entry"` |
| `BankAccountNotConfigured` / `NotFound` / `Closed` | `BANK_ACCOUNT_NOT_CONFIGURED` / `BANK_ACCOUNT_NOT_FOUND` / `BANK_ACCOUNT_CLOSED` | — |
| `ClaimAccount { side }` (créance ou arrondi = banque ou créance) | `SETTLEMENT_COUNTERPARTY_IS_CLAIM_ACCOUNT` | comme `claim_account_failed_proposal` |
| `PaymentBeforeInvoiceDate` | `RECONCILIATION_INVOICE_NOT_ELIGIBLE` | `reason: "payment_date_before_invoice_date"`, comme `date_refusal_failed_proposal` (:2316) |
| `Overpayment` (client, trop reçu) | `RECONCILIATION_OVERPAYMENT` | `transactionAmount`, `sumApplied` |
| `Underpayment` (fournisseur, moins payé) | `RECONCILIATION_UNDERPAYMENT` | `transactionAmount`, `sumApplied` |
| `DifferenceTooLarge` | `RECONCILIATION_GROUP_DIFFERENCE_TOO_LARGE` | `transactionAmount`, `sumApplied`, `bankFees`, `maxDifference` |
| `RoundingAccountNotConfigured` | `ROUNDING_ACCOUNT_NOT_CONFIGURED` | — |
| `BankFeesAccountNotConfigured` | `WRITE_OFF_ACCOUNT_NOT_CONFIGURED` | `nature: "bank_fees"` |
| `PeriodLocked` / `LaterFiscalYearClosed` / `FiscalYearInvalid` | `PERIOD_LOCKED` / `LATER_FISCAL_YEAR_CLOSED` / `FISCAL_YEAR_INVALID` | comme les constructeurs existants (:603, :644) |

Pas de refus « compte non imputable » : l'écriture se crée avec `enforce_postable = false`, comme l'unitaire (43-1a D7) ;
ses comptes sont ceux des réglages et des écritures de vente ou d'achat.

Les constructeurs existants sont **appelés**, non recopiés. Une variante que la 43-1a ajouterait sans ligne ici fait
échouer la compilation (`match` exhaustif sans joker).

### `GET /api/v1/reconciliation/transactions/{id}/group-candidates?q=…&limit=…&exclude=…`

Chemin aligné sur les routes sœurs (L-1). Comptable+, lecture seule. Transaction `pending`, non rejetée, compte ouvert
(sinon `404`, `409 BANK_ACCOUNT_CLOSED`, `409 RECONCILIATION_ALREADY_RECONCILED`, comme `link-candidates`), CHF et non
nulle (sinon `409 RECONCILIATION_GROUP_UNSUPPORTED_TRANSACTION`, D6). **Tous tiers.** Crédit → factures clients
`validated`, reste dû > 0 ; débit → factures fournisseur `open`, hors lot `generated`.

**Recherche `q` (M3, `docs/search-patterns.md`)** — `q` taillé ; vide → pas de filtre ; sinon, disjonction :
- **nom du tiers** : `MATCH(c.name) AGAINST(? IN BOOLEAN MODE)` avec `escape_boolean_ft(q)` suivi de `*` ; si le terme
  échappé est vide **ou** fait moins de 3 caractères, repli `c.name LIKE ? ESCAPE '\\'` avec `escape_like` (pas de silence
  sur un `q` court) ;
- **numéros et référence** (`LIKE '%…%' ESCAPE '\\'`, `escape_like`) : client `invoice_number` ; fournisseur
  `internal_number`, `supplier_invoice_number`, `payment_reference` ;
- la disjonction est présente dans **les deux** branches (terme FULLTEXT vide ou non), comme le prescrit § 6.

**Tri puis troncature** : `imputation_order` est une fonction Rust ; la route lit au plus **500** factures ouvertes
filtrées (plafond de lecture nommé `GROUP_CANDIDATES_SCAN_LIMIT`), les trie par `imputation_order`, puis tronque à
`limit` (50 par défaut, 100 au plus) ; `hasMore` vaut vrai si la troncature a coupé **ou** si le plafond de lecture a été
atteint. `exclude=12,15` (facultatif, 50 identifiants au plus) retire les factures déjà imputées par l'écran avant la
troncature.

```json
{ "direction": "client", "transactionAmount": "1250.00", "maxDifference": "<le réglage de la société>",
  "candidates": [
    { "kind": "invoice", "id": 12, "number": "F-2026-0012", "contactName": "Exemple SA",
      "date": "2026-10-03", "dueDate": "2026-11-02", "totalTtc": "800.00",
      "amountDue": "500.01", "amountDueRaw": "500.0050" } ],
  "hasMore": false }
```

Candidate **fournisseur (M4)** : `{ "kind": "supplier_invoice", "id", "internalNumber", "supplierInvoiceNumber",
"contactName", "date", "dueDate", "paymentReference", "amountDue", "amountDueRaw" }` — `amountDue` = `total_amount` au
centime, `amountDueRaw` = `total_amount` brut ; pas de `totalTtc` distinct. Valeurs de `kind` en **snake_case**
(`invoice`, `supplier_invoice`, et `payment` avec la 43-5), comme `candidateType` (`reconciliation.types.ts:24`), partout
dans ce contrat. `transactionAmount` est **signé** comme la transaction (`-1250.00` pour un débit) ; la différence se
calcule sur la valeur absolue (L-2).

### Vue et annulation (M5)

- `GET /reconciliation/transactions/{id}` : `kind` gagne `"grouped_settlement"` ; pour un groupe, `invoiceId`,
  `supplierInvoiceId` et leurs numéros valent `null` ; champs additifs `groupedInvoices: [{ "kind": "invoice" |
  "supplier_invoice", "id", "number", "amount" }]` (vide hors groupe ; montant brut) et **`cancelBlockedInvoice: { "kind",
  "id", "number" } | null`** — la facture qui porte le premier motif d'un refus d'annulation par facture (créditée, soldée,
  lettrage figé), lue de `entry_cancel_outlook` (43-1a). Le `match` de :5842-5849 gagne le bras.
- `POST /reconciliation/transactions/{id}/cancel` : pour un groupe, `invoiceId` et `supplierInvoiceId` valent `null` ;
  champs additifs `kind` (le code de `ReconciliationKind`, pour tous les rapprochements) et `cancelledInvoices: [{ "kind",
  "id", "number" }]` (vide hors groupe).

## Critères d'acceptation

- **AC1** — Extracteur à six types (D4 : cinq avant le commit fournisseur) ; type inconnu → `400 VALIDATION_ERROR` ;
  pré-vol (M7) : identifiant ≤ 0 → `400` ; `bank_transaction_id()` couvre les deux variantes.
- **AC2** — `invoice_group` : sous le verrou du compte, `settle_invoices_in_tx` ; succès → élément de M1 ; chaque
  `GroupRefusal` → sa ligne de la table, rien d'écrit pour cette proposition, les autres passent ; un `Err(DbError)` →
  `DATABASE_ERROR` / `INTERNAL_ERROR` (D2). La route n'écrit aucune trace : le test lit `reconciliation.accepted` puis
  `invoice.paid` par facture, avec l'acteur et les clés snake_case exigées de la 43-1a (`batch_size` compris).
- **AC3** — `supplier_invoice_group` (commit D4) : symétrique, `pay_supplier_invoices_in_tx`.
- **AC4** — Lot mixte (`invoice` + groupe valide + groupe refusé) : deux `accepted`, un `failed`, dans l'ordre ; un
  `invoice_group` sur un débit refusé `RECONCILIATION_TRANSACTION_NOT_CREDIT` (sonde de C-43-15) ; **interblocage** : deux
  lots concurrents dont les groupes partagent une facture — l'un aboutit, l'autre refuse la facture (`race_during_update` ou
  `invoice_already_paid`) ou est rejoué, jamais un `500` du lot.
- **AC5** — Recherche : crédit et débit ; tiers différents dans la même liste ; un test par champ de `q` (nom ≥ 3
  caractères, nom de 2 caractères par le repli, numéro, numéro interne, numéro du fournisseur, référence) ; injection (`%`,
  `_`, `\`, `*`, `+`, `"` : aucun résultat parasite, aucune erreur) ; `exclude` ; tri puis troncature (`hasMore`) ; une
  facture d'une autre société jamais rendue ; transaction EUR → `409` (D6).
- **AC6** — Différences (D5, D8) : deux restes à 10.0050 et un virement de 20.02 → `difference` `"0.0100"`,
  `roundingDifference` `"0.0100"`, `bankFees` `"0.00"` ; virement de 20.01 → `difference` `"0.0000"`, `roundingDifference`
  `"0.0100"`, `bankFees` `"0.01"` ; trop reçu de 0.01 → `RECONCILIATION_OVERPAYMENT` ; frais au-delà du réglage →
  `RECONCILIATION_GROUP_DIFFERENCE_TOO_LARGE` avec `maxDifference` = le réglage ; `settledInvoices[].amount` au brut ;
  facture au projet archivé après coup acceptée (aucun refus de projet) ; paiement antérieur à une facture refusé
  (`payment_date_before_invoice_date`).
- **AC7** — Vue et annulation : groupe client (et fournisseur, D4) — `grouped_settlement`, `groupedInvoices`,
  `cancelBlockedInvoice` sur une facture créditée ; réponse du `POST` (M5) ; refus par le code du premier motif.
- **AC8** — Garde CHF (D6) : `invoice_group` sur une transaction EUR → `RECONCILIATION_GROUP_INVALID`, `reason:
  "currency"`, sans verrou ni écriture ; la même en CHF passe (sonde).
- **AC9** — `docs/api-external.md` : « Régler plusieurs factures par une transaction » (*depuis la v0.19.0*) — types,
  pré-vol, recherche (règles de `q`), table des codes, élément d'`accepted[]`, brut et centime, plafond nommé, garde CHF,
  champs de la vue et de l'annulation, *Pattern batch*.
- **AC10** — Accès (L-6) : rôle Comptable+ ; clé d'API en écriture : `POST /accept` permis ; clé en lecture
  (`ApiKeyReadOnly`, `middleware/auth.rs:139-142`) : `POST /accept` refusé, `GET group-candidates` permis ; rôle
  inférieur : `403`.

## Exigences sur la 43-1a (à reporter à sa remédiation d'ensemble)

1. **Forme brut/centime au commit T3** : `GroupedSettlement.difference`, `rounding_difference` et
   `GroupedInvoiceRef.amount` en `Decimal` brut (quatre décimales), `bank_fees` au centime ; lecture du plafond réglable
   (`DEFAULT_GROUP_DIFFERENCE_MAX` comme valeur par défaut seulement). *Reporté dans la 43-1a le 2026-11-17 (nuit).*
2. **Clés d'audit** : `reconciliation.accepted` en **snake_case** alignées sur l'unitaire — `bank_transaction_id`,
   `journal_entry_id`, `paid_amount`, `rounding_difference` (= les fractions de centime), `bank_fees`, `batch_size`, plus `proposal_type: "invoice_group"`
   et `invoices: [{ invoice_id, settled_amount }]` ; `settle_invoices_in_tx` et `pay_supplier_invoices_in_tx` **reçoivent
   `batch_size`** (taille du lot reçu).
3. **Signature de `pay_supplier_invoices_in_tx`** figée à T3, symétrique de `settle_invoices_in_tx` (`supplier_invoice_ids`,
   `batch_size`, `acteur`, `aujourd_hui`) ; `GroupRefusal` porte un identifiant de facture des **deux** familles (enum
   `GroupedDocument::Invoice(id) | SupplierInvoice(id)` ou deux champs), avec le numéro affichable.
4. **`GroupedInvoiceRef` fournisseur** : pas de `settlement_id` (la facture **est** son règlement) ; champs `kind`, `id`,
   `number`, `line_id`, `amount`.
5. **`cancelBlockedInvoice`** : `entry_cancel_outlook` rend, pour un groupe, la facture du premier motif par facture ;
   `ReconciliationKind::GroupedSettlement` reste `Copy` (aucun `Vec` dans la variante ; la liste se lit à part).
6. **Variantes de `GroupRefusal`** de la table ci-dessus, `UnsupportedTransaction` (devise, montant nul, D6) comprise,
   refusée avant tout verrou.

## Tests

`crates/kesh-api/tests/reconciliation_group_e2e.rs` (neuf) : AC1 à AC10, chacun avec sa sonde négative ; un test par code
de la table. Rouges d'abord : `kesh-gate-logs/e432/rouges.log` ; mutations : `kesh-gate-logs/e432/mut/` — (a) convertir un
`Err(DbError)` en `AppError` (AC4 rougit) ; (b) écrire `reconciliation.accepted` aussi dans la route (le test qui compte
les traces rougit) ; (c) calculer la différence au centime (AC6 rougit) ; (d) retirer la garde CHF (AC8 rougit) ; (e)
retirer le repli `LIKE` du nom court (AC5 rougit).

## Tâches

- [ ] **T0** — Étape 0 : LOW restants (Dev Notes) ; signatures du commit T3 de la 43-1a relues contre § *Exigences* ;
  écart → au Change Log et à l'orchestrateur.
- [ ] **T1** — Variante client, pré-vol, `bank_transaction_id()`, extracteur à cinq types, aiguillage (AC1).
- [ ] **T2** — `reconciliation_group.rs` : `invoice_group`, conversions D2, garde D6 (AC2, AC4, AC6, AC8).
- [ ] **T3** — Recherche (AC5). — [ ] **T4** — Vue et annulation (AC7). — [ ] **T5** — `docs/api-external.md` (AC9) ;
  accès (AC10).
- [ ] **T6** — **Commit D4** : `supplier_invoice_group` et sa famille partout, message à six types (AC3).
- [ ] **T7** — Gates ; Dev Agent Record ; prompt de revue P1 (lentilles B, E, A ; `docs/api-external.md` nommé).

## Note de version

- **Ajouté (API)** — `POST /reconciliation/accept` accepte `invoice_group` et `supplier_invoice_group` ;
  `GET /reconciliation/transactions/{id}/group-candidates` ; la vue et l'annulation d'un rapprochement rendent
  `grouped_settlement`, la liste des factures et la facture qui bloque (refs #823).

## Dev Notes

### Modules touchés

`kesh-api/routes/reconciliation` (variantes, pré-vol, vue, annulation), `routes/reconciliation_group` (neuf), `routes/mod.rs`,
`lib.rs`, `docs/api-external.md`. `audit_labels.rs` inchangé (aucune action neuve). Les codes neufs
(`RECONCILIATION_GROUP_INVALID`, `RECONCILIATION_GROUP_DIFFERENCE_TOO_LARGE`, `RECONCILIATION_UNDERPAYMENT`,
`RECONCILIATION_GROUP_UNSUPPORTED_TRANSACTION`)
sont des chaînes de `FailedProposal`, sauf le dernier (réponse `409` de la recherche) — vérifier à l'étape 0 si
`errors.rs` doit le porter.

### LOW à l'étape 0

- **L-4** : rouges et mutations sous `kesh-gate-logs/e432/` (repris aux *Tests*).
- **L-5** : MCP — constat du 2026-11-17 (rapport P1) : aucun fichier de `crates/kesh-api/src/mcp` ne cite `reconciliation`
  ni `ReconciliationKind` ; rien à faire, à reconfirmer par `git grep` à l'étape 0.
- **L-7** : les libellés des codes neufs et les catalogues i18n sont portés par la **43-3** ; son test
  `failed-proposal-label.test.ts` recompte les codes dans `reconciliation.rs` **et** `reconciliation_group.rs`.
- **Échelle des `Decimal`** : vérifier que la sérialisation rend quatre décimales pour les champs bruts (`"0.0100"`) ; sinon
  formater explicitement.

### Fichiers communs

Avec la 43-1a (`grouped_settlements.rs`, lu) ; avec la 43-3 (`reconciliation.types.ts`, écrit par elle) ; avec la 42-1
(`lib.rs`, `docs/api-external.md` : blocs distincts, conflit résolu en gardant les deux).

## Dev Agent Record

### Agent Model Used

### Completion Notes List

### File List

## Change Log

- 2026-11-17 — Fiche rédigée (kickoff de l'Epic 43).
- 2026-11-17 (soir) — Révisée : tiers mixtes, recherche par texte, écart, audit par le dépôt, type fournisseur retirable.
- 2026-11-17 (nuit) — Remédiée après la validation P1 (0/2/7/8) : H1 (D2, conversion unique des `DbError`, rejeu),
  H2 (D5, montants bruts), M1 (élément d'`accepted[]`), M2 (clés d'audit, exigences sur la 43-1a), M3 (recherche selon
  `docs/search-patterns.md`, tri puis troncature), M4 (candidate fournisseur, `kind` snake_case), M5 (vue, annulation,
  `cancelBlockedInvoice`), M6 (garde CHF, D6), M7 (pré-vol et ordre des refus), M8 (table complète des codes) ; LOW L-1,
  L-2, L-3, L-6, L-8 appliqués ; L-4, L-5, L-7 en Dev Notes.
- 2026-11-17 (nuit, suite) — Alignée sur la remédiation P2 de la 43-1a et sur Q4 : refus de projet retirés (H-B), refus de
  date (M-B), différences décomposées et refus `OVERPAYMENT` / `UNDERPAYMENT` / `WRITE_OFF_ACCOUNT_NOT_CONFIGURED`
  (proposition M-A en attente de Guy, D8), plafond réglable (D7).
- 2026-11-17 (nuit, P3 de la 43-1a) — Signes de `difference` et `bankFees` (M4, une définition, `group_difference`) ;
  `ACCOUNT_NOT_POSTABLE` retiré de la table (L2).
