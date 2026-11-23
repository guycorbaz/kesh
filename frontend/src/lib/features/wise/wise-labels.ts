/**
 * Story 39-11d (AC-2) — libellés traduits de l'écran Wise : refus de l'import
 * (une phrase par cause, le fichier nommé), avertissement de continuité d'un
 * relevé, compte rendu d'un fichier. Story 39-11d2 (AC-2 à AC-4) — et la
 * revue : genre et « ce qui sera fait » de chaque proposition, une phrase par
 * motif « à examiner » et par écart (listes fermées, comparées à la source
 * Rust par le test), refus de l'acceptation.
 *
 * ⚠️ **Les clés sont écrites en toutes lettres** (jamais construites par
 * gabarit) : `i18n-keys.test.ts` les voit dès qu'elles manquent d'un catalogue.
 *
 * Les refus de l'archive (`WISE_ZIP_INVALID`, sept raisons) et ceux qui ne
 * sont pas propres à Wise (`PROVIDER_*`, `BANK_ACCOUNT_CLOSED`…) se disent par
 * le **message du serveur**, déjà traduit : il nomme l'archive et dit quoi
 * faire (amendement de la 39-11e). Seul le refus de doublon
 * (`BANK_IMPORT_DUPLICATE_FILE`), dont le message ne nomme rien, a sa phrase :
 * elle dit le nom sous lequel le relevé a déjà été importé (revue P1, M1).
 */
import { i18nMsg } from '$lib/shared/utils/i18n.svelte';
import { isApiError } from '$lib/shared/utils/api-error';
import { errorMessageOf } from '$lib/shared/utils/error-message';
import type { AccountResponse } from '$lib/features/accounts/accounts.types';
import {
	accountNumberName,
	providerFailedLabel,
	providerToReviewLabel,
} from '$lib/features/provider/provider-labels';
import type {
	ProviderFinding,
	ProviderLineError,
	ProviderReviewTransaction,
} from '$lib/features/provider/provider-review.types';
import type { WiseFileReport, WiseFileWarning, WiseProposal } from './wise.types';

/** Nombre de numéros de ligne cités au plus par une phrase de refus (le serveur en rend 100). */
export const WISE_LINES_SHOWN = 10;

/** Libellé d'une erreur de ligne du lecteur Wise (39-11a, `WiseLineErrorCode`) ; code brut si inconnu. */
export function wiseLineErrorLabel(l: Pick<ProviderLineError, 'code'>): string {
	switch (l.code) {
		case 'FIELD_COUNT':
			return i18nMsg('wise-csv-line-field-count', 'nombre de cellules différent de l’en-tête');
		case 'DATE':
			return i18nMsg('wise-csv-line-date', 'date illisible ou hors format (JJ-MM-AAAA ou AAAA-MM-JJ attendu)');
		case 'DATE_TIME':
			return i18nMsg('wise-csv-line-date-time', 'horodatage illisible');
		case 'AMOUNT':
			return i18nMsg('wise-csv-line-amount', 'montant illisible ou vide (point décimal attendu)');
		case 'RATE':
			return i18nMsg('wise-csv-line-rate', 'cours de change illisible');
		case 'CURRENCY':
			return i18nMsg('wise-csv-line-currency', 'devise illisible (trois lettres attendues)');
		case 'ID':
			return i18nMsg('wise-csv-line-id', 'identifiant Wise vide ou illisible');
		case 'DIRECTION':
			return i18nMsg('wise-csv-line-direction', 'sens autre que CREDIT ou DEBIT');
		case 'SIGN_MISMATCH':
			return i18nMsg('wise-csv-line-sign-mismatch', 'le signe du montant ne suit pas le sens (crédit ou débit)');
		case 'FEE_SIGN':
			return i18nMsg('wise-csv-line-fee-sign', 'une ligne de frais au crédit');
		case 'OVERFLOW':
			return i18nMsg('wise-csv-line-overflow', 'montant trop grand');
		case 'REQUIRED':
			return i18nMsg('wise-csv-line-required', 'cellule obligatoire vide');
		default:
			return l.code;
	}
}

type Details = Record<string, unknown> | undefined;

function stringsOf(value: unknown): string[] {
	return Array.isArray(value) ? value.filter((v): v is string => typeof v === 'string') : [];
}

