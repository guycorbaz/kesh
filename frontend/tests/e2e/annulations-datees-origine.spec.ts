/**
 * Story 63-1 (#1752, #1753, #1754, AC-15) — les annulations depuis la fiche
 * contre-passent l'écriture à SA date, dans SON exercice.
 *
 * Décor : une société sans exercice (`with-company-no-fy`) où l'on crée 2025 et
 * 2026, tous deux ouverts ; le jour d'exécution est dans 2026, les pièces sont de
 * décembre 2025 — une annulation datée du jour tombait dans 2026.
 *
 * (1) Règlement client : la confirmation dit « datée du règlement » ; après le
 *     clic, l'inverse — rejointe par les liens de l'écriture d'origine — est
 *     datée du règlement, dans 2025.
 * (2) Facture fournisseur validée sous la borne du verrou : le motif « période
 *     verrouillée » remplace le bouton *Annuler la facture*.
 *
 * Recette visuelle (thème clair, animations coupées) : captures dans
 * `KESH_E2E_CAPTURES_DIR` (hors dépôt) — les deux confirmations de la fiche
 * client (dialogue), les deux confirmations de la fiche fournisseur (boîte
 * native du navigateur : son texte est écrit à côté, la boîte n'apparaissant
 * pas sur une capture), et les motifs neufs.
 *
 * Pré-requis : MariaDB up + KESH_TEST_MODE=true (cf. docs/testing.md).
 */

import { expect, test, type APIRequestContext, type Page, type TestInfo } from '@playwright/test';
import fs from 'fs';
import path from 'path';
import {
	seedTestState,
	clearAuthStorage,
	authedApiContext,
	disposeContextSafe,
} from './helpers/test-state';
import { invoiceVersionViaApi } from './helpers/api-fixtures';

const INVOICE_DATE = '2025-12-15';
const SETTLED_ON = '2025-12-20';
const LOCK_THROUGH = '2025-12-31';

test.beforeAll(async () => {
	await seedTestState('with-company-no-fy');
});

test.afterAll(async () => {
	// Les specs suivantes attendent la société du préréglage commun.
	await seedTestState('with-company');
});

test.afterEach(async ({ page }) => {
	await clearAuthStorage(page);
});

async function login(page: Page): Promise<void> {
	await page.goto('/login');
	await page.fill('#username', 'admin');
	await page.fill('#password', 'admin123');
	await page.click('button[type="submit"]');
	await expect(page).toHaveURL('/');
}

function capturePath(testInfo: TestInfo, name: string, ext = 'png'): string {
	const dossier = process.env.KESH_E2E_CAPTURES_DIR;
	return dossier ? path.join(dossier, `${name}.${ext}`) : testInfo.outputPath(`${name}.${ext}`);
}

async function capture(page: Page, testInfo: TestInfo, name: string): Promise<void> {
	await page.screenshot({
		path: capturePath(testInfo, `63-1-${name}`),
		fullPage: true,
		animations: 'disabled',
	});
}

type Account = {
	id: number;
	number: string;
	accountType: string;
	active: boolean;
	postable: boolean;
};

let fy2025 = 0;
let decorPret = false;

/** Les exercices 2025 et 2026 (ouverts), une fois pour le fichier. */
async function exercices(ctx: APIRequestContext): Promise<void> {
	if (decorPret) return;
	for (const year of [2025, 2026]) {
		const res = await ctx.post('/api/v1/fiscal-years', {
			data: { name: `Exercice ${year}`, startDate: `${year}-01-01`, endDate: `${year}-12-31` },
		});
		expect(res.status(), await res.text()).toBe(201);
		if (year === 2025) fy2025 = ((await res.json()) as { id: number }).id;
	}
	decorPret = true;
}

