-- EXTRAIT de `crates/kesh-db/migrations/20260722000001_accounts_role_postable.sql`
-- (Story 14-3a, #269), rejoué après un restore d'installation — Story 16-1c, #281.
--
-- POURQUOI UN EXTRAIT ET PAS LE FICHIER ENTIER : la migration source mêle DDL et
-- données. La rejouer en bloc échouerait dès son premier `ALTER TABLE` (erreur
-- MariaDB 1060, colonne déjà présente). Seuls les 12 `UPDATE` de backfill sont
-- rejouables ; ils sont recopiés ci-dessous VERBATIM.
--
-- CLASSE B — rejeu CONDITIONNÉ. Deux de ces statements (les `postable`, en fin de
-- fichier) ne portent AUCUNE garde contre l'intention utilisateur : rejoués sur
-- une base à jour, ils écraseraient un `postable` posé à la main via
-- `PUT /api/v1/accounts/{id}` (sémantique full-replace). Le rejeu n'a donc lieu
-- que si `accounts.role` ou `accounts.postable` MANQUE au manifeste du backup
-- source — auquel cas le backup précède la migration et il n'existe aucune
-- intention à écraser. Cf. `post_restore.rs`, décision D-C1.
--
-- ⚠️ INTERDICTION DE REFORMATER. Ce fichier n'est pas un lieu où l'on nettoie :
-- le test `extract_statements_are_verbatim_substrings_of_source_migration`
-- vérifie que chaque statement est un sous-texte EXACT du SQL de la migration
-- tel qu'embarqué dans le `MIGRATOR`. Ne pas juger de l'utilité d'une clause non
-- plus — le `NOT EXISTS (journal_entry_lines)` du premier `UPDATE` de `postable`
-- est un prédicat STRUCTUREL de ciblage, pas une garde d'idempotence, et le
-- retirer changerait la population visée.
--
-- ⚠️ ORDRE À PRÉSERVER. Le dernier statement lit le rôle `CurrentYearResult` que
-- le neuvième vient de poser. Les réordonner casserait cette chaîne.

UPDATE accounts SET role = 'Receivable'        WHERE number = '1100' AND role IS NULL AND active = TRUE;
UPDATE accounts SET role = 'VatRecoverable'    WHERE number = '1171' AND role IS NULL AND active = TRUE;
UPDATE accounts SET role = 'Payable'           WHERE number = '2000' AND role IS NULL AND active = TRUE;
UPDATE accounts SET role = 'VatPayable'        WHERE number = '2200' AND role IS NULL AND active = TRUE;
UPDATE accounts SET role = 'VatSettlement'     WHERE number = '2206' AND role IS NULL AND active = TRUE;
UPDATE accounts SET role = 'EquityCapital'     WHERE number = '2800' AND role IS NULL AND active = TRUE;
UPDATE accounts SET role = 'EquityOther'       WHERE number IN ('2900', '2850', '2860') AND role IS NULL AND active = TRUE;
UPDATE accounts SET role = 'RetainedEarnings'  WHERE number = '2970' AND role IS NULL AND active = TRUE;
UPDATE accounts SET role = 'CurrentYearResult' WHERE number = '2979' AND role IS NULL AND active = TRUE;
UPDATE accounts SET role = 'DefaultRevenue'    WHERE number = '3000' AND role IS NULL AND active = TRUE;

UPDATE accounts a
   SET a.postable = FALSE
 WHERE EXISTS (
           SELECT 1 FROM accounts c WHERE c.parent_id = a.id AND c.active = TRUE
       )
   AND NOT EXISTS (
           SELECT 1 FROM journal_entry_lines l WHERE l.account_id = a.id
       );

UPDATE accounts SET postable = FALSE WHERE role = 'CurrentYearResult';
