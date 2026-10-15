import { expect, test, type APIRequestContext, type Page, type TestInfo } from '@playwright/test';
import fs from 'fs';
import path from 'path';
import { fileURLToPath } from 'url';
import {
	seedTestState,
	clearAuthStorage,
	authedApiContext,
	disposeContextSafe,
} from './helpers/test-state';

/**
 * Story 29-1b (#232, AC18) — le parcours complet de la première clôture : préparer,
 * régulariser, contre-passer au début de l'exercice suivant, clôturer, rouvrir.
 *
 * C'est le seul test qui fait traverser à la contre-passation datée la frontière HTTP
 * jusqu'à l'écran, et qui voit les contrôles de la 29-1a changer d'état sous l'effet
 * des gestes de l'utilisateur.
 *
 * Fixtures **synthétiques** (étape 0) : `with-company-no-fy` vide la base et ne pose que
 * cinq comptes (`1000` à `4000`), sans exercice ni compte bancaire. La spec crée :
 * l'exercice N = 2025 (premier de la société : K1 `ok`) et N+1 = 2026 ; `1300 Actifs
 * transitoires` ; `1020 Banque`, compte de grand livre lié, non partagé (⛔ pas `1100`,
 * compte débiteurs par défaut, refusé comme compte lié ; ni `1000`, compte d'impôt
 * préalable) ; un compte bancaire (IBAN de test) lié à `1020` ; le relevé CAMT
 * `camt053_v04_year_end.xml` (01.12.2025 → 31.12.2025, ouverture 0.00, une entrée de
 * 100.00, clôture 100.00). Aucune facture en brouillon ni pièce à compléter (K4, K5).
 *
 * Années fixes : rien ne dépend du jour d'exécution (aucun geste daté du jour n'est exercé ;
 * la régularisation et le relevé sont de 2025, N+1 est 2026 quelle que soit l'année courante).
 */

const CAMT_FIXTURE = path.join(
	path.dirname(fileURLToPath(import.meta.url)),
	'fixtures',
	'camt053_v04_year_end.xml',
);
const TEST_IBAN = 'CH4431999123000889012';

/**
 * Recette visuelle (hors dépôt, sans référence de comparaison) : capture en clair puis
 * en sombre, dans `KESH_E2E_CAPTURES_DIR` s'il est posé, sinon dans la sortie du test.
 */
async function captureBothSchemes(page: Page, testInfo: TestInfo, name: string) {
	for (const [colorScheme, suffixe] of [
		['light', 'clair'],
		['dark', 'sombre'],
	] as const) {
		await page.emulateMedia({ colorScheme });
		const dossier = process.env.KESH_E2E_CAPTURES_DIR;
		const file = `${name}-${suffixe}.png`;
		await page.screenshot({
			path: dossier ? path.join(dossier, file) : testInfo.outputPath(file),
		});
	}
	await page.emulateMedia({ colorScheme: 'light' });
}

test.beforeEach(async () => {
	await seedTestState('with-company-no-fy');
});

test.afterEach(async ({ page }) => {
	await clearAuthStorage(page);
});

async function login(page: Page) {
	await page.goto('/login');
	await page.fill('#username', 'admin');
	await page.fill('#password', 'admin123');
	await page.click('button[type="submit"]');
	await expect(page).toHaveURL('/');
}

async function createFiscalYear(ctx: APIRequestContext, year: number): Promise<number> {
	const res = await ctx.post('/api/v1/fiscal-years', {
		data: { name: `Exercice ${year}`, startDate: `${year}-01-01`, endDate: `${year}-12-31` },
	});
	expect(res.status(), await res.text()).toBe(201);
	return ((await res.json()) as { id: number }).id;
}

async function createAccount(
	ctx: APIRequestContext,
	number: string,
	name: string,
): Promise<number> {
	const res = await ctx.post('/api/v1/accounts', {
		data: { number, name, accountType: 'Asset' },
	});
	expect(res.status(), await res.text()).toBe(201);
	return ((await res.json()) as { id: number }).id;
}

