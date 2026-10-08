// Story 14-2 — tests Vitest pour la page Exercices comptables.
//
// Couvre AC-E/F/J côté frontend :
// - bouton Réouvrir visible seulement si Closed ET Admin (pas Comptable) ;
// - désactivé + tooltip si un exercice postérieur est clos (garde LIFO client) ;
// - modal de confirmation désactivé tant que le motif est vide, activé sinon ;
// - le submit appelle `reopenFiscalYear(id, { motif })` ;
// - (P4-F2) le dialogue de CLÔTURE ne ment plus sur l'irréversibilité absolue
//   (ne rend plus « définitivement » ni « ne pourra plus être enregistré »).
// - (Story 15-12a, #543) la clôture dans l'ordre : bouton « Clôturer » désactivé
//   sous un exercice antérieur ouvert, et refus serveur EARLIER_FISCAL_YEAR_OPEN
//   traité avant la branche « déjà clôturé ».
//
// Mocks (hoistés AVANT l'import du composant) : module API + i18n (fallback) +
// notify. `authState` est le vrai store, piloté via `login`.

import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { render, screen, fireEvent, waitFor } from '@testing-library/svelte';
import { authState } from '$lib/app/stores/auth.svelte';
import type { FiscalYearResponse } from '$lib/features/fiscal-years/fiscal-years.types';

vi.mock('$app/environment', () => ({ browser: true }));

// i18nMsg renvoie le fallback (déterministe, couvre la copie fallback svelte).
vi.mock('$lib/shared/utils/i18n.svelte', () => ({
	i18nMsg: (_key: string, fallback: string) => fallback,
}));

vi.mock('$lib/shared/utils/notify', () => ({
	notifySuccess: vi.fn(),
	notifyError: vi.fn(),
}));

const listFiscalYearsMock = vi.fn<() => Promise<FiscalYearResponse[]>>();
const reopenFiscalYearMock = vi.fn();
const closeFiscalYearMock = vi.fn();
vi.mock('$lib/features/fiscal-years/fiscal-years.api', () => ({
	listFiscalYears: () => listFiscalYearsMock(),
	reopenFiscalYear: (id: number, req: { motif: string }) => reopenFiscalYearMock(id, req),
	closeFiscalYear: (id: number) => closeFiscalYearMock(id),
	createFiscalYear: vi.fn(),
	updateFiscalYear: vi.fn(),
}));

import Page from './+page.svelte';
import { notifyError } from '$lib/shared/utils/notify';

function fy(overrides: Partial<FiscalYearResponse>): FiscalYearResponse {
	return {
		id: 1,
		companyId: 1,
		name: 'Exercice 2026',
		startDate: '2026-01-01',
		endDate: '2026-12-31',
		status: 'Closed',
		createdAt: '2026-01-01T00:00:00',
		updatedAt: '2026-01-01T00:00:00',
		...overrides,
	};
}

beforeEach(() => {
	listFiscalYearsMock.mockReset();
	reopenFiscalYearMock.mockReset();
	reopenFiscalYearMock.mockResolvedValue(fy({ status: 'Open' }));
	closeFiscalYearMock.mockReset();
	vi.mocked(notifyError).mockReset();
});

afterEach(async () => {
	await authState.logout();
});

describe('bouton Réouvrir — visibilité RBAC', () => {
	it('Admin voit le bouton Réouvrir sur un exercice clôturé (sans postérieur clos)', async () => {
		authState.login({ userId: '1', username: 'admin', role: 'Admin', expiresIn: 3600 });
		listFiscalYearsMock.mockResolvedValue([fy({ id: 1, status: 'Closed' })]);

		render(Page);

		const btn = await screen.findByTestId('fiscal-year-reopen-1');
		expect(btn).toBeTruthy();
		expect((btn as HTMLButtonElement).disabled).toBe(false);
	});

	it('Comptable ne voit PAS le bouton Réouvrir', async () => {
		authState.login({ userId: '2', username: 'comptable', role: 'Comptable', expiresIn: 3600 });
		listFiscalYearsMock.mockResolvedValue([fy({ id: 1, status: 'Closed' })]);

		render(Page);

		// La ligne existe (Comptable a canMutate) mais pas le bouton Réouvrir.
		await screen.findByTestId('fiscal-year-row-1');
		expect(screen.queryByTestId('fiscal-year-reopen-1')).toBeNull();
	});
});

