# Story 29.8 : Une transaction bancaire réglée depuis la fiche de la facture se rattache à ce règlement

## Status

**review** — développement fait le 2026-10-14 (Opus 5.5), rebasé sur `main` `12c64c39` (29-4b comprise) ; revue de code P1 à lancer (`29-8-review-prompt-p1.md`). *Historique :* **ready-for-dev** — fiche créée le 2026-10-14 en autonomie (`bmad-create-story`, Opus 5.5), sur `fcd2ae75` (kickoff de
l'Epic 29), branche `story/29-8`, worktree `/home/gcorbaz/devel/kesh-29-8`. Story **ajoutée à l'Epic 29** sur décision
de Guy (2026-10-14) ; sortie dans la **v0.13.4**. Choix consignés au registre propre à la story,
`epic-29-choix-autonomes-29-8.md`, **C-29-8-1 à C-29-8-29** (à verser par union au registre de l'epic). 10 AC.
Validation P1 faite (Sonnet, lentilles R et F : 0/0/4/7 et 0/1/3/5), remédiée le 2026-10-14 ; validation P2 faite
(Opus : 0/0/5/5 et 0/0/5/6), remédiée le 2026-10-14 (Change Log, **signal D5 déclaré**) ; validation P3 faite
(Sonnet : R 0/0/2/7, F 0/0/0/7), remédiée le 2026-10-14 — **validation close** (Change Log : la remédiation ne touche aucune ligne
de production). PR : **`closes #670`** — dans le titre
ou le corps de la PR **seulement**, jamais dans un message de commit, même en prose (B4 de la rétrospective 28).

⚠️ **Dernière de la série de la réconciliation** : elle touche `ReconciliationProposals.svelte` (comme l'AC10 de la
29-6), `routes/reconciliation.rs` (comme les 29-2, 29-3, 29-6) et réécrit des phrases du manuel que **les 29-2, 29-5 et
29-6 écrivent** (« … puis rejetez la transaction »). Le développement **commence par un rebase** sur `origin/main` une
fois les 29-2, 29-5 et 29-6 mergées, et **relit tous les numéros de ligne** de cette fiche, relevés sur `fcd2ae75` (§ *Zones
partagées*).

## Story

**As a** utilisateur qui rapproche ses relevés (Comptable ou Admin),
**I want** rattacher une transaction bancaire au règlement que j'ai déjà enregistré depuis la fiche de la facture,
**so that** la transaction passe **rapprochée** sans qu'aucune écriture ne soit passée une seconde fois — et qu'elle
cesse d'empêcher la clôture du compte bancaire et d'apparaître parmi les transactions en attente avant la clôture d'un
exercice.

Issue : [#670](https://github.com/guycorbaz/kesh/issues/670) (P2, bug ; relevée par la validation P2 de la 29-6).
Epic : `_bmad-output/planning-artifacts/epic-29-preparer-cloture.md` § *29-8*. Cible : **v0.13.4**.

### Les prémisses de #670, établies au code (`fcd2ae75`) — exactes (C-29-8-1)

| Prémisse | Au code | Verdict |
|---|---|---|
| Un règlement par virement saisi depuis la fiche débite le compte de la banque, la transaction importée reste en attente | `settle_invoice`, choix `BankTransfer` : compte du grand livre du compte bancaire lu sous verrou (`invoice_settlements_write.rs:130-154`), écriture `D contrepartie / C créance` (`:269-290`, lignes par `settlement_journal_lines`), ligne `invoice_settlements` avec `settlement_bank_account_id` (`:294-305`). **Aucune** lecture ni écriture de `bank_transactions` | **Exacte** |
| Une transaction rejetée reste `pending` | Le rejet pose `auto_match_rejected_at` sans toucher `status` (migration `20260507100001_reconciliation_8_4.sql:1-16`) ; `find_pending_transactions_for_account` exclut les rejetées de la liste (`repositories/reconciliation.rs:288-311`) | **Exacte** |
| Elle compte comme non rapprochée | `is_reconciled(status, matched_entry_id)` = `status != 'pending' \|\| matched_entry_id.is_some()` (`repositories/bank_imports.rs:377-378`) | **Exacte** |
| Elle empêche la clôture du compte bancaire | `close_in_tx` compte `status = 'pending'`, « rejetées comprises » (`repositories/bank_accounts.rs:1018-1032`) → `ClosureRefusal::PendingTransactions` ; manuel `user-manual.tex:1689-1694` | **Exacte** |
| Elle apparaîtra au contrôle « transactions en attente » de la 29-1a | Fiche 29-1 (non mergée, numéros mouvants : citée par son code), contrôle **K2** `PENDING_BANK_TRANSACTIONS` : `status = 'pending'` sur l'exercice, `rejectedCount` à part | **Exacte** |
| Aucun geste ne la rapproche d'une écriture déjà passée | Les cinq chemins qui posent `matched_entry_id` (`routes/reconciliation.rs:2123`, `:2622`, `:3074`, `:3901`, `:4364`) lient chacun une écriture **qu'ils viennent de créer** ; le manuel le dit (« Kesh ne rapproche pas une transaction d'une écriture qui existerait déjà », `user-manual.tex:2021`) ; doc-comment `bank_accounts.rs:1379-1383` | **Exacte** |

**Ce qui existe déjà pour lier** (établi, C-29-8-2) : la colonne `bank_transactions.matched_entry_id` (FK `ON DELETE
SET NULL`, migration `20260504000001_bank_imports.sql:82-83` ; simple clé étrangère, **aucune** contrainte `UNIQUE`)
et le statut `reconciled` (`CHECK status IN ('pending',
'reconciled')`, `:84`) — **aucune migration n'est nécessaire**. Le lettrage (`letterings`) porte sur des lignes du
grand livre, pas sur `bank_transactions` : il n'entre pas dans le geste.

**L'état que produit l'acceptation d'une proposition « facture »** (`accept_one_invoice`) : une écriture de
règlement, une ligne `invoice_settlements` de type `bank_transfer` avec `settlement_bank_account_id` = le compte de la
transaction (`routes/reconciliation.rs:2069-2083`), puis `status = 'reconciled', matched_entry_id = <écriture de
règlement>`. Un règlement saisi depuis la fiche **sur le bon compte**, puis rattaché, produit **exactement le même
état** — à l'écart près de la 29-6 (AC1). C'est l'argument de la voie retenue.

### Les trois pistes de #670, et leurs conséquences établies au code (C-29-8-3)

**(a) Rattacher à une écriture existante** — la transaction passe `reconciled` en pointant l'écriture d'un règlement
déjà passé ; rien ne s'écrit au journal.

- Aucune migration ; un `UPDATE bank_transactions` et une trace d'audit.
- **Restreint aux règlements clients par virement**, l'état produit est celui d'une proposition acceptée, et les
  gardes en aval **qui lisent le lien** le traitent déjà — l'annulation du règlement depuis la fiche est refusée au
  rang 3 par le **socle de la contre-passation** (`journal_entries::reversal_blockers`, `ReversalBlocker::
  MatchedBankTransaction` → `DbError::EntryNotReversable`, `409 MATCHED_BANK_TRANSACTION` ; `cancel_settlement_in_tx`
  ne refuse lui-même que les rangs 1 à 2 bis, `invoice_settlements_write.rs:819-823`), la suppression de l'import
  (`is_reconciled`), et *Annuler le rapprochement* classe l'écriture en `InvoiceSettlement` et annule le règlement
  (`reconciliation_cancel.rs:69-95`).
- ⚠️ **La dévalidation de la facture ne lit pas ce lien** (validation P1, R2/F4) : son motif 5 (`invoices.rs:1605-1621`)
  cherche une transaction rapprochée de l'écriture de **vente** (`current.journal_entry_id`), jamais de celle du
  règlement. Une facture qui porte un règlement est refusée **avant**, au motif 1 (`invoices.rs:1545-1560`,
  `UnvalidationBlocker::Settled`, `409 INVOICE_HAS_SETTLEMENTS`) — **avant comme après** le rattachement. Le refus
  tient, mais il ne prouve rien du lien : il ne s'asserte qu'en préservation (AC3).
- Couvre la 29-2 (montant égal) et, avec l'écart toléré de l'AC1, la 29-6.

**(b) Choisir une facture à la réconciliation (#666)** — la réconciliation règle elle-même.

- Exige une seconde borne de fenêtre ou une voie d'acceptation sans borne (#548), un dialogue neuf (C-29-2-9 de la 29-2).
- **Ne couvre pas la 29-6** : l'acceptation refuse le trop-perçu (`classify_payment`, partagée avec la fiche ; 29-6,
  voie 2 écartée) — le client qui a payé 3 centimes de trop resterait bloqué.
- Ne résout pas les transactions **déjà** réglées depuis la fiche : il faudrait annuler le règlement pour le refaire.

**(c) Un statut « écartée » non compté par la clôture.**

- Migration **breaking** : la contrainte `chk_bank_transactions_status` change, un binaire 0.13.3 lirait une valeur
  inconnue de `BankTransactionStatus` → bump `min_required` **et** Cargo (P1, P2-bis).
- **Faux rattachement muet** : une transaction écartée n'a aucun lien avec une écriture ; une transaction réelle,
  jamais comptabilisée, écartée par erreur sortirait du contrôle de clôture du compte **et** du K2 — c'est
  précisément ce que ces contrôles doivent attraper (cf. *Un appariement automatique propose, il ne crée jamais* :
  le muet est le danger).

**Retenu : (a), restreint aux règlements clients par virement** — le périmètre minimal sûr. (b) reste #666, une
évolution ; (c) est écarté.

### La règle retenue (C-29-8-4, C-29-8-5, C-29-8-6)

Un règlement `s` est **rattachable** à une transaction bancaire `t` si et seulement si, tous ensemble :

1. **la transaction** est de la société, `status = 'pending'` (rejetée ou non), `amount > 0`, `currency = 'CHF'` ;
2. **le règlement** est de la même société, `settlement_type = 'bank_transfer'`, et
   `settlement_bank_account_id = t.bank_account_id` — **même compte bancaire** ;
3. **son écriture n'est rapprochée d'aucune transaction** : aucune ligne de `bank_transactions` n'a
   `matched_entry_id = s.journal_entry_id` — c'est l'invariant « une écriture, une transaction » que supposent
   `list_by_company_with_balances` (`bank_accounts.rs:1379-1383`) et `reconciliation_cancel` (rang 3) ;
4. **le montant** : soit `M` le mouvement de l'écriture de `s` sur le compte du grand livre **actuel** du compte
   bancaire (`SUM(debit) − SUM(credit)` des lignes de `journal_entry_lines` de cette écriture sur
   `bank_accounts.journal_account_id`), et `écart = t.amount − M` ; il faut **`M > 0`** et **`0 ≤ écart ≤ 0,05`**, la
   tolérance étant `AMOUNT_TOLERANCE_HUNDREDTHS` (`routes/reconciliation.rs:72`), **sans nouvelle constante**
   (C-29-8-22 : elle reste où elle est et la requête la reçoit en paramètre).

**Pourquoi `M > 0`** (C-29-8-18, validation P1, R3/F2). Sans elle, un règlement dont l'écriture n'a **aucun** mouvement
sur le compte lié actuel (`M = 0` : règlement passé sur un **ancien** compte du grand livre, le lien ayant été changé
depuis par `PATCH /bank-accounts/{id}`) vérifierait `0 ≤ t.amount ≤ 0,05` pour **tout** crédit de 1 à 5 centimes
(intérêts, arrondis de la banque) : il serait candidat à « 0.00 ». Un règlement par virement débite toujours la banque
(`settle_invoice` refuse `amount <= 0`, `invoice_settlements_write.rs:76`) : `M > 0` n'écarte aucun cas réel. La somme se fait par une table
dérivée **par écriture**, en **jointure interne** sur les lignes du compte lié actuel : une écriture sans ligne sur ce
compte ne produit **aucune** ligne (ni `COALESCE` à 0, ni `NULL` qui passerait un test mal écrit) — **c'est cette
jointure qui écarte le cas `M = 0` de l'ancien compte lié** (T18). Le `HAVING M > 0` qui la complète ne décide que d'une écriture **qui a** des lignes sur le compte lié actuel et dont la
somme est ≤ 0, ce qu'**aucun état courant** ne produit (la contrepartie d'un virement est débitée du payé,
`invoice_settlements.rs:205-211` ; le compte d'arrondi est de charge ou de produit). Un état rare existe pourtant : le
compte de créance de l'écriture de vente n'est plus celui des réglages, puis la banque lui est liée (le refus du lien
ne compare qu'aux comptes de créance **actuels**, `refuse_if_ledger_is_claim_account`,
`repositories/bank_accounts.rs:1273-1288`) : `M < 0`, et `écart = t − M > t`, donc `écart > 0,05` dès que le règlement
dépasse quelques centimes. **L'équivalence tient donc sauf au centime** (règlement de moins de 5 centimes) : M15 est
déclarée **survivante assumée**, défense en profondeur non tuée par un test (validation P3, R4 = F1 ; elle était
« équivalente » en validation P2, R2 = F2).

**Pourquoi un écart, et dans ce sens seulement** (C-29-8-5). La 29-2 rattache à montant égal (`écart = 0`). La 29-6
fait enregistrer **le montant dû** depuis la fiche quand le client a payé 1 à 5 centimes de trop — la fiche refuse le
trop-perçu (`invoice_settlements_write.rs:223-226`) — : la transaction dépasse alors la ligne de banque de 1 à
5 centimes, et l'écart reste au compte de la banque, à justifier par l'utilisateur (remboursement, ou écriture
manuelle). Un écart **négatif** (la ligne de banque dépasse la transaction) serait de l'argent aux livres que la banque
n'a pas reçu : aucun contournement ne le produit, il est exclu. L'écart est **affiché** à l'écran et écrit à la trace
d'audit (AC4, AC2) : un rattachement avec écart n'est jamais muet.

**Pourquoi pas de fenêtre de dates** (C-29-8-6). Le dialogue de règlement préremplit la date **au jour**
(`SettleInvoiceDialog.svelte:76-90`) et Guy importe un relevé **par mois** : un règlement saisi le jour où l'on
remarque le paiement peut précéder ou suivre la transaction de plusieurs semaines. Une borne refuserait le cas
nominal. La proximité **trie** les candidats et **s'affiche** (AC1, AC4) ; le choix reste humain, et rien n'est
présélectionné.

**Pourquoi le compte du grand livre actuel** : c'est celui que corrige `list_by_company_with_balances` (« on somme la
ligne de l'écriture sur le compte lié actuel », `bank_accounts.rs:1373-1375`). Un règlement passé sur un ancien compte
lié a `M = 0` et n'est pas candidat — la jointure interne sur le compte lié actuel ne lui trouve aucune ligne —,
conservateur, testé (T18), écrit au manuel (angle mort L3).

**Pourquoi un exercice clos ne refuse pas le rattachement** (C-29-8-21). Le rattachement n'écrit rien au journal : un
paiement de décembre rapproché en janvier, une fois décembre clos, doit se rattacher. Il est admis, quel que soit
l'état de l'exercice ou du verrou de période du règlement ; la conséquence est écrite au manuel : son *Annuler le
rapprochement* est ensuite refusé comme celui de tout rapprochement d'exercice clos (`409 FISCAL_YEAR_CLOSED`, ou le
refus du lettrage figé ; `user-manual.tex:2940-2952`) — testé (T19).

**Compte bancaire archivé, compte du grand livre inactif** (C-29-8-20). Comme `post_manual`
(`routes/reconciliation.rs:3645-3700`), un compte bancaire **archivé** est refusé `AppError::BankAccountNotFound`
(même réponse que `post_manual`) : aucun chemin de rapprochement n'opère sur un compte archivé. Un compte du grand
livre lié **inactif** n'est **pas** refusé, à la différence de `post_manual` : celui-ci y écrirait une ligne, le
rattachement n'écrit rien ; `M` se lit sur `journal_account_id` quel que soit son état.

### Ce que le rattachement n'est pas

- **Pas un appariement automatique** : Kesh **propose** une liste, l'utilisateur **choisit** et **confirme** ; rien
  n'est présélectionné ni rattaché d'office, et `GET /reconciliation/proposals` ne change pas (règle *Un appariement
  automatique propose, il ne crée jamais*, C-29-8-7).
- **Pas une écriture** : aucun `journal_entries::create_in_tx`, aucun règlement, aucun lettrage.
- **Pas un lot** : un geste unitaire, une transaction et un règlement ; le pattern batch ne s'applique pas
  (C-29-8-8).

## Critères d'acceptation

**AC1 — Les règlements rattachables d'une transaction se lisent (C-29-8-4 à C-29-8-6).**

`GET /api/v1/reconciliation/transactions/{id}/link-candidates` — lecture, rôle **Comptable** (même groupe que
`GET /reconciliation/proposals`). Réponse `200` :

```json
{ "bankTransactionId": 17, "candidates": [
  { "settlementId": 4, "invoiceId": 9, "invoiceNumber": "F-2026-0042", "contactName": "Dubarde SàRL",
    "settledOn": "2026-09-30", "amount": "100.00", "gap": "0.03", "daysApart": 12, "journalEntryId": 211 } ] }
```

- `amount` = `M`, `gap` = `t.amount − M`, chaînes à **deux décimales** (C-29-8-22). `M` est une somme de
  `DECIMAL(19,4)` et `t.amount` un `DECIMAL(18,2)` (`20260412000001_journal_entries.sql:42-43`,
  `20260504000001_bank_imports.sql:62`) : `gap` sort de MariaDB à l'échelle 4 (« 0.0300 »). Les **conditions** se
  calculent en décimal exact dans la requête ; la **sortie** (JSON et audit) est normalisée en Rust par
  `Decimal::rescale(2)`, patron `routes/reconciliation.rs:4274-4283` (`round_dp(2)` ne complète pas les zéros). T3
  asserte la chaîne `"0.03"`, à l'API **et** à l'audit.
- **La garantie d'échelle sur laquelle cela repose** (validation P2, F8), à écrire au doc-comment de la requête : `M`
  est la **ligne de banque**, au centime **par construction** — un règlement saisi par l'API passe `scale_within(…, 2)`
  (`routes/invoices.rs:1247`, seul point d'entrée, fiche et page des échéances, clés comprises) ; un règlement né
  d'une acceptation porte `t.amount` (`DECIMAL(18,2)`). ⚠️ La garde est **à la route, pas dans `kesh-db`**
  (`settle_invoice` ne contrôle que la positivité, `invoice_settlements_write.rs:76`). `invoice_settlements.amount`,
  lui, peut porter quatre décimales (cas `SettlesWithRounding`, `invoice_settlements_write.rs:247-250`) et n'est
  **jamais** lu. Dans tout état atteignable, la condition exacte et la sortie `rescale(2)` coïncident ; un futur
  appelant de `settle_invoice` sans la garde de route pourrait afficher « 0.00 » pour un écart de 0.004.
- `daysApart` = `|settledOn − date de paiement|`, la date de paiement étant `payment_date(t)`
  (`repositories/reconciliation.rs:113-115`, valeur, à défaut comptabilisation) — **une seule définition** : Rust
  calcule `payment_date(t)` et la **lie en paramètre** ; la requête trie sur `ABS(DATEDIFF(s.settled_on, ?))` et ne
  réécrit jamais `COALESCE(value_date, booking_date)`. La proximité **n'est pas une condition** : elle trie et
  s'affiche (C-29-8-6), et T14 la verrouille.
- La tolérance est **liée en `Decimal`** (`Decimal::new(AMOUNT_TOLERANCE_HUNDREDTHS, 2)`, jamais un `f64`) et passée
  en paramètre à la fonction de `kesh-db`, patron `find_unpaid_invoices_for_window`
  (`repositories/reconciliation.rs:234-240`, `amount_tolerance: Decimal`, construit en `routes/reconciliation.rs:637`).
  La constante reste privée à `routes/reconciliation.rs`, là où les deux handlers neufs la lisent (même module) ; la
  valeur passée à `kesh-db` se construit **une fois** pour les deux handlers (une fonction privée qu'ils appellent),
  le site existant de `:637` restant **inchangé** (§ *Ce qui doit être préservé*) — la définition est la constante,
  pas l'expression ; aucune copie de la constante dans `kesh-db`.
- Tri : `gap` croissant, puis `daysApart` croissant, puis `settlementId` croissant ; **50 au plus** — le filtre et le
  tri sont **dans la requête**, avant `LIMIT` (leçon C-29-6-3 : un filtre posé après coup laisse des lignes occuper le
  `LIMIT`).
- Transaction inexistante ou d'une autre société → `404 NOT_FOUND` (anti-énumération). Transaction rapprochée →
  `404 RECONCILIATION_TRANSACTION_NOT_PENDING` (code existant, `errors.rs:2783-2797`), rendu **après** le pré-vol ci-dessous. Le `GET` applique **le même
  pré-vol** que le `POST` (étapes 2 et 3 de l'AC2, **une seule fonction privée** que les deux handlers appellent,
  C-29-8-27 ; validation P2, R9 = F10 ; P3, R9 = F6) : compte
  bancaire archivé → `AppError::BankAccountNotFound` (comme `post_manual`, C-29-8-20) ; clos → `409
  BANK_ACCOUNT_CLOSED` ; sans compte du grand livre lié → `412 BANK_ACCOUNT_NOT_CONFIGURED` — la liste n'offre jamais
  ce que le `POST` refuserait, et ne tait pas la cause sous une liste vide. **Ordre des refus du `GET`, identique à
  celui du `POST`** : 404 inexistante ou d'une autre société → archivé → clos → 412 non lié → 404 rapprochée ; sur un
  compte clos, I1 fait que toute transaction est rapprochée, et un ordre inverse rendrait `404` au `GET` et `409` au
  `POST` pour le même objet. Transaction de débit, nulle ou hors CHF →
  `200` et `candidates: []` (la condition 1 est fausse : rien n'est rattachable).
- **Une seule définition** (C-29-8-9) : la condition 1 est une fonction pure de `kesh-db`
  (`transaction_can_be_linked(&BankTransaction) -> bool`) ; les conditions 2 à 4 sont **une** requête de
  `kesh-db/src/repositories/reconciliation_link.rs` (module neuf), dont le `POST` rejoue **le même texte** restreint à
  `s.id = ?`. Aucun second test de montant, de compte ou de lien en Rust ou en SQL.
- ⛔ Ne jamais joindre `bank_transactions` dans le `FROM` de la somme des lignes (le produit multiplierait les lignes —
  règle de `bank_accounts.rs:1389-1391`) : la condition 3 est un `NOT EXISTS`, le montant une sous-requête ou une table
  dérivée **par écriture**.

**AC2 — Rattacher : `POST /api/v1/reconciliation/transactions/{id}/link` (C-29-8-8, C-29-8-10, C-29-8-11).**

Écriture (`read-write`), rôle **Comptable**, ouverte aux clés API d'écriture comme les autres routes de
réconciliation ; la clé est nommée à l'audit. Corps `{ "settlementId": 4 }`. Réponse `200` :
`{ "bankTransactionId", "journalEntryId", "invoiceId", "settlementId" }`.

Ordre des refus — chacun **sans rien écrire** :

1. `settlementId` absent ou `≤ 0` → `400 VALIDATION_ERROR` ;
2. transaction inexistante ou d'une autre société → `404 NOT_FOUND` (`bank_account_of_transaction`,
   `routes/reconciliation.rs:4696-4710`) ;
3. compte bancaire archivé → `AppError::BankAccountNotFound` (comme `post_manual`, C-29-8-20) ; clos → `409
   BANK_ACCOUNT_CLOSED` (`ensure_bank_account_open`, `:4928-4939`) — **pré-vol** sur le pool, hors transaction ; sans
   compte du grand livre lié → `412 BANK_ACCOUNT_NOT_CONFIGURED` — **une** fonction privée, `link_preflight`, appelée par
   les deux handlers dans cet ordre (archivé, clos, non lié), avant toute lecture de statut : aucune fonction existante
   ne rend les trois refus (`ensure_bank_account_open` ne rend que le clos, `_ => Ok(())` sinon ; validation P3, R9) ;
4. **sous le verrou nommé du compte** (`with_account_lock`, comme `post_manual`, `:3798-…`), dans une transaction
   neuve, l'ordre des lectures est **imposé** (B5 de la rétrospective 28 — aucune naissance de vue avant le dernier
   verrou) :
   1. `bank_transactions` de `t`, `FOR UPDATE` → absente : `404 NOT_FOUND` ; rapprochée : `404
      RECONCILIATION_TRANSACTION_NOT_PENDING` ; condition 1 fausse : `409 RECONCILIATION_SETTLEMENT_NOT_LINKABLE` ;
   2. **deux instructions, dans cet ordre** (C-29-8-12, validation P1 F6 : l'ordre `s → je` est écrit, non laissé à
      l'optimiseur d'un `JOIN`) : (a) `SELECT journal_entry_id … FROM invoice_settlements WHERE id = ? AND company_id
      = ? FOR UPDATE` → absente ou d'une autre société : `404 NOT_FOUND` ; (b) `SELECT id FROM journal_entries WHERE id
      = ? AND company_id = ? FOR UPDATE` — même ordre `s → je` que `cancel_settlement_in_tx`
      (`invoice_settlements_write.rs:846-891`, deux instructions elles aussi), **sans** l'exercice : le rattachement
      n'écrit rien au journal et ne doit pas faire attendre une clôture d'exercice (C-29-8-21) ;
   3. `super::bank_accounts::ensure_open_in_tx(tx, company_id, t.bank_account_id)` — appel **qualifié** (validation P2,
      F11 : l'ensemble `G` de la 29-4 s'extrait des appels qualifiés ; un `use` suivi d'un appel nu sortirait
      `link_in_tx` de `G` et ferait rougir son égalité sans motif lisible) —, garde **dans la transaction**, première
      lecture ordinaire, donc **après le dernier verrou** (règle de placement de `ensure_open_in_tx`,
      `bank_accounts.rs:782-794`, fonction `:839` ; patron `unreject_once`, `routes/reconciliation.rs:4836-4843`) → `409
      BANK_ACCOUNT_CLOSED` (C-29-8-19) ;
   4. **puis seulement** la requête de l'AC1 restreinte à `s.id` (conditions 2 à 4), lecture ordinaire → absente :
      `409 RECONCILIATION_SETTLEMENT_NOT_LINKABLE` ;
5. `UPDATE bank_transactions SET status = 'reconciled', matched_entry_id = ?, auto_match_rejected_at = NULL,
   updated_at = NOW(3), version = version + 1 WHERE id = ? AND company_id = ? AND status = 'pending' AND
   matched_entry_id IS NULL` ; `rows_affected() != 1` → `409 OPTIMISTIC_LOCK_CONFLICT` (comme `post_manual`,
   `:3896-3925`) ;
6. audit `reconciliation.linked`, entité `bank_transaction`, **même transaction**, détails `bank_transaction_id`,
   `settlement_id`, `invoice_id`, `journal_entry_id`, `amount`, `settlement_amount` (= `M`), `gap`,
   `was_previously_rejected` (clés snake_case, comme `reconciliation.manual_matched`, `:3929-3940`) ; commit.

- **Code neuf** `409 RECONCILIATION_SETTLEMENT_NOT_LINKABLE`, `details.bankTransactionId` et `details.settlementId`,
  message fr-CH « Ce règlement ne peut pas être rattaché à cette transaction : la transaction n'est pas un crédit en
  CHF, ou le règlement est sur un autre compte bancaire, son montant ne correspond pas, ou il est déjà rapproché. »
  (validation P2, R8 : la condition 1 se rend sous ce code à l'étape 4.1, le message la nomme) — **un seul code** pour les conditions 1 à 4
  (C-29-8-10) : l'écran n'offre que des candidats, ce refus ne se voit que par une course ou un appel direct ;
  relire la liste. Repli d'`errors.rs` égal mot pour mot au catalogue fr-CH (garde G9 de `textes_coherents.rs`).
  *(Annoté le 2026-10-14, revue de code P1, B5 : le message nomme aussi le règlement par compte interne ou en solde —
  « n'est pas un virement sur ce compte bancaire » — et le règlement passé sur un ancien compte du grand livre ; code
  inchangé, quatre locales et repli.)*
- **Rejeu** : la tentative est enveloppée par `retry_app_on_deadlock` (comme `post_manual`, `:3779-3782`) ; route
  classée `Traced, SansEcritureAuJournal` au registre `audit_route_registry.rs` (C-29-8-11). Ce registre porte des
  **assertions chiffrées** qui rougiront, à mettre à jour (recompte au T0, valeurs de `fcd2ae75`) :
  `LIB_ROUTES.len() == 118`, `traced == 112`, `sans_ecriture == 94`, `tout.len() == 121` et `LIB_ROUTES.len() +
  TEST_ENDPOINT_ROUTES.len() == 121` (`audit_route_registry.rs:643-670`, `:981-1010`) — attendus **119, 113, 95, 122
  et 122** : le registre ne recense que les routes **mutantes** (`:324-343`), donc le seul `POST` (le `GET` n'y entre
  pas) ; chaque valeur se **recompte** depuis le registre au moment du patch, ne s'incrémente pas de confiance. Le point
  **(vi)** du doc-comment du module (`:136`, « **sept** routes `SansEcritureAuJournal` sont rejouées quand même »)
  passe à **huit** et nomme la route ; elle n'a **pas** de preuve dynamique de rejeu (aucun test d'interblocage forcé
  prévu) : le dire **dans le point (vi)**, sur le précédent de `bank_imports::delete` (« tenue par sa revue, aucune
  preuve dynamique », `:163-167`) — les « sept routes » de `:59` et `:73` sont celles qui **ont** une preuve
  dynamique, autre décompte, inchangé (validation P2, R6). **La prose chiffrée** du même fichier suit les assertions
  (validation P2, R6 = F9) : `:27` et `:31` (« 118 routes » → 119), `:32` (« plus large (121) » → 122), le message de
  `:659` (« 118 routes » → 119), le message de l'assertion `traced` (`:662-675`, qui énumère un à un les ajouts menant
  à 112, jusqu'à « la clôture et la réouverture d'un compte bancaire (28-4a, #642) » : il nomme le rattachement, comme
  celui de `:1007` — validation P3, R7), le message de `:1007` (« 93 routes de lib.rs » → 94, et la liste qu'il énumère
  nomme le rattachement). Et `bank_account_closed_inventory.rs:491` : `assert_eq!(sites.len(), 31, …)` (anti-muet)
  passe à **32** avec le `UPDATE` de `link_in_tx`, et son commentaire (`:488-490`, « W1-W30 … plus
  `write_closed_on` ») nomme le site neuf.
- `reconciliation.linked` s'insère **trié** dans `audit_labels.rs::ACTIONS` (après `reconciliation.cancelled`, avant
  `reconciliation.manual_matched`). `rbac_e2e.rs` et `idor_multi_tenant_e2e.rs` n'énumèrent aucune route de
  réconciliation : non touchés, T11 et T12 portent leur propre RBAC et IDOR.
- Le `UPDATE` de `link_in_tx` est inscrit au `REGISTRE` de `bank_account_closed_inventory.rs` au statut
  `Statut::Gardee("pré-vol (ensure_bank_account_open) et bank_accounts::ensure_open_in_tx après le verrou de
  l'écriture, étape 4.3")` — la forme exacte de `unreject_once` (`:142-148`), **parce que** la garde est lue dans la
  transaction (C-29-8-19 ; validation P1 R9/F8 : un pré-vol seul se classe `Invariant("I1", …)`, ce que la fiche
  écartait sans le dire). **Zone partagée avec la 29-4** (`29-4-cloture-compte-concurrente.md` AC5, non mergée) :
  elle bâtit le troisième inventaire `FENETRE`, dont l'ensemble `G` = les fonctions qui portent une garde non
  verrouillante en transaction, tenu **égal dans les deux sens** ; `link_in_tx` y entre. Celle des deux stories qui
  se merge **la seconde** ajoute la ligne, au statut `SansFenetre("I1 : la transaction est pending, la clôture
  refuse")` comme W30 `unreject_once`, et met à jour ses compteurs.
- **Lock-order**, à écrire au doc-comment de la fonction de `kesh-db` (C-29-8-12, C-29-8-17) : verrou nommé →
  `bank_transactions` (`t`) → `invoice_settlements` (`s`) → `journal_entries` (`je`), **trois verrous de ligne par
  clé primaire, rien d'autre** ; puis la première lecture ordinaire (étape 4.3), qui fait naître la vue. ⛔ **Aucune
  lecture verrouillante sur `matched_entry_id`** (validation P1, F1) : sur un index secondaire non unique, un
  `FOR UPDATE` qui ne trouve rien pose un **verrou d'intervalle** ; deux rattachements simultanés (deux comptes, deux
  verrous nommés) dont les écritures tombent dans le même intervalle terminal le tiennent chacun puis attendent
  l'insertion de l'autre à l'`UPDATE` : **interblocage**. Elle serait de plus redondante (ci-dessous).
- **L'invariant « une écriture, une transaction »** (condition 3 ; aucune contrainte `UNIQUE` ne le défend) se tient
  par la lecture ordinaire de l'étape 4.4, **sous le verrou de ligne du règlement** (C-29-8-17). Établi au code
  (`fcd2ae75`), et à écrire au doc-comment :
  - **le verrou nommé ne sérialise pas** : `with_account_lock` relâche le verrou (`RELEASE_LOCK`,
    `kesh-reconciliation/src/mutex.rs:126-132`) **avant** que la route commite (`conclude_locked_attempt`,
    `routes/reconciliation.rs:4433-4445`) ;
  - **c'est le verrou de ligne de `s` qui sérialise** : deux rattachements du **même** règlement visent
    nécessairement deux transactions du **même** compte (condition 2, `settlement_bank_account_id =
    t.bank_account_id`) ; le second attend à l'étape 4.2 (a) que le premier **commite** (un verrou de ligne se tient
    jusqu'au commit) ; sa vue naît **après** l'attente, à l'étape 4.3 ; la requête de l'étape 4.4 voit donc le
    `matched_entry_id` commité par le premier, le `NOT EXISTS` est faux : `409`, rien d'écrit ;
  - **les autres écrivains ne visent jamais l'écriture d'un règlement existant** : les cinq chemins qui posent
    `matched_entry_id` (`routes/reconciliation.rs:2123`, `:2622`, `:3074`, `:3901`, `:4364`) le posent sur une écriture
    **qu'ils viennent de créer**, dans leur propre transaction, invisible de toute autre avant son commit ; le seul
    autre écrivain de la colonne (`reconciliation_cancel.rs:346`) la remet à `NULL`, sous le verrou de `t` puis de
    `s` (`reconciliation_cancel.rs:287-319`) — même verrou de `s` ;
  - ⚠️ **`SELECT GET_LOCK` ne fait pas naître la vue** : `with_account_lock` n'exécute que `SELECT
    COALESCE(DATABASE(), '')` et `SELECT GET_LOCK(?, ?)` (`mutex.rs:81-90`), qui ne lisent aucune table InnoDB ; la
    fiche 28-4 l'écrit déjà (`28-4-cloture-compte-bancaire.md:961`, « ne lit aucune table InnoDB : il n'ouvre aucune
    vue »). Les lectures du pré-vol (étapes 2 et 3) se font sur le pool, en autocommit, **hors** de la transaction.
    C'est un raisonnement sur le code et la documentation d'InnoDB, **non exécuté** en validation, et **aucun test ne
    le prouve** (validation P2, R1 = F1, décision de l'orchestrateur : l'observation de l'attente est retirée de T16 et
    T17, C-29-8-25) : l'ordre d'acquisition et le moment où naît la vue sont **renvoyés à la revue de code**, qui les
    relit contre ce doc-comment et la table *Absence de cycle* ;
  - corollaire : **toute** lecture ordinaire déplacée avant l'étape 4.2 (a) rouvre le double rattachement — c'est la
    mutation M14, que T17 ne tue que si les deux requêtes se chevauchent (non garanti, AC8).
- Le développement **relit** les ordres des gestes voisins et recopie au Dev Agent Record la table *Absence de cycle*
  des Dev Notes (établie par la lentille F en validation P1), une ligne par geste, corrigée de ce qu'il trouve — le
  rattachement ne prend **jamais** la ligne `invoices` ni `bank_accounts` ni `fiscal_years`.

**AC3 — Ce que le rattachement débloque, et ce qu'il laisse tel quel (C-29-8-13).**

Établi par test d'intégration (`crates/kesh-api/tests/reconciliation_link_e2e.rs`, binaire neuf) :

- après rattachement, `POST /bank-accounts/{id}/close` **réussit** sur un compte dont c'était la seule transaction en
  attente — **le défaut de #670** ; avant, il rend `412 BANK_ACCOUNT_HAS_PENDING_TRANSACTIONS`. Asserté par **T1**, sur
  un compte bancaire **non principal** (un compte principal ne se clôt pas, et ce refus précède le décompte des
  transactions en attente, `bank_accounts.rs:1016-1019`) — et **non** par T15 : une fois le compte clos, toute
  annulation qui suit rend `409 BANK_ACCOUNT_CLOSED` avant ses propres motifs (`reconciliation_cancel.rs:325`,
  `invoice_settlements_write.rs:896`), ce qui rendrait les assertions de T15 inatteignables (validation P2, R4 = F4,
  C-29-8-26) ;
- **aucune écriture** n'est créée : nombre de `journal_entries` et de `invoice_settlements` de la société inchangés ;
  le reste dû et `paid_at` de la facture inchangés ;
- `GET /reconciliation/transactions/{id}` rend `kind = "invoice_settlement"`, `invoiceId`, `cancellable = true` ;
- *Annuler le rapprochement* (`POST …/cancel`) **annule aussi le règlement** saisi depuis la fiche (contre-passation
  datée du jour, facture de nouveau à régler, transaction de nouveau `pending`) — comportement d'une proposition
  acceptée, **inchangé et voulu** ; écrit au manuel (AC6) et angle mort L1 ;
- l'annulation du règlement depuis la fiche est refusée au rang 3 **par le socle de la contre-passation** :
  `DbError::EntryNotReversable { blocker: ReversalBlocker::MatchedBankTransaction }`, réponse `409`, `code:
  "MATCHED_BANK_TRANSACTION"` (`errors.rs:110` de `kesh-db`, `entry_document_refusal_response` de `kesh-api`) — **et
  non** `SettlementNotCancellable`, que `cancel_settlement_in_tx` ne lève qu'aux rangs 1 à 2 bis. ⛔ L'assertion n'est
  une preuve que si elle **échoue sans le rattachement** : le décor porte un **témoin**, un second règlement de la même
  facture (acompte), **par virement sur le même compte bancaire** — il ne diffère du règlement rattaché que par le
  lien —, jamais rattaché, dont l'annulation depuis la fiche **réussit** dans le même test (C-29-8-13) ; l'ordre des
  assertions est écrit à T15 (le refus d'abord, `…/cancel` ensuite : il supprime le règlement,
  `invoice_settlements_write.rs:946`) ;
- la dévalidation de la facture est refusée au motif 1, `409 INVOICE_HAS_SETTLEMENTS`
  (`UnvalidationBlocker::Settled`, `invoices.rs:1545-1560`) — **avant comme après** le rattachement : elle ne lit pas
  le lien (son motif 5 vise l'écriture de vente). Assertée **en préservation seulement**, avant et après, avec ce
  commentaire au test : elle ne prouve rien du rattachement et aucune mutation ne la vise ;
- si la 29-1 est mergée avant le développement : la transaction rattachée **sort** du contrôle K2
  (`PENDING_BANK_TRANSACTIONS`) de la 29-1a — le troisième effet de #670 ; sinon, la ligne s'ajoute à T15 par celle des
  deux stories qui se merge la seconde.

**AC4 — Le geste à l'écran (C-29-8-7, C-29-8-14).**

`ReconciliationProposals.svelte` : un troisième bouton de ligne, **Rattacher à un règlement**, à côté d'*Affecter
manuellement* et d'*Éclater*, sur les lignes dont `transaction.amount > 0` et `transaction.currency === 'CHF'`
(miroir de la condition 1 ; une ligne de débit ne le porte pas). Il ouvre `LinkSettlementModal.svelte` (neuf) :

- titre, rappel de la transaction (date, montant, contrepartie) ;
- chargement, puis un tableau des candidats (choix par bouton radio) : facture, client, date du règlement, montant,
  écart, jours d'écart ; **aucune ligne présélectionnée** ; *Rattacher* grisé tant qu'aucun choix n'est fait ;
- si le candidat choisi porte un écart non nul, un avertissement **visible avant confirmation** : « La transaction
  dépasse ce règlement de 0.03 CHF. L'écart reste au compte de la banque : justifiez-le (remboursement au client, ou
  écriture manuelle). » ;
- une phrase permanente : « Le rattachement ne passe aucune écriture : il relie la transaction au règlement déjà
  enregistré. » ;
- liste vide : « Aucun règlement par virement sur ce compte bancaire, au montant de cette transaction, n'attend de
  transaction. Enregistrez d'abord le règlement depuis la fiche de la facture, sur ce compte bancaire. » ;
- succès → fermeture, `load()`, compteur « 1 opération(s) réussie(s). » (même bilan que l'affectation manuelle,
  `clearBatchReport` d'abord) ; erreur → message lisible (`errorMessageOf`), modal ouverte ; sur `409
  RECONCILIATION_SETTLEMENT_NOT_LINKABLE` en particulier, la modal **recharge** la liste (`getLinkCandidates`) et
  remet le choix à vide — le message dit « relire la liste », la modal le fait (C-29-8-24) ;
- échec du chargement des candidats → message lisible (`errorMessageOf`), *Rattacher* grisé, la modal reste fermable ;
- API : `getLinkCandidates(txId)` et `linkTransaction(txId, settlementId)` dans `reconciliation.api.ts`, types dans
  `reconciliation.types.ts`.
- Recette visuelle : captures **clair et sombre** de la modal (avec un candidat à écart), dans `KESH_E2E_CAPTURES_DIR`,
  hors dépôt.

**AC5 — Textes : i18n quatre locales et audit.**

Clés `reconciliation-link-*` (bouton, titre, colonnes, phrase permanente, avertissement d'écart, liste vide,
*Rattacher*, *Annuler*), `reconciliation-errors-settlement-not-linkable`, `audit-log-action-reconciliation-linked`
(fr-CH « Transaction rattachée à un règlement ») dans les **quatre** `messages.ftl` ; `reconciliation.linked` ajouté à
`audit_labels.rs::ACTIONS`. `npm run lint-i18n-ownership` vert ; replis Svelte égaux au catalogue fr-CH.

**AC6 — Manuel FR (PDF compris), API externe, CHANGELOG (C-29-8-15).**

`docs/manual/fr/user-manual.tex` (relire chaque ligne après rebase ; numéros relevés sur `fcd2ae75`). L'inventaire
part du **symptôme** et non d'une liste (validation P1, R1 ; motif élargi en validation P2, R5 et F5) : `grep -nE
"depuis la fiche|ne la relie pas|ne relie pas|tout seul|reglement-client|rejet|670|crée l'écriture|date de
valeur|issue d'un rapprochement|rien ne la relie" docs/manual/fr/user-manual.tex`, après rebase, chaque site trié au
Dev Agent Record (réécrit, ou inchangé avec sa raison).

⚠️ **Les conseils de rejet se cherchent par le symptôme, au rebase — la fiche ne recopie plus les phrases des fiches
voisines** (décision de l'orchestrateur, validation P3, R1 ; C-29-8-29), car elles bougent (la 29-2 a déjà supprimé la
phrase de `:1305` que cette fiche prescrivait de compléter). À l'étape T0, **après** le rebase sur les 29-2, 29-5 et
29-6 mergées : `grep -nE 'rejet[a-z]*( ensuite)?( la transaction|-la)' docs/manual/fr/user-manual.tex CHANGELOG.md`,
**et toute forme voisine trouvée** (« en rejetant ensuite la transaction », « rejetez-la », « rejetée, elle reste en
attente »). Chaque occurrence qui conseille de rejeter une transaction **réglée depuis la fiche de la facture** est
réécrite en « rattachez » (renvoi `\S\ref{sec:rattacher-reglement}`) ; les autres occurrences (description générale du
rejet, rejet d'une transaction sans règlement) sont **triées et écrites** au Dev Agent Record, avec leur raison. Sites
connus au jour de la fiche, **non prescrits ici** : `:2029`, `:2031`, `:1305` et `CHANGELOG.md:15` (29-2),
`29-5-departage-exigibilite.md:326-327` (29-5), la FAQ `:2911`, `:2050`, `:2057` et l'entrée de CHANGELOG (29-6).
Le tableau ci-dessous ne garde que les **sites propres à la 29-8** ; il renvoie à ce grep pour le reste. Le contrôle de
pré-release de la v0.13.4 (fichier d'epic) doit rendre **0 conseil de rejet** après la 29-8.

Sites propres à la 29-8, triés dès la fiche (sur `fcd2ae75`) :

| Site | Ce qu'il dit | À faire |
|---|---|---|
| `:2911` (FAQ « Que faire ensuite », puce *paiement de client*) | un conseil de **rejet** (sur `fcd2ae75` : « Kesh ne la relie pas » ; réécrit par la 29-6) | **faux** après la 29-8 : le conseil devient « … puis **rattachez** la transaction à ce règlement (\S\ref{sec:rattacher-reglement}) : elle passe rapprochée, sans écriture. » ; l'incise « rejetée, elle reste en attente … » est **retirée** ; la consigne « N'utilisez ni \emph{Affecter manuellement} ni \emph{Éclater} » **reste** — texte exact relevé après rebase (grep ci-dessus) |
| `:2050` (liste des lignes, « Aucune correspondance ») | l'énumération des gestes de la ligne (rejeter ; *Affecter manuellement* ; *Éclater*), telle que la 29-6 l'a réécrite | **incomplète** après la 29-8 (elle omet le troisième bouton) : + « ; une ligne de crédit en CHF se rattache aussi au règlement déjà saisi depuis la fiche (\emph{Rattacher à un règlement}, \S\ref{sec:rattacher-reglement}) » |
| `:259-267` (tuile d'écart du tableau de bord, *dates de valeur*) | « Kesh date les écritures de rapprochement à la date de valeur » ; « seule une écriture **issue d'un rapprochement** est ainsi corrigée : une écriture saisie à la main … — rien ne la relie au relevé » | **incomplet** après la 29-8 (validation P2, F5) : une transaction rattachée pointe l'écriture du règlement, datée du **jour du règlement** (`settled_on`), non de la valeur, et la correction s'y applique (`bank_accounts.rs:1414-1435`) : + « — une transaction rattachée à un règlement saisi depuis la fiche (\S\ref{sec:rattacher-reglement}) garde la date du règlement, et la correction s'y applique de même » ; la puce : « issue d'un rapprochement **ou d'un rattachement** » ; « rien ne la relie au relevé » reste vrai pour une écriture saisie au journal (L3) |
| `:3053` (glossaire, *Réconciliation bancaire*) | « Kesh crée l'écriture, en réglant une facture client ou en imputant un compte de contrepartie. » | **incomplet** après la 29-8 (validation P2, R5 ; même affirmation que `:2021`) : + « ; une transaction dont le règlement a déjà été saisi depuis la fiche de la facture se rattache à ce règlement, sans écriture. » |
| `:1198` (`sec:reglement-client`, le point d'entrée du flux) | « Le rapprochement bancaire enregistre, lui, ses règlements tout seul. » | **incomplet** : + « Si le virement est déjà importé, rattachez ensuite la transaction au règlement (\S\ref{sec:rattacher-reglement}) — sans quoi elle reste en attente et empêche la clôture du compte bancaire. » |
| `:1305` (« Ce que réclame un rappel ») | **la 29-2 a réécrit cette phrase** (elle supprime « enregistrez alors le règlement depuis la fiche de la facture ») | **par le grep** ci-dessus : n'ajouter « , puis rattachez-y la transaction (\S\ref{sec:rattacher-reglement}) » que si un renvoi au règlement depuis la fiche **subsiste** après rebase |
| `:1308` (avertissement des frais de rappel, #401) | le paiement « reste dû **et** frais » est refusé comme trop-perçu ; « seul le montant de la QR-facture s'enregistre » | + l'angle mort **L6** : sa transaction dépasse le règlement de plus de 5 centimes, elle **ne se rattache pas** et reste en attente (issues #671, #401) |
| `:2029` (algorithme de matching : paiement de plus de 30 jours ; trop-perçu de quelques centimes) | des conseils de rejet (29-2, 29-6) | **par le grep** ci-dessus : « rattachez » ; l'incise « rejetée, elle reste en attente … » **retirée** ; « et traitez l'écart » (29-6) reste |
| `:2076` (fin de *Réconciliation manuelle*) | « Pour un paiement de client que la réconciliation ne propose pas, enregistrez le règlement depuis la fiche » | + « , puis rattachez-y la transaction » |
| `:1840` (suppression d'import) | « une transaction **rejetée** … ne compte pas » | **inchangé** : reste vrai ; une transaction rattachée compte comme rapprochée, ce que dit déjà la phrase suivante |
| `:1431` (refus de dévalidation) | « annulez d'abord le règlement, un paiement rapproché ne devant pas pointer dans le vide » | **inchangé** : vrai ; un règlement rattaché ne s'annule qu'après *Annuler le rapprochement*, que dit déjà `:649` |
| `:2940-2952` (exercice clôturé) | un rapprochement dans un exercice clôturé ne s'annule pas | **inchangé** ; la sous-section neuve y renvoie (C-29-8-21) |
| autres sites de `reglement-client` (`:649`, `:966`, `:1345-1347`, `:1528`, `:1644`, `:2197`, `:2305`, `:2332`, `:2521`, `:2944`) | renvois au règlement, sans rapport à la transaction bancaire | **inchangés**, motif écrit d'une ligne chacun au Dev Agent Record |

- **`:2021`** (intro de la réconciliation) : « Kesh ne rapproche pas une transaction d'une écriture qui existerait
  déjà » devient : Kesh **crée** l'écriture …, **sauf** dans un cas : une transaction dont le règlement a déjà été
  enregistré depuis la fiche de la facture se **rattache** à ce règlement, sans écriture (renvoi à la sous-section
  neuve).
- **Sous-section neuve** « Rattacher une transaction à un règlement déjà enregistré » (après *Réconciliation
  manuelle*, `\label{sec:rattacher-reglement}`) : quand (règlement saisi depuis la fiche ou la page des échéances,
  par virement, **sur ce compte bancaire**) ; le geste (bouton, choix, confirmation) ; les conditions (même compte,
  montant égal ou dépassé de 5 centimes au plus, règlement pas encore rapproché) ; l'écart affiché et à justifier ;
  aucune écriture ; une transaction rejetée : *Annuler le rejet* d'abord ; un règlement d'exercice clôturé se
  rattache, mais son rapprochement ne s'annule plus ensuite (\S\ref{sec:annuler-rapprochement}, C-29-8-21) ; ce qu'on
  ne peut pas rattacher (L2 à L7, dont le paiement qui inclut les frais de rappel et le règlement saisi par
  « compte interne » sur le compte du grand livre de la banque).
- **Les sites du tableau ci-dessus** (`:2911`, `:2050`, `:259-267`, `:3053`, `:1198`, `:1308`, `:2076`) : chacun
  réécrit comme le dit le tableau ; `:2029` et `:1305` par le grep.
- **Les conseils de rejet** (« … puis rejetez la transaction » et formes voisines) écrits par les 29-2, 29-5 et 29-6 :
  inventaire **par le symptôme** au rebase (grep du début de l'AC6), non recopié ici. L'incise « rejetée, elle reste en
  attente … et empêche la clôture du compte bancaire » est retirée là où elle ne décrit plus que le contournement ; la
  description **générale** du rejet (`:2057`, « une transaction rejetée compte comme non rapprochée et empêche la
  clôture de son compte bancaire ») **reste** : elle est vraie de tout rejet, et l'indication de l'écran de la 29-6
  (« … décochez-la pour accepter les autres. ») ne nomme aucun geste.
- **`:1689-1694`** (clôture du compte) : « … tant qu'elle n'est pas rapprochée manuellement ou que son import n'est
  pas supprimé » + « ou **rattachée** au règlement saisi depuis la fiche de la facture
  (\S\ref{sec:rattacher-reglement}) ».
- **`:2076`** (fin de *Réconciliation manuelle*) : voir le tableau.
- **§ *Annuler un rapprochement*** (`sec:annuler-rapprochement`, `:2124-…`) : une transaction **rattachée** s'annule comme une proposition
  acceptée — **le règlement est retiré** aussi ; pour défaire seulement le rattachement, il n'y a pas d'autre geste
  (L1).
- PDF régénéré (`make fr` dans `docs/manual/`, sous le verrou de gate) et contrôlé **aplati et normalisé**, comme la
  29-6 (ligatures et apostrophe : `pdftotext docs/manual/fr/user-manual.pdf - | tr '\n' ' ' | tr -s ' ' | sed
  "s/ﬀ/ff/g;s/ﬁ/fi/g;s/ﬂ/fl/g;s/ﬃ/ffi/g;s/’/'/g"`), comptes par `grep -oF` (chaîne fixe : en motif, le point de
  « tout seul. » prend n'importe quel caractère et compte « tout seul~: » de `:1626`) :
  - **positifs, par site** (validation P3, R3) — chaque phrase neuve a **sa** portion, relevée au Dev Agent Record avec
    son compte ; une chaîne partagée par plusieurs sites ne prouve que leur somme, et se compte comme telle :
    « rattachez la transaction à ce règlement » **≥ 2** (FAQ `:2911` et `:2029` — plus tout site que le grep du début
    de l'AC6 réécrit avec la même formule) ; « rattachez ensuite la transaction au règlement » **≥ 1** (`:1198`) ;
    « Rattacher une transaction à un règlement » ≥ 1 (titre de la sous-section) ; « Rattacher à un règlement » ≥ 1
    (`:2050`) ; « garde la date du règlement » ≥ 1 (`:259-267`) ; « se rattache à ce règlement, sans écriture » ≥ 1
    (`:3053` ; l'introduction `:2021` emploie « sauf » et sa propre portion, fixée à la rédaction) ; « ou rattachée au
    règlement » ≥ 1 (`:1689-1694`) ; « rattachez-y la transaction » ≥ 1 (`:2076` ; `:1305` seulement si le grep le
    réécrit) ; « ne se rattache pas » ≥ 1 (`:1308`, L6) ;
  - **négatifs**, qui ne valent qu'adossés aux positifs (un zéro peut venir d'une césure) — rendent **0** : « ne la
    relie pas », « ne relie pas la transaction », « rejetez-la », « existerait déjà », « date les écritures de
    rapprochement à la date de valeur. » — la phrase **se poursuit** par l'incise après un tiret, sans point entre
    « valeur » et l'incise (`:259-267` : la ponctuation est écrite ici, validation P3, R2) ; et aucune occurrence de
    « puis rejetez » liée à un règlement saisi depuis la fiche (tri au Dev Agent Record). ⚠️ « tout seul. » n'est **pas**
    un négatif : `:1198` garde sa phrase « Le rapprochement bancaire enregistre, lui, ses règlements tout seul. », que
    l'implémentation fidèle conserve (validation P3, R2) — il reste au seul relevé « ce qui doit encore y être » ;
  - **relevé sur `fcd2ae75`** (remédiation P2, aplati et normalisé, `grep -oF`) : « ne la relie pas » 1, « tout
    seul. » 1, « Kesh crée l'écriture, en réglant » 1, « date les écritures de rapprochement » 1, « rien ne la relie
    au relevé » 1, « existerait déjà » 1, « ne relie pas la transaction » 0 et « rejetez-la » 0 — ces deux dernières
    naissent au rebase sur la 29-6, et c'est là que leur contrôle négatif cesse d'être vide.

`docs/api-external.md` : les deux routes (paragraphes à côté de `…/unreject`, `:594`), le code
`RECONCILIATION_SETTLEMENT_NOT_LINKABLE` au tableau des erreurs (`:719` voisin), la route à la note ⁹ du § 7 (refus
`BANK_ACCOUNT_CLOSED`, `:240`), la phrase de `kind` (`:571`) : `invoice_settlement` désigne aussi une transaction
rattachée, et le paragraphe **« Interblocages : les écritures au journal sont rejouées »** (`:730`, validation P2,
R7) : la route `POST /reconciliation/transactions/{id}/link` y entre, à côté des exercices, comme route qui n'écrit
pas au journal mais est rejouée — sans quoi elle tomberait sous « Une autre route peut, rarement, rendre `500` »
(`:732`), faux pour elle. `CHANGELOG.md`, section 0.13.4 (créée par les stories précédentes) : *Ajouté* — le
rattachement (#670) ; *Modifié* — le contournement des 29-2, 29-5 et 29-6 n'est plus « rejeter » mais
« rattacher » (chaque entrée de la section qui conseille de rejeter, trouvée par le grep de l'AC6, se réécrit) ;
l'entrée de la 29-6 **dans la même section** (« … rejetez ensuite la transaction … Rejetée, la
transaction reste en attente et empêche la clôture du compte bancaire ([#670]) », sa fiche `:355-362`) se **réécrit**
en « rattachez » plutôt que d'être contredite par l'entrée suivante de la même version ; *Limites connues* — les
angles morts L1 à L7, avec leurs issues (`refs #671`, `refs #672`, #401 ; jamais `closes`).

**AC7 — L'inventaire clos des textes qui supposent « une écriture créée par le rapprochement » (C-29-8-16).**

Relevé par `grep -rnE "cinq chemins|viennent de créer|existerait déjà|une transaction, une écriture|\bcinq\b.*(chemin|site)|écriture \*?\*?(neuve|NEUVE)|crée l'écriture|acceptation d'une proposition|datées? à la date de"`
sur `crates`, `docs/manual/fr/*.tex`, `docs/api-external.md` (`fcd2ae75`), motif **élargi** en validation P1 (R7) puis
en validation P2 (R5, F5 : les trois derniers termes) — plus `grep -rnB1 "^ *///.*NEUVE"` pour la coupure de ligne de
`invoices.rs:4348-4349` que le motif mono-ligne manque. **Douze** sites (neuf du relevé initial, plus trois ajoutés en validation P3, R6), chacun réécrit (le sixième chemin, ses
conditions) ou déclaré inchangé avec sa raison au Dev Agent Record :

| Site | Ce qu'il dit | À faire |
|---|---|---|
| `crates/kesh-db/src/repositories/bank_accounts.rs:1379-1383` | « Les cinq chemins … lient chacun une écriture qu'ils viennent de créer » | six chemins ; le rattachement lie une écriture **existante**, l'invariant « une écriture, une transaction » est tenu par la condition 3 |
| `crates/kesh-db/src/repositories/reconciliation_cancel.rs:11-18` | « Des cinq chemins … un seul crée un règlement client » | deux chemins rendent `InvoiceSettlement` : l'acceptation « facture » et le rattachement |
| `crates/kesh-db/src/repositories/reconciliation_cancel.rs:48` (doc-comment de la variante `InvoiceSettlement`) | « Un **règlement client** (acceptation d'une proposition « facture ») » | + « , ou rattachement d'une transaction à un règlement saisi depuis la fiche » (validation P2, R5) |
| `crates/kesh-db/src/repositories/bank_accounts.rs:1367-1370` | « les écritures de rapprochement sont datées à la date de **valeur** » | + « — sauf l'écriture d'un règlement rattaché, datée du règlement ; la correction lit la date de l'écriture, elle s'y applique de même » (validation P2, F5) |
| `crates/kesh-db/src/repositories/bank_imports.rs:357` | « les cinq chemins qui rapprochent posent les deux colonnes ensemble » | six ; le rattachement aussi |
| `crates/kesh-db/tests/reconciliation_cancel.rs:3` | « Les cinq chemins réels » | ajouter le rattachement ou dire pourquoi le test n'en a pas besoin (couvert par AC3) |
| `docs/manual/fr/user-manual.tex:2021` | « Kesh ne rapproche pas … existerait déjà » | AC6 |
| `crates/kesh-db/src/repositories/invoices.rs:4348-4349` (doc-comment de test, `insert_stub_bank_transaction`) | « le rapprochement pointe une écriture **NEUVE**, jamais celle de vente » | « jamais celle de vente » reste vrai, « NEUVE » devient faux : « … une écriture neuve ou, par le rattachement, celle d'un **règlement**, jamais celle de vente » |
| `crates/kesh-db/src/repositories/invoices.rs:4805-4806` (doc-comment de test, motif 7) | « les cinq sites de `reconciliation.rs` rapprochent une écriture **neuve** » | idem : six chemins, dont le rattachement à l'écriture d'un règlement ; aucun ne rapproche l'écriture de vente |
| `crates/kesh-api/tests/reconciliation_e2e.rs:4739` (commentaire de test) | « aucun chemin réel ne laisse le marqueur sur une transaction rapprochée — **quatre** chemins le remettent à zéro » | cinq : le rattachement remet aussi `auto_match_rejected_at = NULL` dans son `UPDATE` (validation P3, R6) |
| `crates/kesh-db/tests/bank_account_statement_gap.rs:7` | « les écritures de rapprochement par date de [valeur] » | jumeau de `bank_accounts.rs:1367-1370` : même réserve — l'écriture d'un règlement rattaché est datée du règlement (validation P3, R6) |
| `docs/manual/fr/user-manual.tex:2129` | « Quand le rapprochement a réglé une facture client, le même bouton apparaît aussi sur la fiche » | reste vrai du rattachement mais ne le dit pas : + « ou qu'une transaction a été rattachée à son règlement » (renvoi à la puce *Annuler un rapprochement*) (validation P3, R6) |

Les autres lignes que rend le motif élargi sont **hors sujet**, triées ici : `routes/reconciliation.rs:2098` et
`tests/reconciliation_e2e.rs:3415` (« cinq sites qui lisent `paid_at IS NULL` », autre inventaire) ;
`bank_account_closed_inventory.rs:15`, `:523` et les assertions « aucune écriture neuve » des tests de
`invoice_settlement.rs`, `payment_batches_repository.rs`, `supplier_invoices_repository.rs`, `invoice_write_off.rs`
(écriture **au journal**, non rapprochement) ; `user-manual.tex:1417` (revalidation d'une facture) ; `audit_label_registry.rs:210` (une action d'audit « NEUVE »).
Rendus par les termes ajoutés en validation P2 (relevé sur `fcd2ae75`) : `reconciliation_e2e.rs:8317` (acceptation
d'une proposition ventilée) ; `user-manual.tex:403` (saisie manuelle, indicateur *postable*), `:953` (écriture de
validation d'une facture), `:1809` (refus à l'acceptation d'une proposition, compte non lié), `:2932`
(contre-passation) — hors sujet ; `user-manual.tex:3053` (glossaire) : traité à l'AC6.

Après chaque patch, **greper la valeur** (`\bcinq\b`, `\b5\b` près de « chemin ») et non la phrase (§ *Propagation*).

**AC8 — Tests rouges d'abord, puis mutations (C-29-8-9).**

Intégration (`reconciliation_link_e2e.rs`, `#[sqlx::test(migrations = "../kesh-db/test-schema")]`), décor par l'API
(règlement par `POST /invoices/{id}/settlements`, transaction par import CAMT de fixture synthétique) :

| # | Cas | Attendu |
|---|---|---|
| T1 | règlement fiche = montant, même compte, compte bancaire **non principal** dont c'est la seule transaction en attente | avant : `POST /bank-accounts/{id}/close` → `412 BANK_ACCOUNT_HAS_PENDING_TRANSACTIONS` ; 1 candidat `gap 0.00` ; `POST` 200 ; `reconciled`, `matched_entry_id` = écriture du règlement ; aucun `journal_entries` neuf ; audit `reconciliation.linked` aux détails de l'AC2 ; après : la clôture **réussit** — le défaut de #670 (déplacé de T15, C-29-8-26) ; **états préservés** (AC3, validation P3, R5) : nombre de `invoice_settlements` de la société, reste dû et `paid_at` de la facture inchangés (lecture avant/après), et `GET /reconciliation/transactions/{id}` rend `kind = "invoice_settlement"`, `invoiceId`, `cancellable = true` |
| T2 | transaction rejetée | candidate ; `POST` 200 ; `auto_match_rejected_at` à `NULL` ; `was_previously_rejected: true` |
| T3 | cas 29-6 : transaction 100.03, règlement 100.00 | candidat `gap 0.03` ; `POST` 200 ; audit `gap: "0.03"` |
| T4 | écart 0.05 / 0.06 | candidat / absent ; `POST` 409 `RECONCILIATION_SETTLEMENT_NOT_LINKABLE` pour 0.06 |
| T5 | règlement > transaction de 0.01 | absent ; 409 |
| T6 | règlement sur un autre compte bancaire **lié au même compte du grand livre** que celui de `t` (partage admis au sol, `bank_accounts.rs:1438-1491` ; une ligne de montage) — sans ce partage, la jointure sur le compte lié l'écarterait d'elle-même et M1 survivrait (validation P2, R2 = F3) | absent ; 409 |
| T7 | règlement `internal_account` | absent ; 409 (test de comportement : M2 est équivalente) |
| T8 | écriture déjà rapprochée (règlement né d'une acceptation ; ou rattaché à une autre transaction) | absent ; 409 ; la seconde transaction reste `pending` |
| T9 | transaction de débit, nulle (`0.00`), hors CHF | `candidates: []` ; `POST` 409 |
| T10 | transaction déjà rapprochée | `GET` et `POST` 404 `RECONCILIATION_TRANSACTION_NOT_PENDING` |
| T11 | transaction ou règlement d'une autre société | 404, rien d'écrit |
| T12 | rôles : Consultation 403 ; clé lecture seule refusée, clé `read-write` admise et nommée à l'audit | — |
| T13 | compte clos : (a) clôture posée par SQL **avant** le `GET` et le `POST` → 409 `BANK_ACCOUNT_CLOSED` du pré-vol, aux deux (C-29-8-27) ; (b) **course** : une connexion tient le verrou nommé du compte (`GET_LOCK` du même nom), le `POST` est vu bloqué dessus (modèle `wait_blocked_on_account_lock_274`, `reconciliation_e2e.rs:6869-6875`), la clôture est posée par SQL, le verrou relâché → 409 `BANK_ACCOUNT_CLOSED` de `ensure_open_in_tx` (étape 4.3), transaction `pending`. ⚠️ **État forgé** : une clôture réelle refuserait (`412`, I1 : `close_in_tx` compte la `pending` sous le verrou du compte) ; le test exerce la garde, non une course atteignable — elle ne sert en réel que dans la fenêtre L1 de #651 (validation P3, F4) | — |
| T14 | tri, trois candidats dont les **couples** sont écrits : A écart 0.00, réglé **40 jours avant** la date de paiement ; B écart 0.00, réglé **3 jours après** ; C écart 0.02, réglé **1 jour** après | ordre B, A, C ; `daysApart` 3, 40, 1 (jamais négatif) |
| T15 | AC3, sur un compte bancaire que le test **ne clôt pas** (la clôture est assertée par T1, C-29-8-26 ; aucune exigence « non principal » ici, validation P3, F5) ; facture portant deux règlements **par virement sur ce même compte** : `s` (à rattacher) et le témoin `w` (acompte, jamais rattaché). Assertions **dans cet ordre** (validation P2, R4 = F4) : (1) dévalidation `409 INVOICE_HAS_SETTLEMENTS` (préservation, avant) ; (2) rattachement de `t` à `s` ; (3) annulation de `s` depuis la fiche → `409 MATCHED_BANK_TRANSACTION`, **puis** annulation de `w` depuis la fiche → succès ; (4) dévalidation toujours `409 INVOICE_HAS_SETTLEMENTS` (préservation, après) ; (5) K2 si la 29-1 est mergée ; (6) **en dernier** — il supprime `s` (`invoice_settlements_write.rs:946`) — `POST …/cancel` : contre-passation, facture de nouveau à régler, `t` de nouveau `pending` | — |
| T16 | **course réelle** entre l'annulation de `s` depuis la fiche et le rattachement de `t` à `s`, lancés ensemble (`tokio::join!`) | **exactement l'un** réussit, et l'état final est l'un des deux états cohérents : soit annulation `200`, rattachement `404`, `t` `pending` et `matched_entry_id` nul ; soit rattachement `200`, annulation `409 MATCHED_BANK_TRANSACTION`, `s` présent et `t` rapprochée de son écriture. ⚠️ Ce que T16 **ne prouve pas** (C-29-8-25) : l'attente du perdant ni l'ordre d'acquisition — rien n'observe l'attente ; si les deux requêtes ne se chevauchent pas, la seconde voit le résultat de la première par simple succession, et le test passe aussi |
| T17 | **course de double rattachement** (validation P1, F3 ; observation retirée en validation P2, R1 = F1) : deux transactions `t1`, `t2` du même compte, un règlement `s` ; `POST /…/t1/link {s}` et `POST /…/t2/link {s}` lancés ensemble (`tokio::join!`) | **exactement un** `200`, l'autre `409 RECONCILIATION_SETTLEMENT_NOT_LINKABLE` ; en base, **une seule** transaction pointe l'écriture de `s`, l'autre reste `pending`, `matched_entry_id` nul. ⚠️ Ce que T17 **ne prouve pas** (C-29-8-25) : l'ordre d'acquisition (`t` → `s` → `je`, puis la vue) ni que le perdant a attendu le verrou de `s` — les deux requêtes passent par le même verrou nommé, relâché avant le commit, et la fenêtre de chevauchement est étroite ; sans chevauchement, le second voit le lien par simple succession. **Renvoyé à la revue de code** (doc-comment de `link_in_tx`, table *Absence de cycle*) |
| T18 | **ancien compte lié** (R3/F2) : règlement `s` saisi, puis `PATCH /bank-accounts/{id}` change le compte du grand livre lié ; (a) transaction de 0.03 ; (b) transaction égale au montant de `s` | (a) et (b) : `s` absent des candidats ; `POST` 409 |
| T19 | **exercice clos** (C-29-8-21) : règlement dans un exercice clôturé (statut posé par SQL), transaction de l'exercice suivant | `POST` 200 ; puis `…/cancel` refusé `409 FISCAL_YEAR_CLOSED` (code relu au T0) |
| T20 | **compte bancaire archivé** (C-29-8-20, archivage posé par SQL) | `GET` et `POST` : la réponse de `post_manual` (`AppError::BankAccountNotFound`) ; rien d'écrit |

Unitaires `kesh-db` : `transaction_can_be_linked` (crédit, nul, débit, devise ; `-0.00` traité comme nul).
Vitest : `LinkSettlementModal.test.ts` (chargement, liste vide, échec du chargement affiché, aucune présélection,
*Rattacher* grisé puis actif, avertissement d'écart présent/absent, succès → `onSuccess`, erreur affichée, `409
RECONCILIATION_SETTLEMENT_NOT_LINKABLE` → liste rechargée et choix remis à vide), `ReconciliationProposals.test.ts`
(bouton sur crédit CHF, absent sur débit), `reconciliation.api.test.ts` (URL, verbe, corps).

**Mutations à prévoir** (motif unique vérifié, restauration `git checkout -- f && touch f`) :

| M | Mutation | Tuée par |
|---|---|---|
| M1 | retirer `settlement_bank_account_id = ?` | T6 (décor à compte du grand livre partagé, sans quoi elle survit) |
| M2 | retirer `settlement_type = 'bank_transfer'` | **équivalente** (ci-dessous) |
| M3 | retirer le `NOT EXISTS` de la condition 3 | T8 |
| M4 | borne haute de l'écart `≤ 0,05` → `< 0,05` | T4 |
| M5 | borne basse `0 ≤ écart` retirée | T5 |
| M6 | `amount > 0` → `amount >= 0` dans `transaction_can_be_linked` | unitaire seulement — T9 ne la distingue pas : une transaction à 0.00 a `écart = −M < 0`, exclue par la borne basse, même `[]` et même 409 (validation P2, R10 = F7) |
| M7 | `auto_match_rejected_at = NULL` retiré de l'`UPDATE` | T2 |
| M8 | `AND status = 'pending'` retiré de l'`UPDATE` | **équivalente** (ci-dessous : l'étape 4.1 refuse avant l'`UPDATE`, T10 ne l'atteint pas) |
| M9 | audit retiré | T1 |
| M10 | `FOR UPDATE` de l'étape 4.2 (a) retiré | T16, T17 — **si** les deux requêtes se chevauchent, ce que rien ne garantit (C-29-8-25) : une survie se déclare « non tuée de façon fiable » au Dev Agent Record et renvoie à la revue de code |
| M11 | tri : `gap` retiré de l'`ORDER BY` | T14 (C, plus proche, passerait premier) |
| M12 | bouton affiché sur une ligne de débit | Vitest |
| M13 | présélection du premier candidat | Vitest |
| M14 | la requête de l'étape 4.4 déplacée **avant** l'étape 4.2 (vue née avant l'attente) | T17 — même réserve que M10 |
| M15 | `HAVING M > 0` retiré | **survivante assumée** (ci-dessous ; équivalente sauf au centime) |
| M16 | `M` lu sur `invoice_settlements.amount` au lieu des lignes du compte lié actuel (la variante écartée de C-29-8-4) | T18 (b) ; la variante « filtre de compte retiré » rend `M = 0` (écriture équilibrée) et tombe sous T1 |
| M17 | `s.company_id = ?` retiré de la requête | **équivalente** (ci-dessous) |
| M18 | `ABS(…)` retiré de `daysApart` (signé) | T14 (A, à −40, passerait avant B) |
| M19 | `ensure_open_in_tx` retiré de l'étape 4.3 | T13 (b) |

**Déclarées équivalentes ou non mutées, avec raison** (validation P1, F9) : M8 et la garde `matched_entry_id IS NULL`
de l'`UPDATE` — l'étape 4.1 lit et refuse ces deux états sous `FOR UPDATE` de la même ligne, tenu jusqu'au commit :
aucune exécution ne les atteint, ce sont des défenses en profondeur gardées comme celles de `post_manual`
(`:3896-3925`) ; le « `LIMIT` avant le filtre » n'a pas de forme d'un seul jeton dans un `SELECT` unique (il faudrait
un filtre Rust, ce que C-29-8-9 interdit) ; le `FOR UPDATE` de l'étape 4.2 (b) (`je`) n'est tué par aucun test — il
aligne l'ordre `s → je` sur `cancel_settlement_in_tx` ; les gestes qui prennent `je` **sans** `s` existent (la
modification, `journal_entries.rs:1453-1456` ; la suppression, `:1825-1831` ; la contre-passation directe,
`:2603-2607`), mais refusent une écriture possédée par un règlement (`document_owners`, bloc `invoice_settlements`,
`:2265-2270`) avant toute écriture, lien ou pas : le verrou n'aligne que l'ordre, déclaré survivant avec cette raison
(validation P2, F6 — la raison P1, « qu'aucun chemin ne fait aujourd'hui », était fausse).

**Déclarées équivalentes ou survivantes en validation P2** (R2 = F2 / F3, décision de l'orchestrateur : la raison au code, **aucun
décor construit pour les tuer**) :

- **M2** (`settlement_type = 'bank_transfer'` retiré) — la contrainte `chk_invoice_settlements_counterparty`
  (`20261001000004_invoice_settlements_write_off.sql:44-53`) impose `settlement_bank_account_id IS NULL` à tout
  règlement `internal_account` ou `write_off` : la condition 2 (`settlement_bank_account_id = t.bank_account_id`) est
  fausse pour eux (`NULL = x`), avec ou sans le filtre de type. Aucun décor ne la tue sans violer le `CHECK`.
- **M15** (`HAVING M > 0` retiré) — la **jointure interne** sur les lignes du compte lié actuel écarte déjà l'écriture
  sans ligne sur ce compte (T18 (a)) ; le `HAVING` ne décide que d'une somme ≤ 0, qu'aucun état courant ne produit
  (§ *Pourquoi `M > 0`*). **Survivante assumée, non équivalente à la lettre** (validation P3, R4 = F1) : l'état rare
  du compte de créance d'une vente antérieure, lié à la banque, donne `M < 0` ; le `HAVING` ne le distingue que d'un
  règlement de moins de 5 centimes. Aucun test ni décor.
- **M17** (`s.company_id = ?` retiré) — au `POST`, l'étape 4.2 (a) filtre `WHERE id = ? AND company_id = ?` et rend
  404 **avant** la requête ; au `GET` comme au `POST`, la condition 2 lie `s` au compte bancaire de `t`, et un
  règlement ne naît que sur un compte bancaire de sa société (`invoice_settlements_write.rs:137-142`, `WHERE id = ? AND
  company_id = ?`). Seul un règlement forgé en SQL la distinguerait. T11 reste un test d'IDOR, non sa tueuse.
- **Décompte** (validation P3, R8 = F2) : **quatre** mutations sans test tueur — M2, M8, M17 équivalentes strictes, M15
  survivante assumée ; M1 est tuée.
- **M1** reste **tuée** par T6, au prix d'une ligne de montage (deux comptes bancaires liés au même compte du grand
  livre, partage admis au sol, `bank_accounts.rs:1438-1491`) : c'est le seul cas où la condition 2 porte la décision,
  et un faux rattachement réel s'il manquait.

**AC9 — E2E Playwright** (`frontend/tests/e2e/reconciliation-link.spec.ts`, relevé unique à chaque exécution comme
`reconciliation-cancel.spec.ts`) : facture validée, règlement par l'API depuis la fiche, import d'un relevé portant le
paiement ; *Réconciliation* → *Rattacher à un règlement* → choisir → *Rattacher* → la ligne quitte la liste, « 1
opération(s) réussie(s). » ; détail de l'import : transaction rapprochée, lien vers l'écriture du règlement ; captures
clair et sombre (AC4). Gate E2E complet **au dernier commit de code** (D7), jugé fichier par fichier contre
`docs/testing.md` § *Les échecs attendus*.

**AC10 — Aucune migration ; gates.**

Aucun fichier sous `crates/kesh-db/migrations/` (le schéma suffit, C-29-8-2) ; `test_schema_guard` inchangé.
`kesh-db` est touché (nouveau dépôt, doc-comments) : **gate complet même en cours de boucle** (exception `kesh-db` du
`CLAUDE.md`). Gates complets backend, frontend et E2E au dernier commit de code, sous `flock`.

## Tasks / Subtasks

- [x] **T0 — Rebase** sur `origin/main` après merge des 29-2, 29-5 et 29-6 ; relire les numéros de ligne de la fiche ; base
  `kesh_298` et `kesh_e2e_298`, port 3048, `CARGO_TARGET_DIR=<worktree>/target`.
- [x] **T1 — Tests rouges** (AC8) : `reconciliation_link_e2e.rs` T1 à T20, unitaires, Vitest — rouges constatés.
- [x] **T2 — `kesh-db`** (AC1, AC2, AC7) : `repositories/reconciliation_link.rs` (`transaction_can_be_linked`, la
  requête des candidats, `link_in_tx` avec l'ordre des verrous et l'audit), `pub mod` ; doc-comments de l'AC7.
- [x] **T3 — `kesh-api`** (AC1, AC2) : deux handlers dans `routes/reconciliation.rs`, routes dans `lib.rs` (groupe
  Comptable, à côté de `…/unreject`), `AppError::ReconciliationSettlementNotLinkable`, `audit_labels.rs`,
  `audit_route_registry.rs` (cinq assertions chiffrées, prose chiffrée et point (vi)), `bank_account_closed_inventory.rs`
  (statut `Gardee`, `sites.len()` 31 → 32) ; `FENETRE` de la 29-4 si elle est mergée (AC2).
- [x] **T4 — Frontend** (AC4) : `LinkSettlementModal.svelte`, bouton, API, types.
- [x] **T5 — i18n** (AC5), quatre locales.
- [x] **T6 — Manuel FR + PDF, `api-external.md`, CHANGELOG** (AC6) ; inventaire par symptôme trié (tableau de l'AC6,
  repris après rebase) ; contrôles négatifs du PDF aplati.
- [x] **T7 — Mutations** (AC8, M1 à M19 et les équivalences déclarées), table au Dev Agent Record ; table *Absence
  de cycle* recopiée et relue (AC2).
- [x] **T8 — E2E + captures** (AC9) ; gates complets (AC10).

## Dev Notes

### Inventaire (a) — les sites qui posent `matched_entry_id`, au sol (`fcd2ae75`)

`grep -n "matched_entry_id = ?" crates/kesh-api/src/routes/reconciliation.rs` : `:2123` (acceptation facture),
`:2622` (proposition éclatée), `:3074` (règle), `:3901` (manuel), `:4364` (éclatement manuel). Tous **créent**
l'écriture qu'ils lient. Le rattachement est le **sixième** chemin et le **seul** qui lie une écriture existante.
Le seul chemin qui **défait** le lien : `reconciliation_cancel.rs:346`.

### Inventaire (b) — les sites qui comptent une transaction en attente (non résolus par cette story pour les cas hors périmètre)

| Site | Lit | Après rattachement |
|---|---|---|
| `bank_accounts::close_in_tx` (`bank_accounts.rs:1018-1032`) | `status = 'pending'` | la transaction rattachée ne compte plus |
| K2 de la 29-1a (non mergée) | `status = 'pending'` sur l'exercice | idem |
| `GET /reconciliation/proposals` | `pending`, non rejetées | elle quitte la liste |
| Détail de l'import (`is_reconciled`) | statut + lien | « rapprochée », lien vers l'écriture |
| Écart relevé / grand livre (`list_by_company_with_balances`) | transactions rapprochées pour la correction de dates | la ligne de l'écriture sur le compte lié entre dans la correction, comme pour une proposition acceptée |

**Restent bloquées, hors périmètre** — chacun un angle mort écrit, **chacun adossé à une issue** (leçon § 4.3 de la
rétrospective 28 : aucune dette assumée sans issue) :

- **L2** — une transaction de **débit** payée depuis la fiche d'une facture fournisseur (`supplier_invoices::pay_in_tx`,
  choix `BankTransfer`, `supplier_invoices.rs:750-781`) : même blocage que #670, côté fournisseurs — **#671**. Les
  **lots de paiement pain.001** en relèvent aussi (validation P1, F7, établi au code) : `confirm_batch` règle chaque
  facture par `supplier_invoices::pay_in_tx`, une écriture par facture (`kesh-db/src/repositories/payment_batches.rs:461-471`), sans lire
  ni écrire `bank_transactions` ; la transaction du relevé est au **débit** (exclue par la condition 1) et paie
  **N** factures (L4) — le « à vérifier » de #671 est tranché, versé à l'issue par l'orchestrateur ;
- **L3** — une transaction comptabilisée par une **écriture saisie au journal**, ou par un règlement passé sur un
  **ancien** compte du grand livre du compte bancaire (écarté par la jointure interne sur le compte lié actuel, T18) —
  **#671** ;
- **L4** — une transaction qui paie **plusieurs** factures réglées séparément depuis leurs fiches (une transaction,
  un lien : `matched_entry_id` est unique) — **#671** ;
- **L5** — un règlement saisi sur le **mauvais compte bancaire** (le dialogue préremplit le compte principal,
  `SettleInvoiceDialog.svelte:120-127`) : pas candidat ; annuler le règlement et le refaire sur le bon compte —
  réparable par les gestes existants, versé à **#671** par commentaire de l'orchestrateur ;
- **L6** — un paiement qui **dépasse** le règlement de plus de 5 centimes (validation P1, R4) : le cas que le manuel
  documente comme normal pour un rappel — le client paie « reste dû **et** frais de rappel », seul le montant de la
  QR-facture s'enregistre depuis la fiche, le surplus étant refusé comme trop-perçu (`user-manual.tex:1308`,
  `invoice_settlements_write.rs:223-228`) ; la transaction (par exemple 120 pour 100) a `écart = 20 > 0,05` et **ne se
  rattache pas** : elle reste en attente, bloque la clôture du compte et K2. Élargir la tolérance n'est pas du
  périmètre minimal : angle mort écrit (AC6, tableau `:1308` ; CHANGELOG), renvoyé à **#671** (commentaire posé par
  l'orchestrateur) et à **#401** (les frais de rappel ne sont pas comptabilisés) — C-29-8-23 ;
- **L7** — un règlement saisi par **« compte interne »** sur le compte du grand livre **de la banque** (validation P2,
  F3) : `settle_invoice` l'admet (compte actif et imputable, `invoice_settlements_write.rs:179-200`), son écriture
  débite bien la banque, mais il n'est pas un virement (condition 2) : pas candidat. Même remède que L5 — annuler le
  règlement et le refaire par virement sur ce compte bancaire —, versé à **#671** par commentaire de l'orchestrateur.
  Angle mort écrit, **sans test neuf** (C-29-8-28).

**L1** — *Annuler le rapprochement* d'une transaction rattachée annule aussi le règlement : pas de geste « détacher
seulement ». Visible (contre-passation, facture rouverte), réparable (refaire le règlement, rattacher), sans erreur
comptable. Évolution tracée par **#672** (« détacher sans annuler »).

### Absence de cycle — établie par la lentille F en validation P1, à recopier et relire au Dev Agent Record

Ordre du rattachement : verrou nommé → `t` → `s` → `je` (trois verrous de ligne par clé primaire), puis lectures
ordinaires. Relevé sur `fcd2ae75`, **raisonné, non exécuté** :

| Geste | Ordre au code | Avec le rattachement |
|---|---|---|
| `cancel_settlement_in_tx` | facture → `s` → `je ⋈ fy` (`invoice_settlements_write.rs:835`, `:850`, `:881`), puis lectures ordinaires | le rattachement ne prend pas la facture ; il attend `s` sans rien détenir qu'attende le détenteur ; après l'attente, `s` est absente → 404 (T16) |
| `reconciliation_cancel::cancel_in_tx` | `t` → `s` → facture → `je ⋈ fy` (`reconciliation_cancel.rs:287-319`) | même `t` d'abord : exclusion mutuelle sur `t`. Sur deux `t` différents, ils **peuvent** viser le même `s` (validation P2, R10) : `t1` rapprochée de `s` qu'on annule, `t2` en attente qu'on rattache à `s` — l'annulation prend `s` par `journal_entry_id` (`:297`, `classify(…, true)`), le rattachement aussi. Pas de cycle : chacun ne tient que **son** `t` avant `s`, seul verrou commun ; le perdant voit `s` supprimé (404) ou le lien (409) |
| `accept_one_invoice` / `settle_invoice` | verrou nommé → facture → compte → exercice → insertions (`invoice_settlements_write.rs:91`, `:139`) | écritures **neuves** : jamais un `s` ni un `je` existants |
| `journal_entries::delete_in_tx` / `reverse_in_tx_inner` | `je ⋈ fy` → lignes (`journal_entries.rs:1827-1831`, `:2607`, `:2694`) | le rattachement ne verrouille jamais les lignes ; ces gestes ne verrouillent jamais `bank_transactions` ; une écriture de règlement ne se supprime pas (garde de propriété) |
| `bank_accounts::close_in_tx` | ligne du compte → décomptes ordinaires (`bank_accounts.rs:1002-1032`) | le rattachement ne verrouille pas `bank_accounts` ; non commité, la transaction reste `pending` : la clôture refuse (sûr) ; commité, `ensure_open_in_tx` l'a vu ouvert |
| suppression d'import | en-tête → transactions par id croissant, `FORCE INDEX` (`bank_imports.rs:395`, `:455-470`) | le rattachement prend `t` d'abord : exclusion sur `t` ; aucun autre verrou de ligne de `bank_transactions` (l'étape 4.3 d'origine retirée, C-29-8-17) |

⚠️ **Un cycle préexistant, que la story ne crée pas** : `reconciliation_cancel` prend `s` puis la facture,
`cancel_settlement_in_tx` la facture puis `s` ; les deux routes sont rejouées. Le rattachement multiplie les états qui
l'exposent (une transaction rattachée s'annule par la première, son règlement par la seconde), sans le créer ni le
fermer : à écrire tel quel au Dev Agent Record.

### Zones partagées avec les 29-2, 29-5, 29-6 et 29-1 (fiches non mergées, lues en lecture seule)

| Zone | 29-2 | 29-6 | 29-8 |
|---|---|---|---|
| `ReconciliationProposals.svelte` | — | AC10 : case sur une ligne sans proposition, *Accepter* grisé, indication | troisième bouton de ligne, modal ; **ne touche pas** la case ni l'indication |
| Indication fr-CH de la 29-6 « Une transaction sans correspondance ne peut pas être acceptée : décochez-la pour accepter les autres. » (rédaction de sa validation P2, fiche 29-6 `:478`) | — | AC10 | **inchangée** : elle ne dit que ce qui est vrai de la sélection et ne nomme aucun geste (C-29-6-16) ; les boutons de ligne restent offerts |
| Manuel : « puis rejetez la transaction » (et formes voisines) — inventaire **par le grep** de l'AC6, au rebase ; la **29-5** y entre aussi (`29-5-departage-exigibilite.md:326-327`, paragraphe de l'algorithme de matching) | C-29-2-8, `:314`, `:324` de sa fiche | AC7, `:274-345` de sa fiche (`:2029`, `:2050`, `:2057`, FAQ `:2910-2911` ; aucun « 670 » au manuel, C-29-6-17) ; CHANGELOG `:355-362` | réécrit en « rattachez » ; `:2050` complété ; `:2057` inchangé (AC6) |
| `routes/reconciliation.rs` | fenêtre de dates | borne basse du reste dû | deux handlers neufs, aucune ligne de la proposition ni de l'acceptation |
| K2 (29-1a) | — | — | aucune ligne ; la transaction rattachée en sort par son statut (assertion à T15 si la 29-1 est mergée) |
| `bank_account_closed_inventory.rs`, `audit_route_registry.rs` (29-4) | — | — | la 29-4 ajoute le troisième inventaire `FENETRE` et des compteurs : `link_in_tx` y entre, par celle des deux stories qui se merge la seconde (AC2) ; l'ordre de l'epic (« 29-8 après la 29-6 », « 29-2, puis 29-3, puis 29-4 ») ne fixe pas leur ordre relatif |

### Propagation — le symptôme à greper après chaque patch

- `\bcinq\b` près de « chemin(s) », `existerait déjà`, `viennent de créer` (AC7) ;
- `rejet` et `670` dans le manuel, `api-external.md`, `CHANGELOG.md` (AC6) ;
- `RECONCILIATION_SETTLEMENT_NOT_LINKABLE` : `errors.rs`, `api-external.md`, quatre `.ftl` ;
- `reconciliation.linked` : `audit_labels.rs`, quatre `.ftl`, `api-external.md` (si l'audit y est listé), tests ;
- `ne la relie pas`, `tout seul`, `depuis la fiche` dans le manuel `.tex` **et** le PDF aplati (AC6) ;
- **la valeur**, non la formulation (validation P2, R6) : `grep -nE "\b(118|121|112|9[34])\b|\bsept\b"
  crates/kesh-api/tests/audit_route_registry.rs` (cinq assertions, prose `:27`, `:31`, `:32`, messages `:659`, `:662-675` et
  `:1007`, point (vi) `sept` → `huit` ; les « sept » de `:59` et `:73`, autre décompte, restent) et `grep -nE
  "\b3[01]\b" crates/kesh-api/tests/bank_account_closed_inventory.rs` (`sites.len()` 31 → 32, commentaire `:488-490`) ;
- `ne relie pas la transaction`, `rejetez-la`, `crée l'écriture`, `date de valeur`, `issue d'un rapprochement`,
  `rien ne la relie` dans le manuel `.tex` et le PDF aplati (AC6) ;
- **le retrait de l'observation de l'attente** (C-29-8-25) : `grep -nE "INNODB_TRX|LOCK WAIT|vu bloqué|observ"` sur la
  fiche et le registre ne rend plus aucune prescription pour T16 et T17 (T13 (b), qui observe `GET_LOCK` par
  `attendre_une_requete_en_cours`, est hors de ce retrait) ;
- toute lecture verrouillante sur `matched_entry_id` (`grep -n "matched_entry_id = ?" crates/kesh-db/src/repositories/reconciliation_link.rs`) :
  il n'en faut **aucune** suivie de `FOR UPDATE` (C-29-8-17).

### Ce qui doit être préservé

- `GET /reconciliation/proposals` et `POST /reconciliation/accept` **inchangés** (aucune ligne).
- `classify_payment`, `settle_invoice` : inchangés.
- `reconciliation_cancel` : **code** inchangé (seul son doc-comment change, AC7).
- Aucune écriture au journal, aucun règlement, aucun lettrage par le rattachement.

### Exclusions écrites (périmètre minimal)

- Proposer automatiquement les rattachements dans la liste des propositions : non (C-29-8-7).
- Offrir le geste depuis le détail de l'import pour une transaction rejetée : non — *Annuler le rejet* d'abord ;
  l'API, elle, admet une rejetée (comme `/manual`).
- L1 à L7 : hors périmètre, chacun adossé à #671, #672 ou #401.

### Modules touchés (règle de découpage : au plus cinq)

`kesh-db` (dépôt neuf, doc-comments), `kesh-api` (routes, erreurs, libellés d'audit, tests), `kesh-i18n` (quatre
locales), `frontend/src/lib/features/reconciliation` ; plus la documentation (manuel, `api-external.md`, CHANGELOG).
**Quatre modules de code** : pas de découpage (C-29-8-16 n'en dépend pas). Aucune migration.

### Pièges connus du dépôt, applicables ici

- `127.0.0.1` et `KESH_COOKIE_SECURE=false` pour l'E2E ; base de gate remise à zéro avant tout gate complet.
- Après une mutation restaurée, `touch` le fichier (cargo compare des dates).
- Une lecture non verrouillante fige la vue : aucune avant le dernier `FOR UPDATE` (AC2, étape 4).
- Aucun `closes #670` dans un message de commit, même en prose.

### References

- Issue #670 ; #666 (piste b) ; #548 (une seule borne) ; #651 (clôture concurrente, 29-4).
- `crates/kesh-db/src/repositories/invoice_settlements_write.rs`, `reconciliation_cancel.rs`, `bank_accounts.rs`,
  `bank_imports.rs`, `reconciliation.rs`, `settlement_cancellation.rs`.
- `crates/kesh-api/src/routes/reconciliation.rs` (`post_manual`, `post_unreject_transaction`), `lib.rs:646-686`.
- `frontend/src/lib/features/reconciliation/ReconciliationProposals.svelte`, `ManualMatchModal.svelte`.
- Fiches 29-2, 29-6, 29-1 (worktrees `kesh-29-2`, `kesh-29-6`, `kesh-29-1`).

## Dev Agent Record

### Agent Model Used

Claude Opus 5.5 (`bmad-dev-story`, autonomie), 2026-10-14, worktree `/home/gcorbaz/devel/kesh-29-8`, branche `story/29-8`,
bases `kesh_298` / `kesh_e2e_298`, port 3048, `CARGO_TARGET_DIR=/home/gcorbaz/devel/kesh-29-8/target`.

### Debug Log References

Journaux (hors dépôt) : `/home/gcorbaz/devel/kesh-gate-logs/298-*` — `298-red-2.log` (rouge des tests d'intégration),
`298-vitest-red.log` (rouge Vitest), `298-green-*.log` (gates ciblés), `298-mut-*.log` et `298-mutations-result.txt`
(campagne, script `298-mutations.py`), `298-make-fr.log`, `298-final-test-fast-run1.log` / `298-final-front-run1.log`
(premier gate complet, avant le second rebase), `298-final-test-fast.log`, `298-final-front.log`, `298-final-e2e.log` (gates finaux, HEAD rebasé).

### Completion Notes List

**T0 — Rebase** sur `origin/main` `e889346c` : sept commits de fiche rejoués (`240516d4` → `4af8f8ca`). Conflits sur
`sprint-status.yaml` (trois commits) et le fichier d'epic (deux commits), résolus par union avec
`union_sprint.py` / `union_epic.py` : clés de la 29-8 de la branche, les autres de `main` ; l'en-tête de la 29-8
remonté **au-dessus** de ceux de `main` et renuméroté **(159)** (le n° 135 de la branche, remplacé en fin de fiche par
le n° 136, ne subsiste qu'en un en-tête) ; dans le fichier d'epic, la puce « 29-8 après la 29-6 » replacée **après**
celle de la 29-6 (le script l'avait insérée au milieu de la puce 29-2/29-3/29-4), la ligne v0.13.4 unique, le reliquat
« manuel de la release prescrirait… » retiré. `yaml.safe_load` passe, aucun marqueur de conflit ; fiche et registre de
la 29-8 identiques à `d4925683` (`git diff --stat` vide). Le reliquat « 29-6, R7)* : Guy les éprouve… » du fichier
d'epic **préexiste sur `main`** : laissé tel quel. **Second rebase**, en fin de développement, sur `12c64c39` (29-4b) à
la demande de l'orchestrateur : en-têtes de `sprint-status.yaml` (la 29-8 au-dessus, renumérotée **(161)**) ; registres
de tests (`audit_route_registry.rs` point (vi) : texte de la 29-4b gardé, phrase de la 29-8 ajoutée ;
`bank_account_closed_inventory.rs` : doc-comment de T29a-5 à onze sites `G` et **sept** revérifications, raison
`FENETRE` de `link_in_tx` sans la réserve « sous réserve de la 29-4b », comme celle de l'annulation de rejet) ;
`CHANGELOG.md` et `api-external.md` réunis (textes de la 29-4b gardés, ajouts de la 29-8 réinsérés) ; PDF repris de
`main` pendant le conflit puis régénéré (`touch` du `.tex`, `make fr`) et contrôlé aplati : phrases des 29-6 (« ne la
traite pas »), 29-4b (« Deux précisions. », « ne passe pas sur le compte clos ») et 29-8 présentes, aucun « ?? » ; la
suppression du registre propre à la 29-4 gardée. La campagne de mutations a tourné **avant** ce rebase et n'a pas été
rejouée : la 29-4b ne touche aucun des trois fichiers mutés, mais modifie `bank_accounts.rs` (gardes des annulations)
et cinq lignes de `routes/reconciliation.rs` ; le binaire `reconciliation_link_e2e` est vert au gate du HEAD rebasé.

**Prémisses relues sur `e889346c`** — toutes exactes ; seuls les numéros de ligne ont bougé :
- les cinq chemins qui posent `matched_entry_id` : `routes/reconciliation.rs:2293`, `:2792`, `:3244`, `:4071`, `:4534`,
  tous sur une écriture neuve ; `close_in_tx` (`bank_accounts.rs:1215`) compte les `pending`, rejetées comprises ;
  `ensure_open_in_tx` à `:853` ; `AMOUNT_TOLERANCE_HUNDREDTHS` à `:79`, son site existant à `:731` (inchangé) ;
- la **29-6** a rendu les lignes « Aucune correspondance » cochables et rejetables, et propose le reste dû exact ou
  inférieur de 5 centimes ; le client qui a payé quelques centimes de trop s'enregistre depuis la fiche **au reste dû**
  (la fiche refuse le trop-perçu) : la transaction dépasse alors le règlement de 1 à 5 centimes — la prémisse de l'AC1
  tient, aucune règle métier ne change ;
- la **29-1a** est mergée : K2 (`PENDING_BANK_TRANSACTIONS`) existe, l'assertion (5) de T15 est donc écrite ;
- la **29-4a** est mergée la première : le registre `FENETRE` existe, la 29-8 y inscrit `link_in_tx` (C-29-8-36) ; le
  point (vi) d'`audit_route_registry.rs` disait déjà **treize** (C-29-8-35) ;
- `ensure_bank_account_open` ne rend que le clos : le pré-vol commun est `link_preflight` (C-29-8-34).

**Étape 0 — inventaire des conseils de rejet** (`grep -nE 'rejet[a-z]*( ensuite)?( la transaction|-la)|rejetez|rejetée, elle|Rejetée, la'`
sur `user-manual.tex`, `CHANGELOG.md`, `docs/api-external.md`, puis `rejetez|rejetant|puis rejet` sur tout le dépôt
hors `_bmad-output`) — sept occurrences vivantes, **toutes réécrites** en « rattachez » :

| Site (après rebase) | Écrit par | Réécriture |
|---|---|---|
| `user-manual.tex` *Algorithme de matching*, paiement de plus de 30 jours (« puis rejetez la transaction ») | 29-2 | « puis rattachez la transaction à ce règlement (…) : elle passe rapprochée, sans écriture » |
| idem, trop-perçu de quelques centimes (« puis rejetez la transaction … cochez sa ligne ») et l'incise « la transaction rejetée reste en attente … » | 29-6 | « rattachez la transaction à ce règlement » ; l'écart affiché et « traitez l'écart » gardés ; l'incise retirée |
| `sec:facture-posterieure` (« en rejetant ensuite la transaction ») | 29-2 | « en rattachant ensuite la transaction à ce règlement » |
| imputation à score égal (« puis rejetez la transaction ») | 29-5 | « puis rattachez la transaction à ce règlement » |
| FAQ « Que faire ensuite », paiement de client (« Kesh ne relie pas … rejetez-la … rejetée, elle reste en attente ») | 29-6 | « Puis rattachez la transaction à ce règlement (…) : elle passe rapprochée, sans écriture » ; « N'utilisez ni *Affecter manuellement* ni *Éclater* » gardé |
| `CHANGELOG.md` § 0.13.4, entrée #529 (« puis rejetez la transaction ») | 29-2 | « puis rattachez la transaction à ce règlement » |
| `CHANGELOG.md` § 0.13.4, entrée #665 (« rejetez ensuite la transaction … Rejetée, la transaction reste en attente ») | 29-6 | « rattachez ensuite la transaction à ce règlement » ; la case de la ligne « Aucune correspondance » dite « pour être rejetée » |

Restent, **triées et inchangées** : la description générale du rejet (*Rejeter*, « elle compte comme non rapprochée et
empêche la clôture de son compte bancaire ») ; la clôture du compte (« une transaction rejetée … compte comme non
rapprochée », complétée du rattachement) ; la suppression d'import (« une transaction rejetée ne compte pas ») ; le
rejet d'un virement par la banque (lots de paiement, sans rapport) ; le titre « Accepter ou rejeter les propositions » ;
le commentaire de `frontend/tests/e2e/reconciliation-reject-unmatched.spec.ts:7` (décrit la 29-6 telle qu'elle a été
écrite, et le rejet d'une ligne sans proposition reste un geste permis) ; `reconciliation_e2e.rs:7805` (commentaire de
test sur le rejet d'un autre compte).

**Inventaire du manuel par symptôme (AC6)** — `grep -nE "depuis la fiche|ne la relie pas|ne relie pas|tout seul|reglement-client|rejet|670|crée l'écriture|date de valeur|issue d'un rapprochement|rien ne la relie"`,
après rebase, chaque site trié :
- **réécrits** : tuile d'écart `:261-270` (date du règlement, « issue d'un rapprochement ou d'un rattachement ») ;
  `sec:reglement-client` `:1231` (+ « Si le virement est déjà importé, rattachez ensuite la transaction au règlement ») ;
  frais de rappel `:1341` (+ L6, « ne se rattache pas », #671 et #401) ; clôture du compte `:1725` (« ou rattachée au
  règlement saisi depuis la fiche ») ; introduction `:2055` (« sauf dans un cas … se rattache ») ; matching `:2063`,
  `:2065`, imputation `:2079` (conseils de rejet, ci-dessus) ; ligne « Aucune correspondance » `:2090` (+ *Rattacher à
  un règlement*) ; « Pour imputer une transaction autrement » `:2100` ; fin de *Réconciliation manuelle* `:2116` (+
  « puis rattachez-y la transaction ») ; sous-section neuve `:2118-2142` ; *Annuler un rapprochement* (bouton de la
  fiche, puce « le règlement est retiré aussi ») ; FAQ `:2982` ; glossaire `:3124` ;
- `:1305` de la fiche (« Ce que réclame un rappel », `:1338` après rebase) : la 29-2 l'a réécrite, **aucun renvoi au
  règlement depuis la fiche n'y subsiste** — inchangée, comme le prescrivait la fiche ;
- **inchangés**, motif d'une ligne : `:652` (annulation d'une écriture contre-passée, sans rapport à la banque) ; `:986`
  (écriture de validation d'une facture) ; `:999-1000` (une facture se règle depuis la fiche ou par rapprochement : vrai) ;
  `:1378`, `:1380` (dévalidation : annuler d'abord le règlement — vrai, un règlement rattaché ne s'annule qu'après
  *Annuler le rapprochement*, que dit `:652`) ; `:1464` (refus de dévalidation, vrai) ; `:1561` (note du règlement) ;
  `:1659` (« L'import ne se déclenche pas tout seul » — boîte de réception, sans rapport) ; `:1677` (facture
  fournisseur d'origine) ; `:1776`, `:1875`, `:1881` (suppression d'import, rejet, vrais) ; `:2096`, `:2097`, `:2102`
  (acceptation, rejet général, bilan) ; `:2112` (sélecteur des comptes de l'affectation manuelle) ; `:2268`, `:2376`,
  `:2403`, `:2592`, `:3003`, `:3015` (renvois au règlement ou à la contre-passation, sans rapport à la transaction
  bancaire) ; « rien ne la relie au relevé » `:270` reste vrai pour une écriture saisie au journal (L3) ;
  « tout seul. » `:1231` reste vrai (validation P3, R2).

**PDF** (`make fr` sous le verrou, user-manual 3 passes stables ; admin-manual et brochure régénérés sans changement de
source, remis à leur version commitée), aplati et normalisé, comptes `grep -oF` : positifs — « rattachez la
transaction à ce règlement » **4** (matching ×2, imputation, FAQ ; la fiche demandait ≥ 2), « rattachez ensuite la
transaction au règlement » 1 (`:1231`), « Rattacher une transaction à un règlement » 2 (titre et table des matières),
« Rattacher à un règlement » 5 (ligne, imputation, matching, sous-section, FAQ), « garde la date du règlement » 1,
« se rattache à ce règlement, sans écriture » 1 (glossaire), « ou rattachée au règlement » 1, « rattachez-y la
transaction » 1, « ne se rattache pas » 3 (frais de rappel, sous-section ×2), « en rattachant ensuite la transaction » 1 ;
négatifs, adossés aux positifs — « ne la relie pas » 0, « ne relie pas la transaction » 0, « rejetez-la » 0,
« existerait déjà » 0, « date les écritures de rapprochement à la date de valeur. » 0, « puis rejetez » 0 ; relevé
« tout seul. » 1, « rien ne la relie au relevé » 1 (gardés).

**AC7 — les douze sites** : `bank_accounts.rs` (deux doc-comments : date de valeur sauf l'écriture d'un règlement
rattaché ; six chemins, le sixième lie une écriture existante, condition 3) ; `reconciliation_cancel.rs` (en-tête : deux
chemins rendent `InvoiceSettlement` ; doc de la variante) ; `bank_imports.rs:380` (six chemins) ;
`kesh-db/tests/reconciliation_cancel.rs` (en-tête : le sixième chemin couvert par `reconciliation_link_e2e.rs` T15, ce
fichier n'en a pas besoin) ; `invoices.rs` (deux doc-comments de tests : « neuve ou, par le rattachement, celle d'un
règlement ») ; `reconciliation_e2e.rs` (« cinq chemins le remettent à zéro ») ; `bank_account_statement_gap.rs:7` ;
`user-manual.tex` introduction et *Annuler un rapprochement* (ci-dessus). Les hors-sujet de la fiche restent hors sujet
(`routes/reconciliation.rs:2268` « cinq sites qui lisent `paid_at IS NULL` », `reconciliation_e2e.rs:3762`, etc.).
Propagation de la valeur : `grep -rnE "\bcinq\b.*(chemin|site)|quatre chemins|viennent de créer|NEUVE"` ne rend plus
que les hors-sujet triés et `reconciliation_link.rs` (« Les cinq autres chemins »).

**T1 — tests rouges d'abord.** `reconciliation_link_e2e.rs` (T1 à T20, plus `compte_non_lie_rend_412` et
`transaction_inexistante_rend_404`) écrit avant toute ligne de production : **23 exécutés, 0 réussi, 23 échoués**
(`298-red-2.log`), chacun sur l'absence des routes (`404`/`405`), par exemple :

```
thread 't1_le_rattachement_debloque_la_cloture_du_compte' panicked at crates/kesh-api/tests/reconciliation_link_e2e.rs:543:5:
assertion `left == right` failed: null
  left: 404
 right: 200
     Summary [   1.000s] 23 tests run: 0 passed, 23 failed, 0 skipped
```

(Un premier run, `298-red-1.log`, avait révélé un défaut du montage — la dévalidation exige `{ version }`, `422` — corrigé
avant le run rouge retenu.) Vitest, avant la modale et l'API : **4 échoués / 44** sur les trois fichiers
*(annoté le 2026-10-14, revue de code P1, A6 : lire « 4 échoués, 44 réussis (48) » ; `LinkSettlementModal.test.ts`
a échoué **à l'import**, si bien que ses tests n'ont pas été vus rouges un à un — `298-vitest-red.log:1018`)*
(`298-vitest-red.log` : « Failed to resolve import "./LinkSettlementModal.svelte" », « Unable to find an element by:
[data-testid="link-settlement-button"] » ×2, « getLinkCandidates is not a function », « linkTransaction is not a
function »). Les deux unitaires de `kesh-db` (`transaction_can_be_linked`, texte unique de la requête) sont nés avec le
module.

**T2-T5 — réalisé** : `kesh-db/src/repositories/reconciliation_link.rs` (module neuf : `transaction_can_be_linked`,
`candidates_sql`, `find_link_candidates`, `link_in_tx`, `LinkRequest`, `LinkRefusal`, `Linked`) ; deux handlers,
`link_preflight`, `link_amount_tolerance`, `link_once` dans `routes/reconciliation.rs` ; routes dans `lib.rs` ;
`AppError::ReconciliationSettlementNotLinkable` ; `audit_labels.rs` ; registres (`audit_route_registry.rs` : 119 / 113 /
95 / 122 / 122, recomptés par le test lui-même au gate ; `bank_account_closed_inventory.rs` : statut `Gardee`,
`sites.len()` 32, `FENETRE` + 1, `G` 11) ; `textes_coherents.rs` (G9) ; `LinkSettlementModal.svelte`, bouton et
`onLinkSuccess`, API et types ; clés i18n en quatre locales (17 clés `reconciliation-link-*` / `-errors-settlement-not-linkable`
/ `audit-log-action-reconciliation-linked` par locale, comptées au `.ftl`) ; spec E2E `reconciliation-link.spec.ts`.

**Table *Absence de cycle*** — recopiée et relue contre le code du HEAD :

| Geste | Ordre au code | Avec le rattachement |
|---|---|---|
| `link_in_tx` (ce geste) | verrou nommé → `t` (`find_by_id_for_account_for_update`) → `s` (`invoice_settlements … FOR UPDATE`) → `je` (`journal_entries … FOR UPDATE`) → `ensure_open_in_tx` (première lecture ordinaire) → requête de la règle → `UPDATE` → audit | — |
| `cancel_settlement_in_tx` | facture → `s` → `je ⋈ fy` (`invoice_settlements_write.rs:835`, `:850`, `:881`), puis lectures ordinaires | le rattachement ne prend pas la facture ; il attend `s` sans rien détenir qu'attende le détenteur ; après l'attente, `s` absente → 404 (T16) |
| `reconciliation_cancel::cancel_in_tx` | `t` → `s` (`classify(…, true)`, `:302`) → facture (`:304`) → `je ⋈ fy` | même `t` d'abord : exclusion mutuelle ; sur deux `t` différents qui visent le même `s`, chacun ne tient que son `t` avant `s`, seul verrou commun : pas de cycle |
| `accept_one_invoice` / `settle_invoice` | verrou nommé → facture → compte → exercice → insertions | écritures neuves : jamais un `s` ni un `je` existants |
| `journal_entries::delete_in_tx` / `reverse_in_tx_inner` | `je ⋈ fy` → lignes | le rattachement ne verrouille jamais les lignes ; une écriture de règlement ne se supprime pas (garde de propriété) |
| `bank_accounts::close_in_tx` | ligne du compte → décomptes ordinaires | le rattachement ne verrouille pas `bank_accounts` ; non commité, la transaction reste `pending` : la clôture refuse ; commité, `ensure_open_in_tx` l'a vu ouvert |
| suppression d'import | en-tête → transactions par id croissant | le rattachement prend `t` d'abord : exclusion sur `t` |

Aucune correction à la table de la fiche. ⚠️ Le cycle préexistant `reconciliation_cancel` (`s` puis facture) /
`cancel_settlement_in_tx` (facture puis `s`) est **inchangé** ; le rattachement multiplie les états qui l'exposent, sans
le créer ni le fermer. L'ordre d'acquisition et la naissance de la vue restent **raisonnés, non prouvés** (C-29-8-25) :
renvoyés à la revue de code.

**T7 — mutations** (`298-mutations.py`, sous le verrou ; motif vérifié unique, `git checkout -- f` puis `touch f` après
chacune ; `git status` vide de toute source après la campagne) :

| M | Mutation (forme jouée) | Verdict | Tuée par |
|---|---|---|---|
| M1 | `s.settlement_bank_account_id = ?` → `(… OR TRUE)` | tuée | T6 |
| M2 | `settlement_type = 'bank_transfer'` retiré | **survit — équivalente** (`chk_invoice_settlements_counterparty`) | — |
| M3 | `NOT EXISTS (…)` → `TRUE` | tuée | T8, T17 |
| M4 | `<= ?` → `< ?` | tuée | T4 |
| M5 | borne `>= 0` → `(… OR TRUE)` | tuée | T4, T5 |
| M6 | `amount > 0` → `>= 0` | tuée (unitaire) | `transaction_can_be_linked_is_a_chf_credit` |
| M6 (intégration) | même mutation, binaire d'intégration seul | **survit**, comme déclaré (T9 ne la distingue pas) | — |
| M7 | `auto_match_rejected_at = NULL` retiré de l'`UPDATE` | tuée | T2 |
| M8 | `AND status = 'pending'` retiré de l'`UPDATE` | **survit — équivalente** (étape 4.1) | — |
| M8bis | `AND matched_entry_id IS NULL` retiré de l'`UPDATE` | **survit — équivalente** (étape 4.1) | — |
| M9 | audit retiré | tuée | T1, T2, T3, T12 |
| M10 | `FOR UPDATE` de l'étape 4.2 (a) retiré | tuée **sur ce run** | T16 — non fiable (chevauchement non garanti, C-29-8-25) |
| M11 | `gap` retiré de l'`ORDER BY` | tuée | T14 |
| M12 | bouton affiché sur toute ligne | tuée | Vitest `ReconciliationProposals` (M12) |
| M13 | présélection du premier candidat | tuée | Vitest `LinkSettlementModal` (×2) |
| M14 | étape 4.4 déplacée avant 4.2 | tuée | **T11** (le règlement d'une autre société rend alors `409 NOT_LINKABLE` au lieu de `404`) — T17 ne l'a pas tuée sur ce run ; la propriété d'origine (vue née avant l'attente) reste non prouvée, renvoyée à la revue |
| M15 | `HAVING … > 0` → `HAVING TRUE` | **survit — survivante assumée** (équivalente sauf au centime) | — |
| M16 | `M` lu sur `invoice_settlements.amount`, table dérivée retirée | tuée | T18 |
| M17 | `s.company_id = ?` → `(… OR TRUE)` | **survit — équivalente** (étape 4.2 (a), condition 2) | — |
| M18 | `ABS(…)` retiré | tuée | T14 |
| M19 | `ensure_open_in_tx` retiré de l'étape 4.3 | tuée | T13 (b) |

Décompte : **21 mutations jouées** (M1 à M19, plus M6 en intégration et M8bis) : **15 tuées**, **6 survivantes** — M2,
M8, M8bis, M17 équivalentes strictes, M15 survivante assumée, M6 en intégration seule survivante comme prévu (tuée par
l'unitaire). Aucune mutation n'a cassé la compilation.

**Écarts à la fiche** (aucun ne change une règle métier) : C-29-8-30 (transaction posée par le dépôt dans les tests
d'intégration), C-29-8-31 (compte du grand livre lié lu au pré-vol), C-29-8-32 (compteur « 1 opération(s)
réussie(s). » après un rattachement, que l'affectation manuelle n'affiche pas), C-29-8-33 (CHANGELOG sans rubrique
« Limites connues » ni entrée *Modifié*), C-29-8-34 (`link_preflight`), C-29-8-35 (point (vi) : treize → quatorze),
C-29-8-36 (`FENETRE`). Hors des AC : T12 vérifie aussi que la clé `read` **lit** les candidats (`200`), ce que
`docs/api-external.md` affirme. Le premier gate frontend complet (avant le second rebase) a rougi sur l'anti-muet
`i18n-keys.test.ts` (`sitesTotal` 2164 → 2180, +16, recompté aux deux bornes) : corrigé par le commit `test(29-8): garde i18n` ; l'E2E lancé
derrière ce gate rouge, sans `frontend/build`, a été **interrompu par PID** avant tout résultat et relancé.

**Gates au HEAD rebasé sur `12c64c39` (code du dernier commit de code, `918f1c5a`)** — base remise à zéro dans le même journal, sous le verrou :
- `scripts/test-fast.sh` (fmt, clippy `-D warnings`, nextest) sur `kesh_298` remise à zéro : **3730 exécutés, 3730
  réussis, 6 ignorés** (`298-final-test-fast.log`) ;
- frontend (`npm run check` 0 erreur / 29 avertissements préexistants, aucun dans les fichiers de la story ;
  `lint-i18n-ownership` PASS ; `test:unit` **148 fichiers, 1658 tests, tous verts** ; `build` vert)
  (`298-final-front.log`) ;
- **E2E complet** (`298-final-e2e.log`, `kesh_e2e_298` remise à zéro, port 3048, recette de `docs/testing.md`) : **289
  réussis, 7 échoués, 19 ignorés**. Les sept échecs, jugés fichier par fichier contre `docs/testing.md` § *Les échecs
  attendus* : `mode-expert.spec.ts:26`, `:41`, `onboarding-path-b.spec.ts:65`, `:92`, `onboarding.spec.ts:57`, `:77`,
  `:150` — **les sept KF-029 (#97)** ; ni KF-052 ni pollution sur ce run (baseline basse, 7). `reconciliation-link.spec.ts`
  vert, clair et sombre.

Historique des gates, tous déclarés : sur le HEAD **avant** le second rebase — test-fast 3717/3717 deux fois ; un
premier frontend rouge (`i18n-keys.test.ts`, `sitesTotal` 2164 → 2180), corrigé ; puis frontend 1658/1658 et E2E
289 réussis + les 7 KF-029 (`298-final-*-run2.log`) ; la recette visuelle de ce run a montré la modale trop étroite
(en-têtes collés), corrigée par le commit `fix(29-8)` ; un troisième run, sur ce correctif, a été **interrompu par PID**
pendant l'E2E à la demande de l'orchestrateur (`main` avait reçu la 29-4b) — ses journaux sont `*-run3-avant-rebase.log`,
non comptés. Les chiffres ci-dessus sont ceux du HEAD rebasé sur `12c64c39`, dont le code est celui du dernier commit de
code.

**Recette visuelle** : captures `29-8-modale-ecart-clair.png` et `29-8-modale-ecart-sombre.png` dans
`/home/gcorbaz/devel/kesh-gate-logs/298-captures/` (hors dépôt). Revues : modale lisible, six colonnes espacées, avertissement d'écart visible avant la confirmation. ⚠️ La capture
« sombre » est rendue en thème **clair** : `page.emulateMedia({ colorScheme: 'dark' })` ne bascule pas le thème de
l'application dans ce montage — même constat sur les captures sombres de la 29-6 (`29-6-ligne-cochee-sombre.png`) ; la
recette sombre réelle reste à faire à la main (aucun jeton de couleur neuf : `border-warning`, `bg-warning/10`,
`text-destructive`, déjà employés ailleurs).
*(Annoté le 2026-10-14, revue de code P1, A4 : le diagnostic est faux. **Kesh n'a pas de thème sombre** (C-28-1-7,
`frontend/src/lib/components/ui/button/button.svelte:18`) : la capture « sombre » rend le thème clair, comme attendu ;
la modale n'emploie aucune classe `dark:`. Il n'y a **aucune recette sombre manuelle** à faire.)*

**Issues à ouvrir** : aucune. **Actions demandées à l'orchestrateur** : push et PR de `story/29-8` après la revue, avec
**`closes #670`** dans le titre ou le corps de la PR seulement ; verser les choix C-29-8-30 à 36 au registre de l'epic
par union ; inscrire au contrôle de pré-release de la v0.13.4 « 0 conseil de rejet » (C-29-8-29), désormais tenu sur le
manuel, le CHANGELOG et `api-external.md`.

#### Remédiation de la revue de code P1 (2026-10-14/15, Opus 5.5)

Commits `38d5f3fc` (tests), `bc752e39` (code et textes), `d2ab107e` (spec E2E, manuel, API externe ; **dernier commit
de code**), puis ce compte rendu. Diff de la remédiation : `4b358373..HEAD`.

- **A1 / B4** — table des refus de `docs/api-external.md` réécrite dans l'ordre du code, relu au code : `403` (middleware)
  → `400` → `404` transaction → pré-vol (archivé, clos, `412`) → `404 NOT_PENDING` (fin du `GET`) ; au `POST`, `409
  ACCOUNT_LOCKED` → `404` transaction disparue → `404 NOT_PENDING` → `409 NOT_LINKABLE` (condition 1, avant le
  règlement) → `404` règlement → `409 BANK_ACCOUNT_CLOSED` (garde dans la transaction) → `409 NOT_LINKABLE` (conditions
  2 à 4) → `409 OPTIMISTIC_LOCK_CONFLICT` (défensif). Le doc-comment de `post_link_transaction`, qui plaçait le
  verrou en dernier et la condition 1 après le règlement, suit le même ordre.
- **E1 (relevé au-dessus de LOW par l'orchestrateur)** — **T17-bis** (`t17bis_le_second_rattachement_attend_le_verrou_du_reglement`,
  C-29-8-41) ; vert au gate ciblé (24/24) ; rejoué sous mutation (`298-mutations-p1.py`, motif unique, `git checkout --`
  puis `touch`, `298-mutations-p1-result.txt`) :

  | M | Verdict | Tuée par |
  |---|---|---|
  | M10 (`FOR UPDATE` de 4.2 (a) retiré) | tuée | **T17-bis** (« le POST n'a jamais attendu le verrou du règlement : rendu 200 … », `298-mut-p1-M10.log:56`), T16 |
  | M14 (4.4 avant 4.2) | tuée | **T17-bis** (`left: 200, right: 409`, `298-mut-p1-M14.log:36-38`), T11 |
  | M20 (au `GET`, statut lu avant le pré-vol) — A2 | tuée | T13 (a) (`404 RECONCILIATION_TRANSACTION_NOT_PENDING` au lieu de `409`) |
  | M21 (pré-vol : clos avant archivé) — A2 | tuée | T20 |

  La propriété que C-29-8-25 renvoyait à la revue (la vue du second rattachement naît après l'attente du verrou de
  `s`) est **désormais prouvée dynamiquement** par T17-bis ; la motivation de C-29-8-25 est annotée (le privilège
  `PROCESS` ne concerne qu'`INNODB_TRX`).
- **A2** — T13 (a) porte un témoin **rapproché** sur le compte clos (`409 BANK_ACCOUNT_CLOSED` au `GET` et au `POST`) ;
  T20 clôt aussi le compte archivé (`404` d'archivage maintenu).
- **B1 / E3 / B2** — garde de session de la modale (C-29-8-37) ; **B3** — relecture sur `404 NOT_FOUND` et `404
  NOT_PENDING` (C-29-8-38) ; **A7** — date de paiement rappelée (C-29-8-39). Huit tests Vitest neufs dans
  `LinkSettlementModal.test.ts` (9 → 17 exécutés), joués contre l'**ancienne** modale (`298-p1-vitest-red.log`) : **6
  échoués, 11 réussis (17)** — les deux relectures, la date, et trois des quatre tests de session ; le quatrième
  (« un chargement périmé ne remplace pas la liste ») passe sur l'ancienne modale, dont la garde de chargement existait :
  il est prouvé par la mutation MF1. Mutations (`298-mutations-p1-front.py`) : **MF1 à MF6, six tuées sur six** (MF1
  garde de chargement, MF2 refus périmé, MF3 succès périmé, MF4 remise à zéro de `submitting` par l'effet, MF5
  `RELIRE_SUR`, MF6 date). Non prouvés : la garde du `finally` et le `loadGen++` de la fermeture (C-29-8-37).
- **A3** — l'E2E asserte le compteur exact (`/^\s*1\s+opération\(s\) réussie\(s\)\.\s*$/`), l'`href` du lien
  (`/journal-entries/<écriture du règlement>`) et `matchedEntryId === journalEntryId`.
- **B5** — message `RECONCILIATION_SETTLEMENT_NOT_LINKABLE` élargi (virement sur ce compte bancaire, compte interne ou
  solde, ancien compte du grand livre) dans les quatre locales, le repli d'`errors.rs` (G9 vert), le doc-comment de la
  variante et la table des erreurs d'`api-external.md` ; code inchangé, aucune clé neuve (`sitesTotal` inchangé).
- **B6** — puce *Rejeter* du manuel (`user-manual.tex:2097`) et paragraphe de l'annulation de rejet d'`api-external.md`
  : le rattachement figure parmi les gestes offerts. Le paragraphe *Le geste* de `sec:rattacher-reglement` dit la date
  rappelée et la relecture de la liste. PDF régénéré (`make fr` sous le verrou, 2 passes stables, `298-p1-make-fr.log`),
  aplati : « éclatée ou rattachée à un règlement déjà enregistré » 1, « Kesh le dit et relit la liste » 1, « Rattacher
  une transaction à un règlement déjà enregistré » 2, « affectée manuellement ou éclatée » 0, « ?? » 0.
- **E2** — note ⁹ d'`api-external.md` : le rattachement parmi les gestes qui peuvent rendre `409
  RECONCILIATION_ACCOUNT_LOCKED`. **E4** — angle mort de l'archivage dans la fenêtre du pré-vol, déclaré (C-29-8-40),
  sans code. **A4**, **A6** — annotations datées ci-dessus ; **A5** — annotation de C-29-8-35. **B7** — déjà déclaré
  (C-29-8-31), rien.
- **Propagation** (valeurs grepées sur le dépôt) : ancien message dans les quatre locales — seule la fiche le garde (AC2,
  annoté) ; `manuellement ou éclatée` / `rapprochée manuellement ou` — les deux sites réécrits ; `annulation d'un autre
  rapprochement` — un seul site (`api-external.md:242`), complété ; l'ancien ordre de la table — aucune autre copie.

**Gates au dernier commit de code `d2ab107e`** (arbre propre au départ, `298-p1-gates-head.txt`), bases `kesh_298` et
`kesh_e2e_298` remises à zéro, sous le verrou :
- `scripts/test-fast.sh` : **3731 exécutés, 3731 réussis, 6 ignorés** (`298-p1-test-fast.log`) — +1, T17-bis ;
- frontend (`298-p1-front.log`) : `npm run check` 0 erreur, 29 avertissements (inchangés) ; `lint-i18n-ownership` PASS ;
  `test:unit` **148 fichiers, 1666 tests**, tous verts (+8) ; `build` vert ;
- **E2E complet** (`298-p1-e2e.log`, port 3048, recette de `docs/testing.md`) : **288 réussis, 8 échoués, 19 ignorés**.
  Jugés fichier par fichier contre `docs/testing.md` § *Les échecs attendus* : les **sept KF-029** (`mode-expert.spec.ts:26`,
  `:41`, `onboarding-path-b.spec.ts:65`, `:92`, `onboarding.spec.ts:57`, `:77`, `:150`) et
  `sidebar-navigation.spec.ts:75` — **KF-052 (#424) ou pollution**, non départagés (rejeu isolé non fait) ; 8 est dans
  la fourchette 7 à 10. `reconciliation-link.spec.ts` vert, clair et sombre.

### File List

- `crates/kesh-db/src/repositories/reconciliation_link.rs` (neuf)
- `crates/kesh-db/src/repositories/mod.rs`
- `crates/kesh-db/src/repositories/bank_accounts.rs` (doc-comments)
- `crates/kesh-db/src/repositories/bank_imports.rs` (doc-comment)
- `crates/kesh-db/src/repositories/invoices.rs` (doc-comments de tests)
- `crates/kesh-db/src/repositories/reconciliation_cancel.rs` (doc-comments)
- `crates/kesh-db/tests/bank_account_statement_gap.rs` (en-tête)
- `crates/kesh-db/tests/reconciliation_cancel.rs` (en-tête)
- `crates/kesh-api/src/routes/reconciliation.rs`
- `crates/kesh-api/src/lib.rs`
- `crates/kesh-api/src/errors.rs`
- `crates/kesh-api/src/audit_labels.rs`
- `crates/kesh-api/tests/reconciliation_link_e2e.rs` (neuf)
- `crates/kesh-api/tests/audit_route_registry.rs`
- `crates/kesh-api/tests/bank_account_closed_inventory.rs`
- `crates/kesh-api/tests/textes_coherents.rs`
- `crates/kesh-api/tests/reconciliation_e2e.rs` (commentaire)
- `crates/kesh-i18n/locales/{fr-CH,de-CH,en-CH,it-CH}/messages.ftl`
- `frontend/src/lib/features/reconciliation/LinkSettlementModal.svelte` (neuf)
- `frontend/src/lib/features/reconciliation/LinkSettlementModal.test.ts` (neuf)
- `frontend/src/lib/features/reconciliation/ReconciliationProposals.svelte`
- `frontend/src/lib/features/reconciliation/ReconciliationProposals.test.ts`
- `frontend/src/lib/features/reconciliation/reconciliation.api.ts`
- `frontend/src/lib/features/reconciliation/reconciliation.api.test.ts`
- `frontend/src/lib/features/reconciliation/reconciliation.types.ts`
- `frontend/src/lib/shared/i18n-keys.test.ts`
- `frontend/tests/e2e/reconciliation-link.spec.ts` (neuf)
- `docs/manual/fr/user-manual.tex`, `docs/manual/fr/user-manual.pdf`
- `docs/api-external.md`
- `CHANGELOG.md`
- `_bmad-output/implementation-artifacts/29-8-transaction-reglee-ailleurs.md`,
  `epic-29-choix-autonomes-29-8.md`, `29-8-review-prompt-p1.md` (neuf), `29-8-review-prompt-p2-ciblee.md` (neuf, remédiation P1), `sprint-status.yaml`,
  `_bmad-output/planning-artifacts/epic-29-preparer-cloture.md` (rebase)

## Change Log

- **2026-10-14** — Fiche créée (Opus 5.5) ; prémisses de #670 exactes ; voie (a) retenue, restreinte aux règlements
  clients par virement ; aucune migration ; C-29-8-1 à C-29-8-16.
- **2026-10-14** — **Validation P1** (Sonnet, contexte frais, lecture seule ; prompt `29-8-validate-prompt-p1.md`) :
  lentille R **0 CRITICAL / 0 HIGH / 4 MEDIUM / 7 LOW** (R1 à R11), lentille F **0 / 1 / 3 / 5** (F1 à F9) ; trois
  doublons (R2 = F4, R3 = F2, R9 = F8), soit 20 − 3 = 17 findings distincts. **Tous remédiés, aucun refusé** (remédiation Opus 5.5) :
  - **F1 (HIGH)** — étape 4.3 (`matched_entry_id = ? FOR UPDATE`) **retirée** : verrou d'intervalle, interblocage
    entre deux rattachements, redondante ; l'invariant « une écriture, une transaction » établi au code par la lecture
    sous le verrou de `s` et écrit à l'AC2 (C-29-8-17) ; ordre `s → je` en deux instructions (F6).
  - **R3 = F2** — `M > 0` (C-29-8-18), T18, M15 et M16. **F3** — T17 (course de double rattachement), M14 ; `GET_LOCK`
    ne fait pas naître la vue, écrit (raisonné, non exécuté). **R2 = F4** — la dévalidation ne lit pas le lien (motif 1
    `INVOICE_HAS_SETTLEMENTS`, avant comme après : préservation seulement) ; l'annulation depuis la fiche est refusée
    par le socle (`EntryNotReversable`, `409 MATCHED_BANK_TRANSACTION`), avec un témoin non rattaché qui s'annule.
  - **R1** — inventaire du manuel par symptôme (`:2911`, `:1198`, `:1305`, `:1308`, `:2029`, `:2076` et les sites
    inchangés), PDF aplati contrôlé (« ne la relie pas » et « tout seul. » y figurent une fois chacun sur `fcd2ae75`).
    **R4** — angle mort L6 (frais de rappel), #671 et #401 (C-29-8-23). **R5** — #671 et #672 cités ; L5 et les lots
    pain.001 versés à #671 (F7 : lots = L2 + L4, établi au code).
  - **LOW** : R6 (numéros de ligne : migration `:82-83`, manuel `:2076`, `:2124`, K2 cité par son code) ; R7 (deux
    doc-comments de tests `invoices.rs:4348-4349` et `:4805-4806`, inventaire AC7 à sept sites, motif élargi, hors-sujet
    triés) ; R8 (cinq assertions chiffrées d'`audit_route_registry.rs`, attendues 119/113/95/122/122 — routes
    mutantes seules —, point (vi) sept → huit, tri d'`ACTIONS`) ; R9 = F8 (`ensure_open_in_tx` dans la transaction,
    statut `Gardee`, zone `FENETRE` de la 29-4, C-29-8-19, T13 (b), M19) ; R10 (échelle et `rescale(2)`, tolérance en
    paramètre `Decimal`, date de paiement liée et non réécrite, exercice clos admis T19, compte archivé refusé et
    inactif admis T20 — C-29-8-20 à 22) ; R11 (K2 à T15, échec du chargement et rechargement sur 409 — C-29-8-24 —,
    puce d'ordre de l'epic remise en place) ; F5 (table *Absence de cycle* aux Dev Notes, cycle préexistant nommé) ;
    F9 (couples de T14, M8 et garde `IS NULL` déclarées équivalentes, `LIMIT` et `FOR UPDATE` de `je` déclarés).
  - Recomptés depuis la fiche après remédiation : 10 AC, 9 tâches (T0-T8), 20 tests d'intégration (T1-T20),
    19 mutations (M1-M19), 24 choix (C-29-8-17 à 24 ajoutés), 7 sites à l'AC7, 10 lignes au tableau du manuel.
  - Validation P2 à lancer : `29-8-validate-prompt-p2.md` (Opus, lentilles R et F).
- **2026-10-14** — **Validation P2** (Opus, contexte frais, lecture seule ; prompt `29-8-validate-prompt-p2.md`) :
  lentille R **0 CRITICAL / 0 HIGH / 5 MEDIUM / 5 LOW** (R1 à R10), lentille F **0 / 0 / 5 / 6** (F1 à F11), soit
  21 findings rapportés, **14 distincts** (R1 = F1 ; R2 = F2 + F3 ; R4 = F4 ; R6 = F9 ; R9 = F10 ; R10 ⊃ F7).
  - ⚠️ **Signal D5, déclaré au Project Lead** (règle de splitting préventif, amendement du 2026-10-06) : la sévérité
    la plus haute est **égale** de P1 à P2 sur la lentille R (MEDIUM → MEDIUM), et **quatre** défauts MEDIUM de la P2
    sont **nés de la remédiation P1**, tous sur le **plan de test** : l'observation `INNODB_TRX` de T17 (R1 = F1),
    M15 et M17 déclarées tuées à tort (R2.3-4 = F2), la ligne `:2911` décrite d'après une 29-6 périmée (R3), le
    témoin de T15 enchaîné après une clôture (R4 = F4, en partie). C'est le **recyclage** que vise D5 — le correctif
    précédent produit le défaut suivant —, non des défauts d'origine neufs ; mais il reste confiné au plan de test et
    aux textes, sans toucher la règle métier ni la conception des verrous (table refaite par F : aucun cycle). Pas de
    découpage proposé : la remédiation choisie par l'orchestrateur **retire** plutôt qu'elle n'ajoute, ce qui ôte au
    motif sa prise.
  - **Décisions de l'orchestrateur, consignées comme telles** (« retirer plutôt qu'ajouter ») :
    - **R1 = F1** — l'observation de l'attente est **retirée** de T16 et T17, sans forme de remplacement : T17 asserte le
      résultat d'une course réelle de deux rattachements (un `200`, un `409`, un seul lien), T16 celui d'une course
      annulation / rattachement ; ce qu'ils ne prouvent pas (ordre d'acquisition, attente du perdant, naissance de la
      vue) est écrit et renvoyé à la revue de code ; M10 et M14 ne sont tuées que si les requêtes se chevauchent
      (C-29-8-25). La correction proposée par R1 et F1 (sonde `attendre_une_requete_en_cours`) n'est **pas** retenue.
    - **R2 = F2 / F3** — M2 (contrainte `chk_invoice_settlements_counterparty`), M15 (jointure interne) et M17 (filtre
      société de l'étape 4.2 (a)) **déclarées équivalentes** avec leur raison, sans décor ; la phrase « les deux formes
      sont écartées par la même condition » réécrite (c'est la jointure qui écarte) ; M1 reste tuée par T6, décor à
      compte du grand livre partagé (une ligne de montage) ; F3 : angle mort **L7** écrit, sans test neuf (C-29-8-28).
      Ni la mutation de remplacement de M15 (R2), ni le décor « compte interne = compte de la banque » de T7 (F3).
    - **R4 = F4** — T15 sur un compte non principal qu'il **ne clôt pas**, ordre des assertions écrit, témoin par
      virement sur le même compte ; la clôture refusée puis réussie passe à **T1** (C-29-8-26).
  - **R3, R5, F5 — textes** : tableau du manuel refait d'après la fiche 29-6 telle qu'elle est (`3495aca1`) — FAQ
    `:2911` réécrite par la 29-6 (« ne relie pas la transaction … ; rejetez-la »), ligne `:2050` ajoutée, `:2029`
    complété, indication « ne peut pas être acceptée », aucun « 670 » au manuel ; glossaire `:3053` et tuile d'écart
    `:259-267` ajoutés ; motif d'inventaire élargi ; AC7 à **neuf** sites (`reconciliation_cancel.rs:47-48`,
    `bank_accounts.rs:1367-1370`), motif élargi, hors-sujet triés ; contrôles du PDF aplati **et normalisé** en
    `grep -oF`, positifs et négatifs, relevé sur `fcd2ae75` (« ne la relie pas » 1, « tout seul. » 1, « Kesh crée
    l'écriture, en réglant » 1, « date les écritures de rapprochement » 1, « rien ne la relie au relevé » 1,
    « existerait déjà » 1, « ne relie pas la transaction » 0, « rejetez-la » 0) ; entrée de CHANGELOG de la 29-6
    réécrite dans la même section.
  - **LOW** : R6 = F9 (prose chiffrée d'`audit_route_registry.rs` `:27`, `:31`, `:32`, `:659`, `:1007` ; « aucune
    preuve dynamique » dans le point (vi) ; `sites.len()` 31 → 32 de `bank_account_closed_inventory.rs:491` ;
    propagation sur la valeur) ; R7 (`api-external.md:730`, paragraphe des interblocages) ; R8 (message du code neuf
    couvrant la condition 1) ; R9 = F10 (le `GET` applique le pré-vol du `POST`, T13 (a), C-29-8-27) ; R10 (motif de
    la ligne `reconciliation_cancel` de la table *Absence de cycle* ; M6 unitaire seulement = F7) ; F6 (raison du
    `FOR UPDATE` de `je` réécrite : les gestes qui prennent `je` sans `s` existent et refusent par la propriété) ; F8
    (garantie d'échelle écrite, `routes/invoices.rs:1247`, à la route et non dans `kesh-db`) ; F11 (appel qualifié
    `super::bank_accounts::ensure_open_in_tx(`).
  - **Aucun finding refusé** ; trois corrections proposées par les lentilles écartées au profit des décisions
    ci-dessus (sonde de R1/F1, mutation de remplacement de M15, décor de T7).
  - Symptômes grepés après correction (fiche et registre) : `INNODB_TRX|LOCK WAIT|vu bloqué|observ` (reste T13 (b),
    sonde `GET_LOCK` légitime, et le Change Log P1 daté), `L1 à L6`, `sept sites`, `dix lignes`, `(#670)`,
    `par la condition M > 0`.
  - Recomptés depuis la fiche : 10 AC, 9 tâches (T0-T8), 20 tests d'intégration (T1-T20), 19 mutations (M1-M19, dont
    M2, M15, M17 équivalentes), 28 choix (C-29-8-25 à 28 ajoutés), 9 sites à l'AC7, 13 lignes au tableau du manuel,
    angles morts L1 à L7.
  - Validation P3 à lancer : `29-8-validate-prompt-p3.md` (Sonnet, lentilles R et F).
- **2026-10-14** — **Validation P3** (Sonnet, contexte frais, lecture seule ; prompt `29-8-validate-prompt-p3.md`) :
  lentille R **0 CRITICAL / 0 HIGH / 2 MEDIUM / 7 LOW** (R1 à R9), lentille F **0 / 0 / 0 / 7** (F1 à F7). Trend :
  P1 R 0/0/4/7, F 0/1/3/5 → P2 R 0/0/5/5, F 0/0/5/6 → **P3 R 0/0/2/7, F 0/0/0/7**. Recoupements : R4 = F1 (M15), R8 = F2
  (M8), R9 = F6 (ordre du `GET`) ; 16 findings rapportés, 13 distincts. **Tous remédiés, aucun refusé** (remédiation
  Opus 5.5, fiche et registre seulement) :
  - **Décisions de l'orchestrateur, consignées comme telles** — **R1** (MEDIUM) : l'inventaire des conseils de rejet se
    fait **par le symptôme, au rebase**, la fiche ne recopie plus les phrases des fiches voisines (C-29-8-29) ; la 29-5
    entre aux zones partagées ; le contrôle de pré-release de la v0.13.4 (fichier d'epic) doit rendre 0 conseil de rejet
    après la 29-8 — **à inscrire par l'orchestrateur au fichier d'epic**, que cette passe ne touche pas. **R2** (MEDIUM) :
    le contrôle négatif « tout seul. » est retiré ; la ponctuation de l'incise de `:259-267` est écrite.
  - **LOW** : R3 (positifs du PDF par site, formulations exactes, comptes des chaînes partagées) ; **R4 = F1** (M15 :
    citation rectifiée `repositories/bank_accounts.rs:1273-1288`, « aucun état atteignable » remplacé par « aucun état
    courant » et la borne réelle — l'équivalence tient sauf au centime —, M15 déclarée **survivante assumée**, annotation
    au registre) ; R5 (T1 porte les trois états préservés de l'AC3) ; R6 (AC7 à **douze** sites : `reconciliation_e2e.rs:4739`,
    `bank_account_statement_gap.rs:7`, `user-manual.tex:2129`) ; R7 (message de l'assertion `traced`, `:662-675`) ;
    **R8 = F2** (M8 déclarée équivalente dans la table ; annotation datée du décompte du Change Log P2 : « dont M2, M15, M17
    équivalentes » se lit **quatre mutations sans test tueur — M2, M8, M17 équivalentes, M15 survivante assumée** ; le
    relevé P2 n'est pas réécrit) ; **R9 = F6** (ordre des refus du `GET` identique à celui du `POST`, une fonction privée
    `link_preflight` commune, annotation de C-29-8-27) ; **F3** (`ensure_open_in_tx(tx, company_id, …)`) ; **F4** (T13 (b)
    est un état forgé : une clôture réelle refuserait, I1) ; **F5** (exigence « non principal » de T15 retirée, annotation
    de C-29-8-26) ; **F7** (`reconciliation_cancel.rs:48`).
  - Symptômes grepés après correction (fiche et registre) : `tout seul`, `47-48`, `mêmes fonctions`, `non principal`,
    `puis rejetez`, `aucun état atteignable`, `ensure_open_in_tx(`, `équivalente`.
  - Recomptés depuis la fiche et le registre après remédiation : 10 AC, 9 tâches (T0-T8), 20 tests d'intégration (T1-T20),
    19 mutations (M1-M19), 29 choix (C-29-8-29 ajouté), 12 sites à l'AC7, 13 lignes au tableau du manuel, angles morts
    L1 à L7.
  - ✅ **Validation close : la remédiation ne touche aucune ligne de production** (fiche et registre seulement ; le
    motif « la remédiation introduit le défaut suivant » n'a plus de prise, § *La passe ciblée* du `CLAUDE.md`). Prochaine
    étape : développement, **après la 29-6** (rebase sur les 29-2, 29-5 et 29-6).
- **2026-10-14** — **Développement** (`bmad-dev-story`, Opus 5.5, autonomie) : rebase sur `e889346c` (29-6 comprise) puis,
  à la demande de l'orchestrateur, sur `12c64c39` (29-4b comprise) ; prémisses relues, exactes (seuls les numéros de
  ligne ont bougé) ; T1 à T20 et Vitest vus rouges avant le code ; module `reconciliation_link`, deux routes, modale ;
  manuel FR, PDF, `api-external.md`, CHANGELOG ; sept conseils de rejet réécrits en « rattachez » ; 21 mutations jouées
  (15 tuées, 6 survivantes déclarées) ; choix C-29-8-30 à 36 ; gates au HEAD rebasé : test-fast 3730/3730, Vitest
  1658/1658, E2E 289 réussis + les 7 KF-029. Détail au Dev Agent Record. Revue de code P1 à lancer :
  `29-8-review-prompt-p1.md` (lentilles B, E, A).
- **2026-10-14/15** — **Revue de code P1** (prompt `29-8-review-prompt-p1.md`, trois lentilles, contexte frais,
  lecture seule) : lentille B **0 CRITICAL / 0 HIGH / 0 MEDIUM / 7 LOW** (B1 à B7), lentille E **0 / 0 / 0 / 4** (E1 à
  E4), lentille A **0 / 0 / 1 / 6** (A1 MEDIUM, A2 à A7). Recoupements : A1 ⊃ B4 (ordre des refus), B1 = E3 (résultat
  d'une autre transaction appliqué à la modale), B7 ≈ E4 (a) (déjà C-29-8-31) ; 18 findings rapportés (7 + 4 + 7), 15 distincts.
  **E1 relevé au-dessus de LOW par l'orchestrateur** (l'invariant « un règlement ne se rattache qu'une fois » n'a ni
  contrainte `UNIQUE` ni test qui rougisse). Remédiation Opus 5.5, **décisions de l'orchestrateur consignées comme
  telles** (C-29-8-37, 38, 41) ; **tous remédiés, aucun refusé** ; B7 déjà déclaré, rien :
  - **A1 / B4** — table des refus réécrite dans l'ordre réel du code, doc-comment de `post_link_transaction` aligné.
  - **E1** — T17-bis, déterministe (sonde `PROCESSLIST`) ; rouge sous M10 et sous M14, vert sinon ; motivation de
    C-29-8-25 annotée.
  - **B1 / E3 / B2** — garde de session de la modale, au chargement et à la soumission, quatre tests ; **B3** —
    relecture sur `404 NOT_FOUND` et `NOT_PENDING`, deux tests et un témoin ; **A7** — date de paiement rappelée, un
    test ; mutations MF1 à MF6 tuées.
  - **A2** — T13 (a) et T20 tiennent l'ordre du pré-vol ; M20 et M21 tuées. **A3** — l'E2E compare le compteur exact et
    l'écriture du règlement.
  - **LOW de texte** : B5 (message `NOT_LINKABLE`, quatre locales et repli) ; B6 (manuel `:2097`, PDF régénéré et
    contrôlé aplati ; API externe) ; E2 (note ⁹) ; E4 (C-29-8-40, sans code) ; A4, A5, A6 (annotations datées, les
    entrées passées ne sont pas réécrites).
  - Gates au dernier commit de code `d2ab107e` : test-fast 3731/3731 (6 ignorés), Vitest 1666/1666, E2E 288 réussis,
    8 échoués (les 7 KF-029 et `sidebar-navigation.spec.ts:75`, KF-052 ou pollution), 19 ignorés. Détail au Dev Agent
    Record, § *Remédiation de la revue de code P1*.
  - Choix C-29-8-37 à 41 ajoutés (41 au total). Passe suivante : **P2 ciblée**, une lentille sur `4b358373..HEAD`,
    prompt `29-8-review-prompt-p2-ciblee.md`.
