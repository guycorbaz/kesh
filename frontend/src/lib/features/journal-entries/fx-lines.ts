/**
 * Story 37-3b (AC-2) — l'état d'une ligne en devise du formulaire d'écriture,
 * et les règles qui le gouvernent, en fonctions pures (testées sans monter le
 * composant).
 *
 * **Contrat de stockage** : le montant saisi en devise vit dans
 * `foreign.amount`, le sens dans `foreign.side` ; `debit`/`credit` de la ligne
 * ne portent **que le CHF dérivé** (l'un des deux vide selon `side`), recalculé
 * par {@link syncDerivedChf} à chaque changement — seuls lus par
 * `computeBalance`, `classifyLine` et `isDraftLineNonEmpty`. Aucun d'eux ne lit
 * jamais `foreign.amount` comme du CHF.
 *
 * **Provenance envoyée** (D9, U1, 37-3a AC-4, 4) : `afc` seulement si le cours
 * de la ligne **égale** (`Big.eq`) le cours `afc` retenu par la société pour le
 * mois de la date (`afcRate`, `afcMonth`) ; `manual` sinon. En édition, une ligne
 * dont ni le cours, ni la date, ni le compte n'ont changé garde la provenance
 * relue (`loadedRateSource`).
 */

import Big from 'big.js';
import type { AccountResponse } from '$lib/features/accounts/accounts.types';
import { accountCurrency } from '$lib/features/accounts/accounts.types';
import { fxAccountsFor } from '$lib/features/accounts/fx-accounts';
import type { CompanyJson } from '$lib/features/settings/settings.types';
import {
	computeChf,
	firstOfMonth,
	rateFromChf,
	readForeignAmount,
	readRate,
	type FxErrorCode,
} from './fx';
import { journalAmountPayload, readJournalAmount } from './balance';
import type { CreateJournalEntryLineRequest, JournalEntryLineResponse } from './journal-entries.types';

/** Provenance d'un cours envoyée par l'écran (jamais `revaluation`). */
export type SentRateSource = 'afc' | 'manual';

/** Le cours retenu par la société pour un mois, tel que l'a rendu `lookup`. */
export interface CompanyRate {
	/** `effectiveRate` du `lookup`. */
	rate: string;
	/** `source` du `lookup` : `afc`, `manual`, ou toute autre source (A30). */
	source: string;
	/** Le mois du cours (`AAAA-MM-01`, `rateDate` du `lookup`). */
	month: string;
}

/** Partie « devise » d'une ligne du formulaire (AC-2, `LineDraft.foreign`). */
export interface LineForeign {
	/** Devise **du compte** de la ligne (jamais saisie, T3). */
	currency: string;
	/** Colonne du montant en devise : elle donne le sens de la ligne. */
	side: 'debit' | 'credit';
	/** Montant en devise saisi (chaîne brute). */
	amount: string;
	/** Cours saisi ou proposé (chaîne brute). */
	rate: string;
	/** Cours venu de `lookup` et non retouché : reproposé au changement de date. */
	rateProposed: boolean;
	/** Cours `afc` retenu par la société pour `afcMonth` (`null` si `source` ≠ `afc`). */
	afcRate: string | null;
	/** Mois de `afcRate` (`AAAA-MM-01`). */
	afcMonth: string | null;
	/** Dernier cours de la société lu pour la ligne (affiché sous le champ, ↑). */
	company: CompanyRate | null;
	/** Provenance relue en édition ; `null` pour une ligne neuve. */
	loadedRateSource: 'afc' | 'manual' | 'revaluation' | null;
	/** Cours ou compte retouchés depuis le chargement. */
	touched: boolean;
	/** « Garder mon cours » après un refus : la ligne part `manual`. */
	forceManual: boolean;
	/** CHF saisi à la main, tant que le cours n'a pas pu en être déduit. */
	chfEntry: string | null;
	/** Refus du calcul du cours depuis le CHF saisi. */
	chfError: FxErrorCode | null;
}

/** Une ligne en devise vierge, pour un compte de la devise `currency`. */
export function newForeign(currency: string, side: 'debit' | 'credit' = 'debit'): LineForeign {
	return {
		currency,
		side,
		amount: '',
		rate: '',
		rateProposed: true,
		afcRate: null,
		afcMonth: null,
		company: null,
		loadedRateSource: null,
		touched: false,
		forceManual: false,
		chfEntry: null,
		chfError: null,
	};
}

