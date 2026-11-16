/**
 * Story 37-2 (AC-6, AC-8) — les textes de l'écran *Cours de change* et du bandeau
 * des corrections, traduits par `i18nMsg` (repli français). Libellés « cours
 * mensuel AFC » et « cours journalier », jamais « cours moyen ».
 */

import { isApiError } from '$lib/shared/utils/api-client';
import { errorMessageOf } from '$lib/shared/utils/error-message';
import { i18nMsg } from '$lib/shared/utils/i18n.svelte';
import { formatSwissDate, formatSwissDateTime } from '$lib/features/reports/reports.api';
import { formatRate, monthLongLabel, type ArrowKind, type Mention, type NotFetchableReason } from './exchange-rates-grid';
import type { FetchStatusResponse, RateKind } from './exchange-rates.types';

/** Le texte de la mention d'une case (source et valeur publiée). */
export function mentionText(m: Mention): string {
	const rate = formatRate(m.published);
	const suffix = m.otherSource ? ` (${m.otherSource})` : '';
	switch (m.kind) {
		case 'afc-ok':
			return i18nMsg('exchange-rates-mention-afc-ok', 'AFC ✓');
		case 'afc-corrected':
			return i18nMsg('exchange-rates-mention-afc-corrected', 'AFC corrigé : { $rate } (le { $date })', {
				rate,
				date: m.correctedAt ? formatSwissDate(m.correctedAt) : '',
			});
		case 'afc-unverified':
			return i18nMsg('exchange-rates-mention-afc-unverified', 'AFC (valeur publiée non relue)');
		case 'saisi-differs':
		case 'none-published':
			return i18nMsg('exchange-rates-mention-published', 'AFC : { $rate }', { rate }) + suffix;
		case 'saisi-equal':
			return i18nMsg('exchange-rates-mention-saisi-equal', 'AFC ✓ · saisi') + suffix;
		case 'saisi-unpublished':
		case 'none':
			return i18nMsg('exchange-rates-mention-unpublished', 'AFC : non publié') + suffix;
	}
}

/** Le libellé accessible du ↑. */
export function arrowLabel(kind: ArrowKind): string {
	switch (kind) {
		case 'corrected':
			return i18nMsg('exchange-rates-arrow-corrected', 'Reprendre la valeur corrigée');
		case 'published':
			return i18nMsg('exchange-rates-arrow-published', 'Reprendre la valeur publiée');
		case 'remove':
			return i18nMsg('exchange-rates-arrow-remove', 'Retirer le cours saisi');
	}
}

/** La raison, au survol, d'une case non récupérable (elle reste saisissable). */
export function notFetchableText(kind: RateKind, reason: NotFetchableReason): string {
	if (reason === 'before-2000') {
		return i18nMsg(
			'exchange-rates-not-fetchable-before-2000',
			'Antérieur au 1er janvier 2000 : ce cours ne se récupère pas. Il reste saisissable.',
		);
	}
	return kind === 'day'
		? i18nMsg(
				'exchange-rates-not-fetchable-day',
				'Jour non passé : son cours journalier ne se récupère pas encore. Il reste saisissable.',
			)
		: i18nMsg(
				'exchange-rates-not-fetchable-month',
				'Mois au-delà du mois suivant : son cours mensuel AFC ne se récupère pas encore. Il reste saisissable.',
			);
}

/** Le texte de l'échec d'une récupération, affiché sur les cases de sa colonne. */
export function fetchFailureText(err: unknown): string {
	if (isApiError(err)) {
		if (err.code === 'RATE_NOT_PUBLISHED') {
			return i18nMsg('exchange-rates-fetch-not-published', 'Pas encore publié par l’AFC.');
		}
		if (err.code === 'RATE_SOURCE_UNAVAILABLE') {
			return i18nMsg(
				'exchange-rates-fetch-source-unavailable',
				'La source des cours ne répond pas ou a rendu une réponse inutilisable.',
			);
		}
	}
	return errorMessageOf(err);
}

/** Le code d'erreur classé d'une récupération, traduit (`timeout`, `http_status:500`…). */
export function fetchErrorCodeText(code: string): string {
	const [nom, n = ''] = code.split(':');
	switch (nom) {
		case 'timeout':
			return i18nMsg('exchange-rates-error-timeout', 'la source n’a pas répondu à temps');
		case 'connect':
			return i18nMsg('exchange-rates-error-connect', 'la source est injoignable');
		case 'http_status':
			return i18nMsg('exchange-rates-error-http-status', 'la source a répondu par une erreur ({ $status })', {
				status: n,
			});
		case 'too_large':
			return i18nMsg('exchange-rates-error-too-large', 'la réponse de la source est trop volumineuse');
		case 'malformed':
			return i18nMsg('exchange-rates-error-malformed', 'la réponse de la source est illisible');
		case 'not_published':
			return i18nMsg('exchange-rates-error-not-published', 'pas encore publié');
		case 'month_mismatch':
			return i18nMsg('exchange-rates-error-month-mismatch', 'la source a rendu un autre mois que celui demandé');
		case 'date_mismatch':
			return i18nMsg('exchange-rates-error-date-mismatch', 'la source a rendu une autre date que celle demandée');
		case 'no_valid_rate':
			return i18nMsg('exchange-rates-error-no-valid-rate', 'la réponse de la source ne porte aucun cours valable');
		case 'invalid_rates':
			return i18nMsg('exchange-rates-error-invalid-rates', '{ $count } cours écartés comme invalides', {
				count: n,
			});
		default:
			return code;
	}
}

/** Le genre d'un cours en toutes lettres. */
export function kindText(kind: RateKind): string {
	return kind === 'month'
		? i18nMsg('exchange-rates-kind-month', 'cours mensuel AFC')
		: i18nMsg('exchange-rates-kind-day', 'cours journalier');
}

/** L'unité d'un cours : « juillet 2024 » ou « 30.06.2024 ». */
export function unitLongText(kind: RateKind, rateDate: string, locale: string): string {
	return kind === 'month' ? monthLongLabel(rateDate, locale) : formatSwissDate(rateDate);
}

/** L'état de la dernière récupération, ou `null` avant toute récupération. */
export function statusText(s: FetchStatusResponse, locale: string): string | null {
	if (!s.lastRunAt || !s.status) return null;
	const trigger =
		s.trigger === 'manual'
			? i18nMsg('exchange-rates-status-trigger-manual', 'manuelle')
			: i18nMsg('exchange-rates-status-trigger-auto', 'automatique');
	const unit = s.kind && s.rateDate ? `${kindText(s.kind)}, ${unitLongText(s.kind, s.rateDate, locale)}` : '';
	let outcome: string;
	if (s.status === 'ok') {
		outcome = i18nMsg('exchange-rates-status-ok', 'réussie');
		if (s.error) outcome += ` (${fetchErrorCodeText(s.error)})`;
	} else if (s.status === 'not_published') {
		outcome = i18nMsg('exchange-rates-status-not-published', 'pas encore publié');
	} else {
		outcome = i18nMsg('exchange-rates-status-error', 'échec : { $reason }', {
			reason: s.error ? fetchErrorCodeText(s.error) : '',
		});
	}
	return i18nMsg(
		'exchange-rates-status-line',
		'Dernière récupération ({ $trigger }) le { $at } — { $unit } : { $outcome }',
		{ trigger, at: formatSwissDateTime(s.lastRunAt), unit, outcome },
	);
}
