-- Story 6.2 — Multi-tenant scoping refactor
-- Add company_id to users table to enable proper multi-tenant isolation
-- Assumption: mono-tenant at migration time (exactly 1 company exists)

-- Step 1: Add nullable company_id for backfill
ALTER TABLE users ADD COLUMN company_id BIGINT NULL;

-- Step 2: Guard-rail check — ensure at least 1 company exists for backfill
-- Fail with clear error if companies table is empty (prevents data corruption)
DO BEGIN
  DECLARE v_company_count INT;
  SELECT COUNT(*) INTO v_company_count FROM companies;
  IF v_company_count = 0 THEN
    SIGNAL SQLSTATE '45000' SET MESSAGE_TEXT = 'Migration 20260419000002: Cannot backfill users.company_id — no companies exist. Run onboarding first.';
  END IF;
END;

-- Step 3: Backfill — assign all users to the first company (mono-tenant assumption)
-- Safe because DO block above guarantees ≥1 company exists
UPDATE users SET company_id = (SELECT id FROM companies ORDER BY id LIMIT 1);

-- Step 4: Verify all users now have company_id (catch backfill failures)
-- Fail if any user still has NULL company_id (indicates backfill didn't complete)
DO BEGIN
  DECLARE v_null_count INT;
  SELECT COUNT(*) INTO v_null_count FROM users WHERE company_id IS NULL;
  IF v_null_count > 0 THEN
    SIGNAL SQLSTATE '45000' SET MESSAGE_TEXT = CONCAT('Migration 20260419000002: Backfill incomplete — ', v_null_count, ' users still have NULL company_id');
  END IF;
END;

-- Step 5: Make company_id NOT NULL after successful backfill
ALTER TABLE users MODIFY COLUMN company_id BIGINT NOT NULL;

-- Step 6: Add foreign key constraint (ON DELETE CASCADE — deleting a company cascades to its users)
ALTER TABLE users ADD CONSTRAINT fk_users_company FOREIGN KEY (company_id) REFERENCES companies(id) ON DELETE CASCADE;

-- Step 7: Add index for company scoping (used in list_by_company, find_by_id_in_company queries)
CREATE INDEX idx_users_company_id ON users(company_id);
