/**
 * Story 38-11d (AC-2) — les trois filtres facultatifs d'une règle de
 * rapprochement (38-11c1, AC-2 et AC-3), tenus hors du composant pour être
 * testés seuls : « Type chez le prestataire » (exact ; PayPal, Wise depuis la
 * 39-11d), « E-mail exact » et le sens. Ils se
 * combinent au critère existant (`matchType` / `matchValue`, toujours
 * obligatoire) ; le serveur reste l'autorité de leur forme.
 */
import type { ReconciliationRule, RuleDirection } from './rules.types';

/**
 * Forme normalisée d'un e-mail de filtre, miroir de
 * `kesh_reconciliation::rules::normalize_email` : tout blanc retiré,
 * minuscules.
 */
export function normalizeEmailFilter(value: string): string {
	return value.replace(/\s+/g, '').toLowerCase();
}

/**
 * L'e-mail saisi est-il acceptable ? Vide (aucun filtre), ou, normalisé, un
 * seul `@` avec un caractère au moins de chaque côté — le contrôle du serveur
 * (38-11c1, AC-2), qui reste l'autorité (longueur comprise).
 */
export function isValidEmailFilter(value: string): boolean {
	const v = normalizeEmailFilter(value);
	if (v === '') return true;
	const parts = v.split('@');
	return parts.length === 2 && parts[0] !== '' && parts[1] !== '';
}

/** Valeur d'un filtre texte : rognée, `null` si vide. */
export function textFilterPayload(value: string): string | null {
	const v = value.trim();
	return v === '' ? null : v;
}

/** Valeur du filtre e-mail : normalisée, `null` si vide. */
export function emailFilterPayload(value: string): string | null {
	const v = normalizeEmailFilter(value);
	return v === '' ? null : v;
}

/** Valeur du sélecteur « Sens » (`''` = entrées et sorties) → charge. */
export function directionPayload(value: '' | RuleDirection): RuleDirection | null {
	return value === '' ? null : value;
}

/** Les trois filtres d'une règle, tels que l'écran les envoie. */
export interface RuleFilters {
	providerType: string | null;
	counterpartyEmail: string | null;
	direction: RuleDirection | null;
}

/**
 * Les filtres du `PATCH` : seuls ceux qui **changent** sont présents (un
 * filtre absent reste inchangé côté serveur, `null` l'efface). Une règle
 * ouverte puis enregistrée sans toucher aux filtres n'en envoie aucun.
 */
export function changedFilters(
	before: Pick<ReconciliationRule, 'providerType' | 'counterpartyEmail' | 'direction'>,
	after: RuleFilters,
): Partial<RuleFilters> {
	const patch: Partial<RuleFilters> = {};
	if ((before.providerType ?? null) !== after.providerType) patch.providerType = after.providerType;
	if ((before.counterpartyEmail ?? null) !== after.counterpartyEmail) {
		patch.counterpartyEmail = after.counterpartyEmail;
	}
	if ((before.direction ?? null) !== after.direction) patch.direction = after.direction;
	return patch;
}
