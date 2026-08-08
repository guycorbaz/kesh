import { expect, test } from '@playwright/test';
import {
	seedTestState,
	clearAuthStorage,
	authedApiContext,
	disposeContextSafe,
} from './helpers/test-state';

/**
 * Tests E2E — coordonnées de l'émetteur dans les réglages (Story 16-3a, #151).
 *
 * # Ce que ces tests prouvent, et que rien d'autre ne prouve
 *
 * Le Vitest de la story (`settings.api.test.ts`) mocke `apiClient` : il vérifie
 * l'enveloppe HTTP **construite**, jamais le composant. Les tests Rust vérifient
 * que le backend valide et persiste. Entre les deux, **rien** n'exerçait le seul
 * écran de saisie de la story — alors que tous ses `data-testid` étaient déjà
 * posés, points d'ancrage créés pour un test qui n'existait pas.
 *
 * Sans ces scénarios, restaient verts : le retrait du `.trim()` (un champ rempli
 * d'espaces cesserait d'effacer), l'inversion des deux `<Input>` (le téléphone
 * recevrait l'URL), le retrait du `{#if isAdmin}` (tout comptable pourrait
 * éditer), et la branche `OPTIMISTIC_LOCK_CONFLICT` entière.
 *
 * ⚠️ **Convention Playwright du dépôt** : le fichier DOIT s'appeler `*.spec.ts`.
 * Un `*.test.ts` dans `tests/e2e/` est **silencieusement ignoré**.
 *
 * *(Ajouté en passe 6 de `bmad-code-review`.)*
 */

test.beforeAll(async () => {
	await seedTestState('with-company');
});

test.afterEach(async ({ page }) => {
	await clearAuthStorage(page);
});

