-- Story 16-2a (#144) — compte de produit par défaut sur la fiche produit.
--
-- Chaque article du catalogue peut porter **son** compte de produit. `NULL`
-- signifie « cet article n'impose rien » : la ligne de facture montée depuis
-- cet article reste alors à `NULL` et suit le compte de produit par défaut de
-- la société, résolu à la validation — c'est-à-dire exactement le comportement
-- livré par la Story 16-1a, sans changement.
--
-- AUCUNE REPRISE RÉTROACTIVE (décision D7) : DDL pur, aucun `UPDATE`, aucun
-- `INSERT`. Les articles existants naissent à `NULL`, comportement actuel
-- strictement préservé. Cette migration ne relève donc ni du registre
-- `POST_RESTORE_BACKFILLS` ni des exemptions (garde-fou P7) : elle n'écrit
-- aucune donnée, il n'y a rien à rejouer après un import d'installation.
--
-- Non-breaking (ADD COLUMN nullable + FK + INDEX) : un binaire antérieur
-- ignore la colonne, et aucun `SELECT *` sur `products` n'existe dans le
-- workspace — les deux listes de colonnes du repository sont explicites.
-- → pas de bump `kesh_version_min_required` (politique P1/P2 de CLAUDE.md),
-- donc pas de bump de version Cargo (P2-bis).
--
-- FK `ON DELETE RESTRICT` : convention unanime du dépôt, 13 FK vers `accounts`
-- au moment de l'écriture — un compte choisi sur une fiche produit ne doit pas
-- disparaître sous les factures qu'il a imputées.
--
-- INDEX nommé : `company_invoice_settings` n'en déclare pas sur sa propre
-- colonne `default_revenue_account_id` et fonctionne — InnoDB crée lui-même
-- l'index nécessaire à une FK quand aucun ne convient. On le nomme quand même,
-- sur le patron de `idx_invoice_lines_revenue_account` (Story 16-1a) : un index
-- explicite est inspectable et porte une convention de nommage stable, plutôt
-- que de dépendre d'un choix implicite du moteur. Aucun besoin de requête ne le
-- motive — rien ne filtre `products` par compte.

ALTER TABLE products
    ADD COLUMN default_revenue_account_id BIGINT NULL,
    ADD CONSTRAINT fk_products_default_revenue_account
        FOREIGN KEY (default_revenue_account_id) REFERENCES accounts(id) ON DELETE RESTRICT;

CREATE INDEX idx_products_default_revenue_account
    ON products (default_revenue_account_id);
