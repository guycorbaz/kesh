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

-- Step 3: P1-H5 Validation — ensure backfill succeeded before adding NOT NULL constraint
-- Fail fast with descriptive error if users exist without company_id
-- This prevents confusing "constraint violation" error if backfill didn't run
SELECT CASE
  WHEN EXISTS (SELECT 1 FROM users WHERE company_id IS NULL) THEN
    SIGNAL SQLSTATE '45000'
    SET MESSAGE_TEXT = 'MIGRATION FAILED: Users exist without company_id. Backfill in Step 2 did not complete. Check companies table is populated before retrying migration.';
  ELSE 1
END;

-- Step 4: Add NOT NULL constraint to match Rust type (i64, non-nullable)
-- This enforces that every user has a company_id (no orphaned users).
-- PREREQUISITE: Backfill must have assigned company to all existing users, OR no users exist.
-- If this step fails, it means users exist without companies → data consistency issue.
-- New users will always be created with a company_id (enforced by bootstrap + Rust types).
-- Must add constraint BEFORE index to ensure semantic ordering (schema constraints before optimization).
ALTER TABLE users MODIFY COLUMN company_id BIGINT NOT NULL;

-- Step 4: Add index for multi-tenant queries
-- Logically placed after NOT NULL constraint (constraints define schema, then optimize with indices).
CREATE INDEX idx_users_company_id ON users(company_id);

-- Step 5: Add foreign key constraint for referential integrity
-- This protects against orphaned users if a company is deleted.
ALTER TABLE users ADD CONSTRAINT fk_users_company
  FOREIGN KEY (company_id) REFERENCES companies(id) ON DELETE CASCADE;
