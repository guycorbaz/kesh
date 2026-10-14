/**
 * Story 29-1b (#232) — l'état du geste « Contre-passer au début de l'exercice
 * suivant » sur la fiche d'une écriture, déduit de la liste des exercices.
 *
 * ⛔ **L'écran annonce, le serveur décide** : cette lecture ne sert qu'à montrer
 * le bouton, ou le motif qui dit pourquoi il manque (règle de la page : un
 * bouton absent s'explique). Le serveur confirme l'exercice sous verrou et
 * refuse en `NEXT_FISCAL_YEAR_MISSING` / `NEXT_FISCAL_YEAR_CLOSED`.
 *
 * Fonction pure, sans i18n : la page traduit le motif.
 */

import type { FiscalYearResponse } from '$lib/features/fiscal-years/fiscal-years.types';

/** La liste des exercices telle que la page la tient : `null` en cours de lecture, `'error'` en échec. */
export type FiscalYearsState = FiscalYearResponse[] | null | 'error';

/** Ce que la fiche affiche pour le geste. */
export type NextYearReversal =
	/** Liste en cours de lecture : ni bouton ni motif (rien de trompeur). */
	| { kind: 'loading' }
	/** Liste illisible (échec, ou l'exercice de l'écriture n'y figure pas). */
	| { kind: 'unavailable' }
	/** Aucun exercice ne suit celui de l'écriture. */
	| { kind: 'missing' }
	/** L'exercice qui suit est clôturé. */
	| { kind: 'closed'; year: FiscalYearResponse }
	/** L'exercice qui suit est ouvert : le bouton est offert. */
	| { kind: 'open'; year: FiscalYearResponse };

/**
 * L'exercice qui suit celui de l'écriture — le premier dont la date de début
 * est **postérieure** à la fin de l'exercice `fiscalYearId`, comme
 * `fiscal_years::find_next_after` côté serveur — et son état.
 *
 * Les dates sont des chaînes ISO `YYYY-MM-DD` : leur ordre lexical est l'ordre
 * chronologique.
 */
export function nextYearReversal(years: FiscalYearsState, fiscalYearId: number): NextYearReversal {
	if (years === null) return { kind: 'loading' };
	if (years === 'error') return { kind: 'unavailable' };
	const own = years.find((y) => y.id === fiscalYearId);
	if (!own) return { kind: 'unavailable' };
	let next: FiscalYearResponse | undefined;
	for (const y of years) {
		if (y.startDate > own.endDate && (!next || y.startDate < next.startDate)) next = y;
	}
	if (!next) return { kind: 'missing' };
	return next.status === 'Open' ? { kind: 'open', year: next } : { kind: 'closed', year: next };
}
