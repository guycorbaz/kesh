// Story 53-1a (#1355) — le module d'états de la saisie : prédicats sur tous les états,
// lecture du refus de doublon, état qui suit un rejet, lien de vérification.

import { describe, it, expect, vi } from 'vitest';

vi.mock('$lib/shared/utils/i18n.svelte', () => ({
	i18nMsg: (_key: string, fallback: string) => fallback
}));

import {
	ENTRY_SUBMIT_STATES,
	canConfirmFrom,
	canSendFrom,
	canWriteEntry,
	entryDuplicateText,
	isEntryFrozen,
	readEntryDuplicateRefusal,
	stateAfterRejection,
	unknownOutcomeHref,
	unknownOutcomeRangeHref,
	type EntrySubmitState
} from './entry-submit-state';
import { isSettlementOutcomeUnknown } from '$lib/shared/utils/unknown-outcome';

const DETAILS = { journalEntryId: 7, entryNumber: 12, entryDate: '2026-03-02', confirmable: true };
const doublon = (details: unknown = DETAILS) => ({
	code: 'JOURNAL_ENTRY_PROBABLE_DUPLICATE',
	status: 409,
	message: 'serveur',
	details
});

describe('prédicats d’état (table J)', () => {
	const vrai = (f: (s: EntrySubmitState) => boolean) => ENTRY_SUBMIT_STATES.filter(f);

	it('canWriteEntry : vrai en J1, J2a, J7 ; faux en J2b, J3, J5, J6 (M27 : J2a doit rester vrai)', () => {
		expect(vrai(canWriteEntry)).toEqual(['J1', 'J2a', 'J7']);
		expect(canWriteEntry('J2a')).toBe(true);
	});

	it('canSendFrom : J1 et J7 seulement', () => {
		expect(vrai(canSendFrom)).toEqual(['J1', 'J7']);
	});

	it('canConfirmFrom : J5 seulement', () => {
		expect(vrai(canConfirmFrom)).toEqual(['J5']);
	});

	it('isEntryFrozen : tout état sauf J1 et J7', () => {
		expect(vrai(isEntryFrozen)).toEqual(['J2a', 'J2b', 'J3', 'J5', 'J6']);
	});
});

describe('readEntryDuplicateRefusal', () => {
	it('lit les details du contrat', () => {
		expect(readEntryDuplicateRefusal(doublon())).toEqual({ ...DETAILS, entry: null });
	});

	it('lit le rang details.entry de la conversion (1 ou 2), null sinon (Story 53-1b)', () => {
		expect(readEntryDuplicateRefusal(doublon({ ...DETAILS, entry: 1 }))?.entry).toBe(1);
		expect(readEntryDuplicateRefusal(doublon({ ...DETAILS, entry: 2 }))?.entry).toBe(2);
		for (const entry of [0, 3, '1', 1.5, null, undefined]) {
			expect(readEntryDuplicateRefusal(doublon({ ...DETAILS, entry }))?.entry).toBeNull();
		}
	});

	it('null pour un autre code, des details absents ou illisibles', () => {
		expect(readEntryDuplicateRefusal({ code: 'ENTRY_UNBALANCED', status: 400, message: 'x' })).toBeNull();
		expect(readEntryDuplicateRefusal({ ...doublon(), details: undefined })).toBeNull();
		expect(readEntryDuplicateRefusal(doublon(null))).toBeNull();
		expect(readEntryDuplicateRefusal(doublon({ ...DETAILS, journalEntryId: '7' }))).toBeNull();
		expect(readEntryDuplicateRefusal(doublon({ ...DETAILS, entryNumber: 1.5 }))).toBeNull();
		expect(readEntryDuplicateRefusal(doublon({ ...DETAILS, entryDate: '02.03.2026' }))).toBeNull();
		expect(readEntryDuplicateRefusal(doublon({ ...DETAILS, confirmable: 'oui' }))).toBeNull();
		expect(readEntryDuplicateRefusal(new Error('boom'))).toBeNull();
	});
});

describe('stateAfterRejection', () => {
	it('doublon lisible et confirmable → J5', () => {
		expect(stateAfterRejection(doublon(), isSettlementOutcomeUnknown)).toBe('J5');
	});

	it('doublon non confirmable ou illisible → J7 (message du serveur)', () => {
		expect(stateAfterRejection(doublon({ ...DETAILS, confirmable: false }), isSettlementOutcomeUnknown)).toBe('J7');
		expect(stateAfterRejection(doublon(null), isSettlementOutcomeUnknown)).toBe('J7');
	});

	it('issue inconnue (coupure, délai, 5xx, hors ApiError) → J6', () => {
		for (const e of [
			{ code: 'NETWORK_ERROR', status: 0, message: 'x' },
			{ code: 'TIMEOUT', status: 0, message: 'x' },
			{ code: 'INTERNAL_ERROR', status: 500, message: 'x' },
			{ code: 'X', status: 504, message: 'x' },
			new Error('boom')
		]) {
			expect(stateAfterRejection(e, isSettlementOutcomeUnknown)).toBe('J6');
		}
	});

	it('needsEntry (la conversion) : un rang illisible → J7, un rang lisible → J5 ; sans l’option, inchangé', () => {
		const sansRang = doublon();
		const avecRang = doublon({ ...DETAILS, entry: 2 });
		expect(stateAfterRejection(sansRang, isSettlementOutcomeUnknown, { needsEntry: true })).toBe('J7');
		expect(stateAfterRejection(avecRang, isSettlementOutcomeUnknown, { needsEntry: true })).toBe('J5');
		expect(stateAfterRejection(sansRang, isSettlementOutcomeUnknown)).toBe('J5');
		expect(stateAfterRejection(sansRang, isSettlementOutcomeUnknown, {})).toBe('J5');
		// L'issue inconnue n'est pas touchée par l'option.
		expect(
			stateAfterRejection({ code: 'NETWORK_ERROR', status: 0, message: 'x' }, isSettlementOutcomeUnknown, {
				needsEntry: true
			})
		).toBe('J6');
	});

	it('refus ordinaire → J7', () => {
		expect(stateAfterRejection({ code: 'ENTRY_UNBALANCED', status: 400, message: 'x' }, isSettlementOutcomeUnknown)).toBe('J7');
	});
});

describe('textes et liens', () => {
	it('le texte du doublon est celui du catalogue (repli français du serveur)', () => {
		expect(entryDuplicateText()).toBe(
			'Une écriture identique existe déjà à cette date (même journal, même libellé, mêmes comptes et montants). Si c’est la même, ne l’enregistrez pas une seconde fois ; sinon, confirmez que ce n’est pas un doublon.'
		);
	});

	it('le lien de J6 filtre le journal sur le jour de la saisie', () => {
		expect(unknownOutcomeHref('2026-03-02')).toBe('/journal-entries?dateFrom=2026-03-02&dateTo=2026-03-02');
	});

	it('le lien de K6 filtre le journal sur la plage des deux dates, dans l’ordre (Story 53-1b)', () => {
		expect(unknownOutcomeRangeHref('2026-03-05', '2026-03-09')).toBe(
			'/journal-entries?dateFrom=2026-03-05&dateTo=2026-03-09'
		);
		expect(unknownOutcomeRangeHref('2026-03-09', '2026-03-05')).toBe(
			'/journal-entries?dateFrom=2026-03-05&dateTo=2026-03-09'
		);
		expect(unknownOutcomeRangeHref('2026-03-05', '2026-03-05')).toBe(
			'/journal-entries?dateFrom=2026-03-05&dateTo=2026-03-05'
		);
	});
});
