-- Story 2-6 & 5.2 — Company invoice settings table creation
-- Stores per-company invoice configuration (number format, default accounts, journal).
-- Relation 1-1 with companies table. Created for Story 2-6 onboarding pre-fill feature.
--
-- AC 3 DEFERRAL: Fallback UI for missing accounts (AC 3) is NOT IMPLEMENTED.
-- Current implementation assumes Swiss PME charts always contain account 1100 (receivables)
-- and 3000 (revenue). If non-standard charts are added in future, AC 3 fallback UI should
-- be implemented to handle missing accounts gracefully (showing warning instead of error).
-- See Story 2-6 spec AC 3 for details.
--
-- MIGRATION DEPENDENCY: This migration creates FKs referencing the accounts table.
-- The accounts table is created by 20260411000001_accounts.sql which runs first
-- (dates: 20260411 < 20260419). FK constraints will not fail due to missing table.

CREATE TABLE company_invoice_settings (
  company_id BIGINT NOT NULL,
  invoice_number_format VARCHAR(255) NOT NULL DEFAULT 'F-{YEAR}-{SEQ:04}',
  default_receivable_account_id BIGINT NULL,
  default_revenue_account_id BIGINT NULL,
  default_sales_journal VARCHAR(50) NOT NULL DEFAULT 'Ventes',
  journal_entry_description_template VARCHAR(255) NOT NULL DEFAULT '{YEAR}-{INVOICE_NUMBER}',
  version INT NOT NULL DEFAULT 1,
  created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP(),
  updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP() ON UPDATE CURRENT_TIMESTAMP(),

  PRIMARY KEY (company_id),

  CONSTRAINT fk_company_invoice_settings_company
    FOREIGN KEY (company_id) REFERENCES companies(id) ON DELETE CASCADE,

  CONSTRAINT fk_company_invoice_settings_receivable_account
    FOREIGN KEY (default_receivable_account_id) REFERENCES accounts(id) ON DELETE SET NULL,

  CONSTRAINT fk_company_invoice_settings_revenue_account
    FOREIGN KEY (default_revenue_account_id) REFERENCES accounts(id) ON DELETE SET NULL
);

CREATE INDEX idx_company_invoice_settings_created_at ON company_invoice_settings(created_at);
