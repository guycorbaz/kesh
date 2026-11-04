/**
 * Story v014-1 — Tests E2E Playwright pour la sidebar collapsible avec
 * persistence localStorage.
 *
 * AC#33 — couvre :
 *   1. Navigation vers les 5 entrées Administration (Plan comptable,
 *      Exercices, Comptes bancaires, Profils bancaires, Règles d'affectation)
 *      depuis la sidebar.
 *   2. Persistence localStorage : toggle Administration expand → reload
 *      → état préservé.
 *   3. Auto-expand du groupe contenant la route active (F12 Pass 3 Opus).
 */

import path from 'node:path';
import { expect, test, type Page, type TestInfo } from '@playwright/test';
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

async function loginAsAdmin(page: Page): Promise<void> {
	await page.goto('/login');
	await page.fill('#username', 'admin');
	await page.fill('#password', 'admin123');
	await page.click('button[type="submit"]');
	await expect(page).toHaveURL('/');
}

test.describe('Sidebar navigation v014-1', () => {
	// ⚠️ Le groupe se cible par `data-testid`, jamais par `has-text("Administration")`. Le
	// libellé est traduit depuis la story 23-3b : identique en français, en allemand et en
	// anglais — mais `Amministrazione` en italien. Un sélecteur qui passe dans trois langues
	// sur quatre est plus traître qu'un sélecteur cassé, puisque rien ne le signale.
	test('groupe Administration fermé par défaut, ouverture au clic', async ({ page }) => {
		await loginAsAdmin(page);

		const sidebar = page.locator('nav[aria-label="Navigation principale"]');
		const adminDetails = sidebar.locator('[data-testid="nav-group-administration"]');

		// Défaut fermé.
		await expect(adminDetails).not.toHaveAttribute('open', /.*/);

		// Click pour ouvrir.
		await sidebar.locator('[data-testid="nav-group-administration"] summary').click();
		await expect(adminDetails).toHaveAttribute('open', '');
	});

	test('navigue vers les 5 entrées Administration', async ({ page }) => {
		await loginAsAdmin(page);
		const sidebar = page.locator('nav[aria-label="Navigation principale"]');
		await sidebar.locator('[data-testid="nav-group-administration"] summary').click();

		const routes = [
			{ testid: 'nav-link-accounts', url: '/accounts' },
			{ testid: 'nav-link-settings-fiscal-years', url: '/settings/fiscal-years' },
			{ testid: 'nav-link-bank-accounts', url: '/bank-accounts' },
			{ testid: 'nav-link-bank-import-profiles', url: '/bank-import/profiles' },
			{ testid: 'nav-link-reconciliation-rules', url: '/reconciliation/rules' },
		];

		for (const { testid, url } of routes) {
			await sidebar.locator(`[data-testid="${testid}"]`).click();
			await expect(page).toHaveURL(url);
			// Le groupe doit rester ouvert (auto-expand F12 Pass 3 Opus pour route active).
			await expect(
				sidebar.locator('[data-testid="nav-group-administration"]'),
			).toHaveAttribute('open', '');
		}
	});

	test('persistence localStorage : Administration ouvert puis reload reste ouvert', async ({ page }) => {
		await loginAsAdmin(page);
		const sidebar = page.locator('nav[aria-label="Navigation principale"]');
		const adminDetails = sidebar.locator('[data-testid="nav-group-administration"]');

		// Ouvrir le groupe (persiste via localStorage).
		await sidebar.locator('[data-testid="nav-group-administration"] summary').click();
		await expect(adminDetails).toHaveAttribute('open', '');

		// Vérifier la clé localStorage.
		const stored = await page.evaluate(() =>
			localStorage.getItem('kesh:sidebar:expanded:administration'),
		);
		expect(stored).toBe('true');

		// Reload — l'état doit persister.
		await page.reload();
		await expect(adminDetails).toHaveAttribute('open', '');
	});

	test('auto-expand groupe contenant la route active (F12 Pass 3 Opus)', async ({ page }) => {
		await loginAsAdmin(page);

		// Forcer Administration fermé en localStorage AVANT navigation.
		await page.evaluate(() => {
			localStorage.setItem('kesh:sidebar:expanded:administration', 'false');
		});

		// Naviguer directement vers /accounts (sous Administration).
		await page.goto('/accounts');

		// Le groupe Administration doit s'auto-expandre car contient la route active.
		const sidebar = page.locator('nav[aria-label="Navigation principale"]');
		await expect(
			sidebar.locator('[data-testid="nav-group-administration"]'),
		).toHaveAttribute('open', '');
	});
});

