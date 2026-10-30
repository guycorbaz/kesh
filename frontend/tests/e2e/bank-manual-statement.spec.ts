import { expect, test, type Page, type TestInfo } from '@playwright/test';
import path from 'node:path';
import {
	seedTestState,
	clearAuthStorage,
	authedApiContext,
	disposeContextSafe,
} from './helpers/test-state';
import {
	createAndValidateInvoiceViaApi,
	createContactWithAddressViaApi,
	createUser,
	USER_PASSWORD,
	wiredBankAccountViaApi,
} from './helpers/api-fixtures';

/**
 * Tests E2E — saisir un relevé bancaire à la main (Story v016-2b, #816, AC-26).
 *
 * # Ce que ces tests prouvent, et que rien d'autre ne prouve
 *
 * Qu'un **Comptable**, depuis `/bank-import`, suit le lien de saisie, tape un relevé de
 * trois lignes, voit l'écart, le corrige, vérifie et enregistre ; que le relevé paraît
 * « Saisie manuelle » dans la liste ; que sa ligne de crédit est **proposée** et
 * **acceptée** au rapprochement contre la facture qu'elle règle ; et que le journal
 * d'audit porte l'action « Relevé bancaire saisi à la main » sur l'import créé. Que le
 * rôle **Consultation** ne voit pas le lien et que `/bank-import/manual` le renvoie.
 *
 * ⚠️ La base est **tronquée** par `seedTestState('with-company')` en `beforeAll`
 * (`helpers/test-state.ts`) ; le preset ne crée aucun compte bancaire : aucun relevé
 * antérieur, donc aucun avertissement de solde de départ ne peut surgir. La facture vaut
 * un TTC fixe (972.90, `createAndValidateInvoiceViaApi`) ; elle est datée du jour **local**
 * de Zurich (le piège UTC de minuit à 2 h), comme la ligne rapprochée ; les deux autres
 * lignes de la veille, sauf un 1ᵉʳ janvier (alors du jour aussi), dans l'exercice du seed
 * et dans la fenêtre de proposition (au plus 30 jours, Story 28-7).
 *
 * Le troisième test ne sert que la recette visuelle (AC-25) : il ne tourne que si
 * `KESH_E2E_CAPTURES_DIR` est posé.
 */

/** IBAN suisse synthétique fixe (valide ISO 13616), celui des autres specs bancaires. */
const IBAN_A = 'CH4431999123000889012';
/** Second IBAN synthétique valide, pour la recette visuelle. */
const IBAN_B = 'CH9300762011623852957';
/** TTC de la facture de `createAndValidateInvoiceViaApi` (4.5 × 200.00 + TVA 8.1 %). */
const TTC = '972.90';

test.beforeAll(async () => {
	await seedTestState('with-company');
});

test.afterEach(async ({ page }) => {
	await clearAuthStorage(page);
});

async function login(page: Page, username: string, password: string): Promise<void> {
	await page.goto('/login');
	await page.fill('#username', username);
	await page.fill('#password', password);
	await page.click('button[type="submit"]');
	await expect(page).toHaveURL('/');
}

/** Le jour local de Zurich, `AAAA-MM-JJ`. */
function todayZurich(): string {
	return new Date().toLocaleDateString('sv-SE', { timeZone: 'Europe/Zurich' });
}

/** La veille de `day`, sauf un 1ᵉʳ janvier (alors `day` : l'exercice du seed est l'année). */
function dayBefore(day: string): string {
	if (day.endsWith('-01-01')) return day;
	const d = new Date(`${day}T12:00:00Z`);
	d.setUTCDate(d.getUTCDate() - 1);
	return d.toISOString().slice(0, 10);
}

type Ligne = { date: string; details: string; debit?: string; credit?: string; reference?: string };

/** Remplit l'entête et les lignes du formulaire de saisie (déjà ouvert). */
async function remplir(
	page: Page,
	o: { bankAccountId: number; from: string; to: string; opening: string; closing: string; lignes: Ligne[] },
): Promise<void> {
	await page.getByTestId('bank-import-manual-account').selectOption(o.bankAccountId.toString());
	await page.getByTestId('bank-import-manual-period-from').fill(o.from);
	await page.getByTestId('bank-import-manual-period-to').fill(o.to);
	await page.getByTestId('bank-import-manual-opening-balance').fill(o.opening);
	await page.getByTestId('bank-import-manual-closing-balance').fill(o.closing);
	for (let i = 0; i < o.lignes.length; i++) {
		if (i > 0) await page.getByTestId('bank-import-manual-add-line').click();
		const ligne = page.getByTestId('bank-import-manual-line').nth(i);
		const l = o.lignes[i];
		await ligne.getByTestId('bank-import-manual-line-booking-date').fill(l.date);
		await ligne.getByTestId('bank-import-manual-line-details').fill(l.details);
		if (l.reference) await ligne.getByTestId('bank-import-manual-line-reference').fill(l.reference);
		if (l.debit) await ligne.getByTestId('bank-import-manual-line-debit').fill(l.debit);
		if (l.credit) await ligne.getByTestId('bank-import-manual-line-credit').fill(l.credit);
	}
}

