-- Story 8-1b — Migration bank_imports + bank_transactions
--
-- Schéma multi-tenant (KF-002 Pattern 1) : `company_id` sur les deux
-- tables, scoping systématique côté repositories.
--
-- Décisions de conception (cf. spec d'origine 8-1 §T4) :
--   - amount DECIMAL(18,2) signé : positif = crédit titulaire, négatif = débit
--   - status VARCHAR + CHECK plutôt qu'ENUM (aligné journal_entries.kind)
--   - matched_entry_id posé maintenant pour éviter ALTER TABLE Story 8-4
--   - idx_bank_transactions_pending optimise la requête Story 8-4 sans index supplémentaire
--   - Pas de FULLTEXT (Story 8-4 jointe sur journal_entries.description qui a déjà FULLTEXT Story 7-4)
--
-- Validate Pass 1 F8 : valeurs `source_format` en MAJUSCULES (`'CAMT053_V04'`,
-- `'CAMT053_V08'`) — alignement avec `kesh_core::bank_imports::SourceFormatTag::as_db_str()`
-- livré 8-1a (commit 224209f / e1c3052 / 399f761 / 65e3f56).
-- Pas de CHECK constraint sur les valeurs : un futur `'CSV_UBS_PROFILE_2026'`
-- (Story 8-2) doit pouvoir être ajouté sans migration ALTER.

CREATE TABLE bank_imports (
    id BIGINT NOT NULL AUTO_INCREMENT,
    company_id BIGINT NOT NULL,
    bank_account_id BIGINT NOT NULL,
    filename VARCHAR(255) NOT NULL,
    file_hash CHAR(64) NOT NULL,                  -- SHA-256 hex (64 chars)
    source_format VARCHAR(32) NOT NULL,           -- 'CAMT053_V04', 'CAMT053_V08' (v0.1) ; CSV variants Story 8-2
    statement_id VARCHAR(255) NULL,               -- <Stmt><Id> CAMT, NULL pour CSV
    period_from DATE NOT NULL,
    period_to DATE NOT NULL,
    opening_balance DECIMAL(18,2) NULL,
    closing_balance DECIMAL(18,2) NULL,
    transaction_count INT NOT NULL DEFAULT 0,
    imported_at DATETIME(3) NOT NULL DEFAULT CURRENT_TIMESTAMP(3),
    imported_by_user_id BIGINT NOT NULL,
    PRIMARY KEY (id),
    CONSTRAINT fk_bank_imports_company
        FOREIGN KEY (company_id) REFERENCES companies(id) ON DELETE RESTRICT,
    CONSTRAINT fk_bank_imports_bank_account
        FOREIGN KEY (bank_account_id) REFERENCES bank_accounts(id) ON DELETE RESTRICT,
    CONSTRAINT fk_bank_imports_user
        FOREIGN KEY (imported_by_user_id) REFERENCES users(id) ON DELETE RESTRICT,
    CONSTRAINT uq_bank_imports_company_hash UNIQUE (company_id, file_hash),
    CONSTRAINT chk_bank_imports_period CHECK (period_to >= period_from),
    -- Defense-in-depth (Pass 1 review M5) :
    -- transaction_count borné >= 0 (l'app passe `i32::try_from(len) as i32`
    -- mais une régression passant un négatif est silencieusement acceptée
    -- sans cette CHECK).
    CONSTRAINT chk_bank_imports_tx_count CHECK (transaction_count >= 0),
    -- file_hash doit être un SHA-256 hex de 64 chars (le compute_sha256_hex
    -- handler le garantit, mais futur caller pourrait passer un MD5 32
    -- chars ; CHAR(64) padderait avec des espaces silencieusement).
    CONSTRAINT chk_bank_imports_hash_len CHECK (CHAR_LENGTH(file_hash) = 64),
    INDEX idx_bank_imports_company_account_imported (company_id, bank_account_id, imported_at)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

CREATE TABLE bank_transactions (
    id BIGINT NOT NULL AUTO_INCREMENT,
    company_id BIGINT NOT NULL,
    import_id BIGINT NOT NULL,
    bank_account_id BIGINT NOT NULL,
    booking_date DATE NOT NULL,
    value_date DATE NULL,
    amount DECIMAL(18,2) NOT NULL,
    currency CHAR(3) NOT NULL,
    reference VARCHAR(255) NULL,
    details TEXT NOT NULL,
    end_to_end_id VARCHAR(255) NULL,
    transaction_id VARCHAR(255) NULL,
    counterparty_iban VARCHAR(34) NULL,
    counterparty_name VARCHAR(255) NULL,
    status VARCHAR(16) NOT NULL DEFAULT 'pending',
    matched_entry_id BIGINT NULL,
    version INT NOT NULL DEFAULT 1,
    created_at DATETIME(3) NOT NULL DEFAULT CURRENT_TIMESTAMP(3),
    updated_at DATETIME(3) NOT NULL DEFAULT CURRENT_TIMESTAMP(3) ON UPDATE CURRENT_TIMESTAMP(3),
    PRIMARY KEY (id),
    CONSTRAINT fk_bank_transactions_company
        FOREIGN KEY (company_id) REFERENCES companies(id) ON DELETE RESTRICT,
    CONSTRAINT fk_bank_transactions_import
        FOREIGN KEY (import_id) REFERENCES bank_imports(id) ON DELETE CASCADE,
    CONSTRAINT fk_bank_transactions_bank_account
        FOREIGN KEY (bank_account_id) REFERENCES bank_accounts(id) ON DELETE RESTRICT,
    CONSTRAINT fk_bank_transactions_matched_entry
        FOREIGN KEY (matched_entry_id) REFERENCES journal_entries(id) ON DELETE SET NULL,
    CONSTRAINT chk_bank_transactions_status CHECK (status IN ('pending', 'reconciled')),
    CONSTRAINT chk_bank_transactions_currency_iso4217 CHECK (CHAR_LENGTH(currency) = 3),
    INDEX idx_bank_transactions_company_account_date (company_id, bank_account_id, booking_date),
    INDEX idx_bank_transactions_import (import_id),
    INDEX idx_bank_transactions_pending (company_id, bank_account_id, status, booking_date)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;
