/**
 * Story v017-3a (#671, AC-17) — un débit déjà comptabilisé par une écriture saisie au journal se
 * rattache à cette écriture, depuis la page *Réconciliation*.
 *
 * Avant la story, une dépense passée au journal laissait la transaction du relevé en attente à
 * jamais : aucun geste ne lisait l'écriture existante, la rejeter ne la rapprochait pas, et
 * l'affecter passait une seconde écriture. C'est le seul test qui voit l'identifiant de
 * l'écriture choisie traverser la frontière HTTP jusqu'à `…/link-entry` (`CLAUDE.md`, § *E2E*).
 *
 * Parcours : un Comptable saisit par l'API une écriture `D charge / C banque` d'un montant
 * unique ; il importe un relevé dont un **débit** l'égale (patron de
 * `reconciliation-supplier.spec.ts` : montant **et** `<CdtDbtInd>` réécrits, import multipart) ;
 * *Réconciliation* → *Rattacher à une écriture existante* → la section des écritures, rien de
 * présélectionné, *Rattacher* grisé → choix de l'écriture → la phrase datée de l'annulation →
 * *Rattacher* → la ligne quitte la liste ; le détail de l'import montre la transaction
 * rapprochée de cette écriture ; `/audit-log`, filtré sur l'action « Transaction rattachée à une
 * écriture », la retrouve.
 *
 * ⚠️ **Relevé et montants UNIQUES à chaque exécution** (base partagée entre specs) ; fixture
 * CAMT **synthétique**. En `finally` : rapprochement annulé (l'écriture saisie est
 * contre-passée), import supprimé.
 *
 * # Recette visuelle
 *
 * Captures du dialogue (section des écritures, choix fait), dans `KESH_E2E_CAPTURES_DIR` s'il est
 * posé, sinon dans le dossier de sortie du test. **Schéma clair seulement** (Kesh n'a pas de
 * thème sombre). Hors dépôt, sans `toHaveScreenshot`.
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
import { wiredBankAccountViaApi } from './helpers/api-fixtures';

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

/** Le dossier des captures : `KESH_E2E_CAPTURES_DIR`, sinon la sortie du test. */
function capturePath(testInfo: TestInfo, name: string): string {
	const dossier = process.env.KESH_E2E_CAPTURES_DIR;
	return dossier ? path.join(dossier, `${name}.png`) : testInfo.outputPath(`${name}.png`);
}

type Decor = {
	bankAccountId: number;
	importId: number;
	txId: number;
	journalEntryId: number;
	amount: string;
};

