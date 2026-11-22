/**
 * Story 38-12b (AC-7) — le téléchargement PayPal par l'API, de l'écran au
 * serveur PayPal simulé.
 *
 * La configuration des identifiants se lit au **démarrage** du backend : les deux
 * cas se jouent dans le même démarrage, par deux comptes PayPal créés par la
 * spec (CHF, compte du plan sans devise) :
 *
 * (a) « PayPal sans API », dont le libellé n'a pas de série : pas de bouton, la
 *     ligne d'aide de l'Administrateur ; l'import du CSV reste ;
 * (b) « PayPal API E2E », le libellé de `KESH_PAYPAL_1_ACCOUNT` du backend
 *     d'E2E, démarré avec `KESH_PAYPAL_1_CLIENT_ID`, `KESH_PAYPAL_1_SECRET` et
 *     `KESH_PAYPAL_BASE_URL=http://127.0.0.1:<port>` vers le serveur simulé
 *     commun (`support/provider-mock.ts`), que la spec lance sur le port de
 *     `KESH_E2E_PROVIDER_MOCK_PORT` : période de deux mois, progression et pause
 *     visibles, compte rendu, revue de la page qui montre les lignes ; relancer
 *     la même période rend « 0 insérée ». Sans la variable du port, (b) est
 *     ignoré avec son motif (`docs/testing.md`, ignorés connus).
 *
 * Données fictives : identifiants `0TESTDL…`, e-mails `@example.invalid`, aucun
 * appel au vrai PayPal. Captures d'écran (thème clair) dans `KESH_CAPTURES_DIR`.
 */

import { expect, test, type APIRequestContext, type Page } from '@playwright/test';
import { join } from 'node:path';
import { seedTestState, clearAuthStorage, authedApiContext, disposeContextSafe } from './helpers/test-state';
import { todayInZurich } from './helpers/api-fixtures';
import { ajouterJoursIso } from './helpers/dates';
import {
	PROVIDER_MOCK_PORT_VAR,
	paypalMockRoutes,
	providerMockPort,
	startProviderMock,
	type PaypalMockTransaction,
	type ProviderMock,
} from './support/provider-mock';

test.describe.configure({ mode: 'serial' });

const CAPTURES = process.env.KESH_CAPTURES_DIR;
/** Libellé de la série 1 du backend d'E2E (`KESH_PAYPAL_1_ACCOUNT`). */
const AVEC_API = process.env.KESH_E2E_PAYPAL_ACCOUNT ?? 'PayPal API E2E';
const SANS_API = 'PayPal sans API';
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

// Le jour suisse, lu une fois ; la période (les 60 jours qui le précèdent, soit
// trois fenêtres de 30 jours, la pause tombant sur la deuxième) et les
// transactions s'y rapportent, **quel que soit le jour de l'année** : la période
// peut chevaucher le 1er janvier, l'exercice retenu par la préparation la couvre
// entière (revue P1, L3 : bornée au 1er janvier, elle tenait en une fenêtre en
// janvier, sans pause ni transaction).
const TODAY = todayInZurich();
const YEAR = TODAY.slice(0, 4);
const FROM = ajouterJoursIso(TODAY, -60);
/** `AAAA-MM-JJ` → `JJ.MM.AAAA`, la forme des dates à l'écran. */
const swiss = (iso: string) => `${iso.slice(8, 10)}.${iso.slice(5, 7)}.${iso.slice(0, 4)}`;

/** Quatre ventes fictives en CHF, dans la période, à 10:00 UTC. */
function transactions(): PaypalMockTransaction[] {
	const out: PaypalMockTransaction[] = [];
	for (const [i, back] of [50, 35, 20, 5].entries()) {
		const day = ajouterJoursIso(TODAY, -back);
		out.push({
			id: `0TESTDL0000${i + 1}`,
			at: `${day}T10:00:00+00:00`,
			gross: `${(i + 1) * 10}.00`,
			currency: 'CHF',
			name: `Client fictif ${i + 1}`,
			email: `client${i + 1}@example.invalid`,
		});
	}
	return out;
}

let mock: ProviderMock | null = null;
/*
 * Le serveur simulé retient les fenêtres 2 et 3 jusqu'à ce que (b) ait vu le
 * `409` d'une relance (`mock.release()`) : la tâche est alors forcément en cours,
 * quel que soit le temps que la spec a mis (revue P2, L4). La retenue appartient
 * au serveur lancé par `beforeAll`, et `mock.close()` la libère (revue P3, LOW-1).
 */
