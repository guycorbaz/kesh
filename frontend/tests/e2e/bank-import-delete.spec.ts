/**
 * Story 27-1 (#632) — supprimer un import bancaire dont aucune transaction
 * n'est rapprochée, puis réimporter le même fichier.
 *
 * Un scénario : profil CSV monté par l'API, import par l'écran, suppression
 * depuis le détail, réimport du même fichier sans l'avertissement « fichier
 * déjà importé ». La fixture `csv_e2e_delete_import.csv` est SYNTHÉTIQUE.
 *
 * Pré-requis : cf. `docs/testing.md` § « Prérequis Playwright local ».
 */

import { expect, test, type Page } from '@playwright/test';
import path from 'path';
import { fileURLToPath } from 'url';
import {
	seedTestState,
	clearAuthStorage,
	authedApiContext,
	disposeContextSafe,
} from './helpers/test-state';

const FIXTURE_DIR = path.join(path.dirname(fileURLToPath(import.meta.url)), 'fixtures');
const FILENAME = 'csv_e2e_delete_import.csv';
const FIXTURE = path.join(FIXTURE_DIR, FILENAME);
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

/** Compte bancaire et profil CSV du scénario, montés par l'API. */
async function mount(page: Page): Promise<void> {
	const ctx = await authedApiContext(page);
	try {
		const company = await ctx.get('/api/v1/companies/current');
		const json = company.ok() ? await company.json() : {};
		if (!Array.isArray(json.bankAccounts) || json.bankAccounts.length === 0) {
			const res = await ctx.post('/api/v1/bank-accounts', {
				data: { bankName: 'UBS Test', iban: TEST_IBAN, qrIban: null, isPrimary: true },
			});
			expect(res.ok(), `bank-account create failed: ${res.status()}`).toBeTruthy();
		}
		// Corps recopié du formulaire réel (`BankProfileForm.svelte`) ; `debitSign`
		// est exigé depuis la v0.13.1.
		const profile = await ctx.post('/api/v1/bank-profiles', {
			data: {
				bankName: 'E2E suppression',
				filenamePattern: '^csv_e2e_delete_import\\.csv$',
				columnMapping: { date: 0, amount: 1, reference: 2, details: 3 },
				dateFormat: '%Y-%m-%d',
				decimalSeparator: '.',
				fieldSeparator: ';',
				encoding: null,
				headerRowCount: 1,
				debitSign: 'positive',
			},
		});
		expect(
			profile.status() === 201 || profile.status() === 409,
			`bank-profile create failed: ${profile.status()} ${await profile.text()}`,
		).toBeTruthy();
	} finally {
		await disposeContextSafe(ctx);
	}
}

async function importByScreen(page: Page): Promise<void> {
	await page.goto('/bank-import');
	await expect(page.getByTestId('bank-import-page-title')).toBeVisible();
	await page.getByTestId('bank-account-select').selectOption({ index: 1 });
	await page.getByTestId('bank-import-file-input').setInputFiles(FIXTURE);
	await expect(page.getByTestId('bank-import-preview')).toBeVisible();
}

test('supprimer un import non rapproché puis réimporter le même fichier', async ({ page }) => {
	await login(page);
	await mount(page);

	// (2) Import par l'écran.
	await importByScreen(page);
	await page.getByTestId('bank-import-confirm').click();
	await expect(page.getByTestId('bank-import-preview')).toBeHidden();
	const row = page.getByTestId('bank-import-list-row').filter({ hasText: FILENAME });
	await expect(row).toHaveCount(1);
	await row.getByTestId('bank-import-list-link').click();
	await expect(page).toHaveURL(/\/bank-import\/\d+$/);

	// (3) Suppression depuis le détail.
	await page.getByTestId('bank-import-delete-button').click();
	await expect(page.getByTestId('bank-import-delete-dialog')).toBeVisible();
	const body = page.getByTestId('bank-import-delete-dialog-body');
	await expect(body).toContainText(FILENAME);
	// Le nombre de transactions, pas un « 3 » de la période (2026-03-…).
	await expect(body).toContainText(/ses 3 transaction\(s\)/);
	await page.getByTestId('bank-import-delete-submit').click();
	await expect(page).toHaveURL(/\/bank-import$/);
	await expect(page.getByTestId('bank-import-list')).toBeVisible();
	await expect(
		page.getByTestId('bank-import-list-row').filter({ hasText: FILENAME }),
	).toHaveCount(0);

	// (4) Le même fichier : plus d'avertissement de doublon, et il se réimporte.
	await importByScreen(page);
	await expect(page.getByTestId('warning-duplicate-file')).toHaveCount(0);
	await page.getByTestId('bank-import-confirm').click();
	await expect(page.getByTestId('bank-import-preview')).toBeHidden();
	const again = page.getByTestId('bank-import-list-row').filter({ hasText: FILENAME });
	await expect(again).toHaveCount(1);
	await again.getByTestId('bank-import-list-link').click();
	await expect(page.getByTestId('detail-tx-row')).toHaveCount(3);
});
