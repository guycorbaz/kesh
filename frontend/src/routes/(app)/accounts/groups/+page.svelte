<script lang="ts">
	/**
	 * Story 46-5b — « Créer des groupes » : cocher des comptes, juger les
	 * propositions de Kesh une par une (Accepter, Écarter, Revenir), voir
	 * l'arbre qui en résultera, puis confirmer — tout ou rien, par
	 * `POST /api/v1/accounts/groups` (Story 46-5a). Toutes les règles vivent dans
	 * `group-proposals.ts` ; ce composant ne fait qu'afficher et transmettre.
	 */
	import { goto } from '$app/navigation';
	import { Button } from '$lib/components/ui/button';
	import { isApiError } from '$lib/shared/utils/api-client';
	import { authState } from '$lib/app/stores/auth.svelte';
	import { toast } from 'svelte-sonner';
	import { i18nMsg } from '$lib/shared/utils/i18n.svelte';
	import { fetchAccounts, applyGroups } from '$lib/features/accounts/accounts.api';
	import { listBankAccounts } from '$lib/features/bank-accounts/bank-accounts.api';
	import type { AccountResponse } from '$lib/features/accounts/accounts.types';
	import {
		applyEdit,
		buildGroupsRequest,
		checkProposals,
		bankLinkedAccounts,
		type BankLinks,
		parentOptionsFor,
		previewTree,
		proposalSkeletons,
		proposalSummary,
		readGroupRefusal,
		selectableAccounts,
		type CheckedProposal,
		type GroupMode,
		type ProposalEdit,
		type ProposalEdits,
	} from '$lib/features/accounts/group-proposals';
	import { proposalMessageText, typeLabel } from './messages';

	// fx-filtre-comptes : exempté — le plan comptable : tous les comptes, en devise
	// compris (un groupe réunit aussi des comptes en devise, 46-1 AC-6).
	// La liste archivés compris : les archivés comptent pour les numéros pris,
	// jamais comme membres, et ne s'affichent pas.
	let accounts = $state<AccountResponse[]>([]);
	let loading = $state(false);
	let selection = $state<Set<number>>(new Set());
	let mode = $state<GroupMode>('digit');
	let edits = $state<ProposalEdits>({});
	let onlyTouched = $state(false);
	let submitting = $state(false);

	/**
	 * Le refus du serveur, affiché tant que l'utilisateur n'a pas renvoyé.
	 * *Recharger le plan* s'offre sur **tout** refus : le geste est sans danger
	 * (il ne relit que les comptes et garde sélection, mode et choix), et bien
	 * des refus viennent d'un plan changé depuis l'aperçu — écriture reçue
	 * (`ACCOUNT_PARENT_HAS_ENTRIES`), numéro pris, groupe vidé, compte retypé
	 * (`ACCOUNT_GROUP_INVALID`), `RESOURCE_CONFLICT` (revue P1 de la 46-5b, M2).
	 */
	interface ServerRefusal {
		message: string;
		proposalKey: string | null;
		position: number | null;
		proposalNumber: string | null;
		accountNumber: string | null;
	}
	let refusal = $state<ServerRefusal | null>(null);

	let set = $derived(proposalSkeletons(accounts, selection, mode));
	// Les comptes désignés par un compte bancaire (clos et archivés compris) : l'aperçu les
	// refuse comme groupes, comme le serveur (`ACCOUNT_PARENT_BANK_LINKED`).
	let links = $state<BankLinks>(new Map());
	let checked = $derived(checkProposals(set, edits, accounts, selection, mode, links));
	let summary = $derived(proposalSummary(checked));
	let tree = $derived(previewTree(accounts, checked, onlyTouched));
	// Le plan à cocher : l'arbre actuel, sans proposition appliquée.
	let planRows = $derived(previewTree(accounts, []));
	let selectableIds = $derived(new Set(selectableAccounts(accounts).map((a) => a.id)));
	let byId = $derived(new Map(accounts.map((a) => [a.id, a])));

	async function loadAccounts() {
		loading = true;
		try {
			const [plan, banks] = await Promise.all([
				fetchAccounts(true),
				// Sans la liste des comptes bancaires, l'aperçu ne les signale pas ;
				// le serveur refuse quand même (`ACCOUNT_PARENT_BANK_LINKED`).
				listBankAccounts(true, true).catch(() => []),
			]);
			accounts = plan;
			links = bankLinkedAccounts(banks);
		} catch (err) {
			toast.error(isApiError(err) ? err.message : i18nMsg('error-unexpected', 'Erreur inattendue.'));
		} finally {
			loading = false;
		}
	}

	$effect(() => {
		if (authState.isAuthenticated) void loadAccounts();
	});

	function toggle(id: number, on: boolean) {
		const next = new Set(selection);
		if (on) next.add(id);
		else next.delete(id);
		selection = next;
	}

	function selectAll() {
		selection = new Set(selectableIds);
	}

	function selectNone() {
		selection = new Set();
	}

	/** Range le geste : seuls les champs touchés sont gardés (`applyEdit`). */
	function update(p: CheckedProposal, patch: Partial<ProposalEdit>) {
		edits = applyEdit(edits, p, patch);
	}

	function toggleMember(p: CheckedProposal, id: number, on: boolean) {
		const excluded = on ? p.edit.excluded.filter((x) => x !== id) : [...p.edit.excluded, id];
		update(p, { excluded });
	}

	function parentValue(p: CheckedProposal): string {
		return p.edit.parent === null ? '' : String(p.edit.parent);
	}

	function setParent(p: CheckedProposal, value: string) {
		update(p, { parent: value === '' ? null : value === 'root' ? 'root' : Number(value) });
	}

	function kindLabel(p: CheckedProposal): string {
		if (p.kind === 'attach') return i18nMsg('accounts-groups-kind-attach', 'Rattacher au groupe');
		if (p.kind === 'reuse') return i18nMsg('accounts-groups-kind-reuse', 'Réutiliser comme groupe');
		return i18nMsg('accounts-groups-kind-create', 'Nouveau groupe');
	}

	function messageClass(level: string): string {
		if (level === 'error') return 'bg-red-50 text-red-800';
		if (level === 'warning') return 'bg-amber-50 text-amber-800';
		return 'bg-primary/10 text-text';
	}

	function cardClass(p: CheckedProposal): string {
		if (p.edit.state === 'rejected') return 'opacity-60 border-l-border';
		const accepted = p.edit.state === 'accepted' ? ' bg-green-50' : '';
		if (p.hasError) return `border-l-red-600${accepted}`;
		if (p.messages.some((m) => m.level === 'warning')) return `border-l-amber-500${accepted}`;
		return `border-l-green-600${accepted}`;
	}

	/** Le message d'un code existant, tel que le donne l'écran du plan ; sinon celui du serveur. */
	function refusalMessage(code: string, message: string): string {
		if (code === 'OPTIMISTIC_LOCK_CONFLICT') {
			return i18nMsg('accounts-error-stale', 'Les données ont été modifiées. Rechargez la page.');
		}
		if (code === 'RESOURCE_CONFLICT') {
			return i18nMsg('accounts-error-number-exists', 'Ce numéro de compte existe déjà.');
		}
		return message;
	}

	async function confirm() {
		if (!summary.canConfirm || submitting) return;
		const { request, keys } = buildGroupsRequest(checked);
		submitting = true;
		refusal = null;
		try {
			const res = await applyGroups(request);
			toast.success(
				i18nMsg(
					'accounts-groups-success',
					'{ $created } groupe(s) créé(s), { $reused } compte(s) réutilisé(s), { $moved } compte(s) déplacé(s).',
					{ created: res.created.length, reused: res.reused.length, moved: res.moved.length },
				),
			);
			await goto('/accounts');
		} catch (err) {
			if (!isApiError(err)) {
				refusal = {
					message: i18nMsg('error-unexpected', 'Erreur inattendue.'),
					proposalKey: null,
					position: null,
					proposalNumber: null,
					accountNumber: null,
				};
				return;
			}
			const target = readGroupRefusal(err.details);
			const key = target ? (keys[target.groupIndex] ?? null) : null;
			const proposal = key ? checked.find((p) => p.key === key) : undefined;
			const account = target?.accountId != null ? byId.get(target.accountId) : undefined;
			refusal = {
				message: refusalMessage(err.code, err.message),
				proposalKey: key,
				position: target ? target.groupIndex + 1 : null,
				proposalNumber: proposal ? proposal.edit.number.trim() : null,
				accountNumber: account ? account.number : null,
			};
		} finally {
			submitting = false;
		}
	}

	async function reload() {
		refusal = null;
		await loadAccounts();
	}
