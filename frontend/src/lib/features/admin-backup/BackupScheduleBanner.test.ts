// Story 31-1c (#503, AC31 (b), AC37) — le bandeau de l'application :
// Administrateur seulement, sauvegarde activée, échec ou retard dit par le
// serveur ; jamais pour `partial` ; jamais si la PREMIÈRE lecture échoue — un
// échec de lecture ultérieur garde le dernier état lu (Story 32-1b, #711 E-8).
// Story 32-1b (#711 L-B5) : une réussite datée du futur se dit telle.
import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen, waitFor } from '@testing-library/svelte';

const { auth } = vi.hoisted(() => ({
	auth: { currentUser: { role: 'Admin' } as { role: string } | null },
}));

vi.mock('$lib/app/stores/auth.svelte', () => ({ authState: auth }));
vi.mock('./backup-schedule.api', async (importOriginal) => {
	const original = await importOriginal<typeof import('./backup-schedule.api')>();
	return { ...original, fetchBackupSchedule: vi.fn() };
});
vi.mock('$lib/shared/utils/i18n.svelte', () => ({
	i18nMsg: (_key: string, fallback: string, args?: Record<string, string | number>) =>
		args ? fallback.replace(/\{\s*\$(\w+)\s*\}/g, (_m, k: string) => String(args[k] ?? '')) : fallback,
	i18nLocale: () => 'fr-CH',
}));

import * as api from './backup-schedule.api';
import type { BackupSchedule } from './backup-schedule.api';
import { refreshBackupScheduleState, setBackupScheduleState } from './backup-schedule.svelte';
import BackupScheduleBanner from './BackupScheduleBanner.svelte';

const fetchMock = vi.mocked(api.fetchBackupSchedule);

function schedule(over: Partial<BackupSchedule> = {}): BackupSchedule {
	return {
		enabled: true,
		intervalHours: 24,
		retentionCount: 7,
		version: 1,
		lastAttemptAt: '2026-10-20T01:00:00.000Z',
		lastSuccessAt: '2026-10-20T01:00:00.000Z',
		lastSuccessFile: 'kesh-auto-20261020T010000000Z.keshbackup',
		lastSuccessBytes: 10,
		lastStatus: 'ok',
		lastFilesNotIncluded: 0,
		lastCompleteFile: 'kesh-auto-20261020T010000000Z.keshbackup',
		nextDueAt: '2026-10-21T01:00:00.000Z',
		overdue: false,
		lastSuccessInFuture: false,
		...over,
	};
}

async function afficher(s: BackupSchedule | Error) {
	if (s instanceof Error) fetchMock.mockRejectedValue(s);
	else fetchMock.mockResolvedValue(s);
	const r = render(BackupScheduleBanner);
	if (auth.currentUser?.role === 'Admin') await waitFor(() => expect(fetchMock).toHaveBeenCalled());
	await Promise.resolve();
	return r;
}

