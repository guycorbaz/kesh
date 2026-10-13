/**
 * Story 28-4b (#642, AC12, AC17) — clore et rouvrir un compte bancaire, de bout en bout.
 *
 * - **T-b13** — parcours de l'écran *Comptes bancaires* : « Clore » désactivé sur le
 *   principal, dialogue de clôture (date, avertissement de solde), toast, bascule des
 *   clos, badge « Clos le … », seule action « Rouvrir », réouverture.
 * - **T-b12** — un compte clos sort des **six sélecteurs** d'écriture et de la **tuile**
 *   de l'accueil, et y revient après réouverture. Aucun écran d'écriture ne change de
 *   code : leur source (`GET /bank-accounts`, `/companies/current`) exclut les clos par
 *   défaut (AC2). Preuve à deux niveaux, choisis par écran (C-28-4-36) :
 *   - au **DOM** là où le sélecteur se rend sans pièce à fabriquer : tuile de l'accueil,
 *     import de relevés, réconciliation, lot de paiement, règlement d'une facture client
 *     (dialogue ouvert sur une facture validée) ;
 *   - à la **réponse que l'écran consomme** (`GET /api/v1/bank-accounts`) là où le
 *     sélecteur exige une pièce dans un état précis : échéancier (`/invoices/due-dates`)
 *     et paiement d'une facture fournisseur (`/supplier-invoices/{id}`), dont le
 *     chargement appelle `listBankAccounts()` au montage, que la pièce existe ou non.
 *
 * Captures (AC17) : si `KESH_CAPTURES_DIR` est posé, le parcours T-b13 enregistre ses
 * écrans en thème clair puis sombre dans ce dossier — HORS dépôt. Fixtures synthétiques :
 * IBAN de test, aucune donnée réelle.
 *
 * Pré-requis : MariaDB + seed `with-company`.
 */

import { expect, test, type Page } from '@playwright/test';
import { seedTestState, clearAuthStorage, authedApiContext, disposeContextSafe } from './helpers/test-state';
import { createAndValidateInvoiceViaApi, createContactWithAddressViaApi } from './helpers/api-fixtures';

// IBAN de test (MOD-97 valides), propres à cette spec.
const IBAN_PRINCIPAL = 'CH4900483526202000002';
const IBAN_A_CLORE = 'CH7600483526202000001';
const NOM_PRINCIPAL = 'Banque principale E2E 28-4b';
const NOM_A_CLORE = 'Banque à clore E2E 28-4b';

const CAPTURES = process.env.KESH_CAPTURES_DIR;

test.describe.configure({ mode: 'serial' });

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

interface Comptes {
	principal: number;
	aClore: number;
}

/** Comptes de liquidités propres à la spec (le seed n'en a qu'un hors compte débiteurs). */
const LIQUIDITES = [
	{ number: '1021', name: 'Banque principale E2E 28-4b' },
	{ number: '1022', name: 'Banque à clore E2E 28-4b' },
] as const;

/**
 * Crée les deux comptes bancaires de la spec (s'ils n'existent pas), chacun lié à son
 * propre compte de liquidités, créé au besoin — la tuile, le lot de paiement et le
 * règlement ne proposent qu'un compte lié, et le règlement écarte un compte lié au
 * compte débiteurs (`1100` dans le seed). Rend leurs identifiants.
 */
async function preparerComptes(page: Page): Promise<Comptes> {
	const ctx = await authedApiContext(page);
	try {
		const plan = (await (await ctx.get('/api/v1/accounts')).json()) as Array<{
			id: number;
			number: string;
		}>;
		const banques: number[] = [];
		for (const l of LIQUIDITES) {
			const deja = plan.find((a) => a.number === l.number);
			if (deja) {
				banques.push(deja.id);
				continue;
			}
			const res = await ctx.post('/api/v1/accounts', {
				data: { number: l.number, name: l.name, accountType: 'Asset', postable: true, role: null },
			});
			expect(res.ok(), `création du compte ${l.number} : ${res.status()}`).toBeTruthy();
			banques.push(((await res.json()) as { id: number }).id);
		}

		const existants = (await (
			await ctx.get('/api/v1/bank-accounts?includeClosed=true')
		).json()) as Array<{ id: number; iban: string }>;
		const creer = async (bankName: string, iban: string, isPrimary: boolean, journalAccountId: number) => {
			const deja = existants.find((b) => b.iban === iban);
			if (deja) return deja.id;
			const res = await ctx.post('/api/v1/bank-accounts', {
				data: { bankName, iban, isPrimary, journalAccountId },
			});
			expect(res.ok(), `création du compte ${bankName} : ${res.status()}`).toBeTruthy();
			return (await res.json()).id as number;
		};
		return {
			principal: await creer(NOM_PRINCIPAL, IBAN_PRINCIPAL, true, banques[0]),
			aClore: await creer(NOM_A_CLORE, IBAN_A_CLORE, false, banques[1]),
		};
	} finally {
		await disposeContextSafe(ctx);
	}
}

