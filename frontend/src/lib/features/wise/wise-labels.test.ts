// Story 39-11d (AC-2) — libellés de l'écran Wise : chaque refus du lecteur (39-11a) se dit par
// sa cause, le fichier et son archive nommés ; le relevé sans les frais séparés (39-11e) porte
// la consigne ; un refus non propre à Wise garde le message du serveur ; l'avertissement de
// continuité et la ligne du compte rendu. Story 39-11d2 (AC-2 à AC-4) — la revue : genre et
// « ce qui sera fait », une phrase par motif et par écart (listes contre la source Rust), refus
// de l'acceptation (raisons de PROVIDER_PROPOSAL_STALE contre leurs littéraux).

import { describe, it, expect, vi } from 'vitest';
import { readFileSync } from 'node:fs';

vi.mock('$lib/shared/utils/i18n.svelte', () => ({
	i18nMsg: (_key: string, fallback: string, args?: Record<string, string | number>) =>
		args ? fallback.replace(/\{\s*\$(\w+)\s*\}/g, (_, k) => String(args[k] ?? '')) : fallback,
}));

import {
	WISE_CARD_GROUP_UNSUPPORTED_REASONS,
	WISE_GAP_CODES,
	WISE_LINES_SHOWN,
	WISE_STALE_EXCLUDED_REASONS,
	WISE_STALE_RELOAD_REASONS,
	WISE_TO_REVIEW_CODES,
	wiseCardGroupReasonLabel,
	wiseFailedLabel,
	wiseGapLabel,
	wiseLineLabel,
	wisePartlyReconciledEntryIds,
	wiseProposalActionLabel,
	wiseProposalKindLabel,
	wiseToReviewLabel,
	wiseFileReportLine,
	wiseImportErrorLines,
	wiseLineErrorLabel,
	wiseLinesList,
	wiseWarningLabel,
} from './wise-labels';
import type { WiseProposal } from './wise.types';

const refus = (code: string, details: Record<string, unknown>, message = 'message du serveur') => ({
	code,
	status: 422,
	message,
	details,
});
const invalid = (reason: string, more: Record<string, unknown> = {}) =>
	refus('WISE_CSV_INVALID', { filename: 'chf.csv', reason, ...more });

