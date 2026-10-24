// Story 31-1c (#503, AC33, AC37) — la section « Sauvegarde automatique » et la
// liste des sauvegardes du serveur : états, enregistrement, conflit,
// « Sauvegarder maintenant », liste, téléchargement, mention « trop grande ».
// Story 32-1b (#711 E-6, E-7, L-B6) : la saisie non enregistrée survit à
// « Sauvegarder maintenant » sans défaire le verrou optimiste ; une réponse qui
// manque (ou un 5xx) relit l'état et la liste ; la limite dite en deux unités.
import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { render, fireEvent, waitFor, screen } from '@testing-library/svelte';

vi.mock('./backup-schedule.api', async (importOriginal) => {
	const original = await importOriginal<typeof import('./backup-schedule.api')>();
	return {
		...original,
		fetchBackupSchedule: vi.fn(),
		saveBackupSchedule: vi.fn(),
		runBackupNow: vi.fn(),
		fetchServerBackups: vi.fn(),
		downloadServerBackup: vi.fn(),
	};
});
vi.mock('$lib/shared/utils/i18n.svelte', () => ({
	i18nMsg: (_key: string, fallback: string, args?: Record<string, string | number>) =>
		args ? fallback.replace(/\{\s*\$(\w+)\s*\}/g, (_m, k: string) => String(args[k] ?? '')) : fallback,
	i18nLocale: () => 'fr-CH',
}));
vi.mock('svelte-sonner', () => ({ toast: { success: vi.fn(), error: vi.fn() } }));

import * as api from './backup-schedule.api';
import type { BackupSchedule, ServerBackups } from './backup-schedule.api';
import AutoBackupSection from './AutoBackupSection.svelte';
import { formatBackupDate } from './backup-format';

const mock = vi.mocked(api);

// Story 32-1b : les réponses `…Once` non consommées d'un test ne doivent pas
// fuir dans le suivant (`clearAllMocks` garde les implémentations en file).
afterEach(() => vi.resetAllMocks());

function schedule(over: Partial<BackupSchedule> = {}): BackupSchedule {
	return {
		enabled: true,
		intervalHours: 24,
		retentionCount: 7,
		version: 3,
		lastAttemptAt: '2026-10-24T01:00:00.000Z',
		lastSuccessAt: '2026-10-24T01:00:00.000Z',
		lastSuccessFile: 'kesh-auto-20261024T010000000Z.keshbackup',
		lastSuccessBytes: 4096,
		lastStatus: 'ok',
		lastFilesNotIncluded: 0,
		lastCompleteFile: 'kesh-auto-20261024T010000000Z.keshbackup',
		nextDueAt: '2026-10-25T01:00:00.000Z',
		overdue: false,
		lastSuccessInFuture: false,
		...over,
	};
}

function liste(items: ServerBackups['items'] = [], importMaxMib = 7): ServerBackups {
	return { importMaxMib, items };
}

const AUTO = {
	name: 'kesh-auto-20261024T010000000Z.keshbackup',
	kind: 'auto' as const,
	sizeBytes: 4096,
	modifiedAt: '2026-10-24T01:00:00.000Z',
};

async function monter(s: BackupSchedule, b: ServerBackups = liste([AUTO])) {
	mock.fetchBackupSchedule.mockResolvedValue(s);
	mock.fetchServerBackups.mockResolvedValue(b);
	const r = render(AutoBackupSection);
	await screen.findByTestId('auto-backup-state');
	return r;
}

