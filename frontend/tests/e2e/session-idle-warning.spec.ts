import { expect, test } from '@playwright/test';
import type { Browser, Page } from '@playwright/test';
import { seedTestState, clearAuthStorage, authedApiContext, disposeContextSafe } from './helpers/test-state';
import { PDF_SANS_QR, assertInboxDirsUsable, deposit, inboxHandshake, token, uniqueCopy } from './helpers/inbox';

/**
 * Story v0174-session-2 (0.17.4, #895, AC-4) — l'avertissement avant la fermeture de la
 * session pour inactivité, avec « Rester connecté ».
 *
 * La durée N vaut 2 minutes le temps de la spec (posée par l'écran d'API, remise à 60 dans
 * `afterAll`, même en cas d'échec : `workers: 1`, la suite suivante ne doit pas hériter de
 * N = 2). L'horloge de la page est simulée (`page.clock`, installée avant la connexion) ;
 * celle du serveur ne l'est pas : son échéance réelle (connexion + 2 min) n'est jamais
 * atteinte pendant un cas, si bien que le battement qui suit « Rester connecté » est
 * accepté. On n'avance jamais l'horloge pendant une requête en vol (la minuterie de 30 s
 * d'`apiClient` se déclencherait) : attente du réseau au repos, ou de la réponse du
 * battement, avant chaque `fastForward`.
 */

const N = 2;
const W = N * 60_000;

async function setIdleMinutes(browser: Browser, minutes: number): Promise<void> {
	const ctx0 = await browser.newContext();
	const page = await ctx0.newPage();
	try {
		await login(page);
		const api = await authedApiContext(page);
		try {
			const cur = await (await api.get('/api/v1/admin/session-settings')).json();
			const res = await api.put('/api/v1/admin/session-settings', {
				data: { sessionIdleMinutes: minutes, version: cur.version },
			});
			expect(res.ok(), `PUT session-settings : ${res.status()}`).toBeTruthy();
		} finally {
			await disposeContextSafe(api);
		}
	} finally {
		await ctx0.close();
	}
}

async function login(page: Page) {
	await page.goto('/login');
	await page.fill('#username', 'admin');
	await page.fill('#password', 'admin123');
	await page.click('button[type="submit"]');
	await expect(page).toHaveURL('/');
}

/** Connexion sous horloge simulée, puis réseau au repos. */
async function loginWithClock(page: Page) {
	await page.clock.install();
	await login(page);
	await page.waitForLoadState('networkidle');
}

const warning = (page: Page) => page.getByTestId('session-idle-warning');

test.beforeAll(async ({ browser }) => {
	await seedTestState('with-company');
	await setIdleMinutes(browser, N);
});

test.afterAll(async ({ browser }) => {
	await setIdleMinutes(browser, 60);
});

test.afterEach(async ({ page }) => {
	await clearAuthStorage(page);
});

test.describe('Avertissement avant la fermeture pour inactivité (#895)', () => {
	test('(a) l’avis paraît une minute avant ; « Rester connecté » garde la session', async ({ page }) => {
		await loginWithClock(page);
		await expect(warning(page)).toHaveCount(0);
		await page.clock.fastForward(W - 60_000 + 500);
		await expect(warning(page)).toBeVisible();
		// L'horloge de la page continue de couler après le saut : le compte à rebours
		// vaut au plus 1:00, quelques secondes de moins selon la durée de la connexion.
		await expect(page.getByTestId('session-idle-warning-countdown')).toHaveText(
			/Fermeture dans (1:00|0:[0-5]\d)\./,
		);

		const battement = page.waitForResponse('**/api/v1/auth/activity');
		await page.getByTestId('session-idle-warning-stay').click();
		await expect(warning(page)).toHaveCount(0);
		expect((await battement).status()).toBe(200);

		// Au-delà de l'ancienne échéance : toujours la même page.
		await page.clock.fastForward(61_000);
		await page.waitForTimeout(500);
		await expect(page).toHaveURL('/');
	});

	test('(b) sans activité jusqu’à l’échéance : page de connexion, message d’inactivité', async ({ page }) => {
		await loginWithClock(page);
		await page.clock.fastForward(W + 1_000);
		await expect(page).toHaveURL(/\/login\?reason=idle&minutes=2/);
	});

	test('(c) au-dessus d’un dialogue ouvert : le dialogue et sa saisie restent', async ({ page }) => {
		await loginWithClock(page);
		await page.goto('/contacts');
		await page.waitForLoadState('networkidle');
		await page.getByRole('button', { name: /Nouveau contact/ }).click();
		const titre = page.getByRole('heading', { name: /Nouveau contact/ });
		await expect(titre).toBeVisible();
		await page.fill('#form-name', 'Saisie à garder E2E');
		await page.waitForLoadState('networkidle');

		await page.clock.fastForward(W - 60_000 + 500);
		await expect(warning(page)).toBeVisible();
		const battement = page.waitForResponse('**/api/v1/auth/activity');
		await page.getByTestId('session-idle-warning-stay').click();
		expect((await battement).status()).toBe(200);
		await expect(warning(page)).toHaveCount(0);
		await expect(titre).toBeVisible();
		await expect(page.locator('#form-name')).toHaveValue('Saisie à garder E2E');
	});

	test('(d) au-dessus de l’aperçu d’un PDF : le bouton se clique', async ({ page }) => {
		await assertInboxDirsUsable();
		await loginWithClock(page);
		await inboxHandshake(page);
		const name = `e2e-v0174s-${token()}.pdf`;
		await deposit(name, uniqueCopy(PDF_SANS_QR, 'pdf'));
		await page.goto('/supplier-invoices/import');
		await page.getByTestId('inbox-import-trigger').click();
		await expect(page.getByTestId('inbox-import-report')).toBeVisible();
		const row = page.getByTestId('imported-row').filter({ hasText: name });
		await expect(row).toHaveCount(1);
		await row.getByTestId('imported-preview').click();
		await expect(row.getByTestId('document-preview-frame')).toHaveAttribute('src', /^blob:/);
		await page.waitForLoadState('networkidle');

		await page.clock.fastForward(W - 60_000 + 500);
		await expect(warning(page)).toBeVisible();
		const battement = page.waitForResponse('**/api/v1/auth/activity');
		// L'actionnabilité de Playwright prouve que le cadre ne masque pas le bouton.
		await page.getByTestId('session-idle-warning-stay').click();
		expect((await battement).status()).toBe(200);
		await expect(warning(page)).toHaveCount(0);
		await expect(row.getByTestId('document-preview-frame')).toBeVisible();
	});
});
