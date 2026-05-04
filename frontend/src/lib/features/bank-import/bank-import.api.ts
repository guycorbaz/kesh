// Story 8-1b — Client API bank-import.

import { apiClient } from '$lib/shared/utils/api-client';
import type {
	BankImportDetailResponse,
	BankImportListResponse,
	BankImportPreviewResponse,
	BankImportResponse,
} from './bank-import.types';

export async function previewBankImport(
	file: File,
	bankAccountId: number,
): Promise<BankImportPreviewResponse> {
	const form = new FormData();
	form.append('bankAccountId', bankAccountId.toString());
	form.append('file', file);
	return apiClient.postFormData<BankImportPreviewResponse>(
		'/api/v1/bank-imports/preview',
		form,
	);
}

export async function createBankImport(
	file: File,
	bankAccountId: number,
	confirmBalanceMismatch = false,
): Promise<BankImportResponse> {
	const form = new FormData();
	form.append('bankAccountId', bankAccountId.toString());
	form.append('file', file);
	if (confirmBalanceMismatch) {
		form.append('confirmBalanceMismatch', 'true');
	}
	return apiClient.postFormData<BankImportResponse>('/api/v1/bank-imports', form);
}

export async function listBankImports(
	bankAccountId?: number,
	limit = 20,
	offset = 0,
): Promise<BankImportListResponse> {
	const qs = new URLSearchParams();
	if (bankAccountId) qs.set('bankAccountId', bankAccountId.toString());
	qs.set('limit', limit.toString());
	qs.set('offset', offset.toString());
	return apiClient.get<BankImportListResponse>(`/api/v1/bank-imports?${qs.toString()}`);
}

export async function getBankImportDetail(id: number): Promise<BankImportDetailResponse> {
	return apiClient.get<BankImportDetailResponse>(`/api/v1/bank-imports/${id}`);
}
