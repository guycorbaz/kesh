import { expect, test } from '@playwright/test';
import type { Page, TestInfo } from '@playwright/test';
import path from 'node:path';
import {
	seedTestState,
	clearAuthStorage,
	authedApiContext,
	disposeContextSafe,
} from './helpers/test-state';
import {
	PDF_SANS_QR,
	PNG_SANS_QR,
	assertInboxDirsUsable,
	createSupplierViaApi,
	deposit,
	ensurePayableAndExpense,
	fillFirstInvoiceLine,
	inboxHandshake,
	readFixture,
	stillInInbox,
	token,
	uniqueCopy,
} from './helpers/inbox';

/**
 * Parcours « Documents à traiter » — Story 30-2a (#685), parcours E1-a, E2-a et E3 de la
 * fiche ; Story 30-2b, parcours E1, E2 et E4 (la facture se crée à côté du document, ou
 * sans document). Guy ne testera qu'en fin d'epic : ces specs portent la recette.
 *
 * ⚠️ **Aucun `test.skip`.** Les répertoires se lisent dans `helpers/inbox.ts` (défauts de la
 * recette documentée) ; un répertoire absent fait échouer le `beforeAll`, un backend qui lit
 * un autre dossier fait échouer la poignée de main. Les assertions portent sur le **nom** du
 * fichier déposé, jamais sur un nombre de lignes : un résidu d'un run interrompu est
 * réimporté au suivant.
 *
 * ⚠️ **Le PDF ne se rend pas en Chromium sans tête** (pas de visionneuse) : pour un PDF, le
 * test vérifie l'`<iframe>`, son `src` en `blob:` et le type servi par l'API, non le rendu.
 * Le rendu réel d'un PDF est relevé à la main, dans un navigateur à fenêtre (AC6-bis).
 */

test.beforeAll(async () => {
	await seedTestState('with-company');
	await assertInboxDirsUsable();
});

test.afterEach(async ({ page }) => {
	await clearAuthStorage(page);
});

let handshakeDone = false;

async function login(page: Page) {
	await page.goto('/login');
	await page.fill('#username', 'admin');
	await page.fill('#password', 'admin123');
	await page.click('button[type="submit"]');
	await expect(page).toHaveURL('/');
	if (!handshakeDone) {
		await inboxHandshake(page);
		handshakeDone = true;
	}
}

/** Importe depuis l'écran et rend la ligne du fichier `fileName`. */
async function importAndFindRow(page: Page, fileName: string) {
	await page.goto('/supplier-invoices/import');
	await page.getByTestId('inbox-import-trigger').click();
	await expect(page.getByTestId('inbox-import-report')).toBeVisible();
	const row = page.getByTestId('imported-row').filter({ hasText: fileName });
	await expect(row).toHaveCount(1);
	return row;
}

test.describe('Documents à traiter', () => {
	test('E1-a — un document avec QR : badge « QR-facture », aperçu visible', async ({ page }) => {
		await login(page);
		const name = `e2e-e1a-${token()}.png`;
		await deposit(name, uniqueCopy(await readFixture('spc_e2e_invoice.png'), 'png'));

		const row = await importAndFindRow(page, name);
		await expect(row.getByTestId('imported-badge-qr')).toBeVisible();
		await expect(row.getByTestId('imported-badge-qr')).toContainText('Fournisseur E2E SA');
		await expect(row.getByTestId('imported-badge-no-qr')).toHaveCount(0);

		await row.getByTestId('imported-preview').click();
		const img = row.getByTestId('document-preview-image');
		await expect(img).toBeVisible();
		await expect(img).toHaveAttribute('src', /^blob:/);
	});

	test('E2-a — un PNG sans QR : badge « Sans QR-code », aperçu <img>, le fichier a quitté le dossier', async ({
		page,
	}) => {
		await login(page);
		const name = `e2e-e2a-${token()}.png`;
		await deposit(name, uniqueCopy(PNG_SANS_QR, 'png'));

		const row = await importAndFindRow(page, name);
		await expect(row.getByTestId('imported-badge-no-qr')).toBeVisible();
		await expect(row.getByTestId('imported-badge-qr')).toHaveCount(0);
		// Le rapport dit « sans QR-code » pour ce fichier.
		await expect(
			page.getByTestId('inbox-import-accepted-row').filter({ hasText: name }),
		).toContainText('sans QR-code');
		expect(await stillInInbox(name), 'le fichier quitte le dossier dès l’import').toBe(false);

		await row.getByTestId('imported-preview').click();
		await expect(row.getByTestId('document-preview-image')).toHaveAttribute('src', /^blob:/);
	});

	test('E3 — un document qui n’est pas une facture : retiré de la liste, puis revenu en le redéposant', async ({
		page,
	}) => {
		await login(page);
		const name = `e2e-e3-${token()}.pdf`;
		const bytes = uniqueCopy(PDF_SANS_QR, 'pdf');
		await deposit(name, bytes);

		const row = await importAndFindRow(page, name);
		// Aperçu d'un PDF : l'<iframe> et son `blob:`, et le type servi par l'API.
		const served = page.waitForResponse((r) => r.url().includes('/source-document'));
		await row.getByTestId('imported-preview').click();
		expect((await served).headers()['content-type']).toBe('application/pdf');
		await expect(row.getByTestId('document-preview-frame')).toHaveAttribute('src', /^blob:/);

		page.once('dialog', (d) => d.accept());
		await row.getByTestId('imported-discard').click();
		await expect(page.getByTestId('imported-row').filter({ hasText: name })).toHaveCount(0);

		// Le même fichier, redéposé sous un autre nom, réactive la ligne retirée : elle garde
		// son nom d'origine.
		await deposit(`e2e-e3-retour-${token()}.pdf`, bytes);
		await importAndFindRow(page, name);
	});
});

