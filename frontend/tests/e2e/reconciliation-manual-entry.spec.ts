/**
 * Story 34-3 (#967, AC-7) — une écriture manuelle de bout en bout, sur un compte de classe 2.
 *
 * Avant la story, le dialogue ne proposait que les comptes des classes 5, 6 et 7 : un virement
 * vers un compte d'épargne, un apport privé ou un remboursement d'emprunt ne se passait pas
 * depuis la *Réconciliation*. C'est le seul test qui voit l'identifiant du compte choisi
 * traverser la frontière HTTP jusqu'à `POST /reconciliation/manual` et revenir dans l'écriture
 * (`CLAUDE.md`, § *E2E*). Il rougit si le filtre de classes revient (mutation M1).
 *
 * Parcours : un relevé saisi par l'API porte un débit d'un montant unique ; *Réconciliation* →
 * *Écriture manuelle* sur sa ligne → le dialogue ne propose pas le compte de la banque (`1000
 * Caisse CI`, relié par `wiredBankAccountViaApi`) → choix de **`2000 Capital CI`** (preset
 * `with-company`, `crates/kesh-db/src/test_fixtures.rs`) → *Passer l'écriture* → la ligne quitte
 * la liste ; l'écriture existe, au débit de 2000, au montant, lue par l'API.
 *
 * Le preset ne pose aucun rôle de compte : l'avis débiteurs / créanciers (AC-4) n'est pas exercé
 * ici, mais par Vitest (`ManualMatchModal.test.ts`, `TransactionSplitModal.test.ts`).
 *
 * ⚠️ Montant UNIQUE à chaque exécution (base partagée entre specs). En `finally` : rapprochement
 * annulé (l'écriture est contre-passée), import supprimé.
 *
 * # Recette visuelle
 *
 * Capture du dialogue, compte choisi, dans `KESH_E2E_CAPTURES_DIR` s'il est posé, sinon dans le
 * dossier de sortie du test. Schéma clair seulement (Kesh n'a pas de thème sombre).
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

type Compte = { id: number; number: string };

test('une écriture manuelle se passe sur un compte de classe 2, jamais sur celui de la banque (#967)', async ({
	page,
}, testInfo) => {
	await login(page);
	const cents = 20000 + (Date.now() % 30000);
	const amount = (cents / 100).toFixed(2);
	const unique = `${Date.now()}${Math.floor(Math.random() * 1000)}`;
	const bankAccountId = await wiredBankAccountViaApi(page, TEST_IBAN);
	const ctx = await authedApiContext(page);
	let importId: number | null = null;
	let txId: number | null = null;
	try {
		const accounts = (await (await ctx.get('/api/v1/accounts')).json()) as Compte[];
		const capital = accounts.find((a) => a.number === '2000');
		expect(capital, 'compte 2000 Capital CI du préréglage').toBeTruthy();
		const ba = (
			(await (await ctx.get('/api/v1/bank-accounts')).json()) as Array<{
				id: number;
				journalAccountId: number | null;
			}>
		).find((b) => b.id === bankAccountId);
		const banque = accounts.find((a) => a.id === ba?.journalAccountId);
		expect(banque, 'compte de la banque relié au grand livre').toBeTruthy();

		importId = await createManualDebitStatementViaApi(page, {
			bankAccountId,
			bookingDate: todayInZurich(),
			amount,
			counterpartyName: `Épargne 34-3 ${unique}`,
			details: `Virement vers l'épargne ${unique}`,
		});
		const detail = (await (await ctx.get(`/api/v1/bank-imports/${importId}`)).json()) as {
			transactions: Array<{ id: number; amount: string }>;
		};
		expect(detail.transactions).toHaveLength(1);
		txId = detail.transactions[0].id;

		await page.goto('/reconciliation');
		await page.getByTestId('bank-account-select').selectOption(bankAccountId.toString());
		const ligne = page.locator(`[data-testid="reconciliation-row"][data-tx-id="${txId}"]`);
		await expect(ligne).toBeVisible({ timeout: 10000 });

		const bouton = ligne.getByTestId('manual-match-button');
		await expect(bouton).toHaveText('Écriture manuelle');
		await bouton.click();
		const modal = page.getByTestId('manual-match-modal');
		await expect(modal).toBeVisible();

		// Le dialogue ne propose pas le compte de la banque, et propose le compte de classe 2.
		const champ = modal.locator('input[aria-autocomplete="list"]');
		await champ.focus();
		// Revue de code P1 (B1-3, E1-4, A1-1) : ancre positive d'abord — la liste est chargée et
		// montre les voisins de classe 1 (`1100`), sans le compte de la banque ; l'absence qui
		// suit ne peut donc pas passer à vide.
		await champ.fill(banque!.number.slice(0, 1));
		const voisins = modal.getByRole('option');
		await expect(voisins.filter({ hasText: '1100' })).toHaveCount(1);
		await expect(voisins.filter({ hasText: banque!.number })).toHaveCount(0);
		await champ.fill(banque!.number);
		await expect(modal.getByRole('option')).toHaveCount(0);
		await champ.fill('2000');
		const option = modal.getByRole('option').filter({ hasText: '2000' });
		await expect(option).toHaveCount(1);
		await option.dispatchEvent('mousedown');
		await modal.getByTestId('manual-match-description-input').fill(`Épargne ${unique}`);
		const valider = modal.getByTestId('manual-match-submit');
		await expect(valider).toHaveText("Passer l'écriture");
		await expect(valider).toBeEnabled();
		await modal.screenshot({ path: capturePath(testInfo, 'e3403-ecriture-manuelle-classe-2') });

		await valider.click();
		await expect(modal).toBeHidden();
		await expect(ligne).toHaveCount(0);

		// L'écriture a traversé la frontière HTTP : la transaction pointe une écriture, au débit
		// de 2000 et au crédit de la banque, au montant.
		const apres = (await (await ctx.get(`/api/v1/bank-imports/${importId}`)).json()) as {
			transactions: Array<{ id: number; matchedEntryId: number | null }>;
		};
		const entryId = apres.transactions.find((t) => t.id === txId)?.matchedEntryId;
		expect(entryId, 'la transaction est rapprochée d’une écriture').toBeTruthy();
		const entry = (await (await ctx.get(`/api/v1/journal-entries/${entryId}`)).json()) as {
			lines: Array<{ accountId: number; debit: string; credit: string }>;
		};
		const surCapital = entry.lines.filter((l) => l.accountId === capital!.id);
		expect(surCapital).toHaveLength(1);
		expect(Number(surCapital[0].debit)).toBe(Number(amount));
		const surBanque = entry.lines.filter((l) => l.accountId === banque!.id);
		expect(surBanque).toHaveLength(1);
		expect(Number(surBanque[0].credit)).toBe(Number(amount));
	} finally {
		if (txId !== null) {
			await ctx.post(`/api/v1/reconciliation/transactions/${txId}/cancel`, { data: {} });
		}
		if (importId !== null) await ctx.delete(`/api/v1/bank-imports/${importId}`);
		await disposeContextSafe(ctx);
	}
});
