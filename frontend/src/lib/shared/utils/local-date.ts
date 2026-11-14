/**
 * La date du jour **du poste**, `AAAA-MM-JJ` — Story v0174-correctifs-3 (#889,
 * C-v0174c-13). Déplacé de `lib/features/open-items/open-items.ts`, qui le réexporte.
 *
 * Elle se lit sur les champs **locaux** de l'instant (`getFullYear`, `getMonth`,
 * `getDate`), jamais sur sa forme ISO, qui est en UTC : entre minuit et 1 h (2 h en été),
 * celle-ci rend encore la veille à Zurich. Pour un poste réglé sur la Suisse, la date du
 * poste est celle de Zurich.
 *
 * Tout écran qui propose « aujourd'hui » l'importe d'ici ; la garde
 * `local-date-guard.test.ts` tient l'inventaire des sites clos.
 */
export function todayLocal(now: Date = new Date()): string {
	const y = String(now.getFullYear()).padStart(4, '0');
	const m = String(now.getMonth() + 1).padStart(2, '0');
	const d = String(now.getDate()).padStart(2, '0');
	return `${y}-${m}-${d}`;
}

/** Les champs d'un instant à Zurich, en chiffres (forme indépendante de la langue du poste). */
const JOUR_DE_ZURICH = new Intl.DateTimeFormat('en-CA', {
	timeZone: 'Europe/Zurich',
	year: 'numeric',
	month: '2-digit',
	day: '2-digit',
});

/**
 * La date du jour **à Zurich**, `AAAA-MM-JJ` — Story 35-5a (#907, revue P1 E1).
 *
 * C'est le jour que juge le serveur (`kesh_core::horloge::jour_suisse`) pour ce qu'il date du
 * jour ou borne au jour (le verrou de période). Un écran qui propose une borne **que le
 * serveur jugera** la prend d'ici, non de `todayLocal` : sur un poste réglé hors de Suisse,
 * le jour du poste peut différer, et la borne proposée serait refusée. Pour une date que
 * l'utilisateur saisit pour lui (une date de facture), `todayLocal` reste la règle.
 */
export function todayInZurich(now: Date = new Date()): string {
	const champs: Record<string, string> = {};
	for (const p of JOUR_DE_ZURICH.formatToParts(now)) champs[p.type] = p.value;
	return `${champs.year}-${champs.month}-${champs.day}`;
}

/** La veille d'une date pure `AAAA-MM-JJ`, calculée sur le calendrier, sans fuseau. */
export function dayBefore(iso: string): string {
	const [y, m, d] = iso.split('-').map(Number);
	const veille = new Date(Date.UTC(y, m - 1, d - 1));
	const yy = String(veille.getUTCFullYear()).padStart(4, '0');
	const mm = String(veille.getUTCMonth() + 1).padStart(2, '0');
	const dd = String(veille.getUTCDate()).padStart(2, '0');
	return `${yy}-${mm}-${dd}`;
}
