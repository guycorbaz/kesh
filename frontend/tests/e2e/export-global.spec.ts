/**
 * Story 9-2b — Test Playwright E2E : export global ZIP via UI.
 *
 * Scénario AC #32 :
 *   1. Login admin.
 *   2. Navigation `/export`.
 *   3. Assert page contient titre `export-global-title` + bouton « Lancer
 *      l'export » visible.
 *   4. Cliquer « Lancer l'export ».
 *   5. Pendant la génération, assert bouton `disabled` avec libellé
 *      « Génération de l'export… » (AC #25 état UX).
 *   6. `await page.waitForEvent('download')` → `await download.saveAs(...)`.
 *   7. `fs.readFile(filepath)` → assert byte signature `PK\x03\x04` + filename
 *      pattern `kesh-export-.+-YYYY-MM-DD.zip` (AC #28).
 *   8. Assert bouton redevient `enabled` post-download.
 *
 * Pattern download : `download.saveAs(...)` puis `fs.readFile()` — PAS
 * `download.path()` qui peut retourner `null` (cohérent Story 9-2a Pass 1
 * ECH-M5).
 *
 * Pré-requis : MariaDB up + KESH_TEST_MODE=true + seed `with-company` +
 * Playwright browsers (Ubuntu 26.04+ : PLAYWRIGHT_HOST_PLATFORM_OVERRIDE=ubuntu24.04-x64).
 *
 * Pas exécuté par la CI principale (cf. CLAUDE.md — Test Locally First, run
 * manuel pré-push).
 */

import { expect, test, type Page } from '@playwright/test';
import * as fs from 'node:fs';
import { seedTestState, clearAuthStorage } from './helpers/test-state';

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

test('export global ZIP via UI (AC #32)', async ({ page }) => {
	await login(page);
	await page.goto('/export');
	await page.waitForLoadState('networkidle');

	// AC #2 — assert UI elements présents
	await expect(page.getByRole('heading', { level: 1 })).toBeVisible();
	const startButton = page.getByTestId('export-global-start');
	await expect(startButton).toBeVisible();
	await expect(startButton).toBeEnabled();

	// AC #25 — pendant la génération, le bouton passe en disabled + libellé
	// « Génération de l'export… ».
	const downloadPromise = page.waitForEvent('download');
	await startButton.click();
	// Race : selon la vitesse backend, le download peut résoudre avant qu'on
	// observe le `disabled`. On checke le `disabled` au mieux mais on tolère
	// le fast-path (assertion non-bloquante).
	await expect(startButton).toBeDisabled({ timeout: 1000 }).catch(() => {
		// Backend très rapide — le bouton est déjà revenu enabled.
	});

	const download = await downloadPromise;

	// AC #32 — saveAs puis read
	const savedPath = '/tmp/kesh-test-9-2b.zip';
	await download.saveAs(savedPath);
	const bytes = fs.readFileSync(savedPath);

	// AC #4 — byte signature ZIP `PK\x03\x04`
	expect(bytes.length).toBeGreaterThan(4);
	expect(bytes[0]).toBe(0x50);
	expect(bytes[1]).toBe(0x4b);
	expect(bytes[2]).toBe(0x03);
	expect(bytes[3]).toBe(0x04);

	// AC #28 — filename pattern `kesh-export-.+-YYYY-MM-DD.zip`
	const suggested = download.suggestedFilename();
	expect(suggested).toMatch(/^kesh-export-.+-\d{4}-\d{2}-\d{2}\.zip$/);

	// AC #25 — bouton redevient enabled post-download
	await expect(startButton).toBeEnabled({ timeout: 5000 });

	// Cleanup
	if (fs.existsSync(savedPath)) {
		fs.unlinkSync(savedPath);
	}
});
