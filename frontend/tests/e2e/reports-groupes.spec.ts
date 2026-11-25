/**
 * Story 46-2 — le bilan et le compte de résultat par groupes, de bout en bout
 * (AC-9, AC-10) : écran groupé par défaut, niveau 1, ouverture d'un groupe à la
 * main, export CSV dont l'URL porte les mêmes paramètres, vue retrouvée au
 * retour sur la page, hiérarchie masquée qui rend l'écran d'avant.
 *
 * ⚠️ Ce que seul un E2E vérifie : que les paramètres de groupement traversent la
 * frontière HTTP, dans la requête du rapport ET dans celle de l'export, et que
 * la vue mémorisée part avec la PREMIÈRE requête au retour.
 *
 * Plan : le preset `with-company` (comptes plats 1000 à 4000), plus un arbre
 * créé ici — `19` > `190` > `1900`, `1910` —, et une écriture qui le mouvemente.
 * L'export groupé lui-même (le rendu du fichier) est la 46-3 : ici, l'URL.
 *
 * Captures : `KESH_CAPTURES_DIR` (thème clair), une par état touché.
 */

import { readFileSync } from 'node:fs';
import { expect, test, type Page } from '@playwright/test';
import { seedTestState, clearAuthStorage } from './helpers/test-state';
import { login, preparerArbre } from './helpers/reports-groupes';

test.beforeAll(async () => {
	await seedTestState('with-company');
});

test.afterEach(async ({ page }) => {
	await clearAuthStorage(page);
});

const CAPTURES = process.env.KESH_CAPTURES_DIR ?? 'test-results';

async function capture(page: Page, nom: string): Promise<void> {
	await page.screenshot({ path: `${CAPTURES}/46-2-${nom}.png`, fullPage: true });
}

/**
 * Story 46-8 (revue P1, LOW-6) — exporte en CSV l'onglet affiché, hiérarchie
 * masquée : l'export de l'API étant groupé sans paramètre, l'écran à plat
 * demande `detailLevel=flat`, et le fichier n'a pas les colonnes groupées.
 */
async function exporterAPlat(page: Page, route: string, compte: string): Promise<void> {
	const exportReq = page.waitForRequest((r) =>
		new RegExp(`reports/${route}/export\\?`).test(r.url()),
	);
	const download = page.waitForEvent('download');
	await page.getByRole('button', { name: /export csv/i }).click();
	const url = new URL((await exportReq).url());
	expect(url.searchParams.get('detailLevel')).toBe('flat');
	expect(url.searchParams.has('subtotalPosition')).toBe(false);
	const chemin = await (await download).path();
	expect(chemin, 'fichier téléchargé').toBeTruthy();
	const texte = readFileSync(chemin!, 'utf8').replace(/^\uFEFF/, '');
	expect(texte.split(/\r?\n/)[0]).not.toContain('Niveau;TypeLigne');
	expect(texte).toContain(compte);
}

/** Clique « Générer » et attend la requête du bilan ; rend son URL. */
async function generer(page: Page): Promise<URL> {
	const req = page.waitForRequest((r) => /\/api\/v1\/reports\/balance-sheet\?/.test(r.url()));
	await page.getByRole('button', { name: /générer/i }).click();
	return new URL((await req).url());
}

