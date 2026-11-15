import { test, expect } from '@playwright/test';
import {
	seedTestState,
	authedApiContext,
	disposeContextSafe,
	clearAuthStorage,
} from './helpers/test-state';
import { todayInZurich } from './helpers/api-fixtures';

/**
 * Sonde d'horloge — Story 36-7a (#1125, D6, C-36-7-8) : le serveur, le runner et le
 * navigateur disent-ils le même jour ?
 *
 * Trois lectures du jour suisse, chacune par sa propre horloge :
 *
 * - **serveur** : `asOf` de `GET /api/v1/accounts/{id}/open-items` envoyé **sans** `asOf`
 *   (« absent (ou vide) → aujourd'hui », `routes/letterings.rs`), sur la créance `1100` du
 *   preset `with-company`, compte lettrable (la route refuse un compte qui ne l'est pas) ;
 * - **runner** : `todayInZurich()` (`helpers/api-fixtures.ts`), l'horloge du processus Node ;
 * - **navigateur** : la même lecture, faite dans la page (`page.evaluate`), au fuseau de
 *   Zurich (le contexte de Playwright est déjà en `Europe/Zurich`, `playwright.config.ts`).
 *
 * À l'heure réelle, les trois horloges sont celles de la machine : la spec est vraie par
 * construction, et elle tourne avec la suite. **Son usage est ailleurs** : jouée seule sous
 * une horloge de processus décalée (libfaketime, `docs/testing.md` § *Les tests dans une
 * autre année*), elle est la sonde à trois étages de l'E2E en 2027 — si l'un des trois ne
 * suit pas l'horloge décalée, elle rougit et nomme lequel. Elle rougit aussi quand le
 * runner seul est décalé (backend à l'heure réelle) : c'est la preuve de son rouge.
 *
 * Angle mort assumé : à minuit à Zurich, les trois lectures, faites à quelques
 * millisecondes d'intervalle, peuvent tomber de part et d'autre du changement de jour.
 */

test.beforeAll(async () => {
	await seedTestState('with-company');
});

test('serveur, runner et navigateur disent le même jour suisse', async ({ page }) => {
	try {
		await page.goto('/login');
		await page.fill('#username', 'admin');
		await page.fill('#password', 'admin123');
		await page.click('button[type="submit"]');
		await expect(page).toHaveURL('/');

		const api = await authedApiContext(page);
		let serveur: string;
		try {
			const plan = (await (await api.get('/api/v1/accounts')).json()) as Array<{
				id: number;
				number: string;
			}>;
			const creance = plan.find((a) => a.number === '1100');
			expect(creance, 'montage : la créance 1100 du preset with-company').toBeTruthy();
			const res = await api.get(`/api/v1/accounts/${creance!.id}/open-items`);
			expect(res.status(), 'GET open-items sans asOf').toBe(200);
			serveur = ((await res.json()) as { asOf: string }).asOf;
		} finally {
			await disposeContextSafe(api);
		}

		const runner = todayInZurich();
		const navigateur = await page.evaluate(() =>
			new Date().toLocaleDateString('sv-SE', { timeZone: 'Europe/Zurich' }),
		);

		expect({ serveur, runner, navigateur }, 'les trois jours suisses').toEqual({
			serveur: runner,
			runner,
			navigateur: runner,
		});
	} finally {
		await clearAuthStorage(page);
	}
});
