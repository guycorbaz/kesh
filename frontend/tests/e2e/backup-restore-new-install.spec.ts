import { expect, test } from '@playwright/test';
import type { Page, TestInfo } from '@playwright/test';
import { createHash } from 'node:crypto';
import { promises as fs } from 'node:fs';
import path from 'node:path';
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
import {
	DOCUMENTS_DIR,
	assertInboxDirsUsable,
	deposit,
	importViaApi,
	readFixture,
	token,
	uniqueCopy,
} from './helpers/inbox';

/**
 * Story 31-1b (#503, AC20) — restaurer une installation neuve depuis l'assistant, de bout
 * en bout : une comptabilité avec ses deux sortes de pièces (un document fournisseur passé
 * par le dossier d'import, le PDF figé d'une facture), exportée par l'écran ; une
 * installation remise à neuf dont le dossier des documents a PERDU ces deux pièces ;
 * `/setup`, le lien « Restaurer une sauvegarde » de l'assistant, l'import ; la
 * comptabilité revient — la facture, son écriture —, et les deux pièces avec elle.
 *
 * Seul un E2E le prouve : le fichier traverse deux fois la frontière HTTP (téléchargé par
 * le navigateur, renvoyé par le formulaire), et les pièces arrivent dans un dossier que
 * seuls le runner et le backend voient.
 *
 * ⚠️ **Aucun `test.skip`, aucun test qui se taise.** Les pièces sont supprimées du dossier
 * des documents avec une garde bruyante : chacune doit exister avant, et ne plus exister
 * après — un runner qui ne voit pas le dossier du backend (`KESH_DOCUMENTS_DIR`) échoue
 * ici au lieu de « réussir » sur des fichiers jamais supprimés.
 *
 * Les deux autres parcours portent les états de l'écran que le premier n'atteint pas (pièce
 * omise à l'export, pièce manquante après l'import, plafond d'import dépassé, réponse
 * perdue) et en prennent les captures (AC21, thème clair) quand `KESH_E2E_CAPTURES_DIR` est
 * posé. Le plafond et la réponse perdue passent par l'interception réseau : une archive de
 * plus de 2048 Mio ou une coupure réelle ne se fabriquent pas en E2E.
 */

const ADMIN = { username: 'admin', password: 'admin123' };

test.afterEach(async ({ page }) => {
	await clearAuthStorage(page);
});

test.afterAll(async () => {
	// Rendre l'état attendu par les specs suivantes, quelle que soit l'issue.
	await seedTestState('with-company');
});

async function login(page: Page, username = ADMIN.username, password = ADMIN.password) {
	await page.goto('/login');
	await page.fill('#username', username);
	await page.fill('#password', password);
	await page.click('button[type="submit"]');
	await expect(page).toHaveURL('/');
}

function sha256(bytes: Buffer): string {
	return createHash('sha256').update(bytes).digest('hex');
}

async function exists(file: string): Promise<boolean> {
	try {
		await fs.access(file);
		return true;
	} catch {
		return false;
	}
}

/** Supprime une pièce du dossier des documents — bruyamment si elle n'y était pas. */
async function removePiece(file: string): Promise<void> {
	expect(
		await exists(file),
		`${file} absent du dossier des documents avant suppression : le runner et le backend ` +
			`ne voient pas le même KESH_DOCUMENTS_DIR (runner : ${DOCUMENTS_DIR})`,
	).toBe(true);
	await fs.rm(file);
	expect(await exists(file), `${file} toujours présent après suppression`).toBe(false);
}

/** Capture (thème clair) si `KESH_E2E_CAPTURES_DIR` est posé ; sinon, rien. */
async function capture(page: Page, name: string): Promise<void> {
	const dossier = process.env.KESH_E2E_CAPTURES_DIR;
	if (!dossier) return;
	await page.emulateMedia({ colorScheme: 'light' });
	await page.screenshot({ path: path.join(dossier, `${name}.png`) });
}

