// Story v0172-session (0.17.3, CR #891, AC-8) — la section « Session » de
// Administration → Paramètres : lecture, saisie hors bornes refusée sans
// appel, enregistrement, conflit de version, durée appliquée aussitôt à la
// session de l'Administrateur.

import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, fireEvent } from '@testing-library/svelte';

const fetchMock = vi.fn();
const updateMock = vi.fn();
vi.mock('./settings.api', () => ({
	fetchSessionSettings: (...a: unknown[]) => fetchMock(...a),
	updateSessionSettings: (...a: unknown[]) => updateMock(...a),
}));
const setMinutesMock = vi.fn();
vi.mock('$lib/app/stores/session-idle.svelte', () => ({
	sessionIdle: { setMinutes: (...a: unknown[]) => setMinutesMock(...a) },
}));
const notifySuccessMock = vi.fn();
vi.mock('$lib/shared/utils/notify', () => ({
	notifySuccess: (...a: unknown[]) => notifySuccessMock(...a),
}));

import SessionIdleSection from './SessionIdleSection.svelte';

const ETAT = { sessionIdleMinutes: 60, version: 0, min: 1, max: 1440 };

async function monter() {
	const r = render(SessionIdleSection);
	const input = (await r.findByTestId('settings-session-idle-input')) as HTMLInputElement;
	return { ...r, input };
}

describe('SessionIdleSection (Story v0172-session, AC-8)', () => {
	beforeEach(() => {
		vi.clearAllMocks();
		fetchMock.mockResolvedValue(ETAT);
	});

	it('rend la valeur lue et les bornes', async () => {
		const { input, getByTestId } = await monter();
		expect(input.value).toBe('60');
		expect(getByTestId('settings-session-idle-help').textContent).toContain('De 1 à 1440 minutes');
	});

	for (const saisie of ['0', '1441', 'abc', '1.5', '']) {
		it(`« ${saisie} » ⇒ erreur de saisie, aucun PUT`, async () => {
			const { input, getByTestId, findByTestId } = await monter();
			await fireEvent.input(input, { target: { value: saisie } });
			await fireEvent.click(getByTestId('settings-session-save'));
			const err = await findByTestId('settings-session-idle-error');
			expect(err.textContent).toContain('de 1 à 1440 minutes');
			expect(updateMock).not.toHaveBeenCalled();
		});
	}

	it('45 ⇒ PUT, succès, durée appliquée à la session de l’Administrateur', async () => {
		updateMock.mockResolvedValue({ ...ETAT, sessionIdleMinutes: 45, version: 1 });
		const { input, getByTestId } = await monter();
		await fireEvent.input(input, { target: { value: '45' } });
		await fireEvent.click(getByTestId('settings-session-save'));
		await vi.waitFor(() => expect(notifySuccessMock).toHaveBeenCalled());
		expect(updateMock).toHaveBeenCalledWith({ sessionIdleMinutes: 45, version: 0 });
		expect(setMinutesMock).toHaveBeenCalledWith(45);
		expect(input.value).toBe('45');
	});

	it('409 ⇒ rechargement et message de conflit', async () => {
		updateMock.mockRejectedValue({ code: 'OPTIMISTIC_LOCK_CONFLICT', message: 'x', status: 409 });
		fetchMock
			.mockResolvedValueOnce(ETAT)
			.mockResolvedValueOnce({ ...ETAT, sessionIdleMinutes: 30, version: 2 });
		const { input, getByTestId, findByTestId } = await monter();
		await fireEvent.input(input, { target: { value: '45' } });
		await fireEvent.click(getByTestId('settings-session-save'));
		const err = await findByTestId('settings-session-idle-error');
		expect(err.textContent).toContain('Conflit de version');
		expect(fetchMock).toHaveBeenCalledTimes(2);
		expect(input.value).toBe('30');
		expect(setMinutesMock).not.toHaveBeenCalled();
	});
});