describe('AutoBackupSection — états (Story 31-1c, AC33)', () => {
	beforeEach(() => vi.clearAllMocks());

	it('réussie : aucun encart d’échec ni d’avertissement', async () => {
		await monter(schedule());
		expect(screen.queryByRole('alert')).toBeNull();
		expect(screen.queryByTestId('auto-backup-partial')).toBeNull();
		expect(screen.getByTestId('auto-backup-last-attempt').textContent).toContain('réussie');
	});

	it('partial : avertissement role="status" avec le nombre, aucun role="alert"', async () => {
		await monter(schedule({ lastStatus: 'partial', lastFilesNotIncluded: 2 }));
		const avis = screen.getByTestId('auto-backup-partial');
		expect(avis.getAttribute('role')).toBe('status');
		expect(avis.textContent).toContain('pièces non incluses : 2');
		expect(screen.queryByRole('alert')).toBeNull();
	});

	for (const [status, texte] of [
		['failed', 'écriture ou relecture impossible'],
		['insufficient_space', 'supprimant d\'anciennes sauvegardes de ./backup/auto'],
		['disk_full', 'disque ou le quota'],
	] as const) {
		it(`échec ${status} : bandeau d’erreur role="alert" et sa cause`, async () => {
			await monter(schedule({ lastStatus: status }));
			const encart = screen.getByTestId('auto-backup-failure');
			expect(encart.getAttribute('role')).toBe('alert');
			expect(encart.getAttribute('data-status')).toBe(status);
			expect(encart.textContent).toContain(texte);
		});
	}

	it('jamais tentée', async () => {
		await monter(
			schedule({ lastAttemptAt: null, lastSuccessAt: null, lastStatus: null, lastSuccessBytes: null }),
		);
		expect(screen.getByTestId('auto-backup-last-attempt').textContent).toContain('Jamais tentée');
		expect(screen.getByTestId('auto-backup-last-success').textContent).toContain('Aucune');
	});

	it('désactivée : la prochaine sauvegarde ne s’affiche pas', async () => {
		await monter(schedule({ enabled: false, nextDueAt: null }));
		expect(screen.getByTestId('auto-backup-next').textContent).toContain('désactivée');
		expect((screen.getByTestId('auto-backup-enabled') as HTMLInputElement).checked).toBe(false);
	});

	it('dernière complète absente de la liste du serveur → « introuvable »', async () => {
		await monter(
			schedule({
				lastStatus: 'partial',
				lastFilesNotIncluded: 1,
				lastSuccessFile: 'kesh-auto-20261024T020000000Z.keshbackup',
				lastCompleteFile: 'kesh-auto-20261020T010000000Z.keshbackup',
			}),
		);
		await waitFor(() =>
			expect(screen.getByTestId('auto-backup-last-complete-missing')).toBeTruthy(),
		);
	});

	it('dernière complète présente dans la liste → pas de mention', async () => {
		const ancienne = { ...AUTO, name: 'kesh-auto-20261020T010000000Z.keshbackup' };
		await monter(
			schedule({
				lastSuccessFile: 'kesh-auto-20261024T020000000Z.keshbackup',
				lastCompleteFile: ancienne.name,
			}),
			liste([AUTO, ancienne]),
		);
		await screen.findByTestId('server-backups-list');
		expect(screen.getByTestId('auto-backup-last-complete')).toBeTruthy();
		expect(screen.queryByTestId('auto-backup-last-complete-missing')).toBeNull();
	});
});

describe('AutoBackupSection — enregistrer (AC33)', () => {
	beforeEach(() => vi.clearAllMocks());

	it('enregistre le réglage saisi avec la version lue', async () => {
		await monter(schedule());
		mock.saveBackupSchedule.mockResolvedValue(schedule({ version: 4, intervalHours: 6 }));
		await fireEvent.change(screen.getByTestId('auto-backup-frequency'), { target: { value: '6' } });
		await fireEvent.input(screen.getByTestId('auto-backup-retention'), { target: { value: '12' } });
		await fireEvent.click(screen.getByTestId('auto-backup-enabled'));
		await fireEvent.click(screen.getByTestId('auto-backup-save'));
		expect(mock.saveBackupSchedule).toHaveBeenCalledWith({
			enabled: false,
			intervalHours: 6,
			retentionCount: 12,
			version: 3,
		});
	});

	it('409 : conflit affiché, « Recharger » relit le réglage', async () => {
		await monter(schedule());
		mock.saveBackupSchedule.mockRejectedValue({
			code: 'OPTIMISTIC_LOCK_CONFLICT',
			status: 409,
			message: 'conflit',
		});
		await fireEvent.click(screen.getByTestId('auto-backup-save'));
		const encart = await screen.findByTestId('auto-backup-conflict');
		expect(encart.getAttribute('role')).toBe('alert');
		mock.fetchBackupSchedule.mockResolvedValue(schedule({ version: 9 }));
		await fireEvent.click(screen.getByTestId('auto-backup-reload'));
		await waitFor(() => expect(screen.queryByTestId('auto-backup-conflict')).toBeNull());
		expect(mock.fetchBackupSchedule).toHaveBeenCalledTimes(2);
	});

	it('400 : le message du serveur est affiché', async () => {
		await monter(schedule());
		mock.saveBackupSchedule.mockRejectedValue({
			code: 'VALIDATION_ERROR',
			status: 400,
			message: 'Réglage hors domaine',
		});
		await fireEvent.click(screen.getByTestId('auto-backup-save'));
		expect((await screen.findByTestId('auto-backup-save-error')).textContent).toContain(
			'Réglage hors domaine',
		);
	});
});

