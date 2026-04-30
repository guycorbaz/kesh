-- Migration Story 7-4 / KF-005 : Index FULLTEXT pour recherche performante
-- sur les colonnes texte longues. Remplace `LIKE '%query%'` (full table scan)
-- par `MATCH AGAINST IN BOOLEAN MODE` côté repositories.
--
-- Cible 4 colonnes :
--   contacts.name
--   products.name
--   products.description
--   journal_entries.description
--
-- CONTRAINTES MariaDB 11.x InnoDB FULLTEXT (vérifiées doc 2026-04-29) :
--   1. `LOCK=NONE` PAS supporté pour `ADD FULLTEXT INDEX` → minimum requis
--      = `LOCK=SHARED` (lectures concurrentes OK, écritures bloquées le
--      temps du build d'index).
--   2. Un seul `ADD FULLTEXT` par `ALTER TABLE` quand `ALGORITHM=INPLACE`
--      (limitation InnoDB) → `products` doit être splitté en 2 statements
--      séquentiels.
--   3. `ALGORITHM` et `LOCK` sont des `alter_specification` items et
--      doivent donc être séparés par des virgules des autres specs (ADD).
--      Cf. doc MariaDB ALTER TABLE syntax. Une version antérieure de la
--      spec story 7-4 indiquait à tort « pas de virgule avant ALGORITHM » ;
--      la migration sqlx remontait alors une erreur 1064.
--   4. Le PREMIER `ADD FULLTEXT` sur une table déclenche une reconstruction
--      silencieuse de la table pour ajouter une colonne cachée `FTS_DOC_ID`
--      (même avec ALGORITHM=INPLACE). Acceptable v0.1 (volumes < 50k lignes
--      → reconstruction sub-secondaire). Les FULLTEXT ultérieurs sur la
--      même table (cas products: `ft_products_description` après
--      `ft_products_name`) n'ont plus besoin de cette reconstruction.
--
-- COMPORTEMENT TRANSACTIONNEL SQLx + MariaDB :
--   SQLx n'enveloppe PAS les migrations dans une transaction sur
--   MySQL/MariaDB (DDL = auto-commit côté serveur). Pas de rollback
--   atomique inter-statements. Si la 3e ou 4e ALTER échoue, les précédentes
--   restent persistées — le dev doit alors drop manuellement les index
--   appliqués avant de relancer la migration. Cf. docs/search-patterns.md
--   section « Procédure de récupération échec migration ».

ALTER TABLE contacts
    ADD FULLTEXT INDEX ft_contacts_name (name),
    ALGORITHM=INPLACE,
    LOCK=SHARED;

-- products : 2 ALTER séquentiels (limitation InnoDB un FULLTEXT à la fois en INPLACE)
ALTER TABLE products
    ADD FULLTEXT INDEX ft_products_name (name),
    ALGORITHM=INPLACE,
    LOCK=SHARED;

ALTER TABLE products
    ADD FULLTEXT INDEX ft_products_description (description),
    ALGORITHM=INPLACE,
    LOCK=SHARED;

ALTER TABLE journal_entries
    ADD FULLTEXT INDEX ft_journal_entries_description (description),
    ALGORITHM=INPLACE,
    LOCK=SHARED;
