<!--
  Fusion de l'Epic 38 (38-12b) dans l'Epic 40 (40-7d) — le compte rendu commun
  d'un import de prestataire : par devise, transactions insérées et doublons ;
  ce qui est propre au prestataire suit, par `children`. Il sert à la page
  commune (`ProviderReviewPage`, import du fichier) et aux comptes rendus d'un
  prestataire qui s'affichent aussi ailleurs (`PaypalImportReport`, dans le
  dialogue de téléchargement de la 38-12b).

  `testid` préfixe les identifiants de test (« paypal-import-report »,
  « stripe-import-report », « paypal-download-report »…).
-->
<script lang="ts">
	import type { Snippet } from 'svelte';
	import { i18nMsg } from '$lib/shared/utils/i18n.svelte';
	import type { ProviderImportResponseBase } from './provider-review.types';

	interface Props {
		report: ProviderImportResponseBase;
		testid: string;
		/** Ce qui est propre au prestataire, sous les lignes par devise. */
		children?: Snippet;
	}

	let { report, testid, children }: Props = $props();
</script>

<section class="mt-4 rounded border border-border bg-surface-alt p-3 text-sm" data-testid={testid}>
	<h2 class="font-semibold">{i18nMsg('provider-import-report-title', 'Compte rendu de l’import')}</h2>
	<ul class="mt-2 list-disc pl-5">
		{#each report.currencies as c (c.currency)}
			<li data-testid="{testid}-{c.currency}">
				{i18nMsg('provider-import-report-currency', '{ $currency } : { $inserted } transaction(s) insérée(s), { $duplicates } doublon(s)', {
					currency: c.currency,
					inserted: String(c.inserted),
					duplicates: String(c.duplicates),
				})}
			</li>
		{/each}
	</ul>
	{@render children?.()}
</section>
