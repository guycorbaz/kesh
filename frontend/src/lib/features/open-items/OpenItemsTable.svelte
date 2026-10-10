<script lang="ts">
	// Story 15-1c-i (AC2, AC3, AC4, AC8, AC10) — la liste des postes ouverts d'un
	// compte à une date, ses motifs, ses cases et son pied.
	//
	// ⛔ Tri : celui du serveur (le Grand livre), jamais retrié ici. La case suit
	// `manuallyLetterable` — jamais `document != null` : une ligne seulement
	// rapprochée d'une transaction bancaire en a une.
	import { i18nMsg } from '$lib/shared/utils/i18n.svelte';
	import { formatSwissDate } from '$lib/features/reports/reports.api';
	import LetteringCodeLink from './LetteringCodeLink.svelte';
	import DocumentCell from './DocumentCell.svelte';
	import { sameAmount } from './open-items';
	import {
		amountWithSideLabel,
		documentStateLabel,
		entryRefLabel,
		formatAmountString,
		letteredOnLabel,
		noCheckboxLabel,
		reasonLabel,
	} from './open-items-labels';
	import type { OpenItem, OpenItemsResponse } from './open-items.types';

	interface Props {
		data: OpenItemsResponse;
		/** La date locale du jour — la note « état d'aujourd'hui » n'apparaît que pour une date passée. */
		today: string;
		/** Comptable ou Admin : cases et motifs d'absence de case. */
		canWrite: boolean;
		isSelected: (lineId: number) => boolean;
		onToggle: (item: OpenItem) => void;
		onPage: (offset: number) => void;
	}
	let { data, today, canWrite, isSelected, onToggle, onPage }: Props = $props();

	/** Un montant nul ne s'écrit pas : une colonne débit/crédit reste vide. */
	function amountOrBlank(v: string): string {
		return Number(v) === 0 ? '' : formatAmountString(v);
	}

	let pastDate = $derived(data.asOf < today);
	let equal = $derived(sameAmount(data.openTotal, data.balance));
	let from = $derived(data.total === 0 ? 0 : data.offset + 1);
	let to = $derived(Math.min(data.offset + data.items.length, data.total));
</script>

