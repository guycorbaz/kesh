-- Story 6.2 — Multi-tenant scoping refactor
-- Add company_id to users table for multi-tenant isolation.
-- Migration handles both fresh test DBs (no companies yet) and production DBs.

-- Step 1: Add company_id column (nullable initially)
-- Will be populated when company is created/assigned
ALTER TABLE users ADD COLUMN company_id BIGINT NULL;

-- Step 2: Backfill company_id for existing users
-- Conditional: only if companies table is populated (handles production DBs)
-- Fresh test DBs (no companies) → UPDATE matches no rows, no-op, users.company_id remains NULL
-- Assumption: By migration time, either companies are seeded OR no users exist yet
UPDATE users SET company_id = (SELECT id FROM companies ORDER BY id LIMIT 1)
WHERE company_id IS NULL AND EXISTS (SELECT 1 FROM companies LIMIT 1);

-- Step 3: Add NOT NULL constraint to match Rust type (i64, non-nullable)
-- DIAGNOSTIC: if this step fails with "Invalid use of NULL value" (ERROR 1138, SQLSTATE 22004),
-- it means users exist with company_id IS NULL after Step 2 backfill. This indicates either:
--   (a) the companies table was empty when migration ran (Case: fresh DB with users but no companies),
--   (b) Step 2 UPDATE matched no rows due to a missing companies row.
-- Recovery: ensure companies are seeded before running this migration on an existing users table.
-- New users created post-migration always have company_id (enforced by bootstrap + Rust types).
ALTER TABLE users MODIFY COLUMN company_id BIGINT NOT NULL;

-- Step 4: Add index for multi-tenant queries
CREATE INDEX idx_users_company_id ON users(company_id);

-- Step 5: Add foreign key constraint for referential integrity
-- Protects against orphaned users if a company is deleted (CASCADE).
ALTER TABLE users ADD CONSTRAINT fk_users_company
  FOREIGN KEY (company_id) REFERENCES companies(id) ON DELETE CASCADE;