/** Une ligne de brouillon au sens de ce module (sous-ensemble de `LineDraft`). */
export interface FxLineLike {
	accountId: number | null;
	debit: string;
	credit: string;
	foreign?: LineForeign;
	/** Ligne CHF calculée au cours par l'assistant TVA (son cours d'insertion). */
	chfFromForeign?: { rate: string } | null;
}

/**
 * Le CHF d'une ligne en devise, ou `null` tant que montant ou cours manquent.
 * Un refus (`FX_VALUE_OUT_OF_RANGE`, `FX_LINE_ROUNDS_TO_ZERO`) est rendu tel quel.
 */
export function foreignChf(f: LineForeign): { ok: true; value: Big } | { ok: false; error: FxErrorCode } | null {
	const amount = readForeignAmount(f.amount);
	const rate = readRate(f.rate);
	if (amount === 'empty' || rate === 'empty') return null;
	if (amount === null || rate === null) return { ok: false, error: 'FX_VALUE_OUT_OF_RANGE' };
	const r = computeChf(amount, rate);
	return r.ok ? { ok: true, value: r.value } : { ok: false, error: r.error };
}

/**
 * Pose sur `line` le CHF dérivé de sa partie en devise : `debit` ou `credit`
 * (selon `side`) au centime, l'autre vide ; les deux vides si le CHF n'est pas
 * calculable. Sans effet sur une ligne CHF.
 */
export function syncDerivedChf<T extends FxLineLike>(line: T): T {
	const f = line.foreign;
	if (!f) return line;
	const r = foreignChf(f);
	const chf = r !== null && r.ok ? r.value.toFixed(2) : '';
	line.debit = f.side === 'debit' ? chf : '';
	line.credit = f.side === 'credit' ? chf : '';
	return line;
}

/** Problème d'une ligne en devise à afficher, ou `null`. */
export type ForeignLineProblem = FxErrorCode | 'amount-unreadable' | 'rate-unreadable' | 'incomplete';

/**
 * Le message d'une ligne en devise : la lecture du montant (`amount-unreadable`,
 * ambigu compris), du cours (`rate-unreadable`), le refus de `computeChf` —
 * jamais un « montant invalide » générique —, le refus du CHF saisi, ou
 * `incomplete` (montant ou cours vide alors que l'autre est saisi).
 */
export function foreignLineProblem(f: LineForeign): ForeignLineProblem | null {
	if (f.chfError) return f.chfError;
	const amountRead = readJournalAmount(f.amount);
	if (amountRead.kind === 'ambiguous' || amountRead.kind === 'unreadable') return 'amount-unreadable';
	if (readRate(f.rate) === null) return 'rate-unreadable';
	const r = foreignChf(f);
	if (r === null) return f.amount.trim() === '' && f.rate.trim() === '' ? null : 'incomplete';
	return r.ok ? null : r.error;
}

/** Le mois (`AAAA-MM-01`) d'une date `AAAA-MM-JJ`. */
export const monthOf = firstOfMonth;

/**
 * La provenance envoyée pour une ligne en devise (D9, U1, A32) :
 * - « Garder mon cours » → `manual` ;
 * - édition, ligne non retouchée et date inchangée → la provenance relue
 *   (`afc` ou `manual`) ; le montant n'en fait pas partie ;
 * - sinon `afc` si `afcMonth` est le mois de la date et que le cours égale
 *   `afcRate` (en valeur), `manual` dans tous les autres cas.
 */
export function rateSourceToSend(f: LineForeign, entryDate: string, dateChanged: boolean): SentRateSource {
	if (f.forceManual) return 'manual';
	if (!f.touched && !dateChanged && (f.loadedRateSource === 'afc' || f.loadedRateSource === 'manual')) {
		return f.loadedRateSource;
	}
	if (f.afcRate === null || f.afcMonth !== monthOf(entryDate)) return 'manual';
	const rate = readRate(f.rate);
	if (rate === null || rate === 'empty') return 'manual';
	return rate.eq(new Big(f.afcRate)) ? 'afc' : 'manual';
}

/**
 * Le corps d'une ligne du formulaire (`POST`/`PUT /journal-entries`) : une ligne
 * CHF comme avant ; une ligne en devise porte `foreign` (devise du compte
 * toujours envoyée, montant et cours sérialisés par `toFixed`, provenance), et
 * `debit`/`credit` son CHF dérivé. `null` si une valeur n'est pas envoyable.
 */
