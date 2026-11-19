/**
 * Story 36-18a (#1050, #1159, #1172, #1174 ; AC7, AC11) — les montants « 1.250 » des écrans
 * qui écrivent dans les livres : soldes de départ, prix unitaire et montant attendu d'une
 * facture fournisseur, règlement d'une facture.
 *
 * Avant la story, « 1.250 » partait tel quel et le serveur le lisait 1,25 franc. Seul un E2E
 * voit le montant traverser la frontière HTTP : Vitest teste le corps construit, les tests
 * Rust la lecture du serveur, et aucun des deux ne voit ce qui part réellement.
 *
 * Pour chaque écran : « 1.250 » → message visible, **aucune requête d'écriture** (le compteur
 * `page.on('request')`, filtré par méthode et route, n'est qu'un indice ; la preuve est la
 * relecture par l'API de l'état d'avant) ; puis « 1'250 » → le corps porte `1250`, réponse
 * 2xx, relecture comparée **en valeur** (le serveur rend `1250.0000`).
 *
 * Décor synthétique, créé par l'API. Les captures (thème clair) s'écrivent dans
 * `KESH_E2E_CAPTURES_DIR`, sinon dans la sortie du test (hors dépôt).
 *
 * Story 48-10 (#1173, #1175, #1176, #1177 ; AC6, AC11), qui reprend la 36-18b : les écrans de
 * réglage et de filtre — prix d'un article, frais d'un niveau de rappel, montant minimal d'une
 * facture, bornes de montant du filtre des écritures (une requête de lecture, non d'écriture :
 * la borne ambiguë est omise de la requête et de l'URL). Captures préfixées `e4810-`.
 */
import path from 'node:path';
import Big from 'big.js';
import {
	expect,
	test,
	type APIRequestContext,
	type Page,
	type Request,
	type TestInfo,
} from '@playwright/test';
import {
	seedTestState,
	clearAuthStorage,
	authedApiContext,
	disposeContextSafe,
} from './helpers/test-state';
import { invoiceVersionViaApi, todayInZurich } from './helpers/api-fixtures';

/** Le dossier des captures : `KESH_E2E_CAPTURES_DIR`, sinon la sortie du test (hors dépôt). */
async function capture(
	page: Page,
	testInfo: TestInfo,
	name: string,
	prefixe: 'e3618a' | 'e4810' = 'e3618a',
): Promise<void> {
	const dossier = process.env.KESH_E2E_CAPTURES_DIR;
	const fichier = `${prefixe}-${name}.png`;
	await page.screenshot({
		path: dossier ? path.join(dossier, fichier) : testInfo.outputPath(fichier),
		fullPage: true,
	});
}

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

/** Une route d'écriture : méthode et chemin (sans requête). */
type WriteRoute = { method: string; path: RegExp };

/**
 * Relève les requêtes d'écriture de l'écran, armé AVANT le clic (AC7, V1-7) : rend leurs
 * corps, pour qu'un échec montre ce qui serait parti (« 1.250 », avant la story).
 */
function countWrites(page: Page, routes: WriteRoute[]): () => string[] {
	const bodies: string[] = [];
	const matches = (req: Request) => {
		const p = new URL(req.url()).pathname;
		return routes.some((r) => r.method === req.method() && r.path.test(p));
	};
	page.on('request', (req) => {
		if (matches(req)) bodies.push(`${req.method()} ${new URL(req.url()).pathname} ${req.postData()}`);
	});
	return () => bodies;
}

/** Attend la requête d'écriture de l'écran (même filtre). */
function waitForWrite(page: Page, routes: WriteRoute[]): Promise<Request> {
	return page.waitForRequest((req) => {
		const p = new URL(req.url()).pathname;
		return routes.some((r) => r.method === req.method() && r.path.test(p));
	});
}

type Account = {
	id: number;
	number: string;
	accountType: string;
	role: string | null;
	active: boolean;
	postable: boolean;
};

async function accounts(ctx: APIRequestContext): Promise<Account[]> {
	const res = await ctx.get('/api/v1/accounts');
	expect(res.ok(), `comptes : ${res.status()}`).toBeTruthy();
	return (await res.json()) as Account[];
}

function suffix(): string {
	return `${Date.now()}-${Math.floor(Math.random() * 1e6)}`;
}

