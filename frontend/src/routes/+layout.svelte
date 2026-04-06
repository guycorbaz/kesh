<script lang="ts">
	import '../app.css';
	import favicon from '$lib/assets/favicon.svg';
	import { modeState } from '$lib/app/stores/mode.svelte';
	import { Toaster } from 'svelte-sonner';

	let { children } = $props();

	// Appliquer data-mode sur <html> pour activer les CSS custom properties
	// du mode Guidé/Expert. Safe car ssr = false (pas de SSR).
	// modeState.value est un getter réactif — $effect re-exécutera quand le mode change.
	$effect(() => {
		document.documentElement.setAttribute('data-mode', modeState.value);
	});
</script>

<svelte:head>
	<link rel="icon" href={favicon} />
</svelte:head>

{@render children()}
<Toaster theme="light" />
