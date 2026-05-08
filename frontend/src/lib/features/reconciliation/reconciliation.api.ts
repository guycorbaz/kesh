// Story 8-4 — Client API réconciliation.

import { apiClient } from '$lib/shared/utils/api-client';
import type {
	AcceptProposalInput,
	AcceptResponse,
	GetProposalsResponse,
	ManualMatchResponse,
	RejectResponse,
} from './reconciliation.types';

export async function getProposals(
	bankAccountId: number,
	limit = 100,
): Promise<GetProposalsResponse> {
	const qs = new URLSearchParams();
	qs.set('bankAccountId', bankAccountId.toString());
	qs.set('limit', limit.toString());
	return apiClient.get<GetProposalsResponse>(
		`/api/v1/reconciliation/proposals?${qs.toString()}`,
	);
}

export async function acceptProposals(
	bankAccountId: number,
	proposals: AcceptProposalInput[],
): Promise<AcceptResponse> {
	return apiClient.post<AcceptResponse>('/api/v1/reconciliation/accept', {
		bankAccountId,
		proposals,
	});
}

export async function rejectProposals(
	bankAccountId: number,
	bankTransactionIds: number[],
): Promise<RejectResponse> {
	return apiClient.post<RejectResponse>('/api/v1/reconciliation/reject', {
		bankAccountId,
		bankTransactionIds,
	});
}

/**
 * Story 8-5a-base FR45 — réconciliation manuelle.
 *
 * Le compte ledger banque est résolu **serveur-side** via
 * `bank_account.journal_account_id` (foundation 8-5a-zero) — donc le
 * body NE contient PAS `bankLedgerAccountId`. Si le bank_account n'a
 * pas son journal_account_id configuré → 412 BANK_ACCOUNT_NOT_CONFIGURED
 * (le user doit configurer via la page `/bank-accounts`).
 */
export async function manualMatchTransaction(
	bankAccountId: number,
	bankTransactionId: number,
	counterpartyAccountId: number,
	description?: string,
	valueDate?: string,
): Promise<ManualMatchResponse> {
	const body: Record<string, unknown> = {
		bankAccountId,
		bankTransactionId,
		counterpartyAccountId,
	};
	if (description !== undefined && description !== '') body.description = description;
	if (valueDate !== undefined) body.valueDate = valueDate;
	return apiClient.post<ManualMatchResponse>(
		'/api/v1/reconciliation/manual',
		body,
	);
}
