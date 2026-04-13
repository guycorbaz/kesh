/**
 * Client API typé pour le catalogue produits (Story 4.2).
 */

import { apiClient } from '$lib/shared/utils/api-client';
import type {
	ArchiveProductRequest,
	CreateProductRequest,
	ListProductsQuery,
	ListResponse,
	ProductResponse,
	UpdateProductRequest
} from './products.types';

function buildQueryString(q: ListProductsQuery): string {
	const params = new URLSearchParams();
	if (q.search) params.set('search', q.search);
	if (q.includeArchived) params.set('includeArchived', 'true');
	if (q.sortBy) params.set('sortBy', q.sortBy);
	if (q.sortDirection) params.set('sortDirection', q.sortDirection);
	if (q.limit !== undefined) params.set('limit', String(q.limit));
	if (q.offset !== undefined) params.set('offset', String(q.offset));
	const s = params.toString();
	return s ? `?${s}` : '';
}

export async function listProducts(
	query: ListProductsQuery = {}
): Promise<ListResponse<ProductResponse>> {
	return apiClient.get<ListResponse<ProductResponse>>(
		`/api/v1/products${buildQueryString(query)}`
	);
}

export async function getProduct(id: number): Promise<ProductResponse> {
	return apiClient.get<ProductResponse>(`/api/v1/products/${id}`);
}

export async function createProduct(req: CreateProductRequest): Promise<ProductResponse> {
	return apiClient.post<ProductResponse>('/api/v1/products', req);
}

export async function updateProduct(
	id: number,
	req: UpdateProductRequest
): Promise<ProductResponse> {
	return apiClient.put<ProductResponse>(`/api/v1/products/${id}`, req);
}

export async function archiveProduct(
	id: number,
	req: ArchiveProductRequest
): Promise<ProductResponse> {
	return apiClient.put<ProductResponse>(`/api/v1/products/${id}/archive`, req);
}
