// Story 26-2a (#622, AC6) — nom et libellé d'un compte bancaire.
import { describe, expect, it } from 'vitest';
import {
	BANK_ACCOUNT_DESCRIPTION_MAX,
	bankAccountName,
	bankAccountOptionLabel,
	bankAccountShortId,
	cardLast4Label,
	shortIban,
} from './format';

const IBAN = 'CH9300762011623852957';
/**
 * « •••• 1234 », l'espace insécable (U+00A0) comprise (revue P1, A7). Écrite par son
 * code, jamais en littéral invisible (revue P2, B3) : un éditeur qui normaliserait les
 * espaces ne peut pas la changer en espace ordinaire sans que ce test rougisse.
 */
const NBSP = String.fromCharCode(0xa0);
const CARTE = `••••${NBSP}1234`;

// Story 34-11a1 (#972, AC-10) — un compte sans IBAN s'affiche sans planter.
describe('compte sans IBAN', () => {
	it('shortIban est gardé : null rend une chaîne vide, sans TypeError', () => {
		expect(shortIban(null)).toBe('');
		expect(shortIban(IBAN)).toBe('••••2957');
	});

	it('cardLast4Label : « •••• 1234 », null sans quatre chiffres', () => {
		expect(cardLast4Label('1234')).toBe(CARTE);
		// L'espace est insécable : la forme ne se coupe pas en colonne étroite.
		expect(cardLast4Label('1234')).not.toContain(' ');
		expect(cardLast4Label(null)).toBeNull();
		expect(cardLast4Label(undefined)).toBeNull();
	});

	it('bankAccountShortId : IBAN court, carte, ou rien', () => {
		expect(bankAccountShortId({ iban: IBAN, cardLast4: null })).toBe('••••2957');
		expect(bankAccountShortId({ iban: null, cardLast4: '1234' })).toBe(CARTE);
		expect(bankAccountShortId({ iban: null, cardLast4: null })).toBe('');
	});

	it('bankAccountOptionLabel : « •••• 1234 » à la place de l’IBAN, dans les deux styles', () => {
		const carte = { bankName: 'Visa', description: 'Pro', iban: null, cardLast4: '1234' };
		expect(bankAccountOptionLabel(carte)).toBe(`Visa — Pro — ${CARTE}`);
		expect(bankAccountOptionLabel(carte, 'parens')).toBe(`Visa — Pro (${CARTE})`);
	});

	it('bankAccountOptionLabel : le nom seul sans IBAN ni quatre chiffres (jamais « null »)', () => {
		const nu = { bankName: 'Caisse', description: null, iban: null };
		expect(bankAccountOptionLabel(nu)).toBe('Caisse');
		expect(bankAccountOptionLabel(nu, 'parens')).toBe('Caisse');
	});

	it('bankAccountOptionLabel : un IBAN présent garde la priorité sur les quatre chiffres', () => {
		expect(
			bankAccountOptionLabel({ bankName: 'UBS', description: null, iban: IBAN, cardLast4: '1234' }),
		).toBe(`UBS — ${IBAN}`);
	});
});

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
