/**
 * Story 43-3 (#823) — « Régler plusieurs factures… » : une transaction du relevé règle plusieurs
 * factures, ajoutées une à une par recherche, avec l'écart et son compte.
 *
 * C'est le seul test qui voit les identifiants des factures et le compte de l'écart traverser la
 * frontière HTTP jusqu'à `POST /reconciliation/accept` (`invoice_group`, `supplier_invoice_group`),
 * et la recherche (`GET …/group-candidates`) en revenir (`CLAUDE.md`, § *E2E*).
 *
 * Se joue sur la branche réunie (43-1a, 43-1b et 43-2 fusionnées), fiche 43-3, § *Planning*.
 *
 * ⚠️ **Montants et références UNIQUES à chaque exécution** (patron de
 * `reconciliation-link.spec.ts`) : prix des factures et contreparties tirés de l'horloge. En
 * `finally` : groupe annulé, factures annulées (avoir) quand l'API le permet, import supprimé,
 * comptes désignés restaurés.
 *
 * ⚠️ **Sélecteurs par `data-testid`** (garde KF-043, #326).
 *
 * # Recette visuelle (AC8)
 *
 * Captures en thème clair, dans `KESH_E2E_CAPTURES_DIR` s'il est posé (en recette :
 * `kesh-gate-logs/e433/captures/`), sinon dans la sortie du test. Hors dépôt.
 *
 * Pré-requis : MariaDB up + KESH_TEST_MODE=true (cf. docs/testing.md).
 */

import { expect, test, type APIRequestContext, type Page, type TestInfo } from '@playwright/test';
import path from 'path';
import {
	seedTestState,
	clearAuthStorage,
	authedApiContext,
	disposeContextSafe,
} from './helpers/test-state';
import {
	createContactWithAddressViaApi,
	createManualDebitStatementViaApi,
	createSupplierInvoiceViaApi,
	setDesignatedAccountsViaApi,
	todayInZurich,
	wiredBankAccountViaApi,
	type DesignatedAccounts,
} from './helpers/api-fixtures';
import { contrastRatio } from './helpers/contrast';

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

/** Le dossier des captures : `KESH_E2E_CAPTURES_DIR`, sinon la sortie du test. */
function capturePath(testInfo: TestInfo, name: string): string {
	const dossier = process.env.KESH_E2E_CAPTURES_DIR;
	return dossier ? path.join(dossier, `${name}.png`) : testInfo.outputPath(`${name}.png`);
}

type Account = { id: number; number: string; name: string; accountType: string; active: boolean; postable: boolean };

/** Un compte de charge imputable, créé à un numéro unique (les comptes désignés de l'écart). */
async function expenseAccount(ctx: APIRequestContext, name: string): Promise<Account> {
	const number = `G${Date.now().toString().slice(-6)}${Math.floor(Math.random() * 10)}`;
	const res = await ctx.post('/api/v1/accounts', {
		data: { number, name, accountType: 'Expense' },
	});
	expect(res.ok(), `compte ${name} : ${res.status()} ${await res.text()}`).toBeTruthy();
	return (await res.json()) as Account;
}

/**
 * Une facture client validée d'une ligne sans TVA, de `quantity × unitPrice`, datée `date`.
 * Rend son id, son numéro et son reste dû (le total, lu par l'API).
 */
async function clientInvoice(
	ctx: APIRequestContext,
	contactId: number,
	unitPrice: string,
	date: string,
	quantity = '1',
): Promise<{ id: number; number: string; total: string }> {
	const created = await ctx.post('/api/v1/invoices', {
		data: {
			contactId,
			date,
			dueDate: date,
			paymentTerms: '30 jours net',
			lines: [{ description: 'Prestation 43-3', quantity, unitPrice, vatRate: '0.00' }],
		},
	});
	expect(created.ok(), `facture : ${created.status()} ${await created.text()}`).toBeTruthy();
	const { id } = (await created.json()) as { id: number };
	const validated = await ctx.post(`/api/v1/invoices/${id}/validate`);
	expect(validated.ok(), `validation : ${validated.status()}`).toBeTruthy();
	const inv = (await validated.json()) as { invoiceNumber: string };
	// Le reste dû (TTC) lu sur la fiche : `totalAmount` est le HT (`routes/invoices.rs`).
	const fiche = (await (await ctx.get(`/api/v1/invoices/${id}`)).json()) as { amountDue: string };
	return { id, number: inv.invoiceNumber, total: fiche.amountDue };
}

/** Un relevé saisi à la main d'une seule ligne de crédit (patron de `createManualDebitStatementViaApi`). */
async function creditStatement(
	ctx: APIRequestContext,
	opts: { bankAccountId: number; bookingDate: string; amount: string; counterpartyName: string; details: string },
): Promise<{ importId: number; txId: number }> {
	const [y, m] = opts.bookingDate.split('-');
	const lastDay = new Date(Date.UTC(Number(y), Number(m), 0)).getUTCDate();
	const res = await ctx.post('/api/v1/bank-imports/manual', {
		data: {
			bankAccountId: opts.bankAccountId,
			periodFrom: `${y}-${m}-01`,
			periodTo: `${y}-${m}-${String(lastDay).padStart(2, '0')}`,
			openingBalance: '0.00',
			closingBalance: opts.amount,
			lines: [
				{
					bookingDate: opts.bookingDate,
					details: opts.details,
					counterpartyName: opts.counterpartyName,
					amount: opts.amount,
					direction: 'credit',
				},
			],
			confirmDuplicateFile: true,
			confirmDuplicateLines: true,
			confirmOpeningBalanceGap: true,
		},
	});
	expect(res.status(), `relevé : ${await res.text()}`).toBe(201);
	const importId = ((await res.json()) as { id: number }).id;
	const detail = (await (await ctx.get(`/api/v1/bank-imports/${importId}`)).json()) as {
		transactions: Array<{ id: number }>;
	};
	return { importId, txId: detail.transactions[0].id };
}

