<script lang="ts">
	import { onboardingState, i18nMsg } from '$lib/features/onboarding/onboarding.svelte';
	import { appVersion } from '$lib/shared/utils/app-version.svelte';

	let { children } = $props();
</script>

<div class="flex min-h-screen flex-col items-center justify-center bg-surface">
	<!-- Logo -->
	<div class="mb-8">
		<a href="/" class="text-3xl font-semibold text-primary">Kesh</a>
	</div>

	<!-- Contenu wizard -->
	<div class="w-full max-w-lg">
		{#if onboardingState.isStub}
			<!-- Nudge non-bloquant (Story v011-2, Issue #120) : la company a été créée
			     en placeholder par le bootstrap (DB vide). Disparaît dès que l'utilisateur
			     renseigne ses coordonnées (set_coordinates repasse is_stub=FALSE). -->
			<div
				role="status"
				data-testid="onboarding-stub-notice"
				class="mb-6 rounded-md border border-primary/30 bg-primary-light/10 px-4 py-3 text-sm text-text"
			>
				{i18nMsg(
					'onboarding-stub-name-notice',
					'Votre entreprise a un nom provisoire — complétez vos coordonnées'
				)}
			</div>
		{/if}
		{@render children()}
	</div>

	<!-- Footer disclaimer FR7 -->
	<footer class="mt-8 text-center text-xs text-text-muted">
		Kesh v{appVersion.value} &mdash; Logiciel libre (EUPL 1.2). Les donn&eacute;es ne remplacent pas un fiduciaire.
	</footer>
</div>
