<!--
  Story 8-5a-zero — Page de configuration des comptes bancaires.
  Story v014-1 — CRUD complet post-onboarding (création, édition complète,
  archivage, toggle archivés, affichage solde).
-->
<script lang="ts">
	import { onMount } from 'svelte';
	import { i18nMsg } from '$lib/shared/utils/i18n.svelte';
	import { isApiError } from '$lib/shared/utils/api-client';
	import { fetchAccounts } from '$lib/features/accounts/accounts.api';
	import type { AccountResponse } from '$lib/features/accounts/accounts.types';
	import { Button } from '$lib/components/ui/button';
	import { Input } from '$lib/components/ui/input';
	import { toast } from 'svelte-sonner';
	import BankAccountJournalLinkForm from '$lib/features/bank-accounts/BankAccountJournalLinkForm.svelte';
	import {
		listBankAccounts,
		createBankAccount,
		updateBankAccount,
		archiveBankAccount,
		type BankAccountSummary,
		type NewBankAccountPayload,
		type UpdateBankAccountPayload,
	} from '$lib/features/bank-accounts/bank-accounts.api';
	import { formatChfBalance } from '$lib/features/bank-accounts/format';

	let bankAccounts = $state<BankAccountSummary[]>([]);
	let accounts = $state<AccountResponse[]>([]);
	let loading = $state(true);
	let loadError = $state<string | null>(null);
	let includeArchived = $state(false);

	// UI state — modes d'édition. Un seul mode actif à la fois.
	type Mode =
		| { kind: 'none' }
		| { kind: 'create' }
		| { kind: 'edit-full'; id: number }
		| { kind: 'edit-journal'; id: number }
		| { kind: 'archive-confirm'; id: number };
	let mode = $state<Mode>({ kind: 'none' });
	let submitting = $state(false);

	// Form state — partagé create/edit-full.
	let formBankName = $state('');
	let formIban = $state('');
	let formQrIban = $state('');
	let formIsPrimary = $state(false);
	let formJournalAccountId = $state<number | null>(null);
	let formVersion = $state(0);
	let formError = $state<string | null>(null);

	async function reload() {
		const [baResult, accResult] = await Promise.allSettled([
			listBankAccounts(includeArchived),
			fetchAccounts(false),
		]);
		if (baResult.status === 'fulfilled') {
			bankAccounts = baResult.value;
			loadError = null;
		} else {
			loadError = isApiError(baResult.reason)
				? baResult.reason.message
				: String(baResult.reason);
		}
		if (accResult.status === 'fulfilled') {
			accounts = accResult.value;
		} else {
			accounts = [];
		}
	}

	onMount(async () => {
		await reload();
		loading = false;
	});

	// Filtrer accounts dropdown : classes 1 (Asset) + 2 (Liability) actifs.
	let linkableAccounts = $derived(
		accounts.filter(
			(a) => a.active && (a.accountType === 'Asset' || a.accountType === 'Liability'),
		),
	);

	function accountLabel(journalAccountId: number | null): string {
		if (journalAccountId === null) {
			return i18nMsg('bank-accounts-labels-not-configured', 'Non configuré');
		}
		const acc = accounts.find((a) => a.id === journalAccountId);
		if (!acc) return `#${journalAccountId}`;
		return `${acc.number} — ${acc.name}`;
	}

	function formatBalance(balance: number | null): string {
		if (balance === null) {
			return i18nMsg(
				'bank-accounts-labels-balance-unavailable',
				'Solde non disponible (lier au plan comptable)',
			);
		}
		return formatChfBalance(balance);
	}

	function resetForm() {
		formBankName = '';
		formIban = '';
		formQrIban = '';
		formIsPrimary = false;
		formJournalAccountId = null;
		formVersion = 0;
		formError = null;
	}

	function openCreate() {
		resetForm();
		mode = { kind: 'create' };
	}

	function openEditFull(ba: BankAccountSummary) {
		formBankName = ba.bankName;
		formIban = ba.iban;
		formQrIban = ba.qrIban ?? '';
		formIsPrimary = ba.isPrimary;
		formJournalAccountId = ba.journalAccountId;
		formVersion = ba.version;
		formError = null;
		mode = { kind: 'edit-full', id: ba.id };
	}

	function openEditJournal(id: number) {
		mode = { kind: 'edit-journal', id };
	}

	function openArchiveConfirm(id: number) {
		mode = { kind: 'archive-confirm', id };
	}

	function closeForm() {
		mode = { kind: 'none' };
		resetForm();
	}

	/**
	 * #155 — un QR-IBAN valide porte une QR-IID (positions 5–9 de l'IBAN) dans
	 * la plage 30000–31999. On valide côté client pour afficher un message
	 * actionnable (« laissez ce champ vide ») plutôt que l'erreur backend
	 * technique « QR-IID … hors plage … ». Cas le plus fréquent : l'utilisateur
	 * recopie son IBAN normal (ex. BCV, IID 00767) dans ce champ optionnel.
	 */
	function qrIbanHasValidQrIid(value: string): boolean {
		const normalized = value.replace(/\s+/g, '').toUpperCase();
		if (normalized.length < 9) return false;
		const iid = Number(normalized.slice(4, 9));
		return Number.isInteger(iid) && iid >= 30000 && iid <= 31999;
	}

	/** Retourne false (et positionne `formError`) si le QR-IBAN saisi n'en est pas un. */
	function validateQrIbanField(): boolean {
		const qrIban = formQrIban.trim();
		if (qrIban !== '' && !qrIbanHasValidQrIid(qrIban)) {
			formError = i18nMsg(
				'bank-accounts-error-qr-iban-not-qr',
				"Cet IBAN n'est pas un QR-IBAN. Si votre banque ne vous a pas fourni de QR-IBAN dédié aux QR-factures, laissez ce champ vide : votre IBAN normal suffit."
			);
			return false;
		}
		return true;
	}

	async function submitCreate(e: Event) {
		e.preventDefault();
		formError = null;
		if (!validateQrIbanField()) return;
		submitting = true;
		try {
			const payload: NewBankAccountPayload = {
				bankName: formBankName,
				iban: formIban,
				qrIban: formQrIban.trim() === '' ? null : formQrIban,
				isPrimary: formIsPrimary,
				journalAccountId: formJournalAccountId,
			};
			await createBankAccount(payload);
			toast.success(i18nMsg('bank-accounts-toast-create-success', 'Compte bancaire créé.'));
			await reload();
			closeForm();
		} catch (err) {
			formError = isApiError(err) ? err.message : String(err);
		} finally {
			submitting = false;
		}
	}

	async function submitUpdateFull(e: Event, id: number) {
		e.preventDefault();
		formError = null;
		if (!validateQrIbanField()) return;
		submitting = true;
		try {
			const payload: UpdateBankAccountPayload = {
				bankName: formBankName,
				iban: formIban,
				qrIban: formQrIban.trim() === '' ? null : formQrIban,
				isPrimary: formIsPrimary,
				journalAccountId: formJournalAccountId,
				version: formVersion,
			};
			await updateBankAccount(id, payload);
			toast.success(i18nMsg('bank-accounts-toast-update-success', 'Compte bancaire modifié.'));
			await reload();
			closeForm();
		} catch (err) {
			formError = isApiError(err) ? err.message : String(err);
		} finally {
			submitting = false;
		}
	}

	async function confirmArchive(id: number) {
		submitting = true;
		try {
			const ba = bankAccounts.find((b) => b.id === id);
			if (!ba) return;
			await archiveBankAccount(id, ba.version);
			toast.success(i18nMsg('bank-accounts-toast-archive-success', 'Compte bancaire archivé.'));
			await reload();
			closeForm();
		} catch (err) {
			toast.error(isApiError(err) ? err.message : String(err));
		} finally {
			submitting = false;
		}
	}

	function handleJournalUpdated(updated: BankAccountSummary) {
		bankAccounts = bankAccounts.map((ba) => (ba.id === updated.id ? updated : ba));
		closeForm();
	}

	async function handleToggleArchived() {
		// F18 Pass 1 code review : indicateur de chargement pendant le reload
		// pour feedback UX cohérent avec onMount.
		includeArchived = !includeArchived;
		loading = true;
		try {
			await reload();
		} finally {
			loading = false;
		}
	}
