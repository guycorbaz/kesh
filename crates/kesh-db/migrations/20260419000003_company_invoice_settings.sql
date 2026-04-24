-- Story 2-6 & 5.2 — Company invoice settings index creation
-- Note: Table company_invoice_settings is created in migration 20260417000001_invoice_validation.sql
-- This migration only adds the index for performance optimization.
--
-- AC 3 DEFERRAL: Fallback UI for missing accounts (AC 3) is NOT IMPLEMENTED.
-- Current implementation assumes Swiss PME charts always contain account 1100 (receivables)
-- and 3000 (revenue). If non-standard charts are added in future, AC 3 fallback UI should
-- be implemented to handle missing accounts gracefully (showing warning instead of error).
-- See Story 2-6 spec AC 3 for details.

CREATE INDEX idx_company_invoice_settings_created_at ON company_invoice_settings(created_at);
