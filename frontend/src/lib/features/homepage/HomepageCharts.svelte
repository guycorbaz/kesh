<script lang="ts">
	// Story 33-3b (#918) — les deux graphiques de l'accueil et leurs états. Un échec
	// se DIT, une seule fois : prétendre qu'il n'y a rien est le défaut même de #388.
	import { i18nMsg } from '$lib/shared/utils/i18n.svelte';
	import ExpensePieCard from './ExpensePieCard.svelte';
	import MonthlyBarsCard from './MonthlyBarsCard.svelte';
	import type { HomepageSummary } from './homepage-summary.api';

	interface Props {
		state: 'loading' | 'ready' | 'error';
		summary: HomepageSummary | null;
	}
	let { state, summary }: Props = $props();

	const messageClass = 'mt-6 rounded-lg border border-border bg-white p-6 text-sm text-text-muted shadow-sm';
</script>

{#if state === 'loading'}
	<p class={messageClass} aria-busy="true" data-testid="homepage-charts-loading">
		{i18nMsg('homepage-charts-loading', 'Chargement des graphiques…')}
	</p>
{:else if state === 'error' || summary === null}
	<p class={messageClass} data-testid="homepage-charts-unavailable">
		{i18nMsg('homepage-charts-unavailable', 'Graphiques indisponibles pour le moment.')}
	</p>
{:else if summary.state === 'noOpenFiscalYear'}
	<p class={messageClass} data-testid="homepage-charts-no-open-year">
		{i18nMsg('homepage-charts-no-open-year', "Aucun exercice n'est ouvert à la date du jour.")}
	</p>
{:else}
	<div class="mt-6 grid grid-cols-1 gap-6 lg:grid-cols-2">
		<MonthlyBarsCard {summary} />
		<ExpensePieCard {summary} />
	</div>
{/if}