let fiscalYearId: number;
/** Début de l'exercice qui couvre la période (`with-company` : 2020-2030) : début de la période proposée. */
let fiscalYearStart: string;

test.beforeAll(async () => {
	await seedTestState('with-company');
	if (PORT !== null) {
		mock = await startProviderMock(
			PORT,
			paypalMockRoutes({
				transactions: transactions(),
				pageDelayMs: 900,
				rateLimitOnce: { window: 2, retryAfterSeconds: 3 },
				holdFromWindow: { window: 2 },
				lastRefreshed: `${TODAY}T05:00:00+00:00`,
			}),
		);
	}
});

test.afterAll(async () => {
	await mock?.close();
	await seedTestState('with-company');
});

test.beforeEach(async ({ page }) => {
	await clearAuthStorage(page);
});

test('1 — préparation : deux comptes PayPal en CHF, l’exercice qui couvre la période', async ({ page }) => {
	await login(page);
	const ctx = await authedApiContext(page);
	try {
		const ledger = async (number: string, name: string) =>
			(await api<{ id: number }>(ctx, 'post', '/api/v1/accounts', { number, name, accountType: 'Asset' })).id;
		for (const [number, label] of [
			['1061', AVEC_API],
			['1062', SANS_API],
		] as const) {
			await api(ctx, 'post', '/api/v1/bank-accounts', {
				bankName: label,
				journalAccountId: await ledger(number, `Solde ${label}`),
				paypal: { account: label, feeAccountId: null },
			});
		}
		const years = await api<Array<{ id: number; startDate: string; endDate: string }>>(ctx, 'get', '/api/v1/fiscal-years');
		// L'exercice que la revue lit doit couvrir toute la période ; à défaut (le
		// `with-company` a 2020-2030), un exercice du 1er janvier de l'année de son
		// début au 31 décembre de l'année du jour.
		const current = years.find((y) => y.startDate <= FROM && TODAY <= y.endDate);
		const fallbackStart = `${FROM.slice(0, 4)}-01-01`;
		fiscalYearStart = current?.startDate ?? fallbackStart;
		fiscalYearId =
			current?.id ??
			(
				await api<{ id: number }>(ctx, 'post', '/api/v1/fiscal-years', {
					name: `Exercice ${fallbackStart.slice(0, 4)}-${YEAR}`,
					startDate: fallbackStart,
					endDate: `${YEAR}-12-31`,
				})
			).id;
		// La liste dit l'état de chacun, sans secret.
		const accounts = await api<Array<{ paypal: { account: string } | null; providerApi: unknown }>>(
			ctx,
			'get',
			'/api/v1/bank-accounts',
		);
		const sans = accounts.find((a) => a.paypal?.account === SANS_API);
		expect(sans?.providerApi).toEqual({ configured: false });
		if (PORT !== null) {
			const avec = accounts.find((a) => a.paypal?.account === AVEC_API);
			expect(avec?.providerApi).toEqual({ configured: true });
		}
	} finally {
		await disposeContextSafe(ctx);
	}
});

test('2 — (a) compte sans identifiants : pas de bouton, la ligne d’aide de l’Administrateur', async ({ page }) => {
	await login(page);
	await page.goto('/reconciliation/paypal');
	await page.getByTestId('paypal-account').selectOption(SANS_API);
	await expect(page.getByTestId('paypal-import-button')).toBeVisible();
	await expect(page.getByTestId('paypal-download-not-configured')).toContainText('manuel d’administration');
	await expect(page.getByTestId('paypal-download-button')).toHaveCount(0);
	await capture(page, '10-compte-sans-identifiants');
});

