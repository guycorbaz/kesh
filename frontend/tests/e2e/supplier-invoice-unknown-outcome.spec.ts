/**
 * Story 34-8 (#938, T7) — le paiement d'une facture fournisseur dont la réponse est
 * coupée APRÈS l'écriture. Le serveur a payé, le navigateur ne le sait pas : la fiche
 * relit la facture, la montre payée, et le dit dans un encadré. Le bouton *Payer* n'est
 * plus offert, la facture ayant changé de statut.
 *
 * La coupure est réelle (patron `invoices-settle-unknown-outcome.spec.ts`) :
 * `route.fetch()` envoie la requête au serveur, puis `route.abort()` jette sa réponse —
 * un `abort()` seul n'enverrait rien, et rien ne serait écrit.
 *
 * Décor synthétique : le seed `with-company`, un fournisseur créé par l'API. Les
 * captures (thème clair) s'écrivent dans `KESH_E2E_CAPTURES_DIR`, sinon dans la sortie
 * du test.
 */
import path from 'node:path';
import { expect, test, type Page, type TestInfo } from '@playwright/test';
import {
	seedTestState,
	clearAuthStorage,
	authedApiContext,
	disposeContextSafe,
} from './helpers/test-state';

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

async function capture(page: Page, testInfo: TestInfo, name: string): Promise<void> {
	const dossier = process.env.KESH_E2E_CAPTURES_DIR;
	await page.screenshot({
		path: dossier ? path.join(dossier, `${name}.png`) : testInfo.outputPath(`${name}.png`),
		fullPage: true,
	});
}

type Account = {
	id: number;
	number: string;
	accountType: string;
	active: boolean;
	postable: boolean;
};

/** Une facture fournisseur validée de 100.00, datée du jour ; rend son id et le compte de caisse. */
async function validatedSupplierInvoice(page: Page): Promise<{ id: number; cashId: number }> {
	const ctx = await authedApiContext(page);
	try {
		const accounts = (await (await ctx.get('/api/v1/accounts')).json()) as Account[];
		const payable = accounts.find((a) => a.number === '2000');
		const expense = accounts.find((a) => a.accountType === 'Expense' && a.active && a.postable);
		const cash = accounts.find(
			(a) => a.accountType === 'Asset' && a.active && a.postable && a.number !== '2000',
		);
		expect(payable && expense && cash, 'comptes du décor').toBeTruthy();
		const s = await (await ctx.get('/api/v1/company/invoice-settings')).json();
		if (s.defaultPayableAccountId == null) {
			const put = await ctx.put('/api/v1/company/invoice-settings', {
				data: { ...s, defaultPayableAccountId: payable!.id },
			});
			expect(put.ok(), `compte créanciers : ${put.status()}`).toBeTruthy();
		}
		const sup = await ctx.post('/api/v1/contacts', {
			data: {
				contactType: 'Entreprise',
				name: `Fournisseur issue inconnue ${Date.now()}`,
				isClient: false,
				isSupplier: true,
			},
		});
		expect(sup.ok(), `fournisseur : ${sup.status()}`).toBeTruthy();
		const draft = await ctx.post('/api/v1/supplier-invoices', {
			data: {
				contactId: (await sup.json()).id,
				supplierInvoiceNumber: `E348-${Date.now()}`,
				invoiceDate: new Date().toISOString().slice(0, 10),
				lines: [
					{
						description: 'Fourniture',
						quantity: '1',
						unitPrice: '100.00',
						vatRate: '0',
						expenseAccountId: expense!.id,
					},
				],
			},
		});
		expect(draft.ok(), `brouillon : ${draft.status()} ${await draft.text()}`).toBeTruthy();
		const d = await draft.json();
		const validated = await ctx.post(`/api/v1/supplier-invoices/${d.id}/validate`, {
			data: { version: d.version },
		});
		expect(validated.ok(), `validation : ${validated.status()}`).toBeTruthy();
		return { id: d.id as number, cashId: cash!.id };
	} finally {
		await disposeContextSafe(ctx);
	}
}

test('fiche fournisseur : paiement coupé après l’écriture, la fiche relue le dit', async ({
	page,
}, testInfo) => {
	await login(page);
	const { id, cashId } = await validatedSupplierInvoice(page);
	await page.route(`**/api/v1/supplier-invoices/${id}/pay`, async (route) => {
		if (route.request().method() !== 'POST') return route.fallback();
		const response = await route.fetch();
		expect(response.status(), 'le serveur a bien payé').toBe(200);
		await route.abort('connectionreset');
	});
	await page.goto(`/supplier-invoices/${id}`);
	await page.getByTestId('supplier-invoice-pay').getByRole('radio').nth(1).check();
	await page.getByTestId('pay-internal-account').selectOption(String(cashId));
	await page.getByTestId('supplier-invoice-pay-submit').click();

	const notice = page.getByTestId('supplier-unknown-outcome');
	await expect(notice).toBeVisible();
	await expect(notice).toHaveText(
		'L’issue n’est pas connue : la facture a été relue, vérifiez-la avant de réessayer.',
	);
	// Le paiement était écrit : la relecture montre la facture payée, et *Payer* n'est
	// plus offert.
	await expect(page.getByTestId('supplier-invoice-status')).toHaveText('Payée');
	await expect(page.getByTestId('supplier-invoice-pay-submit')).toHaveCount(0);
	await expect(page.getByTestId('supplier-invoice-pay-error')).toHaveCount(0);
	await capture(page, testInfo, 'e348-paiement-issue-inconnue');
});