</script>

<svelte:head>
	<title>{i18nMsg('accounts-groups-title', 'Plan comptable — Créer des groupes')} - Kesh</title>
</svelte:head>

<div class="mb-4 flex flex-wrap items-start justify-between gap-3">
	<div>
		<h1 class="text-2xl font-semibold text-text">
			{i18nMsg('accounts-groups-title', 'Plan comptable — Créer des groupes')}
		</h1>
		<p class="mt-1 max-w-3xl text-sm text-text-muted">
			{i18nMsg(
				'accounts-groups-lead',
				"Kesh propose des comptes de groupe d'après les numéros des comptes que vous cochez. Rien n'est créé ni déplacé avant votre confirmation.",
			)}
		</p>
	</div>
	<Button variant="outline" href="/accounts" data-testid="account-groups-back">
		{i18nMsg('accounts-groups-back', 'Retour au plan comptable')}
	</Button>
</div>

{#if refusal}
	<div class="mb-4 rounded-lg border border-red-300 bg-red-50 p-3 text-sm text-red-800" role="alert" data-testid="account-groups-refusal">
		<p class="font-medium">{i18nMsg('accounts-groups-error-title', "Rien n'a été écrit.")}</p>
		{#if refusal.position !== null}
			<p data-testid="account-groups-refusal-step">
				{i18nMsg('accounts-groups-error-step', 'Proposition { $position } ({ $number })', {
					position: refusal.position,
					number: refusal.proposalNumber ?? '',
				})}{#if refusal.accountNumber}, {i18nMsg('accounts-groups-error-account', 'compte { $number }', {
						number: refusal.accountNumber,
					})}{/if}
			</p>
		{/if}
		<p data-testid="account-groups-refusal-message">{refusal.message}</p>
		<Button variant="outline" size="sm" class="mt-2" onclick={reload} data-testid="account-groups-reload">
			{i18nMsg('accounts-groups-reload', 'Recharger le plan')}
		</Button>
	</div>
{/if}

