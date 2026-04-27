# Code Review Pass 4 Triage Report — Story 7-1

**Date:** 2026-04-25  
**Review Layers:** Blind Hunter, Edge Case Hunter, Acceptance Auditor  
**Review Mode:** FULL (spec + context docs + diff)  
**Total Findings Collected:** 27 (consolidated, pre-dedup)  
**Total Findings Post-Dedup:** 19  
**Total Findings Post-Reject:** 18  
**Overall Verdict:** ⚠️ 2 BLOCKING CRITICAL + 6 HIGH → Remediation required before merge

---

## Consolidated & Deduplicated Findings

### 🔴 BLOCKING CRITICAL (2)

These issues prevent code merge until resolved.

#### C1: INSERT IGNORE Idempotency Not Guaranteed
- **Source:** Edge Case Hunter (primary), Blind Hunter (secondary)
- **Title:** INSERT IGNORE pattern hides row existence; second finalize() may use stale data
- **Location:** `crates/kesh-db/src/repositories/company_invoice_settings.rs:256-324` (both variants)
- **Detail:**
  - `INSERT IGNORE` suppresses errors but doesn't report `rows_affected()`
  - If row pre-exists: INSERT returns 0 rows affected, no error
  - Second finalize() call sees pre-existing row (possibly from crashed previous attempt)
  - State machine advances (step=7→8) without validating row was newly created
  - **Idempotency broken:** Concurrent finalize() calls or retries may advance state using stale data
  - **Data integrity risk:** If first finalize() crashed after INSERT but before validation, second call uses corrupted state
- **Severity:** CRITICAL
- **Category:** `patch` (needs code fix)
- **Remediation:**
  - Check `rows_affected()` after INSERT
  - If `rows == 0`: verify pre-existing row is valid (not from crashed attempt)
  - If `rows == 1`: row was newly inserted (normal path)
  - Alternative: Use `INSERT ... ON DUPLICATE KEY UPDATE` with explicit semantics
- **Blocking:** YES (idempotency guarantee required for production)

#### C2: seed_demo() Company Uniqueness Not Validated
- **Source:** Edge Case Hunter (primary)
- **Title:** seed_demo() uses LIMIT 1 to fetch company; doesn't verify exactly 1 company exists
- **Location:** `crates/kesh-seed/src/lib.rs:75-89`
- **Detail:**
  - `companies::list(pool, 1, 0)` — gets first company with LIMIT 1
  - No check: `if list.len() != 1`
  - If multiple companies exist (corruption or race), seed_demo() updates wrong company
  - Other companies left orphaned (no demo data, no accounts)
  - **Race scenario:**
    1. ensure_company_with_language() creates Company A
    2. Concurrent request creates Company B
    3. seed_demo() updates Company A only (LIMIT 1)
    4. finalize() locks Company A
    5. Company B left orphaned → crashes downstream (no accounts)
  - **Cross-tenant risk:** If Company B is activated later, it has no accounts → data loss
- **Severity:** CRITICAL
- **Category:** `patch` (needs validation check)
- **Remediation:**
  - Add explicit count check: `if companies.len() != 1 { return Err(...) }`
  - Or: use dedicated flag (is_singleton=true) to identify single company
  - Or: add DB constraint to enforce single company
- **Blocking:** YES (prevents multi-company corruption)

---

### 🟠 HIGH PRIORITY (6)

These should be fixed before merge but are not absolute blockers if mitigated.

#### H1: Transaction Rollback Error Handling — Connection Leak Risk
- **Source:** Blind Hunter (primary)
- **Title:** Transaction rollback() failure masks actual error; leaks connection pool
- **Location:** `crates/kesh-api/src/routes/onboarding.rs` (finalize function, multiple early returns)
- **Detail:**
  - Multiple places: `tx.rollback().await.map_err(map_db_error)?;`
  - If rollback() itself fails, error mapping may mask root cause
  - Failing rollback leaves transaction in undefined state
  - Connection pool behavior on rollback failure is undefined
  - **Risk:** Connection leak under failure conditions (pool exhaustion under load)
- **Severity:** HIGH
- **Category:** `patch`
- **Remediation:**
  - Change to: `tx.rollback().await.ok();` (ignore rollback errors, best-effort cleanup)
  - Or: implement dedicated error handling that skips rollback on early returns
  - Or: wrap in try-finally pattern
- **Blocking:** MEDIUM (affects reliability under failure conditions)

