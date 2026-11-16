/**
 * Story 37-3b (AC-1) — les lignes en devise des deux grilles des soldes
 * d'ouverture (génération et complément), en fonctions pures.
 *
 * Contrat des lignes : celui du formulaire d'écriture — le montant en devise et
 * le sens vivent dans `foreign`, `debit`/`credit` ne portent que le CHF dérivé
 * (`round(montant × cours, 2)`, {@link syncRow}). Une ligne en devise dont le
 * montant est saisi sans CHF calculable est **incomplète** : jamais retirée en
 * silence du corps, elle désactive l'envoi.
 *
 * Cours proposé (R6, T4, U1) : le cours retenu par la société à la date du
 * cours d'ouverture — veille de l'ouverture (genre `day`) ou dernier mois de
 * l'exercice précédent (genre `month`), {@link openingRateDate} ; pour un
 * complément daté du jour (`TODAY`), le cours mensuel du mois de cette date.
 * Envoyé `rateSource: "manual"`, proposé ou saisi.
 */

import Big from 'big.js';
import { i18nMsg } from '$lib/shared/utils/i18n.svelte';
import { formatSwissDate } from '$lib/features/reports/reports.api';
import { openingRateDate, readForeignAmount, readRate, type RateKind } from '$lib/features/journal-entries/fx';
import {
	applyChfEntry,
	foreignChf,
	foreignLineProblem,
	type LineForeign,
	type LookupOutcome,
} from '$lib/features/journal-entries/fx-lines';
import type { ForeignLineRequest } from '$lib/features/journal-entries/journal-entries.types';

/** Une ligne de grille (génération ou complément). */
export interface FxGridRow {
	debit: string;
	credit: string;
	foreign?: LineForeign;
}

/** Pose sur la ligne le CHF dérivé de sa partie en devise (sans effet sur une ligne CHF). */
export function syncRow<T extends FxGridRow>(row: T): T {
	const f = row.foreign;
	if (!f) return row;
	const r = foreignChf(f);
	const chf = r !== null && r.ok ? r.value.toFixed(2) : '';
	row.debit = f.side === 'debit' ? chf : '';
	row.credit = f.side === 'credit' ? chf : '';
	return row;
}

/** Une ligne en devise saisie (montant ou CHF) mais sans CHF calculable : incomplète. */
export function isIncompleteForeignRow(row: FxGridRow): boolean {
	const f = row.foreign;
	if (!f) return false;
	const touched = f.amount.trim() !== '' || f.chfEntry !== null;
	if (!touched) return false;
	const r = foreignChf(f);
	return r === null || !r.ok || f.chfError !== null;
}

/** Une ligne en devise à envoyer (montant saisi). */
export function isFilledForeignRow(row: FxGridRow): boolean {
	return row.foreign !== undefined && row.foreign.amount.trim() !== '';
}

/**
 * La partie `foreign` envoyée par une grille : devise du compte, montant et
 * cours (`toFixed`), `manual` toujours ; `null` si la ligne n'est pas complète.
 */
export function rowForeignPayload(row: FxGridRow): ForeignLineRequest | null {
	const f = row.foreign;
	if (!f) return null;
	const amount = readForeignAmount(f.amount);
	const rate = readRate(f.rate);
	if (amount === null || amount === 'empty' || rate === null || rate === 'empty') return null;
	return { currency: f.currency, amount: amount.toFixed(), rate: rate.toFixed(), rateSource: 'manual' };
}

/** Le montant en devise saisi dans la colonne `side`. */
export function setRowAmount<T extends FxGridRow>(row: T, side: 'debit' | 'credit', value: string): T {
	if (!row.foreign) return row;
	row.foreign = { ...row.foreign, side, amount: value, chfEntry: null, chfError: null };
	return syncRow(row);
}

/** Le cours saisi. */
export function setRowRate<T extends FxGridRow>(row: T, value: string): T {
	if (!row.foreign) return row;
	row.foreign = { ...row.foreign, rate: value, rateProposed: false, touched: true, chfEntry: null, chfError: null };
	return syncRow(row);
}

