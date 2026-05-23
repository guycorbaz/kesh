<script lang="ts">
	import '../app.css';
	import { onMount } from 'svelte';
	import favicon from '$lib/assets/favicon.svg';
	import { modeState } from '$lib/app/stores/mode.svelte';
	import DegradedBanner from '$lib/shared/components/DegradedBanner.svelte';
	import { apiHealth } from '$lib/shared/utils/api-health.svelte';
	import { Toaster } from 'svelte-sonner';

	let { children } = $props();

	// Appliquer data-mode sur <html> pour activer les CSS custom properties
	// du mode Guidé/Expert. Safe car ssr = false (pas de SSR).
	// modeState.value est un getter réactif — $effect re-exécutera quand le mode change.
	$effect(() => {
		document.documentElement.setAttribute('data-mode', modeState.value);
	});

	// Story 10.3 (AC #14bis) : ping /health initial au boot. Si la DB est down
	// dès le 1er chargement (ex. /login après reboot NAS avec DB non encore
	// prête), bascule en état dégradé pour afficher immédiatement le banner —
	// sans attendre qu'un fetch API spontané échoue. Wrappé en try/catch
	// silencieux : un échec ici signifie "DB inaccessible" → setDegraded() qui
	// démarrera le polling périodique de recovery.
	//
	// AC #14ter : `AbortSignal.timeout(2000)` borne le fetch à 2s. Sans ce
	// timeout, un kesh-api qui accepte TCP mais hang HTTP (NAS sous OOM /
	// MariaDB lock) bloque indéfiniment la Promise — le catch ne fire jamais,
	// `setDegraded()` n'est pas appelé, l'utilisateur stare devant une UI sans
	// indication jusqu'au 1er fetch lazy qui timeout à 30s.
	onMount(async () => {
		try {
			const res = await fetch('/health', { signal: AbortSignal.timeout(2000) });
			if (!res.ok) {
				apiHealth.setDegraded();
				return;
			}
			const body = (await res.json()) as { db?: unknown };
			if (body.db !== true) {
				apiHealth.setDegraded();
			}
		} catch {
			apiHealth.setDegraded();
		}
	});
</script>

<svelte:head>
	<link rel="icon" href={favicon} />
</svelte:head>

<DegradedBanner />

{@render children()}
<Toaster theme="light" />
