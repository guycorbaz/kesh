/**
 * Les dates proposées pour le premier exercice dans l'assistant de démarrage —
 * Story 37-1 (AC-11).
 *
 * Miroir de `kesh_core::accounting::exercice::fin_par_defaut` (serveur) : la fin
 * proposée est **la veille du même jour un an plus tard** ; le « même jour un an plus
 * tard » d'un 29 février est le 1er mars suivant, d'où une fin au 28 février. Les mêmes
 * cas limites sont testés des deux côtés (`premier-exercice.test.ts`, test d'API
 * `fiscal_years_e2e.rs`).
 *
 * Les dates sont des chaînes `AAAA-MM-JJ`, formées par les champs **locaux** d'une
 * date (comme `todayLocal`), jamais par sa forme ISO, qui est en UTC.
 */

const ISO_DATE_RE = /^(\d{4})-(\d{2})-(\d{2})$/;

function formatLocal(d: Date): string {
	const y = String(d.getFullYear()).padStart(4, '0');
	const m = String(d.getMonth() + 1).padStart(2, '0');
	const j = String(d.getDate()).padStart(2, '0');
	return `${y}-${m}-${j}`;
}

/** Une date `AAAA-MM-JJ` qui existe au calendrier (le 30 février ne passe pas). */
export function isIsoDate(value: string): boolean {
	const p = ISO_DATE_RE.exec(value);
	if (!p) return false;
	const [y, m, j] = [Number(p[1]), Number(p[2]), Number(p[3])];
	const d = new Date(2000, 0, 1);
	d.setFullYear(y, m - 1, j);
	return d.getFullYear() === y && d.getMonth() === m - 1 && d.getDate() === j;
}

/** Le début proposé : le 1er janvier de l'année de `today` (`AAAA-MM-JJ`, `todayLocal`). */
export function defaultFiscalYearStart(today: string): string {
	return `${today.slice(0, 4)}-01-01`;
}

/**
 * La fin proposée pour un exercice qui commence le `start` : la veille du même jour un
 * an plus tard (01.01 → 31.12, 01.04.2026 → 31.03.2027, 29.02.2028 → 28.02.2029,
 * 01.03.2027 → 29.02.2028). `null` si `start` n'est pas une date.
 */
export function defaultFiscalYearEnd(start: string): string | null {
	if (!isIsoDate(start)) return null;
	const [y, m, j] = start.split('-').map(Number);
	// `setFullYear` plutôt que le constructeur, qui lit les années 0 à 99 comme 19xx.
	// Le même jour un an plus tard (le 29 février d'une année non bissextile déborde
	// au 1er mars), puis sa veille : en un seul appel, jour − 1.
	const d = new Date(2000, 0, 1);
	d.setFullYear(y + 1, m - 1, j - 1);
	return formatLocal(d);
}

/** Ce qui empêche de créer l'exercice à ces dates. */
export type FiscalYearRangeProblem = 'invalid-date' | 'end-not-after-start';

/**
 * Le motif qui empêche de créer l'exercice, `null` si les dates conviennent :
 * `invalid-date` si l'une n'est pas une date ou sort des années que la base sait
 * tenir (1000 à 9999, comme `annee_stockable` au serveur) ; `end-not-after-start`
 * si la fin ne suit pas strictement le début (le refus `error-fiscal-year-dates-invalid`
 * du serveur). Aucune borne de durée.
 */
export function fiscalYearRangeProblem(start: string, end: string): FiscalYearRangeProblem | null {
	const stockable = (v: string) => isIsoDate(v) && v >= '1000-01-01' && v <= '9999-12-31';
	if (!stockable(start) || !stockable(end)) return 'invalid-date';
	return end > start ? null : 'end-not-after-start';
}
