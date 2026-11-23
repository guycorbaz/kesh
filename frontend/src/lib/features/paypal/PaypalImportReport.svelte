<!--
  Story 38-12b (AC-3) — le compte rendu d'un import PayPal, extrait de la page
  PayPal de la 38-11d (AC-3) pour servir aux deux sources : l'import du CSV
  (page) et le téléchargement par l'API (dialogue générique, 38-12b) — par
  devise, transactions insérées et doublons, puis lignes écartées.

  `testid` préfixe les identifiants de test (« paypal-import-report » par
  défaut, celui de la 38-11d) : la page et le dialogue peuvent afficher chacun
  le leur.
-->
<script lang="ts">
	import { i18nMsg } from '$lib/shared/utils/i18n.svelte';
	import type { PaypalImportResponse } from './paypal.types';

	interface Props {
		report: PaypalImportResponse;
		testid?: string;
	}

	let { report, testid = 'paypal-import-report' }: Props = $props();
</script>

<section class="mt-4 rounded border border-border bg-surface-alt p-3 text-sm" data-testid={testid}>
	<h2 class="font-semibold">{i18nMsg('paypal-import-report-title', 'Compte rendu de l’import')}</h2>
	<ul class="mt-2 list-disc pl-5">
		{#each report.currencies as c (c.currency)}
			<li data-testid="{testid}-{c.currency}">
				{i18nMsg('paypal-import-report-currency', '{ $currency } : { $inserted } transaction(s) insérée(s), { $duplicates } doublon(s)', {
					currency: c.currency,
					inserted: String(c.inserted),
					duplicates: String(c.duplicates),
				})}
			</li>
		{/each}
	</ul>
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
</section>
