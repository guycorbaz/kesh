<script lang="ts">
	// Story 25-6-a (#388) — la tuile « Dernières écritures », branchée. Elle
	// affichait « Aucune écriture » quel que soit le contenu de la base.
	// ⚠️ Un échec de chargement se DIT : prétendre qu'il n'y a rien est le défaut
	// même de #388. Il prime sur la variante guidée.
	import { Button } from '$lib/components/ui/button';
	import { i18nMsg } from '$lib/shared/utils/i18n.svelte';
	import { formatSwissAmount } from '$lib/features/journal-entries/balance';
	import { formatSwissDate } from '$lib/features/reports/reports.api';
	import type { JournalEntryResponse } from '$lib/features/journal-entries/journal-entries.types';
	import { entryAmount } from './homepage';

	interface Props {
		state: 'loading' | 'ready' | 'error';
		entries: JournalEntryResponse[];
		/** Admin ou Comptable : seuls ces rôles saisissent. */
		canManage: boolean;
		isGuided: boolean;
	}
	let { state, entries, canManage, isGuided }: Props = $props();
</script>

<div class="rounded-lg border border-border bg-white p-6 shadow-sm" data-testid="homepage-card-recent-entries">
	<h2 class="text-lg font-semibold text-text">
		{i18nMsg('homepage-entries-title', 'Dernières écritures')}
	</h2>
	{#if state === 'error'}
		<p class="mt-2 text-sm text-text-muted" data-testid="homepage-entries-unavailable">
			{i18nMsg('homepage-entries-unavailable', 'Écritures indisponibles pour le moment.')}
		</p>
	{:else if state === 'ready' && entries.length > 0}
		<ul class="mt-3 flex flex-col gap-2">
			{#each entries as entry (entry.id)}
				<li class="flex items-baseline justify-between gap-3 text-sm" data-testid="homepage-entry-row">
					<a class="min-w-0 flex-1 truncate text-primary underline" href="/journal-entries/{entry.id}">
						<span class="font-mono text-xs text-text-muted">{formatSwissDate(entry.entryDate)} · n° {entry.entryNumber}</span>
						{entry.description}
					</a>
					<span class="font-mono">{formatSwissAmount(entryAmount(entry))}</span>
				</li>
			{/each}
		</ul>
	{:else if state === 'ready'}
		<p class="mt-2 text-sm text-text-muted">
			{#if isGuided && canManage}
				{i18nMsg(
					'homepage-entries-empty-guided',
					'Aucune écriture pour le moment. Commencez par saisir votre première écriture comptable.',
				)}
			{:else}
				{i18nMsg('homepage-entries-empty', 'Aucune écriture.')}
			{/if}
		</p>
	{/if}
	{#if canManage}
		<Button variant="outline" class="mt-4" href="/journal-entries">
			{i18nMsg('homepage-entries-action', 'Saisir une écriture')}
		</Button>
	{/if}
</div>
