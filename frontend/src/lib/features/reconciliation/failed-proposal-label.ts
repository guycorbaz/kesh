/**
 * Story 15-5c (AC1, AC2, #492) — libellé traduit d'un refus d'acceptation ou de rejet par lot
 * (`FailedProposal` de `POST /reconciliation/accept-batch` et `/reject`).
 *
 * Les codes restent des **constantes canoniques** côté serveur — cf. la § *Pattern batch —
 * FailedProposal per-proposal* du `CLAUDE.md` ; seul leur affichage est traduit. Patron :
 * `failedItemLabel` (`payment-batches/payment-batch-helpers.ts`).
 *
 * **Les 26 codes** sont ceux que `crates/kesh-api/src/routes/reconciliation.rs` peut poser dans
 * `failed[]` : les 25 littéraux relevés par
 * `grep -ohE 'error_code: "[A-Z_]+"' crates/kesh-api/src/routes/reconciliation.rs | sort -u`,
 * plus `ACCOUNT_NOT_POSTABLE`, posé par `DbError::error_code()` (seule forme non littérale du
 * fichier). Un code apparu depuis retombe sur le repli, qui **montre le code brut** : mieux vaut
 * un code qu'une case vide.
 *
 * ⚠️ **Clés réutilisées** (choix C37, étendu par C-15-5c-1) : quand un message existant convient
 * mot pour mot et sans variable, il est lu tel quel au lieu d'être recopié — `error-*` (espace
 * global) ou `reconciliation-*` (espace de cette fonctionnalité). Les autres codes ont une clé
 * neuve `reconciliation-failed-*`.
 *
 * ⚠️ **Les clés sont écrites en toutes lettres, jamais construites par gabarit** : une clé
 * statique est vue par `i18n-keys.test.ts` dès qu'elle manque d'un catalogue.
 *
 * ⚠️ **Deux codes seulement lisent leur `details`** : `ACCOUNT_NOT_POSTABLE` (AC2), pour
 * nommer les comptes, et `RECONCILIATION_INVOICE_NOT_ELIGIBLE`, pour la seule raison
 * `payment_date_before_invoice_date` (revue de code P1, E1 — choix C-15-5c-3). Cette raison
 * est la seule qu'un utilisateur rencontre sans avoir rien fait de travers : la proposition
 * retient les factures datées de 30 jours avant à 30 jours après la transaction, alors que
 * l'acceptation refuse un paiement antérieur de plus d'un jour à la facture (#548). Les
 * autres raisons gardent le libellé générique.
 *
 * ⚠️ **Plusieurs causes sous un libellé unique — limite assumée (choix C32, C37)** :
 * - `PERIOD_LOCKED` porte `lockedThrough` et `attempted`, que le libellé ne lit pas ;
 * - `RECONCILIATION_INVOICE_NOT_ELIGIBLE` porte six raisons dans `details.reason`
 *   (`invoice_not_validated`, `invoice_already_paid`, `invoice_journal_entry_not_set`,
 *   `payment_date_before_invoice_date`, `payment_date_outside_window`,
 *   `race_during_update`) : une seule a son libellé, les cinq autres partagent
 *   « n'est pas éligible » ;
 * - `VALIDATION_ERROR` porte six raisons sur sept sites (`splits_count_out_of_range`,
 *   `split_description_too_long`, `split_amount_not_positive`, `split_amount_scale_too_high`,
 *   `counterparty_equals_bank_ledger`, `zero_amount_transaction`), toutes rendues
 *   « Erreur de validation » par la clé globale `error-validation`.
 * Le code brut reste en `title` de chaque refus, pour le support.
 */
import { i18nMsg } from '$lib/shared/utils/i18n.svelte';

/**
 * Numéros des comptes non imputables d'un `details` de la forme
 * `{ rejected: [{ accountId, accountNumber }] }` (choix C16), ou `[]` pour toute autre forme.
 *
 * `FailedProposal.details` est typé `unknown` : rien n'est supposé — objet, tableau, chaînes
 * non vides sont vérifiés un à un, et une entrée d'une autre forme est ignorée.
 */
