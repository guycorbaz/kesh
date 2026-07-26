// Story 14-3c — tests Vitest pour BalanceSheetView.svelte.
// Couvre la présentation des fonds propres PAR RÔLE : section « Capitaux propres »
// dédiée groupée par rôle, distinction compte physique / ligne calculée (D1), et
// absence des comptes de fonds propres dans la table Passifs (partition D2).
//
// i18nMsg est mocké pour renvoyer le fallback → le sous-titre de rôle affiche la
// valeur brute du rôle (2e argument passé par le composant), suffisant pour asserter
// le regroupement sans dépendre des traductions.

import { describe, it, expect, vi } from 'vitest';
import { render } from '@testing-library/svelte';

vi.mock('$lib/shared/utils/i18n.svelte', () => ({
	i18nMsg: (_key: string, fallback: string) => fallback,
}));

import BalanceSheetView from './BalanceSheetView.svelte';
import type { BalanceSheetDto } from './reports.types';

/** Bilan avec : 1 actif, 1 dette réelle (Payable), 2 comptes de fonds propres
 *  physiques (Capital + Bénéfice reporté 2970 à 50'000), + 2 lignes calculées
 *  virtuelles (report 5'000, résultat 1'200) — collision D1 sur RetainedEarnings. */
function makeDto(overrides: Partial<BalanceSheetDto> = {}): BalanceSheetDto {
	return {
		period: { fiscalYearId: 1, startDate: '2026-01-01', endDate: '2026-12-31' },
		assets: [
			{
				accountId: 1,
				accountNumber: '1000',
				accountName: 'Banque',
				accountType: 'Asset',
				active: true,
				balance: '56200',
				role: null,
			},
		],
		liabilities: [
			{
				accountId: 2,
				accountNumber: '2000',
				accountName: 'Fournisseurs',
				accountType: 'Liability',
				active: true,
				balance: '0',
				role: 'Payable',
			},
		],
		equity: [
			{
				accountId: 3,
				accountNumber: '2800',
				accountName: 'Capital social',
				accountType: 'Liability',
				active: true,
				balance: '20000',
				role: 'EquityCapital',
			},
			{
				accountId: 4,
				accountNumber: '2970',
				accountName: 'Bénéfice reporté (compte)',
				accountType: 'Liability',
				active: true,
				balance: '30000',
				role: 'RetainedEarnings',
			},
		],
		totalAssets: '56200',
		totalLiabilities: '0',
		totalEquity: '50000',
		retainedEarnings: '5000',
		equityResult: '1200',
		equationHolds: true,
		...overrides,
	};
}

describe('BalanceSheetView — fonds propres par rôle (Story 14-3c)', () => {
	it('affiche une section Capitaux propres dédiée avec les comptes physiques', () => {
		const { getAllByText, container } = render(BalanceSheetView, { dto: makeDto() });
		// Titre de section (fallback i18n).
		expect(getAllByText('Capitaux propres').length).toBeGreaterThan(0);
		// Les comptes physiques de fonds propres sont itemisés.
		expect(container.textContent).toContain('Capital social');
		expect(container.textContent).toContain('Bénéfice reporté (compte)');
	});

	it('distingue le compte physique RetainedEarnings de la ligne calculée (D1)', () => {
		const { container } = render(BalanceSheetView, { dto: makeDto() });
		// Ligne calculée explicitement marquée « (calculé) » (clé -calculated).
		expect(container.textContent).toContain('Résultat reporté (calculé)');
		// Le compte physique 2970 reste une ligne distincte sous son numéro.
		expect(container.textContent).toContain('2970');
	});

	it('exclut les comptes de fonds propres de la table Passifs (partition D2)', () => {
		const { container } = render(BalanceSheetView, { dto: makeDto() });
		const tables = container.querySelectorAll('table');
		// Ordre DOM : [0] Actifs, [1] Passifs, [2] Capitaux propres.
		expect(tables.length).toBe(3);
		const liabilities = tables[1].textContent ?? '';
		const equity = tables[2].textContent ?? '';
		// Le capital NE figure PAS dans les Passifs, mais dans Capitaux propres.
		expect(liabilities).not.toContain('Capital social');
		expect(liabilities).toContain('Fournisseurs');
		expect(equity).toContain('Capital social');
		expect(equity).toContain('2970');
	});

	it('affiche la section même sur un reclassement pur equity (report/résultat nuls)', () => {
		const dto = makeDto({
			assets: [],
			liabilities: [],
			totalAssets: '0',
			totalLiabilities: '0',
			retainedEarnings: '0',
			equityResult: '0',
			// equity conservé (2 comptes) → NON vide malgré actif/passif/virtuels nuls.
			totalEquity: '50000',
		});
		const { container, queryByRole } = render(BalanceSheetView, { dto });
		// Pas de message « rapport vide ».
		expect(queryByRole('status')).toBeNull();
		expect(container.textContent).toContain('Capital social');
	});
});
