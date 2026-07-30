<!--
  InvoiceForm (Story 5.1) : formulaire partagé création/édition facture brouillon.
-->
<script lang="ts">
	import { Button } from '$lib/components/ui/button';
	import { Input } from '$lib/components/ui/input';
	import * as Dialog from '$lib/components/ui/dialog';
	import { notifyError, notifySuccess } from '$lib/shared/utils/notify';
	import { isApiError } from '$lib/shared/utils/api-client';
	import { goto } from '$app/navigation';
	import { Plus, Trash2, Package } from '@lucide/svelte';
	import Big from 'big.js';

	import type { ContactResponse } from '$lib/features/contacts/contacts.types';
	import type { ProductResponse } from '$lib/features/products/products.types';
	import type {
		CreateInvoiceLineRequest,
		CreateInvoiceRequest,
		InvoiceResponse,
		UpdateInvoiceRequest,
	} from '$lib/features/invoices/invoices.types';
	import {
		addDaysIso,
		computeInvoiceTotal,
		computeLineTotal,
		formatInvoiceTotal,
	} from '$lib/features/invoices/invoice-helpers';
	import { createInvoice, getInvoice, updateInvoice, getInvoiceSettings } from '$lib/features/invoices/invoices.api';
	import { getContact } from '$lib/features/contacts/contacts.api';
	import { i18nMsg } from '$lib/shared/utils/i18n.svelte';
	import type { InvoiceSettingsResponse } from '$lib/features/invoices/invoices.types';

	import ContactPicker from './ContactPicker.svelte';
	import ProductPicker from './ProductPicker.svelte';
	import { getVatRates } from '$lib/features/vat-rates';
	import AccountAutocomplete from '$lib/features/journal-entries/AccountAutocomplete.svelte';
	import { fetchAccounts } from '$lib/features/accounts/accounts.api';
	import type { AccountResponse } from '$lib/features/accounts/accounts.types';
	import { isAccountUnusable } from '$lib/features/accounts/account-validity';
	import { listProjects } from '$lib/features/projects/projects.api';
	import type { ProjectResponse } from '$lib/features/projects/projects.types';

	type Props = { invoice?: InvoiceResponse | null };
	let { invoice = null }: Props = $props();

	// Story 7.2 (KF-003) : taux TVA chargés dynamiquement depuis le backend.
	// Cache de session via le store `vat-rates`. Pendant le chargement initial,
	// `vatOptions` est vide et le `<select>` est `disabled` pour empêcher
	// toute soumission prématurée.
	let vatOptions = $state<string[]>([]);
	const DEFAULT_VAT = $derived(vatOptions[0] ?? '8.10');

	$effect(() => {
		// Pass 1 remediation #10 : flag `cancelled` pour éviter d'écrire dans
		// `vatOptions` si le composant se démonte pendant le fetch.
		let cancelled = false;
		(async () => {
			try {
				const rates = await getVatRates();
				if (!cancelled) {
					vatOptions = rates.map((r) => r.rate);
				}
			} catch {
				// Échec réseau : on garde la liste vide → `<select>` disabled,
				// l'utilisateur voit l'erreur globale au moment du submit (validate).
			}
		})();
		return () => {
			cancelled = true;
		};
	});

	// Pass 1 remediation #8 : si une ligne a été initialisée avec le fallback
	// hardcodé `'8.10'` (avant resolve du store) et que `'8.10'` n'est pas dans
	// la liste de la company courante, repointer la ligne sur le premier
	// vatOption résolu. No-op pour les companies suisses standard où `8.10`
	// fait partie du seed — défense pour Epic 11-1 (CRUD admin) où la liste
	// peut diverger d'une company à l'autre.
	$effect(() => {
		if (vatOptions.length === 0) return;
		if (vatOptions.includes('8.10')) return;
		const fallback = vatOptions[0];
		for (const l of lines) {
			if (l.vatRate === '8.10') {
				l.vatRate = fallback;
			}
		}
	});

	// Identifiant UI stable (cross-reorder/remove). NE PAS envoyer au backend.
	type LineState = CreateInvoiceLineRequest & { _uiKey: string };

	let uiKeySeq = 0;
	function nextUiKey(): string {
		uiKeySeq += 1;
		return `line-${Date.now()}-${uiKeySeq}`;
	}

	function stripUiKey(l: LineState): CreateInvoiceLineRequest {
		const { _uiKey, ...rest } = l;
		return rest;
	}

	function todayIso(): string {
		const d = new Date();
		return d.toISOString().slice(0, 10);
	}

	function initLines(): LineState[] {
		if (invoice && invoice.lines.length > 0) {
			return invoice.lines.map((l) => ({
				description: l.description,
				quantity: l.quantity,
				unitPrice: l.unitPrice,
				vatRate: l.vatRate,
				// AC9-bis — site 1/4. Recopier depuis la réponse serveur, sinon toute
				// ouverture d'un brouillon perdrait sa ventilation au ré-enregistrement.
				revenueAccountId: l.revenueAccountId,
				_uiKey: nextUiKey(),
			}));
		}
		return [
			{
				description: '',
				quantity: '1',
				unitPrice: '0.00',
				vatRate: DEFAULT_VAT,
				_uiKey: nextUiKey(),
			},
		];
	}

	// svelte-ignore state_referenced_locally
	const initialInvoice = invoice;
	let selectedContact = $state<ContactResponse | null>(null);
	let date = $state<string>(initialInvoice?.date ?? todayIso());
	let dueDate = $state<string>(initialInvoice?.dueDate ?? '');
	let paymentTerms = $state<string>(initialInvoice?.paymentTerms ?? '');
	// Projet analytique document-level (Epic 19, Story 19-4).
	let projectId = $state<number | null>(initialInvoice?.projectId ?? null);
	// Projets actifs pour le sélecteur — échec de chargement toléré (sélecteur
	// masqué, création identique à avant). Pattern identique au fetch VAT rates.
	let projects = $state<ProjectResponse[]>([]);
	// Distingue « liste chargée » d'un échec réseau : sans ce flag, un simple
	// échec de fetch ferait étiqueter « Projet archivé » un tag parfaitement
	// actif (review 19-4 ECH-L1).
	let projectsLoaded = $state(false);
	$effect(() => {
		let cancelled = false;
		(async () => {
			try {
				const list = await listProjects();
				if (!cancelled) {
					projects = list;
					projectsLoaded = true;
				}
			} catch {
				// Échec toléré : sélecteur masqué (sauf tag existant, option ad-hoc).
			}
		})();
		return () => {
			cancelled = true;
		};
	});
	// Story 16-1b (T3, AC5 / D11) — liste des comptes pour le sélecteur par ligne.
	//
	// `fetchAccounts(true)` : le flag `includeArchived` est **OBLIGATOIRE**, et
	// c'est le piège n°1 de la story parce qu'il est SILENCIEUX. `fetchAccounts()`
	// compile, s'exécute, et marche parfaitement — jusqu'au jour où un compte est
	// archivé : il n'est alors plus dans le tableau, `AccountAutocomplete` ne
	// résout plus son libellé, et le champ de la ligne concernée s'affiche VIDE.
	// L'utilisateur croit la ligne sans compte alors qu'elle en porte un.
	// Précédent identique déjà résolu dans le dépôt :
	// `routes/(app)/journal-entries/[id]/+page.svelte:33-43`.
	//
	// Le dropdown continue de ne proposer que `active && postable` — charger la
	// liste complète sert uniquement à résoudre un libellé persisté.
	let accounts = $state<AccountResponse[]>([]);
	// AC7 / D10 : l'échec ne bloque PAS la saisie. Le champ est optionnel ; son
	// indisponibilité fait simplement retomber les lignes sur le défaut société.
	let accountsLoadError = $state(false);
	$effect(() => {
		let cancelled = false;
		(async () => {
			try {
				const list = await fetchAccounts(true);
				if (!cancelled) accounts = list;
			} catch {
				if (!cancelled) accountsLoadError = true;
			}
		})();
		return () => {
			cancelled = true;
		};
	});

	// Sélecteur affiché si des projets actifs existent OU si un tag historique
	// est présent (projet archivé depuis — leçon 19-2 BH-L2 : visible/détaguable).
	const showProjectField = $derived(projects.length > 0 || projectId !== null);
	let lines = $state<LineState[]>(initLines());
	let submitting = $state(false);
	let errorMsg = $state<string>('');
	let productPickerOpen = $state(false);
	let conflictOpen = $state(false);
	let reloading = $state(false);
	let conflictError = $state<string>('');
	// Compteur de séquence : garantit qu'un double-click « Recharger » ne laisse
	// qu'une seule réponse écrire dans l'état (la plus récente).
	let reloadSeq = 0;
	// ID de facture pour lequel le chargement initial du contact a été tenté.
	// Scoping sur `invoice.id` (pas un simple boolean) : si une future consommation
	// de ce composant réutilise l'instance avec une autre facture, le flag se
	// réinitialise automatiquement.
	let initialContactInvoiceId = $state<number | null>(null);

	// Story 2.6: Invoice settings validation
	let invoiceSettings = $state<InvoiceSettingsResponse | null>(null);
	let loadingSettings = $state(true);
	let settingsError = $state<string>('');
	let settingsSeq = 0;

	/** Le compte de produit par défaut de la société, résolu pour l'affichage (D9). */
	const defaultRevenueAccount = $derived(
		invoiceSettings?.defaultRevenueAccountId == null
			? undefined
			: accounts.find((a) => a.id === invoiceSettings!.defaultRevenueAccountId)
	);

	/**
	 * Placeholder d'une ligne sans compte (D9/AC6) : nomme le compte cible.
	 *
	 * `null` veut dire « suivre le défaut société », pas « aucun compte ». Un
	 * champ vide laisserait croire à un oubli, et pousserait l'utilisateur à
	 * sélectionner explicitement le défaut « pour être sûr » — cas que 16-1a
	 * D3-bis existe précisément pour absorber. **Affichage seulement : la ligne
	 * reste à `NULL` en base.**
	 */
	const defaultAccountPlaceholder = $derived(
		defaultRevenueAccount
			? i18nMsg('invoice-line-revenue-account-default', '{ $account } (défaut société)', {
					account: `${defaultRevenueAccount.number} — ${defaultRevenueAccount.name}`
				})
			: undefined
	);

	/**
	 * Numéros (1-based) des lignes dont le compte persisté n'est plus utilisable.
	 *
	 * Règle déléguée à `account-validity`, la MÊME que celle du marqueur affiché
	 * par `AccountAutocomplete` — un blocage sans marqueur, ou l'inverse, serait
	 * pire que rien.
	 */
	const invalidRevenueAccountLines = $derived.by(() => {
		if (accountsLoadError || accounts.length === 0) return [];
		const out: number[] = [];
		lines.forEach((l, i) => {
			if (l.revenueAccountId == null) return;
			const acc = accounts.find((a) => a.id === l.revenueAccountId);
			if (
				isAccountUnusable(acc, {
					requiredAccountType: 'Revenue',
					postableExemptAccountId: invoiceSettings?.defaultRevenueAccountId ?? null
				})
			) {
				out.push(i + 1);
			}
		});
		return out;
	});

	// Charge le contact initial en mode édition, une seule fois par facture.
	// `reloadFromServer` prend le relais pour les recharges ultérieures.
	$effect(() => {
		if (!invoice) return;
		if (initialContactInvoiceId === invoice.id) return;
		if (selectedContact && selectedContact.id === invoice.contactId) return;
		initialContactInvoiceId = invoice.id;
		getContact(invoice.contactId)
			.then((c) => {
				selectedContact = c;
			})
			.catch((err) => {
				// Rendre l'échec visible : sinon le submit sera bloqué par
				// « Veuillez sélectionner un contact » sans feedback.
				errorMsg = isApiError(err)
					? `Impossible de charger le contact (${err.message}) — sélectionnez-le manuellement`
					: 'Impossible de charger le contact initial — sélectionnez-le manuellement';
			});
	});

	// Story 2.6: Load invoice settings on mount to check if accounts are configured
	// F3+F4 MEDIUM FIX: Sequence counter with effect cleanup.
	// F5 HIGH FIX: Revalidate settings before submit to prevent stale data.
	// P10 FIX: Set loadingSettings=true synchronously at effect entry (not inside
	// the async IIFE). The previous code left a microtask window where a stale
	// loadingSettings=false from the prior run could let the user click Submit
	// during the re-fetch.
	$effect(() => {
		const seq = ++settingsSeq;
		loadingSettings = true;

		(async () => {
			try {
				const settings = await getInvoiceSettings();
				if (seq !== settingsSeq) return;
				invoiceSettings = settings;
			} catch (err) {
				// Ignore errors from stale effect runs
				if (seq !== settingsSeq) return;
				settingsError = isApiError(err)
					? `Erreur lors du chargement des paramètres de facturation (${err.message})`
					: 'Erreur lors du chargement des paramètres de facturation';
			} finally {
				// Only the latest effect clears the flag, so a concurrent newer effect
				// keeps loadingSettings=true while it loads. Stale runs leave it alone.
				if (seq === settingsSeq) {
					loadingSettings = false;
				}
			}
		})();
	});

	function onContactSelect(c: ContactResponse) {
		selectedContact = c;
		// Efface un éventuel message d'erreur « Impossible de charger le contact
		// initial » dès que l'utilisateur sélectionne manuellement un contact.
		errorMsg = '';
		// #245 : pré-remplissages NON destructifs (garde « seulement si vide »).
		// Le libellé serveur (langue du contact) prime sur le texte libre legacy.
		if (!paymentTerms && (c.defaultPaymentTermsLabel || c.defaultPaymentTerms)) {
			paymentTerms = c.defaultPaymentTermsLabel ?? c.defaultPaymentTerms ?? '';
		}
		if (!dueDate && date && c.defaultPaymentTermsDays != null) {
			dueDate = addDaysIso(date, c.defaultPaymentTermsDays);
		}
	}

	function addFreeLine() {
		lines.push({
			description: '',
			quantity: '1',
			unitPrice: '0.00',
			vatRate: DEFAULT_VAT,
			// AC9-bis — site 2/4. `null` = suivre le défaut société ; la Story 16-2
			// y branchera le pré-remplissage depuis la fiche produit.
			revenueAccountId: null,
			_uiKey: nextUiKey(),
		});
	}

	function onProductSelect(p: ProductResponse) {
		lines.push({
			description: p.name,
			quantity: '1',
			unitPrice: p.unitPrice,
			vatRate: p.vatRate,
			// AC9-bis — site 3/4. Idem : 16-2 y branchera le compte du produit.
			revenueAccountId: null,
			_uiKey: nextUiKey(),
		});
	}

	function removeLine(i: number) {
		lines.splice(i, 1);
	}

	function lineTotal(l: LineState): string {
		return computeLineTotal(l.quantity, l.unitPrice);
	}

	let totalAmount = $derived(computeInvoiceTotal(lines));

	// Miroirs backend (`crates/kesh-api/src/routes/limits.rs` et invoices.rs).
	const MAX_LINE_TOTAL = '1000000000000'; // 10¹²
	const MAX_LINES = 200;

	function validateClient(): string | null {
		if (!selectedContact) return 'Veuillez sélectionner un contact';
		if (!date) return 'La date est obligatoire';
		// #245 : comparaison lexicographique valide sur ISO YYYY-MM-DD.
		if (dueDate && dueDate < date) {
			return "L'échéance ne peut pas être antérieure à la date de la facture";
		}
		if (lines.length === 0) return 'Une facture doit contenir au moins une ligne';
		if (lines.length > MAX_LINES) {
			return `Une facture doit contenir au plus ${MAX_LINES} lignes`;
		}
		// Story 16-1b (T5/AC8) — blocage GLOBAL nommant TOUTES les lignes fautives.
		//
		// Volontairement placé AVANT la boucle par ligne : celle-ci `return` à la
		// première erreur, ce qui obligerait l'utilisateur à corriger ligne par
		// ligne en re-soumettant à chaque fois. La persistance est un
		// `createInvoice`/`updateInvoice` UNIQUE portant toutes les lignes — le
		// grain « enregistrer les lignes saines et pas les autres » n'existe pas,
		// donc le message doit donner l'état complet d'un seul coup (même geste que
		// 16-1a D6).
		if (invalidRevenueAccountLines.length > 0) {
			return i18nMsg(
				'invoice-lines-revenue-account-invalid',
				'Compte de produit invalide sur les lignes suivantes : { $lines }',
				{ lines: invalidRevenueAccountLines.join(', ') }
			);
		}
		for (let i = 0; i < lines.length; i++) {
			const l = lines[i];
			if (!l.description.trim()) return `Ligne ${i + 1} : description requise`;
			try {
				const qty = new Big(l.quantity);
				const price = new Big(l.unitPrice);
				if (qty.lte(0)) return `Ligne ${i + 1} : quantité > 0`;
				if (price.lt(0)) return `Ligne ${i + 1} : prix unitaire ≥ 0`;
				if (qty.times(price).gt(MAX_LINE_TOTAL)) {
					return `Ligne ${i + 1} : total de ligne trop élevé`;
				}
			} catch {
				return `Ligne ${i + 1} : valeurs numériques invalides`;
			}
			if (vatOptions.length > 0 && !vatOptions.includes(l.vatRate))
				return `Ligne ${i + 1} : taux TVA invalide`;
		}
		return null;
	}

	async function onSubmit(e: Event) {
		e.preventDefault();
		// Bloquer toute soumission pendant que la modale de conflit est ouverte :
		// l'utilisateur doit d'abord trancher (recharger ou annuler) avant de
		// re-soumettre, sinon on relancerait avec une version toujours périmée.
		// Notifier visiblement — Enter dans un input du formulaire passe ici.
		if (conflictOpen) {
			notifyError('Veuillez d\'abord résoudre le conflit de version avant de ré-enregistrer');
			return;
		}
		// Positionner submitting EN PREMIER pour fermer la fenêtre de double-submit
		// (clics rapides avant que la propagation $state n'atteigne le disabled du bouton).
		if (submitting) return;
		submitting = true;
		errorMsg = '';
		const err = validateClient();
		if (err) {
			errorMsg = err;
			submitting = false;
			return;
		}

		// F5 HIGH FIX: Revalidate settings before submit to catch stale data.
		// If settings were modified since mount, re-fetch to ensure account IDs still exist.
		try {
			const freshSettings = await getInvoiceSettings();
			// Only check account IDs (the two fields that gate invoice creation).
			// String fields (format, journal, template) can be normalized server-side without affecting functionality.
			if (freshSettings.defaultReceivableAccountId !== invoiceSettings?.defaultReceivableAccountId ||
			    freshSettings.defaultRevenueAccountId !== invoiceSettings?.defaultRevenueAccountId) {
				errorMsg = 'Les paramètres de facturation ont changé. Rechargez la page et réessayez.';
				submitting = false;
				return;
			}
			// Settings validated successfully — update to fresh values for use in invoice creation.
			invoiceSettings = freshSettings;
		} catch (err) {
			errorMsg = isApiError(err)
				? `Impossible de valider les paramètres: ${err.message}`
				: 'Impossible de valider les paramètres de facturation';
			submitting = false;
			return;
		}
		try {
			const payload: CreateInvoiceRequest = {
				contactId: selectedContact!.id,
				date,
				dueDate: dueDate || null,
				paymentTerms: paymentTerms.trim() || null,
				projectId,
				lines: lines.map(stripUiKey),
			};
			if (invoice) {
				const req: UpdateInvoiceRequest = { ...payload, version: invoice.version };
				const updated = await updateInvoice(invoice.id, req);
				notifySuccess('Facture modifiée');
				// Review P5 : rediriger vers la vue détail pour exposer le bouton
				// « Valider » aux comptables/admins (Scope §9 — validation depuis
				// l'écran d'édition après sauvegarde draft).
				goto(`/invoices/${updated.id}`);
			} else {
				const created = await createInvoice(payload);
				notifySuccess('Facture créée');
				goto(`/invoices/${created.id}`);
			}
		} catch (err) {
			if (isApiError(err)) {
				if (err.code === 'OPTIMISTIC_LOCK_CONFLICT' && invoice) {
					// Modale dédiée — la spec (T4.3) requiert un dialog avec bouton
					// de rechargement, pas un simple toast.
					conflictOpen = true;
				} else {
					errorMsg = err.message;
					notifyError(err.message);
				}
			} else {
				errorMsg = 'Erreur inconnue';
				notifyError(errorMsg);
			}
		} finally {
			submitting = false;
		}
	}

	async function reloadFromServer() {
		if (!invoice) return;
		// Refuser le reload si un submit est en vol : éviter qu'un PUT stale
		// atterrisse APRÈS que `invoice = fresh` ait refresh à v+n.
		if (submitting) {
			conflictError = 'Enregistrement en cours — réessayez dans un instant';
			return;
		}
		const seq = ++reloadSeq;
		reloading = true;
		conflictError = '';
		try {
			const fresh = await getInvoice(invoice.id);
			if (seq !== reloadSeq) return; // réponse périmée (double-click)
			invoice = fresh;
			date = fresh.date;
			dueDate = fresh.dueDate ?? '';
			paymentTerms = fresh.paymentTerms ?? '';
			projectId = fresh.projectId ?? null;
			lines = fresh.lines.map((l) => ({
				description: l.description,
				quantity: l.quantity,
				unitPrice: l.unitPrice,
				vatRate: l.vatRate,
				// AC9-bis — site 4/4, LE PLUS DANGEREUX. Un oubli ici n'est visible
				// qu'APRÈS un conflit de version : conflit optimiste → modale →
				// « Recharger » → les lignes sont remappées sans compte → toutes les
				// ventilations retombent à `NULL` EN SILENCE → le ré-enregistrement
				// les efface en base. `stripUiKey` étant un spread, rien ne signale
				// l'absence du champ.
				revenueAccountId: l.revenueAccountId,
				_uiKey: nextUiKey(),
			}));
			// Await explicite + fallback UI si le contact devient indisponible.
			// FK RESTRICT en DB garantit qu'un contact référencé existe toujours,
			// mais une race théorique (archivage pendant la requête) peut rendre
			// `getContact` échoué — on préfère un état clair à un contact stale.
			if (fresh.contactId && fresh.contactId > 0) {
				try {
					const c = await getContact(fresh.contactId);
					if (seq !== reloadSeq) return;
					selectedContact = c;
				} catch {
					if (seq !== reloadSeq) return;
					selectedContact = null;
				}
			} else {
				selectedContact = null;
			}
			if (seq !== reloadSeq) return;
			errorMsg = '';
			conflictOpen = false;
			notifySuccess('Facture rechargée');
		} catch (err) {
			if (seq !== reloadSeq) return;
			// Modale reste ouverte avec un message d'erreur local (pas de toast silencieux).
			if (isApiError(err)) {
				conflictError = err.message;
				notifyError(err.message);
			} else {
				conflictError = 'Erreur lors du rechargement';
				notifyError(conflictError);
			}
		} finally {
			if (seq === reloadSeq) reloading = false;
		}
	}
