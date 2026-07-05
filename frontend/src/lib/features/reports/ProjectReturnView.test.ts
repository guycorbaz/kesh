// Story 19-6b — tests Vitest pour ProjectReturnView.svelte.

import { describe, it, expect, vi } from 'vitest';
import { render } from '@testing-library/svelte';

vi.mock('$lib/shared/utils/i18n.svelte', () => ({
	i18nMsg: (_key: string, fallback: string) => fallback,
}));

import ProjectReturnView from './ProjectReturnView.svelte';
import type { ProjectReturnDto } from './reports.types';

function makeReport(rendementPct: string | null): ProjectReturnDto {
	return {
		reportType: 'project-return',
		project: { id: 1, code: 'INVEST', name: 'Investissement' },
		mode: 'fiscal_year',
		periodLabel: 'Exercice 2026',
		sections: [
			{
				project: { id: 1, code: 'INVEST', name: 'Investissement' },
				isRoot: true,
				coutInvesti: '450.00',
				revenus: '200.00',
				resultatNet: '100.00',
				rendementPct,
			},
		],
		totals: {
			coutInvesti: '450.00',
			revenus: '200.00',
			resultatNet: '100.00',
			rendementPct,
		},
	};
}

describe('ProjectReturnView', () => {
	it('affiche les totaux et le rendement %', async () => {
		const { findByTestId } = render(ProjectReturnView, { report: makeReport('44.44') });
		await findByTestId('project-return-view');
		const pct = await findByTestId('project-return-total-pct');
		expect(pct.textContent).toContain('44.44%');
	});

	it('affiche « — » quand le rendement est null (coût investi 0)', async () => {
		const { findByTestId } = render(ProjectReturnView, { report: makeReport(null) });
		const pct = await findByTestId('project-return-total-pct');
		expect(pct.textContent).toContain('—');
	});

	it('affiche l’état vide quand aucune section', async () => {
		const r = { ...makeReport(null), sections: [] };
		const { findByTestId } = render(ProjectReturnView, { report: r });
		await findByTestId('project-return-empty');
	});
});
