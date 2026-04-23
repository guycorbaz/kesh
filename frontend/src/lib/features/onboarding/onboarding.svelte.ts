/**
 * Store d'onboarding (Svelte 5 runes).
 *
 * Gère l'état du wizard et synchronise avec l'API.
 * Les POST retournent l'état complet — le store se met à jour à partir de la réponse.
 */

import type { OnboardingState } from './onboarding.types';
import * as api from './onboarding.api';

// Re-export canonique depuis le module partagé — évite un couplage transverse
// où des composants `shared/` importeraient depuis `features/onboarding/`.
export { i18nMsg, loadI18nMessages } from '$lib/shared/utils/i18n.svelte';

let _state = $state<OnboardingState>({
	stepCompleted: 0,
	isDemo: false,
	uiMode: null,
});
let _loaded = $state(false);
let _loading = $state(false);

export const onboardingState = {
	get stepCompleted(): number {
		return _state.stepCompleted;
	},
	get isDemo(): boolean {
		return _state.isDemo;
	},
	get uiMode(): 'guided' | 'expert' | null {
		return _state.uiMode;
	},
	get loaded(): boolean {
		return _loaded;
	},
	get loading(): boolean {
		return _loading;
	},

	async fetchState() {
		_loading = true;
		try {
			_state = await api.fetchState();
			_loaded = true;
		} finally {
			_loading = false;
		}
	},

	async setLanguage(language: string) {
		_loading = true;
		try {
			_state = await api.setLanguage(language);
		} finally {
			_loading = false;
		}
	},

	async setMode(mode: string) {
		_loading = true;
		try {
			_state = await api.setMode(mode);
		} finally {
			_loading = false;
		}
	},

	async seedDemo() {
		_loading = true;
		try {
			_state = await api.seedDemo();
		} finally {
			_loading = false;
		}
	},

	async resetDemo() {
		_loading = true;
		try {
			_state = await api.resetDemo();
		} finally {
			_loading = false;
		}
	},

	// --- Path B (Story 2.3) ---

	async startProduction() {
		_loading = true;
		try {
			_state = await api.startProduction();
		} finally {
			_loading = false;
		}
	},

	async setOrgType(orgType: string) {
		_loading = true;
		try {
			_state = await api.setOrgType(orgType);
		} finally {
			_loading = false;
		}
	},

	async setAccountingLanguage(language: string) {
		_loading = true;
		try {
			_state = await api.setAccountingLanguage(language);
		} finally {
			_loading = false;
		}
	},

	async setCoordinates(name: string, address: string, ideNumber: string | null) {
		_loading = true;
		try {
			_state = await api.setCoordinates(name, address, ideNumber);
		} finally {
			_loading = false;
		}
	},

	async setBankAccount(bankName: string, iban: string, qrIban: string | null) {
		_loading = true;
		try {
			_state = await api.setBankAccount(bankName, iban, qrIban);
		} finally {
			_loading = false;
		}
	},

	async skipBank() {
		_loading = true;
		try {
			_state = await api.skipBank();
		} finally {
			_loading = false;
		}
	},

	async finalize() {
		_loading = true;
		try {
			_state = await api.finalize();
		} finally {
			_loading = false;
		}
	},
};
