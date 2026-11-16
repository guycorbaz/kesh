<script lang="ts">
	import { i18nMsg } from '$lib/features/onboarding/onboarding.svelte';

	function msg(key: string, fallback: string): string {
		return i18nMsg(key, fallback);
	}
</script>

<div
	class="flex items-center justify-between bg-blue-100 px-4 py-2 text-sm text-blue-900"
	role="status"
	data-testid="incomplete-config-banner"
>
	<!-- Story 36-17 (#1187, D5) : terminer AVANT d'utiliser Kesh — sans exercice ni réglages
	     de facturation, Kesh n'est pas prêt. Même phrase que l'écran de reprise de l'assistant. -->
	<span>{msg('incomplete-banner-text', "Configuration incomplète : terminez-la avant d'utiliser Kesh.")}</span>
	<a
		href="/onboarding"
		class="rounded px-3 py-1 text-sm font-medium text-blue-700 underline hover:text-blue-900"
	>
		{msg('incomplete-banner-cta', 'Terminer la configuration')}
	</a>
</div>
