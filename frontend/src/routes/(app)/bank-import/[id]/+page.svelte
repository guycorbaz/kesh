<!--
  Story 8-1b — Page détail d'un import bancaire (review code Pass 1 H2).
  Route /bank-import/[id]. Consomme GET /api/v1/bank-imports/{id}.
  Affiche l'entête (filename, période, soldes, source_format) + table
  des transactions persistées.
-->
<script lang="ts">
	import { page } from '$app/state';
	import { onMount } from 'svelte';
	import { getBankImportDetail } from '$lib/features/bank-import/bank-import.api';
	import type { BankImportDetailResponse } from '$lib/features/bank-import/bank-import.types';
	import { isApiError } from '$lib/shared/utils/api-client';

	let detail = $state<BankImportDetailResponse | null>(null);
	let errorMessage = $state<string | null>(null);
	let loading = $state(true);

	onMount(async () => {
		const idStr = page.params.id;
		const id = idStr ? Number(idStr) : NaN;
		if (!Number.isFinite(id) || id <= 0) {
			errorMessage = 'Identifiant invalide';
			loading = false;
			return;
		}
		try {
			detail = await getBankImportDetail(id);
		} catch (err) {
			if (isApiError(err)) {
				errorMessage =
					err.status === 404 ? 'Import introuvable.' : err.message;
			} else {
				errorMessage = 'Erreur inattendue.';
			}
		} finally {
			loading = false;
		}
	});
</script>

<svelte:head>
	<title>Détail import bancaire - Kesh</title>
</svelte:head>

<a href="/bank-import" class="text-primary underline" data-testid="bank-import-detail-back">
	← Retour aux imports
</a>

<h1 class="mt-2 text-2xl font-semibold text-text" data-testid="bank-import-detail-title">
	Détail import bancaire
</h1>

{#if loading}
	<p class="mt-4 text-text-muted">Chargement…</p>
{:else if errorMessage}
	<div
		class="mt-4 rounded border border-error bg-error-soft p-3"
		data-testid="bank-import-detail-error"
		role="alert"
	>
		<p class="text-error">{errorMessage}</p>
	</div>
{:else if detail}
	<section class="mt-6" data-testid="bank-import-detail">
		<h2 class="text-lg font-semibold">Import #{detail.id}</h2>
		<dl class="mt-2 grid grid-cols-2 gap-2 text-sm">
			<dt>Fichier</dt>
			<dd data-testid="detail-filename">{detail.filename}</dd>
			<dt>Importé le</dt>
			<dd>{detail.importedAt}</dd>
			<dt>Format source</dt>
			<dd data-testid="detail-source-format">{detail.sourceFormat}</dd>
			<dt>Période</dt>
			<dd>{detail.periodFrom} → {detail.periodTo}</dd>
			<dt>Solde ouverture</dt>
			<dd>{detail.openingBalance ?? '—'}</dd>
			<dt>Solde clôture</dt>
			<dd>{detail.closingBalance ?? '—'}</dd>
			<dt>Nombre de transactions</dt>
			<dd data-testid="detail-tx-count">{detail.transactionCount}</dd>
		</dl>

		<h2 class="mt-6 text-lg font-semibold">Transactions</h2>
		<table class="mt-2 w-full table-auto text-sm" data-testid="detail-tx-table">
			<thead>
				<tr>
					<th class="text-left">Date</th>
					<th class="text-right">Montant</th>
					<th class="text-left">Référence</th>
					<th class="text-left">Détails</th>
					<th class="text-left">Statut</th>
				</tr>
			</thead>
			<tbody>
				{#each detail.transactions as tx (tx.id)}
					<tr data-testid="detail-tx-row" data-tx-id={tx.id}>
						<td>{tx.bookingDate}</td>
						<td class="text-right">{tx.amount} {tx.currency}</td>
						<td>{tx.reference ?? '—'}</td>
						<td>{tx.details}</td>
						<td>{tx.status}</td>
					</tr>
				{/each}
			</tbody>
		</table>
	</section>
{/if}
