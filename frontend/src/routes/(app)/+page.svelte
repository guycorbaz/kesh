<script lang="ts">
	import { onMount } from 'svelte';
	import { Button } from '$lib/components/ui/button';
	import { modeState } from '$lib/app/stores/mode.svelte';
	import { i18nMsg } from '$lib/features/onboarding/onboarding.svelte';
	import { fetchCompanyCurrent } from '$lib/features/settings/settings.api';
	import type { BankAccountJson } from '$lib/features/settings/settings.types';

	function msg(key: string, fallback: string): string {
		return i18nMsg(key, fallback);
	}

	let bankAccounts = $state<BankAccountJson[]>([]);
	let bankLoaded = $state(false);

	onMount(async () => {
		try {
			const data = await fetchCompanyCurrent();
			bankAccounts = data.bankAccounts;
		} catch {
			// Company may not exist yet (404) — that's OK
		} finally {
			bankLoaded = true;
		}
	});

	let isGuided = $derived(modeState.value === 'guided');
</script>

<svelte:head>
	<title>Accueil - Kesh</title>
</svelte:head>

<h1 class="mb-6 text-2xl font-semibold text-text">
	{msg('homepage-title', 'Tableau de bord')}
</h1>

<div class="grid grid-cols-1 gap-6 lg:grid-cols-3">
	<!-- Widget : Dernières écritures -->
	<div class="rounded-lg border border-border bg-white p-6 shadow-sm">
		<h3 class="text-lg font-semibold text-text">
			{msg('homepage-entries-title', 'Dernières écritures')}
		</h3>
		{#if isGuided}
			<p class="mt-2 text-sm text-text-muted">
				{msg('homepage-entries-empty-guided', 'Aucune écriture pour le moment. Commencez par saisir votre première écriture comptable.')}
			</p>
		{:else}
			<p class="mt-2 text-sm text-text-muted">
				{msg('homepage-entries-empty', 'Aucune écriture.')}
			</p>
		{/if}
		<Button variant="outline" class="mt-4" href="/journal-entries">
			{msg('homepage-entries-action', 'Saisir une écriture')}
		</Button>
	</div>

	<!-- Widget : Factures ouvertes -->
	<div class="rounded-lg border border-border bg-white p-6 shadow-sm">
		<h3 class="text-lg font-semibold text-text">
			{msg('homepage-invoices-title', 'Factures ouvertes')}
		</h3>
		{#if isGuided}
			<p class="mt-2 text-sm text-text-muted">
				{msg('homepage-invoices-empty-guided', 'Aucune facture ouverte. Créez votre première facture pour facturer vos clients.')}
			</p>
		{:else}
			<p class="mt-2 text-sm text-text-muted">
				{msg('homepage-invoices-empty', 'Aucune facture ouverte.')}
			</p>
		{/if}
		<Button variant="outline" class="mt-4" href="/invoices">
			{msg('homepage-invoices-action', 'Créer une facture')}
		</Button>
	</div>

	<!-- Widget : Soldes comptes bancaires -->
	<div class="rounded-lg border border-border bg-white p-6 shadow-sm">
		<h3 class="text-lg font-semibold text-text">
			{msg('homepage-bank-title', 'Comptes bancaires')}
		</h3>
		{#if !bankLoaded}
			<p class="mt-2 text-sm text-text-muted">Chargement...</p>
		{:else if bankAccounts.length > 0}
			{#each bankAccounts as account}
				<div class="mt-2">
					<p class="font-medium">{account.bankName}</p>
					<p class="text-sm text-text-muted">{account.iban}</p>
					<p class="text-xs text-text-muted">
						{msg('homepage-bank-no-transactions', 'Aucune transaction importée')}
					</p>
				</div>
			{/each}
		{:else if isGuided}
			<p class="mt-2 text-sm text-text-muted">
				{msg('homepage-bank-empty-guided', 'Aucun compte bancaire configuré. Ajoutez votre compte pour importer vos relevés.')}
			</p>
		{/if}
		<Button variant="outline" class="mt-4" href="/settings">
			{msg('homepage-bank-action', 'Configurer')}
		</Button>
	</div>
</div>
