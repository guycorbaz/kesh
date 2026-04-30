# E2E Testing Best Practices

## Overview

End-to-end (E2E) tests using Playwright should use **stable, explicit selectors** to avoid flaky tests and brittle failures. This guide defines the patterns to follow when writing E2E tests in Kesh.

---

## Selector Strategy

### ✅ Best Practice: Use `data-testid`

**Why:** Explicit, stable identifiers that won't break when UI text or structure changes.

```typescript
// ✅ GOOD — Explicit, stable selector
await expect(page.locator('[data-testid="user-table"]')).toBeVisible();
await expect(page.locator('[data-testid="create-invoice-button"]')).toBeEnabled();
await page.locator('[data-testid="admin-user-row"]').click();
```

**Adding data-testid to components:**

```svelte
<!-- Svelte component -->
<button data-testid="create-invoice-button" on:click={...}>
  Créer la facture
</button>

<table data-testid="user-table">
  {#each users as user}
    <tr data-testid="user-row-{user.username}">
      ...
    </tr>
  {/each}
</table>
```

### ❌ Anti-Pattern: Text-based selectors

**Why:** Brittle — breaks when copy changes, ambiguous when multiple elements match.

```typescript
// ❌ BAD — Breaks if text changes, may match multiple elements
await expect(page.getByText('Configuration incomplète')).toBeVisible();
await page.getByText('Créer la facture').click();

// ❌ BAD — Strict mode violation if multiple "Admin" elements exist
await expect(page.getByText('admin')).toBeVisible();  // 5 matches!

// ❌ BAD — Regex fallback adds complexity
await expect(page.getByText(/Configuration/i)).toBeVisible();
```

### ❌ Avoid: Role-based selectors without specificity

```typescript
// ❌ QUESTIONABLE — Works for accessibility but can be fragile
// if the semantic role changes or multiple elements share a role
await page.getByRole('button', { name: 'Créer' }).click();

// ✅ BETTER — Scope to a parent (dialog, role, testid)
await page.getByRole('dialog').getByRole('button', { name: 'Créer' }).click();
// OR simply use:
await page.locator('[data-testid="create-button"]').click();
```

> **Story 7-5 (KF-008) :** `.first()` / `.nth()` ne sont **pas** une réponse acceptable au strict-mode violation. Ils masquent l'ambiguïté au lieu de la résoudre — la première passe peut être stable mais devient flaky dès qu'un autre composant insère un élément du même rôle. Préférer **toujours** un sélecteur unique (testid ou rôle scoped à un parent).

---

## Common Patterns

### Clicking elements

```typescript
// ✅ Preferred
await page.locator('[data-testid="create-invoice-button"]').click();

// ✅ Acceptable (role + name combination, scoped to parent)
await page.getByRole('dialog').getByRole('button', { name: 'Créer' }).click();
```

### Typing into inputs

```typescript
// ✅ Use ID or data-testid
await page.fill('#username', 'testuser');
await page.locator('[data-testid="contact-name-input"]').fill('Acme Inc');

// ❌ Avoid text-based labels without specificity
// await page.fill('label:has-text("Nom") input', '...'); // might match wrong input
```

### Waiting for elements

```typescript
// ✅ Explicit selectors
await expect(page.locator('[data-testid="loading-spinner"]')).toBeVisible();
await expect(page.locator('[data-testid="success-message"]')).toBeVisible({ timeout: 5000 });

// ❌ Avoid
// await expect(page.getByText('Chargement…')).toBeVisible();
```

### Table interactions

```typescript
// ✅ Use data-testid for rows
const userRow = page.locator('[data-testid="user-row-admin"]');
await expect(userRow).toBeVisible();
await userRow.click();

// ✅ Use data-testid for table
const table = page.locator('[data-testid="user-table"]');
await expect(table).toBeVisible();
```

### Form submission

```typescript
// ✅ Preferred
await page.locator('[data-testid="create-invoice-button"]').click();

// Also OK
await page.getByRole('button', { name: 'Créer la facture' }).click();
```

---

## Naming Convention

Use **kebab-case** for `data-testid` values:

