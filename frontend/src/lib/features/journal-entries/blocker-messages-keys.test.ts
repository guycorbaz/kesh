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

/**
 * Les treize codes de modification et la clé de chacun (table littérale).
 *
 * ⚠️ `Record<ModificationBlocker, string>` : le type est une union de chaînes, donc
 * sans existence à l'exécution — la seule liaison possible sans toucher la
 * production est celle du **type-check**. Un code ajouté à l'union sans ligne ici
 * (ou une ligne d'un code retiré) fait rougir `npm run check`, non vitest : angle
 * mort assumé, vitest seul ne voit pas un code neuf.
 */
const CLES_MODIFICATION_PAR_CODE: Record<ModificationBlocker, string> = {
	FISCAL_YEAR_CLOSED: 'journal-entries-modify-blocked-fiscal-year-closed',
	LATER_FISCAL_YEAR_CLOSED: 'journal-entries-modify-blocked-later-fiscal-year-closed',
	PERIOD_LOCKED: 'journal-entries-modify-blocked-period-locked',
	DETACHED_SUPPLIER_SETTLEMENT: 'journal-entries-modify-blocked-detached-settlement',
	ENTRY_IS_FX_REVALUATION: 'journal-entries-modify-blocked-fx-revaluation',
	ENTRY_LETTERED: 'journal-entries-modify-blocked-lettered',
	IS_A_REVERSAL: 'journal-entries-reverse-blocked-is-a-reversal',
	ALREADY_REVERSED: 'journal-entries-reverse-blocked-already-reversed',
	OWNED_BY_INVOICE: 'journal-entries-reverse-blocked-invoice',
	OWNED_BY_CREDIT_NOTE: 'journal-entries-reverse-blocked-credit-note',
	OWNED_BY_SUPPLIER_INVOICE: 'journal-entries-reverse-blocked-supplier-invoice',
	OWNED_BY_SETTLEMENT: 'journal-entries-reverse-blocked-settlement',
	MATCHED_BANK_TRANSACTION: 'journal-entries-reverse-blocked-bank-match'
};

/** Les huit codes de contre-passation et la clé de chacun (même liaison par le type). */
const CLES_CONTREPASSATION_PAR_CODE: Record<ReversalBlocker, string> = {
	IS_A_REVERSAL: 'journal-entries-reverse-blocked-is-a-reversal',
	ALREADY_REVERSED: 'journal-entries-reverse-blocked-already-reversed',
	OWNED_BY_INVOICE: 'journal-entries-reverse-blocked-invoice',
	OWNED_BY_CREDIT_NOTE: 'journal-entries-reverse-blocked-credit-note',
	OWNED_BY_SUPPLIER_INVOICE: 'journal-entries-reverse-blocked-supplier-invoice',
	OWNED_BY_SETTLEMENT: 'journal-entries-reverse-blocked-settlement',
	MATCHED_BANK_TRANSACTION: 'journal-entries-reverse-blocked-bank-match',
	ACCOUNT_ARCHIVED: 'journal-entries-reverse-blocked-account-archived'
};

const CLES_MODIFICATION = Object.entries(CLES_MODIFICATION_PAR_CODE) as Array<
	[ModificationBlocker, string]
>;
const CLES_CONTREPASSATION = Object.entries(CLES_CONTREPASSATION_PAR_CODE) as Array<
	[ReversalBlocker, string]
>;

describe('modificationBlockerLabel — la clé de chaque bras', () => {
	it.each(CLES_MODIFICATION)('%s → %s', (code, cle) => {
		expect(modificationBlockerLabel(code, 'X')).toBe(cle);
	});
});

describe('reversalBlockerLabel — la clé de chaque bras', () => {
	it.each(CLES_CONTREPASSATION)('%s → %s', (code, cle) => {
		expect(reversalBlockerLabel(code)).toBe(cle);
	});
});

describe('chaque clé existe aux quatre catalogues', () => {
	const cles = [
		...new Set([...CLES_MODIFICATION, ...CLES_CONTREPASSATION].map(([, cle]) => cle))
	];

	it.each([...LOCALES])('%s', (locale) => {
		const texte = readFileSync(join(RACINE_FTL, locale, 'messages.ftl'), 'utf-8');
		for (const cle of cles) {
			expect(texte, `${locale} : ${cle}`).toMatch(new RegExp(`^${cle} =`, 'm'));
		}
	});
});
