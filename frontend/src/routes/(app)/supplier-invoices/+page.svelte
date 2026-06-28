<script lang="ts">
	import { onMount } from 'svelte';
	import { goto } from '$app/navigation';
	import {
		createSupplierInvoice,
		listSupplierInvoices,
	} from '$lib/features/supplier-invoices/supplier-invoices.api';
	import {
		formatSupplierInvoiceTotal,
		supplierInvoiceStatusLabel,
	} from '$lib/features/supplier-invoices/supplier-invoice-helpers';
	import type {
		CreateSupplierInvoiceLineRequest,
		SupplierInvoiceListItemResponse,
	} from '$lib/features/supplier-invoices/supplier-invoices.types';
	import { listContacts } from '$lib/features/contacts/contacts.api';
	import type { ContactResponse } from '$lib/features/contacts/contacts.types';
	import { fetchAccounts } from '$lib/features/accounts/accounts.api';
	import type { AccountResponse } from '$lib/features/accounts/accounts.types';
	import { listVatRates } from '$lib/features/vat-rates/vat-rates.api';
	import type { VatRateResponse } from '$lib/features/vat-rates/vat-rates.types';
	import { isApiError } from '$lib/shared/utils/api-client';
	import { i18nMsg } from '$lib/shared/utils/i18n.svelte';

	let items = $state<SupplierInvoiceListItemResponse[]>([]);
	let loading = $state(true);
	let errorMsg = $state('');

	// Données du formulaire.
	let suppliers = $state<ContactResponse[]>([]);
	let expenseAccounts = $state<AccountResponse[]>([]);
	let vatRates = $state<VatRateResponse[]>([]);

	let showForm = $state(false);
	let saving = $state(false);
	let formError = $state('');

	const today = new Date().toISOString().slice(0, 10);
	let fContactId = $state<number | null>(null);
	let fNumber = $state('');
	let fInvoiceDate = $state(today);
	let fDueDate = $state('');
	let fCreditorIban = $state('');
	let fPaymentReference = $state('');
	let fLines = $state<CreateSupplierInvoiceLineRequest[]>([
		{ description: '', quantity: '1', unitPrice: '', vatRate: '0', expenseAccountId: 0 },
	]);

	async function reload() {
		const res = await listSupplierInvoices({ limit: 100 });
		items = res.items;
	}

	onMount(async () => {
		try {
			const [, contactsRes, accountsRes, ratesRes] = await Promise.all([
				reload(),
				listContacts({ isSupplier: true, limit: 200 }),
				fetchAccounts(),
				listVatRates(),
			]);
			suppliers = contactsRes.items;
			expenseAccounts = accountsRes.filter((a) => a.accountType === 'Expense' && a.active);
			vatRates = ratesRes;
		} catch (err) {
			if (isApiError(err)) errorMsg = err.message;
		} finally {
			loading = false;
		}
	});

	function addLine() {
		fLines = [
			...fLines,
			{ description: '', quantity: '1', unitPrice: '', vatRate: '0', expenseAccountId: 0 },
		];
	}

	function removeLine(idx: number) {
		fLines = fLines.filter((_, i) => i !== idx);
	}

	function resetForm() {
		fContactId = null;
		fNumber = '';
		fInvoiceDate = today;
		fDueDate = '';
		fCreditorIban = '';
		fPaymentReference = '';
		fLines = [{ description: '', quantity: '1', unitPrice: '', vatRate: '0', expenseAccountId: 0 }];
		formError = '';
	}

	async function submit() {
		formError = '';
		if (!fContactId) {
			formError = i18nMsg('supplier-invoices-err-supplier', 'Sélectionnez un fournisseur.');
			return;
		}
		if (
			fLines.some(
				(l) =>
					!l.description.trim() ||
					!l.unitPrice ||
					Number(l.unitPrice) <= 0 ||
					!l.quantity ||
					Number(l.quantity) <= 0 ||
					!l.expenseAccountId,
			)
		) {
			formError = i18nMsg(
				'supplier-invoices-err-lines',
				'Chaque ligne requiert une description, un montant et un compte de charge.',
			);
			return;
		}
		saving = true;
		try {
			await createSupplierInvoice({
				contactId: fContactId,
				supplierInvoiceNumber: fNumber.trim() || null,
				invoiceDate: fInvoiceDate,
				dueDate: fDueDate || null,
				creditorIban: fCreditorIban.trim() || null,
				paymentReference: fPaymentReference.trim() || null,
				lines: fLines.map((l) => ({
					description: l.description.trim(),
					quantity: l.quantity || '1',
					unitPrice: l.unitPrice,
					vatRate: l.vatRate || '0',
					expenseAccountId: Number(l.expenseAccountId),
				})),
			});
			resetForm();
			showForm = false;
			await reload();
		} catch (err) {
			if (isApiError(err)) formError = err.message;
		} finally {
			saving = false;
		}
	}