/** Clôt (ou rouvre) un compte par l'API, à sa version courante. */
async function basculerParApi(page: Page, id: number, geste: 'close' | 'reopen'): Promise<void> {
	const ctx = await authedApiContext(page);
	try {
		const liste = (await (
			await ctx.get('/api/v1/bank-accounts?includeClosed=true')
		).json()) as Array<{ id: number; version: number }>;
		const compte = liste.find((b) => b.id === id);
		expect(compte, `compte ${id} introuvable`).toBeTruthy();
		const res = await ctx.post(`/api/v1/bank-accounts/${id}/${geste}`, {
			data: { version: compte!.version },
		});
		expect(res.ok(), `${geste} ${id} : ${res.status()} ${await res.text()}`).toBeTruthy();
	} finally {
		await disposeContextSafe(ctx);
	}
}

async function capture(page: Page, nom: string): Promise<void> {
	if (!CAPTURES) return;
	for (const theme of ['light', 'dark'] as const) {
		await page.emulateMedia({ colorScheme: theme });
		await page.screenshot({ path: `${CAPTURES}/${nom}-${theme}.png`, fullPage: true });
	}
	await page.emulateMedia({ colorScheme: 'light' });
}

/** La date locale du jour, `AAAA-MM-JJ`, et sa forme suisse. */
function aujourdhui(): { iso: string; suisse: string } {
	const d = new Date();
	const [y, m, j] = [d.getFullYear(), String(d.getMonth() + 1).padStart(2, '0'), String(d.getDate()).padStart(2, '0')];
	return { iso: `${y}-${m}-${j}`, suisse: `${j}.${m}.${y}` };
}