describe('wiseImportErrorLines (Story 39-11d)', () => {
	it('frais non séparés : le fichier et son zip, la consigne, les lignes bornées', () => {
		const lines = Array.from({ length: 12 }, (_, i) => i + 2);
		const out = wiseImportErrorLines(
			refus('WISE_CSV_FEES_NOT_SEPARATED', { filename: 'chf.csv', archive: 'r.zip', lines, count: 40 }),
		);
		expect(out[0]).toContain('« chf.csv » (archive « r.zip »)');
		expect(out[0]).toContain('exporté sans les frais séparés');
		expect(out[1]).toContain('« Afficher les transactions dont les frais sont indiqués séparément »');
		expect(out[2]).toBe(`40 opération(s) à frais sans leur ligne de frais (lignes ${lines.slice(0, WISE_LINES_SHOWN).join(', ')}…).`);
	});

	it('frais non séparés d’un fichier envoyé à part, sans lignes : deux phrases, sans archive', () => {
		const out = wiseImportErrorLines(refus('WISE_CSV_FEES_NOT_SEPARATED', { filename: 'eur.csv', archive: null }));
		expect(out).toHaveLength(2);
		expect(out[0]).toContain('« eur.csv » a été');
		expect(out[0]).not.toContain('archive');
	});

	it('deux fichiers d’une même devise, nommés', () => {
		expect(wiseImportErrorLines(refus('WISE_DUPLICATE_CURRENCY_FILE', { currency: 'USD', filenames: ['a.csv', 'b.csv'] }))).toEqual([
			'Deux fichiers portent le relevé en USD (a.csv, b.csv) : un seul fichier par devise. Rien n’a été importé.',
		]);
	});

	it.each([
		['WISE_CSV_MISSING_COLUMNS', { columns: ['Running Balance', 'ID'] }, 'colonnes manquantes (Running Balance, ID)'],
		['WISE_CSV_DUPLICATE_COLUMN', { columns: ['Amount'] }, 'colonne en double : Amount'],
		['WISE_CSV_EMPTY', {}, 'est vide'],
		['WISE_CSV_ENCODING', { cause: 'x' }, 'L’encodage du fichier « chf.csv »'],
		['WISE_CSV_MIXED_CURRENCIES', { currencies: ['CHF', 'EUR'] }, 'plusieurs devises (CHF, EUR)'],
		['WISE_CSV_DUPLICATE_ID', { id: 'CARD-9', lines: [4, 9] }, 'deux fois l’identifiant CARD-9 dans le même sens (lignes 4, 9)'],
		['WISE_CSV_RUNNING_BALANCE_BREAK', { line: 6, expected: '10.00', found: '12.00' }, 'à la ligne 6, le solde courant vaut 12.00 au lieu de 10.00'],
	])('relevé refusé %s : la cause, le fichier nommé', (reason, more, attendu) => {
		const out = wiseImportErrorLines(invalid(reason, more));
		expect(out).toHaveLength(1);
		expect(out[0]).toContain(attendu);
		expect(out[0]).toContain('« chf.csv »');
	});

	// Revue P2 L3 : le serveur ajoute `archive` aux `details` de toute raison de
	// `WISE_CSV_INVALID` ; chaque phrase nomme alors le zip d'où vient le relevé.
	it.each([
		['WISE_CSV_MISSING_COLUMNS', { columns: ['ID'] }],
		['WISE_CSV_DUPLICATE_COLUMN', { columns: ['Amount'] }],
		['WISE_CSV_EMPTY', {}],
		['WISE_CSV_ENCODING', { cause: 'x' }],
		['WISE_CSV_MIXED_CURRENCIES', { currencies: ['CHF', 'EUR'] }],
		['WISE_CSV_DUPLICATE_ID', { id: 'CARD-9', lines: [4, 9] }],
		['WISE_CSV_RUNNING_BALANCE_BREAK', { line: 6, expected: '10.00', found: '12.00' }],
	])('relevé refusé %s sorti d’un zip : le fichier et son archive nommés', (reason, more) => {
		const out = wiseImportErrorLines(invalid(reason, { archive: 'r.zip', ...more }));
		expect(out).toHaveLength(1);
		expect(out[0]).toContain('« chf.csv » (archive « r.zip »)');
	});

	it('relevé refusé WISE_CSV_LINES : une phrase par ligne, cause traduite', () => {
		const out = wiseImportErrorLines(
			invalid('WISE_CSV_LINES', {
				archive: 'r.zip',
				lines: [
					{ line: 3, field: 'Date', code: 'DATE', value: '2020-13-01' },
					{ line: 5, field: null, code: 'INCONNU', value: null },
				],
			}),
		);
		expect(out[0]).toBe('Lignes en erreur dans le fichier « chf.csv » (archive « r.zip »), rien n’a été importé :');
		expect(out[1]).toBe('Ligne 3, Date : date illisible ou hors format (JJ-MM-AAAA ou AAAA-MM-JJ attendu)');
		expect(out[2]).toBe('Ligne 5, — : INCONNU');
	});

	// Revue P1, M1 — le refus de doublon nomme le relevé déjà importé (le serveur
	// rend `existingFilename`) ; sans nom, le message du serveur.
	it('relevé déjà importé : le nom sous lequel il l’a été', () => {
		expect(
			wiseImportErrorLines(
				refus('BANK_IMPORT_DUPLICATE_FILE', { existingImportId: 12, existingFilename: 'statement_1_CHF.csv' }),
			),
		).toEqual(['Un relevé de cet envoi a déjà été importé, sous le nom « statement_1_CHF.csv » : rien n’a été importé.']);
		expect(wiseImportErrorLines(refus('BANK_IMPORT_DUPLICATE_FILE', { existingImportId: 12 }, 'déjà importé'))).toEqual([
			'déjà importé',
		]);
	});

	it('raison inconnue, zip refusé, autre code, erreur non API : le message du serveur', () => {
		expect(wiseImportErrorLines(invalid('WISE_CSV_NOUVEAU'))).toEqual(['message du serveur']);
		expect(wiseImportErrorLines(refus('WISE_ZIP_INVALID', { filename: 'x.zip', reason: 'ENCRYPTED' }, 'zip chiffré'))).toEqual([
			'zip chiffré',
		]);
		expect(wiseImportErrorLines(refus('PROVIDER_CURRENCY_ACCOUNT_MISSING', { currencies: ['USD'] }, 'USD sans compte'))).toEqual([
			'USD sans compte',
		]);
		expect(wiseImportErrorLines(new Error('réseau'))).toHaveLength(1);
	});
});