describe('AutoBackupSection — « Sauvegarder maintenant » (AC33)', () => {
	beforeEach(() => vi.clearAllMocks());

	it('désactivé pendant l’exécution, un seul appel, la liste est relue', async () => {
		await monter(schedule());
		let fin!: (s: BackupSchedule) => void;
		mock.runBackupNow.mockImplementation(() => new Promise((r) => (fin = r)));
		const bouton = screen.getByTestId('auto-backup-run') as HTMLButtonElement;
		await fireEvent.click(bouton);
		expect(bouton.disabled).toBe(true);
		expect(bouton.textContent).toContain('Sauvegarde en cours');
		await fireEvent.click(bouton);
		expect(mock.runBackupNow).toHaveBeenCalledTimes(1);
		fin(schedule({ version: 3 }));
		await waitFor(() => expect(bouton.disabled).toBe(false));
		expect(mock.fetchServerBackups).toHaveBeenCalledTimes(2);
	});

	it('409 BACKUP_IN_PROGRESS : le message est affiché', async () => {
		await monter(schedule());
		mock.runBackupNow.mockRejectedValue({
			code: 'BACKUP_IN_PROGRESS',
			status: 409,
			message: 'Une sauvegarde ou une restauration est déjà en cours.',
		});
		await fireEvent.click(screen.getByTestId('auto-backup-run'));
		expect((await screen.findByTestId('auto-backup-run-error')).textContent).toContain(
			'déjà en cours',
		);
		// 4xx : le serveur a refusé, rien n'a été écrit — aucune relecture (AC-b2).
		expect(mock.fetchBackupSchedule).toHaveBeenCalledTimes(1);
		expect(mock.fetchServerBackups).toHaveBeenCalledTimes(1);
	});
});

/** La tentative en cours vue par une relecture : date neuve, statut de la précédente. */
const NOUVELLE_TENTATIVE = '2026-10-26T03:00:00.000Z';
/** Sa date telle que l'écran l'affiche (comparer « 26 » seul passerait sur « 2026 »). */
const NOUVELLE_TENTATIVE_AFFICHEE = formatBackupDate(NOUVELLE_TENTATIVE, 'fr-CH');

function selecteur(): HTMLSelectElement {
	return screen.getByTestId('auto-backup-frequency') as HTMLSelectElement;
}

function conserve(): HTMLInputElement {
	return screen.getByTestId('auto-backup-retention') as HTMLInputElement;
}

async function lancer(): Promise<void> {
	await fireEvent.click(screen.getByTestId('auto-backup-run'));
	await waitFor(() =>
		expect((screen.getByTestId('auto-backup-run') as HTMLButtonElement).disabled).toBe(false),
	);
}

