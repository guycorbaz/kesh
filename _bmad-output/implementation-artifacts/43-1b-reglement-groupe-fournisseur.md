# Story 43-1b : un règlement pour plusieurs factures — factures fournisseur (#823)

Status: ready-for-dev

*Fiche rédigée le 2026-11-17 au soir par l'agent de rédaction de l'Epic 43 (Opus 5.5), workflow `bmad-create-story`,
découpage de la 43-1 (validation P1, H3). Code lu à `8b50fd195`. Release **0.19.0**. **Part de la 43-1a à son commit T3**
(migration, entités, signatures) ; se fusionne après elle. **Coupable à la coupure** (C-43-18) : sans elle, la migration
reste entière et la colonne fournisseur vide, la 43-2 et la 43-3 n'exposent pas le type fournisseur. `kesh-db` touché :
gate complet en boucle. Décisions : C-43-1 à C-43-20 (fichier d'épique), qui font foi. Validation : à lancer.*

**Issues** : `refs #823` ; aucun mot-clé de fermeture dans un commit.

## Story

En tant que comptable qui paie plusieurs factures fournisseur (d'un ou de plusieurs fournisseurs) par un seul débit, je
veux qu'au rapprochement Kesh passe **un** paiement qui solde et lettre chacune, et qu'un dé-rapprochement le défasse.

## Ce qui se lit au code (`8b50fd195`)

- Acceptation unitaire : `accept_one_supplier_invoice` (`routes/reconciliation.rs:3312`), égalité exacte du montant
  (:3406), `supplier_invoices::lock_for_bank_settlement_in_tx` (`supplier_invoices.rs:2345` : facture `FOR UPDATE`, compte,
  lot `generated` en lecture verrouillante, garde du lot du compte), `pay_in_tx_as` (:1976 ; une écriture **par** facture,
  donc non réemployable telle quelle ; projet d'en-tête `inv.project_id` :2163 ; `UPDATE … settlement_journal_entry_id = ?`
  :2179 ; audit `supplier_invoice.paid` :2219), `purchase_payable_line` (:1891).
- Détachement : `cancel_in_tx` (:2530) remet les colonnes de règlement à `NULL` (:2639) et écrit
  `details.settlementJournalEntryId` ; annulation du règlement : `cancel_settlement_in_tx` (:2819, `UPDATE` :2929).
- Retour en brouillon d'une payée rapprochée refusé : `PaidReconciled` (:1194-1209).

## Critères d'acceptation

- **AC1** — `grouped_settlements::pay_supplier_invoices_in_tx` (signature posée par la 43-1a) : verrous par
  `lock_for_bank_settlement_in_tx`, facture par facture, **identifiants croissants** (réemployé, non recopié) ; refus
  (`GroupRefusal`) : ceux de la 43-1a qui s'appliquent (bornes, doublon, transaction non `pending`, compte, période,
  écart, compte d'arrondi, projet non imputable) et ceux de `SupplierSettlementRefusal` (introuvable, non `open`, lot
  `generated`, tenue par un lot du compte) avec `invoice_id`, et « la transaction n'est pas un débit ».
- **AC2** — Écriture (sans projet d'en-tête) : crédit banque de |montant| ; par facture, débit de **son** compte de dette
  (`purchase_payable_line`) de son `total_amount`, avec le projet de la facture ; ligne d'écart `d = |montant| − Σ
  total_amount` si `d ≠ 0`, au compte d'arrondi, au **débit** si `d > 0` (trop payé), au crédit sinon, sans projet,
  plafond `GROUP_DIFFERENCE_MAX` (C-43-16).
- **AC3** — Chaque facture : `status = 'paid'`, `settlement_type = 'bank_transfer'`, `settlement_bank_account_id`,
  `settlement_journal_entry_id`, **`settlement_journal_entry_line_id`**, `paid_at`, `version + 1`, `WHERE … AND version = ?
  AND status = 'open'` (version lue sous le verrou), `rows_affected() == 1`. Transaction rapprochée comme en 43-1a AC5 ;
  `recheck_open_locked_in_tx` en dernière acquisition.
- **AC4** — Audit (C-43-9) : `reconciliation.accepted` (`proposalType: "supplier_invoice_group"`) d'abord, puis
  `supplier_invoice.paid` par facture avec les détails de `pay_in_tx_as` et l'acteur ; **pas** de doublon.
- **AC5** — Lettrage de chaque facture (`sync_supplier_invoice_in_tx`) : une clé par facture (sa ligne d'achat, sa ligne
  de dette).
- **AC6** — Dé-rapprochement : le bras fournisseur de `grouped_settlements::cancel_in_tx` (ordre de verrous de l'unitaire :
  facture fournisseur **avant** l'écriture, comme `supplier_invoices::cancel_settlement_in_tx`) ; refus par facture
  (`supplier_settlement_cancel_blocker`) avant toute écriture ; délettrage, factures rouvertes (`open`, colonnes
  `settlement_*` **ligne comprise** et `paid_at` à `NULL`, `version + 1`), contre-passation, audit
  `supplier_invoice.settlement_cancelled` (ou l'action de l'unitaire) par facture.
- **AC7** — Détachement (`cancel_in_tx` d'une facture payée du groupe) : la ligne passe aussi à `NULL` (le `CHECK` de la
  migration rougit sinon) ; `details` de l'audit gagne `settlementJournalEntryLineId` (L7). Le dé-rapprochement ultérieur
  contre-passe l'écriture entière et ne rouvre que les factures encore attachées ; quand plus aucune ne l'est, l'écriture
  est classée `entry`. Faux positif bénin du gel « paiement détaché » (`find_modification_blocker`) écrit au doc-comment.
- **AC8** — Gestes unitaires refusés : retour en brouillon (`PaidReconciled`), annulation du règlement
  (`MATCHED_BANK_TRANSACTION`). Tests.

## Tests

Dans `crates/kesh-db/tests/grouped_settlements.rs` (module fournisseur) : groupe de deux fournisseurs ; facture d'un lot
`generated` refusée ; écart accepté et refusé ; projets par ligne ; concurrence avec `pay_in_tx` depuis la fiche et avec
`create_batch` ; dé-rapprochement ; détachement puis dé-rapprochement (AC7) ; acteur clé d'API. Mutations
(`kesh-gate-logs/e431b/mut/`) : omettre la ligne au détachement (AC7 rougit) ; verrous dans l'ordre décroissant (le test
d'interblocage rougit ou se rejoue — à constater) ; retirer `AND version = ?`.

## Tâches

- [ ] **T0** — Étape 0 : LOW de la validation ; ligne lue de `lock_for_bank_settlement_in_tx` pour N factures (la garde (d)
  se lit avec le montant de la transaction).
- [ ] **T1** — AC1 à AC5. — [ ] **T2** — AC6 à AC8. — [ ] **T3** — Gates, Dev Agent Record, prompt de revue P1.

## Note de version

- **Ajouté** — Un débit peut régler plusieurs factures fournisseur, d'un ou de plusieurs fournisseurs : un seul paiement,
  chaque facture soldée et lettrée (refs #823).

## Dev Notes

Modules : `repositories/grouped_settlements`, `repositories/supplier_invoices` (détachement), `reconciliation_cancel`
(bras). Trois.

## Dev Agent Record

### Agent Model Used

### Completion Notes List

### File List

## Change Log

- 2026-11-17 — Fiche née du découpage de la 43-1.
