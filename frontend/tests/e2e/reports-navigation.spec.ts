/**
 * Story 55-1a — la navigation des Rapports : l'état dans l'URL (Retour, Suivant, lien partagé), le
 * sélecteur d'exercice (D15) et le chargement sans *Générer* (D16).
 *
 * ⚠️ Ce que seul un E2E constate : qu'un `popstate` réel (bouton Retour du navigateur) ramène l'écran à
 * l'adresse — Vitest simule `page.url`, il ne voit pas SvelteKit ni l'historique du navigateur.
 *
 * Captures (thème clair) dans `KESH_CAPTURES_DIR` quand il est posé.
 */

import { expect, test, type Page } from '@playwright/test';
import AxeBuilder from '@axe-core/playwright';
import {
	seedTestState,
	clearAuthStorage,
	authedApiContext,
	disposeContextSafe,
} from './helpers/test-state';
import { createJournalEntryViaApi, todayInZurich } from './helpers/api-fixtures';
import { preparerArbre } from './helpers/reports-groupes';

const ANNEE = Number(todayInZurich().slice(0, 4));
const PRECEDENTE = ANNEE - 1;

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

/** Les bornes de l'exercice par défaut de la société (le plus récent ; le preset le fait large). */
async function bornesParDefaut(page: Page): Promise<{ startDate: string; endDate: string }> {
	const ctx = await authedApiContext(page);
	try {
		const fys: Array<{ startDate: string; endDate: string }> = await (await ctx.get('/api/v1/fiscal-years')).json();
		return fys.sort((x, y) => y.startDate.localeCompare(x.startDate))[0];
	} finally {
		await disposeContextSafe(ctx);
	}
}

/** Deux exercices civils, l'année précédente et celle du jour, sur une société sans exercice. */
async function deuxExercices(page: Page): Promise<void> {
	const ctx = await authedApiContext(page);
	try {
		for (const a of [PRECEDENTE, ANNEE]) {
			const res = await ctx.post('/api/v1/fiscal-years', {
				data: { name: `Exercice ${a}`, startDate: `${a}-01-01`, endDate: `${a}-12-31` },
			});
			expect(res.ok(), `exercice ${a} : ${res.status()}`).toBeTruthy();
		}
	} finally {
		await disposeContextSafe(ctx);
	}
}

/** Compte les requêtes de rapport, par rapport (`balance-sheet`, `general-ledger`…). */
function compter(page: Page): Record<string, number> {
	const n: Record<string, number> = {};
	page.on('request', (r) => {
		const m = /\/api\/v1\/reports\/([a-z-]+)\?/.exec(r.url());
		if (m) n[m[1]] = (n[m[1]] ?? 0) + 1;
	});
	return n;
}

async function capture(page: Page, nom: string): Promise<void> {
	if (!process.env.KESH_CAPTURES_DIR) return;
	await page.screenshot({ path: `${process.env.KESH_CAPTURES_DIR}/55-1a-${nom}.png`, fullPage: true });
}

const ongletActif = (page: Page, id: string) =>
	expect(page.locator(`#reports-tab-${id}`)).toHaveAttribute('aria-selected', 'true');

