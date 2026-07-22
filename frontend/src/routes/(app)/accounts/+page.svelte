<script lang="ts">
	import * as Dialog from '$lib/components/ui/dialog';
	import * as Select from '$lib/components/ui/select';
	import { Button } from '$lib/components/ui/button';
	import { Input } from '$lib/components/ui/input';
	import { isApiError } from '$lib/shared/utils/api-client';
	import { authState } from '$lib/app/stores/auth.svelte';
	import { toast } from 'svelte-sonner';
	import { Plus, Pencil, Archive, RotateCcw } from '@lucide/svelte';
	import { i18nMsg } from '$lib/shared/utils/i18n.svelte';
	import {
		fetchAccounts,
		createAccount,
		updateAccount,
		archiveAccount,
		reactivateAccount,
	} from '$lib/features/accounts/accounts.api';
	import {
		ACCOUNT_ROLES,
		accountRoleKey,
		type AccountResponse,
		type AccountRole,
		type AccountType,
	} from '$lib/features/accounts/accounts.types';

	const ACCOUNT_TYPES: AccountType[] = ['Asset', 'Liability', 'Revenue', 'Expense'];

	const TYPE_FALLBACK: Record<AccountType, string> = {
		Asset: 'Actif',
		Liability: 'Passif',
		Revenue: 'Produit',
		Expense: 'Charge',
	};

	/** Libellé traduit d'un type de compte (les clés FTL existaient déjà, inutilisées). */
	function typeLabel(t: AccountType): string {
		return i18nMsg(`account-type-${t.toLowerCase()}`, TYPE_FALLBACK[t]);
	}

	const ROLE_FALLBACK: Record<AccountRole, string> = {
		Receivable: 'Créances clients',
		DefaultRevenue: 'Produit par défaut',
		Payable: 'Dettes fournisseurs',
		VatRecoverable: 'Impôt préalable (TVA récupérable)',
		VatPayable: 'TVA due',
		VatSettlement: 'Décompte TVA',
		EquityCapital: 'Capital',
		EquityOther: 'Autres fonds propres',
		RetainedEarnings: 'Bénéfice/perte reporté',
		CurrentYearResult: "Résultat de l'exercice",
	};

	/** Libellé traduit d'un rôle ; `null` → « Aucun ». */
	function roleLabel(r: AccountRole | null): string {
		if (!r) return i18nMsg('account-role-none', 'Aucun');
		return i18nMsg(accountRoleKey(r), ROLE_FALLBACK[r]);
	}

	/** Valeur sentinelle du sélecteur pour « aucun rôle » (Select n'aime pas null). */
	const NO_ROLE = '';

	// --- State ---
	let accounts = $state<AccountResponse[]>([]);
	let loading = $state(false);
	let showArchived = $state(false);

	// Create dialog
	let createOpen = $state(false);
	let createNumber = $state('');
	let createName = $state('');
	let createType = $state<AccountType>('Asset');
	let createParentId = $state('');
	let createRole = $state<string>(NO_ROLE);
	let createPostable = $state(true);
	let createSubmitting = $state(false);
	let createError = $state('');

	// Edit dialog
	let editOpen = $state(false);
	let editAccount = $state<AccountResponse | null>(null);
	let editName = $state('');
	let editType = $state<AccountType>('Asset');
	let editRole = $state<string>(NO_ROLE);
	let editPostable = $state(true);
	let editSubmitting = $state(false);
	let editError = $state('');

	// Archive dialog
	let archiveOpen = $state(false);
	let archiveTarget = $state<AccountResponse | null>(null);
	let archiveSubmitting = $state(false);

	// Réactivation (Story 14-3a, #269) — pas de dialog : action directe idempotente.
	let reactivatingId = $state<number | null>(null);

	// --- Computed: tree structure ---
	interface TreeAccount extends AccountResponse {
		level: number;
	}

	let treeAccounts = $derived.by(() => {
		const idMap = new Map<number, AccountResponse>();
		for (const a of accounts) {
			idMap.set(a.id, a);
		}

		// Compute level for each account
		function getLevel(a: AccountResponse): number {
			let level = 0;
			let current = a;
			const visited = new Set<number>();
			while (current.parentId && idMap.has(current.parentId)) {
				if (visited.has(current.id)) break;
				visited.add(current.id);
				level++;
				current = idMap.get(current.parentId)!;
			}
			return level;
		}

		// Accounts are already sorted by number from the API
		return accounts.map((a): TreeAccount => ({
			...a,
			level: getLevel(a),
		}));
	});

	// Parent options for create dialog (only group-level accounts)
	let parentOptions = $derived(
		accounts.filter((a) => a.active).map((a) => ({ value: String(a.id), label: `${a.number} — ${a.name}` }))
	);

	// --- API ---
	async function loadAccounts() {
		loading = true;
		try {
			accounts = await fetchAccounts(showArchived);
		} catch (err) {
			if (isApiError(err)) {
				toast.error(err.message);
			} else {
				toast.error(i18nMsg('error-unexpected', 'Erreur inattendue.'));
			}
		} finally {
			loading = false;
		}
	}

	// --- Create ---
	function openCreate() {
		createNumber = '';
		createName = '';
		createType = 'Asset';
		createParentId = '';
		createRole = NO_ROLE;
		createPostable = true;
		createError = '';
		createOpen = true;
	}

	let createValidation = $derived.by(() => {
		if (!createNumber.trim()) return i18nMsg('accounts-error-number-required', 'Le numéro est requis.');
		if (!createName.trim()) return i18nMsg('accounts-error-name-required', 'Le nom est requis.');
		return '';
	});

	async function submitCreate() {
		if (createValidation) {
			createError = createValidation;
			return;
		}
		createSubmitting = true;
		createError = '';
		try {
			await createAccount({
				number: createNumber.trim(),
				name: createName.trim(),
				accountType: createType,
				parentId: createParentId !== '' ? Number(createParentId) : null,
				role: createRole === NO_ROLE ? null : (createRole as AccountRole),
				postable: createPostable,
			});
			toast.success(`${i18nMsg('accounts-created', 'Compte créé')} — ${createNumber.trim()}`);
			createOpen = false;
			await loadAccounts();
		} catch (err) {
			if (isApiError(err)) {
				if (err.code === 'RESOURCE_CONFLICT') {
					createError = i18nMsg('accounts-error-number-exists', 'Ce numéro de compte existe déjà.');
				} else {
					// ACCOUNT_ROLE_ALREADY_ASSIGNED : le backend nomme déjà le compte
					// en conflit dans `message` (+ `details`), on l'affiche tel quel.
					createError = err.message;
				}
			} else {
				createError = i18nMsg('error-unexpected', 'Erreur inattendue.');
			}
		} finally {
			createSubmitting = false;
		}
	}

	// --- Edit ---
	function openEdit(account: AccountResponse) {
		editAccount = account;
		editName = account.name;
		editType = account.accountType;
		editRole = account.role ?? NO_ROLE;
		editPostable = account.postable;
		editError = '';
		editOpen = true;
	}

	let editValidation = $derived.by(() => {
		if (!editName.trim()) return i18nMsg('accounts-error-name-required', 'Le nom est requis.');
		return '';
	});

	async function submitEdit() {
		if (!editAccount) return;
		if (editValidation) {
			editError = editValidation;
			return;
		}
		editSubmitting = true;
		editError = '';
		try {
			await updateAccount(editAccount.id, {
				name: editName.trim(),
				accountType: editType,
				// full-replace : role/postable sont obligatoires côté API.
				role: editRole === NO_ROLE ? null : (editRole as AccountRole),
				postable: editPostable,
				version: editAccount.version,
			});
			toast.success(`${i18nMsg('accounts-updated', 'Compte modifié')} — ${editAccount.number}`);
			editOpen = false;
			await loadAccounts();
		} catch (err) {
			if (isApiError(err)) {
				if (err.code === 'OPTIMISTIC_LOCK_CONFLICT') {
					editError = i18nMsg('accounts-error-stale', 'Les données ont été modifiées. Rechargez la page.');
				} else {
					editError = err.message;
				}
			} else {
				editError = i18nMsg('error-unexpected', 'Erreur inattendue.');
			}
		} finally {
			editSubmitting = false;
		}
	}

	// --- Archive ---
	function openArchive(account: AccountResponse) {
		archiveTarget = account;
		archiveOpen = true;
	}

	async function submitArchive() {
		if (!archiveTarget) return;
		archiveSubmitting = true;
		try {
			await archiveAccount(archiveTarget.id, { version: archiveTarget.version });
			toast.success(`${i18nMsg('accounts-archived', 'Compte archivé')} — ${archiveTarget.number}`);
			archiveOpen = false;
			await loadAccounts();
		} catch (err) {
			if (isApiError(err)) {
				if (err.code === 'OPTIMISTIC_LOCK_CONFLICT') {
					toast.error(i18nMsg('accounts-error-stale', 'Les données ont été modifiées. Rechargez la page.'));
				} else {
					toast.error(err.message);
				}
			} else {
				toast.error(i18nMsg('error-unexpected', 'Erreur inattendue.'));
			}
		} finally {
			archiveSubmitting = false;
		}
	}

	// --- Réactivation (Story 14-3a, #269) ---
	async function submitReactivate(account: AccountResponse) {
		reactivatingId = account.id;
		try {
			await reactivateAccount(account.id, { version: account.version });
			toast.success(`${i18nMsg('accounts-reactivated', 'Compte réactivé')} — ${account.number}`);
			await loadAccounts();
		} catch (err) {
			if (isApiError(err)) {
				if (err.code === 'OPTIMISTIC_LOCK_CONFLICT') {
					toast.error(i18nMsg('accounts-error-stale', 'Les données ont été modifiées. Rechargez la page.'));
				} else {
					// ACCOUNT_ROLE_ALREADY_ASSIGNED (rôle repris pendant l'archivage)
					// et parent archivé arrivent ici avec un message explicite.
					toast.error(err.message);
				}
			} else {
				toast.error(i18nMsg('error-unexpected', 'Erreur inattendue.'));
			}
		} finally {
			reactivatingId = null;
		}
	}

	// --- Helpers ---
	function canModify(): boolean {
		const role = authState.currentUser?.role;
		return role === 'Admin' || role === 'Comptable';
	}

	// Initial load
	$effect(() => {
		if (authState.isAuthenticated) {
			loadAccounts();
		}
	});
