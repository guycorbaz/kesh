/**
 * Story 8-1b T9 — Tests E2E Playwright pour la feature bank-import.
 *
 * 6 scénarios (F13 validate Pass 1) :
 *   1. imports a CAMT.053 v04 file end-to-end (AC #1)
 *   2. requires bank account selection before upload (AC #3 / #4)
 *   3. shows balance mismatch warning and accepts override (AC #14)
 *   4. rejects file > 10 MiB (AC #13)
 *   5. lists previous imports paginated (AC #10)
 *   6. accessibility — axe scan zero violations (AC #20)
 *
 * Pré-requis :
 *   - MariaDB up + KESH_TEST_MODE=true sur le backend (sinon
 *     seedTestState échoue avec un message explicite).
 *   - Au moins un bank_account configuré pour la company de test
 *     (ajouté via l'endpoint onboarding ci-dessous, qui consomme
 *     `with-company` et finalise l'onboarding).
 *   - Playwright browsers installés (Ubuntu 26.04 :
 *     PLAYWRIGHT_HOST_PLATFORM_OVERRIDE=ubuntu24.04-x64).
 */

import { expect, test, type Page } from '@playwright/test';
import AxeBuilder from '@axe-core/playwright';
import path from 'path';
import { fileURLToPath } from 'url';
import {
	seedTestState,
	clearAuthStorage,
	authedApiContext,
	disposeContextSafe,
} from './helpers/test-state';

const FIXTURE_DIR = path.join(path.dirname(fileURLToPath(import.meta.url)), 'fixtures');
const FIXTURE_MINIMAL = path.join(FIXTURE_DIR, 'camt053_v04_minimal.xml');
const FIXTURE_BALANCE_MISMATCH = path.join(FIXTURE_DIR, 'camt053_v04_balance_mismatch.xml');

const TEST_IBAN = 'CH4431999123000889012';

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

/**
 * Configure un bank_account via l'API onboarding (le seul endpoint
 * exposé v0.1). Idempotent : vérifie qu'un bank_account existe avant
 * de tenter la création. M11 : passe le JWT en header explicite.
 */
/**
 * Monte le compte bancaire du scénario via l'API.
 *
 * ⚠️ Passe par `authedApiContext(page)` et NON par `page.request.*` avec un
 * bearer lu en `localStorage` : depuis la Story 10-5, le JWT vit dans un
 * **cookie HttpOnly** inaccessible au JS, et `readAccessTokenFromStorage` est
 * marqué `@deprecated` — il rend toujours `null` en flux normal. Cette spec
 * avait gardé son helper maison et échouait donc au MONTAGE, sur
 * « JWT introuvable post-login », sans jamais atteindre ce qu'elle teste
 * (issue #107, KF-030).
 */
async function ensureBankAccount(page: Page): Promise<number> {
	const ctx = await authedApiContext(page);
	try {
		const company = await ctx.get('/api/v1/companies/current');
		if (company.ok()) {
			const json = await company.json();
			if (Array.isArray(json.bankAccounts) && json.bankAccounts.length > 0) {
				return json.bankAccounts[0].id as number;
			}
		}
		// ⚠️ Route CRUD, et NON `/api/v1/onboarding/bank-account` : cette dernière
		// refuse désormais en 400 `ONBOARDING_STEP_ALREADY_COMPLETED` sur le seed
		// `with-company`, qui marque l'étape franchie SANS créer de compte. Monter
		// un décor par une route d'onboarding était un abus qui s'est retourné le
		// jour où elle a gagné sa garde (issue #107, KF-030).
		const res = await ctx.post('/api/v1/bank-accounts', {
			data: {
				bankName: 'UBS Test',
				iban: TEST_IBAN,
				qrIban: null,
				isPrimary: true,
			},
		});
		expect(res.ok(), `bank-account create failed: ${res.status()}`).toBeTruthy();
		const created = await res.json();
		return created.id ?? 1;
	} finally {
		await disposeContextSafe(ctx);
	}
}

test('imports a CAMT.053 v04 file end-to-end', async ({ page }) => {
	await login(page);
	await ensureBankAccount(page);
	await page.goto('/bank-import');
	await expect(page.getByTestId('bank-import-page-title')).toBeVisible();

	// Sélection bank_account.
	await page.getByTestId('bank-account-select').selectOption({ index: 1 });

	// Upload via file input (drag-drop simulé par le file input).
	await page.getByTestId('bank-import-file-input').setInputFiles(FIXTURE_MINIMAL);

	// Preview affiché.
	await expect(page.getByTestId('bank-import-preview')).toBeVisible();
	await expect(page.getByTestId('preview-iban')).toHaveText(TEST_IBAN);

	// Confirm import.
	await page.getByTestId('bank-import-confirm').click();

	// Reset attendu après succès (preview disparaît).
	await expect(page.getByTestId('bank-import-preview')).toBeHidden();

	// La liste affiche au moins 1 import.
	await expect(page.getByTestId('bank-import-list-row').first()).toBeVisible();
});

