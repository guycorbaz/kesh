/**
 * Story 25-3-b (#418) — annuler un rapprochement, de bout en bout.
 *
 * Importer un relevé, rapprocher sa transaction (rapprochement manuel, par
 * l'API — le décor), puis l'ANNULER par l'interface, depuis le détail de
 * l'import : le seul écran qui montre une transaction rapprochée. La
 * transaction revient « à rapprocher », et dans les propositions.
 *
 * ⚠️ **Un relevé UNIQUE à chaque exécution** : la fixture minimale est
 * réimportée par d'autres specs sur la base partagée ; ses références et son
 * montant sont réécrits pour que ce scénario ne dépende ni de l'ordre des
 * specs, ni d'un doublon.
 *
 * Pré-requis : MariaDB up + KESH_TEST_MODE=true (cf. docs/testing.md).
 */

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

const FIXTURE = path.join(
	path.dirname(fileURLToPath(import.meta.url)),
	'fixtures',
	'camt053_v04_minimal.xml',
);
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
 * Le décor : un compte bancaire câblé sur un compte du grand livre, un import
 * unique, sa transaction rapprochée manuellement. Rend `(importId, txId)`.
 */
async function reconciledTransaction(page: Page): Promise<{ importId: number; txId: number }> {
	const ctx = await authedApiContext(page);
	try {
		const accounts = (await (await ctx.get('/api/v1/accounts')).json()) as Array<{
			id: number;
			number: string;
		}>;
		const ledger = accounts.find((a) => a.number === '1000');
		const revenue = accounts.find((a) => a.number === '3000');
		expect(ledger && revenue, 'comptes 1000 et 3000 du préréglage').toBeTruthy();

		type Ba = { id: number; iban: string; version: number };
		const findAccount = async (): Promise<Ba | undefined> => {
			const list = await ctx.get('/api/v1/bank-accounts');
			expect(list.ok()).toBeTruthy();
			return ((await list.json()) as Ba[]).find((b) => b.iban === TEST_IBAN);
		};
		let ba = await findAccount();
		if (!ba) {
			const res = await ctx.post('/api/v1/bank-accounts', {
				data: { bankName: 'UBS Test', iban: TEST_IBAN, qrIban: null, isPrimary: true },
			});
			expect(res.ok(), `bank-account create: ${res.status()}`).toBeTruthy();
			ba = await findAccount();
		}
		const bankAccountId = ba!.id;
		// Le rapprochement manuel exige un compte bancaire câblé sur le grand livre.
		const wired = await ctx.patch(`/api/v1/bank-accounts/${bankAccountId}`, {
			data: { journalAccountId: ledger!.id, version: ba!.version },
		});
		expect(wired.ok(), `bank-account wiring: ${wired.status()}`).toBeTruthy();

		const unique = `${Date.now()}`;
		const amount = `${100 + (Number(unique.slice(-5)) % 800)}.${unique.slice(-2)}`;
		const xml = fs
			.readFileSync(FIXTURE, 'utf8')
			.replace('STMT-2026-05-001', `STMT-CANCEL-${unique}`)
			.replace('BANK-TX-42', `BANK-TX-CANCEL-${unique}`)
			.replace('E2E-2026-05-001', `E2E-CANCEL-${unique}`)
			.replace('<Amt Ccy="CHF">1234.56</Amt>', `<Amt Ccy="CHF">${amount}</Amt>`);
		const imported = await ctx.post('/api/v1/bank-imports', {
			multipart: {
				bankAccountId: bankAccountId.toString(),
				file: {
					name: `cancel-${unique}.xml`,
					mimeType: 'application/xml',
					buffer: Buffer.from(xml, 'utf8'),
				},
				confirmBalanceMismatch: 'true',
				confirmDuplicateFile: 'true',
			},
		});
		expect(imported.status(), await imported.text()).toBe(201);
		const importId = ((await imported.json()) as { id: number }).id;

		const detail = (await (await ctx.get(`/api/v1/bank-imports/${importId}`)).json()) as {
			transactions: Array<{ id: number }>;
		};
		const txId = detail.transactions[0].id;

		const matched = await ctx.post('/api/v1/reconciliation/manual', {
			data: {
				bankAccountId,
				bankTransactionId: txId,
				counterpartyAccountId: revenue!.id,
				description: 'Recette E2E',
			},
		});
		expect(matched.status(), await matched.text()).toBe(200);
		return { importId, txId };
	} finally {
		await disposeContextSafe(ctx);
	}
}

