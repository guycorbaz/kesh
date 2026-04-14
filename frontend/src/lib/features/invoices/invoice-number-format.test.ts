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