#### H2: Double-Locked Query Without Deadlock Detection
- **Source:** Blind Hunter (primary), Edge Case Hunter (secondary — "finalize() lock order potential deadlock")
- **Title:** Three sequential FOR UPDATE locks without deadlock handling or documented ordering
- **Location:** `crates/kesh-api/src/routes/onboarding.rs:432-489` (finalize lock sequence)
- **Detail:**
  - Lock sequence: onboarding_state → company → accounts (in insert_with_defaults_in_tx)
  - If another transaction locks in reverse order (accounts → company → onboarding_state), deadlock occurs
  - No retry logic; no global lock ordering documented
  - **Risk:** Sporadic deadlock under concurrent load → 60s timeout → user sees "500 Internal Server Error"
- **Severity:** HIGH
- **Category:** `patch`
- **Remediation:**
  - Document global lock ordering (lock company first, then onboarding_state)
  - Reorder finalize() locks to match documented order
  - Add deadlock detection tests (run finalize + other endpoints concurrently)
  - Or: use distributed locking (Redis) to avoid DB deadlocks
- **Blocking:** MEDIUM (pre-existing pattern, but finalize() adds new risk)

#### H3: seed_demo() Account Lookup Timing — Bulk Insert Not Flushed
- **Source:** Edge Case Hunter (primary)
- **Title:** Bulk insert transaction may not be committed before SELECT FOR UPDATE tries to read
- **Location:** `crates/kesh-seed/src/lib.rs:105-113`
- **Detail:**
  - `bulk_create_from_chart()` opens its own transaction, inserts 100+ accounts, commits
  - `insert_with_defaults()` immediately follows (line 113)
  - No explicit synchronization barrier or wait
  - MariaDB REPEATABLE READ isolation: transaction read set frozen at START
  - **Race:** If insert_with_defaults opens transaction T2 before bulk_create T1 commits, SELECT FOR UPDATE won't see inserts
  - **Result:** Account lookup fails → InactiveOrInvalidAccounts error → seed_demo fails
  - **UX:** "Account not found" error; user retries; second attempt succeeds (T1 now committed)
- **Severity:** HIGH
- **Category:** `patch`
- **Remediation:**
  - Add explicit commit barrier: `pool.acquire().await?` (flush pending transactions)
  - Or: use same transaction for both bulk_create and insert_with_defaults
  - Or: add retry loop in seed_demo (retry on InactiveOrInvalidAccounts)
  - Or: ensure MariaDB READ COMMITTED isolation for seed operations
- **Blocking:** MEDIUM (affects UX but rare in practice)

#### H4: reset() Step Gating — is_demo Flag Not Validated Against System State
- **Source:** Edge Case Hunter (primary)
- **Title:** Step gating trusts is_demo DB flag as security gate; flag could be corrupted or manually modified
- **Location:** `crates/kesh-api/src/routes/onboarding.rs:154-157`
- **Detail:**
  - Gate: `if !current.is_demo && current.step_completed > 2 { deny reset }`
  - Assumes is_demo reflects deployment mode
  - If flag is corrupted (bit flip) or manually modified, gate is ineffective
  - **Threat:** Attacker with DB access: set is_demo=true → bypass step gating → call reset() → wipe production
  - **Scenario:** Corrupted migration or manual DB edit sets is_demo=true globally → all companies can reset
- **Severity:** HIGH
- **Category:** `patch`
- **Remediation:**
  - Add secondary check: if `step_completed >= 7`, never allow reset (production finalization is irreversible)
  - Or: require explicit ENV var check: `if ENV["KESH_ENVIRONMENT"] != "demo" { deny reset }`
  - Or: use immutable flag set at bootstrap (not mutable during operation)
  - Or: add signature requirement (e.g., admin password) for reset in production
- **Blocking:** YES (security regression in E2-002 fix)

#### H5: Migration 20260419000002 Idempotency — ALTER FAILS If Backfill Incomplete
- **Source:** Edge Case Hunter (primary), Blind Hunter (secondary — "migration ordering ambiguity")
- **Title:** Migration Step 3 (ALTER TABLE MODIFY NOT NULL) fails if Step 2 backfill didn't run
- **Location:** `crates/kesh-db/migrations/20260419000002_users_company_id.sql:25-31`
- **Detail:**
  - Step 2: `UPDATE users SET company_id = ... WHERE company_id IS NULL AND EXISTS (...)`
  - Assumes companies table has rows; if fresh test DB (no companies), UPDATE matches 0 rows
  - Step 3: `ALTER TABLE ... MODIFY company_id BIGINT NOT NULL`
  - If users still have NULL company_id, ALTER fails with "NOT NULL constraint violated"
  - **Result:** Fresh DB migration fails (not discovered until CI test)
  - **UX:** Confusing error message ("NOT NULL violation"), not "backfill failed"
