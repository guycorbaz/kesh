/**
 * Story v017-4 (#849, AC-30) — le cycle brouillon → validée → payée d'une facture
 * fournisseur, de bout en bout.
 *
 * **Écrite en deux temps** (C-v017-4-31, dérogation) : la 4a livre les étapes **par
 * l'API** — ce fichier tel quel —, la 4b y ajoute les étapes **par l'écran** (formulaire,
 * fiche, page d'édition). Les étapes API prouvent ce qu'aucun test Rust ne voit : que les
 * valeurs traversent la frontière HTTP du serveur réellement servi.
 *
 * Décor synthétique : le seed `with-company`, un fournisseur et un compte bancaire créés
 * par l'API, un relevé **saisi à la main** par l'API (débit égal au total), équilibré au
 * centime.
 */
import { expect, test, type APIRequestContext, type Page } from '@playwright/test';
import {
	seedTestState,
	clearAuthStorage,
	authedApiContext,
	disposeContextSafe,
} from './helpers/test-state';
import {
	createManualDebitStatementViaApi,
	wiredBankAccountViaApi,
} from './helpers/api-fixtures';

/** IBAN synthétique du compte bancaire du décor. */
const TEST_IBAN = 'CH5604835012345678009';
const SUPPLIER = 'Fournisseur cycle brouillon SA';

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

type Account = {
	id: number;
	number: string;
	accountType: string;
	active: boolean;
	postable: boolean;
};

/** Le fournisseur du décor et deux comptes de charge distincts (le « faux » et le bon). */
async function decor(ctx: APIRequestContext): Promise<{
	supplierId: number;
	wrong: number;
	right: number;
}> {
	const accounts = (await (await ctx.get('/api/v1/accounts')).json()) as Account[];
	const expenses = accounts.filter((a) => a.accountType === 'Expense' && a.active && a.postable);
	expect(expenses.length, 'deux comptes de charge imputables').toBeGreaterThanOrEqual(2);
	const payable = accounts.find((a) => a.number === '2000');
	const settings = await (await ctx.get('/api/v1/company/invoice-settings')).json();
	if (settings.defaultPayableAccountId == null) {
		const put = await ctx.put('/api/v1/company/invoice-settings', {
			data: { ...settings, defaultPayableAccountId: payable!.id },
		});
		expect(put.ok(), `compte créanciers : ${put.status()}`).toBeTruthy();
	}
	const sup = await ctx.post('/api/v1/contacts', {
		data: { contactType: 'Entreprise', name: SUPPLIER, isClient: false, isSupplier: true },
	});
	expect(sup.ok(), `fournisseur : ${sup.status()}`).toBeTruthy();
	return {
		supplierId: (await sup.json()).id as number,
		wrong: expenses[0].id,
		right: expenses[1].id,
	};
}

function body(supplierId: number, account: number, price: string, number: string) {
	return {
		contactId: supplierId,
		supplierInvoiceNumber: number,
		invoiceDate: '2026-05-10',
		dueDate: null,
		lines: [
			{
				description: 'Fourniture',
				quantity: '1',
				unitPrice: price,
				vatRate: '0',
				expenseAccountId: account,
			},
		],
	};
}

