<script lang="ts">
	import { Button } from '$lib/components/ui/button';
	import { goto } from '$app/navigation';
	import { page } from '$app/state';
	import { ArrowLeft } from '@lucide/svelte';
	import Big from 'big.js';
	import {
		getJournalEntry,
		reverseJournalEntry
	} from '$lib/features/journal-entries/journal-entries.api';
	import type {
		JournalEntryDetailResponse,
		ReversalBlocker
	} from '$lib/features/journal-entries/journal-entries.types';
	import { i18nMsg } from '$lib/features/onboarding/onboarding.svelte';
	import { toast } from 'svelte-sonner';
	import { fetchAccounts } from '$lib/features/accounts/accounts.api';
	import type { AccountResponse } from '$lib/features/accounts/accounts.types';
	import { listProjects } from '$lib/features/projects/projects.api';
	import type { ProjectResponse } from '$lib/features/projects/projects.types';
	import { formatSwissAmount } from '$lib/features/journal-entries/balance';
	import { isApiError } from '$lib/shared/utils/api-client';

	let entry = $state<JournalEntryDetailResponse | null>(null);
	/** Story 24-4a (#380) — contre-passation. */
	let showReverseConfirm = $state(false);
	let reversing = $state(false);
	let accountsById = $state<Map<number, AccountResponse>>(new Map());
	let projectsById = $state<Map<number, ProjectResponse>>(new Map());
	let loading = $state(true);
	let errorMsg = $state('');

	let id = $derived(parseInt(page.params.id ?? '', 10));

	/**
	 * ⛔ **`$effect` et non `onMount`.** SvelteKit RÉUTILISE le composant quand on
	 * navigue de `/journal-entries/12` à `/journal-entries/13` : même route, autre
	 * paramètre. `onMount` ne se rejoue pas, et la page affichait l'écriture
	 * PRÉCÉDENTE sous la nouvelle URL.
	 *
	 * ⚠️ Ce défaut est né avec la contre-passation, qui est le premier chemin du
	 * dépôt à naviguer d'une fiche d'écriture vers une autre. Trouvé par le seul
	 * test E2E — ni Vitest ni les tests Rust ne voient une navigation.
	 */
	$effect(() => {
		void loadEntry(id);
	});

	async function loadEntry(id: number) {
		loading = true;
		entry = null;
		errorMsg = '';
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
			//
			// Seule l'écriture elle-même est requise : un échec des référentiels
			// (comptes, projets) dégrade l'affichage (`#id` en fallback) sans
			// casser la page — même tolérance que la page liste (allSettled).
			const [entryResult, accountsResult, projectsResult] = await Promise.allSettled([
				getJournalEntry(id),
				fetchAccounts(true),
				listProjects(true)
			]);
			if (entryResult.status === 'rejected') {
				const err = entryResult.reason;
				errorMsg = isApiError(err) ? err.message : "Erreur de chargement de l'écriture";
				return;
			}
			entry = entryResult.value;
			if (accountsResult.status === 'fulfilled') {
				accountsById = new Map(accountsResult.value.map((a) => [a.id, a]));
			}
			if (projectsResult.status === 'fulfilled') {
				projectsById = new Map(projectsResult.value.map((p) => [p.id, p]));
			}
		} finally {
			loading = false;
		}
	}

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

	/**
	 * Motif de blocage, traduit — Story 24-4a (#380).
	 *
	 * ⚠️ Le serveur rend un **code**, jamais une phrase : c'est ici que la
	 * traduction se fait. Un `switch` exhaustif plutôt qu'une table indexée, pour
	 * qu'un code neuf fasse rougir le type-check au lieu d'afficher du vide.
	 */
	function blockedLabel(code: ReversalBlocker): string {
		switch (code) {
			case 'IS_A_REVERSAL':
				return i18nMsg(
					'journal-entries-reverse-blocked-is-a-reversal',
					'Cette écriture est elle-même une contre-passation.'
				);
			case 'ALREADY_REVERSED':
				return i18nMsg(
					'journal-entries-reverse-blocked-already-reversed',
					'Cette écriture a déjà été contre-passée.'
				);
			case 'OWNED_BY_INVOICE':
				return i18nMsg(
					'journal-entries-reverse-blocked-invoice',
					'Cette écriture appartient à une facture client : corrigez-la par un avoir.'
				);
			case 'OWNED_BY_CREDIT_NOTE':
				return i18nMsg(
					'journal-entries-reverse-blocked-credit-note',
					"Cette écriture est celle d'un avoir, qui est déjà une contre-passation."
				);
			case 'OWNED_BY_SUPPLIER_INVOICE':
				return i18nMsg(
					'journal-entries-reverse-blocked-supplier-invoice',
					'Cette écriture appartient à une facture fournisseur : annulez la facture.'
				);
			case 'OWNED_BY_SETTLEMENT':
				return i18nMsg(
					'journal-entries-reverse-blocked-settlement',
					'Cette écriture est un règlement de facture : son annulation viendra avec la contre-passation des règlements.'
				);
			case 'MATCHED_BANK_TRANSACTION':
				return i18nMsg(
					'journal-entries-reverse-blocked-bank-match',
					"Cette écriture est rapprochée d'une transaction bancaire."
				);
			case 'ACCOUNT_ARCHIVED':
				return i18nMsg(
					'journal-entries-reverse-blocked-account-archived',
					'Un compte de cette écriture a été archivé : réactivez-le pour pouvoir la contre-passer.'
				);
			default: {
				// ⛔ **C'est l'affectation à `never` qui fait rougir**, pas le
				// `default` : ajouter un neuvième code au type sans l'ajouter ici
				// casse le type-check. Le paramètre était typé `string`, ce qui
				// ôtait au `switch` toute exhaustivité — trois doc-comments
				// affirmaient pourtant le contraire. *(Passe 2 de revue.)*
				const _exhaustif: never = code;
				void _exhaustif;
				// ⚠️ **Et l'on rend une chaîne vide, PAS `_exhaustif`.** À
				// l'exécution, `never` n'est qu'une fiction de compilation :
				// `return _exhaustif` rendait littéralement `code`, si bien qu'un
				// navigateur au bundle périmé face à un serveur plus récent aurait
				// affiché `NEUVIEME_CODE` en clair à l'utilisateur — une fuite de
				// jeton interne non traduit. La garde protège le compile-time, la
				// chaîne vide protège l'exécution ; il faut les deux.
				// *(Relevé en passe 3 de revue de code.)*
				return '';
			}
		}
	}

	/**
	 * Le motif, suffixé de ce qui le porte quand c'est connu — « … (6000) ».
	 *
	 * ⚠️ Sans ce suffixe, une écriture à dix lignes dont un compte est archivé
	 * affiche « réactivez-le » sans dire lequel.
	 */
	function blockedMessage(entry: JournalEntryDetailResponse): string {
		if (!entry.reversalBlockedBy) return '';
		const motif = blockedLabel(entry.reversalBlockedBy);
		return entry.reversalBlockedLabel ? `${motif} (${entry.reversalBlockedLabel})` : motif;
	}

	async function confirmReverse() {
		if (!entry || reversing) return;
		reversing = true;
		try {
			const created = await reverseJournalEntry(entry.id);
			toast.success(i18nMsg('journal-entries-reverse-success', 'Écriture contre-passée'));
			showReverseConfirm = false;
			await goto(`/journal-entries/${created.id}`);
		} catch (err) {
			// Le serveur porte le message traduit ET le chemin de correction :
			// on l'affiche tel quel plutôt que d'en fabriquer un ici.
			toast.error(
				isApiError(err) ? err.message : i18nMsg('error-unexpected', 'Erreur inattendue.')
			);
		} finally {
			reversing = false;
		}
	}
