import { test, expect } from '@playwright/test';
import { seedTestState, clearAuthStorage } from './helpers/test-state';

/**
 * Tests E2E — Flux d'onboarding Chemin A (Story 2.2)
 *
 * Prérequis backend (Story 6.4) : `KESH_TEST_MODE=true`. Le `beforeEach`
 * seed le preset `fresh` (uniquement user `changeme/changeme`, aucune
 * company, aucun onboarding_state) → chaque test démarre d'un onboarding
 * vierge, évitant la mutation irréversible du singleton `onboarding_state`.
 */

test.describe('Onboarding Wizard', () => {
	test.beforeEach(async ({ page }) => {
		// Reset DB + user `changeme` seul (preset fresh, cf. AC #7).
		await seedTestState('fresh');

		// Login en tant que changeme/changeme (le seul user du preset fresh).
		await page.goto('/login');
		await page.fill('#username', 'changeme');
		await page.fill('#password', 'changeme');
		await page.click('button[type="submit"]');

		// Le guard onboarding devrait rediriger vers /onboarding
		await expect(page).toHaveURL(/\/onboarding/);
	});

	test.afterEach(async ({ page }) => {
		// Clear localStorage after each test to prevent token bleed to next test
		await clearAuthStorage(page);
	});

	test('étape 1 — choix de langue affiche 4 options', async ({ page }) => {
		// 4 boutons de langue
		const langButtons = page.locator('button:has-text("Français"), button:has-text("Deutsch"), button:has-text("Italiano"), button:has-text("English")');
		await expect(langButtons).toHaveCount(4);
	});

	test('flux complet : langue → mode → démo → bannière visible', async ({ page }) => {
		// Étape 1 : Choisir français
		await page.click('button:has-text("Français")');

		// Étape 2 : Choisir mode guidé
		await expect(page.getByText('Guidé')).toBeVisible();
		await expect(page.getByText('Expert')).toBeVisible();
		await page.click('button:has-text("Guidé")');

		// Étape 3 : Choisir démo
		await expect(page.getByText('Explorer avec des données de démo')).toBeVisible();
		await page.click('button:has-text("Explorer avec des données de démo")');

		// Redirect vers / avec bannière démo
		await expect(page).toHaveURL('/');
		await expect(page.getByText('Instance de démonstration')).toBeVisible();
	});

	test('bannière démo — reset redirige vers onboarding', async ({ page }) => {
		// Complete onboarding first
		await page.click('button:has-text("Français")');
		await page.click('button:has-text("Guidé")');
		await page.click('button:has-text("Explorer avec des données de démo")');
		await expect(page).toHaveURL('/');

		// Click reset
		await page.click('button:has-text("Réinitialiser pour la production")');

		// Confirm dialog
		await expect(page.getByText('Toutes les données de démonstration')).toBeVisible();
		await page.click('button:has-text("Confirmer")');

		// Should redirect to onboarding
		await expect(page).toHaveURL(/\/onboarding/);
	});
});

test.describe('Onboarding — Reprise après interruption', () => {
	test('F5 à étape 2 reprend à étape 2', async ({ page }) => {
		// Login
		await page.goto('/login');
		await page.fill('#username', 'admin');
		await page.fill('#password', 'changeme');
		await page.click('button[type="submit"]');
		await expect(page).toHaveURL(/\/onboarding/);

		// Complete step 1
		await page.click('button:has-text("Français")');

		// Should be at step 2 now
		await expect(page.getByText('Guidé')).toBeVisible();

		// Simulate refresh (F5)
		await page.reload();

		// Should still be at step 2
		await expect(page.getByText('Guidé')).toBeVisible();
	});
});

