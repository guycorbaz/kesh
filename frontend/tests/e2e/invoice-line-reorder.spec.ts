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
 * Story 49-2 (#1756, #1242, T6) — réordonner les lignes d'un brouillon, de bout en bout dans le
 * navigateur : on crée toutes les lignes à la suite (ligne libre A, commentaire C, article B),
 * puis on les remet dans l'ordre **au clavier** (↑ du commentaire, ↓ de A) ; le focus suit la
 * ligne, le déplacement est annoncé ; l'ordre se garde à l'enregistrement, se retrouve à la
 * réouverture du brouillon, à la fiche de la facture validée, et son PDF se rend (`%PDF-`). L'ordre
 * des lignes **dans** le PDF est prouvé côté Rust par T4 (`de_haut_en_bas`, `invoice_line_types_e2e.rs`) :
 * aucun outil système n'est demandé ici (idefix n'a pas `pdftotext`).
 *
 * Les captures de la recette visuelle (AC14, thème clair) sont prises quand
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
	// `animations: 'disabled'` : les transitions finies (150 ms d'opacité des boutons) sont menées à leur
	// terme avant la prise, pour qu'un bouton qui vient de s'activer ne soit pas capturé en plein fondu.
	await page.screenshot({
		path: path.join(dossier, `${name}.png`),
		fullPage: true,
		animations: 'disabled',
	});
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

const COMMENTAIRE = 'Merci de votre confiance.';
const LIGNE_A = 'Prestation A';

/** Les textes des lignes du formulaire, de haut en bas (commentaire : son texte). */
async function ordreDuFormulaire(page: Page): Promise<string[]> {
	return page.evaluate(() => {
		const rows = Array.from(
			document.querySelectorAll('[data-testid="invoice-lines-table"] tbody > tr'),
		);
		const out: string[] = [];
		for (const r of rows) {
			const t = r.querySelector('textarea');
			const i = r.querySelector('input[type="text"]') as HTMLInputElement | null;
			if (t) out.push((t as HTMLTextAreaElement).value);
			else if (i) out.push(i.value);
		}
		return out;
	});
}

/**
 * Les boutons ↑ / ↓ de la ligne de rang `n` (à partir de 1) : visés par `data-move`, dans l'ordre du
 * tableau, non par leur nom traduit (garde KF-043, `e2e-selecteurs-traduits.test.ts`).
 */
const monter = (page: Page, n: number) => page.locator('button[data-move="up"]').nth(n - 1);
const descendre = (page: Page, n: number) => page.locator('button[data-move="down"]').nth(n - 1);