// ---------------------------------------------------------------------------
// #1050 — soldes de départ
// ---------------------------------------------------------------------------

const OPENING_WRITES: WriteRoute[] = [
	{ method: 'POST', path: /^\/api\/v1\/opening-balances$/ },
	{ method: 'POST', path: /^\/api\/v1\/opening-balances\/complete$/ },
];

test('soldes de départ : « 1.250 » refusé comme ambigu, rien n’est écrit ; « 1’250 » part 1250', async ({
	page,
}, testInfo) => {
	// Société vierge, comme `opening-balances.spec.ts` (preset `with-company`).
	await seedTestState('with-company');
	await login(page);
	const ctx = await authedApiContext(page);
	try {
		const all = await accounts(ctx);
		const asset = all.find((a) => a.accountType === 'Asset' && a.postable && a.active);
		const equity =
			all.find((a) => a.role === 'RetainedEarnings' && a.postable && a.active) ??
			all.find((a) => a.accountType === 'Liability' && a.postable && a.active);
		expect(asset && equity, 'un actif et une contrepartie de bilan').toBeTruthy();

		await page.goto('/settings/opening-balances');
		await expect(page.getByTestId('opening-balances-grid')).toBeVisible();
		const writes = countWrites(page, OPENING_WRITES);

		// « 1.250 » à l'actif, « 1.25 » au report : équilibré sous l'ancienne lecture.
		await page.getByTestId(`opening-balances-debit-${asset!.number}`).fill('1.250');
		await page.getByTestId(`opening-balances-credit-${equity!.number}`).fill('1.25');
		// `expect.soft` : sans le correctif, le test va jusqu'au clic et montre la requête partie.
		await expect.soft(page.getByTestId('opening-balances-amount-ambiguous')).toBeVisible();
		await expect.soft(page.getByTestId('opening-balances-generate')).toBeDisabled();
		await capture(page, testInfo, 'soldes-ambigu');
		await page.getByTestId('opening-balances-generate').click({ force: true });
		await page.waitForTimeout(300);
		expect(writes(), 'aucune requête d’écriture des soldes').toEqual([]);
		const status = await (await ctx.get('/api/v1/opening-balances/status')).json();
		expect(status.reason, 'aucune écriture d’ouverture').toBe('READY');

		// « 12'34 » : illisible.
		await page.getByTestId(`opening-balances-debit-${asset!.number}`).fill("12'34");
		await expect(page.getByTestId('opening-balances-amount-unreadable')).toBeVisible();
		await capture(page, testInfo, 'soldes-illisible');

		// « 1'250 » des deux côtés : la requête porte 1250.
		await page.getByTestId(`opening-balances-debit-${asset!.number}`).fill("1'250");
		await page.getByTestId(`opening-balances-credit-${equity!.number}`).fill('1’250');
		await expect(page.getByTestId('opening-balances-amount-ambiguous')).toHaveCount(0);
		const sent = waitForWrite(page, OPENING_WRITES);
		await page.getByTestId('opening-balances-generate').click();
		const req = await sent;
		const lines = (req.postDataJSON() as { lines: { accountId: number; debit: string; credit: string }[] })
			.lines;
		expect(lines.find((l) => l.accountId === asset!.id)?.debit).toBe('1250');
		expect(lines.find((l) => l.accountId === equity!.id)?.credit).toBe('1250');
		const res = await req.response();
		expect(res?.ok(), `génération : ${res?.status()}`).toBeTruthy();
		await expect(page.getByTestId('opening-balances-locked')).toBeVisible();
		await capture(page, testInfo, 'soldes-1250-enregistre');

		// Relecture : l'écriture d'ouverture porte 1250, en valeur.
		const entryId = ((await res!.json()) as { id: number }).id;
		const entry = await (await ctx.get(`/api/v1/journal-entries/${entryId}`)).json();
		const debit = (entry.lines as { accountId: number; debit: string }[]).find(
			(l) => l.accountId === asset!.id,
		);
		expect(new Big(debit!.debit).eq(1250), `débit relu : ${debit?.debit}`).toBe(true);
		// Revue P1 (A4) : la capture de l'écriture générée montre la valeur enregistrée.
		await page.goto(`/journal-entries/${entryId}`);
		await expect(page.getByText(/1.250\.00/).first()).toBeVisible();
		await capture(page, testInfo, 'soldes-1250-ecriture');
	} finally {
		await disposeContextSafe(ctx);
	}
});