test.describe('Onboarding — Story 2.6: Invoice Settings Pre-fill (AC 5-6)', () => {
	test.beforeEach(async ({ page }) => {
		// Reset DB + user `changeme` seul (preset fresh)
		await seedTestState('fresh');

		// Login en tant que changeme/changeme
		await page.goto('/login');
		await page.fill('#username', 'changeme');
		await page.fill('#password', 'changeme');
		await page.click('button[type="submit"]');

		// Le guard onboarding devrait rediriger vers /onboarding
		await expect(page).toHaveURL(/\/onboarding/);
	});

	test.afterEach(async ({ page }) => {
		// Clear localStorage after each test
		await clearAuthStorage(page);
	});

	test('AC 5: Path A (démo) — comptes de facturation pré-remplis automatiquement', async ({ page }) => {
		// Step 1: Choisir français
		await page.click('button:has-text("Français")');

		// Step 2: Mode guidé
		await page.click('button:has-text("Guidé")');

		// Step 3: Chemin démo
		await page.click('button:has-text("Explorer avec des données de démo")');

		// Attendre la redirection vers /
		await expect(page).toHaveURL('/');

		// Vérifier que la bannière de démo est visible
		await expect(page.getByText('Instance de démonstration')).toBeVisible();

		// Naviguer vers creation de facture
		await page.goto('/invoices/create');

		// Vérifier que le formulaire de création est accessible
		await expect(page.locator('label:has-text("Contact")')).toBeVisible();

		// Vérifier que la bannière d'avertissement n'est PAS visible
		// (car les comptes sont pré-remplis en mode démo)
		const warningBanner = page.locator('text=Configuration incomplète');
		// NOTE: Si la bannière est visible, cela signifie que la pré-remplissage a échoué
		// Nous ne pouvons pas utiliser toBeVisible() car le selecteur pourrait trouver d'autres textes
		const count = await warningBanner.count();
		if (count > 0) {
			// Si la bannière existe, elle ne doit pas être dans le formulaire de création
			const formWarning = page.locator('form >> text=Configuration incomplète');
			await expect(formWarning).not.toBeVisible();
		}

		// Vérifier que le bouton Créer la facture est activé
		const createBtn = page.locator('button:has-text("Créer la facture")');
		await expect(createBtn).toBeEnabled();
	});

	test('AC 6: Path B (production) — comptes de facturation pré-remplis après onboarding', async ({ page }) => {
		// Step 1: Language
		await page.click('button:has-text("Français")');

		// Step 2: Mode
		await page.click('button:has-text("Guidé")');

		// Step 3: Production path
		await page.click('button:has-text("Configurer pour la production")');

		// Step 4: Org type — Indépendant
		await page.click('button:has-text("Indépendant")');

		// Step 5: Accounting language
		await page.click('button:has-text("Français")');

		// Step 6: Coordinates
		await page.fill('#coord-name', 'Mon Business Indépendant');
		await page.fill('#coord-address', 'Rue des Alpes 1, 1200 Genève');
		await page.click('button:has-text("Continuer")');

		// Step 7: Bank account (skip for now)
		await page.click('button:has-text("Configurer plus tard")');

		// Attendre la redirection vers /
		await expect(page).toHaveURL('/');

		// Vérifier que la bannière "Configuration incomplète" est visible
		// (pour la banque, pas pour les comptes de facturation)
		await expect(page.getByText('Configuration incomplète')).toBeVisible();

		// Naviguer vers creation de facture
		await page.goto('/invoices/create');

		// Vérifier que le formulaire de création est accessible
		await expect(page.locator('label:has-text("Contact")')).toBeVisible();

		// Vérifier que le bouton Créer la facture est activé
		// (car les comptes de facturation ont été pré-remplis)
		const createBtn = page.locator('button:has-text("Créer la facture")');
		await expect(createBtn).toBeEnabled();

		// Bonus: Vérifier que la bannière de configuration des comptes
		// ne s'affiche PAS (car ils sont pré-remplis)
		// Note: Nous recherchons spécifiquement dans le formulaire
		const formWarning = page.locator('form >> div:has-text("Configurez les comptes de facturation")');
		await expect(formWarning).not.toBeVisible();
	});
});
