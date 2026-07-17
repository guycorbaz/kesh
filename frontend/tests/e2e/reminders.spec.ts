/**
 * Tests E2E — Page « Rappels » (Story 21-6b, Epic 21 #231).
 *
 * ⚠️ PRÉ-REQUIS : backend en KESH_TEST_MODE **avec vars SMTP factices** →
 * MockMailer capturant + `GET /api/v1/_test/sent-emails`. Recette : docs/testing.md.
 *
 * Couvre : liste groupée par débiteur, envoi unitaire (aperçu → e-mail capturé),
 * envoi lot, rappel manuel, contact sans e-mail (badge + non-cochable),
 * anti-double-submit (bouton disabled en vol → un seul e-mail).
 */
import { expect, test } from '@playwright/test';
import type { Page } from '@playwright/test';
import AxeBuilder from '@axe-core/playwright';
import { seedTestState, clearAuthStorage } from './helpers/test-state';
import {
	createContactWithAddressViaApi,
	ensurePrimaryBankAccountViaApi,
	createAndValidateInvoiceViaApi,
} from './helpers/api-fixtures';

const BACKEND_URL = process.env.KESH_BACKEND_URL ?? 'http://127.0.0.1';

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

/** Échéance suffisamment passée pour être éligible au niveau 1 (seuil today - 15j). */
function overdueDate(days = 25): string {
	const d = new Date();
	d.setDate(d.getDate() - days);
	return d.toISOString().slice(0, 10);
}

async function fetchSentEmails(page: Page): Promise<Array<Record<string, unknown>>> {
	const res = await page.request.get(`${BACKEND_URL}/api/v1/_test/sent-emails`);
	expect(
		res.ok(),
		`GET /_test/sent-emails → ${res.status()} — backend démarré sans SMTP factice ? (cf. docs/testing.md)`,
	).toBeTruthy();
	return (await res.json()).emails;
}

