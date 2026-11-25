<script lang="ts">
	import { Button } from '$lib/components/ui/button';
	import { Input } from '$lib/components/ui/input';
	import * as Select from '$lib/components/ui/select';
	import { toast } from 'svelte-sonner';
	import { X, Plus } from '@lucide/svelte';
	import { i18nMsg } from '$lib/features/onboarding/onboarding.svelte';
	import { isApiError } from '$lib/shared/utils/api-client';
	import { isSettlementOutcomeUnknown } from '$lib/shared/utils/unknown-outcome';
	import { formatSwissDate } from '$lib/features/reports/reports.api';
	import SupplierInvoiceNotice from '$lib/features/supplier-invoices/SupplierInvoiceNotice.svelte';
	import { notifyMissingFiscalYearOrFallback } from '$lib/shared/utils/notify';
	import { todayLocal } from '$lib/shared/utils/local-date';
	import type { AccountResponse } from '$lib/features/accounts/accounts.types';
	import type { ProjectResponse } from '$lib/features/projects/projects.types';
	import { createJournalEntry, updateJournalEntry } from './journal-entries.api';
	import type {
		CreateJournalEntryRequest,
		Journal,
		JournalEntryResponse,
		UpdateJournalEntryRequest
	} from './journal-entries.types';
	import {
		computeBalance,
		classifyLine,
		formatSwissAmount,
		journalAmountPayload,
		journalAmountReader,
		lineAmountProblem,
		readJournalAmount
	} from './balance';
	import {
		editRefusalOutcome,
		entryDateBounds,
		fromJournalEntryResponse,
		type LineDraft
	} from './form-helpers';
	import { isAccountUnusable } from '$lib/features/accounts/account-validity';
	import { isDraftLineNonEmpty } from './vat-purchase';
	import AccountAutocomplete from './AccountAutocomplete.svelte';
	import VatPurchaseAssistant from './VatPurchaseAssistant.svelte';
	import AccountingTooltip from '$lib/shared/components/AccountingTooltip.svelte';
	// Story 37-3b (AC-2) — lignes en devise.
	import { untrack } from 'svelte';
	import { SvelteSet } from 'svelte/reactivity';
	import Big from 'big.js';
	import { authState } from '$lib/app/stores/auth.svelte';
	import { canEditRates } from '$lib/features/exchange-rates/exchange-rates-access';
	import { fetchRates, lookupRate } from '$lib/features/exchange-rates/exchange-rates.api';
	import type { CompanyJson } from '$lib/features/settings/settings.types';
	import { readRate } from './fx';
	import {
		accountsForCurrencies,
		adoptCompanyRate,
		allowedCurrencies,
		applyAccountChange,
		applyChfEntry,
		commitChfEntry,
		applyLookup,
		companyRateHint,
		foreignLineProblem,
		formIndexOfServerLine,
		lineCurrencies,
		lineToPayload,
		monthOf,
		planDifferenceLine,
		rateIndicator,
		syncDerivedChf,
		type CompanyRate,
		type LineForeign
	} from './fx-lines';
	import { foreignProblemMessage, fxRefusalMessage, isFxRefusalCode, refusalLine } from './fx-refusals';
	import { createRateLookupCache, type LookupResult, type RateLookupCache } from './rate-lookup';
	import { debounce } from './debounce';
	import {
		canConfirmFrom,
		canSendFrom,
		canWriteEntry,
		entryDuplicateText,
		isEntryFrozen,
		readEntryDuplicateRefusal,
		stateAfterRejection,
		unknownOutcomeHref,
		type EntryDuplicateRefusal,
		type EntrySubmitState
	} from './entry-submit-state';

	interface Props {
		accounts: AccountResponse[];
		accountsLoadError: boolean;
		/** Projets analytiques actifs (Epic 19, Story 19-2). Vide = colonne masquée. */
		projects?: ProjectResponse[];
		/**
		 * Borne du verrou de période (Story 24-4c, #380) — `null` = aucun verrou.
		 * ⚠️ Sert UNIQUEMENT au `min` du champ date : c'est un confort de saisie.
		 * **Le refus qui fait autorité est celui du serveur** (400 `PERIOD_LOCKED`),
		 * et il est testé sans passer par l'écran.
		 */
		booksLockedThrough?: string | null;
		/**
		 * Si fourni → mode **édition** (Story 15-8a, #532), ouvert depuis la fiche
		 * de l'écriture. Sinon mode création.
		 */
		initialEntry?: JournalEntryResponse | null;
		/**
		 * Exercice **de l'écriture** en édition — bornes `min`/`max` du champ date
		 * (confort de saisie ; le serveur tranche). `null` en création.
		 */
		entryFiscalYear?: { startDate: string; endDate: string } | null;
		/** Compte d'impôt préalable pour l'assistant TVA achat (Story 18-1c). Null si non configuré. */
		recoverableAccountId?: number | null;
		onSuccess: () => void;
		onCancel: () => void;
		/**
		 * Édition (Story 15-8a) : appelé, **après** le toast, quand le refus du
		 * serveur dit que l'écriture a changé sous l'utilisateur — exercice clos,
		 * pièce ou contre-passation apparue, version périmée. La fiche se recharge.
		 * ⛔ Remplace la modale de conflit de la 3.3, qui ne revient pas.
		 */
		onStale?: () => void;
		/**
		 * Story 37-3b (AC-2) — société multi-devises : les lignes sur un compte en
		 * devise se saisissent en devise. Faux : formulaire strictement inchangé.
		 */
		multiCurrency?: boolean;
		/**
		 * Story 37-3b (AC-2, R8) — les comptes de différence de change **de la
		 * société** (défaut) ; les comptes propres d'un compte en devise se lisent
		 * sur son `AccountResponse`. `null` : inconnus (le bouton d'écart l'explique).
		 */
		companyFx?: Pick<CompanyJson, 'fxGainAccountId' | 'fxLossAccountId'> | null;
		/** Story 37-3b — cache des cours injectable (tests) ; défaut : appels de l'API. */
		rateLookup?: RateLookupCache;
	}

	let {
		accounts,
		accountsLoadError,
		projects = [],
		booksLockedThrough = null,
		initialEntry = null,
		entryFiscalYear = null,
		recoverableAccountId = null,
		onSuccess,
		onCancel,
		onStale,
		multiCurrency = false,
		companyFx = null,
		rateLookup = undefined
	}: Props = $props();
	// fx-filtre-comptes : exempté — le formulaire d'écriture saisit en devise : son
	// sélecteur offre les comptes en devise si la société est multi-devises
	// (`excludeForeignCurrency={!multiCurrency}`), restreints
	// à une devise par écriture (R7, `allowedCurrencies`).

	const isEdit = $derived(initialEntry !== null);

	const JOURNALS: Journal[] = ['Achats', 'Ventes', 'Banque', 'Caisse', 'OD'];


	// --- État formulaire ---
	// Pré-remplissage depuis initialEntry si mode édition. Les warnings
	// `state_referenced_locally` sont intentionnellement supprimés : on
	// veut uniquement capturer la valeur initiale au montage du composant.
	// Si initialEntry change au cours de la vie du composant, le parent
	// doit démonter/remonter le formulaire (ce qui est le cas : la fiche
	// remplace sa vue par le formulaire, puis le retire au retour).
	// #889 : le helper commun de la date du jour du poste (copie retirée, C-v0174c-13).
	/* svelte-ignore state_referenced_locally */
	let entryDate = $state(initialEntry?.entryDate ?? todayLocal());
	/* svelte-ignore state_referenced_locally */
	let journal = $state<Journal>(initialEntry?.journal ?? 'Achats');
	/* svelte-ignore state_referenced_locally */
	let description = $state(initialEntry?.description ?? '');
	/* svelte-ignore state_referenced_locally */
	let lines = $state<LineDraft[]>(
		initialEntry
			? fromJournalEntryResponse(initialEntry)
			: [
					{ accountId: null, debit: '', credit: '', projectId: null },
					{ accountId: null, debit: '', credit: '', projectId: null }
				]
	);
	/* svelte-ignore state_referenced_locally */
	let version = $state(initialEntry?.version ?? 0);
	// Colonne projet affichée si des projets actifs existent — sans projet
	// défini, le formulaire reste identique à avant (zéro friction). En
	// édition, une ligne peut porter un tag historique alors que tous les
	// projets sont archivés (liste active vide) : on affiche quand même la
	// colonne pour que le tag reste visible et détaguable (review Pass 1 BH-L2).
	const showProjectColumn = $derived(
		projects.length > 0 || lines.some((l) => l.projectId !== null)
	);
	const tableColCount = $derived(showProjectColumn ? 5 : 4);

	// Premier jour SAISISSABLE : le lendemain de la borne. La borne étant
	// INCLUSIVE, `min` doit valoir borne + 1 jour — un `min` posé à la borne
	// elle-même laisserait l'écran proposer une date que le serveur refuse. En
	// édition, l'exercice de l'écriture borne aussi (Story 15-8a).
	const dateBounds = $derived(entryDateBounds(entryFiscalYear, booksLockedThrough));

	/**
	 * Story 15-8a — une ligne pré-remplie sur un compte archivé ou non imputable
	 * n'est pas re-sélectionnable : on l'affiche (le libellé se résout sur la
	 * liste complète que la fiche fournit) avec un avertissement. Le refus qui
	 * fait autorité reste le 400 du serveur.
	 */
	function lineAccountUnusable(accountId: number | null): boolean {
		if (accountId === null) return false;
		// Story 37-3b (AC-4) : un compte en devise n'est pas inutilisable ici.
		return isAccountUnusable(
			accounts.find((a) => a.id === accountId),
			{ excludeForeignCurrency: !multiCurrency }
		);
	}

	// =========================================================================
	// Story 37-3b (AC-2) — lignes en devise
	// =========================================================================

	/** Date relue en édition (provenance relue gardée tant qu'elle ne change pas). */
	/* svelte-ignore state_referenced_locally */
	const initialDate = initialEntry?.entryDate ?? null;
	const dateChanged = $derived(isEdit && entryDate !== initialDate);
	/** Devises de l'écriture relue (R7 : plusieurs = ouverture, complément, état hérité). */
	/* svelte-ignore state_referenced_locally */
	const loadedCurrencies = new Set(
		(initialEntry?.lines ?? []).flatMap((l) => (l.foreign ? [l.foreign.currency] : []))
	);
	/* svelte-ignore state_referenced_locally */
	const cache =
		rateLookup ??
		createRateLookupCache({
			lookup: lookupRate,
			fetch: fetchRates,
			canFetch: () => canEditRates(authState.currentUser?.role),
			today: () => todayLocal()
		});
	/** `lookup` en cours, par `devise|mois` (le ↑ attend la fin de celui du mois). */
	const pendingKeys = new SvelteSet<string>();
	/** `devise|mois` sans cours lisible ni récupérable : saisie manuelle exigée. */
	const manualRequired = new SvelteSet<string>();
	/** Promesses en cours, que l'envoi attend. */
	const inflight = new Set<Promise<void>>();

	/** Une date complète dans les bornes du champ (aucun `lookup` sinon). */
	function lookupDateValid(d: string): boolean {
		if (!/^\d{4}-\d{2}-\d{2}$/.test(d)) return false;
		if (dateBounds.min && d < dateBounds.min) return false;
		if (dateBounds.max && d > dateBounds.max) return false;
		return true;
	}

	/** Applique le résultat d'un `lookup` aux lignes de la devise, si le mois est toujours celui de la date. */
	function track(p: Promise<LookupResult>, currency: string, month: string) {
		const key = `${currency}|${month}`;
		pendingKeys.add(key);
		const done: Promise<void> = p
			.then((r) => {
				// Story 53-1a (relevé C1, défense en profondeur) : à partir de J2b le
				// corps est formé et gardé, une écriture tardive le ferait diverger.
				if (!canWriteEntry(untrack(() => phase))) return;
				if (monthOf(entryDate) !== month) return;
				lines = lines.map((l) =>
					l.foreign && l.foreign.currency === currency
						? syncDerivedChf({ ...l, foreign: lookupForLine(l.foreign, month, r.outcome) })
						: l
				);
				if (r.note === 'manual-required') manualRequired.add(key);
				else manualRequired.delete(key);
			})
			.finally(() => {
				pendingKeys.delete(key);
				inflight.delete(done);
			});
		inflight.add(done);
	}

	/**
	 * Le `lookup` appliqué à une ligne. En édition, une ligne relue ni retouchée,
	 * à la date inchangée, garde son cours (et donc son CHF) : le `lookup` lancé
	 * pour une autre ligne ne le remplace pas par le cours actuel de la société
	 * (revue de code P1, 37-3b-CR-P1-11) ; il n'en retient que le cours `afc`.
	 */
	function lookupForLine(f: LineForeign, month: string, outcome: LookupResult['outcome']): LineForeign {
		const keep = f.loadedRateSource !== null && !f.touched && !dateChanged;
		if (!keep) return applyLookup(f, month, outcome);
		return { ...applyLookup({ ...f, rateProposed: false }, month, outcome), rateProposed: f.rateProposed };
	}

	/** Lance (mémoïsé) le `lookup` du mois de la date pour les devises des lignes (ou une seule). */
	function refreshRates(only?: string) {
		if (!multiCurrency || !lookupDateValid(entryDate)) return;
		const date = entryDate;
		for (const currency of lineCurrencies(lines)) {
			if (only !== undefined && currency !== only) continue;
			track(cache.get(currency, date), currency, monthOf(date));
		}
	}

	// Changement de date : `lookup` (débounce) pour toutes les lignes en devise ;
	// le cours n'est remplacé que pour les lignes proposées. En édition, aucun
	// `lookup` au montage.
	const refreshOnDate = debounce(() => refreshRates(), 300);
	/* svelte-ignore state_referenced_locally */
	let lastDate = entryDate;
	$effect(() => {
		const d = entryDate;
		if (d === untrack(() => lastDate)) return;
		lastDate = d;
		// Story 53-1a (relevé C2) : hors des états de saisie, la date ne programme
		// aucune recherche.
		if (!canWriteEntry(untrack(() => phase))) return;
		refreshOnDate();
	});
	$effect(() => () => refreshOnDate.cancel());

	/** Le compte d'une ligne change (U2) : la devise suit le compte. */
	function selectAccount(index: number, id: number | null) {
		// Reprendre le même compte ne change rien (revue de code P2, 37-3b-CR-P2-4) :
		// la ligne relue garde son cours, sa provenance et son CHF.
		if (id === lines[index].accountId) return;
		const account = id === null ? undefined : accounts.find((a) => a.id === id);
		if (!multiCurrency) {
			lines[index].accountId = id;
			return;
		}
		const next = applyAccountChange({ ...lines[index] }, account);
		lines[index] = next;
		clearLineRefusal(index);
		// Même devise comprise : la ligne est retouchée, et sa provenance (`afc`
		// ou `manual`) se juge sur le cours `afc` du mois, qu'il faut avoir lu
		// (revue de code P1, 37-3b-CR-P1-10).
		if (next.foreign) refreshRates(next.foreign.currency);
	}

	/** Le montant en devise saisi dans la colonne `side`. */
	function setForeignAmount(index: number, side: 'debit' | 'credit', value: string) {
		const f = lines[index].foreign;
		if (!f) return;
		lines[index] = syncDerivedChf({
			...lines[index],
			foreign: { ...f, side, amount: value, chfEntry: null, chfError: null }
		});
		clearLineRefusal(index);
	}

	/** Le cours retapé : `manual` sauf s'il égale le cours `afc` du mois (lu par `lookup`). */
	function setForeignRate(index: number, value: string) {
		const f = lines[index].foreign;
		if (!f) return;
		lines[index] = syncDerivedChf({
			...lines[index],
			foreign: {
				...f,
				rate: value,
				rateProposed: false,
				touched: true,
				forceManual: false,
				chfEntry: null,
				chfError: null
			}
		});
		clearLineRefusal(index);
		// En édition, `afcRate` est inconnu tant qu'aucun `lookup` n'a eu lieu :
		// l'envoi attend celui-ci avant d'appliquer la règle d'égalité.
		refreshRates(f.currency);
	}

	/** Le CHF saisi à la main : le cours s'en déduit (A32). */
	function setForeignChf(index: number, value: string) {
		const f = lines[index].foreign;
		if (!f) return;
		lines[index] = syncDerivedChf({ ...lines[index], foreign: applyChfEntry(f, value, entryDate) });
		clearLineRefusal(index);
		refreshRates(f.currency);
	}

	/** Le champ CHF quitte le focus : la saisie résolue cède la place au CHF dérivé. */
	function commitForeignChf(index: number) {
		const f = lines[index]?.foreign;
		if (!f) return;
		lines[index] = syncDerivedChf({ ...lines[index], foreign: commitChfEntry(f) });
	}

	/** Le ↑ : la ligne reprend le cours de la société (la table des cours n'est pas touchée). */
	function adoptRate(index: number, company: CompanyRate) {
		const f = lines[index].foreign;
		if (!f) return;
		lines[index] = syncDerivedChf({ ...lines[index], foreign: adoptCompanyRate(f, company) });
		clearLineRefusal(index);
	}

	/** La valeur affichée dans le champ CHF d'une ligne en devise. */
	function chfFieldValue(index: number): string {
		const l = lines[index];
		const f = l.foreign;
		if (!f) return '';
		if (f.chfEntry !== null) return f.chfEntry;
		return f.side === 'debit' ? l.debit : l.credit;
	}

	/** Le libellé de l'indicateur du cours d'une ligne. */
	function indicatorLabel(index: number): string {
		const f = lines[index].foreign;
		if (!f) return '';
		const ind = rateIndicator(f);
		if (ind.kind === 'afc') return i18nMsg('journal-entries-fx-indicator-afc', 'AFC');
		if (ind.kind === 'saisi') {
			return ind.source === null
				? i18nMsg('journal-entries-fx-indicator-saisi', 'cours saisi')
				: i18nMsg('journal-entries-fx-indicator-saisi-source', 'cours saisi ({ $source })', {
						source: ind.source
					});
		}
		return i18nMsg('journal-entries-fx-indicator-manual', 'manuel');
	}

	/** La mention du cours de la société sous le champ, et son ↑ (D9, U1). */
	function hintFor(index: number): { company: CompanyRate; label: string } | null {
		const f = lines[index].foreign;
		if (!f) return null;
		const pending = pendingKeys.has(`${f.currency}|${monthOf(entryDate)}`);
		const company = companyRateHint(f, entryDate, pending);
		if (company === null) return null;
		const rate = new Big(company.rate).toFixed();
		const label =
			company.source === 'afc'
				? i18nMsg('journal-entries-fx-hint-afc', 'AFC : { $rate }', { rate })
				: company.source === 'manual'
					? i18nMsg('journal-entries-fx-hint-saisi', 'Cours saisi : { $rate }', { rate })
					: i18nMsg('journal-entries-fx-hint-saisi-source', 'Cours saisi ({ $source }) : { $rate }', {
							rate,
							source: company.source
						});
		return { company, label };
	}

	/** Une devise par écriture (R7) : les comptes offerts au sélecteur de la ligne. */
	function accountsForLine(index: number) {
		if (!multiCurrency) return accounts;
		return accountsForCurrencies(accounts, allowedCurrencies(lines, index, loadedCurrencies));
	}

	// --- Refus du serveur sur une ligne (FX) ------------------------------------
	/** Message d'un refus du serveur, par indice de ligne du formulaire. */
	let lineRefusals = $state<Record<number, string>>({});
	/** `FX_RATE_NOT_AFC` : le cours attendu par ligne (`null` : la société a son cours). */
	let rateNotAfc = $state<Record<number, string | null>>({});
	/** `RATE_UNAVAILABLE` reçu : la devise et le mois à récupérer. */
	let rateUnavailable = $state<{ currency: string; month: string } | null>(null);
	/** Refus FX sans ligne (exercice mono-devise, deux devises…), affiché au pied. */
	let formRefusal = $state<string | null>(null);

	function clearLineRefusal(index: number) {
		if (index in lineRefusals) {
			const { [index]: _, ...rest } = lineRefusals;
			void _;
			lineRefusals = rest;
		}
		if (index in rateNotAfc) {
			const { [index]: _, ...rest } = rateNotAfc;
			void _;
			rateNotAfc = rest;
		}
	}

	/** « Reprendre le cours attendu » : la ligne passe au cours AFC (le CHF change). */
	function takeExpectedRate(index: number) {
		const f = lines[index].foreign;
		const expected = rateNotAfc[index];
		if (!f || !expected) return;
		const month = monthOf(entryDate);
		lines[index] = syncDerivedChf({
			...lines[index],
			foreign: {
				...f,
				rate: new Big(expected).toFixed(),
				afcRate: expected,
				afcMonth: month,
				touched: true,
				forceManual: false,
				rateProposed: false,
				chfEntry: null,
				chfError: null
			}
		});
		clearLineRefusal(index);
	}

	/** « Garder mon cours » : la ligne part `manual` à son cours, CHF inchangé. */
	function keepMyRate(index: number) {
		const f = lines[index].foreign;
		if (!f) return;
		lines[index] = { ...lines[index], foreign: { ...f, forceManual: true } };
		clearLineRefusal(index);
	}

	/** `RATE_UNAVAILABLE` → « Récupérer le mois », puis le renvoi. */
	async function fetchMissingMonth() {
		// Story 53-1a (relevé C3) : rien ne part hors des états de saisie.
		if (!isEdit && !canSendFrom(phase)) return;
		const missing = rateUnavailable;
		if (!missing) return;
		rateUnavailable = null;
		const month = monthOf(missing.month);
		track(cache.refresh(missing.currency, month), missing.currency, month);
		await Promise.all([...inflight]);
		await handleSubmit();
	}

	/** `RATE_UNAVAILABLE` → les lignes de la devise passent `manual`. */
	function keepManualForCurrency() {
		const missing = rateUnavailable;
		if (!missing) return;
		rateUnavailable = null;
		lines = lines.map((l) =>
			l.foreign && l.foreign.currency === missing.currency
				? { ...l, foreign: { ...l.foreign, forceManual: true } }
				: l
		);
	}

	// --- Ligne d'écart (R8) -------------------------------------------------------
	const hasForeignLine = $derived(multiCurrency && lines.some((l) => l.foreign));
	/** Ajoute **une ligne** à l'écriture en cours (aucune écriture créée, rien d'envoyé). */
	function addDifferenceLine() {
		const plan = differencePlan;
		if (plan.kind !== 'line') return;
		lines = [
			...lines,
			{
				accountId: plan.accountId,
				debit: plan.side === 'debit' ? plan.amount : '',
				credit: plan.side === 'credit' ? plan.amount : '',
				projectId: null
			}
		];
	}
	/** Un cours a changé depuis l'insertion de l'assistant TVA (lignes CHF dérivées gardées). */
	const vatRateChanged = $derived.by(() => {
		const derived = lines.filter((l) => l.chfFromForeign);
		if (derived.length === 0) return false;
		return lines.some((l) => {
			if (!l.foreign) return false;
			const r = readRate(l.foreign.rate);
			return derived.some(
				(d) => r === null || r === 'empty' || !r.eq(new Big(d.chfFromForeign!.rate))
			);
		});
	});

	let submitting = $state(false);

	// Story 53-1a (#1355, table J) — l'état de l'envoi, à la **création** seulement :
	// en modification, le comportement reste celui d'avant (le second envoi rend
	// `OPTIMISTIC_LOCK_CONFLICT`) et `phase` demeure `J1`.
	let phase = $state<EntrySubmitState>('J1');
	/** Le refus de doublon lu en J5 : l'écriture existante citée. */
	let duplicate = $state<EntryDuplicateRefusal | null>(null);
	/**
	 * Le **corps gardé** de l'envoi en vol : c'est lui, et nul autre, que renvoie la
	 * confirmation de J5 (M18 : jamais reformé) ; sa date alimente le lien de J6.
	 */
	let kept = $state.raw<{ payload: CreateJournalEntryRequest; sentIndexes: number[] } | null>(null);
	/** Figé : tous les contrôles qui écrivent la saisie (`<fieldset disabled>`). */
	const frozen = $derived(!isEdit && isEntryFrozen(phase));
	/** « Valider » n'est offert ni en J5 ni en J6. */
	const showSubmit = $derived(isEdit || (phase !== 'J5' && phase !== 'J6'));

	// Assistant TVA achat (Story 18-1c) : lignes en attente de confirmation
	// de remplacement (si le brouillon n'est pas vierge).
	let pendingAssistant = $state<{ lines: LineDraft[]; description: string } | null>(null);

	/** Applique les lignes générées par l'assistant : remplace le brouillon. */
	function applyAssistant(result: { lines: LineDraft[]; description: string }) {
		lines = result.lines;
		// Les lignes sont remplacées : les refus par ligne ne visent plus rien.
		lineRefusals = {};
		rateNotAfc = {};
		journal = 'Achats';
		if (description.trim() === '') {
			description = result.description;
		}
	}

	/** Reçu de l'assistant : confirme si le brouillon n'est pas vierge, sinon applique. */
	function handleAssistantApply(result: { lines: LineDraft[]; description: string }) {
		const dirty = lines.some(isDraftLineNonEmpty) || description.trim() !== '';
		if (dirty) {
			pendingAssistant = result;
		} else {
			applyAssistant(result);
		}
	}

	function confirmAssistantReplace() {
		if (pendingAssistant) applyAssistant(pendingAssistant);
		pendingAssistant = null;
	}

	// Story 36-3 (#1051, D3) : les montants se lisent par la lecture du journal —
	// apostrophe des milliers admise, « 1.234 » refusé comme ambigu. ⚠️ Pas de
	// `lines.map(classifyLine)` : `map` passerait l'indice pour lecteur.
	const balance = $derived(computeBalance(lines, journalAmountReader));
	/** Story 37-3b (R8) — la ligne d'écart que le bouton ajouterait. */
	const differencePlan = $derived(
		hasForeignLine && !balance.hasInvalidAmount
			? planDifferenceLine(lines, accounts, companyFx, balance.totalDebit, balance.totalCredit)
			: ({ kind: 'none' } as const)
	);
	const lineStatuses = $derived(lines.map((l) => classifyLine(l, journalAmountReader)));
	/** Story 36-3 (D4) : le message de chaque ligne, ambigu prioritaire. */
	const lineProblems = $derived(lines.map((l) => lineAmountProblem(l.debit, l.credit)));
	/** Story 36-3 (D4) : bordure rouge d'un champ ni vide ni clair. */
	function amountFlagged(raw: string): boolean {
		const kind = readJournalAmount(raw).kind;
		return kind !== 'empty' && kind !== 'clear';
	}

	const nonEmptyLines = $derived.by(() => {
		return lines.map((l, i) => ({ line: l, status: lineStatuses[i], index: i }))
			.filter((x) => x.status !== 'empty');
	});

	const canSubmit = $derived.by(() => {
		if (submitting) return false;
		// Story 53-1a : à la création, l'envoi ne part que des états de saisie.
		if (!isEdit && !canWriteEntry(phase)) return false;
		if (description.trim() === '') return false;
		const validLines = nonEmptyLines.filter((x) => x.status === 'valid');
		const partialLines = nonEmptyLines.filter((x) => x.status === 'partial');
		if (partialLines.length > 0) return false;
		if (validLines.length < 2) return false;
		return balance.isBalanced;
	});

	function addLine() {
		lines = [...lines, { accountId: null, debit: '', credit: '', projectId: null }];
	}

	/**
	 * Décale les refus par ligne quand la ligne `index` est retirée : un refus
	 * reste sur sa ligne, jamais sur la suivante (revue de code P1, 37-3b-CR-P1-8).
	 */
	function shiftAfterRemoval<V>(table: Record<number, V>, index: number): Record<number, V> {
		const out: Record<number, V> = {};
		for (const [k, v] of Object.entries(table)) {
			const i = Number(k);
			if (i < index) out[i] = v;
			else if (i > index) out[i - 1] = v;
		}
		return out;
	}

	function removeLine(index: number) {
		if (lines.length <= 2) return;
		lines = lines.filter((_, i) => i !== index);
		lineRefusals = shiftAfterRemoval(lineRefusals, index);
		rateNotAfc = shiftAfterRemoval(rateNotAfc, index);
	}

	// P9 : formatage string-based via formatSwissAmount (zéro perte de précision).
	const formatNumber = formatSwissAmount;

	async function handleSubmit() {
		if (!canSubmit) return;
		// Story 53-1a : à la création, l'envoi ordinaire ne part que de J1 et J7.
		if (!isEdit && !canSendFrom(phase)) return;
		// Story 37-3b (AC-2) : l'envoi attend les `lookup` en cours (provenance) —
		// y compris celui d'un changement de date encore différé (revue de code
		// P1, 37-3b-CR-P1-5 : sans quoi la ligne partirait au cours de l'ancien mois).
		refreshOnDate.flush();
		if (inflight.size > 0) {
			// J2a : le formulaire est figé, le corps n'est pas encore formé ; les
			// recherches en vol peuvent encore écrire la saisie (voulu, `canWriteEntry`).
			if (!isEdit) phase = 'J2a';
			submitting = true;
			await Promise.all([...inflight]);
			submitting = false;
			if (!canSubmit) {
				if (!isEdit) phase = 'J1';
				return;
			}
		}
		lineRefusals = {};
		rateNotAfc = {};
		formRefusal = null;
		// Story 36-3 (D4) : la charge de chaque montant, sans apostrophe ni virgule.
		// `canSubmit` exclut déjà toute ligne ambiguë ou illisible ; un `null` ici
		// n'envoie rien plutôt que de deviner. Story 37-3b : une ligne en devise
		// porte `foreign` (devise du compte toujours envoyée).
		const sentIndexes = nonEmptyLines.map(({ index }) => index);
		const payloadLines = nonEmptyLines.map(({ line }) =>
			multiCurrency || line.foreign
				? lineToPayload(line, entryDate, dateChanged)
				: (() => {
						const debit = journalAmountPayload(line.debit);
						const credit = journalAmountPayload(line.credit);
						return debit === null || credit === null
							? null
							: { accountId: line.accountId!, debit, credit, projectId: line.projectId };
					})()
		);
		if (payloadLines.some((l) => l === null)) {
			if (!isEdit) phase = 'J1';
			return;
		}
		// J2b : requête en vol, le corps est formé et **gardé**.
		if (!isEdit) {
			phase = 'J2b';
			duplicate = null;
		}
		submitting = true;

		const payload: CreateJournalEntryRequest = {
			entryDate,
			journal,
			description: description.trim(),
			lines: payloadLines.map((l) => l!)
		};
		kept = { payload, sentIndexes };
		await transmit(payload, sentIndexes, false);
	}

	/**
	 * « Ce n'est pas un doublon, enregistrer » (J5 → J2b) : renvoie le **corps gardé**
	 * avec `confirmNotDuplicate: true`, une seule fois — la garde est synchrone.
	 */
	async function confirmDuplicate() {
		if (!canConfirmFrom(phase) || kept === null) return;
		phase = 'J2b';
		submitting = true;
		await transmit(kept.payload, kept.sentIndexes, true);
	}

	/** « Modifier la saisie » (J5 → J1). */
	function modifyEntry() {
		if (phase !== 'J5') return;
		duplicate = null;
		phase = 'J1';
	}

	/** « J'ai vérifié, elle n'existe pas — reprendre la saisie » (J6 → J1) : rien n'est envoyé. */
	function resumeEntry() {
		if (phase !== 'J6') return;
		phase = 'J1';
	}

	/** L'envoi lui-même : succès (J3), refus de doublon (J5), issue inconnue (J6), erreur (J7). */
	async function transmit(
		payload: CreateJournalEntryRequest,
		sentIndexes: number[],
		confirm: boolean
	) {
		try {
			if (isEdit && initialEntry) {
				const updatePayload: UpdateJournalEntryRequest = { ...payload, version };
				await updateJournalEntry(initialEntry.id, updatePayload);
			} else {
				await createJournalEntry(confirm ? { ...payload, confirmNotDuplicate: true } : payload);
			}
			if (!isEdit) phase = 'J3';
			toast.success(i18nMsg('journal-entry-saved', 'Écriture enregistrée'));
			onSuccess();
		} catch (err) {
			// Story 53-1a (J6-m) : à la modification, une issue inconnue ne passe plus
			// pour un échec ; le formulaire reste actif (le second envoi rend
			// `OPTIMISTIC_LOCK_CONFLICT`).
			if (isEdit && isSettlementOutcomeUnknown(err)) {
				toast.error(
					i18nMsg(
						'journal-entries-unknown-outcome-update',
						'La modification a peut-être été enregistrée : rechargez l’écriture avant de la modifier encore.'
					)
				);
				return;
			}
			// Table J, sorties de J2b : le refus de doublon (J5), l'issue inconnue (J6).
			if (!isEdit) {
				const next = stateAfterRejection(err, isSettlementOutcomeUnknown);
				if (next === 'J5') {
					duplicate = readEntryDuplicateRefusal(err);
					phase = 'J5';
					return;
				}
				if (next === 'J6') {
					phase = 'J6';
					return;
				}
				phase = 'J7';
			}
			if (isApiError(err)) {
				const code = err.code ?? '';
				// Story 15-8a (D8) — en édition, un refus qui dit que l'écriture a
				// changé sous l'utilisateur : toast, puis la fiche se recharge. ⛔ Pas
				// de modale, et pas `notifyMissingFiscalYearOrFallback` : son conseil
				// (« vérifiez la date saisie ») serait faux, c'est l'exercice DE
				// L'ÉCRITURE qui a été clôturé.
				if (isEdit && editRefusalOutcome(code) === 'stale') {
					let message = err.message;
					if (code === 'FISCAL_YEAR_CLOSED') {
						message = i18nMsg(
							'journal-entries-modify-blocked-fiscal-year-closed',
							'L’exercice de cette écriture est clôturé : elle est figée. Corrigez-la par une contre-passation.'
						);
					} else if (code === 'OPTIMISTIC_LOCK_CONFLICT') {
						message = i18nMsg(
							'journal-entries-edit-conflict',
							'Cette écriture a été modifiée entre-temps : la fiche a été rechargée.'
						);
					}
					toast.error(message);
					onStale?.();
					return;
				}
				// Story 37-3b (AC-2) — refus propres aux devises : message traduit sur la
				// ligne fautive (rang du corps envoyé, sans les lignes vides), saisie
				// conservée, formulaire ouvert.
				if (isFxRefusalCode(code)) {
					const message = fxRefusalMessage(code, err.details, err.message);
					const index = formIndexOfServerLine(sentIndexes, refusalLine(err.details));
					if (index !== null) {
						lineRefusals = { ...lineRefusals, [index]: message };
						if (code === 'FX_RATE_NOT_AFC') {
							const expected = err.details?.expectedRate;
							rateNotAfc = {
								...rateNotAfc,
								[index]:
									typeof expected === 'string' && expected !== ''
										? expected
										: typeof expected === 'number'
											? String(expected)
											: null
							};
						}
					} else {
						formRefusal = message;
					}
					if (code === 'RATE_UNAVAILABLE') {
						const currency = err.details?.currency;
						const month = err.details?.month;
						if (typeof currency === 'string' && typeof month === 'string') {
							rateUnavailable = { currency, month };
						}
					}
					toast.error(message);
					return;
				}
				// Story 3.7 AC #22 — fallback toast actionnable pour NO_FISCAL_YEAR / FISCAL_YEAR_CLOSED.
				if (notifyMissingFiscalYearOrFallback(err)) {
					return;
				}
				switch (code) {
					case 'ENTRY_UNBALANCED':
					case 'DATE_OUTSIDE_FISCAL_YEAR':
					case 'INACTIVE_OR_INVALID_ACCOUNTS':
					// Story 15-5a — le message du serveur nomme le ou les comptes.
					case 'ACCOUNT_NOT_POSTABLE':
					// Story 15-8a — le message nomme la borne et la date refusée.
					case 'PERIOD_LOCKED':
					case 'VALIDATION_ERROR':
						toast.error(err.message);
						break;
					case 'RESOURCE_CONFLICT':
						// Race sur uq_journal_entries_number (création).
						toast.error(
							i18nMsg(
								'error-conflict',
								'Conflit de numérotation — veuillez réessayer'
							)
						);
						break;
					default:
						toast.error(err.message || 'Erreur lors de la sauvegarde');
				}
			} else {
				toast.error('Erreur lors de la sauvegarde');
			}
		} finally {
			submitting = false;
		}
	}

	function handleKeydown(e: KeyboardEvent) {
		// Ctrl+S → submit
		if ((e.ctrlKey || e.metaKey) && e.key === 's') {
			e.preventDefault();
			// Story 35-7, revue P1 (E1, #1000) : le raccourci soumet sans quitter le
			// champ. Rendre le focus fait passer un brouillon de compte par le `blur`
			// d'`AccountAutocomplete`, qui le jette : le champ montre le compte qui part.
			(document.activeElement as HTMLElement | null)?.blur();
			if (canSubmit) handleSubmit();
		}
	}

	function handleLineCreditKeydown(e: KeyboardEvent, index: number) {
		// Enter dans le dernier crédit → ajouter une ligne.
		if (e.key === 'Enter' && index === lines.length - 1) {
			e.preventDefault();
			addLine();
		}
	}
