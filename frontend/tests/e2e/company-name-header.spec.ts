import { expect, test, type Page, type APIRequestContext } from '@playwright/test';
import {
	seedTestState,
	clearAuthStorage,
	authedApiContext,
	disposeContextSafe,
} from './helpers/test-state';

/**
 * Tests E2E — le nom de la comptabilité ouverte, sous le logo et dans le titre de
 * l'onglet (#1582). Vitest simule l'API : seul le navigateur lit un `<title>` réel,
 * observe un `MutationObserver` réel et mesure la coupe d'un nom long.
 *
 * Les tests partagent une société : tout nom modifié est rendu en `finally`.
 */

test.afterEach(async ({ page }) => {
	await page.unrouteAll({ behavior: 'ignoreErrors' });
	await clearAuthStorage(page);
});

type Company = { name: string; version: number };

async function login(page: Page, user = 'admin', pass = 'admin123') {
	await page.goto('/login');
	await page.fill('#username', user);
	await page.fill('#password', pass);
	await page.click('button[type="submit"]');
}

async function readCompany(ctx: APIRequestContext): Promise<Company> {
	const res = await ctx.get('/api/v1/companies/current');
	expect(res.ok()).toBeTruthy();
	return ((await res.json()) as { company: Company }).company;
}

async function renameCompany(ctx: APIRequestContext, name: string) {
	const actuel = await readCompany(ctx);
	if (actuel.name === name) return;
	const res = await ctx.put('/api/v1/companies/current/name', {
		data: { name, version: actuel.version },
	});
	expect(res.ok(), `renommage refusé: ${res.status()}`).toBeTruthy();
}