export function rejectedAccountNumbers(details: unknown): string[] {
	if (typeof details !== 'object' || details === null) return [];
	const rejected = (details as { rejected?: unknown }).rejected;
	if (!Array.isArray(rejected)) return [];
	const numbers: string[] = [];
	for (const entry of rejected) {
		if (typeof entry !== 'object' || entry === null) continue;
		const n = (entry as { accountNumber?: unknown }).accountNumber;
		if (typeof n === 'string' && n.trim() !== '') numbers.push(n);
	}
	return numbers;
}

/**
 * Raison d'un `details` de la forme `{ reason: "<raison>" }`, ou `null` pour toute autre forme.
 * Même prudence que `rejectedAccountNumbers` : `details` est typé `unknown`.
 */
export function failureReason(details: unknown): string | null {
	if (typeof details !== 'object' || details === null) return null;
	const reason = (details as { reason?: unknown }).reason;
	return typeof reason === 'string' ? reason : null;
}

/**
 * Libellé traduit d'un `errorCode` de `failed[]`. `details` n'est lu que pour
 * `ACCOUNT_NOT_POSTABLE` et `RECONCILIATION_INVOICE_NOT_ELIGIBLE` ; un code inconnu rend
 * un repli qui cite le code.
 */
export function failedProposalLabel(code: string, details?: unknown): string {
	switch (code) {
		case 'ACCOUNT_NOT_FOUND':
			return i18nMsg(
				'reconciliation-failed-account-not-found',
				'Un compte de contrepartie est introuvable ou archivé.'
			);
		case 'ACCOUNT_NOT_POSTABLE': {
			const numbers = rejectedAccountNumbers(details);
			if (numbers.length > 0) {
				return i18nMsg(
					'reconciliation-failed-account-not-postable',
					'Compte non imputable : { $numbers }. Un compte de regroupement, de résultat ou de clôture ne reçoit pas d’écriture : choisissez un compte imputable.',
					{ numbers: numbers.join(', ') }
				);
			}
			return i18nMsg(
				'reconciliation-failed-account-not-postable-generic',
				'Un compte de contrepartie n’est pas imputable : un compte de regroupement, de résultat ou de clôture ne reçoit pas d’écriture. Choisissez un compte imputable.'
			);
		}
		case 'BANK_ACCOUNT_NOT_CONFIGURED':
			return i18nMsg(
				'reconciliation-failed-bank-account-not-configured',
				'Le compte bancaire n’est lié à aucun compte comptable actif : liez-en un dans Administration → Comptes bancaires.'
			);
		case 'BANK_ACCOUNT_NOT_FOUND':
			return i18nMsg('reconciliation-failed-bank-account-not-found', 'Compte bancaire introuvable.');
		case 'BANK_TRANSACTION_NOT_FOUND':
			return i18nMsg(
				'reconciliation-failed-bank-transaction-not-found',
				'Transaction bancaire introuvable.'
			);
		case 'DATABASE_ERROR':
			return i18nMsg(
				'reconciliation-failed-database-error',
				'Erreur de la base de données : réessayez ; si le problème persiste, contactez le support.'
			);
		case 'FISCAL_YEAR_INVALID':
		case 'RECONCILIATION_FISCAL_YEAR_CLOSED':
			// Les deux codes naissent du même constat — `find_open_covering_date` ne trouve
			// aucun exercice OUVERT couvrant la date —, et le message existant le dit mot pour mot.
			return i18nMsg('error-fiscal-year-invalid', 'Aucun exercice ouvert ne couvre cette date.');
		case 'INTERNAL_ERROR':
			return i18nMsg('error-internal', 'Erreur interne');
		case 'INVOICE_NOT_FOUND':
			return i18nMsg('reconciliation-failed-invoice-not-found', 'Facture introuvable.');
		case 'INVOICE_SALE_ENTRY_MALFORMED':
			return i18nMsg(
				'reconciliation-failed-invoice-sale-entry-malformed',
				'L’écriture de vente de cette facture n’a pas de ligne au compte débiteurs : le règlement ne peut pas être passé.'
			);
		case 'PERIOD_LOCKED':
			return i18nMsg(
				'reconciliation-failed-period-locked',
				'La date de cette transaction tombe dans la période verrouillée : aucune écriture ne peut y être datée (voir le verrou de période).'
			);
		case 'PROJECT_ARCHIVED':
			return i18nMsg('reconciliation-failed-project-archived', 'Le projet analytique est archivé.');
		case 'PROJECT_NOT_FOUND':
			return i18nMsg('reconciliation-failed-project-not-found', 'Projet analytique introuvable.');
		case 'RECONCILIATION_ALREADY_RECONCILED':
			return i18nMsg(
				'reconciliation-errors-already-reconciled',
				'Cette transaction est déjà réconciliée.'
			);
		case 'RECONCILIATION_CURRENCY_MISMATCH':
			return i18nMsg(
				'reconciliation-failed-currency-mismatch',
				'Seules les transactions en CHF peuvent être rapprochées d’une facture ou d’une règle.'
			);
		case 'RECONCILIATION_INVOICE_NOT_ELIGIBLE':
			if (failureReason(details) === 'payment_date_before_invoice_date') {
				return i18nMsg(
					'reconciliation-failed-payment-before-invoice',
					'Le paiement est daté de plus d’un jour avant la facture : il ne peut pas la régler, même si elle a été proposée.'
				);
			}
			return i18nMsg(
				'reconciliation-errors-invoice-not-eligible',
				"Cette facture n'est pas éligible à la réconciliation."
			);
		case 'RECONCILIATION_OVERPAYMENT':
			return i18nMsg(
				'reconciliation-failed-overpayment',
				'Le montant de la transaction dépasse le reste dû de la facture.'
			);
		case 'RECONCILIATION_RULE_MISMATCH':
			return i18nMsg(
				'reconciliation-failed-rule-mismatch',
				'Le compte de contrepartie de la règle a changé : rechargez la page.'
			);
		case 'RECONCILIATION_RULE_NO_LONGER_MATCHES':
			return i18nMsg(
				'reconciliation-failed-rule-no-longer-matches',
				'La règle ne correspond plus à cette transaction : elle a été modifiée entre-temps.'
			);
		case 'RECONCILIATION_RULE_NOT_FOUND':
			return i18nMsg('reconciliation-failed-rule-not-found', 'Règle introuvable ou désactivée.');
		case 'RECONCILIATION_SCORE_TOO_LOW':
			return i18nMsg(
				'reconciliation-failed-score-too-low',
				'La facture ne correspond pas assez à la transaction pour être rapprochée.'
			);
		case 'RECONCILIATION_SPLIT_IMBALANCE':
			return i18nMsg(
				'reconciliation-split-error-imbalance',
				"L'éclatement n'équilibre pas le montant de la transaction."
			);
		case 'RECONCILIATION_TRANSACTION_NOT_PENDING':
			return i18nMsg(
				'reconciliation-failed-transaction-not-pending',
				'Cette transaction n’est plus en attente de rapprochement.'
			);
		case 'ROUNDING_ACCOUNT_NOT_CONFIGURED':
			return i18nMsg(
				'error-rounding-account-not-configured',
				"Ce paiement solde la facture au centime, mais aucun compte de différences d'arrondi utilisable n'est désigné : choisissez-en un dans Paramètres → Facturation."
			);
		case 'VALIDATION_ERROR':
			return i18nMsg('error-validation', 'Erreur de validation');
		default:
			return i18nMsg('reconciliation-failed-unknown', 'Refus non reconnu ({ $code }).', { code });
	}
}
