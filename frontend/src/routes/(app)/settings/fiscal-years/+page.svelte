<script lang="ts">
	import { onMount } from 'svelte';
	import * as Table from '$lib/components/ui/table';
	import * as Dialog from '$lib/components/ui/dialog';
	import { Button } from '$lib/components/ui/button';
	import { Input } from '$lib/components/ui/input';
	import { Plus, Pencil, Lock } from '@lucide/svelte';

	import {
		closeFiscalYear,
		createFiscalYear,
		listFiscalYears,
		updateFiscalYear
	} from '$lib/features/fiscal-years/fiscal-years.api';
	import {
		currentYearDefaults,
		validateFiscalYearForm
	} from '$lib/features/fiscal-years/fiscal-years.helpers';
	import type {
		FiscalYearResponse,
		CreateFiscalYearRequest
	} from '$lib/features/fiscal-years/fiscal-years.types';
	import { isApiError } from '$lib/shared/utils/api-client';
	import { i18nMsg } from '$lib/shared/utils/i18n.svelte';
	import { notifyError, notifySuccess } from '$lib/shared/utils/notify';
	import { authState } from '$lib/app/stores/auth.svelte';

	function msg(key: string, fallback: string): string {
		return i18nMsg(key, fallback);
	}

	// --- State ---
	let fiscalYears = $state<FiscalYearResponse[]>([]);
	let loading = $state(true);

	// Create dialog
	let createOpen = $state(false);
	let createForm = $state<CreateFiscalYearRequest>(currentYearDefaults());
	let createSubmitting = $state(false);
	let createError = $state('');

	// Rename dialog
	let renameOpen = $state(false);
	let renameTarget = $state<FiscalYearResponse | null>(null);
	let renameName = $state('');
	let renameSubmitting = $state(false);
	let renameError = $state('');

	// Close confirmation dialog
	let closeOpen = $state(false);
	let closeTarget = $state<FiscalYearResponse | null>(null);
	let closeSubmitting = $state(false);
	let closeError = $state('');

	// Derived — RBAC frontend (le backend reste source de vérité avec 403)
	let canMutate = $derived(
		authState.currentUser?.role === 'Admin' || authState.currentUser?.role === 'Comptable'
	);

	// --- Loading ---
	async function loadFiscalYears(): Promise<void> {
		loading = true;
		try {
			fiscalYears = await listFiscalYears();
		} catch (err) {
			if (isApiError(err)) {
				notifyError(err.message);
			} else {
				notifyError(msg('error-unexpected', 'Erreur inattendue.'));
			}
		} finally {
			loading = false;
		}
	}

	onMount(() => {
		void loadFiscalYears();
	});

	// --- Create ---
	function openCreate(): void {
		createForm = currentYearDefaults();
		createError = '';
		createOpen = true;
	}

	async function submitCreate(): Promise<void> {
		const validationKey = validateFiscalYearForm(createForm);
		if (validationKey) {
			createError = msg(validationKey, validationKey);
			return;
		}
		createSubmitting = true;
		createError = '';
		try {
			await createFiscalYear({
				name: createForm.name.trim(),
				startDate: createForm.startDate,
				endDate: createForm.endDate
			});
			notifySuccess(msg('fiscal-year-created', 'Exercice créé'));
			createOpen = false;
			await loadFiscalYears();
		} catch (err) {
			// Code Review Pass 1 F12 — afficher l'erreur uniquement inline dans
			// le formulaire (qui reste ouvert pour correction). Pas de toast en
			// doublon. Le toast reste réservé aux erreurs hors form-validation
			// (cas else `error-unexpected`).
			if (isApiError(err)) {
				createError = err.message;
			} else {
				createError = msg('error-unexpected', 'Erreur inattendue.');
				notifyError(createError);
			}
		} finally {
			createSubmitting = false;
		}
	}

	// --- Rename ---
	function openRename(fy: FiscalYearResponse): void {
		renameTarget = fy;
		renameName = fy.name;
		renameError = '';
		renameOpen = true;
	}

	async function submitRename(): Promise<void> {
		if (!renameTarget) return;
		if (!renameName.trim()) {
			renameError = msg(
				'error-fiscal-year-name-empty',
				"Le nom de l'exercice est obligatoire"
			);
			return;
		}
		renameSubmitting = true;
		renameError = '';
		try {
			await updateFiscalYear(renameTarget.id, { name: renameName.trim() });
			notifySuccess(msg('fiscal-year-renamed', 'Exercice renommé'));
			renameOpen = false;
			renameTarget = null;
			await loadFiscalYears();
		} catch (err) {
			// Code Review Pass 1 F12 — erreur inline uniquement (modale reste
			// ouverte pour correction). Toast réservé au fallback inattendu.
			if (isApiError(err)) {
				renameError = err.message;
			} else {
				renameError = msg('error-unexpected', 'Erreur inattendue.');
				notifyError(renameError);
			}
		} finally {
			renameSubmitting = false;
		}
	}

	// --- Close ---
	function openClose(fy: FiscalYearResponse): void {
		closeTarget = fy;
		closeError = '';
		closeOpen = true;
	}

	async function submitClose(): Promise<void> {
		if (!closeTarget) return;
		closeSubmitting = true;
		closeError = '';
		try {
			await closeFiscalYear(closeTarget.id);
			notifySuccess(msg('fiscal-year-closed', 'Exercice clôturé'));
			closeOpen = false;
			closeTarget = null;
		} catch (err) {
			if (isApiError(err)) {
				// Story 3.7 P3-M8 — mapping context-aware ILLEGAL_STATE_TRANSITION
				// vers la clé i18n spécifique « déjà clôturé ».
				if (err.code === 'ILLEGAL_STATE_TRANSITION') {
					closeError = msg(
						'error-fiscal-year-already-closed',
						'Cet exercice est déjà clôturé'
					);
					notifyError(closeError);
					closeOpen = false;
				} else {
					closeError = err.message;
					notifyError(err.message);
				}
			} else {
				closeError = msg('error-unexpected', 'Erreur inattendue.');
				notifyError(closeError);
			}
		} finally {
			closeSubmitting = false;
		}
		// Code Review Pass 1 F11 — refresh hors du try/catch principal pour
		// que `closeSubmitting` soit toujours libéré, même si le reload échoue.
		// Le reload échoué affiche son propre toast via `loadFiscalYears`.
		await loadFiscalYears();
	}
