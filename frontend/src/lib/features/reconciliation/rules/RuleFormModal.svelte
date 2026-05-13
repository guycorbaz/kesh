<!--
  Story 8-5b FR47 — Modal CREATE/EDIT pour reconciliation_rules.
-->
<script lang="ts">
	import { i18nMsg } from '$lib/shared/utils/i18n.svelte';
	import type { AccountResponse } from '$lib/features/accounts/accounts.types';
	import { createRule, updateRule } from './rules.api';
	import type { MatchType, ReconciliationRule } from './rules.types';

	interface Props {
		rule?: ReconciliationRule | null;
		accounts: AccountResponse[];
		onSuccess: () => void;
		onCancel: () => void;
	}

	let { rule, accounts, onSuccess, onCancel }: Props = $props();

	const isEdit = rule !== null && rule !== undefined;

	let label = $state(rule?.label ?? '');
	let matchType = $state<MatchType>(rule?.matchType ?? 'counterparty_contains');
	let matchValue = $state(rule?.matchValue ?? '');
	let counterpartyAccountId = $state<number | null>(rule?.counterpartyAccountId ?? null);
	let priority = $state(rule?.priority ?? 100);

	let submitting = $state(false);
	let errorMsg = $state<string | null>(null);

	const eligibleAccounts = $derived(
		accounts.filter(
			(a) =>
				a.active &&
				(a.accountType === 'Expense' || a.accountType === 'Revenue'),
		),
	);

	async function handleSubmit(e: SubmitEvent) {
		e.preventDefault();
		errorMsg = null;
		if (!label.trim()) {
			errorMsg = i18nMsg('reconciliation-rules-error-label-required', 'Libellé requis');
			return;
		}
		if (!matchValue.trim()) {
			errorMsg = i18nMsg(
				'reconciliation-rules-error-match-value-required',
				'Valeur requise',
			);
			return;
		}
		if (counterpartyAccountId === null) {
			errorMsg = i18nMsg(
				'reconciliation-rules-error-counterparty-required',
				'Compte de contrepartie requis',
			);
			return;
		}
		submitting = true;
		try {
			if (isEdit && rule) {
				await updateRule(rule.id, {
					expectedVersion: rule.version,
					label,
					matchValue,
					counterpartyAccountId,
					priority,
				});
			} else {
				await createRule({
					label,
					matchType,
					matchValue,
					counterpartyAccountId,
					priority,
				});
			}
			onSuccess();
		} catch (e: unknown) {
			errorMsg = e instanceof Error ? e.message : String(e);
		} finally {
			submitting = false;
		}
	}
</script>

<form
	onsubmit={handleSubmit}
	class="space-y-3 rounded border border-border bg-surface p-4"
	data-testid="rule-form-modal"
>
	<h3 class="font-semibold">
		{isEdit
			? i18nMsg('reconciliation-rules-form-title-edit', 'Modifier la règle')
			: i18nMsg('reconciliation-rules-form-title-create', 'Nouvelle règle')}
	</h3>

	<div>
		<label for="rule-label" class="block text-sm font-medium">
			{i18nMsg('reconciliation-rules-labels-label', 'Libellé')}
		</label>
		<input
			id="rule-label"
			type="text"
			bind:value={label}
			class="mt-1 w-full rounded border border-border px-2 py-1 text-sm"
			maxlength="120"
			data-testid="rule-form-label"
		/>
	</div>

	{#if !isEdit}
		<div>
			<label for="rule-match-type" class="block text-sm font-medium">
				{i18nMsg('reconciliation-rules-labels-match-type', 'Type')}
			</label>
			<select
				id="rule-match-type"
				bind:value={matchType}
				class="mt-1 w-full rounded border border-border px-2 py-1 text-sm"
				data-testid="rule-form-match-type"
			>
				<option value="counterparty_contains">
					{i18nMsg(
						'reconciliation-rules-match-type-counterparty-contains',
						'Contrepartie contient',
					)}
				</option>
				<option value="counterparty_exact">
					{i18nMsg(
						'reconciliation-rules-match-type-counterparty-exact',
						'Contrepartie exacte',
					)}
				</option>
				<option value="reference_contains">
					{i18nMsg(
						'reconciliation-rules-match-type-reference-contains',
						'Référence contient',
					)}
				</option>
				<option value="iban_exact">
					{i18nMsg('reconciliation-rules-match-type-iban-exact', 'IBAN exact')}
				</option>
			</select>
		</div>
	{/if}

	<div>
		<label for="rule-match-value" class="block text-sm font-medium">
			{i18nMsg('reconciliation-rules-labels-match-value', 'Valeur')}
		</label>
		<input
			id="rule-match-value"
			type="text"
			bind:value={matchValue}
			class="mt-1 w-full rounded border border-border px-2 py-1 text-sm"
			maxlength="255"
			data-testid="rule-form-match-value"
		/>
	</div>

	<div>
		<label for="rule-counterparty" class="block text-sm font-medium">
			{i18nMsg(
				'reconciliation-rules-labels-counterparty-account',
				'Compte de contrepartie',
			)}
		</label>
		<select
			id="rule-counterparty"
			bind:value={counterpartyAccountId}
			class="mt-1 w-full rounded border border-border px-2 py-1 text-sm"
			data-testid="rule-form-counterparty"
		>
			<option value={null}>—</option>
			{#each eligibleAccounts as a (a.id)}
				<option value={a.id}>{a.number} — {a.name}</option>
			{/each}
		</select>
	</div>

	<div>
		<label for="rule-priority" class="block text-sm font-medium">
			{i18nMsg('reconciliation-rules-labels-priority', 'Priorité')}
		</label>
		<input
			id="rule-priority"
			type="number"
			min="1"
			max="1000"
			bind:value={priority}
			class="mt-1 w-32 rounded border border-border px-2 py-1 text-sm"
			data-testid="rule-form-priority"
		/>
		<span class="ml-2 text-xs text-text-muted">
			{i18nMsg(
				'reconciliation-rules-labels-priority-hint',
				'Valeur plus basse = priorité plus haute (1-1000)',
			)}
		</span>
	</div>

	{#if errorMsg}
		<p class="text-sm text-error" data-testid="rule-form-error">{errorMsg}</p>
	{/if}

	<div class="flex justify-end gap-2">
		<button
			type="button"
			class="rounded border border-border px-3 py-1 text-sm"
			onclick={onCancel}
			disabled={submitting}
			data-testid="rule-form-cancel"
		>
			{i18nMsg('reconciliation-rules-actions-cancel', 'Annuler')}
		</button>
		<button
			type="submit"
			class="rounded bg-primary px-3 py-1 text-sm text-white"
			disabled={submitting}
			data-testid="rule-form-submit"
		>
			{isEdit
				? i18nMsg('reconciliation-rules-actions-save', 'Enregistrer')
				: i18nMsg('reconciliation-rules-actions-create', 'Créer')}
		</button>
	</div>
</form>
