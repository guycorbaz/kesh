// Story 21-7 — tests Vitest pour AgedReceivablesView.svelte.
// Couvre : rendu des lignes + total général + empty-state + drill-down ?contactId=.

import { describe, it, expect, vi } from 'vitest';
import { render } from '@testing-library/svelte';

vi.mock('$lib/shared/utils/i18n.svelte', () => ({
	i18nMsg: (_key: string, fallback: string) => fallback,
}));

import AgedReceivablesView from './AgedReceivablesView.svelte';
import type { AgedReceivablesDto } from './reports.types';

function makeDto(): AgedReceivablesDto {
	return {
		asOf: '2026-07-20',
		rows: [
			{
				contactId: 42,
				contactName: 'Alpha SA',
				notDue: '110.00',
				days1To30: '205.00',
				days31To60: '307.00',
				days61To90: '409.00',
				daysOver90: '1581.00',
				total: '2612.00',
			},
		],
		totals: {
			notDue: '110.00',
			days1To30: '205.00',
			days31To60: '307.00',
			days61To90: '409.00',
			daysOver90: '1581.00',
			total: '2612.00',
		},
	};
}

describe('AgedReceivablesView', () => {
	it('rend une ligne par contact avec le total et le total général', () => {
		const { getByTestId, getAllByTestId } = render(AgedReceivablesView, { dto: makeDto() });

		expect(getByTestId('aged-receivables-table')).toBeTruthy();
		const rows = getAllByTestId('aged-receivables-row');
		expect(rows).toHaveLength(1);
		expect(rows[0].textContent).toContain('Alpha SA');
		// Montant total de la ligne (formaté suisse).
		expect(rows[0].textContent).toContain('2’612.00');
		// Ligne « Total général ».
		expect(getByTestId('aged-receivables-total').textContent).toContain('2’612.00');
	});

	it('le nom du contact est un lien drill-down vers /invoices?contactId=', () => {
		const { getByRole } = render(AgedReceivablesView, { dto: makeDto() });
		const link = getByRole('link', { name: 'Alpha SA' });
		expect(link.getAttribute('href')).toBe('/invoices?contactId=42');
	});

	it('affiche l’empty-state quand rows est vide', () => {
		const empty: AgedReceivablesDto = {
			asOf: '2026-07-20',
			rows: [],
			totals: {
				notDue: '0.00',
				days1To30: '0.00',
				days31To60: '0.00',
				days61To90: '0.00',
				daysOver90: '0.00',
				total: '0.00',
			},
		};
		const { queryByTestId, getByText } = render(AgedReceivablesView, { dto: empty });
		expect(queryByTestId('aged-receivables-table')).toBeNull();
		expect(getByText('Aucune créance client ouverte.')).toBeTruthy();
	});
});
