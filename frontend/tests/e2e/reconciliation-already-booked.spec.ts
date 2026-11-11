/**
 * Story 34-4 (#828, AC-10) — une transaction qui semble déjà comptabilisée est signalée sur la
 * page *Réconciliation*, et *Écriture manuelle* demande confirmation avant d'ouvrir son dialogue.
 *
 * C'est le seul test qui voit le champ `alreadyBooked` traverser la frontière HTTP jusqu'à
 * l'écran (`CLAUDE.md`, § *E2E*) : Vitest teste le texte à partir d'une réponse simulée, les
 * tests Rust le calcul du champ, et ni l'un ni l'autre ne voit une clé qui disparaîtrait entre
 * les deux.
 *
 * Parcours : un Comptable saisit par l'API une écriture `D charge / C banque` d'un montant
 * unique, puis un relevé dont un **débit** l'égale (`createManualDebitStatementViaApi`) ;
 * *Réconciliation* : la ligne porte l'avertissement (a) ; *Écriture manuelle* → la confirmation
 * nomme la transaction → *Annuler* → aucun dialogue d'écriture ; *Rattacher à une écriture
 * existante* → l'écriture → la ligne quitte la liste. Le second côté d'un virement (b) est
 * couvert par les tests HTTP (AC-2) et Vitest (AC-5) ; un E2E à deux comptes bancaires n'est pas
 * exigé (choix C-e342-8).
 *
 * ⚠️ Montant UNIQUE à chaque exécution (base partagée entre specs). En `finally` : rattachement
 * annulé (l'écriture saisie est contre-passée), import supprimé.
 *
 * # Recette visuelle
 *
 * Captures de la ligne signalée et de la confirmation, dans `KESH_E2E_CAPTURES_DIR` s'il est
 * posé, sinon dans le dossier de sortie du test. Schéma clair seulement (Kesh n'a pas de thème
 * sombre).
 *
 * Pré-requis : MariaDB up + KESH_TEST_MODE=true (cf. docs/testing.md).
 */

import { expect, test, type Page, type TestInfo } from '@playwright/test';
import path from 'path';
import {
	seedTestState,
	clearAuthStorage,
	authedApiContext,
	disposeContextSafe,
} from './helpers/test-state';
import {
	createManualDebitStatementViaApi,
	todayInZurich,
	wiredBankAccountViaApi,
} from './helpers/api-fixtures';

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

test('une transaction déjà comptabilisée est signalée, et Écriture manuelle demande confirmation (#828)', async ({
	page,
}, testInfo) => {
	await login(page);
	const cents = 60000 + (Date.now() % 30000);
	const amount = (cents / 100).toFixed(2);
	const unique = `${Date.now()}${Math.floor(Math.random() * 1000)}`;
	const today = todayInZurich();
	const bankAccountId = await wiredBankAccountViaApi(page, TEST_IBAN);
	const ctx = await authedApiContext(page);
	let importId: number | null = null;
	let txId: number | null = null;
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
				entryDate: today,
				journal: 'Banque',
				description: `Dépense déjà payée 34-4 ${unique}`,
				lines: [
					{ accountId: charge!.id, debit: amount, credit: '0' },
					{ accountId: ba!.journalAccountId, debit: '0', credit: amount },
				],
			},
		});
		expect(entry.status(), await entry.text()).toBe(201);
		const journalEntryId = ((await entry.json()) as { id: number }).id;

		const counterparty = `Fournisseur 34-4 ${unique}`;
		importId = await createManualDebitStatementViaApi(page, {
			bankAccountId,
			bookingDate: today,
			amount,
			counterpartyName: counterparty,
			details: `Paiement déjà comptabilisé ${unique}`,
		});
		const detail = (await (await ctx.get(`/api/v1/bank-imports/${importId}`)).json()) as {
			transactions: Array<{ id: number }>;
		};
		expect(detail.transactions).toHaveLength(1);
		txId = detail.transactions[0].id;

		await page.goto('/reconciliation');
		await page.getByTestId('bank-account-select').selectOption(bankAccountId.toString());
		const ligne = page.locator(`[data-testid="reconciliation-row"][data-tx-id="${txId}"]`);
		await expect(ligne).toBeVisible({ timeout: 10000 });

		// (a) — l'avertissement, sous la ligne, au singulier : une écriture lui correspond.
		const avertissement = page.locator(
			`[data-testid="already-booked-warning"][data-tx-id="${txId}"]`,
		);
		await expect(avertissement).toHaveText(
			'Cette transaction semble déjà comptabilisée : une écriture non rapprochée du compte de la banque lui correspond. Rattachez-la (« Rattacher à une écriture existante ») plutôt que de passer une écriture : elle serait comptée deux fois.',
		);
		await page
			.locator('tbody')
			.screenshot({ path: capturePath(testInfo, 'e344-ligne-signalee') });

		// Écriture manuelle → la confirmation nomme la transaction → Annuler : aucun dialogue.
		await ligne.getByTestId('manual-match-button').click();
		const confirmation = page.getByTestId('already-booked-confirm');
		await expect(confirmation).toBeVisible();
		const item = confirmation.getByTestId('already-booked-confirm-item');
		await expect(item).toHaveCount(1);
		// Le montant tel que le serveur le rend (`normalize` : « -612.3 », « -700 »).
		await expect(item).toContainText(`${-Number(amount)} CHF`);
		await expect(item).toContainText(counterparty);
		await confirmation.screenshot({
			path: capturePath(testInfo, 'e344-confirmation-ecriture-manuelle'),
		});
		await confirmation.getByTestId('already-booked-confirm-cancel').click();
		await expect(confirmation).toBeHidden();
		await expect(page.getByTestId('manual-match-modal')).toHaveCount(0);

		// Rattacher à une écriture existante, sans confirmation → l'écriture → la ligne part.
		await ligne.getByTestId('link-settlement-button').click();
		await expect(page.getByTestId('already-booked-confirm')).toHaveCount(0);
		const modal = page.getByTestId('link-settlement-modal');
		const choix = modal.locator(
			`[data-testid="entry-candidate-radio"][data-entry-id="${journalEntryId}"]`,
		);
		await expect(choix).toBeVisible({ timeout: 10000 });
		await choix.check();
		await modal.getByTestId('link-submit').click();
		await expect(modal).toBeHidden();
		await expect(ligne).toHaveCount(0);
		await expect(avertissement).toHaveCount(0);
	} finally {
		if (txId !== null) {
			await ctx.post(`/api/v1/reconciliation/transactions/${txId}/cancel`, { data: {} });
		}
		if (importId !== null) await ctx.delete(`/api/v1/bank-imports/${importId}`);
		await disposeContextSafe(ctx);
	}
});