async function accountId(ctx: APIRequestContext, number: string): Promise<number> {
	const accounts = (await (await ctx.get('/api/v1/accounts')).json()) as Array<{
		id: number;
		number: string;
	}>;
	const a = accounts.find((x) => x.number === number);
	expect(a, `compte ${number} du préréglage`).toBeTruthy();
	return a!.id;
}

/** Saisit la régularisation de fin d'exercice (débit 1300, crédit 4000), au 31.12.2025. */
async function regularisation(
	ctx: APIRequestContext,
	transitoire: number,
	charge: number,
): Promise<number> {
	const res = await ctx.post('/api/v1/journal-entries', {
		data: {
			entryDate: '2025-12-31',
			journal: 'OD',
			description: 'Charge payée d’avance (transitoire)',
			lines: [
				{ accountId: transitoire, debit: '250.00', credit: '0' },
				{ accountId: charge, debit: '0', credit: '250.00' },
			],
		},
	});
	expect(res.status(), await res.text()).toBe(201);
	return ((await res.json()) as { id: number }).id;
}

type Check = {
	code: string;
	status: string;
	details: { accounts?: Array<{ reason?: string; count?: number }> };
};

async function checks(ctx: APIRequestContext, fyId: number): Promise<Map<string, Check>> {
	const res = await ctx.get(`/api/v1/fiscal-years/${fyId}/closing-checks`);
	expect(res.status()).toBe(200);
	const body = (await res.json()) as { checks: Check[] };
	return new Map(body.checks.map((c) => [c.code, c]));
}

/** Le solde d'un compte au bilan arrêté à `periodEnd`, dans l'exercice `fyId`. */
async function balanceOf(
	ctx: APIRequestContext,
	fyId: number,
	periodEnd: string,
	number: string,
): Promise<string> {
	const res = await ctx.get(
		`/api/v1/reports/balance-sheet?fiscalYearId=${fyId}&periodEnd=${periodEnd}`,
	);
	expect(res.status(), await res.text()).toBe(200);
	const sheet = (await res.json()) as {
		assets: Array<{ accountNumber: string; balance: string }>;
	};
	const line = sheet.assets.find((a) => a.accountNumber === number);
	return line ? line.balance : '0';
}

/** Ouvre « Préparer la clôture » de l'exercice, attend les sept lignes. */
async function openChecks(page: Page, fyId: number) {
	await page.goto('/settings/fiscal-years');
	await page.getByTestId(`fiscal-year-checks-${fyId}`).click();
	const dialog = page.getByTestId('fiscal-year-checks-dialog');
	await expect(dialog.locator('[data-testid^="fiscal-year-checks-row-"]')).toHaveCount(7);
	return dialog;
}

