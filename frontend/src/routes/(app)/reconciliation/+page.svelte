<!--
  Story 8-4 (FR44) — Page de réconciliation automatique.
  Sélection du compte bancaire + liste des propositions matching.
-->
<script lang="ts">
	import { onMount } from 'svelte';
	import ReconciliationProposals from '$lib/features/reconciliation/ReconciliationProposals.svelte';
	import { apiClient } from '$lib/shared/utils/api-client';
	import { i18nMsg } from '$lib/shared/utils/i18n.svelte';

	type BankAccountListItem = { id: number; bankName: string; iban: string };

	let bankAccounts = $state<BankAccountListItem[]>([]);
	let selectedId = $state<number | null>(null);
	let loading = $state(true);

	onMount(async () => {
		try {
			type CompanyResponse = { bankAccounts?: BankAccountListItem[] };
			const company = await apiClient.get<CompanyResponse>('/api/v1/companies/current');
			bankAccounts = company.bankAccounts ?? [];
			if (bankAccounts.length > 0) selectedId = bankAccounts[0].id;
		} catch {
			bankAccounts = [];
		}
		loading = false;
	});
</script>

<svelte:head>
	<title>{i18nMsg('reconciliation-page-title', 'Réconciliation')} - Kesh</title>
</svelte:head>

<h1 class="text-2xl font-semibold text-text">
	{i18nMsg('reconciliation-page-title', 'Réconciliation')}
</h1>
<p class="mt-2 text-sm text-text-muted">
	{i18nMsg(
		'reconciliation-page-subtitle',
		'Propositions automatiques de matching transaction ↔ facture.',
	)}
</p>

{#if loading}
	<p class="mt-4 text-text-muted">…</p>
{:else if bankAccounts.length === 0}
	<p class="mt-4 text-text-muted" data-testid="reconciliation-no-account">
		{i18nMsg(
			'reconciliation-labels-no-account',
			'Aucun compte bancaire configuré.',
		)}
	</p>
{:else}
	<div class="mt-4 mb-6">
		<label class="mb-1 block text-sm" for="bank-account-select">
			{i18nMsg('reconciliation-labels-account-select', 'Compte bancaire')}
		</label>
		<select
			id="bank-account-select"
			class="rounded border border-text px-3 py-2"
			bind:value={selectedId}
			data-testid="bank-account-select"
		>
			{#each bankAccounts as ba (ba.id)}
				<option value={ba.id}>{ba.bankName} — {ba.iban}</option>
			{/each}
		</select>
	</div>
	{#if selectedId}
		{#key selectedId}
			<ReconciliationProposals bankAccountId={selectedId} />
		{/key}
	{/if}
{/if}
