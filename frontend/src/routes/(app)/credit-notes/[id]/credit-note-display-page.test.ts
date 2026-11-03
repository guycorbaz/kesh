// Story v0171 (#866, #879) — la fiche d'un avoir : la quantité d'une ligne lisible
// (`1.0000` → `1`, `readableAmount`) et la date en forme suisse (`02.03.2026`,
// `formatSwissDate`). Mutations M4/M5 : retirer le helper d'un site rougit le test qui
// le nomme.
//
// Le nom du fichier n'est pas `+page.test.ts` : SvelteKit réserve le préfixe `+`.

import { describe, it, expect, vi, afterEach } from 'vitest';
import { render, cleanup } from '@testing-library/svelte';
import type { CreditNoteResponse } from '$lib/features/credit-notes/credit-notes.types';

vi.mock('$app/navigation', () => ({ goto: vi.fn() }));
vi.mock('$app/state', () => ({
	page: { params: { id: '3' }, url: new URL('http://localhost/credit-notes/3') },
}));
vi.mock('$lib/shared/utils/i18n.svelte', () => ({
	i18nMsg: (_key: string, fallback: string) => fallback,
}));
vi.mock('$lib/shared/utils/notify', () => ({ notifyError: vi.fn() }));
const getMock = vi.fn();
vi.mock('$lib/features/credit-notes/credit-notes.api', () => ({
	getCreditNote: (id: number) => getMock(id),
}));
vi.mock('$lib/features/accounts/accounts.api', () => ({
	fetchAccounts: vi.fn(async () => []),
}));

import Page from './+page.svelte';

function ligne(position: number, quantity: string) {
	return {
		position,
		description: `Ligne ${position}`,
		quantity,
		unitPrice: '10.00',
		vatRate: '8.10',
		lineTotal: '10.00',
		revenueAccountId: null,
	};
}

function avoir(over: Partial<CreditNoteResponse> = {}): CreditNoteResponse {
	return {
		id: 3,
		contactId: 1,
		invoiceId: 5,
		creditNoteNumber: 'AV-2026-003',
		status: 'issued',
		date: '2026-03-02',
		totalAmount: '10.00',
		journalEntryId: 9,
		version: 1,
		createdAt: '2026-03-02T09:00:00',
		lines: [],
		...over,
	};
}

afterEach(() => {
	cleanup();
	vi.clearAllMocks();
});

describe('fiche d’un avoir — affichage (v0171)', () => {
	it('quantité `1.0000` ⇒ `1`, `2.5000` ⇒ `2.5`, `33.3333` gardé (mutation M4)', async () => {
		getMock.mockResolvedValue(
			avoir({ lines: [ligne(1, '1.0000'), ligne(2, '2.5000'), ligne(3, '33.3333')] }),
		);
		const { findAllByTestId, container } = render(Page);
		const cells = await findAllByTestId('credit-note-line-quantity');
		expect(cells.map((c) => c.textContent?.trim())).toEqual(['1', '2.5', '33.3333']);
		expect(container.textContent).not.toContain('1.0000');
	});

	it('date `02.03.2026`, jamais l’ISO (mutation M5 : site `creditNote.date`)', async () => {
		getMock.mockResolvedValue(avoir());
		const { findByTestId, container } = render(Page);
		expect((await findByTestId('credit-note-date')).textContent?.trim()).toBe('02.03.2026');
		expect(container.textContent).not.toContain('2026-03-02');
	});
});