test('parcours de la première clôture : préparer, régulariser, contre-passer, clôturer, rouvrir', async ({
	page,
}, testInfo) => {
	await login(page);
	const ctx = await authedApiContext(page);
	try {
		// 0-1. Exercices, comptes, compte bancaire lié, relevé de fin d'année.
		const n = await createFiscalYear(ctx, 2025);
		const n1 = await createFiscalYear(ctx, 2026);
		const transitoire = await createAccount(ctx, '1300', 'Actifs transitoires');
		const banque = await createAccount(ctx, '1020', 'Banque');
		const charge = await accountId(ctx, '4000');
		const ventes = await accountId(ctx, '3000');

		const created = await ctx.post('/api/v1/bank-accounts', {
			data: { bankName: 'Banque Test', iban: TEST_IBAN, qrIban: null, isPrimary: true },
		});
		expect(created.ok(), `bank-account create: ${created.status()}`).toBeTruthy();
		type Ba = { id: number; iban: string; version: number };
		const ba = ((await (await ctx.get('/api/v1/bank-accounts')).json()) as Ba[]).find(
			(b) => b.iban === TEST_IBAN,
		)!;
		const wired = await ctx.patch(`/api/v1/bank-accounts/${ba.id}`, {
			data: { journalAccountId: banque, version: ba.version },
		});
		expect(wired.ok(), `bank-account wiring: ${wired.status()} ${await wired.text()}`).toBeTruthy();

		const imported = await ctx.post('/api/v1/bank-imports', {
			multipart: {
				bankAccountId: ba.id.toString(),
				file: {
					name: 'year-end-2025.xml',
					mimeType: 'application/xml',
					buffer: fs.readFileSync(CAMT_FIXTURE),
				},
			},
		});
		expect(imported.status(), await imported.text()).toBe(201);
		const importId = ((await imported.json()) as { id: number }).id;
		const detail = (await (await ctx.get(`/api/v1/bank-imports/${importId}`)).json()) as {
			transactions: Array<{ id: number }>;
		};
		expect(detail.transactions).toHaveLength(1);
		const txId = detail.transactions[0].id;

		// 2. Préparer la clôture de N : K2 et K3 en attention, K7 ok.
		let dialog = await openChecks(page, n);
		await expect(dialog.getByTestId('fiscal-year-checks-row-PENDING_BANK_TRANSACTIONS')).toHaveAttribute(
			'data-status',
			'attention',
		);
		await expect(dialog.getByTestId('fiscal-year-checks-row-BANK_STATEMENT_AT_YEAR_END')).toHaveAttribute(
			'data-status',
			'attention',
		);
		await expect(dialog.getByTestId('fiscal-year-checks-row-NEXT_FISCAL_YEAR')).toHaveAttribute(
			'data-status',
			'ok',
		);
		const avant = await checks(ctx, n);
		expect(avant.get('PENDING_BANK_TRANSACTIONS')!.status).toBe('attention');
		// AC18 : K2 en attention, avec UNE transaction en attente (revue P1, L4).
		expect(avant.get('PENDING_BANK_TRANSACTIONS')!.details.accounts?.[0]?.count).toBe(1);
		expect(avant.get('BANK_STATEMENT_AT_YEAR_END')!.details.accounts?.[0]?.reason).toBe('GAP');
		expect(avant.get('EARLIER_FISCAL_YEAR_OPEN')!.status).toBe('ok');

		// 3. Rapprocher la transaction (écriture au débit du compte lié) ; K2 et K3 ok.
		const matched = await ctx.post('/api/v1/reconciliation/manual', {
			data: {
				bankAccountId: ba.id,
				bankTransactionId: txId,
				counterpartyAccountId: ventes,
				description: 'Prestation décembre',
			},
		});
		expect(matched.status(), await matched.text()).toBe(200);
		await dialog.getByTestId('fiscal-year-checks-close').click();
		dialog = await openChecks(page, n);
		await expect(dialog.getByTestId('fiscal-year-checks-row-PENDING_BANK_TRANSACTIONS')).toHaveAttribute(
			'data-status',
			'ok',
		);
		await expect(dialog.getByTestId('fiscal-year-checks-row-BANK_STATEMENT_AT_YEAR_END')).toHaveAttribute(
			'data-status',
			'ok',
		);
		await dialog.getByTestId('fiscal-year-checks-close').click();

		// 4. La régularisation au 31.12.2025, puis « Contre-passer au début de l'exercice suivant ».
		const origine = await regularisation(ctx, transitoire, charge);
		const origineNumber = (
			(await (await ctx.get(`/api/v1/journal-entries/${origine}`)).json()) as {
				entryNumber: number;
			}
		).entryNumber;
		await page.goto(`/journal-entries/${origine}`);
		await expect(page.getByTestId('reverse-entry-next-year')).toBeVisible();
		await captureBothSchemes(page, testInfo, '29-1b-bouton');
		await page.getByTestId('reverse-entry-next-year').click();
		// La confirmation nomme l'écriture, la date et l'exercice (revue P1, B-1).
		await expect(page.getByTestId('reverse-entry-next-year-dialog-body')).toHaveText(
			`La contre-passation de l’écriture n° ${origineNumber} « Charge payée d’avance (transitoire) » sera datée du 01.01.2026, dans l’exercice « Exercice 2026 ».`,
		);
		await captureBothSchemes(page, testInfo, '29-1b-confirmation');
		await page.getByTestId('reverse-entry-next-year-confirm').click();
		await expect(page).not.toHaveURL(new RegExp(`/journal-entries/${origine}$`));
		await expect(page.getByTestId('reverses-link')).toBeVisible();
		const inverseId = Number(page.url().split('/').pop());
		const inverse = (await (await ctx.get(`/api/v1/journal-entries/${inverseId}`)).json()) as {
			entryDate: string;
			fiscalYearId: number;
			reversesEntryId: number;
		};
		expect(inverse.entryDate).toBe('2026-01-01');
		expect(inverse.fiscalYearId).toBe(n1);
		expect(inverse.reversesEntryId).toBe(origine);

		// 5. Bilan de N au dernier jour : 1300 porte le montant ; de N+1 au premier jour : nul.
		expect(Number(await balanceOf(ctx, n, '2025-12-31', '1300'))).toBe(250);
		expect(Number(await balanceOf(ctx, n1, '2026-01-01', '1300'))).toBe(0);

		// 6. Clôturer N : le dialogue dit « Aucun point à vérifier ».
		await page.goto('/settings/fiscal-years');
		await page.getByTestId(`fiscal-year-close-${n}`).click();
		await expect(page.getByTestId('fiscal-year-close-checks-none')).toHaveText(
			'Aucun point à vérifier.',
		);
		await page.getByTestId('fiscal-year-close-confirm').click();
		await expect(page.getByTestId(`fiscal-year-reopen-${n}`)).toBeVisible();
		const clos = (await (await ctx.get(`/api/v1/fiscal-years/${n}`)).json()) as { status: string };
		expect(clos.status).toBe('Closed');

		// 7. Un administrateur rouvre N avec un motif ; le panneau se relit.
		await page.getByTestId(`fiscal-year-reopen-${n}`).click();
		await page.getByTestId('fiscal-year-reopen-motif').fill('Recette du parcours de clôture');
		await page.getByTestId('fiscal-year-reopen-confirm').click();
		await expect(page.getByTestId(`fiscal-year-close-${n}`)).toBeVisible();
		const rouvert = (await (await ctx.get(`/api/v1/fiscal-years/${n}`)).json()) as {
			status: string;
		};
		expect(rouvert.status).toBe('Open');
		dialog = await openChecks(page, n);
		await expect(dialog.getByTestId('fiscal-year-checks-row-NEXT_FISCAL_YEAR')).toHaveAttribute(
			'data-status',
			'ok',
		);
	} finally {
		await disposeContextSafe(ctx);
	}
});

