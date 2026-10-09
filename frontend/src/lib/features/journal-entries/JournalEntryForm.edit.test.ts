// Story 15-8a (#532, D8) — le formulaire en MODE ÉDITION : les branches de
// refus du serveur. ⛔ Ce qu'on vérifie : un refus qui dit que l'écriture a
// changé (exercice clos, pièce apparue, version périmée) rend un toast PUIS
// recharge la fiche (`onStale`) — sans modale, et sans le conseil de
// `notifyMissingFiscalYearOrFallback`, faux en édition ; un refus de saisie
// laisse le formulaire ouvert.

import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, fireEvent, screen } from '@testing-library/svelte';

const { updateMock, toastError, notifyMock } = vi.hoisted(() => ({
	updateMock: vi.fn(),
	toastError: vi.fn(),
	notifyMock: vi.fn(() => false)
}));

vi.mock('$lib/shared/utils/i18n.svelte', () => ({
	i18nMsg: (_key: string, fallback: string) => fallback
}));
vi.mock('$lib/features/onboarding/onboarding.svelte', () => ({
	i18nMsg: (_key: string, fallback: string) => fallback
}));
vi.mock('./journal-entries.api', () => ({
	createJournalEntry: vi.fn(),
	updateJournalEntry: updateMock
}));
vi.mock('svelte-sonner', () => ({
	toast: { error: toastError, success: vi.fn() }
}));
vi.mock('$lib/shared/utils/notify', () => ({
	notifyMissingFiscalYearOrFallback: notifyMock
}));

import JournalEntryForm from './JournalEntryForm.svelte';
import type { AccountResponse } from '$lib/features/accounts/accounts.types';
import type { JournalEntryResponse } from './journal-entries.types';

function account(id: number, number: string, extra: Partial<AccountResponse> = {}): AccountResponse {
	return {
		id,
		companyId: 1,
		number,
		name: `Compte ${number}`,
		accountType: 'Asset',
		parentId: null,
		active: true,
		postable: true,
		version: 1,
		createdAt: '2026-01-01T00:00:00',
		updatedAt: '2026-01-01T00:00:00',
		...extra
	} as AccountResponse;
}

const ENTRY: JournalEntryResponse = {
	id: 42,
	companyId: 1,
	fiscalYearId: 7,
	entryNumber: 3,
	entryDate: '2026-04-10',
	journal: 'OD',
	description: 'Soldes de départ',
	version: 1,
	reversesEntryId: null,
	lines: [
		{ id: 1, accountId: 1, lineOrder: 1, debit: '100.0000', credit: '0.0000', projectId: null, letteringKey: null, letteringCode: null, letteringOrigin: null },
		{ id: 2, accountId: 2, lineOrder: 2, debit: '0.0000', credit: '100.0000', projectId: null, letteringKey: null, letteringCode: null, letteringOrigin: null }
	],
	createdAt: '2026-04-10T10:00:00',
	updatedAt: '2026-04-10T10:00:00'
};

function apiError(code: string, message = `message serveur ${code}`) {
	return { code, status: code === 'OPTIMISTIC_LOCK_CONFLICT' ? 409 : 400, message };
}

async function submitWith(error: unknown, accounts = [account(1, '1000'), account(2, '2000')]) {
	updateMock.mockRejectedValueOnce(error);
	const onStale = vi.fn();
	const onSuccess = vi.fn();
	render(JournalEntryForm, {
		props: {
			accounts,
			accountsLoadError: false,
			initialEntry: ENTRY,
			entryFiscalYear: { startDate: '2026-01-01', endDate: '2026-12-31' },
			onSuccess,
			onCancel: vi.fn(),
			onStale
		}
	});
	await fireEvent.click(screen.getByRole('button', { name: 'Valider' }));
	// Laisse la promesse rejetée se dérouler.
	await new Promise((r) => setTimeout(r, 0));
	return { onStale, onSuccess };
}

