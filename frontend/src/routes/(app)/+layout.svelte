<script lang="ts">
	import { Separator } from '$lib/components/ui/separator';
	import * as DropdownMenu from '$lib/components/ui/dropdown-menu';
	import { Button } from '$lib/components/ui/button';
	import { Input } from '$lib/components/ui/input';
	import { modeState, toggleMode } from '$lib/app/stores/mode.svelte';
	import { authState } from '$lib/app/stores/auth.svelte';
	import { onboardingState } from '$lib/features/onboarding/onboarding.svelte';
	import DemoBanner from '$lib/shared/components/DemoBanner.svelte';
	import IncompleteBanner from '$lib/shared/components/IncompleteBanner.svelte';
	import { Search, LogOut, User, Settings, ChevronDown } from '@lucide/svelte';
	import { toast } from 'svelte-sonner';
	import { i18nMsg } from '$lib/features/onboarding/onboarding.svelte';

	let { children } = $props();

	let searchToastShown = false;
	function handleSearchFocus() {
		if (!searchToastShown) {
			toast.info(i18nMsg('search-coming-soon', 'Recherche bientôt disponible'));
			searchToastShown = true;
		}
	}

	function handleKeydown(e: KeyboardEvent) {
		if ((e.ctrlKey || e.metaKey) && e.key === 's') {
			e.preventDefault();
			window.dispatchEvent(new CustomEvent('kesh:save'));
		}
	}

	let isAdmin = $derived(authState.currentUser?.role === 'Admin');

	const navGroups = [
		{
			label: 'Quotidien',
			items: [
				{ label: 'Accueil', href: '/' },
				{ label: 'Facturer', href: '/invoices' },
				{ label: 'Payer', href: '/bank-accounts' },
				{ label: 'Importer', href: '/bank-import' },
			],
		},
		{
			label: 'Mensuel',
			items: [
				{ label: 'Écritures', href: '/journal-entries' },
				{ label: 'Réconciliation', href: '/reconciliation' },
				{ label: 'Rapports', href: '/reports' },
			],
		},
		{
			label: null,
			items: [{ label: 'Paramètres', href: '/settings' }],
		},
	];

	let adminNavItems = $derived(
		isAdmin ? [{ label: 'Utilisateurs', href: '/users' }] : []
	);
</script>

<svelte:window onkeydown={handleKeydown} />

<div class="flex min-h-screen min-w-[1280px] flex-col">
	<!-- Header fixe -->
	<header
		class="sticky top-0 z-30 flex h-14 items-center border-b border-border bg-surface px-4"
		style="gap: var(--kesh-gap);"
	>
		<!-- Logo -->
		<a href="/" class="flex items-center gap-2 font-semibold text-primary">
			<span class="text-xl">Kesh</span>
		</a>

		<!-- Zone recherche -->
		<div class="relative ml-4 flex-1 max-w-md">
			<Search class="absolute left-3 top-1/2 h-4 w-4 -translate-y-1/2 text-text-muted" aria-hidden="true" />
			<Input
				type="search"
				placeholder="Rechercher..."
				class="pl-10"
				readonly
				onfocus={handleSearchFocus}
			/>
		</div>

		<div class="ml-auto flex items-center" style="gap: var(--kesh-gap);">
			<!-- Menu profil -->
			<DropdownMenu.Root>
				<DropdownMenu.Trigger>
					<Button variant="ghost" class="flex items-center gap-2">
						<User class="h-4 w-4" aria-hidden="true" />
						<span class="text-sm">
							{authState.currentUser?.role ?? 'Utilisateur'}
						</span>
						<ChevronDown class="h-3 w-3" aria-hidden="true" />
					</Button>
				</DropdownMenu.Trigger>
				<DropdownMenu.Content align="end">
					<!-- Sélecteur de langue (non fonctionnel — Story 2.1) -->
					<DropdownMenu.Label>Langue</DropdownMenu.Label>
					<DropdownMenu.Item disabled>FR - Fran&ccedil;ais</DropdownMenu.Item>
					<DropdownMenu.Item disabled>DE - Deutsch</DropdownMenu.Item>
					<DropdownMenu.Item disabled>IT - Italiano</DropdownMenu.Item>
					<DropdownMenu.Item disabled>EN - English</DropdownMenu.Item>
					<DropdownMenu.Separator />

					<!-- Bascule mode -->
					<DropdownMenu.Item onclick={toggleMode}>
						Mode : {modeState.value === 'guided' ? 'Guidé' : 'Expert'}
					</DropdownMenu.Item>
					<DropdownMenu.Separator />

					<!-- Déconnexion -->
					<DropdownMenu.Item
						onclick={async () => {
							await authState.logout();
							window.location.replace('/login');
						}}
					>
						<LogOut class="mr-2 h-4 w-4" aria-hidden="true" />
						Se déconnecter
					</DropdownMenu.Item>
				</DropdownMenu.Content>
			</DropdownMenu.Root>
		</div>
	</header>

	<!-- Bannières contextuelles (mutuellement exclusives) -->
	{#if onboardingState.isDemo}
		<DemoBanner />
	{:else if !onboardingState.isDemo && onboardingState.loaded && onboardingState.stepCompleted >= 6 && onboardingState.stepCompleted < 7}
		<IncompleteBanner />
	{/if}

	<!-- Corps : sidebar + contenu -->
	<div class="flex flex-1">
		<!-- Sidebar fixe gauche -->
		<nav
			aria-label="Navigation principale"
			class="sticky top-14 flex h-[calc(100vh-3.5rem)] w-56 flex-col border-r border-border bg-surface-alt"
			style="padding: var(--kesh-padding);"
		>
			{#each navGroups as group, i}
				{#if i > 0}
					<Separator class="my-2" />
				{/if}
				{#if group.label}
					<span class="mb-1 text-xs font-medium uppercase tracking-wider text-text-muted">
						{group.label}
					</span>
				{/if}
				<ul class="flex flex-col" style="gap: 2px;">
					{#each group.items as item}
						<li>
							<a
								href={item.href}
								class="flex items-center rounded-md px-3 text-sm text-text hover:bg-primary-light/10 hover:text-primary transition-colors"
								style="min-height: var(--kesh-target-min-height);"
							>
								{item.label}
							</a>
						</li>
					{/each}
				</ul>
			{/each}
			{#if adminNavItems.length > 0}
				<Separator class="my-2" />
				<span class="mb-1 text-xs font-medium uppercase tracking-wider text-text-muted">
					Administration
				</span>
				<ul class="flex flex-col" style="gap: 2px;">
					{#each adminNavItems as item}
						<li>
							<a
								href={item.href}
								class="flex items-center rounded-md px-3 text-sm text-text hover:bg-primary-light/10 hover:text-primary transition-colors"
								style="min-height: var(--kesh-target-min-height);"
							>
								{item.label}
							</a>
						</li>
					{/each}
				</ul>
			{/if}
		</nav>

		<!-- Zone contenu fluide -->
		<main class="flex-1 overflow-auto" style="padding: var(--kesh-padding);">
			{@render children()}
		</main>
	</div>

	<!-- Footer discret -->
	<footer class="border-t border-border px-4 py-2 text-center text-xs text-text-muted">
		Kesh v0.1.0 &mdash; Logiciel libre (EUPL 1.2). Les données ne remplacent pas un fiduciaire.
	</footer>
</div>