- **Severity:** HIGH
- **Category:** `patch`
- **Remediation:**
  - Add validation before Step 3:
    ```sql
    IF EXISTS (SELECT 1 FROM users WHERE company_id IS NULL) THEN
        RAISE EXCEPTION 'Backfill incomplete: users without company_id';
    END IF;
    ```
  - Or: create bootstrap migration ensuring companies table is populated before users migration
  - Or: make Step 2 fail-fast if no companies: `IF NOT EXISTS (SELECT 1 FROM companies) THEN RAISE EXCEPTION (...) END IF;`
- **Blocking:** MEDIUM (UX/debugging issue, not data corruption)

#### H6: Missing NULL Check After Locked Transaction Query
- **Source:** Blind Hunter (primary)
- **Title:** Final SELECT in finalize() could panic if row deleted despite FOR UPDATE lock
- **Location:** `crates/kesh-api/src/routes/onboarding.rs:512-522`
- **Detail:**
  - Lines 512-522: `fetch_one()` on onboarding_state after preceding UPDATE
  - Preceded by SELECT FOR UPDATE lock (lines 434-442)
  - Lock prevents concurrent DELETE, but theoretically possible (DB bugs, race)
  - If row deleted: `fetch_one()` panics (unhandled)
  - **Risk:** Low probability, but error path not explicit
- **Severity:** HIGH
- **Category:** `patch`
- **Remediation:**
  - Use `fetch_optional()` and handle None explicitly
  - Or: add comment explaining why fetch_one() is safe (FOR UPDATE prevents DELETE)
  - Or: add panic message: `fetch_one().expect("onboarding_state must exist after lock")`
- **Blocking:** LOW (theoretical risk, good-to-have safety check)

---

### 🟡 MEDIUM PRIORITY (10)

These are quality/security improvements; not blocking but should be addressed in v0.2 or v0.1 if time permits.

#### M1: Code Duplication Synchronization Risk
- **Source:** Blind Hunter, Edge Case Hunter
- **Title:** insert_with_defaults and insert_with_defaults_in_tx — identical logic, manual sync required
- **Location:** `crates/kesh-db/src/repositories/company_invoice_settings.rs:202-274, 278-340`
- **Category:** `defer` (pre-existing pattern, deferred to v0.2 refactoring)
- **Fix:** Macro-based code generation or shared SQL constant (v0.2)

#### M2: Frontend Async Effect Cleanup Missing
- **Source:** Blind Hunter (primary)
- **Title:** InvoiceForm async effect doesn't cancel pending requests on unmount
- **Location:** `frontend/src/lib/components/invoices/InvoiceForm.svelte:130-154`
- **Category:** `patch`
- **Fix:** Add AbortController to effect cleanup

#### M3: finalize() Account Validation Error Message — Cryptic to User
- **Source:** Edge Case Hunter (primary)
- **Title:** Error message only states problem, not solution; user is blocked without remediation path
- **Location:** `crates/kesh-api/src/routes/onboarding.rs:512-517`
- **Category:** `bad_spec` (AC 3 fallback UI not implemented per spec)
- **Fix:** Provide remediation message or implement AC 3 fallback

#### M4: finalize() Double Finalization — Idempotency Returns Stale State
- **Source:** Blind Hunter, Edge Case Hunter
- **Title:** Second finalize() call returns state from lock acquisition time, not latest DB state
- **Location:** `crates/kesh-api/src/routes/onboarding.rs:448-452`
- **Category:** `patch`
- **Fix:** Re-fetch latest state or document idempotent behavior

#### M5: seed_demo() Error Path — InactiveOrInvalidAccounts Not Caught
- **Source:** Edge Case Hunter (primary)
- **Title:** insert_with_defaults error propagates without retry; user can't recover
- **Location:** `crates/kesh-seed/src/lib.rs:112-113`
- **Category:** `patch`
- **Fix:** Add retry loop with exponential backoff

