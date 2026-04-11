import { describe, expect, it } from 'vitest';
import { parseQueryFromUrl, serializeQuery } from './query-helpers';
import type { JournalEntryListQuery } from './journal-entries.types';

describe('serializeQuery', () => {
	it('sérialise un query vide en params vides', () => {
		const params = serializeQuery({});
		expect(params.toString()).toBe('');
	});

	it('sérialise les champs non vides (avec tri non-défaut)', () => {
		const q: JournalEntryListQuery = {
			description: 'facture',
			amountMin: '100',
			amountMax: '500',
			dateFrom: '2026-01-01',
			dateTo: '2026-12-31',
			journal: 'Banque',
			// Valeurs NON-défaut pour vérifier qu'elles sont conservées
			// (le défaut EntryDate/Desc est testé séparément ci-dessous).
			sortBy: 'EntryNumber',
			sortDir: 'Asc',
			offset: 50,
			limit: 25
		};
		const params = serializeQuery(q);
		expect(params.get('description')).toBe('facture');
		expect(params.get('amountMin')).toBe('100');
		expect(params.get('amountMax')).toBe('500');
		expect(params.get('dateFrom')).toBe('2026-01-01');
		expect(params.get('dateTo')).toBe('2026-12-31');
		expect(params.get('journal')).toBe('Banque');
		expect(params.get('sortBy')).toBe('EntryNumber');
		expect(params.get('sortDir')).toBe('Asc');
		expect(params.get('offset')).toBe('50');
		expect(params.get('limit')).toBe('25');
	});

	it('omet les champs vides et whitespace-only', () => {
		const q: JournalEntryListQuery = {
			description: '',
			amountMin: '   ',
			amountMax: undefined
		};
		const params = serializeQuery(q);
		expect(params.toString()).toBe('');
	});

	it('omet offset=0 et limit=50 (défauts)', () => {
		const q: JournalEntryListQuery = { offset: 0, limit: 50 };
		const params = serializeQuery(q);
		expect(params.toString()).toBe('');
	});

	it('P4 : omet sortBy=EntryDate et sortDir=Desc (défauts)', () => {
		const q: JournalEntryListQuery = { sortBy: 'EntryDate', sortDir: 'Desc' };
		const params = serializeQuery(q);
		expect(params.toString()).toBe('');
	});

	it('conserve sortBy si différent du défaut', () => {
		const q: JournalEntryListQuery = { sortBy: 'EntryNumber', sortDir: 'Desc' };
		const params = serializeQuery(q);
		expect(params.get('sortBy')).toBe('EntryNumber');
		expect(params.has('sortDir')).toBe(false);
	});

	it('conserve sortDir si différent du défaut', () => {
		const q: JournalEntryListQuery = { sortBy: 'EntryDate', sortDir: 'Asc' };
		const params = serializeQuery(q);
		expect(params.has('sortBy')).toBe(false);
		expect(params.get('sortDir')).toBe('Asc');
	});

	it('conserve offset > 0 et limit != 50', () => {
		const q: JournalEntryListQuery = { offset: 100, limit: 25 };
		const params = serializeQuery(q);
		expect(params.get('offset')).toBe('100');
		expect(params.get('limit')).toBe('25');
	});

	it('trim les strings avant sérialisation', () => {
		const q: JournalEntryListQuery = { description: '  facture  ' };
		const params = serializeQuery(q);
		expect(params.get('description')).toBe('facture');
	});
});

describe('parseQueryFromUrl', () => {
	it('parse les champs valides', () => {
		const search = new URLSearchParams(
			'description=facture&amountMin=100&sortBy=EntryDate&sortDir=Asc&offset=25&limit=50'
		);
		const q = parseQueryFromUrl(search);
		expect(q.description).toBe('facture');
		expect(q.amountMin).toBe('100');
		expect(q.sortBy).toBe('EntryDate');
		expect(q.sortDir).toBe('Asc');
		expect(q.offset).toBe(25);
		expect(q.limit).toBe(50);
	});

	it('ignore les enums invalides', () => {
		const search = new URLSearchParams('sortBy=InvalidValue&journal=UnknownJournal&sortDir=xxx');
		const q = parseQueryFromUrl(search);
		expect(q.sortBy).toBeUndefined();
		expect(q.journal).toBeUndefined();
		expect(q.sortDir).toBeUndefined();
	});

	it('ignore offset/limit négatifs ou non numériques', () => {
		const search = new URLSearchParams('offset=-10&limit=abc');
		const q = parseQueryFromUrl(search);
		expect(q.offset).toBeUndefined();
		expect(q.limit).toBeUndefined();
	});

	it('parse un Journal valide', () => {
		const search = new URLSearchParams('journal=Banque');
		const q = parseQueryFromUrl(search);
		expect(q.journal).toBe('Banque');
	});

	it('retourne un objet vide pour une URL sans params', () => {
		const search = new URLSearchParams();
		const q = parseQueryFromUrl(search);
		expect(q).toEqual({});
	});
});

describe('roundtrip serializeQuery ↔ parseQueryFromUrl', () => {
	it('préserve tous les champs valides (non-défaut)', () => {
		const original: JournalEntryListQuery = {
			description: 'test',
			amountMin: '100',
			amountMax: '500',
			dateFrom: '2026-01-01',
			dateTo: '2026-12-31',
			journal: 'Banque',
			sortBy: 'EntryNumber',
			sortDir: 'Asc',
			offset: 50,
			limit: 25
		};
		const params = serializeQuery(original);
		const roundtripped = parseQueryFromUrl(params);
		expect(roundtripped).toEqual(original);
	});

	it('roundtrip d\'un query vide', () => {
		const params = serializeQuery({});
		const roundtripped = parseQueryFromUrl(params);
		expect(roundtripped).toEqual({});
	});
});
