// Story 26-2b (#622, AC9, site D9) — *Paramètres* nomme chaque compte bancaire
// avec sa description. La réponse simulée de `/api/v1/companies/current` est
// typée `CompanyCurrentResponse` : `description` y est obligatoire, `npm run
// check` refuse une simulation qui l'omet.
//
// Le nom du fichier n'est pas `+page.test.ts` : SvelteKit réserve le préfixe `+`.

import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render } from '@testing-library/svelte';
import type { CompanyCurrentResponse } from '$lib/features/settings/settings.types';

vi.mock('$app/environment', () => ({ browser: true }));
vi.mock('$lib/features/onboarding/onboarding.svelte', () => ({
	i18nMsg: (_key: string, fallback: string) => fallback,
}));
vi.mock('$lib/shared/utils/notify', () => ({ notifySuccess: vi.fn() }));
const fetchCompanyCurrentMock = vi.fn<() => Promise<CompanyCurrentResponse>>();
vi.mock('$lib/features/settings/settings.api', () => ({
	fetchCompanyCurrent: () => fetchCompanyCurrentMock(),
	updateCompanyContactDetails: vi.fn(),
	updateCompanyEmail: vi.fn(),
	lockBooks: vi.fn(),
	releaseBooksLock: vi.fn(),
}));

import Page from './+page.svelte';

function reponse(description: string | null): CompanyCurrentResponse {
	return {
		company: {
			id: 1,
			name: 'Test SA',
			address: 'Rue Test 1\n1000 Lausanne',
			ideNumber: null,
			orgType: 'Pme',
			accountingLanguage: 'FR',
			instanceLanguage: 'FR',
			version: 1,
		} as unknown as CompanyCurrentResponse['company'],
		bankAccounts: [
			{
				id: 7,
				bankName: 'UBS',
				description,
				iban: 'CH9300762011623852957',
				qrIban: null,
				isPrimary: true,
			},
		],
	};
}

describe('page Paramètres — nom des comptes bancaires (Story 26-2b, D9)', () => {
	beforeEach(() => {
		vi.clearAllMocks();
	});

	it('avec description : « UBS — Personnel »', async () => {
		fetchCompanyCurrentMock.mockResolvedValue(reponse('Personnel'));
		const { findByTestId } = render(Page);
		expect((await findByTestId('settings-bank-name-7')).textContent).toBe('UBS — Personnel');
	});

	it('sans description : la banque seule', async () => {
		fetchCompanyCurrentMock.mockResolvedValue(reponse(null));
		const { findByTestId } = render(Page);
		expect((await findByTestId('settings-bank-name-7')).textContent).toBe('UBS');
	});
});
