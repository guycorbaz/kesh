import { expect, test } from '@playwright/test';
import { seedTestState, clearAuthStorage } from './helpers/test-state';

/**
 * Tests E2E — Numéro de client sur la fiche contact (Story 16-3b, #151).
 *
 * Couvre AC5, dont les **deux** preuves sont exigées séparément :
 *
 * 1. **Création** — saisir, enregistrer, recharger, relire. Un E2E est le seul
 *    test qui vérifie qu'une valeur traverse réellement la frontière HTTP :
 *    Vitest teste la construction du payload, les tests Rust la validation, et
 *    ni l'un ni l'autre ne voit une clé qui disparaît entre les deux.
 *
 * 2. **Édition** — et la preuve 1 ne la remplace pas. `PUT /contacts/{id}` est
 *    un **full-replace** : le même payload sert à créer et à modifier, et
 *    `openEdit` hydrate le formulaire champ par champ. Ajouter `clientNumber`
 *    au payload sans ajouter sa ligne d'hydratation ne casse **aucune**
 *    compilation — et modifier un simple téléphone efface le numéro. La preuve
 *    1 reste **verte** sous cette mutation, puisqu'elle ne parcourt que la
 *    création.
 *
 * ⚠️ Le fichier DOIT être nommé `*.spec.ts` : `playwright.config.ts` filtre sur
 * `testMatch: /(.+\.)?spec\.[jt]s/`, et un `*.test.ts` posé ici serait
 * silencieusement ignoré — il ne rougirait jamais, il se tairait.
 */

test.beforeAll(async () => {
	await seedTestState('with-company');
});

test.afterEach(async ({ page }) => {
	await clearAuthStorage(page);
});

async function goToContacts(page: import('@playwright/test').Page) {
	await page.goto('/login');
	await page.fill('#username', 'admin');
	await page.fill('#password', 'admin123');
	await page.click('button[type="submit"]');
	await expect(page).toHaveURL('/');
	await page.goto('/contacts');
	await expect(page).toHaveURL(/\/contacts/);
}

/** Archive le contact pour ne pas polluer les tests suivants ni l'unicité. */
async function archive(page: import('@playwright/test').Page, name: string) {
	const row = page.locator('tr', { hasText: name }).first();
	await row.getByRole('button', { name: /Archiver/ }).click();
	await page.getByRole('dialog').getByRole('button', { name: 'Archiver' }).click();
	await expect(page.getByText(name)).toHaveCount(0, { timeout: 5000 });
}

test.describe('Numéro de client — fiche contact', () => {
	test('saisie à la création, puis relecture après rechargement', async ({ page }) => {
		await goToContacts(page);

		const stamp = Date.now();
		const name = `TestContact CN ${stamp}`;
		const clientNumber = `CLI-${stamp}`;

		await page.getByRole('button', { name: /Nouveau contact/ }).click();
		await page.fill('#form-name', name);
		await page.fill('#form-client-number', clientNumber);
		await page.getByRole('button', { name: 'Créer' }).click();
		await expect(page.getByText(name)).toBeVisible({ timeout: 5000 });

		// Rechargement complet : la valeur doit venir du serveur, pas de l'état
		// du formulaire resté en mémoire.
		await page.reload();
		await expect(page).toHaveURL(/\/contacts/);

		const row = page.locator('tr', { hasText: name }).first();
		await row.getByRole('button', { name: /Modifier/ }).click();
		await expect(page.locator('#form-client-number')).toHaveValue(clientNumber);

		await page.getByRole('button', { name: 'Annuler' }).click();
		await archive(page, name);
	});

	test('le numéro SURVIT à la modification d’un champ sans rapport', async ({ page }) => {
		await goToContacts(page);

		const stamp = Date.now();
		const name = `TestContact CN Edit ${stamp}`;
		const clientNumber = `CLI-EDIT-${stamp}`;

		await page.getByRole('button', { name: /Nouveau contact/ }).click();
		await page.fill('#form-name', name);
		await page.fill('#form-client-number', clientNumber);
		await page.getByRole('button', { name: 'Créer' }).click();
		await expect(page.getByText(name)).toBeVisible({ timeout: 5000 });

		// On ne touche QUE le téléphone.
		const row = page.locator('tr', { hasText: name }).first();
		await row.getByRole('button', { name: /Modifier/ }).click();
		await page.fill('#form-phone', '021 555 00 11');
		await page.getByRole('button', { name: 'Enregistrer' }).click();
		await expect(page.getByRole('dialog')).toHaveCount(0, { timeout: 5000 });

		await page.reload();
		await expect(page).toHaveURL(/\/contacts/);

		const rowAfter = page.locator('tr', { hasText: name }).first();
		await rowAfter.getByRole('button', { name: /Modifier/ }).click();
		await expect(page.locator('#form-client-number')).toHaveValue(clientNumber);
		await expect(page.locator('#form-phone')).toHaveValue('021 555 00 11');

		await page.getByRole('button', { name: 'Annuler' }).click();
		await archive(page, name);
	});
});
