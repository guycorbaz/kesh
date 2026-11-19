/**
 * Story 48-12b (#975, #985) — le refus de doublon à l'écran, la réponse perdue d'un
 * enregistrement, la complétion relue et le repassage en brouillon d'une annulée en
 * doublon (tables A et B de la fiche).
 *
 * Les coupures sont réelles (patron `supplier-invoice-unknown-outcome.spec.ts`) :
 * `route.fetch()` envoie la requête au serveur, puis `route.abort()` jette sa réponse — un
 * `abort()` seul n'enverrait rien. Une seule coupure par test : le filtre porte la méthode
 * et un drapeau.
 *
 * Décor synthétique : le seed `with-company`, un fournisseur créé par l'API à chaque test
 * (nom unique), un document sans QR déposé dans le dossier d'import pour la complétion. Les
 * captures (thème clair) s'écrivent dans `KESH_E2E_CAPTURES_DIR`, sinon dans la sortie du test.
 */
import path from 'node:path';
import { expect, test, type APIRequestContext, type Page, type TestInfo } from '@playwright/test';
import {
	seedTestState,
	clearAuthStorage,
	authedApiContext,
	disposeContextSafe,
} from './helpers/test-state';
import { PNG_SANS_QR, deposit, importViaApi, readFixture, token, uniqueCopy } from './helpers/inbox';
import { todayInZurich } from './helpers/api-fixtures';

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

type Account = { id: number; number: string; accountType: string; active: boolean; postable: boolean };
type Decor = { supplierId: number; supplierName: string; expense: number };

/** Un fournisseur neuf (nom unique) et un compte de charge ; le compte créanciers posé. */
async function decor(ctx: APIRequestContext, nom: string): Promise<Decor> {
	const accounts = (await (await ctx.get('/api/v1/accounts')).json()) as Account[];
	const payable = accounts.find((a) => a.number === '2000');
	const expense = accounts.find((a) => a.accountType === 'Expense' && a.active && a.postable);
	expect(payable && expense, 'comptes du décor').toBeTruthy();
	const s = await (await ctx.get('/api/v1/company/invoice-settings')).json();
	if (s.defaultPayableAccountId == null) {
		const put = await ctx.put('/api/v1/company/invoice-settings', {
			data: { ...s, defaultPayableAccountId: payable!.id },
		});
		expect(put.ok(), `compte créanciers : ${put.status()}`).toBeTruthy();
	}
	const supplierName = `${nom} ${token()}`;
	const sup = await ctx.post('/api/v1/contacts', {
		data: { contactType: 'Entreprise', name: supplierName, isClient: false, isSupplier: true },
	});
	expect(sup.ok(), `fournisseur : ${sup.status()}`).toBeTruthy();
	return { supplierId: (await sup.json()).id as number, supplierName, expense: expense!.id };
}

/** Les factures (non annulées comprises) du fournisseur, par l'API. */
async function invoicesOf(ctx: APIRequestContext, d: Decor): Promise<Array<{ id: number; status: string }>> {
	const r = await ctx.get(`/api/v1/supplier-invoices?limit=100&q=${encodeURIComponent(d.supplierName)}`);
	expect(r.ok(), `liste : ${r.status()}`).toBeTruthy();
	return ((await r.json()) as { items: Array<{ id: number; status: string; contactId: number }> }).items.filter(
		(i) => i.contactId === d.supplierId,
	);
}

async function fillForm(page: Page, d: Decor, numero: string | null, montant = '100.00'): Promise<void> {
	await page.getByTestId('supplier-invoice-supplier').selectOption(String(d.supplierId));
	if (numero !== null) await page.getByTestId('supplier-invoice-number').fill(numero);
	await page.getByTestId('supplier-invoice-date').fill(todayInZurich());
	const line = page.getByTestId('supplier-invoice-line').first();
	await line.getByTestId('supplier-invoice-line-desc').fill('Fourniture');
	await line.getByTestId('supplier-invoice-line-unit-price').fill(montant);
	await line.getByTestId('supplier-invoice-line-account').selectOption(String(d.expense));
}

