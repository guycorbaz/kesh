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

/** Le cache d'un formulaire. */
export class RateLookupCache {
	private readonly memo = new Map<string, Promise<LookupResult>>();

	constructor(private readonly deps: RateLookupDeps) {}

	private key(currency: string, kind: RateKind, date: string): string {
		return `${currency}|${kind}|${kind === 'month' ? firstOfMonth(date) : date}`;
	}

	/**
	 * Le cours retenu par la société pour `currency` à `date` (le mois de la date
	 * pour le genre `month`). Mémoïsé ; le repli du `POST /fetch` n'est tenté
	 * qu'une fois par entrée.
	 */
	get(currency: string, date: string, kind: RateKind = 'month'): Promise<LookupResult> {
		const k = this.key(currency, kind, date);
		const hit = this.memo.get(k);
		if (hit) return hit;
		const p = this.resolve(currency, date, kind, true);
		this.memo.set(k, p);
		return p;
	}

	/**
	 * « Récupérer le mois » (refus `RATE_UNAVAILABLE` du serveur) : un `POST
	 * /fetch` puis une relecture qui **remplace** l'entrée mémoïsée, négative
	 * comprise.
	 */
	refresh(currency: string, date: string, kind: RateKind = 'month'): Promise<LookupResult> {
		const k = this.key(currency, kind, date);
		const p = this.fetchThenRead(currency, date, kind);
		this.memo.set(k, p);
		return p;
	}

	private async read(currency: string, date: string, kind: RateKind): Promise<LookupOutcome | 'unavailable'> {
		try {
			const r = await this.deps.lookup(currency, kind === 'month' ? firstOfMonth(date) : date, kind);
			return {
				kind: 'rate',
				rate: { rate: r.effectiveRate, source: r.source, month: kind === 'month' ? firstOfMonth(r.rateDate) : r.rateDate },
			};
		} catch (err) {
			if (isApiError(err) && err.code === 'RATE_UNAVAILABLE') return 'unavailable';
			return { kind: 'none' };
		}
	}

	private async resolve(currency: string, date: string, kind: RateKind, mayFetch: boolean): Promise<LookupResult> {
		const first = await this.read(currency, date, kind);
		if (first !== 'unavailable') return { outcome: first, note: first.kind === 'none' ? 'manual-required' : null };
		if (!mayFetch) return { outcome: { kind: 'none' }, note: 'manual-required' };
		return this.fetchThenRead(currency, date, kind);
	}

	private async fetchThenRead(currency: string, date: string, kind: RateKind): Promise<LookupResult> {
		if (!this.deps.canFetch() || !isFetchable(kind, date, this.deps.today())) {
			return { outcome: { kind: 'none' }, note: 'manual-required' };
		}
		try {
			await this.deps.fetch(fetchRequest(kind, date));
		} catch {
			// `RATE_NOT_PUBLISHED`, `503`, `400` : saisie manuelle.
			return { outcome: { kind: 'none' }, note: 'manual-required' };
		}
		const second = await this.read(currency, date, kind);
		if (second === 'unavailable' || second.kind === 'none') {
			return { outcome: { kind: 'none' }, note: 'manual-required' };
		}
		return { outcome: second, note: null };
	}
}
