---
stepsCompleted: []
validationPass: 1
passDate: 2026-04-20
validatorModel: claude-haiku-4-5-20251001
storyFile: 6-5-fix-playwright-e2e-auth-flow.md
---

# Story 6-5 Validation Pass 1 — Findings

## Summary

**Story:** 6-5 - Fix Playwright E2E auth flow (KF-007)
**Status:** ready-for-dev
**Validation Date:** 2026-04-20
**Finding Count:** 3 CRITICAL, 4 HIGH, 3 MEDIUM

## 🚨 CRITICAL ISSUES (Must Fix Before Dev)

### 1. AC #6 References Archived Documentation  
**Severity:** CRITICAL  
**Location:** Acceptance Criteria #6  
**Issue:** AC #6 instructs developer to update `docs/known-failures.md` with KF-007 status. However, per CLAUDE.md (line 25):
- `docs/known-failures.md` archived 2026-04-18
- **No new KF entries should be added to this file**
- All KF tracking moved to GitHub Issues

**Current Text:**
```
AC #6 — KF-007 fermée dans `docs/known-failures.md`
```

**Problem:** Developer following this AC literally will edit an archived file instead of closing the GitHub issue.

**Fix Required:** 
- Replace AC #6 to reference GitHub issue #19 closure instead
- Change to: "AC #6 — GitHub issue #19 fermée avec commit message `(closes #19)`"
- Mention this closes the tracked KF-007 via GitHub, not docs file

---

### 2. Missing Test Spec File Paths & Pass/Fail Status  
**Severity:** CRITICAL  
**Location:** Story Context, Developer Context  
**Issue:** Story mentions "~12 specs" and "60–80 tests" but:
- No explicit file paths for test specs (e.g., `frontend/tests/e2e/accounts.spec.ts`)
- Lists "auth.spec.ts 100% OK" and "onboarding.spec.ts OK" without full list
- Lists "9 admin specs" in Testing Strategy but doesn't enumerate them
- Missing clear distinction: **which of the 9+ specs ARE FAILING?**

**Consequence:** Developer must guess which files to focus on, wasting time exploring.

**Fix Required:**
Add new subsection under "Developer Context → Playwright Seeding":
```
**Test Spec Inventory:**
- Status: PASSING
  - auth.spec.ts (4 tests: login, logout, axe)
  - onboarding.spec.ts (3+ tests: fresh start flow)
  
- Status: FAILING (~60 tests)
  - accounts.spec.ts (8 tests)
  - contacts.spec.ts (8 tests)
  - products.spec.ts (8 tests)
  - invoices.spec.ts (8 tests)
  - journal-entries.spec.ts (8 tests)
  - users.spec.ts (8 tests)
  - homepage-settings.spec.ts (4 tests)
  - mode-expert.spec.ts (2 tests)
  - invoices-echeancier.spec.ts (4 tests)
  
- Status: UNKNOWN/PARTIAL
  - onboarding-path-b.spec.ts (verify status)
```

---

### 3. Missing localStorage Key Name Validation  
**Severity:** CRITICAL  
**Location:** Investigation Guide, Step 2  
**Issue:** Story assumes localStorage key is `accessToken` but doesn't verify:
- What key is actually used in `frontend/src/lib/auth.ts`?
- What if it's `token`, `jwt`, `auth_token`, or something else?
- This assumption appears in Step 2 console command and Step 4 post-login navigation

**Consequence:** Developer runs `localStorage.getItem('accessToken')` and gets null even though token exists under different key.

**Fix Required:**
Add verification step in Investigation Guide Step 1 (Local Setup):
```
# Terminal 2b — Verify localStorage Key
cd frontend/src/lib
grep -n "localStorage.setItem" auth.ts
# Note exact key name (likely 'accessToken' but verify)
```

Then update Step 2 to say:
```
3. Switch to Application tab → localStorage — verify the EXACT KEY used by frontend
   (based on auth.ts, expected key is 'accessToken' — confirm this step)
```

---

## 🔴 HIGH ISSUES (Should Fix Before Dev)

### 4. Likely Suspects Lacks Priority Ordering  
**Severity:** HIGH  
**Location:** Investigation Guide, Step 5  
**Issue:** Lists 5 possible root causes (Frontend auth.ts, SvelteKit Load, SvelteKit Hooks, Playwright Context, Backend JWT) without priority. Each has ~50% chance of being correct, but they should be investigated in **likelihood order**.

**Consequence:** Developer spends time checking Backend JWT validation first when Frontend auth.ts is the most likely culprit.

**Fix Required:**
Reorder suspects by likelihood & effort:
```
**Step 5 (Reordered): Likely Suspects (Investigate in Order)**

**FIRST (highest probability):**
1. **Frontend auth.ts** — is `authStore`/token properly synced to localStorage?
   - localStorage.setItem called after login?
   - fetch wrapper sending Authorization header?
   
**SECOND:**
2. **SvelteKit Hooks** (hooks.client.ts or hooks.server.ts) — any redirect logic interfering?

**THIRD:**
3. **SvelteKit Load Functions** (+page.server.ts) — early redirect to /login?

**FOURTH:**
4. **Playwright Context** — localStorage truly persistent across page.goto()?

**FIFTH (least likely):**
5. **Backend JWT Validation** — Authorization header parsing correct?
```

---

