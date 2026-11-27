<script lang="ts">
	// Story 11-2 — Vue Rapport TVA (TVA due / vente).
	import { i18nMsg } from '$lib/shared/utils/i18n.svelte';
	import { formatReportAmount, isReportEmpty } from './reports.api';
	import AccountLink from './AccountLink.svelte';
	import ReportPeriodHeading from './ReportPeriodHeading.svelte';
	import type { ExerciceBornes } from './reports-url';
	import type { AccountRef, VatReportDto } from './reports.types';

	interface Props {
		dto: VatReportDto;
		/** Story 55-1a (AC-17) — les exercices de la société, pour l'en-tête de la période. */
		fiscalYears?: readonly ExerciceBornes[];
		/**
		 * Story 55-2 (AC-6, D7) — les comptes désignés dans les réglages de facturation (TVA due, impôt
		 * préalable), résolus par la page ; `null` : non désigné, absent de la liste des comptes, ou réglages en
		 * échec — sa mention est omise. Aucun des deux : la ligne est omise.
		 */
		vatAccounts?: { payable: AccountRef | null; recoverable: AccountRef | null };
	}
	let { dto, fiscalYears = [], vatAccounts = { payable: null, recoverable: null } }: Props = $props();

	// L'étiquette de chaque compte reprend celle de la colonne ou de la ligne du décompte ; elle ne promet
	// aucune égalité de montant (C-55-2-2 : la TVA due vient des factures, non du solde du compte).
	const vatAccountPairs = $derived(
		[
			{ ref: vatAccounts.payable, label: i18nMsg('reports-vat-column-vat-due', 'TVA due') },
			{ ref: vatAccounts.recoverable, label: i18nMsg('reports-vat-recoverable', 'TVA récupérable') },
		].filter((p): p is { ref: AccountRef; label: string } => p.ref !== null),
	);
	let empty = $derived(isReportEmpty('vat', dto));

	const fmt = formatReportAmount;

	/** Affiche un taux décimal (ex. "8.10") avec le symbole pourcent. */
	function fmtRate(rate: string): string {
		return `${rate} %`;
	}
</script>