/** Le CHF saisi (cours OU CHF) : le cours s'en déduit (`rateFromChf`). */
export function setRowChf<T extends FxGridRow>(row: T, value: string): T {
	if (!row.foreign) return row;
	// Les grilles envoient toujours `manual` : aucun cours `afc` n'est repris (A32 ne s'applique pas).
	row.foreign = applyChfEntry({ ...row.foreign, afcRate: null, afcMonth: null }, value, '');
	return syncRow(row);
}

/** La valeur affichée dans le champ CHF d'une ligne en devise. */
export function rowChfValue(row: FxGridRow): string {
	const f = row.foreign;
	if (!f) return '';
	if (f.chfEntry !== null) return f.chfEntry;
	return f.side === 'debit' ? row.debit : row.credit;
}

/** Le problème d'une ligne en devise à afficher (`foreignLineProblem`). */
export const rowProblem = (row: FxGridRow) => (row.foreign ? foreignLineProblem(row.foreign) : null);

/** Ce qu'un `lookup` a proposé pour la grille : sa date et son genre. */
export interface OpeningProposal {
	kind: RateKind;
	/** Jour (genre `day`) ou 1er du mois (genre `month`). */
	date: string;
}

/**
 * La date et le genre du cours proposé d'une grille : la génération et le
 * complément daté du premier jour prennent le cours d'ouverture (genre de
 * l'exercice) ; le complément daté du jour, le cours mensuel du mois de cette
 * date. `null` si l'exercice ou son genre sont inconnus (aucune proposition).
 */
export function openingProposal(
	fiscalYear: { startDate: string; openingRateKind: RateKind } | null,
	complement?: { dateKind: 'OPENING_DAY' | 'TODAY' | null; date: string | null },
): OpeningProposal | null {
	if (complement && complement.dateKind === 'TODAY') {
		return complement.date ? { kind: 'month', date: `${complement.date.slice(0, 7)}-01` } : null;
	}
	if (!fiscalYear) return null;
	return {
		kind: fiscalYear.openingRateKind,
		date: openingRateDate(fiscalYear.startDate, fiscalYear.openingRateKind),
	};
}

/**
 * Applique un `lookup` à une ligne de grille : le cours de la société est
 * proposé tant que la ligne n'a pas été retouchée ; `company` retient la
 * provenance affichée.
 */
export function applyOpeningLookup<T extends FxGridRow>(row: T, proposal: OpeningProposal, outcome: LookupOutcome): T {
	const f = row.foreign;
	if (!f) return row;
	if (outcome.kind === 'none') {
		row.foreign = { ...f, company: null, rate: f.rateProposed ? '' : f.rate };
	} else {
		row.foreign = {
			...f,
			company: { ...outcome.rate, month: proposal.date },
			rate: f.rateProposed ? new Big(outcome.rate.rate).toFixed() : f.rate,
		};
	}
	return syncRow(row);
}

/**
 * La provenance affichée du cours d'une ligne (A30, U1, lue sur `source`) :
 * « proposé : cours journalier du JJ.MM.AAAA », « proposé : AFC MM.AAAA »,
 * « cours saisi » (`manual`), « cours saisi (ecb) » pour toute autre source ;
 * « manuel » quand la ligne ne porte plus le cours proposé.
 */
export function openingProvenanceLabel(f: LineForeign, proposal: OpeningProposal | null): string {
	const rate = readRate(f.rate);
	const company = f.company;
	if (company === null || proposal === null || rate === null || rate === 'empty' || !rate.eq(new Big(company.rate))) {
		return i18nMsg('opening-balances-fx-provenance-manual', 'manuel');
	}
	if (company.source === 'afc') {
		return proposal.kind === 'day'
			? i18nMsg('opening-balances-fx-provenance-day', 'proposé : cours journalier du { $date }', {
					date: formatSwissDate(proposal.date),
				})
			: i18nMsg('opening-balances-fx-provenance-month', 'proposé : AFC { $month }', {
					month: `${proposal.date.slice(5, 7)}.${proposal.date.slice(0, 4)}`,
				});
	}
	if (company.source === 'manual') return i18nMsg('opening-balances-fx-provenance-saisi', 'cours saisi');
	return i18nMsg('opening-balances-fx-provenance-saisi-source', 'cours saisi ({ $source })', {
		source: company.source,
	});
}
