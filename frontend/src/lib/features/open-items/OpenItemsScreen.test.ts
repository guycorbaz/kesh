// Story 15-1c-i (#518) — l'écran des postes ouverts, de bout en bout côté client :
// état d'URL (AC1), sélection et lettrage (AC4), propositions (AC5), groupe
// (AC6), rôles (AC10). Les appels HTTP sont doublés ; chaque test nomme la
// mutation qu'il attrape.
import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { render, cleanup, fireEvent, waitFor } from '@testing-library/svelte';

vi.mock('$lib/shared/utils/i18n.svelte', () => ({
	i18nMsg: (_k: string, fallback: string, args?: Record<string, string | number>) =>
		args ? fallback.replace(/\{\s*\$(\w+)\s*\}/g, (_, n) => String(args[n] ?? '')) : fallback,
}));

const fetchAccountsMock = vi.fn();
vi.mock('$lib/features/accounts/accounts.api', () => ({
	fetchAccounts: (a: boolean) => fetchAccountsMock(a),
}));

const api = {
	fetchOpenItems: vi.fn(),
	fetchProposals: vi.fn(),
	createLettering: vi.fn(),
	fetchLettering: vi.fn(),
	deleteLettering: vi.fn(),
};
vi.mock('./open-items.api', () => ({
	OPEN_ITEMS_PAGE_SIZE: 50,
	fetchOpenItems: (...a: unknown[]) => api.fetchOpenItems(...a),
	fetchProposals: (...a: unknown[]) => api.fetchProposals(...a),
	createLettering: (...a: unknown[]) => api.createLettering(...a),
	fetchLettering: (...a: unknown[]) => api.fetchLettering(...a),
	deleteLettering: (...a: unknown[]) => api.deleteLettering(...a),
}));

import OpenItemsScreen from './OpenItemsScreen.svelte';
import { authState } from '$lib/app/stores/auth.svelte';
import { todayLocal } from './open-items';
import {
	account,
	group,
	item,
	proposal,
	proposals,
	refusal,
	view,
} from './open-items.test.fixtures';

const BASE = 'http://localhost/open-items';

/** Monte l'écran ; `navigate` réécrit la prop `url`, comme `goto` le ferait de `page.url`. */
function mount(search: string) {
	// ⚠️ L'effet d'URL peut écrire dès le premier rendu, avant que `render` ne rende la
	// main : la réécriture est différée d'une microtâche, quand `rerender` est connu.
	let rerender: ((p: { url: URL }) => unknown) | null = null;
	const navigate = vi.fn((u: URL) => {
		queueMicrotask(() => void rerender?.({ url: new URL(u) }));
	});
	const r = render(OpenItemsScreen, { url: new URL(BASE + search), navigate });
	rerender = r.rerender;
	return { ...r, navigate };
}

function lastUrl(navigate: ReturnType<typeof vi.fn>): URL {
	return navigate.mock.calls.at(-1)![0] as URL;
}

beforeEach(() => {
	authState.login({ userId: '1', username: 'c', role: 'Comptable', expiresIn: 3600 });
	fetchAccountsMock.mockResolvedValue([
		account({ id: 7, number: '1091', name: 'Passage' }),
		account({ id: 8, number: '1092', name: 'Ancien', active: false }),
		account({ id: 9, number: '1020', name: 'Banque', letterable: false }),
	]);
	api.fetchOpenItems.mockResolvedValue(
		view({
			items: [
				item({ lineId: 1, debit: '100.0000', credit: '0.0000' }),
				item({ lineId: 2, debit: '0.0000', credit: '100.0000' }),
				item({ lineId: 3, debit: '0.0000', credit: '60.0000' }),
			],
		}),
	);
	api.fetchProposals.mockResolvedValue(proposals([proposal()]));
	api.createLettering.mockResolvedValue({ key: 28, code: 'AB', origin: 'manual', accountId: 7 });
	api.fetchLettering.mockResolvedValue(group());
	api.deleteLettering.mockResolvedValue(undefined);
});