describe('libellés du lecteur, de la continuité et du compte rendu (Story 39-11d)', () => {
	it.each([
		['FIELD_COUNT', 'nombre de cellules'],
		['DATE_TIME', 'horodatage'],
		['AMOUNT', 'montant illisible'],
		['RATE', 'cours de change'],
		['CURRENCY', 'devise illisible'],
		['ID', 'identifiant Wise'],
		['DIRECTION', 'CREDIT ou DEBIT'],
		['SIGN_MISMATCH', 'signe du montant'],
		['FEE_SIGN', 'frais au crédit'],
		['OVERFLOW', 'trop grand'],
		['REQUIRED', 'obligatoire vide'],
	])('%s', (code, attendu) => {
		expect(wiseLineErrorLabel({ code })).toContain(attendu);
	});

	it('liste de lignes : bornée, « … » au-delà', () => {
		expect(wiseLinesList([1, 2, 3])).toBe('1, 2, 3');
		expect(wiseLinesList([1, 2, 3], 5)).toBe('1, 2, 3…');
	});

	it('avertissement gap : les deux soldes et la devise ; code inconnu tel quel', () => {
		expect(wiseWarningLabel({ code: 'gap', expected: '5.00', found: '7.00', previousImportId: 1 }, 'EUR')).toContain(
			'solde d’ouverture 7.00 EUR, au lieu de 5.00 EUR',
		);
		expect(wiseWarningLabel({ code: 'autre', expected: '0', found: '0', previousImportId: 1 }, 'EUR')).toBe('autre');
	});

	it('ligne du compte rendu d’un fichier', () => {
		expect(
			wiseFileReportLine({
				filename: 'f.csv',
				archive: null,
				currency: 'USD',
				bankAccountId: 1,
				importId: 2,
				rows: 3,
				inserted: 2,
				duplicates: 1,
				openingBalance: '0.00',
				closingBalance: '9.50',
				warnings: [],
			}),
		).toBe('USD : 2 transaction(s) insérée(s), 1 doublon(s) ; solde de 0.00 à 9.50 USD');
	});
});

// ---------------------------------------------------------------------------
// Story 39-11d2 — la revue
// ---------------------------------------------------------------------------

const t = (more: Record<string, unknown> = {}) => ({
	id: 7,
	bankAccountId: 1,
	date: '2020-02-05',
	amount: '-100.00',
	currency: 'EUR',
	providerType: 'CONVERSION',
	name: null,
	transactionId: 'CONVERSION_ORDER-1',
	counterpartyEmail: null,
	...more,
});
const comptes = [
	{ id: 60, number: '6947', name: 'Frais Wise' },
	{ id: 40, number: '4000', name: 'Achats' },
] as never;
const regles = new Map([[2, 'Boutique']]);

