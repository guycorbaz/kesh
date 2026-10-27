<!--
  Story 32-8c (refs #723) — /admin/mcp/sends/[id] : le détail d'un envoi du
  serveur MCP. Page mince : `McpSendDetail` porte la logique ; `{#key}` le
  remonte quand le numéro change. Un numéro illisible est rendu « introuvable »
  par le composant, sans appel.
-->
<script lang="ts">
	import { page } from '$app/state';
	import McpSendDetail from '$lib/features/mcp-sends/McpSendDetail.svelte';

	const id = $derived(Number(page.params.id));
</script>

{#key id}
	<McpSendDetail {id} />
{/key}
