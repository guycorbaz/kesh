<!--
  Story 8-1b — Upload + preview + confirm dans un seul composant
  (state machine inline : idle → file-selected → previewing → preview-shown → confirming → success/error).

  Pour scope v0.1, drop-zone + file input + preview table sont
  cohabités dans le même composant — le découpage en 5 sous-composants
  (BankImportPreviewTable, etc.) reste possible en post-v0.1 si l'UX
  veut isoler. Le data-testid pattern (Story 7-5/KF-008) est utilisé
  partout pour les E2E Playwright.
-->
<script lang="ts">
	import {
		previewBankImport,
		createBankImport,
	} from './bank-import.api';
	import type {
		BankImportPreviewResponse,
		BankImportResponse,
	} from './bank-import.types';
	import { isApiError } from '$lib/shared/utils/api-client';

	type Props = {
		bankAccounts: Array<{ id: number; bankName: string; iban: string }>;
		onSuccess?: (imported: BankImportResponse) => void;
	};
	let { bankAccounts, onSuccess }: Props = $props();

	let selectedAccountId = $state<number | null>(null);
	let selectedFile = $state<File | null>(null);
	let preview = $state<BankImportPreviewResponse | null>(null);
	let confirmBalanceMismatch = $state(false);
	let isLoading = $state(false);
	let errorCode = $state<string | null>(null);
	let errorMessage = $state<string | null>(null);
	let dragActive = $state(false);

	function reset(): void {
		selectedFile = null;
		preview = null;
		confirmBalanceMismatch = false;
		errorCode = null;
		errorMessage = null;
	}

	async function handleFileSelect(file: File): Promise<void> {
		if (!selectedAccountId) {
			errorCode = 'BANK_ACCOUNT_REQUIRED';
			errorMessage = 'Sélectionnez d\'abord un compte bancaire.';
			return;
		}
		selectedFile = file;
		errorCode = null;
		errorMessage = null;
		isLoading = true;
		try {
			preview = await previewBankImport(file, selectedAccountId);
		} catch (err) {
			if (isApiError(err)) {
				errorCode = err.code;
				errorMessage = err.message;
			} else {
				errorCode = 'UNKNOWN_ERROR';
				errorMessage = 'Erreur inattendue.';
			}
			preview = null;
		} finally {
			isLoading = false;
		}
	}

	async function handleConfirm(): Promise<void> {
		if (!selectedFile || !selectedAccountId) return;
		isLoading = true;
		errorCode = null;
		errorMessage = null;
		try {
			const result = await createBankImport(
				selectedFile,
				selectedAccountId,
				confirmBalanceMismatch,
			);
			onSuccess?.(result);
			reset();
		} catch (err) {
			if (isApiError(err)) {
				errorCode = err.code;
				errorMessage = err.message;
			} else {
				errorCode = 'UNKNOWN_ERROR';
				errorMessage = 'Erreur inattendue.';
			}
		} finally {
			isLoading = false;
		}
	}

	function handleDragOver(event: DragEvent): void {
		event.preventDefault();
		dragActive = true;
	}
	function handleDragLeave(): void {
		dragActive = false;
	}
	function handleDrop(event: DragEvent): void {
		event.preventDefault();
		dragActive = false;
		const file = event.dataTransfer?.files?.[0];
		if (file) {
			void handleFileSelect(file);
		}
	}
	function handleFileInput(event: Event): void {
		const input = event.currentTarget as HTMLInputElement;
		const file = input.files?.[0];
		if (file) {
			void handleFileSelect(file);
		}
	}
</script>

