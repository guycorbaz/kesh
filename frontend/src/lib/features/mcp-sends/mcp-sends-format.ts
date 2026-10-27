// Story 32-8c (refs #723) — fonctions pures de l'écran des envois du serveur MCP.

import { i18nLocale, i18nMsg } from '$lib/shared/utils/i18n.svelte';
import { isApiError } from '$lib/shared/utils/api-client';
import { errorMessageOf } from '$lib/shared/utils/error-message';
import type { McpSendState } from './mcp-sends.types';

/**
 * La taille maximale d'un envoi, en octets UTF-8 — la valeur de
 * `MCP_ENVOI_OCTETS_MAX` (`crates/kesh-api/src/services/envois_mcp.rs`), que
 * le serveur applique au dépôt comme à une modification. Tenue égale à la
 * constante Rust par `mcp-sends-i18n.test.ts`, qui lit la source.
 */
export const MCP_ENVOI_OCTETS_MAX = 24_576;

/**
 * La taille d'un texte en **octets UTF-8** — la mesure du serveur (`str::len`),
 * non le nombre de caractères : « é » en compte deux, « ✓ » trois, un emoji
 * quatre. Un texte venu d'une `textarea` n'a jamais de demi-paire de
 * substitution ; `TextEncoder` la compterait trois octets (U+FFFD), le serveur
 * ne la recevrait pas — sans objet ici.
 */
export function octetsUtf8(texte: string): number {
	return new TextEncoder().encode(texte).length;
}

/** Un entier mis en forme selon la langue de l'installation (« 24 576 »). */
export function nombreLocal(n: number): string {
	return new Intl.NumberFormat(i18nLocale()).format(n);
}

/**
 * Une date de la base (`2026-10-26T09:15:00.123`, UTC sans suffixe) montrée à
 * l'heure **locale**, comme le journal d'audit. Une valeur illisible est
 * rendue telle quelle plutôt que masquée.
 */
export function dateLocale(valeur: string | null): string {
	if (!valeur) return '—';
	const iso = /(Z|[+-]\d{2}:\d{2})$/.test(valeur) ? valeur : `${valeur}Z`;
	const d = new Date(iso);
	if (Number.isNaN(d.getTime())) return valeur;
	return new Intl.DateTimeFormat(i18nLocale(), { dateStyle: 'medium', timeStyle: 'short' }).format(d);
}

/** Le libellé du badge d'état ; un envoi validé modifié le dit. */
export function etatAffiche(state: McpSendState, modified: boolean): string {
	switch (state) {
		case 'pending':
			return i18nMsg('mcp-sends-state-pending', 'En attente');
		case 'approved':
			return modified
				? i18nMsg('mcp-sends-state-approved-modified', 'Validé — modifié')
				: i18nMsg('mcp-sends-state-approved', 'Validé');
		case 'refused':
			return i18nMsg('mcp-sends-state-refused', 'Refusé');
		case 'expired':
			return i18nMsg('mcp-sends-state-expired', 'Échu');
	}
}

/** Ce que l'écran montre après une décision refusée par le serveur. */
export interface ErreurDeDecision {
	message: string;
	/** Vrai pour un 409 : l'envoi a changé, l'écran propose *Recharger*. */
	recharger: boolean;
	/** Vrai pour un 400 sur une version modifiée : l'aide aux caractères refusés. */
	aideCaracteres: boolean;
}

/**
 * Traduit l'erreur d'une décision (AC-C5). **Aucun contrôle du contenu côté
 * client** (C-32-8-20) : le serveur est seul juge, son message est montré.
 *
 * - 409 `ILLEGAL_STATE_TRANSITION` (déjà décidé, échu) : le message du serveur,
 *   déjà traduit ; 409 `OPTIMISTIC_LOCK_CONFLICT` (version périmée) : un message
 *   propre à l'écran ; les deux avec *Recharger* ;
 * - 404 : « Envoi introuvable » ;
 * - 400 : le message du serveur (taille, caractère invisible) ;
 * - le reste : le message lisible de l'erreur.
 */
export function erreurDeDecision(err: unknown, modifie: boolean): ErreurDeDecision {
	if (isApiError(err) && err.status === 409) {
		const message =
			err.code === 'ILLEGAL_STATE_TRANSITION'
				? errorMessageOf(err)
				: i18nMsg(
						'mcp-sends-error-conflict',
						'Cet envoi a changé depuis son affichage : rechargez-le avant de décider.',
					);
		return { message, recharger: true, aideCaracteres: false };
	}
	if (isApiError(err) && err.status === 404) {
		return {
			message: i18nMsg('mcp-sends-error-not-found', 'Envoi introuvable.'),
			recharger: false,
			aideCaracteres: false,
		};
	}
	if (isApiError(err) && err.status === 400) {
		return { message: errorMessageOf(err), recharger: false, aideCaracteres: modifie };
	}
	return { message: errorMessageOf(err), recharger: false, aideCaracteres: false };
}
