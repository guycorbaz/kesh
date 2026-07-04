<!--
  Story 8-5a-bis FR48 — Modal éclatement de transaction agrégée.

  Permet à un Comptable d'éclater une `bank_transaction` `pending`
  agrégée en N+1 lignes de `journal_entry` (1 ligne banque agrégée +
  N lignes contrepartie). Le compte ledger banque est résolu
  **serveur-side** via `bank_account.journal_account_id` (foundation
  8-5a-zero) — donc PAS de field `bankLedgerAccountId` dans le body API.

  Tableau éditable (min 2 max 50 lignes) avec indicator balance live
  vert/rouge. Submit désactivé tant que la balance n'est pas exact
  Decimal (cf. AC #93/#95).

  Gestion erreur 412 BANK_ACCOUNT_NOT_CONFIGURED : message + lien vers
  `/bank-accounts` pour configurer le journal_account_id manquant.
-->
<script lang="ts">
	import { Button } from '$lib/components/ui/button';
	import { Input } from '$lib/components/ui/input';
	import * as Dialog from '$lib/components/ui/dialog';
	import { i18nMsg } from '$lib/shared/utils/i18n.svelte';
	import AccountAutocomplete from '$lib/features/journal-entries/AccountAutocomplete.svelte';
	import { isApiError } from '$lib/shared/utils/api-client';
	import type { AccountResponse } from '$lib/features/accounts/accounts.types';
	import { listProjects } from '$lib/features/projects/projects.api';
	import type { ProjectResponse } from '$lib/features/projects/projects.types';
	import type { ReconciliationProposal, SplitProposalLine } from './reconciliation.types';
	import { splitTransaction } from './reconciliation.api';

	type Props = {
		open: boolean;
		onOpenChange: (v: boolean) => void;
		bankAccountId: number;
		proposal: ReconciliationProposal | null;
		/** Pré-chargé par le parent (cohérent pattern ManualMatchModal). */
		accounts: AccountResponse[];
		onSuccess: () => void;
	};

	let { open, onOpenChange, bankAccountId, proposal, accounts, onSuccess }: Props = $props();

	const MIN_SPLITS = 2;
	const MAX_SPLITS = 50;
	const MAX_DESCRIPTION_LEN = 200;

	type SplitLine = {
		counterpartyAccountId: number | null;
		amount: string;
		description: string;
		projectId: number | null;
	};

	function blankLine(): SplitLine {
		return { counterpartyAccountId: null, amount: '', description: '', projectId: null };
	}

	let splits = $state<SplitLine[]>([blankLine(), blankLine()]);
	let valueDate = $state<string>('');
	let submitting = $state(false);
	let errorMsg = $state<string>('');
	let bankNotConfigured = $state(false);

	// Story 19-5 — projets analytiques pour le tag par ligne de ventilation.
	let projects = $state<ProjectResponse[]>([]);
	$effect(() => {
		listProjects()
			.then((p) => (projects = p))
			.catch(() => (projects = []));
	});

	const filteredAccounts = $derived(
		accounts.filter((a) => ['5', '6', '7'].some((c) => a.number.startsWith(c))),
	);

	// Reset à l'ouverture + pré-remplissage valueDate.
	$effect(() => {
		if (open && proposal) {
			splits = [blankLine(), blankLine()];
			errorMsg = '';
			bankNotConfigured = false;
			valueDate = proposal.transaction.valueDate ?? proposal.transaction.bookingDate;
		}
	});

	/** Montant total absolu de la transaction (string Decimal). */
	const txAbsAmount = $derived.by(() => {
		if (!proposal) return '0';
		const raw = proposal.transaction.amount;
		return raw.startsWith('-') ? raw.slice(1) : raw;
	});

	/**
	 * Somme des montants splits (parseFloat tolérant). Retourne `null` si
	 * une ligne a un montant non parsable — bloque submit via clientError.
	 */
	const sumAmounts = $derived.by(() => {
		let sum = 0;
		for (const s of splits) {
			const n = parseFloat(s.amount);
			if (!isFinite(n) || n <= 0) return null;
			sum += n;
		}
		// Arrondi à 2 décimales pour éviter les surprises IEEE 754
		// (la validation Decimal exacte est faite backend-side).
		return Math.round(sum * 100) / 100;
	});

	const balanceExact = $derived.by(() => {
		if (sumAmounts === null) return false;
		const expected = parseFloat(txAbsAmount);
		if (!isFinite(expected)) return false;
		return Math.abs(sumAmounts - expected) < 1e-6;
	});

	const balanceDiff = $derived.by(() => {
		if (sumAmounts === null) return '—';
		const expected = parseFloat(txAbsAmount);
		const diff = sumAmounts - expected;
		return (diff >= 0 ? '+' : '') + diff.toFixed(2);
	});

	const clientError = $derived.by(() => {
		if (!proposal)
			return i18nMsg(
				'reconciliation-split-error-no-proposal',
				'Aucune transaction sélectionnée',
			);
		if (splits.length < MIN_SPLITS) {
			return i18nMsg(
				'reconciliation-split-error-min-lines',
				`Au moins ${MIN_SPLITS} lignes requises`,
			);
		}
		if (splits.length > MAX_SPLITS) {
			return i18nMsg(
				'reconciliation-split-error-max-lines',
				`Maximum ${MAX_SPLITS} lignes`,
			);
		}
		for (let i = 0; i < splits.length; i++) {
			const s = splits[i];
			if (s.counterpartyAccountId === null) {
				return i18nMsg(
					'reconciliation-split-error-account-required',
					`Ligne ${i + 1} : compte requis`,
				);
			}
			const n = parseFloat(s.amount);
			if (!isFinite(n) || n <= 0) {
				return i18nMsg(
					'reconciliation-split-error-amount-positive',
					`Ligne ${i + 1} : montant > 0 requis`,
				);
			}
			if (s.description.length > MAX_DESCRIPTION_LEN) {
				return i18nMsg(
					'reconciliation-split-error-description-too-long',
					`Ligne ${i + 1} : description trop longue (max ${MAX_DESCRIPTION_LEN})`,
				);
			}
		}
		if (!balanceExact) {
			return i18nMsg(
				'reconciliation-split-error-imbalance',
				`Balance non équilibrée (écart ${balanceDiff})`,
			);
		}
		return '';
	});

	function addLine() {
		if (splits.length < MAX_SPLITS) {
			splits = [...splits, blankLine()];
		}
	}

	function removeLine(idx: number) {
		if (splits.length > MIN_SPLITS) {
			splits = splits.filter((_, i) => i !== idx);
		}
	}

	async function handleConfirm() {
		if (clientError || !proposal) return;
		submitting = true;
		errorMsg = '';
		bankNotConfigured = false;
		try {
			const splitInputs: SplitProposalLine[] = splits.map((s) => ({
				counterpartyAccountId: s.counterpartyAccountId as number,
				amount: s.amount,
				description: s.description,
				// Story 19-5 — projet par ligne (omis si null).
				...(s.projectId !== null ? { projectId: s.projectId } : {}),
			}));
			await splitTransaction(
				bankAccountId,
				proposal.bankTransactionId,
				splitInputs,
				valueDate || undefined,
			);
			onSuccess();
			onOpenChange(false);
		} catch (e) {
			if (isApiError(e) && e.code === 'BANK_ACCOUNT_NOT_CONFIGURED') {
				bankNotConfigured = true;
				errorMsg = e.message;
			} else if (isApiError(e)) {
				errorMsg = e.message;
			} else {
				errorMsg = e instanceof Error ? e.message : String(e);
			}
		} finally {
			submitting = false;
		}
	}
</script>

<Dialog.Root {open} {onOpenChange}>
	<Dialog.Content data-testid="transaction-split-modal">
		<Dialog.Header>
			<Dialog.Title>
				{i18nMsg('reconciliation-split-modal-title', 'Éclater la transaction')}
			</Dialog.Title>
		</Dialog.Header>

		{#if proposal}
			<div class="text-sm text-text-muted">
				<span data-testid="split-tx-summary">
					{proposal.transaction.bookingDate} — {proposal.transaction.amount}
					{proposal.transaction.currency}
				</span>
			</div>

			<div class="mt-2">
				<table class="w-full text-sm" data-testid="split-lines-table">
					<thead>
						<tr class="text-left text-xs text-text-muted">
							<th class="py-1">
								{i18nMsg('reconciliation-split-th-account', 'Compte')}
							</th>
							<th class="py-1">
								{i18nMsg('reconciliation-split-th-amount', 'Montant')}
							</th>
							<th class="py-1">
								{i18nMsg('reconciliation-split-th-description', 'Description')}
							</th>
							{#if projects.length > 0}
								<th class="py-1">
									{i18nMsg('reconciliation-split-th-project', 'Projet')}
								</th>
							{/if}
							<th class="py-1"></th>
						</tr>
					</thead>
					<tbody>
						{#each splits as line, i (i)}
							<tr data-testid={`split-row-${i}`}>
								<td class="py-1 pr-1">
									<AccountAutocomplete
										accounts={filteredAccounts}
										value={line.counterpartyAccountId}
										onSelect={(id) => (splits[i].counterpartyAccountId = id)}
										disabled={submitting}
									/>
								</td>
								<td class="py-1 pr-1">
									<Input
										data-testid={`split-amount-${i}`}
										type="number"
										step="0.01"
										bind:value={splits[i].amount}
										disabled={submitting}
									/>
								</td>
								<td class="py-1 pr-1">
									<Input
										data-testid={`split-description-${i}`}
										bind:value={splits[i].description}
										maxlength={MAX_DESCRIPTION_LEN}
										disabled={submitting}
									/>
								</td>
								{#if projects.length > 0}
									<td class="py-1 pr-1">
										<select
											class="w-full rounded border border-border px-2 py-1 text-sm"
											data-testid={`split-line-project-${i}`}
											bind:value={splits[i].projectId}
											disabled={submitting}
										>
											<option value={null}>
												{i18nMsg('reconciliation-split-project-none', '— Aucun')}
											</option>
											{#each projects.filter((p) => p.parentId === null) as root (root.id)}
												<option value={root.id}>{root.code} — {root.name}</option>
												{#each projects.filter((c) => c.parentId === root.id) as child (child.id)}
													<option value={child.id}>&nbsp;&nbsp;↳ {child.code} — {child.name}</option>
												{/each}
											{/each}
										</select>
									</td>
								{/if}
								<td class="py-1">
									<Button
										variant="ghost"
										size="sm"
										onclick={() => removeLine(i)}
										disabled={submitting || splits.length <= MIN_SPLITS}
										data-testid={`split-remove-${i}`}
									>
										{i18nMsg('reconciliation-split-remove-line', '-')}
									</Button>
								</td>
							</tr>
						{/each}
					</tbody>
				</table>
				<Button
					variant="outline"
					size="sm"
					onclick={addLine}
					disabled={submitting || splits.length >= MAX_SPLITS}
					data-testid="split-add-line"
				>
					{i18nMsg('reconciliation-split-add-line', '+ Ajouter une ligne')}
				</Button>
			</div>

			<div class="mt-2">
				<label class="mb-1 block text-xs text-text-muted" for="split-value-date">
					{i18nMsg('reconciliation-split-value-date-label', 'Date de valeur')}
				</label>
				<Input
					id="split-value-date"
					data-testid="split-value-date-input"
					type="date"
					bind:value={valueDate}
					disabled={submitting}
				/>
			</div>

			<div
				class={`mt-2 rounded-md px-3 py-2 text-sm ${balanceExact ? 'border border-green-500 bg-green-500/10 text-green-700' : 'border border-destructive bg-destructive/10 text-destructive'}`}
				data-testid="split-balance-indicator"
				aria-live="polite"
			>
				{#if balanceExact}
					{i18nMsg(
						'reconciliation-split-balance-indicator',
						`Balance équilibrée : ${txAbsAmount}`,
					)}
				{:else}
					{i18nMsg(
						'reconciliation-split-balance-indicator',
						`Balance non équilibrée — attendu ${txAbsAmount}, écart ${balanceDiff}`,
					)}
				{/if}
			</div>

			{#if bankNotConfigured}
				<div
					class="rounded-md border border-destructive bg-destructive/10 px-3 py-2 text-sm text-destructive"
					data-testid="split-bank-not-configured"
				>
					{i18nMsg(
						'reconciliation-split-bank-account-not-configured',
						"Le compte bancaire n'est pas configuré. Configurer le compte comptable lié dans /bank-accounts.",
					)}
					<a class="underline" href="/bank-accounts">/bank-accounts</a>
				</div>
			{:else if clientError}
				<div class="rounded-md border border-destructive bg-destructive/10 px-3 py-2 text-sm text-destructive">
					{clientError}
				</div>
			{:else if errorMsg}
				<div
					class="rounded-md border border-destructive bg-destructive/10 px-3 py-2 text-sm text-destructive"
					data-testid="split-error"
				>
					{errorMsg}
				</div>
			{/if}
		{/if}

		<Dialog.Footer>
			<Button
				variant="outline"
				onclick={() => onOpenChange(false)}
				disabled={submitting}
				data-testid="split-cancel"
			>
				{i18nMsg('common-cancel', 'Annuler')}
			</Button>
			<Button
				onclick={handleConfirm}
				disabled={submitting || !!clientError}
				data-testid="split-submit"
			>
				{i18nMsg('reconciliation-split-button-label', 'Éclater')}
			</Button>
		</Dialog.Footer>
	</Dialog.Content>
</Dialog.Root>
