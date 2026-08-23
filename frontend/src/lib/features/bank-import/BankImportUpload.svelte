<!--
  Story 8-1b — Upload + preview + confirm dans un seul composant
  (state machine inline : idle → file-selected → previewing → preview-shown → confirming → success/error).

  Story 8-3 — extension :
  - 4 nouveaux panneaux warnings : duplicateFile, duplicateLines, invalidLines, encodingMismatch
  - 3 nouveaux flags confirm : confirmDuplicateFile, confirmDuplicateLines (skip|import), confirmPartialImport
  - KF #70 closure : BankProfileSelector + confirmEncodingMismatch wiring
-->
<script lang="ts">
	import { onMount } from 'svelte';
	import { previewBankImport, createBankImport } from './bank-import.api';
	import type {
		BankImportPreviewResponse,
		BankImportResponse,
	} from './bank-import.types';
	import type { BankProfile } from './bank-profile.types';
	import { listBankProfiles } from './bank-profile.api';
	import BankProfileSelector from './BankProfileSelector.svelte';
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
	let confirmDuplicateFile = $state(false);
	let confirmDuplicateLines = $state<'skip' | 'import'>('skip');
	let confirmPartialImport = $state(false);
	let confirmEncodingMismatch = $state(false);
	let bankProfileId = $state<number | null>(null);
	let availableProfiles = $state<BankProfile[]>([]);
	let isLoading = $state(false);
	let errorCode = $state<string | null>(null);
	let errorMessage = $state<string | null>(null);
	let dragActive = $state(false);

	onMount(async () => {
		try {
			const list = await listBankProfiles(1, 50);
			availableProfiles = list.items;
		} catch {
			availableProfiles = [];
		}
	});

	let lastSeenAccountId: number | null = null;
	$effect(() => {
		if (lastSeenAccountId !== null && lastSeenAccountId !== selectedAccountId) {
			reset();
		}
		lastSeenAccountId = selectedAccountId;
	});

	function reset(): void {
		selectedFile = null;
		preview = null;
		confirmBalanceMismatch = false;
		confirmDuplicateFile = false;
		confirmDuplicateLines = 'skip';
		confirmPartialImport = false;
		confirmEncodingMismatch = false;
		errorCode = null;
		errorMessage = null;
	}

	function buildFlags() {
		return {
			confirmBalanceMismatch,
			bankProfileId: bankProfileId ?? undefined,
			confirmEncodingMismatch,
			confirmDuplicateFile,
			confirmDuplicateLines,
			confirmPartialImport,
		};
	}

	async function handleFileSelect(file: File): Promise<void> {
		if (!selectedAccountId) {
			errorCode = 'BANK_ACCOUNT_REQUIRED';
			errorMessage = "Sélectionnez d'abord un compte bancaire.";
			return;
		}
		// F1 (Pass 3 review) — réinitialise les confirm flags quand un
		// nouveau fichier est sélectionné sur le MÊME compte (le `$effect`
		// au-dessus ne déclenche `reset()` que sur changement de
		// `selectedAccountId`). Sans ça, un user qui aurait coché
		// `confirmDuplicateLines=import` sur un fichier #1 se retrouverait
		// silencieusement avec `import` actif sur un fichier #2 où il ne
		// l'a pas explicitement choisi → bug data-integrity latent.
		confirmBalanceMismatch = false;
		confirmDuplicateFile = false;
		confirmDuplicateLines = 'skip';
		confirmPartialImport = false;
		confirmEncodingMismatch = false;
		selectedFile = file;
		errorCode = null;
		errorMessage = null;
		isLoading = true;
		try {
			preview = await previewBankImport(file, selectedAccountId, {
				bankProfileId: bankProfileId ?? undefined,
			});
			// Pré-sélectionne l'auto-matched profile (KF #70).
			if (
				bankProfileId === null &&
				preview.csvProfileMatch?.autoMatched &&
				preview.csvProfileMatch?.profileId
			) {
				bankProfileId = preview.csvProfileMatch.profileId;
			}
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
		// L3 (Pass 1 review) — la garde anti-IDOR `matchingAccount` qui
		// vérifiait `preview.selectedStatement.accountIban === selectedIban`
		// (ajoutée en review code Pass 1 8-1b M8) a été retirée en 8-3 :
		// désormais le `$effect` reset détaille les changements
		// `selectedAccountId` (cf. ligne ~50) et purge `preview` avant
		// que l'utilisateur puisse cliquer Confirmer. La garde devient
		// redondante, mais on documente la raison ici pour qu'un
		// futur reviewer ne la restaure pas par réflexe.
		if (!selectedFile || !selectedAccountId || !preview) return;
		isLoading = true;
		errorCode = null;
		errorMessage = null;
		try {
			const result = await createBankImport(selectedFile, selectedAccountId, buildFlags());
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

	// Helper : confirm-button disabled if any required confirm checkbox is unticked.
	function buttonDisabled(p: BankImportPreviewResponse): boolean {
		const w = p.warnings;
		if (w.balanceMismatch && !confirmBalanceMismatch) return true;
		if (w.duplicateFile && !confirmDuplicateFile) return true;
		if (w.invalidLines && !confirmPartialImport) return true;
		if (w.encodingMismatch && !confirmEncodingMismatch) return true;
		// L2 (Pass 1 review) — `unsupportedCurrency` est un blocage absolu
		// côté backend (422 sans flag override). Désactiver le bouton ici
		// évite l'aller-retour 422 et le faux affordance.
		if (w.unsupportedCurrency) return true;
		return false;
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

	{#if availableProfiles.length > 0}
		<div class="mt-4">
			<BankProfileSelector
				profiles={availableProfiles}
				autoMatchedId={preview?.csvProfileMatch?.profileId ?? null}
				value={bankProfileId}
				onChange={(id) => (bankProfileId = id)}
			/>
		</div>
	{/if}

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
				accept=".xml,application/xml,.csv,text/csv,.txt"
				class="hidden"
				onchange={handleFileInput}
				disabled={!selectedAccountId}
			/>
			<p class="text-text-muted">
				{i18nMsg(
					'bank-import-labels-drop-zone',
					'Glissez votre fichier CAMT.053 ou CSV ici ou cliquez pour parcourir',
				)}
			</p>
		</div>
		<p class="mt-2 text-xs text-text-muted">
			<a href="/bank-import/profiles" data-testid="bank-import-profiles-link" class="underline">
				{i18nMsg('bank-import-profile-labels-page-title', 'Profils bancaires CSV')}
			</a>
		</p>
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

			<!-- Story 8-3 — panneau duplicate file -->
			{#if preview.warnings.duplicateFile}
				<div
					class="mt-4 rounded border border-warning bg-warning-soft p-3"
					data-testid="warning-duplicate-file"
				>
					<p class="font-medium">
						{i18nMsg(
							'bank-import-warnings-duplicate-file',
							'Ce fichier a déjà été importé.',
						)}
					</p>
					<p class="text-sm text-text-muted">
						<a
							href="/bank-import/{preview.warnings.duplicateFile.existingImportId}"
							data-testid="warning-duplicate-file-existing-link"
							class="underline"
						>
							{preview.warnings.duplicateFile.existingFilename}
						</a>
						— {preview.warnings.duplicateFile.existingImportedAt}
					</p>
					<label class="mt-2 flex items-center gap-2 text-sm">
						<input
							type="checkbox"
							data-testid="confirm-duplicate-file"
							bind:checked={confirmDuplicateFile}
						/>
						{i18nMsg(
							'bank-import-labels-confirm-duplicate-file',
							'Importer malgré le fichier déjà importé',
						)}
					</label>
				</div>
			{/if}

			<!-- Story 8-3 — panneau duplicate lines -->
			{#if preview.warnings.duplicateLines.length > 0}
				<div
					class="mt-4 rounded border border-warning bg-warning-soft p-3"
					data-testid="warning-duplicate-lines"
				>
					<p class="font-medium">
						{preview.warnings.duplicateLines.length}
						{i18nMsg(
							'bank-import-warnings-duplicate-lines-summary',
							'transactions chevauchent un import précédent.',
						)}
					</p>
					<table class="mt-2 w-full table-auto text-sm">
						<thead>
							<tr>
								<th class="text-left">Index</th>
								<th class="text-left">ID existant</th>
								<th class="text-left">Clé</th>
							</tr>
						</thead>
						<tbody>
							{#each preview.warnings.duplicateLines as dup (dup.newIndex)}
								<tr data-testid="warning-duplicate-lines-row">
									<td>{dup.newIndex}</td>
									<td>{dup.existingTransactionId}</td>
									<td><code>{dup.key}</code></td>
								</tr>
							{/each}
						</tbody>
					</table>
					<fieldset class="mt-2">
						<legend class="text-sm font-medium">
							{i18nMsg(
								'bank-import-labels-confirm-duplicate-lines',
								'Comportement face aux doublons',
							)}
						</legend>
						<label class="flex items-center gap-2 text-sm">
							<input
								type="radio"
								name="confirm-duplicate-lines"
								value="skip"
								data-testid="confirm-duplicate-lines-skip"
								bind:group={confirmDuplicateLines}
							/>
							{i18nMsg(
								'bank-import-labels-confirm-duplicate-lines-skip',
								'Ignorer les doublons (par défaut)',
							)}
						</label>
						<label class="flex items-center gap-2 text-sm">
							<input
								type="radio"
								name="confirm-duplicate-lines"
								value="import"
								data-testid="confirm-duplicate-lines-import"
								bind:group={confirmDuplicateLines}
							/>
							{i18nMsg(
								'bank-import-labels-confirm-duplicate-lines-import',
								'Importer quand même',
							)}
						</label>
					</fieldset>
				</div>
			{/if}

			<!-- Story 8-3 — panneau invalid lines (CSV partial) -->
			{#if preview.warnings.invalidLines}
				<div
					class="mt-4 rounded border border-warning bg-warning-soft p-3"
					data-testid="warning-invalid-lines"
				>
					<p class="font-medium">
						{preview.warnings.invalidLines.totalErrors}
						{i18nMsg(
							'bank-import-warnings-invalid-lines-summary',
							'lignes invalides détectées',
						)}
					</p>
					{#if preview.warnings.invalidLines.truncated}
						<p class="text-sm text-text-muted">
							{i18nMsg(
								'bank-import-warnings-invalid-lines-truncated',
								'Premières 100 erreurs affichées (cap atteint).',
							)}
						</p>
					{/if}
					<div class="mt-2 max-h-64 overflow-y-auto">
						<table class="w-full table-auto text-sm">
							<thead>
								<tr>
									<th class="text-left">Ligne</th>
									<th class="text-left">Code</th>
									<th class="text-left">Valeur</th>
								</tr>
							</thead>
							<tbody>
								{#each preview.warnings.invalidLines.lines as line, i (i)}
									<tr data-testid="warning-invalid-lines-row">
										<td>{line.line}</td>
										<td><code>{line.code}</code></td>
										<td>{line.value ?? '—'}</td>
									</tr>
								{/each}
							</tbody>
						</table>
					</div>
					<label class="mt-2 flex items-center gap-2 text-sm">
						<input
							type="checkbox"
							data-testid="confirm-partial-import"
							bind:checked={confirmPartialImport}
						/>
						{i18nMsg(
							'bank-import-labels-confirm-partial-import',
							'Importer les lignes valides quand même',
						)}
					</label>
				</div>
			{/if}

			<!-- Story 8-2/8-3 — panneau encoding mismatch (KF #70) -->
			{#if preview.warnings.encodingMismatch}
				<div
					class="mt-4 rounded border border-warning bg-warning-soft p-3"
					data-testid="warning-encoding-mismatch"
				>
					<p class="font-medium">
						{i18nMsg(
							'bank-import-warnings-encoding-mismatch',
							"L'encodage détecté diffère du profil.",
						)}
					</p>
					<p class="text-sm text-text-muted">
						{preview.warnings.encodingMismatch.profile} → {preview.warnings.encodingMismatch
							.detected}
					</p>
					<label class="mt-2 flex items-center gap-2 text-sm">
						<input
							type="checkbox"
							data-testid="confirm-encoding-mismatch"
							bind:checked={confirmEncodingMismatch}
						/>
						{i18nMsg(
							'bank-import-labels-confirm-encoding-mismatch',
							"Importer avec l'encodage détecté",
						)}
					</label>
				</div>
			{/if}

			<!-- 8-1b balance mismatch -->
			{#if preview.warnings.balanceMismatch}
				<div
					class="mt-4 rounded border border-warning bg-warning-soft p-3"
					data-testid="warning-balance-mismatch"
				>
					<p class="font-medium">
						{i18nMsg(
							'bank-import-warnings-balance-mismatch',
							'Solde de clôture incohérent.',
						)}
					</p>
					<p class="text-sm text-text-muted">
						Δ = {preview.warnings.balanceMismatch.diff}
					</p>
					<label class="mt-2 flex items-center gap-2 text-sm">
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
				</div>
			{/if}

			<!-- 8-1b unsupported currency (read-only) -->
			{#if preview.warnings.unsupportedCurrency}
				<div
					class="mt-4 rounded border border-warning bg-warning-soft p-3"
					data-testid="warning-unsupported-currency"
				>
					<p>
						{i18nMsg(
							'bank-import-warnings-unsupported-currency',
							'Devise non supportée v0.1.',
						)}
						{preview.warnings.unsupportedCurrency.currency}
					</p>
				</div>
			{/if}

			<!-- Informational warnings (CSV multiple matches, auto-matched, etc.) -->
			<!-- M8 (Pass 1 review) — applique i18n via i18nMsg avec fallback sur
			     le code brut. Sans cette transformation, l'utilisateur voyait
			     `bank_csv_profile_auto_matched` en clair. La clé i18n suit le
			     pattern `bank-import-info-<snake_to_kebab(info)>`. -->
			{#if (preview.warnings.informational?.length ?? 0) > 0}
				<div class="mt-4 rounded border border-border p-3" data-testid="warning-informational">
					{#each preview.warnings.informational ?? [] as info (info)}
						<p class="text-sm" data-info-code={info}>
							{i18nMsg(`bank-import-info-${info.replace(/_/g, '-')}`, info)}
						</p>
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

			<div class="mt-4 flex gap-2">
				<button
					type="button"
					data-testid="bank-import-confirm"
					class="rounded bg-primary px-4 py-2 text-white disabled:opacity-50"
					disabled={isLoading || buttonDisabled(preview)}
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
