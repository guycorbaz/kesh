import { expect, test } from '@playwright/test';
import { promises as fs } from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import {
	seedTestState,
	clearAuthStorage,
	authedApiContext,
	disposeContextSafe,
} from './helpers/test-state';

/**
 * Tests E2E — Import de factures fournisseurs depuis un dossier (Story 12.5d, #194).
 *
 * Flux UI consommant les endpoints livrés par 12-5c. Le pipeline d'import lui-même
 * (décodage QR, sécurité) est couvert par les 16 tests d'intégration backend
 * `inbox_import_e2e.rs` — ici on valide l'écran, le rapport, la liste, la
 * complétion, l'écart et le download.
 *
 * **Seed du round-trip (DC-d4)** : le test dépose une fixture QR PNG pré-commitée
 * (`fixtures/spc_e2e_invoice.png`, image → rxing, pas de pdfium) dans le dossier
 * inbox du serveur de test, puis déclenche l'import réel. Le chemin inbox est lu
 * via `process.env.KESH_INBOX_DIR` — le harness doit démarrer le backend avec
 * cette variable pointant un dossier inscriptible par le runner (ex.
 * `KESH_INBOX_DIR=/tmp/kesh-e2e-inbox`). Si la variable est absente, les
 * scénarios dépendant de l'import réel sont `skip` (fallback documenté DC-d4).
 */

const INBOX_DIR = process.env.KESH_INBOX_DIR;
const FIXTURE = path.join(
	path.dirname(fileURLToPath(import.meta.url)),
	'fixtures',
	'spc_e2e_invoice.png',
);

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
		}>;
		const payable = accounts.find((a) => a.number === '2000');
		const expense = accounts.find((a) => a.accountType === 'Expense' && a.active);
		expect(payable, 'compte 2000 attendu').toBeTruthy();
		expect(expense, 'un compte de charge attendu').toBeTruthy();
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

test.describe('Import de factures depuis un dossier', () => {
	test('écran accessible, import sur dossier vide → rapport sans erreur', async ({ page }) => {
		await login(page);
		// Vider l'inbox si on connaît son chemin (sinon le dossier par défaut est
		// supposé vide en environnement de test).
		if (INBOX_DIR) {
			await fs.rm(INBOX_DIR, { recursive: true, force: true }).catch(() => {});
			await fs.mkdir(INBOX_DIR, { recursive: true });
		}
		await page.goto('/supplier-invoices/import');
		await expect(page.getByRole('heading', { level: 1 })).toContainText(/Importer/i);

		await page.getByTestId('inbox-import-trigger').click();
		// Le rapport s'affiche (0 créée sur dossier vide), pas de crash.
		await expect(page.getByTestId('inbox-import-report')).toBeVisible();
		await expect(page.getByTestId('imported-empty')).toBeVisible();
	});

	test('round-trip : import fixture → liste → download → complétion', async ({ page }) => {
		test.skip(
			!INBOX_DIR,
			'KESH_INBOX_DIR non défini — round-trip import réel ignoré (DC-d4 fallback ; pipeline couvert par inbox_import_e2e.rs).',
		);
		await login(page);
		const expenseAccountId = await ensureConfigAndExpense(page);
		const supplierId = await createSupplier(page);

		// Déposer la fixture QR PNG dans l'inbox du serveur de test.
		await fs.mkdir(INBOX_DIR!, { recursive: true });
		await fs.copyFile(FIXTURE, path.join(INBOX_DIR!, `e2e-${Date.now()}.png`));

		await page.goto('/supplier-invoices/import');
		await page.getByTestId('inbox-import-trigger').click();
		await expect(page.getByTestId('inbox-import-report')).toBeVisible();

		// L'importée apparaît dans la liste.
		const row = page.getByTestId('imported-row').first();
		await expect(row).toBeVisible();
		await expect(row).toContainText('Fournisseur E2E SA');

		// Download du justificatif (le fichier vient d'être archivé → pas de 410).
		const downloadPromise = page.waitForEvent('download');
		await row.getByTestId('imported-download').click();
		await downloadPromise;

		// Compléter : fournisseur + 1 ligne au montant cible (100.00, TVA 0).
		await row.getByTestId('imported-complete-open').click();
		await row.getByTestId('imported-supplier-select').selectOption(String(supplierId));
		const lineInputs = row.locator('input[inputmode="decimal"]');
		await lineInputs.nth(0).fill('1'); // quantité
		await lineInputs.nth(1).fill('100.00'); // PU HT
		await row.locator('input[placeholder="Description"]').first().fill('Prestation E2E');
		await row.locator('select').nth(2).selectOption(String(expenseAccountId)); // compte de charge
		await row.getByTestId('imported-complete-submit').click();

		// La ligne disparaît du worklist (complétée), on reste sur l'écran d'import.
		await expect(page).toHaveURL(/\/supplier-invoices\/import$/);
		await expect(page.getByTestId('imported-row')).toHaveCount(0);
	});
});
