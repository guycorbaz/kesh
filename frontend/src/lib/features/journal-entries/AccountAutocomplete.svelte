<script lang="ts">
	import { Input } from '$lib/components/ui/input';
	import { i18nMsg } from '$lib/features/onboarding/onboarding.svelte';
	import type { AccountResponse } from '$lib/features/accounts/accounts.types';

	interface Props {
		accounts: AccountResponse[];
		value: number | null;
		loadError?: boolean;
		disabled?: boolean;
		onSelect: (id: number | null) => void;
	}

	let { accounts, value, loadError = false, disabled = false, onSelect }: Props = $props();

	let query = $state('');
	let open = $state(false);
	let highlightIndex = $state(0);

	// Lorsqu'une valeur arrive depuis l'extérieur, afficher le compte correspondant.
	$effect(() => {
		if (value !== null && !loadError) {
			const acc = accounts.find((a) => a.id === value);
			if (acc) {
				query = `${acc.number} — ${acc.name}`;
			}
		} else if (value === null) {
			query = '';
		}
	});

	const active = $derived(accounts.filter((a) => a.active));

	const filtered = $derived.by(() => {
		if (loadError) return [];
		const q = query.trim().toLowerCase();
		if (q === '') return active.slice(0, 20);
		return active
			.filter(
				(a) =>
					a.number.toLowerCase().startsWith(q) ||
					a.name.toLowerCase().includes(q)
			)
			.slice(0, 20);
	});

	function handleInput(e: Event) {
		const target = e.target as HTMLInputElement;
		query = target.value;
		open = true;
		highlightIndex = 0;

		// En mode fallback (loadError), l'utilisateur saisit un ID numérique directement.
		if (loadError) {
			const n = Number(target.value);
			onSelect(Number.isFinite(n) && n > 0 ? n : null);
		}
	}

	function handleSelect(acc: AccountResponse) {
		query = `${acc.number} — ${acc.name}`;
		open = false;
		onSelect(acc.id);
	}

	function handleKeydown(e: KeyboardEvent) {
		if (loadError) return;

		if (e.key === 'ArrowDown') {
			e.preventDefault();
			open = true;
			highlightIndex = Math.min(highlightIndex + 1, filtered.length - 1);
		} else if (e.key === 'ArrowUp') {
			e.preventDefault();
			highlightIndex = Math.max(highlightIndex - 1, 0);
		} else if (e.key === 'Enter') {
			if (open && filtered[highlightIndex]) {
				e.preventDefault();
				handleSelect(filtered[highlightIndex]);
			}
		} else if (e.key === 'Escape') {
			open = false;
		}
	}

	function handleBlur() {
		// Délai pour permettre un clic sur un item du dropdown.
		setTimeout(() => {
			open = false;
		}, 150);
	}
</script>

<div class="relative">
	<Input
		type="text"
		value={query}
		oninput={handleInput}
		onkeydown={handleKeydown}
		onfocus={() => (open = true)}
		onblur={handleBlur}
		{disabled}
		placeholder={loadError
			? i18nMsg('account-autocomplete-unavailable', 'Autocomplétion indisponible — saisir l\'ID du compte')
			: i18nMsg('journal-entry-form-col-account', 'Compte')}
		aria-autocomplete="list"
		aria-expanded={open}
	/>

	{#if open && !loadError && filtered.length > 0}
		<ul
			class="absolute z-20 mt-1 max-h-60 w-full overflow-auto rounded-md border border-border bg-popover shadow-md"
			role="listbox"
		>
			{#each filtered as acc, i (acc.id)}
				<li
					class="cursor-pointer px-3 py-2 text-sm hover:bg-accent"
					class:bg-accent={i === highlightIndex}
					onmousedown={(e) => {
						e.preventDefault();
						handleSelect(acc);
					}}
					role="option"
					aria-selected={i === highlightIndex}
				>
					<span class="font-mono text-xs mr-2">{acc.number}</span>
					<span>{acc.name}</span>
				</li>
			{/each}
		</ul>
	{/if}
</div>
