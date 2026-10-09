// Story 15-12b (#543, AC 15) — détection de l'état hérité « exercice ouvert suivi d'un
// exercice clos », dont le bandeau de l'écran des exercices tire ses trois noms.

import { describe, it, expect } from 'vitest';
import { outOfOrderState } from './fiscal-years.helpers';
import type { FiscalYearResponse } from './fiscal-years.types';

function fy(annee: number, status: 'Open' | 'Closed'): FiscalYearResponse {
	return {
		id: annee,
		companyId: 1,
		name: `Exercice ${annee}`,
		startDate: `${annee}-01-01`,
		endDate: `${annee}-12-31`,
		status,
		createdAt: '2026-01-01T00:00:00',
		updatedAt: '2026-01-01T00:00:00',
	};
}

const noms = (l: FiscalYearResponse[]) => {
	const s = outOfOrderState(l);
	return s === null ? null : [s.open.name, s.closed.name, s.latest.name];
};

describe('outOfOrderState', () => {
	it.each([
		['aucun exercice', []],
		['un seul ouvert', [fy(2026, 'Open')]],
		['tous clos', [fy(2025, 'Closed'), fy(2026, 'Closed')]],
		['clos en préfixe', [fy(2025, 'Closed'), fy(2026, 'Open'), fy(2027, 'Open')]],
	])('état sain (%s) → null', (_cas, liste) => {
		expect(outOfOrderState(liste)).toBeNull();
	});

	it('N ouvert, N+1 clos → les trois noms, {closed} = {latest}', () => {
		expect(noms([fy(2026, 'Open'), fy(2027, 'Closed')])).toEqual([
			'Exercice 2026',
			'Exercice 2027',
			'Exercice 2027',
		]);
	});

	it('{closed} ≠ {latest} : le plus proche clos, puis le plus récent clos', () => {
		expect(
			noms([fy(2028, 'Closed'), fy(2025, 'Open'), fy(2027, 'Open'), fy(2026, 'Closed')])
		).toEqual(['Exercice 2025', 'Exercice 2026', 'Exercice 2028']);
	});

	it('{open} est le PLUS ANCIEN ouvert, même s’il n’est pas le plus proche du clos', () => {
		// 2024 clos, 2025 ouvert, 2026 ouvert, 2027 clos : 2025 est nommé, non 2026.
		expect(
			noms([fy(2024, 'Closed'), fy(2026, 'Open'), fy(2025, 'Open'), fy(2027, 'Closed')])
		).toEqual(['Exercice 2025', 'Exercice 2027', 'Exercice 2027']);
	});
});