// ---------------------------------------------------------------------------
// #1159, #1172 — facture fournisseur (sans document, `/supplier-invoices/new`)
// ---------------------------------------------------------------------------

const SUPPLIER_WRITES: WriteRoute[] = [
	{ method: 'POST', path: /^\/api\/v1\/supplier-invoices$/ },
	{ method: 'PUT', path: /^\/api\/v1\/supplier-invoices\/\d+$/ },
	{ method: 'POST', path: /^\/api\/v1\/imported-supplier-invoices\/\d+\/complete$/ },
];

async function supplierDecor(ctx: APIRequestContext): Promise<{ supplierId: number; expense: number }> {
	const expense = (await accounts(ctx)).find(
		(a) => a.accountType === 'Expense' && a.active && a.postable,
	);
	expect(expense, 'un compte de charge imputable').toBeTruthy();
	const sup = await ctx.post('/api/v1/contacts', {
		data: {
			contactType: 'Entreprise',
			name: `Fournisseur montants ${suffix()}`,
			isClient: false,
			isSupplier: true,
		},
	});
	expect(sup.ok(), `fournisseur : ${sup.status()}`).toBeTruthy();
	return { supplierId: (await sup.json()).id as number, expense: expense!.id };
}

async function supplierInvoiceTotal(ctx: APIRequestContext): Promise<number> {
	const r = await ctx.get('/api/v1/supplier-invoices?limit=1');
	expect(r.ok(), `liste : ${r.status()}`).toBeTruthy();
	return ((await r.json()) as { total: number }).total;
}

async function fillSupplierForm(
	page: Page,
	d: { supplierId: number; expense: number },
	price: string,
): Promise<void> {
	await page.getByTestId('supplier-invoice-supplier').selectOption(String(d.supplierId));
	await page.getByTestId('supplier-invoice-date').fill(todayInZurich());
	const line = page.getByTestId('supplier-invoice-line').first();
	await line.getByTestId('supplier-invoice-line-desc').fill('Fourniture');
	await line.getByTestId('supplier-invoice-line-unit-price').fill(price);
	await line.getByTestId('supplier-invoice-line-account').selectOption(String(d.expense));
}

test('facture fournisseur, prix unitaire : « 1.250 » refusé, rien n’est écrit ; « 1’250 » part 1250', async ({
	page,
}, testInfo) => {
	await seedTestState('with-data');
	await login(page);
	const ctx = await authedApiContext(page);
	try {
		const d = await supplierDecor(ctx);
		const before = await supplierInvoiceTotal(ctx);
		await page.goto('/supplier-invoices/new');
		const writes = countWrites(page, SUPPLIER_WRITES);

		await fillSupplierForm(page, d, '1.250');
		await expect.soft(page.getByTestId('supplier-invoice-line-unit-price-ambiguous')).toBeVisible();
		await page.getByTestId('supplier-invoice-submit').click();
		await page.waitForTimeout(300);
		expect(writes(), 'aucune requête d’écriture de facture fournisseur').toEqual([]);
		await expect(page.getByTestId('supplier-invoice-form-error')).toContainText(
			'Ligne 1 : prix unitaire ambigu',
		);
		await capture(page, testInfo, 'fournisseur-prix-ambigu');
		expect(await supplierInvoiceTotal(ctx), 'aucune facture fournisseur neuve').toBe(before);

		const line = page.getByTestId('supplier-invoice-line').first();
		await line.getByTestId('supplier-invoice-line-unit-price').fill("12'34");
		await expect(page.getByTestId('supplier-invoice-line-unit-price-unreadable')).toBeVisible();
		await capture(page, testInfo, 'fournisseur-prix-illisible');

		await line.getByTestId('supplier-invoice-line-unit-price').fill("1'250");
		const sent = waitForWrite(page, SUPPLIER_WRITES);
		await page.getByTestId('supplier-invoice-submit').click();
		const req = await sent;
		expect((req.postDataJSON() as { lines: { unitPrice: string }[] }).lines[0].unitPrice).toBe('1250');
		const res = await req.response();
		expect(res?.ok(), `création : ${res?.status()}`).toBeTruthy();
		await expect(page).toHaveURL(/\/supplier-invoices\/\d+$/);
		await capture(page, testInfo, 'fournisseur-prix-1250-enregistre');
		const id = ((await res!.json()) as { id: number }).id;
		const inv = await (await ctx.get(`/api/v1/supplier-invoices/${id}`)).json();
		expect(new Big(inv.lines[0].unitPrice).eq(1250), `prix relu : ${inv.lines[0].unitPrice}`).toBe(
			true,
		);

		// AC11 — un brouillon au prix 1.2340 (10 × 1.234 = 12.34) se rouvre 1.2340, sans message.
		const draft = await ctx.post('/api/v1/supplier-invoices', {
			data: {
				contactId: d.supplierId,
				supplierInvoiceNumber: null,
				invoiceDate: todayInZurich(),
				dueDate: null,
				lines: [
					{
						description: 'Fourniture au millième',
						quantity: '10',
						unitPrice: '1.2340',
						vatRate: '0',
						expenseAccountId: d.expense,
					},
				],
			},
		});
		expect(draft.ok(), `brouillon : ${draft.status()} ${await draft.text()}`).toBeTruthy();
		await page.goto(`/supplier-invoices/${(await draft.json()).id}/edit`);
		await expect(page.getByTestId('supplier-invoice-line-unit-price')).toHaveValue('1.2340');
		await expect(page.getByTestId('supplier-invoice-line-unit-price-ambiguous')).toHaveCount(0);
		await capture(page, testInfo, 'fournisseur-brouillon-rouvert-1.2340');
	} finally {
		await disposeContextSafe(ctx);
	}
});

