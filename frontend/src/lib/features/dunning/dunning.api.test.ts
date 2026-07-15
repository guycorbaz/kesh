// Story 21-4 — tests Vitest du client API dunning (niveaux + réglages grâce).
import { describe, expect, it, vi } from 'vitest';

vi.mock('$lib/shared/utils/api-client', () => ({
	apiClient: {
		get: vi.fn(),
		post: vi.fn(),
		put: vi.fn(),
		patch: vi.fn(),
		delete: vi.fn(),
	},
}));

import { apiClient } from '$lib/shared/utils/api-client';
import {
	listDunningLevels,
	createDunningLevel,
	updateDunningLevel,
	deleteDunningLevel,
	getDunningSettings,
	updateDunningSettings,
} from './dunning.api';

type Mock = ReturnType<typeof vi.fn>;

describe('dunning.api — niveaux', () => {
	it('list GET sur le bon path', async () => {
		(apiClient.get as Mock).mockResolvedValue([]);
		await listDunningLevels();
		expect(apiClient.get).toHaveBeenCalledWith('/api/v1/dunning-levels');
	});

	it('create POST délai + frais', async () => {
		(apiClient.post as Mock).mockResolvedValue({ id: 1, levelNumber: 1, delayDays: 10, feeAmount: '0.00', version: 0 });
		await createDunningLevel({ delayDays: 10, feeAmount: '0.00' });
		expect(apiClient.post).toHaveBeenCalledWith('/api/v1/dunning-levels', {
			delayDays: 10,
			feeAmount: '0.00',
		});
	});

	it('update PUT avec version au path /{id}', async () => {
		(apiClient.put as Mock).mockResolvedValue({ id: 7, levelNumber: 2, delayDays: 15, feeAmount: '20.00', version: 2 });
		await updateDunningLevel(7, { delayDays: 15, feeAmount: '20.00', version: 1 });
		expect(apiClient.put).toHaveBeenCalledWith('/api/v1/dunning-levels/7', {
			delayDays: 15,
			feeAmount: '20.00',
			version: 1,
		});
	});

	it('delete DELETE avec version dans le body', async () => {
		(apiClient.delete as Mock).mockResolvedValue(undefined);
		await deleteDunningLevel(7, 2);
		expect(apiClient.delete).toHaveBeenCalledWith('/api/v1/dunning-levels/7', { version: 2 });
	});
});

describe('dunning.api — réglages grâce', () => {
	it('get GET singleton', async () => {
		(apiClient.get as Mock).mockResolvedValue({ gracePeriodDays: 5, seededAt: null, version: 1 });
		const s = await getDunningSettings();
		expect(apiClient.get).toHaveBeenCalledWith('/api/v1/company/dunning-settings');
		expect(s.gracePeriodDays).toBe(5);
	});

	it('update PUT grace + version', async () => {
		(apiClient.put as Mock).mockResolvedValue({ gracePeriodDays: 10, seededAt: '2026-07-15T00:00:00', version: 2 });
		await updateDunningSettings({ gracePeriodDays: 10, version: 1 });
		expect(apiClient.put).toHaveBeenCalledWith('/api/v1/company/dunning-settings', {
			gracePeriodDays: 10,
			version: 1,
		});
	});
});
