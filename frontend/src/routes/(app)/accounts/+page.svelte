<script lang="ts">
	import * as Dialog from '$lib/components/ui/dialog';
	import * as Select from '$lib/components/ui/select';
	import { Button } from '$lib/components/ui/button';
	import { Input } from '$lib/components/ui/input';
	import { isApiError } from '$lib/shared/utils/api-client';
	import { authState } from '$lib/app/stores/auth.svelte';
	import { toast } from 'svelte-sonner';
	import { Plus, Pencil, Archive } from '@lucide/svelte';
	import {
		fetchAccounts,
		createAccount,
		updateAccount,
		archiveAccount,
	} from '$lib/features/accounts/accounts.api';
	import type { AccountResponse, AccountType } from '$lib/features/accounts/accounts.types';

	const ACCOUNT_TYPES: AccountType[] = ['Asset', 'Liability', 'Revenue', 'Expense'];

	const TYPE_LABELS: Record<AccountType, string> = {
		Asset: 'Actif',
		Liability: 'Passif',
		Revenue: 'Produit',
		Expense: 'Charge',
	};

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
	let createSubmitting = $state(false);
	let createError = $state('');

	// Edit dialog
	let editOpen = $state(false);
	let editAccount = $state<AccountResponse | null>(null);
	let editName = $state('');
	let editType = $state<AccountType>('Asset');
	let editSubmitting = $state(false);
	let editError = $state('');

	// Archive dialog
	let archiveOpen = $state(false);
	let archiveTarget = $state<AccountResponse | null>(null);
	let archiveSubmitting = $state(false);

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
				toast.error('Erreur inattendue.');
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
		createError = '';
		createOpen = true;
	}

	let createValidation = $derived.by(() => {
		if (!createNumber.trim()) return 'Le numéro est requis.';
		if (!createName.trim()) return 'Le nom est requis.';
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
			});
			toast.success(`Compte ${createNumber.trim()} créé.`);
			createOpen = false;
			await loadAccounts();
		} catch (err) {
			if (isApiError(err)) {
				if (err.code === 'RESOURCE_CONFLICT') {
					createError = 'Ce numéro de compte existe déjà.';
				} else {
					createError = err.message;
				}
			} else {
				createError = 'Erreur inattendue.';
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
		editError = '';
		editOpen = true;
	}

	let editValidation = $derived.by(() => {
		if (!editName.trim()) return 'Le nom est requis.';
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
				version: editAccount.version,
			});
			toast.success(`Compte ${editAccount.number} modifié.`);
			editOpen = false;
			await loadAccounts();
		} catch (err) {
			if (isApiError(err)) {
				if (err.code === 'OPTIMISTIC_LOCK_CONFLICT') {
					editError = 'Les données ont été modifiées. Rechargez la page.';
				} else {
					editError = err.message;
				}
			} else {
				editError = 'Erreur inattendue.';
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
			toast.success(`Compte ${archiveTarget.number} archivé.`);
			archiveOpen = false;
			await loadAccounts();
		} catch (err) {
			if (isApiError(err)) {
				if (err.code === 'OPTIMISTIC_LOCK_CONFLICT') {
					toast.error('Les données ont été modifiées. Rechargez la page.');
				} else {
					toast.error(err.message);
				}
			} else {
				toast.error('Erreur inattendue.');
			}
		} finally {
			archiveSubmitting = false;
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
	<title>Plan comptable - Kesh</title>
</svelte:head>

<div class="flex items-center justify-between mb-6">
	<h1 class="text-2xl font-semibold text-text">Plan comptable</h1>
	<div class="flex items-center gap-3">
		<label class="flex items-center gap-2 text-sm text-text-muted">
			<input type="checkbox" bind:checked={showArchived} onchange={() => loadAccounts()} class="h-4 w-4 rounded border-border" />
			Afficher les archivés
		</label>
		{#if canModify()}
			<Button onclick={openCreate}>
				<Plus class="mr-2 h-4 w-4" aria-hidden="true" />
				Nouveau compte
			</Button>
		{/if}
	</div>
</div>

<!-- Arborescence des comptes -->
{#if loading && accounts.length === 0}
	<p class="text-sm text-text-muted">Chargement…</p>
{:else if treeAccounts.length === 0}
	<p class="text-sm text-text-muted">Aucun compte trouvé.</p>
{:else}
	<div class="border border-border rounded-lg overflow-hidden">
		{#each treeAccounts as account (account.id)}
			<div
				class="flex items-center justify-between px-4 py-2 border-b border-border last:border-b-0 hover:bg-surface-alt transition-colors {!account.active ? 'opacity-50' : ''}"
				style="padding-left: {16 + account.level * 24}px"
			>
				<div class="flex items-center gap-3 min-w-0">
					<span class="font-mono text-sm text-text-muted whitespace-nowrap">{account.number}</span>
					<span class="text-sm text-text truncate">{account.name}</span>
					<span class="inline-flex items-center rounded-full bg-primary/10 px-2 py-0.5 text-xs font-medium text-primary whitespace-nowrap">
						{TYPE_LABELS[account.accountType]}
					</span>
					{#if !account.active}
						<span class="text-xs text-text-muted">(Archivé)</span>
					{/if}
				</div>
				{#if canModify() && account.active}
					<div class="flex items-center gap-1 shrink-0">
						<Button variant="ghost" size="icon-xs" onclick={() => openEdit(account)} aria-label="Modifier {account.number}">
							<Pencil class="h-4 w-4" aria-hidden="true" />
						</Button>
						<Button variant="ghost" size="icon-xs" onclick={() => openArchive(account)} aria-label="Archiver {account.number}">
							<Archive class="h-4 w-4" aria-hidden="true" />
						</Button>
					</div>
				{/if}
			</div>
		{/each}
	</div>
	<p class="mt-3 text-xs text-text-muted">{accounts.length} comptes</p>
{/if}

<!-- Dialog : Créer un compte -->
<Dialog.Root bind:open={createOpen}>
	<Dialog.Content>
		<Dialog.Header>
			<Dialog.Title>Nouveau compte</Dialog.Title>
			<Dialog.Description>Ajoutez un compte au plan comptable.</Dialog.Description>
		</Dialog.Header>
		<form onsubmit={(e) => { e.preventDefault(); submitCreate(); }} class="flex flex-col gap-4 mt-4">
			<div>
				<label for="create-number" class="text-sm font-medium text-text">Numéro</label>
				<Input id="create-number" bind:value={createNumber} placeholder="1000" autocomplete="off" />
			</div>
			<div>
				<label for="create-name" class="text-sm font-medium text-text">Nom</label>
				<Input id="create-name" bind:value={createName} placeholder="Caisse" autocomplete="off" />
			</div>
			<div>
				<label for="create-type" class="text-sm font-medium text-text">Type</label>
				<Select.Root type="single" bind:value={createType}>
					<Select.Trigger id="create-type" class="w-full mt-1">
						{TYPE_LABELS[createType]}
					</Select.Trigger>
					<Select.Content>
						{#each ACCOUNT_TYPES as t}
							<Select.Item value={t} label={TYPE_LABELS[t]}>{TYPE_LABELS[t]}</Select.Item>
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
			{#if createError}
				<p class="text-sm text-red-600" role="alert">{createError}</p>
			{/if}
			<Dialog.Footer>
				<Dialog.Close>
					<Button variant="outline" type="button">Annuler</Button>
				</Dialog.Close>
				<Button type="submit" disabled={createSubmitting}>
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
				<label for="edit-number" class="text-sm font-medium text-text">Numéro</label>
				<Input id="edit-number" value={editAccount?.number ?? ''} disabled class="bg-surface-alt" />
			</div>
			<div>
				<label for="edit-name" class="text-sm font-medium text-text">Nom</label>
				<Input id="edit-name" bind:value={editName} autocomplete="off" />
			</div>
			<div>
				<label for="edit-type" class="text-sm font-medium text-text">Type</label>
				<Select.Root type="single" bind:value={editType}>
					<Select.Trigger id="edit-type" class="w-full mt-1">
						{TYPE_LABELS[editType]}
					</Select.Trigger>
					<Select.Content>
						{#each ACCOUNT_TYPES as t}
							<Select.Item value={t} label={TYPE_LABELS[t]}>{TYPE_LABELS[t]}</Select.Item>
						{/each}
					</Select.Content>
				</Select.Root>
			</div>
			{#if editError}
				<p class="text-sm text-red-600" role="alert">{editError}</p>
			{/if}
			<Dialog.Footer>
				<Dialog.Close>
					<Button variant="outline" type="button">Annuler</Button>
				</Dialog.Close>
				<Button type="submit" disabled={editSubmitting}>
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
				<Button variant="outline" type="button" autofocus>Annuler</Button>
			</Dialog.Close>
			<Button variant="destructive" disabled={archiveSubmitting} onclick={submitArchive}>
				{archiveSubmitting ? 'Archivage…' : 'Archiver'}
			</Button>
		</Dialog.Footer>
	</Dialog.Content>
</Dialog.Root>
