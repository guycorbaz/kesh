/**
 * Story v014-1 — Tests E2E Playwright pour le CRUD complet des
 * `bank_accounts` post-onboarding sur la page `/bank-accounts`.
 *
 * AC#32 — 7 tests minimum :
 *   1. Création happy path
 *   2. Création IBAN invalide → erreur form
 *   3. Création QR-IBAN invalide → erreur form
 *   4. Édition d'un compte existant
 *   5. Archivage compte sans transactions
 *   6. Tentative archivage primary unique (autorisé AC#10) OU avec transactions (412)
 *   7. Toggle « Afficher les archivés »
 *
 * Pré-requis : MariaDB + seed `with-company` (sans bank_account initial,
 * mais onboarding step 7 complété par le seed).
 */

import { expect, test, type Page } from '@playwright/test';
import { seedTestState, clearAuthStorage, authedApiContext } from './helpers/test-state';

const NEW_IBAN_1 = 'CH9300762011623852957'; // UBS test IBAN (MOD-97 valide)
const NEW_IBAN_2 = 'CH3908704016075473007'; // PostFinance test IBAN (MOD-97 valide)
const INVALID_IBAN = 'CH00INVALID0000';
// Story 26-2a — IBAN distinct de ceux des tests voisins (isolement, AC13).
const DESCRIBED_IBAN = 'CH5604835012345678009';

test.beforeAll(async () => {
	await seedTestState('with-company');
});

test.afterEach(async ({ page }) => {
	await clearAuthStorage(page);
});

async function loginAsAdmin(page: Page): Promise<void> {
	await page.goto('/login');
	await page.fill('#username', 'admin');
	await page.fill('#password', 'admin123');
	await page.click('button[type="submit"]');
	await expect(page).toHaveURL('/');
}

async function goToBankAccounts(page: Page): Promise<void> {
	await loginAsAdmin(page);
	await page.goto('/bank-accounts');
	await expect(page.locator('[data-testid="bank-accounts-page-title"]')).toBeVisible();
}

