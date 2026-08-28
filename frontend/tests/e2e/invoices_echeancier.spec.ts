import { expect, test } from '@playwright/test';
import { seedTestState, clearAuthStorage, authedApiContext, disposeContextSafe } from './helpers/test-state';

test.beforeAll(async () => {
	// Story 6.4 : preset `with-data` (= with-company + 1 contact + 1 product,
	// PAS de facture pré-seedée — les tests ci-dessous créent les leurs via
	// `daysFromToday` pour des dates déterministes).
	await seedTestState('with-data');
});

test.afterEach(async ({ page }) => {
	// Clear localStorage after each test to prevent token bleed to next test
	await clearAuthStorage(page);
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
	const ctx = await authedApiContext(page);
	try {
		const res = await ctx.post('/api/v1/contacts', {
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
	} finally {
		await disposeContextSafe(ctx);
	}
}

async function createAndValidateInvoice(
	page: import('@playwright/test').Page,
	contactId: number,
	date: string,
	dueDate: string,
	amount: string,
): Promise<number> {
	const ctx = await authedApiContext(page);
	try {
		const createRes = await ctx.post('/api/v1/invoices', {
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
		const validateRes = await ctx.post(`/api/v1/invoices/${inv.id}/validate`);
		expect(validateRes.ok(), `validate failed: ${validateRes.status()}`).toBeTruthy();
		return inv.id as number;
	} finally {
		await disposeContextSafe(ctx);
	}
}

function daysFromToday(offset: number): string {
	const d = new Date();
	d.setDate(d.getDate() + offset);
	return d.toISOString().slice(0, 10);
}

test.describe('Échéancier factures — Story 5.4', () => {
	test('golden path : création → échéancier → régler → export CSV', async ({
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

		// Story 24-3 (#372) — « Régler » remplace « Marquer payée » : le règlement
		// produit son écriture, il lui faut donc une contrepartie et un montant.
		//
		// ⚠️ Les contrôles du dialogue se ciblent par `data-testid` et non par
		// leur libellé : un libellé est traduit, et un sélecteur figé dessus
		// casse à la première relecture de la langue (règle du dépôt, #326).
		await overdueRow.getByRole('button', { name: /Régler|Settle|Pagare|Erfassen/i }).click();
		await expect(page.getByRole('dialog')).toBeVisible();
		await page.getByTestId('settle-type').selectOption('internal_account');
		// Le premier compte proposé suffit : ce cas porte sur le PARCOURS, pas sur
		// le choix du compte — celui-ci est couvert par les tests Rust.
		await page.getByTestId('settle-account').selectOption({ index: 1 });
		// ⚠️ **Le montant doit être SAISI ici, et c'est délibéré.** L'échéancier
		// ne connaît pas encore le résiduel (colonnes reportées à une issue
		// séparée), donc le dialogue reçoit `amountDue = null` — « non calculé »
		// — et laisse le champ vide plutôt que de pré-remplir.
		//
		// ⛔ Y pré-remplir le TTC de la ligne serait FAUX : sur une facture
		// déjà partiellement réglée, TTC ≠ résiduel, et l'utilisateur serait
		// conduit vers un trop-perçu que le serveur refuse. Un champ vide dit la
		// vérité ; un champ pré-rempli d'un mauvais chiffre ment.
		//
		// 100.00 HT à 8.10 % ⇒ 108.10 TTC (cf. `createAndValidateInvoice`).
		await page.getByLabel(/Montant|Amount|Importo|Betrag/i).fill('108.10');
		await page.getByTestId('settle-confirm').click();

		// Après reload, la facture en retard disparaît du filtre "Impayées".
		await expect(overdueRow).toHaveCount(0, { timeout: 5000 });

		// Basculer sur "Payées" → la facture réapparaît avec le badge Payée.
		// Regex ancrée (^…$) : `/Payées/` non ancré matcherait aussi l'onglet
		// « Im-payées » (substring) → strict-mode violation à 2 éléments.
		await page.getByRole('tab', { name: /^(Payées|Paid|Pagate|Bezahlt)$/i }).click();
		const paidRow = page.locator('tbody tr', { hasText: contactName }).first();
		await expect(paidRow).toBeVisible({ timeout: 5000 });
		await expect(paidRow.getByText(/Payée|Paid|Pagata|Bezahlt/i).first()).toBeVisible();

		// ⛔ Story 24-3 : le « dé-marquage » a DISPARU, et rien ne le remplace.
		// Annuler un règlement demande une contre-passation — une écriture
		// inverse, à sa propre date — et non le retrait d'un drapeau : c'est
		// l'objet de l'issue #414. Le cas vérifie donc l'ABSENCE du bouton, ce
		// qui est la seule assertion honnête tant que #414 n'est pas livrée.
		await page.goto(`/invoices/${overdueId}`);
		await expect(
			page.getByRole('button', { name: /Dé-marquer|Unmark|Annulla|rückgängig/i }),
		).toHaveCount(0);
		// Et le bouton de règlement s'efface aussi : la facture est soldée.
		await expect(page.getByTestId('settle-open')).toHaveCount(0);

		await page.goto('/invoices/due-dates');

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

	// Story 21-2a (#246) — la colonne Total de l'échéancier affiche le TTC.
	test('la colonne Total affiche le TTC (montant dû), pas le HT', async ({ page }) => {
		await login(page);
		const contactName = uniq('EchTtc');
		const contactId = await createContactViaApi(page, contactName);
		// 100.00 HT @ 8.1 % → TTC 108.10 (les helpers seedent vatRate 8.10).
		await createAndValidateInvoice(page, contactId, daysFromToday(-10), daysFromToday(-1), '100.00');

		await page.goto('/invoices/due-dates');
		const row = page.locator('tbody tr', { hasText: contactName }).first();
		await expect(row).toBeVisible({ timeout: 5000 });
		// Le TTC formaté suisse est affiché ; le HT nu ne l'est pas.
		await expect(row.getByText('108.10')).toBeVisible();
		await expect(row.getByText(/^100\.00$/)).toHaveCount(0);
	});

	test('page échéancier exige une session authentifiée', async ({ page }) => {
		await page.goto('/invoices/due-dates');
		// Redirect vers login si non authentifié.
		await expect(page).toHaveURL(/\/login/);
	});
});
