<script lang="ts">
	// Story 24-1 — Vue Grand livre : l'extrait d'un compte, ligne à ligne.
	//
	// ⚠️ Ce rapport ne prend PAS d'exercice — il franchit la borne, sans quoi
	// il ne concorderait pas avec le bilan (cumulatif depuis l'origine). Les
	// ruptures d'exercice sont matérialisées dans le tableau, à l'endroit où
	// le solde d'un compte de résultat repart de zéro.
	import { i18nMsg } from '$lib/shared/utils/i18n.svelte';
	import { formatReportAmount, formatSwissDate } from './reports.api';
	import type { GeneralLedgerDto, LedgerLine, LedgerSection } from './reports.types';

	interface Props {
		dto: GeneralLedgerDto;
	}
	let { dto }: Props = $props();

	const fmt = formatReportAmount;

	/**
	 * La rupture d'exercice à afficher AVANT la ligne d'indice `idx`, s'il y en
	 * a une. Elle se reconnaît au changement d'exercice entre deux lignes
	 * consécutives — jamais à la date seule : plusieurs écritures peuvent
	 * partager la date de bouclement.
	 */
	function breakBefore(section: LedgerSection, idx: number) {
		if (idx === 0) return null;
		const prev = section.lines[idx - 1];
		const cur = section.lines[idx];
		if (prev.fiscalYearId === cur.fiscalYearId) return null;
		return section.fiscalYearBreaks.find((b) => b.closingFiscalYearId === prev.fiscalYearId) ?? null;
	}

	/** Un montant nul ne s'écrit pas : une colonne débit/crédit reste vide. */
	function amountOrBlank(v: string): string {
		return Number(v) === 0 ? '' : fmt(v);
	}

	/**
	 * Le numéro de pièce, préfixé de son exercice dès que la période en traverse
	 * plusieurs.
	 *
	 * ⚠️ Sans ce préfixe, « pièce n° 12 » ne désigne rien : le numéro repart à 1
	 * à chaque exercice, et un extrait qui en couvre deux en contient alors deux.
	 */
	function piece(s: LedgerSection, l: LedgerLine): string {
		return s.fiscalYearBreaks.length === 0
			? String(l.entryNumber)
			: `${l.fiscalYearName}/${l.entryNumber}`;
	}
</script>

