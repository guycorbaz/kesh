/**
 * Tests E2E — Compteur « à rappeler » du tableau de bord (Story 21-6c, #231, D-c2).
 *
 * Le compteur vit sur le widget « Factures ouvertes » de l'accueil (`/`), pas sur
 * la fiche facture — d'où son propre spec. Il dérive de `GET /api/v1/dunning/reminders`
 * (Comptable+) : un rôle Consultation ne DOIT pas le fetcher (sinon 403 en console).
 */
import { expect, test } from '@playwright/test';
import type { Page } from '@playwright/test';
import { seedTestState, clearAuthStorage, authedApiContext, disposeContextSafe } from './helpers/test-state';
import {
	createContactWithAddressViaApi,
	createAndValidateInvoiceViaApi,
	ensurePrimaryBankAccountViaApi,
	overdueDate,
} from './helpers/api-fixtures';

test.beforeAll(async () => {
	await seedTestState('with-company');
});

test.afterEach(async ({ page }) => {
	await clearAuthStorage(page);
});

async function login(page: Page, username = 'admin', password = 'admin123') {
	await page.goto('/login');
	await page.fill('#username', username);
	await page.fill('#password', password);
	await page.click('button[type="submit"]');
	await expect(page).toHaveURL('/');
}

function uniq(prefix: string): string {
	return `${prefix} ${Date.now()}-${Math.floor(Math.random() * 1e6)}`;
}

test.describe('Tableau de bord — compteur « à rappeler » (21-6c)', () => {
	test('affiche N factures à rappeler pour un Comptable+', async ({ page }) => {
		await login(page);
		await ensurePrimaryBankAccountViaApi(page);

		// Facture échue + éligible au niveau 1 (échéance > today - 15j).
		const contact = await createContactWithAddressViaApi(page, uniq('Rappel Dash SA'));
		await createAndValidateInvoiceViaApi(page, contact, overdueDate(25));

		await page.goto('/');
		const card = page.getByTestId('homepage-card-open-invoices');
		const counter = page.getByTestId('homepage-reminders-count');
		await expect(counter).toBeVisible();
		// Un nombre de factures (jamais un montant CHF, L21-8).
		await expect(counter).toHaveText(/\d/);
		await expect(counter).not.toContainText('CHF');
		// Le compteur est un lien vers la page Rappels (scopé au widget — le menu
		// latéral porte aussi un lien /invoices/reminders).
		await expect(card.locator('a[href="/invoices/reminders"]')).toBeVisible();
	});

	test('un rôle Consultation ne fetch pas le compteur (pas de 403)', async ({ page }) => {
		await login(page);
		// Créer un user Consultation via API (admin connecté).
		const username = `consult-dash-${Date.now()}`;
		const ctx = await authedApiContext(page);
		try {
			const res = await ctx.post('/api/v1/users', {
				data: { username, password: 'MotDePasse12345', role: 'Consultation' },
			});
			expect(res.ok(), `create user failed: ${res.status()}`).toBeTruthy();
		} finally {
			await disposeContextSafe(ctx);
		}
		await clearAuthStorage(page);

		await login(page, username, 'MotDePasse12345');

		// Aucun appel à `/dunning/reminders` ne doit partir pour ce rôle.
		let reminderCalls = 0;
		page.on('request', (req) => {
			if (req.url().includes('/api/v1/dunning/reminders')) reminderCalls += 1;
		});

		await page.goto('/');
		// Anti-vacuous : confirmer que le dashboard a chargé (widget présent)…
		await expect(page.getByTestId('homepage-card-open-invoices')).toBeVisible();
		await page.waitForLoadState('networkidle');
		// …le compteur est absent et le widget garde son empty-state.
		await expect(page.getByTestId('homepage-reminders-count')).toHaveCount(0);
		expect(reminderCalls, 'un Consultation ne doit pas fetcher /dunning/reminders').toBe(0);
	});
});
