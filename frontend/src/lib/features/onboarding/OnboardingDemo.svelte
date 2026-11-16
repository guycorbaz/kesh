<script lang="ts">
	/**
	 * Story 36-17 (#1187, D7) — le chemin « données de démonstration ».
	 *
	 * - `demo-loading` — « Chargement des données de démonstration… » sur toute la zone,
	 *   pendant `seedDemo()` (`role="status"`) ;
	 * - `demo-ready` — l'état « démo, étape ≥ 3 » : les données sont prêtes, *Commencer*
	 *   (même lien qu'à la page terminée, chargement complet). La page navigue d'elle-même
	 *   vers `/` ; si cette navigation échoue, son motif s'affiche ici.
	 *
	 * Avant la story, l'état « étape 3, démo » rendait l'écran du TYPE D'ORGANISATION du
	 * chemin production, jusqu'à ce que la navigation aboutisse — ou indéfiniment.
	 *
	 * Présentationnel : la navigation et son `catch` vivent dans `+page.svelte`.
	 */
	import { Button } from '$lib/components/ui/button';
	import { i18nMsg } from '$lib/shared/utils/i18n.svelte';

	let {
		mode,
		motif = null,
	}: {
		mode: 'demo-loading' | 'demo-ready';
		/** Le motif de l'échec de la navigation vers `/`. */
		motif?: string | null;
	} = $props();
</script>

{#if mode === 'demo-loading'}
	<div
		class="flex justify-center p-8"
		role="status"
		aria-busy="true"
		data-testid="onboarding-screen-demo-loading"
	>
		<p class="text-lg font-medium">
			{i18nMsg('onboarding-demo-loading', 'Chargement des données de démonstration…')}
		</p>
	</div>
{:else}
	<div class="flex flex-col items-center gap-4 p-8 text-center" data-testid="onboarding-screen-demo-ready">
		<p class="text-lg font-medium">
			{i18nMsg('onboarding-demo-ready', 'Les données de démonstration sont prêtes.')}
		</p>
		{#if motif}
			<p class="text-sm" role="alert" data-testid="onboarding-demo-motif">{motif}</p>
		{/if}
		<Button href="/" data-sveltekit-reload data-testid="onboarding-demo-start">
			{i18nMsg('onboarding-start', 'Commencer')}
		</Button>
	</div>
{/if}
