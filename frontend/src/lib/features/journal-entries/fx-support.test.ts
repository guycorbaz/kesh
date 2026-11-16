/**
 * Story 37-3b — modules d'appui des écrans en devise : cache des cours
 * (`rate-lookup.ts`), messages des refus (`fx-refusals.ts`), classement des
 * refus en édition et « Supprimer » d'une réévaluation (`form-helpers.ts`),
 * assistant TVA en devise (`vat-purchase.ts`), assistant Conversion
 * (`conversion.ts`), affichage (`fx-display.ts`).
 */
import { describe, expect, it, vi } from 'vitest';

vi.mock('$lib/shared/utils/i18n.svelte', () => ({
	i18nMsg: (_k: string, fallback: string, args?: Record<string, unknown>) =>
		fallback.replace(/\{ \$(\w+) \}/g, (_m: string, k: string) => String(args?.[k] ?? '')),
}));

import type { AccountResponse } from '$lib/features/accounts/accounts.types';
import { RateLookupCache, isFetchable } from './rate-lookup';
import { fxRefusalMessage, refusalEntry, refusalLine } from './fx-refusals';
import { canDeleteEntry, editRefusalOutcome } from './form-helpers';
import { foreignPurchaseVatEntry } from './vat-purchase';
import { newForeign } from './fx-lines';
import { conversionField, planConversion, type ConversionColumn } from './conversion';
import { foreignLineLabel } from './fx-display';

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

describe('RateLookupCache', () => {
	const lookupOk = { currency: 'USD', kind: 'month', rateDate: '2026-03-01', effectiveRate: '0.8846', source: 'afc', publishedRate: null, publishedUnit: null, sourceDate: null };
	it('mémoïse par (devise, mois), résultats négatifs compris', async () => {
		const lookup = vi.fn().mockRejectedValue({ code: 'MULTI_CURRENCY_DISABLED', status: 409, message: '' });
		const cache = new RateLookupCache({ lookup, fetch: vi.fn(), canFetch: () => true, today: () => '2026-03-15' });
		await cache.get('USD', '2026-03-02');
		await cache.get('USD', '2026-03-28');
		expect(lookup).toHaveBeenCalledTimes(1);
	});
	it('404 RATE_UNAVAILABLE → un POST /fetch puis relecture', async () => {
		const lookup = vi
			.fn()
			.mockRejectedValueOnce({ code: 'RATE_UNAVAILABLE', status: 404, message: '' })
			.mockResolvedValueOnce(lookupOk);
		const fetch = vi.fn().mockResolvedValue({});
		const cache = new RateLookupCache({ lookup, fetch, canFetch: () => true, today: () => '2026-03-15' });
		const r = await cache.get('USD', '2026-03-10');
		expect(fetch).toHaveBeenCalledWith({ year: 2026, month: 3 });
		expect(r.outcome).toEqual({ kind: 'rate', rate: { rate: '0.8846', source: 'afc', month: '2026-03-01' } });
	});
	it('rôle Consultation, mois au-delà du mois suivant, échec du fetch → saisie manuelle', async () => {
		const unavailable = () => vi.fn().mockRejectedValue({ code: 'RATE_UNAVAILABLE', status: 404, message: '' });
		const fetch = vi.fn();
		let r = await new RateLookupCache({ lookup: unavailable(), fetch, canFetch: () => false, today: () => '2026-03-15' }).get('USD', '2026-03-01');
		expect(r.note).toBe('manual-required');
		r = await new RateLookupCache({ lookup: unavailable(), fetch, canFetch: () => true, today: () => '2026-03-15' }).get('USD', '2026-05-01');
		expect(r.note).toBe('manual-required');
		expect(fetch).not.toHaveBeenCalled();
		const failing = vi.fn().mockRejectedValue({ code: 'RATE_NOT_PUBLISHED', status: 404, message: '' });
		r = await new RateLookupCache({ lookup: unavailable(), fetch: failing, canFetch: () => true, today: () => '2026-03-15' }).get('USD', '2026-04-01');
		expect(failing).toHaveBeenCalledTimes(1);
		expect(r.note).toBe('manual-required');
	});
	it('refresh remplace l’entrée négative mémoïsée', async () => {
		const lookup = vi
			.fn()
			.mockRejectedValueOnce({ code: 'X', status: 500, message: '' })
			.mockResolvedValueOnce(lookupOk);
		const cache = new RateLookupCache({ lookup, fetch: vi.fn().mockResolvedValue({}), canFetch: () => true, today: () => '2026-03-15' });
		expect((await cache.get('USD', '2026-03-01')).outcome.kind).toBe('none');
		expect((await cache.refresh('USD', '2026-03-01')).outcome.kind).toBe('rate');
		expect((await cache.get('USD', '2026-03-20')).outcome.kind).toBe('rate');
	});
	it('isFetchable : 2000-01 au mois suivant ; un jour strictement passé', () => {
		expect(isFetchable('month', '1999-12-01', '2026-03-15')).toBe(false);
		expect(isFetchable('month', '2026-04-01', '2026-03-15')).toBe(true);
		expect(isFetchable('month', '2026-05-01', '2026-03-15')).toBe(false);
		expect(isFetchable('month', '2027-01-01', '2026-12-31')).toBe(true);
		expect(isFetchable('day', '2026-03-14', '2026-03-15')).toBe(true);
		expect(isFetchable('day', '2026-03-15', '2026-03-15')).toBe(false);
	});
});

