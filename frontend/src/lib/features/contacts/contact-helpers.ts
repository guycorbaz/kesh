/**
 * Helpers purs pour les contacts (Story 4.1).
 *
 * - `formatIdeNumber` : `CHE109322551` → `CHE-109.322.551`
 * - `validateIdeFormat` : regex TS native `/^CHE[0-9]{9}$/` (format seulement,
 *   pas le checksum — la validation complète est au backend via CheNumber).
 * - `formatContactType` : traduction via i18n canonical.
 */

import type { ContactType } from './contacts.types';

/**
 * Convertit la forme normalisée (12 chars) en forme d'affichage avec séparateurs.
 * Retourne la chaîne telle quelle si elle ne matche pas le format attendu.
 */
export function formatIdeNumber(normalized: string | null | undefined): string {
	if (!normalized) return '';
	if (!/^CHE[0-9]{9}$/.test(normalized)) return normalized;
	const digits = normalized.slice(3);
	return `CHE-${digits.slice(0, 3)}.${digits.slice(3, 6)}.${digits.slice(6, 9)}`;
}

/**
 * Validation client-side du format normalisé uniquement (pas le checksum).
 * Accepte aussi la forme avec séparateurs pour l'UX temps réel.
 * Le backend valide le checksum définitivement via `CheNumber::new`.
 */
export function validateIdeFormat(input: string): boolean {
	const normalized = input.replace(/[\s\-.]/g, '').toUpperCase();
	return /^CHE[0-9]{9}$/.test(normalized);
}

/**
 * Normalise un input IDE vers la forme sans séparateurs avant envoi API.
 * Retourne `null` si vide après normalisation.
 */
export function normalizeIdeForApi(input: string | null | undefined): string | null {
	if (!input) return null;
	const normalized = input.replace(/[\s\-.]/g, '').toUpperCase();
	return normalized === '' ? null : normalized;
}

/**
 * Traduit un `ContactType` via une clé i18n (résolue par le composant appelant).
 * Retourne juste la clé ici — le caller applique `i18nMsg(key, fallback)`.
 */
export function contactTypeI18nKey(type: ContactType): string {
	switch (type) {
		case 'Personne':
			return 'contact-type-personne';
		case 'Entreprise':
			return 'contact-type-entreprise';
	}
}
