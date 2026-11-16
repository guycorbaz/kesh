/**
 * Story 37-3b (AC-2) — l'état d'une ligne en devise : contrat de stockage,
 * provenance envoyée (D9, U1, A30, A32), changement de compte (U2), `lookup`,
 * ↑, une devise par écriture (R7), ligne d'écart (R8, S2, A12), rang des refus.
 */
import Big from 'big.js';
import { describe, expect, it, vi } from 'vitest';

vi.mock('$lib/shared/utils/i18n.svelte', () => ({
	i18nMsg: (_k: string, fallback: string) => fallback,
}));

import type { AccountResponse } from '$lib/features/accounts/accounts.types';
import {
	accountsForCurrencies,
	adoptCompanyRate,
	allowedCurrencies,
	applyAccountChange,
	applyChfEntry,
	applyLookup,
	commitChfEntry,
	companyRateHint,
	foreignChf,
	foreignFromResponse,
	foreignLineProblem,
	formIndexOfServerLine,
	lineToPayload,
	newForeign,
	planDifferenceLine,
	rateIndicator,
	rateSourceToSend,
	syncDerivedChf,
	type FxLineLike,
	type LineForeign,
} from './fx-lines';
import { lineResponseToDraft } from './form-helpers';
import type { JournalEntryLineResponse } from './journal-entries.types';
import { computeBalance, classifyLine, journalAmountReader } from './balance';
import { isDraftLineNonEmpty } from './vat-purchase';

function acc(o: Partial<AccountResponse>): AccountResponse {
	return {
		id: 1,
		companyId: 1,
		number: '1000',
		name: 'Compte',
		accountType: 'Asset',
		parentId: null,
		active: true,
		role: null,
		postable: true,
		letterable: false,
		version: 1,
		createdAt: '',
		updatedAt: '',
		...o,
	};
}

const USD = acc({ id: 10, number: '1022', currency: 'USD' });
const USD_OWN = acc({ id: 11, number: '1023', currency: 'USD', fxGainAccountId: 41, fxLossAccountId: 42 });
const EUR = acc({ id: 12, number: '1021', currency: 'EUR' });
const CHF = acc({ id: 1, number: '1020' });
const GAIN = acc({ id: 31, number: '6840', accountType: 'Revenue' });
const LOSS = acc({ id: 32, number: '6940', accountType: 'Expense' });
const OWN_GAIN = acc({ id: 41, number: '6841', accountType: 'Revenue' });
const OWN_LOSS = acc({ id: 42, number: '6941', accountType: 'Expense' });
const SALES_USD = acc({ id: 13, number: '3200', accountType: 'Revenue', currency: 'USD' });
const ACCOUNTS = [USD, USD_OWN, EUR, CHF, GAIN, LOSS, OWN_GAIN, OWN_LOSS, SALES_USD];
const COMPANY = { fxGainAccountId: 31, fxLossAccountId: 32 };

function fxLine(account: AccountResponse, f: Partial<LineForeign>): FxLineLike {
	return syncDerivedChf({
		accountId: account.id,
		debit: '',
		credit: '',
		foreign: { ...newForeign(account.currency!), ...f },
	});
}

describe('contrat de stockage', () => {
	it('le montant en devise vit dans foreign, debit/credit ne portent que le CHF dérivé', () => {
		const l = fxLine(USD, { amount: '100', rate: '0.8846' });
		expect(l.debit).toBe('88.46');
		expect(l.credit).toBe('');
		const c = fxLine(USD, { amount: '100', rate: '0.8846', side: 'credit' });
		expect([c.debit, c.credit]).toEqual(['', '88.46']);
	});
	it('sans cours, la ligne est partial : aucun CHF, jamais le montant lu comme CHF', () => {
		const l = fxLine(USD, { amount: '100', rate: '' });
		expect([l.debit, l.credit]).toEqual(['', '']);
		expect(classifyLine(l as never, journalAmountReader)).toBe('partial');
		expect(computeBalance([l], journalAmountReader).totalDebit.eq(0)).toBe(true);
		expect(foreignLineProblem(l.foreign!)).toBe('incomplete');
		expect(isDraftLineNonEmpty({ ...l, projectId: null } as never)).toBe(true);
	});
	it('le message est celui du code de computeChf, non un « montant invalide »', () => {
		expect(foreignLineProblem(fxLine(USD, { amount: '0.001', rate: '1' }).foreign!)).toBe('FX_LINE_ROUNDS_TO_ZERO');
		expect(foreignLineProblem(fxLine(USD, { amount: '1.00001', rate: '1' }).foreign!)).toBe('amount-unreadable');
		expect(foreignLineProblem(fxLine(USD, { amount: '1', rate: '0' }).foreign!)).toBe('rate-unreadable');
	});
});