test('facture fournisseur, montant attendu : « 1.250 » refusé, rien n’est écrit ; « 1’250 » part 1250', async ({
	page,
}, testInfo) => {
	await seedTestState('with-data');
	await login(page);
	const ctx = await authedApiContext(page);
	try {
		const d = await supplierDecor(ctx);
		const before = await supplierInvoiceTotal(ctx);
		await page.goto('/supplier-invoices/new');
		const writes = countWrites(page, SUPPLIER_WRITES);

		await fillSupplierForm(page, d, '1250.00');
		await page.getByTestId('supplier-invoice-expected-amount').fill('1.250');
		await expect.soft(page.getByTestId('supplier-invoice-expected-amount-ambiguous')).toBeVisible();
		await page.getByTestId('supplier-invoice-submit').click();
		await page.waitForTimeout(300);
		expect(writes(), 'aucune requête d’écriture de facture fournisseur').toEqual([]);
		await expect(page.getByTestId('supplier-invoice-form-error')).toContainText('Montant ambigu');
		await capture(page, testInfo, 'fournisseur-attendu-ambigu');
		expect(await supplierInvoiceTotal(ctx), 'aucune facture fournisseur neuve').toBe(before);

		await page.getByTestId('supplier-invoice-expected-amount').fill("12'34");
		await expect(page.getByTestId('supplier-invoice-expected-amount-unreadable')).toBeVisible();
		await capture(page, testInfo, 'fournisseur-attendu-illisible');

		await page.getByTestId('supplier-invoice-expected-amount').fill("1'250");
		const sent = waitForWrite(page, SUPPLIER_WRITES);
		await page.getByTestId('supplier-invoice-submit').click();
		const req = await sent;
		expect((req.postDataJSON() as { expectedPaymentAmount: string | null }).expectedPaymentAmount).toBe(
			'1250',
		);
		const res = await req.response();
		expect(res?.ok(), `création : ${res?.status()}`).toBeTruthy();
		await expect(page).toHaveURL(/\/supplier-invoices\/\d+$/);
		await capture(page, testInfo, 'fournisseur-attendu-1250-enregistre');
		const id = ((await res!.json()) as { id: number }).id;
		const inv = await (await ctx.get(`/api/v1/supplier-invoices/${id}`)).json();
		expect(
			new Big(inv.expectedPaymentAmount).eq(1250),
			`montant attendu relu : ${inv.expectedPaymentAmount}`,
		).toBe(true);
	} finally {
		await disposeContextSafe(ctx);
	}
});

// ---------------------------------------------------------------------------
// #1174 — règlement d'une facture client
// ---------------------------------------------------------------------------

const SETTLE_WRITES: WriteRoute[] = [{ method: 'POST', path: /^\/api\/v1\/invoices\/\d+\/settlements$/ }];

