/**
 * Story 40-8b (AC-4) — le téléchargement Stripe par l'API, de l'écran au
 * serveur Stripe simulé.
 *
 * La clé se lit au **démarrage** du backend : les deux cas se jouent dans le
 * même démarrage, par deux comptes Stripe créés par la spec (CHF, compte du plan
 * sans devise) :
 *
 * (a) « Stripe sans API », dont le libellé n'a pas de clé : pas de bouton, la
 *     ligne d'aide de l'Administrateur ; l'import du rapport reste ;
 * (b) « Stripe API E2E », le libellé de `KESH_STRIPE_1_ACCOUNT` du backend
 *     d'E2E, démarré avec `KESH_STRIPE_1_KEY=rk_test_…` et
 *     `KESH_STRIPE_BASE_URL=http://127.0.0.1:<port>` vers le serveur simulé
 *     commun (`support/provider-mock.ts`, `stripeMockRoutes`), que la spec lance
 *     sur le port de `KESH_E2E_PROVIDER_MOCK_PORT` : mention « bac à sable »
 *     (clé de test), période de deux mois, progression « versement 1/2 »
 *     (les transactions du plus ancien versement, lu le second, retenues jusqu'à `mock.release()`),
 *     compte rendu avec le versement manuel, revue de la page qui montre les
 *     charges ; relancer la même période rend « 0 insérée ». Sans la variable du
 *     port, (b) est ignoré avec son motif (`docs/testing.md`, ignorés connus).
 *
 * Données fictives : identifiants `txn_TESTDL…`, `ch_TESTDL…`, `po_TESTDL…`,
 * aucun appel au vrai Stripe. Captures d'écran (thème clair) dans
 * `KESH_CAPTURES_DIR`.
 */

import { expect, test, type APIRequestContext, type Page } from '@playwright/test';
import { join } from 'node:path';
import { seedTestState, clearAuthStorage, authedApiContext, disposeContextSafe } from './helpers/test-state';
import { todayInZurich } from './helpers/api-fixtures';
import { ajouterJoursIso } from './helpers/dates';
import {
	PROVIDER_MOCK_PORT_VAR,
	providerMockPort,
	startProviderMock,
	stripeMockRoutes,
	type ProviderMock,
	type StripeMockPayout,
} from './support/provider-mock';

test.describe.configure({ mode: 'serial' });

const CAPTURES = process.env.KESH_CAPTURES_DIR;
/** Libellé et clé de la série 1 du backend d'E2E (`KESH_STRIPE_1_ACCOUNT`, `KESH_STRIPE_1_KEY`). */
const AVEC_API = process.env.KESH_E2E_STRIPE_ACCOUNT ?? 'Stripe API E2E';
const CLE = process.env.KESH_E2E_STRIPE_KEY ?? 'rk_test_e2efictif';
const SANS_API = 'Stripe sans API';
const PORT = providerMockPort();

async function capture(page: Page, name: string): Promise<void> {
	if (CAPTURES) await page.screenshot({ path: join(CAPTURES, `${name}.png`), fullPage: true });
}

async function captureOf(page: Page, testId: string, name: string): Promise<void> {
	if (CAPTURES) await page.getByTestId(testId).screenshot({ path: join(CAPTURES, `${name}.png`) });
}

async function login(page: Page): Promise<void> {
	await page.goto('/login');
	await page.fill('#username', 'admin');
	await page.fill('#password', 'admin123');
	await page.click('button[type="submit"]');
	await expect(page).toHaveURL('/');
}

async function api<T>(ctx: APIRequestContext, method: 'get' | 'post', path: string, data?: unknown): Promise<T> {
	const res = await ctx[method](path, data === undefined ? undefined : { data });
	expect(res.ok(), `${method.toUpperCase()} ${path} : ${res.status()} ${await res.text()}`).toBeTruthy();
	return (await res.json()) as T;
}

// Le jour suisse, lu une fois ; la période (les 60 jours qui le précèdent) et les
// versements s'y rapportent, quel que soit le jour de l'année.
const TODAY = todayInZurich();
const YEAR = TODAY.slice(0, 4);
const FROM = ajouterJoursIso(TODAY, -60);