</script>

<svelte:head>
	<title>{msg('fiscal-year-title', 'Exercices comptables')} - Kesh</title>
</svelte:head>

<div class="mb-6 flex items-center justify-between">
	<h1 class="text-2xl font-semibold text-text">
		{msg('fiscal-year-title', 'Exercices comptables')}
	</h1>
	{#if canMutate}
		<Button onclick={openCreate} data-testid="fiscal-year-create-button">
			<Plus class="mr-2 h-4 w-4" aria-hidden="true" />
			{msg('fiscal-year-create-button', 'Nouvel exercice')}
		</Button>
	{/if}
</div>

{#if loading}
	<p class="text-sm text-text-muted">{msg('loading', 'Chargement…')}</p>
{:else}
	<Table.Root data-testid="fiscal-year-table">
		<Table.Header>
			<Table.Row>
				<Table.Head>{msg('fiscal-year-name-label', 'Nom')}</Table.Head>
				<Table.Head>{msg('fiscal-year-start-date-label', 'Début')}</Table.Head>
				<Table.Head>{msg('fiscal-year-end-date-label', 'Fin')}</Table.Head>
				<Table.Head>{msg('fiscal-year-status-label', 'Statut')}</Table.Head>
				{#if canMutate}
					<Table.Head class="text-right">Actions</Table.Head>
				{/if}
			</Table.Row>
		</Table.Header>
		<Table.Body>
			{#each fiscalYears as fy (fy.id)}
				<Table.Row data-testid="fiscal-year-row-{fy.id}">
					<Table.Cell class="font-medium">{fy.name}</Table.Cell>
					<Table.Cell>{fy.startDate}</Table.Cell>
					<Table.Cell>{fy.endDate}</Table.Cell>
					<Table.Cell>
						{#if fy.status === 'Open'}
							<span class="inline-flex items-center rounded-full bg-green-100 px-2 py-0.5 text-xs font-medium text-green-700">
								{msg('fiscal-year-status-open', 'Ouvert')}
							</span>
						{:else}
							<span class="inline-flex items-center rounded-full bg-gray-100 px-2 py-0.5 text-xs font-medium text-gray-700">
								{msg('fiscal-year-status-closed', 'Clôturé')}
							</span>
						{/if}
					</Table.Cell>
					{#if canMutate}
						<Table.Cell class="text-right">
							<div class="flex items-center justify-end gap-1">
								<Button
									variant="ghost"
									size="icon-xs"
									onclick={() => openRename(fy)}
									aria-label="{msg('fiscal-year-rename-button', 'Renommer')} {fy.name}"
								>
									<Pencil class="h-4 w-4" aria-hidden="true" />
								</Button>
								{#if fy.status === 'Open'}
									<Button
										variant="ghost"
										size="icon-xs"
										onclick={() => openClose(fy)}
										aria-label="{msg('fiscal-year-close-button', 'Clôturer')} {fy.name}"
									>
										<Lock class="h-4 w-4" aria-hidden="true" />
									</Button>
								{/if}
							</div>
						</Table.Cell>
					{/if}
				</Table.Row>
			{:else}
				<Table.Row>
					<Table.Cell colspan={canMutate ? 5 : 4} class="text-center text-text-muted">
						{msg('fiscal-year-list-empty', 'Aucun exercice comptable.')}
					</Table.Cell>
				</Table.Row>
			{/each}
		</Table.Body>
	</Table.Root>
{/if}

<!-- Dialog : créer un exercice -->
<Dialog.Root bind:open={createOpen}>
	<Dialog.Content>
		<Dialog.Header>
			<Dialog.Title>{msg('fiscal-year-create-button', 'Nouvel exercice')}</Dialog.Title>
		</Dialog.Header>
		<form
			onsubmit={(e) => {
				e.preventDefault();
				void submitCreate();
			}}
			class="mt-4 flex flex-col gap-4"
		>
			<div>
				<label for="fy-create-name" class="text-sm font-medium text-text">
					{msg('fiscal-year-name-label', 'Nom')}
				</label>
				<Input id="fy-create-name" bind:value={createForm.name} autocomplete="off" />
			</div>
			<div>
				<label for="fy-create-start" class="text-sm font-medium text-text">
					{msg('fiscal-year-start-date-label', 'Début')}
				</label>
				<Input id="fy-create-start" type="date" bind:value={createForm.startDate} />
			</div>
			<div>
				<label for="fy-create-end" class="text-sm font-medium text-text">
					{msg('fiscal-year-end-date-label', 'Fin')}
				</label>
				<Input id="fy-create-end" type="date" bind:value={createForm.endDate} />
			</div>
			{#if createError}
				<p class="text-sm text-red-600" role="alert">{createError}</p>
			{/if}
			<Dialog.Footer>
				<Dialog.Close>
					<Button variant="outline" type="button">
						{msg('cancel', 'Annuler')}
					</Button>
				</Dialog.Close>
				<Button type="submit" disabled={createSubmitting}>
					{createSubmitting
						? msg('creating', 'Création…')
						: msg('create', 'Créer')}
				</Button>
			</Dialog.Footer>
		</form>
	</Dialog.Content>
</Dialog.Root>

<!-- Dialog : renommer un exercice -->
<Dialog.Root bind:open={renameOpen}>
	<Dialog.Content>
		<Dialog.Header>
			<Dialog.Title>
				{msg('fiscal-year-rename-button', 'Renommer')} « {renameTarget?.name ?? ''} »
			</Dialog.Title>
		</Dialog.Header>
		<form
			onsubmit={(e) => {
				e.preventDefault();
				void submitRename();
			}}
			class="mt-4 flex flex-col gap-4"
		>
			<div>
				<label for="fy-rename-name" class="text-sm font-medium text-text">
					{msg('fiscal-year-name-label', 'Nom')}
				</label>
				<Input id="fy-rename-name" bind:value={renameName} autocomplete="off" />
			</div>
			{#if renameTarget}
				<div class="grid grid-cols-2 gap-3 text-sm text-text-muted">
					<div>
						<dt class="font-medium">
							{msg('fiscal-year-start-date-label', 'Début')}
						</dt>
						<dd>{renameTarget.startDate}</dd>
					</div>
					<div>
						<dt class="font-medium">
							{msg('fiscal-year-end-date-label', 'Fin')}
						</dt>
						<dd>{renameTarget.endDate}</dd>
					</div>
				</div>
			{/if}
			{#if renameError}
				<p class="text-sm text-red-600" role="alert">{renameError}</p>
			{/if}
			<Dialog.Footer>
				<Dialog.Close>
					<Button variant="outline" type="button">
						{msg('cancel', 'Annuler')}
					</Button>
				</Dialog.Close>
				<Button type="submit" disabled={renameSubmitting}>
					{renameSubmitting
						? msg('saving', 'Enregistrement…')
						: msg('save', 'Enregistrer')}
				</Button>
			</Dialog.Footer>
		</form>
	</Dialog.Content>
</Dialog.Root>

<!-- Dialog : confirmer clôture -->
<Dialog.Root bind:open={closeOpen}>
	<Dialog.Content>
		<Dialog.Header>
			<Dialog.Title>
				{msg('fiscal-year-close-confirmation-title', "Clôturer l'exercice ?")}
			</Dialog.Title>
			<Dialog.Description>
				{#if closeTarget}
					{i18nMsg(
						'fiscal-year-close-confirmation-body',
						`Vous êtes sur le point de clôturer l'exercice « ${closeTarget.name} ». Cette action est irréversible : aucune écriture, facture ou paiement ne pourra plus être enregistré sur cette période. Confirmer ?`,
						{ name: closeTarget.name }
					)}
				{/if}
			</Dialog.Description>
		</Dialog.Header>
		{#if closeError}
			<p class="text-sm text-red-600" role="alert">{closeError}</p>
		{/if}
		<Dialog.Footer>
			<Dialog.Close>
				<Button variant="outline" type="button">
					{msg('cancel', 'Annuler')}
				</Button>
			</Dialog.Close>
			<Button
				type="button"
				class="bg-red-600 hover:bg-red-700"
				disabled={closeSubmitting}
				onclick={submitClose}
			>
				{closeSubmitting
					? msg('closing', 'Clôture…')
					: msg('fiscal-year-close-confirmation-action', 'Clôturer définitivement')}
			</Button>
		</Dialog.Footer>
	</Dialog.Content>
</Dialog.Root>
