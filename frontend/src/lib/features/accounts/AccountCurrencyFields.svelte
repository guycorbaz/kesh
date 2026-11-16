<script lang="ts">
	/**
	 * Story 37-1 (AC-7, AC-4, AC-4 bis) — champs « devise » du formulaire de
	 * compte, rendus par le *Plan comptable* en **multi-devises seulement** (le
	 * parent ne monte pas ce composant en mono-devise : aucun changement visible).
	 *
	 * - « Devise » : sélecteur ISO 4217, vide = CHF ; désactivé avec l'explication
	 *   si la devise est verrouillée (le compte porte une écriture).
	 * - Pour un compte **en devise** : « Comptes de différence de change : par
	 *   défaut / propres ». « Par défaut » rappelle les comptes de la société, lus
	 *   par `fxAccountsFor` ; « propres » ouvre deux sélecteurs (mêmes filtres que
	 *   les paramètres). Remettre la devise à vide repasse le choix à « par défaut ».
	 *
	 * Le composant ne calcule pas le corps envoyé : `currencyFieldsForCreate` /
	 * `currencyFieldsForUpdate` le font, et n'envoient que ce qui change.
	 */
	import type { CompanyJson } from '$lib/features/settings/settings.types';
	import { FOREIGN_CURRENCIES } from '$lib/shared/utils/iso4217';
	import { i18nMsg } from '$lib/shared/utils/i18n.svelte';
	import { withCurrentAccount } from './account-options';
	import type { AccountResponse } from './accounts.types';
	import {
		fxAccountsFor,
		fxDifferenceAccountOptions,
		type AccountCurrencyForm,
		type FxAccountField,
	} from './fx-accounts';

	interface Props {
		form: AccountCurrencyForm;
		accounts: AccountResponse[];
		company: Pick<CompanyJson, 'fxGainAccountId' | 'fxLossAccountId'>;
		/** Préfixe des `id` et `data-testid` (`account-create` / `account-edit`). */
		idPrefix: string;
		/** La devise ne peut plus changer (`currencyLocked`). */
		locked?: boolean;
		/** Refus `FX_ACCOUNT_INVALID` du serveur, par champ (`details.field`). */
		fieldErrors?: Partial<Record<FxAccountField, string>>;
		/**
		 * Le compte modifié (absent à la création) : il ne se propose pas comme
		 * son propre compte de différence, il porte la devise (37-1-CR-P2-1).
		 */
		accountId?: number;
	}

	let {
		form = $bindable(),
		accounts,
		company,
		idPrefix,
		locked = false,
		fieldErrors = {},
		accountId,
	}: Props = $props();

	/** La liste ISO, plus la devise en place si elle n'y figure plus (code retiré). */
	let currencyOptions = $derived(
		form.currency !== '' && !FOREIGN_CURRENCIES.includes(form.currency)
			? [form.currency, ...FOREIGN_CURRENCIES]
			: FOREIGN_CURRENCIES,
	);
	let differenceAccounts = $derived(
		fxDifferenceAccountOptions(accounts).filter((a) => a.id !== accountId),
	);
	let gainOptions = $derived(
		withCurrentAccount(differenceAccounts, form.fxGain === '' ? null : Number(form.fxGain), accounts),
	);
	let lossOptions = $derived(
		withCurrentAccount(differenceAccounts, form.fxLoss === '' ? null : Number(form.fxLoss), accounts),
	);
	/** Les comptes de la société que suivrait le compte « par défaut ». */
	let companyFx = $derived(fxAccountsFor({ currency: form.currency || undefined }, company));

	function accountName(id: number): string {
		const a = accounts.find((x) => x.id === id);
		return a ? `${a.number} ${a.name}` : `#${id}`;
	}

	function onCurrencyChange() {
		if (form.currency === '') {
			form.fxChoice = 'default';
			form.fxGain = '';
			form.fxLoss = '';
		}
	}

	const SELECT_CLASS =
		'mt-1 w-full rounded-md border border-border bg-white px-3 py-2 text-sm disabled:bg-surface-alt disabled:text-text-muted';
</script>