describe('garde LIFO client', () => {
	it('désactive le bouton + tooltip nommant le plus proche postérieur clos', async () => {
		authState.login({ userId: '1', username: 'admin', role: 'Admin', expiresIn: 3600 });
		listFiscalYearsMock.mockResolvedValue([
			fy({ id: 1, name: 'Exercice 2025', startDate: '2025-01-01', endDate: '2025-12-31', status: 'Closed' }),
			fy({ id: 2, name: 'Exercice 2026', startDate: '2026-01-01', endDate: '2026-12-31', status: 'Closed' }),
		]);

		render(Page);

		const btn = (await screen.findByTestId('fiscal-year-reopen-1')) as HTMLButtonElement;
		expect(btn.disabled).toBe(true);
		expect(btn.getAttribute('title') ?? '').toContain('Exercice 2026');
	});
});

describe('modal de réouverture — motif obligatoire + submit', () => {
	it('confirmation désactivée si motif vide, activée sinon, et appelle reopenFiscalYear', async () => {
		authState.login({ userId: '1', username: 'admin', role: 'Admin', expiresIn: 3600 });
		listFiscalYearsMock.mockResolvedValue([fy({ id: 7, status: 'Closed' })]);

		render(Page);

		const openBtn = await screen.findByTestId('fiscal-year-reopen-7');
		await fireEvent.click(openBtn);

		const confirm = (await screen.findByTestId('fiscal-year-reopen-confirm')) as HTMLButtonElement;
		// Motif vide → désactivé.
		expect(confirm.disabled).toBe(true);

		const motif = (await screen.findByTestId('fiscal-year-reopen-motif')) as HTMLTextAreaElement;
		await fireEvent.input(motif, { target: { value: 'Correction TVA' } });

		await waitFor(() => expect(confirm.disabled).toBe(false));

		await fireEvent.click(confirm);

		await waitFor(() => expect(reopenFiscalYearMock).toHaveBeenCalledWith(7, { motif: 'Correction TVA' }));
	});

	it("n'est PAS désactivée pour 500 caractères significatifs suivis d'espaces (F1 — miroir trim serveur)", async () => {
		authState.login({ userId: '1', username: 'admin', role: 'Admin', expiresIn: 3600 });
		listFiscalYearsMock.mockResolvedValue([fy({ id: 7, status: 'Closed' })]);

		render(Page);

		const openBtn = await screen.findByTestId('fiscal-year-reopen-7');
		await fireEvent.click(openBtn);

		const confirm = (await screen.findByTestId('fiscal-year-reopen-confirm')) as HTMLButtonElement;
		const motif = (await screen.findByTestId('fiscal-year-reopen-motif')) as HTMLTextAreaElement;

		// 500 caractères significatifs + espaces de fin : le serveur trim puis
		// compte (500 ≤ 500 → accepté). Le client doit faire pareil et NE PAS
		// bloquer. Avant le fix F1 (`.length` brut = 503 > 500), le bouton était
		// désactivé à tort.
		await fireEvent.input(motif, { target: { value: 'x'.repeat(500) + '   ' } });

		await waitFor(() => expect(confirm.disabled).toBe(false));
	});
});

describe('dialogue de clôture — ne ment plus sur l’irréversibilité (P4-F2)', () => {
	it('ne rend plus « définitivement » ni « ne pourra plus être enregistré »', async () => {
		authState.login({ userId: '1', username: 'admin', role: 'Admin', expiresIn: 3600 });
		listFiscalYearsMock.mockResolvedValue([fy({ id: 3, status: 'Open' })]);

		render(Page);

		// Ouvre le dialogue de clôture via le bouton Lock (aria-label).
		const closeBtn = await screen.findByLabelText(/Clôturer Exercice 2026/);
		await fireEvent.click(closeBtn);

		// Le corps + l'action du dialogue sont désormais montés (portail).
		await waitFor(() => {
			const body = document.body.textContent ?? '';
			expect(body).not.toContain('définitivement');
			expect(body).not.toContain('ne pourra plus être enregistré');
		});
	});
});

