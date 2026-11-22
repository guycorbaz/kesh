/**
 * Revue P1 de la Story 40-7d (M2) — les refus du moteur d'acceptation commun sont traduits pour
 * tous les prestataires, et la liste des codes est comparée à la source Rust.
 */

import { describe, it, expect, vi } from 'vitest';
import { readFileSync } from 'node:fs';

vi.mock('$lib/shared/utils/i18n.svelte', () => ({
	i18nMsg: (key: string, fallback: string, args?: Record<string, string | number>) =>
		`${key}|` + (args ? fallback.replace(/\{\s*\$(\w+)\s*\}/g, (_, k) => String(args[k] ?? '')) : fallback),
}));

import { PROVIDER_COMMON_FAILED_CODES, providerFailedLabel } from './provider-labels';

/**
 * Les codes de refus que le moteur commun peut poser dans `failed[]`, ÉCRITS EN DUR (jamais lus
 * depuis le module : un test qui itérerait sur la table de production serait vert par
 * construction). Relevé : les littéraux de `crates/kesh-api/src/routes/provider_accept.rs`
 * hors tests, plus la constante `PROVIDER_PROPOSAL_STALE`.
 */
const CODES_MOTEUR = [
	'ACCOUNT_NOT_FOUND',
	'BANK_ACCOUNT_CLOSED',
	'BANK_ACCOUNT_NOT_CONFIGURED',
	'BANK_TRANSACTION_NOT_FOUND',
	'COUNTERPARTY_CURRENCY_MISMATCH',
	'DATABASE_ERROR',
	'FX_ACCOUNT_INVALID',
	'FX_RATE_INVALID',
	'NOT_FOUND',
	'OPTIMISTIC_LOCK_CONFLICT',
	'PROVIDER_PROPOSAL_STALE',
	'RECONCILIATION_ALREADY_RECONCILED',
	'RECONCILIATION_ENTRY_NOT_LINKABLE',
	'RECONCILIATION_FISCAL_YEAR_CLOSED',
	'RECONCILIATION_TRANSACTION_NOT_PENDING',
	'VALIDATION_ERROR',
];

describe('provider-labels — les refus du moteur commun', () => {
	it('chaque code commun a une clé provider-failed-*', () => {
		expect([...PROVIDER_COMMON_FAILED_CODES].sort()).toEqual(
			[
				'COUNTERPARTY_CURRENCY_MISMATCH',
				'FX_ACCOUNT_INVALID',
				'FX_RATE_INVALID',
				'NOT_FOUND',
				'OPTIMISTIC_LOCK_CONFLICT',
				'PROVIDER_PROPOSAL_STALE',
				'RECONCILIATION_ENTRY_NOT_LINKABLE',
			],
		);
		for (const code of PROVIDER_COMMON_FAILED_CODES) {
			expect(providerFailedLabel(code)).toMatch(/^provider-failed-[a-z-]+\|/);
		}
	});

	it('les codes écrits en dur sont ceux de la source Rust (hors tests)', () => {
		const source = readFileSync('../crates/kesh-api/src/routes/provider_accept.rs', 'utf-8');
		const horsTests = source.slice(0, source.indexOf('#[cfg(test)]'));
		// Codes que le moteur reçoit de fonctions de `reconciliation.rs` (revue P2, P2-5) : le refus
		// d'un compte clos (`BANK_ACCOUNT_CLOSED`) et celui d'une contrepartie qui est le compte bancaire.
		const reco = readFileSync('../crates/kesh-api/src/routes/reconciliation.rs', 'utf-8');
		const corps = (nom: string) => {
			const debut = reco.indexOf(`fn ${nom}(`);
			expect(debut).toBeGreaterThan(-1);
			return reco.slice(debut, reco.indexOf('\n}\n', debut));
		};
		const autres = corps('closed_account_failed') + corps('counterparty_is_bank_ledger_failed_proposal');
		const trouves = new Set(
			[...(horsTests + autres).matchAll(/"([A-Z][A-Z_]{4,})"/g)].map((m) => m[1]),
		);
		expect([...trouves].sort()).toEqual(CODES_MOTEUR);
	});

	it('aucun code du moteur ne retombe sur « Refus non reconnu » ', () => {
		for (const code of CODES_MOTEUR) {
			expect(providerFailedLabel(code, {})).not.toContain('reconciliation-failed-unknown');
		}
	});
});
