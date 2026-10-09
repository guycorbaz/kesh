/**
 * Helpers pour les exercices comptables (Story 3.7).
 */

import type { CreateFiscalYearRequest, FiscalYearResponse } from './fiscal-years.types';

/**
 * Longueur maximale du nom d'un exercice (cohérent avec la colonne DB
 * `fiscal_years.name VARCHAR(50)`). Story 3.7 Code Review Pass 1 F3.
 */
export const FISCAL_YEAR_NAME_MAX_LENGTH = 50;

/**
 * Valide le formulaire de création d'un exercice côté client.
 *
 * Retourne `null` si OK, sinon une clé i18n que le caller passera à `t()`
 * (voir Pass 2 HP2-M7).
 *
 * Story 3.7 Code Review Pass 1 F3 — pré-validation de la longueur du nom
 * pour éviter qu'un payload trop long ne déclenche un 500 backend.
 */
export function validateFiscalYearForm(input: CreateFiscalYearRequest): string | null {
	const trimmed = input.name.trim();
	if (!trimmed) return 'error-fiscal-year-name-empty';
	if ([...trimmed].length > FISCAL_YEAR_NAME_MAX_LENGTH) return 'error-fiscal-year-name-too-long';
	const start = new Date(input.startDate);
	const end = new Date(input.endDate);
	if (isNaN(start.getTime()) || isNaN(end.getTime())) {
		return 'error-fiscal-year-dates-invalid';
	}
	if (end <= start) return 'error-fiscal-year-dates-invalid';
	return null;
}

/**
 * Format d'affichage d'un exercice (ex: `"Exercice 2027 (Open)"`).
 */
export function formatFiscalYearLabel(fy: FiscalYearResponse): string {
	return `${fy.name} (${fy.status})`;
}

/**
 * Pré-remplit le formulaire avec des valeurs par défaut basées sur l'année
 * calendaire courante : `Exercice {YYYY}` du 1er janvier au 31 décembre.
 */
export function currentYearDefaults(): CreateFiscalYearRequest {
	const year = new Date().getFullYear();
	return {
		name: `Exercice ${year}`,
		startDate: `${year}-01-01`,
		endDate: `${year}-12-31`
	};
}

/**
 * Les trois exercices que nomme le bandeau d'état hérité (Story 15-12b, #543, AC 15).
 */
export interface OutOfOrderState {
	/** Le **plus ancien** exercice ouvert — toujours concerné dès que l'état est fautif. */
	open: FiscalYearResponse;
	/** Son plus proche exercice postérieur clos. */
	closed: FiscalYearResponse;
	/** Le plus récent exercice clos — celui par lequel une réouverture commence (LIFO). */
	latest: FiscalYearResponse;
}

/**
 * Détecte, depuis la liste déjà chargée, l'état hérité « exercice ouvert suivi d'un
 * exercice clos » (données d'une version antérieure, sauvegarde restaurée). Depuis la
 * Story 15-12a, l'application ne peut plus le produire ; le serveur refuse alors toute
 * écriture dans l'exercice ouvert (`LATER_FISCAL_YEAR_CLOSED`), et l'écran doit dire
 * comment rétablir l'ordre. `null` si l'état est sain.
 *
 * Pourquoi `open` est le PLUS ANCIEN exercice ouvert : si un exercice ouvert X précède un
 * exercice clos C, le plus ancien ouvert O vérifie O ≤ X < C — il est concerné lui aussi.
 * C'est lui que l'on clôture d'abord : la clôture le refuse tant qu'un antérieur est ouvert.
 *
 * Comparaison des dates par chaîne `YYYY-MM-DD` (ordre lexicographique = chronologique),
 * comme `nearestLaterClosed` de la page.
 */
export function outOfOrderState(fiscalYears: FiscalYearResponse[]): OutOfOrderState | null {
	let open: FiscalYearResponse | null = null;
	let latest: FiscalYearResponse | null = null;
	for (const fy of fiscalYears) {
		if (fy.status === 'Open' && (open === null || fy.startDate < open.startDate)) open = fy;
		if (fy.status === 'Closed' && (latest === null || fy.startDate > latest.startDate)) latest = fy;
	}
	if (open === null || latest === null || latest.startDate <= open.startDate) return null;
	let closed: FiscalYearResponse | null = null;
	for (const fy of fiscalYears) {
		if (
			fy.status === 'Closed' &&
			fy.startDate > open.startDate &&
			(closed === null || fy.startDate < closed.startDate)
		) {
			closed = fy;
		}
	}
	// `latest` est postérieur à `open` : `closed` existe toujours ici.
	return closed === null ? null : { open, closed, latest };
}