function numbersOf(value: unknown): number[] {
	return Array.isArray(value) ? value.filter((v): v is number => typeof v === 'number') : [];
}

function lineErrorsOf(details: Details): ProviderLineError[] {
	const lines = details?.lines;
	return Array.isArray(lines) ? (lines as ProviderLineError[]).filter((l) => typeof l === 'object' && l !== null) : [];
}

/** Les numéros de ligne cités, bornés à {@link WISE_LINES_SHOWN}, suivis de « … » au-delà. */
export function wiseLinesList(lines: readonly number[], total = lines.length): string {
	const shown = lines.slice(0, WISE_LINES_SHOWN).join(', ');
	return total > Math.min(lines.length, WISE_LINES_SHOWN) ? `${shown}…` : shown;
}

/** Le nom du fichier refusé, avec l'archive d'où il vient s'il sort d'un zip. */
function fileOf(details: Details): string {
	const filename = typeof details?.filename === 'string' ? details.filename : '';
	const archive = typeof details?.archive === 'string' ? details.archive : null;
	return archive
		? i18nMsg('wise-import-error-file-in-archive', '« { $filename } » (archive « { $archive } »)', {
				filename,
				archive,
			})
		: i18nMsg('wise-import-error-file', '« { $filename } »', { filename });
}

/**
 * Les phrases d'un refus de l'import Wise. Un relevé refusé par le lecteur
 * (`WISE_CSV_INVALID`) se dit par sa cause, le fichier nommé ; un relevé exporté
 * sans les frais séparés (`WISE_CSV_FEES_NOT_SEPARATED`), par la consigne de
 * l'exporter de nouveau, option cochée ; deux fichiers d'une même devise, par
 * leurs noms ; un relevé déjà importé, par le nom sous lequel il l'a été
 * (`details.existingFilename` — le serveur ne garde pas le zip d'un import
 * passé, il ne peut donc pas le nommer). Tout autre refus : le message du
 * serveur.
 */
export function wiseImportErrorLines(err: unknown): string[] {
	if (!isApiError(err)) return [errorMessageOf(err)];
	const details = err.details as Details;
	switch (err.code) {
		case 'WISE_CSV_FEES_NOT_SEPARATED': {
			const lines = numbersOf(details?.lines);
			const count = typeof details?.count === 'number' ? details.count : lines.length;
			const out = [
				i18nMsg(
					'wise-import-error-fees-not-separated',
					'Le relevé { $file } a été exporté sans les frais séparés : ses montants comprennent les frais de Wise, que Kesh ne peut alors ni passer en charge ni retrouver. Rien n’a été importé.',
					{ file: fileOf(details) },
				),
				i18nMsg(
					'wise-import-error-fees-not-separated-how',
					'Exportez-le de nouveau depuis Wise en cochant « Afficher les transactions dont les frais sont indiqués séparément », puis importez le nouveau fichier.',
				),
			];
			if (lines.length > 0) {
				out.push(
					i18nMsg('wise-import-error-fees-not-separated-lines', '{ $count } opération(s) à frais sans leur ligne de frais (lignes { $lines }).', {
						count: String(count),
						lines: wiseLinesList(lines, count),
					}),
				);
			}
			return out;
		}
		case 'WISE_DUPLICATE_CURRENCY_FILE': {
			const currency = typeof details?.currency === 'string' ? details.currency : '';
			return [
				i18nMsg(
					'wise-import-error-duplicate-currency-file',
					'Deux fichiers portent le relevé en { $currency } ({ $filenames }) : un seul fichier par devise. Rien n’a été importé.',
					{ currency, filenames: stringsOf(details?.filenames).join(', ') },
				),
			];
		}
		case 'WISE_CSV_INVALID':
			return wiseCsvInvalidLines(err, details);
		case 'BANK_IMPORT_DUPLICATE_FILE': {
			const filename = typeof details?.existingFilename === 'string' ? details.existingFilename : '';
			if (filename === '') return [errorMessageOf(err)];
			return [
				i18nMsg(
					'wise-import-error-duplicate-file',
					'Un relevé de cet envoi a déjà été importé, sous le nom « { $filename } » : rien n’a été importé.',
					{ filename },
				),
			];
		}
		default:
			return [errorMessageOf(err)];
	}
}

