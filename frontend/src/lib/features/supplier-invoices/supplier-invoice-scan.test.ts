import { describe, expect, it } from 'vitest';
import { scanToPrefill } from './supplier-invoice-scan';
import type { ScanQrResponse } from './supplier-invoices.types';

function resp(partial: Partial<ScanQrResponse>): ScanQrResponse {
	return {
		creditorIban: null,
		creditorQrIban: null,
		paymentReference: null,
		expectedPaymentAmount: null,
		currency: 'CHF',
		creditorName: '',
		creditorAddress: null,
		unstructuredMessage: null,
		...partial,
	};
}

describe('scanToPrefill', () => {
	it('mappe un QR-IBAN + QRR vers creditorQrIban (creditorIban vide)', () => {
		const p = scanToPrefill(
			resp({
				creditorQrIban: 'CH4431999123000889012',
				paymentReference: '210000000003139471430009017',
				expectedPaymentAmount: '199.95',
				creditorName: 'Fournisseur QR SA',
			}),
		);
		expect(p.creditorQrIban).toBe('CH4431999123000889012');
		expect(p.creditorIban).toBe('');
		expect(p.paymentReference).toBe('210000000003139471430009017');
		expect(p.expectedAmount).toBe('199.95');
		expect(p.creditorName).toBe('Fournisseur QR SA');
	});

	it('mappe un IBAN classique vers creditorIban (creditorQrIban vide)', () => {
		const p = scanToPrefill(
			resp({ creditorIban: 'CH9300762011623852957', creditorName: 'Fournisseur IBAN' }),
		);
		expect(p.creditorIban).toBe('CH9300762011623852957');
		expect(p.creditorQrIban).toBe('');
		expect(p.paymentReference).toBe('');
	});

	it('normalise les null en chaînes vides (montant ouvert, pas de référence)', () => {
		const p = scanToPrefill(resp({ creditorIban: 'CH93...', expectedPaymentAmount: null }));
		expect(p.expectedAmount).toBe('');
		expect(p.paymentReference).toBe('');
	});

	it('expose la devise (EUR ≠ CHF) pour lever l’ambiguïté du montant', () => {
		const p = scanToPrefill(resp({ creditorIban: 'CH93...', currency: 'EUR' }));
		expect(p.currency).toBe('EUR');
	});
});