{#if loading && accounts.length === 0}
	<p class="text-sm text-text-muted">{i18nMsg('common-loading', 'Chargement…')}</p>
{:else}
	<div class="grid grid-cols-1 items-start gap-4 xl:grid-cols-[minmax(240px,1fr)_minmax(360px,1.5fr)_minmax(260px,1.1fr)]">
		<!-- 1. Sélection -->
		<section class="rounded-lg border border-border bg-surface p-4" aria-labelledby="groups-step-1" data-testid="account-groups-selection">
			<h2 id="groups-step-1" class="mb-2 text-base font-semibold text-text">
				1. {i18nMsg('accounts-groups-step-select', 'Choisir les comptes')}
			</h2>
			<p class="mb-2 text-xs text-text-muted">
				{i18nMsg(
					'accounts-groups-select-hint',
					'Cochez les comptes à ranger. Les comptes de groupe (en gras) ne se cochent pas : ils servent de parent.',
				)}
			</p>
			<div class="mb-2 flex gap-2">
				<Button variant="outline" size="xs" onclick={selectAll} data-testid="account-groups-select-all">
					{i18nMsg('accounts-groups-select-all', 'Tout le plan')}
				</Button>
				<Button variant="outline" size="xs" onclick={selectNone} data-testid="account-groups-select-none">
					{i18nMsg('accounts-groups-select-none', 'Aucun')}
				</Button>
			</div>
			<div class="max-h-[70vh] overflow-y-auto text-sm">
				{#each planRows as row (row.key)}
					{@const id = Number(row.key.slice(1))}
					<div
						class="flex items-center gap-2 border-b border-border py-0.5 {row.isGroup ? 'font-semibold' : ''}"
						style="padding-left: {row.depth * 12}px"
						data-testid="account-groups-plan-{row.number}"
					>
						{#if selectableIds.has(id)}
							<input
								type="checkbox"
								class="h-4 w-4 rounded border-border"
								checked={selection.has(id)}
								onchange={(e) => toggle(id, e.currentTarget.checked)}
								aria-label={i18nMsg('accounts-groups-select-aria', 'Choisir le compte { $number }', { number: row.number })}
								data-testid="account-groups-check-{row.number}"
							/>
						{:else}
							<span class="inline-block w-4"></span>
						{/if}
						<span class="font-mono text-text-muted">{row.number}</span>
						<span class="truncate text-text">{row.name}</span>
						{#if byId.get(id)?.currencyLocked !== true && !row.isGroup}
							<span class="whitespace-nowrap rounded-full border border-border px-1.5 text-xs text-text-muted">
								{i18nMsg('accounts-groups-no-entries', 'sans écriture')}
							</span>
						{/if}
					</div>
				{/each}
			</div>
		</section>

		<!-- 2. Propositions -->
		<section class="rounded-lg border border-border bg-surface p-4" aria-labelledby="groups-step-2" data-testid="account-groups-proposals">
			<h2 id="groups-step-2" class="mb-2 text-base font-semibold text-text">
				2. {i18nMsg('accounts-groups-step-proposals', 'Propositions')}
			</h2>
			<div class="mb-2 inline-flex overflow-hidden rounded-md border border-border" role="group">
				<button
					type="button"
					class="px-3 py-1 text-sm {mode === 'digit' ? 'bg-primary text-primary-foreground' : 'bg-surface text-text'}"
					aria-pressed={mode === 'digit'}
					onclick={() => (mode = 'digit')}
					data-testid="account-groups-mode-digit"
				>
					{i18nMsg('accounts-groups-mode-digit', 'Un groupe par numéro sans son dernier chiffre')}
				</button>
				<button
					type="button"
					class="px-3 py-1 text-sm {mode === 'one' ? 'bg-primary text-primary-foreground' : 'bg-surface text-text'}"
					aria-pressed={mode === 'one'}
					onclick={() => (mode = 'one')}
					data-testid="account-groups-mode-one"
				>
					{i18nMsg('accounts-groups-mode-one', 'Un seul groupe pour la sélection')}
				</button>
			</div>
			<p class="mb-3 text-xs text-text-muted">
				{#if mode === 'digit'}
					{i18nMsg(
						'accounts-groups-mode-digit-hint',
						'Un groupe proposé pour chaque préfixe partagé par au moins deux comptes cochés (p. ex. 1170, 1171 → 117).',
					)}
				{:else}
					{i18nMsg(
						'accounts-groups-mode-one-hint',
						'Un seul groupe pour tous les comptes cochés : le numéro proposé est leur plus long préfixe commun, à changer au besoin.',
					)}
				{/if}
			</p>

			{#each checked as p (p.key)}
				<div
					class="mb-3 rounded-md border border-l-4 border-border p-3 {cardClass(p)} {refusal?.proposalKey === p.key ? 'ring-2 ring-red-500' : ''}"
					data-testid="account-groups-proposal-{p.key}"
					data-state={p.edit.state}
					data-kind={p.kind}
				>
					<div class="flex flex-wrap items-center gap-2">
						<h3 class="text-sm font-semibold text-text">{kindLabel(p)}</h3>
						<span class="rounded-full border border-border px-2 text-xs text-text-muted">
							{i18nMsg('accounts-groups-moves', '{ $count } compte(s) à déplacer', { count: p.moves.length })}
						</span>
						{#if p.edit.state === 'accepted'}
							<span class="rounded-full bg-green-100 px-2 text-xs text-green-800">{i18nMsg('accounts-groups-state-accepted', 'acceptée')}</span>
						{:else if p.edit.state === 'rejected'}
							<span class="rounded-full border border-border px-2 text-xs text-text-muted">{i18nMsg('accounts-groups-state-rejected', 'écartée')}</span>
						{/if}
					</div>
					<div class="mt-2 flex flex-wrap items-end gap-2 text-sm">
						<label class="flex flex-col text-xs text-text-muted">
							{i18nMsg('accounts-groups-field-number', 'Numéro')}
							<input
								type="text"
								class="w-24 rounded border border-border px-2 py-1 font-mono text-sm text-text"
								value={p.edit.number}
								oninput={(e) => update(p, { number: e.currentTarget.value })}
								data-testid="account-groups-number"
							/>
						</label>
						<label class="flex flex-col text-xs text-text-muted">
							{i18nMsg('accounts-groups-field-name', 'Nom')}
							<input
								type="text"
								class="w-48 rounded border border-border px-2 py-1 text-sm text-text disabled:bg-surface-alt"
								value={p.kind === 'create' ? p.edit.name : (p.existing?.name ?? '')}
								disabled={p.kind !== 'create'}
								oninput={(e) => update(p, { name: e.currentTarget.value })}
								data-testid="account-groups-name"
							/>
						</label>
						{#if p.kind === 'create'}
							<label class="flex flex-col text-xs text-text-muted">
								{i18nMsg('accounts-groups-field-parent', 'Parent')}
								<select
									class="rounded border border-border px-2 py-1 text-sm text-text"
									value={parentValue(p)}
									onchange={(e) => setParent(p, e.currentTarget.value)}
									data-testid="account-groups-parent"
								>
									<option value="">{i18nMsg('accounts-groups-parent-choose', '(à choisir)')}</option>
									<option value="root">{i18nMsg('accounts-groups-parent-root', '(racine)')}</option>
									{#each parentOptionsFor(accounts, p, links) as o (o.id)}
										<option value={String(o.id)}>{o.number} {o.name}</option>
									{/each}
								</select>
							</label>
						{/if}
					</div>
					<ul class="mt-2 space-y-0.5 text-sm">
						{#each p.members as m (m.id)}
							<li>
								<label class="flex items-center gap-2">
									<input
										type="checkbox"
										class="h-4 w-4 rounded border-border"
										checked={!p.edit.excluded.includes(m.id)}
										onchange={(e) => toggleMember(p, m.id, e.currentTarget.checked)}
										data-testid="account-groups-member-{m.number}"
									/>
									<span class="font-mono">{m.number}</span>
									<span>{m.name}</span>
									<span class="text-xs text-text-muted">{typeLabel(m.accountType)}{m.currency ? ` · ${m.currency}` : ''}</span>
								</label>
							</li>
						{/each}
					</ul>
					{#each p.messages as msg, i (i)}
						<p class="mt-1 rounded px-2 py-1 text-xs {messageClass(msg.level)}" data-testid="account-groups-msg-{msg.level}" data-code={msg.code}>
							{proposalMessageText(msg)}
						</p>
					{/each}
					{#if p.canReuse}
						<label class="mt-2 flex items-center gap-2 text-sm">
							<input
								type="checkbox"
								class="h-4 w-4 rounded border-border"
								checked={p.edit.reuse}
								onchange={(e) => update(p, { reuse: e.currentTarget.checked })}
								data-testid="account-groups-reuse"
							/>
							{i18nMsg('accounts-groups-reuse', 'Réutiliser ce compte comme groupe')}
						</label>
					{/if}
					<div class="mt-2 flex gap-2">
						<Button
							size="xs"
							disabled={p.hasError || p.edit.state === 'accepted'}
							onclick={() => update(p, { state: 'accepted' })}
							data-testid="account-groups-accept"
						>
							{i18nMsg('accounts-groups-accept', 'Accepter')}
						</Button>
						<Button
							variant="outline"
							size="xs"
							disabled={p.edit.state === 'rejected'}
							onclick={() => update(p, { state: 'rejected' })}
							data-testid="account-groups-reject"
						>
							{i18nMsg('accounts-groups-reject', 'Écarter')}
						</Button>
						{#if p.edit.state !== 'pending'}
							<Button variant="outline" size="xs" onclick={() => update(p, { state: 'pending' })} data-testid="account-groups-reset">
								{i18nMsg('accounts-groups-reset', 'Revenir')}
							</Button>
						{/if}
					</div>
				</div>
			{/each}
			{#if checked.length === 0}
				<p class="text-sm text-text-muted" data-testid="account-groups-no-proposal">
					{i18nMsg('accounts-groups-no-proposal', 'Aucune proposition : cochez au moins deux comptes de même préfixe.')}
				</p>
			{/if}
			{#if set.singles.length > 0}
				<p class="mt-2 rounded bg-primary/10 px-2 py-1 text-xs text-text" data-testid="account-groups-singles">
					{i18nMsg(
						'accounts-groups-singles',
						'Seuls de leur préfixe, sans groupe proposé : { $numbers }. Le mode « un seul groupe » permet de les grouper quand même.',
						{ numbers: set.singles.map((s) => s.number).join(', ') },
					)}
				</p>
			{/if}
		</section>

		<!-- 3. Aperçu -->
		<section class="rounded-lg border border-border bg-surface p-4" aria-labelledby="groups-step-3" data-testid="account-groups-preview">
			<h2 id="groups-step-3" class="mb-2 text-base font-semibold text-text">
				3. {i18nMsg('accounts-groups-step-preview', "Aperçu de l'arbre")}
			</h2>
			<p class="mb-2 text-xs text-text-muted">
				{i18nMsg(
					'accounts-groups-preview-hint',
					'Le plan tel qu’il sera après confirmation. Seules les propositions acceptées et sans erreur comptent.',
				)}
			</p>
			<label class="mb-2 flex items-center gap-2 text-sm">
				<input type="checkbox" class="h-4 w-4 rounded border-border" bind:checked={onlyTouched} data-testid="account-groups-only-touched" />
				{i18nMsg('accounts-groups-only-touched', 'Montrer seulement les branches modifiées')}
			</label>
			<div class="max-h-[70vh] overflow-y-auto text-sm">
				{#each tree as row (row.key)}
					<div
						class="truncate py-0.5 {row.isGroup ? 'font-semibold' : ''}"
						style="padding-left: {row.depth * 14}px"
						data-testid="account-groups-tree-{row.number}"
						data-depth={row.depth}
						data-mark={row.mark ?? ''}
					>
						<span class="font-mono">{row.number}</span>
						{row.name}
						{#if row.mark === 'new'}
							<span class="rounded-full bg-primary/10 px-1.5 text-xs text-primary">{i18nMsg('accounts-groups-mark-new', 'nouveau')}</span>
						{:else if row.mark === 'moved'}
							<span class="rounded-full bg-green-100 px-1.5 text-xs text-green-800">{i18nMsg('accounts-groups-mark-moved', 'déplacé')}</span>
						{:else if row.mark === 'becomesGroup'}
							<span class="rounded-full bg-amber-100 px-1.5 text-xs text-amber-800">{i18nMsg('accounts-groups-mark-becomes-group', 'devient groupe')}</span>
						{/if}
						{#if row.depth === 0}
							<span class="text-xs text-text-muted">{typeLabel(row.accountType)}</span>
						{/if}
					</div>
				{/each}
			</div>
		</section>
	</div>
{/if}

<!-- Pied : décompte et confirmation -->
<div class="sticky bottom-0 z-10 mt-4 flex flex-wrap items-center gap-3 rounded-t-lg border border-border bg-surface px-4 py-3 shadow-sm" data-testid="account-groups-footer">
	<div class="min-w-60 flex-1 text-sm text-text">
		<span data-testid="account-groups-summary">
			{i18nMsg(
				'accounts-groups-summary',
				'{ $created } groupe(s) à créer, { $reused } compte(s) réutilisé(s), { $moved } compte(s) à déplacer.',
				{ created: summary.created, reused: summary.reused, moved: summary.moved },
			)}
		</span>
		<span class="text-xs text-text-muted">
			{i18nMsg('accounts-groups-summary-pending', 'Tout ou rien, en une fois. { $pending } proposition(s) sans réponse.', {
				pending: summary.pending,
			})}
		</span>
		{#if summary.acceptedWithErrors > 0}
			<span class="text-xs text-red-700" data-testid="account-groups-summary-errors">
				{i18nMsg('accounts-groups-summary-errors', '{ $count } proposition(s) acceptée(s) portent une erreur.', {
					count: summary.acceptedWithErrors,
				})}
			</span>
		{/if}
		{#if summary.tooMany}
			<span class="text-xs text-red-700" data-testid="account-groups-too-many">
				{i18nMsg('accounts-groups-too-many', 'Demande trop grande : au plus 100 propositions et 1000 comptes à ranger.')}
			</span>
		{/if}
	</div>
	<Button disabled={!summary.canConfirm || submitting} onclick={confirm} data-testid="account-groups-confirm">
		{submitting
			? i18nMsg('accounts-groups-confirming', 'Enregistrement…')
			: i18nMsg('accounts-groups-confirm', 'Confirmer')}
	</Button>
</div>
