import { apiClient } from '$lib/shared/utils/api-client';
import type {
	AccountResponse,
	CreateAccountRequest,
	UpdateAccountRequest,
	ArchiveAccountRequest,
} from './accounts.types';

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
