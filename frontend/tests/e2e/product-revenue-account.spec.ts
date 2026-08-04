import { expect, test } from '@playwright/test';
import {
	seedTestState,
	clearAuthStorage,
	authedApiContext,
	disposeContextSafe,
} from './helpers/test-state';

/**
 * Tests E2E — compte de produit sur la fiche produit (Story 16-2b, AC-B8).
 *
 * # Ce que ces tests prouvent, et que rien d'autre ne prouve
 *
 * Vitest vérifie que `InvoiceForm` **construit** le bon `revenueAccountId` ; les
 * tests Rust de 16-2a vérifient que le backend **valide** et **persiste**. Aucun
 * des deux ne vérifie que la valeur **traverse la frontière HTTP** depuis la
 * fiche produit : une clé oubliée du payload, un nom divergent entre TS et
 * serde, un `PUT` full-replace qui efface — tout cela passe les deux suites et
 * casse en production.
 *
 * ⚠️ **Convention Playwright du dépôt** : le fichier DOIT s'appeler `*.spec.ts`.
 * Un `*.test.ts` dans `tests/e2e/` est **silencieusement ignoré**.
 *
 * # Les quatre exécutions, et le canal de chacune
 *
 * 1. **catalogue → facture** — le produit est créé **PAR L'INTERFACE**. C'est ce
 *    scénario, et lui seul, qui exerce le **payload HTTP** de la fiche et
 *    discrimine donc la mutation 4 (retrait du champ du payload).
 * 2. **compte archivé → marqueur et refus** — le compte est assigné **par
 *    l'API**, délibérément : par l'interface, ce scénario passerait aussi par le
 *    payload et rougirait sous la mutation 4, qui n'en annonce qu'un.
 * 3. **archiver puis ROUVRIR la fiche produit** — le seul qui exerce le
 *    `fetchAccounts(true)` **du catalogue**. Les scénarios 1 et 2 passent par
 *    celui d'`InvoiceForm`, un appel **distinct** que cette story ne touche pas :
 *    sans ce troisième scénario, la mutation 2 n'aurait aucun tueur E2E.
 * 4. **éditer puis « Nouveau produit »** — l'héritage silencieux d'un article à
 *    l'autre, que `openCreate()` doit couper.
 */

test.beforeAll(async () => {
	await seedTestState('with-company');
});

test.afterEach(async ({ page }) => {
	await clearAuthStorage(page);
});

async function login(page: import('@playwright/test').Page) {
	await page.goto('/login');
	await page.fill('#username', 'admin');
	await page.fill('#password', 'admin123');
	await page.click('button[type="submit"]');
	await expect(page).toHaveURL('/');
}

function uniqSuffix(): string {
	return `${Date.now()}-${Math.floor(Math.random() * 1e6)}`;
}

/** Compte de produit imputable, hors des plages du plan seedé. */
async function mkRevenueAccount(
	ctx: import('@playwright/test').APIRequestContext,
	suffix: string
): Promise<{ id: number; number: string; name: string; version: number }> {
	const number = String(39000 + Math.floor(Math.random() * 900));
	const res = await ctx.post('/api/v1/accounts', {
		data: { number, name: `Compte produit ${suffix}`, accountType: 'Revenue', postable: true, role: null },
	});
	expect(res.ok(), `création compte ${number} échouée: ${res.status()}`).toBeTruthy();
	return (await res.json()) as { id: number; number: string; name: string; version: number };
}

/** Remplit le dialogue de la fiche produit et enregistre. */
async function fillProductForm(
	page: import('@playwright/test').Page,
	name: string,
	accountLabel?: string
) {
	await page.fill('#form-name', name);
	await page.fill('#form-price', '250.00');
	if (accountLabel) {
		await page.fill('#form-revenue-account', accountLabel);
		// Le dropdown filtre sur la saisie ; la première proposition est la bonne.
		await page.click(`text=${accountLabel}`);
	}
	await page.click('button[type="submit"]');
}

// ===========================================================================
// 1 — catalogue → facture. Produit créé PAR L'INTERFACE (discrimine mut. 4).
// ===========================================================================

