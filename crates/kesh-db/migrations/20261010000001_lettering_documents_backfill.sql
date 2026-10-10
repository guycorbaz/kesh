-- Story 15-1a2-ii (#518) — rattrapage M1 : le lettrage des PIÈCES soldées avant
-- la mise à jour, et des contre-passations d'achat (factures fournisseurs
-- annulées avant la 15-1a-ii).
--
-- Non-breaking (P1) : trois UPDATE de colonnes nullables, aucun DDL. Pas de
-- relèvement de kesh_version_min_required (porté à 0.13.0 par 20261009000001).
--
-- REGISTRE DE REJEU, CLASSE A (P7, crates/kesh-db/src/post_restore.rs) : rejouée
-- à chaque import d'installation. Chaque statement est gardé par
-- « lettering_key IS NULL » sur TOUTES les lignes du groupe candidat — le rejeu
-- ne délettre jamais et ne réécrit jamais une marque posée. Il PEUT lettrer ce
-- que la base avait laissé ouvert à bon droit (pièce historique dont l'exercice
-- a été rouvert depuis, compte devenu lettrable) : la justification du registre
-- le dit.
--
-- La règle est celle de la synchronisation des pièces (letterings.rs,
-- sync_invoice_in_tx et sync_supplier_invoice_in_tx), recopiée en SQL parce
-- qu'une migration appliquée ne se modifie plus (P8). Le test d'accord
-- crates/kesh-db/tests/lettering_documents_backfill.rs tient les deux ensemble.
--
-- Règles communes :
--   compte lettrable : account_type Asset ou Liability, et aucun compte bancaire
--     ne le porte (letterings::letterable_account) ;
--   au moins une ligne du groupe en période ouverte : exercice Open, aucun
--     exercice postérieur Closed de la société, date strictement postérieure à
--     companies.books_locked_through — abstention sinon ;
--   au moins deux lignes, somme nulle, toutes les lignes non lettrées ;
--   clé = plus petit id de ligne du groupe.
--
-- Forme : UPDATE joint à une table dérivée (matérialisée : fonctions de fenêtre),
-- jamais WITH en tête — le détecteur de triage P7 classe un statement sur son
-- premier mot-clé. Aucun commentaire de bloc, aucun littéral contenant un
-- point-virgule ou un double tiret (registry_sql_has_no_literal_hazard).
--
-- Aucune entrée d'audit : une migration n'a pas d'acteur (manuel utilisateur).


-- (1) Les factures CLIENTES. C(I) = l'ancre (première ligne au débit de
--     l'écriture de vente) et les lignes sur son compte A des écritures des
--     règlements en vigueur (invoice_settlements, soldes compris) et de l'avoir
--     émis. Les écritures sont énumérées par pièce, puis leurs lignes lues par
--     entry_id.
UPDATE journal_entry_lines jel
JOIN (
    SELECT t.line_id, t.k
    FROM (
        SELECT c.line_id,
               MIN(c.line_id) OVER (PARTITION BY c.piece_id) AS k,
               COUNT(*) OVER (PARTITION BY c.piece_id) AS n,
               SUM(c.debit - c.credit) OVER (PARTITION BY c.piece_id) AS solde,
               SUM(c.lettering_key IS NOT NULL) OVER (PARTITION BY c.piece_id) AS deja,
               SUM(c.ouverte) OVER (PARTITION BY c.piece_id) AS ouvertes
        FROM (
            SELECT e.piece_id, l.id AS line_id, l.debit, l.credit, l.lettering_key,
                   (fy.status = 'Open'
                    AND NOT EXISTS (SELECT 1 FROM fiscal_years fl
                                    WHERE fl.company_id = fy.company_id
                                      AND fl.start_date > fy.start_date
                                      AND fl.status = 'Closed')
                    AND je.entry_date > COALESCE(co.books_locked_through, '0001-01-01')) AS ouverte
            FROM (
                SELECT i.id AS piece_id, i.journal_entry_id AS entry_id, 1 AS vente
                FROM invoices i WHERE i.journal_entry_id IS NOT NULL
                UNION ALL
                SELECT s.invoice_id, s.journal_entry_id, 0
                FROM invoice_settlements s
                UNION ALL
                SELECT cn.invoice_id, cn.journal_entry_id, 0
                FROM credit_notes cn
                WHERE cn.status = 'issued' AND cn.journal_entry_id IS NOT NULL
            ) e
            JOIN invoices i ON i.id = e.piece_id AND i.journal_entry_id IS NOT NULL
            JOIN journal_entry_lines ancre
              ON ancre.id = (SELECT MIN(x.id) FROM journal_entry_lines x
                             WHERE x.entry_id = i.journal_entry_id AND x.debit > 0)
            JOIN accounts a ON a.id = ancre.account_id
            JOIN journal_entry_lines l
              ON l.entry_id = e.entry_id AND l.account_id = ancre.account_id
             AND (e.vente = 0 OR l.id = ancre.id)
            JOIN journal_entries je ON je.id = l.entry_id AND je.company_id = i.company_id
            JOIN fiscal_years fy ON fy.id = je.fiscal_year_id
            JOIN companies co ON co.id = je.company_id
            WHERE a.account_type IN ('Asset', 'Liability')
              AND NOT EXISTS (SELECT 1 FROM bank_accounts b WHERE b.journal_account_id = a.id)
        ) c
    ) t
    WHERE t.n >= 2 AND t.solde = 0 AND t.deja = 0 AND t.ouvertes > 0
) g ON g.line_id = jel.id
SET jel.lettering_key = g.k, jel.lettering_origin = 'document'
WHERE jel.lettering_key IS NULL;


