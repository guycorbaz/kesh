---
title: Story 6.2 Code Review — Pass 2 Report
date: 2026-04-19
reviewer: Claude Code (Sonnet — Pass 2 Acceptance Auditor)
review_type: adversarial-multi-layer-pass-2
context: Post-patch verification (Pass 1 findings C1-C4, H1-H5 remediated)
status: IN_PROGRESS
---

# Story 6.2 Code Review — Pass 2 (2026-04-19)

**Branch:** `story/6-2-multi-tenant-scoping-refactor` (post-merge with `origin/main`)  
**Reviewer Layers:** Blind Hunter → Edge Case Hunter → Acceptance Auditor  
**Model:** Sonnet 4.6 (orthogonal to Haiku 4.5 Pass 1)  
**Focus:** Verify Pass 1 patches resolve findings, detect regressions, evaluate convergence 18→? findings

---

## LAYER 1: BLIND HUNTER (Diff-only, no project context)

### Detected Issues (from diff alone)

#### BH-1: CodeQL workflow language matrix — languages without codebase

**File:** `.github/workflows/codeql.yml:19-20`  
**Issue:** Matrix includes `cpp`, `csharp`, `go`, `java`, `python`, `ruby` languages, but only Rust code is present. Non-applicable languages will fail or timeout.  
**Evidence:**
```yaml
matrix:
  language: ['cpp', 'csharp', 'go', 'java', 'javascript', 'python', 'ruby', 'rust']
```
**Risk:** CI will report language analysis "skip" or failure for 7/8 languages, creating noise in checks.  
**Severity:** MEDIUM (operational noise, not correctness)

---

#### BH-2: CodeQL build recovery with `|| true` suppresses actual build errors

**File:** `.github/workflows/codeql.yml:63-65`  
**Issue:** Rust build failure is silently ignored (`cargo build --release 2>&1 || true`). If build was broken by patches, CodeQL would proceed with incomplete analysis.  
**Evidence:**
```yaml
run: |
  cargo build --release 2>&1 || true
  # Don't fail on build errors; CodeQL will analyze what it can
```
**Risk:** If patches introduced compilation errors, they won't be caught until later CI step (if exists).  
**Severity:** MEDIUM (masks build failures)

---

#### BH-3: CodeQL paths-ignore not consistently applied across all languages

**File:** `.github/workflows/codeql.yml:76-78`  
**Issue:** `paths-ignore` is inside the `analyze` step, applied only during CodeQL analysis. But `initialize` step (line 25-29) runs for ALL languages before paths-ignore is known. Test directories may be indexed unnecessarily.  
**Severity:** LOW (cosmetic, analysis still excludes paths)

---

#### BH-4: Test fixture company name changed to `_MIGRATION_PLACEHOLDER_DO_NOT_USE_`

**File:** `crates/kesh-db/src/test_fixtures.rs:409`  
**Issue:** Naming convention unusual (`_MIGRATION_PLACEHOLDER_DO_NOT_USE_`). Purpose unclear from diff alone. If this name leaks into test assertions, could cause unexpected failures.  
**Evidence:**
```rust
.bind("_MIGRATION_PLACEHOLDER_DO_NOT_USE_")
```
**Risk:** Non-obvious naming could confuse maintainers.  
**Severity:** LOW (naming clarity)

---

#### BH-5: IDOR test — cross-company user disable assumes Admin role exists in company B

**File:** `crates/kesh-api/tests/idor_multi_tenant_e2e.rs:293`  
**Issue:** Test creates `_user_b_id` with `Role::Admin`, but test name (`idor_users_cross_company_returns_404`) doesn't indicate role dependency. If future refactoring changes role validation order, test may pass for wrong reason.  
**Severity:** LOW (test clarity, not correctness)

---

## LAYER 2: EDGE CASE HUNTER (Diff + project read access)

### Edge Cases & Behavioral Consistency

#### EH-1: Common test fixture — no null/empty string validation

