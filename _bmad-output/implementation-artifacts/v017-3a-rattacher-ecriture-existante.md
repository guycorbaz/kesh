# Story v017-3a : une transaction déjà comptabilisée se rattache à son écriture, et l'écran le signale

Status: ready-for-dev

*Fiche rédigée le 2026-10-30, en autonomie (Opus 5.5), worktree `/home/gcorbaz/devel/kesh-0173`, branche
`story/v017-3-rapprochement-attente` partie de `c5101b07` (tête de la candidate `v0.16.1-rc1`) ; la story se fusionne
dans la release **0.17.0** (première candidate `v0.17.0-rc1`). Sous-story de `v017-3-rapprochement-attente`, découpée
avec **v017-3b** (#687) — motif : C-v017-3-1. Registre : `v017-3-choix-autonomes.md` (C-v017-3-1 à C-v017-3-12,
C-v017-3-17 ; C-v017-3-21 à C-v017-3-23 ajoutés à la validation P1 — les numéros 18 à 20 sont ceux de la v017-3b). Clé courte : `v0173a`. Issue : **#671** (P2, bug), en `refs` seulement dans les commits ; la fermeture se
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
| Un règlement fournisseur par virement depuis la fiche laisse la transaction en attente | `supplier_invoices::pay_in_tx_as` (`crates/kesh-db/src/repositories/supplier_invoices.rs:761-1013`) : écriture `D créanciers / C compte lié de la banque` (`:919-952`), `UPDATE supplier_invoices … status = 'paid'` (`:956-976`), audit `supplier_invoice.paid` ; **aucune** lecture ni écriture de `bank_transactions` | **Exacte** |
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
| D12 | Chaque écriture candidate porte le motif qui empêcherait **d'annuler** le rattachement (la fonction de blocage de l'annulation, réutilisée) ; le dialogue l'affiche et exige une confirmation, sans promettre de contre-passation | C-v017-3-21 |
| D13 | L'autre côté d'un virement entre comptes propres garde un avertissement : `alreadyBooked.matchedElsewhereCount` | C-v017-3-22 |
| D14 | Une seule fonction de blocage de l'annulation, extraite de `cancel_blocker` (`reconciliation_cancel.rs:232`) | C-v017-3-21 |
| D15 | Coût mesuré sur un historique volumineux ; dette et issue si la pente est mauvaise | C-v017-3-23 |

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
  *(Validation P1, R4.)* La jointure interne de la somme est ici **redondante** avec la condition 3 (`M = t.amount ≠ 0`
  écarte déjà une écriture sans ligne sur le compte) : le doc-comment le dit, au lieu de recopier celui de la 29-8 (« c'est
  cette jointure qui écarte le cas `M = 0` », vrai là-bas seulement). *(Validation P1, R7, C-v017-3-22.)* Le même
  constructeur de requête rend une **seconde forme** (`EntryCandidateMode::MatchedOnOtherAccount`), où la condition 4
  devient « rapprochée d'une transaction d'un **autre** compte bancaire » (`EXISTS bt … bt.bank_account_id <>
  t.bank_account_id`) et la condition 6 tombe ; `count_entries_matched_on_other_account(…)` la lit. Une seule fonction de
  texte, deux modes ; un test asserte que les deux textes ne diffèrent que par ces deux prédicats.
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
  *(Validation P1, R9.)* AC-2.6, second volet : la contre-passation `R` d'une écriture `E` a un mouvement de signe opposé ;
  elle se teste **avec une transaction de montant opposé** (`−t`), faute de quoi M5 survit. *(R7.)* 12. une écriture
  `D banque B / C banque A` rapprochée de la transaction du compte A → absente de la liste du compte B, et comptée par
  `count_entries_matched_on_other_account` pour la transaction de B ; rapprochée d'une transaction du **même** compte →
  non comptée.
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
  *(Validation P1, R8, R2.)* Le point (a) couvre aussi la **modification** et la **suppression** d'une écriture saisie
  (`journal_entries::update_in_tx`, `:1459`, `FOR UPDATE` en premier acte, puis `modification_guard` sous ce verrou,
  `:1526-1527` ; `delete_in_tx`, `:1830-…`) et `reverse_at_next_fiscal_year_start_in_tx` (même `reverse_in_tx_inner`,
  `:2741-2745` puis `:2802`). Établi par la lentille R au code, à **reconfirmer** à l'étape 0 : (a) vrai —
  `reverse_in_tx_inner` verrouille `je` (`:2745`) avant `reversal_blockers` (`:2802`) ; (b) vrai — l'inventaire des
  sites qui verrouillent `bank_transactions` est `bank_imports.rs:420-421` (suppression d'un import),
  `reconciliation.rs:1186-1227` (`bank_tx_by_id_for_account_sql(" FOR UPDATE")`), `reconciliation_cancel.rs:200-204` ;
  aucun ne tient une ligne `journal_entries` avant (`cancel_in_tx` prend `t` puis `je`, même ordre que la 3a).
  ⚠️ Le détecteur mono-ligne de la première rédaction (`grep … | grep "FOR UPDATE"`) était **muet** : le motif et le mot-clé
  ne sont jamais sur la même ligne. L'inventaire se refait par fonction, non par ligne.
- **AC-4** — Tests `#[sqlx::test]` de `link_entry_in_tx` : nominal (état final `reconciled`, `matched_entry_id`, rejet
  remis à `NULL`, une trace, ses détails) ; chaque refus sans écriture (transaction absente, d'une autre société,
  rapprochée, en EUR ; écriture absente, d'une autre société, non candidate ; compte bancaire clos **dans** la
  transaction) ; une transaction **rejetée** se rattache (`was_previously_rejected: true`) ; deux rattachements
  successifs de la même écriture à deux transactions du même compte → le second `NotLinkable`, rien d'écrit. *Le test
  de concurrence vraie (deux connexions) suit le patron de la 29-8 s'il existe — le citer au Dev Agent Record ; sinon,
  écrire que l'invariant est tenu par raisonnement (AC-3), comme la 29-8 (C-29-8-25).*
  *(Validation P1, R3 : ce patron existe ; la phrase précédente est remplacée par ce qui suit.)*
  **Courses**, sur le patron de `crates/kesh-api/tests/reconciliation_link_e2e.rs` — `t16_course_annulation_et_rattachement`
  (`:1316`), `t17_course_de_double_rattachement` (`:1348`) et `t17bis_le_second_rattachement_attend_le_verrou_du_reglement`
  (`:1391`) —, dans `reconciliation_link_entry_e2e.rs` : **T-race-1**, deux `link-entry` simultanés de la même écriture à
  deux transactions du même compte → exactement un `200`, l'autre `409 RECONCILIATION_ENTRY_NOT_LINKABLE`, une seule
  transaction pointe l'écriture ; **T-race-2**, `link-entry` contre `POST /journal-entries/{id}/reverse` sur la même
  écriture saisie → exactement l'un réussit ; états finaux possibles : rattachée et contre-passation refusée `409
  MATCHED_BANK_TRANSACTION`, ou contre-passée et rattachement refusé `409 RECONCILIATION_ENTRY_NOT_LINKABLE` ; **T-race-3**,
  sur le patron de `t17bis`, le second rattachement **attend** le verrou de ligne de l'écriture (sonde de
  `PROCESSLIST`, privilège `PROCESS`, jamais `INNODB_TRX`). Les réserves de `t16`/`t17` (ce qu'une course gagnée ne prouve
  pas) se recopient au doc-comment des tests.

### Les routes (`kesh-api`)

- **AC-5** — `GET /api/v1/reconciliation/transactions/{id}/entry-candidates`, rôle Comptable (groupe de
  `link-candidates`, `lib.rs:857-858`) ; réponse `200` `{ bankTransactionId, candidates: [{ journalEntryId,
  entryNumber, fiscalYearId, entryDate, journal, description, amount, daysApart }] }`, `amount` chaîne à deux
  décimales. Ordre des refus **identique** au `POST` et à la 29-8 : `404 NOT_FOUND` (inexistante, autre société) →
  `link_preflight` (**réutilisé**, non recopié : archivé `BankAccountNotFound`, clos `409 BANK_ACCOUNT_CLOSED`, non lié
  `412 BANK_ACCOUNT_NOT_CONFIGURED`) → `404 RECONCILIATION_TRANSACTION_NOT_PENDING`. Transaction nulle ou hors CHF →
  `200`, `candidates: []`.
  *(Validation P1, R6, C-v017-3-21.)* Chaque candidate porte `cancelBlockedBy: string | null` — le code du **premier
  motif** qui empêcherait d'annuler plus tard ce rattachement (`FISCAL_YEAR_CLOSED`, `DOCUMENT_LETTERING_IN_CLOSED_PERIODS`,
  `ACCOUNT_ARCHIVED`, `NO_OPEN_FISCAL_YEAR_TODAY`, …, les codes existants de `SettlementCancelBlocker`), calculé par **la**
  fonction de blocage de l'annulation : `cancel_blocker` (`reconciliation_cancel.rs:232-270`) est scindée en une lecture
  par **écriture** (`entry_cancel_blocker(conn, company_id, entry_id, unlinking)` — classement puis bras client,
  fournisseur ou écriture propre) qu'elle appelle désormais, et que la liste appelle avec `unlinking = None` (l'écriture
  n'étant rapprochée d'aucune transaction, le rang 3 ne peut pas sortir). Aucune seconde précédence. Tests : une écriture
  d'un exercice clôturé est **candidate** avec `cancelBlockedBy = "FISCAL_YEAR_CLOSED"` ; une écriture d'exercice ouvert,
  `null`.
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
  écriture ne peut pas être rattachée à cette transaction : la transaction n'est pas en CHF ou son montant est nul, ou le
  mouvement de l'écriture sur le compte de la banque n'égale pas le montant de la transaction, ou elle est déjà
  rapprochée, contre-passée, ou c'est un règlement de facture client à rattacher comme tel. » *(Validation P1, F6 : la
  condition 1 ajoutée au message.)* ; repli d'`errors.rs` égal mot pour mot au catalogue fr-CH (garde G9 de
  `textes_coherents.rs`) ; quatre locales.
  *(Validation P1, F5.)* La clé `reconciliation-errors-entry-not-linkable` s'**inscrit** à la liste fermée de la garde G9
  de `crates/kesh-api/tests/textes_coherents.rs` (à côté de `("reconciliation-errors-settlement-not-linkable", 1, &[])`,
  `:805`) : sans cette ligne, G9 reste verte et le repli n'est contrôlé par personne. Mutation M24.
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
  *(Validation P1, R6.)* 5. une écriture saisie d'un **exercice clôturé**, rattachée : *Annuler le rapprochement* est
  refusé `409` avec son motif (`FISCAL_YEAR_CLOSED`), rien d'écrit, et ce motif est celui que la liste annonçait
  (`cancelBlockedBy`) — même fonction.
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

  *(Validation P1, R1, F10.)* Les autres valeurs chiffrées du même registre se recomptent aussi : `LIB_ROUTES.len(), 135`
  (`:909-911`), `138` (`:944`), la prose `:33` (« plus large (138) ») ; et le test `le_manuel_dit_le_compte_du_registre`
  (`:954-985`) exige que le **manuel d'administration** dise le compte : la phrase « 129 des 135 routes à verbe mutant …
  129 + 4 + 2 = 135 » (`docs/manual/fr/admin-manual.tex:2966`) se récrit aux valeurs recomptées, et le PDF
  d'administration se régénère. **Collision** : la v016-2 réécrit la même phrase et les mêmes assertions (§ *Collision*).
  *(R11.)* Le tri des doc-comments élargit le motif : `grep -rnE "chemins qui posent|sept chemins|cinq autres chemins|cinq
  chemins|six chemins|Les cinq chemins" crates/` (rend aussi `reconciliation_e2e.rs:5274`, `tests/reconciliation_cancel.rs:3`,
  et `journal_entry_reversal_e2e.rs:552`, `:765` — autre notion, à écarter) ; et la **définition** de l'« écriture
  propre » de `reconciliation_cancel.rs:11-23` (« une écriture que seule la transaction possède ») se récrit : une
  écriture saisie au journal, indépendante de la transaction, rattachée par la 3a, est elle aussi contre-passée à
  l'annulation.

