/**
 * Story 40-7d (AC-3) — règles de la page Stripe, hors composant pour être
 * testées seules : clé d'une proposition, corps de l'acceptation, ordre
 * d'envoi.
 */
import { selectedInDateOrder } from '$lib/features/provider/provider-review';
import type { StripeAcceptProposal, StripeProposal } from './stripe.types';

/** La transaction qui identifie une proposition. */
export function stripeProposalTransactionId(p: StripeProposal): number {
	return p.bankTransactionId;
}

/** Clé stable d'une proposition (cases à cocher). */
export function stripeProposalKey(p: StripeProposal): string {
	return `${p.type}-${p.bankTransactionId}`;
}

/** Le corps d'acceptation d'une proposition : ses seuls champs (le versement avec son compte de versement). */
export function stripeAcceptBody(p: StripeProposal): StripeAcceptProposal {
	switch (p.type) {
		case 'link_entry':
			return { type: 'link_entry', bankTransactionId: p.bankTransactionId, journalEntryId: p.journalEntryId };
		case 'fee':
			return { type: 'fee', bankTransactionId: p.bankTransactionId, feeAccountId: p.feeAccountId };
		case 'payout':
			return {
				type: 'payout',
				bankTransactionId: p.bankTransactionId,
				payoutBankAccountId: p.payoutBankAccountId,
			};
	}
}

/** Les corps à envoyer pour la sélection, dans l'ordre des dates (puis de la transaction). */
export function stripeSelectedBodies(
	proposals: readonly StripeProposal[],
	selected: ReadonlySet<string>,
): StripeAcceptProposal[] {
	return selectedInDateOrder(proposals, selected, stripeProposalKey, stripeProposalTransactionId).map(
		stripeAcceptBody,
	);
}
