<script lang="ts">
	import { Button } from '$lib/components/ui/button';
	import { Input } from '$lib/components/ui/input';
	import * as Select from '$lib/components/ui/select';
	import { untrack } from 'svelte';
	import { ChevronDown, ChevronRight } from '@lucide/svelte';
	import { i18nMsg } from '$lib/features/onboarding/onboarding.svelte';
	import { accountCurrency, type AccountResponse } from '$lib/features/accounts/accounts.types';
	import { getVatRates } from '$lib/features/vat-rates/vat-rates.store.svelte';
	import type { VatRateResponse } from '$lib/features/vat-rates/vat-rates.types';
	import type { LineDraft } from './form-helpers';
	import {
		foreignPurchaseVatEntry,
		htAmountProblem,
		purchaseVatEntry,
		readHtAmount
	} from './vat-purchase';
	import {
		accountsForCurrencies,
		applyLookup,
		monthOf,
		newForeign,
		type LineForeign
	} from './fx-lines';
	import { readRate } from './fx';
	import type { LookupResult } from './rate-lookup';
	import AccountAutocomplete from './AccountAutocomplete.svelte';

	interface Props {
		accounts: AccountResponse[];
		accountsLoadError: boolean;
		/** Compte d'impôt préalable (`default_vat_recoverable_account_id`), null si non configuré. */
		recoverableAccountId: number | null;
		/** Appelé à l'insertion : lignes générées + libellé suggéré. */
		onApply: (result: { lines: LineDraft[]; description: string }) => void;
		/**
		 * Story 37-3b (S5) — société multi-devises : la charge ou la contrepartie
		 * peut être un compte en devise ; l'assistant passe alors en devise.
		 */
		multiCurrency?: boolean;
		/** Story 37-3b (S5) — la date de l'écriture (mois du cours proposé). */
		entryDate?: string;
		/** Story 37-3b (S5) — le `lookup` mémoïsé du formulaire (aucun second cache). */
		lookupRate?: (currency: string, date: string) => Promise<LookupResult>;
	}

	let {
		accounts,
		accountsLoadError,
		recoverableAccountId,
		onApply,
		multiCurrency = false,
		entryDate = '',
		lookupRate = undefined
	}: Props = $props();
	// fx-filtre-comptes : exempté — en multi-devises, ses deux sélecteurs offrent les
	// comptes en devise (`excludeForeignCurrency = false`) et l'assistant restreint
	// lui-même à CHF plus une devise (37-3b, S5) ; en mono-devise, le défaut filtre.

	const uid = $props.id();

	let expanded = $state(false);
	let chargeAccountId = $state<number | null>(null);
	let htAmount = $state('');
	let ratePercent = $state<string | null>(null);
	let counterpartyAccountId = $state<number | null>(null);

	let rates = $state<VatRateResponse[]>([]);
	let ratesLoaded = $state(false);

	// Chargement paresseux des taux à la première ouverture du panneau.
	async function ensureRates() {
		if (ratesLoaded) return;
		try {
			rates = await getVatRates();
		} catch {
			rates = [];
		} finally {
			ratesLoaded = true;
		}
	}

	function toggle() {
		expanded = !expanded;
		if (expanded) void ensureRates();
	}

	const configMissing = $derived(recoverableAccountId === null);
	const noRates = $derived(ratesLoaded && rates.length === 0);

	/** Libellé d'affichage d'un taux (rate formaté + catégorie, fallback robuste). */
	function rateLabel(r: VatRateResponse): string {
		const cat =
			r.category && r.category.length > 0
				? i18nMsg(`vat-category-${r.category}`, r.label)
				: r.label || `${r.rate} %`;
		return `${r.rate} % — ${cat}`;
	}

	const selectedRate = $derived(rates.find((r) => r.rate === ratePercent) ?? null);

	// HT valide ssi clair à la lecture du journal ET arrondi au centime strictement
	// > 0 : un HT sub-centime (ex. 0.001) arrondirait la ligne de charge à 0.00 →
	// écriture rejetée (`debit > 0`). Story 36-3 (#1051, D5) : « 1'234.50 » admis,
	// « 1.234 » refusé comme ambigu (`readHtAmount`).
	const htReading = $derived(readHtAmount(htAmount));
	const htValid = $derived(htReading.usable);
	const htProblem = $derived(htAmountProblem(htReading));

	// AC8 : garde de validation des entrées.
	const sameChargeCounterparty = $derived(
		chargeAccountId !== null && chargeAccountId === counterpartyAccountId
	);
	const chargeIsRecoverable = $derived(
		chargeAccountId !== null && chargeAccountId === recoverableAccountId
	);
	const counterpartyIsRecoverable = $derived(
		counterpartyAccountId !== null && counterpartyAccountId === recoverableAccountId
	);

	// --- Story 37-3b (S5) : l'assistant en devise -----------------------------
	function currencyOf(id: number | null): string | null {
		if (id === null) return null;
		const a = accounts.find((x) => x.id === id);
		return a ? accountCurrency(a) : null;
	}
	const chargeCurrency = $derived(multiCurrency ? currencyOf(chargeAccountId) : null);
	const counterpartyCurrency = $derived(multiCurrency ? currencyOf(counterpartyAccountId) : null);
	/** Charge et contrepartie dans deux devises différentes : refusé (R7). */
	const twoCurrencies = $derived(
		chargeCurrency !== null && counterpartyCurrency !== null && chargeCurrency !== counterpartyCurrency
	);
	/** La devise de l'assistant, `null` en CHF. */
	const fxCurrency = $derived(twoCurrencies ? null : (chargeCurrency ?? counterpartyCurrency));
	/** Le sélecteur de l'autre compte n'offre que CHF et la devise déjà choisie. */
	const chargeOptions = $derived(
		accountsForCurrencies(accounts, counterpartyCurrency ? new Set([counterpartyCurrency]) : null)
	);
	const counterpartyOptions = $derived(
		accountsForCurrencies(accounts, chargeCurrency ? new Set([chargeCurrency]) : null)
	);

	/** Le cours de l'assistant (gabarit des lignes en devise), proposé par `lookup`. */
	let fxTemplate = $state<LineForeign | null>(null);
	let fxLookupToken = 0;
	$effect(() => {
		const currency = fxCurrency;
		const date = entryDate;
		if (currency === null) {
			fxTemplate = null;
			return;
		}
		untrack(() => {
			if (fxTemplate === null || fxTemplate.currency !== currency) fxTemplate = newForeign(currency);
		});
		if (!lookupRate || !/^\d{4}-\d{2}-\d{2}$/.test(date)) return;
		const token = ++fxLookupToken;
		void lookupRate(currency, date).then((r) => {
			if (token !== fxLookupToken || fxTemplate === null) return;
			fxTemplate = applyLookup(fxTemplate, monthOf(date), r.outcome);
		});
	});
	const fxRateValid = $derived(
		fxTemplate === null || (() => {
			const r = readRate(fxTemplate.rate);
			return r !== null && r !== 'empty';
		})()
	);
	/** Refus du calcul en devise (ligne CHF dérivée à 0.00), à l'insertion. */
	let fxError = $state<string | null>(null);

	/** Prévisualisation du refus « ligne CHF à zéro » (insertion désactivée). */
	const fxPreview = $derived.by(() => {
		if (fxTemplate === null || recoverableAccountId === null || ratePercent === null) return null;
		if (chargeAccountId === null || counterpartyAccountId === null) return null;
		return foreignPurchaseVatEntry(
			{
				chargeAccountId,
				htAmount,
				ratePercent,
				counterpartyAccountId,
				recoverableAccountId
			},
			{ template: fxTemplate, chargeIsForeign: chargeCurrency !== null, counterpartyIsForeign: counterpartyCurrency !== null }
		);
	});
	const fxPreviewError = $derived(fxPreview !== null && 'error' in fxPreview ? fxPreview.error : null);

	const canInsert = $derived(
		!configMissing &&
			!noRates &&
			chargeAccountId !== null &&
			counterpartyAccountId !== null &&
			ratePercent !== null &&
			htValid &&
			!sameChargeCounterparty &&
			!chargeIsRecoverable &&
			!counterpartyIsRecoverable &&
			!twoCurrencies &&
			fxRateValid &&
			fxPreviewError === null
	);

	function insert() {
		if (!canInsert || recoverableAccountId === null || ratePercent === null) return;
		fxError = null;
		const params = {
			chargeAccountId: chargeAccountId!,
			htAmount,
			ratePercent,
			counterpartyAccountId: counterpartyAccountId!,
			recoverableAccountId
		};
		// Story 36-3 (D5) : lignes tirées de la charge normalisée (« 1'234.50 » →
		// 1234.50), jamais du HT brut ; `null` si le HT n'est pas utilisable.
		// Story 37-3b (S5) : en devise, la TVA se calcule sur le montant en devise
		// et chaque ligne CHF au même cours.
		const entry =
			fxTemplate !== null
				? foreignPurchaseVatEntry(params, {
						template: fxTemplate,
						chargeIsForeign: chargeCurrency !== null,
						counterpartyIsForeign: counterpartyCurrency !== null
					})
				: purchaseVatEntry(params);
		if (entry === null) return;
		if ('error' in entry) {
			fxError = entry.error;
			return;
		}
		const { lines, isExempt } = entry;
		const description = isExempt
			? i18nMsg('vat-purchase-description-exempt', 'Achat — sans TVA')
			: i18nMsg('vat-purchase-description', 'Achat — TVA {$rate} % récupérable', {
					rate: ratePercent
				});
		onApply({ lines, description });
	}
