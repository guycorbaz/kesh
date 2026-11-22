// Story 40-7d (AC-2, AC-3) — libellés de l'écran Stripe : refus de l'import (colonne
// source_id, versement sans identifiant), genres et actions des propositions, écarts,
// motifs « à examiner », refus de l'acceptation ; codes d'écart et de motif contre leur
// source Rust.

import { describe, it, expect, vi } from 'vitest';
import { readFileSync } from 'node:fs';
import { join } from 'node:path';

vi.mock('$lib/shared/utils/i18n.svelte', () => ({
	i18nMsg: (key: string, fallback: string, args?: Record<string, string | number>) =>
		`${key}|` + (args ? fallback.replace(/\{\s*\$(\w+)\s*\}/g, (_, k) => String(args[k] ?? '')) : fallback),
}));

import {
	STRIPE_GAP_CODES,
	STRIPE_LINKED_OBJECT_COLUMNS,
	STRIPE_TO_REVIEW_CODES,
	stripeFailedLabel,
	stripeGapDetails,
	stripeGapLabel,
	stripeImportErrorLines,
	stripeLineErrorLabel,
	stripeProposalActionLabel,
	stripeProposalKindLabel,
	stripeToReviewLabel,
} from './stripe-labels';
import type { StripeProposal } from './stripe.types';

const apiError = (code: string, details?: Record<string, unknown>, message = `message ${code}`) => ({
	code,
	status: 422,
	message,
	details,
});

const tx = {
	id: 7,
	bankAccountId: 2,
	date: '2020-01-29',
	amount: '100.00',
	currency: 'CHF',
	providerType: 'charge',
	name: null,
	transactionId: 'txn_TEST0001',
	counterpartyEmail: null,
};

describe('stripe-labels — import', () => {
	it('source_id absent : la marche à suivre dans Stripe, sans liste de colonnes', () => {
		const lines = stripeImportErrorLines(
			apiError('STRIPE_CSV_INVALID', { reason: 'STRIPE_CSV_MISSING_COLUMNS', columns: [STRIPE_LINKED_OBJECT_COLUMNS] }),
		);
		expect(lines).toHaveLength(1);
		expect(lines[0]).toMatch(/^stripe-import-error-linked-object\|/);
		expect(lines[0]).toContain('ajoutez la colonne source_id');
	});

	it('d’autres colonnes manquent aussi : nommées, puis la marche à suivre', () => {
		const lines = stripeImportErrorLines(
			apiError('STRIPE_CSV_INVALID', {
				reason: 'STRIPE_CSV_MISSING_COLUMNS',
				columns: ['net', STRIPE_LINKED_OBJECT_COLUMNS],
			}),
		);
		expect(lines).toHaveLength(2);
		expect(lines[0]).toContain('Colonnes manquantes : net.');
		expect(lines[0]).not.toContain('source_id|charge_id');
		expect(lines[1]).toMatch(/^stripe-import-error-linked-object\|/);
	});

	it('lignes en erreur : une phrase par ligne ; versement sans identifiant → rapport des versements automatiques', () => {
		const lines = stripeImportErrorLines(
			apiError('STRIPE_CSV_INVALID', {
				reason: 'STRIPE_CSV_LINES',
				lines: [
					{ line: 3, field: 'automatic_payout_id', code: 'ID', value: '' },
					{ line: 4, field: 'net', code: 'NET_MISMATCH', value: '9.00' },
				],
			}),
		);
		expect(lines).toHaveLength(3);
		expect(lines[1]).toContain('Ligne 3, automatic_payout_id');
		expect(lines[1]).toContain('stripe-csv-line-payout-id-missing');
		expect(lines[2]).toContain('Ligne 4, net');
		expect(lines[2]).toContain('le net ne vaut pas le brut moins les frais');
	});

	it('un identifiant illisible ailleurs que le versement garde son libellé ordinaire', () => {
		expect(stripeLineErrorLabel({ code: 'ID', field: 'balance_transaction_id', value: '' })).toMatch(
			/^stripe-csv-line-id\|/,
		);
		expect(stripeLineErrorLabel({ code: 'ID', field: 'automatic_payout_id', value: 'po x' })).toMatch(
			/^stripe-csv-line-id\|/,
		);
	});

	it('chaque code d’erreur de ligne du lecteur (40-7a) a son libellé', () => {
		// Écrits en dur, relevés dans `crates/kesh-import/src/stripe.rs` (`StripeLineErrorCode::as_str`).
		for (const code of ['FIELD_COUNT', 'ID', 'DATE', 'CURRENCY', 'AMOUNT', 'NET_MISMATCH', 'PAYOUT_CURRENCY_MIXED']) {
			expect(stripeLineErrorLabel({ code, field: null, value: null })).toMatch(/^stripe-csv-line-/);
		}
	});

	it('autre refus (devise sans compte) : le message du serveur, qui nomme les devises', () => {
		expect(
			stripeImportErrorLines(apiError('PROVIDER_CURRENCY_ACCOUNT_MISSING', { currencies: ['GBP'] }, 'GBP sans compte')),
		).toEqual(['GBP sans compte']);
	});
});

