-- NON BREAKING au sens du paragraphe ⚠️ de P1 (CLAUDE.md, Migration breaking
-- policy) : « sans relever `kesh_version_min_required` et sans corrompre le
-- schéma ». `kesh_version_min_required` reste à 0.17.6 ; aucune version Cargo
-- n'est relevée pour elle (P2/P2-bis).
--
-- Story 38-11b (Epic 38, phase 1) — un compte bancaire, solde d'un compte
-- PayPal dans une devise, et l'import du CSV PayPal. DDL seul : aucune ligne
-- écrite, ni registre de rejeu ni exemption (P7). Ligne ajoutée à
-- docs/migrations-idempotence-audit.md (P5).
--
-- 1. bank_accounts : réglages FACULTATIFS d'un prestataire — `provider`
--    (paypal ; wise et stripe réservés au schéma pour les Epics 39 et 40, qu'aucun
--    chemin de l'API n'écrit en phase 1), `provider_account` (le libellé du
--    compte du prestataire, « PayPal www ») et `fee_account_id` (compte de
--    charge des frais, clé étrangère RESTRICT). La CHECK
--    chk_bank_accounts_provider : tout NULL, ou un prestataire du domaine fermé
--    À L'OCTET (BINARY, comme 20261110000001 : la collation utf8mb4_unicode_ci
--    admettrait sinon 'PayPal'), sur un compte de type bank, jamais principal
--    (le principal porte la QR-facture, qui lit son IBAN), au libellé non vide.
--    ⚠️ Les `IS NOT NULL` explicites ne sont pas redondants : un CHECK qui vaut
--    NULL est SATISFAIT. Sans eux, `provider` NULL avec un libellé, ou un
--    prestataire sans libellé, rendaient NULL au second membre (BINARY NULL,
--    TRIM(NULL)) et passaient (constaté au test de la story).
--    La contrainte d'IBAN chk_bank_accounts_bank_has_iban (20261110000001) est
--    RETIRÉE PUIS REPOSÉE SOUS LE MÊME NOM, élargie au compte d'un prestataire :
--    inchangée pour tout compte sans prestataire. DROP CONSTRAINT n'est pas dans
--    la liste de P3.
--    ⚠️ Forme prouvée sur MariaDB 10.11 (Story 38-11b, T0) : dans UN SEUL
--    ALTER TABLE, `DROP CONSTRAINT IF EXISTS x, ADD CONSTRAINT IF NOT EXISTS x`
--    RETIRE la contrainte sans la reposer (le IF NOT EXISTS est jugé sur l'état
--    d'avant, où elle existe). La pose se fait donc SANS IF NOT EXISTS après le
--    DROP IF EXISTS : rejouée, la paire retire et repose, la contrainte reste.
-- 2. bank_transactions : `provider_type` (le Type PayPal) et
--    `counterparty_email`, NULL pour tout import bancaire ; index
--    (bank_account_id, transaction_id) pour la recherche de doublon de l'import
--    PayPal.
-- 3. reconciliation_rules : trois filtres FACULTATIFS (servis par 38-11c1),
--    combinés au critère existant (match_type, match_value inchangés et
--    obligatoires). Pas de valeur neuve à l'ENUM match_type (ce serait un
--    MODIFY COLUMN, P3). L'index unique uq_reconciliation_rules_match_active
--    n'est pas touché (limite écrite de la story).
--
-- Ce que la 0.19.x voit : elle refuse de démarrer sur une base migrée
-- (« was previously applied but is missing ») et refuse toute sauvegarde qui
-- porte les colonnes neuves ; aucun chemin ne lui fait donc lire un compte sans
-- IBAN de type bank, que sa validation ne connaît pas.
--
-- Idempotence (P5) : formes IF [NOT] EXISTS partout, sauf la repose de
-- chk_bank_accounts_bank_has_iban (ci-dessus), précédée de son retrait : une
-- re-exécution hors sqlx est sans effet (verdict `yes`). DDL non
-- transactionnel : un échec entre deux ALTER laisse une ligne `success = 0` ;
-- reprise : `DELETE FROM _sqlx_migrations WHERE version = 20261120000001 AND
-- success = 0;` puis redémarrer, le rejeu termine.
--
-- LE SQL QUI DÉFAIT CETTE MIGRATION (P1), sur une base où aucun compte ne porte
-- de prestataire (SELECT COUNT(*) FROM bank_accounts WHERE provider IS NOT NULL;
-- doit rendre 0) :
--   ALTER TABLE reconciliation_rules DROP CONSTRAINT chk_reconciliation_rules_filters, DROP COLUMN provider_type, DROP COLUMN counterparty_email, DROP COLUMN direction;
--   ALTER TABLE bank_transactions ADD INDEX fk_bank_transactions_bank_account (bank_account_id), DROP INDEX idx_bank_transactions_account_txid, DROP COLUMN provider_type, DROP COLUMN counterparty_email;
--   (l'index neuf, de tête bank_account_id, porte la clé étrangère du compte : MariaDB a retiré
--   l'index implicite fk_bank_transactions_bank_account ; il se repose avant le retrait, erreur 1553 sinon)
--   ALTER TABLE bank_accounts DROP CONSTRAINT chk_bank_accounts_provider, DROP FOREIGN KEY fk_bank_accounts_fee_account, DROP CONSTRAINT chk_bank_accounts_bank_has_iban, ADD CONSTRAINT chk_bank_accounts_bank_has_iban CHECK (account_kind <> 'bank' OR iban IS NOT NULL);
--   ALTER TABLE bank_accounts DROP COLUMN provider, DROP COLUMN provider_account, DROP COLUMN fee_account_id;
--   DELETE FROM _sqlx_migrations WHERE version = 20261120000001;
-- Le retour ordinaire reste la sauvegarde d'avant la mise à jour.

ALTER TABLE bank_accounts
    ADD COLUMN IF NOT EXISTS provider VARCHAR(16) NULL
        COMMENT 'Prestataire dont ce compte est le solde dans une devise : paypal (wise, stripe réservés) ; NULL = compte bancaire ordinaire',
    ADD COLUMN IF NOT EXISTS provider_account VARCHAR(120) NULL
        COMMENT 'Libellé du compte du prestataire (p. ex. « PayPal www ») ; un compte bancaire par devise et par libellé',
    ADD COLUMN IF NOT EXISTS fee_account_id BIGINT NULL
        COMMENT 'Compte de charge des frais du prestataire ; NULL = frais à examiner',
    ADD CONSTRAINT fk_bank_accounts_fee_account
        FOREIGN KEY IF NOT EXISTS (fee_account_id) REFERENCES accounts(id) ON DELETE RESTRICT,
    ADD CONSTRAINT IF NOT EXISTS chk_bank_accounts_provider
        CHECK ((provider IS NULL AND provider_account IS NULL AND fee_account_id IS NULL)
            OR (provider IS NOT NULL
                AND provider_account IS NOT NULL
                AND BINARY provider IN ('paypal', 'wise', 'stripe')
                AND account_kind = 'bank'
                AND is_primary = FALSE
                AND CHAR_LENGTH(TRIM(provider_account)) > 0)),
    DROP CONSTRAINT IF EXISTS chk_bank_accounts_bank_has_iban,
    ADD CONSTRAINT chk_bank_accounts_bank_has_iban
        CHECK (account_kind <> 'bank' OR iban IS NOT NULL OR provider IS NOT NULL);

ALTER TABLE bank_transactions
    ADD COLUMN IF NOT EXISTS provider_type VARCHAR(64) NULL
        COMMENT 'Type de la transaction chez le prestataire (Type PayPal) ; NULL pour un import bancaire',
    ADD COLUMN IF NOT EXISTS counterparty_email VARCHAR(254) NULL
        COMMENT 'Adresse électronique de la contrepartie (import d''un prestataire)',
    ADD INDEX IF NOT EXISTS idx_bank_transactions_account_txid (bank_account_id, transaction_id);

ALTER TABLE reconciliation_rules
    ADD COLUMN IF NOT EXISTS provider_type VARCHAR(64) NULL
        COMMENT 'Filtre facultatif : type de la transaction chez le prestataire',
    ADD COLUMN IF NOT EXISTS counterparty_email VARCHAR(254) NULL
        COMMENT 'Filtre facultatif : adresse électronique de la contrepartie',
    ADD COLUMN IF NOT EXISTS direction VARCHAR(8) NULL
        COMMENT 'Filtre facultatif : sens de la transaction, credit ou debit',
    ADD CONSTRAINT IF NOT EXISTS chk_reconciliation_rules_filters
        CHECK ((provider_type IS NULL OR CHAR_LENGTH(TRIM(provider_type)) > 0)
            AND (counterparty_email IS NULL OR CHAR_LENGTH(TRIM(counterparty_email)) > 0)
            AND (direction IS NULL OR BINARY direction IN ('credit', 'debit')));
