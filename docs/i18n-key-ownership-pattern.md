# i18n Key Ownership Pattern

## 1. Overview

The **Key Ownership Pattern** enforces a strict scoping rule for internationalization (i18n) keys in the Kesh frontend. It prevents cross-feature key pollution and ensures that feature-specific messages stay localized to their feature folder.

**Problem (Debt D8 / KF-006)**: The sidebar navigation was hardcoded in French, making it impossible to support multilingual users without code changes.

**Solution**: 
- All sidebar labels moved to FTL message files with feature-namespaced keys (`nav-*`)
- A linter rule enforces that keys belong to their feature or are in a global allowlist
- Integration into CI ensures new code cannot violate the rule

---

## 2. Convention

### Feature-Specific Keys

Keys are namespaced by the feature folder they belong to:

| Feature Folder | Allowed Key Prefix | Example |
|---|---|---|
| `features/journal-entries/` | `journal-entries-*` | `journal-entries-title` |
| `features/contacts/` | `contacts-*` | `contacts-add-button` |
| `features/invoices/` | `invoices-*` | `invoices-draft-status` |
| `shared/` or layout | (must use global) | `nav-home`, `error-required-field` |

**Rule**: If a file is at `frontend/src/lib/features/FEATURE_NAME/`, it may only use keys with namespace `FEATURE_NAME-*` (unless the namespace is global).

### Global (Allowlisted) Namespaces

These namespaces can be used anywhere:

```
error-*         # Validation/API errors
tooltip-*       # Hover and help text
common-*        # Cross-feature labels (OK, Cancel, Save, etc.)
mode-*          # UI mode toggles (guided, expert)
shortcut-*      # Keyboard shortcut hints
demo-*          # Demo-mode banners and labels
nav-*           # Navigation items (sidebar, menus)
```

**Why allowlist these?** 
- **error-\***: Errors are truly cross-feature (every form can fail).
- **tooltip-\***: Help text is generic and reused everywhere.
- **common-\***: Button labels and basic UI strings appear in multiple features.
- **mode-\***, **shortcut-\***, **demo-\***: System-level UI, not feature-specific.
- **nav-\***: Navigation is a shell concern, not a feature.

---

## 3. Development Workflow

### Adding a New i18n Key

1. **Identify the scope**: Is this message feature-specific or global?
   - Feature-specific? Use the feature name as prefix.
   - Cross-feature or system? Use a global namespace or request allowlisting.

2. **Add to FTL files** (one entry per locale):
   ```
   # frontend/crates/kesh-i18n/locales/fr-CH/messages.ftl
   contacts-add-button = Ajouter un contact
   
   # frontend/crates/kesh-i18n/locales/de-CH/messages.ftl
   contacts-add-button = Kontakt hinzufügen
   
   # ... (IT and EN)
   ```

3. **Use in code** (Svelte or TypeScript):
   ```svelte
   <script>
     import { i18nMsg } from '$lib/shared/utils/i18n.svelte';
   </script>
   
   <button>{i18nMsg('contacts-add-button', 'Add Contact')}</button>
   ```

4. **Run the linter locally**:
   ```bash
   npm run lint-i18n-ownership
   ```
   Should pass with no violations.

5. **Push**: The linter runs in CI; no special flags needed.

---

## 4. Allowlist & Exceptions

### When to Propose a New Global Namespace

If you have a message type that is genuinely cross-feature (used in 2+ unrelated features), propose adding a new global namespace:

1. **Check existing allowlist** at `frontend/scripts/lint-i18n-ownership.js` line 16:
   ```javascript
   const GLOBAL_NAMESPACES = ['error', 'tooltip', 'common', 'mode', 'shortcut', 'demo', 'nav'];
   ```

2. **Open a GitHub discussion** (not a PR) to propose the new namespace and explain why it's not feature-specific.

3. **Once approved**, update `GLOBAL_NAMESPACES` in the lint script and re-run linter.

### Example: Why `invoice-*` is NOT Global

Even though invoices are used in multiple features (contacts, accounting, reports), invoice-specific labels (`invoice-status`, `invoice-date-due`) belong to the invoices feature. Other features reference those via the invoices API; they don't define their own `invoice-*` keys.

---

## 5. Linting Locally

### Run the Linter

```bash
cd frontend
npm run lint-i18n-ownership
```

**Success** (exit 0):
```
✅ lint-i18n-ownership: PASS — No cross-feature i18n violations detected
```