// ===========================================================================
// v0172-correctifs (#851) — « des entrées du menu disparaissent ». La cause lue
// au code : la barre latérale a une hauteur fixe (`h-[calc(100vh-3.5rem)]`)
// sans défilement, alors que l'Administrateur y a 33 entrées. AC-B5 est
// l'ORACLE DU SYMPTÔME, joué avant tout correctif : son résultat décide du
// correctif (C-v0172b-8). AC-B5 bis garde le correctif, AC-B5 ter l'aspect de
// la barre, AC-B6 l'hypothèse (réfutée) du rôle nul.
// ===========================================================================

const SIDEBAR = 'nav[aria-label="Navigation principale"]';
/** Les entrées, recomptées depuis `navGroups` (`+layout.svelte`). */
const QUOTIDIEN = [
	'nav-link-home',
	'nav-link-contacts',
	'nav-link-products',
	'nav-link-invoices',
	'nav-link-invoices-due-dates',
	'nav-link-invoices-reminders',
	'nav-link-credit-notes',
	'nav-link-supplier-invoices',
	'nav-link-supplier-invoices-import',
	'nav-link-payment-batches',
	'nav-link-bank-import',
];
const MENSUEL = [
	'nav-link-journal-entries',
	'nav-link-reconciliation',
	'nav-link-open-items',
	'nav-link-reports',
];
const ADMINISTRATION = [
	'nav-link-accounts',
	'nav-link-settings-fiscal-years',
	'nav-link-bank-accounts',
	'nav-link-bank-import-profiles',
	'nav-link-reconciliation-rules',
	'nav-link-settings-projects',
	'nav-link-export',
	'nav-link-settings',
	'nav-link-settings-opening-balances',
	'nav-link-audit-log',
	'nav-link-users',
	'nav-link-settings-invoicing',
	'nav-link-settings-email-templates',
	'nav-link-admin-backup',
	'nav-link-admin-restore',
	'nav-link-admin-mcp',
	'nav-link-admin-mcp-sends',
	'nav-link-admin-mcp-prompts',
];
const GROUP_KEYS = ['quotidien', 'mensuel', 'administration'];

/** Le dossier des captures : `KESH_E2E_CAPTURES_DIR`, sinon la sortie du test (hors dépôt). */
function capture851(testInfo: TestInfo, name: string): string {
	const dossier = process.env.KESH_E2E_CAPTURES_DIR;
	// `KESH_E2E_CAPTURE_PHASE` (« avant » / « apres ») distingue les deux jeux de captures.
	const phase = process.env.KESH_E2E_CAPTURE_PHASE ?? 'apres';
	const file = `v0172b-851-${phase}-${name}.png`;
	return dossier ? path.join(dossier, file) : testInfo.outputPath(file);
}

/** Vrai si le lien est dans la fenêtre et que le point au centre de sa boîte est le lien. */
async function reachableNow(page: Page, testid: string): Promise<boolean> {
	return page.evaluate((id) => {
		const a = document.querySelector(`[data-testid="${id}"]`);
		if (!(a instanceof HTMLElement)) return false;
		const r = a.getBoundingClientRect();
		if (r.width === 0 || r.height === 0) return false;
		const cx = r.left + r.width / 2;
		const cy = r.top + r.height / 2;
		if (cx < 0 || cy < 0 || cx >= window.innerWidth || cy >= window.innerHeight) return false;
		const hit = document.elementFromPoint(cx, cy);
		return hit !== null && (hit === a || a.contains(hit));
	}, testid);
}