</script>

<svelte:head>
	<title>{i18nMsg('supplier-invoices-title', 'Factures fournisseurs')} — Kesh</title>
</svelte:head>

<div class="mb-6 flex items-center justify-between">
	<h1 class="text-2xl font-semibold">
		{i18nMsg('supplier-invoices-title', 'Factures fournisseurs')}
	</h1>
	<button
		class="rounded bg-primary px-4 py-2 text-sm text-primary-foreground"
		data-testid="supplier-invoice-new"
		onclick={() => (showForm = !showForm)}
	>
		{showForm
			? i18nMsg('supplier-invoices-form-close', 'Fermer')
			: i18nMsg('supplier-invoices-new', 'Enregistrer une facture')}
	</button>
</div>

{#if showForm}
	<form
		class="mb-8 space-y-4 rounded border border-border p-4"
		data-testid="supplier-invoice-form"
		onsubmit={(e) => {
			e.preventDefault();
			submit();
		}}
	>
		<div class="grid grid-cols-2 gap-4">
			<label class="block text-sm">
				{i18nMsg('supplier-invoices-field-supplier', 'Fournisseur')}
				<select class="mt-1 w-full rounded border px-2 py-1" bind:value={fContactId}>
					<option value={null}>—</option>
					{#each suppliers as s (s.id)}
						<option value={s.id}>{s.name}</option>
					{/each}
				</select>
			</label>
			<label class="block text-sm">
				{i18nMsg('supplier-invoices-field-number', 'N° facture fournisseur')}
				<input class="mt-1 w-full rounded border px-2 py-1" bind:value={fNumber} />
			</label>
			<label class="block text-sm">
				{i18nMsg('supplier-invoices-field-date', 'Date de facture')}
				<input type="date" class="mt-1 w-full rounded border px-2 py-1" bind:value={fInvoiceDate} />
			</label>
			<label class="block text-sm">
				{i18nMsg('supplier-invoices-field-due', 'Échéance')}
				<input type="date" class="mt-1 w-full rounded border px-2 py-1" bind:value={fDueDate} />
			</label>
			<label class="block text-sm">
				{i18nMsg('supplier-invoices-field-iban', 'IBAN / QR-IBAN (optionnel)')}
				<input class="mt-1 w-full rounded border px-2 py-1" bind:value={fCreditorIban} />
			</label>
			<label class="block text-sm">
				{i18nMsg('supplier-invoices-field-reference', 'Référence (optionnel)')}
				<input class="mt-1 w-full rounded border px-2 py-1" bind:value={fPaymentReference} />
			</label>
		</div>

		<div>
			<div class="mb-2 text-sm font-medium">{i18nMsg('supplier-invoices-lines', 'Lignes')}</div>
			{#each fLines as line, idx (idx)}
				<div class="mb-2 grid grid-cols-12 gap-2">
					<input
						class="col-span-4 rounded border px-2 py-1 text-sm"
						placeholder={i18nMsg('supplier-invoices-line-desc', 'Description')}
						bind:value={line.description}
					/>
					<input
						class="col-span-1 rounded border px-2 py-1 text-sm"
						placeholder="Qté"
						bind:value={line.quantity}
					/>
					<input
						class="col-span-2 rounded border px-2 py-1 text-sm"
						placeholder={i18nMsg('supplier-invoices-line-ht', 'HT')}
						bind:value={line.unitPrice}
					/>
					<select class="col-span-2 rounded border px-2 py-1 text-sm" bind:value={line.vatRate}>
						<option value="0">0%</option>
						{#each vatRates as r (r.category)}
							<option value={r.rate}>{r.rate}%</option>
						{/each}
					</select>
					<select
						class="col-span-2 rounded border px-2 py-1 text-sm"
						bind:value={line.expenseAccountId}
					>
						<option value={0}>{i18nMsg('supplier-invoices-line-account', 'Compte')}</option>
						{#each expenseAccounts as a (a.id)}
							<option value={a.id}>{a.number} {a.name}</option>
						{/each}
					</select>
					<button
						type="button"
						class="col-span-1 text-destructive"
						onclick={() => removeLine(idx)}
						disabled={fLines.length === 1}>✕</button
					>
				</div>
			{/each}
			<button type="button" class="text-sm text-primary" onclick={addLine}>
				+ {i18nMsg('supplier-invoices-add-line', 'Ajouter une ligne')}
			</button>
		</div>

		{#if formError}
			<p class="text-sm text-destructive" data-testid="supplier-invoice-form-error">{formError}</p>
		{/if}

		<button
			type="submit"
			class="rounded bg-primary px-4 py-2 text-sm text-primary-foreground"
			data-testid="supplier-invoice-submit"
			disabled={saving}
		>
			{saving ? '…' : i18nMsg('supplier-invoices-save', 'Enregistrer')}
		</button>
	</form>
{/if}

{#if loading}
	<p class="text-sm text-text-muted">Chargement…</p>
{:else if errorMsg}
	<p class="text-sm text-destructive">{errorMsg}</p>
{:else if items.length === 0}
	<p class="text-sm text-text-muted">
		{i18nMsg('supplier-invoices-empty', 'Aucune facture fournisseur enregistrée.')}
	</p>
{:else}
	<table class="w-full border-collapse text-sm" data-testid="supplier-invoices-table">
		<thead>
			<tr class="border-b text-left text-text-muted">
				<th class="py-2">{i18nMsg('supplier-invoices-col-number', 'N°')}</th>
				<th class="py-2">{i18nMsg('supplier-invoices-col-supplier', 'Fournisseur')}</th>
				<th class="py-2">{i18nMsg('supplier-invoices-col-date', 'Date')}</th>
				<th class="py-2">{i18nMsg('supplier-invoices-col-due', 'Échéance')}</th>
				<th class="py-2">{i18nMsg('supplier-invoices-col-status', 'Statut')}</th>
				<th class="py-2 text-right">{i18nMsg('supplier-invoices-col-total', 'TTC')}</th>
			</tr>
		</thead>
		<tbody>
			{#each items as inv (inv.id)}
				<tr
					class="cursor-pointer border-b hover:bg-surface-hover"
					data-testid="supplier-invoice-row"
					onclick={() => goto(`/supplier-invoices/${inv.id}`)}
				>
					<td class="py-2 font-mono">{inv.supplierInvoiceNumber ?? `#${inv.id}`}</td>
					<td class="py-2">{inv.contactName}</td>
					<td class="py-2">{inv.invoiceDate}</td>
					<td class="py-2">{inv.dueDate ?? '—'}</td>
					<td class="py-2">{supplierInvoiceStatusLabel(inv.status)}</td>
					<td class="py-2 text-right">{formatSupplierInvoiceTotal(inv.totalAmount)}</td>
				</tr>
			{/each}
		</tbody>
	</table>
{/if}
