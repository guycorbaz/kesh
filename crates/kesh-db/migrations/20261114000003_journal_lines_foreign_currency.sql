-- Non breaking (P1, P3) : ADD COLUMN NULL et ADD CONSTRAINT seulement. Un
-- binaire 0.17.6 ignore les colonnes neuves (aucun SELECT * ne les décode) ;
-- kesh_version_min_required reste à 0.17.6.
--
-- Story 37-3a (Epic 37, multi-devises) — lignes d'écriture en devise.
--
-- journal_entry_lines gagne trois colonnes, NULL ensemble pour une ligne en
-- devise principale (CHF) :
--   amount_currency  montant en devise (4 décimales) ;
--   exchange_rate    cours par unité de devise (10 décimales) ;
--   rate_source      provenance du cours : afc, manual, revaluation.
-- AUCUNE colonne de devise (CR-4, T3) : la devise d'une ligne est celle de son
-- compte (accounts.currency, Story 37-1), lue par jointure ; elle est
-- verrouillée dès la première ligne du compte (37-1, AC-4), ce qui garantit
-- qu'une ligne passée garde sa devise.
--
-- Contraintes NOMMÉES, chacune tolérante à la ligne CHF (trois NULL). Elles
-- sont un FILET structurel : une violation rend le 400 générique
-- CHECK_CONSTRAINT_VIOLATION ; les refus précis viennent de kesh-core et du
-- dépôt, qui les précèdent. chk_jel_fx_chf couvre les chemins qui n'appellent
-- pas la validation (contre-passation, restauration .keshbackup). Le ROUND de
-- MariaDB sur un DECIMAL arrondit l'équidistant loin de zéro, comme
-- kesh-core. chk_jel_fx_chf est déclarée EN DERNIER (ordre d'évaluation des
-- CHECK).
--
-- rate_source comparé À L'OCTET (BINARY), comme account_kind de
-- 20261110000001 : sous la collation de la table, 'AFC' passerait.
--
-- ⚠️ Ce que le schéma ne garde pas : aucune contrainte de table ne lie une
-- ligne en devise à un compte en devise (un CHECK ne lit pas accounts) ; une
-- ligne à amount_currency non NULL sur un compte CHF n'est refusée que par le
-- dépôt (FX_ACCOUNT_IN_CHF).
--
-- Aucune écriture de données (P7 : rien à trier). Ligne ajoutée à
-- docs/migrations-idempotence-audit.md (P5).
ALTER TABLE journal_entry_lines
    ADD COLUMN amount_currency DECIMAL(19,4) NULL
        COMMENT 'Montant en devise du compte, NULL = ligne en devise principale',
    ADD COLUMN exchange_rate DECIMAL(20,10) NULL
        COMMENT 'Cours par unité de devise, NULL = ligne en devise principale',
    ADD COLUMN rate_source VARCHAR(12) NULL
        COMMENT 'Provenance du cours (afc, manual, revaluation), NULL = ligne en devise principale',
    ADD CONSTRAINT chk_jel_fx_all_or_none
        CHECK ((amount_currency IS NULL AND exchange_rate IS NULL AND rate_source IS NULL)
            OR (amount_currency IS NOT NULL AND exchange_rate IS NOT NULL AND rate_source IS NOT NULL)),
    ADD CONSTRAINT chk_jel_fx_rate_source
        CHECK (rate_source IS NULL OR BINARY rate_source IN ('afc', 'manual', 'revaluation')),
    ADD CONSTRAINT chk_jel_fx_amount
        CHECK (amount_currency IS NULL
            OR (BINARY rate_source = 'revaluation' AND amount_currency = 0)
            OR (rate_source IS NOT NULL AND BINARY rate_source <> 'revaluation' AND amount_currency > 0)),
    ADD CONSTRAINT chk_jel_fx_rate
        CHECK (exchange_rate IS NULL OR exchange_rate > 0),
    ADD CONSTRAINT chk_jel_fx_chf
        CHECK (amount_currency IS NULL OR BINARY rate_source = 'revaluation'
            OR debit + credit = ROUND(amount_currency * exchange_rate, 2));
