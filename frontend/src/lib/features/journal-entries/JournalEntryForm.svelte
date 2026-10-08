<script lang="ts">
	import { Button } from '$lib/components/ui/button';
	import { Input } from '$lib/components/ui/input';
	import * as Select from '$lib/components/ui/select';
	import { toast } from 'svelte-sonner';
	import { X, Plus } from '@lucide/svelte';
	import { i18nMsg } from '$lib/features/onboarding/onboarding.svelte';
	import { isApiError } from '$lib/shared/utils/api-client';
	import { notifyMissingFiscalYearOrFallback } from '$lib/shared/utils/notify';
	import type { AccountResponse } from '$lib/features/accounts/accounts.types';
	import type { ProjectResponse } from '$lib/features/projects/projects.types';
	import { createJournalEntry, updateJournalEntry } from './journal-entries.api';
	import type {
		CreateJournalEntryRequest,
		Journal,
		JournalEntryResponse,
		UpdateJournalEntryRequest
	} from './journal-entries.types';
	import { computeBalance, classifyLine, formatSwissAmount, isValidAmount } from './balance';
	import {
		editRefusalOutcome,
		entryDateBounds,
		fromJournalEntryResponse,
		type LineDraft
	} from './form-helpers';
	import { isAccountUnusable } from '$lib/features/accounts/account-validity';
	import { isDraftLineNonEmpty } from './vat-purchase';
	import AccountAutocomplete from './AccountAutocomplete.svelte';
	import VatPurchaseAssistant from './VatPurchaseAssistant.svelte';
	import AccountingTooltip from '$lib/shared/components/AccountingTooltip.svelte';

	interface Props {
		accounts: AccountResponse[];
		accountsLoadError: boolean;
		/** Projets analytiques actifs (Epic 19, Story 19-2). Vide = colonne masquée. */
		projects?: ProjectResponse[];
		/**
		 * Borne du verrou de période (Story 24-4c, #380) — `null` = aucun verrou.
		 * ⚠️ Sert UNIQUEMENT au `min` du champ date : c'est un confort de saisie.
		 * **Le refus qui fait autorité est celui du serveur** (400 `PERIOD_LOCKED`),
		 * et il est testé sans passer par l'écran.
		 */
		booksLockedThrough?: string | null;
		/**
		 * Si fourni → mode **édition** (Story 15-8a, #532), ouvert depuis la fiche
		 * de l'écriture. Sinon mode création.
		 */
		initialEntry?: JournalEntryResponse | null;
		/**
		 * Exercice **de l'écriture** en édition — bornes `min`/`max` du champ date
		 * (confort de saisie ; le serveur tranche). `null` en création.
		 */
		entryFiscalYear?: { startDate: string; endDate: string } | null;
		/** Compte d'impôt préalable pour l'assistant TVA achat (Story 18-1c). Null si non configuré. */
		recoverableAccountId?: number | null;
		onSuccess: () => void;
		onCancel: () => void;
		/**
		 * Édition (Story 15-8a) : appelé, **après** le toast, quand le refus du
		 * serveur dit que l'écriture a changé sous l'utilisateur — exercice clos,
		 * pièce ou contre-passation apparue, version périmée. La fiche se recharge.
		 * ⛔ Remplace la modale de conflit de la 3.3, qui ne revient pas.
		 */
		onStale?: () => void;
	}

	let {
		accounts,
		accountsLoadError,
		projects = [],
		booksLockedThrough = null,
		initialEntry = null,
		entryFiscalYear = null,
		recoverableAccountId = null,
		onSuccess,
		onCancel,
		onStale
	}: Props = $props();

	const isEdit = $derived(initialEntry !== null);

	const JOURNALS: Journal[] = ['Achats', 'Ventes', 'Banque', 'Caisse', 'OD'];

	function todayISO(): string {
		const d = new Date();
		const yyyy = d.getFullYear();
		const mm = String(d.getMonth() + 1).padStart(2, '0');
		const dd = String(d.getDate()).padStart(2, '0');
		return `${yyyy}-${mm}-${dd}`;
	}

	// --- État formulaire ---
	// Pré-remplissage depuis initialEntry si mode édition. Les warnings
	// `state_referenced_locally` sont intentionnellement supprimés : on
	// veut uniquement capturer la valeur initiale au montage du composant.
	// Si initialEntry change au cours de la vie du composant, le parent
	// doit démonter/remonter le formulaire (ce qui est le cas : la fiche
	// remplace sa vue par le formulaire, puis le retire au retour).
	/* svelte-ignore state_referenced_locally */
	let entryDate = $state(initialEntry?.entryDate ?? todayISO());
	/* svelte-ignore state_referenced_locally */
	let journal = $state<Journal>(initialEntry?.journal ?? 'Achats');
	/* svelte-ignore state_referenced_locally */
	let description = $state(initialEntry?.description ?? '');
	/* svelte-ignore state_referenced_locally */
	let lines = $state<LineDraft[]>(
		initialEntry
			? fromJournalEntryResponse(initialEntry)
			: [
					{ accountId: null, debit: '', credit: '', projectId: null },
					{ accountId: null, debit: '', credit: '', projectId: null }
				]
	);
	/* svelte-ignore state_referenced_locally */
	let version = $state(initialEntry?.version ?? 0);
	// Colonne projet affichée si des projets actifs existent — sans projet
	// défini, le formulaire reste identique à avant (zéro friction). En
	// édition, une ligne peut porter un tag historique alors que tous les
	// projets sont archivés (liste active vide) : on affiche quand même la
	// colonne pour que le tag reste visible et détaguable (review Pass 1 BH-L2).
	const showProjectColumn = $derived(
		projects.length > 0 || lines.some((l) => l.projectId !== null)
	);
	const tableColCount = $derived(showProjectColumn ? 5 : 4);

	// Premier jour SAISISSABLE : le lendemain de la borne. La borne étant
	// INCLUSIVE, `min` doit valoir borne + 1 jour — un `min` posé à la borne
	// elle-même laisserait l'écran proposer une date que le serveur refuse. En
	// édition, l'exercice de l'écriture borne aussi (Story 15-8a).
	const dateBounds = $derived(entryDateBounds(entryFiscalYear, booksLockedThrough));

	/**
	 * Story 15-8a — une ligne pré-remplie sur un compte archivé ou non imputable
	 * n'est pas re-sélectionnable : on l'affiche (le libellé se résout sur la
	 * liste complète que la fiche fournit) avec un avertissement. Le refus qui
	 * fait autorité reste le 400 du serveur.
	 */
	function lineAccountUnusable(accountId: number | null): boolean {
		if (accountId === null) return false;
		return isAccountUnusable(accounts.find((a) => a.id === accountId));
	}

	let submitting = $state(false);

	// Assistant TVA achat (Story 18-1c) : lignes en attente de confirmation
	// de remplacement (si le brouillon n'est pas vierge).
	let pendingAssistant = $state<{ lines: LineDraft[]; description: string } | null>(null);

	/** Applique les lignes générées par l'assistant : remplace le brouillon. */
	function applyAssistant(result: { lines: LineDraft[]; description: string }) {
		lines = result.lines;
		journal = 'Achats';
		if (description.trim() === '') {
			description = result.description;
		}
	}

	/** Reçu de l'assistant : confirme si le brouillon n'est pas vierge, sinon applique. */
	function handleAssistantApply(result: { lines: LineDraft[]; description: string }) {
		const dirty = lines.some(isDraftLineNonEmpty) || description.trim() !== '';
		if (dirty) {
			pendingAssistant = result;
		} else {
			applyAssistant(result);
		}
	}

	function confirmAssistantReplace() {
		if (pendingAssistant) applyAssistant(pendingAssistant);
		pendingAssistant = null;
	}

	const balance = $derived(computeBalance(lines));
	const lineStatuses = $derived(lines.map(classifyLine));

	const nonEmptyLines = $derived.by(() => {
		return lines.map((l, i) => ({ line: l, status: lineStatuses[i], index: i }))
			.filter((x) => x.status !== 'empty');
	});

	const canSubmit = $derived.by(() => {
		if (submitting) return false;
		if (description.trim() === '') return false;
		const validLines = nonEmptyLines.filter((x) => x.status === 'valid');
		const partialLines = nonEmptyLines.filter((x) => x.status === 'partial');
		if (partialLines.length > 0) return false;
		if (validLines.length < 2) return false;
		return balance.isBalanced;
	});

	function addLine() {
		lines = [...lines, { accountId: null, debit: '', credit: '', projectId: null }];
	}

	function removeLine(index: number) {
		if (lines.length <= 2) return;
		lines = lines.filter((_, i) => i !== index);
	}

	// P9 : formatage string-based via formatSwissAmount (zéro perte de précision).
	const formatNumber = formatSwissAmount;

	async function handleSubmit() {
		if (!canSubmit) return;
		submitting = true;

		const payload: CreateJournalEntryRequest = {
			entryDate,
			journal,
			description: description.trim(),
			lines: nonEmptyLines.map(({ line }) => ({
				accountId: line.accountId!,
				debit: (line.debit === '' ? '0' : line.debit.replace(',', '.')),
				credit: (line.credit === '' ? '0' : line.credit.replace(',', '.')),
				projectId: line.projectId
			}))
		};

		try {
			if (isEdit && initialEntry) {
				const updatePayload: UpdateJournalEntryRequest = { ...payload, version };
				await updateJournalEntry(initialEntry.id, updatePayload);
			} else {
				await createJournalEntry(payload);
			}
			toast.success(i18nMsg('journal-entry-saved', 'Écriture enregistrée'));
			onSuccess();
		} catch (err) {
			if (isApiError(err)) {
				const code = err.code ?? '';
				// Story 15-8a (D8) — en édition, un refus qui dit que l'écriture a
				// changé sous l'utilisateur : toast, puis la fiche se recharge. ⛔ Pas
				// de modale, et pas `notifyMissingFiscalYearOrFallback` : son conseil
				// (« vérifiez la date saisie ») serait faux, c'est l'exercice DE
				// L'ÉCRITURE qui a été clôturé.
				if (isEdit && editRefusalOutcome(code) === 'stale') {
					let message = err.message;
					if (code === 'FISCAL_YEAR_CLOSED') {
						message = i18nMsg(
							'journal-entries-modify-blocked-fiscal-year-closed',
							'L’exercice de cette écriture est clôturé : elle est figée. Corrigez-la par une contre-passation.'
						);
					} else if (code === 'OPTIMISTIC_LOCK_CONFLICT') {
						message = i18nMsg(
							'journal-entries-edit-conflict',
							'Cette écriture a été modifiée entre-temps : la fiche a été rechargée.'
						);
					}
					toast.error(message);
					onStale?.();
					return;
				}
				// Story 3.7 AC #22 — fallback toast actionnable pour NO_FISCAL_YEAR / FISCAL_YEAR_CLOSED.
				if (notifyMissingFiscalYearOrFallback(err)) {
					return;
				}
				switch (code) {
					case 'ENTRY_UNBALANCED':
					case 'DATE_OUTSIDE_FISCAL_YEAR':
					case 'INACTIVE_OR_INVALID_ACCOUNTS':
					// Story 15-5a — le message du serveur nomme le ou les comptes.
					case 'ACCOUNT_NOT_POSTABLE':
					// Story 15-8a — le message nomme la borne et la date refusée.
					case 'PERIOD_LOCKED':
					case 'VALIDATION_ERROR':
						toast.error(err.message);
						break;
					case 'RESOURCE_CONFLICT':
						// Race sur uq_journal_entries_number (création).
						toast.error(
							i18nMsg(
								'error-conflict',
								'Conflit de numérotation — veuillez réessayer'
							)
						);
						break;
					default:
						toast.error(err.message || 'Erreur lors de la sauvegarde');
				}
			} else {
				toast.error('Erreur lors de la sauvegarde');
			}
		} finally {
			submitting = false;
		}
	}

	function handleKeydown(e: KeyboardEvent) {
		// Ctrl+S → submit
		if ((e.ctrlKey || e.metaKey) && e.key === 's') {
			e.preventDefault();
			if (canSubmit) handleSubmit();
		}
	}

	function handleLineCreditKeydown(e: KeyboardEvent, index: number) {
		// Enter dans le dernier crédit → ajouter une ligne.
		if (e.key === 'Enter' && index === lines.length - 1) {
			e.preventDefault();
			addLine();
		}
	}
