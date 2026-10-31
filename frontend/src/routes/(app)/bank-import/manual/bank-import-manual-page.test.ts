// Story v016-2b (#816), revue de code P1, B9 — la page `/bank-import/manual` : sa garde
// Comptable+ (`+page.ts`), l'échec du chargement des comptes dit, et ce qui suit un
// enregistrement (notification, retour à `/bank-import`).
//
// Le nom du fichier n'est pas `+page.test.ts` : SvelteKit réserve le préfixe `+`.

import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { render, cleanup } from '@testing-library/svelte';
import { tick } from 'svelte';

vi.mock('$app/environment', () => ({ browser: true }));
vi.mock('$app/navigation', () => ({ goto: vi.fn() }));
vi.mock('$lib/shared/utils/i18n.svelte', () => ({
	i18nMsg: (_k: string, fallback: string, args?: Record<string, string | number>) =>
		args ? fallback.replace(/\{\s*\$(\w+)\s*\}/g, (_, n) => String(args[n] ?? '')) : fallback,
}));
vi.mock('$lib/shared/utils/notify', () => ({ notifySuccess: vi.fn() }));
const role = vi.hoisted(() => ({ value: 'Comptable' as string | null }));
vi.mock('$lib/app/stores/auth.svelte', () => ({
	authState: {
		get currentUser() {
			return role.value === null ? null : { role: role.value, username: 'u', userId: '1' };
		},
	},
}));
const companyGet = vi.hoisted(() => vi.fn());
vi.mock('$lib/shared/utils/api-client', async (importOriginal) => ({
	...(await importOriginal<typeof import('$lib/shared/utils/api-client')>()),
	apiClient: { get: companyGet },
}));
// Le formulaire est remplacé par un témoin qui expose `onSaved`.
const saved = vi.hoisted(() => ({ onSaved: null as null | ((i: unknown) => void) }));
vi.mock('$lib/features/bank-import/ManualStatementForm.svelte', async () => {
	const Temoin = (await import('./TemoinFormulaire.test.helper.svelte')).default;
	return { default: Temoin };
});

import Page from './+page.svelte';
import { load } from './+page';
import { goto } from '$app/navigation';
import { notifySuccess } from '$lib/shared/utils/notify';

async function flush(): Promise<void> {
	for (let i = 0; i < 3; i++) {
		await new Promise((r) => setTimeout(r, 0));
		await tick();
	}
}

beforeEach(() => {
	companyGet.mockReset();
	vi.mocked(goto).mockClear();
	vi.mocked(notifySuccess).mockClear();
	role.value = 'Comptable';
	saved.onSaved = null;
	(globalThis as { __temoinSaved?: typeof saved }).__temoinSaved = saved;
});
afterEach(() => cleanup());

describe('/bank-import/manual — garde Comptable+', () => {
	it('Consultation et sans utilisateur ⇒ redirection vers l’accueil ; Admin et Comptable passent', () => {
		for (const r of ['Consultation', null]) {
			role.value = r;
			// Revue de code P2, P4 : la redirection exacte (302 vers `/`), non une exception quelconque.
			let leve: unknown = null;
			try {
				load();
			} catch (e) {
				leve = e;
			}
			expect(leve).toMatchObject({ status: 302, location: '/' });
		}
		for (const r of ['Admin', 'Comptable']) {
			role.value = r;
			expect(() => load()).not.toThrow();
		}
	});
});

describe('/bank-import/manual — page', () => {
	it('un échec du chargement des comptes est dit', async () => {
		companyGet.mockRejectedValue(new Error('réseau'));
		const { container } = render(Page);
		await flush();
		expect(container.querySelector('[data-testid="bank-import-manual-load-error"]')?.textContent).toContain(
			"Les comptes bancaires n'ont pas pu être chargés",
		);
	});

	it('les comptes chargés : aucun message d’échec', async () => {
		companyGet.mockResolvedValue({ bankAccounts: [] });
		const { container } = render(Page);
		await flush();
		expect(container.querySelector('[data-testid="bank-import-manual-load-error"]')).toBeNull();
	});

	it('après l’enregistrement : notification du nombre de transactions, retour à /bank-import', async () => {
		companyGet.mockResolvedValue({ bankAccounts: [] });
		render(Page);
		await flush();
		expect(saved.onSaved).not.toBeNull();
		saved.onSaved!({ transactionCount: 3 });
		expect(notifySuccess).toHaveBeenCalledWith('Relevé enregistré : 3 transaction(s).');
		expect(goto).toHaveBeenCalledWith('/bank-import');
	});
});
