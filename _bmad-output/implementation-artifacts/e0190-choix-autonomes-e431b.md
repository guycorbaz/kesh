# Story 43-1b — registre des choix (développement, 2026-11-17)

Registre de la story 43-1b (clé `e431b`). Les décisions de l'épique (C-43-*) et du contrat de la 43-1a (`t3.md`,
registre `e0190-choix-autonomes-e431a.md`) font foi ; ce registre consigne les choix tranchés par l'agent de
développement. Chacun est réversible sans migration.

**C-e431b-1 — Binaire de test à part** : `crates/kesh-db/tests/grouped_settlements_supplier.rs`, et non un module
fournisseur de `grouped_settlements.rs` comme la fiche l'écrit. Raison : la 43-1a, en revue P2, modifie encore ce
fichier ; deux branches qui ajoutent chacune leurs tests à sa fin se fusionneraient en conflit. Autre option : le
module dans le même fichier (rejetée pour le conflit). Coût : un binaire de test de plus (compilation).

**C-e431b-2 — La dette lue sur l'écriture d'achat, contrôlée égale au `total_amount`** : la ligne de dette du groupe
porte le `total_amount` de la facture (fiche, AC2 ; la vue relit ce montant), et le compte de dette est celui de la
ligne de crédit de l'écriture d'achat (`purchase_payable_line`, comme `pay_in_tx_as`). Si les deux montants diffèrent,
`Err(DbError::Invariant)` : le lettrage (achat, règlement) ne s'équilibrerait pas. Les deux sont égaux par
construction à la validation. Une écriture d'achat absente ou sans ligne de crédit : `Invariant`, comme l'unitaire —
aucune variante de refus n'existe pour elles (`SaleEntryMalformed` est propre au client). Autre option : payer le
montant de la ligne d'achat, comme l'unitaire (rejetée : la vue et la fiche parlent de `total_amount`).

**C-e431b-3 — Refus de `lock_for_bank_settlement_in_tx` traduits** : `NotFound` → `InvoiceNotFound` ; `NotOpen` →
`InvoiceNotEligible { SupplierNotOpen }` ; `InGeneratedBatch` → `InvoiceNotEligible { SupplierInPaymentBatch }` ;
`HeldByPaymentBatch { supplier_invoice_id, payment_batch_id }` → `HeldByPaymentBatch { piece: l'AUTRE facture (celle
que le lot tient, comme le `supplierInvoiceId` de l'unitaire), payment_batch_id: Some(..) }` ; les trois refus du
compte bancaire → leurs variantes. La garde (d) reçoit |montant| de la transaction, comme l'unitaire : un débit égal à
une facture d'un lot `generated` du compte reste refusé. L'ordre des refus est celui de la fonction **réemployée**
(facture, compte bancaire, lot, garde du lot du compte), puis la date, puis la dette — la liste de la fiche (AC1)
nommait « lot » avant « compte bancaire ».

**C-e431b-4 — Le compte d'écart d'un groupe fournisseur : les lignes de dette lues nommément** (`supplier_difference_account_of`,
**récrit à la revue P1**, décision de l'orchestrateur sur A MEDIUM-1, B-2, E1). Une facture **détachée** (AC7) n'est
plus membre, mais sa ligne de dette reste dans l'écriture : lire l'écriture par les seuls membres rendrait un écart faux
ou un `Invariant`. La première version (2026-11-17) devinait le bloc des dettes en l'étendant aux lignes suivantes de
même compte — **fausse** : la dernière facture détachée, sur un autre compte de dette, était lue comme ligne d'écart,
sans erreur, dès que le groupe n'avait pas d'écart d'origine (`Some(dette)` au lieu de `None`, constaté rouge par
`a_detached_invoice_on_another_payable_account_leaves_no_false_gap`). Ce registre affirmait le contraire.
- **Source retenue** : les lignes des membres (`settlement_journal_entry_line_id`) **plus** celles des factures
  détachées, nommées par la trace de leur détachement (`supplier_invoice.cancelled`,
  `details.settlementJournalEntryLineId`, que la 43-1b écrit, AC7), lues par `DETACHED_SETTLEMENT_TRACES` et
  `DETACHED_SETTLEMENT_ENTRY` (la lecture du gel « paiement détaché »). Ensemble, elles doivent occuper exactement les
  positions 1 à `n` ; la décomposition est recalculée sur leurs montants (principe de C-e431a-8), la queue contrôlée
  en nombre ; toute autre forme rend `Invariant`.
