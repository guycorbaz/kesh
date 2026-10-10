import { expect, test, type Page } from '@playwright/test';
import {
	seedTestState,
	clearAuthStorage,
	authedApiContext,
	disposeContextSafe,
} from './helpers/test-state';
import {
	createAndValidateInvoiceViaApi,
	createContactWithAddressViaApi,
} from './helpers/api-fixtures';

/**
 * Tests E2E — l'écran des postes ouverts (Story 15-1c-i, #518, AC13 part i).
 *
 * ⚠️ **Le montage crée son propre compte lettrable** (C-15-1c-13) : un actif au numéro
 * unique, jamais rattaché à un compte bancaire, et des montants uniques. Le spec ne lit
 * que les lignes qu'il a créées, par leur `lineId` en `data-testid` (garde #326 : aucun
 * sélecteur sur un libellé traduit). Seul le scénario (1) emploie le compte de créance
 * du preset `with-company` (`1100`, lettrable dans ce preset : aucun compte bancaire
 * ne le désigne — relevé au T0 sur l'API).
 *
 * ⚠️ C'est le seul test qui voit les valeurs traverser la frontière HTTP : `asOf`
 * toujours envoyé, les deux `lineId` d'une proposition, le code rendu par le `POST`,
 * le `DELETE` d'un groupe ouvert par son code.
 */

test.beforeAll(async () => {
	await seedTestState('with-company');
});

test.afterEach(async ({ page }) => {
	await clearAuthStorage(page);
});

/** La date LOCALE (Zurich) — celle que l'écran écrit dans l'URL. */
function todayZurich(): string {
	return new Date().toLocaleDateString('sv-SE', { timeZone: 'Europe/Zurich' });
}

async function login(page: Page, username = 'admin', password = 'admin123') {
	await page.goto('/login');
	await page.fill('#username', username);
	await page.fill('#password', password);
	await page.click('button[type="submit"]');
	await expect(page).toHaveURL('/');
}

/** Un montant unique au centime, pour ne jamais rencontrer une ligne d'un autre test. */
function uniqueAmount(): string {
	return `${1000 + (Date.now() % 8000)}.${String(Date.now() % 97).padStart(2, '0')}`;
}

let compteur = 0;

interface Montage {
	accountId: number;
	/** La ligne au débit du compte créé (écriture A). */
	debitLine: number;
	/** La ligne au crédit du compte créé (écriture B). */
	creditLine: number;
}

/**
 * Crée un compte lettrable (actif, numéro ≤ 10 caractères) et deux écritures
 * opposées d'un même montant sur lui, contre la caisse `1000` du preset.
 */
async function monter(page: Page, amount = uniqueAmount(), extra?: string): Promise<Montage & { extraLine?: number }> {
	const ctx = await authedApiContext(page);
	try {
		// ≤ 10 caractères (`routes/accounts.rs`) ; le compteur départage deux montages
		// tombés dans la même milliseconde.
		const number = `T${Date.now().toString().slice(-5)}${(compteur++ % 10).toString()}`;
		const accRes = await ctx.post('/api/v1/accounts', {
			data: { number, name: `Passage E2E ${number}`, accountType: 'Asset' },
		});
		expect(accRes.ok(), `create account: ${accRes.status()}`).toBeTruthy();
		const account = (await accRes.json()) as { id: number; letterable: boolean };
		expect(account.letterable, 'le compte créé est lettrable').toBe(true);

		const accounts = (await (await ctx.get('/api/v1/accounts')).json()) as {
			id: number;
			number: string;
		}[];
		const caisse = accounts.find((a) => a.number === '1000');
		expect(caisse, 'compte 1000 du preset').toBeTruthy();

		const ecrire = async (debitAcc: number, creditAcc: number, value: string, label: string) => {
			const res = await ctx.post('/api/v1/journal-entries', {
				data: {
					entryDate: todayZurich(),
					journal: 'OD',
					description: label,
					lines: [
						{ accountId: debitAcc, debit: value, credit: '0.00' },
						{ accountId: creditAcc, debit: '0.00', credit: value },
					],
				},
			});
			expect(res.ok(), `create entry: ${res.status()}`).toBeTruthy();
			const entry = (await res.json()) as { lines: { id: number; accountId: number }[] };
			return entry.lines.find((l) => l.accountId === account.id)!.id;
		};
		const debitLine = await ecrire(account.id, caisse!.id, amount, `Avance ${number}`);
		const creditLine = await ecrire(caisse!.id, account.id, amount, `Remboursement ${number}`);
		const extraLine = extra ? await ecrire(caisse!.id, account.id, extra, `Partiel ${number}`) : undefined;
		return { accountId: account.id, debitLine, creditLine, extraLine };
	} finally {
		await disposeContextSafe(ctx);
	}
}

