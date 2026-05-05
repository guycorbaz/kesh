<!--
  Story 8-2 T6 — Formulaire create/edit profil CSV bancaire.

  Validation client-side basique (XOR amount/debit_credit, regex preview).
  La validation complète vit côté API (CsvProfile::validate) — le client
  ne peut pas garantir l'exhaustivité (collision indices, chrono format).
-->
<script lang="ts">
	import { i18nMsg } from '$lib/shared/utils/i18n.svelte';
	import { isApiError } from '$lib/shared/utils/api-client';
	import {
		createBankProfile,
		updateBankProfile,
	} from './bank-profile.api';
	import type {
		BankProfile,
		CreateOrUpdateBankProfileRequest,
	} from './bank-profile.types';

	type Props = {
		existing?: BankProfile;
		onSuccess?: (profile: BankProfile) => void;
	};
	let { existing, onSuccess }: Props = $props();

	let bankName = $state(existing?.bankName ?? '');
	let filenamePattern = $state(existing?.filenamePattern ?? '');
	let dateFormat = $state(existing?.dateFormat ?? '%Y-%m-%d');
	let decimalSeparator = $state(existing?.decimalSeparator ?? '.');
	let fieldSeparator = $state(existing?.fieldSeparator ?? ';');
	let encoding = $state(existing?.encoding ?? '');
	let headerRowCount = $state(existing?.headerRowCount ?? 1);

	// Column mapping (XOR amount vs debit_credit_split)
	let useDebitCreditSplit = $state(
		existing?.columnMapping.debit_credit_split !== undefined,
	);
	let dateIdx = $state(existing?.columnMapping.date ?? 0);
	let amountIdx = $state(existing?.columnMapping.amount ?? 1);
	let debitIdx = $state(existing?.columnMapping.debit_credit_split?.[0] ?? 1);
	let creditIdx = $state(existing?.columnMapping.debit_credit_split?.[1] ?? 2);
	let referenceIdx = $state<number | null>(existing?.columnMapping.reference ?? null);
	let detailsIdx = $state<number | null>(existing?.columnMapping.details ?? null);

	let isLoading = $state(false);
	let errorMessage = $state<string | null>(null);

	async function handleSubmit(event: Event): Promise<void> {
		event.preventDefault();
		errorMessage = null;
		isLoading = true;

		const columnMapping = useDebitCreditSplit
			? {
					date: dateIdx,
					debit_credit_split: [debitIdx, creditIdx] as [number, number],
					reference: referenceIdx ?? undefined,
					details: detailsIdx ?? undefined,
				}
			: {
					date: dateIdx,
					amount: amountIdx,
					reference: referenceIdx ?? undefined,
					details: detailsIdx ?? undefined,
				};

		const req: CreateOrUpdateBankProfileRequest = {
			bankName,
			filenamePattern: filenamePattern || null,
			columnMapping,
			dateFormat,
			decimalSeparator,
			fieldSeparator,
			encoding: encoding || null,
			headerRowCount,
		};

		try {
			const profile = existing
				? await updateBankProfile(existing.id, req)
				: await createBankProfile(req);
			onSuccess?.(profile);
		} catch (err) {
			if (isApiError(err)) {
				errorMessage = err.message;
			} else {
				errorMessage = 'Erreur inattendue.';
			}
		} finally {
			isLoading = false;
		}
	}
</script>

