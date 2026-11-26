/**
 * Story 39-11d2 (AC-2) — lecture **commune** des règles d'affectation pour
 * les pages des prestataires (PayPal, Wise) : la revue ne sert que `ruleId`,
 * l'écran en montre le libellé. Sortie de la page PayPal (38-11d, revue P1
 * L9) pour servir aux deux pages ; hors de `rules.api.ts`, que les tests des
 * pages simulent en entier.
 */
import { listRules } from '$lib/features/reconciliation/rules/rules.api';

/** Taille d'une page lue (le plafond du serveur) et nombre de pages lues au plus. */
const PER_PAGE = 200;
const MAX_PAGES = 50;

/**
 * Toutes les règles, page par page (le serveur borne une page à
 * {@link PER_PAGE}) : leur identifiant et leur libellé. Un refus du serveur se
 * propage (l'appelant lit au mieux).
 */
export async function listAllRules(): Promise<{ id: number; label: string }[]> {
	const out: { id: number; label: string }[] = [];
	for (let page = 1; page <= MAX_PAGES; page++) {
		const res = await listRules(page, PER_PAGE);
		out.push(...res.items);
		if (res.items.length === 0 || out.length >= res.total) break;
	}
	return out;
}

/** Les libellés des règles par identifiant, lus au mieux : une carte vide si la lecture échoue. */
export async function ruleLabelsById(): Promise<Map<number, string>> {
	try {
		return new Map((await listAllRules()).map((r) => [r.id, r.label]));
	} catch {
		return new Map();
	}
}
