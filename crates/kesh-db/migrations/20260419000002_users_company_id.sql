-- Story 6.2 — Multi-tenant scoping refactor
-- Add company_id to users table to enable proper multi-tenant isolation

-- Step 0: Lock tables to prevent race condition (concurrent INSERTs while migrating)
-- Must lock both users (WRITE) and companies (READ) since UPDATE uses companies in subquery
LOCK TABLES users WRITE, companies READ;

-- Step 1: Add nullable company_id for backfill
ALTER TABLE users ADD COLUMN company_id BIGINT NULL;

-- Step 2: Backfill — assign all users to the first company (mono-tenant assumption)
UPDATE users SET company_id = (SELECT id FROM companies ORDER BY id LIMIT 1);

-- Step 3: Guard — Defensive data integrity checks before making company_id NOT NULL
-- Check 1 (C1): Verify guard conditions and signal if backfill failed
SET @user_count = (SELECT COUNT(*) FROM users);
SET @company_count = (SELECT COUNT(*) FROM companies);
SET @should_fail = 0;
SET @error_msg = '';

-- If users exist but no companies, UPDATE produced NULLs (backfill impossible)
IF @user_count > 0 AND @company_count = 0 THEN
  SET @should_fail = 1;
  SET @error_msg = 'Migration failed: users exist but no companies. Cannot backfill company_id. Create company before migration.';
END IF;

-- Check 2 (C2): Detect multi-company backfill corruption (if re-run or rollback edge case)
IF @user_count > 1 THEN
  SET @distinct_company_count = (SELECT COUNT(DISTINCT company_id) FROM users WHERE company_id IS NOT NULL);
  IF @distinct_company_count > 1 THEN
    SET @should_fail = 1;
    SET @error_msg = 'Migration failed: users in multiple companies detected. Check data integrity before retrying.';
  END IF;
END IF;

-- Execute signal if guard condition triggered
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
