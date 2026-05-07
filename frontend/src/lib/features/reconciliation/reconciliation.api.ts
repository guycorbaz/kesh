// Story 8-4 — Client API réconciliation.

import { apiClient } from '$lib/shared/utils/api-client';
import type {
	AcceptProposalInput,
	AcceptResponse,
	GetProposalsResponse,
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