<section class="space-y-6" data-testid="general-ledger">
	<header class="text-sm text-gray-600">
		<strong>{i18nMsg('reports-filter-period', 'Période')}:</strong>
		{formatSwissDate(dto.period.from)} — {formatSwissDate(dto.period.to)}
	</header>

	{#if dto.sections.length === 0}
		<p class="rounded bg-blue-50 p-4 text-blue-900" role="status">
			{i18nMsg('reports-ledger-empty', 'Aucun compte à afficher sur cette période.')}
		</p>
	{/if}

	{#each dto.sections as s (s.accountId)}
		<article class="rounded border" data-testid="ledger-section-{s.accountNumber}">
			<header class="flex flex-wrap items-baseline gap-2 border-b bg-gray-50 px-3 py-2">
				<span class="font-mono font-semibold">{s.accountNumber}</span>
				<span class="font-semibold">{s.accountName}</span>
				{#if !s.active}
					<span class="rounded bg-gray-200 px-1 text-xs"
						>{i18nMsg('reports-ledger-archived', 'archivé')}</span
					>
				{/if}
				{#if s.unnaturalBalance}
					<span
						class="rounded bg-amber-100 px-2 text-xs text-amber-900"
						title={i18nMsg(
							'reports-ledger-unnatural-hint',
							'Ce compte présente un solde du côté opposé à sa nature. À vérifier.',
						)}
						data-testid="ledger-unnatural"
					>
						⚠️ {i18nMsg('reports-ledger-unnatural', 'Solde contre nature')}
					</span>
				{/if}
			</header>

			<table class="w-full border-collapse text-sm">
				<thead>
					<tr class="border-b text-left">
						<th class="px-2 py-1">{i18nMsg('reports-column-entry-date', 'Date')}</th>
						<th class="px-2 py-1">{i18nMsg('reports-ledger-column-piece', 'Pièce')}</th>
						<th class="px-2 py-1">{i18nMsg('reports-ledger-column-journal', 'Journal')}</th>
						<th class="px-2 py-1">{i18nMsg('reports-column-description', 'Libellé')}</th>
						<th class="px-2 py-1">
							{i18nMsg('reports-ledger-column-counterpart', 'Contrepartie')}
						</th>
						<th class="px-2 py-1 text-right">{i18nMsg('reports-column-debit', 'Débit')}</th>
						<th class="px-2 py-1 text-right">{i18nMsg('reports-column-credit', 'Crédit')}</th>
						<th class="px-2 py-1 text-right">
							{i18nMsg('reports-ledger-column-running', 'Solde progressif')}
						</th>
					</tr>
				</thead>
				<tbody>
					<tr class="border-b bg-gray-50/50 italic">
						<td class="px-2 py-1" colspan="7">
							{i18nMsg('reports-ledger-opening', "Solde d'ouverture")}
						</td>
						<td class="px-2 py-1 text-right font-mono" data-testid="ledger-opening">
							{fmt(s.opening)}
						</td>
					</tr>

					{#each s.lines as l, idx (l.lineId)}
						{@const brk = breakBefore(s, idx)}
						{#if brk}
							<tr class="border-y bg-indigo-50 text-xs text-indigo-900">
								<td class="px-2 py-1" colspan="7">
									{formatSwissDate(brk.date)} — {i18nMsg(
										'reports-ledger-fy-break',
										"Clôture de l'exercice — le solde repart de zéro",
									)}
								</td>
								<td class="px-2 py-1 text-right font-mono">{fmt(brk.closingBalance)}</td>
							</tr>
						{/if}
						<tr class="border-b last:border-b-0">
							<td class="px-2 py-1 whitespace-nowrap">{formatSwissDate(l.entryDate)}</td>
							<td class="px-2 py-1 font-mono">{piece(s, l)}</td>
							<td class="px-2 py-1">{l.journal}</td>
							<td class="px-2 py-1">{l.description}</td>
							<td class="px-2 py-1 font-mono text-xs">{l.counterpart.join(', ')}</td>
							<td class="px-2 py-1 text-right font-mono">{amountOrBlank(l.debit)}</td>
							<td class="px-2 py-1 text-right font-mono">{amountOrBlank(l.credit)}</td>
							<td class="px-2 py-1 text-right font-mono">{fmt(l.runningBalance)}</td>
						</tr>
					{:else}
						<tr>
							<td class="px-2 py-2 text-sm italic text-gray-500" colspan="8">
								{i18nMsg(
									'reports-ledger-no-movement',
									"Aucun mouvement sur la période. Le solde d'ouverture reste dû.",
								)}
							</td>
						</tr>
					{/each}
				</tbody>
				<tfoot>
					<tr class="border-t">
						<td class="px-2 py-1" colspan="5">
							{i18nMsg('reports-ledger-movements-total', 'Total des mouvements')}
						</td>
						<td class="px-2 py-1 text-right font-mono">{fmt(s.totalDebit)}</td>
						<td class="px-2 py-1 text-right font-mono">{fmt(s.totalCredit)}</td>
						<td class="px-2 py-1"></td>
					</tr>
					<tr class="border-t font-semibold">
						<td class="px-2 py-1" colspan="7">
							{i18nMsg('reports-ledger-closing', 'Solde de clôture')}
						</td>
						<td class="px-2 py-1 text-right font-mono" data-testid="ledger-closing">
							{fmt(s.closing)}
						</td>
					</tr>
				</tfoot>
			</table>

			{#if s.lines.length < s.lineCount}
				<p class="border-t bg-amber-50 px-3 py-2 text-xs text-amber-900" role="note">
					{i18nMsg(
						'reports-ledger-truncated',
						`Seules les ${s.lines.length} premières lignes sur ${s.lineCount} sont affichées. L'export les contient toutes.`,
						{ shown: s.lines.length, total: s.lineCount },
					)}
				</p>
			{/if}
		</article>
	{/each}
</section>
