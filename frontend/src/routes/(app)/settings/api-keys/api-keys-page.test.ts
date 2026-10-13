// Story 28-2 (#520, AC9 — B6) — premier test de la page des clés d'API : un
// rejet du chargement de la liste qui n'est pas une `ApiError` affiche le
// générique traduit, ni le message technique ni « [object Object] ».
//
// Le nom du fichier n'est pas `+page.test.ts` : SvelteKit réserve le préfixe `+`.

import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, fireEvent, waitFor } from '@testing-library/svelte';

// Revue P1 E-L5 : création et révocation pilotées pour les cas B7/B8.
const { listApiKeysMock, createApiKeyMock, revokeApiKeyMock } = vi.hoisted(() => ({
	listApiKeysMock: vi.fn(),
	createApiKeyMock: vi.fn(),
	revokeApiKeyMock: vi.fn(),
}));

vi.mock('$lib/shared/utils/i18n.svelte', () => ({
	i18nMsg: (_key: string, fallback: string) => fallback,
}));
vi.mock('svelte-sonner', () => ({ toast: { success: vi.fn(), error: vi.fn() } }));
vi.mock('$lib/features/api-keys/api-keys.api', () => ({
	listApiKeys: () => listApiKeysMock(),
	createApiKey: (p: unknown) => createApiKeyMock(p),
	revokeApiKey: (id: number, v: number) => revokeApiKeyMock(id, v),
}));

import { toast } from 'svelte-sonner';
import Page from './+page.svelte';

describe('page des clés d’API — échec du chargement (Story 28-2, B6)', () => {
	beforeEach(() => {
		vi.clearAllMocks();
		vi.spyOn(console, 'error').mockImplementation(() => {});
	});

	it('rejet non-ApiError → le générique', async () => {
		listApiKeysMock.mockRejectedValue(new Error('boom'));
		const { findByTestId } = render(Page);
		const erreur = await findByTestId('api-keys-load-error');
		expect(erreur.textContent).toBe('Erreur inattendue.');
		expect(erreur.textContent).not.toContain('boom');
		expect(erreur.textContent).not.toContain('[object Object]');
	});

	it('rejet ApiError → son message', async () => {
		listApiKeysMock.mockRejectedValue({ code: 'FORBIDDEN', message: 'Accès refusé.', status: 403 });
		const { findByTestId } = render(Page);
		expect((await findByTestId('api-keys-load-error')).textContent).toBe('Accès refusé.');
	});
});

// Story 28-2 (revue P1 E-L5 — B7, B8) — création et révocation rejetées hors ApiError.
describe('page des clés d’API — échec imprévu d’une écriture (Story 28-2, B7/B8)', () => {
	const cle = {
		id: 7,
		name: 'CI',
		scope: 'read',
		createdAt: '2026-01-01T00:00:00',
		lastUsedAt: null,
		revokedAt: null,
		expiresAt: null,
		version: 3,
	};

	beforeEach(() => {
		vi.clearAllMocks();
		vi.spyOn(console, 'error').mockImplementation(() => {});
		listApiKeysMock.mockResolvedValue([cle]);
	});

	it('B7 — création rejetée hors ApiError → le générique dans le formulaire', async () => {
		createApiKeyMock.mockRejectedValue(new Error('boom'));
		const r = render(Page);
		await fireEvent.click(await r.findByTestId('api-keys-create-button'));
		await fireEvent.input(await r.findByTestId('api-keys-name-input'), { target: { value: 'Export' } });
		await fireEvent.click(await r.findByTestId('api-keys-submit'));
		await waitFor(() => expect(createApiKeyMock).toHaveBeenCalledTimes(1));
		const erreur = await r.findByTestId('api-keys-form-error');
		expect(erreur.textContent?.trim()).toBe('Erreur inattendue.');
		expect(erreur.textContent).not.toContain('boom');
	});

	it('B8 — révocation rejetée hors ApiError → le générique en toast', async () => {
		revokeApiKeyMock.mockRejectedValue(new Error('boom'));
		const r = render(Page);
		await fireEvent.click(await r.findByTestId('api-keys-revoke-7'));
		await fireEvent.click(await r.findByTestId('api-keys-revoke-confirm-button'));
		await waitFor(() => expect(revokeApiKeyMock).toHaveBeenCalledWith(7, 3));
		await waitFor(() => expect(toast.error).toHaveBeenCalledWith('Erreur inattendue.'));
	});
});