test.describe('Postes ouverts', () => {
	test('(1) une facture soldée n’est pas ouverte ; son groupe s’ouvre par son code, sans « Délettrer »', async ({ page }) => {
		await login(page);
		const contactId = await createContactWithAddressViaApi(page, `Client Lettrage ${Date.now()}`);
		const invoiceId = await createAndValidateInvoiceViaApi(page, contactId);
		const ctx = await authedApiContext(page);
		let receivableId: number;
		let line: { id: number; letteringCode: string | null };
		let invoiceNumber: string;
		try {
			const settings = (await (await ctx.get('/api/v1/company/invoice-settings')).json()) as {
				defaultReceivableAccountId: number;
			};
			receivableId = settings.defaultReceivableAccountId;
			const accounts = (await (await ctx.get('/api/v1/accounts')).json()) as {
				id: number;
				number: string;
				letterable: boolean;
			}[];
			expect(accounts.find((a) => a.id === receivableId)?.letterable, 'créance lettrable').toBe(true);
			const caisse = accounts.find((a) => a.number === '1000')!;
			const settled = await ctx.post(`/api/v1/invoices/${invoiceId}/settlements`, {
				data: {
					settlementType: 'internal_account',
					accountId: caisse.id,
					amount: '972.90',
					settledOn: todayZurich(),
				},
			});
			expect(settled.ok(), `règlement : ${settled.status()}`).toBeTruthy();
			const invoice = (await (await ctx.get(`/api/v1/invoices/${invoiceId}`)).json()) as {
				journalEntryId: number;
				invoiceNumber: string;
			};
			invoiceNumber = invoice.invoiceNumber;
			const entry = (await (await ctx.get(`/api/v1/journal-entries/${invoice.journalEntryId}`)).json()) as {
				lines: { id: number; accountId: number; letteringCode: string | null }[];
			};
			line = entry.lines.find((l) => l.accountId === receivableId)!;
			expect(line.letteringCode, 'la créance est lettrée avec son règlement').toBeTruthy();
		} finally {
			await disposeContextSafe(ctx);
		}

		await page.goto(`/open-items?accountId=${receivableId}`);
		await expect(page.getByTestId('open-items-list')).toBeVisible();
		await expect(page.getByTestId(`open-item-row-${line.id}`)).toHaveCount(0);

		await page.goto(`/open-items?group=${line.letteringCode}`);
		await expect(page.getByTestId('lettering-group-origin')).toContainText('lettrage de la pièce');
		await expect(page.getByTestId('lettering-group-origin')).toContainText(invoiceNumber);
		await expect(page.getByTestId('lettering-group-blocked')).toBeVisible();
		await expect(page.getByTestId('lettering-group-dissolve')).toHaveCount(0);
	});

	test.describe.serial('(2) proposer et lettrer, (3) délettrer par le code', () => {
		let m: Montage;
		let code: string;

		test('(2) deux écritures opposées sont proposées, lettrées d’un clic, et la vue survit au rechargement', async ({ page }) => {
			await login(page);
			m = await monter(page);
			await page.goto(`/open-items?accountId=${m.accountId}`);
			// La date du jour est écrite dans l'URL (remplacement) : un rechargement retombe sur la même vue.
			await expect(page).toHaveURL(new RegExp(`asOf=${todayZurich()}`));
			await expect(page.getByTestId(`open-item-row-${m.debitLine}`)).toBeVisible();
			await expect(page.getByTestId(`open-item-row-${m.creditLine}`)).toBeVisible();

			const pair = `${m.debitLine}-${m.creditLine}`;
			await expect(page.getByTestId(`proposal-${pair}`)).toBeVisible();
			const posted = page.waitForResponse(
				(r) => r.url().endsWith('/api/v1/letterings') && r.request().method() === 'POST',
			);
			await page.getByTestId(`proposal-letter-${pair}`).click();
			const res = await posted;
			expect(res.status()).toBe(201);
			expect(res.request().postDataJSON()).toEqual({ lineIds: [m.debitLine, m.creditLine] });
			code = ((await res.json()) as { code: string }).code;

			await expect(page.getByTestId('open-items-message').getByTestId(`lettering-code-link-${code}`)).toBeVisible();
			await expect(page.getByTestId(`open-item-row-${m.debitLine}`)).toHaveCount(0);
			await expect(page.getByTestId(`open-item-row-${m.creditLine}`)).toHaveCount(0);

			await page.reload();
			await expect(page).toHaveURL(new RegExp(`accountId=${m.accountId}`));
			await expect(page.getByTestId('open-items-account')).toHaveValue(String(m.accountId));
			await expect(page.getByTestId('open-items-list')).toBeVisible();
			await expect(page.getByTestId(`open-item-row-${m.debitLine}`)).toHaveCount(0);
		});

		test('(3) le groupe ouvert par son code se délettre, et les deux lignes rouvrent', async ({ page }) => {
			await login(page);
			await page.goto(`/open-items?group=${code}`);
			await expect(page.getByTestId(`lettering-group-line-${m.debitLine}`)).toBeVisible();
			await expect(page.getByTestId('lettering-group-origin')).toContainText('lettrage manuel');
			await page.getByTestId('lettering-group-dissolve').click();
			await expect(page.getByTestId('open-items-message')).toContainText(code);
			await expect(page).not.toHaveURL(/group=/);

			await page.goto(`/open-items?accountId=${m.accountId}`);
			await expect(page.getByTestId(`open-item-row-${m.debitLine}`)).toBeVisible();
			await expect(page.getByTestId(`open-item-row-${m.creditLine}`)).toBeVisible();
		});
	});

	test('(4) une sélection déséquilibrée garde « Lettrer » inactif et dit l’écart', async ({ page }) => {
		await login(page);
		const m = await monter(page, '500.00', '120.00');
		await page.goto(`/open-items?accountId=${m.accountId}`);
		await page.getByTestId(`open-item-select-${m.debitLine}`).check();
		await page.getByTestId(`open-item-select-${m.extraLine}`).check();
		await expect(page.getByTestId('open-items-letter')).toBeDisabled();
		await expect(page.getByTestId('open-items-letter-blocker')).toContainText('380.00');
		// Équilibrée : le bouton s'active.
		await page.getByTestId(`open-item-select-${m.extraLine}`).uncheck();
		await page.getByTestId(`open-item-select-${m.creditLine}`).check();
		await expect(page.getByTestId('open-items-letter')).toBeEnabled();
	});

	test('(5) le bandeau de frontière avec la réconciliation est présent', async ({ page }) => {
		await login(page);
		await page.goto('/open-items');
		await expect(page.getByTestId('open-items-boundary')).toBeVisible();
	});

	test('(6) rôle Consultation : liste visible, ni case, ni « Lettrer », ni « Délettrer »', async ({ page }) => {
		await login(page);
		const m = await monter(page);
		const username = `consult-oi-${Date.now()}`;
		const ctx = await authedApiContext(page);
		let groupCode: string;
		try {
			const res = await ctx.post('/api/v1/users', {
				data: { username, password: 'MotDePasse12345', role: 'Consultation' },
			});
			expect(res.ok(), `create user failed: ${res.status()}`).toBeTruthy();
			const lettered = await ctx.post('/api/v1/letterings', { data: { lineIds: [m.debitLine, m.creditLine] } });
			expect(lettered.status()).toBe(201);
			groupCode = ((await lettered.json()) as { code: string }).code;
		} finally {
			await disposeContextSafe(ctx);
		}
		// Une seconde paire reste ouverte : la liste a des lignes, et des propositions.
		const second = await monter(page);
		await clearAuthStorage(page);
		await login(page, username, 'MotDePasse12345');

		await page.goto(`/open-items?accountId=${second.accountId}&group=${groupCode}`);
		// Présence avant absence.
		await expect(page.getByTestId(`open-item-row-${second.debitLine}`)).toBeVisible();
		await expect(page.getByTestId(`proposal-${second.debitLine}-${second.creditLine}`)).toBeVisible();
		await expect(page.getByTestId('lettering-group-origin')).toBeVisible();
		await expect(page.locator('[data-testid^="open-item-select-"]')).toHaveCount(0);
		await expect(page.getByTestId('open-items-letter')).toHaveCount(0);
		await expect(page.locator('[data-testid^="proposal-letter-"]')).toHaveCount(0);
		await expect(page.getByTestId('lettering-group-dissolve')).toHaveCount(0);
		await expect(page.getByTestId('lettering-group-blocked')).toHaveCount(0);

		// Le groupe lettré pour ce test reste en base, figé : un administrateur le délettre.
		await clearAuthStorage(page);
		await login(page);
		const admin = await authedApiContext(page);
		try {
			expect((await admin.delete(`/api/v1/letterings/${groupCode}`)).status()).toBe(204);
		} finally {
			await disposeContextSafe(admin);
		}
	});
});