afterEach(async () => {
	cleanup();
	vi.clearAllMocks();
	await authState.logout();
});

describe('AC1 — compte, date, URL (tests 1 et 2)', () => {
	it('asOf absent → la date LOCALE du jour est écrite dans l’URL (remplacement)', async () => {
		const { navigate } = mount('?accountId=7');
		await waitFor(() => expect(navigate).toHaveBeenCalled());
		const u = lastUrl(navigate);
		expect(u.searchParams.get('asOf')).toBe(todayLocal());
		expect(u.searchParams.get('accountId')).toBe('7');
		// La liste est demandée AVEC la date, jamais sans (C-15-1c-5).
		await waitFor(() => expect(api.fetchOpenItems).toHaveBeenCalledWith(7, todayLocal(), 0, 50));
	});

	it('asOf mal formé → date du jour, sans jamais appeler la vue avec la valeur fautive', async () => {
		const { navigate } = mount('?accountId=7&asOf=2026-02-30');
		await waitFor(() => expect(navigate).toHaveBeenCalled());
		expect(lastUrl(navigate).searchParams.get('asOf')).toBe(todayLocal());
		await waitFor(() => expect(api.fetchOpenItems).toHaveBeenCalled());
		for (const c of api.fetchOpenItems.mock.calls) expect(c[1]).not.toBe('2026-02-30');
	});

	it('sélecteur : seuls les comptes lettrables, archivés compris et marqués', async () => {
		const { getByTestId } = mount('?asOf=2026-03-31');
		await waitFor(() => expect(fetchAccountsMock).toHaveBeenCalledWith(true));
		await waitFor(() => expect(getByTestId('open-items-account').querySelectorAll('option').length).toBe(3));
		const opts = [...getByTestId('open-items-account').querySelectorAll('option')].map((o) => o.textContent ?? '');
		expect(opts.some((t) => t.includes('1091'))).toBe(true);
		expect(opts.find((t) => t.includes('1092'))).toContain('archivé');
		expect(opts.some((t) => t.includes('1020'))).toBe(false);
	});

	it('changer de compte écrit l’URL ; la liste repart en page 1, la sélection se vide', async () => {
		const r = mount('?accountId=7&asOf=2026-03-31');
		await waitFor(() => expect(r.getByTestId('open-item-select-1')).toBeTruthy());
		await fireEvent.click(r.getByTestId('open-item-select-1'));
		expect(r.getByTestId('open-items-selection-count').textContent).toContain(': 1');
		await fireEvent.change(r.getByTestId('open-items-account'), { target: { value: '8' } });
		await waitFor(() => expect(api.fetchOpenItems).toHaveBeenCalledWith(8, '2026-03-31', 0, 50));
		await waitFor(() =>
			expect(r.getByTestId('open-items-selection-count').textContent).toContain(': 0'),
		);
	});

	it('409 de la vue → le message du serveur à la place de la liste', async () => {
		api.fetchOpenItems.mockRejectedValue(
			refusal(409, 'LETTERING_ACCOUNT_NOT_LETTERABLE', 'Ce compte ne se lettre pas.'),
		);
		const r = mount('?accountId=9&asOf=2026-03-31');
		await waitFor(() => expect(r.getByTestId('open-items-refused').textContent).toContain('Ce compte ne se lettre pas.'));
		expect(r.queryByTestId('open-items-list')).toBeNull();
	});

	it('404 de la vue → « compte introuvable »', async () => {
		api.fetchOpenItems.mockRejectedValue(refusal(404, 'NOT_FOUND'));
		const r = mount('?accountId=99&asOf=2026-03-31');
		await waitFor(() => expect(r.getByTestId('open-items-not-found').textContent).toContain('Compte introuvable.'));
	});
});

