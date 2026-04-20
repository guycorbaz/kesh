---
stepsCompleted: []
validationPass: 2
passDate: 2026-04-20
validatorModel: claude-haiku-4-5-20251001-fresh-context
storyFile: 6-5-fix-playwright-e2e-auth-flow.md (post-Pass-1-fixes)
priorFindingCount: 10
---

# Story 6-5 Validation Pass 2 — Findings

## Summary

**Story:** 6-5 - Fix Playwright E2E auth flow (KF-007)
**Validation Date:** 2026-04-20 (Fresh Context)
**Finding Count:** 1 CRITICAL (NEW), 2 HIGH (CARRY-OVER), 3 MEDIUM

**Confidence:** Pass 1 fixes applied correctly. However, 1 CRITICAL regression introduced + 2 HIGH items not addressed.

---

## 🚨 CRITICAL ISSUES (New — Must Fix)

### 1. SUCCESS METRICS CONTRADICTS AC #6 (REGRESSION)  
**Severity:** CRITICAL  
**Location:** Success Metrics section, last bullet  
**Issue:** Pass 1 correctly updated AC #6 to reference GitHub issue #19 closure. However, **Success Metrics still references the ARCHIVED docs/known-failures.md**:

Current Text (Line 296):
```
✅ **KF-007 Closed** : `docs/known-failures.md` updated with fix explanation
```

**The Problem:**
- AC #6 explicitly says: NO edit to `docs/known-failures.md` (archived since 2026-04-18)
- AC #6 says: Close GitHub issue #19 via commit message
- Success Metrics contradicts AC #6
- Developer reads Success Metrics and tries to edit archived file, wasting time

**This is a REGRESSION:** Pass 1 fixed AC #6 but forgot to update Success Metrics.

**Fix Required:**
Replace Success Metrics bullet:
```
✅ **KF-007 Closed** : GitHub issue #19 closed via commit message (closes #19), no edits to archived docs files
```

---

## 🔴 HIGH ISSUES (Carry-Over From Pass 1)

