# Story 29.8 : Une transaction bancaire réglée depuis la fiche de la facture se rattache à ce règlement

## Status

**ready-for-dev** — fiche créée le 2026-10-14 en autonomie (`bmad-create-story`, Opus 5.5), sur `fcd2ae75` (kickoff de
l'Epic 29), branche `story/29-8`, worktree `/home/gcorbaz/devel/kesh-29-8`. Story **ajoutée à l'Epic 29** sur décision
de Guy (2026-10-14) ; sortie dans la **v0.13.4**. Choix consignés au registre propre à la story,
`epic-29-choix-autonomes-29-8.md`, **C-29-8-1 à C-29-8-16** (à verser par union au registre de l'epic). 10 AC.
Validation P1 à lancer (`29-8-validate-prompt-p1.md`, Sonnet, lentilles R et F). PR : **`closes #670`** — dans le titre
ou le corps de la PR **seulement**, jamais dans un message de commit, même en prose (B4 de la rétrospective 28).

⚠️ **Dernière de la série de la réconciliation** : elle touche `ReconciliationProposals.svelte` (comme l'AC10 de la
29-6), `routes/reconciliation.rs` (comme les 29-2, 29-3, 29-6) et réécrit des phrases du manuel que **la 29-2 et la
29-6 écrivent** (« … puis rejetez la transaction »). Le développement **commence par un rebase** sur `origin/main` une
fois la 29-6 mergée, et **relit tous les numéros de ligne** de cette fiche, relevés sur `fcd2ae75` (§ *Zones
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
| Elle apparaîtra au contrôle « transactions en attente » de la 29-1a | Fiche 29-1 (non mergée), K2 `PENDING_BANK_TRANSACTIONS` : `status = 'pending'` sur l'exercice, `rejectedCount` à part (`29-1-preparer-cloture.md:137`) | **Exacte** |
| Aucun geste ne la rapproche d'une écriture déjà passée | Les cinq chemins qui posent `matched_entry_id` (`routes/reconciliation.rs:2123`, `:2622`, `:3074`, `:3901`, `:4364`) lient chacun une écriture **qu'ils viennent de créer** ; le manuel le dit (« Kesh ne rapproche pas une transaction d'une écriture qui existerait déjà », `user-manual.tex:2021`) ; doc-comment `bank_accounts.rs:1379-1383` | **Exacte** |

**Ce qui existe déjà pour lier** (établi, C-29-8-2) : la colonne `bank_transactions.matched_entry_id` (FK `ON DELETE
SET NULL`, migration `20260504000001_bank_imports.sql:75`) et le statut `reconciled` (`CHECK status IN ('pending',
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
- **Restreint aux règlements clients par virement**, l'état produit est celui d'une proposition acceptée : toutes les
  gardes en aval le traitent déjà — l'annulation du règlement depuis la fiche est refusée au rang 3
  (`MatchedBankTransaction`, `settlement_cancellation.rs:132-157`), la dévalidation de la facture au motif 5
  (`invoices.rs:1605-1621`), la suppression de l'import (`is_reconciled`), et *Annuler le rapprochement* classe
  l'écriture en `InvoiceSettlement` et annule le règlement (`reconciliation_cancel.rs:69-95`).
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
   `bank_accounts.journal_account_id`), et `écart = t.amount − M` ; il faut **`0 ≤ écart ≤ 0,05`**, la tolérance
   étant `AMOUNT_TOLERANCE_HUNDREDTHS` (`routes/reconciliation.rs:72`), **sans nouvelle constante**.

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
lié a `M = 0` et n'est pas candidat — conservateur, écrit au manuel (angle mort L3).

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

- `amount` = `M`, `gap` = `t.amount − M`, deux décimales (même sérialisation que les montants de
  `GET /reconciliation/proposals`) ; `daysApart` = `|settledOn − date de paiement|`, la date de paiement étant
  `payment_date(t)` (`repositories/reconciliation.rs:113-115`, valeur, à défaut comptabilisation).
- Tri : `gap` croissant, puis `daysApart` croissant, puis `settlementId` croissant ; **50 au plus** — le filtre et le
  tri sont **dans la requête**, avant `LIMIT` (leçon C-29-6-3 : un filtre posé après coup laisse des lignes occuper le
  `LIMIT`).
- Transaction inexistante ou d'une autre société → `404 NOT_FOUND` (anti-énumération). Transaction rapprochée →
  `404 RECONCILIATION_TRANSACTION_NOT_PENDING` (code existant, `errors.rs:2783-2797`). Transaction de débit, nulle ou
  hors CHF → `200` et `candidates: []` (la condition 1 est fausse : rien n'est rattachable).
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
3. compte bancaire clos → `409 BANK_ACCOUNT_CLOSED` (`ensure_bank_account_open`, `:4928-4939`) ; sans compte du grand
   livre lié → `412 BANK_ACCOUNT_NOT_CONFIGURED` ;
4. **sous le verrou nommé du compte** (`with_account_lock`, comme `post_manual`, `:3798-…`), dans une transaction
   neuve, l'ordre des lectures est **imposé** (B5 de la rétrospective 28 — aucune naissance de vue avant le dernier
   verrou) :
   1. `bank_transactions` de `t`, `FOR UPDATE` → absente : `404 NOT_FOUND` ; rapprochée : `404
      RECONCILIATION_TRANSACTION_NOT_PENDING` ; condition 1 fausse : `409 RECONCILIATION_SETTLEMENT_NOT_LINKABLE` ;
   2. la ligne `invoice_settlements` **et** son écriture (`journal_entries`), jointes, `FOR UPDATE` — même ordre
      `s → je` que `cancel_settlement_in_tx` (`invoice_settlements_write.rs:846-891`) → absente ou d'une autre société :
      `404 NOT_FOUND` ;
   3. `SELECT id FROM bank_transactions WHERE company_id = ? AND matched_entry_id = ? … FOR UPDATE` — lecture
      **courante** de la condition 3 ;
   4. **puis seulement** les lectures non verrouillantes : la requête de l'AC1 restreinte à `s.id` (conditions 2 à 4)
      → absente : `409 RECONCILIATION_SETTLEMENT_NOT_LINKABLE` ;
5. `UPDATE bank_transactions SET status = 'reconciled', matched_entry_id = ?, auto_match_rejected_at = NULL,
   updated_at = NOW(3), version = version + 1 WHERE id = ? AND company_id = ? AND status = 'pending' AND
   matched_entry_id IS NULL` ; `rows_affected() != 1` → `409 OPTIMISTIC_LOCK_CONFLICT` (comme `post_manual`,
   `:3896-3925`) ;
6. audit `reconciliation.linked`, entité `bank_transaction`, **même transaction**, détails `bank_transaction_id`,
   `settlement_id`, `invoice_id`, `journal_entry_id`, `amount`, `settlement_amount` (= `M`), `gap`,
   `was_previously_rejected` (clés snake_case, comme `reconciliation.manual_matched`, `:3929-3940`) ; commit.

- **Code neuf** `409 RECONCILIATION_SETTLEMENT_NOT_LINKABLE`, `details.bankTransactionId` et `details.settlementId`,
  message fr-CH « Ce règlement ne peut pas être rattaché à cette transaction : il est sur un autre compte bancaire,
  son montant ne correspond pas, ou il est déjà rapproché. » — **un seul code** pour les conditions 1 à 4
  (C-29-8-10) : l'écran n'offre que des candidats, ce refus ne se voit que par une course ou un appel direct ;
  relire la liste. Repli d'`errors.rs` égal mot pour mot au catalogue fr-CH (garde G9 de `textes_coherents.rs`).
- **Rejeu** : la tentative est enveloppée par `retry_app_on_deadlock` (comme `post_manual`, `:3779-3782`) ; route
  classée `Traced, SansEcritureAuJournal` au registre `audit_route_registry.rs` (C-29-8-11).
- Le `UPDATE` est inscrit au `REGISTRE` de `bank_account_closed_inventory.rs` avec le statut « gardé » (garde de
  l'étape 3).
- **Lock-order**, à écrire au doc-comment de la fonction de `kesh-db` (C-29-8-12) : verrou nommé → `bank_transactions`
  (`t`) → `invoice_settlements` → `journal_entries` → `bank_transactions` (`matched_entry_id`). Le développement
  **relit** les ordres de `cancel_settlement_in_tx` (facture → règlement → écriture), de
  `reconciliation_cancel::cancel_in_tx`, d'`accept_one_invoice` et de `settle_invoice` et écrit au Dev Agent Record
  pourquoi aucun cycle ne se ferme — le rattachement ne prend **jamais** la ligne `invoices`.

**AC3 — Ce que le rattachement débloque, et ce qu'il laisse tel quel (C-29-8-13).**

Établi par test d'intégration (`crates/kesh-api/tests/reconciliation_link_e2e.rs`, binaire neuf) :

- après rattachement, `POST /bank-accounts/{id}/close` **réussit** sur un compte dont c'était la seule transaction en
  attente — **le défaut de #670** ; avant, il rend `412 BANK_ACCOUNT_HAS_PENDING_TRANSACTIONS` ;
- **aucune écriture** n'est créée : nombre de `journal_entries` et de `invoice_settlements` de la société inchangés ;
  le reste dû et `paid_at` de la facture inchangés ;
- `GET /reconciliation/transactions/{id}` rend `kind = "invoice_settlement"`, `invoiceId`, `cancellable = true` ;
- *Annuler le rapprochement* (`POST …/cancel`) **annule aussi le règlement** saisi depuis la fiche (contre-passation
  datée du jour, facture de nouveau à régler, transaction de nouveau `pending`) — comportement d'une proposition
  acceptée, **inchangé et voulu** ; écrit au manuel (AC6) et angle mort L1 ;
- l'annulation du règlement depuis la fiche est refusée `MATCHED_BANK_TRANSACTION` (rang 3), la dévalidation de la
  facture aussi — gardes existantes, assertées une fois chacune.

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
  `clearBatchReport` d'abord) ; erreur → message lisible (`errorMessageOf`), modal ouverte ;
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

`docs/manual/fr/user-manual.tex` (relire chaque ligne après rebase ; numéros relevés sur `fcd2ae75`) :

- **`:2021`** (intro de la réconciliation) : « Kesh ne rapproche pas une transaction d'une écriture qui existerait
  déjà » devient : Kesh **crée** l'écriture …, **sauf** dans un cas : une transaction dont le règlement a déjà été
  enregistré depuis la fiche de la facture se **rattache** à ce règlement, sans écriture (renvoi à la sous-section
  neuve).
- **Sous-section neuve** « Rattacher une transaction à un règlement déjà enregistré » (après *Réconciliation
  manuelle*, `\label{sec:rattacher-reglement}`) : quand (règlement saisi depuis la fiche ou la page des échéances,
  par virement, **sur ce compte bancaire**) ; le geste (bouton, choix, confirmation) ; les conditions (même compte,
  montant égal ou dépassé de 5 centimes au plus, règlement pas encore rapproché) ; l'écart affiché et à justifier ;
  aucune écriture ; une transaction rejetée : *Annuler le rejet* d'abord ; ce qu'on ne peut pas rattacher (L2 à L5).
- **Les phrases « … puis rejetez la transaction »** écrites par la 29-2 (C-29-2-8 : paragraphe de l'algorithme de
  matching, reliquat de plus de 30 jours) et par la 29-6 (AC7 : condition de montant, contournement du trop-perçu,
  puces *Rejeter* et « Aucune correspondance », réponse de la FAQ) : le geste prescrit devient « … puis **rattachez**
  la transaction à ce règlement (\S\ref{sec:rattacher-reglement}) ». Les incises « rejetée, elle reste en attente et
  empêche la clôture du compte (#670) » sont **retirées** là où elles ne décrivent plus que le contournement ; la
  description générale du rejet (`:2057`) garde « une transaction rejetée compte comme non rapprochée ».
  Inventaire par `grep -nE "rejet|670" docs/manual/fr/user-manual.tex` après rebase, chaque site trié au Dev Agent
  Record.
- **`:1689-1694`** (clôture du compte) : « … tant qu'elle n'est pas rapprochée manuellement ou que son import n'est
  pas supprimé » + « ou **rattachée** au règlement saisi depuis la fiche de la facture
  (\S\ref{sec:rattacher-reglement}) ».
- **`:2079`** (fin de *Réconciliation manuelle*, « enregistrez le règlement depuis la fiche de la facture ») : + « ,
  puis rattachez-y la transaction ».
- **§ *Annuler un rapprochement*** (`:2125-…`) : une transaction **rattachée** s'annule comme une proposition
  acceptée — **le règlement est retiré** aussi ; pour défaire seulement le rattachement, il n'y a pas d'autre geste
  (L1).
- PDF régénéré (`make fr` dans `docs/manual/`, sous le verrou de gate) et contrôlé aplati
  (`pdftotext … | tr '\n' ' ' | tr -s ' '`) : chaque phrase neuve trouvée, aucune occurrence restante de « puis
  rejetez » liée à un règlement saisi depuis la fiche.

`docs/api-external.md` : les deux routes (paragraphes à côté de `…/unreject`, `:594`), le code
`RECONCILIATION_SETTLEMENT_NOT_LINKABLE` au tableau des erreurs (`:719` voisin), la route à la note ⁹ du § 7 (refus
`BANK_ACCOUNT_CLOSED`, `:240`), et la phrase de `kind` (`:571`) : `invoice_settlement` désigne aussi une transaction
rattachée. `CHANGELOG.md`, section 0.13.4 (créée par les stories précédentes) : *Ajouté* — le rattachement (#670) ;
*Modifié* — le contournement de la 29-2 et de la 29-6 n'est plus « rejeter » mais « rattacher ».

**AC7 — L'inventaire clos des textes qui supposent « une écriture créée par le rapprochement » (C-29-8-16).**

Relevé par `grep -rnE "cinq chemins|viennent de créer|existerait déjà|une transaction, une écriture"` sur `crates`,
`docs/manual/fr`, `docs/api-external.md` (`fcd2ae75`) — **cinq** sites, chacun réécrit (le sixième chemin, ses
conditions) ou déclaré inchangé avec sa raison au Dev Agent Record :

| Site | Ce qu'il dit | À faire |
|---|---|---|
| `crates/kesh-db/src/repositories/bank_accounts.rs:1379-1383` | « Les cinq chemins … lient chacun une écriture qu'ils viennent de créer » | six chemins ; le rattachement lie une écriture **existante**, l'invariant « une écriture, une transaction » est tenu par la condition 3 |
| `crates/kesh-db/src/repositories/reconciliation_cancel.rs:11-18` | « Des cinq chemins … un seul crée un règlement client » | deux chemins rendent `InvoiceSettlement` : l'acceptation « facture » et le rattachement |
| `crates/kesh-db/src/repositories/bank_imports.rs:357` | « les cinq chemins qui rapprochent posent les deux colonnes ensemble » | six ; le rattachement aussi |
| `crates/kesh-db/tests/reconciliation_cancel.rs:3` | « Les cinq chemins réels » | ajouter le rattachement ou dire pourquoi le test n'en a pas besoin (couvert par AC3) |
| `docs/manual/fr/user-manual.tex:2021` | « Kesh ne rapproche pas … existerait déjà » | AC6 |

Après chaque patch, **greper la valeur** (`\bcinq\b`, `\b5\b` près de « chemin ») et non la phrase (§ *Propagation*).

**AC8 — Tests rouges d'abord, puis mutations (C-29-8-9).**

Intégration (`reconciliation_link_e2e.rs`, `#[sqlx::test(migrations = "../kesh-db/test-schema")]`), décor par l'API
(règlement par `POST /invoices/{id}/settlements`, transaction par import CAMT de fixture synthétique) :

| # | Cas | Attendu |
|---|---|---|
| T1 | règlement fiche = montant, même compte | 1 candidat `gap 0.00` ; `POST` 200 ; `reconciled`, `matched_entry_id` = écriture du règlement ; aucun `journal_entries` neuf ; audit `reconciliation.linked` aux détails de l'AC2 |
| T2 | transaction rejetée | candidate ; `POST` 200 ; `auto_match_rejected_at` à `NULL` ; `was_previously_rejected: true` |
| T3 | cas 29-6 : transaction 100.03, règlement 100.00 | candidat `gap 0.03` ; `POST` 200 ; audit `gap: "0.03"` |
| T4 | écart 0.05 / 0.06 | candidat / absent ; `POST` 409 `RECONCILIATION_SETTLEMENT_NOT_LINKABLE` pour 0.06 |
| T5 | règlement > transaction de 0.01 | absent ; 409 |
| T6 | règlement sur un autre compte bancaire | absent ; 409 |
| T7 | règlement `internal_account` | absent ; 409 |
| T8 | écriture déjà rapprochée (règlement né d'une acceptation ; ou rattaché à une autre transaction) | absent ; 409 ; la seconde transaction reste `pending` |
| T9 | transaction de débit, nulle (`0.00`), hors CHF | `candidates: []` ; `POST` 409 |
| T10 | transaction déjà rapprochée | `GET` et `POST` 404 `RECONCILIATION_TRANSACTION_NOT_PENDING` |
| T11 | transaction ou règlement d'une autre société | 404, rien d'écrit |
| T12 | rôles : Consultation 403 ; clé lecture seule refusée, clé `read-write` admise et nommée à l'audit | — |
| T13 | compte clos (clôture posée par SQL sur une transaction en attente — état de course) | 409 `BANK_ACCOUNT_CLOSED` |
| T14 | tri : trois candidats (écarts 0.02 / 0.00 / 0.00 à 3 et 40 jours) | ordre 0.00·3 j, 0.00·40 j, 0.02 |
| T15 | AC3 : clôture du compte refusée avant, réussie après ; `…/cancel` annule le règlement ; annulation fiche et dévalidation refusées | — |
| T16 | course : une connexion tient `invoice_settlements` `FOR UPDATE` puis supprime le règlement et commite ; le `POST` lancé pendant l'attente rend 404, la transaction reste `pending` | — |

Unitaires `kesh-db` : `transaction_can_be_linked` (crédit, nul, débit, devise ; `-0.00` traité comme nul).
Vitest : `LinkSettlementModal.test.ts` (chargement, liste vide, aucune présélection, *Rattacher* grisé puis actif,
avertissement d'écart présent/absent, succès → `onSuccess`, erreur affichée), `ReconciliationProposals.test.ts`
(bouton sur crédit CHF, absent sur débit), `reconciliation.api.test.ts` (URL, verbe, corps).

**Mutations à prévoir** (motif unique vérifié, restauration `git checkout -- f && touch f`) :

| M | Mutation | Tuée par |
|---|---|---|
| M1 | retirer `settlement_bank_account_id = ?` | T6 |
| M2 | retirer `settlement_type = 'bank_transfer'` | T7 |
| M3 | retirer le `NOT EXISTS` de la condition 3 | T8 |
| M4 | borne haute de l'écart `≤ 0,05` → `< 0,05` | T4 |
| M5 | borne basse `0 ≤ écart` retirée | T5 |
| M6 | `amount > 0` → `amount >= 0` dans `transaction_can_be_linked` | T9, unitaire |
| M7 | `auto_match_rejected_at = NULL` retiré de l'`UPDATE` | T2 |
| M8 | `AND status = 'pending'` retiré de l'`UPDATE` | T10 (sous verrou) ou déclarée équivalente avec raison |
| M9 | audit retiré | T1 |
| M10 | `FOR UPDATE` de l'étape 4.2 retiré | T16 |
| M11 | tri : `gap` retiré de l'`ORDER BY` | T14 |
| M12 | bouton affiché sur une ligne de débit | Vitest |
| M13 | présélection du premier candidat | Vitest |

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

- [ ] **T0 — Rebase** sur `origin/main` après merge de la 29-6 ; relire les numéros de ligne de la fiche ; base
  `kesh_298` et `kesh_e2e_298`, port 3048, `CARGO_TARGET_DIR=<worktree>/target`.
- [ ] **T1 — Tests rouges** (AC8) : `reconciliation_link_e2e.rs` T1 à T16, unitaires, Vitest — rouges constatés.
- [ ] **T2 — `kesh-db`** (AC1, AC2, AC7) : `repositories/reconciliation_link.rs` (`transaction_can_be_linked`, la
  requête des candidats, `link_in_tx` avec l'ordre des verrous et l'audit), `pub mod` ; doc-comments de l'AC7.
- [ ] **T3 — `kesh-api`** (AC1, AC2) : deux handlers dans `routes/reconciliation.rs`, routes dans `lib.rs` (groupe
  Comptable, à côté de `…/unreject`), `AppError::ReconciliationSettlementNotLinkable`, `audit_labels.rs`,
  `audit_route_registry.rs`, `bank_account_closed_inventory.rs`.
- [ ] **T4 — Frontend** (AC4) : `LinkSettlementModal.svelte`, bouton, API, types.
- [ ] **T5 — i18n** (AC5), quatre locales.
- [ ] **T6 — Manuel FR + PDF, `api-external.md`, CHANGELOG** (AC6) ; inventaire `rejet|670` trié.
- [ ] **T7 — Mutations** (AC8), table au Dev Agent Record.
- [ ] **T8 — E2E + captures** (AC9) ; gates complets (AC10).

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

**Restent bloquées, hors périmètre** — chacun un angle mort écrit, à couvrir par issue (rapport) :

- **L2** — une transaction de **débit** payée depuis la fiche d'une facture fournisseur (`supplier_invoices::pay_in_tx`,
  choix `BankTransfer`, `supplier_invoices.rs:750-781`) : même blocage que #670, côté fournisseurs ;
- **L3** — une transaction comptabilisée par une **écriture saisie au journal**, ou par un règlement passé sur un
  **ancien** compte du grand livre du compte bancaire ;
- **L4** — une transaction qui paie **plusieurs** factures réglées séparément depuis leurs fiches (une transaction,
  un lien : `matched_entry_id` est unique) ;
- **L5** — un règlement saisi sur le **mauvais compte bancaire** (le dialogue préremplit le compte principal,
  `SettleInvoiceDialog.svelte:120-127`) : pas candidat ; annuler le règlement et le refaire sur le bon compte.

**L1** — *Annuler le rapprochement* d'une transaction rattachée annule aussi le règlement : pas de geste « détacher
seulement ». Visible (contre-passation, facture rouverte), réparable (refaire le règlement, rattacher), sans erreur
comptable. Issue d'évolution demandée (rapport) — leçon § 4.3 de la rétrospective 28.

### Zones partagées avec les 29-2, 29-6 et 29-1 (fiches non mergées, lues en lecture seule)

| Zone | 29-2 | 29-6 | 29-8 |
|---|---|---|---|
| `ReconciliationProposals.svelte` | — | AC10 : case sur une ligne sans proposition, *Accepter* grisé, indication | troisième bouton de ligne, modal ; **ne touche pas** la case ni l'indication |
| Indication fr-CH de la 29-6 « Une transaction sans correspondance ne peut qu'être rejetée : décochez-la pour accepter les autres. » | — | AC10 | **inchangée** : elle parle des boutons de lot ; les boutons de ligne restent offerts (C-29-6-16) |
| Manuel : « puis rejetez la transaction » | C-29-2-8, `:314`, `:324` de sa fiche | AC7, `:281-316` de sa fiche | réécrit en « rattachez » (AC6) |
| `routes/reconciliation.rs` | fenêtre de dates | borne basse du reste dû | deux handlers neufs, aucune ligne de la proposition ni de l'acceptation |
| K2 (29-1a) | — | — | aucune ligne ; la transaction rattachée en sort par son statut |

### Propagation — le symptôme à greper après chaque patch

- `\bcinq\b` près de « chemin(s) », `existerait déjà`, `viennent de créer` (AC7) ;
- `rejet` et `670` dans le manuel, `api-external.md`, `CHANGELOG.md` (AC6) ;
- `RECONCILIATION_SETTLEMENT_NOT_LINKABLE` : `errors.rs`, `api-external.md`, quatre `.ftl` ;
- `reconciliation.linked` : `audit_labels.rs`, quatre `.ftl`, `api-external.md` (si l'audit y est listé), tests.

### Ce qui doit être préservé

- `GET /reconciliation/proposals` et `POST /reconciliation/accept` **inchangés** (aucune ligne).
- `classify_payment`, `settle_invoice` : inchangés.
- `reconciliation_cancel` : **code** inchangé (seul son doc-comment change, AC7).
- Aucune écriture au journal, aucun règlement, aucun lettrage par le rattachement.

### Exclusions écrites (périmètre minimal)

- Proposer automatiquement les rattachements dans la liste des propositions : non (C-29-8-7).
- Offrir le geste depuis le détail de l'import pour une transaction rejetée : non — *Annuler le rejet* d'abord ;
  l'API, elle, admet une rejetée (comme `/manual`).
- L1 à L5 : hors périmètre.

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

### Debug Log References

### Completion Notes List

### File List

## Change Log

- **2026-10-14** — Fiche créée (Opus 5.5) ; prémisses de #670 exactes ; voie (a) retenue, restreinte aux règlements
  clients par virement ; aucune migration ; C-29-8-1 à C-29-8-16.
