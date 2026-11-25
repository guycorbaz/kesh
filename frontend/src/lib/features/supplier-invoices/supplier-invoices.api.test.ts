// Story 48-12b (#975, règle du drapeau V4-1, V5-1) — `unvalidateSupplierInvoice` n'ajoute
// `confirmNotDuplicate: true` au corps que si son troisième argument vaut EXACTEMENT `true` :
// un événement de clic, un objet ou `false` laissent le corps `{ version }`, et le refus de
// doublon d'un repassage en brouillon n'est jamais levé en silence.
//
// Mutation qu'il attrape : « drapeau de l'API testé par sa vérité » (`confirmNotDuplicate ?`
// au lieu de `=== true`) — le corps de l'objet ou de l'événement porterait le drapeau.
import { describe, it, expect, vi, beforeEach } from 'vitest';

vi.mock('$lib/shared/utils/api-client', () => ({
	apiClient: { get: vi.fn(), post: vi.fn(), put: vi.fn(), delete: vi.fn() },
}));

import { apiClient } from '$lib/shared/utils/api-client';
import { unvalidateSupplierInvoice } from './supplier-invoices.api';

const client = vi.mocked(apiClient);

describe('unvalidateSupplierInvoice — la règle du drapeau', () => {
	beforeEach(() => {
		vi.clearAllMocks();
		client.post.mockResolvedValue({});
	});

	it('sans troisième argument : corps { version }', async () => {
		await unvalidateSupplierInvoice(9, 3);
		expect(client.post).toHaveBeenCalledWith('/api/v1/supplier-invoices/9/unvalidate', { version: 3 });
	});

	it.each([
		['undefined', undefined],
		['false', false],
		['un objet vide', {}],
		['un objet de forme MouseEvent', { type: 'click', isTrusted: true, detail: 1 }],
		['la chaîne « true »', 'true'],
		['1', 1],
	])('troisième argument %s : corps { version }, sans drapeau', async (_label, flag) => {
		await unvalidateSupplierInvoice(9, 3, flag);
		expect(client.post).toHaveBeenCalledWith('/api/v1/supplier-invoices/9/unvalidate', { version: 3 });
	});

	it('true seulement : corps { version, confirmNotDuplicate: true }', async () => {
		await unvalidateSupplierInvoice(9, 3, true);
		expect(client.post).toHaveBeenCalledWith('/api/v1/supplier-invoices/9/unvalidate', {
			version: 3,
			confirmNotDuplicate: true,
		});
	});
});
