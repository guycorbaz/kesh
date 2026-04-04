-- Migration initiale : schéma de base pour companies, users, fiscal_years
-- Story 1.4 — Schéma de base & repository pattern

CREATE TABLE companies (
    id BIGINT NOT NULL AUTO_INCREMENT PRIMARY KEY,
    name VARCHAR(255) NOT NULL,
    address TEXT NOT NULL,
    ide_number VARCHAR(15) NULL COMMENT 'Format: CHExxxxxxxxx (normalisé, sans séparateurs)',
    org_type VARCHAR(20) NOT NULL COMMENT 'Independant|Association|Pme (ASCII par design, pas d''accent)',
    accounting_language CHAR(2) NOT NULL COMMENT 'FR|DE|IT|EN — langue des libellés comptables',
    instance_language CHAR(2) NOT NULL COMMENT 'FR|DE|IT|EN — langue de l''interface',
    version INT NOT NULL DEFAULT 1,
    created_at DATETIME(3) NOT NULL DEFAULT CURRENT_TIMESTAMP(3),
    updated_at DATETIME(3) NOT NULL DEFAULT CURRENT_TIMESTAMP(3) ON UPDATE CURRENT_TIMESTAMP(3),
    CONSTRAINT uq_companies_ide_number UNIQUE (ide_number),
    CONSTRAINT chk_companies_org_type CHECK (BINARY org_type IN (BINARY 'Independant', BINARY 'Association', BINARY 'Pme')),
    CONSTRAINT chk_companies_accounting_language CHECK (BINARY accounting_language IN (BINARY 'FR', BINARY 'DE', BINARY 'IT', BINARY 'EN')),
    CONSTRAINT chk_companies_instance_language CHECK (BINARY instance_language IN (BINARY 'FR', BINARY 'DE', BINARY 'IT', BINARY 'EN')),
    CONSTRAINT chk_companies_name_nonempty CHECK (CHAR_LENGTH(TRIM(name)) > 0),
    CONSTRAINT chk_companies_address_nonempty CHECK (CHAR_LENGTH(TRIM(address)) > 0),
    CONSTRAINT chk_companies_ide_format CHECK (ide_number IS NULL OR ide_number REGEXP '^CHE[0-9]{9}$')
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

CREATE TABLE users (
    id BIGINT NOT NULL AUTO_INCREMENT PRIMARY KEY,
    username VARCHAR(64) NOT NULL,
    password_hash VARCHAR(512) NOT NULL COMMENT 'Argon2id — format PHC string (jusqu''à 512 chars pour supporter les paramètres custom)',
    role VARCHAR(20) NOT NULL COMMENT 'Admin|Comptable|Consultation',
    active BOOLEAN NOT NULL DEFAULT TRUE,
    version INT NOT NULL DEFAULT 1,
    created_at DATETIME(3) NOT NULL DEFAULT CURRENT_TIMESTAMP(3),
    updated_at DATETIME(3) NOT NULL DEFAULT CURRENT_TIMESTAMP(3) ON UPDATE CURRENT_TIMESTAMP(3),
    CONSTRAINT uq_users_username UNIQUE (username),
    CONSTRAINT chk_users_role CHECK (BINARY role IN (BINARY 'Admin', BINARY 'Comptable', BINARY 'Consultation')),
    CONSTRAINT chk_users_username_nonempty CHECK (CHAR_LENGTH(TRIM(username)) > 0),
    CONSTRAINT chk_users_password_hash_len CHECK (OCTET_LENGTH(password_hash) >= 20)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

CREATE TABLE fiscal_years (
    id BIGINT NOT NULL AUTO_INCREMENT PRIMARY KEY,
    company_id BIGINT NOT NULL,
    name VARCHAR(50) NOT NULL COMMENT 'ex: "Exercice 2026"',
    start_date DATE NOT NULL,
    end_date DATE NOT NULL,
    status VARCHAR(10) NOT NULL DEFAULT 'Open' COMMENT 'Open|Closed',
    created_at DATETIME(3) NOT NULL DEFAULT CURRENT_TIMESTAMP(3),
    updated_at DATETIME(3) NOT NULL DEFAULT CURRENT_TIMESTAMP(3) ON UPDATE CURRENT_TIMESTAMP(3),
    CONSTRAINT fk_fiscal_years_company FOREIGN KEY (company_id) REFERENCES companies(id) ON DELETE RESTRICT,
    CONSTRAINT uq_fiscal_years_company_name UNIQUE (company_id, name),
    CONSTRAINT uq_fiscal_years_company_start_date UNIQUE (company_id, start_date),
    CONSTRAINT chk_fiscal_years_dates CHECK (end_date > start_date),
    CONSTRAINT chk_fiscal_years_status CHECK (BINARY status IN (BINARY 'Open', BINARY 'Closed')),
    CONSTRAINT chk_fiscal_years_name_nonempty CHECK (CHAR_LENGTH(TRIM(name)) > 0)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;
