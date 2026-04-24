---
story_id: 7.1
story_slug: 7-1-audit-complete-kf-002-multi-tenant
review_pass: 1
review_date: 2026-04-24
reviewed_by: Haiku (blind+edge+auditor layers)
status: remediation-in-progress
stepsCompleted:
  - gather-context
  - parallel-review-layers
  - triage-consolidation
---

# Story 7-1 Code Review — Pass 1 Results

**Status:** Remediation required before Pass 2  
**Total Findings:** 29 raw → 17 consolidated findings  
**Failed Layers:** None (all 3 layers completed)

---

## Summary

Story 7-1 is a comprehensive multi-tenant scoping audit. The **spec compliance is incomplete** (AC 1 & AC 2 deliverables empty), and the **implementation has 9 critical+high severity code bugs** related to concurrency, transaction safety, and data validation.

**Story cannot pass review until:**
1. AC 1 + AC 2 deliverables are generated (CSV report, SQL audit)
2. 5 CRITICAL code bugs are fixed (race conditions, NULL handling, unsafe reset endpoint)
3. 4 HIGH code bugs are fixed (migration idempotency, account state, transaction consistency)

---

## 🔴 CRITICAL Findings (5)

### P1-001: AC 1 NOT MET — endpoints-audit.csv Empty
- **Classification:** intent_gap
- **Severity:** CRITICAL
- **Spec Requirement:** AC 1: "Générer un rapport structuré CSV/JSON listant chaque endpoint et son status de scoping"
- **Issue:** `endpoints-audit.csv` contains header row only; zero data rows. Audit analysis exists (31 endpoints, 28 scoped, 3 public) but was not exported to CSV.
- **Remediation:** Execute `scripts/audit-tenant-scoping.py` and populate CSV with all 31 endpoints + scoping status
- **Effort:** 15 minutes
- **Blocking:** YES — must complete before Pass 2

### P1-002: AC 2 NOT MET — sql-audit.md Empty
- **Classification:** intent_gap
- **Severity:** CRITICAL
- **Spec Requirement:** AC 2: "Lister toutes les migrations (`migrations/*.sql`) et vérifier les constraints de tenant"
- **Issue:** `sql-audit.md` is template-only; zero audit data. Migrations not itemized; findings not severity-classified per AC 2
- **Remediation:** Generate itemized list of 10 migrations with tenant constraint verification status per migration
- **Effort:** 20 minutes
- **Blocking:** YES — must complete before Pass 2

### P1-003: Race Condition finalize() — INSERT IGNORE Idempotency Issue
- **Classification:** patch
- **Severity:** CRITICAL
- **Location:** `crates/kesh-api/src/routes/onboarding.rs:452-512` + `crates/kesh-db/src/repositories/company_invoice_settings.rs:304-315`
- **Issue:** 
  - INSERT IGNORE silently suppresses duplicates without explicit rows_affected() check
  - Concurrent finalize() calls: Thread A locks, inserts, commits; Thread B waits, acquires lock, sees step==8, returns early without re-checking if INSERT succeeded
  - SELECT after INSERT assumes insert completed, but if another transaction modifies between INSERT and SELECT, stale data returned
  - Pattern: INSERT IGNORE → (stale window) → SELECT → use SELECT result assuming INSERT succeeded
- **Risk:** Data consistency corruption; potential IDOR if account IDs become NULL
- **Remediation:**
  - Add explicit rows_affected() check: `if rows_affected == 0 { return AlreadyExists }`
  - Or use `INSERT ... ON DUPLICATE KEY UPDATE` pattern for explicit handling
  - Or redesign with upsert semantics (not INSERT IGNORE)
- **Effort:** 1-2 hours
- **Blocking:** YES — CRITICAL race condition affecting core onboarding

### P1-004: INSERT IGNORE Creates Rows with NULL Account IDs
- **Classification:** patch
- **Severity:** CRITICAL
- **Location:** `crates/kesh-db/src/repositories/company_invoice_settings.rs:304-315` (insert_with_defaults_in_tx)
- **Issue:**
  - Account lookups (SELECT FOR UPDATE) return NULL if accounts 1100/3000 don't exist
  - NULL values are bound to INSERT parameters and inserted with company_invoice_settings
  - FK constraint permits NULL (correct design), so INSERT succeeds with NULL account references
  - Validation at finalize() rejects, but NULL row persists in DB
  - Subsequent calls overwrite via INSERT IGNORE idempotency, leaving corrupted rows
