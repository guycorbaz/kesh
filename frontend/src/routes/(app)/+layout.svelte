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
	import { i18nMsg } from '$lib/shared/utils/i18n.svelte';
	import { appVersion } from '$lib/shared/utils/app-version.svelte';
	import { page } from '$app/state';

	type NavItem =
		| { i18nKey: string; fallback: string; href: string }
		| { label: string; href: string };

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
		// Ctrl+N : nouvelle écriture (Expert mode uniquement, onboarding complété)
		if ((e.ctrlKey || e.metaKey) && e.key === 'n' && modeState.value === 'expert' && onboardingState.loaded && onboardingState.stepCompleted >= 3) {
			e.preventDefault();
			import('$app/navigation').then(({ goto }) => goto('/journal-entries'));
		}
	}

	let isAdmin = $derived(authState.currentUser?.role === 'Admin');

	// Story v014-1 : sidebar restructurée en 3 groupes (Quotidien / Mensuel / Administration).
	// 4 pages orphelines pré-v014-1 ajoutées à Administration (Plan comptable, Exercices,
	// Profils bancaires, Règles d'affectation). Entrée « Payer » supprimée
	// (nom trompeur — c'est la configuration des comptes bancaires) et déplacée en
	// « Comptes bancaires » sous Administration. Items « Export global » et « Paramètres »
	// déplacés de leur ancien groupe sans label vers Administration. Items admin-only
	// (Utilisateurs, Facturation) fusionnés dans Administration via {#if isAdmin}.
	const navGroups: Array<{ key: string; label: string; defaultExpanded: boolean; items: NavItem[]; adminOnly?: NavItem[] }> = [
		{
			key: 'quotidien',
			label: 'Quotidien',
			defaultExpanded: true,
			items: [
				{ i18nKey: 'nav-home', fallback: 'Accueil', href: '/' },
				{ i18nKey: 'nav-contacts', fallback: "Carnet d'adresses", href: '/contacts' },
				{ i18nKey: 'nav-products', fallback: 'Catalogue', href: '/products' },
				{ i18nKey: 'nav-invoices', fallback: 'Facturer', href: '/invoices' },
				{ i18nKey: 'nav-invoicing-due-dates', fallback: 'Échéancier', href: '/invoices/due-dates' },
				{ i18nKey: 'nav-credit-notes', fallback: 'Avoirs', href: '/credit-notes' },
				{
					i18nKey: 'nav-supplier-invoices',
					fallback: 'Factures fournisseurs',
					href: '/supplier-invoices',
				},
				{
					i18nKey: 'nav-supplier-invoices-import',
					fallback: 'Importer des factures',
					href: '/supplier-invoices/import',
				},
				{
					i18nKey: 'nav-payment-batches',
					fallback: 'Paiements fournisseurs',
					href: '/payment-batches',
				},
				{ label: 'Importer', href: '/bank-import' },
			],
		},
		{
			key: 'mensuel',
			label: 'Mensuel',
			defaultExpanded: true,
			// Note FINDING-12 Pass 1 : items Mensuel conservés en `label:` hardcodé FR
			// (cohérent avec l'existant pré-v014-1). Migration i18n complète reportée v0.2.
			items: [
				{ label: 'Écritures', href: '/journal-entries' },
				{ label: 'Réconciliation', href: '/reconciliation' },
				{ label: 'Rapports', href: '/reports' },
			],
		},
		{
			key: 'administration',
			label: 'Administration',
			defaultExpanded: false,
			items: [
				{ i18nKey: 'nav-accounts', fallback: 'Plan comptable', href: '/accounts' },
				{ i18nKey: 'nav-fiscal-years', fallback: 'Exercices comptables', href: '/settings/fiscal-years' },
				{ i18nKey: 'nav-bank-accounts', fallback: 'Comptes bancaires', href: '/bank-accounts' },
				{ i18nKey: 'nav-bank-profiles', fallback: 'Profils bancaires', href: '/bank-import/profiles' },
				{ i18nKey: 'nav-reconciliation-rules', fallback: "Règles d'affectation", href: '/reconciliation/rules' },
				{ i18nKey: 'nav-projects', fallback: 'Projets analytiques', href: '/settings/projects' },
				{ i18nKey: 'nav-export-global', fallback: 'Export global', href: '/export' },
				{ i18nKey: 'nav-settings', fallback: 'Paramètres', href: '/settings' },
			],
			// Note FINDING-7 Pass 1 : admin-only items conservés en `label:` hardcodé FR
			// (i18n complète reportée v0.2 — hors scope du hotfix UX v0.1.4).
			adminOnly: [
				{ label: 'Utilisateurs', href: '/users' },
				{ label: 'Facturation', href: '/settings/invoicing' },
				// Story 20-2 (Epic 20) — section Admin « Modèles d'e-mail ».
				{
					i18nKey: 'nav-email-templates',
					fallback: "Modèles d'e-mail",
					href: '/settings/email-templates',
				},
				// Story 17-3b — sauvegarde complète de l'installation (.keshbackup).
				// i18nKey (rendu via getItemLabel, AC10/AC27). Distinct de
				// l'« Export global » per-company (/export, Story 9-2b).
				{
					i18nKey: 'nav-admin-backup',
					fallback: 'Sauvegarde complète',
					href: '/admin/backup',
				},
				// Story 17-3d — import/restauration complète de l'installation.
				{
					i18nKey: 'nav-admin-restore',
					fallback: 'Restaurer / Importer',
					href: '/admin/restore',
				},
			],
		},
	];

	function getItemLabel(item: NavItem): string {
		if ('i18nKey' in item) {
			return i18nMsg(item.i18nKey, item.fallback);
		}
		return item.label;
	}

	// Story v014-1 (FINDING-3 Pass 2 Haiku) — Persistence localStorage SSR-safe.
	// Pattern cohérent `lib/app/stores/mode.svelte.ts:21,30` typeof guard.
	const STORAGE_PREFIX = 'kesh:sidebar:expanded:';

	function readPersisted(key: string): boolean | null {
		if (typeof localStorage === 'undefined') return null;
		const v = localStorage.getItem(STORAGE_PREFIX + key);
		if (v === 'true') return true;
		if (v === 'false') return false;
		return null;
	}

	function persistGroupState(key: string, open: boolean): void {
		if (typeof localStorage !== 'undefined') {
			localStorage.setItem(STORAGE_PREFIX + key, open ? 'true' : 'false');
		}
	}

	function initialOpen(key: string, defaultExpanded: boolean): boolean {
		const persisted = readPersisted(key);
		return persisted ?? defaultExpanded;
	}

	// Story v014-1 (FINDING-12 Pass 3 Opus) — auto-expand groupe contenant la route
	// active au mount (UX + a11y). Ne PAS persister cet auto-expand (sinon l'utilisateur
	// ne peut plus jamais cacher le groupe contenant sa route fréquente).
	function findGroupKeyForPath(pathname: string): string | null {
		for (const g of navGroups) {
			const allItems = [...g.items, ...(g.adminOnly ?? [])];
			if (allItems.some((it) => it.href === pathname)) {
				return g.key;
			}
		}
		return null;
	}

	let expandedState = $state<Record<string, boolean>>({
		quotidien: initialOpen('quotidien', true),
		mensuel: initialOpen('mensuel', true),
		administration: initialOpen('administration', false),
	});

	// F10 Pass 1 code review — set des groupes déjà auto-expandés cette session
	// pour éviter que le $effect re-ouvre le groupe quand l'utilisateur le
	// ferme manuellement (cf. ECH-7/BH-6). On track UNIQUEMENT le pathname
	// pour lequel l'auto-expand a déjà été appliqué.
	let autoExpandedPaths = $state<Set<string>>(new Set());

	// Force-open groupe contenant la route active (session uniquement, pas
	// persisté). N'auto-expand qu'UNE seule fois par pathname — si l'utilisateur
	// ferme ensuite le groupe, on respecte son choix jusqu'à navigation vers
	// une autre route.
	$effect(() => {
		const pathname = page.url.pathname;
		if (autoExpandedPaths.has(pathname)) return;
		const groupKey = findGroupKeyForPath(pathname);
		if (groupKey && !expandedState[groupKey]) {
			expandedState = { ...expandedState, [groupKey]: true };
		}
		autoExpandedPaths = new Set(autoExpandedPaths).add(pathname);
	});

	function handleToggle(key: string, e: Event): void {
		const open = (e.currentTarget as HTMLDetailsElement).open;
		expandedState = { ...expandedState, [key]: open };
		persistGroupState(key, open);
	}

	/**
	 * Calcule le slug `data-testid` d'un lien sidebar à partir de son href.
	 *
	 * `/users`              → `nav-link-users`
	 * `/invoices/due-dates` → `nav-link-invoices-due-dates`
	 * `/settings/invoicing` → `nav-link-settings-invoicing`
	 * `/`                   → `nav-link-home`
	 *
	 * Note (Story 7-5 KF-008) : on lowercase explicitement pour éviter qu'un futur
	 * href avec casse mixte (`/Users`) génère un testid distinct du même elt sémantique.
	 * Les hrefs actuels de `navGroups` et `adminNavItems` produisent des slugs
	 * uniques entre eux ; toute future addition doit préserver cette unicité.
	 */
	function navTestid(href: string): string {
		const slug = href.replace(/^\//, '').replace(/\//g, '-').toLowerCase();
		return `nav-link-${slug || 'home'}`;
	}
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
					{#snippet child({ props })}
						<Button variant="ghost" class="flex items-center gap-2" {...props}>
							<User class="h-4 w-4" aria-hidden="true" />
							<span class="text-sm">
								{authState.currentUser?.role ?? 'Utilisateur'}
							</span>
							<ChevronDown class="h-3 w-3" aria-hidden="true" />
						</Button>
					{/snippet}
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
						Mode : {modeState.value === 'guided' ? i18nMsg('mode-guided-label', 'Guidé') : i18nMsg('mode-expert-label', 'Expert')}
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
				<details
					open={expandedState[group.key]}
					ontoggle={(e) => handleToggle(group.key, e)}
					data-testid={`nav-group-${group.key}`}
				>
					<summary class="kesh-sidebar-summary flex cursor-pointer items-center justify-between rounded-md px-1 py-1 text-xs font-medium uppercase tracking-wider text-text-muted hover:text-text">
						<span>{group.label}</span>
						<ChevronDown class="kesh-sidebar-chevron h-4 w-4 transition-transform" aria-hidden="true" />
					</summary>
					<ul class="flex flex-col mt-1" style="gap: 2px;">
						{#each group.items as item}
							<li>
								<a
									href={item.href}
									data-testid={navTestid(item.href)}
									class="flex items-center rounded-md px-3 text-sm text-text hover:bg-primary-light/10 hover:text-primary transition-colors"
									style="min-height: var(--kesh-target-min-height);"
								>
									{getItemLabel(item)}
								</a>
							</li>
						{/each}
						{#if group.adminOnly && isAdmin}
							{#each group.adminOnly as item}
								<li>
									<a
										href={item.href}
										data-testid={navTestid(item.href)}
										class="flex items-center rounded-md px-3 text-sm text-text hover:bg-primary-light/10 hover:text-primary transition-colors"
										style="min-height: var(--kesh-target-min-height);"
									>
										{getItemLabel(item)}
									</a>
								</li>
							{/each}
						{/if}
					</ul>
				</details>
			{/each}
		</nav>

		<!-- Zone contenu fluide -->
		<main class="flex-1 overflow-auto" style="padding: var(--kesh-padding);">
			{@render children()}
		</main>
	</div>

	<!-- Footer discret -->
	<footer class="border-t border-border px-4 py-2 text-center text-xs text-text-muted">
		Kesh v{appVersion.value} &mdash; Logiciel libre (EUPL 1.2). Les données ne remplacent pas un fiduciaire.
		{#if modeState.value === 'expert'}
			<span class="ml-4">{i18nMsg('shortcut-new-entry', 'Ctrl+N : Nouvelle écriture')}</span>
		{/if}
	</footer>
</div>

<style>
	/* Story v014-1 — chevron rotation pour <details>/<summary> collapsible. */
	:global(details[open]) > summary > :global(.kesh-sidebar-chevron) {
		transform: rotate(180deg);
	}
	/* Supprimer le marker natif du <summary> (triangle disclosure browser). */
	summary.kesh-sidebar-summary {
		list-style: none;
	}
	summary.kesh-sidebar-summary::-webkit-details-marker {
		display: none;
	}
</style>
