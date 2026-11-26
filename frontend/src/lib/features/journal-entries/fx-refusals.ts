/**
 * Story 37-3b (AC-1, AC-2, AC-2 bis) — le message d'écran des refus du serveur
 * propres aux devises, partagé par le formulaire d'écriture, les deux grilles
 * des soldes d'ouverture et l'assistant *Conversion*.
 *
 * Les refus qui demandent une conduite propre à l'écran ont leur message
 * (quatre locales) : `FX_CURRENCY_MISMATCH` et `FX_ACCOUNT_IN_CHF` (le compte a
 * changé depuis le chargement : recharger la page), `MULTI_CURRENCY_DISABLED`
 * (exercice mono-devise), `FX_RATE_NOT_AFC` (avec ou sans cours attendu),
 * `RATE_UNAVAILABLE`. Les autres rendent le message du serveur, déjà traduit.
 * Le message est affiché **sur la ligne fautive** quand le refus porte
 * `details.line` ; la ligne n'est pas répétée dans le message.
 */

import { i18nMsg } from '$lib/shared/utils/i18n.svelte';
import { isFxRefusalCode } from './form-helpers';

/** Les `details` d'un refus (objet JSON quelconque). */
export type RefusalDetails = Record<string, unknown> | undefined;

/** Lit une chaîne de `details`, `''` sinon. */
function str(details: RefusalDetails, key: string): string {
	const v = details?.[key];
	return typeof v === 'string' ? v : v === undefined || v === null ? '' : String(v);
}

/**
 * Le mois `AAAA-MM-01` en `MM.AAAA` (une valeur d'une autre forme rendue
 * telle quelle). Exportée sous ce nom pour les refus des prestataires
 * (Story 39-11d2, `provider-labels.ts`) : `exchange-rates-grid.ts` exporte
 * déjà un `monthLabel` d'une autre signature.
 */
export function fxMonthLabel(iso: string): string {
	return /^\d{4}-\d{2}/.test(iso) ? `${iso.slice(5, 7)}.${iso.slice(0, 4)}` : iso;
}

/** Vrai si le code est un refus propre aux devises (voir `isFxRefusalCode`). */
export { isFxRefusalCode };

/**
 * Le message d'écran d'un refus propre aux devises, `serverMessage` pour les
 * codes sans message propre.
 */
export function fxRefusalMessage(code: string, details: RefusalDetails, serverMessage: string): string {
	switch (code) {
		case 'FX_CURRENCY_MISMATCH':
			return i18nMsg(
				'error-fx-screen-currency-mismatch',
				'Le compte de cette ligne est tenu en { $expected } : sa devise a changé depuis le chargement. Rechargez la page.',
				{ expected: str(details, 'expected') }
			);
		case 'FX_ACCOUNT_IN_CHF':
			return i18nMsg(
				'error-fx-screen-account-in-chf',
				'Le compte de cette ligne est tenu en CHF : il ne porte plus de devise. Rechargez la page.'
			);
		case 'MULTI_CURRENCY_DISABLED':
			return i18nMsg(
				'error-fx-screen-multi-currency-disabled',
				'Cet exercice est mono-devise (il a été clos avant l’activation du multi-devises) : aucun compte en devise n’y est utilisable.'
			);
		case 'FX_RATE_NOT_AFC': {
			const expected = details?.expectedRate;
			if (expected === null || expected === undefined || expected === '') {
				return i18nMsg(
					'error-fx-screen-rate-not-afc-manual',
					'La société a fixé son propre cours pour ce mois : il n’y a plus de cours AFC à reprendre. Gardez votre cours comme cours saisi.'
				);
			}
			return i18nMsg(
				'error-fx-screen-rate-not-afc',
				'Ce n’est pas le cours AFC retenu pour ce mois ({ $rate }) : reprenez ce cours, ou gardez le vôtre comme cours saisi.',
				{ rate: String(expected) }
			);
		}
		case 'RATE_UNAVAILABLE':
			return i18nMsg(
				'error-fx-screen-rate-unavailable',
				'Aucun cours { $currency } retenu pour { $month } : récupérez le mois, ou gardez la ligne comme cours saisi.',
				{ currency: str(details, 'currency'), month: fxMonthLabel(str(details, 'month')) }
			);
		default:
			return serverMessage;
	}
}

/** Le rang `details.line` d'un refus (1-based dans le corps envoyé), ou `null`. */
export function refusalLine(details: RefusalDetails): number | null {
	const v = details?.line;
	return typeof v === 'number' && Number.isInteger(v) && v >= 1 ? v : null;
}

/** Le rang `details.entry` d'un refus de la conversion (1 ou 2), ou `null`. */
export function refusalEntry(details: RefusalDetails): 1 | 2 | null {
	const v = details?.entry;
	return v === 1 || v === 2 ? v : null;
}

/**
 * Le message traduit d'un problème de ligne en devise (`foreignLineProblem`),
 * partagé par le formulaire d'écriture et l'assistant *Conversion*.
 */
export function foreignProblemMessage(problem: string): string {
	switch (problem) {
		case 'FX_VALUE_OUT_OF_RANGE':
			return i18nMsg(
				'journal-entries-fx-value-out-of-range',
				'Montant en devise, cours ou montant CHF hors des limites : montant positif à quatre décimales au plus, cours positif à dix décimales et 1 000 000 au plus.'
			);
		case 'FX_LINE_ROUNDS_TO_ZERO':
			return i18nMsg(
				'journal-entries-fx-rounds-to-zero',
				'Le montant en CHF calculé s’arrondit à zéro.'
			);
		case 'FX_AMOUNT_MISMATCH':
			return i18nMsg(
				'journal-entries-fx-chf-no-rate',
				'Aucun cours à dix décimales ne redonne ce montant CHF : corrigez le CHF ou saisissez le cours.'
			);
		case 'amount-unreadable':
			return i18nMsg(
				'journal-entries-fx-amount-unreadable',
				'Montant en devise illisible ou ambigu : chiffres, apostrophe des milliers (1\'234), point ou virgule décimale, quatre décimales au plus.'
			);
		case 'rate-unreadable':
			return i18nMsg(
				'journal-entries-fx-rate-invalid',
				'Cours invalide : strictement positif, dix décimales et 1 000 000 au plus.'
			);
		default:
			return i18nMsg(
				'journal-entries-fx-incomplete',
				'Saisissez le montant en devise et le cours (ou le montant CHF).'
			);
	}
}
