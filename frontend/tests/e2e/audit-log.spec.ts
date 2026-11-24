import { expect, test } from '@playwright/test';
import {
	seedTestState,
	clearAuthStorage,
	authedApiContext,
	disposeContextSafe,
} from './helpers/test-state';
import { createUser, todayInZurich, USER_PASSWORD } from './helpers/api-fixtures';

/**
 * Tests E2E — l'écran du journal d'audit (Story 25-1c-b1, #378).
 *
 * # Ce que ces tests prouvent, et que rien d'autre ne prouve
 *
 * Que l'écran, ouvert **par le menu** par un **Comptable**, affiche une entrée
 * réellement écrite par le backend, **traduite** (le libellé que la route
 * elle-même rend pour ce code), qu'il filtre sur des jours **UTC**, déplie le
 * détail, exporte, et qu'un rôle **Consultation** n'y a pas accès.
 *
 * ⚠️ **La suite partage une base** : une ligne s'isole par le type d'entité et
 * l'identifiant du contact créé, jamais par sa position dans la liste. Les
 * filtres se pilotent **par valeur** (`selectOption({ value })`), jamais par
 * libellé : la suite ne tourne qu'en français, mais un sélecteur traduit est
 * une dette.
 */

test.beforeAll(async () => {
	await seedTestState('with-company');
});

test.afterEach(async ({ page }) => {
	await clearAuthStorage(page);
});

/** Connexion d'un utilisateur NOMMÉ (patron `company-contact-details.spec.ts`). */
async function login(
	page: import('@playwright/test').Page,
	username = 'admin',
	password = 'admin123',
) {
	await page.goto('/login');
	await page.fill('#username', username);
	await page.fill('#password', password);
	await page.click('button[type="submit"]');
	await expect(page).toHaveURL('/');
}

function uniqSuffix(): string {
	return `${Date.now()}-${Math.floor(Math.random() * 1e6)}`;
}

// `createUser` (création d'un utilisateur d'un rôle, en Admin) est extraite vers
// `helpers/api-fixtures.ts` (Story v016-2b, #816 : `bank-manual-statement.spec.ts` l'emploie
// aussi). Son mot de passe est `USER_PASSWORD`.

test('un Comptable ouvre le journal par le menu, filtre, déplie et exporte', async ({ page }) => {
	const comptable = await createUser(page, 'Comptable');
	await login(page, comptable, USER_PASSWORD);

	// ⚠️ `authedApiContext` reprend la session de la PAGE : c'est le Comptable
	// qui crée le contact, et c'est lui que la route doit servir.
	const ctx = await authedApiContext(page);
	let contactId: number;
	let label: string;
	try {
		const res = await ctx.post('/api/v1/contacts', {
			data: {
				contactType: 'Entreprise',
				name: `Audit ${uniqSuffix()}`,
				isClient: true,
				isSupplier: false,
				addressStructured: { street: '', building: '', postalCode: '', city: '', country: 'CH' },
			},
		});
		expect(res.ok(), `création du contact : ${res.status()}`).toBeTruthy();
		contactId = (await res.json()).id as number;
		const vocab = await (await ctx.get('/api/v1/audit-log/vocabulary')).json();
		label = (vocab.actions as { code: string; label: string }[]).find(
			(a) => a.code === 'contact.created',
		)!.label;
	} finally {
		await disposeContextSafe(ctx);
	}

	// 1. Par le menu — le groupe « Administration » est replié.
	await page.goto('/');
	await page.locator('[data-testid="nav-group-administration"] summary').click();
	await page.getByTestId('nav-link-audit-log').click();
	await expect(page).toHaveURL(/\/audit-log/);

	await page.getByTestId('audit-log-filter-entity-type').selectOption({ value: 'contact' });
	await page.getByTestId('audit-log-filter-entity-id').fill(String(contactId));
	await page.getByTestId('audit-log-filter-entity-id').press('Enter');
	await page.getByTestId('audit-log-filter-entity-id').blur();

	const row = page.locator('[data-testid="audit-log-row"][data-action="contact.created"]');
	await expect(row).toHaveCount(1);
	// ⛔ La cellule vaut le libellé que la route rend pour ce code : l'écran
	// affiche la traduction, exacte, quelle que soit la langue.
	await expect(row.getByTestId('audit-log-row-action')).toHaveText(label);

	// 2. L'URL survit au rechargement.
	await expect(page).toHaveURL(new RegExp(`entityId=${contactId}`));
	await page.reload();
	await expect(row).toHaveCount(1);

	// 3. Le détail se déplie et contient le JSON.
	await row.getByTestId('audit-log-details-toggle').click();
	await expect(row.getByTestId('audit-log-details')).toContainText('{');

	// 4. L'export télécharge le fichier nommé par la route.
	const [download] = await Promise.all([
		page.waitForEvent('download'),
		page.getByTestId('audit-log-export').click(),
	]);
	expect(download.suggestedFilename()).toMatch(/^kesh-journal-audit-/);

	// 5. Une période PASSÉE l'écarte.
	await page.getByTestId('audit-log-filter-date-from').fill('2000-01-01');
	await page.getByTestId('audit-log-filter-date-to').fill('2000-01-02');
	await page.getByTestId('audit-log-filter-date-to').blur();
	await expect(page.getByTestId('audit-log-empty')).toBeVisible();
	await expect(row).toHaveCount(0);

	// 6. ⛔ La période qui la GARDE se calcule en jour de Zurich, comme le filtre de la route
	// depuis la Story 36-10 (#1147) : entre 22 h ou 23 h et minuit UTC, le jour de Zurich est le
	// LENDEMAIN du jour UTC, et un jour UTC ne garderait pas l'entrée. La spec ne joue cette
	// nuit-là que lancée à cette heure : le filtre de nuit se teste, déterministe, par HTTP
	// (`crates/kesh-api/tests/audit_log_e2e.rs`).
	await page.getByTestId('audit-log-filter-date-from').fill(todayInZurich());
	await page.getByTestId('audit-log-filter-date-to').fill(todayInZurich());
	await page.getByTestId('audit-log-filter-date-to').blur();
	await expect(row).toHaveCount(1);
});