/** Les phrases d'un relevé refusé par le lecteur (39-11a), selon `details.reason`. */
function wiseCsvInvalidLines(err: unknown, details: Details): string[] {
	const file = fileOf(details);
	const reason = typeof details?.reason === 'string' ? details.reason : '';
	switch (reason) {
		case 'WISE_CSV_MISSING_COLUMNS':
			return [
				i18nMsg(
					'wise-import-error-missing-columns',
					'Le fichier { $file } n’est pas un relevé Wise : colonnes manquantes ({ $columns }). Exportez de Wise le relevé du solde au format CSV.',
					{ file, columns: stringsOf(details?.columns).join(', ') },
				),
			];
		case 'WISE_CSV_DUPLICATE_COLUMN':
			return [
				i18nMsg('wise-import-error-duplicate-column', 'Le fichier { $file } porte une colonne en double : { $columns }.', {
					file,
					columns: stringsOf(details?.columns).join(', '),
				}),
			];
		case 'WISE_CSV_EMPTY':
			return [i18nMsg('wise-import-error-empty', 'Le fichier { $file } est vide : aucune ligne de données.', { file })];
		case 'WISE_CSV_ENCODING':
			return [i18nMsg('wise-import-error-encoding', 'L’encodage du fichier { $file } n’est pas reconnu.', { file })];
		case 'WISE_CSV_MIXED_CURRENCIES':
			return [
				i18nMsg(
					'wise-import-error-mixed-currencies',
					'Le fichier { $file } porte plusieurs devises ({ $currencies }) : Wise exporte un relevé par solde, un fichier par devise.',
					{ file, currencies: stringsOf(details?.currencies).join(', ') },
				),
			];
		case 'WISE_CSV_DUPLICATE_ID':
			return [
				i18nMsg(
					'wise-import-error-duplicate-id',
					'Le fichier { $file } porte deux fois l’identifiant { $id } dans le même sens (lignes { $lines }).',
					{
						file,
						id: typeof details?.id === 'string' ? details.id : '',
						lines: wiseLinesList(numbersOf(details?.lines)),
					},
				),
			];
		case 'WISE_CSV_RUNNING_BALANCE_BREAK':
			return [
				i18nMsg(
					'wise-import-error-running-balance',
					'Le fichier { $file } ne se suit pas : à la ligne { $line }, le solde courant vaut { $found } au lieu de { $expected }. Exportez de nouveau le relevé complet de la période.',
					{
						file,
						line: String(details?.line ?? ''),
						found: String(details?.found ?? ''),
						expected: String(details?.expected ?? ''),
					},
				),
			];
		case 'WISE_CSV_LINES':
			return [
				i18nMsg('wise-import-error-lines', 'Lignes en erreur dans le fichier { $file }, rien n’a été importé :', { file }),
				...lineErrorsOf(details).map((l) =>
					i18nMsg('wise-import-error-line', 'Ligne { $line }, { $field } : { $cause }', {
						line: String(l.line),
						field: l.field ?? '—',
						cause: wiseLineErrorLabel(l),
					}),
				),
			];
		default:
			return [errorMessageOf(err)];
	}
}

/**
 * L'avertissement de continuité d'un fichier (39-11b, AC-5) : le relevé ne
 * commence pas où finissait le précédent du compte. L'import est passé.
 */
export function wiseWarningLabel(w: WiseFileWarning, currency: string): string {
	if (w.code === 'gap') {
		return i18nMsg(
			'wise-import-warning-gap',
			'Le relevé ne commence pas où finissait le précédent : solde d’ouverture { $found } { $currency }, au lieu de { $expected } { $currency } à la clôture du relevé précédent. Une période manque peut-être : importez-la.',
			{ found: w.found, expected: w.expected, currency },
		);
	}
	return w.code;
}

/** La ligne du compte rendu d'un fichier : devise, lignes insérées, doublons, soldes. */
export function wiseFileReportLine(f: WiseFileReport): string {
	return i18nMsg(
		'wise-import-report-file',
		'{ $currency } : { $inserted } transaction(s) insérée(s), { $duplicates } doublon(s) ; solde de { $opening } à { $closing } { $currency }',
		{
			currency: f.currency,
			inserted: String(f.inserted),
			duplicates: String(f.duplicates),
			opening: f.openingBalance,
			closing: f.closingBalance,
		},
	);
}

// ---------------------------------------------------------------------------
// Revue (Story 39-11d2)
// ---------------------------------------------------------------------------

