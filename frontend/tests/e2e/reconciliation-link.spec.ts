/**
 * Story 29-8 (#670, AC9) — une transaction réglée depuis la fiche de la facture se rattache à
 * ce règlement, depuis la page *Réconciliation*.
 *
 * Avant la story, un règlement par virement saisi depuis la fiche laissait la transaction du
 * relevé en attente à jamais : la rejeter ne la rapprochait pas, et elle empêchait la clôture du
 * compte bancaire. C'est le seul test qui voit l'identifiant du règlement choisi traverser la
 * frontière HTTP jusqu'à `…/link` (`CLAUDE.md`, § *E2E*).
 *
 * Parcours : facture validée, règlement par virement **depuis la fiche** (par l'API), relevé
 * importé dont la transaction dépasse le règlement de 3 centimes (le cas de la 29-6) ;
 * *Réconciliation* → *Rattacher à un règlement* → aucun choix présélectionné, *Rattacher* grisé →
 * choix du règlement → l'avertissement d'écart → *Rattacher* → la ligne quitte la liste,
 * « 1 opération(s) réussie(s). » ; au détail de l'import, la transaction est rapprochée et
 * pointe une écriture.
 *
 * ⚠️ **Relevé et montants UNIQUES à chaque exécution** (patron de
 * `reconciliation-amount-due.spec.ts`) : références, montant du règlement (100.00 à 399.99) et
 * contrepartie réécrits. En `finally`, le rapprochement est annulé (ce qui annule aussi le
 * règlement, AC3) puis l'import supprimé.
 *
 * ⚠️ **Sélecteurs par `data-testid`** (garde KF-043, #326) ; les textes se vérifient par
 * `toContainText` sur un élément ciblé.
 *
 * # Recette visuelle
 *
 * Une fois par schéma de couleurs (`page.emulateMedia`), une capture de la modale avec le
 * candidat à écart choisi et son avertissement, dans `KESH_E2E_CAPTURES_DIR` s'il est posé,
 * sinon dans le dossier de sortie du test. Hors dépôt, sans `toHaveScreenshot`.
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
import {
	createContactWithAddressViaApi,
	createAndValidateInvoiceViaApi,
	wiredBankAccountViaApi,
} from './helpers/api-fixtures';

const FIXTURE = path.join(
	path.dirname(fileURLToPath(import.meta.url)),
	'fixtures',
	'camt053_v04_minimal.xml',
);
const TEST_IBAN = 'CH4431999123000889012';

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

/**
 * Le décor : une facture réglée par virement depuis la fiche, et un relevé dont la
 * transaction dépasse ce règlement de 3 centimes.
 */
