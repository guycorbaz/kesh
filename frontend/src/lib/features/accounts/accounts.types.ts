export type AccountType = 'Asset' | 'Liability' | 'Revenue' | 'Expense';

export interface AccountResponse {
	id: number;
	companyId: number;
	number: string;
	name: string;
	accountType: AccountType;
	parentId: number | null;
	active: boolean;
	version: number;
	createdAt: string;
	updatedAt: string;
}

export interface CreateAccountRequest {
	number: string;
	name: string;
	accountType: AccountType;
	parentId?: number | null;
}

export interface UpdateAccountRequest {
	name: string;
	accountType: AccountType;
	version: number;
}

export interface ArchiveAccountRequest {
	version: number;
}
