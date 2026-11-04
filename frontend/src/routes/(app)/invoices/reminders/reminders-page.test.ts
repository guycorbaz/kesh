// Story v0171 (#879) — l'écran des relances : l'échéance d'une facture et la date du
// dernier rappel (« dernier le … ») s'affichent en forme suisse (`31.03.2026`, jamais
// `2026-03-31`). Mutation M5 : retirer `formatSwissDate` d'un site rougit le test qui le
// nomme.
//
// Le nom du fichier n'est pas `+page.test.ts` : SvelteKit réserve le préfixe `+`.

import { describe, it, expect, vi, afterEach } from 'vitest';
import { render, cleanup } from '@testing-library/svelte';

vi.mock('$app/navigation', () => ({ goto: vi.fn() }));
vi.mock('$lib/shared/utils/i18n.svelte', () => ({
	i18nMsg: (_k: string, fallback: string, args?: Record<string, string | number>) =>
		args ? fallback.replace(/\{\s*\$(\w+)\s*\}/g, (_, n) => String(args[n] ?? '')) : fallback,
}));
vi.mock('$lib/shared/utils/notify', () => ({ notifyError: vi.fn(), notifySuccess: vi.fn() }));
vi.mock('$lib/app/stores/auth.svelte', () => ({
	authState: { currentUser: { role: 'Comptable', username: 'c', userId: '1' } },
}));
vi.mock('$lib/features/dunning/dunning.api', () => ({
	listDunningLevels: vi.fn(async () => []),
}));
vi.mock('$lib/features/reminders/reminders.api', () => ({
	listReminders: vi.fn(async () => ({
		groups: [
			{
				contactId: 1,
				contactName: 'Client Démo SA',
				hasEmail: true,
				invoices: [
					{
						invoiceId: 5,
						invoiceNumber: 'F-2026-005',
						dueDate: '2026-03-31',
						currentLevel: 1,
						nextLevel: 2,
						terminal: false,
						lastReminderAt: '2026-04-15T08:00:00',
					},
				],
			},
		],
	})),
	getReminderPreview: vi.fn(),
	sendReminder: vi.fn(),
	sendReminderBatch: vi.fn(),
	recordManualReminder: vi.fn(),
}));

import Page from './+page.svelte';

afterEach(() => {
	cleanup();
	vi.clearAllMocks();
});

describe('relances — dates (v0171)', () => {
	it('échéance `31.03.2026` (mutation M5 : site `inv.dueDate`)', async () => {
		const { findByTestId, container } = render(Page);
		expect((await findByTestId('reminder-due-date')).textContent?.trim()).toBe('31.03.2026');
		expect(container.textContent).not.toContain('2026-03-31');
	});

	it('« dernier le 15.04.2026 » (mutation M5 : site `lastReminderAt`)', async () => {
		const { findByTestId, container } = render(Page);
		expect((await findByTestId('reminder-last-sent')).textContent?.trim()).toBe(
			'dernier le 15.04.2026',
		);
		expect(container.textContent).not.toContain('2026-04-15');
	});
});
