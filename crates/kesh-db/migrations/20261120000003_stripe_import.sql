-- NON BREAKING au sens du paragraphe ⚠️ de P1 (CLAUDE.md, Migration breaking
-- policy) : « sans relever `kesh_version_min_required` et sans corrompre le
-- schéma ». `kesh_version_min_required` reste à 0.17.6 ; aucune version Cargo
-- n'est relevée pour elle (P2/P2-bis).
--
-- Story 40-7b (Epic 40, phase 1) — un compte bancaire, solde d'un compte
-- Stripe dans une devise, désigne en plus le compte bancaire où Stripe verse
-- (Wise, banque…). DDL seul : aucune ligne écrite, ni registre de rejeu ni
-- exemption (P7). Ligne ajoutée à docs/migrations-idempotence-audit.md (P5).
--
-- Une seule instruction, sur `bank_accounts` :
--   * `payout_bank_account_id`, NULL = versements « à examiner » (40-7c) ;
--   * sa clé étrangère vers `bank_accounts(id)`, RESTRICT : un compte désigné
--     comme destination de versement ne se supprime pas (précédent
--     `accounts.parent_id`, clé étrangère sur la même table, tolérée par la
--     restauration : vidage sous FOREIGN_KEY_CHECKS = 0) ;
--   * la CHECK `chk_bank_accounts_payout_stripe` : la colonne ne se renseigne
--     que sur un compte `stripe`, comparé À L'OCTET (BINARY, comme
--     `chk_bank_accounts_provider`).
--     ⚠️ Les `IS NOT NULL` explicites ne sont pas redondants : une CHECK qui
--     vaut NULL est SATISFAITE. `payout IS NULL OR BINARY provider = 'stripe'`
--     laisserait passer une destination sur un compte SANS prestataire
--     (`BINARY NULL = 'stripe'` vaut NULL) ; le second membre garde donc
--     `provider IS NOT NULL`.
-- `chk_bank_accounts_provider` n'est PAS touchée : la migration 20261120000001
-- admet déjà 'stripe' dans son domaine fermé.
--
-- Ce que la version précédente voit : elle refuse de démarrer sur une base
-- migrée (« was previously applied but is missing ») et refuse toute
-- sauvegarde qui porte la colonne neuve ; aucun chemin ne lui fait lire la
-- colonne.
--
-- Idempotence (P5) : formes IF NOT EXISTS partout ; une re-exécution hors sqlx
-- est sans effet (verdict `yes`). DDL non transactionnel : un échec au milieu
-- laisse une ligne `success = 0` ; reprise : `DELETE FROM _sqlx_migrations
-- WHERE version = 20261120000003 AND success = 0;` puis redémarrer.
--
-- LE SQL QUI DÉFAIT CETTE MIGRATION (P1), sur une base où aucun compte ne
-- porte de destination de versement
-- (SELECT COUNT(*) FROM bank_accounts WHERE payout_bank_account_id IS NOT NULL;
-- doit rendre 0) :
--   ALTER TABLE bank_accounts DROP CONSTRAINT chk_bank_accounts_payout_stripe, DROP FOREIGN KEY fk_bank_accounts_payout_bank_account;
--   ALTER TABLE bank_accounts DROP COLUMN payout_bank_account_id;
--   DELETE FROM _sqlx_migrations WHERE version = 20261120000003;
-- Le retour ordinaire reste la sauvegarde d'avant la mise à jour.

ALTER TABLE bank_accounts
    ADD COLUMN IF NOT EXISTS payout_bank_account_id BIGINT NULL
        COMMENT 'Compte Stripe : compte bancaire où Stripe verse (même devise) ; NULL = versements à examiner',
    ADD CONSTRAINT fk_bank_accounts_payout_bank_account
        FOREIGN KEY IF NOT EXISTS (payout_bank_account_id) REFERENCES bank_accounts(id) ON DELETE RESTRICT,
    ADD CONSTRAINT IF NOT EXISTS chk_bank_accounts_payout_stripe
        CHECK (payout_bank_account_id IS NULL
            OR (provider IS NOT NULL AND BINARY provider = 'stripe'));