describe('stripe-labels — propositions', () => {
	const link: StripeProposal = {
		type: 'link_entry',
		bankTransactionId: 7,
		journalEntryId: 40,
		transaction: tx,
		entry: { journalEntryId: 40, entryNumber: 12, entryDate: '2020-01-29', description: 'Vente ch_TEST0001', ids: ['ch_TEST0001'] },
	};
	const fee: StripeProposal = { type: 'fee', bankTransactionId: 8, feeAccountId: 60, transaction: tx };
	const payout: StripeProposal = {
		type: 'payout',
		bankTransactionId: 9,
		payoutBankAccountId: 3,
		payoutAccountLabel: 'Banque · Principal',
		transaction: tx,
	};

	it('genres : vente par son numéro d’écriture, frais, versement vers le compte de la revue', () => {
		expect(stripeProposalKindLabel(link)).toContain('Rattachement à la vente n° 12');
		expect(stripeProposalKindLabel(fee)).toMatch(/^stripe-kind-fee\|Frais$/);
		expect(stripeProposalKindLabel(payout)).toContain('Versement vers Banque · Principal');
	});

	it('actions : aucun « comptabiliser automatiquement », le compte de frais nommé', () => {
		const accounts = [{ id: 60, number: '6940', name: 'Frais Stripe', accountType: 'Expense', active: true, postable: true }];
		expect(stripeProposalActionLabel(link, accounts as never)).toContain('sans nouvelle écriture');
		expect(stripeProposalActionLabel(fee, accounts as never)).toContain('6940 — Frais Stripe');
		expect(stripeProposalActionLabel(payout, [])).toContain('vers Banque · Principal');
		for (const p of [link, fee, payout]) {
			expect(stripeProposalActionLabel(p, accounts as never)).not.toMatch(/automatiquement/i);
		}
	});
});

describe('stripe-labels — écarts et motifs', () => {
	it('écarts communs libellés pour Stripe (charge, vente)', () => {
		expect(stripeGapLabel({ code: 'receipt_without_sale' })).toContain('Charge sans vente');
		expect(stripeGapLabel({ code: 'sale_without_receipt' })).toContain('Vente sans charge');
		expect(stripeGapLabel({ code: 'amount_or_currency_mismatch' })).toContain('Montant ou devise différents');
		expect(
			stripeGapLabel({ code: 'amount_or_currency_mismatch', details: { alreadyLinked: true } }),
		).toContain('déjà rattachée à une autre charge');
	});

	it('écarts propres à Stripe : versement incomplet (attendu et trouvé), sans réception', () => {
		const l = stripeGapLabel({ code: 'payout_total_mismatch', details: { payoutId: 'po_1', expected: '-125.15', found: '-96.80' } });
		expect(l).toContain('font -96.80, pour un versement de -125.15');
		expect(stripeGapLabel({ code: 'payout_without_receipt' })).toMatch(/^stripe-gap-payout-without-receipt\|/);
	});

	it('détails d’un montant différent : attendu et trouvé (règle commune)', () => {
		expect(
			stripeGapDetails({
				code: 'amount_or_currency_mismatch',
				details: { expectedAmount: '200.00', expectedCurrency: 'EUR', foundAmount: '190.00', foundCurrency: 'EUR' },
			}),
		).toContain('Attendu 200.00 EUR, trouvé 190.00 EUR');
		expect(stripeGapDetails({ code: 'payout_without_receipt' })).toBeNull();
	});

	it('motifs : communs (clés provider-) et propres à Stripe, la catégorie nommée', () => {
		expect(stripeToReviewLabel({ code: 'ambiguous_sale' })).toMatch(/^provider-review-ambiguous-sale\|/);
		expect(stripeToReviewLabel({ code: 'fee_account_missing' })).toMatch(/^provider-review-fee-account-missing\|/);
		expect(stripeToReviewLabel({ code: 'other_category', details: { category: 'dispute' } })).toContain('(dispute)');
		expect(stripeToReviewLabel({ code: 'refund' })).toMatch(/^stripe-review-refund\|/);
	});

	it('chaque code d’écart et de motif a un libellé (aucun ne retombe sur le code brut)', () => {
		for (const code of STRIPE_GAP_CODES) expect(stripeGapLabel({ code })).toContain('|');
		for (const code of STRIPE_TO_REVIEW_CODES) expect(stripeToReviewLabel({ code })).toContain('|');
		expect(stripeGapLabel({ code: 'inconnu' })).toBe('inconnu');
		expect(stripeToReviewLabel({ code: 'inconnu' })).toBe('inconnu');
	});
});

