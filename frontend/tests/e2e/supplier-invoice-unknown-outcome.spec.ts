/**
 * Story 34-8 (#938, T7) — le paiement d'une facture fournisseur dont la réponse est
 * coupée APRÈS l'écriture. Le serveur a payé, le navigateur ne le sait pas : la fiche
 * relit la facture, la montre payée, et le dit dans un encadré. Le bouton *Payer* n'est
 * plus offert, la facture ayant changé de statut.
 *
 * La coupure est réelle (patron `invoices-settle-unknown-outcome.spec.ts`) :
 * `route.fetch()` envoie la requête au serveur, puis `route.abort()` jette sa réponse —
 * un `abort()` seul n'enverrait rien, et rien ne serait écrit.
 *
 * Décor synthétique : le seed `with-company`, un fournisseur créé par l'API. Les
 * captures (thème clair) s'écrivent dans `KESH_E2E_CAPTURES_DIR`, sinon dans la sortie
 * du test.
 */
import path from 'node:path';
import { expect, test, type Page, type TestInfo } from '@playwright/test';
import {
	seedTestState,
	clearAuthStorage,
	authedApiContext,
	disposeContextSafe,
} from './helpers/test-state';
import { PNG_SANS_QR, deposit, importViaApi, token, uniqueCopy } from './helpers/inbox';

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

async function capture(page: Page, testInfo: TestInfo, name: string): Promise<void> {
	const dossier = process.env.KESH_E2E_CAPTURES_DIR;
	await page.screenshot({
		path: dossier ? path.join(dossier, `${name}.png`) : testInfo.outputPath(`${name}.png`),
		fullPage: true,
	});
}

type Account = {
	id: number;
	number: string;
	accountType: string;
	active: boolean;
	postable: boolean;
};

/** Une facture fournisseur validée de 100.00, datée du jour ; rend son id et le compte de caisse. */
async function validatedSupplierInvoice(page: Page): Promise<{ id: number; cashId: number }> {
	const ctx = await authedApiContext(page);
	try {
		const accounts = (await (await ctx.get('/api/v1/accounts')).json()) as Account[];
		const payable = accounts.find((a) => a.number === '2000');
		const expense = accounts.find((a) => a.accountType === 'Expense' && a.active && a.postable);
		const cash = accounts.find(
			(a) => a.accountType === 'Asset' && a.active && a.postable && a.number !== '2000',
		);
		expect(payable && expense && cash, 'comptes du décor').toBeTruthy();
		const s = await (await ctx.get('/api/v1/company/invoice-settings')).json();
		if (s.defaultPayableAccountId == null) {
			const put = await ctx.put('/api/v1/company/invoice-settings', {
				data: { ...s, defaultPayableAccountId: payable!.id },
			});
			expect(put.ok(), `compte créanciers : ${put.status()}`).toBeTruthy();
		}
		const sup = await ctx.post('/api/v1/contacts', {
			data: {
				contactType: 'Entreprise',
				name: `Fournisseur issue inconnue ${Date.now()}`,
				isClient: false,
				isSupplier: true,
			},
		});
		expect(sup.ok(), `fournisseur : ${sup.status()}`).toBeTruthy();
		const draft = await ctx.post('/api/v1/supplier-invoices', {
			data: {
				contactId: (await sup.json()).id,
				supplierInvoiceNumber: `E348-${Date.now()}`,
				invoiceDate: new Date().toISOString().slice(0, 10),
				lines: [
					{
						description: 'Fourniture',
						quantity: '1',
						unitPrice: '100.00',
						vatRate: '0',
						expenseAccountId: expense!.id,
					},
				],
			},
		});
		expect(draft.ok(), `brouillon : ${draft.status()} ${await draft.text()}`).toBeTruthy();
		const d = await draft.json();
		const validated = await ctx.post(`/api/v1/supplier-invoices/${d.id}/validate`, {
			data: { version: d.version },
		});
		expect(validated.ok(), `validation : ${validated.status()}`).toBeTruthy();
		return { id: d.id as number, cashId: cash!.id };
	} finally {
		await disposeContextSafe(ctx);
	}
}

