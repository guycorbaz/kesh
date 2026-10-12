<!--
  Story 8-1b — Liste paginée des imports précédents.
  data-testid sur chaque ligne pour Playwright (Story 7-5/KF-008).
-->
<script lang="ts">
	import { i18nMsg } from '$lib/shared/utils/i18n.svelte';
	import type { BankImportResponse } from './bank-import.types';

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
					<th class="text-left">{i18nMsg('bank-import-labels-list-col-imported-at', 'Date import')}</th>
					<th class="text-left">{i18nMsg('bank-import-labels-list-col-filename', 'Fichier')}</th>
					<th class="text-left">{i18nMsg('bank-import-labels-period', 'Période')}</th>
					<th class="text-right">{i18nMsg('bank-import-labels-list-col-transactions', 'Transactions')}</th>
				</tr>
			</thead>
			<tbody>
				{#each imports as imp (imp.id)}
					<tr data-testid="bank-import-list-row" data-import-id={imp.id}>
						<td>{imp.importedAt}</td>
						<td><a class="text-primary underline" data-testid="bank-import-list-link" href="/bank-import/{imp.id}">{imp.filename}</a></td>
						<td>{imp.periodFrom} → {imp.periodTo}</td>
						<td class="text-right">{imp.transactionCount}</td>
					</tr>
				{/each}
			</tbody>
		</table>
	{/if}
</section>
