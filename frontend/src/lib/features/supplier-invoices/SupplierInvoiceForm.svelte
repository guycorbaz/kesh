<!--
  Story 30-2b (#685, AC10 à AC12, AC14 à AC16, C-30-2-11) — LE formulaire de facture
  fournisseur.

  Extrait du formulaire en ligne de la liste (12-2, scan 12-4, projet 19-3) et du
  formulaire de complétion en ligne de la page d'import (12-5d), qui disparaissent :
  une définition au lieu de deux. La page `/supplier-invoices/new` l'affiche seul ou à
  côté du document.

  Trois modes, décidés par `document` **et** `hasDocument` (Story v017-4b, AC-23) :
  - **sans document** (`document === null && !hasDocument`) — tout se saisit, le scan
    navigateur 12.4 est proposé ;
  - **document sans QR** — tout se saisit (IBAN, référence, montant attendu compris),
    sans scan : le document est déjà sous les yeux. La page de complétion passe le
    document lui-même ; l'édition d'un brouillon qui a un document passe `document =
    null` et `hasDocument = true` (document sans QR, joint après coup, ou lien antérieur
    à la 0.17.0) ;
  - **document avec QR** — IBAN ou QR-IBAN, référence et montant attendu viennent du
    QR, affichés **en lecture seule** et absents du corps envoyé (le serveur refuse
    `QR_FIELDS_FROM_DOCUMENT`, C-30-2-9) ; le total TTC vivant est comparé au montant
    du QR ; pas de scan. L'édition ne passe le QR (`sourceDocument.qr`) que pour une
    facture **née de la complétion** de son document (`linkOrigin === 'completion'`).

  ⚠️ Le scan s'offre **si et seulement si `document === null && !hasDocument`** — jamais
  `!hasDocument` seul, qui le ferait réapparaître sur la page de complétion d'un document
  sans QR (`hasDocument` y vaut son défaut, `false`).

  Story v017-4b — `initial` (édition d'un brouillon) pré-remplit la saisie ; sa date de
  facture **prime** sur la règle « date vide avec un document » (#853). Le bouton dit
  « Enregistrer le brouillon » à la création, « Enregistrer » à l'édition. Les montants
  partent avec un point décimal (#830), le total vivant s'affiche aux décimales exactes
  (#835), une échéance antérieure à la date est signalée sous son champ et bloque
  l'envoi (#853).

  Fournisseur : les propositions reçues (`proposals`, calculées par la page) s'affichent
  avec leur motif et un bouton « Retenir » ; **aucune n'est présélectionnée** — la liste
  reste vide tant que l'utilisateur n'a ni retenu ni choisi (« un appariement propose,
  il ne crée jamais »). « Nouveau fournisseur » ouvre un panneau qui crée le contact
  sans quitter la saisie, puis le sélectionne.

  Un refus du serveur s'affiche **sans perdre la saisie** : `INVALID_PAYMENT_DETAILS`
  sous le champ nommé par `details.field`, les autres en tête (AC16).
-->
<script lang="ts">
	import { i18nMsg } from '$lib/shared/utils/i18n.svelte';
	import { notifySuccess, notifyError, notifyWarning } from '$lib/shared/utils/notify';
	import { isApiError } from '$lib/shared/utils/api-error';
	import { qrIbanHasValidQrIid } from '$lib/shared/utils/iban';
	import { formatExactAmount } from '$lib/features/invoices/invoice-helpers';
	import type { ContactResponse, StructuredAddress } from '$lib/features/contacts/contacts.types';
	import type { AccountResponse } from '$lib/features/accounts/accounts.types';
	import type { VatRateResponse } from '$lib/features/vat-rates/vat-rates.types';
	import type { ProjectResponse } from '$lib/features/projects/projects.types';
	import type { ImportedSupplierInvoice } from '$lib/features/imported-supplier-invoices/imported-supplier-invoices.types';
	import { scanQrSupplierInvoice } from './supplier-invoices.api';
	import { decodeQrFromImageFile, scanToPrefill } from './supplier-invoice-scan';
	import type {
		CreateSupplierInvoiceLineRequest,
		CreateSupplierInvoiceRequest,
	} from './supplier-invoices.types';
	import type { SupplierProposal } from './supplier-proposals';
	import {
		amountMismatch,
		describeSubmitError,
		dueBeforeInvoiceDate,
		dueBeforeInvoiceDateMessage,
		emptyLine,
		firstLineNotInCentimes,
		formatLiveTotal,
		formLineFromResponse,
		readableAmount,
		lineNotInCentimes,
		linesForServer,
		linesInvalid,
		paymentFields,
		sumTtc,
		type SupplierInvoiceFormInitial,
	} from './supplier-invoice-form';
	import { addDaysIso, hasSubCentime } from '$lib/features/invoices/invoice-helpers';
	import { todayLocal } from '$lib/features/open-items/open-items';
	import SupplierCreatePanel from './SupplierCreatePanel.svelte';

	/**
	 * Story v017-4b (validation P3, F8 ; P4, F3) — les **douze** champs du document que
	 * le formulaire lit : un `ImportedSupplierInvoice` les contient (complétion), l'objet
	 * `sourceDocument.qr` d'une facture aussi (édition).
	 */
	type FormDocument = Pick<
		ImportedSupplierInvoice,
		| 'creditorIban'
		| 'isQrIban'
		| 'referenceValue'
		| 'amount'
		| 'currency'
		| 'creditorName'
		| 'creditorAddressType'
		| 'creditorLine1'
		| 'creditorLine2'
		| 'creditorPostalCode'
		| 'creditorTown'
		| 'creditorCountry'
	>;

	interface Props {
		/** Fournisseurs de la liste déroulante ; un fournisseur retenu ou créé y est ajouté. */
		suppliers: ContactResponse[];
		expenseAccounts: AccountResponse[];
		vatRates: VatRateResponse[];
		projects: ProjectResponse[];
		/** Le document à côté duquel la facture se crée (ou son QR, à l'édition), ou `null`. */
		document: FormDocument | null;
		/** Fournisseurs proposés (document avec QR seulement), jamais présélectionnés. */
		proposals: SupplierProposal[];
		/** Enregistre ; rejette avec l'erreur de l'API, la saisie restant intacte. */
		onSubmit: (req: CreateSupplierInvoiceRequest) => Promise<void>;
		/**
		 * Story v017-4b (AC-23) — la facture a un document, même si `document` est `null`
		 * (édition d'un brouillon dont le document n'impose rien) : le scan est caché.
		 * Défaut `false` : la page de création et celle de complétion ne la passent pas.
		 */
		hasDocument?: boolean;
		/** Story v017-4b (AC-23) — le brouillon édité, ou `null` à la création. */
		initial?: SupplierInvoiceFormInitial | null;
	}

	let {
		suppliers = $bindable(),
		expenseAccounts,
		vatRates,
		projects,
		document,
		proposals,
		onSubmit,
		hasDocument = false,
		initial = null,
	}: Props = $props();

	/** Le document porte un QR : coordonnées de paiement verrouillées (AC11). */
	const qrLocked = $derived(document !== null && document.creditorIban !== null);
	/** Montant du QR, cible du total TTC (`null` sans QR ou montant ouvert). */
	const target = $derived(qrLocked ? (document?.amount ?? null) : null);
	/** Story v017-4b (AC-23, M35) : le scan, seulement sans aucun document. */
	const scanOffered = $derived(document === null && !hasDocument);

	/**
	 * La saisie de départ — Story v017-4b. Lue UNE fois, au montage (le formulaire ne se
	 * resynchronise pas : la saisie appartient ensuite à l'utilisateur).
	 * - `initial` (édition) : la facture relue ; sa date prime ;
	 * - sinon, la date du jour, **locale** (`todayLocal`, non la date UTC qui serait la
	 *   veille entre minuit et 1 h ou 2 h en Suisse). v0172-correctifs (#885,
	 *   C-v0172b-15) : avec un document, elle est **proposée et marquée « à vérifier »**
	 *   (`dateToVerify`) — la 0.17.0 (#853) la laissait vide, la date du jour n'étant
	 *   presque jamais celle d'une facture reçue ; l'avis le dit désormais, sans bloquer.
	 *   L'échéance, elle, est calculée par `dueFollows` ci-dessous.
	 */
	function startState() {
		const today = todayLocal();
		const init = initial;
		if (init === null) {
			return {
				contactId: null as number | null,
				number: '',
				invoiceDate: today,
				dueDate: '',
				iban: '',
				reference: '',
				expectedAmount: '',
				projectId: null as number | null,
				lines: [emptyLine()],
			};
		}
		return {
			contactId: init.contactId as number | null,
			number: init.supplierInvoiceNumber ?? '',
			invoiceDate: init.invoiceDate,
			dueDate: init.dueDate ?? '',
			iban: init.creditorQrIban ?? init.creditorIban ?? '',
			reference: init.paymentReference ?? '',
			// Revue P1 (B-8) : lisible comme le prix (`100.0000` → `100.00`).
			expectedAmount:
				init.expectedPaymentAmount == null ? '' : readableAmount(init.expectedPaymentAmount, 2),
			projectId: init.projectId,
			lines:
				init.lines.length > 0
					? init.lines.map((l) => formLineFromResponse(l, vatRates))
					: [emptyLine()],
		};
	}
	const start = startState();

	/**
	 * Revue P1 (E-7) — les taux des lignes du brouillon qui ne sont plus dans la liste des
	 * taux actifs (taux expiré) : proposés quand même, sans quoi le `<select>` les
	 * afficherait vides alors que le total vivant et l'envoi les emploient.
	 */
	const extraRates = start.lines
		.map((l) => l.vatRate)
		.filter((r, i, all) => r !== '0' && all.indexOf(r) === i)
		.filter((r) => !vatRates.some((v) => v.rate === r));

	let fContactId = $state<number | null>(start.contactId);
	let fNumber = $state(start.number);
	let fInvoiceDate = $state(start.invoiceDate);
	let fDueDate = $state(start.dueDate);
	let fIban = $state(start.iban);
	let fReference = $state(start.reference);
	let fExpectedAmount = $state(start.expectedAmount);
	let fProjectId = $state<number | null>(start.projectId);
	let fLines = $state<CreateSupplierInvoiceLineRequest[]>(start.lines);

	/**
	 * v0172-correctifs (#885, C-v0172b-15) — à la création à côté d'un document, la date
	 * proposée (le jour même) est « à vérifier » : un avis visible le dit sous le champ
	 * jusqu'à ce que l'utilisateur la modifie ou la confirme. Il ne bloque pas l'envoi.
	 */
	let dateToVerify = $state(initial === null && (document !== null || hasDocument));
	/**
	 * v0172-correctifs (#885, C-v0172b-16) — à la création, l'échéance **suit** la date :
	 * date + délai de paiement du fournisseur choisi (`defaultPaymentTermsDays`), ou la
	 * date elle-même sans délai ni fournisseur ; vide si la date l'est. Une saisie dans le
	 * champ échéance l'arrête pour de bon. À l'édition, elle ne suit jamais.
	 */
	let dueFollows = $state(initial === null);
	$effect(() => {
		if (!dueFollows) return;
		if (!fInvoiceDate) {
			fDueDate = '';
			return;
		}
		const days = suppliers.find((s) => s.id === fContactId)?.defaultPaymentTermsDays ?? null;
		fDueDate = days != null ? addDaysIso(fInvoiceDate, days) : fInvoiceDate;
	});

	/** Story v017-4b (#853) — l'échéance précède la date : signalée en direct. */
	const dueBefore = $derived(dueBeforeInvoiceDate(fInvoiceDate, fDueDate));
	/** Le refus serveur de l'échéance, affiché au même endroit (effacé à la saisie). */
	let dueServerError = $state('');
	let dateError = $state('');

	// Scan navigateur (Story 12.4), proposé seulement sans document.
	let scanning = $state(false);
	let scanFileInput = $state<HTMLInputElement | null>(null);
	/** L'IBAN affiché vient d'un scan de QR-IBAN : sa référence QRR lui est liée. */
	let scannedQrIban = $state(false);
	let scannedCurrency = $state('');
	let scannedCreditorName = $state('');

	let showNewSupplier = $state(false);
	let saving = $state(false);
	let formError = $state('');
	let ibanError = $state('');
	let referenceError = $state('');
	/** Story v017-3b (#687) — un refus serveur qui désigne une ligne (rang 0-based). */
	let lineServerError = $state<{ index: number; message: string } | null>(null);

	/** Adresse pré-remplie du panneau : QR de type `S` seulement (C-30-2-8). */
	const prefillAddress = $derived<StructuredAddress | null>(
		qrLocked && document?.creditorAddressType === 'S'
			? {
					street: document.creditorLine1 ?? '',
					building: document.creditorLine2 ?? '',
					postalCode: document.creditorPostalCode ?? '',
					city: document.creditorTown ?? '',
					country: document.creditorCountry ?? 'CH',
				}
			: null,
	);

	function addLine() {
		fLines = [...fLines, emptyLine()];
	}

	function removeLine(idx: number) {
		fLines = fLines.filter((_, i) => i !== idx);
	}

	/** Ajoute le contact à la liste s'il n'y est pas, puis le sélectionne. */
	function select(contact: ContactResponse) {
		if (!suppliers.some((s) => s.id === contact.id)) {
			suppliers = [...suppliers, contact].sort((a, b) => a.name.localeCompare(b.name));
		}
		fContactId = contact.id;
	}

	function onSupplierCreated(contact: ContactResponse) {
		select(contact);
		showNewSupplier = false;
		notifySuccess(
			i18nMsg('supplier-invoices-new-supplier-created', 'Fournisseur créé et sélectionné.'),
		);
	}

	/** Décodage jsQR (navigateur) → parse backend → pré-remplissage (Story 12.4). */
	async function onScanFile(e: Event) {
		const input = e.currentTarget as HTMLInputElement;
		const file = input.files?.[0];
		input.value = ''; // autoriser le re-scan du même fichier
		if (!file) return;
		// Garde-fou taille (LOW-3) : une photo brute peut geler le décodage canvas.
		if (file.size > 15 * 1024 * 1024) {
			notifyError(i18nMsg('supplier-invoices-scan-too-large', 'Image trop volumineuse (max 15 Mo).'));
			return;
		}
		scanning = true;
		formError = '';
		try {
			const spc = await decodeQrFromImageFile(file);
			if (!spc) {
				notifyWarning(i18nMsg('supplier-invoices-scan-no-qr', 'Aucun QR-code détecté sur cette image.'));
				return;
			}
			const p = scanToPrefill(await scanQrSupplierInvoice(spc));
			scannedQrIban = p.creditorQrIban !== '';
			fIban = p.creditorQrIban || p.creditorIban;
			fReference = p.paymentReference;
			fExpectedAmount = p.expectedAmount;
			scannedCurrency = p.currency;
			scannedCreditorName = p.creditorName;
			notifySuccess(i18nMsg('supplier-invoices-scan-ok', 'QR-facture lue — coordonnées pré-remplies.'));
		} catch (err) {
			notifyError(
				(isApiError(err) && err.message) ||
					i18nMsg('supplier-invoices-scan-failed', 'Impossible de lire cette image.'),
			);
		} finally {
			scanning = false;
		}
	}

	/**
	 * Un montant du QR, aux décimales exactes (Story v017-4b, #835 : la même lecture que
	 * le total vivant, `formatExactAmount`).
	 */
	function fmt(amount: string | null): string {
		if (!amount) return '—';
		return formatExactAmount(amount) || amount;
	}

	async function submit() {
		formError = '';
		ibanError = '';
		referenceError = '';
		dateError = '';
		dueServerError = '';
		lineServerError = null;
		if (!fContactId) {
			formError = i18nMsg('supplier-invoices-err-supplier', 'Sélectionnez un fournisseur.');
			return;
		}
		// Story v017-4b (#853, AC-24 (b), validation P1, F8) — une date vide a SON message,
		// sous le champ, et non celui des lignes.
		if (!fInvoiceDate.trim()) {
			// Revue P1 (E-6) : sans document, le message ne parle pas d'un document.
			dateError =
				document !== null || hasDocument
					? i18nMsg(
							'supplier-invoices-err-invoice-date',
							'Saisissez la date de la facture, telle qu’imprimée sur le document.',
						)
					: i18nMsg('supplier-invoices-err-invoice-date-plain', 'Saisissez la date de la facture.');
			return;
		}
		// L'échéance antérieure : déjà signalée sous son champ, elle bloque l'envoi.
		if (dueBefore) return;
		if (linesInvalid(fLines)) {
			formError = i18nMsg(
				'supplier-invoices-err-lines',
				'Chaque ligne requiert une description, un montant et un compte de charge.',
			);
			return;
		}
		// Story v017-3b (#687, C-v017-3-19) — APRÈS `linesInvalid` : une ligne à la fois
		// incomplète et sous le centime reçoit le message générique. Le bouton n'est pas
		// désactivé : le refus se dit au clic, avec sa cause.
		if (firstLineNotInCentimes(fLines) !== null) {
			formError = i18nMsg(
				'supplier-invoices-err-line-not-in-centimes',
				'Une ligne n’est pas au centime : saisissez-la au montant facturé par le fournisseur, au centime.',
			);
			return;
		}
		const base: CreateSupplierInvoiceRequest = {
			contactId: fContactId,
			supplierInvoiceNumber: fNumber.trim() || null,
			invoiceDate: fInvoiceDate,
			dueDate: fDueDate || null,
			projectId: fProjectId,
			// Story v017-4b (#830) : montants lus une fois, envoyés avec un point.
			lines: linesForServer(fLines),
		};
		// Avec QR, le corps ne porte AUCUNE coordonnée de paiement (AC11).
		const req = qrLocked
			? base
			: {
					...base,
					...paymentFields({ iban: fIban, reference: fReference, expectedAmount: fExpectedAmount }),
				};
		saving = true;
		try {
			await onSubmit(req);
		} catch (err) {
			// La saisie reste intacte : rien n'est vidé ici (AC16).
			const e = describeSubmitError(err);
			if (e.field === 'iban') ibanError = e.message;
			else if (e.field === 'reference') referenceError = e.message;
			else if (e.field === 'due') dueServerError = e.message;
			else if (e.field === 'line' && e.line !== undefined && e.line >= 1 && e.line <= fLines.length)
				lineServerError = { index: e.line - 1, message: e.message };
			else formError = e.message;
		} finally {
			saving = false;
		}
	}
</script>

<form
	class="space-y-4"
	data-testid="supplier-invoice-form"
	onsubmit={(e) => {
		e.preventDefault();
		void submit();
	}}
>
	{#if scanOffered}
		<!-- Story 12.4 : scan QR-facture → pré-remplissage des coordonnées. Story v017-4b
		     (AC-23) : seulement si `document === null && !hasDocument`. -->
		<div class="flex flex-wrap items-center gap-3 rounded bg-muted/40 p-3">
			<input
				type="file"
				accept="image/*"
				class="hidden"
				bind:this={scanFileInput}
				data-testid="supplier-invoice-scan-file"
				onchange={onScanFile}
			/>
			<button
				type="button"
				class="rounded border px-3 py-1 text-sm disabled:opacity-60"
				data-testid="supplier-invoice-scan-btn"
				disabled={scanning}
				onclick={() => scanFileInput?.click()}
			>
				{scanning
					? i18nMsg('supplier-invoices-scan-running', 'Lecture…')
					: i18nMsg('supplier-invoices-scan', 'Scanner une QR-facture')}
			</button>
			<span class="text-xs text-text-muted">
				{i18nMsg(
					'supplier-invoices-scan-hint',
					'Chargez une image de la QR-facture pour pré-remplir IBAN, référence et montant.',
				)}
			</span>
			{#if scannedCreditorName}
				<span class="text-xs" data-testid="supplier-invoice-scan-creditor">
					{i18nMsg('supplier-invoices-scan-detected', 'Créancier détecté')} :
					<strong>{scannedCreditorName}</strong>
				</span>
			{/if}
		</div>
	{/if}

	{#if qrLocked && document}
		<div class="rounded border border-border bg-muted/30 p-3 text-sm" data-testid="supplier-invoice-qr-locked">
			<p class="mb-2 text-text-muted">
				{i18nMsg(
					'supplier-invoices-qr-locked',
					'Coordonnées lues sur le QR-code du document : elles ne se modifient pas.',
				)}
			</p>
			<div class="grid grid-cols-1 gap-3 sm:grid-cols-3">
				<label class="block">
					{document.isQrIban
						? i18nMsg('supplier-invoices-qr-field-qr-iban', 'QR-IBAN')
						: i18nMsg('supplier-invoices-qr-field-iban', 'IBAN')}
					<input
						class="mt-1 w-full rounded border bg-muted px-2 py-1 font-mono"
						data-testid="supplier-invoice-iban"
						readonly
						value={document.creditorIban ?? ''}
					/>
				</label>
				<label class="block">
					{i18nMsg('supplier-invoices-detail-reference', 'Référence')}
					<input
						class="mt-1 w-full rounded border bg-muted px-2 py-1 font-mono"
						data-testid="supplier-invoice-reference"
						readonly
						value={document.referenceValue ?? ''}
					/>
				</label>
				<label class="block">
					{i18nMsg('supplier-invoices-qr-field-amount', 'Montant du QR')}
					<input
						class="mt-1 w-full rounded border bg-muted px-2 py-1"
						data-testid="supplier-invoice-expected-amount"
						readonly
						value={document.amount ? `${fmt(document.amount)} ${document.currency ?? ''}` : '—'}
					/>
				</label>
			</div>
		</div>
		{#if hasSubCentime(target)}
			<p class="rounded border border-warning bg-warning-soft p-2 text-sm">
				{i18nMsg(
					'supplier-invoices-subcentime',
					'Le montant du QR ({$amount}) contient des sous-centimes — impossible à atteindre par des lignes centime-exactes. Recommandation : retirer ce document de la liste.',
					{ amount: target ?? '' },
				)}
			</p>
		{/if}
	{/if}

	<div class="space-y-2">
		<label class="block text-sm">
			{i18nMsg('supplier-invoices-field-supplier', 'Fournisseur')}
			<select
				class="mt-1 w-full rounded border px-2 py-1"
				data-testid="supplier-invoice-supplier"
				bind:value={fContactId}
			>
				<option value={null}>—</option>
				{#each suppliers as s (s.id)}
					<option value={s.id}>{s.name}</option>
				{/each}
			</select>
		</label>
		{#if proposals.length > 0}
			<ul class="space-y-1 text-sm" data-testid="supplier-proposals">
				{#each proposals as p (p.contact.id)}
					<li class="flex flex-wrap items-center gap-2" data-testid="supplier-proposal">
						<span>
							{i18nMsg('supplier-invoices-proposed', 'Proposé : {$name}', { name: p.contact.name })}
						</span>
						<span class="text-xs text-text-muted">
							({p.reason === 'iban'
								? i18nMsg('supplier-invoices-proposed-iban', 'même IBAN')
								: i18nMsg('supplier-invoices-proposed-name', 'nom proche')})
						</span>
						<button
							type="button"
							class="rounded border px-2 py-0.5 text-xs disabled:opacity-60"
							data-testid="supplier-proposal-retain"
							disabled={fContactId === p.contact.id}
							onclick={() => select(p.contact)}
						>
							{i18nMsg('supplier-invoices-proposed-retain', 'Retenir')}
						</button>
					</li>
				{/each}
			</ul>
		{/if}
		{#if !showNewSupplier}
			<button
				type="button"
				class="text-sm text-primary"
				data-testid="supplier-new-open"
				onclick={() => (showNewSupplier = true)}
			>
				+ {i18nMsg('supplier-invoices-new-supplier', 'Nouveau fournisseur')}
			</button>
		{:else}
			<SupplierCreatePanel
				initialName={qrLocked ? (document?.creditorName ?? '') : ''}
				initialAddress={prefillAddress}
				onCreated={onSupplierCreated}
				onCancel={() => (showNewSupplier = false)}
			/>
		{/if}
	</div>

	<div class="grid grid-cols-1 gap-4 sm:grid-cols-2">
		<label class="block text-sm">
			{i18nMsg('supplier-invoices-field-number', 'N° facture fournisseur')}
			<input class="mt-1 w-full rounded border px-2 py-1" data-testid="supplier-invoice-number" bind:value={fNumber} />
		</label>
		<label class="block text-sm">
			{i18nMsg('supplier-invoices-field-date', 'Date de facture')}
			<input
				type="date"
				class="mt-1 w-full rounded border px-2 py-1"
				data-testid="supplier-invoice-date"
				aria-required="true"
				aria-invalid={dateError !== ''}
				aria-describedby={dateToVerify ? 'supplier-invoice-date-to-verify' : undefined}
				bind:value={fInvoiceDate}
				oninput={() => {
					dateError = '';
					// Revue P1 (B-9 = E-5) : le refus serveur de l'échéance portait les dates
					// d'avant ; corriger la date de facture l'efface aussi.
					dueServerError = '';
					// v0172 (#885) : la date est modifiée, elle n'est plus « à vérifier ».
					dateToVerify = false;
				}}
			/>
			<!-- v0172-correctifs (#885) : la date proposée au jour même est signalée,
			     à la place de l'aide, jusqu'à ce qu'elle soit modifiée ou confirmée. -->
			{#if dateToVerify}
				<span
					id="supplier-invoice-date-to-verify"
					class="mt-1 flex flex-wrap items-center gap-2 rounded border border-warning bg-warning/10 px-2 py-1 text-xs"
					data-testid="supplier-invoice-date-to-verify"
					aria-live="polite"
				>
					<span>
						{i18nMsg(
							'supplier-invoices-date-to-verify',
							'Date du jour, proposée par défaut — ce n’est pas celle du document. Vérifiez-la.',
						)}
					</span>
					<button
						type="button"
						class="rounded border border-border bg-surface px-2 py-0.5"
						data-testid="supplier-invoice-date-confirm"
						onclick={() => (dateToVerify = false)}
					>
						{i18nMsg('supplier-invoices-date-confirm', 'C’est la bonne date')}
					</button>
				</span>
			{:else if document !== null || hasDocument}
				<!-- Story v017-4b (#853, AC-24 (b)) : l'aide dit quelle date saisir. -->
				<span class="mt-1 block text-xs text-text-muted" data-testid="supplier-invoice-date-help">
					{i18nMsg(
						'supplier-invoices-field-date-help',
						'Date de la facture, telle qu’imprimée sur le document',
					)}
				</span>
			{/if}
			{#if dateError}
				<span class="mt-1 block text-xs text-destructive" role="alert" data-testid="supplier-invoice-date-error"
					>{dateError}</span
				>
			{/if}
		</label>
		<label class="block text-sm">
			{i18nMsg('supplier-invoices-field-due', 'Échéance')}
			<input
				type="date"
				class="mt-1 w-full rounded border px-2 py-1"
				data-testid="supplier-invoice-due"
				aria-invalid={dueBefore || dueServerError !== ''}
				bind:value={fDueDate}
				oninput={() => {
					dueServerError = '';
					// v0172 (#885) : saisie à la main, l'échéance ne suit plus la date.
					dueFollows = false;
				}}
			/>
			<!-- Story v017-4b (#853) : l'échéance antérieure, sous le champ — en direct, ou
			     le refus du serveur (`SUPPLIER_INVOICE_DUE_BEFORE_INVOICE_DATE`). -->
			{#if dueBefore || dueServerError}
				<span class="mt-1 block text-xs text-destructive" role="alert" data-testid="supplier-invoice-due-error"
					>{dueBefore ? dueBeforeInvoiceDateMessage(fInvoiceDate, fDueDate) : dueServerError}</span
				>
			{/if}
		</label>
		{#if !qrLocked}
			<label class="block text-sm">
				{qrIbanHasValidQrIid(fIban)
					? i18nMsg('supplier-invoices-field-qr-iban', 'QR-IBAN (optionnel)')
					: i18nMsg('supplier-invoices-field-iban', 'IBAN / QR-IBAN (optionnel)')}
				<input
					class="mt-1 w-full rounded border px-2 py-1 font-mono"
					data-testid="supplier-invoice-iban"
					aria-invalid={ibanError !== ''}
					bind:value={fIban}
					oninput={() => {
						ibanError = '';
						// Édition manuelle de l'IBAN après un scan de QR-IBAN : la référence
						// QRR scannée lui était liée, le créancier aussi — on les retire.
						// Une référence saisie à la main n'est jamais effacée.
						if (scannedQrIban) {
							fReference = '';
							scannedCurrency = '';
							scannedCreditorName = '';
						}
						scannedQrIban = false;
					}}
				/>
				{#if ibanError}
					<span class="mt-1 block text-xs text-destructive" role="alert" data-testid="supplier-invoice-iban-error"
						>{ibanError}</span
					>
				{/if}
			</label>
			<label class="block text-sm">
				{i18nMsg('supplier-invoices-field-reference', 'Référence (optionnel)')}
				<input
					class="mt-1 w-full rounded border px-2 py-1 font-mono"
					data-testid="supplier-invoice-reference"
					aria-invalid={referenceError !== ''}
					bind:value={fReference}
					oninput={() => (referenceError = '')}
				/>
				{#if referenceError}
					<span
						class="mt-1 block text-xs text-destructive"
						role="alert"
						data-testid="supplier-invoice-reference-error">{referenceError}</span
					>
				{/if}
			</label>
			<label class="block text-sm">
				{i18nMsg('supplier-invoices-field-expected-amount', 'Montant attendu TTC (optionnel)')}
				{#if scannedCurrency}
					<span class="text-xs text-text-muted" data-testid="supplier-invoice-scan-currency"
						>({scannedCurrency})</span
					>
				{/if}
				<input
					class="mt-1 w-full rounded border px-2 py-1"
					data-testid="supplier-invoice-expected-amount"
					inputmode="decimal"
					bind:value={fExpectedAmount}
				/>
			</label>
		{/if}
		{#if projects.length > 0}
			<label class="block text-sm">
				{i18nMsg('supplier-invoices-field-project', 'Projet analytique (optionnel)')}
				<select
					class="mt-1 w-full rounded border px-2 py-1"
					data-testid="supplier-invoice-project"
					bind:value={fProjectId}
				>
					<option value={null}>{i18nMsg('supplier-invoices-project-none', '— Aucun')}</option>
					{#each projects.filter((p) => p.parentId === null) as root (root.id)}
						<option value={root.id}>{root.code} — {root.name}</option>
						{#each projects.filter((c) => c.parentId === root.id) as child (child.id)}
							<option value={child.id}>&nbsp;&nbsp;↳ {child.code} — {child.name}</option>
						{/each}
					{/each}
				</select>
			</label>
		{/if}
	</div>

	<div>
		<div class="mb-2 text-sm font-medium">{i18nMsg('supplier-invoices-lines', 'Lignes')}</div>
		{#each fLines as line, idx (idx)}
			{@const subCentime = lineNotInCentimes(line)}
			<div class="mb-2 grid grid-cols-12 gap-2" data-testid="supplier-invoice-line">
				<input
					class="col-span-4 rounded border px-2 py-1 text-sm"
					data-testid="supplier-invoice-line-desc"
					placeholder={i18nMsg('supplier-invoices-line-desc', 'Description')}
					bind:value={line.description}
				/>
				<input
					class="col-span-1 rounded border px-2 py-1 text-sm"
					data-testid="supplier-invoice-line-qty"
					inputmode="decimal"
					placeholder={i18nMsg('supplier-invoices-col-qty', 'Qté')}
					bind:value={line.quantity}
				/>
				<input
					class="col-span-2 rounded border px-2 py-1 text-sm"
					data-testid="supplier-invoice-line-unit-price"
					inputmode="decimal"
					placeholder={i18nMsg('supplier-invoices-line-ht', 'HT')}
					bind:value={line.unitPrice}
				/>
				<select
					class="col-span-2 rounded border px-2 py-1 text-sm"
					data-testid="supplier-invoice-line-vat"
					bind:value={line.vatRate}
				>
					<option value="0">0%</option>
					{#each extraRates as r (r)}
						<!-- Revue P2 (E2-5) : l'étiquette à l'échelle des taux actifs (7.7000 → 7.70),
						     la valeur inchangée. -->
						<option value={r}>{readableAmount(r, 2)}%</option>
					{/each}
					{#each vatRates as r (r.id)}
						<option value={r.rate}>{r.rate}%</option>
					{/each}
				</select>
				<select
					class="col-span-2 rounded border px-2 py-1 text-sm"
					data-testid="supplier-invoice-line-account"
					bind:value={line.expenseAccountId}
				>
					<option value={0}>{i18nMsg('supplier-invoices-line-account', 'Compte')}</option>
					{#each expenseAccounts as a (a.id)}
						<option value={a.id}>{a.number} {a.name}</option>
					{/each}
				</select>
				<button
					type="button"
					class="col-span-1 text-destructive"
					onclick={() => removeLine(idx)}
					disabled={fLines.length === 1}>✕</button
				>
			</div>
			{#if subCentime !== null}
				<p class="mb-2 text-sm text-destructive" data-testid="supplier-invoice-line-subcentime">
					{i18nMsg(
						'supplier-invoices-line-not-in-centimes',
						'Le montant de la ligne ({ $lineTotal }) n’est pas au centime. Saisissez la ligne au montant facturé par le fournisseur, au centime.',
						{ lineTotal: subCentime },
					)}
				</p>
			{/if}
			{#if lineServerError?.index === idx}
				<p class="mb-2 text-sm text-destructive" role="alert" data-testid="supplier-invoice-line-server-error">
					{lineServerError.message}
				</p>
			{/if}
		{/each}
		<button type="button" class="text-sm text-primary" data-testid="supplier-invoice-add-line" onclick={addLine}>
			+ {i18nMsg('supplier-invoices-add-line', 'Ajouter une ligne')}
		</button>
	</div>

	<div class="text-sm" data-testid="supplier-invoice-sum-ttc">
		{i18nMsg('supplier-invoices-sum-ttc', 'Total TTC des lignes')} :
		<!-- Story v017-4b (#835, AC-24 (d)) : aux décimales exactes. -->
		<span class:text-destructive={amountMismatch(fLines, target)}>
			{formatLiveTotal(sumTtc(fLines))}
		</span>
		{#if target}
			/ {i18nMsg('supplier-invoices-target', 'montant du QR')} : {fmt(target)}
			{#if amountMismatch(fLines, target)}
				<span class="text-destructive" data-testid="supplier-invoice-mismatch"
					>— {i18nMsg('supplier-invoices-mismatch', 'écart à corriger')}</span
				>
			{/if}
		{/if}
	</div>

	{#if formError}
		<p class="text-sm text-destructive" role="alert" data-testid="supplier-invoice-form-error">{formError}</p>
	{/if}

	<button
		type="submit"
		class="rounded bg-primary px-4 py-2 text-sm text-primary-foreground disabled:opacity-60"
		data-testid="supplier-invoice-submit"
		disabled={saving}
	>
		<!-- Story v017-4b (AC-24 (a)) : la création enregistre un brouillon. -->
		{saving
			? '…'
			: initial === null
				? i18nMsg('supplier-invoices-save-draft', 'Enregistrer le brouillon')
				: i18nMsg('supplier-invoices-save', 'Enregistrer')}
	</button>
</form>