test.describe('Bank accounts CRUD', () => {
	test('création d\'un compte bancaire happy path', async ({ page }) => {
		await goToBankAccounts(page);

		await page.click('[data-testid="create-bank-account-button"]');
		await expect(page.locator('[data-testid="create-bank-account-form"]')).toBeVisible();

		await page.fill('[data-testid="form-bank-name"]', 'PostFinance E2E');
		await page.fill('[data-testid="form-iban"]', NEW_IBAN_1);
		await page.click('[data-testid="form-submit"]');

		// Toast success + reload de la liste.
		await expect(page.locator('[data-testid="bank-accounts-list"]')).toBeVisible({ timeout: 5000 });
		await expect(page.getByText('PostFinance E2E')).toBeVisible();
	});

	test('création avec IBAN invalide affiche erreur form', async ({ page }) => {
		await goToBankAccounts(page);

		await page.click('[data-testid="create-bank-account-button"]');
		await page.fill('[data-testid="form-bank-name"]', 'Bad Bank');
		await page.fill('[data-testid="form-iban"]', INVALID_IBAN);
		await page.click('[data-testid="form-submit"]');

		// Erreur affichée inline (backend 400 IBAN invalide).
		await expect(page.locator('[data-testid="form-error"]')).toBeVisible();
		await expect(page.locator('[data-testid="form-error"]')).toContainText(/IBAN/i);
	});

	test('création avec QR-IBAN invalide affiche erreur', async ({ page }) => {
		await goToBankAccounts(page);

		await page.click('[data-testid="create-bank-account-button"]');
		await page.fill('[data-testid="form-bank-name"]', 'Bank QR');
		await page.fill('[data-testid="form-iban"]', NEW_IBAN_1);
		await page.fill('[data-testid="form-qr-iban"]', INVALID_IBAN);
		await page.click('[data-testid="form-submit"]');

		await expect(page.locator('[data-testid="form-error"]')).toBeVisible();
	});

	test('édition complète d\'un compte existant', async ({ page }) => {
		await goToBankAccounts(page);

		// D'abord créer un compte.
		await page.click('[data-testid="create-bank-account-button"]');
		await page.fill('[data-testid="form-bank-name"]', 'Original Bank');
		await page.fill('[data-testid="form-iban"]', NEW_IBAN_1);
		await page.click('[data-testid="form-submit"]');
		await expect(page.getByText('Original Bank')).toBeVisible();

		// Trouver l'ID de la row pour ouvrir l'édition.
		const editButton = page.locator('[data-testid^="edit-button-"]').first();
		await editButton.click();
		await expect(page.locator('[data-testid="edit-bank-account-form"]')).toBeVisible();

		// Modifier le nom.
		const editBankName = page.locator('[data-testid="edit-bank-name"]');
		await editBankName.fill('');
		await editBankName.fill('Renamed Bank');
		await page.click('[data-testid="edit-form-submit"]');

		await expect(page.getByText('Renamed Bank')).toBeVisible({ timeout: 5000 });
	});

	test('archivage d\'un compte sans transactions', async ({ page }) => {
		await goToBankAccounts(page);

		// Créer un compte non-primary.
		await page.click('[data-testid="create-bank-account-button"]');
		await page.fill('[data-testid="form-bank-name"]', 'Archive Me');
		await page.fill('[data-testid="form-iban"]', NEW_IBAN_2);
		await page.click('[data-testid="form-submit"]');
		await expect(page.getByText('Archive Me')).toBeVisible();

		// Archiver.
		// KF-049 (#346) — ⚠️ ce sélecteur était `.first()`, c'est-à-dire le premier
		// bouton d'archivage DE LA LISTE et non celui du compte que le test vient
		// de créer. Selon l'ordre de la liste, il archivait un AUTRE compte, puis
		// s'étonnait de voir « Archive Me » toujours là.
		//
		// ⚠️ **Vérifié dans l'application avant de toucher au test**, comme l'issue
		// le demandait : `includeArchived` vaut `false` par défaut et la liste
		// filtre bien (`+page.svelte:31,54`). Le comportement métier est correct —
		// c'était le test qui visait à côté.
		const archiveBtn = page
			.locator('tr', { hasText: 'Archive Me' })
			.locator('[data-testid^="archive-button-"]');
		await archiveBtn.click();
		await expect(page.locator('[data-testid="archive-confirm"]')).toBeVisible();
		await page.click('[data-testid="archive-confirm-button"]');

		// Le compte disparaît de la liste (filtre archived=FALSE par défaut).
		await expect(page.getByText('Archive Me')).not.toBeVisible({ timeout: 5000 });
	});

	test('toggle « Afficher les archivés » montre les comptes archivés', async ({ page }) => {
		await goToBankAccounts(page);

		// Créer + archiver un compte (cf test précédent).
		await page.click('[data-testid="create-bank-account-button"]');
		await page.fill('[data-testid="form-bank-name"]', 'Toggle Test');
		await page.fill('[data-testid="form-iban"]', NEW_IBAN_1);
		await page.click('[data-testid="form-submit"]');
		await expect(page.getByText('Toggle Test')).toBeVisible();

		// KF-049 (#346) — même défaut que le test précédent, et il passait par
		// ACCIDENT : `.first()` visait le premier bouton de la liste, que l'ordre
		// rendait par chance le bon. Corriger le test voisin a suffi à le faire
		// rougir. **Greper le symptôme et non le site** — c'est la règle du dépôt,
		// et elle vient de payer une fois de plus.
		const archiveBtn = page
			.locator('tr', { hasText: 'Toggle Test' })
			.locator('[data-testid^="archive-button-"]');
		await archiveBtn.click();
		await page.click('[data-testid="archive-confirm-button"]');
		await expect(page.getByText('Toggle Test')).not.toBeVisible({ timeout: 5000 });

		// Toggle show archived.
		await page.click('[data-testid="toggle-archived"]');
		await expect(page.getByText('Toggle Test')).toBeVisible({ timeout: 5000 });
		await expect(page.locator('[data-testid^="archived-badge-"]').first()).toBeVisible();
	});

	test('navigation depuis sidebar Administration → Comptes bancaires', async ({ page }) => {
		await loginAsAdmin(page);
		await expect(page).toHaveURL('/');

		const sidebar = page.locator('nav[aria-label="Navigation principale"]');
		// Ouvrir le groupe Administration (défaut fermé).
		await sidebar.locator('[data-testid="nav-group-administration"] summary').click();
		await sidebar.locator('[data-testid="nav-link-bank-accounts"]').click();

		await expect(page).toHaveURL('/bank-accounts');
		await expect(page.locator('[data-testid="bank-accounts-page-title"]')).toBeVisible();
	});
});

