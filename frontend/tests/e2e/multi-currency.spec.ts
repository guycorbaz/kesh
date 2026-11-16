/**
 * Story 37-1 (AC-3, AC-4, AC-7, CR-7) — le multi-devises, de l'écran au serveur.
 *
 * 1. En mono-devise, le *Plan comptable* est inchangé : ni champ ni colonne
 *    « Devise ».
 * 2. *Paramètres → Comptabilité* : la case cochée ouvre les trois sélecteurs ;
 *    *Enregistrer* ouvre une confirmation (activation définitive). *Annuler*
 *    n'envoie aucun `PUT` ; *Activer* active, et la case reste cochée et
 *    désactivée, après rechargement compris.
 * 3. *Plan comptable* en multi-devises : un compte USD créé par l'écran (champ
 *    « Devise ») porte sa devise, au serveur et dans la liste.
 * 4. Verrou (AC-4) : une écriture passée par l'API sur un compte **encore CHF**
 *    — une ligne CHF sur un compte déjà en devise serait refusée par le contrôle
 *    central de la 37-3a — fige sa devise : le champ est désactivé avec
 *    l'explication, et le serveur refuse `ACCOUNT_CURRENCY_LOCKED`.
 *
 * Aucun plan livré n'a de compte de conversion ni de compte de différences de
 * change : les deux sont créés par l'API. L'activation est irréversible : la
 * spec repart d'un `with-company` neuf et en reseede un en fin de parcours,
 * pour que les specs suivantes retrouvent une société mono-devise.
 */

import { expect, test, type APIRequestContext, type Page } from '@playwright/test';
import {
	seedTestState,
	clearAuthStorage,
	authedApiContext,
	disposeContextSafe,
} from './helpers/test-state';
import { todayInZurich } from './helpers/api-fixtures';

test.describe.configure({ mode: 'serial' });

test.beforeAll(async () => {
	await seedTestState('with-company');
});

test.afterAll(async () => {
	// L'activation ne se défait pas : seule la remise à zéro rend le mono-devise.
	await seedTestState('with-company');
});

test.afterEach(async ({ page }) => {
	await clearAuthStorage(page);
});

interface AccountJson {
	id: number;
	number: string;
	name: string;
	accountType: string;
	role: string | null;
	postable: boolean;
	version: number;
	/** Absent pour un compte en CHF. */
	currency?: string;
	/** Absent quand la devise n'est pas verrouillée. */
	currencyLocked?: boolean;
}

const TRANSIT_NUMBER = '1090';
const FX_NUMBER = '6840';
const USD_NUMBER = '1025';
const LOCKED_NUMBER = '1030';

async function login(page: Page): Promise<void> {
	await page.goto('/login');
	await page.fill('#username', 'admin');
	await page.fill('#password', 'admin123');
	await page.click('button[type="submit"]');
	await expect(page).toHaveURL('/');
}

async function createAccount(
	ctx: APIRequestContext,
	number: string,
	name: string,
	accountType: string,
): Promise<number> {
	const res = await ctx.post('/api/v1/accounts', { data: { number, name, accountType } });
	expect(res.ok(), `compte ${number} : ${res.status()} ${await res.text()}`).toBeTruthy();
	return ((await res.json()) as { id: number }).id;
}

async function accountByNumber(ctx: APIRequestContext, number: string): Promise<AccountJson> {
	const res = await ctx.get('/api/v1/accounts');
	expect(res.ok(), `liste des comptes : ${res.status()}`).toBeTruthy();
	const found = ((await res.json()) as AccountJson[]).find((a) => a.number === number);
	expect(found, `compte ${number} présent`).toBeTruthy();
	return found!;
}

async function companyMultiCurrency(ctx: APIRequestContext): Promise<boolean> {
	const res = await ctx.get('/api/v1/companies/current');
	expect(res.ok(), `société : ${res.status()}`).toBeTruthy();
	return ((await res.json()) as { company: { multiCurrencyEnabled: boolean } }).company
		.multiCurrencyEnabled;
}

