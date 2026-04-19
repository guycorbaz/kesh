-- Story 6.2 — Multi-tenant scoping refactor
-- Add company_id to users table to enable proper multi-tenant isolation

-- Step 0: Lock tables to prevent race condition (concurrent INSERTs while migrating)
-- Must lock both users (WRITE) and companies (READ) since UPDATE uses companies in subquery
LOCK TABLES users WRITE, companies READ;

-- Step 0.5: Guard — check if users exist but no companies (data integrity check)
-- If this fails, migration will abort with a clear error before any changes
SET @user_count = (SELECT COUNT(*) FROM users);
SET @company_count = (SELECT COUNT(*) FROM companies);
IF @user_count > 0 AND @company_count = 0 THEN
  SIGNAL SQLSTATE '45000' SET MESSAGE_TEXT = 'Migration failed: users exist but no companies in DB. Backfill impossible. Create at least one company before migration.';
END IF;

-- Step 1: Add nullable company_id for backfill
ALTER TABLE users ADD COLUMN company_id BIGINT NULL;

-- Step 2: Backfill — assign all users to the first company (mono-tenant assumption)
UPDATE users SET company_id = (SELECT id FROM companies ORDER BY id LIMIT 1);

-- Step 3: Guard — this ALTER will fail naturally if any users are orphaned (company_id IS NULL)
-- Also fails if users exist but no companies (company_id would be NULL after UPDATE with empty subquery)
-- This replaces explicit IF checks which aren't supported in .sql migration files

-- Step 4: Make company_id NOT NULL after successful backfill
ALTER TABLE users MODIFY COLUMN company_id BIGINT NOT NULL;

-- Step 5: Add foreign key constraint (ON DELETE CASCADE — deleting a company cascades to its users)
ALTER TABLE users ADD CONSTRAINT fk_users_company FOREIGN KEY (company_id) REFERENCES companies(id) ON DELETE CASCADE;

-- Step 6: Add index for company scoping (used in list_by_company, find_by_id_in_company queries)
CREATE INDEX idx_users_company_id ON users(company_id);

-- Step 7: Release lock
UNLOCK TABLES;
