<!--
  Story 8-1b — Upload + preview + confirm dans un seul composant.

  Machine d'état (Story 27-3, #626) :
  - repos : aucun fichier retenu (`selectedFile` nul), zone de dépôt affichée ;
  - aperçu en cours (`isPreviewing`) : demandé par un dépôt (`previewReason`
    = 'drop'), un changement de profil ('profile') ou un refus 409 de la
    confirmation ('stale'). Pendant ce temps ni l'ancien aperçu, ni la zone de
    dépôt ne sont affichés ; les cases de confirmation sont remises à zéro. Chaque
    demande porte un numéro de séquence (`seq`, variable ordinaire) : seule la
    dernière pose son résultat, son erreur ou éteint l'indicateur ;
  - aperçu affiché (`preview`) : un INSTANTANÉ (fichier, compte, profil résolu,
    empreinte) est pris à sa réception ; la confirmation part de lui, jamais de
    l'état courant des sélecteurs ;
  - confirmation en cours (`isConfirming`) : sélecteurs et Annuler désactivés.
    Sur 409 BANK_IMPORT_PREVIEW_STALE, l'aperçu est recalculé depuis l'instantané
    et un avis neutre l'annonce ; l'utilisateur reconfirme ;
  - `reset()` (changement de compte, Annuler, succès) invalide la séquence et
    revient au repos.

  Story 8-3 — panneaux d'avertissement (duplicateFile, duplicateLines,
  invalidLines, encodingMismatch) et cases confirm associées ; KF #70 :
  BankProfileSelector, présélection du profil auto-détecté.
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
	import {
		bankAccountOptionLabel,
		type BankAccountOptionSource,
	} from '$lib/features/bank-accounts/format';
	import { csvLineErrorLabel } from './csv-line-error-label';
	import type { InvalidLinesPayload } from './bank-import.types';

	type Props = {
		bankAccounts: Array<BankAccountOptionSource & { id: number }>;
		onSuccess?: (imported: BankImportResponse) => void;
	};
	let { bankAccounts, onSuccess }: Props = $props();

	/** Raison d'un aperçu : elle choisit le texte de l'indicateur (AC2). */
	type PreviewReason = 'drop' | 'profile' | 'stale';

	/**
	 * Ce que la confirmation envoie : l'aperçu AFFICHÉ, pas l'état des
	 * sélecteurs (Story 27-3, AC5). `bankProfileId` est le profil RÉSOLU par
	 * l'aperçu (absent pour un CAMT, dont l'aperçu ignore le profil).
	 */
	type PreviewSnapshot = {
		file: File;
		accountId: number;
		bankProfileId: number | undefined;
		fingerprint: string;
	};

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
	let isPreviewing = $state(false);
	let isConfirming = $state(false);
	let previewReason = $state<PreviewReason>('drop');
	// Avis « le profil, le fichier ou le compte a changé » (AC9). Effacé en
	// trois sites : `reset()`, `runPreview` hors 'stale', `handleConfirm`.
	let showRecomputedNotice = $state(false);
	let errorCode = $state<string | null>(null);
	let errorMessage = $state<string | null>(null);
	// Story 26-1 (#620) — lignes du refus `BANK_CSV_AMOUNT_SIGN_MISMATCH`
	// (même forme que `warnings.invalidLines`). Posé et effacé avec `errorCode`.
	let errorDetails = $state<InvalidLinesPayload | null>(null);
	let dragActive = $state(false);
	// Instantané de l'aperçu affiché — lu par la seule confirmation, jamais
	// rendu : variable ordinaire.
	let snapshot: PreviewSnapshot | null = null;
	// Numéro de la dernière demande d'aperçu. Variable ORDINAIRE, jamais un
	// `$state` : `reset()` l'incrémente depuis le `$effect` du compte, qui
	// dépendrait sinon de lui (AC3).
	let seq = 0;
	// Numéro de la dernière relecture des profils (variable ordinaire, cf. `seq`).
	let profilesSeq = 0;

	/** Lignes du refus de signe, si `details` en porte (sinon `null`). */
	function signRefusalDetails(err: { code: string; details?: Record<string, unknown> }): InvalidLinesPayload | null {
		if (err.code !== 'BANK_CSV_AMOUNT_SIGN_MISMATCH' || !err.details) return null;
		const d = err.details;
		if (!Array.isArray(d.lines)) return null;
		return {
			lines: d.lines as InvalidLinesPayload['lines'],
			totalErrors: typeof d.totalErrors === 'number' ? d.totalErrors : d.lines.length,
			truncated: d.truncated === true,
		};
	}

	/**
	 * Charge la liste des profils du sélecteur. Appelée au montage et après un
	 * `409 BANK_IMPORT_PREVIEW_STALE` (revue de code P1, B-1, C-27-3-51) : le
	 * profil a pu être renommé, supprimé ou créé ailleurs. En cas d'échec, la
	 * liste en place est gardée (vide au montage) : un rafraîchissement raté ne
	 * doit pas faire disparaître le sélecteur.
	 *
	 * Si le profil choisi n'est plus dans la liste relue (supprimé ailleurs), le
	 * choix revient à l'auto-détection (revue de code P2, M1, C-27-3-57) : sans
	 * quoi le sélecteur afficherait « Auto-détection » pendant qu'un nouveau
	 * dépôt partirait avec l'id supprimé (404), et une liste vide masquerait le
	 * sélecteur sans issue. Le recalcul déjà en vol garde son propre profil, et
	 * son erreur s'affiche. Seule la dernière relecture pose son résultat (L4).
	 */
	async function loadProfiles(): Promise<void> {
		const mySeq = ++profilesSeq;
		try {
			const list = await listBankProfiles(1, 50);
			if (mySeq !== profilesSeq) return;
			availableProfiles = list.items;
			if (bankProfileId !== null && !list.items.some((p) => p.id === bankProfileId)) {
				bankProfileId = null;
			}
		} catch {
			// liste en place gardée
		}
	}

	onMount(loadProfiles);

	let lastSeenAccountId: number | null = null;
	$effect(() => {
		if (lastSeenAccountId !== null && lastSeenAccountId !== selectedAccountId) {
			reset();
		}
		lastSeenAccountId = selectedAccountId;
	});

	/**
	 * Remet les cases et le choix des doublons à leur état initial. Les
	 * avertissements d'un nouvel aperçu ne sont pas ceux de l'ancien (F1 Pass 3,
	 * étendu au recalcul par la Story 27-3) : une case cochée sur l'ancien
	 * n'autorise rien sur le nouveau.
	 */
	function resetConfirmations(): void {
		confirmBalanceMismatch = false;
		confirmDuplicateFile = false;
		confirmDuplicateLines = 'skip';
		confirmPartialImport = false;
		confirmEncodingMismatch = false;
	}

	function clearError(): void {
		errorCode = null;
		errorMessage = null;
		errorDetails = null;
	}

	/** Pose l'erreur d'une requête (aperçu ou confirmation). */
	function setError(err: unknown): void {
		if (isApiError(err)) {
			errorCode = err.code;
			errorMessage = err.message;
			// Le refus de signe peut arriver à l'aperçu comme à la confirmation.
			errorDetails = signRefusalDetails(err);
		} else {
			errorCode = 'UNKNOWN_ERROR';
			errorMessage = i18nMsg('error-unexpected', 'Erreur inattendue.');
			errorDetails = null;
		}
	}

	function reset(): void {
		// Invalide l'aperçu en vol : sa réponse ne pourra plus éteindre
		// l'indicateur, c'est donc ici qu'il s'éteint (AC3).
		seq += 1;
		selectedFile = null;
		preview = null;
		snapshot = null;
		resetConfirmations();
		clearError();
		isPreviewing = false;
		// Défensif : pendant une confirmation, compte et Annuler sont désactivés,
		// et le succès éteint déjà `isConfirming` dans son `finally` (C-27-3-38).
		isConfirming = false;
		showRecomputedNotice = false;
	}

	/**
	 * Calcule l'aperçu de `file` sur `accountId` avec `profileId` (null =
	 * auto-détection). Seule la dernière demande pose son résultat (AC3).
	 */
	async function runPreview(
		file: File,
		accountId: number,
		profileId: number | null,
		reason: PreviewReason,
	): Promise<void> {
		const mySeq = ++seq;
		previewReason = reason;
		if (reason !== 'stale') showRecomputedNotice = false;
		resetConfirmations();
		selectedFile = file;
		preview = null;
		snapshot = null;
		clearError();
		isPreviewing = true;
		try {
			const result = await previewBankImport(file, accountId, {
				bankProfileId: profileId ?? undefined,
			});
			if (mySeq !== seq) return;
			preview = result;
			snapshot = {
				file,
				accountId,
				bankProfileId: result.csvProfileMatch?.profileId ?? undefined,
				fingerprint: result.previewFingerprint,
			};
			// Présélectionne le profil auto-détecté (KF #70) — affectation
			// programmatique : elle ne relance pas l'aperçu (AC4).
			if (
				profileId === null &&
				result.csvProfileMatch?.autoMatched &&
				result.csvProfileMatch?.profileId
			) {
				bankProfileId = result.csvProfileMatch.profileId;
			}
		} catch (err) {
			if (mySeq !== seq) return;
			setError(err);
			preview = null;
		} finally {
			if (mySeq === seq) isPreviewing = false;
		}
	}

	async function handleFileSelect(file: File): Promise<void> {
		if (!selectedAccountId) {
			errorCode = 'BANK_ACCOUNT_REQUIRED';
			errorMessage = i18nMsg(
				'bank-import-errors-bank-account-required',
				"Sélectionnez d'abord un compte bancaire.",
			);
			errorDetails = null;
			return;
		}
		await runPreview(file, selectedAccountId, bankProfileId, 'drop');
	}

	/**
	 * Geste de l'utilisateur sur le sélecteur de profil (AC1, AC1-bis) : avec un
	 * fichier retenu, l'aperçu est recalculé — aussi après un aperçu en échec.
	 * Jamais un `$effect` sur `bankProfileId` : la présélection (AC4) n'est pas
	 * un geste.
	 */
	function handleProfileChange(id: number | null): void {
		bankProfileId = id;
		if (selectedFile && selectedAccountId) {
			void runPreview(selectedFile, selectedAccountId, id, 'profile');
		}
	}

	async function handleConfirm(): Promise<void> {
		// L3 (Pass 1 review 8-3) — la garde anti-IDOR qui comparait l'IBAN de
		// l'aperçu au compte choisi n'a pas lieu d'être : la confirmation part de
		// l'instantané de l'aperçu affiché (Story 27-3), et le serveur compare
		// l'empreinte de cet aperçu à ses propres entrées. Ne pas la restaurer
		// par réflexe.
		if (!snapshot || !preview) return;
		const snap = snapshot;
		showRecomputedNotice = false;
		isConfirming = true;
		clearError();
		try {
			const result = await createBankImport(snap.file, snap.accountId, {
				confirmBalanceMismatch,
				bankProfileId: snap.bankProfileId,
				confirmEncodingMismatch,
				confirmDuplicateFile,
				confirmDuplicateLines,
				confirmPartialImport,
				previewFingerprint: snap.fingerprint,
			});
			onSuccess?.(result);
			reset();
		} catch (err) {
			if (isApiError(err) && err.code === 'BANK_IMPORT_PREVIEW_STALE') {
				// Le profil, le fichier ou le compte a changé : recalcul depuis
				// l'instantané, sans nouvelle confirmation automatique (AC9).
				// `isConfirming` s'éteint AVANT la relance, non attendue : pendant
				// le recalcul, les sélecteurs sont actifs (C-27-3-33).
				isConfirming = false;
				showRecomputedNotice = true;
				void runPreview(snap.file, snap.accountId, snap.bankProfileId ?? null, 'stale');
				// Le sélecteur relit les profils : un profil renommé ou supprimé
				// ailleurs n'y garde pas son ancien nom (B-1, C-27-3-51).
				void loadProfiles();
			} else {
				setError(err);
			}
		} finally {
			isConfirming = false;
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
		// Story 27-3 (AC16) : vider la valeur, sans quoi re-choisir le même
		// fichier n'émet pas `change` (le `File` est déjà capturé).
		input.value = '';
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
		disabled={isConfirming}
		class="mt-1 block w-full rounded border-border bg-surface p-2 text-text"
	>
		<option value={null}>— {i18nMsg('bank-import-labels-bank-account-placeholder', 'Sélectionner un compte')} —</option>
		{#each bankAccounts as account (account.id)}
			<!-- Story 26-2b (C-26-2-26) : « UBS (CH…) » sans description, comme avant. -->
			<option value={account.id}>{bankAccountOptionLabel(account, 'parens')}</option>
		{/each}
	</select>

	{#if availableProfiles.length > 0}
		<div class="mt-4">
			<BankProfileSelector
				profiles={availableProfiles}
				autoMatchedId={preview?.csvProfileMatch?.autoMatched
					? preview.csvProfileMatch.profileId
					: null}
				value={bankProfileId}
				disabled={isConfirming}
				onChange={handleProfileChange}
			/>
		</div>
	{/if}

	{#if !preview && !isPreviewing}
		<div
			data-testid="bank-import-drop-zone"
			role="button"
			tabindex="0"
			aria-label={i18nMsg(
				'bank-import-labels-drop-zone',
				'Glissez votre fichier CAMT.053 ou CSV ici ou cliquez pour parcourir',
			)}
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

	{#if isPreviewing || isConfirming}
		<!-- Story 27-3 (AC2) : la confirmation d'abord, puis la raison de l'aperçu
		     (`previewReason` garde sa valeur après l'aperçu, C-27-3-45). -->
		<p class="mt-4 text-text-muted" data-testid="bank-import-loading" role="status">
			{isConfirming || previewReason === 'drop'
				? i18nMsg('common-loading', 'Chargement…')
				: i18nMsg('bank-import-labels-preview-recomputing', 'Recalcul de la prévisualisation…')}
		</p>
	{/if}

	{#if showRecomputedNotice}
		<!-- Story 27-3 (AC9) : hors du bloc de l'aperçu, pour rester visible si le
		     recalcul échoue. Neutre : l'écran ne sait pas ce qui a changé. -->
		<p
			class="mt-4 rounded border border-warning bg-warning-soft p-3"
			data-testid="bank-import-preview-recomputed"
			role="status"
		>
			{i18nMsg(
				'bank-import-labels-preview-recomputed',
				'Le profil, le fichier ou le compte a changé depuis la prévisualisation : Kesh la recalcule. Vérifiez le résultat avant de confirmer.',
			)}
		</p>
	{/if}

	{#if errorMessage}
		<div
			class="mt-4 rounded border border-error bg-error-soft p-3"
			data-testid="bank-import-error"
			data-error-code={errorCode}
			role="alert"
		>
			<p class="text-error">{errorMessage}</p>
			{#if errorDetails}
				<!-- Story 26-1 (#620) : les lignes du refus de signe. Aucune case
				     de confirmation : ce refus n'est pas confirmable. -->
				{#if errorDetails.truncated}
					<p class="mt-2 text-sm text-text-muted">
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
								<th class="text-left">{i18nMsg('bank-import-labels-col-line', 'Ligne')}</th>
								<th class="text-left">{i18nMsg('bank-import-labels-col-code', 'Code')}</th>
								<th class="text-left">{i18nMsg('bank-import-labels-col-value', 'Valeur')}</th>
							</tr>
						</thead>
						<tbody>
							{#each errorDetails.lines as line, i (i)}
								<tr data-testid="bank-import-error-line">
									<td>{line.line}</td>
									<td>
										<code>{line.code}</code>
										<span data-testid="bank-import-error-line-message">{csvLineErrorLabel(line.code)}</span>
									</td>
									<td>{line.value ?? '—'}</td>
								</tr>
							{/each}
						</tbody>
					</table>
				</div>
			{/if}
		</div>
	{/if}

	{#if preview}
		<div class="mt-6" data-testid="bank-import-preview">
			<h2 class="text-lg font-semibold">
				{i18nMsg('bank-import-labels-preview-title', 'Prévisualisation')}
			</h2>
			<dl class="mt-2 grid grid-cols-2 gap-2 text-sm">
				<dt>{i18nMsg('bank-import-labels-iban', 'IBAN')}</dt>
				<dd data-testid="preview-iban">{preview.selectedStatement.accountIban}</dd>
				<dt>{i18nMsg('bank-import-labels-period', 'Période')}</dt>
				<dd>{preview.selectedStatement.periodFrom} → {preview.selectedStatement.periodTo}</dd>
				<dt>{i18nMsg('bank-import-labels-currency', 'Devise')}</dt>
				<dd>{preview.selectedStatement.currency}</dd>
				<dt>{i18nMsg('bank-import-labels-opening-balance', 'Solde ouverture')}</dt>
				<dd>{preview.selectedStatement.openingBalance ?? '—'}</dd>
				<dt>{i18nMsg('bank-import-labels-closing-balance', 'Solde clôture')}</dt>
				<dd>{preview.selectedStatement.closingBalance ?? '—'}</dd>
				<dt>{i18nMsg('bank-import-labels-transaction-count', 'Nombre de transactions')}</dt>
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
								<th class="text-left">{i18nMsg('bank-import-labels-duplicate-col-index', 'Index')}</th>
								<th class="text-left">{i18nMsg('bank-import-labels-duplicate-col-existing-id', 'ID existant')}</th>
								<th class="text-left">{i18nMsg('bank-import-labels-duplicate-col-key', 'Clé')}</th>
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
									<th class="text-left">{i18nMsg('bank-import-labels-col-line', 'Ligne')}</th>
									<th class="text-left">{i18nMsg('bank-import-labels-col-code', 'Code')}</th>
									<th class="text-left">{i18nMsg('bank-import-labels-col-value', 'Valeur')}</th>
								</tr>
							</thead>
							<tbody>
								{#each preview.warnings.invalidLines.lines as line, i (i)}
									<tr data-testid="warning-invalid-lines-row">
										<td>{line.line}</td>
										<td>
											<code>{line.code}</code>
											<span data-testid="warning-invalid-lines-message">{csvLineErrorLabel(line.code)}</span>
										</td>
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
							'Devise non supportée : seul le franc suisse (CHF) est pris en charge. Devise du relevé :',
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
						{preview.ignoredStatements.length}
						{i18nMsg(
							'bank-import-labels-ignored-statements-summary',
							'statement(s) ignoré(s) (autres IBAN dans le fichier).',
						)}
					</p>
				</div>
			{/if}

			<table class="mt-4 w-full table-auto text-sm" data-testid="preview-tx-table">
				<thead>
					<tr>
						<th class="text-left">{i18nMsg('bank-import-labels-tx-col-date', 'Date')}</th>
						<th class="text-right">{i18nMsg('bank-import-labels-tx-col-amount', 'Montant')}</th>
						<th class="text-left">{i18nMsg('bank-import-labels-tx-col-details', 'Détails')}</th>
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
					disabled={isConfirming || buttonDisabled(preview)}
					onclick={handleConfirm}
				>
					{i18nMsg('bank-import-labels-confirm-import', "Confirmer l'import")}
				</button>
				<!-- Story 27-3 (D15) : désactivé pendant la confirmation — Annuler
				     suivi d'un 409 aurait fait réapparaître l'aperçu. -->
				<button
					type="button"
					data-testid="bank-import-cancel"
					class="rounded border border-border px-4 py-2 disabled:opacity-50"
					disabled={isConfirming}
					onclick={reset}
				>
					{i18nMsg('bank-import-labels-cancel', 'Annuler')}
				</button>
			</div>
		</div>
	{/if}
</section>
