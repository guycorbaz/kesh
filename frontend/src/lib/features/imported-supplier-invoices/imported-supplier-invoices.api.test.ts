// Story v017-5 (#854, AC-6, T11) — le délai de l'import du dossier : l'écran attend cinq
// minutes (aligné sur le proxy documenté), sans rejeu, au lieu des 30 s par défaut.
import { describe, it, expect, vi, beforeEach } from 'vitest';

vi.mock('$lib/shared/utils/api-client', () => ({
	apiClient: { get: vi.fn(), post: vi.fn(), delete: vi.fn(), getBlob: vi.fn() },
}));

import { apiClient } from '$lib/shared/utils/api-client';
import { INBOX_IMPORT_TIMEOUT_MS, triggerInboxImport } from './imported-supplier-invoices.api';

const client = vi.mocked(apiClient);

describe('triggerInboxImport', () => {
	beforeEach(() => {
		vi.clearAllMocks();
	});

	it('attend cinq minutes', () => {
		expect(INBOX_IMPORT_TIMEOUT_MS).toBe(300_000);
	});

	it('passe le délai de cinq minutes et interdit le rejeu', async () => {
		client.post.mockResolvedValueOnce({ accepted: [], failed: [], warnings: [] });
		await triggerInboxImport();
		expect(client.post).toHaveBeenCalledWith(
			'/api/v1/inbox-import',
			{},
			{ timeoutMs: INBOX_IMPORT_TIMEOUT_MS, retry: false },
		);
	});
});
