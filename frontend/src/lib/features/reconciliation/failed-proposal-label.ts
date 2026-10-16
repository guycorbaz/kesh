/**
 * Story 15-5c (AC1, AC2, #492) — libellé traduit d'un refus d'acceptation ou de rejet par lot
 * (`FailedProposal` de `POST /reconciliation/accept-batch` et `/reject`).
 *
 * Les codes restent des **constantes canoniques** côté serveur — cf. la § *Pattern batch —
 * FailedProposal per-proposal* du `CLAUDE.md` ; seul leur affichage est traduit. Patron :
 * `failedItemLabel` (`payment-batches/payment-batch-helpers.ts`).
 *
 * **Les 37 codes** sont ceux que `crates/kesh-api/src/routes/reconciliation.rs` peut poser dans
 * `failed[]` : les 34 littéraux relevés par
 * `grep -ohE 'error_code: "[A-Z_]+"' crates/kesh-api/src/routes/reconciliation.rs | sort -u`,
 * plus les trois codes posés par `DbError::error_code()` (seules formes non littérales du
 * fichier) : `ACCOUNT_NOT_POSTABLE`, depuis la Story 15-6b (#474)
 * `SETTLEMENT_COUNTERPARTY_IS_CLAIM_ACCOUNT`, et depuis la Story 30-1b (#686)
 * `INACTIVE_OR_INVALID_ACCOUNTS` (le règlement d'une facture fournisseur). Un code apparu depuis
 * retombe sur le repli, qui **montre le code brut** : mieux vaut un code qu'une case vide.
 *
 * Story 30-1b (#686) — cinq littéraux neufs, qui portent le compte de 29 à 34 :
 * `RECONCILIATION_TRANSACTION_NOT_DEBIT` (un crédit ou un montant nul proposé contre une facture
 * fournisseur), `SUPPLIER_INVOICE_NOT_FOUND` (même libellé que la facture client),
 * `RECONCILIATION_AMOUNT_MISMATCH` (le règlement fournisseur est du total exact),
 * `RECONCILIATION_SUPPLIER_PAYMENT_HOLD` (une autre facture du même montant dans un lot en
 * cours sur ce compte) et `RECONCILIATION_RULE_MASKED` (une facture défend la règle).
 *
 * ⚠️ **Clés réutilisées** (choix C37, étendu par C-15-5c-1) : quand un message existant convient
 * mot pour mot et sans variable, il est lu tel quel au lieu d'être recopié — `error-*` (espace
 * global) ou `reconciliation-*` (espace de cette fonctionnalité). Les autres codes ont une clé
 * neuve `reconciliation-failed-*`.
 *
 * ⚠️ **Les clés sont écrites en toutes lettres, jamais construites par gabarit** : une clé
 * statique est vue par `i18n-keys.test.ts` dès qu'elle manque d'un catalogue.
 *
 * Le littéral ajouté par la Story 27-4 (#634) (ce qui portait le compte à 27) est
 * `RECONCILIATION_ALREADY_REJECTED` : une
 * transaction rejetée refusée par le pré-vol d'un lot (auparavant, le lot entier était refusé
 * en `400`). Depuis la Story 28-5b (#526), son libellé dit où annuler le rejet : le détail de
 * l'import de la transaction. Depuis la Story 28-6 (#482), le littéral qui porte le compte à
 * 28 est `RECONCILIATION_TRANSACTION_NOT_CREDIT` : un débit ou un montant nul proposé contre une
 * facture (un seul libellé pour ses deux `details.reason`). Depuis la Story 28-4b (#642), le
 * littéral ajouté en dernier, qui porte le compte à 29, est `BANK_ACCOUNT_CLOSED` : les pré-vols
 * R1 et R2 d'un lot dont le compte bancaire est clos (Story 28-4a).
 *
 * ⚠️ **Six codes seulement lisent leur `details`** : `SETTLEMENT_COUNTERPARTY_IS_CLAIM_ACCOUNT`
 * (Story 15-6b), pour son `role` — le compte de banque lié au compte débiteurs et le compte
 * d'arrondi désigné qui l'est n'appellent pas le même remède — et, depuis la Story 30-1b, pour sa
 * `claim` (`payable` : le compte **créanciers** d'une facture fournisseur) ;
 * `ACCOUNT_NOT_POSTABLE` (AC2), pour nommer les comptes ; `LATER_FISCAL_YEAR_CLOSED`
 * (Story 15-12b, #543), pour nommer l'exercice postérieur clos (`details.fiscalYearName`, repli
 * sans nom si le champ manque) ; `RECONCILIATION_RULE_MASKED` (Story 30-1b), pour sa raison
 * (`payment_batch`, `invoice_candidate`, `invoice_slightly_overpaid`) ; et
 * `RECONCILIATION_INVOICE_NOT_ELIGIBLE`, pour deux raisons : `payment_date_before_invoice_date`
 * (revue de code P1, E1 — choix C-15-5c-3), dont le libellé lit `details.tolerance_days`
 * depuis la Story 30-1b (1 jour client, 7 jours fournisseur ; repli 1), et
 * `supplier_invoice_in_payment_batch` (Story 30-1b). Depuis la
 * Story 28-7 (#548), la proposition et l'acceptation partagent la même fenêtre de dates
 * (`InvoiceDateWindow`, `kesh-db`) : une facture datée de plus d'un jour après le paiement
 * n'est plus proposée, et cette raison ne se rencontre plus que si la facture a été redatée
 * entre la proposition et l'acceptation, ou par un appel d'API direct. Les autres raisons
 * gardent le libellé générique. Enfin, depuis la Story 27-4 (#634, choix
 * C-27-4-8), `RECONCILIATION_ALREADY_RECONCILED`, pour la seule raison `race_during_update` : la
 * transaction a changé entre la relecture et l'écriture — rapprochée, rejetée **ou supprimée** —,
 * et « déjà réconciliée » serait faux pour une suppression.
 *
 * ⚠️ **Plusieurs causes sous un libellé unique — limite assumée (choix C32, C37)** :
 * - `PERIOD_LOCKED` porte `lockedThrough` et `attempted`, que le libellé ne lit pas ;
 * - `RECONCILIATION_INVOICE_NOT_ELIGIBLE` porte huit raisons dans `details.reason`
 *   (`invoice_not_validated`, `invoice_already_paid`, `invoice_journal_entry_not_set`,
 *   `payment_date_before_invoice_date`, `payment_date_outside_window`,
 *   `race_during_update`, et depuis la Story 30-1b `supplier_invoice_not_open`,
 *   `supplier_invoice_in_payment_batch`) : deux ont leur libellé, les six autres partagent
 *   « n'est pas éligible » ;
 * - `VALIDATION_ERROR` porte six raisons sur huit sites (`splits_count_out_of_range`,
 *   `split_description_too_long`, `split_amount_not_positive`, `split_amount_scale_too_high`,
 *   `counterparty_equals_bank_ledger` — sur deux sites, la proposition ventilée et, depuis la
 *   Story 15-6d (#524), la proposition par règle —, `zero_amount_transaction`), toutes rendues
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
 * Rôle d'un `details` de la forme `{ role: "<rôle>" }` (Story 15-6b), ou `null` pour toute autre
 * forme. Même prudence que `failureReason`.
 */
