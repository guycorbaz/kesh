<script lang="ts">
	/**
	 * Story 37-3b (AC-2 bis, R7) — l'assistant *Conversion* : deux colonnes (le
	 * compte débité au relevé, le compte crédité au relevé), cours proposés et
	 * « cours OU CHF » comme au formulaire, aperçu des deux écritures avec la
	 * ligne de différence calculée (non modifiable, visible avant l'envoi, D6),
	 * puis « Passer les écritures » (`POST /journal-entries/conversions`).
	 */
	import Big from 'big.js';
	import { toast } from 'svelte-sonner';
	import { Plus, X } from '@lucide/svelte';
	import { Button } from '$lib/components/ui/button';
	import { Input } from '$lib/components/ui/input';
	import { i18nMsg } from '$lib/shared/utils/i18n.svelte';
	import { isApiError } from '$lib/shared/utils/api-client';
	import { todayLocal } from '$lib/shared/utils/local-date';
	import { accountCurrency, type AccountResponse } from '$lib/features/accounts/accounts.types';
	import type { CompanyJson } from '$lib/features/settings/settings.types';
	import { formatSwissAmount } from './balance';
	import { createConversion } from './journal-entries.api';
	import AccountAutocomplete from './AccountAutocomplete.svelte';
	import {
		applyChfEntry,
		applyLookup,
		monthOf,
		newForeign,
		rateIndicator,
		syncDerivedChf,
		type LineForeign
	} from './fx-lines';
	import { conversionField, planConversion, type ConversionBlocker, type ConversionColumn } from './conversion';
	import { fxRefusalMessage, isFxRefusalCode, refusalEntry, refusalLine } from './fx-refusals';
	import type { RateLookupCache } from './rate-lookup';
	import { formatForeignAmount, formatRate } from './fx-display';
	import { formatSwissDate } from '$lib/features/reports/reports.api';

	interface Props {
		accounts: AccountResponse[];
		accountsLoadError?: boolean;
		company: Pick<CompanyJson, 'fxGainAccountId' | 'fxLossAccountId' | 'fxTransitAccountId'> | null;
		openFiscalYears: { startDate: string; endDate: string }[];
		cache: RateLookupCache;
		onDone: () => void;
	}

	let { accounts, accountsLoadError = false, company, openFiscalYears, cache, onDone }: Props = $props();
	// fx-filtre-comptes : exempté — l'assistant choisit ses comptes en devise par
	// devise (37-3b, AC-2 bis) : `excludeForeignCurrency={false}` ; les frais n'offrent
	// que le CHF et la devise de leur colonne.

	const today = todayLocal();
	let debitCol = $state<ConversionColumn>({ date: today, accountId: null, foreign: null, fees: [] });
	let creditCol = $state<ConversionColumn>({ date: today, accountId: null, foreign: null, fees: [] });
	let submitting = $state(false);
	/** Refus du serveur : sur une colonne (`details.entry`), ou au-dessus de l'aperçu. */
	let columnError = $state<{ debit: string | null; credit: string | null }>({ debit: null, credit: null });
	let feeErrors = $state<{ debit: Record<number, string>; credit: Record<number, string> }>({
		debit: {},
		credit: {}
	});
	let globalError = $state<string | null>(null);

	const foreignAccounts = $derived(accounts.filter((a) => accountCurrency(a) !== null));
	const fromCurrency = $derived(columnCurrency(debitCol));
	const toCurrency = $derived(columnCurrency(creditCol));

	function columnCurrency(col: ConversionColumn): string | null {
		const a = accounts.find((x) => x.id === col.accountId);
		return a ? accountCurrency(a) : null;
	}

	/** Comptes de la colonne : les comptes en devise, sauf ceux de la devise de l'autre colonne. */
	function columnOptions(other: string | null): AccountResponse[] {
		return foreignAccounts.filter((a) => other === null || accountCurrency(a) !== other);
	}

	/** Comptes des frais : CHF et la devise de la colonne (jamais une autre devise). */
	function feeOptions(currency: string | null): AccountResponse[] {
		return accounts.filter((a) => {
			const c = accountCurrency(a);
			return c === null || c === currency;
		});
	}

	function getCol(side: 'debit' | 'credit'): ConversionColumn {
		return side === 'debit' ? debitCol : creditCol;
	}
	function setCol(side: 'debit' | 'credit', col: ConversionColumn) {
		if (side === 'debit') debitCol = col;
		else creditCol = col;
		columnError = { ...columnError, [side]: null };
		globalError = null;
	}

	/** Le `lookup` (mémoïsé) du mois de la date de la colonne ; propose le cours. */
	function refreshRate(side: 'debit' | 'credit') {
		const col = getCol(side);
		const f = col.foreign;
		if (!f || !/^\d{4}-\d{2}-\d{2}$/.test(col.date)) return;
		const date = col.date;
		const currency = f.currency;
		void cache.get(currency, date).then((r) => {
			const now = getCol(side);
			if (!now.foreign || now.foreign.currency !== currency || now.date !== date) return;
			setCol(side, { ...now, foreign: applyLookup(now.foreign, monthOf(date), r.outcome) });
		});
	}

	function selectAccount(side: 'debit' | 'credit', id: number | null) {
		const col = getCol(side);
		const a = accounts.find((x) => x.id === id);
		const currency = a ? accountCurrency(a) : null;
		const keep = col.foreign && col.foreign.currency === currency;
		setCol(side, {
			...col,
			accountId: id,
			foreign: currency === null ? null : keep ? col.foreign : newForeign(currency, side === 'debit' ? 'credit' : 'debit')
		});
		refreshRate(side);
	}

	function setDate(side: 'debit' | 'credit', date: string) {
		setCol(side, { ...getCol(side), date });
		refreshRate(side);
	}

	function updateForeign(side: 'debit' | 'credit', patch: (f: LineForeign) => LineForeign) {
		const col = getCol(side);
		if (!col.foreign) return;
		setCol(side, { ...col, foreign: patch(col.foreign) });
	}

	function chfOf(side: 'debit' | 'credit'): string {
		const f = getCol(side).foreign;
		if (!f) return '';
		if (f.chfEntry !== null) return f.chfEntry;
		const line = syncDerivedChf({ accountId: 0, debit: '', credit: '', foreign: f });
		return line.debit !== '' ? line.debit : line.credit;
	}

	function indicator(f: LineForeign): string {
		const ind = rateIndicator(f);
		if (ind.kind === 'afc') return i18nMsg('journal-entries-fx-indicator-afc', 'AFC');
		if (ind.kind === 'saisi') {
			return ind.source === null
				? i18nMsg('journal-entries-fx-indicator-saisi', 'cours saisi')
				: i18nMsg('journal-entries-fx-indicator-saisi-source', 'cours saisi ({ $source })', { source: ind.source });
		}
		return i18nMsg('journal-entries-fx-indicator-manual', 'manuel');
	}

	function addFee(side: 'debit' | 'credit') {
		const col = getCol(side);
		setCol(side, { ...col, fees: [...col.fees, { accountId: null, amount: '' }] });
	}
	function removeFee(side: 'debit' | 'credit', k: number) {
		const col = getCol(side);
		setCol(side, { ...col, fees: col.fees.filter((_, i) => i !== k) });
	}
	function setFee(side: 'debit' | 'credit', k: number, patch: Partial<{ accountId: number | null; amount: string }>) {
		const col = getCol(side);
		setCol(side, { ...col, fees: col.fees.map((f, i) => (i === k ? { ...f, ...patch } : f)) });
		feeErrors = { ...feeErrors, [side]: {} };
	}

	const labels = $derived({
		first: i18nMsg('journal-entries-conversion-label-1', 'Conversion { $from } → { $to } (1/2)', {
			from: fromCurrency ?? '',
			to: toCurrency ?? ''
		}),
		second: i18nMsg('journal-entries-conversion-label-2', 'Conversion { $from } → { $to } (2/2)', {
			from: fromCurrency ?? '',
			to: toCurrency ?? ''
		})
	});

	const plan = $derived(planConversion({ accounts, company, openFiscalYears }, debitCol, creditCol, labels));

	function accountLabel(id: number): string {
		const a = accounts.find((x) => x.id === id);
		return a ? `${a.number} ${a.name}` : String(id);
	}

	function blockerMessage(b: ConversionBlocker): string {
		switch (b) {
			case 'incomplete':
				return i18nMsg(
					'journal-entries-conversion-blocked-incomplete',
					'Choisissez les deux comptes en devise et saisissez, pour chacun, le montant et le cours (ou le CHF).'
				);
			case 'date-outside-open-fiscal-year':
				return i18nMsg(
					'journal-entries-conversion-blocked-date',
					'Chaque date doit tomber dans un exercice ouvert.'
				);
			case 'same-currency':
				return i18nMsg(
					'journal-entries-conversion-blocked-same-currency',
					'Les deux comptes sont dans la même devise : ce n’est pas une conversion, passez une écriture ordinaire.'
				);
			case 'transit-missing':
				return i18nMsg(
					'journal-entries-conversion-blocked-transit',
					'Aucun compte de conversion n’est désigné : réglez-le dans Paramètres → Comptabilité.'
				);
			case 'pair-invalid':
				return plan.kind === 'blocked' && plan.pairOrigin === 'own'
					? i18nMsg(
							'journal-entries-conversion-blocked-pair-own',
							'Les comptes de différence de change propres du compte { $number } sont invalides (archivés, non imputables…) : corrigez-les dans le plan comptable.',
							{ number: plan.holderNumber ?? '' }
						)
					: i18nMsg(
							'journal-entries-conversion-blocked-pair-company',
							'Les comptes de différence de change de la société manquent ou sont invalides : réglez-les dans Paramètres → Comptabilité.'
						);
			case 'fee-second-foreign-account':
				return i18nMsg(
					'journal-entries-conversion-blocked-second-foreign',
					'Un frais de la colonne Crédit est sur un autre compte en devise qui n’est pas un compte de charge de la devise : avec une différence de change, ses comptes de différence seraient indéterminés. Passez ce frais en CHF ou sur un compte de charge de la devise.'
				);
			case 'transit-not-positive':
				return i18nMsg(
					'journal-entries-conversion-blocked-transit-amount',
					'Les frais de la colonne Débit dépassent le montant converti : le compte de conversion ne recevrait rien.'
				);
			case 'fee-invalid':
				return i18nMsg(
					'journal-entries-conversion-blocked-fee',
					'Une ligne de frais est incomplète : choisissez son compte et un montant positif.'
				);
		}
	}

	async function submit() {
		if (plan.kind !== 'ready' || submitting) return;
		const sentPlan = plan;
		submitting = true;
		columnError = { debit: null, credit: null };
		feeErrors = { debit: {}, credit: {} };
		globalError = null;
		try {
			await createConversion({ entries: sentPlan.entries });
			toast.success(i18nMsg('journal-entries-conversion-done', 'Conversion passée : deux écritures enregistrées.'));
			onDone();
		} catch (err) {
			if (!isApiError(err)) {
				globalError = i18nMsg('journal-entries-conversion-error', 'La conversion n’a pas pu être passée.');
				return;
			}
			const message = isFxRefusalCode(err.code) ? fxRefusalMessage(err.code, err.details, err.message) : err.message;
			const entry = refusalEntry(err.details);
			const line = refusalLine(err.details);
			if (entry === null) {
				globalError = message;
			} else {
				const side = entry === 1 ? 'debit' : 'credit';
				const feeCount = getCol(side).fees.filter((f) => f.accountId !== null).length;
				const field = line === null ? null : conversionField(line, feeCount);
				if (field?.kind === 'fee') feeErrors = { ...feeErrors, [side]: { [field.index]: message } };
				else if (field?.kind === 'difference') globalError = message;
				else columnError = { ...columnError, [side]: message };
			}
			toast.error(message);
		} finally {
			submitting = false;
		}
	}
