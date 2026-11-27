/**
 * Story 55-3b (#1607) — les justificatifs d'une écriture, ouverts depuis le Journal (et le Grand livre).
 *
 * Ce que seul un E2E constate : que `hasDocuments` traverse la frontière HTTP jusqu'à l'icône, que la liste
 * (`GET /journal-entries/{id}/documents`) arrive au dialogue, que l'écran **ne lit pas** `/invoices/{id}/pdf`
 * d'une facture non figée (il la figerait : D12), qu'il lit celui d'une facture figée, et que le dialogue se
 * ferme par l'historique du navigateur.
 *
 * ⚠️ Montants uniques à chaque exécution (patron de `reconciliation-group.spec.ts`) ; règlement groupé
 * annulé et import supprimé en `finally`. Les aides du règlement groupé vivent ici : `api-fixtures.ts` et le
 * spec de l'Epic 43 ne sont pas touchés.
 *
 * ⚠️ Sélecteurs par `data-testid` (garde KF-043, #326).
 *
 * Captures (thème clair) dans `KESH_CAPTURES_DIR` quand il est posé.
 */

import { expect, test, type APIRequestContext, type Page } from '@playwright/test';
import AxeBuilder from '@axe-core/playwright';
import {
	seedTestState,
	clearAuthStorage,
	authedApiContext,
	disposeContextSafe,
} from './helpers/test-state';
import {
	createContactWithAddressViaApi,
	createJournalEntryViaApi,
	createManualCreditStatementViaApi,
	createUser,
	todayInZurich,
	USER_PASSWORD,
	wiredBankAccountViaApi,
} from './helpers/api-fixtures';

// Un IBAN ordinaire (non QR-IBAN) : le PDF de la facture l'exige comme compte principal.
const TEST_IBAN = 'CH9300762011623852957';

test.beforeAll(async () => {
	await seedTestState('with-company');
});

test.afterEach(async ({ page }) => {
	await clearAuthStorage(page);
});

// Le PDF d'une facture exige un compte bancaire principal (QR-facture) : créé et câblé une fois par test.
test.beforeEach(async ({ page }) => {
	await login(page);
	await wiredBankAccountViaApi(page, TEST_IBAN);
});

async function login(page: Page, username = 'admin', password = 'admin123'): Promise<void> {
	await page.goto('/login');
	await page.fill('#username', username);
	await page.fill('#password', password);
	await page.click('button[type="submit"]');
	await expect(page).toHaveURL('/');
}

async function capture(page: Page, nom: string): Promise<void> {
	if (!process.env.KESH_CAPTURES_DIR) return;
	// Recette seulement : le lecteur PDF du navigateur (avec tête) rend la page un peu après le cadre.
	await page.waitForTimeout(2000);
	await page.screenshot({ path: `${process.env.KESH_CAPTURES_DIR}/55-3b-${nom}.png`, fullPage: false });
}

type Facture = { id: number; number: string; total: string; journalEntryId: number };

/** Une facture client validée, sans TVA, à un prix unique ; rend son écriture de vente. */
async function factureValidee(ctx: APIRequestContext, contactId: number, unitPrice: string): Promise<Facture> {
	const date = todayInZurich();
	const created = await ctx.post('/api/v1/invoices', {
		data: {
			contactId,
			date,
			dueDate: date,
			paymentTerms: '30 jours net',
			lines: [{ description: 'Prestation 55-3b', quantity: '1', unitPrice, vatRate: '0.00' }],
		},
	});
	expect(created.ok(), `facture : ${created.status()} ${await created.text()}`).toBeTruthy();
	const { id } = (await created.json()) as { id: number };
	const validated = await ctx.post(`/api/v1/invoices/${id}/validate`);
	expect(validated.ok(), `validation : ${validated.status()} ${await validated.text()}`).toBeTruthy();
	const fiche = (await (await ctx.get(`/api/v1/invoices/${id}`)).json()) as {
		invoiceNumber: string;
		amountDue: string;
		journalEntryId: number;
	};
	return { id, number: fiche.invoiceNumber, total: fiche.amountDue, journalEntryId: fiche.journalEntryId };
}

/** Le geste de la fiche : le premier téléchargement fige le PDF. */
async function figer(ctx: APIRequestContext, invoiceId: number): Promise<void> {
	const res = await ctx.get(`/api/v1/invoices/${invoiceId}/pdf`);
	expect(res.status(), `PDF de la facture ${invoiceId} : ${res.status() === 200 ? '' : await res.text()}`).toBe(200);
}

/** L'adresse du Journal sur l'exercice par défaut (le plus récent). */
async function journalUrl(page: Page): Promise<string> {
	const ctx = await authedApiContext(page);
	try {
		const fys: Array<{ id: number; startDate: string; endDate: string }> = await (
			await ctx.get('/api/v1/fiscal-years')
		).json();
		const fy = fys.sort((x, y) => y.startDate.localeCompare(x.startDate))[0];
		return `/reports?tab=journals&fy=${fy.id}&from=${fy.startDate}&to=${fy.endDate}`;
	} finally {
		await disposeContextSafe(ctx);
	}
}

