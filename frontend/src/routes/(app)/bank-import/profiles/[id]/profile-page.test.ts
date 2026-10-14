// Story 28-2 (#520, AC9 — A4) — la page d'un profil bancaire affiche le
// message du refus (« introuvable », traduit par le serveur), sans le préfixe
// « Profil introuvable : », qui était faux pour une panne réseau.
//
// Le nom du fichier n'est pas `+page.test.ts` : SvelteKit réserve le préfixe `+`.
//
// Story 29-9 (#655, AC6) — `page` est une doublure RÉACTIVE (`profile-page-state.svelte.ts`) :
// une navigation réassigne `fakePage.params` (objet neuf), comme SvelteKit.

import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, fireEvent } from '@testing-library/svelte';
import { tick } from 'svelte';
import type { BankProfile } from '$lib/features/bank-import/bank-profile.types';

const { getMock, updateMock } = vi.hoisted(() => ({ getMock: vi.fn(), updateMock: vi.fn() }));

vi.mock('$app/state', async () => ({
	page: (await import('./profile-page-state.svelte')).fakePage,
}));
vi.mock('$lib/features/bank-import/bank-profile.api', () => ({
	getBankProfile: (id: number) => getMock(id),
	// Story 29-9 (V17) — le formulaire importe les deux ; seul `updateBankProfile` est appelé.
	updateBankProfile: (id: number, req: unknown) => updateMock(id, req),
	createBankProfile: vi.fn(),
}));

import { fakePage } from './profile-page-state.svelte';
import Page from './+page.svelte';

beforeEach(() => {
	// Module partagé entre les tests : sans cette remise, l'ordre des tests déciderait.
	fakePage.params = { id: '5' };
});

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

