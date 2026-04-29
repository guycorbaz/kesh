/**
 * Helpers de notification harmonisés au-dessus de `svelte-sonner`.
 *
 * Story 3.5 : wrapper minimaliste qui standardise les durées et fournit
 * un point d'entrée unique pour les futures évolutions (changement de
 * lib, ajout d'actions par défaut, etc.).
 *
 * **Convention d'usage** : le nouveau code 3.5+ utilise ces helpers.
 * Les call sites existants (stories 3.2/3.3/3.4) continuent d'utiliser
 * `toast.*` directement — pas de refactor forcé. Un cleanup transverse
 * post-MVP peut migrer tous les sites en une fois si le pattern devient
 * gênant.
 */

import { goto } from '$app/navigation';
import { toast } from 'svelte-sonner';

import type { ApiError } from '$lib/shared/types/api';
import { i18nMsg } from '$lib/shared/utils/i18n.svelte';

/** Durée par défaut pour les notifications succès/info (ms). */
const DEFAULT_DURATION = 4000;

/** Durée pour les notifications warning/error (plus visibles). */
const ERROR_DURATION = 6000;

/**
 * Notification de succès (toast vert). Durée : 4 secondes.
 *
 * @param message - Titre principal du toast
 * @param description - Détails optionnels affichés sous le titre
 */
export function notifySuccess(message: string, description?: string): void {
	toast.success(message, { description, duration: DEFAULT_DURATION });
}

/**
 * Notification informative (toast neutre). Durée : 4 secondes.
 */
export function notifyInfo(message: string, description?: string): void {
	toast.info(message, { description, duration: DEFAULT_DURATION });
}

/**
 * Notification d'avertissement (toast orange). Durée : 6 secondes (plus visible).
 *
 * Usage prévu pour les opérations partielles : import bancaire avec
 * lignes ignorées, doublon détecté, etc. Pas de callsite en v0.1 (story
 * 3.5) — helper préparatoire pour les stories 6.x (import bancaire).
 */
export function notifyWarning(message: string, description?: string): void {
	toast.warning(message, { description, duration: ERROR_DURATION });
}

/**
 * Notification d'erreur (toast rouge). Durée : 6 secondes (plus visible).
 */
export function notifyError(message: string, description?: string): void {
	toast.error(message, { description, duration: ERROR_DURATION });
}

/**
 * Codes d'erreur backend signalant qu'aucun exercice comptable ouvert ne couvre
 * la date demandée (Story 3.7 AC #22). Voir backend `errors.rs:472-478`,
 * `errors.rs:326-338` et `journal_entries::create`.
 *
 * Code Review Pass 1 F5 — `FISCAL_YEAR_CLOSED` est sémantiquement distinct
 * (l'exercice **existe** mais est clos) ; on affiche un message dédié au
 * lieu du « Créez d'abord » trompeur.
 */
const FY_MISSING_CODES = ['FISCAL_YEAR_INVALID', 'NO_FISCAL_YEAR'] as const;
const FY_CLOSED_CODE = 'FISCAL_YEAR_CLOSED';

/**
 * Helper centralisé pour le toast actionnable lié aux erreurs d'exercice
 * comptable (Story 3.7 AC #22). Différencie deux cas :
 *
 * - `FISCAL_YEAR_INVALID` / `NO_FISCAL_YEAR` → « Créez d'abord un exercice » +
 *   bouton « Ouvrir Paramètres ».
 * - `FISCAL_YEAR_CLOSED` (Pass 1 F5) → « L'exercice qui couvre cette date est
 *   clôturé. Vérifiez la date ou consultez vos exercices. » + bouton
 *   « Ouvrir Paramètres » (en lecture seulement — on ne peut pas créer
 *   un exercice qui chevaucherait l'existant).
 *
 * Utilisation :
 * ```ts
 * try { await validateInvoice(id); }
 * catch (err) {
 *   if (isApiError(err) && notifyMissingFiscalYearOrFallback(err)) return;
 *   notifyError(err.message);
 * }
 * ```
 *
 * @returns `true` si le code d'erreur correspond et que le toast a été
 *   affiché (le caller peut alors skip son handler par défaut).
 *   `false` sinon — le caller continue son handler.
 */
export function notifyMissingFiscalYearOrFallback(err: ApiError): boolean {
	const isMissing = (FY_MISSING_CODES as readonly string[]).includes(err.code);
	const isClosed = err.code === FY_CLOSED_CODE;
	if (!isMissing && !isClosed) return false;

	const message = isClosed
		? i18nMsg(
				'error-fiscal-year-closed-for-date',
				"L'exercice qui couvre cette date est clôturé. Vérifiez la date saisie ou consultez vos exercices."
			)
		: i18nMsg(
				'error-fiscal-year-missing',
				"Créez d'abord un exercice comptable dans Paramètres → Exercices"
			);

	toast.error(message, {
		duration: ERROR_DURATION,
		action: {
			label: i18nMsg('go-to-settings', 'Ouvrir Paramètres'),
			onClick: () => {
				void goto('/settings/fiscal-years');
			}
		}
	});
	return true;
}
