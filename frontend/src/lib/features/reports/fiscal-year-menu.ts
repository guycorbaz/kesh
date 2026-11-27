/**
 * Story 55-1a (D15) — les règles pures du sélecteur d'exercice des Rapports : le texte d'un exercice,
 * l'état du menu déduit des dates, et l'en-tête de la période d'un rapport. Aucun état, aucun texte
 * traduit : le composant (`ReportSelector`, `ReportPeriodHeading`) et la page ne font que les appeler.
 */

import { formatSwissDate } from './reports.api';
import { exerciceDe, type ExerciceBornes } from './reports-url';

export { exerciceDe, type ExerciceBornes };

const annee = (iso: string): string => iso.slice(0, 4);

/**
 * Le nom que Kesh donne à un exercice : réplique de `nom_par_defaut`
 * (`crates/kesh-core/src/accounting/exercice.rs`) — « Exercice AAAA » si début et fin tombent la même
 * année, « Exercice AAAA/AAAA » sinon. À garder identique à la fonction Rust.
 */
export function nomParDefaut(startDate: string, endDate: string): string {
	return `Exercice ${anneesDe(startDate, endDate)}`;
}

/** Les années d'un exercice : « 2025 », ou « 2025/2026 » pour un exercice décalé. */
export function anneesDe(startDate: string, endDate: string): string {
	return annee(startDate) === annee(endDate)
		? annee(startDate)
		: `${annee(startDate)}/${annee(endDate)}`;
}

/** Une date `AAAA-MM-JJ` en jour-mois suisse : « 01.03 ». */
export function formatJourMois(iso: string): string {
	return `${iso.slice(8, 10)}.${iso.slice(5, 7)}`;
}

/**
 * Le texte de base d'un exercice : ses années si son nom est celui que Kesh aurait donné, sinon le nom
 * saisi, tel quel.
 */
function texteDeBase(fy: ExerciceBornes): string {
	return fy.name === nomParDefaut(fy.startDate, fy.endDate)
		? anneesDe(fy.startDate, fy.endDate)
		: fy.name;
}

/** Les entrées dont le texte est porté par plus d'une. */
function doublons(textes: Map<number, string>): Set<number> {
	const compte = new Map<string, number>();
	for (const t of textes.values()) compte.set(t, (compte.get(t) ?? 0) + 1);
	return new Set([...textes].filter(([, t]) => (compte.get(t) ?? 0) > 1).map(([id]) => id));
}

/**
 * Le texte de chaque exercice dans le menu (AC-14) : années ou nom saisi ; les **seules** entrées en
 * doublon reçoivent leurs bornes jour-mois, « 2025 (01.03–30.06) », et, si cela ne les sépare pas
 * encore, leurs bornes complètes, « Compta (01.01.2024–31.12.2024) ».
 */
export function libellesExercices(fiscalYears: readonly ExerciceBornes[]): Map<number, string> {
	const base = new Map(fiscalYears.map((fy) => [fy.id, texteDeBase(fy)]));
	const enDoublon = doublons(base);
	const jourMois = new Map(base);
	for (const fy of fiscalYears) {
		if (enDoublon.has(fy.id)) {
			jourMois.set(
				fy.id,
				`${base.get(fy.id)} (${formatJourMois(fy.startDate)}–${formatJourMois(fy.endDate)})`,
			);
		}
	}
	const encore = doublons(jourMois);
	const final = new Map(jourMois);
	for (const fy of fiscalYears) {
		if (encore.has(fy.id)) {
			final.set(
				fy.id,
				`${base.get(fy.id)} (${formatSwissDate(fy.startDate)}–${formatSwissDate(fy.endDate)})`,
			);
		}
	}
	return final;
}

/**
 * L'état du menu d'exercice : un exercice (`whole` vrai si la période en est exactement les bornes),
 * « Plusieurs » (deux exercices), « Hors exercice » (une date hors de tout exercice).
 */
export type EtatMenu =
	| { kind: 'fy'; id: number; whole: boolean }
	| { kind: 'multiple' }
	| { kind: 'outside' };

const DATE = /^\d{4}-\d{2}-\d{2}$/;

/**
 * L'état du menu déduit des dates (AC-15). Début après fin, ou une date vide ou incomplète : le menu ne
 * change pas (`precedent`) — c'est une erreur de saisie, que la page signale.
 */
export function etatDuMenu(
	du: string,
	au: string,
	fiscalYears: readonly ExerciceBornes[],
	precedent: EtatMenu,
): EtatMenu {
	if (!DATE.test(du) || !DATE.test(au) || du > au) return precedent;
	const debut = exerciceDe(du, fiscalYears);
	const fin = exerciceDe(au, fiscalYears);
	if (!debut || !fin) return { kind: 'outside' };
	if (debut.id !== fin.id) return { kind: 'multiple' };
	return { kind: 'fy', id: debut.id, whole: du === debut.startDate && au === debut.endDate };
}

/** Ce que dit l'en-tête d'un rapport (AC-17) : la clé i18n et ses arguments, dates déjà formatées. */
export type EnTete =
	| { key: 'reports-nav-heading-fy'; args: { years: string; from: string; to: string } }
	| { key: 'reports-nav-heading-named'; args: { name: string; from: string; to: string } }
	| { key: 'reports-nav-heading-part'; args: { fiscalYear: string; from: string; to: string } }
	| { key: 'reports-nav-heading-several'; args: { fiscalYears: string; from: string; to: string } }
	| { key: 'reports-nav-heading-outside'; args: { from: string; to: string } };

/**
 * L'en-tête de la période **réelle** d'un rapport (celle de la réponse, non des champs) :
 * exercice entier, partie d'un exercice, plusieurs exercices, hors exercice. `listFormat` assemble la
 * liste des exercices (`Intl.ListFormat` de la locale) : aucune phrase n'est concaténée.
 */
export function enTetePeriode(
	du: string,
	au: string,
	fiscalYears: readonly ExerciceBornes[],
	listFormat: (items: string[]) => string,
): EnTete {
	const from = formatSwissDate(du);
	const to = formatSwissDate(au);
	const libelles = libellesExercices(fiscalYears);
	const debut = exerciceDe(du, fiscalYears);
	const fin = exerciceDe(au, fiscalYears);
	if (!debut || !fin) return { key: 'reports-nav-heading-outside', args: { from, to } };
	if (debut.id !== fin.id) {
		const touches = fiscalYears
			.filter((fy) => fy.endDate >= du && fy.startDate <= au)
			.slice()
			.sort((a, b) => a.startDate.localeCompare(b.startDate))
			.map((fy) => libelles.get(fy.id) ?? fy.name);
		return { key: 'reports-nav-heading-several', args: { fiscalYears: listFormat(touches), from, to } };
	}
	const parDefaut = debut.name === nomParDefaut(debut.startDate, debut.endDate);
	const texte = libelles.get(debut.id) ?? debut.name;
	if (du === debut.startDate && au === debut.endDate) {
		return parDefaut
			? { key: 'reports-nav-heading-fy', args: { years: texte, from, to } }
			: { key: 'reports-nav-heading-named', args: { name: texte, from, to } };
	}
	return {
		key: 'reports-nav-heading-part',
		args: { fiscalYear: parDefaut ? texte : `« ${texte} »`, from, to },
	};
}
