// Story v0172-session (0.17.3, #876, AC-9 et AC-6) — la page de connexion :
// le message de fin de session pour inactivité, et la connexion qui transmet
// les champs d'inactivité à `authState.login`. Le nom du fichier n'est pas
// `+page.test.ts` : SvelteKit réserve le préfixe `+`.
//
// ⚠️ Ce test établit le texte FRANÇAIS, non une traduction : sur `/login`, les
// messages i18n ne sont pas chargés (endpoint authentifié).

import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, fireEvent } from '@testing-library/svelte';
import { readable } from 'svelte/store';

let url = new URL('http://localhost/login');
vi.mock('$app/stores', () => ({
	page: { subscribe: (fn: (v: unknown) => void) => readable({ url }).subscribe(fn) },
}));
vi.mock('$app/navigation', () => ({ goto: vi.fn() }));
const postMock = vi.fn();
vi.mock('$lib/shared/utils/api-client', () => ({
	apiClient: { post: (...a: unknown[]) => postMock(...a) },
	isApiError: () => false,
}));
const loginMock = vi.fn();
vi.mock('$lib/app/stores/auth.svelte', () => ({
	authState: { login: (...a: unknown[]) => loginMock(...a) },
}));

import Page from './+page.svelte';

function at(search: string) {
	url = new URL(`http://localhost/login${search}`);
}

describe('page de connexion — fin de session (Story v0172-session, AC-9)', () => {
	beforeEach(() => vi.clearAllMocks());

	it('?reason=idle&minutes=60 ⇒ « Session fermée après 60 minutes d’inactivité. »', () => {
		at('?reason=idle&minutes=60');
		const { getByTestId, queryByText } = render(Page);
		expect(getByTestId('login-session-idle').textContent?.trim()).toBe(
			"Session fermée après 60 minutes d'inactivité.",
		);
		expect(queryByText('Votre session a expiré. Veuillez vous reconnecter.')).toBeNull();
	});

	it('?reason=idle&minutes=x ⇒ le message de session expirée', () => {
		at('?reason=idle&minutes=x');
		const { queryByTestId, getByText } = render(Page);
		expect(queryByTestId('login-session-idle')).toBeNull();
		expect(getByText('Votre session a expiré. Veuillez vous reconnecter.')).toBeTruthy();
	});

	// Revue de code P1 (N-B4/N-A9/N-E6) — bornes 1 à 1440, et le singulier.
	it('?reason=idle&minutes=1 ⇒ « 1 minute », au singulier', () => {
		at('?reason=idle&minutes=1');
		const { getByTestId } = render(Page);
		expect(getByTestId('login-session-idle').textContent?.trim()).toBe(
			"Session fermée après 1 minute d'inactivité.",
		);
	});
	it('?reason=idle&minutes=1440 ⇒ la borne haute s’affiche', () => {
		at('?reason=idle&minutes=1440');
		const { getByTestId } = render(Page);
		expect(getByTestId('login-session-idle').textContent?.trim()).toBe(
			"Session fermée après 1440 minutes d'inactivité.",
		);
	});
	for (const hors of ['0', '00', '1441', '99999999999999999999', '1.5', '']) {
		it(`?reason=idle&minutes=${hors} ⇒ hors bornes : le message de session expirée`, () => {
			at(`?reason=idle&minutes=${hors}`);
			const { queryByTestId, getByText } = render(Page);
			expect(queryByTestId('login-session-idle')).toBeNull();
			expect(getByText('Votre session a expiré. Veuillez vous reconnecter.')).toBeTruthy();
		});
	}

	it('?reason=session_expired ⇒ inchangé', () => {
		at('?reason=session_expired');
		const { queryByTestId, getByText } = render(Page);
		expect(queryByTestId('login-session-idle')).toBeNull();
		expect(getByText('Votre session a expiré. Veuillez vous reconnecter.')).toBeTruthy();
	});

	it('la connexion transmet idleTimeoutMinutes et idleExpiresIn à authState.login', async () => {
		at('');
		postMock.mockResolvedValue({
			accessToken: 'a',
			refreshToken: 'r',
			expiresIn: 900,
			userId: 3,
			username: 'alice',
			role: 'Admin',
			idleTimeoutMinutes: 60,
			idleExpiresIn: 3600,
		});
		const { container } = render(Page);
		await fireEvent.input(container.querySelector('#username') as HTMLInputElement, {
			target: { value: 'alice' },
		});
		await fireEvent.input(container.querySelector('#password') as HTMLInputElement, {
			target: { value: 'secret-password' },
		});
		await fireEvent.submit(container.querySelector('form') as HTMLFormElement);
		await vi.waitFor(() => expect(loginMock).toHaveBeenCalled());
		expect(loginMock).toHaveBeenCalledWith({
			userId: '3',
			username: 'alice',
			role: 'Admin',
			expiresIn: 900,
			idleTimeoutMinutes: 60,
			idleExpiresIn: 3600,
		});
	});
});
