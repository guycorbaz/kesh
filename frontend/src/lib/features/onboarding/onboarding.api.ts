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

/** Adresse structurée (#213) — miroir de `StructuredAddressInput` backend. */
export interface StructuredAddressPayload {
	street: string;
	building: string;
	postalCode: string;
	city: string;
	country: string;
}

export async function setCoordinates(
	name: string,
	firstName: string | null,
	lastName: string | null,
	address: StructuredAddressPayload,
	ideNumber: string | null
): Promise<OnboardingState> {
	return apiClient.post<OnboardingState>('/api/v1/onboarding/coordinates', {
		name,
		firstName,
		lastName,
		address,
		ideNumber,
	});
}

/**
 * Le compte bancaire de l'étape 6 de l'assistant — Story 34-1 (#966).
 *
 * `description` et `journalAccountId` sont **toujours** envoyés, `null`
 * compris : côté serveur, un champ absent vaut « inchangé » et `null` vaut
 * « effacé » (C-e341-34). L'écran envoie donc ce qu'il affiche.
 */
export interface BankAccountStepPayload {
	bankName: string;
	iban: string;
	qrIban: string | null;
	/** Description normalisée par l'appelant (`trim`, vide → `null`). */
	description: string | null;
	/** Compte comptable lié, `null` pour « Non configuré ». */
	journalAccountId: number | null;
}

export async function setBankAccount(payload: BankAccountStepPayload): Promise<OnboardingState> {
	return apiClient.post<OnboardingState>('/api/v1/onboarding/bank-account', {
		bankName: payload.bankName,
		iban: payload.iban,
		qrIban: payload.qrIban,
		description: payload.description,
		journalAccountId: payload.journalAccountId,
	});
}

export async function skipBank(): Promise<OnboardingState> {
	return apiClient.post<OnboardingState>('/api/v1/onboarding/skip-bank');
}

// Story 2.6: Finalize onboarding and pre-fill invoice settings
export async function finalize(): Promise<OnboardingState> {
	return apiClient.post<OnboardingState>('/api/v1/onboarding/finalize');
}