- **Autre option : le `difference_account_id` de l'audit `reconciliation.accepted`** — une ligne, certaine aussi.
  Écartée parce qu'elle **recopie** une valeur au lieu de la relire sur l'écriture (C-e431a-8 a rejeté l'audit comme
  source de la vue pour cette raison), alors que la trace du détachement ne sert qu'à **nommer** des lignes, qui restent
  lues sur l'écriture ; et parce que c'est la seule trace qui garde la ligne d'une facture détachée — le gel « paiement
  détaché » s'appuie déjà sur elle. Les deux sources sont des traces d'audit : celle retenue est la plus étroite.
- **Angle mort** : une facture détachée par un code antérieur à la 43-1b n'aurait pas de ligne à sa trace — sans
  objet, aucun groupe n'existe avant elle. *(Annoté à la revue P2, LOW-1 : la phrase « si une trace manque, la vue
  rend `Invariant` » était **fausse** pour la dernière facture détachée d'un groupe sans écart — la forme restait
  cohérente et la vue rendait sa dette comme compte d'écart, constaté rouge par
  `a_missing_detachment_trace_makes_the_view_an_invariant`. Elle est vraie depuis C-e431b-15.)*
- Tests : `a_detached_invoice_on_another_payable_account_leaves_no_false_gap` (deux comptes de dette, sans écart et
  avec écart, la dernière détachée), `detaching_the_first_or_a_middle_invoice_keeps_the_view_right`,
  `a_detached_invoice_leaves_the_group_and_the_rest_unreconciles`. Le bras client est inchangé.

**C-e431b-5 — La tête fournisseur extraite** (`supplier_invoices::supplier_settlement_cancel_head`, `pub(crate)`) :
patron de `settlement_cancel_head` (C-e431a-15). `supplier_settlement_cancel_blocker_unlinking` l'appelle, puis la
queue (comportement unitaire inchangé) ; le groupe fournisseur l'appelle par facture, puis la queue une fois. Le rang 1
n'est écrit qu'à un endroit.

**C-e431b-6 — `ensure_open_in_tx` dans le paiement groupé** : redondant avec la lecture verrouillante de `closed_on`
par `lock_for_bank_settlement_in_tx`, il est posé à l'étape (5) comme dans le client, parce que l'inventaire du compte
clos (`kesh-api/tests/bank_account_closed_inventory.rs`) n'admet une revérification `recheck_open_locked_in_tx` que
dans une fonction qui porte une garde non verrouillante (registre `FENETRE`, (i) et (ii)), et que la fiche exige la
revérification (AC3). Les registres sont complétés : `REGISTRE` (deux fonctions), `ANNULATIONS` (6 → 7), `FENETRE`
(deux entrées), décomptes anti-muets 41 → 45 sites, G 14 → 16, revérifications 9 → 11.