test('fiche fournisseur : paiement coupé après l’écriture, la fiche relue le dit', async ({
	page,
}, testInfo) => {
	await login(page);
	const { id, cashId } = await validatedSupplierInvoice(page);
	await page.route(`**/api/v1/supplier-invoices/${id}/pay`, async (route) => {
		if (route.request().method() !== 'POST') return route.fallback();
		const response = await route.fetch();
		expect(response.status(), 'le serveur a bien payé').toBe(200);
		await route.abort('connectionreset');
	});
	await page.goto(`/supplier-invoices/${id}`);
	await page.getByTestId('supplier-invoice-pay').getByRole('radio').nth(1).check();
	await page.getByTestId('pay-internal-account').selectOption(String(cashId));
	await page.getByTestId('supplier-invoice-pay-submit').click();

	const notice = page.getByTestId('supplier-unknown-outcome');
	await expect(notice).toBeVisible();
	await expect(notice).toHaveText(
		'L’issue n’est pas connue : la facture a été relue, vérifiez-la avant de réessayer.',
	);
	// Le paiement était écrit : la relecture montre la facture payée, et *Payer* n'est
	// plus offert.
	await expect(page.getByTestId('supplier-invoice-status')).toHaveText('Payée');
	await expect(page.getByTestId('supplier-invoice-pay-submit')).toHaveCount(0);
	await expect(page.getByTestId('supplier-invoice-pay-error')).toHaveCount(0);
	await capture(page, testInfo, 'e348-paiement-issue-inconnue');
});

/**
 * Revue P1 de la 34-8 (A-1) — le second avis : la réponse du paiement est coupée, puis
 * la relecture de la facture aussi. L'écran ne dit pas « la facture a été relue ».
 */
test('fiche fournisseur : paiement coupé et relecture en échec, la fiche dit de recharger', async ({
	page,
}, testInfo) => {
	await login(page);
	const { id, cashId } = await validatedSupplierInvoice(page);
	let paid = false;
	await page.route(`**/api/v1/supplier-invoices/${id}/pay`, async (route) => {
		if (route.request().method() !== 'POST') return route.fallback();
		const response = await route.fetch();
		expect(response.status(), 'le serveur a bien payé').toBe(200);
		paid = true;
		await route.abort('connectionreset');
	});
	// La relecture qui suit le paiement échoue elle aussi ; le chargement de la fiche,
	// avant, passe. Un `500` et non une coupure : le client rejoue un `GET` coupé
	// (`NETWORK_ERROR`, retry exponentiel de ~14 s, bandeau « réessai automatique »),
	// pas un `500`.
	await page.route(`**/api/v1/supplier-invoices/${id}`, async (route) => {
		if (route.request().method() !== 'GET' || !paid) return route.fallback();
		await route.fulfill({
			status: 500,
			contentType: 'application/json',
			body: JSON.stringify({ error: { code: 'INTERNAL_ERROR', message: 'Relecture refusée' } }),
		});
	});
	await page.goto(`/supplier-invoices/${id}`);
	await page.getByTestId('supplier-invoice-pay').getByRole('radio').nth(1).check();
	await page.getByTestId('pay-internal-account').selectOption(String(cashId));
	await page.getByTestId('supplier-invoice-pay-submit').click();

	const notice = page.getByTestId('supplier-unknown-outcome');
	await expect(notice).toHaveText(
		'L’issue n’est pas connue, et la facture n’a pas pu être relue : rechargez la page avant de réessayer.',
	);
	await expect(notice).toHaveAttribute('data-outcome', 'reread-failed');
	await capture(page, testInfo, 'e348-relecture-en-echec');
});

/**
 * Revue P1 de la 34-8 (A-1) — le sélecteur de document qui porte l'issue inconnue : la
 * demande de rattachement est coupée **avant** d'arriver au serveur, si bien que la
 * facture relue n'a pas de document. L'erreur reste dans le sélecteur, ouvert, et
 * l'encadré ne s'affiche pas (C-e348-3).
 */
test('fiche fournisseur : rattachement coupé, le sélecteur dit que l’issue n’est pas connue', async ({
	page,
}, testInfo) => {
	await login(page);
	const { id } = await validatedSupplierInvoice(page);
	const name = `e2e-e348-${token()}.png`;
	await deposit(name, uniqueCopy(PNG_SANS_QR, 'png'));
	const report = await importViaApi(page);
	expect(report.accepted.map((a) => a.fileName)).toContain(name);
	await page.route(`**/api/v1/supplier-invoices/${id}/source-document`, async (route) => {
		if (route.request().method() !== 'POST') return route.fallback();
		await route.abort('connectionreset');
	});
	await page.goto(`/supplier-invoices/${id}`);
	await page.getByTestId('supplier-invoice-attach').click();
	const row = page.getByTestId('attach-document-row').filter({ hasText: name });
	await expect(row).toHaveCount(1);
	page.once('dialog', (d) => void d.accept());
	await row.getByTestId('attach-document-submit').click();

	await expect(page.getByTestId('attach-document-error')).toHaveText(
		'L’issue n’est pas connue : la facture a été relue, vérifiez-la avant de réessayer.',
	);
	await expect(page.getByTestId('attach-document-picker')).toBeVisible();
	await expect(page.getByTestId('supplier-unknown-outcome')).toHaveCount(0);
	await capture(page, testInfo, 'e348-selecteur-issue-inconnue');
});

/**
 * Revue P3 de la 34-8 (A-5, B-3 = E-2) — le second clic après une relecture en échec : le
 * serveur refuse le second paiement (la facture est payée), et la relecture qui suit ce refus
 * échoue aussi. L'encadré dit que la facture n'a pas pu être relue, sans « l'issue n'est pas
 * connue », le refus, lui, étant connu.
 */
