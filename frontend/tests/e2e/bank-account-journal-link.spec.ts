/**
 * Story 8-5a-zero — Tests E2E Playwright pour la page de configuration
 * `/bank-accounts`.
 *
 * 2 scénarios :
 *   1. Lier un bank_account au compte 1100 Banque CI (AC #78 UI)
 *   2. Accessibility — axe scan zero violations sur la modal/form (AC #82)
 *
 * Pré-requis :
 *   - MariaDB up + KESH_TEST_MODE=true sur le backend.
 *   - Le seed `with-company` crée les comptes 1000-4000 mais pas de
 *     bank_account ; on le crée via `POST /api/v1/bank-accounts` (route CRUD).
 *   - Playwright browsers installés (Ubuntu 26.04+ :
 *     PLAYWRIGHT_HOST_PLATFORM_OVERRIDE=ubuntu24.04-x64).
 */

import { expect, test, type Page } from '@playwright/test';
import AxeBuilder from '@axe-core/playwright';
import {
	seedTestState,
	clearAuthStorage,
	authedApiContext,
	disposeContextSafe,
} from './helpers/test-state';

const TEST_IBAN = 'CH4431999123000889012';

test.beforeAll(async () => {
	await seedTestState('with-company');
});

test.afterEach(async ({ page }) => {
	await clearAuthStorage(page);
});

async function login(page: Page): Promise<void> {
	await page.goto('/login');
	await page.fill('#username', 'admin');
	await page.fill('#password', 'admin123');
	await page.click('button[type="submit"]');
	await expect(page).toHaveURL('/');
}

/**
 * Monte le compte bancaire du scénario via l'API.
 *
 * ⚠️ Passe par `authedApiContext(page)` et NON par `page.request.*` avec un
 * bearer lu en `localStorage` : depuis la Story 10-5, le JWT vit dans un
 * **cookie HttpOnly** inaccessible au JS, et `readAccessTokenFromStorage` est
 * marqué `@deprecated` — il rend toujours `null` en flux normal. Cette spec
 * avait gardé son helper maison et échouait donc au MONTAGE, sur
 * « JWT introuvable post-login », sans jamais atteindre ce qu'elle teste
 * (issue #107, KF-030).
 */
async function ensureBankAccount(page: Page): Promise<number> {
	const ctx = await authedApiContext(page);
	try {
		const company = await ctx.get('/api/v1/companies/current');
		if (company.ok()) {
			const json = await company.json();
			if (Array.isArray(json.bankAccounts) && json.bankAccounts.length > 0) {
				return json.bankAccounts[0].id as number;
			}
		}
		// ⚠️ Route CRUD, et NON `/api/v1/onboarding/bank-account` : cette dernière
		// refuse désormais en 400 `ONBOARDING_STEP_ALREADY_COMPLETED` sur le seed
		// `with-company`, qui marque l'étape franchie SANS créer de compte. Monter
		// un décor par une route d'onboarding était un abus qui s'est retourné le
		// jour où elle a gagné sa garde (issue #107, KF-030).
		const res = await ctx.post('/api/v1/bank-accounts', {
			data: {
				bankName: 'UBS Test',
				iban: TEST_IBAN,
				qrIban: null,
				isPrimary: true,
			},
		});
		expect(res.ok(), `bank-account create failed: ${res.status()}`).toBeTruthy();
		const created = await res.json();
		return created.id ?? 1;
	} finally {
		await disposeContextSafe(ctx);
	}
}

test('bank-account journal link end-to-end', async ({ page }) => {
	await login(page);
	await ensureBankAccount(page);

	await page.goto('/bank-accounts');
	await expect(page.getByTestId('bank-accounts-page-title')).toBeVisible();
	await expect(page.getByTestId('bank-accounts-list')).toBeVisible();

	// Click sur « Lier ».
	const firstRow = page.locator('[data-testid^="bank-account-row-"]').first();
	const rowId = await firstRow.getAttribute('data-testid');
	expect(rowId).toBeTruthy();
	const linkBtn = firstRow.locator('[data-testid^="link-button-"]');
	await linkBtn.click();

	// Le formulaire est ouvert.
	await expect(page.getByTestId('bank-account-journal-link-form')).toBeVisible();
	await expect(page.getByTestId('journal-account-select')).toBeVisible();

	// Sélectionne le compte 1100 Banque CI (Asset, classe 1).
	const select = page.getByTestId('journal-account-select');
	const options = await select.locator('option').allTextContents();
	const banqueOption = options.find((o) => o.includes('1100'));
	expect(banqueOption, 'Account 1100 must be in dropdown').toBeTruthy();
	await select.selectOption({ label: banqueOption! });

	// Submit.
	await page.getByTestId('submit-link').click();

	// La page recharge l'état (le composant met à jour la liste in-place) —
	// vérifier que la cellule du compte comptable affiche maintenant 1100.
	await expect(
		page.locator('[data-testid^="journal-account-cell-"]').first(),
	).toContainText('1100');
});

test('accessibility — bank-account-journal-link form axe scan', async ({ page }) => {
	await login(page);
	await ensureBankAccount(page);

	await page.goto('/bank-accounts');
	await expect(page.getByTestId('bank-accounts-page-title')).toBeVisible();
	await expect(page.getByTestId('bank-accounts-list')).toBeVisible();

	// Ouvrir le form.
	const firstRow = page.locator('[data-testid^="bank-account-row-"]').first();
	await firstRow.locator('[data-testid^="link-button-"]').click();
	await expect(page.getByTestId('bank-account-journal-link-form')).toBeVisible();

	const results = await new AxeBuilder({ page })
		.include('[data-testid="bank-account-journal-link-form"]')
		.analyze();
	expect(
		results.violations,
		`axe violations: ${JSON.stringify(results.violations, null, 2)}`,
	).toEqual([]);
});