/** Une facture validée de 2000.00 HT (2162.00 TTC), sans règlement ; rend son id. */
async function validatedInvoice(ctx: APIRequestContext): Promise<number> {
	const contact = await ctx.post('/api/v1/contacts', {
		data: {
			contactType: 'Entreprise',
			name: `Client montants ${suffix()}`,
			isClient: true,
			isSupplier: false,
		},
	});
	expect(contact.ok(), `contact : ${contact.status()}`).toBeTruthy();
	const created = await ctx.post('/api/v1/invoices', {
		data: {
			contactId: (await contact.json()).id,
			date: todayInZurich(),
			dueDate: todayInZurich(),
			paymentTerms: null,
			lines: [{ description: 'Prestation', quantity: '1', unitPrice: '2000.00', vatRate: '8.10' }],
		},
	});
	expect(created.ok(), `facture : ${created.status()}`).toBeTruthy();
	const id = (await created.json()).id as number;
	const validated = await ctx.post(`/api/v1/invoices/${id}/validate`);
	expect(validated.ok(), `validation : ${validated.status()}`).toBeTruthy();
	return id;
}

async function amountDue(ctx: APIRequestContext, id: number): Promise<string> {
	const res = await ctx.get(`/api/v1/invoices/${id}`);
	expect(res.ok()).toBeTruthy();
	return ((await res.json()) as { amountDue: string }).amountDue;
}

test('règlement : « 1.250 » refusé comme ambigu, rien n’est écrit ; « 1’250 » part 1250', async ({
	page,
}, testInfo) => {
	await seedTestState('with-data');
	await login(page);
	const ctx = await authedApiContext(page);
	try {
		const id = await validatedInvoice(ctx);
		const dueBefore = await amountDue(ctx, id);
		const versionBefore = await invoiceVersionViaApi(ctx, id);
		await page.goto(`/invoices/${id}`);
		await page.getByTestId('settle-open').click();
		const writes = countWrites(page, SETTLE_WRITES);
		await page.getByTestId('settle-type').selectOption('internal_account');
		await page.getByTestId('settle-account').selectOption({ index: 1 });

		await page.locator('#settle-amount').fill('1.250');
		await expect.soft(page.locator('#settle-client-error')).toContainText('Montant ambigu');
		await expect.soft(page.locator('#settle-amount')).toHaveAttribute('aria-invalid', 'true');
		await expect.soft(page.getByTestId('settle-confirm')).toBeDisabled();
		await capture(page, testInfo, 'reglement-ambigu');
		await page.getByTestId('settle-confirm').click({ force: true });
		await page.waitForTimeout(300);
		expect(writes(), 'aucune requête de règlement').toEqual([]);
		expect(new Big(await amountDue(ctx, id)).eq(new Big(dueBefore)), 'reste dû inchangé').toBe(true);
		expect(await invoiceVersionViaApi(ctx, id)).toBe(versionBefore);

		await page.locator('#settle-amount').fill("12'34");
		await expect(page.locator('#settle-client-error')).toContainText('2 décimales au plus');
		await capture(page, testInfo, 'reglement-illisible');

		await page.locator('#settle-amount').fill("1'250");
		await expect(page.locator('#settle-client-error')).toHaveCount(0);
		const sent = waitForWrite(page, SETTLE_WRITES);
		await page.getByTestId('settle-confirm').click();
		const req = await sent;
		expect((req.postDataJSON() as { amount: string }).amount).toBe('1250');
		const res = await req.response();
		expect(res?.ok(), `règlement : ${res?.status()}`).toBeTruthy();
		await expect(page.getByTestId('settle-confirm')).toHaveCount(0);
		await expect(page.getByTestId('invoice-amount-settled')).toContainText('250.00');
		await capture(page, testInfo, 'reglement-1250-enregistre');
		const due = new Big(await amountDue(ctx, id));
		expect(due.eq(new Big(dueBefore).minus(1250)), `reste dû relu : ${due.toFixed()}`).toBe(true);
	} finally {
		await disposeContextSafe(ctx);
	}
});

// ---------------------------------------------------------------------------
// Story 48-10 — #1173, prix d'un article
// ---------------------------------------------------------------------------

