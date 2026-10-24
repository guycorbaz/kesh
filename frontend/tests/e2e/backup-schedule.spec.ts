import { expect, test } from '@playwright/test';
import type { Page, Route } from '@playwright/test';
import { promises as fs } from 'node:fs';
import path from 'node:path';
import { inflateRawSync } from 'node:zlib';
import {
	seedTestState,
	clearAuthStorage,
	authedApiContext,
	disposeContextSafe,
} from './helpers/test-state';

/**
 * Story 31-1c (#503, AC38, AC40) — la sauvegarde automatique faite par Kesh, de bout en
 * bout : un Administrateur règle la fréquence et le nombre conservé (gardés au
 * rechargement), « Sauvegarder maintenant » écrit une sauvegarde que la liste du serveur
 * montre, et son téléchargement rend une archive dont le manifeste dit `formatVersion: 2`.
 * Un Comptable n'a ni l'entrée de menu ni la page (redirigé vers `/`) ; le refus de l'API
 * (403) est prouvé côté Rust (`admin_auto_backup_e2e.rs`).
 *
 * ⚠️ La table `_kesh_backup_schedule` est une table SYSTÈME : `seedTestState` ne la remet
 * pas à zéro. Le réglage est rendu à ses défauts en fin de spec (24 h, 7, activée), quelle
 * que soit l'issue.
 *
 * ⚠️ La tâche de fond du backend E2E écrit une sauvegarde 5 minutes après son démarrage :
 * si elle tourne à l'instant de « Sauvegarder maintenant », la route rend 409 — la spec
 * réessaie une fois.
 *
 * Les captures (AC40, thème clair) sont prises quand `KESH_E2E_CAPTURES_DIR` est posé. Les
 * états que l'écran n'atteint pas seul — statuts d'échec, « jamais tentée », bandeau,
 * conflit, refus de validation, sauvegarde en cours, 409, ligne trop grande — sont posés
 * par interception réseau (`page.route`) : la spec n'a pas d'accès direct à la base
 * (C-31-1-83).
 */

const ADMIN = { username: 'admin', password: 'admin123' };
const SCHEDULE_URL = '**/api/v1/admin/backup-schedule';

test.describe.configure({ mode: 'serial' });

test.beforeAll(async () => {
	await seedTestState('with-company');
});

test.afterEach(async ({ page }) => {
	await clearAuthStorage(page);
});

test.afterAll(async ({ browser }) => {
	// Réglage rendu aux défauts (la table survit aux `seedTestState`).
	const page = await browser.newPage();
	try {
		await login(page);
		const ctx = await authedApiContext(page);
		try {
			const etat = await (await ctx.get('/api/v1/admin/backup-schedule')).json();
			await ctx.put('/api/v1/admin/backup-schedule', {
				data: { enabled: true, intervalHours: 24, retentionCount: 7, version: etat.version },
			});
		} finally {
			await disposeContextSafe(ctx);
		}
	} finally {
		await page.close();
	}
});

async function login(page: Page, username = ADMIN.username, password = ADMIN.password) {
	await page.goto('/login');
	await page.fill('#username', username);
	await page.fill('#password', password);
	await page.click('button[type="submit"]');
	await expect(page).toHaveURL('/');
}

/** Capture (thème clair) si `KESH_E2E_CAPTURES_DIR` est posé ; sinon, rien. */
async function capture(page: Page, name: string, locator?: ReturnType<Page['locator']>) {
	const dossier = process.env.KESH_E2E_CAPTURES_DIR;
	if (!dossier) return;
	await fs.mkdir(dossier, { recursive: true });
	const chemin = path.join(dossier, `${name}.png`);
	if (locator) await locator.screenshot({ path: chemin });
	else await page.screenshot({ path: chemin, fullPage: true });
}

/**
 * Lit `manifest.json` d'une archive ZIP sans bibliothèque : parcours des en-têtes
 * locaux (`PK\x03\x04`), entrée `Stored` ou `Deflated` (l'exporteur écrit les tailles
 * dans l'en-tête local, l'écrivain étant positionnable).
 */