test.describe('nom de la comptabilité — société ordinaire', () => {
	test.beforeAll(async () => {
		await seedTestState('with-company');
	});

	test('AC1/AC3 : nom sous le logo et dans le titre, une seule fois, à chaque navigation', async ({
		page,
	}) => {
		await login(page);
		await expect(page).toHaveURL('/');
		const ctx = await authedApiContext(page);
		try {
			const nom = (await readCompany(ctx)).name;
			await expect(page.getByTestId('company-name')).toHaveText(nom);
			await expect(page).toHaveTitle(new RegExp(`^Kesh — ${escapeRe(nom)}( — .+)?$`));
			for (const [href, page_] of [
				['/contacts', 'Contacts'],
				['/settings', 'Paramètres'],
			]) {
				await page.goto(href);
				await expect(page.getByTestId('company-name')).toHaveText(nom);
				await expect(page).toHaveTitle(
					new RegExp(`^Kesh — ${escapeRe(nom)} — .*${page_}.*$`, 'i'),
				);
				const titre = await page.title();
				expect(titre.split(nom).length - 1, `nom une seule fois dans « ${titre} »`).toBe(1);
				expect(titre.match(/Kesh/g)?.length, `« Kesh » une fois dans « ${titre} »`).toBe(1);
			}
		} finally {
			await disposeContextSafe(ctx);
		}
	});

	test('AC6 : corrigé dans Paramètres, l’en-tête et le titre suivent sans rechargement', async ({
		page,
	}) => {
		await login(page);
		await expect(page).toHaveURL('/');
		const ctx = await authedApiContext(page);
		const initial = (await readCompany(ctx)).name;
		const neuf = 'Suivi Corrigé SA';
		try {
			await page.goto('/settings');
			await expect(page.getByTestId('company-name')).toHaveText(initial);
			await page.getByTestId('settings-company-name-edit').click();
			await page.getByTestId('settings-company-name-input').fill(neuf);
			await page.getByTestId('settings-company-name-save').click();
			await expect(page.getByTestId('company-name')).toHaveText(neuf);
			await expect(page).toHaveTitle(new RegExp(`^Kesh — ${escapeRe(neuf)} — `));
			await page.reload();
			await expect(page.getByTestId('company-name')).toHaveText(neuf);
			await expect(page).toHaveTitle(new RegExp(`^Kesh — ${escapeRe(neuf)}`));
		} finally {
			await renameCompany(ctx, initial);
			await disposeContextSafe(ctx);
		}
	});

	test('AC4 : un nom de 70 caractères est coupé, titré, l’en-tête garde ses 56 px', async ({
		page,
	}) => {
		await page.setViewportSize({ width: 1280, height: 800 });
		await login(page);
		await expect(page).toHaveURL('/');
		const ctx = await authedApiContext(page);
		const initial = (await readCompany(ctx)).name;
		try {
			const nom = page.getByTestId('company-name');
			// sonde positive : un nom court n'est pas coupé
			await expect(nom).toBeVisible();
			expect(await nom.evaluate((e) => e.scrollWidth > e.clientWidth)).toBe(false);
			const asideAvant = await page.locator('aside').boundingBox();

			const long = 'Établissements Dupont-Müller & Fils, Société Anonyme de Construction'.slice(0, 70);
			expect([...long].length).toBe(70);
			await renameCompany(ctx, long);
			await page.reload();
			await expect(nom).toHaveText(long);
			expect(await nom.evaluate((e) => e.scrollWidth > e.clientWidth), 'coupé').toBe(true);
			await expect(nom).toHaveAttribute('title', long);
			const header = await page.locator('header').first().boundingBox();
			expect(header?.height).toBe(56);
			const asideApres = await page.locator('aside').boundingBox();
			expect(asideApres?.width).toBe(asideAvant?.width);
			// tient à 1280 px sans retour à la ligne : le menu profil reste dans la fenêtre
			const menu = await page.locator('header button').last().boundingBox();
			expect(menu!.x + menu!.width).toBeLessThanOrEqual(1280);
		} finally {
			await renameCompany(ctx, initial);
			await disposeContextSafe(ctx);
		}
	});

	test('AC5 : replier ou déplier les groupes ne bouge pas le nom', async ({ page }) => {
		await login(page);
		await expect(page).toHaveURL('/');
		const nom = page.getByTestId('company-name');
		await expect(nom).toBeVisible();
		const ref = await nom.boundingBox();
		const texte = await nom.textContent();
		for (const cle of ['quotidien', 'mensuel', 'administration']) {
			const summary = page.getByTestId(`nav-group-${cle}`).locator('summary');
			await summary.click();
			expect(await nom.boundingBox(), `après clic ${cle}`).toEqual(ref);
			await summary.click();
			expect(await nom.boundingBox(), `après second clic ${cle}`).toEqual(ref);
			expect(await nom.textContent()).toBe(texte);
		}
	});

	test('AC9 : le lien du logo porte « Kesh <nom> » et l’axe (avec contraste) reste vert', async ({
		page,
	}) => {
		const AxeBuilder = (await import('@axe-core/playwright')).default;
		await login(page);
		await expect(page).toHaveURL('/');
		await page.waitForLoadState('networkidle');
		await expect(page.getByRole('link', { name: /^Kesh .+/ })).toBeVisible();
		const results = await new AxeBuilder({ page }).analyze();
		expect(results.violations).toEqual([]);
	});

	test('AC10 : lecture de la société en échec (500) : ni nom, ni toast, ni redirection', async ({
		page,
	}) => {
		await page.route('**/api/v1/companies/current', (r) => r.fulfill({ status: 500, body: '{}' }));
		await login(page);
		await expect(page).toHaveURL('/');
		// sonde positive : le lien du logo est là
		await expect(page.getByRole('link', { name: 'Kesh', exact: true })).toBeVisible();
		await expect(page.getByTestId('company-name')).toHaveCount(0);
		await expect(page.locator('[data-sonner-toast]')).toHaveCount(0);
		expect(await page.title()).not.toContain(' — ');
	});
});

test.describe('nom de la comptabilité — mode démonstration', () => {
	test('AC7 : le bandeau et le nom de la société de démonstration sont visibles', async ({
		page,
	}) => {
		await seedTestState('fresh');
		await login(page, 'changeme', 'changeme');
		await expect(page).toHaveURL(/\/onboarding/);
		await page.getByTestId('onboarding-language-FR').click();
		await page.getByTestId('onboarding-mode-guided').click();
		await page.getByTestId('onboarding-path-demo').click();
		await expect(page).toHaveURL(/\/$/);
		const ctx = await authedApiContext(page);
		try {
			const nom = (await readCompany(ctx)).name;
			// Le bandeau n'a pas de data-testid (DemoBanner reste inchangé) : sa classe de fond.
			await expect(page.locator('div.bg-yellow-100[role="status"]')).toBeVisible();
			await expect(page.getByTestId('company-name')).toHaveText(nom);
		} finally {
			await disposeContextSafe(ctx);
		}
	});
});

function escapeRe(s: string): string {
	return s.replace(/[.*+?^${}()|[\]\\]/g, '\\$&');
}
