/**
 * Le dialogue « Solder le reste » — Story 25-4-d2b (#490).
 *
 * ⚠️ Chaque test nomme la MUTATION qu'il attrape.
 */
import { describe, it, expect, vi, afterEach } from 'vitest';
import { render, fireEvent, cleanup } from '@testing-library/svelte';

vi.mock('$lib/shared/utils/i18n.svelte', () => ({
	i18nMsg: (_k: string, fallback: string) => fallback,
}));

import WriteOffDialog from './WriteOffDialog.svelte';
import type { InvoiceSettingsResponse } from './invoices.types';

const settings = {
	defaultDiscountAccountId: 1,
	defaultBankFeesAccountId: null,
	defaultBadDebtAccountId: 3,
	defaultRoundingAccountId: 4,
} as InvoiceSettingsResponse;

function open(amountDue: string, onConfirm = vi.fn(), errorMsg = '') {
	return render(WriteOffDialog, {
		props: {
			open: true,
			onOpenChange: vi.fn(),
			invoiceDate: '2026-03-01',
			amountDue,
			settings,
			errorMsg,
			onConfirm,
		},
	});
}

afterEach(() => cleanup());

describe('WriteOffDialog', () => {
	it('le reste d’arrondi n’est pas proposé à partir de 0.05 (mutation : filtre absent)', async () => {
		const r = open('60.0000');
		expect(r.queryByTestId('write-off-nature-rounding')).toBeNull();
		expect(r.getByTestId('write-off-nature-discount')).toBeTruthy();
		cleanup();
		const r2 = open('0.0040');
		expect(r2.getByTestId('write-off-nature-rounding')).toBeTruthy();
		expect(r2.getByTestId('write-off-amount').textContent).toContain('0.0040');
		expect(r2.getByTestId('write-off-sub-centime')).toBeTruthy();
	});

	it('une nature sans compte est signalée et sa confirmation désactivée (mutation : bouton actif)', async () => {
		const onConfirm = vi.fn();
		const r = open('60.0000', onConfirm);
		await fireEvent.click(r.getByTestId('write-off-nature-bank_fees'));
		expect(r.getByTestId('write-off-missing-account')).toBeTruthy();
		const confirm = r.getByTestId('write-off-confirm') as HTMLButtonElement;
		expect(confirm.disabled).toBe(true);
		await fireEvent.click(confirm);
		expect(onConfirm).not.toHaveBeenCalled();
	});

	it('confirmer émet la nature et la date (mutation : nature non transmise)', async () => {
		const onConfirm = vi.fn();
		const r = open('60.0000', onConfirm);
		await fireEvent.click(r.getByTestId('write-off-nature-discount'));
		await fireEvent.click(r.getByTestId('write-off-confirm'));
		expect(onConfirm).toHaveBeenCalledTimes(1);
		expect(onConfirm.mock.calls[0][0].nature).toBe('discount');
		expect(onConfirm.mock.calls[0][0].settledOn).toMatch(/^\d{4}-\d{2}-\d{2}$/);
	});

	it('le message du serveur s’affiche dans le dialogue', () => {
		const r = open('60.0000', vi.fn(), 'Aucun compte de frais bancaires utilisable');
		expect(r.getByTestId('write-off-error').textContent).toContain('frais bancaires');
	});
});
