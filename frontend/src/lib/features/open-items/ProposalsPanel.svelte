<script lang="ts">
	// Story 15-1c-i (AC5, C-15-1c-7) — « Rapprochements proposés ».
	//
	// ⛔ **Aucun lettrage sans clic** (CLAUDE.md, « Un appariement automatique
	// propose, il ne crée jamais ») : ce panneau n'écrit rien de lui-même ;
	// « Lettrer » remonte la paire à l'écran, qui envoie un `POST` par clic.
	// Les propositions sont calculées **aujourd'hui** : elles ne suivent pas la
	// date de la liste, et une ligne d'une paire peut ne pas y figurer — d'où ses
	// deux lignes affichées en entier, lues dans la paire.
	import { i18nMsg } from '$lib/shared/utils/i18n.svelte';
	import { formatSwissDate } from '$lib/features/reports/reports.api';
	import DocumentCell from './DocumentCell.svelte';
	import { entryRefLabel, formatAmountString } from './open-items-labels';
	import type { Proposal, ProposalLine, ProposalsState } from './open-items.types';

	interface Props {
		proposals: ProposalsState;
		canWrite: boolean;
		/** Un geste d'écriture est en cours : les boutons attendent. */
		busy: boolean;
		onAccept: (p: Proposal) => void;
	}
	let { proposals, canWrite, busy, onAccept }: Props = $props();
</script>

{#snippet proposalLine(l: ProposalLine, side: string)}
	<div class="flex flex-wrap gap-x-3 text-xs" data-testid="proposal-line-{l.lineId}">
		<span class="font-semibold">{side}</span>
		<span>{formatSwissDate(l.date)}</span>
		<a href="/journal-entries/{l.entryId}" class="text-primary underline"
			>{entryRefLabel(l.fiscalYearName, l.entryNumber)}</a
		>
		<span>{l.journal}</span>
		<span>{l.description}</span>
		<DocumentCell document={l.document} />
	</div>
{/snippet}

<section class="space-y-2 rounded border p-3" data-testid="open-items-proposals">
	<h2 class="font-semibold">{i18nMsg('open-items-proposals-title', 'Rapprochements proposés')}</h2>
	<p class="text-xs text-gray-600">
		{i18nMsg(
			'open-items-proposals-today',
			"Calculés sur les lignes ouvertes aujourd'hui, quelle que soit la date choisie.",
		)}
	</p>

	{#if proposals.status === 'loading'}
		<p class="text-sm text-gray-500" role="status">{i18nMsg('common-loading', 'Chargement…')}</p>
	{:else if proposals.status === 'error'}
		<p class="rounded bg-amber-50 p-2 text-sm text-amber-900" role="alert" data-testid="open-items-proposals-error">
			{proposals.message}
		</p>
	{:else if proposals.data.items.length === 0}
		<p class="text-sm italic text-gray-500" data-testid="open-items-proposals-empty">
			{i18nMsg('open-items-proposals-empty', 'Aucun rapprochement à proposer.')}
		</p>
	{:else}
		<ul class="space-y-2">
			{#each proposals.data.items as p (`${p.debit.lineId}-${p.credit.lineId}`)}
				<li class="rounded border p-2" data-testid="proposal-{p.debit.lineId}-{p.credit.lineId}">
					<div class="flex flex-wrap items-center gap-3 text-sm">
						<span class="font-mono font-semibold">{formatAmountString(p.amount)}</span>
						<span class="text-xs text-gray-600">
							{i18nMsg('open-items-proposal-days', 'Écart de dates (jours) : { $days }', {
								days: p.daysApart,
							})}
						</span>
						{#if p.reversalPair}
							<span
								class="rounded bg-indigo-100 px-2 text-xs text-indigo-900"
								data-testid="proposal-reversal-{p.debit.lineId}-{p.credit.lineId}"
								>{i18nMsg('open-items-proposal-reversal', 'contre-passation')}</span
							>
						{/if}
						{#if canWrite}
							<button
								type="button"
								class="ml-auto rounded bg-primary px-3 py-1 text-primary-foreground disabled:opacity-50"
								disabled={busy}
								onclick={() => onAccept(p)}
								data-testid="proposal-letter-{p.debit.lineId}-{p.credit.lineId}"
								>{i18nMsg('open-items-letter', 'Lettrer')}</button
							>
						{/if}
					</div>
					{@render proposalLine(p.debit, i18nMsg('open-items-col-debit', 'Débit'))}
					{@render proposalLine(p.credit, i18nMsg('open-items-col-credit', 'Crédit'))}
				</li>
			{/each}
		</ul>
		{#if proposals.data.total > proposals.data.items.length}
			<p class="text-xs text-gray-600" data-testid="open-items-proposals-more">
				{i18nMsg(
					'open-items-proposals-more',
					'Rapprochements non affichés : { $count } — ils apparaîtront quand ceux-ci seront lettrés.',
					{ count: proposals.data.total - proposals.data.items.length },
				)}
			</p>
		{/if}
	{/if}
</section>
