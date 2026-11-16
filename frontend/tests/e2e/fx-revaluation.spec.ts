/**
 * Story 37-5 (AC-6, AC-7, AC-9) — la réévaluation de fin d'exercice des comptes
 * en devise, de la base à l'écran.
 *
 * Société passée en multi-devises par l'API, un compte USD, un exercice 2025
 * (fin passée : le cours du jour se récupère) où l'on écrit USD 1000 à 0.85.
 * Le cours mensuel de décembre 2025 est récupéré (source factice : 0.8800) ; le
 * journalier du 31.12.2025 ne l'est pas encore.
 *
 * 1. Écran « Réévaluer les devises » : repli mensuel signalé → « Récupérer le
 *    cours du 31.12.2025 » (source factice : 0.8912) → aperçu relu au cours du
 *    jour, le champ non modifié ayant repris le défaut → « Passer les écritures ».
 * 2. Une écriture tardive : « Préparer la clôture » montre K9 en attention avec
 *    l'aperçu, la valeur cumulée déjà passée et le lien vers sa première
 *    écriture → « Réévaluer » → K9 « Rien à signaler », aucun aperçu → la
 *    clôture est permise.
 *
 * L'activation est irréversible : la spec repart d'un `with-company` neuf et en
 * reseede un en fin de parcours. Captures (thème clair) dans
 * `KESH_E2E_CAPTURES_DIR` s'il est posé.
 */

import path from 'node:path';
import { expect, test, type APIRequestContext, type Page, type TestInfo } from '@playwright/test';
import {
	seedTestState,
	clearAuthStorage,
	authedApiContext,
	disposeContextSafe,
} from './helpers/test-state';

test.describe.configure({ mode: 'serial' });

// Sans exercice : l'exercice 2025 se crée par l'API, après l'activation (le
// preset `with-company` porte un exercice 2020–2030 qui le chevaucherait).
test.beforeAll(async () => {
	await seedTestState('with-company-no-fy');
});

test.afterAll(async () => {
	await seedTestState('with-company');
});

test.afterEach(async ({ page }) => {
	await clearAuthStorage(page);
});

function capturePath(testInfo: TestInfo, name: string): string {
	const dossier = process.env.KESH_E2E_CAPTURES_DIR;
	return dossier ? path.join(dossier, `${name}.png`) : testInfo.outputPath(`${name}.png`);
}

async function login(page: Page): Promise<void> {
	await page.goto('/login');
	await page.fill('#username', 'admin');
	await page.fill('#password', 'admin123');
	await page.click('button[type="submit"]');
	await expect(page).toHaveURL('/');
}

async function createAccount(ctx: APIRequestContext, data: Record<string, unknown>): Promise<number> {
	const res = await ctx.post('/api/v1/accounts', { data });
	expect(res.ok(), `compte ${data.number} : ${res.status()} ${await res.text()}`).toBeTruthy();
	return ((await res.json()) as { id: number }).id;
}

async function accountId(ctx: APIRequestContext, number: string): Promise<number> {
	const res = await ctx.get('/api/v1/accounts');
	const found = ((await res.json()) as { id: number; number: string }[]).find(
		(a) => a.number === number,
	);
	expect(found, `compte ${number}`).toBeTruthy();
	return found!.id;
}

async function writeUsd(
	ctx: APIRequestContext,
	usd: number,
	capital: number,
	date: string,
	amount: string,
	chf: string,
	rate: string,
): Promise<void> {
	const res = await ctx.post('/api/v1/journal-entries', {
		data: {
			entryDate: date,
			journal: 'OD',
			description: `E2E 37-5 — USD ${amount}`,
			lines: [
				{ accountId: usd, debit: chf, credit: '0', foreign: { amount, rate, rateSource: 'manual' } },
				{ accountId: capital, debit: '0', credit: chf },
			],
		},
	});
	expect(res.ok(), `écriture : ${res.status()} ${await res.text()}`).toBeTruthy();
}

