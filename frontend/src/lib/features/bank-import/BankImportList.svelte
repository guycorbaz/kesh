<!--
  Story 8-1b — Liste paginée des imports précédents.
  data-testid sur chaque ligne pour Playwright (Story 7-5/KF-008).

  Story v016-2b (#816, AC-23) — un relevé saisi à la main s'y affiche « Saisie
  manuelle » (et son numéro) au lieu de son `filename` ; les en-têtes valent
  pour les deux origines (« Enregistré le », « Relevé »).
-->
<script lang="ts">
	import { i18nMsg } from '$lib/shared/utils/i18n.svelte';
	import type { BankImportResponse } from './bank-import.types';
	import { importDisplayName } from './bank-import-display';

	type Props = {
		imports: BankImportResponse[];
	};
	let { imports }: Props = $props();
</script>

<section data-testid="bank-import-list">
	{#if imports.length === 0}
		<p class="text-text-muted" data-testid="bank-import-list-empty">
			{i18nMsg('bank-import-labels-empty', 'Aucun import bancaire.')}
		</p>
	{:else}
		<table class="w-full table-auto text-sm">
			<thead>
				<tr>
					<th class="text-left">{i18nMsg('bank-import-labels-list-col-imported-at', 'Enregistré le')}</th>
					<th class="text-left">{i18nMsg('bank-import-labels-list-col-filename', 'Relevé')}</th>
					<th class="text-left">{i18nMsg('bank-import-labels-period', 'Période')}</th>
					<th class="text-right">{i18nMsg('bank-import-labels-list-col-transactions', 'Transactions')}</th>
				</tr>
			</thead>
			<tbody>
				{#each imports as imp (imp.id)}
					<tr data-testid="bank-import-list-row" data-import-id={imp.id} data-source-format={imp.sourceFormat}>
						<td>{imp.importedAt}</td>
						<td><a class="text-primary underline break-all" data-testid="bank-import-list-link" href="/bank-import/{imp.id}">{importDisplayName(imp)}</a></td>
						<td>{imp.periodFrom} → {imp.periodTo}</td>
						<td class="text-right">{imp.transactionCount}</td>
					</tr>
				{/each}
			</tbody>
		</table>
	{/if}
</section>
