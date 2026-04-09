-- Story 3.1 : Plan comptable — table accounts
-- Stocke les comptes du plan comptable par company.
-- Hiérarchie via parent_id (auto-référentiel).
-- Archivage soft via le champ active.

CREATE TABLE accounts (
    id BIGINT NOT NULL AUTO_INCREMENT PRIMARY KEY,
    company_id BIGINT NOT NULL,
    number VARCHAR(10) NOT NULL,
    name VARCHAR(255) NOT NULL,
    account_type VARCHAR(20) NOT NULL COMMENT 'Asset|Liability|Revenue|Expense',
    parent_id BIGINT NULL,
    active BOOLEAN NOT NULL DEFAULT TRUE,
    version INT NOT NULL DEFAULT 1,
    created_at DATETIME(3) NOT NULL DEFAULT CURRENT_TIMESTAMP(3),
    updated_at DATETIME(3) NOT NULL DEFAULT CURRENT_TIMESTAMP(3) ON UPDATE CURRENT_TIMESTAMP(3),
    CONSTRAINT fk_accounts_company FOREIGN KEY (company_id) REFERENCES companies(id) ON DELETE RESTRICT,
    CONSTRAINT fk_accounts_parent FOREIGN KEY (parent_id) REFERENCES accounts(id) ON DELETE RESTRICT,
    CONSTRAINT uq_accounts_company_number UNIQUE (company_id, number),
    CONSTRAINT chk_accounts_type CHECK (BINARY account_type IN (BINARY 'Asset', BINARY 'Liability', BINARY 'Revenue', BINARY 'Expense')),
    CONSTRAINT chk_accounts_number_nonempty CHECK (CHAR_LENGTH(TRIM(number)) > 0),
    CONSTRAINT chk_accounts_name_nonempty CHECK (CHAR_LENGTH(TRIM(name)) > 0)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;
