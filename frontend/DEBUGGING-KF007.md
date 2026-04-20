# KF-007 Investigation & Root Cause Analysis

**Date:** 2026-04-20  
**Story:** 6-5 - Fix Playwright E2E auth flow  
**Issue:** GitHub #19 — E2E tests fail with persistent `/login` redirect after successful login

## Investigation Process

### Symptom Reproduced Locally

**Setup:**
```bash
# Terminal 1: Backend
export KESH_TEST_MODE=true
export KESH_HOST=127.0.0.1
export DATABASE_URL=mysql://root:kesh_root@127.0.0.1:3306/kesh
cargo run -p kesh-api

# Terminal 2: E2E Tests
cd frontend
npm run test:e2e -- --debug accounts.spec.ts
```

**Observed Behavior:**
1. Login form: POST `/api/v1/auth/login` → 200 OK, response contains `accessToken` + `refreshToken`
2. DevTools Application tab → localStorage: **NO `kesh:auth:*` keys found**
3. Test attempts navigation to `/accounts` → Redirected to `/login` (401 Unauthorized)
4. Authorization header inspection: Not present in subsequent API calls

### Root Cause Analysis

**Root Cause:** JWT tokens were stored in **in-memory Svelte state only**, not persisted to browser localStorage.

**Why This Breaks Playwright:**
- Playwright test flow: `page.goto('/login')` → submit form → backend returns token → JS stores in `_accessToken` variable
- Test navigates: `page.goto('/accounts')` → **full page reload**
- On page reload, **JavaScript state is cleared**, but `_accessToken` variable is lost
- SvelteKit load functions check `authState.isAuthenticated` → false (state cleared)
- Load function redirects to `/login` → 401 Unauthorized

**Why Pre-Existing Bug:** This is not a new regression (pre-Story 6-4). All E2E tests were failing with the same pattern before Story 6-4 fixtures were added. The bug was masked by earlier CI runs that used `continue-on-error: true`.

### Evidence

```javascript
// BEFORE FIX (auth.svelte.ts)
export const authState = {
  login(accessToken, refreshToken, expiresIn) {
    const claims = decodeJwtPayload(accessToken);
    _accessToken = accessToken;        // ← In-memory state only
    _refreshToken = refreshToken;      // ← Not persisted
    _expiresIn = expiresIn;            // ← Lost on page reload
    _currentUser = { ... };
  },
};

// PLAYWRIGHT FLOW:
// 1. page.goto('/login') → authState.login() called → _accessToken set
// 2. page.goto('/accounts') → page reload → _accessToken cleared
// 3. load() checks authState.isAuthenticated → false
// 4. load() redirects('/login')
```

## Fix Implementation

### Changes Made

1. **localStorage Persistence** (`auth.svelte.ts`)
   - Added storage key constants: `kesh:auth:accessToken`, `kesh:auth:refreshToken`, `kesh:auth:expiresIn`
   - Modified `login()` to persist tokens to localStorage
   - Modified `logout()` and `clearSession()` to remove from localStorage

2. **Token Hydration** (`auth.svelte.ts`)
   - Added `hydrate()` method to restore tokens from localStorage on app startup
   - Validates token claims (sub, role) before restoring
   - **Validates token expiration** — rejects expired tokens, clears localStorage
   - Logs errors for debugging: "Token expired", "Hydration failed"

3. **Client-Side Hook** (`hooks.client.ts`)
   - Calls `authState.hydrate()` at module load time (before SvelteKit load functions)
   - SSR-safe: checks `typeof window` before accessing localStorage

4. **Test Isolation** (`frontend/tests/e2e/`)
   - Added `clearAuthStorage()` helper to remove auth tokens from localStorage between tests
   - Replaced ineffective `page.context().clearCookies()` with explicit localStorage cleanup
   - Applied to all 12 test specs to prevent token bleed

### Code Path After Fix

```javascript
// PAGE LOAD
1. Browser loads page
2. hooks.client.ts runs → authState.hydrate()
   - Checks localStorage for kesh:auth:* keys
   - If present + valid + not expired: restore to _accessToken, _refreshToken, _currentUser
3. SvelteKit load() runs
   - Checks authState.isAuthenticated → true (from hydrated state)
   - Does NOT redirect to /login
4. Page renders with auth state
5. API calls include Authorization: Bearer <token>

// PLAYWRIGHT TEST
1. test.beforeEach() → clearAuthStorage() removes all auth keys from localStorage
2. page.goto('/login') → fresh page, no hydrated tokens
3. Login form submit → backend returns tokens
4. authState.login() → saves tokens to localStorage + memory
5. page.goto('/accounts') → page reload occurs
6. On reload: hooks.client.ts → hydrate() restores tokens from localStorage
7. load() sees authState.isAuthenticated = true → no redirect
8. Page renders protected content ✅
```

## Validation

### Local Test Results

Before fix:
```
76 failed, 0 passed, 8 skipped
```

After fix:
```
40 passed, 36 failed, 8 skipped
36 remaining failures are pre-existing Playwright selector flakiness (strict mode violations)
```

**Key Validation:** Auth tests now pass 100%
- `auth.spec.ts`: All login/logout tests passing
- Navigation to protected routes post-login: Working
- Token persistence across page reload: Working

### Known Limitations

**Token Expiration Check:** Added validation during hydration to reject expired tokens, but API client does not automatically refresh expired tokens before requests. Pre-existing limitation, not addressed in Story 6-5.

**Multi-Tab Sync:** localStorage changes in one tab don't trigger updates in others. Pre-existing limitation, deferred to future story.

**Playwright Storage Isolation:** With `workers: 1`, Playwright reuses browser context. `clearAuthStorage()` clears auth keys but not entire localStorage. Sufficient for this scope but noted for future optimization.

## Conclusion

**Root Cause:** JWT tokens not persisted to localStorage → lost on page reload → SvelteKit load redirects to `/login`

**Fix:** Implement localStorage persistence + hydration on app startup + error handling for expired tokens

**Result:** E2E tests now functional (40/76 passing, up from 0/76). Remaining failures are unrelated (Playwright selector issues, not auth flow).

**AC Compliance:**
- ✅ AC #1: Investigation local reproduced bug (above)
- ✅ AC #2: Root cause documented (this file)
- ✅ AC #3: Fix applied & compiled
- ⚠️ AC #4: 40/76 tests passing (up from 0/76, but spec requires 100%)
- ✅ AC #5: `continue-on-error: true` removed from CI
- ✅ AC #6: GitHub issue #19 closed via commit message
- ⏳ AC #7: Pending AC #4 completion
