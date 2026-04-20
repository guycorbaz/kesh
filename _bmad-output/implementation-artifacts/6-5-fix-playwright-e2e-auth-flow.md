# Story 6.5 : Fix Playwright E2E auth flow

Status: ready-for-dev

## Story

As a **développeur (Guy, mainteneur solo)**,
I want **diagnostiquer et corriger le bug Playwright E2E qui empêche la navigation authentifiée (redirection `/login` persistante post-seed)**,
so that **les tests Playwright passent 100% et la CI Gate E2E devient fonctionnelle, débloquant la release v0.1 sans debt temporaire de `continue-on-error: true`**.

### Contexte

**Known Failure KF-007** — découvert pendant Story 6-4 (Fixtures E2E déterministes) et documenté dans GitHub issue #19. Symptôme : après login API `/auth/login` apparemment réussi, toute navigation vers une page authentifiée (`/accounts`, `/products`, `/contacts`, `/users`, `/journal-entries`, `/invoices`, `/homepage-settings`, etc.) redirige vers `/login` ou affiche un titre `h1` vide. Variante : `page.request.get('/api/v1/accounts')` retourne `401 Unauthorized`.

**Pré-existant** — tous les runs CI précédant Story 6-4 étaient rouges avec le même pattern, y compris PR #16 Story 6-1. **Non introduit par Story 6-4** (fixtures Rust + endpoints `/api/v1/_test/*` fonctionnent correctement).

**Mitigation temporaire (Story 6-4, PR #18)** — `continue-on-error: true` appliqué au job `e2e` dans `.github/workflows/ci.yml`. Permet PR #18 de merger et génère l'image Docker, mais **aucune détection de régression UI en CI**.

**Blocant PROD v0.1** — ces tests doivent être corrigés et `continue-on-error: true` retiré AVANT la première release production. Sans E2E fonctionnel, toute régression UI passe inaperçue.

### État pré-Story-6.5

- **Backend** : Rust + Axum, auth via JWT tokens, stockage accessToken/refreshToken en localStorage côté client
- **Frontend** : SvelteKit app, fichier `frontend/src/lib/auth.ts` gère persistance JWT + injection dans Authorization header
- **Tests Playwright** : ~12 specs dans `frontend/tests/e2e/`, 60–80 tests au total, ~60 en échec (déjà migrés vers `seedTestState` helper en Story 6-4)
- **CI job `e2e`** : lance tests contre backend `:3000` + SvelteKit preview `:4173` (proxy non-fonctionnel)
- **5 fixes tentés sans succès** :
  1. Proxy `vite preview` dans config
  2. Playwright cible `:3000` directement au lieu de `:4173`
  3. Rate limiter élargi (`KESH_RATE_LIMIT_MAX_ATTEMPTS: 1000`)
  4. `workers: 1` dans `playwright.config.ts`
  5. Migrations + seed CI élargi

### Root cause hypothétiques (à investiguer)

Selon issue #19 :

1. **Bug auth frontend** — localStorage/cookie non persisté entre `page.goto()`, SvelteKit load function redirigeant prématurement, hydratation race condition
2. **Interaction SvelteKit/Playwright** — SPA fallback routing côté `ServeDir` backend interfère avec les transitions frontend
3. **Timing Playwright** — submit formulaire login détecté incorrectement, JWT assigné à localStorage mais non synchronisé avant requête suivante
4. **Network/redirect silencieux** — HTTP 302/303 côté backend non capturé, ou redirects JS côté frontend passant inaperçus

### Scope & Décisions

**Volet 1 — Investigation & Debugging (LOCAL FIRST)**
- Lancer backend + Playwright en local avec `KESH_TEST_MODE=true`
- DevTools : observer localStorage, Network, redirects HTTP
- Comparer passing tests (`auth.spec.ts` 100% OK, `onboarding.spec.ts` OK) vs failing tests
- Isoler le pattern de régression

**Volet 2 — Root Cause Analysis**
- JWT stockage/transmission : localStorage.getItem('accessToken'), Authorization header présent ?
- SvelteKit load function : redirection `/login` vient d'où ? (`load.route.goto('/login')` ? Navigation implicit ?)
- Race conditions : Promise.all vs sequential await ?
- Playwright context : cookies vs localStorage isolation entre tests ?

**Volet 3 — Correctif & Validation**
- Patch root cause (probablement côté auth.ts frontend ou SvelteKit load)
- Retirer `continue-on-error: true` de `.github/workflows/ci.yml`
- Tests Playwright 100% passant en local + CI

**Scope volontairement HORS story**
- Performance optimizations des tests E2E (actuellement lents) — future task
- Refactor global de la logique auth (Story 1-5 est complète) — only if evidence of broader issues
- Multi-worker parallel e2e (actuellement `workers: 1`) — future task après que KF-007 soit closed

## Acceptance Criteria

### AC #1 — Investigation locale reproduit le bug
**Given** backend démarré localement avec `KESH_TEST_MODE=true KESH_HOST=127.0.0.1 DATABASE_URL=...` (DB live ou ephemeral),
**When** `cd frontend && npm run test:e2e -- --debug accounts.spec.ts` (un seul test),
**Then** j'observe DevTools : localStorage après login, Network requests avec Authorization headers, HTTP status codes de redirects.

### AC #2 — Root cause documenté
**Given** investigation locale terminée,
**When** inspecté `frontend/src/lib/auth.ts`, `frontend/src/routes/+page.svelte` (login form), SvelteKit `+page.server.ts` (load functions),
**Then** j'identifie le point spécifique où redirection `/login` survient (ou pourquoi JWT n'est pas envoyé) ET cette cause est documentée dans `frontend/DEBUGGING-KF007.md` (ou section Dev Notes ci-dessous).

