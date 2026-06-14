-- Migration Story 11-1 : CRUD admin des taux TVA — colonnes `version` + `category`.
--
-- Contexte : la table `vat_rates` (Story 7.2 / KF-003) était read-only v0.1.
-- Story 11-1 introduit le CRUD admin. Deux ajouts :
--
--   1. `version INT NOT NULL DEFAULT 0` — verrou optimiste pour les UPDATE
--      (pattern `bank_accounts`). Les lignes existantes (seed/backfill) prennent 0.
--
--   2. `category VARCHAR(32) NOT NULL DEFAULT 'custom'` — discriminant métier
--      STABLE et EXTENSIBLE de catégorie TVA, distinct du `label` d'affichage.
--      Permet de suivre « le taux normal » au fil des années (7.7 % → 8.1 %).
--      **PAS de contrainte `CHECK IN (liste fermée)`** : décision projet
--      (Story 11-1) — les autorités peuvent introduire de NOUVELLES catégories
--      officielles sans migration de schéma (une nouvelle catégorie = une
--      nouvelle valeur de clé). Seule contrainte : non vide.
--      Clés réservées connues : normal / reduced / special / exempt ; custom = libre.
--
-- Backfill `category` des taux seedés depuis leur `label` (idempotent : un
-- re-jeu laisse les valeurs déjà posées inchangées via le CASE/ELSE).
--
-- Index `(company_id, category, active)` pour `find_for_category_at_date`.
-- L'index existant `idx_vat_rates_company_active (company_id, active)` est
-- CONSERVÉ (sert `list_active_for_company` sans filtre catégorie — non couvert
-- en préfixe par le nouvel index ordonné).
--
-- Non-breaking (`ADD COLUMN … DEFAULT` ignoré par les anciens binaires) →
-- pas de bump `kesh_version_min_required`.

ALTER TABLE vat_rates
    ADD COLUMN version INT NOT NULL DEFAULT 0,
    ADD COLUMN category VARCHAR(32) NOT NULL DEFAULT 'custom',
    ADD CONSTRAINT chk_vat_rates_category_not_empty CHECK (CHAR_LENGTH(TRIM(category)) > 0);

-- Backfill des catégories des 4 taux suisses seedés depuis leur clé i18n `label`.
UPDATE vat_rates
SET category = CASE label
        WHEN 'product-vat-normal'  THEN 'normal'
        WHEN 'product-vat-reduced' THEN 'reduced'
        WHEN 'product-vat-special' THEN 'special'
        WHEN 'product-vat-exempt'  THEN 'exempt'
        ELSE category
    END
WHERE label IN ('product-vat-normal', 'product-vat-reduced', 'product-vat-special', 'product-vat-exempt');

CREATE INDEX idx_vat_rates_company_category_active ON vat_rates (company_id, category, active);
