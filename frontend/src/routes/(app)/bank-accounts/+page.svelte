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
		// P-M5 Pass 1 code review Sonnet 4.6 : Promise.allSettled au lieu de
		// Promise.all pour dégradation gracieuse. Si `fetchAccounts` échoue
		// mais `listBankAccounts` réussit, on affiche la liste avec
		// `accountLabel` qui retournera `#X` pour les ids non résolus
		// (acceptable v0.1). Si `listBankAccounts` échoue, on affiche
		// l'erreur principale.
		const [baResult, accResult] = await Promise.allSettled([
			listBankAccounts(),
			fetchAccounts(false),
		]);
		if (baResult.status === 'fulfilled') {
			bankAccounts = baResult.value;
		} else {
			loadError = isApiError(baResult.reason)
				? baResult.reason.message
				: String(baResult.reason);
		}
		if (accResult.status === 'fulfilled') {
			accounts = accResult.value;
		} else {
			// Liste vide : `accountLabel` retournera `#X` pour les ids posés.
			// On ne propage l'erreur dans `loadError` que si la liste des
			// bank_accounts est elle aussi en échec (sinon on affiche la liste
			// avec labels dégradés plutôt que de bloquer la page entière).
			accounts = [];
		}
		loading = false;
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
