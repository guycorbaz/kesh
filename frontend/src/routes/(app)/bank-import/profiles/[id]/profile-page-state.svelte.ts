// Story 29-9 (#655, AC6) — doublure de `page` (`$app/state`) pour `profile-page.test.ts`, qui
// imite SvelteKit : `params` est un `$state.raw`, et une navigation le RÉASSIGNE (objet neuf),
// comme `client.js` le fait dès que `page_changed` est vrai — même à `id` égal. Ne jamais écrire
// `fakePage.params.id = …` : SvelteKit ne le fait pas, et un `$state.raw` ne le verrait pas.

class FakePage {
	params = $state.raw<Record<string, string>>({ id: '5' });
}

export const fakePage = new FakePage();
