<script lang="ts">
	import { onMount } from 'svelte';
	import { goto } from '$app/navigation';
	import {
		createPaymentBatch,
		listPaymentBatches,
	} from '$lib/features/payment-batches/payment-batches.api';
	import {
		failedItemLabel,
		formatBatchTotal,
		paymentBatchStatusLabel,
	} from '$lib/features/payment-batches/payment-batch-helpers';
	import type {
		CreatePaymentBatchResponse,
		PaymentBatchListItemResponse,
	} from '$lib/features/payment-batches/payment-batches.types';
	import { listSupplierInvoices } from '$lib/features/supplier-invoices/supplier-invoices.api';
	import type { SupplierInvoiceListItemResponse } from '$lib/features/supplier-invoices/supplier-invoices.types';
	import { listBankAccounts } from '$lib/features/bank-accounts/bank-accounts.api';
	import type { BankAccountSummary } from '$lib/features/bank-accounts/bank-accounts.api';
	import { isApiError } from '$lib/shared/utils/api-client';
	import { i18nMsg } from '$lib/shared/utils/i18n.svelte';

	let batches = $state<PaymentBatchListItemResponse[]>([]);
	let openInvoices = $state<SupplierInvoiceListItemResponse[]>([]);
	let bankAccounts = $state<BankAccountSummary[]>([]);
	let loading = $state(true);
	let errorMsg = $state('');

	let showForm = $state(false);
	let saving = $state(false);
	let formError = $state('');
	let lastResult = $state<CreatePaymentBatchResponse | null>(null);

	const today = new Date().toISOString().slice(0, 10);
	let fBankAccountId = $state<number | null>(null);
	let fExecutionDate = $state(today);
	let fSelected = $state<Set<number>>(new Set());

	async function reload() {
		const res = await listPaymentBatches({ limit: 100 });
		batches = res.items;
	}

	onMount(async () => {
		try {
			const [, invRes, banks] = await Promise.all([
				reload(),
				listSupplierInvoices({ limit: 200 }),
				listBankAccounts(),
			]);
			openInvoices = invRes.items.filter((i) => i.status === 'open');
			bankAccounts = banks.filter((b) => b.journalAccountId !== null);
		} catch (err) {
			if (isApiError(err)) errorMsg = err.message;
		} finally {
			loading = false;
		}
	});

	function toggle(id: number) {
		const next = new Set(fSelected);
		if (next.has(id)) next.delete(id);
		else next.add(id);
		fSelected = next;
	}

	async function submit() {
		formError = '';
		lastResult = null;
		if (!fBankAccountId) {
			formError = i18nMsg('payment-batches-err-bank', 'Sélectionnez un compte bancaire source.');
			return;
		}
		if (fSelected.size === 0) {
			formError = i18nMsg('payment-batches-err-invoices', 'Sélectionnez au moins une facture.');
			return;
		}
		saving = true;
		try {
			lastResult = await createPaymentBatch({
				bankAccountId: fBankAccountId,
				requestedExecutionDate: fExecutionDate,
				supplierInvoiceIds: [...fSelected],
			});
			fSelected = new Set();
			await reload();
			// Recharger les factures ouvertes (celles incluses sont maintenant verrouillées).
			const invRes = await listSupplierInvoices({ limit: 200 });
			openInvoices = invRes.items.filter((i) => i.status === 'open');
		} catch (err) {
			if (isApiError(err)) formError = err.message;
		} finally {
			saving = false;
		}
	}
</script>

<svelte:head>
	<title>{i18nMsg('payment-batches-title', 'Paiements fournisseurs')} — Kesh</title>
</svelte:head>

<div class="mb-6 flex items-center justify-between">
	<h1 class="text-2xl font-semibold">
		{i18nMsg('payment-batches-title', 'Paiements fournisseurs')}
	</h1>
	<button
		class="rounded bg-primary px-4 py-2 text-sm text-primary-foreground"
		data-testid="payment-batch-new"
		onclick={() => (showForm = !showForm)}
	>
		{showForm
			? i18nMsg('payment-batches-form-close', 'Fermer')
			: i18nMsg('payment-batches-new', 'Créer un lot de virements')}
	</button>
</div>

