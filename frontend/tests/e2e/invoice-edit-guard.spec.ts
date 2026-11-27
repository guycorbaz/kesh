import { expect, test } from '@playwright/test';
import {
	seedTestState,
	clearAuthStorage,
	authedApiContext,
	disposeContextSafe,
} from './helpers/test-state';
import {
	createContactWithAddressViaApi,
	ensurePrimaryBankAccountViaApi,
	createAndValidateInvoiceViaApi,
	createUser,
	USER_PASSWORD,
} from './helpers/api-fixtures';

/**
 * Tests E2E — Story 64-1 (#1758) : la page `/invoices/{id}/edit` refuse d'entrée une
 * facture qui n'est plus un brouillon, et un rôle sans droit d'écriture ; un brouillon
 * garde son formulaire.
 */

test.beforeAll(async () => {
	await seedTestState('with-company');
});

test.afterEach(async ({ page }) => {
	await clearAuthStorage(page);
});

async function login(page: import('@playwright/test').Page, user = 'admin', pwd = 'admin123') {
	await page.goto('/login');
	await page.fill('#username', user);
	await page.fill('#password', pwd);
	await page.click('button[type="submit"]');
	await expect(page).toHaveURL('/');
}

async function createDraftViaApi(
	page: import('@playwright/test').Page,
	contactId: number,
): Promise<number> {
	const ctx = await authedApiContext(page);
	try {
		const today = new Date().toISOString().slice(0, 10);
		const res = await ctx.post('/api/v1/invoices', {
			data: {
				contactId,
				date: today,
				dueDate: today,
				paymentTerms: '30 jours net',
				lines: [{ description: 'Brouillon 64-1', quantity: '1', unitPrice: '100.00', vatRate: '8.10' }],
			},
		});
		expect(res.ok(), `création du brouillon : ${res.status()}`).toBeTruthy();
		return (await res.json()).id as number;
	} finally {
		await disposeContextSafe(ctx);
	}
}

test.describe('Page de modification d’une facture (64-1)', () => {
	test('brouillon : le formulaire ; validée : avis et lien, pas de formulaire ; Consultation : avis de rôle', async ({
		page,
	}) => {
		const consultation = await createUser(page, 'Consultation');
		await login(page);
		await ensurePrimaryBankAccountViaApi(page);
		const contactId = await createContactWithAddressViaApi(page, `Client 64-1 ${Date.now()}`);
		const draftId = await createDraftViaApi(page, contactId);
		const validatedId = await createAndValidateInvoiceViaApi(page, contactId);

		// Brouillon, Administrateur : le formulaire (sonde positive : le <form> d'InvoiceForm).
		await page.goto(`/invoices/${draftId}/edit`);
		await expect(page.locator('form')).toHaveCount(1);
		await expect(page.getByTestId('invoice-edit-not-draft')).toHaveCount(0);
		await expect(page.getByTestId('invoice-edit-forbidden')).toHaveCount(0);

		// Facture validée : l'avis et le lien, jamais le formulaire.
		await page.goto(`/invoices/${validatedId}/edit`);
		await expect(page.getByTestId('invoice-edit-not-draft')).toBeVisible();
		await expect(page.getByTestId('invoice-edit-back')).toHaveAttribute(
			'href',
			`/invoices/${validatedId}`,
		);
		await expect(page.locator('form')).toHaveCount(0);

		// Consultation, sur un brouillon : l'avis de rôle, pas de formulaire.
		await clearAuthStorage(page);
		await login(page, consultation, USER_PASSWORD);
		await page.goto(`/invoices/${draftId}/edit`);
		await expect(page.getByTestId('invoice-edit-forbidden')).toBeVisible();
		await expect(page.getByTestId('invoice-edit-back')).toHaveAttribute(
			'href',
			`/invoices/${draftId}`,
		);
		await expect(page.locator('form')).toHaveCount(0);
	});
});
