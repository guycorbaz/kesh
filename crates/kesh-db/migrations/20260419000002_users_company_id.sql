-- Story 6.2 — Multi-tenant scoping refactor
-- Add company_id to users table for multi-tenant isolation.
-- Migration handles both fresh test DBs (no companies yet) and production DBs.

-- Step 1: Add company_id column (nullable)
-- Will be populated when company is created/assigned
ALTER TABLE users ADD COLUMN company_id BIGINT NULL;

-- Step 2: Attempt backfill for production DBs
-- In production: assigns existing users to first company if one exists
-- In fresh test DBs: no-op (will be NULL until company created)
UPDATE users SET company_id = (SELECT id FROM companies ORDER BY id LIMIT 1)
WHERE company_id IS NULL AND EXISTS (SELECT 1 FROM companies LIMIT 1);

-- Step 3: Add index for multi-tenant queries
CREATE INDEX idx_users_company_id ON users(company_id);

-- Step 4: Add foreign key constraint for referential integrity
-- This protects against orphaned users if a company is deleted.
-- NULL company_id values are allowed during transition period.
ALTER TABLE users ADD CONSTRAINT fk_users_company
  FOREIGN KEY (company_id) REFERENCES companies(id) ON DELETE CASCADE;
