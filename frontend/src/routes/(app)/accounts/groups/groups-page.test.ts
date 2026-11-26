// Story 46-5b (AC-3 à AC-5, AC-8) — la page « Créer des groupes » : Confirmer
// inactif puis actif, Écarter et Revenir, refus du serveur affiché sans perdre
// l'état. Harnais calqué sur `accounts-page.test.ts` (mocks hoistés, `isApiError`
// réel : une erreur simulée porte `code` ET `status`).
import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { render, screen, fireEvent, waitFor } from '@testing-library/svelte';
import { authState } from '$lib/app/stores/auth.svelte';
import type { AccountResponse } from '$lib/features/accounts/accounts.types';
import { toast } from 'svelte-sonner';

vi.mock('$app/environment', () => ({ browser: true }));
const gotoMock = vi.fn();
vi.mock('$app/navigation', () => ({ goto: (url: string) => gotoMock(url) }));
vi.mock('$lib/shared/utils/i18n.svelte', () => ({
	i18nMsg: (_key: string, fallback: string) => fallback,
}));
vi.mock('svelte-sonner', () => ({ toast: { success: vi.fn(), error: vi.fn() } }));

const fetchAccountsMock = vi.fn<(includeArchived?: boolean) => Promise<AccountResponse[]>>();
const applyGroupsMock = vi.fn();
vi.mock('$lib/features/accounts/accounts.api', () => ({
	fetchAccounts: (includeArchived?: boolean) => fetchAccountsMock(includeArchived),
	applyGroups: (req: unknown) => applyGroupsMock(req),
}));

import Page from './+page.svelte';

function account(id: number, number: string, name: string, parentId: number | null): AccountResponse {
	return {
		id,
		companyId: 1,
		number,
		name,
		accountType: 'Asset',
		parentId,
		active: true,
		role: null,
		postable: true,
		letterable: false,
		currencyLocked: true,
		version: 2,
		createdAt: '2026-01-01T00:00:00',
		updatedAt: '2026-01-01T00:00:00',
	};
}

const PLAN = [
	{ ...account(1, '10', 'Actifs circulants', null), currencyLocked: false },
	account(2, '1030', 'Banque A CHF', 1),
	account(3, '1031', 'Banque A EUR', 1),
];

/** Deux banques sous 10, et 1032 d'une autre banque (revue P1 de la 46-5b, M1, M2, L6). */
const PLAN_WIDE = [
	...PLAN,
	account(4, '1032', 'Banque B CHF', 1),
	account(5, '1040', 'Dépôt 1', 1),
	account(6, '1041', 'Dépôt 2', 1),
];

const cardOf = (key: string) => screen.getByTestId(`account-groups-proposal-${key}`);
const click = (card: HTMLElement, testId: string) => fireEvent.click(card.querySelector(`[data-testid="${testId}"]`)!);

async function renderWithProposal() {
	render(Page);
	await fireEvent.click(await screen.findByTestId('account-groups-check-1030'));
	await fireEvent.click(screen.getByTestId('account-groups-check-1031'));
	return screen.findByTestId('account-groups-proposal-digit:103');
}

