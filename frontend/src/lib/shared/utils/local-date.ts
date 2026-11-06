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
