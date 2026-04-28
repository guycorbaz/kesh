/**
 * Re-exports publics de la feature `vat-rates` (Story 7.2 — KF-003 closure).
 */

export type { VatRateResponse } from './vat-rates.types';
export { listVatRates } from './vat-rates.api';
export { getVatRates, resetVatRatesCache } from './vat-rates.store.svelte';
