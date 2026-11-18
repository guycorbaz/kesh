/**
 * Story 37-2 (AC-5) — client des routes des cours de change.
 *
 * `fetchRates` est la **seule** entrée vers `POST /api/v1/exchange-rates/fetch`,
 * partagée par l'écran *Cours de change*, la saisie des écritures (37-3b) et la
 * réévaluation (37-5) : une récupération attend la source externe (délais de 5 s
 * et 20 s côté serveur, sous un verrou par unité), d'où un délai client de 90 s,
 * sans rejeu ; et son `503 RATE_SOURCE_UNAVAILABLE` est un refus métier, qui
 * n'allume pas le bandeau « service dégradé » (`degradedOn503: false`).
 */

import { apiClient, type CallOptions } from '$lib/shared/utils/api-client';
import type {
	AcknowledgeNotice,
	Cell,
	FetchRatesRequest,
	FetchRatesResponse,
	FetchStatusResponse,
	FiscalYearRatesResponse,
	ListRatesResponse,
	LookupResponse,
	NoticesResponse,
	PutRateRequest,
	RateKind,
	RateKindsRequest,
	RateKindsResponse,
	ResetRateRequest,
} from './exchange-rates.types';

/** Délai d'une récupération des cours (AC-5) : 90 s. */
export const FETCH_RATES_TIMEOUT_MS = 90_000;

/** Les options de l'appel de `POST /fetch` : 90 s, sans rejeu, 503 sans bandeau dégradé. */
export const FETCH_RATES_CALL: CallOptions = {
	timeoutMs: FETCH_RATES_TIMEOUT_MS,
	retry: false,
	degradedOn503: false,
};

/** Nombre maximal d'entrées d'un acquittement (borne du serveur). */
export const MAX_ACKNOWLEDGE_NOTICES = 500;

/** `GET /exchange-rates/lookup` — cours effectif (lecture seule, tout rôle). */
export async function lookupRate(
	currency: string,
	date: string,
	kind: RateKind = 'month',
): Promise<LookupResponse> {
	const q = new URLSearchParams({ currency, date, kind });
	return apiClient.get<LookupResponse>(`/api/v1/exchange-rates/lookup?${q}`);
}

/** `POST /exchange-rates/fetch` — récupère un mois ou un jour auprès de la source. */
export async function fetchRates(req: FetchRatesRequest): Promise<FetchRatesResponse> {
	return apiClient.post<FetchRatesResponse>('/api/v1/exchange-rates/fetch', req, FETCH_RATES_CALL);
}

/** `GET /exchange-rates/status` — état de la dernière récupération. */
export async function getFetchStatus(): Promise<FetchStatusResponse> {
	return apiClient.get<FetchStatusResponse>('/api/v1/exchange-rates/status');
}

/** `GET /exchange-rates?month=AAAA-MM|date=AAAA-MM-JJ[&currency=…]`. */
export async function listRates(
	params: { month: string; currency?: string } | { date: string; currency?: string },
): Promise<ListRatesResponse> {
	const q = new URLSearchParams();
	if ('month' in params) q.set('month', params.month);
	else q.set('date', params.date);
	if (params.currency) q.set('currency', params.currency);
	return apiClient.get<ListRatesResponse>(`/api/v1/exchange-rates?${q}`);
}

/** `GET /fiscal-years/{id}/exchange-rates[?all=true]` — tableau annuel. */
export async function getYearGrid(fiscalYearId: number, all = false): Promise<FiscalYearRatesResponse> {
	const suffix = all ? '?all=true' : '';
	return apiClient.get<FiscalYearRatesResponse>(
		`/api/v1/fiscal-years/${fiscalYearId}/exchange-rates${suffix}`,
	);
}

/** `PUT /exchange-rates/rates` — saisit un cours (`manual`). */
export async function putRate(req: PutRateRequest): Promise<Cell> {
	return apiClient.put<Cell>('/api/v1/exchange-rates/rates', req);
}

/** `POST /exchange-rates/rates/reset` — le ↑ : reprend la valeur publiée, ou retire un cours saisi. */
export async function resetRate(req: ResetRateRequest): Promise<Cell> {
	return apiClient.post<Cell>('/api/v1/exchange-rates/rates/reset', req);
}

/** `PUT /fiscal-years/{id}/rate-kinds` — genres d'ouverture et de clôture. */
export async function putRateKinds(
	fiscalYearId: number,
	req: RateKindsRequest,
): Promise<RateKindsResponse> {
	return apiClient.put<RateKindsResponse>(`/api/v1/fiscal-years/${fiscalYearId}/rate-kinds`, req);
}

/** `GET /exchange-rates/notices` — corrections de l'AFC à acquitter (société). */
export async function getNotices(call?: CallOptions): Promise<NoticesResponse> {
	return apiClient.get<NoticesResponse>('/api/v1/exchange-rates/notices', call);
}

/**
 * `POST /exchange-rates/notices/acknowledge` — acquitte les corrections affichées,
 * par paquets de {@link MAX_ACKNOWLEDGE_NOTICES} (borne du serveur). Rend le nombre
 * d'acquittements.
 */
export async function acknowledgeNotices(notices: AcknowledgeNotice[]): Promise<number> {
	let total = 0;
	for (let i = 0; i < notices.length; i += MAX_ACKNOWLEDGE_NOTICES) {
		const paquet = notices.slice(i, i + MAX_ACKNOWLEDGE_NOTICES);
		const r = await apiClient.post<{ acknowledged: number }>(
			'/api/v1/exchange-rates/notices/acknowledge',
			{ notices: paquet },
		);
		total += r.acknowledged;
	}
	return total;
}
