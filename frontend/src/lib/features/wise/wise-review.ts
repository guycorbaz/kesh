/**
 * Story 39-11d2 (AC-2) — règles de la page Wise, hors composant pour être
 * testées seules (patron de `stripe-review.ts`) : la transaction et la clé
 * d'une proposition, le corps de l'acceptation, l'ordre d'envoi, et les
 * **sous-lignes** d'un paiement par carte groupé (les lignes après la
 * principale) d'une proposition ou d'une ligne « à examiner ».
 */
import { selectedInDateOrder } from '$lib/features/provider/provider-review';
import type { ProviderFinding, ProviderReviewTransaction } from '$lib/features/provider/provider-review.types';
import type { WiseAcceptProposal, WiseProposal } from './wise.types';

/** La transaction qui identifie une proposition : la débitée d'une conversion, la principale d'un groupe. */
export function wiseProposalTransactionId(p: WiseProposal): number {
	switch (p.type) {
		case 'conversion':
			return p.debitTransactionId;
		case 'card_group':
			return p.bankTransactionIds[0] ?? p.transaction.id;
		default:
			return p.bankTransactionId;
	}
}

/** Clé stable d'une proposition (cases à cocher) : `<type>-<transaction>`. */
export function wiseProposalKey(p: WiseProposal): string {
	return `${p.type}-${wiseProposalTransactionId(p)}`;
}

/** Le corps d'acceptation d'une proposition : ses seuls champs (un groupe sans `lines` ni `merchantAmount`). */
export function wiseAcceptBody(p: WiseProposal): WiseAcceptProposal {
	switch (p.type) {
		case 'fee':
			return { type: 'fee', bankTransactionId: p.bankTransactionId, feeAccountId: p.feeAccountId };
		case 'conversion':
			return {
				type: 'conversion',
				debitTransactionId: p.debitTransactionId,
				creditTransactionId: p.creditTransactionId,
			};
		case 'link_entry':
			return { type: 'link_entry', bankTransactionId: p.bankTransactionId, journalEntryId: p.journalEntryId };
		case 'rule':
			return {
				type: 'rule',
				bankTransactionId: p.bankTransactionId,
				ruleId: p.ruleId,
				counterpartyAccountId: p.counterpartyAccountId,
			};
		case 'card_group':
			return {
				type: 'card_group',
				bankTransactionIds: [...p.bankTransactionIds],
				ruleId: p.ruleId,
				counterpartyAccountId: p.counterpartyAccountId,
			};
	}
}

/** Les corps à envoyer pour la sélection, dans l'ordre des dates (puis de la transaction). */
export function wiseSelectedBodies(
	proposals: readonly WiseProposal[],
	selected: ReadonlySet<string>,
): WiseAcceptProposal[] {
	return selectedInDateOrder(proposals, selected, wiseProposalKey, wiseProposalTransactionId).map(wiseAcceptBody);
}

/** Les sous-lignes d'une proposition : les lignes d'un groupe après la principale (`lines[1..]`), aucune sinon. */
export function wiseProposalLines(p: WiseProposal): ProviderReviewTransaction[] {
	return p.type === 'card_group' ? p.lines.slice(1) : [];
}

/** Une transaction résumée par la revue ? (garde des `details.lines`, lus sans type) */
function isSummary(v: unknown): v is ProviderReviewTransaction {
	return typeof v === 'object' && v !== null && typeof (v as { id?: unknown }).id === 'number';
}

/**
 * Les sous-lignes d'une ligne « à examiner » d'un paiement groupé (motifs
 * `card_group_*`, `details.lines`) : ses lignes hors la principale — la
 * transaction du motif, qui garde la ligne du tableau ; aucune pour un autre motif.
 */
export function wiseFindingLines(f: ProviderFinding): ProviderReviewTransaction[] {
	if (!f.code.startsWith('card_group_')) return [];
	const lines = f.details?.lines;
	if (!Array.isArray(lines)) return [];
	return lines.filter(isSummary).filter((t) => t.id !== f.transaction?.id);
}