**Failure** (exit 1):
```
❌ lint-i18n-ownership: FAIL — Found 3 violation(s):

  ./src/lib/features/contacts/ContactForm.svelte
    uses key "invoice-number" (invoice namespace) from different feature
    Recommendation: Move key to global namespace or feature folder
```

### CI Integration

The linter runs automatically in GitHub Actions:
```yaml
- name: Lint i18n key-ownership
  run: npm run lint-i18n-ownership
```

Lint failures block PR merges (via branch protection rules).

---

## 6. Examples

### ✅ Correct: Feature-Specific Key

**File**: `frontend/src/lib/features/journal-entries/JournalEntryForm.svelte`

```svelte
<script>
  import { i18nMsg } from '$lib/shared/utils/i18n.svelte';
</script>

<!-- journal-entries-* key is OK here because we're in journal-entries feature -->
<label>{i18nMsg('journal-entries-account', 'Compte')}</label>
```

**Lint result**: ✅ PASS

---

### ✅ Correct: Global Namespace

**File**: `frontend/src/lib/features/contacts/ContactForm.svelte`

```svelte
<button>{i18nMsg('common-save', 'Enregistrer')}</button>
<span>{i18nMsg('error-required', 'Champ obligatoire')}</span>
```

**Lint result**: ✅ PASS (common-* and error-* are allowlisted)

---

### ❌ Incorrect: Cross-Feature Key

**File**: `frontend/src/lib/features/contacts/ContactForm.svelte`

```svelte
<!-- WRONG: invoices-* key is not allowed in contacts feature -->
<label>{i18nMsg('invoices-number', 'Numéro de facture')}</label>
```

**Lint result**:
```
❌ FAIL — Found 1 violation(s):
  ./src/lib/features/contacts/ContactForm.svelte
    uses key "invoices-number" (invoices namespace) from different feature
    Recommendation: Move key to global namespace or feature folder
```

**Fix**: Either (a) move the label to `invoices-*` namespace and import from invoices feature, or (b) add it to a global namespace if truly cross-feature.

---

### ✅ Correct: Global Namespace Used Everywhere

**File**: `frontend/src/routes/(app)/+layout.svelte` (shared layout, not a feature)

```svelte
<a href="/">{i18nMsg('nav-home', 'Accueil')}</a>
<button>{i18nMsg('common-logout', 'Déconnexion')}</button>
```

**Lint result**: ✅ PASS (nav-* and common-* are allowlisted)

---

## 7. Troubleshooting

### Q: My lint fails even though my key looks correct

**A**: Check the file path. The linter identifies the feature based on the folder structure:
```
frontend/src/lib/features/FEATURE_NAME/...
                          ^^^^^^^^^^^^^^
                          This is extracted as the feature name
```

If you're in `frontend/src/lib/features/contacts/form.svelte`, you can only use `contacts-*` keys (or global namespaces).

---

### Q: Can I use a key from another feature?

**A**: No. The linter enforces single-feature ownership. Instead:

1. **Move the key to a global namespace** if it's truly cross-feature.
2. **Re-export the message** from the owning feature's component (React/Svelte patterns, not i18n).
3. **Create a new allowlist entry** (see Section 4).

---

### Q: The linter says my key violates ownership, but I didn't use it in a different feature

**A**: Check your file is in the right folder. Example:

- ❌ File at `frontend/src/lib/shared/MyComponent.svelte` using `contacts-name` → FAIL (not in features folder, so it's "global", but using feature-specific key)
- ✅ File at `frontend/src/lib/features/contacts/MyComponent.svelte` using `contacts-name` → PASS

Shared components in `frontend/src/lib/shared/` should only use global namespaces.

---

### Q: I need a key that belongs to multiple features

**A**: This is a sign the key should be global. Propose adding it to `GLOBAL_NAMESPACES` in `frontend/scripts/lint-i18n-ownership.js`:

```javascript
const GLOBAL_NAMESPACES = ['error', 'tooltip', 'common', 'mode', 'shortcut', 'demo', 'nav', 'your-new-namespace'];
```

Example: If `status-pending`, `status-approved`, `status-draft` appear in invoices AND journal-entries, they belong in a global `status-*` namespace.

---

## References

- **FTL Message Files**: `crates/kesh-i18n/locales/*/messages.ftl`
- **Lint Script**: `frontend/scripts/lint-i18n-ownership.js`
- **i18n Helper**: `frontend/src/lib/shared/utils/i18n.svelte`
- **GitHub Issue (Debt D8 / KF-006)**: [guycorbaz/kesh/issues/6](https://github.com/guycorbaz/kesh/issues/6)
