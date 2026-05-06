// Story 8-1b — Client API bank-import.
// Story 8-3 — extension : 3 nouveaux flags confirm + bankProfileId (KF #70).

import { apiClient } from '$lib/shared/utils/api-client';
import type {
	BankImportDetailResponse,
	BankImportListResponse,
	BankImportPreviewResponse,
	BankImportResponse,
} from './bank-import.types';

/// Story 8-3 — options du POST /preview et /bank-imports.
export interface BankImportFlags {
	/// 8-1b — confirmBalanceMismatch.
	confirmBalanceMismatch?: boolean;
	/// 8-2 — bankProfileId explicite (CSV uniquement, override auto-match).
	bankProfileId?: number;
	/// 8-2 — confirmEncodingMismatch (KF #70 frontend wiring).
	confirmEncodingMismatch?: boolean;
	/// 8-3 — confirmDuplicateFile.
	confirmDuplicateFile?: boolean;
	/// 8-3 — confirmDuplicateLines : 'skip' (default) ou 'import'.
	confirmDuplicateLines?: 'skip' | 'import';
	/// 8-3 — confirmPartialImport.
	confirmPartialImport?: boolean;
}

function appendFlags(form: FormData, flags?: BankImportFlags): void {
	if (!flags) return;
	if (flags.confirmBalanceMismatch) form.append('confirmBalanceMismatch', 'true');
	// L1 (Pass 1 review) — `if (flags.bankProfileId)` skipperait `0`. Les
	// IDs DB MariaDB AUTO_INCREMENT commencent à 1 mais on prend la
	// précaution explicite : null/undefined → skip, sinon append.
	if (flags.bankProfileId !== undefined && flags.bankProfileId !== null) {
		form.append('bankProfileId', flags.bankProfileId.toString());
	}
	if (flags.confirmEncodingMismatch) form.append('confirmEncodingMismatch', 'true');
	if (flags.confirmDuplicateFile) form.append('confirmDuplicateFile', 'true');
	// L9 (Pass 1 review) — n'envoie que si différent de la valeur par
	// défaut backend (`Skip`). Évite de polluer chaque requête multipart
	// avec `confirmDuplicateLines=skip` (équivalent à omettre le champ).
	if (flags.confirmDuplicateLines && flags.confirmDuplicateLines !== 'skip') {
		form.append('confirmDuplicateLines', flags.confirmDuplicateLines);
	}
	if (flags.confirmPartialImport) form.append('confirmPartialImport', 'true');
}

export async function previewBankImport(
	file: File,
	bankAccountId: number,
	flags?: BankImportFlags,
): Promise<BankImportPreviewResponse> {
	const form = new FormData();
	form.append('bankAccountId', bankAccountId.toString());
	form.append('file', file);
	appendFlags(form, flags);
	return apiClient.postFormData<BankImportPreviewResponse>(
		'/api/v1/bank-imports/preview',
		form,
	);
}

export async function createBankImport(
	file: File,
	bankAccountId: number,
	flags?: BankImportFlags,
): Promise<BankImportResponse> {
	const form = new FormData();
	form.append('bankAccountId', bankAccountId.toString());
	form.append('file', file);
	appendFlags(form, flags);
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