/** L'écriture de la dépense, saisie au journal, et le relevé dont le débit l'égale. */
async function decor(page: Page): Promise<Decor> {
	const unique = `${Date.now()}${Math.floor(Math.random() * 1000)}`;
	const cents = 10000 + (Date.now() % 30000);
	const amount = (cents / 100).toFixed(2);
	const bankAccountId = await wiredBankAccountViaApi(page, TEST_IBAN);
	const ctx = await authedApiContext(page);
	try {
		const ba = (
			(await (await ctx.get('/api/v1/bank-accounts')).json()) as Array<{
				id: number;
				journalAccountId: number | null;
			}>
		).find((b) => b.id === bankAccountId);
		expect(ba?.journalAccountId, 'compte bancaire relié au grand livre').toBeTruthy();
		const accounts = (await (await ctx.get('/api/v1/accounts')).json()) as Array<{
			id: number;
			accountType: string;
			active: boolean;
			postable: boolean;
		}>;
		const charge = accounts.find((a) => a.accountType === 'Expense' && a.active && a.postable);
		expect(charge, 'un compte de charge imputable').toBeTruthy();
		const entry = await ctx.post('/api/v1/journal-entries', {
			data: {
				entryDate: '2026-05-14',
				journal: 'Banque',
				description: `Dépense déjà payée ${unique}`,
				lines: [
					{ accountId: charge!.id, debit: amount, credit: '0' },
					{ accountId: ba!.journalAccountId, debit: '0', credit: amount },
				],
			},
		});
		expect(entry.status(), await entry.text()).toBe(201);
		const journalEntryId = ((await entry.json()) as { id: number }).id;

		let xml = fs.readFileSync(FIXTURE, 'utf8');
		const remplacements: [string, string][] = [
			['STMT-2026-05-001', `STMT-V0173A-${unique}`],
			['BANK-TX-42', `BANK-TX-V0173A-${unique}`],
			['E2E-2026-05-001', `E2E-V0173A-${unique}`],
			[
				'<Amt Ccy="CHF">1234.56</Amt>\n        <CdtDbtInd>CRDT</CdtDbtInd>',
				`<Amt Ccy="CHF">${amount}</Amt>\n        <CdtDbtInd>DBIT</CdtDbtInd>`,
			],
			['<Nm>Acme SA</Nm>', `<Nm>Fournisseur V0173A ${unique}</Nm>`],
			['<Dbtr>', '<Cdtr>'],
			['</Dbtr>', '</Cdtr>'],
			['<DbtrAcct>', '<CdtrAcct>'],
			['</DbtrAcct>', '</CdtrAcct>'],
		];
		for (const [from, to] of remplacements) {
			expect(xml.includes(from), `fixture : « ${from} » introuvable`).toBeTruthy();
			xml = xml.replace(from, to);
		}
		const imported = await ctx.post('/api/v1/bank-imports', {
			multipart: {
				bankAccountId: bankAccountId.toString(),
				file: {
					name: `ecriture-${unique}.xml`,
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
		expect(Number(detail.transactions[0].amount)).toBe(-Number(amount));
		return { bankAccountId, importId, txId: detail.transactions[0].id, journalEntryId, amount };
	} finally {
		await disposeContextSafe(ctx);
	}
}

test('un débit déjà comptabilisé au journal se rattache à son écriture (#671)', async ({
	page,
}, testInfo) => {
	await login(page);
	const d = await decor(page);
	const ctx = await authedApiContext(page);
	try {
		await page.goto('/reconciliation');
		await page.getByTestId('bank-account-select').selectOption(d.bankAccountId.toString());
		const ligne = page.locator(`[data-testid="reconciliation-row"][data-tx-id="${d.txId}"]`);
		await expect(ligne).toBeVisible({ timeout: 10000 });

		const bouton = ligne.getByTestId('link-settlement-button');
		await expect(bouton).toHaveText('Rattacher à une écriture existante');
		await bouton.click();
		const modal = page.getByTestId('link-settlement-modal');
		await expect(modal.getByTestId('link-no-entry')).toContainText('aucune écriture');
		// Un débit : la seule section des écritures.
		await expect(modal.getByTestId('link-settlements-section')).toHaveCount(0);
		const choix = modal.locator(
			`[data-testid="entry-candidate-radio"][data-entry-id="${d.journalEntryId}"]`,
		);
		await expect(choix).toBeVisible({ timeout: 10000 });
		await expect(choix).not.toBeChecked();
		await expect(modal.getByTestId('link-submit')).toBeDisabled();

		await choix.check();
		await expect(modal.getByTestId('entry-cancel-today')).toContainText(
			"Aujourd'hui, Kesh ne voit aucun obstacle",
		);
		await expect(modal.getByTestId('link-submit')).toBeEnabled();
		await modal.screenshot({ path: capturePath(testInfo, 'v017-3a-dialogue-debit-choisi') });

		await modal.getByTestId('link-submit').click();
		await expect(modal).toBeHidden();
		await expect(page.getByTestId('reconciliation-success')).toHaveText(
			/^\s*1\s+opération\(s\) réussie\(s\)\.\s*$/,
		);
		await expect(ligne).toHaveCount(0);

		// Le rattachement a traversé la frontière HTTP : le détail de l'import le montre,
		// pointant l'écriture CHOISIE.
		await page.goto(`/bank-import/${d.importId}`);
		const row = page.locator(`[data-testid="detail-tx-row"][data-tx-id="${d.txId}"]`);
		await expect(row).toContainText('reconciled');
		await expect(row.getByTestId('detail-tx-entry-link')).toHaveAttribute(
			'href',
			`/journal-entries/${d.journalEntryId}`,
		);

		// Le journal d'audit, filtré sur l'action, la retrouve.
		await page.goto(
			`/audit-log?entityType=bank_transaction&entityId=${d.txId}&action=reconciliation.entry_linked`,
		);
		const filtre = page.getByTestId('audit-log-filter-action');
		await expect(filtre).toHaveValue('reconciliation.entry_linked');
		await expect(filtre.locator('option:checked')).toHaveText(
			'Transaction rattachée à une écriture',
		);
		const trace = page.locator(
			'[data-testid="audit-log-row"][data-action="reconciliation.entry_linked"]',
		);
		await expect(trace).toHaveCount(1);
		await expect(trace.getByTestId('audit-log-row-action')).toHaveText(
			'Transaction rattachée à une écriture',
		);
	} finally {
		// Annuler le rapprochement contre-passe l'écriture saisie, puis l'import se supprime.
		await ctx.post(`/api/v1/reconciliation/transactions/${d.txId}/cancel`, { data: {} });
		await ctx.delete(`/api/v1/bank-imports/${d.importId}`);
		await disposeContextSafe(ctx);
	}
});
