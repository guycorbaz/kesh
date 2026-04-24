<script lang="ts">
	import * as Table from '$lib/components/ui/table';
	import * as Dialog from '$lib/components/ui/dialog';
	import * as Select from '$lib/components/ui/select';
	import { Button } from '$lib/components/ui/button';
	import { Input } from '$lib/components/ui/input';
	import { apiClient, isApiError } from '$lib/shared/utils/api-client';
	import { authState } from '$lib/app/stores/auth.svelte';
	import { toast } from 'svelte-sonner';
	import { UserPlus, Pencil, KeyRound, UserX } from '@lucide/svelte';
	import type { Role, UserResponse, UserListResponse } from '$lib/shared/types/user';

	const PAGE_SIZE = 50;
	const ROLES: Role[] = ['Admin', 'Comptable', 'Consultation'];
	const MIN_PASSWORD_LENGTH = 12;

	// --- State ---
	let users = $state<UserResponse[]>([]);
	let total = $state(0);
	let offset = $state(0);
	let loading = $state(false);

	// Create dialog
	let createOpen = $state(false);
	let createUsername = $state('');
	let createPassword = $state('');
	let createConfirm = $state('');
	let createRole = $state<Role>('Comptable');
	let createSubmitting = $state(false);
	let createError = $state('');

	// Edit dialog
	let editOpen = $state(false);
	let editUser = $state<UserResponse | null>(null);
	let editRole = $state<Role>('Comptable');
	let editActive = $state(true);
	let editSubmitting = $state(false);

	// Disable dialog
	let disableOpen = $state(false);
	let disableUser = $state<UserResponse | null>(null);
	let disableSubmitting = $state(false);

	// Reset password dialog
	let resetOpen = $state(false);
	let resetUser = $state<UserResponse | null>(null);
	let resetPassword = $state('');
	let resetConfirm = $state('');
	let resetSubmitting = $state(false);
	let resetError = $state('');

	// Derived
	let currentUserId = $derived(authState.currentUser?.userId);
	let hasPrev = $derived(offset > 0);
	let hasNext = $derived(offset + PAGE_SIZE < total);

	// --- API ---
	async function loadUsers() {
		loading = true;
		try {
			const res = await apiClient.get<UserListResponse>(`/api/v1/users?limit=${PAGE_SIZE}&offset=${offset}`);
			users = res.items;
			total = res.total;
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

	function prevPage() {
		offset = Math.max(0, offset - PAGE_SIZE);
		loadUsers();
	}

	function nextPage() {
		if (!hasNext) return;
		offset = offset + PAGE_SIZE;
		loadUsers();
	}

	// --- Create ---
	function openCreate() {
		createUsername = '';
		createPassword = '';
		createConfirm = '';
		createRole = 'Comptable';
		createError = '';
		createOpen = true;
	}

	let createValidation = $derived.by(() => {
		if (!createUsername.trim()) return 'Le nom d\'utilisateur est requis.';
		if (createPassword.length < MIN_PASSWORD_LENGTH) return `Le mot de passe doit contenir au moins ${MIN_PASSWORD_LENGTH} caractères.`;
		if (createPassword !== createConfirm) return 'Les mots de passe ne correspondent pas.';
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
			await apiClient.post('/api/v1/users', {
				username: createUsername.trim(),
				password: createPassword,
				role: createRole,
			});
			toast.success(`Utilisateur « ${createUsername.trim()} » créé.`);
			createOpen = false;
			await loadUsers();
		} catch (err) {
			if (isApiError(err)) {
				if (err.code === 'RESOURCE_CONFLICT') {
					createError = 'Ce nom d\'utilisateur est déjà pris.';
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
	function openEdit(user: UserResponse) {
		editUser = user;
		editRole = user.role;
		editActive = user.active;
		editOpen = true;
	}

	async function submitEdit() {
		if (!editUser) return;
		editSubmitting = true;
		try {
			await apiClient.put(`/api/v1/users/${editUser.id}`, {
				role: editRole,
				active: editActive,
				version: editUser.version,
			});
			toast.success(`Utilisateur « ${editUser.username} » modifié.`);
			editOpen = false;
			await loadUsers();
		} catch (err) {
			if (isApiError(err)) {
				if (err.code === 'OPTIMISTIC_LOCK_CONFLICT') {
					toast.error('Les données ont été modifiées. Rechargez la page.');
				} else if (err.code === 'CANNOT_DISABLE_SELF') {
					toast.error('Vous ne pouvez pas désactiver votre propre compte.');
				} else if (err.code === 'CANNOT_DISABLE_LAST_ADMIN') {
					toast.error('Impossible de désactiver le dernier administrateur.');
				} else {
					toast.error(err.message);
				}
			} else {
				toast.error('Erreur inattendue.');
			}
		} finally {
			editSubmitting = false;
		}
	}

	// --- Disable ---
	function openDisable(user: UserResponse) {
		disableUser = user;
		disableOpen = true;
	}

	async function submitDisable() {
		if (!disableUser) return;
		disableSubmitting = true;
		try {
			await apiClient.put(`/api/v1/users/${disableUser.id}/disable`);
			toast.success(`Utilisateur « ${disableUser.username} » désactivé.`);
			disableOpen = false;
			await loadUsers();
		} catch (err) {
			if (isApiError(err)) {
				if (err.code === 'CANNOT_DISABLE_SELF') {
					toast.error('Vous ne pouvez pas désactiver votre propre compte.');
				} else if (err.code === 'CANNOT_DISABLE_LAST_ADMIN') {
					toast.error('Impossible de désactiver le dernier administrateur.');
				} else {
					toast.error(err.message);
				}
			} else {
				toast.error('Erreur inattendue.');
			}
		} finally {
			disableSubmitting = false;
		}
	}

	// --- Reset password ---
	function openReset(user: UserResponse) {
		resetUser = user;
		resetPassword = '';
		resetConfirm = '';
		resetError = '';
		resetOpen = true;
	}

	let resetValidation = $derived.by(() => {
		if (resetPassword.length < MIN_PASSWORD_LENGTH) return `Le mot de passe doit contenir au moins ${MIN_PASSWORD_LENGTH} caractères.`;
		if (resetPassword !== resetConfirm) return 'Les mots de passe ne correspondent pas.';
		return '';
	});

	async function submitReset() {
		if (!resetUser) return;
		if (resetValidation) {
			resetError = resetValidation;
			return;
		}
		resetSubmitting = true;
		resetError = '';
		try {
			await apiClient.put(`/api/v1/users/${resetUser.id}/reset-password`, {
				newPassword: resetPassword,
			});
			toast.success(`Mot de passe de « ${resetUser.username} » réinitialisé.`);
			resetOpen = false;
			await loadUsers();
		} catch (err) {
			if (isApiError(err)) {
				resetError = err.message;
			} else {
				resetError = 'Erreur inattendue.';
			}
		} finally {
			resetSubmitting = false;
		}
	}

	// --- Helpers ---
	function formatDate(iso: string): string {
		const date = new Date(iso);
		if (isNaN(date.getTime())) return '—';
		return date.toLocaleDateString('fr-CH', {
			day: '2-digit',
			month: '2-digit',
			year: 'numeric',
		});
	}

	function isCurrentUser(user: UserResponse): boolean {
		return user.id.toString() === currentUserId;
	}

	// Initial load — $effect ensures auth state is hydrated before API call
	$effect(() => {
		if (authState.isAuthenticated) {
			loadUsers();
		}
	});
</script>

<svelte:head>
	<title>Utilisateurs - Kesh</title>
</svelte:head>

<div class="flex items-center justify-between mb-6">
	<h1 class="text-2xl font-semibold text-text">Utilisateurs</h1>
	<Button onclick={openCreate}>
		<UserPlus class="mr-2 h-4 w-4" aria-hidden="true" />
		Nouvel utilisateur
	</Button>
</div>

<!-- Tableau des utilisateurs -->
{#if loading && users.length === 0}
	<p class="text-sm text-text-muted">Chargement…</p>
{:else}
	<Table.Root data-testid="user-table">
		<Table.Header>
			<Table.Row>
				<Table.Head>Nom d'utilisateur</Table.Head>
				<Table.Head>Rôle</Table.Head>
				<Table.Head>Statut</Table.Head>
				<Table.Head>Créé le</Table.Head>
				<Table.Head class="text-right">Actions</Table.Head>
			</Table.Row>
		</Table.Header>
		<Table.Body>
			{#each users as user (user.id)}
				<Table.Row class={!user.active ? 'opacity-50' : ''} data-testid="user-row-{user.username}">
					<Table.Cell>
						{user.username}
						{#if isCurrentUser(user)}
							<span class="ml-2 inline-flex items-center rounded-full bg-primary/10 px-2 py-0.5 text-xs font-medium text-primary" data-testid="current-user-badge">
								Vous
							</span>
						{/if}
					</Table.Cell>
					<Table.Cell>{user.role}</Table.Cell>
					<Table.Cell>
						{#if user.active}
							<span class="text-sm text-green-700">Actif</span>
						{:else}
							<span class="text-sm text-text-muted">Désactivé</span>
						{/if}
					</Table.Cell>
					<Table.Cell>{formatDate(user.createdAt)}</Table.Cell>
					<Table.Cell class="text-right">
						<div class="flex items-center justify-end gap-1">
							<Button variant="ghost" size="icon-xs" onclick={() => openEdit(user)} aria-label="Modifier {user.username}">
								<Pencil class="h-4 w-4" aria-hidden="true" />
							</Button>
							<Button variant="ghost" size="icon-xs" onclick={() => openReset(user)} aria-label="Réinitialiser le mot de passe de {user.username}">
								<KeyRound class="h-4 w-4" aria-hidden="true" />
							</Button>
							{#if !isCurrentUser(user) && user.active}
								<Button variant="ghost" size="icon-xs" onclick={() => openDisable(user)} aria-label="Désactiver {user.username}">
									<UserX class="h-4 w-4" aria-hidden="true" />
								</Button>
							{/if}
						</div>
					</Table.Cell>
				</Table.Row>
			{:else}
				<Table.Row>
					<Table.Cell colspan={5} class="text-center text-text-muted">Aucun utilisateur trouvé.</Table.Cell>
				</Table.Row>
			{/each}
		</Table.Body>
	</Table.Root>

	<!-- Pagination -->
	{#if total > PAGE_SIZE}
		<div class="mt-4 flex items-center justify-between text-sm text-text-muted">
			<span>{offset + 1}–{Math.min(offset + PAGE_SIZE, total)} sur {total}</span>
			<div class="flex gap-2">
				<Button variant="outline" size="sm" disabled={!hasPrev} onclick={prevPage}>Précédent</Button>
				<Button variant="outline" size="sm" disabled={!hasNext} onclick={nextPage}>Suivant</Button>
			</div>
		</div>
	{/if}
{/if}

<!-- Dialog : Créer un utilisateur -->
<Dialog.Root bind:open={createOpen}>
	<Dialog.Content>
		<Dialog.Header>
			<Dialog.Title>Nouvel utilisateur</Dialog.Title>
			<Dialog.Description>Créez un nouveau compte utilisateur.</Dialog.Description>
		</Dialog.Header>
		<form onsubmit={(e) => { e.preventDefault(); submitCreate(); }} class="flex flex-col gap-4 mt-4">
			<div>
				<label for="create-username" class="text-sm font-medium text-text">Nom d'utilisateur</label>
				<Input id="create-username" bind:value={createUsername} autocomplete="off" aria-describedby={createError ? 'create-error' : undefined} />
			</div>
			<div>
				<label for="create-password" class="text-sm font-medium text-text">Mot de passe</label>
				<Input id="create-password" type="password" bind:value={createPassword} autocomplete="new-password" aria-describedby={createError ? 'create-error' : undefined} />
			</div>
			<div>
				<label for="create-confirm" class="text-sm font-medium text-text">Confirmer le mot de passe</label>
				<Input id="create-confirm" type="password" bind:value={createConfirm} autocomplete="new-password" aria-describedby={createError ? 'create-error' : undefined} />
			</div>
			<div>
				<label for="create-role" class="text-sm font-medium text-text">Rôle</label>
				<Select.Root type="single" bind:value={createRole}>
					<Select.Trigger id="create-role" class="w-full mt-1">
						{createRole}
					</Select.Trigger>
					<Select.Content>
						{#each ROLES as role}
							<Select.Item value={role} label={role}>{role}</Select.Item>
						{/each}
					</Select.Content>
				</Select.Root>
			</div>
			{#if createError}
				<p id="create-error" class="text-sm text-red-600" role="alert">{createError}</p>
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

<!-- Dialog : Modifier un utilisateur -->
<Dialog.Root bind:open={editOpen}>
	<Dialog.Content>
		<Dialog.Header>
			<Dialog.Title>Modifier « {editUser?.username} »</Dialog.Title>
			<Dialog.Description>Modifiez le rôle ou le statut de cet utilisateur.</Dialog.Description>
		</Dialog.Header>
		<form onsubmit={(e) => { e.preventDefault(); submitEdit(); }} class="flex flex-col gap-4 mt-4">
			<div>
				<label for="edit-role" class="text-sm font-medium text-text">Rôle</label>
				<Select.Root type="single" bind:value={editRole}>
					<Select.Trigger id="edit-role" class="w-full mt-1">
						{editRole}
					</Select.Trigger>
					<Select.Content>
						{#each ROLES as role}
							<Select.Item value={role} label={role}>{role}</Select.Item>
						{/each}
					</Select.Content>
				</Select.Root>
			</div>
			<div class="flex items-center gap-2">
				<input id="edit-active" type="checkbox" bind:checked={editActive} class="h-4 w-4 rounded border-border" />
				<label for="edit-active" class="text-sm font-medium text-text">Compte actif</label>
			</div>
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

<!-- Dialog : Désactiver un utilisateur -->
<Dialog.Root bind:open={disableOpen}>
	<Dialog.Content>
		<Dialog.Header>
			<Dialog.Title>Désactiver « {disableUser?.username} » ?</Dialog.Title>
			<Dialog.Description>
				Cette action désactivera le compte. Ses sessions seront révoquées et il ne pourra plus se connecter.
			</Dialog.Description>
		</Dialog.Header>
		<Dialog.Footer class="mt-4">
			<Dialog.Close>
				<Button variant="outline" type="button" autofocus>Annuler</Button>
			</Dialog.Close>
			<Button variant="destructive" disabled={disableSubmitting} onclick={submitDisable}>
				{disableSubmitting ? 'Désactivation…' : 'Désactiver'}
			</Button>
		</Dialog.Footer>
	</Dialog.Content>
</Dialog.Root>

<!-- Dialog : Réinitialiser le mot de passe -->
<Dialog.Root bind:open={resetOpen}>
	<Dialog.Content>
		<Dialog.Header>
			<Dialog.Title>Réinitialiser le mot de passe de « {resetUser?.username} »</Dialog.Title>
			<Dialog.Description>Définissez un nouveau mot de passe pour cet utilisateur.</Dialog.Description>
		</Dialog.Header>
		<form onsubmit={(e) => { e.preventDefault(); submitReset(); }} class="flex flex-col gap-4 mt-4">
			<div>
				<label for="reset-password" class="text-sm font-medium text-text">Nouveau mot de passe</label>
				<Input id="reset-password" type="password" bind:value={resetPassword} autocomplete="new-password" aria-describedby={resetError ? 'reset-error' : undefined} />
			</div>
			<div>
				<label for="reset-confirm" class="text-sm font-medium text-text">Confirmer le mot de passe</label>
				<Input id="reset-confirm" type="password" bind:value={resetConfirm} autocomplete="new-password" aria-describedby={resetError ? 'reset-error' : undefined} />
			</div>
			{#if resetError}
				<p id="reset-error" class="text-sm text-red-600" role="alert">{resetError}</p>
			{/if}
			<Dialog.Footer>
				<Dialog.Close>
					<Button variant="outline" type="button">Annuler</Button>
				</Dialog.Close>
				<Button type="submit" disabled={resetSubmitting}>
					{resetSubmitting ? 'Réinitialisation…' : 'Réinitialiser'}
				</Button>
			</Dialog.Footer>
		</form>
	</Dialog.Content>
</Dialog.Root>
