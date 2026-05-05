-- Story 8-2 — Migration bank_profiles
--
-- Profils de format CSV par banque (FR53 PRD). Permet de configurer
-- le mapping colonnes + format date + encoding + séparateurs pour les
-- relevés CSV qui ne suivent pas un format standard.
--
-- Schéma multi-tenant (KF-002 Pattern 1) : `company_id` scope strict.
-- UNIQUE (company_id, bank_name) : un seul profil par banque par tenant
-- (mais "UBS" peut exister dans company_A ET company_B).
--
-- column_mapping JSON : indices 0-based vers les colonnes du CSV.
-- Schéma JSON validé côté Rust via serde_json::from_str + struct
-- ColumnMapping (T2.2). Le CHECK MariaDB JSON_VALID garantit la
-- syntaxe JSON mais pas la sémantique (XOR amount/debit_credit_split,
-- collision indices, etc. validés handler API).
--
-- Pas de FK `bank_imports.bank_profile_id` v0.1 (cf. Limitations L4
-- spec 8-2). Le `bank_profile_id` est tracé uniquement dans
-- `audit_log.details_json` avec snapshot `bank_profile_name` pour
-- préserver la trace humaine post-delete profil.

CREATE TABLE bank_profiles (
    id BIGINT NOT NULL AUTO_INCREMENT,
    company_id BIGINT NOT NULL,
    bank_name VARCHAR(100) NOT NULL,
    filename_pattern VARCHAR(200) NULL,           -- regex compilable, NULL = pas d'auto-match
    column_mapping JSON NOT NULL,                  -- struct ColumnMapping (date, amount?, debit_credit_split?, reference?, details?, counterparty?)
    date_format VARCHAR(50) NOT NULL,              -- chrono format ex. "%d.%m.%Y"
    decimal_separator CHAR(1) NOT NULL,
    field_separator CHAR(1) NOT NULL,
    encoding VARCHAR(20) NULL,                     -- NULL = auto-détect, sinon "UTF-8" | "ISO-8859-1"
    header_row_count TINYINT UNSIGNED NOT NULL DEFAULT 1,
    created_at DATETIME(6) NOT NULL DEFAULT CURRENT_TIMESTAMP(6),
    updated_at DATETIME(6) NOT NULL DEFAULT CURRENT_TIMESTAMP(6) ON UPDATE CURRENT_TIMESTAMP(6),
    PRIMARY KEY (id),
    CONSTRAINT fk_bank_profiles_company
        FOREIGN KEY (company_id) REFERENCES companies(id) ON DELETE CASCADE,
    CONSTRAINT chk_bank_profiles_field_separator
        CHECK (field_separator IN (',', ';', '\t')),
    CONSTRAINT chk_bank_profiles_decimal_separator
        CHECK (decimal_separator IN ('.', ',')),
    CONSTRAINT chk_bank_profiles_separators_distinct
        CHECK (field_separator <> decimal_separator),
    CONSTRAINT chk_bank_profiles_header_row_count
        CHECK (header_row_count <= 5),
    CONSTRAINT chk_bank_profiles_bank_name_len
        CHECK (CHAR_LENGTH(bank_name) BETWEEN 1 AND 100),
    CONSTRAINT chk_bank_profiles_column_mapping_valid
        CHECK (JSON_VALID(column_mapping)),
    CONSTRAINT chk_bank_profiles_date_format_len
        CHECK (CHAR_LENGTH(date_format) BETWEEN 1 AND 50),
    CONSTRAINT chk_bank_profiles_filename_pattern_len
        CHECK (filename_pattern IS NULL OR CHAR_LENGTH(filename_pattern) BETWEEN 1 AND 200),
    CONSTRAINT uq_bank_profiles_company_name
        UNIQUE (company_id, bank_name),
    INDEX idx_bank_profiles_company (company_id),
    INDEX idx_bank_profiles_company_pattern (company_id, filename_pattern)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;
