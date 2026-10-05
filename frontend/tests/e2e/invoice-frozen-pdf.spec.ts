import { expect, test } from '@playwright/test';
import {
	seedTestState,
	clearAuthStorage,
	authedApiContext,
	disposeContextSafe,
} from './helpers/test-state';
import {
	createContactWithAddressViaApi,
	createAndValidateInvoiceViaApi,
	ensurePrimaryBankAccountViaApi,
} from './helpers/api-fixtures';

/**
 * Tests E2E — le PDF d'une facture est figé (Story 25-6-b, #387).
 *
 * Seul un E2E voit les octets traverser la frontière HTTP jusqu'au client. Les
 * deux parcours : deux téléchargements rendent le même fichier ; une facture
 * annulée par un avoir garde le PDF qu'elle a émis.
 *
 * ⚠️ Chaque test crée **sa propre** facture : les fichiers figés persistent
 * sous `KESH_DOCUMENTS_DIR` (`/tmp/kesh-e2e/documents`) d'une exécution à
 * l'autre, et une facture déjà figée par un run précédent ne prouverait rien.
 */

test.beforeAll(async () => {
	await seedTestState('with-company');
});

test.afterEach(async ({ page }) => {
	await clearAuthStorage(page);
});

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

test.describe('PDF figé', () => {
	test('deux téléchargements rendent le même fichier', async ({ page }) => {
		await login(page);
		// Le PDF exige un compte bancaire principal.
		await ensurePrimaryBankAccountViaApi(page);
		const contactId = await createContactWithAddressViaApi(page, uniq('PDF Figé'));
		const invoiceId = await createAndValidateInvoiceViaApi(page, contactId);

		const ctx = await authedApiContext(page);
		try {
			const premier = await ctx.get(`/api/v1/invoices/${invoiceId}/pdf`);
			expect(premier.status()).toBe(200);
			const second = await ctx.get(`/api/v1/invoices/${invoiceId}/pdf`);
			expect(second.status()).toBe(200);
			const a = await premier.body();
			const b = await second.body();
			expect(a.subarray(0, 7).toString('utf8')).toMatch(/^%PDF-1\./);
			// Avant la story, la date de création et l'`/ID` du PDF changeaient.
			expect(b.equals(a)).toBe(true);
		} finally {
			await disposeContextSafe(ctx);
		}

		// La fiche dit que le document est figé.
		await page.goto(`/invoices/${invoiceId}`);
		await expect(page.getByTestId('invoice-pdf-frozen-at')).toBeVisible();
	});

	test('une facture annulée par un avoir garde son PDF émis', async ({ page }) => {
		await login(page);
		// Le PDF exige un compte bancaire principal.
		await ensurePrimaryBankAccountViaApi(page);
		const contactId = await createContactWithAddressViaApi(page, uniq('PDF Annulée'));
		const invoiceId = await createAndValidateInvoiceViaApi(page, contactId);

		const ctx = await authedApiContext(page);
		try {
			const emis = await ctx.get(`/api/v1/invoices/${invoiceId}/pdf`);
			expect(emis.status()).toBe(200);
			const avant = await emis.body();

			const cn = await ctx.post('/api/v1/credit-notes', {
				data: { invoiceId, date: new Date().toISOString().slice(0, 10) },
			});
			expect(cn.ok(), `create credit note failed: ${cn.status()}`).toBeTruthy();
			const inv = await ctx.get(`/api/v1/invoices/${invoiceId}`);
			expect((await inv.json()).status).toBe('cancelled');

			const apres = await ctx.get(`/api/v1/invoices/${invoiceId}/pdf`);
			expect(apres.status()).toBe(200);
			expect((await apres.body()).equals(avant)).toBe(true);
		} finally {
			await disposeContextSafe(ctx);
		}

		// Le bouton PDF reste sur la fiche de la facture annulée.
		await page.goto(`/invoices/${invoiceId}`);
		await expect(page.getByTestId('invoice-download-pdf')).toBeVisible();
	});
});