/** Les trois lignes du parcours : le crédit de la facture du jour, deux débits de la veille. */
function troisLignes(today: string): Ligne[] {
	const veille = dayBefore(today);
	return [
		{ date: today, details: 'Virement client E2E', credit: TTC },
		{ date: veille, details: 'Frais bancaires', debit: '5.00' },
		{ date: veille, details: 'Retrait au guichet', debit: '100' },
	];
}

/** Accepte la proposition de la facture `invoiceId` au rapprochement du compte. */
async function rapprocher(page: Page, bankAccountId: number, invoiceId: number): Promise<void> {
	await page.goto('/reconciliation');
	await page.getByTestId('bank-account-select').selectOption(bankAccountId.toString());
	const ligne = page
		.getByTestId('reconciliation-row')
		.filter({ has: page.locator(`[data-testid="candidate-top-1"][data-invoice-id="${invoiceId}"]`) });
	await expect(ligne).toHaveCount(1, { timeout: 10000 });
	await ligne.getByTestId('tx-checkbox').check();
	await page.getByTestId('reconciliation-accept-btn').click();
	await expect(page.getByTestId('reconciliation-success')).toHaveText(
		/^\s*1\s+opération\(s\) réussie\(s\)\.\s*$/,
	);
	await expect(ligne).toHaveCount(0);
}

test('un Comptable saisit un relevé de trois lignes, le rapproche et le retrouve au journal d’audit', async ({
	page,
}) => {
	const comptable = await createUser(page, 'Comptable');
	await login(page, comptable, USER_PASSWORD);
	const today = todayZurich();
	const contactId = await createContactWithAddressViaApi(page, `Saisie ${Date.now()}`);
	const invoiceId = await createAndValidateInvoiceViaApi(page, contactId, today);
	const bankAccountId = await wiredBankAccountViaApi(page, IBAN_A);

	// 1. Depuis /bank-import, le lien.
	await page.goto('/bank-import');
	await page.getByTestId('bank-import-manual-link').click();
	await expect(page).toHaveURL(/\/bank-import\/manual$/);
	await expect(page.getByTestId('bank-import-manual-line')).toHaveCount(1);

	// 2. Trois lignes, un solde d'arrivée faux : l'écart s'affiche, « Enregistrer » est inactif.
	// 1000.00 + 972.90 − 5.00 − 100.00 = 1867.90 ; saisi 1800.00 → écart −67.90.
	await remplir(page, {
		bankAccountId,
		from: dayBefore(today),
		to: today,
		opening: '1000',
		closing: '1800',
		lignes: troisLignes(today),
	});
	await expect(page.getByTestId('bank-import-manual-gap')).toHaveText('-67.90');
	await expect(page.getByTestId('bank-import-manual-save')).toBeDisabled();
	await expect(page.getByTestId('bank-import-manual-block-reason')).toHaveAttribute('data-reason', 'gap');

	// 3. Le solde corrigé, la vérification, l'enregistrement.
	await page.getByTestId('bank-import-manual-closing-balance').fill('1867,90');
	await expect(page.getByTestId('bank-import-manual-gap')).toHaveText('0.00');
	await expect(page.getByTestId('bank-import-manual-block-reason')).toHaveAttribute('data-reason', 'preview');
	await page.getByTestId('bank-import-manual-verify').click();
	await expect(page.getByTestId('bank-import-manual-preview-ok')).toBeVisible();
	await expect(page.getByTestId('bank-import-manual-save')).toBeEnabled();
	await page.getByTestId('bank-import-manual-save').click();

	// 4. Retour à la liste : le relevé en tête, « Saisie manuelle ».
	await expect(page).toHaveURL(/\/bank-import$/);
	// La notification (svelte-sonner), visée par son attribut et non par son libellé traduit
	// (garde `e2e-selecteurs-traduits.test.ts`, KF-043) ; son nombre de transactions.
	await expect(page.locator('[data-sonner-toast]').first()).toContainText('3');
	const premiere = page.getByTestId('bank-import-list-row').first();
	await expect(premiere).toHaveAttribute('data-source-format', 'MANUAL');
	await expect(premiere.getByTestId('bank-import-list-link')).toHaveText('Saisie manuelle');
	const importId = Number(await premiere.getAttribute('data-import-id'));
	expect(importId).toBeGreaterThan(0);

	// 5. Le rapprochement : la ligne de crédit est proposée contre la facture, et acceptée.
	await rapprocher(page, bankAccountId, invoiceId);

	// 6. Le journal d'audit : l'action nouvelle, sur l'import créé.
	const ctx = await authedApiContext(page);
	let label: string;
	try {
		const vocab = await (await ctx.get('/api/v1/audit-log/vocabulary')).json();
		label = (vocab.actions as { code: string; label: string }[]).find(
			(a) => a.code === 'bank_import.entered_manually',
		)!.label;
	} finally {
		await disposeContextSafe(ctx);
	}
	expect(label).toBe('Relevé bancaire saisi à la main');
	await page.goto('/audit-log');
	await page.getByTestId('audit-log-filter-action').selectOption({ value: 'bank_import.entered_manually' });
	await page.getByTestId('audit-log-filter-entity-type').selectOption({ value: 'bank_imports' });
	await page.getByTestId('audit-log-filter-entity-id').fill(String(importId));
	await page.getByTestId('audit-log-filter-entity-id').press('Enter');
	await page.getByTestId('audit-log-filter-entity-id').blur();
	const row = page.locator('[data-testid="audit-log-row"][data-action="bank_import.entered_manually"]');
	await expect(row).toHaveCount(1);
	await expect(row.getByTestId('audit-log-row-action')).toHaveText(label);
});

