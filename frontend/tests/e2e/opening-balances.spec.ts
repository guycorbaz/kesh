import { expect, test } from '@playwright/test';
import {
	seedTestState,
	clearAuthStorage,
	authedApiContext,
	disposeContextSafe,
} from './helpers/test-state';

/**
 * Tests E2E — Bilan d'ouverture / soldes de départ (Story 14-4, P1-M3-AA).
 *
 * Parcours migrant bout-en-bout navigateur : company vierge avec un premier
 * exercice Open (preset `with-company` : plan comptable seedé, AUCUNE
 * écriture) → statut `READY` → grille → saisie de soldes équilibrés (un actif
 * + la contrepartie sur le compte de rôle RetainedEarnings) → Générer →
 * toast + état verrouillé `ALREADY_HAS_ENTRIES` in-place → présence des
 * soldes au bilan (/reports).
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

/**
 * Résout via l'API un compte d'actif postable et sa contrepartie capitaux
 * propres : le compte de rôle `RetainedEarnings` si le plan seedé en a un,
 * sinon n'importe quel passif postable (le plan minimal du preset
 * `with-company` a 5 comptes SANS rôles — `2000 Capital CI` sert alors de
 * contrepartie equity). Résolution par RÔLE/TYPE, jamais par numéro hardcodé
 * (principe 14-3).
 */
async function getGridAccounts(
	page: import('@playwright/test').Page
): Promise<{ assetNumber: string; equityNumber: string }> {
	const ctx = await authedApiContext(page);
	try {
		const resp = await ctx.get('/api/v1/accounts?includeArchived=false');
		expect(resp.ok()).toBeTruthy();
		const accounts: Array<{
			number: string;
			accountType: string;
			role: string | null;
			postable: boolean;
			active: boolean;
		}> = await resp.json();

		const asset = accounts.find((a) => a.accountType === 'Asset' && a.postable && a.active);
		const equity =
			accounts.find((a) => a.role === 'RetainedEarnings' && a.postable && a.active) ??
			accounts.find((a) => a.accountType === 'Liability' && a.postable && a.active);
		expect(asset, 'un compte Asset postable doit exister dans le plan seedé').toBeTruthy();
		expect(
			equity,
			'une contrepartie de bilan (RetainedEarnings ou passif postable) doit exister'
		).toBeTruthy();
		return { assetNumber: asset!.number, equityNumber: equity!.number };
	} finally {
		await disposeContextSafe(ctx);
	}
}

test('parcours migrant : grille READY → saisie équilibrée → génération → verrou + bilan', async ({
	page,
}) => {
	// Seed frais POUR CE TEST : la company doit être vierge de toute écriture
	// (le beforeAll peut être pollué si un test précédent du même fichier a
	// généré l'écriture — re-seed défensif).
	await seedTestState('with-company');
	await login(page);

	// L'entrée de menu (Administration, Comptable+) est visible pour l'admin.
	await page.goto('/settings/opening-balances');
	await expect(page.getByTestId('nav-link-settings-opening-balances')).toBeVisible();

	// Statut READY sur company vierge → grille visible, pas de verrou.
	await expect(page.getByTestId('opening-balances-grid')).toBeVisible();
	await expect(page.getByTestId('opening-balances-locked')).not.toBeVisible();

	const { assetNumber, equityNumber } = await getGridAccounts(page);

	// Saisie : 5000 à l'actif, contrepartie 5000 en capitaux propres.
	await page.getByTestId(`opening-balances-debit-${assetNumber}`).fill('5000');
	await page.getByTestId(`opening-balances-credit-${equityNumber}`).fill('5000');

	// Bandeau : équilibré, bouton actif.
	await expect(page.getByTestId('opening-balances-total-debit')).toHaveText(/5.000\.00/);
	await expect(page.getByTestId('opening-balances-total-credit')).toHaveText(/5.000\.00/);
	const generate = page.getByTestId('opening-balances-generate');
	await expect(generate).toBeEnabled();

	await generate.click();

	// Succès : toast + verrou ALREADY_HAS_ENTRIES in-place (P1-M2-BH), avec
	// liens vers le bilan et le journal.
	const locked = page.getByTestId('opening-balances-locked');
	await expect(locked).toBeVisible();
	await expect(locked).toHaveAttribute('data-reason', 'ALREADY_HAS_ENTRIES');
	await expect(page.getByTestId('opening-balances-goto-balance-sheet')).toBeVisible();
	await expect(page.getByTestId('opening-balances-goto-journal')).toBeVisible();
	await expect(page.getByTestId('opening-balances-grid')).not.toBeVisible();
	// Story 25-7 : le preset n'a aucun compte de rôle « Bénéfice reporté » —
	// le complément est indisponible, et l'écran dit pourquoi.
	await expect(page.getByTestId('opening-balances-complete-unavailable')).toHaveAttribute(
		'data-reason',
		'NO_RETAINED_EARNINGS'
	);

	// Présence des soldes au bilan (/reports) — l'OD datée fy_start est
	// incluse nativement dans le calcul cumulatif.
	await page.goto('/reports');
	await page.waitForLoadState('networkidle');
	await page.getByRole('button', { name: /générer/i }).click();
	// Total actifs = 5'000.00 (apostrophe suisse U+2019 → regex laxiste).
	await expect(page.getByText(/total actifs/i)).toBeVisible({ timeout: 5000 });
	await expect(page.getByText(/5.000\.00/).first()).toBeVisible();
});

