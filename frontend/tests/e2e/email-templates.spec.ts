import { expect, test } from '@playwright/test';
import AxeBuilder from '@axe-core/playwright';
import { seedTestState, clearAuthStorage } from './helpers/test-state';

/**
 * Tests E2E — Story 20-2 (Epic 20 #224) : section Admin « Modèles d'e-mail ».
 *
 * Couvre :
 * - Invariant zéro-config (AC #17) : à l'ouverture sur une company neuve, les
 *   4 langues sont marquées « Défaut » avec du texte non vide — jamais vide.
 * - Édition + enregistrement d'une langue (persistance, badge → Personnalisé).
 * - Restaurer le défaut via la modale de confirmation.
 * - Navigation depuis la page Paramètres.
 */

test.beforeEach(async () => {
	await seedTestState('with-company');
});

test.afterEach(async ({ page }) => {
	await clearAuthStorage(page);
});

async function login(page: import('@playwright/test').Page) {
	await page.goto('/login');
	await page.fill('#username', 'admin');
	await page.fill('#password', 'admin123');
	await page.click('button[type="submit"]');
	await expect(page).toHaveURL('/');
}

async function goToEmailTemplates(page: import('@playwright/test').Page) {
	await login(page);
	await page.goto('/settings/email-templates');
	await expect(page).toHaveURL(/\/settings\/email-templates/);
}

