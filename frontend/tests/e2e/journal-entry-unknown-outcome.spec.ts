/**
 * Story 53-1a (#1355, AC-8 à AC-10, AC-12) — la saisie d'une écriture dont la réponse
 * est coupée APRÈS l'écriture (J6), le doublon que le serveur refuse ensuite (J5), la
 * confirmation, et la modification dont la réponse est perdue (J6-m).
 *
 * La coupure est réelle (patron `supplier-invoice-unknown-outcome.spec.ts`) :
 * `route.fetch()` envoie la requête au serveur, puis `route.abort()` jette sa réponse —
 * un `abort()` seul n'enverrait rien, et rien ne serait écrit.
 *
 * Décor synthétique : le seed `with-company`. Les captures (thème clair) s'écrivent dans
 * `KESH_E2E_CAPTURES_DIR`, sinon dans la sortie du test.
 */
import path from 'node:path';
import { expect, test, type Page, type Request, type TestInfo } from '@playwright/test';
import {
	seedTestState,
	clearAuthStorage,
	authedApiContext,
	disposeContextSafe,
} from './helpers/test-state';
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

/** Un compte d'actif et un compte de produits ou de passif, cherchés par numéro. */
async function twoAccounts(page: Page): Promise<{ debit: string; credit: string }> {
	const ctx = await authedApiContext(page);
	try {
		const resp = await ctx.get('/api/v1/accounts?includeArchived=false');
		expect(resp.ok()).toBeTruthy();
		const accounts: Array<{ number: string }> = await resp.json();
		const asset = accounts.find((a) => /^10[0-9]{2}$/.test(a.number)) ?? accounts[0];
		const revenue =
			accounts.find((a) => /^3[0-9]{3}$/.test(a.number)) ??
			accounts.find((a) => /^2[0-9]{3}$/.test(a.number)) ??
			accounts[1];
		return { debit: asset.number, credit: revenue.number };
	} finally {
		await disposeContextSafe(ctx);
	}
}