### 2. Step 6 "FINDINGS.md" Location Undefined  
**Severity:** HIGH  
**Location:** Investigation Guide, Step 6  
**Issue:** Step 6 says "Write a brief `FINDINGS.md` explaining what's broken" but:
- No location specified (root? frontend/? DEBUGGING-KF007.md mentioned in AC #2?)
- No format/template provided
- AC #2 says "document in `frontend/DEBUGGING-KF007.md`" but Step 6 says "FINDINGS.md"

**Consequence:** Developer unsure where/how to document finding, creates wrong filename or location.

**Fix Required:**
Update Step 6 to reference AC #2:
```
Once root cause is clear, **before coding**:
- Document finding in `frontend/DEBUGGING-KF007.md` (per AC #2) or as inline code comments
- Explain what's broken and why (e.g., "localStorage key not sent in header because fetch wrapper missing")
- Propose the fix with confidence level (1-2 line fix? Larger refactor?)

Then code the fix and validate locally.
```

---

### 3. Conditional Guidance Missing for localStorage Key Verification  
**Severity:** HIGH  
**Location:** Step 1 & Step 2  
**Issue:** Step 1 Terminal 2 says "verify the actual key name used" but **Step 2 doesn't handle the case where key ≠ 'accessToken'**

Current Step 2 instruction (Line 210):
```
run `localStorage.getItem('<KEY_FROM_STEP_1>')` manually
```

**Problem:** If Step 1 finds key is `token` or `auth` instead of `accessToken`, what happens? Step 2 uses placeholder `<KEY_FROM_STEP_1>` but doesn't explain HOW to substitute it or what to do if key doesn't exist.

**Consequence:** Developer might run step 2 literally with `<KEY_FROM_STEP_1>` as the string instead of the actual key name.

**Fix Required:**
Add to Step 1 outcome:
```
# Step 1 Outcome
Confirm the actual localStorage key (note it for Steps 2-5).
Example: if grep returns:
  localStorage.setItem('accessToken', token)
Then in subsequent steps, use 'accessToken' (not 'auth', 'jwt', etc.)
```

Then update Step 2 to be more explicit:
```
4. Console — run `localStorage.getItem('<ACTUAL_KEY_NAME>')` manually
   Example: `localStorage.getItem('accessToken')` — should return JWT string
   If returns null or undefined — token not persisted, STOP and investigate Step 1 logic
```

---

## 🟡 MEDIUM ISSUES (Carry-Over + New)

### 4. Step 3 Post-Pause Workflow Still Ambiguous  
**Severity:** MEDIUM  
**Location:** Investigation Guide, Step 3  
**Issue:** (Carry-over from Pass 1) Step 3 says "add pause: `await page.pause();`" but doesn't explain what to do once paused.

Current (Line 213):
```
1. In Playwright test, after login assertion, add pause: `await page.pause();`
2. DevTools → Network tab, clear + reload
```

**Ambiguity:** After pause, does Playwright stay paused? Does test continue? What's the intended workflow?

**Suggested Fix (optional, MEDIUM priority):**
```
1. In Playwright test, after successful login, add debugging pause:
   await page.pause(); // Playwright halts here; DevTools opens in browser
2. While paused, inspect DevTools:
   - Application tab → localStorage: verify token persisted
   - Network tab → clear, then manually navigate to /accounts
   - Observe: Is Authorization header present? What HTTP status?
3. Resume test by closing DevTools or clicking Play in Playwright UI
```

---

### 5. Terminal Numbering Clarity (MEDIUM)  
**Severity:** MEDIUM  
**Location:** Step 1 Local Setup  
**Issue:** Terminal numbering is "1", "2b", "3" which is slightly confusing. Terminal "2b" suggests a secondary terminal that's different from main workflow.

Current:
```
# Terminal 1 — Backend
# Terminal 2 — Verify localStorage Key
# Terminal 3 — Frontend E2E Tests
```

**Better:**
```
# Terminal A — Backend
# Terminal B — Verify localStorage Key  
# Terminal C — Frontend E2E Tests
```

Or:
```
# Terminal 1 — Backend (keep running)
# Terminal 2 — Frontend E2E Tests
# (Before running Terminal 2, run once: cd frontend/src/lib && grep -n "localStorage.setItem" auth.ts)
```

---

### 6. Test Spec Inventory Lacks Test Order/Priority  
**Severity:** MEDIUM  
**Location:** Test Spec Inventory section  
**Issue:** Inventory lists 10 test files but no indication of which to test first/most important. Developer might run all 60 tests at once when debugging might be faster with a single failing spec.

**Suggested Enhancement (optional):**
```
*Recommended test order (start with single failing spec):*
1. `accounts.spec.ts` (start here — simplest post-auth navigation)
2. `contacts.spec.ts` (similar pattern)
3. Others... (once accounts fixed, verify pattern applies to all)
```

---

## 📊 Validation Pass 2 Summary

| Severity | Count | Status | Blocking? |
|----------|-------|--------|-----------|
| CRITICAL | 1 | NEW (Regression) | **YES** |
| HIGH | 2 | CARRY-OVER | **Partially** |
| MEDIUM | 3 | CARRY-OVER/NEW | No |
| **TOTAL** | **6** | Mix | **1 YES** |

---

## 🎯 Verdict

**STORY STATUS: BLOCKED** (1 CRITICAL regression)

The 1 NEW CRITICAL issue (Success Metrics contradiction) **MUST be fixed before dev** to prevent developer confusion.

The 2 HIGH carry-overs from Pass 1 should be addressed — they impact developer clarity.

**Action Required:**
1. ✏️ **FIX CRITICAL** — Update Success Metrics line 296 to match AC #6 (GitHub issue closure, not docs file edit)
2. ✏️ **ENHANCE HIGH** — Add FINDINGS.md location clarification + localStorage key conditional handling

Once fixes applied → **Story ready for Validation Pass 3 (Opus)** to verify convergence to 0 CRITICAL/HIGH findings.

---

**Pass Trend:** 10 findings (Pass 1) → 6 findings (Pass 2) → ??? (Pass 3)

Target: 0 CRITICAL/HIGH + only MEDIUM findings remaining (acceptable for dev).