async function comptes(ctx: APIRequestContext): Promise<{ cash: Account; expense: Account }> {
	const accounts = (await (await ctx.get('/api/v1/accounts')).json()) as Account[];
	const cash = accounts.find(
		(a) => a.active && a.postable && a.accountType === 'Asset' && a.number.startsWith('10'),
	);
	const expense = accounts.find((a) => a.active && a.postable && a.accountType === 'Expense');
	expect(cash, 'un compte de liquidités imputable').toBeTruthy();
	expect(expense, 'un compte de charge imputable').toBeTruthy();
	return { cash: cash!, expense: expense! };
}

async function contact(ctx: APIRequestContext, supplier: boolean): Promise<number> {
	const res = await ctx.post('/api/v1/contacts', {
		data: {
			contactType: 'Entreprise',
			name: `63-1 ${supplier ? 'Fournisseur' : 'Client'} ${Date.now()}-${Math.floor(Math.random() * 1e6)}`,
			isClient: !supplier,
			isSupplier: supplier,
		},
	});
	expect(res.ok(), `contact : ${res.status()}`).toBeTruthy();
	return ((await res.json()) as { id: number }).id;
}

/** Une facture client du 15.12.2025, validée, réglée de `montant` le 20.12.2025. */
async function factureReglee(ctx: APIRequestContext, montant: string): Promise<number> {
	await exercices(ctx);
	const { cash } = await comptes(ctx);
	const created = await ctx.post('/api/v1/invoices', {
		data: {
			contactId: await contact(ctx, false),
			date: INVOICE_DATE,
			dueDate: INVOICE_DATE,
			paymentTerms: null,
			lines: [{ description: 'Prestation', quantity: '1', unitPrice: '100.00', vatRate: '8.10' }],
		},
	});
	expect(created.ok(), `facture : ${created.status()} ${await created.text()}`).toBeTruthy();
	const id = ((await created.json()) as { id: number }).id;
	const validated = await ctx.post(`/api/v1/invoices/${id}/validate`);
	expect(validated.ok(), `validation : ${validated.status()} ${await validated.text()}`).toBeTruthy();
	const settled = await ctx.post(`/api/v1/invoices/${id}/settlements`, {
		data: {
			settlementType: 'internal_account',
			accountId: cash.id,
			amount: montant,
			settledOn: SETTLED_ON,
			version: await invoiceVersionViaApi(ctx, id),
		},
	});
	expect(settled.ok(), `règlement : ${settled.status()} ${await settled.text()}`).toBeTruthy();
	return id;
}

/** Une facture fournisseur du 15.12.2025, validée ; payée le 20.12.2025 si `payer`. */
async function factureFournisseur(ctx: APIRequestContext, payer: boolean): Promise<number> {
	await exercices(ctx);
	const { cash, expense } = await comptes(ctx);
	const accounts = (await (await ctx.get('/api/v1/accounts')).json()) as Account[];
	const settings = await (await ctx.get('/api/v1/company/invoice-settings')).json();
	if (settings.defaultPayableAccountId == null) {
		const payable = accounts.find((a) => a.number === '2000');
		const put = await ctx.put('/api/v1/company/invoice-settings', {
			data: { ...settings, defaultPayableAccountId: payable!.id },
		});
		expect(put.ok(), `compte créanciers : ${put.status()}`).toBeTruthy();
	}
	const created = await ctx.post('/api/v1/supplier-invoices', {
		data: {
			contactId: await contact(ctx, true),
			supplierInvoiceNumber: `FF-631-${Date.now()}`,
			invoiceDate: INVOICE_DATE,
			dueDate: null,
			lines: [
				{
					description: 'Fourniture',
					quantity: '1',
					unitPrice: '100.00',
					vatRate: '0',
					expenseAccountId: expense.id,
				},
			],
		},
	});
	expect(created.status(), await created.text()).toBe(201);
	let inv = (await created.json()) as { id: number; version: number };
	const validated = await ctx.post(`/api/v1/supplier-invoices/${inv.id}/validate`, {
		data: { version: inv.version },
	});
	expect(validated.status(), await validated.text()).toBe(200);
	inv = await validated.json();
	if (payer) {
		const paid = await ctx.post(`/api/v1/supplier-invoices/${inv.id}/pay`, {
			data: { settlementType: 'internal_account', accountId: cash.id, paymentDate: SETTLED_ON },
		});
		expect(paid.status(), await paid.text()).toBe(200);
	}
	return inv.id;
}