export function lineToPayload(
	line: FxLineLike & { projectId: number | null },
	entryDate: string,
	dateChanged: boolean,
): CreateJournalEntryLineRequest | null {
	const debit = journalAmountPayload(line.debit);
	const credit = journalAmountPayload(line.credit);
	if (debit === null || credit === null || line.accountId === null) return null;
	const out: CreateJournalEntryLineRequest = {
		accountId: line.accountId,
		debit,
		credit,
		projectId: line.projectId,
	};
	const f = line.foreign;
	if (f) {
		const amount = readForeignAmount(f.amount);
		const rate = readRate(f.rate);
		if (amount === null || amount === 'empty' || rate === null || rate === 'empty') return null;
		out.foreign = {
			currency: f.currency,
			amount: amount.toFixed(),
			rate: rate.toFixed(),
			rateSource: rateSourceToSend(f, entryDate, dateChanged),
		};
	}
	return out;
}

/**
 * Le CHF saisi à la main (« cours OU CHF », A32) : si le cours `afc` retenu du
 * mois de la date redonne ce CHF, le cours devient ce cours `afc` ; sinon le
 * cours de {@link rateFromChf}. Rend la partie en devise mise à jour (cours
 * retouché, plus proposé), ou avec `chfError` si aucun cours ne convient.
 */
export function applyChfEntry(f: LineForeign, rawChf: string, entryDate: string): LineForeign {
	const next: LineForeign = { ...f, chfEntry: rawChf, chfError: null };
	const chfRead = readJournalAmount(rawChf);
	const amount = readForeignAmount(f.amount);
	if (chfRead.kind === 'empty') return { ...next, chfEntry: null };
	if (chfRead.kind !== 'clear' || amount === null || amount === 'empty') {
		return { ...next, chfError: 'FX_VALUE_OUT_OF_RANGE' };
	}
	const chf = chfRead.value;
	if (f.afcRate !== null && f.afcMonth === monthOf(entryDate)) {
		const viaAfc = computeChf(amount, f.afcRate);
		if (viaAfc.ok && viaAfc.value.eq(chf)) {
			return { ...next, rate: new Big(f.afcRate).toFixed(), chfEntry: null, rateProposed: false, touched: true };
		}
	}
	const r = rateFromChf(amount, chf);
	if (!r.ok) return { ...next, chfError: r.error };
	return { ...next, rate: r.value.toFixed(), chfEntry: null, rateProposed: false, touched: true };
}

/**
 * Le compte d'une ligne change (U2, CR-5) — la devise suit le compte :
 * - vers un compte CHF : `foreign` retiré, montant CHF vide ;
 * - CHF → devise, ou devise → **autre** devise : partie en devise neuve
 *   (montant et cours vides, cours à reproposer), sens gardé s'il y en avait un ;
 * - même devise : montant et cours gardés ;
 * dans tous les cas, une retouche (la provenance relue n'est plus gardée).
 */
export function applyAccountChange<T extends FxLineLike>(line: T, account: AccountResponse | undefined): T {
	const currency = account ? accountCurrency(account) : null;
	const before = line.foreign;
	line.accountId = account?.id ?? null;
	if (currency === null) {
		if (before) {
			delete line.foreign;
			line.debit = '';
			line.credit = '';
		}
		return line;
	}
	if (before && before.currency === currency) {
		line.foreign = { ...before, touched: true };
		return syncDerivedChf(line);
	}
	const side: 'debit' | 'credit' = before?.side ?? (line.credit.trim() !== '' && line.debit.trim() === '' ? 'credit' : 'debit');
	line.foreign = { ...newForeign(currency, side), touched: true };
	line.debit = '';
	line.credit = '';
	return line;
}

/** Ce que rend un `lookup` pour (devise, mois) : un cours de la société, ou aucun. */
export type LookupOutcome = { kind: 'rate'; rate: CompanyRate } | { kind: 'none' };

/**
 * Applique le résultat d'un `lookup` du mois `month` à une ligne en devise :
 * `afcRate`/`afcMonth` rafraîchis (cours `afc` seulement si `source = afc`,
 * A30), cours de la société retenu pour l'affichage ; le cours n'est remplacé
 * que pour une ligne **proposée** — effacé si le mois n'a pas de cours.
 */
