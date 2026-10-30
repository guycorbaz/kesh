# Story v017-3a : une transaction déjà comptabilisée se rattache à son écriture, et l'écran le signale

Status: ready-for-dev

*Fiche rédigée le 2026-10-30, en autonomie (Opus 5.5), worktree `/home/gcorbaz/devel/kesh-0173`, branche
`story/v017-3-rapprochement-attente` partie de `c5101b07` (tête de la candidate `v0.16.1-rc1`) ; la story se fusionne
dans la release **0.17.0** (première candidate `v0.17.0-rc1`). Sous-story de `v017-3-rapprochement-attente`, découpée
avec **v017-3b** (#687) — motif : C-v017-3-1. Registre : `v017-3-choix-autonomes.md` (C-v017-3-1 à C-v017-3-12,
C-v017-3-17). Clé courte : `v0173a`. Issue : **#671** (P2, bug), en `refs` seulement dans les commits ; la fermeture se
décide à la PR de release (C-v017-3-17). Les numéros de ligne cités sont ceux de `c5101b07`.*

**Aucune migration. Aucune version bumpée.** La contrainte du Project Lead du 2026-10-30
(`kesh-gate-logs/prompts/v017-donnees-a-garder.md`) s'applique : rien de ce que porte une base 0.16.x n'est réécrit ;
une transaction en attente à tort se règle par un geste de l'utilisateur.

## Story

En tant que Comptable qui rapproche ses relevés, et dont certains mouvements bancaires sont **déjà comptabilisés** —
facture fournisseur payée depuis sa fiche, écriture saisie au journal, règlement par compte interne sur le compte de la
banque —, je veux que Kesh **me signale** qu'une transaction en attente ressemble à une écriture déjà passée, et me
laisse la **rattacher** à cette écriture, afin qu'elle passe rapprochée sans seconde écriture, qu'elle cesse
d'empêcher la clôture du compte bancaire et de figurer au contrôle K2 de la préparation de clôture — et que je ne
l'affecte plus par erreur, ce qui compterait le mouvement deux fois.

## Origine

- **#671** (P2) : « Les transactions réglées hors réconciliation côté fournisseurs, ou par une écriture manuelle,
  restent elles aussi en attente pour toujours. » Ses compléments (2026-10-14, 2026-10-15) ajoutent les lots pain.001,
  le règlement sur le mauvais compte bancaire, les frais de rappel, le règlement client par compte interne.
- **Précision du Project Lead sur #671 (2026-10-23)** : « Kesh doit afficher un avertissement et demander à
  l'utilisateur quoi faire : la garder en attente, la rejeter, la rattacher à l'écriture existante, etc. — jamais
  l'affecter d'office […]. Le danger visé : affecter manuellement une transaction déjà comptabilisée passe une seconde
  écriture. En attendant, consigne d'usage : une telle transaction se rejette, elle ne s'affecte pas. »
- **Contrainte du Project Lead du 2026-10-30** : les données de la 0.16.1-rc1 se gardent ; aucune réécriture silencieuse
  par une migration ; une transaction en attente à tort se règle par un geste de l'utilisateur ou par une proposition
  qu'il accepte.
- Voisinage : **#670 / Story 29-8** (rattacher un crédit à un règlement client par virement — le seul geste qui lie une
  écriture existante, dont celle-ci est la généralisation) ; **#418 / Story 25-3-b** (annuler un rapprochement) ;
  **#526 / Story 28-5** (une transaction rejetée se retrouve et se dé-rejette) ; **#672** (ouverte : défaire le seul
  lien) ; **#687 / v017-3b** (un règlement fournisseur sous le centime n'égale aucune transaction).

## Ce que le code dit (relevé le 2026-10-30, `c5101b07`)

### Les prémisses de #671 — confirmées, une précisée

| Prémisse | Au code | Verdict |
|---|---|---|
| Un règlement fournisseur par virement depuis la fiche laisse la transaction en attente | `supplier_invoices::pay_in_tx_as` (`crates/kesh-db/src/repositories/supplier_invoices.rs:761-1018`) : écriture `D créanciers / C compte lié de la banque` (`:919-952`), `UPDATE supplier_invoices … status = 'paid'` (`:956-976`), audit `supplier_invoice.paid` ; **aucune** lecture ni écriture de `bank_transactions` | **Exacte** |
| Une écriture saisie au journal laisse la transaction en attente | Les chemins qui posent `matched_entry_id` lient une écriture **qu'ils viennent de créer**, sauf la 29-8 (`reconciliation_link.rs:16-23`) ; la 29-8 ne lit que `invoice_settlements` de type `bank_transfer` (`reconciliation_link.rs:157-168`) | **Exacte** |
| Elle bloque la clôture du compte bancaire et reste au contrôle K2 | Établi par la 29-8 (fiche 29-8, tableau des prémisses : `close_in_tx` compte `status = 'pending'`, rejetées comprises ; K2 `PENDING_BANK_TRANSACTIONS`) ; non modifié depuis | **Exacte** |
| Les lots pain.001 sont hors du rattachement | `payment_batches::confirm_batch` règle chaque facture par `pay_in_tx` (`payment_batches.rs:554-570`), une écriture **par facture** ; le fichier ne pose pas `BtchBookg` (`grep -rn BtchBookg crates/kesh-payment/src` : rien) — la banque débite souvent le lot en **une** ligne | **Exacte** ; *précisée* : un lot d'**une** facture produit une écriture égale au débit et devient rattachable ici ; un lot de **plusieurs** ne l'est pas (une transaction pointe une seule écriture — C-v017-3-10) |
| La proposition ne lit pas les factures déjà payées | `routes/reconciliation.rs:1209` : « ⛔ Rien ne lit ici les factures déjà payées : l'angle mort L1 (#671) » ; `repositories/reconciliation.rs:766-769` | **Exacte** |

