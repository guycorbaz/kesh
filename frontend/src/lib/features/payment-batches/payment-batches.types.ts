/**
 * Types DTO des lots de paiement pain.001 — Story 12.3 (#191).
 * Miroir camelCase des réponses backend (kesh-api routes/payment_batches.rs).
 */

export type PaymentBatchStatus = 'generated' | 'confirmed' | 'cancelled';

export interface PaymentBatchItemResponse {
	supplierInvoiceId: number;
	position: number;
	endToEndId: string;
	amount: string;
}

export interface PaymentBatchResponse {
	id: number;
	bankAccountId: number;
	status: PaymentBatchStatus;
	requestedExecutionDate: string;
	totalAmount: string;
	msgId: string;
	confirmedAt: string | null;
	version: number;
	createdAt: string;
	items: PaymentBatchItemResponse[];
}

export interface PaymentBatchListItemResponse {
	id: number;
	bankAccountId: number;
	status: PaymentBatchStatus;
	requestedExecutionDate: string;
	totalAmount: string;
	createdAt: string;
}

export interface PaymentBatchFailedItemResponse {
	supplierInvoiceId: number;
	errorCode: string;
	details: unknown | null;
}

export interface CreatePaymentBatchRequest {
	bankAccountId: number;
	requestedExecutionDate: string;
	supplierInvoiceIds: number[];
}

export interface CreatePaymentBatchResponse {
	batch: PaymentBatchResponse | null;
	failed: PaymentBatchFailedItemResponse[];
}

export interface ConfirmPaymentBatchRequest {
	paymentDate: string;
}

export interface ListPaymentBatchesQuery {
	limit?: number;
	offset?: number;
}

export interface ListResponse<T> {
	items: T[];
	total: number;
	offset: number;
	limit: number;
}