async function scrollPositions(page: Page): Promise<string> {
	return page.evaluate(
		(sel) => `${window.scrollY}:${document.querySelector(sel)?.scrollTop ?? 0}`,
		SIDEBAR,
	);
}

/**
 * « Atteignable » (AC-B5) : depuis le haut de la page et de la barre, la souris
 * posée sur la barre, des crans de molette (au plus 20, jusqu'à stabilité),
 * jusqu'à ce que le lien soit visé par le point au centre de sa boîte.
 */
async function reachable(page: Page, testid: string): Promise<boolean> {
	await page.evaluate((sel) => {
		window.scrollTo(0, 0);
		const nav = document.querySelector(sel);
		if (nav) nav.scrollTop = 0;
	}, SIDEBAR);
	await page.mouse.move(110, 400);
	for (let i = 0; i < 20; i++) {
		if (await reachableNow(page, testid)) return true;
		const before = await scrollPositions(page);
		await page.mouse.wheel(0, 200);
		await page.waitForTimeout(60);
		if ((await scrollPositions(page)) === before) break;
	}
	return reachableNow(page, testid);
}

type Releve = Record<string, boolean>;

async function releve(page: Page, ids: string[]): Promise<Releve> {
	const out: Releve = {};
	for (const id of ids) out[id] = await reachable(page, id);
	return out;
}

/** Part de l'état par défaut des groupes (Quotidien et Mensuel dépliés, Administration replié). */
async function resetGroups(page: Page): Promise<void> {
	await page.evaluate((keys) => {
		for (const k of keys) localStorage.removeItem(`kesh:sidebar:expanded:${k}`);
	}, GROUP_KEYS);
	await page.reload();
	await expect(page.locator(`${SIDEBAR} [data-testid="nav-group-quotidien"]`)).toHaveAttribute(
		'open',
		'',
	);
	await expect(
		page.locator(`${SIDEBAR} [data-testid="nav-group-administration"]`),
	).not.toHaveAttribute('open', /.*/);
	const minHeight = await page
		.getByTestId('nav-link-home')
		.evaluate((a) => getComputedStyle(a).minHeight);
	expect(minHeight, 'montage : mode guidé (liens de 44 px)').toBe('44px');
}

/** Affiche le bandeau de sauvegarde automatique (réponse de l'API simulée). */
async function showBackupBanner(page: Page): Promise<void> {
	await page.route('**/api/v1/admin/backup-schedule', (route) =>
		route.fulfill({
			status: 200,
			contentType: 'application/json',
			body: JSON.stringify({
				enabled: true,
				intervalHours: 24,
				retentionCount: 7,
				version: 1,
				lastAttemptAt: '2026-11-01T02:00:00Z',
				lastSuccessAt: null,
				lastSuccessFile: null,
				lastSuccessBytes: null,
				lastStatus: 'failed',
				lastFilesNotIncluded: null,
				lastCompleteFile: null,
				nextDueAt: '2026-11-05T02:00:00Z',
				overdue: false,
				lastSuccessInFuture: false,
			}),
		}),
	);
}

/**
 * Le scénario de l'issue : (1) relevé des groupes dépliés par défaut ; (2)
 * ouvrir Administration ; (3) relevé des 18 entrées d'Administration (et des
 * 15 autres) ; (4) cliquer Serveur MCP, puis relever de nouveau les 33.
 */
