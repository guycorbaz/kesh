// Story 9-1 — Page rapports comptables.
// Pas de restriction de rôle : Admin + Comptable + Consultation peuvent lire.

import { listFiscalYears } from '$lib/features/fiscal-years/fiscal-years.api';
import type { FiscalYearResponse } from '$lib/features/fiscal-years/fiscal-years.types';

export const ssr = false;

export async function load(): Promise<{ fiscalYears: FiscalYearResponse[] }> {
	try {
		const fiscalYears = await listFiscalYears();
		return { fiscalYears };
	} catch {
		return { fiscalYears: [] };
	}
}
