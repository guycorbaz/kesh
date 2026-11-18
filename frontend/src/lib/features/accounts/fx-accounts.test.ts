// Story 37-1 (AC-4 bis) — miroir de la résolution serveur des comptes de
// différence de change, et champs de devise envoyés par le formulaire.
import { describe, it, expect } from 'vitest';
import type { AccountResponse } from './accounts.types';
import {
	currencyFieldsForCreate,
	currencyFieldsForUpdate,
	fxAccountsFor,
	fxDifferenceAccountOptions,
	fxTransitAccountOptions,
	initialCurrencyForm,
	ownAccountsIncomplete,
	readFxAccountField,
	type AccountCurrencyForm,
} from './fx-accounts';

function account(overrides: Partial<AccountResponse> = {}): AccountResponse {
	return {
		id: 1,
		companyId: 1,
		number: '1020',
		name: 'Banque',
		accountType: 'Asset',
		parentId: null,
		active: true,
		role: null,
		postable: true,
		letterable: false,
		version: 1,
		createdAt: '2026-01-01T00:00:00',
		updatedAt: '2026-01-01T00:00:00',
		...overrides,
	};
}

const COMPANY = { fxGainAccountId: 68, fxLossAccountId: 69 };

describe('fxAccountsFor', () => {
	it('compte en devise avec comptes propres : les siens (origin own)', () => {
		const a = account({ currency: 'USD', fxGainAccountId: 10, fxLossAccountId: 11 });
		expect(fxAccountsFor(a, COMPANY)).toEqual({ gain: 10, loss: 11, origin: 'own' });
	});

	it('compte en devise sans comptes propres : ceux de la société (origin company)', () => {
		const a = account({ currency: 'EUR' });
		expect(fxAccountsFor(a, COMPANY)).toEqual({ gain: 68, loss: 69, origin: 'company' });
	});

	it('par défaut, suit le réglage de la société après son changement', () => {
		const a = account({ currency: 'EUR' });
		expect(fxAccountsFor(a, { fxGainAccountId: 70, fxLossAccountId: 71 })).toEqual({
			gain: 70,
			loss: 71,
			origin: 'company',
		});
	});

	it('les comptes propres ne suivent pas le changement du réglage', () => {
		const a = account({ currency: 'USD', fxGainAccountId: 10, fxLossAccountId: 11 });
		expect(fxAccountsFor(a, { fxGainAccountId: 70, fxLossAccountId: 71 })?.origin).toBe('own');
	});

	it('compte CHF : null, même avec un réglage de société', () => {
		expect(fxAccountsFor(account(), COMPANY)).toBeNull();
	});

	it('compte en devise, société sans comptes (état hérité) : null', () => {
		expect(fxAccountsFor(account({ currency: 'USD' }), { fxGainAccountId: null, fxLossAccountId: null })).toBeNull();
	});
});

describe('filtres des sélecteurs', () => {
	const all = [
		account({ id: 1, accountType: 'Expense', number: '6840' }),
		account({ id: 2, accountType: 'Revenue', number: '3000', role: 'DefaultRevenue' }),
		account({ id: 3, accountType: 'Revenue', number: '6950' }),
		account({ id: 4, accountType: 'Expense', number: '6841', active: false }),
		account({ id: 5, accountType: 'Expense', number: '6842', postable: false }),
		account({ id: 6, accountType: 'Expense', number: '6843', currency: 'USD' }),
		account({ id: 7, accountType: 'Asset', number: '1090' }),
		account({ id: 8, accountType: 'Liability', number: '2090' }),
		account({ id: 9, accountType: 'Asset', number: '1021', currency: 'EUR' }),
	];

	it('gains et pertes : Expense/Revenue imputables actifs en CHF, hors produit par défaut', () => {
		expect(fxDifferenceAccountOptions(all).map((a) => a.id)).toEqual([1, 3]);
	});

	it('conversion : Asset/Liability imputables actifs en CHF', () => {
		expect(fxTransitAccountOptions(all).map((a) => a.id)).toEqual([7, 8]);
	});
});

function form(overrides: Partial<AccountCurrencyForm> = {}): AccountCurrencyForm {
	return { currency: '', fxChoice: 'default', fxGain: '', fxLoss: '', ...overrides };
}

