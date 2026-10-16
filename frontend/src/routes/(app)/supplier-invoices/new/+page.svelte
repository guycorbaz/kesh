<!--
  Story 30-2b (#685, AC10, AC14, AC16, C-30-2-11) — saisir une facture fournisseur.

  - `/supplier-invoices/new` : le formulaire seul, avec le scan navigateur 12.4 ; le
    bouton « Enregistrer une facture » de la liste y mène ; après enregistrement,
    l'écran va à la fiche de la facture créée.
  - `/supplier-invoices/new?document=<id>` : l'aperçu du document fixe à gauche, le
    formulaire à droite, empilés sous 1024 px ; **le document reste visible pendant
    toute la saisie** (pas de modal). Le document se lit dans la liste des documents à
    traiter (aucune lecture unitaire n'existe) : un document inconnu, d'une autre
    société ou qui n'est plus à traiter donne **le même** message — ne pas distinguer
    « inconnu » d'« autre société » est voulu.

  Fournisseur proposé (document avec QR seulement) : par IBAN, puis par nom
  (`proposeSuppliers`) ; jamais présélectionné, jamais créé d'office.

  La création est Comptable+ : pour un autre rôle, la page ne propose rien.
-->
<script lang="ts">
	import { onMount } from 'svelte';
	import { goto } from '$app/navigation';
	import { page } from '$app/state';
	import { authState } from '$lib/app/stores/auth.svelte';
	import { i18nMsg } from '$lib/shared/utils/i18n.svelte';
	import { notifySuccess } from '$lib/shared/utils/notify';
	import { isApiError } from '$lib/shared/utils/api-error';
	import { createSupplierInvoice } from '$lib/features/supplier-invoices/supplier-invoices.api';
	import type { CreateSupplierInvoiceRequest } from '$lib/features/supplier-invoices/supplier-invoices.types';
	import SupplierInvoiceForm from '$lib/features/supplier-invoices/SupplierInvoiceForm.svelte';
	import {
		proposeSuppliers,
		searchTermsFor,
		type SupplierProposal,
	} from '$lib/features/supplier-invoices/supplier-proposals';
	import {
		completeImport,
		fetchImportedSourceDocument,
		getSupplierCandidates,
		listImported,
	} from '$lib/features/imported-supplier-invoices/imported-supplier-invoices.api';
	import {
		hasQr,
		type ImportedSupplierInvoice,
	} from '$lib/features/imported-supplier-invoices/imported-supplier-invoices.types';
	import DocumentPreview from '$lib/features/imported-supplier-invoices/DocumentPreview.svelte';
	import { listContacts } from '$lib/features/contacts/contacts.api';
	import type { ContactResponse } from '$lib/features/contacts/contacts.types';
	import { fetchAccounts } from '$lib/features/accounts/accounts.api';
	import type { AccountResponse } from '$lib/features/accounts/accounts.types';
	import { listVatRates } from '$lib/features/vat-rates/vat-rates.api';
	import type { VatRateResponse } from '$lib/features/vat-rates/vat-rates.types';
	import { listProjects } from '$lib/features/projects/projects.api';
	import type { ProjectResponse } from '$lib/features/projects/projects.types';

	const canWrite = $derived(
		authState.currentUser?.role === 'Admin' || authState.currentUser?.role === 'Comptable',
	);

	/** `?document=<id>` : `null` sans paramètre ; un paramètre illisible vaut « inconnu ». */
	const documentParam = page.url.searchParams.get('document');
	const documentId: number | null =
		documentParam === null ? null : /^\d+$/.test(documentParam) ? Number(documentParam) : -1;

	let loading = $state(true);
	let loadError = $state('');
	/** Le document n'est plus à traiter (inconnu, autre société, traité ou retiré). */
	let documentGone = $state(false);
	let doc = $state<ImportedSupplierInvoice | null>(null);

	let suppliers = $state<ContactResponse[]>([]);
	let expenseAccounts = $state<AccountResponse[]>([]);
	let vatRates = $state<VatRateResponse[]>([]);
	let projects = $state<ProjectResponse[]>([]);
	let proposals = $state<SupplierProposal[]>([]);

	/** Fonction de lecture STABLE du document (une closure neuve relancerait la lecture). */
	let previewLoad = $state<(() => Promise<Blob>) | null>(null);

	/**
	 * Propositions de fournisseur (AC14) : par IBAN (serveur), puis par nom (recherche
	 * large, filtrée et classée ici). Un échec ne dit rien à l'utilisateur — ce n'est
	 * pas son action — mais se journalise : « aucune proposition » et « la recherche a
	 * échoué » doivent rester distinguables pour qui débogue.
	 */
	async function loadProposals(d: ImportedSupplierInvoice) {
		const terms = searchTermsFor(d.creditorName);
		try {
			const [byIban, byName] = await Promise.all([
				getSupplierCandidates(d.id).then((r) => r.candidates),
				terms
					? listContacts({ search: terms, isSupplier: true, limit: 20 }).then((r) => r.items)
					: Promise.resolve([] as ContactResponse[]),
			]);
			proposals = proposeSuppliers(byIban, byName, d.creditorName);
		} catch (e) {
			console.error('[propositions fournisseur] échec', e);
			proposals = [];
		}
	}

	onMount(async () => {
		if (!canWrite) {
			loading = false;
			return;
		}
		try {
			const [contactsRes, accountsRes, ratesRes, projectsRes, toComplete] = await Promise.all([
				listContacts({ isSupplier: true, limit: 200 }),
				fetchAccounts(),
				listVatRates(),
				listProjects(), // actifs uniquement (includeArchived=false)
				documentId === null ? Promise.resolve(null) : listImported('to_complete'),
			]);
			suppliers = contactsRes.items;
			expenseAccounts = accountsRes.filter(
				(a) => a.accountType === 'Expense' && a.active && a.postable,
			); // 14-3b : compte de charge posté à la validation de facture fournisseur
			vatRates = ratesRes;
			projects = projectsRes;
			if (toComplete !== null) {
				const found = toComplete.find((r) => r.id === documentId) ?? null;
				if (found === null) {
					documentGone = true;
				} else {
					doc = found;
					const id = found.id;
					previewLoad = () => fetchImportedSourceDocument(id);
					if (hasQr(found)) void loadProposals(found);
				}
			}
		} catch (err) {
			loadError =
				(isApiError(err) && err.message) ||
				i18nMsg('supplier-invoices-new-load-error', 'La page n’a pas pu être chargée.');
		} finally {
			loading = false;
		}
	});

	/** Enregistre ; une erreur remonte au formulaire, qui l'affiche sans perdre la saisie. */
	async function save(req: CreateSupplierInvoiceRequest) {
		const created = doc ? await completeImport(doc.id, req) : await createSupplierInvoice(req);
		notifySuccess(i18nMsg('imported-supplier-invoices-completed', 'Facture créée.'));
		await goto(`/supplier-invoices/${created.id}`);
	}
</script>

<svelte:head>
	<title>{i18nMsg('supplier-invoices-new-title', 'Nouvelle facture fournisseur')} — Kesh</title>
</svelte:head>

<div class="mb-6 flex flex-wrap items-center justify-between gap-3">
	<h1 class="text-2xl font-semibold">
		{i18nMsg('supplier-invoices-new-title', 'Nouvelle facture fournisseur')}
	</h1>
	<a
		class="text-sm text-primary"
		href={documentId === null ? '/supplier-invoices' : '/supplier-invoices/import'}
	>
		{documentId === null
			? i18nMsg('supplier-invoices-back-to-list', 'Revenir aux factures fournisseurs')
			: i18nMsg('supplier-invoices-back-to-documents', 'Revenir aux documents à traiter')}
	</a>
</div>

{#if !canWrite}
	<p class="text-sm text-text-muted" data-testid="supplier-invoice-new-forbidden">
		{i18nMsg(
			'supplier-invoices-new-forbidden',
			'Votre rôle ne permet pas de saisir une facture fournisseur.',
		)}
	</p>
{:else if loading}
	<p class="text-sm text-text-muted" data-testid="supplier-invoice-new-loading">
		{i18nMsg('common-loading', 'Chargement…')}
	</p>
{:else if loadError}
	<p class="text-sm text-destructive" role="alert" data-testid="supplier-invoice-new-error">
		{loadError}
	</p>
{:else if documentGone}
	<div class="space-y-2" data-testid="supplier-invoice-new-doc-gone">
		<p class="text-sm">
			{i18nMsg('supplier-invoices-doc-not-pending', 'Ce document n’est plus à traiter.')}
		</p>
		<a class="text-sm text-primary" href="/supplier-invoices/import">
			{i18nMsg('supplier-invoices-back-to-documents', 'Revenir aux documents à traiter')}
		</a>
	</div>
{:else if doc && previewLoad}
	<div class="grid gap-6 lg:grid-cols-2">
		<div
			class="lg:sticky lg:top-4 lg:max-h-[calc(100vh-2rem)] lg:self-start lg:overflow-auto"
			data-testid="supplier-invoice-new-preview"
		>
			<p class="mb-2 text-sm font-medium" data-testid="supplier-invoice-new-file-name">
				{doc.originalFilename}
			</p>
			<DocumentPreview load={previewLoad} fileName={doc.originalFilename} />
		</div>
		<div>
			<SupplierInvoiceForm
				bind:suppliers
				{expenseAccounts}
				{vatRates}
				{projects}
				document={doc}
				{proposals}
				onSubmit={save}
			/>
		</div>
	</div>
{:else}
	<SupplierInvoiceForm
		bind:suppliers
		{expenseAccounts}
		{vatRates}
		{projects}
		document={null}
		proposals={[]}
		onSubmit={save}
	/>
{/if}