- **Risk:** Data corruption; if accounts deleted concurrently after SELECT FOR UPDATE lock released, NULL rows created
- **Remediation:**
  - Add explicit NULL check BEFORE INSERT: `if receivable.is_none() || revenue.is_none() { return Err("Accounts not found") }`
  - Or use INSERT ... ON DUPLICATE KEY UPDATE with non-NULL defaults
  - Move validation before INSERT, not after
- **Effort:** 30 minutes
- **Blocking:** YES — prevents data corruption

### P1-005: POST /api/v1/onboarding/reset Unguarded — Can Reset Post-Completion
- **Classification:** patch
- **Severity:** CRITICAL
- **Location:** `crates/kesh-api/src/routes/onboarding.rs:390-398`
- **Issue:**
  - No precondition check; can be called at any step, including step==8 (post-finalization)
  - Calling reset after finalization deletes ALL demo data (companies, accounts, invoices, journal entries, products, contacts) and resets onboarding_state to step==0
  - No confirmation dialog, no audit trail, no step gating
  - User can accidentally wipe all data by calling reset
- **Risk:** Data loss; accidental reset post-completion violates principle of least surprise
- **Remediation:**
  - Add step gating: `if step_completed > 2 { return AppError::Forbidden("Cannot reset after step 2") }`
  - Or require explicit confirmation (POST body flag: `{ confirm: true }`)
  - Or deprecate endpoint and require explicit admin action
- **Effort:** 30 minutes
- **Blocking:** YES — prevents accidental data loss

---

## 🟠 HIGH Findings (4)

### P1-006: Migration Idempotency Risk — Conditional UPDATE + NOT NULL
- **Classification:** patch
- **Severity:** HIGH
- **Location:** `crates/kesh-db/migrations/20260419000002_users_company_id.sql` (Step 2-3)
- **Issue:**
  - Step 2 uses conditional UPDATE with EXISTS check (not truly idempotent)
  - Step 3 adds NOT NULL constraint after UPDATE
  - If migration framework loses track and re-runs, UPDATE is no-op (all company_id already set), Step 3 succeeds harmlessly
  - Risk: Fresh database with NO companies/NO users creates bootstrap dependency; later changes could violate constraint
- **Remediation:**
  - Ensure sqlx migration framework tracks completion correctly
  - Add explicit version tracking in migration or idempotency guards
  - Consider using `ALTER TABLE ... MODIFY ... DEFAULT ...` before NOT NULL
- **Effort:** 1 hour
- **Blocking:** YES — migration safety

### P1-007: Missing NULL Validation Before INSERT — Account Lookup
- **Classification:** patch
- **Severity:** HIGH
- **Location:** `crates/kesh-db/src/repositories/company_invoice_settings.rs:278-326` (insert_with_defaults_in_tx)
- **Issue:**
  - Account lookups return NULL if accounts 1100/3000 don't exist
  - NULL values bound directly to INSERT, creating rows with NULL account IDs
  - Validation happens AFTER INSERT (in finalize()), not before
  - Error path: insert_with_defaults() returns Ok, finalize() later rejects during validation
  - Inefficient: NULL row created before rejection
- **Remediation:**
  - Add early validation BEFORE INSERT: `if receivable.is_none() || revenue.is_none() { return Err(...) }`
  - Fail-fast: reject before touching database
- **Effort:** 30 minutes
- **Blocking:** YES — error path efficiency

### P1-008: GitHub Issues Section — Contradicts Story Completion Notes
- **Classification:** patch
- **Severity:** HIGH
- **Location:** `_bmad-output/implementation-artifacts/KF-002-AUDIT-REPORT.md` (GitHub Issues section)
- **Issue:**
  - Report states: "Created: None yet (workflow requirement: create during story execution)"
  - Story completion notes claim: "Created issues #40, #41 for findings"
  - Actual `gh issue list` shows issues #40, #41 exist
  - Contradiction between audit report and story state
