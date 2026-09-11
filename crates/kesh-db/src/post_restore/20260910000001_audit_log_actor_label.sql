-- EXTRAIT de `crates/kesh-db/migrations/20260910000001_audit_log_actor_label.sql`
-- (Story 25-1a, #377), rejoué après un restore d'installation.
--
-- POURQUOI UN EXTRAIT ET PAS LE FICHIER ENTIER : la migration source mêle DDL et
-- données. La rejouer en bloc échouerait dès son `ALTER TABLE` (erreur MariaDB
-- 1060, colonne déjà présente). Seul l'`UPDATE` de backfill est rejouable ; il
-- est recopié ci-dessous VERBATIM.
--
-- CLASSE B — rejeu CONDITIONNÉ à l'absence de `audit_log.actor_label` au
-- manifeste. La sentinelle est VALIDE parce que la colonne est ajoutée par le
-- MÊME `ALTER TABLE` que cet `UPDATE`.
--
-- ⚠️ CE QUE LA JOINTURE DÉSIGNE, ET POURQUOI ELLE NE SE TROMPE PAS ICI.
-- L'objection naturelle est que `user_id` appartient à l'espace d'identifiants de
-- l'instance SOURCE, et que le résoudre contre la table `users` locale nommerait
-- la mauvaise personne. Elle ne tient pas **dans ce flux**, pour deux raisons qui
-- se complètent :
--
--   (a) l'ORDRE — ce rejeu tourne APRÈS `restore_tables_in_tx`
--       (`routes/admin.rs`, étape 5-bis), donc `users` a déjà été remplacée par
--       celle du backup : l'espace d'identifiants courant EST celui de la source ;
--   (b) la GARDE — `actor_label = ''` n'atteint que les lignes venues d'une
--       instance antérieure à cette migration. Les entrées LOCALES conservées par
--       la fusion portent déjà leur libellé, posé à l'écriture par le sous-SELECT
--       du repository, et sont donc hors d'atteinte. C'est ce qui protège
--       l'instantané : la garde, pas la sentinelle.
--
-- ⛔ REJEU MANUEL, HORS FLUX D'IMPORT. Exécuter ce SQL à la main est SANS EFFET
-- sur une base à jour (la garde ne trouve rien) — mais le faire sur une base où
-- des entrées d'archive attendent encore leur libellé, SANS que `users` ait été
-- remplacée par celle de leur instance d'origine, viole (a) et attribue les
-- entrées au porteur ACTUEL de chaque identifiant. La piste mentirait alors
-- silencieusement sur qui a fait quoi — le défaut même que la Story 25-1a ferme.
-- Le rejeu n'existe qu'en tant qu'étape de l'import : il n'y a aucune voie
-- supportée qui rejouerait sans restaurer.

UPDATE audit_log a
    LEFT JOIN users u ON u.id = a.user_id
   SET a.actor_label = COALESCE(u.username, '(inconnu)')
 WHERE a.actor_label = '';
