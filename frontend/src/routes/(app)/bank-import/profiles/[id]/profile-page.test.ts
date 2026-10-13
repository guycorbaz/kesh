// Story 28-2 (#520, AC9 — A4) — la page d'un profil bancaire affiche le
// message du refus (« introuvable », traduit par le serveur), sans le préfixe
// « Profil introuvable : », qui était faux pour une panne réseau.
//
// Le nom du fichier n'est pas `+page.test.ts` : SvelteKit réserve le préfixe `+`.

import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render } from '@testing-library/svelte';

const { getMock } = vi.hoisted(() => ({ getMock: vi.fn() }));

vi.mock('$app/state', () => ({ page: { params: { id: '5' } } }));
vi.mock('$lib/features/bank-import/bank-profile.api', () => ({
	getBankProfile: (id: number) => getMock(id),
}));

import Page from './+page.svelte';

describe('page profil bancaire — erreur de chargement (Story 28-2, A4)', () => {
	beforeEach(() => {
		vi.clearAllMocks();
		vi.spyOn(console, 'error').mockImplementation(() => {});
	});

	it('ApiError 404 → son message, sans préfixe ni « [object Object] »', async () => {
		getMock.mockRejectedValue({ code: 'NOT_FOUND', message: 'Profil bancaire introuvable.', status: 404 });
		const { findByText, container } = render(Page);
		expect(await findByText('Profil bancaire introuvable.')).toBeTruthy();
		expect(getMock).toHaveBeenCalledWith(5);
		expect(container.textContent).not.toContain('[object Object]');
		expect(container.textContent).not.toContain('Profil introuvable :');
	});

	it('échec imprévu → le générique', async () => {
		getMock.mockRejectedValue(new Error('boom'));
		const { findByText } = render(Page);
		expect(await findByText('Erreur inattendue.')).toBeTruthy();
	});
});
