<script lang="ts">
	import { onMount } from 'svelte';
	import { Button } from '$lib/components/ui/button';
	import { Input } from '$lib/components/ui/input';
	import { notifyError, notifySuccess } from '$lib/shared/utils/notify';
	import { isApiError } from '$lib/shared/utils/api-client';
	import { authState } from '$lib/app/stores/auth.svelte';
	import {
		getInvoiceSettings,
		updateInvoiceSettings,
	} from '$lib/features/invoices/invoices.api';
	import type {
		InvoiceSettingsResponse,
		JournalCode,
	} from '$lib/features/invoices/invoices.types';
	import {
		previewInvoiceNumber,
		validateFormatTemplate,
	} from '$lib/features/invoices/invoice-number-format';
	import { fetchAccounts } from '$lib/features/accounts/accounts.api';
	import type { AccountResponse } from '$lib/features/accounts/accounts.types';

	const JOURNAL_CODES: JournalCode[] = ['Achats', 'Ventes', 'Banque', 'Caisse', 'OD'];

	let settings = $state<InvoiceSettingsResponse | null>(null);
	let accounts = $state<AccountResponse[]>([]);
	let loading = $state(true);
	let submitting = $state(false);
	let loadError = $state('');

	let format = $state('');
	let descriptionTemplate = $state('');
	let receivableId = $state<number | null>(null);
	let revenueId = $state<number | null>(null);
	let salesJournal = $state<JournalCode>('Ventes');
	let version = $state(0);

	let assetAccounts = $derived(
		accounts.filter((a) => a.active && a.accountType === 'Asset'),
	);
	let revenueAccounts = $derived(
		accounts.filter((a) => a.active && a.accountType === 'Revenue'),
	);

	let formatValidation = $derived(validateFormatTemplate(format));
	let formatPreview = $derived(
		formatValidation.ok ? previewInvoiceNumber(format, 2026, '2026', 1) : '',
	);

	let isAdmin = $derived(authState.currentUser?.role === 'Admin');

	onMount(async () => {
		try {
			const [s, a] = await Promise.all([getInvoiceSettings(), fetchAccounts(false)]);
			settings = s;
			accounts = a;
			format = s.invoiceNumberFormat;
			descriptionTemplate = s.journalEntryDescriptionTemplate;
			receivableId = s.defaultReceivableAccountId;
			revenueId = s.defaultRevenueAccountId;
			salesJournal = s.defaultSalesJournal;
			version = s.version;
		} catch (err) {
			if (isApiError(err)) loadError = err.message;
			else loadError = 'Erreur de chargement';
		} finally {
			loading = false;
		}
	});

	async function save() {
		if (!formatValidation.ok) {
			notifyError(formatValidation.error ?? 'Format invalide');
			return;
		}
		submitting = true;
		try {
			const updated = await updateInvoiceSettings({
				invoiceNumberFormat: format,
				defaultReceivableAccountId: receivableId,
				defaultRevenueAccountId: revenueId,
				defaultSalesJournal: salesJournal,
				journalEntryDescriptionTemplate: descriptionTemplate,
				version,
			});
			settings = updated;
			version = updated.version;
			notifySuccess('Configuration enregistrée');
		} catch (err) {
			if (isApiError(err)) {
				notifyError(err.message);
				if (err.code === 'OPTIMISTIC_LOCK_CONFLICT') {
					// Reload the current settings.
					try {
						const fresh = await getInvoiceSettings();
						settings = fresh;
						format = fresh.invoiceNumberFormat;
						descriptionTemplate = fresh.journalEntryDescriptionTemplate;
						receivableId = fresh.defaultReceivableAccountId;
						revenueId = fresh.defaultRevenueAccountId;
						salesJournal = fresh.defaultSalesJournal;
						version = fresh.version;
					} catch {
						// keep error toast
					}
				}
			} else {
				notifyError('Erreur lors de la sauvegarde');
			}
		} finally {
			submitting = false;
		}
	}
