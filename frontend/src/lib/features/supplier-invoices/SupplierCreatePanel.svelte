<!--
  Story 30-2b (#685, AC15, C-30-2-8) — créer le fournisseur sans quitter la saisie.

  Un panneau de la page, au minimum : type (Entreprise par défaut), nom (pré-rempli du
  créancier du QR), adresse structurée (pré-remplie pour un QR de type `S` seulement —
  les lignes combinées d'un type `K` ne se découpent pas sans deviner), fournisseur posé.
  Le signal « contacts au nom proche » du socle 22-2a s'affiche AVANT l'enregistrement :
  **signaler, jamais bloquer** (deux fournisseurs peuvent légitimement porter des noms
  proches).

  ⚠️ Ce n'est pas un `<form>` : il vit à l'intérieur du formulaire de la facture, et un
  formulaire imbriqué est invalide en HTML. Ses boutons sont `type="button"`, et la
  touche Entrée dans un de ses champs crée le fournisseur au lieu de soumettre la
  facture (`onPanelKeydown`, C-30-2-41).

  *Créer* appelle `createContact` ; le parent ajoute le contact à sa liste et le
  SÉLECTIONNE — c'est le geste humain qui le désigne. Une erreur (IDE en double,
  validation) s'affiche ici sans rien perdre ; *Annuler* ne crée rien.
-->
<script lang="ts">
	import { onDestroy } from 'svelte';
	import { createContact, listContacts } from '$lib/features/contacts/contacts.api';
	import {
		emptyAddress,
		type ContactResponse,
		type ContactType,
		type StructuredAddress,
	} from '$lib/features/contacts/contacts.types';
	import { describeProches, probeTerm, rank } from '$lib/features/contacts/duplicate-probe';
	import { isApiError } from '$lib/shared/utils/api-error';
	import { i18nMsg } from '$lib/shared/utils/i18n.svelte';

	interface Props {
		/** Nom pré-rempli (créancier du QR), ou `''`. */
		initialName: string;
		/** Adresse pré-remplie (QR de type `S`), ou `null`. */
		initialAddress: StructuredAddress | null;
		/** Le contact vient d'être créé : le parent l'ajoute et le sélectionne. */
		onCreated: (contact: ContactResponse) => void;
		onCancel: () => void;
	}

	let { initialName, initialAddress, onCreated, onCancel }: Props = $props();

	// Valeurs de départ lues une fois à l'ouverture : le panneau est recréé à chaque
	// ouverture, et une saisie en cours ne doit pas être écrasée par le parent.
	let contactType = $state<ContactType>('Entreprise');
	// svelte-ignore state_referenced_locally
	let name = $state(initialName);
	let firstName = $state('');
	let lastName = $state('');
	// svelte-ignore state_referenced_locally
	let address = $state<StructuredAddress>({ ...(initialAddress ?? emptyAddress()) });
	let saving = $state(false);
	let error = $state('');

	let proches = $state<ContactResponse[]>([]);
	const descriptions = $derived(describeProches(proches));
	let probeSeq = 0;
	let probeTimer: ReturnType<typeof setTimeout> | null = null;

	/** Sonde « nom proche » (patron de la page des contacts) — muette en cas d'échec. */
	async function runProbe() {
		const { normalized, armed } = probeTerm(contactType, name, firstName, lastName);
		if (!armed) {
			probeSeq++;
			proches = [];
			return;
		}
		const seq = ++probeSeq;
		try {
			const res = await listContacts({ search: normalized, limit: 20, includeArchived: false });
			if (seq !== probeSeq) return;
			proches = rank(res.items, normalized).slice(0, 5);
		} catch (e) {
			// Une sonde n'est pas une action de l'utilisateur : pas de rapport d'échec,
			// mais une trace — « la sonde a planté » et « aucun doublon » doivent rester
			// distinguables pour qui débogue.
			console.error('[sonde nom fournisseur] échec', e);
			if (seq === probeSeq) proches = [];
		}
	}

	function scheduleProbe() {
		if (probeTimer) clearTimeout(probeTimer);
		probeTimer = setTimeout(() => void runProbe(), 300);
	}

	// Sonde initiale : le nom pré-rempli du créancier peut déjà désigner un contact.
	scheduleProbe();
	onDestroy(() => {
		if (probeTimer) clearTimeout(probeTimer);
	});

	function nameMissing(): boolean {
		return contactType === 'Personne'
			? !firstName.trim() || !lastName.trim()
			: !name.trim();
	}

	/**
	 * Entrée dans un champ du panneau crée le FOURNISSEUR, jamais la facture
	 * (revue P1 de la 30-2b, B1 = E1 = A1 ; C-30-2-41).
	 *
	 * Le panneau vit dans le `<form>` de la facture : sans ce garde, Entrée dans un
	 * `<input>` déclenche la soumission implicite de ce formulaire — la facture
	 * s'enregistrait sur le fournisseur déjà sélectionné pendant que l'on en tapait un
	 * autre, faux rattachement muet. `preventDefault()` sur le `keydown` annule cette
	 * soumission implicite ; seules les cibles `<input>` sont visées (Entrée sur un
	 * bouton garde son clic, un `<select>` ne soumet pas), et une composition IME en
	 * cours n'est pas une validation.
	 */
	function onPanelKeydown(e: KeyboardEvent) {
		if (e.key !== 'Enter' || e.isComposing) return;
		if (!(e.target instanceof HTMLInputElement)) return;
		e.preventDefault();
		if (!saving) void create();
	}

	async function create() {
		error = '';
		if (nameMissing()) {
			error = i18nMsg('supplier-invoices-new-supplier-err-name', 'Le nom est requis.');
			return;
		}
		saving = true;
		try {
			const created = await createContact({
				contactType,
				name: contactType === 'Personne' ? `${firstName.trim()} ${lastName.trim()}` : name.trim(),
				firstName: contactType === 'Personne' ? firstName.trim() : null,
				lastName: contactType === 'Personne' ? lastName.trim() : null,
				isClient: false,
				isSupplier: true,
				addressStructured: { ...address },
			});
			onCreated(created);
		} catch (err) {
			error =
				(isApiError(err) && err.message) ||
				i18nMsg('supplier-invoices-err-generic', 'Erreur inattendue.');
		} finally {
			saving = false;
		}
	}
</script>

<!-- svelte-ignore a11y_no_noninteractive_element_interactions — le `keydown` n'ajoute
     aucune interaction au groupe : il intercepte la touche Entrée remontée de ses champs
     (`onPanelKeydown`), qui restent les seuls éléments interactifs. -->
<div
	class="space-y-3 rounded border border-primary/40 bg-muted/30 p-4"
	data-testid="supplier-new-panel"
	role="group"
	aria-label={i18nMsg('supplier-invoices-new-supplier', 'Nouveau fournisseur')}
	onkeydown={onPanelKeydown}
>
	<div class="text-sm font-medium">{i18nMsg('supplier-invoices-new-supplier', 'Nouveau fournisseur')}</div>
	<div class="grid grid-cols-2 gap-3">
		<label class="block text-sm">
			{i18nMsg('supplier-invoices-new-supplier-type', 'Type')}
			<select
				class="mt-1 w-full rounded border px-2 py-1"
				bind:value={contactType}
				onchange={scheduleProbe}
				data-testid="supplier-new-type"
			>
				<option value="Entreprise">{i18nMsg('supplier-invoices-new-supplier-type-company', 'Entreprise')}</option>
				<option value="Personne">{i18nMsg('supplier-invoices-new-supplier-type-person', 'Personne')}</option>
			</select>
		</label>
		{#if contactType === 'Personne'}
			<label class="block text-sm">
				{i18nMsg('supplier-invoices-new-supplier-first-name', 'Prénom')}
				<input class="mt-1 w-full rounded border px-2 py-1" bind:value={firstName} oninput={scheduleProbe} />
			</label>
			<label class="block text-sm">
				{i18nMsg('supplier-invoices-new-supplier-last-name', 'Nom de famille')}
				<input class="mt-1 w-full rounded border px-2 py-1" bind:value={lastName} oninput={scheduleProbe} />
			</label>
		{:else}
			<label class="block text-sm">
				{i18nMsg('supplier-invoices-new-supplier-name', 'Nom')}
				<input
					class="mt-1 w-full rounded border px-2 py-1"
					data-testid="supplier-new-name"
					bind:value={name}
					oninput={scheduleProbe}
				/>
			</label>
		{/if}
		<label class="block text-sm">
			{i18nMsg('supplier-invoices-new-supplier-street', 'Rue')}
			<input class="mt-1 w-full rounded border px-2 py-1" data-testid="supplier-new-street" bind:value={address.street} />
		</label>
		<label class="block text-sm">
			{i18nMsg('supplier-invoices-new-supplier-building', 'N°')}
			<input class="mt-1 w-full rounded border px-2 py-1" bind:value={address.building} />
		</label>
		<label class="block text-sm">
			{i18nMsg('supplier-invoices-new-supplier-postal-code', 'NPA')}
			<input class="mt-1 w-full rounded border px-2 py-1" data-testid="supplier-new-postal-code" bind:value={address.postalCode} />
		</label>
		<label class="block text-sm">
			{i18nMsg('supplier-invoices-new-supplier-city', 'Localité')}
			<input class="mt-1 w-full rounded border px-2 py-1" data-testid="supplier-new-city" bind:value={address.city} />
		</label>
		<label class="block text-sm">
			{i18nMsg('supplier-invoices-new-supplier-country', 'Pays')}
			<input class="mt-1 w-full rounded border px-2 py-1" maxlength="2" bind:value={address.country} />
		</label>
	</div>

	{#if proches.length > 0}
		<div class="rounded border border-warning bg-warning-soft p-2 text-sm" data-testid="supplier-new-proches">
			<p>
				{i18nMsg(
					'supplier-invoices-new-supplier-proches',
					'Des contacts au nom proche existent déjà — vérifiez avant de créer :',
				)}
			</p>
			<ul class="ml-4 list-disc">
				{#each proches as p, i (p.id)}
					<li>{p.name}{descriptions[i]}</li>
				{/each}
			</ul>
		</div>
	{/if}

	{#if error}
		<p class="text-sm text-destructive" role="alert" data-testid="supplier-new-error">{error}</p>
	{/if}

	<div class="flex gap-2">
		<button
			type="button"
			class="rounded bg-primary px-3 py-1 text-sm text-primary-foreground disabled:opacity-60"
			data-testid="supplier-new-create"
			disabled={saving}
			onclick={create}
		>
			{saving ? '…' : i18nMsg('supplier-invoices-new-supplier-create', 'Créer le fournisseur')}
		</button>
		<button type="button" class="rounded border px-3 py-1 text-sm" data-testid="supplier-new-cancel" onclick={onCancel}>
			{i18nMsg('common-cancel', 'Annuler')}
		</button>
	</div>
</div>
