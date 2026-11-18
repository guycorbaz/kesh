<script lang="ts">
	import { onMount } from 'svelte';
	import { Button } from '$lib/components/ui/button';
	import { Input } from '$lib/components/ui/input';
	import { i18nMsg } from '$lib/shared/utils/i18n.svelte';
	import { isApiError } from '$lib/shared/utils/api-client';
	import { notifySuccess } from '$lib/shared/utils/notify';
	import { fetchAccounts } from '$lib/features/accounts/accounts.api';
	// fx-filtre-comptes : exempté — les soldes d'ouverture se saisissent aussi sur les
	// comptes en devise (C-37-5, 37-3b AC-1).
	import { accountRoleKey, type AccountResponse } from '$lib/features/accounts/accounts.types';
	import {
		completeOpeningBalances,
		generateOpeningBalances,
		getOpeningBalancesStatus
	} from '$lib/features/opening-balances/opening-balances.api';
	import type {
		CompletableAccount,
		OpeningBalancesStatus
	} from '$lib/features/opening-balances/opening-balances.types';
	import {
		complementCounterpart,
		computeOpeningTotals,
		openingAmountReader,
		retainedEarningsWarning
	} from '$lib/features/opening-balances/opening-balances-totals';
	import {
		computeBalance,
		formatSwissAmount,
		journalAmountPayload,
		lineAmountProblem
	} from '$lib/features/journal-entries/balance';
	import { amountEntryMessage } from '$lib/features/journal-entries/amount-entry-message';
	import { formatSwissDate } from '$lib/features/reports/reports.api';
	import type Big from 'big.js';
	// Story 37-3b (AC-1) — soldes d'ouverture en devise.
	import { accountCurrency } from '$lib/features/accounts/accounts.types';
	import { listFiscalYears } from '$lib/features/fiscal-years/fiscal-years.api';
	import { authState } from '$lib/app/stores/auth.svelte';
	import { canEditRates } from '$lib/features/exchange-rates/exchange-rates-access';
	import { fetchRates, lookupRate } from '$lib/features/exchange-rates/exchange-rates.api';
	import { todayLocal } from '$lib/shared/utils/local-date';
	import { newForeign, type LineForeign } from '$lib/features/journal-entries/fx-lines';
	import { createRateLookupCache } from '$lib/features/journal-entries/rate-lookup';
	import {
		fxRefusalMessage,
		isFxRefusalCode,
		refusalLine
	} from '$lib/features/journal-entries/fx-refusals';
	import {
		applyOpeningLookup,
		isFilledForeignRow,
		isIncompleteForeignRow,
		openingProposal,
		openingProvenanceLabel,
		commitRowChf,
		rowChfValue,
		rowForeignPayload,
		rowProblem,
		setRowAmount,
		setRowChf,
		setRowRate,
		type OpeningProposal
	} from '$lib/features/opening-balances/opening-fx';

	// ------------------------------------------------------------------
	// État de chargement (P3-BH3-2) : le statut pilote grille-vs-verrou,
	// donc son échec n'est PAS tolérable — état d'erreur explicite + Réessayer,
	// jamais de grille par défaut.
	// ------------------------------------------------------------------
	let loading = $state(true);
	let statusError = $state(false);
	let status = $state<OpeningBalancesStatus | null>(null);
	let accounts = $state<AccountResponse[]>([]);

	// Grille : une ligne par compte de bilan, montants en string décimale.
	// Story 37-3b (AC-1) : `foreign` pour un compte en devise (contrat du
	// `LineDraft` du formulaire : montant et sens dans `foreign`, `debit`/`credit`
	// = CHF dérivé seul).
	interface GridRow {
		account: AccountResponse;
		debit: string;
		credit: string;
		foreign?: LineForeign;
	}
	let rows = $state<GridRow[]>([]);

	let submitting = $state(false);
	let submitError = $state<string | null>(null);

	// Mode « compléter » (Story 25-7, AC 6) : une ligne par compte complétable
	// que rend le STATUS — la liste n'est jamais recalculée côté client.
	interface ComplementRow {
		account: CompletableAccount;
		debit: string;
		credit: string;
		foreign?: LineForeign;
	}

	// --- Story 37-3b (AC-1) : cours proposés --------------------------------
	/** Genre d'ouverture du premier exercice (`GET /fiscal-years`) ; `null` inconnu. */
	let firstYear = $state<{ startDate: string; openingRateKind: 'day' | 'month' } | null>(null);
	let fiscalYearsError = $state(false);
	const cache = createRateLookupCache({
		lookup: lookupRate,
		fetch: fetchRates,
		canFetch: () => canEditRates(authState.currentUser?.role),
		today: () => todayLocal()
	});
	const gridProposal = $derived<OpeningProposal | null>(openingProposal(firstYear));
	const complementProposal = $derived<OpeningProposal | null>(
		openingProposal(firstYear, {
			dateKind: status?.complementDateKind ?? null,
			date: status?.complementDate ?? null
		})
	);
	/** Refus du serveur par ligne de grille, par identifiant du compte de la ligne. */
	let gridRefusals = $state<Record<number, string>>({});
	let complementRefusals = $state<Record<number, string>>({});

	/** Propose (mémoïsé) le cours de la société aux lignes en devise d'une grille. */
	function proposeRates(grid: 'grid' | 'complement') {
		const proposal = grid === 'grid' ? gridProposal : complementProposal;
		if (proposal === null) return;
		const source = grid === 'grid' ? rows : complementRows;
		const currencies = new Set(source.flatMap((r) => (r.foreign ? [r.foreign.currency] : [])));
		for (const currency of currencies) {
			void cache.get(currency, proposal.date, proposal.kind).then((res) => {
				const apply = <T extends GridRow | ComplementRow>(r: T): T =>
					r.foreign && r.foreign.currency === currency
						? applyOpeningLookup({ ...r }, proposal, res.outcome)
						: r;
				if (grid === 'grid') rows = rows.map(apply);
				else complementRows = complementRows.map(apply);
			});
		}
	}

	function foreignOf(currency: string | null | undefined): LineForeign | undefined {
		return currency ? newForeign(currency) : undefined;
	}

	/** Le message d'un problème de ligne en devise. */
	function fxProblemMessage(problem: string): string {
		switch (problem) {
			case 'FX_LINE_ROUNDS_TO_ZERO':
				return i18nMsg('opening-balances-fx-rounds-to-zero', 'Le montant en CHF calculé s’arrondit à zéro.');
			case 'FX_AMOUNT_MISMATCH':
				return i18nMsg(
					'opening-balances-fx-chf-no-rate',
					'Aucun cours à dix décimales ne redonne ce montant CHF : corrigez le CHF ou saisissez le cours.'
				);
			case 'FX_VALUE_OUT_OF_RANGE':
			case 'amount-unreadable':
			case 'rate-unreadable':
				return i18nMsg(
					'opening-balances-fx-out-of-range',
					'Montant en devise, cours ou montant CHF invalide : montant positif à quatre décimales au plus, cours positif à dix décimales et 1 000 000 au plus.'
				);
			default:
				return i18nMsg(
					'opening-balances-fx-incomplete',
					'Saisissez le cours ou le montant CHF de ce solde en devise.'
				);
		}
	}

	/**
	 * Le corps d'une ligne de grille (`foreign` pour un compte en devise). Montants CHF par
	 * `payloadOf` (Story 36-18a, D5 : la charge de la lecture du journal, `'0'` si vide) —
	 * pour une ligne en devise, le CHF dérivé (`toFixed(2)`), toujours clair.
	 */
	function linePayload(r: GridRow | ComplementRow) {
		const foreign = rowForeignPayload(r);
		return {
			accountId: r.account.id,
			debit: payloadOf(r.debit),
			credit: payloadOf(r.credit),
			...(foreign ? { foreign } : {})
		};
	}

	/**
	 * Classe un refus du serveur sur la ligne fautive ; rend le message global
	 * sinon. `sent` : l'**identifiant du compte** de chaque ligne envoyée — non
	 * sa position, que le rechargement du statut après un refus décale (revue
	 * de code P1, 37-3b-CR-P1-14).
	 */
	function placeRefusal(
		err: { code: string; message: string; details?: Record<string, unknown> },
		sent: number[],
		grid: 'grid' | 'complement'
	): string {
		const message = isFxRefusalCode(err.code)
			? fxRefusalMessage(err.code, err.details, err.message)
			: err.message;
		const line = refusalLine(err.details);
		const accountId = line === null ? undefined : sent[line - 1];
		if (accountId !== undefined) {
			if (grid === 'grid') gridRefusals = { [accountId]: message };
			else complementRefusals = { [accountId]: message };
		}
		return message;
	}
	let complementRows = $state<ComplementRow[]>([]);
	let completing = $state(false);
	let completeError = $state<string | null>(null);

	// Jeton de génération (Pass 3 review, ECH3-1) : rend la course
	// last-writer-wins impossible PAR CONSTRUCTION — un load() périmé (lancé
	// avant, résolu après) ne peut plus écraser l'état posé par un load() plus
	// récent (patron leçon 21-6b : fermer la fenêtre, pas ajouter un garde).
	let loadGen = 0;

	async function load() {
		const gen = ++loadGen;
		loading = true;
		statusError = false;
		const [statusResult, accountsResult, yearsResult] = await Promise.allSettled([
			getOpeningBalancesStatus(),
			fetchAccounts(false),
			// Story 37-3b (AC-1) : le genre d'ouverture du premier exercice. Un échec
			// ne propose aucun cours (saisie manuelle), il n'arrête pas la page.
			Promise.resolve().then(() => listFiscalYears())
		]);
		// Réponse périmée : un load() plus récent a pris la main — aucune
		// écriture d'état.
		if (gen !== loadGen) return;

		// Tolérance de panne asymétrique : le statut est obligatoire (il
		// décide quoi afficher), les comptes seulement si la grille s'ouvre.
		if (statusResult.status === 'fulfilled') {
			status = statusResult.value;
		} else {
			status = null;
			statusError = true;
		}

		if (accountsResult.status === 'fulfilled') {
			accounts = accountsResult.value;
		} else {
			accounts = [];
			// Sans comptes, la grille serait vide et inutilisable : on traite
			// l'échec comme un échec de chargement global (même bouton Réessayer).
			if (status?.canEnter) {
				statusError = true;
			}
		}

		// Grille = comptes de bilan actifs ET postables (D4). Les capitaux
		// propres sont des comptes de type Liability dans les 3 plans.
		rows = accounts
			.filter(
				(a) =>
					a.active &&
					a.postable &&
					(a.accountType === 'Asset' || a.accountType === 'Liability')
			)
			.map((account) => {
				const row: GridRow = { account, debit: '', credit: '' };
				const foreign = foreignOf(accountCurrency(account));
				if (foreign) row.foreign = foreign;
				return row;
			});
		// Au rechargement (après un refus métier), la saisie des comptes ENCORE
		// proposés est conservée : seul le compte fautif disparaît (revue de code
		// P3, F-1 — reconstruire à vide faisait tout ressaisir). Story 37-3b : la
		// partie en devise (montant, cours, provenance) l'est aussi.
		const previous = new Map(complementRows.map((r) => [r.account.id, r]));
		// Un rechargement du status qui ÉCHOUE ne touche pas à la saisie : elle
		// survit à « Réessayer » (revue de code P4, R4-4).
		if (statusResult.status === 'fulfilled') {
			complementRows = (status?.completableAccounts ?? []).map((account) => {
				const before = previous.get(account.id);
				const row: ComplementRow = {
					account,
					debit: before?.debit ?? '',
					credit: before?.credit ?? ''
				};
				const foreign =
					before?.foreign && before.foreign.currency === account.currency
						? before.foreign
						: foreignOf(account.currency);
				if (foreign) row.foreign = foreign;
				return row;
			});
		}

		// Story 37-3b (AC-1) : genre d'ouverture du premier exercice (le plus ancien).
		if (yearsResult.status === 'fulfilled' && Array.isArray(yearsResult.value)) {
			const first = [...yearsResult.value].sort((a, b) =>
				a.startDate.localeCompare(b.startDate)
			)[0];
			firstYear = first
				? { startDate: first.startDate, openingRateKind: first.openingRateKind ?? 'day' }
				: null;
			fiscalYearsError = false;
		} else {
			firstYear = null;
			fiscalYearsError = true;
		}

		loading = false;
		proposeRates('grid');
		proposeRates('complement');
	}

	onMount(() => {
		void load();
	});

	// Bandeau de total en direct (D3) — big.js via balance.ts, jamais parseFloat.
	// Story 36-18a (#1050, D5) : chaque solde se lit comme le montant d'une écriture —
	// « 1.250 » (ambigu) n'est plus lu 1,25 franc, il rend la saisie invalide.
	// Revue P1 (B4, C-36-18-13) : sur la saisie rognée (`openingAmountReader`).
	const balance = $derived(computeBalance(rows, openingAmountReader));

	/** Story 36-18a (D5) — un montant que la lecture du journal ne rend pas (ambigu ou illisible). */
	function unread(raw: string): boolean {
		return openingAmountReader(raw) === null;
	}

	/**
	 * Story 36-18a (D3, B4) — le refus à afficher sous une ligne, sur la saisie rognée.
	 * Nommée `amountProblem` à l'intégration de la v0.18.0 dans l'Epic 37 : `rowProblem`
	 * est le problème de la partie en devise d'une ligne (`opening-fx.ts`, 37-3b).
	 */
	function amountProblem(r: { debit: string; credit: string }): 'ambiguous' | 'unreadable' | null {
		return lineAmountProblem(r.debit.trim(), r.credit.trim());
	}

	/** Story 36-18a (D5, B4) — la charge d'un solde : celle de la saisie rognée, `'0'` si vide. */
	function payloadOf(raw: string): string {
		return journalAmountPayload(raw.trim()) ?? '0';
	}

	/**
	 * Story 36-18a (D5) — une ligne saisie : non vide, et soit à un montant non lu (gardée
	 * par prudence, le bouton étant désactivé en amont), soit à un montant > 0.
	 */
	function isEnteredLine(r: { debit: string; credit: string }): boolean {
		if (r.debit.trim() === '' && r.credit.trim() === '') return false;
		const debit = openingAmountReader(r.debit);
		const credit = openingAmountReader(r.credit);
		if (debit === null || credit === null) return true;
		return debit.gt(0) || credit.gt(0);
	}

	// Lignes non vides envoyées au POST (les autres sont ignorées). Une ligne
	// dont les deux côtés valent 0 — vide OU « 0 »/« 0.00 » tapé explicitement —
	// est traitée comme vide et N'EST PAS envoyée (Pass 2 review, ECH2-2) :
	// le serveur la rejetterait en `EntryLineDebitCreditExclusive` (XOR strict).
	// Même cas que la classification « partial » de `JournalEntryForm`, mais
	// remède différent (Pass 3 AA-LOW-3) : le formulaire d'écriture BLOQUE le
	// submit, ici la ligne nulle est simplement retirée du payload — plus doux,
	// sans risque (elle n'apporte rien à l'écriture). Les montants invalides
	// restent inclus par prudence — `balance.hasInvalidAmount` fait retomber
	// `isBalanced` (balance.ts:65) et désactive le bouton en amont.
	// Story 37-3b (AC-1) : une ligne en devise saisie n'est jamais retirée en
	// silence, même sans CHF calculable (elle désactive alors l'envoi). Intégration
	// de la v0.18.0 : les lignes CHF se lisent par la lecture du journal (36-18a).
	const nonEmptyRows = $derived(rows.filter((r) => isFilledForeignRow(r) || isEnteredLine(r)));
	const gridIncomplete = $derived(rows.some(isIncompleteForeignRow));

	const canGenerate = $derived(!submitting && balance.isBalanced && !gridIncomplete);

	// Totaux actif / passif et montant à porter (AC 2), avertissement du report
	// à-nouveau (AC 1) — il signale, il ne bloque pas : `canGenerate` l'ignore.
	const totals = $derived(computeOpeningTotals(rows));
	const warning = $derived(retainedEarningsWarning(accounts, rows, balance.isBalanced));

	// Complément : lignes saisies, contrepartie en direct (arbitrage 3).
	const complementLines = $derived(
		complementRows.filter((r) => isFilledForeignRow(r) || isEnteredLine(r))
	);
	const complementInvalid = $derived(
		complementRows.some((r) => unread(r.debit) || unread(r.credit) || isIncompleteForeignRow(r))
	);
	const counterpart = $derived(complementCounterpart(complementLines));
	const canSubmitComplement = $derived(
		!completing && complementLines.length > 0 && !complementInvalid
	);

	/**
	 * Débit/Crédit mutuellement exclusifs par ligne : saisir l'un vide l'autre. Story 36-18a
	 * (revue P2, R2) : sur la saisie rognée — un blanc seul, lu comme une cellule vide
	 * (C-36-18-13), n'efface pas l'autre côté. Servent les deux grilles.
	 */
	function onDebitInput(row: { debit: string; credit: string }) {
		if (row.debit.trim() !== '') row.credit = '';
	}
	function onCreditInput(row: { debit: string; credit: string }) {
		if (row.credit.trim() !== '') row.debit = '';
	}

	function roleLabel(account: AccountResponse): string {
		if (!account.role) return '';
		return i18nMsg(accountRoleKey(account.role), account.role);
	}

	async function handleGenerate() {
		if (!canGenerate) return;
		submitting = true;
		submitError = null;
		gridRefusals = {};
		const sent = nonEmptyRows.map((r) => r.account.id);
		try {
			// Story 36-18a (D5) : la charge de la lecture du journal (« 1'250 » → `1250`,
			// vide → `0`), par `linePayload` ; jamais `null` ici, `canGenerate` exigeant `isBalanced`.
			await generateOpeningBalances({ lines: nonEmptyRows.map(linePayload) });
			notifySuccess(i18nMsg('opening-balances-success', 'Écriture d’ouverture générée.'));
			// Comportement déterministe post-génération (P1-M2-BH) : recharger
			// le statut → l'écran repasse en état verrouillé ALREADY_HAS_ENTRIES
			// in-place (liens bilan + journal), pas de redirection.
			await load();
		} catch (err) {
			// Le serveur localise déjà tous les messages : afficher tel quel
			// (AC-E — aucun err.code n'est reformulé côté client).
			// Story 37-3b (AC-1) : un refus propre aux devises est traduit et posé sur
			// la ligne fautive (`details.line`, rang du corps envoyé).
			submitError = isApiError(err)
				? placeRefusal(err, sent, 'grid')
				: i18nMsg('opening-balances-status-error', 'Impossible de charger l’état des soldes de départ.');
			// Course perdue (409 : un concurrent a rendu la company non-vierge) :
			// recharger le statut pour verrouiller l'écran in-place, comme le
			// chemin succès — sinon la grille reste active et chaque nouvel
			// essai re-échoue à l'identique (Pass 3 review, BH3-LOW).
			if (isApiError(err) && err.code === 'ILLEGAL_STATE_TRANSITION') {
				await load();
			}
		} finally {
			submitting = false;
		}
	}

	async function handleComplete() {
		if (!canSubmitComplement) return;
		// La date est dans la confirmation : elle peut avoir changé depuis la
		// saisie (verrou de période posé, rechargement après un refus — revue de
		// code P4, R4-2), et c'est le dernier écran avant l'écriture.
		const ok = window.confirm(
			i18nMsg(
				'opening-balances-complete-confirm',
				'Enregistrer cette écriture de complément, datée du { $date } ? Elle reste modifiable depuis sa fiche tant que l’exercice est ouvert — sauf si l’une de ses lignes est lettrée : délettrez-la d’abord.',
				{ date: formatSwissDate(status?.complementDate ?? '') }
			)
		);
		if (!ok) return;
		completing = true;
		completeError = null;
		complementRefusals = {};
		const sent = complementLines.map((r) => r.account.id);
		try {
			// Story 36-18a (D5) : idem ; `canSubmitComplement` exige `!complementInvalid`.
			await completeOpeningBalances({ lines: complementLines.map(linePayload) });
			notifySuccess(i18nMsg('opening-balances-complete-success', 'Complément enregistré.'));
			// Le compte complété disparaît de la liste : c'est le status qui le dit.
			await load();
		} catch (err) {
			// Le serveur localise ses refus (codes OPENING_COMPLEMENT_*) : tel quel.
			completeError = isApiError(err)
				? placeRefusal(err, sent, 'complement')
				: i18nMsg('opening-balances-complete-error', 'Le complément n’a pas pu être enregistré. Réessayez.');
			// Un refus métier (compte mouvementé entre-temps, date, report…) rend
			// la liste périmée : recharger le status, sinon chaque nouvel essai
			// échoue à l'identique (revue de code P1, E-F1). Le message, posé
			// avant, survit au rechargement — il est affiché hors de la grille.
			if (
				isApiError(err) &&
				err.code.startsWith('OPENING_COMPLEMENT_') &&
				!SHAPE_REFUSALS.has(err.code)
			) {
				await load();
			}
		} finally {
			completing = false;
		}
	}

	/**
	 * Refus de FORME du handler : la liste n'est pas périmée, la saisie se
	 * corrige — on ne recharge pas, ce qui la viderait (revue de code P2, R-L2).
	 */
	const SHAPE_REFUSALS = new Set([
		'OPENING_COMPLEMENT_NO_LINES',
		'OPENING_COMPLEMENT_TOO_MANY_LINES',
		'OPENING_COMPLEMENT_INVALID_AMOUNT',
		'OPENING_COMPLEMENT_DUPLICATE_ACCOUNT'
	]);

	/** Message du mode « compléter » indisponible, par `completeReason`. */
	function unavailableMessage(reason: string): string {
		switch (reason) {
			case 'NO_ENTRIES':
				return i18nMsg(
					'opening-balances-complete-unavailable-no-entries',
					'La société n’a encore aucune écriture : utilisez la génération du bilan d’ouverture.'
				);
			case 'NO_OPEN_FISCAL_YEAR':
				return i18nMsg(
					'opening-balances-complete-unavailable-no-open-fiscal-year',
					'Aucun exercice ouvert ne couvre la date du jour : le complément ne peut pas être daté.'
				);
			case 'DATE_LOCKED':
				return i18nMsg(
					'opening-balances-complete-unavailable-date-locked',
					'La date du jour tombe dans la période verrouillée : le complément ne peut pas être daté.'
				);
			case 'NO_RETAINED_EARNINGS':
				return i18nMsg(
					'opening-balances-complete-unavailable-no-retained-earnings',
					'Aucun compte en service ne porte le rôle « Bénéfice/perte reporté » : attribuez-le dans le plan comptable pour compléter un compte oublié.'
				);
			case 'RETAINED_EARNINGS_NOT_POSTABLE':
				return i18nMsg(
					'opening-balances-complete-unavailable-retained-earnings-not-postable',
					'Le compte de bénéfice reporté n’est pas imputable : rendez-le imputable pour compléter un compte oublié.'
				);
			case 'NO_COMPLETABLE_ACCOUNT':
				return i18nMsg(
					'opening-balances-complete-unavailable-no-completable-account',
					'Tous les comptes de bilan proposables — en service, imputables, hors compte de report — ont déjà des mouvements : il ne reste aucun compte à compléter. Un montant faux se corrige dans le journal.'
				);
			default:
				return i18nMsg('opening-balances-status-error', 'Impossible de charger l’état des soldes de départ.');
		}
	}

	const formatNumber = formatSwissAmount;
	/** Deux décimales, ou quatre si le montant en porte davantage (saisie jusqu'à 4). */
	function formatExact(amount: Big): string {
		return formatSwissAmount(amount, amount.round(2).eq(amount) ? 2 : 4);
	}