test('un utilisateur Consultation ne voit pas le lien, et /bank-import/manual le renvoie', async ({ page }) => {
	const lecteur = await createUser(page, 'Consultation');
	await login(page, lecteur, USER_PASSWORD);
	await page.goto('/bank-import');
	await expect(page.getByTestId('bank-import-page-title')).toBeVisible();
	await expect(page.getByTestId('bank-import-list')).toBeVisible();
	await expect(page.getByTestId('bank-import-manual-link')).toHaveCount(0);
	await page.goto('/bank-import/manual');
	await expect(page).toHaveURL('/');
});

// ---------------------------------------------------------------------------
// Recette visuelle (AC-25) — seulement si KESH_E2E_CAPTURES_DIR est posé
// ---------------------------------------------------------------------------

async function capture(page: Page, testInfo: TestInfo, name: string): Promise<void> {
	const dossier = process.env.KESH_E2E_CAPTURES_DIR;
	const chemin = dossier ? path.join(dossier, `${name}.png`) : testInfo.outputPath(`${name}.png`);
	await page.screenshot({ path: chemin, fullPage: true });
}

test('recette visuelle de la saisie d’un relevé (AC-25)', async ({ page }, testInfo) => {
	test.skip(!process.env.KESH_E2E_CAPTURES_DIR, 'captures seulement (KESH_E2E_CAPTURES_DIR)');
	test.setTimeout(240_000);
	await page.setViewportSize({ width: 1400, height: 1000 });
	const comptable = await createUser(page, 'Comptable');
	await login(page, comptable, USER_PASSWORD);
	const today = todayZurich();
	const contactId = await createContactWithAddressViaApi(page, `Recette ${Date.now()}`);
	const invoiceId = await createAndValidateInvoiceViaApi(page, contactId, today);
	const compteB = await wiredBankAccountViaApi(page, IBAN_B);

	// /bank-import et son lien ; le formulaire vide.
	await page.goto('/bank-import');
	await expect(page.getByTestId('bank-import-manual-link')).toBeVisible();
	await capture(page, testInfo, '01-bank-import-lien');
	await page.getByTestId('bank-import-manual-link').click();
	await expect(page.getByTestId('bank-import-manual-line')).toHaveCount(1);
	await capture(page, testInfo, '02-formulaire-vide');

	// Soldes vides : aucun écart affiché.
	const base = { bankAccountId: compteB, from: dayBefore(today), to: today, lignes: troisLignes(today) };
	await remplir(page, { ...base, opening: '', closing: '' });
	await expect(page.getByTestId('bank-import-manual-gap')).toHaveCount(0);
	await capture(page, testInfo, '03-soldes-vides-sans-ecart');

	// Déséquilibré : écart affiché, Enregistrer inactif, raison « écart ».
	await page.getByTestId('bank-import-manual-opening-balance').fill('1000');
	await page.getByTestId('bank-import-manual-closing-balance').fill('1800');
	await expect(page.getByTestId('bank-import-manual-gap')).toHaveText('-67.90');
	await capture(page, testInfo, '04-desequilibre-ecart');

	// Une ligne invalide : la raison à côté du bouton.
	const l2 = page.getByTestId('bank-import-manual-line').nth(1);
	await l2.getByTestId('bank-import-manual-line-debit').fill('0');
	await expect(page.getByTestId('bank-import-manual-block-reason')).toHaveAttribute('data-reason', 'line');
	await capture(page, testInfo, '05-ligne-invalide-raison');
	await l2.getByTestId('bank-import-manual-line-debit').fill('5.00');

	// Une erreur du serveur, la ligne désignée (libellé de plus de 500 caractères).
	await l2.getByTestId('bank-import-manual-line-details').fill('x'.repeat(501));
	await page.getByTestId('bank-import-manual-verify').click();
	await expect(page.getByTestId('bank-import-manual-error')).toContainText('Ligne 2');
	await capture(page, testInfo, '06-erreur-serveur-ligne-designee');
	await l2.getByTestId('bank-import-manual-line-details').fill('Frais bancaires');

	// Équilibré, vérifié ; puis périmé par une modification.
	await page.getByTestId('bank-import-manual-closing-balance').fill('1867.90');
	await page.getByTestId('bank-import-manual-verify').click();
	await expect(page.getByTestId('bank-import-manual-preview-ok')).toBeVisible();
	await capture(page, testInfo, '07-equilibre-verifie');
	await page.getByTestId('bank-import-manual-statement-number').fill('10/2026');
	await expect(page.getByTestId('bank-import-manual-preview-stale')).toBeVisible();
	await capture(page, testInfo, '08-apercu-perime');

	// Enregistrer : la notification, la liste.
	await page.getByTestId('bank-import-manual-verify').click();
	await expect(page.getByTestId('bank-import-manual-preview-ok')).toBeVisible();
	await page.getByTestId('bank-import-manual-save').click();
	await expect(page.locator('[data-sonner-toast]').first()).toBeVisible();
	await capture(page, testInfo, '09-notification-succes-et-liste');
	const premiere = page.getByTestId('bank-import-list-row').first();
	const importId = Number(await premiere.getAttribute('data-import-id'));

	// Le détail d'un relevé saisi, ses transactions ; la boîte de suppression.
	await page.goto(`/bank-import/${importId}`);
	await expect(page.getByTestId('detail-source-format')).toHaveText('Saisie manuelle');
	await capture(page, testInfo, '10-detail-releve-saisi');
	await page.getByTestId('bank-import-delete-button').click();
	await expect(page.getByTestId('bank-import-delete-dialog')).toBeVisible();
	await capture(page, testInfo, '11-boite-de-suppression');
	await page.keyboard.press('Escape');

	// Le même relevé de nouveau : relevé identique, doublons, chevauchement ; les cases.
	await page.goto('/bank-import/manual');
	await remplir(page, { ...base, opening: '1000', closing: '1867.90' });
	await page.getByTestId('bank-import-manual-statement-number').fill('10/2026');
	await page.getByTestId('bank-import-manual-verify').click();
	await expect(page.getByTestId('bank-import-manual-warning-duplicate-file')).toBeVisible();
	await expect(page.getByTestId('bank-import-manual-warning-duplicate-lines')).toBeVisible();
	await expect(page.getByTestId('bank-import-manual-info-overlap')).toBeVisible();
	await capture(page, testInfo, '12-avertissements-identique-doublons-chevauchement');
	await page.getByTestId('bank-import-manual-confirm-duplicate-file').check();
	await page.getByTestId('bank-import-manual-confirm-duplicate-lines').check();
	await expect(page.getByTestId('bank-import-manual-save')).toBeEnabled();
	await capture(page, testInfo, '13-cases-cochees');

	// Le relevé suivant, avec un solde de départ qui ne suit pas : l'avertissement.
	const lendemain = new Date(`${today}T12:00:00Z`);
	lendemain.setUTCDate(lendemain.getUTCDate() + 1);
	const demain = lendemain.toISOString().slice(0, 10);
	await page.goto('/bank-import/manual');
	await remplir(page, {
		bankAccountId: compteB,
		from: demain,
		to: demain,
		opening: '1900',
		closing: '1890',
		lignes: [{ date: demain, details: 'Frais', debit: '10' }],
	});
	await page.getByTestId('bank-import-manual-verify').click();
	await expect(page.getByTestId('bank-import-manual-warning-opening-gap')).toBeVisible();
	await capture(page, testInfo, '14-avertissement-solde-de-depart');

	// Rapproché : suppression inactive, paragraphe bloqué.
	await rapprocher(page, compteB, invoiceId);
	await page.goto(`/bank-import/${importId}`);
	await expect(page.getByTestId('bank-import-delete-blocked')).toBeVisible();
	await capture(page, testInfo, '15-releve-rapproche-suppression-bloquee');

	// Erreur de suppression : le relevé supprimé entre-temps (404), puis la notification.
	const ctx = await authedApiContext(page);
	try {
		await page.goto('/bank-import/manual');
		await remplir(page, {
			bankAccountId: compteB,
			from: demain,
			to: demain,
			opening: '1867.90',
			closing: '1857.90',
			lignes: [{ date: demain, details: 'Frais', debit: '10' }],
		});
		await page.getByTestId('bank-import-manual-verify').click();
		await page.getByTestId('bank-import-manual-save').click();
		await expect(page).toHaveURL(/\/bank-import$/);
		const aSupprimer = Number(
			await page.getByTestId('bank-import-list-row').first().getAttribute('data-import-id'),
		);
		await page.goto(`/bank-import/${aSupprimer}`);
		await page.getByTestId('bank-import-delete-button').click();
		const del = await ctx.delete(`/api/v1/bank-imports/${aSupprimer}`);
		expect(del.status()).toBe(204);
		await page.getByTestId('bank-import-delete-submit').click();
		await expect(page.getByTestId('bank-import-delete-error')).toContainText("Ce relevé n'existe plus");
		await capture(page, testInfo, '16-erreur-de-suppression');

		await page.goto('/bank-import/manual');
		await remplir(page, {
			bankAccountId: compteB,
			from: demain,
			to: demain,
			opening: '1867.90',
			closing: '1857.90',
			lignes: [{ date: demain, details: 'Frais', debit: '10' }],
		});
		await page.getByTestId('bank-import-manual-verify').click();
		await page.getByTestId('bank-import-manual-save').click();
		await expect(page).toHaveURL(/\/bank-import$/);
		const id2 = Number(await page.getByTestId('bank-import-list-row').first().getAttribute('data-import-id'));
		await page.goto(`/bank-import/${id2}`);
		await page.getByTestId('bank-import-delete-button').click();
		await page.getByTestId('bank-import-delete-submit').click();
		await expect(page.locator('[data-sonner-toast]').first()).toContainText(demain);
		await capture(page, testInfo, '17-notification-de-suppression');

		// Le refus d'un compte clos à la saisie.
		const res = await ctx.post('/api/v1/bank-accounts', {
			data: { bankName: 'Banque close', iban: 'CH5604835012345678009', qrIban: null, isPrimary: false },
		});
		expect(res.ok(), `compte à clore : ${res.status()}`).toBeTruthy();
		const compteC = (await res.json()) as { id: number; version: number };
		await page.goto('/bank-import/manual');
		await remplir(page, {
			bankAccountId: compteC.id,
			from: demain,
			to: demain,
			opening: '0',
			closing: '10',
			lignes: [{ date: demain, details: 'Dépôt', credit: '10' }],
		});
		const close = await ctx.post(`/api/v1/bank-accounts/${compteC.id}/close`, {
			data: { version: compteC.version },
		});
		expect(close.ok(), `clôture : ${close.status()}`).toBeTruthy();
		await page.getByTestId('bank-import-manual-verify').click();
		await expect(page.getByTestId('bank-import-manual-error')).toBeVisible();
		await capture(page, testInfo, '18-refus-compte-clos');
	} finally {
		await disposeContextSafe(ctx);
	}

	// Le filtre d'action du journal d'audit, avec l'option nouvelle.
	await page.goto('/audit-log');
	await page.getByTestId('audit-log-filter-action').selectOption({ value: 'bank_import.entered_manually' });
	await expect(page.locator('[data-testid="audit-log-row"]').first()).toBeVisible();
	await capture(page, testInfo, '19-journal-audit-filtre-action');

	// /bank-import vu par un Consultation : pas de lien.
	await clearAuthStorage(page);
	const lecteur = await createUser(page, 'Consultation');
	await login(page, lecteur, USER_PASSWORD);
	await page.goto('/bank-import');
	await expect(page.getByTestId('bank-import-list')).toBeVisible();
	await capture(page, testInfo, '20-consultation-sans-lien');
});
