<script lang="ts">
	import * as Dialog from '$lib/components/ui/dialog';
	import { Button } from '$lib/components/ui/button';
	import { Input } from '$lib/components/ui/input';
	import { isApiError } from '$lib/shared/utils/api-client';
	import { notifyError, notifySuccess } from '$lib/shared/utils/notify';
	import { i18nMsg } from '$lib/shared/utils/i18n.svelte';
	import { Plus, Pencil, Archive, Search } from '@lucide/svelte';
	import { onMount, untrack } from 'svelte';
	import { goto } from '$app/navigation';
	import { page } from '$app/state';

	import {
		archiveProduct,
		createProduct,
		listProducts,
		updateProduct
	} from '$lib/features/products/products.api';
	import type {
		ProductResponse,
		ProductSortBy,
		ListProductsQuery,
		SortDirection
	} from '$lib/features/products/products.types';
	import {
		classifyPriceInput,
		formatPrice,
		formatVatRate,
		normalizePriceInput
	} from '$lib/features/products/product-helpers';
	import Big from 'big.js';

	import { getVatRates } from '$lib/features/vat-rates';

	type VatOption = { value: string; labelKey: string; fallback: string };

	// Story 7.2 (KF-003) : taux TVA chargés dynamiquement depuis le backend.
	// Le `label` retourné par l'API est déjà la clé i18n (`product-vat-normal`,
	// etc.). Le fallback est dérivé par convention si la clé i18n n'est pas
	// résolue. Pendant le chargement initial, `vatOptions` est vide → le
	// `<select>` est `disabled` pour empêcher toute soumission prématurée.
	let vatOptions = $state<VatOption[]>([]);
	// Pass 2 LOW : distingue chargement en cours (`'loading'`) vs échec
	// réseau (`'error'`) vs succès (`'ready'`) pour offrir un feedback
	// précis dans le formulaire — sinon l'utilisateur voit une infinie
	// « chargement… » même quand le fetch a échoué.
	let vatLoadState = $state<'loading' | 'ready' | 'error'>('loading');

	$effect(() => {
		// Pass 1 remediation #10 : flag `cancelled` pour éviter d'écrire dans
		// `vatOptions` si le composant se démonte pendant le fetch.
		let cancelled = false;
		(async () => {
			try {
				const rates = await getVatRates();
				if (!cancelled) {
					vatOptions = rates.map((r) => ({
						value: r.rate,
						labelKey: r.label,
						fallback: `${r.rate} % — ${r.label.replace('product-vat-', '')}`,
					}));
					vatLoadState = 'ready';
				}
			} catch {
				if (!cancelled) {
					vatLoadState = 'error';
				}
			}
		})();
		return () => {
			cancelled = true;
		};
	});

	// --- State ---
	let products = $state<ProductResponse[]>([]);
	let total = $state(0);
	let loading = $state(false);

	// Filters + pagination (sync with URL)
	let search = $state('');
	let includeArchived = $state(false);
	let sortBy = $state<ProductSortBy>('Name');
	let sortDirection = $state<SortDirection>('Asc');
	let limit = $state(20);
	let offset = $state(0);

	// Debounce for search
	let searchDebounceHandle: ReturnType<typeof setTimeout> | null = null;
	let effectiveSearch = $state('');

	// --- Create/Edit dialog state ---
	let formOpen = $state(false);
	let editing = $state<ProductResponse | null>(null);
	let formName = $state('');
	let formDescription = $state('');
	let formPrice = $state('');
	let formVatRate = $state('8.10');
	let formSubmitting = $state(false);
	let formError = $state('');
	// Évite d'afficher une erreur « champ vide » avant toute interaction utilisateur.
	let formTouched = $state(false);

	// --- Archive dialog state ---
	let archiveOpen = $state(false);
	let archiveTarget = $state<ProductResponse | null>(null);
	let archiveSubmitting = $state(false);

	// --- Conflict modal ---
	let conflictOpen = $state(false);

	// Garde : empêche le `$effect` de déclencher un premier `loadProducts()`
	// avec les valeurs par défaut avant que `onMount` n'ait lu l'URL.
	let mounted = $state(false);

	// Compteur de séquence : garantit que seule la réponse la plus récente
	// de `listProducts` peut écrire dans `products` — évite les écrasements
	// par des réponses hors-ordre lors de changements rapides de filtres.
	let loadSeq = 0;

	// --- URL state sync (pattern Story 4.1 post-P1 : onMount pour lecture initiale) ---
	const VALID_SORT_BY: ProductSortBy[] = ['Name', 'UnitPrice', 'VatRate', 'CreatedAt'];
	const VALID_SORT_DIR: SortDirection[] = ['Asc', 'Desc'];

	onMount(() => {
		const params = page.url.searchParams;
		search = params.get('search') ?? '';
		effectiveSearch = search;
		includeArchived = params.get('includeArchived') === 'true';
		// Whitelist les valeurs d'URL : une URL partagée avec des paramètres invalides
		// doit dégrader gracieusement plutôt que d'envoyer des strings arbitraires au backend.
		const rawSortBy = params.get('sortBy') ?? 'Name';
		sortBy = (VALID_SORT_BY as string[]).includes(rawSortBy)
			? (rawSortBy as ProductSortBy)
			: 'Name';
		const rawSortDir = params.get('sortDirection') ?? 'Asc';
		sortDirection = (VALID_SORT_DIR as string[]).includes(rawSortDir)
			? (rawSortDir as SortDirection)
			: 'Asc';
		// Cap à 100 (= MAX_LIST_LIMIT backend) pour éviter un état bloqué 400 sur URL partagée.
		limit = Math.min(100, Math.max(1, parseInt(params.get('limit') ?? '20', 10) || 20));
		offset = Math.max(0, parseInt(params.get('offset') ?? '0', 10) || 0);
		mounted = true;

		return () => {
			if (searchDebounceHandle) clearTimeout(searchDebounceHandle);
		};
	});

	function syncUrl() {
		const params = new URLSearchParams();
		if (effectiveSearch) params.set('search', effectiveSearch);
		if (includeArchived) params.set('includeArchived', 'true');
		if (sortBy !== 'Name') params.set('sortBy', sortBy);
		if (sortDirection !== 'Asc') params.set('sortDirection', sortDirection);
		if (limit !== 20) params.set('limit', String(limit));
		if (offset !== 0) params.set('offset', String(offset));
		const qs = params.toString();
		const newUrl = qs ? `/products?${qs}` : '/products';
		// `goto` retourne une Promise ; on swallow explicitement pour éviter une
		// unhandled rejection côté navigateur lors de navigations concurrentes.
		untrack(() => {
			void goto(newUrl, { replaceState: true, keepFocus: true, noScroll: true });
		});
	}

	async function loadProducts() {
		const seq = ++loadSeq;
		loading = true;
		try {
			const query: ListProductsQuery = {
				limit,
				offset,
				sortBy,
				sortDirection,
				includeArchived
			};
			if (effectiveSearch.trim()) query.search = effectiveSearch.trim();

			const result = await listProducts(query);
			// Ignorer toute réponse rendue obsolète par un appel plus récent.
			if (seq !== loadSeq) return;
			products = result.items;
			total = result.total;
		} catch (err) {
			if (seq !== loadSeq) return;
			if (isApiError(err)) {
				notifyError(err.message);
			} else {
				notifyError(i18nMsg('error-unexpected', 'Erreur inattendue.'));
			}
		} finally {
			if (seq === loadSeq) loading = false;
		}
	}

	$effect(() => {
		void effectiveSearch;
		void includeArchived;
		void sortBy;
		void sortDirection;
		void limit;
		void offset;
		// Ne rien faire avant que `onMount` ait lu l'URL : évite un premier
		// fetch avec les valeurs par défaut suivi d'un second fetch après
		// l'init URL.
		if (!mounted) return;
		untrack(() => {
			syncUrl();
			void loadProducts();
		});
	});

	function onSearchInput() {
		if (searchDebounceHandle) clearTimeout(searchDebounceHandle);
		searchDebounceHandle = setTimeout(() => {
			// Ne réinitialiser la pagination que si le filtre change réellement,
			// sinon un clic pagination pendant la fenêtre de debounce est écrasé.
			if (effectiveSearch !== search) {
				effectiveSearch = search;
				offset = 0;
			}
		}, 300);
	}

	function resetFilters() {
		search = '';
		effectiveSearch = '';
		includeArchived = false;
		sortBy = 'Name';
		sortDirection = 'Asc';
		limit = 20;
		offset = 0;
	}

	function toggleSort(column: ProductSortBy) {
		if (sortBy === column) {
			sortDirection = sortDirection === 'Asc' ? 'Desc' : 'Asc';
		} else {
			sortBy = column;
			sortDirection = 'Asc';
		}
		// Repartir de la première page : un nouveau tri avec un offset hérité
		// retourne un sous-ensemble incohérent.
		offset = 0;
	}

	// --- Form handlers ---
	function openCreate() {
		editing = null;
		formName = '';
		formDescription = '';
		formPrice = '';
		formVatRate = vatOptions[0]?.value ?? '8.10';
		formError = '';
		formTouched = false;
		formOpen = true;
	}

	function openEdit(p: ProductResponse) {
		editing = p;
		formName = p.name;
		formDescription = p.description ?? '';
		// Préserve les 4 décimales backend pour éviter toute troncature à la
		// prochaine soumission ; on retire seulement les zéros non significatifs
		// au-delà des 2 décimales pour garder un affichage lisible.
		try {
			const fixed = new Big(p.unitPrice).toFixed(4);
			formPrice = fixed.replace(/(\.\d{2})0+$/, '$1');
		} catch {
			formPrice = p.unitPrice;
		}
		// Fallback défensif si le backend retourne un taux hors store actuel.
		formVatRate = vatOptions.some((o) => o.value === p.vatRate)
			? p.vatRate
			: (vatOptions[0]?.value ?? '8.10');
		formError = '';
		// En édition, les champs sont déjà remplis : afficher la validation dès l'ouverture.
		formTouched = true;
		formOpen = true;
	}

	let formValidation = $derived.by(() => {
		if (!formName.trim()) return i18nMsg('product-error-name-required', 'Le nom est obligatoire');
		if (formName.trim().length > 255)
			return i18nMsg('product-error-name-too-long', 'Le nom doit faire au plus 255 caractères');
		const cls = classifyPriceInput(formPrice);
		if (cls === 'empty')
			return i18nMsg('product-error-price-required', 'Le prix est obligatoire');
		if (cls === 'negative')
			return i18nMsg('product-error-price-negative', 'Le prix doit être positif ou nul');
		if (cls === 'invalid')
			return i18nMsg('product-error-price-invalid', 'Format de prix invalide');
		return '';
	});

	async function submitForm() {
		// Garde anti double-submit : la touche Entrée peut déclencher le form
		// même si le bouton est `disabled`.
		if (formSubmitting) return;
		// Pass 1 remediation #7 + Pass 2 LOW : la touche Entrée peut soumettre
		// alors que le `<select disabled>` n'est pas encore peuplé (store en
		// vol) ou que le fetch a échoué. Sans ce guard, `formVatRate` reste
		// à sa valeur initiale `'8.10'` même si la company a une liste
		// différente. Distinguer loading vs error donne un feedback précis.
		if (vatOptions.length === 0) {
			formError = vatLoadState === 'error'
				? i18nMsg(
					'product-error-vat-fetch-failed',
					'Impossible de charger les taux TVA. Vérifiez la connexion réseau et rechargez la page.'
				)
				: i18nMsg(
					'product-error-vat-loading',
					'Chargement des taux TVA en cours, veuillez patienter…'
				);
			return;
		}
		formError = formValidation;
		if (formError) return;

		formSubmitting = true;
		try {
			const payload = {
				name: formName.trim(),
				description: formDescription.trim() || null,
				// Normalise la virgule décimale (claviers mobiles suisses) en point
				// avant envoi au backend qui attend la représentation canonique.
				unitPrice: normalizePriceInput(formPrice),
				vatRate: formVatRate
			};

			if (editing) {
				await updateProduct(editing.id, { ...payload, version: editing.version });
				notifySuccess(i18nMsg('product-updated-success', 'Produit modifié'));
			} else {
				await createProduct(payload);
				notifySuccess(i18nMsg('product-created-success', 'Produit créé'));
			}

			formOpen = false;
			editing = null;
			await loadProducts();
		} catch (err) {
			if (isApiError(err)) {
				if (err.code === 'OPTIMISTIC_LOCK_CONFLICT') {
					// Libère la référence stale pour éviter un second conflit garanti
					// si l'utilisateur rouvre le formulaire sans recharger.
					editing = null;
					formOpen = false;
					conflictOpen = true;
				} else if (err.code === 'RESOURCE_CONFLICT') {
					formError = i18nMsg(
						'product-error-name-duplicate',
						'Un produit avec ce nom existe déjà'
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
	function openArchive(p: ProductResponse) {
		archiveTarget = p;
		archiveOpen = true;
	}

	async function confirmArchive() {
		if (archiveSubmitting) return;
		if (!archiveTarget) return;
		archiveSubmitting = true;
		try {
			await archiveProduct(archiveTarget.id, { version: archiveTarget.version });
			notifySuccess(i18nMsg('product-archived-success', 'Produit archivé'));
			archiveOpen = false;
			archiveTarget = null;
			await loadProducts();
		} catch (err) {
			if (isApiError(err)) {
				if (err.code === 'OPTIMISTIC_LOCK_CONFLICT') {
					archiveTarget = null;
					archiveOpen = false;
					conflictOpen = true;
				} else if (err.code === 'ILLEGAL_STATE_TRANSITION') {
					archiveOpen = false;
					archiveTarget = null;
					notifyError(err.message);
					await loadProducts();
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

	async function reloadAfterConflict() {
		conflictOpen = false;
		await loadProducts();
	}

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

<div class="container mx-auto py-6">
	<div class="flex items-center justify-between mb-6">
		<h1 class="text-2xl font-bold">
			{i18nMsg('products-page-title', 'Catalogue produits/services')}
		</h1>
		<Button onclick={openCreate}>
			<Plus class="w-4 h-4 mr-2" />
			{i18nMsg('product-list-new', 'Nouveau produit')}
		</Button>
	</div>

	<!-- Filtres -->
	<div class="flex flex-wrap gap-3 mb-4 items-end">
		<div class="flex-1 min-w-[240px]">
			<label for="filter-search" class="text-xs mb-1 block">
				{i18nMsg('product-filter-search', 'Rechercher par nom ou description…')}
			</label>
			<div class="relative">
				<Search class="absolute left-2 top-2.5 w-4 h-4 text-muted-foreground" />
				<Input
					id="filter-search"
					type="search"
					bind:value={search}
					oninput={onSearchInput}
					placeholder={i18nMsg('product-filter-search', 'Rechercher…')}
					class="pl-8"
				/>
			</div>
		</div>

		<label class="flex items-center gap-2 text-sm pb-2">
			<input type="checkbox" bind:checked={includeArchived} class="w-4 h-4" />
			{i18nMsg('product-filter-archived', 'Inclure archivés')}
		</label>

		<Button variant="outline" size="sm" onclick={resetFilters}>
			{i18nMsg('product-filter-reset', 'Réinitialiser')}
		</Button>
	</div>

	<!-- Table -->
	<div class="border rounded-md overflow-hidden">
		<table class="w-full text-sm">
			<thead class="bg-muted/50">
				<tr>
					<th class="text-left px-3 py-2">
						<button type="button" class="font-medium hover:underline" onclick={() => toggleSort('Name')}>
							{i18nMsg('product-col-name', 'Nom')}
							{#if sortBy === 'Name'}
								<span>{sortDirection === 'Asc' ? '↑' : '↓'}</span>
							{/if}
						</button>
					</th>
					<th class="text-left px-3 py-2">
						{i18nMsg('product-col-description', 'Description')}
					</th>
					<th class="text-right px-3 py-2">
						<button type="button" class="font-medium hover:underline" onclick={() => toggleSort('UnitPrice')}>
							{i18nMsg('product-col-price', 'Prix')}
							{#if sortBy === 'UnitPrice'}
								<span>{sortDirection === 'Asc' ? '↑' : '↓'}</span>
							{/if}
						</button>
					</th>
					<th class="text-right px-3 py-2">
						<button type="button" class="font-medium hover:underline" onclick={() => toggleSort('VatRate')}>
							{i18nMsg('product-col-vat', 'TVA')}
							{#if sortBy === 'VatRate'}
								<span>{sortDirection === 'Asc' ? '↑' : '↓'}</span>
							{/if}
						</button>
					</th>
					<th class="text-right px-3 py-2">{i18nMsg('product-col-actions', 'Actions')}</th>
				</tr>
			</thead>
			<tbody>
				{#if loading}
					<tr><td colspan="5" class="text-center py-8 text-muted-foreground">…</td></tr>
				{:else if products.length === 0}
					<tr>
						<td colspan="5" class="text-center py-8 text-muted-foreground">
							{i18nMsg(
								'product-empty-list',
								'Aucun produit. Créez votre premier produit avec le bouton « Nouveau produit ».'
							)}
						</td>
					</tr>
				{:else}
					{#each products as p (p.id)}
						<tr class="border-t" class:opacity-60={!p.active}>
							<td class="px-3 py-2">{p.name}</td>
							<td class="px-3 py-2 text-muted-foreground">{p.description ?? ''}</td>
							<td class="px-3 py-2 text-right font-mono">{formatPrice(p.unitPrice)}</td>
							<td class="px-3 py-2 text-right font-mono">{formatVatRate(p.vatRate)}</td>
							<td class="px-3 py-2 text-right">
								{#if p.active}
									<Button
										variant="ghost"
										size="sm"
										onclick={() => openEdit(p)}
										aria-label={i18nMsg('product-list-edit', 'Modifier')}
									>
										<Pencil class="w-4 h-4" />
									</Button>
									<Button
										variant="ghost"
										size="sm"
										onclick={() => openArchive(p)}
										aria-label={i18nMsg('product-list-archive', 'Archiver')}
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
			{pageStart}-{pageEnd} {i18nMsg('product-pagination-of', 'sur')} {total}
		</div>
		<div class="flex gap-2">
			<Button variant="outline" size="sm" disabled={!canPrev} onclick={prevPage}>
				{i18nMsg('product-pagination-prev', 'Précédent')}
			</Button>
			<Button variant="outline" size="sm" disabled={!canNext} onclick={nextPage}>
				{i18nMsg('product-pagination-next', 'Suivant')}
			</Button>
		</div>
	</div>
</div>

<!-- Create/Edit dialog -->
<Dialog.Root bind:open={formOpen}>
	<Dialog.Content class="max-w-lg">
		<Dialog.Header>
			<Dialog.Title>
				{editing
					? i18nMsg('product-form-edit-title', 'Modifier le produit')
					: i18nMsg('product-form-create-title', 'Nouveau produit')}
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
				<label for="form-name">{i18nMsg('product-form-name', 'Nom')} *</label>
				<Input
					id="form-name"
					type="text"
					bind:value={formName}
					oninput={() => (formTouched = true)}
					required
					maxlength={255}
				/>
			</div>

			<div>
				<label for="form-description">{i18nMsg('product-form-description', 'Description')}</label>
				<textarea
					id="form-description"
					bind:value={formDescription}
					maxlength={1000}
					rows="2"
					class="w-full rounded-md border border-input bg-background px-3 py-2 text-sm"
				></textarea>
			</div>

			<div>
				<label for="form-price">{i18nMsg('product-form-price', 'Prix unitaire')} *</label>
				<Input
					id="form-price"
					type="text"
					inputmode="decimal"
					bind:value={formPrice}
					oninput={() => (formTouched = true)}
					placeholder="0.00"
					required
				/>
			</div>

			<div>
				<label for="form-vat-rate">{i18nMsg('product-form-vat-rate', 'Taux TVA')} *</label>
				<select
					id="form-vat-rate"
					bind:value={formVatRate}
					disabled={vatOptions.length === 0}
					class="w-full h-9 rounded-md border border-input bg-background px-3 text-sm"
				>
					{#each vatOptions as opt (opt.value)}
						<option value={opt.value}>{i18nMsg(opt.labelKey, opt.fallback)}</option>
					{/each}
				</select>
				<p class="text-xs text-muted-foreground mt-1">
					{i18nMsg('product-form-vat-help', 'Taux suisses en vigueur depuis le 01.01.2024')}
				</p>
			</div>

			{#if formError}
				<p class="text-sm text-destructive">{formError}</p>
			{:else if formTouched && formValidation}
				<p class="text-sm text-muted-foreground">{formValidation}</p>
			{/if}

			<div class="flex justify-end gap-2 pt-2">
				<Button type="button" variant="outline" onclick={() => (formOpen = false)}>
					{i18nMsg('product-form-cancel', 'Annuler')}
				</Button>
				<Button type="submit" disabled={formSubmitting || !!formValidation}>
					{editing
						? i18nMsg('product-form-submit-edit', 'Enregistrer')
						: i18nMsg('product-form-submit-create', 'Créer')}
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
				{i18nMsg('product-archive-confirm-title', 'Archiver le produit ?')}
			</Dialog.Title>
		</Dialog.Header>
		<p class="text-sm text-muted-foreground">
			{i18nMsg(
				'product-archive-confirm-body',
				'Le produit ne sera plus visible dans la liste par défaut. Vous pourrez toujours le consulter en activant « Inclure archivés ».'
			)}
		</p>
		<div class="flex justify-end gap-2 pt-4">
			<Button variant="outline" onclick={() => (archiveOpen = false)}>
				{i18nMsg('product-archive-cancel', 'Annuler')}
			</Button>
			<Button variant="destructive" disabled={archiveSubmitting} onclick={confirmArchive}>
				{i18nMsg('product-archive-confirm', 'Archiver')}
			</Button>
		</div>
	</Dialog.Content>
</Dialog.Root>

<!-- Version conflict dialog -->
<Dialog.Root bind:open={conflictOpen}>
	<Dialog.Content class="max-w-sm">
		<Dialog.Header>
			<Dialog.Title>{i18nMsg('product-conflict-title', 'Conflit de version')}</Dialog.Title>
		</Dialog.Header>
		<p class="text-sm text-muted-foreground">
			{i18nMsg(
				'product-conflict-body',
				'Ce produit a été modifié ailleurs. Voulez-vous recharger la version actuelle ?'
			)}
		</p>
		<div class="flex justify-end gap-2 pt-4">
			<Button variant="outline" onclick={() => (conflictOpen = false)}>
				{i18nMsg('product-form-cancel', 'Annuler')}
			</Button>
			<Button onclick={reloadAfterConflict}>
				{i18nMsg('product-conflict-reload', 'Recharger')}
			</Button>
		</div>
	</Dialog.Content>
</Dialog.Root>
