// Story 38-12b (AC-1) — l'état du téléchargement d'un compte PayPal, lu sur ses
// comptes bancaires (`providerApi` de la liste).

import { describe, it, expect } from 'vitest';
import { paypalApiStatus } from './provider';

const solde = (account: string, providerApi?: { configured: boolean; mode?: 'live' | 'sandbox' } | null) => ({
	provider: 'paypal',
	paypal: { account, feeAccountId: null },
	providerApi,
});

describe('paypalApiStatus', () => {
	it('configuré si un solde du libellé l’est, sans casse ni accents ; son mode', () => {
		const accounts = [solde('PayPal Zürich', { configured: false }), solde('paypal zurich', { configured: true, mode: 'sandbox' })];
		expect(paypalApiStatus(accounts, 'PAYPAL ZURICH')).toEqual({ configured: true, mode: 'sandbox' });
	});
	it('aucun solde configuré : non configuré ; champ absent (ancien serveur) : de même', () => {
		expect(paypalApiStatus([solde('PayPal www', { configured: false })], 'PayPal www')).toEqual({ configured: false });
		expect(paypalApiStatus([solde('PayPal www')], 'PayPal www')).toEqual({ configured: false });
	});
	it('aucun compte de ce libellé, ou compte ordinaire : null', () => {
		expect(paypalApiStatus([solde('PayPal www', { configured: true })], 'Autre')).toBeNull();
		expect(paypalApiStatus([{ provider: null, paypal: null, providerApi: null }], 'PayPal www')).toBeNull();
	});
});