/**
 * Story 26-2a (#622, AC13 (a)) — la description d'un compte bancaire, saisie
 * puis effacée sur l'écran *Comptes bancaires*.
 *
 * Isolement (base E2E partagée entre specs) : le compte créé est **non
 * principal**, d'IBAN distinct des tests voisins, et **archivé** en fin de
 * test par l'API — d'autres specs choisissent un compte bancaire par position
 * ou comptent les comptes.
 */
test.describe('Bank accounts — description (Story 26-2a)', () => {
	const createdIds: number[] = [];

	test.afterEach(async ({ page }) => {
		if (createdIds.length === 0) return;
		const api = await authedApiContext(page);
		try {
			const list = await (await api.get('/api/v1/bank-accounts')).json();
			for (const id of createdIds.splice(0)) {
				const ba = (list as Array<{ id: number; version: number }>).find((b) => b.id === id);
				if (!ba) continue;
				const resp = await api.delete(`/api/v1/bank-accounts/${id}`, {
					data: { version: ba.version },
				});
				expect(resp.status(), `archivage du compte ${id}`).toBe(200);
			}
		} finally {
			await api.dispose();
		}
	});

	test('saisir puis effacer la description d’un compte', async ({ page }) => {
		await goToBankAccounts(page);

		await page.click('[data-testid="create-bank-account-button"]');
		const description = page.locator('[data-testid="form-description"]');
		await expect(description).toHaveAttribute('maxlength', '60');
		await page.fill('[data-testid="form-bank-name"]', 'UBS E2E');
		await description.fill('Épargne E2E');
		await page.fill('[data-testid="form-iban"]', DESCRIBED_IBAN);
		await expect(page.locator('[data-testid="form-is-primary"]')).not.toBeChecked();

		const created = page.waitForResponse(
			(r) => r.url().endsWith('/api/v1/bank-accounts') && r.request().method() === 'POST',
		);
		await page.click('[data-testid="form-submit"]');
		const createdResp = await created;
		expect(createdResp.status()).toBe(201);
		const id = (await createdResp.json()).id as number;
		createdIds.push(id);

		const name = page.locator(`[data-testid="bank-account-name-${id}"]`);
		await expect(name).toHaveText('UBS E2E — Épargne E2E', { timeout: 5000 });

		// Modifier en effaçant la description → la banque seule.
		await page.click(`[data-testid="edit-button-${id}"]`);
		const editDescription = page.locator('[data-testid="edit-description"]');
		await expect(editDescription).toHaveValue('Épargne E2E');
		await expect(editDescription).toHaveAttribute('maxlength', '60');
		await editDescription.fill('');
		await page.click('[data-testid="edit-form-submit"]');
		await expect(page.locator('[data-testid="edit-bank-account-form"]')).toHaveCount(0, {
			timeout: 5000,
		});
		await expect(name).toHaveText('UBS E2E');
	});
});

