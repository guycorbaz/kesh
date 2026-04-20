-- Story 6.2 — Multi-tenant scoping refactor
-- Add company_id to users table to enable proper multi-tenant isolation

-- Step 1: Add nullable company_id for backfill
ALTER TABLE users ADD COLUMN company_id BIGINT NULL;

-- Step 2: Backfill — assign all users to the first company (mono-tenant assumption)
-- (Guards for data integrity are applied at application level during testing)
UPDATE users SET company_id = (SELECT id FROM companies ORDER BY id LIMIT 1);

-- Step 3: Make company_id NOT NULL after successful backfill
-- Note: If backfill failed (e.g., no companies exist), this will fail naturally on NULL violation
ALTER TABLE users MODIFY COLUMN company_id BIGINT NOT NULL;

-- Step 4: Add foreign key constraint (ON DELETE CASCADE — deleting a company cascades to its users)
ALTER TABLE users ADD CONSTRAINT fk_users_company FOREIGN KEY (company_id) REFERENCES companies(id) ON DELETE CASCADE;

-- Step 5: Add index for company scoping (used in list_by_company, find_by_id_in_company queries)
CREATE INDEX idx_users_company_id ON users(company_id);