/**
 * Prix unique en francs, multiple de 5 centimes (sans quoi la validation exige un compte d'arrondi, que le
 * seed ne désigne pas) ; `decalage` en centimes, multiple de 5.
 */
function prixUnique(decalage = 0): string {
	return ((10000 + 5 * (Date.now() % 4000) + decalage) / 100).toFixed(2);
}

/** Écoute les lectures du PDF d'une facture (`/invoices/{id}/pdf`). */
function lecturesPdf(page: Page): number[] {
	const ids: number[] = [];
	page.on('request', (r) => {
		const m = /\/api\/v1\/invoices\/(\d+)\/pdf$/.exec(new URL(r.url()).pathname);
		if (m) ids.push(Number(m[1]));
	});
	return ids;
}

const dialog = (page: Page) => page.locator('[role="dialog"]');

test('AC-1, AC-5, AC-7, AC-10, AC-11 — Journal : facture non émise signalée sans lecture, puis figée et affichée ; fermetures ; axe', async ({
	page,
}) => {
	await login(page);
	const ctx = await authedApiContext(page);
	let f: Facture;
	try {
		const contact = await createContactWithAddressViaApi(page, `Client 55-3b ${Date.now()}`);
		f = await factureValidee(ctx, contact, prixUnique());
	} finally {
		await disposeContextSafe(ctx);
	}
	// Une écriture sans justificatif, pour la hauteur de ligne (AC-1).
	const sans = await createJournalEntryViaApi(page, 'E2E 55-3b — sans justificatif', '12.30');
	const lectures = lecturesPdf(page);
	await page.goto(await journalUrl(page));
	const icone = page.getByTestId(`journal-entry-documents-${f.journalEntryId}`);
	await expect(icone).toBeVisible({ timeout: 10000 });
	await expect(page.getByTestId(`journal-entry-documents-${sans}`)).toHaveCount(0);

	// AC-1 — hauteur de ligne inchangée (un pixel près), couleur de l'icône = celle du numéro.
	const hAvec = (await page.getByTestId(`journal-entry-header-${f.journalEntryId}`).boundingBox())!.height;
	const hSans = (await page.getByTestId(`journal-entry-header-${sans}`).boundingBox())!.height;
	expect(Math.abs(hAvec - hSans)).toBeLessThanOrEqual(1);
	const couleurIcone = await icone.locator('svg').evaluate((el) => getComputedStyle(el).color);
	const couleurNumero = await page
		.getByTestId(`journal-entry-number-${f.journalEntryId}`)
		.evaluate((el) => getComputedStyle(el).color);
	expect(couleurIcone).toBe(couleurNumero);
	await capture(page, '1-journal-icone');

	// AC-5 — facture non figée : « PDF pas encore émis », aucune lecture de PDF, aucune liste ; l'URL ne bouge pas.
	const avant = page.url();
	await icone.click();
	await expect(dialog(page)).toBeVisible();
	await expect(page.getByTestId('entry-documents-not-issued')).toBeVisible();
	await expect(page.getByTestId('entry-documents-not-issued')).toContainText('PDF pas encore émis');
	await expect(page.getByTestId('document-preview')).toHaveCount(0);
	await expect(page.getByTestId('entry-documents-list')).toHaveCount(0);
	expect(page.url()).toBe(avant);
	await capture(page, '2-pdf-pas-encore-emis');
	const axe1 = await new AxeBuilder({ page }).withTags(['wcag2a', 'wcag2aa']).include('[role="dialog"]').analyze();
	expect(axe1.violations).toEqual([]);
	// Échap ferme, le focus revient à l'icône.
	await page.keyboard.press('Escape');
	await expect(dialog(page)).toHaveCount(0);
	await expect(icone).toBeFocused();
	expect(lectures).toEqual([]);
	const ctx2 = await authedApiContext(page);
	try {
		const fiche = (await (await ctx2.get(`/api/v1/invoices/${f.id}`)).json()) as { pdfFrozenAt: string | null };
		expect(fiche.pdfFrozenAt).toBeNull();
		// Le geste d'émission (fiche ou API) fige la facture.
		await figer(ctx2, f.id);
	} finally {
		await disposeContextSafe(ctx2);
	}

	// Seconde ouverture : le PDF figé dans la visionneuse, sans message.
	const nAvant = lectures.length;
	await icone.click();
	const frame = page.getByTestId('document-preview-frame');
	await expect(frame).toBeVisible({ timeout: 10000 });
	await expect(page.getByTestId('entry-documents-not-issued')).toHaveCount(0);
	expect(lectures.slice(nAvant)).toEqual([f.id]);
	// AC-11 — le PDF s'ouvre sans le panneau des vignettes (fragment d'ouverture).
	expect(await frame.getAttribute('src')).toMatch(/^blob:[^#]+#pagemode=none&navpanes=0$/);
	// Hauteur : le dialogue tient dans 85 % de la fenêtre, sans défilement interne.
	const vh = page.viewportSize()!.height;
	const boite = (await dialog(page).boundingBox())!;
	expect(boite.height).toBeLessThanOrEqual(vh * 0.85 + 1);
	const defile = await dialog(page).evaluate((el) => el.scrollHeight > el.clientHeight + 1);
	expect(defile).toBe(false);
	await capture(page, '3-visionneuse-un-justificatif');
	const axe2 = await new AxeBuilder({ page }).withTags(['wcag2a', 'wcag2aa']).include('[role="dialog"]').analyze();
	expect(axe2.violations).toEqual([]);
	// Un clic à côté (sur le fond) ferme.
	await page.mouse.click(5, 5);
	await expect(dialog(page)).toHaveCount(0);
	await expect(icone).toBeFocused();

	// Erreur 410 (fichier figé disparu) : le message du serveur dans le dialogue, le journal reste affiché.
	await page.route(`**/api/v1/invoices/${f.id}/pdf`, (route) =>
		route.fulfill({
			status: 410,
			contentType: 'application/json',
			body: JSON.stringify({
				error: { code: 'INVOICE_PDF_GONE', message: 'Le PDF émis de cette facture est introuvable (fichier abc.pdf).' },
			}),
		}),
	);
	try {
		await icone.click();
		await expect(page.getByTestId('document-preview-error')).toContainText('introuvable (fichier abc.pdf)');
		await capture(page, '6-erreur-410');
		await page.getByTestId('entry-documents-close').click();
		await expect(dialog(page)).toHaveCount(0);
		await expect(icone).toBeVisible();
	} finally {
		await page.unroute(`**/api/v1/invoices/${f.id}/pdf`);
	}
});

test('AC-2 — règlement groupé : une icône, la liste des deux factures, un clic change le document', async ({ page }) => {
	await login(page);
	const bankAccountId = await wiredBankAccountViaApi(page, TEST_IBAN);
	const ctx = await authedApiContext(page);
	let importId = 0;
	let txId = 0;
	try {
		const unique = `${Date.now()}`;
		const a = await createContactWithAddressViaApi(page, `Groupe A 55-3b ${unique}`);
		const b = await createContactWithAddressViaApi(page, `Groupe B 55-3b ${unique}`);
		const fa = await factureValidee(ctx, a, prixUnique());
		const fb = await factureValidee(ctx, b, prixUnique(3700));
		const montant = ((Math.round(Number(fa.total) * 100) + Math.round(Number(fb.total) * 100)) / 100).toFixed(2);
		({ importId, txId } = await createManualCreditStatementViaApi(ctx, {
			bankAccountId,
			bookingDate: todayInZurich(),
			amount: montant,
			counterpartyName: `Groupe A 55-3b ${unique}`,
			details: `Règlement groupé 55-3b ${unique}`,
		}));
		// Montant exact : aucun écart, aucun compte désigné requis.
		const r = await ctx.post('/api/v1/reconciliation/accept', {
			data: {
				bankAccountId,
				proposals: [{ type: 'invoice_group', bankTransactionId: txId, invoiceIds: [fa.id, fb.id] }],
			},
		});
		const corps = (await r.json()) as { accepted: Array<{ journalEntryId: number }> };
		expect(corps.accepted, JSON.stringify(corps)).toHaveLength(1);
		const reglement = corps.accepted[0].journalEntryId;
		await figer(ctx, fa.id);
		await figer(ctx, fb.id);
		const lectures = lecturesPdf(page);
		await page.goto(await journalUrl(page));
		const icone = page.getByTestId(`journal-entry-documents-${reglement}`);
		await expect(icone).toBeVisible({ timeout: 10000 });
		await expect(page.getByTestId(`journal-entry-header-${reglement}`).locator('button')).toHaveCount(1);
		await icone.click();
		const liste = page.getByTestId('entry-documents-list');
		await expect(liste).toBeVisible();
		await expect(liste.locator('li')).toHaveCount(2);
		const premier = Math.min(fa.id, fb.id);
		const second = Math.max(fa.id, fb.id);
		await expect(page.getByTestId(`entry-documents-item-invoice-${premier}`)).toHaveAttribute('aria-current', 'true');
		await expect(page.getByTestId('document-preview-frame')).toBeVisible({ timeout: 10000 });
		await capture(page, '4-visionneuse-deux-justificatifs');
		await page.getByTestId(`entry-documents-item-invoice-${second}`).click();
		await expect(page.getByTestId(`entry-documents-item-invoice-${second}`)).toHaveAttribute('aria-current', 'true');
		await expect
			.poll(() => lectures.filter((id) => id === second).length, { timeout: 10000 })
			.toBe(1);
		await expect(page.getByTestId('document-preview-frame')).toBeVisible({ timeout: 10000 });
		expect(lectures.filter((id) => id === premier).length).toBe(1);
		await capture(page, '5-visionneuse-second-justificatif');
	} finally {
		if (txId) await ctx.post(`/api/v1/reconciliation/transactions/${txId}/cancel`, { data: {} });
		if (importId) await ctx.delete(`/api/v1/bank-imports/${importId}`);
		await disposeContextSafe(ctx);
	}
});

test('AC-7 — Retour dialogue ouvert : l’onglet précédent, le dialogue fermé, aucun voile resté', async ({ page }) => {
	await login(page);
	const ctx = await authedApiContext(page);
	let f: Facture;
	try {
		const contact = await createContactWithAddressViaApi(page, `Client retour 55-3b ${Date.now()}`);
		f = await factureValidee(ctx, contact, prixUnique(15));
	} finally {
		await disposeContextSafe(ctx);
	}
	await page.goto('/reports');
	await expect(page.locator('#reports-tab-balance-sheet')).toHaveAttribute('aria-selected', 'true', { timeout: 10000 });
	await page.locator('#reports-tab-journals').click();
	await expect(page).toHaveURL(/tab=journals/);
	const icone = page.getByTestId(`journal-entry-documents-${f.journalEntryId}`);
	await expect(icone).toBeVisible({ timeout: 10000 });
	await icone.click();
	await expect(dialog(page)).toBeVisible();
	await page.goBack();
	await expect(page).toHaveURL(/tab=balance-sheet/);
	await expect(dialog(page)).toHaveCount(0);
	// Aucun voile ni verrou de défilement : un clic sur un onglet réussit.
	await page.locator('#reports-tab-journals').click();
	await expect(page.locator('#reports-tab-journals')).toHaveAttribute('aria-selected', 'true');
});

test('AC-6 — Grand livre : la colonne « Justificatifs », l’icône sur la ligne, le même dialogue', async ({ page }) => {
	await login(page);
	const ctx = await authedApiContext(page);
	let url: string;
	let f: Facture;
	try {
		const contact = await createContactWithAddressViaApi(page, `Client GL 55-3b ${Date.now()}`);
		f = await factureValidee(ctx, contact, prixUnique(25));
		await figer(ctx, f.id);
		const accounts = (await (await ctx.get('/api/v1/accounts')).json()) as Array<{ id: number; number: string }>;
		const debiteurs = accounts.find((a) => a.number === '1100')!;
		const fys: Array<{ startDate: string; endDate: string }> = await (await ctx.get('/api/v1/fiscal-years')).json();
		const fy = fys.sort((x, y) => y.startDate.localeCompare(x.startDate))[0];
		url = `/reports?tab=general-ledger&accountId=${debiteurs.id}&from=${fy.startDate}&to=${fy.endDate}`;
	} finally {
		await disposeContextSafe(ctx);
	}
	await page.goto(url);
	const section = page.getByTestId('ledger-section-1100');
	await expect(section).toBeVisible({ timeout: 10000 });
	await expect(section.locator('thead th', { hasText: 'Justificatifs' })).toHaveCount(1);
	const icone = section.locator('[data-testid^="ledger-entry-documents-"]').last();
	await expect(icone).toBeVisible();
	await capture(page, '7-grand-livre-colonne');
	await icone.click();
	await expect(page.getByTestId('document-preview-frame')).toBeVisible({ timeout: 10000 });
	await capture(page, '8-grand-livre-visionneuse');
	await page.getByTestId('entry-documents-close').click();
	await expect(dialog(page)).toHaveCount(0);
});

test('AC-4 — un utilisateur Consultation ouvre le PDF figé d’une facture depuis le Journal', async ({ page }) => {
	await login(page);
	const ctx = await authedApiContext(page);
	let f: Facture;
	try {
		const contact = await createContactWithAddressViaApi(page, `Client consult 55-3b ${Date.now()}`);
		f = await factureValidee(ctx, contact, prixUnique(35));
		await figer(ctx, f.id);
	} finally {
		await disposeContextSafe(ctx);
	}
	await clearAuthStorage(page);
	const username = await createUser(page, 'Consultation');
	await login(page, username, USER_PASSWORD);
	await page.goto(await journalUrl(page));
	const icone = page.getByTestId(`journal-entry-documents-${f.journalEntryId}`);
	await expect(icone).toBeVisible({ timeout: 10000 });
	await icone.click();
	await expect(page.getByTestId('document-preview-frame')).toBeVisible({ timeout: 10000 });
});
