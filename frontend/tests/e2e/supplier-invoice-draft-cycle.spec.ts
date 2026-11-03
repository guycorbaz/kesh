/**
 * Story v017-4 (#849, AC-30) — le cycle brouillon → validée → payée d'une facture
 * fournisseur, de bout en bout.
 *
 * **Écrite en deux temps** (C-v017-4-31, dérogation) : la 4a livre les étapes **par
 * l'API** — ce fichier tel quel —, la 4b y ajoute les étapes **par l'écran** (formulaire,
 * fiche, page d'édition). Les étapes API prouvent ce qu'aucun test Rust ne voit : que les
 * valeurs traversent la frontière HTTP du serveur réellement servi.
 *
 * Décor synthétique : le seed `with-company`, un fournisseur et un compte bancaire créés
 * par l'API, un relevé **saisi à la main** par l'API (débit égal au total), équilibré au
 * centime.
 *
 * **Story v017-4b — les étapes par l'écran** (`describe` « par l'écran ») : le même cycle
 * conduit au formulaire, sur la fiche et sur la page d'édition, payé par la page
 * *Réconciliation* ; la suppression d'un brouillon par l'Administrateur ; le mode « document
 * avec QR » de l'édition d'un brouillon né d'une complétion. Les captures de la recette
 * visuelle (thème clair) s'écrivent dans `KESH_E2E_CAPTURES_DIR`, sinon dans la sortie du test.
 */
import path from 'node:path';
import { expect, test, type APIRequestContext, type Page, type TestInfo } from '@playwright/test';
import {
	seedTestState,
	clearAuthStorage,
	authedApiContext,
	disposeContextSafe,
} from './helpers/test-state';
import {
	createManualDebitStatementViaApi,
	wiredBankAccountViaApi,
} from './helpers/api-fixtures';
import {
	deposit,
	fillFirstInvoiceLine,
	importViaApi,
	readFixture,
	token,
	uniqueCopy,
} from './helpers/inbox';

/** Le dossier des captures : `KESH_E2E_CAPTURES_DIR`, sinon la sortie du test (hors dépôt). */
function capturePath(testInfo: TestInfo, name: string): string {
	const dossier = process.env.KESH_E2E_CAPTURES_DIR;
	return dossier ? path.join(dossier, `${name}.png`) : testInfo.outputPath(`${name}.png`);
}

/** Capture la page entière, thème clair (Kesh n'a pas de thème sombre). */
async function capture(page: Page, testInfo: TestInfo, name: string): Promise<void> {
	await page.screenshot({ path: capturePath(testInfo, `v0174b-${name}`), fullPage: true });
}

/** IBAN synthétique du compte bancaire du décor. */
const TEST_IBAN = 'CH5604835012345678009';
const SUPPLIER = 'Fournisseur cycle brouillon SA';

