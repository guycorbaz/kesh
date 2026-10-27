<!--
  Story 32-10b (refs #723) — /admin/mcp/prompts/[id] : le détail d'un prompt du
  serveur MCP et de tout son historique. Page mince : `McpPromptDetail` porte
  la logique ; `{#key}` le remonte quand le numéro change. Un numéro illisible
  est rendu « introuvable » par le composant, sans appel.
-->
<script lang="ts">
	import { page } from '$app/state';
	import McpPromptDetail from '$lib/features/mcp-prompts/McpPromptDetail.svelte';

	const id = $derived(Number(page.params.id));
</script>

{#key id}
	<McpPromptDetail {id} />
{/key}