test('rapprocher puis annuler le rapprochement depuis le détail de l’import', async ({ page }) => {
	await login(page);
	const { importId, txId } = await reconciledTransaction(page);

	await page.goto(`/bank-import/${importId}`);
	const row = page.locator(`[data-testid="detail-tx-row"][data-tx-id="${txId}"]`);
	await expect(row).toContainText('reconciled');
	await expect(row.getByTestId('detail-tx-entry-link')).toBeVisible();

	await row.getByTestId('detail-tx-cancel-reconciliation').click();
	const dialog = page.getByTestId('reconciliation-cancel-dialog');
	await expect(dialog.getByTestId('reconciliation-cancel-confirm-text')).toContainText(
		'écriture inverse',
	);
	await dialog.getByTestId('reconciliation-cancel-submit').click();

	// Le détail est relu : la transaction est de nouveau « à rapprocher ».
	await expect(dialog).toBeHidden();
	await expect(row).toContainText('pending');
	await expect(row.getByTestId('detail-tx-cancel-reconciliation')).toHaveCount(0);

	// ⛔ Et elle revient dans les propositions — c'est ce que l'écran de
	// réconciliation lit ; sans cela, elle serait perdue pour l'utilisateur.
	const ctx = await authedApiContext(page);
	try {
		const tx = (await (await ctx.get(`/api/v1/reconciliation/transactions/${txId}`)).json()) as {
			status: string;
			matchedEntryId: number | null;
		};
		expect(tx.status).toBe('pending');
		expect(tx.matchedEntryId).toBeNull();
	} finally {
		await disposeContextSafe(ctx);
	}
});

// ---------------------------------------------------------------------------
// Story 53-6a (#1674, AC-15) — l'annulation se date de l'écriture du rapprochement
// ---------------------------------------------------------------------------

/** La date de l'écriture du rapprochement manuel : celle de la transaction de la fixture. */
const ENTRY_DATE = '2026-05-15';
/** La borne du verrou du scénario « période verrouillée » : postérieure à la date de l'écriture. */
const LOCK_THROUGH = '2026-06-30';

function capturePath(testInfo: TestInfo, name: string): string {
	const dossier = process.env.KESH_E2E_CAPTURES_DIR;
	return dossier ? path.join(dossier, `${name}.png`) : testInfo.outputPath(`${name}.png`);
}

/** Recette visuelle : thème clair seulement, animations coupées (hors dépôt, `KESH_E2E_CAPTURES_DIR`). */
async function capture(page: Page, testInfo: TestInfo, name: string): Promise<void> {
	await page.screenshot({
		path: capturePath(testInfo, `53-6a-${name}`),
		fullPage: true,
		animations: 'disabled',
	});
}

test('53-6a — même exercice : le dialogue dit la date de l’écriture et son exercice ; l’inverse en est datée', async ({
	page,
}, testInfo) => {
	await login(page);
	const { importId, txId } = await reconciledTransaction(page);
	await page.goto(`/bank-import/${importId}`);
	const row = page.locator(`[data-testid="detail-tx-row"][data-tx-id="${txId}"]`);
	await row.getByTestId('detail-tx-cancel-reconciliation').click();
	const dialog = page.getByTestId('reconciliation-cancel-dialog');
	const texte = dialog.getByTestId('reconciliation-cancel-confirm-text');
	await expect(texte).toContainText('datée du 15.05.2026');
	await expect(texte).toContainText("dans l'exercice « Exercice CI 2020-2030 »");
	await expect(texte).not.toContainText("aujourd'hui");
	await expect(dialog.getByTestId('reconciliation-cancel-vat')).toContainText(
		"Si cette écriture porte le compte d'impôt préalable",
	);
	await expect(dialog.getByTestId('reconciliation-cancel-vat')).toContainText(
		"l'annulation sera alors refusée, ce qui protège la déclaration",
	);
	await capture(page, testInfo, 'confirmation-datee');

	const reponse = page.waitForResponse(
		(r) => r.url().endsWith(`/reconciliation/transactions/${txId}/cancel`) && r.ok(),
	);
	await dialog.getByTestId('reconciliation-cancel-submit').click();
	const { reversalJournalEntryId } = (await (await reponse).json()) as {
		reversalJournalEntryId: number;
	};
	await expect(dialog).toBeHidden();

	// *Voir l'écriture* de l'inverse : datée du 15 mai, non du jour.
	await page.goto(`/journal-entries/${reversalJournalEntryId}`);
	await expect(page.locator('body')).toContainText('15.05.2026');
	await capture(page, testInfo, 'inverse-datee');
	const ctx = await authedApiContext(page);
	try {
		const inverse = (await (
			await ctx.get(`/api/v1/journal-entries/${reversalJournalEntryId}`)
		).json()) as { entryDate: string };
		expect(inverse.entryDate).toBe(ENTRY_DATE);
	} finally {
		await disposeContextSafe(ctx);
	}
});

