// Story v0171 (#879) — la liste des avoirs affiche la date en forme suisse
// (`02.03.2026`, jamais `2026-03-02`). Mutation M5 : retirer `formatSwissDate` du site
// rougit ce test.
//
// Le nom du fichier n'est pas `+page.test.ts` : SvelteKit réserve le préfixe `+`.

import { describe, it, expect, vi } from 'vitest';
import { render } from '@testing-library/svelte';

vi.mock('$app/navigation', () => ({ goto: vi.fn() }));
vi.mock('$lib/shared/utils/i18n.svelte', () => ({
	i18nMsg: (_key: string, fallback: string) => fallback,
}));
vi.mock('$lib/features/credit-notes/credit-notes.api', () => ({
	listCreditNotes: () =>
		Promise.resolve({
			items: [
				{
					id: 3,
					contactId: 1,
					invoiceId: 5,
					creditNoteNumber: 'AV-2026-003',
					status: 'issued',
					date: '2026-03-02',
					totalAmount: '10.00',
				},
			],
			total: 1,
			offset: 0,
			limit: 100,
		}),
}));

import Page from './+page.svelte';

describe('liste des avoirs — date (v0171)', () => {
	it('`2026-03-02` s’affiche `02.03.2026` (mutation M5)', async () => {
		const { findByTestId, container } = render(Page);
		const row = await findByTestId('credit-note-row');
		expect(row.textContent).toContain('02.03.2026');
		expect(container.textContent).not.toContain('2026-03-02');
	});
});