// Story 29-9 (#655, AC6) — d'un profil à l'autre sans quitter la route : l'erreur du précédent
// ne masque pas le suivant, une réponse ancienne n'écrase rien, et une navigation au MÊME
// identifiant ne relit rien ni ne démonte le formulaire.
describe('page profil bancaire — navigation d’un profil à l’autre (Story 29-9, #655)', () => {
	beforeEach(() => {
		vi.clearAllMocks();
		vi.spyOn(console, 'error').mockImplementation(() => {});
	});

	function profil(id: number): BankProfile {
		return {
			id,
			bankName: `Banque ${id}`,
			filenamePattern: null,
			columnMapping: { date: 0, amount: 1 },
			dateFormat: '%Y-%m-%d',
			decimalSeparator: '.',
			fieldSeparator: ';',
			encoding: null,
			headerRowCount: 1,
			debitSign: 'positive',
			createdAt: '2026-10-01T00:00:00',
			updatedAt: '2026-10-01T00:00:00',
		};
	}

	function deferred<T>() {
		let resolve!: (v: T) => void;
		let reject!: (e: unknown) => void;
		const promise = new Promise<T>((res, rej) => {
			resolve = res;
			reject = rej;
		});
		return { promise, resolve, reject };
	}

	const panne = (message: string) => ({ code: 'INTERNAL_ERROR', message, status: 500 });

	async function naviguer(id: string): Promise<void> {
		fakePage.params = { id };
		await tick();
	}

	function champBanque(container: HTMLElement): HTMLInputElement | null {
		return container.querySelector<HTMLInputElement>('[data-testid="bank-name-input"]');
	}

	it('V8 — 5 en échec, puis 6 en succès ⇒ le message disparaît, le formulaire est rendu (M9)', async () => {
		getMock.mockImplementation((id: number) =>
			id === 5 ? Promise.reject(panne('Panne du 5.')) : Promise.resolve(profil(id)),
		);
		const { findByText, queryByText, findByTestId } = render(Page);
		expect(await findByText('Panne du 5.')).toBeTruthy();
		await naviguer('6');
		expect(((await findByTestId('bank-name-input')) as HTMLInputElement).value).toBe('Banque 6');
		expect(queryByText('Panne du 5.')).toBeNull();
	});

	it('V9 — 5 lent, 6 rapide ; 6 réussit, puis 5 échoue ⇒ aucun message, le profil 6 (M10)', async () => {
		const lent = deferred<BankProfile>();
		getMock.mockImplementation((id: number) => (id === 5 ? lent.promise : Promise.resolve(profil(6))));
		const { findByTestId, queryByText, container } = render(Page);
		await naviguer('6');
		expect(((await findByTestId('bank-name-input')) as HTMLInputElement).value).toBe('Banque 6');
		lent.reject(panne('Panne du 5.'));
		await new Promise((r) => setTimeout(r, 0));
		expect(queryByText('Panne du 5.')).toBeNull();
		expect(champBanque(container)?.value).toBe('Banque 6');
	});

	it('V13 — navigation au même identifiant ⇒ aucune relecture, jamais « Chargement… », la saisie intacte (M15)', async () => {
		getMock.mockImplementation((id: number) => Promise.resolve(profil(id)));
		const { findByTestId, queryByText } = render(Page);
		const champ = (await findByTestId('bank-name-input')) as HTMLInputElement;
		await fireEvent.input(champ, { target: { value: 'Saisie en cours' } });
		await naviguer('5');
		expect(queryByText('Chargement…')).toBeNull();
		await new Promise((r) => setTimeout(r, 0));
		expect(getMock).toHaveBeenCalledTimes(1);
		expect(((await findByTestId('bank-name-input')) as HTMLInputElement).value).toBe('Saisie en cours');
	});

	it('V14 — identifiant changé ⇒ « Chargement… » sans formulaire pendant la lecture, puis les champs du profil 6 (M13)', async () => {
		const six = deferred<BankProfile>();
		getMock.mockImplementation((id: number) => (id === 5 ? Promise.resolve(profil(5)) : six.promise));
		const { findByTestId, queryByText, container } = render(Page);
		expect(((await findByTestId('bank-name-input')) as HTMLInputElement).value).toBe('Banque 5');
		await naviguer('6');
		expect(queryByText('Chargement…')).not.toBeNull();
		expect(champBanque(container)).toBeNull();
		six.resolve(profil(6));
		expect(((await findByTestId('bank-name-input')) as HTMLInputElement).value).toBe('Banque 6');
	});

	it('V15 — 5 réussit pendant que 6 est en vol ⇒ toujours « Chargement… », jamais « Banque 5 » ; puis « Banque 6 » (M16)', async () => {
		const cinq = deferred<BankProfile>();
		const six = deferred<BankProfile>();
		getMock.mockImplementation((id: number) => (id === 5 ? cinq.promise : six.promise));
		const { findByTestId, queryByText, container } = render(Page);
		await naviguer('6');
		cinq.resolve(profil(5));
		await new Promise((r) => setTimeout(r, 0));
		expect(queryByText('Chargement…')).not.toBeNull();
		expect(champBanque(container)).toBeNull();
		six.resolve(profil(6));
		expect(((await findByTestId('bank-name-input')) as HTMLInputElement).value).toBe('Banque 6');
	});

	it('V16 — 5 lent, puis un identifiant invalide ; 5 échoue ensuite ⇒ « ID invalide » reste (M17)', async () => {
		const cinq = deferred<BankProfile>();
		getMock.mockImplementation(() => cinq.promise);
		const { findByText, queryByText } = render(Page);
		await naviguer('abc');
		expect(await findByText('ID invalide')).toBeTruthy();
		cinq.reject(panne('Panne du 5.'));
		await new Promise((r) => setTimeout(r, 0));
		expect(queryByText('ID invalide')).not.toBeNull();
		expect(queryByText('Panne du 5.')).toBeNull();
	});

	it('V17 — une sauvegarde du 5 qui finit après la navigation vers 6 n’écrit pas le profil affiché (M19)', async () => {
		getMock.mockImplementation((id: number) => Promise.resolve(profil(id)));
		const sauvegarde5 = deferred<BankProfile>();
		updateMock.mockReturnValueOnce(sauvegarde5.promise).mockResolvedValue(profil(6));
		const { findByTestId, container } = render(Page);
		await findByTestId('bank-name-input');
		await fireEvent.submit(await findByTestId('bank-import-profile-form'));
		expect(updateMock).toHaveBeenCalledTimes(1);
		expect(updateMock.mock.calls[0][0]).toBe(5);

		await naviguer('6');
		expect(((await findByTestId('bank-name-input')) as HTMLInputElement).value).toBe('Banque 6');
		sauvegarde5.resolve(profil(5));
		await new Promise((r) => setTimeout(r, 0));
		expect(champBanque(container)?.value).toBe('Banque 6');

		await fireEvent.submit(await findByTestId('bank-import-profile-form'));
		expect(updateMock).toHaveBeenCalledTimes(2);
		expect(updateMock.mock.calls[1][0]).toBe(6);
	});
});
