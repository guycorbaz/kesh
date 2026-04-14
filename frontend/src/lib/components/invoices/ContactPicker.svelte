<!--
  ContactPicker (Story 5.1) : combobox cherchable de contacts actifs.
  Implémentation simple (input + dropdown positionné), debounce 300ms.
-->
<script lang="ts">
	import { listContacts } from '$lib/features/contacts/contacts.api';
	import type { ContactResponse } from '$lib/features/contacts/contacts.types';
	import { Input } from '$lib/components/ui/input';
	import { onMount } from 'svelte';

	type Props = {
		selected: ContactResponse | null;
		onSelect: (c: ContactResponse) => void;
		placeholder?: string;
	};
	let { selected, onSelect, placeholder = 'Rechercher un contact…' }: Props = $props();

	// ID unique par instance — généré au mount côté client pour garantir
	// la stabilité hydration (évite qu'un rendu SSR produise une valeur
	// différente du client). Suit le pattern combobox ARIA : les IDs doivent
	// exister au premier render, donc on initialise immédiatement avec
	// `crypto.randomUUID` (supporté par tous les runtimes cibles de Kesh :
	// navigateurs modernes + Node 18+ pour SSR).
	const instanceId = crypto.randomUUID().slice(0, 8);
	const listboxId = `contact-picker-list-${instanceId}`;
	const optionId = (contactId: number) => `contact-picker-opt-${instanceId}-${contactId}`;

	let query = $state('');
	let open = $state(false);
	let loading = $state(false);
	let results = $state<ContactResponse[]>([]);
	// -1 = rien de surligné (aucun résultat). Pattern standard combobox ARIA.
	let highlighted = $state(-1);
	let inputEl: HTMLInputElement | null = $state(null);
	let debounceHandle: ReturnType<typeof setTimeout> | null = null;
	let searchSeq = 0;

	// Synchronisation one-shot du nom affiché dans l'input à chaque fois qu'un
	// nouveau `selected` non-null apparaît. Ne pas binder `query` directement
	// sur `selected.name` : l'utilisateur doit pouvoir effacer et chercher un
	// autre contact (le `$effect` ne doit pas recoller le nom à chaque frappe).
	// Reset du flag quand `selected` redevient null (ex. reload après conflit).
	let initialSyncDone = $state(false);
	$effect(() => {
		if (!selected) {
			initialSyncDone = false;
			return;
		}
		if (!initialSyncDone && query === '') {
			query = selected.name;
			initialSyncDone = true;
		}
	});

	async function runSearch(q: string) {
		const seq = ++searchSeq;
		loading = true;
		try {
			const r = await listContacts({ search: q, limit: 50, includeArchived: false });
			if (seq !== searchSeq) return;
			results = r.items;
			highlighted = r.items.length > 0 ? 0 : -1;
		} catch {
			if (seq !== searchSeq) return;
			results = [];
			highlighted = -1;
		} finally {
			if (seq === searchSeq) loading = false;
		}
	}

	function onInput(e: Event) {
		query = (e.target as HTMLInputElement).value;
		open = true;
		if (debounceHandle) clearTimeout(debounceHandle);
		debounceHandle = setTimeout(() => runSearch(query.trim()), 300);
	}

	function pick(c: ContactResponse) {
		onSelect(c);
		query = c.name;
		open = false;
	}

	function onKeydown(e: KeyboardEvent) {
		if (!open) return;
		if (e.key === 'ArrowDown') {
			e.preventDefault();
			if (results.length > 0) {
				highlighted = Math.min(results.length - 1, Math.max(0, highlighted) + 1);
			}
		} else if (e.key === 'ArrowUp') {
			e.preventDefault();
			if (results.length > 0) {
				highlighted = Math.max(0, highlighted - 1);
			}
		} else if (e.key === 'Enter' && highlighted >= 0 && results[highlighted]) {
			e.preventDefault();
			pick(results[highlighted]);
		} else if (e.key === 'Escape') {
			e.preventDefault();
			open = false;
			// Retour de focus sur l'input pour que l'utilisateur puisse continuer à taper
			// sans avoir à cliquer — pattern ARIA combobox standard.
			inputEl?.focus();
		}
	}

	function onBlur() {
		// Laisse le click sur l'élément se déclencher avant fermeture.
		setTimeout(() => (open = false), 150);
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
			if (results.length === 0) runSearch('');
		}}
		onblur={onBlur}
		onkeydown={onKeydown}
		{placeholder}
		aria-expanded={open}
		aria-controls={listboxId}
		aria-activedescendant={open && highlighted >= 0 && results[highlighted]
			? optionId(results[highlighted].id)
			: undefined}
		role="combobox"
		autocomplete="off"
	/>
	{#if open}
		<ul
			id={listboxId}
			role="listbox"
			class="absolute z-20 mt-1 max-h-64 w-full overflow-auto rounded-md border border-border bg-surface shadow"
		>
			{#if loading}
				<li class="px-3 py-2 text-sm text-text-muted">Chargement…</li>
			{:else if results.length === 0}
				<li class="px-3 py-2 text-sm text-text-muted">Aucun contact</li>
			{:else}
				{#each results as c, i (c.id)}
					<li
						id={optionId(c.id)}
						role="option"
						aria-selected={i === highlighted}
						class="cursor-pointer px-3 py-2 text-sm hover:bg-muted {i ===
						highlighted
							? 'bg-muted'
							: ''}"
						onmousedown={() => pick(c)}
					>
						<div class="font-medium">{c.name}</div>
						{#if c.ideNumber}
							<div class="text-xs text-text-muted">{c.ideNumber}</div>
						{/if}
					</li>
				{/each}
			{/if}
		</ul>
	{/if}
</div>
