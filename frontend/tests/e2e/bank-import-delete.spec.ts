/**
 * Story 27-1 (#632) — supprimer un import bancaire dont aucune transaction
 * n'est rapprochée, puis réimporter le même fichier.
 *
 * Un scénario : profil CSV monté par l'API, import par l'écran, suppression
 * depuis le détail, réimport du même fichier sans l'avertissement « fichier
 * déjà importé ». La fixture `csv_e2e_delete_import.csv` est SYNTHÉTIQUE.
 *
 * Le montage (`login`, `mount`, `importByScreen`) vit dans
 * `helpers/bank-import-fixtures.ts` depuis la Story 28-1 (#640), qui le partage
 * avec `destructive-buttons.spec.ts` ; le scénario n'a pas changé.
 *
 * Pré-requis : cf. `docs/testing.md` § « Prérequis Playwright local ».
 */

import { expect, test } from '@playwright/test';
import { seedTestState, clearAuthStorage } from './helpers/test-state';
import {
	DELETE_IMPORT_FILENAME as FILENAME,
	importByScreen,
	login,
	mount,
} from './helpers/bank-import-fixtures';

test.beforeAll(async () => {
	await seedTestState('with-company');
});

test.afterEach(async ({ page }) => {
	await clearAuthStorage(page);
});

test('supprimer un import non rapproché puis réimporter le même fichier', async ({ page }) => {
	await login(page);
	await mount(page);

	// (2) Import par l'écran.
	await importByScreen(page);
	await page.getByTestId('bank-import-confirm').click();
	await expect(page.getByTestId('bank-import-preview')).toBeHidden();
	const row = page.getByTestId('bank-import-list-row').filter({ hasText: FILENAME });
	await expect(row).toHaveCount(1);
	await row.getByTestId('bank-import-list-link').click();
	await expect(page).toHaveURL(/\/bank-import\/\d+$/);

	// (3) Suppression depuis le détail.
	await page.getByTestId('bank-import-delete-button').click();
	await expect(page.getByTestId('bank-import-delete-dialog')).toBeVisible();
	const body = page.getByTestId('bank-import-delete-dialog-body');
	await expect(body).toContainText(FILENAME);
	// Le nombre de transactions, pas un « 3 » de la période (2026-03-…).
	await expect(body).toContainText(/ses 3 transaction\(s\)/);
	await page.getByTestId('bank-import-delete-submit').click();
	await expect(page).toHaveURL(/\/bank-import$/);
	await expect(page.getByTestId('bank-import-list')).toBeVisible();
	await expect(
		page.getByTestId('bank-import-list-row').filter({ hasText: FILENAME }),
	).toHaveCount(0);

	// (4) Le même fichier : plus d'avertissement de doublon, et il se réimporte.
	await importByScreen(page);
	await expect(page.getByTestId('warning-duplicate-file')).toHaveCount(0);
	await page.getByTestId('bank-import-confirm').click();
	await expect(page.getByTestId('bank-import-preview')).toBeHidden();
	const again = page.getByTestId('bank-import-list-row').filter({ hasText: FILENAME });
	await expect(again).toHaveCount(1);
	await again.getByTestId('bank-import-list-link').click();
	await expect(page.getByTestId('detail-tx-row')).toHaveCount(3);
});
