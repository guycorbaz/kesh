/**
 * Montage d'un import bancaire par l'écran — extrait de
 * `bank-import-delete.spec.ts` (Story 27-1) par la Story 28-1 (#640, AC8,
 * C-28-1-16), pour que `destructive-buttons.spec.ts` le partage.
 *
 * Une spec n'importe pas une autre spec : Playwright y enregistrerait ses tests
 * une seconde fois. D'où ce module de helpers.
 *
 * Profil CSV monté par l'API, import par l'écran. La fixture
 * `csv_e2e_delete_import.csv` est SYNTHÉTIQUE.
 */

import { expect, type Page } from '@playwright/test';
import path from 'path';
import { fileURLToPath } from 'url';
import { authedApiContext, disposeContextSafe } from './test-state';

const FIXTURE_DIR = path.join(path.dirname(fileURLToPath(import.meta.url)), '..', 'fixtures');

/** Nom du fichier synthétique importé. */
export const DELETE_IMPORT_FILENAME = 'csv_e2e_delete_import.csv';
const FIXTURE = path.join(FIXTURE_DIR, DELETE_IMPORT_FILENAME);
const TEST_IBAN = 'CH4431999123000889012';

/** Connexion de l'administrateur du seed. */
export async function login(page: Page): Promise<void> {
	await page.goto('/login');
	await page.fill('#username', 'admin');
	await page.fill('#password', 'admin123');
	await page.click('button[type="submit"]');
	await expect(page).toHaveURL('/');
}

/** Compte bancaire et profil CSV du scénario, montés par l'API. */
export async function mount(page: Page): Promise<void> {
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

/** Ouvre l'écran d'import, choisit le compte et le fichier ; s'arrête à l'aperçu. */
export async function importByScreen(page: Page): Promise<void> {
	await page.goto('/bank-import');
	await expect(page.getByTestId('bank-import-page-title')).toBeVisible();
	await page.getByTestId('bank-account-select').selectOption({ index: 1 });
	await page.getByTestId('bank-import-file-input').setInputFiles(FIXTURE);
	await expect(page.getByTestId('bank-import-preview')).toBeVisible();
}
