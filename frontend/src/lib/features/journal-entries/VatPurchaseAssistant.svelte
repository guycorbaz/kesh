<script lang="ts">
	import { Button } from '$lib/components/ui/button';
	import { Input } from '$lib/components/ui/input';
	import * as Select from '$lib/components/ui/select';
	import { ChevronDown, ChevronRight } from '@lucide/svelte';
	import { i18nMsg } from '$lib/features/onboarding/onboarding.svelte';
	import type { AccountResponse } from '$lib/features/accounts/accounts.types';
	import { getVatRates } from '$lib/features/vat-rates/vat-rates.store.svelte';
	import type { VatRateResponse } from '$lib/features/vat-rates/vat-rates.types';
	import { isValidAmount } from './balance';
	import type { LineDraft } from './form-helpers';
	import { buildPurchaseVatLines, lineVatAmount } from './vat-purchase';
	import AccountAutocomplete from './AccountAutocomplete.svelte';

	interface Props {
		accounts: AccountResponse[];
		accountsLoadError: boolean;
		/** Compte d'impôt préalable (`default_vat_recoverable_account_id`), null si non configuré. */
		recoverableAccountId: number | null;
		/** Appelé à l'insertion : lignes générées + libellé suggéré. */
		onApply: (result: { lines: LineDraft[]; description: string }) => void;
	}

	let { accounts, accountsLoadError, recoverableAccountId, onApply }: Props = $props();

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

	const htValid = $derived(isValidAmount(htAmount) && htAmount.trim() !== '' && Number(htAmount.replace(',', '.')) > 0);

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

	const canInsert = $derived(
		!configMissing &&
			!noRates &&
			chargeAccountId !== null &&
			counterpartyAccountId !== null &&
			ratePercent !== null &&
			htValid &&
			!sameChargeCounterparty &&
			!chargeIsRecoverable &&
			!counterpartyIsRecoverable
	);

	function insert() {
		if (!canInsert || recoverableAccountId === null || ratePercent === null) return;
		const lines = buildPurchaseVatLines({
			chargeAccountId: chargeAccountId!,
			htAmount,
			ratePercent,
			counterpartyAccountId: counterpartyAccountId!,
			recoverableAccountId
		});
		const isExempt = lineVatAmount(htAmount, ratePercent) === '0.00';
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
						'Configurez le compte d’impôt préalable dans Paramètres → Facturation pour utiliser l’assistant.'
					)}
				</p>
			{:else if noRates}
				<p class="text-sm text-destructive">
					{i18nMsg(
						'vat-purchase-no-rates',
						'Aucun taux TVA configuré — voir Paramètres → Taux TVA.'
					)}
				</p>
			{/if}

			<div class="grid grid-cols-1 gap-4 md:grid-cols-2">
				<div>
					<label for="{uid}-charge" class="mb-1 block text-sm font-medium">
						{i18nMsg('vat-purchase-charge-account', 'Compte de charge')}
					</label>
					<AccountAutocomplete
						{accounts}
						value={chargeAccountId}
						loadError={accountsLoadError}
						onSelect={(id) => (chargeAccountId = id)}
					/>
				</div>
				<div>
					<label for="{uid}-ht" class="mb-1 block text-sm font-medium">
						{i18nMsg('vat-purchase-ht', 'Montant HT')}
					</label>
					<Input
						id="{uid}-ht"
						type="text"
						inputmode="decimal"
						bind:value={htAmount}
						class={htAmount !== '' && !htValid ? 'border-destructive' : 'tabular-nums text-right'}
						placeholder="0.00"
					/>
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
						{accounts}
						value={counterpartyAccountId}
						loadError={accountsLoadError}
						onSelect={(id) => (counterpartyAccountId = id)}
					/>
				</div>
			</div>

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