<section class="space-y-4">
	<!-- Story 55-1a (AC-17) — la période réelle de la réponse, et l'exercice dont elle est l'entier ou une partie. -->
	<header>
		<ReportPeriodHeading from={dto.period.startDate} to={dto.period.endDate} {fiscalYears} />
	</header>

	<!-- Story 18-1e : bandeau de réconciliation (INFO non bloquant). Indépendant du
	     flag `empty` ; visible uniquement si la TVA due dérivée ne correspond pas au
	     solde du compte TVA due au grand livre (écriture validée modifiée à la main). -->
	{#if dto.reconciliationStatus === 'delta'}
		<p class="rounded bg-amber-50 p-3 text-sm text-amber-900" role="alert">
			{i18nMsg(
				'reports-vat-reconciliation-warning',
				'Le décompte ne correspond pas aux écritures comptables (écart : { $delta }). Vérifiez les écritures validées modifiées manuellement.',
				{ delta: fmt(dto.reconciliationDelta) }
			)}
		</p>
	{/if}

	{#if empty}
		<p class="rounded bg-blue-50 p-4 text-blue-900" role="status">
			{i18nMsg('reports-error-no-entries-in-period', 'Aucune écriture sur cette période.')}
		</p>
	{:else}
		<table class="mt-2 w-full border-collapse">
			<thead>
				<tr class="border-b bg-gray-50 text-left text-sm">
					<th class="px-2 py-1">{i18nMsg('reports-vat-column-rate', 'Taux')}</th>
					<th class="px-2 py-1 text-right"
						>{i18nMsg('reports-vat-column-base-ht', "Chiffre d'affaires HT")}</th
					>
					<th class="px-2 py-1 text-right">{i18nMsg('reports-vat-column-vat-due', 'TVA due')}</th>
				</tr>
			</thead>
			<tbody>
				{#each dto.rows as row (row.rate)}
					<tr>
						<td class="px-2 py-1 font-mono">{fmtRate(row.rate)}</td>
						<td class="px-2 py-1 text-right font-mono">{fmt(row.baseHt)}</td>
						<td class="px-2 py-1 text-right font-mono">{fmt(row.vatDue)}</td>
					</tr>
				{/each}
			</tbody>
			<tfoot>
				<tr class="border-t font-semibold">
					<td class="px-2 py-1">{i18nMsg('reports-vat-total-base-ht', 'Total CA HT')}</td>
					<td class="px-2 py-1 text-right font-mono">{fmt(dto.totalBaseHt)}</td>
					<td class="px-2 py-1 text-right font-mono" data-testid="vat-total-vat-due">
						{fmt(dto.totalVatDue)}
					</td>
				</tr>
				<!-- Story 25-4-d2c : les diminutions de contre-prestation (soldes) —
				     seulement s'il y en a dans la période. -->
				{#if dto.writeOffRows.length > 0}
					<tr class="border-t" data-testid="vat-write-off-section">
						<td colspan="3" class="px-2 py-1 font-semibold">
							{i18nMsg(
								'reports-vat-write-off-title',
								'Diminutions de contre-prestation (soldes)',
							)}
						</td>
					</tr>
					{#each dto.writeOffRows as row (row.rate)}
						<tr data-testid="vat-write-off-row">
							<td class="px-2 py-1 font-mono">{fmtRate(row.rate)}</td>
							<td class="px-2 py-1 text-right font-mono">{fmt(row.baseHt)}</td>
							<td class="px-2 py-1 text-right font-mono">{fmt(row.vat)}</td>
						</tr>
					{/each}
					<tr>
						<td colspan="2" class="px-2 py-1"
							>{i18nMsg('reports-vat-total-write-off', 'Total TVA des soldes')}</td
						>
						<td class="px-2 py-1 text-right font-mono">{fmt(dto.totalVatWriteOff)}</td>
					</tr>
					<tr class="font-semibold">
						<td colspan="2" class="px-2 py-1"
							>{i18nMsg('reports-vat-due-net', 'TVA due nette')}</td
						>
						<td class="px-2 py-1 text-right font-mono" data-testid="vat-total-vat-due-net">
							{fmt(dto.totalVatDueNet)}
						</td>
					</tr>
				{/if}
				<tr>
					<td colspan="2" class="px-2 py-1"
						>{i18nMsg('reports-vat-recoverable', 'TVA récupérable')}</td
					>
					<td class="px-2 py-1 text-right font-mono">{fmt(dto.totalVatRecoverable)}</td>
				</tr>
				<tr class="border-t font-semibold">
					<td colspan="2" class="px-2 py-1">{i18nMsg('reports-vat-balance', 'Solde')}</td>
					<td class="px-2 py-1 text-right font-mono" data-testid="vat-balance">
						{fmt(dto.vatBalance)}
					</td>
				</tr>
			</tfoot>
		</table>
	{/if}

	<!-- Story 55-2 (AC-6, D7) — les comptes de TVA au grand livre, après le contenu du décompte (tableau, ou
	     message d'un décompte vide), sur la période du décompte. -->
	{#if vatAccountPairs.length > 0}
		<p class="flex flex-wrap items-baseline gap-x-2 text-sm text-gray-700" data-testid="vat-accounts">
			<span>{i18nMsg('reports-nav-vat-accounts', 'Comptes au grand livre :')}</span>
			{#each vatAccountPairs as p, i (p.label)}
				{#if i > 0}{' '}<span aria-hidden="true">·</span>{/if}{' '}<span
					>{p.label}
					<span class="font-mono"
						><AccountLink
							accountId={p.ref.id}
							number={p.ref.number}
							from={dto.period.startDate}
							to={dto.period.endDate}
						/></span
					></span
				>
			{/each}
		</p>
	{/if}
</section>
