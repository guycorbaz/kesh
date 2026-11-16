/**
 * Story 37-3b (AC-1, AC-2) — le cours retenu par la société pour une devise et
 * un mois (ou un jour), lu par `GET /exchange-rates/lookup`, mémoïsé par
 * formulaire.
 *
 * - mémoïsation par `(devise, genre, mois ou jour)`, **résultats négatifs
 *   compris** ;
 * - `lookup` est en lecture seule et ne rend `404 RATE_UNAVAILABLE` que si la
 *   société n'a **aucun** cours pour ce mois : alors **un** `POST
 *   /exchange-rates/fetch` (la fonction de 37-2, seule entrée), puis une
 *   relecture qui **remplace** l'entrée mémoïsée — pour les rôles Comptable et
 *   Administrateur, et seulement pour un mois récupérable (≥ 2000-01 et ≤ le
 *   mois suivant le mois courant, à toute date ; un jour : ≥ 2000-01-01 et
 *   strictement passé) ; sinon la saisie manuelle est exigée (`note`) ;
 * - un `404 RATE_NOT_PUBLISHED`, un `503`, un `400` du `POST /fetch` se
 *   traitent comme « non publié » : saisie manuelle.
 *
 * Aucune dépendance au composant : les appels sont injectés (tests sans réseau).
 */

import { isApiError } from '$lib/shared/utils/api-client';
import type { FetchRatesRequest, LookupResponse } from '$lib/features/exchange-rates/exchange-rates.types';
import { firstOfMonth, type RateKind } from './fx';
import type { LookupOutcome } from './fx-lines';

/** Les appels d'un cache (ceux de `exchange-rates.api.ts` en production). */
export interface RateLookupDeps {
	lookup: (currency: string, date: string, kind: RateKind) => Promise<LookupResponse>;
	fetch: (req: FetchRatesRequest) => Promise<unknown>;
	/** Rôle Comptable ou Administrateur (`canEditRates`). */
	canFetch: () => boolean;
	/** La date du jour du poste (`AAAA-MM-JJ`). */
	today: () => string;
}

/** Pourquoi aucun cours n'est proposé : la saisie manuelle est exigée. */
export type LookupNote = 'manual-required' | null;

/** Résultat d'un `lookup` mémoïsé. */
export interface LookupResult {
	outcome: LookupOutcome;
	/** `manual-required` quand aucun cours n'a pu être lu ni récupéré. */
	note: LookupNote;
}

/** Le mois suivant le mois d'une date (`AAAA-MM-01`). */
function nextMonth(iso: string): string {
	const y = Number(iso.slice(0, 4));
	const m = Number(iso.slice(5, 7));
	const ny = m === 12 ? y + 1 : y;
	const nm = m === 12 ? 1 : m + 1;
	return `${ny}-${String(nm).padStart(2, '0')}-01`;
}

/**
 * Le `POST /fetch` est-il permis pour cette unité (37-2 AC-5) ? Un mois :
 * ≥ 2000-01 et ≤ le mois suivant le mois courant ; un jour : ≥ 2000-01-01 et
 * strictement passé.
 */
export function isFetchable(kind: RateKind, date: string, today: string): boolean {
	if (date < '2000-01-01') return false;
	if (kind === 'day') return date < today;
	return firstOfMonth(date) <= nextMonth(today);
}

/** La requête de `POST /fetch` d'une unité. */
function fetchRequest(kind: RateKind, date: string): FetchRatesRequest {
	if (kind === 'day') return { date };
	return { year: Number(date.slice(0, 4)), month: Number(date.slice(5, 7)) };
}

/** Échec passager : réseau, délai (408), limite (429) ou réponse 5xx. */
function isTransientFailure(err: unknown): boolean {
	return !isApiError(err) || err.status >= 500 || err.status === 408 || err.status === 429;
}