test('bouton Générer désactivé tant que la saisie est déséquilibrée', async ({ page }) => {
	await seedTestState('with-company');
	await login(page);
	await page.goto('/settings/opening-balances');
	await expect(page.getByTestId('opening-balances-grid')).toBeVisible();

	const { assetNumber } = await getGridAccounts(page);

	// Vide → désactivé.
	const generate = page.getByTestId('opening-balances-generate');
	await expect(generate).toBeDisabled();

	// Débit seul (déséquilibré) → toujours désactivé.
	await page.getByTestId(`opening-balances-debit-${assetNumber}`).fill('1000');
	await expect(generate).toBeDisabled();
});

/**
 * Story 25-7 (#445) — compléter un compte oublié.
 *
 * Le preset `with-company` n'a pas de compte de report : on le crée par l'API
 * AVANT la génération (il sert alors de contrepartie, cf. `getGridAccounts`).
 * L'ouverture porte le premier actif ; le second, oublié, est complété ensuite.
 * L'exercice seedé couvre 2020-2030 et il est ouvert : branche (a), le
 * complément est daté du premier jour de l'exercice.
 */
test('compléter un compte oublié après la génération', async ({ page }) => {
	await seedTestState('with-company');
	await login(page);

	const ctx = await authedApiContext(page);
	let forgotten: string;
	try {
		const created = await ctx.post('/api/v1/accounts', {
			data: {
				number: '2970',
				name: 'Bénéfice reporté',
				accountType: 'Liability',
				role: 'RetainedEarnings',
			},
		});
		expect(created.ok(), await created.text()).toBeTruthy();
		const resp = await ctx.get('/api/v1/accounts?includeArchived=false');
		const accounts: Array<{ number: string; accountType: string; postable: boolean; active: boolean }> =
			await resp.json();
		const assets = accounts
			.filter((a) => a.accountType === 'Asset' && a.postable && a.active)
			.map((a) => a.number)
			.sort();
		expect(assets.length, 'il faut deux actifs : un saisi, un oublié').toBeGreaterThanOrEqual(2);
		forgotten = assets[1];
	} finally {
		await disposeContextSafe(ctx);
	}

	await page.goto('/settings/opening-balances');
	await expect(page.getByTestId('opening-balances-grid')).toBeVisible();
	const { assetNumber, equityNumber } = await getGridAccounts(page);
	expect(equityNumber).toBe('2970');
	expect(assetNumber).not.toBe(forgotten);

	await page.getByTestId(`opening-balances-debit-${assetNumber}`).fill('5000');
	await page.getByTestId(`opening-balances-credit-${equityNumber}`).fill('5000');
	// Report porté : pas d'avertissement.
	await expect(page.getByTestId('opening-balances-no-retained-earnings')).not.toBeVisible();
	await page.getByTestId('opening-balances-generate').click();
	await expect(page.getByTestId('opening-balances-locked')).toHaveAttribute(
		'data-reason',
		'ALREADY_HAS_ENTRIES'
	);

	// Le mode « compléter » : l'actif oublié est proposé, celui de l'ouverture non.
	const complete = page.getByTestId('opening-balances-complete');
	await expect(complete).toBeVisible();
	await expect(page.getByTestId(`opening-balances-complete-debit-${assetNumber}`)).toHaveCount(0);
	await expect(page.getByTestId('opening-balances-complete-date')).toHaveAttribute(
		'data-kind',
		'OPENING_DAY'
	);

	await page.getByTestId(`opening-balances-complete-debit-${forgotten}`).fill('750');
	await expect(page.getByTestId('opening-balances-complete-counterpart')).toHaveAttribute(
		'data-side',
		'credit'
	);
	page.once('dialog', (d) => void d.accept());
	await page.getByTestId('opening-balances-complete-submit').click();

	// Rechargé : le compte complété n'est plus proposé.
	await expect(page.getByTestId(`opening-balances-complete-debit-${forgotten}`)).toHaveCount(0);

	// La balance le porte (revue de code P1, A-M2 : par l'API, sans libellé
	// traduit ni format monétaire) : solde de clôture 750 sur le compte oublié.
	const api = await authedApiContext(page);
	try {
		const status = await (await api.get('/api/v1/opening-balances/status')).json();
		const resp = await api.get(
			`/api/v1/reports/trial-balance?fiscalYearId=${status.fiscalYear.id}`
		);
		expect(resp.ok(), await resp.text()).toBeTruthy();
		const tb = await resp.json();
		const row = tb.rows.find((r: { accountNumber: string }) => r.accountNumber === forgotten);
		expect(row, `ligne ${forgotten} dans la balance`).toBeTruthy();
		expect(Number(row.closingBalance)).toBe(750);
	} finally {
		await disposeContextSafe(api);
	}
});