- **Remediation:**
  - Update KF-002-AUDIT-REPORT.md "GitHub Issues" section to correctly reference issues #40, #41 as "Created" with links
  - Fix documentation accuracy
- **Effort:** 15 minutes
- **Blocking:** YES — spec compliance (AC 5 requires accurate issue reporting)

### P1-009: Account Deactivation Leaves Stale Account IDs in Settings
- **Classification:** patch
- **Severity:** HIGH
- **Location:** `crates/kesh-db/src/repositories/company_invoice_settings.rs:365-376` vs. `frontend/src/lib/components/invoices/InvoiceForm.svelte:248-268`
- **Issue:**
  - Account lookups filter `active = true` when writing defaults
  - Frontend doesn't validate if accounts are still active when reading settings
  - If account is deactivated AFTER settings creation, account ID remains (FK not violated; hard delete prevented by ON DELETE RESTRICT)
  - InvoiceForm displays stale account ID; ID won't appear in UI dropdowns (filter by `active=true`)
  - User creates invoice with stale account ID → server may reject or allow depending on validation
- **Risk:** Confusing UX; invoice creation may fail with "account not found" even though ID is stored
- **Remediation:**
  - Validate account `active` status in finalize() or at invoice creation
  - Or cascade soft-delete: update company_invoice_settings to NULL account IDs when account is deactivated
  - Or explicitly load account and check active flag before using in invoice
- **Effort:** 1-2 hours
- **Blocking:** MEDIUM — affects UX but not data safety

### P1-010: Inconsistent Transaction Handling — insert_with_defaults Variants
- **Classification:** patch
- **Severity:** HIGH
- **Location:** `crates/kesh-db/src/repositories/company_invoice_settings.rs` (pool vs. tx variants)
- **Issue:**
  - `insert_with_defaults()` (pool variant): auto-commits; implicit rollback on error
  - `insert_with_defaults_in_tx()` (tx variant): caller handles commit; no auto-rollback
  - If pool variant fails partway, company updates may be committed even if settings insertion fails
  - Caller (seed_demo) doesn't distinguish between the two error paths
- **Risk:** Inconsistent error recovery; seed operation could leave DB in partial state
- **Remediation:**
  - Standardize error handling: ensure both variants either auto-rollback or auto-commit
  - Or explicitly document the difference and handle both paths in seed_demo
  - Prefer: both variants should fail-safe (rollback on partial success)
- **Effort:** 1 hour
- **Blocking:** YES — error handling consistency

---

## 🟡 MEDIUM Findings (6)

### P1-011: Spec Constraint — Missing Severity Classification in Supporting Reports
- **Classification:** patch
- **Severity:** MEDIUM
- **Location:** `endpoints-audit.csv`, `sql-audit.md`
- **Issue:** AC 5 requires findings classified by severity (CRITICAL/HIGH/MEDIUM/LOW). Main report does this; supporting reports (CSV, SQL audit) are empty.
- **Remediation:** Remplir les deliverables avec severity classification (see P1-001 + P1-002)
- **Effort:** Included in P1-001, P1-002 remediation
- **Blocking:** YES (covered by intent_gap items)

### P1-012: SELECT FOR UPDATE Deadlock Potential — Missing Global Lock Ordering Documentation
- **Classification:** defer
- **Severity:** MEDIUM
- **Location:** `crates/kesh-api/src/routes/onboarding.rs` (finalize function)
- **Issue:** Lock order in finalize(): onboarding_state → companies → accounts. No documented global lock ordering for ALL endpoints. Future endpoints could lock in different order → circular deadlock.
- **Remediation:** Document global lock ordering in ADR. Enforce in code review for all future endpoints.
- **Status:** Pre-existing architecture issue; not caused by Story 7-1. Should be tracked as KF-003.
- **Blocking:** NO (defer to separate KF)

### P1-013: Implicit Assumption — Exactly One Company Per Deployment
- **Classification:** defer
- **Severity:** MEDIUM
- **Location:** `crates/kesh-api/src/routes/onboarding.rs:429` + `crates/kesh-seed/src/lib.rs:75`
- **Issue:** Both finalize() and seed_demo() use `LIMIT 1` without "exactly one company" check. Assumption documented but not enforced at runtime. If multi-company feature added later, LIMIT 1 silently picks first company.
- **Remediation:** Add explicit runtime check: `if companies.len() != 1 { return Err(...) }`
- **Status:** Pre-existing; tracker as KF-004.
- **Blocking:** NO (defer to separate KF)

