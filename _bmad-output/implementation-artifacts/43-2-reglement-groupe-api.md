# Story 43-2 : un règlement pour plusieurs factures — l'API de la réconciliation (#823)

Status: review

*Fiche rédigée le 2026-11-17 par l'agent de rédaction de l'Epic 43 (Opus 5.5), workflow `bmad-create-story`, code lu à
`8b50fd195` ; révisée le soir même (décisions de Guy), puis **remédiée après sa validation P1**
(`kesh-gate-logs/e43/validation-43-2-p1.md`, 0/2/7/8 : H1, H2, M1-M8 appliqués ; LOW d'une ligne appliqués, les autres en
Dev Notes, étape 0), puis après ses validations P2 et P3, et sur la **décision de Guy du 2026-11-17 sur la différence**
(`kesh-gate-logs/e43/decision-difference.md`). Release **0.19.0**. **La recherche (tâche TR : dépôt et route) ne dépend
d'aucun commit de la 43-1a et part tout de suite** ; le reste part du commit T3 de la 43-1a, dont le contrat est celui
figé dans `kesh-gate-logs/e43/t3.md` (commit `241839568`, branche `story/43-1a-reglement-groupe-modele`) ; se fusionne
après elle. Les champs fournisseur de la vue (AC7) dépendent de l'AC6 bis de la **43-1b**. Aucune migration. Décisions : C-43-1 à
C-43-26 (fichier d'épique), qui font foi. Validation terminée en P5 (0/0/0/5) ; correction de C-43-25 (arrondi automatique) relue par une passe ciblée.*

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
  reçu, Story 27-4, :1764-1766), `journal_entry_id`, `sale_journal_entry_id`, `settled_amount`, `paid_amount`,
  `rounding_difference` (près de :3003-3075).
- Aucune garde de devise dans `accept_one_invoice` ni `accept_one_supplier_invoice` ; la garde CHF n'est que dans les
  propositions (:506, :1101) et le rattachement (`transaction_can_be_linked_to_entry`,
  `repositories/reconciliation_link.rs:535` : montant non nul et `currency == "CHF"` ; miroir écran `canLink`).
- Recherche : `docs/search-patterns.md` (§ 1-4 et 6 : `contacts.name` en FULLTEXT `ft_contacts_name`, jetons de 3
  caractères au moins, préfixe seulement ; numéros et références en `LIKE … ESCAPE` ; disjonction dans les deux branches) ;
  `crates/kesh-db/src/util/search.rs` (`escape_like` :24, `escape_boolean_ft` :121).
- Vue `ReconciliationTransactionResponse` (:5821) : `(invoice_id, supplier_invoice_id)` par un `match` exhaustif sur
  `ReconciliationKind` (:5869-5875) ; `CancelReconciliationResponse` (:5896-5905) : **ni `kind` ni liste** —
  `bank_transaction`, `reversal_journal_entry_id`, `invoice_id`, `supplier_invoice_id`. Routes sœurs :
  `/reconciliation/transactions/{id}/link-candidates` et `…/entry-candidates` (`lib.rs:886-943`).
- Pré-vol réel de `post_accept` : lecture de **toutes** les transactions du lot (`find_by_ids_for_account`, Step 0ter,
  :1729-1740) et classement par `classify_batch_transaction` (:1530-1550 : `BANK_TRANSACTION_NOT_FOUND`,
  `RECONCILIATION_ALREADY_RECONCILED`, `RECONCILIATION_ALREADY_REJECTED`) **avant** tout traitement ; verrou du compte pris
  **une fois pour le lot** dans `accept_once` (:1767-1812), avant `accept_one`.
- Garde de sens **unique** : `refuse_wrong_direction` (après `refuse_non_credit_settlement`, près de :2258-2300) — sens
  contraire et montant nul en `RECONCILIATION_TRANSACTION_NOT_CREDIT` / `NOT_DEBIT`, `details.reason` =
  `debit_transaction` / `credit_transaction` ou `zero_amount_transaction`, et `details.transactionAmount` ; publié par
  `docs/api-external.md:679`. Refus de devise existant : `RECONCILIATION_CURRENCY_MISMATCH`, `details { transactionCurrency,
  expected: "CHF" }` (:4170-4179, proposition `rule`), libellé `failed-proposal-label.ts:271`.
- Erreurs du lettrage : `lettering_error_to_failed_proposal` (:2191-2217) classe en `INTERNAL_ERROR` (après
  `tracing::error!`) `Invariant`, `NotFound` et les refus de lettrage exclus par construction (`LetteringConcurrentChange`,
  `LetteringLineAlreadyLettered`, …), le reste en `DATABASE_ERROR` ; doc-comment :2180-2190 (« `DATABASE_ERROR` invite au
  rejeu »).
- Route sœur de recherche : `get_link_candidates` (près de :6348) — `bank_account_of_transaction`, `link_preflight` (compte
  archivé → `BankAccountNotFound`, clos → `409 BANK_ACCOUNT_CLOSED`, sans compte du grand livre → `412
  BANK_ACCOUNT_NOT_CONFIGURED`), puis transaction rapprochée → `AppError::ReconciliationTransactionNotPending` (`404
  RECONCILIATION_TRANSACTION_NOT_PENDING`) ; elle ne contrôle pas le rejet. Principe écrit à `link_preflight` : « la liste
  n'offre jamais ce que le `POST` refuserait ». Borne de date : `InvoiceDateWindow::latest_invoice_date`
  (`repositories/reconciliation.rs:546-551`).
- Aucune requête `MATCH(` dans `crates/kesh-api/src` : les recherches vivent dans les dépôts de `kesh-db`
  (`docs/search-patterns.md` § 4, § 6) ; le reste dû dérivé est une jointure de `kesh-db` (`amount_due_derived_joins`).

## Décisions propres à la story

- **D1** — Code neuf dans `crates/kesh-api/src/routes/reconciliation_group.rs` (module déclaré dans `routes/mod.rs`) ;
  `reconciliation.rs` ne gagne que les variantes, le pré-vol, l'aiguillage, le texte de l'extracteur et les champs de la vue.
  **La requête de recherche vit dans un dépôt neuf, propre à la 43-2** (M6) :
  `crates/kesh-db/src/repositories/reconciliation_group_candidates.rs` (`find_group_candidates`), non partagé avec la
  43-1a ni la 43-1b. **`kesh-db` est donc touché : gate complet même en cours de boucle** (`CLAUDE.md`, exception
  `kesh-db`), assumé.
- **D2 — Aucune erreur d'un groupe ne sort au lot (H1, M1).** Les refus de `GroupRefusal` deviennent des `FailedProposal` par
  **une seule** fonction, `group_refusal_to_failed_proposal(bank_transaction_id, GroupRefusal)` ; tout `Err(DbError)` des
  fonctions de groupe passe par `group_error_to_failed_proposal(bank_transaction_id, DbError)`, qui **délègue** à
  `lettering_error_to_failed_proposal` (:2191-2217) : même classement que l'unitaire (`INTERNAL_ERROR` pour `Invariant`,
  `NotFound` et les refus de lettrage exclus par construction ; `DATABASE_ERROR` avec `details.message` sinon) — jamais une
  `AppError`. L'interblocage (1213) est ainsi absorbé comme dans l'unitaire et le lot rejoué par `accept_batch`. Un corps
  illisible reste un `400` du lot.
- **D3 — Audit écrit par le dépôt** (C-43-9) : la route n'écrit **aucune** trace pour un groupe.
- **D4 — Type fournisseur retirable** (C-43-18) : la variante `SupplierInvoiceGroup`, sa branche du pré-vol, du `match` de
  `bank_transaction_id()` et de l'aiguillage, le **message de l'extracteur passé de cinq à six types**, la famille
  fournisseur de la recherche et de la vue, et leurs tests vivent dans **un seul commit** à part, retirable si la 43-1b
  sort ; un test fixe le texte du message à cinq types avant ce commit, à six après. *Annoté le 2026-11-18 : la 43-1b est
  close ce jour et D4 sort avec elle ; le commit n'est plus retirable (§ Ordre de retrait).*
- **D5 — Montants bruts de bout en bout (H2, C-43-21).** Un reste dû peut porter quatre décimales (10.0050) ; le serveur
  calcule l'écart sur le brut. Le contrat rend donc le **brut** (`amountDueRaw`, `difference`, `settledInvoices[].amount`,
  chaînes à quatre décimales) **et** le centime pour l'affichage (`amountDue`). L'écran calcule en entiers de 1/10 000 sur le
  brut (43-3). La différence `d = |montant| − Σ apports bruts` (valeur absolue du montant de la transaction, comme
  `group_difference`, 43-1a) se décompose (D8 bis) en **`e`**, `d` arrondi au centime, sur le compte choisi, et **`r = d −
  e`**, la fraction sous le centime, sur le compte d'arrondi ; seul `|e|` se compare au plafond.