test('sans exercice suivant : pas de bouton, le motif le dit, et l’API refuse en 409', async ({
	page,
}, testInfo) => {
	await login(page);
	const ctx = await authedApiContext(page);
	try {
		await createFiscalYear(ctx, 2025);
		const transitoire = await createAccount(ctx, '1300', 'Actifs transitoires');
		const charge = await accountId(ctx, '4000');
		const origine = await regularisation(ctx, transitoire, charge);

		await page.goto(`/journal-entries/${origine}`);
		await expect(page.getByTestId('reverse-entry-next-year-reason')).toHaveText(
			'Pour contre-passer au début de l’exercice suivant, créez d’abord l’exercice suivant.',
		);
		await expect(page.getByTestId('reverse-entry-next-year')).toHaveCount(0);
		// « Contre-passer » (du jour) reste offert.
		await expect(page.getByTestId('reverse-entry')).toBeVisible();
		await captureBothSchemes(page, testInfo, '29-1b-refus-motif');

		const res = await ctx.post(
			`/api/v1/journal-entries/${origine}/reverse-at-next-fiscal-year-start`,
		);
		expect(res.status()).toBe(409);
		const body = (await res.json()) as { error: { code: string; details: { fiscalYearName: string } } };
		expect(body.error.code).toBe('NEXT_FISCAL_YEAR_MISSING');
		expect(body.error.details.fiscalYearName).toBe('Exercice 2025');
	} finally {
		await disposeContextSafe(ctx);
	}
});