/**
 * Les écarts de la revue Wise : ceux de Wise (`GAP_*` de
 * `kesh-reconciliation/src/wise.rs`) ; aucun écart de la revue commune n'est
 * servi pour Wise (ni vente, ni encaissement). Exportée pour le test, comparée
 * à la source Rust.
 */
export const WISE_GAP_CODES = ['conversion_half_missing', 'fee_without_origin', 'balance_gap'] as const;

/**
 * Les motifs « à examiner » de la revue Wise : ceux de Wise (`REVIEW_*` de
 * `wise.rs`) et ceux de la revue commune que Wise sert (frais sans compte de
 * frais, aucune règle). Exportée pour le test, comparée à la source Rust.
 */
export const WISE_TO_REVIEW_CODES = [
	'fee_account_missing',
	'no_rule',
	'ambiguous_link',
	'stripe_payout_unmatched',
	'card_group_no_rule',
	'card_group_unsupported',
	'card_group_statements_missing',
	'card_group_partly_reconciled',
] as const;

/** Les raisons d'un paiement par carte qui n'a pas la forme attendue (`judge_card_group`, `wise.rs`). */
export const WISE_CARD_GROUP_UNSUPPORTED_REASONS = [
	'not_a_debit',
	'merchant_scale',
	'exchange_missing',
	'merchant_not_chf',
	'exchange_currency_mismatch',
	'exchange_amount_missing',
	'card_line_missing',
] as const;

/**
 * Les raisons d'un refus `PROVIDER_PROPOSAL_STALE` qui se disent par la phrase
 * commune « Rechargez la revue » (littéraux `stale(…, "<raison>")` du moteur
 * commun et de la route Wise). `withdrawal_foreign_currency` en est exclue par
 * écrit ({@link WISE_STALE_EXCLUDED_REASONS}) : inatteignable depuis Wise, le
 * rejugement refuse avant par `kind_changed`. Les deux raisons du blocage d'un
 * paiement groupé (`card_group_statements_missing`, `card_group_partly_reconciled`)
 * ne sont pas des littéraux de `stale` : elles disent la phrase de leur motif.
 */
export const WISE_STALE_RELOAD_REASONS = [
	'transaction_not_found',
	'provider_settings_changed',
	'transaction_not_pending',
	'kind_changed',
	'entry_changed',
	'entry_linked',
	'rule_no_longer_matches',
	'conversion_pair_changed',
	'card_group_changed',
] as const;

/** Les raisons de `stale` du moteur commun qu'aucune acceptation Wise ne rend (voir ci-dessus). */
export const WISE_STALE_EXCLUDED_REASONS = ['withdrawal_foreign_currency'] as const;

/** Une valeur de `details`, en texte (`''` si absente). */
function detailText(f: ProviderFinding, key: string): string {
	const v = f.details?.[key];
	return v === null || v === undefined ? '' : String(v);
}

/** Un montant signé, sans son signe (`"-100.00"` → `"100.00"`). */
function absAmount(amount: string): string {
	return amount.startsWith('-') ? amount.slice(1) : amount;
}

/** Le genre d'une proposition (colonne « Genre »). */
export function wiseProposalKindLabel(p: WiseProposal): string {
	switch (p.type) {
		case 'fee':
			return i18nMsg('wise-kind-fee', 'Frais');
		case 'conversion':
			return i18nMsg('wise-kind-conversion', 'Conversion');
		case 'link_entry':
			return i18nMsg('wise-kind-link-entry', 'Rattachement');
		case 'rule':
			return i18nMsg('wise-kind-rule', 'Règle');
		case 'card_group':
			return i18nMsg('wise-kind-card-group', 'Paiement par carte');
	}
}

/**
 * Ce que fera une proposition (colonne « Ce qui sera fait ») : `accounts`, les
 * comptes du plan, et `ruleLabels`, les libellés des règles par identifiant,
 * lus au chargement (best-effort : `#<id>` à défaut).
 */
