-- Story 16-1a (#152, CR #265) — compte de produit par ligne de facture.
--
-- Chaque ligne de facture, et son snapshot d'avoir, peut porter son propre
-- compte de produit. `NULL` signifie « utiliser le compte de produit par
-- défaut de la société **au moment de la validation** » (liaison tardive,
-- décision D2) : le brouillon ne fige rien, la validation matérialise le
-- compte effectif dans cette colonne.
--
-- AUCUN BACKFILL ici (décision D2-bis). Le parc de factures **déjà validées**
-- avant le déploiement est traité par la Story 16-1a-bis, dont la migration
-- porte un timestamp postérieur à celle-ci. Tant qu'elle n'est pas livrée, ces
-- lignes conservent `NULL` — c'est-à-dire exactement le comportement
-- d'aujourd'hui (repli sur le défaut société), aucune régression.
--
-- Non-breaking (ADD COLUMN nullable + INDEX + FK) : un binaire antérieur
-- ignore la colonne, aucun `SELECT *` sur `invoice_lines` / `credit_note_lines`
-- dans le workspace (toutes les listes de colonnes sont explicites).
-- → pas de bump `kesh_version_min_required` (politique P1/P2 de CLAUDE.md),
-- donc pas de bump de version Cargo (P2-bis).
--
-- FK `ON DELETE RESTRICT` : convention unanime du dépôt pour les 11 FK vers
-- `accounts` — un compte référencé par une ligne de facture ne doit pas
-- disparaître sous l'écriture qu'il a produite.

ALTER TABLE invoice_lines
    ADD COLUMN revenue_account_id BIGINT NULL,
    ADD CONSTRAINT fk_invoice_lines_revenue_account
        FOREIGN KEY (revenue_account_id) REFERENCES accounts(id) ON DELETE RESTRICT;

CREATE INDEX idx_invoice_lines_revenue_account ON invoice_lines (revenue_account_id);

ALTER TABLE credit_note_lines
    ADD COLUMN revenue_account_id BIGINT NULL,
    ADD CONSTRAINT fk_credit_note_lines_revenue_account
        FOREIGN KEY (revenue_account_id) REFERENCES accounts(id) ON DELETE RESTRICT;

CREATE INDEX idx_credit_note_lines_revenue_account
    ON credit_note_lines (revenue_account_id);
