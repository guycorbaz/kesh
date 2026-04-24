---
title: "Story 7.6 — E2E Tests : Refactorer getByText() → data-testid robustes"
status: ready-for-dev
story_id: "7.6"
epic: 7
related_kf: "KF-010"
created: 2026-04-24
---

# Story 7.6: E2E Selector Refactoring

**Status:** ready-for-dev  
**Epic:** 7 (Technical Debt Closure)  
**Related Issue:** KF-010

---

## Story

**As a** QA/Developer,  
**I want** que les tests E2E utilisent des sélecteurs CSS robustes et spécifiques (`data-testid`) au lieu des localisateurs fragiles basés sur le texte (`getByText()`),  
**So that** les tests restent stables lors des changements de copie UI et réduisent les faux positifs (strict mode violations).

---

## Contexte

### Problème identifié

Les tests E2E (`onboarding.spec.ts`, `users.spec.ts`, et autres) utilisent largement des localisateurs Playwright basés sur le texte :

```typescript
// ❌ Fragile — cassera si la copie change
await expect(page.getByText('admin')).toBeVisible();

// ❌ Strict mode violation — matche plusieurs éléments
Locator: getByText('admin') resolved to 5 elements:
  1) <span class="text-sm">Admin</span> aka locator('#bits-c1').getByRole('button', { name: 'Admin' })
  2) <span class="mb-1 text-xs font-medium uppercase tracking-wider text-text-muted">Administration</span>
  3) <td data-slot="table-cell">…</td> aka getByRole('cell', { name: 'admin Vous' })
  4) <td data-slot="table-cell">…</td> aka getByRole('cell', { name: 'Admin' }).nth(1)
  5) <td data-slot="table-cell">…</td> aka getByRole('cell', { name: 'Admin' }).nth(3)
```

### Impact

1. **Maintenance long-terme fragile :** Chaque changement de libellé UI casse les tests
2. **False positives en strict mode :** Sélecteur ambigü → test flaky, ralentit le CI
3. **Inefficacité de refactoring UI :** Impossible de changer la copie sans casser les E2E
4. **Équipe frustration :** Reparation répétée de tests qui ne testent pas le comportement

### Solution recommandée

Utiliser `data-testid` attributes pour les localisateurs de test :

```typescript
// ✅ Robuste — localisateur explicite et stable
await expect(page.locator('[data-testid="admin-user-row"]')).toBeVisible();

// ✅ Combine text + precise selector pour actions
await page.locator('[data-testid="user-list"]').getByText('admin').nth(0).click();
```

---

## Acceptance Criteria

### AC 1 : Ajouter data-testid aux composants clés

**Given** composants Svelte/HTML dans le codebase  
**When** ces composants sont destinés à être testés en E2E  
**Then** ils doivent avoir un attribut `data-testid` unique et stable :
- Format: `data-testid="component-semantic-name"` (kebab-case)
- Exemples: `data-testid="user-list"`, `data-testid="admin-user-row"`, `data-testid="create-invoice-button"`
- Placer sur l'élément interactif ou le conteneur principal

**Files to update:**
- `frontend/src/lib/components/**/*.svelte` — tous les composants testés
- `frontend/src/routes/**/*.svelte` — pages testées

### AC 2 : Refactorer onboarding.spec.ts

**Given** tests AC 5 et AC 6 en `onboarding.spec.ts`  
**When** ces tests lancent  
**Then** ils doivent utiliser `data-testid` au lieu de `getByText()` :
- AC 5 (Path A - démo): Remplacer `getByText('Configuration incomplète')` → `data-testid="invoice-config-banner"`
- AC 5: Remplacer `getByText('Créer la facture')` → `data-testid="create-invoice-button"`
- AC 6 (Path B - production): Mêmes changements de sélecteur
- Valider que AC 5 et AC 6 tests **PASSING** après refactor

### AC 3 : Refactorer users.spec.ts

**Given** tests utilisateurs  
**When** ces tests lancent  
**Then** remplacer tous les `getByText('admin')` par des sélecteurs `data-testid` :
- `data-testid="admin-name-header"` pour bouton Admin
- `data-testid="user-table"` pour le tableau utilisateurs
- `data-testid="user-row-{username}"` pour chaque ligne utilisateur
- Valider que "liste des utilisateurs affichée avec tableau" test **PASSING**

### AC 4 : Audit et refactoring généralisé

**Given** tous les fichiers `.spec.ts` dans `frontend/tests/e2e/`  
**When** scanning pour `getByText()`, `getByLabel()`, `getByRole()` sans fallback `data-testid`  
**Then** créer un plan de migration pour les fichiers identifiés :
- Lister les fichiers à refactorer
- Identifier les sélecteurs à remplacer
- Prioriser par fréquence d'utilisation (files with most brittle selectors first)
- Documenter la stratégie

### AC 5 : Tests valident strict mode compliance

**Given** tous les tests E2E refactorisés  
**When** `npx playwright test` lancé  
**Then** aucune "strict mode violation" ne doit apparaître dans la sortie
- Chaque localisateur doit matcher **exactement** 1 élément
- Aucun `.nth(N)` pour disambiguer
- Logs: "✅ No strict mode violations"

### AC 6 : Documentation du pattern

**Given** refactoring complété  
**When** développeurs futurs écrivent des tests E2E  
**Then** un guide `E2E_TESTING_BEST_PRACTICES.md` doit exister :
- Pattern: "Use `data-testid` for all test-level selectors"
- Bad: `getByText('exact-copy')` — too brittle
- Good: `data-testid="user-list-row"` — stable & explicit
- Code examples pour les patterns courants (clicking, typing, waiting)