describe('provenance envoyée (D9, U1)', () => {
	const date = '2026-03-15';
	it('lookup source afc → cours égal à afcRate du mois → afc', () => {
		const f = applyLookup(newForeign('USD'), '2026-03-01', {
			kind: 'rate',
			rate: { rate: '0.8846', source: 'afc', month: '2026-03-01' },
		});
		expect(f.rate).toBe('0.8846');
		expect(rateSourceToSend(f, date, false)).toBe('afc');
		expect(rateIndicator(f)).toEqual({ kind: 'afc' });
	});
	it('lookup source manual → ligne proposée mais manual (cours de la société)', () => {
		const f = applyLookup(newForeign('USD'), '2026-03-01', {
			kind: 'rate',
			rate: { rate: '0.88', source: 'manual', month: '2026-03-01' },
		});
		expect(f.rateProposed).toBe(true);
		expect(f.afcRate).toBeNull();
		expect(rateSourceToSend(f, date, false)).toBe('manual');
		expect(rateIndicator(f)).toEqual({ kind: 'saisi', source: null });
	});
	it('A30 : source inconnue (ecb) → afcRate null, manual, « cours saisi (ecb) »', () => {
		const f = applyLookup(newForeign('USD'), '2026-03-01', {
			kind: 'rate',
			rate: { rate: '0.88', source: 'ecb', month: '2026-03-01' },
		});
		expect(f.afcRate).toBeNull();
		expect(rateSourceToSend(f, date, false)).toBe('manual');
		expect(rateIndicator(f)).toEqual({ kind: 'saisi', source: 'ecb' });
	});
	it('cours retapé égal à l’AFC → afc ; différent → manual ; mois quitté → manual', () => {
		const base = applyLookup(newForeign('USD'), '2026-03-01', {
			kind: 'rate',
			rate: { rate: '0.8846', source: 'afc', month: '2026-03-01' },
		});
		expect(rateSourceToSend({ ...base, rate: '0.884600', touched: true, rateProposed: false }, date, false)).toBe('afc');
		expect(rateSourceToSend({ ...base, rate: '0.9', touched: true }, date, false)).toBe('manual');
		expect(rateSourceToSend(base, '2026-04-02', true)).toBe('manual');
	});
	it('lookup sans cours : le cours proposé est effacé, afcRate null', () => {
		const f = applyLookup({ ...newForeign('USD'), rate: '0.8846' }, '2026-04-01', { kind: 'none' });
		expect(f.rate).toBe('');
		expect(rateSourceToSend({ ...f, rate: '0.8846' }, '2026-04-15', true)).toBe('manual');
		// Ligne non proposée : son cours reste.
		const kept = applyLookup({ ...newForeign('USD'), rate: '0.9', rateProposed: false }, '2026-04-01', { kind: 'none' });
		expect(kept.rate).toBe('0.9');
	});
	it('édition : ligne non retouchée → provenance relue conservée, montant changé compris', () => {
		const draft = lineResponseToDraft({
			id: 1,
			accountId: 10,
			lineOrder: 1,
			debit: '88.4600',
			credit: '0.0000',
			projectId: null,
			letteringKey: null,
			letteringCode: null,
			letteringOrigin: null,
			foreign: { currency: 'USD', amount: '100', rate: '0.8846', rateSource: 'afc' },
		});
		const f = { ...draft.foreign!, amount: '200' };
		expect(rateSourceToSend(f, '2026-03-15', false)).toBe('afc');
		expect(f.rateProposed).toBe(true);
		// Une ligne relue manual n'est pas proposée.
		expect(
			foreignFromResponse({
				id: 2,
				accountId: 10,
				lineOrder: 2,
				debit: '1.0000',
				credit: '0',
				projectId: null,
				letteringKey: null,
				letteringCode: null,
				letteringOrigin: null,
				foreign: { currency: 'USD', amount: '1', rate: '1', rateSource: 'manual' },
			})!.rateProposed,
		).toBe(false);
		// Date changée : la provenance relue n'est plus gardée.
		expect(rateSourceToSend(f, '2026-03-15', true)).toBe('manual');
	});
	it('« Garder mon cours » → manual', () => {
		expect(rateSourceToSend({ ...newForeign('USD'), rate: '1', afcRate: '1', afcMonth: '2026-03-01', forceManual: true }, '2026-03-02', false)).toBe('manual');
	});
});