### 5. Missing Playwright & Browser Version Requirements  
**Severity:** HIGH  
**Location:** Developer Context, Testing Strategy  
**Issue:** No mention of:
- Playwright version (assumed from package.json but not stated)
- Browser version used (Chromium? Firefox? WebKit?)
- Any known Playwright timing issues with this version

**Consequence:** If local Playwright differs from CI, developer might get different results.

**Fix Required:**
Add to Developer Context subsection:
```
**Playwright Configuration Requirements:**
- Playwright version: Check `frontend/package.json` for exact version
- Recommended browser: Chromium (same as CI)
- Timeout: Default 30s (may need adjustment if network slow)
- Workers: 1 (already set, see playwright.config.ts)
```

---

### 6. AC #6 Violation (Duplicate of CRITICAL #1)  
**Severity:** HIGH (already noted above as CRITICAL)  
Documented in CRITICAL #1 — AC #6 References Archived Documentation

---

### 7. Missing Test Comparison Assertion  
**Severity:** HIGH  
**Location:** Testing Strategy, Post-Fix Validation  
**Issue:** Lists tests to run post-fix but doesn't mention:
- What should NOT change (e.g., auth.spec.ts should still pass)
- How to verify no regressions in other auth flows
- Explicit assertion: "Auth-spec, onboarding-spec should STILL PASS"

**Consequence:** Developer might fix accounts.spec but break auth.spec, thinking it's ok.

**Fix Required:**
Add to Testing Strategy:
```
**Regression Prevention:**
- auth.spec.ts MUST still pass (no auth regression)
- onboarding.spec.ts MUST still pass (no onboarding regression)
- No changes to frontend/src/lib/auth.ts logic (only bug fixes, not refactors)
```

---

## 🟡 MEDIUM ISSUES (Nice to Have)

### 8. Investigation Guide Step 3 Ambiguous  
**Severity:** MEDIUM  
**Location:** Investigation Guide, Step 3  
**Issue:** "add pause: `await page.pause();`" — what happens when paused?

Current text:
```
In Playwright test, after login assertion, add pause: `await page.pause();`
```

This assumes developer knows what to do next. Should explain the workflow.

**Fix:** Expand to:
```
In Playwright test, after login assertion, add debug pause:
\`\`\`javascript
// After login assertion, add:
await page.pause(); // Playwright stops here; DevTools opens for manual inspection
\`\`\`
When paused, use browser DevTools to manually:
1. Open Application tab → localStorage
2. Check Network tab for Authorization headers
3. Manually navigate to /accounts and observe behavior
```

---

### 9. Step 4 Missing Exact File Path References  
**Severity:** MEDIUM  
**Location:** Investigation Guide, Step 4  
**Issue:** "Check source: `frontend/tests/e2e/auth.spec.ts`" — good. But for load functions:

Current:
```
Check source : `frontend/tests/e2e/auth.spec.ts`
Check source : `frontend/src/routes/+page.server.ts`
```

Should include:
- Exact path to passing test that uses authenticated navigation
- Example file paths for accounts, products routes

**Fix:** List exact paths:
```
**Passing tests to reference:**
- frontend/tests/e2e/auth.spec.ts (login only, no post-auth navigation)
- frontend/tests/e2e/onboarding.spec.ts (onboarding flow, check if it navigates to /accounts or stays in /onboarding)

**Load functions to check:**
- frontend/src/routes/accounts/+page.server.ts
- frontend/src/routes/contacts/+page.server.ts
- frontend/src/routes/(auth)/+layout.server.ts (if parent layout has auth check)
```

---

### 10. Root Cause Hypotheticals Missing Context  
**Severity:** MEDIUM  
**Location:** Context section, "Root cause hypothétiques"  
**Issue:** Lists 4 hypotheticals but doesn't mention:
- What error is actually being observed (timeout? 401 Unauthorized? Redirect loop?)
- Does localStorage exist but is empty, or is it not being set at all?

**Consequence:** Developer must infer from symptoms what to investigate.

**Fix:** Update Context to clarify observed behavior:
```
### Observed Symptoms
- POST /api/v1/auth/login returns 200 ✅ (token in response body)
- localStorage contains accessToken ✅ (manually verified in DevTools)
- GET /accounts → 401 Unauthorized OR redirect to /login (unclear which)
- Hypothesis: JWT is set in localStorage but not being sent in Authorization header
```

---

## 📊 Findings Summary

| Severity | Count | Actionable | Blocker? |
|----------|-------|-----------|----------|
| CRITICAL | 3 | Yes | **YES** |
| HIGH | 4 | Yes | **Partially** |
| MEDIUM | 3 | Yes | No |
| **TOTAL** | **10** | **All** | **Yes** |

## 🎯 Recommended Action

**BLOCK DEVELOPMENT** until CRITICAL issues are resolved:

1. ✏️ **Fix AC #6** — Update to reference GitHub issue #19 closure
2. ✏️ **Add Test Spec Inventory** — Explicit list of passing vs failing specs
3. ✏️ **Add localStorage Key Verification** — Step in Investigation Guide

Then address HIGH issues in priority order:

4. ✏️ **Prioritize Likely Suspects** — Reorder by investigation likelihood
5. ✏️ **Add Playwright/Browser Version** — Developer Context section
6. ✏️ **Add Regression Prevention** — Testing Strategy section

MEDIUM issues can be addressed concurrently or post-fix.

---

**Next Step:** Apply fixes and run **Validation Pass 2** (Sonnet model) to verify improvements.

