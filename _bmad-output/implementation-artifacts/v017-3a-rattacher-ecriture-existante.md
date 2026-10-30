# Story v017-3a : une transaction déjà comptabilisée se rattache à son écriture

Status: ready-for-dev

*Fiche rédigée le 2026-10-30, en autonomie (Opus 5.5), worktree `/home/gcorbaz/devel/kesh-0173`, branche
`story/v017-3-rapprochement-attente` partie de `c5101b07` (tête de la candidate `v0.16.1-rc1`) ; la story se fusionne
dans la release **0.17.0** (première candidate `v0.17.0-rc1`). Sous-story de `v017-3-rapprochement-attente`, découpée
avec **v017-3b** (#687) — motif : C-v017-3-1. Registre : `v017-3-choix-autonomes.md` (C-v017-3-1 à C-v017-3-12,
C-v017-3-17 ; C-v017-3-21 à C-v017-3-23 ajoutés à la validation P1 — les numéros 18 à 20 sont ceux de la v017-3b ;
C-v017-3-25, découpage à la validation P2 : l'avertissement des propositions passe à **v017-3c**, hors release). Clé courte : `v0173a`. Issue : **#671** (P2, bug), en `refs` seulement dans les commits ; la fermeture se
décide à la PR de release (C-v017-3-17). Les numéros de ligne cités sont ceux de `c5101b07`.*

**Aucune migration. Aucune version bumpée.** La contrainte du Project Lead du 2026-10-30
(`kesh-gate-logs/prompts/v017-donnees-a-garder.md`) s'applique : rien de ce que porte une base 0.16.x n'est réécrit ;
une transaction en attente à tort se règle par un geste de l'utilisateur.

## Story

En tant que Comptable qui rapproche ses relevés, et dont certains mouvements bancaires sont **déjà comptabilisés** —
facture fournisseur payée depuis sa fiche, écriture saisie au journal, règlement par compte interne sur le compte de la
banque —, je veux que Kesh me laisse **rattacher** une transaction en attente à cette écriture déjà passée, afin qu'elle passe rapprochée sans seconde écriture, qu'elle cesse
d'empêcher la clôture du compte bancaire et de figurer au contrôle K2 de la préparation de clôture. *(Validation P2,
C-v017-3-25 : le signalement automatique d'une transaction qui « semble déjà comptabilisée », demandé par la précision
du Project Lead ci-dessous, est renvoyé à la v017-3c, hors de la release 0.17.0.)*

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

*Réorganisé à la validation P2 (C-v017-3-25, découpage) : l'avertissement des propositions (`alreadyBooked`, ses deux
modes, la confirmation avant *Accepter*, *Affecter*, *Éclater*) et la mesure de coût qui l'accompagnait passent à
**v017-3c** (`v017-3c-avertissements-propositions.md`, `backlog`, hors de la release 0.17.0). Les anciennes D6, D13 et
D15 y vont ; la numérotation des autres est gardée.*

