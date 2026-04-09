<script lang="ts">
	import { i18nMsg } from '$lib/features/onboarding/onboarding.svelte';

	function msg(key: string, fallback: string): string {
		return i18nMsg(key, fallback);
	}
</script>

<div
	class="flex items-center justify-between bg-blue-100 px-4 py-2 text-sm text-blue-900"
	role="status"
>
	<span>{msg('incomplete-banner-text', 'Configuration incomplète — Terminer la configuration')}</span>
	<a
		href="/onboarding"
		class="rounded px-3 py-1 text-sm font-medium text-blue-700 underline hover:text-blue-900"
	>
		{msg('incomplete-banner-cta', 'Terminer la configuration')}
	</a>
</div>
