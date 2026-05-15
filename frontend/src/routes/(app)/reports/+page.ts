// Story 9-1 + 9-2a — Page rapports comptables.
// Pas de restriction de rôle : Admin + Comptable + Consultation peuvent lire.
// P21 — Pass 1 code review : distinguer auth/network errors d'une absence légitime de fy.
// Story 9-2a T7.3 — charge `companyName` en parallèle via `fetchCompanyCurrent`
// pour construire les filenames d'export (AC #22).

import { listFiscalYears } from '$lib/features/fiscal-years/fiscal-years.api';
import type { FiscalYearResponse } from '$lib/features/fiscal-years/fiscal-years.types';
import { fetchCompanyCurrent } from '$lib/features/settings/settings.api';
import { isApiError } from '$lib/shared/utils/api-client';

export const ssr = false;

export async function load(): Promise<{
	fiscalYears: FiscalYearResponse[];
	companyName: string;
}> {
	// Chargement en parallèle (cf. ECH3-L1 — perf marginale acceptée).
	const [fiscalYears, companyName] = await Promise.all([
		loadFiscalYears(),
		loadCompanyName(),
	]);
	return { fiscalYears, companyName };
}

async function loadFiscalYears(): Promise<FiscalYearResponse[]> {
	try {
		return await listFiscalYears();
	} catch (err) {
		// Pass 1 code-review M9 (ECH4-M1) : aligné sur `loadCompanyName` —
		// 401 re-throw (auth guard `+layout.ts` gère redirect login). L'ancien
		// pattern « return [] » swallowait l'erreur et masquait la cause si
		// le redirect échouait (e.g. network down). Re-throw 401 propage
		// l'erreur au boundary `+error.svelte` qui affiche un état cohérent.
		if (isApiError(err) && err.status === 401) {
			throw err;
		}
		// Pour toute autre erreur (500, network, etc.), propager pour qu'elle remonte
		// au boundary +error.svelte — fiscalYears est critique pour la fonctionnalité.
		throw err;
	}
}

/**
 * Charge le nom de la company courante pour construire les filenames d'export.
 *
 * Gestion erreurs (Pass 2 ECH2-H2) :
 * - `401` → re-throw (auth guard `+layout.ts` redirect login).
 * - `403`, `500`, network, autres → fallback `'company'` (filename dégradé mais
 *   page utilisable — cosmétique pour les exports, cohérent L9/L12).
 */
async function loadCompanyName(): Promise<string> {
	try {
		const data = await fetchCompanyCurrent();
		return data.company.name;
	} catch (err) {
		if (isApiError(err) && err.status === 401) {
			throw err;
		}
		// Fallback gracieux sur erreur backend transitoire.
		console.warn('[reports/+page.ts] companyName fetch failed, using fallback:', err);
		return 'company';
	}
}
