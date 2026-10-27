<!--
  Story 32-7 (refs #723) — l'écran « Serveur MCP » (Administration → Serveur MCP).

  Deux sections : les réglages du serveur (activation, filtre d'adresse) et les clés
  MCP. Les trois écrans du serveur MCP — celui-ci, les envois (32-8c) et les prompts
  (32-10b) — se tiennent ensemble par le menu et par les liens ci-dessous.
-->
<script lang="ts">
	import { i18nMsg } from '$lib/shared/utils/i18n.svelte';
	import McpSettingsSection from './McpSettingsSection.svelte';
	import McpKeysSection from './McpKeysSection.svelte';
</script>

<div class="space-y-4">
	<h1 class="text-2xl font-semibold" data-testid="mcp-server-title">{i18nMsg('mcp-server-title', 'Serveur MCP')}</h1>
	<p class="text-sm text-text-muted">
		{i18nMsg(
			'mcp-server-intro',
			'Le serveur MCP permet à un agent — un client MCP comme Claude Code — de lire les livres de cette société avec une clé MCP. Il est désactivé par défaut ; aucune portée n’écrit dans les livres.',
		)}
	</p>
	<nav class="flex flex-wrap gap-4 text-sm" data-testid="mcp-server-nav">
		<a class="text-primary underline" href="/admin/mcp/sends" data-testid="mcp-server-nav-sends">
			{i18nMsg('mcp-server-nav-sends', 'Envois du serveur MCP')}
		</a>
		<a class="text-primary underline" href="/admin/mcp/prompts" data-testid="mcp-server-nav-prompts">
			{i18nMsg('mcp-server-nav-prompts', 'Prompts du serveur MCP')}
		</a>
	</nav>
	<McpSettingsSection />
	<McpKeysSection />
</div>
