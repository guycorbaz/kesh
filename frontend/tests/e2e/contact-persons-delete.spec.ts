/**
 * Story 29-9 (#646, AC9) — la suppression d'une personne de contact demande confirmation.
 *
 * « Supprimer » archive la personne, et aucun écran ne la rétablit : avant la story, un clic
 * malheureux l'effaçait aussitôt. Le dialogue est le `window.confirm` natif, invisible des
 * captures (aucune capture, C-29-9-10). Le refus laisse la personne ; l'acceptation la retire,
 * et `GET /api/v1/contacts/{id}/persons` ne la rend plus — la frontière HTTP traversée.
 *
 * Noms uniques par exécution ; le contact est archivé en `finally`.
 *
 * Pré-requis : MariaDB up + KESH_TEST_MODE=true (cf. docs/testing.md).
 */

import { expect, test, type Page } from '@playwright/test';
import {
	seedTestState,
	clearAuthStorage,
	authedApiContext,
	disposeContextSafe,
} from './helpers/test-state';

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

test('supprimer une personne de contact : refus ⇒ elle reste ; acceptation ⇒ elle part', async ({ page }) => {
	await login(page);
	const ctx = await authedApiContext(page);
	const stamp = Date.now();
	const name = `Entreprise Personnes ${stamp}`;
	const lastName = `Muster${stamp}`;
	let contact: { id: number; version: number } | null = null;
	try {
		const created = await ctx.post('/api/v1/contacts', {
			data: { contactType: 'Entreprise', name, isClient: true, isSupplier: false },
		});
		expect(created.ok(), `create contact: ${created.status()}`).toBeTruthy();
		contact = (await created.json()) as { id: number; version: number };
		const person = await ctx.post(`/api/v1/contacts/${contact.id}/persons`, {
			data: { firstName: 'Anna', lastName },
		});
		expect(person.ok(), `create person: ${person.status()}`).toBeTruthy();
		const personId = ((await person.json()) as { id: number }).id;

		await page.goto('/contacts');
		const row = page.locator('tr', { hasText: name }).first();
		await row.getByRole('button', { name: /Modifier/ }).click();
		const manager = page.getByTestId('contact-persons-manager');
		await expect(manager.getByText(`Anna ${lastName}`)).toBeVisible();
		const bouton = manager.getByTestId(`contact-person-delete-${personId}`);

		// Refus : rien n'est envoyé, la personne reste.
		let message = '';
		page.once('dialog', (d) => {
			message = d.message();
			void d.dismiss();
		});
		await bouton.click();
		await expect.poll(() => message).toContain(`Anna ${lastName}`);
		await expect(manager.getByText(`Anna ${lastName}`)).toBeVisible();
		const avant = await ctx.get(`/api/v1/contacts/${contact.id}/persons`);
		expect(((await avant.json()) as Array<{ id: number }>).map((p) => p.id)).toContain(personId);

		// Acceptation : la personne part, et le serveur ne la rend plus.
		page.once('dialog', (d) => void d.accept());
		await bouton.click();
		await expect(manager.getByText(`Anna ${lastName}`)).toHaveCount(0);
		const apres = await ctx.get(`/api/v1/contacts/${contact.id}/persons`);
		expect(((await apres.json()) as Array<{ id: number }>).map((p) => p.id)).not.toContain(personId);
	} finally {
		if (contact !== null) {
			await ctx.put(`/api/v1/contacts/${contact.id}/archive`, { data: { version: contact.version } });
		}
		await disposeContextSafe(ctx);
	}
});