test('AC-1 — Retour après un lien de compte : le Bilan, sans requête ; Suivant : le grand livre, sans requête', async ({ page }) => {
	await login(page);
	await createJournalEntryViaApi(page, 'E2E 55-1a — lien de compte', '500.00');
	const fy = await bornesParDefaut(page);
	const n = compter(page);
	await page.goto('/reports');
	const lien = page.getByRole('tabpanel').getByRole('link').first();
	await expect(lien).toBeVisible({ timeout: 10000 });
	// D16 : aucun bouton Générer sur le Bilan (ceux qui restent : grand livre, comptes analytiques).
	await expect(page.locator('[data-testid="ledger-generate"], [data-testid="project-report-generate"]')).toHaveCount(0);
	await expect(page).toHaveURL(new RegExp(`tab=balance-sheet&fy=\\d+&from=${fy.startDate}&to=${fy.endDate}`));
	await capture(page, '1-bilan-avant-le-lien');
	await lien.click();
	await expect(page).toHaveURL(/tab=general-ledger&accountId=\d+&from=/);
	await expect(page.locator('[data-testid^="ledger-section-"]').first()).toBeVisible({ timeout: 10000 });
	await capture(page, '2-grand-livre-apres-le-lien');
	await page.goBack();
	await ongletActif(page, 'balance-sheet');
	await expect(page.getByRole('tabpanel').getByRole('link').first()).toBeVisible();
	await capture(page, '3-bilan-apres-retour');
	expect(n['balance-sheet']).toBe(1);
	await page.goForward();
	await ongletActif(page, 'general-ledger');
	await expect(page.locator('[data-testid^="ledger-section-"]').first()).toBeVisible();
	expect(n['general-ledger']).toBe(1);
});

test('AC-2, AC-22 — clic sur un onglet : une étape d’historique, chargé une fois ; Retour et Suivant sans requête', async ({ page }) => {
	await login(page);
	const n = compter(page);
	await page.goto('/reports');
	await expect(page.getByText(/aucune écriture|total actifs/i).first()).toBeVisible({ timeout: 10000 });
	const avant = await page.evaluate(() => history.length);
	await page.locator('#reports-tab-income-statement').click();
	await expect(page).toHaveURL(/tab=income-statement&fy=\d+&from=/);
	await expect(page.getByTestId('report-period-heading')).toBeVisible();
	expect(await page.evaluate(() => history.length)).toBe(avant + 1);
	await page.goBack();
	await ongletActif(page, 'balance-sheet');
	await page.goForward();
	await ongletActif(page, 'income-statement');
	expect(n['balance-sheet']).toBe(1);
	expect(n['income-statement']).toBe(1);
});

test('AC-3, AC-20 — dates changées puis quittées : une requête, l’adresse remplacée ; Entrée de même', async ({ page }) => {
	await login(page);
	const n = compter(page);
	await page.goto('/reports');
	await expect(page.getByTestId('report-period-heading')).toBeVisible({ timeout: 10000 });
	const avant = await page.evaluate(() => history.length);
	await page.getByTestId('report-to').fill(`${ANNEE}-06-30`);
	await page.getByTestId('report-from').focus(); // « Au » → « Du » : on reste dans le groupe
	expect(n['balance-sheet']).toBe(1);
	await page.locator('h1').click(); // sortie du groupe
	await expect(page).toHaveURL(new RegExp(`to=${ANNEE}-06-30`));
	await expect(page.getByTestId('report-period-heading')).toContainText(`au 30.06.${ANNEE}`);
	expect(await page.evaluate(() => history.length)).toBe(avant);
	expect(n['balance-sheet']).toBe(2);
	await page.getByTestId('report-to').fill(`${ANNEE}-03-31`);
	await page.getByTestId('report-to').press('Enter');
	await expect(page.getByTestId('report-period-heading')).toContainText(`au 31.03.${ANNEE}`);
	expect(n['balance-sheet']).toBe(3);
});

test('AC-4 — lien partagé dans un nouveau contexte (cookies de la session) et rechargement : même rapport, contrôles remplis', async ({ page, browser }) => {
	await login(page);
	const adresse = `/reports?tab=general-ledger&from=${ANNEE}-01-01&to=${ANNEE}-12-31&includeZero=1`;
	await page.goto(adresse);
	await expect(page.getByTestId('ledger-from')).toHaveValue(`${ANNEE}-01-01`);
	await expect(page.getByTestId('ledger-include-zero')).toBeChecked();
	await page.reload();
	await ongletActif(page, 'general-ledger');
	await expect(page.getByTestId('ledger-include-zero')).toBeChecked();
	const etat = await page.context().storageState();
	const autre = await browser.newContext({ storageState: { cookies: etat.cookies, origins: [] } });
	try {
		const p2 = await autre.newPage();
		const classique = `/reports?tab=income-statement&from=${ANNEE}-02-01&to=${ANNEE}-03-31`;
		await p2.goto(classique);
		await ongletActif(p2, 'income-statement');
		await expect(p2.getByTestId('report-from')).toHaveValue(`${ANNEE}-02-01`);
		await expect(p2.getByTestId('report-period-heading')).toContainText(`Du 01.02.${ANNEE} au 31.03.${ANNEE}`);
		await expect(p2).toHaveURL(new RegExp(`tab=income-statement&fy=\\d+&from=${ANNEE}-02-01&to=${ANNEE}-03-31`));
		await capture(p2, '4-lien-colle-nouveau-contexte');
	} finally {
		await autre.close();
	}
});

