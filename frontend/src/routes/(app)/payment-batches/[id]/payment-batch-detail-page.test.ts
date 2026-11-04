// Story v0171 (#871) — la fiche d'un lot affiche la date d'exécution et « Confirmé le »
// en forme suisse (`29.01.2026`), jamais l'ISO ni l'horodatage naïf du serveur
// (`2026-01-29T10:15:00`). Mutation M5 : retirer `formatSwissDate` d'un site rougit le
// test qui le nomme.
//
// Le nom du fichier n'est pas `+page.test.ts` : SvelteKit réserve le préfixe `+`.

import { describe, it, expect, vi, afterEach } from 'vitest';
import { render, cleanup } from '@testing-library/svelte';
import type { PaymentBatchResponse } from '$lib/features/payment-batches/payment-batches.types';

vi.mock('$app/navigation', () => ({ goto: vi.fn() }));
vi.mock('$app/state', () => ({
	page: { params: { id: '7' }, url: new URL('http://localhost/payment-batches/7') },
}));
vi.mock('$lib/shared/utils/i18n.svelte', () => ({
	i18nMsg: (_key: string, fallback: string) => fallback,
}));
const getMock = vi.fn();
vi.mock('$lib/features/payment-batches/payment-batches.api', () => ({
	getPaymentBatch: (id: number) => getMock(id),
	confirmPaymentBatch: vi.fn(),
	cancelPaymentBatch: vi.fn(),
	pain001DownloadUrl: (id: number) => `/api/v1/payment-batches/${id}/pain001`,
}));

import Page from './+page.svelte';

function lot(over: Partial<PaymentBatchResponse> = {}): PaymentBatchResponse {
	return {
		id: 7,
		bankAccountId: 1,
		status: 'confirmed',
		requestedExecutionDate: '2026-01-29',
		totalAmount: '100.00',
		msgId: 'MSG-7',
		confirmedAt: '2026-02-03T10:15:00',
		version: 2,
		createdAt: '2026-01-20T08:00:00',
		items: [],
		...over,
	};
}

afterEach(() => {
	cleanup();
	vi.clearAllMocks();
});

describe('fiche d’un lot de virements — dates (v0171)', () => {
	it('date d’exécution `29.01.2026` (mutation M5 : site `requestedExecutionDate`)', async () => {
		getMock.mockResolvedValue(lot());
		const { findByTestId, container } = render(Page);
		expect((await findByTestId('payment-batch-execution-date')).textContent?.trim()).toBe(
			'29.01.2026',
		);
		expect(container.textContent).not.toContain('2026-01-29');
	});

	it('« Confirmé le » `03.02.2026`, sans l’heure (mutation M5 : site `confirmedAt`)', async () => {
		getMock.mockResolvedValue(lot());
		const { findByTestId, container } = render(Page);
		expect((await findByTestId('payment-batch-confirmed-at')).textContent?.trim()).toBe(
			'03.02.2026',
		);
		expect(container.textContent).not.toContain('2026-02-03');
		expect(container.textContent).not.toContain('10:15');
	});
});
