import { expect, test } from '@playwright/test';
import {
	seedTestState,
	clearAuthStorage,
	authedApiContext,
	disposeContextSafe,
} from './helpers/test-state';

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

/** Crée un utilisateur du rôle donné, en Admin ; rend son nom. */
async function createUser(page: import('@playwright/test').Page, role: string): Promise<string> {
	await login(page);
	const ctx = await authedApiContext(page);
	const username = `${role.toLowerCase()}-${uniqSuffix()}`;
	try {
		const res = await ctx.post('/api/v1/users', {
			data: { username, password: 'MotDePasse12345', role },
		});
		expect(res.ok(), `création de l'utilisateur ${role} : ${res.status()}`).toBeTruthy();
	} finally {
		await disposeContextSafe(ctx);
	}
	await clearAuthStorage(page);
	return username;
}

/** Le jour UTC courant, `AAAA-MM-JJ` — la route filtre sur des jours UTC. */
function todayUtc(): string {
	return new Date().toISOString().slice(0, 10);
}

test('un Comptable ouvre le journal par le menu, filtre, déplie et exporte', async ({ page }) => {
	const comptable = await createUser(page, 'Comptable');
	await login(page, comptable, 'MotDePasse12345');

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

	// 5. Le détail se déplie et contient le JSON.
	await row.getByTestId('audit-log-details-toggle').click();
	await expect(row.getByTestId('audit-log-details')).toContainText('{');

	// 6. L'export télécharge le fichier nommé par la route.
	const [download] = await Promise.all([
		page.waitForEvent('download'),
		page.getByTestId('audit-log-export').click(),
	]);
	expect(download.suggestedFilename()).toMatch(/^kesh-journal-audit-/);

	// 3. Une période PASSÉE l'écarte.
	await page.getByTestId('audit-log-filter-date-from').fill('2000-01-01');
	await page.getByTestId('audit-log-filter-date-to').fill('2000-01-02');
	await page.getByTestId('audit-log-filter-date-to').blur();
	await expect(page.getByTestId('audit-log-empty')).toBeVisible();
	await expect(row).toHaveCount(0);

	// 4. ⛔ La période qui la GARDE se calcule en jour UTC, non en jour local :
	// entre 00:00 et 02:00 à Zurich, le jour local précède le jour UTC.
	await page.getByTestId('audit-log-filter-date-from').fill(todayUtc());
	await page.getByTestId('audit-log-filter-date-to').fill(todayUtc());
	await page.getByTestId('audit-log-filter-date-to').blur();
	await expect(row).toHaveCount(1);
});

test("un Consultation ne voit pas l'entrée de menu, et /audit-log le redirige", async ({ page }) => {
	const lecteur = await createUser(page, 'Consultation');
	await login(page, lecteur, 'MotDePasse12345');

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
