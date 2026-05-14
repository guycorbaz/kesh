// Story 8-5b T5.6 — tests minimaux Vitest sur rules.api.ts.
import { describe, expect, it, vi } from 'vitest';

vi.mock('$lib/shared/utils/api-client', () => ({
	apiClient: {
		get: vi.fn(),
		post: vi.fn(),
		patch: vi.fn(),
		delete: vi.fn(),
	},
}));

import { apiClient } from '$lib/shared/utils/api-client';
import { createRule, deleteRule, listRules, updateRule } from './rules.api';

describe('rules.api', () => {
	it('listRules forwards page + perPage + active filter', async () => {
		(apiClient.get as ReturnType<typeof vi.fn>).mockResolvedValue({
			items: [],
			total: 0,
			limit: 50,
			offset: 0,
		});
		await listRules(2, 25, true);
		expect(apiClient.get).toHaveBeenCalledWith(
			expect.stringContaining('page=2'),
		);
		expect(apiClient.get).toHaveBeenCalledWith(expect.stringContaining('perPage=25'));
		expect(apiClient.get).toHaveBeenCalledWith(expect.stringContaining('active=true'));
	});

	it('createRule POSTs the payload', async () => {
		(apiClient.post as ReturnType<typeof vi.fn>).mockResolvedValue({});
		await createRule({
			label: 'R1',
			matchType: 'counterparty_contains',
			matchValue: 'Swisscom',
			counterpartyAccountId: 42,
		});
		expect(apiClient.post).toHaveBeenCalledWith(
			'/api/v1/reconciliation/rules',
			expect.objectContaining({ matchValue: 'Swisscom' }),
		);
	});

	it('updateRule PATCHes with expectedVersion', async () => {
		(apiClient.patch as ReturnType<typeof vi.fn>).mockResolvedValue({});
		await updateRule(7, { expectedVersion: 2, label: 'New' });
		expect(apiClient.patch).toHaveBeenCalledWith(
			'/api/v1/reconciliation/rules/7',
			expect.objectContaining({ expectedVersion: 2 }),
		);
	});

	it('deleteRule DELETEs at the right path', async () => {
		(apiClient.delete as ReturnType<typeof vi.fn>).mockResolvedValue(undefined);
		await deleteRule(123);
		expect(apiClient.delete).toHaveBeenCalledWith('/api/v1/reconciliation/rules/123');
	});
});