describe('CHF saisi à la main (A32)', () => {
	const date = '2026-03-15';
	const afc = { ...newForeign('USD'), amount: '100', afcRate: '0.91193', afcMonth: '2026-03-01' };
	it('CHF qui redonne afcRate → cours afcRate, afc', () => {
		const f = applyChfEntry(afc, '91.19', date);
		expect(f.rate).toBe('0.91193');
		expect(rateSourceToSend(f, date, false)).toBe('afc');
	});
	it('CHF qui ne le redonne pas → cours de rateFromChf, manual', () => {
		const f = applyChfEntry(afc, '91.50', date);
		expect(f.rate).toBe('0.915');
		expect(rateSourceToSend(f, date, false)).toBe('manual');
	});
	it('mois sans afcRate → rateFromChf, manual', () => {
		const f = applyChfEntry({ ...afc, afcRate: null }, '91.19', date);
		expect(f.rate).toBe('0.9119');
		expect(rateSourceToSend(f, date, false)).toBe('manual');
	});
	it('CHF impossible → chfError', () => {
		// Aucun cours ne redonne un CHF à trois décimales (vecteur de la fixture).
		expect(applyChfEntry(afc, '0.123', date).chfError).toBe('FX_AMOUNT_MISMATCH');
		expect(applyChfEntry({ ...afc, amount: '' }, '10', date).chfError).toBe('FX_VALUE_OUT_OF_RANGE');
	});	it('CHF refusé : le CHF de l’ancien cours n’est ni calculé ni envoyé (37-3b-CR-P1-1)', () => {
		const withRate = { ...afc, rate: '0.9' };
		const f = applyChfEntry(withRate, '0.123', date);
		expect(f.rate).toBe('0.9');
		expect(foreignChf(f)).toBeNull();
		const line = syncDerivedChf({ accountId: 10, debit: '90.00', credit: '', foreign: f });
		expect([line.debit, line.credit]).toEqual(['', '']);
	});
	it('saisie du CHF touche après touche : la saisie brute reste affichée (37-3b-CR-P1-2)', () => {
		const noAfc = { ...afc, afcRate: null };
		// « 9 » puis « 95 » puis « 950 » : le cours se déduit à chaque frappe et
		// la saisie brute reste, jamais réécrite en « 9.00 ».
		let f = applyChfEntry(noAfc, '9', date);
		expect(f.chfEntry).toBe('9');
		expect(f.chfError).toBeNull();
		f = applyChfEntry(f, '95', date);
		f = applyChfEntry(f, '950', date);
		expect(f.chfEntry).toBe('950');
		expect(f.rate).toBe('9.5');
		const line = syncDerivedChf({ accountId: 10, debit: '', credit: '', foreign: f });
		expect(line.debit).toBe('950.00');
		// Le focus quitte le champ : la saisie résolue cède la place au CHF dérivé.
		expect(commitChfEntry(f).chfEntry).toBeNull();
		// Une saisie refusée reste affichée avec son refus.
		const bad = applyChfEntry(noAfc, '0.123', date);
		expect(commitChfEntry(bad).chfEntry).toBe('0.123');
	});
});

describe('écriture relue : montant normalisé par le serveur (37-3b-CR-P1-3)', () => {
	const line = (amount: string): JournalEntryLineResponse => ({
		id: 1,
		accountId: 10,
		lineOrder: 1,
		debit: '1.0900',
		credit: '0',
		projectId: null,
		letteringKey: null,
		letteringCode: null,
		letteringOrigin: null,
		foreign: { currency: 'USD', amount, rate: '0.8846', rateSource: 'manual' },
	});
	it('« 1.234 » (trois décimales) relu à quatre décimales, jamais ambigu', () => {
		const f = foreignFromResponse(line('1.234'))!;
		expect(f.amount).toBe('1.2340');
		expect(foreignLineProblem(f)).toBeNull();
		expect(lineToPayload({ accountId: 10, debit: '1.09', credit: '', projectId: null, foreign: f }, '2026-03-15', false)).not.toBeNull();
	});
	it('les autres montants restent tels quels', () => {
		expect(foreignFromResponse(line('100'))!.amount).toBe('100');
		expect(foreignFromResponse(line('1234.5'))!.amount).toBe('1234.5');
	});
});