describe('wise-labels — genre et « ce qui sera fait » (Story 39-11d2)', () => {
	it('frais, rattachement, règle : le compte nommé, le numéro d’écriture, la règle', () => {
		const fee = { type: 'fee', bankTransactionId: 7, feeAccountId: 60, transaction: t() } as const;
		expect(wiseProposalKindLabel(fee)).toBe('Frais');
		expect(wiseProposalActionLabel(fee, comptes)).toBe('Écriture au compte de frais 6947 — Frais Wise');
		const link: WiseProposal = {
			type: 'link_entry',
			bankTransactionId: 7,
			journalEntryId: 90,
			transaction: t(),
			entry: { journalEntryId: 90, entryNumber: 12, entryDate: '2020-01-10', description: 'x', ids: [] },
		};
		expect(wiseProposalKindLabel(link)).toBe('Rattachement');
		expect(wiseProposalActionLabel(link, comptes)).toBe('Rattachée à l’écriture n° 12 déjà passée, sans nouvelle écriture');
		const rule = { type: 'rule', bankTransactionId: 7, ruleId: 2, counterpartyAccountId: 40, transaction: t() } as const;
		expect(wiseProposalKindLabel(rule)).toBe('Règle');
		expect(wiseProposalActionLabel(rule, comptes, regles)).toBe('Règle Boutique → 4000 — Achats');
		expect(wiseProposalActionLabel({ ...rule, ruleId: 9, counterpartyAccountId: 99 }, comptes, regles)).toBe('Règle #9 → #99');
	});

	it('conversion : les montants absolus au cours de Wise ; une écriture avec un côté en CHF, sinon deux', () => {
		const deux = {
			type: 'conversion',
			debitTransactionId: 7,
			creditTransactionId: 8,
			transaction: t(),
			otherTransaction: t({ id: 8, amount: '109.00', currency: 'USD' }),
		} as const;
		expect(wiseProposalKindLabel(deux)).toBe('Conversion');
		expect(wiseProposalActionLabel(deux, comptes)).toBe(
			'Au cours de Wise : 100.00 EUR → 109.00 USD, deux écritures par le compte de conversion',
		);
		const une = { ...deux, transaction: t({ amount: '-500.00', currency: 'CHF' }), otherTransaction: t({ id: 8, amount: '462.30' }) };
		expect(wiseProposalActionLabel(une, comptes)).toBe('Au cours de Wise : 500.00 CHF → 462.30 EUR, une écriture');
	});

	it('paiement par carte : une écriture au montant du marchand, la règle et son compte', () => {
		const g: WiseProposal = {
			type: 'card_group',
			bankTransactionIds: [7, 8],
			ruleId: 2,
			counterpartyAccountId: 40,
			lines: [],
			merchantAmount: '97.95',
			transaction: t(),
		};
		expect(wiseProposalKindLabel(g)).toBe('Paiement par carte');
		expect(wiseProposalActionLabel(g, comptes, regles)).toBe(
			'Une écriture : 97.95 CHF au compte 4000 — Achats (règle Boutique), chaque solde crédité de sa part, la différence de change sur son compte',
		);
	});

	it('« frais » après une sous-ligne FEE-, rien sinon', () => {
		expect(wiseLineLabel(t({ transactionId: 'FEE-CARD-1' }))).toBe('frais');
		expect(wiseLineLabel(t({ transactionId: 'CARD-1' }))).toBeNull();
		expect(wiseLineLabel(t({ transactionId: null }))).toBeNull();
	});
});