test.describe('Clôture d’un compte bancaire (Story 28-4b)', () => {
	test('T-b13 — clore depuis l’écran, retrouver le compte par la bascule, le rouvrir', async ({ page }) => {
		await login(page);
		const { principal, aClore } = await preparerComptes(page);
		await page.goto('/bank-accounts');
		await expect(page.getByTestId(`bank-account-row-${aClore}`)).toBeVisible();

		// Le principal : « Clore » désactivé, expliqué.
		await expect(page.getByTestId(`close-${principal}`)).toBeDisabled();
		await expect(page.getByTestId(`close-primary-hint-${principal}`)).toContainText(
			"désignez d'abord un autre compte principal",
		);
		await capture(page, '01-liste-principal-desactive');

		// Dialogue de clôture : date du jour préremplie.
		await page.getByTestId(`close-${aClore}`).click();
		await expect(page.getByTestId('close-confirm')).toBeVisible();
		await expect(page.getByTestId('close-date')).toHaveValue(aujourdhui().iso);
		await capture(page, '02-dialogue-cloture');
		await page.getByTestId('close-confirm-button').click();

		// Clos masqué par défaut : la ligne disparaît, le toast dit comment la retrouver.
		await expect(page.getByText('Affichez les comptes clos pour le retrouver')).toBeVisible();
		await expect(page.getByTestId(`bank-account-row-${aClore}`)).toHaveCount(0);

		// Bascule : la ligne revient, badge daté, seule action « Rouvrir ».
		await page.getByTestId('toggle-closed').click();
		await expect(page.getByTestId(`closed-badge-${aClore}`)).toHaveText(`Clos le ${aujourdhui().suisse}`);
		await expect(page.getByTestId(`reopen-${aClore}`)).toBeVisible();
		for (const absent of ['close', 'link-button', 'edit-button', 'archive-button']) {
			await expect(page.getByTestId(`${absent}-${aClore}`)).toHaveCount(0);
		}
		await capture(page, '03-liste-compte-clos');

		// Réouverture.
		await page.getByTestId(`reopen-${aClore}`).click();
		await expect(page.getByTestId('reopen-confirm')).toBeVisible();
		await capture(page, '04-dialogue-reouverture');
		await page.getByTestId('reopen-confirm-button').click();
		await expect(page.getByTestId(`closed-badge-${aClore}`)).toHaveCount(0);
		await expect(page.getByTestId(`close-${aClore}`)).toBeEnabled();
	});

	test('T-b13 bis — un solde comptable non nul est signalé, sans bloquer (capture)', async ({ page }) => {
		await login(page);
		const { aClore } = await preparerComptes(page);
		// Une écriture sur le compte lié rend son solde non nul (C-28-4-6).
		const ctx = await authedApiContext(page);
		try {
			const liste = (await (await ctx.get('/api/v1/bank-accounts')).json()) as Array<{
				id: number;
				journalAccountId: number | null;
			}>;
			const lie = liste.find((b) => b.id === aClore)!.journalAccountId!;
			const plan = (await (await ctx.get('/api/v1/accounts')).json()) as Array<{
				id: number;
				number: string;
				postable: boolean;
				active: boolean;
			}>;
			const produit = plan.find((a) => a.postable && a.active && /^3\d{3}$/.test(a.number))!;
			// Patron de `createJournalEntryViaApi` (date de Zurich, journal OD), mais au
			// débit du compte LIÉ au compte à clore — le helper débite le premier 10xx.
			const res = await ctx.post('/api/v1/journal-entries', {
				data: {
					entryDate: new Date().toLocaleDateString('sv-SE', { timeZone: 'Europe/Zurich' }),
					journal: 'OD',
					description: 'E2E 28-4b — solde non nul',
					lines: [
						{ accountId: lie, debit: '42.50', credit: '0.00' },
						{ accountId: produit.id, debit: '0.00', credit: '42.50' },
					],
				},
			});
			expect(res.ok(), `écriture : ${res.status()} ${await res.text()}`).toBeTruthy();
		} finally {
			await disposeContextSafe(ctx);
		}
		await page.goto('/bank-accounts');
		await page.getByTestId(`close-${aClore}`).click();
		await expect(page.getByTestId('close-balance-warning')).toContainText('42.50');
		await expect(page.getByTestId('close-confirm-button')).toBeEnabled();
		await capture(page, '05-dialogue-avertissement-solde');
		await page.getByTestId('close-cancel-button').click();
		await expect(page.getByTestId('close-confirm')).toHaveCount(0);
	});

	test('T-b12 — un compte clos sort des six sélecteurs et de la tuile, et y revient rouvert', async ({ page }) => {
		await login(page);
		const { principal, aClore } = await preparerComptes(page);
		const contact = await createContactWithAddressViaApi(page, 'Client E2E 28-4b');
		const facture = await createAndValidateInvoiceViaApi(page, contact);

		/** Ids rendus par `GET /api/v1/bank-accounts` (sans paramètre) au chargement de `chemin`. */
		async function sourceDe(chemin: string): Promise<number[]> {
			const reponse = page.waitForResponse(
				(r) =>
					r.request().method() === 'GET' &&
					new URL(r.url()).pathname === '/api/v1/bank-accounts' &&
					new URL(r.url()).search === '',
			);
			await page.goto(chemin);
			return ((await (await reponse).json()) as Array<{ id: number }>).map((b) => b.id);
		}

		async function verifier(present: boolean): Promise<void> {
			const attendu = (loc: ReturnType<Page['locator']>) =>
				present ? expect(loc).toHaveCount(1) : expect(loc).toHaveCount(0);

			// (0) Tuile de l'accueil.
			await page.goto('/');
			await expect(page.getByTestId(`homepage-bank-name-${principal}`)).toBeVisible();
			await attendu(page.getByTestId(`homepage-bank-name-${aClore}`));

			// (1) Import de relevés, (2) réconciliation — `/companies/current`.
			for (const chemin of ['/bank-import', '/reconciliation']) {
				await page.goto(chemin);
				const select = page.getByTestId('bank-account-select');
				await expect(select.locator('option', { hasText: NOM_PRINCIPAL })).toHaveCount(1);
				await attendu(select.locator('option', { hasText: NOM_A_CLORE }));
			}

			// (3) Lot de paiement.
			await page.goto('/payment-batches');
			await page.getByTestId('payment-batch-new').click();
			const lot = page.locator('form select').first();
			await expect(lot.locator('option', { hasText: NOM_PRINCIPAL })).toHaveCount(1);
			await attendu(lot.locator('option', { hasText: NOM_A_CLORE }));

			// (4) Règlement d'une facture client.
			await page.goto(`/invoices/${facture}`);
			await page.getByTestId('settle-open').click();
			const reglement = page.getByTestId('settle-bank');
			await expect(reglement.locator('option', { hasText: NOM_PRINCIPAL })).toHaveCount(1);
			await attendu(reglement.locator('option', { hasText: NOM_A_CLORE }));

			// (5) Échéancier, (6) paiement d'une facture fournisseur — la source de l'écran.
			for (const chemin of ['/invoices/due-dates', '/supplier-invoices/999999999']) {
				const ids = await sourceDe(chemin);
				expect(ids, chemin).toContain(principal);
				expect(ids.includes(aClore), `${chemin} : compte clos ${present ? 'absent' : 'présent'}`).toBe(present);
			}
		}

		await basculerParApi(page, aClore, 'close');
		await verifier(false);
		await basculerParApi(page, aClore, 'reopen');
		await verifier(true);
	});
});