-- (2) Les factures FOURNISSEURS payées. C(S) = l'ancre (première ligne au
--     crédit de l'écriture d'achat, sur la dette B) et la ligne sur B de
--     l'écriture de règlement en vigueur. Une facture ouverte n'a jamais de
--     groupe, une facture annulée non plus (son achat est contre-passé).
UPDATE journal_entry_lines jel
JOIN (
    SELECT t.line_id, t.k
    FROM (
        SELECT c.line_id,
               MIN(c.line_id) OVER (PARTITION BY c.piece_id) AS k,
               COUNT(*) OVER (PARTITION BY c.piece_id) AS n,
               SUM(c.debit - c.credit) OVER (PARTITION BY c.piece_id) AS solde,
               SUM(c.lettering_key IS NOT NULL) OVER (PARTITION BY c.piece_id) AS deja,
               SUM(c.ouverte) OVER (PARTITION BY c.piece_id) AS ouvertes
        FROM (
            SELECT e.piece_id, l.id AS line_id, l.debit, l.credit, l.lettering_key,
                   (fy.status = 'Open'
                    AND NOT EXISTS (SELECT 1 FROM fiscal_years fl
                                    WHERE fl.company_id = fy.company_id
                                      AND fl.start_date > fy.start_date
                                      AND fl.status = 'Closed')
                    AND je.entry_date > COALESCE(co.books_locked_through, '0001-01-01')) AS ouverte
            FROM (
                SELECT si.id AS piece_id, si.purchase_journal_entry_id AS entry_id, 1 AS achat
                FROM supplier_invoices si
                WHERE si.status = 'paid' AND si.settlement_journal_entry_id IS NOT NULL
                UNION ALL
                SELECT si.id, si.settlement_journal_entry_id, 0
                FROM supplier_invoices si
                WHERE si.status = 'paid' AND si.settlement_journal_entry_id IS NOT NULL
            ) e
            JOIN supplier_invoices s ON s.id = e.piece_id
            JOIN journal_entry_lines ancre
              ON ancre.id = (SELECT MIN(x.id) FROM journal_entry_lines x
                             WHERE x.entry_id = s.purchase_journal_entry_id AND x.credit > 0)
            JOIN accounts a ON a.id = ancre.account_id
            JOIN journal_entry_lines l
              ON l.entry_id = e.entry_id AND l.account_id = ancre.account_id
             AND (e.achat = 0 OR l.id = ancre.id)
            JOIN journal_entries je ON je.id = l.entry_id AND je.company_id = s.company_id
            JOIN fiscal_years fy ON fy.id = je.fiscal_year_id
            JOIN companies co ON co.id = je.company_id
            WHERE a.account_type IN ('Asset', 'Liability')
              AND NOT EXISTS (SELECT 1 FROM bank_accounts b WHERE b.journal_account_id = a.id)
        ) c
    ) t
    WHERE t.n >= 2 AND t.solde = 0 AND t.deja = 0 AND t.ouvertes > 0
) g ON g.line_id = jel.id
SET jel.lettering_key = g.k, jel.lettering_origin = 'document'
WHERE jel.lettering_key IS NULL;


-- (3) Les paires de CONTRE-PASSATION dont l'origine est l'écriture d'ACHAT
--     d'une facture fournisseur (factures annulées avant la 15-1a-ii). La forme
--     de la contre-passation (15-1a-ii R6) : chaque ligne de l'origine, sur un
--     compte lettrable, appariée PAR POSITION (rang dans ORDER BY line_order)
--     à la ligne du miroir (journal_entries.reverses_entry_id), même compte,
--     montants croisés. Les autres paires sont à la migration suivante,
--     EXEMPTÉE du rejeu (20261010000002) : celle-ci les exclut, celle-là
--     exclut celles-ci.
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
          AND EXISTS (SELECT 1 FROM supplier_invoices si
                      WHERE si.purchase_journal_entry_id = o.entry_id)
    ) p
    CROSS JOIN (SELECT 1 AS n UNION ALL SELECT 2) cote
) g ON g.line_id = jel.id
SET jel.lettering_key = g.k, jel.lettering_origin = 'reversal'
WHERE jel.lettering_key IS NULL;
