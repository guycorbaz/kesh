/**
 * Story 30-1b (#686, AC-b12) — la réconciliation règle les factures fournisseur d'un débit.
 *
 * ⚠️ **Ces tests portent la recette** : Guy ne teste qu'en fin d'epic. Ce sont les seuls à voir
 * l'identifiant de la facture fournisseur traverser la frontière HTTP, de la proposition à
 * `POST /reconciliation/accept` (`type: 'supplier_invoice'`), puis le dé-rapprochement le défaire
 * (`CLAUDE.md`, § *E2E*).
 *
 * Trois parcours :
 * 1. **avec QR** — une facture fournisseur ouverte, portant une référence de paiement QR, datée
 *    et échue plusieurs mois avant le débit et **saisie le jour du test** (l'usage de Guy) ; un
 *    relevé dont le débit porte la référence ; *Réconciliation* → la facture proposée (badge,
 *    fournisseur, numéro, montant) → *Accepter* → la fiche dit « Payée » à la date du paiement,
 *    l'écriture de règlement est lettrée, la transaction rapprochée → *Annuler le rapprochement*
 *    (détail de l'import) → la facture redevient ouverte et la ligne revient ;
 * 2. un **crédit** du même montant ne propose pas la facture fournisseur ;
 * 3. **sans QR** — facture sans référence de paiement, débit sans référence, désigné par le
 *    montant et le nom du fournisseur → proposée, acceptée, payée.
 *
 * ⚠️ **Montants, références et contreparties UNIQUES à chaque exécution** (patron de
 * `reconciliation-link.spec.ts`) ; fixture CAMT **synthétique**, `camt053_v04_minimal.xml`
 * réécrite en `DBIT` (le créancier en contrepartie). En `finally` : rapprochement annulé, facture
 * annulée, import supprimé.
 *
 * # Recette visuelle
 *
 * Captures de la ligne proposée et du dialogue d'annulation, dans `KESH_E2E_CAPTURES_DIR` s'il est
 * posé, sinon dans le dossier de sortie du test. **Schéma clair seulement** : Kesh n'a pas de thème
 * sombre (C-28-1-7). Hors dépôt, sans `toHaveScreenshot`.
 *
 * Pré-requis : MariaDB up + KESH_TEST_MODE=true (cf. docs/testing.md).
 */

import { expect, test, type Page, type TestInfo } from '@playwright/test';
import fs from 'fs';
import path from 'path';
import { fileURLToPath } from 'url';
import {
	seedTestState,
	clearAuthStorage,
	authedApiContext,
	disposeContextSafe,
} from './helpers/test-state';
import { createSupplierInvoiceViaApi, wiredBankAccountViaApi } from './helpers/api-fixtures';

const FIXTURE = path.join(
	path.dirname(fileURLToPath(import.meta.url)),
	'fixtures',
	'camt053_v04_minimal.xml',
);
const TEST_IBAN = 'CH4431999123000889012';
/** La date du débit, celle de la fixture (comptabilisation et valeur). */
const PAYMENT_DATE = '2026-05-15';

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

/** Le dossier des captures : `KESH_E2E_CAPTURES_DIR`, sinon la sortie du test. */
function capturePath(testInfo: TestInfo, name: string): string {
	const dossier = process.env.KESH_E2E_CAPTURES_DIR;
	return dossier ? path.join(dossier, `${name}.png`) : testInfo.outputPath(`${name}.png`);
}

type Decor = {
	bankAccountId: number;
	importId: number;
	txId: number;
	supplierInvoiceId: number;
	number: string | null;
	supplierName: string;
	amount: string;
};

/**
 * La facture fournisseur ouverte et le relevé qui la paie. `sens` : `DBIT` (un paiement) ou
 * `CRDT` ; `withReference` : la facture porte une référence de paiement QR et le débit la porte.
 */