/** Compte les requêtes `POST /api/v1/supplier-invoices` parties du navigateur. */
function countCreates(page: Page): { n: number } {
	const c = { n: 0 };
	page.on('request', (r) => {
		if (r.method() === 'POST' && /\/api\/v1\/supplier-invoices$/.test(new URL(r.url()).pathname)) c.n += 1;
	});
	return c;
}

const TEXTE_E6 =
	'L’enregistrement a peut-être abouti. Vérifiez dans la liste si la facture existe avant de la saisir de nouveau.';
const TEXTE_REGLE_1 =
	'Une facture de ce fournisseur porte déjà ce numéro pour la même année. Si c’est la même facture, ne l’enregistrez pas une seconde fois. Si le fournisseur a recommencé sa numérotation en cours d’année, ajoutez un caractère au numéro, par exemple « 15-b ».';

test('création coupée : E6 figé, un seul brouillon, reprise sans requête, puis E4 avec la facture citée', async ({
	page,
}, testInfo) => {
	await login(page);
	const ctx = await authedApiContext(page);
	try {
		const d = await decor(ctx, 'Fournisseur doublon E6');
		await page.goto('/supplier-invoices/new');
		const creates = countCreates(page);
		let coupe = false;
		await page.route('**/api/v1/supplier-invoices', async (route) => {
			if (route.request().method() !== 'POST' || coupe) return route.fallback();
			coupe = true;
			const response = await route.fetch();
			expect(response.status(), 'le serveur a bien créé').toBe(201);
			await route.abort('connectionreset');
		});
		await fillForm(page, d, 'FA-E6-1');
		await page.getByTestId('supplier-invoice-submit').click();

		const notice = page.getByTestId('supplier-invoice-unknown-outcome');
		await expect(notice).toBeVisible({ timeout: 40_000 });
		await expect(page.getByTestId('supplier-invoice-unknown-outcome-text')).toHaveText(TEXTE_E6);
		await expect(page.getByTestId('supplier-invoice-submit')).toBeDisabled();
		await expect(page.getByTestId('supplier-invoice-number')).toBeDisabled();
		await expect(page.getByTestId('supplier-invoice-confirm-not-duplicate')).toHaveCount(0);
		const link = page.getByTestId('supplier-invoice-unknown-outcome-link');
		await expect(link).toHaveAttribute('target', '_blank');
		await expect(link).toHaveAttribute(
			'href',
			`/supplier-invoices?q=${encodeURIComponent(d.supplierName)}`,
		);
		await capture(page, testInfo, 'e4812b-E6-creation');
		expect((await invoicesOf(ctx, d)).length, 'un seul brouillon').toBe(1);

		// Reprise : E1, saisie intacte, aucune requête.
		const avant = creates.n;
		await page.getByTestId('supplier-invoice-resume-entry').click();
		await expect(notice).toHaveCount(0);
		await expect(page.getByTestId('supplier-invoice-number')).toBeEnabled();
		await expect(page.getByTestId('supplier-invoice-number')).toHaveValue('FA-E6-1');
		expect(creates.n, 'la reprise n’envoie rien').toBe(avant);

		// Enregistrer de nouveau : le serveur refuse (même numéro) → E4.
		await page.getByTestId('supplier-invoice-submit').click();
		const dup = page.getByTestId('supplier-invoice-duplicate');
		await expect(dup).toBeVisible();
		await expect(page.getByTestId('supplier-invoice-duplicate-text')).toHaveText(TEXTE_REGLE_1);
		await expect(page.getByTestId('supplier-invoice-duplicate-link')).toHaveAttribute('target', '_blank');
		await expect(page.getByTestId('supplier-invoice-confirm-not-duplicate')).toHaveCount(0);
		await expect(page.getByTestId('supplier-invoice-number')).toBeEnabled();
		await capture(page, testInfo, 'e4812b-E4-doublon');
		expect((await invoicesOf(ctx, d)).length, 'toujours un seul brouillon').toBe(1);
	} finally {
		await disposeContextSafe(ctx);
	}
});