async function login(
	page: import('@playwright/test').Page,
	username = 'admin',
	password = 'admin123'
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

/** Remet les deux coordonnées à `null` — les tests partagent une seule société. */
async function resetContactDetails(ctx: import('@playwright/test').APIRequestContext) {
	const cur = await ctx.get('/api/v1/companies/current');
	expect(cur.ok()).toBeTruthy();
	const { company } = (await cur.json()) as { company: { version: number } };
	const res = await ctx.put('/api/v1/companies/current/contact-details', {
		data: { phone: null, website: null, version: company.version },
	});
	expect(res.ok(), `remise à zéro échouée: ${res.status()}`).toBeTruthy();
}

// ===========================================================================
// 1 — aller-retour complet PAR L'INTERFACE.
// ===========================================================================

test('saisir téléphone et site web dans les réglages : les deux traversent la frontière HTTP', async ({
	page,
}) => {
	await login(page);
	const ctx = await authedApiContext(page);

	try {
		await resetContactDetails(ctx);
		const suffix = uniqSuffix();
		const phone = `+41 21 ${suffix.slice(-6)}`;
		const website = `https://demo-${suffix}.ch`;

		await page.goto('/settings');
		await expect(page.getByTestId('settings-company-phone')).toHaveText('—');

		await page.getByTestId('settings-company-contact-edit').click();
		await page.getByTestId('settings-company-phone-input').fill(phone);
		await page.getByTestId('settings-company-website-input').fill(website);
		await page.getByTestId('settings-company-contact-save').click();

		// --- L'affichage revient en lecture avec les deux valeurs.
		await expect(page.getByTestId('settings-company-phone')).toHaveText(phone);
		await expect(page.getByTestId('settings-company-website')).toHaveText(website);

		// --- Relu depuis l'API : les valeurs ont bien traversé, et surtout
		// CHACUNE DANS SON CHAMP. C'est ce volet qui discrimine une inversion des
		// deux `<Input>` — que les assertions d'affichage ci-dessus, elles,
		// laisseraient passer si le composant inversait symétriquement.
		const relu = await ctx.get('/api/v1/companies/current');
		expect(relu.ok()).toBeTruthy();
		const { company } = (await relu.json()) as {
			company: { phone: string | null; website: string | null };
		};
		expect(company.phone, 'le téléphone doit être stocké dans `phone`').toBe(phone);
		expect(company.website, 'le site web doit être stocké dans `website`').toBe(website);
	} finally {
		await disposeContextSafe(ctx);
	}
});

// ===========================================================================
// 2 — pré-remplissage à la réouverture (`startContactEdit`).
// ===========================================================================

test('rouvrir l’édition : les champs portent les valeurs existantes', async ({ page }) => {
	await login(page);
	const ctx = await authedApiContext(page);

	try {
		await resetContactDetails(ctx);
		const suffix = uniqSuffix();
		const phone = `+41 22 ${suffix.slice(-6)}`;

		const cur = await ctx.get('/api/v1/companies/current');
		const { company } = (await cur.json()) as { company: { version: number } };
		const posed = await ctx.put('/api/v1/companies/current/contact-details', {
			data: { phone, website: null, version: company.version },
		});
		expect(posed.ok()).toBeTruthy();

		await page.goto('/settings');
		await page.getByTestId('settings-company-contact-edit').click();

		// Sans le pré-remplissage de `startContactEdit`, le champ serait VIDE —
		// et l'enregistrement suivant effacerait la valeur en silence, le `PUT`
		// étant un full-replace des deux champs.
		await expect(
			page.getByTestId('settings-company-phone-input'),
			'le champ doit être pré-rempli, sinon enregistrer effacerait la valeur'
		).toHaveValue(phone);
		await expect(page.getByTestId('settings-company-website-input')).toHaveValue('');
	} finally {
		await disposeContextSafe(ctx);
	}
});

// ===========================================================================
// 3 — un champ rempli d'ESPACES efface (le `.trim()`).
// ===========================================================================

test('un champ rempli d’espaces efface la coordonnée', async ({ page }) => {
	await login(page);
	const ctx = await authedApiContext(page);

	try {
		await resetContactDetails(ctx);
		const suffix = uniqSuffix();
		const phone = `+41 23 ${suffix.slice(-6)}`;

		const cur = await ctx.get('/api/v1/companies/current');
		const { company } = (await cur.json()) as { company: { version: number } };
		expect(
			(
				await ctx.put('/api/v1/companies/current/contact-details', {
					data: { phone, website: null, version: company.version },
				})
			).ok()
		).toBeTruthy();

		await page.goto('/settings');
		await page.getByTestId('settings-company-contact-edit').click();

		// ⚠️ Des ESPACES, pas un champ vide. Sans le `.trim()` du composant,
		// `'   ' || null` rend `'   '` — une chaîne TRUTHY : la valeur partirait
		// telle quelle et serait stockée. Un champ vide, lui, effacerait même
		// sans `.trim()`, et ne discriminerait donc rien.
		await page.getByTestId('settings-company-phone-input').fill('   ');
		await page.getByTestId('settings-company-contact-save').click();

		await expect(page.getByTestId('settings-company-phone')).toHaveText('—');

		const relu = await ctx.get('/api/v1/companies/current');
		const { company: apres } = (await relu.json()) as { company: { phone: string | null } };
		expect(apres.phone, 'des espaces seuls doivent EFFACER, pas être stockés').toBeNull();
	} finally {
		await disposeContextSafe(ctx);
	}
});

// ===========================================================================
// 4 — conflit de version : la branche OPTIMISTIC_LOCK_CONFLICT.
// ===========================================================================

test('écriture concurrente : le conflit de version est signalé et les données rechargées', async ({
	page,
}) => {
	await login(page);
	const ctx = await authedApiContext(page);

	try {
		await resetContactDetails(ctx);
		const suffix = uniqSuffix();

		await page.goto('/settings');
		// L'écran détient maintenant une version ; l'édition la fige.
		await page.getByTestId('settings-company-contact-edit').click();

		// --- Quelqu'un d'autre écrit entre-temps : la version est bumpée.
		const cur = await ctx.get('/api/v1/companies/current');
		const { company } = (await cur.json()) as { company: { version: number } };
		const concurrent = await ctx.put('/api/v1/companies/current/contact-details', {
			data: { phone: `+41 24 ${suffix.slice(-6)}`, website: null, version: company.version },
		});
		expect(concurrent.ok(), 'l’écriture concurrente doit réussir').toBeTruthy();

		// --- L'écran enregistre avec sa version périmée.
		await page.getByTestId('settings-company-phone-input').fill('+41 99 000 00 00');
		await page.getByTestId('settings-company-contact-save').click();

		await expect(
			page.getByTestId('settings-company-contact-error'),
			'le conflit doit être signalé à l’utilisateur, pas avalé'
		).toContainText('Conflit de version');
	} finally {
		await disposeContextSafe(ctx);
	}
});

// ===========================================================================
// 5 — gating `{#if isAdmin}` : un non-admin lit mais n'édite pas.
// ===========================================================================

test('un utilisateur non-admin voit les coordonnées mais n’a pas le bouton d’édition', async ({
	page,
}) => {
	await login(page);
	const ctx = await authedApiContext(page);
	const comptable = `comptable-${uniqSuffix()}`;

	try {
		const cree = await ctx.post('/api/v1/users', {
			data: { username: comptable, password: 'MotDePasse12345', role: 'Comptable' },
		});
		expect(cree.ok(), `création du comptable échouée: ${cree.status()}`).toBeTruthy();
	} finally {
		await disposeContextSafe(ctx);
	}

	await clearAuthStorage(page);
	await login(page, comptable, 'MotDePasse12345');
	await page.goto('/settings');

	// La valeur reste LISIBLE — le gating porte sur l'édition, pas sur la
	// consultation. Asserter les deux : sans le premier volet, un écran vide ou
	// une redirection satisferait le second sans rien prouver.
	await expect(page.getByTestId('settings-company-phone')).toBeVisible();
	await expect(
		page.getByTestId('settings-company-contact-edit'),
		'le bouton d’édition est Admin-only'
	).toHaveCount(0);
});
