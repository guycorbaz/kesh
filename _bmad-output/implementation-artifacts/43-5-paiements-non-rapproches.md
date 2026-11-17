# Story 43-5 : un mouvement du relevé rapproche des paiements déjà passés — lot pain.001 confirmé, « Payer » à la main (#823)

Status: ready-for-dev

*Fiche rédigée le 2026-11-17 au soir par l'agent de rédaction de l'Epic 43 (Opus 5.5), workflow `bmad-create-story`, sur
la décision de Guy du même jour (« seulement des factures ouvertes : la facture n'est définitivement fermée que lorsque son
paiement est rapproché ») ; **révisée dans la nuit** : sa colonne sort de la migration de la 43-1a pour une **migration
propre** (validation P2 de la 43-1a, M-D) ; **décision de Guy sur Q5 = (a)**. Code lu à `8b50fd195`. Release **0.19.0** si
elle est close à la coupure ; **la première à sortir sinon**, avec sa migration (C-43-18). Part de la 43-1a fusionnée, et de
la 43-1b pour le cas fournisseur. `kesh-db` touché : gate complet en boucle. Décisions : C-43-16, C-43-17 (fichier
d'épique), qui font foi. Validation : à lancer.*

**Issues** : `refs #823`.

## Story

En tant que comptable qui a confirmé un lot pain.001 (ou payé des factures depuis leur fiche), je veux rapprocher le débit
unique du relevé de ces paiements déjà passés — avec, au besoin, des factures encore impayées et des frais — pour que leurs
factures soient enfin closes au rapprochement.

## Ce qui se lit au code (`8b50fd195`)

- La confirmation d'un lot passe une écriture par facture (`payment_batches.rs:574-590`, `pay_in_tx`).
- Le rattachement unitaire d'une transaction à une écriture existante (v017-3a, `reconciliation_link::link_entry_in_tx`,
  `reconciliation_link.rs:1156`) ; son dé-rapprochement défait le règlement (`reconciliation_cancel.rs:11-50`).
- La modification d'écriture (`PUT`) supprime et réinsère les lignes à colonnes explicites (`journal_entries.rs:1649-1655`,
  :1672) : une colonne neuve de `journal_entry_lines` y serait perdue ; le garde-fou
  `crates/kesh-db/tests/journal_entries_modification.rs:153-171` fige la liste des colonnes.
