<!--
  Story 40-8b — le compte rendu d'un import Stripe, extrait de la page Stripe
  de la 40-7d (fragment `importReportExtra`) pour servir aux deux sources :
  l'import du rapport CSV (page commune des prestataires, fragment
  `importReport`) et le téléchargement par l'API (dialogue générique de la
  38-12b), sur le patron de `PaypalImportReport`. Les lignes par devise sont
  celles du compte rendu commun (`ProviderImportReport`) ; ce composant y ajoute
  les versements lus et, pour un téléchargement, les versements manuels de la
  période, non lus (« à examiner »).

  `testid` préfixe les identifiants de test (« stripe-import-report » par
  défaut, celui de la 40-7d).
-->
<script lang="ts">
	import { i18nMsg } from '$lib/shared/utils/i18n.svelte';
	import ProviderImportReport from '$lib/features/provider/ProviderImportReport.svelte';
	import type { StripeImportResponse } from './stripe.types';

	interface Props {
		report: StripeImportResponse & { manualPayouts?: string[] };
		testid?: string;
	}

	let { report, testid = 'stripe-import-report' }: Props = $props();
</script>

<ProviderImportReport {report} {testid}>
	{#if report.payouts.length > 0}
		<p class="mt-2">{i18nMsg('stripe-import-report-payouts', 'Versements lus :')}</p>
		<ul class="mt-1 list-disc pl-5" data-testid="{testid}-payouts">
			{#each report.payouts as po (po.payoutId)}
				<li data-testid="{testid}-payout-{po.payoutId}">
					{i18nMsg('stripe-import-report-payout', '{ $payout } : { $lines } ligne(s), net { $net } { $currency }', {
						payout: po.payoutId,
						lines: String(po.lines),
						net: po.net,
						currency: po.currency,
					})}
				</li>
			{/each}
		</ul>
	{/if}
	{#if report.manualPayouts && report.manualPayouts.length > 0}
		<p class="mt-2" data-testid="{testid}-manual">
			{i18nMsg(
				'stripe-download-report-manual',
				'Versements manuels à examiner, non téléchargés : { $payouts }',
				{ payouts: report.manualPayouts.join(', ') },
			)}
		</p>
	{/if}
</ProviderImportReport>
