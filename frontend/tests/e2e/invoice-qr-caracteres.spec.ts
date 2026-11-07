import { inflateSync } from 'node:zlib';
import { expect, test } from '@playwright/test';
import {
	seedTestState,
	clearAuthStorage,
	authedApiContext,
	disposeContextSafe,
} from './helpers/test-state';
import {
	createContactWithAddressViaApi,
	createAndValidateInvoiceViaApi,
	ensurePrimaryBankAccountViaApi,
} from './helpers/api-fixtures';

/**
 * Tests E2E — une QR-facture dont le débiteur porte « & » (Story 33-12a, #941).
 *
 * `Müller & Söhne` est au jeu de caractères de la directive (§ 4.1.1) : le PDF se
 * produit, là où Kesh répondait « caractère non autorisé » à un nom réel. Seul un
 * E2E voit le nom traverser la route jusqu'au fichier.
 */

/** Les flux du PDF, décompressés au besoin, en minuscules. */
function contenuDuPdf(pdf: Buffer): string {
	const brut = pdf.toString('latin1');
	const flux = /stream\r?\n([\s\S]*?)endstream/g;
	let sortie = '';
	for (const m of brut.matchAll(flux)) {
		const octets = Buffer.from(m[1], 'latin1');
		try {
			sortie += inflateSync(octets).toString('latin1');
		} catch {
			sortie += m[1];
		}
	}
	return sortie.toLowerCase();
}

/** Un texte tel qu'il s'écrit dans un `Tj` hexadécimal, Latin-1 valant WinAnsi ici. */
function hexWinAnsi(texte: string): string {
	return Buffer.from(texte, 'latin1').toString('hex');
}

test.beforeAll(async () => {
	await seedTestState('with-company');
});

test.afterEach(async ({ page }) => {
	await clearAuthStorage(page);
});

test('un débiteur au nom portant « & » reçoit sa QR-facture', async ({ page }) => {
	await page.goto('/login');
	await page.fill('#username', 'admin');
	await page.fill('#password', 'admin123');
	await page.click('button[type="submit"]');
	await expect(page).toHaveURL('/');

	await ensurePrimaryBankAccountViaApi(page);
	const nom = `Müller & Söhne ${Date.now()}-${Math.floor(Math.random() * 1e6)}`;
	const contactId = await createContactWithAddressViaApi(page, nom);
	const invoiceId = await createAndValidateInvoiceViaApi(page, contactId);

	const ctx = await authedApiContext(page);
	try {
		const pdf = await ctx.get(`/api/v1/invoices/${invoiceId}/pdf`);
		expect(pdf.status(), await pdf.text()).toBe(200);
		expect(pdf.headers()['content-type']).toContain('application/pdf');
		const octets = await pdf.body();
		expect(octets.subarray(0, 5).toString()).toBe('%PDF-');

		// Le nom traverse jusqu'au papier : au corps et en partie paiement, le NPA et
		// la localité aussi. Au reçu, ce nom long se coupe. La fixture crée une personne
		// de prénom « Pia » : Kesh compose son nom affiché avec.
		const contenu = contenuDuPdf(octets);
		const affiche = `<${hexWinAnsi(`Pia ${nom}`)}> tj`;
		expect(contenu.split(affiche).length - 1).toBe(2);
		expect(contenu).toContain(`<${hexWinAnsi('9400 Rorschach')}> tj`);
	} finally {
		await disposeContextSafe(ctx);
	}
});