### La proposition (`GET /reconciliation/proposals`)

- **AC-10** — `ReconciliationProposal` gagne `already_booked: Option<AlreadyBooked>`, `AlreadyBooked { settlement_count:
  usize, entry_count: usize }`, `null` si les deux sont nuls. Les deux comptes viennent de **`find_link_candidates`** et
  de **`find_entry_candidates`**, appelées pour chaque transaction de la page, sur le compte du grand livre lu au
  pré-vol de la page (aucune seconde règle, aucun `COUNT` à part) ; un compte bancaire sans compte du grand livre lié
  → `null` partout. Champ additif ; `candidates`, `notice` et l'ordre des propositions inchangés. Tests HTTP : une
  transaction qui a une écriture candidate porte `alreadyBooked.entryCount = 1` **et** garde ses propositions ; une
  qui n'en a pas porte `null` ; un crédit qui a un règlement client candidat porte `settlementCount = 1`.
  *(Validation P1, R7, C-v017-3-22.)* `AlreadyBooked` gagne `matched_elsewhere_count` (`count_entries_matched_on_other_account`,
  AC-1) ; `already_booked` est non `null` dès qu'un des trois comptes est positif. Test HTTP : après rattachement du côté A
  d'un virement entre deux comptes propres, la transaction du côté B porte `matchedElsewhereCount = 1`.
