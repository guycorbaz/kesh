import { expect, test } from '@playwright/test';
import { seedTestState, clearAuthStorage, authedApiContext, disposeContextSafe } from './helpers/test-state';

/**
 * Tests E2E — compte de produit par ligne de facture (Story 16-1b, AC15).
 *
 * # Ce que ce test prouve, et que rien d'autre ne prouve
 *
 * Les tests unitaires Vitest vérifient que `InvoiceForm` **construit** le bon
 * `revenueAccountId` ; les tests Rust de 16-1a vérifient que le backend
 * **ventile** correctement. Aucun des deux ne vérifie que la valeur **traverse
 * réellement** la frontière HTTP : un champ oublié dans le payload, un nom de clé
 * divergent entre TS et serde, un `PUT` qui écrase — tout cela passe les deux
 * suites et casse en production.
 *
 * Le scénario pilote donc l'**édition d'un brouillon** plutôt qu'une création
 * complète. C'est délibéré : ce chemin est exactement celui qu'AC9-bis désigne
 * comme le plus risqué (charger des lignes existantes, poser les comptes, sauver),
 * et il évite le `ContactPicker`, dont la mécanique n'a rien à voir avec cette
 * story.
 *
 * ⚠️ Convention Playwright du dépôt : le fichier DOIT s'appeler `*.spec.ts`.
 * Un `*.test.ts` dans `tests/e2e/` est **silencieusement ignoré**.
 */

test.beforeAll(async () => {
	await seedTestState('with-company');
});

test.afterEach(async ({ page }) => {
	await clearAuthStorage(page);
});

async function login(page: import('@playwright/test').Page) {
	await page.goto('/login');
	await page.fill('#username', 'admin');
	await page.fill('#password', 'admin123');
	await page.click('button[type="submit"]');
	await expect(page).toHaveURL('/');
}

function uniqSuffix(): string {
	return `${Date.now()}-${Math.floor(Math.random() * 1e6)}`;
}

