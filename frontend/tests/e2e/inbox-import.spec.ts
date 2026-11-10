import { expect, test } from '@playwright/test';
import { promises as fs } from 'node:fs';
import { seedTestState, clearAuthStorage } from './helpers/test-state';
import {
	INBOX_DIR,
	PNG_SANS_QR,
	assertInboxDirsUsable,
	copieIdentique,
	createSupplierViaApi,
	deposit,
	discardViaApi,
	ensurePayableAndExpense,
	fillFirstInvoiceLine,
	inboxHandshake,
	nomDansFailed,
	readFixture,
	retirerDeFailed,
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

/**
 * Story 34-9 (#969, AC-7) — le rapport nomme chaque fichier, et un doublon ne passe plus
 * inaperçu : le constat de production était un import mêlant documents neufs et déjà
 * importés, dont le rapport commençait par « N document(s) importé(s) ».
 */
test.describe('Documents à traiter — le rapport nomme chaque fichier (Story 34-9)', () => {
	/** Identifiant d'un document de la liste, lu sur l'ancre de sa ligne (`imported-<id>`). */
	async function idDeLaLigne(page: import('@playwright/test').Page, name: string): Promise<number> {
		const row = page.getByTestId('imported-row').filter({ hasText: name });
		await expect(row).toBeVisible();
		const id = await row.getAttribute('id');
		expect(id, `ancre de la ligne ${name}`).toMatch(/^imported-\d+$/);
		return Number(id!.slice('imported-'.length));
	}

	test('entre deux imports : un document neuf et une copie identique d’un document déjà importé', async ({
		page,
	}) => {
		await login(page);
		await inboxHandshake(page);
		const t = token();
		const bytesA = uniqueCopy(PNG_SANS_QR, 'png');
		const nameA = `e2e-349-a-${t}.png`;
		await deposit(nameA, bytesA);
		await page.goto('/supplier-invoices/import');
		await page.getByTestId('inbox-import-trigger').click();
		const idA = await idDeLaLigne(page, nameA);

		// Le même import : un document neuf B, et une copie identique de A.
		const nameB = `e2e-349-b-${t}.png`;
		const nameCopie = `e2e-349-a-copie-${t}.png`;
		await deposit(nameB, uniqueCopy(PNG_SANS_QR, 'png'));
		await deposit(nameCopie, copieIdentique(bytesA));
		await page.getByTestId('inbox-import-trigger').click();

		const report = page.getByTestId('inbox-import-report');
		const lignes = report.getByTestId('inbox-import-file-row');
		await expect(lignes).toHaveCount(2);
		await expect(lignes.filter({ hasText: nameB })).toHaveAttribute('data-state', 'imported-no-qr');
		const doublon = lignes.filter({ hasText: nameCopie });
		await expect(doublon).toHaveAttribute('data-state', 'duplicate');
		await expect(doublon).toContainText('Déjà importé');
		await expect(doublon).toContainText('en attente dans Documents à traiter');
		const moved = nomDansFailed(nameCopie, bytesA);
		await expect(doublon).toContainText(`failed/ sous le nom ${moved}`);
		await expect(doublon.getByTestId('inbox-import-duplicate-link')).toHaveAttribute(
			'href',
			`#imported-${idA}`,
		);
		await expect(report.getByTestId('inbox-import-summary')).toContainText(
			'1 document(s) importé(s), 1 déjà importé(s).',
		);
		// L'avertissement, visé par son type et non par son libellé traduit.
		await expect(page.locator('[data-sonner-toast][data-type="warning"]').first()).toBeVisible();

		// Le lien mène à la ligne du document existant.
		await doublon.getByTestId('inbox-import-duplicate-link').click();
		await expect(page).toHaveURL(new RegExp(`#imported-${idA}$`));
		await expect(page.locator(`#imported-${idA}`)).toBeInViewport();

		// Nettoyage : les deux documents retirés de la liste, la copie retirée de failed/.
		const idB = await idDeLaLigne(page, nameB);
		await discardViaApi(page, idA);
		await discardViaApi(page, idB);
		await retirerDeFailed(moved);
	});

	test('dans le même import : deux fichiers neufs de mêmes octets — un importé, un déjà importé', async ({
		page,
	}) => {
		await login(page);
		await inboxHandshake(page);
		const t = token();
		const bytes = uniqueCopy(PNG_SANS_QR, 'png');
		const name1 = `e2e-349-c-1-${t}.png`;
		const name2 = `e2e-349-c-2-${t}.png`;
		await deposit(name1, bytes);
		await deposit(name2, copieIdentique(bytes));
		await page.goto('/supplier-invoices/import');
		await page.getByTestId('inbox-import-trigger').click();

		const lignes = page
			.getByTestId('inbox-import-report')
			.getByTestId('inbox-import-file-row')
			.filter({ hasText: 'e2e-349-c-' });
		await expect(lignes).toHaveCount(2);
		// `read_dir` ne garantit aucun ordre : on asserte l'ensemble des états, non le nom
		// qui porte chacun (P2-9-4).
		const etats = (
			await Promise.all([0, 1].map((i) => lignes.nth(i).getAttribute('data-state')))
		).sort();
		expect(etats).toEqual(['duplicate', 'imported-no-qr']);
		await expect(page.locator('[data-sonner-toast][data-type="warning"]').first()).toBeVisible();

		// Le document importé est celui que vise le lien ; le doublon est dans failed/.
		const enDouble =
			(await lignes.filter({ hasText: name1 }).getAttribute('data-state')) === 'duplicate'
				? name1
				: name2;
		const importe = enDouble === name1 ? name2 : name1;
		const doublon = lignes.filter({ hasText: enDouble });
		await expect(doublon).toContainText('Déjà importé');
		const id = await idDeLaLigne(page, importe);
		await expect(doublon.getByTestId('inbox-import-duplicate-link')).toHaveAttribute(
			'href',
			`#imported-${id}`,
		);
		await discardViaApi(page, id);
		await retirerDeFailed(nomDansFailed(enDouble, bytes));
	});
});