test('AC-5 — valeurs invalides : ignorées, adresse normalisée sans entrée, aucune erreur', async ({ page }) => {
	await login(page);
	const fy = await bornesParDefaut(page);
	await page.goto('/reports?tab=xyz&fy=abc');
	await ongletActif(page, 'balance-sheet');
	await expect(page).toHaveURL(new RegExp(`tab=balance-sheet&fy=\\d+&from=${fy.startDate}&to=${fy.endDate}$`));
	await page.goto(`/reports?tab=general-ledger&from=${ANNEE}-02-30&to=${ANNEE}-12-31&accountId=-3`);
	await ongletActif(page, 'general-ledger');
	await expect(page).toHaveURL(new RegExp(`tab=general-ledger&to=${ANNEE}-12-31$`));
	await capture(page, '5-adresse-normalisee');
	await expect(page.getByRole('alert')).toHaveCount(0);
});

test('AC-10 — un clic d’onglet garde le défilement ; axe à zéro violation', async ({ page }) => {
	await page.setViewportSize({ width: 1000, height: 420 });
	await login(page);
	await page.goto('/reports');
	await expect(page.getByTestId('report-period-heading')).toBeVisible({ timeout: 10000 });
	await page.evaluate(() => window.scrollTo(0, 120));
	const y = await page.evaluate(() => window.scrollY);
	expect(y).toBeGreaterThan(0);
	await page.locator('#reports-tab-trial-balance').click();
	await ongletActif(page, 'trial-balance');
	expect(await page.evaluate(() => window.scrollY)).toBe(y);
	await page.setViewportSize({ width: 1280, height: 900 });
	const axe = await new AxeBuilder({ page }).withTags(['wcag2a', 'wcag2aa']).analyze();
	expect(axe.violations).toEqual([]);
});

test('AC-24 — rechargement : le rapport reste, grisé, aria-busy', async ({ page }) => {
	await login(page);
	await page.goto('/reports');
	await expect(page.getByTestId('report-period-heading')).toBeVisible({ timeout: 10000 });
	let liberer!: () => void;
	const attente = new Promise<void>((res) => (liberer = res));
	await page.route(/\/api\/v1\/reports\/balance-sheet\?/, async (route) => {
		await attente;
		await route.continue();
	});
	await page.getByTestId('report-to').fill(`${ANNEE}-09-30`);
	await page.getByTestId('report-to').press('Enter');
	await expect(page.getByRole('tabpanel')).toHaveAttribute('aria-busy', 'true');
	await expect(page.getByTestId('report-content')).toHaveClass(/opacity-60/);
	await capture(page, '11-rechargement-grise');
	liberer();
	await expect(page.getByRole('tabpanel')).toHaveAttribute('aria-busy', 'false');
	await expect(page.getByTestId('report-period-heading')).toContainText(`au 30.09.${ANNEE}`);
	await page.unroute(/\/api\/v1\/reports\/balance-sheet\?/);
});

// ----- Story 55-1b (D4) — la présentation des rapports groupés dans l'URL -------------------------

async function capture1b(page: Page, nom: string): Promise<void> {
	if (!process.env.KESH_CAPTURES_DIR) return;
	await page.screenshot({ path: `${process.env.KESH_CAPTURES_DIR}/55-1b-${nom}.png`, fullPage: true });
}

