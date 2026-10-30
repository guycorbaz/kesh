-- Montage de TEST (Story v017-3b, #687) — jamais une base réelle.
--
-- Amène une facture fournisseur créée par Kesh à `1 × 100.00`, taux 0, OUVERTE,
-- à l'état exact qu'une version 0.16.x produisait pour `1 × 100.004` : la ligne de
-- facture, le TTC, et les deux lignes de l'écriture d'achat (charge au débit,
-- créanciers au crédit) à `100.0040`, l'écriture restant équilibrée.
--
-- Prérequis : la variable de session `@invoice_id` est posée sur la MÊME session
-- (ce fichier ne la pose pas — validation P6, C2) :
--   - l'aide de test `sous_centime.rs` la pose elle-même avant le texte ;
--   - sur la base de démonstration, l'opérateur tape
--       SET @invoice_id := (SELECT id FROM supplier_invoices WHERE supplier_invoice_number = '…');
--     puis `source montage-sous-centime.sql` dans la même session.
--
-- Pour l'état « payée », payer APRÈS ce fichier : le règlement lit le TTC sur la
-- ligne créanciers et écrit lui-même la ligne de banque à `100.0040`. Pour l'état
-- « annulée », annuler APRÈS ce fichier. Une facture par état.

SET @entry_id := (SELECT purchase_journal_entry_id FROM supplier_invoices WHERE id = @invoice_id AND status = 'open');

UPDATE supplier_invoice_lines
   SET unit_price = 100.0040, line_total = 100.0040
 WHERE supplier_invoice_id = @invoice_id AND quantity = 1 AND unit_price = 100.00;

UPDATE supplier_invoices
   SET total_amount = 100.0040
 WHERE id = @invoice_id AND status = 'open' AND total_amount = 100.00;

UPDATE journal_entry_lines
   SET debit = 100.0040
 WHERE entry_id = @entry_id AND debit = 100.00 AND credit = 0;

UPDATE journal_entry_lines
   SET credit = 100.0040
 WHERE entry_id = @entry_id AND credit = 100.00 AND debit = 0;
