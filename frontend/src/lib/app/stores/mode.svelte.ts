/**
 * Mode d'interaction : Guidé (débutants) ou Expert (professionnels).
 *
 * Contrôle les espacements, densité d'information et taille des cibles
 * via les CSS custom properties `--kesh-*` définies dans `app.css`.
 *
 * Exposé via un objet avec getter réactif (`modeState.value`) pour que
 * `$effect` dans le layout puisse tracker les changements.
 *
 * La persistance localStorage sera ajoutée en Story 2.5.
 */

export type Mode = 'guided' | 'expert';

let _mode = $state<Mode>('guided');

/** Objet réactif — `modeState.value` est trackable par `$effect`. */
export const modeState = {
	get value(): Mode {
		return _mode;
	},
	set value(v: Mode) {
		_mode = v;
	},
};

export function toggleMode() {
	_mode = _mode === 'guided' ? 'expert' : 'guided';
}
