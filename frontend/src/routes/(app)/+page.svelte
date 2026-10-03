<script lang="ts">
	// Story 25-6-a (#388, #389) — la page d'accueil charge ses trois sources et
	// les confie aux tuiles de `features/homepage/`, testables isolément. Les
	// trois endpoints sont ouverts à tout rôle authentifié ; seul le compteur
	// « à rappeler » est réservé à Admin/Comptable.
	import { onMount } from 'svelte';
	import { modeState } from '$lib/app/stores/mode.svelte';
	import { i18nMsg } from '$lib/shared/utils/i18n.svelte';
	import { listBankAccounts, type BankAccountSummary } from '$lib/features/bank-accounts/bank-accounts.api';
	import { authState } from '$lib/app/stores/auth.svelte';
	import { listReminders } from '$lib/features/reminders/reminders.api';
	import { fetchJournalEntries } from '$lib/features/journal-entries/journal-entries.api';
	import type { JournalEntryResponse } from '$lib/features/journal-entries/journal-entries.types';
	import { listDueDates } from '$lib/features/invoices/invoices.api';
	import type { DueDatesSummary } from '$lib/features/invoices/invoices.types';
	import RecentEntriesCard from '$lib/features/homepage/RecentEntriesCard.svelte';
	import OpenInvoicesCard from '$lib/features/homepage/OpenInvoicesCard.svelte';
	import BankAccountsCard from '$lib/features/homepage/BankAccountsCard.svelte';

	type LoadState = 'loading' | 'ready' | 'error';

	// Story 21-6c (D-c2) — `/dunning/reminders` est Comptable+ : un Consultation
	// prendrait un 403 → ne PAS fetcher pour ce rôle.
	let canManage = $derived(
		authState.currentUser?.role === 'Admin' || authState.currentUser?.role === 'Comptable',
	);
	let isGuided = $derived(modeState.value === 'guided');

	let entriesState = $state<LoadState>('loading');
	let entries = $state<JournalEntryResponse[]>([]);
	let invoicesState = $state<LoadState>('loading');
	let summary = $state<DueDatesSummary | null>(null);
	let reminderCount = $state(0);
	let bankAccounts = $state<BankAccountSummary[]>([]);
	let bankLoaded = $state(false);

	async function loadEntries() {
		try {
			entries = (await fetchJournalEntries({ limit: 5 })).items;
			entriesState = 'ready';
		} catch {
			entriesState = 'error';
		}
	}

	async function loadInvoices() {
		try {
			// Seul le résumé sert : la liste paginée qui l'accompagne est réduite.
			summary = (await listDueDates({ limit: 1 })).summary;
			invoicesState = 'ready';
		} catch {
			invoicesState = 'error';
		}
	}

	async function loadBankAccounts() {
		try {
			bankAccounts = await listBankAccounts(false);
		} catch {
			// Tuile cachée si l'appel échoue ou s'il n'y a aucun compte.
		} finally {
			bankLoaded = true;
		}
	}

	async function loadReminders() {
		if (!canManage) return;
		try {
			const res = await listReminders();
			reminderCount = res.groups.reduce((n, g) => n + g.invoices.length, 0);
		} catch {
			// Compteur silencieux : un échec ne pollue pas l'accueil.
		}
	}

	onMount(() => {
		void loadEntries();
		void loadInvoices();
		void loadBankAccounts();
		void loadReminders();
	});
</script>

<svelte:head>
	<title>Accueil - Kesh</title>
</svelte:head>

<h1 class="mb-6 text-2xl font-semibold text-text">
	{i18nMsg('homepage-title', 'Tableau de bord')}
</h1>

<div class="grid grid-cols-1 gap-6 lg:grid-cols-3">
	<RecentEntriesCard state={entriesState} {entries} {canManage} {isGuided} />
	<OpenInvoicesCard state={invoicesState} {summary} {canManage} {isGuided} {reminderCount} />
	<!-- Story v014-1 (AC#29) — tuile retirée s'il n'y a aucun compte bancaire. -->
	{#if bankLoaded && bankAccounts.length > 0}
		<BankAccountsCard accounts={bankAccounts} />
	{/if}
</div>