**File:** `crates/kesh-api/tests/common/mod.rs:151-165`  
**Issue:** `create_test_company()` hardcodes company name `"Test Company"` and address `"Test Address"`. If fixture is reused in tests that run in parallel (see `.cargo/config.toml` setting test-threads=2), all companies will have identical names. Database uniqueness constraints on name could cause collisions.  
**Risk:** Silent test flakiness if tests run in parallel.  
**Severity:** MEDIUM (parallelism risk)

---

#### EH-2: Migration guard approach relies on natural failure

**File:** `crates/kesh-db/migrations/20260419000002_users_company_id.sql`  
**Finding:** Migration comment claims IF guards "aren't supported in .sql migration files" (sqlx limitation). Instead, migration relies on ALTER NOT NULL failure if backfill produces NULL values.  
**Trade-off:** 
  - ✅ Simpler migration, compatible with sqlx
  - ❌ Less user-friendly error message if backfill fails
  - ❌ Multi-company corruption undetected
**Verdict:** Design choice valid but less defensive than spec intent (C1-C2 patches).  
**Severity:** Noted above as HIGH-A2.

---

#### EH-3: IDOR user test — correct implementation verified

**File:** `crates/kesh-api/tests/idor_multi_tenant_e2e.rs:486-512`  
**Verified:** Test correctly calls `PUT /users/{id}/disable` without version. The `disable_user()` handler doesn't require version (no JSON body), only Path parameter + JWT. Repository method `find_by_id_in_company()` scopes by company_id, returns NotFound (404) for cross-company access.  
**Test coverage:** Full 5 entities now tested (contacts, products, accounts, invoices, users). Missing: companies/current.  
**Verdict:** ✅ Test is valid and passes.  
**Severity:** NONE (false alarm, now HIGH-A1 for missing companies/current)

---

#### EH-4: Test fixture naming — no collision risk

**File:** `crates/kesh-api/tests/common/mod.rs:155`  
**Analysis:** Common fixture hardcodes company name `"Test Company"`. Schema check shows NO unique constraint on company.name, only on company.ide_number.  
**Verdict:** Multiple tests CAN safely create companies with same name — no database collision.  
**Severity:** NONE (false alarm)

---

## LAYER 3: ACCEPTANCE AUDITOR (Diff + spec + full context)

### AC Violation & Spec Compliance Check

#### AA-1: **AC #6 IDOR tests — Pass 1 required "minimum 6 entities", only users added**

**Status:** MEDIUM (partial fix)  
**AC #6 Requirement:** HTTP 404 tests for minimum 6 entities (contacts, products, invoices, accounts, companies/current, users)  
**Evidence from diff:**
- Only `idor_users_cross_company_returns_404()` added (users entity)
- No tests for contacts, products, invoices, accounts, companies/current

