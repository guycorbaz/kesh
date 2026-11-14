/**
 * Story 35-5c (#929, D3) — l'arithmétique de jours des specs E2E.
 *
 * Le jour d'une spec se prend de `todayInZurich()` (`api-fixtures.ts`), seule lecture de
 * l'horloge des specs ; un jour décalé se calcule ici, **depuis le jour reçu**, sans relire
 * l'horloge. Le calcul se fait sur les composants UTC d'une date pure (minuit UTC du jour
 * donné) : aucun fuseau, aucun passage d'heure d'été ne peut le décaler. Nommé autrement que
 * l'`addDaysIso` de `src/` (aucun homonyme) : les specs n'importent pas `src/`.
 */

const JOUR_ISO = /^(\d{4})-(\d{2})-(\d{2})$/;

const deux = (n: number): string => String(n).padStart(2, '0');

/** Une année que `Date.UTC` lit telle quelle (de 0 à 99, il la lirait 1900 + année). */
const anneeAdmise = (a: number): boolean => a >= 1000 && a <= 9999;

const formater = (d: Date): string =>
	`${d.getUTCFullYear()}-${deux(d.getUTCMonth() + 1)}-${deux(d.getUTCDate())}`;

/**
 * Le jour `iso` (`AAAA-MM-JJ`) décalé de `n` jours (négatif : en arrière), au même format.
 *
 * Revue P1 de la 35-5c (A5, B4, E3) : refuse aussi une année hors de 1000 à 9999, à l'entrée
 * comme au résultat ; un jour impossible (`2026-02-30`, `2026-13-01`), qui ne se relit pas à
 * l'identique ; un `n` qui n'est pas un entier fini (`NaN`, `0.5`, `Infinity`).
 *
 * @throws si `iso` n'est pas un jour réel `AAAA-MM-JJ` des années 1000 à 9999, ou si `n` n'est
 * pas un entier, ou si le résultat sort de ces années.
 */
export function ajouterJoursIso(iso: string, n: number): string {
	const m = JOUR_ISO.exec(iso);
	if (!m) throw new Error(`jour attendu au format AAAA-MM-JJ : « ${iso} »`);
	if (!Number.isInteger(n)) throw new Error(`décalage attendu entier : « ${n} »`);
	const [annee, mois, jour] = [Number(m[1]), Number(m[2]), Number(m[3])];
	if (!anneeAdmise(annee)) throw new Error(`année hors de 1000 à 9999 : « ${iso} »`);
	if (formater(new Date(Date.UTC(annee, mois - 1, jour))) !== iso) {
		throw new Error(`jour impossible : « ${iso} »`);
	}
	const d = new Date(Date.UTC(annee, mois - 1, jour + n));
	if (!anneeAdmise(d.getUTCFullYear())) {
		throw new Error(`résultat hors des années 1000 à 9999 : « ${iso} » + ${n}`);
	}
	return formater(d);
}