</script>

<div class="rounded-md border border-border bg-muted/20" data-testid="vat-purchase-assistant">
	<button
		type="button"
		class="flex w-full items-center gap-2 px-4 py-2 text-left text-sm font-medium"
		onclick={toggle}
		aria-expanded={expanded}
		aria-controls="{uid}-panel"
	>
		{#if expanded}
			<ChevronDown class="h-4 w-4" />
		{:else}
			<ChevronRight class="h-4 w-4" />
		{/if}
		{i18nMsg('vat-purchase-title', 'Assistant TVA achat')}
	</button>

	{#if expanded}
		<div id="{uid}-panel" class="space-y-4 border-t border-border p-4">
			{#if configMissing}
				<p class="text-sm text-destructive">
					{i18nMsg(
						'vat-purchase-config-required',
						'Configurez le compte d’impôt préalable dans Administration → Facturation pour utiliser l’assistant.'
					)}
				</p>
			{:else if noRates}
				<p class="text-sm text-destructive">
					{i18nMsg(
						'vat-purchase-no-rates',
						'Aucun taux de TVA configuré — voir Paramètres → Taux de TVA.'
					)}
				</p>
			{/if}

			<div class="grid grid-cols-1 gap-4 md:grid-cols-2">
				<div>
					<label for="{uid}-charge" class="mb-1 block text-sm font-medium">
						{i18nMsg('vat-purchase-charge-account', 'Compte de charge')}
					</label>
					<AccountAutocomplete
						accounts={chargeOptions}
						value={chargeAccountId}
						loadError={accountsLoadError}
						excludeForeignCurrency={!multiCurrency}
						onSelect={(id) => (chargeAccountId = id)}
					/>
				</div>
				<div>
					<label for="{uid}-ht" class="mb-1 block text-sm font-medium">
						{#if fxCurrency !== null}
							{i18nMsg('journal-entries-vat-ht-currency', 'Montant HT ({ $currency })', {
								currency: fxCurrency
							})}
						{:else}
							{i18nMsg('vat-purchase-ht', 'Montant HT')}
						{/if}
					</label>
					<Input
						id="{uid}-ht"
						type="text"
						inputmode="decimal"
						bind:value={htAmount}
						class={htAmount !== '' && !htValid ? 'border-destructive' : 'tabular-nums text-right'}
						placeholder="0.00"
					/>
					<!-- Story 36-3, revue P1 (B3, E3) : chaque refus du HT dit pourquoi. -->
					{#if htProblem === 'ambiguous'}
						<p class="mt-1 text-xs text-destructive" data-testid="vat-purchase-ht-ambiguous">
							{i18nMsg(
								'journal-entry-form-amount-ambiguous',
								'Montant ambigu : un point ou une virgule suivis de trois chiffres peuvent séparer les milliers. Écrivez 1\'234 pour mille deux cent trente-quatre, ou 1.2340 pour un franc vingt-trois et quatre dixièmes de centime.'
							)}
						</p>
					{:else if htProblem === 'unreadable'}
						<p class="mt-1 text-xs text-destructive" data-testid="vat-purchase-ht-unreadable">
							{i18nMsg(
								'journal-entry-form-amount-unreadable',
								'Montant illisible : chiffres, apostrophe des milliers (1\'234), point ou virgule décimale, quatre décimales au plus.'
							)}
						</p>
					{:else if htProblem === 'below-centime'}
						<p class="mt-1 text-xs text-destructive" data-testid="vat-purchase-ht-below-centime">
							{i18nMsg(
								'vat-purchase-ht-below-centime',
								'Le montant HT s\'arrondit à 0.00 au centime : la ligne de charge serait nulle.'
							)}
						</p>
					{/if}
				</div>
				<div>
					<label for="{uid}-rate" class="mb-1 block text-sm font-medium">
						{i18nMsg('vat-purchase-rate', 'Taux TVA')}
					</label>
					<Select.Root
						type="single"
						value={ratePercent ?? ''}
						onValueChange={(v) => (ratePercent = v)}
					>
						<Select.Trigger id="{uid}-rate">
							{selectedRate
								? rateLabel(selectedRate)
								: i18nMsg('vat-purchase-rate-placeholder', 'Choisir un taux')}
						</Select.Trigger>
						<Select.Content>
							{#each rates as r (r.id)}
								<Select.Item value={r.rate}>{rateLabel(r)}</Select.Item>
							{/each}
						</Select.Content>
					</Select.Root>
				</div>
				<div>
					<label for="{uid}-counterparty" class="mb-1 block text-sm font-medium">
						{i18nMsg('vat-purchase-counterparty', 'Compte de contrepartie')}
					</label>
					<AccountAutocomplete
						accounts={counterpartyOptions}
						value={counterpartyAccountId}
						loadError={accountsLoadError}
						excludeForeignCurrency={!multiCurrency}
						onSelect={(id) => (counterpartyAccountId = id)}
					/>
				</div>
				{#if fxTemplate !== null}
					<div>
						<label for="{uid}-fx-rate" class="mb-1 block text-sm font-medium">
							{i18nMsg('journal-entries-vat-fx-rate', 'Cours ({ $currency } → CHF)', {
								currency: fxTemplate.currency
							})}
						</label>
						<Input
							id="{uid}-fx-rate"
							type="text"
							inputmode="decimal"
							data-testid="vat-purchase-fx-rate"
							value={fxTemplate.rate}
							oninput={(e) => {
								if (fxTemplate === null) return;
								fxTemplate = {
									...fxTemplate,
									rate: (e.currentTarget as HTMLInputElement).value,
									rateProposed: false,
									touched: true
								};
							}}
							class={fxRateValid ? 'tabular-nums text-right' : 'border-destructive'}
							placeholder="0.0000"
						/>
						{#if !fxRateValid}
							<p class="mt-1 text-xs text-destructive">
								{i18nMsg(
									'journal-entries-fx-rate-invalid',
									'Cours invalide : strictement positif, dix décimales et 1 000 000 au plus.'
								)}
							</p>
						{/if}
					</div>
				{/if}
			</div>

			{#if twoCurrencies}
				<p class="text-xs text-destructive" data-testid="vat-purchase-two-currencies">
					{i18nMsg(
						'journal-entries-vat-two-currencies',
						'La charge et la contrepartie sont dans deux devises différentes : une écriture ne porte qu’une devise étrangère.'
					)}
				</p>
			{/if}
			{#if fxPreviewError === 'FX_LINE_ROUNDS_TO_ZERO' || fxError === 'FX_LINE_ROUNDS_TO_ZERO'}
				<p class="text-xs text-destructive" data-testid="vat-purchase-rounds-to-zero">
					{i18nMsg(
						'journal-entries-fx-rounds-to-zero',
						'Le montant en CHF calculé s’arrondit à zéro.'
					)}
				</p>
			{/if}

			{#if sameChargeCounterparty}
				<p class="text-xs text-destructive">
					{i18nMsg(
						'vat-purchase-same-account',
						'Le compte de charge et la contrepartie doivent être différents.'
					)}
				</p>
			{/if}
			{#if chargeIsRecoverable || counterpartyIsRecoverable}
				<p class="text-xs text-destructive">
					{i18nMsg(
						'vat-purchase-recoverable-conflict',
						'Le compte de charge et la contrepartie ne peuvent pas être le compte d’impôt préalable.'
					)}
				</p>
			{/if}

			<div class="flex justify-end">
				<Button type="button" size="sm" onclick={insert} disabled={!canInsert}>
					{i18nMsg('vat-purchase-insert', 'Insérer les lignes')}
				</Button>
			</div>
		</div>
	{/if}
</div>