- **D6 — Sens, montant nul, devise : les gardes existantes (M2, C-43-22).** Dans cet ordre, sur la transaction rechargée :
  (1) **sens et montant nul** par `refuse_wrong_direction` — `RECONCILIATION_TRANSACTION_NOT_CREDIT` / `NOT_DEBIT`, mêmes
  `reason` et `details` que l'unitaire, `zero_amount_transaction` compris (atteignable : c'est cette garde qui le rend) ;
  (2) **devise** : non CHF → `RECONCILIATION_CURRENCY_MISMATCH`, `details { transactionCurrency, expected: "CHF" }`, forme de
  :4174. Aucun code neuf, aucun `reason: "currency" | "zero_amount"` sous `RECONCILIATION_GROUP_INVALID`. La recherche,
  alignée sur `get_link_candidates` (P3, M2), rend **`200`, `direction: null`, `candidates: []`** pour un montant nul ou une
  devise non CHF — **aucun `409` neuf** ; écran : la condition reste `canLink`, et la modale a un état « aucune facture à
  proposer pour cette transaction ».
- **D7 — Plafond réglable** (décision de Guy, Q4 ; C-43-24). La valeur est le réglage
  `company_invoice_settings.group_settlement_max_difference`, lu au moment du geste (43-1a AC3 bis) ; la constante
  `DEFAULT_GROUP_DIFFERENCE_MAX` n'en est que la valeur par défaut. La recherche rend le réglage (`maxDifference`), les
  refus le portent (`details.maxDifference`), `docs/api-external.md` le nomme sans recopier de valeur figée hors de la
  mention « 5.00 par défaut » ; un test règle 1.00 **par le dépôt ou un `UPDATE` direct**, jamais par
  `PUT /company/invoice-settings` (la 43-7 peut sortir la première, L2), et vérifie `maxDifference` et le refus.
- **D8 — Différences : décision de Guy du 2026-11-17** (C-43-16, `kesh-gate-logs/e43/decision-difference.md`) :
  l'écart au centime `e` (D8 bis ; moins `e < 0`, plus `e > 0`) est **admis dans les deux sens**, pour les deux familles,
  sur le compte **choisi par l'utilisateur** (`differenceAccountId`, obligatoire dès que `e ≠ 0`), dans la limite du
  plafond (`|e| ≤` plafond ; `r` n'y est jamais soumis). Plus de refus `RECONCILIATION_OVERPAYMENT`
  ni `RECONCILIATION_UNDERPAYMENT` pour un groupe ; le code neuf `RECONCILIATION_UNDERPAYMENT` disparaît. Aucun refus de
  projet (C-43-20).
- **D8 bis — Décomposition de l'écart (décision de l'orchestrateur, version définitive ; C-43-25 ;
  `kesh-gate-logs/e43/decision-difference.md`, § « Décomposition de l'écart », dont le tableau fait foi)** :
  - `d = |montant| − Σ restes bruts`, sur quatre décimales ;
  - **`e` = `d` arrondi au centime**, demi-centime arrondi en s'éloignant de zéro, comme `amount_due_to_centime` ;
  - **`r = d − e`**, la fraction sous le centime (`|r| ≤ 0.005`).

  `r` va au **compte d'arrondi**, comme pour le règlement d'une seule facture (aucune ligne si `r = 0`) ; `e` va au **compte
  choisi** s'il n'est pas nul — c'est la « différence » au sens de Guy, et la seule que borne le plafond. L'**arrondi
  automatique** est le cas `e = 0` (`|d| < 0.005`) : aucun compte demandé, ni l'API ni l'écran ne parlent de différence.
  `r` et `e` peuvent être non nuls ensemble ; aucune ligne à quatre décimales n'est écrite sur le compte choisi. *(Remplace
  les formes précédentes du 2026-11-18 : `centime(n × 0.005)`, puis `|d| < 0.005` avec `gap = d` brut.)*

  **Ordre — celui du code** (`group_difference`, `grouped_settlements.rs:306-322`, puis `settle_client_inner`, :664-716) :
  (1) **le plafond d'abord** — `|e| >` réglage → `DifferenceTooLarge`, rendu **dans** `group_difference`, avant tout
  contrôle de compte (un écart trop grand **sans** compte rend donc `RECONCILIATION_GROUP_DIFFERENCE_TOO_LARGE`, non
  `difference_account_required`) ; (2) le **compte d'arrondi, requis dès que `r ≠ 0`**, avec ses contrôles habituels
  (`ROUNDING_ACCOUNT_NOT_CONFIGURED`, compte d'arrondi égal à une créance) ; (3) `DifferenceAccountRequired` ; (4) les
  contrôles D9 du compte choisi.
- **D9 — Compte de différence interdit, avec les refus existants** (P3 M1, P4 M2) — aucun code neuf ; **ordre de la
  contrepartie choisie** (`is_bank_ledger`, doc près de :709-713 ; flux ventilé :3663-3711 et règle :4100-4139) :
  1. compte égal au compte de la banque de la transaction → `VALIDATION_ERROR`, `reason: "counterparty_equals_bank_ledger"`
     (`counterparty_is_bank_ledger_failed_proposal`, près de :726) — d'abord, pour ne pas inviter à agir sur la mauvaise
     chose ;
  2. compte inexistant, archivé ou d'une autre société → **`ACCOUNT_NOT_FOUND`**, `details { missingAccountIds: [id] }`
     (convention des flux ventilé et règle, libellé existant « Un compte de contrepartie est introuvable ou archivé ») —
     plus d'`INACTIVE_OR_INVALID_ACCOUNTS` ici ;
  3. compte non imputable → `ACCOUNT_NOT_POSTABLE` (`non_postable_failed_proposal`, :682, qui exige le **numéro** du compte :
     porté par la variante, ou lu par la route) ;
  4. compte égal à un compte de tiers **des factures du groupe** (leurs comptes de créance ou de dette ; un compte collectif
     d'une autre famille, 2000 pour un groupe client par exemple, n'est pas interdit — L5) → `SETTLEMENT_COUNTERPARTY_IS_CLAIM_ACCOUNT`,
     `role: "counterparty"`, `claim`, **sans `bankAccountId`** ;
  - compte absent alors qu'un compte est requis → `RECONCILIATION_GROUP_INVALID`, `reason: "difference_account_required"`.
  **Discriminant des deux causes de `role: "counterparty"`** (P4, M1) : le même code et le même rôle disent aussi « le compte
  de créance d'une facture est le compte de la banque » (43-1a étape 5, configuration). La route les sépare **sûrement** en
  comparant `details.accountId` au `differenceAccountId` envoyé ; `details.bankAccountId` n'est présent **que** dans le cas
  banque. `docs/api-external.md` documente les deux. Les rôles de `SETTLEMENT_COUNTERPARTY_IS_CLAIM_ACCOUNT` restent ceux du
  code : `counterparty`, `rounding`, `write_off_nature`, `vat_payable` ; le compte d'arrondi égal à une créance garde
  `rounding` (le dépôt de la 43-1a ne le compare qu'aux créances ; « égal à la banque » est à vérifier à l'étape 0 contre
  l'unitaire : soit un test l'exerce, soit la mention reste retirée — P5, L4). Un compte de différence qui est le compte du grand livre d'**une autre** banque est admis (seule la
  banque de la transaction est interdite), comme pour le rapprochement manuel : phrase à `docs/api-external.md`.

## Contrat

### `POST /api/v1/reconciliation/accept` — deux types neufs

```json
{ "bankAccountId": 3, "proposals": [
  { "type": "invoice_group", "bankTransactionId": 41, "invoiceIds": [12, 15, 19], "differenceAccountId": 6940 },
  { "type": "supplier_invoice_group", "bankTransactionId": 42, "supplierInvoiceIds": [7, 8] }
] }
```

`differenceAccountId` (facultatif, entier > 0 ; ≤ 0 → `400 VALIDATION_ERROR` au pré-vol) : le compte de l'écart, choisi
par l'utilisateur (D8) ; **obligatoire dès que l'écart au centime `e` est non nul** (D8 bis), ignoré sinon. *(La 43-5 et `paymentEntryIds` sont
sortis de la 0.19.0.)*

