/**
 * Story 8-3 T9.1 — Tests E2E Playwright pour les nouveaux flags
 * confirm de l'import bancaire (FR43 doublons + FR51 partial commit
 * + KF #70 frontend wiring).
 *
 * 5 scénarios :
 *   1. duplicate file warning shows panel and accepts override (AC #22)
 *   2. duplicate lines warning shows panel with skip-or-import radio (AC #23)
 *   3. csv partial failure shows panel and accepts partial commit (AC #24)
 *   4. csv encoding mismatch confirm flow end-to-end (AC #20 — KF #70)
 *   5. accessibility — bank import preview with warnings axe scan zero violations (AC #28)
 *
 * Pré-requis :
 *   - MariaDB up + KESH_TEST_MODE=true sur le backend.
 *   - Au moins un bank_account configuré (via onboarding).
 *   - Pour les scénarios CSV : un bank_profile créé.
 *   - Playwright browsers installés
 *     (Ubuntu 26.04 : `PLAYWRIGHT_HOST_PLATFORM_OVERRIDE=ubuntu24.04-x64`).
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
const FIXTURE_OVERLAP = path.join(FIXTURE_DIR, 'camt053_v04_overlap.xml');
const FIXTURE_CSV_PARTIAL = path.join(FIXTURE_DIR, 'csv_partial_failure.csv');

const TEST_IBAN = 'CH4431999123000889012';

// ⚠️ `beforeEach`, et NON `beforeAll` : les tests de cette spec importent tous
// les mêmes fixtures, donc chacun laisse en base de quoi faire dévier le
// suivant — un fichier déjà vu, des lignes déjà vues. Avec un seed par spec,
// l'échec dépendait de l'ORDRE et se déplaçait de test en test à chaque
// correctif ; c'est la « base de gate piégée » du CLAUDE.md, transposée aux
// E2E. Un seed par TEST supprime la classe entière plutôt que ses symptômes
// (issue #107, KF-030).
test.beforeEach(async () => {
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

async function uploadFile(page: Page, filePath: string): Promise<void> {
	await page.getByTestId('bank-account-select').selectOption({ index: 1 });
	await page.getByTestId('bank-import-file-input').setInputFiles(filePath);
}

test('duplicate file warning shows panel and accepts override', async ({ page }) => {
	await login(page);
	await ensureBankAccount(page);

	// Premier upload — happy path, persiste l'import.
	await page.goto('/bank-import');
	await uploadFile(page, FIXTURE_MINIMAL);
	await expect(page.getByTestId('bank-import-preview')).toBeVisible();
	await page.getByTestId('bank-import-confirm').click();
	await expect(page.getByTestId('bank-import-preview')).toBeHidden();

	// Second upload du même fichier — preview retourne duplicateFile.
	await uploadFile(page, FIXTURE_MINIMAL);
	await expect(page.getByTestId('warning-duplicate-file')).toBeVisible();
	await expect(page.getByTestId('warning-duplicate-file-existing-link')).toBeVisible();

	// Bouton confirm désactivé tant que la checkbox n'est pas cochée.
	await expect(page.getByTestId('bank-import-confirm')).toBeDisabled();
	await page.getByTestId('confirm-duplicate-file').check();
	await expect(page.getByTestId('bank-import-confirm')).toBeEnabled();
	await page.getByTestId('bank-import-confirm').click();
	await expect(page.getByTestId('bank-import-preview')).toBeHidden();
});

test('duplicate lines warning shows panel with skip-or-import radio', async ({ page }) => {
	await login(page);
	await ensureBankAccount(page);

	// Ce test a besoin d'un ÉTAT : une transaction en base, que le second upload
	// viendra chevaucher. Il ne teste pas la primeur du fichier.
	//
	// ⚠️ Sur la base PARTAGÉE, cet état peut déjà exister — une spec précédente a
	// pu importer la même fixture. Le preview porte alors un avertissement de
	// fichier déjà importé, et le bouton de confirmation reste désactivé tant
	// qu'on ne lève pas DEUX gardes (le fichier, puis ses lignes). Forcer ce
	// passage ferait exactement ce que le second upload doit mesurer.
	//
	// On constate donc l'état au lieu de le forcer : si la fixture est déjà en
	// base, on annule et on passe à la suite. Sans cela, l'échec dépend de
	// l'ORDRE des tests et se déplace d'un run à l'autre (issue #107).
	await page.goto('/bank-import');
	await uploadFile(page, FIXTURE_MINIMAL);
	// ⚠️ DEUX signaux, pas un : selon ce qu'une spec précédente a importé, le
	// doublon se présente comme fichier déjà vu OU comme lignes déjà vues. Ne
	// guetter que le premier laissait le bouton désactivé sur le second, et le
	// clic partait en timeout.
	const vu = async (id: string) =>
		await page
			.getByTestId(id)
			.isVisible()
			.catch(() => false);
	const dejaEnBase = (await vu('confirm-duplicate-file')) || (await vu('confirm-duplicate-lines-skip'));
	if (dejaEnBase) {
		await page.getByTestId('bank-import-cancel').click();
	} else {
		await page.getByTestId('bank-import-confirm').click();
	}
	await expect(page.getByTestId('bank-import-preview')).toBeHidden();

	// Second upload — overlap.xml, hash distinct mais 1 transaction
	// matchant la clé composite stable → duplicateLines warning.
	await uploadFile(page, FIXTURE_OVERLAP);
	await expect(page.getByTestId('warning-duplicate-lines')).toBeVisible();

	// Le radio group propose skip (default coché) + import.
	await expect(page.getByTestId('confirm-duplicate-lines-skip')).toBeChecked();
	await page.getByTestId('confirm-duplicate-lines-import').check();
	await expect(page.getByTestId('confirm-duplicate-lines-import')).toBeChecked();

	await page.getByTestId('bank-import-confirm').click();
	await expect(page.getByTestId('bank-import-preview')).toBeHidden();
});

test('csv partial failure shows panel and accepts partial commit', async ({ page }) => {
	test.skip(
		true,
		"Skipped : nécessite la création d'un bank_profile CSV via API + onboarding " +
			'spécifique. Couvert par les tests E2E HTTP backend `post_import_csv_accepts_partial_with_confirm`.',
	);
	// Implémentation référence (à activer quand le helper de seeding bank_profile existe) :
	await login(page);
	await ensureBankAccount(page);
	await page.goto('/bank-import');
	await uploadFile(page, FIXTURE_CSV_PARTIAL);

	await expect(page.getByTestId('warning-invalid-lines')).toBeVisible();
	await expect(page.getByTestId('bank-import-confirm')).toBeDisabled();
	await page.getByTestId('confirm-partial-import').check();
	await expect(page.getByTestId('bank-import-confirm')).toBeEnabled();
	await page.getByTestId('bank-import-confirm').click();
	await expect(page.getByTestId('bank-import-preview')).toBeHidden();
});

test('csv encoding mismatch confirm flow end-to-end', async ({ page }) => {
	test.skip(
		true,
		"Skipped : nécessite un bank_profile avec `encoding=ISO-8859-1` et un fichier UTF-8 — " +
			"setup non trivial sans helper de profile-seed. Couvert par tests unitaires Vitest.",
	);
	// Référence : flow attendu
	await login(page);
	await ensureBankAccount(page);
	await page.goto('/bank-import');
	await expect(page.getByTestId('warning-encoding-mismatch')).toBeVisible();
	await page.getByTestId('confirm-encoding-mismatch').check();
	await page.getByTestId('bank-import-confirm').click();
});

test('accessibility — bank import preview with warnings axe scan zero violations', async ({
	page,
}) => {
	await login(page);
	await ensureBankAccount(page);
	await page.goto('/bank-import');

	// Premier import pour seed.
	await uploadFile(page, FIXTURE_MINIMAL);
	await page.getByTestId('bank-import-confirm').click();
	await expect(page.getByTestId('bank-import-preview')).toBeHidden();

	// Re-upload pour déclencher le warning duplicateFile.
	await uploadFile(page, FIXTURE_MINIMAL);
	await expect(page.getByTestId('warning-duplicate-file')).toBeVisible();

	const axeResult = await new AxeBuilder({ page })
		.exclude('[data-testid="bank-import-list-row"]')
		.analyze();
	expect(axeResult.violations).toEqual([]);
});
