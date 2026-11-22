/**
 * Story 38-12b (AC-2, AC-3) — règles **génériques** du dialogue de
 * téléchargement d'un prestataire, tenues hors du composant pour être testées
 * seules : période par défaut, refus du lancement (dont la reprise sur `409`),
 * messages d'échec d'une tâche, texte de progression et de pause, politique de
 * suivi. Aucun nom de prestataire : l'appelant passe ses libellés.
 */
import { isApiError } from '$lib/shared/utils/api-error';
import { errorMessageOf } from '$lib/shared/utils/error-message';
import { i18nMsg } from '$lib/shared/utils/i18n.svelte';
import type { ProviderDownloadError } from './provider-download.api';

/** Intervalle d'interrogation de la tâche (AC-3). */
export const POLL_INTERVAL_MS = 1000;

/**
 * Erreurs réseau **consécutives** au-delà desquelles le suivi est déclaré
 * perdu (AC-3) ; une réponse lue remet le compte à zéro.
 */
export const MAX_POLL_ERRORS = 3;

/** Une période `AAAA-MM-JJ`, bornes comprises. */
export interface Period {
	from: string;
	to: string;
}

/**
 * Période proposée (AC-2) : l'exercice choisi, sa fin ramenée à aujourd'hui (le
 * jour de Zurich, que juge le serveur) ; sans exercice, le jour même. Un exercice
 * qui commence après aujourd'hui propose le jour même (le serveur refuserait un
 * début futur).
 */
export function defaultPeriod(fiscalYear: { startDate: string; endDate: string } | null, today: string): Period {
	if (!fiscalYear) return { from: today, to: today };
	const to = fiscalYear.endDate < today ? fiscalYear.endDate : today;
	const from = fiscalYear.startDate < to ? fiscalYear.startDate : to;
	return { from, to };
}

/** Le motif d'un refus de période (`PROVIDER_PERIOD_INVALID`, `details.reason`). */
export function periodRefusalLabel(reason: unknown): string {
	switch (reason) {
		case 'from_after_to':
			return i18nMsg('provider-download-period-from-after-to', 'La date de début doit précéder la date de fin.');
		case 'future':
			return i18nMsg('provider-download-period-future', 'La date de début est dans le futur.');
		case 'too_old':
			return i18nMsg(
				'provider-download-period-too-old',
				'La date de début remonte à plus de trois ans : le prestataire ne remonte pas plus loin.',
			);
		default:
			return i18nMsg(
				'provider-download-period-invalid',
				'Période refusée : le début doit précéder la fin, ne pas être dans le futur, et dater de moins de trois ans.',
			);
	}
}

/** Issue d'un refus du lancement : reprendre le suivi d'une tâche en cours, ou dire pourquoi. */
export type StartRefusal = { resumeJobId: number } | { message: string };

/**
 * Ce qu'un refus du `POST` devient à l'écran (AC-2). Sur `409
 * PROVIDER_DOWNLOAD_RUNNING`, le dialogue **reprend le suivi** de la tâche en
 * cours (`details.jobId`) au lieu d'afficher une erreur ; sinon, le motif.
 */
export function startRefusal(err: unknown): StartRefusal {
	if (isApiError(err)) {
		const details = err.details;
		switch (err.code) {
			case 'PROVIDER_DOWNLOAD_RUNNING': {
				const jobId = details?.jobId;
				if (typeof jobId === 'number' && Number.isInteger(jobId)) return { resumeJobId: jobId };
				break;
			}
			case 'PROVIDER_PERIOD_INVALID':
				return { message: periodRefusalLabel(details?.reason) };
			case 'PROVIDER_API_NOT_CONFIGURED':
				return {
					message: i18nMsg(
						'provider-download-not-configured',
						'Le téléchargement n’est pas configuré pour ce compte : voir le manuel d’administration. L’import du fichier reste possible.',
					),
				};
		}
	}
	return { message: errorMessageOf(err) };
}

/** `details.window` et `details.page` d'un échec, s'il y en a. */
function windowAndPage(details: Record<string, unknown> | undefined): string | null {
	const window = typeof details?.window === 'string' ? details.window : null;
	const page = typeof details?.page === 'number' ? details.page : null;
	if (window && page !== null)
		return i18nMsg('provider-download-error-window-page', 'Période { $window }, page { $page }.', {
			window,
			page: String(page),
		});
	if (window) return i18nMsg('provider-download-error-window', 'Période { $window }.', { window });
	return null;
}

/** Une erreur de ligne d'une réponse refusée (`details.lines[]`, forme de l'import CSV). */
interface LineError {
	line: number;
	field: string | null;
	code: string;
}

function linesOf(details: Record<string, unknown> | undefined): LineError[] {
	const lines = details?.lines;
	return Array.isArray(lines) ? (lines as LineError[]) : [];
}

