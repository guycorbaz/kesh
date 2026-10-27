<!--
  Story 32-10b (refs #723) — le rendu d'une version : le texte servi, tel que
  le serveur l'a rendu visible (AC-5), et son verdict (AC-6).

  ⛔ **Ce qui est affiché est le champ rendu visible par le serveur, et rien
  d'autre.** `descriptionServie.visible` et `texteServi.visible` sont ce que
  `prompts/get` servirait si la version était en vigueur — marque d'un prompt
  d'agent comprise, en tête, sans champ séparé —, chaque caractère caché que
  Kesh refuse écrit `\u{XXXX}` par `rendre_visible`. L'écran n'a AUCUNE logique
  Unicode : il interpole le champ, sans découpe, sans encadré, sans `trim`.
  Le texte EXACT n'est jamais affiché.

  ⛔ **Règle des blancs de gabarit** : chaque `<pre>` s'écrit d'un tenant, sur
  une ligne — un `<pre>` affiche les blancs du gabarit comme du texte. Ne pas
  reformater ces lignes.

  ⛔ **Règle des `data-testid`** : une même version peut être montée dans
  plusieurs zones de la page ; chaque `data-testid` est préfixé par la `zone`
  reçue (`servi-`, `non-servi-`, `a-valider-`, `historique-`).

  Une version non servable : le serveur ne compose aucun texte servi
  (`texteServi` nul) ; l'avis du verdict, puis le texte ENREGISTRÉ rendu
  visible par le serveur (`texte.visible`), sous des `data-testid` `stored-`
  (C-32-10b-21).
-->
<script lang="ts">
	import { i18nMsg } from '$lib/shared/utils/i18n.svelte';
	import type { McpPromptVersion } from './mcp-prompts.types';

	type Props = { version: McpPromptVersion; zone: string };
	let { version, zone }: Props = $props();

	const servi = $derived(
		version.servable && version.descriptionServie !== null && version.texteServi !== null
			? { description: version.descriptionServie, texte: version.texteServi }
			: null,
	);
</script>

<div class="space-y-2">
	{#if version.origine === 'agent'}
		<span
			class="inline-flex items-center rounded-full px-2 py-0.5 text-xs font-medium pastille-agent"
			data-testid="{zone}mcp-prompt-agent-badge-{version.numero}"
		>
			{version.etat === 'pending'
				? i18nMsg('mcp-prompts-badge-agent-pending', 'Écrit par un agent — à valider')
				: i18nMsg('mcp-prompts-badge-agent', 'Écrit par un agent')}
		</span>
	{/if}

	{#if version.nature === 'withdrawal'}
		<p class="text-sm text-text-muted" data-testid="{zone}mcp-prompt-withdrawal-{version.numero}">
			{i18nMsg('mcp-prompts-withdrawal-body', 'Retrait : cette version ne porte ni description ni texte.')}
		</p>
	{:else if servi}
		<div class="space-y-1">
			<p class="text-xs font-medium text-text-muted">{i18nMsg('mcp-prompts-field-description', 'Description')}</p>
			<pre class="whitespace-pre-wrap break-words rounded-md border border-border bg-surface-alt px-3 py-2 font-mono text-sm" dir="ltr" data-testid="{zone}mcp-prompt-served-description-{version.numero}">{servi.description.visible}</pre>
		</div>
		<div class="space-y-1">
			<p class="text-xs font-medium text-text-muted">{i18nMsg('mcp-prompts-field-text', 'Texte')}</p>
			<pre class="whitespace-pre-wrap break-words rounded-md border border-border bg-surface-alt px-3 py-2 font-mono text-sm" dir="ltr" data-testid="{zone}mcp-prompt-served-text-{version.numero}">{servi.texte.visible}</pre>
		</div>
	{:else}
		<div
			class="rounded-md border border-destructive bg-destructive/10 px-3 py-2 text-sm text-destructive"
			role="alert"
			data-testid="{zone}mcp-prompt-not-servable-{version.numero}"
		>
			{i18nMsg(
				'mcp-prompts-not-servable',
				'Kesh ne servirait pas cette version : son contenu n’est pas valide (caractère invisible, longueur, contenu vide ou description sur plusieurs lignes). Elle a pu être restaurée d’une sauvegarde ou écrite hors de Kesh.',
			)}
		</div>
		{#if version.description !== null && version.texte !== null}
			<div class="space-y-1">
				<p class="text-xs font-medium text-text-muted">{i18nMsg('mcp-prompts-field-description', 'Description')}</p>
				<pre class="whitespace-pre-wrap break-words rounded-md border border-border bg-surface-alt px-3 py-2 font-mono text-sm" dir="ltr" data-testid="{zone}mcp-prompt-stored-description-{version.numero}">{version.description.visible}</pre>
			</div>
			<div class="space-y-1">
				<p class="text-xs font-medium text-text-muted">{i18nMsg('mcp-prompts-field-text', 'Texte')}</p>
				<pre class="whitespace-pre-wrap break-words rounded-md border border-border bg-surface-alt px-3 py-2 font-mono text-sm" dir="ltr" data-testid="{zone}mcp-prompt-stored-text-{version.numero}">{version.texte.visible}</pre>
			</div>
		{/if}
	{/if}
</div>

<style>
	.pastille-agent {
		background-color: color-mix(in srgb, var(--color-warning) 22%, transparent);
		color: var(--color-text);
	}
</style>
