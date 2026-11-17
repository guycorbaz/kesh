import { expect, test } from '@playwright/test';
import {
	seedTestState,
	clearAuthStorage,
	authedApiContext,
	disposeContextSafe,
} from './helpers/test-state';

/**
 * Tests E2E — corriger le nom de la société dans les réglages (Story v0190-nom-societe, #1097).
 *
 * Vitest simule l'API, les tests Rust construisent leur propre URL : seul ce fichier vérifie
 * qu'un nom saisi à l'écran traverse la frontière HTTP. Le refus se joue par un caractère hors
 * du jeu de la QR-facture, non par 71 caractères : le `maxlength={70}` du champ tronque la
 * saisie, `fill()` compris (le refus de longueur est couvert en Vitest et en Rust).
 */

test.beforeAll(async () => {
	await seedTestState('with-company');
});

test.afterEach(async ({ page }) => {
	await clearAuthStorage(page);
});

type Company = { name: string; version: number };
type Ctx = import('@playwright/test').APIRequestContext;

async function login(page: import('@playwright/test').Page) {
	await page.goto('/login');
	await page.fill('#username', 'admin');
	await page.fill('#password', 'admin123');
	await page.click('button[type="submit"]');
	await expect(page).toHaveURL('/');
}

async function readCompany(ctx: Ctx): Promise<Company> {
	const res = await ctx.get('/api/v1/companies/current');
	expect(res.ok()).toBeTruthy();
	return ((await res.json()) as { company: Company }).company;
}

/** Les tests partagent une seule société : on rend le nom tel qu'on l'a trouvé. */
async function restoreName(ctx: Ctx, name: string) {
	const actuel = await readCompany(ctx);
	if (actuel.name === name) return;
	const res = await ctx.put('/api/v1/companies/current/name', {
		data: { name, version: actuel.version },
	});
	expect(res.ok(), `restauration échouée: ${res.status()}`).toBeTruthy();
}

test('corriger le nom dans les réglages : il traverse la frontière HTTP et survit au rechargement', async ({
	page,
}) => {
	await login(page);
	const ctx = await authedApiContext(page);
	const initial = (await readCompany(ctx)).name;
	try {
		await page.goto('/settings');
		await expect(page.getByTestId('settings-company-name')).toHaveText(initial);
		await page.getByTestId('settings-company-name-edit').click();
		await expect(page.getByTestId('settings-company-name-input')).toHaveValue(initial);

		await page.getByTestId('settings-company-name-input').fill('Démo Corrigée & Fils SA');
		await page.getByTestId('settings-company-name-save').click();

		await expect(page.getByTestId('settings-company-name')).toHaveText('Démo Corrigée & Fils SA');
		await expect(page.getByTestId('settings-company-name-input')).toHaveCount(0);
		expect((await readCompany(ctx)).name).toBe('Démo Corrigée & Fils SA');

		await page.reload();
		await expect(page.getByTestId('settings-company-name')).toHaveText('Démo Corrigée & Fils SA');
	} finally {
		await restoreName(ctx, initial);
		await disposeContextSafe(ctx);
	}
});

test('un caractère que la QR-facture refuse est refusé à l’écran, nommé, et rien n’est écrit', async ({
	page,
}) => {
	await login(page);
	const ctx = await authedApiContext(page);
	const avant = await readCompany(ctx);
	try {
		await page.goto('/settings');
		await page.getByTestId('settings-company-name-edit').click();
		await page.getByTestId('settings-company-name-input').fill('Démo 张 SA');
		await page.getByTestId('settings-company-name-save').click();

		const erreur = page.getByTestId('settings-company-name-error');
		await expect(erreur).toContainText('张');
		await expect(erreur).toContainText('U+5F20');
		// Le formulaire reste ouvert : la saisie n'est pas perdue.
		await expect(page.getByTestId('settings-company-name-input')).toHaveValue('Démo 张 SA');

		const apres = await readCompany(ctx);
		expect(apres.version).toBe(avant.version);
		expect(apres.name).toBe(avant.name);
	} finally {
		await restoreName(ctx, avant.name);
		await disposeContextSafe(ctx);
	}
});
