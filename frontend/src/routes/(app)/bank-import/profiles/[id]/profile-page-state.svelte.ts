// Story 29-9 (#655, AC6) — doublure de `page` (`$app/state`) pour `profile-page.test.ts`, qui
// imite SvelteKit : `params` est un `$state.raw`, et une navigation le RÉASSIGNE (objet neuf),
// comme `client.js` le fait dès que `page_changed` est vrai — même à `id` égal. Ne jamais écrire
// `fakePage.params.id = …` : SvelteKit ne le fait pas, et un `$state.raw` ne le verrait pas.
//
// Un objet à accesseurs, et non une classe : `i18n-libelle-en-dur.test.ts` tient pour prémisse
// qu'aucune classe n'existe dans `src/` (C-29-9-27).

let params = $state.raw<Record<string, string>>({ id: '5' });

export const fakePage = {
	get params(): Record<string, string> {
		return params;
	},
	set params(value: Record<string, string>) {
		params = value;
	},
};
