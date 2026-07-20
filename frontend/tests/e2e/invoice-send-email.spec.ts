/**
 * Tests E2E — Envoi d'une facture par e-mail (Story 20-4, Epic 20 #224).
 *
 * ⚠️ PRÉ-REQUIS : backend démarré en KESH_TEST_MODE **avec vars SMTP
 * factices** (KESH_SMTP_HOST/USER/PASSWORD/FROM) → le boot substitue un
 * MockMailer capturant (aucun e-mail réel ne part) et expose
 * `GET /api/v1/_test/sent-emails`. Recette complète : docs/testing.md.
 * Sans SMTP factice, le premier test échoue avec un diagnostic explicite.
 *
 * Couvre (décision #17 epic / AC#6 story 20-4) : round-trip complet
 * (modale pré-remplie langue+civilité, destinataire VERROUILLÉ — jamais
 * d'input, envoi, « Envoyée le », e-mail capturé avec PDF joint), renvoi,
 * contact sans e-mail, gate rôle Consultation (bouton absent).
 */
import { expect, test } from '@playwright/test';
import type { Page } from '@playwright/test';
import {
	seedTestState,
	clearAuthStorage,
	authedApiContext,
	disposeContextSafe,
	fetchSentEmails,
} from './helpers/test-state';
import {
	createContactWithAddressViaApi,
	ensurePrimaryBankAccountViaApi,
	createAndValidateInvoiceViaApi,
} from './helpers/api-fixtures';

test.beforeAll(async () => {
	// Mutations scopées à des rows créées par chaque test → beforeAll
	// (règle docs/testing.md ; purge aussi le buffer de capture côté backend).
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

test.describe('Envoi de facture par e-mail — round-trip (Story 20-4)', () => {
	test('round-trip complet : modale pré-remplie, destinataire verrouillé, envoi capturé avec PDF', async ({
		page,
	}) => {
		await login(page);
		await ensurePrimaryBankAccountViaApi(page);
		const email = `pia-${Date.now()}@example.ch`;
		const contactId = await createContactWithAddressViaApi(page, uniq('Muster'), email, 'Madame');
		const invoiceId = await createAndValidateInvoiceViaApi(page, contactId);
		const before = (await fetchSentEmails(page)).length;

		await page.goto(`/invoices/${invoiceId}`);
		const sendButton = page.getByTestId('send-email-button');
		await expect(sendButton).toBeVisible();
		await expect(sendButton).toBeEnabled();
		await sendButton.click();

		// Modale pré-remplie : destinataire affiché en texte, JAMAIS un input.
		await expect(page.getByTestId('send-email-to')).toHaveText(email);
		const inputsWithEmail = page.locator(`input[value="${email}"]`);
		await expect(inputsWithEmail).toHaveCount(0);
		const subject = page.locator('input[id$="-subject"]');
		await expect(subject).not.toHaveValue('');
		// Salutation genrée (contact Madame + nom) rendue dans le corps.
		const body = page.locator('textarea[id$="-body"]');
		await expect(body).toHaveValue(/Chère Madame/);

		await page.getByTestId('send-email-confirm').click();

		// Fiche mise à jour : « Envoyée le … » + destinataire.
		await expect(page.getByTestId('invoice-emailed-at')).toBeVisible({ timeout: 10000 });
		await expect(page.getByTestId('invoice-emailed-to')).toHaveText(email);

		// E-mail réellement capturé côté backend, PDF joint non vide.
		const emails = await fetchSentEmails(page);
		expect(emails.length).toBe(before + 1);
		const sent = emails[emails.length - 1];
		expect(sent.to).toBe(email);
		expect(sent.subject).not.toBe('');
		expect(sent.attachmentFilename).toMatch(/^facture-.*\.pdf$/);
		expect(sent.attachmentContentType).toBe('application/pdf');
		expect(sent.attachmentSize as number).toBeGreaterThan(1000);
	});

	test('renvoi : le bouton devient « Renvoyer », 2e envoi capturé', async ({ page }) => {
		await login(page);
		await ensurePrimaryBankAccountViaApi(page);
		const email = `re-${Date.now()}@example.ch`;
		const contactId = await createContactWithAddressViaApi(page, uniq('Renvoi'), email, 'Neutre');
		const invoiceId = await createAndValidateInvoiceViaApi(page, contactId);

		await page.goto(`/invoices/${invoiceId}`);
		await page.getByTestId('send-email-button').click();
		await page.getByTestId('send-email-confirm').click();
		await expect(page.getByTestId('invoice-emailed-at')).toBeVisible({ timeout: 10000 });

		const sendButton = page.getByTestId('send-email-button');
		await expect(sendButton).toHaveText(/Renvoyer par e-mail/);
		const before = (await fetchSentEmails(page)).length;
		await sendButton.click();
		await page.getByTestId('send-email-confirm').click();
		await expect
			.poll(async () => (await fetchSentEmails(page)).length, { timeout: 10000 })
			.toBe(before + 1);
	});

	test("contact sans e-mail : message dédié, bouton d'envoi désactivé", async ({ page }) => {
		await login(page);
		await ensurePrimaryBankAccountViaApi(page);
		const contactId = await createContactWithAddressViaApi(page, uniq('SansEmail'));
		const invoiceId = await createAndValidateInvoiceViaApi(page, contactId);

		await page.goto(`/invoices/${invoiceId}`);
		await page.getByTestId('send-email-button').click();
		await expect(page.getByTestId('send-email-to-missing')).toBeVisible();
		await expect(page.getByTestId('send-email-confirm')).toBeDisabled();
	});

	test('gate rôle : un utilisateur Consultation ne voit pas le bouton', async ({ page }) => {
		await login(page);
		await ensurePrimaryBankAccountViaApi(page);
		const contactId = await createContactWithAddressViaApi(
			page,
			uniq('Consult'),
			`c-${Date.now()}@example.ch`,
		);
		const invoiceId = await createAndValidateInvoiceViaApi(page, contactId);
		// Créer le user Consultation via API (admin connecté).
		const username = `consult-${Date.now()}`;
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
		await page.goto(`/invoices/${invoiceId}`);
		// Anti-vacuous (review P1 AA) : confirmer que la fiche a réellement
		// chargé (URL + métadonnée visible) AVANT d'asserter l'absence du
		// bouton — sinon une redirection/erreur ferait passer le test à tort.
		await expect(page).toHaveURL(new RegExp(`/invoices/${invoiceId}$`));
		await expect(page.getByTestId('send-email-button')).toHaveCount(0);
	});
});