{#if showForm}
	<form
		class="mb-8 space-y-4 rounded border border-border p-4"
		data-testid="payment-batch-form"
		onsubmit={(e) => {
			e.preventDefault();
			submit();
		}}
	>
		<div class="grid grid-cols-2 gap-4">
			<label class="block text-sm">
				{i18nMsg('payment-batches-field-bank', 'Compte bancaire source')}
				<select class="mt-1 w-full rounded border px-2 py-1" bind:value={fBankAccountId}>
					<option value={null}>—</option>
					{#each bankAccounts as b (b.id)}
						<option value={b.id}>{b.bankName} — {b.iban}</option>
					{/each}
				</select>
			</label>
			<label class="block text-sm">
				{i18nMsg('payment-batches-field-date', "Date d'exécution souhaitée")}
				<input type="date" class="mt-1 w-full rounded border px-2 py-1" bind:value={fExecutionDate} />
			</label>
		</div>

		<div>
			<div class="mb-2 text-sm font-medium">
				{i18nMsg('payment-batches-select-invoices', 'Factures à régler par virement')}
			</div>
			{#if openInvoices.length === 0}
				<p class="text-sm text-text-muted">
					{i18nMsg('payment-batches-no-open', 'Aucune facture fournisseur ouverte.')}
				</p>
			{:else}
				<div class="max-h-64 overflow-y-auto rounded border">
					{#each openInvoices as inv (inv.id)}
						<label class="flex items-center gap-2 border-b px-2 py-1 text-sm last:border-0">
							<input
								type="checkbox"
								checked={fSelected.has(inv.id)}
								onchange={() => toggle(inv.id)}
								data-testid="payment-batch-invoice-checkbox"
							/>
							<span class="flex-1">{inv.supplierInvoiceNumber ?? `#${inv.id}`} — {inv.contactName}</span>
							<span class="tabular-nums">{formatBatchTotal(inv.totalAmount)}</span>
						</label>
					{/each}
				</div>
			{/if}
		</div>

		{#if formError}
			<p class="text-sm text-destructive" data-testid="payment-batch-form-error">{formError}</p>
		{/if}

		<button
			type="submit"
			class="rounded bg-primary px-4 py-2 text-sm text-primary-foreground"
			data-testid="payment-batch-submit"
			disabled={saving}
		>
			{saving ? '…' : i18nMsg('payment-batches-generate', 'Créer le lot')}
		</button>
	</form>
{/if}

{#if lastResult}
	<div class="mb-6 rounded border border-border p-4" data-testid="payment-batch-result">
		{#if lastResult.batch}
			<p class="text-sm">
				{i18nMsg('payment-batches-created', 'Lot créé')} —
				<a class="text-primary underline" href={`/payment-batches/${lastResult.batch.id}`}>
					{i18nMsg('payment-batches-open', 'ouvrir et télécharger le fichier')}
				</a>
			</p>
		{/if}
		{#if lastResult.failed.length > 0}
			<p class="mt-2 text-sm font-medium text-destructive">
				{i18nMsg('payment-batches-rejected', 'Factures non incluses :')}
			</p>
			<ul class="ml-4 list-disc text-sm text-destructive">
				{#each lastResult.failed as f (f.supplierInvoiceId)}
					<li>#{f.supplierInvoiceId} — {failedItemLabel(f.errorCode)}</li>
				{/each}
			</ul>
		{/if}
	</div>
{/if}

{#if loading}
	<p class="text-sm text-text-muted">{i18nMsg('common-loading', 'Chargement…')}</p>
{:else if errorMsg}
	<p class="text-sm text-destructive">{errorMsg}</p>
{:else if batches.length === 0}
	<p class="text-sm text-text-muted">
		{i18nMsg('payment-batches-empty', 'Aucun lot de paiement.')}
	</p>
{:else}
	<table class="w-full border-collapse text-sm" data-testid="payment-batches-table">
		<thead>
			<tr class="border-b text-left text-text-muted">
				<th class="py-2">{i18nMsg('payment-batches-col-id', 'Lot')}</th>
				<th class="py-2">{i18nMsg('payment-batches-col-date', "Exécution")}</th>
				<th class="py-2">{i18nMsg('payment-batches-col-status', 'Statut')}</th>
				<th class="py-2 text-right">{i18nMsg('payment-batches-col-total', 'Total')}</th>
			</tr>
		</thead>
		<tbody>
			{#each batches as b (b.id)}
				<tr
					class="cursor-pointer border-b hover:bg-surface-hover"
					data-testid="payment-batch-row"
					onclick={() => goto(`/payment-batches/${b.id}`)}
				>
					<td class="py-2 font-mono">#{b.id}</td>
					<td class="py-2">{b.requestedExecutionDate}</td>
					<td class="py-2">{paymentBatchStatusLabel(b.status)}</td>
					<td class="py-2 text-right">{formatBatchTotal(b.totalAmount)}</td>
				</tr>
			{/each}
		</tbody>
	</table>
{/if}
