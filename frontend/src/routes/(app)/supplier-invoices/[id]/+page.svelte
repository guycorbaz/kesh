<script lang="ts">
	import { onMount } from 'svelte';
	import { goto } from '$app/navigation';
	import { page } from '$app/state';
	import {
		cancelSupplierInvoice,
		getSupplierInvoice,
		paySupplierInvoice,
	} from '$lib/features/supplier-invoices/supplier-invoices.api';
	import {
		formatSupplierInvoiceTotal,
		supplierInvoiceStatusLabel,
	} from '$lib/features/supplier-invoices/supplier-invoice-helpers';
	import type { SupplierInvoiceResponse } from '$lib/features/supplier-invoices/supplier-invoices.types';
	import { listBankAccounts } from '$lib/features/bank-accounts/bank-accounts.api';
	import type { BankAccountSummary } from '$lib/features/bank-accounts/bank-accounts.api';
	import { fetchAccounts } from '$lib/features/accounts/accounts.api';
	import type { AccountResponse } from '$lib/features/accounts/accounts.types';
	import { listProjects } from '$lib/features/projects/projects.api';
	import type { ProjectResponse } from '$lib/features/projects/projects.types';
	import { isApiError } from '$lib/shared/utils/api-client';
	import { i18nMsg } from '$lib/shared/utils/i18n.svelte';
	import { notifyInfo, notifyWarning, notifyError } from '$lib/shared/utils/notify';
	import { downloadSupplierInvoiceSourceDocument } from '$lib/features/imported-supplier-invoices/imported-supplier-invoices.api';

	const id = Number(page.params.id);

	let invoice = $state<SupplierInvoiceResponse | null>(null);
	let loading = $state(true);
	let errorMsg = $state('');

	let bankAccounts = $state<BankAccountSummary[]>([]);
	let accounts = $state<AccountResponse[]>([]);
	let projects = $state<ProjectResponse[]>([]);

	/** Libellé du projet analytique affecté (Story 19-3), ou `null`. */
	const projectLabel = $derived.by(() => {
		const pid = invoice?.projectId;
		if (pid == null) return null;
		const p = projects.find((x) => x.id === pid);
		return p ? `${p.code} — ${p.name}` : `#${pid}`;
	});

	// Formulaire de règlement binaire.
	let settlementType = $state<'bank_transfer' | 'internal_account'>('bank_transfer');
	const today = new Date().toISOString().slice(0, 10);
	let paymentDate = $state(today);
	let payBankAccountId = $state<number | null>(null);
	let payAccountId = $state<number | null>(null);
	let paying = $state(false);
	let payError = $state('');

	async function load() {
		invoice = await getSupplierInvoice(id);
	}

	onMount(async () => {
		try {
			const [, banks, accts, projs] = await Promise.all([
				load(),
				listBankAccounts(),
				fetchAccounts(),
				listProjects(true), // inclut les archivés pour résoudre le nom d'un projet tagué puis archivé
			]);
			bankAccounts = banks.filter((b) => b.journalAccountId !== null);
			accounts = accts.filter((a) => a.active && a.postable); // 14-3b : compte posté
			projects = projs;
		} catch (err) {
			if (isApiError(err)) errorMsg = err.message;
		} finally {
			loading = false;
		}
	});

	async function pay() {
		payError = '';
		if (settlementType === 'bank_transfer' && !payBankAccountId) {
			payError = i18nMsg('supplier-invoices-pay-err-bank', 'Sélectionnez un compte bancaire.');
			return;
		}
		if (settlementType === 'internal_account' && !payAccountId) {
			payError = i18nMsg('supplier-invoices-pay-err-account', 'Sélectionnez un compte.');
			return;
		}
		paying = true;
		try {
			invoice = await paySupplierInvoice(id, {
				settlementType,
				bankAccountId:
					settlementType === 'bank_transfer' ? (payBankAccountId ?? undefined) : undefined,
				accountId: settlementType === 'internal_account' ? (payAccountId ?? undefined) : undefined,
				paymentDate,
			});
		} catch (err) {
			if (isApiError(err)) payError = err.message;
		} finally {
			paying = false;
		}
	}

	async function cancel() {
		if (!confirm(i18nMsg('supplier-invoices-cancel-confirm', 'Annuler cette facture fournisseur ?')))
			return;
		try {
			invoice = await cancelSupplierInvoice(id);
		} catch (err) {
			if (isApiError(err)) errorMsg = err.message;
		}
	}

	/**
	 * Télécharge le justificatif d'origine (Story 12.5d, DC-d1). Le lien est
	 * toujours rendu ; une facture créée directement (sans import) n'a pas de
	 * justificatif → 404 géré gracieusement (notifyInfo), jamais bloquant.
	 */
	async function viewSourceDocument() {
		try {
			await downloadSupplierInvoiceSourceDocument(id);
		} catch (err) {
			if (isApiError(err) && err.code === 'SOURCE_DOCUMENT_NOT_FOUND') {
				notifyInfo(
					i18nMsg('imported-supplier-invoices-no-source-doc', 'Cette facture n’a pas de justificatif importé.'),
				);
			} else if (isApiError(err) && err.code === 'SOURCE_DOCUMENT_GONE') {
				notifyWarning(i18nMsg('imported-supplier-invoices-source-doc-gone', 'Le justificatif n’a pas été restauré.'));
			} else {
				notifyError(
					(isApiError(err) && err.message) ||
						i18nMsg('imported-supplier-invoices-source-doc-failed', 'Téléchargement impossible.'),
				);
			}
		}
	}