/** Additionne des montants décimaux en centimes entiers (montants à deux décimales). */
function sumCents(...amounts: string[]): number {
	return amounts.reduce((s, a) => s + Math.round(Number(a) * 100), 0);
}

/** Ouvre la modale du règlement groupé sur la transaction `txId`. */
async function openGroup(page: Page, bankAccountId: number, txId: number) {
	await page.goto('/reconciliation');
	await page.getByTestId('bank-account-select').selectOption(bankAccountId.toString());
	const ligne = page.locator(`[data-testid="reconciliation-row"][data-tx-id="${txId}"]`);
	await expect(ligne).toBeVisible({ timeout: 10000 });
	await ligne.getByTestId('group-settlement-button').click();
	const modal = page.getByTestId('group-settlement-modal');
	await expect(modal).toBeVisible();
	return { ligne, modal };
}

/** Cherche par `q`, puis ajoute la facture `invoiceId`. */
async function add(modal: ReturnType<Page['getByTestId']>, q: string, invoiceId: number) {
	await modal.getByTestId('group-search-input').fill(q);
	const bouton = modal.locator(`[data-testid="group-add"][data-invoice-id="${invoiceId}"]`);
	await expect(bouton).toBeVisible({ timeout: 10000 });
	await bouton.click();
	await expect(
		modal.locator(`[data-testid="group-imputed-row"][data-invoice-id="${invoiceId}"]`),
	).toBeVisible();
}

/** Le décor client : deux factures de deux clients, un crédit de `sum + deltaCents`. */
async function clientScene(page: Page, tag: string, deltaCents: number) {
	const unique = `${Date.now()}-${tag}`;
	const today = todayInZurich();
	const bankAccountId = await wiredBankAccountViaApi(page, TEST_IBAN);
	const ctx = await authedApiContext(page);
	const a = await createContactWithAddressViaApi(page, `Groupe A ${unique}`);
	const b = await createContactWithAddressViaApi(page, `Groupe B ${unique}`);
	const base = 10000 + (Date.now() % 20000);
	const fa = await clientInvoice(ctx, a, (base / 100).toFixed(2), today);
	const fb = await clientInvoice(ctx, b, ((base + 3700) / 100).toFixed(2), today);
	const amount = ((sumCents(fa.total, fb.total) + deltaCents) / 100).toFixed(2);
	const { importId, txId } = await creditStatement(ctx, {
		bankAccountId,
		bookingDate: today,
		amount,
		counterpartyName: `Groupe A ${unique}`,
		details: `Règlement groupé ${unique}`,
	});
	return { ctx, unique, bankAccountId, fa, fb, a, b, amount, importId, txId };
}

/** Le nettoyage : groupe annulé s'il a été passé, import supprimé. */
async function cleanup(ctx: APIRequestContext, txId: number, importId: number) {
	await ctx.post(`/api/v1/reconciliation/transactions/${txId}/cancel`, { data: {} });
	await ctx.delete(`/api/v1/bank-imports/${importId}`);
}

