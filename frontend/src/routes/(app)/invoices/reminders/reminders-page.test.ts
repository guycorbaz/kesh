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
// Story v0174-correctifs-3 (#889) : l'instant du dernier rappel se règle par test.
const last = vi.hoisted(() => ({ at: '2026-04-15T08:00:00' }));
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
						lastReminderAt: last.at,
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
import { pinTimeZone } from '$lib/shared/utils/pin-time-zone.test-helper';

afterEach(() => {
	cleanup();
	vi.clearAllMocks();
	last.at = '2026-04-15T08:00:00';
});

// Story v0174-correctifs-3 (#889, AC-3.8, g) — le dernier rappel envoyé à 23 h 30 UTC le
// 4 novembre se lit à sa date suisse, le 5.
describe('relances — l’instant du dernier rappel à l’heure suisse (#889)', () => {
	pinTimeZone('Europe/Zurich');
	it('« dernier le 05.11.2026 »', async () => {
		last.at = '2026-11-04T23:30:00';
		const { findByTestId } = render(Page);
		expect((await findByTestId('reminder-last-sent')).textContent?.trim()).toBe(
			'dernier le 05.11.2026',
		);
	});
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
