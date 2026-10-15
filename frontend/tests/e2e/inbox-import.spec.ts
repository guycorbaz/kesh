import { expect, test } from '@playwright/test';
import { promises as fs } from 'node:fs';
import {
	seedTestState,
	clearAuthStorage,
	authedApiContext,
	disposeContextSafe,
} from './helpers/test-state';
import {
	INBOX_DIR,
	assertInboxDirsUsable,
	deposit,
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
 * complétion, le retrait et le téléchargement.
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

/** Crée un fournisseur (is_supplier) via API et retourne son id. */
async function createSupplier(page: import('@playwright/test').Page): Promise<number> {
	const ctx = await authedApiContext(page);
	try {
		const res = await ctx.post('/api/v1/contacts', {
			data: {
				contactType: 'Entreprise',
				name: uniq('Fournisseur E2E'),
				isClient: false,
				isSupplier: true,
				address: 'Rue 2\n1000 Lausanne',
				defaultPaymentTerms: '30 jours net',
			},
		});
		expect(res.ok(), `createSupplier failed: ${res.status()}`).toBeTruthy();
		return (await res.json()).id as number;
	} finally {
		await disposeContextSafe(ctx);
	}
}

/** Configure default_payable (2000) si absent + retourne un compte de charge. */
async function ensureConfigAndExpense(page: import('@playwright/test').Page): Promise<number> {
	const ctx = await authedApiContext(page);
	try {
		const accounts = (await (await ctx.get('/api/v1/accounts')).json()) as Array<{
			id: number;
			number: string;
			accountType: string;
			active: boolean;
			postable: boolean;
		}>;
		const payable = accounts.find((a) => a.number === '2000');
		const expense = accounts.find((a) => a.accountType === 'Expense' && a.active && a.postable);
		expect(payable, 'compte 2000 attendu').toBeTruthy();
		expect(expense, 'un compte de charge attendu').toBeTruthy();
		// Le seed `with-company` (`seed_accounting_company`) ne désigne pas le compte
		// créanciers : on le pose ici quand il manque (Story 15-5d, AC6 — contournement
		// gardé ; il ne dépend pas de #521).
		const s = await (await ctx.get('/api/v1/company/invoice-settings')).json();
		if (s.defaultPayableAccountId == null) {
			const putRes = await ctx.put('/api/v1/company/invoice-settings', {
				data: { ...s, defaultPayableAccountId: payable!.id },
			});
			expect(putRes.ok(), `set payable failed: ${putRes.status()}`).toBeTruthy();
		}
		return expense!.id;
	} finally {
		await disposeContextSafe(ctx);
	}
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

	test('round-trip : import fixture → liste → download → complétion', async ({ page }) => {
		await login(page);
		await inboxHandshake(page);
		const expenseAccountId = await ensureConfigAndExpense(page);
		const supplierId = await createSupplier(page);

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

		// Compléter : fournisseur + 1 ligne au montant cible (100.00, TVA 0).
		await row.getByTestId('imported-complete-open').click();
		// Validation client (M2) : sans fournisseur, le bouton Valider est désactivé.
		await expect(row.getByTestId('imported-complete-submit')).toBeDisabled();
		await row.getByTestId('imported-supplier-select').selectOption(String(supplierId));
		const lineInputs = row.locator('input[inputmode="decimal"]');
		await lineInputs.nth(0).fill('1'); // quantité
		await lineInputs.nth(1).fill('100.00'); // PU HT
		await row.locator('input[placeholder="Description"]').first().fill('Prestation E2E');
		await row.locator('select').nth(2).selectOption(String(expenseAccountId)); // compte de charge
		await row.getByTestId('imported-complete-submit').click();

		// Le document quitte la liste (complété), on reste sur l'écran.
		await expect(page).toHaveURL(/\/supplier-invoices\/import$/);
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
