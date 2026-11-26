import { test, expect, type APIRequestContext, type Page, type TestInfo } from '@playwright/test';
import path from 'node:path';
import { seedTestState, clearAuthStorage, authedApiContext, disposeContextSafe } from './helpers/test-state';
import { login, creerCompte } from './helpers/reports-groupes';
import { todayInZurich } from './helpers/api-fixtures';

/**
 * Tests E2E — « Créer des groupes » (Story 46-5b, AC-8), contre la route de la
 * 46-5a (`POST /api/v1/accounts/groups`) : deux groupes créés, un rattachement et
 * une réutilisation, le tout en une confirmation ; un numéro mouvementé refusé à
 * l'aperçu ; une proposition écartée non créée ; un refus du serveur (compte
 * modifié depuis l'aperçu) qui n'écrit rien, puis « Recharger le plan » ; l'accès
 * refusé au rôle Consultation. Les comptes portent un préfixe unique par
 * exécution, sous un groupe racine propre au test.
 *
 * Captures (thème clair) dans `KESH_E2E_CAPTURES_DIR` s'il est posé, sinon dans
 * la sortie du test — hors dépôt.
 */

test.beforeAll(async () => {
	await seedTestState('with-company');
});

test.afterEach(async ({ page }) => {
	await clearAuthStorage(page);
});

function capturePath(testInfo: TestInfo, name: string): string {
	const dossier = process.env.KESH_E2E_CAPTURES_DIR;
	return dossier ? path.join(dossier, `${name}.png`) : testInfo.outputPath(`${name}.png`);
}

type Account = { id: number; number: string; parentId: number | null; postable: boolean; name: string; version: number; accountType: string };

async function lirePlan(api: APIRequestContext): Promise<Account[]> {
	const res = await api.get('/api/v1/accounts?includeArchived=true');
	expect(res.ok()).toBeTruthy();
	return (await res.json()) as Account[];
}

/** Le plan du test sous la racine `Z<k>` : rend le préfixe. */
async function preparerPlan(page: Page): Promise<string> {
	const k = `Z${Date.now().toString().slice(-4)}`;
	const api = await authedApiContext(page);
	try {
		const racine = await creerCompte(api, { number: k, name: 'Liquidités du test', postable: false });
		const sous = (number: string, name: string, parentId = racine) => creerCompte(api, { number, name, parentId });
		await sous(`${k}10`, 'Banque A CHF');
		await sous(`${k}11`, 'Banque A EUR');
		await sous(`${k}20`, 'Banque B CHF');
		await sous(`${k}21`, 'Banque B EUR');
		const mouvemente = await sous(`${k}3`, 'Compte mouvementé');
		await sous(`${k}30`, 'Banque C CHF');
		await sous(`${k}31`, 'Banque C EUR');
		await sous(`${k}40`, 'Banque D CHF');
		await sous(`${k}41`, 'Banque D EUR');
		const groupe5 = await creerCompte(api, { number: `${k}5`, name: 'Dépôts', parentId: racine, postable: false });
		await sous(`${k}50`, 'Dépôt locaux', groupe5);
		await sous(`${k}51`, 'Dépôt véhicule');
		await sous(`${k}6`, 'Avoirs sans écriture');
		await sous(`${k}60`, 'Terme 1');
		await sous(`${k}61`, 'Terme 2');
		const plan = await lirePlan(api);
		const ventes = plan.find((a) => a.number === '3000');
		expect(ventes, 'compte 3000 du preset').toBeTruthy();
		const entry = await api.post('/api/v1/journal-entries', {
			data: {
				entryDate: todayInZurich(),
				journal: 'OD',
				description: 'Écriture du test des groupes',
				lines: [
					{ accountId: mouvemente, debit: '100.00', credit: '0.00' },
					{ accountId: ventes!.id, debit: '0.00', credit: '100.00' },
				],
			},
		});
		expect(entry.ok(), `écriture : ${entry.status()} ${await entry.text()}`).toBeTruthy();
		return k;
	} finally {
		await disposeContextSafe(api);
	}
}