test('fiche fournisseur : second paiement refusé et relecture en échec, l’encadré dit de recharger', async ({
	page,
}, testInfo) => {
	await login(page);
	const { id, cashId } = await validatedSupplierInvoice(page);
	let paid = false;
	await page.route(`**/api/v1/supplier-invoices/${id}/pay`, async (route) => {
		if (route.request().method() !== 'POST') return route.fallback();
		if (paid) return route.fallback(); // le second envoi va au serveur, qui refuse
		const response = await route.fetch();
		expect(response.status(), 'le serveur a bien payé').toBe(200);
		paid = true;
		await route.abort('connectionreset');
	});
	await page.route(`**/api/v1/supplier-invoices/${id}`, async (route) => {
		if (route.request().method() !== 'GET' || !paid) return route.fallback();
		await route.fulfill({
			status: 500,
			contentType: 'application/json',
			body: JSON.stringify({ error: { code: 'INTERNAL_ERROR', message: 'Relecture refusée' } }),
		});
	});
	await page.goto(`/supplier-invoices/${id}`);
	await page.getByTestId('supplier-invoice-pay').getByRole('radio').nth(1).check();
	await page.getByTestId('pay-internal-account').selectOption(String(cashId));
	await page.getByTestId('supplier-invoice-pay-submit').click();
	await expect(page.getByTestId('supplier-unknown-outcome')).toHaveAttribute(
		'data-outcome',
		'reread-failed',
	);
	await page.getByTestId('supplier-invoice-pay-submit').click();

	const notice = page.getByTestId('supplier-unknown-outcome');
	await expect(notice).toHaveAttribute('data-outcome', 'refused-reread-failed');
	await expect(notice).toHaveText(
		'La facture n’a pas pu être relue : rechargez la page avant de réessayer.',
	);
	await expect(page.getByTestId('supplier-invoice-pay-error')).toBeVisible();
	await capture(page, testInfo, 'e348-refus-relecture-en-echec');
});

/**
 * Revue P3 de la 34-8 (A-5 ; revue P2, B-4) — le second clic refusé « déjà traité » sur SON
 * document : le rattachement est écrit, la réponse perdue, et la relecture qui suit ne montre
 * pas encore le document (servie ici depuis la fiche d'avant). Au second clic, le serveur
 * refuse ; la fiche relit, trouve ce document, et le dit joint, non en avertissement.
 */
test('fiche fournisseur : second rattachement refusé sur son propre document, la fiche le dit joint', async ({
	page,
}, testInfo) => {
	await login(page);
	const { id } = await validatedSupplierInvoice(page);
	const name = `e2e-e348-own-${token()}.png`;
	await deposit(name, uniqueCopy(PNG_SANS_QR, 'png'));
	const report = await importViaApi(page);
	expect(report.accepted.map((a) => a.fileName)).toContain(name);
	let before: string | null = null;
	let attached = false;
	let rereadServed = false;
	await page.route(`**/api/v1/supplier-invoices/${id}`, async (route) => {
		if (route.request().method() !== 'GET') return route.fallback();
		if (before === null) {
			const response = await route.fetch();
			before = await response.text();
			return route.fulfill({ response, body: before });
		}
		if (attached && !rereadServed) {
			rereadServed = true; // la relecture d'après la réponse perdue : la fiche d'avant
			return route.fulfill({ status: 200, contentType: 'application/json', body: before });
		}
		return route.fallback();
	});
	await page.route(`**/api/v1/supplier-invoices/${id}/source-document`, async (route) => {
		if (route.request().method() !== 'POST' || attached) return route.fallback();
		const response = await route.fetch();
		expect(response.status(), 'le serveur a bien joint').toBe(200);
		attached = true;
		await route.abort('connectionreset');
	});
	await page.goto(`/supplier-invoices/${id}`);
	await page.getByTestId('supplier-invoice-attach').click();
	const row = page.getByTestId('attach-document-row').filter({ hasText: name });
	await expect(row).toHaveCount(1);
	page.once('dialog', (d) => void d.accept());
	await row.getByTestId('attach-document-submit').click();
	await expect(page.getByTestId('attach-document-error')).toBeVisible();

	page.once('dialog', (d) => void d.accept());
	await row.getByTestId('attach-document-submit').click();
	// Une notification de succès, non d'avertissement (sélecteur sans libellé traduit, KF-043).
	await expect(page.locator('[data-sonner-toast][data-type="success"]')).toBeVisible();
	await expect(page.locator('[data-sonner-toast][data-type="warning"]')).toHaveCount(0);
	await expect(page.getByTestId('supplier-invoice-document-name')).toHaveText(name);
	await capture(page, testInfo, 'e348-propre-document-joint');
});
