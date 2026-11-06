/**
 * Story 33-1b (#926, AC10) — le numéro interne d'une facture fournisseur, de bout en bout :
 * il naît à la validation, se voit à la liste, se cherche, se trie, survit au retour en
 * brouillon et ne change pas à la revalidation ; le format se règle, et seule la facture
 * suivante le porte.
 *
 * ⚠️ Assertions **relatives** : la base partagée a un compteur qui ne redescend pas, le
 * numéro attendu se lit donc sur la réponse. Le format est restauré par un `afterAll`, qui
 * survit à un échec en cours de spec. Décor synthétique : le seed `with-company`.
 */
import path from 'node:path';
import { expect, test, type APIRequestContext, type Page, type TestInfo } from '@playwright/test';
import {
	seedTestState,
	clearAuthStorage,
	authedApiContext,
	disposeContextSafe,
} from './helpers/test-state';
import { wiredBankAccountViaApi } from './helpers/api-fixtures';

const FORMAT_DEFAUT = 'FF-{YEAR}-{SEQ:03}';
const BACKEND = process.env.KESH_BACKEND_URL ?? 'http://127.0.0.1:3000';

test.beforeAll(async () => {
	await seedTestState('with-company');
});

test.afterEach(async ({ page }) => {
	await clearAuthStorage(page);
});

test.afterAll(async ({ request }) => {
	const login = await request.post(`${BACKEND}/api/v1/auth/login`, {
		data: { username: 'admin', password: 'admin123' },
	});
	if (!login.ok()) return;
	const { accessToken } = (await login.json()) as { accessToken: string };
	const headers = { Authorization: `Bearer ${accessToken}` };
	const s = await (await request.get(`${BACKEND}/api/v1/company/invoice-settings`, { headers })).json();
	await request.put(`${BACKEND}/api/v1/company/invoice-settings`, {
		headers,
		data: { ...s, supplierInternalNumberFormat: FORMAT_DEFAUT },
	});
});

async function login(page: Page): Promise<void> {
	await page.goto('/login');
	await page.fill('#username', 'admin');
	await page.fill('#password', 'admin123');
	await page.click('button[type="submit"]');
	await expect(page).toHaveURL('/');
}

async function capture(page: Page, testInfo: TestInfo, name: string): Promise<void> {
	const dossier = process.env.KESH_E2E_CAPTURES_DIR;
	await page.screenshot({
		path: dossier ? path.join(dossier, `${name}.png`) : testInfo.outputPath(`${name}.png`),
		fullPage: true,
	});
}

interface Decor {
	ctx: APIRequestContext;
	supplierId: number;
	expenseId: number;
	payableId: number;
}

async function decor(page: Page): Promise<Decor> {
	const ctx = await authedApiContext(page);
	const accounts = (await (await ctx.get('/api/v1/accounts')).json()) as Array<{
		id: number;
		number: string;
		accountType: string;
		active: boolean;
		postable: boolean;
	}>;
	const payable = accounts.find((a) => a.number === '2000');
	const expense = accounts.find((a) => a.accountType === 'Expense' && a.active && a.postable);
	expect(payable && expense).toBeTruthy();
	const s = await (await ctx.get('/api/v1/company/invoice-settings')).json();
	if (s.defaultPayableAccountId == null) {
		const put = await ctx.put('/api/v1/company/invoice-settings', {
			data: { ...s, defaultPayableAccountId: payable!.id },
		});
		expect(put.ok()).toBeTruthy();
	}
	const sup = await ctx.post('/api/v1/contacts', {
		data: {
			contactType: 'Entreprise',
			name: `Fournisseur numéro ${Date.now()}`,
			isClient: false,
			isSupplier: true,
		},
	});
	expect(sup.ok()).toBeTruthy();
	return { ctx, supplierId: (await sup.json()).id, expenseId: expense!.id, payableId: payable!.id };
}

async function brouillon(d: Decor, numero: string | null): Promise<{ id: number; version: number }> {
	const res = await d.ctx.post('/api/v1/supplier-invoices', {
		data: {
			contactId: d.supplierId,
			supplierInvoiceNumber: numero,
			invoiceDate: new Date().toISOString().slice(0, 10),
			creditorIban: 'CH5604835012345678009',
			paymentReference: 'Facture E2E',
			lines: [
				{
					description: 'Achat',
					quantity: '1',
					unitPrice: '100.00',
					vatRate: '0',
					expenseAccountId: d.expenseId,
				},
			],
		},
	});
	expect(res.ok(), await res.text()).toBeTruthy();
	return res.json();
}