test.describe('Règlement groupé — l’écran (Story 43-3, #823)', () => {
	let before: DesignatedAccounts | null = null;
	let fees: Account;
	let rounding: Account;

	test.beforeEach(async ({ page }) => {
		await login(page);
		const ctx = await authedApiContext(page);
		try {
			fees = await expenseAccount(ctx, 'Frais bancaires 43-3');
			rounding = await expenseAccount(ctx, 'Différences d’arrondi 43-3');
		} finally {
			await disposeContextSafe(ctx);
		}
		before = await setDesignatedAccountsViaApi(page, {
			defaultRoundingAccountId: rounding.id,
			defaultBankFeesAccountId: fees.id,
			// Sans arrondi des pièces à 5 centimes : le reste dû suit les lignes (cas 20.01).
			roundTo5Centimes: false,
		});
	});

	test.afterEach(async ({ page }) => {
		if (before) {
			await login(page);
			await setDesignatedAccountsViaApi(page, before);
			before = null;
		}
	});

	test('deux factures de deux clients, moins-payé de 0.40 sur le compte prérempli, puis annulation', async ({
		page,
	}, testInfo) => {
		const s = await clientScene(page, 'moins', -40);
		try {
			const { ligne, modal } = await openGroup(page, s.bankAccountId, s.txId);
			// Aucune présélection ; la recherche est préremplie par la contrepartie.
			await expect(modal.getByTestId('group-imputed-empty')).toBeVisible();
			await expect(modal.getByTestId('group-search-input')).toHaveValue(`Groupe A ${s.unique}`);
			await expect(modal.getByTestId('group-submit')).toBeDisabled();
			await modal.screenshot({ path: capturePath(testInfo, '43-3-modale-vide') });

			await add(modal, s.fa.number, s.fa.id);
			await modal.screenshot({ path: capturePath(testInfo, '43-3-une-facture') });
			await add(modal, s.fb.number, s.fb.id);
			await expect(modal.getByTestId('group-gap')).toHaveText('Moins-payé de 0.40');
			// Le texte de l'écart reste lisible (L-4, P2) : contraste sur son fond. Les couleurs
			// calculées (souvent `oklab(…)` avec Tailwind 4) passent par un canevas 1 × 1, qui
			// rend leurs composantes sRGB.
			const couleurs = await modal.getByTestId('group-gap').evaluate((el) => {
				const enRgb = (css: string): [number, number, number] => {
					const c = document.createElement('canvas');
					c.width = 1;
					c.height = 1;
					const g = c.getContext('2d')!;
					g.fillStyle = '#ffffff';
					g.fillRect(0, 0, 1, 1);
					g.fillStyle = css;
					g.fillRect(0, 0, 1, 1);
					const [r, v, b] = g.getImageData(0, 0, 1, 1).data;
					return [r, v, b];
				};
				let fond = 'rgb(255, 255, 255)';
				for (let n: Element | null = el; n; n = n.parentElement) {
					const bg = getComputedStyle(n).backgroundColor;
					if (bg && bg !== 'transparent' && !/rgba\(.*,\s*0\)$/.test(bg)) {
						fond = bg;
						break;
					}
				}
				return { texte: enRgb(getComputedStyle(el).color), fond: enRgb(fond) };
			});
			expect(contrastRatio(couleurs.texte, couleurs.fond)).toBeGreaterThanOrEqual(4.5);
			const champ = modal.locator('input[aria-autocomplete="list"]');
			await expect(champ).toHaveValue(`${fees.number} — ${fees.name}`);
			await expect(modal.getByTestId('group-submit')).toBeEnabled();
			await modal.screenshot({ path: capturePath(testInfo, '43-3-moins-paye-prerempli') });

			const accept = page.waitForRequest(
				(r) => r.url().includes('/api/v1/reconciliation/accept') && r.method() === 'POST',
			);
			await modal.getByTestId('group-submit').click();
			const body = (await accept).postDataJSON() as { proposals: unknown[] };
			expect(body.proposals).toEqual([
				{
					type: 'invoice_group',
					bankTransactionId: s.txId,
					invoiceIds: [s.fa.id, s.fb.id],
					differenceAccountId: fees.id,
				},
			]);
			await expect(modal).toBeHidden();
			await expect(page.getByTestId('reconciliation-success')).toHaveText(
				/^\s*1\s+opération\(s\) réussie\(s\)\.\s*$/,
			);
			// Sonde négative : la transaction rapprochée n'offre plus le bouton.
			await expect(ligne).toHaveCount(0);

			const view = (await (
				await s.ctx.get(`/api/v1/reconciliation/transactions/${s.txId}`)
			).json()) as { kind: string; gap: string; groupedInvoices: Array<{ id: number }> };
			expect(view.kind).toBe('grouped_settlement');
			expect(view.gap).toBe('-0.40');
			expect(view.groupedInvoices.map((g) => g.id).sort()).toEqual([s.fa.id, s.fb.id].sort());

			// L'annulation, depuis le détail de l'import : les deux factures, l'écart et son compte.
			await page.goto(`/bank-import/${s.importId}`);
			const row = page.locator(`[data-testid="detail-tx-row"][data-tx-id="${s.txId}"]`);
			await row.getByTestId('detail-tx-cancel-reconciliation').click();
			const dialog = page.getByTestId('reconciliation-cancel-dialog');
			await expect(dialog.getByTestId('reconciliation-cancel-group-invoice')).toHaveCount(2);
			await expect(dialog.getByTestId('reconciliation-cancel-group-gap')).toContainText(fees.number);
			await dialog.screenshot({ path: capturePath(testInfo, '43-3-annulation-groupe') });
			await dialog.getByTestId('reconciliation-cancel-submit').click();
			await expect(row.getByTestId('detail-tx-cancel-reconciliation')).toHaveCount(0);
			for (const f of [s.fa, s.fb]) {
				const inv = (await (await s.ctx.get(`/api/v1/invoices/${f.id}`)).json()) as { paidAt: string | null };
				expect(inv.paidAt, `${f.number} rouverte`).toBeNull();
			}
		} finally {
			await cleanup(s.ctx, s.txId, s.importId);
			await disposeContextSafe(s.ctx);
		}
	});

	test('trop-perçu de 0.40 sur un compte choisi', async ({ page }, testInfo) => {
		const s = await clientScene(page, 'trop', 40);
		try {
			const { modal } = await openGroup(page, s.bankAccountId, s.txId);
			await add(modal, s.fa.number, s.fa.id);
			await add(modal, s.fb.number, s.fb.id);
			await expect(modal.getByTestId('group-gap')).toHaveText('Trop-perçu de 0.40');
			const champ = modal.locator('input[aria-autocomplete="list"]');
			await expect(champ).toHaveValue(`${rounding.number} — ${rounding.name}`);
			// Choisir un autre compte : les frais bancaires.
			await champ.fill(fees.number);
			await modal.getByRole('option', { name: new RegExp(fees.number) }).first().dispatchEvent('mousedown');
			await expect(champ).toHaveValue(`${fees.number} — ${fees.name}`);
			await modal.screenshot({ path: capturePath(testInfo, '43-3-trop-percu') });
			const accept = page.waitForRequest((r) => r.url().includes('/api/v1/reconciliation/accept'));
			await modal.getByTestId('group-submit').click();
			expect(((await accept).postDataJSON() as { proposals: Array<{ differenceAccountId: number }> }).proposals[0].differenceAccountId).toBe(fees.id);
			await expect(modal).toBeHidden();
		} finally {
			await cleanup(s.ctx, s.txId, s.importId);
			await disposeContextSafe(s.ctx);
		}
	});

	test('écart nul : aucun compte demandé ; écart trop grand : Enregistrer reste inactif', async ({
		page,
	}, testInfo) => {
		const s = await clientScene(page, 'grand', -600);
		try {
			const { modal } = await openGroup(page, s.bankAccountId, s.txId);
			await add(modal, s.fa.number, s.fa.id);
			await add(modal, s.fb.number, s.fb.id);
			await expect(modal.getByTestId('group-gap')).toHaveText(
				'Écart trop grand : Moins-payé de 6.00 (au plus 5.00)',
			);
			await expect(modal.getByTestId('group-block-hint')).toContainText('L’écart dépasse le plafond admis');
			await expect(modal.getByTestId('group-account-section')).toHaveCount(0);
			await expect(modal.getByTestId('group-submit')).toBeDisabled();
			await modal.screenshot({ path: capturePath(testInfo, '43-3-ecart-trop-grand') });
		} finally {
			await cleanup(s.ctx, s.txId, s.importId);
			await disposeContextSafe(s.ctx);
		}
		const z = await clientScene(page, 'nul', 0);
		try {
			const { modal } = await openGroup(page, z.bankAccountId, z.txId);
			await add(modal, z.fa.number, z.fa.id);
			await add(modal, z.fb.number, z.fb.id);
			await expect(modal.getByTestId('group-gap')).toHaveText('Aucun écart');
			await expect(modal.getByTestId('group-account-section')).toHaveCount(0);
			await modal.screenshot({ path: capturePath(testInfo, '43-3-ecart-nul') });
			const accept = page.waitForRequest((r) => r.url().includes('/api/v1/reconciliation/accept'));
			await modal.getByTestId('group-submit').click();
			expect(((await accept).postDataJSON() as { proposals: Array<Record<string, unknown>> }).proposals[0]).not.toHaveProperty('differenceAccountId');
			await expect(modal).toBeHidden();
		} finally {
			await cleanup(z.ctx, z.txId, z.importId);
			await disposeContextSafe(z.ctx);
		}
	});

	test('deux restes sous le centime payés 20.01 : aucun écart, aucun compte demandé', async ({ page }, testInfo) => {
		const unique = `${Date.now()}-centime`;
		const today = todayInZurich();
		const bankAccountId = await wiredBankAccountViaApi(page, TEST_IBAN);
		const ctx = await authedApiContext(page);
		const c = await createContactWithAddressViaApi(page, `Centime ${unique}`);
		// 0.5 × 20.01 = 10.005 : un reste dû à fraction de centime (à vérifier sur l'API réelle).
		const f1 = await clientInvoice(ctx, c, '20.01', today, '0.5');
		const f2 = await clientInvoice(ctx, c, '20.01', today, '0.5');
		const { importId, txId } = await creditStatement(ctx, {
			bankAccountId,
			bookingDate: today,
			amount: '20.01',
			counterpartyName: `Centime ${unique}`,
			details: `Centime ${unique}`,
		});
		try {
			const { modal } = await openGroup(page, bankAccountId, txId);
			await add(modal, f1.number, f1.id);
			await add(modal, f2.number, f2.id);
			await expect(modal.getByTestId('group-gap')).toHaveText('Aucun écart');
			await expect(modal.getByTestId('group-account-section')).toHaveCount(0);
			await expect(modal.getByTestId('group-submit')).toBeEnabled();
			await modal.screenshot({ path: capturePath(testInfo, '43-3-sous-le-centime') });
		} finally {
			await cleanup(ctx, txId, importId);
			await disposeContextSafe(ctx);
		}
	});

	test('un compte de créance choisi pour l’écart est refusé, et la phrase le dit sous le sélecteur', async ({
		page,
	}, testInfo) => {
		const s = await clientScene(page, 'creance', -40);
		try {
			const { modal } = await openGroup(page, s.bankAccountId, s.txId);
			await add(modal, s.fa.number, s.fa.id);
			await add(modal, s.fb.number, s.fb.id);
			const champ = modal.locator('input[aria-autocomplete="list"]');
			await champ.fill('1100');
			await modal.getByRole('option', { name: /1100/ }).first().dispatchEvent('mousedown');
			await modal.getByTestId('group-submit').click();
			await expect(modal.getByTestId('group-account-error')).toHaveText(
				'Le compte choisi pour l’écart est le compte de créance (ou de dette) d’une facture du groupe : choisissez-en un autre.',
			);
			await modal.screenshot({ path: capturePath(testInfo, '43-3-compte-refuse-creance') });
		} finally {
			await cleanup(s.ctx, s.txId, s.importId);
			await disposeContextSafe(s.ctx);
		}
	});

	test('une ligne signalée « déjà comptabilisée » demande confirmation avant la modale', async ({
		page,
	}, testInfo) => {
		const s = await clientScene(page, 'deja', 0);
		let entryId: number | null = null;
		try {
			const ba = ((await (await s.ctx.get('/api/v1/bank-accounts')).json()) as Array<{
				id: number;
				journalAccountId: number | null;
			}>).find((b) => b.id === s.bankAccountId);
			const entry = await s.ctx.post('/api/v1/journal-entries', {
				data: {
					entryDate: todayInZurich(),
					journal: 'Banque',
					description: `Déjà comptabilisée 43-3 ${s.unique}`,
					lines: [
						{ accountId: ba!.journalAccountId, debit: s.amount, credit: '0' },
						{ accountId: fees.id, debit: '0', credit: s.amount },
					],
				},
			});
			expect(entry.status(), await entry.text()).toBe(201);
			entryId = ((await entry.json()) as { id: number }).id;

			await page.goto('/reconciliation');
			await page.getByTestId('bank-account-select').selectOption(s.bankAccountId.toString());
			const ligne = page.locator(`[data-testid="reconciliation-row"][data-tx-id="${s.txId}"]`);
			await expect(ligne).toBeVisible({ timeout: 10000 });
			await ligne.getByTestId('group-settlement-button').click();
			const confirmation = page.getByTestId('already-booked-confirm');
			await expect(confirmation).toBeVisible();
			await page.screenshot({ path: capturePath(testInfo, '43-3-deja-comptabilisee') });
			await confirmation.getByTestId('already-booked-confirm-cancel').click();
			await expect(page.getByTestId('group-settlement-modal')).toHaveCount(0);
			await ligne.getByTestId('group-settlement-button').click();
			await page.getByTestId('already-booked-confirm-continue').click();
			await expect(page.getByTestId('group-settlement-modal')).toBeVisible();
		} finally {
			if (entryId !== null) {
				await s.ctx.post(`/api/v1/journal-entries/${entryId}/reverse`, { data: {} });
			}
			await cleanup(s.ctx, s.txId, s.importId);
			await disposeContextSafe(s.ctx);
		}
	});

	// D6 — retirable avec le commit fournisseur si la 43-1b sort.
	test('un débit règle deux factures fournisseur (D6)', async ({ page }, testInfo) => {
		const unique = `${Date.now()}-fourn`;
		const today = todayInZurich();
		const bankAccountId = await wiredBankAccountViaApi(page, TEST_IBAN);
		const ctx = await authedApiContext(page);
		const base = 20000 + (Date.now() % 20000);
		const a = (base / 100).toFixed(2);
		const b = ((base + 1100) / 100).toFixed(2);
		const s1 = await createSupplierInvoiceViaApi(page, {
			supplierName: `Fournisseur A ${unique}`,
			number: `FA-${unique}`,
			paymentReference: null,
			invoiceDate: today,
			dueDate: today,
			amount: a,
		});
		const s2 = await createSupplierInvoiceViaApi(page, {
			supplierName: `Fournisseur B ${unique}`,
			number: `FB-${unique}`,
			paymentReference: null,
			invoiceDate: today,
			dueDate: today,
			amount: b,
		});
		const debit = ((sumCents(a, b) + 40) / 100).toFixed(2);
		const importId = await createManualDebitStatementViaApi(page, {
			bankAccountId,
			bookingDate: today,
			amount: debit,
			counterpartyName: `Fournisseur A ${unique}`,
			details: `Paiement groupé ${unique}`,
		});
		const txId = ((await (await ctx.get(`/api/v1/bank-imports/${importId}`)).json()) as {
			transactions: Array<{ id: number }>;
		}).transactions[0].id;
		try {
			const { modal } = await openGroup(page, bankAccountId, txId);
			await add(modal, `FA-${unique}`, s1.supplierInvoiceId);
			await add(modal, `FB-${unique}`, s2.supplierInvoiceId);
			await expect(modal.getByTestId('group-gap')).toHaveText('Débit supérieur aux factures de 0.40');
			await expect(modal.locator('input[aria-autocomplete="list"]')).toHaveValue(`${fees.number} — ${fees.name}`);
			await modal.screenshot({ path: capturePath(testInfo, '43-3-fournisseur-debit-superieur') });
			const accept = page.waitForRequest((r) => r.url().includes('/api/v1/reconciliation/accept'));
			await modal.getByTestId('group-submit').click();
			expect(((await accept).postDataJSON() as { proposals: Array<{ type: string }> }).proposals[0].type).toBe(
				'supplier_invoice_group',
			);
			await expect(modal).toBeHidden();

			// Revue P1 (A L-9) : l'annulation d'un groupe fournisseur, jouée par l'interface.
			await page.goto(`/bank-import/${importId}`);
			const row = page.locator(`[data-testid="detail-tx-row"][data-tx-id="${txId}"]`);
			await row.getByTestId('detail-tx-cancel-reconciliation').click();
			const dialog = page.getByTestId('reconciliation-cancel-dialog');
			await expect(dialog.getByTestId('reconciliation-cancel-confirm-text')).toContainText(
				'le paiement des factures fournisseur',
			);
			await expect(dialog.getByTestId('reconciliation-cancel-group-invoice')).toHaveCount(2);
			await expect(dialog.getByTestId('reconciliation-cancel-group-gap')).toContainText(
				'Débit supérieur aux factures de 0.40',
			);
			await expect(dialog.getByTestId('reconciliation-cancel-group-gap')).toContainText(fees.number);
			await dialog.screenshot({ path: capturePath(testInfo, '43-3-annulation-groupe-fournisseur') });
			await dialog.getByTestId('reconciliation-cancel-submit').click();
			await expect(row.getByTestId('detail-tx-cancel-reconciliation')).toHaveCount(0);
			for (const id of [s1.supplierInvoiceId, s2.supplierInvoiceId]) {
				const si = (await (await ctx.get(`/api/v1/supplier-invoices/${id}`)).json()) as { status: string };
				expect(si.status, `facture fournisseur ${id} rouverte`).not.toBe('paid');
			}
		} finally {
			await cleanup(ctx, txId, importId);
			await disposeContextSafe(ctx);
		}
	});

	// Revue P1 (A L-4) : « retenu par un lot » sur un DÉBIT réel ; la retenue elle-même est
	// interceptée (le champ `heldByPaymentBatch` n'arrive qu'avec la tête définitive de la 43-2).
	test('un débit dont le montant est retenu par un lot : l’avis nomme le lot', async ({ page }, testInfo) => {
		const unique = `${Date.now()}-retenu`;
		const today = todayInZurich();
		const bankAccountId = await wiredBankAccountViaApi(page, TEST_IBAN);
		const ctx = await authedApiContext(page);
		const importId = await createManualDebitStatementViaApi(page, {
			bankAccountId,
			bookingDate: today,
			amount: ((30000 + (Date.now() % 20000)) / 100).toFixed(2),
			counterpartyName: `Fournisseur ${unique}`,
			details: `Débit retenu ${unique}`,
		});
		const txId = ((await (await ctx.get(`/api/v1/bank-imports/${importId}`)).json()) as {
			transactions: Array<{ id: number }>;
		}).transactions[0].id;
		await page.route('**/group-candidates**', async (route) => {
			const reponse = await route.fetch();
			const corps = await reponse.json();
			await route.fulfill({
				response: reponse,
				json: {
					...corps,
					candidates: [],
					hasMore: false,
					heldByPaymentBatch: { supplierInvoiceId: 1, paymentBatchId: 31 },
				},
			});
		});
		try {
			const { modal } = await openGroup(page, bankAccountId, txId);
			await expect(modal.getByTestId('group-held-by-batch')).toContainText('lot de paiement n° 31');
			await expect(modal.getByTestId('group-gap')).toHaveText('—');
			await modal.screenshot({ path: capturePath(testInfo, '43-3-retenu-par-un-lot') });
		} finally {
			await page.unroute('**/group-candidates**');
			await cleanup(ctx, txId, importId);
			await disposeContextSafe(ctx);
		}
	});

	// ---------------------------------------------------------------------------------------
	// Les états de l'écran (AC8). Le décor est réel ; les états fugaces ou rares — chargement,
	// `hasMore`, `direction: null`, refus du serveur, réglages altérés — sont obtenus en
	// interceptant UNE réponse (`page.route`), le reste traversant la frontière HTTP.
	// ---------------------------------------------------------------------------------------

	test('états de la recherche : chargement, hasMore, aucun résultat, aucune facture à proposer', async ({
		page,
	}, testInfo) => {
		const s = await clientScene(page, 'recherche', 0);
		let mode: 'lent' | 'hasMore' | 'vide' | 'sansSens' = 'lent';
		const verrou = { relacher: () => {} };
		await page.route('**/group-candidates**', async (route) => {
			const reponse = await route.fetch();
			const corps = (await reponse.json()) as Record<string, unknown>;
			if (mode === 'lent') {
				await new Promise<void>((res) => (verrou.relacher = res));
				return route.fulfill({ response: reponse, json: corps });
			}
			if (mode === 'hasMore') return route.fulfill({ response: reponse, json: { ...corps, hasMore: true } });
			if (mode === 'vide') {
				return route.fulfill({ response: reponse, json: { ...corps, candidates: [], hasMore: false } });
			}
			return route.fulfill({
				response: reponse,
				json: { ...corps, direction: null, candidates: [], hasMore: false },
			});
		});
		try {
			const { modal } = await openGroup(page, s.bankAccountId, s.txId);
			await expect(modal.getByTestId('group-loading')).toHaveText('Recherche…');
			await modal.screenshot({ path: capturePath(testInfo, '43-3-chargement') });
			mode = 'hasMore';
			verrou.relacher();
			await modal.getByTestId('group-search-input').fill(s.fa.number);
			await expect(modal.getByTestId('group-has-more')).toBeVisible();
			await modal.screenshot({ path: capturePath(testInfo, '43-3-has-more') });
			mode = 'vide';
			await modal.getByTestId('group-search-input').fill('zzz');
			await expect(modal.getByTestId('group-results-empty')).toBeVisible();
			await modal.screenshot({ path: capturePath(testInfo, '43-3-aucun-resultat') });
			mode = 'sansSens';
			await modal.getByTestId('group-search-input').fill('x');
			await expect(modal.getByTestId('group-no-direction')).toHaveText(
				'Aucune facture à proposer pour cette transaction.',
			);
			await modal.screenshot({ path: capturePath(testInfo, '43-3-aucune-facture-a-proposer') });
		} finally {
			await page.unroute('**/group-candidates**');
			await cleanup(s.ctx, s.txId, s.importId);
			await disposeContextSafe(s.ctx);
		}
	});

	test('arrondi : seul, avec un écart, et sans compte d’arrondi désigné', async ({ page }, testInfo) => {
		const unique = `${Date.now()}-arrondi`;
		const today = todayInZurich();
		const bankAccountId = await wiredBankAccountViaApi(page, TEST_IBAN);
		const ctx = await authedApiContext(page);
		const c = await createContactWithAddressViaApi(page, `Arrondi ${unique}`);
		// 0.3 × 33.33 = 9.999 et 10.00 : 19.999 payés 20.00 → arrondi seul de +0.0010.
		const f1 = await clientInvoice(ctx, c, '33.33', today, '0.3');
		const f2 = await clientInvoice(ctx, c, '10.00', today);
		// 3 × (0.5 × 20.01) = 30.015 payés 30.06 → écart de 0.05 et arrondi de -0.0050.
		const g = await Promise.all([1, 2, 3].map(() => clientInvoice(ctx, c, '20.01', today, '0.5')));
		const t1 = await creditStatement(ctx, {
			bankAccountId,
			bookingDate: today,
			amount: '20.00',
			counterpartyName: `Arrondi ${unique}`,
			details: `Arrondi seul ${unique}`,
		});
		const t2 = await creditStatement(ctx, {
			bankAccountId,
			bookingDate: today,
			amount: '30.06',
			counterpartyName: `Arrondi ${unique}`,
			details: `Écart et arrondi ${unique}`,
		});
		try {
			let { modal } = await openGroup(page, bankAccountId, t1.txId);
			await add(modal, f1.number, f1.id);
			await add(modal, f2.number, f2.id);
			await expect(modal.getByTestId('group-gap')).toHaveText('Aucun écart');
			await expect(modal.getByTestId('group-rounding')).toHaveText(
				'Arrondi de +0.0010 passé au compte d’arrondi',
			);
			await expect(modal.getByTestId('group-submit')).toBeEnabled();
			await modal.screenshot({ path: capturePath(testInfo, '43-3-arrondi-seul') });

			({ modal } = await openGroup(page, bankAccountId, t2.txId));
			for (const f of g) await add(modal, f.number, f.id);
			await expect(modal.getByTestId('group-gap')).toHaveText('Trop-perçu de 0.05');
			await expect(modal.getByTestId('group-rounding')).toHaveText(
				'Arrondi de -0.0050 passé au compte d’arrondi',
			);
			await modal.screenshot({ path: capturePath(testInfo, '43-3-ecart-et-arrondi') });

			// Sans compte d'arrondi désigné (réglage lu par la modale, réponse altérée).
			await page.route('**/api/v1/company/invoice-settings', async (route) => {
				const reponse = await route.fetch();
				const corps = await reponse.json();
				await route.fulfill({ response: reponse, json: { ...corps, defaultRoundingAccountId: null } });
			});
			({ modal } = await openGroup(page, bankAccountId, t1.txId));
			await add(modal, f1.number, f1.id);
			await add(modal, f2.number, f2.id);
			await expect(modal.getByTestId('group-rounding-not-configured')).toBeVisible();
			await expect(modal.getByTestId('group-submit')).toBeDisabled();
			await modal.screenshot({ path: capturePath(testInfo, '43-3-arrondi-sans-compte') });
		} finally {
			await page.unroute('**/api/v1/company/invoice-settings');
			await cleanup(ctx, t1.txId, t1.importId);
			await cleanup(ctx, t2.txId, t2.importId);
			await disposeContextSafe(ctx);
		}
	});

	test('compte désigné inutilisable : le champ reste vide', async ({ page }, testInfo) => {
		const s = await clientScene(page, 'inutilisable', -40);
		await page.route('**/api/v1/company/invoice-settings', async (route) => {
			const reponse = await route.fetch();
			const corps = await reponse.json();
			// Un identifiant qu'aucun compte offert ne porte (archivé ou d'une autre société).
			await route.fulfill({ response: reponse, json: { ...corps, defaultBankFeesAccountId: 999999 } });
		});
		try {
			const { modal } = await openGroup(page, s.bankAccountId, s.txId);
			await add(modal, s.fa.number, s.fa.id);
			await add(modal, s.fb.number, s.fb.id);
			await expect(modal.getByTestId('group-gap')).toHaveText('Moins-payé de 0.40');
			await expect(modal.locator('input[aria-autocomplete="list"]')).toHaveValue('');
			await expect(modal.getByTestId('group-submit')).toBeDisabled();
			await modal.screenshot({ path: capturePath(testInfo, '43-3-compte-designe-inutilisable') });
		} finally {
			await page.unroute('**/api/v1/company/invoice-settings');
			await cleanup(s.ctx, s.txId, s.importId);
			await disposeContextSafe(s.ctx);
		}
	});

	test('refus du serveur : une phrase par cause du compte de l’écart, et la facture refusée nommée', async ({
		page,
	}, testInfo) => {
		const s = await clientScene(page, 'refus', -40);
		let refus: { errorCode: string; details: Record<string, unknown> } | null = null;
		await page.route('**/api/v1/reconciliation/accept', async (route) => {
			if (!refus) return route.continue();
			await route.fulfill({
				json: { accepted: [], failed: [{ bankTransactionId: s.txId, ...refus }] },
			});
		});
		// Revue P1 (A L-3) : une facture refusée « n'est plus réglable » ne revient pas dans les
		// résultats — le serveur, dont le refus est intercepté, la jugerait encore réglable.
		let retiree: number | null = null;
		await page.route('**/group-candidates**', async (route) => {
			const reponse = await route.fetch();
			const corps = (await reponse.json()) as { candidates: Array<{ id: number }> };
			await route.fulfill({
				response: reponse,
				json: { ...corps, candidates: corps.candidates.filter((c) => c.id !== retiree) },
			});
		});
		// Revue P1 (A L-5) : un compte au nom neutre, que la capture ne dément pas.
		const neutre = await expenseAccount(s.ctx, 'Compte choisi pour l’écart 43-3');
		try {
			const { modal } = await openGroup(page, s.bankAccountId, s.txId);
			await add(modal, s.fa.number, s.fa.id);
			await add(modal, s.fb.number, s.fb.id);
			const champ = modal.locator('input[aria-autocomplete="list"]');
			await champ.fill(neutre.number);
			await modal.getByRole('option', { name: new RegExp(neutre.number) }).first().dispatchEvent('mousedown');
			await expect(champ).toHaveValue(`${neutre.number} — ${neutre.name}`);
			const causes: [string, string, Record<string, unknown>, string][] = [
				['banque', 'VALIDATION_ERROR', { reason: 'counterparty_equals_bank_ledger' }, 'compte de la banque'],
				['introuvable', 'ACCOUNT_NOT_FOUND', { missingAccountIds: [neutre.id] }, 'introuvable ou archivé'],
				[
					'non-imputable',
					'ACCOUNT_NOT_POSTABLE',
					{ rejected: [{ accountId: neutre.id, accountNumber: neutre.number }] },
					'n’est pas imputable',
				],
				['devise', 'ACCOUNT_IN_FOREIGN_CURRENCY', { accountId: neutre.id, currency: 'EUR' }, 'tenu en EUR'],
			];
			for (const [nom, errorCode, details, phrase] of causes) {
				refus = { errorCode, details };
				await modal.getByTestId('group-submit').click();
				await expect(modal.getByTestId('group-account-error')).toContainText(phrase);
				await modal.screenshot({ path: capturePath(testInfo, `43-3-compte-refuse-${nom}`) });
			}
			refus = {
				errorCode: 'RECONCILIATION_INVOICE_NOT_ELIGIBLE',
				details: { reason: 'invoice_already_paid', invoiceId: s.fb.id, invoiceNumber: s.fb.number },
			};
			retiree = s.fb.id;
			await modal.getByTestId('group-submit').click();
			await expect(modal.getByTestId('group-refusal')).toContainText(`Facture ${s.fb.number}`);
			await expect(
				modal.locator(`[data-testid="group-imputed-row"][data-invoice-id="${s.fb.id}"]`),
			).toHaveCount(0);
			await expect(
				modal.locator(`[data-testid="group-result-row"][data-invoice-id="${s.fb.id}"]`),
			).toHaveCount(0);
			await modal.screenshot({ path: capturePath(testInfo, '43-3-facture-refusee-retiree') });
		} finally {
			await page.unroute('**/group-candidates**');
			await page.unroute('**/api/v1/reconciliation/accept');
			await cleanup(s.ctx, s.txId, s.importId);
			await disposeContextSafe(s.ctx);
		}
	});

	test('annulation d’un groupe refusée : la facture en cause est nommée', async ({ page }, testInfo) => {
		const s = await clientScene(page, 'annul-refus', 0);
		try {
			const r = await s.ctx.post('/api/v1/reconciliation/accept', {
				data: {
					bankAccountId: s.bankAccountId,
					proposals: [{ type: 'invoice_group', bankTransactionId: s.txId, invoiceIds: [s.fa.id, s.fb.id] }],
				},
			});
			expect(((await r.json()) as { accepted: unknown[] }).accepted).toHaveLength(1);
			// Le motif par facture (avoir sur une facture du groupe) est altéré dans la vue lue.
			await page.route(`**/api/v1/reconciliation/transactions/${s.txId}`, async (route) => {
				const reponse = await route.fetch();
				const corps = await reponse.json();
				await route.fulfill({
					response: reponse,
					json: {
						...corps,
						cancellable: false,
						cancelBlockedBy: 'INVOICE_CREDITED',
						cancelBlockedInvoice: { kind: 'invoice', id: s.fb.id, number: s.fb.number },
					},
				});
			});
			await page.goto(`/bank-import/${s.importId}`);
			const row = page.locator(`[data-testid="detail-tx-row"][data-tx-id="${s.txId}"]`);
			await row.getByTestId('detail-tx-cancel-reconciliation').click();
			const dialog = page.getByTestId('reconciliation-cancel-dialog');
			await expect(dialog.getByTestId('reconciliation-cancel-motif')).toContainText(
				`Une des factures de ce rapprochement, ${s.fb.number}`,
			);
			await dialog.screenshot({ path: capturePath(testInfo, '43-3-annulation-refusee') });
		} finally {
			await page.unroute(`**/api/v1/reconciliation/transactions/${s.txId}`);
			await cleanup(s.ctx, s.txId, s.importId);
			await disposeContextSafe(s.ctx);
		}
	});
});
