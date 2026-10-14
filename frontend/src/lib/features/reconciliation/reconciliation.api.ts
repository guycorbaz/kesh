// Story 8-4 — Client API réconciliation.

import { apiClient } from '$lib/shared/utils/api-client';
import type {
	AcceptProposalInput,
	AcceptResponse,
	CancelReconciliationResponse,
	GetProposalsResponse,
	LinkCandidatesResponse,
	LinkResponse,
	ManualMatchResponse,
	ReconciliationTransactionResponse,
	RejectResponse,
	SplitProposalLine,
	SplitResponse,
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
	projectId?: number | null,
): Promise<ManualMatchResponse> {
	const body: Record<string, unknown> = {
		bankAccountId,
		bankTransactionId,
		counterpartyAccountId,
	};
	if (description !== undefined && description !== '') body.description = description;
	// P-L2 Pass 1 code review : filtrer `valueDate === ''` aussi (le
	// datepicker HTML retourne `''` quand vidé, et le backend ne sait
	// pas désérialiser `""` en `Option<NaiveDate>` → 422 ValidationError).
	if (valueDate !== undefined && valueDate !== '') body.valueDate = valueDate;
	// Story 19-5 — projet analytique document-level (recopié sur les 2 lignes).
	if (projectId !== undefined && projectId !== null) body.projectId = projectId;
	return apiClient.post<ManualMatchResponse>(
		'/api/v1/reconciliation/manual',
		body,
	);
}

/**
 * Story 8-5a-bis FR48 — éclatement de transaction agrégée en N lignes
 * contreparties + 1 ligne banque.
 *
 * Le compte ledger banque est résolu **serveur-side** via
 * `bank_account.journal_account_id` (foundation 8-5a-zero) — donc le
 * body NE contient PAS `bankLedgerAccountId`. Si NULL → 412
 * BANK_ACCOUNT_NOT_CONFIGURED.
 */
export async function splitTransaction(
	bankAccountId: number,
	bankTransactionId: number,
	splits: SplitProposalLine[],
	valueDate?: string,
): Promise<SplitResponse> {
	const body: Record<string, unknown> = {
		bankAccountId,
		bankTransactionId,
		splits,
	};
	if (valueDate !== undefined && valueDate !== '') body.valueDate = valueDate;
	return apiClient.post<SplitResponse>('/api/v1/reconciliation/split', body);
}

/**
 * Story 25-3-b (#418) — la transaction et ce qui empêche d'annuler son
 * rapprochement. Lu **au clic**, pour une seule transaction : calculé pour
 * chaque ligne d'un import, il coûterait des milliers de requêtes.
 */
export async function getReconciliationTransaction(
	bankTransactionId: number,
): Promise<ReconciliationTransactionResponse> {
	return apiClient.get<ReconciliationTransactionResponse>(
		`/api/v1/reconciliation/transactions/${bankTransactionId}`,
	);
}

/**
 * Story 25-3-b (#418) — annule un rapprochement : l'écriture est
 * contre-passée (datée du jour), la transaction redevient à rapprocher, et le
 * règlement d'une facture est retiré.
 */
export async function cancelReconciliation(
	bankTransactionId: number,
): Promise<CancelReconciliationResponse> {
	return apiClient.post<CancelReconciliationResponse>(
		`/api/v1/reconciliation/transactions/${bankTransactionId}/cancel`,
		{},
	);
}

/**
 * Story 28-5 (#526) — annule le rejet d'une transaction : elle redevient
 * proposable à la réconciliation. Aucune écriture comptable ; le geste est
 * tracé au journal d'audit. Réponse `204` sans corps. Refus : `404`
 * (introuvable), `409 RECONCILIATION_TRANSACTION_NOT_REJECTED` (non rejetée
 * ou rapprochée), `409 RECONCILIATION_ACCOUNT_LOCKED`.
 */
export async function unrejectTransaction(bankTransactionId: number): Promise<void> {
	return apiClient.post<void>(
		`/api/v1/reconciliation/transactions/${bankTransactionId}/unreject`,
		{},
	);
}

/**
 * Story 29-8 (#670) — les règlements clients par virement, déjà saisis depuis la fiche de la
 * facture, auxquels la transaction se rattache : même compte bancaire, montant égal ou
 * dépassé de 5 centimes au plus, règlement pas encore rapproché. Triés par écart puis par
 * proximité de date ; rien n'est présélectionné.
 */
export async function getLinkCandidates(
	bankTransactionId: number,
): Promise<LinkCandidatesResponse> {
	return apiClient.get<LinkCandidatesResponse>(
		`/api/v1/reconciliation/transactions/${bankTransactionId}/link-candidates`,
	);
}

/**
 * Story 29-8 (#670) — rattache la transaction au règlement : elle passe rapprochée en
 * pointant l'écriture du règlement, **sans aucune écriture**. Refus notable :
 * `409 RECONCILIATION_SETTLEMENT_NOT_LINKABLE` (la liste a changé — la relire).
 */
export async function linkTransaction(
	bankTransactionId: number,
	settlementId: number,
): Promise<LinkResponse> {
	return apiClient.post<LinkResponse>(
		`/api/v1/reconciliation/transactions/${bankTransactionId}/link`,
		{ settlementId },
	);
}
