-- Story 8-5b — Moteur de règles de réconciliation (FR47).
--
-- Table `reconciliation_rules` qui stocke les règles d'affectation
-- automatique d'une transaction bancaire vers un compte de contrepartie
-- (Charges/Produits typiquement) sans passer par une facture. Utilisée
-- au flow `GET /api/v1/reconciliation/proposals` pour proposer un
-- compte de contrepartie quand aucune facture ne match, puis au flow
-- `POST /api/v1/reconciliation/accept type=rule` pour créer le journal
-- entry via le helper `manual::build_journal_entry_for_counterparty`
-- (réutilisé de 8-5a-base).
--
-- §rules-schema décision Q3 Guy 2026-05-07 : soft-delete via
-- `active=false` au lieu de DELETE physique, pour préserver l'audit
-- historique des `reconciliation_rule.applied` qui référencent
-- l'ancien `rule_id`. La contrainte UNIQUE
-- `uq_reconciliation_rules_match_active` doit donc être **partielle**
-- (effective seulement sur les rules actives) pour permettre la
-- réactivation après archivage et tolérer plusieurs rules archivées
-- avec mêmes `(company_id, match_type, match_value)`.
--
-- MariaDB n'a pas de UNIQUE partiel natif (pas de syntaxe
-- `WHERE active = TRUE`). Workaround Option A : colonne synthétique
-- `active_uniq` (GENERATED ALWAYS AS VIRTUAL) qui vaut `match_value`
-- si `active=true` et NULL sinon. Le UNIQUE sur `(company_id,
-- match_type, active_uniq)` exploite la convention SQL « NULL n'est
-- jamais égal à NULL » → les rules archivées (active_uniq=NULL) ne
-- participent pas au UNIQUE. Pre-requis : MariaDB ≥ 10.6 pour UNIQUE
-- sur VIRTUAL column (Docker Compose pin `mariadb:10.11`, OK — requis MariaDB ≥ 10.6).
--
-- Cohérence cross-tenant : pas de CHECK trigger DELIMITER pour
-- garantir `rule.company_id == counterparty_account.company_id`
-- (pattern Story 6-2 / 8-5a-zero — application handler-side). Le
-- handler `post_create` valide `accounts::find_by_id_in_company`
-- AVANT INSERT pour rejeter cross-tenant en 404 `ACCOUNT_NOT_FOUND`.

CREATE TABLE reconciliation_rules (
    id BIGINT NOT NULL AUTO_INCREMENT,
    company_id BIGINT NOT NULL,
    label VARCHAR(120) NOT NULL,
    match_type ENUM('counterparty_contains','counterparty_exact','reference_contains','iban_exact') NOT NULL,
    match_value VARCHAR(255) NOT NULL,
    counterparty_account_id BIGINT NOT NULL,
    priority INT NOT NULL DEFAULT 100,
    active BOOLEAN NOT NULL DEFAULT TRUE,
    applied_count BIGINT NOT NULL DEFAULT 0,
    last_applied_at DATETIME(3) NULL,
    version INT NOT NULL DEFAULT 1,
    created_at DATETIME(3) NOT NULL DEFAULT CURRENT_TIMESTAMP(3),
    updated_at DATETIME(3) NOT NULL DEFAULT CURRENT_TIMESTAMP(3) ON UPDATE CURRENT_TIMESTAMP(3),
    -- Q3 workaround : `active_uniq` vaut `match_value` quand `active=true`,
    -- NULL sinon. Le UNIQUE `uq_reconciliation_rules_match_active`
    -- exploite que NULL ≠ NULL en SQL standard pour réaliser un
    -- UNIQUE partiel effectif uniquement sur les rules actives.
    active_uniq VARCHAR(255) GENERATED ALWAYS AS (IF(active, match_value, NULL)) VIRTUAL,
    PRIMARY KEY (id),
    CONSTRAINT fk_reconciliation_rules_company
        FOREIGN KEY (company_id) REFERENCES companies(id) ON DELETE RESTRICT,
    CONSTRAINT fk_reconciliation_rules_account
        FOREIGN KEY (counterparty_account_id) REFERENCES accounts(id) ON DELETE RESTRICT,
    CONSTRAINT chk_reconciliation_rules_label_non_empty
        CHECK (CHAR_LENGTH(TRIM(label)) > 0),
    CONSTRAINT chk_reconciliation_rules_match_value_non_empty
        CHECK (CHAR_LENGTH(TRIM(match_value)) > 0),
    CONSTRAINT chk_reconciliation_rules_priority_range
        CHECK (priority BETWEEN 1 AND 1000),
    CONSTRAINT chk_reconciliation_rules_applied_count_positive
        CHECK (applied_count >= 0),
    CONSTRAINT uq_reconciliation_rules_match_active
        UNIQUE (company_id, match_type, active_uniq),
    INDEX idx_reconciliation_rules_company_active_priority (company_id, active, priority, id)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;
