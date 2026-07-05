// Story 19-6a — tests Vitest pour ProjectExpensesView.svelte.
// Couvre le rendu des sections + le drill-down expandable (code-review Pass 1).

import { describe, it, expect, vi } from 'vitest';
import { render, fireEvent } from '@testing-library/svelte';

vi.mock('$lib/shared/utils/i18n.svelte', () => ({
	i18nMsg: (_key: string, fallback: string) => fallback,
}));

import ProjectExpensesView from './ProjectExpensesView.svelte';
import type { ProjectExpensesDto } from './reports.types';

function makeReport(): ProjectExpensesDto {
	return {
		reportType: 'project-expenses',
		project: { id: 1, code: 'RENOV', name: 'Rénovation' },
		mode: 'fiscal_year',
		periodLabel: 'Exercice 2026',
		grandTotal: '340.00',
		sections: [
			{
				project: { id: 1, code: 'RENOV', name: 'Rénovation' },
				isRoot: true,
				subtotal: '140.00',
				rows: [
					{
						accountId: 10,
						accountNumber: '4000',
						accountName: 'Charges',
						amount: '140.00',
						entries: [
							{
								entryId: 5,
								entryNumber: 42,
								entryDate: '2026-03-01',
								description: 'Matériaux chantier',
								amount: '140.00',
							},
						],
					},
				],
			},
			{
				project: { id: 2, code: 'RENOV-CHALET', name: 'Chalet' },
				isRoot: false,
				subtotal: '200.00',
				rows: [
					{
						accountId: 10,
						accountNumber: '4000',
						accountName: 'Charges',
						amount: '200.00',
						entries: [],
					},
				],
			},
		],
	};
}

describe('ProjectExpensesView', () => {
	it('rend les sections (racine + sous-projet), sous-totaux et total', async () => {
		const { findByTestId, getByTestId } = render(ProjectExpensesView, { report: makeReport() });
		await findByTestId('project-expenses-view');
		await findByTestId('project-expenses-section-1');
		await findByTestId('project-expenses-section-2');
		expect(getByTestId('project-expenses-grand-total').textContent).toContain('340');
	});

	it("déplie une ligne de compte pour montrer les écritures (drill-down)", async () => {
		const { findByTestId, queryByTestId } = render(ProjectExpensesView, { report: makeReport() });
		// Avant clic : l'écriture n'est pas affichée.
		expect(queryByTestId('project-expenses-entry-5')).toBeNull();
		// Clic sur la ligne de compte de la section racine.
		const row = await findByTestId('project-expenses-row-1-10');
		await fireEvent.click(row);
		// Après clic : l'écriture contributrice apparaît.
		await findByTestId('project-expenses-entry-5');
	});

	it('affiche l’état vide quand aucune section', async () => {
		const empty = { ...makeReport(), sections: [], grandTotal: '0.00' };
		const { findByTestId } = render(ProjectExpensesView, { report: empty });
		await findByTestId('project-expenses-empty');
	});
});
