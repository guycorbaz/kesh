// Story 37-3b (FR-37-3b-6, revue de code P2) — la date et le genre du cours proposé d'une grille.

import { describe, it, expect } from 'vitest';
import { openingProposal } from './opening-fx';
import { openingRateDate } from '$lib/features/journal-entries/fx';

const fy = { startDate: '2026-01-01', openingRateKind: 'month' as const };

describe('openingProposal', () => {
	it('génération : cours d’ouverture, genre de l’exercice', () => {
		expect(openingProposal(fy)).toEqual({ kind: 'month', date: openingRateDate('2026-01-01', 'month') });
		expect(openingProposal({ ...fy, openingRateKind: 'day' })).toEqual({ kind: 'day', date: openingRateDate('2026-01-01', 'day') });
	});
	it('complément daté du premier jour : le cours d’ouverture', () => {
		expect(openingProposal(fy, { dateKind: 'OPENING_DAY', date: '2026-01-01' })).toEqual({
			kind: 'month',
			date: openingRateDate('2026-01-01', 'month'),
		});
		expect(openingProposal({ ...fy, openingRateKind: 'day' }, { dateKind: 'OPENING_DAY', date: '2026-01-01' })).toEqual({
			kind: 'day',
			date: openingRateDate('2026-01-01', 'day'),
		});
	});
	it('complément daté du jour : le cours mensuel du mois de la date, même si l’exercice est au jour', () => {
		expect(
			openingProposal({ ...fy, openingRateKind: 'day' }, { dateKind: 'TODAY', date: '2026-03-15' }),
		).toEqual({ kind: 'month', date: '2026-03-01' });
		expect(openingProposal(fy, { dateKind: 'TODAY', date: null })).toBeNull();
	});
	it('exercice inconnu : aucune proposition', () => {
		expect(openingProposal(null)).toBeNull();
	});
});
