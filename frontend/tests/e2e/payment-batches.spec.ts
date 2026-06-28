import { expect, test } from '@playwright/test';
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

/**
 * Tests E2E — Lots de paiement pain.001 (Story 12.3, #191).
 *
 * Parcours : enregistrer une facture fournisseur (avec IBAN) + générer un lot (API),
 * puis confirmer le lot via l'UI → le lot passe « Confirmé » et la facture « payée ».
 */

async function login(page: import('@playwright/test').Page) {
	await page.goto('/login');
	await page.fill('#username', 'admin');
	await page.fill('#password', 'admin123');
	await page.click('button[type="submit"]');
	await expect(page).toHaveURL('/');
}

function uniq(prefix: string): string {
	return `${prefix} ${Date.now()}-${Math.floor(Math.random() * 1e6)}`;
}

/** Garantit la config (compte créanciers + compte de charge) et retourne un bank account avec journal_account_id. */
async function ensureSetup(page: import('@playwright/test').Page): Promise<{
	expenseAccountId: number;
	bankAccountId: number;
}> {
	const ctx = await authedApiContext(page);
	try {
		const accRes = await ctx.get('/api/v1/accounts');
		const accounts = (await accRes.json()) as Array<{
			id: number;
			number: string;
			accountType: string;
			active: boolean;
		}>;
		const payable = accounts.find((a) => a.number === '2000');
		const expense = accounts.find((a) => a.accountType === 'Expense' && a.active);
		const liquid = accounts.find((a) => a.accountType === 'Asset' && a.active);
		expect(payable && expense && liquid).toBeTruthy();

		// Config compte créanciers si absent.
		const sRes = await ctx.get('/api/v1/company/invoice-settings');
		const s = await sRes.json();
		if (s.defaultPayableAccountId == null) {
			const put = await ctx.put('/api/v1/company/invoice-settings', {
				data: { ...s, defaultPayableAccountId: payable!.id },
			});
			expect(put.ok()).toBeTruthy();
		}

		// Compte bancaire avec journal_account_id.
		const banksRes = await ctx.get('/api/v1/bank-accounts');
		const banks = (await banksRes.json()) as Array<{ id: number; journalAccountId: number | null }>;
		let bank = banks.find((b) => b.journalAccountId !== null);
		if (!bank) {
			// Créer un compte bancaire + lier au compte liquide.
			const createRes = await ctx.post('/api/v1/bank-accounts', {
				data: { bankName: 'Banque E2E', iban: 'CH9300762011623852957', isPrimary: banks.length === 0 },
			});
			expect(createRes.ok(), `create bank failed: ${createRes.status()}`).toBeTruthy();
			const created = await createRes.json();
			const patchRes = await ctx.put(`/api/v1/bank-accounts/${created.id}`, {
				data: { ...created, journalAccountId: liquid!.id },
			});
			expect(patchRes.ok(), `link bank failed: ${patchRes.status()}`).toBeTruthy();
			bank = { id: created.id, journalAccountId: liquid!.id };
		}
		return { expenseAccountId: expense!.id, bankAccountId: bank.id };
	} finally {
		await disposeContextSafe(ctx);
	}
}

async function createSupplierWithInvoice(
	page: import('@playwright/test').Page,
	expenseAccountId: number,
): Promise<number> {
	const ctx = await authedApiContext(page);
	try {
		const supRes = await ctx.post('/api/v1/contacts', {
			data: { contactType: 'Entreprise', name: uniq('Fourn'), isClient: false, isSupplier: true },
		});
		const supplierId = (await supRes.json()).id as number;
		const today = new Date().toISOString().slice(0, 10);
		const invRes = await ctx.post('/api/v1/supplier-invoices', {
			data: {
				contactId: supplierId,
				supplierInvoiceNumber: uniq('FF'),
				invoiceDate: today,
				creditorIban: 'CH5604835012345678009',
				paymentReference: 'Facture E2E',
				lines: [
					{ description: 'Achat', quantity: '1', unitPrice: '100.00', vatRate: '0', expenseAccountId },
				],
			},
		});
		expect(invRes.ok(), `create supplier invoice failed: ${invRes.status()}`).toBeTruthy();
		return (await invRes.json()).id as number;
	} finally {
		await disposeContextSafe(ctx);
	}
}

test.describe('Lots de paiement pain.001', () => {
	test('génère un lot et le confirme via l’UI → facture payée', async ({ page }) => {
		await login(page);
		const { expenseAccountId, bankAccountId } = await ensureSetup(page);
		const invoiceId = await createSupplierWithInvoice(page, expenseAccountId);

		// Génération du lot via API.
		const ctx = await authedApiContext(page);
		let batchId: number;
		try {
			const today = new Date().toISOString().slice(0, 10);
			const res = await ctx.post('/api/v1/payment-batches', {
				data: {
					bankAccountId,
					requestedExecutionDate: today,
					supplierInvoiceIds: [invoiceId],
				},
			});
			expect(res.ok(), `create batch failed: ${res.status()}`).toBeTruthy();
			const body = await res.json();
			expect(body.batch).not.toBeNull();
			batchId = body.batch.id as number;
		} finally {
			await disposeContextSafe(ctx);
		}

		// Détail + confirmation via UI.
		await page.goto(`/payment-batches/${batchId}`);
		await expect(page.getByTestId('payment-batch-status')).toContainText(/Généré/i);
		await page.getByTestId('payment-batch-confirm').click();
		await expect(page.getByTestId('payment-batch-status')).toContainText(/Confirmé/i);

		// Vérif backend : lot confirmé + facture payée.
		const ctx2 = await authedApiContext(page);
		try {
			const inv = await (await ctx2.get(`/api/v1/supplier-invoices/${invoiceId}`)).json();
			expect(inv.status).toBe('paid');
			expect(inv.settlementType).toBe('bank_transfer');
			// pain.001 téléchargeable.
			const pain = await ctx2.get(`/api/v1/payment-batches/${batchId}/pain001`);
			expect(pain.status()).toBe(200);
			expect(pain.headers()['content-type']).toContain('xml');
			const xml = await pain.text();
			expect(xml).toContain('pain.001.001.09');
		} finally {
			await disposeContextSafe(ctx2);
		}
	});
});