</script>

{#snippet column(side: 'debit' | 'credit')}
	{@const col = getCol(side)}
	{@const other = side === 'debit' ? toCurrency : fromCurrency}
	<section class="space-y-3 rounded-md border border-border p-4" data-testid="conversion-column-{side}">
		<h2 class="font-semibold">
			{side === 'debit'
				? i18nMsg('journal-entries-conversion-debit', 'Débit')
				: i18nMsg('journal-entries-conversion-credit', 'Crédit')}
		</h2>
		<p class="text-xs text-text-muted">
			{side === 'debit'
				? i18nMsg('journal-entries-conversion-debit-help', 'Le compte débité sur son relevé (crédité au grand livre).')
				: i18nMsg('journal-entries-conversion-credit-help', 'Le compte crédité sur son relevé (débité au grand livre).')}
		</p>
		<label class="block text-sm">
			{i18nMsg('journal-entries-conversion-date', 'Date')}
			<Input
				type="date"
				value={col.date}
				data-testid="conversion-{side}-date"
				oninput={(e) => setDate(side, (e.currentTarget as HTMLInputElement).value)}
			/>
		</label>
		<div class="text-sm">
			<span class="mb-1 block">{i18nMsg('journal-entries-conversion-account', 'Compte')}</span>
			<AccountAutocomplete
				accounts={columnOptions(other)}
				value={col.accountId}
				loadError={accountsLoadError}
				excludeForeignCurrency={false}
				ariaLabel={i18nMsg('journal-entries-conversion-account', 'Compte')}
				onSelect={(id) => selectAccount(side, id)}
			/>
		</div>
		{#if col.foreign}
			{@const f = col.foreign}
			<label class="block text-sm">
				{i18nMsg('journal-entries-conversion-amount', 'Montant { $currency }', { currency: f.currency })}
				<Input
					type="text"
					inputmode="decimal"
					class="tabular-nums text-right"
					data-testid="conversion-{side}-amount"
					value={f.amount}
					oninput={(e) =>
						updateForeign(side, (x) => ({
							...x,
							amount: (e.currentTarget as HTMLInputElement).value,
							chfEntry: null,
							chfError: null
						}))}
				/>
			</label>
			<div class="flex items-end gap-2 text-sm">
				<label class="block flex-1">
					{i18nMsg('journal-entries-fx-rate', 'Cours')}
					<Input
						type="text"
						inputmode="decimal"
						class="tabular-nums text-right"
						data-testid="conversion-{side}-rate"
						value={f.rate}
						oninput={(e) =>
							updateForeign(side, (x) => ({
								...x,
								rate: (e.currentTarget as HTMLInputElement).value,
								rateProposed: false,
								touched: true,
								chfEntry: null,
								chfError: null
							}))}
					/>
				</label>
				<span class="pb-2 text-xs text-text-muted" data-testid="conversion-{side}-indicator">{indicator(f)}</span>
			</div>
			<label class="block text-sm">
				CHF
				<Input
					type="text"
					inputmode="decimal"
					class="tabular-nums text-right"
					data-testid="conversion-{side}-chf"
					value={chfOf(side)}
					oninput={(e) =>
						updateForeign(side, (x) => applyChfEntry(x, (e.currentTarget as HTMLInputElement).value, col.date))}
				/>
			</label>
		{/if}
		{#each col.fees as fee, k (k)}
			<div class="flex items-start gap-2" data-testid="conversion-{side}-fee-{k}">
				<div class="flex-1">
					<AccountAutocomplete
						accounts={feeOptions(col.foreign?.currency ?? null)}
						value={fee.accountId}
						loadError={accountsLoadError}
						excludeForeignCurrency={false}
						ariaLabel={i18nMsg('journal-entries-conversion-fee-account', 'Compte du frais')}
						onSelect={(id) => setFee(side, k, { accountId: id })}
					/>
				</div>
				<Input
					type="text"
					inputmode="decimal"
					class="w-28 tabular-nums text-right"
					aria-label={i18nMsg('journal-entries-conversion-fee-amount', 'Montant du frais')}
					value={fee.amount}
					oninput={(e) => setFee(side, k, { amount: (e.currentTarget as HTMLInputElement).value })}
				/>
				<button
					type="button"
					class="p-1 text-muted-foreground hover:text-destructive"
					aria-label={i18nMsg('journal-entries-conversion-fee-remove', 'Retirer ce frais')}
					onclick={() => removeFee(side, k)}
				>
					<X class="h-4 w-4" />
				</button>
			</div>
			{#if feeErrors[side][k]}
				<p class="text-xs text-destructive">{feeErrors[side][k]}</p>
			{/if}
		{/each}
		<Button type="button" size="sm" variant="outline" onclick={() => addFee(side)}>
			<Plus class="mr-1 h-4 w-4" />
			{i18nMsg('journal-entries-conversion-add-fee', 'Ajouter des frais')}
		</Button>
		{#if columnError[side]}
			<p class="text-sm text-destructive" data-testid="conversion-{side}-error">{columnError[side]}</p>
		{/if}
	</section>
{/snippet}

<div class="space-y-6" data-testid="conversion-assistant">
	<div class="grid grid-cols-1 gap-4 md:grid-cols-2">
		{@render column('debit')}
		{@render column('credit')}
	</div>

	<section class="space-y-3 rounded-md border border-border p-4" data-testid="conversion-preview">
		<h2 class="font-semibold">{i18nMsg('journal-entries-conversion-preview', 'Aperçu')}</h2>
		{#if globalError}
			<p class="text-sm text-destructive" data-testid="conversion-global-error">{globalError}</p>
		{/if}
		{#if plan.kind === 'ready'}
			{#each plan.preview as entryLines, e (e)}
				<div>
					<p class="text-sm font-medium">
						{e + 1}) {formatSwissDate(plan.entries[e].entryDate)} — {plan.entries[e].description}
					</p>
					<table class="w-full text-sm tabular-nums">
						<tbody>
							{#each entryLines as l, k (k)}
								<tr data-testid="conversion-preview-{e + 1}-{l.role}">
									<td class="py-1 pr-2">{accountLabel(l.accountId)}</td>
									<td class="py-1 pr-2 text-xs text-text-muted">
										{#if l.foreign}
											{l.foreign.currency} {formatForeignAmount(l.foreign.amount)} × {formatRate(l.foreign.rate)}
										{/if}
									</td>
									<td class="w-28 py-1 pr-2 text-right">{l.debit ? formatSwissAmount(new Big(l.debit)) : ''}</td>
									<td class="w-28 py-1 text-right">{l.credit ? formatSwissAmount(new Big(l.credit)) : ''}</td>
								</tr>
							{/each}
						</tbody>
					</table>
				</div>
			{/each}
			{#if plan.difference}
				<p class="text-xs text-text-muted" data-testid="conversion-difference-origin">
					{plan.pairOrigin === 'own'
						? i18nMsg(
								'journal-entries-conversion-difference-own',
								'Différence de change sur les comptes propres du compte { $number }.',
								{ number: plan.holderNumber }
							)
						: i18nMsg(
								'journal-entries-conversion-difference-company',
								'Différence de change sur les comptes de la société.'
							)}
				</p>
			{/if}
			<p class="text-sm" data-testid="conversion-transit-zero">
				{i18nMsg('journal-entries-conversion-transit-zero', 'Compte de conversion après les deux écritures : 0.00')}
			</p>
		{:else}
			<ul class="list-disc pl-5 text-sm text-destructive" data-testid="conversion-blockers">
				{#each plan.blockers as b (b)}
					<li data-testid="conversion-blocker-{b}">{blockerMessage(b)}</li>
				{/each}
			</ul>
		{/if}
		<div class="flex justify-end">
			<Button
				type="button"
				data-testid="conversion-submit"
				disabled={plan.kind !== 'ready' || submitting}
				onclick={submit}
			>
				{i18nMsg('journal-entries-conversion-submit', 'Passer les écritures')}
			</Button>
		</div>
	</section>
</div>
