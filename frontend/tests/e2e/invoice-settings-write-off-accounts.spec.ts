/**
 * Story 25-4-d1 (#384) — les comptes des natures d'écart soldé, désignés dans
 * Paramètres → Facturation, section « Solde du reste ».
 *
 * Choisir un compte pour chacune des trois natures, enregistrer, recharger la
 * page : les trois valeurs reviennent du serveur. C'est le seul test qui voit
 * les trois champs traverser réellement la frontière HTTP — Vitest teste le
 * payload, les tests Rust la validation. Le test remet les réglages à vide à la
 * fin : la base est partagée entre specs.
 *
 * Pré-requis : MariaDB up + KESH_TEST_MODE=true (cf. docs/testing.md).
 */

import { expect, test, type Page } from '@playwright/test';
import { seedTestState, clearAuthStorage } from './helpers/test-state';

const SELECTS = [
	'settings-discount-account',
	'settings-bank-fees-account',
	'settings-bad-debt-account',
] as const;

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

async function saveSettings(page: Page): Promise<void> {
	const saved = page.waitForResponse(
		(r) => r.url().includes('/api/v1/company/invoice-settings') && r.request().method() === 'PUT',
	);
	await page.getByTestId('settings-invoicing-save').click();
	expect((await saved).status()).toBe(200);
}

/** La valeur de la première option non vide d'un sélecteur. */
async function firstAccountValue(page: Page, testid: string): Promise<string> {
	// Les comptes arrivent après le rendu (requête de `onMount`) : attendre une
	// option au-delà de « — Sélectionner — » avant de lire.
	await expect(page.getByTestId(testid).locator('option').nth(1)).toBeAttached();
	const values = await page
		.getByTestId(testid)
		.locator('option')
		.evaluateAll((options) => options.map((o) => (o as HTMLOptionElement).value));
	const value = values.find((v) => v !== '' && v !== 'null');
	expect(value, `${testid} : aucun compte proposé`).toBeTruthy();
	return value!;
}

test('les comptes du solde du reste sont enregistrés et relus', async ({ page }) => {
	await login(page);
	await page.goto('/settings/invoicing');
	try {
		const chosen: string[] = [];
		for (const testid of SELECTS) {
			const value = await firstAccountValue(page, testid);
			await page.getByTestId(testid).selectOption(value);
			chosen.push(value);
		}
		await saveSettings(page);

		await page.reload();
		for (const [i, testid] of SELECTS.entries()) {
			await expect(page.getByTestId(testid)).toHaveValue(chosen[i]);
		}
	} finally {
		// Remise à vide : la base est partagée entre specs.
		await page.goto('/settings/invoicing');
		for (const testid of SELECTS) {
			await page.getByTestId(testid).selectOption({ index: 0 });
		}
		await saveSettings(page);
	}
});
