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

// --- Path B (Story 2.3) ---

export async function startProduction(): Promise<OnboardingState> {
	return apiClient.post<OnboardingState>('/api/v1/onboarding/start-production');
}

export async function setOrgType(orgType: string): Promise<OnboardingState> {
	return apiClient.post<OnboardingState>('/api/v1/onboarding/org-type', { orgType });
}

export async function setAccountingLanguage(language: string): Promise<OnboardingState> {
	return apiClient.post<OnboardingState>('/api/v1/onboarding/accounting-language', { language });
}

export async function setCoordinates(
	name: string,
	address: string,
	ideNumber: string | null
): Promise<OnboardingState> {
	return apiClient.post<OnboardingState>('/api/v1/onboarding/coordinates', {
		name,
		address,
		ideNumber,
	});
}

export async function setBankAccount(
	bankName: string,
	iban: string,
	qrIban: string | null
): Promise<OnboardingState> {
	return apiClient.post<OnboardingState>('/api/v1/onboarding/bank-account', {
		bankName,
		iban,
		qrIban,
	});
}

export async function skipBank(): Promise<OnboardingState> {
	return apiClient.post<OnboardingState>('/api/v1/onboarding/skip-bank');
}

// Story 2.6: Finalize onboarding and pre-fill invoice settings
export async function finalize(): Promise<OnboardingState> {
	return apiClient.post<OnboardingState>('/api/v1/onboarding/finalize');
}