/**
 * Les codes d'écart et de motif, ÉCRITS EN DUR ci-dessous (jamais lus depuis le module : un test
 * qui itérerait sur la table de production serait vert par construction), comparés aux
 * constantes `GAP_*` et `REVIEW_*` des sources Rust : celles de Stripe
 * (`crates/kesh-reconciliation/src/stripe.rs`) et celles de la revue commune qu'un compte
 * Stripe peut rendre (`provider_review.rs` : ni conversion ni retrait ni règle chez Stripe).
 */
describe('stripe-labels — les codes contre leur source Rust', () => {
	const ATTENDUS_ECARTS = [
		'amount_or_currency_mismatch',
		'payout_total_mismatch',
		'payout_without_receipt',
		'receipt_without_sale',
		'sale_without_receipt',
	];
	const ATTENDUS_MOTIFS = [
		'ambiguous_sale',
		'fee_account_missing',
		'other_category',
		'payout_account_missing',
		'payout_not_negative',
		'payout_reversal',
		'refund',
	];
	const constantes = (fichier: string, prefixe: 'GAP' | 'REVIEW'): string[] => {
		const source = readFileSync(join('../crates/kesh-reconciliation/src', fichier), 'utf-8');
		const re = new RegExp(`pub const ${prefixe}_[A-Z_]+: &str = "([a-z_]+)";`, 'g');
		return [...source.matchAll(re)].map((m) => m[1]);
	};
	// Motifs de la revue commune qu'aucun compte Stripe ne rend (conversions, retraits, règles :
	// `SPEC` de Stripe ne les active pas).
	const HORS_STRIPE = ['conversion_unpaired', 'withdrawal_foreign_currency', 'no_rule'];

	it('les écarts écrits en dur sont ceux des sources, et ceux du module', () => {
		const source = [...constantes('stripe.rs', 'GAP'), ...constantes('provider_review.rs', 'GAP')].sort();
		expect(source).toEqual(ATTENDUS_ECARTS);
		expect([...STRIPE_GAP_CODES].sort()).toEqual(ATTENDUS_ECARTS);
	});

	it('les motifs écrits en dur sont ceux des sources (hors conversions, retraits, règles), et ceux du module', () => {
		const source = [...constantes('stripe.rs', 'REVIEW'), ...constantes('provider_review.rs', 'REVIEW')]
			.filter((c) => !HORS_STRIPE.includes(c))
			.sort();
		expect(source).toEqual(ATTENDUS_MOTIFS);
		expect([...STRIPE_TO_REVIEW_CODES].sort()).toEqual(ATTENDUS_MOTIFS);
	});
});

describe('stripe-labels — refus de l’acceptation', () => {
	it('proposition périmée : la clé commune ; les autres délégués au rapprochement', () => {
		expect(stripeFailedLabel('PROVIDER_PROPOSAL_STALE')).toMatch(/^provider-failed-proposal-stale\|/);
		expect(stripeFailedLabel('BANK_ACCOUNT_CLOSED', { bankAccountId: 3 })).not.toContain('reconciliation-failed-unknown');
		expect(stripeFailedLabel('RATE_UNAVAILABLE', {})).not.toContain('reconciliation-failed-unknown');
		// Destination rejugée à l'acceptation (revue d'intégration de l'Epic 39, M2).
		expect(
			stripeFailedLabel('STRIPE_PAYOUT_ACCOUNT_INVALID', { bankAccountId: 3, reason: 'paypal' }),
		).toMatch(/^error-stripe-payout-account-invalid\|/);
	});
});