</script>

<svelte:window onkeydown={handleKeydown} />

<div class="space-y-6 p-6 bg-card rounded-lg border border-border">
	<h2 class="text-xl font-semibold">
		{i18nMsg('journal-entry-form-title', 'Saisie d\'écriture')}
	</h2>

	<div class="grid grid-cols-1 md:grid-cols-3 gap-4">
		<div>
			<label for="entry-date" class="block text-sm font-medium mb-1">
				{i18nMsg('journal-entry-form-date', 'Date')}
			</label>
			<Input
				id="entry-date"
				type="date"
				bind:value={entryDate}
				min={dateBounds.min}
				max={dateBounds.max}
				required
			/>
		</div>
		<div>
			<label for="entry-journal" class="block text-sm font-medium mb-1">
				<AccountingTooltip term="journal">
					<!-- svelte-ignore a11y_no_noninteractive_tabindex -->
					<span tabindex="0" class="cursor-help underline underline-offset-2 decoration-dotted">
						{i18nMsg('journal-entry-form-journal', 'Journal')}
					</span>
				</AccountingTooltip>
			</label>
			<Select.Root type="single" value={journal} onValueChange={(v) => (journal = v as Journal)}>
				<Select.Trigger id="entry-journal">
					{journal}
				</Select.Trigger>
				<Select.Content>
					{#each JOURNALS as j (j)}
						<Select.Item value={j}>{i18nMsg(`journal-${j.toLowerCase()}`, j)}</Select.Item>
					{/each}
				</Select.Content>
			</Select.Root>
		</div>
		<div>
			<label for="entry-description" class="block text-sm font-medium mb-1">
				{i18nMsg('journal-entry-form-description', 'Libellé')}
			</label>
			<Input id="entry-description" type="text" bind:value={description} required />
		</div>
	</div>

	{#if !isEdit}
		<VatPurchaseAssistant
			{accounts}
			{accountsLoadError}
			{recoverableAccountId}
			onApply={handleAssistantApply}
		/>
	{/if}

	<table class="w-full border-collapse">
		<thead>
			<tr class="border-b border-border">
				<th class="text-left py-2 text-sm font-medium">
					{i18nMsg('journal-entry-form-col-account', 'Compte')}
				</th>
				<th class="text-right py-2 text-sm font-medium w-32">
					<AccountingTooltip term="debit">
						<!-- svelte-ignore a11y_no_noninteractive_tabindex -->
						<span tabindex="0" class="cursor-help underline underline-offset-2 decoration-dotted">
							{i18nMsg('journal-entry-form-col-debit', 'Débit')}
						</span>
					</AccountingTooltip>
				</th>
				<th class="text-right py-2 text-sm font-medium w-32">
					<AccountingTooltip term="credit">
						<!-- svelte-ignore a11y_no_noninteractive_tabindex -->
						<span tabindex="0" class="cursor-help underline underline-offset-2 decoration-dotted">
							{i18nMsg('journal-entry-form-col-credit', 'Crédit')}
						</span>
					</AccountingTooltip>
				</th>
				{#if showProjectColumn}
					<th class="text-left py-2 text-sm font-medium w-44">
						{i18nMsg('journal-entry-form-col-project', 'Projet')}
					</th>
				{/if}
				<th class="w-10"></th>
			</tr>
		</thead>
		<tbody>
			{#each lines as line, i (i)}
				{@const status = lineStatuses[i]}
				<tr class="border-b border-border/50">
					<td class="py-2 pr-2">
						<AccountAutocomplete
							{accounts}
							value={line.accountId}
							loadError={accountsLoadError}
							onSelect={(id) => (lines[i].accountId = id)}
						/>
						{#if lineAccountUnusable(line.accountId)}
							<p class="mt-1 text-xs text-destructive" data-testid="line-account-unusable">
								{i18nMsg(
									'journal-entries-line-account-unusable',
									'Compte archivé ou non imputable — à remplacer'
								)}
							</p>
						{/if}
					</td>
					<td class="py-2 pr-2">
						<Input
							type="text"
							inputmode="decimal"
							bind:value={lines[i].debit}
							class={!isValidAmount(line.debit) ? 'border-destructive' : 'tabular-nums text-right'}
							placeholder="0.00"
						/>
					</td>
					<td class="py-2 pr-2">
						<Input
							type="text"
							inputmode="decimal"
							bind:value={lines[i].credit}
							onkeydown={(e) => handleLineCreditKeydown(e, i)}
							class={!isValidAmount(line.credit) ? 'border-destructive' : 'tabular-nums text-right'}
							placeholder="0.00"
						/>
					</td>
					{#if showProjectColumn}
						<td class="py-2 pr-2">
							<!-- Sélecteur arbre 2 niveaux (racines + sous-projets indentés),
							     même pattern que la facture fournisseur (Story 19-3). -->
							<select
								bind:value={lines[i].projectId}
								data-testid="journal-entry-line-project-{i}"
								aria-label={i18nMsg('journal-entry-form-col-project', 'Projet')}
								class="w-full h-9 rounded-md border border-input bg-background px-2 text-sm"
							>
								<option value={null}>{i18nMsg('journal-entry-project-none', '— Aucun')}</option>
								{#if line.projectId !== null && !projects.some((p) => p.id === line.projectId)}
									<!-- Tag historique sur un projet archivé (absent de la liste des
									     actifs) : option ad-hoc pour préserver la valeur au round-trip
									     (le backend exempte les tags pré-existants de la validation). -->
									<option value={line.projectId}>
										{i18nMsg('journal-entry-project-archived', 'Projet archivé')}
									</option>
								{/if}
								{#each projects.filter((p) => p.parentId === null) as root (root.id)}
									<option value={root.id}>{root.code} — {root.name}</option>
									{#each projects.filter((c) => c.parentId === root.id) as child (child.id)}
										<option value={child.id}>&nbsp;&nbsp;↳ {child.code} — {child.name}</option>
									{/each}
								{/each}
							</select>
						</td>
					{/if}
					<td class="py-2">
						{#if lines.length > 2}
							<button
								type="button"
								onclick={() => removeLine(i)}
								class="text-muted-foreground hover:text-destructive p-1"
								aria-label={i18nMsg('journal-entry-form-remove-line', 'Retirer cette ligne')}
							>
								<X class="w-4 h-4" />
							</button>
						{/if}
					</td>
				</tr>
				{#if status === 'partial'}
					<tr>
						<td colspan={tableColCount} class="text-xs text-destructive pb-2">
							{i18nMsg('journal-entry-form-incomplete-line', 'Ligne incomplète')}
						</td>
					</tr>
				{/if}
				{#if !isValidAmount(line.debit) || !isValidAmount(line.credit)}
					<tr>
						<td colspan={tableColCount} class="text-xs text-destructive pb-2">
							{i18nMsg('journal-entry-form-max-decimals', 'Maximum 4 décimales')}
						</td>
					</tr>
				{/if}
			{/each}
		</tbody>
	</table>

	<Button type="button" variant="outline" size="sm" onclick={addLine}>
		<Plus class="w-4 h-4 mr-1" />
		{i18nMsg('journal-entry-form-add-line', '+ Ajouter une ligne')}
	</Button>

	<div
		class="flex items-center justify-between rounded-md border p-4 tabular-nums {balance.isBalanced
			? 'border-green-600 bg-green-50 dark:bg-green-950/30'
			: balance.totalDebit.gt(0) || balance.totalCredit.gt(0)
				? 'border-destructive bg-red-50 dark:bg-red-950/30'
				: 'border-border'}"
	>
		<div class="space-x-4 text-sm">
			<span>
				<strong>{i18nMsg('journal-entry-form-total-debit', 'Total débits')} :</strong>
				{formatNumber(balance.totalDebit)}
			</span>
			<span>
				<strong>{i18nMsg('journal-entry-form-total-credit', 'Total crédits')} :</strong>
				{formatNumber(balance.totalCredit)}
			</span>
			<span>
				<strong>{i18nMsg('journal-entry-form-diff', 'Différence')} :</strong>
				{formatNumber(balance.diff)}
			</span>
		</div>
		<div class="text-sm font-medium">
			{#if balance.isBalanced || balance.totalDebit.gt(0) || balance.totalCredit.gt(0)}
				<AccountingTooltip term="balanced">
					{#if balance.isBalanced}
						<!-- svelte-ignore a11y_no_noninteractive_tabindex -->
						<span tabindex="0" class="text-green-700 dark:text-green-400 cursor-help">
							✓ {i18nMsg('journal-entry-form-balanced', 'Équilibré')}
						</span>
					{:else}
						<!-- svelte-ignore a11y_no_noninteractive_tabindex -->
						<span tabindex="0" class="text-destructive cursor-help">
							✗ {i18nMsg('journal-entry-form-unbalanced', 'Déséquilibré')}
						</span>
					{/if}
				</AccountingTooltip>
			{:else}
				<!-- Formulaire vide (débit=0, crédit=0) : on n'instancie PAS
				     AccountingTooltip pour éviter un bouton bits-ui focusable
				     sans accessible name (WCAG 4.1.2). Placeholder invisible. -->
				<span aria-hidden="true" class="opacity-0">—</span>
			{/if}
		</div>
	</div>

	<div class="flex justify-end gap-2">
		<Button type="button" variant="outline" onclick={onCancel} disabled={submitting}>
			{i18nMsg('journal-entry-form-cancel', 'Annuler')}
		</Button>
		<Button type="button" onclick={handleSubmit} disabled={!canSubmit}>
			{i18nMsg('journal-entry-form-submit', 'Valider')}
		</Button>
	</div>
</div>

<!-- Modale de confirmation de remplacement par l'assistant TVA achat (Story 18-1c) -->
{#if pendingAssistant}
	<div
		class="fixed inset-0 z-50 flex items-center justify-center bg-black/50"
		role="dialog"
		aria-modal="true"
		aria-labelledby="vat-replace-title"
		aria-describedby="vat-replace-desc"
	>
		<div class="bg-card border border-border rounded-lg p-6 max-w-md mx-4 shadow-lg">
			<h2 id="vat-replace-title" class="text-lg font-semibold mb-2">
				{i18nMsg('vat-purchase-replace-title', 'Remplacer le brouillon ?')}
			</h2>
			<p id="vat-replace-desc" class="text-sm text-text-muted mb-4">
				{i18nMsg(
					'vat-purchase-replace-message',
					'Des lignes ou un libellé ont déjà été saisis. Continuer écrasera le brouillon actuel.'
				)}
			</p>
			<div class="flex justify-end gap-2">
				<Button type="button" variant="outline" onclick={() => (pendingAssistant = null)}>
					{i18nMsg('journal-entry-form-cancel', 'Annuler')}
				</Button>
				<Button type="button" onclick={confirmAssistantReplace}>
					{i18nMsg('vat-purchase-replace-confirm', 'Remplacer')}
				</Button>
			</div>
		</div>
	</div>
{/if}
