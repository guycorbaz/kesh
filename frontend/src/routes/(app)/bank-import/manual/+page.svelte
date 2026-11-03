<!--
  Story v016-2b (#816) — `/bank-import/manual` : saisir un relevé bancaire à la
  main, pour un compte dont la banque ne fournit ni CAMT.053 ni CSV. Garde
  Comptable+ dans `+page.ts`. Après l'enregistrement : notification, retour à
  `/bank-import`, où le relevé paraît en tête de la liste.
-->
<script lang="ts">
	import { onMount } from 'svelte';
	import { goto } from '$app/navigation';
	import ManualStatementForm from '$lib/features/bank-import/ManualStatementForm.svelte';
	import { apiClient } from '$lib/shared/utils/api-client';
	import { i18nMsg } from '$lib/shared/utils/i18n.svelte';
	import { notifySuccess } from '$lib/shared/utils/notify';
	import type { BankImportResponse } from '$lib/features/bank-import/bank-import.types';
	import type { BankAccountOptionSource } from '$lib/features/bank-accounts/format';
	import type { CompanyCurrentResponse } from '$lib/features/settings/settings.types';

	let bankAccounts = $state<Array<BankAccountOptionSource & { id: number }>>([]);
	let loading = $state(true);
	// Revue de code P1, B9 : un échec du chargement des comptes se dit, il ne laisse pas un
	// sélecteur vide sans explication.
	let loadFailed = $state(false);

	onMount(async () => {
		try {
			// Les comptes ni archivés ni clos (`docs/api-external.md`), comme l'import.
			const company = await apiClient.get<CompanyCurrentResponse>('/api/v1/companies/current');
			bankAccounts = company.bankAccounts ?? [];
		} catch {
			bankAccounts = [];
			loadFailed = true;
		}
		loading = false;
	});

	function onSaved(imported: BankImportResponse): void {
		notifySuccess(
			i18nMsg('bank-import-manual-success', 'Relevé enregistré : { $count } transaction(s).', {
				count: imported.transactionCount,
			}),
		);
		void goto('/bank-import');
	}
</script>

<svelte:head>
	<title>{i18nMsg('bank-import-manual-page-title', 'Saisir un relevé bancaire à la main')} — Kesh</title>
</svelte:head>

<a href="/bank-import" class="text-primary underline" data-testid="bank-import-manual-back">
	{i18nMsg('bank-import-manual-back', '← Retour aux imports')}
</a>

<h1 class="mt-2 text-2xl font-semibold text-text" data-testid="bank-import-manual-title">
	{i18nMsg('bank-import-manual-page-title', 'Saisir un relevé bancaire à la main')}
</h1>

<p class="mt-2 max-w-3xl text-sm text-text-muted">
	{i18nMsg(
		'bank-import-manual-intro',
		"Pour un compte dont la banque ne fournit ni CAMT.053 ni CSV : recopiez le relevé papier ou PDF. Le solde de départ, augmenté des crédits et diminué des débits, doit donner exactement le solde d'arrivée. Les montants sont en CHF.",
	)}
</p>

{#if loading}
	<p class="mt-4 text-text-muted">{i18nMsg('common-loading', 'Chargement…')}</p>
{:else}
	{#if loadFailed}
		<p class="mt-4 rounded border border-destructive bg-destructive/10 p-3 text-sm text-destructive" role="alert" data-testid="bank-import-manual-load-error">
			{i18nMsg(
				'bank-import-manual-load-error',
				"Les comptes bancaires n'ont pas pu être chargés : rechargez la page.",
			)}
		</p>
	{/if}
	<div class="mt-6">
		<ManualStatementForm {bankAccounts} {onSaved} />
	</div>
{/if}