describe('refus propres aux devises', () => {
	it('messages propres : devise changée, compte devenu CHF, exercice mono-devise, cours', () => {
		expect(fxRefusalMessage('FX_CURRENCY_MISMATCH', { line: 1, expected: 'EUR' }, 'srv')).toContain('tenu en EUR');
		expect(fxRefusalMessage('FX_CURRENCY_MISMATCH', {}, 'srv')).toContain('Rechargez la page');
		expect(fxRefusalMessage('FX_ACCOUNT_IN_CHF', { line: 2 }, 'srv')).toContain('Rechargez la page');
		expect(fxRefusalMessage('MULTI_CURRENCY_DISABLED', { fiscalYearId: 3 }, 'srv')).toContain('mono-devise');
		expect(fxRefusalMessage('FX_RATE_NOT_AFC', { expectedRate: '0.8846' }, 'srv')).toContain('0.8846');
		expect(fxRefusalMessage('FX_RATE_NOT_AFC', { expectedRate: null }, 'srv')).toContain('propre cours');
		expect(fxRefusalMessage('RATE_UNAVAILABLE', { currency: 'USD', month: '2026-03-01' }, 'srv')).toContain('03.2026');
		expect(fxRefusalMessage('FX_AMOUNT_MISMATCH', {}, 'message du serveur')).toBe('message du serveur');
	});
	it('details.line et details.entry', () => {
		expect(refusalLine({ line: 2 })).toBe(2);
		expect(refusalLine({ line: 0 })).toBeNull();
		expect(refusalEntry({ entry: 2 })).toBe(2);
		expect(refusalEntry({})).toBeNull();
	});
	it('editRefusalOutcome : réévaluation stale, refus FX stay', () => {
		expect(editRefusalOutcome('ENTRY_IS_FX_REVALUATION')).toBe('stale');
		for (const code of [
			'RATE_UNAVAILABLE',
			'FX_RATE_NOT_AFC',
			'FX_AMOUNT_MISMATCH',
			'FX_CURRENCY_MISMATCH',
			'FX_ACCOUNT_IN_CHF',
			'FX_MULTIPLE_CURRENCIES',
			'FX_VALUE_OUT_OF_RANGE',
			'FX_LINE_ROUNDS_TO_ZERO',
			'ACCOUNT_REQUIRES_CURRENCY',
			'MULTI_CURRENCY_DISABLED',
		]) {
			expect(editRefusalOutcome(code), code).toBe('stay');
		}
		expect(editRefusalOutcome('AUTRE')).toBe('other');
	});
});

describe('« Supprimer » d’une réévaluation (AC-3)', () => {
	const reval = {
		modifiable: false,
		modificationBlockedBy: 'ENTRY_IS_FX_REVALUATION',
		entryDate: '2026-12-31',
		lines: [{ letteringKey: null }],
	};
	it('libre : offert ; période verrouillée ou lettrée : absent ; borne inconnue : absent', () => {
		expect(canDeleteEntry(reval, null)).toBe(true);
		expect(canDeleteEntry(reval, '2026-12-31')).toBe(false);
		expect(canDeleteEntry({ ...reval, lines: [{ letteringKey: 4 }] }, null)).toBe(false);
		expect(canDeleteEntry(reval, undefined)).toBe(false);
		expect(canDeleteEntry({ ...reval, modificationBlockedBy: 'PERIOD_LOCKED' }, null)).toBe(false);
		expect(canDeleteEntry({ ...reval, modifiable: true }, undefined)).toBe(true);
	});
});