/** Deux versements automatiques en CHF (deux charges chacun) et un manuel, dans la période. */
function payouts(): StripeMockPayout[] {
	const payout = (n: number, back: number, automatic: boolean): StripeMockPayout => {
		const day = ajouterJoursIso(TODAY, -back);
		const created = ajouterJoursIso(TODAY, -back - 2);
		return {
			id: `po_TESTDL000${n}`,
			arrivalAt: `${day}T08:00:00Z`,
			automatic,
			charges: automatic
				? [1, 2].map((k) => ({
						id: `txn_TESTDL${n}${k}`,
						source: `ch_TESTDL${n}${k}`,
						gross: `${n * 10 + k}.00`,
						fee: '0.60',
						createdAt: `${created}T10:00:00Z`,
					}))
				: [],
		};
	};
	return [payout(1, 40, true), payout(2, 10, true), payout(3, 20, false)];
}
/** Transactions insérées par le premier téléchargement : brut et frais des quatre charges, et les deux versements. */
const INSEREES = 4 * 2 + 2;

let mock: ProviderMock | null = null;
let fiscalYearId: number;

test.beforeAll(async () => {
	await seedTestState('with-company');
	if (PORT !== null) {
		mock = await startProviderMock(PORT, stripeMockRoutes({ key: CLE, payouts: payouts(), holdPayout: 'po_TESTDL0001' }));
	}
});

test.afterAll(async () => {
	await mock?.close();
	await seedTestState('with-company');
});

test.beforeEach(async ({ page }) => {
	await clearAuthStorage(page);
});

test('1 — préparation : deux comptes Stripe en CHF, l’exercice qui couvre la période', async ({ page }) => {
	await login(page);
	const ctx = await authedApiContext(page);
	try {
		const ledger = async (number: string, name: string) =>
			(await api<{ id: number }>(ctx, 'post', '/api/v1/accounts', { number, name, accountType: 'Asset' })).id;
		for (const [number, label] of [
			['1071', AVEC_API],
			['1072', SANS_API],
		] as const) {
			await api(ctx, 'post', '/api/v1/bank-accounts', {
				bankName: label,
				journalAccountId: await ledger(number, `Solde ${label}`),
				stripe: { account: label, feeAccountId: null, payoutBankAccountId: null },
			});
		}
		const years = await api<Array<{ id: number; startDate: string; endDate: string }>>(ctx, 'get', '/api/v1/fiscal-years');
		const current = years.find((y) => y.startDate <= FROM && TODAY <= y.endDate);
		const fallbackStart = `${FROM.slice(0, 4)}-01-01`;
		fiscalYearId =
			current?.id ??
			(
				await api<{ id: number }>(ctx, 'post', '/api/v1/fiscal-years', {
					name: `Exercice ${fallbackStart.slice(0, 4)}-${YEAR}`,
					startDate: fallbackStart,
					endDate: `${YEAR}-12-31`,
				})
			).id;
		// La liste dit l'état de chacun, et le mode lu à la clé de test ; jamais la clé.
		const accounts = await api<Array<{ stripe: { account: string } | null; providerApi: unknown }>>(
			ctx,
			'get',
			'/api/v1/bank-accounts',
		);
		expect(accounts.find((a) => a.stripe?.account === SANS_API)?.providerApi).toEqual({ configured: false });
		if (PORT !== null) {
			expect(accounts.find((a) => a.stripe?.account === AVEC_API)?.providerApi).toEqual({
				configured: true,
				mode: 'sandbox',
			});
		}
		expect(JSON.stringify(accounts)).not.toContain(CLE);
	} finally {
		await disposeContextSafe(ctx);
	}
});

test('2 — (a) compte sans clé : pas de bouton, la ligne d’aide de l’Administrateur', async ({ page }) => {
	await login(page);
	await page.goto('/reconciliation/stripe');
	await page.getByTestId('stripe-account').selectOption(SANS_API);
	await expect(page.getByTestId('stripe-import-button')).toBeVisible();
	await expect(page.getByTestId('stripe-download-not-configured')).toContainText('manuel d’administration');
	await expect(page.getByTestId('stripe-download-button')).toHaveCount(0);
	await capture(page, '10-compte-sans-cle');
});

