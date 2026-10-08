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
	import type { JournalEntryDetailResponse } from '$lib/features/journal-entries/journal-entries.types';
	import {
		MODIFICATION_LABEL_IN_MESSAGE,
		modificationBlockerLabel,
		reversalBlockerLabel
	} from '$lib/features/journal-entries/blocker-messages';
	import JournalEntryForm from '$lib/features/journal-entries/JournalEntryForm.svelte';
	import { listFiscalYears } from '$lib/features/fiscal-years/fiscal-years.api';
	import { fetchCompanyCurrent } from '$lib/features/settings/settings.api';
	import { getInvoiceSettings } from '$lib/features/invoices/invoices.api';
	import { authState } from '$lib/app/stores/auth.svelte';
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
	let accounts = $state<AccountResponse[]>([]);
	let accountsLoadError = $state(false);
	let accountsById = $derived(new Map(accounts.map((a) => [a.id, a])));
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
				accounts = accountsResult.value;
				accountsLoadError = false;
			} else {
				accounts = [];
				accountsLoadError = true;
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
	 * Le motif de contre-passation, suffixé de ce qui le porte quand c'est connu
	 * — « … (6000) ». Traduction : `reversalBlockerLabel` (Story 24-4a), extrait
	 * de cette page par la Story 15-8a pour être partagé avec le motif de
	 * modification.
	 *
	 * ⚠️ Sans ce suffixe, une écriture à dix lignes dont un compte est archivé
	 * affiche « réactivez-le » sans dire lequel.
	 */
	function blockedMessage(entry: JournalEntryDetailResponse): string {
		if (!entry.reversalBlockedBy) return '';
		const motif = reversalBlockerLabel(entry.reversalBlockedBy);
		return entry.reversalBlockedLabel ? `${motif} (${entry.reversalBlockedLabel})` : motif;
	}

	/**
	 * Le motif de modification (Story 15-8a, D8). L'étiquette est suffixée comme
	 * pour la contre-passation, sauf quand le message la porte déjà (nom de
	 * l'exercice postérieur clos, borne du verrou).
	 */
	function modificationMessage(entry: JournalEntryDetailResponse): string {
		const code = entry.modificationBlockedBy;
		if (!code) return '';
		const motif = modificationBlockerLabel(code, entry.modificationBlockedLabel);
		if (!entry.modificationBlockedLabel || MODIFICATION_LABEL_IN_MESSAGE.has(code)) return motif;
		return `${motif} (${entry.modificationBlockedLabel})`;
	}

	/**
	 * ⛔ Rôle **Consultation** (C-15-8-14) : ni « Modifier » ni « Contre-passer »,
	 * sans motif affiché. Le 403 du serveur reste le refus qui fait autorité.
	 */
	let canWrite = $derived(
		authState.currentUser?.role === 'Admin' || authState.currentUser?.role === 'Comptable'
	);

	/**
	 * Le motif de modification n'est affiché que s'il dit autre chose que celui de
	 * la contre-passation — deux fois la même phrase n'apprend rien.
	 */
	let showModificationReason = $derived(
		!!entry &&
			!entry.modifiable &&
			!!entry.modificationBlockedBy &&
			!(entry.reversalBlockedBy === entry.modificationBlockedBy && !entry.reversable)
	);

	// --- Mode édition (Story 15-8a, D8) ---------------------------------
	let editing = $state(false);
	let editLoading = $state(false);
	let editProjects = $state<ProjectResponse[]>([]);
	let editBooksLockedThrough = $state<string | null>(null);
	let editRecoverableAccountId = $state<number | null>(null);
	let editFiscalYear = $state<{ startDate: string; endDate: string } | null>(null);

	/**
	 * Ouvre le formulaire en mode édition, pré-rempli. Ses props se chargent au
	 * clic, comme la page liste les charge pour la création ; un échec dégrade
	 * le confort de saisie (bornes de date, projets), jamais l'enregistrement —
	 * le serveur tranche.
	 */
	async function startEdit() {
		if (!entry) return;
		editLoading = true;
		try {
			const [projectsR, companyR, settingsR, yearsR] = await Promise.allSettled([
				listProjects(),
				fetchCompanyCurrent(),
				getInvoiceSettings(),
				listFiscalYears()
			]);
			editProjects = projectsR.status === 'fulfilled' ? projectsR.value : [];
			editBooksLockedThrough =
				companyR.status === 'fulfilled' ? companyR.value.company.booksLockedThrough : null;
			editRecoverableAccountId =
				settingsR.status === 'fulfilled'
					? (settingsR.value.defaultVatRecoverableAccountId ?? null)
					: null;
			const fy =
				yearsR.status === 'fulfilled'
					? yearsR.value.find((y) => y.id === entry?.fiscalYearId)
					: undefined;
			editFiscalYear = fy ? { startDate: fy.startDate, endDate: fy.endDate } : null;
			editing = true;
		} finally {
			editLoading = false;
		}
	}

	/** Enregistrée : retour à la fiche, relue (même `id`, même numéro). */
	function onEditSaved() {
		editing = false;
		void loadEntry(id);
	}

	/** Le serveur dit que l'écriture a changé : la fiche se relit, sans modale. */
	function onEditStale() {
		editing = false;
		void loadEntry(id);
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
	<!-- ⛔ Les boutons sont ABSENTS, pas désactivés, quand le geste n'est pas
	     possible : un bouton grisé n'explique rien. Le motif est affiché à leur
	     place, traduit depuis le code rendu par le serveur. Rôle Consultation :
	     ni l'un ni l'autre, sans motif (C-15-8-14). -->
	{#if entry && canWrite && !editing}
		<div class="flex flex-col items-end gap-2">
			<div class="flex gap-2">
				{#if entry.modifiable}
					<Button
						variant="outline"
						data-testid="edit-entry"
						disabled={editLoading}
						onclick={startEdit}
					>
						{i18nMsg('journal-entry-edit', 'Modifier')}
					</Button>
				{/if}
				{#if entry.reversable}
					<Button
						variant="outline"
						data-testid="reverse-entry"
						onclick={() => (showReverseConfirm = true)}
					>
						{i18nMsg('journal-entries-reverse-action', 'Contre-passer')}
					</Button>
				{/if}
			</div>
			{#if showModificationReason}
				<p class="text-sm text-text-muted" data-testid="modification-blocked-reason">
					{modificationMessage(entry)}
				</p>
			{/if}
			{#if !entry.reversable && entry.reversalBlockedBy}
				<p class="text-sm text-text-muted" data-testid="reverse-blocked-reason">
					{blockedMessage(entry)}
				</p>
			{/if}
		</div>
	{/if}
</div>

{#if loading}
	<p class="text-sm text-text-muted">Chargement…</p>
{:else if errorMsg}
	<div class="rounded-md border border-destructive bg-destructive/10 px-3 py-2 text-sm text-destructive">
		{errorMsg}
	</div>
{:else if entry && editing}
	<h1 class="mb-4 text-2xl font-semibold text-text">Écriture n°{entry.entryNumber}</h1>
	<JournalEntryForm
		{accounts}
		{accountsLoadError}
		projects={editProjects}
		booksLockedThrough={editBooksLockedThrough}
		recoverableAccountId={editRecoverableAccountId}
		initialEntry={entry}
		entryFiscalYear={editFiscalYear}
		onSuccess={onEditSaved}
		onCancel={() => (editing = false)}
		onStale={onEditStale}
	/>
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