describe('changement de compte (U2)', () => {
	it('CHF → devise : partie en devise neuve, montant vide', () => {
		const l = applyAccountChange({ accountId: 1, debit: '50', credit: '' } as FxLineLike, USD);
		expect(l.foreign?.currency).toBe('USD');
		expect(l.foreign?.amount).toBe('');
		expect(l.debit).toBe('');
	});
	it('devise → CHF : foreign retiré (aucune clé dans le corps)', () => {
		const l = applyAccountChange(fxLine(USD, { amount: '100', rate: '0.8846' }), CHF);
		expect(l.foreign).toBeUndefined();
		const payload = lineToPayload({ ...l, debit: '10', projectId: null }, '2026-03-15', false);
		expect(payload).not.toHaveProperty('foreign');
	});
	it('USD → EUR (ligne afc relue) : montant et cours réinitialisés, devise EUR', () => {
		const relue = fxLine(USD, { amount: '100', rate: '0.8846', loadedRateSource: 'afc' });
		const l = applyAccountChange(relue, EUR);
		expect(l.foreign).toMatchObject({ currency: 'EUR', amount: '', rate: '', rateProposed: true, touched: true });
	});
	it('USD → USD : montant et cours gardés, retouche', () => {
		const l = applyAccountChange(fxLine(USD, { amount: '100', rate: '0.8846', loadedRateSource: 'afc' }), USD_OWN);
		expect(l.foreign).toMatchObject({ currency: 'USD', amount: '100', rate: '0.8846', touched: true });
		expect(l.debit).toBe('88.46');
	});
});

describe('cours de la société sous le champ et ↑', () => {
	const date = '2026-03-15';
	const f = applyLookup({ ...newForeign('USD'), rate: '0.9', rateProposed: false }, '2026-03-01', {
		kind: 'rate',
		rate: { rate: '0.8846', source: 'afc', month: '2026-03-01' },
	});
	it('affiché quand il diffère, absent pendant le lookup du mois, absent pour un mois quitté', () => {
		expect(companyRateHint(f, date, false)?.rate).toBe('0.8846');
		expect(companyRateHint(f, date, true)).toBeNull();
		expect(companyRateHint(f, '2026-04-02', false)).toBeNull();
	});
	it('↑ remet la ligne au cours de la société, proposée', () => {
		const up = adoptCompanyRate(f, f.company!);
		expect(up.rate).toBe('0.8846');
		expect(up.rateProposed).toBe(true);
		expect(rateSourceToSend(up, date, false)).toBe('afc');
		expect(companyRateHint(up, date, false)).toBeNull();
	});
});

describe('une devise par écriture (R7)', () => {
	it('avec une ligne USD, les autres lignes offrent CHF et USD, aucun EUR', () => {
		const lines = [fxLine(USD, {}), { accountId: null, debit: '', credit: '' }];
		const allowed = allowedCurrencies(lines, 1, new Set());
		const offered = accountsForCurrencies(ACCOUNTS, allowed).map((a) => a.id);
		expect(offered).toContain(CHF.id);
		expect(offered).toContain(USD_OWN.id);
		expect(offered).not.toContain(EUR.id);
		// La ligne USD elle-même peut changer de devise.
		expect(allowedCurrencies(lines, 0, new Set())).toBeNull();
	});
	it('retirer la ligne en devise rend toutes les devises', () => {
		expect(allowedCurrencies([{ accountId: null, debit: '', credit: '' }], 0, new Set())).toBeNull();
	});
	it('écriture relue à EUR et USD : CHF, EUR et USD, aucune autre', () => {
		const allowed = allowedCurrencies([fxLine(USD, {})], 0, new Set(['EUR', 'USD']));
		expect([...allowed!].sort()).toEqual(['EUR', 'USD']);
	});
});

