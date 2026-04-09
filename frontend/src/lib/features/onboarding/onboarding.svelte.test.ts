import { describe, it, expect, vi, beforeEach } from 'vitest';
import { onboardingState } from './onboarding.svelte';

// Mock the API module and api-client (imported by onboarding store for i18n)
vi.mock('./onboarding.api', () => ({
	fetchState: vi.fn(),
	setLanguage: vi.fn(),
	setMode: vi.fn(),
	seedDemo: vi.fn(),
	resetDemo: vi.fn(),
}));

vi.mock('$lib/shared/utils/api-client', () => ({
	apiClient: { get: vi.fn().mockResolvedValue({ locale: 'fr-CH', messages: {} }) },
}));

import * as api from './onboarding.api';

const mockApi = vi.mocked(api);

describe('onboardingState', () => {
	beforeEach(() => {
		vi.clearAllMocks();
	});

	it('starts with default values', () => {
		expect(onboardingState.stepCompleted).toBe(0);
		expect(onboardingState.isDemo).toBe(false);
		expect(onboardingState.uiMode).toBeNull();
		expect(onboardingState.loaded).toBe(false);
	});

	it('fetchState updates state from API', async () => {
		mockApi.fetchState.mockResolvedValue({
			stepCompleted: 2,
			isDemo: false,
			uiMode: 'guided',
		});

		await onboardingState.fetchState();

		expect(onboardingState.stepCompleted).toBe(2);
		expect(onboardingState.uiMode).toBe('guided');
		expect(onboardingState.loaded).toBe(true);
	});

	it('setLanguage calls API and updates state', async () => {
		mockApi.setLanguage.mockResolvedValue({
			stepCompleted: 1,
			isDemo: false,
			uiMode: null,
		});

		await onboardingState.setLanguage('FR');

		expect(mockApi.setLanguage).toHaveBeenCalledWith('FR');
		expect(onboardingState.stepCompleted).toBe(1);
	});

	it('setMode calls API and updates state', async () => {
		mockApi.setMode.mockResolvedValue({
			stepCompleted: 2,
			isDemo: false,
			uiMode: 'expert',
		});

		await onboardingState.setMode('expert');

		expect(mockApi.setMode).toHaveBeenCalledWith('expert');
		expect(onboardingState.uiMode).toBe('expert');
	});

	it('seedDemo calls API and sets isDemo true', async () => {
		mockApi.seedDemo.mockResolvedValue({
			stepCompleted: 3,
			isDemo: true,
			uiMode: 'guided',
		});

		await onboardingState.seedDemo();

		expect(mockApi.seedDemo).toHaveBeenCalled();
		expect(onboardingState.stepCompleted).toBe(3);
		expect(onboardingState.isDemo).toBe(true);
	});

	it('resetDemo calls API and resets state', async () => {
		mockApi.resetDemo.mockResolvedValue({
			stepCompleted: 0,
			isDemo: false,
			uiMode: null,
		});

		await onboardingState.resetDemo();

		expect(mockApi.resetDemo).toHaveBeenCalled();
		expect(onboardingState.stepCompleted).toBe(0);
		expect(onboardingState.isDemo).toBe(false);
	});

	it('loading is true during API calls', async () => {
		let resolvePromise: (value: unknown) => void;
		const pendingPromise = new Promise((resolve) => {
			resolvePromise = resolve;
		});

		mockApi.fetchState.mockReturnValue(pendingPromise as Promise<never>);

		const fetchPromise = onboardingState.fetchState();
		expect(onboardingState.loading).toBe(true);

		resolvePromise!({ stepCompleted: 0, isDemo: false, uiMode: null });
		await fetchPromise;
		expect(onboardingState.loading).toBe(false);
	});
});