describe('wise-labels — une phrase par motif et par écart', () => {
	it('chaque motif et chaque écart de la liste a sa phrase ; un code inconnu reste brut, jamais vide', () => {
		for (const code of WISE_TO_REVIEW_CODES) {
			const label = wiseToReviewLabel({ code, transaction: t(), details: {} });
			expect(label, code).not.toBe(code);
			expect(label.length, code).toBeGreaterThan(20);
		}
		for (const code of WISE_GAP_CODES) expect(wiseGapLabel({ code, details: {} }), code).not.toBe(code);
		expect(wiseToReviewLabel({ code: 'inconnu' })).toBe('inconnu');
		expect(wiseGapLabel({ code: 'inconnu' })).toBe('inconnu');
	});

	it('les phrases tirent leurs valeurs des détails', () => {
		expect(
			wiseToReviewLabel({
				code: 'ambiguous_link',
				entries: [
					{ journalEntryId: 1, entryNumber: 14, entryDate: '2020-01-01', description: '', ids: [] },
					{ journalEntryId: 2, entryNumber: 15, entryDate: '2020-01-02', description: '', ids: [] },
				],
			}),
		).toContain('(n° 14, 15) : rattachez la ligne à la bonne');
		expect(wiseToReviewLabel({ code: 'card_group_no_rule', details: { merchantAmount: '97.95' } })).toBe(
			'Paiement par carte de 97.95 CHF : aucune règle ne lui donne un compte en CHF ; créez-en une.',
		);
		expect(wiseToReviewLabel({ code: 'card_group_unsupported', details: { reason: 'exchange_missing' } })).toBe(
			'Les lignes de ce paiement par carte n’ont pas la forme attendue (une ligne en devise ne dit pas sa conversion vers le CHF) : traitez chaque ligne à la main.',
		);
		expect(
			wiseToReviewLabel({ code: 'card_group_statements_missing', details: { missingCurrencies: ['CHF', 'EUR'] } }),
		).toBe(
			'Une ligne de ce paiement peut manquer : le relevé du solde CHF, EUR ne couvre pas cette date ; importez-le, ou traitez les lignes à la main.',
		);
		expect(wiseToReviewLabel({ code: 'no_rule' })).toContain('filtre de type CARD, TRANSFER');
		expect(wiseToReviewLabel({ code: 'stripe_payout_unmatched' })).toContain('importez le rapport Stripe');
		expect(wiseToReviewLabel({ code: 'fee_account_missing' })).toContain('compte de frais');
		expect(wiseGapLabel({ code: 'balance_gap', details: { currency: 'CHF', expected: '0.00', found: '1000.00', date: '2020-05-01' } })).toBe(
			'Le relevé ne commence pas où finissait le précédent : solde d’ouverture 1000.00 CHF, au lieu de 0.00 CHF à la clôture du relevé précédent. Une période manque peut-être : importez-la.',
		);
	});

	it('chaque raison d’un paiement de forme inattendue a sa phrase ; une raison inconnue reste brute', () => {
		for (const r of WISE_CARD_GROUP_UNSUPPORTED_REASONS) expect(wiseCardGroupReasonLabel(r), r).not.toBe(r);
		expect(wiseCardGroupReasonLabel('autre')).toBe('autre');
	});

	it('les écritures d’une part déjà rapprochée : des identifiants, pour ce seul motif', () => {
		expect(wisePartlyReconciledEntryIds({ code: 'card_group_partly_reconciled', details: { journalEntryIds: [5, 'x', 9] } })).toEqual([5, 9]);
		expect(wisePartlyReconciledEntryIds({ code: 'no_rule', details: { journalEntryIds: [5] } })).toEqual([]);
		expect(wiseToReviewLabel({ code: 'card_group_partly_reconciled', details: { journalEntryIds: [5] } })).not.toContain('n°');
	});
});

describe('wise-labels — refus de l’acceptation', () => {
	it('un groupe périmé par son blocage dit la phrase de son motif', () => {
		expect(
			wiseFailedLabel('PROVIDER_PROPOSAL_STALE', { reason: 'card_group_statements_missing', missingCurrencies: ['CHF'] }),
		).toContain('le relevé du solde CHF ne couvre pas cette date');
		expect(wiseFailedLabel('PROVIDER_PROPOSAL_STALE', { reason: 'card_group_partly_reconciled', journalEntryIds: [3] })).toContain(
			'Une part de ce paiement est déjà rapprochée',
		);
	});

	it('toute autre raison : « Rechargez la revue » ; les autres refus par le libellé commun', () => {
		for (const reason of WISE_STALE_RELOAD_REASONS) {
			expect(wiseFailedLabel('PROVIDER_PROPOSAL_STALE', { reason }), reason).toContain('Rechargez la revue');
		}
		expect(wiseFailedLabel('RATE_UNAVAILABLE', { currency: 'USD', month: '2020-03-01' })).toContain(
			'Aucun cours mensuel USD retenu pour 03.2020',
		);
		expect(wiseFailedLabel('COUNTERPARTY_CURRENCY_MISMATCH', { accountCurrency: 'EUR', transactionCurrency: 'CHF' })).toContain(
			'compte tenu en EUR',
		);
	});
});

/**
 * Les listes fermées de la revue Wise, ÉCRITES EN DUR (jamais lues depuis le module : un test qui
 * itérerait sur la table de production serait vert par construction), comparées à la source Rust :
 * les constantes `GAP_*` et `REVIEW_*` de `wise.rs` et de la revue commune que Wise sert, les
 * raisons de `judge_card_group`, les littéraux `stale(…, "<raison>")` hors tests.
 */
