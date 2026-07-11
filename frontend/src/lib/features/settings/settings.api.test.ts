// Story 20-3b2 — tests Vitest du wrapper e-mail société (Reply-To).
import { describe, expect, it, vi } from 'vitest';

vi.mock('$lib/shared/utils/api-client', () => ({
	apiClient: {
		get: vi.fn(),
		post: vi.fn(),
		put: vi.fn(),
		patch: vi.fn(),
		delete: vi.fn(),
	},
}));

import { apiClient } from '$lib/shared/utils/api-client';
import { updateCompanyEmail } from './settings.api';

describe('settings.api — updateCompanyEmail (20-3b2)', () => {
	it('PUT email + version au bon path', async () => {
		(apiClient.put as ReturnType<typeof vi.fn>).mockResolvedValue({
			id: 1,
			email: 'info@pme.ch',
			version: 2,
		});
		const company = await updateCompanyEmail({ email: 'info@pme.ch', version: 1 });
		expect(apiClient.put).toHaveBeenCalledWith('/api/v1/companies/current/email', {
			email: 'info@pme.ch',
			version: 1,
		});
		expect(company.version).toBe(2);
	});

	it('effacement : email null transmis tel quel (Reply-To omis)', async () => {
		(apiClient.put as ReturnType<typeof vi.fn>).mockResolvedValue({ id: 1, email: null, version: 3 });
		await updateCompanyEmail({ email: null, version: 2 });
		expect(apiClient.put).toHaveBeenCalledWith('/api/v1/companies/current/email', {
			email: null,
			version: 2,
		});
	});
});