async function settledFromTheInvoice(
	page: Page,
	tag: string,
): Promise<{
	bankAccountId: number;
	importId: number;
	txId: number;
	settlementId: number;
	journalEntryId: number;
	settled: string;
}> {
	const unique = `${Date.now()}-${tag}`;
	const contactName = `Rattache ${unique}`;
	const contactId = await createContactWithAddressViaApi(page, contactName);
	const invoiceId = await createAndValidateInvoiceViaApi(page, contactId, '2026-05-10');
	const ctx = await authedApiContext(page);
	try {
		const bankAccountId = await wiredBankAccountViaApi(page, TEST_IBAN);
		const cents = 10000 + (Date.now() % 30000);
		const settled = (cents / 100).toFixed(2);
		const res = await ctx.post(`/api/v1/invoices/${invoiceId}/settlements`, {
			data: {
				settlementType: 'bank_transfer',
				bankAccountId,
				amount: settled,
				settledOn: '2026-05-16',
			},
		});
		expect(res.ok(), `règlement : ${res.status()} ${await res.text()}`).toBeTruthy();
		const journalEntryId = ((await res.json()) as { journalEntryId: number }).journalEntryId;
		const list = (await (await ctx.get(`/api/v1/invoices/${invoiceId}/settlements`)).json()) as Array<{
			id: number;
			journalEntryId: number;
		}>;
		const settlementId = list.find((s) => s.journalEntryId === journalEntryId)!.id;

		let xml = fs.readFileSync(FIXTURE, 'utf8');
		for (const [from, to] of [
			['STMT-2026-05-001', `STMT-LINK298-${unique}`],
			['BANK-TX-42', `BANK-TX-LINK298-${unique}`],
			['E2E-2026-05-001', `E2E-LINK298-${unique}`],
			['<Amt Ccy="CHF">1234.56</Amt>', `<Amt Ccy="CHF">${((cents + 3) / 100).toFixed(2)}</Amt>`],
			['<Nm>Acme SA</Nm>', `<Nm>${contactName}</Nm>`],
		]) {
			expect(xml.includes(from), `fixture : « ${from} » introuvable`).toBeTruthy();
			xml = xml.replace(from, to);
		}
		const imported = await ctx.post('/api/v1/bank-imports', {
			multipart: {
				bankAccountId: bankAccountId.toString(),
				file: {
					name: `link-${unique}.xml`,
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
			transactions: Array<{ id: number }>;
		};
		return {
			bankAccountId,
			importId,
			txId: detail.transactions[0].id,
			settlementId,
			journalEntryId,
			settled,
		};
	} finally {
		await disposeContextSafe(ctx);
	}
}

/** Le dossier des captures : `KESH_E2E_CAPTURES_DIR`, sinon la sortie du test. */
function capturePath(testInfo: TestInfo, name: string): string {
	const dossier = process.env.KESH_E2E_CAPTURES_DIR;
	return dossier ? path.join(dossier, `${name}.png`) : testInfo.outputPath(`${name}.png`);
}

for (const [colorScheme, suffixe] of [
	['light', 'clair'],
	['dark', 'sombre'],
] as const) {
	test(`(${suffixe}) — une transaction se rattache au règlement saisi depuis la fiche`, async ({
		page,
	}, testInfo) => {
		await page.emulateMedia({ colorScheme });
		await login(page);
		const d = await settledFromTheInvoice(page, suffixe);
		const ctx = await authedApiContext(page);
		try {
			await page.goto('/reconciliation');
			await page.getByTestId('bank-account-select').selectOption(d.bankAccountId.toString());
			const ligne = page.locator(`[data-testid="reconciliation-row"][data-tx-id="${d.txId}"]`);
			await expect(ligne).toBeVisible({ timeout: 10000 });

			await ligne.getByTestId('link-settlement-button').click();
			const modal = page.getByTestId('link-settlement-modal');
			await expect(modal.getByTestId('link-no-entry')).toContainText('aucune écriture');
			const choix = modal.locator(
				`[data-testid="link-candidate-radio"][data-settlement-id="${d.settlementId}"]`,
			);
			await expect(choix).toBeVisible({ timeout: 10000 });
			await expect(choix).not.toBeChecked();
			await expect(modal.getByTestId('link-submit')).toBeDisabled();

			await choix.check();
			await expect(modal.getByTestId('link-gap-warning')).toContainText('0.03 CHF');
			await expect(modal.getByTestId('link-submit')).toBeEnabled();
			await modal.screenshot({ path: capturePath(testInfo, `29-8-modale-ecart-${suffixe}`) });

			await modal.getByTestId('link-submit').click();
			await expect(modal).toBeHidden();
			await expect(page.getByTestId('reconciliation-success')).toHaveText(
				/^\s*1\s+opération\(s\) réussie\(s\)\.\s*$/,
			);
			await expect(ligne).toHaveCount(0);

			// Le rattachement a traversé la frontière HTTP : le détail de l'import le montre.
			await page.goto(`/bank-import/${d.importId}`);
			const row = page.locator(`[data-testid="detail-tx-row"][data-tx-id="${d.txId}"]`);
			await expect(row).toContainText('reconciled');
			// Le lien pointe l'écriture DU RÈGLEMENT, pas une autre (revue de code P1, A3).
			await expect(row.getByTestId('detail-tx-entry-link')).toHaveAttribute(
				'href',
				`/journal-entries/${d.journalEntryId}`,
			);
			const tx = (await (
				await ctx.get(`/api/v1/reconciliation/transactions/${d.txId}`)
			).json()) as { kind: string; matchedEntryId: number | null };
			expect(tx.kind).toBe('invoice_settlement');
			expect(tx.matchedEntryId).toBe(d.journalEntryId);
		} finally {
			// Annuler le rapprochement annule aussi le règlement (AC3), puis l'import se supprime.
			await ctx.post(`/api/v1/reconciliation/transactions/${d.txId}/cancel`, { data: {} });
			await ctx.delete(`/api/v1/bank-imports/${d.importId}`);
			await disposeContextSafe(ctx);
		}
	});
}
