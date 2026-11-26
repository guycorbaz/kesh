/**
 * Aide commune des specs des rapports par groupes — bilan (Stories 46-2 et 46-3)
 * et balance (Story 46-6b) : connexion,
 * création d'un arbre de comptes `19` > `190` > `1900`, `1910` et d'une écriture
 * qui le mouvemente (1250.00 contre le compte 3000 du preset `with-company`).
 */

import { expect, type APIRequestContext, type Page } from '@playwright/test';
import { authedApiContext, disposeContextSafe } from './test-state';
import { todayInZurich } from './api-fixtures';

export async function login(page: Page): Promise<void> {
	await page.goto('/login');
	await page.fill('#username', 'admin');
	await page.fill('#password', 'admin123');
	await page.click('button[type="submit"]');
	await expect(page).toHaveURL('/');
}

export async function creerCompte(
	api: APIRequestContext,
	data: { number: string; name: string; parentId?: number; postable?: boolean },
): Promise<number> {
	const res = await api.post('/api/v1/accounts', { data: { accountType: 'Asset', ...data } });
	expect(res.ok(), `compte ${data.number} : ${res.status()} ${await res.text()}`).toBeTruthy();
	return (await res.json()).id as number;
}

/** L'arbre 19 > 190 > {1900, 1910}, mouvementé contre 3000 ; rend l'id de 19. */
export async function preparerArbre(page: Page): Promise<number> {
	const api = await authedApiContext(page);
	try {
		const g19 = await creerCompte(api, { number: '19', name: 'Actifs du test', postable: false });
		const g190 = await creerCompte(api, {
			number: '190',
			name: 'Liquidités du test',
			parentId: g19,
			postable: false,
		});
		const banque = await creerCompte(api, { number: '1900', name: 'Banque du test', parentId: g190 });
		const caisse = await creerCompte(api, { number: '1910', name: 'Caisse du test', parentId: g190 });
		const acc = await api.get('/api/v1/accounts?includeArchived=false');
		const ventes = ((await acc.json()) as Array<{ id: number; number: string }>).find(
			(a) => a.number === '3000',
		);
		expect(ventes, 'compte 3000 du preset').toBeTruthy();
		const entry = await api.post('/api/v1/journal-entries', {
			data: {
				entryDate: todayInZurich(),
				journal: 'OD',
				description: 'Ventes du test des groupes',
				lines: [
					{ accountId: banque, debit: '1000.00', credit: '0.00' },
					{ accountId: caisse, debit: '250.00', credit: '0.00' },
					{ accountId: ventes!.id, debit: '0.00', credit: '1250.00' },
				],
			},
		});
		expect(entry.ok(), `écriture : ${entry.status()} ${await entry.text()}`).toBeTruthy();
		return g19;
	} finally {
		await disposeContextSafe(api);
	}
}