describe('AC4 — sélection et lettrage manuel (tests 6 et 7)', () => {
	async function ready(search = '?accountId=7&asOf=2026-03-31') {
		const r = mount(search);
		await waitFor(() => expect(r.getByTestId('open-item-select-1')).toBeTruthy());
		return r;
	}

	it('sélection déséquilibrée : bouton inactif, l’écart dit', async () => {
		const r = await ready();
		await fireEvent.click(r.getByTestId('open-item-select-1'));
		await fireEvent.click(r.getByTestId('open-item-select-3'));
		expect((r.getByTestId('open-items-letter') as HTMLButtonElement).disabled).toBe(true);
		expect(r.getByTestId('open-items-letter-blocker').textContent).toContain('écart 40.00');
	});

	it('sélection conservée d’une page à l’autre ; le compteur dit combien hors de la page', async () => {
		api.fetchOpenItems.mockResolvedValueOnce(
			view({
				total: 60,
				items: [item({ lineId: 1 }), item({ lineId: 2, debit: '0.0000', credit: '100.0000' })],
			}),
		);
		const r = await ready();
		expect(api.fetchOpenItems).toHaveBeenCalledTimes(1);
		await fireEvent.click(r.getByTestId('open-item-select-1'));
		api.fetchOpenItems.mockResolvedValueOnce(
			view({ offset: 50, total: 60, items: [item({ lineId: 51, debit: '0.0000', credit: '100.0000' })] }),
		);
		// La page courante n'est pas dans l'URL : le bouton relit la vue à l'offset suivant.
		await fireEvent.click(r.getByTestId('open-items-next'));
		expect(api.fetchOpenItems).toHaveBeenLastCalledWith(7, '2026-03-31', 50, 50);
		await waitFor(() => expect(r.getByTestId('open-item-select-51')).toBeTruthy());
		await fireEvent.click(r.getByTestId('open-item-select-51'));
		expect(r.getByTestId('open-items-selection-count').textContent).toBe(
			'Lignes sélectionnées : 2 (dont 1 hors de cette page)',
		);
		expect((r.getByTestId('open-items-letter') as HTMLButtonElement).disabled).toBe(false);
		await fireEvent.click(r.getByTestId('open-items-letter'));
		expect(api.createLettering).toHaveBeenCalledWith([1, 51]);
	});

	it('changer de date efface la sélection — et ne recharge pas les propositions', async () => {
		const r = await ready();
		await fireEvent.click(r.getByTestId('open-item-select-1'));
		const before = api.fetchProposals.mock.calls.length;
		await fireEvent.change(r.getByTestId('open-items-as-of'), { target: { value: '2026-02-28' } });
		await waitFor(() => expect(api.fetchOpenItems).toHaveBeenCalledWith(7, '2026-02-28', 0, 50));
		await waitFor(() => expect(r.getByTestId('open-items-selection-count').textContent).toContain(': 0'));
		expect(api.fetchProposals.mock.calls.length).toBe(before);
	});

	it('succès : code annoncé en lien, liste et propositions relues, sélection vide', async () => {
		const r = await ready();
		await fireEvent.click(r.getByTestId('open-item-select-1'));
		await fireEvent.click(r.getByTestId('open-item-select-2'));
		const lists = api.fetchOpenItems.mock.calls.length;
		const props = api.fetchProposals.mock.calls.length;
		await fireEvent.click(r.getByTestId('open-items-letter'));
		expect(api.createLettering).toHaveBeenCalledWith([1, 2]);
		await waitFor(() => expect(r.getByTestId('lettering-code-link-AB')).toBeTruthy());
		expect(r.getByTestId('lettering-code-link-AB').getAttribute('href')).toContain('group=AB');
		expect(api.fetchOpenItems.mock.calls.length).toBe(lists + 1);
		expect(api.fetchProposals.mock.calls.length).toBe(props + 1);
		await waitFor(() => expect(r.getByTestId('open-items-selection-count').textContent).toContain(': 0'));
	});

	const CODES_409 = [
		'LETTERING_ACCOUNTS_DIFFER',
		'LETTERING_ACCOUNT_NOT_LETTERABLE',
		'LETTERING_ALL_LINES_IN_CLOSED_PERIODS',
		'LETTERING_LINE_OWNED_BY_DOCUMENT',
		'LETTERING_LINE_ALREADY_LETTERED',
		// La somme AFFICHÉE est nulle (montants retenus) ; le serveur a d'autres montants.
		'LETTERING_UNBALANCED',
		// « Réessayez » — et pourtant la sélection est effacée, sans exception (C-15-1c-24).
		'LETTERING_CONCURRENT_CHANGE',
	];
	for (const code of CODES_409) {
		it(`409 ${code} : son message, liste et propositions relues, sélection vidée`, async () => {
			api.createLettering.mockRejectedValueOnce(refusal(409, code));
			const r = await ready();
			await fireEvent.click(r.getByTestId('open-item-select-1'));
			await fireEvent.click(r.getByTestId('open-item-select-2'));
			const lists = api.fetchOpenItems.mock.calls.length;
			const props = api.fetchProposals.mock.calls.length;
			await fireEvent.click(r.getByTestId('open-items-letter'));
			await waitFor(() => expect(r.getByTestId('open-items-message').textContent).toContain(`message de ${code}`));
			expect(api.fetchOpenItems.mock.calls.length).toBe(lists + 1);
			expect(api.fetchProposals.mock.calls.length).toBe(props + 1);
			await waitFor(() => expect(r.getByTestId('open-items-selection-count').textContent).toContain(': 0'));
		});
	}

	it('404 : le texte de l’écran (« une ligne sélectionnée n’existe plus »), et rechargement', async () => {
		api.createLettering.mockRejectedValueOnce(refusal(404, 'NOT_FOUND', 'Ressource introuvable'));
		const r = await ready();
		await fireEvent.click(r.getByTestId('open-item-select-1'));
		await fireEvent.click(r.getByTestId('open-item-select-2'));
		const props = api.fetchProposals.mock.calls.length;
		await fireEvent.click(r.getByTestId('open-items-letter'));
		await waitFor(() =>
			expect(r.getByTestId('open-items-message').textContent).toContain(
				"Une ligne sélectionnée n'existe plus",
			),
		);
		expect(api.fetchProposals.mock.calls.length).toBe(props + 1);
		await waitFor(() => expect(r.getByTestId('open-items-selection-count').textContent).toContain(': 0'));
	});

	it('400 de forme : son message, sélection GARDÉE, rien de relu', async () => {
		api.createLettering.mockRejectedValueOnce(refusal(400, 'LETTERING_TOO_FEW_LINES'));
		const r = await ready();
		await fireEvent.click(r.getByTestId('open-item-select-1'));
		await fireEvent.click(r.getByTestId('open-item-select-2'));
		const lists = api.fetchOpenItems.mock.calls.length;
		await fireEvent.click(r.getByTestId('open-items-letter'));
		await waitFor(() => expect(r.getByTestId('open-items-message').textContent).toContain('LETTERING_TOO_FEW_LINES'));
		expect(api.fetchOpenItems.mock.calls.length).toBe(lists);
		expect(r.getByTestId('open-items-selection-count').textContent).toContain(': 2');
	});
});