---

## Spécifications Techniques

### T1: Ajouter data-testid aux composants critiques

**Files:** `frontend/src/lib/components/` et `frontend/src/routes/`

**Pattern pour chaque composant testé:**

```svelte
<!-- Avant -->
<button on:click={...}>Créer la facture</button>

<!-- Après -->
<button data-testid="create-invoice-button" on:click={...}>Créer la facture</button>
```

**Components prioritaires (basé sur E2E suite):**
1. InvoiceForm.svelte — buttons, warning banner
2. UserList.svelte — user rows, admin header
3. OnboardingFlow.svelte — step buttons
4. ContactList.svelte — table cells
5. JournalEntries.svelte — buttons, filters

**Nommage:** Format `kebab-case`, sémantique, inchangé lors des refactors UI

### T2: Refactorer onboarding.spec.ts (AC 5 & AC 6)

**File:** `frontend/tests/e2e/onboarding.spec.ts` (lines 119–202)

**Before:**
```typescript
// AC 5 Path A
await expect(page.getByText('Configuration incomplète')).not.toBeVisible();
await expect(page.getByText('Créer la facture')).toBeEnabled();

// AC 6 Path B
await expect(page.getByText('Configuration incomplète')).not.toBeVisible();
```

**After:**
```typescript
// AC 5 Path A
await expect(page.locator('[data-testid="invoice-config-warning"]')).not.toBeVisible();
await expect(page.locator('[data-testid="create-invoice-button"]')).toBeEnabled();

// AC 6 Path B
await expect(page.locator('[data-testid="invoice-config-warning"]')).not.toBeVisible();
```

### T3: Refactorer users.spec.ts

**File:** `frontend/tests/e2e/users.spec.ts` (lines 44–98)

**Before:**
```typescript
await expect(page.getByText('admin')).toBeVisible();  // ❌ 5 matches!
await expect(page.getByText('Vous')).toBeVisible();
await expect(page.getByText('test-1777018536189')).toBeVisible();
```

**After:**
```typescript
await expect(page.locator('[data-testid="admin-section-header"]')).toBeVisible();
await expect(page.locator('[data-testid="current-user-badge"]')).toBeVisible();
await expect(page.locator('[data-testid="user-row-test-1777018536189"]')).toBeVisible();
```

### T4: Migration audit & prioritization

**Script:** Create `frontend/scripts/audit-e2e-selectors.js` or equivalent

```bash
# Identify all getByText/getByLabel/getByRole without data-testid fallback
grep -r "getByText\|getByLabel\|getByRole" frontend/tests/e2e/*.spec.ts \
  | grep -v "data-testid" \
  | wc -l
```

**Output:** List all brittle selectors and plan refactoring order

### T5: Documentation

**File:** Create `frontend/docs/E2E_TESTING_BEST_PRACTICES.md`

**Contents:**
- ✅ Best practices: `data-testid` pattern
- ❌ Anti-patterns: `getByText()`, `getByLabel()` sans fallback
- Code examples: click, type, wait patterns
- Maintenance: when/how to update selectors
- CI integration: strict mode validation

---

## Dev Notes

### Related Stories / Epics
- **Epic 6.5** (Fix Playwright E2E auth): Reduced KF-007 (localStorage) but didn't address selector brittleness
- **Epic 6** (Quality & CI/CD): Foundation for test robustness
- **Story 2-6** Code Review: Identified this as MEDIUM finding during onboarding tests refactoring

### Known Constraints
- `data-testid` attributes add ~5 bytes per element (negligible)
- Svelte components already support `data-*` attributes natively
- No breaking changes to production builds (test attributes are inert)

### Testing Approach
1. **Unit tests:** None needed (data-testid is HTML markup)
2. **Integration tests:** Run Playwright suite before/after → compare pass/fail counts
3. **E2E validation:**
   - Before refactor: 40/76 tests passing (KF-008 baseline)
   - After refactor: expect same or higher pass rate
   - Strict mode violations should drop to 0

### Risks & Mitigations

| Risk | Mitigation |
|------|-----------|
| Incomplete refactor leaves orphaned `getByText()` | Run audit script at end (T4) to verify 100% coverage |
| Incorrect `data-testid` naming breaks tests | Use consistent kebab-case naming, document pattern in T5 |
| Merge conflicts with concurrent UI work | Coordinate with frontend team or merge on main before refactoring |
| Performance regression | `data-testid` attributes are inert; no perf impact expected |

---

## Fichiers à toucher

```
frontend/src/lib/components/invoices/InvoiceForm.svelte
frontend/src/lib/components/users/UserList.svelte
frontend/src/lib/components/onboarding/OnboardingFlow.svelte
frontend/src/lib/components/contacts/ContactList.svelte
frontend/src/lib/components/journal-entries/JournalEntries.svelte
frontend/tests/e2e/onboarding.spec.ts
frontend/tests/e2e/users.spec.ts
frontend/tests/e2e/contacts.spec.ts
frontend/tests/e2e/journal-entries.spec.ts
frontend/tests/e2e/accounts.spec.ts
frontend/tests/e2e/products.spec.ts
frontend/docs/E2E_TESTING_BEST_PRACTICES.md (create)
frontend/scripts/audit-e2e-selectors.js (create)
```

---

## Change Log

### 2026-04-24 — Story Creation (Pass 1)
- **Status:** ready-for-dev
- **Created:** Initial comprehensive story with full AC + technical specs
- **Source:** KF-010 (Story 2-6 code review finding, MEDIUM severity)
- **Next:** `/bmad-dev-story` to implement AC 1–6

---

**🎯 Ready for implementation. Developer has everything needed for flawless execution!**
