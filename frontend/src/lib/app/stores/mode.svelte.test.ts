import { describe, it, expect, vi, beforeEach } from 'vitest';

// Mock api-client before importing mode store
vi.mock('$lib/shared/utils/api-client', () => ({
	apiClient: { put: vi.fn().mockResolvedValue(undefined) },
}));

import { modeState, toggleMode, syncModeFromServer } from './mode.svelte';

describe('modeState', () => {
	beforeEach(() => {
		modeState.value = 'guided';
		vi.stubGlobal('localStorage', {
			getItem: vi.fn().mockReturnValue(null),
			setItem: vi.fn(),
		});
	});

	it('defaults to guided', () => {
		expect(modeState.value).toBe('guided');
	});

	it('setter updates value and writes localStorage', () => {
		modeState.value = 'expert';
		expect(modeState.value).toBe('expert');
	});

	it('toggleMode switches guided → expert', () => {
		modeState.value = 'guided';
		toggleMode();
		expect(modeState.value).toBe('expert');
	});

	it('toggleMode switches expert → guided', () => {
		modeState.value = 'expert';
		toggleMode();
		expect(modeState.value).toBe('guided');
	});

	it('syncModeFromServer updates store when different', () => {
		modeState.value = 'guided';
		syncModeFromServer('expert');
		expect(modeState.value).toBe('expert');
	});

	it('syncModeFromServer does nothing when same', () => {
		modeState.value = 'expert';
		syncModeFromServer('expert');
		expect(modeState.value).toBe('expert');
	});

	it('syncModeFromServer ignores null', () => {
		modeState.value = 'guided';
		syncModeFromServer(null);
		expect(modeState.value).toBe('guided');
	});
});