describe('AC5 — les propositions (test 8)', () => {
	it('aucun POST au montage ni au rendu', async () => {
		const r = mount('?accountId=7&asOf=2026-03-31');
		await waitFor(() => expect(r.getByTestId('proposal-1-2')).toBeTruthy());
		expect(api.createLettering).not.toHaveBeenCalled();
	});

	it('422 : son message dans le panneau ; la liste et le lettrage manuel restent', async () => {
		api.fetchProposals.mockRejectedValue(
			refusal(422, 'LETTERING_PROPOSALS_TOO_MANY_LINES', 'Trop de lignes ouvertes'),
		);
		const r = mount('?accountId=7&asOf=2026-03-31');
		await waitFor(() => expect(r.getByTestId('open-items-proposals-error').textContent).toContain('Trop de lignes ouvertes'));
		expect(r.getByTestId('open-item-select-1')).toBeTruthy();
		expect(r.getByTestId('open-items-letter')).toBeTruthy();
	});

	it('autre échec : un message d’échec dans le panneau, la liste reste', async () => {
		api.fetchProposals.mockRejectedValue(new Error('réseau'));
		const r = mount('?accountId=7&asOf=2026-03-31');
		await waitFor(() =>
			expect(r.getByTestId('open-items-proposals-error').textContent).toContain(
				"n'ont pas pu être chargés",
			),
		);
		expect(r.getByTestId('open-items-list')).toBeTruthy();
	});

	it('« Lettrer » envoie les deux lineId ; un refus 409 recharge liste et propositions', async () => {
		api.createLettering.mockRejectedValueOnce(refusal(409, 'LETTERING_LINE_ALREADY_LETTERED'));
		const r = mount('?accountId=7&asOf=2026-03-31');
		await waitFor(() => expect(r.getByTestId('proposal-letter-1-2')).toBeTruthy());
		const lists = api.fetchOpenItems.mock.calls.length;
		const props = api.fetchProposals.mock.calls.length;
		await fireEvent.click(r.getByTestId('proposal-letter-1-2'));
		expect(api.createLettering).toHaveBeenCalledWith([1, 2]);
		await waitFor(() => expect(api.fetchProposals.mock.calls.length).toBe(props + 1));
		expect(api.fetchOpenItems.mock.calls.length).toBe(lists + 1);
	});

	it('paire dont une ligne a disparu (404) : le texte de l’écran, et rechargement', async () => {
		api.createLettering.mockRejectedValueOnce(refusal(404, 'NOT_FOUND'));
		const r = mount('?accountId=7&asOf=2026-03-31');
		await waitFor(() => expect(r.getByTestId('proposal-letter-1-2')).toBeTruthy());
		const props = api.fetchProposals.mock.calls.length;
		await fireEvent.click(r.getByTestId('proposal-letter-1-2'));
		await waitFor(() => expect(r.getByTestId('open-items-message').textContent).toContain("n'existe plus"));
		expect(api.fetchProposals.mock.calls.length).toBe(props + 1);
	});
});

