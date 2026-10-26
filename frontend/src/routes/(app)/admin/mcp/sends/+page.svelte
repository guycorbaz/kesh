<!--
  Story 32-8c (refs #723) — /admin/mcp/sends : la liste des envois du serveur
  MCP. Page mince : la logique et l'UI vivent dans `McpSendsList`.
  L'entrée de menu est réservée à l'Administrateur ; la page, atteinte par un
  autre rôle, montre le refus de l'API (403), qui reste l'autorité (AC-C1).
-->
<script lang="ts">
	import McpSendsList from '$lib/features/mcp-sends/McpSendsList.svelte';
</script>

<McpSendsList />
