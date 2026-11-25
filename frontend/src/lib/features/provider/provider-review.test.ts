// Story 40-9 — règles communes des pages des prestataires : le lien « Comptes
// bancaires » d'un refus d'import (revue P1 de la 40-7d, L7).
import { describe, it, expect } from 'vitest';
import { importErrorCode, importErrorConcernsAccounts } from './provider-review';

describe('importErrorConcernsAccounts (Story 40-9)', () => {
	it.each(['PROVIDER_ACCOUNT_NOT_FOUND', 'PROVIDER_CURRENCY_ACCOUNT_MISSING', 'PROVIDER_CURRENCY_DUPLICATE', 'BANK_ACCOUNT_CLOSED'])(
		'%s : le lien',
		(code) => expect(importErrorConcernsAccounts(code)).toBe(true),
	);
	it.each(['STRIPE_CSV_INVALID', 'PAYPAL_CSV_INVALID', 'BANK_IMPORT_DUPLICATE_FILE', 'NETWORK_ERROR', 'PROVIDER_LEDGER_REQUIRED'])(
		'%s : pas de lien',
		(code) => expect(importErrorConcernsAccounts(code)).toBe(false),
	);
	it('erreur sans code : pas de lien', () => {
		expect(importErrorConcernsAccounts(importErrorCode(new Error('panne')))).toBe(false);
		expect(importErrorCode({ code: 'BANK_ACCOUNT_CLOSED', status: 409, message: 'clos' })).toBe('BANK_ACCOUNT_CLOSED');
	});
});