function manifestOf(zip: Buffer): Record<string, unknown> {
	let i = 0;
	while (i + 30 <= zip.length && zip.readUInt32LE(i) === 0x04034b50) {
		const methode = zip.readUInt16LE(i + 8);
		const taille = zip.readUInt32LE(i + 18);
		const longueurNom = zip.readUInt16LE(i + 26);
		const longueurExtra = zip.readUInt16LE(i + 28);
		const nom = zip.subarray(i + 30, i + 30 + longueurNom).toString('utf-8');
		const debut = i + 30 + longueurNom + longueurExtra;
		const donnees = zip.subarray(debut, debut + taille);
		if (nom === 'manifest.json') {
			const brut = methode === 8 ? inflateRawSync(donnees) : donnees;
			return JSON.parse(brut.toString('utf-8'));
		}
		i = debut + taille;
	}
	throw new Error('manifest.json introuvable dans l’archive');
}

/** Un état de `GET backup-schedule` pour l'interception. */
function etat(over: Record<string, unknown>) {
	return {
		enabled: true,
		intervalHours: 24,
		retentionCount: 7,
		version: 1,
		lastAttemptAt: '2026-10-24T01:00:00.000Z',
		lastSuccessAt: '2026-10-24T01:00:00.000Z',
		lastSuccessFile: 'kesh-auto-20261024T010000000Z.keshbackup',
		lastSuccessBytes: 48213,
		lastStatus: 'ok',
		lastFilesNotIncluded: 0,
		lastCompleteFile: 'kesh-auto-20261024T010000000Z.keshbackup',
		nextDueAt: '2026-10-25T01:00:00.000Z',
		overdue: false,
		...over,
	};
}

/** Remplace `GET backup-schedule` par `body` (les autres méthodes passent). */
async function poser(page: Page, body: Record<string, unknown>) {
	await page.unroute(SCHEDULE_URL);
	await page.route(SCHEDULE_URL, async (route: Route) => {
		if (route.request().method() !== 'GET') return route.fallback();
		await route.fulfill({ status: 200, contentType: 'application/json', body: JSON.stringify(body) });
	});
}

test('Administrateur : réglage gardé, « Sauvegarder maintenant », liste et téléchargement', async ({
	page,
}) => {
	await login(page);
	await page.goto('/admin/backup');
	const section = page.getByTestId('auto-backup-section');
	await expect(section).toBeVisible();

	// 1. Réglage : fréquence et nombre conservé, enregistrés, gardés au rechargement.
	await page.getByTestId('auto-backup-frequency').selectOption('12');
	await page.getByTestId('auto-backup-retention').fill('5');
	const enregistre = page.waitForResponse(
		(r) => r.url().endsWith('/api/v1/admin/backup-schedule') && r.request().method() === 'PUT',
	);
	await page.getByTestId('auto-backup-save').click();
	expect((await enregistre).status()).toBe(200);
	await page.reload();
	await expect(page.getByTestId('auto-backup-frequency')).toHaveValue('12');
	await expect(page.getByTestId('auto-backup-retention')).toHaveValue('5');

	// 2. « Sauvegarder maintenant » (un 409 de la tâche de fond se réessaie une fois).
	const lancer = async () => {
		const reponse = page.waitForResponse('**/api/v1/admin/backup-schedule/run');
		await page.getByTestId('auto-backup-run').click();
		return (await reponse).status();
	};
	let statut = await lancer();
	if (statut === 409) {
		await page.waitForTimeout(2000);
		statut = await lancer();
	}
	expect(statut).toBe(200);
	await expect(page.getByTestId('auto-backup-last-attempt')).toContainText('réussie');
	const ligne = page.getByTestId('server-backup-row').filter({ hasText: 'kesh-auto-' }).first();
	await expect(ligne).toBeVisible();
	await expect(ligne).toHaveAttribute('data-kind', 'auto');
	await capture(page, 'c-reussie', section);
	await capture(page, 'l-liste-remplie', page.getByTestId('server-backups-section'));

	// 3. Télécharger cette ligne : archive non vide, manifeste de format 2.
	const telechargement = page.waitForEvent('download');
	await ligne.getByTestId('server-backup-download').click();
	const fichier = await (await telechargement).path();
	const octets = await fs.readFile(fichier);
	expect(octets.length).toBeGreaterThan(0);
	expect(octets.readUInt32LE(0)).toBe(0x04034b50);
	expect(manifestOf(octets).formatVersion).toBe(2);
});

