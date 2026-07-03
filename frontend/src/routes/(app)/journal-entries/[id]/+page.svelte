<script lang="ts">
	import { Button } from '$lib/components/ui/button';
	import { onMount } from 'svelte';
	import { goto } from '$app/navigation';
	import { page } from '$app/state';
	import { ArrowLeft } from '@lucide/svelte';
	import Big from 'big.js';
	import { getJournalEntry } from '$lib/features/journal-entries/journal-entries.api';
	import type { JournalEntryResponse } from '$lib/features/journal-entries/journal-entries.types';
	import { fetchAccounts } from '$lib/features/accounts/accounts.api';
	import type { AccountResponse } from '$lib/features/accounts/accounts.types';
	import { listProjects } from '$lib/features/projects/projects.api';
	import type { ProjectResponse } from '$lib/features/projects/projects.types';
	import { formatSwissAmount } from '$lib/features/journal-entries/balance';
	import { isApiError } from '$lib/shared/utils/api-client';

	let entry = $state<JournalEntryResponse | null>(null);
	let accountsById = $state<Map<number, AccountResponse>>(new Map());
	let projectsById = $state<Map<number, ProjectResponse>>(new Map());
	let loading = $state(true);
	let errorMsg = $state('');

	let id = $derived(parseInt(page.params.id ?? '', 10));

	onMount(async () => {
		if (!Number.isFinite(id) || id <= 0) {
			errorMsg = "Identifiant d'écriture invalide";
			loading = false;
			return;
		}
		try {
			// Comptes chargés en parallèle pour résoudre accountId → numéro + nom.
			// `fetchAccounts(true)` inclut les comptes archivés : une écriture
			// historique peut référencer un compte depuis archivé. Idem
			// `listProjects(true)` pour les tags analytiques (Epic 19) : un
			// projet archivé doit rester lisible dans l'historique.
			const [e, accounts, projects] = await Promise.all([
				getJournalEntry(id),
				fetchAccounts(true),
				listProjects(true)
			]);
			entry = e;
			accountsById = new Map(accounts.map((a) => [a.id, a]));
			projectsById = new Map(projects.map((p) => [p.id, p]));
		} catch (err) {
			errorMsg = isApiError(err) ? err.message : "Erreur de chargement de l'écriture";
		} finally {
			loading = false;
		}
	});

	function accountLabel(accountId: number): string {
		const a = accountsById.get(accountId);
		return a ? `${a.number} — ${a.name}` : `#${accountId}`;
	}

	function projectLabel(projectId: number): string {
		const p = projectsById.get(projectId);
		return p ? `${p.code} — ${p.name}` : `#${projectId}`;
	}

	// Colonne projet affichée seulement si au moins une ligne est taguée —
	// les écritures non-analytiques gardent leur affichage d'avant.
	let hasProjects = $derived((entry?.lines ?? []).some((l) => l.projectId !== null));

	/** Blanchit les montants nuls (convention comptable : débit OU crédit par ligne). */
	function fmtAmount(v: string): string {
		try {
			const b = new Big(v || '0');
			return b.eq(0) ? '' : formatSwissAmount(b);
		} catch {
			return v;
		}
	}

	function sumLines(field: 'debit' | 'credit'): Big {
		if (!entry) return new Big(0);
		return entry.lines.reduce((acc, l) => acc.plus(new Big(l[field] || '0')), new Big(0));
	}

	let totalDebit = $derived(sumLines('debit'));
	let totalCredit = $derived(sumLines('credit'));
</script>

<svelte:head>
	<title>{entry ? `Écriture n°${entry.entryNumber}` : 'Écriture'} — Kesh</title>
</svelte:head>

<div class="mb-6 flex items-center justify-between">
	<Button variant="ghost" onclick={() => goto('/journal-entries')}>
		<ArrowLeft class="h-4 w-4" aria-hidden="true" />
		Retour
	</Button>
</div>

{#if loading}
	<p class="text-sm text-text-muted">Chargement…</p>
{:else if errorMsg}
	<div class="rounded-md border border-destructive bg-destructive/10 px-3 py-2 text-sm text-destructive">
		{errorMsg}
	</div>
{:else if entry}
	<h1 class="mb-4 text-2xl font-semibold text-text">Écriture n°{entry.entryNumber}</h1>

	<dl class="mb-6 grid grid-cols-1 gap-x-8 gap-y-2 text-sm sm:grid-cols-2">
		<div class="flex justify-between border-b border-border py-1">
			<dt class="text-text-muted">Date</dt>
			<dd class="font-medium">{entry.entryDate}</dd>
		</div>
		<div class="flex justify-between border-b border-border py-1">
			<dt class="text-text-muted">Journal</dt>
			<dd class="font-medium">{entry.journal}</dd>
		</div>
		<div class="flex justify-between border-b border-border py-1 sm:col-span-2">
			<dt class="text-text-muted">Libellé</dt>
			<dd class="font-medium">{entry.description}</dd>
		</div>
	</dl>

	<table class="w-full border-collapse text-sm">
		<thead>
			<tr class="border-b border-border text-left">
				<th class="py-2 pr-2">Compte</th>
				{#if hasProjects}
					<th class="py-2 pr-2">Projet</th>
				{/if}
				<th class="py-2 pr-2 w-36 text-right">Débit</th>
				<th class="py-2 pr-2 w-36 text-right">Crédit</th>
			</tr>
		</thead>
		<tbody>
			{#each entry.lines as line (line.id)}
				<tr class="border-b border-border">
					<td class="py-2 pr-2">{accountLabel(line.accountId)}</td>
					{#if hasProjects}
						<td class="py-2 pr-2">
							{line.projectId !== null ? projectLabel(line.projectId) : ''}
						</td>
					{/if}
					<td class="py-2 pr-2 text-right font-mono">{fmtAmount(line.debit)}</td>
					<td class="py-2 pr-2 text-right font-mono">{fmtAmount(line.credit)}</td>
				</tr>
			{/each}
		</tbody>
		<tfoot>
			<tr class="font-semibold">
				<td class="py-3 text-right" colspan={hasProjects ? 2 : 1}>Total</td>
				<td class="py-3 pr-2 text-right font-mono">{formatSwissAmount(totalDebit)}</td>
				<td class="py-3 pr-2 text-right font-mono">{formatSwissAmount(totalCredit)}</td>
			</tr>
		</tfoot>
	</table>
{/if}