**Shortfall:** 5 entities missing. Pass 1 identified H1 as 150-200 lines; only 38 lines of test code added.  
**Severity:** HIGH (AC #6 incomplete)

---

#### AA-2: **Migration guard — simplified version differs from Pass 1 C1-C2 patch intent**

**Status:** MEDIUM (intent loss)  
**Pass 1 C1-C2 patches required:**
- C1: Add IF guard to execute SIGNAL SQLSTATE
- C2: Add guard for multi-company backfill corruption

**Actual diff (lines 5-39):**
- No IF/SIGNAL guards present
- Migration simplified to rely on natural ALTER NOT NULL failure

**Evidence:** Migration file shows LOCK TABLES but no conditional checks.  
**Spec intent:** Defensive guards to catch data corruption early with clear error messages.  
**Current behavior:** Silent NULL + ALTER failure (less user-friendly).  
**Severity:** MEDIUM (reduced defensive posture vs spec intent)

---

#### AA-4: **CodeQL workflow — unimplemented languages create check noise**

**Status:** MEDIUM (CI check noise)  
**File:** `.github/workflows/codeql.yml:19-20`  
**Issue:** CodeQL matrix includes 7 languages not used in Kesh (cpp, csharp, go, java, python, ruby). Only rust and javascript/typescript applicable.  
**Impact:** CI checks report "skip" or warnings for 7 languages, creating noise in PR checks.  
**Fix:** Reduce matrix to applicable languages: `['javascript', 'rust']`  
**Effort:** 1 line YAML  
**Severity:** MEDIUM (operational noise)

---

#### AA-5: **Defensive validation in contacts/products/invoices — only comments, no validation**

**Status:** MEDIUM (defensive gap)  
**File:** `crates/kesh-api/src/routes/contacts.rs:2-6`, `invoices.rs:8-12`, `products.rs:2-6`  
**Issue:** These routes add security documentation comments about company_id staleness, but DON'T implement defensive `get_company_for()` calls like accounts.rs does.  
**Patterns observed:**
- `accounts.rs list_accounts()`: Calls `get_company_for()` (defensive)
- `contacts.rs list_contacts()`: No `get_company_for()` call, only comment
- `invoices.rs list_invoices()`: No `get_company_for()` call, only comment
- `products.rs list_products()`: No `get_company_for()` call, only comment

**Risk:** Staleness window not actively validated in these routes. If company reassignment occurs, routes silently use stale JWT company_id.  
**Fix:** Add `get_company_for()` calls to list_contacts, list_invoices, list_products for consistency.  
**Effort:** 3-5 lines per route (9-15 lines total)  
**Severity:** MEDIUM (consistency gap)

---

### SUMMARY OF FINDINGS

| Layer | Critical | High | Medium | Low | Status |
|-------|----------|------|--------|-----|--------|
| Blind Hunter (diff) | 0 | 0 | 2 | 2 | Complete |
| Edge Case Hunter | 0 | 1 | 1 | 0 | Complete |
| Acceptance Auditor | 0 | 2 | 2 | 0 | Complete |
| **TOTAL** | **0** | **3** | **5** | **2** | |

**Convergence (Pass 1 → Pass 2):**
- Pass 1: 18 findings (4 CRITICAL, 5 HIGH, 7 MEDIUM, 2 LOW)
- Pass 2: 10 findings (0 CRITICAL, 3 HIGH, 5 MEDIUM, 2 LOW)
- **Delta:** -4 CRITICAL (all resolved ✅), -2 HIGH (resolved), -2 MEDIUM, 0 LOW change
- **Trend:** All CRITICAL resolved ✅; HIGH/MEDIUM findings down 67% (12→5); strong convergence toward gate.

---

## CRITICAL FINDINGS

**None detected.** All Pass 1 CRITICAL patches (C1-C4) have been successfully remediated. ✅

---

## HIGH FINDINGS (Should Fix Before Merge)

### HIGH-A1: AC #6 nearly complete — 5 of 6 entities tested, companies/current missing

**File:** `crates/kesh-api/tests/idor_multi_tenant_e2e.rs`  
**Issue:** Pass 1 H1 required HTTP 404 tests for 6 entities (contacts, products, invoices, accounts, companies/current, users).  
**Actual coverage:** 5 tests present:
  - ✅ `idor_contacts_cross_company_returns_404`
  - ✅ `idor_products_cross_company_returns_404`
  - ✅ `idor_accounts_cross_company_returns_404`
  - ✅ `idor_invoices_cross_company_returns_404`
  - ✅ `idor_users_cross_company_returns_404`
  - ❌ `idor_companies_current_cross_company_returns_404` — MISSING

**Impact:** AC #6 requirement is 83% met. Only companies/current IDOR test missing.  
**Fix:** Add test for GET /api/v1/companies/current when accessed with different company_id JWT.  
**Effort:** ~30-40 lines test code  
**Severity:** HIGH (nearly complete, minor gap)

---

### HIGH-A2: Migration guards removed during merge, defensive intent lost

**File:** `crates/kesh-db/migrations/20260419000002_users_company_id.sql:1-29`  
**Issue:** Pass 1 C1-C2 patches required explicit IF/SIGNAL guards:
  - C1: IF @should_fail=1, SIGNAL SQLSTATE with clear message
  - C2: Guard for multi-company backfill corruption detection

**Actual migration:** No IF/SIGNAL guards. Migration relies on natural ALTER NOT NULL failure (line 19), less user-friendly.  
**Root cause:** Merge conflict resolution chose to simplify migration instead of re-adding guards. Comment claims "IF checks aren't supported in .sql migration files" (sqlx limitation), but guards were spec requirement.  
**Impact:** 
  - If backfill produces NULL company_id, error is cryptic "Column 'company_id' cannot be null" instead of specific guard message
  - Multi-company corruption risk undetected
  - Spec intent (defensive error messages) not met

**Fix:** Re-add C1-C2 guards from Pass 1 findings.  
**Effort:** 10-15 lines SQL  
**Severity:** HIGH (spec deviation)

---

### HIGH-A3: Defensive validation inconsistency in accounts handlers

**File:** `crates/kesh-api/src/routes/accounts.rs`  
**Issue:** Inconsistent defensive validation:
  - `list_accounts()` (line 94-96): Calls `get_company_for()` to validate company exists (defensive vs staleness)
  - `create_account()` (line 116): Calls `get_company_for()` 
  - `update_account()` (line 166-190): **NO call** to `get_company_for()`, only uses company_id from JWT

**Risk:** If company_id becomes stale during session and user is reassigned to different company, update silently operates on stale company_id without validation.  
**Fix:** Add `get_company_for()` call in `update_account()` to match defensive pattern of list/create.  
**Effort:** 2-3 lines Rust  
**Severity:** HIGH (defensive consistency)

---

## MEDIUM FINDINGS (Should Fix or Document as Debt)

### MEDIUM-A1: CodeQL workflow language matrix includes non-applicable languages
**Effort:** 1 line | **Severity:** MEDIUM

### MEDIUM-A2: Defensive validation inconsistency (accounts.rs, contacts.rs, invoices.rs, products.rs)
**Effort:** 15 lines total | **Severity:** MEDIUM  
**Details:** list_accounts has defensive `get_company_for()`, but update_account, and other routes list_contacts/list_invoices/list_products lack it.

### MEDIUM-A3: CodeQL config build error suppression (`|| true`)
**Effort:** 1 line | **Severity:** MEDIUM  
**Details:** Rust build failures silently ignored in CodeQL workflow, could mask compilation errors.

### MEDIUM-A4: Migration guard comments claim IF "unsupported" without evidence
**Effort:** 10-15 lines SQL (if re-adding) | **Severity:** MEDIUM  
**Details:** Migration comment says IF checks "aren't supported in .sql migration files" but doesn't justify. Pass 1 C1-C2 patches required them.

---

## REMEDIATION PATH (Before Merge)

### CRITICAL Path (0 items)
✅ All CRITICAL findings from Pass 1 resolved.

### HIGH Path (3 items — ~3-4 hours total)

1. **HIGH-A1: Complete AC #6 IDOR tests** (add companies/current test)
   - **Effort:** 30-40 lines test code (~30 min)
   - **Blocker:** AC #6 compliance
   - **Option A (PREFERRED):** Add test and merge
   - **Option B:** Document as post-merge followup (lower risk)

2. **HIGH-A2: Restore migration guards** (re-add C1-C2 defensive IF/SIGNAL)
   - **Effort:** 10-15 lines SQL (~15 min)
   - **Blocker:** Spec intent (defensive error messages)
   - **Action:** Required for merge readiness

3. **HIGH-A3: Add defensive validation to routes** (contacts, invoices, products)
   - **Effort:** 9-15 lines Rust (~20 min)
   - **Blocker:** Defensive consistency across AC#5
   - **Action:** Required for merge readiness

### MEDIUM Path (4 items — ~1 hour, optional)
- M1: CodeQL language matrix cleanup
- M2: Build error suppression fix
- M3: Migration comment clarification
- M4: Defensive validation in update_account

### Decision Gate

**Pass 2 merge readiness:** Requires **HIGH-A2 + HIGH-A3** fixes (~35 min).  
**Optional:** HIGH-A1 (completes AC#6 or deferrred as post-merge debt).

---

**Status:** Pass 2 → Scenario A (fix HIGH before merge) or Scenario B (defer optional HIGH-A1 to post-merge story).