**Pré-vol (M7)**, dans `post_accept`, comme les types existants : un identifiant ≤ 0 dans `invoiceIds` /
`supplierInvoiceIds` → `400 VALIDATION_ERROR` du lot ; la taille du corps reste bornée par la limite existante. **Ordre
réel des refus (M3)** : (a) pré-vol par lot de `post_accept` — chaque transaction lue et classée (`BANK_TRANSACTION_NOT_FOUND`,
`RECONCILIATION_ALREADY_RECONCILED`, `RECONCILIATION_ALREADY_REJECTED`) ; (b) verrou du compte, une fois pour le lot
(`accept_once`) ; (c) par proposition, dans `settle_invoices_in_tx` (43-1a, étape 2), après son `SAVEPOINT` et **sous le
verrou du compte**, avant toute écriture et tout verrou de ligne : tableau vide ou d'un élément, plus de 50, doublon →
`RECONCILIATION_GROUP_INVALID` (`reason: "too_few" | "too_many" | "duplicate"`) — **une seule** définition, dans le dépôt
(aucune copie dans `post_accept`) ; (d) transaction rechargée, sens, montant nul, devise (D6) ; (e) factures. Test de
l'ordre : transaction inexistante et une seule facture → `BANK_TRANSACTION_NOT_FOUND`, non `too_few`.

**Élément de `accepted[]` pour un groupe (M1)** — la structure complète :

```json
{ "bankTransactionId": 41, "invoiceId": 0, "supplierInvoiceId": null, "journalEntryId": 905,
  "score": { "...": "sous-scores à 0, total conventionnel 1.0, comme le type rule" },
  "difference": "-0.4000", "roundingDifference": "0.0000", "gap": "-0.40", "differenceAccountId": 6940,
  "settledInvoices": [ { "invoiceId": 12, "amount": "500.0000" }, { "invoiceId": 15, "amount": "300.0000" } ] }
```