test('sans numéro : E5 figé, confirmation, deux brouillons ; confirmation coupée → E6 sans confirmation', async ({
	page,
}, testInfo) => {
	await login(page);
	const ctx = await authedApiContext(page);
	try {
		const d = await decor(ctx, 'Fournisseur doublon E5');
		await page.goto('/supplier-invoices/new');
		await fillForm(page, d, null, '42.00');
		await page.getByTestId('supplier-invoice-submit').click();
		await page.waitForURL(/\/supplier-invoices\/\d+$/);
		expect((await invoicesOf(ctx, d)).length).toBe(1);

		// Même date, même montant, sans numéro → E5.
		await page.goto('/supplier-invoices/new');
		await fillForm(page, d, null, '42.00');
		await page.getByTestId('supplier-invoice-submit').click();
		await expect(page.getByTestId('supplier-invoice-probable-duplicate')).toBeVisible();
		await expect(page.getByTestId('supplier-invoice-submit')).toHaveCount(0);
		await expect(page.getByTestId('supplier-invoice-line-desc')).toBeDisabled();
		await capture(page, testInfo, 'e4812b-E5-doublon-probable');

		// La confirmation coupée après l'écriture → E6, sans bouton de confirmation.
		let coupe = false;
		await page.route('**/api/v1/supplier-invoices', async (route) => {
			if (route.request().method() !== 'POST' || coupe) return route.fallback();
			coupe = true;
			const body = route.request().postDataJSON() as { confirmNotDuplicate?: boolean };
			expect(body.confirmNotDuplicate, 'le corps gardé porte le drapeau').toBe(true);
			const response = await route.fetch();
			expect(response.status()).toBe(201);
			await route.abort('connectionreset');
		});
		await page.getByTestId('supplier-invoice-confirm-not-duplicate').click();
		await expect(page.getByTestId('supplier-invoice-unknown-outcome')).toBeVisible({ timeout: 40_000 });
		await expect(page.getByTestId('supplier-invoice-confirm-not-duplicate')).toHaveCount(0);
		expect((await invoicesOf(ctx, d)).length, 'la confirmation a écrit : deux brouillons').toBe(2);
	} finally {
		await disposeContextSafe(ctx);
	}
});

test('sans numéro : la confirmation enregistre le second brouillon', async ({ page }) => {
	await login(page);
	const ctx = await authedApiContext(page);
	try {
		const d = await decor(ctx, 'Fournisseur doublon confirme');
		await page.goto('/supplier-invoices/new');
		await fillForm(page, d, null, '55.00');
		await page.getByTestId('supplier-invoice-submit').click();
		await page.waitForURL(/\/supplier-invoices\/\d+$/);
		await page.goto('/supplier-invoices/new');
		await fillForm(page, d, null, '55.00');
		await page.getByTestId('supplier-invoice-submit').click();
		await page.getByTestId('supplier-invoice-confirm-not-duplicate').click();
		await page.waitForURL(/\/supplier-invoices\/\d+$/);
		expect((await invoicesOf(ctx, d)).length).toBe(2);
	} finally {
		await disposeContextSafe(ctx);
	}
});

/** Un document sans QR, déposé et importé ; rend son identifiant. */
async function documentATraiter(page: Page): Promise<number> {
	const name = `e2e-e4812b-${token()}.png`;
	await deposit(name, uniqueCopy(PNG_SANS_QR, 'png'));
	const report = await importViaApi(page);
	const entry = report.accepted.find((a) => a.fileName === name);
	expect(entry, `document importé : ${JSON.stringify(report)}`).toBeTruthy();
	return entry!.importedSupplierInvoiceId as number;
}