describe('AutoBackupSection — saisie et « Sauvegarder maintenant » (Story 32-1b, #711 E-6)', () => {
	beforeEach(() => vi.clearAllMocks());

	it('(i) la fréquence modifiée sans enregistrer survit au run ; « Enregistrer » envoie la version lue', async () => {
		await monter(schedule());
		await fireEvent.change(selecteur(), { target: { value: '12' } });
		mock.runBackupNow.mockResolvedValue(
			schedule({ lastAttemptAt: NOUVELLE_TENTATIVE, lastSuccessAt: NOUVELLE_TENTATIVE }),
		);
		await lancer();
		expect(selecteur().value).toBe('12');
		expect(screen.getByTestId('auto-backup-last-attempt').textContent).toContain(
			NOUVELLE_TENTATIVE_AFFICHEE,
		);
		expect(screen.getByTestId('auto-backup-last-attempt').textContent).toContain('réussie');
		mock.saveBackupSchedule.mockResolvedValue(schedule({ version: 4, intervalHours: 12 }));
		await fireEvent.click(screen.getByTestId('auto-backup-save'));
		expect(mock.saveBackupSchedule).toHaveBeenCalledWith({
			enabled: true,
			intervalHours: 12,
			retentionCount: 7,
			version: 3,
		});
	});

	it('(ii) sans saisie, le formulaire suit le réglage rendu par le run', async () => {
		await monter(schedule());
		mock.runBackupNow.mockResolvedValue(schedule({ retentionCount: 9 }));
		await lancer();
		expect(conserve().value).toBe('9');
	});

	it('(iii) concurrence : la saisie est gardée et s’enregistre contre la version lue → 409', async () => {
		await monter(schedule());
		await fireEvent.input(conserve(), { target: { value: '12' } });
		mock.runBackupNow.mockResolvedValue(schedule({ version: 4, retentionCount: 10 }));
		await lancer();
		expect(conserve().value).toBe('12');
		mock.saveBackupSchedule.mockRejectedValue({
			code: 'OPTIMISTIC_LOCK_CONFLICT',
			status: 409,
			message: 'conflit',
		});
		await fireEvent.click(screen.getByTestId('auto-backup-save'));
		expect(mock.saveBackupSchedule).toHaveBeenCalledWith({
			enabled: true,
			intervalHours: 24,
			retentionCount: 12,
			version: 3,
		});
		expect(await screen.findByTestId('auto-backup-conflict')).toBeTruthy();
	});
});

describe('AutoBackupSection — réponse manquante ou en erreur (Story 32-1b, #711 E-7)', () => {
	beforeEach(() => vi.clearAllMocks());

	const PAS_PARVENUE = "La réponse du serveur n'est pas parvenue";

	for (const [nom, rejet] of [
		['TIMEOUT', { code: 'TIMEOUT', status: 0, message: 'Le serveur ne répond pas.' }],
		['NETWORK_ERROR', { code: 'NETWORK_ERROR', status: 0, message: 'Coupé.' }],
		['502', { code: 'UNKNOWN_ERROR', status: 502, message: 'Bad Gateway' }],
		['504', { code: 'UNKNOWN_ERROR', status: 504, message: 'Gateway Timeout' }],
		['rejet hors ApiError', new TypeError('x')],
	] as const) {
		it(`${nom} : message « pas parvenue », état et liste relus`, async () => {
			await monter(schedule());
			mock.runBackupNow.mockRejectedValue(rejet);
			await lancer();
			expect(screen.getByTestId('auto-backup-run-error').textContent).toContain(PAS_PARVENUE);
			expect(mock.fetchBackupSchedule).toHaveBeenCalledTimes(2);
			expect(mock.fetchServerBackups).toHaveBeenCalledTimes(2);
		});
	}

	it('200 au corps illisible (`undefined`) : message « pas parvenue », relecture faite', async () => {
		await monter(schedule());
		mock.runBackupNow.mockResolvedValue(undefined as unknown as BackupSchedule);
		await lancer();
		expect(screen.getByTestId('auto-backup-run-error').textContent).toContain(PAS_PARVENUE);
		expect(mock.fetchBackupSchedule).toHaveBeenCalledTimes(2);
		expect(mock.fetchServerBackups).toHaveBeenCalledTimes(2);
		// Le panneau reste affiché (le réglage relu, non un état vide).
		expect(screen.getByTestId('auto-backup-state')).toBeTruthy();
	});

	it('TIMEOUT pendant l’écriture : la relecture montre la tentative en cours, le message dit « s’écrit encore »', async () => {
		await monter(schedule());
		mock.runBackupNow.mockRejectedValue({ code: 'TIMEOUT', status: 0, message: 'délai' });
		mock.fetchBackupSchedule.mockResolvedValue(schedule({ lastAttemptAt: NOUVELLE_TENTATIVE }));
		await lancer();
		expect(screen.getByTestId('auto-backup-run-error').textContent).toContain("ou s'écrit encore");
		const tentative = screen.getByTestId('auto-backup-last-attempt').textContent ?? '';
		expect(tentative).toContain(NOUVELLE_TENTATIVE_AFFICHEE);
		// La tentative en cours avec le statut de la précédente (C-32-1-30, #725).
		expect(tentative).toContain('réussie');
	});

	it('500 : le message du serveur, état et liste relus', async () => {
		await monter(schedule());
		mock.runBackupNow.mockRejectedValue({
			code: 'INTERNAL_ERROR',
			status: 500,
			message: 'Erreur interne',
		});
		await lancer();
		const texte = screen.getByTestId('auto-backup-run-error').textContent ?? '';
		expect(texte).toContain('Erreur interne');
		expect(texte).not.toContain(PAS_PARVENUE);
		expect(mock.fetchBackupSchedule).toHaveBeenCalledTimes(2);
		expect(mock.fetchServerBackups).toHaveBeenCalledTimes(2);
	});

	it('403 : le message du serveur, aucune relecture', async () => {
		await monter(schedule());
		mock.runBackupNow.mockRejectedValue({ code: 'FORBIDDEN', status: 403, message: 'Accès refusé' });
		await lancer();
		const texte = screen.getByTestId('auto-backup-run-error').textContent ?? '';
		expect(texte).toContain('Accès refusé');
		expect(texte).not.toContain(PAS_PARVENUE);
		expect(mock.fetchBackupSchedule).toHaveBeenCalledTimes(1);
		expect(mock.fetchServerBackups).toHaveBeenCalledTimes(1);
	});

	it('relecture en échec : alerte de lecture ; un run suivant réussi l’efface', async () => {
		await monter(schedule());
		mock.runBackupNow.mockRejectedValueOnce({ code: 'TIMEOUT', status: 0, message: 'délai' });
		mock.fetchBackupSchedule.mockRejectedValueOnce({
			code: 'NETWORK_ERROR',
			status: 0,
			message: 'Impossible de contacter le serveur.',
		});
		await lancer();
		expect(screen.getByTestId('auto-backup-run-error').textContent).toContain(PAS_PARVENUE);
		expect(screen.getByTestId('auto-backup-load-error')).toBeTruthy();
		// Le réglage affiché reste le précédent.
		expect(screen.getByTestId('auto-backup-state')).toBeTruthy();
		mock.runBackupNow.mockResolvedValueOnce(schedule());
		await lancer();
		expect(screen.queryByTestId('auto-backup-load-error')).toBeNull();
		expect(screen.queryByTestId('auto-backup-run-error')).toBeNull();
	});
});

