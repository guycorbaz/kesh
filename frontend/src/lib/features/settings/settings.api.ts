import { apiClient } from '$lib/shared/utils/api-client';
import type { CompanyCurrentResponse } from './settings.types';

export async function fetchCompanyCurrent(): Promise<CompanyCurrentResponse> {
	return apiClient.get<CompanyCurrentResponse>('/api/v1/companies/current');
}