async function scenario851(
	page: Page,
	testInfo: TestInfo,
	variant: 'sans-bandeau' | 'avec-bandeau',
): Promise<Releve[]> {
	if (variant === 'avec-bandeau') await showBackupBanner(page);
	await loginAsAdmin(page);
	await resetGroups(page);
	if (variant === 'avec-bandeau') {
		await expect(page.getByTestId('backup-schedule-banner')).toBeVisible();
	} else {
		await expect(page.getByTestId('backup-schedule-banner')).toHaveCount(0);
	}
	const r1 = await releve(page, [...QUOTIDIEN, ...MENSUEL]);
	await page.locator(`${SIDEBAR} [data-testid="nav-group-administration"] summary`).click();
	await expect(
		page.locator(`${SIDEBAR} [data-testid="nav-group-administration"]`),
	).toHaveAttribute('open', '');
	const r3 = await releve(page, [...QUOTIDIEN, ...MENSUEL, ...ADMINISTRATION]);
	await page.getByTestId('nav-link-admin-mcp').click();
	await expect(page).toHaveURL('/admin/mcp');
	const r4 = await releve(page, [...QUOTIDIEN, ...MENSUEL, ...ADMINISTRATION]);
	// Capture : la barre défilée jusqu'en bas, sur `/admin/mcp`, cadrée sur la fenêtre.
	await reachable(page, 'nav-link-admin-mcp-prompts');
	await page.screenshot({ path: capture851(testInfo, `admin-mcp-${variant}`) });
	const releves = [r1, r3, r4];
	await testInfo.attach(`releves-851-${variant}.json`, {
		body: JSON.stringify(releves, null, 2),
		contentType: 'application/json',
	});
	if (process.env.KESH_E2E_CAPTURES_DIR) {
		const fs = await import('node:fs');
		fs.writeFileSync(
			path.join(
				process.env.KESH_E2E_CAPTURES_DIR,
				`v0172b-851-${process.env.KESH_E2E_CAPTURE_PHASE ?? 'apres'}-releves-${variant}.json`,
			),
			JSON.stringify(releves, null, 2),
		);
	}
	return releves;
}

/** Les entrées atteignables au relevé `prev` qui ne le sont plus à `next`. */
function lost(prev: Releve, next: Releve): string[] {
	return Object.keys(prev).filter((id) => prev[id] && id in next && !next[id]);
}