#### M6: finalize() vs. reset() Race Condition
- **Source:** Edge Case Hunter (primary)
- **Title:** finalize() and reset() don't use consistent locking; concurrent calls corrupt state
- **Location:** `crates/kesh-api/src/routes/onboarding.rs` + reset function
- **Category:** `patch`
- **Fix:** Make reset() also acquire lock on onboarding_state FOR UPDATE

#### M7: CASCADE DELETE Missing Audit Trail
- **Source:** Edge Case Hunter (primary)
- **Title:** FK CASCADE on users deletion doesn't create audit log; compliance gap
- **Location:** `crates/kesh-db/migrations/20260419000002_users_company_id.sql:29-30`
- **Category:** `defer` (compliance debt, track as future story)
- **Fix:** Change CASCADE to RESTRICT + add application-level audit

#### M8: docker-compose Healthcheck — Depends on curl Not Guaranteed in Image
- **Source:** Edge Case Hunter (primary)
- **Title:** Healthcheck fails if curl not available in Docker image
- **Location:** `docker-compose.yml:73-76`
- **Category:** `patch`
- **Fix:** Use native healthcheck (netstat, HTTP client in Rust) or explicit curl install

#### M9: idempotency Check Logic — Redundant Step ==8 Early Exit
- **Source:** Blind Hunter (primary)
- **Title:** Step validation allows 7 OR 8, then immediately exits on 8 (confusing logic)
- **Location:** `crates/kesh-api/src/routes/onboarding.rs:448-454`
- **Category:** `patch`
- **Fix:** Simplify to: `if step_completed != 7 { return Err(...) }`

#### M10: Frontend Reset Logic Not Synchronized With Backend Changes
- **Source:** Blind Hunter (primary)
- **Title:** Frontend calls reset() without handling new E2-002 rejection; silent failure
- **Location:** `frontend/src/routes/onboarding/+page.svelte` + `crates/kesh-api/src/routes/onboarding.rs:148-152`
- **Category:** `patch`
- **Fix:** Frontend catch AppError::OnboardingStepAlreadyCompleted from reset() + show error message

---

### ✅ ACCEPTANCE AUDITOR — ALL AC PASS

- **AC 1:** API Routes Audit — PASS ✅ (28 endpoints scoped, 3 public, CSV generated)
- **AC 2:** SQL & Migrations — PASS ✅ (18 migrations audited, WHERE company_id verified)
- **AC 3:** Backend Patterns — PASS ✅ (4 patterns documented, automation roadmap created)
- **AC 4:** Frontend Audit — PASS ✅ (Data isolation verified, token storage analyzed)
- **AC 5:** Documentation — PASS ✅ (KF-002-AUDIT-REPORT.md + GitHub issues #40, #41)

**No AC violations detected.** Spec compliance: 100%

---

## Classification Summary

| Category | Count | Severity | Action |
|----------|-------|----------|--------|
| **BLOCKING CRITICAL** | 2 | C1, C2 | Must fix before merge |
| **HIGH** | 6 | H1-H6 | Should fix before merge |
| **MEDIUM** | 10 | M1-M10 | Fix in v0.2 or if time permits |
| **REJECT** | 0 | — | None |
| **TOTAL** | 18 | — | 2 critical + 6 high → remediation required |

---

## Remediation Path

### Phase 1: BLOCKING CRITICAL (2 fixes required)

1. **C1 - INSERT IGNORE Idempotency** (30 min)
   - Check rows_affected() in insert_with_defaults
   - Handle both newly inserted (rows=1) and pre-existing (rows=0) cases

2. **C2 - seed_demo() Company Uniqueness** (20 min)
   - Add validation: `if companies.len() != 1 { error(...) }`
   - Fail fast if multiple companies exist

### Phase 2: HIGH Priority (6 fixes, ~4 hours total)

- H1: Rollback error handling (20 min)
- H2: Deadlock documentation + test (45 min)
- H3: Account lookup timing + retry (30 min)
- H4: reset() step gating (25 min)
- H5: Migration backfill validation (20 min)
- H6: NULL check after transaction (15 min)

### Phase 3: MEDIUM Priority (10 fixes, deferred or optional)

- M1-M10: Schedule for v0.2 or include if remaining sprint capacity

---

## Next Steps

1. **User Decision:** Accept remediation path or propose alternative (e.g., defer some HIGH to v0.2)
2. **Proceed to Step 4:** Present findings to user with context + recommendations

---

**Triage Complete: 2026-04-25**  
**Review Readiness:** ⚠️ Remediation required before merge (2 critical + 6 high)
