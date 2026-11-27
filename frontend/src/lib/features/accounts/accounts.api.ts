import { apiClient } from '$lib/shared/utils/api-client';
import type {
	AccountResponse,
	CreateAccountRequest,
	UpdateAccountRequest,
	ArchiveAccountRequest,
	ReactivateAccountRequest,
} from './accounts.types';
import type { ApplyGroupsRequest, ApplyGroupsResponse } from './group-proposals';

export async function fetchAccounts(includeArchived = false): Promise<AccountResponse[]> {
	return apiClient.get<AccountResponse[]>(
		`/api/v1/accounts?includeArchived=${includeArchived}`
	);
}

export async function createAccount(req: CreateAccountRequest): Promise<AccountResponse> {
	return apiClient.post<AccountResponse>('/api/v1/accounts', req);
}

export async function updateAccount(
	id: number,
	req: UpdateAccountRequest
): Promise<AccountResponse> {
	return apiClient.put<AccountResponse>(`/api/v1/accounts/${id}`, req);
}

export async function archiveAccount(
	id: number,
	req: ArchiveAccountRequest
): Promise<AccountResponse> {
	return apiClient.put<AccountResponse>(`/api/v1/accounts/${id}/archive`, req);
}

/**
 * Réactive un compte archivé (Story 14-3a, #269).
 *
 * `PUT` par symétrie locale avec `archiveAccount` — la feature Projets utilise
 * `POST /unarchive`, mais on privilégie ici la cohérence de la ressource.
 *
 * Peut échouer en 409 : parent archivé, conflit de version, ou
 * `ACCOUNT_ROLE_ALREADY_ASSIGNED` si le rôle singleton du compte a été repris
 * par un autre compte pendant son archivage.
 */
export async function reactivateAccount(
	id: number,
	req: ReactivateAccountRequest
): Promise<AccountResponse> {
	return apiClient.put<AccountResponse>(`/api/v1/accounts/${id}/reactivate`, req);
}

/**
 * Story 46-5b — crée des comptes de groupe et y range des comptes, en une
 * transaction, tout ou rien (`POST /api/v1/accounts/groups`, Story 46-5a).
 * Un refus n'a rien écrit ; il porte `details.groupStep` (codes existants) ou
 * `details.groupIndex` (`ACCOUNT_GROUP_INVALID`). Fermée aux clés d'API.
 */
export async function applyGroups(req: ApplyGroupsRequest): Promise<ApplyGroupsResponse> {
	return apiClient.post<ApplyGroupsResponse>('/api/v1/accounts/groups', req);
}