test('Comptable : ni l’entrée de menu, ni la page', async ({ page }) => {
	await login(page);
	const ctx = await authedApiContext(page);
	const comptable = `comptable-311c-${Date.now()}`;
	try {
		const cree = await ctx.post('/api/v1/users', {
			data: { username: comptable, password: 'MotDePasse12345', role: 'Comptable' },
		});
		expect(cree.ok(), `création du comptable : ${cree.status()}`).toBeTruthy();
	} finally {
		await disposeContextSafe(ctx);
	}
	await clearAuthStorage(page);
	await login(page, comptable, 'MotDePasse12345');
	await expect(page.locator('nav a[href="/admin/backup"]')).toHaveCount(0);
	await page.goto('/admin/backup');
	await expect(page).toHaveURL('/');
	await expect(page.getByTestId('backup-schedule-banner')).toHaveCount(0);
});

test('états de l’écran et du bandeau (captures de l’AC40)', async ({ page }) => {
	await login(page);
	await page.goto('/admin/backup');
	const section = page.getByTestId('auto-backup-section');
	await expect(section).toBeVisible();

	const etats: Array<[string, Record<string, unknown>, string]> = [
		[
			'a-jamais-tentee',
			{ lastAttemptAt: null, lastSuccessAt: null, lastStatus: null, lastSuccessBytes: null, lastSuccessFile: null, lastCompleteFile: null },
			'Jamais tentée',
		],
		['b-desactivee', { enabled: false, nextDueAt: null }, 'Sauvegarde automatique désactivée'],
		['d-partial', { lastStatus: 'partial', lastFilesNotIncluded: 2, lastSuccessFile: 'kesh-auto-20261024T020000000Z.keshbackup' }, 'pièces non incluses : 2'],
		['e-failed', { lastStatus: 'failed', lastAttemptAt: '2026-10-24T03:00:00.000Z' }, 'écriture ou relecture impossible'],
		['f-insufficient-space', { lastStatus: 'insufficient_space', lastAttemptAt: '2026-10-24T03:00:00.000Z' }, 'espace disque est insuffisant'],
		['g-disk-full', { lastStatus: 'disk_full', lastAttemptAt: '2026-10-24T03:00:00.000Z' }, 'quota'],
	];
	for (const [nom, over, texte] of etats) {
		await poser(page, etat(over));
		await page.reload();
		await expect(section).toContainText(texte);
		await capture(page, nom, section);
	}

	// Bandeau de l'application : échec (n), puis aucune réussite depuis deux intervalles (p).
	await poser(page, etat({ lastStatus: 'disk_full' }));
	await page.goto('/');
	const bandeau = page.getByTestId('backup-schedule-banner');
	await expect(bandeau).toHaveAttribute('data-kind', 'failed');
	await expect(bandeau).toHaveAttribute('role', 'status');
	await capture(page, 'n-bandeau-echec');
	await poser(page, etat({ overdue: true, lastSuccessAt: '2026-10-20T01:00:00.000Z' }));
	await page.reload();
	await expect(bandeau).toHaveAttribute('data-kind', 'overdue');
	await expect(bandeau).toContainText('Aucune sauvegarde automatique réussie depuis le');
	await capture(page, 'p-bandeau-aucune-reussite');
	// `partial` n'allume pas le bandeau (C-31-1-40).
	await poser(page, etat({ lastStatus: 'partial', lastFilesNotIncluded: 1 }));
	await page.reload();
	await expect(page.getByTestId('auto-backup-section')).toHaveCount(0);
	await expect(bandeau).toHaveCount(0);
	await page.unroute(SCHEDULE_URL);

	// (h) en cours et (i) 409 : états d'un verrou en mémoire, par interception de POST …/run.
	await page.goto('/admin/backup');
	await expect(section).toBeVisible();
	let liberer!: () => void;
	const retenue = new Promise<void>((r) => (liberer = r));
	await page.route('**/api/v1/admin/backup-schedule/run', async (route) => {
		await retenue;
		await route.fulfill({
			status: 409,
			contentType: 'application/json',
			body: JSON.stringify({
				error: {
					code: 'BACKUP_IN_PROGRESS',
					message: 'Une sauvegarde ou une restauration est déjà en cours. Réessayez dans quelques instants.',
				},
			}),
		});
	});
	await page.getByTestId('auto-backup-run').click();
	await expect(page.getByTestId('auto-backup-run')).toBeDisabled();
	await expect(page.getByTestId('auto-backup-run')).toContainText('Sauvegarde en cours');
	await capture(page, 'h-en-cours', section);
	liberer();
	await expect(page.getByTestId('auto-backup-run-error')).toContainText('déjà en cours');
	await capture(page, 'i-409-en-cours', section);
	await page.unroute('**/api/v1/admin/backup-schedule/run');

	// (j) conflit de version et (k) refus de validation, par interception de PUT.
	const refus = async (status: number, code: string, message: string) => {
		await page.unroute(SCHEDULE_URL);
		await page.route(SCHEDULE_URL, async (route) => {
			if (route.request().method() !== 'PUT') return route.fallback();
			await route.fulfill({
				status,
				contentType: 'application/json',
				body: JSON.stringify({ error: { code, message } }),
			});
		});
		await page.getByTestId('auto-backup-save').click();
	};
	await refus(409, 'OPTIMISTIC_LOCK_CONFLICT', 'Conflit de version');
	await expect(page.getByTestId('auto-backup-conflict')).toBeVisible();
	await expect(page.getByTestId('auto-backup-reload')).toBeVisible();
	await capture(page, 'j-conflit-de-version', section);
	await refus(
		400,
		'VALIDATION_ERROR',
		'Réglage de sauvegarde hors domaine : fréquence 6, 12, 24 ou 168 heures, de 1 à 60 sauvegardes conservées.',
	);
	await expect(page.getByTestId('auto-backup-save-error')).toContainText('hors domaine');
	await capture(page, 'k-erreur-de-validation', section);
	await page.unroute(SCHEDULE_URL);

	// (m) liste vide et (r) ligne trop grande, par interception de GET backups.
	const liste = async (body: Record<string, unknown>) => {
		await page.unroute('**/api/v1/admin/backups');
		await page.route('**/api/v1/admin/backups', (route) =>
			route.fulfill({ status: 200, contentType: 'application/json', body: JSON.stringify(body) }),
		);
		await page.reload();
	};
	await liste({ importMaxMib: 2048, items: [] });
	await expect(page.getByTestId('server-backups-empty')).toBeVisible();
	await capture(page, 'm-liste-vide', page.getByTestId('server-backups-section'));
	await liste({
		importMaxMib: 2048,
		items: [
			{ name: 'kesh-auto-20261024T010000000Z.keshbackup', kind: 'auto', sizeBytes: 2049 * 1_048_576, modifiedAt: '2026-10-24T01:00:00.000Z' },
			{ name: 'kesh-pre-import-20261023T090000000-1-0.keshbackup', kind: 'pre-import', sizeBytes: 52_000, modifiedAt: '2026-10-23T09:00:00.000Z' },
		],
	});
	await expect(page.getByTestId('server-backup-too-large')).toHaveCount(1);
	await expect(page.getByTestId('server-backup-too-large')).toContainText('limite 2048 Mio');
	await capture(page, 'r-trop-grande', page.getByTestId('server-backups-section'));

	// (q) la section à largeur étroite (la section seule : la barre latérale fixe du layout,
	// antérieure à la story, déborde à cette largeur).
	await page.setViewportSize({ width: 390, height: 1400 });
	await page.reload();
	await expect(section).toBeVisible();
	await capture(page, 'q-largeur-etroite', section);
	await page.unroute('**/api/v1/admin/backups');
});