describe('BackupScheduleBanner — Story 31-1c', () => {
	beforeEach(() => {
		vi.clearAllMocks();
		auth.currentUser = { role: 'Admin' };
		setBackupScheduleState(null);
	});

	it('échec : affiché, role="status", lien vers le panneau', async () => {
		await afficher(schedule({ lastStatus: 'disk_full' }));
		const bandeau = await screen.findByTestId('backup-schedule-banner');
		expect(bandeau.getAttribute('role')).toBe('status');
		expect(bandeau.getAttribute('data-kind')).toBe('failed');
		expect(bandeau.textContent).toContain('a échoué');
		expect(screen.getByTestId('backup-schedule-banner-link').getAttribute('href')).toBe(
			'/admin/backup',
		);
		expect(screen.queryByRole('alert')).toBeNull();
	});

	it('retard dit par le serveur : « aucune sauvegarde réussie depuis le … »', async () => {
		await afficher(schedule({ overdue: true }));
		const bandeau = await screen.findByTestId('backup-schedule-banner');
		expect(bandeau.getAttribute('data-kind')).toBe('overdue');
		expect(bandeau.textContent).toContain('Aucune sauvegarde automatique réussie depuis le');
		expect(bandeau.textContent).toContain('2026');
	});

	it('ok et pas en retard : absent (aucune date comparée à l’horloge du navigateur)', async () => {
		await afficher(schedule({ lastSuccessAt: '2001-01-01T00:00:00.000Z', overdue: false }));
		expect(screen.queryByTestId('backup-schedule-banner')).toBeNull();
	});

	it('partial : absent', async () => {
		await afficher(schedule({ lastStatus: 'partial', lastFilesNotIncluded: 3 }));
		expect(screen.queryByTestId('backup-schedule-banner')).toBeNull();
	});

	it('désactivée : absent même sur un échec', async () => {
		await afficher(schedule({ enabled: false, lastStatus: 'failed', overdue: true }));
		expect(screen.queryByTestId('backup-schedule-banner')).toBeNull();
	});

	it('jamais tentée : absent', async () => {
		await afficher(
			schedule({ lastAttemptAt: null, lastSuccessAt: null, lastStatus: null, overdue: false }),
		);
		expect(screen.queryByTestId('backup-schedule-banner')).toBeNull();
	});

	it('lecture en échec : absent, sans erreur', async () => {
		await afficher(new Error('réseau'));
		expect(screen.queryByTestId('backup-schedule-banner')).toBeNull();
	});

	it('Comptable : absent, et l’état n’est pas lu', async () => {
		auth.currentUser = { role: 'Comptable' };
		await afficher(schedule({ lastStatus: 'failed' }));
		expect(screen.queryByTestId('backup-schedule-banner')).toBeNull();
		expect(fetchMock).not.toHaveBeenCalled();
	});

	it('suit l’état posé par une action du panneau', async () => {
		await afficher(schedule());
		expect(screen.queryByTestId('backup-schedule-banner')).toBeNull();
		setBackupScheduleState(schedule({ lastStatus: 'failed' }));
		expect(await screen.findByTestId('backup-schedule-banner')).toBeTruthy();
	});
});

describe('BackupScheduleBanner — Story 32-1b (#711 E-8, L-B5)', () => {
	beforeEach(() => {
		vi.clearAllMocks();
		auth.currentUser = { role: 'Admin' };
		setBackupScheduleState(null);
	});

	it('E-8 (i) : un sondage en échec garde le dernier état lu', async () => {
		await afficher(schedule({ lastStatus: 'failed' }));
		expect((await screen.findByTestId('backup-schedule-banner')).getAttribute('data-kind')).toBe(
			'failed',
		);
		fetchMock.mockRejectedValue(new Error('réseau'));
		await refreshBackupScheduleState();
		expect(fetchMock).toHaveBeenCalledTimes(2);
		await Promise.resolve();
		expect(screen.getByTestId('backup-schedule-banner').getAttribute('data-kind')).toBe('failed');
	});

	it('E-8 (ii) : une réponse tardive ne remplace pas l’état posé par le panneau', async () => {
		let rendre!: (s: BackupSchedule) => void;
		fetchMock.mockImplementation(() => new Promise((r) => (rendre = r)));
		render(BackupScheduleBanner);
		await waitFor(() => expect(fetchMock).toHaveBeenCalled());
		setBackupScheduleState(schedule());
		rendre(schedule({ lastStatus: 'failed' }));
		await new Promise((r) => setTimeout(r, 0));
		expect(screen.queryByTestId('backup-schedule-banner')).toBeNull();
	});

	it('L-B5 : réussite datée après l’heure du serveur → le bandeau le dit, avec la date', async () => {
		await afficher(
			schedule({
				overdue: true,
				lastSuccessInFuture: true,
				lastSuccessAt: '2027-03-01T08:00:00.000Z',
			}),
		);
		const bandeau = await screen.findByTestId('backup-schedule-banner');
		expect(bandeau.getAttribute('data-kind')).toBe('overdue');
		expect(bandeau.textContent).toContain("après l'heure du serveur : son horloge a reculé");
		expect(bandeau.textContent).toContain('2027');
		expect(bandeau.textContent).not.toContain('Aucune sauvegarde automatique réussie depuis le');
	});

	it('L-B5 : en retard sans réussite future → le texte du retard', async () => {
		await afficher(schedule({ overdue: true, lastSuccessInFuture: false }));
		const bandeau = await screen.findByTestId('backup-schedule-banner');
		expect(bandeau.textContent).toContain('Aucune sauvegarde automatique réussie depuis le');
		expect(bandeau.textContent).not.toContain('horloge');
	});
});
