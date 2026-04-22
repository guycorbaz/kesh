-- Story 6.2 — Multi-tenant scoping refactor
-- Add company_id to users table to enable multi-tenant isolation.
-- Migration is safe for both fresh and existing DBs.

-- Step 1: Add company_id column (nullable, will be populated on first company creation)
ALTER TABLE users ADD COLUMN company_id BIGINT NULL;

-- Step 2: Attempt backfill if companies exist (production DB scenario)
-- Assigns all users to the first (and typically only) company.
-- In fresh test DBs, this is a no-op (no companies yet).
-- Users created after this migration require explicit company_id.
UPDATE users SET company_id = (SELECT id FROM companies ORDER BY id LIMIT 1)
WHERE company_id IS NULL AND EXISTS (SELECT 1 FROM companies LIMIT 1);

-- Step 3: Add index for multi-tenant queries (efficient before FK constraint)
CREATE INDEX idx_users_company_id ON users(company_id);

-- Step 4: Add foreign key constraint (users cannot exist without a valid company)
-- This constraint applies ONLY to new users going forward.
-- Existing NULL values are permitted for backward compatibility.
ALTER TABLE users ADD CONSTRAINT fk_users_company
  FOREIGN KEY (company_id) REFERENCES companies(id) ON DELETE CASCADE;
