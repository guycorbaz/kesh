-- Story 5.2 : Validation & numérotation des factures (FR33, FR34, FR35).
--
-- Ajouts :
-- 1. Table `invoice_number_sequences` : compteur séquentiel par
--    (company_id, fiscal_year_id) pour la numérotation « sans trou » à la
--    validation. Incrémenté atomiquement via SELECT FOR UPDATE dans la
--    transaction de validation. Rollback = compteur intact.
-- 2. Table `company_invoice_settings` : config facturation par company
--    (format numéro, comptes par défaut, journal, template libellé écriture).
--    PK = company_id (relation 1-1). Lazy insert via INSERT IGNORE.
-- 3. ALTER invoices : ajout colonne `journal_entry_id` (FK vers écriture
--    comptable générée à la validation) + contrainte UNIQUE(company_id,
--    invoice_number) — MariaDB autorise plusieurs NULL dans un UNIQUE.
--
-- Pas de nouvelle colonne `value_date` : `due_date` (Story 5.1) couvre
-- déjà le besoin. Le backend défaut `due_date = date` si non fourni.
--
-- CHECK BINARY syntax alignée avec `chk_journal_entries_journal`
-- (migration 20260412000001) — pattern validé en prod MariaDB 11.x.

CREATE TABLE invoice_number_sequences (
    id BIGINT NOT NULL AUTO_INCREMENT PRIMARY KEY,
    company_id BIGINT NOT NULL,
    fiscal_year_id BIGINT NOT NULL,
    next_number BIGINT NOT NULL DEFAULT 1,
    version INT NOT NULL DEFAULT 1,
    created_at DATETIME(3) NOT NULL DEFAULT CURRENT_TIMESTAMP(3),
    updated_at DATETIME(3) NOT NULL DEFAULT CURRENT_TIMESTAMP(3) ON UPDATE CURRENT_TIMESTAMP(3),
    CONSTRAINT fk_ins_company FOREIGN KEY (company_id) REFERENCES companies(id) ON DELETE RESTRICT,
    CONSTRAINT fk_ins_fiscal_year FOREIGN KEY (fiscal_year_id) REFERENCES fiscal_years(id) ON DELETE RESTRICT,
    CONSTRAINT uq_ins_company_fy UNIQUE (company_id, fiscal_year_id),
    CONSTRAINT chk_ins_next_positive CHECK (next_number >= 1)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

CREATE TABLE company_invoice_settings (
    company_id BIGINT NOT NULL PRIMARY KEY,
    invoice_number_format VARCHAR(64) NOT NULL DEFAULT 'F-{YEAR}-{SEQ:04}',
    default_receivable_account_id BIGINT NULL,
    default_revenue_account_id BIGINT NULL,
    default_sales_journal VARCHAR(10) NOT NULL DEFAULT 'Ventes',
    journal_entry_description_template VARCHAR(128) NOT NULL DEFAULT '{YEAR}-{INVOICE_NUMBER}',
    version INT NOT NULL DEFAULT 1,
    created_at DATETIME(3) NOT NULL DEFAULT CURRENT_TIMESTAMP(3),
    updated_at DATETIME(3) NOT NULL DEFAULT CURRENT_TIMESTAMP(3) ON UPDATE CURRENT_TIMESTAMP(3),
    CONSTRAINT fk_cis_company FOREIGN KEY (company_id) REFERENCES companies(id) ON DELETE RESTRICT,
    CONSTRAINT fk_cis_receivable FOREIGN KEY (default_receivable_account_id) REFERENCES accounts(id) ON DELETE RESTRICT,
    CONSTRAINT fk_cis_revenue FOREIGN KEY (default_revenue_account_id) REFERENCES accounts(id) ON DELETE RESTRICT,
    CONSTRAINT chk_cis_journal CHECK (BINARY default_sales_journal IN (BINARY 'Achats', BINARY 'Ventes', BINARY 'Banque', BINARY 'Caisse', BINARY 'OD')),
    CONSTRAINT chk_cis_format_nonempty CHECK (CHAR_LENGTH(TRIM(invoice_number_format)) > 0),
    CONSTRAINT chk_cis_je_template_nonempty CHECK (CHAR_LENGTH(TRIM(journal_entry_description_template)) > 0)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

ALTER TABLE invoices
    ADD COLUMN journal_entry_id BIGINT NULL AFTER total_amount,
    ADD CONSTRAINT fk_invoices_journal_entry
        FOREIGN KEY (journal_entry_id) REFERENCES journal_entries(id) ON DELETE RESTRICT,
    ADD CONSTRAINT uq_invoices_number UNIQUE (company_id, invoice_number);
