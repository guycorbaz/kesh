// Story 31-1c (#503, AC32, AC33, AC37) — API de la sauvegarde automatique :
// appels (chemins, corps, options longues), comparaison au plafond d'import,
// choix du bandeau de l'application.
import { describe, it, expect, vi, beforeEach } from 'vitest';

vi.mock('$lib/shared/utils/api-client', () => ({
	apiClient: { get: vi.fn(), put: vi.fn(), post: vi.fn(), getBlob: vi.fn() },
	LONG_OPERATION_TIMEOUT_MS: 3_600_000,
}));
vi.mock('$lib/shared/utils/download', () => ({
	parseContentDispositionFilename: vi.fn(() => 'servi.keshbackup'),
	triggerDownload: vi.fn(),
}));

import { apiClient } from '$lib/shared/utils/api-client';
import { triggerDownload } from '$lib/shared/utils/download';
import {
	bannerKind,
	downloadServerBackup,
	exceedsImportLimit,
	fetchBackupSchedule,
	fetchServerBackups,
	runBackupNow,
	saveBackupSchedule,
	type BackupSchedule,
} from './backup-schedule.api';

const client = vi.mocked(apiClient);

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
		...over,
	};
}

describe('backup-schedule.api — appels (Story 31-1c)', () => {
	beforeEach(() => vi.clearAllMocks());

	it('lit le réglage et la liste aux bons chemins', async () => {
		client.get.mockResolvedValue({});
		await fetchBackupSchedule();
		await fetchServerBackups();
		expect(client.get).toHaveBeenNthCalledWith(1, '/api/v1/admin/backup-schedule');
		expect(client.get).toHaveBeenNthCalledWith(2, '/api/v1/admin/backups');
	});

	it('enregistre le réglage avec le corps exact (aucun champ en plus)', async () => {
		client.put.mockResolvedValue(schedule());
		await saveBackupSchedule({ enabled: false, intervalHours: 168, retentionCount: 12, version: 3 });
		expect(client.put).toHaveBeenCalledWith('/api/v1/admin/backup-schedule', {
			enabled: false,
			intervalHours: 168,
			retentionCount: 12,
			version: 3,
		});
	});

	it('« Sauvegarder maintenant » : appel long, sans rejeu', async () => {
		client.post.mockResolvedValue(schedule());
		await runBackupNow();
		expect(client.post).toHaveBeenCalledWith('/api/v1/admin/backup-schedule/run', undefined, {
			timeoutMs: 3_600_000,
			retry: false,
		});
	});

	it('télécharge une sauvegarde du serveur par son type et son nom, appel long', async () => {
		client.getBlob.mockResolvedValue({
			blob: () => Promise.resolve(new Blob([new Uint8Array([0x50, 0x4b, 3, 4])])),
			headers: new Headers({ 'Content-Disposition': 'attachment' }),
		} as unknown as Response);
		await downloadServerBackup({
			name: 'kesh-auto-20261024T010000000Z.keshbackup',
			kind: 'auto',
			sizeBytes: 4,
			modifiedAt: null,
		});
		expect(client.getBlob).toHaveBeenCalledWith(
			'/api/v1/admin/backups/auto/kesh-auto-20261024T010000000Z.keshbackup',
			{ timeoutMs: 3_600_000, retry: false },
		);
		expect(vi.mocked(triggerDownload)).toHaveBeenCalledTimes(1);
	});

	it('refuse une réponse vide', async () => {
		client.getBlob.mockResolvedValue({
			blob: () => Promise.resolve(new Blob([])),
			headers: new Headers(),
		} as unknown as Response);
		await expect(
			downloadServerBackup({ name: 'x', kind: 'auto', sizeBytes: 0, modifiedAt: null }),
		).rejects.toThrow();
		expect(vi.mocked(triggerDownload)).not.toHaveBeenCalled();
	});
});

describe('exceedsImportLimit — comparaison stricte au plafond (C-31-1-52)', () => {
	it('au-dessus du plafond → vrai', () => {
		expect(exceedsImportLimit(7 * 1_048_576 + 1, 7)).toBe(true);
	});
	it('égal au plafond → faux', () => {
		expect(exceedsImportLimit(7 * 1_048_576, 7)).toBe(false);
	});
	it('au-dessous → faux', () => {
		expect(exceedsImportLimit(1024, 7)).toBe(false);
	});
});

describe('bannerKind — le bandeau de l’application (AC31 (b))', () => {
	for (const status of ['failed', 'insufficient_space', 'disk_full'] as const) {
		it(`activée, échec ${status} → failed`, () => {
			expect(bannerKind(schedule({ lastStatus: status }))).toBe('failed');
		});
	}
	it('activée, en retard selon le serveur → overdue', () => {
		expect(bannerKind(schedule({ overdue: true }))).toBe('overdue');
	});
	it('activée, ok, pas en retard → rien', () => {
		expect(bannerKind(schedule())).toBeNull();
	});
	it('partial n’allume pas le bandeau', () => {
		expect(bannerKind(schedule({ lastStatus: 'partial', lastFilesNotIncluded: 2 }))).toBeNull();
	});
	it('désactivée, même sur un échec → rien', () => {
		expect(bannerKind(schedule({ enabled: false, lastStatus: 'disk_full', overdue: true }))).toBeNull();
	});
	it('jamais tentée (tout nul) → rien', () => {
		expect(
			bannerKind(
				schedule({ lastAttemptAt: null, lastSuccessAt: null, lastStatus: null, overdue: false }),
			),
		).toBeNull();
	});
	it('état illisible → rien', () => {
		expect(bannerKind(null)).toBeNull();
	});
});