describe('JournalEntryForm — mode édition', () => {
	beforeEach(() => {
		updateMock.mockReset();
		toastError.mockReset();
		notifyMock.mockClear();
	});

	it('envoie le PUT avec la version lue', async () => {
		updateMock.mockResolvedValueOnce(ENTRY);
		const onSuccess = vi.fn();
		render(JournalEntryForm, {
			props: {
				accounts: [account(1, '1000'), account(2, '2000')],
				accountsLoadError: false,
				initialEntry: ENTRY,
				onSuccess,
				onCancel: vi.fn()
			}
		});
		await fireEvent.click(screen.getByRole('button', { name: 'Valider' }));
		await new Promise((r) => setTimeout(r, 0));
		expect(updateMock).toHaveBeenCalledWith(42, expect.objectContaining({ version: 1 }));
		expect(onSuccess).toHaveBeenCalled();
	});

	it("FISCAL_YEAR_CLOSED : toast du motif de l'écran, rechargement, sans notifyMissingFiscalYear", async () => {
		const { onStale } = await submitWith(apiError('FISCAL_YEAR_CLOSED'));
		expect(toastError).toHaveBeenCalledWith(
			expect.stringContaining('exercice de cette écriture est clôturé')
		);
		expect(onStale).toHaveBeenCalledTimes(1);
		expect(notifyMock).not.toHaveBeenCalled();
	});

	it.each([
		'LATER_FISCAL_YEAR_CLOSED',
		'ENTRY_IS_REVERSED',
		'OWNED_BY_INVOICE',
		'DETACHED_SUPPLIER_SETTLEMENT',
		'ENTRY_LETTERED'
	])(
		'%s : toast du message serveur, puis rechargement',
		async (code) => {
			const { onStale } = await submitWith(apiError(code));
			expect(toastError).toHaveBeenCalledWith(`message serveur ${code}`);
			expect(onStale).toHaveBeenCalledTimes(1);
		}
	);

	it('OPTIMISTIC_LOCK_CONFLICT : toast, rechargement, AUCUNE modale', async () => {
		const { onStale } = await submitWith(apiError('OPTIMISTIC_LOCK_CONFLICT'));
		expect(toastError).toHaveBeenCalledWith(
			'Cette écriture a été modifiée entre-temps : la fiche a été rechargée.'
		);
		expect(onStale).toHaveBeenCalledTimes(1);
		expect(screen.queryByRole('dialog')).toBeNull();
	});

	it.each(['PERIOD_LOCKED', 'ACCOUNT_NOT_POSTABLE', 'DATE_OUTSIDE_FISCAL_YEAR'])(
		'%s : toast, le formulaire reste ouvert',
		async (code) => {
			const { onStale } = await submitWith(apiError(code));
			expect(toastError).toHaveBeenCalledWith(`message serveur ${code}`);
			expect(onStale).not.toHaveBeenCalled();
		}
	);

	it('une ligne sur un compte archivé est signalée, à remplacer', async () => {
		render(JournalEntryForm, {
			props: {
				accounts: [account(1, '1000', { active: false }), account(2, '2000')],
				accountsLoadError: false,
				initialEntry: ENTRY,
				onSuccess: vi.fn(),
				onCancel: vi.fn()
			}
		});
		expect(screen.getAllByTestId('line-account-unusable')).toHaveLength(1);
	});

	it("les bornes de date suivent l'exercice de l'écriture", () => {
		render(JournalEntryForm, {
			props: {
				accounts: [account(1, '1000'), account(2, '2000')],
				accountsLoadError: false,
				initialEntry: ENTRY,
				entryFiscalYear: { startDate: '2026-01-01', endDate: '2026-12-31' },
				booksLockedThrough: '2026-03-31',
				onSuccess: vi.fn(),
				onCancel: vi.fn()
			}
		});
		const date = document.getElementById('entry-date') as HTMLInputElement;
		expect(date.min).toBe('2026-04-01');
		expect(date.max).toBe('2026-12-31');
	});
});
