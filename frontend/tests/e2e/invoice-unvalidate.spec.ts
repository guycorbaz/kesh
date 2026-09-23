import { expect, test } from '@playwright/test';
import { seedTestState, clearAuthStorage, authedApiContext, disposeContextSafe } from './helpers/test-state';
import {
	createContactWithAddressViaApi,
	ensurePrimaryBankAccountViaApi,
	createAndValidateInvoiceViaApi,
} from './helpers/api-fixtures';

/**
 * Tests E2E — Story 25-2-b-2 (#440) : **les deux cycles de la dévalidation**,
 * bout à bout, par l'écran.
 *
 * ⛔ **Ce qu'aucun autre test ne voit.** Les tests de dépôt établissent le
 * `DbError`, les tests Rust d'API le statut et le code, Vitest la construction
 * du payload. Aucun ne montre qu'un **utilisateur** peut aller d'un bout à
 * l'autre : que le bouton existe, qu'il ouvre la bonne modale, que celle-ci
 * appelle la bonne route, et que l'écran reflète ensuite le nouvel état.
 *
 * ⚠️ Le précédent qui a motivé ces tests est écrit dans la fiche : une **seule**
 * modale servait les deux branches, titre et pied en dur. Renommer le bouton
 * sans toucher à la modale aurait donné un « Dévalider » qui ouvre « Supprimer
 * la facture » et **appelle la suppression** — laquelle échoue en 409 depuis
 * cette story. *Rien d'autre que ces tests ne l'aurait montré.*
 */

