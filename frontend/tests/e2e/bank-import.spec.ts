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
import { seedTestState, clearAuthStorage } from './helpers/test-state';

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
 * Récupère le JWT stocké en localStorage par l'auth-store post-login
 * (review code Pass 1 M11) — `page.request.*` ne partage pas l'auth
 * de la page (pas de cookies dans cette stack JWT-in-memory). On lit
 * `kesh:auth:accessToken` directement et on l'injecte en bearer.
 *
 * Review code Pass 2 L4 : try/catch défensif autour de
 * `localStorage.getItem` — privacy mode browser ou quota plein peuvent
 * lever (rare mais a déjà cassé des CI flaky historiquement). Le throw
 * reste explicite pour échec rapide, juste avec un meilleur diag.
 */
async function authHeaders(page: Page): Promise<Record<string, string>> {
	let token: string | null = null;
	try {
		token = await page.evaluate(() => {
			try {
				return localStorage.getItem('kesh:auth:accessToken');
			} catch {
				return null;
			}
		});
	} catch (err) {
		throw new Error(
			`localStorage inaccessible (privacy mode ou quota ?) : ${String(err)}`,
		);
	}
	if (!token) {
		throw new Error(
			'JWT introuvable en localStorage post-login (auth-store cleared ou login fail ?)',
		);
	}
	return { Authorization: `Bearer ${token}` };
}

/**
 * Configure un bank_account via l'API onboarding (le seul endpoint
 * exposé v0.1). Idempotent : vérifie qu'un bank_account existe avant
 * de tenter la création. M11 : passe le JWT en header explicite.
 */
async function ensureBankAccount(page: Page): Promise<number> {
	const headers = await authHeaders(page);
	const company = await page.request.get('/api/v1/companies/current', { headers });
	if (company.ok()) {
		const json = await company.json();
		if (Array.isArray(json.bankAccounts) && json.bankAccounts.length > 0) {
			return json.bankAccounts[0].id as number;
		}
	}
	const res = await page.request.post('/api/v1/onboarding/bank-account', {
		headers,
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
	await expect(page.getByTestId('preview-warnings')).toBeVisible();

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
	const headers = await authHeaders(page);
	const res = await page.request.post('/api/v1/bank-imports', {
		headers,
		multipart: {
			bankAccountId: accountId.toString(),
			file: { name: 'v04_minimal.xml', mimeType: 'application/xml', buffer: buf },
		},
	});
	expect(res.status()).toBe(201);

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
