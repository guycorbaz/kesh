/**
 * Story 37-3b (AC-3) — l'affichage d'une ligne en devise dans la fiche et la
 * liste des écritures : montant en devise avec son code (celui de la réponse,
 * la devise du compte, jamais une valeur gardée par l'écran), cours sans zéros
 * terminaux, provenance (`AFC`, `manuel`, `réévaluation`).
 */

import Big from 'big.js';
import { i18nMsg } from '$lib/shared/utils/i18n.svelte';
import { formatSwissAmount } from './balance';
import type { ForeignLineResponse, JournalEntryResponse } from './journal-entries.types';

/** La provenance d'un cours, traduite. */
export function rateSourceLabel(source: ForeignLineResponse['rateSource']): string {
	switch (source) {
		case 'afc':
			return i18nMsg('journal-entries-fx-source-afc', 'AFC');
		case 'revaluation':
			return i18nMsg('journal-entries-fx-source-revaluation', 'réévaluation');
		default:
			return i18nMsg('journal-entries-fx-source-manual', 'manuel');
	}
}

/** Le montant en devise : deux décimales, quatre s'il en porte plus. */
export function formatForeignAmount(amount: string): string {
	try {
		const b = new Big(amount);
		const decimals = Math.max(0, b.c.length - b.e - 1) > 2 ? 4 : 2;
		return formatSwissAmount(b, decimals);
	} catch {
		return amount;
	}
}

/** Le cours sans zéros terminaux (`0.8846`, jamais `0.8846000000`). */
export function formatRate(rate: string): string {
	try {
		return new Big(rate).toFixed();
	} catch {
		return rate;
	}
}

/** « USD 1’078.40 × 0.8846 · AFC » — une ligne en devise d'une réponse. */
export function foreignLineLabel(f: ForeignLineResponse): string {
	return `${f.currency} ${formatForeignAmount(f.amount)} × ${formatRate(f.rate)} · ${rateSourceLabel(f.rateSource)}`;
}

/** Les mentions en devise d'une écriture (liste) : une par ligne en devise. */
export function entryForeignLabels(entry: JournalEntryResponse): string[] {
	return entry.lines.flatMap((l) => (l.foreign ? [foreignLineLabel(l.foreign)] : []));
}
