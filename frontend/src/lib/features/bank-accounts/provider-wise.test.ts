// Story 39-11d (AC-1) — règles d'écran du compte Wise : prédicat, libellés distincts des seuls
// comptes Wise, ligne de la liste, charge `wise` (sans compte de versement), paramètres lus.

import { describe, it, expect } from 'vitest';
import type { AccountResponse } from '$lib/features/accounts/accounts.types';
import {
	CONFIGURABLE_PROVIDERS,
	isProvider,
	isWise,
	providerPayload,
	providerSettingsOf,
	providerSummaryLine,
	wiseAccountLabels,
	withoutProviders,
} from './provider';

const eur = { id: 9, currency: 'EUR' } as AccountResponse;
const wise = (account: string, journalAccountId = 9) => ({
	provider: 'wise' as const,
	wise: { account, feeAccountId: null },
	journalAccountId,
});

describe('provider — le compte Wise (Story 39-11d)', () => {
	it('isWise, isProvider, withoutProviders', () => {
		expect(isWise({ provider: 'wise' })).toBe(true);
		expect(isWise({ provider: 'paypal' })).toBe(false);
		expect(isProvider({ provider: 'wise' })).toBe(true);
		expect(withoutProviders([{ provider: 'wise' }, { provider: null }])).toEqual([{ provider: null }]);
		expect(CONFIGURABLE_PROVIDERS).toEqual(['paypal', 'wise', 'stripe']);
	});

	it('wiseAccountLabels : libellés distincts des seuls comptes Wise, sans casse ni accents, triés', () => {
		expect(
			wiseAccountLabels([
				wise('Wise Zürich'),
				wise('wise zurich'),
				wise('Wise Pro'),
				{ provider: 'paypal', paypal: { account: 'PayPal www', feeAccountId: null } },
			]),
		).toEqual(['Wise Pro', 'Wise Zürich']);
	});

	it('providerSettingsOf lit `wise` ; un compte Wise n’a pas de paramètres PayPal', () => {
		expect(providerSettingsOf(wise('Wise Pro'), 'wise')).toEqual({ account: 'Wise Pro', feeAccountId: null });
		expect(providerSettingsOf(wise('Wise Pro'), 'paypal')).toBeNull();
	});

	it('providerSummaryLine : « Wise · <compte> · <devise> »', () => {
		expect(providerSummaryLine(wise('Wise Pro'), [eur])).toBe('Wise · Wise Pro · EUR');
		expect(providerSummaryLine(wise('Wise Pro', 8), [eur])).toBe('Wise · Wise Pro · CHF');
	});

	it('providerPayload wise : sous `wise`, libellé rogné, jamais de compte de versement', () => {
		expect(providerPayload('wise', '  Wise Pro ', 7, 12)).toEqual({ wise: { account: 'Wise Pro', feeAccountId: 7 } });
		expect(providerPayload('wise', ' ', 7, null)).toBeNull();
	});
});
