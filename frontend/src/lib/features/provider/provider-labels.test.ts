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

import {
	PROVIDER_COMMON_FAILED_CODES,
	accountNumberName,
	alreadyPostedEntryId,
	providerFailedLabel,
} from './provider-labels';

/**
 * Les codes de refus que le moteur commun peut poser dans `failed[]`, ÉCRITS EN DUR (jamais lus
 * depuis le module : un test qui itérerait sur la table de production serait vert par
 * construction). Relevé : les littéraux de `crates/kesh-api/src/routes/provider_accept.rs`
 * hors tests (la constante `PROVIDER_PROPOSAL_STALE` comprise), ceux des sept fonctions qu'il
 * importe de `reconciliation.rs` et de `fx_db_refusal`, et trois codes de `DbError::error_code()`.
 */
const CODES_MOTEUR = [
	'ACCOUNT_NOT_FOUND',
	'ACCOUNT_NOT_POSTABLE',
	'ACCOUNT_REQUIRES_CURRENCY',
	'BANK_ACCOUNT_CLOSED',
	'BANK_ACCOUNT_NOT_CONFIGURED',
	'BANK_TRANSACTION_NOT_FOUND',
	'COUNTERPARTY_CURRENCY_MISMATCH',
	'DATABASE_ERROR',
	'ENTRY_IS_FX_REVALUATION',
	'FX_ACCOUNT_INVALID',
	'FX_ACCOUNT_IN_CHF',
	'FX_CURRENCY_MISMATCH',
	'FX_MULTIPLE_CURRENCIES',
	'FX_RATE_INVALID',
	'FX_RATE_NOT_AFC',
	'LATER_FISCAL_YEAR_CLOSED',
	'MULTI_CURRENCY_DISABLED',
	'NOT_FOUND',
	'OPTIMISTIC_LOCK_CONFLICT',
	'PERIOD_LOCKED',
	'PROJECT_ARCHIVED',
	'PROJECT_NOT_FOUND',
	'PROVIDER_PROPOSAL_STALE',
	'PROVIDER_TRANSACTION_ALREADY_POSTED',
	'RATE_UNAVAILABLE',
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
				'PROVIDER_TRANSACTION_ALREADY_POSTED',
				'RATE_UNAVAILABLE',
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
		// Codes que le moteur reçoit des sept fonctions qu'il importe de `reconciliation.rs` (revue P2,
		// P2-5 ; revue P3, P3-4 : les sept, non deux). La liste des fonctions est relue à l'import : une
		// huitième ferait rougir ce test.
		const reco = readFileSync('../crates/kesh-api/src/routes/reconciliation.rs', 'utf-8');
		const erreurs = readFileSync('../crates/kesh-api/src/errors.rs', 'utf-8');
		const corps = (src: string, nom: string) => {
			const debut = src.indexOf(`fn ${nom}(`);
			expect(debut).toBeGreaterThan(-1);
			return src.slice(debut, src.indexOf('\n}\n', debut));
		};
		const importees = horsTests
			.slice(horsTests.indexOf('use super::reconciliation::{'))
			.split('};')[0]
			.match(/\b[a-z_]+\b/g)!
			.filter((n) => n !== 'use' && n !== 'super' && n !== 'reconciliation');
		expect(importees.sort()).toEqual([
			'closed_account_failed',
			'counterparty_is_bank_ledger_failed_proposal',
			'fx_failed_proposal',
			'later_fiscal_year_closed_failed_proposal',
			'non_postable_failed_proposal',
			'period_locked_failed_proposal',
			'project_error_to_failed_proposal',
		]);
		// Les littéraux des corps, plus ceux de `fx_db_refusal` (`errors.rs`), vers qui
		// `fx_failed_proposal` renvoie les refus du contrôle des devises.
		const autres =
			importees.map((n) => corps(reco, n)).join('\n') + corps(erreurs, 'fx_db_refusal');
		expect(corps(reco, 'fx_failed_proposal')).toContain('fx_db_refusal(');
		// Trois codes ne sont pas des littéraux de ces corps : ils sortent de `DbError::error_code()`
		// (`non_postable_failed_proposal`, et `fx_failed_proposal` pour les deux variantes qu'elle
		// traite elle-même). Écrits en dur, et cherchés dans la table de `kesh-db`.
		const parErrorCode: Record<string, string> = {
			ACCOUNT_NOT_POSTABLE: 'AccountsNotPostable',
			MULTI_CURRENCY_DISABLED: 'MultiCurrencyDisabled',
			RATE_UNAVAILABLE: 'RateUnavailable',
		};
		const tableDb = readFileSync('../crates/kesh-db/src/errors.rs', 'utf-8');
		for (const [code, variante] of Object.entries(parErrorCode)) {
			expect(tableDb).toMatch(new RegExp(`Self::${variante}\\b[^=]*=> "${code}"`));
		}
		expect(corps(reco, 'non_postable_failed_proposal')).toContain('AccountsNotPostable(list).error_code()');
		// Le refus d'une transaction déjà comptabilisée est la constante `FAILED_ALREADY_POSTED` de
		// `kesh-reconciliation` (correctif des prestataires), que le moteur importe et pose : non
		// un littéral du fichier, relue dans sa source.
		const revue = readFileSync('../crates/kesh-reconciliation/src/provider_review.rs', 'utf-8');
		const constante = revue.match(/pub const FAILED_ALREADY_POSTED: &str = "([A-Z_]+)";/);
		expect(constante).not.toBeNull();
		expect(horsTests).toMatch(/\bFAILED_ALREADY_POSTED,\n/);
		const trouves = new Set([
			...[...(horsTests + autres).matchAll(/"([A-Z][A-Z_]{4,})"/g)].map((m) => m[1]),
			...Object.keys(parErrorCode),
			constante![1],
		]);
		expect([...trouves].sort()).toEqual(CODES_MOTEUR);
	});

	it('aucun code du moteur ne retombe sur « Refus non reconnu » ', () => {
		for (const code of CODES_MOTEUR) {
			expect(providerFailedLabel(code, {})).not.toContain('reconciliation-failed-unknown');
		}
	});
});

