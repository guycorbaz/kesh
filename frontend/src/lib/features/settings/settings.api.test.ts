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
import { updateCompanyContactDetails, updateCompanyEmail } from './settings.api';

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

describe('settings.api — updateCompanyContactDetails (16-3a, #151)', () => {
	// ⚠️ Le SEUL lien entre l'écran de réglages et la nouvelle route est ce
	// littéral de chemin. Une faute de frappe, un `/company/` au singulier, un
	// renommage côté Rust : `npm run check` valide le TypeScript contre lui-même,
	// et les tests Rust construisent leur propre URL — aucun gate ne le verrait.
	// Le défaut n'apparaîtrait qu'à l'usage, en 404 sur « Enregistrer ».
	// *(Revue de code, passe 3 — le jumeau e-mail avait ce test, pas celui-ci.)*
	it('PUT phone + website + version au bon path', async () => {
		(apiClient.put as ReturnType<typeof vi.fn>).mockResolvedValue({
			id: 1,
			phone: '+41 21 123 45 67',
			website: 'https://demo.ch',
			version: 4,
		});
		const company = await updateCompanyContactDetails({
			phone: '+41 21 123 45 67',
			website: 'https://demo.ch',
			version: 3,
		});
		expect(apiClient.put).toHaveBeenCalledWith('/api/v1/companies/current/contact-details', {
			phone: '+41 21 123 45 67',
			website: 'https://demo.ch',
			version: 3,
		});
		expect(company.version).toBe(4);
	});

	it('effacement : null transmis tel quel sur les deux champs', async () => {
		(apiClient.put as ReturnType<typeof vi.fn>).mockResolvedValue({
			id: 1,
			phone: null,
			website: null,
			version: 5,
		});
		await updateCompanyContactDetails({ phone: null, website: null, version: 4 });
		expect(apiClient.put).toHaveBeenCalledWith('/api/v1/companies/current/contact-details', {
			phone: null,
			website: null,
			version: 4,
		});
	});
});
