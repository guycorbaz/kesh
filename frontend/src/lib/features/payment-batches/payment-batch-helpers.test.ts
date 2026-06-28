import { describe, expect, it } from 'vitest';
import {
	failedItemLabel,
	formatBatchTotal,
	paymentBatchStatusLabel,
} from './payment-batch-helpers';

describe('formatBatchTotal', () => {
	it('formate suisse', () => {
		expect(formatBatchTotal('300.0000')).toBe('300.00');
	});
	it("vide → ''", () => {
		expect(formatBatchTotal(null)).toBe('');
		expect(formatBatchTotal('x')).toBe('');
	});
});

describe('paymentBatchStatusLabel', () => {
	it('mappe les statuts', () => {
		expect(paymentBatchStatusLabel('generated')).toBe('Généré');
		expect(paymentBatchStatusLabel('confirmed')).toBe('Confirmé');
		expect(paymentBatchStatusLabel('cancelled')).toBe('Annulé');
	});
});

describe('failedItemLabel', () => {
	it('mappe les codes connus', () => {
		expect(failedItemLabel('NO_PAYMENT_COORDINATES')).toContain('coordonnées');
		expect(failedItemLabel('ALREADY_IN_GENERATED_BATCH')).toContain('lot');
		expect(failedItemLabel('UNKNOWN')).toBe('UNKNOWN');
	});
});
