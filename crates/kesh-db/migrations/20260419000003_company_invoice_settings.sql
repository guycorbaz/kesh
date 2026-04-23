-- Story 2-6 & 5.2 — Company invoice settings table creation
-- Stores per-company invoice configuration (number format, default accounts, journal).
-- Relation 1-1 with companies table. Created for Story 2-6 onboarding pre-fill feature.

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
