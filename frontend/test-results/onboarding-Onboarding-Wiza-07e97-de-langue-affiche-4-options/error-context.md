# Instructions

- Following Playwright test failed.
- Explain why, be concise, respect Playwright best practices.
- Provide a snippet of code with the fix, if possible.

# Test info

- Name: onboarding.spec.ts >> Onboarding Wizard >> étape 1 — choix de langue affiche 4 options
- Location: tests/e2e/onboarding.spec.ts:31:2

# Error details

```
Error: expect(locator).toHaveCount(expected) failed

Locator:  locator('button:has-text("Français"), button:has-text("Deutsch"), button:has-text("Italiano"), button:has-text("English")')
Expected: 4
Received: 0
Timeout:  5000ms

Call log:
  - Expect "toHaveCount" with timeout 5000ms
  - waiting for locator('button:has-text("Français"), button:has-text("Deutsch"), button:has-text("Italiano"), button:has-text("English")')
    9 × locator resolved to 0 elements
      - unexpected value "0"

```

```
Error: page.evaluate: SecurityError: Failed to read the 'localStorage' property from 'Window': Access is denied for this document.
    at UtilityScript.evaluate (<anonymous>:304:16)
    at UtilityScript.<anonymous> (<anonymous>:1:44)
```