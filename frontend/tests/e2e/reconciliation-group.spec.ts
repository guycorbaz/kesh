/**
 * Story 43-3 (#823) — « Régler plusieurs factures… » : une transaction du relevé règle plusieurs
 * factures, ajoutées une à une par recherche, avec l'écart et son compte.
 *
 * C'est le seul test qui voit les identifiants des factures et le compte de l'écart traverser la
 * frontière HTTP jusqu'à `POST /reconciliation/accept` (`invoice_group`, `supplier_invoice_group`),
 * et la recherche (`GET …/group-candidates`) en revenir (`CLAUDE.md`, § *E2E*).
 *
 * ⏳ **EN ATTENTE** (`test.describe.fixme`) tant que l'API de la 43-2 n'est pas dans la branche :
 * le spec est écrit sur son contrat et se joue sur la branche réunie (43-1a et 43-2 fusionnées),
 * fiche 43-3, § *Planning*. Retirer le `fixme` à ce moment-là.
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
	const inv = (await validated.json()) as { invoiceNumber: string; totalAmount: string };
	return { id, number: inv.invoiceNumber, total: inv.totalAmount };
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

test.describe.fixme('Règlement groupé — l’écran (Story 43-3, #823)', () => {
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
			await expect(modal.getByTestId('group-gap')).toHaveText('Écart trop grand (au plus 5.00)');
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
		} finally {
			await cleanup(ctx, txId, importId);
			await disposeContextSafe(ctx);
		}
	});
});
