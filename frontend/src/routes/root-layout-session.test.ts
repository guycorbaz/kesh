// Story v0172-session (0.17.3, #876, AC-6 19, C-v0172s-50) — l'abonnement du
// layout racine au suivi d'activité : à l'IDENTIFIANT de l'utilisateur, non à
// l'objet que `authState.login()` remplace à chaque renouvellement.

import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { render } from '@testing-library/svelte';
import { createRawSnippet, flushSync, tick } from 'svelte';

const startMock = vi.fn();
const stopMock = vi.fn();
const stayActiveMock = vi.fn();
// Story v0174-session-2 (#895, AC-3) : le simulacre expose l'état de
// l'avertissement, posé AVANT chaque rendu (non réactif : un rendu par état).
// `undefined` vaut un simulacre qui n'expose pas le champ.
const idleMock = vi.hoisted(() => ({ warning: undefined as number | null | undefined }));
vi.mock('$lib/app/stores/session-idle.svelte', () => ({
	sessionIdle: {
		start: (...a: unknown[]) => startMock(...a),
		stop: (...a: unknown[]) => stopMock(...a),
		stayActive: (...a: unknown[]) => stayActiveMock(...a),
		get warningRemainingMs() {
			return idleMock.warning;
		},
	},
}));
vi.mock('$lib/shared/utils/i18n.svelte', () => ({
	i18nMsg: (_k: string, f: string, args?: Record<string, string | number>) =>
		args ? f.replace(/\{\s*\$(\w+)\s*\}/g, (_, k) => String(args[k] ?? '')) : f,
	loadI18nMessages: vi.fn(),
}));

import Layout from './+layout.svelte';
import { authState } from '$lib/app/stores/auth.svelte';

const children = createRawSnippet(() => ({ render: () => '<p>contenu</p>' }));
const P = (userId: string, idle = 60) => ({
	userId,
	username: 'u',
	role: 'Admin',
	expiresIn: 900,
	idleTimeoutMinutes: idle,
	idleExpiresIn: idle * 60,
});

describe('layout racine — abonnement du suivi d’activité (Story v0172-session)', () => {
	beforeEach(() => {
		vi.clearAllMocks();
		// `Toaster` (svelte-sonner) lit `matchMedia`, absent de jsdom.
		vi.stubGlobal(
			'matchMedia',
			vi.fn().mockReturnValue({
				matches: false,
				addEventListener: () => {},
				removeEventListener: () => {},
			}),
		);
		vi.stubGlobal(
			'fetch',
			vi.fn().mockResolvedValue({ ok: true, json: () => Promise.resolve({ db: true }) }),
		);
		authState.clearSession({ broadcast: false });
	});
	afterEach(() => {
		authState.clearSession({ broadcast: false });
		vi.unstubAllGlobals();
	});

	it('démarre à la connexion avec les champs de transit, ne redémarre pas pour le même identifiant', async () => {
		render(Layout, { props: { children } });
		await tick();
		expect(startMock).not.toHaveBeenCalled();
		authState.login(P('1'), { broadcast: false });
		flushSync();
		expect(startMock).toHaveBeenCalledTimes(1);
		expect(startMock).toHaveBeenCalledWith({ idleTimeoutMinutes: 60, idleExpiresIn: 3600 });
		// Un renouvellement : `login()` remplace l'objet et les champs de transit.
		authState.login(P('1', 30), { broadcast: false });
		flushSync();
		expect(startMock).toHaveBeenCalledTimes(1);
		expect(stopMock).not.toHaveBeenCalled();
	});

	it('un autre identifiant ⇒ arrêt puis démarrage ; clearSession ⇒ arrêt', async () => {
		render(Layout, { props: { children } });
		authState.login(P('1'), { broadcast: false });
		flushSync();
		authState.login(P('2'), { broadcast: false });
		flushSync();
		expect(stopMock).toHaveBeenCalledTimes(1);
		expect(startMock).toHaveBeenCalledTimes(2);
		authState.clearSession({ broadcast: false });
		flushSync();
		expect(stopMock).toHaveBeenCalledTimes(2);
		expect(startMock).toHaveBeenCalledTimes(2);
	});

	// Story v0174-session-2 (#895, AC-3, C-v0174s-20) — l'avertissement est
	// rendu si et seulement si `warningRemainingMs` est un nombre.
	describe('l’avertissement avant la fermeture', () => {
		afterEach(() => {
			idleMock.warning = undefined;
		});
		for (const [nom, valeur] of [
			['champ absent du simulacre', undefined],
			['null', null],
		] as const) {
			it(`${nom} ⇒ rien n’est rendu`, async () => {
				idleMock.warning = valeur;
				const { queryByTestId } = render(Layout, { props: { children } });
				await tick();
				expect(queryByTestId('session-idle-warning')).toBeNull();
			});
		}
		it('un nombre ⇒ l’avis est rendu, son bouton appelle stayActive', async () => {
			idleMock.warning = 42_000;
			const { getByTestId } = render(Layout, { props: { children } });
			await tick();
			expect(getByTestId('session-idle-warning-countdown').textContent?.trim()).toBe(
				'Fermeture dans 0:42.',
			);
			getByTestId('session-idle-warning-stay').click();
			expect(stayActiveMock).toHaveBeenCalledTimes(1);
		});
	});
});
