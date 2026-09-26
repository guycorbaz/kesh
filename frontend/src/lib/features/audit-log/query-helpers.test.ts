import { describe, it, expect } from 'vitest';
import { filtersOnly, parseQueryFromUrl, serializeQuery } from './query-helpers';

describe('query-helpers du journal d’audit', () => {
	it('aller-retour URL', () => {
		const q = {
			dateFrom: '2026-09-01',
			dateTo: '2026-09-16',
			entityType: 'contact',
			entityId: 12,
			action: 'contact.created',
			offset: 50,
			limit: 20,
		};
		expect(parseQueryFromUrl(serializeQuery(q))).toEqual(q);
	});

	it('omet les valeurs par défaut et vides', () => {
		expect(serializeQuery({ offset: 0, limit: 50, action: '' }).toString()).toBe('');
	});

	it('ignore les valeurs invalides', () => {
		const q = parseQueryFromUrl(
			new URLSearchParams(
				'dateFrom=16.09.2026&dateTo=2026-9-1&entityType=contact&entityId=abc&offset=-3&limit=0',
			),
		);
		expect(q).toEqual({ entityType: 'contact' });
		expect(
			parseQueryFromUrl(new URLSearchParams('entityType=contact&entityId=0')).entityId,
		).toBeUndefined();
	});

	it('conserve entityType et action hors vocabulaire (code historique)', () => {
		const q = parseQueryFromUrl(new URLSearchParams('entityType=zz_old&action=zz.legacy'));
		expect(q).toEqual({ entityType: 'zz_old', action: 'zz.legacy' });
	});

	it('⛔ un entityId SANS entityType est ignoré à la lecture et omis à la sérialisation', () => {
		expect(parseQueryFromUrl(new URLSearchParams('entityId=7'))).toEqual({});
		expect(serializeQuery({ entityId: 7 }).has('entityId')).toBe(false);
		expect(filtersOnly({ entityId: 7 })).toEqual({});
	});
});