describe('AC6 — le groupe (test 9)', () => {
	it('?group= seul : le groupe s’affiche sans liste ; la liste vient à la demande', async () => {
		const r = mount('?asOf=2026-03-31&group=AA');
		await waitFor(() => expect(r.getByTestId('lettering-group-origin')).toBeTruthy());
		expect(api.fetchLettering).toHaveBeenCalledWith('AA');
		expect(api.fetchOpenItems).not.toHaveBeenCalled();
		await fireEvent.click(r.getByTestId('open-items-show-group-account'));
		expect(lastUrl(r.navigate).searchParams.get('accountId')).toBe('7');
	});

	it('le champ « Code » ouvre le groupe saisi (?group= écrit dans l’URL)', async () => {
		const r = mount('?accountId=7&asOf=2026-03-31');
		await waitFor(() => expect(r.getByTestId('open-items-code')).toBeTruthy());
		await fireEvent.input(r.getByTestId('open-items-code'), { target: { value: ' AC ' } });
		await fireEvent.click(r.getByTestId('open-items-code-open'));
		const u = lastUrl(r.navigate);
		expect(u.searchParams.get('group')).toBe('AC');
		expect(u.searchParams.get('accountId')).toBe('7');
		await waitFor(() => expect(api.fetchLettering).toHaveBeenCalledWith('AC'));
	});

	it('204 : « délettré », liste et propositions relues, `group` retiré de l’URL', async () => {
		const r = mount('?accountId=7&asOf=2026-03-31&group=AA');
		await waitFor(() => expect(r.getByTestId('lettering-group-dissolve')).toBeTruthy());
		const props = api.fetchProposals.mock.calls.length;
		await fireEvent.click(r.getByTestId('lettering-group-dissolve'));
		expect(api.deleteLettering).toHaveBeenCalledWith('AA');
		await waitFor(() => expect(r.getByTestId('open-items-message').textContent).toContain('Le groupe AA est délettré.'));
		expect(lastUrl(r.navigate).searchParams.has('group')).toBe(false);
		expect(api.fetchProposals.mock.calls.length).toBeGreaterThan(props);
		await waitFor(() => expect(r.queryByTestId('lettering-group-panel')).toBeNull());
	});

	it('refus au clic 409 : son message, puis groupe, liste et propositions relus', async () => {
		api.deleteLettering.mockRejectedValueOnce(refusal(409, 'LETTERING_CONCURRENT_CHANGE'));
		const r = mount('?accountId=7&asOf=2026-03-31&group=AA');
		await waitFor(() => expect(r.getByTestId('lettering-group-dissolve')).toBeTruthy());
		const groups = api.fetchLettering.mock.calls.length;
		const lists = api.fetchOpenItems.mock.calls.length;
		await fireEvent.click(r.getByTestId('lettering-group-dissolve'));
		await waitFor(() => expect(r.getByTestId('open-items-message').textContent).toContain('message de LETTERING_CONCURRENT_CHANGE'));
		expect(api.fetchLettering.mock.calls.length).toBe(groups + 1);
		expect(api.fetchOpenItems.mock.calls.length).toBe(lists + 1);
	});

	it('refus au clic 404 : « aucun groupe ne porte ce code », et rechargement', async () => {
		api.deleteLettering.mockRejectedValueOnce(refusal(404, 'NOT_FOUND'));
		const r = mount('?accountId=7&asOf=2026-03-31&group=AA');
		await waitFor(() => expect(r.getByTestId('lettering-group-dissolve')).toBeTruthy());
		const groups = api.fetchLettering.mock.calls.length;
		await fireEvent.click(r.getByTestId('lettering-group-dissolve'));
		await waitFor(() => expect(r.getByTestId('open-items-message').textContent).toContain('Aucun groupe ne porte ce code.'));
		expect(api.fetchLettering.mock.calls.length).toBe(groups + 1);
	});

	it('404 à l’ouverture : « aucun groupe ne porte ce code »', async () => {
		api.fetchLettering.mockRejectedValue(refusal(404, 'NOT_FOUND'));
		const r = mount('?asOf=2026-03-31&group=ZZ');
		await waitFor(() => expect(r.getByTestId('lettering-group-not-found')).toBeTruthy());
	});

	it('vue en 409 et groupe affiché ensemble — et délettrable (C104)', async () => {
		api.fetchOpenItems.mockRejectedValue(refusal(409, 'LETTERING_ACCOUNT_NOT_LETTERABLE', 'Ce compte ne se lettre pas.'));
		const r = mount('?accountId=7&asOf=2026-03-31&group=AA');
		await waitFor(() => expect(r.getByTestId('open-items-refused')).toBeTruthy());
		await waitFor(() => expect(r.getByTestId('lettering-group-dissolve')).toBeTruthy());
	});
});

describe('AC7, AC10 — bandeau et rôle Consultation (tests 10 et 11)', () => {
	it('le bandeau de frontière est là, atteint par son data-testid', async () => {
		const r = mount('?asOf=2026-03-31');
		expect(r.getByTestId('open-items-boundary').textContent).toContain('Réconciliation');
	});

	it('Consultation : liste visible, ni case, ni « Lettrer », ni « Délettrer »', async () => {
		await authState.logout();
		authState.login({ userId: '2', username: 'v', role: 'Consultation', expiresIn: 3600 });
		const r = mount('?accountId=7&asOf=2026-03-31&group=AA');
		await waitFor(() => expect(r.getByTestId('open-item-row-1')).toBeTruthy());
		await waitFor(() => expect(r.getByTestId('proposal-1-2')).toBeTruthy());
		await waitFor(() => expect(r.getByTestId('lettering-group-origin')).toBeTruthy());
		expect(r.container.querySelector('input[type="checkbox"]')).toBeNull();
		expect(r.queryByTestId('open-items-letter')).toBeNull();
		expect(r.container.querySelector('[data-testid^="proposal-letter-"]')).toBeNull();
		expect(r.queryByTestId('lettering-group-dissolve')).toBeNull();
	});
});
