// Story 37-2 (AC-5) — client des routes des cours de change : URL, corps, et
// options de la récupération (90 s, sans rejeu, 503 sans bandeau dégradé).

import { describe, it, expect, vi, beforeEach } from 'vitest';

const { get, post, put } = vi.hoisted(() => ({ get: vi.fn(), post: vi.fn(), put: vi.fn() }));
vi.mock('$lib/shared/utils/api-client', () => ({ apiClient: { get, post, put } }));

import {
	acknowledgeNotices,
	FETCH_RATES_CALL,
	FETCH_RATES_TIMEOUT_MS,
	fetchRates,
	getFetchStatus,
	getNotices,
	getYearGrid,
	listRates,
	lookupRate,
	MAX_ACKNOWLEDGE_NOTICES,
	putRate,
	putRateKinds,
	resetRate,
} from './exchange-rates.api';

describe('exchange-rates.api', () => {
	beforeEach(() => {
		vi.clearAllMocks();
		post.mockResolvedValue({ acknowledged: 0 });
	});

	it('fetchRates : POST /fetch, 90 s, sans rejeu, degradedOn503: false', async () => {
		await fetchRates({ year: 2024, month: 12 });
		expect(FETCH_RATES_TIMEOUT_MS).toBe(90_000);
		expect(post).toHaveBeenCalledWith(
			'/api/v1/exchange-rates/fetch',
			{ year: 2024, month: 12 },
			{ timeoutMs: 90_000, retry: false, degradedOn503: false },
		);
		expect(FETCH_RATES_CALL.degradedOn503).toBe(false);
		await fetchRates({ date: '2024-06-28' });
		expect(post).toHaveBeenLastCalledWith('/api/v1/exchange-rates/fetch', { date: '2024-06-28' }, FETCH_RATES_CALL);
	});

	it('lookup, liste, état, tableau annuel', async () => {
		await lookupRate('USD', '2024-12-15');
		expect(get).toHaveBeenLastCalledWith('/api/v1/exchange-rates/lookup?currency=USD&date=2024-12-15&kind=month');
		await lookupRate('JPY', '2024-12-15', 'day');
		expect(get).toHaveBeenLastCalledWith('/api/v1/exchange-rates/lookup?currency=JPY&date=2024-12-15&kind=day');
		await listRates({ month: '2024-12', currency: 'USD' });
		expect(get).toHaveBeenLastCalledWith('/api/v1/exchange-rates?month=2024-12&currency=USD');
		await listRates({ date: '2024-12-02' });
		expect(get).toHaveBeenLastCalledWith('/api/v1/exchange-rates?date=2024-12-02');
		await getFetchStatus();
		expect(get).toHaveBeenLastCalledWith('/api/v1/exchange-rates/status');
		await getYearGrid(7);
		expect(get).toHaveBeenLastCalledWith('/api/v1/fiscal-years/7/exchange-rates');
		await getYearGrid(7, true);
		expect(get).toHaveBeenLastCalledWith('/api/v1/fiscal-years/7/exchange-rates?all=true');
		await getNotices({ retry: false });
		expect(get).toHaveBeenLastCalledWith('/api/v1/exchange-rates/notices', { retry: false });
	});

	it('PUT d’un cours, ↑, genres', async () => {
		await putRate({ currency: 'USD', kind: 'month', rateDate: '2024-12-01', rate: '0.89', version: 2 });
		expect(put).toHaveBeenLastCalledWith('/api/v1/exchange-rates/rates', {
			currency: 'USD',
			kind: 'month',
			rateDate: '2024-12-01',
			rate: '0.89',
			version: 2,
		});
		await resetRate({ currency: 'USD', kind: 'day', rateDate: '2024-06-30', publishedRate: '0.9' });
		expect(post).toHaveBeenLastCalledWith('/api/v1/exchange-rates/rates/reset', {
			currency: 'USD',
			kind: 'day',
			rateDate: '2024-06-30',
			publishedRate: '0.9',
		});
		await putRateKinds(3, { openingRateKind: 'month', closingRateKind: 'day', applyToNextOpening: false });
		expect(put).toHaveBeenLastCalledWith('/api/v1/fiscal-years/3/rate-kinds', {
			openingRateKind: 'month',
			closingRateKind: 'day',
			applyToNextOpening: false,
		});
	});

	it('acquittement : la liste affichée, par paquets de 500', async () => {
		post.mockResolvedValue({ acknowledged: 2 });
		const n = { currency: 'USD', kind: 'month' as const, rateDate: '2024-12-01', publishedRate: '0.885' };
		expect(await acknowledgeNotices([n, n])).toBe(2);
		expect(post).toHaveBeenCalledWith('/api/v1/exchange-rates/notices/acknowledge', { notices: [n, n] });
		post.mockClear();
		await acknowledgeNotices(Array.from({ length: MAX_ACKNOWLEDGE_NOTICES + 1 }, () => n));
		expect(post).toHaveBeenCalledTimes(2);
		expect(post.mock.calls[0][1].notices).toHaveLength(500);
		expect(post.mock.calls[1][1].notices).toHaveLength(1);
	});
});