| # | Décision | Registre |
|---|---|---|
| D1 | Rattacher à une **écriture** du journal : routes neuves `GET …/entry-candidates` et `POST …/link-entry` ; la 29-8 inchangée | C-v017-3-2 **(Q1)** |
| D2 | Montant **exact**, signé, sur le compte du grand livre actuel du compte bancaire | C-v017-3-3 |
| D3 | Aucune fenêtre de dates ; la proximité trie | C-v017-3-4 **(Q3)** |
| D4 | Le domaine de la 29-8 (règlement client par virement sur ce compte) est exclu de la liste des écritures | C-v017-3-5 |
| D5 | Ni écriture contre-passée, ni contre-passation | C-v017-3-6 |
| D6 | *(Passée à la v017-3c.)* | C-v017-3-7, C-v017-3-25 |
| D7 | Un dialogue, deux sections ; bouton « Rattacher à une écriture existante » sur toute ligne CHF non nulle | C-v017-3-8 |
| D8 | L'annulation d'un rattachement suit le classement existant (contre-passe une écriture saisie, annule un règlement) | C-v017-3-9 **(Q4)** |
| D9 | Hors périmètre, chacun tracé : une transaction pour plusieurs écritures et le second côté d'un virement (#823) ; ancien compte du grand livre, autre compte bancaire (issue demandée) ; frais de rappel (#401) | C-v017-3-10 **(Q5)** |
| D10 | Données existantes intouchées ; une transaction rejetée se rattache après *Annuler le rejet*, ou directement par l'API | C-v017-3-11 |
| D11 | Trace `reconciliation.entry_linked` ; refus `409 RECONCILIATION_ENTRY_NOT_LINKABLE` | C-v017-3-12 |
| D12 | Chaque écriture candidate porte le motif qui empêche **aujourd'hui** d'annuler le rattachement (la fonction de blocage de l'annulation, réutilisée) ; le dialogue le dit comme une **vérité datée** — « tant que … subsiste » —, avec le moyen de le lever, sans promesse inconditionnelle de contre-passation | C-v017-3-21 (amendé) |
| D13 | *(Passée à la v017-3c.)* | C-v017-3-22, C-v017-3-25 |
| D14 | Une seule fonction de blocage de l'annulation, extraite de `cancel_blocker` (`reconciliation_cancel.rs:232`) | C-v017-3-21 |
| D15 | *(Passée à la v017-3c.)* | C-v017-3-23, C-v017-3-25 |

### La règle (une seule définition)

Une écriture `e` est **rattachable** à une transaction `t` si et seulement si, tous ensemble :

1. **la transaction** est de la société, `pending` (rejetée ou non), en CHF, de montant **non nul**
   (`transaction_can_be_linked_to_entry`, fonction pure de `kesh-db`, à côté de `transaction_can_be_linked`) ;
2. **l'écriture** est de la même société ;
3. son **mouvement** `M` sur le compte du grand livre **actuel** du compte bancaire de `t` (`SUM(debit) − SUM(credit)`
   de ses lignes sur ce compte) est **égal** à `t.amount`, en décimal exact (un crédit de la banque appelle `M > 0`, un
   débit `M < 0`) ; une écriture sans ligne sur ce compte n'est pas candidate ;
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
  rattaché d'office (règle *Un appariement automatique propose, il ne crée jamais*).
- **Pas une écriture** : aucun `journal_entries::create_in_tx`, aucun règlement, aucun lettrage.
- **Pas un lot** : un geste unitaire ; le *Pattern batch* ne s'applique pas.
- **Pas un avertissement sur la page de propositions** : celui-ci relève de la v017-3c (C-v017-3-25). Dans la 0.17.0,
  l'utilisateur ouvre le dialogue de lui-même ; la consigne du manuel reste de **rejeter** plutôt que d'affecter une
  transaction déjà comptabilisée qu'il ne rattache pas.

## Critères d'acceptation

### La règle et la lecture (`kesh-db`)

- **AC-1** — Module `crates/kesh-db/src/repositories/reconciliation_link.rs` étendu (ou sous-module voisin, à nommer au
  Dev Agent Record) : `transaction_can_be_linked_to_entry(&BankTransaction) -> bool` (CHF, `amount ≠ 0` — `-0.00`
  compris comme nul) ; `entry_candidates_sql(one: bool) -> String` (conditions 2 à 6, tri, `LIMIT`) ;
  `find_entry_candidates(executor, company_id, &t, bank_ledger_account_id) -> Vec<EntryCandidate>` (vide si la
  condition 1 est fausse) ; `EntryCandidate { journal_entry_id, entry_number, fiscal_year_id, entry_date, journal,
  description, amount, days_apart }`, `amount` normalisé par `Decimal::rescale(2)` (patron `:112-130`). Un test
  unitaire asserte que la requête restreinte est la requête de liste plus `AND e.id = ?` (patron `:495-501`), et
  qu'elle ne contient pas `FOR UPDATE`. La jointure interne de la somme est **redondante** avec la condition 3 (`M =
  t.amount ≠ 0` écarte déjà une écriture sans ligne sur le compte) : le doc-comment le dit, au lieu de recopier celui de
  la 29-8 (« c'est cette jointure qui écarte le cas `M = 0` », vrai là-bas seulement) (validation P1, R4).
  *(Le second mode de la requête, `MatchedOnOtherAccount`, ajouté à la validation P1, est passé à la v017-3c.)*
- **AC-2** — Tests `#[sqlx::test]` de `find_entry_candidates`, un par condition, chacun avec une écriture **qui
  passerait sans elle** (sinon le test est muet) :
  1. un débit de `-250.00` et une écriture saisie au journal `D charge 250 / C banque 250` → candidate, `amount =
     "-250.00"` ;
  2. un crédit de `+80.00` et une écriture `D banque 80 / C produit 80` → candidate ; la même écriture contre un débit
     de `-80.00` → absente (signe) ;
  3. montant différent d'un centime → absente ;
  4. écriture passée sur un **autre** compte du grand livre (ancien compte lié, ou autre banque à compte distinct) →
     absente ;
  5. écriture déjà rapprochée d'une autre transaction → absente ;
  6. écriture contre-passée → absente ; sa contre-passation, testée **avec une transaction de montant opposé** (`−t` :
     son mouvement est de signe opposé — sans cela M5 survit, validation P1, R9) → absente ;
  7. règlement client par virement sur ce compte bancaire, au montant exact → absent de cette liste (et présent de
     `find_link_candidates`, assertion croisée) ; règlement client par **compte interne** dont la contrepartie est le
     compte de la banque → présent ;
  8. règlement fournisseur par virement depuis la fiche (`pay_in_tx`) → présent ; lot pain.001 d'une facture confirmé
     → présent ; lot de **deux** factures contre un débit égal à leur somme → **aucune** candidate (aucune écriture
     n'égale la somme) — l'angle mort L1 asserté ;
  9. autre société → absente. *(Validation P2, R4.)* La fixture est **SQL directe** : une écriture de la société B
     portant une ligne sur le compte du grand livre **de A** (état qu'aucune route ne produit). Une fixture construite
     par les fonctions de dépôt n'a, pour B, aucune ligne sur ce compte : la condition 3 l'écarterait seule, et M18
     survivrait (test muet) ;
  10. tri : trois candidates à 2, 30 et 2 jours → ordre par jours puis `id` ; 51 candidates → 50 rendues, la plus
      lointaine coupée ;
  11. transaction en EUR, ou de montant nul → liste vide sans requête (condition 1) ;
  12. *(Validation P2, R1.)* deux comptes bancaires **partageant** un compte du grand livre (état admis :
      `20260507200001_bank_account_journal_link.sql:21`, aucune contrainte `UNIQUE` ; `bank_accounts.rs:1786`,
      `:1849-1850`, `sharing`) : une écriture non rapprochée de A, au montant d'une transaction de B, **est** candidate
      pour B — comportement assumé (angle mort L3), asserté pour qu'il ne change pas en silence ; et un règlement client
      **par virement sur B**, au montant d'une transaction de A, est présent pour A (la condition 6 n'exclut que le
      domaine de la 29-8 **du compte de `t`**) — c'est ce cas qui tue M7bis.
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
  `:290-293`), et **qui sérialise** : le verrou de ligne de `je`.
  **(a)** Les gestes qui contre-passent, modifient ou suppriment l'écriture la verrouillent **avant** de lire
  `bank_transactions` pour le motif `MatchedBankTransaction` — établi par la lentille R au code, à reconfirmer à
  l'étape 0 : `reverse_in_tx_inner` (`journal_entries.rs:2741-2745` puis `reversal_blockers`, `:2802`), qui sert aussi
  `reverse_at_next_fiscal_year_start_in_tx` ; `update_in_tx` (`:1457-1459`, `FOR UPDATE` en premier acte, puis la garde
  d'écriture sous ce verrou, `:1540-1541` — *citation `:1526-1527` de la remédiation P1 corrigée, validation P2, R8*) ;
  `delete_in_tx` (`:1819`, verrou `:1831-1835`).
  **(b)** *(Réécrit à la validation P2, R6.)* L'absence de cycle avec `t → je` ne tient **pas** à ce qu'« aucun geste ne
  tient une ligne `journal_entries` avant `bank_transactions` » : l'inventaire complet des sites qui prennent une ligne
  de `bank_transactions` comprend les lectures verrouillantes (`bank_imports.rs:420-421`, `reconciliation.rs:1186-1227`,
  `reconciliation_cancel.rs:200-204`) **et** les `UPDATE` (`routes/reconciliation.rs:2810`, `:3373`, `:3739`, `:4218`,
  `:4595`, `:5046`, `:5509`, `:6013` ; `reconciliation_link.rs:403` ; `reconciliation_cancel.rs:502`), et un lot
  d'acceptation lettre une facture (lignes de son écriture de vente) avant de mettre à jour la transaction suivante.
  Ce qui exclut le cycle, c'est que **tous** ces gestes tiennent le **même verrou nommé du compte** pendant leurs
  instructions, et que les routes **rejouent** l'interblocage (`retry_app_on_deadlock`). Le doc-comment le dit ainsi,
  ainsi que le fait que l'étape 4.2 verrouille l'écriture **désignée par le corps**, candidate ou non (le refus
  `NotLinkable` ne vient qu'en 4.4). L'inventaire se refait **par fonction**, jamais par un grep mono-ligne (le
  détecteur de la première rédaction était muet). Si l'étape 0 contredit (a) ou (b), le dire au Dev Agent Record et
  **s'arrêter** : la conception est à revoir, pas à contourner.
  **(b), réécrit à la validation P3 (R1) — remplace les deux paragraphes ci-dessus pour la raison et pour le critère
  d'arrêt.** L'affirmation « **tous** ces gestes tiennent le **même verrou nommé** du compte » est **fausse** : la
  suppression d'un import (`delete_once`, `crates/kesh-api/src/routes/bank_imports.rs:1507-1518`, `pool.begin()` sans
  `with_account_lock`) et les routes du journal (contre-passation, modification, suppression d'une écriture) ne le
  prennent pas. La vraie raison de l'absence de cycle avec `t → je` : **aucun geste qui tient une ligne
  `journal_entries` ne verrouille ensuite `bank_transactions`** — `reverse_in_tx_inner` (`journal_entries.rs:2741-2745`)
  lit `bank_transactions` par `reversal_blockers` **sans verrou** (sous-requête ordinaire, `:2278`) ; `update_in_tx`
  (`:1457-1459`) et `delete_in_tx` (`:1831-1835`) n'écrivent pas `bank_transactions` (le `ON DELETE SET NULL` ne trouve
  aucune ligne, la garde ayant déjà refusé une écriture rapprochée) ; la suppression d'un import ne prend aucun verrou
  sur `journal_entries`. Les gestes qui prennent `bt` **puis** `je` (`reconciliation_cancel::cancel_in_tx`, `link_in_tx`,
  `link_entry_in_tx`) tiennent, eux, le verrou nommé et sont rejoués ; le verrou nommé ne couvre qu'eux. **Critère
  d'arrêt de l'étape 0, sur cette formulation** : si l'étape 0 trouve un geste qui tient une ligne `journal_entries` puis
  **verrouille** (`FOR UPDATE`, `LOCK IN SHARE MODE`, `UPDATE`, `DELETE`) une ligne de `bank_transactions`, le dire au Dev
  Agent Record et **s'arrêter**.
- **AC-4** — Tests `#[sqlx::test]` de `link_entry_in_tx` : nominal (état final `reconciled`, `matched_entry_id`, rejet
  remis à `NULL`, une trace, ses détails) ; chaque refus sans écriture (transaction absente, d'une autre société,
  rapprochée, en EUR ; écriture absente, d'une autre société, non candidate ; compte bancaire clos **dans** la
  transaction) ; une transaction **rejetée** se rattache (`was_previously_rejected: true`) ; deux rattachements
  successifs de la même écriture à deux transactions du même compte → le second `NotLinkable`, rien d'écrit ; *(R10 de
  la P1)* deux candidates, le corps désigne la **seconde** → c'est elle qui est rattachée (tue M17).
  **Courses**, dans `crates/kesh-api/tests/reconciliation_link_entry_e2e.rs`, sur le patron de
  `reconciliation_link_e2e.rs` — `t16_course_annulation_et_rattachement` (`:1316`), `t17_course_de_double_rattachement`
  (`:1348`), `t17bis_le_second_rattachement_attend_le_verrou_du_reglement` (`:1391`) :
  - **T-race-1** — deux `link-entry` simultanés de la même écriture à deux transactions du même compte → exactement un
    `200`, l'autre `409 RECONCILIATION_ENTRY_NOT_LINKABLE`, une seule transaction pointe l'écriture ;
  - **T-race-2** — `link-entry` contre `POST /journal-entries/{id}/reverse` sur la même écriture saisie → exactement
    l'un réussit ; états finaux possibles : rattachée et contre-passation refusée `409 MATCHED_BANK_TRANSACTION`, ou
    contre-passée et rattachement refusé `409 RECONCILIATION_ENTRY_NOT_LINKABLE` ;
  - **T-race-3** *(réécrite à la validation P2, R3 : le patron `t17bis` **complet**)* — une connexion `tenue` ouvre une
    transaction et prend `SELECT … FROM journal_entries WHERE id = e FOR UPDATE` ; le `POST …/link-entry` de `t2` est
    lancé ; la sonde (`attendre_une_requete_en_cours`, motifs `["journal_entries", "FOR UPDATE"]`, privilège `PROCESS`,
    jamais `INNODB_TRX`) le voit **attendre** ; `tenue` pose `matched_entry_id = e`, `status = 'reconciled'` sur `t1` et
    **commite** ; le `POST` doit rendre `409 RECONCILIATION_ENTRY_NOT_LINKABLE`, et **un seul** lien subsister. Ce test
    tue M10 (la vue née avant l'attente relirait l'ancien état et rattacherait `t2` : `200`, deux liens) et M20.
  Les réserves de `t16`/`t17` (ce qu'une course gagnée ne prouve pas) se recopient au doc-comment des tests.

### Les routes (`kesh-api`)

- **AC-5** — `GET /api/v1/reconciliation/transactions/{id}/entry-candidates`, rôle Comptable (groupe de
  `link-candidates`, `lib.rs:857-858`) ; réponse `200` `{ bankTransactionId, candidates: [{ journalEntryId,
  entryNumber, fiscalYearId, entryDate, journal, description, amount, daysApart, cancelBlockedBy, cancelBlockedLabel }]
  }`, `amount` chaîne à deux décimales. Ordre des refus **identique** au `POST` et à la 29-8 : `404 NOT_FOUND`
  (inexistante, autre société) → `link_preflight` (**réutilisé**, non recopié : archivé `BankAccountNotFound`, clos `409
  BANK_ACCOUNT_CLOSED`, non lié `412 BANK_ACCOUNT_NOT_CONFIGURED`) → `404 RECONCILIATION_TRANSACTION_NOT_PENDING`.
  Transaction nulle ou hors CHF → `200`, `candidates: []`.
  **`cancelBlockedBy`** *(validation P1, R6 ; réécrit à la validation P2, R2, F3, F4)* — le **code de fil** du premier
  motif qui, **au moment de la lecture**, empêcherait d'annuler ce rattachement, ou `null`, calculé par **la** fonction de
  blocage de l'annulation : `cancel_blocker` (`reconciliation_cancel.rs:232-270`) est scindée en une lecture par
  **écriture** (`entry_cancel_blocker(conn, company_id, entry_id, unlinking)` — classement puis bras client,
  fournisseur ou écriture propre) qu'elle appelle désormais, et que la liste appelle avec `unlinking = None`
  (l'écriture n'étant rapprochée d'aucune transaction, le rang 3 ne peut pas sortir). Aucune seconde précédence. Les
  codes sont ceux que rend `SettlementCancelBlocker::code()` (`crates/kesh-db/src/errors.rs:440-452`) —
  `FISCAL_YEAR_CLOSED`, `LETTERING_ALL_LINES_IN_CLOSED_PERIODS`, `ACCOUNT_ARCHIVED`, `FISCAL_YEAR_INVALID`,
  `INVOICE_WRITTEN_OFF`, `INVOICE_CREDITED` —, **jamais** les noms de variantes Rust (`DOCUMENT_LETTERING_IN_CLOSED_PERIODS`
  ou `NO_OPEN_FISCAL_YEAR_TODAY`, écrits à tort à la remédiation P1). Les trois codes fournisseur
  (`SUPPLIER_INVOICE_NOT_PAID`, `SUPPLIER_INVOICE_CANCELLED`, `SUPPLIER_INVOICE_IN_PAYMENT_BATCH`) ne peuvent pas sortir
  pour une candidate (le classement exige une facture `paid`, `reconciliation_cancel.rs` `SUPPLIER_SETTLEMENT_CLASSIFY_SQL`,
  et une facture payée n'est dans aucun lot `generated`) : le doc-comment le dit ; le type TS reste
  `ReconciliationCancelCode` (`reconciliation.types.ts:154-164`). **`cancelBlockedLabel`** : le numéro du compte
  archivé (rang 4), comme `GET /reconciliation/transactions/{id}` (`reconciliation.types.ts:186-187`), sans lequel
  « réactivez-le » ne dit pas lequel (F3). Tests : une écriture d'un exercice clôturé est **candidate** avec
  `cancelBlockedBy = "FISCAL_YEAR_CLOSED"` ; une écriture dont un compte est archivé, `"ACCOUNT_ARCHIVED"` et le numéro
  du compte ; une écriture d'exercice ouvert, `null`.
  *(Validation P3, R5.)* **Coût** : `entry_cancel_blocker` coûte, par candidate, le classement (deux lectures),
  l'exercice, le lettrage figé, `reversal_blockers` (trois sous-requêtes) et l'exercice du jour — de huit à douze
  requêtes ; pour 50 candidates, de l'ordre de 500 requêtes à chaque ouverture du dialogue. **Mesure** : sur la base de
  test `kesh_v0173a`, une transaction avec **50** écritures candidates (montage synthétique, jamais une base de Guy),
  `GET …/entry-candidates` en moins de **1 s** (trois mesures, écrites au Dev Agent Record). Au-delà : une section
  *Performance debt* (constat, chiffres, propriétaire) et une issue demandée à l'orchestrateur, avec pour remède nommé
  le calcul du motif **à la sélection** d'une candidate (une route de lecture par écriture) ou une lecture groupée.
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
  rapprochée, contre-passée, ou c'est un règlement de facture client à rattacher comme tel. » ; repli d'`errors.rs` égal
  mot pour mot au catalogue fr-CH ; quatre locales. La clé `reconciliation-errors-entry-not-linkable` s'**inscrit** à la
  liste fermée de la garde G9 de `crates/kesh-api/tests/textes_coherents.rs` (à côté de
  `("reconciliation-errors-settlement-not-linkable", 1, &[])`, `:805`) : sans cette ligne, G9 reste verte et le repli
  n'est contrôlé par personne. Mutation M24.
- **AC-8** — Tests HTTP (`crates/kesh-api/tests/reconciliation_link_entry_e2e.rs`, nouveau) : nominal par la liste puis
  le geste, pour (a) une écriture saisie par `POST /journal-entries`, (b) une facture fournisseur payée par `POST
  /supplier-invoices/{id}/pay` et *(validation P2, R10)* (c) un règlement client par **compte interne** dont la
  contrepartie est le compte de la banque ; chaque refus de l'AC-6 avec son code ; RBAC (Consultation `403` ; clé `read`
  `403 API_KEY_READ_ONLY` ; clé `read-write` `200` et trace qui nomme la clé) ; IDOR (transaction et écriture d'une autre
  société → `404`) ; puis les **suites** du rattachement, chacune assertée :
  1. la transaction ne compte plus au refus de clôture du compte bancaire (`POST /bank-accounts/{id}/close` passe quand
     c'était la dernière en attente) ;
  2. elle sort du contrôle K2 de la préparation de clôture (une route existante de la préparation, à citer) ;
  3. *Annuler le rapprochement* (`POST /reconciliation/transactions/{id}/cancel`) : pour (a), l'écriture saisie est
     **contre-passée** et la transaction redevient `pending` ; pour (b), le **règlement fournisseur est annulé** et la
     facture redevient `open` ; pour (c), le **règlement client est retiré** et la facture redevient due (C-v017-3-9 —
     le comportement existant, asserté pour qu'il ne change pas en silence) ;
  4. la contre-passation directe de l'écriture rattachée (`POST /journal-entries/{id}/reverse`) est refusée `409
     MATCHED_BANK_TRANSACTION` ; l'annulation du règlement fournisseur depuis la fiche, de même (le code existant de
     ce refus, à citer) ;
  5. une écriture saisie d'un **exercice clôturé**, rattachée : *Annuler le rapprochement* est refusé `409` avec son
     motif (`FISCAL_YEAR_CLOSED`), rien d'écrit, et ce motif est celui que la liste annonçait (`cancelBlockedBy`) — même
     fonction ; puis, l'exercice rouvert par un Administrateur, l'annulation passe : le motif était **daté**, non
     définitif (R2).
- **AC-9** — Registres. `audit_route_registry.rs` classe `POST …/link-entry` `Traced, SansEcritureAuJournal`, rejouée
  (point (vi) du doc-comment, « sans preuve dynamique » comme `link`) ; **toutes** les valeurs chiffrées du fichier se
  **recomptent depuis le registre** au moment du patch — valeurs de départ à `c5101b07` : `LIB_ROUTES.len(), 135`
  (`:909-911`), `traced, 129` (`:915`), `138` (`:944`), `sans_ecriture, 110` (`:1312`), `tout.len(), 138` (`:1324`), la
  prose `:33` (« plus large (138) ») et *(validation P2, R7)* la prose `:157` (« **dix-sept** routes
  `SansEcritureAuJournal` sont rejouées », à passer à dix-huit et à nommer la route). Le test
  `le_manuel_dit_le_compte_du_registre` (`:954-985`) exige que le **manuel d'administration** dise le compte : la
  phrase « 129 des 135 routes à verbe mutant … 129 + 4 + 2 = 135 » (`docs/manual/fr/admin-manual.tex:2966`) se récrit
  aux valeurs recomptées, et le PDF d'administration se régénère (**collision** avec la v016-2a, § *Collision*).
  `bank_account_closed_inventory.rs` inscrit l'`UPDATE` de `link_entry_in_tx` au `REGISTRE` en `Statut::Gardee(…)`
  (forme de `link_in_tx`, `:237`), l'ajoute à `FENETRE` (`:892`), recompte `assert_eq!(sites.len(), 33, …)` (`:519`) et
  son commentaire (`:516`), et *(validation P2, R7)* `gardes_non_verrouillantes(&sources).len(), 11` (`:1280-1284`,
  l'ensemble G : 11 → 12, l'appel qualifié de l'AC-3 l'y fait entrer) et son message. `audit_labels.rs` gagne
  `reconciliation.entry_linked`, **trié** (après `reconciliation.cancelled`, avant `reconciliation.linked`), libellé
  fr-CH « Transaction rattachée à une écriture » et quatre locales ; `tests/audit_label_registry.rs` vert.
  Les doc-comments qui comptent ou décrivent les chemins de `matched_entry_id` passent d'un chemin de plus et nomment
  celui-ci : `reconciliation_link.rs:16-23`, `reconciliation_cancel.rs:11-28` — y compris *(validation P2, R10)* `:17`
  (« Un seul rend un **règlement fournisseur** ») et `:27` (« Le seul chemin qui la rapproche est l'acceptation »),
  devenus faux — et la **définition** de l'« écriture propre » (« une écriture que seule la transaction possède »), qui se
  récrit : une écriture saisie au journal, rattachée par la 3a, est elle aussi contre-passée à l'annulation. Filet :
  `grep -rnE "chemins qui posent|sept chemins|cinq autres chemins|cinq chemins|six chemins|Les cinq chemins|seul chemin|Un seul rend" crates/`
  (rend aussi `reconciliation_e2e.rs:5274`, `tests/reconciliation_cancel.rs:3`, et `journal_entry_reversal_e2e.rs:552`,
  `:765` — autre notion, à écarter) ; tri au Dev Agent Record.

### L'écran (`frontend`)

- **AC-10** — *(Retirée à la validation P2 : l'avertissement des propositions est passé à la v017-3c.)*
- **AC-11** — *(Retirée à la validation P2 : la mesure de coût de l'avertissement est passée à la v017-3c.)*
- **AC-12** — *(Retirée à la validation P2 : passée à la v017-3c.)*
- **AC-13** — Le bouton « Rattacher à un règlement » devient **« Rattacher à une écriture existante »** (clé
  `reconciliation-link-button-label` récrite dans les quatre catalogues, `data-testid="link-settlement-button"` gardé),
  offert sur toute ligne en CHF de montant non nul (`canLink` récrit : miroir de `transaction_can_be_linked_to_entry`,
  et son commentaire). Le dialogue (`LinkSettlementModal.svelte`, généralisé, renommé s'il le faut — le dire) montre
  deux sections : « Règlements de factures clients » (crédits seulement ; la liste, les colonnes et l'écart de la 29-8
  inchangés) et « Écritures passées sur le compte de la banque » (n°, date, journal, libellé, montant, jours d'écart).
  **Un seul** choix parmi les deux, rien de présélectionné ; *Rattacher* appelle `link` ou `link-entry` selon la
  section. `reconciliation.api.ts` gagne `getEntryCandidates` et `linkEntry` ; `reconciliation.types.ts` leurs types.
  **Ce que le dialogue dit de l'annulation** *(réécrit à la validation P2, R2, F4, F3, D12)* — aucune promesse
  inconditionnelle :
  - écriture choisie, `cancelBlockedBy = null` : « Aujourd'hui, annuler ce rapprochement contre-passerait cette écriture
    (ou annulerait le règlement qu'elle porte), tant que son exercice reste ouvert. » ;
    *(validation P3, F6 — remplace la phrase ci-dessus)* « Aujourd'hui, Kesh ne voit aucun obstacle : annuler ce
    rapprochement contre-passerait cette écriture (ou annulerait le règlement qu'elle porte). Une clôture d'exercice, une
    période verrouillée ou un exercice postérieur clos pourraient l'empêcher plus tard. » ;
  - écriture choisie, `cancelBlockedBy ≠ null` : « Tant que la raison suivante subsiste, ce rattachement ne pourra pas
    être annulé : » suivi du texte **existant** du motif, rendu par `reconciliationCancelMessage(code, label)`
    (`reconciliation-cancel.ts:38`, **réutilisée**, DRY — elle dit déjà le moyen de lever le motif : rouvrir les
    exercices, réactiver le compte, créer l'exercice du jour, annuler le solde) ; *Rattacher* exige alors une case à
    cocher (`data-testid="entry-cancel-blocked-ack"`) qui reprend la même vérité datée ;
    *(validation P3, R4, F5 — corrige les deux points ci-dessus.)* **Gabarit** : une clé d'introduction neuve,
    `reconciliation-link-entry-cancel-blocked-intro` (« Ce rattachement ne pourra pas être annulé tant que dure l'obstacle
    suivant. »), **phrase complète**, puis le texte du motif dans un **paragraphe distinct** : ni second deux-points, ni
    majuscule parasite derrière un deux-points. La parenthèse « elle dit déjà le moyen de lever le motif » est
    **inexacte** pour deux codes : `INVOICE_CREDITED` (« … il ne s'annule pas » — aucun moyen ; état hérité d'avant la
    0.12.1, `invoice_settlements_write.rs:776-785`, rare) et `LETTERING_ALL_LINES_IN_CLOSED_PERIODS` (moyen long, que le
    texte décrit) ; pour `INVOICE_CREDITED`, l'introduction reste vraie (« tant que dure »), et L9 le dit ;
  - *(validation P2, F14)* la section des règlements de factures clients ne porte pas ce calcul (elle reste celle de la
    29-8) : l'asymétrie se dit au manuel (AC-18).
  Le code `RECONCILIATION_ENTRY_NOT_LINKABLE` entre dans l'ensemble `RELIRE_SUR` (`LinkSettlementModal.svelte:53-57`) —
  le dialogue le dit et relit les deux listes, comme le manuel le promet (`user-manual.tex:2388` — *citation `:2390`
  corrigée à la validation P2, F12*) ; Vitest du cas « course » calqué sur `LinkSettlementModal.test.ts`.
  `data-testid` de la section écritures : `entry-candidate-row`, `entry-candidate-radio` (`data-entry-id`),
  `entry-candidate-cancel-blocked`, `entry-cancel-blocked-ack`.
  **Listes vides** *(validation P2, F16)* : les deux vides → le message existant de la 29-8 (`link-empty`), récrit pour
  les deux sections ; une seule vide → la section vide porte sa propre phrase (`link-settlements-empty` ou
  `link-entries-empty`), l'autre sa liste ; `link-no-entry` (« aucune écriture », que l'E2E existant asserte,
  `reconciliation-link.spec.ts:179`) garde son sens — le rappel que le rattachement ne passe aucune écriture — et n'est
  pas confondu avec ces testids. Vitest des trois cas.
  *(Validation P3, F3 — remplace les cas ci-dessus : ils sont **quatre**.)*
  1. **crédit, deux listes vides** → `link-empty`, récrit : « Aucun règlement de facture client ni aucune écriture du
     compte de la banque ne correspond à cette transaction. Enregistrez d'abord le règlement depuis la fiche de la
     facture, ou saisissez l'écriture, puis revenez ici. » ;
  2. **crédit, règlements vides, écritures non vides** → la section règlements porte `link-settlements-empty` (« Aucun
     règlement de facture client ne correspond. »), la section écritures sa liste ;
  3. **crédit, écritures vides, règlements non vides** → la section écritures porte `link-entries-empty` (« Aucune
     écriture du compte de la banque ne correspond. »), la section règlements sa liste ;
  4. **débit** (une seule section) **sans candidate** → `link-entries-empty-debit` : « Aucune écriture du compte de la
     banque ne correspond à ce débit. S'il est déjà comptabilisé par un autre chemin, rejetez-le plutôt que de
     l'affecter. » — le cas que l'utilisateur verra pour chaque débit réellement non comptabilisé ; aucune mention de
     règlement client.
  Vitest des quatre cas, texte asserté ; captures des quatre (AC-16).
  **Tests et clés existants à récrire**, nommés : `ReconciliationProposals.test.ts:1030-1038` (le bouton sur un débit :
  attendu `['1','2']` pour `100.03` et `-50.00`, toujours absent pour `0.00` et l'EUR) ; `reconciliation-link.spec.ts:177`
  (inchangé : un seul bouton par ligne) ; les clés `reconciliation-link-no-entry` (`messages.ftl:1397`), `-modal-title`,
  `-empty` ; *(validation P2, F11)* le compteur `sitesTotal` de `frontend/src/lib/shared/i18n-keys.test.ts` (`:653` et
  son historique daté), recompté par `grep -o "i18nMsg("` aux deux bornes.
  *(Validation P3, F7 : la constante est à `:767`, `sitesTotal: 2619` ; `:653` est un commentaire d'historique daté.)*
  *(Validation P3, F2.)* **`frontend/src/lib/features/reconciliation/LinkSettlementModal.test.ts`**, le fichier le plus
  touché : le mock (`:12-15`, `getLinkCandidates` et `linkTransaction` seulement) gagne `getEntryCandidates` et
  `linkEntry` ; l'attente de `link-empty` (`:123`, « Enregistrez d'abord le règlement depuis la fiche ») suit le texte
  récrit ; la relecture sur course (`:216`, `getLinkCandidates` appelé deux fois) relit désormais **les deux** listes ; le
  test des replis (`:357-370`), qui lit `LinkSettlementModal.svelte` **par son nom** et ne contrôle que les clés
  préfixées `reconciliation-link-`, suit le composant s'il est renommé, et toutes les clés neuves portent ce préfixe.
  *(F10.)* Les commentaires de `ReconciliationProposals.svelte:21` (« une ligne de crédit en CHF porte un troisième
  bouton, « Rattacher à un règlement » »), `:76` et `:577-578` (« lignes de crédit en CHF seulement ») se récrivent avec
  celui de `canLink`.
- **AC-14** — *(Retirée à la validation P2 : la confirmation avant *Accepter*, *Affecter*, *Éclater* est passée à la
  v017-3c.)*
- **AC-15** — i18n : toutes les clés nouvelles ou récrites dans les **quatre** catalogues
  (`crates/kesh-i18n/locales/{fr,de,it,en}-CH/messages.ftl`), aucun texte en dur ; `npm run lint-i18n-ownership` vert ;
  *(validation P2, F10)* la clé `reconciliation-labels-more-capped` (`messages.ftl:1310`, « … éclatées ou rattachées à un
  règlement ») récrite (« … ou rattachées à une écriture existante ») dans les quatre catalogues.
- **AC-16** — Recette visuelle (thème clair, base de démonstration `kesh_v0173a`) : le dialogue avec ses deux sections
  (un crédit qui a un règlement client et une écriture) ; le dialogue d'un débit (une seule section) ; un crédit sans
  règlement mais avec une écriture ; les deux listes vides ; une liste vide sur deux ; une écriture choisie avec la
  phrase datée de contre-passation ; une écriture d'exercice clôturé avec sa case ; l'erreur de course et la
  relecture ; la ligne disparue après rattachement. *(Validation P3, F3, F9.)* S'ajoutent : les **quatre** cas de
  listes vides (dont le débit sans candidate) ; un motif `ACCOUNT_ARCHIVED` avec le numéro du compte
  (`cancelBlockedLabel`) ; la page de propositions avec le bouton **renommé** sur une ligne de débit. Captures dans `/home/gcorbaz/devel/kesh-gate-logs/v0173a/captures/`,
  listées au Dev Agent Record et jointes au prompt de revue.

### Le parcours (Playwright)

- **AC-17** — `frontend/tests/e2e/reconciliation-link-entry.spec.ts` : un Comptable (montants **uniques** par
  exécution, base partagée entre specs) saisit par l'API une écriture `D charge / C banque` d'un montant unique,
  importe un relevé dont un **débit** l'égale — *(validation P2, F13)* par le patron de
  `frontend/tests/e2e/reconciliation-supplier.spec.ts:116-123` (remplacement de `<Amt Ccy="CHF">1234.56</Amt>` **et** de
  `<CdtDbtInd>`, import multipart), non par celui de `reconciliation-amount-due.spec.ts`, qui ne produit qu'un crédit —,
  ouvre `/reconciliation`, ouvre le dialogue de la ligne, choisit l'écriture, *Rattacher* ; la ligne disparaît ;
  `/audit-log` filtré sur « Transaction rattachée à une écriture » la retrouve.

### Les textes

- **AC-18** — Manuel utilisateur FR (`docs/manual/fr/user-manual.tex`, PDF régénéré). La sous-section
  `\label{sec:rattacher-reglement}` (`:2384-2412`) devient *Rattacher une transaction à une écriture déjà passée* (label
  gardé, ou renommé avec chaque `\ref` suivi : `grep -rn "rattacher-reglement" docs`) : quand (les chemins couverts), le
  geste et les deux sections, la règle (montant exact, compte du grand livre actuel, non rapprochée, non contre-passée),
  aucune écriture, **défaire** (contre-passe l'écriture saisie, annule le règlement — #672 — **tant que** le motif
  d'annulation n'est pas présent ; un motif présent se lève par le geste que le dialogue nomme), l'asymétrie avec la
  section des règlements clients (F14), une transaction **rejetée** (*Annuler le rejet* d'abord), et la consigne qui
  reste vraie en 0.17.0 : une transaction déjà comptabilisée qu'on ne rattache pas se **rejette**, elle ne s'affecte
  pas (l'avertissement automatique est l'objet de la v017-3c). **Liste close des passages à récrire** (numéros de
  `c5101b07`) :
  1. § *Régler une facture fournisseur*, `:1634-1640` — « Ne payez pas depuis la fiche une facture dont le débit est (ou
     sera) au relevé : Kesh ne sait pas encore rapprocher ce débit du paiement enregistré » : devient « payée depuis la
     fiche, la facture se rattache ensuite à la transaction » ; le processus « saisir ouverte, accepter à la
     réconciliation » reste le plus direct ;
  2. l'**encadré** `keshwarning` « Dépense déjà réglée par un autre chemin », `:2324-2343` — récrit en entier : une
     dépense payée depuis la fiche, par un lot d'**une** facture ou par une écriture saisie **se rattache** (même dans un
     exercice clôturé, avec la vérité datée de D12) ; « laissez la transaction en attente (issue #671) » disparaît ;
     restent en attente, avec la consigne de **rejeter**, les cas de D9, chacun renvoyé à **son** issue (lot de plusieurs
     factures et second côté d'un virement : #823 ; ancien compte et autre compte bancaire : l'issue demandée ; frais de
     rappel : #401) ;
  3. § des règles d'affectation, `:2450` — « Une dépense déjà réglée … ne la masque pas : voyez l'encadré » et
     « *Affecter manuellement* ne convient que si … » : renvoi au rattachement ;
  4. diagnostic, `:3337-3338` — « n'acceptez rien pour ce débit (encadré …) » : « rattachez-la (§…) » ;
  5. la liste « Ce qui ne se rattache pas », `:2402-2409`, puce par puce : `:2402` « (issue #671) » → les issues de
     chaque puce ; `:2404` (plus de 5 centimes) **gardée**, précisée « pour un règlement de facture client ; une écriture
     se rattache au montant exact » ; `:2405` (autre compte bancaire) **gardée**, renvoi à l'issue demandée ; `:2406`
     (compte interne) **supprimée** — un règlement par compte interne sur le compte de la banque se rattache désormais
     (AC-2.7) ; `:2407` (plusieurs factures) **gardée**, renvoi à #823 ; `:2408` **scindée** — l'écriture saisie au
     journal se rattache, l'ancien compte reste (issue demandée) ; `:2409` (un débit) **supprimée** ; ajoutées : le
     second côté d'un virement entre comptes propres (#823) et le règlement fournisseur sous le centime (renvoi à
     `sec:facture-fournisseur-sous-centime`, v017-3b) ;
  6. § *Annuler un rapprochement*, `:2462-2484` : une transaction rattachée à une **écriture saisie** — l'annulation la
     **contre-passe** ; à un **règlement fournisseur** — le règlement est annulé, la facture redevient ouverte, geste
     offert depuis le détail de l'import (`:2467-2470`) ; à un **règlement client par compte interne** — le règlement est
     retiré (R10) ; un motif d'annulation présent (exercice clôturé…) se lève d'abord ; l'introduction couvre le
     **rattachement** erroné ;
  7. `:1961-1963` (issues d'une transaction rejetée qui bloque la clôture du compte) ; `:2357` et *(validation P2, F1)*
     son jumeau `:2363` (« …rattachée à un règlement déjà enregistré ») : « ou rattachée à une écriture existante » ;
  8. *(validation P2, F1)* `:2292` (introduction de la § *Réconciliation* : « sauf dans un cas : une transaction dont le
     règlement a déjà été enregistré depuis la fiche … se rattache à ce règlement ») : toute écriture déjà passée ;
  9. *(F1)* `:3483` (glossaire, *Réconciliation bancaire*) : même correction ;
  10. *(F1)* `:264-272` (tuile, écart de relevé) : « une écriture saisie à la main pour un mouvement bancaire, à une autre
      date, apparaît dans l'écart — rien ne la relie au relevé » (`:272`) devient faux : une écriture saisie **rattachée**
      est corrigée comme les autres ; seule une écriture saisie **non rattachée** reste dans l'écart ;
  11. *(F1)* `:2306` (lot en cours : « la confirmation enregistre son propre paiement, qui resterait sans transaction ») :
      vrai pour un lot de plusieurs factures débité en une ligne (#823), faux pour un lot d'**une** facture, qui se
      rattache ;
  12. *(F1)* `:1403` (frais de rappel : « (issues \#671 et \#401) ») : « (issue \#401) ».
  13. *(Validation P3, R2 = F1)* `:2300` (« … puis rattachez la transaction à ce règlement (\emph{Rattacher à un
      règlement}, …) ») : le nom du bouton devient « Rattacher à une écriture existante » ;
  14. *(R2 = F1)* `:2354` (« une ligne de crédit en CHF se rattache aussi au règlement déjà saisi depuis la fiche
      (\emph{Rattacher à un règlement}, …) ») : toute ligne en CHF de montant non nul se rattache à une écriture déjà
      passée (règlement saisi depuis une fiche, écriture saisie au journal) ; nom du bouton ;
  15. *(R2 = F1)* `:2366` (« ou, pour un crédit en CHF dont le règlement a déjà été saisi depuis la fiche de la facture,
      \emph{Rattacher à un règlement} ») : « ou, pour une transaction déjà comptabilisée par un autre chemin, \emph{Rattacher à
      une écriture existante} » ;
  16. *(R2 = F1)* `:3337` : outre « n'acceptez rien » (point 4), la phrase « Puis rattachez la transaction à ce règlement
      (\emph{Rattacher à un règlement} sur sa ligne, …) » prend le nom du bouton.
  Le nom du bouton change donc à `:2300`, `:2354`, `:2366`, `:2388`, `:3337` ; le titre de la sous-section (`:2384`) suit
  l'AC-18 ; le libellé d'audit cité à `:2394` (« Transaction rattachée à un règlement ») reste vrai pour la 29-8 et gagne
  celui de la 3a (« Transaction rattachée à une écriture »).
  **Le filet**, au `.tex` **et** au PDF aplati (`pdftotext docs/manual/fr/user-manual.pdf - | tr '\n' ' ' | tr -s ' '`),
  *(validation P2, F2 : élargi, les fragments de la P1 étaient muets sur `rattachée`, `rattache à ce règlement`,
  `issues \#671`)* : le fragment **`rattach`** (environ soixante lignes, triées à la main, sort de chacune au Dev Agent
  Record — *validation P3, F10 : 44 lignes au `.tex`, 45 fragments au PDF aplati ; filet exécuté, sortie ci-dessous*),
  et *(R2 = F1)* `Rattacher à`, `crédit en CHF`, **`\#671`** nu (et `#671` au PDF), `déjà réglée`, `sans transaction`, `ne sait pas encore`, `acceptez rien`,
  `relie au relevé`, `sans écriture` ; les deux apostrophes (`'`, `’`) pour tout fragment qui en porte. Les manuels
  DE/IT/EN ne sont pas traduits dans cette version. *(F7.)* Chaque renvoi à un reste pointe **son** issue ; `#671` ne
  reste que dans l'historique du CHANGELOG.
  **Filet `rattach` exécuté à la validation P3** (sur `c5101b07`, sortie copiée ; le tri à la main se refait au
  développement et va au Dev Agent Record). Commande `.tex` : `grep -n -i "rattach" docs/manual/fr/user-manual.tex` —
  **44 lignes** ; sort de chacune :

  | Lignes | Sort |
  |---|---|
  | `:264`, `:265`, `:270` | point 10 (tuile) |
  | `:689`, `:1425`, `:1430`, `:1713`, `:1898`, `:2485`, `:2630`, `:2938`, `:3229`, `:3238`, `:3486` | sans rapport (transitoire, lettrage d'un avoir, paiement à rattacher à une facture, pièce, projet, MCP, sous-compte) |
  | `:1293`, `:2302`, `:2320`, `:2382` | inchangées : « rattachez la transaction à ce règlement » reste vrai pour un règlement client (29-8) ; leur `\ref` suit le label |
  | `:1403` | point 12 |
  | `:1962`, `:1963`, `:2357`, `:2363` | point 7 |
  | `:2292` | point 8 |
  | `:2300` | point 13 |
  | `:2354` | point 14 |
  | `:2366` | point 15 |
  | `:2384`–`:2409` | la sous-section récrite (corps de l'AC-18) et point 5 |
  | `:2450` | point 3 |
  | `:2464`, `:2465`, `:2478`, `:2479` | point 6 |
  | `:3337` | points 4 et 16 |
  | `:3483` | point 9 |

  <details><summary>Sortie `.tex` (44 lignes, extraits autour de « rattach »)</summary>

  ```
  264: … transaction rattachée à un règlement saisi depuis la fiche de la facture…
  265: … (\S\ref{sec rattacher-reglement}) garde la date du règlement, et la correction s'y appliq…
  270: …textbf{issue d'un rapprochement ou d'un rattachement} est ainsi corrigée   une…
  689: … Une \textbf{transitoire} rattache à l'exercice qui se termine ce qui lui appartient mais …
  1293: … seul. Si le virement est déjà importé, rattachez ensuite la transaction au règlement (\S\…
  1403: …nt de plus de 5 centimes, \textbf{ne se rattache pas} à lui (\S\ref{sec rattacher-reglemen…
  1425: …cte, hors la marque de lettrage, qui la rattache à l'avoir (principe d'intangibilité compt…
  1430: …cte, hors la marque de lettrage, qui la rattache à l'avoir).…
  1713: … doit ou qui attend d'être rattachée à la bonne facture. Ces paiements se retrouvent parmi…
  1898: …cument}. Un règlement annulé n'est plus rattaché à la facture, pas plus que celui d'une…
  1962: …           rapprochée manuellement, ou rattachée au règlement saisi depuis la fiche de la…
  1963: …           facture (\S\ref{sec rattacher-reglement}), ou que son import n'est pas supprimé…
  2292: …epuis la fiche de la facture se \textbf{rattache} à ce règlement, sans aucune écriture (\S…
  2300: …ef{sec reglement-client}), \textbf{puis rattachez la transaction à ce règlement} (\emph{Ra…
  2302: …l'échéance et le dernier rappel) --- en rattachant ensuite la transaction à ce règlement (…
  2320: …ue (\S\ref{sec reglement-client}), puis rattachez la transaction à ce règlement (\S\ref{se…
  2354: …clater}~; une ligne de crédit en CHF se rattache aussi au règlement déjà saisi depuis la f…
  2357: …es, affectées manuellement, éclatées ou rattachées à un règlement.…
  2363: …ptée, affectée manuellement, éclatée ou rattachée à un règlement déjà enregistré (\S\ref{s…
  2366: …si depuis la fiche de la facture, \emph{Rattacher à un règlement} (\S\ref{sec rattacher-re…
  2382: …re (\S\ref{sec reglement-client}), puis rattachez-y la transaction (\S\ref{sec rattacher-r…
  2384: … \subsection{Rattacher une transaction à un règlement déjà enregistré}\label{sec rattacher…
  2386: …trop, imputation choisie à la main). Le rattachement relie la transaction à ce règlement~ …
  2388: …} Sur la ligne de la transaction, \emph{Rattacher à un règlement} (lignes de crédit en CHF…
  2394: … \textbf{Aucune écriture.} Le rattachement ne passe aucune écriture, aucun règlement, aucu…
  2396: … \textbf{Une transaction rejetée} se rattache par la même voie~  \emph{Annulez d'abord le …
  2398: …{Un règlement d'un exercice clôturé} se rattache~  un paiement de décembre rapproché en ja…
  2400: … \textbf{Défaire un rattachement.} \emph{Annuler le rapprochement} (\S\ref{sec annuler-rap…
  2402: … \textbf{Ce qui ne se rattache pas} (issue \#671)~ …
  2407: …istrées séparément~  une transaction se rattache à un seul règlement~;…
  2409: …urnisseur, ou un lot de paiement, ne se rattache pas --- saisissez plutôt la facture fourn…
  2450: …puis la fiche de la facture client puis rattachez-y la transaction (\S\ref{sec rattacher-r…
  2464: … client}, ou qu'une transaction a été rattachée à son règlement…
  2465: … (\S\ref{sec rattacher-reglement}), le même bouton apparaît aussi sur la fiche de la factu…
  2478: …en va de même d'une transaction \textbf{rattachée} à un règlement…
  2479: …      saisi depuis la fiche (\S\ref{sec rattacher-reglement})~  le règlement est retiré…
  2485: …           prête à être rattachée à la bonne pièce.…
  2630: …sans pièce} --- le règlement n'est plus rattaché…
  2938: … Kesh permet de rattacher chaque opération comptable à un \textbf{projet} pour l'analyser…
  3229: …dministration, § \emph{Le serveur MCP}) rattachés à la société. Les numéros…
  3238: … nomme les utilisateurs rattachés, les clés d'API et les clés du serveur MCP…
  3337: …ait due, sans que rien le signale. Puis rattachez la transaction à ce règlement (\emph{Rat…
  3483: … saisi depuis la fiche de la facture se rattache à ce règlement, sans écriture.…
  3486: …s-compte (auxiliaire)] Compte de détail rattaché à un compte principal (ex. 1100.1 pour un…
  ```

  </details>

  Commande PDF : `pdftotext docs/manual/fr/user-manual.pdf - | tr '\n' ' ' | tr -s ' ' | grep -oi ".\{40\}rattach.\{40\}"`
  — **45 fragments** (le sommaire et la sous-section 14.3 en ajoutent un par rapport au `.tex`) ; aucun ne sort de la
  table ci-dessus.

  <details><summary>Sortie PDF (45 fragments)</summary>

  ```
  79 14.2 Réconciliation manuelle 81 14.3 Rattacher une transaction à un règlement déjà e
  t à la date de valeur — une transaction rattachée à un règlement saisi depuis la fiche 
  riture issue d’un rapprochement ou d’un rattachement est ainsi corrigée : une écriture 
  oires de fin d’exercice Une transitoire rattache à l’exercice qui se termine ce qui lui
   seul. Si le virement est déjà importé, rattachez ensuite la transaction au règlement (
   règlement de plus de 5 centimes, ne se rattache pas à lui (§14.3) : elle reste en atte
  cte, hors la marque de lettrage, qui la rattache à l’avoir (principe d’intangibilité co
  cte, hors la marque de lettrage, qui la rattache à l’avoir). L’avoir est consultable da
  rnisseur vous doit ou qui attend d’être rattachée à la bonne facture. Ces paiements se 
  ocument. Un règlement annulé n’est plus rattaché à la facture, pas plus que celui d’une
  e n’est pas rapprochée manuellement, ou rattachée au règlement saisi depuis la fiche de
  gistré depuis la fiche de la facture se rattache à ce règlement, sans aucune écriture (
  is la fiche de la facture (§10.9), puis rattachez la transaction à ce règlement (Rattac
   dû depuis la fiche de la facture, puis rattachez la transaction à ce règlement (§14.3)
  s l’échéance et le dernier rappel) — en rattachant ensuite la transaction à ce règlemen
  iche de la facture voulue (§10.9), puis rattachez la transaction à ce règlement (§14.3)
  Éclater ; une ligne de crédit en CHF se rattache aussi au règlement déjà saisi depuis l
  es, affectées manuellement, éclatées ou rattachées à un règlement. Cochez les transacti
  ptée, affectée manuellement, éclatée ou rattachée à un règlement déjà enregistré (§14.3
  té saisi depuis la fiche de la facture, Rattacher à un règlement (§14.3). © Équipe Kesh
  is la fiche de la facture (§10.9), puis rattachez-y la transaction (§14.3). © Équipe Ke
  éconciliation bancaire Kesh 0.16.1 14.3 Rattacher une transaction à un règlement déjà e
  trop, imputation choisie à la main). Le rattachement relie la transaction à ce règlemen
   geste. Sur la ligne de la transaction, Rattacher à un règlement (lignes de crédit en C
  ice : cochez celui qui correspond, puis Rattacher. Le bilan dit « 1 opération(s) réussi
  c — règlement annulé depuis la fiche ou rattaché entre-temps, transaction rapprochée ai
  h le dit et relit la liste ; rien n’est rattaché. Les conditions. Le règlement doit êtr
  écriture manuelle). Aucune écriture. Le rattachement ne passe aucune écriture, aucun rè
  inscrit au journal d’audit (Transaction rattachée à un règlement). Une transaction reje
  . Un règlement d’un exercice clôturé se rattache : un paiement de décembre rapproché en
  un exercice clôturé (§14.6). Défaire un rattachement. Annuler le rapprochement (§14.6) 
  ation bancaire Kesh 0.16.1 Ce qui ne se rattache pas (issue #671) : • un paiement qui d
  istrées séparément : une transaction se rattache à un seul règlement ; • une transactio
  urnisseur, ou un lot de paiement, ne se rattache pas — saisissez plutôt la facture four
  puis la fiche de la facture client puis rattachez-y la transaction (§14.3). Affecter ma
  ure client, ou qu’une transaction a été rattachée à son règlement (§14.3), le même bout
  nt). Il en va de même d’une transaction rattachée à un règlement saisi depuis la fiche 
  ît dans la réconciliation, prête à être rattachée à la bonne pièce. Le journal d’audit 
  es sans pièce — le règlement n’est plus rattaché à sa facture — et sont de nouveau prop
  té analytique par projet Kesh permet de rattacher chaque opération comptable à un proje
  uel d’administration, § Le serveur MCP) rattachés à la société. Les numéros des entrées
  ctif. Son détail nomme les utilisateurs rattachés, les clés d’API et les clés du serveu
  ait due, sans que rien le signale. Puis rattachez la transaction à ce règlement (Rattac
   saisi depuis la fiche de la facture se rattache à ce règlement, sans écriture. © Équip
  us-compte (auxiliaire) Compte de détail rattaché à un compte principal (ex. 1100.1 pour
  ```

  </details>
- **AC-19** — `docs/api-external.md` : les deux routes (après `:675-677`) avec `cancelBlockedBy` et `cancelBlockedLabel`,
  le code `RECONCILIATION_ENTRY_NOT_LINKABLE` au tableau des codes (après `:868`), la route ajoutée à la liste des routes
  rejouées (`:878`) ; `:626` (« ⚠️ Dépense déjà réglée par un autre chemin (angle mort, #671) … Laisser la transaction
  en attente jusqu'au rattachement (#671) ») récrit comme l'encadré du manuel ; *(validation P2, F8)* `:243` (liste des
  routes refusées sur compte clos, « rattachement d'une transaction à un règlement et liste de ses candidats ») gagne les
  deux routes neuves ; « *(Depuis la v0.17.0.)* ». Filet `grep -n "#671\|jusqu'au rattachement\|rattach" docs/api-external.md`.
  *(Le champ `alreadyBooked` des propositions n'y entre pas : v017-3c.)*
- **AC-20** — `CHANGELOG.md` : section **Corrigé** *(validation P3, F8 : **Ajouté**, comme le précédent de la 29-8,
  `CHANGELOG.md:343-346` — deux routes, un code d'erreur, un bouton renommé ; l'entrée dit aussi que les transactions
  restées en attente se règlent désormais par ce geste)*, une entrée (`refs` [#671] par un lien, sans mot-clé de fermeture)
  qui dit le rattachement à une écriture et ce qui reste (D9, avec les issues). *(Réécrit à la validation P2, F15.)* À
  `c5101b07`, la tête du journal est `## [0.16.1] — 2026-10-29` ; la **v016-2a la renomme** `## [0.17.0] — Non publié`
  (sa fiche, AC-29 point 1). Dans cette branche, l'entrée s'écrit sous un en-tête **provisoire** `## [0.17.0] — Non
  publié`, placé au-dessus de `[0.16.1]` : à la réunion, il y aura **deux** sections `[0.17.0]` — l'orchestrateur les
  fusionne en une. Aucune version touchée. *(F9.)* § *Passer à la 0.17.0* du manuel d'administration (renommé par la
  v016-2a depuis `sec:maj-0-16-1`, `admin-manual.tex:2481`) : sur le précédent de la 29-8 (`admin-manual.tex:2563`,
  « Sont **nouveaux** : … `link-candidates` et `POST …/link` »), un paragraphe *Intégrations par l'API* y nomme les deux
  routes et le code neufs. Dans cette branche, il s'écrit dans la section telle qu'elle est (`sec:maj-0-16-1`) ; la
  réunion le reporte dans la section renommée.
- **AC-20-bis** — *(Validation P2, F10.)* `README.md:41`, *Fonctionnalités* : « rattachement d'une transaction au
  règlement déjà saisi depuis la fiche de la facture » devient « rattachement d'une transaction à une écriture déjà
  passée (règlement saisi depuis une fiche, écriture saisie au journal) ».
  *(Validation P3, F7 : la ligne porte en réalité `**rattachement d'une transaction au règlement** déjà saisi depuis la
  fiche de la facture, sans nouvelle écriture` — gras fermé au milieu. Le remplacement garde le balisage et « sans
  nouvelle écriture », vrai de la 3a aussi : `**rattachement d'une transaction à une écriture déjà passée** (règlement
  saisi depuis une fiche, écriture saisie au journal), sans nouvelle écriture`. Collision avec la v016-2b, § *Collision*.)*

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
3. `kesh-db` : règle, lecture, geste, scission de `cancel_blocker` (AC-1 à AC-4).
4. `kesh-api` : erreur, routes, montage, registres (AC-5 à AC-9).
5. `frontend` : types, API, dialogue, i18n (AC-13, AC-15) ; Vitest.
6. E2E (AC-17) ; captures (AC-16).
7. Textes (AC-18 à AC-20-bis) ; PDF utilisateur et d'administration régénérés (`make fr` dans `docs/manual/`, sous
   verrou et `mem-guard`) ; **greper la valeur** de chaque phrase corrigée.
8. Gates, mutations, Dev Agent Record.

## Tests et mutations prévus

**Rouges d'abord** : AC-2 (au moins 1, 2, 6, 7, 8), AC-4 nominal et rejetée, AC-8 nominal (a), (b) et (c), Vitest de
l'AC-13. Un test qui ne compile pas parce que la fonction n'existe pas compte comme rouge si l'erreur la nomme.

| # | Mutation (une à la fois, motif vérifié unique, restauration `git checkout -- f && touch f`) | Doit rougir |
|---|---|---|
| M1 | condition 3 : `m.movement = ?` → `ABS(m.movement) = ABS(?)` | AC-2.2 (signe) |
| M2 | *(Validation P1, R4 : mutant équivalent, retiré.)* | — |
| M2bis | table dérivée : filtre `l.account_id = ?` retiré | *(Corrigé à la validation P2, R5.)* AC-2.1, AC-2.2 : la somme porte alors sur **toutes** les lignes d'une écriture équilibrée, `M = 0`, aucune candidate attendue ne sort |
| M3 | condition 4 retirée | AC-2.5, AC-4 (second rattachement) |
| M4 | condition 5 : le `NOT EXISTS` de la contre-passée retiré | AC-2.6 |
| M5 | condition 5 : `reverses_entry_id IS NULL` retiré | AC-2.6 (transaction de montant opposé) |
| M6 | condition 6 retirée | AC-2.7 (règlement client par virement absent) |
| M7 | *(Validation P2, R4 : `settlement_type = 'bank_transfer'` retiré est **équivalent** — la contrainte `chk_invoice_settlements_counterparty`, `20261001000004_invoice_settlements_write_off.sql:44-54`, ne laisse `settlement_bank_account_id` non nul qu'à `bank_transfer`. Remplacée par M7bis.)* | — |
| M7bis | condition 6 : `s.settlement_bank_account_id = t.bank_account_id` retiré | AC-2.7 (un règlement client par virement sur un **autre** compte bancaire, au même montant — fixture : deux comptes bancaires partageant le compte du grand livre, AC-2.12 — devient absent alors qu'il doit être présent) |
| M8 | `transaction_can_be_linked_to_entry` : `amount != 0` → `amount > 0` | AC-2.1, AC-8 |
| M9 | tri : `ORDER BY e.id` seul | AC-2.10 |
| M10 | 4.3 déplacé avant 4.2 (lecture ordinaire avant le verrou de l'écriture) | **T-race-3** (*validation P2, R3 : tuée, comme la M14 de la 29-8 l'a été par T17-bis — `29-8-transaction-reglee-ailleurs.md:1072` ; l'exception de survie de la P1 est retirée*) |
| M11 | `UPDATE` sans `auto_match_rejected_at = NULL` | AC-4 (rejetée) |
| M12 | trace : action `reconciliation.linked` | AC-4, AC-9 |
| M13 | *(Passée à la v017-3c.)* | — |
| M14 | écran : `canLink` réservé aux crédits | Vitest AC-13 |
| M15 | *(Passée à la v017-3c.)* | — |
| M16 | écran : *Rattacher* appelle `link` pour une écriture | Vitest AC-13 |
| M17 | geste : restriction `AND e.id = ?` retirée (la première candidate est rattachée) | AC-4 (deux candidates, la seconde désignée) |
| M18 | règle : `e.company_id = ?` retiré | AC-2.9 (*validation P2, R4 : fixture SQL directe, seule à le tuer*) |
| M19 | règle : `LIMIT 50` retiré | AC-2.10 |
| M20 | geste : `FOR UPDATE` de l'étape 4.2 retiré | T-race-3 (le second n'attend plus) — *validation P3, R3 : T-race-1 ne le tue pas (`with_account_lock` sérialise deux gestes du même compte ; `reconciliation_link_e2e.rs:1344-1346`)* |
| M21 | *(Passée à la v017-3c.)* | — |
| M22 | liste : `cancelBlockedBy` toujours `null` | AC-5 (exercice clôturé), AC-8.5 |
| M23 | dialogue : `RECONCILIATION_ENTRY_NOT_LINKABLE` absent de `RELIRE_SUR` | Vitest AC-13 (course) |
| M24 | repli d'`errors.rs` altéré d'un mot | G9 (`textes_coherents.rs`, AC-7) |
| M25 | dialogue : phrase « tant que … subsiste » remplacée par une promesse inconditionnelle de contre-passation | Vitest AC-13 |
| M26 | liste : `cancelBlockedLabel` toujours `null` | AC-5 (compte archivé) |
| M27 | `transaction_can_be_linked_to_entry` : condition « en CHF » retirée | AC-2.11 (EUR → liste vide) — *validation P3, R6* |
| M28 | geste : étape 4.3 (`ensure_open_in_tx`) retirée | AC-4 (compte clos dans la transaction) — *validation P3, R6* |
| M29 | dialogue : *Rattacher* actif sans la case `entry-cancel-blocked-ack` | Vitest AC-13 — *validation P3, F9* |
| M30 | dialogue : première candidate présélectionnée | Vitest AC-13 (rien de présélectionné) — *validation P3, F9* |

Chaque mutation : le test qui rougit et la raison, au Dev Agent Record. Une mutation **survivante** est un défaut de test
à corriger.

## Gates

`kesh-db` est touché (dépôt) : **gate complet même en cours de boucle** (`CLAUDE.md` § *Exception kesh-db*). Base de
gate remise à zéro avant chaque gate complet. Au **dernier commit de code** : `scripts/test-fast.sh --ci` sous
`flock /home/gcorbaz/devel/kesh-gate-logs/.gate.lock`, `cargo build --workspace --all-targets`, Vitest complet, `npm run
check`, `npm run lint-i18n-ownership`, `npm run build`, et la **suite E2E complète** selon `docs/testing.md` à la lettre
(dossiers propres au worktree passés au backend **et** au runner), échecs jugés fichier par fichier contre `docs/testing.md`
§ *Les échecs attendus*. Ne déclarer que ce qui a tourné, chiffres recomptés.

## Ce que les gardes ne voient pas (angles morts assumés)

- **L1 — Une transaction pour plusieurs écritures** : lot pain.001 de plusieurs factures débité en une ligne, paiement
  groupé de factures réglées séparément. Aucune écriture n'égale la somme ; la transaction reste en attente. **#823**.
- **L2 — Virement entre deux comptes bancaires propres** : une écriture `D banque B / C banque A` correspond à **deux**
  transactions (une par compte). L'invariant « une écriture, une transaction » ne laisse rattacher qu'**un** côté ;
  l'autre reste en attente, **sans avertissement** dans la 0.17.0 (l'avertissement est l'objet de la v017-3c) : le
  manuel dit de le **rejeter**. **#823**.
- **L3 — Ancien compte du grand livre, autre compte bancaire** : un mouvement passé sur un compte du grand livre qui
  n'est plus celui du compte bancaire, ou sur un autre compte bancaire **à compte du grand livre distinct**, n'a pas de
  mouvement sur le compte lié actuel. *(Validation P2, R1.)* Deux comptes bancaires qui **partagent** un compte du grand
  livre voient au contraire, chacun, les écritures non rapprochées de l'autre parmi leurs candidates (AC-2.12) : faux
  candidats de la classe L6, datés et non présélectionnés. Issue demandée (ancien compte, autre compte).
- **L4 — Frais de rappel** (**#401**) : le paiement dépasse le règlement ; ni la 29-8 (au-delà de 5 centimes) ni la 3a
  (montant exact) ne le prennent.
- **L5 — Règlement fournisseur sous le centime** (#687) : son mouvement n'égale jamais une transaction ; la v017-3b
  ferme la source et décrit le traitement de l'existant.
- **L6 — Faux candidat** : un mouvement récurrent de même montant (loyer) dont l'écriture d'un mois précédent n'est pas
  rapprochée est candidat pour la transaction du mois courant ; la date de chaque candidate s'affiche, rien n'est
  présélectionné.
- **L7 — Écriture d'ouverture** : si le montant d'une transaction égale le solde d'ouverture du compte de la banque,
  l'écriture d'ouverture est candidate. Elle s'affiche avec son libellé ; rien n'est présélectionné.
- **L8 — Rejetées** : le dialogue n'est offert que depuis la liste des propositions, d'où les rejetées sont exclues ; une
  transaction rejetée selon la consigne d'attente se rattache après *Annuler le rejet*.
- **L10 — `INVOICE_CREDITED`** *(validation P3, R4)* : ce motif ne se lève pas (« … il ne s'annule pas ») ; état hérité
  d'avant la 0.12.1 (`invoice_settlements_write.rs:776-785`), rare. Le dialogue le dit par l'introduction « tant que dure
  l'obstacle suivant », vraie sans promettre de levée ; « chaque motif se lève » (registre, amendement P2 de C-v017-3-21)
  est faux pour lui.
- **L9 — `cancelBlockedBy` est un instantané** *(réécrit à la validation P2, R2, R9)* : le motif est lu **au moment de la
  liste**. Il peut apparaître après le rattachement (clôture ultérieure de l'exercice, le cas même de #671 l'année
  suivante) ou disparaître (réouverture, compte réactivé, exercice du jour créé) : le dialogue le dit par la formule
  datée, sans promesse. S'y ajoutent les angles morts **documentés** du prédicteur (`settlement_cancellation.rs:50-66` :
  verrou de période du jour ; exercice postérieur clos, #568), où la lecture rend `null` et le clic refuse ; et les
  motifs de **propriété** (`OWNED_BY_*`) que la queue commune ignore (`settlement_cancellation.rs:131-133`) : une écriture
  de vente, d'avoir ou d'achat classée « écriture propre » serait annoncée annulable et refusée — inatteignable tant que
  le compte de la banque n'est pas un compte de créance ou de dette, ce que refusent `refuse_if_ledger_is_claim_account`
  (`bank_accounts.rs:523`, `:654`) et `ClaimAccountLinkedToBankAccount`, sauf réglage antérieur à la 15-6c conservé.

## À recontrôler à l'étape 0 du développement

- `reconciliation_link.rs:77-79`, `:150-175`, `:329-448` ; `routes/reconciliation.rs:6064-6100`, `:6137-6301`.
- `reconciliation_cancel.rs:11-28`, `:57-79`, `:141`, `:215-270` (`cancel_blocker`, à scinder — D14) ;
  `settlement_cancellation.rs:50-66`, `:87-182`, `:131-133`.
- `journal_entries.rs:1457-1459`, `:1540-1541`, `:1819-1835`, `:2368`, `:2492`, `:2741-2745`, `:2802` (AC-3 (a)).
- L'inventaire de l'AC-3 (b), par fonction.
- `crates/kesh-db/src/errors.rs:440-452` (codes de fil) ; `frontend/src/lib/features/reconciliation/reconciliation-cancel.ts`,
  `reconciliation.types.ts:154-190`.
- `supplier_invoices.rs:761-1013` (écriture de règlement, compte de la banque crédité).
- `bank_account_closed_inventory.rs:237`, `:516-519`, `:892`, `:1280-1284` ; `audit_route_registry.rs:33`, `:157`,
  `:908-1324` (valeurs du moment) ; `admin-manual.tex:2481`, `:2563`, `:2966`.
- `LinkSettlementModal.svelte:53-57` ; `ReconciliationProposals.svelte:230-260`, `:577-586`, `:658-666` ;
  `i18n-keys.test.ts:653` ; `textes_coherents.rs:805` ; `reconciliation_link_e2e.rs:1316-1430`.
- Le geste *Annuler le rejet* du détail d'import (28-5b) et sa route.

## Collision avec la story parallèle v016-2 (v016-2a et v016-2b)

*(Relue à la validation P2, F9 : la v016-2 est découpée en `v016-2a-releve-manuel-backend.md` et
`v016-2b-releve-manuel-ecran.md`, `/home/gcorbaz/devel/kesh-0162/_bmad-output/implementation-artifacts/`.)* Aucun fichier
de **réconciliation** en commun. Points de fusion :

1. **`crates/kesh-api/tests/audit_route_registry.rs`** — la v016-2a ajoute deux routes mutantes (sa fiche,
   `:198-201` *(citation corrigée à la validation P3, F4 : AC-15, `:226-229`)* : `LIB_ROUTES` 135 → 137, `traced` 129 → 130, `no_matter` 2 → 3, `:944` 138 → 140, `sans_ecriture` 110 →
   112, `tout.len()` 138 → 140), la v017-3a une. Ils réécrivent les **mêmes** assertions et la même prose : à la
   réunion, les compteurs se **recomptent depuis le registre fusionné**, jamais par addition des deux diffs.
2. **`docs/manual/fr/admin-manual.tex:2966`** (« 129 des 135 routes … ») et son PDF, que lit
   `le_manuel_dit_le_compte_du_registre` : la v016-2a la récrit aussi ; même règle que le point 1, PDF régénéré après.
3. `crates/kesh-api/tests/bank_account_closed_inventory.rs` — la v017-3a ajoute un écrivain et un membre de G ; la
   v016-2a n'en ajoute pas (son AC-8). Conflit de texte possible sur `sites.len()` et `gardes_non_verrouillantes`.
4. `crates/kesh-api/tests/audit_label_registry.rs` — la v016-2a y touche `SITES_INDIRECTS` (`:184-195`) ; la v017-3a
   n'y ajoute qu'une action triée : fusion à la main.
5. **CHANGELOG** — la v016-2a renomme `[0.16.1]` en `[0.17.0]` ; la v017-3a écrit sous un en-tête provisoire : deux
   sections `[0.17.0]` à fusionner (AC-20).
6. **§ *Passer à la 0.17.0*** — renommé par la v016-2a ; le paragraphe *Intégrations par l'API* de la v017-3a s'y reporte
   (AC-20).
7. `crates/kesh-api/src/errors.rs`, `audit_labels.rs` (liste triée), `lib.rs` (montage), les quatre catalogues i18n,
   `docs/api-external.md`, `docs/manual/fr/user-manual.tex` (sections distinctes) et son PDF : insertions voisines,
   à fusionner à la main ; le PDF se **régénère** après fusion.
7-bis. *(Validation P3, F4, R7.)* **Mêmes lignes, non voisines** — conflits de texte certains, à recompter ou fusionner à
   la main : `README.md:41` (la v016-2b le récrit, `v016-2b-releve-manuel-ecran.md:203`, AC-30 ; la 3a aussi, AC-20-bis) ;
   `frontend/src/lib/shared/i18n-keys.test.ts`, `ATTENDU.sitesTotal` (`:767`), cardinalité exacte que les deux stories
   changent (`v016-2b-releve-manuel-ecran.md:109`) : à **recompter** après réunion, jamais additionner ;
   `docs/api-external.md:243` et `:878` (listes que la v016-2a étend aussi, `v016-2a-releve-manuel-backend.md:397`).
8. **Synergie** : un compte sans relevé électronique (v016-2) dont Guy a passé les mouvements au journal produira des
   transactions `MANUAL` en attente **déjà comptabilisées** — exactement le cas de la v017-3a, qui ne lit pas
   `source_format`. Un test de l'une sur un relevé de l'autre est souhaitable après la réunion (à l'orchestrateur).

## Collision avec la v017-3b (#687) — sœur, même release

La 3b récrit des passages du manuel que la 3a récrit aussi ; ils se réconcilient à la réunion des deux branches, la 3a
récrivant l'encadré et la 3b n'y ajoutant que ses renvois à `sec:facture-fournisseur-sous-centime` (fiche 3b,
§ *Dépendance à la v017-3a*) :

| Section (`user-manual.tex`, `c5101b07`) | v017-3a | v017-3b |
|---|---|---|
| `sec:regler-facture-fournisseur`, `:1630-1640` | récrit (AC-18.1) | porte la sous-section `sec:facture-fournisseur-sous-centime` |
| `sec:matching-fournisseur`, `:2304`, encadré `:2324-2343` | encadré récrit (AC-18.2) | complète « un total à plus de deux décimales n'est jamais proposé » |
| `sec:rattacher-reglement`, `:2384-2412` | récrit (AC-18, AC-18.5) | sa puce « règlement sous le centime » renvoie à `sec:facture-fournisseur-sous-centime` |
| `sec:paiements-fournisseurs`, `:1737`, et lot en cours `:2306` | `:2306` (AC-18.11) | `:1737` renvoi |
| Diagnostic, `:3325-3340` | `:3337-3338` (AC-18.4) | `:3329` |

## Collision avec la v017-3c (hors release)

La v017-3c (`v017-3c-avertissements-propositions.md`, `backlog`) part de la 3a fusionnée : elle reprend
`find_link_candidates` et `find_entry_candidates` pour compter, et le dialogue de l'AC-13 pour le bouton de son
avertissement. Aucun fichier de la 3a n'attend la 3c.

## Issues

*(Réécrit à la validation P2, F7 : « aucune autre » ne s'écrit plus — chaque reste a son issue.)*

| Reste (D9) | Issue |
|---|---|
| Une transaction pour plusieurs écritures (lot de plusieurs factures, paiement groupé) | **#823** (ouverte) |
| Second côté d'un virement entre comptes propres | **#823** (ouverte) — correction de commentaire à poser, ci-dessous |
| Paiement qui inclut les frais de rappel | **#401** (ouverte) |
| Règlement passé sur un **ancien** compte du grand livre, ou sur un **autre** compte bancaire | **#827** (ouverte depuis — *validation P3, R8*) |
| Avertissement sur les propositions, confirmations, coût | **#828** (ouverte depuis, v017-3c — *validation P3, R8*) |

**Issue à ouvrir** *(ouverte depuis : #827)* — titre : « Rapprochement : un mouvement comptabilisé sur un ancien compte du grand livre de la
banque, ou sur un autre compte bancaire, ne se rattache pas à sa transaction ». Corps : « Constat (fiche v017-3a, #671,
2026-10-30). La v017-3a rattache une transaction en attente à une écriture dont le mouvement sur le compte du grand
livre **actuel** de son compte bancaire l'égale. Deux cas restent en attente pour toujours : (1) le compte du grand
livre lié au compte bancaire a été changé (`PATCH /bank-accounts/{id}`) après le règlement, qui porte son mouvement sur
l'ancien compte ; (2) le règlement a été saisi sur un autre compte bancaire que celui de la transaction (un commentaire
de #671 du 2026-10-14 le relève pour un règlement client). Consigne en attendant : annuler le règlement et le ressaisir
sur le bon compte ; pour (1), rejeter la transaction. Attendu : un geste de correction, ou une règle de candidature qui
lise aussi l'ancien compte. Refs #671. » Labels proposés : `bug`, P3.

**Correction à poser sur #823** (le commentaire du 2026-10-30 y dit : « Depuis la story v017-3a (#671, release 0.17.0) :
… l'autre côté garde un avertissement (`alreadyBooked.matchedElsewhereCount`, choix C-v017-3-22) … ») : « Correction
(2026-10-30, validation P2 de la v017-3a) : le commentaire précédent n'est plus exact. L'avertissement du second côté
(`alreadyBooked.matchedElsewhereCount`) est retiré de la v017-3a et renvoyé, avec l'avertissement des propositions, à
une story hors de la release 0.17.0 (v017-3c, issue à venir). Dans la 0.17.0, le second côté d'un virement entre deux
comptes propres ne porte **aucun** avertissement : la consigne du manuel reste de le rejeter, non de l'affecter.
L'objet de cette issue est inchangé. »

## LOW non pris

Aucun : R5 à R10 et F9 à F16 sont appliqués, ou passés à la v017-3c avec leur sujet (F14 pour la phrase « à 5
centimes près » de l'avertissement, F16 en partie). R9 va aux angles morts (L9), comme demandé.

## Questions au Project Lead

Tranchées au registre ; listées pour être soumises à Guy.

- **Q1 (C-v017-3-2)** — Le rattachement porte sur **toute écriture** du journal dont le mouvement sur le compte de la
  banque égale la transaction, pas seulement sur les règlements fournisseur. D'accord ?
- **Q2** — *(Passée à la v017-3c avec l'avertissement.)*
- **Q3 (C-v017-3-4)** — Aucune fenêtre de dates (comme la 29-8) : une écriture de l'an dernier au même montant serait
  candidate, triée en dernier. Une fenêtre vous paraît-elle préférable ?
- **Q4 (C-v017-3-9, C-v017-3-21)** — *Annuler le rapprochement* d'une transaction rattachée à une écriture saisie
  **contre-passe** cette écriture (#672 garde le geste « défaire le seul lien », qui demande une migration). Et un
  rattachement dont l'annulation est empêchée (exercice clôturé…) reste admis, le dialogue disant « tant que … subsiste »
  et le moyen de lever le motif. Acceptable pour la 0.17.0 ?
- **Q5 (C-v017-3-10)** — Les restes sont tracés (#823, #401, une issue à ouvrir) ; l'avertissement des propositions est
  renvoyé à la v017-3c, hors de la 0.17.0. Fermer #671 à la release ?

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
- 2026-10-30 — **Validation P2 et découpage** (Opus, lentilles R et F) : R 0/0/4/6, F 0/0/8/8 — 0 CRITICAL, 0 HIGH,
  12 MEDIUM, 14 LOW. La sévérité ne baisse pas (P1 : 1 HIGH, 12 MEDIUM ; P2 : 12 MEDIUM), et une part des défauts naît de
  la remédiation P1 — `cancelBlockedBy` et ses codes (F3, F4, R2, R9), `matchedElsewhereCount` (F5, F6, R1) : recyclage,
  déclaré à Guy par l'orchestrateur. **Décision C-v017-3-25 (orchestrateur) : découpage.** La 3a garde le cœur de #671
  (routes, section « écritures » du dialogue, règle, verrous et courses, registres, audit, manuel, `api-external.md`) ;
  `alreadyBooked` et ses deux modes, l'avertissement des propositions, les confirmations et la mesure de coût passent à
  **v017-3c** (`backlog`, hors de la 0.17.0) avec R1 (comptes du grand livre partagés), F5, F6, F14 et F16 en partie
  (anciens AC-10, AC-11, AC-12, AC-14, D6, D13, D15, M13, M15, M21, Q2). Les AC et le corps de la fiche sont
  **réorganisés** en conséquence (les entrées passées de ce Change Log restent telles quelles). Prémisses vérifiées au
  code ; aucun finding rejeté. Dans la 3a : `cancelBlockedBy` devient une **vérité datée** (« tant que … subsiste », le
  moyen de lever par `reconciliationCancelMessage`, réutilisée), codes de fil (`errors.rs:440-452`) et
  `cancelBlockedLabel`, aucune promesse inconditionnelle (R2, F3, F4, D12) ; T-race-3 écrite sur le patron `t17bis`
  complet, M10 **tuée** (R3) ; M7 et M18 équivalentes — M7bis, fixture SQL pour M18 (R4) ; M2bis réattribuée (R5) ;
  inventaire de l'AC-3 (b) par fonction, la vraie raison (verrou nommé et rejeu) écrite (R6) ; G 11 → 12 et prose `:157`
  (R7) ; citations (R8) ; L9 instantané et motifs de propriété (R9) ; règlement client par compte interne testé et
  doc-comments « seul chemin » (R10) ; douze passages du manuel, filet `rattach` et `\#671` (F1, F2) ; chaque reste
  renvoyé à son issue, une issue à ouvrir (F7) ; `api-external.md:243` (F8) ; collisions avec la v016-2a/2b relues (F9)
  ; `reconciliation-labels-more-capped` et `README.md:41` (F10) ; `sitesTotal` (F11) ; citations du manuel (F12) ;
  patron d'un débit pour l'E2E (F13) ; asymétrie dite (F14) ; CHANGELOG provisoire et § *Passer à la 0.17.0* (F15, F9) ;
  listes vides, trois cas (F16) ; AC-2.12 sur les comptes partagés (R1, côté 3a). Prochaine étape : validation P3
  complète (`v017-3a-validate-prompt-p3.md`, Sonnet, lentilles R et F).
- 2026-10-30 — **Validation P3** (Sonnet, lentilles R et F) : R 0/0/2/6, F 0/0/4/6 — 0 CRITICAL, 0 HIGH, 6 MEDIUM,
  12 LOW (P2 : 12 MEDIUM). Prémisses vérifiées au code ; aucun finding rejeté. R1 : l'AC-3 (b) est réécrit sur la vraie
  raison — aucun geste qui tient `journal_entries` ne verrouille ensuite `bank_transactions`, `reversal_blockers` lit sans
  verrou ; l'affirmation sur le verrou nommé est retirée (`delete_once` ne le prend pas) ; critère d'arrêt de l'étape 0
  reformulé. R2 = F1 : `:2300`, `:2354`, `:2366`, `:3337` ajoutés (points 13 à 16) ; filet `rattach` exécuté au `.tex` (44
  lignes) et au PDF aplati (45 fragments), sortie et tri copiés dans la fiche. F2 : `LinkSettlementModal.test.ts` nommé
  (mock, `link-empty`, relecture, replis). F3 : quatre cas de listes vides, textes, Vitest, captures. F4, R7 : collisions
  sur les mêmes lignes (`README.md:41`, `sitesTotal`, `api-external.md:243`, `:878`), citation `:226-229`. R5 : coût de
  `cancelBlockedBy` écrit, AC de mesure (50 candidates, moins de 1 s), dette et issue au-delà. LOW : M20 (R3) ;
  `INVOICE_CREDITED`, L10 (R4) ; M27, M28 (R6) ; #827 et #828, C-v017-3-10 annoté (R8) ; gabarit de la phrase du motif
  (F5) ; phrase sans motif (F6) ; citations `README.md:41`, `sitesTotal:767` (F7) ; CHANGELOG en *Ajouté* (F8) ; M29, M30,
  captures (F9) ; commentaires de l'écran, décompte 44 (F10). Le résidu signalé par la 3b (colonne 3b du tableau commun)
  était déjà corrigé à la validation P2 : la colonne renvoie à `sec:facture-fournisseur-sous-centime` (vérifié). Prochaine
  étape : validation P4 ciblée (`v017-3a-validate-prompt-p4.md`, Opus, une lentille).