export function wiseProposalActionLabel(
	p: WiseProposal,
	accounts: readonly AccountResponse[],
	ruleLabels: ReadonlyMap<number, string> = new Map(),
): string {
	const rule = (id: number) => ruleLabels.get(id) ?? `#${id}`;
	switch (p.type) {
		case 'fee':
			return i18nMsg('wise-action-fee', 'Écriture au compte de frais { $account }', {
				account: accountNumberName(accounts, p.feeAccountId),
			});
		case 'conversion': {
			const from = `${absAmount(p.transaction.amount)} ${p.transaction.currency}`;
			const other = p.otherTransaction;
			const to = other ? `${absAmount(other.amount)} ${other.currency}` : '';
			return p.transaction.currency === 'CHF' || other?.currency === 'CHF'
				? i18nMsg('wise-action-conversion-one', 'Au cours de Wise : { $from } → { $to }, une écriture', { from, to })
				: i18nMsg(
						'wise-action-conversion-two',
						'Au cours de Wise : { $from } → { $to }, deux écritures par le compte de conversion',
						{ from, to },
					);
		}
		case 'link_entry':
			return i18nMsg(
				'wise-action-link-entry',
				'Rattachée à l’écriture n° { $number } déjà passée, sans nouvelle écriture',
				{ number: String(p.entry?.entryNumber ?? p.journalEntryId) },
			);
		case 'rule':
			return i18nMsg('wise-action-rule', 'Règle { $rule } → { $account }', {
				rule: rule(p.ruleId),
				account: accountNumberName(accounts, p.counterpartyAccountId),
			});
		case 'card_group':
			return i18nMsg(
				'wise-action-card-group',
				'Une écriture : { $amount } CHF au compte { $account } (règle { $rule }), chaque solde crédité de sa part, la différence de change sur son compte',
				{
					amount: p.merchantAmount,
					account: accountNumberName(accounts, p.counterpartyAccountId),
					rule: rule(p.ruleId),
				},
			);
	}
}

/** La raison d'un paiement par carte qui n'a pas la forme attendue, en clair ; brute si inconnue. */
export function wiseCardGroupReasonLabel(reason: string): string {
	switch (reason) {
		case 'not_a_debit':
			return i18nMsg('wise-card-group-reason-not-a-debit', 'une ligne n’est pas au débit');
		case 'merchant_scale':
			return i18nMsg('wise-card-group-reason-merchant-scale', 'un montant en CHF a plus de deux décimales');
		case 'exchange_missing':
			return i18nMsg('wise-card-group-reason-exchange-missing', 'une ligne en devise ne dit pas sa conversion vers le CHF');
		case 'merchant_not_chf':
			return i18nMsg('wise-card-group-reason-merchant-not-chf', 'une conversion ne va pas vers le CHF');
		case 'exchange_currency_mismatch':
			return i18nMsg(
				'wise-card-group-reason-exchange-currency-mismatch',
				'une conversion ne part pas de la devise de son solde',
			);
		case 'exchange_amount_missing':
			return i18nMsg('wise-card-group-reason-exchange-amount-missing', 'une conversion ne dit pas son montant en CHF');
		case 'card_line_missing':
			return i18nMsg('wise-card-group-reason-card-line-missing', 'seuls des frais, sans la ligne du paiement');
		default:
			return reason;
	}
}

/** Les chaînes d'une liste de `details` (devises), sans les autres valeurs. */
function stringList(value: unknown): string[] {
	return Array.isArray(value) ? value.filter((v): v is string => typeof v === 'string') : [];
}

/** La phrase d'un paiement groupé dont une ligne peut manquer : les devises des soldes non couverts. */
function statementsMissingLabel(currencies: readonly string[]): string {
	return i18nMsg(
		'wise-review-card-group-statements-missing',
		'Une ligne de ce paiement peut manquer : le relevé du solde { $currencies } ne couvre pas cette date ; importez-le, ou traitez les lignes à la main.',
		{ currencies: currencies.join(', ') },
	);
}

/** La phrase d'un paiement groupé dont une part est déjà rapprochée. */
function partlyReconciledLabel(): string {
	return i18nMsg(
		'wise-review-card-group-partly-reconciled',
		'Une part de ce paiement est déjà rapprochée : annulez ce rapprochement puis acceptez le paiement entier, ou finissez à la main.',
	);
}

