import { expect, test } from '@playwright/test';
import { seedTestState, clearAuthStorage, authedApiContext, disposeContextSafe } from './helpers/test-state';
import { todayInZurich } from './helpers/api-fixtures';

/**
 * Story 48-13 (#1233, AC3) — le client d'un brouillon ouvert en modification se charge
 * après l'ouverture (`GET /api/v1/contacts/{id}`). Si l'utilisateur choisit un autre
 * client avant la réponse, celle-ci, arrivée tard, ne remplace plus le client choisi :
 * le champ le montre, l'enregistrement part avec lui, et le brouillon rechargé le porte.
 *
 * Le retard est posé par `page.route` sur le seul client d'origine. Sélecteurs par rôle
 * à motif ou par `data-testid` (garde `e2e-selecteurs-traduits.test.ts`).
 */

test.beforeAll(async () => {
	await seedTestState('with-company');
});

test.afterEach(async ({ page }) => {
	await clearAuthStorage(page);
});

function uniq(prefix: string): string {
	return `${prefix} ${Date.now()}-${Math.floor(Math.random() * 1e6)}`;
}

test('un client choisi avant la fin du chargement du client d’origine n’est pas remplacé (#1233)', async ({
	page,
}) => {
	await page.goto('/login');
	await page.fill('#username', 'admin');
	await page.fill('#password', 'admin123');
	await page.click('button[type="submit"]');
	await expect(page).toHaveURL('/');

	const origine = uniq('Client origine 4813');
	const choisi = uniq('Client choisi 4813');
	const ctx = await authedApiContext(page);
	let idOrigine = 0;
	let idChoisi = 0;
	let idFacture = 0;
	try {
		const creer = async (name: string) => {
			const res = await ctx.post('/api/v1/contacts', {
				data: { contactType: 'Entreprise', name, isClient: true, isSupplier: false },
			});
			expect(res.ok(), `création du contact échouée: ${res.status()}`).toBeTruthy();
			return ((await res.json()) as { id: number }).id;
		};
		idOrigine = await creer(origine);
		idChoisi = await creer(choisi);
		const res = await ctx.post('/api/v1/invoices', {
			data: {
				contactId: idOrigine,
				date: todayInZurich(),
				lines: [{ description: 'Conseil 4813', quantity: '1', unitPrice: '100.00', vatRate: '8.10' }],
			},
		});
		expect(res.ok(), `création du brouillon échouée: ${res.status()}`).toBeTruthy();
		idFacture = ((await res.json()) as { id: number }).id;
	} finally {
		await disposeContextSafe(ctx);
	}

	// Le client d'origine ne répond qu'à la demande du test.
	let liberer!: () => void;
	const tenu = new Promise<void>((r) => (liberer = r));
	const cheminOrigine = `/api/v1/contacts/${idOrigine}`;
	const versOrigine = (url: URL) => url.pathname === cheminOrigine;
	await page.route(
		versOrigine,
		async (route) => {
			await tenu;
			await route.continue();
		},
	);

	await page.goto(`/invoices/${idFacture}/edit`);
	const champ = page.getByRole('combobox', { name: /Rechercher un contact/ });
	await expect(champ).toBeVisible();
	await expect(champ).toHaveValue('');

	await champ.click();
	await champ.fill(choisi);
	await page.getByRole('option', { name: new RegExp(choisi) }).first().click();
	await expect(champ).toHaveValue(choisi);

	const reponse = page.waitForResponse((r) => new URL(r.url()).pathname === cheminOrigine);
	liberer();
	await reponse;
	// La réponse a été traitée par la page (sa promesse s'est résolue) : un tour de tâches.
	await page.evaluate(() => new Promise((r) => setTimeout(r, 100)));
	await expect(champ).toHaveValue(choisi);

	await page.getByTestId('create-invoice-button').click();
	await expect(page).toHaveURL(new RegExp(`/invoices/${idFacture}$`));

	const ctx2 = await authedApiContext(page);
	try {
		const res = await ctx2.get(`/api/v1/invoices/${idFacture}`);
		expect(res.ok()).toBeTruthy();
		expect(((await res.json()) as { contactId: number }).contactId).toBe(idChoisi);
	} finally {
		await disposeContextSafe(ctx2);
	}

	// Rechargement dur : le brouillon porte le client choisi.
	await page.unroute(versOrigine);
	await page.goto(`/invoices/${idFacture}/edit`);
	await page.reload();
	await expect(page.getByRole('combobox', { name: /Rechercher un contact/ })).toHaveValue(choisi);
});