/** Ouvre le formulaire de création et saisit une écriture de 100.00. */
async function fillEntry(page: Page, label: string): Promise<void> {
	const { debit, credit } = await twoAccounts(page);
	await page.goto('/journal-entries');
	await page.getByRole('button', { name: /Nouvelle écriture/ }).click();
	await expect(page.getByText(/Saisie d'écriture/)).toBeVisible();
	await page.fill('#entry-description', label);
	const inputs = page.locator('input[aria-autocomplete="list"]');
	await inputs.nth(0).fill(debit);
	await page.getByRole('listbox').getByRole('option').first().click();
	await page.locator('input[inputmode="decimal"]').nth(0).fill('100.00');
	await inputs.nth(1).fill(credit);
	await page.getByRole('listbox').getByRole('option').first().click();
	await page.locator('input[inputmode="decimal"]').nth(3).fill('100.00');
	await expect(page.getByText(/✓ Équilibré/)).toBeVisible();
}

const submit = (page: Page) => page.getByTestId('journal-entry-submit');
const fields = (page: Page) => page.getByTestId('journal-entry-fields');

test('saisie coupée après l’écriture : J6, puis la reprise sans requête, puis le doublon (J5), puis la confirmation', async ({
	page,
}, testInfo) => {
	await login(page);
	const label = `E2E 53-1a coupure ${Date.now()}`;
	await fillEntry(page, label);
	await capture(page, testInfo, 'j1-saisie');

	// Le serveur écrit, la réponse est jetée. Les envois suivants passent.
	const posts: Request[] = [];
	let cut = true;
	await page.route('**/api/v1/journal-entries', async (route) => {
		if (route.request().method() !== 'POST') return route.fallback();
		posts.push(route.request());
		if (!cut) return route.fallback();
		const response = await route.fetch();
		expect(response.status(), 'le serveur a bien écrit').toBe(201);
		await route.abort('connectionreset');
	});

	await submit(page).click();

	// J6 : figé, avis, lien filtré dans un nouvel onglet, aucun envoi actif.
	const notice = page.getByTestId('journal-entry-unknown-outcome');
	await expect(notice).toBeVisible();
	await expect(page.getByTestId('journal-entry-unknown-outcome-text')).toHaveText(
		'L’enregistrement a peut-être abouti. Vérifiez dans le journal si l’écriture existe avant de la saisir de nouveau.',
	);
	const today = todayInZurich();
	const link = page.getByTestId('journal-entry-unknown-outcome-link');
	await expect(link).toHaveAttribute('href', `/journal-entries?dateFrom=${today}&dateTo=${today}`);
	await expect(link).toHaveAttribute('target', '_blank');
	await expect(fields(page)).toBeDisabled();
	await expect(submit(page)).toHaveCount(0);
	await expect(page.getByTestId('journal-entry-cancel')).toBeEnabled();
	// Aucun toast d'échec : l'avis remplace le toast « Erreur lors de la sauvegarde ».
	await expect(page.locator('[data-sonner-toast]')).toHaveCount(0);
	await capture(page, testInfo, 'j6-issue-inconnue');

	// Ctrl+S sans effet.
	await page.keyboard.press('Control+s');
	await page.waitForTimeout(300);
	expect(posts).toHaveLength(1);

	// La reprise rend la saisie intacte, sans requête.
	await page.getByTestId('journal-entry-resume-entry').click();
	expect(posts).toHaveLength(1);
	await expect(notice).toHaveCount(0);
	await expect(fields(page)).toBeEnabled();
	await expect(page.locator('#entry-description')).toHaveValue(label);
	await expect(submit(page)).toBeEnabled();

	// AC-9 : l'envoi rend le doublon (l'écriture a bien été écrite) : J5.
	cut = false;
	await submit(page).click();
	const dup = page.getByTestId('journal-entry-probable-duplicate');
	await expect(dup).toBeVisible();
	expect(posts).toHaveLength(2);
	await expect(page.getByTestId('journal-entry-probable-duplicate-text')).toContainText(
		'Une écriture identique existe déjà à cette date',
	);
	const cited = page.getByTestId('journal-entry-duplicate-link');
	await expect(cited).toHaveAttribute('href', /^\/journal-entries\/\d+$/);
	await expect(cited).toHaveAttribute('target', '_blank');
	await expect(cited).toContainText('Écriture n°');
	await expect(page.getByTestId('journal-entry-confirm-not-duplicate')).toBeVisible();
	await expect(page.getByTestId('journal-entry-modify-entry')).toBeVisible();
	await expect(submit(page)).toHaveCount(0);
	await expect(fields(page)).toBeDisabled();
	await expect(page.getByTestId('journal-entry-cancel')).toBeEnabled();
	await capture(page, testInfo, 'j5-doublon');

	// AC-10 : « Modifier la saisie » rend J1, l'envoi suivant est ordinaire (refusé encore).
	await page.getByTestId('journal-entry-modify-entry').click();
	await expect(fields(page)).toBeEnabled();
	await submit(page).click();
	await expect(dup).toBeVisible();
	expect(posts).toHaveLength(3);
	expect(posts[2].postDataJSON().confirmNotDuplicate).toBeUndefined();

	// La confirmation renvoie le corps gardé avec confirmNotDuplicate, une requête.
	await page.getByTestId('journal-entry-confirm-not-duplicate').click();
	await expect(page).toHaveURL(/\/journal-entries/);
	await expect(dup).toHaveCount(0);
	expect(posts).toHaveLength(4);
	expect(posts[3].postDataJSON().confirmNotDuplicate).toBe(true);
	expect(posts[3].postDataJSON().description).toBe(label);

	// Les deux écritures existent (la première, écrite par la coupure ; la seconde, confirmée).
	const ctx = await authedApiContext(page);
	try {
		const list = await ctx.get(`/api/v1/journal-entries?description=${encodeURIComponent(label)}`);
		expect(list.ok()).toBeTruthy();
		expect((await list.json()).items).toHaveLength(2);
	} finally {
		await disposeContextSafe(ctx);
	}
});

test('modification coupée après l’écriture : toast J6-m, formulaire actif', async ({
	page,
}, testInfo) => {
	await login(page);
	const label = `E2E 53-1a modif ${Date.now()}`;
	const { debit, credit } = await twoAccounts(page);
	const ctx = await authedApiContext(page);
	let id: number;
	try {
		const accounts: Array<{ id: number; number: string }> = await (
			await ctx.get('/api/v1/accounts?includeArchived=false')
		).json();
		const d = accounts.find((a) => a.number === debit)!;
		const c = accounts.find((a) => a.number === credit)!;
		const created = await ctx.post('/api/v1/journal-entries', {
			data: {
				entryDate: todayInZurich(),
				journal: 'OD',
				description: label,
				lines: [
					{ accountId: d.id, debit: '50.00', credit: '0' },
					{ accountId: c.id, debit: '0', credit: '50.00' },
				],
			},
		});
		expect(created.ok(), `création : ${created.status()}`).toBeTruthy();
		id = (await created.json()).id;
	} finally {
		await disposeContextSafe(ctx);
	}

	await page.route(`**/api/v1/journal-entries/${id}`, async (route) => {
		if (route.request().method() !== 'PUT') return route.fallback();
		const response = await route.fetch();
		expect(response.status(), 'le serveur a bien modifié').toBe(200);
		await route.abort('connectionreset');
	});
	await page.goto(`/journal-entries/${id}`);
	await page.getByTestId('edit-entry').click();
	await page.fill('#entry-description', `${label} corrigé`);
	await submit(page).click();

	await expect(
		page.getByText(
			'La modification a peut-être été enregistrée : rechargez l’écriture avant de la modifier encore.',
		),
	).toBeVisible();
	// Un seul toast : celui de J6-m, non « Erreur lors de la sauvegarde ».
	await expect(page.locator('[data-sonner-toast]')).toHaveCount(1);
	// Le formulaire reste actif.
	await expect(submit(page)).toBeEnabled();
	await capture(page, testInfo, 'j6m-modification-issue-inconnue');
});