describe('AutoBackupSection — sauvegardes du serveur (AC33, AC34)', () => {
	beforeEach(() => vi.clearAllMocks());

	it('liste vide', async () => {
		await monter(schedule(), liste([]));
		expect(await screen.findByTestId('server-backups-empty')).toBeTruthy();
	});

	it('télécharge la ligne choisie', async () => {
		await monter(schedule());
		mock.downloadServerBackup.mockResolvedValue();
		await fireEvent.click(await screen.findByTestId('server-backup-download'));
		expect(mock.downloadServerBackup).toHaveBeenCalledWith(AUTO);
	});

	it('mention « trop grande » au-dessus du plafond seulement (égal, au-dessous : aucune)', async () => {
		const plafond = 7 * 1_048_576;
		await monter(
			schedule(),
			liste([
				{ ...AUTO, name: 'kesh-auto-20261024T030000000Z.keshbackup', sizeBytes: plafond + 1 },
				{ ...AUTO, name: 'kesh-auto-20261024T020000000Z.keshbackup', sizeBytes: plafond },
				{ ...AUTO, sizeBytes: 1024 },
			]),
		);
		const lignes = await screen.findAllByTestId('server-backup-row');
		expect(lignes).toHaveLength(3);
		const mentions = lignes.map((l) => l.querySelector('[data-testid="server-backup-too-large"]'));
		expect(mentions[0]?.textContent).toContain('limite 7 Mio');
		// Story 32-1b (#711 L-B6) : l'équivalent décimal, dans l'unité des tailles listées —
		// écrit tel qu'observé au run : `Intl` sépare le nombre de l'unité par une espace
		// fine insécable (U+202F) en fr-CH.
		expect(mentions[0]?.textContent).toContain('limite 7 Mio, soit 7,3 Mo');
		expect(mentions[1]).toBeNull();
		expect(mentions[2]).toBeNull();
	});
});
