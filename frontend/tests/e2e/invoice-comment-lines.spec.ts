import { expect, test, type Page } from '@playwright/test';
import fs from 'node:fs/promises';
import path from 'node:path';
import {
	seedTestState,
	clearAuthStorage,
	authedApiContext,
	disposeContextSafe,
} from './helpers/test-state';
import {
	createContactWithAddressViaApi,
	ensurePrimaryBankAccountViaApi,
} from './helpers/api-fixtures';

/**
 * Story 49-1c (#1242, T10) — la ligne de commentaire de bout en bout, dans le navigateur :
 * saisie (AC3), relecture du brouillon et réenregistrement (AC4), fiche de la facture
 * brouillon et validée (AC5), PDF (AC6), avoir qui la reprend (AC5, AC6).
 *
 * Les captures de la recette visuelle (AC11, thème clair) sont prises quand
 * `KESH_E2E_CAPTURES_DIR` est posé ; le PDF de la facture y est aussi déposé.
 */

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

function uniq(prefix: string): string {
	return `${prefix} ${Date.now()}-${Math.floor(Math.random() * 1e6)}`;
}

/** Capture (thème clair) si `KESH_E2E_CAPTURES_DIR` est posé ; sinon, rien. */
async function capture(page: Page, name: string) {
	const dossier = process.env.KESH_E2E_CAPTURES_DIR;
	if (!dossier) return;
	await fs.mkdir(dossier, { recursive: true });
	await page.screenshot({ path: path.join(dossier, `${name}.png`), fullPage: true });
}

async function createProductViaApi(page: Page, name: string): Promise<void> {
	const ctx = await authedApiContext(page);
	try {
		const res = await ctx.post('/api/v1/products', {
			data: { name, unitPrice: '150.00', vatRate: '8.10' },
		});
		expect(res.ok(), `createProduct failed: ${res.status()}`).toBeTruthy();
	} finally {
		await disposeContextSafe(ctx);
	}
}

const COMMENTAIRE = 'Merci de votre confiance.\nTravaux réalisés en novembre.';

