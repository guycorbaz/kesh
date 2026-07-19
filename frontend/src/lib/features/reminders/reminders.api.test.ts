// Story 21-6b — tests Vitest du client API reminders (liste, aperçu, envoi, lot, manuel).
import { beforeEach, describe, expect, it, vi } from 'vitest';

vi.mock('$lib/shared/utils/api-client', () => ({
	apiClient: {
		get: vi.fn(),
		post: vi.fn(),
		put: vi.fn(),
		patch: vi.fn(),
		delete: vi.fn(),
	},
}));

import { apiClient } from '$lib/shared/utils/api-client';
import {
	listReminders,
	getReminderPreview,
	sendReminder,
	sendReminderBatch,
	recordManualReminder,
	listReminderHistory,
	pauseDunning,
	resumeDunning,
} from './reminders.api';

type Mock = ReturnType<typeof vi.fn>;

describe('reminders.api', () => {
	beforeEach(() => {
		vi.clearAllMocks();
	});

	it('listReminders GET sur le bon path', async () => {
		(apiClient.get as Mock).mockResolvedValue({ groups: [] });
		await listReminders();
		expect(apiClient.get).toHaveBeenCalledWith('/api/v1/dunning/reminders');
	});

	it('getReminderPreview GET avec le niveau en query', async () => {
		(apiClient.get as Mock).mockResolvedValue({
			to: 'd@example.ch',
			language: 'FR',
			level: 2,
			subject: '2e rappel',
			body: 'Bonjour',
		});
		await getReminderPreview(42, 2);
		expect(apiClient.get).toHaveBeenCalledWith('/api/v1/invoices/42/reminder-preview?level=2');
	});

	it('sendReminder POST le payload sans champ `to`', async () => {
		(apiClient.post as Mock).mockResolvedValue({ id: 1 });
		await sendReminder(42, { levelNumber: 2, subject: 'Objet', body: 'Corps' });
		expect(apiClient.post).toHaveBeenCalledWith('/api/v1/invoices/42/reminders/send', {
			levelNumber: 2,
			subject: 'Objet',
			body: 'Corps',
		});
		const payload = (apiClient.post as Mock).mock.calls[0][1] as Record<string, unknown>;
		expect('to' in payload).toBe(false);
	});

	it('sendReminderBatch POST { invoiceIds } sur le bon path', async () => {
		(apiClient.post as Mock).mockResolvedValue({ accepted: [], failed: [] });
		await sendReminderBatch([1, 2, 3]);
		expect(apiClient.post).toHaveBeenCalledWith('/api/v1/dunning/reminders/send-batch', {
			invoiceIds: [1, 2, 3],
		});
	});

	it('recordManualReminder POST avec un sentAt daté-heure (jamais date nue, bug #249)', async () => {
		(apiClient.post as Mock).mockResolvedValue({ id: 7 });
		await recordManualReminder(42, {
			levelNumber: 1,
			sentAt: '2026-07-17T12:00:00',
			note: 'recommandé',
		});
		expect(apiClient.post).toHaveBeenCalledWith('/api/v1/invoices/42/reminders/manual', {
			levelNumber: 1,
			sentAt: '2026-07-17T12:00:00',
			note: 'recommandé',
		});
		const payload = (apiClient.post as Mock).mock.calls[0][1] as { sentAt: string };
		// Garde-fou : le contrat backend NaiveDateTime exige le composant horaire.
		expect(payload.sentAt).toMatch(/T\d{2}:\d{2}:\d{2}$/);
	});

	// --- Story 21-6c : historique + suspension/reprise ---

	it('listReminderHistory GET sur le bon path', async () => {
		(apiClient.get as Mock).mockResolvedValue([]);
		await listReminderHistory(42);
		expect(apiClient.get).toHaveBeenCalledWith('/api/v1/invoices/42/reminders');
	});

	it('pauseDunning PUT { version, note } sur le bon path', async () => {
		(apiClient.put as Mock).mockResolvedValue({
			invoiceId: 42,
			dunningPausedAt: '2026-07-19T12:00:00',
			dunningPausedNote: 'litige',
			version: 4,
		});
		await pauseDunning(42, { version: 3, note: 'litige' });
		expect(apiClient.put).toHaveBeenCalledWith('/api/v1/invoices/42/dunning-pause', {
			version: 3,
			note: 'litige',
		});
	});

	it('pauseDunning transmet une note nulle sans la transformer', async () => {
		(apiClient.put as Mock).mockResolvedValue({
			invoiceId: 42,
			dunningPausedAt: '2026-07-19T12:00:00',
			dunningPausedNote: null,
			version: 4,
		});
		await pauseDunning(42, { version: 3, note: null });
		expect(apiClient.put).toHaveBeenCalledWith('/api/v1/invoices/42/dunning-pause', {
			version: 3,
			note: null,
		});
	});

	it('resumeDunning PUT { version } sur le bon path', async () => {
		(apiClient.put as Mock).mockResolvedValue({
			invoiceId: 42,
			dunningPausedAt: null,
			dunningPausedNote: null,
			version: 5,
		});
		await resumeDunning(42, { version: 4 });
		expect(apiClient.put).toHaveBeenCalledWith('/api/v1/invoices/42/dunning-resume', {
			version: 4,
		});
	});
});