</script>

<svelte:head>
	<title>{entry ? `Écriture n°${entry.entryNumber}` : 'Écriture'} — Kesh</title>
</svelte:head>

<div class="mb-6 flex items-center justify-between">
	<Button variant="ghost" onclick={() => goto('/journal-entries')}>
		<ArrowLeft class="h-4 w-4" aria-hidden="true" />
		Retour
	</Button>
	<!-- ⛔ Le bouton est ABSENT, pas désactivé, quand l'écriture n'est pas
	     contre-passable : un bouton grisé n'explique rien. Le motif est affiché
	     à sa place, traduit depuis le code rendu par le serveur. -->
	{#if entry?.reversable}
		<Button
			variant="outline"
			data-testid="reverse-entry"
			onclick={() => (showReverseConfirm = true)}
		>
			{i18nMsg('journal-entries-reverse-action', 'Contre-passer')}
		</Button>
	{:else if entry?.reversalBlockedBy}
		<p class="text-sm text-text-muted" data-testid="reverse-blocked-reason">
			{blockedMessage(entry)}
		</p>
	{/if}
</div>

{#if loading}
	<p class="text-sm text-text-muted">Chargement…</p>
{:else if errorMsg}
	<div class="rounded-md border border-destructive bg-destructive/10 px-3 py-2 text-sm text-destructive">
		{errorMsg}
	</div>
{:else if entry}
	<h1 class="mb-4 text-2xl font-semibold text-text">Écriture n°{entry.entryNumber}</h1>

	<!-- Renvois croisés : la correction doit se VOIR depuis les deux bouts. -->
	{#if entry.reversesEntryId}
		<p class="mb-4 text-sm" data-testid="reverses-link">
			<a class="underline" href="/journal-entries/{entry.reversesEntryId}">
				{i18nMsg('journal-entries-reverses-link', "Contre-passe l'écriture n° { $number }", {
					number: entry.reversesEntryId
				})}
			</a>
		</p>
	{/if}
	{#if entry.reversedByEntryId}
		<p class="mb-4 text-sm" data-testid="reversed-by-link">
			<a class="underline" href="/journal-entries/{entry.reversedByEntryId}">
				{i18nMsg(
					'journal-entries-reversed-by-link',
					'Contre-passée par l\'écriture n° { $number }',
					{ number: entry.reversedByEntryId }
				)}
			</a>
		</p>
	{/if}

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

<!-- Confirmation de contre-passation — Story 24-4a (#380). -->
{#if showReverseConfirm && entry}
	<div
		class="fixed inset-0 z-50 flex items-center justify-center bg-black/50"
		role="dialog"
		aria-modal="true"
		aria-labelledby="reverse-confirm-title"
		aria-describedby="reverse-confirm-desc"
	>
		<div class="bg-card border border-border rounded-lg p-6 max-w-md mx-4 shadow-lg">
			<h2 id="reverse-confirm-title" class="text-lg font-semibold mb-2">
				{i18nMsg('journal-entries-reverse-dialog-title', 'Contre-passer cette écriture ?')}
			</h2>
			<p id="reverse-confirm-desc" class="text-sm text-text-muted mb-4">
				{i18nMsg(
					'journal-entries-reverse-dialog-body',
					"Kesh créera une écriture inverse à la date du jour. L'écriture d'origine reste intacte : c'est la correction qui doit se voir, pas disparaître."
				)}
			</p>
			<div class="flex justify-end gap-2">
				<Button
					type="button"
					variant="outline"
					onclick={() => (showReverseConfirm = false)}
					disabled={reversing}
				>
					{i18nMsg('journal-entries-reverse-cancel', 'Annuler')}
				</Button>
				<Button
					type="button"
					data-testid="reverse-entry-confirm"
					onclick={confirmReverse}
					disabled={reversing}
				>
					{i18nMsg('journal-entries-reverse-confirm', 'Contre-passer')}
				</Button>
			</div>
		</div>
	</div>
{/if}