test.describe('Cycle brouillon → validée → payée (étapes API, 4a)', () => {
	test('brouillon, correction, validation, dévalidation, revalidation, puis payée par la réconciliation', async ({
		page,
	}) => {
		await login(page);
		const bankAccountId = await wiredBankAccountViaApi(page, TEST_IBAN);
		const ctx = await authedApiContext(page);
		try {
			const d = await decor(ctx);

			// Brouillon, avec un compte de charge faux : aucune écriture.
			const created = await ctx.post('/api/v1/supplier-invoices', {
				data: body(d.supplierId, d.wrong, '100.20', 'FF-CYCLE-1'),
			});
			expect(created.status()).toBe(201);
			let inv = await created.json();
			expect(inv.status).toBe('draft');
			expect(inv.purchaseJournalEntryId).toBeNull();

			// Correction du compte (PUT), puis validation : l'écriture est passée.
			let res = await ctx.put(`/api/v1/supplier-invoices/${inv.id}`, {
				data: { ...body(d.supplierId, d.right, '100.20', 'FF-CYCLE-1'), version: inv.version },
			});
			expect(res.status(), await res.text()).toBe(200);
			inv = await res.json();
			res = await ctx.post(`/api/v1/supplier-invoices/${inv.id}/validate`, {
				data: { version: inv.version },
			});
			expect(res.status(), await res.text()).toBe(200);
			inv = await res.json();
			expect(inv.status).toBe('open');
			expect(typeof inv.purchaseJournalEntryId).toBe('number');

			// Dévalidation : l'écriture d'achat est supprimée.
			const entry = inv.purchaseJournalEntryId as number;
			res = await ctx.post(`/api/v1/supplier-invoices/${inv.id}/unvalidate`, {
				data: { version: inv.version },
			});
			expect(res.status(), await res.text()).toBe(200);
			inv = await res.json();
			expect(inv.status).toBe('draft');
			expect(inv.purchaseJournalEntryId).toBeNull();
			expect((await ctx.get(`/api/v1/journal-entries/${entry}`)).status()).toBe(404);

			// Prix corrigé (100.20 → 100.00), puis validation.
			res = await ctx.put(`/api/v1/supplier-invoices/${inv.id}`, {
				data: { ...body(d.supplierId, d.right, '100.00', 'FF-CYCLE-1'), version: inv.version },
			});
			expect(res.status(), await res.text()).toBe(200);
			inv = await res.json();
			expect(Number(inv.totalAmount)).toBe(100);
			res = await ctx.post(`/api/v1/supplier-invoices/${inv.id}/validate`, {
				data: { version: inv.version },
			});
			expect(res.status(), await res.text()).toBe(200);

			// Un relevé saisi à la main porte un débit égal : la facture est proposée…
			await createManualDebitStatementViaApi(page, {
				bankAccountId,
				bookingDate: '2026-05-20',
				amount: '100.00',
				counterpartyName: SUPPLIER,
				details: 'Paiement fournisseur cycle',
			});
			const props = await (
				await ctx.get(`/api/v1/reconciliation/proposals?bankAccountId=${bankAccountId}`)
			).json();
			const proposal = (props.proposals as Array<Record<string, unknown>>).find((p) =>
				(p.candidates as Array<Record<string, unknown>>).some(
					(c) => c.supplierInvoiceId === inv.id,
				),
			);
			expect(proposal, 'la facture validée est proposée pour le débit').toBeTruthy();

			// … et l'accepter la rend payée.
			res = await ctx.post('/api/v1/reconciliation/accept', {
				data: {
					bankAccountId,
					proposals: [
						{
							type: 'supplier_invoice',
							bankTransactionId: proposal!.bankTransactionId,
							supplierInvoiceId: inv.id,
						},
					],
				},
			});
			expect(res.status()).toBe(200);
			const accepted = await res.json();
			expect(accepted.failed, JSON.stringify(accepted)).toHaveLength(0);
			const paid = await (await ctx.get(`/api/v1/supplier-invoices/${inv.id}`)).json();
			expect(paid.status).toBe('paid');
		} finally {
			await disposeContextSafe(ctx);
		}
	});

	test("un brouillon au montant d'un débit n'est pas proposé, et l'administrateur le supprime", async ({
		page,
	}) => {
		await login(page);
		const bankAccountId = await wiredBankAccountViaApi(page, TEST_IBAN);
		const ctx = await authedApiContext(page);
		try {
			const d = await decor(ctx);
			const created = await ctx.post('/api/v1/supplier-invoices', {
				data: body(d.supplierId, d.right, '77.70', 'FF-CYCLE-2'),
			});
			expect(created.status()).toBe(201);
			const draft = await created.json();
			await createManualDebitStatementViaApi(page, {
				bankAccountId,
				bookingDate: '2026-06-12',
				amount: '77.70',
				counterpartyName: SUPPLIER,
				details: 'Débit au montant du brouillon',
			});
			const props = await (
				await ctx.get(`/api/v1/reconciliation/proposals?bankAccountId=${bankAccountId}`)
			).json();
			for (const p of props.proposals as Array<Record<string, unknown>>) {
				for (const c of p.candidates as Array<Record<string, unknown>>) {
					expect(c.supplierInvoiceId, 'un brouillon n’est jamais proposé').not.toBe(draft.id);
				}
			}
			const del = await ctx.delete(`/api/v1/supplier-invoices/${draft.id}`);
			expect(del.status()).toBe(204);
			expect((await ctx.get(`/api/v1/supplier-invoices/${draft.id}`)).status()).toBe(404);
		} finally {
			await disposeContextSafe(ctx);
		}
	});
});