Une seule définition pour les deux familles, celle de `group_difference` (43-1a D4 bis) : `difference = |montant| − Σ
apports bruts` (signé, brut : négatif quand moins reçu ou moins payé, positif sinon) ; **`roundingDifference` = `r`**, la
fraction sous le centime passée au compte d'arrondi (brut, signé, `|r| ≤ 0.005`) ; **`gap` = `e`**, `d` arrondi au centime,
passé au compte choisi (**au centime, signé** : négatif, la banque a reçu ou payé moins que les factures ; positif, plus ;
`"0.00"` dans l'arrondi automatique) — il remplace `bankFees` du contrat précédent ; `differenceAccountId` = le compte
choisi, `null` quand `gap` est nul. Invariant : `difference = roundingDifference + gap` ; les deux peuvent être non nuls
ensemble (D8 bis).

`invoiceId` vaut `0` (sentinelle documentée, `docs/api-external.md:679`), `supplierInvoiceId` `null` ; pour un groupe
fournisseur, `settledInvoices[].supplierInvoiceId`. `journalEntryId` existe déjà ; seuls `difference`, `roundingDifference`,
`gap`, `differenceAccountId` et `settledInvoices` sont neufs (additifs, absents pour les autres types). Montants au brut
(D5), sauf `gap`, au centime. Le type TypeScript `AcceptedProposal` gagne **cinq** champs facultatifs ; un test de la 43-3
désérialise cet élément.

**Refus (`failed[]`, `200`) — table complète variante de `GroupRefusal` → code (M8)**, construite par
`group_refusal_to_failed_proposal` seule. `details` porte toujours `invoiceId` ou `supplierInvoiceId` quand une facture est
en cause, et **`invoiceNumber`** (numéro affichable : `invoice_number` client, numéro interne ou à défaut numéro du
fournisseur) pour que l'écran la nomme (43-3).

| Variante (43-1a / 43-1b) | `errorCode` | `details` |
|---|---|---|
| `TooFew`, `TooMany`, `Duplicate` | `RECONCILIATION_GROUP_INVALID` | `reason` : `too_few`, `too_many`, `duplicate` (+ `invoiceId`) |
| `CurrencyMismatch` | `RECONCILIATION_CURRENCY_MISMATCH` | `transactionCurrency`, `expected: "CHF"` (forme de :4174) |
| `TransactionNotFound` | `BANK_TRANSACTION_NOT_FOUND` | — |
| `TransactionNotPending` | `RECONCILIATION_ALREADY_RECONCILED` | — |
| `WrongDirection` (sens contraire ou montant nul) | `RECONCILIATION_TRANSACTION_NOT_CREDIT` / `…_NOT_DEBIT` | par `refuse_wrong_direction` : `reason` (`debit_transaction` / `credit_transaction` / `zero_amount_transaction`), `transactionAmount` |
| `InvoiceNotFound` | `INVOICE_NOT_FOUND` / `SUPPLIER_INVOICE_NOT_FOUND` | id |
| `InvoiceNotEligible { reason }` | `RECONCILIATION_INVOICE_NOT_ELIGIBLE` | `reason` : `invoice_not_validated`, `invoice_already_paid`, `invoice_journal_entry_not_set`, `nothing_due`, `supplier_invoice_not_open`, `supplier_invoice_in_payment_batch`, `race_during_update` (version) |
| `HeldByPaymentBatch` | `RECONCILIATION_SUPPLIER_PAYMENT_HOLD` | `reason: "other_invoice_in_payment_batch"`, ids |
| `SaleEntryMalformed` | `INVOICE_SALE_ENTRY_MALFORMED` | `reason: "no_debit_line_on_sale_entry"` |
| `BankAccountNotConfigured` / `NotFound` / `Closed` | `BANK_ACCOUNT_NOT_CONFIGURED` / `BANK_ACCOUNT_NOT_FOUND` / `BANK_ACCOUNT_CLOSED` | `bankAccountId` pour `NOT_CONFIGURED` et `CLOSED`, comme l'unitaire (revue P1, B6) ; — pour `NOT_FOUND` |
| `ClaimAccount { claim, role }` (compte de tiers égal à la banque, ou compte d'arrondi ou de différence égal à un compte de tiers des factures du groupe) | `SETTLEMENT_COUNTERPARTY_IS_CLAIM_ACCOUNT` | comme `claim_account_failed_proposal` : `accountId`, `accountNumber`, `claim`, `role` ∈ rôles **réels** — `counterparty` (créance ou dette = banque, **avec `bankAccountId`** ; **compte de différence choisi**, **sans `bankAccountId`**, `accountId` = `differenceAccountId` envoyé — D9), `rounding` (compte d'arrondi) |
| `DifferenceAccountIsBankLedger` (contrôle 1) | `VALIDATION_ERROR` | `reason: "counterparty_equals_bank_ledger"` (D9) |
| compte de différence manquant, archivé ou d'une autre société (contrôle 2) | `ACCOUNT_NOT_FOUND` | `missingAccountIds: [id]` |
| `DifferenceAccountNotPostable` (contrôle 3) | `ACCOUNT_NOT_POSTABLE` | comme `non_postable_failed_proposal` (:682), numéro du compte compris |
| `DifferenceAccountInForeignCurrency` (Epic 37, après le contrôle 3 ; annotation de `t3.md` du 2026-11-18) | `ACCOUNT_IN_FOREIGN_CURRENCY` | `accountId`, `currency`, comme `DbError::AccountInForeignCurrency` (`errors.rs`) |
| `DifferenceAccountRequired` | `RECONCILIATION_GROUP_INVALID` | `reason: "difference_account_required"`, `gap` |
| `PaymentBeforeInvoiceDate` | `RECONCILIATION_INVOICE_NOT_ELIGIBLE` | `reason: "payment_date_before_invoice_date"`, `tolerance_days` (comme `date_refusal_failed_proposal`, :2316-2335) **plus** `invoiceId` / `supplierInvoiceId` et `invoiceNumber`, que `group_refusal_to_failed_proposal` **ajoute** (L5) |
| `DifferenceTooLarge` | `RECONCILIATION_GROUP_DIFFERENCE_TOO_LARGE` | `transactionAmount`, `sumApplied`, `gap`, `maxDifference` |
| `RoundingAccountNotConfigured` | `ROUNDING_ACCOUNT_NOT_CONFIGURED` | — |
| `PeriodLocked` / `LaterFiscalYearClosed` / `FiscalYearInvalid` | `PERIOD_LOCKED` / `LATER_FISCAL_YEAR_CLOSED` / `FISCAL_YEAR_INVALID` | comme les constructeurs existants (:603, :644) |

Plus de lignes `Overpayment`, `Underpayment` ni `BankFeesAccountNotConfigured` (décision de Guy, D8). L'écriture se crée
avec `enforce_postable = false`, comme l'unitaire (43-1a D7), ses comptes étant ceux des réglages et des pièces ; **le
compte de différence, choisi par l'utilisateur, est contrôlé à part** (D9).

Les constructeurs existants sont **appelés**, non recopiés. Une variante que la 43-1a ajouterait sans ligne ici fait
échouer la compilation (`match` exhaustif sans joker).

### `GET /api/v1/reconciliation/transactions/{id}/group-candidates?q=…&limit=…&exclude=…`

Chemin aligné sur les routes sœurs (L-1). Comptable+, lecture seule. **Refus, par les fonctions de `get_link_candidates`
réemployées (M5)** : transaction introuvable → `404 NOT_FOUND` (`bank_account_of_transaction`) ; compte archivé →
`BankAccountNotFound`, clos → `409 BANK_ACCOUNT_CLOSED`, sans compte du grand livre → `412 BANK_ACCOUNT_NOT_CONFIGURED`
(`link_preflight`) ; transaction rapprochée → `404 RECONCILIATION_TRANSACTION_NOT_PENDING` (garde `is_reconciled`) ;
**transaction rejetée** : **comme `get_link_candidates`**, qui ne contrôle pas le rejet — la liste est servie (l'écran
n'offre pas le bouton sur une ligne rejetée, et le `POST` la refuse au pré-vol par lot) ; non CHF ou montant nul → **`200`,
`direction: null`, `candidates: []`**, comme la route sœur (doc-comment près de :6338-6346) — **aucun `409` neuf, aucune
`AppError` neuve** (P3, M2). **Tous tiers.** Crédit → factures clients `validated`, reste dû > 0 ;
débit → factures fournisseur `open`, hors lot `generated`. **Borne de date (M4)** : seules les factures dont
`invoice_date <= InvoiceDateWindow::for_transaction(&t).latest_invoice_date` (client ; `for_supplier_transaction` pour un
débit) sont rendues — la liste n'offre jamais ce que le `POST` refuserait. Requête : `find_group_candidates` du dépôt neuf
(D1).

**Recherche `q` (M3, `docs/search-patterns.md`)** — `q` taillé ; vide → pas de filtre ; sinon, disjonction :
- **nom du tiers** : `MATCH(c.name) AGAINST(? IN BOOLEAN MODE)` avec `escape_boolean_ft(q)` suivi de `*` ; si le terme
  échappé n'a **aucun jeton de 3 caractères ou plus** (vide, « BL », « BL SA »), repli `c.name LIKE ? ESCAPE '\\'` avec
  `escape_like` (pas de silence sur un nom à jetons courts, `docs/search-patterns.md` § 3.1, L5) ;
- **numéros et référence** (`LIKE '%…%' ESCAPE '\\'`, `escape_like`) : client `invoice_number` ; fournisseur
  `internal_number`, `supplier_invoice_number`, `payment_reference` ;
- la disjonction est présente dans **les deux** branches (terme FULLTEXT vide ou non), comme le prescrit § 6.

**Tri puis troncature** : `imputation_order` est une fonction Rust, générique sur `PayableDocument`
(`repositories/reconciliation.rs:344-352`, `:416`) : le DTO du dépôt neuf **implémente** ce trait (ses trois méthodes). La
requête lit au plus **500** factures ouvertes filtrées, **`ORDER BY` la date puis l'identifiant** (échantillon déterminé
au-delà de 500, L2 ; plafond nommé `GROUP_CANDIDATES_SCAN_LIMIT`), la route les trie par `imputation_order`, puis tronque à
`limit` (50 par défaut, 100 au plus) ; `hasMore` vaut vrai si la troncature a coupé **ou** si le plafond de lecture a été
atteint. `exclude=12,15` (facultatif, 50 identifiants au plus) retire les factures déjà imputées par l'écran avant la
troncature.

```json
{ "direction": "client", "transactionAmount": "1250.00", "maxDifference": "5.00",
  "candidates": [
    { "kind": "invoice", "id": 12, "number": "F-2026-0012", "contactName": "Exemple SA",
      "date": "2026-10-03", "dueDate": "2026-11-02", "totalTtc": "800.00",
      "amountDue": "500.01", "amountDueRaw": "500.0050" } ],
  "hasMore": false }
```

Candidate **fournisseur (M4)** : `{ "kind": "supplier_invoice", "id", "internalNumber", "supplierInvoiceNumber",
"contactName", "date", "dueDate", "paymentReference", "amountDue", "amountDueRaw" }`. `maxDifference` (ici et dans
`details.maxDifference` du refus) est le réglage de la société (43-7) rendu à **deux décimales**, comme le `GET` des
réglages ; la colonne en porte quatre. — `amountDue` = `total_amount` au
centime, `amountDueRaw` = `total_amount` brut ; pas de `totalTtc` distinct. Valeurs de `kind` en **snake_case**
(`invoice`, `supplier_invoice`), comme `candidateType` (`reconciliation.types.ts:27`), partout
dans ce contrat. `transactionAmount` est **signé** comme la transaction (`-1250.00` pour un débit) ; la différence se
calcule sur la valeur absolue (L-2).

### Vue et annulation (M5)

- `GET /reconciliation/transactions/{id}` : `kind` gagne `"grouped_settlement"` ; pour un groupe, `invoiceId`,
  `supplierInvoiceId` et leurs numéros valent `null` ; champs additifs `groupedInvoices: [{ "kind": "invoice" |
  "supplier_invoice", "id", "number", "amount" }]` (vide hors groupe ; montant brut), **`gap` et `differenceAccount: { "id",
  "number", "name" } | null`**. **Source de chaque champ** (le contrat `t3.md` ne rend que
  `ReconciliationView::{grouped_invoices, cancel_blocked_invoice, grouped_difference_account_id}`) :
  - `groupedInvoices` ← `view.grouped_invoices` (dépôt ; bras fournisseur par la 43-1b, AC6 bis) ;
  - `gap` ← **calculé par la route** : `group_difference(|view.bank_transaction.amount|, <montants bruts de
    view.grouped_invoices>, Decimal::MAX).gap` (fonction `pub` pure de la 43-1a, `Decomposition.gap`, C-e431a-8), rendu
    au centime, signé ;
  - `differenceAccount.id` ← `view.grouped_difference_account_id` (le dépôt le relit par le même recalcul, ligne
    d'écart prise **par position** dans l'écriture) ; `number` et `name` ← une **lecture du compte** par cet identifiant
    (`accounts`, société de la requête) ; `null` si l'identifiant est `None` ;
  - aucun de ces champs ne se lit dans les **comptes désignés des réglages**, qui ont pu changer depuis (P4, L2) ; le
    recalcul porte sur les montants du règlement, non sur les réglages.

  Et **`cancelBlockedInvoice: { "kind",
  "id", "number" } | null`** — la facture qui porte le premier motif d'un refus d'annulation par facture (créditée, soldée,
  lettrage figé), lue de `entry_cancel_outlook` (43-1a). Le `match` de :5869-5875 gagne le bras.
- `POST /reconciliation/transactions/{id}/cancel` : pour un groupe, `invoiceId` et `supplierInvoiceId` valent `null` ;
  champs additifs `kind` (le code de `ReconciliationKind`, pour tous les rapprochements) et `cancelledInvoices: [{ "kind",
  "id", "number" }]` (vide hors groupe).

## Critères d'acceptation

- **AC1** — Extracteur à six types (D4 : cinq avant le commit fournisseur) ; type inconnu → `400 VALIDATION_ERROR` ;
  pré-vol (M7) : identifiant ≤ 0 → `400` ; `bank_transaction_id()` couvre les deux variantes.
- **AC2** — `invoice_group` : sous le verrou du compte, `settle_invoices_in_tx` ; succès → élément de M1 ; chaque
  `GroupRefusal` → sa ligne de la table, rien d'écrit pour cette proposition, les autres passent ; un `Err(DbError)` →
  `DATABASE_ERROR` / `INTERNAL_ERROR` par délégation à `lettering_error_to_failed_proposal` (D2) — test : une
  `LetteringLineAlreadyLettered` ne se force pas par l'API (refus exclu par construction) : **test unitaire** de
  `group_error_to_failed_proposal` sur des `DbError` construits (`LetteringLineAlreadyLettered` et `Invariant` →
  `INTERNAL_ERROR`, `Sqlx` → `DATABASE_ERROR`), comme pour une facture seule (L3). La route n'écrit aucune trace : le test lit `reconciliation.accepted` puis
  `invoice.paid` par facture, avec l'acteur et les clés snake_case de la 43-1a (`t3.md`) — dont **`gap`** et
  **`difference_account_id`** (la décision de Guy : compte choisi et montant), `rounding_difference` et `batch_size` ; plus
  de `bank_fees` (P4, L3).
- **AC3** — `supplier_invoice_group` (commit D4) : symétrique, `pay_supplier_invoices_in_tx`.
- **AC4** — Lot mixte (`invoice` + groupe valide + groupe refusé) : deux `accepted`, un `failed`, dans l'ordre ; un
  `invoice_group` sur un débit refusé `RECONCILIATION_TRANSACTION_NOT_CREDIT`, `reason: "debit_transaction"` (sonde de
  C-43-15) ; sur un montant nul, `reason: "zero_amount_transaction"` ; **ordre (M3)** : transaction inexistante et une
  seule facture → `BANK_TRANSACTION_NOT_FOUND` (pré-vol par lot), non `too_few` ; **interblocage** : deux
  lots concurrents dont les groupes partagent une facture — l'un aboutit, l'autre refuse la facture (`race_during_update` ou
  `invoice_already_paid`) ou est rejoué, jamais un `500` du lot.
- **AC5** — Recherche : crédit et débit ; tiers différents dans la même liste ; un test par champ de `q` (nom ≥ 3
  caractères, nom à jetons courts « BL SA » par le repli, numéro, numéro interne, numéro du fournisseur, référence) ;
  injection (`%`, `_`, `\`, `*`, `+`, `"` : aucun résultat parasite, aucune erreur) ; `exclude` ; tri puis troncature
  (`hasMore`) ; une facture d'une autre société jamais rendue ; **borne de date** : une facture datée au-delà de
  `latest_invoice_date` n'est pas rendue, la même à la date limite l'est (M4) ; **refus** : `404 NOT_FOUND`, `409
  BANK_ACCOUNT_CLOSED`, `412 BANK_ACCOUNT_NOT_CONFIGURED`, `404 RECONCILIATION_TRANSACTION_NOT_PENDING` ; EUR ou montant
  nul → `200`, `direction: null`, liste vide ; transaction rejetée → servie comme par `get_link_candidates` (P3, M2) ;
  ordre déterminé au-delà de 500 factures (L2).
- **AC6** — Différences (D5, D8, D8 bis définitive, D9) — **le tableau de `decision-difference.md` fait foi**, un test par
  ligne (`d`, `e` = `gap`, `r` = `roundingDifference`, lignes écrites) :

  | Cas | `difference` (`d`) | `gap` (`e`) | `roundingDifference` (`r`) | Effet |
  |---|---|---|---|---|
  | 2 × 10.0050 payés 20.01 | 0.0000 | 0.00 | 0.0000 | aucun écart, aucun compte, aucune ligne |
  | 2 × 10.0050 payés 20.02 | +0.0100 | +0.01 | 0.0000 | compte demandé (0.01) |
  | 500.00 + 300.00 payés 799.99 | −0.0100 | −0.01 | 0.0000 | compte demandé |
  | 3 × 10.0050 payés 30.06 | +0.0450 | +0.05 | −0.0050 | compte demandé (0.05) **et** ligne d'arrondi de −0.0050 (`r`) |
  | 10.0049 + 10.0000 payés 20.00 | −0.0049 | 0.00 | −0.0049 | arrondi automatique, sans compte |
  | 10.0050 + 10.0000 payés 20.00 | −0.0050 | −0.01 | +0.0050 | compte demandé (0.01) **et** ligne d'arrondi de +0.0050 (`r`) |

  Sans compte quand `e ≠ 0` → `RECONCILIATION_GROUP_INVALID`, `reason: "difference_account_required"` ; 500.00 + 300.00
  payés 799.60 avec un compte → `gap` `"-0.40"` ;
  - **ordre** (D8 bis) : **le plafond d'abord** — plafond réglé à 0.00, cas 20.02 **sans** compte →
    `RECONCILIATION_GROUP_DIFFERENCE_TOO_LARGE`, non `difference_account_required` ; `r ≠ 0` sans compte d'arrondi désigné
    (écart sous le plafond) → `ROUNDING_ACCOUNT_NOT_CONFIGURED`, **même si `e ≠ 0` et qu'aucun compte n'est choisi** ;
    `r = 0`, `e ≠ 0`, sans compte choisi → `difference_account_required`, jamais le refus du compte d'arrondi ;
  - **plafond sur `|e|` seulement** : plafond réglé à 0.00 → le cas 10.0049 (arrondi automatique) est accepté, le cas 20.02
    est refusé `RECONCILIATION_GROUP_DIFFERENCE_TOO_LARGE`, avec ou sans compte choisi ;
  - **plus** de 0.40 avec un compte → accepté, `gap` `"0.40"` (côté client et côté fournisseur) ; écart au-delà du réglage → `RECONCILIATION_GROUP_DIFFERENCE_TOO_LARGE` avec `maxDifference` = le réglage ;
  **contrôles du compte, dans l'ordre de D9, un test par cause** : = la banque → `VALIDATION_ERROR`,
  `counterparty_equals_bank_ledger` (même s'il est aussi non imputable) ; archivé, d'une autre société ou inexistant →
  `ACCOUNT_NOT_FOUND`, `missingAccountIds` ; non imputable → `ACCOUNT_NOT_POSTABLE` avec le numéro ; = le compte de créance
  d'une facture du groupe → `SETTLEMENT_COUNTERPARTY_IS_CLAIM_ACCOUNT`, `role: "counterparty"`, **sans `bankAccountId`**,
  `accountId` = le compte envoyé ; **et** le cas configuration (compte de créance d'une facture = la banque, 43-1a étape 5)
  → même code et même rôle, **avec `bankAccountId`** (sonde du discriminant, P4 M1) ; = le compte 2000 pour un groupe
  client → admis (L5) ; = le compte du grand livre d'une autre banque → admis ; `settledInvoices[].amount` au brut ; facture au projet
  archivé après coup acceptée ; paiement antérieur à une facture refusé (`payment_date_before_invoice_date`, `details` avec
  `invoiceId`, `invoiceNumber` et `tolerance_days`, L5).
- **AC7** — Vue et annulation : groupe client (et fournisseur, D4) — `grouped_settlement`, `groupedInvoices`,
  `cancelBlockedInvoice` sur une facture créditée ; réponse du `POST` (M5) ; refus par le code du premier motif ;
  `gap` et `differenceAccount` (numéro et nom) selon leurs sources (M5), pour `e > 0`, `e < 0` et `e = 0` (`null`) ;
  **un groupe n'est jamais un retrait seul** (C-e431a-6) : `expect_unlink_only = Some(true)` sur un groupe → `409
  RECONCILIATION_CANCEL_CHANGED` (le dépôt rend `ReconciliationCancelChanged { unlink_only: false }`), rien d'écrit. La
  partie fournisseur dépend de l'AC6 bis (vue) et de l'AC6 (dé-rapprochement) de la 43-1b.
- **AC8** — Garde CHF (D6, M2, M3) : `invoice_group` sur une transaction EUR → `RECONCILIATION_CURRENCY_MISMATCH`,
  `details { transactionCurrency: "EUR", expected: "CHF" }`, **rien d'écrit** (point de sauvegarde de la 43-1a) ; la même
  en CHF passe (sonde).
- **AC9** — `docs/api-external.md` : « Régler plusieurs factures par une transaction » (*depuis la v0.19.0*) — types,
  ordre des refus, recherche (règles de `q`, borne de date, codes, liste vide), table des codes, `differenceAccountId` et
  ses refus dans leur ordre, le discriminant des deux causes de `role: "counterparty"`, le compte d'une autre banque admis,
  la décomposition `e` / `r` de D8 bis (`e` au centime sur le compte choisi, `r` au compte d'arrondi, arrondi automatique
  quand `e = 0`) et l'ordre de leurs contrôles, `gap` (signé), élément d'`accepted[]`, brut et centime, plafond nommé, garde CHF,
  champs de la vue et de l'annulation, *Pattern batch*.
- **AC10** — Accès (L-6) : rôle Comptable+ ; clé d'API en écriture : `POST /accept` permis ; clé en lecture
  (`ApiKeyReadOnly`, `middleware/auth.rs:139-142`) : `POST /accept` refusé, `GET group-candidates` permis ; rôle
  inférieur : `403`.

## Exigences sur la 43-1a (historique ; le contrat qui fait foi est celui du commit T3)

**Le contrat de la 43-2 avec la 43-1a est celui du commit T3 de la 43-1a**, figé dans `kesh-gate-logs/e43/t3.md` (commit
`241839568`) ; l'agent qui développe la 43-1a a reçu les ajustements (`WrongDirection`, `CurrencyMismatch`,
`ClaimAccount.role` aux rôles réels, différence selon la décision de Guy : `difference_account_id`, deux sens, refus D9).
L'étape 0 relit `t3.md` contre la table des refus : **chaque** variante `DifferenceAccount*` doit y avoir sa ligne
et son code (`DifferenceAccountNotUsable` n'existe pas : la table est complète sur la liste finale de `t3.md`) — le `match` exhaustif le garantit à la compilation, non la justesse du
code choisi (P4, L6) ; un écart va au Change Log et à l'orchestrateur. Les points ci-dessous restent comme trace des
exigences transmises (les mentions de `bank_fees` y sont **périmées** : remplacées par `gap` et `difference_account_id`) :

0. *(P3, L4)* le point de sauvegarde propre de la 43-1a : un `ROLLBACK TO` qui échoue en 1305 après un 1213 **remonte en
   `Err(DbError)`**, jamais avalé en refus — test `is_savepoint_lost`.

1. **Forme brut/centime au commit T3** *(chemin périmé : le contrat final porte
   `GroupedSettlement.decomposition: Decomposition { difference, rounding_difference, gap }`)* :
   `GroupedSettlement.difference`, `rounding_difference` et `GroupedInvoiceRef.amount` en `Decimal` brut (quatre décimales), `gap` au centime ; lecture du plafond réglable
   (`DEFAULT_GROUP_DIFFERENCE_MAX` comme valeur par défaut seulement). *Reporté dans la 43-1a le 2026-11-17 (nuit).*
2. **Clés d'audit** : `reconciliation.accepted` en **snake_case** alignées sur l'unitaire — `bank_transaction_id`,
   `journal_entry_id`, `paid_amount`, `rounding_difference`, `gap`, `difference_account_id`, `batch_size`, plus `proposal_type: "invoice_group"`
   et `invoices: [{ invoice_id, settled_amount }]` ; `settle_invoices_in_tx` et `pay_supplier_invoices_in_tx` **reçoivent
   `batch_size`** (taille du lot reçu).
3. **Signature de `pay_supplier_invoices_in_tx`** figée à T3, symétrique de `settle_invoices_in_tx` (`supplier_invoice_ids`,
   `batch_size`, `acteur`, `aujourd_hui`) ; `GroupRefusal` porte un identifiant de facture des **deux** familles (enum
   `GroupedDocument::Invoice(id) | SupplierInvoice(id)` ou deux champs), avec le numéro affichable.
4. **`GroupedInvoiceRef` fournisseur** : pas de `settlement_id` (la facture **est** son règlement) ; champs `kind`, `id`,
   `number`, `line_id`, `amount`.
5. **`cancelBlockedInvoice`** : `entry_cancel_outlook` rend, pour un groupe, la facture du premier motif par facture ;
   `ReconciliationKind::GroupedSettlement` reste `Copy` (aucun `Vec` dans la variante ; la liste se lit à part).
6. **Variantes de `GroupRefusal`** de la table ci-dessus. *Mise à jour de la P2 de la 43-2 (à transmettre à la 43-1a en
   développement)* : `UnsupportedTransaction` devient `CurrencyMismatch` (devise seule) ; le sens et le montant nul sont
   une variante `WrongDirection` traduite par `refuse_wrong_direction` (montant porté) ; l'ordre de l'étape 3 est : sens et
   montant nul **d'abord**, devise ensuite ; `ClaimAccount` porte `claim` et un `role` parmi les rôles réels
   (`counterparty`, `rounding`) — *corrigé à la P3 : `receivable` est une valeur de `claim`, `bank_fees` n'existe pas*.

## Tests

`crates/kesh-api/tests/reconciliation_group_e2e.rs` (neuf) : AC1 à AC10, chacun avec sa sonde négative ; un test par code
de la table. Rouges d'abord : `kesh-gate-logs/e432/rouges.log` ; mutations : `kesh-gate-logs/e432/mut/` — (a) convertir un
`Err(DbError)` en `AppError` (AC4 rougit) ; (b) écrire `reconciliation.accepted` aussi dans la route (le test qui compte
les traces rougit) ; (c) calculer la différence au centime (AC6 rougit) ; (d) retirer la garde CHF (AC8 rougit) ; (e)
retirer le repli `LIKE` du nom court (AC5 rougit) ; (f) retirer la borne de date de la recherche (AC5 rougit) ; (g)
remplacer la délégation de D2 par un `DATABASE_ERROR` sans exception (le test unitaire d'AC2 rougit) ; (h) accepter un
compte de différence égal à la banque (AC6 rougit). Tests de
dépôt de `find_group_candidates` dans `crates/kesh-db/tests/reconciliation_group_candidates.rs` (neuf, squash).

## Tâches

- [x] **T0** — Étape 0 : LOW restants (Dev Notes) ; signatures du commit T3 de la 43-1a relues contre § *Exigences* ;
  écart → au Change Log et à l'orchestrateur.
- [x] **TR (d'abord)** — Recherche : `reconciliation_group_candidates.rs` (dépôt, **gate complet**) puis la route (AC5).
  *Nommée TR pour ne pas la confondre avec le « commit T3 » de la 43-1a* (P5, L5).
  **Ne dépend d'aucun commit de la 43-1a** (ni migration, ni dépôt de règlement) : elle part tout de suite, en parallèle.
- [x] **T1** — Variante client, pré-vol (`differenceAccountId` ≤ 0 compris), `bank_transaction_id()`, extracteur à cinq
  types, aiguillage (AC1). Attend le commit T3 de la 43-1a.
- [x] **T2** — `reconciliation_group.rs` : `invoice_group`, conversions D2, gardes D6 et D9 (AC2, AC4, AC6, AC8) ; les
  fonctions privées de `reconciliation.rs` qu'il emploie (`lettering_error_to_failed_proposal`, `refuse_wrong_direction`,
  `SettlementDirection`, `date_refusal_failed_proposal`, `claim_account_failed_proposal`, `period_locked_failed_proposal`,
  `non_postable_failed_proposal`, `counterparty_is_bank_ledger_failed_proposal`) passent en **`pub(super)`** (L1) ; aucun
  test d'intégration n'en a besoin (vérifié à l'étape 0 : `pub(super)` est invisible de `tests/`). — [x] **T4** — Vue et annulation (AC7). — [x] **T5** — `docs/api-external.md` (AC9) ;
  accès (AC10).
- [x] **T6** — **Commit D4** : `supplier_invoice_group` et sa famille partout, message à six types (AC3). Tests qui
  règlent un groupe fournisseur marqués `#[ignore]` (exigent la 43-1b).
- [x] **T7** — Gates ; Dev Agent Record ; prompt de revue P1 (lentilles B, E, A ; `docs/api-external.md` nommé).

## Note de version

- **Ajouté (API)** — `POST /reconciliation/accept` accepte `invoice_group` et `supplier_invoice_group` ;
  `GET /reconciliation/transactions/{id}/group-candidates` ; un écart (moins-payé ou trop-perçu, dans la limite du
  plafond) se comptabilise sur le compte `differenceAccountId` ; la vue et l'annulation d'un rapprochement rendent
  `grouped_settlement`, la liste des factures, l'écart et son compte, et la facture qui bloque (refs #823).

## Dev Notes

### Modules touchés

`kesh-api/routes/reconciliation` (variantes, pré-vol, vue, annulation), `routes/reconciliation_group` (neuf), `routes/mod.rs`,
`lib.rs`, `docs/api-external.md`, et **`kesh-db/repositories/reconciliation_group_candidates`** (neuf, propre à la 43-2,
M6) avec `repositories/mod.rs`. **`kesh-db` touché : gate complet même en cours de boucle**, assumé (exception `kesh-db`
du `CLAUDE.md`) ; le fichier est neuf et n'est partagé ni avec la 43-1a ni avec la 43-1b. `audit_labels.rs` inchangé
(aucune action neuve). Codes neufs, deux seulement, des chaînes de `FailedProposal` : `RECONCILIATION_GROUP_INVALID`,
`RECONCILIATION_GROUP_DIFFERENCE_TOO_LARGE`. **Aucune `AppError` neuve**, donc ni `errors.rs`, ni catalogues Rust, ni
garde G9 dans le périmètre : la recherche ne rend que des refus existants ou une liste vide (P3, M2), et le compte de
différence interdit réemploie des codes existants (D9). `RECONCILIATION_UNDERPAYMENT` n'est plus créé.

### Ordre de retrait (si le temps manque)

**Le commit fournisseur (D4, `70f74f964`) n'est plus retirable** : la **43-1b est close le 2026-11-18** et versée dans
`epic/43-transaction-plusieurs-factures` (`c22647e7a`) ; D4 sort avec elle (décision de l'orchestrateur du 2026-11-18,
revue P2, P2-1). La barrière et la procédure de retrait écrites à la remédiation P1 sont levées.

Si le temps manque : les champs de vue et d'annulation `cancelBlockedInvoice` et `cancelledInvoices` (la 43-3 écrit
alors la limite) ; jamais la recherche, l'acceptation (client et fournisseur) ni les refus de la différence.

### LOW à l'étape 0

- **L-4** : rouges et mutations sous `kesh-gate-logs/e432/` (repris aux *Tests*).
- **L-5** : MCP — constat du 2026-11-17 (rapport P1) : aucun fichier de `crates/kesh-api/src/mcp` ne cite `reconciliation`
  ni `ReconciliationKind` ; rien à faire, à reconfirmer par `git grep` à l'étape 0.
- **L-7** : les libellés des codes neufs et les catalogues i18n sont portés par la **43-3** ; son test
  `failed-proposal-label.test.ts` recompte les codes dans `reconciliation.rs` **et** `reconciliation_group.rs`.
- **Échelle des `Decimal`** : vérifier que la sérialisation rend quatre décimales pour les champs bruts (`"0.0100"`) ; sinon
  formater explicitement.
- **L6 (P3)** : `lettering_error_to_failed_proposal` journalise « encaissement : lettrage de la facture impossible » pour
  tout `Invariant`, y compris le « reste dû > 0 » de la 43-1a ; sans effet sur le contrat — paramétrer le message ou le
  laisser, à décider à l'étape 0.

### Fichiers communs

Avec la 43-1a (`grouped_settlements.rs`, lu ; `repositories/mod.rs`, une ligne chacune) ; avec la 43-3
(`reconciliation.types.ts`, écrit par elle) ; avec la 42-1 (`lib.rs`, `docs/api-external.md` : blocs distincts, conflit
résolu en gardant les deux).

## Dev Agent Record

### Agent Model Used

Opus 5.5 (agent de développement, en autonomie), 2026-11-17.

### Completion Notes List

- **Commits** : `b32c3d42f` (recherche, `invoice_group`, vue, annulation, guide d'API) ; `7994a7a34` (trois défauts de test) ;
  **`70f74f964` = commit D4** (type fournisseur, retirable) ; `6e656ad88` (test de `conclude`, mutation a) — **dernier commit
  de code de la story** ; `420bd6317` = fusion de la branche de la 43-1a (C-e432-12) ; puis fiches seules.
- **Tests ajoutés** (périmètre `cf77cff18`..`6e656ad88`, recomptés par `grep -c`) : `kesh-api/tests/reconciliation_group_e2e.rs`
  25 `#[sqlx::test]`, dont **3 `#[ignore]`** (groupe fournisseur : exigent la 43-1b, `pay_supplier_invoices_in_tx` encore
  `unimplemented!`) ; `kesh-db/tests/reconciliation_group_candidates.rs` 7 ; unitaires : 5 dans `reconciliation_group.rs`,
  1 dans `reconciliation_group_candidates.rs`.
- **Rouges** (`kesh-gate-logs/e432/rouges.log`) : le code de TR, T1, T2 et T4 a été écrit avant ses tests (aucun « rouge
  d'abord » strict, dit au journal) ; les huit rouges rencontrés étaient des défauts de test, corrigés (`7994a7a34`).
- **Mutations** (`kesh-gate-logs/e432/mut/campagne.py`, `campagne.log`, `campagne-a2.log`) : (b) à (h) **tuées** au commit
  `70f74f964` par les tests annoncés ; (a) **survivante** au premier passage (aucun test n'atteignait une `DbError` dans le
  chemin groupé), puis **tuée** par `the_conclusion_keeps_errors_in_the_proposal` (`6e656ad88`) — ⚠️ ce rejeu a été troublé :
  j'ai restauré le fichier à la main pendant qu'il tournait (17:47:29), le binaire muté était probablement déjà compilé ;
  verdict « probable », à rejouer si la revue le demande. (f) tue dix tests, dont des tests sans rapport avec la date :
  probablement par erreur SQL du motif muté plutôt que par l'assertion de borne.
- **Gates au passage en review** (base `kesh_e432` remise à neuf avant) :
  - `scripts/test-fast.sh --ci` au commit `420bd6317` : **5589 passés, 0 échec, 16 ignorés**
    (`kesh-gate-logs/e432/test-fast-review-2.log`) ;
  - `scripts/gate-ci.sh kesh_ci_e432` au commit `420bd6317` : **5596 passés, 0 échoué, 20 ignorés**
    (`kesh-gate-logs/e432/gate-ci-review.log`) ;
  - le premier `test-fast --ci`, au commit `6e656ad88` sur la base `241839568`, rendait 17 rouges, tous dans du code et des
    tests de la 43-1a, corrigés par elle après cette base (`test-fast-review.log`, C-e432-12) ;
  - frontend : non touché, non lancé ; **E2E non lancé** (règle `e35-dev-commun.md` : à la clôture).
- **Non exécuté** : les trois tests fournisseur `#[ignore]` (43-1b) ; aucune capture (aucun contrôle visible : l'écran est la
  43-3).
- **Choix** : C-e432-1 à C-e432-12 (`e0190-choix-autonomes-e432.md`). Code écrit par l'outil Edit seulement ; la campagne
  de mutations applique ses motifs par script (permis pour les mutations).

#### Remédiation de la revue P1 (2026-11-17 au soir, décisions de l'orchestrateur)

- **Commit de code** : `ed8ab9c97` (dernier commit de code). Revue P1 : 0 CRITICAL, 1 HIGH (B1/E1/A4), 6 MEDIUM, ~30 LOW
  (`kesh-gate-logs/e432/review-p1-{B,E,A}.md`).
- **Traité** : barrière de release de D4 (fiche, § *Ordre de retrait* ; épique, sous C-43-18) ; corps provisoire de
  `pay_supplier_invoices_in_tx` en `Err(DbError::Invariant)` (C-e432-13) ; `bankAccountId` des refus `BANK_ACCOUNT_CLOSED` et
  `…_NOT_CONFIGURED` (B6, C-e432-10 révisé) ; test en table des 25 variantes de `GroupRefusal` (A2/B4) ; `effective_limit`
  extraite et testée (E4/B9) ; tests B2, A3, B3, B10, B11, A14, E3, E2/A13, E6, A6 (dont le retrait seul, assertion ajoutée
  à `reconciliation_link_entry_e2e.rs`), A12 ; guide d'API A10, A11/B7, B12 ; registre E8, E9, E10 (C-e432-14 à 16) ; T7
  cochée (A9) ; journaux de gate en-têtés de leur commit (A7).
- **Non traité, par décision** : E5 (disjonction `hasMore` redondante tant que `limit ≤ 100 < 500`, non demandée) ; E8 écrit
  seulement (C-e432-14) ; E10 en issue proposée (C-e432-16).
- **Tests à `ed8ab9c97`** (recomptés par `grep -c`) : `reconciliation_group_e2e.rs` 31 `#[sqlx::test]` dont 3 `#[ignore]` ;
  `reconciliation_group_candidates.rs` (dépôt) 7 ; unitaires de `reconciliation_group.rs` 6.
- **Mutations** :
  - (f) de la première campagne était **non concluante** (motif invalide : `? IS NOT NULL` suivi du `?` du bind, erreur
    SQL) ; rejouée avec des mutants valides (`mut/campagne2.py`, `campagne2.log`, au commit `ed8ab9c97`) : (f1) client tuée
    par `client_candidates_follow_the_settlement_rules` et `la_recherche_d_un_credit_…`, (f2) fournisseur tuée par
    `supplier_candidates_follow_the_settlement_rules` ;
  - neuves, toutes tuées : (i) reste dû → TTC (`campagne2-i.log`, mutant corrigé après un premier motif qui ne compilait
    pas), (j) `HAVING` retiré, (k) statut client retiré, (l) TTC fournisseur `>= 0`, (m) tri retiré, (n) plafond de `limit`
    retiré, (o) `bankAccountId` retiré ;
  - (a) rejouée seule sans écriture concurrente (`campagne-a3.log`) : **tuée** par
    `the_conclusion_keeps_errors_in_the_proposal` et par `le_type_fournisseur_se_desserialise_et_ne_panique_pas`.
- **Gates** (base `kesh_e432` remise à neuf avant) :
  - ciblé : `kesh-gate-logs/e432/cible-3.log`, commit `ed8ab9c97`, 99 passés ;
  - `scripts/test-fast.sh --ci` : **5596 passés, 0 échec, 16 ignorés** (`test-fast-p1.log`, en-tête : commit `ed8ab9c97`) ;
  - `scripts/gate-ci.sh kesh_ci_e432` : **5603 passés, 0 échoué, 20 ignorés** (`gate-ci-p1.out`/`gate-ci-p1.log`, en-tête :
    commit `ed8ab9c97`) ;
  - frontend non touché ; E2E à la clôture.

### File List

- `crates/kesh-db/src/repositories/reconciliation_group_candidates.rs` (neuf), `repositories/mod.rs`,
  `repositories/reconciliation_cancel.rs` (champ `kind`, C-e432-4)
- `crates/kesh-db/tests/reconciliation_group_candidates.rs` (neuf)
- `crates/kesh-api/src/routes/reconciliation_group.rs` (neuf), `routes/reconciliation.rs`, `routes/mod.rs`, `lib.rs`
- `crates/kesh-api/tests/reconciliation_group_e2e.rs` (neuf) ; `crates/kesh-api/tests/reconciliation_link_entry_e2e.rs`
  (deux assertions du retrait seul, revue P1, A6)
- `crates/kesh-db/src/repositories/grouped_settlements.rs` (corps provisoire de `pay_supplier_invoices_in_tx`, C-e432-13 ;
  fichier de la 43-1a)
- `_bmad-output/planning-artifacts/epic-43-transaction-plusieurs-factures.md` (barrière de release de D4)
- `docs/api-external.md`
- `_bmad-output/implementation-artifacts/e0190-choix-autonomes-e432.md` (neuf), cette fiche, son prompt de revue P1

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
- 2026-11-18 — Remédiée après la validation P2 (0/0/6/6) : M1 (délégation à `lettering_error_to_failed_proposal`), M2
  (`refuse_wrong_direction` et `RECONCILIATION_CURRENCY_MISMATCH`, sens et montant nul avant la devise, plus de code de
  devise neuf), M3 (ordre réel du pré-vol, bornes dans le dépôt sous le verrou du compte, AC8 « rien d'écrit »), M4
  (borne de date de la recherche), M5 (codes réels de la recherche, transaction rejetée), M6 (dépôt neuf
  `reconciliation_group_candidates.rs`, gate complet assumé) ; L1 à L6.
- 2026-11-18 (P3) — Remédiée après la validation P3 (0/0/3/6) et sur la décision de Guy du 2026-11-17 sur la différence :
  `differenceAccountId` dans le `POST`, `gap` (signé) et `differenceAccountId` dans la réponse, `gap` et
  `differenceAccount` dans la vue ; plus de refus de sens de l'écart ; M1 (rôles réels, compte de différence interdit par
  les refus existants, D9, aucun code neuf) ; M2 (recherche : `200`, liste vide, `direction: null` pour EUR et montant nul,
  rejet comme `get_link_candidates`, aucune `AppError` neuve) ; M3 (contrat = commit T3 de la 43-1a, `t3.md`) ; L1
  (`pub(super)`), L2 (`ORDER BY`, `PayableDocument`), L3 (test unitaire, mutation g), L4 (1305 remonte), L5 (identifiants
  ajoutés aux refus de date), L6 (en Dev Notes) ; la recherche part sans attendre la 43-1a ; ordre de retrait.
- 2026-11-18 (P4) — Remédiée après la validation P4 (0/0/2/6) : M1 (discriminant des deux causes de `role: "counterparty"`,
  `bankAccountId` seulement dans le cas banque, comparaison à `differenceAccountId`, un test par cause), M2 (D9 aligné sur
  la contrepartie choisie : banque → manquant `ACCOUNT_NOT_FOUND` → non imputable avec numéro → compte de tiers des
  factures du groupe ; plus d'`INACTIVE_OR_INVALID_ACCOUNTS`) ; **arrondi automatique** (D8 bis, décision de
  l'orchestrateur : `|e| ≤ centime(n × 0.005)` au compte d'arrondi, aucun compte demandé ; `difference = roundingDifference
  + gap`) ; L2 (source de `gap` et du compte dans la vue), L3 (clés d'audit `gap`, `difference_account_id`), L4 (réglé par
  l'arrondi automatique), L5 (comptes de tiers des factures du groupe seulement), L6 (recompte de `t3.md`) ; compte d'une
  autre banque admis et documenté.
- 2026-11-18 (P5) — **Clôture de la validation.** Tendance : P1 Sonnet 0/2/7/8 → P2 Opus 0/0/6/6 → P3 Sonnet 0/0/3/6 → P4
  Opus 0/0/2/6 → **P5 Sonnet 0/0/0/5** : aucune passe au-dessus de LOW à la cinquième, la boucle se clôt. Modèles : Sonnet 5.5
  (P1, P3, P5), Opus 5.5 (P2, P4) ; remédiations par l'agent de rédaction (Opus 5.5). LOW de la P5 appliqués : L1 et L2
  (rendus sans objet par la correction ci-dessous), L3 (ordre : compte d'arrondi dans la zone automatique, puis
  `DifferenceAccountRequired`), L4 (« ou à la banque » du compte d'arrondi : à vérifier à l'étape 0), L5 (tâche de la
  recherche renommée TR). **Correction de l'orchestrateur sur C-43-25**, appliquée dans le même commit : l'arrondi
  automatique ne s'applique plus que si `|d| < 0.005` (`d = |montant| − Σ restes bruts`) ; sinon `gap = d` entier sur le
  compte choisi ; AC6 recalculé (20.01 arrondi ; 20.02 et 799.99 compte demandé ; 30.06 → `gap` 0.0450 ; bornes 0.0049 et
  0.0050). Cette correction change une règle : une passe ciblée la relit. Statut : `ready-for-dev`.
- 2026-11-18 (C-43-25 définitive) — Décomposition de l'écart selon `decision-difference.md`, § « Décomposition de l'écart » :
  `e` = `d` arrondi au centime sur le compte choisi (le seul que borne le plafond), `r = d − e` sur le compte d'arrondi ;
  arrondi automatique = `e = 0` ; `gap = e` au centime, `roundingDifference = r`, les deux pouvant être non nuls ; compte
  d'arrondi requis si `r ≠ 0`, vérifié avant `DifferenceAccountRequired` ; AC6 refait sur le tableau de la décision.
  Remplace l'entrée précédente sur ce point (« `gap = d` entier »). La passe ciblée relit ce commit et le précédent.
- 2026-11-18 (P6 ciblée) — LOW-1 (AC9 : « son seuil » → décomposition `e` / `r` et ordre des contrôles), LOW-3 (tableau
  d'AC6 : lignes d'arrondi signées, −0.0050 et +0.0050). Les mentions restantes de `bankFees`, `bank_fees` et
  `centime(n × 0.005)` sont des traces marquées périmées ou remplacées (l. 129, 195, 312, 379, 402) ou des entrées
  historiques du Change Log.
- 2026-11-18 (fiches contre le contrat `t3.md`) — MEDIUM-3 : la vue écrit la source de chaque champ (`gap` recalculé par
  `group_difference`, compte par `grouped_difference_account_id`, numéro et nom lus du compte ; « lus sur la ligne par le
  dépôt » était faux) ; MEDIUM-1 et LOW-1 : D8 bis et AC6 placent le plafond **en premier**, code attendu au test ; LOW-6 :
  `t3.md` existant, `DifferenceAccountNotUsable` inexistante, chemin `decomposition` du contrat final, `maxDifference` à
  deux décimales ; LOW-7 : sonde « un groupe n'est jamais un retrait seul » à l'AC7 ; dépendance de l'AC7 envers l'AC6 bis
  et l'AC6 de la 43-1b (HIGH-1).
- 2026-11-17 (développement, étape 0) — **Clôture de la validation** : P1 Sonnet 0/2/7/8 → P2 Opus 0/0/6/6 → P3 Sonnet
  0/0/3/6 → P4 Opus 0/0/2/6 → P5 Sonnet 0/0/0/5 → P6 ciblée (43-2, 43-3, épique) 1/1/0/4 → passe contre `t3.md`
  (quatre fiches et l'épique) 0/2/4/9 → passe ciblée 2 0/0/0/4 (rapports `kesh-gate-logs/e43/validation-43-23-p6-ciblee.md`,
  `validation-fiches-contre-t3.md`, `validation-fiches-p-ciblee-2.md`, totaux relus) ; la dernière passe ne rend rien
  au-dessus de LOW. LOW traités :
  - **L-3 (ciblée 2)** : l. 11, chemin local `/home/gcorbaz/devel/kesh-43-1a` remplacé par la branche
    `story/43-1a-reglement-groupe-modele` (ligne lue : « commit `241839568` ; code dans `/home/gcorbaz/devel/kesh-43-1a` »).
    L-1, L-2, L-4 de la même passe portent sur la 43-7 et l'épique : hors de cette fiche.
  - **L-5 (MCP)** : `git grep -n -i 'reconciliation\|ReconciliationKind' -- crates/kesh-api/src/mcp*` ne rend rien ; aucun
    outil MCP ne lit la réconciliation, rien à faire.
  - **L6 (P3, message du lettrage)** : laissé tel quel (C-e432-8) ; ligne lue :
    `tracing::error!("encaissement : lettrage de la facture impossible : {err}");` (`routes/reconciliation.rs`).
  - **L4 (P5, compte d'arrondi égal à la banque)** : l'unitaire ne le compare qu'à la créance (`refuse_if_claim_account(…,
    receivable_account_id, ClaimSubject::Designated(GapAccountRole::Rounding))`, `routes/reconciliation.rs`, (c-bis)), le
    dépôt de la 43-1a aussi (`if receivables.contains(&id)`, `grouped_settlements.rs`) : la mention reste retirée.
  - **Échelle des `Decimal`** : formatage explicite par `fixed(montant, décimales)` (`reconciliation_group.rs`), test
    `fixed_writes_the_requested_scale`.
  - **Contrat `t3.md` relu contre la table des refus** : les vingt-cinq variantes de la liste finale ont leur bras dans
    `group_refusal_to_failed_proposal` (`match` sans joker : la compilation le garantit) ; `DifferenceAccountInvalid` →
    `ACCOUNT_NOT_FOUND`, `DifferenceAccountNotPostable` → `ACCOUNT_NOT_POSTABLE`. Écart : `ReconciliationCancellation` ne
    portait pas le genre du rapprochement, exigé par la réponse de `POST …/cancel` → champ `kind` ajouté au dépôt
    (C-e432-4), à signaler à la 43-1a.
