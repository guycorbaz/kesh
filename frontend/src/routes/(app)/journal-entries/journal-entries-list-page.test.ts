/**
 * v0172-correctifs (#880, AC-C2) — la liste du journal dit ses dates en forme suisse :
 * la date de chaque écriture et celle du bandeau « Livres verrouillés jusqu'au … ».
 *
 * Premier test de cette page. Patron des `*-page.test.ts` : mocks hissés AVANT
 * l'import du composant ; le `page` de `$app/state` est un objet simple (la page ne lit
 * l'adresse qu'au montage).
 */
import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { render, fireEvent, cleanup, waitFor } from '@testing-library/svelte';

vi.mock('$app/state', () => ({
	page: { url: new URL('http://localhost/journal-entries') },
}));
vi.mock('$app/navigation', () => ({ goto: vi.fn() }));
vi.mock('$lib/features/onboarding/onboarding.svelte', () => ({
	i18nMsg: (_k: string, fallback: string, args?: Record<string, string | number>) =>
		args ? fallback.replace(/\{\s*\$(\w+)\s*\}/g, (_, n) => String(args[n] ?? '')) : fallback,
}));
vi.mock('$lib/shared/utils/i18n.svelte', () => ({
	i18nMsg: (_k: string, fallback: string, args?: Record<string, string | number>) =>
		args ? fallback.replace(/\{\s*\$(\w+)\s*\}/g, (_, n) => String(args[n] ?? '')) : fallback,
	i18nLocale: () => 'fr-CH',
}));
const fetchEntriesMock = vi.fn();
vi.mock('$lib/features/journal-entries/journal-entries.api', () => ({
	fetchJournalEntries: (q: unknown) => fetchEntriesMock(q),
}));
vi.mock('$lib/features/accounts/accounts.api', () => ({ fetchAccounts: vi.fn(async () => []) }));
vi.mock('$lib/features/projects/projects.api', () => ({ listProjects: vi.fn(async () => []) }));
vi.mock('$lib/features/invoices/invoices.api', () => ({
	getInvoiceSettings: vi.fn(async () => ({ defaultVatRecoverableAccountId: null })),
}));
const companyMock = vi.fn();
vi.mock('$lib/features/settings/settings.api', () => ({
	fetchCompanyCurrent: () => companyMock(),
}));

import Page from './+page.svelte';

function entry(id: number, entryDate: string) {
	return {
		id,
		companyId: 1,
		fiscalYearId: 1,
		entryNumber: id,
		entryDate,
		journal: 'Achats',
		description: `Écriture ${id}`,
		version: 1,
		reversesEntryId: null,
		lines: [],
		createdAt: '2026-01-01T00:00:00',
		updatedAt: '2026-01-01T00:00:00',
	};
}

beforeEach(() => {
	fetchEntriesMock.mockResolvedValue({
		items: [entry(1, '2026-01-29'), entry(2, '2026-03-02')],
		total: 2,
		offset: 0,
		limit: 50,
	});
	companyMock.mockResolvedValue({ company: { booksLockedThrough: '2025-12-31' } });
});
afterEach(() => {
	cleanup();
	vi.clearAllMocks();
});

describe('liste du journal — dates en forme suisse (v0172, #880)', () => {
	it('chaque écriture dit 29.01.2026 / 02.03.2026, jamais l’ISO (MC2)', async () => {
		const { container } = render(Page);
		await waitFor(() => expect(container.textContent).toContain('Écriture 2'));
		expect(container.textContent).toContain('29.01.2026');
		expect(container.textContent).toContain('02.03.2026');
		expect(container.textContent).not.toContain('2026-01-29');
		expect(container.textContent).not.toContain('2026-03-02');
	});

	it('le bandeau du verrou dit « jusqu’au 31.12.2025 inclus » (MC2)', async () => {
		const { findByTestId } = render(Page);
		const banner = await findByTestId('books-locked-banner');
		expect(banner.textContent).toContain('31.12.2025');
		expect(banner.textContent).not.toContain('2025-12-31');
	});
});

describe('journal — une date en cours de frappe (Story 33-4)', () => {
	for (const [id, cle] of [
		['filter-date-from', 'dateFrom'],
		['filter-date-to', 'dateTo'],
	]) {
		it(`#${id} : l’année tapée n’interroge pas et ne vide pas la liste (mutation : valeur appliquée sans contrôle)`, async () => {
			const { container } = render(Page);
			await waitFor(() => expect(container.textContent).toContain('Écriture 2'));
			const input = container.querySelector(`#${id}`) as HTMLInputElement;
			const avant = fetchEntriesMock.mock.calls.length;
			// Chromium émet `change` à chaque segment.
			for (const v of ['0002-11-06', '0020-11-06', '0202-11-06']) {
				await fireEvent.change(input, { target: { value: v } });
				expect(input.value).toBe(v);
			}
			await new Promise((r) => setTimeout(r, 0));
			expect(fetchEntriesMock.mock.calls.length).toBe(avant);
			expect(container.textContent).toContain('Écriture 2');

			await fireEvent.change(input, { target: { value: '2026-11-06' } });
			await waitFor(() =>
				expect(fetchEntriesMock).toHaveBeenLastCalledWith(
					expect.objectContaining({ [cle]: '2026-11-06' }),
				),
			);
			await fireEvent.change(input, { target: { value: '0202-11-06' } });
			await fireEvent.blur(input);
			expect(input.value).toBe('2026-11-06');
		});
	}
});