### P1-014: CodeQL Workflow Disabled — No Static Analysis on Every Push/PR
- **Classification:** defer
- **Severity:** MEDIUM
- **Location:** `.github/workflows/codeql.yml`
- **Issue:** CodeQL disabled for automatic runs; only manual trigger available. At multi-tenant scoping stage, security scanning is critical.
- **Remediation:** Re-enable CodeQL for nightly runs or PR-based scheduled runs (accept slower CI).
- **Status:** Pre-existing CI/CD decision; tracker as KF-005.
- **Blocking:** NO (defer to separate KF)

### P1-015: seed_demo() Uses LIMIT 1 Without Uniqueness Check
- **Classification:** defer
- **Severity:** MEDIUM
- **Location:** `crates/kesh-seed/src/lib.rs:75`
- **Issue:** seed_demo() fetches first company without checking if it's the only one. If multiple companies exist (corrupted state), overwrites first company with demo data.
- **Remediation:** Add check: `if companies.len() != 1 { return Err(...) }`
- **Status:** Pre-existing; tracker as KF-006.
- **Blocking:** NO (defer to separate KF)

### P1-016: Missing Test Coverage — insert_with_defaults_in_tx() Transaction Variant
- **Classification:** defer
- **Severity:** MEDIUM
- **Location:** `crates/kesh-db/src/repositories/company_invoice_settings.rs:278-326`
- **Issue:** Transaction-aware variant only called from finalize(). No unit tests. E2E tests cover happy path but not concurrent scenarios.
- **Remediation:** Add unit tests for insert_with_defaults_in_tx() covering concurrent access, lock ordering, account lookup failures.
- **Status:** Pre-existing testing gap; tracker as KF-007.
- **Blocking:** NO (defer to separate KF)

---

## 📋 Remediation Checklist (Pass 1 → Pass 2)

**BLOCKING Remediation (must complete before Pass 2):**
- [ ] P1-001: Execute audit script, populate endpoints-audit.csv (15 min)
- [ ] P1-002: Generate sql-audit.md with 10 migrations itemized (20 min)
- [ ] P1-003: Fix INSERT IGNORE race condition (1-2 hours)
- [ ] P1-004: Add NULL validation before INSERT (30 min)
- [ ] P1-005: Add step gating to reset endpoint (30 min)
- [ ] P1-006: Ensure migration idempotency (1 hour)
- [ ] P1-007: Add early NULL validation in insert_with_defaults_in_tx (30 min)
- [ ] P1-008: Update GitHub Issues section in audit report (15 min)
- [ ] P1-009: Fix account deactivation issue (1-2 hours)
- [ ] P1-010: Standardize transaction handling (1 hour)

**DEFERRED (tracker as KF, not blocking):**
- KF-003: Lock ordering deadlock potential
- KF-004: Single company assumption
- KF-005: CodeQL disabled
- KF-006: seed_demo() uniqueness
- KF-007: Missing insert_with_defaults_in_tx() tests

**Total Effort:** ~6-8 hours (blocking items)

---

## Next Steps

**When you've applied all blocking remediation patches:**
1. Confirm that changes are committed locally
2. I will run **Pass 2** with a different LLM model (Sonnet) and fresh context
3. Pass 2 will verify whether the patches addressed the findings
4. If Pass 2 finds > LOW severity issues, we repeat with another LLM (Haiku)
5. Continue until zero CRITICAL/HIGH/MEDIUM or 8 passes reached

**To trigger Pass 2:**
Just respond with: `Pass 2 ready` (or similar) once patches are applied and committed.

---

## Review Metadata

| Field | Value |
|-------|-------|
| Pass | 1 |
| Date | 2026-04-24 |
| LLM | Haiku (blind+edge+auditor) |
| Raw Findings | 29 |
| Consolidated | 17 |
| CRITICAL | 5 |
| HIGH | 4 |
| MEDIUM | 6 |
| Defer (KF) | 5 |
| Rejected | 3 |

---