describe('assistant TVA achat en devise (S5)', () => {
	const params = (ht: string) => ({
		chargeAccountId: 1,
		htAmount: ht,
		ratePercent: '8.1',
		counterpartyAccountId: 2,
		recoverableAccountId: 3,
	});
	const template = (rate: string) => ({ ...newForeign('USD'), rate });
	it('USD 100, 8,1 %, 0.8846, charge CHF, contrepartie USD : équilibré', () => {
		const r = foreignPurchaseVatEntry(params('100'), { template: template('0.8846'), chargeIsForeign: false, counterpartyIsForeign: true });
		expect(r && 'lines' in r).toBe(true);
		if (!r || !('lines' in r)) return;
		expect(r.lines.map((l) => [l.debit, l.credit])).toEqual([
			['88.46', ''],
			['7.17', ''],
			['', '95.63'],
		]);
		expect(r.lines[2].foreign).toMatchObject({ currency: 'USD', amount: '108.10', side: 'credit' });
		expect(r.lines[0].chfFromForeign).toEqual({ rate: '0.8846' });
		expect(r.lines[1].chfFromForeign).toEqual({ rate: '0.8846' });
	});
	it('cas x.xx5 : USD 12.29, 0.8850 → débit 11.77, crédit 11.76 (un centime d’écart)', () => {
		const r = foreignPurchaseVatEntry(params('12.29'), { template: template('0.8850'), chargeIsForeign: false, counterpartyIsForeign: true });
		if (!r || !('lines' in r)) throw new Error('attendu des lignes');
		expect(r.lines.map((l) => l.debit || l.credit)).toEqual(['10.88', '0.89', '11.76']);
		expect(r.lines[2].foreign?.amount).toBe('13.29');
	});
	it('ligne CHF dérivée arrondie à 0.00 → FX_LINE_ROUNDS_TO_ZERO ; exempt → deux lignes', () => {
		expect(foreignPurchaseVatEntry(params('0.05'), { template: template('0.0057'), chargeIsForeign: false, counterpartyIsForeign: true })).toEqual({
			error: 'FX_LINE_ROUNDS_TO_ZERO',
		});
		const ex = foreignPurchaseVatEntry({ ...params('100'), ratePercent: '0' }, { template: template('0.9'), chargeIsForeign: true, counterpartyIsForeign: true });
		if (!ex || !('lines' in ex)) throw new Error('attendu des lignes');
		expect(ex.lines).toHaveLength(2);
		expect(ex.isExempt).toBe(true);
	});
});