/** Le cache d'un formulaire (fabriqué par {@link createRateLookupCache}). */
export interface RateLookupCache {
	/**
	 * Le cours retenu par la société pour `currency` à `date` (le mois de la date
	 * pour le genre `month`). Mémoïsé ; le repli du `POST /fetch` n'est tenté
	 * qu'une fois par entrée.
	 */
	get(currency: string, date: string, kind?: RateKind): Promise<LookupResult>;
	/**
	 * « Récupérer le mois » (refus `RATE_UNAVAILABLE` du serveur) : un `POST
	 * /fetch` puis une relecture qui **remplace** l'entrée mémoïsée, négative
	 * comprise.
	 */
	refresh(currency: string, date: string, kind?: RateKind): Promise<LookupResult>;
}

/**
 * Fabrique le cache d'un formulaire. Une fermeture et non une classe : les
 * gardes de libellé du dépôt ne lisent pas les méthodes de classe
 * (`i18n-libelle-en-dur.test.ts`).
 */
export function createRateLookupCache(deps: RateLookupDeps): RateLookupCache {
	const memo = new Map<string, Promise<LookupResult>>();
	/**
	 * Suivi d'une résolution : une lecture qui échoue pour une raison
	 * **passagère** (réseau, délai, réponse 5xx) le note, et son résultat
	 * négatif n'est pas mémoïsé — la lecture suivante réessaie (revue de code
	 * P1, 37-3b-CR-P1-7).
	 */
	type Trace = { transient: boolean };
	const key = (currency: string, kind: RateKind, date: string) =>
		`${currency}|${kind}|${kind === 'month' ? firstOfMonth(date) : date}`;

	async function read(currency: string, date: string, kind: RateKind, trace?: Trace): Promise<LookupOutcome | 'unavailable'> {
		try {
			const r = await deps.lookup(currency, kind === 'month' ? firstOfMonth(date) : date, kind);
			return {
				kind: 'rate',
				rate: { rate: r.effectiveRate, source: r.source, month: kind === 'month' ? firstOfMonth(r.rateDate) : r.rateDate },
			};
		} catch (err) {
			if (isApiError(err) && err.code === 'RATE_UNAVAILABLE') return 'unavailable';
			if (trace && isTransientFailure(err)) trace.transient = true;
			return { kind: 'none' };
		}
	}

	async function fetchThenRead(currency: string, date: string, kind: RateKind, trace?: Trace): Promise<LookupResult> {
		if (!deps.canFetch() || !isFetchable(kind, date, deps.today())) {
			return { outcome: { kind: 'none' }, note: 'manual-required' };
		}
		try {
			await deps.fetch(fetchRequest(kind, date));
		} catch (err) {
			// `RATE_NOT_PUBLISHED`, `400` : saisie manuelle, mémoïsée. Un échec
			// passager (réseau, 5xx) ne l'est pas (revue de code P2, 37-3b-CR-P2-3).
			if (trace && isTransientFailure(err)) trace.transient = true;
			return { outcome: { kind: 'none' }, note: 'manual-required' };
		}
		const second = await read(currency, date, kind, trace);
		if (second === 'unavailable' || second.kind === 'none') {
			return { outcome: { kind: 'none' }, note: 'manual-required' };
		}
		return { outcome: second, note: null };
	}

	async function resolve(currency: string, date: string, kind: RateKind, trace: Trace): Promise<LookupResult> {
		const first = await read(currency, date, kind, trace);
		if (first !== 'unavailable') return { outcome: first, note: first.kind === 'none' ? 'manual-required' : null };
		return fetchThenRead(currency, date, kind, trace);
	}

	return {
		get(currency, date, kind = 'month') {
			const k = key(currency, kind, date);
			const hit = memo.get(k);
			if (hit) return hit;
			const trace: Trace = { transient: false };
			const p: Promise<LookupResult> = resolve(currency, date, kind, trace).then((r) => {
				// Un échec passager ne se mémoïse pas : l'entrée est retirée.
				if (trace.transient && memo.get(k) === p) memo.delete(k);
				return r;
			});
			memo.set(k, p);
			return p;
		},
		refresh(currency, date, kind = 'month') {
			const k = key(currency, kind, date);
			const trace: Trace = { transient: false };
			const p: Promise<LookupResult> = fetchThenRead(currency, date, kind, trace).then((r) => {
				if (trace.transient && memo.get(k) === p) memo.delete(k);
				return r;
			});
			memo.set(k, p);
			return p;
		},
	};
}
