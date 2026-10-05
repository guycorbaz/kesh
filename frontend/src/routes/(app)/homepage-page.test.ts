// Story 25-6-a (#388, #389), revue de code P2 — le câblage de la page d'accueil :
// un rejet de l'API des comptes bancaires mène à la tuile en échec, pas à une
// tuile cachée (le défaut même de #388). Les quatre sources sont mockées.
import { describe, it, expect, vi } from 'vitest';
import { render, screen, waitFor } from '@testing-library/svelte';

vi.mock('$lib/shared/utils/i18n.svelte', () => ({
	i18nMsg: (_key: string, fallback: string) => fallback,
}));
vi.mock('$lib/features/bank-accounts/bank-accounts.api', () => ({
	listBankAccounts: vi.fn().mockRejectedValue(new Error('réseau')),
}));
vi.mock('$lib/features/journal-entries/journal-entries.api', () => ({
	fetchJournalEntries: vi.fn().mockResolvedValue({ items: [], total: 0, offset: 0, limit: 5 }),
}));
vi.mock('$lib/features/invoices/invoices.api', () => ({
	listDueDates: vi.fn().mockResolvedValue({
		items: [],
		total: 0,
		offset: 0,
		limit: 1,
		summary: { unpaidCount: 0, unpaidTotal: '0', overdueCount: 0, overdueTotal: '0' },
	}),
}));
vi.mock('$lib/features/reminders/reminders.api', () => ({
	listReminders: vi.fn().mockResolvedValue({ groups: [] }),
}));

import Page from './+page.svelte';

describe('page d’accueil — câblage des échecs', () => {
	it('un rejet de listBankAccounts affiche la tuile bancaire en échec', async () => {
		render(Page);
		await waitFor(() => expect(screen.getByTestId('homepage-bank-unavailable')).toBeTruthy());
	});
});