export function applyLookup(f: LineForeign, month: string, outcome: LookupOutcome): LineForeign {
	if (outcome.kind === 'none') {
		return {
			...f,
			afcRate: null,
			afcMonth: month,
			company: null,
			rate: f.rateProposed ? '' : f.rate,
		};
	}
	const { rate } = outcome;
	return {
		...f,
		afcRate: rate.source === 'afc' ? rate.rate : null,
		afcMonth: month,
		company: rate,
		rate: f.rateProposed ? new Big(rate.rate).toFixed() : f.rate,
	};
}

/**
 * Le cours de la société affiché sous le champ (D9, U1) quand il diffère du
 * cours de la ligne — seulement pour le mois de la date courante, aucun
 * `lookup` de ce mois en cours ; `null` sinon (aucune mention, aucun ↑).
 */
export function companyRateHint(f: LineForeign, entryDate: string, pending: boolean): CompanyRate | null {
	if (pending || f.company === null || f.company.month !== monthOf(entryDate)) return null;
	const rate = readRate(f.rate);
	if (rate !== null && rate !== 'empty' && rate.eq(new Big(f.company.rate))) return null;
	return f.company;
}

/** Le ↑ : la ligne reprend le cours de la société (sans jamais toucher la table des cours). */
export function adoptCompanyRate(f: LineForeign, company: CompanyRate): LineForeign {
	return {
		...f,
		rate: new Big(company.rate).toFixed(),
		rateProposed: true,
		touched: true,
		forceManual: false,
		chfEntry: null,
		chfError: null,
	};
}

/**
 * Indicateur du cours d'une ligne : la provenance du cours de la société quand
 * la ligne le porte (« AFC », « cours saisi », « cours saisi (ecb) ») ; sinon
 * `manual` (retouché à la main).
 */
export function rateIndicator(f: LineForeign): { kind: 'afc' } | { kind: 'saisi'; source: string | null } | { kind: 'manual' } {
	const rate = readRate(f.rate);
	if (f.company !== null && rate !== null && rate !== 'empty' && rate.eq(new Big(f.company.rate))) {
		if (f.company.source === 'afc') return { kind: 'afc' };
		return { kind: 'saisi', source: f.company.source === 'manual' ? null : f.company.source };
	}
	if (!f.touched && f.loadedRateSource === 'afc') return { kind: 'afc' };
	return { kind: 'manual' };
}

/**
 * Une ligne de la réponse en partie en devise de brouillon (`lineResponseToDraft`) :
 * devise de la réponse (lue sur le compte par le serveur), sens selon la colonne
 * du CHF, provenance relue ; une ligne relue `afc` est tenue pour proposée.
 */
export function foreignFromResponse(line: JournalEntryLineResponse): LineForeign | undefined {
	const f = line.foreign;
	if (!f) return undefined;
	const side: 'debit' | 'credit' = new Big(line.credit).gt(0) ? 'credit' : 'debit';
	return {
		...newForeign(f.currency, side),
		amount: f.amount,
		rate: f.rate,
		rateProposed: f.rateSource === 'afc',
		loadedRateSource: f.rateSource,
	};
}

/** Les devises portées par des lignes (hors ligne `except`). */
export function lineCurrencies(lines: FxLineLike[], except?: number): Set<string> {
	const out = new Set<string>();
	lines.forEach((l, i) => {
		if (i !== except && l.foreign) out.add(l.foreign.currency);
	});
	return out;
}

/**
 * Une devise par écriture (R7) : les devises qu'offre le sélecteur de la ligne
 * `index` — `null` = toutes ; sinon celles des **autres** lignes, ou, pour une
 * écriture relue qui en porte déjà plusieurs (ouverture, complément, état
 * hérité), celles de l'écriture relue — jamais une de plus.
 */
export function allowedCurrencies(lines: FxLineLike[], index: number, loaded: Set<string>): Set<string> | null {
	if (loaded.size > 1) return loaded;
	const others = lineCurrencies(lines, index);
	return others.size === 0 ? null : others;
}

/** Les comptes offerts au sélecteur : CHF, et ceux des devises admises. */
export function accountsForCurrencies(accounts: AccountResponse[], allowed: Set<string> | null): AccountResponse[] {
	if (allowed === null) return accounts;
	return accounts.filter((a) => {
		const c = accountCurrency(a);
		return c === null || allowed.has(c);
	});
}