test.describe('Factures — réordonner les lignes (49-2)', () => {
	test('créer toutes les lignes, puis réordonner au clavier ; l’ordre tient jusqu’à la facture validée', async ({
		page,
	}) => {
		await login(page);
		await ensurePrimaryBankAccountViaApi(page);
		const contactName = uniq('Ordre SA');
		await createContactWithAddressViaApi(page, contactName);
		const produit = uniq('Article B');
		await createProductViaApi(page, produit);

		await page.goto('/invoices/new');
		const envoyer = page.getByTestId('create-invoice-button');
		await expect(envoyer).toBeVisible();
		const contact = page.getByRole('combobox', { name: /Rechercher un contact/ });
		await contact.click();
		await contact.fill(contactName);
		await page.getByRole('option', { name: new RegExp(contactName) }).first().click();

		// A : la ligne libre par défaut.
		const premiere = page.locator('tbody tr').first();
		await premiere.locator('input[type="text"]').first().fill(LIGNE_A);
		const nombres = premiere.locator('input[inputmode="decimal"]');
		await nombres.nth(0).fill('2');
		await nombres.nth(1).fill('100.00');

		// C : un commentaire.
		await page.getByTestId('invoice-add-comment-line').click();
		await page.getByTestId('invoice-line-comment').fill(COMMENTAIRE);

		// B : un article du catalogue.
		await page.getByRole('button', { name: /Depuis catalogue/ }).click();
		await expect(page.getByRole('dialog')).toBeVisible();
		await page.getByPlaceholder(/Rechercher un produit/).fill(produit);
		await page.getByRole('dialog').getByRole('button').filter({ hasText: produit }).first().click();
		await expect(page.locator('[data-testid="invoice-lines-table"] tbody > tr')).toHaveCount(3);
		await expect.poll(() => ordreDuFormulaire(page)).toEqual([LIGNE_A, COMMENTAIRE, produit]);

		// AC1 : bornes désactivées, deux boutons par ligne.
		await expect(monter(page, 1)).toBeDisabled();
		await expect(descendre(page, 3)).toBeDisabled();
		await expect(monter(page, 2)).toBeEnabled();
		await expect(descendre(page, 1)).toBeEnabled();
		// Le nom accessible suit le rang (lu par `aria-label`, hors sélecteur).
		await expect(monter(page, 2)).toHaveAttribute('aria-label', 'Monter la ligne 2');
		await capture(page, 'C1-formulaire-trois-lignes');

		// Au clavier : focus, puis Entrée. ↑ du commentaire : il passe en tête, ↑ devient
		// désactivé, le focus passe à ↓ de la même ligne.
		await monter(page, 2).focus();
		await page.keyboard.press('Enter');
		await expect.poll(() => ordreDuFormulaire(page)).toEqual([COMMENTAIRE, LIGNE_A, produit]);
		await expect(descendre(page, 1)).toBeFocused();
		await expect(page.getByTestId('invoice-lines-move-status')).toHaveText(
			'Ligne déplacée en position 1 sur 3',
		);
		await capture(page, 'C2-commentaire-en-tete');

		// ↓ de A (devenue 2e), cette fois à la touche *Espace* (le manuel promet Entrée et Espace) :
		// A passe en dernière, ↓ désactivé, le focus passe à ↑.
		await descendre(page, 2).focus();
		await page.keyboard.press('Space');
		await expect.poll(() => ordreDuFormulaire(page)).toEqual([COMMENTAIRE, produit, LIGNE_A]);
		await expect(monter(page, 3)).toBeFocused();
		await expect(page.getByTestId('invoice-lines-move-status')).toHaveText(
			'Ligne déplacée en position 3 sur 3',
		);
		// Le total ne bouge pas : 2 × 100 + 150.
		await expect(page.locator('[data-testid="invoice-lines-table"] tfoot')).toContainText('350.00');
		await capture(page, 'C3-focus-apres-deplacement-clavier');

		await envoyer.click();
		await expect(page).toHaveURL(/\/invoices\/\d+$/);
		const invoiceId = Number(page.url().split('/').pop());

		// Rouvrir : même ordre.
		await page.goto(`/invoices/${invoiceId}/edit`);
		await expect(page.locator('[data-testid="invoice-lines-table"] tbody > tr')).toHaveCount(3);
		await expect.poll(() => ordreDuFormulaire(page)).toEqual([COMMENTAIRE, produit, LIGNE_A]);
		await capture(page, 'C4-brouillon-rouvert');
		await page.goto(`/invoices/${invoiceId}`);

		// Valider depuis la fiche : l'ordre se retrouve, sans boutons de déplacement.
		await page.getByRole('button', { name: /Valider/i }).first().click();
		await page.getByTestId('invoice-validate-dialog').getByRole('button', { name: /Valider/i }).click();
		await expect(page.getByTestId('invoice-unvalidate-button')).toBeVisible();
		await expect(page.locator('button[data-move]')).toHaveCount(0);
		await capture(page, 'C5-fiche-validee');

		const ctx = await authedApiContext(page);
		try {
			const lu = await (await ctx.get(`/api/v1/invoices/${invoiceId}`)).json();
			expect(lu.lines.map((l: { description: string }) => l.description)).toEqual([
				COMMENTAIRE,
				produit,
				LIGNE_A,
			]);
			expect(lu.lines.map((l: { position: number }) => l.position)).toEqual([0, 1, 2]);
			const pdf = await ctx.get(`/api/v1/invoices/${invoiceId}/pdf`);
			expect(pdf.status()).toBe(200);
			const octets = await pdf.body();
			expect(octets.subarray(0, 5).toString('utf8')).toBe('%PDF-');
			// L'ordre des lignes dans le PDF est prouvé par T4 (Rust, `de_haut_en_bas`).
			const dossier = process.env.KESH_E2E_CAPTURES_DIR;
			if (dossier) {
				await fs.mkdir(dossier, { recursive: true });
				await fs.writeFile(path.join(dossier, 'C6-facture.pdf'), octets);
			}
		} finally {
			await disposeContextSafe(ctx);
		}
	});
});