### AC #3 — Correctif appliqué & compilé
**Given** root cause identifiée,
**When** patch appliqué (modification fichiers TypeScript/SvelteKit),
**Then** `cargo fmt --all -- --check` ✅, `cargo clippy -- -D warnings` ✅, `npm run check` ✅, `npm run build` ✅.

### AC #4 — Tests Playwright locaux 100% verts
**Given** correctif en place,
**When** `npm run test:e2e -- --reporter=list` lancé localement (backend live + frontend build via SvelteKit dev server ou preview),
**Then** sortie : **aucune ligne `FAILED`**, tous les 60–80 tests Playwright passent.

### AC #5 — `continue-on-error: true` retiré de CI
**Given** tests Playwright locaux stables,
**When** inspectant `.github/workflows/ci.yml`, job `e2e`,
**Then** ligne `continue-on-error: true` est SUPPRIMÉE ou commentée (commit explicite documentant "KF-007 fixed").

### AC #6 — GitHub issue #19 fermée avec le correctif
**Given** correctif validé en CI,
**When** inspectant GitHub issue #19 (KF-007),
**Then** issue fermée via commit message `(closes #19)` ou `(fixes #19)`, sans édition de `docs/known-failures.md` (archivé depuis 2026-04-18).

### AC #7 — CI Gate E2E verte (all required checks pass)
**Given** correctif merged sur main,
**When** GitHub Actions job `e2e` lancé,
**Then** job complété avec `conclusion: success` (pas `failure`, pas `skipped`). Branch protection Gateway passe.

## Developer Context

### Architecture & Code Patterns

**Frontend Auth System** (`frontend/src/lib/auth.ts`)
- JWT tokens (accessToken + refreshToken) stockés dans localStorage
- `Authorization: Bearer <token>` header injecté dans toutes les requêtes API via fetch wrapper ou SvelteKit hooks
- Login POST → `/api/v1/auth/login` → response avec `accessToken` + `refreshToken`
- Logout : clear localStorage

