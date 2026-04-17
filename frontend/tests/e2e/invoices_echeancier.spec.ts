import { expect, test } from '@playwright/test';
import { seedTestState } from './helpers/test-state';

test.beforeAll(async () => {
	// Story 6.4 : preset `with-data` (= with-company + 1 contact + 1 product,
	// PAS de facture pré-seedée — les tests ci-dessous créent les leurs via
	// `daysFromToday` pour des dates déterministes).
	await seedTestState('with-data');
});

/**
 * Tests E2E — Échéancier factures (Story 5.4)
 *
 * Golden path : créer + valider 2 factures (une en retard, une future) →
 * naviguer /invoices/due-dates → marquer la passée payée → vérifier qu'elle
 * disparaît de « Impayées » → basculer sur « Payées » → vérifier le badge →
 * dé-marquer depuis la page détail → export CSV.
 *
 * Prérequis seed (identique aux autres `invoices.spec.ts`) : admin bootstrap,
 * une company, fiscal_year couvrant aujourd'hui, company_invoice_settings
 * avec comptes par défaut configurés.
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

async function createContactViaApi(
	page: import('@playwright/test').Page,
	name: string,
): Promise<number> {
	const res = await page.request.post('/api/v1/contacts', {
		data: {
			contactType: 'Entreprise',
			name,
			isClient: true,
			isSupplier: false,
			defaultPaymentTerms: '30 jours net',
		},
	});
	expect(res.ok(), `createContact failed: ${res.status()}`).toBeTruthy();
	return (await res.json()).id as number;
}

async function createAndValidateInvoice(
	page: import('@playwright/test').Page,
	contactId: number,
	date: string,
	dueDate: string,
	amount: string,
): Promise<number> {
	const createRes = await page.request.post('/api/v1/invoices', {
		data: {
			contactId,
			date,
			dueDate,
			paymentTerms: null,
			lines: [
				{ description: 'Prestation', quantity: '1', unitPrice: amount, vatRate: '8.10' },
			],
		},
	});
	expect(createRes.ok(), `create invoice failed: ${createRes.status()}`).toBeTruthy();
	const inv = await createRes.json();
	const validateRes = await page.request.post(`/api/v1/invoices/${inv.id}/validate`);
	expect(validateRes.ok(), `validate failed: ${validateRes.status()}`).toBeTruthy();
	return inv.id as number;
}

function daysFromToday(offset: number): string {
	const d = new Date();
	d.setDate(d.getDate() + offset);
	return d.toISOString().slice(0, 10);
}

test.describe('Échéancier factures — Story 5.4', () => {
	test('golden path : création → échéancier → marquer payée → dé-marquer → export CSV', async ({
		page,
	}) => {
		await login(page);

		const contactName = uniq('EchContact');
		const contactId = await createContactViaApi(page, contactName);

		// Facture en retard (due_date = hier) + facture future.
		const overdueId = await createAndValidateInvoice(
			page,
			contactId,
			daysFromToday(-30),
			daysFromToday(-1),
			'100.00',
		);
		const futureId = await createAndValidateInvoice(
			page,
			contactId,
			daysFromToday(0),
			daysFromToday(30),
			'250.00',
		);

		await page.goto('/invoices/due-dates');
		await expect(page.getByRole('heading', { name: /Échéancier/i })).toBeVisible();

		// Les 2 doivent apparaître (filtre défaut = unpaid).
		const overdueRow = page.locator('tbody tr', { hasText: contactName }).filter({
			hasText: daysFromToday(-1),
		});
		await expect(overdueRow).toBeVisible({ timeout: 5000 });
		// Badge "En retard" présent au moins une fois dans le tableau.
		await expect(page.locator('tbody').getByText(/En retard|Overdue|In ritardo|Überfällig/i).first()).toBeVisible();

		// Cliquer "Marquer payée" sur la facture en retard.
		await overdueRow.getByRole('button', { name: /Marquer payée|Mark as paid|Segna|Als bezahlt/i }).click();
		// Dialog s'ouvre → confirmer (date défaut = today).
		await expect(page.getByRole('dialog')).toBeVisible();
		await page.getByRole('button', { name: /Confirmer|Confirm|Conferma|bestätigen/i }).click();

		// Après reload, la facture en retard disparaît du filtre "Impayées".
		await expect(overdueRow).toHaveCount(0, { timeout: 5000 });

		// Basculer sur "Payées" → la facture réapparaît avec le badge Payée.
		await page.getByRole('tab', { name: /Payées|Paid|Pagate|Bezahlt/i }).click();
		const paidRow = page.locator('tbody tr', { hasText: contactName }).first();
		await expect(paidRow).toBeVisible({ timeout: 5000 });
		await expect(paidRow.getByText(/Payée|Paid|Pagata|Bezahlt/i).first()).toBeVisible();

		// Naviguer vers la page détail → dé-marquer.
		await page.goto(`/invoices/${overdueId}`);
		await page.getByRole('button', { name: /Dé-marquer|Unmark|Annulla|rückgängig/i }).click();
		await page.getByRole('dialog').getByRole('button', { name: /Dé-marquer|Unmark|Annulla|rückgängig/i }).click();

		// Retour échéancier : la facture réapparaît en Impayées/En retard.
		await page.goto('/invoices/due-dates');
		const backRow = page.locator('tbody tr', { hasText: contactName }).filter({
			hasText: daysFromToday(-1),
		});
		await expect(backRow).toBeVisible({ timeout: 5000 });

		// Export CSV : intercepter le téléchargement.
		const [download] = await Promise.all([
			page.waitForEvent('download'),
			page.getByRole('button', { name: /Exporter|Export/i }).click(),
		]);
		expect(download.suggestedFilename()).toMatch(/^echeancier-\d{4}-\d{2}-\d{2}\.csv$/);

		// Sanity : les 2 IDs existent (pas orphelins).
		expect(overdueId).toBeGreaterThan(0);
		expect(futureId).toBeGreaterThan(0);
	});

	test('page échéancier exige une session authentifiée', async ({ page }) => {
		await page.goto('/invoices/due-dates');
		// Redirect vers login si non authentifié.
		await expect(page).toHaveURL(/\/login/);
	});
});
