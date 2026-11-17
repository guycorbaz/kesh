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

**C-e431b-4 — Le compte d'écart d'un groupe fournisseur relu sur l'écriture** (`supplier_difference_account_of`) :
le bloc des dettes va de la ligne 1 à la dernière ligne d'un membre, puis s'étend aux lignes suivantes dont le compte
est celui d'une ligne du bloc ; les apports sont les montants de ces lignes. La fiche (AC6 bis) écrivait « apports =
`total_amount` des membres » : c'est la même chose tant qu'aucune facture n'est détachée, mais une facture **détachée**
(AC7) n'est plus membre alors que sa ligne reste dans l'écriture — l'écriture lue par les membres seuls rendrait un
écart faux ou un `Invariant`, et la vue d'une transaction ainsi rapprochée échouerait. Le bloc s'étend sans erreur
parce qu'une ligne d'arrondi ou d'écart ne porte jamais une dette du groupe (refus au paiement, C-e431a-7). Angle
mort écrit au doc-comment : une facture détachée dont la dette est sur un **autre** compte que celle de toutes les
factures restantes, et qui était la dernière du groupe, serait lue comme une ligne de la queue ; le contrôle du nombre
de lignes rend alors `Invariant` (jamais une valeur fausse sans erreur, sauf coïncidence de montants). Test :
`a_detached_invoice_leaves_the_group_and_the_rest_unreconciles` (la dernière facture détachée). Le bras client est
inchangé.

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

**C-e431b-11 — Mutation M3 équivalente** : retirer `AND version = ?` (et sa liaison) de l'`UPDATE` des factures ne
change rien d'observable, la facture étant tenue `FOR UPDATE` depuis `lock_for_bank_settlement_in_tx` : aucune
version ne peut bouger entre la lecture et l'`UPDATE`. La clause reste (AC3 la demande, défense en profondeur).
