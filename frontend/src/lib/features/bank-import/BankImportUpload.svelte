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
	import { i18nMsg } from '$lib/shared/utils/i18n.svelte';

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

	// Review code Pass 1 M7 : reset l'état (preview, fichier, erreurs) si
	// l'utilisateur change de bank_account après un upload — sinon le
	// preview / message d'erreur affichés correspondent à un compte
	// différent que celui maintenant sélectionné.
	let lastSeenAccountId: number | null = null;
	$effect(() => {
		if (lastSeenAccountId !== null && lastSeenAccountId !== selectedAccountId) {
			selectedFile = null;
			preview = null;
			confirmBalanceMismatch = false;
			errorCode = null;
			errorMessage = null;
		}
		lastSeenAccountId = selectedAccountId;
	});

	// Review code Pass 1 M9 : mapping warning code (raw API string) →
	// clé i18n. Le composant affiche la clé tant que l'i18n runtime
	// front n'est pas wired ici ; au moins le code n'apparaît plus
	// brut « balance_mismatch » dans l'UI.
	function warningLabel(code: string): string {
		// Mapping warning code (snake_case raw API) → clé i18n kebab-case.
		// Convention `bank-import-warnings-{slug}` (validate Pass 3 O1+O2).
		const key = `bank-import-warnings-${code.replace(/_/g, '-')}`;
		const fallbackByCode: Record<string, string> = {
			balance_mismatch: 'Solde de clôture incohérent.',
			unsupported_currency: 'Devise non supportée v0.1.',
		};
		return i18nMsg(key, fallbackByCode[code] ?? code);
	}

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
		if (!selectedFile || !selectedAccountId || !preview) return;
		// Review code Pass 1 M8 : guard anti-stale-preview. Si l'utilisateur
		// a changé le bank_account entre le preview et le confirm (le
		// reset $effect vide normalement preview, mais belt-and-suspenders),
		// vérifier que l'IBAN du preview correspond toujours à un account
		// existant côté liste.
		const previewIban = preview.selectedStatement.accountIban.replace(/\s/g, '').toUpperCase();
		const matchingAccount = bankAccounts.find(
			(a) => a.id === selectedAccountId && a.iban.replace(/\s/g, '').toUpperCase() === previewIban,
		);
		if (!matchingAccount) {
			errorCode = 'STALE_PREVIEW';
			errorMessage = "La sélection de compte a changé. Recharger le fichier.";
			return;
		}
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
		{i18nMsg('bank-import-labels-bank-account-selector', 'Compte bancaire cible')}
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
				{i18nMsg(
					'bank-import-labels-drop-zone',
					'Glissez votre fichier CAMT.053 ici ou cliquez pour parcourir',
				)}
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
			<h2 class="text-lg font-semibold">
				{i18nMsg('bank-import-labels-preview-title', 'Prévisualisation')}
			</h2>
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
						<p data-warning-code={warn}>{warningLabel(warn)}</p>
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
					{i18nMsg(
						'bank-import-labels-confirm-balance-mismatch',
						"Importer malgré l'écart de solde",
					)}
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
					{i18nMsg('bank-import-labels-confirm-import', "Confirmer l'import")}
				</button>
				<button
					type="button"
					data-testid="bank-import-cancel"
					class="rounded border border-border px-4 py-2"
					onclick={reset}
				>
					{i18nMsg('bank-import-labels-cancel', 'Annuler')}
				</button>
			</div>
		</div>
	{/if}
</section>