describe('wise-labels — les listes contre la source Rust', () => {
	const RECO = '../crates/kesh-reconciliation/src';
	const constantes = (fichier: string, prefixe: string) => {
		const source = readFileSync(`${RECO}/${fichier}`, 'utf-8');
		const re = new RegExp(`pub const ${prefixe}_[A-Z_]+: &str = "([a-z_]+)";`, 'g');
		return [...source.matchAll(re)].map((m) => m[1]);
	};
	// Codes de la revue commune qu'aucun compte Wise ne rend : ni vente ni encaissement (`no_ids`,
	// aucun genre `SaleReceipt`), ni retrait, et une conversion sans sa paire va aux écarts sous le
	// code propre de Wise (`SPEC.conversion_unpaired`).
	const HORS_WISE = [
		'receipt_without_sale',
		'sale_without_receipt',
		'amount_or_currency_mismatch',
		'ambiguous_sale',
		'conversion_unpaired',
		'withdrawal_foreign_currency',
	];
	const ATTENDUS_ECARTS = ['balance_gap', 'conversion_half_missing', 'fee_without_origin'];
	const ATTENDUS_MOTIFS = [
		'already_posted',
		'ambiguous_link',
		'card_group_no_rule',
		'card_group_partly_reconciled',
		'card_group_statements_missing',
		'card_group_unsupported',
		'fee_account_missing',
		'no_rule',
		'stripe_payout_unmatched',
	];

	it('les écarts', () => {
		const source = [...constantes('wise.rs', 'GAP'), ...constantes('provider_review.rs', 'GAP')]
			.filter((c) => !HORS_WISE.includes(c))
			.sort();
		expect(source).toEqual(ATTENDUS_ECARTS);
		expect([...WISE_GAP_CODES].sort()).toEqual(ATTENDUS_ECARTS);
	});

	it('les motifs « à examiner »', () => {
		const source = [...constantes('wise.rs', 'REVIEW'), ...constantes('provider_review.rs', 'REVIEW')]
			.filter((c) => !HORS_WISE.includes(c))
			.sort();
		expect(source).toEqual(ATTENDUS_MOTIFS);
		expect([...WISE_TO_REVIEW_CODES].sort()).toEqual(ATTENDUS_MOTIFS);
	});

	it('les raisons d’un paiement par carte de forme inattendue', () => {
		const source = readFileSync(`${RECO}/wise.rs`, 'utf-8');
		const corps = source.slice(source.indexOf('pub fn judge_card_group('), source.indexOf('fn order_lines('));
		const raisons = [...new Set([...corps.matchAll(/Unsupported\("([a-z_]+)"\)/g)].map((m) => m[1]))].sort();
		const attendues = [
			'card_line_missing',
			'exchange_amount_missing',
			'exchange_currency_mismatch',
			'exchange_missing',
			'merchant_not_chf',
			'merchant_scale',
			'not_a_debit',
		];
		expect(raisons).toEqual(attendues);
		expect([...WISE_CARD_GROUP_UNSUPPORTED_REASONS].sort()).toEqual(attendues);
	});

	it('les raisons de PROVIDER_PROPOSAL_STALE : les littéraux de stale, hors tests, et l’exclusion écrite', () => {
		const horsTests = (chemin: string) => {
			const s = readFileSync(chemin, 'utf-8');
			const i = s.indexOf('#[cfg(test)]');
			return i < 0 ? s : s.slice(0, i);
		};
		const sources =
			horsTests('../crates/kesh-api/src/routes/provider_accept.rs') + horsTests('../crates/kesh-api/src/routes/wise.rs');
		// Ancré sur un appel `stale(<identifiant>, "<raison>")` : la définition
		// `pub fn stale(…, reason: &str)` n'a pas de littéral et n'est pas comptée.
		const litteraux = [...new Set([...sources.matchAll(/\bstale\([a-z_]+, "([a-z_]+)"\)/g)].map((m) => m[1]))].sort();
		const attendues = [
			'card_group_changed',
			'conversion_pair_changed',
			'entry_changed',
			'entry_linked',
			'kind_changed',
			'provider_settings_changed',
			'rule_no_longer_matches',
			'transaction_not_found',
			'transaction_not_pending',
			'withdrawal_foreign_currency',
		];
		expect(litteraux).toEqual(attendues);
		expect([...WISE_STALE_RELOAD_REASONS, ...WISE_STALE_EXCLUDED_REASONS].sort()).toEqual(attendues);
	});
});