</script>

<svelte:head>
	<title>{i18nMsg('accounts-title', 'Plan comptable')} - Kesh</title>
</svelte:head>

<div class="flex items-center justify-between mb-6">
	<h1 class="text-2xl font-semibold text-text">{i18nMsg('accounts-title', 'Plan comptable')}</h1>
	<div class="flex items-center gap-3">
		<label class="flex items-center gap-2 text-sm text-text-muted">
			<input type="checkbox" data-testid="account-show-archived-toggle" bind:checked={showArchived} onchange={() => loadAccounts()} class="h-4 w-4 rounded border-border" />
			{i18nMsg('accounts-show-archived', 'Afficher les archivés')}
		</label>
		{#if canModify()}
			<Button onclick={openCreate} data-testid="account-create-button">
				<Plus class="mr-2 h-4 w-4" aria-hidden="true" />
				{i18nMsg('accounts-add', 'Nouveau compte')}
			</Button>
		{/if}
	</div>
</div>

<!-- Arborescence des comptes -->
{#if loading && accounts.length === 0}
	<p class="text-sm text-text-muted">{i18nMsg('common-loading', 'Chargement…')}</p>
{:else if treeAccounts.length === 0}
	<p class="text-sm text-text-muted">{i18nMsg('common-empty', 'Aucun compte trouvé.')}</p>
{:else}
	<div class="border border-border rounded-lg overflow-hidden" data-testid="account-table">
		{#each treeAccounts as account (account.id)}
			<div
				class="flex items-center justify-between px-4 py-2 border-b border-border last:border-b-0 hover:bg-surface-alt transition-colors {!account.active ? 'opacity-50' : ''}"
				style="padding-left: {16 + account.level * 24}px"
				data-testid="account-row-{account.number}"
			>
				<div class="flex items-center gap-3 min-w-0">
					<span class="font-mono text-sm text-text-muted whitespace-nowrap" data-testid="account-row-{account.number}-number">{account.number}</span>
					<span class="text-sm text-text truncate" data-testid="account-row-{account.number}-name">{account.name}</span>
					<span class="inline-flex items-center rounded-full bg-primary/10 px-2 py-0.5 text-xs font-medium text-primary whitespace-nowrap" data-testid="account-row-{account.number}-type-badge">
						{typeLabel(account.accountType)}
					</span>
					<!-- Story 14-3a : rôle explicite. Sur une ligne archivée il est
					     affiché barré + explicité, sinon l'utilisateur qui coche
					     « afficher les archivés » voit deux comptes porter le même
					     rôle sans comprendre pourquoi. -->
					{#if account.role}
						<span
							class="inline-flex items-center rounded-full px-2 py-0.5 text-xs font-medium whitespace-nowrap {account.active
								? 'bg-surface-alt text-text-muted'
								: 'bg-surface-alt text-text-muted line-through'}"
							title={account.active ? undefined : i18nMsg('account-role-archived-hint', 'Rôle inactif — ce compte est archivé')}
							data-testid="account-row-{account.number}-role-badge"
						>
							{roleLabel(account.role)}
						</span>
					{/if}
					{#if !account.postable}
						<span
							class="inline-flex items-center rounded-full bg-amber-500/10 px-2 py-0.5 text-xs font-medium text-amber-700 dark:text-amber-400 whitespace-nowrap"
							title={i18nMsg('account-postable-hint', 'Indicatif — la saisie d’écriture ne le bloque pas encore')}
							data-testid="account-row-{account.number}-postable-badge"
						>
							{i18nMsg('account-postable-no', 'Non postable')}
						</span>
					{/if}
					{#if !account.active}
						<span class="text-xs text-text-muted">({i18nMsg('account-archived-label', 'Archivé')})</span>
					{/if}
				</div>
				{#if canModify()}
					<div class="flex items-center gap-1 shrink-0">
						{#if account.active}
							<Button variant="ghost" size="icon-xs" onclick={() => openEdit(account)} aria-label="{i18nMsg('accounts-edit', 'Modifier le compte')} {account.number}" data-testid="account-row-{account.number}-edit-button">
								<Pencil class="h-4 w-4" aria-hidden="true" />
							</Button>
							<Button variant="ghost" size="icon-xs" onclick={() => openArchive(account)} aria-label="{i18nMsg('accounts-archive', 'Archiver')} {account.number}" data-testid="account-row-{account.number}-archive-button">
								<Archive class="h-4 w-4" aria-hidden="true" />
							</Button>
						{:else}
							<!-- #269 : jusqu'ici une ligne archivée n'offrait AUCUNE action. -->
							<Button
								variant="ghost"
								size="icon-xs"
								disabled={reactivatingId === account.id}
								onclick={() => submitReactivate(account)}
								aria-label={i18nMsg('accounts-reactivate-aria', 'Réactiver le compte { $number }', { number: account.number })}
								data-testid="account-row-{account.number}-reactivate-button"
							>
								<RotateCcw class="h-4 w-4" aria-hidden="true" />
							</Button>
						{/if}
					</div>
				{/if}
			</div>
		{/each}
	</div>
	<p class="mt-3 text-xs text-text-muted">{i18nMsg('accounts-count', '{ $count } comptes', { count: accounts.length })}</p>
{/if}

<!-- Dialog : Créer un compte -->
<Dialog.Root bind:open={createOpen}>
	<Dialog.Content>
		<Dialog.Header>
			<Dialog.Title>{i18nMsg('accounts-add', 'Nouveau compte')}</Dialog.Title>
			<Dialog.Description>Ajoutez un compte au plan comptable.</Dialog.Description>
		</Dialog.Header>
		<form onsubmit={(e) => { e.preventDefault(); submitCreate(); }} class="flex flex-col gap-4 mt-4">
			<div>
				<label for="create-number" class="text-sm font-medium text-text">{i18nMsg('account-field-number', 'Numéro')}</label>
				<Input id="create-number" bind:value={createNumber} placeholder="1000" autocomplete="off" />
			</div>
			<div>
				<label for="create-name" class="text-sm font-medium text-text">{i18nMsg('account-field-name', 'Nom')}</label>
				<Input id="create-name" bind:value={createName} placeholder="Caisse" autocomplete="off" />
			</div>
			<div>
				<label for="create-type" class="text-sm font-medium text-text">{i18nMsg('account-field-type', 'Type')}</label>
				<Select.Root type="single" bind:value={createType}>
					<Select.Trigger id="create-type" class="w-full mt-1">
						{typeLabel(createType)}
					</Select.Trigger>
					<Select.Content>
						{#each ACCOUNT_TYPES as t}
							<Select.Item value={t} label={typeLabel(t)}>{typeLabel(t)}</Select.Item>
						{/each}
					</Select.Content>
				</Select.Root>
			</div>
			<div>
				<label for="create-parent" class="text-sm font-medium text-text">Compte parent (optionnel)</label>
				<Select.Root type="single" bind:value={createParentId}>
					<Select.Trigger id="create-parent" class="w-full mt-1">
						{createParentId ? parentOptions.find(p => p.value === createParentId)?.label ?? '—' : 'Aucun'}
					</Select.Trigger>
					<Select.Content>
						<Select.Item value="" label="Aucun">Aucun</Select.Item>
						{#each parentOptions as opt}
							<Select.Item value={opt.value} label={opt.label}>{opt.label}</Select.Item>
						{/each}
					</Select.Content>
				</Select.Root>
			</div>
			<div>
				<label for="create-role" class="text-sm font-medium text-text">{i18nMsg('account-field-role', 'Rôle')}</label>
				<Select.Root type="single" bind:value={createRole}>
					<Select.Trigger id="create-role" class="w-full mt-1" data-testid="account-create-dialog-role">
						{roleLabel(createRole === NO_ROLE ? null : (createRole as AccountRole))}
					</Select.Trigger>
					<Select.Content>
						<Select.Item value={NO_ROLE} label={roleLabel(null)}>{roleLabel(null)}</Select.Item>
						{#each ACCOUNT_ROLES as r}
							<Select.Item value={r} label={roleLabel(r)}>{roleLabel(r)}</Select.Item>
						{/each}
					</Select.Content>
				</Select.Root>
			</div>
			<div>
				<label class="flex items-center gap-2 text-sm font-medium text-text">
					<input
						type="checkbox"
						bind:checked={createPostable}
						class="h-4 w-4 rounded border-border"
						data-testid="account-create-dialog-postable"
					/>
					{i18nMsg('account-field-postable', 'Postable')}
				</label>
				<p class="mt-1 text-xs text-text-muted">
					{i18nMsg('account-postable-hint', 'Indicatif — la saisie d’écriture ne le bloque pas encore')}
				</p>
			</div>
			{#if createError}
				<p class="text-sm text-red-600" role="alert">{createError}</p>
			{/if}
			<Dialog.Footer>
				<Dialog.Close>
					<Button variant="outline" type="button" data-testid="account-create-dialog-cancel">Annuler</Button>
				</Dialog.Close>
				<Button type="submit" disabled={createSubmitting} data-testid="account-create-dialog-submit">
					{createSubmitting ? 'Création…' : 'Créer'}
				</Button>
			</Dialog.Footer>
		</form>
	</Dialog.Content>
</Dialog.Root>

<!-- Dialog : Modifier un compte -->
<Dialog.Root bind:open={editOpen}>
	<Dialog.Content>
		<Dialog.Header>
			<Dialog.Title>Modifier le compte {editAccount?.number}</Dialog.Title>
			<Dialog.Description>Le numéro n'est pas modifiable après création.</Dialog.Description>
		</Dialog.Header>
		<form onsubmit={(e) => { e.preventDefault(); submitEdit(); }} class="flex flex-col gap-4 mt-4">
			<div>
				<label for="edit-number" class="text-sm font-medium text-text">{i18nMsg('account-field-number', 'Numéro')}</label>
				<Input id="edit-number" value={editAccount?.number ?? ''} disabled class="bg-surface-alt" />
			</div>
			<div>
				<label for="edit-name" class="text-sm font-medium text-text">{i18nMsg('account-field-name', 'Nom')}</label>
				<Input id="edit-name" bind:value={editName} autocomplete="off" />
			</div>
			<div>
				<label for="edit-type" class="text-sm font-medium text-text">{i18nMsg('account-field-type', 'Type')}</label>
				<Select.Root type="single" bind:value={editType}>
					<Select.Trigger id="edit-type" class="w-full mt-1">
						{typeLabel(editType)}
					</Select.Trigger>
					<Select.Content>
						{#each ACCOUNT_TYPES as t}
							<Select.Item value={t} label={typeLabel(t)}>{typeLabel(t)}</Select.Item>
						{/each}
					</Select.Content>
				</Select.Root>
			</div>
			<div>
				<label for="edit-role" class="text-sm font-medium text-text">{i18nMsg('account-field-role', 'Rôle')}</label>
				<Select.Root type="single" bind:value={editRole}>
					<Select.Trigger id="edit-role" class="w-full mt-1" data-testid="account-edit-dialog-role">
						{roleLabel(editRole === NO_ROLE ? null : (editRole as AccountRole))}
					</Select.Trigger>
					<Select.Content>
						<Select.Item value={NO_ROLE} label={roleLabel(null)}>{roleLabel(null)}</Select.Item>
						{#each ACCOUNT_ROLES as r}
							<Select.Item value={r} label={roleLabel(r)}>{roleLabel(r)}</Select.Item>
						{/each}
					</Select.Content>
				</Select.Root>
			</div>
			<div>
				<label class="flex items-center gap-2 text-sm font-medium text-text">
					<input
						type="checkbox"
						bind:checked={editPostable}
						class="h-4 w-4 rounded border-border"
						data-testid="account-edit-dialog-postable"
					/>
					{i18nMsg('account-field-postable', 'Postable')}
				</label>
				<p class="mt-1 text-xs text-text-muted">
					{i18nMsg('account-postable-hint', 'Indicatif — la saisie d’écriture ne le bloque pas encore')}
				</p>
			</div>
			{#if editError}
				<p class="text-sm text-red-600" role="alert">{editError}</p>
			{/if}
			<Dialog.Footer>
				<Dialog.Close>
					<Button variant="outline" type="button" data-testid="account-edit-dialog-cancel">Annuler</Button>
				</Dialog.Close>
				<Button type="submit" disabled={editSubmitting} data-testid="account-edit-dialog-submit">
					{editSubmitting ? 'Enregistrement…' : 'Enregistrer'}
				</Button>
			</Dialog.Footer>
		</form>
	</Dialog.Content>
</Dialog.Root>

<!-- Dialog : Archiver un compte -->
<Dialog.Root bind:open={archiveOpen}>
	<Dialog.Content>
		<Dialog.Header>
			<Dialog.Title>Archiver le compte {archiveTarget?.number} ?</Dialog.Title>
			<Dialog.Description>
				Le compte ne sera plus disponible dans les sélections futures, mais restera visible dans les écritures existantes et dans cette liste.
			</Dialog.Description>
		</Dialog.Header>
		<Dialog.Footer class="mt-4">
			<Dialog.Close>
				<Button variant="outline" type="button" autofocus data-testid="account-archive-dialog-cancel">Annuler</Button>
			</Dialog.Close>
			<Button variant="destructive" disabled={archiveSubmitting} onclick={submitArchive} data-testid="account-archive-dialog-confirm">
				{archiveSubmitting ? 'Archivage…' : 'Archiver'}
			</Button>
		</Dialog.Footer>
	</Dialog.Content>
</Dialog.Root>
