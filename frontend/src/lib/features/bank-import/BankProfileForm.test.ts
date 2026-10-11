// Story 26-1 (#620, #621), AC12 — tests Vitest du formulaire de profil CSV :
// le réglage du sens des débits (mode séparé seulement) et le plafond des
// lignes à ignorer.
//
// Pattern : mock de `bank-profile.api` + render via `@testing-library/svelte`
// (patron `reconciliation/TransactionSplitModal.test.ts`). Mocks définis AVANT
// l'import du composant (hoisting Vitest).

import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { render, fireEvent, cleanup } from '@testing-library/svelte';

vi.mock('./bank-profile.api', () => ({
	createBankProfile: vi.fn(),
	updateBankProfile: vi.fn(),
}));

vi.mock('$lib/shared/utils/i18n.svelte', () => ({
	i18nMsg: (_key: string, fallback: string) => fallback,
}));

import * as api from './bank-profile.api';
import BankProfileForm from './BankProfileForm.svelte';
import type { BankProfile } from './bank-profile.types';

const mockApi = vi.mocked(api);

function existingProfile(overrides: Partial<BankProfile> = {}): BankProfile {
	return {
		id: 3,
		bankName: 'UBS',
		filenamePattern: null,
		columnMapping: { date: 2, debit_credit_split: [5, 6], reference: 9, details: 10 },
		dateFormat: '%Y-%m-%d',
		decimalSeparator: '.',
		fieldSeparator: ';',
		encoding: null,
		headerRowCount: 9,
		debitSign: 'negative',
		createdAt: '2026-01-01T00:00:00',
		updatedAt: '2026-01-01T00:00:00',
		...overrides,
	};
}

async function submit(container: HTMLElement): Promise<void> {
	const form = container.querySelector('[data-testid="bank-import-profile-form"]');
	expect(form).not.toBeNull();
	await fireEvent.submit(form as HTMLFormElement);
}

function radio(container: HTMLElement, testid: string): HTMLInputElement {
	const el = container.querySelector(`[data-testid="${testid}"]`);
	expect(el, testid).not.toBeNull();
	return el as HTMLInputElement;
}

describe('BankProfileForm — sens des débits (Story 26-1)', () => {
	beforeEach(() => {
		mockApi.createBankProfile.mockReset();
		mockApi.updateBankProfile.mockReset();
		mockApi.createBankProfile.mockResolvedValue(existingProfile());
		mockApi.updateBankProfile.mockResolvedValue(existingProfile());
	});
	afterEach(() => cleanup());

	it('(a) mode séparé + « signe moins » → la requête porte debitSign negative', async () => {
		const { container } = render(BankProfileForm, { props: {} });
		await fireEvent.input(container.querySelector('[data-testid="bank-name-input"]')!, {
			target: { value: 'UBS' },
		});
		await fireEvent.click(container.querySelector('[data-testid="use-debit-credit-split"]')!);
		await fireEvent.click(radio(container, 'debit-sign-negative'));
		await submit(container);
		expect(mockApi.createBankProfile).toHaveBeenCalledTimes(1);
		const req = mockApi.createBankProfile.mock.calls[0][0];
		expect(req.debitSign).toBe('negative');
		expect(req.columnMapping.debit_credit_split).toBeDefined();
		expect(req.columnMapping.amount).toBeUndefined();
	});

	it('(b) colonne unique → debitSign positive, même après avoir choisi « signe moins » puis décoché', async () => {
		const { container } = render(BankProfileForm, { props: {} });
		const split = container.querySelector('[data-testid="use-debit-credit-split"]')!;
		// Groupe absent en colonne unique.
		expect(container.querySelector('[data-testid="debit-sign-negative"]')).toBeNull();
		await fireEvent.click(split);
		await fireEvent.click(radio(container, 'debit-sign-negative'));
		await fireEvent.click(split);
		expect(container.querySelector('[data-testid="debit-sign-negative"]')).toBeNull();
		await submit(container);
		const req = mockApi.createBankProfile.mock.calls[0][0];
		expect(req.debitSign).toBe('positive');
		expect(req.columnMapping.amount).toBeDefined();
	});

	it('(c) un profil existant negative s’ouvre avec le bon bouton coché', () => {
		const { container } = render(BankProfileForm, { props: { existing: existingProfile() } });
		expect(radio(container, 'debit-sign-negative').checked).toBe(true);
		expect(radio(container, 'debit-sign-positive').checked).toBe(false);
	});

	it('(d) le champ des lignes à ignorer porte max="20"', () => {
		const { container } = render(BankProfileForm, { props: {} });
		const input = container.querySelector('[data-testid="header-row-count-input"]');
		expect(input?.getAttribute('max')).toBe('20');
		expect(input?.getAttribute('min')).toBe('0');
	});

	it('(e) « signe moins », décocher, recocher → le choix est gardé', async () => {
		const { container } = render(BankProfileForm, { props: {} });
		const split = container.querySelector('[data-testid="use-debit-credit-split"]')!;
		await fireEvent.click(split);
		await fireEvent.click(radio(container, 'debit-sign-negative'));
		await fireEvent.click(split);
		await fireEvent.click(split);
		expect(radio(container, 'debit-sign-negative').checked).toBe(true);
		await submit(container);
		expect(mockApi.createBankProfile.mock.calls[0][0].debitSign).toBe('negative');
	});

	it('un profil existant sans signe s’ouvre sur « sans signe » et le renvoie au PUT', async () => {
		const { container } = render(BankProfileForm, {
			props: { existing: existingProfile({ debitSign: 'positive' }) },
		});
		expect(radio(container, 'debit-sign-positive').checked).toBe(true);
		await submit(container);
		expect(mockApi.updateBankProfile).toHaveBeenCalledWith(
			3,
			expect.objectContaining({ debitSign: 'positive', headerRowCount: 9 }),
		);
	});
});