test('53-6a — une écriture sous le verrou de période : le motif, sans bouton de confirmation', async ({
	page,
}, testInfo) => {
	await login(page);
	const { importId, txId } = await reconciledTransaction(page);
	const ctx = await authedApiContext(page);
	try {
		// La borne (30 juin) est postérieure à l'écriture (15 mai) : le motif doit nommer la borne
		// comme borne du verrou, non comme la date de l'écriture (revue P1, L1).
		const lock = await ctx.post('/api/v1/companies/current/books-lock', {
			data: { through: LOCK_THROUGH },
		});
		expect(lock.ok(), `verrou : ${lock.status()} ${await lock.text()}`).toBeTruthy();

		await page.goto(`/bank-import/${importId}`);
		const row = page.locator(`[data-testid="detail-tx-row"][data-tx-id="${txId}"]`);
		await row.getByTestId('detail-tx-cancel-reconciliation').click();
		const dialog = page.getByTestId('reconciliation-cancel-dialog');
		const motif = dialog.getByTestId('reconciliation-cancel-motif');
		await expect(motif).toContainText('période verrouillée');
		await expect(motif).toContainText("La période est verrouillée jusqu'au 30.06.2026 inclus.");
		await expect(motif).not.toContainText('(30.06.2026)');
		await expect(dialog.getByTestId('reconciliation-cancel-submit')).toHaveCount(0);
		await capture(page, testInfo, 'motif-periode-verrouillee');
	} finally {
		// Un verrou laissé en place verrouillerait les specs suivantes.
		await ctx.post('/api/v1/companies/current/books-lock/release', {
			data: { through: null, motif: 'Fin du test E2E 53-6a' },
		});
		await disposeContextSafe(ctx);
	}
});

// ---------------------------------------------------------------------------
// Story 53-6a (#1674, AC-15 (1), revue P1 M1) — LE cas du défaut : l'annulation faite
// après la fin de l'exercice de l'écriture
// ---------------------------------------------------------------------------

const YEAR_END_FIXTURE = path.join(
	path.dirname(fileURLToPath(import.meta.url)),
	'fixtures',
	'camt053_v04_year_end.xml',
);

/** La date de valeur de l'écriture du rapprochement : distincte de celle de la transaction (15.12.2025). */
const EARLIER_ENTRY_DATE = '2025-12-12';

async function createFiscalYear(ctx: APIRequestContext, year: number): Promise<number> {
	const res = await ctx.post('/api/v1/fiscal-years', {
		data: { name: `Exercice ${year}`, startDate: `${year}-01-01`, endDate: `${year}-12-31` },
	});
	expect(res.status(), await res.text()).toBe(201);
	return ((await res.json()) as { id: number }).id;
}

