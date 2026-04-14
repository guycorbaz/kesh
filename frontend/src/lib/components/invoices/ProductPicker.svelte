<!--
  ProductPicker (Story 5.1) : dialog simple pour sélectionner un produit
  actif du catalogue et renvoyer un snapshot (name/unitPrice/vatRate).
-->
<script lang="ts">
	import * as Dialog from '$lib/components/ui/dialog';
	import { Button } from '$lib/components/ui/button';
	import { Input } from '$lib/components/ui/input';
	import { listProducts } from '$lib/features/products/products.api';
	import type { ProductResponse } from '$lib/features/products/products.types';
	import { formatInvoiceTotal } from '$lib/features/invoices/invoice-helpers';
	import { isApiError } from '$lib/shared/utils/api-client';
	import { notifyError } from '$lib/shared/utils/notify';
	import { onMount } from 'svelte';

	type Props = {
		open: boolean;
		onOpenChange: (o: boolean) => void;
		onSelect: (p: ProductResponse) => void;
	};
	let { open, onOpenChange, onSelect }: Props = $props();

	let search = $state('');
	let items = $state<ProductResponse[]>([]);
	let loading = $state(false);
	let debounceHandle: ReturnType<typeof setTimeout> | null = null;
	let seq = 0;

	async function load(q: string) {
		const s = ++seq;
		loading = true;
		try {
			const r = await listProducts({ search: q, includeArchived: false, limit: 50 });
			if (s !== seq) return;
			items = r.items;
		} catch (err) {
			if (s !== seq) return;
			items = [];
			if (isApiError(err)) notifyError(err.message);
			else notifyError('Erreur de chargement des produits');
		} finally {
			if (s === seq) loading = false;
		}
	}

	$effect(() => {
		if (open) {
			load('');
			search = '';
		}
	});

	function onSearchInput(e: Event) {
		search = (e.target as HTMLInputElement).value;
		if (debounceHandle) clearTimeout(debounceHandle);
		debounceHandle = setTimeout(() => load(search.trim()), 300);
	}

	function pick(p: ProductResponse) {
		onSelect(p);
		onOpenChange(false);
	}

	onMount(() => {
		return () => {
			if (debounceHandle) clearTimeout(debounceHandle);
		};
	});
</script>

<Dialog.Root {open} {onOpenChange}>
	<Dialog.Content>
		<Dialog.Header>
			<Dialog.Title>Sélectionner un produit</Dialog.Title>
		</Dialog.Header>
		<Input
			type="search"
			placeholder="Rechercher un produit…"
			value={search}
			oninput={onSearchInput}
		/>
		<div class="mt-3 max-h-80 overflow-auto rounded-md border border-border">
			{#if loading}
				<div class="px-3 py-2 text-sm text-text-muted">Chargement…</div>
			{:else if items.length === 0}
				<div class="px-3 py-2 text-sm text-text-muted">Aucun produit</div>
			{:else}
				<ul>
					{#each items as p (p.id)}
						<li>
							<button
								type="button"
								class="w-full cursor-pointer border-b border-border px-3 py-2 text-left text-sm hover:bg-muted last:border-b-0"
								onclick={() => pick(p)}
							>
								<div class="font-medium">{p.name}</div>
								<div class="text-xs text-text-muted">
									{formatInvoiceTotal(p.unitPrice)} CHF · TVA {p.vatRate}%
								</div>
							</button>
						</li>
					{/each}
				</ul>
			{/if}
		</div>
		<Dialog.Footer>
			<Button variant="outline" onclick={() => onOpenChange(false)}>Fermer</Button>
		</Dialog.Footer>
	</Dialog.Content>
</Dialog.Root>
