/**
 * Helpers de sérialisation/désérialisation des query params de listing.
 *
 * Story 3.4 : permet la synchronisation bidirectionnelle entre l'état
 * du formulaire de filtres et l'URL (`$page.url.searchParams`), pour
 * supporter le partage de lien, le rafraîchissement, et le retour arrière.
 */

import type { Journal, JournalEntryListQuery, SortBy, SortDirection } from './journal-entries.types';

const VALID_SORT_BY: SortBy[] = ['EntryDate', 'EntryNumber', 'Journal', 'Description'];
const VALID_SORT_DIR: SortDirection[] = ['Asc', 'Desc'];
const VALID_JOURNAL: Journal[] = ['Achats', 'Ventes', 'Banque', 'Caisse', 'OD'];

/**
 * Sérialise un `JournalEntryListQuery` en `URLSearchParams`.
 * Les champs vides, null ou undefined sont omis pour garder l'URL propre.
 */
export function serializeQuery(query: JournalEntryListQuery): URLSearchParams {
	const params = new URLSearchParams();

	if (query.description && query.description.trim() !== '') {
		params.set('description', query.description.trim());
	}
	if (query.amountMin && query.amountMin.trim() !== '') {
		params.set('amountMin', query.amountMin.trim());
	}
	if (query.amountMax && query.amountMax.trim() !== '') {
		params.set('amountMax', query.amountMax.trim());
	}
	if (query.dateFrom && query.dateFrom.trim() !== '') {
		params.set('dateFrom', query.dateFrom.trim());
	}
	if (query.dateTo && query.dateTo.trim() !== '') {
		params.set('dateTo', query.dateTo.trim());
	}
	if (query.journal) {
		params.set('journal', query.journal);
	}
	// P4 : omet les défauts pour cohérence avec offset/limit.
	if (query.sortBy && query.sortBy !== 'EntryDate') {
		params.set('sortBy', query.sortBy);
	}
	if (query.sortDir && query.sortDir !== 'Desc') {
		params.set('sortDir', query.sortDir);
	}
	if (query.offset !== undefined && query.offset !== null && query.offset > 0) {
		params.set('offset', String(query.offset));
	}
	if (query.limit !== undefined && query.limit !== null && query.limit !== 50) {
		// Omet le défaut 50 pour garder l'URL propre.
		params.set('limit', String(query.limit));
	}

	return params;
}

/**
 * Reconstitue un `JournalEntryListQuery` depuis des `URLSearchParams`.
 * Les valeurs invalides ou inconnues sont ignorées silencieusement
 * (fallback sur les défauts côté consommateur).
 */
export function parseQueryFromUrl(searchParams: URLSearchParams): JournalEntryListQuery {
	const query: JournalEntryListQuery = {};

	const description = searchParams.get('description');
	if (description && description.trim() !== '') query.description = description;

	const amountMin = searchParams.get('amountMin');
	if (amountMin && amountMin.trim() !== '') query.amountMin = amountMin;

	const amountMax = searchParams.get('amountMax');
	if (amountMax && amountMax.trim() !== '') query.amountMax = amountMax;

	const dateFrom = searchParams.get('dateFrom');
	if (dateFrom && dateFrom.trim() !== '') query.dateFrom = dateFrom;

	const dateTo = searchParams.get('dateTo');
	if (dateTo && dateTo.trim() !== '') query.dateTo = dateTo;

	const journal = searchParams.get('journal');
	if (journal && (VALID_JOURNAL as string[]).includes(journal)) {
		query.journal = journal as Journal;
	}

	const sortBy = searchParams.get('sortBy');
	if (sortBy && (VALID_SORT_BY as string[]).includes(sortBy)) {
		query.sortBy = sortBy as SortBy;
	}

	const sortDir = searchParams.get('sortDir');
	if (sortDir && (VALID_SORT_DIR as string[]).includes(sortDir)) {
		query.sortDir = sortDir as SortDirection;
	}

	const offset = searchParams.get('offset');
	if (offset !== null) {
		const n = Number(offset);
		if (Number.isFinite(n) && n >= 0) query.offset = Math.floor(n);
	}

	const limit = searchParams.get('limit');
	if (limit !== null) {
		const n = Number(limit);
		if (Number.isFinite(n) && n > 0) query.limit = Math.floor(n);
	}

	return query;
}