- **AC-11** — Coût : mesurer, sur une base de démonstration, `GET /reconciliation/proposals?limit=500` avant et après
  (trois mesures chacun), et l'écrire au Dev Agent Record ; si l'écart dépasse une seconde, le dire et demander
  l'issue (*Performance debt*), sans optimiser par une seconde règle.
  *(Validation P1, R5, C-v017-3-23 — remplace le protocole ci-dessus.)* La somme par écriture parcourt **tout**
  l'historique du compte du grand livre à chaque appel, et la page en fait jusqu'à trois par transaction : la mesure se
  fait sur un **historique synthétique volumineux** (base `kesh_v0173a`, jamais une base de Guy) — 500 transactions en
  attente et, sur le compte de la banque, **1 000, 5 000 puis 20 000** écritures synthétiques —, trois mesures par palier,
  avant et après la story. Est « mauvaise » une pente où le temps de la page à 20 000 écritures dépasse **3 s**, ou croît
  plus vite que linéairement d'un palier à l'autre. Dans ce cas : une section *Performance debt* de la fiche (constat,
  chiffres, propriétaire, story de remédiation) **et** l'issue demandée à l'orchestrateur, avec pour remède nommé « une
  lecture par page » (`SUM` par écriture du compte filtré `HAVING movement IN (…)` sur les montants de la page, puis
  appariement en Rust) — qui ne s'implémente **pas** dans cette story.