test.beforeAll(async () => {
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

type Account = {
	id: number;
	number: string;
	accountType: string;
	active: boolean;
	postable: boolean;
};

/** Le fournisseur du décor et deux comptes de charge distincts (le « faux » et le bon). */
async function decor(ctx: APIRequestContext): Promise<{
	supplierId: number;
	wrong: number;
	right: number;
}> {
	const accounts = (await (await ctx.get('/api/v1/accounts')).json()) as Account[];
	const expenses = accounts.filter((a) => a.accountType === 'Expense' && a.active && a.postable);
	// Le seed `with-company` ne porte qu'un compte de charge imputable : le second (le
	// « bon ») se crée ici, sous un numéro absent du plan.
	if (expenses.length < 2) {
		const number = ['6990', '6991', '6992'].find((n) => !accounts.some((a) => a.number === n));
		const created = await ctx.post('/api/v1/accounts', {
			data: {
				number,
				name: 'Charges cycle brouillon',
				accountType: 'Expense',
				postable: true,
				role: null,
			},
		});
		expect(created.ok(), `compte de charge : ${created.status()} ${await created.text()}`).toBeTruthy();
		expenses.push((await created.json()) as Account);
	}
	expect(expenses.length, 'deux comptes de charge imputables').toBeGreaterThanOrEqual(2);
	const payable = accounts.find((a) => a.number === '2000');
	const settings = await (await ctx.get('/api/v1/company/invoice-settings')).json();
	if (settings.defaultPayableAccountId == null) {
		const put = await ctx.put('/api/v1/company/invoice-settings', {
			data: { ...settings, defaultPayableAccountId: payable!.id },
		});
		expect(put.ok(), `compte créanciers : ${put.status()}`).toBeTruthy();
	}
	const sup = await ctx.post('/api/v1/contacts', {
		data: { contactType: 'Entreprise', name: SUPPLIER, isClient: false, isSupplier: true },
	});
	expect(sup.ok(), `fournisseur : ${sup.status()}`).toBeTruthy();
	return {
		supplierId: (await sup.json()).id as number,
		wrong: expenses[0].id,
		right: expenses[1].id,
	};
}

function body(supplierId: number, account: number, price: string, number: string) {
	return {
		contactId: supplierId,
		supplierInvoiceNumber: number,
		invoiceDate: '2026-05-10',
		dueDate: null,
		lines: [
			{
				description: 'Fourniture',
				quantity: '1',
				unitPrice: price,
				vatRate: '0',
				expenseAccountId: account,
			},
		],
	};
}

test.describe('Cycle brouillon → validée → payée (étapes API, 4a)', () => {
	test('brouillon, correction, validation, dévalidation, revalidation, puis payée par la réconciliation', async ({
		page,
	}) => {
		await login(page);
		const bankAccountId = await wiredBankAccountViaApi(page, TEST_IBAN);
		const ctx = await authedApiContext(page);
		try {
			const d = await decor(ctx);

			// Brouillon, avec un compte de charge faux : aucune écriture.
			const created = await ctx.post('/api/v1/supplier-invoices', {
				data: body(d.supplierId, d.wrong, '100.20', 'FF-CYCLE-1'),
			});
			expect(created.status()).toBe(201);
			let inv = await created.json();
			expect(inv.status).toBe('draft');
			expect(inv.purchaseJournalEntryId).toBeNull();

			// Correction du compte (PUT), puis validation : l'écriture est passée.
			let res = await ctx.put(`/api/v1/supplier-invoices/${inv.id}`, {
				data: { ...body(d.supplierId, d.right, '100.20', 'FF-CYCLE-1'), version: inv.version },
			});
			expect(res.status(), await res.text()).toBe(200);
			inv = await res.json();
			res = await ctx.post(`/api/v1/supplier-invoices/${inv.id}/validate`, {
				data: { version: inv.version },
			});
			expect(res.status(), await res.text()).toBe(200);
			inv = await res.json();
			expect(inv.status).toBe('open');
			expect(typeof inv.purchaseJournalEntryId).toBe('number');

			// Dévalidation : l'écriture d'achat est supprimée.
			const entry = inv.purchaseJournalEntryId as number;
			res = await ctx.post(`/api/v1/supplier-invoices/${inv.id}/unvalidate`, {
				data: { version: inv.version },
			});
			expect(res.status(), await res.text()).toBe(200);
			inv = await res.json();
			expect(inv.status).toBe('draft');
			expect(inv.purchaseJournalEntryId).toBeNull();
			expect((await ctx.get(`/api/v1/journal-entries/${entry}`)).status()).toBe(404);

			// Prix corrigé (100.20 → 100.00), puis validation.
			res = await ctx.put(`/api/v1/supplier-invoices/${inv.id}`, {
				data: { ...body(d.supplierId, d.right, '100.00', 'FF-CYCLE-1'), version: inv.version },
			});
			expect(res.status(), await res.text()).toBe(200);
			inv = await res.json();
			expect(Number(inv.totalAmount)).toBe(100);
			res = await ctx.post(`/api/v1/supplier-invoices/${inv.id}/validate`, {
				data: { version: inv.version },
			});
			expect(res.status(), await res.text()).toBe(200);

			// Un relevé saisi à la main porte un débit égal : la facture est proposée…
			await createManualDebitStatementViaApi(page, {
				bankAccountId,
				bookingDate: '2026-05-20',
				amount: '100.00',
				counterpartyName: SUPPLIER,
				details: 'Paiement fournisseur cycle',
			});
			const props = await (
				await ctx.get(`/api/v1/reconciliation/proposals?bankAccountId=${bankAccountId}`)
			).json();
			const proposal = (props.proposals as Array<Record<string, unknown>>).find((p) =>
				(p.candidates as Array<Record<string, unknown>>).some(
					(c) => c.supplierInvoiceId === inv.id,
				),
			);
			expect(proposal, 'la facture validée est proposée pour le débit').toBeTruthy();

			// … et l'accepter la rend payée.
			res = await ctx.post('/api/v1/reconciliation/accept', {
				data: {
					bankAccountId,
					proposals: [
						{
							type: 'supplier_invoice',
							bankTransactionId: proposal!.bankTransactionId,
							supplierInvoiceId: inv.id,
						},
					],
				},
			});
			expect(res.status()).toBe(200);
			const accepted = await res.json();
			expect(accepted.failed, JSON.stringify(accepted)).toHaveLength(0);
			const paid = await (await ctx.get(`/api/v1/supplier-invoices/${inv.id}`)).json();
			expect(paid.status).toBe('paid');
		} finally {
			await disposeContextSafe(ctx);
		}
	});

	test("un brouillon au montant d'un débit n'est pas proposé, et l'administrateur le supprime", async ({
		page,
	}) => {
		await login(page);
		const bankAccountId = await wiredBankAccountViaApi(page, TEST_IBAN);
		const ctx = await authedApiContext(page);
		try {
			const d = await decor(ctx);
			const created = await ctx.post('/api/v1/supplier-invoices', {
				data: body(d.supplierId, d.right, '77.70', 'FF-CYCLE-2'),
			});
			expect(created.status()).toBe(201);
			const draft = await created.json();
			await createManualDebitStatementViaApi(page, {
				bankAccountId,
				bookingDate: '2026-06-12',
				amount: '77.70',
				counterpartyName: SUPPLIER,
				details: 'Débit au montant du brouillon',
			});
			const props = await (
				await ctx.get(`/api/v1/reconciliation/proposals?bankAccountId=${bankAccountId}`)
			).json();
			for (const p of props.proposals as Array<Record<string, unknown>>) {
				for (const c of p.candidates as Array<Record<string, unknown>>) {
					expect(c.supplierInvoiceId, 'un brouillon n’est jamais proposé').not.toBe(draft.id);
				}
			}
			const del = await ctx.delete(`/api/v1/supplier-invoices/${draft.id}`);
			expect(del.status()).toBe(204);
			expect((await ctx.get(`/api/v1/supplier-invoices/${draft.id}`)).status()).toBe(404);
		} finally {
			await disposeContextSafe(ctx);
		}
	});
});

// ---------------------------------------------------------------------------
// Story v017-4b — les étapes par l'écran (AC-30, AC-31)
// ---------------------------------------------------------------------------

/** L'identifiant de la facture de l'URL de sa fiche. */
function idFromUrl(page: Page): number {
	const m = /\/supplier-invoices\/(\d+)$/.exec(new URL(page.url()).pathname);
	expect(m, `URL de fiche attendue : ${page.url()}`).toBeTruthy();
	return Number(m![1]);
}

/** Valide la facture affichée depuis sa fiche (confirmation comprise). */
async function validerParLaFiche(page: Page, testInfo?: TestInfo, captureName?: string) {
	await page.getByTestId('supplier-invoice-validate').click();
	const dialog = page.getByTestId('supplier-invoice-validate-dialog');
	await expect(dialog).toContainText('L’écriture d’achat sera passée au journal');
	if (testInfo && captureName) await capture(page, testInfo, captureName);
	await page.getByTestId('supplier-invoice-validate-confirm').click();
	await expect(page.getByTestId('supplier-invoice-status')).toHaveText('Validée');
}

test.describe('Cycle brouillon → validée → payée (étapes par l’écran, 4b)', () => {
	test('saisie d’un brouillon au compte faux, correction, validation, dévalidation, prix corrigé, revalidation, paiement par la réconciliation', async ({
		page,
	}, testInfo) => {
		await login(page);
		const bankAccountId = await wiredBankAccountViaApi(page, TEST_IBAN);
		const ctx = await authedApiContext(page);
		try {
			const d = await decor(ctx);
			const number = `FF-ECRAN-${token()}`;

			// 1. Le formulaire : un brouillon, avec un compte de charge faux. Le prix est saisi
			//    à la virgule (#830) ; une échéance antérieure est refusée sous son champ (#853).
			await page.goto('/supplier-invoices/new');
			await expect(page.getByTestId('supplier-invoice-submit')).toHaveText('Enregistrer le brouillon');
			await page.getByTestId('supplier-invoice-supplier').selectOption(String(d.supplierId));
			await page.getByTestId('supplier-invoice-number').fill(number);
			await page.getByTestId('supplier-invoice-date').fill('2026-07-10');
			const line = page.getByTestId('supplier-invoice-line').first();
			await line.getByTestId('supplier-invoice-line-desc').fill('Fourniture écran');
			await line.getByTestId('supplier-invoice-line-qty').fill('3');
			await line.getByTestId('supplier-invoice-line-unit-price').fill('33,3333');
			await line.getByTestId('supplier-invoice-line-account').selectOption(String(d.wrong));
			// #835 : le total vivant aux décimales exactes, sous le refus de la ligne.
			await expect(page.getByTestId('supplier-invoice-sum-ttc')).toContainText('99.9999');
			await page.getByTestId('supplier-invoice-due').fill('2026-07-01');
			await expect(page.getByTestId('supplier-invoice-due-error')).toHaveText(
				"L'échéance (01.07.2026) précède la date de la facture (10.07.2026).",
			);
			await capture(page, testInfo, 'formulaire-echeance-et-total-exact');
			await page.getByTestId('supplier-invoice-due').fill('2026-08-09');
			await line.getByTestId('supplier-invoice-line-qty').fill('1');
			await line.getByTestId('supplier-invoice-line-unit-price').fill('100,20');
			await page.getByTestId('supplier-invoice-submit').click();
			await expect(page).toHaveURL(/\/supplier-invoices\/\d+$/);
			const id = idFromUrl(page);

			// 2. La fiche d'un brouillon : aucune écriture, ni Payer ni Annuler.
			await expect(page.getByTestId('supplier-invoice-status')).toHaveText('Brouillon');
			await expect(page.getByTestId('supplier-invoice-purchase-entry')).toHaveCount(0);
			await expect(page.getByTestId('supplier-invoice-pay')).toHaveCount(0);
			await expect(page.getByTestId('supplier-invoice-cancel-block')).toHaveCount(0);
			await capture(page, testInfo, 'fiche-brouillon-administrateur');
			let inv = await (await ctx.get(`/api/v1/supplier-invoices/${id}`)).json();
			expect(inv.status).toBe('draft');
			expect(inv.purchaseJournalEntryId).toBeNull();
			expect(inv.lines[0].unitPrice).toMatch(/^100\.20*$/);

			// 3. Modifier : le bon compte, puis Enregistrer.
			await page.getByTestId('supplier-invoice-edit').click();
			await expect(page).toHaveURL(new RegExp(`/supplier-invoices/${id}/edit$`));
			await expect(page.getByTestId('supplier-invoice-scan-btn')).toBeVisible();
			await capture(page, testInfo, 'edition-sans-document');
			await page
				.getByTestId('supplier-invoice-line')
				.first()
				.getByTestId('supplier-invoice-line-account')
				.selectOption(String(d.right));
			await expect(page.getByTestId('supplier-invoice-submit')).toHaveText('Enregistrer');
			await page.getByTestId('supplier-invoice-submit').click();
			await expect(page).toHaveURL(new RegExp(`/supplier-invoices/${id}$`));
			await expect(page.getByTestId('supplier-invoice-status')).toHaveText('Brouillon');

			// 4. Valider : l'écriture d'achat est passée, la fiche y mène.
			await validerParLaFiche(page, testInfo, 'confirmation-valider');
			const link = page.getByTestId('supplier-invoice-purchase-entry');
			await expect(link).toHaveAttribute('href', /^\/journal-entries\/\d+$/);
			const entryHref = (await link.getAttribute('href'))!;
			await expect(page.getByTestId('supplier-invoice-unvalidate')).toBeVisible();
			await capture(page, testInfo, 'fiche-validee-devalider');

			// 5. Dévalider : l'écriture d'achat est supprimée, la facture redevient brouillon.
			await page.getByTestId('supplier-invoice-unvalidate').click();
			await expect(page.getByTestId('supplier-invoice-unvalidate-dialog')).toContainText('trou');
			await capture(page, testInfo, 'confirmation-devalider');
			await page.getByTestId('supplier-invoice-unvalidate-confirm').click();
			await expect(page.getByTestId('supplier-invoice-status')).toHaveText('Brouillon');
			await expect(page.getByTestId('supplier-invoice-purchase-entry')).toHaveCount(0);
			const entryId = entryHref.split('/').pop();
			expect((await ctx.get(`/api/v1/journal-entries/${entryId}`)).status()).toBe(404);

			// 6. Le prix corrigé (100.20 → 100.00), puis revalider.
			await page.getByTestId('supplier-invoice-edit').click();
			await page
				.getByTestId('supplier-invoice-line')
				.first()
				.getByTestId('supplier-invoice-line-unit-price')
				.fill('100.00');
			await page.getByTestId('supplier-invoice-submit').click();
			await expect(page).toHaveURL(new RegExp(`/supplier-invoices/${id}$`));
			await validerParLaFiche(page);
			inv = await (await ctx.get(`/api/v1/supplier-invoices/${id}`)).json();
			expect(inv.status).toBe('open');
			expect(Number(inv.totalAmount)).toBe(100);

			// 7. Un relevé saisi à la main porte un débit égal : la réconciliation propose la
			//    facture, Accepter la rend payée.
			await createManualDebitStatementViaApi(page, {
				bankAccountId,
				bookingDate: '2026-07-20',
				amount: '100.00',
				counterpartyName: SUPPLIER,
				details: 'Paiement fournisseur écran',
			});
			await page.goto('/reconciliation');
			await page.getByTestId('bank-account-select').selectOption(String(bankAccountId));
			const ligne = page
				.locator('[data-testid="reconciliation-row"]')
				.filter({ has: page.locator(`[data-supplier-invoice-id="${id}"]`) });
			await expect(ligne).toHaveCount(1, { timeout: 10000 });
			await ligne.getByTestId('tx-checkbox').check();
			await page.getByTestId('reconciliation-accept-btn').click();
			await expect(page.getByTestId('reconciliation-success')).toBeVisible();
			await page.goto(`/supplier-invoices/${id}`);
			await expect(page.getByTestId('supplier-invoice-status')).toHaveText('Payée');
			await expect(page.getByTestId('supplier-invoice-unvalidate')).toHaveCount(0);
		} finally {
			await disposeContextSafe(ctx);
		}
	});

	test('liste, refus de dévalidation, contrôle de clôture K8, suppression d’un brouillon par l’administrateur', async ({
		page,
	}, testInfo) => {
		await login(page);
		const bankAccountId = await wiredBankAccountViaApi(page, TEST_IBAN);
		const ctx = await authedApiContext(page);
		try {
			const d = await decor(ctx);
			// Un brouillon, et une facture validée placée dans un lot créé (`generated`).
			const draftRes = await ctx.post('/api/v1/supplier-invoices', {
				data: body(d.supplierId, d.right, '45.50', `FF-LISTE-${token()}`),
			});
			expect(draftRes.status()).toBe(201);
			const draft = await draftRes.json();
			const lotRes = await ctx.post('/api/v1/supplier-invoices', {
				data: {
					...body(d.supplierId, d.right, '61.00', `FF-LOT-${token()}`),
					creditorIban: 'CH9300762011623852957',
				},
			});
			expect(lotRes.status()).toBe(201);
			const lotDraft = await lotRes.json();
			const val = await ctx.post(`/api/v1/supplier-invoices/${lotDraft.id}/validate`, {
				data: { version: lotDraft.version },
			});
			expect(val.status(), await val.text()).toBe(200);
			const batch = await ctx.post('/api/v1/payment-batches', {
				data: {
					bankAccountId,
					requestedExecutionDate: '2026-07-31',
					supplierInvoiceIds: [lotDraft.id],
				},
			});
			expect(batch.ok(), await batch.text()).toBeTruthy();
			expect((await batch.json()).batch, 'le lot est créé').not.toBeNull();

			// Revue P1 (B-5 = E-9, A-2) : une facture ANNULÉE réelle, pour que l'étape
			// « masquées puis affichées » prouve le filtre à travers HTTP.
			const cancelRes = await ctx.post('/api/v1/supplier-invoices', {
				data: body(d.supplierId, d.right, '17.25', `FF-ANNULEE-${token()}`),
			});
			expect(cancelRes.status()).toBe(201);
			const toCancel = await cancelRes.json();
			const valCancel = await ctx.post(`/api/v1/supplier-invoices/${toCancel.id}/validate`, {
				data: { version: toCancel.version },
			});
			expect(valCancel.status(), await valCancel.text()).toBe(200);
			const cancelled = await ctx.post(`/api/v1/supplier-invoices/${toCancel.id}/cancel`);
			expect(cancelled.status(), await cancelled.text()).toBe(200);
			expect((await cancelled.json()).status).toBe('cancelled');

			// La liste : le brouillon porte son badge ; l'annulée est absente case décochée,
			// présente (badge « Annulée ») case cochée.
			await page.goto('/supplier-invoices');
			const row = page.getByTestId('supplier-invoice-row').filter({ hasText: draft.supplierInvoiceNumber });
			const cancelledRow = page
				.getByTestId('supplier-invoice-row')
				.filter({ hasText: toCancel.supplierInvoiceNumber });
			await expect(row.getByTestId('supplier-invoice-status-badge')).toHaveText('Brouillon');
			await expect(page.getByTestId('supplier-invoices-show-cancelled')).not.toBeChecked();
			await expect(cancelledRow).toHaveCount(0);
			await capture(page, testInfo, 'liste-annulees-masquees');
			await page.getByTestId('supplier-invoices-show-cancelled').check();
			await expect(cancelledRow.getByTestId('supplier-invoice-status-badge')).toHaveText('Annulée');
			await expect(row).toBeVisible();
			await capture(page, testInfo, 'liste-annulees-affichees');
			await page.getByTestId('supplier-invoices-show-cancelled').uncheck();
			await expect(cancelledRow).toHaveCount(0);

			// Revue P1 (E-2 = A-1, limite L4), reprise en P2 (B2-1) : une facture placée dans un
			// lot CONFIRMÉ, puis dont le règlement est annulé, redevient validée ; sa fiche et la
			// confirmation de la dévalidation disent qu'elle FIGURE dans ce lot (un fait
			// historique, jamais « payée par ce lot »).
			const paidRes = await ctx.post('/api/v1/supplier-invoices', {
				data: {
					...body(d.supplierId, d.right, '72.40', `FF-LOT-CONFIRME-${token()}`),
					creditorIban: 'CH9300762011623852957',
				},
			});
			expect(paidRes.status()).toBe(201);
			const paidDraft = await paidRes.json();
			const valPaid = await ctx.post(`/api/v1/supplier-invoices/${paidDraft.id}/validate`, {
				data: { version: paidDraft.version },
			});
			expect(valPaid.status(), await valPaid.text()).toBe(200);
			const today = new Date().toISOString().slice(0, 10);
			const batch2 = await ctx.post('/api/v1/payment-batches', {
				data: { bankAccountId, requestedExecutionDate: today, supplierInvoiceIds: [paidDraft.id] },
			});
			expect(batch2.ok(), await batch2.text()).toBeTruthy();
			const batch2Id = (await batch2.json()).batch.id as number;
			const confirmed = await ctx.post(`/api/v1/payment-batches/${batch2Id}/confirm`, {
				data: { paymentDate: today },
			});
			expect(confirmed.ok(), await confirmed.text()).toBeTruthy();
			const settlementCancel = await ctx.post(
				`/api/v1/supplier-invoices/${paidDraft.id}/settlement/cancel`,
			);
			expect(settlementCancel.ok(), await settlementCancel.text()).toBeTruthy();
			await page.goto(`/supplier-invoices/${paidDraft.id}`);
			await expect(page.getByTestId('supplier-invoice-status')).toHaveText('Validée');
			const batchWarning = page.getByTestId('supplier-invoice-confirmed-batch-warning');
			await expect(batchWarning).toContainText(
				`Cette facture figure dans le lot de paiement n° ${batch2Id}, confirmé le `,
			);
			await expect(batchWarning).toContainText('vérifiez le relevé avant de changer son montant');
			await expect(batchWarning).not.toContainText('payée par');
			await capture(page, testInfo, 'fiche-validee-lot-confirme');
			await page.getByTestId('supplier-invoice-unvalidate').click();
			await expect(
				page.getByTestId('supplier-invoice-unvalidate-dialog-batch-warning'),
			).toContainText(`lot de paiement n° ${batch2Id}`);
			await capture(page, testInfo, 'confirmation-devalider-lot-confirme');
			// Fermer sans dévalider (Échap : aucun sélecteur sur un libellé traduit).
			await page.keyboard.press('Escape');
			await expect(page.getByTestId('supplier-invoice-unvalidate-dialog')).toHaveCount(0);
			await expect(page.getByTestId('supplier-invoice-status')).toHaveText('Validée');

			// Le refus de la dévalidation : la facture est dans un lot créé.
			await page.goto(`/supplier-invoices/${lotDraft.id}`);
			await page.getByTestId('supplier-invoice-unvalidate').click();
			await page.getByTestId('supplier-invoice-unvalidate-confirm').click();
			await expect(page.getByTestId('supplier-invoice-unvalidate-error')).toContainText(
				'Cette facture figure dans un lot de paiement en cours : annulez d\'abord le lot.',
			);
			await expect(page.getByTestId('supplier-invoice-status')).toHaveText('Validée');
			await capture(page, testInfo, 'refus-devalidation');

			// La clôture : K8 compte le brouillon, qui mène à sa fiche.
			const fys = (await (await ctx.get('/api/v1/fiscal-years')).json()) as Array<{ id: number }>;
			await page.goto('/settings/fiscal-years');
			await page.getByTestId(`fiscal-year-checks-${fys[0].id}`).click();
			const dialog = page.getByTestId('fiscal-year-checks-dialog');
			await expect(dialog.locator('[data-testid^="fiscal-year-checks-row-"]')).toHaveCount(8);
			const k8 = dialog.getByTestId('fiscal-year-checks-row-DRAFT_SUPPLIER_INVOICES');
			await expect(k8).toHaveAttribute('data-status', 'attention');
			await expect(k8.locator(`a[href="/supplier-invoices/${draft.id}"]`)).toHaveCount(1);
			// Revue P1 (A-2) : le dialogue défile ; K8, huitième et dernier, s'amène à la vue
			// avant la capture, et la ligne se capture aussi seule.
			await k8.scrollIntoViewIfNeeded();
			await expect(k8).toBeInViewport();
			await dialog.screenshot({ path: capturePath(testInfo, 'v0174b-cloture-k8') });
			await k8.screenshot({ path: capturePath(testInfo, 'v0174b-cloture-k8-ligne') });
			await dialog.getByTestId('fiscal-year-checks-close').click();

			// La suppression du brouillon, par l'administrateur.
			await page.goto(`/supplier-invoices/${draft.id}`);
			await page.getByTestId('supplier-invoice-delete').click();
			await expect(page.getByTestId('supplier-invoice-delete-dialog')).toBeVisible();
			await capture(page, testInfo, 'confirmation-supprimer');
			await page.getByTestId('supplier-invoice-delete-confirm').click();
			await expect(page).toHaveURL(/\/supplier-invoices$/);
			expect((await ctx.get(`/api/v1/supplier-invoices/${draft.id}`)).status()).toBe(404);
		} finally {
			await disposeContextSafe(ctx);
		}
	});

	test('un brouillon né d’un document à QR : date à saisir, coordonnées du QR verrouillées à l’édition, sans scan ; un Comptable ne le supprime pas', async ({
		page,
	}, testInfo) => {
		await login(page);
		const ctx = await authedApiContext(page);
		const username = `comptable-v0174b-${Date.now()}`;
		let userId: number | null = null;
		try {
			const d = await decor(ctx);
			const name = `e2e-v0174b-${token()}.png`;
			await deposit(name, uniqueCopy(await readFixture('spc_e2e_invoice.png'), 'png'));
			const report = await importViaApi(page);
			const entry = report.accepted.find((a) => a.fileName === name);
			expect(entry, `${name} importé`).toBeTruthy();

			// La complétion : la date démarre vide, avec son aide (#853).
			await page.goto(`/supplier-invoices/new?document=${entry!.importedSupplierInvoiceId}`);
			await expect(page.getByTestId('supplier-invoice-date')).toHaveValue('');
			await expect(page.getByTestId('supplier-invoice-date-help')).toBeVisible();
			await page.getByTestId('supplier-invoice-supplier').selectOption(String(d.supplierId));
			await page
				.getByTestId('supplier-invoice-line')
				.first()
				.getByTestId('supplier-invoice-line-desc')
				.fill('Prestation QR');
			await page.getByTestId('supplier-invoice-submit').click();
			await expect(page.getByTestId('supplier-invoice-date-error')).toHaveText(
				'Saisissez la date de la facture, telle qu’imprimée sur le document.',
			);
			await capture(page, testInfo, 'completion-date-vide');
			await fillFirstInvoiceLine(page, 'Prestation QR', '100.00', d.right);
			await page.getByTestId('supplier-invoice-submit').click();
			await expect(page).toHaveURL(/\/supplier-invoices\/\d+$/);
			const id = idFromUrl(page);
			await expect(page.getByTestId('supplier-invoice-status')).toHaveText('Brouillon');

			// L'édition : le mode « document avec QR », sans scan.
			await page.getByTestId('supplier-invoice-edit').click();
			await expect(page.getByTestId('supplier-invoice-qr-locked')).toBeVisible();
			await expect(page.getByTestId('supplier-invoice-iban')).toHaveAttribute('readonly', '');
			await expect(page.getByTestId('supplier-invoice-scan-btn')).toHaveCount(0);
			await capture(page, testInfo, 'edition-document-qr');

			// Un Comptable : Modifier et Valider, jamais Supprimer le brouillon.
			const created = await ctx.post('/api/v1/users', {
				data: { username, password: 'MotDePasse12345', role: 'Comptable' },
			});
			expect(created.ok(), `utilisateur : ${created.status()}`).toBeTruthy();
			userId = ((await created.json()) as { id: number }).id;
			await clearAuthStorage(page);
			await page.goto('/login');
			await page.fill('#username', username);
			await page.fill('#password', 'MotDePasse12345');
			await page.click('button[type="submit"]');
			await expect(page).toHaveURL('/');
			await page.goto(`/supplier-invoices/${id}`);
			await expect(page.getByTestId('supplier-invoice-validate')).toBeVisible();
			await expect(page.getByTestId('supplier-invoice-edit')).toBeVisible();
			await expect(page.getByTestId('supplier-invoice-delete')).toHaveCount(0);
			await capture(page, testInfo, 'fiche-brouillon-comptable');
		} finally {
			// Le contexte d'API reste celui de l'administrateur, ouvert avant le changement de session.
			if (userId !== null) await ctx.put(`/api/v1/users/${userId}/disable`);
			await disposeContextSafe(ctx);
		}
	});
});