describe('assistant Conversion (AC-2 bis)', () => {
	const EUR = acc({ id: 21, number: '1021', currency: 'EUR' });
	const USD = acc({ id: 22, number: '1022', currency: 'USD' });
	const USD_OWN = acc({ id: 23, number: '1023', currency: 'USD', fxGainAccountId: 41, fxLossAccountId: 42 });
	const USD_BIS = acc({ id: 24, number: '1024', currency: 'USD' });
	const FEE_USD = acc({ id: 25, number: '6911', accountType: 'Expense', currency: 'USD' });
	const TRANSIT = acc({ id: 30, number: '1090' });
	const GAIN = acc({ id: 31, number: '6840', accountType: 'Revenue' });
	const LOSS = acc({ id: 32, number: '6940', accountType: 'Expense' });
	const OWN_GAIN = acc({ id: 41, number: '6841', accountType: 'Revenue' });
	const OWN_LOSS = acc({ id: 42, number: '6941', accountType: 'Expense' });
	const ctx = {
		accounts: [EUR, USD, USD_OWN, USD_BIS, FEE_USD, TRANSIT, GAIN, LOSS, OWN_GAIN, OWN_LOSS],
		company: { fxGainAccountId: 31, fxLossAccountId: 32, fxTransitAccountId: 30 },
		openFiscalYears: [{ startDate: '2026-01-01', endDate: '2026-12-31' }],
	};
	const labels = { first: 'C (1/2)', second: 'C (2/2)' };
	const col = (accountId: number, currency: string, amount: string, rate: string, date: string): ConversionColumn => ({
		date,
		accountId,
		foreign: { ...newForeign(currency), amount, rate },
		fees: [],
	});
	it('la maquette : 947.00 et 953.95, gain 6.95 sur les comptes de la société', () => {
		const p = planConversion(ctx, col(21, 'EUR', '1000', '0.9470', '2026-03-05'), col(22, 'USD', '1078.40', '0.8846', '2026-03-06'), labels);
		if (p.kind !== 'ready') throw new Error(JSON.stringify(p));
		expect(p.difference).toEqual({ accountId: 31, side: 'credit', amount: '6.95' });
		expect(p.entries[0].lines.map((l) => [l.accountId, l.debit, l.credit])).toEqual([
			[21, '0', '947.00'],
			[30, '947.00', '0'],
		]);
		expect(p.entries[1].lines.map((l) => [l.accountId, l.debit, l.credit])).toEqual([
			[22, '953.95', '0'],
			[30, '0', '947.00'],
			[31, '0', '6.95'],
		]);
		expect(p.entries[1].lines[0].foreign).toEqual({ currency: 'USD', amount: '1078.4', rate: '0.8846', rateSource: 'manual' });
	});
	it('compte USD à comptes propres → différence sur ses comptes', () => {
		const p = planConversion(ctx, col(21, 'EUR', '1000', '0.9470', '2026-03-05'), col(23, 'USD', '1000', '0.9', '2026-03-06'), labels);
		if (p.kind !== 'ready') throw new Error('prêt attendu');
		expect(p.difference).toEqual({ accountId: 42, side: 'debit', amount: '47.00' });
		expect(p.pairOrigin).toBe('own');
	});
	it('frais 6911 USD au débit (A33) : bouton actif, différence après le frais', () => {
		const c2 = { ...col(22, 'USD', '1000', '0.9', '2026-03-06'), fees: [{ accountId: 25, amount: '10' }] };
		const p = planConversion(ctx, col(21, 'EUR', '1000', '0.9470', '2026-03-05'), c2, labels);
		if (p.kind !== 'ready') throw new Error('prêt attendu');
		// 900 + 9 − 947 = −38 → perte 38.00.
		expect(p.difference).toEqual({ accountId: 32, side: 'debit', amount: '38.00' });
		expect(p.preview[1].map((l) => l.role)).toEqual(['currency', 'fee', 'transit', 'difference']);
	});
	it('frais sur un second compte USD (pas une charge) avec différence → désactivé, motif', () => {
		const c2 = { ...col(22, 'USD', '1000', '0.9', '2026-03-06'), fees: [{ accountId: 24, amount: '10' }] };
		const p = planConversion(ctx, col(21, 'EUR', '1000', '0.9470', '2026-03-05'), c2, labels);
		expect(p).toMatchObject({ kind: 'blocked', blockers: ['fee-second-foreign-account'] });
	});
	it('date hors exercice ouvert, même devise, paire invalide → motifs', () => {
		const out = planConversion(ctx, col(21, 'EUR', '1000', '0.9470', '2025-12-31'), col(22, 'USD', '1', '1', '2026-03-06'), labels);
		expect(out.kind === 'blocked' && out.blockers).toContain('date-outside-open-fiscal-year');
		const same = planConversion(ctx, col(22, 'USD', '1', '1', '2026-03-05'), col(23, 'USD', '1', '1', '2026-03-06'), labels);
		expect(same.kind === 'blocked' && same.blockers).toContain('same-currency');
		const archived = { ...ctx, accounts: ctx.accounts.map((a) => (a.id === 31 ? { ...a, active: false } : a)) };
		const pair = planConversion(archived, col(21, 'EUR', '1000', '0.947', '2026-03-05'), col(22, 'USD', '1000', '0.947', '2026-03-06'), labels);
		expect(pair).toMatchObject({ kind: 'blocked', blockers: ['pair-invalid'], pairOrigin: 'company' });
	});
	it('différence nulle : aucune ligne de différence', () => {
		const p = planConversion(ctx, col(21, 'EUR', '1000', '0.947', '2026-03-05'), col(22, 'USD', '1000', '0.947', '2026-03-06'), labels);
		if (p.kind !== 'ready') throw new Error('prêt attendu');
		expect(p.difference).toBeNull();
		expect(p.entries[1].lines).toHaveLength(2);
	});
	it('table des refus : compte en devise, frais, conversion, différence', () => {
		expect(conversionField(1, 1)).toEqual({ kind: 'currency' });
		expect(conversionField(2, 1)).toEqual({ kind: 'fee', index: 0 });
		expect(conversionField(3, 1)).toEqual({ kind: 'transit' });
		expect(conversionField(4, 1)).toEqual({ kind: 'difference' });
	});
});

describe('affichage (AC-3)', () => {
	it('montant avec son code, cours sans zéros terminaux, provenance', () => {
		expect(foreignLineLabel({ currency: 'USD', amount: '1078.4', rate: '0.8846000000', rateSource: 'afc' })).toBe(
			'USD 1’078.40 × 0.8846 · AFC',
		);
		expect(foreignLineLabel({ currency: 'EUR', amount: '0', rate: '0.95', rateSource: 'revaluation' })).toContain('réévaluation');
		expect(foreignLineLabel({ currency: 'EUR', amount: '1.2345', rate: '1', rateSource: 'manual' })).toBe('EUR 1.2345 × 1 · manuel');
	});
});