test('requires bank account selection before upload', async ({ page }) => {
	await login(page);
	await ensureBankAccount(page);
	await page.goto('/bank-import');

	// Sans sélection bank_account, le file input est disabled.
	await expect(page.getByTestId('bank-import-file-input')).toBeDisabled();
});

test('shows balance mismatch warning and accepts override', async ({ page }) => {
	await login(page);
	await ensureBankAccount(page);
	await page.goto('/bank-import');

	await page.getByTestId('bank-account-select').selectOption({ index: 1 });
	await page
		.getByTestId('bank-import-file-input')
		.setInputFiles(FIXTURE_BALANCE_MISMATCH);

	// Warning visible.
	// ⚠️ `preview-warnings` n'a JAMAIS existé côté composant — vérifié au grep
	// sur tout `src/` : le seul porteur du testid était cette ligne. Le
	// conteneur réel est nommé d'après le warning qu'il porte, ce qui est plus
	// discriminant : un test qui attend « un warning quelconque » passerait sur
	// le mauvais (issue #107, KF-030).
	await expect(page.getByTestId('warning-balance-mismatch')).toBeVisible();

	// Confirm bouton désactivé tant que la checkbox confirmBalanceMismatch
	// n'est pas cochée.
	await expect(page.getByTestId('bank-import-confirm')).toBeDisabled();

	await page.getByTestId('confirm-balance-mismatch').check();

	await expect(page.getByTestId('bank-import-confirm')).toBeEnabled();
	await page.getByTestId('bank-import-confirm').click();
	await expect(page.getByTestId('bank-import-preview')).toBeHidden();
});

test('rejects file > 10 MiB', async ({ page }) => {
	await login(page);
	await ensureBankAccount(page);
	await page.goto('/bank-import');

	await page.getByTestId('bank-account-select').selectOption({ index: 1 });

	// Génère un fichier 11 MiB en mémoire (Buffer côté Node test runner).
	const big = Buffer.alloc(11 * 1024 * 1024, '<');
	await page.getByTestId('bank-import-file-input').setInputFiles({
		name: 'huge.xml',
		mimeType: 'application/xml',
		buffer: big,
	});

	const error = page.getByTestId('bank-import-error');
	await expect(error).toBeVisible();
	await expect(error).toHaveAttribute('data-error-code', 'BANK_IMPORT_TOO_LARGE');
});

test('lists previous imports paginated', async ({ page }) => {
	await login(page);
	await ensureBankAccount(page);
	await page.goto('/bank-import');

	// Crée un import via l'API (rapidité — pas via UI).
	// Review code Pass 1 : suppression du `page.evaluate(fetch)` qui
	// était dead code (404 systématique car SvelteKit ne sert pas
	// `/tests/fixtures/`). On lit directement la fixture côté Node
	// et on POST avec multipart natif Playwright + JWT (M11).
	const accountId = await ensureBankAccount(page);
	const fs = await import('fs');
	const buf = fs.readFileSync(FIXTURE_MINIMAL);
	const ctx = await authedApiContext(page);
	try {
		const res = await ctx.post('/api/v1/bank-imports', {
			multipart: {
				bankAccountId: accountId.toString(),
				file: { name: 'v04_minimal.xml', mimeType: 'application/xml', buffer: buf },
				// ⚠️ Ce test veut QU'UN import existe, pas qu'il soit le premier : la
				// même fixture a déjà été importée par le scénario end-to-end, qui
				// tourne avant lui sur la base partagée. Sans ce drapeau, le POST rend
				// **422** (doublon de fichier) et l'échec dépend de l'ORDRE des tests,
				// pas du code — il changeait de test d'un run à l'autre (issue #107).
				confirmDuplicateFile: 'true',
			},
		});
		expect(res.status()).toBe(201);
	} finally {
		await disposeContextSafe(ctx);
	}

	await page.reload();
	await expect(page.getByTestId('bank-import-list')).toBeVisible();
	await expect(page.getByTestId('bank-import-list-row').first()).toBeVisible();
});

test('accessibility — axe scan zero violations', async ({ page }) => {
	await login(page);
	await ensureBankAccount(page);
	await page.goto('/bank-import');
	await expect(page.getByTestId('bank-import-page-title')).toBeVisible();

	const results = await new AxeBuilder({ page }).analyze();
	expect(results.violations).toEqual([]);
});