/** Le refus de la ligne d'écart, ou les comptes de la ligne. */
export type DifferenceLinePlan =
	| { kind: 'none' }
	| {
			kind: 'invalid';
			/** Numéro du compte en devise retenu. */
			holderNumber: string;
			/** Comptes propres du compte, ou réglage de la société, ou aucun réglage. */
			origin: 'own' | 'company' | 'missing';
	  }
	| {
			kind: 'line';
			accountId: number;
			side: 'debit' | 'credit';
			/** Montant au centime (|Σ débit − Σ crédit|). */
			amount: string;
			/** Le compte en devise retenu. */
			holderNumber: string;
			origin: 'own' | 'company';
			/** D'autres comptes en devise de l'écriture résolvent une autre paire (A12). */
			mixedPairs: boolean;
			/** L'écart dépasse n × 0.005 CHF : ce n'est pas un arrondi. */
			beyondRounding: boolean;
	  };

/** Un compte de différence de change utilisable (actif, imputable, charge ou produit, en CHF). */
function usableFxAccount(accounts: AccountResponse[], id: number): boolean {
	const a = accounts.find((x) => x.id === id);
	return (
		a !== undefined &&
		a.active &&
		a.postable &&
		(a.accountType === 'Expense' || a.accountType === 'Revenue') &&
		accountCurrency(a) === null
	);
}

/**
 * La ligne de différence de change que le bouton ajouterait (R8, S2, A12) :
 * sur les comptes de différence (`fxAccountsFor`) du compte en devise concerné —
 * le premier compte en devise de bilan dans l'ordre des lignes, sinon le premier
 * compte en devise ; au crédit du compte de gains si les débits l'emportent, au
 * débit du compte de pertes sinon. `none` si l'écriture est équilibrée ou ne
 * porte aucune ligne en devise.
 *
 * `totalDebit`/`totalCredit` : les totaux du formulaire (`computeBalance`).
 */
export function planDifferenceLine(
	lines: FxLineLike[],
	accounts: AccountResponse[],
	company: Pick<CompanyJson, 'fxGainAccountId' | 'fxLossAccountId'> | null,
	totalDebit: Big,
	totalCredit: Big,
): DifferenceLinePlan {
	const diff = totalDebit.minus(totalCredit);
	if (diff.eq(0)) return { kind: 'none' };
	const fxAccounts = lines
		.filter((l) => l.foreign && l.accountId !== null)
		.map((l) => accounts.find((a) => a.id === l.accountId))
		.filter((a): a is AccountResponse => a !== undefined);
	if (fxAccounts.length === 0) return { kind: 'none' };
	const holder =
		fxAccounts.find((a) => a.accountType === 'Asset' || a.accountType === 'Liability') ?? fxAccounts[0];
	const settings = company ?? { fxGainAccountId: null, fxLossAccountId: null };
	const pair = fxAccountsFor(holder, settings);
	if (pair === null) return { kind: 'invalid', holderNumber: holder.number, origin: 'missing' };
	if (!usableFxAccount(accounts, pair.gain) || !usableFxAccount(accounts, pair.loss)) {
		return { kind: 'invalid', holderNumber: holder.number, origin: pair.origin };
	}
	const mixedPairs = fxAccounts.some((a) => {
		const p = fxAccountsFor(a, settings);
		return p === null || p.gain !== pair.gain || p.loss !== pair.loss;
	});
	const derived = lines.filter((l) => l.foreign || l.chfFromForeign).length;
	const threshold = new Big('0.005').times(derived);
	const amount = diff.abs();
	return {
		kind: 'line',
		accountId: diff.gt(0) ? pair.gain : pair.loss,
		side: diff.gt(0) ? 'credit' : 'debit',
		amount: amount.toFixed(2),
		holderNumber: holder.number,
		origin: pair.origin,
		mixedPairs,
		beyondRounding: amount.gt(threshold),
	};
}

/**
 * Le rang `line` (1-based) d'un refus du serveur, dans le corps envoyé — qui
 * omet les lignes vides —, traduit en indice de ligne du formulaire par la table
 * `sentIndexes` (indice de la ligne du formulaire de chaque ligne envoyée).
 */
export function formIndexOfServerLine(sentIndexes: number[], line: unknown): number | null {
	if (typeof line !== 'number' || !Number.isInteger(line) || line < 1) return null;
	return sentIndexes[line - 1] ?? null;
}