let fyId = 0;
let usdId = 0;
let capitalId = 0;

test('écran de réévaluation : repli mensuel, récupération du cours du jour, passation', async ({
	page,
}, testInfo) => {
	await login(page);
	const ctx = await authedApiContext(page);
	try {
		const transit = await createAccount(ctx, { number: '1092', name: 'Conversion', accountType: 'Asset' });
		const gain = await createAccount(ctx, { number: '6951', name: 'Gains de change', accountType: 'Revenue' });
		const loss = await createAccount(ctx, { number: '6941', name: 'Pertes de change', accountType: 'Expense' });
		const company = await ctx.get('/api/v1/companies/current');
		const version = ((await company.json()) as { company: { version: number } }).company.version;
		const activate = await ctx.put('/api/v1/companies/current/currency-settings', {
			data: {
				multiCurrencyEnabled: true,
				fxGainAccountId: gain,
				fxLossAccountId: loss,
				fxTransitAccountId: transit,
				version,
			},
		});
		expect(activate.ok(), `activation : ${activate.status()} ${await activate.text()}`).toBeTruthy();
		usdId = await createAccount(ctx, {
			number: '1027',
			name: 'Banque USD',
			accountType: 'Asset',
			currency: 'USD',
		});
		capitalId = await accountId(ctx, '2000');
		const fy = await ctx.post('/api/v1/fiscal-years', {
			data: { name: 'Exercice 2025 E2E', startDate: '2025-01-01', endDate: '2025-12-31' },
		});
		expect(fy.ok(), `exercice : ${fy.status()} ${await fy.text()}`).toBeTruthy();
		fyId = ((await fy.json()) as { id: number }).id;
		// Le cours mensuel de décembre 2025 (source factice : USD 0.8800).
		const month = await ctx.post('/api/v1/exchange-rates/fetch', { data: { year: 2025, month: 12 } });
		expect(month.ok(), `cours de décembre : ${month.status()} ${await month.text()}`).toBeTruthy();
		await writeUsd(ctx, usdId, capitalId, '2025-06-15', '1000', '850.00', '0.85');
	} finally {
		await disposeContextSafe(ctx);
	}

	await page.goto('/settings/fiscal-years');
	await page.getByTestId(`fiscal-year-revaluation-${fyId}`).click();
	const dialog = page.getByTestId('fiscal-year-revaluation-dialog');
	await expect(dialog.getByTestId('revaluation-rate-kind-USD')).toContainText(
		'cours du jour de clôture indisponible',
	);
	await expect(dialog.getByTestId(`revaluation-delta-${usdId}`)).toContainText('30.00');
	await dialog.screenshot({ path: capturePath(testInfo, '37-5-ecran-repli-mensuel') });

	// Récupérer le cours du jour (source factice : USD 0.89120).
	await dialog.getByTestId('revaluation-fetch-day-USD').click();
	await expect(dialog.getByTestId('revaluation-rate-kind-USD')).toHaveText('cours du 31.12.2025');
	await expect(dialog.getByTestId('revaluation-rate-input-USD')).toHaveValue(/^0\.8912/);
	// 1000 × 0.8912 = 891.20 − 850 = +41.20, au compte de gains.
	await expect(dialog.getByTestId(`revaluation-delta-${usdId}`)).toContainText('41.20');
	await expect(dialog.getByTestId(`revaluation-entry-${usdId}`)).toContainText('6951 Gains de change');
	await dialog.screenshot({ path: capturePath(testInfo, '37-5-ecran-cours-du-jour') });

	await dialog.getByTestId('revaluation-submit').click();
	await expect(dialog.getByTestId('revaluation-posted')).toContainText('1 écriture(s) passée(s)');
	// Relu : plus d'écart, la réévaluation passée et sa valeur cumulée.
	await expect(dialog.getByTestId(`revaluation-delta-${usdId}`)).toContainText('0.00');
	await expect(dialog.getByTestId(`revaluation-previous-${usdId}`)).toContainText('41.20');
	await dialog.screenshot({ path: capturePath(testInfo, '37-5-ecran-apres-passation') });
	await dialog.getByTestId('fiscal-year-revaluation-close').click();
});

