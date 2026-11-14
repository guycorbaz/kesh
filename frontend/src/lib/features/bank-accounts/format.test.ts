// Story 26-2a (#622, AC6) — nom et libellé d'un compte bancaire.
import { describe, expect, it } from 'vitest';
import type { AccountResponse } from '$lib/features/accounts/accounts.types';
import {
	BANK_ACCOUNT_DESCRIPTION_MAX,
	bankAccountName,
	bankAccountOptionLabel,
	bankAccountShortId,
	cardLast4Label,
	defaultOpeningSide,
	formatChfBalance,
	formatChfExact,
	linkableLedgerAccounts,
	openingAmountKind,
	openingAmountPayload,
	openingAmountPreview,
	shortIban,
} from './format';
import openingAmountCases from './opening-amount-cases.json';
import { admitsLedgerType } from './credit-card';

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

	// Revue P1 de la 34-2 (B-1 = E-6), re-adossé au classement par la Story 35-6 :
	// « 1'000,000 » (partie entière groupée) cesse d'être ambigu (#1025), « 1.234 » le
	// devient (#1026).
	it("« 1,234 » et « 1.234 » sont ambigus ; « 1,23 », « 1,2345 », « 1'234,50 », « 1'000,000 » ne le sont pas", () => {
		for (const ambigu of ['1,234', '12,500', ' 9,999 ', '1.234']) {
			expect(openingAmountKind(ambigu).kind, ambigu).toBe('ambiguous');
		}
		for (const clair of ['1,23', '1,2345', "1'234,50", "1'000,000", '1234', '', '1,2']) {
			expect(openingAmountKind(clair).kind, clair).not.toBe('ambiguous');
		}
	});

	it('l’aperçu donne le montant écrit au format suisse, rien sinon', () => {
		expect(openingAmountPreview("1'234.50")).toBe(formatChfBalance('1234.50'));
		expect(openingAmountPreview('75,25')).toBe(formatChfBalance('75.25'));
		for (const rien of ['', '1,234', 'abc', '-5', '1.00001', '1.2.3']) {
			expect(openingAmountPreview(rien), rien).toBeNull();
		}
	});

	// L'aperçu dit ce que le serveur écrit (quatre décimales), sans l'arrondir au centime.
	it('l’aperçu garde jusqu’à quatre décimales, sans arrondi', () => {
		expect(openingAmountPreview('1.2345')).toContain('1.2345');
		// Story 35-6 : « 1.005 » est ambigu (famille A) ; la preuve passe à « 1.0050 ».
		expect(openingAmountPreview('1.0050')).toContain('1.005');
		expect(formatChfExact('1234.5')).toBe(formatChfBalance('1234.50'));
	});
});

/**
 * Story 35-6 (#1026, #1028, #1025, AC1) — la table D3 de la fiche, écrite une seule fois
 * dans `opening-amount-cases.json` et relue ici ligne par ligne : l'issue du classement
 * ET la charge envoyée. Le test de parité du serveur (`opening_balances.rs`, AC8) relit
 * la même fixture.
 */
type OpeningAmountCase = {
	raw: string;
	kind: string;
	payload?: string;
	serverRejects?: boolean;
	serverAccepts?: string;
};
const CASES: OpeningAmountCase[] = openingAmountCases.cases;

describe('Story 35-6 — la saisie du solde d’ouverture, table D3', () => {
	it.each(CASES.map((c) => [JSON.stringify(c.raw), c] as const))(
		'%s : issue et charge',
		(_label, c) => {
			const expected = c.payload === undefined ? { kind: c.kind } : { kind: c.kind, payload: c.payload };
			expect(openingAmountKind(c.raw)).toEqual(expected);
		},
	);

	it('une charge claire suit ^[0-9]+(\\.[0-9]{1,4})?$ ; seule une saisie claire en porte une', () => {
		for (const c of CASES) {
			if (c.kind === 'clear') {
				expect(c.payload, c.raw).toMatch(/^[0-9]+(\.[0-9]{1,4})?$/);
			} else {
				expect(c.payload, c.raw).toBeUndefined();
			}
		}
	});

	it('openingAmountPayload rend la charge d’une saisie claire, null sinon', () => {
		for (const c of CASES) {
			expect(openingAmountPayload(c.raw), JSON.stringify(c.raw)).toBe(
				c.kind === 'clear' ? c.payload : null,
			);
		}
	});

	it('l’aperçu formate la charge d’une saisie claire, rien sinon, jamais « NaN »', () => {
		for (const c of CASES) {
			const preview = openingAmountPreview(c.raw);
			if (c.kind === 'clear') {
				expect(preview, JSON.stringify(c.raw)).toBe(formatChfExact(c.payload as string));
				expect(preview, JSON.stringify(c.raw)).not.toContain('NaN');
			} else {
				expect(preview, JSON.stringify(c.raw)).toBeNull();
			}
		}
	});

	// Les bornes de D4 sont dans la table, et la borne du serveur y est dite.
	it('la table porte les bornes de la règle et la borne du serveur', () => {
		const kindOf = (raw: string) => CASES.find((c) => c.raw === raw)?.kind;
		expect(kindOf('1.000')).toBe('ambiguous');
		expect(kindOf('100.250')).toBe('ambiguous');
		expect(kindOf('1000.000')).toBe('clear');
		expect(kindOf('0.999')).toBe('clear');
		expect(CASES.filter((c) => c.serverRejects).map((c) => c.raw)).toEqual(['1234567890123456']);
	});

	// AC3 — l'aperçu de « 1_000 » (#1028) : mille, à l'espace fine U+202F.
	it('« 1_000 » s’affiche 1 000.00 CHF, jamais « NaN CHF »', () => {
		expect(openingAmountPreview('1_000')).toBe(formatChfBalance('1000'));
		expect(openingAmountPreview('1_000')).toContain(`1${String.fromCharCode(0x202f)}000.00`);
		expect(openingAmountPreview(`1${String.fromCharCode(0x202f)}234.50`)).toBe(
			formatChfBalance('1234.50'),
		);
		expect(openingAmountPreview('0.500')).toBe(formatChfExact('0.500'));
	});
});

// Intégration de la 0.17.6 (C-e34a2-1, raccord 1) — le filtre du lien reprend la règle
// `admitsLedgerType` : passifs seuls pour une carte, actifs ou passifs sinon.
describe('linkableLedgerAccounts — type du compte bancaire', () => {
	const compte = (id: number, number: string, accountType: string) =>
		({ id, number, accountType, active: true, postable: true }) as AccountResponse;
	const plan = [
		compte(1, '1020', 'Asset'),
		compte(2, '2100', 'Liability'),
		compte(3, '6500', 'Expense'),
		compte(4, '3000', 'Revenue'),
	];

	it('compte bancaire : actifs et passifs ; carte (liabilityOnly) : passifs seuls', () => {
		expect(linkableLedgerAccounts(plan, new Set()).map((a) => a.id)).toEqual([1, 2]);
		expect(
			linkableLedgerAccounts(plan, new Set(), { liabilityOnly: true }).map((a) => a.id),
		).toEqual([2]);
	});

	it('le filtre suit admitsLedgerType pour chaque type de compte du plan', () => {
		for (const kind of ['bank', 'credit_card'] as const) {
			const attendu = plan.filter((a) => admitsLedgerType(kind, a.accountType)).map((a) => a.id);
			const rendu = linkableLedgerAccounts(plan, new Set(), {
				liabilityOnly: kind === 'credit_card',
			}).map((a) => a.id);
			expect(rendu, kind).toEqual(attendu);
		}
	});
});