**SvelteKit Navigation & Load Functions**
- Entry point `frontend/src/routes/+page.svelte` : accueil après login (GET `/`)
- Spec-specific routes : `/accounts`, `/products`, `/contacts`, `/journal-entries`, `/invoices`, `/users`, `/homepage-settings`, etc.
- Chaque route peut avoir un `+page.server.ts` avec `load()` qui **redirection vers `/login` si token invalide ou absent**
- **Clé à vérifier** : est-ce que la redirection `/login` vient du backend (route protégée retourne 401 → SvelteKit auto-redir) ou du frontend (load() explicit `goto('/login')`)?

**Playwright Seeding (Story 6-4)**
- `frontend/tests/e2e/helpers/test-state.ts` : `seedTestState(preset)` appelle `/api/v1/_test/seed` 
- Login : `page.goto('/login'); page.fill('input[name=username]', 'admin'); page.fill('input[name=password]', '...'); page.click('button[type=submit]')`
- Assertion : `page.goto('/accounts')` + assert page title ou `h1` content

**Test Spec Inventory**

*Passing specs:*
- `frontend/tests/e2e/auth.spec.ts` (4 tests: login, logout, accessibility)
- `frontend/tests/e2e/onboarding.spec.ts` (3+ tests: onboarding flow)

*Failing specs (~60 tests total):*
- `frontend/tests/e2e/accounts.spec.ts` (8 tests)
- `frontend/tests/e2e/contacts.spec.ts` (8 tests)
- `frontend/tests/e2e/products.spec.ts` (8 tests)
- `frontend/tests/e2e/invoices.spec.ts` (8 tests)
- `frontend/tests/e2e/journal-entries.spec.ts` (8 tests)
- `frontend/tests/e2e/users.spec.ts` (8 tests)
- `frontend/tests/e2e/homepage-settings.spec.ts` (4 tests)
- `frontend/tests/e2e/mode-expert.spec.ts` (2 tests)
- `frontend/tests/e2e/invoices-echeancier.spec.ts` (4 tests)
- `frontend/tests/e2e/onboarding-path-b.spec.ts` (status to verify)

**Playwright & Browser Configuration**
- Playwright version: `frontend/package.json` specifies version (confirm before dev)
- Recommended browser: Chromium (same as CI in `.github/workflows/ci.yml`)
- Default timeout: 30s (sufficient for local testing; may need adjustment if network slow)
- Workers: 1 (already configured in `frontend/playwright.config.ts` — **do not change**)

### Recent Pattern Learnings (Stories 6-1 → 6-4)

- **Test Mode & CI Isolation** : `KESH_TEST_MODE=true` env var gates endpoints + refusal of non-loopback binds. Pattern réussi.
- **Endpoint Seeding vs SQL Inline** : `/api/v1/_test/seed` + `seedTestState` helper > SQL inline. Pattern validé Story 6-4.
- **Jest/Playwright Timing** : `await page.fill(...)` + `await page.click(...)` + explicit waits. Async/await strictly required.
- **LocalStorage Persistence** : localStorage persists across `page.goto()` sauf context/browser close. **À vérifier** : localStorage set en login est-il bien persisté avant navigation ?
- **Multi-spec Isolation** : `test.beforeAll()` avec reset complet préserve isolation. Pattern working.

### Known Technical Debt

- **D-6-4-E** (KF-007) — cf. Story 6-4 Dev Notes. No auth on `/api/v1/_test/*` (mitigated by KESH_TEST_MODE gate).
- **D-6-5-? (TBD)** — Issues découvertes pendant investigation de KF-007 à documenter ici pendant dev.

### Git & Workflow Context

- **Branch** : `story/6-5-fix-playwright-e2e-auth-flow`
- **Base** : `main` (dernière = 7da8328 sprint status update)
- **CI pattern** : push → run `ci.yml` (Backend, Docker, Frontend, E2E tests)
- **Merging** : PR review + branch protection checks (Backend, Docker, Frontend, E2E) then squash-merge
- **Local testing first** : cf. feedback_local_tests_first.md — all tests pass locally before push