### L'écran (`frontend`)

- **AC-12** — `ReconciliationProposals.svelte` : sur une ligne dont `alreadyBooked` n'est pas `null`, un avertissement
  (`data-testid="already-booked-warning"`) : « Cette transaction semble déjà comptabilisée : {count} écriture(s) du même
  montant, non rapprochée(s), sur le compte de la banque. Rattachez-la plutôt que de l'affecter : l'affecter passerait
  une seconde écriture. », avec le bouton du dialogue. Le texte ne cite aucun numéro d'issue.
  *(Validation P1, R12 = F4 — remplace le texte ci-dessus.)* `{count}` = `settlementCount + entryCount` ; texte : « Cette
  transaction semble déjà comptabilisée : {count} écriture(s) non rapprochée(s) du compte de la banque lui correspond(ent)
  — au même montant, ou à 5 centimes près pour un règlement de facture client. Rattachez-la plutôt que de l'affecter :
  l'affecter passerait une seconde écriture. » ; *(R7)* quand `matchedElsewhereCount > 0`, une seconde phrase : « Une
  écriture du même montant est déjà rapprochée d'une transaction d'un autre de vos comptes bancaires : c'est
  probablement l'autre côté d'un virement entre vos comptes. Ne l'affectez pas : rejetez-la. » Vitest des quatre cas
  (seulement `entryCount`, seulement `settlementCount`, les deux, seulement `matchedElsewhereCount`). *(F8.)*
  L'avertissement **n'a pas de second bouton** : il renvoie au bouton de la ligne (un seul `link-settlement-button` par
  ligne, mode strict de Playwright).
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
  *(Validation P1, R6, C-v017-3-21.)* Le rappel « Annuler plus tard ce rapprochement contre-passera cette écriture… »
  ne s'affiche que pour une candidate dont `cancelBlockedBy` est `null`. Pour une candidate dont il ne l'est pas, le
  dialogue affiche, sur sa ligne et à son choix, « Ce rattachement ne pourra pas être annulé : {motif traduit}. » et
  *Rattacher* exige une case à cocher (`data-testid="entry-irreversible-ack"`) ; aucune promesse de contre-passation.
  *(F7.)* Le code `RECONCILIATION_ENTRY_NOT_LINKABLE` entre dans l'ensemble `RELIRE_SUR` (`LinkSettlementModal.svelte:53-57`)
  — le dialogue le dit et relit les deux listes, comme le manuel le promet (`user-manual.tex:2390`) ; Vitest du cas
  « course » calqué sur `LinkSettlementModal.test.ts`. `data-testid` de la section écritures : `entry-candidate-row`,
  `entry-candidate-radio` (`data-entry-id`), `entry-candidate-irreversible`, `link-entry-empty`.
  *(F8.)* Tests existants à récrire, nommés : `ReconciliationProposals.test.ts:1030-1038` (le bouton sur un débit : attendu
  `['1','2']` pour `100.03` et `-50.00`, toujours absent pour `0.00` et l'EUR) ; `frontend/tests/e2e/reconciliation-link.spec.ts:177`
  (inchangé si un seul bouton) ; les clés `reconciliation-link-no-entry` (`messages.ftl:1397`, garder les mots « aucune
  écriture », que l'E2E asserte), `reconciliation-link-modal-title` et `reconciliation-link-empty`, récrites pour les
  deux sections dans les quatre catalogues.
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

  *(Validation P1, F12.)* S'ajoutent : le dialogue aux **deux listes vides** ; le dialogue d'un **crédit sans règlement
  mais avec une écriture** ; une candidate choisie avec le **rappel** de contre-passation ; une candidate **irréversible**
  (exercice clôturé) avec sa case ; l'erreur de course et la relecture ; la ligne du **second côté** d'un virement
  (`matchedElsewhereCount`).

### Le parcours (Playwright)

- **AC-17** — `frontend/tests/e2e/reconciliation-link-entry.spec.ts` : un Comptable (montants **uniques** par
  exécution, base partagée entre specs — patron de `reconciliation-amount-due.spec.ts`) saisit par l'API une écriture
  `D charge / C banque` d'un montant unique, importe un relevé dont un débit l'égale (*validation P1, F9* : par le patron multipart de
  `reconciliation-amount-due.spec.ts:139-142`, `ctx.post('/api/v1/bank-imports', { multipart: … })`, qui produit un montant
  unique ; **non** `bank-import-fixtures.ts`, qui importe une fixture CSV fixe), ouvre `/reconciliation`, voit
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
  *(Validation P1, F1 — HIGH ; F2 ; F3 ; F11.)* **Les passages à récrire, nommés un à un** (numéros de `c5101b07`) :
  1. § *Régler une facture fournisseur*, `:1634-1640` — « Ne payez pas depuis la fiche une facture dont le débit est (ou
     sera) au relevé : Kesh ne sait pas encore rapprocher ce débit du paiement enregistré » : devient « payée depuis la
     fiche, la facture se rattache ensuite à la transaction » ; le processus « saisir ouverte, accepter à la
     réconciliation » reste le plus direct ;
  2. l'**encadré** `keshwarning` « Dépense déjà réglée par un autre chemin », `:2324-2343` — récrit en entier : une
     dépense payée depuis la fiche, par un lot d'**une** facture ou par une écriture saisie **se rattache** (même dans un
     exercice clôturé, avec la mise en garde de l'irréversibilité, D12) ; « laissez la transaction en attente (issue
     #671) » disparaît ; restent en attente, avec la consigne de **rejeter**, les cas de D9 (lot de plusieurs factures,
     virement entre comptes propres — second côté —, ancien compte, autre compte), renvoyés à **#823** ;
  3. § des règles d'affectation, `:2450` — « Une dépense déjà réglée … ne la masque pas : voyez l'encadré » et
     « *Affecter manuellement* ne convient que si … » : renvoi à l'avertissement et au rattachement ;
  4. diagnostic, `:3337-3338` — « n'acceptez rien pour ce débit (encadré …) » : « rattachez-la (§…) » ;
  5. `docs/api-external.md:626` — « ⚠️ Dépense déjà réglée par un autre chemin (angle mort, #671) … Laisser la
     transaction en attente jusqu'au rattachement (#671) » : récrit (AC-19) ;
  6. *(F2)* la liste « Ce qui ne se rattache pas », `:2402-2409`, puce par puce : `:2402` « (issue #671) » → « (issue
     #823) » ; `:2404` (plus de 5 centimes) **gardée**, précisée « pour un règlement de facture client ; une écriture se
     rattache au montant exact » ; `:2405` (autre compte bancaire) **gardée** ; `:2406` (compte interne) **supprimée** —
     un règlement par compte interne sur le compte de la banque se rattache désormais (AC-2.7) ; `:2407` (plusieurs
     factures) **gardée**, renvoi à #823 ; `:2408` **scindée** — l'écriture saisie au journal se rattache, l'ancien compte
     reste ; `:2409` (un débit) **supprimée** ; ajoutées : le second côté d'un virement entre comptes propres (D13) et le
     règlement fournisseur sous le centime (renvoi à la v017-3b) ;
  7. *(F3)* § *Annuler un rapprochement*, `:2462-2484` : une transaction rattachée à une **écriture saisie** — l'annulation
     la **contre-passe** ; rattachée à un **règlement fournisseur** — le règlement est annulé, la facture redevient
     ouverte, geste offert depuis le détail de l'import (`:2467-2470`) ; un rattachement à une écriture d'exercice
     clôturé ne s'annule pas (D12) ; l'introduction couvre le **rattachement** erroné ;
  8. *(F3)* `:1961-1963` (issues d'une transaction rejetée qui bloque la clôture du compte) et `:2358` (« …éclatées ou
     rattachées à un règlement ») : « ou rattachée à une écriture existante ».
  *(F1.)* Le filet de propagation, au `.tex` **et** au PDF aplati (`pdftotext … - | tr '\n' ' ' | tr -s ' '`), sur des
  **fragments courts**, en grepant les **deux** apostrophes (`'` et `’`) : `déjà réglée par un autre chemin`,
  `ne sait pas encore rapprocher`, `acceptez rien`, `en attente`, `issue \#671`, `rattachées à un règlement`,
  `Rattacher à un règlement`, `ne se rattache`. Le troisième motif de la première rédaction (« Kesh ne rapproche pas une
  transaction d'une écriture qui existerait déjà ») ne rend **rien** (0 au `.tex` et au PDF) : retiré. *(F11.)* Les
  renvois du manuel au reste pointent **#823** ; `#671` ne reste que dans l'historique du CHANGELOG.
- **AC-19** — `docs/api-external.md` : les deux routes (après `:675-677`), le champ `alreadyBooked` des propositions,
  le code `RECONCILIATION_ENTRY_NOT_LINKABLE` au tableau des codes (après `:868`), la route ajoutée à la liste des
  routes rejouées (`:878`), « *(Depuis la v0.17.0.)* ».
  *(Validation P1, F1.)* Et `:626` (« Dépense déjà réglée par un autre chemin », angle mort #671) récrit comme
  l'encadré du manuel ; filet `grep -n "#671\|jusqu'au rattachement" docs/api-external.md`.
- **AC-20** — `CHANGELOG.md` : *(validation P1, F10 : la section n'existe pas à `c5101b07` ; la v016-2 la crée — dans
  cette branche, l'entrée s'écrit sous un en-tête **provisoire** `## [0.17.0] — Non publié`, que l'orchestrateur fusionne)*
  sous `## [0.17.0]`, section **Corrigé**, une entrée (`refs` [#671] par un lien, sans
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
| M2 | condition 3 : jointure interne → `LEFT JOIN … COALESCE(…, 0)` | AC-2.4 (autre compte) — *validation P1, R4 : mutant **équivalent** (`M = 0 ≠ t.amount` l'écarte déjà) ; remplacé par M2bis* |
| M2bis | table dérivée : filtre `l.account_id = ?` retiré | AC-2.4 (une écriture sur un autre compte, au même montant, devient candidate) |
| M3 | condition 4 retirée | AC-2.5, AC-4 (second rattachement) |
| M4 | condition 5 : le `NOT EXISTS` de la contre-passée retiré | AC-2.6 |
| M5 | condition 5 : `reverses_entry_id IS NULL` retiré | AC-2.6 |
| M6 | condition 6 retirée | AC-2.7 (règlement client par virement absent) |
| M7 | condition 6 : `settlement_type = 'bank_transfer'` retiré | AC-2.7 (compte interne présent) |
| M8 | `transaction_can_be_linked_to_entry` : `amount != 0` → `amount > 0` | AC-2.1, AC-8 |
| M9 | tri : `ORDER BY e.id` seul | AC-2.10 |
| M10 | 4.3 déplacé avant 4.2 (lecture ordinaire avant le verrou de l'écriture) | revue (C-29-8-25) — **survivante assumée** si aucun test de concurrence n'existe ; le dire. *Validation P1, R3 : les tests de concurrence existent (T-race-1 à 3) ; M10 reste survivante si aucun ne voit la naissance de la vue, et cela se déclare comme la M14 de la 29-8* |
| M11 | `UPDATE` sans `auto_match_rejected_at = NULL` | AC-4 (rejetée) |
| M12 | trace : action `reconciliation.linked` | AC-4, AC-9 |
| M13 | proposition : `entry_count` toujours 0 | AC-10 |
| M14 | écran : `canLink` réservé aux crédits | Vitest AC-13 |
| M15 | écran : confirmation sautée pour *Affecter manuellement* | Vitest AC-14 |
| M16 | écran : *Rattacher* appelle `link` pour une écriture | Vitest AC-13 |
| M17 | geste : restriction `AND e.id = ?` retirée (la première candidate est rattachée) | AC-4 — *validation P1, R10 : test à deux candidates qui désigne la seconde* |
| M18 | règle : `e.company_id = ?` retiré | AC-2.9 |
| M19 | règle : `LIMIT 50` retiré | AC-2.10 |
| M20 | geste : `FOR UPDATE` de l'étape 4.2 retiré | T-race-3 (le second n'attend plus) ; T-race-1 — *validation P1, R3* |
| M21 | mode `MatchedOnOtherAccount` : prédicat `bt.bank_account_id <> t.bank_account_id` retiré | AC-2.12 (même compte compté) |
| M22 | liste : `cancelBlockedBy` toujours `null` | AC-5 (exercice clôturé), AC-8.5 |
| M23 | dialogue : `RECONCILIATION_ENTRY_NOT_LINKABLE` absent de `RELIRE_SUR` | Vitest AC-13 (course) |
| M24 | repli d'`errors.rs` altéré d'un mot | G9 (`textes_coherents.rs`, AC-7) |

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
  demandée (C-v017-3-10) — *ouverte depuis : **#823***.
- **L2 — Virement entre deux comptes bancaires propres** : une écriture `D banque B / C banque A` correspond à **deux**
  transactions (une par compte). L'invariant « une écriture, une transaction » ne laisse rattacher qu'**un** côté ;
  l'autre reste en attente. Dans l'issue de L1 **(Q5)**. *(Validation P1, R7, C-v017-3-22.)* L'autre côté **garde un
  avertissement** (`matchedElsewhereCount`, AC-10, AC-12) qui conseille de le rejeter : sans lui, il aurait perdu tout
  signal au premier rattachement et se serait laissé affecter en double. Le traitement complet relève de #823.
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
- **L9 — Rattachement sans retour** *(validation P1, R6, C-v017-3-21)* : un rattachement à une écriture d'un exercice
  clôturé, ou dont l'annulation est refusée pour un autre motif, ne s'annule pas. Il reste **admis** (le cas d'usage de
  #671 : un relevé importé après la clôture), mais le dialogue le dit, par candidate, et exige une case à cocher ; aucune
  promesse de contre-passation n'est faite pour lui.
- **L8 — Rejetées** : l'avertissement ne se calcule que pour les transactions de la liste (non rejetées) ; une
  transaction rejetée selon la consigne d'attente n'est signalée qu'après *Annuler le rejet*.

## À recontrôler à l'étape 0 du développement

- `reconciliation_link.rs:77-79`, `:150-175`, `:329-448` ; `routes/reconciliation.rs:6064-6100`, `:6137-6301`.
- `reconciliation_cancel.rs:57-79`, `:141` — ce que devient une écriture saisie au journal et un règlement fournisseur
  à l'annulation (fonde AC-8.3).
- `journal_entries::reverse_in_tx` / `reverse_owned_in_tx` (`journal_entries.rs:2492`, `:2368` `reversal_blockers`) :
  ordre des verrous et lecture du motif `MatchedBankTransaction` (AC-3 (a)).
- Tout `bank_transactions … FOR UPDATE` pris après un verrou de `journal_entries` (AC-3 (b)) :
  `grep -rn "bank_transactions" crates/kesh-db/src/repositories/*.rs | grep -n "FOR UPDATE"`. *Validation P1, R2 : ce
  détecteur est **muet** (motif et mot-clé jamais sur la même ligne) ; l'inventaire se fait par fonction — sites établis
  à l'AC-3, à reconfirmer : `bank_imports.rs:420-421`, `reconciliation.rs:1186-1227`, `reconciliation_cancel.rs:200-204`.*
- *(Validation P1.)* `reconciliation_cancel.rs:215-270` (`cancel_blocker`, à scinder — D14) ;
  `settlement_cancellation.rs:87-182` (motifs) ; `LinkSettlementModal.svelte:53-57` (`RELIRE_SUR`) ;
  `textes_coherents.rs:805` (G9) ; `audit_route_registry.rs:909-985` ; `admin-manual.tex:2966` ;
  `reconciliation_link_e2e.rs:1316-1430` (t16, t17, t17bis).
- `supplier_invoices.rs:761-1013` (écriture de règlement, compte de la banque crédité).
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
3-bis. *(Validation P1, R1 = F10.)* **`docs/manual/fr/admin-manual.tex:2966`** (« 129 des 135 routes à verbe mutant …
   129 + 4 + 2 = 135 ») et son PDF, que lit le test `le_manuel_dit_le_compte_du_registre`
   (`audit_route_registry.rs:954-985`) : la v016-2 la récrit aussi (sa fiche, AC-2 et § release). Comme les assertions
   du point 1, elle se **recompte depuis le registre fusionné** à la réunion, jamais en additionnant les deux diffs ; le
   PDF d'administration se régénère ensuite.
3-ter. `CHANGELOG.md` : la section `## [0.17.0]` n'existe pas à `c5101b07` ; la v016-2 la crée ; la 3a écrit sous un
   en-tête provisoire (AC-20).
4. **Synergie** : un compte sans relevé électronique (v016-2) dont Guy a passé les mouvements au journal produira des
   transactions `MANUAL` en attente **déjà comptabilisées** — exactement le cas de la v017-3a, qui ne lit pas
   `source_format`. Un test de l'une sur un relevé de l'autre est souhaitable après la réunion (à l'orchestrateur).

## Collision avec la v017-3b (#687) — sœur, même release

*(Validation P1, F10.)* La 3b récrit des passages du manuel que la 3a récrit aussi ; ils se réconcilient à la réunion des
deux branches, la 3a récrivant l'encadré et la 3b n'y ajoutant que ses renvois (fiche 3b, § *Dépendance à la v017-3a*,
qui porte le tableau commun) :

| Section (`user-manual.tex`, `c5101b07`) | v017-3a | v017-3b |
|---|---|---|
| `sec:regler-facture-fournisseur`, `:1630-1640` | récrit (AC-18.1) | facture existante sous le centime, conduite |
| `sec:matching-fournisseur`, `:2304`, encadré `:2324-2343` | encadré récrit (AC-18.2) | complète « un total à plus de deux décimales n'est jamais proposé » |
| `sec:rattacher-reglement`, `:2384-2412` | récrit (AC-18, AC-18.6) | sa puce « règlement sous le centime » renvoie à la 3b |
| Diagnostic, `:3325-3340` | `:3337-3338` (AC-18.4) | `:3329` |

## Issues à ouvrir (demandées par la validation P1)

- *Selon la mesure de l'AC-11* : « Réconciliation : le coût de la page de propositions croît avec l'historique du compte
  de la banque » (dette de performance), si la pente est mauvaise — constat et chiffres au Dev Agent Record.
- Aucune autre : le reste de #671 est **#823** (ouverte), que complète le second côté d'un virement (L2) — à y ajouter en
  commentaire par l'orchestrateur : « le second côté garde un avertissement depuis la v017-3a ; son traitement reste
  ici ».

## LOW non pris

Aucun : R8 à R13 et F6, F8 à F12 sont appliqués (R13 : citations `:761-1013` ; `reconciliation_cancel.rs:57-79` désigne
bien le doc-comment de l'énumération, laissé).

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
  *(Validation P1, R6 : et un rattachement à une écriture d'un exercice clôturé **ne s'annule pas** du tout ; il reste
  admis, signalé et confirmé par une case — C-v017-3-21. D'accord ?)*
- **Q5 (C-v017-3-10, L2)** — Un virement entre deux de vos comptes, passé au journal, ne se rattache que d'un côté.
  L'issue demandée couvre ce cas avec les lots de plusieurs factures. D'accord pour l'ouvrir (elle fera monter le
  décompte), et pour fermer #671 à la release ? *(Validation P1 : l'issue est ouverte, #823 ; le second côté garde un
  avertissement, C-v017-3-22. Reste : fermer #671 à la release ?)*

## Dev Agent Record

### Agent Model Used

### Completion Notes List

### File List

## Change Log

- 2026-10-30 — Fiche créée (Opus 5.5, autonomie), découpée de `v017-3-rapprochement-attente` avec la v017-3b ; registre
  C-v017-3-1 à 12 et 17 ; prompt de validation P1 (`v017-3a-validate-prompt-p1.md`). Prochaine étape : questions Q1–Q5
  à Guy, puis validation P1 (Sonnet, lentilles R et F).
- 2026-10-30 — **Validation P1** (Sonnet, lentilles R et F) : R 0/0/7/6, F 0/1/5/6 — 0 CRITICAL, 1 HIGH, 12 MEDIUM,
  12 LOW. Recoupements : R12 = F4 ; R1 ≈ F10. Chaque prémisse vérifiée au code avant application ; **aucun finding
  rejeté** (R4 est un mutant équivalent : confirmé, M2 remplacée). Remédiation : registres, manuel d'administration
  `:2966` et collision déclarée (R1, F10) ; détecteur muet remplacé par l'inventaire par fonction (R2) ; courses T-race-1
  à 3 sur le patron de `t16`/`t17`/`t17bis`, M20 (R3) ; M2bis (R4) ; coût mesuré sur 1 000 / 5 000 / 20 000 écritures,
  dette et issue si la pente est mauvaise (R5, C-v017-3-23) ; `cancelBlockedBy` par la fonction de blocage de
  l'annulation, scindée et réutilisée, dialogue sans fausse promesse, test d'exercice clôturé (R6, C-v017-3-21, D12, D14,
  L9) ; second côté d'un virement averti, `matchedElsewhereCount` (R7, C-v017-3-22, D13) ; point (a) élargi (R8) ;
  signe opposé (R9) ; M17 à M24 (R10, F5) ; doc-comments et définition de l'« écriture propre » (R11) ; texte de
  l'avertissement et `{count}` (R12, F4) ; citations (R13) ; huit passages du manuel et `api-external.md:626` nommés,
  filet sur fragments courts aux deux apostrophes (F1 HIGH, F2, F3) ; G9 (F5) ; condition 1 au message (F6) ;
  `RELIRE_SUR` et `data-testid` (F7) ; tests et clés existants (F8) ; patron multipart (F9) ; CHANGELOG provisoire, v017-3b
  (F10) ; #823 (F11) ; captures (F12). Prochaine étape : validation P2 (`v017-3a-validate-prompt-p2.md`, Opus, lentilles R
  et F).
