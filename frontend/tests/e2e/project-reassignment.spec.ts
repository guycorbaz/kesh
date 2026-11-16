/**
 * Story 36-16a (#1167) — changer le compte analytique d'une facture client, même payée, depuis
 * sa fiche.
 *
 * Montage par l'API, en Administrateur (l'utilisateur de l'E2E) : deux comptes analytiques, une
 * facture de 100.— sans TVA au premier, validée et réglée entièrement. Puis, au navigateur : le
 * geste, le libellé changé, et — par `GET /journal-entries/{id}` — toutes les lignes de la vente et
 * du règlement au nouveau compte (la valeur qui traverse la frontière HTTP). Second scénario : le
 * même geste derrière un verrou des livres posé après la vente — avertissement, *Confirmer*,
 * succès.
 *
 * Nettoyage : le verrou est levé en `finally` (session Administrateur ; une clé est refusée), et
 * un `beforeAll` lève tout verrou résiduel d'un run interrompu, pour que la suite ne reste pas
 * verrouillée.
 *
 * Pré-requis : MariaDB up + KESH_TEST_MODE=true (cf. docs/testing.md).
 */

import path from 'node:path';
import { expect, test, type Page, type TestInfo } from '@playwright/test';
import {
	seedTestState,
	clearAuthStorage,
	authedApiContext,
	disposeContextSafe,
} from './helpers/test-state';
import {
	createContactWithAddressViaApi,
	createUser,
	invoiceVersionViaApi,
	todayInZurich,
	USER_PASSWORD,
} from './helpers/api-fixtures';
import { ajouterJoursIso } from './helpers/dates';

type Ctx = Awaited<ReturnType<typeof authedApiContext>>;

async function login(page: Page): Promise<void> {
	await page.goto('/login');
	await page.fill('#username', 'admin');
	await page.fill('#password', 'admin123');
	await page.click('button[type="submit"]');
	await expect(page).toHaveURL('/');
}

function capturePath(testInfo: TestInfo, name: string): string {
	const dossier = process.env.KESH_E2E_CAPTURES_DIR;
	return dossier ? path.join(dossier, `${name}.png`) : testInfo.outputPath(`${name}.png`);
}

async function capture(page: Page, testInfo: TestInfo, name: string): Promise<void> {
	await page.screenshot({ path: capturePath(testInfo, `36-16a-${name}`), fullPage: true });
}

async function releaseBooksLock(ctx: Ctx): Promise<void> {
	await ctx.post('/api/v1/companies/current/books-lock/release', {
		data: { through: null, motif: 'Fin du test E2E 36-16a' },
	});
}

test.beforeAll(async ({ browser }) => {
	await seedTestState('with-company');
	// Un verrou laissé par un run interrompu verrouillerait toute la suite.
	const page = await browser.newPage();
	try {
		await login(page);
		const ctx = await authedApiContext(page);
		try {
			await releaseBooksLock(ctx);
		} finally {
			await disposeContextSafe(ctx);
		}
	} finally {
		await page.close();
	}
});

test.afterEach(async ({ page }) => {
	await clearAuthStorage(page);
});

function uniq(prefix: string): string {
	return `${prefix}${Date.now() % 1_000_000}${Math.floor(Math.random() * 100)}`;
}

async function createProject(ctx: Ctx, code: string): Promise<{ id: number; code: string }> {
	const res = await ctx.post('/api/v1/projects', { data: { code, name: `Compte ${code}` } });
	expect(res.ok(), `compte analytique ${code} : ${res.status()}`).toBeTruthy();
	return { id: (await res.json()).id as number, code };
}

/** Une facture de 100.— sans TVA au compte `projectId`, validée et réglée entièrement. */
async function paidInvoice(page: Page, ctx: Ctx, projectId: number, date: string): Promise<number> {
	const contactId = await createContactWithAddressViaApi(page, uniq('Client Analytique '));
	const created = await ctx.post('/api/v1/invoices', {
		data: {
			contactId,
			date,
			dueDate: date,
			projectId,
			lines: [{ description: 'Prestation', quantity: '1', unitPrice: '100.00', vatRate: '0' }],
		},
	});
	expect(created.ok(), `facture : ${created.status()}`).toBeTruthy();
	const id = (await created.json()).id as number;
	const validated = await ctx.post(`/api/v1/invoices/${id}/validate`);
	expect(validated.ok(), `validation : ${validated.status()}`).toBeTruthy();
	const accounts = (await (await ctx.get('/api/v1/accounts')).json()) as {
		id: number;
		number: string;
	}[];
	const caisse = accounts.find((a) => a.number === '1000');
	expect(caisse, 'compte 1000 du seed').toBeTruthy();
	const settled = await ctx.post(`/api/v1/invoices/${id}/settlements`, {
		data: {
			settlementType: 'internal_account',
			accountId: caisse!.id,
			amount: '100.00',
			settledOn: date,
			version: await invoiceVersionViaApi(ctx, id),
		},
	});
	expect(settled.ok(), `règlement : ${settled.status()} ${await settled.text()}`).toBeTruthy();
	return id;
}