describe('currencyFieldsForCreate', () => {
	it('mono-devise : rien, même si un champ était rempli', () => {
		expect(currencyFieldsForCreate(form({ currency: 'USD' }), false)).toEqual({});
	});

	it('CHF (vide ou « chf ») : rien', () => {
		expect(currencyFieldsForCreate(form(), true)).toEqual({});
		expect(currencyFieldsForCreate(form({ currency: 'chf' }), true)).toEqual({});
	});

	it('devise par défaut : la devise seule', () => {
		expect(currencyFieldsForCreate(form({ currency: 'USD' }), true)).toEqual({ currency: 'USD' });
	});

	it('devise avec comptes propres : la devise et les deux comptes', () => {
		expect(
			currencyFieldsForCreate(form({ currency: 'EUR', fxChoice: 'own', fxGain: '10', fxLoss: '11' }), true),
		).toEqual({ currency: 'EUR', fxGainAccountId: 10, fxLossAccountId: 11 });
	});
});

describe('currencyFieldsForUpdate', () => {
	it('mono-devise : rien', () => {
		expect(currencyFieldsForUpdate(account(), form({ currency: 'USD' }), false)).toEqual({});
	});

	it('rien ne change (compte « par défaut ») : rien n’est envoyé', () => {
		const a = account({ currency: 'USD' });
		expect(currencyFieldsForUpdate(a, initialCurrencyForm(a), true)).toEqual({});
	});

	it('rien ne change (comptes propres) : rien n’est envoyé', () => {
		const a = account({ currency: 'USD', fxGainAccountId: 10, fxLossAccountId: 11 });
		expect(currencyFieldsForUpdate(a, initialCurrencyForm(a), true)).toEqual({});
	});

	it('CHF → devise, par défaut : la devise seule', () => {
		expect(currencyFieldsForUpdate(account(), form({ currency: 'USD' }), true)).toEqual({ currency: 'USD' });
	});

	it('par défaut → propres : les deux comptes', () => {
		const a = account({ currency: 'USD' });
		expect(
			currencyFieldsForUpdate(a, form({ currency: 'USD', fxChoice: 'own', fxGain: '10', fxLoss: '11' }), true),
		).toEqual({ fxGainAccountId: 10, fxLossAccountId: 11 });
	});

	it('propres → par défaut : null pour les deux', () => {
		const a = account({ currency: 'USD', fxGainAccountId: 10, fxLossAccountId: 11 });
		expect(currencyFieldsForUpdate(a, form({ currency: 'USD' }), true)).toEqual({
			fxGainAccountId: null,
			fxLossAccountId: null,
		});
	});

	it('un seul compte propre changé : les deux sont envoyés', () => {
		const a = account({ currency: 'USD', fxGainAccountId: 10, fxLossAccountId: 11 });
		expect(
			currencyFieldsForUpdate(a, form({ currency: 'USD', fxChoice: 'own', fxGain: '10', fxLoss: '12' }), true),
		).toEqual({ fxGainAccountId: 10, fxLossAccountId: 12 });
	});

	it('devise remise à vide sur un compte à comptes propres : currency null et comptes null', () => {
		const a = account({ currency: 'USD', fxGainAccountId: 10, fxLossAccountId: 11 });
		expect(
			currencyFieldsForUpdate(a, form({ currency: '', fxChoice: 'own', fxGain: '10', fxLoss: '11' }), true),
		).toEqual({ currency: null, fxGainAccountId: null, fxLossAccountId: null });
	});

	it('devise remise à vide sur un compte « par défaut » : currency null seul', () => {
		expect(currencyFieldsForUpdate(account({ currency: 'USD' }), form(), true)).toEqual({ currency: null });
	});
});

describe('ownAccountsIncomplete', () => {
	it('propres sans les deux comptes : incomplet', () => {
		expect(ownAccountsIncomplete(form({ currency: 'USD', fxChoice: 'own', fxGain: '10' }))).toBe(true);
	});
	it('par défaut, ou compte CHF : jamais incomplet', () => {
		expect(ownAccountsIncomplete(form({ currency: 'USD' }))).toBe(false);
		expect(ownAccountsIncomplete(form({ fxChoice: 'own' }))).toBe(false);
	});
});

describe('readFxAccountField', () => {
	it('lit le champ désigné, null sinon', () => {
		expect(readFxAccountField({ field: 'fxLossAccountId', reason: 'archived' })).toBe('fxLossAccountId');
		expect(readFxAccountField({ field: 'other' })).toBeNull();
		expect(readFxAccountField(undefined)).toBeNull();
	});
});
