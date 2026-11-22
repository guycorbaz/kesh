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
import type { ProviderFinding } from './provider-review.types';

/** Libellé d'un refus de `POST /<prestataire>/accept` (hors codes propres au prestataire). */
export function providerFailedLabel(code: string, details?: unknown): string {
	if (code === 'PROVIDER_PROPOSAL_STALE') {
		return i18nMsg(
			'provider-failed-proposal-stale',
			"Cette proposition n'est plus valable : la transaction, l'écriture, la règle ou les réglages du compte ont changé depuis la revue. Rechargez la revue.",
		);
	}
	return failedProposalLabel(code, details);
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
		default:
			return null;
	}
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
