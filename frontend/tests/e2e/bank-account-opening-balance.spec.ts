/**
 * Story 34-2 (#533) — le solde d'ouverture saisi avec le compte bancaire (T7).
 *
 * 1. Un compte bancaire créé avec un solde de 1'234.50 *Avoir*, lié à un compte
 *    du plan neuf (créé par l'API, jamais mouvementé) : la liste montre ce solde
 *    dans la colonne *Solde comptable* — la valeur a traversé HTTP, l'écriture
 *    d'ouverture et le calcul de `list_by_company_with_balances`.
 * 2. Un second essai sur un compte du plan déjà mouvementé : le motif du
 *    serveur s'affiche dans le formulaire, et aucun compte n'est créé.
 *
 * Preset `with-company` : plan minimal SANS compte de rôle `RetainedEarnings` —
 * il est créé ici par l'API (2970), comme les comptes liés. IBAN de test, aucune
 * donnée réelle.
 */

import { expect, test, type Page } from '@playwright/test';
import {
	seedTestState,
	clearAuthStorage,
	authedApiContext,
	disposeContextSafe,
} from './helpers/test-state';

const IBAN_OK = 'CH9300762011623852957';
const IBAN_REFUS = 'CH3908704016075473007';

test.beforeEach(async () => {
	// Base vierge d'écriture pour chaque test : la portée « compte bancaire »
	// est éprouvée sur la société sans écriture, son cas nominal.
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

/** Crée un compte du plan par l'API et rend son id. */
async function createAccount(
	page: Page,
	number: string,
	name: string,
	accountType: 'Asset' | 'Liability',
	role: string | null = null,
): Promise<number> {
	const ctx = await authedApiContext(page);
	try {
		const res = await ctx.post('/api/v1/accounts', {
			data: { number, name, accountType, parentId: null, role },
		});
		expect(res.ok(), `compte ${number} : ${res.status()} ${await res.text()}`).toBeTruthy();
		return ((await res.json()) as { id: number }).id;
	} finally {
		await disposeContextSafe(ctx);
	}
}

/** Une écriture ordinaire au débit de `accountId` : le compte devient mouvementé. */
async function moveAccount(page: Page, accountId: number, counterpartId: number): Promise<void> {
	const ctx = await authedApiContext(page);
	try {
		const res = await ctx.post('/api/v1/journal-entries', {
			data: {
				entryDate: new Date().toLocaleDateString('sv-SE', { timeZone: 'Europe/Zurich' }),
				journal: 'OD',
				description: 'E2E 34-2 — compte mouvementé',
				lines: [
					{ accountId, debit: '10.00', credit: '0.00' },
					{ accountId: counterpartId, debit: '0.00', credit: '10.00' },
				],
			},
		});
		expect(res.ok(), `écriture : ${res.status()} ${await res.text()}`).toBeTruthy();
	} finally {
		await disposeContextSafe(ctx);
	}
}

async function openCreateForm(page: Page, bankName: string, iban: string, ledgerId: number) {
	await page.goto('/bank-accounts');
	await expect(page.getByTestId('bank-accounts-page-title')).toBeVisible();
	await page.getByTestId('create-bank-account-button').click();
	await page.getByTestId('form-bank-name').fill(bankName);
	await page.getByTestId('form-iban').fill(iban);
	await page.getByTestId('form-journal-account').selectOption(String(ledgerId));
}

test('un compte créé avec son solde d’ouverture affiche ce solde dans la liste', async ({ page }) => {
	await login(page);
	await createAccount(page, '2970', 'Bénéfice reporté E2E', 'Liability', 'RetainedEarnings');
	const ledger = await createAccount(page, '1025', 'Banque E2E solde', 'Asset');

	await openCreateForm(page, 'Banque Solde E2E', IBAN_OK, ledger);
	await expect(page.getByTestId('form-opening-side')).toHaveValue('debit');
	await expect(page.getByTestId('form-opening-consequence')).toBeVisible();
	await page.getByTestId('form-opening-amount').fill("1'234.50");
	await page.getByTestId('form-submit').click();

	await expect(page.getByTestId('create-bank-account-form')).toHaveCount(0);
	const row = page.locator('[data-testid^="bank-account-row-"]', { hasText: 'Banque Solde E2E' });
	await expect(row).toBeVisible();
	await expect(row.locator('[data-testid^="balance-cell-"]')).toContainText(/1.234\.50/);
});

test('un compte du plan déjà mouvementé : le motif s’affiche, aucun compte n’est créé', async ({ page }) => {
	await login(page);
	const retained = await createAccount(page, '2970', 'Bénéfice reporté E2E', 'Liability', 'RetainedEarnings');
	const ledger = await createAccount(page, '1026', 'Banque E2E mouvementée', 'Asset');
	await moveAccount(page, ledger, retained);

	await openCreateForm(page, 'Banque Refus E2E', IBAN_REFUS, ledger);
	await page.getByTestId('form-opening-amount').fill('500');
	await page.getByTestId('form-submit').click();

	await expect(page.getByTestId('form-error')).toContainText('1026');
	await expect(page.getByTestId('form-error')).toContainText('mouvements');
	await expect(page.getByTestId('create-bank-account-form')).toBeVisible();

	const ctx = await authedApiContext(page);
	try {
		const liste = (await (await ctx.get('/api/v1/bank-accounts')).json()) as Array<{ bankName: string }>;
		expect(liste.some((b) => b.bankName === 'Banque Refus E2E')).toBe(false);
	} finally {
		await disposeContextSafe(ctx);
	}
});