<section data-testid="bank-import-upload">
	<label class="block text-sm font-medium" for="bank-account-select">
		Compte bancaire cible
	</label>
	<select
		id="bank-account-select"
		data-testid="bank-account-select"
		bind:value={selectedAccountId}
		class="mt-1 block w-full rounded border-border bg-surface p-2 text-text"
	>
		<option value={null}>— Sélectionner un compte —</option>
		{#each bankAccounts as account (account.id)}
			<option value={account.id}>{account.bankName} ({account.iban})</option>
		{/each}
	</select>

	{#if !preview}
		<div
			data-testid="bank-import-drop-zone"
			role="button"
			tabindex="0"
			aria-label="Glisser-déposer un fichier CAMT.053 ou cliquer pour parcourir"
			class="mt-4 flex h-48 cursor-pointer items-center justify-center rounded border-2 border-dashed p-4 transition-colors"
			class:border-border={!dragActive}
			class:border-primary={dragActive}
			ondragover={handleDragOver}
			ondragleave={handleDragLeave}
			ondrop={handleDrop}
			onclick={() => document.getElementById('bank-import-file-input')?.click()}
			onkeydown={(e) => {
				if (e.key === 'Enter' || e.key === ' ') {
					document.getElementById('bank-import-file-input')?.click();
				}
			}}
		>
			<input
				id="bank-import-file-input"
				data-testid="bank-import-file-input"
				type="file"
				accept=".xml,application/xml"
				class="hidden"
				onchange={handleFileInput}
				disabled={!selectedAccountId}
			/>
			<p class="text-text-muted">
				Glissez votre fichier CAMT.053 ici ou cliquez pour parcourir
			</p>
		</div>
	{/if}

	{#if isLoading}
		<p class="mt-4 text-text-muted" data-testid="bank-import-loading">Chargement…</p>
	{/if}

	{#if errorMessage}
		<div
			class="mt-4 rounded border border-error bg-error-soft p-3"
			data-testid="bank-import-error"
			data-error-code={errorCode}
			role="alert"
		>
			<p class="text-error">{errorMessage}</p>
		</div>
	{/if}

	{#if preview}
		<div class="mt-6" data-testid="bank-import-preview">
			<h2 class="text-lg font-semibold">Prévisualisation</h2>
			<dl class="mt-2 grid grid-cols-2 gap-2 text-sm">
				<dt>IBAN</dt>
				<dd data-testid="preview-iban">{preview.selectedStatement.accountIban}</dd>
				<dt>Période</dt>
				<dd>{preview.selectedStatement.periodFrom} → {preview.selectedStatement.periodTo}</dd>
				<dt>Devise</dt>
				<dd>{preview.selectedStatement.currency}</dd>
				<dt>Solde ouverture</dt>
				<dd>{preview.selectedStatement.openingBalance ?? '—'}</dd>
				<dt>Solde clôture</dt>
				<dd>{preview.selectedStatement.closingBalance ?? '—'}</dd>
				<dt>Nombre de transactions</dt>
				<dd data-testid="preview-tx-count">{preview.transactions.length}</dd>
			</dl>

			{#if preview.warnings.length > 0}
				<div
					class="mt-4 rounded border border-warning bg-warning-soft p-3"
					data-testid="preview-warnings"
				>
					{#each preview.warnings as warn (warn)}
						<p data-warning-code={warn}>{warn}</p>
					{/each}
				</div>
			{/if}

			{#if preview.ignoredStatements.length > 0}
				<div class="mt-4" data-testid="preview-ignored-statements">
					<p class="text-sm text-text-muted">
						{preview.ignoredStatements.length} statement(s) ignoré(s) (autres IBAN dans le fichier).
					</p>
				</div>
			{/if}

			<table class="mt-4 w-full table-auto text-sm" data-testid="preview-tx-table">
				<thead>
					<tr>
						<th class="text-left">Date</th>
						<th class="text-right">Montant</th>
						<th class="text-left">Détails</th>
					</tr>
				</thead>
				<tbody>
					{#each preview.transactions as tx, i (i)}
						<tr data-testid="preview-tx-row">
							<td>{tx.bookingDate}</td>
							<td class="text-right">{tx.amount} {tx.currency}</td>
							<td>{tx.details}</td>
						</tr>
					{/each}
				</tbody>
			</table>

			{#if preview.warnings.includes('balance_mismatch')}
				<label class="mt-4 flex items-center gap-2 text-sm">
					<input
						type="checkbox"
						data-testid="confirm-balance-mismatch"
						bind:checked={confirmBalanceMismatch}
					/>
					Importer malgré l'écart de solde
				</label>
			{/if}

			<div class="mt-4 flex gap-2">
				<button
					type="button"
					data-testid="bank-import-confirm"
					class="rounded bg-primary px-4 py-2 text-white disabled:opacity-50"
					disabled={isLoading ||
						(preview.warnings.includes('balance_mismatch') && !confirmBalanceMismatch)}
					onclick={handleConfirm}
				>
					Confirmer l'import
				</button>
				<button
					type="button"
					data-testid="bank-import-cancel"
					class="rounded border border-border px-4 py-2"
					onclick={reset}
				>
					Annuler
				</button>
			</div>
		</div>
	{/if}
</section>