/** Le compte analytique de chaque ligne des écritures de la facture (vente et règlement). */
async function lineProjects(ctx: Ctx, invoiceId: number): Promise<(number | null)[]> {
	const invoice = await (await ctx.get(`/api/v1/invoices/${invoiceId}`)).json();
	const settlements = (await (
		await ctx.get(`/api/v1/invoices/${invoiceId}/settlements`)
	).json()) as { journalEntryId: number }[];
	const ids = [invoice.journalEntryId as number, ...settlements.map((s) => s.journalEntryId)];
	expect(ids).toHaveLength(2);
	const out: (number | null)[] = [];
	for (const id of ids) {
		const res = await ctx.get(`/api/v1/journal-entries/${id}`);
		expect(res.ok(), `écriture ${id} : ${res.status()}`).toBeTruthy();
		const entry = await res.json();
		for (const line of entry.lines as { projectId: number | null }[]) out.push(line.projectId);
	}
	return out;
}

async function chooseAndSave(page: Page, invoiceId: number, projectId: number): Promise<void> {
	await page.getByTestId('invoice-reassign-project-button').click();
	const select = page.getByTestId(`project-reassign-select-${invoiceId}`);
	await expect(page.getByTestId('project-reassign-save')).toBeDisabled();
	await select.selectOption(String(projectId));
	await page.getByTestId('project-reassign-save').click();
}

test('une facture payée change de compte analytique, toutes ses écritures avec elle', async ({
	page,
}, testInfo) => {
	await login(page);
	const ctx = await authedApiContext(page);
	try {
		const p = await createProject(ctx, uniq('P'));
		const q = await createProject(ctx, uniq('Q'));
		const id = await paidInvoice(page, ctx, p.id, ajouterJoursIso(todayInZurich(), -10));
		expect(await lineProjects(ctx, id)).toEqual([p.id, p.id, p.id, p.id]);

		await page.goto(`/invoices/${id}`);
		await expect(page.getByTestId('invoice-project')).toContainText(p.code);
		await page.getByTestId('invoice-reassign-project-button').click();
		await capture(page, testInfo, 'dialogue');
		await page.getByTestId(`project-reassign-select-${id}`).selectOption(String(q.id));
		await page.getByTestId('project-reassign-save').click();

		await expect(page.getByTestId('invoice-project')).toContainText(q.code);
		await expect(page.getByTestId('project-reassign-save')).toHaveCount(0);
		await capture(page, testInfo, 'succes');
		expect(await lineProjects(ctx, id)).toEqual([q.id, q.id, q.id, q.id]);
	} finally {
		await disposeContextSafe(ctx);
	}
});

test('rôle Lecture : la fiche d’une facture payée n’a pas le bouton', async ({ page }, testInfo) => {
	await login(page);
	const ctx = await authedApiContext(page);
	let id = 0;
	try {
		const p = await createProject(ctx, uniq('P'));
		id = await paidInvoice(page, ctx, p.id, ajouterJoursIso(todayInZurich(), -10));
	} finally {
		await disposeContextSafe(ctx);
	}
	const lecteur = await createUser(page, 'Consultation');
	await page.goto('/login');
	await page.fill('#username', lecteur);
	await page.fill('#password', USER_PASSWORD);
	await page.click('button[type="submit"]');
	await expect(page).toHaveURL('/');
	await page.goto(`/invoices/${id}`);
	await expect(page.getByTestId('invoice-project')).toBeVisible();
	await expect(page.getByTestId('invoice-reassign-project-button')).toHaveCount(0);
	await capture(page, testInfo, 'lecture-sans-bouton');
});

test('derrière le verrou des livres : avertissement, Confirmer, succès', async ({
	page,
}, testInfo) => {
	await login(page);
	const ctx = await authedApiContext(page);
	const today = todayInZurich();
	const lockedThrough = ajouterJoursIso(today, -1);
	try {
		const p = await createProject(ctx, uniq('P'));
		const q = await createProject(ctx, uniq('Q'));
		const id = await paidInvoice(page, ctx, p.id, ajouterJoursIso(today, -10));
		const lock = await ctx.post('/api/v1/companies/current/books-lock', {
			data: { through: lockedThrough },
		});
		expect(lock.ok(), `verrou : ${lock.status()} ${await lock.text()}`).toBeTruthy();

		await page.goto(`/invoices/${id}`);
		await chooseAndSave(page, id, q.id);
		const warning = page.getByTestId('project-reassign-warning');
		await expect(warning).toBeVisible();
		const [y, m, d] = lockedThrough.split('-');
		await expect(page.getByTestId('project-reassign-warning-locked')).toContainText(
			`${d}.${m}.${y}`,
		);
		await expect(warning).toContainText('2');
		await capture(page, testInfo, 'avertissement');
		await page.getByTestId('project-reassign-confirm').click();

		await expect(page.getByTestId('invoice-project')).toContainText(q.code);
		expect(await lineProjects(ctx, id)).toEqual([q.id, q.id, q.id, q.id]);
	} finally {
		await releaseBooksLock(ctx);
		await disposeContextSafe(ctx);
	}
});
