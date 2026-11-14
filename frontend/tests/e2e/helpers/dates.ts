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

/**
 * Le jour `iso` (`AAAA-MM-JJ`) décalé de `n` jours (négatif : en arrière), au même format.
 *
 * @throws si `iso` n'est pas de la forme `AAAA-MM-JJ`.
 */
export function ajouterJoursIso(iso: string, n: number): string {
	const m = JOUR_ISO.exec(iso);
	if (!m) throw new Error(`jour attendu au format AAAA-MM-JJ : « ${iso} »`);
	const d = new Date(Date.UTC(Number(m[1]), Number(m[2]) - 1, Number(m[3]) + n));
	return `${String(d.getUTCFullYear()).padStart(4, '0')}-${deux(d.getUTCMonth() + 1)}-${deux(d.getUTCDate())}`;
}
