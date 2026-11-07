import { expect, test } from '@playwright/test';
import {
	seedTestState,
	clearAuthStorage,
	authedApiContext,
	disposeContextSafe,
} from './helpers/test-state';

/**
 * Tests E2E — corriger l'adresse de la société dans les réglages (Story 33-5, #909).
 *
 * Vitest simule l'API, les tests Rust construisent leur propre URL : seul ce
 * fichier vérifie qu'une adresse saisie à l'écran traverse la frontière HTTP.
 * L'état « pays déjà inconnu » ne s'écrit plus par aucune route : l'avis se teste
 * en Vitest, et le défaut lui-même dans `company_address_e2e.rs`.
 */

test.beforeAll(async () => {
	await seedTestState('with-company');
});

test.afterEach(async ({ page }) => {
	await clearAuthStorage(page);
});

type Address = {
	street: string;
	building: string;
	postalCode: string;
	city: string;
	country: string | null;
};
type Company = { address: string; addressStructured: Address; version: number };
type Ctx = import('@playwright/test').APIRequestContext;

async function login(
	page: import('@playwright/test').Page,
	username = 'admin',
	password = 'admin123',
) {
	await page.goto('/login');
	await page.fill('#username', username);
	await page.fill('#password', password);
	await page.click('button[type="submit"]');
	await expect(page).toHaveURL('/');
}

async function readCompany(ctx: Ctx): Promise<Company> {
	const res = await ctx.get('/api/v1/companies/current');
	expect(res.ok()).toBeTruthy();
	return ((await res.json()) as { company: Company }).company;
}

/** Les tests partagent une seule société : on rend l'adresse telle qu'on l'a trouvée. */
async function restoreAddress(ctx: Ctx, address: Address) {
	const { version } = await readCompany(ctx);
	const res = await ctx.put('/api/v1/companies/current/address', {
		data: { address, version },
	});
	expect(res.ok(), `restauration échouée: ${res.status()}`).toBeTruthy();
}

test('corriger l’adresse dans les réglages : les cinq champs traversent la frontière HTTP', async ({
	page,
}) => {
	await login(page);
	const ctx = await authedApiContext(page);
	const initial = (await readCompany(ctx)).addressStructured;
	try {
		await page.goto('/settings');
		await page.getByTestId('settings-company-address-edit').click();

		// Préremplissage depuis les composants, non depuis le texte d'affichage.
		await expect(page.getByTestId('settings-company-address-postal-code-input')).toHaveValue(
			initial.postalCode,
		);

		await page.getByTestId('settings-company-address-street-input').fill('Chemin des Essais');
		await page.getByTestId('settings-company-address-building-input').fill('7B');
		await page.getByTestId('settings-company-address-postal-code-input').fill('1800');
		await page.getByTestId('settings-company-address-city-input').fill('Vevey');
		await page.getByTestId('settings-company-address-country-input').fill('fr');
		await page.getByTestId('settings-company-address-save').click();

		await expect(page.getByTestId('settings-company-address')).toContainText('Vevey');
		await expect(page.getByTestId('settings-company-address-country-input')).toHaveCount(0);

		// L'assertion porte sur le corps relu : les champs ne se croisent pas.
		const relue = await readCompany(ctx);
		expect(relue.addressStructured).toEqual({
			street: 'Chemin des Essais',
			building: '7B',
			postalCode: '1800',
			city: 'Vevey',
			country: 'FR',
		});

		await page.reload();
		await expect(page.getByTestId('settings-company-address')).toContainText('Chemin des Essais');
	} finally {
		await restoreAddress(ctx, initial);
		await disposeContextSafe(ctx);
	}
});

test('un pays inconnu est refusé à l’écran, avec le code, et rien n’est écrit', async ({ page }) => {
	await login(page);
	const ctx = await authedApiContext(page);
	const avant = await readCompany(ctx);
	try {
		await page.goto('/settings');
		await page.getByTestId('settings-company-address-edit').click();
		await page.getByTestId('settings-company-address-country-input').fill('SU');
		await page.getByTestId('settings-company-address-save').click();

		await expect(page.getByTestId('settings-company-address-error')).toContainText('SU');
		// Le formulaire reste ouvert : la saisie n'est pas perdue.
		await expect(page.getByTestId('settings-company-address-country-input')).toHaveValue('SU');

		const apres = await readCompany(ctx);
		expect(apres.version).toBe(avant.version);
		expect(apres.addressStructured).toEqual(avant.addressStructured);
	} finally {
		await disposeContextSafe(ctx);
	}
});

test('écriture concurrente : le conflit de version est signalé, puis l’essai suivant passe', async ({
	page,
}) => {
	await login(page);
	const ctx = await authedApiContext(page);
	const initial = (await readCompany(ctx)).addressStructured;
	try {
		await page.goto('/settings');
		await page.getByTestId('settings-company-address-edit').click();

		const { version } = await readCompany(ctx);
		const concurrente = await ctx.put('/api/v1/companies/current/address', {
			data: { address: { ...initial, city: 'Nyon' }, version },
		});
		expect(concurrente.ok(), 'l’écriture concurrente doit réussir').toBeTruthy();

		await page.getByTestId('settings-company-address-city-input').fill('Morges');
		await page.getByTestId('settings-company-address-save').click();
		await expect(page.getByTestId('settings-company-address-error')).toContainText(
			'Conflit de version',
		);

		// Les données rechargées portent la nouvelle version : le nouvel essai aboutit.
		await page.getByTestId('settings-company-address-save').click();
		await expect(page.getByTestId('settings-company-address')).toContainText('Morges');
	} finally {
		await restoreAddress(ctx, initial);
		await disposeContextSafe(ctx);
	}
});

test('un Comptable lit l’adresse mais n’a pas le bouton Modifier', async ({ page }) => {
	await login(page);
	const ctx = await authedApiContext(page);
	const comptable = `comptable-${Date.now()}-${Math.floor(Math.random() * 1e6)}`;
	try {
		const cree = await ctx.post('/api/v1/users', {
			data: { username: comptable, password: 'MotDePasse12345', role: 'Comptable' },
		});
		expect(cree.ok(), `création du comptable échouée: ${cree.status()}`).toBeTruthy();
	} finally {
		await disposeContextSafe(ctx);
	}

	await clearAuthStorage(page);
	await login(page, comptable, 'MotDePasse12345');
	await page.goto('/settings');

	await expect(page.getByTestId('settings-company-address')).toBeVisible();
	await expect(page.getByTestId('settings-company-address-edit')).toHaveCount(0);
});