test('fiche produit avec compte → facture depuis catalogue → la ligne porte le compte', async ({
	page,
}) => {
	await login(page);
	const suffix = uniqSuffix();
	const ctx = await authedApiContext(page);

	try {
		const account = await mkRevenueAccount(ctx, suffix);
		const productName = `Article E2E ${suffix}`;

		// --- Le produit est créé PAR L'INTERFACE : c'est ce geste, et lui seul,
		// qui fait transiter le champ par le payload HTTP de la fiche.
		await page.goto('/products');
		await page.click('text=Nouveau produit');
		await fillProductForm(page, productName, `${account.number} — ${account.name}`);

		// Relu depuis l'API : le champ a bien traversé la frontière HTTP.
		const list = await ctx.get(`/api/v1/products?search=${encodeURIComponent(productName)}`);
		expect(list.ok()).toBeTruthy();
		const body = (await list.json()) as { items: Array<{ id: number; defaultRevenueAccountId: number | null }> };
		const created = body.items.find((p) => p.defaultRevenueAccountId !== null);
		expect(
			created?.defaultRevenueAccountId,
			'le compte doit avoir traversé le payload de la fiche produit'
		).toBe(account.id);

		// --- La facture montée depuis cet article porte le compte sur sa ligne.
		const contactRes = await ctx.post('/api/v1/contacts', {
			data: { contactType: 'Entreprise', name: `Client ${suffix}`, isCustomer: true },
		});
		expect(contactRes.ok()).toBeTruthy();
		const contact = (await contactRes.json()) as { id: number };

		const invRes = await ctx.post('/api/v1/invoices', {
			data: { contactId: contact.id, date: '2026-06-15', lines: [] },
		});
		expect(invRes.ok()).toBeTruthy();
		const invoice = (await invRes.json()) as { id: number };

		await page.goto(`/invoices/${invoice.id}/edit`);
		await page.click('text=Depuis catalogue');
		await page.click(`text=${productName}`);

		const accountField = page.locator('input[aria-autocomplete="list"]').last();
		await expect(
			accountField,
			'la ligne créée depuis le catalogue doit porter le compte de l’article'
		).toHaveValue(new RegExp(account.number));
	} finally {
		await disposeContextSafe(ctx);
	}
});

// ===========================================================================
// 2 — compte archivé → marqueur et refus. Compte assigné PAR L'API.
// ===========================================================================

test('article dont le compte a été archivé : la ligne est marquée et l’enregistrement refusé', async ({
	page,
}) => {
	await login(page);
	const suffix = uniqSuffix();
	const ctx = await authedApiContext(page);

	try {
		const account = await mkRevenueAccount(ctx, suffix);

		// ⚠️ Assigné PAR L'API, pas par l'interface : ce scénario n'a rien à
		// prouver sur le payload, et y passer le ferait rougir sous la mutation 4.
		const prodRes = await ctx.post('/api/v1/products', {
			data: {
				name: `Article archivé ${suffix}`,
				unitPrice: '250.00',
				vatRate: '8.10',
				defaultRevenueAccountId: account.id,
			},
		});
		expect(prodRes.ok(), `création produit échouée: ${prodRes.status()}`).toBeTruthy();
		const product = (await prodRes.json()) as { id: number; name: string };

		// Le compte est archivé APRÈS coup, depuis le plan comptable.
		const archiveRes = await ctx.put(`/api/v1/accounts/${account.id}/archive`, {
			data: { version: account.version },
		});
		expect(archiveRes.ok(), `archivage échoué: ${archiveRes.status()}`).toBeTruthy();

		const contactRes = await ctx.post('/api/v1/contacts', {
			data: { contactType: 'Entreprise', name: `Client ${suffix}`, isCustomer: true },
		});
		const contact = (await contactRes.json()) as { id: number };
		const invRes = await ctx.post('/api/v1/invoices', {
			data: { contactId: contact.id, date: '2026-06-15', lines: [] },
		});
		const invoice = (await invRes.json()) as { id: number };

		await page.goto(`/invoices/${invoice.id}/edit`);
		await page.click('text=Depuis catalogue');
		await page.click(`text=${product.name}`);

		// D-B1 : afficher + signaler + bloquer. Le compte est recopié tel quel —
		// retomber en silence sur le défaut société CACHERAIT que la fiche
		// produit est à corriger.
		const accountField = page.locator('input[aria-autocomplete="list"]').last();
		await expect(accountField, 'le libellé du compte archivé reste affiché').toHaveValue(
			new RegExp(account.number)
		);
		await expect(page.locator('text=/Compte invalide/').first()).toBeVisible();
		await expect(page.getByTestId('create-invoice-button')).toBeDisabled();
	} finally {
		await disposeContextSafe(ctx);
	}
});

