/**
 * Story 37-2 (AC-5) — types miroirs des réponses des routes des cours de change
 * (`crates/kesh-api/src/routes/exchange_rates.rs`, serde `camelCase`).
 *
 * Décimaux en chaînes (cours à dix décimales, tels que le serveur les rend) ;
 * dates `AAAA-MM-JJ` ; horodatages ISO en UTC, affichés à l'heure suisse.
 *
 * ⚠️ Miroir écrit à la main : rien ne le vérifie contre le serveur.
 */

import type { RateKind } from '$lib/features/fiscal-years/fiscal-years.types';
import type { FiscalYearResponse } from '$lib/features/fiscal-years/fiscal-years.types';

export type { RateKind };

/** La ligne `exchange_rates` d'une case (cours retenu par la société). */
export interface CellRow {
	id: number;
	/** Verrou optimiste, à renvoyer au `PUT` et au ↑. */
	version: number;
	updatedAt: string;
	/** Vrai pour une copie automatique par Kesh (`updated_by IS NULL`). */
	updatedByKesh: boolean;
}

/** La dernière valeur publiée d'une case, pour information. */
export interface CellPublished {
	/** Cours **par unité** (1 JPY, jamais 100 JPY). */
	rate: string;
	/** Unité de la publication (1, 100, 1000, 10000). */
	publishedUnit: number;
	/** Cours publié pour `publishedUnit` unités. */
	publishedRate: string;
	/** Date de publication lue (`day`), `null` pour un mois. */
	sourceDate: string | null;
	fetchedAt: string;
	/** Récupération qui a apporté la valeur actuelle (date d'une correction). */
	valueChangedAt: string;
}

/** Une case : le cours retenu par la société (une seule valeur) et la valeur publiée. */
export interface Cell {
	kind: RateKind;
	/** 1er du mois (`month`) ou jour (`day`). */
	rateDate: string;
	/** Cours retenu par la société ; `null` sans ligne. */
	effectiveRate: string | null;
	/** `afc`, `manual`, ou la valeur lue telle quelle (p. ex. `ecb`) ; `null` sans ligne. */
	source: string | null;
	row: CellRow | null;
	published: CellPublished | null;
	/** Ligne `afc` dont la valeur publiée diffère (correction de l'AFC non reprise). */
	corrected: boolean;
	/** `corrected` et acquittée à cette valeur publiée. */
	acknowledged: boolean;
}

/** Une ligne du tableau annuel : une devise. */
export interface FiscalYearRatesRow {
	currency: string;
	/** Au moins un compte de la société (archivé compris) porte la devise. */
	hasAccount: boolean;
	/** Une case par mois de l'exercice, dans l'ordre de `months`. */
	months: Cell[];
	opening: Cell;
	closing: Cell;
}

/** `GET /api/v1/fiscal-years/{id}/exchange-rates[?all=true]`. */
export interface FiscalYearRatesResponse {
	fiscalYearId: number;
	startDate: string;
	endDate: string;
	openingRateKind: RateKind;
	closingRateKind: RateKind;
	openingDate: string;
	closingDate: string;
	/** Les mois de l'exercice (`AAAA-MM-01`), dans l'ordre de l'exercice, quelle que soit sa durée. */
	months: string[];
	rows: FiscalYearRatesRow[];
}

/** `GET /api/v1/exchange-rates/lookup` — cours effectif d'une unité. */
export interface LookupResponse {
	currency: string;
	kind: RateKind;
	rateDate: string;
	effectiveRate: string;
	source: string;
	publishedRate: string | null;
	publishedUnit: number | null;
	sourceDate: string | null;
}

/** Une case avec sa devise (`GET /api/v1/exchange-rates`). */
export type CurrencyCell = Cell & { currency: string };

/** `GET /api/v1/exchange-rates?month=…|date=…`. */
export interface ListRatesResponse {
	kind: RateKind;
	rateDate: string;
	rates: CurrencyCell[];
}

/** Corps de `POST /api/v1/exchange-rates/fetch` : un mois, ou un jour. */
export type FetchRatesRequest = { year: number; month: number } | { date: string };

/** `200` de `POST /api/v1/exchange-rates/fetch`. */
export interface FetchRatesResponse {
	kind: RateKind;
	rateDate: string;
	sourceDate: string | null;
	count: number;
	rejected: number;
	corrected: number;
}

/** `GET /api/v1/exchange-rates/status` — tous les champs à `null` avant toute récupération. */
export interface FetchStatusResponse {
	trigger: 'auto' | 'manual' | null;
	lastRunAt: string | null;
	status: 'ok' | 'not_published' | 'error' | null;
	/** Code classé (`timeout`, `http_status:500`, `invalid_rates:2`…). */
	error: string | null;
	kind: RateKind | null;
	rateDate: string | null;
}

/** Corps de `PUT /api/v1/exchange-rates/rates` (cours saisi, par unité). */
export interface PutRateRequest {
	currency: string;
	kind: RateKind;
	rateDate: string;
	rate: string;
	/** Absent = création ; présent = remplacement de cette version. */
	version?: number;
}

/** Corps de `POST /api/v1/exchange-rates/rates/reset` (le ↑). */
export interface ResetRateRequest {
	currency: string;
	kind: RateKind;
	rateDate: string;
	/** Obligatoire si la ligne existe, interdite sinon. */
	version?: number;
	/** La valeur publiée que l'écran a montrée (par unité). */
	publishedRate?: string;
}

/** Une correction de l'AFC à acquitter, pour la société. */
export interface ExchangeRateNotice {
	currency: string;
	kind: RateKind;
	rateDate: string;
	effectiveRate: string;
	publishedRate: string;
	publishedUnit: number;
	valueChangedAt: string;
}

/** `GET /api/v1/exchange-rates/notices`. */
export interface NoticesResponse {
	notices: ExchangeRateNotice[];
}

/** Une entrée de `POST /api/v1/exchange-rates/notices/acknowledge`. */
export interface AcknowledgeNotice {
	currency: string;
	kind: RateKind;
	rateDate: string;
	publishedRate: string;
}

/** Corps de `PUT /api/v1/fiscal-years/{id}/rate-kinds`. */
export interface RateKindsRequest {
	openingRateKind: RateKind;
	closingRateKind: RateKind;
	/** U4 : appliquer le genre de clôture changé à l'ouverture de l'exercice suivant. */
	applyToNextOpening: boolean;
}

/** Réponse de `PUT /api/v1/fiscal-years/{id}/rate-kinds`. */
export interface RateKindsResponse {
	fiscalYear: FiscalYearResponse;
	nextFiscalYear: {
		id: number;
		openingRateKind: RateKind;
		applied: boolean;
		reason?: 'closed';
	} | null;
}