</script>

<svelte:head>
	<title>{i18nMsg('opening-balances-title', 'Soldes de départ')} — Kesh</title>
</svelte:head>

<!-- Story 37-3b (AC-1) : un solde en devise — montant en devise dans la colonne
     du sens, puis cours OU CHF et provenance du cours. -->
{#snippet fxAmountCells(row: GridRow | ComplementRow, number: string, name: string, prefix = '')}
	{#each ['debit', 'credit'] as const as side (side)}
		<td class="py-1.5 {side === 'debit' ? 'pr-2' : ''}">
			<div class="flex items-center gap-1">
				<span class="text-xs text-text-muted">{row.foreign?.currency}</span>
				<Input
					type="text"
					inputmode="decimal"
					placeholder="0.00"
					class="text-right tabular-nums"
					value={row.foreign && row.foreign.side === side ? row.foreign.amount : ''}
					oninput={(e) => setRowAmount(row, side, (e.currentTarget as HTMLInputElement).value)}
					aria-label={side === 'debit'
						? i18nMsg(
								'opening-balances-fx-debit-for',
								'Débit en { $currency } du compte { $number } « { $name } »',
								{ currency: row.foreign?.currency ?? '', number, name }
							)
						: i18nMsg(
								'opening-balances-fx-credit-for',
								'Crédit en { $currency } du compte { $number } « { $name } »',
								{ currency: row.foreign?.currency ?? '', number, name }
							)}
					data-testid="opening-balances-{prefix}fx-{side}-{number}"
				/>
			</div>
		</td>
	{/each}
{/snippet}

{#snippet fxRateRow(
	row: GridRow | ComplementRow,
	number: string,
	proposal: OpeningProposal | null,
	prefix = ''
)}
	{@const f = row.foreign!}
	{@const problem = rowProblem(row)}
	<tr data-testid="opening-balances-{prefix}fx-row-{number}">
		<td colspan="3" class="pb-2">
			<div class="flex flex-wrap items-center gap-2 text-sm">
				<label class="flex items-center gap-1">
					<span class="text-xs text-text-muted">{i18nMsg('opening-balances-fx-rate', 'Cours')}</span>
					<Input
						type="text"
						inputmode="decimal"
						class="w-32 text-right tabular-nums"
						value={f.rate}
						oninput={(e) => setRowRate(row, (e.currentTarget as HTMLInputElement).value)}
						data-testid="opening-balances-{prefix}fx-rate-{number}"
						placeholder="0.0000"
					/>
				</label>
				<span class="text-xs text-text-muted" data-testid="opening-balances-{prefix}fx-provenance-{number}">
					{openingProvenanceLabel(f, proposal)}
				</span>
				<label class="flex items-center gap-1">
					<span class="text-xs text-text-muted">CHF</span>
					<Input
						type="text"
						inputmode="decimal"
						class="w-32 text-right tabular-nums"
						value={rowChfValue(row)}
						oninput={(e) => setRowChf(row, (e.currentTarget as HTMLInputElement).value)}
						onblur={() => commitRowChf(row)}
						data-testid="opening-balances-{prefix}fx-chf-{number}"
						placeholder="0.00"
					/>
				</label>
			</div>
			{#if problem !== null && (f.amount.trim() !== '' || f.chfEntry !== null)}
				<p class="mt-1 text-xs text-destructive" data-testid="opening-balances-{prefix}fx-problem-{number}">
					{fxProblemMessage(problem)}
				</p>
			{/if}
		</td>
	</tr>
{/snippet}

<div class="mx-auto max-w-4xl space-y-6">
	<h1 class="text-2xl font-semibold">
		{i18nMsg('opening-balances-title', 'Soldes de départ')}
	</h1>

	{#if loading}
		<!-- (a) Chargement : jamais de grille tant que le statut n'est pas résolu. -->
		<p class="text-text-muted" data-testid="opening-balances-loading" role="status">…</p>
	{:else if statusError}
		<!-- (b) Échec du fetch statut : message + Réessayer, PAS de grille. -->
		<div
			class="rounded-lg border border-destructive bg-red-50 p-6 dark:bg-red-950/30"
			data-testid="opening-balances-status-error"
		>
			<p class="text-sm text-destructive">
				{i18nMsg('opening-balances-status-error', 'Impossible de charger l’état des soldes de départ.')}
			</p>
			<!-- Anti double-clic (Pass 4 BH4-LOW) : la protection vient du
			     DÉMONTAGE synchrone de cette branche (`loading = true` → Svelte 5
			     flushe → le bouton disparaît avant qu'un 2e clic ne porte), pas
			     d'un attribut disabled — inerte ici car la branche d'erreur est
			     mutuellement exclusive avec l'état loading. `loadGen` reste la
			     défense en profondeur pour tout futur appelant concurrent. -->
			<Button
				variant="outline"
				size="sm"
				class="mt-3"
				onclick={() => void load()}
				data-testid="opening-balances-retry"
			>
				{i18nMsg('opening-balances-retry', 'Réessayer')}
			</Button>
		</div>
	{:else if status && !status.canEnter}
		<!-- État verrouillé : message explicite selon reason (D6). Pas de grille. -->
		<div
			class="rounded-lg border border-border bg-surface p-6"
			data-testid="opening-balances-locked"
			data-reason={status.reason}
		>
			<p class="text-sm text-text">
				{#if status.reason === 'NO_FISCAL_YEAR'}
					{i18nMsg(
						'opening-balances-locked-no-fiscal-year',
						'Aucun exercice comptable : créez d’abord un exercice (Paramètres → Exercices comptables) pour saisir vos soldes de départ.'
					)}
				{:else if status.reason === 'FIRST_YEAR_CLOSED'}
					{i18nMsg(
						'opening-balances-locked-first-year-closed',
						'Le premier exercice « { $name } » est clôturé : avant la saisie des soldes de départ, un administrateur doit rouvrir les exercices clôturés jusqu’à celui-ci, en commençant par le plus récent.',
						{ name: status.fiscalYear?.name ?? '' }
					)}
				{:else if status.reason === 'ALREADY_HAS_ENTRIES'}
					{i18nMsg(
						'opening-balances-locked-already-has-entries',
						'La société contient déjà des écritures : le bilan d’ouverture a été généré et ne se régénère plus. Un compte de bilan oublié se complète ci-dessous ; un montant faux sur un compte déjà saisi se corrige dans le journal, en modifiant l’écriture d’ouverture tant que l’exercice est ouvert (après l’avoir délettrée si l’une de ses lignes est lettrée), ou par une contre-passation ou une écriture de correction.'
					)}
				{:else}
					<!-- reason inconnue (skew de version, évolution future) : pas de
					     message trompeur — état neutre invitant à recharger (Pass 3
					     review, ECH3-LOW : l'ancien fourre-tout affichait le message
					     ALREADY_HAS_ENTRIES pour toute reason inconnue). -->
					{i18nMsg('opening-balances-status-error', 'Impossible de charger l’état des soldes de départ.')}
				{/if}
			</p>
			<div class="mt-4 flex gap-3">
				{#if status.reason === 'ALREADY_HAS_ENTRIES'}
					<Button
						variant="outline"
						size="sm"
						href="/reports"
						data-testid="opening-balances-goto-balance-sheet"
					>
						{i18nMsg('opening-balances-goto-balance-sheet', 'Voir le bilan')}
					</Button>
					<Button
						variant="outline"
						size="sm"
						href="/journal-entries"
						data-testid="opening-balances-goto-journal"
					>
						{i18nMsg('opening-balances-goto-journal', 'Ouvrir le journal')}
					</Button>
				{:else if status.reason === 'NO_FISCAL_YEAR' || status.reason === 'FIRST_YEAR_CLOSED'}
					<Button variant="outline" size="sm" href="/settings/fiscal-years">
						{i18nMsg('nav-fiscal-years', 'Exercices comptables')}
					</Button>
				{/if}
			</div>
		</div>

		{#if status.reason === 'ALREADY_HAS_ENTRIES'}
			<!-- Mode « compléter » (Story 25-7, AC 6) : seulement sous ce bandeau-là —
			     sous NO_FISCAL_YEAR / FIRST_YEAR_CLOSED, il conseillerait une
			     génération impossible. -->
			{#if status.canComplete && status.retainedEarningsAccount}
				<section class="space-y-4" data-testid="opening-balances-complete">
					<h2 class="text-lg font-semibold">
						{i18nMsg('opening-balances-complete-title', 'Compléter un compte oublié')}
					</h2>
					<p class="text-sm text-text-muted">
						{i18nMsg(
							'opening-balances-complete-intro',
							'Saisissez le solde des seuls comptes de bilan oubliés à l’ouverture. Kesh porte la contrepartie sur le compte de report { $number } « { $name } ».',
							{
								number: status.retainedEarningsAccount.number,
								name: status.retainedEarningsAccount.name
							}
						)}
					</p>
					<p class="text-sm text-text-muted" data-testid="opening-balances-complete-date" data-kind={status.complementDateKind}>
						{#if status.complementDateKind === 'TODAY'}
							{i18nMsg(
								'opening-balances-complete-date-today',
								'L’écriture sera datée du { $date }, dans l’exercice « { $name } » : une régularisation, le premier jour de l’ouverture n’acceptant plus d’écriture (exercice clôturé ou période verrouillée).',
								{
									date: formatSwissDate(status.complementDate ?? ''),
									name: status.complementFiscalYear?.name ?? ''
								}
							)}
						{:else}
							{i18nMsg(
								'opening-balances-complete-date-opening',
								'L’écriture sera datée du { $date }, premier jour de l’exercice « { $name } » : le compte oublié faisait partie de l’ouverture.',
								{
									date: formatSwissDate(status.complementDate ?? ''),
									name: status.complementFiscalYear?.name ?? ''
								}
							)}
						{/if}
						{i18nMsg('opening-balances-complete-date-note', 'La date est confirmée à l’enregistrement.')}
					</p>
					<table class="w-full border-collapse text-sm">
						<thead>
							<tr class="border-b border-border text-left text-xs uppercase tracking-wider text-text-muted">
								<th scope="col" class="py-2 pr-2">{i18nMsg('opening-balances-account', 'Compte')}</th>
								<th scope="col" class="w-40 py-2 pr-2 text-right">{i18nMsg('opening-balances-debit', 'Débit')}</th>
								<th scope="col" class="w-40 py-2 text-right">{i18nMsg('opening-balances-credit', 'Crédit')}</th>
							</tr>
						</thead>
						<tbody>
							{#each complementRows as row (row.account.id)}
								{@const problem = amountProblem(row)}
								{@const problemId = `opening-balances-complete-amount-problem-${row.account.number}`}
								<tr class="border-b border-border/50" data-testid="opening-balances-complete-row-{row.account.number}">
									<td class="py-1.5 pr-2">
										<span class="tabular-nums font-medium">{row.account.number}</span>
										<span class="ml-2">{row.account.name}</span>
									</td>
									{#if row.foreign}
										{@render fxAmountCells(row, row.account.number, row.account.name, 'complete-')}
									{:else}
									<td class="py-1.5 pr-2">
										<Input
											type="text"
											inputmode="decimal"
											placeholder="0.00"
											class="text-right tabular-nums"
											bind:value={row.debit}
											oninput={() => onDebitInput(row)}
											aria-invalid={unread(row.debit)}
											aria-describedby={problem ? problemId : undefined}
											aria-label={i18nMsg('opening-balances-debit-for', 'Débit du compte { $number } « { $name } »', { number: row.account.number, name: row.account.name })}
											data-testid="opening-balances-complete-debit-{row.account.number}"
										/>
									</td>
									<td class="py-1.5">
										<Input
											type="text"
											inputmode="decimal"
											placeholder="0.00"
											class="text-right tabular-nums"
											bind:value={row.credit}
											oninput={() => onCreditInput(row)}
											aria-invalid={unread(row.credit)}
											aria-describedby={problem ? problemId : undefined}
											aria-label={i18nMsg('opening-balances-credit-for', 'Crédit du compte { $number } « { $name } »', { number: row.account.number, name: row.account.name })}
											data-testid="opening-balances-complete-credit-{row.account.number}"
										/>
									</td>
									{/if}
								</tr>
								{#if row.foreign}
									{@render fxRateRow(row, row.account.number, complementProposal, 'complete-')}
								{/if}
								{#if complementRefusals[row.account.id] !== undefined}
									<tr>
										<td colspan="3" class="pb-2 text-xs text-destructive" data-testid="opening-balances-complete-row-refusal-{row.account.number}">
											{complementRefusals[row.account.id]}
										</td>
									</tr>
								{/if}
								{#if problem}
									<!-- Story 36-18a (#1050, D3, D5) — le montant ambigu ou illisible, sous sa ligne. -->
									<tr>
										<td
											colspan="3"
											class="pb-2 text-xs text-destructive"
											id={problemId}
											data-testid="opening-balances-amount-{problem}"
										>
											{amountEntryMessage(problem, 4)}
										</td>
									</tr>
								{/if}
							{/each}
						</tbody>
					</table>
					<!-- Story 36-18a (revue P1, E3) : tant qu'une ligne porte un montant ambigu ou
					     illisible, aucune contrepartie n'est annoncée — elle serait calculée sans
					     cette ligne (« 1.250 » seul donnait « Les lignes s'équilibrent »). -->
					<p
						class="text-sm"
						data-testid="opening-balances-complete-counterpart"
						data-side={complementInvalid ? 'invalid' : counterpart.side}
					>
						{#if complementInvalid}
							<!-- Rien : le message sous la ligne fautive dit quoi corriger. -->
						{:else if counterpart.side === 'credit'}
							{i18nMsg(
								'opening-balances-complete-counterpart-credit',
								'Contrepartie : { $amount } au crédit du compte { $number }.',
								{ amount: formatExact(counterpart.amount), number: status.retainedEarningsAccount.number }
							)}
						{:else if counterpart.side === 'debit'}
							{i18nMsg(
								'opening-balances-complete-counterpart-debit',
								'Contrepartie : { $amount } au débit du compte { $number }.',
								{ amount: formatExact(counterpart.amount), number: status.retainedEarningsAccount.number }
							)}
						{:else if complementLines.length > 0}
							{i18nMsg(
								'opening-balances-complete-counterpart-none',
								'Les lignes s’équilibrent entre elles : aucune contrepartie.'
							)}
						{/if}
					</p>
					<div class="flex justify-end">
						<Button
							onclick={handleComplete}
							disabled={!canSubmitComplement}
							data-testid="opening-balances-complete-submit"
						>
							{completing
								? i18nMsg('opening-balances-complete-submitting', 'Enregistrement…')
								: i18nMsg('opening-balances-complete-submit', 'Compléter')}
						</Button>
					</div>
				</section>
			{:else}
				<p
					class="text-sm text-text-muted"
					data-testid="opening-balances-complete-unavailable"
					data-reason={status.completeReason}
				>
					{unavailableMessage(status.completeReason)}
				</p>
			{/if}
			{#if completeError}
				<!-- Hors de la grille : il survit au rechargement qui suit un refus. -->
				<p class="text-sm text-destructive" data-testid="opening-balances-complete-error" role="alert">
					{completeError}
				</p>
			{/if}
		{/if}
	{:else if status && status.canEnter}
		<!-- Grille de saisie (statut READY). -->
		<p class="text-sm text-text-muted" data-testid="opening-balances-intro">
			{i18nMsg(
				'opening-balances-intro',
				'Saisissez les soldes de vos comptes de bilan repris de votre ancienne comptabilité. Une écriture d’ouverture équilibrée sera générée au { $date } (premier jour de l’exercice « { $name } »). Posez votre report à-nouveau accumulé sur votre compte de report pour équilibrer l’écriture.',
				{
					// Format suisse dd.mm.yyyy (Pass 3 review, BH3-LOW) — pas l'ISO brut.
					date: formatSwissDate(status.fiscalYear?.startDate ?? ''),
					name: status.fiscalYear?.name ?? ''
				}
			)}
		</p>

		{#if rows.length === 0}
			<!-- Plan comptable atypique : aucun compte de bilan actif+postable —
			     sans ce message, la grille serait un en-tête nu avec un bouton
			     durablement désactivé, sans explication (Pass 1 review, ECH-LOW). -->
			<p class="text-sm text-text-muted" data-testid="opening-balances-empty-grid">
				{i18nMsg(
					'opening-balances-empty-grid',
					'Aucun compte de bilan actif et postable dans votre plan comptable — créez ou réactivez vos comptes d’actifs et de passifs (Plan comptable) avant de saisir les soldes de départ.'
				)}
			</p>
		{:else}
		<table class="w-full border-collapse text-sm" data-testid="opening-balances-grid">
			<thead>
				<tr class="border-b border-border text-left text-xs uppercase tracking-wider text-text-muted">
					<th scope="col" class="py-2 pr-2">{i18nMsg('opening-balances-account', 'Compte')}</th>
					<th scope="col" class="w-40 py-2 pr-2 text-right">{i18nMsg('opening-balances-debit', 'Débit')}</th>
					<th scope="col" class="w-40 py-2 text-right">{i18nMsg('opening-balances-credit', 'Crédit')}</th>
				</tr>
			</thead>
			<tbody>
				{#each rows as row (row.account.id)}
					{@const problem = amountProblem(row)}
					{@const problemId = `opening-balances-amount-problem-${row.account.number}`}
					<tr
						class="border-b border-border/50"
						data-testid="opening-balances-row-{row.account.number}"
					>
						<td class="py-1.5 pr-2">
							<span class="tabular-nums font-medium">{row.account.number}</span>
							<span class="ml-2">{row.account.name}</span>
							{#if row.account.role}
								<span
									class="ml-2 rounded bg-primary-light/20 px-1.5 py-0.5 text-xs text-primary"
									data-testid="opening-balances-row-{row.account.number}-role-badge"
								>
									{roleLabel(row.account)}
								</span>
							{/if}
						</td>
						{#if row.foreign}
							{@render fxAmountCells(row, row.account.number, row.account.name)}
						{:else}
						<td class="py-1.5 pr-2">
							<Input
								type="text"
								inputmode="decimal"
								placeholder="0.00"
								class="text-right tabular-nums"
								bind:value={row.debit}
								oninput={() => onDebitInput(row)}
								aria-invalid={unread(row.debit)}
								aria-describedby={problem ? problemId : undefined}
								aria-label={i18nMsg('opening-balances-debit-for', 'Débit du compte { $number } « { $name } »', { number: row.account.number, name: row.account.name })}
								data-testid="opening-balances-debit-{row.account.number}"
							/>
						</td>
						<td class="py-1.5">
							<Input
								type="text"
								inputmode="decimal"
								placeholder="0.00"
								class="text-right tabular-nums"
								bind:value={row.credit}
								oninput={() => onCreditInput(row)}
								aria-invalid={unread(row.credit)}
								aria-describedby={problem ? problemId : undefined}
								aria-label={i18nMsg('opening-balances-credit-for', 'Crédit du compte { $number } « { $name } »', { number: row.account.number, name: row.account.name })}
								data-testid="opening-balances-credit-{row.account.number}"
							/>
						</td>
						{/if}
					</tr>
					{#if row.foreign}
						{@render fxRateRow(row, row.account.number, gridProposal)}
					{/if}
					{#if gridRefusals[row.account.id] !== undefined}
						<tr>
							<td colspan="3" class="pb-2 text-xs text-destructive" data-testid="opening-balances-row-refusal-{row.account.number}">
								{gridRefusals[row.account.id]}
							</td>
						</tr>
					{/if}
					{#if problem}
						<!-- Story 36-18a (#1050, D3, D5) — le montant ambigu ou illisible, sous sa ligne. -->
						<tr>
							<td
								colspan="3"
								class="pb-2 text-xs text-destructive"
								id={problemId}
								data-testid="opening-balances-amount-{problem}"
							>
								{amountEntryMessage(problem, 4)}
							</td>
						</tr>
					{/if}
				{/each}
			</tbody>
		</table>

		<!-- Bandeau de total en direct (D3), miroir JournalEntryForm. -->
		<div
			class="flex items-center justify-between rounded-md border p-4 tabular-nums {balance.isBalanced
				? 'border-green-600 bg-green-50 dark:bg-green-950/30'
				: balance.totalDebit.gt(0) || balance.totalCredit.gt(0)
					? 'border-destructive bg-red-50 dark:bg-red-950/30'
					: 'border-border'}"
			data-testid="opening-balances-totals"
		>
			<div class="space-x-4 text-sm">
				<span>
					<strong>{i18nMsg('opening-balances-total-debit', 'Total débits')} :</strong>
					<span data-testid="opening-balances-total-debit">{formatExact(balance.totalDebit)}</span>
				</span>
				<span>
					<strong>{i18nMsg('opening-balances-total-credit', 'Total crédits')} :</strong>
					<span data-testid="opening-balances-total-credit">{formatExact(balance.totalCredit)}</span>
				</span>
				<span>
					<strong>{i18nMsg('opening-balances-diff', 'Différence')} :</strong>
					<span data-testid="opening-balances-diff">{formatExact(balance.diff)}</span>
				</span>
			</div>
			<div class="text-sm font-medium">
				{#if balance.isBalanced}
					<span class="text-green-700 dark:text-green-400">
						✓ {i18nMsg('journal-entry-form-balanced', 'Équilibré')}
					</span>
				{:else if balance.totalDebit.gt(0) || balance.totalCredit.gt(0)}
					<span class="text-destructive">
						✗ {i18nMsg('journal-entry-form-unbalanced', 'Déséquilibré')}
					</span>
				{/if}
			</div>
		</div>

		<!-- Totaux actif / passif et montant à porter (AC 2). Il aide la saisie ; le
		     contrôle, c'est la comparaison avec l'ancien bilan (dite à côté). -->
		<div class="rounded-md border border-border p-4 text-sm tabular-nums" data-testid="opening-balances-bilan-totals">
			<dl class="grid grid-cols-2 gap-x-4 gap-y-1">
				<dt>{i18nMsg('opening-balances-total-assets', 'Actifs')}</dt>
				<dd class="text-right" data-testid="opening-balances-total-assets">{formatExact(totals.assets)}</dd>
				<dt>{i18nMsg('opening-balances-total-liabilities', 'Passifs et capitaux (hors report)')}</dt>
				<dd class="text-right" data-testid="opening-balances-total-liabilities">{formatExact(totals.liabilities)}</dd>
				<dt>{i18nMsg('opening-balances-amount-to-carry', 'Montant à porter au compte de report')}</dt>
				<dd class="text-right" data-testid="opening-balances-amount-to-carry" data-side={totals.amountToCarry.gt(0) ? 'credit' : totals.amountToCarry.lt(0) ? 'debit' : 'none'}>
					{formatExact(totals.amountToCarry.abs())}
					{#if totals.amountToCarry.gt(0)}
						{i18nMsg('opening-balances-side-credit', 'au crédit')}
					{:else if totals.amountToCarry.lt(0)}
						{i18nMsg('opening-balances-side-debit', 'au débit')}
					{/if}
				</dd>
				<dt>{i18nMsg('opening-balances-retained-entered', 'Report saisi')}</dt>
				<dd class="text-right" data-testid="opening-balances-retained-entered">{formatExact(totals.retainedEntered)}</dd>
				<dt>{i18nMsg('opening-balances-remaining-gap', 'Écart restant')}</dt>
				<dd class="text-right" data-testid="opening-balances-remaining-gap">{formatExact(totals.remainingGap)}</dd>
			</dl>
			<p class="mt-2 text-text-muted">
				{i18nMsg(
					'opening-balances-compare-hint',
					'Comparez les totaux Actifs et Passifs à ceux du bilan de votre ancienne comptabilité : une saisie équilibrée peut encore être fausse si un montant a été porté sur le mauvais compte.'
				)}
			</p>
		</div>

		{#if warning}
			<!-- Avertissement du report à-nouveau (AC 1) : il signale, il ne bloque
			     pas — le bouton « Générer » reste actif. -->
			<div
				class="rounded-md border border-amber-500 bg-amber-50 p-4 text-sm dark:bg-amber-950/30"
				data-testid="opening-balances-no-retained-earnings"
				data-kind={warning.kind}
				role="status"
			>
				{#if warning.kind === 'NO_ROLE'}
					{i18nMsg(
						'opening-balances-warning-no-retained-role',
						'Aucun compte en service ne porte le rôle « Bénéfice/perte reporté » : Kesh ne peut pas vérifier où le report à-nouveau est porté. Attribuez ce rôle dans le plan comptable, et comparez les totaux Actifs et Passifs à ceux de l’ancien bilan.'
					)}
				{:else if warning.kind === 'NOT_POSTABLE'}
					{i18nMsg(
						'opening-balances-warning-retained-not-postable',
						'Le compte de report { $number } « { $name } » n’est pas imputable : il n’apparaît pas dans la grille, et le report à-nouveau ne peut pas y être porté. Rendez-le imputable dans le plan comptable.',
						{ number: warning.account.number, name: warning.account.name }
					)}
				{:else}
					{i18nMsg(
						'opening-balances-warning-no-retained-earnings',
						'La saisie est équilibrée sans rien sur le compte de report { $number } « { $name } ». Le report à-nouveau — la différence entre les actifs et les passifs de votre ancien bilan — doit y être porté : soit l’écart a été mis sur un autre compte, soit le report vaut réellement zéro. Comparez les totaux Actifs et Passifs à ceux de l’ancien bilan.',
						{ number: warning.account.number, name: warning.account.name }
					)}
				{/if}
			</div>
		{/if}

		{#if fiscalYearsError && rows.some((r) => r.foreign)}
			<p class="text-sm text-text-muted" data-testid="opening-balances-fx-no-proposal">
				{i18nMsg(
					'opening-balances-fx-no-proposal',
					'La liste des exercices n’a pas pu être lue : aucun cours n’est proposé. Saisissez le cours ou le montant CHF de chaque solde en devise.'
				)}
			</p>
		{/if}
		{#if gridIncomplete}
			<p class="text-sm text-destructive" data-testid="opening-balances-fx-incomplete">
				{i18nMsg(
					'opening-balances-fx-incomplete-grid',
					'Un solde en devise n’a pas de montant CHF calculable : complétez son cours ou son montant CHF avant de générer.'
				)}
			</p>
		{/if}
		{#if submitError}
			<p class="text-sm text-destructive" data-testid="opening-balances-submit-error" role="alert">
				{submitError}
			</p>
		{/if}

		<div class="flex justify-end">
			<Button
				onclick={handleGenerate}
				disabled={!canGenerate}
				data-testid="opening-balances-generate"
			>
				{submitting
					? i18nMsg('opening-balances-generating', 'Génération…')
					: i18nMsg('opening-balances-generate', 'Générer l’écriture d’ouverture')}
			</Button>
		</div>
		{/if}
	{/if}
</div>
