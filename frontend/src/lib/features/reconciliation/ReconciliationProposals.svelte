<!--
  Story 8-4 (FR44) — Liste de propositions de réconciliation avec
  sélection per-tx (radio sur la candidate) + actions batch
  Accept/Reject. Erreurs partielles affichées en bas via `failed`.
-->
<script lang="ts">
	import { i18nMsg } from '$lib/shared/utils/i18n.svelte';
	import {
		acceptProposals,
		getProposals,
		rejectProposals,
	} from './reconciliation.api';
	import type {
		FailedProposal,
		ReconciliationProposal,
	} from './reconciliation.types';
	import ScoreBadge from './ScoreBadge.svelte';

	type Props = {
		bankAccountId: number;
	};
	let { bankAccountId }: Props = $props();

	let proposals = $state<ReconciliationProposal[]>([]);
	let selected = $state<Map<number, number>>(new Map()); // txId → invoiceId
	let loading = $state(false);
	let busy = $state(false);
	let errorMsg = $state<string | null>(null);
	let failed = $state<FailedProposal[]>([]);
	let lastSuccessCount = $state(0);

	async function load() {
		loading = true;
		errorMsg = null;
		try {
			const r = await getProposals(bankAccountId);
			proposals = r.proposals;
			selected = new Map();
		} catch (e) {
			errorMsg = e instanceof Error ? e.message : String(e);
		} finally {
			loading = false;
		}
	}

	$effect(() => {
		void load();
	});

	function toggle(txId: number, invoiceId: number) {
		const next = new Map(selected);
		if (next.get(txId) === invoiceId) next.delete(txId);
		else next.set(txId, invoiceId);
		selected = next;
	}

	async function onAccept() {
		if (selected.size === 0) return;
		busy = true;
		errorMsg = null;
		failed = [];
		try {
			const items = Array.from(selected.entries()).map(([bankTransactionId, invoiceId]) => ({
				bankTransactionId,
				invoiceId,
			}));
			const r = await acceptProposals(bankAccountId, items);
			lastSuccessCount = r.accepted.length;
			failed = r.failed;
			await load();
		} catch (e) {
			errorMsg = e instanceof Error ? e.message : String(e);
		} finally {
			busy = false;
		}
	}

	async function onReject() {
		if (selected.size === 0) return;
		busy = true;
		errorMsg = null;
		failed = [];
		try {
			const ids = Array.from(selected.keys());
			const r = await rejectProposals(bankAccountId, ids);
			lastSuccessCount = r.rejected.length;
			failed = r.failed;
			await load();
		} catch (e) {
			errorMsg = e instanceof Error ? e.message : String(e);
		} finally {
			busy = false;
		}
	}
</script>

<section data-testid="reconciliation-proposals">
	{#if loading}
		<p class="text-text-muted" data-testid="reconciliation-loading">
			{i18nMsg('reconciliation-labels-loading', 'Chargement des propositions…')}
		</p>
	{:else if errorMsg}
		<p class="text-red-700" data-testid="reconciliation-error">{errorMsg}</p>
	{:else if proposals.length === 0}
		<p class="text-text-muted" data-testid="reconciliation-empty">
			{i18nMsg(
				'reconciliation-labels-empty',
				'Aucune transaction en attente de réconciliation.',
			)}
		</p>
	{:else}
		<div class="mb-4 flex gap-2">
			<button
				type="button"
				class="rounded bg-primary px-4 py-2 text-white disabled:opacity-50"
				disabled={busy || selected.size === 0}
				onclick={onAccept}
				data-testid="reconciliation-accept-btn"
			>
				{i18nMsg('reconciliation-actions-accept', 'Accepter')} ({selected.size})
			</button>
			<button
				type="button"
				class="rounded border border-text px-4 py-2 disabled:opacity-50"
				disabled={busy || selected.size === 0}
				onclick={onReject}
				data-testid="reconciliation-reject-btn"
			>
				{i18nMsg('reconciliation-actions-reject', 'Rejeter')}
			</button>
		</div>

		{#if lastSuccessCount > 0}
			<p class="mb-2 text-green-700" data-testid="reconciliation-success">
				{lastSuccessCount}
				{i18nMsg('reconciliation-labels-success-suffix', 'opération(s) réussie(s).')}
			</p>
		{/if}

		<table class="w-full table-auto text-sm" data-testid="reconciliation-table">
			<thead>
				<tr>
					<th class="text-left">{i18nMsg('reconciliation-cols-tx-date', 'Date')}</th>
					<th class="text-right">{i18nMsg('reconciliation-cols-tx-amount', 'Montant')}</th>
					<th class="text-left">{i18nMsg('reconciliation-cols-tx-counterparty', 'Contrepartie')}</th>
					<th class="text-left">{i18nMsg('reconciliation-cols-candidate', 'Candidate')}</th>
					<th class="text-left">{i18nMsg('reconciliation-cols-score', 'Score')}</th>
				</tr>
			</thead>
			<tbody>
				{#each proposals as p (p.bankTransactionId)}
					{#if p.candidates.length === 0}
						<tr data-testid="reconciliation-row" data-tx-id={p.bankTransactionId}>
							<td>{p.transaction.bookingDate}</td>
							<td class="text-right">{p.transaction.amount} {p.transaction.currency}</td>
							<td>{p.transaction.counterpartyName ?? ''}</td>
							<td colspan="2" class="text-text-muted">
								{i18nMsg('reconciliation-labels-no-candidate', 'Aucune correspondance')}
							</td>
						</tr>
					{:else}
						{#each p.candidates as c, idx (c.invoiceId)}
							<tr data-testid="reconciliation-row" data-tx-id={p.bankTransactionId}>
								{#if idx === 0}
									<td rowspan={p.candidates.length}>{p.transaction.bookingDate}</td>
									<td rowspan={p.candidates.length} class="text-right">
										{p.transaction.amount}
										{p.transaction.currency}
									</td>
									<td rowspan={p.candidates.length}>
										{p.transaction.counterpartyName ?? ''}
									</td>
								{/if}
								<td>
									<label class="flex items-center gap-2">
										<input
											type="radio"
											name="candidate-{p.bankTransactionId}"
											checked={selected.get(p.bankTransactionId) === c.invoiceId}
											onchange={() => toggle(p.bankTransactionId, c.invoiceId)}
											data-testid="candidate-radio"
											data-tx-id={p.bankTransactionId}
											data-invoice-id={c.invoiceId}
										/>
										<span>{c.invoiceNumber ?? `#${c.invoiceId}`} ({c.invoiceAmount})</span>
									</label>
								</td>
								<td>
									<ScoreBadge score={c.score.total} />
								</td>
							</tr>
						{/each}
					{/if}
				{/each}
			</tbody>
		</table>

		{#if failed.length > 0}
			<div class="mt-4" data-testid="reconciliation-failed">
				<h3 class="font-semibold text-red-700">
					{i18nMsg('reconciliation-labels-failed', 'Échecs partiels')}
				</h3>
				<ul class="list-disc pl-6 text-sm">
					{#each failed as f (f.bankTransactionId)}
						<li>
							TX #{f.bankTransactionId} — {f.errorCode}
						</li>
					{/each}
				</ul>
			</div>
		{/if}
	{/if}
</section>
