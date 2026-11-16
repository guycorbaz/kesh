-- Non breaking (P1, P3) : ADD COLUMN avec défaut, ADD CONSTRAINT, ADD INDEX
-- seulement. Un binaire 0.17.6 ignore les colonnes neuves (aucun SELECT * ne
-- les décode) ; kesh_version_min_required reste à 0.17.6.
--
-- Story 37-1 (Epic 37, multi-devises) — réglage multi-devises de la société,
-- mode et genres de cours de l'exercice, devise et comptes de différence de
-- change propres d'un compte.
--
-- 1. companies : le réglage (multi_currency_enabled) et les trois comptes de la
--    société — gains et pertes de change, compte de conversion (bilan, CHF).
--    SANS CLÉ ÉTRANGÈRE vers accounts, délibérément : accounts référence déjà
--    companies (fk_accounts_company), une FK inverse formerait un cycle
--    companies <-> accounts ; la remise à zéro de la société vide accounts en
--    gardant companies, et l'import d'une sauvegarde réinsère les deux tables
--    sans ordre imposé. L'intégrité (compte de la société, imputable, actif,
--    sans devise, du bon type) est tenue par le dépôt, sous verrou, et
--    revalidée à l'usage.
-- 2. fiscal_years : le mode de l'exercice (mono-devise ou multi-devises), posé
--    à sa création et à l'activation, jamais changé ensuite ; les genres des
--    cours d'ouverture et de clôture (day = cours journalier, month = cours
--    mensuel). Les exercices existants prennent FALSE, 'day', 'day' : une base
--    0.17.6 est mono-devise.
-- 3. accounts.currency : NULL = la devise principale (CHF). Collation binaire
--    ASCII ET BINARY dans les CHECK : la table est en utf8mb4_unicode_ci, sous
--    laquelle REGEXP ignorerait la casse et 'usd' passerait. 'CHF' est refusé :
--    la devise principale n'a qu'une graphie, NULL.
-- 4. accounts.fx_gain_account_id / fx_loss_account_id : comptes de différence
--    de change propres à un compte en devise ; NULL = suivre le réglage de la
--    société, lu à l'usage. Sans clé étrangère, même raison qu'en 1 (relation
--    logique vers accounts.id de la même société, intégrité par le dépôt ; la
--    remise à zéro vide accounts entière, l'import s'exécute sans contrôle de
--    clé). Les deux ensemble ou aucun ; posables sur un compte en devise
--    seulement. Deux index : la recherche du compte qui désigne un compte donné
--    ne balaie pas toutes les lignes accounts de la société.
--
-- Aucune écriture de données (P7 : rien à trier). Ligne ajoutée à
-- docs/migrations-idempotence-audit.md (P5).
ALTER TABLE companies
    ADD COLUMN multi_currency_enabled BOOLEAN NOT NULL DEFAULT FALSE
        COMMENT 'Mode multi-devises de la société, irréversible une fois activé',
    ADD COLUMN fx_gain_account_id BIGINT NULL
        COMMENT 'Compte des gains de change (accounts.id, sans FK : intégrité par le dépôt)',
    ADD COLUMN fx_loss_account_id BIGINT NULL
        COMMENT 'Compte des pertes de change (accounts.id, sans FK : intégrité par le dépôt)',
    ADD COLUMN fx_transit_account_id BIGINT NULL
        COMMENT 'Compte de conversion entre devises, de bilan, en CHF (accounts.id, sans FK)';

ALTER TABLE fiscal_years
    ADD COLUMN multi_currency BOOLEAN NOT NULL DEFAULT FALSE
        COMMENT 'Exercice multi-devises, posé à la création et à l''activation, jamais changé ensuite',
    ADD COLUMN opening_rate_kind ENUM('day', 'month') NOT NULL DEFAULT 'day'
        COMMENT 'Genre du cours d''ouverture : day = veille de start_date, month = mois de la veille',
    ADD COLUMN closing_rate_kind ENUM('day', 'month') NOT NULL DEFAULT 'day'
        COMMENT 'Genre du cours de clôture : day = end_date, month = mois de end_date';

ALTER TABLE accounts
    ADD COLUMN currency CHAR(3) CHARACTER SET ascii COLLATE ascii_bin NULL
        COMMENT 'Code ISO 4217 de la devise du compte, NULL = devise principale (CHF)',
    ADD COLUMN fx_gain_account_id BIGINT NULL
        COMMENT 'Compte de gains de change propre (accounts.id, sans FK), NULL = réglage de la société',
    ADD COLUMN fx_loss_account_id BIGINT NULL
        COMMENT 'Compte de pertes de change propre (accounts.id, sans FK), NULL = réglage de la société',
    ADD CONSTRAINT chk_accounts_currency_format
        CHECK (currency IS NULL OR BINARY currency REGEXP '^[A-Z]{3}$'),
    ADD CONSTRAINT chk_accounts_currency_not_base
        CHECK (currency IS NULL OR BINARY currency <> BINARY 'CHF'),
    ADD CONSTRAINT chk_accounts_fx_own_pair
        CHECK ((fx_gain_account_id IS NULL) = (fx_loss_account_id IS NULL)),
    ADD CONSTRAINT chk_accounts_fx_own_foreign
        CHECK (fx_gain_account_id IS NULL OR currency IS NOT NULL),
    ADD INDEX idx_accounts_fx_gain (company_id, fx_gain_account_id),
    ADD INDEX idx_accounts_fx_loss (company_id, fx_loss_account_id);