/** Le dossier des captures : `KESH_E2E_CAPTURES_DIR`, sinon la sortie du test (hors dépôt). */
function capturePath(testInfo: TestInfo, name: string): string {
	const dossier = process.env.KESH_E2E_CAPTURES_DIR;
	return dossier ? path.join(dossier, `${name}.png`) : testInfo.outputPath(`${name}.png`);
}

/** Capture la page en clair puis en sombre (préférence du navigateur), puis revient au clair. */
async function captureBothSchemes(page: Page, testInfo: TestInfo, name: string) {
	for (const [colorScheme, suffixe] of [
		['light', 'clair'],
		['dark', 'sombre'],
	] as const) {
		await page.emulateMedia({ colorScheme });
		await page.screenshot({ path: capturePath(testInfo, `${name}-${suffixe}`), fullPage: true });
	}
	await page.emulateMedia({ colorScheme: 'light' });
}

/** Lit une facture fournisseur et le statut HTTP de son document, par l'API. */
async function invoiceAndDocumentStatus(page: Page, invoiceId: number) {
	const ctx = await authedApiContext(page);
	try {
		const inv = await (await ctx.get(`/api/v1/supplier-invoices/${invoiceId}`)).json();
		const doc = await ctx.get(`/api/v1/supplier-invoices/${invoiceId}/source-document`);
		return { inv, docStatus: doc.status() };
	} finally {
		await disposeContextSafe(ctx);
	}
}

/** L'id de la facture de la fiche ouverte (`/supplier-invoices/<id>`). */
function invoiceIdFromUrl(page: Page): number {
	const m = /\/supplier-invoices\/(\d+)$/.exec(new URL(page.url()).pathname);
	expect(m, `URL de fiche attendue : ${page.url()}`).not.toBeNull();
	return Number(m![1]);
}

