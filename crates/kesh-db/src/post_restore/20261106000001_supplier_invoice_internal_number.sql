-- EXTRAIT de `crates/kesh-db/migrations/20261106000001_supplier_invoice_internal_number.sql`
-- (Story 33-1a, #926), rejoué après un restore d'installation.
--
-- POURQUOI UN EXTRAIT ET PAS LE FICHIER ENTIER : la migration source mêle DDL et
-- données. La rejouer en bloc échouerait dès son premier `ALTER TABLE` (erreur
-- MariaDB 1060, colonne déjà présente). Seuls les deux `UPDATE` sont rejouables ;
-- ils sont recopiés ci-dessous VERBATIM.
--
-- CLASSE B — rejeu CONDITIONNÉ à l'absence de `supplier_invoices.internal_number`.
--
-- La sentinelle est VALIDE parce que la colonne est ajoutée par le MÊME fichier que
-- ces `UPDATE` : « colonne présente » implique donc bien « backfill appliqué ». Une
-- sauvegarde 0.17.4 reçoit ainsi les numéros et les compteurs de la migration ; une
-- sauvegarde 0.17.5 garde les siens, trous compris (C-e331-13).

UPDATE supplier_invoices si
  JOIN (
    SELECT s.id, YEAR(f.start_date) AS annee,
           ROW_NUMBER() OVER (PARTITION BY s.company_id, YEAR(f.start_date)
                              ORDER BY f.start_date, s.invoice_date, s.id) AS rang
      FROM supplier_invoices s
      JOIN journal_entries je ON je.id = s.purchase_journal_entry_id AND je.company_id = s.company_id
      JOIN fiscal_years f ON f.id = je.fiscal_year_id AND f.company_id = s.company_id
     WHERE s.status <> 'draft'
  ) n ON n.id = si.id
   SET si.internal_number = CONCAT('FF-', n.annee, '-', LPAD(n.rang, GREATEST(3, CHAR_LENGTH(n.rang)), '0')),
       si.updated_at = si.updated_at
 WHERE si.internal_number IS NULL;

UPDATE fiscal_years fy
  JOIN (
    SELECT r.fiscal_year_id, MAX(r.rang) AS dernier
      FROM (
        SELECT f.id AS fiscal_year_id,
               ROW_NUMBER() OVER (PARTITION BY s.company_id, YEAR(f.start_date)
                                  ORDER BY f.start_date, s.invoice_date, s.id) AS rang
          FROM supplier_invoices s
          JOIN journal_entries je ON je.id = s.purchase_journal_entry_id AND je.company_id = s.company_id
          JOIN fiscal_years f ON f.id = je.fiscal_year_id AND f.company_id = s.company_id
         WHERE s.status <> 'draft'
      ) r
     GROUP BY r.fiscal_year_id
  ) c ON c.fiscal_year_id = fy.id
   SET fy.supplier_invoice_next_number = GREATEST(fy.supplier_invoice_next_number, c.dernier + 1),
       fy.updated_at = fy.updated_at;