/** Accepte la boîte native, et écrit son texte à côté des captures (recette visuelle). */
function accepterEtNoter(page: Page, testInfo: TestInfo, name: string): Promise<string> {
	return new Promise((resolve) => {
		page.once('dialog', async (dialog) => {
			const message = dialog.message();
			fs.writeFileSync(capturePath(testInfo, `63-1-${name}`, 'txt'), message + '\n');
			await dialog.accept();
			resolve(message);
		});
	});
}

test.describe.serial('63-1 — annuler depuis la fiche, daté de l’origine', () => {
	test('(1) règlement client : la confirmation et l’inverse disent la date du règlement, dans 2025', async ({
		page,
	}, testInfo) => {
		await login(page);
		const ctx = await authedApiContext(page);
		let id: number;
		let settlementEntry: number;
		try {
			id = await factureReglee(ctx, '40.00');
			const list = (await (await ctx.get(`/api/v1/invoices/${id}/settlements`)).json()) as Array<{
				journalEntryId: number;
				cancellable: boolean;
			}>;
			settlementEntry = list[0].journalEntryId;
			expect(list[0].cancellable).toBe(true);
		} finally {
			await disposeContextSafe(ctx);
		}

		await page.goto(`/invoices/${id}`);
		await page.getByTestId('invoice-settlement-cancel').click();
		const dialog = page.getByRole('dialog');
		await expect(dialog).toContainText(
			"Annuler ce règlement ? Une écriture inverse, datée du règlement, sera passée au grand livre dans son exercice",
		);
		// Revue P5, M1 — un règlement ne porte pas de TVA : le texte de `invoices-settlement-cancel-confirm`
		// (fr-CH), qui dit le verrou à faire reculer, non la période à faire verrouiller.
		await expect(dialog).toContainText(
			"Les soldes de la période du règlement changent, la TVA non : un règlement n'en porte pas. Si cette période est verrouillée, l'annulation sera refusée : un administrateur doit d'abord faire reculer le verrou.",
		);
		await expect(dialog).not.toContainText("faites d'abord verrouiller");
		await expect(dialog).not.toContainText("aujourd'hui");
		await capture(page, testInfo, 'confirmation-reglement-client');
		await page.getByTestId('invoice-settlement-cancel-confirm').click();
		await expect(page.getByTestId('invoice-settlements')).toHaveCount(0, { timeout: 5000 });

		// L'inverse, rejointe depuis l'écriture d'origine : datée du 20.12.2025.
		await page.goto(`/journal-entries/${settlementEntry}`);
		const renvoi = page.getByTestId('reversed-by-link');
		await expect(renvoi).toBeVisible();
		await renvoi.getByRole('link').click();
		await expect(page.getByTestId('reverses-link')).toBeVisible();
		await expect(page.locator('body')).toContainText('20.12.2025');
		await capture(page, testInfo, 'inverse-datee-du-reglement');
		const url = page.url();
		const reversalId = Number(url.split('/').pop());
		const api = await authedApiContext(page);
		try {
			const inverse = (await (await api.get(`/api/v1/journal-entries/${reversalId}`)).json()) as {
				entryDate: string;
				fiscalYearId: number;
			};
			expect(inverse.entryDate).toBe(SETTLED_ON);
			expect(inverse.fiscalYearId).toBe(fy2025);
		} finally {
			await disposeContextSafe(api);
		}
	});

	test('(recette) confirmation du solde, fiche client', async ({ page }, testInfo) => {
		await login(page);
		const ctx = await authedApiContext(page);
		let id: number;
		try {
			await exercices(ctx);
			const { expense } = await comptes(ctx);
			const settings = await (await ctx.get('/api/v1/company/invoice-settings')).json();
			const put = await ctx.put('/api/v1/company/invoice-settings', {
				data: { ...settings, defaultBankFeesAccountId: expense.id },
			});
			expect(put.ok(), `compte de frais : ${put.status()} ${await put.text()}`).toBeTruthy();
			id = await factureReglee(ctx, '100.00');
			const wo = await ctx.post(`/api/v1/invoices/${id}/write-off`, {
				data: {
					nature: 'bank_fees',
					settledOn: SETTLED_ON,
					version: await invoiceVersionViaApi(ctx, id),
				},
			});
			expect(wo.ok(), `solde : ${wo.status()} ${await wo.text()}`).toBeTruthy();
		} finally {
			await disposeContextSafe(ctx);
		}
		await page.goto(`/invoices/${id}`);
		// Le solde est le dernier règlement : son bouton « Annuler le solde ».
		const boutons = page.getByTestId('invoice-settlement-cancel');
		await boutons.last().click();
		const dialog = page.getByRole('dialog');
		await expect(dialog).toContainText(
			// Revue P4, M1 — un solde en frais bancaires ne corrige pas la TVA : le texte sans TVA.
			'Annuler ce solde ? Une écriture inverse, datée du solde, sera passée au grand livre dans son exercice, et le montant redeviendra dû. Les soldes de la période du solde changent, la TVA non',
		);
		await expect(dialog).not.toContainText('TVA comprise');
		await capture(page, testInfo, 'confirmation-solde-client');
	});

	test('(recette) les deux confirmations de la fiche fournisseur', async ({ page }, testInfo) => {
		await login(page);
		const ctx = await authedApiContext(page);
		let payee: number;
		let validee: number;
		try {
			payee = await factureFournisseur(ctx, true);
			validee = await factureFournisseur(ctx, false);
		} finally {
			await disposeContextSafe(ctx);
		}
		await page.goto(`/supplier-invoices/${payee}`);
		await expect(page.getByTestId('supplier-invoice-settlement-cancel')).toBeVisible();
		await capture(page, testInfo, 'fiche-fournisseur-payee');
		const reglement = accepterEtNoter(page, testInfo, 'confirmation-reglement-fournisseur');
		await page.getByTestId('supplier-invoice-settlement-cancel').click();
		expect(await reglement).toContain(
			'Une écriture inverse, datée du paiement, sera passée au grand livre dans son exercice',
		);

		await page.goto(`/supplier-invoices/${validee}`);
		const facture = accepterEtNoter(page, testInfo, 'confirmation-facture-fournisseur');
		await page.getByTestId('supplier-invoice-cancel').click();
		expect(await facture).toContain(
			"Une écriture inverse de l'achat, datée comme lui, sera passée au grand livre dans son exercice",
		);
		await expect(page.getByTestId('supplier-invoice-cancelled-info')).toBeVisible({ timeout: 5000 });
	});

	test('(2) facture fournisseur sous la borne du verrou : le motif remplace le bouton', async ({
		page,
	}, testInfo) => {
		await login(page);
		const ctx = await authedApiContext(page);
		let id: number;
		let client: number;
		try {
			id = await factureFournisseur(ctx, false);
			client = await factureReglee(ctx, '40.00');
			const lock = await ctx.post('/api/v1/companies/current/books-lock', {
				data: { through: LOCK_THROUGH },
			});
			expect(lock.ok(), `verrou : ${lock.status()} ${await lock.text()}`).toBeTruthy();

			await page.goto(`/supplier-invoices/${id}`);
			await expect(page.getByTestId('supplier-invoice-cancel')).toHaveCount(0);
			const motif = page.getByTestId('supplier-invoice-cancel-blocked');
			await expect(motif).toContainText(
				"L'écriture d'achat de cette facture est dans une période verrouillée",
			);
			await expect(motif).toContainText("La période est verrouillée jusqu'au 31.12.2025 inclus.");
			await capture(page, testInfo, 'motif-facture-fournisseur-periode-verrouillee');

			// Le même motif, queue partagée, sur la fiche client (règlement partiel).
			await page.goto(`/invoices/${client}`);
			await expect(page.getByTestId('invoice-settlement-cancel')).toHaveCount(0);
			const motifClient = page.getByTestId('invoice-settlement-cancel-blocked');
			await expect(motifClient).toContainText('Cette écriture est dans une période verrouillée');
			await expect(motifClient).toContainText(
				"La période est verrouillée jusqu'au 31.12.2025 inclus.",
			);
			await capture(page, testInfo, 'motif-reglement-client-periode-verrouillee');
		} finally {
			// Un verrou laissé en place verrouillerait les specs suivantes.
			await ctx.post('/api/v1/companies/current/books-lock/release', {
				data: { through: null, motif: 'Fin du test E2E 63-1' },
			});
			await disposeContextSafe(ctx);
		}
	});

	// Revue P1 de la 63-1 (L3, L4) — le verrou posé ENTRE la lecture de la fiche et le clic : le
	// serveur refuse en `400 PERIOD_LOCKED`, et la fiche le dit comme son motif, en date suisse.
	test('(3) verrou posé entre la lecture et le clic : le refus dit le motif de la fiche, en date suisse', async ({
		page,
	}, testInfo) => {
		await login(page);
		const ctx = await authedApiContext(page);
		const poserLeVerrou = async () => {
			const lock = await ctx.post('/api/v1/companies/current/books-lock', {
				data: { through: LOCK_THROUGH },
			});
			expect(lock.ok(), `verrou : ${lock.status()} ${await lock.text()}`).toBeTruthy();
		};
		const leverLeVerrou = async () => {
			await ctx.post('/api/v1/companies/current/books-lock/release', {
				data: { through: null, motif: 'Fin du test E2E 63-1' },
			});
		};
		try {
			const client = await factureReglee(ctx, '40.00');
			const fournisseur = await factureFournisseur(ctx, false);

			// Fiche client : le dialogue est ouvert AVANT le verrou, le clic de confirmation APRÈS.
			await page.goto(`/invoices/${client}`);
			await page.getByTestId('invoice-settlement-cancel').click();
			await expect(page.getByRole('dialog')).toBeVisible();
			await poserLeVerrou();
			await page.getByTestId('invoice-settlement-cancel-confirm').click();
			const refusClient = page.getByTestId('invoice-settlement-cancel-error');
			await expect(refusClient).toContainText('Cette écriture est dans une période verrouillée');
			await expect(refusClient).toContainText(
				"La période est verrouillée jusqu'au 31.12.2025 inclus.",
			);
			await expect(refusClient).not.toContainText('2025-12-31');
			await capture(page, testInfo, 'refus-au-clic-reglement-client');
			await leverLeVerrou();

			// Fiche fournisseur : le bouton est offert à la lecture, le verrou posé avant le clic ; la
			// facture est relue (le bouton cède la place au motif) et le refus reste affiché.
			await page.goto(`/supplier-invoices/${fournisseur}`);
			await expect(page.getByTestId('supplier-invoice-cancel')).toBeVisible();
			await poserLeVerrou();
			const confirmation = accepterEtNoter(page, testInfo, 'refus-au-clic-facture-fournisseur');
			await page.getByTestId('supplier-invoice-cancel').click();
			await confirmation;
			const refusFournisseur = page.getByTestId('supplier-invoice-cancel-error');
			await expect(refusFournisseur).toContainText(
				"La période est verrouillée jusqu'au 31.12.2025 inclus.",
			);
			await expect(refusFournisseur).not.toContainText('2025-12-31');
			// La relecture a remplacé le bouton par le motif (L3).
			await expect(page.getByTestId('supplier-invoice-cancel')).toHaveCount(0);
			await expect(page.getByTestId('supplier-invoice-cancel-blocked')).toContainText('31.12.2025');
			await capture(page, testInfo, 'refus-au-clic-facture-fournisseur');
		} finally {
			// Un verrou laissé en place verrouillerait les specs suivantes.
			await leverLeVerrou();
			await disposeContextSafe(ctx);
		}
	});
});
