// Story v0171 (#879) — l'historique des rappels d'une facture : la date d'envoi et la
// mention « Annulé le … » s'affichent en forme suisse (`15.04.2026`, jamais l'ISO).
// Mutation M7 : retirer `formatSwissDate` de `cancelledAt` seul rougit le second test
// seul ; de `sentAt` seul, le premier seul.

import { describe, it, expect, vi, afterEach } from 'vitest';
import { render, cleanup } from '@testing-library/svelte';
import type { ReminderResponse } from './reminders.types';

vi.mock('$lib/shared/utils/i18n.svelte', () => ({
	i18nMsg: (_k: string, fallback: string, args?: Record<string, string | number>) =>
		args ? fallback.replace(/\{\s*\$(\w+)\s*\}/g, (_, n) => String(args[n] ?? '')) : fallback,
}));

import ReminderHistory from './ReminderHistory.svelte';
import { pinTimeZone } from '$lib/shared/utils/pin-time-zone.test-helper';

function rappel(over: Partial<ReminderResponse> = {}): ReminderResponse {
	return {
		id: 1,
		levelNumber: 1,
		feeAmount: '0.00',
		sentAt: '2026-04-15T08:00:00',
		channel: 'email',
		sentTo: 'client@example.invalid',
		subject: 'Rappel',
		body: 'Rappel',
		note: null,
		cancelledAt: null,
		...over,
	};
}

afterEach(() => cleanup());

describe('historique des rappels — dates (v0171)', () => {
	it('date d’envoi `15.04.2026` (mutation M7 : site `sentAt`)', () => {
		const { getByTestId } = render(ReminderHistory, { reminders: [rappel()] });
		const row = getByTestId('reminder-history-row');
		expect(row.textContent).toContain('15.04.2026');
		expect(row.textContent).not.toContain('2026-04-15');
	});

	it('« Annulé le 20.04.2026 » (mutation M7 : site `cancelledAt`)', () => {
		const { getByTestId } = render(ReminderHistory, {
			reminders: [rappel({ cancelledAt: '2026-04-20T10:00:00' })],
		});
		const cancelled = getByTestId('reminder-history-cancelled-at');
		expect(cancelled.textContent?.trim()).toBe('Annulé le 20.04.2026');
		expect(getByTestId('reminder-history').textContent).not.toContain('2026-04-20');
	});
});

// Story v0174-correctifs-3 (#889, AC-3.8, d et e) — un instant UTC du 4 novembre à 23 h 30
// s'affiche à sa date suisse, le 5.
describe('historique des rappels — instants à l’heure suisse (#889)', () => {
	pinTimeZone('Europe/Zurich');
	it('envoi et annulation à 23 h 30 UTC : `05.11.2026`', () => {
		const { getByTestId } = render(ReminderHistory, {
			reminders: [
				rappel({ sentAt: '2026-11-04T23:30:00', cancelledAt: '2026-11-04T23:30:00' }),
			],
		});
		expect(getByTestId('reminder-history-row').textContent).toContain('05.11.2026');
		expect(getByTestId('reminder-history-cancelled-at').textContent?.trim()).toBe(
			'Annulé le 05.11.2026',
		);
	});
});
