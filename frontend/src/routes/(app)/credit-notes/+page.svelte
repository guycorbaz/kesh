<script lang="ts">
	import { onMount } from 'svelte';
	import { goto } from '$app/navigation';
	import { listCreditNotes } from '$lib/features/credit-notes/credit-notes.api';
	import {
		creditNoteStatusLabel,
		formatCreditNoteTotal,
	} from '$lib/features/credit-notes/credit-note-helpers';
	import type { CreditNoteListItemResponse } from '$lib/features/credit-notes/credit-notes.types';
	import { isApiError } from '$lib/shared/utils/api-client';
	import { i18nMsg } from '$lib/shared/utils/i18n.svelte';

	let items = $state<CreditNoteListItemResponse[]>([]);
	let loading = $state(true);
	let errorMsg = $state('');

	onMount(async () => {
		try {
			const res = await listCreditNotes({ limit: 100 });
			items = res.items;
		} catch (err) {
			if (isApiError(err)) errorMsg = err.message;
		} finally {
			loading = false;
		}
	});
</script>

<svelte:head>
	<title>{i18nMsg('credit-notes-title', 'Avoirs')} — Kesh</title>
</svelte:head>

<h1 class="mb-6 text-2xl font-semibold">{i18nMsg('credit-notes-title', 'Avoirs')}</h1>

{#if loading}
	<p class="text-sm text-text-muted">Chargement…</p>
{:else if errorMsg}
	<p class="text-sm text-destructive">{errorMsg}</p>
{:else if items.length === 0}
	<p class="text-sm text-text-muted">
		{i18nMsg(
			'credit-notes-empty',
			'Aucun avoir. Créez un avoir depuis une facture validée pour l’annuler.',
		)}
	</p>
{:else}
	<table class="w-full border-collapse text-sm" data-testid="credit-notes-table">
		<thead>
			<tr class="border-b text-left text-text-muted">
				<th class="py-2">{i18nMsg('credit-notes-col-number', 'N° d’avoir')}</th>
				<th class="py-2">{i18nMsg('credit-notes-col-date', 'Date')}</th>
				<th class="py-2">{i18nMsg('credit-notes-col-status', 'Statut')}</th>
				<th class="py-2 text-right">{i18nMsg('credit-notes-col-total', 'Total HT')}</th>
			</tr>
		</thead>
		<tbody>
			{#each items as cn (cn.id)}
				<tr
					class="cursor-pointer border-b hover:bg-surface-hover"
					data-testid="credit-note-row"
					onclick={() => goto(`/credit-notes/${cn.id}`)}
				>
					<td class="py-2 font-mono">{cn.creditNoteNumber ?? `#${cn.id}`}</td>
					<td class="py-2">{cn.date}</td>
					<td class="py-2">{creditNoteStatusLabel(cn.status)}</td>
					<td class="py-2 text-right">{formatCreditNoteTotal(cn.totalAmount)}</td>
				</tr>
			{/each}
		</tbody>
	</table>
{/if}