test('3 — (b) téléchargement de deux mois : progression par versement, compte rendu, revue ; relancé : 0 insérée', async ({
	page,
}) => {
	test.skip(PORT === null, `${PROVIDER_MOCK_PORT_VAR} non posée : backend d'E2E sans KESH_STRIPE_1_* (docs/testing.md)`);
	test.setTimeout(120_000);
	await login(page);
	await page.goto('/reconciliation/stripe');
	await page.getByTestId('stripe-account').selectOption(AVEC_API);
	await page.getByTestId('stripe-fiscal-year').selectOption(String(fiscalYearId));
	await expect(page.getByTestId('stripe-download-not-configured')).toHaveCount(0);
	await expect(page.getByTestId('stripe-download-sandbox')).toContainText('bac à sable');
	const button = page.getByTestId('stripe-download-button');
	await expect(button).toBeVisible();
	await capture(page, '11-bouton-telecharger');

	await button.click();
	const dialog = page.getByTestId('provider-download-dialog');
	await expect(dialog).toBeVisible();
	await expect(page.getByTestId('provider-download-notice')).toContainText('Les versements manuels ne se téléchargent pas');
	await expect(page.getByTestId('provider-download-sandbox')).toContainText('bac à sable');
	await expect(page.getByTestId('provider-download-to')).toHaveValue(TODAY);
	await page.getByTestId('provider-download-from').fill(FROM);
	await captureOf(page, 'provider-download-dialog', '12-dialogue-periode');
	await page.getByTestId('provider-download-start').click();

	// Le plus récent versement lu (la liste de Stripe va du plus récent au plus ancien), l'autre retenu par le serveur simulé : « versement 1/2 ».
	await expect(page.getByTestId('provider-download-progress')).toContainText('versement 1/2', { timeout: 15_000 });
	await expect(page.getByTestId('provider-download-bar')).toBeVisible();
	await captureOf(page, 'provider-download-dialog', '13-dialogue-progression');
	mock?.release();

	// Compte rendu : celui de l'import du rapport, puis le versement manuel.
	const report = page.getByTestId('stripe-download-report-CHF');
	await expect(report).toContainText(`${INSEREES} transaction(s) insérée(s), 0 doublon(s)`, { timeout: 30_000 });
	await expect(page.getByTestId('stripe-download-report-payout-po_TESTDL0001')).toContainText('2 ligne(s), net 21.80 CHF');
	await expect(page.getByTestId('stripe-download-report-manual')).toContainText('po_TESTDL0003');
	await captureOf(page, 'provider-download-dialog', '14-dialogue-compte-rendu');
	await page.getByTestId('provider-download-close').click();
	await expect(dialog).toHaveCount(0);

	// La revue de la page montre les charges téléchargées (sans vente : écarts).
	for (const id of ['txn_TESTDL11', 'txn_TESTDL22']) {
		await expect(page.getByTestId('stripe-gaps')).toContainText(id);
	}
	await capture(page, '15-revue-apres-telechargement');

	// Le serveur simulé n'a reçu que des lectures, avec la clé de test.
	expect(mock!.requests.every((r) => r.method === 'GET' && r.authorization === `Bearer ${CLE}`)).toBe(true);
	expect(mock!.requests.filter((r) => r.path === '/v1/balance_transactions')).toHaveLength(2);

	// Relancer la même période : rien n'est doublé.
	await button.click();
	await page.getByTestId('provider-download-from').fill(FROM);
	await page.getByTestId('provider-download-start').click();
	await expect(page.getByTestId('stripe-download-report-CHF')).toContainText(
		`0 transaction(s) insérée(s), ${INSEREES} doublon(s)`,
		{ timeout: 30_000 },
	);
	await captureOf(page, 'provider-download-dialog', '16-dialogue-relance-sans-doublon');
});