export function failureRole(details: unknown): string | null {
	if (typeof details !== 'object' || details === null) return null;
	const role = (details as { role?: unknown }).role;
	return typeof role === 'string' ? role : null;
}

/**
 * Créance d'un `details` de la forme `{ claim: "<receivable|payable>" }` (Story 15-6b, lu
 * depuis la Story 30-1b), ou `null` pour toute autre forme.
 */
export function failureClaim(details: unknown): string | null {
	if (typeof details !== 'object' || details === null) return null;
	const claim = (details as { claim?: unknown }).claim;
	return typeof claim === 'string' ? claim : null;
}

/**
 * La tolérance, en jours, d'un refus `payment_date_before_invoice_date`
 * (`details.tolerance_days`, Story 30-1b : 1 pour une facture client, 7 pour une facture
 * fournisseur) ; repli `1` — la valeur d'avant le champ.
 */
export function toleranceDays(details: unknown): number {
	if (typeof details !== 'object' || details === null) return 1;
	const days = (details as { tolerance_days?: unknown }).tolerance_days;
	return typeof days === 'number' && Number.isInteger(days) && days > 0 ? days : 1;
}

/**
 * Nom de l'exercice d'un `details` de la forme `{ fiscalYearName: "<nom>" }` (refus
 * `LATER_FISCAL_YEAR_CLOSED`, Story 15-12b), ou `null` pour toute autre forme ou un nom vide.
 * Même prudence que `rejectedAccountNumbers` : `details` est typé `unknown`.
 */
