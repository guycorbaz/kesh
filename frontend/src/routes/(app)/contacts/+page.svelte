<script lang="ts">
	import * as Dialog from '$lib/components/ui/dialog';
	import * as Select from '$lib/components/ui/select';
	import { Button } from '$lib/components/ui/button';
	import { Input } from '$lib/components/ui/input';
	import ContactPersonsManager from '$lib/features/contacts/ContactPersonsManager.svelte';
	import { isApiError } from '$lib/shared/utils/api-client';
	import { notifyError, notifySuccess } from '$lib/shared/utils/notify';
	import { i18nMsg } from '$lib/shared/utils/i18n.svelte';
	import { Plus, Pencil, Archive, Search } from '@lucide/svelte';
	import { onMount, untrack } from 'svelte';
	import { goto } from '$app/navigation';
	import { page } from '$app/state';

	import {
		archiveContact,
		createContact,
		listContacts,
		updateContact
	} from '$lib/features/contacts/contacts.api';
	import type {
		ContactLanguage,
		ContactResponse,
		ContactSortBy,
		ContactType,
		ListContactsQuery,
		Salutation,
		SortDirection
	} from '$lib/features/contacts/contacts.types';
	import {
		formatIdeNumber,
		normalizeIdeForApi,
		validateIdeFormat
	} from '$lib/features/contacts/contact-helpers';

	const CONTACT_TYPES: ContactType[] = ['Personne', 'Entreprise'];

	// --- State ---
	let contacts = $state<ContactResponse[]>([]);
	let total = $state(0);
	let loading = $state(false);

	// Filters + pagination (sync with URL)
	let search = $state('');
	let filterType = $state<ContactType | ''>('');
	let filterIsClient = $state<'' | 'true' | 'false'>('');
	let filterIsSupplier = $state<'' | 'true' | 'false'>('');
	let includeArchived = $state(false);
	let sortBy = $state<ContactSortBy>('Name');
	let sortDirection = $state<SortDirection>('Asc');
	let limit = $state(20);
	let offset = $state(0);

	// Debounce for search (pattern Story 3.4).
	let searchDebounceHandle: ReturnType<typeof setTimeout> | null = null;
	let effectiveSearch = $state('');

	// --- Create/Edit dialog state ---
	let formOpen = $state(false);
	let editing = $state<ContactResponse | null>(null);
	let formContactType = $state<ContactType>('Entreprise');
	let formName = $state('');
	// #213 — prénom/nom (Personne) + adresse structurée.
	let formFirstName = $state('');
	let formLastName = $state('');
	let formIsClient = $state(true);
	let formIsSupplier = $state(false);
	let formEmail = $state('');
	let formPhone = $state('');
	let formStreet = $state('');
	let formBuilding = $state('');
	let formPostalCode = $state('');
	let formCity = $state('');
	let formCountry = $state('CH');
	let formIde = $state('');
	// Story 16-3b (#151) — numéro de client ('' = non renseigné).
	let formClientNumber = $state('');
	let formPaymentTerms = $state('');
	// #245 — délai de paiement en jours ('' = non renseigné). Renseigné →
	// prime sur le texte libre (libellé auto-généré côté serveur).
	let formPaymentTermsDays = $state('');
	// Story 20-3b2 — langue de correspondance ('' = héritée instance) + civilité.
	let formLanguage = $state<ContactLanguage | ''>('');
	let formSalutation = $state<Salutation>('Neutre');
	let formSubmitting = $state(false);
	let formError = $state('');

	// --- Archive dialog state ---
	let archiveOpen = $state(false);
	let archiveTarget = $state<ContactResponse | null>(null);
	let archiveSubmitting = $state(false);

	// --- Conflict modal (pattern Story 3.3) ---
	let conflictOpen = $state(false);

	// --- URL state sync ---
	// Lecture initiale depuis l'URL dans onMount (PAS $effect, car $effect
	// créerait une dépendance réactive sur $page qui bouclera avec syncUrl).
	// Pattern identique à journal-entries/+page.svelte.
	onMount(() => {
		const params = page.url.searchParams;
		search = params.get('search') ?? '';
		effectiveSearch = search;
		filterType = (params.get('contactType') ?? '') as ContactType | '';
		filterIsClient = (params.get('isClient') ?? '') as '' | 'true' | 'false';
		filterIsSupplier = (params.get('isSupplier') ?? '') as '' | 'true' | 'false';
		includeArchived = params.get('includeArchived') === 'true';
		sortBy = (params.get('sortBy') ?? 'Name') as ContactSortBy;
		sortDirection = (params.get('sortDirection') ?? 'Asc') as SortDirection;
		limit = Math.max(1, parseInt(params.get('limit') ?? '20', 10) || 20);
		offset = Math.max(0, parseInt(params.get('offset') ?? '0', 10) || 0);

		// Cleanup debounce timer si l'utilisateur quitte avant expiration.
		return () => {
			if (searchDebounceHandle) clearTimeout(searchDebounceHandle);
		};
	});

	// Push state changes to URL (replaceState to avoid polluting history).
	function syncUrl() {
		const params = new URLSearchParams();
		if (effectiveSearch) params.set('search', effectiveSearch);
		if (filterType) params.set('contactType', filterType);
		if (filterIsClient) params.set('isClient', filterIsClient);
		if (filterIsSupplier) params.set('isSupplier', filterIsSupplier);
		if (includeArchived) params.set('includeArchived', 'true');
		if (sortBy !== 'Name') params.set('sortBy', sortBy);
		if (sortDirection !== 'Asc') params.set('sortDirection', sortDirection);
		if (limit !== 20) params.set('limit', String(limit));
		if (offset !== 0) params.set('offset', String(offset));
		const queryString = params.toString();
		const newUrl = queryString ? `/contacts?${queryString}` : '/contacts';
		untrack(() => goto(newUrl, { replaceState: true, keepFocus: true, noScroll: true }));
	}

	// --- API load ---
	async function loadContacts() {
		loading = true;
		try {
			const query: ListContactsQuery = {
				limit,
				offset,
				sortBy,
				sortDirection,
				includeArchived
			};
			if (effectiveSearch.trim()) query.search = effectiveSearch.trim();
			if (filterType) query.contactType = filterType;
			if (filterIsClient) query.isClient = filterIsClient === 'true';
			if (filterIsSupplier) query.isSupplier = filterIsSupplier === 'true';

			const result = await listContacts(query);
			contacts = result.items;
			total = result.total;
		} catch (err) {
			if (isApiError(err)) {
				notifyError(err.message);
			} else {
				notifyError(i18nMsg('error-unexpected', 'Erreur inattendue.'));
			}
		} finally {
			loading = false;
		}
	}

	// Reactively reload when filters/pagination/sort change.
	$effect(() => {
		// Track all dependencies.
		void effectiveSearch;
		void filterType;
		void filterIsClient;
		void filterIsSupplier;
		void includeArchived;
		void sortBy;
		void sortDirection;
		void limit;
		void offset;
		untrack(() => {
			syncUrl();
			loadContacts();
		});
	});

	// Debounce search input → effectiveSearch.
	function onSearchInput() {
		if (searchDebounceHandle) clearTimeout(searchDebounceHandle);
		searchDebounceHandle = setTimeout(() => {
			effectiveSearch = search;
			offset = 0;
		}, 300);
	}

	function resetFilters() {
		search = '';
		effectiveSearch = '';
		filterType = '';
		filterIsClient = '';
		filterIsSupplier = '';
		includeArchived = false;
		sortBy = 'Name';
		sortDirection = 'Asc';
		limit = 20;
		offset = 0;
	}

	function toggleSort(column: ContactSortBy) {
		if (sortBy === column) {
			sortDirection = sortDirection === 'Asc' ? 'Desc' : 'Asc';
		} else {
			sortBy = column;
			sortDirection = 'Asc';
		}
	}

	// --- Form handlers ---
	function openCreate() {
		editing = null;
		formContactType = 'Entreprise';
		formName = '';
		formFirstName = '';
		formLastName = '';
		formIsClient = true;
		formIsSupplier = false;
		formEmail = '';
		formPhone = '';
		formStreet = '';
		formBuilding = '';
		formPostalCode = '';
		formCity = '';
		formCountry = 'CH';
		formIde = '';
		formClientNumber = '';
		formPaymentTerms = '';
		formPaymentTermsDays = '';
		formLanguage = '';
		formSalutation = 'Neutre';
		formError = '';
		formOpen = true;
	}

	function openEdit(c: ContactResponse) {
		editing = c;
		formContactType = c.contactType;
		formName = c.name;
		formFirstName = c.firstName ?? '';
		formLastName = c.lastName ?? '';
		formIsClient = c.isClient;
		formIsSupplier = c.isSupplier;
		formEmail = c.email ?? '';
		formPhone = c.phone ?? '';
		formStreet = c.addressStructured?.street ?? '';
		formBuilding = c.addressStructured?.building ?? '';
		formPostalCode = c.addressStructured?.postalCode ?? '';
		formCity = c.addressStructured?.city ?? '';
		formCountry = c.addressStructured?.country || 'CH';
		formIde = formatIdeNumber(c.ideNumber);
		// ⚠️ `PUT` est un FULL-REPLACE et le même payload sert à créer et à
		// modifier : sans cette ligne, modifier un simple téléphone EFFACERAIT
		// le numéro de client, sans qu'aucune compilation ne casse.
		formClientNumber = c.clientNumber ?? '';
		formPaymentTerms = c.defaultPaymentTerms ?? '';
		formPaymentTermsDays = c.defaultPaymentTermsDays != null ? String(c.defaultPaymentTermsDays) : '';
		formLanguage = c.language ?? '';
		formSalutation = c.salutation;
		formError = '';
		formOpen = true;
	}

	// Une adresse partielle (au moins un champmais NPA/localité manquants) est refusée.
	let addressPartial = $derived(
		(formStreet.trim() || formBuilding.trim() || formPostalCode.trim() || formCity.trim()) !== '' &&
			(!formPostalCode.trim() || !formCity.trim())
	);

	let formValidation = $derived.by(() => {
		if (formContactType === 'Personne') {
			if (!formFirstName.trim() || !formLastName.trim())
				return i18nMsg('contact-error-person-name', 'Prénom et nom obligatoires pour une personne');
		} else if (!formName.trim()) {
			return i18nMsg('contact-error-name-required', 'La raison sociale est obligatoire');
		}
		if (formName.trim().length > 255)
			return i18nMsg('contact-error-name-too-long', 'Le nom doit faire au plus 255 caractères');
		if (addressPartial)
			return i18nMsg('contact-error-address-npa-city', 'NPA et localité obligatoires si une adresse est saisie');
		if (formIde.trim() && !validateIdeFormat(formIde.trim())) {
			return i18nMsg('contact-error-ide-invalid', 'Numéro IDE suisse invalide');
		}
		// #245 : vide OU entier 0..365 (miroir de la borne backend).
		if (formPaymentTermsDays.trim() !== '') {
			const d = Number(formPaymentTermsDays.trim());
			if (!Number.isInteger(d) || d < 0 || d > 365) {
				return i18nMsg(
					'contact-error-payment-terms-days-range',
					'Le délai de paiement doit être un nombre entier entre 0 et 365 jours'
				);
			}
		}
		return '';
	});

	// #245 : préséance jours > texte — quand le délai est renseigné, le texte
	// libre est désactivé (libellé auto-généré côté serveur).
	let paymentTermsDaysSet = $derived(formPaymentTermsDays.trim() !== '');

	async function submitForm() {
		formError = formValidation;
		if (formError) return;

		formSubmitting = true;
		try {
			const payload = {
				contactType: formContactType,
				name: formName.trim(),
				firstName: formContactType === 'Personne' ? formFirstName.trim() || null : null,
				lastName: formContactType === 'Personne' ? formLastName.trim() || null : null,
				isClient: formIsClient,
				isSupplier: formIsSupplier,
				email: formEmail.trim() || null,
				phone: formPhone.trim() || null,
				addressStructured: {
					street: formStreet.trim(),
					building: formBuilding.trim(),
					postalCode: formPostalCode.trim(),
					city: formCity.trim(),
					country: formCountry.trim() || 'CH'
				},
				ideNumber: normalizeIdeForApi(formIde),
				clientNumber: formClientNumber.trim() || null,
				// #245 (review BH-2) : quand le délai est renseigné, le texte libre
				// est REMPLACÉ (null) — sinon un texte legacy invisible (champ
				// désactivé) ressusciterait à l'effacement ultérieur du délai.
				defaultPaymentTerms: paymentTermsDaysSet ? null : formPaymentTerms.trim() || null,
				// #245 : null explicite (convention du form), jamais omis.
				defaultPaymentTermsDays:
					formPaymentTermsDays.trim() === '' ? null : Number(formPaymentTermsDays.trim()),
				// Story 20-3b2 : '' = héritée instance → null ; la civilité ne
				// s'applique qu'aux Personnes (l'Entreprise reçoit la formule
				// neutre côté backend quoi qu'il arrive).
				language: formLanguage === '' ? null : formLanguage,
				salutation: formContactType === 'Personne' ? formSalutation : ('Neutre' as Salutation)
			};

			if (editing) {
				await updateContact(editing.id, { ...payload, version: editing.version });
				notifySuccess(i18nMsg('contact-updated-success', 'Contact modifié'));
			} else {
				await createContact(payload);
				notifySuccess(i18nMsg('contact-created-success', 'Contact créé'));
			}

			formOpen = false;
			editing = null;
			await loadContacts();
		} catch (err) {
			if (isApiError(err)) {
				if (err.code === 'OPTIMISTIC_LOCK_CONFLICT') {
					formOpen = false;
					conflictOpen = true;
				} else if (err.code === 'IDE_ALREADY_EXISTS') {
					formError = i18nMsg('contact-error-ide-duplicate', 'Un contact avec ce numéro IDE existe déjà');
					notifyError(formError);
				} else if (err.code === 'CLIENT_NUMBER_ALREADY_EXISTS') {
					formError = i18nMsg(
						'contact-error-client-number-duplicate',
						'Un contact avec ce numéro de client existe déjà'
					);
					notifyError(formError);
				} else {
					formError = err.message;
					notifyError(err.message);
				}
			} else {
				formError = i18nMsg('error-unexpected', 'Erreur inattendue.');
				notifyError(formError);
			}
		} finally {
			formSubmitting = false;
		}
	}

	// --- Archive handlers ---
	function openArchive(c: ContactResponse) {
		archiveTarget = c;
		archiveOpen = true;
	}

	async function confirmArchive() {
		if (!archiveTarget) return;
		archiveSubmitting = true;
		try {
			await archiveContact(archiveTarget.id, { version: archiveTarget.version });
			notifySuccess(i18nMsg('contact-archived-success', 'Contact archivé'));
			archiveOpen = false;
			archiveTarget = null;
			await loadContacts();
		} catch (err) {
			if (isApiError(err)) {
				if (err.code === 'OPTIMISTIC_LOCK_CONFLICT') {
					archiveOpen = false;
					conflictOpen = true;
				} else if (err.code === 'ILLEGAL_STATE_TRANSITION') {
					archiveOpen = false;
					archiveTarget = null;
					notifyError(i18nMsg('contact-error-archived-no-modify', 'Contact archivé — modification interdite'));
					await loadContacts();
				} else {
					notifyError(err.message);
				}
			} else {
				notifyError(i18nMsg('error-unexpected', 'Erreur inattendue.'));
			}
		} finally {
			archiveSubmitting = false;
		}
	}

	// --- Conflict reload ---
	async function reloadAfterConflict() {
		conflictOpen = false;
		await loadContacts();
	}

	// --- Pagination helpers ---
	let pageStart = $derived(total === 0 ? 0 : offset + 1);
	let pageEnd = $derived(Math.min(offset + limit, total));
	let canPrev = $derived(offset > 0);
	let canNext = $derived(offset + limit < total);

	function prevPage() {
		offset = Math.max(0, offset - limit);
	}
	function nextPage() {
		if (canNext) offset = offset + limit;
	}
