// Story 8-2 — Client API bank-profile CRUD.

import { apiClient } from '$lib/shared/utils/api-client';
import type {
	BankProfile,
	BankProfileListResponse,
	CreateOrUpdateBankProfileRequest,
} from './bank-profile.types';

export async function listBankProfiles(
	page = 1,
	perPage = 50,
): Promise<BankProfileListResponse> {
	return apiClient.get<BankProfileListResponse>(
		`/api/v1/bank-profiles?page=${page}&per_page=${perPage}`,
	);
}

export async function getBankProfile(id: number): Promise<BankProfile> {
	return apiClient.get<BankProfile>(`/api/v1/bank-profiles/${id}`);
}

export async function createBankProfile(
	req: CreateOrUpdateBankProfileRequest,
): Promise<BankProfile> {
	return apiClient.post<BankProfile>('/api/v1/bank-profiles', req);
}

export async function updateBankProfile(
	id: number,
	req: CreateOrUpdateBankProfileRequest,
): Promise<BankProfile> {
	return apiClient.put<BankProfile>(`/api/v1/bank-profiles/${id}`, req);
}

export async function deleteBankProfile(id: number): Promise<void> {
	return apiClient.delete(`/api/v1/bank-profiles/${id}`);
}