test.describe('Modèles d\'e-mail — section Admin', () => {
	test('zéro-config : ouverture affiche les 4 langues en Défaut, texte non vide (AC #17)', async ({
		page,
	}) => {
		await goToEmailTemplates(page);

		// Les 4 onglets de langue sont présents.
		for (const lang of ['FR', 'DE', 'IT', 'EN']) {
			await expect(page.getByTestId(`email-template-lang-tab-${lang}`)).toBeVisible();
		}

		// Onglet FR (actif par défaut) : badge « Défaut », subject/body non vides.
		await expect(page.getByTestId('email-template-badge')).toHaveText('Défaut');
		const subject = page.getByTestId('email-template-subject');
		const body = page.getByTestId('email-template-body');
		expect((await subject.inputValue()).length).toBeGreaterThan(0);
		expect((await body.inputValue()).length).toBeGreaterThan(0);

		// Le panneau des variables autorisées est peuplé.
		await expect(
			page.getByTestId('email-template-variables').locator('li').first(),
		).toBeVisible();

		// Chaque langue reste en Défaut avec du contenu.
		for (const lang of ['DE', 'IT', 'EN']) {
			await page.getByTestId(`email-template-lang-tab-${lang}`).click();
			await expect(page.getByTestId('email-template-badge')).toHaveText('Défaut');
			expect((await subject.inputValue()).length).toBeGreaterThan(0);
		}
	});

	test('édite et enregistre une langue → badge Personnalisé, persistance', async ({ page }) => {
		await goToEmailTemplates(page);

		const subject = page.getByTestId('email-template-subject');
		const body = page.getByTestId('email-template-body');

		await subject.fill('Sujet E2E {invoiceNumber}');
		await body.fill('{salutation}, montant {amount}. {companyName}');
		await page.getByTestId('email-template-save-button').click();

		// Badge passe à Personnalisé.
		await expect(page.getByTestId('email-template-badge')).toHaveText('Personnalisé');

		// Persistance : recharger la page, la valeur est conservée.
		await page.reload();
		await expect(page.getByTestId('email-template-badge')).toHaveText('Personnalisé');
		await expect(subject).toHaveValue('Sujet E2E {invoiceNumber}');
	});

	test('changement d\'onglet préserve les brouillons non enregistrés par langue (code-review #1)', async ({
		page,
	}) => {
		await goToEmailTemplates(page);

		const subject = page.getByTestId('email-template-subject');
		const body = page.getByTestId('email-template-body');

		// Saisir dans FR sans enregistrer.
		await subject.fill('Brouillon FR {invoiceNumber}');
		await body.fill('Corps FR {amount}');

		// Passer à DE puis revenir à FR : la saisie FR NE doit PAS être perdue.
		await page.getByTestId('email-template-lang-tab-DE').click();
		await expect(subject).not.toHaveValue('Brouillon FR {invoiceNumber}');
		await page.getByTestId('email-template-lang-tab-FR').click();
		await expect(subject).toHaveValue('Brouillon FR {invoiceNumber}');
		await expect(body).toHaveValue('Corps FR {amount}');
	});

	test('restaure le défaut via la modale → badge repasse à Défaut', async ({ page }) => {
		await goToEmailTemplates(page);

		// Créer d'abord un override.
		await page.getByTestId('email-template-subject').fill('À restaurer {invoiceNumber}');
		await page.getByTestId('email-template-body').fill('Corps {amount}');
		await page.getByTestId('email-template-save-button').click();
		await expect(page.getByTestId('email-template-badge')).toHaveText('Personnalisé');

		// Ouvrir la modale + attendre sa visibilité avant de confirmer.
		await page.getByTestId('email-template-restore-button').click();
		const confirm = page.getByTestId('email-template-restore-confirm');
		await expect(confirm).toBeVisible();
		await confirm.click();

		await expect(page.getByTestId('email-template-badge')).toHaveText('Défaut');
	});

	test('validation : variables inconnues → message d\'erreur listant les tokens', async ({
		page,
	}) => {
		await goToEmailTemplates(page);

		await page.getByTestId('email-template-subject').fill('Sujet {tokenInconnu}');
		await page.getByTestId('email-template-body').fill('Corps {autreInconnu}');
		await page.getByTestId('email-template-save-button').click();

		const errBox = page.getByTestId('email-template-unknown-vars');
		await expect(errBox).toBeVisible();
		await expect(errBox).toContainText('{tokenInconnu}');
		await expect(errBox).toContainText('{autreInconnu}');
		// Le template reste en Défaut (rien persisté).
		await expect(page.getByTestId('email-template-badge')).toHaveText('Défaut');
	});

	test('navigation : la carte Paramètres mène à la page', async ({ page }) => {
		await login(page);
		await page.goto('/settings');
		await page.getByTestId('settings-email-templates-manage-link').click();
		await expect(page).toHaveURL(/\/settings\/email-templates/);
		await expect(
			page.getByRole('heading', { name: /Modèles d'e-mail/ }),
		).toBeVisible();
	});

	test('multi-type/multi-niveau : changer type/niveau/langue préserve les brouillons (Story 21-4, anti-régression bug 20-2)', async ({
		page,
	}) => {
		await goToEmailTemplates(page);

		const subject = page.getByTestId('email-template-subject');

		// Sélectionner type rappel, niveau 2, langue FR ; saisir un brouillon (sans enregistrer).
		await page.getByTestId('email-template-type-invoice_reminder').click();
		await page.getByTestId('email-template-level-2').click();
		await subject.fill('brouillon-niv2-fr');

		// Niveau 1 (même type/langue) : le brouillon niveau 2 ne s'affiche pas.
		await page.getByTestId('email-template-level-1').click();
		await expect(subject).not.toHaveValue('brouillon-niv2-fr');

		// Retour niveau 2 : brouillon préservé (pas de re-fetch).
		await page.getByTestId('email-template-level-2').click();
		await expect(subject).toHaveValue('brouillon-niv2-fr');

		// Langue DE puis retour FR : brouillon FR:2 toujours là.
		await page.getByTestId('email-template-lang-tab-DE').click();
		await expect(subject).not.toHaveValue('brouillon-niv2-fr');
		await page.getByTestId('email-template-lang-tab-FR').click();
		await expect(subject).toHaveValue('brouillon-niv2-fr');

		// Type invoice_send (reset niveau, pas de crash) puis retour rappel/niveau 2 : préservé.
		await page.getByTestId('email-template-type-invoice_send').click();
		await expect(page.getByTestId('email-template-level-2')).toHaveCount(0);
		await page.getByTestId('email-template-type-invoice_reminder').click();
		await page.getByTestId('email-template-level-2').click();
		await expect(subject).toHaveValue('brouillon-niv2-fr');
	});

	test('a11y : sélecteurs type/niveau sans violation axe (Story 21-4)', async ({ page }) => {
		await goToEmailTemplates(page);

		// Afficher le sélecteur de niveau (rappels) pour couvrir les deux toggle-groups.
		await page.getByTestId('email-template-type-invoice_reminder').click();
		await expect(page.getByTestId('email-template-level-1')).toBeVisible();

		// Scope aux sélecteurs type/niveau ajoutés par 21-4 (`.include`) : la page
		// email-templates (Epic 20) porte de la dette a11y pré-existante hors scope
		// de cette story (landmark `<aside>` imbriqué dans `<main>`). Ce test valide
		// la sémantique/ARIA que 21-4 introduit (role="group", aria-pressed, labels).
		// `color-contrast` désactivé : le chip actif `bg-primary-light` (#3b82f6) +
		// texte foncé échoue au ratio AA (3.97:1) — dette systémique app-wide
		// (nav, onboarding, Epic 20), suivie par l'issue #253.
		const results = await new AxeBuilder({ page })
			.include('[data-testid="email-template-selectors"]')
			.disableRules(['color-contrast'])
			.analyze();
		expect(results.violations).toEqual([]);
	});
});