test('complétion coupée après l’écriture : E9, la pièce est déjà enregistrée, lien vers la facture', async ({
	page,
}, testInfo) => {
	await login(page);
	const ctx = await authedApiContext(page);
	try {
		const d = await decor(ctx, 'Fournisseur complétion E9');
		const docId = await documentATraiter(page);
		let coupe = false;
		await page.route(`**/api/v1/imported-supplier-invoices/${docId}/complete`, async (route) => {
			if (route.request().method() !== 'POST' || coupe) return route.fallback();
			coupe = true;
			const response = await route.fetch();
			expect(response.status(), 'le serveur a bien complété').toBe(200);
			await route.abort('connectionreset');
		});
		await page.goto(`/supplier-invoices/new?document=${docId}`);
		await fillForm(page, d, 'FA-E9-1');
		await page.getByTestId('supplier-invoice-submit').click();
		const notice = page.getByTestId('supplier-invoice-already-completed');
		await expect(notice).toBeVisible({ timeout: 40_000 });
		await expect(notice).not.toContainText('déjà été traité ou retiré');
		const link = page.getByTestId('supplier-invoice-already-completed-link');
		await expect(link).toHaveAttribute('target', '_blank');
		const created = await invoicesOf(ctx, d);
		expect(created.length).toBe(1);
		await expect(link).toHaveAttribute('href', `/supplier-invoices/${created[0].id}`);
		await expect(page.getByTestId('supplier-invoice-submit')).toBeDisabled();
		await expect(page.getByTestId('supplier-invoice-number')).toBeDisabled();
		await capture(page, testInfo, 'e4812b-E9-deja-enregistree');
	} finally {
		await disposeContextSafe(ctx);
	}
});

test('complétion coupée avant l’envoi : E7 « pour l’instant », le document reste à traiter', async ({
	page,
}, testInfo) => {
	await login(page);
	const ctx = await authedApiContext(page);
	try {
		const d = await decor(ctx, 'Fournisseur complétion E7');
		const docId = await documentATraiter(page);
		let coupe = false;
		await page.route(`**/api/v1/imported-supplier-invoices/${docId}/complete`, async (route) => {
			if (route.request().method() !== 'POST' || coupe) return route.fallback();
			coupe = true;
			await route.abort('connectionreset');
		});
		await page.goto(`/supplier-invoices/new?document=${docId}`);
		await fillForm(page, d, 'FA-E7-1');
		await page.getByTestId('supplier-invoice-submit').click();
		await expect(page.getByTestId('supplier-invoice-form-error')).toHaveText(
			'Aucune facture n’est enregistrée pour l’instant : vous pouvez réessayer.',
			{ timeout: 40_000 },
		);
		await expect(page.getByTestId('supplier-invoice-submit')).toBeEnabled();
		await capture(page, testInfo, 'e4812b-E7-pour-l-instant');
		expect((await invoicesOf(ctx, d)).length).toBe(0);
	} finally {
		await disposeContextSafe(ctx);
	}
});

test('fiche d’une annulée en doublon : F4, le repassage confirmé passe', async ({ page }, testInfo) => {
	await login(page);
	const ctx = await authedApiContext(page);
	try {
		const d = await decor(ctx, 'Fournisseur annulée F4');
		const body = (numero: string) => ({
			contactId: d.supplierId,
			supplierInvoiceNumber: numero,
			invoiceDate: todayInZurich(),
			lines: [
				{ description: 'Fourniture', quantity: '1', unitPrice: '30.00', vatRate: '0', expenseAccountId: d.expense },
			],
		});
		const a = await (await ctx.post('/api/v1/supplier-invoices', { data: body('F4-15') })).json();
		const v = await (
			await ctx.post(`/api/v1/supplier-invoices/${a.id}/validate`, { data: { version: a.version } })
		).json();
		const x = await ctx.post(`/api/v1/supplier-invoices/${a.id}/cancel`, { data: {} });
		expect(x.ok(), `annulation : ${x.status()}`).toBeTruthy();
		expect(v.status).toBe('open');
		const b = await ctx.post('/api/v1/supplier-invoices', { data: body('F4-15') });
		expect(b.ok(), `facture active : ${b.status()}`).toBeTruthy();
		const bId = (await b.json()).id as number;

		await page.goto(`/supplier-invoices/${a.id}`);
		await page.getByTestId('supplier-invoice-unvalidate').click();
		await page.getByTestId('supplier-invoice-unvalidate-confirm').click();
		const notice = page.getByTestId('supplier-invoice-unvalidate-duplicate');
		await expect(notice).toBeVisible();
		await expect(page.getByTestId('supplier-invoice-unvalidate-duplicate-text')).toHaveText(
			'Une facture équivalente non annulée existe. Repasser celle-ci en brouillon en ferait un doublon : ne le faites que pour la retirer ou la corriger ensuite.',
		);
		await expect(page.getByTestId('supplier-invoice-unvalidate-duplicate-link')).toHaveAttribute(
			'href',
			`/supplier-invoices/${bId}`,
		);
		await expect(page.getByTestId('supplier-invoice-status')).toHaveText('Annulée');
		await capture(page, testInfo, 'e4812b-F4-repassage-doublon');
		await page.getByTestId('supplier-invoice-unvalidate-anyway').click();
		await expect(page.getByTestId('supplier-invoice-status')).toHaveText('Brouillon');
		await expect(notice).toHaveCount(0);
	} finally {
		await disposeContextSafe(ctx);
	}
});

