-- Story 15-1a2-ii (#518) — rattrapage M2 : le lettrage des CONTRE-PASSATIONS
-- passées avant la 15-1a-ii, hors celles des écritures d'achat (portées par
-- 20261010000001, au registre de rejeu) : contre-passations d'écritures
-- manuelles, de rapprochements, de règlements annulés.
--
-- Non-breaking (P1) : un UPDATE de colonnes nullables, aucun DDL.
--
-- EXEMPTÉE DU REJEU (P7, EXEMPT_MIGRATIONS, ExemptionBasis::Durable) : un groupe
-- reversal sans ligne de pièce se délettre à la main (DELETE /letterings) — un
-- NULL peut y être un choix de l'utilisateur, qu'un rejeu à chaque import
-- réécrirait en silence. Coût assumé : une sauvegarde d'avant la 15-1a2-ii
-- importée laisse ces paires ouvertes, lettrables à la main.
--
-- Deux fichiers et non un : un extrait de registre doit porter TOUS les
-- statements d'écriture de sa migration, et une migration ne peut être à la
-- fois au registre et exemptée.
--
-- La forme est celle de la contre-passation (15-1a-ii R6) : chaque ligne de
-- l'origine, sur un compte lettrable, appariée PAR POSITION (rang dans ORDER BY
-- line_order) à la ligne du miroir (journal_entries.reverses_entry_id), même
-- compte, montants croisés ; au moins une des deux lignes en période ouverte
-- (exercice Open, aucun exercice postérieur Closed, date postérieure à
-- companies.books_locked_through) ; les deux non lettrées ; clé = plus petit id.
-- Les paires dont l'origine est une écriture d'achat sont EXCLUES explicitement
-- (NOT EXISTS ... purchase_journal_entry_id) : un écart futur entre les deux
-- migrations ne doit pas faire porter par celle-ci, non rejouée, une paire que
-- seule la précédente doit porter.
--
-- Une ligne d'origine déjà lettrée (groupe manuel posé avant la
-- contre-passation) garde son groupe, et son miroir reste ouvert — comme R6.
--
-- Aucune entrée d'audit : une migration n'a pas d'acteur.


UPDATE journal_entry_lines jel
JOIN (
    SELECT CASE cote.n WHEN 1 THEN p.origine ELSE p.miroir END AS line_id,
           LEAST(p.origine, p.miroir) AS k
    FROM (
        SELECT o.line_id AS origine, m.line_id AS miroir
        FROM (
            SELECT l.id AS line_id, l.entry_id, l.account_id, l.debit, l.credit,
                   l.lettering_key,
                   ROW_NUMBER() OVER (PARTITION BY l.entry_id ORDER BY l.line_order) AS rang,
                   (fy.status = 'Open'
                    AND NOT EXISTS (SELECT 1 FROM fiscal_years fl
                                    WHERE fl.company_id = fy.company_id
                                      AND fl.start_date > fy.start_date
                                      AND fl.status = 'Closed')
                    AND je.entry_date > COALESCE(co.books_locked_through, '0001-01-01')) AS ouverte
            FROM journal_entry_lines l
            JOIN journal_entries je ON je.id = l.entry_id
            JOIN fiscal_years fy ON fy.id = je.fiscal_year_id
            JOIN companies co ON co.id = je.company_id
            WHERE EXISTS (SELECT 1 FROM journal_entries r WHERE r.reverses_entry_id = l.entry_id)
        ) o
        JOIN journal_entries mj ON mj.reverses_entry_id = o.entry_id
        JOIN (
            SELECT l.id AS line_id, l.entry_id, l.account_id, l.debit, l.credit,
                   l.lettering_key,
                   ROW_NUMBER() OVER (PARTITION BY l.entry_id ORDER BY l.line_order) AS rang,
                   (fy.status = 'Open'
                    AND NOT EXISTS (SELECT 1 FROM fiscal_years fl
                                    WHERE fl.company_id = fy.company_id
                                      AND fl.start_date > fy.start_date
                                      AND fl.status = 'Closed')
                    AND je.entry_date > COALESCE(co.books_locked_through, '0001-01-01')) AS ouverte
            FROM journal_entry_lines l
            JOIN journal_entries je ON je.id = l.entry_id
            JOIN fiscal_years fy ON fy.id = je.fiscal_year_id
            JOIN companies co ON co.id = je.company_id
            WHERE je.reverses_entry_id IS NOT NULL
        ) m ON m.entry_id = mj.id AND m.rang = o.rang
        JOIN accounts a ON a.id = o.account_id
        WHERE m.account_id = o.account_id
          AND m.debit = o.credit AND m.credit = o.debit
          AND o.lettering_key IS NULL AND m.lettering_key IS NULL
          AND (o.ouverte OR m.ouverte)
          AND a.account_type IN ('Asset', 'Liability')
          AND NOT EXISTS (SELECT 1 FROM bank_accounts b WHERE b.journal_account_id = a.id)
          AND NOT EXISTS (SELECT 1 FROM supplier_invoices si
                      WHERE si.purchase_journal_entry_id = o.entry_id)
    ) p
    CROSS JOIN (SELECT 1 AS n UNION ALL SELECT 2) cote
) g ON g.line_id = jel.id
SET jel.lettering_key = g.k, jel.lettering_origin = 'reversal'
WHERE jel.lettering_key IS NULL;