test("un Consultation ne voit pas l'entrée de menu, et /audit-log le redirige", async ({ page }) => {
	const lecteur = await createUser(page, 'Consultation');
	await login(page, lecteur, USER_PASSWORD);

	await page.locator('[data-testid="nav-group-administration"] summary').click();
	// Anti-vacuité : une entrée du même groupe, visible de tous, est rendue
	// AVANT qu'on constate l'absence.
	await expect(page.getByTestId('nav-link-settings')).toHaveCount(1);
	await expect(page.getByTestId('nav-link-audit-log')).toHaveCount(0);

	// La redirection porte à elle seule la preuve : une page qui ne se charge
	// pas garde l'URL `/audit-log`, et rien d'autre que la garde ne mène un
	// Consultation sur `/`.
	await page.goto('/audit-log');
	await expect(page).toHaveURL('/');
	await expect(page.getByTestId('homepage-card-open-invoices')).toBeVisible();
});

// Story 50-6 (#1511, AC11) — la date de la ligne est à l'heure de Zurich, quel que soit le fuseau
// du navigateur. Le navigateur est réglé sur New York : avant la story, la cellule montrait l'heure
// de New York (cinq ou six heures de moins), jamais celle de Zurich.
//
// ⛔ Les deux attendus sont calculés **dans le navigateur** (`page.evaluate`) : le corps d'un test
// tourne dans Node, dont l'ICU pourrait différer de celui de Chromium, qui met la cellule en forme.
test.describe('le fuseau du poste est celui de New York (50-6)', () => {
	test.use({ timezoneId: 'America/New_York' });

	test('la cellule de date égale l’heure de Zurich de createdAt, et diffère de celle de New York', async ({
		page,
	}) => {
		await login(page);
		const ctx = await authedApiContext(page);
		let contactId: number;
		let createdAt: string;
		try {
			const res = await ctx.post('/api/v1/contacts', {
				data: {
					contactType: 'Entreprise',
					name: `Audit fuseau ${uniqSuffix()}`,
					isClient: true,
					isSupplier: false,
					addressStructured: { street: '', building: '', postalCode: '', city: '', country: 'CH' },
				},
			});
			expect(res.ok(), `création du contact : ${res.status()}`).toBeTruthy();
			contactId = (await res.json()).id as number;
			const lu = await ctx.get(`/api/v1/audit-log?entityType=contact&entityId=${contactId}`);
			expect(lu.ok(), `lecture du journal : ${lu.status()}`).toBeTruthy();
			const items = (await lu.json()).items as { action: string; createdAt: string }[];
			const e = items.find((i) => i.action === 'contact.created');
			expect(e, 'entrée contact.created').toBeDefined();
			createdAt = e!.createdAt;
		} finally {
			await disposeContextSafe(ctx);
		}
		// Le serveur écrit un `Z` : l'instant est sans ambiguïté.
		expect(createdAt).toMatch(/Z$/);

		await page.goto(`/audit-log?entityType=contact&entityId=${contactId}`);
		const cellule = page
			.locator('[data-testid="audit-log-row"][data-action="contact.created"]')
			.getByTestId('audit-log-row-date');
		await expect(cellule).toHaveCount(1);

		const formater = (zone: string) =>
			page.evaluate(
				([iso, timeZone]) =>
					new Intl.DateTimeFormat('fr-CH', { dateStyle: 'medium', timeStyle: 'short', timeZone }).format(
						new Date(iso),
					),
				[createdAt, zone] as const,
			);
		const zurich = await formater('Europe/Zurich');
		const newYork = await formater('America/New_York');
		// Garde-fou : le décalage de six heures (cinq ou six selon l'été) se voit.
		expect(zurich).not.toBe(newYork);
		await expect(cellule).toHaveText(zurich);
		await expect(cellule).not.toHaveText(newYork);
	});
});
