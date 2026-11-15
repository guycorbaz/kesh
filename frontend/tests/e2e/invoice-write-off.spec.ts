/**
 * Story 25-4-d2b (#490 ; API : 25-4-d2a, #384) — solder le reste d'une facture
 * depuis sa fiche.
 *
 * Montage : un compte d'escompte désigné par le formulaire des paramètres (le
 * 4000 *Charges* du seed), une facture validée de 972.90 TTC à 8.1 % (multiple
 * de 0.05 : aucun compte d'arrondi requis), réglée de 500.— par l'API. Puis, à
 * l'écran : solder le reste en escompte, vérifier la liste et le motif qui
 * bloque l'annulation du règlement antérieur, annuler le solde.
 *
 * Story 36-9 (#1044) : un second scénario retype le compte de TVA due par l'API
 * et vérifie le refus du solde dans le dialogue ; le compte est rendu à son type.
 *
 * Nettoyage : le compte d'escompte remis à vide — la base est partagée entre
 * specs (patron `invoice-settings-write-off-accounts.spec.ts`).
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
	createAndValidateInvoiceViaApi,
	invoiceVersionViaApi,
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

async function setDiscountAccount(page: Page, number: string | null): Promise<void> {
	await page.goto('/settings/invoicing');
	const select = page.getByTestId('settings-discount-account');
	// Les comptes arrivent après le rendu : attendre une option de compte.
	await expect(select.locator('option').nth(1)).toBeAttached();
	if (number === null) {
		await select.selectOption({ index: 0 });
	} else {
		const value = await select
			.locator('option', { hasText: number })
			.first()
			.getAttribute('value');
		expect(value, `compte ${number} proposé`).toBeTruthy();
		await select.selectOption(value!);
	}
	const saved = page.waitForResponse(
		(r) => r.url().includes('/api/v1/company/invoice-settings') && r.request().method() === 'PUT',
	);
	await page.getByTestId('settings-invoicing-save').click();
	expect((await saved).status()).toBe(200);
}

/** Un compte tel que le rend `GET /api/v1/accounts`. */
type Account = {
	id: number;
	number: string;
	name: string;
	accountType: string;
	role: string | null;
	postable: boolean;
	version: number;
};

/**
 * Change le type d'un compte par `PUT /api/v1/accounts/{id}` (le chemin réel),
 * rôle et imputabilité gardés, confirmation du retypage donnée (il porte des
 * écritures). Rend le compte tel qu'il était avant.
 */
async function retypeAccount(
	ctx: Awaited<ReturnType<typeof authedApiContext>>,
	accountId: number,
	accountType: string,
): Promise<Account> {
	const accounts = (await (await ctx.get('/api/v1/accounts')).json()) as Account[];
	const before = accounts.find((a) => a.id === accountId);
	expect(before, `compte ${accountId} de la société`).toBeTruthy();
	const res = await ctx.put(`/api/v1/accounts/${accountId}`, {
		data: {
			name: before!.name,
			accountType,
			role: before!.role,
			postable: before!.postable,
			version: before!.version,
			confirmAccountRetype: true,
		},
	});
	expect(res.ok(), `retypage du compte ${before!.number} : ${res.status()}`).toBeTruthy();
	return before!;
}

/**
 * Story 36-9 (#1044, AC9) — le compte de TVA due des paramètres, retypé en
 * actif depuis sa désignation, fait refuser le solde du reste en escompte : le
 * dialogue reste ouvert et montre le message du serveur, qui nomme le compte et
 * *Administration → Facturation* ; la facture reste validée, son reste dû
 * inchangé. Le compte est rendu à son type dans un `finally` (base partagée).
 */
