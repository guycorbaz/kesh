<script lang="ts">
	import { onMount } from 'svelte';
	import { goto } from '$app/navigation';
	import { page } from '$app/state';
	import {
		cancelPaymentBatch,
		confirmPaymentBatch,
		getPaymentBatch,
		pain001DownloadUrl,
	} from '$lib/features/payment-batches/payment-batches.api';
	import {
		formatBatchTotal,
		paymentBatchStatusLabel,
	} from '$lib/features/payment-batches/payment-batch-helpers';
	import type { PaymentBatchResponse } from '$lib/features/payment-batches/payment-batches.types';
	import { isApiError } from '$lib/shared/utils/api-client';
	import { i18nMsg } from '$lib/shared/utils/i18n.svelte';

	const id = Number(page.params.id);

	let batch = $state<PaymentBatchResponse | null>(null);
	let loading = $state(true);
	let errorMsg = $state('');

	const today = new Date().toISOString().slice(0, 10);
	let paymentDate = $state(today);
	let acting = $state(false);
	let actionError = $state('');

	onMount(async () => {
		try {
			batch = await getPaymentBatch(id);
		} catch (err) {
			if (isApiError(err)) errorMsg = err.message;
		} finally {
			loading = false;
		}
	});

	async function confirmBatch() {
		actionError = '';
		acting = true;
		try {
			batch = await confirmPaymentBatch(id, { paymentDate });
		} catch (err) {
			if (isApiError(err)) actionError = err.message;
		} finally {
			acting = false;
		}
	}

	async function cancelBatch() {
		const ok = window.confirm(
			i18nMsg('payment-batches-cancel-confirm', 'Annuler ce lot ? Les factures redeviennent réglables.'),
		);
		if (!ok) return;
		actionError = '';
		acting = true;
		try {
			batch = await cancelPaymentBatch(id);
		} catch (err) {
			if (isApiError(err)) actionError = err.message;
		} finally {
			acting = false;
		}
	}
</script>

<svelte:head>
	<title>{i18nMsg('payment-batches-detail-title', 'Lot de paiement')} — Kesh</title>
</svelte:head>

<button class="mb-4 text-sm text-primary" onclick={() => goto('/payment-batches')}>
	← {i18nMsg('payment-batches-back', 'Retour')}
</button>

{#if loading}
	<p class="text-sm text-text-muted">Chargement…</p>
{:else if errorMsg}
	<p class="text-sm text-destructive">{errorMsg}</p>
{:else if batch}
	<div class="mb-6 flex items-center justify-between">
		<h1 class="text-2xl font-semibold">{i18nMsg('payment-batches-lot', 'Lot')} #{batch.id}</h1>
		<span data-testid="payment-batch-status">{paymentBatchStatusLabel(batch.status)}</span>
	</div>

	<dl class="mb-6 grid grid-cols-2 gap-2 text-sm">
		<dt class="text-text-muted">{i18nMsg('payment-batches-col-date', "Date d'exécution")}</dt>
		<dd>{batch.requestedExecutionDate}</dd>
		<dt class="text-text-muted">{i18nMsg('payment-batches-col-total', 'Total')}</dt>
		<dd>{formatBatchTotal(batch.totalAmount)}</dd>
		<dt class="text-text-muted">{i18nMsg('payment-batches-msg-id', 'Référence message')}</dt>
		<dd class="font-mono">{batch.msgId}</dd>
		{#if batch.confirmedAt}
			<dt class="text-text-muted">{i18nMsg('payment-batches-confirmed-at', 'Confirmé le')}</dt>
			<dd>{batch.confirmedAt}</dd>
		{/if}
	</dl>

	<a
		class="mb-6 inline-block rounded border px-4 py-2 text-sm text-primary"
		data-testid="payment-batch-download"
		href={pain001DownloadUrl(batch.id)}
		download={`pain001-${batch.id}.xml`}
	>
		{i18nMsg('payment-batches-download', 'Télécharger le fichier pain.001')}
	</a>

	<table class="mb-6 w-full border-collapse text-sm">
		<thead>
			<tr class="border-b text-left text-text-muted">
				<th class="py-2">{i18nMsg('payment-batches-item-invoice', 'Facture')}</th>
				<th class="py-2">EndToEndId</th>
				<th class="py-2 text-right">{i18nMsg('payment-batches-col-total', 'Montant')}</th>
			</tr>
		</thead>
		<tbody>
			{#each batch.items as item (item.position)}
				<tr class="border-b">
					<td class="py-2">
						<a class="text-primary underline" href={`/supplier-invoices/${item.supplierInvoiceId}`}>
							#{item.supplierInvoiceId}
						</a>
					</td>
					<td class="py-2 font-mono text-xs">{item.endToEndId}</td>
					<td class="py-2 text-right">{formatBatchTotal(item.amount)}</td>
				</tr>
			{/each}
		</tbody>
	</table>

	{#if batch.status === 'generated'}
		<div class="rounded border border-border p-4" data-testid="payment-batch-actions">
			<h2 class="mb-3 font-medium">{i18nMsg('payment-batches-confirm-title', 'Confirmer le règlement')}</h2>
			<p class="mb-3 text-sm text-text-muted">
				{i18nMsg(
					'payment-batches-confirm-hint',
					'Une fois le virement exécuté dans votre e-banking, confirmez pour comptabiliser les règlements.',
				)}
			</p>
			<label class="mb-3 block text-sm">
				{i18nMsg('payment-batches-payment-date', 'Date de règlement effective')}
				<input type="date" class="mt-1 w-full rounded border px-2 py-1" bind:value={paymentDate} />
			</label>
			{#if actionError}
				<p class="mb-2 text-sm text-destructive" data-testid="payment-batch-action-error">{actionError}</p>
			{/if}
			<div class="flex gap-2">
				<button
					class="rounded bg-primary px-4 py-2 text-sm text-primary-foreground"
					data-testid="payment-batch-confirm"
					onclick={confirmBatch}
					disabled={acting}
				>
					{acting ? '…' : i18nMsg('payment-batches-confirm', 'Confirmer le lot')}
				</button>
				<button
					class="rounded border px-4 py-2 text-sm text-destructive"
					data-testid="payment-batch-cancel"
					onclick={cancelBatch}
					disabled={acting}
				>
					{i18nMsg('payment-batches-cancel', 'Annuler le lot')}
				</button>
			</div>
		</div>
	{/if}
{/if}