</script>

<svelte:head>
	<title>Contacts — Kesh</title>
</svelte:head>

<div class="container mx-auto py-6">
	<div class="flex items-center justify-between mb-6">
		<h1 class="text-2xl font-bold">{i18nMsg('contacts-page-title', "Carnet d'adresses")}</h1>
		<Button onclick={openCreate}>
			<Plus class="w-4 h-4 mr-2" />
			{i18nMsg('contact-list-new', 'Nouveau contact')}
		</Button>
	</div>

	<!-- Filtres -->
	<div class="flex flex-wrap gap-3 mb-4 items-end">
		<div class="flex-1 min-w-[200px]">
			<label for="filter-search" class="text-xs mb-1 block">
				{i18nMsg('contact-filter-search-placeholder', 'Rechercher par nom, email ou n° client…')}
			</label>
			<div class="relative">
				<Search class="absolute left-2 top-2.5 w-4 h-4 text-muted-foreground" />
				<Input
					id="filter-search"
					type="search"
					bind:value={search}
					oninput={onSearchInput}
					placeholder={i18nMsg('contact-filter-search-placeholder', 'Rechercher…')}
					class="pl-8"
				/>
			</div>
		</div>

		<div>
			<label for="filter-type" class="text-xs mb-1 block">{i18nMsg('contact-form-type', 'Type')}</label>
			<select
				id="filter-type"
				bind:value={filterType}
				class="h-9 rounded-md border border-input bg-background px-3 text-sm"
			>
				<option value="">{i18nMsg('contact-filter-type-all', 'Tous les types')}</option>
				{#each CONTACT_TYPES as t (t)}
					<option value={t}>
						{i18nMsg(
							t === 'Personne' ? 'contact-type-personne' : 'contact-type-entreprise',
							t
						)}
					</option>
				{/each}
			</select>
		</div>

		<div>
			<label for="filter-is-client" class="text-xs mb-1 block">{i18nMsg('contact-form-is-client', 'Client')}</label>
			<select
				id="filter-is-client"
				bind:value={filterIsClient}
				class="h-9 rounded-md border border-input bg-background px-3 text-sm"
			>
				<option value="">—</option>
				<option value="true">{i18nMsg('contact-form-is-client', 'Client')}</option>
				<option value="false">Non</option>
			</select>
		</div>

		<div>
			<label for="filter-is-supplier" class="text-xs mb-1 block">{i18nMsg('contact-form-is-supplier', 'Fournisseur')}</label>
			<select
				id="filter-is-supplier"
				bind:value={filterIsSupplier}
				class="h-9 rounded-md border border-input bg-background px-3 text-sm"
			>
				<option value="">—</option>
				<option value="true">{i18nMsg('contact-form-is-supplier', 'Fournisseur')}</option>
				<option value="false">Non</option>
			</select>
		</div>

		<label class="flex items-center gap-2 text-sm pb-2">
			<input type="checkbox" bind:checked={includeArchived} class="w-4 h-4" />
			{i18nMsg('contact-filter-archived', 'Inclure archivés')}
		</label>

		<Button variant="outline" size="sm" onclick={resetFilters}>Réinitialiser</Button>
	</div>

	<!-- Table -->
	<div class="border rounded-md overflow-hidden">
		<table class="w-full text-sm">
			<thead class="bg-muted/50">
				<tr>
					<th class="text-left px-3 py-2">
						<button
							type="button"
							class="font-medium hover:underline"
							onclick={() => toggleSort('Name')}
						>
							{i18nMsg('contact-col-name', 'Nom')}
							{#if sortBy === 'Name'}
								<span aria-label={sortDirection === 'Asc' ? 'tri ascendant' : 'tri descendant'}>
									{sortDirection === 'Asc' ? '↑' : '↓'}
								</span>
							{/if}
						</button>
					</th>
					<th class="text-left px-3 py-2">{i18nMsg('contact-col-type', 'Type')}</th>
					<th class="text-left px-3 py-2">{i18nMsg('contact-col-flags', 'Rôles')}</th>
					<th class="text-left px-3 py-2">{i18nMsg('contact-col-ide', 'IDE')}</th>
					<th class="text-left px-3 py-2">{i18nMsg('contact-col-email', 'Email')}</th>
					<th class="text-right px-3 py-2">{i18nMsg('contact-col-actions', 'Actions')}</th>
				</tr>
			</thead>
			<tbody>
				{#if loading}
					<tr><td colspan="6" class="text-center py-8 text-muted-foreground">…</td></tr>
				{:else if contacts.length === 0}
					<tr>
						<td colspan="6" class="text-center py-8 text-muted-foreground">
							{i18nMsg(
								'contact-empty-list',
								"Aucun contact. Créez votre premier contact avec le bouton « Nouveau contact »."
							)}
						</td>
					</tr>
				{:else}
					{#each contacts as c (c.id)}
						<tr class="border-t" class:opacity-60={!c.active}>
							<td class="px-3 py-2">{c.name}</td>
							<td class="px-3 py-2">
								{i18nMsg(
									c.contactType === 'Personne'
										? 'contact-type-personne'
										: 'contact-type-entreprise',
									c.contactType
								)}
							</td>
							<td class="px-3 py-2">
								<div class="flex gap-1">
									{#if c.isClient}
										<span
											class="inline-block px-2 py-0.5 rounded text-xs bg-green-100 text-green-800 dark:bg-green-900 dark:text-green-200"
										>
											{i18nMsg('contact-form-is-client', 'Client')}
										</span>
									{/if}
									{#if c.isSupplier}
										<span
											class="inline-block px-2 py-0.5 rounded text-xs bg-blue-100 text-blue-800 dark:bg-blue-900 dark:text-blue-200"
										>
											{i18nMsg('contact-form-is-supplier', 'Fournisseur')}
										</span>
									{/if}
								</div>
							</td>
							<td class="px-3 py-2 font-mono text-xs">{formatIdeNumber(c.ideNumber)}</td>
							<td class="px-3 py-2">{c.email ?? ''}</td>
							<td class="px-3 py-2 text-right">
								{#if c.active}
									<Button
										variant="ghost"
										size="sm"
										onclick={() => openEdit(c)}
										aria-label={i18nMsg('contact-list-edit', 'Modifier')}
									>
										<Pencil class="w-4 h-4" />
									</Button>
									<Button
										variant="ghost"
										size="sm"
										onclick={() => openArchive(c)}
										aria-label={i18nMsg('contact-list-archive', 'Archiver')}
									>
										<Archive class="w-4 h-4" />
									</Button>
								{/if}
							</td>
						</tr>
					{/each}
				{/if}
			</tbody>
		</table>
	</div>

	<!-- Pagination -->
	<div class="flex items-center justify-between mt-4 text-sm">
		<div class="text-muted-foreground">
			{pageStart}-{pageEnd} sur {total}
		</div>
		<div class="flex gap-2">
			<Button variant="outline" size="sm" disabled={!canPrev} onclick={prevPage}>Précédent</Button>
			<Button variant="outline" size="sm" disabled={!canNext} onclick={nextPage}>Suivant</Button>
		</div>
	</div>
</div>

<!-- Create/Edit dialog -->
<Dialog.Root bind:open={formOpen}>
	<!-- Story 20-3b2 : le formulaire a grandi (langue + civilité) — scroll
	     interne pour que le footer reste atteignable (E2E « Créer » hors
	     viewport sinon). -->
	<Dialog.Content class="max-h-[90vh] max-w-lg overflow-y-auto">
		<Dialog.Header>
			<Dialog.Title>
				{editing
					? i18nMsg('contact-form-edit-title', 'Modifier le contact')
					: i18nMsg('contact-form-create-title', 'Nouveau contact')}
			</Dialog.Title>
		</Dialog.Header>

		<form
			class="space-y-3"
			onsubmit={(e) => {
				e.preventDefault();
				submitForm();
			}}
		>
			<div>
				<label for="form-type">{i18nMsg('contact-form-type', 'Type')}</label>
				<select
					id="form-type"
					bind:value={formContactType}
					class="w-full h-9 rounded-md border border-input bg-background px-3 text-sm"
				>
					{#each CONTACT_TYPES as t (t)}
						<option value={t}>
							{i18nMsg(
								t === 'Personne' ? 'contact-type-personne' : 'contact-type-entreprise',
								t
							)}
						</option>
					{/each}
				</select>
			</div>

			{#if formContactType === 'Personne'}
				<div class="grid grid-cols-2 gap-3">
					<div>
						<label for="form-firstname">{i18nMsg('field-first-name', 'Prénom')} *</label>
						<Input id="form-firstname" type="text" bind:value={formFirstName} maxlength={70} />
					</div>
					<div>
						<label for="form-lastname">{i18nMsg('field-last-name', 'Nom')} *</label>
						<Input id="form-lastname" type="text" bind:value={formLastName} maxlength={70} />
					</div>
				</div>
				<div>
					<label for="form-salutation">{i18nMsg('contact-form-salutation', 'Civilité')}</label>
					<select
						id="form-salutation"
						bind:value={formSalutation}
						class="w-full h-9 rounded-md border border-input bg-background px-3 text-sm"
					>
						<option value="Neutre">{i18nMsg('contact-salutation-neutre', 'Neutre')}</option>
						<option value="Monsieur">{i18nMsg('contact-salutation-monsieur', 'Monsieur')}</option>
						<option value="Madame">{i18nMsg('contact-salutation-madame', 'Madame')}</option>
					</select>
				</div>
			{:else}
				<div>
					<label for="form-name">{i18nMsg('contact-form-name', 'Raison sociale')} *</label>
					<Input id="form-name" type="text" bind:value={formName} maxlength={255} />
				</div>
			{/if}

			<div class="flex gap-4">
				<label class="flex items-center gap-2">
					<input type="checkbox" bind:checked={formIsClient} class="w-4 h-4" />
					{i18nMsg('contact-form-is-client', 'Client')}
				</label>
				<label class="flex items-center gap-2">
					<input type="checkbox" bind:checked={formIsSupplier} class="w-4 h-4" />
					{i18nMsg('contact-form-is-supplier', 'Fournisseur')}
				</label>
			</div>

			<div>
				<label for="form-email">{i18nMsg('contact-form-email', 'Email')}</label>
				<Input id="form-email" type="email" bind:value={formEmail} maxlength={320} />
			</div>

			<div>
				<label for="form-phone">{i18nMsg('contact-form-phone', 'Téléphone')}</label>
				<Input id="form-phone" type="tel" bind:value={formPhone} maxlength={50} />
			</div>

			<div>
				<label for="form-language">
					{i18nMsg('contact-form-language', 'Langue de correspondance')}
				</label>
				<select
					id="form-language"
					bind:value={formLanguage}
					class="w-full h-9 rounded-md border border-input bg-background px-3 text-sm"
				>
					<option value="">
						{i18nMsg('contact-form-language-inherited', "Héritée (langue de l'instance)")}
					</option>
					<option value="FR">FR</option>
					<option value="DE">DE</option>
					<option value="IT">IT</option>
					<option value="EN">EN</option>
				</select>
			</div>

			<fieldset class="rounded-md border border-input p-3">
				<legend class="px-1 text-sm font-medium">{i18nMsg('field-address', 'Adresse')}</legend>
				<div class="grid grid-cols-3 gap-3">
					<div class="col-span-2">
						<label for="form-street" class="text-xs text-muted-foreground">{i18nMsg('field-street', 'Rue')}</label>
						<Input id="form-street" type="text" bind:value={formStreet} maxlength={70} />
					</div>
					<div>
						<label for="form-building" class="text-xs text-muted-foreground">{i18nMsg('field-building', 'N°')}</label>
						<Input id="form-building" type="text" bind:value={formBuilding} maxlength={16} />
					</div>
					<div>
						<label for="form-npa" class="text-xs text-muted-foreground">{i18nMsg('field-postal-code', 'NPA')}</label>
						<Input id="form-npa" type="text" bind:value={formPostalCode} maxlength={16} />
					</div>
					<div class="col-span-2">
						<label for="form-city" class="text-xs text-muted-foreground">{i18nMsg('field-city', 'Localité')}</label>
						<Input id="form-city" type="text" bind:value={formCity} maxlength={35} />
					</div>
					<div>
						<label for="form-country" class="text-xs text-muted-foreground">{i18nMsg('field-country', 'Pays')}</label>
						<Input id="form-country" type="text" bind:value={formCountry} maxlength={2} placeholder="CH" />
					</div>
				</div>
			</fieldset>

			<div>
				<label for="form-ide">{i18nMsg('contact-form-ide', 'Numéro IDE (CHE)')}</label>
				<Input id="form-ide" type="text" bind:value={formIde} placeholder="CHE-123.456.789" />
				<p class="text-xs text-muted-foreground mt-1">
					{i18nMsg('contact-form-ide-help', 'Format : CHE-123.456.789')}
				</p>
			</div>

			<div>
				<label for="form-client-number">
					{i18nMsg('contact-form-client-number', 'Numéro de client')}
				</label>
				<Input
					id="form-client-number"
					type="text"
					bind:value={formClientNumber}
					maxlength={50}
					placeholder="CLI-2026-00042"
				/>
				<p class="text-xs text-muted-foreground mt-1">
					{i18nMsg(
						'contact-form-client-number-hint',
						'Figure sur le PDF de facture, pour que votre client rapproche de son dossier fournisseur.'
					)}
				</p>
			</div>

			<div>
				<label for="form-payment-terms-days">
					{i18nMsg('contact-form-payment-terms-days', 'Délai de paiement (jours)')}
				</label>
				<Input
					id="form-payment-terms-days"
					type="text"
					inputmode="numeric"
					bind:value={formPaymentTermsDays}
					maxlength={3}
					placeholder="30"
				/>
				<p class="text-xs text-muted-foreground mt-1">
					{i18nMsg(
						'contact-form-payment-terms-days-hint',
						"L'échéance des factures sera pré-calculée et le libellé des conditions généré automatiquement."
					)}
				</p>
			</div>

			<div>
				<label for="form-payment-terms">
					{i18nMsg('contact-form-payment-terms', 'Conditions de paiement')}
				</label>
				<Input
					id="form-payment-terms"
					type="text"
					bind:value={formPaymentTerms}
					maxlength={100}
					disabled={paymentTermsDaysSet}
					placeholder={i18nMsg('contact-form-payment-terms-placeholder', 'ex: 30 jours net')}
				/>
				{#if paymentTermsDaysSet}
					<p class="text-xs text-muted-foreground mt-1">
						{i18nMsg(
							'contact-form-payment-terms-disabled-hint',
							'Libellé généré automatiquement depuis le délai de paiement.'
						)}
					</p>
				{/if}
			</div>

			{#if editing && formContactType === 'Entreprise'}
				<!-- #213 : personnes de contact (CRM) — uniquement sur une entreprise existante. -->
				<ContactPersonsManager contactId={editing.id} />
			{/if}

			{#if formError}
				<p class="text-sm text-destructive">{formError}</p>
			{/if}

			<div class="flex justify-end gap-2 pt-2">
				<Button type="button" variant="outline" onclick={() => (formOpen = false)}>
					{i18nMsg('contact-form-cancel', 'Annuler')}
				</Button>
				<Button type="submit" disabled={formSubmitting || !!formValidation}>
					{editing
						? i18nMsg('contact-form-submit-edit', 'Enregistrer')
						: i18nMsg('contact-form-submit-create', 'Créer')}
				</Button>
			</div>
		</form>
	</Dialog.Content>
</Dialog.Root>

<!-- Archive confirm dialog -->
<Dialog.Root bind:open={archiveOpen}>
	<Dialog.Content class="max-w-sm">
		<Dialog.Header>
			<Dialog.Title>
				{i18nMsg('contact-archive-confirm-title', 'Archiver le contact ?')}
			</Dialog.Title>
		</Dialog.Header>
		<p class="text-sm text-muted-foreground">
			{i18nMsg(
				'contact-archive-confirm-body',
				"Le contact ne sera plus visible dans la liste par défaut. Vous pourrez toujours le consulter en activant « Inclure archivés »."
			)}
		</p>
		<div class="flex justify-end gap-2 pt-4">
			<Button variant="outline" onclick={() => (archiveOpen = false)}>
				{i18nMsg('contact-archive-cancel', 'Annuler')}
			</Button>
			<Button variant="destructive" disabled={archiveSubmitting} onclick={confirmArchive}>
				{i18nMsg('contact-archive-confirm', 'Archiver')}
			</Button>
		</div>
	</Dialog.Content>
</Dialog.Root>

<!-- Version conflict dialog (pattern Story 3.3) -->
<Dialog.Root bind:open={conflictOpen}>
	<Dialog.Content class="max-w-sm">
		<Dialog.Header>
			<Dialog.Title>{i18nMsg('contact-conflict-title', 'Conflit de version')}</Dialog.Title>
		</Dialog.Header>
		<p class="text-sm text-muted-foreground">
			{i18nMsg(
				'contact-conflict-body',
				'Ce contact a été modifié ailleurs. Voulez-vous recharger la version actuelle ?'
			)}
		</p>
		<div class="flex justify-end pt-4">
			<Button onclick={reloadAfterConflict}>Recharger</Button>
		</div>
	</Dialog.Content>
</Dialog.Root>
