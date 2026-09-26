/**
 * Store i18n partagé (Svelte 5 runes).
 *
 * Canonical location pour le runtime i18n frontend. Tout composant ou
 * feature qui résout une clé i18n doit importer depuis ici, pas depuis
 * `$lib/features/onboarding/onboarding.svelte` (couplage transverse).
 */

import { apiClient } from '$lib/shared/utils/api-client';

let _messages = $state<Record<string, string>>({});
// Story 25-1c-b1 (#378) — la locale de l'interface, servie avec les messages.
// Elle était jetée ; l'écran du journal d'audit en a besoin pour trier les
// listes (`Intl.Collator`) et formater les dates (`Intl.DateTimeFormat`) dans
// la langue de l'installation. `'fr-CH'` avant tout chargement.
let _locale = $state('fr-CH');

/** Résout un message i18n avec fallback. */
export function i18nMsg(key: string, fallback: string, args?: Record<string, string | number>): string {
	const raw = _messages[key] || fallback;
	if (!args) return raw;
	// Le backend pré-résout les messages Fluent SANS arguments : une variable
	// `{ $x }` non résolue en ressort entourée des marques d'isolation
	// bidirectionnelle U+2068 (FSI) / U+2069 (PDI) — invisibles à l'écran, mais
	// bien présentes dans le DOM, où elles cassent toute assertion ou regex
	// portant sur la valeur interpolée. On les consomme avec le placeholder.
	return raw.replace(/\u2068?\{\s*\$(\w+)\s*\}\u2069?/g, (_, k) => String(args[k] ?? ''));
}

/**
 * La locale de l'interface (`KESH_LANG`), telle que le serveur la sert avec
 * les messages — `'fr-CH'` avant tout chargement. Kesh n'a pas de langue par
 * utilisateur : c'est celle de l'installation. ⚠️ Ne pas y substituer
 * `navigator.language` : la langue du navigateur n'est pas celle de Kesh.
 */
export function i18nLocale(): string {
	return _locale;
}

/** Charge les traductions depuis l'API (appel idempotent côté serveur). */
export async function loadI18nMessages(): Promise<void> {
	try {
		const data = await apiClient.get<{ locale: string; messages: Record<string, string> }>(
			'/api/v1/i18n/messages'
		);
		_messages = data.messages;
		if (data.locale) _locale = data.locale;
	} catch {
		// Fallback silencieux — les labels par défaut sont en français
	}
}