</script>

<svelte:head>
	<title>Paramètres — Facturation — Kesh</title>
</svelte:head>

<h1 class="mb-6 text-2xl font-semibold">Paramètres — Facturation</h1>

{#if !isAdmin}
	<p class="rounded-md border border-amber-400 bg-amber-50 px-4 py-3 text-sm text-amber-900">
		Accès réservé aux administrateurs.
	</p>
{:else if loading}
	<p class="text-sm text-text-muted">Chargement…</p>
{:else if loadError}
	<p class="text-sm text-destructive">{loadError}</p>
{:else if settings}
	<form
		class="space-y-6"
		onsubmit={(e) => {
			e.preventDefault();
			void save();
		}}
	>
		<section class="space-y-3 rounded-lg border border-border bg-white p-6 shadow-sm">
			<h2 class="text-lg font-semibold">Numérotation</h2>
			<div>
				<label class="mb-1 block text-sm font-medium" for="format">Format</label>
				<Input id="format" bind:value={format} placeholder="F-{'{YEAR}'}-{'{SEQ:04}'}" />
				<p class="mt-1 text-xs text-text-muted">
					Placeholders : <code>{'{YEAR}'}</code>, <code>{'{FY}'}</code>,
					<code>{'{SEQ}'}</code>, <code>{'{SEQ:NN}'}</code> (NN entre 1 et 10).
				</p>
				{#if formatValidation.ok}
					<p class="mt-1 text-sm">
						Aperçu : <span class="font-mono">{formatPreview}</span>
					</p>
				{:else}
					<p class="mt-1 text-sm text-destructive">{formatValidation.error}</p>
				{/if}
			</div>

			<div>
				<label class="mb-1 block text-sm font-medium" for="desc">Libellé de l'écriture comptable</label>
				<Input
					id="desc"
					bind:value={descriptionTemplate}
					placeholder="{'{YEAR}'}-{'{INVOICE_NUMBER}'}"
				/>
				<p class="mt-1 text-xs text-text-muted">
					Placeholders : <code>{'{YEAR}'}</code>, <code>{'{INVOICE_NUMBER}'}</code>,
					<code>{'{CONTACT_NAME}'}</code>.
				</p>
			</div>
		</section>

		<section class="space-y-3 rounded-lg border border-border bg-white p-6 shadow-sm">
			<h2 class="text-lg font-semibold">Comptes par défaut</h2>
			<div>
				<label class="mb-1 block text-sm font-medium" for="receivable">Compte créance client (Actif)</label>
				<select
					id="receivable"
					class="w-full rounded-md border border-border bg-white px-3 py-2 text-sm"
					bind:value={receivableId}
				>
					<option value={null}>— Sélectionner —</option>
					{#each assetAccounts as a (a.id)}
						<option value={a.id}>{a.number} — {a.name}</option>
					{/each}
				</select>
			</div>
			<div>
				<label class="mb-1 block text-sm font-medium" for="revenue">Compte produit (Revenue)</label>
				<select
					id="revenue"
					class="w-full rounded-md border border-border bg-white px-3 py-2 text-sm"
					bind:value={revenueId}
				>
					<option value={null}>— Sélectionner —</option>
					{#each revenueAccounts as a (a.id)}
						<option value={a.id}>{a.number} — {a.name}</option>
					{/each}
				</select>
			</div>
			<div>
				<label class="mb-1 block text-sm font-medium" for="journal">Journal</label>
				<select
					id="journal"
					class="w-full rounded-md border border-border bg-white px-3 py-2 text-sm"
					bind:value={salesJournal}
				>
					{#each JOURNAL_CODES as code (code)}
						<option value={code}>{code}</option>
					{/each}
				</select>
			</div>
		</section>

		<div class="flex justify-end">
			<Button type="submit" disabled={submitting || !formatValidation.ok}>
				{submitting ? 'Enregistrement…' : 'Enregistrer'}
			</Button>
		</div>
	</form>
{/if}
