/**
 * Story 21-8 — E2E round-trip du cycle « rappels débiteurs » (Epic 21).
 *
 * Un test unique qui chaîne le parcours utilisateur de bout en bout, piloté par
 * l'UI : config des niveaux (seed lazy) → factures échues → liste à rappeler →
 * envoi unitaire (e-mail capturé) → envoi lot (e-mail capturé) → historique sur
 * la fiche → suspension (motif persisté) → balance âgée (facture suspendue
 * incluse, D10).
 *
 * ⚠️ PRÉ-REQUIS : backend `KESH_TEST_MODE` **avec SMTP factice** → MockMailer
 * capturant + `GET /api/v1/_test/sent-emails`. Recette : docs/testing.md.
 *
 * La facture A (unitaire) DISPARAÎT de la liste à rappeler après son envoi
 * niveau 1 (niveau 2 non encore dû) — d'où la facture B, séparée, pour le lot
 * (validé validate 21-8, `dunning_eligibility.rs`).
 */
import { expect, test, type Page } from '@playwright/test';
import { seedTestState, clearAuthStorage, fetchSentEmails } from './helpers/test-state';
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

async function login(page: Page): Promise<void> {
	await page.goto('/login');
	await page.fill('#username', 'admin');
	await page.fill('#password', 'admin123');
	await page.click('button[type="submit"]');
	await expect(page).toHaveURL('/');
}

function uniq(prefix: string): string {
	return `${prefix} ${Date.now()}-${Math.floor(Math.random() * 1e6)}`;
}

test('round-trip rappels : config → envoi unitaire + lot → historique → suspension → balance âgée', async ({
	page,
}) => {
	await login(page);
	await ensurePrimaryBankAccountViaApi(page);

	// (1) CONFIG — le seed lazy pose 3 niveaux au 1er GET de /settings/dunning.
	await page.goto('/settings/dunning');
	await expect(page.getByTestId('dunning-level-row')).toHaveCount(3);

	// (2) FACTURES ÉCHUES — A (unitaire+historique+suspension) et B (lot).
	const nameA = uniq('Débiteur A');
	const nameB = uniq('Débiteur B');
	const contactA = await createContactWithAddressViaApi(page, nameA, 'debiteur-a@example.ch', 'Madame');
	const contactB = await createContactWithAddressViaApi(page, nameB, 'debiteur-b@example.ch', 'Monsieur');
	const invoiceA = await createAndValidateInvoiceViaApi(page, contactA, overdueDate());
	await createAndValidateInvoiceViaApi(page, contactB, overdueDate());

	// (3) LISTE À RAPPELER — les deux débiteurs apparaissent.
	await page.goto('/invoices/reminders');
	const groupA = page.locator('div.rounded', { hasText: nameA });
	const groupB = page.locator('div.rounded', { hasText: nameB });
	await expect(groupA.getByTestId('reminder-row').first()).toBeVisible();
	await expect(groupB.getByTestId('reminder-row').first()).toBeVisible();

	// (4) ENVOI UNITAIRE (facture A) — e-mail capturé avec PDF joint.
	let before = (await fetchSentEmails(page)).length;
	await groupA.getByTestId('reminder-row').first().getByTestId('reminder-send-open').click();
	await expect(page.getByTestId('reminder-send-to')).toContainText('debiteur-a@example.ch');
	await page.getByTestId('reminder-send-confirm').click();
	await expect
		.poll(async () => (await fetchSentEmails(page)).length, { timeout: 10000 })
		.toBe(before + 1);
	const unit = (await fetchSentEmails(page)).at(-1)!;
	expect(unit.to).toBe('debiteur-a@example.ch');
	// La PJ du rappel est la QR-facture PDF (backend `facture-{base}.pdf`).
	expect(String(unit.attachmentFilename)).toMatch(/^facture-.*\.pdf$/);

	// (5) ENVOI LOT (facture B) — la facture A a quitté la liste après son envoi
	// niveau 1 ; B reste sélectionnable.
	before = (await fetchSentEmails(page)).length;
	await groupB.getByTestId('reminder-row').first().getByTestId('reminder-batch-checkbox').check();
	await expect(page.getByTestId('reminder-selected-count')).toContainText('1');
	await page.getByTestId('reminder-batch-send').click();
	await expect(page.getByTestId('reminder-batch-report')).toBeVisible({ timeout: 10000 });
	await expect
		.poll(async () => (await fetchSentEmails(page)).length, { timeout: 10000 })
		.toBe(before + 1);

	// (6) HISTORIQUE — la fiche de la facture A montre le rappel envoyé (canal e-mail).
	await page.goto(`/invoices/${invoiceA}`);
	const history = page.getByTestId('reminder-history');
	await expect(history).toBeVisible();
	const histRow = history.getByTestId('reminder-history-row').first();
	await expect(histRow).toBeVisible();
	await expect(histRow).toContainText('E-mail');

	// (7) SUSPENSION — motif saisi, badge « Suspendu », motif persisté (title).
	const motif = 'litige round-trip';
	await page.getByTestId('dunning-pause-button').click();
	await page.getByTestId('dunning-pause-note').fill(motif);
	await page.getByTestId('dunning-pause-confirm').click();
	const badge = page.getByTestId('invoice-paused-badge');
	await expect(badge).toBeVisible();
	// Le motif n'a d'autre surface d'affichage que l'infobulle du badge.
	await expect(badge).toHaveAttribute('title', new RegExp(motif));

	// (8) BALANCE ÂGÉE — le débiteur A y figure malgré la suspension (D10).
	await page.goto('/reports?tab=aged-receivables');
	await page.getByTestId('aged-report-generate').click();
	await expect(page.getByTestId('aged-receivables-table')).toBeVisible({ timeout: 5000 });
	await expect(
		page.getByTestId('aged-receivables-row').filter({ hasText: nameA }),
	).toBeVisible();
});
