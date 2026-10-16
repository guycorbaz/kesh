// Story 30-2b (#685, AC12, AC12-bis) — normalisation et classification QR-IBAN,
// une seule définition côté écran (C-30-2-20).

import { describe, it, expect } from 'vitest';
import { normalizeIban, qrIbanHasValidQrIid } from './iban';

/** Une chaîne écrite par points de code : rien d'invisible dans la source. */
const cp = (...codes: number[]) => String.fromCodePoint(...codes);

/**
 * Les sept blancs de l'ensemble ÉCRIT de l'AC12-bis — la même liste que le test Rust
 * `create_accepts_and_normalizes_payment_details` (`inbox_import_e2e.rs`) : `\s` en
 * JavaScript et `char::is_whitespace` en Rust ne coïncident pas d'eux-mêmes (U+0085,
 * U+FEFF), d'où une liste tenue des deux côtés.
 *
 * Écrits par points de code (revue P1 de la 30-2b, A6) : des caractères invisibles
 * littéraux pourraient disparaître sous un formateur, du code comme du test, et le
 * test se viderait avec le code sans rougir.
 */
const BLANCS = [0x0020, 0x0009, 0x00a0, 0x202f, 0x0085, 0x200b, 0xfeff].map((c) => cp(c));

describe('normalizeIban', () => {
	it('retire chacun des sept blancs de l’ensemble écrit', () => {
		expect(BLANCS).toHaveLength(7);
		for (const b of BLANCS) {
			expect(b).toHaveLength(1);
			expect(normalizeIban(`CH93${b}0076${b}2011${b}6238${b}5295${b}7`), JSON.stringify(b)).toBe(
				'CH9300762011623852957',
			);
		}
	});

	it('passe en majuscules, et une chaîne blanche devient vide', () => {
		expect(normalizeIban('ch93 0076 2011 6238 5295 7')).toBe('CH9300762011623852957');
		expect(normalizeIban(cp(0x20, 0xa0, 0x200b, 0x20))).toBe('');
	});
});

describe('qrIbanHasValidQrIid', () => {
	it('reconnaît un QR-IBAN suisse ou liechtensteinois (QR-IID 30000-31999)', () => {
		expect(qrIbanHasValidQrIid('CH44 3199 9123 0008 8901 2')).toBe(true);
		expect(qrIbanHasValidQrIid('CH4430000123000889012')).toBe(true);
		expect(qrIbanHasValidQrIid('LI2130000123456789012')).toBe(true);
	});

	it('refuse un IBAN ordinaire, hors plage QR', () => {
		expect(qrIbanHasValidQrIid('CH9300762011623852957')).toBe(false);
		expect(qrIbanHasValidQrIid('CH4429999123000889012')).toBe(false);
		expect(qrIbanHasValidQrIid('CH4432000123000889012')).toBe(false);
	});

	it('refuse un IBAN d’un autre pays, même à positions 5-9 dans la plage', () => {
		expect(qrIbanHasValidQrIid('DE89310004400532013000')).toBe(false);
	});

	it('refuse une saisie trop courte ou non numérique aux positions 5-9', () => {
		expect(qrIbanHasValidQrIid('CH44')).toBe(false);
		expect(qrIbanHasValidQrIid('CH44ABCDE')).toBe(false);
		expect(qrIbanHasValidQrIid('')).toBe(false);
	});
});
