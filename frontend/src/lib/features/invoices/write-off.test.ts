/**
 * Les natures d'un solde et le pré-contrôle des comptes — Story 25-4-d2b (#490).
 *
 * ⚠️ Chaque test nomme la MUTATION qu'il attrape.
 */
import { describe, it, expect, vi } from 'vitest';

vi.mock('$lib/shared/utils/i18n.svelte', () => ({
	i18nMsg: (_k: string, fallback: string) => fallback,
}));

import { isNatureOffered, missingAccount, writeOffNatureLabel } from './write-off';
import { formatExactAmount, hasSubCentime } from './invoice-helpers';
import type { InvoiceSettingsResponse } from './invoices.types';

function settings(partial: Partial<InvoiceSettingsResponse> = {}): InvoiceSettingsResponse {
	return {
		defaultDiscountAccountId: 1,
		defaultBankFeesAccountId: 2,
		defaultBadDebtAccountId: 3,
		defaultRoundingAccountId: 4,
		...partial,
	} as InvoiceSettingsResponse;
}

describe('hasSubCentime et formatExactAmount', () => {
	it('« 68.1000 » n’a pas de fraction de centime — comparaison de VALEURS (mutation : comparaison de chaînes)', () => {
		expect(hasSubCentime('68.1000')).toBe(false);
		expect(hasSubCentime('10.0050')).toBe(true);
		expect(hasSubCentime('0.0040')).toBe(true);
	});

	it('un reste à fraction de centime s’affiche aux quatre décimales (#490)', () => {
		expect(formatExactAmount('0.0040')).toBe('0.0040');
		expect(formatExactAmount('68.1000')).toBe('68.10');
	});
});

describe('isNatureOffered', () => {
	it('le reste d’arrondi n’est proposé que sous 0.05, sur le reste EXACT (mutation : comparaison sur l’affiché)', () => {
		expect(isNatureOffered('rounding', '0.0450')).toBe(true);
		expect(isNatureOffered('rounding', '0.0500')).toBe(false);
		expect(isNatureOffered('discount', '1000.00')).toBe(true);
	});
});

describe('missingAccount', () => {
	it('le compte de la nature manque', () => {
		expect(missingAccount(settings({ defaultBadDebtAccountId: null }), 'bad_debt', '10.00')).toBe(
			'nature',
		);
		expect(missingAccount(settings(), 'bad_debt', '10.00')).toBeNull();
	});

	it('le compte d’arrondi est exigé pour TOUTE nature sur un reste à fraction de centime (mutation : vérifié pour `rounding` seulement)', () => {
		const sansArrondi = settings({ defaultRoundingAccountId: null });
		expect(missingAccount(sansArrondi, 'discount', '10.0050')).toBe('rounding');
		expect(missingAccount(sansArrondi, 'discount', '68.1000')).toBeNull();
		expect(missingAccount(sansArrondi, 'rounding', '0.0040')).toBe('nature');
	});

	it('réglages inconnus : aucun pré-contrôle (mutation : quatre natures désactivées)', () => {
		expect(missingAccount(null, 'discount', '10.0050')).toBeNull();
	});
});

describe('writeOffNatureLabel', () => {
	it('un intitulé par nature', () => {
		expect(writeOffNatureLabel('discount')).toBe('Escompte accordé');
		expect(writeOffNatureLabel('bank_fees')).toBe('Frais bancaires');
		expect(writeOffNatureLabel('bad_debt')).toBe('Perte sur débiteur');
		expect(writeOffNatureLabel('rounding')).toBe("Reste d'arrondi");
	});
});