test.describe('53-6a — une écriture d’un exercice antérieur encore ouvert', () => {
	// Société sans exercice : 2025 et 2026 se créent ici, tous deux ouverts (rien n'est clôturé).
	// 2026 couvre le jour d'exécution (et 2025 n'est plus « l'exercice du jour » dès le 1er janvier
	// 2026) : une contre-passation datée du jour tombait dans 2026, hors de l'exercice de l'écriture.
	test.beforeAll(async () => {
		await seedTestState('with-company-no-fy');
	});
	// La suite du fichier (et les specs suivantes) attendent la société du préréglage commun.
	test.afterAll(async () => {
		await seedTestState('with-company');
	});

	test('le dialogue et l’inverse disent la date de l’écriture et SON exercice, non le jour ni l’exercice du jour', async ({
		page,
	}, testInfo) => {
		await login(page);
		const ctx = await authedApiContext(page);
		let importId: number;
		let txId: number;
		let entryId: number;
		let fy2025: number;
		try {
			fy2025 = await createFiscalYear(ctx, 2025);
			await createFiscalYear(ctx, 2026);
			const accounts = (await (await ctx.get('/api/v1/accounts')).json()) as Array<{
				id: number;
				number: string;
			}>;
			const revenue = accounts.find((a) => a.number === '3000');
			expect(revenue, 'compte 3000 du préréglage').toBeTruthy();
			const ledgerRes = await ctx.post('/api/v1/accounts', {
				data: { number: '1020', name: 'Banque', accountType: 'Asset' },
			});
			expect(ledgerRes.status(), await ledgerRes.text()).toBe(201);
			const ledgerId = ((await ledgerRes.json()) as { id: number }).id;

			const created = await ctx.post('/api/v1/bank-accounts', {
				data: { bankName: 'UBS Test', iban: TEST_IBAN, qrIban: null, isPrimary: true },
			});
			expect(created.ok(), `bank-account create: ${created.status()}`).toBeTruthy();
			type Ba = { id: number; iban: string; version: number };
			const ba = ((await (await ctx.get('/api/v1/bank-accounts')).json()) as Ba[]).find(
				(b) => b.iban === TEST_IBAN,
			)!;
			const wired = await ctx.patch(`/api/v1/bank-accounts/${ba.id}`, {
				data: { journalAccountId: ledgerId, version: ba.version },
			});
			expect(wired.ok(), `bank-account wiring: ${wired.status()}`).toBeTruthy();

			// Relevé de fin 2025 (transaction du 15.12.2025) ; l'écriture du rapprochement est
			// datée du 12.12.2025 (date de valeur donnée au geste) : la date de l'écriture
			// diffère de celle de la transaction — une date lue sur la transaction ne passerait pas.
			const imported = await ctx.post('/api/v1/bank-imports', {
				multipart: {
					bankAccountId: ba.id.toString(),
					file: {
						name: 'annulation-exercice-anterieur.xml',
						mimeType: 'application/xml',
						buffer: fs.readFileSync(YEAR_END_FIXTURE),
					},
					confirmBalanceMismatch: 'true',
					confirmDuplicateFile: 'true',
				},
			});
			expect(imported.status(), await imported.text()).toBe(201);
			importId = ((await imported.json()) as { id: number }).id;
			const detail = (await (await ctx.get(`/api/v1/bank-imports/${importId}`)).json()) as {
				transactions: Array<{ id: number; bookingDate: string }>;
			};
			txId = detail.transactions[0].id;
			expect(detail.transactions[0].bookingDate).toBe('2025-12-15');

			const matched = await ctx.post('/api/v1/reconciliation/manual', {
				data: {
					bankAccountId: ba.id,
					bankTransactionId: txId,
					counterpartyAccountId: revenue!.id,
					description: 'Recette de fin 2025',
					valueDate: EARLIER_ENTRY_DATE,
				},
			});
			expect(matched.status(), await matched.text()).toBe(200);
			entryId = ((await matched.json()) as { journalEntryId: number }).journalEntryId;
		} finally {
			await disposeContextSafe(ctx);
		}

		// *Voir l'écriture* de la ligne rapprochée : l'écriture d'origine, du 12.12.2025.
		await page.goto(`/bank-import/${importId}`);
		const row = page.locator(`[data-testid="detail-tx-row"][data-tx-id="${txId}"]`);
		await expect(row).toContainText('reconciled');
		const origine = row.getByTestId('detail-tx-entry-link');
		await expect(origine).toHaveAttribute('href', `/journal-entries/${entryId}`);
		await origine.click();
		await expect(page).toHaveURL(`/journal-entries/${entryId}`);
		await expect(page.locator('body')).toContainText('12.12.2025');
		await page.goBack();

		// Le dialogue : la date de l'ÉCRITURE (12.12), non celle de la transaction (15.12) ni le
		// jour ; l'exercice de l'écriture, 2025, non l'exercice du jour.
		await row.getByTestId('detail-tx-cancel-reconciliation').click();
		const dialog = page.getByTestId('reconciliation-cancel-dialog');
		const texte = dialog.getByTestId('reconciliation-cancel-confirm-text');
		await expect(texte).toContainText('datée du 12.12.2025');
		await expect(texte).toContainText("dans l'exercice « Exercice 2025 »");
		await expect(texte).not.toContainText('15.12.2025');
		await expect(texte).not.toContainText("aujourd'hui");
		await capture(page, testInfo, 'exercice-anterieur-confirmation');

		const reponse = page.waitForResponse(
			(r) => r.url().endsWith(`/reconciliation/transactions/${txId}/cancel`) && r.ok(),
		);
		await dialog.getByTestId('reconciliation-cancel-submit').click();
		const { reversalJournalEntryId } = (await (await reponse).json()) as {
			reversalJournalEntryId: number;
		};
		await expect(dialog).toBeHidden();
		await expect(row).toContainText('pending');

		// Le lien *Voir l'écriture* de la ligne a disparu avec le rapprochement : l'inverse se
		// rejoint depuis l'écriture d'origine, par son renvoi « Contre-passée par l'écriture n° … ».
		await page.goto(`/journal-entries/${entryId}`);
		const renvoi = page.getByTestId('reversed-by-link');
		await expect(renvoi).toBeVisible();
		await renvoi.getByRole('link').click();
		await expect(page).toHaveURL(`/journal-entries/${reversalJournalEntryId}`);
		await expect(page.locator('body')).toContainText('12.12.2025');
		await expect(page.getByTestId('reverses-link')).toBeVisible();
		await capture(page, testInfo, 'exercice-anterieur-inverse');

		// Et l'exercice de l'inverse : 2025 (celui de l'écriture), jamais celui du jour.
		const api = await authedApiContext(page);
		try {
			const inverse = (await (
				await api.get(`/api/v1/journal-entries/${reversalJournalEntryId}`)
			).json()) as { entryDate: string; fiscalYearId: number };
			expect(inverse.entryDate).toBe(EARLIER_ENTRY_DATE);
			expect(inverse.fiscalYearId).toBe(fy2025);
		} finally {
			await disposeContextSafe(api);
		}
	});
});