### Ce sur quoi la conception s'appuie

- **La 29-8**, dont la 3a reprend la forme **sans la recopier** : `reconciliation_link.rs` — condition sur la
  transaction en fonction pure (`transaction_can_be_linked`, `:77-79`), conditions sur l'objet lié en **une** requête
  dont la liste et le geste lisent le même texte (`candidates_sql(one)`, `:150-175`), montant sommé dans une table
  dérivée **par écriture** en jointure interne sur le compte lié **actuel** (`:158-162`), `NOT EXISTS` sur
  `bank_transactions.matched_entry_id` (`:168`), verrous `t` puis objet par clé primaire puis
  `bank_accounts::ensure_open_in_tx` (`:329-381`), `UPDATE` gardé (`:402-416`), trace dans la même transaction
  (`:419-441`). Côté route : `link_preflight` (`routes/reconciliation.rs:6083-6100`), `with_account_lock` +
  `conclude_locked_attempt` + `retry_app_on_deadlock` (`:6228-6301`).
- **L'invariant « une écriture, une transaction »** : aucune contrainte `UNIQUE` sur `matched_entry_id` ; il est
  supposé par `reconciliation_cancel` (rang 3) et tenu par la condition « non rapprochée » lue sous le verrou de ligne
  de l'objet lié. Ici l'objet lié **est** l'écriture : c'est le verrou de ligne de `journal_entries` qui sérialise deux
  rattachements de la même écriture.
- **La contre-passation** : `journal_entries.reverses_entry_id` (`20260828000002_journal_entries_reversal.sql`,
  `UNIQUE`) ; une écriture contre-passée est désignée par une autre.
- **L'annulation d'un rapprochement** classe l'écriture liée — règlement client, règlement fournisseur, écriture propre
  — et défait son propriétaire (`reconciliation_cancel.rs:57-79`, `classify` `:141`). Elle ne change pas (C-v017-3-9).
- **La proposition** : `GET /reconciliation/proposals` (`routes/reconciliation.rs:964-1418`) rend par transaction
  `candidates` et `notice` (`:108-118`) ; le champ ajouté est additif.
- **Le solde de la tuile** lit les écritures rapprochées pour corriger les dates (`bank_accounts.rs:1751-1790`,
  `BALANCES_SQL`, `je.id IN (SELECT bt.matched_entry_id …)`) : un rattachement l'améliore, sans changement de code.

## Décisions (détail et alternatives au registre)

| # | Décision | Registre |
|---|---|---|
| D1 | Rattacher à une **écriture** du journal : routes neuves `GET …/entry-candidates` et `POST …/link-entry` ; la 29-8 inchangée | C-v017-3-2 **(Q1)** |
| D2 | Montant **exact**, signé, sur le compte du grand livre actuel du compte bancaire | C-v017-3-3 |
| D3 | Aucune fenêtre de dates ; la proximité trie | C-v017-3-4 **(Q3)** |
| D4 | Le domaine de la 29-8 (règlement client par virement sur ce compte) est exclu de la liste des écritures | C-v017-3-5 |
| D5 | Ni écriture contre-passée, ni contre-passation | C-v017-3-6 |
| D6 | `alreadyBooked` dans les propositions ; avertissement sur la ligne ; confirmation avant *Accepter*, *Affecter manuellement*, *Éclater* ; aucun refus serveur | C-v017-3-7 **(Q2)** |
| D7 | Un dialogue, deux sections ; bouton « Rattacher à une écriture existante » sur toute ligne CHF non nulle | C-v017-3-8 |
| D8 | L'annulation d'un rattachement suit le classement existant (contre-passe une écriture saisie, annule un règlement) | C-v017-3-9 **(Q4)** |
| D9 | Hors périmètre : une transaction pour plusieurs écritures, ancien compte du grand livre, autre compte bancaire, frais de rappel, virement entre deux comptes propres (second côté) ; issue demandée | C-v017-3-10 **(Q5)** |
| D10 | Données existantes intouchées ; une transaction rejetée se rattache après *Annuler le rejet*, ou directement par l'API | C-v017-3-11 |
| D11 | Trace `reconciliation.entry_linked` ; refus `409 RECONCILIATION_ENTRY_NOT_LINKABLE` | C-v017-3-12 |

### La règle (une seule définition)

Une écriture `e` est **rattachable** à une transaction `t` si et seulement si, tous ensemble :

1. **la transaction** est de la société, `pending` (rejetée ou non), en CHF, de montant **non nul**
   (`transaction_can_be_linked_to_entry`, fonction pure de `kesh-db`, à côté de `transaction_can_be_linked`) ;
2. **l'écriture** est de la même société ;
3. son **mouvement** `M` sur le compte du grand livre **actuel** du compte bancaire de `t` (`SUM(debit) − SUM(credit)`
   de ses lignes sur ce compte) est **égal** à `t.amount`, en décimal exact (un crédit de la banque appelle `M > 0`, un
   débit `M < 0`) ; une écriture sans ligne sur ce compte n'est pas candidate (jointure interne, patron
   `reconciliation_link.rs:158-162`) ;
4. elle n'est rapprochée d'**aucune** transaction (`NOT EXISTS bank_transactions.matched_entry_id = e.id`) ;
5. elle ne contre-passe rien (`e.reverses_entry_id IS NULL`) et n'est contre-passée par rien (`NOT EXISTS` d'une
   écriture dont `reverses_entry_id = e.id`) ;
