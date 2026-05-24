<script lang="ts">
	import { apiHealth } from '$lib/shared/utils/api-health.svelte';
	import { i18nMsg } from '$lib/shared/utils/i18n.svelte';
</script>

{#if apiHealth.isDegraded}
	<div
		class="flex items-center justify-between bg-yellow-100 px-4 py-2 text-sm text-yellow-900"
		role="status"
		aria-live="polite"
		data-testid="degraded-banner"
	>
		<span>
			{i18nMsg(
				'db-unavailable-banner',
				'Base de données temporairement indisponible — réessai automatique en cours'
			)}
		</span>
	</div>
{/if}
