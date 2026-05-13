<!--
  Story 8-5b FR47 — Page /reconciliation/rules : CRUD règles d'affectation.
-->
<script lang="ts">
	import { onMount } from 'svelte';
	import { i18nMsg } from '$lib/shared/utils/i18n.svelte';
	import { fetchAccounts } from '$lib/features/accounts/accounts.api';
	import type { AccountResponse } from '$lib/features/accounts/accounts.types';
	import RulesList from '$lib/features/reconciliation/rules/RulesList.svelte';
	import RuleFormModal from '$lib/features/reconciliation/rules/RuleFormModal.svelte';
	import { listRules } from '$lib/features/reconciliation/rules/rules.api';
	import type { ReconciliationRule } from '$lib/features/reconciliation/rules/rules.types';

	let rules = $state<ReconciliationRule[]>([]);
	let accounts = $state<AccountResponse[]>([]);
	let loading = $state(true);
	let loadError = $state<string | null>(null);
	let showForm = $state(false);
	let editingRule = $state<ReconciliationRule | null>(null);

	async function loadAll() {
		loading = true;
		loadError = null;
		try {
			const [rulesResp, accountsResp] = await Promise.all([
				listRules(1, 200),
				fetchAccounts(),
			]);
			rules = rulesResp.items;
			accounts = accountsResp;
		} catch (e: unknown) {
			loadError = e instanceof Error ? e.message : String(e);
		} finally {
			loading = false;
		}
	}

	onMount(loadAll);

	function handleCreate() {
		editingRule = null;
		showForm = true;
	}

	function handleEdit(rule: ReconciliationRule) {
		editingRule = rule;
		showForm = true;
	}

	function handleSuccess() {
		showForm = false;
		editingRule = null;
		loadAll();
	}

	function handleCancel() {
		showForm = false;
		editingRule = null;
	}
</script>

<svelte:head>
	<title
		>{i18nMsg('reconciliation-rules-page-title', 'Règles d’affectation')} — Kesh</title
	>
</svelte:head>

<div class="space-y-4 p-6">
	<header class="flex items-center justify-between">
		<h1 class="text-xl font-bold">
			{i18nMsg('reconciliation-rules-page-title', 'Règles d’affectation')}
		</h1>
		{#if !showForm}
			<button
				type="button"
				class="rounded bg-primary px-3 py-2 text-sm text-white"
				onclick={handleCreate}
				data-testid="rules-create-button"
			>
				{i18nMsg('reconciliation-rules-actions-new', 'Nouvelle règle')}
			</button>
		{/if}
	</header>

	{#if loadError}
		<p class="text-error" data-testid="rules-page-error">{loadError}</p>
	{/if}

	{#if showForm}
		<RuleFormModal
			rule={editingRule}
			{accounts}
			onSuccess={handleSuccess}
			onCancel={handleCancel}
		/>
	{/if}

	{#if loading}
		<p class="text-text-muted">
			{i18nMsg('reconciliation-rules-loading', 'Chargement…')}
		</p>
	{:else}
		<RulesList {rules} {accounts} onEdit={handleEdit} onRefresh={loadAll} />
	{/if}
</div>
