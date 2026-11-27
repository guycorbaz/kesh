// #592 — l'IDENTITÉ de la clé i18n que chaque bras des motifs d'écran emploie.
//
// `blocker-messages.test.ts` mocke `i18nMsg` pour rendre le REPLI : il tient les
// phrases, jamais la clé (changer la clé d'un bras ne rougit rien : mutation F2 de
// la 15-1a-ii). Ici le mock rend la CLÉ, et une table `code → clé` écrite à la main
// (non extraite du source : les deux côtés d'un même extracteur seraient vrais par
// construction) est comparée au rendu ; puis chaque clé est cherchée dans les quatre
// catalogues (`^<clé> =`), lus à la source comme `i18n-keys.test.ts`.
//
// Le test ne contient aucun appel `i18nMsg` de production : le moissonneur de
// `i18n-keys.test.ts` (`sitesTotal`) ne le compte pas.

import { describe, it, expect, vi } from 'vitest';
import { readFileSync } from 'node:fs';
import { join } from 'node:path';

vi.mock('$lib/shared/utils/i18n.svelte', () => ({
	i18nMsg: (key: string) => key
}));

import { modificationBlockerLabel, reversalBlockerLabel } from './blocker-messages';
import type { ModificationBlocker, ReversalBlocker } from './journal-entries.types';

const LOCALES = ['fr-CH', 'de-CH', 'it-CH', 'en-CH'] as const;
const RACINE_FTL = '../crates/kesh-i18n/locales';

/** Les treize codes de modification et la clé de chacun (table littérale). */
const CLES_MODIFICATION: ReadonlyArray<[ModificationBlocker, string]> = [
	['FISCAL_YEAR_CLOSED', 'journal-entries-modify-blocked-fiscal-year-closed'],
	['LATER_FISCAL_YEAR_CLOSED', 'journal-entries-modify-blocked-later-fiscal-year-closed'],
	['PERIOD_LOCKED', 'journal-entries-modify-blocked-period-locked'],
	['DETACHED_SUPPLIER_SETTLEMENT', 'journal-entries-modify-blocked-detached-settlement'],
	['ENTRY_IS_FX_REVALUATION', 'journal-entries-modify-blocked-fx-revaluation'],
	['ENTRY_LETTERED', 'journal-entries-modify-blocked-lettered'],
	['IS_A_REVERSAL', 'journal-entries-reverse-blocked-is-a-reversal'],
	['ALREADY_REVERSED', 'journal-entries-reverse-blocked-already-reversed'],
	['OWNED_BY_INVOICE', 'journal-entries-reverse-blocked-invoice'],
	['OWNED_BY_CREDIT_NOTE', 'journal-entries-reverse-blocked-credit-note'],
	['OWNED_BY_SUPPLIER_INVOICE', 'journal-entries-reverse-blocked-supplier-invoice'],
	['OWNED_BY_SETTLEMENT', 'journal-entries-reverse-blocked-settlement'],
	['MATCHED_BANK_TRANSACTION', 'journal-entries-reverse-blocked-bank-match']
];

/** Les huit codes de contre-passation et la clé de chacun (table littérale). */
const CLES_CONTREPASSATION: ReadonlyArray<[ReversalBlocker, string]> = [
	['IS_A_REVERSAL', 'journal-entries-reverse-blocked-is-a-reversal'],
	['ALREADY_REVERSED', 'journal-entries-reverse-blocked-already-reversed'],
	['OWNED_BY_INVOICE', 'journal-entries-reverse-blocked-invoice'],
	['OWNED_BY_CREDIT_NOTE', 'journal-entries-reverse-blocked-credit-note'],
	['OWNED_BY_SUPPLIER_INVOICE', 'journal-entries-reverse-blocked-supplier-invoice'],
	['OWNED_BY_SETTLEMENT', 'journal-entries-reverse-blocked-settlement'],
	['MATCHED_BANK_TRANSACTION', 'journal-entries-reverse-blocked-bank-match'],
	['ACCOUNT_ARCHIVED', 'journal-entries-reverse-blocked-account-archived']
];

describe('modificationBlockerLabel — la clé de chaque bras', () => {
	it.each(CLES_MODIFICATION)('%s → %s', (code, cle) => {
		expect(modificationBlockerLabel(code, 'X')).toBe(cle);
	});

	it('la table couvre 13 codes', () => {
		expect(CLES_MODIFICATION).toHaveLength(13);
	});
});

describe('reversalBlockerLabel — la clé de chaque bras', () => {
	it.each(CLES_CONTREPASSATION)('%s → %s', (code, cle) => {
		expect(reversalBlockerLabel(code)).toBe(cle);
	});

	it('la table couvre 8 codes', () => {
		expect(CLES_CONTREPASSATION).toHaveLength(8);
	});
});

describe('chaque clé existe aux quatre catalogues', () => {
	const cles = [
		...new Set([...CLES_MODIFICATION, ...CLES_CONTREPASSATION].map(([, cle]) => cle))
	];

	it('14 clés distinctes (6 propres à la modification, 8 de la contre-passation)', () => {
		expect(cles).toHaveLength(14);
	});

	it.each([...LOCALES])('%s', (locale) => {
		const texte = readFileSync(join(RACINE_FTL, locale, 'messages.ftl'), 'utf-8');
		for (const cle of cles) {
			expect(texte, `${locale} : ${cle}`).toMatch(new RegExp(`^${cle} =`, 'm'));
		}
	});
});
