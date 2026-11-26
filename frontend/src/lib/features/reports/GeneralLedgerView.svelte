<script lang="ts">
	// Story 24-1 — Vue Grand livre : l'extrait d'un compte, ligne à ligne.
	//
	// ⚠️ Ce rapport ne prend PAS d'exercice (l'API ; à l'écran, l'exercice n'est qu'un raccourci pour la
	// période, Story 55-2) — il franchit la borne, sans quoi
	// il ne concorderait pas avec le bilan (cumulatif depuis l'origine). Les
	// ruptures d'exercice sont matérialisées dans le tableau, à l'endroit où
	// le solde d'un compte de résultat repart de zéro.
	import { i18nMsg } from '$lib/shared/utils/i18n.svelte';
	import LetteringCodeLink from '$lib/features/open-items/LetteringCodeLink.svelte';
	import { openItemsHref } from '$lib/features/open-items/open-items';
	import Big from 'big.js';
	import { formatRate } from '$lib/features/exchange-rates/exchange-rates-grid';
	import { formatCurrencyAmount, formatReportAmount, formatSwissDate } from './reports.api';
	import type {
		FiscalYearBreak,
		GeneralLedgerDto,
		LedgerLine,
		LedgerSection,
	} from './reports.types';
	import AccountLink from './AccountLink.svelte';
	import ReportPeriodHeading from './ReportPeriodHeading.svelte';
	import type { ExerciceBornes } from './reports-url';

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
		/** Story 55-1a (AC-17) — les exercices de la société, pour l'en-tête de la période. */
		fiscalYears?: readonly ExerciceBornes[];
		/**
		 * Story 55-2 (AC-4, D8, C-55-2-3) — numéro → identifiant de compte, construit par la page depuis la
		 * liste des comptes qu'elle charge déjà : les contreparties (numéros seuls dans le DTO) deviennent des
		 * liens. Un numéro absent (liste en échec ou pas encore arrivée) reste du texte.
		 */
		accountIdByNumber?: ReadonlyMap<string, number>;
	}
	let {
		dto,
		letterableAccountIds = new Set<number>(),
		fiscalYears = [],
		accountIdByNumber = new Map<string, number>(),
	}: Props = $props();

	// Story 55-2 (AC-5, D8) — le titre d'une section n'est un lien que si le rapport en rend plusieurs : un
	// lien vers le compte qu'on lit déjà n'apprend rien.
	const titleLinked = $derived(dto.sections.length > 1);

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
	 * a une — même règle que `LedgerSection::break_before` (kesh-report), qui
	 * sert le PDF et le CSV.
	 *
	 * Une rupture précède la première ligne de son nouvel exercice et en porte la
	 * date. Entre deux lignes, elle se reconnaît au changement d'exercice : c'est
	 * celle qui clôt l'exercice de la ligne précédente — jamais à la date seule.
	 * En tête de section, aucune ligne précédente ne dit quel exercice se clôt :
	 * c'est alors la rupture datée du jour de la ligne, qui clôt un autre
	 * exercice que le sien (#777 — l'ouverture appartient à l'exercice qui
	 * contient le début de la période, et la première ligne peut tomber dans le
	 * suivant).
	 */
	function breakBefore(section: LedgerSection, idx: number) {
		const cur = section.lines[idx];
		if (idx > 0) {
			const prev = section.lines[idx - 1];
			if (prev.fiscalYearId === cur.fiscalYearId) return null;
			return (
				section.fiscalYearBreaks.find((b) => b.closingFiscalYearId === prev.fiscalYearId) ?? null
			);
		}
		return (
			section.fiscalYearBreaks.find(
				(b) => b.date === cur.entryDate && b.closingFiscalYearId !== cur.fiscalYearId,
			) ?? null
		);
	}

	/**
	 * Les ruptures qu'aucune ligne ne suit : une fin d'exercice franchie après la
	 * dernière ligne de la période (#778). Elles s'affichent sous les lignes —
	 * toutes, quand la section n'en a aucune. Ce sont celles que `breakBefore`
	 * ne place devant aucune ligne : chaque rupture s'affiche une fois, aucune ne
	 * se perd (même règle que `LedgerSection::trailing_breaks`).
	 */
	function trailingBreaks(section: LedgerSection) {
		const placees = new Set(
			section.lines.map((_, idx) => breakBefore(section, idx)?.closingFiscalYearId),
		);
		return section.fiscalYearBreaks.filter((b) => !placees.has(b.closingFiscalYearId));
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

	/**
	 * Story 37-4 (AC-2) — la seconde ligne d'un mouvement d'un compte en devise :
	 * « USD 31.35 × 0.8808 · solde USD 1'233.57 », ou, pour une réévaluation,
	 * « Réévaluation au cours X · solde … », sans montant en devise. La devise est
	 * celle du compte (la section), jamais d'une ligne.
	 */
	function fxMovement(currency: string, l: LedgerLine): string {
		const rate = formatRate(l.exchangeRate);
		const balance = formatCurrencyAmount(currency, l.runningBalanceCurrency);
		if (l.rateSource === 'revaluation') {
			return i18nMsg(
				'reports-fx-revaluation-balance',
				'Réévaluation au cours { $rate } · solde { $balance }',
				{ rate, balance },
			);
		}
		const amount = Big(l.debitCurrency ?? '0').plus(l.creditCurrency ?? '0').toString();
		return i18nMsg('reports-fx-movement', '{ $amount } × { $rate } · solde { $balance }', {
			amount: formatCurrencyAmount(currency, amount),
			rate,
			balance,
		});
	}
</script>

{#snippet fxBalance(currency: string | undefined, v: string | undefined, testid: string)}
	<!-- Story 37-4 — le même solde en devise, compte en devise seulement. -->
	{#if currency}
		<span class="ml-2 font-mono text-xs text-gray-600" data-testid={testid}
			>{formatCurrencyAmount(currency, v)}</span
		>
	{/if}
{/snippet}

{#snippet fyBreak(brk: FiscalYearBreak, currency: string | undefined)}
	<tr class="border-y bg-indigo-50 text-xs text-indigo-900" data-testid="ledger-fy-break">
		<td class="px-2 py-1" colspan={BALANCE_LABEL_SPAN}>
			{formatSwissDate(brk.date)} — {i18nMsg(
				'reports-ledger-fy-break',
				"Clôture de l'exercice — le solde repart de zéro",
			)}
			{@render fxBalance(currency, brk.closingBalanceCurrency, 'ledger-fy-break-fx')}
		</td>
		<td class="px-2 py-1 text-right font-mono">{fmt(brk.closingBalance)}</td>
		{#each { length: AFTER_BALANCE }, i (i)}<td class="px-2 py-1"></td>{/each}
	</tr>
{/snippet}

<section class="space-y-6" data-testid="general-ledger">
	<!-- Story 55-1a (AC-17) — la période réelle de la réponse, et l'exercice dont elle est l'entier ou une partie. -->
	<header>
		<ReportPeriodHeading from={dto.period.from} to={dto.period.to} {fiscalYears} />
	</header>

	{#if dto.sections.length === 0}
		<p class="rounded bg-blue-50 p-4 text-blue-900" role="status">
			{i18nMsg('reports-ledger-empty', 'Aucun compte à afficher sur cette période.')}
		</p>
	{/if}

	{#each dto.sections as s (s.accountId)}
		<article class="rounded border" data-testid="ledger-section-{s.accountNumber}">
			<header class="flex flex-wrap items-baseline gap-2 border-b bg-gray-50 px-3 py-2">
				<span class="font-mono font-semibold" data-testid="ledger-title-{s.accountNumber}"
					><AccountLink
						accountId={s.accountId}
						number={s.accountNumber}
						from={dto.period.from}
						to={dto.period.to}
						linked={titleLinked}
					/></span
				>
				<span class="font-semibold">{s.accountName}</span>
				{#if s.currency}
					<span
						class="rounded bg-sky-100 px-1 font-mono text-xs text-sky-900"
						data-testid="ledger-currency-{s.accountNumber}">{s.currency}</span
					>
				{/if}
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
							{@render fxBalance(s.currency, s.openingCurrency, 'ledger-opening-fx')}
						</td>
						<td class="px-2 py-1 text-right font-mono" data-testid="ledger-opening">
							{fmt(s.opening)}
						</td>
						{#each { length: AFTER_BALANCE }, i (i)}<td class="px-2 py-1"></td>{/each}
					</tr>

					{#each s.lines as l, idx (l.lineId)}
						{@const brk = breakBefore(s, idx)}
						{#if brk}
							{@render fyBreak(brk, s.currency)}
						{/if}
						<tr class="border-b last:border-b-0">
							<td class="px-2 py-1 whitespace-nowrap">{formatSwissDate(l.entryDate)}</td>
							<td class="px-2 py-1 font-mono">{piece(s, l)}</td>
							<td class="px-2 py-1">{l.journal}</td>
							<td class="px-2 py-1">{l.description}</td>
							<!-- Story 55-2 (AC-4, D8) — chaque contrepartie ouvre son grand livre, même période ; « , » entre elles. -->
							<td class="px-2 py-1 font-mono text-xs" data-testid="ledger-counterpart-{l.lineId}"
								>{#each l.counterpart as num, i (i)}{#if i > 0}{', '}{/if}<AccountLink
										accountId={accountIdByNumber.get(num) ?? null}
										number={num}
										from={dto.period.from}
										to={dto.period.to}
									/>{/each}</td
							>
							<td class="px-2 py-1 text-right font-mono">{amountOrBlank(l.debit)}</td>
							<td class="px-2 py-1 text-right font-mono">{amountOrBlank(l.credit)}</td>
							<td class="px-2 py-1 text-right font-mono">{fmt(l.runningBalance)}</td>
							<td class="px-2 py-1" data-testid="ledger-lettering-{l.lineId}">
								{#if l.letteringCode}<LetteringCodeLink code={l.letteringCode} />{/if}
							</td>
						</tr>
						{#if s.currency}
							<!-- Story 37-4 (AC-2) — seconde ligne, en devise du compte. -->
							<tr class="border-b text-xs text-gray-600" data-testid="ledger-fx-{l.lineId}">
								<td class="px-2 pb-1" colspan={indexOf('description')}></td>
								<td class="px-2 pb-1 font-mono" colspan={COLUMN_COUNT - indexOf('description')}
									>{fxMovement(s.currency, l)}</td
								>
							</tr>
						{/if}
					{:else}
						<!-- #777/#778 — « l'ouverture reste due » est faux quand une rupture
						     finale la clôt dans la période : le message le dit alors. -->
						<tr data-testid="ledger-no-movement">
							<td class="px-2 py-2 text-sm italic text-gray-500" colspan={COLUMN_COUNT}>
								{#if trailingBreaks(s).length > 0}
									{i18nMsg(
										'reports-ledger-no-movement-closed',
										"Aucun mouvement sur la période. L'exercice de l'ouverture se clôt dans la période : le solde repart de zéro.",
									)}
								{:else}
									{i18nMsg(
										'reports-ledger-no-movement',
										"Aucun mouvement sur la période. Le solde d'ouverture reste dû.",
									)}
								{/if}
							</td>
						</tr>
					{/each}
					{#each trailingBreaks(s) as brk (brk.closingFiscalYearId)}
						{@render fyBreak(brk, s.currency)}
					{/each}
				</tbody>
				<tfoot>
					<tr class="border-t">
						<td class="px-2 py-1" colspan={MOVEMENTS_LABEL_SPAN}>
							{i18nMsg('reports-ledger-movements-total', 'Total des mouvements')}
							{#if s.currency}
								<!-- Story 37-4 — les seuls totaux en devise : ceux d'une section. -->
								<span class="ml-2 font-mono text-xs text-gray-600" data-testid="ledger-totals-fx"
									>{i18nMsg('reports-fx-totals', 'débit { $debit } · crédit { $credit }', {
										debit: formatCurrencyAmount(s.currency, s.totalDebitCurrency),
										credit: formatCurrencyAmount(s.currency, s.totalCreditCurrency),
									})}</span
								>
							{/if}
						</td>
						<td class="px-2 py-1 text-right font-mono">{fmt(s.totalDebit)}</td>
						<td class="px-2 py-1 text-right font-mono">{fmt(s.totalCredit)}</td>
						{#each { length: AFTER_MOVEMENTS }, i (i)}<td class="px-2 py-1"></td>{/each}
					</tr>
					<tr class="border-t font-semibold">
						<td class="px-2 py-1" colspan={BALANCE_LABEL_SPAN}>
							{i18nMsg('reports-ledger-closing', 'Solde de clôture')}
							{@render fxBalance(s.currency, s.closingCurrency, 'ledger-closing-fx')}
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
