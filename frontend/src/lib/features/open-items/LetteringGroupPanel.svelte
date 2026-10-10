<script lang="ts">
	// Story 15-1c-i (AC6, AC10) — un groupe de lettrage, lu par son code
	// (`GET /letterings/{code}`, réponse enrichie de la 15-1c-0), et son
	// délettrage.
	//
	// ⛔ « Délettrer » suit `manualDissolutionBlockedBy` (prévision **indicative**
	// du serveur, lue sans verrou) : aucun ordre de refus recopié ici. Bloqué, le
	// panneau dit le motif **au texte même** du refus du serveur (`error-lettering-*`).
	// Consultation : ni bouton, ni motif (patron C-15-8-14 de la fiche d'écriture).
	import { i18nMsg } from '$lib/shared/utils/i18n.svelte';
	import { formatSwissDate } from '$lib/features/reports/reports.api';
	import DocumentCell from './DocumentCell.svelte';
	import { groupDocument } from './open-items';
	import {
		dissolutionBlockerLabel,
		entryRefLabel,
		formatAmountString,
		originLabel,
	} from './open-items-labels';
	import type { GroupState } from './open-items.types';

	interface Props {
		group: GroupState;
		canWrite: boolean;
		busy: boolean;
		onDissolve: (code: string) => void;
		onClose: () => void;
	}
	let { group, canWrite, busy, onDissolve, onClose }: Props = $props();

	let doc = $derived(group.status === 'ready' ? groupDocument(group.data) : null);

	/** Un montant nul ne s'écrit pas. */
	function amountOrBlank(v: string): string {
		return Number(v) === 0 ? '' : formatAmountString(v);
	}
</script>

<section class="space-y-2 rounded border border-indigo-200 p-3" data-testid="lettering-group-panel">
	<div class="flex items-start justify-between gap-2">
		<h2 class="font-semibold">
			{i18nMsg('open-items-group-title', 'Groupe { $code }', {
				code: group.status === 'ready' ? group.data.code : group.code,
			})}
		</h2>
		<button
			type="button"
			class="rounded border px-2 py-0.5 text-xs"
			onclick={onClose}
			data-testid="lettering-group-close">{i18nMsg('open-items-group-close', 'Fermer')}</button
		>
	</div>

	{#if group.status === 'loading'}
		<p class="text-sm text-gray-500" role="status">{i18nMsg('common-loading', 'Chargement…')}</p>
	{:else if group.status === 'notFound'}
		<p class="text-sm" role="alert" data-testid="lettering-group-not-found">
			{i18nMsg('open-items-group-not-found', 'Aucun groupe ne porte ce code.')}
		</p>
	{:else if group.status === 'error'}
		<p class="text-sm text-red-700" role="alert" data-testid="lettering-group-error">
			{group.message}
		</p>
	{:else}
		{@const g = group.data}
		<p class="text-sm" data-testid="lettering-group-origin">
			{originLabel(g.origin, doc?.number ?? null)}
		</p>
		<p class="text-sm" data-testid="lettering-group-account">
			<span class="font-mono">{g.accountNumber}</span>
			{g.accountName}
		</p>
		<div class="overflow-x-auto">
			<table class="w-full border-collapse text-sm">
				<thead>
					<tr class="border-b text-left">
						<th class="px-2 py-1" scope="col">{i18nMsg('open-items-col-date', 'Date')}</th>
						<th class="px-2 py-1" scope="col">{i18nMsg('open-items-col-entry', 'Écriture')}</th>
						<th class="px-2 py-1" scope="col">{i18nMsg('open-items-col-journal', 'Journal')}</th>
						<th class="px-2 py-1" scope="col">
							{i18nMsg('open-items-col-description', 'Libellé')}
						</th>
						<th class="px-2 py-1" scope="col">{i18nMsg('open-items-col-document', 'Pièce')}</th>
						<th class="px-2 py-1 text-right" scope="col">
							{i18nMsg('open-items-col-debit', 'Débit')}
						</th>
						<th class="px-2 py-1 text-right" scope="col">
							{i18nMsg('open-items-col-credit', 'Crédit')}
						</th>
					</tr>
				</thead>
				<tbody>
					{#each g.lines as l (l.id)}
						<tr class="border-b" data-testid="lettering-group-line-{l.id}">
							<td class="px-2 py-1 whitespace-nowrap">{formatSwissDate(l.date)}</td>
							<td class="px-2 py-1 whitespace-nowrap">
								<a href="/journal-entries/{l.entryId}" class="text-primary underline"
									>{entryRefLabel(l.fiscalYearName, l.entryNumber)}</a
								>
							</td>
							<td class="px-2 py-1">{l.journal}</td>
							<td class="px-2 py-1">{l.description}</td>
							<td class="px-2 py-1"><DocumentCell document={l.document} /></td>
							<td class="px-2 py-1 text-right font-mono">{amountOrBlank(l.debit)}</td>
							<td class="px-2 py-1 text-right font-mono">{amountOrBlank(l.credit)}</td>
						</tr>
					{/each}
				</tbody>
			</table>
		</div>

		{#if canWrite}
			{#if g.manualDissolutionBlockedBy === null}
				<button
					type="button"
					class="rounded bg-primary px-3 py-1 text-primary-foreground disabled:opacity-50"
					disabled={busy}
					onclick={() => onDissolve(g.code)}
					data-testid="lettering-group-dissolve"
					>{i18nMsg('open-items-group-dissolve', 'Délettrer')}</button
				>
			{:else}
				<p class="rounded bg-gray-50 p-2 text-sm" data-testid="lettering-group-blocked">
					{dissolutionBlockerLabel(g.manualDissolutionBlockedBy)}
					{#if g.origin === 'document' && doc?.href}
						<a href={doc.href} class="text-primary underline" data-testid="lettering-group-document-link"
							>{doc.number}</a
						>
					{/if}
				</p>
			{/if}
		{/if}
	{/if}
</section>
