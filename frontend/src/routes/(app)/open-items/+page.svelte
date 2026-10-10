<script lang="ts">
	// Story 15-1c-i (#518) — route de l'écran des postes ouverts. Tout l'écran
	// vit dans `features/open-items/OpenItemsScreen.svelte` ; la route lui donne
	// l'URL courante et la manière de l'écrire (remplacement d'historique).
	import { page } from '$app/state';
	import { goto } from '$app/navigation';
	import { i18nMsg } from '$lib/shared/utils/i18n.svelte';
	import OpenItemsScreen from '$lib/features/open-items/OpenItemsScreen.svelte';

	function navigate(url: URL) {
		void goto(url, { replaceState: true, keepFocus: true, noScroll: true });
	}
</script>

<svelte:head>
	<title>{i18nMsg('nav-open-items', 'Postes ouverts')} — Kesh</title>
</svelte:head>

<OpenItemsScreen url={page.url} {navigate} />
