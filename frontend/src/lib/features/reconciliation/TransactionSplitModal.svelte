<!--
  Story 8-5a-bis FR48 — Modal éclatement de transaction agrégée.

  Permet à un Comptable d'éclater une `bank_transaction` `pending`
  agrégée en N+1 lignes de `journal_entry` (1 ligne banque agrégée +
  N lignes contrepartie). Le compte ledger banque est résolu
  **serveur-side** via `bank_account.journal_account_id` (foundation
  8-5a-zero) — donc PAS de field `bankLedgerAccountId` dans le body API.

  Tableau éditable (min 2 max 50 lignes) avec indicator balance live
  vert/rouge. Submit désactivé tant que la balance n'est pas exact
  Decimal (cf. AC #93/#95).

  Gestion erreur 412 BANK_ACCOUNT_NOT_CONFIGURED : message + lien vers
  `/bank-accounts` pour configurer le journal_account_id manquant.

  Story 34-3 (#967, AC-3, C-e342-2) : chaque ligne propose la même liste que
  l'écriture manuelle — tout compte actif et imputable, de toute classe, sauf
  le compte comptable du compte bancaire (`bankLedgerAccountId`, désormais
  reçu du parent ; le filtre des classes 5, 6 et 7 l'écartait jusque-là par
  accident). Règle partagée : `counterpartyAccounts`. Un compte au rôle
  débiteurs ou créanciers appelle un avis sous sa ligne (AC-4).
-->
<script lang="ts">
	import { Button } from '$lib/components/ui/button';
	import { Input } from '$lib/components/ui/input';
	import * as Dialog from '$lib/components/ui/dialog';
	import { i18nMsg } from '$lib/shared/utils/i18n.svelte';
	// v0172-correctifs (#880) : date de la transaction en forme suisse.
	import { formatSwissDate } from '$lib/features/reports/reports.api';
	import AccountAutocomplete from '$lib/features/journal-entries/AccountAutocomplete.svelte';
	import { isApiError } from '$lib/shared/utils/api-client';
	import { errorMessageOf } from '$lib/shared/utils/error-message';
	import type { AccountResponse } from '$lib/features/accounts/accounts.types';
	import { listProjects } from '$lib/features/projects/projects.api';
	import type { ProjectResponse } from '$lib/features/projects/projects.types';
	import type { ReconciliationProposal, SplitProposalLine } from './reconciliation.types';
	import { splitTransaction } from './reconciliation.api';
	import {
		claimRoleOfId,
		counterpartyAccounts,
		keepOfferedCounterparty,
	} from './reconciliation-counterparties';
	import CounterpartyClaimNotice from './CounterpartyClaimNotice.svelte';
	import Big from 'big.js';
	import { readJournalAmount } from '$lib/features/journal-entries/balance';

	type Props = {
		open: boolean;
		onOpenChange: (v: boolean) => void;
		bankAccountId: number;
		proposal: ReconciliationProposal | null;
		/** Pré-chargé par le parent (cohérent pattern ManualMatchModal). */
		accounts: AccountResponse[];
		/**
		 * Story 34-3 (AC-3) — compte comptable du compte bancaire, écarté des contreparties de
		 * chaque ligne, comme dans `ManualMatchModal`. `null` (défaut) : aucun filtrage — la
		 * garde serveur de `POST /reconciliation/split` refuse en 400.
		 */
		bankLedgerAccountId?: number | null;
		onSuccess: () => void;
	};

	let {
		open,
		onOpenChange,
		bankAccountId,
		proposal,
		accounts,
		bankLedgerAccountId = null,
		onSuccess,
	}: Props = $props();

	const MIN_SPLITS = 2;
	const MAX_SPLITS = 50;
	const MAX_DESCRIPTION_LEN = 200;

	type SplitLine = {
		counterpartyAccountId: number | null;
		/** La saisie telle que tapée (champ texte, Story 36-2, D5). */
		amount: string;
		description: string;
		projectId: number | null;
	};

	function blankLine(): SplitLine {
		return { counterpartyAccountId: null, amount: '', description: '', projectId: null };
	}

	let splits = $state<SplitLine[]>([blankLine(), blankLine()]);
	let valueDate = $state<string>('');
	let submitting = $state(false);
	let errorMsg = $state<string>('');
	let bankNotConfigured = $state(false);

	// Story 19-5 — projets analytiques pour le tag par ligne de ventilation.
	let projects = $state<ProjectResponse[]>([]);
	$effect(() => {
		listProjects()
			.then((p) => (projects = p))
			.catch(() => (projects = []));
	});

	// Story 34-3 (AC-3) : même règle que l'écriture manuelle, écrite une fois.
	const filteredAccounts = $derived(counterpartyAccounts(accounts, bankLedgerAccountId));

	// Story 35-7 (#1000, D1) : une ligne dont le compte n'est plus proposé — celui de la banque,
	// choisi avant que le parent l'ait identifié — perd ce compte, et elle seule. Le message
	// « Ligne N : compte requis » le dit. N'écrit que les lignes dont la valeur change.
	$effect(() => {
		for (let i = 0; i < splits.length; i++) {
			const current = splits[i].counterpartyAccountId;
			const kept = keepOfferedCounterparty(current, accounts, bankLedgerAccountId);
			if (kept !== current) splits[i].counterpartyAccountId = kept;
		}
	});

	// Reset à l'ouverture + pré-remplissage valueDate.
	$effect(() => {
		if (open && proposal) {
			splits = [blankLine(), blankLine()];
			errorMsg = '';
			bankNotConfigured = false;
			valueDate = proposal.transaction.valueDate ?? proposal.transaction.bookingDate;
		}
	});

	/** Montant total absolu de la transaction (string Decimal). */
	const txAbsAmount = $derived.by(() => {
		if (!proposal) return '0';
		const raw = proposal.transaction.amount;
		return raw.startsWith('-') ? raw.slice(1) : raw;
	});

	/** La lecture d'un montant de ligne (Story 36-2, D5). */
	type LectureMontant =
		| { kind: 'ok'; value: Big }
		| { kind: 'empty' | 'nonpositive' | 'unreadable' | 'ambiguous' | 'cents' };

	/**
	 * Story 36-2 (#1150, D5, C-36-2-9, C-36-2-13) — lit une saisie de montant par la grammaire
	 * du journal (`readJournalAmount`, 36-3 : apostrophe des milliers, point ou virgule décimale,
	 * « 1.234 » ambigu, aucun signe), puis applique la **garde du centime** : le serveur refuse
	 * plus de deux décimales, la grammaire en admet quatre. Une lecture claire n'est acceptée que
	 * si elle égale son arrondi au centime (« 60.0000 » oui, « 0.005 » non) ; le mode d'arrondi
	 * de `big.js` est indifférent, `eq` ne testant que l'existence d'un chiffre au-delà du centime.
	 */
	function lireMontant(raw: string): LectureMontant {
		const r = readJournalAmount(raw);
		if (r.kind !== 'clear') return { kind: r.kind };
		if (!r.value.eq(r.value.round(2))) return { kind: 'cents' };
		if (r.value.lte(0)) return { kind: 'nonpositive' };
		return { kind: 'ok', value: r.value };
	}

	const lectures = $derived(splits.map((s) => lireMontant(s.amount)));

	/** Montant attendu, en `Big` (`null` si le serveur rendait une valeur illisible). */
	const expectedBig = $derived.by(() => {
		try {
			return new Big(txAbsAmount);
		} catch {
			return null;
		}
	});

	/**
	 * Somme exacte des montants (`Big`, jamais `Number` : 0.10 + 0.20 égalent 0.30). `null` si
	 * une ligne n'a pas de montant accepté — `clientError` dit alors laquelle.
	 */
	const sumAmounts = $derived.by(() => {
		let sum = new Big(0);
		for (const l of lectures) {
			if (l.kind !== 'ok') return null;
			sum = sum.plus(l.value);
		}
		return sum;
	});

	const balanceExact = $derived(
		sumAmounts !== null && expectedBig !== null && sumAmounts.eq(expectedBig),
	);

	/**
	 * L'attendu tel qu'affiché : deux décimales, comme l'écart (C-36-2-19) — le serveur rend le
	 * montant normalisé (« -388.2 »), et « attendu 388.2, écart -358.20 » se lisait mal.
	 */
	const expectedLabel = $derived(expectedBig !== null ? expectedBig.toFixed(2) : txAbsAmount);

	/**
	 * L'écart signé à deux décimales ; `null` quand il n'est pas calculable (une ligne sans
	 * montant lu, ou un montant de transaction illisible) — l'indicateur le dit alors en toutes
	 * lettres au lieu d'un « — » qui se confondait avec la ponctuation (revue P1, B3).
	 */
	/**
	 * Story 36-2 (#1151, C-36-2-18, revue P1 E1/E2) — les largeurs des colonnes, à partir de
	 * 640 px seulement (`sm:`) : en dessous, le tableau garde son partage automatique. Deux jeux,
	 * selon la présence de la colonne *Projet*, pour que compte et description reçoivent la
	 * place ; la colonne du bouton « - » est au plus étroit (`w-px`). Classes littérales : Tailwind
	 * ne voit que celles écrites en toutes lettres.
	 */
	const largeurs = $derived(
		projects.length > 0
			? { compte: 'sm:w-[30%]', montant: 'sm:w-[17%]', description: 'sm:w-[33%]' }
			: { compte: 'sm:w-[34%]', montant: 'sm:w-[20%]', description: 'sm:w-[42%]' },
	);

	const balanceDiff = $derived.by(() => {
		if (sumAmounts === null || expectedBig === null) return null;
		const diff = sumAmounts.minus(expectedBig);
		return (diff.gte(0) ? '+' : '') + diff.toFixed(2);
	});

	const clientError = $derived.by(() => {
		if (!proposal)
			return i18nMsg(
				'reconciliation-split-error-no-proposal',
				'Aucune transaction sélectionnée',
			);
		if (splits.length < MIN_SPLITS) {
			return i18nMsg(
				'reconciliation-split-error-min-lines',
				'Au moins { $min } lignes requises',
				{ min: MIN_SPLITS },
			);
		}
		if (splits.length > MAX_SPLITS) {
			return i18nMsg(
				'reconciliation-split-error-max-lines',
				'Maximum { $max } lignes',
				{ max: MAX_SPLITS },
			);
		}
		for (let i = 0; i < splits.length; i++) {
			const s = splits[i];
			if (s.counterpartyAccountId === null) {
				return i18nMsg(
					'reconciliation-split-error-account-required',
					'Ligne { $line } : compte requis',
					{ line: i + 1 },
				);
			}
			// Story 36-2 (#1150, D5) : une ligne à la fois, la raison nommée.
			const lecture = lectures[i];
			if (lecture.kind === 'empty' || lecture.kind === 'nonpositive') {
				return i18nMsg(
					'reconciliation-split-error-amount-positive',
					'Ligne { $line } : montant > 0 requis',
					{ line: i + 1 },
				);
			}
			if (lecture.kind === 'unreadable') {
				return i18nMsg(
					'reconciliation-split-error-amount-unreadable',
					"Ligne { $line } : montant illisible — chiffres, apostrophe des milliers (1'234), point ou virgule décimale, deux décimales au plus, sans signe ni espace.",
					{ line: i + 1 },
				);
			}
			if (lecture.kind === 'ambiguous') {
				return i18nMsg(
					'reconciliation-split-error-amount-ambiguous',
					"Ligne { $line } : montant ambigu — un point ou une virgule suivis de trois chiffres peuvent séparer les milliers ; écrivez 1'234 pour mille deux cent trente-quatre francs.",
					{ line: i + 1 },
				);
			}
			if (lecture.kind === 'cents') {
				return i18nMsg(
					'reconciliation-split-error-amount-cents',
					'Ligne { $line } : deux décimales au plus — le montant se compte en centimes.',
					{ line: i + 1 },
				);
			}
			if (s.description.length > MAX_DESCRIPTION_LEN) {
				return i18nMsg(
					'reconciliation-split-error-description-too-long',
					'Ligne { $line } : description trop longue (max { $max })',
					{ line: i + 1, max: MAX_DESCRIPTION_LEN },
				);
			}
		}
		if (!balanceExact) {
			// Toutes les lignes sont lues : seul un montant de transaction illisible rend l'écart
			// incalculable ; le message générique, sans arguments, le dit alors.
			if (balanceDiff === null) {
				return i18nMsg(
					'reconciliation-split-error-imbalance',
					"L'éclatement n'équilibre pas le montant de la transaction.",
				);
			}
			// Story 36-2 (#1150, D3) : l'attendu et l'écart viennent du catalogue, une clé propre
			// au dialogue, et le message dit quoi faire — l'indicateur au-dessus constate le
			// déséquilibre, le message ne le répète pas (revue P1, B3).
			// `reconciliation-split-error-imbalance`, message de la `400
			// RECONCILIATION_SPLIT_IMBALANCE` et de l'échec partiel, reste inchangée.
			return i18nMsg(
				'reconciliation-split-error-imbalance-detail',
				'Ajustez les montants : leur somme doit égaler { $expected } (écart actuel { $diff }).',
				{ expected: expectedLabel, diff: balanceDiff },
			);
		}
		return '';
	});

	function addLine() {
		if (splits.length < MAX_SPLITS) {
			splits = [...splits, blankLine()];
		}
	}

	function removeLine(idx: number) {
		if (splits.length > MIN_SPLITS) {
			splits = splits.filter((_, i) => i !== idx);
		}
	}

	async function handleConfirm() {
		if (clientError || !proposal) return;
		submitting = true;
		errorMsg = '';
		bankNotConfigured = false;
		try {
			// Story 36-2 (#1150, D2) : le montant part en chaîne décimale au centime, écrite avec
			// un point (`value.toFixed(2)`) — jamais en nombre JSON, que `rust_decimal`
			// (`serde-str`) refuse, ni en saisie brute. `clientError` vide garantit des lectures `ok`.
			const splitInputs: SplitProposalLine[] = splits.map((s, i) => ({
				counterpartyAccountId: s.counterpartyAccountId as number,
				amount: (lectures[i] as { kind: 'ok'; value: Big }).value.toFixed(2),
				description: s.description,
				// Story 19-5 — projet par ligne (omis si null).
				...(s.projectId !== null ? { projectId: s.projectId } : {}),
			}));
			await splitTransaction(
				bankAccountId,
				proposal.bankTransactionId,
				splitInputs,
				valueDate || undefined,
			);
			onSuccess();
			onOpenChange(false);
		} catch (e) {
			if (isApiError(e) && e.code === 'BANK_ACCOUNT_NOT_CONFIGURED') {
				bankNotConfigured = true;
				errorMsg = e.message;
			} else {
				errorMsg = errorMessageOf(e);
			}
		} finally {
			submitting = false;
		}
	}
</script>

<Dialog.Root {open} {onOpenChange}>
	<!-- Story 36-2 (#1151, D6, C-36-2-18) : quatre à cinq colonnes ; la largeur par défaut
	     (`sm:max-w-sm`) les écrasait. `sm:max-w-3xl` laissait encore une description de trente
	     caractères tronquée (R0 (g)) : `sm:max-w-4xl`, et des largeurs de colonnes qui donnent la
	     place au compte et à la description. La largeur seule, sans `max-h` ni défilement : un
	     ancêtre défilant rognerait la liste du champ de compte, rendue en `absolute` (#1181). -->
	<Dialog.Content class="sm:max-w-4xl" data-testid="transaction-split-modal">
		<Dialog.Header>
			<Dialog.Title>
				{i18nMsg('reconciliation-split-modal-title', 'Éclater la transaction')}
			</Dialog.Title>
		</Dialog.Header>

		{#if proposal}
			<div class="text-sm text-text-muted">
				<span data-testid="split-tx-summary">
					{formatSwissDate(proposal.transaction.bookingDate)} — {proposal.transaction.amount}
					{proposal.transaction.currency}
				</span>
			</div>

			<div class="mt-2">
				<table class="w-full text-sm" data-testid="split-lines-table">
					<thead>
						<tr class="text-left text-xs text-text-muted">
							<th class={`${largeurs.compte} py-1`}>
								{i18nMsg('reconciliation-split-th-account', 'Compte')}
							</th>
							<th class={`${largeurs.montant} py-1`}>
								{i18nMsg('reconciliation-split-th-amount', 'Montant')}
							</th>
							<th class={`${largeurs.description} py-1`}>
								{i18nMsg('reconciliation-split-th-description', 'Description')}
							</th>
							{#if projects.length > 0}
								<th class="sm:w-[15%] py-1">
									{i18nMsg('reconciliation-split-th-project', 'Projet')}
								</th>
							{/if}
							<!-- Revue P1 (E1, B2) : la colonne du bouton « - » au plus étroit ; sans colonne
							     *Projet*, elle prenait le reste (≈ 20 %), vide. -->
							<th class="w-px py-1"></th>
						</tr>
					</thead>
					<tbody>
						{#each splits as line, i (i)}
							<tr data-testid={`split-row-${i}`}>
								<td class="py-1 pr-1">
									<AccountAutocomplete
										accounts={filteredAccounts}
										value={line.counterpartyAccountId}
										onSelect={(id) => (splits[i].counterpartyAccountId = id)}
										disabled={submitting}
									/>
								</td>
								<td class="py-1 pr-1">
									<Input
										data-testid={`split-amount-${i}`}
										type="text"
										inputmode="decimal"
										autocomplete="off"
										bind:value={splits[i].amount}
										disabled={submitting}
									/>
								</td>
								<td class="py-1 pr-1">
									<Input
										data-testid={`split-description-${i}`}
										bind:value={splits[i].description}
										maxlength={MAX_DESCRIPTION_LEN}
										disabled={submitting}
									/>
								</td>
								{#if projects.length > 0}
									<td class="py-1 pr-1">
										<select
											class="w-full rounded border border-border px-2 py-1 text-sm"
											data-testid={`split-line-project-${i}`}
											bind:value={splits[i].projectId}
											disabled={submitting}
										>
											<option value={null}>
												{i18nMsg('reconciliation-split-project-none', '— Aucun')}
											</option>
											{#each projects.filter((p) => p.parentId === null) as root (root.id)}
												<option value={root.id}>{root.code} — {root.name}</option>
												{#each projects.filter((c) => c.parentId === root.id) as child (child.id)}
													<option value={child.id}>&nbsp;&nbsp;↳ {child.code} — {child.name}</option>
												{/each}
											{/each}
										</select>
									</td>
								{/if}
								<td class="py-1">
									<Button
										variant="ghost"
										size="sm"
										onclick={() => removeLine(i)}
										disabled={submitting || splits.length <= MIN_SPLITS}
										data-testid={`split-remove-${i}`}
									>
										{i18nMsg('reconciliation-split-remove-line', '-')}
									</Button>
								</td>
							</tr>
							<!-- Story 34-3 (AC-4) — l'avis débiteurs / créanciers, sous la ligne concernée. -->
							{#if claimRoleOfId(filteredAccounts, line.counterpartyAccountId) !== null}
								<tr data-testid={`split-row-notice-${i}`}>
									<td colspan={projects.length > 0 ? 5 : 4} class="pb-1">
										<CounterpartyClaimNotice
											accounts={filteredAccounts}
											accountId={line.counterpartyAccountId}
											line={i}
										/>
									</td>
								</tr>
							{/if}
						{/each}
					</tbody>
				</table>
				<Button
					variant="outline"
					size="sm"
					onclick={addLine}
					disabled={submitting || splits.length >= MAX_SPLITS}
					data-testid="split-add-line"
				>
					{i18nMsg('reconciliation-split-add-line', '+ Ajouter une ligne')}
				</Button>
			</div>

			<div class="mt-2">
				<label class="mb-1 block text-xs text-text-muted" for="split-value-date">
					{i18nMsg('reconciliation-split-value-date-label', 'Date de valeur')}
				</label>
				<Input
					id="split-value-date"
					data-testid="split-value-date-input"
					type="date"
					bind:value={valueDate}
					disabled={submitting}
				/>
			</div>

			<div
				class={`mt-2 rounded-md px-3 py-2 text-sm ${balanceExact ? 'border border-green-500 bg-green-500/10 text-green-700' : 'border border-destructive bg-destructive/10 text-destructive'}`}
				data-testid="split-balance-indicator"
				aria-live="polite"
			>
				<!-- Story 36-2 (#1150, D3) : deux clés, deux états ; le catalogue disait « Balance »
				     pour les deux, et le montant n'était que dans des replis jamais affichés. -->
				{#if balanceExact}
					{i18nMsg('reconciliation-split-balance-indicator', 'Balance équilibrée : { $expected }', {
						expected: expectedLabel,
					})}
				{:else if balanceDiff === null}
					{i18nMsg(
						'reconciliation-split-balance-incomplete',
						'Balance non équilibrée — attendu { $expected }, écart non calculable',
						{ expected: expectedLabel },
					)}
				{:else}
					{i18nMsg(
						'reconciliation-split-balance-gap',
						'Balance non équilibrée — attendu { $expected }, écart { $diff }',
						{ expected: expectedLabel, diff: balanceDiff },
					)}
				{/if}
			</div>

			{#if bankNotConfigured}
				<div
					class="rounded-md border border-destructive bg-destructive/10 px-3 py-2 text-sm text-destructive"
					data-testid="split-bank-not-configured"
				>
					{i18nMsg(
						'reconciliation-split-bank-account-not-configured',
						"Le compte bancaire n'est pas configuré. Configurer le compte comptable lié dans /bank-accounts.",
					)}
					<a class="underline" href="/bank-accounts">/bank-accounts</a>
				</div>
			{:else if clientError}
				<!-- Story 36-2 (#1150, D4) : le message qui dit pourquoi *Éclater* est grisé ; sans
				     rôle `status`, l'indicateur voisin (`aria-live`) disant déjà le déséquilibre. -->
				<div
					class="rounded-md border border-destructive bg-destructive/10 px-3 py-2 text-sm text-destructive"
					data-testid="split-client-error"
				>
					{clientError}
				</div>
			{:else if errorMsg}
				<div
					class="rounded-md border border-destructive bg-destructive/10 px-3 py-2 text-sm text-destructive"
					data-testid="split-error"
				>
					{errorMsg}
				</div>
			{/if}
		{/if}

		<Dialog.Footer>
			<Button
				variant="outline"
				onclick={() => onOpenChange(false)}
				disabled={submitting}
				data-testid="split-cancel"
			>
				{i18nMsg('common-cancel', 'Annuler')}
			</Button>
			<Button
				onclick={handleConfirm}
				disabled={submitting || !!clientError}
				data-testid="split-submit"
			>
				{i18nMsg('reconciliation-split-button-label', 'Éclater')}
			</Button>
		</Dialog.Footer>
	</Dialog.Content>
</Dialog.Root>