6. elle n'est pas l'écriture d'un règlement client par virement **sur ce compte bancaire** (`NOT EXISTS
   invoice_settlements s WHERE s.journal_entry_id = e.id AND s.settlement_type = 'bank_transfer' AND
   s.settlement_bank_account_id = t.bank_account_id`) — celle-là relève de la 29-8.

Tri : jours d'écart croissants (`|e.entry_date − payment_date(t)|`, la date de paiement **liée en paramètre**, jamais
réécrite en SQL — patron `reconciliation_link.rs:147-149`), puis `e.id` croissant ; **50** au plus ; filtre et tri dans
la requête, **avant** `LIMIT`. Les conditions 2 à 6 sont **une** requête (`entry_candidates_sql(one)`), que la liste
lit entière et le geste restreinte à `e.id`. ⛔ `bank_transactions` n'entre jamais dans le `FROM` de la somme (le
produit multiplierait les lignes).

### Ce que le rattachement n'est pas

- **Pas un appariement automatique** : Kesh liste, l'utilisateur choisit et confirme ; rien n'est présélectionné ni
  rattaché d'office ; l'avertissement n'est qu'un avis (règle *Un appariement automatique propose, il ne crée jamais*).
- **Pas une écriture** : aucun `journal_entries::create_in_tx`, aucun règlement, aucun lettrage.
- **Pas un lot** : un geste unitaire ; le *Pattern batch* ne s'applique pas.

## Critères d'acceptation

### La règle et la lecture (`kesh-db`)

- **AC-1** — Module `crates/kesh-db/src/repositories/reconciliation_link.rs` étendu (ou sous-module voisin, à nommer au
  Dev Agent Record) : `transaction_can_be_linked_to_entry(&BankTransaction) -> bool` (CHF, `amount ≠ 0` — `-0.00`
  compris comme nul) ; `entry_candidates_sql(one: bool) -> String` (conditions 2 à 6, tri, `LIMIT`) ;
  `find_entry_candidates(executor, company_id, &t, bank_ledger_account_id) -> Vec<EntryCandidate>` (vide si la
  condition 1 est fausse) ; `EntryCandidate { journal_entry_id, entry_number, fiscal_year_id, entry_date, journal,
  description, amount, days_apart }`, `amount` normalisé par `Decimal::rescale(2)` (patron `:112-130`). Un test
  unitaire asserte que la requête restreinte est la requête de liste plus `AND e.id = ?` (patron `:495-501`), et
  qu'elle ne contient pas `FOR UPDATE`.
- **AC-2** — Tests `#[sqlx::test]` de `find_entry_candidates`, un par condition, chacun avec une écriture **qui
  passerait sans elle** (sinon le test est muet) :
  1. un débit de `-250.00` et une écriture saisie au journal `D charge 250 / C banque 250` → candidate, `amount =
     "-250.00"` ;
  2. un crédit de `+80.00` et une écriture `D banque 80 / C produit 80` → candidate ; la même écriture contre un débit
     de `-80.00` → absente (signe) ;
  3. montant différent d'un centime → absente ;
  4. écriture passée sur un **autre** compte du grand livre (ancien compte lié, ou autre banque) → absente ;
  5. écriture déjà rapprochée d'une autre transaction → absente ;
  6. écriture contre-passée → absente ; sa contre-passation → absente ;
  7. règlement client par virement sur ce compte bancaire, au montant exact → absent de cette liste (et présent de
     `find_link_candidates`, assertion croisée) ; règlement client par **compte interne** dont la contrepartie est le
     compte de la banque → présent ;
  8. règlement fournisseur par virement depuis la fiche (`pay_in_tx`) → présent ; lot pain.001 d'une facture confirmé
     → présent ; lot de **deux** factures contre un débit égal à leur somme → **aucune** candidate (aucune écriture
     n'égale la somme) — l'angle mort D9 asserté ;
  9. autre société → absente ;
  10. tri : trois candidates à 2, 30 et 2 jours → ordre par jours puis `id` ; 51 candidates → 50 rendues, la plus
      lointaine coupée ;
  11. transaction en EUR, ou de montant nul → liste vide sans requête (condition 1).
- **AC-3** — `link_entry_in_tx(tx, LinkEntryRequest) -> Result<Result<EntryLinked, LinkRefusal>, DbError>` (réutilise
  `LinkRefusal`), sous le verrou nommé du compte que tient l'appelant, sans `BEGIN` ni `COMMIT`, **dans cet ordre** :
  4.1 `bank_transactions` de `t` `FOR UPDATE` (`find_by_id_for_account_for_update`) → absente `NotFound`, rapprochée
  `NotPending`, condition 1 fausse `NotLinkable` ; 4.2 `SELECT id FROM journal_entries WHERE id = ? AND company_id = ?
  FOR UPDATE` → absente ou d'une autre société `NotFound` ; 4.3 `super::bank_accounts::ensure_open_in_tx(…)` — appel
  **qualifié** (l'ensemble `G` de `bank_account_closed_inventory.rs` s'extrait des appels qualifiés) —, première
  lecture ordinaire, **après** le dernier verrou ; 4.4 la requête de l'AC-1 restreinte à `e.id` → absente
  `NotLinkable` ; 5 `UPDATE bank_transactions SET status = 'reconciled', matched_entry_id = ?, auto_match_rejected_at =
  NULL, updated_at = NOW(3), version = version + 1 WHERE id = ? AND company_id = ? AND status = 'pending' AND
  matched_entry_id IS NULL` (`rows_affected() != 1` → `OptimisticLockConflict`) ; 6 trace
  `reconciliation.entry_linked`, entité `bank_transaction`, `NewAuditLogEntry::for_actor` (clé d'API nommée), détails
  `bank_transaction_id`, `journal_entry_id`, `entry_number`, `fiscal_year_id`, `amount` (deux décimales),
  `was_previously_rejected`. Doc-comment : ordre des verrous (`t` → `je`, deux verrous de ligne par clé primaire, rien
  d'autre), raison de l'absence de verrou sur `matched_entry_id` (verrou d'intervalle, interblocage — reprise de
  `:290-293`), et **qui sérialise** : le verrou de ligne de `je`. ⚠️ **À établir à l'étape 0** et à écrire au
  doc-comment, ligne lue citée : (a) que `journal_entries::reverse_in_tx` et `reverse_owned_in_tx` prennent la ligne
  de l'écriture `FOR UPDATE` **avant** de lire `bank_transactions` pour le motif `MatchedBankTransaction` (sinon une
  contre-passation et un rattachement concurrents de la même écriture se croisent : rattachée **et** contre-passée) ;
  (b) qu'aucun geste ne tient une ligne `journal_entries` puis demande une ligne `bank_transactions` `FOR UPDATE`
  (cycle avec `t → je`). Si (a) ou (b) est faux, le dire au Dev Agent Record et **s'arrêter** : la conception est à
  revoir, pas à contourner.
- **AC-4** — Tests `#[sqlx::test]` de `link_entry_in_tx` : nominal (état final `reconciled`, `matched_entry_id`, rejet
  remis à `NULL`, une trace, ses détails) ; chaque refus sans écriture (transaction absente, d'une autre société,
  rapprochée, en EUR ; écriture absente, d'une autre société, non candidate ; compte bancaire clos **dans** la
  transaction) ; une transaction **rejetée** se rattache (`was_previously_rejected: true`) ; deux rattachements
  successifs de la même écriture à deux transactions du même compte → le second `NotLinkable`, rien d'écrit. *Le test
  de concurrence vraie (deux connexions) suit le patron de la 29-8 s'il existe — le citer au Dev Agent Record ; sinon,
  écrire que l'invariant est tenu par raisonnement (AC-3), comme la 29-8 (C-29-8-25).*

### Les routes (`kesh-api`)

- **AC-5** — `GET /api/v1/reconciliation/transactions/{id}/entry-candidates`, rôle Comptable (groupe de
  `link-candidates`, `lib.rs:857-858`) ; réponse `200` `{ bankTransactionId, candidates: [{ journalEntryId,
  entryNumber, fiscalYearId, entryDate, journal, description, amount, daysApart }] }`, `amount` chaîne à deux
  décimales. Ordre des refus **identique** au `POST` et à la 29-8 : `404 NOT_FOUND` (inexistante, autre société) →
  `link_preflight` (**réutilisé**, non recopié : archivé `BankAccountNotFound`, clos `409 BANK_ACCOUNT_CLOSED`, non lié
  `412 BANK_ACCOUNT_NOT_CONFIGURED`) → `404 RECONCILIATION_TRANSACTION_NOT_PENDING`. Transaction nulle ou hors CHF →
  `200`, `candidates: []`.
- **AC-6** — `POST /api/v1/reconciliation/transactions/{id}/link-entry`, corps `{ "journalEntryId": 211 }`, rôle
  Comptable, clés `read-write` admises (comme `link`) ; réponse `200` `{ bankTransactionId, journalEntryId }`. Refus,
  chacun sans écriture : `400 VALIDATION_ERROR` (absent ou `≤ 0`) → `404 NOT_FOUND` → `link_preflight` → `409
  RECONCILIATION_ACCOUNT_LOCKED` → sous le verrou, ceux de l'AC-3 : `404 NOT_FOUND`, `404
  RECONCILIATION_TRANSACTION_NOT_PENDING`, `409 RECONCILIATION_ENTRY_NOT_LINKABLE` (condition 1), `404 NOT_FOUND`
  (écriture), `409 BANK_ACCOUNT_CLOSED`, `409 RECONCILIATION_ENTRY_NOT_LINKABLE` (conditions 2 à 6), `409
  OPTIMISTIC_LOCK_CONFLICT`. Rejeu `retry_app_on_deadlock("reconciliation::link_entry", …)`. Une fonction privée
  `link_entry_once` sur le patron de `link_once` (`:6264-6301`) — si les deux ne diffèrent que par l'appel intérieur et
  la réponse, factoriser (DRY) et le dire.
- **AC-7** — `AppError::ReconciliationEntryNotLinkable { bank_transaction_id, journal_entry_id }` : `409`, code
  `RECONCILIATION_ENTRY_NOT_LINKABLE`, `details.bankTransactionId`, `details.journalEntryId`, message fr-CH « Cette
  écriture ne peut pas être rattachée à cette transaction : son mouvement sur le compte de la banque n'égale pas le
  montant de la transaction, ou elle est déjà rapprochée, contre-passée, ou c'est un règlement de facture client à
  rattacher comme tel. » ; repli d'`errors.rs` égal mot pour mot au catalogue fr-CH (garde G9 de
  `textes_coherents.rs`) ; quatre locales.
- **AC-8** — Tests HTTP (`crates/kesh-api/tests/reconciliation_link_entry_e2e.rs`, nouveau) : nominal par la liste puis
  le geste, pour (a) une écriture saisie par `POST /journal-entries` et (b) une facture fournisseur payée par `POST
  /supplier-invoices/{id}/pay` ; chaque refus de l'AC-6 avec son code ; RBAC (Consultation `403` ; clé `read` `403
  API_KEY_READ_ONLY` ; clé `read-write` `200` et trace qui nomme la clé) ; IDOR (transaction et écriture d'une autre
  société → `404`) ; puis les **suites** du rattachement, chacune assertée :
  1. la transaction ne compte plus au refus de clôture du compte bancaire (`POST /bank-accounts/{id}/close` passe quand
     c'était la dernière en attente) ;
  2. elle sort du contrôle K2 de la préparation de clôture (une route existante de la préparation, à citer) ;
  3. *Annuler le rapprochement* (`POST /reconciliation/transactions/{id}/cancel`) : pour (a), l'écriture saisie est
     **contre-passée** et la transaction redevient `pending` ; pour (b), le **règlement fournisseur est annulé** et la
     facture redevient `open` (C-v017-3-9 — le comportement existant, asserté pour qu'il ne change pas en silence) ;
  4. la contre-passation directe de l'écriture rattachée (`POST /journal-entries/{id}/reverse`) est refusée `409
     MATCHED_BANK_TRANSACTION` ; l'annulation du règlement fournisseur depuis la fiche, de même (le code existant de
     ce refus, à citer).
- **AC-9** — Registres : `audit_route_registry.rs` classe `POST …/link-entry` `Traced, SansEcritureAuJournal`, rejouée
  (point (vi) du doc-comment, « sans preuve dynamique » comme `link`) ; les assertions chiffrées et la prose chiffrée du
  fichier se **recomptent depuis le registre** au moment du patch (à `c5101b07` : `traced, 129` `:915`,
  `sans_ecriture, 110` `:1312`, `tout.len(), 138` `:1324` — valeurs de départ, non d'arrivée) ;
  `bank_account_closed_inventory.rs` inscrit l'`UPDATE` de `link_entry_in_tx` au `REGISTRE` en `Statut::Gardee(…)`
  (forme de `link_in_tx`, `:237`), l'ajoute à `FENETRE` (`:892`) et recompte `assert_eq!(sites.len(), 33, …)` (`:519`)
  et son commentaire (`:516`) ; `audit_labels.rs` gagne `reconciliation.entry_linked`, **trié** (après
  `reconciliation.cancelled`, avant `reconciliation.linked`), libellé fr-CH « Transaction rattachée à une écriture » et
  quatre locales ; `tests/audit_label_registry.rs` vert. Les doc-comments qui comptent les chemins de
  `matched_entry_id` passent d'un chemin de plus et nomment celui-ci : `reconciliation_link.rs:16-23`,
  `reconciliation_cancel.rs:11-23` (« Des sept chemins »), et ce que rend `grep -rn "chemins qui posent\|sept chemins\|cinq autres chemins" crates/` (tri au Dev Agent Record).

### La proposition (`GET /reconciliation/proposals`)

- **AC-10** — `ReconciliationProposal` gagne `already_booked: Option<AlreadyBooked>`, `AlreadyBooked { settlement_count:
  usize, entry_count: usize }`, `null` si les deux sont nuls. Les deux comptes viennent de **`find_link_candidates`** et
  de **`find_entry_candidates`**, appelées pour chaque transaction de la page, sur le compte du grand livre lu au
  pré-vol de la page (aucune seconde règle, aucun `COUNT` à part) ; un compte bancaire sans compte du grand livre lié
  → `null` partout. Champ additif ; `candidates`, `notice` et l'ordre des propositions inchangés. Tests HTTP : une
  transaction qui a une écriture candidate porte `alreadyBooked.entryCount = 1` **et** garde ses propositions ; une
  qui n'en a pas porte `null` ; un crédit qui a un règlement client candidat porte `settlementCount = 1`.
- **AC-11** — Coût : mesurer, sur une base de démonstration, `GET /reconciliation/proposals?limit=500` avant et après
  (trois mesures chacun), et l'écrire au Dev Agent Record ; si l'écart dépasse une seconde, le dire et demander
  l'issue (*Performance debt*), sans optimiser par une seconde règle.

### L'écran (`frontend`)

- **AC-12** — `ReconciliationProposals.svelte` : sur une ligne dont `alreadyBooked` n'est pas `null`, un avertissement
  (`data-testid="already-booked-warning"`) : « Cette transaction semble déjà comptabilisée : {count} écriture(s) du même
  montant, non rapprochée(s), sur le compte de la banque. Rattachez-la plutôt que de l'affecter : l'affecter passerait
  une seconde écriture. », avec le bouton du dialogue. Le texte ne cite aucun numéro d'issue.
- **AC-13** — Le bouton « Rattacher à un règlement » devient **« Rattacher à une écriture existante »** (clé
  `reconciliation-link-button-label` récrite dans les quatre catalogues, `data-testid="link-settlement-button"` gardé),
  offert sur toute ligne en CHF de montant non nul (`canLink` récrit : miroir de `transaction_can_be_linked_to_entry`,
  et son commentaire). Le dialogue (`LinkSettlementModal.svelte`, généralisé, renommé s'il le faut — le dire) montre
  deux sections : « Règlements de factures clients » (crédits seulement ; la liste, les colonnes et l'écart de la 29-8
  inchangés) et « Écritures passées sur le compte de la banque » (n°, date, journal, libellé, montant, jours d'écart).
  **Un seul** choix parmi les deux, rien de présélectionné ; *Rattacher* appelle `link` ou `link-entry` selon la
  section. Le dialogue rappelle, quand une écriture est choisie : « Annuler plus tard ce rapprochement contre-passera
  cette écriture (ou annulera le règlement qu'elle porte). » Deux listes vides : le message existant de la 29-8, récrit
  pour les deux. `reconciliation.api.ts` gagne `getEntryCandidates` et `linkEntry` ; `reconciliation.types.ts` les
  types et `alreadyBooked`.
- **AC-14** — Confirmation : *Accepter*, *Affecter manuellement* et *Éclater*, quand la sélection (ou la ligne)
  contient une transaction dont `alreadyBooked` n'est pas `null`, ouvrent d'abord une confirmation
  (`data-testid="already-booked-confirm"`) qui nomme ces transactions (date, montant, contrepartie) et offre
  *Continuer quand même* / *Annuler* ; *Rejeter* n'en demande pas. Vitest : sans transaction signalée, aucune
  confirmation ; avec, la confirmation, puis l'appel seulement après *Continuer quand même*.
- **AC-15** — i18n : toutes les clés nouvelles ou récrites dans les **quatre** catalogues
  (`crates/kesh-i18n/locales/{fr,de,it,en}-CH/messages.ftl`), aucun texte en dur ; `npm run lint-i18n-ownership` vert.
- **AC-16** — Recette visuelle (thème clair, base de démonstration) : la ligne avec l'avertissement ; le dialogue avec
  ses deux sections (un crédit qui a un règlement client et une écriture) ; le dialogue d'un débit (une seule section) ;
  la confirmation ; la ligne disparue après rattachement. Captures dans
  `/home/gcorbaz/devel/kesh-gate-logs/v0173a/captures/`, listées au Dev Agent Record et jointes au prompt de revue.

### Le parcours (Playwright)

- **AC-17** — `frontend/tests/e2e/reconciliation-link-entry.spec.ts` : un Comptable (montants **uniques** par
  exécution, base partagée entre specs — patron de `reconciliation-amount-due.spec.ts`) saisit par l'API une écriture
  `D charge / C banque` d'un montant unique, importe un relevé dont un débit l'égale (aide d'import des specs
  existantes, `frontend/tests/e2e/helpers/bank-import-fixtures.ts` — à citer), ouvre `/reconciliation`, voit
  l'avertissement, coche la ligne, clique *Affecter manuellement* et voit la confirmation, l'annule, ouvre le dialogue,
  choisit l'écriture, *Rattacher* ; la ligne disparaît ; `/audit-log` filtré sur « Transaction rattachée à une
  écriture » la retrouve.

### Les textes

- **AC-18** — Manuel utilisateur FR (`docs/manual/fr/user-manual.tex`, PDF régénéré) : la sous-section
  `\label{sec:rattacher-reglement}` (`:2384-2412`) devient *Rattacher une transaction à une écriture déjà passée* (label
  gardé, ou renommé avec chaque `\ref` suivi : `grep -rn "rattacher-reglement" docs`) : quand (les chemins couverts),
  l'avertissement et pourquoi ne pas affecter, le geste et les deux sections, la règle (montant exact, compte du grand
  livre actuel, non rapprochée, non contre-passée), aucune écriture, **défaire** (contre-passe l'écriture saisie,
  annule le règlement — #672), une transaction **rejetée** (*Annuler le rejet* d'abord ; c'est le cas des transactions
  rejetées selon la consigne d'attente), et la liste *Ce qui ne se rattache pas* **récrite** selon D9 (plus le
  règlement sous le centime : renvoi à la v017-3b, `sec:…` de sa fiche). Les phrases devenues fausses se récrivent :
  `:2354`, `:2366`, `:2300` (« Rattacher à un règlement »), `:2408-2409` (débit, écriture saisie au journal), `:3337` —
  et tout ce que rend `grep -n "Rattacher à un règlement\|ne se rattache pas\|Kesh ne rapproche pas une transaction d'une écriture" docs/manual/fr/user-manual.tex`.
  Les manuels DE/IT/EN ne sont pas traduits dans cette version.
- **AC-19** — `docs/api-external.md` : les deux routes (après `:675-677`), le champ `alreadyBooked` des propositions,
  le code `RECONCILIATION_ENTRY_NOT_LINKABLE` au tableau des codes (après `:868`), la route ajoutée à la liste des
  routes rejouées (`:878`), « *(Depuis la v0.17.0.)* ».
- **AC-20** — `CHANGELOG.md` : sous `## [0.17.0]`, section **Corrigé**, une entrée (`refs` [#671] par un lien, sans
  mot-clé de fermeture) qui dit le rattachement à une écriture, l'avertissement, et ce qui reste (D9). Aucune version
  touchée.

### Les données existantes (contrainte du 2026-10-30)

- **AC-21** — Aucun fichier sous `crates/kesh-db/migrations/` n'est ajouté ni modifié (`git diff --stat c5101b07 --
  crates/kesh-db/migrations` vide) ; aucune instruction `UPDATE`/`INSERT`/`DELETE` nouvelle hors de `link_entry_in_tx`
  et de sa trace (`git diff c5101b07 -- crates/*/src | grep -nE '^\+.*\b(UPDATE|INSERT|DELETE)\b'`, tri au Dev Agent
  Record).
- **AC-22** — Fixture « base 0.16.x » : une transaction **rejetée** (`auto_match_rejected_at` posé, `pending`) et une
  facture fournisseur payée depuis la fiche, créées par les fonctions de dépôt de la 0.16 (non par SQL ad hoc) ; après
  chargement de la page de propositions et de la liste des candidates, **aucune** ligne n'a changé (état relu à
  l'identique) ; le rattachement ne se fait que par le `POST` (après dé-rejet, ou direct).

## Tâches

1. **Étape 0** — recontrôler chaque ligne du § *À recontrôler à l'étape 0*, le noter au Change Log, ligne lue citée ;
   établir (a) et (b) de l'AC-3. Bases `kesh_v0173a` et `kesh_e2e_v0173a` (`docs/agents-orchestration.md` § 4).
2. **Tests rouges d'abord** (§ *Tests et mutations*), commités rouges avec un message qui le dit.
3. `kesh-db` : règle, lecture, geste (AC-1 à AC-4).
4. `kesh-api` : erreur, routes, montage, proposition, registres (AC-5 à AC-11).
5. `frontend` : types, API, dialogue, avertissement, confirmation, i18n (AC-12 à AC-15) ; Vitest.
6. E2E (AC-17) ; captures (AC-16).
7. Textes (AC-18 à AC-20) ; PDF régénéré (`make fr` dans `docs/manual/`, sous verrou et `mem-guard`) ; **greper la
   valeur** de chaque phrase corrigée.
8. Gates, mutations, Dev Agent Record.

## Tests et mutations prévus

**Rouges d'abord** : AC-2 (au moins 1, 2, 6, 7, 8), AC-4 nominal et rejetée, AC-8 nominal (a) et (b), AC-10, Vitest de
l'AC-14. Un test qui ne compile pas parce que la fonction n'existe pas compte comme rouge si l'erreur la nomme.

| # | Mutation (une à la fois, motif vérifié unique, restauration `git checkout -- f && touch f`) | Doit rougir |
|---|---|---|
| M1 | condition 3 : `m.movement = ?` → `ABS(m.movement) = ABS(?)` | AC-2.2 (signe) |
| M2 | condition 3 : jointure interne → `LEFT JOIN … COALESCE(…, 0)` | AC-2.4 (autre compte) |
| M3 | condition 4 retirée | AC-2.5, AC-4 (second rattachement) |
| M4 | condition 5 : le `NOT EXISTS` de la contre-passée retiré | AC-2.6 |
| M5 | condition 5 : `reverses_entry_id IS NULL` retiré | AC-2.6 |
| M6 | condition 6 retirée | AC-2.7 (règlement client par virement absent) |
| M7 | condition 6 : `settlement_type = 'bank_transfer'` retiré | AC-2.7 (compte interne présent) |
| M8 | `transaction_can_be_linked_to_entry` : `amount != 0` → `amount > 0` | AC-2.1, AC-8 |
| M9 | tri : `ORDER BY e.id` seul | AC-2.10 |
| M10 | 4.3 déplacé avant 4.2 (lecture ordinaire avant le verrou de l'écriture) | revue (C-29-8-25) — **survivante assumée** si aucun test de concurrence n'existe ; le dire |
| M11 | `UPDATE` sans `auto_match_rejected_at = NULL` | AC-4 (rejetée) |
| M12 | trace : action `reconciliation.linked` | AC-4, AC-9 |
| M13 | proposition : `entry_count` toujours 0 | AC-10 |
| M14 | écran : `canLink` réservé aux crédits | Vitest AC-13 |
| M15 | écran : confirmation sautée pour *Affecter manuellement* | Vitest AC-14 |
| M16 | écran : *Rattacher* appelle `link` pour une écriture | Vitest AC-13 |

Chaque mutation : le test qui rougit et la raison, au Dev Agent Record. Une mutation **survivante** est un défaut de test
à corriger, sauf M10 si elle est déclarée comme la M14 de la 29-8.

## Gates

`kesh-db` est touché (dépôt) : **gate complet même en cours de boucle** (`CLAUDE.md` § *Exception kesh-db*). Base de
gate remise à zéro avant chaque gate complet. Au **dernier commit de code** : `scripts/test-fast.sh --ci` sous
`flock /home/gcorbaz/devel/kesh-gate-logs/.gate.lock`, `cargo build --workspace --all-targets`, Vitest complet, `npm run
check`, `npm run lint-i18n-ownership`, `npm run build`, et la **suite E2E complète** selon `docs/testing.md` à la lettre
(dossiers propres au worktree passés au backend **et** au runner), échecs jugés fichier par fichier contre `docs/testing.md`
§ *Les échecs attendus*. Ne déclarer que ce qui a tourné, chiffres recomptés.

## Ce que les gardes ne voient pas (angles morts assumés)

- **L1 — Une transaction pour plusieurs écritures** : lot pain.001 de plusieurs factures débité en une ligne, paiement
  groupé de factures réglées séparément. Aucune écriture n'égale la somme ; la transaction reste en attente. Issue
  demandée (C-v017-3-10).
- **L2 — Virement entre deux comptes bancaires propres** : une écriture `D banque B / C banque A` correspond à **deux**
  transactions (une par compte). L'invariant « une écriture, une transaction » ne laisse rattacher qu'**un** côté ;
  l'autre reste en attente. Dans l'issue de L1 **(Q5)**.
- **L3 — Ancien compte du grand livre, autre compte bancaire** : un mouvement passé sur un compte du grand livre qui
  n'est plus celui du compte bancaire, ou sur un autre compte bancaire, n'a pas de mouvement sur le compte lié actuel.
- **L4 — Frais de rappel** (#401) : le paiement dépasse le règlement ; ni la 29-8 (au-delà de 5 centimes) ni la 3a
  (montant exact) ne le prennent.
- **L5 — Règlement fournisseur sous le centime** (#687) : son mouvement n'égale jamais une transaction ; la v017-3b
  ferme la source et décrit le traitement de l'existant.
- **L6 — Faux avertissement** : un mouvement récurrent de même montant (loyer) dont l'écriture d'un mois précédent
  n'est pas rapprochée fait signaler la transaction du mois courant. L'avertissement est un avis ; la date de chaque
  candidate s'affiche.
- **L7 — Écriture d'ouverture** : si le montant d'une transaction égale le solde d'ouverture du compte de la banque,
  l'écriture d'ouverture est candidate. Elle s'affiche avec son libellé ; rien n'est présélectionné.
- **L8 — Rejetées** : l'avertissement ne se calcule que pour les transactions de la liste (non rejetées) ; une
  transaction rejetée selon la consigne d'attente n'est signalée qu'après *Annuler le rejet*.

## À recontrôler à l'étape 0 du développement

- `reconciliation_link.rs:77-79`, `:150-175`, `:329-448` ; `routes/reconciliation.rs:6064-6100`, `:6137-6301`.
- `reconciliation_cancel.rs:57-79`, `:141` — ce que devient une écriture saisie au journal et un règlement fournisseur
  à l'annulation (fonde AC-8.3).
- `journal_entries::reverse_in_tx` / `reverse_owned_in_tx` (`journal_entries.rs:2492`, `:2368` `reversal_blockers`) :
  ordre des verrous et lecture du motif `MatchedBankTransaction` (AC-3 (a)).
- Tout `bank_transactions … FOR UPDATE` pris après un verrou de `journal_entries` (AC-3 (b)) :
  `grep -rn "bank_transactions" crates/kesh-db/src/repositories/*.rs | grep -n "FOR UPDATE"`.
- `supplier_invoices.rs:761-1018` (écriture de règlement, compte de la banque crédité).
- `bank_account_closed_inventory.rs:237`, `:516-519`, `:892` ; `audit_route_registry.rs:908-1324` (valeurs du moment).
- `ReconciliationProposals.svelte:230-260`, `:577-586`, `:658-666` ; `LinkSettlementModal.svelte`.
- Le geste *Annuler le rejet* du détail d'import (28-5b) et sa route.

## Collision avec la story parallèle v016-2 (saisie manuelle d'un relevé)

Lue : `/home/gcorbaz/devel/kesh-0162/_bmad-output/implementation-artifacts/v016-2-releve-manuel.md`. Aucun fichier de
**réconciliation** en commun (la v016-2 n'y écrit rien, son AC-16 ne fait que l'exercer). Points de fusion, tous
mécaniques sauf le premier :

1. **`crates/kesh-api/tests/audit_route_registry.rs`** — les deux stories ajoutent des routes mutantes (v016-2 : deux
   `POST`, dont `preview` ; v017-3a : un `POST`) et réécrivent les **mêmes** assertions chiffrées et la même prose
   chiffrée. À la réunion, les compteurs se **recomptent** depuis le registre fusionné, jamais par addition des deux
   diffs.
2. `crates/kesh-api/tests/bank_account_closed_inventory.rs` — la v017-3a ajoute un écrivain (`link_entry_in_tx`) ; la
   v016-2 exige de ne **pas** toucher `REGISTRE` ni `FENETRE` (son AC-8). Pas de conflit de sens ; conflit de texte
   possible sur le compte `sites.len()`.
3. `crates/kesh-api/src/errors.rs`, `audit_labels.rs` (liste triée), `lib.rs` (montage), les quatre catalogues i18n,
   `docs/api-external.md`, `CHANGELOG.md` (`## [0.17.0]`), `docs/manual/fr/user-manual.tex` (sections *Import
   bancaire* pour la v016-2, *Réconciliation* pour la v017-3a) et son PDF : insertions voisines, à fusionner à la main.
   Le PDF se **régénère** après fusion, il ne se fusionne pas.
4. **Synergie** : un compte sans relevé électronique (v016-2) dont Guy a passé les mouvements au journal produira des
   transactions `MANUAL` en attente **déjà comptabilisées** — exactement le cas de la v017-3a, qui ne lit pas
   `source_format`. Un test de l'une sur un relevé de l'autre est souhaitable après la réunion (à l'orchestrateur).

## Questions au Project Lead

Tranchées au registre ; listées pour être soumises à Guy.

- **Q1 (C-v017-3-2)** — Le rattachement porte sur **toute écriture** du journal dont le mouvement sur le compte de la
  banque égale la transaction, pas seulement sur les règlements fournisseur. D'accord ?
- **Q2 (C-v017-3-7)** — L'avertissement est un avis : *Accepter*, *Affecter manuellement* et *Éclater* restent
  possibles après une confirmation ; rien n'est refusé. Préférez-vous qu'une ligne signalée ne propose plus rien,
  comme celle d'un lot de paiement en cours ?
- **Q3 (C-v017-3-4)** — Aucune fenêtre de dates (comme la 29-8) : une écriture de l'an dernier au même montant serait
  candidate, triée en dernier. Une fenêtre vous paraît-elle préférable ?
- **Q4 (C-v017-3-9)** — *Annuler le rapprochement* d'une transaction rattachée à une écriture saisie **contre-passe**
  cette écriture (#672 garde le geste « défaire le seul lien », qui demande une migration). Acceptable pour la 0.17.0 ?
- **Q5 (C-v017-3-10, L2)** — Un virement entre deux de vos comptes, passé au journal, ne se rattache que d'un côté.
  L'issue demandée couvre ce cas avec les lots de plusieurs factures. D'accord pour l'ouvrir (elle fera monter le
  décompte), et pour fermer #671 à la release ?

## Dev Agent Record

### Agent Model Used

### Completion Notes List

### File List

## Change Log

- 2026-10-30 — Fiche créée (Opus 5.5, autonomie), découpée de `v017-3-rapprochement-attente` avec la v017-3b ; registre
  C-v017-3-1 à 12 et 17 ; prompt de validation P1 (`v017-3a-validate-prompt-p1.md`). Prochaine étape : questions Q1–Q5
  à Guy, puis validation P1 (Sonnet, lentilles R et F).
