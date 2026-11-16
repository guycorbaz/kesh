// Story 37-3b (AC-2, S5) — l'assistant TVA achat en société multi-devises :
// passage en devise quand la contrepartie est un compte en devise (montant HT
// dans la devise, cours proposé par le `lookup` du formulaire), lignes insérées
// dans la devise du compte ; charge et contrepartie de deux devises refusées.

import { describe, it, expect, vi } from 'vitest';
import { render, fireEvent, screen, waitFor } from '@testing-library/svelte';

vi.mock('$lib/shared/utils/i18n.svelte', () => ({
	i18nMsg: (_key: string, fallback: string, args?: Record<string, unknown>) =>
		fallback.replace(/\{ ?\$(\w+) ?\}/g, (_m: string, k: string) => String(args?.[k] ?? ''))
}));
vi.mock('$lib/features/onboarding/onboarding.svelte', () => ({
	i18nMsg: (_key: string, fallback: string, args?: Record<string, unknown>) =>
		fallback.replace(/\{ ?\$(\w+) ?\}/g, (_m: string, k: string) => String(args?.[k] ?? ''))
}));
vi.mock('$lib/features/vat-rates/vat-rates.store.svelte', () => ({
	getVatRates: vi.fn(async () => [{ id: 1, rate: '8.10', label: 'Normal', category: '' }])
}));

import VatPurchaseAssistant from './VatPurchaseAssistant.svelte';
import type { AccountResponse } from '$lib/features/accounts/accounts.types';
import type { LookupResult } from './rate-lookup';

function account(id: number, number: string, extra: Partial<AccountResponse> = {}): AccountResponse {
	return {
		id,
		companyId: 1,
		number,
		name: `Compte ${number}`,
		accountType: 'Asset',
		parentId: null,
		active: true,
		role: null,
		postable: true,
		letterable: false,
		version: 1,
		createdAt: '',
		updatedAt: '',
		...extra
	};
}

const ACCOUNTS = [
	account(4, '4000', { accountType: 'Expense' }),
	account(22, '1022', { currency: 'USD' }),
	account(21, '1021', { currency: 'EUR' }),
	account(9, '1170')
];

const lookupRate = vi.fn(
	async (): Promise<LookupResult> => ({
		outcome: { kind: 'rate', rate: { rate: '0.8846', source: 'afc', month: '2026-03-01' } },
		note: null
	})
);

async function pickIn(index: number, number: string) {
	const input = screen.getAllByRole('textbox', { name: 'Compte' })[index];
	await fireEvent.focus(input);
	const option = screen.getAllByRole('option').find((o) => (o.textContent ?? '').includes(number));
	if (!option) throw new Error(`option ${number} introuvable`);
	await fireEvent.mouseDown(option);
	await fireEvent.blur(input);
}

describe('VatPurchaseAssistant — en devise', () => {
	it('contrepartie USD : HT en USD, cours proposé, lignes USD 100 → 88.46 / 7.17 / 95.63', async () => {
		const onApply = vi.fn();
		render(VatPurchaseAssistant, {
			props: {
				accounts: ACCOUNTS,
				accountsLoadError: false,
				recoverableAccountId: 9,
				onApply,
				multiCurrency: true,
				entryDate: '2026-03-15',
				lookupRate
			}
		});
		await fireEvent.click(screen.getByRole('button', { name: /Assistant TVA achat/ }));
		await pickIn(0, '4000');
		await pickIn(1, '1022');
		await waitFor(() =>
			expect((screen.getByTestId('vat-purchase-fx-rate') as HTMLInputElement).value).toBe('0.8846')
		);
		expect(screen.getByText('Montant HT (USD)')).toBeTruthy();
		expect(lookupRate).toHaveBeenCalledWith('USD', '2026-03-15');
		await fireEvent.input(screen.getByPlaceholderText('0.00'), { target: { value: '100' } });
		// Le taux de TVA n'est pas choisi (le Select de bits-ui ne s'ouvre pas sous
		// jsdom) : l'insertion reste désactivée. Les lignes insérées sont tenues par
		// le Vitest de `foreignPurchaseVatEntry` (`fx-support.test.ts`).
		expect(
			(screen.getByRole('button', { name: 'Insérer les lignes' }) as HTMLButtonElement).disabled
		).toBe(true);
		// Un cours retapé invalide le signale.
		await fireEvent.input(screen.getByTestId('vat-purchase-fx-rate'), { target: { value: '0' } });
		expect(screen.getByText(/Cours invalide/)).toBeTruthy();
		expect(onApply).not.toHaveBeenCalled();
	});

	it('charge USD et contrepartie EUR : impossible à composer (sélecteur restreint)', async () => {
		render(VatPurchaseAssistant, {
			props: {
				accounts: ACCOUNTS,
				accountsLoadError: false,
				recoverableAccountId: 9,
				onApply: vi.fn(),
				multiCurrency: true,
				entryDate: '2026-03-15',
				lookupRate
			}
		});
		await fireEvent.click(screen.getByRole('button', { name: /Assistant TVA achat/ }));
		await pickIn(0, '1022');
		const input = screen.getAllByRole('textbox', { name: 'Compte' })[1];
		await fireEvent.focus(input);
		const options = screen.queryAllByRole('option').map((o) => o.textContent ?? '');
		expect(options.some((o) => o.includes('1021'))).toBe(false);
		expect(options.some((o) => o.includes('1170'))).toBe(true);
	});
});