```typescript
// ✅ GOOD
data-testid="user-table"
data-testid="admin-user-row"
data-testid="create-invoice-button"
data-testid="config-warning-banner"

// ❌ AVOID
data-testid="userTable"         // camelCase
data-testid="user_table"        // snake_case
data-testid="UserTable"         // PascalCase
data-testid="Create Button"     // spaces
```

**Dynamic IDs:**

```svelte
<!-- Include context in the testid -->
<tr data-testid="user-row-{user.username}">
<div data-testid="account-{account.id}">
<button data-testid="delete-invoice-{invoice.id}">
```

---

## Strict Mode Compliance

Playwright's **strict mode** requires selectors to match exactly one element. Violating this causes tests to fail.

```typescript
// ✅ GOOD — Matches exactly one element
await expect(page.locator('[data-testid="admin-section-header"]')).toBeVisible();

// ❌ BAD — Strict mode violation: "admin" matches 5 elements
await expect(page.getByText('admin')).toBeVisible();

// ❌ BAD — Must use .first() or .nth() to disambiguate (loses clarity)
await expect(page.getByText('admin').first()).toBeVisible();
```

**How to fix strict mode violations:**

1. Add `data-testid` to the component (preferred)
2. Combine selectors for specificity: `page.locator('table').getByText('value')` or `page.getByRole('dialog').getByText(...)`
3. **Do NOT use `.first()` / `.nth()` to bypass strict-mode** — masque l'ambiguïté sans la résoudre, source de flakiness. Réserver `.first()` à la sémantique légitime « première ligne quelconque » avec un commentaire `// strict-mode safe: ...` justifiant l'intention.

---

## Audit & Maintenance

### Running the selector audit

```bash
node frontend/scripts/audit-e2e-selectors.js
```

This script scans all `.spec.ts` files and identifies:
- **🔴 HIGH priority:** `getByText()` calls (brittle to copy changes)
- **🟡 MEDIUM priority:** `getByRole()` without specificity

### Refactoring priority

1. **Phase 1:** Replace `getByText()` with `data-testid` (41 occurrences)
2. **Phase 2:** Review and stabilize `getByRole()` patterns (141 occurrences)
3. **Phase 3:** Achieve 0 strict mode violations in CI

---

## CI Integration

The test pipeline enforces:

```bash
npx playwright test --reporter=line
```

To validate locally before committing:

```bash
npm run test:e2e
```

All tests must pass without strict mode violations.

---

## Migration Checklist

When refactoring existing brittle selectors:

- [ ] Add `data-testid` to component (if not already present)
- [ ] Update test to use `page.locator('[data-testid="..."]')`
- [ ] Run test locally: `npx playwright test <spec-file>`
- [ ] Verify no strict mode violations in output
- [ ] Run full suite: `npm run test:e2e`
- [ ] Commit with message: `fix(e2e): stabilize <selector> in <test-name>`

---

## Examples by Feature

### Users page

```typescript
// ✅ DO THIS
await expect(page.locator('[data-testid="user-table"]')).toBeVisible();
await expect(page.locator('[data-testid="user-row-admin"]')).toBeVisible();
await expect(page.locator('[data-testid="current-user-badge"]')).toBeVisible();

// ❌ DON'T DO THIS
await expect(page.getByText('admin')).toBeVisible();        // Ambiguous
await expect(page.getByText('Vous')).toBeVisible();         // Fragile
await expect(page.locator('table')).toBeVisible();          // Too general
```

### Invoices form

```typescript
// ✅ DO THIS
await expect(page.locator('[data-testid="create-invoice-button"]')).toBeEnabled();
await expect(page.locator('[data-testid="invoice-config-warning"]')).not.toBeVisible();

// ❌ DON'T DO THIS
await expect(page.getByText('Créer la facture')).toBeEnabled();
await expect(page.getByText('Configuration incomplète')).not.toBeVisible();
```

### Accounts page (Story 7-5 — KF-008)

