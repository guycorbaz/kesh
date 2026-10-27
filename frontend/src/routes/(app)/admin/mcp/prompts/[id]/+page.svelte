<!--
  Story 32-10b (refs #723) — /admin/mcp/prompts/[id] : le détail d'un prompt du
  serveur MCP et de tout son historique. Page mince : `McpPromptDetail` porte
  la logique ; `{#key}` le remonte quand le numéro change. Un numéro illisible
  est rendu « introuvable » par le composant, sans appel ; seuls des chiffres
  décimaux sont lus (`idDeLUrl`, revue de code P1, E9).
-->
<script lang="ts">
	import { page } from '$app/state';
	import McpPromptDetail from '$lib/features/mcp-prompts/McpPromptDetail.svelte';
	import { idDeLUrl } from '$lib/features/mcp-prompts/mcp-prompts-format';

	const id = $derived(idDeLUrl(page.params.id ?? ''));
</script>

{#key id}
	<McpPromptDetail {id} />
{/key}
