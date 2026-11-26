/**
 * Store du nom de la comptabilité ouverte (Svelte 5 runes) — #1582.
 *
 * Le nom est `company.name`, tel que les Réglages l'enregistrent et que le PDF
 * l'imprime (jamais une recomposition de `firstName` et `lastName`, D1). Il est lu
 * par le layout `(app)` au montage (`load`), mis à jour par les Réglages quand la
 * société est adoptée (`set`, D8) et vidé au démontage (`clear`, AC8).
 *
 * Un échec de lecture n'abîme rien : le nom reste absent et l'erreur va à la
 * console (D9). Une lecture tardive ne l'emporte jamais sur un `set()` ou un
 * `clear()` survenu pendant son vol (numéro de séquence).
 */
import { fetchCompanyCurrent } from '$lib/features/settings/settings.api';

let _name = $state<string | null>(null);
// ⚠️ Un `let` ordinaire, volontairement pas un `$state` : un `$effect` qui appelle `load()`
// dépendrait de ce que `load()` écrit et se relancerait (`effect_update_depth_exceeded`).
let seq = 0;

/** Nom rogné ; vide ou non textuel → `null` (D2). */
function normalize(value: unknown): string | null {
	if (typeof value !== 'string') return null;
	const trimmed = value.trim();
	return trimmed === '' ? null : trimmed;
}

export const companyIdentity = {
	/** Nom de la comptabilité ouverte, ou `null` (inconnu, en lecture, vide ou en échec). */
	get name(): string | null {
		return _name;
	},

	/**
	 * Lit la société courante. Remet d'abord le nom à `null`, de façon synchrone : pendant
	 * la lecture, ou après son échec, rien ne s'affiche plutôt qu'un nom périmé (AC8).
	 * Ne propage jamais d'erreur (D9).
	 */
	async load(): Promise<void> {
		const mine = ++seq;
		_name = null;
		try {
			const response = await fetchCompanyCurrent();
			if (mine !== seq) return;
			_name = normalize(response.company.name);
		} catch (err) {
			if (mine !== seq) return;
			_name = null;
			console.error('company identity: lecture de la société impossible', err);
		}
	},

	/** Pose le nom (société adoptée par les Réglages, D8). Une lecture en vol devient sans effet. */
	set(name: string): void {
		seq++;
		_name = normalize(name);
	},

	/** Vide le nom (démontage du layout). Une lecture en vol devient sans effet. */
	clear(): void {
		seq++;
		_name = null;
	}
};
