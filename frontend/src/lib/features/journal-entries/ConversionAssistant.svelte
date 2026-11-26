<script lang="ts">
	/**
	 * Story 37-3b (AC-2 bis, R7) — l'assistant *Conversion* : deux colonnes (le
	 * compte débité au relevé, le compte crédité au relevé), cours proposés et
	 * « cours OU CHF » comme au formulaire, aperçu des deux écritures avec la
	 * ligne de différence calculée (non modifiable, visible avant l'envoi, D6),
	 * puis « Passer les écritures » (`POST /journal-entries/conversions`).
	 *
	 * Story 53-1b (#1365, table K de la fiche) — **une conversion ne passe pas deux
	 * fois** : l'envoi suit les états du module commun `entry-submit-state.ts`, dont les
	 * états K se rangent un à un : K1 = `J1` (saisie), K2a = `J2a` (attente des
	 * recherches de cours en vol, corps non formé), K2b = `J2b` (requête en vol, corps
	 * gardé), K3 = `J3` (fait), K5 = `J5` (doublon probable : avis, lien, confirmation),
	 * K6 = `J6` (issue inconnue : avis, lien de vérification, reprise — **rien n'est
	 * renvoyé tout seul**), K7 = `J7` (erreur ordinaire). Les contrôles sont figés
	 * (`<fieldset disabled>`) sauf en K1 et K7.
	 */
	import Big from 'big.js';
	import { toast } from 'svelte-sonner';
	import { Plus, X } from '@lucide/svelte';
	import { Button } from '$lib/components/ui/button';
	import { Input } from '$lib/components/ui/input';
	import { i18nMsg } from '$lib/shared/utils/i18n.svelte';
	import { isApiError } from '$lib/shared/utils/api-client';
	import { isSettlementOutcomeUnknown } from '$lib/shared/utils/unknown-outcome';
	import { todayLocal } from '$lib/shared/utils/local-date';
	import SupplierInvoiceNotice from '$lib/features/supplier-invoices/SupplierInvoiceNotice.svelte';
	import { accountCurrency, type AccountResponse } from '$lib/features/accounts/accounts.types';
	import type { CompanyJson } from '$lib/features/settings/settings.types';
	import { formatSwissAmount } from './balance';
	import { createConversion } from './journal-entries.api';
	import type { CreateConversionRequest } from './journal-entries.types';
	import AccountAutocomplete from './AccountAutocomplete.svelte';
	import {
		canConfirmFrom,
		canSendFrom,
		canWriteEntry,
		isEntryFrozen,
		readEntryDuplicateRefusal,
		stateAfterRejection,
		unknownOutcomeRangeHref,
		type EntryDuplicateRefusal,
		type EntrySubmitState
	} from './entry-submit-state';
	import {
		applyChfEntry,
		applyLookup,
		commitChfEntry,
		foreignLineProblem,
		monthOf,
		newForeign,
		rateIndicator,
		syncDerivedChf,
		type LineForeign
	} from './fx-lines';
	import { conversionField, planConversion, type ConversionBlocker, type ConversionColumn } from './conversion';
	import { foreignProblemMessage, fxRefusalMessage, isFxRefusalCode, refusalEntry, refusalLine } from './fx-refusals';
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
	/** L'état de l'envoi (table K, rangée dans les états J du module commun). */
	let phase = $state<EntrySubmitState>('J1');
	/** Le refus de doublon lu en K5 : l'écriture existante citée, et le rang visé. */
	let duplicate = $state<EntryDuplicateRefusal | null>(null);
	/**
	 * Le **corps gardé** de l'envoi en vol : c'est lui, et nul autre, que renvoie la
	 * confirmation de K5 (jamais reformé) ; ses dates alimentent le lien de K6.
	 */
	let kept = $state.raw<{ body: CreateConversionRequest; dates: [string, string] } | null>(null);
	/** Figé : tous les contrôles de la saisie (`<fieldset disabled>`), sauf en K1 et K7. */
	const frozen = $derived(isEntryFrozen(phase));
	/** « Passer les écritures » n'est offert ni en K5 ni en K6. */
	const showSubmit = $derived(phase !== 'J5' && phase !== 'J6');
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

	/** `lookup` en cours, que l'envoi attend (revue de code P1, 37-3b-CR-P1-6). */
	const inflight = new Set<Promise<void>>();

	/** Le `lookup` (mémoïsé) du mois de la date de la colonne ; propose le cours. */
	function refreshRate(side: 'debit' | 'credit') {
		const col = getCol(side);
		const f = col.foreign;
		if (!f || !/^\d{4}-\d{2}-\d{2}$/.test(col.date)) return;
		const date = col.date;
		const currency = f.currency;
		const done: Promise<void> = cache
			.get(currency, date)
			.then((r) => {
				// Une recherche résolue **après** la formation du corps (K2b et au-delà) ne
				// doit plus écrire la saisie : l'écran divergerait de ce qui a été envoyé.
				// En K2a (attente des recherches) elle écrit, c'est voulu : le plan est relu
				// ensuite, et le cours mis à jour part avec le corps. Défense en profondeur :
				// ce cas est inatteignable par l'interface (les contrôles sont figés).
				if (!canWriteEntry(phase)) return;
				const now = getCol(side);
				if (!now.foreign || now.foreign.currency !== currency || now.date !== date) return;
				setCol(side, { ...now, foreign: applyLookup(now.foreign, monthOf(date), r.outcome) });
			})
			.finally(() => inflight.delete(done));
		inflight.add(done);
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
		const next = patch(col.foreign);
		// Une sortie de champ qui ne change rien n'efface pas un refus affiché.
		if (next === col.foreign) return;
		setCol(side, { ...col, foreign: next });
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
		feeErrors = { ...feeErrors, [side]: {} };
	}
	function removeFee(side: 'debit' | 'credit', k: number) {
		const col = getCol(side);
		setCol(side, { ...col, fees: col.fees.filter((_, i) => i !== k) });
		feeErrors = { ...feeErrors, [side]: {} };
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
					'Une ligne de frais est incomplète ou invalide : choisissez un compte en CHF ou dans la devise de la colonne, et un montant positif (au centime pour un frais en CHF).'
				);
		}
	}

	async function submit() {
		// L'envoi ordinaire ne part que de K1 et K7 ; la garde est synchrone (K2a
		// dès le premier événement), un second événement d'envoi ne part pas.
		if (plan.kind !== 'ready' || !canSendFrom(phase) || submitting) return;
		phase = 'J2a';
		submitting = true;
		// K2a : l'envoi attend les `lookup` en cours (cours proposé du mois de la
		// date, provenance), comme le formulaire ; le plan est relu ensuite.
		if (inflight.size > 0) {
			await Promise.all([...inflight]);
			if (plan.kind !== 'ready') {
				phase = 'J1';
				submitting = false;
				return;
			}
		}
		const sentPlan = plan;
		columnError = { debit: null, credit: null };
		feeErrors = { debit: {}, credit: {} };
		globalError = null;
		// K2b : requête en vol, le corps est formé et **gardé**.
		const body: CreateConversionRequest = { entries: sentPlan.entries };
		kept = {
			body,
			dates: [sentPlan.entries[0].entryDate, sentPlan.entries[1].entryDate]
		};
		duplicate = null;
		phase = 'J2b';
		await transmit(body, false);
	}

	/**
	 * « Ce n'est pas un doublon, passer la conversion » (K5 → K2b) : renvoie le **corps
	 * gardé** avec `confirmNotDuplicate: true`, une seule fois — la garde est synchrone.
	 */
	async function confirmDuplicate() {
		if (!canConfirmFrom(phase) || kept === null || submitting) return;
		phase = 'J2b';
		submitting = true;
		await transmit(kept.body, true);
	}

	/** « Modifier la saisie » (K5 → K1) : rien n'est envoyé. */
	function modifyEntry() {
		if (phase !== 'J5') return;
		duplicate = null;
		phase = 'J1';
	}

	/** « J'ai vérifié, elle n'a pas été passée — reprendre la saisie » (K6 → K1) : rien n'est envoyé. */
	function resumeEntry() {
		if (phase !== 'J6') return;
		phase = 'J1';
	}

	/** L'envoi lui-même : succès (K3), doublon (K5), issue inconnue (K6), erreur (K7). */
	async function transmit(body: CreateConversionRequest, confirm: boolean) {
		try {
			await createConversion(confirm ? { ...body, confirmNotDuplicate: true } : body);
			phase = 'J3';
			toast.success(i18nMsg('journal-entries-conversion-done', 'Conversion passée : deux écritures enregistrées.'));
			onDone();
		} catch (err) {
			// Sorties de K2b : le refus de doublon (K5, **avec** son rang), l'issue inconnue
			// (K6 : tout 5xx, coupure, délai, erreur qui n'est pas une `ApiError` — on ne
			// dit plus « La conversion n'a pas pu être passée » quand l'issue est
			// inconnue), le reste (K7, traitement ordinaire).
			const next = stateAfterRejection(err, isSettlementOutcomeUnknown, { needsEntry: true });
			if (next === 'J5') {
				duplicate = readEntryDuplicateRefusal(err);
				phase = 'J5';
				return;
			}
			if (next === 'J6') {
				phase = 'J6';
				return;
			}
			// Défensif : `isSettlementOutcomeUnknown` range déjà toute erreur hors
			// `ApiError` en K6 ; une autre ne dirait rien de l'issue.
			if (!isApiError(err)) {
				phase = 'J6';
				return;
			}
			phase = 'J7';
			const message = isFxRefusalCode(err.code) ? fxRefusalMessage(err.code, err.details, err.message) : err.message;
			const entry = refusalEntry(err.details);
			const line = refusalLine(err.details);
			if (entry === null) {
				globalError = message;
			} else {
				const side = entry === 1 ? 'debit' : 'credit';
				// Le corps ne porte que les frais dont le compte est choisi : le rang
				// d'un frais refusé se ramène à son indice dans `col.fees`, lignes
				// vides comprises (revue de code P1, 37-3b-CR-P1-12).
				const sentFees = getCol(side)
					.fees.map((f, k) => ({ f, k }))
					.filter(({ f }) => f.accountId !== null)
					.map(({ k }) => k);
				const field = line === null ? null : conversionField(line, sentFees.length);
				if (field?.kind === 'fee' && sentFees[field.index] !== undefined) {
					feeErrors = { ...feeErrors, [side]: { [sentFees[field.index]]: message } };
				} else if (field?.kind === 'difference') globalError = message;
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
					onblur={() => updateForeign(side, commitChfEntry)}
				/>
			</label>
			{@const problem = foreignLineProblem(f)}
			{#if problem !== null && (f.amount.trim() !== '' || f.chfError !== null)}
				<p class="text-xs text-destructive" data-testid="conversion-{side}-problem">{foreignProblemMessage(problem)}</p>
			{/if}
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
	<!-- Story 53-1b (table K) — UN `<fieldset disabled>` enveloppe tous les contrôles qui
	     écrivent la saisie (date, comptes, montants, cours, CHF, frais) ; figé en K2a, K2b,
	     K3, K5, K6, sans rien vider. Atténué : un champ inactif doit en avoir l'air. L'aperçu,
	     les avis (K5, K6) et leurs boutons sont hors du fieldset, donc actifs. -->
	<fieldset
		class="m-0 min-w-0 border-0 p-0 disabled:opacity-60"
		disabled={frozen}
		data-testid="conversion-fields"
	>
		<div class="grid grid-cols-1 gap-4 md:grid-cols-2">
			{@render column('debit')}
			{@render column('credit')}
		</div>
	</fieldset>

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
		<!-- Story 53-1b — les avis de K5 (doublon probable) et K6 (issue inconnue). -->
		{#if phase === 'J5' && duplicate !== null}
			<SupplierInvoiceNotice
				testid="conversion-probable-duplicate"
				text={duplicate.entry === 1
					? i18nMsg(
							'journal-entries-conversion-probable-duplicate-debit',
							'L’écriture de la colonne Débit de cette conversion existe déjà à l’identique. Si la conversion est déjà passée, ne la passez pas une seconde fois ; sinon, confirmez que ce n’est pas un doublon.'
						)
					: i18nMsg(
							'journal-entries-conversion-probable-duplicate-credit',
							'L’écriture de la colonne Crédit de cette conversion existe déjà à l’identique. Si la conversion est déjà passée, ne la passez pas une seconde fois ; sinon, confirmez que ce n’est pas un doublon.'
						)}
				link={{
					href: `/journal-entries/${duplicate.journalEntryId}`,
					label: i18nMsg('journal-entries-duplicate-existing', 'Écriture n° { $number } du { $date }', {
						number: duplicate.entryNumber,
						date: formatSwissDate(duplicate.entryDate)
					}),
					testid: 'conversion-duplicate-link'
				}}
			>
				<button
					type="button"
					class="rounded bg-primary px-3 py-1 text-sm text-primary-foreground"
					data-testid="conversion-confirm-not-duplicate"
					onclick={() => confirmDuplicate()}
				>
					{i18nMsg('journal-entries-duplicate-confirm', 'Ce n’est pas un doublon, enregistrer')}
				</button>
				<button
					type="button"
					class="rounded border px-3 py-1 text-sm"
					data-testid="conversion-modify-entry"
					onclick={() => modifyEntry()}
				>
					{i18nMsg('journal-entries-duplicate-edit', 'Modifier la saisie')}
				</button>
			</SupplierInvoiceNotice>
		{/if}
		{#if phase === 'J6'}
			<SupplierInvoiceNotice
				testid="conversion-unknown-outcome"
				text={i18nMsg(
					'journal-entries-conversion-unknown-outcome',
					'La conversion a peut-être été passée. Vérifiez dans le journal si ses deux écritures existent avant de la passer de nouveau.'
				)}
				link={{
					href: unknownOutcomeRangeHref(
						kept?.dates[0] ?? debitCol.date,
						kept?.dates[1] ?? creditCol.date
					),
					label: i18nMsg('journal-entries-conversion-unknown-outcome-link', 'Voir les écritures de ces dates'),
					testid: 'conversion-unknown-outcome-link'
				}}
			>
				<button
					type="button"
					class="rounded border px-3 py-1 text-sm"
					data-testid="conversion-resume-entry"
					onclick={() => resumeEntry()}
				>
					{i18nMsg(
						'journal-entries-conversion-unknown-outcome-resume',
						'J’ai vérifié, elle n’a pas été passée — reprendre la saisie'
					)}
				</button>
			</SupplierInvoiceNotice>
		{/if}
		<div class="flex justify-end">
			{#if showSubmit}
				<Button
					type="button"
					data-testid="conversion-submit"
					disabled={plan.kind !== 'ready' || submitting || !canSendFrom(phase)}
					onclick={submit}
				>
					{i18nMsg('journal-entries-conversion-submit', 'Passer les écritures')}
				</Button>
			{/if}
		</div>
	</section>
</div>