<section class="space-y-2" data-testid="open-items-list">
	{#if pastDate}
		<p class="rounded bg-blue-50 px-3 py-2 text-xs text-blue-900" data-testid="open-items-today-note">
			{i18nMsg(
				'open-items-state-today-note',
				"L'état de la pièce (seconde ligne du motif) est celui d'aujourd'hui ; la première ligne dit le motif à la date choisie.",
			)}
		</p>
	{/if}

	<div class="overflow-x-auto">
		<table class="w-full border-collapse text-sm">
			<thead>
				<tr class="border-b text-left">
					{#if canWrite}
						<th class="px-2 py-1" scope="col">
							<span class="sr-only">{i18nMsg('open-items-col-select', 'Sélection')}</span>
						</th>
					{/if}
					<th class="px-2 py-1" scope="col">{i18nMsg('open-items-col-date', 'Date')}</th>
					<th class="px-2 py-1" scope="col">{i18nMsg('open-items-col-entry', 'Écriture')}</th>
					<th class="px-2 py-1" scope="col">{i18nMsg('open-items-col-journal', 'Journal')}</th>
					<th class="px-2 py-1" scope="col">
						{i18nMsg('open-items-col-description', 'Libellé')}
					</th>
					<th class="px-2 py-1" scope="col">{i18nMsg('open-items-col-document', 'Pièce')}</th>
					<th class="px-2 py-1 text-right" scope="col">
						{i18nMsg('open-items-col-debit', 'Débit')}
					</th>
					<th class="px-2 py-1 text-right" scope="col">
						{i18nMsg('open-items-col-credit', 'Crédit')}
					</th>
					<th class="px-2 py-1" scope="col">
						<span
							title={i18nMsg(
								'open-items-reason-help',
								"Pourquoi la ligne est ouverte à la date choisie, puis où en est sa pièce aujourd'hui.",
							)}>{i18nMsg('open-items-col-reason', 'Motif')}</span
						>
					</th>
				</tr>
			</thead>
			<tbody>
				{#each data.items as item (item.lineId)}
					{@const ref = entryRefLabel(item.fiscalYearName, item.entryNumber)}
					<tr class="border-b align-top" data-testid="open-item-row-{item.lineId}">
						{#if canWrite}
							<td class="px-2 py-1">
								{#if item.manuallyLetterable}
									<input
										type="checkbox"
										checked={isSelected(item.lineId)}
										onchange={() => onToggle(item)}
										aria-label={i18nMsg('open-items-select-line', 'Sélectionner la ligne { $entry }', {
											entry: ref,
										})}
										data-testid="open-item-select-{item.lineId}"
									/>
								{:else}
									{@const why = noCheckboxLabel(item)}
									<span
										class="text-gray-400"
										title={why}
										data-testid="open-item-no-select-{item.lineId}"
									>
										<span aria-hidden="true">—</span>
										<span class="sr-only">{why}</span>
									</span>
								{/if}
							</td>
						{/if}
						<td class="px-2 py-1 whitespace-nowrap">{formatSwissDate(item.date)}</td>
						<td class="px-2 py-1 whitespace-nowrap">
							<a href="/journal-entries/{item.entryId}" class="text-primary underline">{ref}</a>
						</td>
						<td class="px-2 py-1">{item.journal}</td>
						<td class="px-2 py-1">{item.description}</td>
						<td class="px-2 py-1"><DocumentCell document={item.document} /></td>
						<td class="px-2 py-1 text-right font-mono whitespace-nowrap">
							{amountOrBlank(item.debit)}
						</td>
						<td class="px-2 py-1 text-right font-mono whitespace-nowrap">
							{amountOrBlank(item.credit)}
						</td>
						<td class="px-2 py-1" data-testid="open-item-reason-{item.lineId}">
							<div>
								{reasonLabel(item.reason)}
								{#if item.reason === 'letteredAfterAsOf' && item.letteringCode}
									<LetteringCodeLink
										code={item.letteringCode}
										accountId={data.accountId}
										asOf={data.asOf}
									/>
									{#if item.letteredOn}
										{letteredOnLabel(formatSwissDate(item.letteredOn))}
									{/if}
								{/if}
							</div>
							{#if item.documentState}
								<div class="text-xs text-gray-600" data-testid="open-item-state-{item.lineId}">
									{documentStateLabel(item.documentState, item.amountDue)}
								</div>
							{/if}
						</td>
					</tr>
				{:else}
					<tr>
						<td class="px-2 py-2 text-sm italic text-gray-500" colspan={canWrite ? 9 : 8}>
							{i18nMsg('open-items-empty', 'Aucun poste ouvert à cette date.')}
						</td>
					</tr>
				{/each}
			</tbody>
		</table>
	</div>

	<div class="flex flex-wrap items-center justify-between gap-2 text-sm">
		<span data-testid="open-items-range">
			{i18nMsg('open-items-range', '{ $from }–{ $to } sur { $total }', {
				from,
				to,
				total: data.total,
			})}
		</span>
		<span class="flex gap-2">
			<button
				type="button"
				class="rounded border px-2 py-1 disabled:opacity-50"
				disabled={data.offset === 0}
				onclick={() => onPage(Math.max(0, data.offset - data.limit))}
				data-testid="open-items-prev">{i18nMsg('common-previous', 'Précédent')}</button
			>
			<button
				type="button"
				class="rounded border px-2 py-1 disabled:opacity-50"
				disabled={data.offset + data.items.length >= data.total}
				onclick={() => onPage(data.offset + data.limit)}
				data-testid="open-items-next">{i18nMsg('common-next', 'Suivant')}</button
			>
		</span>
	</div>

	<dl class="space-y-1 rounded border bg-gray-50 p-3 text-sm" data-testid="open-items-footer">
		<div class="flex flex-wrap justify-between gap-2">
			<dt>
				{i18nMsg('open-items-open-total', 'Total des postes ouverts au { $date }', {
					date: formatSwissDate(data.asOf),
				})}
			</dt>
			<dd class="font-mono font-semibold" data-testid="open-items-open-total">
				{amountWithSideLabel(data.openTotal)}
			</dd>
		</div>
		<div class="flex flex-wrap justify-between gap-2">
			<dt>
				{i18nMsg('open-items-balance', 'Solde du compte au { $date }', {
					date: formatSwissDate(data.asOf),
				})}
			</dt>
			<dd class="font-mono font-semibold" data-testid="open-items-balance">
				{amountWithSideLabel(data.balance)}
			</dd>
		</div>
		{#if equal}
			<p class="pt-1 text-xs text-gray-600" data-testid="open-items-balance-equal">
				{i18nMsg(
					'open-items-balance-equal',
					"Le total des postes ouverts au { $date } égale le solde du compte au { $date } — celui que la Balance montre pour ce compte à cette date, du côté naturel du compte (solde cumulé depuis l'ouverture des livres).",
					{ date: formatSwissDate(data.asOf) },
				)}
			</p>
		{/if}
	</dl>
</section>
