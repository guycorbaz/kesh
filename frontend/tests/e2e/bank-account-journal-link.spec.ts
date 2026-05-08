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
 *     bank_account ; on le crée via `/api/v1/onboarding/bank-account`.
 *   - Playwright browsers installés (Ubuntu 26.04+ :
 *     PLAYWRIGHT_HOST_PLATFORM_OVERRIDE=ubuntu24.04-x64).
 */

import { expect, test, type Page } from '@playwright/test';
import AxeBuilder from '@axe-core/playwright';
import { seedTestState, clearAuthStorage } from './helpers/test-state';

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

async function authHeaders(page: Page): Promise<Record<string, string>> {
	const token = await page.evaluate(() => {
		try {
			return localStorage.getItem('kesh:auth:accessToken');
		} catch {
			return null;
		}
	});
	if (!token) {
		throw new Error('JWT introuvable post-login');
	}
	return { Authorization: `Bearer ${token}` };
}

async function ensureBankAccount(page: Page): Promise<number> {
	const headers = await authHeaders(page);
	const company = await page.request.get('/api/v1/companies/current', { headers });
	if (company.ok()) {
		const json = await company.json();
		if (Array.isArray(json.bankAccounts) && json.bankAccounts.length > 0) {
			return json.bankAccounts[0].id as number;
		}
	}
	const res = await page.request.post('/api/v1/onboarding/bank-account', {
		headers,
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
