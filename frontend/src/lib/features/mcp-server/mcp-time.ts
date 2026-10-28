// Story 32-7 (refs #723) — les dates de l'écran « Serveur MCP » (AC-4 ; #738 B7 = E6).
//
// Les dates d'une clé sont des `NaiveDateTime` UTC sérialisées SANS fuseau
// (`2026-10-25T10:00:00.123`) ; `new Date()` les lirait en heure LOCALE. Celles des
// réglages portent leur `Z`. `depuisUtcNaif` lit les deux comme de l'UTC.
//
// L'expiration va dans l'autre sens : la date choisie (`AAAA-MM-JJ`) est la
// dernière journée de validité en heure locale du navigateur, envoyée en UTC par
// `toISOString()` (le serveur exige un fuseau).

import { i18nLocale } from '$lib/shared/utils/i18n.svelte';

/** Une chaîne qui porte déjà un fuseau : `Z` ou `±hh:mm` / `±hhmm` final. */
const AVEC_FUSEAU = /([zZ]|[+-]\d{2}:?\d{2})$/;

/**
 * Lit une date du serveur comme de l'UTC. Une chaîne sans fuseau reçoit un `Z` ;
 * une chaîne qui en porte un est lue telle quelle. `null`, vide ou illisible → `null`
 * (jamais `Invalid Date`).
 */
export function depuisUtcNaif(s: string | null | undefined): Date | null {
	if (typeof s !== 'string' || s.trim() === '') return null;
	const texte = s.trim();
	const d = new Date(AVEC_FUSEAU.test(texte) ? texte : `${texte}Z`);
	return Number.isNaN(d.getTime()) ? null : d;
}

/** Date et heure dans le fuseau du navigateur, dans la langue de l'installation. */
export function dateHeureAffichee(s: string | null | undefined): string | null {
	const d = depuisUtcNaif(s);
	if (d === null) return null;
	return new Intl.DateTimeFormat(i18nLocale(), { dateStyle: 'short', timeStyle: 'short' }).format(d);
}

/** Date seule dans le fuseau du navigateur (`31.12.2026` en `fr-CH`). */
export function dateAffichee(s: string | null | undefined): string | null {
	const d = depuisUtcNaif(s);
	if (d === null) return null;
	return new Intl.DateTimeFormat(i18nLocale(), { day: '2-digit', month: '2-digit', year: 'numeric' }).format(d);
}

/**
 * L'expiration saisie (`AAAA-MM-JJ`) → la fin de cette journée en heure locale
 * (`23:59:59.999`), en ISO UTC. `null` si la saisie est vide ou illisible.
 */
export function finDeJourneeLocale(date: string): string | null {
	const m = /^(\d{4})-(\d{2})-(\d{2})$/.exec(date.trim());
	if (!m) return null;
	const d = new Date(Number(m[1]), Number(m[2]) - 1, Number(m[3]), 23, 59, 59, 999);
	return Number.isNaN(d.getTime()) ? null : d.toISOString();
}

/** Le lendemain local, `AAAA-MM-JJ` — le `min` du champ d'expiration. */
export function lendemainLocal(maintenant: Date = new Date()): string {
	const d = new Date(maintenant.getFullYear(), maintenant.getMonth(), maintenant.getDate() + 1);
	const mm = String(d.getMonth() + 1).padStart(2, '0');
	const jj = String(d.getDate()).padStart(2, '0');
	return `${d.getFullYear()}-${mm}-${jj}`;
}

/** Vrai si l'instant est passé (une clé expirée). */
export function estPasse(s: string | null | undefined, maintenant: Date = new Date()): boolean {
	const d = depuisUtcNaif(s);
	return d !== null && d.getTime() <= maintenant.getTime();
}