async function decor(
	page: Page,
	tag: string,
	sens: 'DBIT' | 'CRDT',
	withReference: boolean,
): Promise<Decor> {
	const unique = `${Date.now()}${Math.floor(Math.random() * 1000)}`;
	const supplierName = `Fournisseur ${tag} ${unique}`;
	const cents = 10000 + (Date.now() % 30000);
	const amount = (cents / 100).toFixed(2);
	// Une référence numérique de 27 chiffres, comme une QRR, unique à l'exécution.
	const qrr = withReference ? `21${unique}`.padEnd(27, '7').slice(0, 27) : null;
	const number = withReference ? `FF-${unique}` : null;
	const { supplierInvoiceId } = await createSupplierInvoiceViaApi(page, {
		supplierName,
		number,
		paymentReference: qrr,
		// Datée et échue plusieurs mois avant le paiement, saisie aujourd'hui.
		invoiceDate: '2026-02-10',
		dueDate: '2026-03-12',
		amount,
	});
	const bankAccountId = await wiredBankAccountViaApi(page, TEST_IBAN);
	let xml = fs.readFileSync(FIXTURE, 'utf8');
	const remplacements: [string, string][] = [
		['STMT-2026-05-001', `STMT-SUP301B-${unique}`],
		['BANK-TX-42', `BANK-TX-SUP301B-${unique}`],
		['E2E-2026-05-001', `E2E-SUP301B-${unique}`],
		[
			'<Amt Ccy="CHF">1234.56</Amt>\n        <CdtDbtInd>CRDT</CdtDbtInd>',
			`<Amt Ccy="CHF">${amount}</Amt>\n        <CdtDbtInd>${sens}</CdtDbtInd>`,
		],
		['<Nm>Acme SA</Nm>', `<Nm>${supplierName}</Nm>`],
	];
	if (sens === 'DBIT') {
		// Pour un paiement, la contrepartie est le créancier.
		remplacements.push(['<Dbtr>', '<Cdtr>'], ['</Dbtr>', '</Cdtr>']);
		remplacements.push(['<DbtrAcct>', '<CdtrAcct>'], ['</DbtrAcct>', '</CdtrAcct>']);
	}
	if (qrr) {
		remplacements.push(['<Ref>RF18539007547034</Ref>', `<Ref>${qrr}</Ref>`]);
	} else {
		// Sans référence structurée : le parseur se rabat sur la référence de la banque, unique.
		remplacements.push([
			'<Strd>\n                <CdtrRefInf>\n                  <Ref>RF18539007547034</Ref>\n                </CdtrRefInf>\n              </Strd>\n',
			'',
		]);
	}
	for (const [from, to] of remplacements) {
		expect(xml.includes(from), `fixture : « ${from} » introuvable`).toBeTruthy();
		xml = xml.replace(from, to);
	}
	const ctx = await authedApiContext(page);
	try {
		const imported = await ctx.post('/api/v1/bank-imports', {
			multipart: {
				bankAccountId: bankAccountId.toString(),
				file: {
					name: `fournisseur-${unique}.xml`,
					mimeType: 'application/xml',
					buffer: Buffer.from(xml, 'utf8'),
				},
				confirmBalanceMismatch: 'true',
				confirmDuplicateFile: 'true',
			},
		});
		expect(imported.status(), await imported.text()).toBe(201);
		const importId = ((await imported.json()) as { id: number }).id;
		const detail = (await (await ctx.get(`/api/v1/bank-imports/${importId}`)).json()) as {
			transactions: Array<{ id: number; amount: string }>;
		};
		expect(detail.transactions).toHaveLength(1);
		const expected = sens === 'DBIT' ? `-${amount}` : amount;
		expect(Number(detail.transactions[0].amount)).toBe(Number(expected));
		return {
			bankAccountId,
			importId,
			txId: detail.transactions[0].id,
			supplierInvoiceId,
			number,
			supplierName,
			amount,
		};
	} finally {
		await disposeContextSafe(ctx);
	}
}

