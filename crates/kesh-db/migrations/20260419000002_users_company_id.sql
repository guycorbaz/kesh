-- Story 6.2 — Multi-tenant scoping refactor
-- Add company_id to users table to enable proper multi-tenant isolation

-- Step 0: Lock tables to prevent race condition (concurrent INSERTs while migrating)
-- Must lock both users (WRITE) and companies (READ) since UPDATE uses companies in subquery
LOCK TABLES users WRITE, companies READ;

-- Step 1: Add nullable company_id for backfill
ALTER TABLE users ADD COLUMN company_id BIGINT NULL;

-- Step 2: Backfill — assign all users to the first company (mono-tenant assumption)
UPDATE users SET company_id = (SELECT id FROM companies ORDER BY id LIMIT 1);

-- Step 2b: Guard against multi-company data corruption (edge case: rollback + re-run on multi-tenant DB)
SET @distinct_companies = (SELECT COUNT(DISTINCT company_id) FROM users WHERE company_id IS NOT NULL);
IF @distinct_companies > 1 THEN
  SET @should_fail = 1;
  SET @error_msg = 'MIGRATION ERROR: Backfill detected users assigned to multiple companies. This indicates a multi-tenant DB or partial rollback. Restore from backup or manually clean.';
END IF;

-- Step 3: Verify backfill succeeded (fail if any users are orphaned)
SELECT COUNT(*) as orphaned_users FROM users WHERE company_id IS NULL INTO @orphaned_count;
SET @has_users = (SELECT COUNT(*) FROM users);
SET @has_companies = (SELECT COUNT(*) FROM companies);

-- Guard clause: if users exist but no companies, backfill failed
SELECT CASE
    WHEN @has_users > 0 AND @has_companies = 0 THEN 1
    WHEN @has_users > 0 AND @orphaned_count > 0 THEN 1
    ELSE 0
END INTO @should_fail;

-- If guard condition triggered, error out with explicit message
SET @error_msg = CASE
    WHEN @has_users > 0 AND @has_companies = 0 THEN 'MIGRATION ERROR: Cannot backfill users.company_id — no companies exist. Create at least one company before running this migration.'
    WHEN @has_users > 0 AND @orphaned_count > 0 THEN 'MIGRATION ERROR: Backfill failed — some users remain without company_id.'
    ELSE ''
END;

-- Trigger the error if condition is true (MySQL 5.7+ SIGNAL construct)
IF @should_fail = 1 THEN
  SIGNAL SQLSTATE '45000' SET MESSAGE_TEXT = @error_msg;
END IF;

-- Step 4: Make company_id NOT NULL after successful backfill
ALTER TABLE users MODIFY COLUMN company_id BIGINT NOT NULL;

-- Step 5: Add foreign key constraint (ON DELETE CASCADE — deleting a company cascades to its users)
ALTER TABLE users ADD CONSTRAINT fk_users_company FOREIGN KEY (company_id) REFERENCES companies(id) ON DELETE CASCADE;

-- Step 6: Add index for company scoping (used in list_by_company, find_by_id_in_company queries)
CREATE INDEX idx_users_company_id ON users(company_id);

-- Step 7: Release lock
UNLOCK TABLES;