## Previous Story Intelligence (Story 6-4)

### Learnings from 6-4 Implementation

1. **Test endpoint security** — `continue-on-error: true` is pragmatic temporary fix but unacceptable long-term. Proper fix required before prod.
2. **DB seeding reliability** — `seedTestState` helper (HTTP call) beats inline SQL. Apply same principle to auth investigation.
3. **DevTools debugging** — 5 fixes tried without root-cause investigation. **This story prioritizes understanding over trial-and-error**.
4. **Pre-existing bugs** — CI was already failing pre-6-4. Likely a fundamental frontend auth issue unrelated to fixtures.

### Refactoring Opportunities

- If auth is broken, consider documenting the bug pattern (for future maintainers)
- Once fixed, add a regression test (Playwright spec that explicitly validates auth + navigation sequence)

## Investigation Guide (Step-by-step)

### Step 1 : Local Setup
```bash
# Terminal 1 — Backend
export KESH_TEST_MODE=true
export KESH_HOST=127.0.0.1
export DATABASE_URL=mysql://test:test@localhost:3306/test  # or use sqlx offline mode
cd crates/kesh-api
cargo run  # starts on :3000

# Terminal 2 — Verify localStorage Key (IMPORTANT)
cd frontend/src/lib
grep -n "localStorage.setItem" auth.ts
# Expected: 'accessToken' key, but verify the actual key name used

# Terminal 3 — Frontend E2E Tests
cd frontend
npm run test:e2e -- --debug accounts.spec.ts
# Playwright launches with browser UI + DevTools
```

### Step 2 : Observe Login
1. Watch login form submission in DevTools Network tab
2. POST `/api/v1/auth/login` — check response status + JSON body (should contain accessToken)
3. Switch to Application tab → localStorage — is the localStorage key (verified in Step 1) present and non-empty?
4. Console — run `localStorage.getItem('<KEY_FROM_STEP_1>')` manually (should return JWT string, e.g., `localStorage.getItem('accessToken')`)

### Step 3 : Post-Login Navigation
1. In Playwright test, after login assertion, add pause: `await page.pause();`
2. DevTools → Network tab, clear + reload
3. Manually click link to `/accounts` (or `page.goto('/accounts')` in console)
4. Watch Network requests :
   - Is `Authorization: Bearer <token>` header present in GET `/api/v1/accounts`?
   - What HTTP status? 401? 200? Redirect 302?
5. If 401 → backend rejecting token. If 302 → backend redirecting. If empty header → frontend not sending token.

### Step 4 : Compare Passing Tests
- `auth.spec.ts` passes (~4 tests) — login + axe scan. **What's different?**
  - Does it actually navigate post-login, or just validates login endpoint?
  - Check source : `frontend/tests/e2e/auth.spec.ts`
- `onboarding.spec.ts` passes (some tests) — uses `seedTestState('fresh')`. **Check navigation pattern**
  - Does it navigate to authenticated routes after "starting onboarding"?
  - or stays in onboarding flow (which might not require full auth)?

### Step 5 : Likely Suspects (by Investigation Priority)

Investigate in this order — **start with #1, skip others if root cause found**:

**PRIORITY 1 (highest likelihood) — Frontend auth.ts**
- Is token persisted to localStorage correctly? `localStorage.setItem('accessToken', ...)` called?
- Is fetch wrapper injecting `Authorization: Bearer <token>` header?
- Search: `grep -n "Authorization\|Bearer" frontend/src/lib/auth.ts`
- Test: `localStorage.getItem('accessToken')` in DevTools console after login — should return JWT string

**PRIORITY 2 — SvelteKit Hooks Interference**
- Are `hooks.server.ts` or `hooks.client.ts` intercepting requests/navigation and redirecting prematurely?
- Search: `grep -r "redirect\|goto" frontend/src/hooks.*.ts`
- **Hooks run BEFORE load functions**, so early redirect here blocks authenticated routes

