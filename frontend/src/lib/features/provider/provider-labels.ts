/**
 * Story 40-7d (AC-3) — libellés **communs** des pages des prestataires : le
 * refus d'une proposition périmée et les motifs « à examiner » que la revue
 * commune (38-11c2) sert pour tout prestataire. Clés `provider-*`, servies par
 * les catalogues depuis la 38-11c2.
 *
 * ⚠️ **Les clés sont écrites en toutes lettres** (jamais construites par
 * gabarit) : `i18n-keys.test.ts` les voit dès qu'elles manquent d'un catalogue.
 * Les autres refus sont **délégués** à `failedProposalLabel` (rapprochement
 * générique), qui n'est pas étendu.
 */
import { i18nMsg } from '$lib/shared/utils/i18n.svelte';
import { failedProposalLabel } from '$lib/features/reconciliation/failed-proposal-label';
import { fxMonthLabel } from '$lib/features/journal-entries/fx-refusals';
import type { AccountResponse } from '$lib/features/accounts/accounts.types';
import type { ProviderFinding } from './provider-review.types';

/**
 * Codes du moteur d'acceptation commun (`crates/kesh-api/src/routes/provider_accept.rs`) que cette
 * page traduit elle-même (`provider-failed-*`) ; tout autre code passe par `failedProposalLabel`.
 * Exportée pour le test, qui la compare à la source Rust.
 */
export const PROVIDER_COMMON_FAILED_CODES = [
	'PROVIDER_PROPOSAL_STALE',
	'FX_ACCOUNT_INVALID',
	'FX_RATE_INVALID',
	'NOT_FOUND',
	'OPTIMISTIC_LOCK_CONFLICT',
	'RECONCILIATION_ENTRY_NOT_LINKABLE',
	'COUNTERPARTY_CURRENCY_MISMATCH',
	'PROVIDER_TRANSACTION_ALREADY_POSTED',
	'RATE_UNAVAILABLE',
] as const;

/** Libellé d'un refus de `POST /<prestataire>/accept` (hors codes propres au prestataire). */
export function providerFailedLabel(code: string, details?: unknown): string {
	switch (code) {
		case 'PROVIDER_PROPOSAL_STALE':
			return i18nMsg(
				'provider-failed-proposal-stale',
				"Cette proposition n'est plus valable : la transaction, l'écriture, la règle ou les réglages du compte ont changé depuis la revue. Rechargez la revue.",
			);
		case 'FX_ACCOUNT_INVALID':
			return i18nMsg(
				'provider-failed-fx-account-invalid',
				'Aucun compte de conversion utilisable n’est désigné : un administrateur doit en choisir un dans les paramètres du multi-devises.',
			);
		case 'FX_RATE_INVALID':
			return i18nMsg(
				'provider-failed-fx-rate-invalid',
				'Le cours de la conversion ne se calcule pas à partir des deux montants : passez l’écriture à la main.',
			);
		case 'NOT_FOUND':
			return i18nMsg(
				'provider-failed-not-found',
				'La transaction ou l’écriture visée est introuvable : elle a été supprimée entre-temps. Rechargez la page.',
			);
		case 'OPTIMISTIC_LOCK_CONFLICT':
			return i18nMsg(
				'provider-failed-optimistic-lock',
				'La transaction a été modifiée entre-temps. Rechargez la page.',
			);
		case 'RECONCILIATION_ENTRY_NOT_LINKABLE':
			return i18nMsg(
				'provider-failed-entry-not-linkable',
				'L’écriture de vente n’est plus rattachable à cette transaction (déjà rattachée, contre-passée ou modifiée). Rechargez la page.',
			);
		// La règle désigne un compte dans une devise qui n'est ni le CHF ni celle de la transaction
		// (voie des règles du moteur commun, sans effet pour Stripe).
		case 'COUNTERPARTY_CURRENCY_MISMATCH': {
			const d = details && typeof details === 'object' ? (details as Record<string, unknown>) : {};
			return i18nMsg(
				'provider-failed-counterparty-currency-mismatch',
				'Cette règle désigne un compte tenu en { $accountCurrency } : elle ne s’applique qu’aux transactions en { $accountCurrency }, et celle-ci est en { $transactionCurrency }. Choisissez un compte en CHF ou dans la devise de la transaction.',
				{
					accountCurrency: typeof d.accountCurrency === 'string' ? d.accountCurrency : '',
					transactionCurrency: typeof d.transactionCurrency === 'string' ? d.transactionCurrency : '',
				},
			);
		}
		// Une transaction de prestataire n'est écrite qu'une fois : déjà comptabilisée de l'autre
		// côté, elle se rattache à l'écriture existante (`FAILED_ALREADY_POSTED`).
		case 'PROVIDER_TRANSACTION_ALREADY_POSTED':
			return i18nMsg(
				'provider-failed-already-posted',
				'La transaction est déjà comptabilisée (l’autre côté est rattaché à son écriture) : rien n’a été écrit. Rechargez la page et rattachez-la à cette écriture.',
			);
		// Story 39-11d2 (AC-4) — le cours mensuel retenu manque (frais ou règle sur un solde en
		// devise, conversion entre deux devises étrangères, part en devise d'un paiement groupé) :
		// la phrase renvoie aux cours de change ; le libellé générique (« gardez la ligne comme
		// cours saisi ») ne convient pas à une proposition. `details` absents : phrase sans valeurs.
		case 'RATE_UNAVAILABLE': {
			const d = details && typeof details === 'object' ? (details as Record<string, unknown>) : {};
			return i18nMsg(
				'provider-failed-rate-unavailable',
				'Aucun cours mensuel { $currency } retenu pour { $month } : récupérez ou saisissez ce cours dans les cours de change, puis acceptez de nouveau.',
				{
					currency: typeof d.currency === 'string' ? d.currency : '',
					month: typeof d.month === 'string' ? fxMonthLabel(d.month) : '',
				},
			);
		}
		default:
			return failedProposalLabel(code, details);
	}
}

