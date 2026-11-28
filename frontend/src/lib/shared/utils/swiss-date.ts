/**
 * Le formateur de date suisse du frontend — le seul (`JJ.MM.AAAA`), et la seule lecture
 * « horodatage naïf = UTC ».
 *
 * Vit dans `lib/shared/` : il n'importe rien, si bien que la couche partagée (`settlement-cancel-blocked.ts`,
 * `local-date.ts`…) l'emploie sans remonter vers `features/`. `features/reports/reports.api.ts` le
 * ré-exporte, pour que les imports historiques (`$lib/features/reports/reports.api`) tiennent encore.
 * Revue P1 de la 63-1 (L1) : il se trouvait auparavant dans `reports.api.ts`, qui tire `big.js`, `api-client`
 * et `features/journal-entries/balance`.
 */

/** Une date pure `AAAA-MM-JJ`, rien d'autre. */
const PURE_DATE_RE = /^(\d{4})-(\d{2})-(\d{2})$/;
/**
 * Un horodatage : `AAAA-MM-JJ`, `T` ou une espace, `HH:mm` (secondes et fraction
 * facultatives), puis `Z`, un décalage `±HH:MM`, ou rien.
 */
const TIMESTAMP_RE = /^(\d{4}-\d{2}-\d{2})[T ](\d{2}:\d{2}(?::\d{2}(?:\.\d+)?)?)(Z|[+-]\d{2}:\d{2})?$/;

/** Le fuseau où Kesh montre ses instants : celui de la Suisse, quel que soit le poste (#1511). */
export const ZONE_SUISSE = 'Europe/Zurich';

/** Les composants d'un instant à l'heure de Zurich (forme indépendante de la langue). */
const ZURICH_PARTS = new Intl.DateTimeFormat('fr-CH', {
	timeZone: ZONE_SUISSE,
	year: 'numeric',
	month: '2-digit',
	day: '2-digit',
	hour: '2-digit',
	minute: '2-digit',
	hourCycle: 'h23',
});

/**
 * L'instant d'un horodatage du serveur — `null` si `iso` n'en est pas un. Sans `Z` ni
 * décalage, l'horodatage est lu comme **UTC** : c'est ce que rend le serveur
 * (`naive_utc()`, session MariaDB en `+00:00` ; v0172-correctifs, C-v0172b-10). La seule
 * lecture « naïf = UTC » du frontend : `zurichParts` et `formatInstant` l'appellent.
 */
export function instantDuServeur(iso: string): Date | null {
	const m = TIMESTAMP_RE.exec(iso);
	if (!m) return null;
	const d = new Date(`${m[1]}T${m[2]}${m[3] ?? 'Z'}`);
	return Number.isNaN(d.getTime()) ? null : d;
}

/** Les composants d'un instant à l'heure de Zurich — `null` si `iso` n'est pas un horodatage. */
function zurichParts(iso: string): Record<'day' | 'month' | 'year' | 'hour' | 'minute', string> | null {
	const instant = instantDuServeur(iso);
	if (!instant) return null;
	const out = { day: '', month: '', year: '', hour: '', minute: '' };
	for (const p of ZURICH_PARTS.formatToParts(instant)) {
		if (p.type in out) out[p.type as keyof typeof out] = p.value;
	}
	return out;
}

/**
 * Formate une date en forme suisse `JJ.MM.AAAA`.
 *
 * - une date pure `AAAA-MM-JJ` : ses composants, **sans fuseau** ;
 * - un horodatage (v0172-correctifs, #880, #883.6) : la **date à Zurich** de cet instant
 *   (sans décalage, lu comme UTC) — avant la 0.17.2, `2026-11-03T17:40:45` rendait
 *   `03T17:40:45.11.2026` ;
 * - toute autre entrée est rendue telle quelle.
 */
export function formatSwissDate(iso: string): string {
	const d = PURE_DATE_RE.exec(iso);
	if (d) return `${d[3]}.${d[2]}.${d[1]}`;
	const z = zurichParts(iso);
	return z ? `${z.day}.${z.month}.${z.year}` : iso;
}

/**
 * Formate un horodatage en `JJ.MM.AAAA HH:mm`, **à l'heure de Zurich** (sans décalage, lu
 * comme UTC) ; une date pure se rend `JJ.MM.AAAA`, toute autre entrée telle quelle
 * (v0172-correctifs, #880, C-v0172b-10).
 */
export function formatSwissDateTime(iso: string): string {
	const z = zurichParts(iso);
	return z ? `${z.day}.${z.month}.${z.year} ${z.hour}:${z.minute}` : formatSwissDate(iso);
}
