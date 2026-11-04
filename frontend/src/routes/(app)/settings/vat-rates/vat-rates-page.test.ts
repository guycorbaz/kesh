/**
 * v0172-correctifs (#880, § 24 bis de la fiche, AC-C2) — l'écran *Taux de TVA* dit les
 * dates de validité en forme suisse ; une validité sans fin reste « — ».
 *
 * Premier test de cette page. Le nom n'est pas `+page.test.ts` : SvelteKit réserve le
 * préfixe `+`.
 */
import { describe, it, expect, vi, afterEach } from 'vitest';
import { render, cleanup } from '@testing-library/svelte';

vi.mock('$lib/shared/utils/i18n.svelte', () => ({
	i18nMsg: (_k: string, fallback: string) => fallback,
}));
vi.mock('svelte-sonner', () => ({ toast: { success: vi.fn(), error: vi.fn() } }));
const listMock = vi.fn();
vi.mock('$lib/features/vat-rates/vat-rates.api', () => ({
	listVatRates: (h: boolean) => listMock(h),
	createVatRate: vi.fn(),
	updateVatRate: vi.fn(),
	deactivateVatRate: vi.fn(),
}));

import Page from './+page.svelte';

afterEach(() => {
	cleanup();
	vi.clearAllMocks();
});

describe('taux de TVA — dates de validité en forme suisse (v0172, #880)', () => {
	it('du 01.01.2018 au 31.12.2023, puis dès le 01.01.2024 ; validité sans fin « — » ; jamais l’ISO (MC2)', async () => {
		listMock.mockResolvedValue([
			{
				id: 1,
				category: 'normal',
				label: 'Normal',
				rate: '7.70',
				validFrom: '2018-01-01',
				validTo: '2023-12-31',
				active: false,
				version: 1,
			},
			{
				id: 2,
				category: 'normal',
				label: 'Normal',
				rate: '8.10',
				validFrom: '2024-01-01',
				validTo: null,
				active: true,
				version: 1,
			},
		]);
		const { findAllByTestId } = render(Page);
		const rows = await findAllByTestId('vat-rate-row');
		const cells = rows.map((r) =>
			Array.from(r.querySelectorAll('td'))
				.slice(1, 3)
				.map((td) => td.textContent?.trim()),
		);
		// L'écran trie les taux d'une catégorie ; l'ordre n'est pas l'objet du test.
		expect(cells.sort()).toEqual([
			['01.01.2018', '31.12.2023'],
			['01.01.2024', '—'],
		]);
		expect(rows.map((r) => r.textContent).join(' ')).not.toMatch(/20\d{2}-\d{2}-\d{2}/);
	});
});