test.describe('Menu latéral — #851 (v0172-correctifs)', () => {
	test.use({ viewport: { width: 1280, height: 720 } });

	for (const variant of ['sans-bandeau', 'avec-bandeau'] as const) {
		test(`AC-B5 — oracle du symptôme (${variant}) : aucune entrée atteignable ne cesse de l’être`, async ({
			page,
		}, testInfo) => {
			test.setTimeout(180_000);
			const [r1, r3, r4] = await scenario851(page, testInfo, variant);
			expect(lost(r1, r3), 'entrées perdues à l’ouverture d’Administration').toEqual([]);
			expect(lost(r1, r4), 'entrées perdues sur /admin/mcp (Quotidien, Mensuel)').toEqual([]);
			expect(lost(r3, r4), 'entrées perdues sur /admin/mcp').toEqual([]);
		});

		test(`AC-B5 bis — garde du correctif (${variant}) : toutes les entrées sont atteignables`, async ({
			page,
		}, testInfo) => {
			test.setTimeout(180_000);
			const releves = await scenario851(page, testInfo, variant);
			const unreachable = releves.flatMap((r, i) =>
				Object.entries(r)
					.filter(([, ok]) => !ok)
					.map(([id]) => `relevé ${i + 1} : ${id}`),
			);
			expect(unreachable).toEqual([]);
			if (variant === 'sans-bandeau') {
				const m = await page.evaluate((sel) => {
					window.scrollTo(0, 0);
					const nav = document.querySelector(sel) as HTMLElement;
					return {
						scrollY: window.scrollY,
						overflows: nav.scrollHeight > nav.clientHeight,
						overflowY: getComputedStyle(nav).overflowY,
					};
				}, SIDEBAR);
				expect(m).toEqual({ scrollY: 0, overflows: true, overflowY: 'auto' });
			}
		});
	}

	/** Mesure la boîte qui porte le fond et le filet de la barre, et le pied de page. */
	async function barBox(page: Page) {
		return page.evaluate((sel) => {
			window.scrollTo(0, 0);
			const nav = document.querySelector(sel) as HTMLElement;
			const bar = nav.closest('.bg-surface-alt') as HTMLElement;
			const r = bar.getBoundingClientRect();
			return {
				innerHeight: window.innerHeight,
				bottom: r.bottom,
				height: r.height,
				footerTop: document.querySelector('footer')!.getBoundingClientRect().top,
				borderRightWidth: getComputedStyle(bar).borderRightWidth,
			};
		}, SIDEBAR);
	}

	function expectBarAspect(m: Awaited<ReturnType<typeof barBox>>): void {
		expect(m.bottom, `fond et filet jusqu'au bas de la fenêtre : ${JSON.stringify(m)}`).toBeGreaterThanOrEqual(
			m.innerHeight - 1,
		);
		expect(m.height, JSON.stringify(m)).toBeGreaterThanOrEqual(m.innerHeight - 56 - 1);
		expect(m.footerTop, `pied de page sous le pli : ${JSON.stringify(m)}`).toBeGreaterThanOrEqual(
			m.innerHeight - 1,
		);
		expect(m.borderRightWidth).toBe('1px');
	}

	test('AC-B5 ter — l’aspect de la barre : Comptable groupes repliés, Administrateur tout déplié', async ({
		page,
	}, testInfo) => {
		test.setTimeout(120_000);
		// Comptable : pas de bloc `adminOnly`.
		await loginAsAdmin(page);
		const comptable = `comptable-851-${Date.now()}`;
		const ctx = await authedApiContext(page);
		try {
			const cree = await ctx.post('/api/v1/users', {
				data: { username: comptable, password: 'MotDePasse12345', role: 'Comptable' },
			});
			expect(cree.ok(), `création du comptable : ${cree.status()}`).toBeTruthy();
		} finally {
			await disposeContextSafe(ctx);
		}
		await clearAuthStorage(page);
		await page.goto('/login');
		await page.fill('#username', comptable);
		await page.fill('#password', 'MotDePasse12345');
		await page.click('button[type="submit"]');
		await expect(page).toHaveURL('/');
		await page.evaluate((keys) => {
			for (const k of keys) localStorage.setItem(`kesh:sidebar:expanded:${k}`, 'false');
		}, GROUP_KEYS);
		await page.reload();
		// Sur `/`, le groupe Quotidien se déplie de lui-même (route active,
		// `+layout.svelte`, `$effect` d'auto-dépliage) : on le replie d'un clic.
		const quotidien = page.locator(`${SIDEBAR} [data-testid="nav-group-quotidien"]`);
		await expect(quotidien).toHaveAttribute('open', '');
		await quotidien.locator('summary').click();
		for (const k of GROUP_KEYS) {
			await expect(page.locator(`${SIDEBAR} [data-testid="nav-group-${k}"]`)).not.toHaveAttribute(
				'open',
				/.*/,
			);
		}
		expectBarAspect(await barBox(page));
		await page.screenshot({ path: capture851(testInfo, 'aspect-comptable-replie') });

		// Administrateur, tous groupes dépliés.
		await clearAuthStorage(page);
		await loginAsAdmin(page);
		await page.evaluate((keys) => {
			for (const k of keys) localStorage.setItem(`kesh:sidebar:expanded:${k}`, 'true');
		}, GROUP_KEYS);
		await page.reload();
		await expect(
			page.locator(`${SIDEBAR} [data-testid="nav-group-administration"]`),
		).toHaveAttribute('open', '');
		expectBarAspect(await barBox(page));
		await page.screenshot({ path: capture851(testInfo, 'aspect-admin-deplie') });
	});

	test('AC-B6 — le groupe Administration compte 18 entrées sur chaque page', async ({ page }) => {
		await loginAsAdmin(page);
		for (const url of ['/admin/mcp', '/admin/mcp/sends', '/admin/mcp/prompts', '/users', '/']) {
			await page.goto(url);
			await expect(
				page.locator(`${SIDEBAR} [data-testid="nav-group-administration"] a[data-testid^="nav-link-"]`),
				url,
			).toHaveCount(18);
		}
	});
});