/** Exporte par l'écran et enregistre le fichier dans la sortie du test. */
async function exportByScreen(page: Page, testInfo: TestInfo): Promise<string> {
	await page.goto('/admin/backup');
	const downloadPromise = page.waitForEvent('download');
	await page.getByTestId('admin-backup-export-button').click();
	const download = await downloadPromise;
	const file = testInfo.outputPath(`installation-${token()}.keshbackup`);
	await download.saveAs(file);
	await expect(page.getByTestId('admin-backup-export-button')).toBeEnabled();
	return file;
}

/** Choisit le fichier, ouvre la confirmation, confirme. */
async function importByScreen(page: Page, file: string): Promise<void> {
	await page.getByTestId('admin-restore-file-input').setInputFiles(file);
	await page.getByTestId('admin-restore-import-button').click();
	await page.getByTestId('admin-restore-confirm').click();
}

/** Crée une facture validée et fige son PDF ; rend l'id et les octets du PDF. */
async function invoiceWithFrozenPdf(page: Page, label: string) {
	await ensurePrimaryBankAccountViaApi(page);
	const contactId = await createContactWithAddressViaApi(
		page,
		`${label} ${Date.now()}-${Math.floor(Math.random() * 1e6)}`,
	);
	const invoiceId = await createAndValidateInvoiceViaApi(page, contactId);
	const ctx = await authedApiContext(page);
	try {
		const pdf = await ctx.get(`/api/v1/invoices/${invoiceId}/pdf`);
		expect(pdf.status()).toBe(200);
		return { invoiceId, pdf: await pdf.body() };
	} finally {
		await disposeContextSafe(ctx);
	}
}

