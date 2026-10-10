<script lang="ts">
	// Story 15-1c-i (#518) — l'écran « Postes ouverts » : choisir un compte et une
	// date, voir ce qui y reste ouvert et pourquoi, lettrer à la main ou sur
	// proposition, ouvrir et délettrer un groupe.
	//
	// ⚠️ **L'URL est la source de vérité** (AC1, AC6) : `accountId`, `asOf`,
	// `group`. Un effet relit l'URL et charge ce qui a changé ; les gestes de
	// l'utilisateur ne font qu'**écrire** l'URL (`navigate`, remplacement
	// d'historique). Un lien de code cliqué dans l'écran (`?group=`) passe donc
	// par le même chemin qu'un lien venu d'ailleurs.
	//
	// ⛔ **Tout refus 404 ou 409 dit que la liste est périmée** (C-15-1c-17) :
	// liste et propositions rechargées, sélection effacée, sans liste de codes —
	// `LETTERING_CONCURRENT_CHANGE` compris (C-15-1c-24).
	import { untrack } from 'svelte';
	import { SvelteMap } from 'svelte/reactivity';
	import { i18nMsg } from '$lib/shared/utils/i18n.svelte';
	import { isApiError } from '$lib/shared/utils/api-client';
	import { authState } from '$lib/app/stores/auth.svelte';
	import { fetchAccounts } from '$lib/features/accounts/accounts.api';
	import type { AccountResponse } from '$lib/features/accounts/accounts.types';
	import LetteringCodeLink from './LetteringCodeLink.svelte';
	import OpenItemsTable from './OpenItemsTable.svelte';
	import ProposalsPanel from './ProposalsPanel.svelte';
	import LetteringGroupPanel from './LetteringGroupPanel.svelte';
	import {
		createLettering,
		deleteLettering,
		fetchLettering,
		fetchOpenItems,
		fetchProposals,
		OPEN_ITEMS_PAGE_SIZE,
	} from './open-items.api';
	import {
		isIsoDate,
		isStaleRefusal,
		letterBlocker,
		parseScreenState,
		screenUrl,
		selectionSum,
		todayLocal,
		type ScreenState,
	} from './open-items';
	import { formatAmount, letterBlockerLabel } from './open-items-labels';
	import type {
		GroupState,
		OpenItem,
		OpenItemsResponse,
		Proposal,
		ProposalsState,
		SelectedLine,
	} from './open-items.types';

	interface Props {
		/** L'URL courante (`page.url`) — relue à chaque changement. */
		url: URL;
		/** Écrit l'URL, par remplacement d'historique. */
		navigate: (url: URL) => void;
	}
	let { url, navigate }: Props = $props();

	type ListState =
		| { status: 'idle' }
		| { status: 'loading' }
		| { status: 'ready'; data: OpenItemsResponse }
		| { status: 'refused'; message: string }
		| { status: 'notFound' }
		| { status: 'error' };

	type Message =
		| { kind: 'error'; text: string }
		| { kind: 'lettered'; code: string }
		| { kind: 'dissolved'; code: string };

	const today = todayLocal();

	let accounts = $state<AccountResponse[]>([]);
	let accountsStatus = $state<'loading' | 'ready' | 'error'>('loading');
	let accountId = $state<number | null>(null);
	let asOf = $state<string>('');
	let groupCode = $state<string | null>(null);
	let offset = $state(0);
	let list = $state<ListState>({ status: 'idle' });
	let proposals = $state<ProposalsState | null>(null);
	let group = $state<GroupState | null>(null);
	let message = $state<Message | null>(null);
	let busy = $state(false);
	let codeInput = $state('');
	/** La sélection, conservée d'une page à l'autre (C-15-1c-4). */
	const selection = new SvelteMap<number, SelectedLine>();

	// Numéros de requête : une réponse arrivée après une plus récente est ignorée.
	let listSeq = 0;
	let proposalsSeq = 0;
	let groupSeq = 0;

	let canWrite = $derived(
		authState.currentUser?.role === 'Admin' || authState.currentUser?.role === 'Comptable',
	);
	let selected = $derived([...selection.values()]);
	let blocker = $derived(letterBlocker(selected));
	let pageIds = $derived(
		list.status === 'ready' ? new Set(list.data.items.map((i) => i.lineId)) : new Set<number>(),
	);
	let offPage = $derived(selected.filter((l) => !pageIds.has(l.lineId)).length);
	let showProposals = $derived(
		accountId !== null && list.status !== 'refused' && list.status !== 'notFound',
	);

	function current(): ScreenState {
		return { accountId, asOf, group: groupCode };
	}

	// --- chargements ---------------------------------------------------------

	async function loadAccounts() {
		try {
			// Archivés compris : un compte archivé reste lettrable (C96).
			accounts = (await fetchAccounts(true)).filter((a) => a.letterable);
			accountsStatus = 'ready';
		} catch {
			accounts = [];
			accountsStatus = 'error';
		}
	}

	async function loadList() {
		// Le numéro avance AUSSI quand rien n'est demandé : une réponse encore en vol
		// pour l'état précédent ne doit pas réapparaître (revue P1, E-1).
		const seq = ++listSeq;
		if (accountId === null) {
			list = { status: 'idle' };
			return;
		}
		list = { status: 'loading' };
		try {
			let data = await fetchOpenItems(accountId, asOf, offset, OPEN_ITEMS_PAGE_SIZE);
			// Une page devenue vide (sa dernière ligne lettrée, une écriture supprimée) :
			// la vue se rabat sur la dernière page non vide — le serveur ne borne pas
			// l'offset (revue P1, B-1 = E-2).
			if (seq === listSeq && data.items.length === 0 && offset > 0) {
				// Ensemble devenu vide : la page 1 (revue P2, E2-5).
				offset =
					data.total > 0
						? Math.floor((data.total - 1) / OPEN_ITEMS_PAGE_SIZE) * OPEN_ITEMS_PAGE_SIZE
						: 0;
				data = await fetchOpenItems(accountId, asOf, offset, OPEN_ITEMS_PAGE_SIZE);
			}
			if (seq === listSeq) list = { status: 'ready', data };
		} catch (err) {
			if (seq !== listSeq) return;
			if (isApiError(err) && err.status === 409) list = { status: 'refused', message: err.message };
			else if (isApiError(err) && err.status === 404) list = { status: 'notFound' };
			else list = { status: 'error' };
		}
	}

	async function loadProposals() {
		const seq = ++proposalsSeq;
		if (accountId === null) {
			proposals = null;
			return;
		}
		proposals = { status: 'loading' };
		try {
			const data = await fetchProposals(accountId);
			if (seq === proposalsSeq) proposals = { status: 'ready', data };
		} catch (err) {
			if (seq !== proposalsSeq) return;
			proposals = {
				status: 'error',
				message:
					isApiError(err) && err.status === 422
						? err.message
						: i18nMsg(
								'open-items-proposals-error',
								"Les rapprochements proposés n'ont pas pu être chargés.",
							),
			};
		}
	}

	async function loadGroup() {
		const code = groupCode;
		const seq = ++groupSeq;
		if (code === null) {
			group = null;
			return;
		}
		group = { status: 'loading', code };
		try {
			const data = await fetchLettering(code);
			if (seq === groupSeq) group = { status: 'ready', data };
		} catch (err) {
			if (seq !== groupSeq) return;
			group =
				isApiError(err) && err.status === 404
					? { status: 'notFound', code }
					: {
							status: 'error',
							code,
							message: i18nMsg('open-items-group-error', "Le groupe n'a pas pu être chargé."),
						};
		}
	}

	/** Après un geste (réussi ou périmé) : la liste et les propositions se relisent. */
	function reloadListAndProposals() {
		void loadList();
		void loadProposals();
	}

	// --- l'URL, source de vérité ------------------------------------------------

	function applyState(s: ScreenState) {
		if (s.asOf === null) {
			// Absente ou mal formée : la date locale du jour, ÉCRITE dans l'URL
			// (remplacement) — un rechargement ou un lien copié retombe sur la même vue.
			navigate(screenUrl(url, { ...s, asOf: today }));
			return;
		}
		const accountChanged = s.accountId !== accountId;
		const dateChanged = s.asOf !== asOf;
		accountId = s.accountId;
		asOf = s.asOf;
		if (accountChanged || dateChanged) {
			offset = 0;
			selection.clear();
			// Un message (et le lien de code qu'il porte) parle de la vue précédente.
			message = null;
			void loadList();
		}
		// Les propositions ne suivent pas la date (calculées aujourd'hui, AC5).
		if (accountChanged) void loadProposals();
		if (s.group !== groupCode) {
			groupCode = s.group;
			if (groupCode !== null) message = null;
			void loadGroup();
		}
	}

	$effect(() => {
		const s = parseScreenState(url.searchParams);
		untrack(() => applyState(s));
	});

	$effect(() => {
		untrack(() => void loadAccounts());
	});

	// --- gestes ---------------------------------------------------------------

	function onAccountChange(e: Event) {
		const v = (e.currentTarget as HTMLSelectElement).value;
		navigate(screenUrl(url, { ...current(), accountId: v === '' ? null : Number(v) }));
	}

	function onDateChange(e: Event) {
		const v = (e.currentTarget as HTMLInputElement).value;
		// Une valeur intermédiaire de la frappe (année `0002`, `0020`…) est ignorée ici,
		// jamais réécrite : la restaurer interromprait la saisie (revue P2, B2-1 = E2-1).
		if (isIsoDate(v)) navigate(screenUrl(url, { ...current(), asOf: v }));
	}

	/** À la sortie du champ, une date vide ou incomplète retrouve celle de la vue (revue P1, E-4). */
	function onDateBlur(e: FocusEvent) {
		const input = e.currentTarget as HTMLInputElement;
		if (!isIsoDate(input.value)) input.value = asOf;
	}

	function onPage(next: number) {
		offset = next;
		void loadList();
	}

	function toggle(item: OpenItem) {
		if (selection.has(item.lineId)) selection.delete(item.lineId);
		else
			selection.set(item.lineId, {
				lineId: item.lineId,
				debit: item.debit,
				credit: item.credit,
				inOpenPeriod: item.inOpenPeriod,
			});
	}

	function openCode(e: SubmitEvent) {
		e.preventDefault();
		const code = codeInput.trim();
		if (code === '') return;
		// Le même code ressaisi (après un échec, ou pour relire) : l'URL ne change pas,
		// la lecture se refait ici (revue P1, E-9).
		if (code === groupCode) {
			message = null;
			void loadGroup();
		}
		else navigate(screenUrl(url, { ...current(), group: code }));
	}

	function closeGroup() {
		navigate(screenUrl(url, { ...current(), group: null }));
	}

	function showGroupAccount() {
		if (group?.status === 'ready') {
			navigate(screenUrl(url, { ...current(), accountId: group.data.accountId }));
		}
	}

	/** Le texte d'un refus : son message (déjà traduit par le serveur), ou celui de l'écran. */
	function refusalText(err: unknown, notFoundText: string): string {
		if (isApiError(err)) return err.status === 404 ? notFoundText : err.message;
		return i18nMsg('open-items-action-error', "L'opération a échoué.");
	}

	async function letter(lineIds: number[]) {
		busy = true;
		message = null;
		try {
			const created = await createLettering(lineIds);
			message = { kind: 'lettered', code: created.code };
			selection.clear();
			reloadListAndProposals();
		} catch (err) {
			message = {
				kind: 'error',
				text: refusalText(
					err,
					i18nMsg(
						'open-items-line-gone',
						"Une ligne sélectionnée n'existe plus : son écriture a été modifiée ou supprimée.",
					),
				),
			};
			if (isStaleRefusal(err)) {
				selection.clear();
				reloadListAndProposals();
			}
		} finally {
			busy = false;
		}
	}

	function letterSelection() {
		if (blocker === null) void letter(selected.map((l) => l.lineId));
	}

	function acceptProposal(p: Proposal) {
		void letter([p.debit.lineId, p.credit.lineId]);
	}

	async function dissolve(code: string) {
		busy = true;
		message = null;
		try {
			await deleteLettering(code);
			message = { kind: 'dissolved', code };
			reloadListAndProposals();
			navigate(screenUrl(url, { ...current(), group: null }));
		} catch (err) {
			message = {
				kind: 'error',
				text: refusalText(
					err,
					i18nMsg('open-items-group-not-found', 'Aucun groupe ne porte ce code.'),
				),
			};
			if (isStaleRefusal(err)) {
				selection.clear();
				void loadGroup();
				reloadListAndProposals();
			}
		} finally {
			busy = false;
		}
	}