describe('ligne d’écart (R8, S2, A12)', () => {
	const bal = (lines: FxLineLike[]) => computeBalance(lines, journalAmountReader);
	it('compte à comptes propres → ligne sur ses comptes, au crédit du gain', () => {
		const lines = [fxLine(USD_OWN, { amount: '100', rate: '0.8846' }), { accountId: 1, debit: '', credit: '88.40' }];
		const b = bal(lines);
		const p = planDifferenceLine(lines, ACCOUNTS, COMPANY, b.totalDebit, b.totalCredit);
		expect(p).toMatchObject({ kind: 'line', accountId: 41, side: 'credit', amount: '0.06', origin: 'own' });
	});
	it('compte par défaut → comptes de la société, perte au débit', () => {
		const lines = [fxLine(USD, { amount: '100', rate: '0.8846' }), { accountId: 1, debit: '', credit: '90.00' }];
		const b = bal(lines);
		const p = planDifferenceLine(lines, ACCOUNTS, COMPANY, b.totalDebit, b.totalCredit);
		expect(p).toMatchObject({ kind: 'line', accountId: 32, side: 'debit', amount: '1.54', origin: 'company', beyondRounding: true });
	});
	it('deux comptes USD à paires différentes → le compte de bilan, avis', () => {
		const lines = [
			fxLine(SALES_USD, { amount: '10', rate: '0.9', side: 'credit' }),
			fxLine(USD_OWN, { amount: '10', rate: '0.9011' }),
		];
		const b = bal(lines);
		const p = planDifferenceLine(lines, ACCOUNTS, COMPANY, b.totalDebit, b.totalCredit);
		expect(p).toMatchObject({ kind: 'line', holderNumber: '1023', mixedPairs: true, beyondRounding: false });
	});
	it('compte propre archivé → explication, aucune ligne', () => {
		const archived = ACCOUNTS.map((a) => (a.id === 41 ? { ...a, active: false } : a));
		const lines = [fxLine(USD_OWN, { amount: '100', rate: '0.8846' }), { accountId: 1, debit: '', credit: '88.40' }];
		const b = bal(lines);
		expect(planDifferenceLine(lines, archived, COMPANY, b.totalDebit, b.totalCredit)).toEqual({
			kind: 'invalid',
			holderNumber: '1023',
			origin: 'own',
		});
	});
	it('comptes de la société inconnus → missing ; équilibrée → none', () => {
		const lines = [fxLine(USD, { amount: '100', rate: '0.8846' }), { accountId: 1, debit: '', credit: '88.40' }];
		const b = bal(lines);
		expect(planDifferenceLine(lines, ACCOUNTS, null, b.totalDebit, b.totalCredit).kind).toBe('invalid');
		expect(planDifferenceLine(lines, ACCOUNTS, COMPANY, new Big(1), new Big(1)).kind).toBe('none');
	});
	it('seuil n × 0.005 : lignes CHF dérivées (chfFromForeign) comptées', () => {
		const lines: FxLineLike[] = [
			{ accountId: 1, debit: '10.88', credit: '', chfFromForeign: { rate: '0.885' } },
			{ accountId: 2, debit: '0.89', credit: '', chfFromForeign: { rate: '0.885' } },
			fxLine(USD, { amount: '13.29', rate: '0.885', side: 'credit' }),
		];
		const b = bal(lines);
		const p = planDifferenceLine(lines, ACCOUNTS, COMPANY, b.totalDebit, b.totalCredit);
		expect(p).toMatchObject({ kind: 'line', amount: '0.01', beyondRounding: false });
		// Sans l'indicateur (montants retouchés), une seule ligne dérivée : 0.01 > 0.005.
		const retouched = lines.map((l) => ({ ...l, chfFromForeign: null }));
		const p2 = planDifferenceLine(retouched, ACCOUNTS, COMPANY, b.totalDebit, b.totalCredit);
		expect(p2).toMatchObject({ beyondRounding: true });
	});
});

describe('corps et rang des refus', () => {
	it('la ligne en devise porte devise, montant et cours par toFixed, provenance', () => {
		const l = fxLine(USD, { amount: "1'078.40", rate: '0.88460' });
		expect(lineToPayload({ ...l, projectId: null }, '2026-03-15', false)).toEqual({
			accountId: 10,
			debit: '953.95',
			credit: '0',
			projectId: null,
			foreign: { currency: 'USD', amount: '1078.4', rate: '0.8846', rateSource: 'manual' },
		});
	});
	it('le rang du corps (sans lignes vides) se traduit par la table des lignes envoyées', () => {
		expect(formIndexOfServerLine([0, 2, 3], 2)).toBe(2);
		expect(formIndexOfServerLine([0, 2, 3], 9)).toBeNull();
		expect(formIndexOfServerLine([0, 2, 3], 'x')).toBeNull();
	});
});
