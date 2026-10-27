<!--
  Story 32-10b (refs #723) — /admin/mcp/prompts : la liste des prompts du
  serveur MCP et la création d'un prompt. Page mince : la logique et l'UI
  vivent dans `McpPromptsList`. L'entrée de menu est réservée à
  l'Administrateur ; la page, atteinte par un autre rôle, montre le refus de
  l'API (403), qui reste l'autorité (AC-1, C-32-10b-9).
-->
<script lang="ts">
	import McpPromptsList from '$lib/features/mcp-prompts/McpPromptsList.svelte';
</script>

<McpPromptsList />
