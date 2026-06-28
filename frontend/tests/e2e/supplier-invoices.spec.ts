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
 * Tests E2E — Factures fournisseurs & règlement binaire (Story 12.2, #191).
 *
 * Parcours : enregistrer une facture fournisseur (API) puis la régler via l'UI
 * (compte interne), et vérifier que la facture passe « Payée ».
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

async function createSupplierViaApi(
	page: import('@playwright/test').Page,
	name: string,
): Promise<number> {
	const ctx = await authedApiContext(page);
	try {
		const res = await ctx.post('/api/v1/contacts', {
			data: {
				contactType: 'Entreprise',
				name,
				isClient: false,
				isSupplier: true,
				address: 'Rue 2\n1000 Lausanne',
				defaultPaymentTerms: '30 jours net',
			},
		});
		expect(res.ok(), `createSupplier failed: ${res.status()}`).toBeTruthy();
		return (await res.json()).id as number;
	} finally {
		await disposeContextSafe(ctx);
	}
}

/** Garantit qu'un compte créanciers (2000) est configuré + retourne un compte de charge + un compte interne. */
async function ensureConfigAndAccounts(page: import('@playwright/test').Page): Promise<{
	expenseAccountId: number;
	internalAccountId: number;
}> {
	const ctx = await authedApiContext(page);
	try {
		const accRes = await ctx.get('/api/v1/accounts');
		expect(accRes.ok()).toBeTruthy();
		const accounts = (await accRes.json()) as Array<{
			id: number;
			number: string;
			accountType: string;
			active: boolean;
		}>;
		const payable = accounts.find((a) => a.number === '2000');
		const expense = accounts.find((a) => a.accountType === 'Expense' && a.active);
		const internal = accounts.find((a) => a.accountType === 'Asset' && a.active);
		expect(payable, 'compte 2000 attendu dans le plan comptable').toBeTruthy();
		expect(expense, 'un compte de charge attendu').toBeTruthy();
		expect(internal, 'un compte Asset attendu').toBeTruthy();

		// Configurer default_payable_account_id si absent (remplacement intégral des settings).
		const sRes = await ctx.get('/api/v1/company/invoice-settings');
		expect(sRes.ok()).toBeTruthy();
		const s = await sRes.json();
		if (s.defaultPayableAccountId == null) {
			const putRes = await ctx.put('/api/v1/company/invoice-settings', {
				data: { ...s, defaultPayableAccountId: payable!.id },
			});
			expect(putRes.ok(), `set payable failed: ${putRes.status()}`).toBeTruthy();
		}
		return { expenseAccountId: expense!.id, internalAccountId: internal!.id };
	} finally {
		await disposeContextSafe(ctx);
	}
}

async function createSupplierInvoiceViaApi(
	page: import('@playwright/test').Page,
	contactId: number,
	expenseAccountId: number,
): Promise<number> {
	const today = new Date().toISOString().slice(0, 10);
	const ctx = await authedApiContext(page);
	try {
		const res = await ctx.post('/api/v1/supplier-invoices', {
			data: {
				contactId,
				supplierInvoiceNumber: uniq('FF'),
				invoiceDate: today,
				dueDate: today,
				lines: [
					{
						description: 'Achat fourniture',
						quantity: '1',
						unitPrice: '500.00',
						vatRate: '8.10',
						expenseAccountId,
					},
				],
			},
		});
		expect(res.ok(), `create supplier invoice failed: ${res.status()}`).toBeTruthy();
		return (await res.json()).id as number;
	} finally {
		await disposeContextSafe(ctx);
	}
}

test.describe('Factures fournisseurs', () => {
	test('enregistre et règle (compte interne) une facture fournisseur', async ({ page }) => {
		await login(page);
		const { expenseAccountId, internalAccountId } = await ensureConfigAndAccounts(page);
		const supplierId = await createSupplierViaApi(page, uniq('Fournisseur'));
		const invoiceId = await createSupplierInvoiceViaApi(page, supplierId, expenseAccountId);

		// La facture apparaît dans la liste.
		await page.goto('/supplier-invoices');
		await expect(page.getByTestId('supplier-invoices-table')).toBeVisible();

		// Détail → règlement par compte interne.
		await page.goto(`/supplier-invoices/${invoiceId}`);
		await expect(page.getByTestId('supplier-invoice-status')).toContainText(/Ouverte/i);
		await page.getByLabel(/Compte interne/i).check();
		await page.getByTestId('pay-internal-account').selectOption(String(internalAccountId));
		await page.getByTestId('supplier-invoice-pay-submit').click();

		// Statut → Payée.
		await expect(page.getByTestId('supplier-invoice-status')).toContainText(/Payée/i);

		// Vérif backend : statut paid + écriture de règlement liée.
		const ctx = await authedApiContext(page);
		try {
			const res = await ctx.get(`/api/v1/supplier-invoices/${invoiceId}`);
			expect(res.ok()).toBeTruthy();
			const inv = await res.json();
			expect(inv.status).toBe('paid');
			expect(inv.settlementType).toBe('internal_account');
			expect(inv.settlementJournalEntryId).not.toBeNull();
		} finally {
			await disposeContextSafe(ctx);
		}
	});
});
