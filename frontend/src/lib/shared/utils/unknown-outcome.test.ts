// Story 32-1b (#711 E-7, AC-b2) — le prédicat d'issue inconnue, extrait
// d'`AdminRestorePanel.svelte` et partagé avec « Sauvegarder maintenant ».
import { describe, it, expect } from 'vitest';
import { isSettlementOutcomeUnknown, isUnknownOutcome } from './unknown-outcome';

// Revue P2 de la 33-6 (P2-2) : un règlement n'est pas idempotent, et un 5xx peut
// tomber à l'instant du commit.
describe('isSettlementOutcomeUnknown', () => {
	for (const err of [
		{ code: 'TIMEOUT', status: 0, message: 'délai' },
		{ code: 'NETWORK_ERROR', status: 0, message: 'coupé' },
		{ code: 'INTERNAL_ERROR', status: 500, message: 'Erreur interne' },
		{ code: 'UNKNOWN_ERROR', status: 502, message: 'Bad Gateway' },
		{ code: 'SERVICE_UNAVAILABLE', status: 503, message: 'Base injoignable' },
		{ code: 'UNKNOWN_ERROR', status: 504, message: 'Gateway Timeout' },
		new TypeError('réponse illisible'),
		undefined,
	]) {
		it(`vrai : ${JSON.stringify(err) ?? String(err)}`, () => {
			expect(isSettlementOutcomeUnknown(err)).toBe(true);
		});
	}

	for (const err of [
		{ code: 'VALIDATION_ERROR', status: 400, message: 'Trop-perçu' },
		{ code: 'FORBIDDEN', status: 403, message: 'Accès refusé' },
		{ code: 'OPTIMISTIC_LOCK_CONFLICT', status: 409, message: 'Conflit' },
		{ code: 'X', status: 499, message: 'borne' },
	]) {
		it(`faux, un refus certain : ${err.code} ${err.status}`, () => {
			expect(isSettlementOutcomeUnknown(err)).toBe(false);
		});
	}
});

describe('isUnknownOutcome', () => {
	for (const err of [
		{ code: 'TIMEOUT', status: 0, message: 'délai' },
		{ code: 'NETWORK_ERROR', status: 0, message: 'coupé' },
		{ code: 'UNKNOWN_ERROR', status: 502, message: 'Bad Gateway' },
		{ code: 'UNKNOWN_ERROR', status: 504, message: 'Gateway Timeout' },
	]) {
		it(`vrai : ${err.code} ${err.status}`, () => {
			expect(isUnknownOutcome(err)).toBe(true);
		});
	}

	for (const err of [
		{ code: 'INTERNAL_ERROR', status: 500, message: 'Erreur interne' },
		{ code: 'SERVICE_UNAVAILABLE', status: 503, message: 'Base injoignable' },
		{ code: 'FORBIDDEN', status: 403, message: 'Accès refusé' },
		{ code: 'BACKUP_IN_PROGRESS', status: 409, message: 'déjà en cours' },
	]) {
		it(`faux : ${err.code} ${err.status}`, () => {
			expect(isUnknownOutcome(err)).toBe(false);
		});
	}

	it('faux pour ce qui n’est pas une ApiError', () => {
		expect(isUnknownOutcome(new TypeError('x'))).toBe(false);
		expect(isUnknownOutcome(undefined)).toBe(false);
		expect(isUnknownOutcome({ status: 504 })).toBe(false);
	});
});
