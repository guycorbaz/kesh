/**
 * Story 25-6-a (#388, #389) — le tableau de bord dit vrai : les dernières
 * écritures branchées, les factures ouvertes chiffrées pour TOUS les rôles.
 *
 * Seul un E2E voit les valeurs traverser la frontière HTTP. Sélecteurs
 * `data-testid` uniquement — jamais un libellé traduit (garde
 * `e2e-selecteurs-traduits`, #326).
 *
 * Pré-requis : MariaDB up + KESH_TEST_MODE=true (cf. docs/testing.md).
 */
import { expect, test } from '@playwright/test';
import type { Page } from '@playwright/test';
import AxeBuilder from '@axe-core/playwright';
import { seedTestState, clearAuthStorage, authedApiContext, disposeContextSafe } from './helpers/test-state';
import {
	createAndValidateInvoiceViaApi,
	createContactWithAddressViaApi,
	createJournalEntryViaApi,
	ensurePrimaryBankAccountViaApi,
} from './helpers/api-fixtures';

test.beforeAll(async () => {
	await seedTestState('with-company');
});

test.afterEach(async ({ page }) => {
	await clearAuthStorage(page);
});

async function login(page: Page, username = 'admin', password = 'admin123') {
	await page.goto('/login');
	await page.fill('#username', username);
	await page.fill('#password', password);
	await page.click('button[type="submit"]');
	await expect(page).toHaveURL('/');
}

function uniq(prefix: string): string {
	return `${prefix} ${Date.now()}-${Math.floor(Math.random() * 1e6)}`;
}

test.describe('Tableau de bord — les tuiles disent vrai (25-6-a)', () => {
	test('une écriture postée apparaît dans « Dernières écritures »', async ({ page }) => {
		await login(page);
		const label = uniq('Écriture accueil');
		const id = await createJournalEntryViaApi(page, label);
		await page.goto('/');
		const row = page.getByTestId('homepage-entry-row').filter({ hasText: label });
		await expect(row).toHaveCount(1);
		await expect(row.locator(`a[href="/journal-entries/${id}"]`)).toHaveCount(1);
	});

	test('une facture validée apparaît chiffrée dans « Factures ouvertes »', async ({ page }) => {
		await login(page);
		await ensurePrimaryBankAccountViaApi(page);
		const contact = await createContactWithAddressViaApi(page, uniq('Accueil SA'));
		await createAndValidateInvoiceViaApi(page, contact);
		await page.goto('/');
		const count = page.getByTestId('homepage-invoices-open-count');
		await expect(count).toBeVisible();
		const amount = await count.getAttribute('data-amount');
		expect(Number.parseFloat(amount ?? '0'), 'reste dû total').toBeGreaterThan(0);
	});

	test('un rôle Consultation voit les factures ouvertes chiffrées, sans bouton d’action', async ({
		page,
	}) => {
		await login(page);
		const contact = await createContactWithAddressViaApi(page, uniq('Accueil Consult SA'));
		await createAndValidateInvoiceViaApi(page, contact);
		const username = `consult-tiles-${Date.now()}`;
		const ctx = await authedApiContext(page);
		try {
			const res = await ctx.post('/api/v1/users', {
				data: { username, password: 'MotDePasse12345', role: 'Consultation' },
			});
			expect(res.ok(), `create user failed: ${res.status()}`).toBeTruthy();
		} finally {
			await disposeContextSafe(ctx);
		}
		await clearAuthStorage(page);
		await login(page, username, 'MotDePasse12345');

		await expect(page.getByTestId('homepage-invoices-open-count')).toBeVisible();
		// Présence avant absence : la tuile des écritures est chargée (liste ou
		// texte vide) avant d'affirmer qu'elle n'a pas de bouton.
		const entriesCard = page.getByTestId('homepage-card-recent-entries');
		await expect(entriesCard.locator('ul, p').first()).toBeVisible();
		const invoicesCard = page.getByTestId('homepage-card-open-invoices');
		await expect(invoicesCard.locator('a[href="/invoices"]')).toHaveCount(0);
		await expect(entriesCard.locator('a[href="/journal-entries"]')).toHaveCount(0);
	});

	test('axe-core sans violations sur l’accueil peuplé', async ({ page }) => {
		await login(page);
		await ensurePrimaryBankAccountViaApi(page);
		await createJournalEntryViaApi(page, uniq('Écriture axe'));
		const contact = await createContactWithAddressViaApi(page, uniq('Axe SA'));
		await createAndValidateInvoiceViaApi(page, contact);
		await page.goto('/');
		await expect(page.getByTestId('homepage-entry-row').first()).toBeVisible();
		await expect(page.getByTestId('homepage-invoices-open-count')).toBeVisible();
		const results = await new AxeBuilder({ page }).analyze();
		expect(results.violations).toEqual([]);
	});
});