// ---------------------------------------------------------------------------
// Recette visuelle — les états que les parcours ci-dessus ne montrent pas (fiche 48-12b,
// § Recette visuelle). Réponses synthétiques (`route.fulfill`) pour les refus sans
// `details` ; le serveur n'en rend jamais, l'écran doit pourtant les montrer lisiblement.
// ---------------------------------------------------------------------------

/** Une promesse que le test résout quand il veut relâcher une requête retenue. */
function verrou(): { attendre: Promise<void>; relacher: () => void } {
	let relacher = () => {};
	const attendre = new Promise<void>((r) => (relacher = r));
	return { attendre, relacher };
}

test('recette visuelle : E2, E3, E1 inactif (scan, panneau), notification de C2, E6 modification et complétion, E7 et F6 sans details', async ({
	page,
}, testInfo) => {
	test.setTimeout(240_000);
	await login(page);
	const ctx = await authedApiContext(page);
	try {
		const d = await decor(ctx, 'Fournisseur recette');

		// E2 — envoi retenu : formulaire figé, « … ».
		const e2 = verrou();
		await page.route('**/api/v1/supplier-invoices', async (route) => {
			if (route.request().method() !== 'POST') return route.fallback();
			await e2.attendre;
			await route.fallback();
		});
		await page.goto('/supplier-invoices/new');
		await fillForm(page, d, 'REC-E2');
		await page.getByTestId('supplier-invoice-submit').click();
		await expect(page.getByTestId('supplier-invoice-submit')).toHaveText('…');
		await expect(page.getByTestId('supplier-invoice-number')).toBeDisabled();
		await capture(page, testInfo, 'e4812b-E2-envoi');
		// E3 — succès, navigation retenue : figé, jamais réactivé.
		const e3 = verrou();
		await page.route('**/_app/immutable/nodes/**', async (route) => {
			await e3.attendre;
			await route.fallback();
		});
		e2.relacher();
		await expect(page.getByTestId('supplier-invoice-submit')).not.toHaveText('…');
		await expect(page.getByTestId('supplier-invoice-number')).toBeDisabled();
		await capture(page, testInfo, 'e4812b-E3-enregistre');
		e3.relacher();
		await page.waitForURL(/\/supplier-invoices\/\d+$/);
		await page.unroute('**/_app/immutable/nodes/**');
		await page.unroute('**/api/v1/supplier-invoices');

		// E1 — « Enregistrer » inactif, panneau « nouveau fournisseur » ouvert.
		await page.goto('/supplier-invoices/new');
		await fillForm(page, d, 'REC-E1');
		await page.getByTestId('supplier-new-open').click();
		await expect(page.getByTestId('supplier-invoice-submit')).toBeDisabled();
		await capture(page, testInfo, 'e4812b-E1-panneau-ouvert');

		// C2 — création du contact retenue, « Annuler », envoi refusé (E5), puis résolution :
		// le contact entre dans la liste sans être sélectionné.
		const c2 = verrou();
		await page.route('**/api/v1/contacts', async (route) => {
			if (route.request().method() !== 'POST') return route.fallback();
			await c2.attendre;
			await route.fallback();
		});
		await page.getByTestId('supplier-new-name').fill(`Fournisseur C2 ${token()}`);
		await page.getByTestId('supplier-new-create').click();
		await page.getByTestId('supplier-new-cancel').click();
		await expect(page.getByTestId('supplier-invoice-submit')).toBeEnabled();
		await page.route('**/api/v1/supplier-invoices', async (route) => {
			if (route.request().method() !== 'POST') return route.fallback();
			await route.fulfill({
				status: 409,
				contentType: 'application/json',
				body: JSON.stringify({
					error: {
						code: 'SUPPLIER_INVOICE_PROBABLE_DUPLICATE',
						message: 'refus synthétique',
						details: { supplierInvoiceId: 1, internalNumber: null, status: 'draft', confirmable: true },
					},
				}),
			});
		});
		await page.getByTestId('supplier-invoice-submit').click();
		await expect(page.getByTestId('supplier-invoice-probable-duplicate')).toBeVisible();
		c2.relacher();
		await expect(page.locator('body')).toContainText('Fournisseur créé, non sélectionné.');
		await expect(page.getByTestId('supplier-invoice-supplier')).toHaveValue(String(d.supplierId));
		await capture(page, testInfo, 'e4812b-C2-fournisseur-non-selectionne');
		await page.unroute('**/api/v1/contacts');
		await page.unroute('**/api/v1/supplier-invoices');

		// E1 — scan en cours : « Enregistrer » inactif.
		const scan = verrou();
		await page.route('**/api/v1/supplier-invoices/scan-qr', async (route) => {
			await scan.attendre;
			await route.fallback();
		});
		await page.goto('/supplier-invoices/new');
		await fillForm(page, d, 'REC-SCAN');
		await page.getByTestId('supplier-invoice-scan-file').setInputFiles({
			name: 'spc_e2e_invoice.png',
			mimeType: 'image/png',
			buffer: await readFixture('spc_e2e_invoice.png'),
		});
		await expect(page.getByTestId('supplier-invoice-scan-btn')).toBeDisabled({ timeout: 20_000 });
		await expect(page.getByTestId('supplier-invoice-submit')).toBeDisabled();
		await capture(page, testInfo, 'e4812b-E1-scan-en-cours');
		scan.relacher();
		await page.unroute('**/api/v1/supplier-invoices/scan-qr');

		// E7 — un doublon sans `details` lisibles : le message du serveur, saisie active.
		await page.goto('/supplier-invoices/new');
		await fillForm(page, d, 'REC-E7');
		await page.route('**/api/v1/supplier-invoices', async (route) => {
			if (route.request().method() !== 'POST') return route.fallback();
			await route.fulfill({
				status: 409,
				contentType: 'application/json',
				body: JSON.stringify({
					error: { code: 'SUPPLIER_INVOICE_DUPLICATE', message: 'Message du serveur, sans details.' },
				}),
			});
		});
		await page.getByTestId('supplier-invoice-submit').click();
		await expect(page.getByTestId('supplier-invoice-form-error')).toHaveText('Message du serveur, sans details.');
		await expect(page.getByTestId('supplier-invoice-submit')).toBeEnabled();
		await capture(page, testInfo, 'e4812b-E7-doublon-sans-details');
		await page.unroute('**/api/v1/supplier-invoices');

		// E6, variante modification — PUT coupé après l'écriture.
		const draft = await (
			await ctx.post('/api/v1/supplier-invoices', {
				data: {
					contactId: d.supplierId,
					supplierInvoiceNumber: `REC-PUT-${token()}`,
					invoiceDate: todayInZurich(),
					lines: [
						{ description: 'Fourniture', quantity: '1', unitPrice: '12.00', vatRate: '0', expenseAccountId: d.expense },
					],
				},
			})
		).json();
		let coupe = false;
		await page.route(`**/api/v1/supplier-invoices/${draft.id}`, async (route) => {
			if (route.request().method() !== 'PUT' || coupe) return route.fallback();
			coupe = true;
			const response = await route.fetch();
			expect(response.status()).toBe(200);
			await route.abort('connectionreset');
		});
		await page.goto(`/supplier-invoices/${draft.id}/edit`);
		await page.getByTestId('supplier-invoice-line-desc').first().fill('Fourniture modifiée');
		await page.getByTestId('supplier-invoice-submit').click();
		await expect(page.getByTestId('supplier-invoice-unknown-outcome')).toBeVisible({ timeout: 40_000 });
		await expect(page.getByTestId('supplier-invoice-unknown-outcome-link')).toHaveAttribute(
			'href',
			`/supplier-invoices/${draft.id}`,
		);
		await capture(page, testInfo, 'e4812b-E6-modification');

		// E6, complétion — réponse en 503, relecture du document en échec.
		const docId = await documentATraiter(page);
		await page.route(`**/api/v1/imported-supplier-invoices/${docId}/complete`, async (route) => {
			if (route.request().method() !== 'POST') return route.fallback();
			await route.fulfill({
				status: 503,
				contentType: 'application/json',
				body: JSON.stringify({ error: { code: 'SERVICE_UNAVAILABLE', message: 'indisponible' } }),
			});
		});
		await page.goto(`/supplier-invoices/new?document=${docId}`);
		await expect(page.getByTestId('supplier-invoice-form')).toBeVisible();
		await page.route('**/api/v1/imported-supplier-invoices?status=*', async (route) => {
			await route.fulfill({
				status: 500,
				contentType: 'application/json',
				body: JSON.stringify({ error: { code: 'INTERNAL_ERROR', message: 'relecture refusée' } }),
			});
		});
		await fillForm(page, d, 'REC-E6C');
		await page.getByTestId('supplier-invoice-submit').click();
		await expect(page.getByTestId('supplier-invoice-unknown-outcome')).toBeVisible({ timeout: 40_000 });
		await capture(page, testInfo, 'e4812b-E6-completion');
		await page.unroute('**/api/v1/imported-supplier-invoices?status=*');

		// F6 — repassage d'une annulée refusé par un doublon sans `details` : message du serveur.
		const a = await (
			await ctx.post('/api/v1/supplier-invoices', {
				data: {
					contactId: d.supplierId,
					supplierInvoiceNumber: `REC-F6-${token()}`,
					invoiceDate: todayInZurich(),
					lines: [
						{ description: 'Fourniture', quantity: '1', unitPrice: '13.00', vatRate: '0', expenseAccountId: d.expense },
					],
				},
			})
		).json();
		await ctx.post(`/api/v1/supplier-invoices/${a.id}/validate`, { data: { version: a.version } });
		await ctx.post(`/api/v1/supplier-invoices/${a.id}/cancel`, { data: {} });
		await page.route(`**/api/v1/supplier-invoices/${a.id}/unvalidate`, async (route) => {
			await route.fulfill({
				status: 409,
				contentType: 'application/json',
				body: JSON.stringify({
					error: { code: 'SUPPLIER_INVOICE_DUPLICATE', message: 'Message du serveur, sans details.' },
				}),
			});
		});
		await page.goto(`/supplier-invoices/${a.id}`);
		await page.getByTestId('supplier-invoice-unvalidate').click();
		await page.getByTestId('supplier-invoice-unvalidate-confirm').click();
		await expect(page.getByTestId('supplier-invoice-unvalidate-error')).toHaveText(
			'Message du serveur, sans details.',
		);
		await expect(page.getByTestId('supplier-invoice-unvalidate-duplicate')).toHaveCount(0);
		await capture(page, testInfo, 'e4812b-F6-doublon-sans-details');
	} finally {
		await disposeContextSafe(ctx);
	}
});
