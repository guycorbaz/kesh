/**
 * Story 49-4a (#1364, AC5) — la capacité du PDF se contrôle à la validation.
 *
 * (a) Un brouillon de neuf lignes chiffrées (créé par l'API, sans TVA, total rond : aucun
 * récapitulatif, place de huit lignes) → *Valider* → le refus de capacité est visible dans le
 * dialogue, avec ses deux nombres (9 et 8) ; la facture reste un brouillon, sans numéro.
 * (b) Une ligne retirée, la validation réussit et le PDF se télécharge (`200`).
 *
 * Aucun fichier de `frontend/src` ne change (D4) : l'écran affiche le message du serveur.
 *
 * Pré-requis : MariaDB up + KESH_TEST_MODE=true (cf. docs/testing.md).
 */

import { expect, test, type Page } from '@playwright/test';
import {
	seedTestState,
	clearAuthStorage,
	authedApiContext,
	disposeContextSafe,
} from './helpers/test-state';
import {
	createContactWithAddressViaApi,
	ensurePrimaryBankAccountViaApi,
	todayInZurich,
} from './helpers/api-fixtures';

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

/** `n` lignes chiffrées de 100.00 sans TVA. */
function lignes(n: number) {
	return Array.from({ length: n }, (_, i) => ({
		description: `Prestation ${i + 1}`,
		quantity: '1',
		unitPrice: '100.00',
		vatRate: '0',
	}));
}

test('une facture qui ne tient pas sur la page de son PDF est refusée à la validation', async ({
	page,
}) => {
	await login(page);
	await ensurePrimaryBankAccountViaApi(page);
	const contactId = await createContactWithAddressViaApi(
		page,
		`Client Capacité ${Date.now()}-${Math.floor(Math.random() * 1e6)}`,
	);
	const today = todayInZurich();
	const ctx = await authedApiContext(page);
	let invoiceId = 0;
	try {
		const created = await ctx.post('/api/v1/invoices', {
			data: { contactId, date: today, dueDate: today, lines: lignes(9) },
		});
		expect(created.ok(), `création : ${created.status()}`).toBeTruthy();
		invoiceId = (await created.json()).id;

		// (a) Le refus, visible dans le dialogue, avec ses deux nombres.
		await page.goto(`/invoices/${invoiceId}`);
		await page.getByTestId('invoice-validate-button').click();
		await page.getByTestId('invoice-validate-confirm').click();
		const error = page.getByTestId('invoice-validate-error');
		await expect(error).toBeVisible();
		await expect(error).toContainText('ne tiennent pas sur la page de son PDF');
		await expect(error).toContainText('elles demandent 9 lignes, la page en offre 8');
		await expect(error).toContainText('La facture reste un brouillon.');

		const brouillon = await (await ctx.get(`/api/v1/invoices/${invoiceId}`)).json();
		expect(brouillon.status).toBe('draft');
		expect(brouillon.invoiceNumber ?? null).toBeNull();
		expect(brouillon.journalEntryId ?? null).toBeNull();

		// (b) Une ligne retirée, la validation passe et le PDF se rend.
		const modifie = await ctx.put(`/api/v1/invoices/${invoiceId}`, {
			data: {
				contactId,
				date: today,
				dueDate: today,
				lines: lignes(8),
				version: brouillon.version,
			},
		});
		expect(modifie.ok(), `modification : ${modifie.status()}`).toBeTruthy();

		await page.goto(`/invoices/${invoiceId}`);
		await page.getByTestId('invoice-validate-button').click();
		const valide = page.waitForResponse(
			(r) => r.url().endsWith(`/api/v1/invoices/${invoiceId}/validate`),
		);
		await page.getByTestId('invoice-validate-confirm').click();
		expect((await valide).status()).toBe(200);
		await expect(page.getByTestId('invoice-validate-dialog')).toBeHidden();

		const pdf = await ctx.get(`/api/v1/invoices/${invoiceId}/pdf`);
		expect(pdf.status()).toBe(200);
		expect((await pdf.body()).subarray(0, 7).toString('utf8')).toMatch(/^%PDF-1\./);
	} finally {
		await disposeContextSafe(ctx);
	}
});
