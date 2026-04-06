<script lang="ts">
	import { Button } from '$lib/components/ui/button';
	import { Input } from '$lib/components/ui/input';
	import * as Select from '$lib/components/ui/select';
	import * as Table from '$lib/components/ui/table';
	import * as Dialog from '$lib/components/ui/dialog';
	import * as Tooltip from '$lib/components/ui/tooltip';
	import * as DropdownMenu from '$lib/components/ui/dropdown-menu';
	import { toast } from 'svelte-sonner';
	import { modeState, toggleMode } from '$lib/app/stores/mode.svelte';

	const mode = $derived(modeState.value);

	const sampleData = [
		{ id: 1, account: '1000', label: 'Caisse', debit: "1'250.00", credit: '0.00' },
		{ id: 2, account: '1100', label: 'Banque UBS', debit: '0.00', credit: "1'250.00" },
		{ id: 3, account: '3200', label: "Ventes de marchandises", debit: '0.00', credit: "5'430.50" },
		{ id: 4, account: '4200', label: "Achats de marchandises", debit: "5'430.50", credit: '0.00' },
	];

	let dialogOpen = $state(false);
</script>

<div class="max-w-4xl mx-auto" style="padding: var(--kesh-padding);">
	<h1 class="text-2xl font-semibold text-primary mb-4">Design System Kesh</h1>
	<p class="text-text-muted mb-8">Validation visuelle des tokens, typographie et composants — Story 1.9</p>

	<!-- Mode Toggle -->
	<section style="margin-block: var(--kesh-section-margin);">
		<h2 class="text-xl font-semibold mb-2">Mode : {mode}</h2>
		<Button onclick={toggleMode}>
			Basculer en mode {mode === 'guided' ? 'Expert' : 'Guidé'}
		</Button>
	</section>

	<!-- Couleurs -->
	<section style="margin-block: var(--kesh-section-margin);">
		<h2 class="text-xl font-semibold mb-2">Palette de couleurs</h2>
		<div class="flex flex-wrap" style="gap: var(--kesh-gap);">
			<div class="w-24 h-16 rounded bg-primary flex items-center justify-center text-white text-sm">Primary</div>
			<div class="w-24 h-16 rounded bg-primary-light flex items-center justify-center text-white text-sm">P. Light</div>
			<div class="w-24 h-16 rounded bg-success flex items-center justify-center text-white text-sm">Success</div>
			<div class="w-24 h-16 rounded bg-error flex items-center justify-center text-white text-sm">Error</div>
			<div class="w-24 h-16 rounded bg-warning flex items-center justify-center text-white text-sm">Warning</div>
			<div class="w-24 h-16 rounded bg-info flex items-center justify-center text-white text-sm">Info</div>
			<div class="w-24 h-16 rounded bg-surface-alt border border-border flex items-center justify-center text-sm">Surface Alt</div>
		</div>
	</section>

	<!-- Typographie -->
	<section style="margin-block: var(--kesh-section-margin);">
		<h2 class="text-xl font-semibold mb-2">Typographie (Inter)</h2>
		<div class="space-y-2">
			<p class="text-2xl font-semibold">H1 — Titre principal (24px/600)</p>
			<p class="text-xl font-semibold">H2 — Section (20px/600)</p>
			<p class="text-base font-semibold">H3 — Sous-section (16px/600)</p>
			<p class="text-sm">Corps — Texte courant (14px/400)</p>
			<p class="text-sm font-medium tabular-nums">Montants : 1'234.56 CHF — tabular-nums</p>
			<p class="text-xs text-text-muted">Petit texte — Tooltips, métadonnées (12px/400)</p>
		</div>
	</section>

	<!-- Boutons -->
	<section style="margin-block: var(--kesh-section-margin);">
		<h2 class="text-xl font-semibold mb-2">Boutons</h2>
		<div class="flex flex-wrap items-center" style="gap: var(--kesh-gap);">
			<Button>Default</Button>
			<Button variant="secondary">Secondary</Button>
			<Button variant="destructive">Destructive</Button>
			<Button variant="outline">Outline</Button>
			<Button variant="ghost">Ghost</Button>
			<Button disabled>Disabled</Button>
		</div>
	</section>

	<!-- Inputs -->
	<section style="margin-block: var(--kesh-section-margin);">
		<h2 class="text-xl font-semibold mb-2">Inputs & Select</h2>
		<div class="flex flex-wrap items-end" style="gap: var(--kesh-gap);">
			<div>
				<label class="text-sm font-medium mb-1 block">Nom d'utilisateur</label>
				<Input placeholder="alice" />
			</div>
			<div>
				<label class="text-sm font-medium mb-1 block">Montant</label>
				<Input type="text" placeholder="1'250.00" class="tabular-nums text-right" />
			</div>
		</div>
	</section>

	<!-- Table -->
	<section style="margin-block: var(--kesh-section-margin);">
		<h2 class="text-xl font-semibold mb-2">Table comptable</h2>
		<Table.Root>
			<Table.Header>
				<Table.Row style="height: var(--kesh-table-row-height);">
					<Table.Head>Compte</Table.Head>
					<Table.Head>Libellé</Table.Head>
					<Table.Head class="text-right">Débit</Table.Head>
					<Table.Head class="text-right">Crédit</Table.Head>
				</Table.Row>
			</Table.Header>
			<Table.Body>
				{#each sampleData as row}
					<Table.Row style="height: var(--kesh-table-row-height);">
						<Table.Cell class="font-medium">{row.account}</Table.Cell>
						<Table.Cell>{row.label}</Table.Cell>
						<Table.Cell class="text-right tabular-nums">{row.debit}</Table.Cell>
						<Table.Cell class="text-right tabular-nums">{row.credit}</Table.Cell>
					</Table.Row>
				{/each}
			</Table.Body>
		</Table.Root>
	</section>

	<!-- Dialog -->
	<section style="margin-block: var(--kesh-section-margin);">
		<h2 class="text-xl font-semibold mb-2">Dialog, Tooltip, Dropdown</h2>
		<div class="flex flex-wrap items-center" style="gap: var(--kesh-gap);">
			<Dialog.Root bind:open={dialogOpen}>
				<Dialog.Trigger>
					{#snippet child({ props })}
						<Button {...props}>Ouvrir Dialog</Button>
					{/snippet}
				</Dialog.Trigger>
				<Dialog.Content>
					<Dialog.Header>
						<Dialog.Title>Confirmation</Dialog.Title>
						<Dialog.Description>Voulez-vous valider cette écriture comptable ?</Dialog.Description>
					</Dialog.Header>
					<Dialog.Footer>
						<Button variant="outline" onclick={() => (dialogOpen = false)}>Annuler</Button>
						<Button onclick={() => (dialogOpen = false)}>Confirmer</Button>
					</Dialog.Footer>
				</Dialog.Content>
			</Dialog.Root>

			<Tooltip.Root>
				<Tooltip.Trigger>
					{#snippet child({ props })}
						<Button variant="outline" {...props}>Tooltip</Button>
					{/snippet}
				</Tooltip.Trigger>
				<Tooltip.Content>
					<p>Info-bulle avec style Kesh</p>
				</Tooltip.Content>
			</Tooltip.Root>

			<DropdownMenu.Root>
				<DropdownMenu.Trigger>
					{#snippet child({ props })}
						<Button variant="outline" {...props}>Menu ▾</Button>
					{/snippet}
				</DropdownMenu.Trigger>
				<DropdownMenu.Content>
					<DropdownMenu.Item>Modifier</DropdownMenu.Item>
					<DropdownMenu.Item>Dupliquer</DropdownMenu.Item>
					<DropdownMenu.Separator />
					<DropdownMenu.Item class="text-error">Supprimer</DropdownMenu.Item>
				</DropdownMenu.Content>
			</DropdownMenu.Root>
		</div>
	</section>

	<!-- Toast (Sonner) -->
	<section style="margin-block: var(--kesh-section-margin);">
		<h2 class="text-xl font-semibold mb-2">Toast (Sonner)</h2>
		<div class="flex flex-wrap items-center" style="gap: var(--kesh-gap);">
			<Button onclick={() => toast.success('Écriture enregistrée')}>Toast succès</Button>
			<Button variant="destructive" onclick={() => toast.error('Écriture déséquilibrée')}>Toast erreur</Button>
			<Button variant="outline" onclick={() => toast.info('Import en cours...')}>Toast info</Button>
		</div>
	</section>

	<!-- Espacements -->
	<section style="margin-block: var(--kesh-section-margin);">
		<h2 class="text-xl font-semibold mb-2">Espacements ({mode})</h2>
		<div class="space-y-2 text-sm">
			<p>Gap : <code>var(--kesh-gap)</code> = {mode === 'guided' ? '16px' : '8px'}</p>
			<p>Padding : <code>var(--kesh-padding)</code> = {mode === 'guided' ? '24px' : '16px'}</p>
			<p>Section margin : <code>var(--kesh-section-margin)</code> = {mode === 'guided' ? '32px' : '16px'}</p>
			<p>Table row height : <code>var(--kesh-table-row-height)</code> = {mode === 'guided' ? '48px' : '36px'}</p>
			<p>Target min height : <code>var(--kesh-target-min-height)</code> = {mode === 'guided' ? '44px' : '32px'}</p>
		</div>
	</section>
</div>
