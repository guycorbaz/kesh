<!--
  Story 38-12b (AC-3) — le compte rendu d'un import PayPal, extrait de la page
  PayPal de la 38-11d (AC-3) pour servir aux deux sources : l'import du CSV
  (page commune des prestataires, `ProviderReviewPage`, par son fragment
  `importReport`) et le téléchargement par l'API (dialogue générique, 38-12b).
  Les lignes par devise sont celles du compte rendu commun
  (`ProviderImportReport`, 40-7d) ; ce composant y ajoute les lignes écartées,
  propres à PayPal.

  `testid` préfixe les identifiants de test (« paypal-import-report » par
  défaut, celui de la 38-11d) : la page et le dialogue peuvent afficher chacun
  le leur.
-->
<script lang="ts">
	import { i18nMsg } from '$lib/shared/utils/i18n.svelte';
	import ProviderImportReport from '$lib/features/provider/ProviderImportReport.svelte';
	import type { PaypalImportResponse } from './paypal.types';

	interface Props {
		report: PaypalImportResponse;
		testid?: string;
	}

	let { report, testid = 'paypal-import-report' }: Props = $props();
</script>

<ProviderImportReport {report} {testid}>
	<p class="mt-2" data-testid="{testid}-skipped">
		{i18nMsg('paypal-import-report-skipped', 'Écartées : { $notCompleted } ligne(s) non terminée(s), { $memo } ligne(s) Memo, { $pairs } paire(s) neutre(s).', {
			notCompleted: String(report.skipped.notCompleted.length),
			memo: String(report.skipped.memo),
			pairs: String(report.skipped.neutralPairs),
		})}
	</p>
	{#if report.skipped.notCompleted.length > 0}
		<ul class="mt-1 list-disc pl-5 text-xs text-text-muted" data-testid="{testid}-not-completed">
			{#each report.skipped.notCompleted as l (l.transactionId)}
				<li>{l.transactionId} — {l.type} — {l.status}</li>
			{/each}
		</ul>
	{/if}
</ProviderImportReport>