test('K9 : avis avec l’aperçu, « Réévaluer », contrôle levé, clôture permise', async ({
	page,
}, testInfo) => {
	await login(page);
	const ctx = await authedApiContext(page);
	try {
		// Écriture tardive : USD 100 à 0.80 (80.00).
		await writeUsd(ctx, usdId, capitalId, '2025-12-20', '100', '80.00', '0.80');
	} finally {
		await disposeContextSafe(ctx);
	}

	await page.goto('/settings/fiscal-years');
	await page.getByTestId(`fiscal-year-checks-${fyId}`).click();
	const dialog = page.getByTestId('fiscal-year-checks-dialog');
	const k9 = dialog.getByTestId('fiscal-year-checks-row-FOREIGN_ACCOUNTS_NOT_REVALUED');
	await expect(k9).toHaveAttribute('data-status', 'attention');
	await expect(k9).toContainText('Bloquant');
	const apercu = k9.getByTestId('fiscal-year-checks-k9-preview');
	await expect(apercu.getByTestId('revaluation-panel')).toBeVisible();
	// 1100 × 0.8912 = 980.32 ; 850 + 41.20 + 80 = 971.20 → +9.12.
	await expect(apercu.getByTestId(`revaluation-delta-${usdId}`)).toContainText('9.12');
	await expect(apercu.getByTestId(`revaluation-previous-${usdId}`)).toContainText('41.20');
	await expect(apercu.getByTestId(`revaluation-after-${usdId}`)).toContainText('50.32');
	const premiere = apercu.locator('[data-testid^="revaluation-previous-entry-"]').first();
	await expect(premiere).toHaveAttribute('href', /^\/journal-entries\/\d+$/);
	// La ligne K9 seule : le panneau défile, sa capture n'en montrerait que le haut.
	await k9.scrollIntoViewIfNeeded();
	await k9.screenshot({ path: capturePath(testInfo, '37-5-k9-avis-apercu') });

	await apercu.getByTestId('revaluation-submit').click();
	await expect(
		dialog.getByTestId('fiscal-year-checks-row-FOREIGN_ACCOUNTS_NOT_REVALUED'),
	).toHaveAttribute('data-status', 'ok');
	await expect(
		dialog.getByTestId('fiscal-year-checks-status-FOREIGN_ACCOUNTS_NOT_REVALUED'),
	).toHaveText('Rien à signaler');
	await expect(dialog.getByTestId('fiscal-year-checks-k9-preview')).toHaveCount(0);
	const k9Leve = dialog.getByTestId('fiscal-year-checks-row-FOREIGN_ACCOUNTS_NOT_REVALUED');
	await k9Leve.scrollIntoViewIfNeeded();
	await k9Leve.screenshot({ path: capturePath(testInfo, '37-5-k9-leve') });
	await dialog.getByTestId('fiscal-year-checks-close').click();

	// La clôture est permise.
	await page.getByTestId(`fiscal-year-close-${fyId}`).click();
	// Le rappel lu (seul l'exercice suivant manque, sans bloquer), sans K9.
	await expect(
		page.getByTestId('fiscal-year-close-checks-point-NEXT_FISCAL_YEAR'),
	).toBeVisible();
	await expect(
		page.getByTestId('fiscal-year-close-checks-point-FOREIGN_ACCOUNTS_NOT_REVALUED'),
	).toHaveCount(0);
	await expect(page.getByTestId('fiscal-year-close-blocked-k9')).toHaveCount(0);
	const confirmer = page.getByTestId('fiscal-year-close-confirm');
	await expect(confirmer).toBeEnabled();
	await confirmer.click();
	await expect(page.getByTestId(`fiscal-year-row-${fyId}`)).toContainText('Clôturé');
});
