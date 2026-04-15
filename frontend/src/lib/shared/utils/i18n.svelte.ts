/**
 * Store i18n partagé (Svelte 5 runes).
 *
 * Canonical location pour le runtime i18n frontend. Tout composant ou
 * feature qui résout une clé i18n doit importer depuis ici, pas depuis
 * `$lib/features/onboarding/onboarding.svelte` (couplage transverse).
 */

import { apiClient } from '$lib/shared/utils/api-client';

let _messages = $state<Record<string, string>>({});

/** Résout un message i18n avec fallback. */
export function i18nMsg(key: string, fallback: string, args?: Record<string, string | number>): string {
	const raw = _messages[key] || fallback;
	if (!args) return raw;
	return raw.replace(/\{\s*\$(\w+)\s*\}/g, (_, k) => String(args[k] ?? ''));
}

/** Charge les traductions depuis l'API (appel idempotent côté serveur). */
export async function loadI18nMessages(): Promise<void> {
	try {
		const data = await apiClient.get<{ locale: string; messages: Record<string, string> }>(
			'/api/v1/i18n/messages'
		);
		_messages = data.messages;
	} catch {
		// Fallback silencieux — les labels par défaut sont en français
	}
}
