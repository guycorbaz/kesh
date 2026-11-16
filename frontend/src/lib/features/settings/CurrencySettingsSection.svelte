<script lang="ts">
	/**
	 * Story 37-1 (AC-7) — bloc « Comptabilité multi-devises » de
	 * *Paramètres → Comptabilité* : la case d'activation et les trois comptes
	 * (gains de change, pertes de change, conversion).
	 *
	 * - L'activation est **irréversible** (CR-7) : enregistrer la case cochée sur
	 *   une société mono-devise ouvre une confirmation, et seul *Activer* envoie
	 *   le `PUT`. Une fois actif, la case reste cochée et désactivée ; les trois
	 *   sélecteurs restent modifiables (jamais vidables).
	 * - Éditable par l'Administrateur seul ; lecture seule pour les autres rôles
	 *   (le serveur refuse de toute façon le `PUT`, 403).
	 * - Les filtres des sélecteurs sont un confort ; le refus qui fait autorité est
	 *   celui du serveur (`FX_ACCOUNT_INVALID`, affiché sous le sélecteur que
	 *   désigne `details.field`).
	 */
	import { onMount } from 'svelte';
	import * as Dialog from '$lib/components/ui/dialog';
	import { Button } from '$lib/components/ui/button';
	import { fetchAccounts } from '$lib/features/accounts/accounts.api';
	import type { AccountResponse } from '$lib/features/accounts/accounts.types';
	import { withCurrentAccount } from '$lib/features/accounts/account-options';
	import {
		fxDifferenceAccountOptions,
		fxTransitAccountOptions,
		readFxAccountField,
		type FxAccountField,
	} from '$lib/features/accounts/fx-accounts';
	import { isApiError } from '$lib/shared/utils/api-client';
	import { errorMessageOf } from '$lib/shared/utils/error-message';
	import { i18nMsg } from '$lib/shared/utils/i18n.svelte';
	import { notifySuccess } from '$lib/shared/utils/notify';
	import { fetchCompanyCurrent, updateCurrencySettings } from './settings.api';
	import type { CompanyJson } from './settings.types';

	interface Props {
		company: CompanyJson;
		/** Seul l'Administrateur règle le multi-devises. */
		isAdmin: boolean;
		/** Reçoit la société rendue par le serveur (enregistrement ou relecture après un 409). */
		onSaved: (company: CompanyJson) => void;
	}

	let { company, isAdmin, onSaved }: Props = $props();

	let accounts = $state<AccountResponse[]>([]);
	let enabled = $state(false);
	let gainId = $state<number | null>(null);
	let lossId = $state<number | null>(null);
	let transitId = $state<number | null>(null);
	/** Version de la société dont viennent les champs : une société plus récente les recharge. */
	let syncedVersion = $state<number | null>(null);
	let submitting = $state(false);
	let error = $state('');
	let fieldErrors = $state<Partial<Record<FxAccountField, string>>>({});
	let confirmOpen = $state(false);

	function resetFrom(c: CompanyJson) {
		enabled = c.multiCurrencyEnabled;
		gainId = c.fxGainAccountId;
		lossId = c.fxLossAccountId;
		transitId = c.fxTransitAccountId;
		syncedVersion = c.version;
	}

	$effect(() => {
		if (!submitting && company.version !== syncedVersion) resetFrom(company);
	});

	onMount(async () => {
		try {
			accounts = await fetchAccounts(false);
		} catch {
			// Sans la liste, les sélecteurs n'offrent que la valeur enregistrée ; le
			// serveur reste juge de tout enregistrement.
			accounts = [];
		}
	});

	let active = $derived(company.multiCurrencyEnabled);
	let editable = $derived(isAdmin && !submitting);
	let differenceAccounts = $derived(fxDifferenceAccountOptions(accounts));
	let transitAccounts = $derived(fxTransitAccountOptions(accounts));
	let gainOptions = $derived(withCurrentAccount(differenceAccounts, gainId, accounts));
	let lossOptions = $derived(withCurrentAccount(differenceAccounts, lossId, accounts));
	let transitOptions = $derived(withCurrentAccount(transitAccounts, transitId, accounts));

	let dirty = $derived(
		enabled !== company.multiCurrencyEnabled ||
			(enabled &&
				(gainId !== company.fxGainAccountId ||
					lossId !== company.fxLossAccountId ||
					transitId !== company.fxTransitAccountId)),
	);

	/** Libellé d'une option : numéro et nom ; un compte archivé le dit. */
	function accountOptionLabel(a: AccountResponse): string {
		const label = `${a.number} — ${a.name}`;
		return a.active ? label : `${label} ${i18nMsg('settings-invoicing-account-archived', '(archivé)')}`;
	}

	function save() {
		error = '';
		fieldErrors = {};
		if (!dirty) return;
		if (enabled && (gainId === null || lossId === null || transitId === null)) {
			error = i18nMsg(
				'settings-currency-accounts-required',
				'Choisissez les trois comptes : gains de change, pertes de change et conversion.',
			);
			return;
		}
		if (enabled && !company.multiCurrencyEnabled) {
			confirmOpen = true;
			return;
		}
		void send();
	}

	async function send() {
		confirmOpen = false;
		submitting = true;
		error = '';
		fieldErrors = {};
		try {
			const updated = await updateCurrencySettings({
				multiCurrencyEnabled: enabled,
				fxGainAccountId: enabled ? gainId : null,
				fxLossAccountId: enabled ? lossId : null,
				fxTransitAccountId: enabled ? transitId : null,
				version: company.version,
			});
			resetFrom(updated);
			onSaved(updated);
			notifySuccess(i18nMsg('settings-currency-saved', 'Réglage multi-devises enregistré'));
		} catch (err) {
			if (isApiError(err) && err.code === 'OPTIMISTIC_LOCK_CONFLICT') {
				try {
					const fresh = await fetchCompanyCurrent();
					resetFrom(fresh.company);
					onSaved(fresh.company);
					error = i18nMsg(
						'settings-currency-conflict',
						'Conflit de version — les données ont été rechargées, réessayez.',
					);
				} catch {
					error = i18nMsg(
						'settings-currency-conflict-reload-failed',
						'Conflit de version et rechargement impossible — rechargez la page.',
					);
				}
			} else if (isApiError(err) && err.code === 'FX_ACCOUNT_INVALID') {
				const field = readFxAccountField(err.details);
				if (field) fieldErrors = { [field]: errorMessageOf(err) };
				else error = errorMessageOf(err);
			} else {
				error = errorMessageOf(err);
			}
		} finally {
			submitting = false;
		}
	}

	const SELECT_CLASS =
		'w-full rounded-md border border-border bg-white px-3 py-2 text-sm disabled:bg-surface-alt disabled:text-text-muted';
