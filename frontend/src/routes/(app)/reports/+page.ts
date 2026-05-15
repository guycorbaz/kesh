// Story 9-1 — Page rapports comptables.
// Pas de restriction de rôle : Admin + Comptable + Consultation peuvent lire.
// P21 — Pass 1 code review : distinguer auth/network errors d'une absence légitime de fy.

import { listFiscalYears } from '$lib/features/fiscal-years/fiscal-years.api';
import type { FiscalYearResponse } from '$lib/features/fiscal-years/fiscal-years.types';
import { isApiError } from '$lib/shared/utils/api-client';

export const ssr = false;

export async function load(): Promise<{ fiscalYears: FiscalYearResponse[] }> {
	try {
		const fiscalYears = await listFiscalYears();
		return { fiscalYears };
	} catch (err) {
		// 401 → api-client a déjà déclenché redirect login ; le catch ici évite un crash
		// transitoire de SvelteKit pendant le repaint pré-redirect.
		if (isApiError(err) && err.status === 401) {
			return { fiscalYears: [] };
		}
		// Pour toute autre erreur (500, network, etc.), propager pour qu'elle remonte
		// au boundary +error.svelte et soit visible à l'utilisateur — pas de masquage
		// silencieux qui ferait passer une panne backend pour "aucun fy disponible".
		throw err;
	}
}