const PRODUCT_WRITES: WriteRoute[] = [
	{ method: 'POST', path: /^\/api\/v1\/products$/ },
	{ method: 'PUT', path: /^\/api\/v1\/products\/\d+$/ },
];

type Product = { id: number; name: string; unitPrice: string };

async function productsNamed(ctx: APIRequestContext, name: string): Promise<Product[]> {
	const r = await ctx.get(`/api/v1/products?search=${encodeURIComponent(name)}&limit=100`);
	expect(r.ok(), `articles : ${r.status()}`).toBeTruthy();
	return ((await r.json()) as { items: Product[] }).items.filter((p) => p.name === name);
}

test('article, prix : « 1.250 » refusé comme ambigu, rien n’est écrit ; « 1’250 » part 1250', async ({
	page,
}, testInfo) => {
	await seedTestState('with-company');
	await login(page);
	const ctx = await authedApiContext(page);
	try {
		const name = `Article montants ${suffix()}`;
		await page.goto('/products');
		await page.getByRole('button', { name: /Nouveau produit/ }).click();
		const writes = countWrites(page, PRODUCT_WRITES);
		await page.fill('#form-name', name);
		await page.fill('#form-price', '1.250');
		await expect.soft(page.getByTestId('product-form-price-error')).toContainText('Montant ambigu');
		await expect.soft(page.locator('#form-price')).toHaveAttribute('aria-invalid', 'true');
		await expect.soft(page.getByRole('button', { name: 'Créer' })).toBeDisabled();
		await capture(page, testInfo, 'article-prix-ambigu', 'e4810');
		await page.getByRole('button', { name: 'Créer' }).click({ force: true });
		await page.locator('#form-price').press('Enter');
		await page.waitForTimeout(300);
		expect(writes(), 'aucune requête d’écriture d’article').toEqual([]);
		expect(await productsNamed(ctx, name), 'aucun article créé').toEqual([]);
		await expect(page.locator('#form-price')).toHaveValue('1.250');

		await page.fill('#form-price', "12'34");
		// D1 : l'illisible garde le message actuel du prix (`product-error-price-invalid`).
		await expect(page.getByTestId('product-form-price-error')).toContainText('Format de prix invalide');
		await capture(page, testInfo, 'article-prix-illisible', 'e4810');

		await page.fill('#form-price', "1'250");
		await expect(page.getByTestId('product-form-price-error')).toHaveCount(0);
		const sent = waitForWrite(page, PRODUCT_WRITES);
		await page.getByRole('button', { name: 'Créer' }).click();
		const req = await sent;
		expect((req.postDataJSON() as { unitPrice: string }).unitPrice).toBe('1250');
		const res = await req.response();
		expect(res?.ok(), `création : ${res?.status()}`).toBeTruthy();
		await expect(page.locator('tbody').getByText(name)).toBeVisible();
		await capture(page, testInfo, 'article-prix-1250-enregistre', 'e4810');
		const [lu] = await productsNamed(ctx, name);
		expect(new Big(lu.unitPrice).eq(1250), `prix relu : ${lu.unitPrice}`).toBe(true);

		// AC1, AC11 — un article au prix 1.2340 se rouvre « 1.2340 », sans message.
		const millieme = `Article millième ${suffix()}`;
		const created = await ctx.post('/api/v1/products', {
			data: { name: millieme, description: null, unitPrice: '1.2340', vatRate: '8.10' },
		});
		expect(created.ok(), `article : ${created.status()} ${await created.text()}`).toBeTruthy();
		await page.goto(`/products?search=${encodeURIComponent(millieme)}`);
		await page.locator('tbody tr', { hasText: millieme }).getByLabel('Modifier').click();
		await expect(page.locator('#form-price')).toHaveValue('1.2340');
		await expect(page.getByTestId('product-form-price-error')).toHaveCount(0);
		await capture(page, testInfo, 'article-rouvert-1.2340', 'e4810');
	} finally {
		await disposeContextSafe(ctx);
	}
});

// ---------------------------------------------------------------------------
// Story 48-10 — #1175, frais d'un niveau de rappel
// ---------------------------------------------------------------------------

const DUNNING_WRITES: WriteRoute[] = [
	{ method: 'POST', path: /^\/api\/v1\/dunning-levels$/ },
	{ method: 'PUT', path: /^\/api\/v1\/dunning-levels\/\d+$/ },
];