</script>

<div class="space-y-4" data-testid="open-items-screen">
	<h1 class="text-2xl font-bold">{i18nMsg('open-items-title', 'Postes ouverts')}</h1>

	<p class="rounded border-l-4 border-blue-400 bg-blue-50 p-3 text-sm text-blue-900" data-testid="open-items-boundary">
		{i18nMsg(
			'open-items-boundary',
			"Cet écran solde entre elles les lignes des comptes de tiers et de passage. Le rapprochement des relevés bancaires se fait dans Mensuel → Réconciliation ; les comptes bancaires n'apparaissent pas ici.",
		)}
	</p>

	<div class="flex flex-wrap items-end gap-4">
		<label class="flex flex-col gap-1 text-sm">
			<span>{i18nMsg('open-items-account-label', 'Compte')}</span>
			<select
				class="rounded border px-2 py-1"
				value={accountId === null ? '' : String(accountId)}
				onchange={onAccountChange}
				disabled={busy}
				data-testid="open-items-account"
			>
				<option value="">{i18nMsg('open-items-account-placeholder', 'Choisissez un compte')}</option>
				{#each accounts as a (a.id)}
					<option value={String(a.id)}>
						{a.number} — {a.name}{#if !a.active}
							({i18nMsg('open-items-account-archived', 'archivé')}){/if}
					</option>
				{/each}
			</select>
		</label>
		<label class="flex flex-col gap-1 text-sm">
			<span>{i18nMsg('open-items-as-of-label', 'Date')}</span>
			<input
				type="date"
				class="rounded border px-2 py-1"
				value={asOf}
				onchange={onDateChange}
				onblur={onDateBlur}
				disabled={busy}
				data-testid="open-items-as-of"
			/>
		</label>
		<form class="flex items-end gap-2 text-sm" onsubmit={openCode}>
			<label class="flex flex-col gap-1">
				<span>{i18nMsg('open-items-code-label', 'Code')}</span>
				<input
					type="text"
					class="w-28 rounded border px-2 py-1 font-mono"
					bind:value={codeInput}
					data-testid="open-items-code"
				/>
			</label>
			<button type="submit" class="rounded border px-3 py-1" data-testid="open-items-code-open"
				>{i18nMsg('open-items-code-open', 'Ouvrir le groupe')}</button
			>
		</form>
	</div>

	{#if accountsStatus === 'error'}
		<p class="rounded bg-red-50 p-3 text-sm text-red-800" role="alert" data-testid="open-items-accounts-error">
			{i18nMsg('open-items-accounts-error', "La liste des comptes n'a pas pu être chargée.")}
		</p>
	{:else if accountsStatus === 'ready' && accounts.length === 0}
		<p class="text-sm text-gray-600" data-testid="open-items-no-account">
			{i18nMsg(
				'open-items-no-account',
				"Aucun compte de ce plan comptable ne se lettre.",
			)}
		</p>
	{/if}

	{#if message}
		{#if message.kind === 'error'}
			<p class="rounded bg-red-50 p-3 text-sm text-red-800" role="alert" data-testid="open-items-message">
				{message.text}
			</p>
		{:else if message.kind === 'lettered'}
			<p class="rounded bg-green-50 p-3 text-sm text-green-900" role="status" data-testid="open-items-message">
				{i18nMsg('open-items-lettered', 'Lettrage posé, code :')}
				<LetteringCodeLink code={message.code} {accountId} {asOf} />
			</p>
		{:else}
			<p class="rounded bg-green-50 p-3 text-sm text-green-900" role="status" data-testid="open-items-message">
				{i18nMsg('open-items-group-dissolved', 'Le groupe { $code } est délettré.', {
					code: message.code,
				})}
			</p>
		{/if}
	{/if}

	{#if group}
		<LetteringGroupPanel
			{group}
			{canWrite}
			{busy}
			onDissolve={(c) => void dissolve(c)}
			onClose={closeGroup}
		/>
		{#if group.status === 'ready' && group.data.accountId !== accountId}
			<button
				type="button"
				class="rounded border px-3 py-1 text-sm"
				onclick={showGroupAccount}
				data-testid="open-items-show-group-account"
				>{i18nMsg('open-items-group-show-list', 'Afficher les postes ouverts de ce compte')}</button
			>
		{/if}
	{/if}

	{#if accountId === null}
		<p class="text-sm text-gray-600" data-testid="open-items-choose-account">
			{i18nMsg('open-items-choose-account', 'Choisissez un compte pour voir ses postes ouverts.')}
		</p>
	{:else if list.status === 'loading' || list.status === 'idle'}
		<p class="text-sm text-gray-500" role="status">{i18nMsg('common-loading', 'Chargement…')}</p>
	{:else if list.status === 'refused'}
		<p class="rounded bg-amber-50 p-3 text-sm text-amber-900" role="alert" data-testid="open-items-refused">
			{list.message}
		</p>
	{:else if list.status === 'notFound'}
		<p class="rounded bg-amber-50 p-3 text-sm text-amber-900" role="alert" data-testid="open-items-not-found">
			{i18nMsg('open-items-account-not-found', 'Compte introuvable.')}
		</p>
	{:else if list.status === 'error'}
		<p class="rounded bg-red-50 p-3 text-sm text-red-800" role="alert" data-testid="open-items-load-error">
			{i18nMsg('open-items-load-error', "Les postes ouverts n'ont pas pu être chargés.")}
		</p>
	{:else}
		{#if canWrite}
			<div class="flex flex-wrap items-center gap-3 rounded border p-2 text-sm" data-testid="open-items-selection">
				<span data-testid="open-items-selection-count">
					{i18nMsg(
						'open-items-selection-count',
						'Lignes sélectionnées : { $count } (dont { $offPage } hors de cette page)',
						{ count: selected.length, offPage },
					)}
				</span>
				<span data-testid="open-items-selection-sum">
					{i18nMsg('open-items-selection-sum', 'Somme de la sélection : { $amount }', {
						amount: formatAmount(selectionSum(selected)),
					})}
				</span>
				<button
					type="button"
					class="rounded bg-primary px-3 py-1 text-primary-foreground disabled:opacity-50"
					disabled={blocker !== null || busy}
					aria-describedby={blocker ? 'open-items-letter-blocker' : undefined}
					onclick={letterSelection}
					data-testid="open-items-letter">{i18nMsg('open-items-letter', 'Lettrer')}</button
				>
				{#if blocker}
					<span id="open-items-letter-blocker" class="text-gray-600" data-testid="open-items-letter-blocker">
						{letterBlockerLabel(blocker)}
					</span>
				{/if}
				{#if selected.length > 0}
					<button
						type="button"
						class="rounded border px-2 py-1"
						onclick={() => selection.clear()}
						data-testid="open-items-selection-clear"
						>{i18nMsg('open-items-selection-clear', 'Vider la sélection')}</button
					>
				{/if}
			</div>
		{/if}
		<OpenItemsTable
			data={list.data}
			{today}
			{canWrite}
			isSelected={(id) => selection.has(id)}
			onToggle={toggle}
			{onPage}
		/>
	{/if}

	{#if showProposals && proposals}
		<ProposalsPanel {proposals} {canWrite} {busy} onAccept={acceptProposal} />
	{/if}
</div>
