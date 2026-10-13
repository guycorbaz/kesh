// Story 28-2 (#520, AC9 — B6) — premier test de la page des clés d'API : un
// rejet du chargement de la liste qui n'est pas une `ApiError` affiche le
// générique traduit, ni le message technique ni « [object Object] ».
//
// Le nom du fichier n'est pas `+page.test.ts` : SvelteKit réserve le préfixe `+`.

import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render } from '@testing-library/svelte';

const { listApiKeysMock } = vi.hoisted(() => ({ listApiKeysMock: vi.fn() }));

vi.mock('$lib/shared/utils/i18n.svelte', () => ({
	i18nMsg: (_key: string, fallback: string) => fallback,
}));
vi.mock('svelte-sonner', () => ({ toast: { success: vi.fn(), error: vi.fn() } }));
vi.mock('$lib/features/api-keys/api-keys.api', () => ({
	listApiKeys: () => listApiKeysMock(),
	createApiKey: vi.fn(),
	revokeApiKey: vi.fn(),
}));

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
