<script lang="ts">
	// Story 24-1 — Vue Grand livre : l'extrait d'un compte, ligne à ligne.
	//
	// ⚠️ Ce rapport ne prend PAS d'exercice — il franchit la borne, sans quoi
	// il ne concorderait pas avec le bilan (cumulatif depuis l'origine). Les
	// ruptures d'exercice sont matérialisées dans le tableau, à l'endroit où
	// le solde d'un compte de résultat repart de zéro.
	import { i18nMsg } from '$lib/shared/utils/i18n.svelte';
	import LetteringCodeLink from '$lib/features/open-items/LetteringCodeLink.svelte';
	import { openItemsHref } from '$lib/features/open-items/open-items';
	import { formatReportAmount, formatSwissDate } from './reports.api';
	import type { GeneralLedgerDto, LedgerLine, LedgerSection } from './reports.types';

	interface Props {
		dto: GeneralLedgerDto;
		/**
		 * Story 15-1c-ii (AC9, C-15-1c-8) — les comptes lettrables, lus par la
		 * page dans la liste des comptes qu'elle charge déjà (archivés compris).
		 * `kesh-report` ne voit pas la lettrabilité (C-15-1b-8) : sans cette
		 * liste — vide si elle n'a pas pu être chargée —, aucun lien « Postes
		 * ouverts de ce compte », jamais un lien vers un compte non lettrable
		 * (qui rendrait 409).
		 */
		letterableAccountIds?: ReadonlySet<number>;
	}
	let { dto, letterableAccountIds = new Set<number>() }: Props = $props();

	/**
	 * Story 15-1c-ii (AC9, C-15-1c-20) — les colonnes, **une seule fois** : les
	 * en-têtes s'en déduisent, et chaque `colspan` aussi. « Lettrage » est la
	 * dernière, si bien que « Débit » et « Crédit » ne bougent pas et que les
	 * totaux du pied restent dessous.
	 *
	 * ⚠️ Avant cette story, les `colspan` étaient écrits en dur (7, 8, 5) : une
	 * colonne de plus les aurait tous laissés faux, sans rien qui rougisse.
	 */
	const COLUMNS = [
		{ id: 'date', right: false },
		{ id: 'piece', right: false },
		{ id: 'journal', right: false },
		{ id: 'description', right: false },
		{ id: 'counterpart', right: false },
		{ id: 'debit', right: true },
		{ id: 'credit', right: true },
		{ id: 'running', right: true },
		{ id: 'lettering', right: false }
	] as const;
	type ColumnId = (typeof COLUMNS)[number]['id'];

	/** Le nombre de colonnes : la constante d'où se calcule tout `colspan`. */
	const COLUMN_COUNT = COLUMNS.length;
	const indexOf = (id: ColumnId) => COLUMNS.findIndex((c) => c.id === id);
	/** Libellé d'une ligne de solde : tout ce qui précède le solde progressif. */
	const BALANCE_LABEL_SPAN = indexOf('running');
	/** Cellules vides après le solde progressif (la colonne « Lettrage »). */
	const AFTER_BALANCE = COLUMN_COUNT - BALANCE_LABEL_SPAN - 1;
	/** Libellé du total des mouvements : tout ce qui précède « Débit ». */
	const MOVEMENTS_LABEL_SPAN = indexOf('debit');
	/** Cellules vides après « Crédit » dans le total des mouvements. */
	const AFTER_MOVEMENTS = COLUMN_COUNT - indexOf('credit') - 1;

	/** Libellé d'un en-tête — un appel littéral par clé, que lisent les gardes i18n. */
	function columnLabel(id: ColumnId): string {
		switch (id) {
			case 'date':
				return i18nMsg('reports-column-entry-date', 'Date');
			case 'piece':
				return i18nMsg('reports-ledger-column-piece', 'Pièce');
			case 'journal':
				return i18nMsg('reports-ledger-column-journal', 'Journal');
			case 'description':
				return i18nMsg('reports-column-description', 'Libellé');
			case 'counterpart':
				return i18nMsg('reports-ledger-column-counterpart', 'Contrepartie');
			case 'debit':
				return i18nMsg('reports-column-debit', 'Débit');
			case 'credit':
				return i18nMsg('reports-column-credit', 'Crédit');
			case 'running':
				return i18nMsg('reports-ledger-column-running', 'Solde progressif');
			case 'lettering':
				return i18nMsg('reports-ledger-column-lettering', 'Lettrage');
		}
	}

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
				{#if letterableAccountIds.has(s.accountId)}
					<!-- Story 15-1c-ii (AC9) — les postes ouverts au dernier jour affiché :
					     leur total égale la clôture de la section, au signe près (le Grand
					     livre la montre du côté naturel du compte, l'écran en valeur
					     absolue suivie de « débiteur » ou « créditeur »). -->
					<a
						class="ml-auto text-xs text-primary underline"
						href={openItemsHref(s.accountId, dto.period.to)}
						data-testid="ledger-open-items-{s.accountNumber}"
						>{i18nMsg('reports-ledger-open-items-link', 'Postes ouverts de ce compte')}</a
					>
				{/if}
			</header>

			<table class="w-full border-collapse text-sm">
				<thead>
					<tr class="border-b text-left">
						{#each COLUMNS as c (c.id)}
							<th class="px-2 py-1" class:text-right={c.right}>{columnLabel(c.id)}</th>
						{/each}
					</tr>
				</thead>
				<tbody>
					<tr class="border-b bg-gray-50/50 italic">
						<td class="px-2 py-1" colspan={BALANCE_LABEL_SPAN}>
							{i18nMsg('reports-ledger-opening', "Solde d'ouverture")}
						</td>
						<td class="px-2 py-1 text-right font-mono" data-testid="ledger-opening">
							{fmt(s.opening)}
						</td>
						{#each { length: AFTER_BALANCE }, i (i)}<td class="px-2 py-1"></td>{/each}
					</tr>

					{#each s.lines as l, idx (l.lineId)}
						{@const brk = breakBefore(s, idx)}
						{#if brk}
							<tr class="border-y bg-indigo-50 text-xs text-indigo-900">
								<td class="px-2 py-1" colspan={BALANCE_LABEL_SPAN}>
									{formatSwissDate(brk.date)} — {i18nMsg(
										'reports-ledger-fy-break',
										"Clôture de l'exercice — le solde repart de zéro",
									)}
								</td>
								<td class="px-2 py-1 text-right font-mono">{fmt(brk.closingBalance)}</td>
								{#each { length: AFTER_BALANCE }, i (i)}<td class="px-2 py-1"></td>{/each}
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
							<td class="px-2 py-1" data-testid="ledger-lettering-{l.lineId}">
								{#if l.letteringCode}<LetteringCodeLink code={l.letteringCode} />{/if}
							</td>
						</tr>
					{:else}
						<tr>
							<td class="px-2 py-2 text-sm italic text-gray-500" colspan={COLUMN_COUNT}>
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
						<td class="px-2 py-1" colspan={MOVEMENTS_LABEL_SPAN}>
							{i18nMsg('reports-ledger-movements-total', 'Total des mouvements')}
						</td>
						<td class="px-2 py-1 text-right font-mono">{fmt(s.totalDebit)}</td>
						<td class="px-2 py-1 text-right font-mono">{fmt(s.totalCredit)}</td>
						{#each { length: AFTER_MOVEMENTS }, i (i)}<td class="px-2 py-1"></td>{/each}
					</tr>
					<tr class="border-t font-semibold">
						<td class="px-2 py-1" colspan={BALANCE_LABEL_SPAN}>
							{i18nMsg('reports-ledger-closing', 'Solde de clôture')}
						</td>
						<td class="px-2 py-1 text-right font-mono" data-testid="ledger-closing">
							{fmt(s.closing)}
						</td>
						{#each { length: AFTER_BALANCE }, i (i)}<td class="px-2 py-1"></td>{/each}
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
