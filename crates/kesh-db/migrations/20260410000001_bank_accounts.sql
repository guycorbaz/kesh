-- Migration : table bank_accounts pour la configuration des comptes bancaires
-- Story 2.3 — Flux d'onboarding Chemin B (Production)

CREATE TABLE bank_accounts (
    id BIGINT NOT NULL AUTO_INCREMENT PRIMARY KEY,
    company_id BIGINT NOT NULL,
    bank_name VARCHAR(255) NOT NULL,
    iban VARCHAR(34) NOT NULL COMMENT 'IBAN normalisé sans espaces',
    qr_iban VARCHAR(34) NULL COMMENT 'QR-IBAN optionnel (QR-IID 30000-31999)',
    is_primary BOOLEAN NOT NULL DEFAULT FALSE,
    version INT NOT NULL DEFAULT 1,
    created_at DATETIME(3) NOT NULL DEFAULT CURRENT_TIMESTAMP(3),
    updated_at DATETIME(3) NOT NULL DEFAULT CURRENT_TIMESTAMP(3) ON UPDATE CURRENT_TIMESTAMP(3),
    CONSTRAINT fk_bank_accounts_company FOREIGN KEY (company_id) REFERENCES companies(id) ON DELETE RESTRICT,
    CONSTRAINT chk_bank_accounts_bank_name_nonempty CHECK (CHAR_LENGTH(TRIM(bank_name)) > 0),
    CONSTRAINT chk_bank_accounts_iban_nonempty CHECK (CHAR_LENGTH(TRIM(iban)) > 0)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;