type Validee = { id: number; version: number; internalNumber: string };

async function valider(d: Decor, f: { id: number; version: number }): Promise<Validee> {
	const res = await d.ctx.post(`/api/v1/supplier-invoices/${f.id}/validate`, {
		data: { version: f.version },
	});
	expect(res.ok(), await res.text()).toBeTruthy();
	return res.json();
}

function numerosAffiches(page: Page) {
	return page.getByTestId('supplier-invoice-row-internal-number');
}

test.describe('Le numéro interne', () => {
	test('naît à la validation, se cherche, se trie, survit au brouillon', async ({
		page,
	}, testInfo) => {
		await login(page);
		const d = await decor(page);
		try {
			const numeroFournisseur = `ABC-${Date.now()}`;
			const b = await brouillon(d, numeroFournisseur);
			expect((b as unknown as { internalNumber: unknown }).internalNumber).toBeNull();

			// Le brouillon jamais validé : pas de numéro à la fiche.
			await page.goto(`/supplier-invoices/${b.id}`);
			await expect(page.getByTestId('supplier-invoice-detail-internal-number')).toContainText(
				'Attribué à la validation',
			);
			await capture(page, testInfo, 'fiche-brouillon-jamais-valide');

			const v = await valider(d, b);
			expect(v.internalNumber).toMatch(/^FF-\d{4}-\d{3,}$/);

			// La liste : les deux colonnes.
			await page.goto('/supplier-invoices');
			await expect(numerosAffiches(page).filter({ hasText: v.internalNumber })).toHaveCount(1);
			await capture(page, testInfo, 'liste-deux-colonnes');

			// Trouvé par `q` : le numéro interne, puis le numéro du fournisseur.
			await page.getByTestId('supplier-invoices-search').fill(v.internalNumber);
			await expect(page).toHaveURL(/[?&]q=/);
			await expect(numerosAffiches(page)).toHaveText([v.internalNumber]);
			await capture(page, testInfo, 'liste-filtree');
			await page.getByTestId('supplier-invoices-search').fill(numeroFournisseur);
			await expect(numerosAffiches(page)).toHaveText([v.internalNumber]);
			await page.getByTestId('supplier-invoices-search').fill('introuvable-33-1b');
			await expect(page.getByTestId('supplier-invoices-empty')).toContainText(
				'Aucune facture ne correspond',
			);
			await capture(page, testInfo, 'liste-sans-resultat');

			// Tri croissant et décroissant, vérifié sur l'ordre des lignes.
			const autre = await valider(d, await brouillon(d, null));
			await page.goto('/supplier-invoices');
			await page.getByTestId('supplier-invoices-sort-internal-number').click();
			await expect(page).toHaveURL(/sort=internalNumber/);
			await expect(page).toHaveURL(/order=asc/);
			await expect(numerosAffiches(page).first()).not.toHaveText('—');
			const croissant = (await numerosAffiches(page).allTextContents()).filter((n) => n !== '—');
			expect(croissant.indexOf(v.internalNumber)).toBeLessThan(
				croissant.indexOf(autre.internalNumber),
			);
			await capture(page, testInfo, 'liste-triee');
			await page.getByTestId('supplier-invoices-sort-internal-number').click();
			await expect(page).toHaveURL(/order=desc/);
			await expect(numerosAffiches(page).first()).toHaveText(
				croissant[croissant.length - 1],
			);

			// Repassé en brouillon : conservé ; revalidé : le même.
			const fiche = await page.goto(`/supplier-invoices/${v.id}`);
			expect(fiche?.ok()).toBeTruthy();
			await capture(page, testInfo, 'fiche-validee');
			const u = await d.ctx.post(`/api/v1/supplier-invoices/${v.id}/unvalidate`, {
				data: { version: v.version },
			});
			expect(u.ok(), await u.text()).toBeTruthy();
			const redraft = (await u.json()) as Validee;
			expect(redraft.internalNumber).toBe(v.internalNumber);
			await page.reload();
			await expect(page.getByTestId('supplier-invoice-detail-number-kept')).toBeVisible();
			await capture(page, testInfo, 'fiche-brouillon-numerote');
			await page.getByTestId('supplier-invoice-delete').click();
			await expect(page.getByTestId('supplier-invoice-delete-number-warning')).toContainText(
				v.internalNumber,
			);
			await capture(page, testInfo, 'dialogue-suppression');
			await page.keyboard.press('Escape');
			const revalidee = await valider(d, redraft);
			expect(revalidee.internalNumber).toBe(v.internalNumber);
		} finally {
			await disposeContextSafe(d.ctx);
		}
	});

	test('le réglage du format, et ses refus sur place', async ({ page }, testInfo) => {
		await login(page);
		const d = await decor(page);
		try {
			const premiere = await valider(d, await brouillon(d, null));
			await page.goto('/settings/invoicing');
			const champ = page.getByTestId('settings-supplier-internal-number-format');
			await expect(champ).toHaveValue(FORMAT_DEFAUT);
			await capture(page, testInfo, 'reglage-valide');

			for (const ko of ['FF-{YEAR}', 'FF-{SEQ:03}', 'FF-{FY}-{SEQ}', '{YEAR}-{YEAR}-{SEQ}']) {
				await champ.fill(ko);
				await expect(page.getByTestId('settings-supplier-format-error')).toBeVisible();
			}
			await capture(page, testInfo, 'reglage-refus');

			await champ.fill('KF-{YEAR}-{SEQ:04}');
			await expect(page.getByTestId('settings-supplier-format-preview')).toContainText('KF-2026-0001');
			await page.locator('form button[type="submit"]').first().click();
			await expect(async () => {
				const s = await (await d.ctx.get('/api/v1/company/invoice-settings')).json();
				expect(s.supplierInternalNumberFormat).toBe('KF-{YEAR}-{SEQ:04}');
			}).toPass();

			const suivante = await valider(d, await brouillon(d, null));
			expect(suivante.internalNumber).toMatch(/^KF-\d{4}-\d{4,}$/);
			const relue = (await (
				await d.ctx.get(`/api/v1/supplier-invoices/${premiere.id}`)
			).json()) as { internalNumber: string };
			expect(relue.internalNumber).toBe(premiere.internalNumber);

			const s = await (await d.ctx.get('/api/v1/company/invoice-settings')).json();
			const remis = await d.ctx.put('/api/v1/company/invoice-settings', {
				data: { ...s, supplierInternalNumberFormat: FORMAT_DEFAUT },
			});
			expect(remis.ok()).toBeTruthy();
		} finally {
			await disposeContextSafe(d.ctx);
		}
	});

	test('postes ouverts et lots disent le numéro interne', async ({ page }, testInfo) => {
		await login(page);
		const d = await decor(page);
		try {
			const v = await valider(d, await brouillon(d, 'ABC-LOT'));

			// Postes ouverts du compte créanciers : la pièce dit le numéro interne.
			await page.goto(`/open-items?accountId=${d.payableId}`);
			await expect(page.getByTestId('open-items-list')).toContainText(v.internalNumber);
			await capture(page, testInfo, 'postes-ouverts');

			// La sélection d'un lot montre les deux numéros ; le détail, le numéro interne.
			await page.goto('/payment-batches');
			await expect(page.locator('body')).toContainText(`${v.internalNumber} (ABC-LOT)`);
			await capture(page, testInfo, 'lot-selection');
			const bankAccountId = await wiredBankAccountViaApi(page, 'CH9300762011623852957');
			const res = await d.ctx.post('/api/v1/payment-batches', {
				data: {
					bankAccountId,
					requestedExecutionDate: new Date().toISOString().slice(0, 10),
					supplierInvoiceIds: [v.id],
				},
			});
			expect(res.ok(), await res.text()).toBeTruthy();
			const body = await res.json();
			expect(body.batch.items[0].internalNumber).toBe(v.internalNumber);
			await page.goto(`/payment-batches/${body.batch.id}`);
			await expect(
				page.locator(`a[href="/supplier-invoices/${v.id}"]`),
			).toHaveText(v.internalNumber);
			await capture(page, testInfo, 'lot-detail');
		} finally {
			await disposeContextSafe(d.ctx);
		}
	});
});