test.describe('Factures — ligne de commentaire (49-1c)', () => {
	test('saisie, brouillon relu, validation, PDF, avoir', async ({ page }) => {
		await login(page);
		await ensurePrimaryBankAccountViaApi(page);
		const contactName = uniq('Commentaire SA');
		await createContactWithAddressViaApi(page, contactName);
		const produit = uniq('Article commentaire');
		await createProductViaApi(page, produit);

		await page.goto('/invoices/new');
		await expect(page.getByRole('heading', { name: 'Nouvelle facture' })).toBeVisible();
		const contact = page.getByRole('combobox', { name: /Rechercher un contact/ });
		await contact.click();
		await contact.fill(contactName);
		await page.getByRole('option', { name: new RegExp(contactName) }).first().click();

		// Ligne libre (celle par défaut).
		const premiere = page.locator('tbody tr').first();
		await premiere.locator('input[type="text"]').first().fill('Conseil');
		const nombres = premiere.locator('input[inputmode="decimal"]');
		await nombres.nth(0).fill('2');
		await nombres.nth(1).fill('100.00');

		// « Commentaire » : un champ multiligne en fin de tableau, sans taux.
		await page.getByTestId('invoice-add-comment-line').click();
		const champ = page.getByTestId('invoice-line-comment');
		await expect(champ).toHaveCount(1);
		await expect(page.getByTestId('invoice-line-vat-rate')).toHaveCount(1);

		// Vide, il bloque l'envoi en nommant la ligne.
		await page.getByRole('button', { name: 'Créer la facture' }).click();
		await expect(page.getByTestId('invoice-form-error')).toContainText(
			'Ligne 2 : le texte du commentaire est obligatoire',
		);
		await expect(page).toHaveURL(/\/invoices\/new$/);
		await capture(page, '02-formulaire-commentaire-vide-bloque');

		await champ.fill(COMMENTAIRE);

		// Ligne du catalogue, après le commentaire.
		await page.getByRole('button', { name: /Depuis catalogue/ }).click();
		await expect(page.getByRole('dialog')).toBeVisible();
		await page.getByPlaceholder(/Rechercher un produit/).fill(produit);
		await page.getByRole('dialog').getByRole('button').filter({ hasText: produit }).first().click();
		await expect(page.locator('[data-testid="invoice-lines-table"] tbody > tr')).toHaveCount(3);
		// Le total ignore le commentaire : 2 × 100 + 150.
		await expect(page.locator('[data-testid="invoice-lines-table"] tfoot')).toContainText('350.00');
		await capture(page, '01-formulaire-avec-commentaire');

		await page.getByRole('button', { name: 'Créer la facture' }).click();
		await expect(page).toHaveURL(/\/invoices\/\d+$/);
		const invoiceId = Number(page.url().split('/').pop());

		// Fiche du brouillon : une cellule sur toute la largeur, sauts de ligne gardés.
		const cellule = page.getByTestId('invoice-line-comment');
		await expect(cellule).toHaveText(COMMENTAIRE);
		await expect(cellule).toHaveAttribute('colspan', '6');
		await capture(page, '03-fiche-brouillon');

		// Rouvrir : le commentaire est là, à sa place (deuxième rangée).
		await page.goto(`/invoices/${invoiceId}/edit`);
		const rangees = page.locator('[data-testid="invoice-lines-table"] tbody > tr');
		await expect(rangees).toHaveCount(3);
		await expect(rangees.nth(1)).toHaveAttribute('data-testid', 'invoice-line-comment-row');
		await expect(page.getByTestId('invoice-line-comment')).toHaveValue(COMMENTAIRE);
		await page.getByRole('button', { name: 'Enregistrer' }).click();
		await expect(page).toHaveURL(new RegExp(`/invoices/${invoiceId}$`));

		const ctx = await authedApiContext(page);
		try {
			const lu = await (await ctx.get(`/api/v1/invoices/${invoiceId}`)).json();
			expect(lu.lines.map((l: { lineType: string }) => l.lineType)).toEqual([
				'priced',
				'comment',
				'priced',
			]);
			expect(lu.lines[1].description).toBe(COMMENTAIRE);
			expect(lu.totalAmount).toBe('350.0000');
		} finally {
			await disposeContextSafe(ctx);
		}

		// Valider depuis la fiche.
		await page.getByRole('button', { name: /Valider/i }).first().click();
		await page.getByTestId('invoice-validate-dialog').getByRole('button', { name: /Valider/i }).click();
		await expect(page.getByTestId('invoice-unvalidate-button')).toBeVisible();
		await expect(page.getByTestId('invoice-line-comment')).toHaveText(COMMENTAIRE);
		await capture(page, '04-fiche-validee');

		// PDF de la facture : 200 et %PDF ; déposé pour la recette visuelle.
		const ctx2 = await authedApiContext(page);
		try {
			const pdf = await ctx2.get(`/api/v1/invoices/${invoiceId}/pdf`);
			expect(pdf.status()).toBe(200);
			const octets = await pdf.body();
			expect(octets.subarray(0, 5).toString('utf8')).toBe('%PDF-');
			const dossier = process.env.KESH_E2E_CAPTURES_DIR;
			if (dossier) await fs.writeFile(path.join(dossier, '05-facture.pdf'), octets);
		} finally {
			await disposeContextSafe(ctx2);
		}

		// L'avoir reprend le commentaire.
		await page.getByRole('button', { name: /Créer un avoir/i }).click();
		await page.getByRole('dialog').getByRole('button', { name: /Créer un avoir/i }).click();
		await expect(page).toHaveURL(/\/credit-notes\/\d+$/);
		const avoir = page.getByTestId('credit-note-line-comment');
		await expect(avoir).toHaveText(COMMENTAIRE);
		await expect(avoir).toHaveAttribute('colspan', '6');
		await capture(page, '06-fiche-avoir');
	});
});