test.beforeAll(async () => {
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

function uniq(prefix: string): string {
	return `${prefix} ${Date.now()}-${Math.floor(Math.random() * 1e6)}`;
}

/** Lit l'état serveur de la facture, sans passer par l'écran. */
async function etatServeur(
	page: import('@playwright/test').Page,
	invoiceId: number,
): Promise<{ status: string; invoiceNumber: string | null; journalEntryId: number | null }> {
	const ctx = await authedApiContext(page);
	try {
		const res = await ctx.get(`/api/v1/invoices/${invoiceId}`);
		expect(res.ok(), `GET invoice failed: ${res.status()}`).toBeTruthy();
		const body = await res.json();
		return {
			status: body.status,
			invoiceNumber: body.invoiceNumber ?? null,
			journalEntryId: body.journalEntryId ?? null,
		};
	} finally {
		await disposeContextSafe(ctx);
	}
}

test.describe('Dévalidation — les deux cycles (25-2-b-2, #440)', () => {
	test('cycle « dévalider puis effacer » : la facture part, en deux gestes', async ({ page }) => {
		await login(page);
		await ensurePrimaryBankAccountViaApi(page);
		const contact = await createContactWithAddressViaApi(page, uniq('Cycle Effacer SA'));
		const invoiceId = await createAndValidateInvoiceViaApi(page, contact);

		await page.goto(`/invoices/${invoiceId}`);

		// (1) Le bouton existe sur une facture validée, et il dit « Dévalider ».
		const devaliderBtn = page.getByTestId('invoice-unvalidate-button');
		await expect(devaliderBtn).toBeVisible();
		await devaliderBtn.click();

		// (2) La modale est CELLE de la dévalidation, pas celle de la suppression.
		await expect(page.getByTestId('invoice-unvalidate-dialog')).toBeVisible();
		await page.getByTestId('invoice-unvalidate-confirm').click();

		// (3) La facture est un brouillon, SANS écriture, et elle garde son numéro.
		await expect(devaliderBtn).toBeHidden();
		const apres = await etatServeur(page, invoiceId);
		expect(apres.status).toBe('draft');
		expect(apres.journalEntryId).toBeNull();
		expect(apres.invoiceNumber, 'le numéro survit à la dévalidation').not.toBeNull();

		// (4) Le brouillon se supprime — et la confirmation AVERTIT du trou de
		// séquence, puisqu'il porte déjà un numéro.
		await page.reload();
		await page.getByRole('button', { name: /Supprimer/i }).click();
		const modale = page.getByRole('dialog');
		await expect(modale).toContainText(apres.invoiceNumber!);
		await modale.getByRole('button', { name: /Supprimer/i }).click();

		await expect(page).toHaveURL(/\/invoices$/);
		const ctx = await authedApiContext(page);
		try {
			const res = await ctx.get(`/api/v1/invoices/${invoiceId}`);
			expect(res.status(), 'la facture a bien disparu').toBe(404);
		} finally {
			await disposeContextSafe(ctx);
		}
	});

	test('cycle « dévalider, corriger, revalider » : le numéro est le MÊME au bout', async ({
		page,
	}) => {
		await login(page);
		await ensurePrimaryBankAccountViaApi(page);
		const contact = await createContactWithAddressViaApi(page, uniq('Cycle Corriger SA'));
		const invoiceId = await createAndValidateInvoiceViaApi(page, contact);

		const avant = await etatServeur(page, invoiceId);
		expect(avant.status).toBe('validated');
		const numeroInitial = avant.invoiceNumber;
		expect(numeroInitial).not.toBeNull();

		await page.goto(`/invoices/${invoiceId}`);
		await page.getByTestId('invoice-unvalidate-button').click();
		await page.getByTestId('invoice-unvalidate-confirm').click();
		await expect(page.getByTestId('invoice-unvalidate-button')).toBeHidden();

		// La correction passe par l'écran d'édition, rouvert au brouillon.
		await page.goto(`/invoices/${invoiceId}/edit`);
		await expect(page).toHaveURL(new RegExp(`/invoices/${invoiceId}/edit$`));

		// Revalidation depuis la fiche.
		await page.goto(`/invoices/${invoiceId}`);
		await page.getByRole('button', { name: /Valider/i }).first().click();
		await page.getByTestId('invoice-validate-dialog').getByRole('button', { name: /Valider/i }).click();
		await expect(page.getByTestId('invoice-unvalidate-button')).toBeVisible();

		// ⛔ **L'invariant-titre de l'epic** : le même numéro, et une écriture
		// neuve. Si la revalidation tirait du compteur, chaque cycle creuserait
		// un trou dans la séquence des FACTURES — la maladie que la 25-2-c a
		// fermée pour les écritures.
		const apres = await etatServeur(page, invoiceId);
		expect(apres.status).toBe('validated');
		expect(apres.invoiceNumber, 'la revalidation REPREND le numéro').toBe(numeroInitial);
		expect(apres.journalEntryId, 'une écriture neuve est créée').not.toBeNull();
	});

	test('un refus s’affiche en NOMMANT son motif, dans la modale, et ne change rien', async ({
		page,
	}) => {
		await login(page);
		await ensurePrimaryBankAccountViaApi(page);
		const contact = await createContactWithAddressViaApi(page, uniq('Refus Nomme SA'));
		const invoiceId = await createAndValidateInvoiceViaApi(page, contact);

		await page.goto(`/invoices/${invoiceId}`);
		// La modale s'ouvre avec la version que l'écran a chargée.
		await page.getByTestId('invoice-unvalidate-button').click();
		await expect(page.getByTestId('invoice-unvalidate-dialog')).toBeVisible();

		// ⚠️ **Le refus employé est le CONFLIT DE VERSION**, et ce choix s'écrit —
		// il a coûté trois essais, chacun instructif :
		//   • « envoyée » (motif 4) suppose un envoi SMTP réel, impossible ici ;
		//   • **l'avoir (motif 2) fait passer la facture en `cancelled`**, donc le
		//     bouton disparaît et le test n'atteint jamais le refus qu'il mesure ;
		//   • le **règlement** (motif 1) exige des paramètres de facturation, et
		//     le **rappel manuel** (motif 3) un niveau de relance configuré ;
		//   • le verrou de période (motif 6) se poserait sur la **société**, que
		//     toutes les specs partagent.
		// Le conflit de version n'exige aucun montage, et il exerce **le même
		// chemin d'affichage** : c'est lui que ce test mesure, pas le motif.
		const ctx = await authedApiContext(page);
		try {
			const get = await ctx.get(`/api/v1/invoices/${invoiceId}`);
			const { version } = await get.json();
			const res = await ctx.put(`/api/v1/invoices/${invoiceId}/dunning-pause`, {
				data: { version, note: 'modifiée ailleurs' },
			});
			expect(res.ok(), `la version doit avoir bougé: ${res.status()}`).toBeTruthy();
		} finally {
			await disposeContextSafe(ctx);
		}

		await page.getByTestId('invoice-unvalidate-confirm').click();

		// ⛔ Le motif est lisible DANS la modale, qui reste ouverte : un refus qui
		// fermerait la modale et n'afficherait qu'un toast laisserait
		// l'utilisateur sans rien à relire.
		const erreur = page.getByTestId('invoice-unvalidate-error');
		await expect(erreur).toBeVisible();
		await expect(erreur).not.toBeEmpty();
		await expect(page.getByTestId('invoice-unvalidate-dialog')).toBeVisible();

		// Et rien n'a bougé : la facture est toujours validée, avec son écriture.
		const apres = await etatServeur(page, invoiceId);
		expect(apres.status).toBe('validated');
		expect(apres.journalEntryId).not.toBeNull();
	});
});