/** Un changement de présentation, et l'adresse que sa réponse écrit. */
async function presentation(page: Page, geste: () => Promise<void>, attendu: RegExp): Promise<void> {
	const req = page.waitForRequest((r) => /\/api\/v1\/reports\/balance-sheet\?/.test(r.url()));
	await geste();
	await req;
	await expect(page).toHaveURL(attendu);
}

test('55-1b AC-1, AC-4 — la présentation dans l’adresse : lien dans un nouveau contexte, rechargement, Retour', async ({
	page,
	browser,
}) => {
	// Plan groupé neuf (19 > 190 > 1900, 1910) : la base est remise au preset, l'arbre créé une fois.
	await seedTestState('with-company');
	await login(page);
	const g19 = await preparerArbre(page);
	await page.goto('/reports');
	await expect(page.getByTestId('grouped-group-19')).toBeVisible({ timeout: 10000 });
	// La présentation n'entre dans l'adresse que par la réponse affichée.
	await presentation(page, () => page.getByTestId('grouping-level').selectOption('1'), /view=grouped&level=1&subtotals=header$/);
	await presentation(page, () => page.getByTestId('group-toggle-19').click(), new RegExp(`level=1&subtotals=header&open=${g19}$`));
	await presentation(
		page,
		() => page.getByTestId('grouping-position').selectOption('footer'),
		new RegExp(`level=1&subtotals=footer&open=${g19}$`),
	);
	await capture1b(page, '1-bilan-groupe-avant-lien');
	const adresse = page.url();

	// AC-1 — nouveau contexte, cookies de la session, mémoire du navigateur vide : la présentation de l'adresse.
	const etat = await page.context().storageState();
	const autre = await browser.newContext({ storageState: { cookies: etat.cookies, origins: [] } });
	try {
		const p2 = await autre.newPage();
		await p2.goto(adresse);
		await expect(p2.getByTestId('grouping-level')).toHaveValue('1');
		await expect(p2.getByTestId('grouping-position')).toHaveValue('footer');
		await expect(p2.getByTestId('group-toggle-19')).toHaveAttribute('aria-expanded', 'true');
		await expect(p2.getByTestId('grouped-group-190')).toBeVisible();
		await expect(p2).toHaveURL(adresse);
		await capture1b(p2, '3-lien-colle-nouveau-contexte');
		// Rechargement avec une AUTRE vue en mémoire : l'adresse l'emporte (AC-2).
		await p2.evaluate(() => {
			for (const k of Object.keys(localStorage)) {
				if (k.startsWith('kesh-report-grouping:') && k.endsWith(':balance-sheet')) {
					localStorage.setItem(k, JSON.stringify({ shown: true, detailLevel: 2, subtotalPosition: 'header', open: [], closed: [] }));
				}
			}
		});
		await p2.reload();
		await expect(p2.getByTestId('grouping-level')).toHaveValue('1');
		await expect(p2.getByTestId('grouping-position')).toHaveValue('footer');
		await expect(p2).toHaveURL(adresse);
	} finally {
		await autre.close();
	}

	// AC-4 — Bilan au niveau 1 ; Compte de résultat ; Bilan ; niveau 2 ; Retour deux fois : le niveau 1.
	await page.locator('#reports-tab-income-statement').click();
	await ongletActif(page, 'income-statement');
	await page.locator('#reports-tab-balance-sheet').click();
	await expect(page).toHaveURL(adresse);
	await presentation(page, () => page.getByTestId('grouping-level').selectOption('2'), /view=grouped&level=2&subtotals=footer$/);
	await page.goBack();
	await ongletActif(page, 'income-statement');
	await page.goBack();
	await ongletActif(page, 'balance-sheet');
	await expect(page).toHaveURL(adresse);
	await expect(page.getByTestId('grouping-level')).toHaveValue('1');
	await expect(page.getByTestId('group-toggle-19')).toHaveAttribute('aria-expanded', 'true');
	await expect(page.getByRole('tabpanel')).toHaveAttribute('aria-busy', 'false');
	await capture1b(page, '2-apres-retour');
});

