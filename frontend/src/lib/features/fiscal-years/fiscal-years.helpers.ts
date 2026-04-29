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