test.describe('Créer la facture à côté du document (Story 30-2b)', () => {
	test('E1 — avec QR : coordonnées verrouillées, fournisseur proposé et retenu, la facture porte le document', async ({
		page,
	}, testInfo) => {
		await login(page);
		const expenseAccountId = await ensurePayableAndExpense(page);
		// Le fournisseur proposé : au NOM EXACT du créancier de la fixture.
		const supplierId = await createSupplierViaApi(page, 'Fournisseur E2E SA');
		const name = `e2e-e1-${token()}.png`;
		await deposit(name, uniqueCopy(await readFixture('spc_e2e_invoice.png'), 'png'));

		const row = await importAndFindRow(page, name);
		await row.getByTestId('imported-create-invoice').click();
		await expect(page).toHaveURL(/\/supplier-invoices\/new\?document=\d+$/);

		// L'aperçu est là pendant la saisie.
		const preview = page.getByTestId('supplier-invoice-new-preview');
		await expect(preview.getByTestId('document-preview-image')).toHaveAttribute('src', /^blob:/);

		// Coordonnées du QR en lecture seule, pas de scan.
		await expect(page.getByTestId('supplier-invoice-iban')).toHaveValue('CH9300762011623852957');
		await expect(page.getByTestId('supplier-invoice-iban')).toHaveAttribute('readonly', '');
		await expect(page.getByTestId('supplier-invoice-expected-amount')).toHaveAttribute('readonly', '');
		await expect(page.getByTestId('supplier-invoice-scan-btn')).toHaveCount(0);

		// Fournisseur PROPOSÉ, jamais présélectionné ; « Retenir » désigné par son nom.
		const proposal = page
			.getByTestId('supplier-proposal')
			.filter({ hasText: 'Fournisseur E2E SA' })
			.first();
		await expect(proposal).toBeVisible();
		await expect(page.getByTestId('supplier-invoice-supplier')).not.toHaveValue(String(supplierId));
		await proposal.getByTestId('supplier-proposal-retain').click();
		await expect(page.getByTestId('supplier-invoice-supplier')).toHaveValue(String(supplierId));

		// Lignes égales au montant du QR (100.00, TVA 0) : plus d'écart signalé.
		await fillFirstInvoiceLine(page, 'Prestation E1', '100.00', expenseAccountId);
		await expect(page.getByTestId('supplier-invoice-mismatch')).toHaveCount(0);
		await expect(preview).toBeVisible();
		await captureBothSchemes(page, testInfo, '302b-cote-a-cote');

		await page.getByTestId('supplier-invoice-submit').click();
		await expect(page).toHaveURL(/\/supplier-invoices\/\d+$/);
		const { inv, docStatus } = await invoiceAndDocumentStatus(page, invoiceIdFromUrl(page));
		expect(inv.contactId).toBe(supplierId);
		expect(inv.creditorIban).toBe('CH9300762011623852957');
		expect(docStatus, 'la facture créée porte le document').toBe(200);

		await page.goto('/supplier-invoices/import');
		await expect(page.getByTestId('imported-row').filter({ hasText: name })).toHaveCount(0);
	});

	test('E2 — sans QR : IBAN et référence saisis, fournisseur créé dans le panneau, aperçu toujours visible', async ({
		page,
	}, testInfo) => {
		await login(page);
		const expenseAccountId = await ensurePayableAndExpense(page);
		const name = `e2e-e2-${token()}.png`;
		await deposit(name, uniqueCopy(PNG_SANS_QR, 'png'));

		const row = await importAndFindRow(page, name);
		await row.getByTestId('imported-create-invoice').click();
		await expect(page).toHaveURL(/\/supplier-invoices\/new\?document=\d+$/);
		const preview = page.getByTestId('supplier-invoice-new-preview');
		await expect(preview.getByTestId('document-preview-image')).toBeVisible();
		await expect(page.getByTestId('supplier-invoice-qr-locked')).toHaveCount(0);

		// IBAN valide synthétique, saisi en groupes ; référence libre.
		await page.getByTestId('supplier-invoice-iban').fill('CH56 0483 5012 3456 7800 9');
		await page.getByTestId('supplier-invoice-reference').fill('Facture E2-302b');
		await fillFirstInvoiceLine(page, 'Prestation E2', '42.00', expenseAccountId);

		// Le fournisseur se crée sans quitter la saisie.
		const supplierName = `Fournisseur Panneau ${token()}`;
		await page.getByTestId('supplier-new-open').click();
		await expect(page.getByTestId('supplier-new-panel')).toBeVisible();
		await expect(preview).toBeVisible();
		await page.getByTestId('supplier-new-name').fill(supplierName);
		await captureBothSchemes(page, testInfo, '302b-panneau-fournisseur');
		// Entrée dans le nom crée le FOURNISSEUR, jamais la facture : le panneau vit dans
		// le formulaire de la facture, dont la soumission implicite est interceptée
		// (revue P1, B1 = E1 = A1 ; C-30-2-41). Le clic sur « Créer le fournisseur » est
		// couvert par Vitest.
		await page.getByTestId('supplier-new-name').press('Enter');
		await expect(page.getByTestId('supplier-new-panel')).toHaveCount(0);
		await expect(page).toHaveURL(/\/supplier-invoices\/new\?document=\d+$/);
		await expect(page.getByTestId('supplier-invoice-form-error')).toHaveCount(0);
		await expect(page.getByTestId('supplier-invoice-supplier').locator('option:checked')).toHaveText(
			supplierName,
		);

		// Saisie intacte, aperçu toujours là.
		await expect(page.getByTestId('supplier-invoice-iban')).toHaveValue('CH56 0483 5012 3456 7800 9');
		await expect(page.getByTestId('supplier-invoice-line-desc').first()).toHaveValue('Prestation E2');
		await expect(preview).toBeVisible();

		await page.getByTestId('supplier-invoice-submit').click();
		await expect(page).toHaveURL(/\/supplier-invoices\/\d+$/);
		// La fiche montre l'IBAN saisi, normalisé par le serveur.
		await expect(page.getByText('CH5604835012345678009')).toBeVisible();
		const { inv, docStatus } = await invoiceAndDocumentStatus(page, invoiceIdFromUrl(page));
		expect(inv.creditorIban).toBe('CH5604835012345678009');
		expect(inv.paymentReference).toBe('Facture E2-302b');
		expect(docStatus, 'la facture créée porte le document').toBe(200);
	});

	test('E4 — sans document : « Enregistrer une facture » mène à la page, la facture se crée sans pièce', async ({
		page,
	}) => {
		await login(page);
		const expenseAccountId = await ensurePayableAndExpense(page);
		const supplierId = await createSupplierViaApi(page, `Fournisseur E4 ${token()}`);

		await page.goto('/supplier-invoices');
		await page.getByTestId('supplier-invoice-new').click();
		await expect(page).toHaveURL(/\/supplier-invoices\/new$/);
		await expect(page.getByTestId('supplier-invoice-new-preview')).toHaveCount(0);
		await expect(page.getByTestId('supplier-invoice-scan-btn')).toBeVisible();

		await page.getByTestId('supplier-invoice-supplier').selectOption(String(supplierId));
		await fillFirstInvoiceLine(page, 'Prestation E4', '30.00', expenseAccountId);
		await page.getByTestId('supplier-invoice-submit').click();
		await expect(page).toHaveURL(/\/supplier-invoices\/\d+$/);
		const { inv, docStatus } = await invoiceAndDocumentStatus(page, invoiceIdFromUrl(page));
		expect(inv.contactId).toBe(supplierId);
		expect(docStatus, 'une facture sans document n’en a pas').toBe(404);
	});
});