describe('provider-labels — l’écriture déjà passée d’un motif « à examiner »', () => {
	it('already_posted : l’écriture ; tout autre motif ou sans identifiant : null', () => {
		expect(alreadyPostedEntryId({ code: 'already_posted', details: { journalEntryId: 42 } })).toBe(42);
		expect(alreadyPostedEntryId({ code: 'no_rule', details: { journalEntryId: 42 } })).toBeNull();
		expect(alreadyPostedEntryId({ code: 'already_posted', details: { journalEntryId: '42' } })).toBeNull();
		expect(alreadyPostedEntryId({ code: 'already_posted' })).toBeNull();
	});
});

/**
 * Story 39-11d2 (AC-4) — le cours mensuel retenu manque : un libellé propre aux prestataires, qui
 * renvoie aux cours de change, le mois en forme suisse ; jamais le libellé générique, qui dit de
 * garder la ligne comme cours saisi (sans objet pour une proposition).
 */
describe('provider-labels — cours mensuel manquant', () => {
	it('la devise et le mois en MM.AAAA, la marche à suivre', () => {
		const label = providerFailedLabel('RATE_UNAVAILABLE', { currency: 'USD', month: '2020-03-01' });
		expect(label).toBe(
			'provider-failed-rate-unavailable|Aucun cours mensuel USD retenu pour 03.2020 : récupérez ou saisissez ce cours dans les cours de change, puis acceptez de nouveau.',
		);
		expect(label).not.toContain('cours saisi');
	});

	it('détails absents ou vides : la phrase, sans valeur brute', () => {
		for (const details of [undefined, null, {}, { month: 7 }]) {
			const label = providerFailedLabel('RATE_UNAVAILABLE', details);
			expect(label).toMatch(/^provider-failed-rate-unavailable\|Aucun cours mensuel {2}retenu pour  :/);
		}
	});
});

describe('provider-labels — nom d’un compte', () => {
	it('« numéro — nom », ou l’identifiant faute du compte', () => {
		const accounts = [{ id: 4, number: '6940', name: 'Frais' }] as never;
		expect(accountNumberName(accounts, 4)).toBe('6940 — Frais');
		expect(accountNumberName(accounts, 9)).toBe('#9');
	});
});
