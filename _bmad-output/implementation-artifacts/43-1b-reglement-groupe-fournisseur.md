# Story 43-1b : un règlement pour plusieurs factures — factures fournisseur (#823)

Status: ready-for-dev

*Fiche rédigée le 2026-11-17 au soir par l'agent de rédaction de l'Epic 43 (Opus 5.5), workflow `bmad-create-story`,
découpage de la 43-1 (validation P1, H3) ; **alignée dans la nuit** sur la remédiation de la 43-1a (validation P2 : H-B,
M-A, M-B, M-C ; exigences de la 43-2). Code lu à `8b50fd195`. Release **0.19.0**. **Part de la 43-1a à son commit T3**
(migration, entités, signatures) ; se fusionne après elle. **Coupable à la coupure** (C-43-18) : sans elle, la migration
reste entière et la colonne fournisseur vide, la 43-2 et la 43-3 retirent leurs commits fournisseur. `kesh-db` touché :
gate complet en boucle. Décisions : C-43-1 à C-43-24 (fichier d'épique), qui font foi ; la règle de différence (C-43-16)
est une proposition en attente de Guy. Validation : à lancer.*

**Issues** : `refs #823` ; aucun mot-clé de fermeture dans un commit.

## Story

En tant que comptable qui paie plusieurs factures fournisseur (d'un ou de plusieurs fournisseurs) par un seul débit, je
veux qu'au rapprochement Kesh passe **un** paiement qui solde et lettre chacune, et qu'un dé-rapprochement le défasse.

## Ce qui se lit au code (`8b50fd195`)

- Acceptation unitaire : `accept_one_supplier_invoice` (`routes/reconciliation.rs:3312`), égalité exacte du montant
  (:3406), `supplier_invoices::lock_for_bank_settlement_in_tx` (`supplier_invoices.rs:2345` : facture `FOR UPDATE`, compte,
  lot `generated` en lecture verrouillante, garde du lot du compte), `pay_in_tx_as` (:1976 ; une écriture **par** facture,
  non réemployable telle quelle ; projet d'en-tête `inv.project_id` :2163 ; `UPDATE … settlement_journal_entry_id = ?`
  :2179 ; audit `supplier_invoice.paid` :2219), `purchase_payable_line` (:1891) ; fenêtre fournisseur
  `InvoiceDateWindow::for_supplier_transaction` (`repositories/reconciliation.rs:541`).
- Détachement : `cancel_in_tx` (:2530) remet les colonnes de règlement à `NULL` (:2639) et écrit
  `details.settlementJournalEntryId` ; annulation du règlement : `cancel_settlement_in_tx` (:2819, `UPDATE` :2929).
- Retour en brouillon d'une payée rapprochée refusé : `PaidReconciled` (:1194-1209).

## Critères d'acceptation

- **AC1** — `grouped_settlements::pay_supplier_invoices_in_tx(tx, company_id, bank_account_id, bank_transaction_id,
  supplier_invoice_ids, batch_size, acteur, aujourd_hui)` (signature posée par la 43-1a à T3), avec le **point de
  sauvegarde propre** de la 43-1a (D6) : bornes et doublon d'abord ; transaction (introuvable, non `pending`, pas un débit,
  non CHF ou nulle) ; puis la méthode de **son** unitaire : `lock_for_bank_settlement_in_tx` facture par facture,
  **identifiants croissants** (réemployé, non recopié) ; refus (`GroupRefusal`, `GroupedDocument::SupplierInvoice`, numéro
  affichable) : introuvable, non `open`, lot `generated`, tenue par un lot du compte, compte bancaire, date (borne basse
  seule : `InvoiceDateWindow::for_supplier_transaction(&tx).check(…)`, `PaymentBeforeInvoice` → refus,
  `PaymentTooLate` ignoré), compte de dette égal à la banque, et ceux de la différence et de `create_in_tx` (AC2). **Aucun
  refus de projet** (C-43-20).
- **AC2** — Écriture (sans projet d'en-tête, forme D7 de la 43-1a) : crédit banque de |montant| ; par facture, débit de
  **son** compte de dette (`purchase_payable_line`) de son `total_amount`, avec le projet de la facture estampillé sans
  validation. Différences (C-43-16, proposition en attente) : calculées par **la même** fonction pure
  `group_difference` que le bras client (43-1a D4 bis), avec ses définitions (`difference = |montant| − Σ brut`, signé ;
  `rounding_difference = r` ; `bank_fees = |e| ≥ 0`) : arrondi au compte d'arrondi (au débit si `r > 0`) ; `e > 0` (plus
  payé : frais) → débit du compte de frais bancaires, `|e| ≤` plafond (43-1a AC3 bis) sinon `DifferenceTooLarge` ; `e < 0`
  (moins payé) → refus `Underpayment` ; refus des comptes non configurés comme en 43-1a. Ni projet ni lettrage sur ces
  lignes ; `enforce_postable = false`.
- **AC3** — Chaque facture : `status = 'paid'`, `settlement_type = 'bank_transfer'`, `settlement_bank_account_id`,
  `settlement_journal_entry_id`, **`settlement_journal_entry_line_id`**, `paid_at`, `version + 1`, `WHERE … AND version = ?
  AND status = 'open'` (version lue sous le verrou de `lock_for_bank_settlement_in_tx`, qui **est** la méthode de l'unitaire
  fournisseur), `rows_affected() == 1`. Transaction rapprochée comme en 43-1a AC5 (`AND version = ?`) ;
  `recheck_open_locked_in_tx` en dernière acquisition.
- **AC4** — Audit (C-43-9) : `reconciliation.accepted` (`proposal_type: "supplier_invoice_group"`, clés snake_case,
  `batch_size`) d'abord, puis `supplier_invoice.paid` par facture avec les détails de `pay_in_tx_as` et l'acteur ; pas de
  doublon.
- **AC5** — Lettrage de chaque facture (`sync_supplier_invoice_in_tx`) : une clé par facture (sa ligne d'achat, sa ligne
  de dette).
- **AC6** — Dé-rapprochement : le bras fournisseur de `grouped_settlements::cancel_in_tx` (ordre de verrous de
  l'unitaire : facture fournisseur **avant** l'écriture, comme `supplier_invoices::cancel_settlement_in_tx`) ; refus par
  facture (`supplier_settlement_cancel_blocker`) avant toute écriture ; délettrage, factures rouvertes (`open`, colonnes
  `settlement_*` **ligne comprise** et `paid_at` à `NULL`, `version + 1`), contre-passation, audit
  `supplier_invoice.settlement_cancelled` (ou l'action de l'unitaire) par facture ; la facture qui bloque est rendue à
  `entry_cancel_outlook` comme côté client.
- **AC7** — **Les deux sites** qui remettent `settlement_journal_entry_id` à `NULL` — `supplier_invoices.rs:2639`
  (`cancel_in_tx`, détachement) et `:2929` (`cancel_settlement_in_tx`, inatteignable aujourd'hui pour un membre d'un groupe
  rapproché, mais gardé quand même) — emploient le fragment partagé `CLEAR_SETTLEMENT_SQL` de la 43-1a (AC1.3, T2), qui
  remet aussi la ligne à `NULL` (le `CHECK` rendrait sinon une 500, `4025`). Détachement d'une facture payée du groupe : `details` de l'audit gagne `settlementJournalEntryLineId`. Le dé-rapprochement ultérieur
  contre-passe l'écriture entière et ne rouvre que les factures encore attachées ; quand plus aucune ne l'est, l'écriture
  est classée `entry`. Faux positif bénin du gel « paiement détaché » (`find_modification_blocker`) écrit au doc-comment.
- **AC8** — Gestes unitaires refusés : retour en brouillon (`PaidReconciled`), annulation du règlement
  (`MATCHED_BANK_TRANSACTION`). Tests.

## Tests

Dans `crates/kesh-db/tests/grouped_settlements.rs` (module fournisseur) : groupe de deux fournisseurs ; facture d'un lot
`generated` refusée ; frais acceptés, au-delà du plafond refusés, moins payé refusé ; projets par ligne, **projet archivé
après coup accepté** ; date antérieure refusée ; un refus validé par le test laisse les tables inchangées (point de
sauvegarde) ; concurrence avec `pay_in_tx` depuis la fiche et avec `create_batch` ; dé-rapprochement ; détachement puis
dé-rapprochement (AC7) ; acteur clé d'API. Mutations (`kesh-gate-logs/e431b/mut/`) : omettre la ligne au détachement
(AC7 rougit) ; retirer `AND version = ?` ; inverser le sens de `e` (le test « moins payé refusé » rougit).

## Tâches

- [ ] **T0** — Étape 0 : LOW de la validation ; signatures du T3 de la 43-1a relues ; réponse de Guy sur M-A.
- [ ] **T1** — AC1 à AC5. — [ ] **T2** — AC6 à AC8. — [ ] **T3** — Gates, Dev Agent Record, prompt de revue P1.

## Note de version

- **Ajouté** — Un débit peut régler plusieurs factures fournisseur, d'un ou de plusieurs fournisseurs : un seul paiement,
  chaque facture soldée et lettrée ; des frais bancaires (5.00 au plus par défaut, réglable) passent au compte de frais
  bancaires (refs #823).

## Dev Notes

Modules : `repositories/grouped_settlements`, `repositories/supplier_invoices` (détachement), `reconciliation_cancel`
(bras). Trois.

## Dev Agent Record

### Agent Model Used

### Completion Notes List

### File List

## Change Log

- 2026-11-17 — Fiche née du découpage de la 43-1.
- 2026-11-17 (nuit) — Alignée sur la remédiation de la 43-1a : projet sans validation, différences (proposition M-A), borne
  basse de date, point de sauvegarde, signature figée, audit snake_case.
- 2026-11-17 (nuit, P3 de la 43-1a) — Les deux sites de remise à `NULL` et le fragment partagé (M2) ; la fonction
  `group_difference` commune et ses signes (M4) ; `enforce_postable = false` (L2).