describe('page Créer des groupes', () => {
	beforeEach(() => {
		authState.login({ userId: '1', username: 'admin', role: 'Admin', expiresIn: 3600 });
		fetchAccountsMock.mockResolvedValue(PLAN);
	});
	afterEach(async () => {
		vi.clearAllMocks();
		await authState.logout();
	});

	it('lit le plan archivés compris ; Confirmer inactif tant que rien n’est accepté, puis actif', async () => {
		const card = await renderWithProposal();
		expect(fetchAccountsMock).toHaveBeenCalledWith(true);
		const confirm = screen.getByTestId('account-groups-confirm') as HTMLButtonElement;
		expect(confirm.disabled).toBe(true);
		expect(card.getAttribute('data-state')).toBe('pending');
		await fireEvent.click(card.querySelector('[data-testid="account-groups-accept"]')!);
		expect(card.getAttribute('data-state')).toBe('accepted');
		expect(confirm.disabled).toBe(false);
	});

	it('Écarter puis Revenir : la proposition sort puis revient en attente', async () => {
		const card = await renderWithProposal();
		await fireEvent.click(card.querySelector('[data-testid="account-groups-reject"]')!);
		expect(card.getAttribute('data-state')).toBe('rejected');
		expect((screen.getByTestId('account-groups-confirm') as HTMLButtonElement).disabled).toBe(true);
		await fireEvent.click(card.querySelector('[data-testid="account-groups-reset"]')!);
		expect(card.getAttribute('data-state')).toBe('pending');
	});

	it('confirme : la demande part, puis retour à l’écran du plan', async () => {
		applyGroupsMock.mockResolvedValue({ created: [PLAN[0]], reused: [], moved: [PLAN[1], PLAN[2]] });
		const card = await renderWithProposal();
		await fireEvent.click(card.querySelector('[data-testid="account-groups-accept"]')!);
		await fireEvent.click(screen.getByTestId('account-groups-confirm'));
		await waitFor(() => expect(gotoMock).toHaveBeenCalledWith('/accounts'));
		expect(applyGroupsMock).toHaveBeenCalledWith({
			groups: [
				{
					kind: 'create',
					number: '103',
					name: 'Banque A',
					parentId: 1,
					accounts: [
						{ id: 2, version: 2 },
						{ id: 3, version: 2 },
					],
				},
			],
		});
		expect(toast.success).toHaveBeenCalled();
	});

	it('un refus du serveur s’affiche avec la proposition et le compte, sans perdre l’état', async () => {
		applyGroupsMock.mockRejectedValue({
			code: 'OPTIMISTIC_LOCK_CONFLICT',
			message: 'Conflit',
			status: 409,
			details: { groupStep: { groupIndex: 0, accountId: 3 } },
		});
		const card = await renderWithProposal();
		await fireEvent.click(card.querySelector('[data-testid="account-groups-accept"]')!);
		await fireEvent.click(screen.getByTestId('account-groups-confirm'));
		const alert = await screen.findByTestId('account-groups-refusal');
		expect(screen.getByTestId('account-groups-refusal-message').textContent).toContain(
			'Les données ont été modifiées',
		);
		expect(screen.getByTestId('account-groups-refusal-step')).toBeTruthy();
		expect(alert.textContent).toContain('compte');
		expect(gotoMock).not.toHaveBeenCalled();
		// L'état reste : la proposition, acceptée, désignée ; la sélection cochée.
		expect(card.getAttribute('data-state')).toBe('accepted');
		expect(card.className).toContain('ring-red-500');
		expect((screen.getByTestId('account-groups-check-1030') as HTMLInputElement).checked).toBe(true);
		// Recharger relit le plan sans rien perdre.
		await fireEvent.click(screen.getByTestId('account-groups-reload'));
		await waitFor(() => expect(fetchAccountsMock).toHaveBeenCalledTimes(2));
		expect(screen.getByTestId('account-groups-proposal-digit:103').getAttribute('data-state')).toBe('accepted');
	});

	it('ACCOUNT_GROUP_INVALID : le message du serveur, tel quel', async () => {
		applyGroupsMock.mockRejectedValue({
			code: 'ACCOUNT_GROUP_INVALID',
			message: 'Proposition 1 : les comptes à ranger sont de types différents.',
			status: 400,
			details: { groupIndex: 0, reason: 'mixedTypes', accountId: 3 },
		});
		const card = await renderWithProposal();
		await fireEvent.click(card.querySelector('[data-testid="account-groups-accept"]')!);
		await fireEvent.click(screen.getByTestId('account-groups-confirm'));
		expect((await screen.findByTestId('account-groups-refusal-message')).textContent).toBe(
			'Proposition 1 : les comptes à ranger sont de types différents.',
		);
		// Recharger le plan s'offre sur tout refus (revue P1 de la 46-5b, M2).
		expect(screen.getByTestId('account-groups-reload')).toBeTruthy();
	});

	it('une acceptée dont les comptes changent repasse en attente, nom recalculé (M1)', async () => {
		fetchAccountsMock.mockResolvedValue(PLAN_WIDE);
		const card = await renderWithProposal();
		await click(card, 'account-groups-accept');
		expect(card.getAttribute('data-state')).toBe('accepted');
		await fireEvent.click(screen.getByTestId('account-groups-check-1032'));
		const wider = cardOf('digit:103');
		expect(wider.getAttribute('data-state')).toBe('pending');
		expect((wider.querySelector('[data-testid="account-groups-name"]') as HTMLInputElement).value).toBe('Banque');
		expect((screen.getByTestId('account-groups-confirm') as HTMLButtonElement).disabled).toBe(true);
	});

	it('une écartée le reste après un changement de mode, puis après Recharger le plan offert sur ACCOUNT_PARENT_HAS_ENTRIES (L6, M2)', async () => {
		fetchAccountsMock.mockResolvedValue(PLAN_WIDE);
		applyGroupsMock.mockRejectedValue({
			code: 'ACCOUNT_PARENT_HAS_ENTRIES',
			message: 'Le compte parent porte des écritures.',
			status: 409,
			details: { groupStep: { groupIndex: 0 } },
		});
		await renderWithProposal();
		await fireEvent.click(screen.getByTestId('account-groups-check-1040'));
		await fireEvent.click(screen.getByTestId('account-groups-check-1041'));
		await click(cardOf('digit:104'), 'account-groups-reject');
		expect(cardOf('digit:104').getAttribute('data-state')).toBe('rejected');
		await fireEvent.click(screen.getByTestId('account-groups-mode-one'));
		expect(screen.queryByTestId('account-groups-proposal-digit:104')).toBeNull();
		await fireEvent.click(screen.getByTestId('account-groups-mode-digit'));
		expect(cardOf('digit:104').getAttribute('data-state')).toBe('rejected');
		await click(cardOf('digit:103'), 'account-groups-accept');
		await fireEvent.click(screen.getByTestId('account-groups-confirm'));
		expect((await screen.findByTestId('account-groups-refusal-message')).textContent).toBe(
			'Le compte parent porte des écritures.',
		);
		const sent = applyGroupsMock.mock.calls[0][0] as { groups: { number: string }[] };
		expect(sent.groups.map((g) => g.number)).toEqual(['103']);
		await fireEvent.click(screen.getByTestId('account-groups-reload'));
		await waitFor(() => expect(fetchAccountsMock).toHaveBeenCalledTimes(2));
		expect(cardOf('digit:104').getAttribute('data-state')).toBe('rejected');
		expect(cardOf('digit:103').getAttribute('data-state')).toBe('accepted');
		expect(screen.queryByTestId('account-groups-refusal')).toBeNull();
	});
});
