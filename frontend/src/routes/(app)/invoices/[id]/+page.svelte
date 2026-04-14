<script lang="ts">
	import { Button } from '$lib/components/ui/button';
	import * as Dialog from '$lib/components/ui/dialog';
	import { onMount } from 'svelte';
	import { goto } from '$app/navigation';
	import { page } from '$app/state';
	import { Pencil, Trash2, ArrowLeft } from '@lucide/svelte';

	import { getInvoice, deleteInvoice } from '$lib/features/invoices/invoices.api';
	import type { InvoiceResponse } from '$lib/features/invoices/invoices.types';
	import { formatInvoiceTotal } from '$lib/features/invoices/invoice-helpers';
	import { isApiError } from '$lib/shared/utils/api-client';
	import { notifyError, notifySuccess } from '$lib/shared/utils/notify';

	let invoice = $state<InvoiceResponse | null>(null);
	let loading = $state(true);
	let errorMsg = $state('');
	let deleteOpen = $state(false);
	let deleteSubmitting = $state(false);
	let deleteError = $state('');

	let id = $derived(parseInt(page.params.id ?? '', 10));

	onMount(async () => {
		if (!Number.isFinite(id) || id <= 0) {
			errorMsg = 'Identifiant de facture invalide';
			loading = false;
			return;
		}
		try {
			invoice = await getInvoice(id);
		} catch (err) {
			if (isApiError(err)) errorMsg = err.message;
		} finally {
			loading = false;
		}
	});

	async function confirmDelete() {
		if (!invoice) return;
		deleteSubmitting = true;
		deleteError = '';
		try {
			await deleteInvoice(invoice.id);
			notifySuccess('Facture supprimée');
			goto('/invoices');
		} catch (err) {
			// Cohérence avec la modale de conflit (InvoiceForm) : erreur visible
			// dans la modale en plus du toast.
			if (isApiError(err)) {
				deleteError = err.message;
				notifyError(err.message);
			} else {
				deleteError = 'Erreur lors de la suppression';
				notifyError(deleteError);
			}
		} finally {
			deleteSubmitting = false;
		}
	}

	function statusLabel(s: string): string {
		if (s === 'draft') return 'Brouillon';
		if (s === 'validated') return 'Validée';
		return 'Annulée';
	}
</script>

<svelte:head>
	<title>Facture — Kesh</title>
</svelte:head>

<div class="mb-6 flex items-center justify-between">
	<Button variant="ghost" onclick={() => goto('/invoices')}>
		<ArrowLeft class="h-4 w-4" aria-hidden="true" />
		Retour
	</Button>
	{#if invoice?.status === 'draft'}
		<div class="flex gap-2">
			<Button onclick={() => goto(`/invoices/${invoice!.id}/edit`)}>
				<Pencil class="h-4 w-4" aria-hidden="true" />
				Modifier
			</Button>
			<Button variant="destructive" onclick={() => (deleteOpen = true)}>
				<Trash2 class="h-4 w-4" aria-hidden="true" />
				Supprimer
			</Button>
		</div>
	{/if}
</div>

{#if loading}
	<p class="text-sm text-text-muted">Chargement…</p>
{:else if errorMsg}
	<p class="text-sm text-destructive">{errorMsg}</p>
{:else if invoice}
	<div class="space-y-6">
		<div>
			<h1 class="text-2xl font-semibold">Facture</h1>
			<p class="text-sm text-text-muted">
				{invoice.invoiceNumber ?? 'Brouillon'} — {statusLabel(invoice.status)}
			</p>
		</div>

		<div class="grid grid-cols-2 gap-4 text-sm">
			<div>
				<div class="text-text-muted">Date</div>
				<div>{invoice.date}</div>
			</div>
			<div>
				<div class="text-text-muted">Échéance</div>
				<div>{invoice.dueDate ?? '—'}</div>
			</div>
			<div>
				<div class="text-text-muted">Conditions de paiement</div>
				<div>{invoice.paymentTerms ?? '—'}</div>
			</div>
		</div>

		<table class="w-full border-collapse text-sm">
			<thead>
				<tr class="border-b border-border text-left">
					<th class="py-2 pr-2">Description</th>
					<th class="py-2 pr-2">Quantité</th>
					<th class="py-2 pr-2">Prix unitaire</th>
					<th class="py-2 pr-2">TVA %</th>
					<th class="py-2 pr-2 text-right">Total</th>
				</tr>
			</thead>
			<tbody>
				{#each invoice.lines as l (l.id)}
					<tr class="border-b border-border">
						<td class="py-2 pr-2">{l.description}</td>
						<td class="py-2 pr-2">{l.quantity}</td>
						<td class="py-2 pr-2 font-mono">{formatInvoiceTotal(l.unitPrice)}</td>
						<td class="py-2 pr-2">{l.vatRate}%</td>
						<td class="py-2 pr-2 text-right font-mono">
							{formatInvoiceTotal(l.lineTotal)}
						</td>
					</tr>
				{/each}
			</tbody>
			<tfoot>
				<tr>
					<td colspan="4" class="py-3 text-right font-semibold">Total</td>
					<td class="py-3 text-right font-mono text-lg font-semibold">
						{formatInvoiceTotal(invoice.totalAmount)}
					</td>
				</tr>
			</tfoot>
		</table>
	</div>

	<Dialog.Root
		open={deleteOpen}
		onOpenChange={(o) => {
			deleteOpen = o;
			if (!o) deleteError = '';
		}}
	>
		<Dialog.Content>
			<Dialog.Header>
				<Dialog.Title>Supprimer la facture</Dialog.Title>
			</Dialog.Header>
			<p class="text-sm">Confirmer la suppression définitive de cette facture brouillon ?</p>
			{#if deleteError}
				<div class="rounded-md border border-destructive bg-destructive/10 px-3 py-2 text-sm text-destructive">
					{deleteError}
				</div>
			{/if}
			<Dialog.Footer>
				<Button variant="outline" onclick={() => (deleteOpen = false)}>Annuler</Button>
				<Button variant="destructive" onclick={confirmDelete} disabled={deleteSubmitting}>
					Supprimer
				</Button>
			</Dialog.Footer>
		</Dialog.Content>
	</Dialog.Root>
{/if}
