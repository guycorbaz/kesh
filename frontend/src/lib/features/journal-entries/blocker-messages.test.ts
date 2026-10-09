// Story 15-8a (#532, D8) — les motifs traduits de la fiche d'une écriture.
//
// ⛔ Un test PAR CODE : le `switch` est exhaustif au type-check (affectation à
// `never`), mais seule l'exécution prouve que chaque code rend SON message, et
// non une chaîne vide ou celui d'un voisin. Le mock i18n rend le repli, ce qui
// permet d'asserter la phrase française.

import { describe, it, expect, vi } from 'vitest';

vi.mock('$lib/shared/utils/i18n.svelte', () => ({
	i18nMsg: (_key: string, fallback: string, args?: Record<string, string | number>) =>
		args
			? fallback.replace(/\{\s*\$(\w+)\s*\}/g, (_: string, k: string) => String(args[k] ?? ''))
			: fallback
}));

import {
	MODIFICATION_LABEL_IN_MESSAGE,
	modificationBlockerLabel,
	reversalBlockerLabel
} from './blocker-messages';
import type { ModificationBlocker } from './journal-entries.types';

/** Les douze codes d'écran, et un fragment propre à chacun. */
const ATTENDU: ReadonlyArray<[ModificationBlocker, string]> = [
	['FISCAL_YEAR_CLOSED', 'exercice de cette écriture est clôturé'],
	['LATER_FISCAL_YEAR_CLOSED', 'exercice postérieur Exercice 2027 est clôturé'],
	['IS_A_REVERSAL', 'elle-même une contre-passation'],
	['ALREADY_REVERSED', 'déjà été contre-passée'],
	['OWNED_BY_INVOICE', 'facture client'],
	['OWNED_BY_CREDIT_NOTE', "celle d'un avoir"],
	['OWNED_BY_SUPPLIER_INVOICE', 'facture fournisseur : elle se corrige'],
	['OWNED_BY_SETTLEMENT', 'règlement de facture'],
	['MATCHED_BANK_TRANSACTION', 'transaction bancaire'],
	['DETACHED_SUPPLIER_SETTLEMENT', 'facture fournisseur annulée'],
	['PERIOD_LOCKED', 'verrouillée jusqu’au 2026-03-31'],
	['ENTRY_LETTERED', 'est lettrée : délettrez-la d’abord']
];

describe('modificationBlockerLabel — un message par code', () => {
	it.each(ATTENDU)('%s', (code, fragment) => {
		const label =
			code === 'LATER_FISCAL_YEAR_CLOSED'
				? 'Exercice 2027'
				: code === 'PERIOD_LOCKED'
					? '2026-03-31'
					: null;
		expect(modificationBlockerLabel(code, label)).toContain(fragment);
	});

	it('couvre exactement douze codes, tous distincts', () => {
		const messages = ATTENDU.map(([c]) => modificationBlockerLabel(c, 'X'));
		expect(ATTENDU).toHaveLength(12);
		expect(new Set(messages).size).toBe(12);
	});

	it('les codes communs rendent le message de la contre-passation (une seule source)', () => {
		for (const code of [
			'IS_A_REVERSAL',
			'ALREADY_REVERSED',
			'OWNED_BY_INVOICE',
			'OWNED_BY_CREDIT_NOTE',
			'OWNED_BY_SUPPLIER_INVOICE',
			'OWNED_BY_SETTLEMENT',
			'MATCHED_BANK_TRANSACTION'
		] as const) {
			expect(modificationBlockerLabel(code, null)).toBe(reversalBlockerLabel(code));
		}
	});

	it('un code inconnu (bundle périmé) rend une chaîne vide, jamais le code en clair', () => {
		expect(modificationBlockerLabel('NEUVIEME_CODE' as ModificationBlocker, null)).toBe('');
	});

	it("seuls l'exercice postérieur et la borne portent l'étiquette dans le message", () => {
		expect([...MODIFICATION_LABEL_IN_MESSAGE].sort()).toEqual([
			'LATER_FISCAL_YEAR_CLOSED',
			'PERIOD_LOCKED'
		]);
	});
});
