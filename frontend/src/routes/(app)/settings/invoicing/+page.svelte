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
		validateDescriptionTemplate,
		validateFormatTemplate,
	} from '$lib/features/invoices/invoice-number-format';
	import { fetchAccounts } from '$lib/features/accounts/accounts.api';
	import type { AccountResponse } from '$lib/features/accounts/accounts.types';
	import { i18nMsg } from '$lib/shared/utils/i18n.svelte';

	const JOURNAL_CODES: JournalCode[] = ['Achats', 'Ventes', 'Banque', 'Caisse', 'OD'];

	// IDs DOM stables et HTTP-LAN-safe ($props.id() — pas de crypto.randomUUID,
	// indisponible hors contexte sécurisé sur déploiement HTTP NAS, cf. #145).
	const uid = $props.id();

	let settings = $state<InvoiceSettingsResponse | null>(null);
	let accounts = $state<AccountResponse[]>([]);
	let loading = $state(true);
	let submitting = $state(false);
	let loadError = $state('');

	let format = $state('');
	let descriptionTemplate = $state('');
	let receivableId = $state<number | null>(null);
	let revenueId = $state<number | null>(null);
	let vatPayableId = $state<number | null>(null);
	let vatRecoverableId = $state<number | null>(null);
	let vatDecompteId = $state<number | null>(null);
	let salesJournal = $state<JournalCode>('Ventes');
	let version = $state(0);

	// Story 14-3b : comptes de config de facturation → postés à la génération
	// d'écriture, donc filtrés `postable` (protège d'un rejet en aval).
	let assetAccounts = $derived(
		accounts.filter((a) => a.active && a.postable && a.accountType === 'Asset'),
	);
	let revenueAccounts = $derived(
		accounts.filter((a) => a.active && a.postable && a.accountType === 'Revenue'),
	);
	let liabilityAccounts = $derived(
		accounts.filter((a) => a.active && a.postable && a.accountType === 'Liability'),
	);

	let formatValidation = $derived(validateFormatTemplate(format));
	let formatPreview = $derived(
		formatValidation.ok ? previewInvoiceNumber(format, 2026, '2026', 1) : '',
	);
	let descriptionValidation = $derived(validateDescriptionTemplate(descriptionTemplate));

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
			vatPayableId = s.defaultVatPayableAccountId;
			vatRecoverableId = s.defaultVatRecoverableAccountId;
			vatDecompteId = s.defaultVatDecompteAccountId;
			salesJournal = s.defaultSalesJournal;
			version = s.version;
		} catch (err) {
			if (isApiError(err)) loadError = err.message;
			else loadError = i18nMsg('settings-invoicing-load-error', 'Erreur de chargement');
		} finally {
			loading = false;
		}
	});

	async function save() {
		if (!formatValidation.ok) {
			notifyError(
				formatValidation.error ?? i18nMsg('settings-invoicing-format-invalid', 'Format invalide'),
			);
			return;
		}
		if (!descriptionValidation.ok) {
			notifyError(
				descriptionValidation.error ??
					i18nMsg('settings-invoicing-description-invalid', 'Libellé invalide'),
			);
			return;
		}
		submitting = true;
		try {
			const updated = await updateInvoiceSettings({
				invoiceNumberFormat: format,
				defaultReceivableAccountId: receivableId,
				defaultRevenueAccountId: revenueId,
				defaultVatPayableAccountId: vatPayableId,
				defaultVatRecoverableAccountId: vatRecoverableId,
				defaultVatDecompteAccountId: vatDecompteId,
				defaultSalesJournal: salesJournal,
				journalEntryDescriptionTemplate: descriptionTemplate,
				version,
			});
			settings = updated;
			version = updated.version;
			notifySuccess(i18nMsg('settings-invoicing-save-success', 'Configuration enregistrée'));
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
						vatPayableId = fresh.defaultVatPayableAccountId;
						vatRecoverableId = fresh.defaultVatRecoverableAccountId;
						vatDecompteId = fresh.defaultVatDecompteAccountId;
						salesJournal = fresh.defaultSalesJournal;
						version = fresh.version;
					} catch {
						// keep error toast
					}
				}
			} else {
				notifyError(i18nMsg('settings-invoicing-save-error', 'Erreur lors de la sauvegarde'));
			}
		} finally {
			submitting = false;
		}
	}
</script>

<svelte:head>
	<title>{i18nMsg('settings-invoicing-title', 'Paramètres — Facturation')} — Kesh</title>
</svelte:head>

<h1 class="mb-6 text-2xl font-semibold">
	{i18nMsg('settings-invoicing-title', 'Paramètres — Facturation')}
</h1>