test.describe('Page Rappels (Story 21-6b)', () => {
	test('liste groupée + envoi unitaire → e-mail capturé', async ({ page }) => {
		await login(page);
		await ensurePrimaryBankAccountViaApi(page);
		const name = uniq('Débiteur SA');
		const contact = await createContactWithAddressViaApi(page, name, 'debiteur@example.ch', 'Madame');
		await createAndValidateInvoiceViaApi(page, contact, overdueDate());

		await page.goto('/invoices/reminders');
		const group = page.locator('div.rounded', { hasText: name });
		const row = group.getByTestId('reminder-row');
		await expect(page.getByText(name).first()).toBeVisible();
		await expect(row.first()).toBeVisible();

		const before = (await fetchSentEmails(page)).length;

		// Ouvrir la modale d'envoi unitaire → aperçu serveur chargé.
		await row.first().getByTestId('reminder-send-open').click();
		await expect(page.getByTestId('reminder-send-to')).toContainText('debiteur@example.ch');
		await expect(page.getByTestId('reminder-send-confirm')).toBeEnabled();

		await page.getByTestId('reminder-send-confirm').click();
		// L'e-mail est capturé (avec PDF joint).
		await expect
			.poll(async () => (await fetchSentEmails(page)).length, { timeout: 10000 })
			.toBe(before + 1);
		const sent = (await fetchSentEmails(page)).at(-1)!;
		expect(sent.to).toBe('debiteur@example.ch');
		expect(String(sent.attachmentFilename)).toMatch(/\.pdf$/);
	});

	test('contact sans e-mail : badge + case désactivée + envoi absent, manuel possible', async ({
		page,
	}) => {
		await login(page);
		await ensurePrimaryBankAccountViaApi(page);
		const name = uniq('Sans Email SA');
		// Pas d'e-mail sur ce contact.
		const contact = await createContactWithAddressViaApi(page, name);
		await createAndValidateInvoiceViaApi(page, contact, overdueDate());

		await page.goto('/invoices/reminders');
		const group = page.locator('div.rounded', { hasText: name });
		await expect(group.getByTestId('reminder-no-email-badge').first()).toBeVisible();
		// Case de sélection désactivée.
		await expect(group.getByTestId('reminder-batch-checkbox').first()).toBeDisabled();
		// Bouton d'envoi e-mail absent.
		await expect(group.getByTestId('reminder-send-open')).toHaveCount(0);
		// Rappel manuel disponible.
		await expect(group.getByTestId('reminder-manual-open').first()).toBeVisible();
	});

	test('rappel manuel → cycle avancé', async ({ page }) => {
		await login(page);
		await ensurePrimaryBankAccountViaApi(page);
		const name = uniq('Manuel SA');
		const contact = await createContactWithAddressViaApi(page, name, 'manuel@example.ch');
		await createAndValidateInvoiceViaApi(page, contact, overdueDate());

		await page.goto('/invoices/reminders');
		const group = page.locator('div.rounded', { hasText: name });
		const row = group.getByTestId('reminder-row');
		await row.first().getByTestId('reminder-manual-open').click();
		await expect(page.getByTestId('manual-reminder-confirm')).toBeVisible();
		await page.getByTestId('manual-reminder-confirm').click();
		// Toast succès (la modale se ferme) — la ligne reste (niveau avancé).
		await expect(page.getByTestId('manual-reminder-confirm')).toHaveCount(0, { timeout: 10000 });
	});

	test('envoi lot → rapport { accepted, failed }', async ({ page }) => {
		await login(page);
		await ensurePrimaryBankAccountViaApi(page);
		const name = uniq('Lot SA');
		const contact = await createContactWithAddressViaApi(page, name, 'lot@example.ch');
		await createAndValidateInvoiceViaApi(page, contact, overdueDate());

		await page.goto('/invoices/reminders');
		const group = page.locator('div.rounded', { hasText: name });
		const row = group.getByTestId('reminder-row');
		const before = (await fetchSentEmails(page)).length;

		await row.first().getByTestId('reminder-batch-checkbox').check();
		await expect(page.getByTestId('reminder-selected-count')).toContainText('1');
		await page.getByTestId('reminder-batch-send').click();

		await expect(page.getByTestId('reminder-batch-report')).toBeVisible({ timeout: 10000 });
		await expect
			.poll(async () => (await fetchSentEmails(page)).length, { timeout: 10000 })
			.toBe(before + 1);
	});

	/**
	 * AC 25 — anti-double-submit (AC de premier plan). Le backend n'a AUCUNE
	 * garde ; un double-clic non protégé enverrait deux e-mails réels au débiteur.
	 * On ralentit la route d'envoi pour observer le `disabled` en vol, puis on
	 * vérifie qu'un seul e-mail est capturé malgré deux clics.
	 */
	test('anti-double-submit : le bouton se désactive en vol, un seul e-mail', async ({ page }) => {
		await login(page);
		await ensurePrimaryBankAccountViaApi(page);
		const name = uniq('AntiDouble SA');
		const contact = await createContactWithAddressViaApi(page, name, 'anti@example.ch');
		await createAndValidateInvoiceViaApi(page, contact, overdueDate());

		await page.goto('/invoices/reminders');
		const group = page.locator('div.rounded', { hasText: name });
		const row = group.getByTestId('reminder-row');
		const before = (await fetchSentEmails(page)).length;

		// Ralentit le POST d'envoi (laisse le temps d'observer le disabled).
		await page.route('**/reminders/send', async (route) => {
			await new Promise((r) => setTimeout(r, 800));
			await route.continue();
		});

		await row.first().getByTestId('reminder-send-open').click();
		const confirm = page.getByTestId('reminder-send-confirm');
		await confirm.click();
		// Immédiatement après le clic, le bouton est disabled (couche B).
		await expect(confirm).toBeDisabled();
		// Un 2e clic ne peut pas passer (bouton disabled) — tenter quand même.
		await confirm.click({ force: true }).catch(() => {});

		await expect
			.poll(async () => (await fetchSentEmails(page)).length, { timeout: 10000 })
			.toBe(before + 1);
		// Toujours exactement un seul e-mail (pas de double-envoi).
		expect((await fetchSentEmails(page)).length).toBe(before + 1);
	});

	test('a11y : page peuplée sans violation axe dans le sous-arbre de la story', async ({ page }) => {
		await login(page);
		await ensurePrimaryBankAccountViaApi(page);
		const name = uniq('A11y SA');
		const contact = await createContactWithAddressViaApi(page, name, 'a11y@example.ch');
		await createAndValidateInvoiceViaApi(page, contact, overdueDate());

		await page.goto('/invoices/reminders');
		await expect(page.getByTestId('reminders-list')).toBeVisible();
		await page.waitForLoadState('networkidle');
		// Scopé au sous-arbre de la story ; dettes systémiques pré-existantes
		// neutralisées : color-contrast (#253), button-name (#256).
		const results = await new AxeBuilder({ page })
			.include('[data-testid="reminders-list"]')
			.disableRules(['color-contrast', 'button-name'])
			.analyze();
		expect(results.violations).toEqual([]);
	});
});
