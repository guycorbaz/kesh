// Story 26-1 (AC3-bis) — tests du mapping code d'erreur de ligne CSV → libellé.
import { describe, expect, it, vi } from 'vitest';

// i18nMsg rend la clé demandée : le test vérifie la clé, pas le repli.
vi.mock('$lib/shared/utils/i18n.svelte', () => ({
	i18nMsg: (key: string) => `[${key}]`,
}));

import { CSV_LINE_ERROR_LABELS, csvLineErrorLabel } from './csv-line-error-label';

describe('csvLineErrorLabel', () => {
	it('rend la clé de chaque code connu', () => {
		const attendu: Record<string, string> = {
			INVALID_DATE: 'bank-import-csv-errors-invalid-date',
			INVALID_AMOUNT: 'bank-import-csv-errors-invalid-amount',
			AMBIGUOUS_DEBIT_CREDIT: 'bank-import-csv-errors-ambiguous-debit-credit',
			EMPTY_MANDATORY_FIELD: 'bank-import-csv-errors-empty-mandatory-field',
			ROW_TOO_SHORT: 'bank-import-csv-errors-row-too-short',
			AMOUNT_SIGN_MISMATCH: 'bank-import-csv-errors-amount-sign-mismatch',
		};
		expect(Object.keys(CSV_LINE_ERROR_LABELS).sort()).toEqual(Object.keys(attendu).sort());
		for (const [code, cle] of Object.entries(attendu)) {
			expect(csvLineErrorLabel(code)).toBe(`[${cle}]`);
		}
	});

	it('rend le code brut pour un code inconnu', () => {
		expect(csvLineErrorLabel('SOMETHING_NEW')).toBe('SOMETHING_NEW');
	});
});
