<!--
  ContactPicker (Story 5.1) : combobox cherchable de contacts actifs.
  Implémentation simple (input + dropdown positionné), debounce 300ms.

  Story v0190-correction (#1162) : le champ ne montre jamais un client autre
  que celui que le parent enregistrera. Le texte tapé est un BROUILLON, jeté au
  départ du champ ; *Entrée* ne laisse partir le formulaire englobant que sur un
  champ non touché ; une surbrillance ne vaut choix que DÉSIGNÉE (par la saisie
  ou par une flèche). Règle reprise d'`AccountAutocomplete` (Story 36-1), avec
  ses différences : liste servie par le serveur (anti-rebond, réponse attendue
  avant qu'*Entrée* ne désigne), pas d'effacement (la facture exige un client).
-->
<script lang="ts">
	import { listContacts } from '$lib/features/contacts/contacts.api';
	import type { ContactResponse } from '$lib/features/contacts/contacts.types';
	import { Input } from '$lib/components/ui/input';
	// Couplage transverse assumé (fiche v0190-correction, D3) : une seule source
	// des classes de désignation et de survol pour les deux sélecteurs à texte tapé.
	import {
		CLASSE_DESIGNEE,
		CLASSE_SURVOL
	} from '$lib/features/journal-entries/account-autocomplete-classes';
	import { onMount, untrack } from 'svelte';

	type Props = {
		selected: ContactResponse | null;
		onSelect: (c: ContactResponse) => void;
		placeholder?: string;
	};
	let { selected, onSelect, placeholder = 'Rechercher un contact…' }: Props = $props();

	// ID unique par instance pour le pattern combobox ARIA (les IDs doivent
	// exister au premier render). On utilise le rune `$props.id()` de Svelte 5,
	// unique par instance et stable en hydration. NE PAS utiliser
	// `crypto.randomUUID()` : cette API n'est disponible qu'en contexte
	// sécurisé (HTTPS ou http://localhost) et lève `TypeError: crypto.randomUUID
	// is not a function` sur un déploiement HTTP LAN (cf. NAS Synology), ce qui
	// crashe au rendu toute page consommant ce composant (#145).
	const instanceId = $props.id();
	const listboxId = `contact-picker-list-${instanceId}`;
	const optionId = (contactId: number) => `contact-picker-opt-${instanceId}-${contactId}`;

	/**
	 * Texte en cours de frappe ; `null` = aucune édition (D1). Le champ montre le
	 * brouillon s'il existe, sinon le client retenu : une fois quitté, il ne peut
	 * plus contredire ce que le parent enregistrera.
	 */
	let draft = $state<string | null>(null);
	const query = $derived(draft ?? selected?.name ?? '');

	let open = $state(false);
	let loading = $state(false);
	/**
	 * Vrai de la frappe à la réponse de la recherche lancée pour le brouillon
	 * ACTUEL (D2, C-v0190c-12) : couvre l'anti-rebond, où `loading` est encore faux.
	 */
	let pending = $state(false);
	let results = $state<ContactResponse[]>([]);
	/**
	 * Option désignée ; `-1` = aucune (D3). Invariant : `≥ 0` seulement si une
	 * option a été désignée — par la saisie (brouillon non blanc, à l'arrivée de
	 * la réponse de sa recherche, liste ouverte) ou par une flèche. Liste fermée,
	 * il vaut `-1` : c'est ce qui fait partir les flèches du premier / du dernier.
	 */
	let highlighted = $state(-1);
	let inputEl: HTMLInputElement | null = $state(null);
	let debounceHandle: ReturnType<typeof setTimeout> | null = null;
	let searchSeq = 0;

	/** Une recherche pour la saisie affichée est encore attendue. */
	const searching = $derived(pending || loading);

	/**
	 * Abandon de la recherche en vol et de l'anti-rebond : la réponse tardive
	 * est ignorée (`searchSeq`), « Chargement… » ne survit pas, la liste d'une
	 * recherche abandonnée n'est plus proposée. Commun au `blur`, au choix, au
	 * brouillon blanc jeté par *Entrée* et au changement de client.
	 * ⚠️ *Échap* ne l'appelle PAS (validation P8-1, P8-2) : il ferme la liste et
	 * laisse la recherche aboutir, sans désignation puisque la liste est fermée.
	 */
	function abandonSearch() {
		if (debounceHandle) {
			clearTimeout(debounceHandle);
			debounceHandle = null;
		}
		searchSeq++;
		pending = false;
		loading = false;
		results = [];
	}

	// Un client changé par le parent (choix, 409 rechargé, *Réinitialiser* d'un
	// filtre) jette le brouillon et purge la liste : le champ montre le nouveau
	// client, et aucune désignation de l'ancienne liste ne survit (D1, V2-5,
	// P3-1). L'effet ne suit que l'identifiant ; le reste est lu sans suivi.
	const selectedId = $derived(selected?.id ?? null);
	let lastSelectedId: number | null | undefined = undefined;
	$effect(() => {
		const id = selectedId;
		untrack(() => {
			if (lastSelectedId === undefined) {
				lastSelectedId = id;
				return;
			}
			if (id === lastSelectedId) return;
			lastSelectedId = id;
			abandonSearch();
			draft = null;
			open = false;
			highlighted = -1;
		});
	});

	/**
	 * Recherche. `fleche` : relance par une flèche sur une liste vide (D3, P6-1)
	 * — la direction appartient à CETTE recherche, capturée avec `seq` ; une
	 * recherche abandonnée l'emporte avec elle.
	 */
	async function runSearch(q: string, fleche?: 'down' | 'up') {
		const seq = ++searchSeq;
		loading = true;
		let items: ContactResponse[];
		try {
			const r = await listContacts({ search: q, limit: 50, includeArchived: false });
			if (seq !== searchSeq) return;
			items = r.items;
		} catch {
			if (seq !== searchSeq) return;
			// Un échec vaut réponse (V2-4) : « Aucun contact », sans attente fantôme.
			items = [];
		} finally {
			if (seq === searchSeq) {
				loading = false;
				pending = false;
			}
		}
		results = items;
		// Une réponse arrivée liste fermée (*Échap*) ne désigne rien (P6-4, P7-1).
		let h = -1;
		if (open && items.length > 0) {
			if (fleche === 'down') h = 0;
			else if (fleche === 'up') h = items.length - 1;
			// Désignation par la saisie, sur le brouillon COURANT (V2-4) ; la
			// recherche du focus (aucun brouillon) ne désigne rien (P5-2).
			else if (draft !== null && draft.trim() !== '') h = 0;
		}
		highlighted = h;
	}

	function onInput(e: Event) {
		draft = (e.target as HTMLInputElement).value;
		open = true;
		// Toute frappe invalide la recherche en vol et la désignation (V1-1, V2-2) :
		// *Entrée* n'agit qu'une fois arrivée la réponse pour le texte affiché.
		searchSeq++;
		pending = true;
		highlighted = -1;
		if (debounceHandle) clearTimeout(debounceHandle);
		debounceHandle = setTimeout(() => {
			debounceHandle = null;
			runSearch((draft ?? '').trim());
		}, 300);
	}

	function pick(c: ContactResponse) {
		abandonSearch();
		draft = null;
		open = false;
		highlighted = -1;
		onSelect(c);
	}

	function onArrow(e: KeyboardEvent, direction: 'down' | 'up') {
		e.preventDefault();
		open = true;
		// Pendant la recherche, la flèche ouvre sans rien désigner (V2-2).
		if (searching) {
			highlighted = -1;
			return;
		}
		if (results.length === 0) {
			// Relance avec le texte de la frappe, non `''` d'office (P4-2).
			runSearch((draft ?? '').trim(), direction);
			return;
		}
		// Depuis « rien de désigné » : le premier (bas) ou le dernier (haut).
		if (direction === 'down') {
			highlighted = highlighted < 0 ? 0 : Math.min(highlighted + 1, results.length - 1);
		} else {
			highlighted = highlighted < 0 ? results.length - 1 : Math.max(highlighted - 1, 0);
		}
	}

	/**
	 * *Entrée* (D2) : (1) option désignée, liste ouverte, recherche terminée →
	 * elle est choisie ; (2) un brouillon est affiché → jamais d'envoi : blanc, il
	 * est jeté ; sinon la liste s'ouvre et, recherche terminée, le premier est
	 * désigné ; (3) champ non touché → le formulaire part avec le client affiché.
	 */
	function onEnter(e: KeyboardEvent) {
		// Composition IME : cette Entrée valide le mot composé, pas le champ
		// (`keyCode === 229` : WebKit, cf. `AccountAutocomplete`).
		if (e.isComposing || e.keyCode === 229) return;
		// Touche maintenue : un seul geste.
		if (e.repeat) {
			e.preventDefault();
			return;
		}
		const designe = open && !searching && highlighted >= 0 ? results[highlighted] : undefined;
		if (designe) {
			e.preventDefault();
			pick(designe);
			return;
		}
		if (draft === null) return;

		e.preventDefault();
		if (draft.trim() === '') {
			// Brouillon blanc (E5 de la 36-1) : jeté, le client retenu reparaît.
			abandonSearch();
			draft = null;
			open = false;
			highlighted = -1;
			return;
		}
		open = true;
		if (!searching && results.length > 0) highlighted = 0;
	}

	function onKeydown(e: KeyboardEvent) {
		if (e.key === 'ArrowDown') {
			onArrow(e, 'down');
		} else if (e.key === 'ArrowUp') {
			onArrow(e, 'up');
		} else if (e.key === 'Enter') {
			onEnter(e);
		} else if (e.key === 'Escape') {
			if (!open) return;
			e.preventDefault();
			// Ferme sans abandonner la recherche (P8-1, P8-2) : sa réponse
			// s'applique, liste fermée, sans rien désigner.
			open = false;
			highlighted = -1;
			// Retour de focus sur l'input pour que l'utilisateur puisse continuer à taper
			// sans avoir à cliquer — pattern ARIA combobox standard.
			inputEl?.focus();
		}
	}

	/**
	 * `blur` immédiat (D1) : le brouillon est jeté, le champ réaffiche le client
	 * retenu. Un clic dans la liste ne fait pas perdre le focus (`preventDefault`
	 * au `mousedown`) : le délai d'origine n'a plus d'objet.
	 */
	function onBlur() {
		abandonSearch();
		draft = null;
		open = false;
		highlighted = -1;
	}

	onMount(() => {
		return () => {
			if (debounceHandle) clearTimeout(debounceHandle);
		};
	});
</script>

<div class="relative">
	<Input
		bind:ref={inputEl}
		type="text"
		value={query}
		oninput={onInput}
		onfocus={() => {
			open = true;
			// Ouvrir n'est pas désigner (#1165).
			highlighted = -1;
			if (results.length === 0 && !searching) runSearch('');
		}}
		onblur={onBlur}
		onkeydown={onKeydown}
		{placeholder}
		aria-expanded={open}
		aria-controls={listboxId}
		aria-activedescendant={open && !searching && highlighted >= 0 && results[highlighted]
			? optionId(results[highlighted].id)
			: undefined}
		role="combobox"
		autocomplete="off"
	/>
	{#if open}
		<!-- `preventDefault` au `mousedown` (V2-7) : le champ garde le focus, même
		     au clic sur la barre de défilement de la liste. -->
		<ul
			id={listboxId}
			role="listbox"
			class="absolute z-20 mt-1 max-h-64 w-full overflow-auto rounded-md border border-border bg-surface shadow"
			onmousedown={(e) => e.preventDefault()}
		>
			{#if searching}
				<li class="px-3 py-2 text-sm text-text-muted">Chargement…</li>
			{:else if results.length === 0}
				<li class="px-3 py-2 text-sm text-text-muted">Aucun contact</li>
			{:else}
				{#each results as c, i (c.id)}
					<li
						id={optionId(c.id)}
						role="option"
						aria-selected={i === highlighted}
						class={[
							'group cursor-pointer px-3 py-2 text-sm',
							i === highlighted ? CLASSE_DESIGNEE : CLASSE_SURVOL
						]}
						onmousedown={(e) => {
							e.preventDefault();
							pick(c);
						}}
					>
						<div class="font-medium">{c.name}</div>
						{#if c.ideNumber}
							<!-- Lisible dans les deux états (V2-6, P3-5) ; aucune variante
							     de survol sur l'option désignée (P4-1). -->
							<div
								class={i === highlighted
									? 'text-xs text-primary-foreground'
									: 'text-xs text-text-muted group-hover:text-text'}
							>
								{c.ideNumber}
							</div>
						{/if}
					</li>
				{/each}
			{/if}
		</ul>
	{/if}
</div>
