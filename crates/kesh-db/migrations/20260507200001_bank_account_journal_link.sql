-- Story 8-5a-zero — foundation `bank_account.journal_account_id`.
--
-- Lie chaque `bank_account` à un compte du plan comptable (typiquement
-- classe 1 — 1020 Caisse / 1030 Banque, ou classe 2 si découvert
-- chronique 2100). Permet à 8-5a-base (FR45 manual match) et 8-5a-bis
-- (FR48 split) de résoudre serveur-side le ledger account côté banque
-- sans body field `bankLedgerAccountId` (anti-pattern UX éliminé F2''
-- Pass 3 validate Opus 4.7).
--
-- Décision §schema-migration : NULLABLE initialement + pas de FK
-- DB-level. La cohérence `bank_account.company_id == account.company_id`
-- est appliquée handler-side (cf. route `PATCH /bank-accounts/{id}` qui
-- check `accounts::find_by_id_in_company`). FK MariaDB ne peut pas
-- garantir cross-tenant company sans CHECK trigger DELIMITER (non
-- supporté ici, cf. Story 6-2 migration). v0.2 si refactor schema-level.
--
-- ALGORITHM=INSTANT évite la copie complète de la table sur MariaDB
-- 10.3+ (instant ADD COLUMN nullable). LOCK=NONE garantit la concurrent
-- DML pendant la migration (pattern hérité 8-4 / 8-1b).
ALTER TABLE bank_accounts
    ADD COLUMN journal_account_id BIGINT NULL AFTER qr_iban,
    ALGORITHM=INSTANT, LOCK=NONE;

-- Index pour les jointures futures GET /proposals (8-5a-base) qui
-- chargent le compte comptable lié au bank_account pour résoudre
-- serveur-side le ledger account du flow manual/split.
CREATE INDEX IF NOT EXISTS idx_bank_accounts_journal_account
    ON bank_accounts (journal_account_id);
