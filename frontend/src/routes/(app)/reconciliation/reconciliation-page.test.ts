// Story 38-11d (revue P1, M2) — la page de réconciliation générique n'offre pas un compte de
// prestataire (PayPal) : le serveur ne refuse pas toutes ses propositions génériques (angle
// mort de 38-11b, AC-3 b), ce filtre est la seule barrière de l'écran ; ses transactions se
// rapprochent par la page PayPal.

import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render } from '@testing-library/svelte';

vi.mock('$lib/shared/utils/i18n.svelte', () => ({
	i18nMsg: (_key: string, fallback: string) => fallback,
}));

const getMock = vi.fn();
vi.mock('$lib/shared/utils/api-client', () => ({
	apiClient: { get: (...a: unknown[]) => getMock(...a) },
}));

vi.mock('$lib/features/reconciliation/ReconciliationProposals.svelte', async () => ({
	default: (await import('./ReconciliationProposalsStub.test.svelte')).default,
}));

import Page from './+page.svelte';

const compte = (id: number, bankName: string, provider: string | null) => ({
	id,
	bankName,
	description: null,
	iban: provider ? null : 'CH9300762011623852957',
	qrIban: null,
	isPrimary: id === 1,
	provider,
});

describe('page de réconciliation — comptes de prestataire écartés (Story 38-11d)', () => {
	beforeEach(() => getMock.mockReset());

	it('un compte PayPal n’est pas proposé ; le compte ordinaire l’est', async () => {
		getMock.mockResolvedValue({ company: {}, bankAccounts: [compte(1, 'UBS', null), compte(2, 'PayPal', 'paypal')] });
		const r = render(Page);
		const select = (await r.findByTestId('bank-account-select')) as HTMLSelectElement;
		expect(Array.from(select.options).map((o) => o.value)).toEqual(['1']);
	});

	// Story 40-7d (AC-1) — de même un compte Stripe ou Wise.
	it.each(['stripe', 'wise'])('un compte %s n’est pas proposé ; le compte ordinaire l’est', async (provider) => {
		getMock.mockResolvedValue({ company: {}, bankAccounts: [compte(1, 'UBS', null), compte(2, provider, provider)] });
		const r = render(Page);
		const select = (await r.findByTestId('bank-account-select')) as HTMLSelectElement;
		expect(Array.from(select.options).map((o) => o.value)).toEqual(['1']);
	});

	it('seulement des comptes PayPal : « aucun compte », aucune proposition chargée', async () => {
		getMock.mockResolvedValue({ company: {}, bankAccounts: [compte(2, 'PayPal', 'paypal')] });
		const r = render(Page);
		expect(await r.findByTestId('reconciliation-no-account')).toBeTruthy();
		expect(r.queryByTestId('proposals-stub')).toBeNull();
	});
});