/** Libellé d'un motif « à examiner » commun à tous les prestataires ; `null` pour un autre motif. */
export function providerToReviewLabel(code: string): string | null {
	switch (code) {
		case 'ambiguous_sale':
			return i18nMsg(
				'provider-review-ambiguous-sale',
				'Plusieurs écritures de vente portent cet identifiant : choisissez celle à rattacher.',
			);
		case 'fee_account_missing':
			return i18nMsg(
				'provider-review-fee-account-missing',
				'Frais sans compte de frais : réglez le compte de frais du compte bancaire.',
			);
		// Une transaction déjà comptabilisée de l'autre côté (correctif des prestataires) : elle se
		// rattache à l'écriture existante, elle ne s'écrit pas une seconde fois.
		case 'already_posted':
			return i18nMsg(
				'provider-review-already-posted',
				'Déjà comptabilisée : l’écriture existe et l’autre côté y est rattaché. Rattachez cette transaction à l’écriture, ne l’écrivez pas une seconde fois.',
			);
		default:
			return null;
	}
}

/**
 * L'écriture déjà passée que nomme un motif « à examiner » `already_posted`
 * (`details.journalEntryId`) ; `null` pour tout autre motif ou sans identifiant numérique.
 * La page commune y met le lien « Écriture déjà passée » à la place de « Écriture manuelle ».
 */
export function alreadyPostedEntryId(f: ProviderFinding): number | null {
	const id = f.details?.journalEntryId;
	return f.code === 'already_posted' && typeof id === 'number' ? id : null;
}

/**
 * Story 38-11d (revue P1, M3), commune aux prestataires depuis la 40-7d — l'écriture d'un écart « montant ou devise différents » est-elle
 * déjà rattachée à une autre transaction (`details.alreadyLinked`) ? Le serveur range ce cas
 * sous le même code ; l'écran lui donne son propre libellé.
 */
export function isAlreadyLinked(f: ProviderFinding): boolean {
	return f.code === 'amount_or_currency_mismatch' && f.details?.alreadyLinked === true;
}

/**
 * Ce qu'un écart « montant ou devise différents » attend et trouve ; `null` pour un autre écart,
 * des détails absents, ou une écriture déjà rattachée de même montant et devise (le libellé dit
 * tout). Écriture sans débit au compte : « aucun débit trouvé », jamais « trouvé » suivi de rien.
 */
export function providerMismatchDetails(f: ProviderFinding): string | null {
	const d = f.details;
	if (!d || f.code !== 'amount_or_currency_mismatch') return null;
	const s = (k: string) => (d[k] === null || d[k] === undefined ? '' : String(d[k]));
	const expected = `${s('expectedAmount')} ${s('expectedCurrency')}`.trim();
	if (s('foundAmount') === '') {
		return i18nMsg('provider-gap-mismatch-details-none', 'Attendu { $expected }, aucun débit trouvé dans l’écriture', {
			expected,
		});
	}
	const found = `${s('foundAmount')} ${s('foundCurrency')}`.trim();
	if (isAlreadyLinked(f) && found === expected) return null;
	return i18nMsg('provider-gap-mismatch-details', 'Attendu { $expected }, trouvé { $found }', { expected, found });
}

/**
 * Story 39-11d2 — un compte du plan nommé « numéro — nom » parmi `accounts`,
 * `#<id>` s'il n'y est pas (lecture best-effort) : la colonne « Ce qui sera
 * fait » des pages des prestataires.
 */
export function accountNumberName(accounts: readonly AccountResponse[], id: number): string {
	const a = accounts.find((x) => x.id === id);
	return a ? `${a.number} — ${a.name}` : `#${id}`;
}