/**
 * Les phrases d'une tâche en échec (AC-3) : le motif traduit du code
 * `PROVIDER_*`, la fenêtre et la page de `details` s'il y en a, les devises sans
 * compte, les lignes refusées (libellées par `lineLabel`, propre au
 * prestataire), puis « rien n'a été enregistré, relancez ». Un code inconnu de
 * l'écran (`INTERNAL_ERROR`, `BANK_ACCOUNT_CLOSED`…) garde le message du serveur.
 */
export function failureLines(error: ProviderDownloadError, lineLabel: (code: string) => string = (c) => c): string[] {
	const details = error.details;
	const out: string[] = [];
	switch (error.code) {
		case 'PROVIDER_AUTH_FAILED':
			out.push(
				i18nMsg(
					'provider-download-error-auth-failed',
					'Le prestataire refuse les identifiants de ce compte : l’administrateur doit les vérifier.',
				),
			);
			break;
		case 'PROVIDER_PERMISSION_DENIED':
			out.push(
				i18nMsg(
					'provider-download-error-permission-denied',
					'L’application du prestataire n’a pas le droit de lire les transactions : l’administrateur doit lui donner cette fonction.',
				),
			);
			break;
		case 'PROVIDER_RATE_LIMITED':
			out.push(
				i18nMsg(
					'provider-download-error-rate-limited',
					'Le prestataire limite le nombre de demandes, même après six essais : réessayez plus tard.',
				),
			);
			break;
		case 'PROVIDER_UNAVAILABLE':
			out.push(
				i18nMsg(
					'provider-download-error-unavailable',
					'Le prestataire ne répond pas, même après six essais : réessayez plus tard.',
				),
			);
			break;
		case 'PROVIDER_API_ERROR':
			out.push(i18nMsg('provider-download-error-api-error', 'Le prestataire a refusé la demande.'));
			break;
		case 'PROVIDER_CURRENCY_ACCOUNT_MISSING': {
			const currencies = Array.isArray(details?.currencies) ? details.currencies.map(String).join(', ') : '';
			out.push(
				i18nMsg(
					'provider-download-error-currency-account-missing',
					'Le téléchargement porte des mouvements en { $currencies }, sans compte bancaire de ce compte pour ces devises : créez-les dans les comptes bancaires.',
					{ currencies },
				),
			);
			break;
		}
		case 'PROVIDER_RESPONSE_INVALID': {
			out.push(
				i18nMsg('provider-download-error-response-invalid', 'La réponse du prestataire n’a pas pu être lue.'),
			);
			if (typeof details?.transactionId === 'string')
				out.push(
					i18nMsg(
						'provider-download-error-duplicate-transaction',
						'La transaction { $transactionId } est rendue deux fois avec des montants différents.',
						{ transactionId: details.transactionId },
					),
				);
			for (const l of linesOf(details))
				out.push(
					i18nMsg('provider-download-error-line', 'Transaction { $line }, { $field } : { $cause }', {
						line: String(l.line),
						field: l.field ?? '—',
						cause: lineLabel(l.code),
					}),
				);
			break;
		}
		default:
			out.push(
				error.message.trim() !== ''
					? error.message
					: i18nMsg('provider-download-error-unknown', 'Le téléchargement a échoué.'),
			);
	}
	const where = windowAndPage(details);
	if (where) out.push(where);
	out.push(i18nMsg('provider-download-error-nothing-saved', 'Rien n’a été enregistré : relancez le téléchargement.'));
	return out;
}

/** Le texte de progression : « fenêtre 5/13 », ou « fenêtre 5 » quand le total est inconnu. */
export function progressLabel(steps: { done: number; total: number | null }, stepLabel: string): string {
	return steps.total === null
		? i18nMsg('provider-download-progress-open', '{ $step } { $done }', { step: stepLabel, done: String(steps.done) })
		: i18nMsg('provider-download-progress', '{ $step } { $done }/{ $total }', {
				step: stepLabel,
				done: String(steps.done),
				total: String(steps.total),
			});
}

/** La part faite, de 0 à 100 ; `null` sans total (compteur sans barre). */
export function progressPercent(steps: { done: number; total: number | null }): number | null {
	if (steps.total === null) return null;
	if (steps.total <= 0) return 100;
	return Math.min(100, Math.round((steps.done / steps.total) * 100));
}

/** Ce que fait le suivi après une erreur de `GET` : réessayer, ou déclarer le suivi perdu. */
export function afterPollError(err: unknown, consecutiveErrors: number): 'retry' | 'lost' {
	if (isApiError(err) && err.status === 404) return 'lost';
	return consecutiveErrors >= MAX_POLL_ERRORS ? 'lost' : 'retry';
}
