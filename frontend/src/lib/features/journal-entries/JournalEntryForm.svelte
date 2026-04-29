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
	import { createJournalEntry, updateJournalEntry } from './journal-entries.api';
	import type {
		CreateJournalEntryRequest,
		Journal,
		JournalEntryResponse,
		UpdateJournalEntryRequest
	} from './journal-entries.types';
	import { computeBalance, classifyLine, formatSwissAmount, isValidAmount } from './balance';
	import { fromJournalEntryResponse, type LineDraft } from './form-helpers';
	import AccountAutocomplete from './AccountAutocomplete.svelte';
	import AccountingTooltip from '$lib/shared/components/AccountingTooltip.svelte';

	interface Props {
		accounts: AccountResponse[];
		accountsLoadError: boolean;
		/** Si fourni → mode édition. Sinon mode création. */
		initialEntry?: JournalEntryResponse | null;
		onSuccess: () => void;
		onCancel: () => void;
		/** Appelé quand un conflit de version (409) doit déclencher un reload. */
		onConflictReload?: () => void;
	}

	let {
		accounts,
		accountsLoadError,
		initialEntry = null,
		onSuccess,
		onCancel,
		onConflictReload
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
	// doit démonter/remonter le formulaire (ce qui est le cas : le parent
	// passe de mode 'list' à 'edit' en changeant mode ET editingEntry).
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
					{ accountId: null, debit: '', credit: '' },
					{ accountId: null, debit: '', credit: '' }
				]
	);
	/* svelte-ignore state_referenced_locally */
	let version = $state(initialEntry?.version ?? 0);
	let submitting = $state(false);

	// Modale de conflit 409.
	let showConflictDialog = $state(false);

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
		lines = [...lines, { accountId: null, debit: '', credit: '' }];
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
				credit: (line.credit === '' ? '0' : line.credit.replace(',', '.'))
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
				// Story 3.7 AC #22 — fallback toast actionnable pour NO_FISCAL_YEAR / FISCAL_YEAR_CLOSED.
				if (notifyMissingFiscalYearOrFallback(err)) {
					return;
				}
				const code = err.code ?? '';
				switch (code) {
					case 'ENTRY_UNBALANCED':
					case 'DATE_OUTSIDE_FISCAL_YEAR':
					case 'INACTIVE_OR_INVALID_ACCOUNTS':
					case 'VALIDATION_ERROR':
						toast.error(err.message);
						break;
					case 'OPTIMISTIC_LOCK_CONFLICT':
						// Conflit de version (story 3.3) — ouvrir la modale de reload.
						showConflictDialog = true;
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

	function handleConflictReload() {
		showConflictDialog = false;
		if (onConflictReload) {
			onConflictReload();
		} else {
			onCancel();
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
			<Input id="entry-date" type="date" bind:value={entryDate} required />
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
						<td colspan="4" class="text-xs text-destructive pb-2">
							{i18nMsg('journal-entry-form-incomplete-line', 'Ligne incomplète')}
						</td>
					</tr>
				{/if}
				{#if !isValidAmount(line.debit) || !isValidAmount(line.credit)}
					<tr>
						<td colspan="4" class="text-xs text-destructive pb-2">
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

<!-- Modale de conflit de version 409 (story 3.3) -->
{#if showConflictDialog}
	<div
		class="fixed inset-0 z-50 flex items-center justify-center bg-black/50"
		role="dialog"
		aria-modal="true"
		aria-labelledby="conflict-title"
		aria-describedby="conflict-desc"
	>
		<div class="bg-card border border-border rounded-lg p-6 max-w-md mx-4 shadow-lg">
			<h2 id="conflict-title" class="text-lg font-semibold mb-2">
				{i18nMsg('journal-entry-conflict-title', 'Conflit de version')}
			</h2>
			<p id="conflict-desc" class="text-sm text-text-muted mb-4">
				{i18nMsg(
					'journal-entry-conflict-message',
					'Cette écriture a été modifiée par un autre utilisateur. Voulez-vous recharger ?'
				)}
			</p>
			<div class="flex justify-end gap-2">
				<!-- P5 : "Annuler" déclenche aussi un reload — sinon la version est
				     périmée et tout resubmit retourne 409 en boucle. Le seul chemin
				     pour quitter le mode édition "proprement" est via le bouton
				     Annuler du formulaire principal. Ici "Annuler le conflit" = "Recharger". -->
				<!-- svelte-ignore a11y_autofocus -->
				<Button type="button" variant="outline" onclick={handleConflictReload} autofocus>
					{i18nMsg('journal-entry-form-cancel', 'Annuler')}
				</Button>
				<Button type="button" onclick={handleConflictReload}>
					{i18nMsg('journal-entry-conflict-reload', 'Recharger')}
				</Button>
			</div>
		</div>
	</div>
{/if}