describe('clôture dans l’ordre (Story 15-12a, #543)', () => {
	it('désactive « Clôturer » sous un antérieur ouvert, tooltip nommant le PLUS ANCIEN', async () => {
		authState.login({ userId: '1', username: 'admin', role: 'Admin', expiresIn: 3600 });
		listFiscalYearsMock.mockResolvedValue([
			fy({ id: 3, name: 'Exercice 2026', startDate: '2026-01-01', endDate: '2026-12-31', status: 'Open' }),
			fy({ id: 2, name: 'Exercice 2025', startDate: '2025-01-01', endDate: '2025-12-31', status: 'Open' }),
			fy({ id: 1, name: 'Exercice 2024', startDate: '2024-01-01', endDate: '2024-12-31', status: 'Open' }),
		]);

		render(Page);

		const btn2026 = (await screen.findByTestId('fiscal-year-close-3')) as HTMLButtonElement;
		expect(btn2026.disabled).toBe(true);
		const title = btn2026.getAttribute('title') ?? '';
		expect(title).toContain('« Exercice 2024 »');
		expect(title).not.toContain('Exercice 2025');
		// Le plus ancien se clôture.
		const btn2024 = (await screen.findByTestId('fiscal-year-close-1')) as HTMLButtonElement;
		expect(btn2024.disabled).toBe(false);
		expect(btn2024.getAttribute('title')).toBeNull();
	});

	it('un antérieur CLOS ne bloque pas', async () => {
		authState.login({ userId: '1', username: 'admin', role: 'Admin', expiresIn: 3600 });
		listFiscalYearsMock.mockResolvedValue([
			fy({ id: 2, name: 'Exercice 2026', startDate: '2026-01-01', endDate: '2026-12-31', status: 'Open' }),
			fy({ id: 1, name: 'Exercice 2025', startDate: '2025-01-01', endDate: '2025-12-31', status: 'Closed' }),
		]);

		render(Page);

		const btn = (await screen.findByTestId('fiscal-year-close-2')) as HTMLButtonElement;
		expect(btn.disabled).toBe(false);
	});

	// ⛔ Tue la mutation (viii) : si la branche ILLEGAL_STATE_TRANSITION
	// attrape ce code, l'écran dit « Cet exercice est déjà clôturé » — faux.
	it('refus serveur EARLIER_FISCAL_YEAR_OPEN : message du serveur, pas « déjà clôturé »', async () => {
		authState.login({ userId: '1', username: 'admin', role: 'Admin', expiresIn: 3600 });
		// L'écran croit 2025 clos (liste périmée) : le bouton est actif, le
		// serveur refuse.
		listFiscalYearsMock.mockResolvedValue([
			fy({ id: 2, name: 'Exercice 2026', startDate: '2026-01-01', endDate: '2026-12-31', status: 'Open' }),
			fy({ id: 1, name: 'Exercice 2025', startDate: '2025-01-01', endDate: '2025-12-31', status: 'Closed' }),
		]);
		const serveur =
			'Clôturez d’abord l’exercice « Exercice 2025 », plus ancien et encore ouvert : le bilan est cumulatif, et un exercice ne se clôt qu’après tous ceux qui le précèdent.';
		closeFiscalYearMock.mockRejectedValue({
			code: 'EARLIER_FISCAL_YEAR_OPEN',
			status: 409,
			message: serveur,
		});

		render(Page);

		await fireEvent.click(await screen.findByTestId('fiscal-year-close-2'));
		await fireEvent.click(await screen.findByTestId('fiscal-year-close-confirm'));

		await waitFor(() => expect(vi.mocked(notifyError)).toHaveBeenCalledWith(serveur));
		expect(vi.mocked(notifyError)).not.toHaveBeenCalledWith('Cet exercice est déjà clôturé');
		expect(closeFiscalYearMock).toHaveBeenCalledWith(2);
		// La liste se recharge.
		await waitFor(() => expect(listFiscalYearsMock).toHaveBeenCalledTimes(2));
	});
});