test('une facture ventilée sur 2 comptes produit 2 lignes de crédit distinctes', async ({
	page,
}) => {
	await login(page);
	const suffix = uniqSuffix();
	const ctx = await authedApiContext(page);

	let invoiceId: number;
	let honoraires: { id: number; number: string; name: string };
	let marchandises: { id: number; number: string; name: string; version: number };

	try {
		// --- Deux comptes de produit distincts, imputables ---
		const mkAccount = async (number: string, name: string) => {
			const res = await ctx.post('/api/v1/accounts', {
				data: { number, name, accountType: 'Revenue', postable: true, role: null },
			});
			expect(res.ok(), `création compte ${number} échouée: ${res.status()}`).toBeTruthy();
			return (await res.json()) as {
				id: number;
				number: string;
				name: string;
				version: number;
			};
		};
		// Numéros hors du plan comptable seedé (qui occupe 3000/3200/3400/…), et
		// dérivés du SEUL tirage aléatoire : `suffix.slice(-3)` pouvait chevaucher
		// le séparateur `-` et produire `NaN` (numéro de compte « NaN »), ou
		// retomber dans une plage déjà occupée. (Passe 2 de revue, 3 lentilles.)
		const base = 39000 + Math.floor(Math.random() * 900);
		honoraires = await mkAccount(String(base), `Honoraires ${suffix}`);
		marchandises = await mkAccount(String(base + 1), `Marchandises ${suffix}`);

		// --- Contact client ---
		const contactRes = await ctx.post('/api/v1/contacts', {
			data: {
				contactType: 'Entreprise',
				name: `Client ventilation ${suffix}`,
				isClient: true,
				isSupplier: false,
			},
		});
		expect(contactRes.ok(), `création contact échouée: ${contactRes.status()}`).toBeTruthy();
		const contactId = (await contactRes.json()).id as number;

		// --- Brouillon à 2 lignes, SANS compte : l'état que l'UI doit corriger ---
		const today = new Date().toISOString().slice(0, 10);
		const draftRes = await ctx.post('/api/v1/invoices', {
			data: {
				contactId,
				date: today,
				lines: [
					{ description: 'Prestation', quantity: '1', unitPrice: '1000.00', vatRate: '0.00' },
					{ description: 'Marchandise', quantity: '1', unitPrice: '400.00', vatRate: '0.00' },
				],
			},
		});
		expect(draftRes.ok(), `création brouillon échouée: ${draftRes.status()}`).toBeTruthy();
		invoiceId = (await draftRes.json()).id as number;
	} finally {
		await disposeContextSafe(ctx);
	}

	// --- L'UI : poser un compte différent sur chaque ligne ---
	await page.goto(`/invoices/${invoiceId}/edit`);

	// `aria-autocomplete="list"` identifie les seuls champs de compte (les autres
	// inputs de la ligne — description, quantité, prix — sont aussi des textbox).
	const accountFields = page.locator('input[aria-autocomplete="list"]');
	await expect(accountFields).toHaveCount(2);

	const pick = async (index: number, account: { number: string; name: string }) => {
		const field = accountFields.nth(index);
		await field.click();
		await field.fill(account.number);
		// Le dropdown ne propose que les comptes actifs ET imputables.
		await page.getByRole('option').filter({ hasText: account.name }).first().click();
		await expect(field).toHaveValue(`${account.number} — ${account.name}`);
	};

	await pick(0, honoraires!);
	await pick(1, marchandises!);

	await page.getByTestId('create-invoice-button').click();
	// L'enregistrement redirige vers le détail.
	await expect(page).toHaveURL(new RegExp(`/invoices/${invoiceId}$`));

	// --- Le compte est visible sur l'écran de détail (AC6-bis) ---
	const cells = page.getByTestId('invoice-line-revenue-account');
	await expect(cells.nth(0)).toContainText(honoraires!.name);
	await expect(cells.nth(1)).toContainText(marchandises!.name);

	// --- Validation puis inspection de l'écriture réellement générée ---
	const ctx2 = await authedApiContext(page);
	try {
		const persisted = await (await ctx2.get(`/api/v1/invoices/${invoiceId}`)).json();
		// La traversée HTTP : ce que l'UI a envoyé est bien ce que la base porte.
		expect(persisted.lines[0].revenueAccountId).toBe(honoraires!.id);
		expect(persisted.lines[1].revenueAccountId).toBe(marchandises!.id);

		const validateRes = await ctx2.post(`/api/v1/invoices/${invoiceId}/validate`);
		expect(validateRes.ok(), `validation échouée: ${validateRes.status()}`).toBeTruthy();
		const validated = await validateRes.json();

		const entryId = validated.journalEntryId ?? validated.journalEntry?.id;
		expect(entryId, "l'écriture générée doit être référencée").toBeTruthy();
		const entry = await (await ctx2.get(`/api/v1/journal-entries/${entryId}`)).json();

		// LE CŒUR DE LA STORY : deux crédits de produit DISTINCTS, et non un seul
		// crédit agrégé sur le compte par défaut de la société.
		const credits = (entry.lines as { accountId: number; credit: string }[])
			.filter((l) => Number(l.credit) > 0)
			.filter((l) => l.accountId === honoraires!.id || l.accountId === marchandises!.id);

		expect(credits).toHaveLength(2);
		expect(new Set(credits.map((l) => l.accountId)).size).toBe(2);
		expect(
			credits.find((l) => l.accountId === honoraires!.id)?.credit
		).toMatch(/^1000\.0*$/);
		expect(
			credits.find((l) => l.accountId === marchandises!.id)?.credit
		).toMatch(/^400\.0*$/);

		// --- LE MÊME PIÈGE, CÔTÉ AVOIR (passe 2 de revue) ---
		//
		// ⚠️ L'avoir est émis AVANT l'archivage : 16-1a D5-bis refuse l'émission
		// si une ligne référence un compte archivé (comportement arbitré, #280).
		// L'ordre inverse fait échouer le POST — vérifié en conditions réelles.
		//
		// La passe 1 avait fermé ce finding en ne couvrant QU'UN des deux écrans :
		// muter `fetchAccounts(true)` dans `credit-notes/[id]` ne cassait toujours
		// rien. Le volet avoir est pourtant le plus exposé — un avoir porte par
		// construction des comptes hérités d'une facture ancienne, donc les plus
		// susceptibles d'avoir été archivés depuis.
		const cnRes = await ctx2.post('/api/v1/credit-notes', {
			data: { invoiceId, date: new Date().toISOString().slice(0, 10) },
		});
		expect(cnRes.ok(), `émission de l'avoir échouée: ${cnRes.status()}`).toBeTruthy();
		const creditNoteId = (await cnRes.json()).id as number;

		// --- Non-régression de D11 CÔTÉ LECTURE (AC14-bis) ---
		//
		// Le piège de D11 se rejoue à l'identique sur l'écran de détail : sans
		// `fetchAccounts(true)`, un compte archivé DEPUIS la validation n'est plus
		// dans la liste, et la colonne retombe sur `#3327` au lieu du nom du compte.
		// C'est le seul garde-fou automatisé de ce câblage : les tests Vitest ne
		// couvrent que la fonction pure `account-label`, jamais la page.
		// (Trou relevé en passe 1 de revue, Acceptance Auditor.)
		// `version` vient de la réponse de création : aucune route `GET
		// /accounts/{id}` n'existe (seulement le listing), et rien n'a modifié le
		// compte entre-temps.
		const archiveRes = await ctx2.put(`/api/v1/accounts/${marchandises!.id}/archive`, {
			data: { version: marchandises!.version },
		});
		expect(archiveRes.ok(), `archivage échoué: ${archiveRes.status()}`).toBeTruthy();

		await page.goto(`/invoices/${invoiceId}`);
		await expect(page.getByTestId('invoice-line-revenue-account').nth(1)).toContainText(
			marchandises!.name
		);

		await page.goto(`/credit-notes/${creditNoteId}`);
		await expect(page.getByTestId('credit-note-line-revenue-account').nth(1)).toContainText(
			marchandises!.name
		);
	} finally {
		await disposeContextSafe(ctx2);
	}
});