/** Libellé d'un motif « à examiner » de la revue Wise ; code brut si inconnu, jamais une ligne vide. */
export function wiseToReviewLabel(f: ProviderFinding): string {
	switch (f.code) {
		case 'no_rule':
			return i18nMsg(
				'wise-review-no-rule',
				'Aucune règle ne reconnaît cette ligne : créez une règle d’affectation (filtre de type CARD, TRANSFER…), ou passez l’écriture à la main puis rattachez la ligne.',
			);
		case 'ambiguous_link':
			return i18nMsg(
				'wise-review-ambiguous-link',
				'Plusieurs écritures déjà passées peuvent être l’autre côté de ce virement (n° { $numbers }) : rattachez la ligne à la bonne.',
				{ numbers: (f.entries ?? []).map((e) => String(e.entryNumber)).join(', ') },
			);
		case 'stripe_payout_unmatched':
			return i18nMsg(
				'wise-review-stripe-payout-unmatched',
				'Versement Stripe sans l’écriture de son versement : importez le rapport Stripe et acceptez le versement, ou passez l’écriture à la main puis rattachez la ligne ; aucune règle ne s’y applique.',
			);
		case 'card_group_no_rule':
			return i18nMsg(
				'wise-review-card-group-no-rule',
				'Paiement par carte de { $amount } CHF : aucune règle ne lui donne un compte en CHF ; créez-en une.',
				{ amount: detailText(f, 'merchantAmount') },
			);
		case 'card_group_unsupported':
			return i18nMsg(
				'wise-review-card-group-unsupported',
				'Les lignes de ce paiement par carte n’ont pas la forme attendue ({ $reason }) : traitez chaque ligne à la main.',
				{ reason: wiseCardGroupReasonLabel(detailText(f, 'reason')) },
			);
		case 'card_group_statements_missing':
			return statementsMissingLabel(stringList(f.details?.missingCurrencies));
		case 'card_group_partly_reconciled':
			return partlyReconciledLabel();
		default:
			return providerToReviewLabel(f.code) ?? f.code;
	}
}

/** Les écritures d'une part déjà rapprochée (`details.journalEntryIds`, des identifiants), pour leurs liens. */
export function wisePartlyReconciledEntryIds(f: ProviderFinding): number[] {
	if (f.code !== 'card_group_partly_reconciled') return [];
	const ids = f.details?.journalEntryIds;
	return Array.isArray(ids) ? ids.filter((v): v is number => typeof v === 'number') : [];
}

/** Libellé d'un écart de la revue Wise ; code brut si inconnu. */
export function wiseGapLabel(f: ProviderFinding): string {
	switch (f.code) {
		case 'conversion_half_missing':
			return i18nMsg(
				'wise-gap-conversion-half-missing',
				'L’autre moitié de cette conversion manque : importez le relevé de l’autre devise, ou elle est déjà rapprochée.',
			);
		case 'fee_without_origin':
			return i18nMsg(
				'wise-gap-fee-without-origin',
				'Frais sans la ligne dont il vient sur ce solde : importez le relevé complet de la période.',
			);
		case 'balance_gap':
			// La phrase de l'avertissement de l'import, réutilisée (la date est dans sa colonne).
			return wiseWarningLabel(
				{
					code: 'gap',
					expected: detailText(f, 'expected'),
					found: detailText(f, 'found'),
					previousImportId: Number(f.details?.previousImportId ?? 0),
				},
				detailText(f, 'currency'),
			);
		default:
			return f.code;
	}
}

/**
 * Libellé d'un refus de `POST /wise/accept` : un groupe périmé par son blocage
 * (`PROVIDER_PROPOSAL_STALE`, `details.reason` `card_group_statements_missing`
 * ou `card_group_partly_reconciled`) dit la phrase de son motif ; tout autre
 * refus est délégué à `providerFailedLabel` (puis `failedProposalLabel`).
 */
export function wiseFailedLabel(code: string, details?: unknown): string {
	const d = details && typeof details === 'object' ? (details as Record<string, unknown>) : {};
	if (code === 'PROVIDER_PROPOSAL_STALE') {
		if (d.reason === 'card_group_statements_missing') return statementsMissingLabel(stringList(d.missingCurrencies));
		if (d.reason === 'card_group_partly_reconciled') return partlyReconciledLabel();
	}
	return providerFailedLabel(code, details);
}

/** Le texte après une sous-ligne d'un paiement groupé : « frais » pour une ligne `FEE-…`, rien sinon. */
export function wiseLineLabel(t: ProviderReviewTransaction): string | null {
	return t.transactionId?.startsWith('FEE-') ? i18nMsg('wise-line-fee', 'frais') : null;
}