<div>
	<label for="{idPrefix}-currency" class="text-sm font-medium text-text">
		{i18nMsg('accounts-currency-label', 'Devise')}
	</label>
	<select
		id="{idPrefix}-currency"
		class={SELECT_CLASS}
		bind:value={form.currency}
		onchange={onCurrencyChange}
		disabled={locked}
		data-testid="{idPrefix}-dialog-currency"
	>
		<option value="">{i18nMsg('accounts-currency-base', 'CHF (devise principale)')}</option>
		{#each currencyOptions as c (c)}
			<option value={c}>{c}</option>
		{/each}
	</select>
	{#if locked}
		<p class="mt-1 text-xs text-text-muted" data-testid="{idPrefix}-dialog-currency-locked">
			{i18nMsg(
				'accounts-currency-locked',
				'La devise ne peut plus changer : ce compte porte déjà des écritures.',
			)}
		</p>
	{/if}
</div>

{#if form.currency !== ''}
	<fieldset class="space-y-2" data-testid="{idPrefix}-dialog-fx">
		<legend class="text-sm font-medium text-text">
			{i18nMsg('accounts-fx-legend', 'Comptes de différence de change')}
		</legend>
		<label class="flex items-center gap-2 text-sm">
			<input
				type="radio"
				value="default"
				bind:group={form.fxChoice}
				data-testid="{idPrefix}-dialog-fx-default"
			/>
			{i18nMsg('accounts-fx-default', 'Par défaut (ceux de la société)')}
		</label>
		{#if form.fxChoice === 'default'}
			<p class="ml-6 text-xs text-text-muted" data-testid="{idPrefix}-dialog-fx-company">
				{#if companyFx}
					{accountName(companyFx.gain)} / {accountName(companyFx.loss)}
				{:else}
					{i18nMsg(
						'accounts-fx-company-none',
						'La société n’a pas désigné de comptes de différence de change.',
					)}
				{/if}
			</p>
		{/if}
		<label class="flex items-center gap-2 text-sm">
			<input type="radio" value="own" bind:group={form.fxChoice} data-testid="{idPrefix}-dialog-fx-own" />
			{i18nMsg('accounts-fx-own', 'Propres à ce compte')}
		</label>
		{#if form.fxChoice === 'own'}
			<div class="ml-6 space-y-2">
				<div>
					<label for="{idPrefix}-fx-gain" class="text-xs font-medium text-text">
						{i18nMsg('accounts-fx-gain-account', 'Compte de gains de change')}
					</label>
					<select
						id="{idPrefix}-fx-gain"
						class={SELECT_CLASS}
						bind:value={form.fxGain}
						aria-invalid={fieldErrors.fxGainAccountId ? 'true' : undefined}
						data-testid="{idPrefix}-dialog-fx-gain"
					>
						<option value="">{i18nMsg('accounts-fx-select-none', '— Sélectionner —')}</option>
						{#each gainOptions as a (a.id)}
							<option value={String(a.id)}>{a.number} — {a.name}</option>
						{/each}
					</select>
					{#if fieldErrors.fxGainAccountId}
						<p class="mt-1 text-xs text-destructive" role="alert" data-testid="{idPrefix}-dialog-fx-gain-error">
							{fieldErrors.fxGainAccountId}
						</p>
					{/if}
				</div>
				<div>
					<label for="{idPrefix}-fx-loss" class="text-xs font-medium text-text">
						{i18nMsg('accounts-fx-loss-account', 'Compte de pertes de change')}
					</label>
					<select
						id="{idPrefix}-fx-loss"
						class={SELECT_CLASS}
						bind:value={form.fxLoss}
						aria-invalid={fieldErrors.fxLossAccountId ? 'true' : undefined}
						data-testid="{idPrefix}-dialog-fx-loss"
					>
						<option value="">{i18nMsg('accounts-fx-select-none', '— Sélectionner —')}</option>
						{#each lossOptions as a (a.id)}
							<option value={String(a.id)}>{a.number} — {a.name}</option>
						{/each}
					</select>
					{#if fieldErrors.fxLossAccountId}
						<p class="mt-1 text-xs text-destructive" role="alert" data-testid="{idPrefix}-dialog-fx-loss-error">
							{fieldErrors.fxLossAccountId}
						</p>
					{/if}
				</div>
			</div>
		{/if}
	</fieldset>
{/if}
