// Story 37-4 — tests Vitest pour JournalReportView.svelte (absents jusque-là) :
// sections, lignes, totaux, et la partie en devise d'une ligne (écran seulement,
// AC-7) ; mono-devise, rien de visible ne change (AC-6).

import { describe, it, expect, vi } from 'vitest';
import { render } from '@testing-library/svelte';

vi.mock('$lib/shared/utils/i18n.svelte', () => ({
	i18nMsg: (_key: string, fallback: string, args?: Record<string, string | number>) =>
		fallback.replace(/\{ \$(\w+) \}/g, (_: string, k: string) => String(args?.[k] ?? '')),
}));

import JournalReportView from './JournalReportView.svelte';
import type { JournalEntryLineRow, JournalReportDto } from './reports.types';

function ligne(over: Partial<JournalEntryLineRow> = {}): JournalEntryLineRow {
	return {
		accountId: 1,
		accountNumber: '1000',
		accountName: 'Caisse',
		debit: '91.00',
		credit: '0.00',
		lineOrder: 0,
		...over,
	};
}

function makeDto(lines: JournalEntryLineRow[]): JournalReportDto {
	return {
		period: { fiscalYearId: 1, startDate: '2026-01-01', endDate: '2026-12-31' },
		journals: [
			{
				journal: 'OD',
				entries: [
					{
						entryId: 1,
						entryNumber: 7,
						entryDate: '2026-02-10',
						description: 'Vente USD',
						lines,
					},
				],
				sectionTotalDebit: '91.00',
				sectionTotalCredit: '91.00',
			},
		],
		grandTotalDebit: '91.00',
		grandTotalCredit: '91.00',
	};
}

const CHF = [
	ligne(),
	ligne({
		accountId: 2,
		accountNumber: '3000',
		accountName: 'Ventes',
		debit: '0.00',
		credit: '91.00',
		lineOrder: 1,
	}),
];

describe('JournalReportView', () => {
	it('affiche l’écriture, ses lignes et les totaux', () => {
		const { container } = render(JournalReportView, { dto: makeDto(CHF) });
		const text = container.textContent ?? '';
		expect(text).toContain('#7');
		expect(text).toContain('Vente USD');
		expect(text).toContain('Ventes');
		expect(text).toContain('91.00');
	});

	it('Story 37-4 — la partie en devise d’une ligne sous la ligne, la réévaluation sans montant', () => {
		const usd = ligne({
			accountId: 3,
			accountNumber: '1020',
			accountName: 'Banque USD',
			currency: 'USD',
			amountCurrency: '100.0000',
			exchangeRate: '0.9100000000',
			rateSource: 'manual',
		});
		const reeval = ligne({
			accountId: 3,
			accountNumber: '1020',
			accountName: 'Banque USD',
			debit: '15.00',
			lineOrder: 2,
			currency: 'USD',
			amountCurrency: '0.0000',
			exchangeRate: '0.9312000000',
			rateSource: 'revaluation',
		});
		const { getAllByTestId } = render(JournalReportView, {
			dto: makeDto([usd, CHF[1], reeval]),
		});
		const fx = getAllByTestId('journal-fx-line').map((r) => r.textContent?.trim());
		expect(fx).toEqual(['USD 100.00 × 0.9100', 'Réévaluation au cours 0.9312']);
	});

	it('mono-devise : rien de visible ne change (aucune ligne en devise)', () => {
		const { container } = render(JournalReportView, { dto: makeDto(CHF) });
		expect(container.innerHTML).not.toContain('journal-fx-line');
		expect(container.querySelectorAll('tbody tr')).toHaveLength(2);
	});
});

// Story 55-2 (AC-3) — chaque ligne d'écriture du journal : le numéro ouvre le grand livre du compte sur la
// période du journal. Rouge sans le correctif (numéro en texte).
describe('JournalReportView — Story 55-2, liens de compte', () => {
	it('le numéro de chaque ligne est un lien vers le grand livre du compte, période du journal', async () => {
		const { LINK_CLASS } = await import('$lib/shared/navigation/link-class');
		const { getByRole } = render(JournalReportView, { dto: makeDto(CHF) });
		const caisse = getByRole('link', { name: '1000' });
		expect(caisse.getAttribute('href')).toBe('/reports?tab=general-ledger&accountId=1&from=2026-01-01&to=2026-12-31');
		expect(caisse.getAttribute('class')).toBe(LINK_CLASS);
		expect(caisse.getAttribute('title')).toBe('Voir le détail dans le grand livre');
		expect(getByRole('link', { name: '3000' }).getAttribute('href')).toBe(
			'/reports?tab=general-ledger&accountId=2&from=2026-01-01&to=2026-12-31',
		);
	});
});
