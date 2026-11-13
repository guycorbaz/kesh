import { describe, expect, it } from 'vitest';

import {
	MAX_SEQ_PADDING,
	previewInvoiceNumber,
	validateFormatTemplate,
} from './invoice-number-format';

describe('previewInvoiceNumber', () => {
	it('rend F-{YEAR}-{SEQ:04}', () => {
		expect(previewInvoiceNumber('F-{YEAR}-{SEQ:04}', 2026, '2026', 42)).toBe('F-2026-0042');
	});

	it('rend FACT{SEQ}', () => {
		expect(previewInvoiceNumber('FACT{SEQ}', 2026, '2026', 1)).toBe('FACT1');
	});

	it('rend {FY}/{SEQ:06}', () => {
		expect(previewInvoiceNumber('{FY}/{SEQ:06}', 2026, '2025/2026', 42)).toBe('2025/2026/000042');
	});

	it('conserve le numéro complet si seq > padding', () => {
		expect(previewInvoiceNumber('{SEQ:03}', 2026, '2026', 12345)).toBe('12345');
	});
});

describe('validateFormatTemplate', () => {
	it('accepte F-{YEAR}-{SEQ:04}', () => {
		expect(validateFormatTemplate('F-{YEAR}-{SEQ:04}').ok).toBe(true);
	});

	it('rejette template vide', () => {
		expect(validateFormatTemplate('').ok).toBe(false);
		expect(validateFormatTemplate('   ').ok).toBe(false);
	});

	it('rejette absence de placeholder', () => {
		expect(validateFormatTemplate('FACT').ok).toBe(false);
	});

	it('rejette placeholder inconnu', () => {
		expect(validateFormatTemplate('{INVALID}').ok).toBe(false);
	});

	it('rejette caractère non autorisé', () => {
		expect(validateFormatTemplate('F-{YEAR}!{SEQ}').ok).toBe(false);
	});

	it('rejette {SEQ:0}', () => {
		expect(validateFormatTemplate('{SEQ:0}').ok).toBe(false);
	});

	it('rejette {SEQ:11}', () => {
		expect(validateFormatTemplate('{SEQ:11}').ok).toBe(false);
	});

	it(`accepte {SEQ:${MAX_SEQ_PADDING}} combiné avec texte court`, () => {
		expect(validateFormatTemplate('F-{YEAR}-{SEQ:10}').ok).toBe(true);
	});

	it('rejette template > 64 chars', () => {
		expect(validateFormatTemplate('a'.repeat(65)).ok).toBe(false);
	});
});

// Story 33-1b (#926) — l'option `numbering`, miroir de `validate_numbering_template`.
describe('validateFormatTemplate, option numbering', () => {
	for (const ok of ['FF-{YEAR}-{SEQ:03}', 'KF-{YEAR}-{SEQ:04}', '{YEAR}/{SEQ}', 'F1-{YEAR}-{SEQ}']) {
		it(`accepte ${ok}`, () => {
			expect(validateFormatTemplate(ok, { numbering: true }).ok).toBe(true);
		});
	}
	for (const ko of [
		'',
		'FF-{YEAR}',
		'FF-{SEQ:03}',
		'FF-{YEAR}-{SEQ:11}',
		'FF-{FY}-{SEQ}',
		'FF-{FY}-{YEAR}-{SEQ}',
		'{YEAR}-{YEAR}-{SEQ}',
		'{SEQ}-{YEAR}',
		'FF-{YEAR}{SEQ}',
		'FF-{YEAR}-1{SEQ}',
		'FF-{YEAR}-{SEQ}-X',
		' FF-{YEAR}-{SEQ}',
		'\tFF-{YEAR}-{SEQ}',
		'FF\t-{YEAR}-{SEQ}',
		'FF-{YEAR}\t{SEQ}',
	]) {
		it(`refuse ${JSON.stringify(ko)}`, () => {
			expect(validateFormatTemplate(ko, { numbering: true }).ok).toBe(false);
		});
	}
	// Story 35-1 (#932) — réécrit : la 33-1b figeait ici `F-{YEAR}` accepté pour les formats
	// client et avoir ; depuis la 35-1, un compteur y est obligatoire (D1).
	it('les formats client et avoir exigent un compteur, pas la forme du numéro interne', () => {
		const sansCompteur = validateFormatTemplate('F-{YEAR}');
		expect(sansCompteur.ok).toBe(false);
		expect(sansCompteur.code).toBe('NO_SEQ');
		expect(validateFormatTemplate('{FY}{SEQ}').ok).toBe(true);
	});
	it('le numéro interne garde son refus de forme, sans code NO_SEQ', () => {
		const r = validateFormatTemplate('FF-{YEAR}', { numbering: true });
		expect(r.ok).toBe(false);
		expect(r.code).toBeUndefined();
	});
});

// Story 35-1 (#932, AC5, D1) — le compteur obligatoire, miroir de `FormatError::NoSeq`.
describe('validateFormatTemplate, compteur obligatoire', () => {
	for (const ko of ['F-{YEAR}', '{YEAR}', 'AV-{FY}', 'F-{YEAR}-{FY}']) {
		it(`refuse ${ko} en NO_SEQ`, () => {
			const r = validateFormatTemplate(ko);
			expect(r.ok).toBe(false);
			expect(r.code).toBe('NO_SEQ');
			expect(r.error).toContain('compteur');
		});
	}
	for (const ok of ['F-{YEAR}-{SEQ:04}', 'FACT{SEQ}', '{FY}/{SEQ:06}']) {
		it(`accepte ${ok}`, () => {
			expect(validateFormatTemplate(ok)).toEqual({ ok: true });
		});
	}
	it('les autres refus priment et ne portent pas de code', () => {
		const sansPlaceholder = validateFormatTemplate('FACTURE');
		expect(sansPlaceholder.ok).toBe(false);
		expect(sansPlaceholder.code).toBeUndefined();
		const caractere = validateFormatTemplate('F-{YEAR}!');
		expect(caractere.ok).toBe(false);
		expect(caractere.code).toBeUndefined();
	});
});
