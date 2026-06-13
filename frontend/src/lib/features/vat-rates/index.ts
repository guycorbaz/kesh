/**
 * Re-exports publics de la feature `vat-rates` (Story 7.2 — KF-003 ; CRUD 11-1).
 */

export type {
	VatRateResponse,
	NewVatRatePayload,
	UpdateVatRatePayload,
} from './vat-rates.types';
export { listVatRates, createVatRate, updateVatRate, deactivateVatRate } from './vat-rates.api';
export { getVatRates, resetVatRatesCache } from './vat-rates.store.svelte';