test('en mono-devise, le plan comptable n’a ni champ ni colonne « Devise »', async ({ page }) => {
	await login(page);
	await page.goto('/accounts');
	await expect(page.locator('[data-testid="account-row-1000"]')).toBeVisible();
	await expect(page.locator('[data-testid$="-currency"]')).toHaveCount(0);

	await page.getByTestId('account-create-button').click();
	await expect(page.getByRole('dialog')).toBeVisible();
	await expect(page.locator('#create-number')).toBeVisible();
	await expect(page.getByTestId('account-create-dialog-currency')).toHaveCount(0);
	await page.getByTestId('account-create-dialog-cancel').click();
});

test('Paramètres → Comptabilité : confirmation, Annuler n’envoie rien, Activer active', async ({
	page,
}) => {
	await login(page);
	const ctx = await authedApiContext(page);
	let transitId: number;
	let fxId: number;
	try {
		transitId = await createAccount(ctx, TRANSIT_NUMBER, 'Compte de conversion', 'Asset');
		fxId = await createAccount(ctx, FX_NUMBER, 'Différences de change', 'Expense');
		expect(await companyMultiCurrency(ctx)).toBe(false);
	} finally {
		await disposeContextSafe(ctx);
	}

	const puts: string[] = [];
	page.on('request', (r) => {
		if (r.method() === 'PUT' && r.url().includes('/companies/current/currency-settings')) {
			puts.push(r.url());
		}
	});

	await page.goto('/settings');
	const box = page.getByTestId('settings-currency-enabled');
	await expect(box).toBeVisible();
	await expect(box).not.toBeChecked();
	await expect(box).toBeEnabled();
	await expect(page.getByTestId('settings-currency-gain')).toHaveCount(0);
	await expect(page.getByTestId('settings-currency-help-definitive')).toBeVisible();

	await box.check();
	await page.getByTestId('settings-currency-gain').selectOption(String(fxId));
	await page.getByTestId('settings-currency-loss').selectOption(String(fxId));
	await page.getByTestId('settings-currency-transit').selectOption(String(transitId));

	// Enregistrer ouvre la confirmation ; Annuler la ferme sans rien envoyer.
	await page.getByTestId('settings-currency-save').click();
	const dialog = page.getByRole('dialog');
	await expect(dialog).toBeVisible();
	await expect(dialog).toContainText('ne pourra plus être désactivé');
	await page.getByTestId('settings-currency-confirm-cancel').click();
	await expect(dialog).toBeHidden();
	expect(puts, 'Annuler n’envoie aucun PUT').toHaveLength(0);

	const check = await authedApiContext(page);
	try {
		expect(await companyMultiCurrency(check), 'toujours mono-devise').toBe(false);
	} finally {
		await disposeContextSafe(check);
	}

	// Activer envoie le PUT ; la case reste cochée et désactivée.
	await page.getByTestId('settings-currency-save').click();
	await expect(dialog).toBeVisible();
	const saved = page.waitForResponse(
		(r) =>
			r.url().includes('/companies/current/currency-settings') && r.request().method() === 'PUT',
	);
	await page.getByTestId('settings-currency-confirm-activate').click();
	const res = await saved;
	expect(res.status(), await res.text()).toBe(200);
	expect(puts).toHaveLength(1);
	await expect(dialog).toBeHidden();
	await expect(box).toBeChecked();
	await expect(box).toBeDisabled();

	await page.reload();
	await expect(page.getByTestId('settings-currency-enabled')).toBeChecked();
	await expect(page.getByTestId('settings-currency-enabled')).toBeDisabled();
	await expect(page.getByTestId('settings-currency-transit')).toHaveValue(String(transitId));
	// Les trois comptes restent modifiables.
	await expect(page.getByTestId('settings-currency-gain')).toBeEnabled();

	const after = await authedApiContext(page);
	try {
		expect(await companyMultiCurrency(after), 'multi-devises actif au serveur').toBe(true);
	} finally {
		await disposeContextSafe(after);
	}
});

