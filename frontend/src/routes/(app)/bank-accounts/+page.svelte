<!--
  Story 8-5a-zero — Page de configuration des comptes bancaires.

  Réécriture complète du placeholder Epic 6 « Payer » qui squattait ce path
  par accident historique (F2''' Pass 3 Opus 4.7). Epic 6 paiements
  `pain.001` utilisera une autre route (ex. `/payments` ou `/payer`) à
  coordonner lors de la planification Epic 6.
-->
<script lang="ts">
	import { onMount } from 'svelte';
	import { i18nMsg } from '$lib/shared/utils/i18n.svelte';
	import { isApiError } from '$lib/shared/utils/api-client';
	import { fetchAccounts } from '$lib/features/accounts/accounts.api';
	import type { AccountResponse } from '$lib/features/accounts/accounts.types';
	import BankAccountList from '$lib/features/bank-accounts/BankAccountList.svelte';
	import {
		listBankAccounts,
		type BankAccountSummary,
	} from '$lib/features/bank-accounts/bank-accounts.api';

	let bankAccounts = $state<BankAccountSummary[]>([]);
	let accounts = $state<AccountResponse[]>([]);
	let loading = $state(true);
	let loadError = $state<string | null>(null);

	onMount(async () => {
		try {
			const [ba, acc] = await Promise.all([listBankAccounts(), fetchAccounts(false)]);
			bankAccounts = ba;
			accounts = acc;
		} catch (err) {
			loadError = isApiError(err) ? err.message : String(err);
		} finally {
			loading = false;
		}
	});

	function handleUpdated(updated: BankAccountSummary) {
		bankAccounts = bankAccounts.map((ba) => (ba.id === updated.id ? updated : ba));
	}
</script>

<svelte:head>
	<title>{i18nMsg('bank-accounts-labels-page-title', 'Comptes bancaires')} - Kesh</title>
</svelte:head>

<h1 class="text-2xl font-semibold text-text" data-testid="bank-accounts-page-title">
	{i18nMsg('bank-accounts-labels-page-title', 'Comptes bancaires')}
</h1>
<p class="mt-2 text-sm text-text-muted">
	{i18nMsg(
		'bank-accounts-labels-page-subtitle',
		'Lier chaque compte bancaire à un compte du plan comptable (classe 1 typique : 1020 Caisse, 1030 Banque).',
	)}
</p>

<div class="mt-6">
	{#if loading}
		<p class="text-text-muted">
			{i18nMsg('bank-accounts-labels-loading', 'Chargement…')}
		</p>
	{:else if loadError}
		<p class="text-red-600" role="alert" data-testid="load-error">{loadError}</p>
	{:else}
		<BankAccountList {bankAccounts} {accounts} onUpdated={handleUpdated} />
	{/if}
</div>