{#if !isAdmin}
	<p class="rounded-md border border-amber-400 bg-amber-50 px-4 py-3 text-sm text-amber-900">
		{i18nMsg('common-admin-only', 'Accès réservé aux administrateurs.')}
	</p>
{:else if loading}
	<p class="text-sm text-text-muted">{i18nMsg('common-loading', 'Chargement…')}</p>
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
			<h2 class="text-lg font-semibold">
				{i18nMsg('settings-invoicing-numbering-title', 'Numérotation')}
			</h2>
			<div>
				<label class="mb-1 block text-sm font-medium" for="format">
					{i18nMsg('settings-invoicing-format-label', 'Format de numérotation')}
				</label>
				<Input id="format" bind:value={format} placeholder="F-{'{YEAR}'}-{'{SEQ:04}'}" />
				<p class="mt-1 text-xs text-text-muted">
					{i18nMsg('settings-invoicing-format-help', 'Placeholders : {YEAR}, {FY}, {SEQ}, {SEQ:NN}')}
					{i18nMsg('settings-invoicing-seq-range', '(NN entre 1 et 10)')}
				</p>
				{#if formatValidation.ok}
					<p class="mt-1 text-sm">
						{i18nMsg('settings-invoicing-format-preview', 'Aperçu')} :
						<span class="font-mono">{formatPreview}</span>
					</p>
				{:else}
					<p class="mt-1 text-sm text-destructive">{formatValidation.error}</p>
				{/if}
			</div>

			<div>
				<label class="mb-1 block text-sm font-medium" for="desc">
					{i18nMsg('settings-invoicing-description-template', "Libellé de l'écriture comptable")}
				</label>
				<Input
					id="desc"
					bind:value={descriptionTemplate}
					placeholder="{'{YEAR}'}-{'{INVOICE_NUMBER}'}"
				/>
				<p class="mt-1 text-xs text-text-muted">
					{i18nMsg(
						'settings-invoicing-description-help',
						'Placeholders : {YEAR}, {INVOICE_NUMBER}, {CONTACT_NAME}.',
					)}
				</p>
				{#if !descriptionValidation.ok}
					<p class="mt-1 text-sm text-destructive">{descriptionValidation.error}</p>
				{/if}
			</div>
		</section>

		<section class="space-y-3 rounded-lg border border-border bg-white p-6 shadow-sm">
			<h2 class="text-lg font-semibold">
				{i18nMsg('settings-invoicing-default-accounts-title', 'Comptes par défaut')}
			</h2>
			<div>
				<label class="mb-1 block text-sm font-medium" for="receivable">
					{i18nMsg('settings-invoicing-receivable-account', 'Compte créance client (Actif)')}
				</label>
				<select
					id="receivable"
					class="w-full rounded-md border border-border bg-white px-3 py-2 text-sm"
					bind:value={receivableId}
				>
					<option value={null}>{i18nMsg('settings-invoicing-select-none', '— Sélectionner —')}</option>
					{#each assetAccounts as a (a.id)}
						<option value={a.id}>{a.number} — {a.name}</option>
					{/each}
				</select>
			</div>
			<div>
				<label class="mb-1 block text-sm font-medium" for="revenue">
					{i18nMsg('settings-invoicing-revenue-account', 'Compte produit (Revenue)')}
				</label>
				<select
					id="revenue"
					class="w-full rounded-md border border-border bg-white px-3 py-2 text-sm"
					bind:value={revenueId}
				>
					<option value={null}>{i18nMsg('settings-invoicing-select-none', '— Sélectionner —')}</option>
					{#each revenueAccounts as a (a.id)}
						<option value={a.id}>{a.number} — {a.name}</option>
					{/each}
				</select>
			</div>
			<div>
				<label class="mb-1 block text-sm font-medium" for="journal">{i18nMsg('settings-invoicing-journal', 'Journal')}</label>
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

		<section class="space-y-3 rounded-lg border border-border bg-white p-6 shadow-sm">
			<h2 class="text-lg font-semibold">
				{i18nMsg('invoices-settings-vat-accounts-title', 'Comptes TVA')}
			</h2>
			<p class="text-xs text-text-muted">
				{i18nMsg(
					'invoices-settings-vat-accounts-hint',
					'Comptes utilisés pour la comptabilisation de la TVA (préparé pour le décompte AFC).',
				)}
			</p>
			<div>
				<label class="mb-1 block text-sm font-medium" for="{uid}-vat-payable">
					{i18nMsg('invoices-settings-vat-payable', 'Compte TVA due (Passif)')}
				</label>
				<select
					id="{uid}-vat-payable"
					class="w-full rounded-md border border-border bg-white px-3 py-2 text-sm"
					bind:value={vatPayableId}
				>
					<option value={null}>{i18nMsg('settings-invoicing-select-none', '— Sélectionner —')}</option>
					{#each liabilityAccounts as a (a.id)}
						<option value={a.id}>{a.number} — {a.name}</option>
					{/each}
				</select>
			</div>
			<div>
				<label class="mb-1 block text-sm font-medium" for="{uid}-vat-recoverable">
					{i18nMsg('invoices-settings-vat-recoverable', 'Compte TVA récupérable (Actif)')}
				</label>
				<select
					id="{uid}-vat-recoverable"
					class="w-full rounded-md border border-border bg-white px-3 py-2 text-sm"
					bind:value={vatRecoverableId}
				>
					<option value={null}>{i18nMsg('settings-invoicing-select-none', '— Sélectionner —')}</option>
					{#each assetAccounts as a (a.id)}
						<option value={a.id}>{a.number} — {a.name}</option>
					{/each}
				</select>
			</div>
			<div>
				<label class="mb-1 block text-sm font-medium" for="{uid}-vat-decompte">
					{i18nMsg('invoices-settings-vat-decompte', 'Compte de décompte TVA (Passif)')}
				</label>
				<select
					id="{uid}-vat-decompte"
					class="w-full rounded-md border border-border bg-white px-3 py-2 text-sm"
					bind:value={vatDecompteId}
				>
					<option value={null}>{i18nMsg('settings-invoicing-select-none', '— Sélectionner —')}</option>
					{#each liabilityAccounts as a (a.id)}
						<option value={a.id}>{a.number} — {a.name}</option>
					{/each}
				</select>
			</div>
		</section>

		<div class="flex justify-end">
			<Button type="submit" disabled={submitting || !formatValidation.ok || !descriptionValidation.ok}>
				{submitting
					? i18nMsg('common-saving', 'Enregistrement…')
					: i18nMsg('settings-invoicing-save', 'Enregistrer')}
			</Button>
		</div>
	</form>
{/if}
