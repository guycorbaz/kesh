<!--
  Story 8-4 (FR44) — top-1 only par tx via checkbox tx-level. Candidates
  2..N ignorées côté UI v0.1 — voir KF-026 #76 pour le pattern multi-
  candidates v0.2.

  Liste de propositions de réconciliation avec sélection par tx
  (checkbox) + actions batch Accept/Reject. Pour chaque transaction
  pending, on affiche la candidate top-1 (la mieux scorée) et un
  checkbox au niveau tx. Si la transaction n'a aucune candidate,
  on affiche « Aucune correspondance » sans checkbox (tx
  non-sélectionnable). Erreurs partielles affichées en bas via
  `failed`.
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
	// γ refactor BS1 — checkbox tx-level top-1 only : selected = Set<txId>
	// (l'invoiceId top-1 est ré-extrait au moment du submit).
	let selected = $state<Set<number>>(new Set());
	let loading = $state(false);
	let busy = $state(false);
	let errorMsg = $state<string | null>(null);
	let failed = $state<FailedProposal[]>([]);
	let lastSuccessCount = $state(0);

	// H7 Pass 1 code review — generation tag pour drop les responses
	// `getProposals` stale (race sur fast account switch).
	let loadGen = 0;

	async function load() {
		const myGen = ++loadGen;
		loading = true;
		errorMsg = null;
		try {
			const r = await getProposals(bankAccountId);
			if (myGen !== loadGen) return; // stale response, drop
			proposals = r.proposals;
			selected = new Set();
		} catch (e) {
			if (myGen !== loadGen) return;
			errorMsg = e instanceof Error ? e.message : String(e);
		} finally {
			if (myGen === loadGen) loading = false;
		}
	}

	$effect(() => {
		void load();
	});

	function toggle(txId: number) {
		const next = new Set(selected);
		if (next.has(txId)) next.delete(txId);
		else next.add(txId);
		selected = next;
	}

	async function onAccept() {
		if (selected.size === 0) return;
		busy = true;
		errorMsg = null;
		failed = [];
		try {
			// γ refactor — pour chaque txId sélectionné, retrouver la
			// candidate top-1 (candidates[0]) dans la proposal courante.
			// Une tx sans candidate ne peut pas être sélectionnée (cf.
			// disabled checkbox), donc candidates[0] est garanti exister.
			const items = Array.from(selected).map((txId) => {
				const p = proposals.find((x) => x.bankTransactionId === txId);
				if (!p || p.candidates.length === 0) {
					throw new Error(`No candidate for txId=${txId}`);
				}
				return {
					bankTransactionId: txId,
					invoiceId: p.candidates[0].invoiceId,
				};
			});
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
			const ids = Array.from(selected);
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
					<th class="text-left"></th>
					<th class="text-left">{i18nMsg('reconciliation-cols-tx-date', 'Date')}</th>
					<th class="text-right">{i18nMsg('reconciliation-cols-tx-amount', 'Montant')}</th>
					<th class="text-left">{i18nMsg('reconciliation-cols-tx-counterparty', 'Contrepartie')}</th>
					<th class="text-left">{i18nMsg('reconciliation-cols-candidate', 'Candidate')}</th>
					<th class="text-left">{i18nMsg('reconciliation-cols-score', 'Score')}</th>
				</tr>
			</thead>
			<tbody>
				{#each proposals as p (p.bankTransactionId)}
					<tr data-testid="reconciliation-row" data-tx-id={p.bankTransactionId}>
						<td>
							{#if p.candidates.length > 0}
								<input
									type="checkbox"
									checked={selected.has(p.bankTransactionId)}
									onchange={() => toggle(p.bankTransactionId)}
									data-testid="tx-checkbox"
									data-tx-id={p.bankTransactionId}
								/>
							{/if}
						</td>
						<td>{p.transaction.bookingDate}</td>
						<td class="text-right">{p.transaction.amount} {p.transaction.currency}</td>
						<td>{p.transaction.counterpartyName ?? ''}</td>
						{#if p.candidates.length === 0}
							<td colspan="2" class="text-text-muted">
								{i18nMsg('reconciliation-labels-no-candidate', 'Aucune correspondance')}
							</td>
						{:else}
							<td>
								<span data-testid="candidate-top-1" data-invoice-id={p.candidates[0].invoiceId}>
									{p.candidates[0].invoiceNumber ?? `#${p.candidates[0].invoiceId}`}
									({p.candidates[0].invoiceAmount})
								</span>
							</td>
							<td>
								<ScoreBadge score={p.candidates[0].score.total} />
							</td>
						{/if}
					</tr>
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
