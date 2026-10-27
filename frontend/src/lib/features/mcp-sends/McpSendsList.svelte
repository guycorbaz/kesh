<!--
  Story 32-8c (refs #723) — la liste des envois du serveur MCP (AC-C2).

  Un tableau : n°, clé (« révoquée » le cas échéant), outil, demandé le,
  échéance, état, lien vers le détail. L'ordre est celui du serveur (en
  attente d'abord). **Aucun contenu** : l'API de liste ne le rend pas.
  Un Administrateur seulement : pour tout autre rôle, l'API répond 403 et la
  page montre ce refus.
-->
<script lang="ts">
	import { onMount } from 'svelte';
	import { Button } from '$lib/components/ui/button';
	import { i18nMsg } from '$lib/shared/utils/i18n.svelte';
	import { errorMessageOf } from '$lib/shared/utils/error-message';
	import { listMcpSends } from './mcp-sends.api';
	import type { McpSendSummary } from './mcp-sends.types';
	import { dateLocale, etatAffiche } from './mcp-sends-format';

	let envois = $state<McpSendSummary[]>([]);
	let chargement = $state(true);
	let erreur = $state<string | null>(null);

	async function charger() {
		chargement = true;
		erreur = null;
		try {
			envois = await listMcpSends();
		} catch (err) {
			erreur = errorMessageOf(err);
		} finally {
			chargement = false;
		}
	}

	onMount(() => {
		void charger();
	});
</script>

<div class="space-y-4">
	<div class="flex flex-wrap items-center justify-between gap-2">
		<h1 class="text-2xl font-semibold" data-testid="mcp-sends-title">
			{i18nMsg('mcp-sends-title', 'Envois du serveur MCP')}
		</h1>
		<Button variant="outline" onclick={charger} disabled={chargement} data-testid="mcp-sends-refresh">
			{i18nMsg('mcp-sends-refresh', 'Actualiser')}
		</Button>
	</div>

	<p class="text-sm text-text-muted">
		{i18nMsg(
			'mcp-sends-intro',
			"Sous une clé du serveur MCP dont la validation est requise, ce qu'un agent demande attend ici votre décision : rien ne lui parvient avant que vous l'ayez validé.",
		)}
	</p>

	{#if erreur}
		<div
			class="rounded-md border border-destructive bg-destructive/10 px-3 py-2 text-sm text-destructive"
			role="alert"
			data-testid="mcp-sends-error"
		>
			{erreur}
		</div>
	{:else if chargement && envois.length === 0}
		<p class="text-sm text-text-muted">{i18nMsg('common-loading', 'Chargement…')}</p>
	{:else if envois.length === 0}
		<p class="text-sm text-text-muted" data-testid="mcp-sends-empty">
			{i18nMsg('mcp-sends-empty', 'Aucun envoi')}
		</p>
	{:else}
		<div class="overflow-x-auto">
			<table class="w-full text-sm" data-testid="mcp-sends-table">
				<thead>
					<tr class="border-b border-border text-left">
						<th class="py-2 pr-4">{i18nMsg('mcp-sends-col-number', 'N°')}</th>
						<th class="py-2 pr-4">{i18nMsg('mcp-sends-col-key', 'Clé')}</th>
						<th class="py-2 pr-4">{i18nMsg('mcp-sends-col-tool', 'Outil')}</th>
						<th class="py-2 pr-4">{i18nMsg('mcp-sends-col-created', 'Demandé le')}</th>
						<th class="py-2 pr-4">{i18nMsg('mcp-sends-col-expires', 'Échéance')}</th>
						<th class="py-2 pr-4">{i18nMsg('mcp-sends-col-state', 'État')}</th>
						<th class="py-2"></th>
					</tr>
				</thead>
				<tbody>
					{#each envois as e (e.id)}
						<tr class="border-b border-border" data-testid="mcp-sends-row-{e.id}">
							<td class="py-2 pr-4 font-mono">{e.id}</td>
							<td class="py-2 pr-4">
								{e.mcpKeyName}
								{#if e.mcpKeyRevoked}
									<span class="text-xs text-text-muted">
										({i18nMsg('mcp-sends-key-revoked', 'révoquée')})
									</span>
								{/if}
							</td>
							<td class="py-2 pr-4 font-mono text-xs">{e.toolName}</td>
							<td class="py-2 pr-4 whitespace-nowrap">{dateLocale(e.createdAt)}</td>
							<td class="py-2 pr-4 whitespace-nowrap">{dateLocale(e.expiresAt)}</td>
							<td class="py-2 pr-4">
								<span
									class="etat etat-{e.state} inline-flex items-center rounded-full px-2 py-0.5 text-xs font-medium"
									data-testid="mcp-sends-state-{e.id}"
								>
									{etatAffiche(e.state, e.modified)}
								</span>
							</td>
							<td class="py-2">
								<a
									class="text-primary underline"
									href="/admin/mcp/sends/{e.id}"
									data-testid="mcp-sends-open-{e.id}"
								>
									{i18nMsg('mcp-sends-open', 'Voir')}
								</a>
							</td>
						</tr>
					{/each}
				</tbody>
			</table>
		</div>
	{/if}
</div>

<style>
	.etat {
		background-color: color-mix(in srgb, var(--color-text-muted) 18%, transparent);
		color: var(--color-text);
	}
	.etat-pending {
		background-color: color-mix(in srgb, var(--color-warning) 22%, transparent);
	}
	.etat-approved {
		background-color: color-mix(in srgb, var(--color-success) 20%, transparent);
	}
	.etat-refused {
		background-color: color-mix(in srgb, var(--color-destructive) 16%, transparent);
	}
</style>
