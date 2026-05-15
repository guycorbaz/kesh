/**
 * Story 9-2a — Test Playwright E2E : export PDF du bilan via UI.
 *
 * Scénario AC #35 (Pass 1 ECH-M5) :
 *   1. Login admin.
 *   2. Navigation `/reports`.
 *   3. Cliquer « Générer » (Bilan, preset with-company sans écritures → empty
 *      report mais flow complet jusqu'au DTO).
 *   4. Cliquer « Export PDF ».
 *   5. `await page.waitForEvent('download')` → `await download.saveAs(...)`.
 *   6. `fs.readFile(filepath)` → assert byte signature `%PDF-1.` + filename pattern.
 *
 * Pattern download : `download.saveAs(...)` puis `fs.readFile()` — PAS
 * `download.path()` qui peut retourner `null` (Pass 1 ECH-M5).
 *
 * Pré-requis : MariaDB up + KESH_TEST_MODE=true + seed `with-company` +
 * Playwright browsers (Ubuntu 26.04+ : PLAYWRIGHT_HOST_PLATFORM_OVERRIDE=ubuntu24.04-x64).
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

test('export PDF balance-sheet via UI (AC #35, T12.1)', async ({ page }) => {
	await login(page);
	await page.goto('/reports');
	await page.waitForLoadState('networkidle');

	// Étape 1 : générer le bilan (déclenche l'apparition du tabpanel avec un DTO)
	const generateButton = page.getByRole('button', { name: /générer/i });
	await expect(generateButton).toBeEnabled();
	await generateButton.click();

	// Attendre que le rapport soit prêt — le tabpanel doit afficher un contenu.
	const tabpanel = page.getByRole('tabpanel');
	await expect(tabpanel).toBeVisible();

	// Le bouton Export PDF doit être visible et activé une fois le rapport généré.
	const exportPdfButton = page.getByRole('button', { name: /export pdf/i });
	await expect(exportPdfButton).toBeVisible();
	await expect(exportPdfButton).toBeEnabled({ timeout: 10000 });

	// Étape 2 : cliquer et capturer l'event download
	const downloadPromise = page.waitForEvent('download');
	await exportPdfButton.click();
	const download = await downloadPromise;

	// Étape 3 : sauver dans /tmp puis lire les bytes
	const savedPath = '/tmp/kesh-test-9-2a.pdf';
	await download.saveAs(savedPath);
	const bytes = fs.readFileSync(savedPath);

	// Assertion 1 : byte signature PDF
	const header = bytes.subarray(0, 8).toString('latin1');
	expect(header.startsWith('%PDF-1.')).toBe(true);

	// Assertion 2 : filename pattern `kesh-bilan-...-YYYY-MM-DD_YYYY-MM-DD.pdf`
	const suggested = download.suggestedFilename();
	expect(suggested).toMatch(
		/^kesh-bilan-.*-\d{4}-\d{2}-\d{2}_\d{4}-\d{2}-\d{2}\.pdf$/,
	);

	// Cleanup
	if (fs.existsSync(savedPath)) {
		fs.unlinkSync(savedPath);
	}
});