test('3 — (b) téléchargement de deux mois : progression, pause, compte rendu, revue ; relancé : 0 insérée', async ({
	page,
}) => {
	test.skip(PORT === null, `${PROVIDER_MOCK_PORT_VAR} non posée : backend d'E2E sans KESH_PAYPAL_1_* (docs/testing.md)`);
	test.setTimeout(120_000);
	const expected = transactions();
	await login(page);
	await page.goto('/reconciliation/paypal');
	await page.getByTestId('paypal-account').selectOption(AVEC_API);
	await page.getByTestId('paypal-fiscal-year').selectOption(String(fiscalYearId));
	await expect(page.getByTestId('paypal-download-not-configured')).toHaveCount(0);
	const button = page.getByTestId('paypal-download-button');
	await expect(button).toBeVisible();
	await capture(page, '11-bouton-telecharger');

	await button.click();
	const dialog = page.getByTestId('provider-download-dialog');
	await expect(dialog).toBeVisible();
	await expect(page.getByTestId('provider-download-notice')).toContainText('trois heures');
	await expect(page.getByTestId('provider-download-from')).toHaveValue(fiscalYearStart);
	await expect(page.getByTestId('provider-download-to')).toHaveValue(TODAY);
	await page.getByTestId('provider-download-from').fill(FROM);
	await captureOf(page, 'provider-download-dialog', '12-dialogue-periode');
	await page.getByTestId('provider-download-start').click();

	// Progression : « fenêtre n/3 » et la barre ; puis la pause demandée par le serveur simulé.
	await expect(page.getByTestId('provider-download-progress')).toContainText(/fenêtre \d\/\d/);
	await expect(page.getByTestId('provider-download-bar')).toBeVisible();
	await captureOf(page, 'provider-download-dialog', '13-dialogue-progression');

	// Fermer, rouvrir, lancer une AUTRE période : le serveur refuse (409) et rend la
	// période de la tâche en cours, que le dialogue annonce en reprenant son suivi
	// (revue P1, M1). La tâche ne peut pas être finie : le serveur simulé retient la
	// fenêtre 2 jusqu'à `mock.release()` (revue P2, L4).
	await page.getByTestId('provider-download-close').click();
	await expect(dialog).toHaveCount(0);
	await button.click();
	await page.getByTestId('provider-download-from').fill(ajouterJoursIso(FROM, 1));
	await page.getByTestId('provider-download-start').click();
	await expect(page.getByTestId('provider-download-resumed')).toContainText(
		`Un téléchargement du ${swiss(FROM)} au ${swiss(TODAY)} est déjà en cours pour ce compte`,
	);
	await captureOf(page, 'provider-download-dialog', '18-dialogue-reprise-autre-periode');
	// Le 409 est vu : le serveur simulé rend la fenêtre 2 (la pause), puis la suite.
	mock?.release();
	await expect(page.getByTestId('provider-download-paused')).toContainText('reprise dans', { timeout: 15_000 });
	await captureOf(page, 'provider-download-dialog', '14-dialogue-pause');

	// Compte rendu : le même que l'import du CSV.
	const report = page.getByTestId('paypal-download-report-CHF');
	await expect(report).toContainText(`${expected.length} transaction(s) insérée(s), 0 doublon(s)`, { timeout: 30_000 });
	await expect(page.getByTestId('provider-download-last-refreshed')).toContainText('à jour jusqu’au');
	await captureOf(page, 'provider-download-dialog', '15-dialogue-compte-rendu');
	await page.getByTestId('provider-download-close').click();
	await expect(dialog).toHaveCount(0);

	// La revue de la page montre les lignes téléchargées (ventes sans écriture : écarts).
	for (const t of expected) {
		await expect(page.getByTestId('paypal-gaps')).toContainText(t.id);
	}
	await capture(page, '16-revue-apres-telechargement');

	// Le serveur simulé n'a reçu que des demandes authentifiées, en lecture.
	expect(mock!.requests.every((r) => r.authorization !== null)).toBe(true);
	expect(mock!.requests.every((r) => r.path === '/v1/oauth2/token' || r.method === 'GET')).toBe(true);
	expect(mock!.requests.some((r) => r.query.get('fields') === 'all')).toBe(true);

	// Relancer la même période : rien n'est doublé.
	await button.click();
	await page.getByTestId('provider-download-from').fill(FROM);
	await page.getByTestId('provider-download-start').click();
	await expect(page.getByTestId('paypal-download-report-CHF')).toContainText(
		`0 transaction(s) insérée(s), ${expected.length} doublon(s)`,
		{ timeout: 30_000 },
	);
	await expect(page.getByTestId('provider-download-resumed')).toHaveCount(0);
	await captureOf(page, 'provider-download-dialog', '17-dialogue-relance-sans-doublon');
});