test('Plan comptable : un compte USD créé par l’écran porte sa devise', async ({ page }) => {
	await login(page);
	await page.goto('/accounts');
	await expect(page.locator('[data-testid="account-row-1000"]')).toBeVisible();

	await page.getByTestId('account-create-button').click();
	await expect(page.getByRole('dialog')).toBeVisible();
	await page.fill('#create-number', USD_NUMBER);
	await page.fill('#create-name', 'Banque USD');
	const currency = page.getByTestId('account-create-dialog-currency');
	await expect(currency).toBeEnabled();
	await currency.selectOption('USD');
	// Un compte en devise : le choix des comptes de différence, « par défaut » coché.
	await expect(page.getByTestId('account-create-dialog-fx')).toBeVisible();
	await expect(page.getByTestId('account-create-dialog-fx-default')).toBeChecked();
	await expect(page.getByTestId('account-create-dialog-fx-company')).toContainText(FX_NUMBER);

	const created = page.waitForResponse(
		(r) => r.url().endsWith('/api/v1/accounts') && r.request().method() === 'POST',
	);
	await page.getByTestId('account-create-dialog-submit').click();
	const res = await created;
	expect(res.status(), await res.text()).toBe(201);

	await expect(page.getByTestId(`account-row-${USD_NUMBER}-currency`)).toHaveText('USD');
	// Un compte CHF n'a pas de badge.
	await expect(page.getByTestId('account-row-1000-currency')).toHaveCount(0);

	const ctx = await authedApiContext(page);
	try {
		const account = await accountByNumber(ctx, USD_NUMBER);
		expect(account.currency).toBe('USD');
		// `currencyLocked` est absent quand il est faux (contrat du JSON).
		expect(account.currencyLocked).toBeUndefined();
	} finally {
		await disposeContextSafe(ctx);
	}
});

test('verrou : un compte CHF mouvementé ne reçoit plus de devise', async ({ page }) => {
	await login(page);
	const ctx = await authedApiContext(page);
	try {
		const lockedId = await createAccount(ctx, LOCKED_NUMBER, 'Banque mouvementée', 'Asset');
		const counterpart = await accountByNumber(ctx, '2000');
		// La ligne se passe sur un compte ENCORE CHF (AC-4).
		const entry = await ctx.post('/api/v1/journal-entries', {
			data: {
				entryDate: todayInZurich(),
				journal: 'OD',
				description: 'E2E 37-1 — compte mouvementé',
				lines: [
					{ accountId: lockedId, debit: '10.00', credit: '0.00' },
					{ accountId: counterpart.id, debit: '0.00', credit: '10.00' },
				],
			},
		});
		expect(entry.ok(), `écriture : ${entry.status()} ${await entry.text()}`).toBeTruthy();

		const locked = await accountByNumber(ctx, LOCKED_NUMBER);
		// CHF : `currency` absent du JSON.
		expect(locked.currency).toBeUndefined();
		expect(locked.currencyLocked).toBe(true);

		// Le serveur refuse la devise.
		const refused = await ctx.put(`/api/v1/accounts/${lockedId}`, {
			data: {
				name: locked.name,
				accountType: locked.accountType,
				role: locked.role,
				postable: locked.postable,
				currency: 'USD',
				version: locked.version,
			},
		});
		expect(refused.status()).toBe(409);
		expect(((await refused.json()) as { error: { code: string } }).error.code).toBe(
			'ACCOUNT_CURRENCY_LOCKED',
		);
	} finally {
		await disposeContextSafe(ctx);
	}

	// À l'écran, le champ est désactivé d'avance, avec l'explication.
	await page.goto('/accounts');
	await page.getByTestId(`account-row-${LOCKED_NUMBER}-edit-button`).click();
	await expect(page.getByRole('dialog')).toBeVisible();
	await expect(page.getByTestId('account-edit-dialog-currency')).toBeDisabled();
	await expect(page.getByTestId('account-edit-dialog-currency-locked')).toContainText(
		'porte déjà des écritures',
	);
	await page.getByTestId('account-edit-dialog-cancel').click();
});
