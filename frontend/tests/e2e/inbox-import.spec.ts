import { expect, test } from '@playwright/test';
import { promises as fs } from 'node:fs';
import { seedTestState, clearAuthStorage } from './helpers/test-state';
import {
	INBOX_DIR,
	assertInboxDirsUsable,
	createSupplierViaApi,
	deposit,
	ensurePayableAndExpense,
	fillFirstInvoiceLine,
	inboxHandshake,
	readFixture,
	token,
	uniqueCopy,
} from './helpers/inbox';

/**
 * Tests E2E — « Documents à traiter » : l'import du dossier (Story 12.5d, #194 ; écran
 * renommé par la Story 30-2a, #685).
 *
 * Flux UI consommant les endpoints livrés par 12-5c. Le pipeline d'import lui-même
 * (décodage QR, sécurité, documents sans QR) est couvert par les tests d'intégration
 * backend `inbox_import_e2e.rs` — ici on valide l'écran, le rapport, la liste, la
 * création de la facture (Story 30-2b : sur la page `/supplier-invoices/new?document=`), le
 * retrait et le téléchargement.
 *
 * **Seed du round-trip (DC-d4)** : le test dépose une copie de la fixture QR PNG
 * pré-commitée (`fixtures/spc_e2e_invoice.png`, image → rxing, pas de pdfium) dans le
 * dossier d'import du serveur de test, puis déclenche l'import réel.
 *
 * ⚠️ Story 30-2a : **plus de `test.skip`**. Le dossier se lit dans `helpers/inbox.ts`, avec
 * pour défaut celui de la recette documentée (`/tmp/kesh-e2e/inbox`) ; un dossier absent
 * fait échouer le `beforeAll` en le nommant, un backend qui en lit un autre fait échouer la
 * poignée de main. Le fallback « skip » de DC-d4 laissait la spec se taire.
 */

test.beforeAll(async () => {
	await seedTestState('with-company');
	await assertInboxDirsUsable();
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

test.describe('Documents à traiter — import du dossier', () => {
	test('écran accessible, import sur dossier vide → rapport sans erreur', async ({ page }) => {
		await login(page);
		// Vider le dossier d'import : l'écran doit rester vide après un import à vide.
		await fs.rm(INBOX_DIR, { recursive: true, force: true }).catch(() => {});
		await fs.mkdir(INBOX_DIR, { recursive: true });
		await page.goto('/supplier-invoices/import');
		await expect(page.getByRole('heading', { level: 1 })).toContainText('Documents à traiter');

		await page.getByTestId('inbox-import-trigger').click();
		// Le rapport s'affiche (0 document sur dossier vide), pas de crash.
		await expect(page.getByTestId('inbox-import-report')).toBeVisible();
		await expect(page.getByTestId('imported-empty')).toBeVisible();
	});

	test('round-trip : import fixture → liste → download → création de la facture', async ({ page }) => {
		await login(page);
		await inboxHandshake(page);
		const expenseAccountId = await ensurePayableAndExpense(page);
		const supplierId = await createSupplierViaApi(page, uniq('Fournisseur E2E'));

		// Déposer une copie unique de la fixture QR PNG dans le dossier du serveur de test.
		const name = `e2e-${token()}.png`;
		await deposit(name, uniqueCopy(await readFixture('spc_e2e_invoice.png'), 'png'));

		await page.goto('/supplier-invoices/import');
		await page.getByTestId('inbox-import-trigger').click();
		await expect(page.getByTestId('inbox-import-report')).toBeVisible();

		// Le document apparaît dans la liste, avec son QR.
		const row = page.getByTestId('imported-row').filter({ hasText: name });
		await expect(row).toBeVisible();
		await expect(row).toContainText('Fournisseur E2E SA');

		// Download du document (le fichier vient d'être archivé → pas de 410).
		const downloadPromise = page.waitForEvent('download');
		await row.getByTestId('imported-download').click();
		await downloadPromise;

		// Story 30-2b : « Créer une facture » mène à la page de création, à côté du document.
		await row.getByTestId('imported-create-invoice').click();
		await expect(page).toHaveURL(/\/supplier-invoices\/new\?document=\d+$/);
		// Sans fournisseur, l'enregistrement est refusé à l'écran, saisie intacte.
		await fillFirstInvoiceLine(page, 'Prestation E2E', '100.00', expenseAccountId);
		await page.getByTestId('supplier-invoice-submit').click();
		await expect(page.getByTestId('supplier-invoice-form-error')).toBeVisible();
		await page.getByTestId('supplier-invoice-supplier').selectOption(String(supplierId));
		await page.getByTestId('supplier-invoice-submit').click();

		// L'écran va à la fiche de la facture créée ; le document a quitté la liste.
		await expect(page).toHaveURL(/\/supplier-invoices\/\d+$/);
		await page.goto('/supplier-invoices/import');
		await expect(page.getByTestId('imported-row').filter({ hasText: name })).toHaveCount(0);
	});

	test('retirer un document de la liste → disparaît', async ({ page }) => {
		await login(page);

		// Fixture au contenu distinct (QR différent de celui du round-trip).
		const name = `e2e-discard-${token()}.png`;
		await deposit(name, uniqueCopy(await readFixture('spc_e2e_discard.png'), 'png'));

		await page.goto('/supplier-invoices/import');
		await page.getByTestId('inbox-import-trigger').click();
		await expect(page.getByTestId('inbox-import-report')).toBeVisible();

		const row = page.getByTestId('imported-row').filter({ hasText: name });
		await expect(row).toBeVisible();
		await expect(row).toContainText('Fournisseur Discard SA');

		// La confirmation `confirm()` du navigateur est acceptée automatiquement.
		page.once('dialog', (d) => d.accept());
		await row.getByTestId('imported-discard').click();

		await expect(page.getByTestId('imported-row').filter({ hasText: name })).toHaveCount(0);
	});
});