export function fiscalYearName(details: unknown): string | null {
	if (typeof details !== 'object' || details === null) return null;
	const name = (details as { fiscalYearName?: unknown }).fiscalYearName;
	return typeof name === 'string' && name.trim() !== '' ? name : null;
}

/**
 * Libellé traduit d'un `errorCode` de `failed[]`. `details` n'est lu que pour
 * `SETTLEMENT_COUNTERPARTY_IS_CLAIM_ACCOUNT`, `ACCOUNT_NOT_POSTABLE`, `LATER_FISCAL_YEAR_CLOSED`,
 * `RECONCILIATION_INVOICE_NOT_ELIGIBLE`, `RECONCILIATION_ALREADY_RECONCILED` et
 * `RECONCILIATION_RULE_MASKED` ; un code inconnu rend un repli qui cite le code.
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
		// Story 28-4b (#642, AC13) — compte clos (pré-vols R1, R2 de la 28-4a) ; atteignable
		// par une course seulement (compte clos dans un autre onglet). Texte de C-28-4-27.
		case 'BANK_ACCOUNT_CLOSED':
			return i18nMsg(
				'reconciliation-failed-bank-account-closed',
				'Le compte bancaire est clos : il est en lecture seule. Pour y enregistrer une opération, rouvrez d’abord le compte (Administration → Comptes bancaires).'
			);
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
				'Transaction introuvable : elle a été supprimée avec son import, ou n’appartient pas à ce compte bancaire.'
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
		// Story 30-1b (#686, P3 R2) — un compte de l'écriture de règlement désactivé (le compte
		// créanciers de l'achat, ou le compte du grand livre de la banque) : la clé globale.
		case 'INACTIVE_OR_INVALID_ACCOUNTS':
			return i18nMsg('error-inactive-accounts', 'Un ou plusieurs comptes sont archivés ou invalides.');
		case 'INVOICE_NOT_FOUND':
		// Story 30-1b — la facture fournisseur absente ou d'une autre société : même libellé que
		// la facture client (P2 R-L16).
		case 'SUPPLIER_INVOICE_NOT_FOUND':
			return i18nMsg('reconciliation-failed-invoice-not-found', 'Facture introuvable.');
		case 'INVOICE_SALE_ENTRY_MALFORMED':
			return i18nMsg(
				'reconciliation-failed-invoice-sale-entry-malformed',
				'L’écriture de vente de cette facture n’a pas de ligne au compte débiteurs : le règlement ne peut pas être passé.'
			);
		case 'LATER_FISCAL_YEAR_CLOSED': {
			// Story 15-12b (#543) — l'exercice de la transaction est suivi d'un exercice clos
			// (état hérité) : le bilan de celui-ci reprendrait l'écriture. La clé globale
			// `error-later-fiscal-year-closed` porte une phrase entière, trop longue pour une
			// ligne d'échec (choix C100).
			const name = fiscalYearName(details);
			if (name !== null) {
				return i18nMsg(
					'reconciliation-failed-later-fiscal-year-closed',
					'Exercice postérieur « { $name } » clôturé : aucune écriture ne peut être datée avant lui.',
					{ name }
				);
			}
			return i18nMsg(
				'reconciliation-failed-later-fiscal-year-closed-generic',
				'Un exercice postérieur est clôturé : aucune écriture ne peut être datée avant lui.'
			);
		}
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
			// Story 27-4 (C-27-4-8) — `race_during_update` : changée pendant le traitement, peut-être
			// supprimée ; « déjà réconciliée » serait faux.
			if (failureReason(details) === 'race_during_update') {
				return i18nMsg(
					'reconciliation-failed-changed-during-batch',
					'La transaction a été modifiée pendant le traitement — rapprochée, rejetée ou supprimée par une autre opération.'
				);
			}
			return i18nMsg(
				'reconciliation-errors-already-reconciled',
				'Cette transaction est déjà réconciliée.'
			);
		case 'RECONCILIATION_ALREADY_REJECTED':
			return i18nMsg(
				'reconciliation-failed-already-rejected',
				'Cette transaction a déjà été rejetée : elle n’est plus proposée. Son rejet s’annule depuis le détail de son import (Importer des relevés).'
			);
		case 'RECONCILIATION_CURRENCY_MISMATCH':
			return i18nMsg(
				'reconciliation-failed-currency-mismatch',
				'Seules les transactions en CHF peuvent être rapprochées d’une facture ou d’une règle.'
			);
		case 'RECONCILIATION_INVOICE_NOT_ELIGIBLE': {
			const reason = failureReason(details);
			if (reason === 'payment_date_before_invoice_date') {
				// Story 30-1b — la tolérance est lue sur `details.tolerance_days` (1 jour pour une
				// facture client, 7 pour une facture fournisseur) ; repli 1. Deux clés et non un
				// sélecteur Fluent : le serveur pré-résout les messages sans argument, un
				// `{ $days -> … }` n'atteindrait pas l'écran. Un jour garde le libellé d'avant
				// (« plus d'un jour »), cité au manuel (revue de code P1, A5/B2).
				const days = toleranceDays(details);
				if (days === 1) {
					return i18nMsg(
						'reconciliation-failed-payment-before-invoice',
						'Le paiement est daté de plus d’un jour avant la facture : il ne peut pas la régler.'
					);
				}
				return i18nMsg(
					'reconciliation-failed-payment-before-invoice-days',
					'Le paiement est daté de plus de { $days } jours avant la facture : il ne peut pas la régler.',
					{ days }
				);
			}
			// Story 30-1b (#686, AC-b6) — la facture fournisseur est dans un lot de paiement en
			// cours. Le remède : annuler le lot, ou affecter à la main — jamais « confirmer ».
			if (reason === 'supplier_invoice_in_payment_batch') {
				return i18nMsg(
					'reconciliation-failed-supplier-invoice-in-batch',
					'Cette facture fournisseur figure dans un lot de paiement en cours : annulez le lot si cette transaction en est le paiement, sinon affectez la transaction à la main.'
				);
			}
			return i18nMsg(
				'reconciliation-errors-invoice-not-eligible',
				"Cette facture n'est pas éligible à la réconciliation."
			);
		}
		// Story 30-1b (#686, AC-b6) — le règlement fournisseur est binaire : le débit doit
		// égaler le TTC de la facture.
		case 'RECONCILIATION_AMOUNT_MISMATCH':
			return i18nMsg(
				'reconciliation-failed-amount-mismatch',
				'Le montant de la transaction diffère du total de la facture fournisseur : un paiement fournisseur règle le total exact.'
			);
		// Story 30-1b (#686, AC-b6 (d)) — une AUTRE facture du même montant est dans un lot en
		// cours sur ce compte : ce débit est peut-être le paiement de ce lot.
		case 'RECONCILIATION_SUPPLIER_PAYMENT_HOLD':
			return i18nMsg(
				'reconciliation-failed-supplier-payment-hold',
				'Une autre facture fournisseur de ce montant figure dans un lot de paiement en cours sur ce compte : cette transaction en est peut-être le paiement. Annulez le lot si c’est le cas, sinon affectez la transaction à la main.'
			);
		// Story 30-1b (#686, AC-b17) — une facture défend la règle : l'accepter compterait la
		// charge (ou le produit) deux fois. La raison dit laquelle, et le geste juste : traiter la
		// facture — l'accepter ici (débit), rattacher la transaction à son règlement (crédit).
		// « Affecter manuellement » n'est conseillé que si la transaction ne règle aucune facture
		// saisie : sinon il compterait la dépense deux fois (encadré L1 du manuel ; revue de
		// code P1 de la 30-1b, INFO de la lentille A).
		case 'RECONCILIATION_RULE_MASKED': {
			const reason = failureReason(details);
			if (reason === 'payment_batch') {
				return i18nMsg(
					'reconciliation-failed-rule-masked-payment-batch',
					'La règle n’est pas appliquée : une facture fournisseur de ce montant figure dans un lot de paiement en cours sur ce compte. Si cette transaction en est le paiement, annulez le lot, puis acceptez ici la facture proposée. N’utilisez « Affecter manuellement » que si ce débit ne règle aucune facture saisie.'
				);
			}
			if (reason === 'invoice_candidate') {
				return i18nMsg(
					'reconciliation-failed-rule-masked-invoice-candidate',
					'La règle n’est pas appliquée : une facture fournisseur ouverte correspond à cette transaction. Rechargez la page et acceptez la facture proposée si ce débit la règle. N’utilisez « Affecter manuellement » que si ce débit ne règle aucune facture saisie.'
				);
			}
			if (reason === 'invoice_slightly_overpaid') {
				return i18nMsg(
					'reconciliation-failed-rule-masked-overpaid',
					'La règle n’est pas appliquée : une facture client que ce paiement dépasse de quelques centimes lui correspond. Enregistrez le règlement depuis la fiche de la facture, puis rattachez-y la transaction. N’utilisez « Affecter manuellement » que si ce paiement ne règle aucune facture.'
				);
			}
			return i18nMsg(
				'reconciliation-failed-rule-masked',
				'La règle n’est pas appliquée : une facture correspond à cette transaction. Rechargez la page et traitez la facture. N’utilisez « Affecter manuellement » que si cette transaction ne règle aucune facture saisie.'
			);
		}
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
		case 'RECONCILIATION_TRANSACTION_NOT_CREDIT':
			return i18nMsg(
				'reconciliation-failed-transaction-not-credit',
				'Cette transaction n’est pas une entrée d’argent (débit ou montant nul) : elle ne peut pas régler une facture client.'
			);
		// Story 30-1b (#686, AC-b7) — le symétrique fournisseur.
		case 'RECONCILIATION_TRANSACTION_NOT_DEBIT':
			return i18nMsg(
				'reconciliation-failed-transaction-not-debit',
				'Cette transaction n’est pas une sortie d’argent (crédit ou montant nul) : elle ne peut pas régler une facture fournisseur.'
			);
		case 'RECONCILIATION_TRANSACTION_NOT_PENDING':
			return i18nMsg(
				'reconciliation-failed-transaction-not-pending',
				'Cette transaction n’est plus en attente de rapprochement.'
			);
		case 'ROUNDING_ACCOUNT_NOT_CONFIGURED':
			return i18nMsg(
				'error-rounding-account-not-configured',
				"Ce paiement solde la facture au centime, mais aucun compte de différences d'arrondi utilisable n'est désigné : choisissez-en un dans Administration → Facturation."
			);
		case 'SETTLEMENT_COUNTERPARTY_IS_CLAIM_ACCOUNT':
			// Story 15-6b (#474) : l'écriture serait `D 1100 / C 1100`. Deux remèdes selon le
			// compte en cause — le seul `rounding` renvoie aux réglages ; tout autre rôle est le
			// compte de banque (`counterparty`), seul autre rôle que pose l'acceptation.
			if (failureRole(details) === 'rounding') {
				return i18nMsg(
					'reconciliation-failed-rounding-account-is-claim-account',
					'Le compte d’arrondi désigné est le compte débiteurs de la facture : un administrateur doit en désigner un autre dans Administration → Facturation.'
				);
			}
			// Story 30-1b (P1 R6) — pour une facture fournisseur, la dette est au compte
			// créanciers (`details.claim = "payable"`).
			if (failureClaim(details) === 'payable') {
				return i18nMsg(
					'reconciliation-failed-counterparty-is-payable-account',
					'Le compte bancaire est lié au compte créanciers de la facture : reliez-le à son propre compte de banque.'
				);
			}
			return i18nMsg(
				'reconciliation-failed-counterparty-is-claim-account',
				'Le compte bancaire est lié au compte débiteurs de la facture : reliez-le à son propre compte de banque.'
			);
		case 'VALIDATION_ERROR':
			return i18nMsg('error-validation', 'Erreur de validation');
		default:
			return i18nMsg('reconciliation-failed-unknown', 'Refus non reconnu ({ $code }).', { code });
	}
}
