import { apiClient } from '$lib/shared/utils/api-client';
import type { OnboardingState } from './onboarding.types';

export async function fetchState(): Promise<OnboardingState> {
	return apiClient.get<OnboardingState>('/api/v1/onboarding/state');
}

export async function setLanguage(language: string): Promise<OnboardingState> {
	return apiClient.post<OnboardingState>('/api/v1/onboarding/language', { language });
}

export async function setMode(mode: string): Promise<OnboardingState> {
	return apiClient.post<OnboardingState>('/api/v1/onboarding/mode', { mode });
}

export async function seedDemo(): Promise<OnboardingState> {
	return apiClient.post<OnboardingState>('/api/v1/onboarding/seed-demo');
}

export async function resetDemo(): Promise<OnboardingState> {
	return apiClient.post<OnboardingState>('/api/v1/onboarding/reset');
}