</script>

<svelte:window onkeydown={handleKeydown} />

<div class="space-y-6 p-6 bg-card rounded-lg border border-border">
	<h2 class="text-xl font-semibold">
		{i18nMsg('journal-entry-form-title', 'Saisie d\'écriture')}
	</h2>

	<!-- Story 53-1a (table J) — UN `<fieldset disabled>` enveloppe tous les contrôles qui
	     écrivent la saisie (date, journal, libellé, lignes, assistant TVA, cours) ; figé en
	     J2a, J2b, J3, J5, J6, sans rien vider. Atténué : un champ inactif doit en avoir l'air.
	     Les avis (J5, J6) et les boutons d'action sont hors du fieldset, donc actifs. -->
	<fieldset
		class="m-0 min-w-0 space-y-6 border-0 p-0 disabled:opacity-60"
		disabled={frozen}
		data-testid="journal-entry-fields"
	>
	<div class="grid grid-cols-1 md:grid-cols-3 gap-4">
		<div>
			<label for="entry-date" class="block text-sm font-medium mb-1">
				{i18nMsg('journal-entry-form-date', 'Date')}
			</label>
			<Input
				id="entry-date"
				type="date"
				bind:value={entryDate}
				min={dateBounds.min}
				max={dateBounds.max}
				required
			/>
		</div>
		<div>
			<label for="entry-journal" class="block text-sm font-medium mb-1">
				<AccountingTooltip term="journal">
					<!-- svelte-ignore a11y_no_noninteractive_tabindex -->
					<span tabindex="0" class="cursor-help underline underline-offset-2 decoration-dotted">
						{i18nMsg('journal-entry-form-journal', 'Journal')}
					</span>
				</AccountingTooltip>
			</label>
			<Select.Root type="single" value={journal} onValueChange={(v) => (journal = v as Journal)}>
				<Select.Trigger id="entry-journal">
					{journal}
				</Select.Trigger>
				<Select.Content>
					{#each JOURNALS as j (j)}
						<Select.Item value={j}>{i18nMsg(`journal-${j.toLowerCase()}`, j)}</Select.Item>
					{/each}
				</Select.Content>
			</Select.Root>
		</div>
		<div>
			<label for="entry-description" class="block text-sm font-medium mb-1">
				{i18nMsg('journal-entry-form-description', 'Libellé')}
			</label>
			<Input id="entry-description" type="text" bind:value={description} required />
		</div>
	</div>

	{#if !isEdit}
		<VatPurchaseAssistant
			{accounts}
			{accountsLoadError}
			{recoverableAccountId}
			onApply={handleAssistantApply}
			{multiCurrency}
			{entryDate}
			lookupRate={(currency, date) => cache.get(currency, date)}
		/>
	{/if}

	<table class="w-full border-collapse">
		<thead>
			<tr class="border-b border-border">
				<th class="text-left py-2 text-sm font-medium">
					{i18nMsg('journal-entry-form-col-account', 'Compte')}
				</th>
				<th class="text-right py-2 text-sm font-medium w-32">
					<AccountingTooltip term="debit">
						<!-- svelte-ignore a11y_no_noninteractive_tabindex -->
						<span tabindex="0" class="cursor-help underline underline-offset-2 decoration-dotted">
							{i18nMsg('journal-entry-form-col-debit', 'Débit')}
						</span>
					</AccountingTooltip>
				</th>
				<th class="text-right py-2 text-sm font-medium w-32">
					<AccountingTooltip term="credit">
						<!-- svelte-ignore a11y_no_noninteractive_tabindex -->
						<span tabindex="0" class="cursor-help underline underline-offset-2 decoration-dotted">
							{i18nMsg('journal-entry-form-col-credit', 'Crédit')}
						</span>
					</AccountingTooltip>
				</th>
				{#if showProjectColumn}
					<th class="text-left py-2 text-sm font-medium w-44">
						{i18nMsg('journal-entry-form-col-project', 'Compte analytique')}
					</th>
				{/if}
				<th class="w-10"></th>
			</tr>
		</thead>
		<tbody>
			{#each lines as line, i (i)}
				{@const status = lineStatuses[i]}
				<tr class="border-b border-border/50">
					<td class="py-2 pr-2">
						<AccountAutocomplete
							accounts={accountsForLine(i)}
							value={line.accountId}
							loadError={accountsLoadError}
							excludeForeignCurrency={!multiCurrency}
							onSelect={(id) => selectAccount(i, id)}
						/>
						{#if lineAccountUnusable(line.accountId)}
							<p class="mt-1 text-xs text-destructive" data-testid="line-account-unusable">
								{i18nMsg(
									'journal-entries-line-account-unusable',
									'Compte archivé ou non imputable — à remplacer'
								)}
							</p>
						{/if}
					</td>
					{#if line.foreign}
						{@const f = line.foreign}
						<!-- Story 37-3b (AC-2) : montant en devise dans la colonne du sens. -->
						{#each ['debit', 'credit'] as const as side (side)}
							<td class="py-2 pr-2">
								<div class="flex items-center gap-1">
									<span class="text-xs text-muted-foreground">{f.currency}</span>
									<Input
										type="text"
										inputmode="decimal"
										data-testid="journal-entry-line-fx-{side}-{i}"
										aria-label={i18nMsg('journal-entries-fx-amount-label', 'Montant ({ $currency })', {
											currency: f.currency
										})}
										value={f.side === side ? f.amount : ''}
										oninput={(e) =>
											setForeignAmount(i, side, (e.currentTarget as HTMLInputElement).value)}
										onkeydown={side === 'credit' ? (e) => handleLineCreditKeydown(e, i) : undefined}
										class="tabular-nums text-right"
										placeholder="0.00"
									/>
								</div>
							</td>
						{/each}
					{:else}
						<td class="py-2 pr-2">
							<Input
								type="text"
								inputmode="decimal"
								bind:value={lines[i].debit}
								data-testid="journal-entry-line-debit-{i}"
								oninput={() => (lines[i].chfFromForeign = null)}
								class={amountFlagged(line.debit) ? 'border-destructive' : 'tabular-nums text-right'}
								placeholder="0.00"
							/>
						</td>
						<td class="py-2 pr-2">
							<Input
								type="text"
								inputmode="decimal"
								bind:value={lines[i].credit}
								data-testid="journal-entry-line-credit-{i}"
								oninput={() => (lines[i].chfFromForeign = null)}
								onkeydown={(e) => handleLineCreditKeydown(e, i)}
								class={amountFlagged(line.credit) ? 'border-destructive' : 'tabular-nums text-right'}
								placeholder="0.00"
							/>
						</td>
					{/if}
					{#if showProjectColumn}
						<td class="py-2 pr-2">
							<!-- Sélecteur arbre 2 niveaux (racines + sous-projets indentés),
							     même pattern que la facture fournisseur (Story 19-3). -->
							<select
								bind:value={lines[i].projectId}
								data-testid="journal-entry-line-project-{i}"
								aria-label={i18nMsg('journal-entry-form-col-project', 'Compte analytique')}
								class="w-full h-9 rounded-md border border-input bg-background px-2 text-sm"
							>
								<option value={null}>{i18nMsg('journal-entry-project-none', '— Aucun')}</option>
								{#if line.projectId !== null && !projects.some((p) => p.id === line.projectId)}
									<!-- Tag historique sur un projet archivé (absent de la liste des
									     actifs) : option ad-hoc pour préserver la valeur au round-trip
									     (le backend exempte les tags pré-existants de la validation). -->
									<option value={line.projectId}>
										{i18nMsg('journal-entry-project-archived', 'Compte analytique archivé')}
									</option>
								{/if}
								{#each projects.filter((p) => p.parentId === null) as root (root.id)}
									<option value={root.id}>{root.code} — {root.name}</option>
									{#each projects.filter((c) => c.parentId === root.id) as child (child.id)}
										<option value={child.id}>&nbsp;&nbsp;↳ {child.code} — {child.name}</option>
									{/each}
								{/each}
							</select>
						</td>
					{/if}
					<td class="py-2">
						{#if lines.length > 2}
							<button
								type="button"
								onclick={() => removeLine(i)}
								class="text-muted-foreground hover:text-destructive p-1"
								aria-label={i18nMsg('journal-entry-form-remove-line', 'Retirer cette ligne')}
							>
								<X class="w-4 h-4" />
							</button>
						{/if}
					</td>
				</tr>
				{#if line.foreign}
					{@const f = line.foreign}
					{@const hint = hintFor(i)}
					{@const problem = foreignLineProblem(f)}
					<!-- Story 37-3b (AC-2) : cours OU CHF, provenance, cours de la société et ↑. -->
					<tr data-testid="journal-entry-line-fx-{i}">
						<td colspan={tableColCount} class="pb-2">
							<div class="flex flex-wrap items-center gap-2 text-sm">
								<label class="flex items-center gap-1">
									<span class="text-xs text-muted-foreground">
										{i18nMsg('journal-entries-fx-rate', 'Cours')}
									</span>
									<Input
										type="text"
										inputmode="decimal"
										class="w-32 tabular-nums text-right"
										data-testid="journal-entry-line-fx-rate-{i}"
										value={f.rate}
										oninput={(e) => setForeignRate(i, (e.currentTarget as HTMLInputElement).value)}
										placeholder="0.0000"
									/>
								</label>
								<span class="text-xs text-muted-foreground" data-testid="journal-entry-line-fx-indicator-{i}">
									{indicatorLabel(i)}
								</span>
								<label class="flex items-center gap-1">
									<span class="text-xs text-muted-foreground">CHF</span>
									<Input
										type="text"
										inputmode="decimal"
										class="w-32 tabular-nums text-right"
										data-testid="journal-entry-line-fx-chf-{i}"
										aria-label={i18nMsg('journal-entries-fx-chf-label', 'Montant CHF')}
										value={chfFieldValue(i)}
										oninput={(e) => setForeignChf(i, (e.currentTarget as HTMLInputElement).value)}
										onblur={() => commitForeignChf(i)}
										placeholder="0.00"
									/>
								</label>
								{#if hint}
									<span class="text-xs" data-testid="journal-entry-line-fx-hint-{i}">{hint.label}</span>
									<button
										type="button"
										class="rounded border border-border px-2 text-xs hover:bg-muted"
										data-testid="journal-entry-line-fx-adopt-{i}"
										aria-label={i18nMsg('journal-entries-fx-adopt', 'Reprendre le cours de la société')}
										title={i18nMsg('journal-entries-fx-adopt', 'Reprendre le cours de la société')}
										onclick={() => adoptRate(i, hint.company)}
									>
										↑
									</button>
								{/if}
							</div>
							{#if problem !== null && (f.amount.trim() !== '' || f.chfError !== null)}
								<p class="mt-1 text-xs text-destructive" data-testid="journal-entry-line-fx-problem-{i}">
									{foreignProblemMessage(problem)}
								</p>
							{/if}
							{#if manualRequired.has(`${f.currency}|${monthOf(entryDate)}`) && f.rate.trim() === ''}
								<p class="mt-1 text-xs text-muted-foreground" data-testid="journal-entry-line-fx-manual-{i}">
									{i18nMsg(
										'journal-entries-fx-manual-required',
										'Aucun cours n’est disponible pour ce mois : saisissez le cours ou le montant CHF.'
									)}
								</p>
							{/if}
						</td>
					</tr>
				{:else if status === 'partial'}
					<tr>
						<td colspan={tableColCount} class="text-xs text-destructive pb-2">
							{i18nMsg('journal-entry-form-incomplete-line', 'Ligne incomplète')}
						</td>
					</tr>
				{/if}
				{#if lineRefusals[i] !== undefined}
					<tr>
						<td colspan={tableColCount} class="pb-2 text-xs text-destructive" data-testid="journal-entry-line-refusal-{i}">
							{lineRefusals[i]}
							{#if i in rateNotAfc}
								<span class="ml-2 inline-flex gap-2">
									{#if rateNotAfc[i] !== null}
										<button
											type="button"
											class="underline"
											data-testid="journal-entry-line-take-expected-{i}"
											onclick={() => takeExpectedRate(i)}
										>
											{i18nMsg('journal-entries-fx-take-expected', 'Reprendre le cours attendu')}
										</button>
									{/if}
									<button
										type="button"
										class="underline"
										data-testid="journal-entry-line-keep-rate-{i}"
										onclick={() => keepMyRate(i)}
									>
										{i18nMsg('journal-entries-fx-keep-rate', 'Garder mon cours')}
									</button>
								</span>
							{/if}
						</td>
					</tr>
				{/if}
				{#if lineProblems[i] === 'ambiguous'}
					<tr>
						<td
							colspan={tableColCount}
							class="text-xs text-destructive pb-2"
							data-testid="journal-entry-line-amount-ambiguous"
						>
							{i18nMsg(
								'journal-entry-form-amount-ambiguous',
								'Montant ambigu : un point ou une virgule suivis de trois chiffres peuvent séparer les milliers. Écrivez 1\'234 pour mille deux cent trente-quatre, ou 1.2340 pour un franc vingt-trois et quatre dixièmes de centime.'
							)}
						</td>
					</tr>
				{:else if lineProblems[i] === 'unreadable'}
					<tr>
						<td
							colspan={tableColCount}
							class="text-xs text-destructive pb-2"
							data-testid="journal-entry-line-amount-unreadable"
						>
							{i18nMsg(
								'journal-entry-form-amount-unreadable',
								'Montant illisible : chiffres, apostrophe des milliers (1\'234), point ou virgule décimale, quatre décimales au plus.'
							)}
						</td>
					</tr>
				{/if}
			{/each}
		</tbody>
	</table>

	<div class="flex flex-wrap items-center gap-2">
		<Button type="button" variant="outline" size="sm" onclick={addLine}>
			<Plus class="w-4 h-4 mr-1" />
			{i18nMsg('journal-entry-form-add-line', '+ Ajouter une ligne')}
		</Button>
		{#if differencePlan.kind === 'line'}
			<Button
				type="button"
				variant="outline"
				size="sm"
				data-testid="journal-entry-add-fx-difference"
				aria-label={i18nMsg('journal-entries-fx-difference-aria', 'Ajouter une ligne de différence de change')}
				title={i18nMsg('journal-entries-fx-difference-aria', 'Ajouter une ligne de différence de change')}
				onclick={addDifferenceLine}
			>
				{i18nMsg('journal-entries-fx-difference', 'Ajouter la différence de change')}
			</Button>
		{/if}
		{#if hasForeignLine}
			<a
				href="/journal-entries/conversion"
				class="text-sm underline text-primary"
				data-testid="journal-entry-conversion-link"
			>
				{i18nMsg('journal-entries-fx-conversion-link', 'Changer de devise : assistant Conversion')}
			</a>
		{/if}
	</div>
	{#if differencePlan.kind === 'line'}
		<p class="text-xs text-muted-foreground" data-testid="journal-entry-fx-difference-note">
			{differencePlan.beyondRounding
				? i18nMsg(
						'journal-entries-fx-difference-not-rounding',
						'Écart de { $amount } CHF : ce n’est pas un arrondi. Vérifiez les montants et les cours avant d’ajouter la ligne.',
						{ amount: differencePlan.amount }
					)
				: i18nMsg(
						'journal-entries-fx-difference-rounding',
						'Écart de { $amount } CHF : un arrondi des conversions en CHF.',
						{ amount: differencePlan.amount }
					)}
			{#if differencePlan.mixedPairs}
				{i18nMsg(
					'journal-entries-fx-difference-mixed',
					'Les comptes en devise de l’écriture n’ont pas les mêmes comptes de différence : la ligne suit ceux du compte { $number }.',
					{ number: differencePlan.holderNumber }
				)}
			{/if}
		</p>
	{:else if differencePlan.kind === 'invalid'}
		<p class="text-xs text-destructive" data-testid="journal-entry-fx-difference-invalid">
			{differencePlan.origin === 'own'
				? i18nMsg(
						'journal-entries-fx-difference-invalid-own',
						'Les comptes de différence de change propres du compte { $number } sont archivés, non imputables ou invalides : corrigez-les dans le plan comptable.',
						{ number: differencePlan.holderNumber }
					)
				: differencePlan.origin === 'company'
					? i18nMsg(
							'journal-entries-fx-difference-invalid-company',
							'Les comptes de différence de change du réglage de la société sont archivés, non imputables ou invalides : corrigez-les dans Paramètres → Comptabilité.'
						)
					: i18nMsg(
							'journal-entries-fx-difference-missing',
							'Aucun compte de différence de change n’est désigné (réglage de la société, ou comptes propres du compte { $number }) : désignez-les avant d’ajouter la différence.',
							{ number: differencePlan.holderNumber }
						)}
		</p>
	{/if}
	{#if vatRateChanged}
		<p class="text-xs text-destructive" data-testid="journal-entry-vat-rate-changed">
			{i18nMsg(
				'journal-entries-fx-vat-rate-changed',
				'Cours changé depuis l’insertion de l’assistant TVA : réinsérez l’assistant.'
			)}
		</p>
	{/if}
	{#if formRefusal}
		<p class="text-sm text-destructive" data-testid="journal-entry-form-refusal">{formRefusal}</p>
	{/if}
	{#if rateUnavailable}
		<div class="flex flex-wrap gap-2 text-sm" data-testid="journal-entry-rate-unavailable">
			<Button type="button" size="sm" variant="outline" onclick={fetchMissingMonth}>
				{i18nMsg('journal-entries-fx-fetch-month', 'Récupérer le mois')}
			</Button>
			<Button type="button" size="sm" variant="outline" onclick={keepManualForCurrency}>
				{i18nMsg('journal-entries-fx-keep-manual', 'Garder les cours comme cours saisis')}
			</Button>
		</div>
	{/if}

	<div
		class="flex items-center justify-between rounded-md border p-4 tabular-nums {balance.isBalanced
			? 'border-green-600 bg-green-50 dark:bg-green-950/30'
			: balance.totalDebit.gt(0) || balance.totalCredit.gt(0)
				? 'border-destructive bg-red-50 dark:bg-red-950/30'
				: 'border-border'}"
	>
		<div class="space-x-4 text-sm">
			<span>
				<strong>{i18nMsg('journal-entry-form-total-debit', 'Total débits')} :</strong>
				{formatNumber(balance.totalDebit)}
			</span>
			<span>
				<strong>{i18nMsg('journal-entry-form-total-credit', 'Total crédits')} :</strong>
				{formatNumber(balance.totalCredit)}
			</span>
			<span>
				<strong>{i18nMsg('journal-entry-form-diff', 'Différence')} :</strong>
				{formatNumber(balance.diff)}
			</span>
		</div>
		<div class="text-sm font-medium">
			{#if balance.isBalanced || balance.totalDebit.gt(0) || balance.totalCredit.gt(0)}
				<AccountingTooltip term="balanced">
					{#if balance.isBalanced}
						<!-- svelte-ignore a11y_no_noninteractive_tabindex -->
						<span tabindex="0" class="text-green-700 dark:text-green-400 cursor-help">
							✓ {i18nMsg('journal-entry-form-balanced', 'Équilibré')}
						</span>
					{:else}
						<!-- svelte-ignore a11y_no_noninteractive_tabindex -->
						<span tabindex="0" class="text-destructive cursor-help">
							✗ {i18nMsg('journal-entry-form-unbalanced', 'Déséquilibré')}
						</span>
					{/if}
				</AccountingTooltip>
			{:else}
				<!-- Formulaire vide (débit=0, crédit=0) : on n'instancie PAS
				     AccountingTooltip pour éviter un bouton bits-ui focusable
				     sans accessible name (WCAG 4.1.2). Placeholder invisible. -->
				<span aria-hidden="true" class="opacity-0">—</span>
			{/if}
		</div>
	</div>

	</fieldset>

	<!-- Story 53-1a — les avis de J5 (doublon probable) et J6 (issue inconnue). -->
	{#if phase === 'J5' && duplicate !== null}
		<SupplierInvoiceNotice
			testid="journal-entry-probable-duplicate"
			text={entryDuplicateText()}
			link={{
				href: `/journal-entries/${duplicate.journalEntryId}`,
				label: i18nMsg(
					'journal-entries-duplicate-existing',
					'Écriture n° { $number } du { $date }',
					{ number: duplicate.entryNumber, date: formatSwissDate(duplicate.entryDate) }
				),
				testid: 'journal-entry-duplicate-link'
			}}
		>
			<button
				type="button"
				class="rounded bg-primary px-3 py-1 text-sm text-primary-foreground"
				data-testid="journal-entry-confirm-not-duplicate"
				onclick={() => confirmDuplicate()}
			>
				{i18nMsg('journal-entries-duplicate-confirm', 'Ce n’est pas un doublon, enregistrer')}
			</button>
			<button
				type="button"
				class="rounded border px-3 py-1 text-sm"
				data-testid="journal-entry-modify-entry"
				onclick={() => modifyEntry()}
			>
				{i18nMsg('journal-entries-duplicate-edit', 'Modifier la saisie')}
			</button>
		</SupplierInvoiceNotice>
	{/if}
	{#if phase === 'J6'}
		<SupplierInvoiceNotice
			testid="journal-entry-unknown-outcome"
			text={i18nMsg(
				'journal-entries-unknown-outcome',
				'L’enregistrement a peut-être abouti. Vérifiez dans le journal si l’écriture existe avant de la saisir de nouveau.'
			)}
			link={{
				href: unknownOutcomeHref(kept?.payload.entryDate ?? entryDate),
				label: i18nMsg('journal-entries-unknown-outcome-link', 'Voir les écritures de cette date'),
				testid: 'journal-entry-unknown-outcome-link'
			}}
		>
			<button
				type="button"
				class="rounded border px-3 py-1 text-sm"
				data-testid="journal-entry-resume-entry"
				onclick={() => resumeEntry()}
			>
				{i18nMsg(
					'journal-entries-unknown-outcome-resume',
					'J’ai vérifié, elle n’existe pas — reprendre la saisie'
				)}
			</button>
		</SupplierInvoiceNotice>
	{/if}

	<div class="flex justify-end gap-2">
		<Button type="button" variant="outline" onclick={onCancel} disabled={submitting}>
			{i18nMsg('journal-entry-form-cancel', 'Annuler')}
		</Button>
		{#if showSubmit}
			<Button type="button" onclick={handleSubmit} disabled={!canSubmit} data-testid="journal-entry-submit">
				{i18nMsg('journal-entry-form-submit', 'Valider')}
			</Button>
		{/if}
	</div>
</div>

<!-- Modale de confirmation de remplacement par l'assistant TVA achat (Story 18-1c) -->
{#if pendingAssistant}
	<div
		class="fixed inset-0 z-50 flex items-center justify-center bg-black/50"
		role="dialog"
		aria-modal="true"
		aria-labelledby="vat-replace-title"
		aria-describedby="vat-replace-desc"
	>
		<div class="bg-card border border-border rounded-lg p-6 max-w-md mx-4 shadow-lg">
			<h2 id="vat-replace-title" class="text-lg font-semibold mb-2">
				{i18nMsg('vat-purchase-replace-title', 'Remplacer le brouillon ?')}
			</h2>
			<p id="vat-replace-desc" class="text-sm text-text-muted mb-4">
				{i18nMsg(
					'vat-purchase-replace-message',
					'Des lignes ou un libellé ont déjà été saisis. Continuer écrasera le brouillon actuel.'
				)}
			</p>
			<div class="flex justify-end gap-2">
				<Button type="button" variant="outline" onclick={() => (pendingAssistant = null)}>
					{i18nMsg('journal-entry-form-cancel', 'Annuler')}
				</Button>
				<Button type="button" onclick={confirmAssistantReplace}>
					{i18nMsg('vat-purchase-replace-confirm', 'Remplacer')}
				</Button>
			</div>
		</div>
	</div>
{/if}