</script>

<svelte:head>
	<title>{i18nMsg('bank-accounts-labels-page-title', 'Comptes bancaires')} - Kesh</title>
</svelte:head>

<div class="flex items-center justify-between">
	<div>
		<h1 class="text-2xl font-semibold text-text" data-testid="bank-accounts-page-title">
			{i18nMsg('bank-accounts-labels-page-title', 'Comptes bancaires')}
		</h1>
		<p class="mt-2 text-sm text-text-muted">
			{i18nMsg(
				'bank-accounts-labels-page-subtitle',
				'Lier chaque compte bancaire à un compte du plan comptable (classe 1 typique : 1020 Caisse, 1030 Banque).',
			)}
		</p>
	</div>
	<div class="flex items-center gap-3">
		<Button
			variant="ghost"
			onclick={handleToggleArchived}
			data-testid="toggle-archived"
		>
			{includeArchived
				? i18nMsg('bank-accounts-actions-hide-archived', 'Masquer les archivés')
				: i18nMsg('bank-accounts-actions-show-archived', 'Afficher les archivés')}
		</Button>
		<Button onclick={openCreate} data-testid="create-bank-account-button" disabled={mode.kind === 'create'}>
			{i18nMsg('bank-accounts-actions-create', 'Nouveau compte bancaire')}
		</Button>
	</div>
