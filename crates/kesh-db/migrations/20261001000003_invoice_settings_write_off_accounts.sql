-- Story 25-4-d1 (#384) — les comptes des natures d'écart soldé, désignés dans
-- les paramètres de facturation.
--
-- Arbitrage du Project Lead (2026-10-01) : solder le reste d'une facture impute
-- l'écart au compte de sa NATURE — escompte accordé, frais bancaires retenus par
-- la banque du client, perte sur débiteur. Un RÉGLAGE par nature (pas un rôle de
-- compte), désigné d'office à la création d'une société depuis le marqueur
-- `writeOffNature` du plan. La Story 25-4-d2 écrira le solde en lisant ces
-- réglages.
--
-- Patron : le compte de différences d'arrondi
-- (`20260930000001_invoice_settings_rounding_account.sql`) — colonne
-- facultative, FK `ON DELETE RESTRICT`.
--
-- DDL seul, AUCUNE donnée écrite : les sociétés existantes désignent leurs
-- comptes elles-mêmes (triage P7 sans objet). Non-breaking (`ADD COLUMN`
-- nullable, ignoré par les anciens binaires, qui listent leurs colonnes) →
-- PAS de bump `kesh_version_min_required`. Ligne ajoutée à
-- `docs/migrations-idempotence-audit.md` (politique P5).

ALTER TABLE company_invoice_settings
    ADD COLUMN default_discount_account_id BIGINT NULL,
    ADD COLUMN default_bank_fees_account_id BIGINT NULL,
    ADD COLUMN default_bad_debt_account_id BIGINT NULL,
    ADD CONSTRAINT fk_cis_discount
        FOREIGN KEY (default_discount_account_id) REFERENCES accounts(id) ON DELETE RESTRICT,
    ADD CONSTRAINT fk_cis_bank_fees
        FOREIGN KEY (default_bank_fees_account_id) REFERENCES accounts(id) ON DELETE RESTRICT,
    ADD CONSTRAINT fk_cis_bad_debt
        FOREIGN KEY (default_bad_debt_account_id) REFERENCES accounts(id) ON DELETE RESTRICT;
