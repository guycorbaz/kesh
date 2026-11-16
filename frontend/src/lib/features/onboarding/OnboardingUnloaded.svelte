<script lang="ts">
	/**
	 * Story 36-17 (#1187, D9) — l'état de l'assistant n'a pas pu être lu.
	 *
	 * La garde de `/onboarding` journalise l'échec de la lecture sans bloquer ; la page
	 * rendait alors l'écran de l'étape 0 (la langue) pour une installation peut-être à
	 * l'étape 7. Cet écran le dit, avec *Réessayer*. Pendant une lecture, il affiche
	 * « Chargement… » (c'est aussi l'écran du chargement initial).
	 *
	 * Présentationnel : la relecture et son `catch` vivent dans `+page.svelte`.
	 */
	import { Button } from '$lib/components/ui/button';
	import { i18nMsg } from '$lib/shared/utils/i18n.svelte';

	let {
		loading,
		motif,
		onRetry,
	}: {
		/** Une lecture de l'état est en cours. */
		loading: boolean;
		/** Le motif du dernier échec de *Réessayer*, s'il en dit plus que le constat. */
		motif: string | null;
		onRetry: () => void;
	} = $props();
</script>

{#if loading}
	<div
		class="flex justify-center p-8"
		role="status"
		aria-busy="true"
		data-testid="onboarding-screen-unloaded"
	>
		<p class="text-text-muted">{i18nMsg('common-loading', 'Chargement…')}</p>
	</div>
{:else}
	<div class="flex flex-col items-center gap-4 p-8 text-center" data-testid="onboarding-screen-unloaded">
		<div role="alert">
			<p class="font-medium">
				{i18nMsg('onboarding-state-load-error', "L'état de la configuration n'a pas pu être lu.")}
			</p>
			{#if motif}
				<p class="mt-1 text-sm text-text-muted" data-testid="onboarding-unloaded-motif">{motif}</p>
			{/if}
		</div>
		<Button type="button" onclick={onRetry} data-testid="onboarding-unloaded-retry">
			{i18nMsg('onboarding-retry', 'Réessayer')}
		</Button>
	</div>
{/if}
