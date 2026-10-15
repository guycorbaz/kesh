import { expect, test } from '@playwright/test';
import type { Page } from '@playwright/test';
import { seedTestState, clearAuthStorage } from './helpers/test-state';
import {
	PDF_SANS_QR,
	PNG_SANS_QR,
	assertInboxDirsUsable,
	deposit,
	inboxHandshake,
	readFixture,
	stillInInbox,
	token,
	uniqueCopy,
} from './helpers/inbox';

/**
 * Parcours « Documents à traiter » — Story 30-2a (#685), parcours E1-a, E2-a et E3 de la
 * fiche. Guy ne testera qu'en fin d'epic : ces specs portent la recette.
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
