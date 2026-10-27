// Story 32-10b (refs #723) — fonctions pures de l'écran des prompts du serveur MCP.
//
// ⛔ L'écran REFLÈTE les règles du serveur (AC-i6, AC-i7 de la 32-10a), il ne
// les refait pas : la version en vigueur est celle que nomme `numeroEnVigueur`,
// le verdict `servable` est celui de `composer_version`. Et l'écran n'a AUCUNE
// logique Unicode (fiche 32-10b, AC-6) : ni prédicat, ni classe du moteur
// JavaScript — ce qui est rendu visible l'est par le serveur.

import { i18nMsg } from '$lib/shared/utils/i18n.svelte';
import { isApiError } from '$lib/shared/utils/api-client';
import { errorMessageOf } from '$lib/shared/utils/error-message';
import type {
	Etat,
	McpPromptDetail,
	McpPromptSummary,
	McpPromptVersion,
	Nature,
	Origine,
} from './mcp-prompts.types';

// La mise en forme des dates est celle de l'écran des envois (UTC de la base
// montrée à l'heure locale) : une seule fonction, réemployée (C-32-10b-23).
export { dateLocale } from '$lib/features/mcp-sends/mcp-sends-format';

/** Le libellé d'une origine. */
export function origineAffichee(origine: Origine): string {
	switch (origine) {
		case 'standard':
			return i18nMsg('mcp-prompts-origin-standard', 'Standard');
		case 'admin':
			return i18nMsg('mcp-prompts-origin-admin', 'Administrateur');
		case 'agent':
			return i18nMsg('mcp-prompts-origin-agent', 'Agent');
	}
}

/** Le libellé d'un état. */
export function etatAffiche(etat: Etat): string {
	switch (etat) {
		case 'approved':
			return i18nMsg('mcp-prompts-state-approved', 'Approuvée');
		case 'pending':
			return i18nMsg('mcp-prompts-state-pending', 'À valider');
		case 'refused':
			return i18nMsg('mcp-prompts-state-refused', 'Refusée');
	}
}

/** Le libellé d'une nature. */
export function natureAffichee(nature: Nature): string {
	return nature === 'text'
		? i18nMsg('mcp-prompts-nature-text', 'Texte')
		: i18nMsg('mcp-prompts-nature-withdrawal', 'Retrait');
}

/** La version en vigueur : celle que nomme le serveur (`numeroEnVigueur`), nulle sinon. */
export function versionEnVigueur(detail: McpPromptDetail): McpPromptVersion | null {
	if (detail.numeroEnVigueur === null) return null;
	return detail.versions.find((v) => v.numero === detail.numeroEnVigueur) ?? null;
}

/** La dernière version du prompt : le plus grand numéro de la réponse. */
export function derniereVersion(detail: McpPromptDetail): McpPromptVersion | null {
	let derniere: McpPromptVersion | null = null;
	for (const v of detail.versions) {
		if (derniere === null || v.numero > derniere.numero) derniere = v;
	}
	return derniere;
}

/** Une proposition d'agent qui attend une décision. */
function aValider(v: McpPromptVersion): boolean {
	return v.origine === 'agent' && v.etat === 'pending';
}

/**
 * *Approuver* n'existe que sur la DERNIÈRE version, si elle est une proposition
 * d'agent à valider, de texte, et que le serveur la dit `servable` (AC-7 ;
 * C-32-10b-17 — le serveur refuserait l'approbation d'une version non servable).
 */
export function peutApprouver(detail: McpPromptDetail, v: McpPromptVersion): boolean {
	return (
		aValider(v) &&
		v.nature === 'text' &&
		v.servable &&
		derniereVersion(detail)?.numero === v.numero
	);
}

/** *Refuser* : toute proposition d'agent à valider, dernière ou non (AC-i7 de la 32-10a). */
export function peutRefuser(v: McpPromptVersion): boolean {
	return aValider(v);
}

/** Les propositions à valider, plus récente en tête. */
export function versionsAValider(detail: McpPromptDetail): McpPromptVersion[] {
	return detail.versions.filter(aValider).sort((a, b) => b.numero - a.numero);
}

/**
 * *Revenir au texte standard* : le drapeau du serveur levé ET le texte du code
 * connu (un standard que ce binaire ne porte plus serait refusé en 409 — AC-10).
 */
export function peutRevenirAuStandard(detail: McpPromptDetail): boolean {
	return detail.standardDiffere && detail.standardDuCode !== null;
}

/**
 * L'étiquette d'un standard dans la liste (AC-3, C-32-10b-19) : le drapeau
 * `standardDiffere` n'est PAS affiché tel quel — il ne distingue pas « vous
 * l'avez personnalisé » de « le texte de Kesh a changé ».
 * - `personnalise` : la version en vigueur d'un standard n'est pas d'origine `standard` ;
 * - `differe` : elle est d'origine `standard` et le drapeau est levé ;
 * - `null` sinon (prompt non standard, standard à jour, aucune version en vigueur).
 */
export function etiquetteStandard(p: McpPromptSummary): 'personnalise' | 'differe' | null {
	if (!p.standard || p.enVigueur === null) return null;
	if (p.enVigueur.origine !== 'standard') return 'personnalise';
	return p.standardDiffere ? 'differe' : null;
}

/** Ce que l'écran montre après un geste refusé par le serveur. */
export interface ErreurDeGeste {
	message: string;
	/** Vrai pour un 409 : le prompt a changé, l'écran propose *Recharger*. */
	recharger: boolean;
}

/**
 * Traduit l'erreur d'un geste (AC-12). **Aucun contrôle du contenu côté
 * écran** : le serveur est seul juge, son message est montré.
 *
 * - 409 `ILLEGAL_STATE_TRANSITION` (non la dernière, déjà décidée, standard
 *   inconnu du code…) : le message du serveur, déjà traduit ; tout autre 409
 *   (`OPTIMISTIC_LOCK_CONFLICT`, verrou périmé) : le message de l'écran ; les
 *   deux avec *Recharger* — **aucune relance automatique** ;
 * - 404 : « Prompt introuvable » ;
 * - 400 : le message du serveur (nom, contenu, version non servable — D2) ;
 * - 403 : « Réservé à l'Administrateur » ;
 * - le reste : le message lisible de l'erreur.
 */
export function erreurDeGeste(err: unknown): ErreurDeGeste {
	if (isApiError(err) && err.status === 409) {
		const message =
			err.code === 'ILLEGAL_STATE_TRANSITION'
				? errorMessageOf(err)
				: i18nMsg(
						'mcp-prompts-error-conflict',
						'Ce prompt a changé depuis son affichage : rechargez-le avant d’agir.',
					);
		return { message, recharger: true };
	}
	if (isApiError(err) && err.status === 404) {
		return { message: i18nMsg('mcp-prompts-error-not-found', 'Prompt introuvable.'), recharger: false };
	}
	if (isApiError(err) && err.status === 403) {
		return {
			message: i18nMsg('mcp-prompts-error-forbidden', 'Réservé à l’Administrateur.'),
			recharger: false,
		};
	}
	return { message: errorMessageOf(err), recharger: false };
}