<form data-testid="bank-import-profile-form" onsubmit={handleSubmit}>
	<label class="block text-sm font-medium" for="bank-name-input">
		{i18nMsg('bank-import-profile-labels-bank-name', 'Nom de la banque')}
	</label>
	<input
		id="bank-name-input"
		data-testid="bank-name-input"
		type="text"
		bind:value={bankName}
		required
		maxlength="100"
		class="mt-1 block w-full rounded border-border bg-surface p-2 text-text"
	/>

	<label class="mt-4 block text-sm font-medium" for="filename-pattern-input">
		{i18nMsg('bank-import-profile-labels-filename-pattern', 'Pattern filename (regex)')}
		<span class="text-xs text-text-muted">
			{i18nMsg(
				'bank-import-profile-labels-filename-pattern-help',
				'Regex case-sensitive (utilisez `(?i)` pour case-insensitive)',
			)}
		</span>
	</label>
	<input
		id="filename-pattern-input"
		data-testid="filename-pattern-input"
		type="text"
		bind:value={filenamePattern}
		maxlength="200"
		class="mt-1 block w-full rounded border-border bg-surface p-2 text-text"
	/>

	<label class="mt-4 block text-sm font-medium" for="date-format-input">
		{i18nMsg('bank-import-profile-labels-date-format', 'Format date (chrono)')}
	</label>
	<input
		id="date-format-input"
		data-testid="date-format-input"
		type="text"
		bind:value={dateFormat}
		required
		maxlength="50"
		class="mt-1 block w-full rounded border-border bg-surface p-2 text-text"
	/>

	<div class="mt-4 grid grid-cols-2 gap-4">
		<div>
			<label class="block text-sm font-medium" for="field-separator-select">
				{i18nMsg('bank-import-profile-labels-field-separator', 'Séparateur champs')}
			</label>
			<select
				id="field-separator-select"
				data-testid="field-separator-select"
				bind:value={fieldSeparator}
				class="mt-1 block w-full rounded border-border bg-surface p-2 text-text"
			>
				<option value=";">; (point-virgule)</option>
				<option value=",">, (virgule)</option>
				<option value={"\t"}>\t (tabulation)</option>
			</select>
		</div>
		<div>
			<label class="block text-sm font-medium" for="decimal-separator-select">
				{i18nMsg('bank-import-profile-labels-decimal-separator', 'Séparateur décimal')}
			</label>
			<select
				id="decimal-separator-select"
				data-testid="decimal-separator-select"
				bind:value={decimalSeparator}
				class="mt-1 block w-full rounded border-border bg-surface p-2 text-text"
			>
				<option value=".">.</option>
				<option value=",">,</option>
			</select>
		</div>
	</div>

	<label class="mt-4 block text-sm font-medium" for="encoding-select">
		{i18nMsg('bank-import-profile-labels-encoding', 'Encodage (optionnel)')}
	</label>
	<select
		id="encoding-select"
		data-testid="encoding-select"
		bind:value={encoding}
		class="mt-1 block w-full rounded border-border bg-surface p-2 text-text"
	>
		<option value="">— Auto-détection —</option>
		<option value="UTF-8">UTF-8</option>
		<option value="ISO-8859-1">ISO-8859-1</option>
	</select>

	<label class="mt-4 block text-sm font-medium" for="header-row-count-input">
		{i18nMsg('bank-import-profile-labels-header-row-count', 'Nb lignes header (0-5)')}
	</label>
	<input
		id="header-row-count-input"
		data-testid="header-row-count-input"
		type="number"
		bind:value={headerRowCount}
		min="0"
		max="5"
		class="mt-1 block w-full rounded border-border bg-surface p-2 text-text"
	/>

	<fieldset class="mt-6 border border-border p-4">
		<legend class="text-sm font-medium">
			{i18nMsg('bank-import-profile-labels-column-mapping', 'Mapping colonnes (0-indexed)')}
		</legend>
		<label class="mt-2 flex items-center gap-2 text-sm">
			<input
				type="checkbox"
				data-testid="use-debit-credit-split"
				bind:checked={useDebitCreditSplit}
			/>
			{i18nMsg('bank-import-profile-labels-use-debit-credit-split', 'Colonnes débit/crédit séparées')}
		</label>

		<div class="mt-2 grid grid-cols-2 gap-2">
			<div>
				<label for="col-date-input">date</label>
				<input id="col-date-input" data-testid="col-date-input" type="number" min="0" bind:value={dateIdx} />
			</div>
			{#if useDebitCreditSplit}
				<div>
					<label for="col-debit-input">débit</label>
					<input id="col-debit-input" data-testid="col-debit-input" type="number" min="0" bind:value={debitIdx} />
				</div>
				<div>
					<label for="col-credit-input">crédit</label>
					<input id="col-credit-input" data-testid="col-credit-input" type="number" min="0" bind:value={creditIdx} />
				</div>
			{:else}
				<div>
					<label for="col-amount-input">montant</label>
					<input id="col-amount-input" data-testid="col-amount-input" type="number" min="0" bind:value={amountIdx} />
				</div>
			{/if}
			<div>
				<label for="col-reference-input">référence</label>
				<input id="col-reference-input" data-testid="col-reference-input" type="number" min="0" bind:value={referenceIdx} />
			</div>
			<div>
				<label for="col-details-input">détails</label>
				<input id="col-details-input" data-testid="col-details-input" type="number" min="0" bind:value={detailsIdx} />
			</div>
		</div>
	</fieldset>

	{#if errorMessage}
		<div class="mt-4 rounded border border-error bg-error-soft p-3" data-testid="bank-import-profile-error" role="alert">
			<p class="text-error">{errorMessage}</p>
		</div>
	{/if}

	<div class="mt-6 flex gap-2">
		<button
			type="submit"
			data-testid="bank-import-profile-submit"
			class="rounded bg-primary px-4 py-2 text-white disabled:opacity-50"
			disabled={isLoading}
		>
			{i18nMsg(existing ? 'bank-import-profile-labels-update' : 'bank-import-profile-labels-create', existing ? 'Mettre à jour' : 'Créer')}
		</button>
	</div>
</form>