// ===========================================================================
// 3 — archiver puis ROUVRIR la fiche produit (seul tueur E2E de la mutation 2).
// ===========================================================================

test('rouvrir la fiche d’un article dont le compte a été archivé : le libellé reste affiché', async ({
	page,
}) => {
	await login(page);
	const suffix = uniqSuffix();
	const ctx = await authedApiContext(page);

	try {
		const account = await mkRevenueAccount(ctx, suffix);
		const productName = `Article rouvert ${suffix}`;

		const prodRes = await ctx.post('/api/v1/products', {
			data: {
				name: productName,
				unitPrice: '250.00',
				vatRate: '8.10',
				defaultRevenueAccountId: account.id,
			},
		});
		expect(prodRes.ok()).toBeTruthy();

		const archiveRes = await ctx.put(`/api/v1/accounts/${account.id}/archive`, {
			data: { version: account.version },
		});
		expect(archiveRes.ok()).toBeTruthy();

		// --- Le geste que ce scénario est SEUL à exercer : rouvrir la fiche.
		//
		// `fetchAccounts(true)` du catalogue est un appel DISTINCT de celui
		// d'`InvoiceForm`. Sans le flag, le compte archivé n'est plus résoluble
		// et le champ PARAÎT VIDE — et D-B2 garantit qu'aucun marqueur ne
		// viendrait le nuancer sur cette fiche.
		await page.goto('/products');
		await page.click(`tr:has-text("${productName}") button[aria-label*="odifier"], tr:has-text("${productName}") >> text=Modifier`);

		await expect(
			page.locator('#form-revenue-account'),
			'le libellé du compte archivé doit rester affiché, pas un champ vide'
		).toHaveValue(new RegExp(account.number));
	} finally {
		await disposeContextSafe(ctx);
	}
});

// ===========================================================================
// 4 — éditer puis « Nouveau produit » : pas d'héritage silencieux.
// ===========================================================================

test('éditer un article avec compte puis « Nouveau produit » : le sélecteur est vide', async ({
	page,
}) => {
	await login(page);
	const suffix = uniqSuffix();
	const ctx = await authedApiContext(page);

	try {
		const account = await mkRevenueAccount(ctx, suffix);
		const productName = `Article source ${suffix}`;

		const prodRes = await ctx.post('/api/v1/products', {
			data: {
				name: productName,
				unitPrice: '250.00',
				vatRate: '8.10',
				defaultRevenueAccountId: account.id,
			},
		});
		expect(prodRes.ok()).toBeTruthy();

		await page.goto('/products');
		await page.click(`tr:has-text("${productName}") button[aria-label*="odifier"], tr:has-text("${productName}") >> text=Modifier`);
		await expect(page.locator('#form-revenue-account')).toHaveValue(new RegExp(account.number));

		// Fermer, puis ouvrir la création — les champs sont des `$state` de
		// niveau PAGE, ils survivent à la fermeture du dialogue.
		await page.keyboard.press('Escape');
		await page.click('text=Nouveau produit');

		await expect(
			page.locator('#form-revenue-account'),
			'sans le reset d’openCreate(), le nouvel article HÉRITERAIT du compte du précédent — ' +
				'et D-B2 garantit que rien ne le signalerait'
		).toHaveValue('');
	} finally {
		await disposeContextSafe(ctx);
	}
});