type DunningLevel = { id: number; levelNumber: number; feeAmount: string };

async function dunningLevels(ctx: APIRequestContext): Promise<DunningLevel[]> {
	const r = await ctx.get('/api/v1/dunning-levels');
	expect(r.ok(), `niveaux : ${r.status()}`).toBeTruthy();
	return (await r.json()) as DunningLevel[];
}

test('rappels, frais : « 1.250 » refusé comme ambigu, rien n’est écrit ; « 1’250 » part 1250', async ({
	page,
}, testInfo) => {
	await seedTestState('with-company');
	await login(page);
	const ctx = await authedApiContext(page);
	try {
		await page.goto('/settings/dunning');
		await expect(page.getByTestId('dunning-level-row')).toHaveCount(3, { timeout: 5000 });
		const before = await dunningLevels(ctx);
		const writes = countWrites(page, DUNNING_WRITES);
		await page.getByTestId('dunning-level-row').nth(1).getByRole('button', { name: 'Modifier' }).click();
		await page.getByTestId('dunning-form-fee').fill('1.250');
		await expect.soft(page.getByTestId('dunning-form-fee-error')).toContainText('Montant ambigu');
		await expect.soft(page.getByTestId('dunning-form-fee')).toHaveAttribute('aria-invalid', 'true');
		await page.getByTestId('dunning-form-submit').click();
		await page.waitForTimeout(300);
		expect(writes(), 'aucune requête d’écriture de niveau').toEqual([]);
		await capture(page, testInfo, 'rappel-frais-ambigu', 'e4810');
		expect(await dunningLevels(ctx), 'niveaux inchangés').toEqual(before);
		await expect(page.getByTestId('dunning-form-fee')).toHaveValue('1.250');

		await page.getByTestId('dunning-form-fee').fill("12'34");
		await expect(page.getByTestId('dunning-form-fee-error')).toContainText('2 décimales au plus');
		await capture(page, testInfo, 'rappel-frais-illisible', 'e4810');

		await page.getByTestId('dunning-form-fee').fill("1'250");
		await expect(page.getByTestId('dunning-form-fee-error')).toHaveCount(0);
		const sent = waitForWrite(page, DUNNING_WRITES);
		await page.getByTestId('dunning-form-submit').click();
		const req = await sent;
		expect((req.postDataJSON() as { feeAmount: string }).feeAmount).toBe('1250');
		const res = await req.response();
		expect(res?.ok(), `modification : ${res?.status()}`).toBeTruthy();
		await expect(page.getByTestId('dunning-form-fee')).toHaveCount(0);
		await capture(page, testInfo, 'rappel-frais-1250-enregistre', 'e4810');
		const lu = (await dunningLevels(ctx)).find((l) => l.id === before[1].id)!;
		expect(new Big(lu.feeAmount).eq(1250), `frais relus : ${lu.feeAmount}`).toBe(true);
	} finally {
		await disposeContextSafe(ctx);
	}
});

// ---------------------------------------------------------------------------
// Story 48-10 — #1176, montant minimal d'une facture
// ---------------------------------------------------------------------------

const INVOICE_SETTINGS_WRITES: WriteRoute[] = [
	{ method: 'PUT', path: /^\/api\/v1\/company\/invoice-settings$/ },
];

async function minimumInvoiceAmount(ctx: APIRequestContext): Promise<string | null> {
	const r = await ctx.get('/api/v1/company/invoice-settings');
	expect(r.ok(), `réglages : ${r.status()}`).toBeTruthy();
	return ((await r.json()) as { minimumInvoiceAmount: string | null }).minimumInvoiceAmount;
}