- Les lecteurs « cette écriture est rapprochée », par `matched_entry_id` (inventaire d'AC3).

## Modèle et décisions (C-43-17)

- **Migration propre** : `crates/kesh-db/migrations/20261118000002_bank_line_transaction.sql`, DDL seul :
  `journal_entry_lines` `ADD COLUMN bank_transaction_id BIGINT NULL`, `fk_jel_bank_transaction` → `bank_transactions(id)
  ON DELETE RESTRICT`, `INDEX idx_jel_bank_transaction (bank_transaction_id)`. Non breaking (même raisonnement que la
  43-1a, C-43-4). Elle **sort avec la story**.
- Un **paiement existant** : l'écriture d'un règlement client par virement (`invoice_settlements`, `bank_transfer`) ou
  d'une facture fournisseur payée (`settlement_journal_entry_id`), dont la ligne de banque est sur le compte du grand livre
  de la transaction, qu'aucune transaction ne rapproche (ni `matched_entry_id`, ni la colonne). Apport : le montant de sa
  ligne de banque (au centime). Même sens que la transaction.
- Le geste rapproche une transaction d'un ensemble `{factures impayées} ∪ {paiements existants}`, de deux éléments au
  moins. S'il y a des factures impayées ou une différence non nulle, la 43-1a / 43-1b passent l'écriture groupée (la
  différence compte les apports des paiements existants) et `matched_entry_id` la pointe ; sinon `matched_entry_id` pointe
  le plus petit identifiant des paiements existants. **Toutes** les lignes de banque des paiements existants reçoivent
  `bank_transaction_id` = la transaction, celle que pointe `matched_entry_id` comprise (une écriture pointée **sans** la
  colonne reste un rattachement unitaire, défait comme aujourd'hui).
- **Dé-rapprochement — décision de Guy (Q5 = a, 2026-11-17)** : les paiements rattachés ne perdent **que** leur lien
  `bank_transaction_id` ; ils ne sont pas contre-passés et leurs factures restent payées. Dans un geste mixte, les
  règlements **créés** par le geste (factures impayées) sont annulés et leur écriture est contre-passée (43-1a, 43-1b) ;
  les paiements **rattachés** ne perdent que leur lien. Différence assumée avec le rattachement unitaire (v017-3a), écrite
  au manuel.

## Critères d'acceptation

- **AC1** — Migration, `docs/migrations-idempotence-audit.md` (une ligne ; totaux à **93**, compteurs recomptés),
  `migrations_upgrade_path.rs` (`assert_eq!(total, 93)`, `total - 59` : frontière à la 35ᵉ), squash `test-schema`.
  `JournalEntryLine.bank_transaction_id: Option<i64>`, `LINE_COLUMNS` et les listes qui l'hydratent ; l'`INSERT` de création
  et la réinsertion du `PUT` (`journal_entries.rs:512`, :1672) la portent ; export CSV de `journal_entry_lines`
  (`exports/global.rs:302`). **`journal_entries_modification.rs`** : la liste des colonnes (:153-171) gagne la colonne ; la
  colonne `bank_transaction_id` ne contient pas `journal_entry` et n'entre pas dans l'inventaire de :173-195 (à vérifier) ;
  décision écrite au test : une écriture dont une ligne porte la colonne est **possédée** (AC3, `document_owners`), donc
  jamais réécrite par le `PUT` — test « `PUT` d'une écriture rattachée par la colonne refusé, colonne intacte ».
- **AC2** — `grouped_settlements` accepte des paiements existants (paramètre `payment_entry_ids`), verrouillés **après**
  les factures, par identifiant croissant ; refus par paiement (`GroupRefusal`, `GroupedDocument::PaymentEntry(id)`) :
  introuvable, autre société, pas un paiement au sens du modèle, sens contraire, autre compte de banque, déjà rapproché,
  exercice clos. Différence (C-43-16) calculée sur tous les apports.
- **AC3** — **Inventaire des sites qui disent « rapprochée »** — chacun lit aussi la colonne, ou est assumé avec sa raison,
  au Change Log :
  - `journal_entries.rs:2343-2344` (`document_owners`, bloc des transactions) : ajouter les écritures dont une ligne porte
    la colonne — gel, contre-passation et modification refusés (`MATCHED_BANK_TRANSACTION`) ;
  - `supplier_invoices.rs:1331` (`DetachedPayment`) et `:1456` (`matched_transaction_locked`, `PaidReconciled`) ;
  - `settlement_cancellation.rs:147` ;
  - `bank_accounts.rs:1884`, `:1896` (écart de relevé) ;
  - `reconciliation_link.rs:161`, `:703-706`, `:742-753`, `:781` (candidates et autre côté d'un virement : une écriture
    rapprochée par la colonne n'est plus candidate) ;
  - `reconciliation_cancel.rs` `classify` : une écriture pointée dont la ligne de banque porte la colonne est un
    rattachement multiple ;
  - `invoices.rs:1640` (écriture de **vente** rapprochée) : assumé, la colonne ne se pose que sur des paiements ;
  - tout autre site rendu par `git grep -n "matched_entry_id"` (sites de production) s'ajoute.
- **AC4** — Dé-rapprochement selon Q5 = (a) : liens retirés sans contre-passer les paiements existants, factures
  rattachées toujours payées ; l'écriture groupée éventuelle défaite (règlements créés annulés) ; audit
  `reconciliation.cancelled` avec la liste des paiements détachés et l'acteur.
- **AC5** — API (avec la 43-2) : `paymentEntryIds` dans `invoice_group` et `supplier_invoice_group` ; la recherche rend
  aussi les paiements existants (`kind: "payment"`, `entryId`, facture, montant) ; `groupedInvoices` de la vue les marque.
  `docs/api-external.md`.
- **AC6** — Écran (avec la 43-3) : les paiements existants s'ajoutent comme les factures, marqués « payée, paiement non
  rapproché ». Manuel utilisateur : le cas du lot pain.001 confirmé, et le fait qu'un dé-rapprochement laisse ces paiements
  en place.

## Tests

Un lot de trois factures confirmé, puis son débit rapproché (aucune écriture neuve, trois colonnes posées) ; le même avec
une facture impayée en plus et des frais de 0.40 ; dé-rapprochement mixte (paiements intacts et factures payées, règlement
créé annulé et contre-passé) ; chaque site d'AC3 par un test qui rougit sans la lecture de la colonne (mutation : retirer
la branche de `document_owners` — la contre-passation d'un paiement rattaché passe) ; refus d'AC2 ; montée 92 → 93.

## Tâches

- [ ] **T0** — Étape 0 ; inventaire d'AC3 relu à la tête de la branche réunie. — [ ] **T1** — Migration et AC1.
- [ ] **T2** — AC2. — [ ] **T3** — AC3. — [ ] **T4** — AC4. — [ ] **T5** — AC5, AC6 (commits séparés, retirables).
- [ ] **T6** — Gates, revue.

## Note de version

- **Ajouté** — Le débit d'un lot de paiement confirmé (ou d'un paiement saisi depuis la fiche) se rapproche des paiements
  déjà passés, avec ou sans factures impayées en plus ; la facture est close au rapprochement ; un dé-rapprochement retire
  seulement ce lien (refs #823).

## Dev Notes

Modules : migration, `grouped_settlements`, `journal_entries` (`document_owners`, colonnes), `supplier_invoices`,
`settlement_cancellation`, `bank_accounts`, `reconciliation_link`, `reconciliation_cancel` — au-delà de la règle de
splitting. Dérogation assumée, écrite ici : la colonne n'est sûre que si **tous** ses lecteurs la lisent dans le même
commit fusionné ; les séparer laisserait une écriture rapprochée que certains gestes croiraient libre. Si la story ne tient
pas jeudi, elle sort entière, migration comprise, et le compte de migrations de la release reste 92.

## Dev Agent Record

### Agent Model Used

### Completion Notes List

### File List

## Change Log

- 2026-11-17 — Fiche rédigée (décision de Guy du 2026-11-17).
- 2026-11-17 (nuit) — Migration propre `20261118000002_bank_line_transaction.sql` (M-D), garde-fou de la modification
  d'écriture, compte à 93 ; décision de Guy Q5 = (a).
