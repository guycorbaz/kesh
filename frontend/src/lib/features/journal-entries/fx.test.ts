/**
 * Story 37-3b (NFR-37-3b-6) — `fx.ts`, jumeau de `kesh-core` : les fixtures
 * **partagées** de 37-3a et 37-2 sont lues ici dans `crates/kesh-core/tests/fixtures/`
 * (même fichier que le test Rust, aucune copie).
 */
import { readFileSync } from 'node:fs';
import { join } from 'node:path';
import Big from 'big.js';
import { describe, expect, it } from 'vitest';
import {
	computeChf,
	openingRateDate,
	rateFromChf,
	readForeignAmount,
	readRate,
	RATE_FROM_CHF_MAX_STEPS,
} from './fx';

const FIXTURES = join('..', 'crates', 'kesh-core', 'tests', 'fixtures');

function fixture<T>(name: string): T {
	return JSON.parse(readFileSync(join(FIXTURES, name), 'utf8')) as T;
}

interface Arrondi {
	case: string;
	amount: string;
	rate: string;
	chf?: string;
	error?: string;
}
interface CoursDeduit {
	case: string;
	amount: string;
	chf: string;
	rate?: string;
	error?: string;
	expected?: string;
}
interface Ouverture {
	startDate: string;
	openingRateKind: 'day' | 'month';
	openingDate: string;
}

describe('computeChf — fixture fx_rounding.json', () => {
	const vecteurs = fixture<Arrondi[]>('fx_rounding.json');
	it('lit la fixture complète', () => {
		expect(vecteurs.length).toBeGreaterThanOrEqual(20);
	});
	for (const v of vecteurs) {
		it(v.case, () => {
			const r = computeChf(v.amount, v.rate);
			if (v.chf !== undefined) {
				expect(r.ok).toBe(true);
				if (r.ok) expect(r.value.eq(new Big(v.chf))).toBe(true);
			} else {
				expect(r).toEqual({ ok: false, error: v.error });
			}
		});
	}
	it('un CHF arrondi nul rend FX_LINE_ROUNDS_TO_ZERO', () => {
		expect(computeChf('0.001', '1')).toEqual({ ok: false, error: 'FX_LINE_ROUNDS_TO_ZERO' });
	});
});

describe('rateFromChf — fixture fx_rate_from_chf.json', () => {
	const vecteurs = fixture<CoursDeduit[]>('fx_rate_from_chf.json');
	it('lit la fixture et partage la borne de pas', () => {
		expect(vecteurs.length).toBeGreaterThanOrEqual(10);
		expect(RATE_FROM_CHF_MAX_STEPS).toBe(10);
	});
	for (const v of vecteurs) {
		it(v.case, () => {
			const r = rateFromChf(v.amount, v.chf);
			if (v.rate !== undefined) {
				expect(r.ok).toBe(true);
				if (r.ok) expect(r.value.eq(new Big(v.rate))).toBe(true);
			} else {
				expect(r.ok).toBe(false);
				if (!r.ok) {
					expect(r.error).toBe(v.error);
					if (v.expected !== undefined) expect(r.expected?.eq(new Big(v.expected))).toBe(true);
				}
			}
		});
	}
});

describe('openingRateDate — fixture fx_opening_dates.json (37-2)', () => {
	const vecteurs = fixture<Ouverture[]>('fx_opening_dates.json');
	it('lit la fixture (jours et mois, plusieurs débuts)', () => {
		expect(vecteurs.length).toBeGreaterThanOrEqual(8);
		expect(new Set(vecteurs.map((v) => v.openingRateKind))).toEqual(new Set(['day', 'month']));
	});
	for (const v of vecteurs) {
		it(`${v.startDate} ${v.openingRateKind} → ${v.openingDate}`, () => {
			expect(openingRateDate(v.startDate, v.openingRateKind)).toBe(v.openingDate);
		});
	}
	it('ouverture au 15.03 en genre month : mars (veille 14.03), jamais février', () => {
		expect(openingRateDate('2025-03-15', 'month')).toBe('2025-03-01');
	});
});

describe('lectures de saisie', () => {
	it('readRate : dix décimales, sept chiffres entiers, > 0 et ≤ 10⁶', () => {
		expect(readRate('')).toBe('empty');
		expect((readRate('0.9123456789') as Big).toFixed()).toBe('0.9123456789');
		expect(readRate('0,8846')).not.toBeNull();
		expect(readRate('0')).toBeNull();
		expect(readRate('0.12345678901')).toBeNull();
		expect(readRate('1000000.1')).toBeNull();
		expect(readRate('abc')).toBeNull();
	});
	it('readForeignAmount : lecture du journal, borne 10¹³', () => {
		expect(readForeignAmount('')).toBe('empty');
		expect((readForeignAmount("1'078.40") as Big).toFixed()).toBe('1078.4');
		expect(readForeignAmount('1.234')).toBeNull(); // ambigu
		expect(readForeignAmount('10000000000001')).toBeNull();
	});
});