test('facturation, montant minimum : « 1.250 » refusé comme ambigu, rien n’est écrit ; « 1’250 » part 1250', async ({
	page,
}, testInfo) => {
	await seedTestState('with-company');
	await login(page);
	const ctx = await authedApiContext(page);
	try {
		const before = await minimumInvoiceAmount(ctx);
		await page.goto('/settings/invoicing');
		const field = page.getByTestId('settings-minimum-invoice-amount');
		await expect(field).toBeVisible();
		const writes = countWrites(page, INVOICE_SETTINGS_WRITES);
		await field.fill('1.250');
		await expect.soft(page.getByTestId('settings-minimum-invoice-amount-error')).toContainText(
			'Montant ambigu',
		);
		await expect.soft(field).toHaveAttribute('aria-invalid', 'true');
		await expect.soft(page.getByTestId('settings-invoicing-save')).toBeDisabled();
		await capture(page, testInfo, 'minimum-ambigu', 'e4810');
		await page.getByTestId('settings-invoicing-save').click({ force: true });
		await field.press('Enter');
		await page.waitForTimeout(300);
		expect(writes(), 'aucune requête d’écriture des réglages').toEqual([]);
		expect(await minimumInvoiceAmount(ctx), 'minimum inchangé').toBe(before);
		await expect(field).toHaveValue('1.250');

		await field.fill("12'34");
		await expect(page.getByTestId('settings-minimum-invoice-amount-error')).toContainText(
			'2 décimales au plus',
		);
		await capture(page, testInfo, 'minimum-illisible', 'e4810');

		await field.fill("1'250");
		await expect(page.getByTestId('settings-minimum-invoice-amount-error')).toHaveCount(0);
		const sent = waitForWrite(page, INVOICE_SETTINGS_WRITES);
		await page.getByTestId('settings-invoicing-save').click();
		const req = await sent;
		expect(
			(req.postDataJSON() as { minimumInvoiceAmount: string | null }).minimumInvoiceAmount,
		).toBe('1250');
		const res = await req.response();
		expect(res?.ok(), `enregistrement : ${res?.status()}`).toBeTruthy();
		const lu = await minimumInvoiceAmount(ctx);
		expect(lu !== null && new Big(lu).eq(1250), `minimum relu : ${lu}`).toBe(true);
		// Rouvert : la forme rendue par le serveur (`1250.0000`) se relit sans message.
		await page.reload();
		await expect(field).toHaveValue(lu!);
		await expect(page.getByTestId('settings-minimum-invoice-amount-error')).toHaveCount(0);
		await capture(page, testInfo, 'minimum-1250-enregistre-relu', 'e4810');
	} finally {
		await disposeContextSafe(ctx);
	}
});

// ---------------------------------------------------------------------------
// Story 48-10 — #1177, bornes de montant du filtre des écritures (lecture)
// ---------------------------------------------------------------------------

/** La prochaine lecture de la liste des écritures (`GET /api/v1/journal-entries`). */
function nextListRequest(page: Page): Promise<Request> {
	return page.waitForRequest(
		(req) => req.method() === 'GET' && new URL(req.url()).pathname === '/api/v1/journal-entries',
	);
}

test('filtre des écritures : « 1’250 » filtre à 1250, « 1.250 » n’est pas appliqué et le dit', async ({
	page,
}, testInfo) => {
	await seedTestState('with-data');
	await login(page);
	await page.goto('/journal-entries');
	const min = page.locator('#filter-amount-min');
	await expect(min).toBeVisible();

	let req = nextListRequest(page);
	await min.fill("1'250");
	expect(new URL((await req).url()).searchParams.get('amountMin')).toBe('1250');
	await expect(page).toHaveURL(/[?&]amountMin=1250(&|$)/);

	req = nextListRequest(page);
	await min.fill('1.250');
	expect(new URL((await req).url()).searchParams.has('amountMin'), 'borne ambiguë omise').toBe(false);
	await expect(page.getByTestId('journal-entries-filter-amount-min-problem')).toContainText(
		'Montant ambigu',
	);
	await expect(page.getByTestId('journal-entries-filter-amount-min-problem')).toContainText(
		'pas appliqué',
	);
	await expect(page).not.toHaveURL(/amountMin=/);
	await expect(min).toHaveValue('1.250');
	await capture(page, testInfo, 'filtre-ambigu', 'e4810');

	// Signet ancien : la borne s'affiche, n'est pas envoyée, l'URL la retire.
	req = nextListRequest(page);
	await page.goto('/journal-entries?amountMin=1.250');
	expect(new URL((await req).url()).searchParams.has('amountMin'), 'signet : borne omise').toBe(false);
	await expect(page.getByTestId('journal-entries-filter-amount-min-problem')).toBeVisible();
	await expect(page.locator('#filter-amount-min')).toHaveValue('1.250');
	await expect(page).not.toHaveURL(/amountMin=/);
	await capture(page, testInfo, 'filtre-signet-ambigu', 'e4810');
});
