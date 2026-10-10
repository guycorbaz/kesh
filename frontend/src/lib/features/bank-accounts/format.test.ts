// Story 26-2a (#622, AC6) — nom et libellé d'un compte bancaire.
import { describe, expect, it } from 'vitest';
import {
	BANK_ACCOUNT_DESCRIPTION_MAX,
	bankAccountName,
	bankAccountOptionLabel,
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
