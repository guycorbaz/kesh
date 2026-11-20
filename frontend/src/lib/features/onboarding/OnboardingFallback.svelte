<script lang="ts">
	/**
	 * Story 36-17 (#1187, D8) — l'écran de repli : un état de l'assistant qu'aucun autre
	 * écran ne couvre (étape négative ou non entière). Plus aucune combinaison ne rend une
	 * page vide ; *Recharger la page* relit l'état par un chargement complet.
	 */
	import { i18nMsg } from '$lib/shared/utils/i18n.svelte';
</script>

<div class="flex flex-col items-center gap-4 p-8 text-center" data-testid="onboarding-screen-fallback">
	<p class="font-medium" role="alert">
		{i18nMsg('onboarding-fallback', "État de l'assistant inattendu.")}
	</p>
	<a
		href="/onboarding"
		data-sveltekit-reload
		class="font-medium text-primary underline hover:no-underline"
		data-testid="onboarding-fallback-reload"
	>
		{i18nMsg('onboarding-fallback-reload', 'Recharger la page')}
	</a>
</div>