</script>

<div class="mt-4 space-y-3 border-t border-border pt-4" data-testid="settings-currency">
	<label class="flex items-center gap-2 text-sm font-medium">
		<input
			type="checkbox"
			class="h-4 w-4 rounded border-border"
			bind:checked={enabled}
			disabled={!editable || active}
			data-testid="settings-currency-enabled"
		/>
		{i18nMsg('settings-currency-enabled-label', 'Comptabilité multi-devises')}
	</label>
	<div class="space-y-1 text-xs text-text-muted" data-testid="settings-currency-help">
		<p>
			{i18nMsg(
				'settings-currency-help-base',
				'Le franc suisse (CHF) reste la devise principale : les livres, les soldes et les rapports sont tenus en CHF. Un compte du plan comptable peut recevoir une autre devise.',
			)}
		</p>
		<p>
			{i18nMsg(
				'settings-currency-help-fiscal-years',
				'À l’activation, les exercices ouverts passent en multi-devises, et tout exercice créé ensuite l’est aussi. Les exercices clos restent mono-devise.',
			)}
		</p>
		<p>
			{i18nMsg(
				'settings-currency-help-transit',
				'Une conversion entre deux devises passe par le compte de conversion, un compte de bilan en CHF qui revient à zéro après la conversion.',
			)}
		</p>
		<p class="font-medium text-text" data-testid="settings-currency-help-definitive">
			{i18nMsg(
				'settings-currency-help-definitive',
				'L’activation est définitive : le multi-devises ne pourra plus être désactivé.',
			)}
		</p>
	</div>

	{#if enabled}
		<div class="grid max-w-xl gap-3">
			<div>
				<label class="mb-1 block text-sm font-medium" for="settings-currency-gain">
					{i18nMsg('settings-currency-gain-account', 'Compte de gains de change')}
				</label>
				<select
					id="settings-currency-gain"
					class={SELECT_CLASS}
					bind:value={gainId}
					disabled={!editable}
					aria-invalid={fieldErrors.fxGainAccountId ? 'true' : undefined}
					data-testid="settings-currency-gain"
				>
					<option value={null}>{i18nMsg('settings-invoicing-select-none', '— Sélectionner —')}</option>
					{#each gainOptions as a (a.id)}
						<option value={a.id}>{accountOptionLabel(a)}</option>
					{/each}
				</select>
				{#if fieldErrors.fxGainAccountId}
					<p class="mt-1 text-xs text-destructive" role="alert" data-testid="settings-currency-gain-error">
						{fieldErrors.fxGainAccountId}
					</p>
				{/if}
			</div>
			<div>
				<label class="mb-1 block text-sm font-medium" for="settings-currency-loss">
					{i18nMsg('settings-currency-loss-account', 'Compte de pertes de change')}
				</label>
				<select
					id="settings-currency-loss"
					class={SELECT_CLASS}
					bind:value={lossId}
					disabled={!editable}
					aria-invalid={fieldErrors.fxLossAccountId ? 'true' : undefined}
					data-testid="settings-currency-loss"
				>
					<option value={null}>{i18nMsg('settings-invoicing-select-none', '— Sélectionner —')}</option>
					{#each lossOptions as a (a.id)}
						<option value={a.id}>{accountOptionLabel(a)}</option>
					{/each}
				</select>
				{#if fieldErrors.fxLossAccountId}
					<p class="mt-1 text-xs text-destructive" role="alert" data-testid="settings-currency-loss-error">
						{fieldErrors.fxLossAccountId}
					</p>
				{/if}
			</div>
			<div>
				<label class="mb-1 block text-sm font-medium" for="settings-currency-transit">
					{i18nMsg('settings-currency-transit-account', 'Compte de conversion (compte de bilan)')}
				</label>
				<select
					id="settings-currency-transit"
					class={SELECT_CLASS}
					bind:value={transitId}
					disabled={!editable}
					aria-invalid={fieldErrors.fxTransitAccountId ? 'true' : undefined}
					data-testid="settings-currency-transit"
				>
					<option value={null}>{i18nMsg('settings-invoicing-select-none', '— Sélectionner —')}</option>
					{#each transitOptions as a (a.id)}
						<option value={a.id}>{accountOptionLabel(a)}</option>
					{/each}
				</select>
				{#if fieldErrors.fxTransitAccountId}
					<p class="mt-1 text-xs text-destructive" role="alert" data-testid="settings-currency-transit-error">
						{fieldErrors.fxTransitAccountId}
					</p>
				{/if}
			</div>
		</div>
	{/if}

	{#if error}
		<p class="text-xs text-destructive" role="alert" data-testid="settings-currency-error">{error}</p>
	{/if}

	{#if isAdmin}
		<Button size="sm" onclick={save} disabled={submitting || !dirty} data-testid="settings-currency-save">
			{submitting ? i18nMsg('common-saving', 'Enregistrement…') : i18nMsg('common-save', 'Enregistrer')}
		</Button>
	{:else}
		<p class="text-xs text-text-muted" data-testid="settings-currency-admin-only">
			{i18nMsg('settings-currency-admin-only', 'Seul un administrateur peut modifier ce réglage.')}
		</p>
	{/if}
</div>

<Dialog.Root bind:open={confirmOpen}>
	<Dialog.Content>
		<Dialog.Header>
			<Dialog.Title>{i18nMsg('settings-currency-confirm-title', 'Activer le multi-devises ?')}</Dialog.Title>
			<Dialog.Description>
				{i18nMsg(
					'settings-currency-confirm-body',
					'Le multi-devises ne pourra plus être désactivé ; les exercices ouverts passent en multi-devises.',
				)}
			</Dialog.Description>
		</Dialog.Header>
		<Dialog.Footer class="mt-4">
			<Dialog.Close>
				<Button variant="outline" type="button" autofocus data-testid="settings-currency-confirm-cancel">
					{i18nMsg('common-cancel', 'Annuler')}
				</Button>
			</Dialog.Close>
			<Button onclick={() => void send()} disabled={submitting} data-testid="settings-currency-confirm-activate">
				{i18nMsg('settings-currency-confirm-activate', 'Activer')}
			</Button>
		</Dialog.Footer>
	</Dialog.Content>
</Dialog.Root>
