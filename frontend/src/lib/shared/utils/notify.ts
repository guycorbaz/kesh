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

import { toast } from 'svelte-sonner';

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