const card = (page: Page, prefix: string) => page.getByTestId(`account-groups-proposal-digit:${prefix}`);

test('créer deux groupes, rattacher, réutiliser ; refus mouvementé, écartée non créée, refus du serveur', async ({
	page,
}, testInfo) => {
	await login(page);
	const k = await preparerPlan(page);
	await page.goto('/accounts');
	const bouton = page.getByTestId('account-groups-button');
	await expect(bouton).toBeVisible();
	await page.screenshot({ path: capturePath(testInfo, '01-bouton-ecran-du-plan'), fullPage: false });
	await bouton.click();
	await expect(page).toHaveURL('/accounts/groups');
	await expect(page.getByTestId('account-groups-no-proposal')).toBeVisible();
	await page.screenshot({ path: capturePath(testInfo, '02-page-vide'), fullPage: true });

	for (const suffix of ['10', '11', '20', '21', '30', '31', '40', '41', '51', '60', '61']) {
		await page.getByTestId(`account-groups-check-${k}${suffix}`).check();
	}
	// Les comptes de groupe ne se cochent pas.
	await expect(page.getByTestId(`account-groups-check-${k}`)).toHaveCount(0);
	await expect(page.getByTestId(`account-groups-check-${k}5`)).toHaveCount(0);

	// Les trois cas d'un numéro pris : mouvementé (refus), groupe (rattachement), sans écriture (réutilisation).
	await expect(card(page, `${k}3`).locator('[data-code="numberTakenEntries"]')).toBeVisible();
	await expect(card(page, `${k}3`).getByTestId('account-groups-accept')).toBeDisabled();
	await expect(card(page, `${k}5`)).toHaveAttribute('data-kind', 'attach');
	await expect(card(page, `${k}6`).locator('[data-code="numberTakenReusable"]')).toBeVisible();
	await expect(card(page, `${k}1`).getByTestId('account-groups-name')).toHaveValue('Banque A');
	await expect(page.getByTestId('account-groups-confirm')).toBeDisabled();
	await page.screenshot({ path: capturePath(testInfo, '03-propositions-trois-cas'), fullPage: true });

	await card(page, `${k}6`).getByTestId('account-groups-reuse').check();
	await expect(card(page, `${k}6`)).toHaveAttribute('data-kind', 'reuse');
	await card(page, `${k}1`).getByTestId('account-groups-accept').click();
	await card(page, `${k}2`).getByTestId('account-groups-reject').click();
	await card(page, `${k}4`).getByTestId('account-groups-accept').click();
	await card(page, `${k}5`).getByTestId('account-groups-accept').click();
	await card(page, `${k}6`).getByTestId('account-groups-accept').click();
	await expect(card(page, `${k}2`)).toHaveAttribute('data-state', 'rejected');
	await expect(page.getByTestId(`account-groups-tree-${k}1`)).toHaveAttribute('data-mark', 'new');
	await expect(page.getByTestId(`account-groups-tree-${k}2`)).toHaveCount(0);
	await expect(page.getByTestId(`account-groups-tree-${k}6`)).toHaveAttribute('data-mark', 'becomesGroup');
	await page.screenshot({ path: capturePath(testInfo, '04-proposition-ecartee-et-acceptees'), fullPage: true });

	// Mode « un seul groupe » : une proposition ; le retour garde les choix.
	await page.getByTestId('account-groups-mode-one').click();
	await expect(page.getByTestId('account-groups-proposal-one')).toBeVisible();
	await page.screenshot({ path: capturePath(testInfo, '05-mode-un-seul-groupe'), fullPage: true });
	await page.getByTestId('account-groups-mode-digit').click();
	await expect(card(page, `${k}1`)).toHaveAttribute('data-state', 'accepted');
	await expect(card(page, `${k}2`)).toHaveAttribute('data-state', 'rejected');

	// Un compte modifié depuis l'aperçu : le serveur refuse, rien n'est écrit.
	const api = await authedApiContext(page);
	try {
		const avant = await lirePlan(api);
		const d40 = avant.find((a) => a.number === `${k}40`)!;
		const put = await api.put(`/api/v1/accounts/${d40.id}`, {
			data: { name: 'Banque D CHF bis', accountType: d40.accountType, role: null, postable: true, version: d40.version },
		});
		expect(put.ok(), `modification : ${put.status()}`).toBeTruthy();

		await page.getByTestId('account-groups-confirm').click();
		await expect(page.getByTestId('account-groups-refusal')).toBeVisible();
		await expect(page.getByTestId('account-groups-refusal-step')).toContainText(`${k}4`);
		await expect(card(page, `${k}4`)).toHaveClass(/ring-red-500/);
		await page.screenshot({ path: capturePath(testInfo, '06-erreur-du-serveur'), fullPage: true });
		const apres = await lirePlan(api);
		expect(apres.some((a) => a.number === `${k}1`), 'rien n’a été écrit').toBe(false);

		await page.getByTestId('account-groups-reload').click();
		await expect(page.getByTestId('account-groups-refusal')).toHaveCount(0);
		await expect(card(page, `${k}4`)).toHaveAttribute('data-state', 'accepted');
		await page.getByTestId('account-groups-confirm').click();
		await expect(page).toHaveURL('/accounts');
		await expect(page.getByTestId(`account-row-${k}1`)).toBeVisible();
		await page.screenshot({ path: capturePath(testInfo, '07-plan-apres-confirmation'), fullPage: true });

		const fin = await lirePlan(api);
		const par = (n: string) => fin.find((a) => a.number === n);
		const g1 = par(`${k}1`)!;
		const g4 = par(`${k}4`)!;
		expect(g1.postable).toBe(false);
		expect(g4.postable).toBe(false);
		expect(g1.parentId).toBe(par(k)!.id);
		expect(par(`${k}10`)!.parentId).toBe(g1.id);
		expect(par(`${k}11`)!.parentId).toBe(g1.id);
		expect(par(`${k}40`)!.parentId).toBe(g4.id);
		expect(par(`${k}51`)!.parentId).toBe(par(`${k}5`)!.id);
		expect(par(`${k}6`)!.postable).toBe(false);
		expect(par(`${k}60`)!.parentId).toBe(par(`${k}6`)!.id);
		// La proposition écartée n'est pas créée ; ses comptes n'ont pas bougé.
		expect(par(`${k}2`)).toBeUndefined();
		expect(par(`${k}20`)!.parentId).toBe(par(k)!.id);
		// Le numéro mouvementé n'a rien reçu.
		expect(par(`${k}30`)!.parentId).toBe(par(k)!.id);
	} finally {
		await disposeContextSafe(api);
	}
});

test('rôle Consultation : la page redirige vers le plan, sans bouton', async ({ page }) => {
	await login(page);
	const ctx = await authedApiContext(page);
	const username = `consult-groups-${Date.now()}`;
	let userId: number | null = null;
	try {
		const created = await ctx.post('/api/v1/users', {
			data: { username, password: 'MotDePasse12345', role: 'Consultation' },
		});
		expect(created.ok(), `création de l'utilisateur : ${created.status()}`).toBeTruthy();
		userId = ((await created.json()) as { id: number }).id;
		await clearAuthStorage(page);
		await page.goto('/login');
		await page.fill('#username', username);
		await page.fill('#password', 'MotDePasse12345');
		await page.click('button[type="submit"]');
		await expect(page).toHaveURL('/');
		await page.goto('/accounts');
		await expect(page.getByTestId('account-table')).toBeVisible();
		await expect(page.getByTestId('account-groups-button')).toHaveCount(0);
		await page.goto('/accounts/groups');
		await expect(page).toHaveURL('/accounts');
	} finally {
		if (userId !== null) await ctx.put(`/api/v1/users/${userId}/disable`);
		await disposeContextSafe(ctx);
	}
});
