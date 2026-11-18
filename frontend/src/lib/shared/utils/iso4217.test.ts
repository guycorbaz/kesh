// Story 37-1 (AC-4, AC-7) — la liste ISO 4217 de l'écran.
import { describe, it, expect } from 'vitest';
import { readFileSync } from 'node:fs';
import { FOREIGN_CURRENCIES, ISO_4217_CURRENCIES, isIso4217Currency } from './iso4217';

describe('ISO_4217_CURRENCIES', () => {
	it('ne porte que des codes de trois lettres majuscules, triés, sans doublon', () => {
		for (const c of ISO_4217_CURRENCIES) expect(c).toMatch(/^[A-Z]{3}$/);
		expect([...ISO_4217_CURRENCIES].sort()).toEqual([...ISO_4217_CURRENCIES]);
		expect(new Set(ISO_4217_CURRENCIES).size).toBe(ISO_4217_CURRENCIES.length);
	});

	it('porte les monnaies usuelles et le CHF', () => {
		for (const c of ['CHF', 'EUR', 'USD', 'GBP', 'JPY']) expect(isIso4217Currency(c)).toBe(true);
	});

	it('exclut codes réservés, métaux, droits de tirage, unités de compte et de fonds', () => {
		const exclus = [
			'XXX', 'XTS', 'XAU', 'XAG', 'XPT', 'XPD', 'XDR', 'XBA', 'XBB', 'XBC', 'XBD', 'XSU', 'XUA',
			'BOV', 'CHE', 'CHW', 'CLF', 'COU', 'MXV', 'USN', 'UYI', 'UYW',
		];
		for (const c of exclus) expect(isIso4217Currency(c)).toBe(false);
	});

	it('n’admet pas un code en minuscules (la normalisation est l’affaire de l’appelant)', () => {
		expect(isIso4217Currency('usd')).toBe(false);
	});
});

describe('miroir de la liste serveur', () => {
	it('est identique, au code et à l’ordre près, à `ISO_4217_CURRENCIES` de kesh-core', () => {
		const source = readFileSync('../crates/kesh-core/src/currency.rs', 'utf-8');
		const bloc = /pub const ISO_4217_CURRENCIES: &\[&str\] = &\[([\s\S]*?)\];/.exec(source);
		expect(bloc, 'constante introuvable dans currency.rs').not.toBeNull();
		const serveur = [...bloc![1].matchAll(/"([A-Z]{3})"/g)].map((m) => m[1]);
		// Garde anti-test-muet : un extracteur cassé rendrait une liste vide.
		expect(serveur.length).toBeGreaterThan(100);
		expect([...ISO_4217_CURRENCIES]).toEqual(serveur);
	});
});

describe('FOREIGN_CURRENCIES', () => {
	it('est la liste sans le CHF, la devise principale choisie en laissant le champ vide', () => {
		expect(FOREIGN_CURRENCIES).not.toContain('CHF');
		expect(FOREIGN_CURRENCIES.length).toBe(ISO_4217_CURRENCIES.length - 1);
	});
});