</script>

<form onsubmit={onSubmit} class="space-y-6">
	{#if errorMsg}
		<div class="rounded-md border border-destructive bg-destructive/10 px-3 py-2 text-sm text-destructive">
			{errorMsg}
		</div>
	{/if}

	{#if invoiceSettings && (!invoiceSettings.defaultReceivableAccountId || !invoiceSettings.defaultRevenueAccountId)}
		<div class="rounded-md border border-warning bg-warning/10 px-3 py-2 text-sm text-warning" data-testid="invoice-config-warning">
			{i18nMsg('config-incomplete-title', 'Configuration incomplète')} —
			<a href="/settings/invoicing" class="underline font-medium">{i18nMsg('config-incomplete-link', 'Configurez les comptes de facturation')}</a>
		</div>
	{/if}

	<div class="grid grid-cols-1 gap-4 md:grid-cols-2">
		<div>
			<label class="mb-1 block text-sm font-medium" for="invoice-contact">Contact</label>
			<ContactPicker selected={selectedContact} onSelect={onContactSelect} />
		</div>
		<div>
			<label class="mb-1 block text-sm font-medium" for="invoice-date">Date</label>
			<Input id="invoice-date" type="date" bind:value={date} />
		</div>
		<div>
			<label class="mb-1 block text-sm font-medium" for="invoice-due-date">Échéance</label>
			<Input id="invoice-due-date" type="date" bind:value={dueDate} />
		</div>
		<div>
			<label class="mb-1 block text-sm font-medium" for="invoice-payment-terms">
				Conditions de paiement
			</label>
			<Input
				id="invoice-payment-terms"
				type="text"
				bind:value={paymentTerms}
				placeholder="ex: 30 jours net"
			/>
		</div>
		{#if showProjectField}
			<div>
				<label class="mb-1 block text-sm font-medium" for="invoice-project">
					{i18nMsg('invoice-field-project', 'Projet analytique (optionnel)')}
				</label>
				<!-- Arbre 2 niveaux (racines + sous-projets indentés), pattern 19-3. -->
				<select
					id="invoice-project"
					bind:value={projectId}
					data-testid="invoice-project"
					class="w-full h-9 rounded-md border border-input bg-background px-2 text-sm"
				>
					<option value={null}>{i18nMsg('invoice-project-none', '— Aucun')}</option>
					{#if projectId !== null && !projects.some((p) => p.id === projectId)}
						<!-- Tag hors liste : projet archivé (liste chargée) ou liste
						     indisponible (échec réseau) — libellé neutre dans ce cas.
						     Valeur préservée au round-trip (grandfathering backend). -->
						<option value={projectId}>
							{projectsLoaded
								? i18nMsg('invoice-project-archived', 'Projet archivé')
								: i18nMsg('invoice-project-current', 'Projet actuel')}
						</option>
					{/if}
					{#each projects.filter((p) => p.parentId === null) as root (root.id)}
						<option value={root.id}>{root.code} — {root.name}</option>
						{#each projects.filter((c) => c.parentId === root.id) as child (child.id)}
							<option value={child.id}>&nbsp;&nbsp;↳ {child.code} — {child.name}</option>
						{/each}
					{/each}
				</select>
			</div>
		{/if}
	</div>

	<div>
		<h3 class="mb-2 text-lg font-semibold">Lignes</h3>

		<table class="w-full border-collapse text-sm" data-testid="invoice-lines-table">
			<thead>
				<tr class="border-b border-border text-left">
					<th class="py-2 pr-2">Description</th>
					<th class="py-2 pr-2 w-24">Quantité</th>
					<th class="py-2 pr-2 w-32">Prix unitaire</th>
					<th class="py-2 pr-2 w-28">TVA %</th>
					<!--
						AC11 : les chaînes NOUVELLES passent par `i18nMsg`. Les 5 en-têtes
						existants restent en français codé en dur — les mettre en i18n
						imposerait de les ajouter aux 4 catalogues, hors périmètre (même
						arbitrage qu'AC6-bis côté écran de détail).
					-->
					<th class="py-2 pr-2 w-56">
						{i18nMsg('invoice-line-col-revenue-account', 'Compte de produit')}
					</th>
					<th class="py-2 pr-2 w-32 text-right">Total</th>
					<th class="py-2 w-12"></th>
				</tr>
			</thead>
			<tbody>
				{#each lines as line, i (line._uiKey)}
					<tr class="border-b border-border align-top">
						<td class="py-2 pr-2">
							<Input type="text" bind:value={line.description} />
						</td>
						<td class="py-2 pr-2">
							<Input type="text" inputmode="decimal" bind:value={line.quantity} />
						</td>
						<td class="py-2 pr-2">
							<Input type="text" inputmode="decimal" bind:value={line.unitPrice} />
						</td>
						<td class="py-2 pr-2">
							<select
								bind:value={line.vatRate}
								disabled={vatOptions.length === 0}
								aria-label="Taux TVA"
								data-testid="invoice-line-vat-rate"
								class="h-9 w-full rounded-md border border-border bg-background px-2 text-sm"
							>
								{#each vatOptions as v (v)}
									<option value={v}>{v}%</option>
								{/each}
							</select>
						</td>
						<td class="py-2 pr-2">
							<AccountAutocomplete
								{accounts}
								value={line.revenueAccountId ?? null}
								loadError={accountsLoadError}
								allowClear
								markInvalid
								requiredAccountType="Revenue"
								postableExemptAccountId={invoiceSettings?.defaultRevenueAccountId ?? null}
								placeholder={defaultAccountPlaceholder}
								onSelect={(id) => (line.revenueAccountId = id)}
							/>
						</td>
						<td class="py-2 pr-2 text-right font-mono">
							{formatInvoiceTotal(lineTotal(line))}
						</td>
						<td class="py-2">
							<Button
								type="button"
								variant="ghost"
								size="sm"
								onclick={() => removeLine(i)}
								disabled={lines.length <= 1}
								aria-label="Supprimer la ligne"
							>
								<Trash2 class="h-4 w-4" aria-hidden="true" />
							</Button>
						</td>
					</tr>
				{/each}
			</tbody>
			<tfoot>
				<tr>
					<td colspan="5" class="py-3 text-right font-semibold">Total</td>
					<td class="py-3 text-right font-mono text-lg font-semibold">
						{formatInvoiceTotal(totalAmount)}
					</td>
					<td></td>
				</tr>
			</tfoot>
		</table>

		<div class="mt-3 flex gap-2">
			<Button type="button" variant="outline" onclick={addFreeLine}>
				<Plus class="h-4 w-4" aria-hidden="true" />
				Ligne libre
			</Button>
			<Button type="button" variant="outline" onclick={() => (productPickerOpen = true)}>
				<Package class="h-4 w-4" aria-hidden="true" />
				Depuis catalogue
			</Button>
		</div>
	</div>

	<div class="flex justify-end gap-2">
		<Button type="button" variant="outline" onclick={() => goto('/invoices')}>Annuler</Button>
		<Button data-testid="create-invoice-button"
			type="submit"
			disabled={submitting || conflictOpen || loadingSettings || (invoiceSettings && !invoiceSettings.defaultReceivableAccountId) || (invoiceSettings && !invoiceSettings.defaultRevenueAccountId) || invalidRevenueAccountLines.length > 0}
			title={loadingSettings ? i18nMsg('common-loading', 'Chargement...') : (invoiceSettings && (!invoiceSettings.defaultReceivableAccountId || !invoiceSettings.defaultRevenueAccountId) ? i18nMsg('invoice-settings-required', "Configurez d'abord les comptes de facturation dans les paramètres") : (invalidRevenueAccountLines.length > 0 ? i18nMsg('invoice-lines-revenue-account-invalid', 'Compte de produit invalide sur les lignes suivantes : { $lines }', { lines: invalidRevenueAccountLines.join(', ') }) : undefined))}
		>
			{invoice ? 'Enregistrer' : 'Créer la facture'}
		</Button>
	</div>
</form>

<ProductPicker
	open={productPickerOpen}
	onOpenChange={(o) => (productPickerOpen = o)}
	onSelect={onProductSelect}
/>

<Dialog.Root
	open={conflictOpen}
	onOpenChange={(open) => {
		conflictOpen = open;
		// Reset le message d'erreur quand la modale se referme (Escape, overlay, Annuler).
		if (!open) conflictError = '';
	}}
>
	<Dialog.Content>
		<Dialog.Header>
			<Dialog.Title>Conflit de version</Dialog.Title>
		</Dialog.Header>
		<p class="text-sm">
			Cette facture a été modifiée ailleurs depuis votre ouverture.
			Rechargez la version actuelle pour continuer — vos modifications locales non
			enregistrées seront perdues.
		</p>
		{#if conflictError}
			<div class="rounded-md border border-destructive bg-destructive/10 px-3 py-2 text-sm text-destructive">
				{conflictError}
			</div>
		{/if}
		<Dialog.Footer>
			<Button
				variant="outline"
				onclick={() => {
					conflictOpen = false;
					conflictError = '';
				}}
			>
				Annuler
			</Button>
			<Button onclick={reloadFromServer} disabled={reloading}>Recharger</Button>
		</Dialog.Footer>
	</Dialog.Content>
</Dialog.Root>