test('un compte de TVA due changé de type fait refuser le solde en escompte', async ({ page }) => {
	await login(page);
	let vatAccountId: number | null = null;
	let originalType: string | null = null;
	try {
		await setDiscountAccount(page, '4000');
		const contactId = await createContactWithAddressViaApi(page, 'Client TVA retypée');
		const invoiceId = await createAndValidateInvoiceViaApi(page, contactId);

		const ctx = await authedApiContext(page);
		let vatNumber = '';
		try {
			const settings = await (await ctx.get('/api/v1/company/invoice-settings')).json();
			vatAccountId = settings.defaultVatPayableAccountId as number;
			expect(vatAccountId, 'compte de TVA due désigné au seed').toBeTruthy();
			const before = await retypeAccount(ctx, vatAccountId, 'Asset');
			originalType = before.accountType;
			vatNumber = before.number;
			expect(originalType, 'la TVA due du seed est un passif').toBe('Liability');
		} finally {
			await disposeContextSafe(ctx);
		}

		await page.goto(`/invoices/${invoiceId}`);
		await expect(page.getByTestId('invoice-amount-due')).toContainText('972.90');
		await page.getByTestId('write-off-open').click();
		await page.getByTestId('write-off-nature-discount').check();
		const refused = page.waitForResponse((r) =>
			r.url().includes(`/api/v1/invoices/${invoiceId}/write-off`),
		);
		await page.getByTestId('write-off-confirm').click();
		expect((await refused).status()).toBe(400);

		// Le dialogue reste ouvert, avec le message du serveur.
		const error = page.getByTestId('write-off-error');
		await expect(error).toBeVisible();
		await expect(error).toContainText(`Le compte ${vatNumber}`);
		await expect(error).toContainText('Administration → Facturation');
		await expect(page.getByTestId('write-off-confirm')).toBeVisible();
		// Capture (thème clair) si `KESH_E2E_CAPTURES_DIR` est posé ; sinon, rien.
		const captures = process.env.KESH_E2E_CAPTURES_DIR;
		if (captures) {
			await page.screenshot({ path: `${captures}/write-off-tva-due-retypee.png` });
		}

		// Rien n'a été écrit : la facture reste validée, son reste dû inchangé.
		await expect(page.getByTestId('invoice-amount-due')).toContainText('972.90');
		const check = await authedApiContext(page);
		try {
			const invoice = await (await check.get(`/api/v1/invoices/${invoiceId}`)).json();
			expect(invoice.status).toBe('validated');
			expect(invoice.paidAt).toBeNull();
		} finally {
			await disposeContextSafe(check);
		}
	} finally {
		if (vatAccountId !== null && originalType !== null) {
			const ctx = await authedApiContext(page);
			try {
				await retypeAccount(ctx, vatAccountId, originalType);
			} finally {
				await disposeContextSafe(ctx);
			}
		}
		await setDiscountAccount(page, null);
	}
});

test('solder le reste en escompte depuis la fiche, puis annuler le solde', async ({ page }) => {
	await login(page);
	try {
		await setDiscountAccount(page, '4000');

		const contactId = await createContactWithAddressViaApi(page, 'Client Escompte');
		const invoiceId = await createAndValidateInvoiceViaApi(page, contactId);
		const ctx = await authedApiContext(page);
		try {
			const accounts = (await (await ctx.get('/api/v1/accounts')).json()) as {
				id: number;
				number: string;
			}[];
			const caisse = accounts.find((a) => a.number === '1000');
			expect(caisse, 'compte 1000 du seed').toBeTruthy();
			const settled = await ctx.post(`/api/v1/invoices/${invoiceId}/settlements`, {
				data: {
					settlementType: 'internal_account',
					accountId: caisse!.id,
					amount: '500.00',
					settledOn: todayInZurich(),
					version: await invoiceVersionViaApi(ctx, invoiceId),
				},
			});
			expect(settled.ok(), `règlement : ${settled.status()}`).toBeTruthy();
		} finally {
			await disposeContextSafe(ctx);
		}

		await page.goto(`/invoices/${invoiceId}`);
		await expect(page.getByTestId('invoice-amount-due')).toContainText('472.90');

		// Solder le reste en escompte.
		await page.getByTestId('write-off-open').click();
		await expect(page.getByTestId('write-off-amount')).toContainText('472.90');
		await page.getByTestId('write-off-nature-discount').check();
		await page.getByTestId('write-off-confirm').click();

		await expect(page.getByTestId('invoice-amount-due')).toContainText('0.00');
		await expect(page.getByTestId('invoice-amount-settled')).toContainText('500.00');
		await expect(page.getByTestId('invoice-amount-written-off')).toContainText('472.90');
		const modes = page.getByTestId('invoice-settlement-mode');
		await expect(modes).toHaveCount(2);
		await expect(modes.nth(1)).toContainText('Escompte accordé');
		// Le règlement antérieur ne s'annule pas tant que le solde existe.
		await expect(page.getByTestId('invoice-settlement-cancel-blocked')).toContainText(
			"annulez d'abord le solde",
		);

		// Story 25-4-d2c — AVANT l'annulation (un solde annulé quitte le rapport) :
		// le rapport TVA de la période retranche la TVA du solde. Seul test qui voit
		// les champs des diminutions traverser la frontière HTTP.
		await page.goto('/reports');
		await page.waitForLoadState('networkidle');
		await page.getByRole('tab', { name: /^TVA$/ }).click();
		await page.getByRole('button', { name: /générer/i }).click();
		await expect(page.getByTestId('vat-write-off-section')).toBeVisible();
		await expect(page.getByTestId('vat-total-vat-due-net')).toBeVisible();
		await page.goto(`/invoices/${invoiceId}`);

		// Annuler le solde : le reste réapparaît.
		const cancel = page.getByTestId('invoice-settlement-cancel');
		await expect(cancel).toHaveCount(1);
		await expect(cancel).toContainText('Annuler le solde');
		await cancel.click();
		await page.getByTestId('invoice-settlement-cancel-confirm').click();
		await expect(page.getByTestId('invoice-amount-due')).toContainText('472.90');
		await expect(page.getByTestId('invoice-settlement-mode')).toHaveCount(1);
	} finally {
		await setDiscountAccount(page, null);
	}
});
