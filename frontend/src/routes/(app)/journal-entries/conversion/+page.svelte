<script lang="ts">
	/**
	 * Story 37-3b (AC-2 bis) — page de l'assistant *Conversion* : charge les
	 * comptes, la société (comptes de conversion et de différence) et les
	 * exercices ouverts, puis monte l'assistant.
	 */
	import { onMount } from 'svelte';
	import { goto } from '$app/navigation';
	import { ArrowLeft } from '@lucide/svelte';
	import { Button } from '$lib/components/ui/button';
	import { i18nMsg } from '$lib/shared/utils/i18n.svelte';
	import { todayLocal } from '$lib/shared/utils/local-date';
	import { authState } from '$lib/app/stores/auth.svelte';
	import { fetchAccounts } from '$lib/features/accounts/accounts.api';
	// fx-filtre-comptes : exempté — passe les comptes à l'assistant Conversion, qui
	// choisit ses comptes en devise par devise (37-3b, AC-2 bis).
	import type { AccountResponse } from '$lib/features/accounts/accounts.types';
	import { fetchCompanyCurrent } from '$lib/features/settings/settings.api';
	import type { CompanyJson } from '$lib/features/settings/settings.types';
	import { listFiscalYears } from '$lib/features/fiscal-years/fiscal-years.api';
	import { canEditRates } from '$lib/features/exchange-rates/exchange-rates-access';
	import { fetchRates, lookupRate } from '$lib/features/exchange-rates/exchange-rates.api';
	import { createRateLookupCache } from '$lib/features/journal-entries/rate-lookup';
	import ConversionAssistant from '$lib/features/journal-entries/ConversionAssistant.svelte';

	let accounts = $state<AccountResponse[]>([]);
	let accountsLoadError = $state(false);
	let company = $state<CompanyJson | null>(null);
	let openFiscalYears = $state<{ startDate: string; endDate: string }[]>([]);
	let loading = $state(true);

	const cache = createRateLookupCache({
		lookup: lookupRate,
		fetch: fetchRates,
		canFetch: () => canEditRates(authState.currentUser?.role),
		today: () => todayLocal()
	});

	onMount(async () => {
		const [a, c, y] = await Promise.allSettled([
			fetchAccounts(false),
			fetchCompanyCurrent(),
			listFiscalYears()
		]);
		if (a.status === 'fulfilled') accounts = a.value;
		else accountsLoadError = true;
		company = c.status === 'fulfilled' ? c.value.company : null;
		openFiscalYears =
			y.status === 'fulfilled'
				? y.value
						.filter((fy) => fy.status === 'Open')
						.map((fy) => ({ startDate: fy.startDate, endDate: fy.endDate }))
				: [];
		loading = false;
	});
</script>

<svelte:head>
	<title>{i18nMsg('journal-entries-conversion-title', 'Conversion de devises')} — Kesh</title>
</svelte:head>

<div class="space-y-6">
	<div class="flex items-center gap-2">
		<Button variant="ghost" onclick={() => goto('/journal-entries')}>
			<ArrowLeft class="h-4 w-4" aria-hidden="true" />
			{i18nMsg('journal-entries-conversion-back', 'Écritures')}
		</Button>
		<h1 class="text-2xl font-semibold">
			{i18nMsg('journal-entries-conversion-title', 'Conversion de devises')}
		</h1>
	</div>
	<p class="text-sm text-text-muted">
		{i18nMsg(
			'journal-entries-conversion-intro',
			'Une conversion entre deux devises passe par votre compte de conversion, en deux écritures : chacune se rapproche du relevé de son compte. La différence de change équilibre la seconde ; le compte de conversion revient à zéro.'
		)}
	</p>
	{#if loading}
		<p class="text-text-muted" role="status">…</p>
	{:else}
		<ConversionAssistant
			{accounts}
			{accountsLoadError}
			{company}
			{openFiscalYears}
			{cache}
			onDone={() => goto('/journal-entries')}
		/>
	{/if}
</div>