**PRIORITY 3 — SvelteKit Load Functions**
- Any +page.server.ts with early `redirect(303, '/login')`?
- Search: `grep -r "redirect.*'/login'" frontend/src/routes/`
- Are load functions checking for valid token before running?

**PRIORITY 4 — Playwright Context Persistence**
- Is localStorage truly persisted across `page.goto()` in Playwright?
- Add test: `const token = await page.evaluate(() => localStorage.getItem('accessToken')); console.log('Token after nav:', token);`
- Verify localStorage not cleared between login and navigation

**PRIORITY 5 (lowest likelihood) — Backend JWT Validation**
- Is backend properly validating Authorization header?
- Search: `grep -r "Authorization" crates/kesh-api/src/`
- Is JWT verification middleware rejecting valid tokens silently?

### Step 6 : Document Finding & Implement Fix

Once root cause is clear, **before coding**:
- Write a brief `FINDINGS.md` explaining what's broken
- Propose the fix (1–2 line change? Larger refactor?)
- Get confidence on the hypothesis by adding a targeted log/assertion

Then code the fix and validate locally.

## Testing Strategy

### Pre-Implementation Testing
- Start with single-test runs (`--debug accounts.spec.ts`) to minimize noise
- Compare against passing tests (`auth.spec.ts`)
- Use DevTools to validate JWT presence + transmission

### Post-Fix Validation
- All 9 admin specs : `npm run test:e2e -- accounts.spec.ts contacts.spec.ts products.spec.ts invoices.spec.ts journal-entries.spec.ts users.spec.ts homepage-settings.spec.ts mode-expert.spec.ts invoices-echeancier.spec.ts`
- All 2 onboarding specs : `npm run test:e2e -- onboarding.spec.ts onboarding-path-b.spec.ts`
- Auth spec : `npm run test:e2e -- auth.spec.ts`
- **Full suite** : `npm run test:e2e -- --reporter=list` (all 12 specs)

### Regression Prevention
- ✅ `auth.spec.ts` MUST still pass 100% (no auth flow regression)
- ✅ `onboarding.spec.ts` MUST still pass 100% (no onboarding flow regression)
- ✅ No logic changes to `frontend/src/lib/auth.ts` beyond bug fix (refactors deferred to future story)
- ✅ Backend JWT validation logic unchanged (fix is in frontend only)

### CI Validation
- Merge to main branch → trigger `.github/workflows/ci.yml`
- Confirm all 4 jobs pass : Backend, Docker, Frontend, **E2E** (without `continue-on-error: true`)

## Success Metrics

✅ **Local Tests** : 100% of Playwright specs pass without retry or skip
✅ **CI Tests** : E2E job passes with `conclusion: success` (not `skipped`)
✅ **Code Quality** : `cargo fmt`, `cargo clippy`, `npm run check` all clean
✅ **KF-007 Closed** : `docs/known-failures.md` updated with fix explanation
✅ **No Regressions** : Backend + Frontend unit/integration tests unchanged

## Change Log

### Pre-Implementation
| Date | Phase | Notes |
|------|-------|-------|
| 2026-04-20 | Story File Creation | Story 6-5 created via `bmad-create-story` with comprehensive investigation guide. Ready for dev. |
| 2026-04-20 | Validation Pass 1 | Haiku review: 10 findings (3 CRITICAL, 4 HIGH, 3 MEDIUM). Applied 3 CRITICAL fixes: (1) AC #6 now references GitHub issue #19 closure instead of archived docs file, (2) Added test spec inventory with explicit pass/fail status, (3) Added localStorage key verification step in investigation guide. Also added 4 HIGH enhancements: prioritized Likely Suspects by investigation order, added Playwright/Browser configuration section, enhanced Step 2 with key name verification, added regression prevention checklist. Ready for Validation Pass 2 (Sonnet model). |

---

**Next Phase** : `bmad-dev-story` for implementation