</script>

<svelte:head>
	<title>{i18nMsg('supplier-invoices-detail-title', 'Facture fournisseur')} — Kesh</title>
</svelte:head>

<button class="mb-4 text-sm text-primary" onclick={() => goto('/supplier-invoices')}>
	← {i18nMsg('common-back', 'Retour')}
</button>

{#if loading}
	<p class="text-sm text-text-muted">{i18nMsg('common-loading', 'Chargement…')}</p>
{:else if errorMsg}
	<p class="text-sm text-destructive">{errorMsg}</p>
{:else if invoice}
	<div class="mb-6 flex items-center justify-between">
		<h1 class="text-2xl font-semibold">
			{invoice.supplierInvoiceNumber ?? `#${invoice.id}`}
		</h1>
		<span data-testid="supplier-invoice-status">{supplierInvoiceStatusLabel(invoice.status)}</span>
	</div>

	<dl class="mb-6 grid grid-cols-2 gap-2 text-sm">
		<dt class="text-text-muted">{i18nMsg('supplier-invoices-col-date', 'Date')}</dt>
		<dd>{invoice.invoiceDate}</dd>
		<dt class="text-text-muted">{i18nMsg('supplier-invoices-col-due', 'Échéance')}</dt>
		<dd>{invoice.dueDate ?? '—'}</dd>
		<dt class="text-text-muted">{i18nMsg('supplier-invoices-col-total', 'TTC')}</dt>
		<dd>{formatSupplierInvoiceTotal(invoice.totalAmount)}</dd>
		{#if projectLabel}
			<dt class="text-text-muted">{i18nMsg('supplier-invoices-detail-project', 'Projet analytique')}</dt>
			<dd data-testid="supplier-invoice-project">{projectLabel}</dd>
		{/if}
		{#if invoice.creditorIban}
			<dt class="text-text-muted">IBAN</dt>
			<dd class="font-mono">{invoice.creditorIban}</dd>
		{/if}
		{#if invoice.paymentReference}
			<dt class="text-text-muted">{i18nMsg('supplier-invoices-detail-reference', 'Référence')}</dt>
			<dd class="font-mono">{invoice.paymentReference}</dd>
		{/if}
	</dl>

	<button
		class="mb-6 inline-block rounded border px-4 py-2 text-sm text-primary"
		data-testid="supplier-invoice-source-document"
		onclick={viewSourceDocument}
	>
		{i18nMsg('imported-supplier-invoices-view-source', 'Voir la facture d’origine')}
	</button>

	<table class="mb-6 w-full border-collapse text-sm">
		<thead>
			<tr class="border-b text-left text-text-muted">
				<th class="py-2">{i18nMsg('supplier-invoices-line-desc', 'Description')}</th>
				<th class="py-2 text-right">{i18nMsg('supplier-invoices-col-qty', 'Qté')}</th>
				<th class="py-2 text-right">{i18nMsg('supplier-invoices-line-ht', 'HT')}</th>
				<th class="py-2 text-right">{i18nMsg('supplier-invoices-col-vat', 'TVA')}</th>
				<!-- ⚠️ PAS `supplier-invoices-col-total` : cette colonne affiche `line.lineTotal`, qui
					 vaut `quantity × unit_price` HORS TAXE (`supplier_invoice.rs:89`), tandis que la clé
					 `-col-total` sert `invoice.totalAmount`, documenté TTC (`supplier_invoice.rs:29`).
					 Les deux ont longtemps partagé une clé sans dommage, parce qu'elle manquait des
					 quatre catalogues et que chaque site retombait sur SON repli. Entrer une valeur
					 unique au catalogue l'aurait imposée aux trois sites — « TTC » au-dessus de
					 montants HT. (Story 23-3, révélé par le moissonneur.) -->
				<th class="py-2 text-right">{i18nMsg('supplier-invoices-line-total', 'Total HT')}</th>
			</tr>
		</thead>
		<tbody>
			{#each invoice.lines as line (line.position)}
				<tr class="border-b">
					<td class="py-2">{line.description}</td>
					<td class="py-2 text-right">{line.quantity}</td>
					<td class="py-2 text-right">{formatSupplierInvoiceTotal(line.unitPrice)}</td>
					<td class="py-2 text-right">{line.vatRate}%</td>
					<td class="py-2 text-right">{formatSupplierInvoiceTotal(line.lineTotal)}</td>
				</tr>
			{/each}
		</tbody>
	</table>

	{#if invoice.status === 'open'}
		<div class="mb-6 rounded border border-border p-4" data-testid="supplier-invoice-pay">
			<h2 class="mb-3 font-medium">{i18nMsg('supplier-invoices-pay-title', 'Payer la facture')}</h2>
			<div class="mb-3 flex gap-4 text-sm">
				<label class="flex items-center gap-1">
					<input type="radio" bind:group={settlementType} value="bank_transfer" />
					{i18nMsg('supplier-invoices-pay-transfer', 'Virement bancaire')}
				</label>
				<label class="flex items-center gap-1">
					<input type="radio" bind:group={settlementType} value="internal_account" />
					{i18nMsg('supplier-invoices-pay-internal', 'Compte interne (caisse, carte…)')}
				</label>
			</div>

			{#if settlementType === 'bank_transfer'}
				<select
					class="mb-3 w-full rounded border px-2 py-1 text-sm"
					data-testid="pay-bank-account"
					bind:value={payBankAccountId}
				>
					<option value={null}>{i18nMsg('supplier-invoices-pay-bank-ph', 'Compte bancaire source')}</option>
					{#each bankAccounts as b (b.id)}
						<option value={b.id}>{b.bankName} — {b.iban}</option>
					{/each}
				</select>
			{:else}
				<select
					class="mb-3 w-full rounded border px-2 py-1 text-sm"
					data-testid="pay-internal-account"
					bind:value={payAccountId}
				>
					<option value={null}>{i18nMsg('supplier-invoices-pay-account-ph', 'Compte de contrepartie')}</option>
					{#each accounts as a (a.id)}
						<option value={a.id}>{a.number} {a.name}</option>
					{/each}
				</select>
			{/if}

			<label class="mb-3 block text-sm">
				{i18nMsg('supplier-invoices-pay-date', 'Date de règlement')}
				<input type="date" class="mt-1 w-full rounded border px-2 py-1" bind:value={paymentDate} />
			</label>

			{#if payError}
				<p class="mb-2 text-sm text-destructive" data-testid="supplier-invoice-pay-error">{payError}</p>
			{/if}

			<div class="flex gap-2">
				<button
					class="rounded bg-primary px-4 py-2 text-sm text-primary-foreground"
					data-testid="supplier-invoice-pay-submit"
					onclick={pay}
					disabled={paying}
				>
					{paying ? '…' : i18nMsg('supplier-invoices-pay-submit', 'Payer')}
				</button>
				<button
					class="rounded border px-4 py-2 text-sm text-destructive"
					data-testid="supplier-invoice-cancel"
					onclick={cancel}
				>
					{i18nMsg('supplier-invoices-cancel', 'Annuler la facture')}
				</button>
			</div>
		</div>
	{:else if invoice.status === 'paid'}
		<p class="text-sm text-text-muted" data-testid="supplier-invoice-paid-info">
			{i18nMsg('supplier-invoices-paid-info', 'Facture réglée.')}
			{invoice.paidAt ?? ''}
		</p>
	{/if}
{/if}