test('bilan par groupes : défaut, niveau 1, ouverture à la main, export, retour, à plat (Story 46-2)', async ({
	page,
}) => {
	await login(page);
	const g19 = await preparerArbre(page);

	await page.goto('/reports');
	await page.waitForLoadState('networkidle');
	await expect(page.getByTestId('grouping-controls')).toBeVisible();

	// 1. Première visite : groupé, tous les comptes, sous-totaux sur le titre (AC-1).
	let url = await generer(page);
	expect(url.searchParams.get('detailLevel')).toBe('all');
	expect(url.searchParams.get('subtotalPosition')).toBe('header');
	expect(url.searchParams.has('open')).toBe(false);
	await expect(page.getByTestId('grouped-group-19')).toContainText('1’250.00');
	await expect(page.getByTestId('grouped-group-190')).toContainText('Liquidités du test');
	await expect(page.getByTestId('grouped-account-1900')).toContainText('1’000.00');
	await expect(page.getByTestId('group-toggle-19')).toHaveAttribute('aria-expanded', 'true');
	await expect(page.getByText('Total actifs').locator('..')).toContainText('1’250.00');
	await capture(page, '1-defaut-tous-les-comptes');

	// 2. Sous-totaux en pied.
	let req = page.waitForRequest((r) => /reports\/balance-sheet\?/.test(r.url()));
	await page.getByTestId('grouping-position').selectOption('footer');
	expect(new URL((await req).url()).searchParams.get('subtotalPosition')).toBe('footer');
	await expect(page.getByTestId('grouped-subtotal-19')).toContainText('Total Actifs du test');
	await expect(page.getByTestId('grouped-subtotal-19')).toContainText('1’250.00');
	await capture(page, '2-sous-totaux-en-pied');
	req = page.waitForRequest((r) => /reports\/balance-sheet\?/.test(r.url()));
	await page.getByTestId('grouping-position').selectOption('header');
	await req;

	// 3. Niveau 1 : 19 fermé sur son sous-total, sa descendance cachée.
	req = page.waitForRequest((r) => /reports\/balance-sheet\?/.test(r.url()));
	await page.getByTestId('grouping-level').selectOption('1');
	expect(new URL((await req).url()).searchParams.get('detailLevel')).toBe('1');
	await expect(page.getByTestId('grouped-account-1900')).toHaveCount(0);
	await expect(page.getByTestId('group-toggle-19')).toHaveAttribute('aria-expanded', 'false');
	await expect(page.getByTestId('grouped-group-19')).toContainText('1’250.00');
	await capture(page, '3-niveau-1');

	// 4. Ouverture de 19 à la main : 190 paraît, fermé par le niveau.
	req = page.waitForRequest((r) => /reports\/balance-sheet\?/.test(r.url()));
	await page.getByTestId('group-toggle-19').click();
	url = new URL((await req).url());
	expect(url.searchParams.get('open')).toBe(String(g19));
	await expect(page.getByTestId('group-toggle-190')).toHaveAttribute('aria-expanded', 'false');
	await expect(page.getByTestId('grouped-group-190')).toContainText('1’250.00');
	await expect(page.getByTestId('grouped-account-1900')).toHaveCount(0);
	await capture(page, '4-groupe-ouvert-a-la-main');

	// 5. Export CSV : les mêmes paramètres que l'écran (AC-6).
	const exportReq = page.waitForRequest((r) => /reports\/balance-sheet\/export\?/.test(r.url()));
	const download = page.waitForEvent('download');
	await page.getByRole('button', { name: /export csv/i }).click();
	const exportUrl = new URL((await exportReq).url());
	expect(exportUrl.searchParams.get('format')).toBe('csv');
	expect(exportUrl.searchParams.get('detailLevel')).toBe('1');
	expect(exportUrl.searchParams.get('subtotalPosition')).toBe('header');
	expect(exportUrl.searchParams.get('open')).toBe(String(g19));
	expect(exportUrl.searchParams.has('closed')).toBe(false);
	await download;

	// 6. Quitter la page, revenir : même niveau, même groupe ouvert, dès la 1re requête (AC-10).
	await page.goto('/');
	await page.goto('/reports');
	await page.waitForLoadState('networkidle');
	await expect(page.getByTestId('grouping-level')).toHaveValue('1');
	url = await generer(page);
	expect(url.searchParams.get('detailLevel')).toBe('1');
	expect(url.searchParams.get('open')).toBe(String(g19));
	await expect(page.getByTestId('group-toggle-19')).toHaveAttribute('aria-expanded', 'true');
	await expect(page.getByTestId('grouped-group-190')).toBeVisible();
	await capture(page, '5-vue-retrouvee');

	// 7. Hiérarchie masquée : l'écran d'avant, requête sans paramètre (C-46-2-2).
	req = page.waitForRequest((r) => /reports\/balance-sheet\?/.test(r.url()));
	await page.getByTestId('grouping-shown').uncheck();
	url = new URL((await req).url());
	expect(url.searchParams.has('detailLevel')).toBe(false);
	await expect(page.getByTestId('grouped-group-19')).toHaveCount(0);
	await expect(page.getByRole('link', { name: '1900' })).toBeVisible();
	await expect(page.getByRole('link', { name: '1910' })).toBeVisible();
	await expect(page.getByTestId('grouping-level')).toBeDisabled();
	await capture(page, '6-hierarchie-masquee');
	await exporterAPlat(page, 'balance-sheet', '1900');

	// La remise en place laisse la vue par défaut pour les specs suivantes.
	req = page.waitForRequest((r) => /reports\/balance-sheet\?/.test(r.url()));
	await page.getByTestId('grouping-shown').check();
	await req;
	await page.getByTestId('grouping-open-all').click();

	// 8. Compte de résultat : groupé par défaut, son propre réglage.
	await page.locator('#reports-tab-income-statement').click();
	const isReq = page.waitForRequest((r) => /reports\/income-statement\?/.test(r.url()));
	await page.getByRole('button', { name: /générer/i }).click();
	expect(new URL((await isReq).url()).searchParams.get('detailLevel')).toBe('all');
	await expect(page.getByTestId('grouped-account-3000')).toContainText('1’250.00');
	await expect(page.getByTestId('grouping-level')).toBeDisabled(); // plan sans groupe ici
	await capture(page, '7-compte-de-resultat');

	// 9. Compte de résultat, hiérarchie masquée : l'export demande `detailLevel=flat`.
	req = page.waitForRequest((r) => /reports\/income-statement\?/.test(r.url()));
	await page.getByTestId('grouping-shown').uncheck();
	expect(new URL((await req).url()).searchParams.has('detailLevel')).toBe(false);
	await exporterAPlat(page, 'income-statement', '3000');
	req = page.waitForRequest((r) => /reports\/income-statement\?/.test(r.url()));
	await page.getByTestId('grouping-shown').check();
	await req;
});
