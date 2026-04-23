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

	type Props = { invoice?: InvoiceResponse | null };
	let { invoice = null }: Props = $props();

	const VAT_OPTIONS = ['0.00', '2.60', '3.80', '8.10'];
	const DEFAULT_VAT = '8.10';

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
	$effect(() => {
		const seq = ++settingsSeq;

		(async () => {
			try {
				loadingSettings = true;
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
				// F3 MEDIUM FIX: Always clear loading flag (even if seq mismatch)
				// to prevent indefinite disabled state. If new effect is running,
				// it will set loadingSettings=true again.
				loadingSettings = false;
			}
		})();
	});

	function onContactSelect(c: ContactResponse) {
		selectedContact = c;
		// Efface un éventuel message d'erreur « Impossible de charger le contact
		// initial » dès que l'utilisateur sélectionne manuellement un contact.
		errorMsg = '';
		if (!paymentTerms && c.defaultPaymentTerms) {
			paymentTerms = c.defaultPaymentTerms;
		}
	}

	function addFreeLine() {
		lines.push({
			description: '',
			quantity: '1',
			unitPrice: '0.00',
			vatRate: DEFAULT_VAT,
			_uiKey: nextUiKey(),
		});
	}

	function onProductSelect(p: ProductResponse) {
		lines.push({
			description: p.name,
			quantity: '1',
			unitPrice: p.unitPrice,
			vatRate: p.vatRate,
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
		if (lines.length === 0) return 'Une facture doit contenir au moins une ligne';
		if (lines.length > MAX_LINES) {
			return `Une facture doit contenir au plus ${MAX_LINES} lignes`;
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
			if (!VAT_OPTIONS.includes(l.vatRate)) return `Ligne ${i + 1} : taux TVA invalide`;
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
			lines = fresh.lines.map((l) => ({
				description: l.description,
				quantity: l.quantity,
				unitPrice: l.unitPrice,
				vatRate: l.vatRate,
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
		<div class="rounded-md border border-warning bg-warning/10 px-3 py-2 text-sm text-warning">
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
	</div>

	<div>
		<div class="mb-2 flex items-center justify-between">
			<h3 class="text-lg font-semibold">Lignes</h3>
			<div class="flex gap-2">
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

		<table class="w-full border-collapse text-sm">
			<thead>
				<tr class="border-b border-border text-left">
					<th class="py-2 pr-2">Description</th>
					<th class="py-2 pr-2 w-24">Quantité</th>
					<th class="py-2 pr-2 w-32">Prix unitaire</th>
					<th class="py-2 pr-2 w-28">TVA %</th>
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
								class="h-9 w-full rounded-md border border-border bg-background px-2 text-sm"
							>
								{#each VAT_OPTIONS as v (v)}
									<option value={v}>{v}%</option>
								{/each}
							</select>
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
					<td colspan="4" class="py-3 text-right font-semibold">Total</td>
					<td class="py-3 text-right font-mono text-lg font-semibold">
						{formatInvoiceTotal(totalAmount)}
					</td>
					<td></td>
				</tr>
			</tfoot>
		</table>
	</div>

	<div class="flex justify-end gap-2">
		<Button type="button" variant="outline" onclick={() => goto('/invoices')}>Annuler</Button>
		<Button
			type="submit"
			disabled={submitting || conflictOpen || loadingSettings || (invoiceSettings && !invoiceSettings.defaultReceivableAccountId) || (invoiceSettings && !invoiceSettings.defaultRevenueAccountId)}
			title={loadingSettings ? i18nMsg('common-loading', 'Chargement...') : (invoiceSettings && (!invoiceSettings.defaultReceivableAccountId || !invoiceSettings.defaultRevenueAccountId) ? i18nMsg('invoice-settings-required', "Configurez d'abord les comptes de facturation dans les paramètres") : undefined)}
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
