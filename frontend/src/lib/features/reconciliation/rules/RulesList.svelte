<!--
  Story 8-5b FR47 — Liste des reconciliation_rules avec actions
  (PATCH active toggle + DELETE soft). Click « Modifier » ouvre
  `RuleFormModal` en mode édition.
-->
<script lang="ts">
	import { i18nMsg } from '$lib/shared/utils/i18n.svelte';
	import type { AccountResponse } from '$lib/features/accounts/accounts.types';
	import { deleteRule, updateRule } from './rules.api';
	import type { ReconciliationRule } from './rules.types';

	interface Props {
		rules: ReconciliationRule[];
		accounts: AccountResponse[];
		onEdit: (rule: ReconciliationRule) => void;
		onRefresh: () => void;
	}

	let { rules, accounts, onEdit, onRefresh }: Props = $props();

	let busyId = $state<number | null>(null);
	let errorMsg = $state<string | null>(null);

	function accountLabel(counterpartyAccountId: number): string {
		const acc = accounts.find((a) => a.id === counterpartyAccountId);
		if (!acc) return `#${counterpartyAccountId}`;
		return `${acc.number} — ${acc.name}`;
	}

	function matchTypeLabel(t: string): string {
		switch (t) {
			case 'counterparty_contains':
				return i18nMsg(
					'reconciliation-rules-match-type-counterparty-contains',
					'Contrepartie contient',
				);
			case 'counterparty_exact':
				return i18nMsg(
					'reconciliation-rules-match-type-counterparty-exact',
					'Contrepartie exacte',
				);
			case 'reference_contains':
				return i18nMsg(
					'reconciliation-rules-match-type-reference-contains',
					'Référence contient',
				);
			case 'iban_exact':
				return i18nMsg('reconciliation-rules-match-type-iban-exact', 'IBAN exact');
			default:
				return t;
		}
	}

	async function handleToggleActive(rule: ReconciliationRule) {
		busyId = rule.id;
		errorMsg = null;
		try {
			await updateRule(rule.id, {
				expectedVersion: rule.version,
				active: !rule.active,
			});
			onRefresh();
		} catch (e: unknown) {
			errorMsg = e instanceof Error ? e.message : String(e);
		} finally {
			busyId = null;
		}
	}

	async function handleDelete(rule: ReconciliationRule) {
		if (
			!confirm(
				i18nMsg(
					'reconciliation-rules-confirm-delete',
					'Archiver cette règle ? Les écritures déjà appliquées sont préservées.',
				),
			)
		) {
			return;
		}
		busyId = rule.id;
		errorMsg = null;
		try {
			await deleteRule(rule.id);
			onRefresh();
		} catch (e: unknown) {
			errorMsg = e instanceof Error ? e.message : String(e);
		} finally {
			busyId = null;
		}
	}
</script>

{#if errorMsg}
	<p class="mb-3 text-sm text-error" data-testid="rules-list-error">{errorMsg}</p>
{/if}

{#if rules.length === 0}
	<p class="text-text-muted" data-testid="rules-empty">
		{i18nMsg('reconciliation-rules-labels-empty', 'Aucune règle configurée.')}
	</p>
{:else}
	<table class="w-full text-sm" data-testid="rules-list">
		<thead>
			<tr class="border-b border-border text-left">
				<th class="py-2 pr-4 font-semibold"
					>{i18nMsg('reconciliation-rules-labels-label', 'Libellé')}</th
				>
				<th class="py-2 pr-4 font-semibold"
					>{i18nMsg('reconciliation-rules-labels-match-type', 'Type')}</th
				>
				<th class="py-2 pr-4 font-semibold"
					>{i18nMsg('reconciliation-rules-labels-match-value', 'Valeur')}</th
				>
				<th class="py-2 pr-4 font-semibold"
					>{i18nMsg(
						'reconciliation-rules-labels-counterparty-account',
						'Compte de contrepartie',
					)}</th
				>
				<th class="py-2 pr-4 font-semibold"
					>{i18nMsg('reconciliation-rules-labels-priority', 'Priorité')}</th
				>
				<th class="py-2 pr-4 font-semibold"
					>{i18nMsg('reconciliation-rules-labels-applied-count', 'Appliquée')}</th
				>
				<th class="py-2 pr-4 font-semibold"
					>{i18nMsg('reconciliation-rules-labels-status', 'État')}</th
				>
				<th class="py-2"></th>
			</tr>
		</thead>
		<tbody>
			{#each rules as r (r.id)}
				<tr
					class="border-b border-border {!r.active ? 'opacity-60' : ''}"
					data-testid="rule-row-{r.id}"
				>
					<td class="py-2 pr-4">{r.label}</td>
					<td class="py-2 pr-4">{matchTypeLabel(r.matchType)}</td>
					<td class="py-2 pr-4 font-mono text-xs">{r.matchValue}</td>
					<td class="py-2 pr-4">{accountLabel(r.counterpartyAccountId)}</td>
					<td class="py-2 pr-4">{r.priority}</td>
					<td class="py-2 pr-4">{r.appliedCount}</td>
					<td class="py-2 pr-4">
						{r.active
							? i18nMsg('reconciliation-rules-labels-active', 'Active')
							: i18nMsg('reconciliation-rules-labels-archived', 'Archivée')}
					</td>
					<td class="py-2 space-x-1">
						<button
							type="button"
							class="rounded border border-border px-2 py-1 text-xs"
							onclick={() => onEdit(r)}
							disabled={busyId === r.id}
							data-testid="edit-button-{r.id}"
						>
							{i18nMsg('reconciliation-rules-actions-edit', 'Modifier')}
						</button>
						<button
							type="button"
							class="rounded border border-border px-2 py-1 text-xs"
							onclick={() => handleToggleActive(r)}
							disabled={busyId === r.id}
							data-testid="toggle-active-button-{r.id}"
						>
							{r.active
								? i18nMsg('reconciliation-rules-actions-deactivate', 'Désactiver')
								: i18nMsg('reconciliation-rules-actions-reactivate', 'Réactiver')}
						</button>
						{#if r.active}
							<button
								type="button"
								class="rounded border border-error px-2 py-1 text-xs text-error"
								onclick={() => handleDelete(r)}
								disabled={busyId === r.id}
								data-testid="delete-button-{r.id}"
							>
								{i18nMsg('reconciliation-rules-actions-archive', 'Archiver')}
							</button>
						{/if}
					</td>
				</tr>
			{/each}
		</tbody>
	</table>
{/if}
