<!--
  Story v017-4b (#849, AC-23) — modifier un brouillon de facture fournisseur.

  La page charge la facture ; si elle n'est pas un brouillon, elle renvoie à sa fiche avec
  un message (une facture validée se dévalide d'abord). Sinon elle rend
  `SupplierInvoiceForm` **pré-rempli** (`initial`) ; l'enregistrement envoie le `PUT` avec
  la `version` lue, puis va à la fiche.

  Ce que la page passe au formulaire, selon le document de la facture
  (`sourceDocument`, Story v017-4a) :
  - né de la **complétion** d'un document **à QR** (`linkOrigin === 'completion'` et
    `qr !== null`) : `document = qr` — coordonnées du QR en lecture seule, total comparé
    au montant du QR (le serveur applique les mêmes règles au `PUT`) ;
  - tout autre document — sans QR, joint après coup (`'attachment'`), lien antérieur à la
    0.17.0 (`null`) : `document = null`, saisie libre ; passer le QR d'un document joint
    après coup serait le mutant M34 ;
  - `hasDocument` vaut `sourceDocument !== null` : le scan est caché dès que la facture a
    un document (M35).

  Les données du QR viennent de la facture (`sourceDocument.qr`) : aucune lecture de la
  liste des documents. Un refus laisse la saisie intacte et s'affiche à sa place ;
  `OPTIMISTIC_LOCK_CONFLICT` invite à recharger (`describeSubmitError`).
-->
<script lang="ts">
	import { onMount } from 'svelte';
	import { goto } from '$app/navigation';
	import { page } from '$app/state';
	import { authState } from '$lib/app/stores/auth.svelte';
	import { i18nMsg } from '$lib/shared/utils/i18n.svelte';
	import { notifySuccess, notifyWarning } from '$lib/shared/utils/notify';
	import { isApiError } from '$lib/shared/utils/api-error';
	import {
		getSupplierInvoice,
		updateSupplierInvoice,
	} from '$lib/features/supplier-invoices/supplier-invoices.api';
	import type {
		CreateSupplierInvoiceRequest,
		SupplierInvoiceResponse,
	} from '$lib/features/supplier-invoices/supplier-invoices.types';
	import SupplierInvoiceForm from '$lib/features/supplier-invoices/SupplierInvoiceForm.svelte';
	import { getContact, listContacts } from '$lib/features/contacts/contacts.api';
	import type { ContactResponse } from '$lib/features/contacts/contacts.types';
	import { fetchAccounts } from '$lib/features/accounts/accounts.api';
	import type { AccountResponse } from '$lib/features/accounts/accounts.types';
	import { listVatRates } from '$lib/features/vat-rates/vat-rates.api';
	import type { VatRateResponse } from '$lib/features/vat-rates/vat-rates.types';
	import { listProjects } from '$lib/features/projects/projects.api';
	import type { ProjectResponse } from '$lib/features/projects/projects.types';

	const id = Number(page.params.id);

	const canWrite = $derived(
		authState.currentUser?.role === 'Admin' || authState.currentUser?.role === 'Comptable',
	);

	let loading = $state(true);
	let loadError = $state('');
	let invoice = $state<SupplierInvoiceResponse | null>(null);
	let suppliers = $state<ContactResponse[]>([]);
	let expenseAccounts = $state<AccountResponse[]>([]);
	let vatRates = $state<VatRateResponse[]>([]);
	let projects = $state<ProjectResponse[]>([]);

	/** Le QR que le formulaire verrouille : seulement pour une facture née d'un QR. */
	const qrDocument = $derived.by(() => {
		const doc = invoice?.sourceDocument;
		return doc && doc.linkOrigin === 'completion' && doc.qr !== null ? doc.qr : null;
	});
	const hasDocument = $derived(invoice?.sourceDocument != null);

	onMount(async () => {
		if (!canWrite) {
			loading = false;
			return;
		}
		try {
			const [inv, contactsRes, accountsRes, ratesRes, projectsRes] = await Promise.all([
				getSupplierInvoice(id),
				listContacts({ isSupplier: true, limit: 200 }),
				fetchAccounts(),
				listVatRates(),
				listProjects(true), // archivés compris : le projet d'un brouillon reste lisible
			]);
			if (inv.status !== 'draft') {
				notifyWarning(
					i18nMsg(
						'supplier-invoices-edit-not-draft',
						'Seul un brouillon se modifie : dévalidez d’abord la facture.',
					),
				);
				await goto(`/supplier-invoices/${id}`);
				return;
			}
			let list = contactsRes.items;
			// Le fournisseur du brouillon reste sélectionnable, même hors de la liste.
			if (!list.some((c) => c.id === inv.contactId)) {
				try {
					list = [...list, await getContact(inv.contactId)];
				} catch {
					// introuvable : le formulaire montre « — », l'utilisateur choisit
				}
			}
			suppliers = list;
			// Les comptes de charge postés ; un compte d'une ligne devenu inactif est gardé
			// pour que la ligne se lise, le serveur jugeant à l'enregistrement.
			const lineAccounts = new Set(inv.lines.map((l) => l.expenseAccountId));
			expenseAccounts = accountsRes.filter(
				(a) =>
					a.accountType === 'Expense' && ((a.active && a.postable) || lineAccounts.has(a.id)),
			);
			vatRates = ratesRes;
			projects = projectsRes;
			invoice = inv;
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
		if (!invoice) return;
		await updateSupplierInvoice(id, { ...req, version: invoice.version });
		notifySuccess(i18nMsg('supplier-invoices-updated', 'Brouillon enregistré.'));
		await goto(`/supplier-invoices/${id}`);
	}
</script>

<svelte:head>
	<title>{i18nMsg('supplier-invoices-edit-title', 'Modifier le brouillon')} — Kesh</title>
</svelte:head>

<div class="mb-6 flex flex-wrap items-center justify-between gap-3">
	<h1 class="text-2xl font-semibold">
		{i18nMsg('supplier-invoices-edit-title', 'Modifier le brouillon')}
	</h1>
	<a class="text-sm text-primary" href="/supplier-invoices/{id}">
		{i18nMsg('supplier-invoices-back-to-invoice', 'Revenir à la facture')}
	</a>
</div>

{#if !canWrite}
	<p class="text-sm text-text-muted" data-testid="supplier-invoice-edit-forbidden">
		{i18nMsg(
			'supplier-invoices-new-forbidden',
			'Votre rôle ne permet pas de saisir une facture fournisseur.',
		)}
	</p>
{:else if loading}
	<p class="text-sm text-text-muted" data-testid="supplier-invoice-edit-loading">
		{i18nMsg('common-loading', 'Chargement…')}
	</p>
{:else if loadError}
	<p class="text-sm text-destructive" role="alert" data-testid="supplier-invoice-edit-error">
		{loadError}
	</p>
{:else if invoice}
	{#if invoice.sourceDocument}
		<p class="mb-4 text-sm text-text-muted" data-testid="supplier-invoice-edit-document">
			{i18nMsg('supplier-invoices-doc-label', 'Document')} : {invoice.sourceDocument.originalFilename}
		</p>
	{/if}
	<SupplierInvoiceForm
		bind:suppliers
		{expenseAccounts}
		{vatRates}
		{projects}
		document={qrDocument}
		{hasDocument}
		initial={invoice}
		proposals={[]}
		onSubmit={save}
	/>
{/if}
