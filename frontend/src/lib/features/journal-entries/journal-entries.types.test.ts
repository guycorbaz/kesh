// Story 53-1a (D5, M24) — `confirmNotDuplicate` ne fuit ni vers le `PUT` ni vers les
// **entrées** de la conversion (`entries[i]`) ; le **corps** de la conversion le porte
// (Story 53-1b). Les directives `@ts-expect-error` sont vérifiées par `npm run check` : si
// le champ entrait dans `CreateJournalEntryRequest` (interface commune), elles ne
// signaleraient plus d'erreur, et le contrôle échouerait.

import { describe, it, expect } from 'vitest';
import type {
	CreateConversionRequest,
	CreateJournalEntryBody,
	CreateJournalEntryRequest,
	UpdateJournalEntryRequest
} from './journal-entries.types';

const entree: CreateJournalEntryRequest = {
	entryDate: '2026-03-02',
	journal: 'OD',
	description: 'x',
	lines: []
};

describe('corps de la saisie', () => {
	it('le corps de la saisie et celui de la conversion portent le champ ; le PUT et les entrées de la conversion non', () => {
		const saisie: CreateJournalEntryBody = { ...entree, confirmNotDuplicate: true };

		const put: UpdateJournalEntryRequest = {
			...entree,
			version: 1,
			// @ts-expect-error — le PUT n'a pas `confirmNotDuplicate`
			confirmNotDuplicate: true
		};

		const conversion: CreateConversionRequest = {
			confirmNotDuplicate: true,
			entries: [
				// @ts-expect-error — une entrée de conversion n'a pas `confirmNotDuplicate`
				{ ...entree, confirmNotDuplicate: true },
				entree
			]
		};

		expect(saisie.confirmNotDuplicate).toBe(true);
		expect(put.version).toBe(1);
		expect(conversion.entries).toHaveLength(2);
		expect(conversion.confirmNotDuplicate).toBe(true);
	});
});