/** Défait le décor : rapprochement annulé (s'il y en a un), facture annulée, import supprimé. */
async function nettoyer(page: Page, d: Decor): Promise<void> {
	const ctx = await authedApiContext(page);
	try {
		await ctx.post(`/api/v1/reconciliation/transactions/${d.txId}/cancel`, { data: {} });
		await ctx.post(`/api/v1/supplier-invoices/${d.supplierInvoiceId}/cancel`, { data: {} });
		await ctx.delete(`/api/v1/bank-imports/${d.importId}`);
	} finally {
		await disposeContextSafe(ctx);
	}
}

/** Le statut d'une facture fournisseur et son `paidAt`, par l'API. */
async function factureFournisseur(
	page: Page,
	id: number,
): Promise<{ status: string; paidAt: string | null; settlementJournalEntryId: number | null }> {
	const ctx = await authedApiContext(page);
	try {
		const res = await ctx.get(`/api/v1/supplier-invoices/${id}`);
		expect(res.ok()).toBeTruthy();
		const body = await res.json();
		const inv = body.invoice ?? body;
		return {
			status: inv.status,
			paidAt: inv.paidAt ?? null,
			settlementJournalEntryId: inv.settlementJournalEntryId ?? null,
		};
	} finally {
		await disposeContextSafe(ctx);
	}
}

/** La ligne de la transaction dans *Réconciliation*, compte choisi. */
async function ligneDeReconciliation(page: Page, d: Decor) {
	await page.goto('/reconciliation');
	await page.getByTestId('bank-account-select').selectOption(d.bankAccountId.toString());
	const ligne = page.locator(`[data-testid="reconciliation-row"][data-tx-id="${d.txId}"]`);
	await expect(ligne).toBeVisible({ timeout: 10000 });
	return ligne;
}

/** Accepte la proposition affichée de la ligne : « 1 opération(s) réussie(s). », la ligne part. */
async function accepter(page: Page, d: Decor): Promise<void> {
	const ligne = page.locator(`[data-testid="reconciliation-row"][data-tx-id="${d.txId}"]`);
	await ligne.getByTestId('tx-checkbox').check();
	await page.getByTestId('reconciliation-accept-btn').click();
	await expect(page.getByTestId('reconciliation-success')).toHaveText(
		/^\s*1\s+opération\(s\) réussie\(s\)\.\s*$/,
	);
	await expect(ligne).toHaveCount(0);
}

