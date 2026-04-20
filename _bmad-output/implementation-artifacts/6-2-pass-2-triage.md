---
title: Story 6.2 Pass 2 — Triage Report
date: 2026-04-19
pass: 2
findings_total: 10
findings_reject: 0
status: READY_FOR_PRESENTATION
---

# Pass 2 Triage Report — Story 6.2

**Findings normalized:** 10 issues (0 CRITICAL, 3 HIGH, 5 MEDIUM, 2 LOW)  
**Duplicates resolved:** 0  
**False positives dropped:** 0  
**Actionable findings:** 10

---

## FINDINGS BY CLASSIFICATION

### HIGH (3 items — ~3.5 hours effort, blocking merge)

#### [H1] AC #6 incomplete — 5 of 6 IDOR entities tested
- **ID:** HIGH-A1
- **Source:** Acceptance Auditor (AC coverage check)
- **Classification:** `patch` (code addition)
- **Location:** `crates/kesh-api/tests/idor_multi_tenant_e2e.rs`
- **Detail:** Pass 1 H1 required HTTP 404 tests for 6 entities. Actual coverage: 5/6.
  - ✅ contacts, products, accounts, invoices, users
  - ❌ companies/current (missing)
- **Impact:** AC #6 requirement is 83% complete; missing one entity test.
- **Fix:** Add HTTP 404 test for GET /api/v1/companies/current cross-company scenario.
- **Effort:** 30-40 lines test code (~30 min)
- **Merge blocker:** YES (AC #6 compliance) — OR defer as post-merge story

#### [H2] Migration guards removed — defensive intent lost
- **ID:** HIGH-A2
- **Source:** Acceptance Auditor (spec deviation check)
- **Classification:** `patch` (code restoration)
- **Location:** `crates/kesh-db/migrations/20260419000002_users_company_id.sql:1-29`
- **Detail:** Pass 1 C1-C2 patches required explicit IF/SIGNAL guards for:
  - Data integrity check (users without companies)
  - Multi-company backfill corruption detection
  
  Merge conflict resolution removed guards, relying on natural ALTER NOT NULL failure. Less user-friendly error messages.
- **Impact:** Spec intent (defensive error messages) not met; operational debugging harder.
- **Fix:** Re-add guards from Pass 1 C1-C2:
  ```sql
  IF @should_fail = 1 THEN
    SIGNAL SQLSTATE '45000' SET MESSAGE_TEXT = @error_msg;
  END IF;
  ```
- **Effort:** 10-15 lines SQL (~15 min)
- **Merge blocker:** YES (spec compliance)

#### [H3] Defensive validation inconsistency — some routes, not others
- **ID:** HIGH-A3
- **Source:** Acceptance Auditor (AC #5 consistency check)
- **Classification:** `patch` (code addition)
- **Location:** `crates/kesh-api/src/routes/accounts.rs:166-190`, `contacts.rs`, `invoices.rs`, `products.rs`
- **Detail:** AC #5 routes should use `get_company_for()` for defensive staleness validation:
  - ✅ accounts.rs list_accounts() — has `get_company_for()`
  - ✅ accounts.rs create_account() — has `get_company_for()`
  - ❌ accounts.rs update_account() — NO defensive call
  - ❌ contacts.rs list_contacts() — only comment, no call
  - ❌ invoices.rs list_invoices() — only comment, no call
  - ❌ products.rs list_products() — only comment, no call

  If company_id becomes stale during session, these routes use stale JWT value without validation.
- **Impact:** Inconsistent defensive depth; staleness window not actively validated.
- **Fix:** Add `let _ = get_company_for(&current_user, &state.pool).await?;` to:
  - accounts.rs update_account()
  - contacts.rs list_contacts()
  - invoices.rs list_invoices()
  - products.rs list_products()
- **Effort:** 2-3 lines per route (9-12 lines total, ~20 min)
- **Merge blocker:** YES (defensive consistency in AC #5)

---

### MEDIUM (5 items — ~1-1.5 hours effort, nice-to-have)

#### [M1] CodeQL workflow language matrix includes 7 non-applicable languages
- **ID:** MEDIUM-A1 (BH-1 + AA-4 merged)
- **Source:** Blind Hunter (diff anomaly) + Acceptance Auditor (check noise)
- **Classification:** `patch` (configuration cleanup)
- **Location:** `.github/workflows/codeql.yml:19-20`
- **Detail:** Matrix includes `cpp, csharp, go, java, python, ruby` languages not present in Kesh repo. Only Rust and JavaScript/TypeScript applicable.
- **Impact:** CI checks report "skip" for 7 languages, creating noise. No functional impact.
- **Fix:** Reduce matrix to `['javascript', 'rust']`
- **Effort:** 1 line YAML (~2 min)

#### [M2] CodeQL Rust build error suppression with `|| true`
- **ID:** MEDIUM-A2 (BH-2)
- **Source:** Blind Hunter (diff pattern detection)
- **Classification:** `patch` (error handling)
- **Location:** `.github/workflows/codeql.yml:63-65`
- **Detail:** Rust build errors silently ignored: `cargo build --release 2>&1 || true`. If patches broke compilation, error won't surface until later step.
- **Impact:** Mask build failures; may hide real compilation errors introduced by patches.
- **Fix:** Remove `|| true` or handle failure explicitly:
  ```yaml
  cargo build --release 2>&1 || echo "Build failed; CodeQL will analyze what it can"
  ```
- **Effort:** 1 line YAML (~2 min)

#### [M3] Migration comment claims IF "unsupported" without justification
- **ID:** MEDIUM-A3 (from EH-2)
- **Source:** Edge Case Hunter (design rationale check)
- **Classification:** `patch` (documentation clarity)
- **Location:** `crates/kesh-db/migrations/20260419000002_users_company_id.sql:16`
- **Detail:** Comment says "explicit IF checks which aren't supported in .sql migration files" but:
  - Doesn't provide evidence for unsupported claim
  - MariaDB/MySQL supports IF in .sql files
  - Claim may refer to sqlx migration runner limitation (not documented)
  - Pass 1 C1-C2 patches required IF guards

  Either re-add guards (if sqlx supports) or clarify the limitation.
- **Impact:** Confusing comment; unclear why guards were removed.
- **Fix:** Either re-add guards (HIGH-A2 item) or update comment with evidence/reference.
- **Effort:** 1-2 lines clarification OR 10-15 lines guard restoration (~5-15 min)

#### [M4] Defensive validation in update_account missing
- **ID:** MEDIUM-A4 (part of HIGH-A3, split as separate item)
- **Source:** Acceptance Auditor (consistency check)
- **Classification:** `patch` (code addition, already covered in HIGH-A3)
- **Note:** Merged into HIGH-A3 remediation.

#### [M5] routes/contacts, invoices, products add security notes but no defensive validation
- **ID:** MEDIUM-A5 (AA-5)
- **Source:** Acceptance Auditor (pattern consistency)
- **Classification:** `patch` (already covered in HIGH-A3)
- **Location:** `crates/kesh-api/src/routes/contacts.rs:2-6`, `invoices.rs:8-12`, `products.rs:2-6`
- **Detail:** Routes add documentation about company_id staleness window but don't implement defensive `get_company_for()` calls like accounts.rs does.
- **Note:** Merged into HIGH-A3 remediation (list_contacts, list_invoices, list_products).

---

### LOW (2 items — optional, ~30 min effort)

#### [L1] CodeQL initialization runs for all languages before paths-ignore known
- **ID:** LOW-A1 (BH-3)
- **Source:** Blind Hunter (workflow structure)
- **Classification:** `reject` (cosmetic, harmless)
- **Detail:** CodeQL `initialize` step runs for all languages before `paths-ignore` is available. Test directories indexed unnecessarily, but `paths-ignore` still filters analysis results.
- **Impact:** Cosmetic. Test files processed but excluded from final analysis.
- **Verdict:** No action needed; skip.

#### [L2] IDOR test naming doesn't indicate role dependency
- **ID:** LOW-A2 (BH-5 modified after testing)
- **Source:** Blind Hunter (test clarity)
- **Classification:** `reject` (test passes, no fix needed)
- **Detail:** Test `idor_users_cross_company_returns_404` creates user_b with Role::Admin. If role validation order changes, test might pass for wrong reason. However, test is verified to pass correctly.
- **Verdict:** Test is valid. No change needed.

---

## DEDUPLICATION SUMMARY

**Initial findings:** 10 items  
**Merged into fewer items:** -2 (AA-5 merged into HIGH-A3; M4 merged into HIGH-A3)  
**Net unique issues:** 8 actionable + 2 low-priority rejects = 10 total

---

## CLASSIFICATION SUMMARY

| Category | Count | Effort | Blocker | Action |
|----------|-------|--------|---------|--------|
| HIGH | 3 | 3.5 hours | YES | Fix before merge |
| MEDIUM | 5 | 1-1.5 hours | NO | Fix before merge or defer |
| LOW | 2 | — | NO | Reject (no action) |
| **TOTAL** | **10** | **~4-5 hours** | **3 blockers** | |

---

## MERGE GATE DECISION

**Status:** Pass 2 validation identifies **3 HIGH blockers** (~3.5 hours remediation).

**Scenarios:**
- **Scenario A (FIX ALL HIGH):** Complete HIGH-A1, HIGH-A2, HIGH-A3 before merge (~3.5 hours). Result: READY FOR MERGE ✅
- **Scenario B (DEFER HIGH-A1):** Fix HIGH-A2, HIGH-A3 (~35 min) + defer HIGH-A1 (AC #6 incomplete) to post-merge story. Result: MERGE WITH DEBT 📋

**Recommendation:** Scenario A preferred (completes AC #6), but Scenario B acceptable if time-constrained.

---

**Pass 2 complete.** Proceeding to presentation phase.