test('D15 — le sélecteur : libellé, exercice choisi → dates, « Exercice entier », « Plusieurs », « Hors exercice »', async ({ page }) => {
	// Deux exercices civils : le preset ne porte qu'un exercice très large. Reseed d'une société sans
	// exercice — d'où la place de ce test, le DERNIER du fichier.
	await seedTestState('with-company-no-fy');
	await login(page);
	await deuxExercices(page);
	const n = compter(page);
	await page.goto('/reports');
	// Le libellé « Exercice » (AC-14) est vérifié par le Vitest et par axe : l'E2E vise le data-testid
	// (garde KF-043, #326 : aucun sélecteur sur un libellé traduit).
	const menu = page.getByTestId('report-fiscal-year');
	await expect(menu).toBeVisible();
	await expect(page.getByTestId('report-period-heading')).toContainText(`du 01.01.${ANNEE} au 31.12.${ANNEE}`, { timeout: 10000 });
	await capture(page, '6-selecteur-ferme');
	// « menu ouvert » : un <select> natif ouvert ne se capture pas en mode sans tête ; on le déplie en liste
	// (`size`) le temps de la capture, pour montrer les options en graisse normale sous le menu en gras.
	await menu.evaluate((el) => ((el as HTMLSelectElement).size = 3));
	await capture(page, '7-selecteur-ouvert-en-liste');
	await menu.evaluate((el) => ((el as HTMLSelectElement).size = 0));

	await menu.selectOption({ label: String(PRECEDENTE) });
	await expect(page.getByTestId('report-from')).toHaveValue(`${PRECEDENTE}-01-01`);
	await expect(page.getByTestId('report-to')).toHaveValue(`${PRECEDENTE}-12-31`);
	await expect(page.getByTestId('report-period-heading')).toContainText(`Exercice ${PRECEDENTE}`);
	await expect(page).toHaveURL(new RegExp(`from=${PRECEDENTE}-01-01`));

	await page.getByTestId('report-from').fill(`${PRECEDENTE}-04-01`);
	await page.locator('h1').click();
	await expect(page.getByTestId('report-fy-whole')).toBeVisible();
	await expect(page.getByTestId('report-period-heading')).toContainText(`(partie de l’exercice ${PRECEDENTE})`);
	await capture(page, '8-exercice-entier-et-en-tete-partiel');
	await page.getByTestId('report-fy-whole').click();
	await expect(page.getByTestId('report-from')).toHaveValue(`${PRECEDENTE}-01-01`);
	await expect(page.getByTestId('report-fy-whole')).toHaveCount(0);

	const avant = n['balance-sheet'];
	await page.getByTestId('report-to').fill(`${ANNEE}-06-30`);
	await page.locator('h1').click();
	await expect(menu).toHaveValue('__multiple');
	await expect(page.getByTestId('report-needs-one-fy')).toBeVisible();
	await expect(page.getByTestId('report-content')).toHaveCount(0);
	expect(n['balance-sheet']).toBe(avant);
	await capture(page, '9-plusieurs');
	const axe = await new AxeBuilder({ page }).withTags(['wcag2a', 'wcag2aa']).analyze();
	expect(axe.violations).toEqual([]);

	await page.getByTestId('report-from').fill(`${PRECEDENTE - 1}-06-01`);
	await page.getByTestId('report-to').fill(`${PRECEDENTE - 1}-06-30`);
	await page.locator('h1').click();
	await expect(menu).toHaveValue('__outside');
	await capture(page, '10-hors-exercice');

	await menu.selectOption({ label: String(ANNEE) });
	await expect(menu.locator('option[value="__outside"]')).toHaveCount(0);
	await expect(page.getByTestId('report-content')).toBeVisible();
});