test.describe('Restaurer une installation neuve depuis l’assistant (Story 31-1b)', () => {
	test('AC20 — la comptabilité et ses pièces reviennent sur une installation neuve', async ({
		page,
	}, testInfo) => {
		test.setTimeout(180_000);
		await seedTestState('with-company');
		await assertInboxDirsUsable();
		await login(page);

		// (1) Un document fournisseur passé par le dossier d'import (octets propres au spec)…
		const docName = `e2e-restore-${token()}.png`;
		const docBytes = uniqueCopy(await readFixture('spc_e2e_invoice.png'), 'png');
		await deposit(docName, docBytes);
		const report = await importViaApi(page);
		expect(
			report.accepted.find((a) => a.fileName === docName),
			`${docName} absent du rapport d'import : ${JSON.stringify(report)}`,
		).toBeTruthy();
		const docFile = path.join(DOCUMENTS_DIR, `${sha256(docBytes)}.png`);

		// … et une facture validée dont le PDF est figé par un téléchargement.
		const { invoiceId, pdf: pdfBefore } = await invoiceWithFrozenPdf(page, 'Restauration');
		const pdfFile = path.join(DOCUMENTS_DIR, `${sha256(pdfBefore)}.pdf`);

		const ctx = await authedApiContext(page);
		let invoiceBefore: { invoiceNumber: string; journalEntryId: number; totalAmount: string };
		let entryBefore: { lines: unknown[] };
		try {
			invoiceBefore = await (await ctx.get(`/api/v1/invoices/${invoiceId}`)).json();
			expect(invoiceBefore.invoiceNumber, 'facture validée : numéro attendu').toBeTruthy();
			expect(invoiceBefore.journalEntryId, 'facture validée : écriture attendue').toBeTruthy();
			entryBefore = await (
				await ctx.get(`/api/v1/journal-entries/${invoiceBefore.journalEntryId}`)
			).json();
			expect(entryBefore.lines.length).toBeGreaterThan(0);
		} finally {
			await disposeContextSafe(ctx);
		}

		// (2) Export par l'écran : avis permanent sur le .env, aucune pièce omise.
		const backup = await exportByScreen(page, testInfo);
		await expect(page.getByTestId('backup-env-notice')).toBeVisible();
		await expect(page.getByTestId('admin-backup-files-not-included')).toHaveCount(0);
		await expect(page.getByTestId('admin-backup-exceeds-import-limit')).toHaveCount(0);
		await capture(page, 'c-backup-panneau-avis-env');

		// (3) Installation remise à neuf, et les deux pièces perdues.
		await clearAuthStorage(page);
		await seedTestState('setup-required');
		await removePiece(docFile);
		await removePiece(pdfFile);

		// (4) /setup (un nouvel administrateur), l'assistant, le lien de restauration.
		await page.goto('/setup');
		const setupPassword = 'restauration-e2e-12';
		await page.getByTestId('setup-username').fill('nouvel-admin');
		await page.getByTestId('setup-password').fill(setupPassword);
		await page.getByTestId('setup-password-confirm').fill(setupPassword);
		await page.getByTestId('setup-submit').click();
		await page.waitForURL(/\/onboarding$/, { timeout: 10_000 });
		const link = page.getByTestId('onboarding-restore-link');
		await expect(link).toBeVisible();
		await capture(page, 'a-assistant-etape-0-lien-restaurer');
		await link.click();
		await expect(page).toHaveURL(/\/onboarding\/restore$/);
		await expect(page.getByTestId('backup-env-notice')).toBeVisible();
		await capture(page, 'b-onboarding-restore');

		await importByScreen(page, backup);

		// (5) Ce que la restauration a rendu, lu AVANT « Se reconnecter » (aucune redirection
		// avant le clic, C-31-1-39).
		await expect(page.getByTestId('admin-restore-files-restored')).toContainText('2', {
			timeout: 60_000,
		});
		await expect(page.getByTestId('admin-restore-files-missing')).toHaveCount(0);
		await expect(page).toHaveURL(/\/onboarding\/restore$/);
		await capture(page, 'g-fin-import-sans-piece-manquante');
		await page.getByTestId('admin-restore-relogin').click();
		await page.waitForURL(/\/login/);

		// … reconnexion avec les identifiants DE LA SAUVEGARDE.
		await clearAuthStorage(page);
		await login(page);

		// (6) L'application s'ouvre (pas l'assistant), la comptabilité est revenue.
		await page.goto('/invoices');
		await expect(page.getByRole('cell', { name: invoiceBefore!.invoiceNumber, exact: true })).toBeVisible();

		const ctxAfter = await authedApiContext(page);
		try {
			const invoiceAfter = await (await ctxAfter.get(`/api/v1/invoices/${invoiceId}`)).json();
			expect(invoiceAfter.invoiceNumber).toBe(invoiceBefore!.invoiceNumber);
			expect(invoiceAfter.totalAmount).toBe(invoiceBefore!.totalAmount);
			expect(invoiceAfter.journalEntryId).toBe(invoiceBefore!.journalEntryId);
			const entryAfter = await (
				await ctxAfter.get(`/api/v1/journal-entries/${invoiceBefore!.journalEntryId}`)
			).json();
			expect(entryAfter.lines).toEqual(entryBefore!.lines);

			// Le PDF de la facture se télécharge, octets égaux à ceux d'avant.
			const pdfAfter = await ctxAfter.get(`/api/v1/invoices/${invoiceId}/pdf`);
			expect(pdfAfter.status()).toBe(200);
			expect((await pdfAfter.body()).equals(pdfBefore)).toBe(true);
		} finally {
			await disposeContextSafe(ctxAfter);
		}

		// Le document fournisseur s'affiche dans son aperçu.
		await page.goto('/supplier-invoices/import');
		const row = page.getByTestId('imported-row').filter({ hasText: docName });
		await expect(row).toHaveCount(1);
		await row.getByTestId('imported-preview').click();
		await expect(row.getByTestId('document-preview-image')).toHaveAttribute('src', /^blob:/);

		// Les deux pièces existent de nouveau dans le dossier des documents.
		expect(await exists(docFile), `${docFile} non restauré`).toBe(true);
		expect(await exists(pdfFile), `${pdfFile} non restauré`).toBe(true);
		expect(sha256(await fs.readFile(pdfFile))).toBe(sha256(pdfBefore));
	});

	test('AC18/AC19 — une pièce absente à l’export est annoncée, puis manquante après l’import', async ({
		page,
	}, testInfo) => {
		test.setTimeout(120_000);
		await seedTestState('with-company');
		await assertInboxDirsUsable();
		await login(page);

		const { pdf } = await invoiceWithFrozenPdf(page, 'Pièce omise');
		const pdfFile = path.join(DOCUMENTS_DIR, `${sha256(pdf)}.pdf`);
		// La pièce disparaît AVANT l'export : l'archive ne la porte pas.
		await removePiece(pdfFile);

		const backup = await exportByScreen(page, testInfo);
		await expect(page.getByTestId('admin-backup-files-not-included')).toContainText('1');
		await capture(page, 'e-export-pieces-non-incluses');

		// Import de cette archive sur la même installation, la pièce toujours absente.
		await page.goto('/admin/restore');
		await expect(page.getByTestId('backup-env-notice')).toBeVisible();
		await capture(page, 'd-restore-panneau-avis-env');
		await importByScreen(page, backup);
		await expect(page.getByTestId('admin-restore-files-missing')).toContainText('1', {
			timeout: 60_000,
		});
		await expect(page.getByTestId('admin-restore-relogin')).toBeVisible();
		await expect(page).toHaveURL(/\/admin\/restore$/);
		await capture(page, 'h-fin-import-pieces-manquantes');
		await page.getByTestId('admin-restore-relogin').click();
		await page.waitForURL(/\/login/);
		expect(await exists(pdfFile), 'l’import ne fabrique pas une pièce qu’il n’a pas').toBe(false);
	});

	test('AC18/AC18b — plafond d’import dépassé, réponse de l’import perdue (interception réseau)', async ({
		page,
	}) => {
		await seedTestState('with-company');
		await login(page);

		// L'en-tête de la 31-1a, ajouté à la réponse réelle de l'export.
		await page.route('**/api/v1/admin/full-export', async (route) => {
			const response = await route.fetch();
			await route.fulfill({
				response,
				headers: { ...response.headers(), 'x-kesh-backup-exceeds-import-limit': '2048' },
			});
		});
		await page.goto('/admin/backup');
		const downloadPromise = page.waitForEvent('download');
		await page.getByTestId('admin-backup-export-button').click();
		await downloadPromise;
		await expect(page.getByTestId('admin-backup-exceeds-import-limit')).toContainText('2048 Mio');
		await capture(page, 'f-export-depasse-taille-import');

		// Un proxy inverse qui abandonne l'attente : 504, rien n'atteint Kesh.
		let reached = false;
		await page.route('**/api/v1/admin/full-import', async (route) => {
			reached = true;
			await route.fulfill({ status: 504, contentType: 'text/html', body: '<h1>504 Gateway Time-out</h1>' });
		});
		await page.goto('/admin/restore');
		await page.getByTestId('admin-restore-file-input').setInputFiles({
			name: 'installation.keshbackup',
			mimeType: 'application/octet-stream',
			buffer: Buffer.from('PK'),
		});
		await page.getByTestId('admin-restore-import-button').click();
		await page.getByTestId('admin-restore-confirm').click();
		const error = page.getByTestId('admin-restore-error');
		await expect(error).toContainText("La réponse du serveur n'est pas arrivée");
		await expect(error).not.toContainText('Erreur serveur');
		await expect(page.getByTestId('admin-restore-relogin')).toBeVisible();
		expect(reached).toBe(true);
		await capture(page, 'i-import-issue-inconnue');
	});
});
