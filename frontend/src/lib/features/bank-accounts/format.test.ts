// Story 26-2a (#622, AC6) — nom et libellé d'un compte bancaire.
import { describe, expect, it } from 'vitest';
import type { AccountResponse } from '$lib/features/accounts/accounts.types';
import {
	BANK_ACCOUNT_DESCRIPTION_MAX,
	bankAccountName,
	bankAccountOptionLabel,
	defaultOpeningSide,
	openingAmountPayload,
} from './format';

const IBAN = 'CH9300762011623852957';

describe('bankAccountName', () => {
	it('avec description : « UBS — Personnel »', () => {
		expect(bankAccountName({ bankName: 'UBS', description: 'Personnel' })).toBe(
			'UBS — Personnel',
		);
	});

	it('sans description : la banque seule', () => {
		expect(bankAccountName({ bankName: 'UBS', description: null })).toBe('UBS');
	});

	it('description vide ou faite d’espaces : absente (jamais « UBS — »)', () => {
		expect(bankAccountName({ bankName: 'UBS', description: '' })).toBe('UBS');
		expect(bankAccountName({ bankName: 'UBS', description: '  ' })).toBe('UBS');
	});

	it('description undefined (donnée incomplète) : absente', () => {
		const a = { bankName: 'UBS' } as unknown as { bankName: string; description: string | null };
		expect(bankAccountName(a)).toBe('UBS');
	});

	it('le séparateur est un tiret cadratin U+2014 entouré d’espaces', () => {
		expect(bankAccountName({ bankName: 'UBS', description: 'Épargne' })).toBe(
			'UBS — Épargne',
		);
	});
});

describe('bankAccountOptionLabel', () => {
	it('sans description, identique à l’ancien libellé « banque — IBAN » (dash)', () => {
		const bankName = 'UBS';
		const iban = IBAN;
		expect(bankAccountOptionLabel({ bankName, description: null, iban })).toBe(
			`${bankName} — ${iban}`,
		);
	});

	it('sans description, identique à l’ancien libellé « banque (IBAN) » (parens)', () => {
		const bankName = 'UBS';
		const iban = IBAN;
		expect(bankAccountOptionLabel({ bankName, description: null, iban }, 'parens')).toBe(
			`${bankName} (${iban})`,
		);
	});

	it('avec description, dash : « UBS — Personnel — CH… »', () => {
		expect(bankAccountOptionLabel({ bankName: 'UBS', description: 'Personnel', iban: IBAN })).toBe(
			`UBS — Personnel — ${IBAN}`,
		);
	});

	it('avec description, parens : « UBS — Personnel (CH…) »', () => {
		expect(
			bankAccountOptionLabel({ bankName: 'UBS', description: 'Personnel', iban: IBAN }, 'parens'),
		).toBe(`UBS — Personnel (${IBAN})`);
	});

	it('description d’espaces : traitée comme absente', () => {
		expect(bankAccountOptionLabel({ bankName: 'UBS', description: '  ', iban: IBAN })).toBe(
			`UBS — ${IBAN}`,
		);
	});
});

describe('BANK_ACCOUNT_DESCRIPTION_MAX', () => {
	it('vaut 60, la borne de la colonne et du serveur', () => {
		expect(BANK_ACCOUNT_DESCRIPTION_MAX).toBe(60);
	});
});

// Story 34-2 (#533, C-e341-7) — la présélection du sens et le montant envoyé.
describe("solde d'ouverture", () => {
	const actif = { id: 1, accountType: 'Asset' } as AccountResponse;
	const passif = { id: 2, accountType: 'Liability' } as AccountResponse;

	it('le sens suit le type du compte lié : actif → debit, passif → credit', () => {
		expect(defaultOpeningSide([actif, passif], 1)).toBe('debit');
		expect(defaultOpeningSide([actif, passif], 2)).toBe('credit');
		expect(defaultOpeningSide([actif, passif], null)).toBe('debit');
		expect(defaultOpeningSide([actif, passif], 99)).toBe('debit');
	});

	it('le montant perd ses séparateurs suisses ; vide → null', () => {
		expect(openingAmountPayload("1'234.50")).toBe('1234.50');
		expect(openingAmountPayload('1’234,5')).toBe('1234.5');
		expect(openingAmountPayload(' 12 000 ')).toBe('12000');
		expect(openingAmountPayload('   ')).toBeNull();
		expect(openingAmountPayload('')).toBeNull();
	});
});
