import { expect, test, type Page } from '@playwright/test';
import { authedApiContext, clearAuthStorage, disposeContextSafe, seedTestState } from './helpers/test-state';

/**
 * Tests E2E — sondes anti-doublon à la saisie d'un contact (Story 22-2b, #301).
 *
 * ⚠️ **LE PRESET NE SEEDE AUCUN CONTACT.** `with-company` n'appelle que
 * `seed_accounting_company` + `mark_onboarding_complete` ; le seul
 * `INSERT INTO contacts` du dépôt est réservé au preset `with-data`. Chaque test
 * monte donc **son propre** porteur — sans quoi les assertions (i) seraient
 * insatisfiables, la table étant vide, et partiraient rouges pour une raison
 * SANS RAPPORT avec ce qu'elles mesurent.
 *
 * ⚠️ **L'ORDRE DES TROIS ASSERTIONS N'EST PAS DÉCORATIF.** Les assertions (ii)
 * et (iii) sont **VERTES SUR `main`** : créer un contact fonctionne déjà, et un
 * IDE dupliqué donne déjà `409`. Sans (i) — la VISIBILITÉ du signal — la
 * mutation « n'implémenter aucune sonde » laisserait les deux tests au vert.
 *
 * ⚠️ **Archiver ne libère PAS l'IDE** (contrainte plate, contrairement au numéro
 * de client). Chaque test emploie donc un IDE **distinct** à checksum valide, et
 * l'isolation entre exécutions repose sur le `truncate_all` de `seedTestState`,
 * jamais sur un nettoyage de fin de test.
 *
 * ⚠️ Le fichier DOIT être nommé `*.spec.ts` : `playwright.config.ts` filtre sur
 * `testMatch: /(.+\.)?spec\.[jt]s/`, et un `*.test.ts` posé ici serait
 * silencieusement ignoré — il ne rougirait jamais, il se tairait.
 */

// Deux IDE distincts, checksums recalculés (poids [5,4,3,2,7,6,5,4], mod 11) :
// 109322551 → clé attendue 1, portée 1 ✓ ; 123456788 → 8, portée 8 ✓.
const IDE_TEST_1 = 'CHE-109.322.551';
const IDE_TEST_2 = 'CHE-123.456.788';

test.beforeAll(async () => {
	await seedTestState('with-company');
});

test.afterEach(async ({ page }) => {
	await clearAuthStorage(page);
});

async function login(page: Page) {
	await page.goto('/login');
	await page.fill('#username', 'admin');
	await page.fill('#password', 'admin123');
	await page.click('button[type="submit"]');
	await expect(page).toHaveURL('/');
}

/** Monte un contact via l'API — le preset n'en seede aucun. */
async function creerContact(page: Page, name: string, ideNumber: string | null) {
	const ctx = await authedApiContext(page);
	try {
		const res = await ctx.post('/api/v1/contacts', {
			data: {
				contactType: 'Entreprise',
				name,
				isClient: true,
				isSupplier: false,
				addressStructured: { street: '', building: '', postalCode: '', city: '', country: 'CH' },
				ideNumber
			}
		});
		expect(res.status(), `montage : création de ${name}`).toBe(201);
	} finally {
		await disposeContextSafe(ctx);
	}
}

async function ouvrirNouveauContact(page: Page) {
	await page.goto('/contacts');
	await expect(page).toHaveURL(/\/contacts/);
	await page.getByRole('button', { name: /Nouveau contact/i }).click();
	await expect(page.locator('#form-name')).toBeVisible();
}

test.describe('Sondes anti-doublon — la saisie d’un contact', () => {
	test('signal NUANCÉ : il prévient, il ne bloque pas, et le contact est créé', async ({ page }) => {
		await login(page);
		// (0) MONTAGE — sans lui, rien ne peut être « proche ».
		await creerContact(page, 'Dubarde Vins SA', IDE_TEST_1);

		await ouvrirNouveauContact(page);
		await page.fill('#form-name', 'Dubarde Vins Sàrl');

		// (i) LA SEULE ASSERTION QUI PROUVE QUELQUE CHOSE DE CETTE STORY.
		const zone = page.getByTestId('contact-duplicate-nearby');
		await expect(zone).toContainText('Dubarde Vins SA', { timeout: 5000 });

		// (ii) l'interface ne bloque pas.
		const enregistrer = page.getByRole('button', { name: /^Créer$|^Enregistrer$/ });
		await expect(enregistrer).toBeEnabled();

		// (iii) et le contact est bien créé — vérifié après rechargement.
		//
		// ⚠️ **Attendre la RÉPONSE du POST avant de naviguer.** `click()` n'attend
		// que la distribution de l'événement, pas la fin de la requête qu'il
		// déclenche : un `goto` immédiat annule les requêtes en vol, et
		// l'assertion qui suit — celle qui prouve que le contact est créé malgré
		// l'avertissement — pouvait échouer par intermittence, pour une raison
		// sans aucun rapport avec ce qu'elle mesure. C'est le pire mode d'échec
		// d'un test : il rougit, on cherche ailleurs, et on finit par affaiblir
		// l'assertion. Relevé en passe 2 de revue de code.
		const cree = page.waitForResponse(
			(r) => r.url().includes('/api/v1/contacts') && r.request().method() === 'POST'
		);
		await enregistrer.click();
		await cree;
		await page.goto('/contacts');
		await expect(page.getByText('Dubarde Vins Sàrl')).toBeVisible({ timeout: 5000 });
	});

	test('signal FRANC : il prévient, il ne bloque pas, et l’enregistrement est refusé', async ({
		page
	}) => {
		await login(page);
		// (0) MONTAGE — le porteur de l'IDE que le test va retaper.
		await creerContact(page, 'Ancienne Maison SA', IDE_TEST_2);

		await ouvrirNouveauContact(page);
		await page.fill('#form-name', 'Nouvelle Maison SA');
		await page.fill('#form-ide', IDE_TEST_2);

		// (i) le signal FRANC est visible, et il nomme le porteur.
		const zone = page.getByTestId('contact-duplicate-ide');
		await expect(zone).toContainText('Ancienne Maison SA', { timeout: 5000 });

		// (ii) l'interface ne bloque pas davantage qu'elle ne le ferait sans sonde.
		const enregistrer = page.getByRole('button', { name: /^Créer$|^Enregistrer$/ });
		await expect(enregistrer).toBeEnabled();

		// (iii) et la contrainte, elle, refuse — la sonde évite d'y ARRIVER,
		// elle ne remplace pas la garde.
		await enregistrer.click();
		await expect(page.getByText(/numéro IDE existe déjà/i).first()).toBeVisible({ timeout: 5000 });
	});
});
