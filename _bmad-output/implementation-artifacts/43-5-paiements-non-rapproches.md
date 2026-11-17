# Story 43-5 : un mouvement du relevé rapproche des paiements déjà passés — lot pain.001 confirmé, « Payer » à la main (#823)

Status: ready-for-dev

*Fiche rédigée le 2026-11-17 au soir par l'agent de rédaction de l'Epic 43 (Opus 5.5), workflow `bmad-create-story`, sur
la décision de Guy du même jour (« seulement des factures ouvertes : la facture n'est définitivement fermée que lorsque son
paiement est rapproché »). Code lu à `8b50fd195`. Release **0.19.0** si elle est close à la coupure ; **la première à
sortir sinon** (C-43-18). Part de la 43-1a fusionnée (la colonne `journal_entry_lines.bank_transaction_id` y est posée),
et de la 43-1b pour le cas fournisseur. `kesh-db` touché : gate complet en boucle. Décisions : C-43-16, C-43-17 (fichier
d'épique), qui font foi. Validation : à lancer.*

**Issues** : `refs #823`.

## Story

En tant que comptable qui a confirmé un lot pain.001 (ou payé des factures depuis leur fiche), je veux rapprocher le débit
unique du relevé de ces paiements déjà passés — avec, au besoin, des factures encore impayées et une petite différence —
pour que leurs factures soient enfin closes au rapprochement.

## Ce qui se lit au code (`8b50fd195`)

- La confirmation d'un lot passe une écriture par facture (`payment_batches.rs:574-590`, `pay_in_tx`).
- Le rattachement unitaire d'une transaction à une écriture existante (v017-3a, `reconciliation_link::link_entry_in_tx`,
  `reconciliation_link.rs:1156`) ; son dé-rapprochement défait le règlement (`reconciliation_cancel.rs:11-50`).
- Les lecteurs « cette écriture est rapprochée », par `matched_entry_id` (inventaire ci-dessous).

## Modèle (C-43-17)

- Un **paiement existant** : l'écriture d'un règlement client par virement (`invoice_settlements`, `bank_transfer`) ou
  d'une facture fournisseur payée (`settlement_journal_entry_id`), dont la ligne de banque est sur le compte du grand livre
  de la transaction, qu'aucune transaction ne rapproche (ni `matched_entry_id`, ni la colonne). Apport : le montant de sa
  ligne de banque. Même sens que la transaction.
- Le geste rapproche une transaction d'un ensemble `{factures impayées} ∪ {paiements existants}`, de deux éléments au
  moins. S'il y a des factures impayées ou un écart non nul, la 43-1a / 43-1b passent l'écriture groupée (l'écart y
  compte les apports des paiements existants) et `matched_entry_id` la pointe ; sinon `matched_entry_id` pointe le plus
  petit identifiant des paiements existants. **Toutes** les lignes de banque des paiements existants reçoivent
  `bank_transaction_id` = la transaction (celle que pointe `matched_entry_id` comprise).
- Dé-rapprochement : les paiements existants **gardent** leur écriture ; seule la colonne revient à `NULL` (C-43-17, Q5) ;
  l'écriture groupée éventuelle se défait comme en 43-1a / 43-1b.

## Critères d'acceptation

- **AC1** — `JournalEntryLine.bank_transaction_id: Option<i64>` et les listes de colonnes qui l'hydratent ; export CSV de
  `journal_entry_lines`.
- **AC2** — `grouped_settlements` accepte des paiements existants (paramètre `payment_entry_ids`), verrouillés **après**
  les factures, par identifiant croissant ; refus par paiement (`GroupRefusal`, `entry_id`) : introuvable, autre société,
  pas un paiement au sens du modèle, sens contraire, autre compte de banque, déjà rapproché, exercice clos. Écart
  (C-43-16) calculé sur tous les apports.
- **AC3** — **Inventaire des sites qui disent « rapprochée »** — chacun lit aussi la colonne, ou est assumé avec sa
  raison, au Change Log :
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
- **AC4** — Dé-rapprochement : liens retirés sans contre-passer les paiements existants ; l'écriture groupée éventuelle
  défaite ; audit `reconciliation.cancelled` avec la liste des paiements détachés et l'acteur.
- **AC5** — API (avec la 43-2) : `paymentEntryIds` dans `invoice_group` et `supplier_invoice_group` ; la recherche des
  candidates rend aussi les paiements existants (`kind: "payment"`, `entryId`, facture, montant) ; `groupedInvoices` de la
  vue les marque. `docs/api-external.md`.
- **AC6** — Écran (avec la 43-3) : les paiements existants s'ajoutent comme les factures, marqués « payée, paiement non
  rapproché ». Manuel utilisateur : le cas du lot pain.001 confirmé, et le fait qu'un dé-rapprochement laisse ces paiements
  en place.

## Tests

Un lot de trois factures confirmé, puis son débit rapproché (aucune écriture neuve, trois colonnes posées) ; le même avec
une facture impayée en plus et un écart de 0.40 ; dé-rapprochement (paiements intacts) ; chaque site d'AC3 par un test qui
rougit sans la lecture de la colonne (mutation : retirer la branche de `document_owners` — la contre-passation d'un
paiement rapproché par la colonne passe) ; refus d'AC2.

## Tâches

- [ ] **T0** — Étape 0 ; inventaire d'AC3 relu à la tête de la branche réunie. — [ ] **T1** — AC1, AC2. — [ ] **T2** — AC3.
- [ ] **T3** — AC4. — [ ] **T4** — AC5, AC6 (commits séparés, retirables). — [ ] **T5** — Gates, revue.

## Note de version

- **Ajouté** — Le débit d'un lot de paiement confirmé (ou d'un paiement saisi depuis la fiche) se rapproche des paiements
  déjà passés, avec ou sans factures impayées en plus ; la facture est close au rapprochement (refs #823).

## Dev Notes

Modules : `grouped_settlements`, `journal_entries` (`document_owners`), `supplier_invoices`, `settlement_cancellation`,
`bank_accounts`, `reconciliation_link`, `reconciliation_cancel` — **sept** : au-delà de la règle de splitting. Dérogation
assumée, écrite ici : la colonne n'est sûre que si **tous** ses lecteurs la lisent dans le même commit fusionné ; les
séparer laisserait une écriture rapprochée que certains gestes croiraient libre. Si la story ne tient pas jeudi, elle sort
entière.

## Dev Agent Record

### Agent Model Used

### Completion Notes List

### File List

## Change Log

- 2026-11-17 — Fiche rédigée (décision de Guy du 2026-11-17).