test('avec QR — un débit propose la facture fournisseur, Accepter la règle, Annuler le rapprochement la rouvre', async ({
	page,
}, testInfo) => {
	await login(page);
	const d = await decor(page, 'QR', 'DBIT', true);
	try {
		// 1. La proposition : badge, fournisseur, numéro, montant.
		const ligne = await ligneDeReconciliation(page, d);
		const candidate = ligne.getByTestId('candidate-top-1');
		await expect(candidate).toHaveAttribute('data-candidate-type', 'supplier_invoice');
		await expect(candidate).toHaveAttribute(
			'data-supplier-invoice-id',
			d.supplierInvoiceId.toString(),
		);
		await expect(candidate).toContainText(d.supplierName);
		await expect(candidate).toContainText(d.number!);
		await expect(candidate).toContainText(String(Number(d.amount)));
		await expect(ligne.getByTestId('candidate-supplier-invoice-badge')).toContainText(
			'Facture fournisseur',
		);
		await ligne.screenshot({ path: capturePath(testInfo, '30-1b-ligne-proposee-clair') });

		// 2. Accepter.
		await accepter(page, d);

		// 3. La fiche : payée, à la date du paiement ; le lien vers l'écriture de règlement.
		await page.goto(`/supplier-invoices/${d.supplierInvoiceId}`);
		await expect(page.getByTestId('supplier-invoice-status')).toHaveText('Payée');
		// Story 30-2c (AC25) : la fiche affiche la DATE au format suisse, jamais `paidAt` brut.
		await expect(page.getByTestId('supplier-invoice-paid-info')).toContainText(
			PAYMENT_DATE.split('-').reverse().join('.'),
		);
		await expect(page.getByTestId('supplier-invoice-paid-info')).not.toContainText('T00:00:00');
		const paid = await factureFournisseur(page, d.supplierInvoiceId);
		expect(paid.status).toBe('paid');
		expect(paid.paidAt).toMatch(new RegExp(`^${PAYMENT_DATE}T00:00:00`));
		expect(paid.settlementJournalEntryId).not.toBeNull();
		await expect(page.getByTestId('supplier-invoice-settlement-entry')).toHaveAttribute(
			'href',
			`/journal-entries/${paid.settlementJournalEntryId}`,
		);

		// 4. L'écriture de règlement est lettrée (le paiement avec son achat).
		await page.goto(`/journal-entries/${paid.settlementJournalEntryId}`);
		const codes = page.locator('[data-testid^="entry-line-lettering-"]');
		await expect(codes.first()).toBeVisible();
		const textes = (await codes.allTextContents()).map((t) => t.trim()).filter((t) => t !== '');
		expect(textes.length, 'une ligne de règlement lettrée').toBeGreaterThan(0);

		// 5. Le détail de l'import : rapprochée, pointant l'écriture de règlement.
		await page.goto(`/bank-import/${d.importId}`);
		const row = page.locator(`[data-testid="detail-tx-row"][data-tx-id="${d.txId}"]`);
		await expect(row).toContainText('reconciled');
		await expect(row.getByTestId('detail-tx-entry-link')).toHaveAttribute(
			'href',
			`/journal-entries/${paid.settlementJournalEntryId}`,
		);

		// 6. Annuler le rapprochement : le texte fournisseur, puis la facture rouverte.
		await row.getByTestId('detail-tx-cancel-reconciliation').click();
		const dialog = page.getByTestId('reconciliation-cancel-dialog');
		const texte = dialog.getByTestId('reconciliation-cancel-confirm-text');
		await expect(texte).toContainText(`facture fournisseur ${d.number}`);
		await expect(texte).toContainText('redeviendra ouverte, à payer');
		await dialog.screenshot({ path: capturePath(testInfo, '30-1b-dialogue-annulation-clair') });
		await dialog.getByTestId('reconciliation-cancel-submit').click();
		await expect(dialog).toBeHidden();
		await expect(row).toContainText('pending');
		const rouverte = await factureFournisseur(page, d.supplierInvoiceId);
		expect(rouverte.status).toBe('open');

		// 7. La ligne revient dans la réconciliation, la même facture proposée.
		const retour = await ligneDeReconciliation(page, d);
		await expect(retour.getByTestId('candidate-top-1')).toHaveAttribute(
			'data-supplier-invoice-id',
			d.supplierInvoiceId.toString(),
		);
	} finally {
		await nettoyer(page, d);
	}
});

test('un crédit du même montant ne propose jamais la facture fournisseur', async ({ page }) => {
	await login(page);
	const d = await decor(page, 'Credit', 'CRDT', true);
	try {
		const ligne = await ligneDeReconciliation(page, d);
		await expect(ligne.locator('[data-candidate-type="supplier_invoice"]')).toHaveCount(0);
		await expect(ligne.getByTestId('candidate-supplier-invoice-badge')).toHaveCount(0);
	} finally {
		await nettoyer(page, d);
	}
});

test('sans QR — un débit sans référence est désigné par le montant et le fournisseur, et accepté', async ({
	page,
}) => {
	await login(page);
	const d = await decor(page, 'SansQR', 'DBIT', false);
	try {
		const ligne = await ligneDeReconciliation(page, d);
		const candidate = ligne.getByTestId('candidate-top-1');
		await expect(candidate).toHaveAttribute('data-candidate-type', 'supplier_invoice');
		await expect(candidate).toHaveAttribute(
			'data-supplier-invoice-id',
			d.supplierInvoiceId.toString(),
		);
		// Sans numéro : l'identifiant, précédé de « # ».
		await expect(candidate).toContainText(`#${d.supplierInvoiceId}`);
		await accepter(page, d);
		const paid = await factureFournisseur(page, d.supplierInvoiceId);
		expect(paid.status).toBe('paid');
		expect(paid.paidAt).toMatch(new RegExp(`^${PAYMENT_DATE}T00:00:00`));
	} finally {
		await nettoyer(page, d);
	}
});
