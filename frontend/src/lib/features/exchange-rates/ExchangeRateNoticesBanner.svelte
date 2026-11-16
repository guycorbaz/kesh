<!--
  Story 37-2 (AC-6, U1) — bandeau des corrections de l'AFC, par société.

  Sur le modèle de `BackupScheduleBanner.svelte` : monté dans
  `routes/(app)/+layout.svelte`, lu au montage (donc après chaque connexion) puis
  toutes les 30 minutes ; `role="status"`, jamais `alert`. Pour l'Administrateur
  et le Comptable d'une société multi-devises seulement, tant que
  `GET /exchange-rates/notices` rend une liste non vide : les cours que la société
  a retenus et que l'AFC a corrigés depuis (devise, mois ou jour, cours retenu →
  valeur corrigée), un lien vers l'écran *Cours de change* (exercice qui couvre le
  premier cours de la liste) et « Acquitter », qui envoie la liste affichée puis
  relit. L'acquittement vaut pour la société et pour cette valeur : une correction
  suivante avertit de nouveau. Un échec de lecture n'affiche aucun bandeau.
-->
<script lang="ts">
	import { authState } from '$lib/app/stores/auth.svelte';
	import { i18nLocale, i18nMsg } from '$lib/shared/utils/i18n.svelte';
	import { fetchCompanyCurrent } from '$lib/features/settings/settings.api';
	import { acknowledgeNotices, getNotices } from './exchange-rates.api';
	import { canEditRates } from './exchange-rates-access';
	import { formatRate, unitLabel } from './exchange-rates-grid';
	import type { ExchangeRateNotice } from './exchange-rates.types';

	const REFRESH_MS = 30 * 60_000;

	let allowed = $derived(canEditRates(authState.currentUser?.role));
	let notices = $state<ExchangeRateNotice[]>([]);
	let acknowledging = $state(false);
	let error = $state('');
	/** Numéro de la dernière lecture : une réponse tardive est ignorée. */
	let seq = 0;

	async function refresh() {
		const mine = ++seq;
		try {
			const company = await fetchCompanyCurrent();
			if (mine !== seq) return;
			if (!company.company.multiCurrencyEnabled) {
				notices = [];
				return;
			}
			const r = await getNotices({ retry: false });
			if (mine === seq) notices = r.notices;
		} catch {
			if (mine === seq) notices = [];
		}
	}

	$effect(() => {
		if (!allowed) {
			notices = [];
			return;
		}
		void refresh();
		const timer = setInterval(() => void refresh(), REFRESH_MS);
		return () => clearInterval(timer);
	});

	async function acknowledge() {
		acknowledging = true;
		error = '';
		try {
			await acknowledgeNotices(
				notices.map((n) => ({
					currency: n.currency,
					kind: n.kind,
					rateDate: n.rateDate,
					publishedRate: n.publishedRate,
				})),
			);
			await refresh();
		} catch {
			error = i18nMsg(
				'exchange-rates-banner-acknowledge-failed',
				'L’acquittement a échoué : réessayez.',
			);
		} finally {
			acknowledging = false;
		}
	}

	let link = $derived(
		notices.length > 0 ? `/settings/exchange-rates?date=${notices[0].rateDate}` : '/settings/exchange-rates',
	);
</script>

{#if allowed && notices.length > 0}
	<div
		role="status"
		class="flex flex-col gap-2 border-b border-warning bg-warning/10 px-4 py-2 text-sm text-text"
		data-testid="exchange-rate-notices-banner"
	>
		<div class="flex flex-wrap items-center justify-between gap-3">
			<span>
				{i18nMsg(
					'exchange-rates-banner-title',
					'L’AFC a corrigé { $count } cours que votre société a retenus.',
					{ count: notices.length },
				)}
			</span>
			<span class="flex items-center gap-3">
				<a href={link} class="font-medium underline" data-testid="exchange-rate-notices-banner-link">
					{i18nMsg('exchange-rates-banner-link', 'Voir les cours de change')}
				</a>
				<button
					type="button"
					class="rounded border border-border bg-white px-2 py-0.5 font-medium disabled:opacity-50"
					disabled={acknowledging}
					onclick={() => void acknowledge()}
					data-testid="exchange-rate-notices-banner-acknowledge"
				>
					{i18nMsg('exchange-rates-banner-acknowledge', 'Acquitter')}
				</button>
			</span>
		</div>
		<ul class="list-inside list-disc" data-testid="exchange-rate-notices-banner-list">
			{#each notices as n (n.currency + n.kind + n.rateDate)}
				<li data-testid="exchange-rate-notice-{n.currency}-{n.kind}-{n.rateDate}">
					{i18nMsg('exchange-rates-banner-item', '{ $currency }, { $unit } : { $retained } → { $corrected }', {
						currency: n.currency,
						unit: unitLabel(n.kind, n.rateDate, i18nLocale()),
						retained: formatRate(n.effectiveRate),
						corrected: formatRate(n.publishedRate),
					})}
				</li>
			{/each}
		</ul>
		{#if error}
			<span class="text-error" data-testid="exchange-rate-notices-banner-error">{error}</span>
		{/if}
	</div>
{/if}
