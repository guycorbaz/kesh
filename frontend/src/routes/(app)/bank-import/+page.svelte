<script lang="ts">
	import { onMount } from 'svelte';
	import BankImportUpload from '$lib/features/bank-import/BankImportUpload.svelte';
	import BankImportList from '$lib/features/bank-import/BankImportList.svelte';
	import { listBankImports } from '$lib/features/bank-import/bank-import.api';
	import { apiClient } from '$lib/shared/utils/api-client';
	import { i18nMsg } from '$lib/shared/utils/i18n.svelte';
	import type { BankImportResponse } from '$lib/features/bank-import/bank-import.types';

	type BankAccountListItem = { id: number; bankName: string; iban: string };

	let bankAccounts = $state<BankAccountListItem[]>([]);
	let imports = $state<BankImportResponse[]>([]);
	let loading = $state(true);

	async function refresh(): Promise<void> {
		const list = await listBankImports();
		imports = list.items;
	}

	onMount(async () => {
		try {
			// Story 1-9 : `/api/v1/companies/current` retourne l'objet
			// company, mais bank_accounts est listé via un endpoint dédié.
			// Pour v0.1 on pioche depuis l'onboarding bank-account
			// (présent dans la company). Story 8-1b assume au moins 1
			// bank_account configuré côté onboarding (Path B production).
			// Pas d'endpoint dédié `GET /api/v1/bank-accounts` v0.1 →
			// fallback : liste vide si non disponible. À étendre si besoin.
			type CompanyResponse = {
				bankAccounts?: BankAccountListItem[];
			};
			const company = await apiClient.get<CompanyResponse>('/api/v1/companies/current');
			bankAccounts = company.bankAccounts ?? [];
		} catch {
			bankAccounts = [];
		}
		await refresh();
		loading = false;
	});

	function onSuccess(): void {
		void refresh();
	}
</script>

<svelte:head>
	<title>Import bancaire - Kesh</title>
</svelte:head>

<h1 class="text-2xl font-semibold text-text" data-testid="bank-import-page-title">
	{i18nMsg('bank-import-labels-page-title', 'Import bancaire CAMT.053')}
</h1>

{#if loading}
	<p class="mt-4 text-text-muted">Chargement…</p>
{:else}
	<div class="mt-6">
		<BankImportUpload {bankAccounts} {onSuccess} />
	</div>

	<section class="mt-10">
		<h2 class="text-lg font-semibold">
			{i18nMsg('bank-import-labels-list-title', 'Imports précédents')}
		</h2>
		<BankImportList {imports} />
	</section>
{/if}