```typescript
// ✅ Liste : conteneur + lignes par numéro de compte (clé naturelle PME)
await expect(page.locator('[data-testid="account-table"]')).toBeVisible();
await expect(page.locator('[data-testid="account-row-1000"]')).toBeVisible();
await expect(page.locator('[data-testid="account-row-1000-type-badge"]')).toContainText('Actif');

// ✅ Actions sur ligne : suffixe -edit-button / -archive-button scoped au numéro
await page.locator('[data-testid="account-row-1000-edit-button"]').click();

// ✅ Dialogs distincts (create / edit / archive) : suffixe pour disambiguer 3 boutons "Annuler"
await page.locator('[data-testid="account-create-dialog-cancel"]').click();
await page.locator('[data-testid="account-edit-dialog-cancel"]').click();
await page.locator('[data-testid="account-archive-dialog-confirm"]').click();
```

### Homepage cards & sidebar nav

```typescript
// ✅ Cards d'accueil
await expect(page.locator('[data-testid="homepage-card-recent-entries"]')).toBeVisible();
await expect(page.locator('[data-testid="homepage-card-open-invoices"]')).toBeVisible();
await expect(page.locator('[data-testid="homepage-card-bank-accounts"]')).toBeVisible();

// ✅ Sidebar : testid dérivé du href (kebab-case, slash → tiret)
//   /users → nav-link-users
//   /invoices/due-dates → nav-link-invoices-due-dates
await expect(page.locator('[data-testid="nav-link-users"]')).toBeVisible();
```

### Onboarding Path B

```typescript
// ✅ Boutons d'option (org type, mode, etc.) avec testid plutôt que texte traduit
await page.locator('[data-testid="onboarding-org-type-pme"]').click();
await page.locator('[data-testid="onboarding-org-type-association"]').click();

// ✅ Bannière "Configuration incomplète" rendue dans (app)/+layout.svelte
//    via lib/shared/components/IncompleteBanner.svelte — NE PAS confondre
//    avec invoice-config-warning (testid distinct dans InvoiceForm.svelte).
await expect(page.locator('[data-testid="incomplete-config-banner"]')).toBeVisible();
```

### Validation messages (formulaires)

```typescript
// ✅ Scope au dialog ouvert (la modale a un role="dialog" implicite via bits-ui)
await expect(page.getByRole('dialog')).toContainText('au moins 12 caractères');
await expect(page.getByRole('dialog')).toContainText('ne correspondent pas');

// ✅ Toast de succès (svelte-sonner expose role="region" name="Notifications")
await expect(page.getByLabel(/Notifications/)).toContainText(`Compte ${testNumber} créé`);

// ✅ Erreur inline scoped au testid
await expect(page.locator('[data-testid="product-form-error"]')).toContainText(/existe déjà/i);
```

## Naming Reference (par entité)

| Entité | `key` row | Pattern testid type |
|---|---|---|
| `accounts` | numéro de compte (`1000`, `2000`) | `account-row-{number}`, `account-row-{number}-{action}-button`, `account-{action}-dialog-{verb}` |
| `users` | `username` | `user-table`, `user-row-{username}`, `user-row-{username}-username-cell`, `user-create-button`, `current-user-badge` |
| `homepage` | section sémantique | `homepage-card-{slug}` (recent-entries, open-invoices, bank-accounts) |
| `sidebar nav` | href kebab | `nav-link-{href-slug}` (e.g. `nav-link-organization`, `nav-link-invoices-due-dates`) |
| `onboarding` | type | `onboarding-org-type-{slug}` (independant, association, pme) |
| `banners` | nature | `incomplete-config-banner`, `invoice-config-warning` (NE PAS confondre — composants distincts) |

## Audit baseline (Story 7-5 — 2026-04-30)

Pre-refactor : 192 brittle selectors (41 HIGH `getByText` + 151 MEDIUM `getByRole`).
Post-refactor : 173 brittle selectors (**9 HIGH** = 8 onboarding hors scope + 1 auth.spec.ts:34 `Kesh` accepté + 164 MEDIUM `getByRole` non-ambigus).

Strict-mode violations : 5 → **0**.

---

## Questions?

Refer to:
- **Playwright docs:** https://playwright.dev/docs/locators
- **Kesh E2E tests:** `frontend/tests/e2e/*.spec.ts`
- **Audit script:** `node frontend/scripts/audit-e2e-selectors.js`
