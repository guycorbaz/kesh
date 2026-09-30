-- Story 25-4-c3-a1 (#476) — le compte de différences d'arrondi, désigné dans
-- les paramètres de facturation.
--
-- Arbitrage du Project Lead (2026-09-30) : l'écart d'arrondi au centime
-- qu'un paiement arrondi laisse sur une facture se PASSE EN ÉCRITURE, sur un
-- compte désigné par un RÉGLAGE (pas un rôle de compte). Le compte se crée
-- dans le plan comptable et se choisit ici ; les plans livrés en proposeront
-- un (Story 25-4-c3-a2). La Story 25-4-c3-b écrira l'écart en lisant ce réglage.
--
-- Patron : les comptes TVA (`20260614000001_vat_accounts_config.sql`) — colonne
-- facultative, FK `ON DELETE RESTRICT`.
--
-- DDL seul, AUCUNE donnée écrite : les sociétés existantes désignent leur
-- compte elles-mêmes (triage P7 sans objet). Non-breaking (`ADD COLUMN`
-- nullable, ignoré par les anciens binaires, qui listent leurs colonnes) →
-- PAS de bump `kesh_version_min_required`. Ligne ajoutée à
-- `docs/migrations-idempotence-audit.md` (politique P5).

ALTER TABLE company_invoice_settings
    ADD COLUMN default_rounding_account_id BIGINT NULL,
    ADD CONSTRAINT fk_cis_rounding
        FOREIGN KEY (default_rounding_account_id) REFERENCES accounts(id) ON DELETE RESTRICT;