</div>

<div class="mt-6">
	{#if loading}
		<p class="text-text-muted">
			{i18nMsg('bank-accounts-labels-loading', 'Chargement…')}
		</p>
	{:else if loadError}
		<p class="text-red-600" role="alert" data-testid="load-error">{loadError}</p>
	{:else}
		<!-- Formulaire création inline -->
		{#if mode.kind === 'create'}
			<form
				onsubmit={submitCreate}
				class="mb-6 rounded border border-border bg-surface-alt p-4"
				data-testid="create-bank-account-form"
			>
				<h2 class="mb-3 text-base font-semibold">
					{i18nMsg('bank-accounts-actions-create', 'Nouveau compte bancaire')}
				</h2>
				<div class="grid grid-cols-2 gap-3">
					<div>
						<label for="form-bank-name" class="block text-sm font-medium text-text mb-1">{i18nMsg('bank-accounts-labels-bank-name', 'Banque')}</label>
						<Input id="form-bank-name" bind:value={formBankName} required data-testid="form-bank-name" />
					</div>
					<div>
						<label for="form-iban" class="block text-sm font-medium text-text mb-1">{i18nMsg('bank-accounts-labels-iban', 'IBAN')}</label>
						<Input id="form-iban" bind:value={formIban} required placeholder="CH..." data-testid="form-iban" />
					</div>
					<div>
						<label for="form-qr-iban" class="block text-sm font-medium text-text mb-1">{i18nMsg('bank-accounts-labels-qr-iban', 'QR-IBAN (optionnel)')}</label>
						<Input id="form-qr-iban" bind:value={formQrIban} placeholder="CH..." data-testid="form-qr-iban" />
						<p class="mt-1 text-xs text-text-muted" data-testid="form-qr-iban-help">{i18nMsg('bank-accounts-help-qr-iban', 'À remplir uniquement si votre banque vous a fourni un QR-IBAN dédié aux QR-factures (numéro spécial avec un identifiant 30000–31999). Sinon, laissez ce champ vide : votre IBAN normal suffit pour générer des QR-factures.')}</p>
					</div>
					<div>
						<label for="form-journal-account" class="block text-sm font-medium text-text mb-1">{i18nMsg('bank-accounts-labels-journal-account-id', 'Compte comptable lié')}</label>
						<select
							id="form-journal-account"
							bind:value={formJournalAccountId}
							class="w-full rounded border border-border px-3 py-2 text-sm"
							data-testid="form-journal-account"
						>
							<option value={null}>{i18nMsg('bank-accounts-labels-not-configured', 'Non configuré')}</option>
							{#each linkableAccounts as acc}
								<option value={acc.id}>{acc.number} — {acc.name}</option>
							{/each}
						</select>
						<p class="mt-1 text-xs text-text-muted">
							{i18nMsg(
								'bank-accounts-tooltip-journal-account',
								'Lie ce compte bancaire à un compte du plan comptable (typiquement 1020 Caisse, 1030 Banque). Permet à la réconciliation automatique de créer les écritures vers le bon compte, et l\'affichage du solde sur la page d\'accueil. Note multi-comptes : si plusieurs comptes courants distincts, lier au sous-compte spécifique (1030.001 BCV CHF), pas au parent 1030.',
							)}
						</p>
					</div>
					<div class="col-span-2 flex items-center gap-2">
						<input
							id="form-is-primary"
							type="checkbox"
							bind:checked={formIsPrimary}
							data-testid="form-is-primary"
						/>
						<label for="form-is-primary" class="block text-sm font-medium text-text mb-1">{i18nMsg('bank-accounts-labels-is-primary', 'Compte principal')}</label>
					</div>
				</div>
				{#if formError}
					<p class="mt-3 text-sm text-red-600" role="alert" data-testid="form-error">{formError}</p>
				{/if}
				<div class="mt-4 flex gap-2">
					<Button type="submit" disabled={submitting} data-testid="form-submit">
						{i18nMsg('bank-accounts-actions-submit-create', 'Créer')}
					</Button>
					<Button type="button" variant="ghost" onclick={closeForm} data-testid="form-cancel">
						{i18nMsg('bank-accounts-actions-cancel', 'Annuler')}
					</Button>
				</div>
			</form>
		{/if}

		<!-- Liste -->
		{#if bankAccounts.length === 0}
			<p class="text-text-muted" data-testid="bank-accounts-empty">
				{i18nMsg('bank-accounts-labels-empty', 'Aucun compte bancaire configuré.')}
			</p>
		{:else}
			<table class="w-full text-sm" data-testid="bank-accounts-list">
				<thead>
					<tr class="border-b border-border text-left">
						<th class="py-2 pr-4 font-semibold">{i18nMsg('bank-accounts-labels-bank-name', 'Banque')}</th>
						<th class="py-2 pr-4 font-semibold">{i18nMsg('bank-accounts-labels-iban', 'IBAN')}</th>
						<th class="py-2 pr-4 font-semibold text-right">{i18nMsg('bank-accounts-labels-balance', 'Solde')}</th>
						<th class="py-2 pr-4 font-semibold">{i18nMsg('bank-accounts-labels-journal-account-id', 'Compte comptable lié')}</th>
						<th class="py-2"></th>
					</tr>
				</thead>
				<tbody>
					{#each bankAccounts as ba (ba.id)}
						<tr class="border-b border-border {ba.archived ? 'opacity-50' : ''}" data-testid="bank-account-row-{ba.id}">
							<td class="py-2 pr-4">
								{ba.bankName}
								{#if ba.isPrimary}
									<span class="ml-2 rounded bg-primary-light/20 px-1.5 py-0.5 text-xs text-primary" data-testid="primary-badge-{ba.id}">
										{i18nMsg('bank-accounts-labels-primary-badge', 'Principal')}
									</span>
								{/if}
								{#if ba.archived}
									<span class="ml-2 rounded bg-gray-200 px-1.5 py-0.5 text-xs text-gray-700" data-testid="archived-badge-{ba.id}">
										{i18nMsg('bank-accounts-labels-archived-badge', 'Archivé')}
									</span>
								{/if}
							</td>
							<td class="py-2 pr-4 font-mono text-xs" title={ba.iban}>{ba.iban}</td>
							<td class="py-2 pr-4 text-right font-mono" data-testid="balance-cell-{ba.id}">
								{formatBalance(ba.currentBalance)}
							</td>
							<td class="py-2 pr-4" data-testid="journal-account-cell-{ba.id}">
								{accountLabel(ba.journalAccountId)}
							</td>
							<td class="py-2 whitespace-nowrap">
								{#if !ba.archived && mode.kind === 'none'}
									<button
										type="button"
										class="rounded border border-border px-2 py-1 text-xs"
										onclick={() => openEditJournal(ba.id)}
										data-testid="link-button-{ba.id}"
									>
										{i18nMsg('bank-accounts-actions-link-account', 'Lier')}
									</button>
									<button
										type="button"
										class="ml-1 rounded border border-border px-2 py-1 text-xs"
										onclick={() => openEditFull(ba)}
										data-testid="edit-button-{ba.id}"
									>
										{i18nMsg('bank-accounts-actions-edit', 'Modifier')}
									</button>
									<button
										type="button"
										class="ml-1 rounded border border-border px-2 py-1 text-xs text-red-600"
										onclick={() => openArchiveConfirm(ba.id)}
										data-testid="archive-button-{ba.id}"
									>
										{i18nMsg('bank-accounts-actions-archive', 'Archiver')}
									</button>
								{/if}
							</td>
						</tr>
						<!-- Form journal_account_id inline (legacy 8-5a-zero) -->
						{#if mode.kind === 'edit-journal' && mode.id === ba.id}
							<tr>
								<td colspan="5" class="py-3">
									<BankAccountJournalLinkForm
										bankAccount={ba}
										accounts={linkableAccounts}
										onSuccess={handleJournalUpdated}
										onCancel={closeForm}
									/>
								</td>
							</tr>
						{/if}
						<!-- Form édition complète inline -->
						{#if mode.kind === 'edit-full' && mode.id === ba.id}
							<tr>
								<td colspan="5" class="py-3">
									<form
										onsubmit={(e) => submitUpdateFull(e, ba.id)}
										class="rounded border border-border bg-surface-alt p-4"
										data-testid="edit-bank-account-form"
									>
										<h2 class="mb-3 text-base font-semibold">
											{i18nMsg('bank-accounts-actions-edit', 'Modifier')}
										</h2>
										<div class="grid grid-cols-2 gap-3">
											<div>
												<label for="edit-bank-name" class="block text-sm font-medium text-text mb-1">{i18nMsg('bank-accounts-labels-bank-name', 'Banque')}</label>
												<Input id="edit-bank-name" bind:value={formBankName} required data-testid="edit-bank-name" />
											</div>
											<div>
												<label for="edit-iban" class="block text-sm font-medium text-text mb-1">{i18nMsg('bank-accounts-labels-iban', 'IBAN')}</label>
												<Input id="edit-iban" bind:value={formIban} required data-testid="edit-iban" />
											</div>
											<div>
												<label for="edit-qr-iban" class="block text-sm font-medium text-text mb-1">{i18nMsg('bank-accounts-labels-qr-iban', 'QR-IBAN (optionnel)')}</label>
												<Input id="edit-qr-iban" bind:value={formQrIban} data-testid="edit-qr-iban" />
												<p class="mt-1 text-xs text-text-muted" data-testid="edit-qr-iban-help">{i18nMsg('bank-accounts-help-qr-iban', 'À remplir uniquement si votre banque vous a fourni un QR-IBAN dédié aux QR-factures (numéro spécial avec un identifiant 30000–31999). Sinon, laissez ce champ vide : votre IBAN normal suffit pour générer des QR-factures.')}</p>
											</div>
											<div>
												<label for="edit-journal-account" class="block text-sm font-medium text-text mb-1">{i18nMsg('bank-accounts-labels-journal-account-id', 'Compte comptable lié')}</label>
												<select
													id="edit-journal-account"
													bind:value={formJournalAccountId}
													class="w-full rounded border border-border px-3 py-2 text-sm"
													data-testid="edit-journal-account"
												>
													<option value={null}>{i18nMsg('bank-accounts-labels-not-configured', 'Non configuré')}</option>
													{#each linkableAccounts as acc}
														<option value={acc.id}>{acc.number} — {acc.name}</option>
													{/each}
												</select>
											</div>
											<div class="col-span-2 flex items-center gap-2">
												<input
													id="edit-is-primary"
													type="checkbox"
													bind:checked={formIsPrimary}
													data-testid="edit-is-primary"
												/>
												<label for="edit-is-primary" class="block text-sm font-medium text-text mb-1">{i18nMsg('bank-accounts-labels-is-primary', 'Compte principal')}</label>
											</div>
										</div>
										{#if formError}
											<p class="mt-3 text-sm text-red-600" role="alert" data-testid="edit-form-error">{formError}</p>
										{/if}
										<div class="mt-4 flex gap-2">
											<Button type="submit" disabled={submitting} data-testid="edit-form-submit">
												{i18nMsg('bank-accounts-actions-submit-update', 'Enregistrer')}
											</Button>
											<Button type="button" variant="ghost" onclick={closeForm} data-testid="edit-form-cancel">
												{i18nMsg('bank-accounts-actions-cancel', 'Annuler')}
											</Button>
										</div>
									</form>
								</td>
							</tr>
						{/if}
						<!-- Confirm dialog archive inline -->
						{#if mode.kind === 'archive-confirm' && mode.id === ba.id}
							<tr>
								<td colspan="5" class="py-3">
									<div class="rounded border border-red-200 bg-red-50 p-4" data-testid="archive-confirm">
										<p class="text-sm">
											{i18nMsg(
												'bank-accounts-confirm-archive',
												'Confirmer l\'archivage de ce compte bancaire ? Cette action est irréversible v0.1.',
											)}
										</p>
										<div class="mt-3 flex gap-2">
											<Button
												onclick={() => confirmArchive(ba.id)}
												disabled={submitting}
												data-testid="archive-confirm-button"
												variant="destructive"
											>
												{i18nMsg('bank-accounts-actions-confirm-archive', 'Archiver')}
											</Button>
											<Button type="button" variant="ghost" onclick={closeForm} data-testid="archive-cancel-button">
												{i18nMsg('bank-accounts-actions-cancel', 'Annuler')}
											</Button>
										</div>
									</div>
								</td>
							</tr>
						{/if}
					{/each}
				</tbody>
			</table>
		{/if}
	{/if}
</div>