**C-e431b-7 — Formes d'audit** : `reconciliation.accepted` porte `supplier_invoices: [{supplier_invoice_id,
settled_amount}]` (le client : `invoices`) et `transaction_amount` ; `supplier_invoice.paid` reprend les clés de
`pay_in_tx_as` (`settlementType`, `settlementJournalEntryId`, `amount` en chaîne) et ajoute
`settlementJournalEntryLineId`, `reconciliationAuditId`, `grouped: true` ; `supplier_invoice.settlement_cancelled`
(dé-rapprochement) : les clés de l'unitaire plus `settlementJournalEntryLineId`, `grouped` ; `reconciliation.cancelled`
porte `supplier_invoices: [{supplier_invoice_id, number}]`. Le détachement (`supplier_invoice.cancelled`) gagne
`settlementJournalEntryLineId` (AC7).

**C-e431b-8 — AC8, le refus de l'annulation du règlement** : la fiche dit « `MATCHED_BANK_TRANSACTION` ». Constaté au
test : `cancel_settlement_in_tx` laisse ce rang au socle, qui rend `DbError::EntryNotReversable { blocker:
ReversalBlocker::MatchedBankTransaction }` — le motif est bien celui-là, le code d'erreur de la variante est
`ENTRY_NOT_REVERSABLE`. Le test asserte le motif. Comportement inchangé.

**C-e431b-9 — Libellé de l'écriture** : « Paiement groupé de N factures fournisseur : n1, n2, … », numéros affichables
(interne, à défaut celui du fournisseur, à défaut l'identifiant), borné à 500 caractères par `bounded_description`,
extraite du libellé client (une seule définition de la coupe).

**C-e431b-10 — Code client touché, au minimum** : `claim_refusal` délègue à `claim_refusal_on` (côté soldé en
paramètre, la dette pour le fournisseur) ; `entry_description` délègue la coupe à `bounded_description` ;
`difference_account_of`, `group_cancel_blocker`, `view_members` et `cancel_in_tx` aiguillent vers le bras fournisseur
quand l'écriture n'a aucun règlement client (C-e431a-9, C-e431a-16). Aucune autre ligne du bras client ne change.
*(Amendé à la revue P1 : C-e431b-13 extrait aussi la règle de la différence et la création de l'écriture, que le
bras client appelle désormais — à comportement égal.)*

**C-e431b-11 — Mutation M3 équivalente** : retirer `AND version = ?` (et sa liaison) de l'`UPDATE` des factures ne
change rien d'observable, la facture étant tenue `FOR UPDATE` depuis `lock_for_bank_settlement_in_tx` : aucune
version ne peut bouger entre la lecture et l'`UPDATE`. La clause reste (AC3 la demande, défense en profondeur). La
clause de la **transaction** (lue sans verrou) n'est pas équivalente : mutation M7, tuée par
`a_transaction_changed_meanwhile_is_refused_after_the_entry` (revue P1, A LOW-5).

## Revue P1 (2026-11-18) — décisions de l'orchestrateur appliquées

**C-e431b-12 — L'ordre des verrous : facture, compte, lot** (B-1, E2). Le paiement groupé verrouille **toutes** ses
factures par identifiant croissant, avant le compte, par la lecture de tête de `create_batch` réemployée
(`payment_batches::lock_invoices_sql`, `FORCE INDEX (PRIMARY) … ORDER BY id FOR UPDATE`) ;
`lock_for_bank_settlement_in_tx` les redemande ensuite, déjà tenues — la fonction n'est pas scindée, ses appelants
unitaires sont inchangés. Avant, le groupe tenait le compte (pris avec la facture 1) en attendant la facture 2 : un
paiement depuis la fiche de la facture 2 formait un cycle — **constaté rouge** (1213) par
`a_fiche_payment_between_two_invoices_does_not_deadlock`, qui lance la fiche entre les deux factures. Mutation M8.

**C-e431b-13 — La règle de la différence écrite une fois** (B-3) : `difference_accounts` (plafond, compte d'arrondi
requis et jamais un compte de tiers, puis l'ordre des refus du compte d'écart), `push_difference_lines` (côté des
lignes `r` et `e` selon le côté de la banque) et `create_group_entry` (exercice, création, traduction des erreurs de
période et d'exercice en refus) — appelées par les deux bras. Le bras client garde exactement son comportement :
mêmes refus dans le même ordre, mêmes lignes (vérifié par ses binaires `grouped_settlements` et
`reconciliation_cancel`, rejoués). Seule différence de forme : l'exercice est lu après la construction des lignes,
qui ne refuse rien. **Dette restante** (issue proposée) : le verrou écriture + exercice, le lien défait et l'audit
`reconciliation.cancelled` du dé-rapprochement, et l'`UPDATE` de la transaction au paiement, restent en deux
exemplaires (client et fournisseur) ; leur extraction toucherait l'inventaire du compte clos (sites et registre de la
43-1a) et a été jugée trop risquée avant la coupure.

**C-e431b-14 — Le compte du grand livre lu sous verrou** (E3) : `LockedForSettlement` porte désormais
`bank_ledger_account_id`, lu par l'étape (b) en `FOR UPDATE` ; le paiement groupé l'emploie au lieu de le relire sur
l'instantané. Le champ est additif : `accept_one_supplier_invoice` et les tests de `payment_batches` ne lisent que
`invoice`. `ensure_open_in_tx` reste (C-e431b-6), commenté comme inerte, sauf un refus à tort d'un compte rouvert
après la naissance de l'instantané. *(Revue P2, LOW-3 : tenu par `the_bank_ledger_is_the_one_read_under_lock`,
mutation M10.)*

## Revue P2 (2026-11-18) — décisions de l'orchestrateur appliquées

**C-e431b-15 — Le nombre de lignes de dette recoupé** (LOW-1) : la vue d'un groupe fournisseur exige autant de lignes
de dette nommées (membres et détachées) que de factures soldées par le paiement, nombre lu sur son entrée
`reconciliation.accepted` (`details.supplier_invoices`, écrite dans la même transaction que l'écriture,
`group_size_at_payment`). C'est un **recoupement**, non une source : le compte d'écart se relit toujours sur
l'écriture (C-e431b-4). Entrée manquante ou multiple, nombre différent : `Invariant`. Autre option (proposée par la
revue) : recouper le compte d'écart rendu avec `difference_account_id` de la même entrée — écartée, le nombre de
factures ferme le cas sans comparer de valeur.

**C-e431b-16 — Les traces de détachement des seules factures annulées** (LOW-6) : `detached_debt_lines` ajoute
`si.status = 'cancelled'`, le filtre de `modification_guard`. Il borne le balayage et écarte une trace étrangère
restaurée ; une facture détachée puis repassée en brouillon ne compte plus, mais ce repassage exige l'écriture
contre-passée — la vue ne se lit plus.

**C-e431b-17 — Limite du test d'interblocage** (LOW-2, inscrite sans traitement, décision de l'orchestrateur) :
`a_fiche_payment_between_two_invoices_does_not_deadlock` attend la requête du groupe sur les motifs
`supplier_invoices` et `FOR UPDATE` — une requête **en cours**, non **en attente**. Sur un code fautif (sans la
lecture de tête), la sonde peut voir le verrou bref de la première facture avant que le groupe ait pris le compte ;
la fiche partirait alors trop tôt, prendrait le compte la première, et le test passerait sans former le cycle. Il ne
peut donc pas passer à tort sur le code sain, mais sa preuve contre le code fautif dépend du minutage (M8 a été tuée
à l'essai). Remède possible, non appliqué : ajouter le motif `FORCE INDEX (PRIMARY)` à la sonde (le test neuf
`the_bank_ledger_is_the_one_read_under_lock` le fait), ou attendre l'état `LOCK WAIT` de
`information_schema.INNODB_TRX`.

**C-e431b-18 — Le test de la 43-1a adapté, renommé** (LOW-7) : `a_group_without_client_settlements_is_an_invariant`
devient `a_malformed_supplier_group_is_an_invariant_for_the_view` ; son assertion porte sur la variante
(`DbError::Invariant`), non plus sur un texte. La branche défensive « aucun règlement » du dé-rapprochement groupé est
tenue par `a_group_cancel_on_an_entry_without_settlement_is_an_invariant` ; celle de `supplier_difference_account_of`
et de `supplier_cancel_in_tx` (membres vides) reste inatteignable par la classification, écrite comme telle.
*(Annoté à la revue P3 ciblée, LOW-B : la phrase se contredit pour `supplier_cancel_in_tx`. Sa branche « aucun
règlement » **est atteinte** — par l'aiguillage de `grouped_settlements::cancel_in_tx` (fonction publique), pour
toute écriture sans règlement client ni fournisseur ; c'est ce que fait le test cité. C'est la **classification**
(`reconciliation_cancel::classify`) qui n'y mène jamais : elle ne rend un groupe qu'avec un membre. Seule la branche
de `supplier_difference_account_of` (privée, appelée par la vue après la classification) reste inatteignable. Le
contrôle des positions de cette fonction, que le test adapté n'atteint plus (il tombe sur le recoupement, C-e431b-15),
est tenu par `debt_lines_out_of_their_positions_make_the_view_an_invariant`.)*

**C-e431b-19 — Reprise de `main` après l'Epic 37 (2026-11-18, branche `chore/43-reprise-main`)** — le détail commun
est à C-e431a-17 (registre de la 43-1a). Pour le bras fournisseur : `foreign: None` sur la ligne de banque (au crédit)
et sur chaque ligne de dette de `pay_supplier_invoices_in_tx`, et sur les lignes d'arrondi et d'écart
(`push_difference_lines`, commune) ; le refus `DifferenceAccountInForeignCurrency` vient de la règle commune
`difference_accounts` qu'a extraite la 43-1b (revue P1, B-3), sans code propre au bras. Test rouge d'abord
(`grouped_settlements_supplier.rs`, `a_foreign_currency_difference_account_is_refused` : rouge `MultiCurrencyDisabled`,
puis vert), avec l'ordre « non imputable avant devise » et une sonde positive.
