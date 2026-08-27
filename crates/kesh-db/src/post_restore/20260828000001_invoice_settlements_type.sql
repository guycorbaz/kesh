-- EXTRAIT de `crates/kesh-db/migrations/20260828000001_invoice_settlements_type.sql`
-- (Story 24-3, #372), rejoué après un restore d'installation.
--
-- POURQUOI UN EXTRAIT ET PAS LE FICHIER ENTIER : la migration source mêle DDL et
-- données. La rejouer en bloc échouerait dès son premier `ALTER TABLE` (erreur
-- MariaDB 1060, colonne déjà présente). Seul l'`UPDATE` de backfill est
-- rejouable ; il est recopié ci-dessous VERBATIM.
--
-- CLASSE B — rejeu CONDITIONNÉ à l'absence de `invoice_settlements.settlement_bank_account_id`.
--
-- La sentinelle est VALIDE parce que la colonne est ajoutée par le MÊME
-- `ALTER TABLE` que cet `UPDATE` : « colonne présente » implique donc bien
-- « backfill appliqué ». C'est la condition que
-- `class_b_sentinel_column_is_added_by_its_own_migration` verrouille.
--
-- ⚠️ Pourquoi B et non A, alors que l'`UPDATE` est gardé `IS NULL` et ne peut
-- écraser aucun choix utilisateur — la contrainte `chk_invoice_settlements_counterparty`
-- rendant ce `NULL` impossible sur un `bank_transfer` : parce qu'un rejeu
-- INCONDITIONNEL ferait exécuter cette jointure à CHAQUE import, pour un cas qui
-- ne peut plus se présenter. La sentinelle étant exacte ici, elle coûte moins et
-- dit la même chose.

UPDATE invoice_settlements s
JOIN journal_entry_lines jel
    ON jel.entry_id = s.journal_entry_id AND jel.debit > 0
JOIN bank_accounts ba
    ON ba.journal_account_id = jel.account_id AND ba.company_id = s.company_id
SET s.settlement_bank_account_id = ba.id
WHERE s.settlement_bank_account_id IS NULL;
