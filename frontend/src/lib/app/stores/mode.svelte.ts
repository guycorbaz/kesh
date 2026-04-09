/**
 * Mode d'interaction : Guidé (débutants) ou Expert (professionnels).
 *
 * Contrôle les espacements, densité d'information et taille des cibles
 * via les CSS custom properties `--kesh-*` définies dans `app.css`.
 *
 * Persistence double :
 * - localStorage pour chargement instantané (pas de flash au refresh)
 * - Serveur via PUT /api/v1/profile/mode (sync cross-device)
 */

import { apiClient } from '$lib/shared/utils/api-client';

export type Mode = 'guided' | 'expert';

const STORAGE_KEY = 'kesh-mode';

/** Safe localStorage read (handles private browsing SecurityError). */
function readStorage(): string | null {
	try {
		return typeof localStorage !== 'undefined' ? localStorage.getItem(STORAGE_KEY) : null;
	} catch {
		return null;
	}
}

/** Safe localStorage write. */
function writeStorage(v: string) {
	try {
		if (typeof localStorage !== 'undefined') {
			localStorage.setItem(STORAGE_KEY, v);
		}
	} catch {
		// Private browsing or storage quota — silently ignore
	}
}

// Init depuis localStorage (instantané, pas de flash)
const stored = readStorage();
let _mode = $state<Mode>(stored === 'expert' ? 'expert' : 'guided');

/** Objet réactif — `modeState.value` est trackable par `$effect`. */
export const modeState = {
	get value(): Mode {
		return _mode;
	},
	set value(v: Mode) {
		_mode = v;
		writeStorage(v);
	},
};

export function toggleMode() {
	const next: Mode = _mode === 'guided' ? 'expert' : 'guided';
	modeState.value = next; // Passe par le setter → localStorage
	// Fire-and-forget PUT au serveur — erreur non bloquante
	apiClient.put('/api/v1/profile/mode', { mode: next }).catch((err) => {
		console.error('[mode] Failed to sync mode to server:', err);
	});
}

/**
 * Synchronise le mode depuis le serveur (onboarding_state.uiMode).
 * Appelé au startup dans (app)/+layout.ts après fetch de l'onboarding state.
 * Le serveur fait foi en cas de divergence avec localStorage.
 */
export function syncModeFromServer(uiMode: 'guided' | 'expert' | null) {
	if (uiMode && uiMode !== _mode) {
		modeState.value = uiMode; // Passe par le setter → localStorage
	}
}
