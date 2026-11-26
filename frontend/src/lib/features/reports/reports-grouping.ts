/**
 * Story 46-2 — la vue groupée du bilan et du compte de résultat (et, depuis la
 * Story 46-6b, de la balance, avec sa propre mémoire) : son état
 * (hiérarchie montrée ou masquée, niveau de détail, place des sous-totaux,
 * groupes ouverts ou fermés à la main), ses transitions, et sa mémoire par
 * navigateur.
 *
 * ⚠️ **Aucune règle d'ouverture n'est calculée ici** (C-46-2-1) : l'écran
 * redemande le rapport à l'API à chaque changement, et c'est la 46-1 qui dit
 * quels groupes sont ouverts. Ce module ne fait que tenir les choix de
 * l'utilisateur et les traduire en paramètres de requête. Seule exception, qui ne
 * décide d'aucun affichage : un clic qui rend à un groupe l'état du niveau le
 * retire des listes (`openByLevel`, Story 46-7).
 *
 * Mémoire (AC-5, AC-10) : `localStorage`, **une clé par société et par
 * rapport** (par navigateur, non par utilisateur, comme `kesh-mode`), toute
 * lecture et écriture sous `try`/`catch` ; stockage vide, illisible ou refusé :
 * la vue par défaut, sans erreur. Une valeur relue invalide est ignorée champ par
 * champ, jamais transmise à l'API.
 */

import type {
	DetailLevel,
	GroupingEcho,
	GroupingParams,
	SubtotalPosition,
} from './reports.types';

/** Les rapports qui ont une vue groupée : le bilan, le compte de résultat et la balance (46-6b). */
export type GroupedReport = 'balance-sheet' | 'income-statement' | 'trial-balance';

/** Ce que l'utilisateur a choisi pour un rapport. */
export interface GroupingView {
	/** Hiérarchie montrée (écran groupé) ou masquée (écran à plat d'avant l'Epic 46). */
	shown: boolean;
	detailLevel: DetailLevel;
	subtotalPosition: SubtotalPosition;
	/** Groupes ouverts à la main. */
	open: number[];
	/** Groupes fermés à la main. */
	closed: number[];
}

/** Niveau de détail le plus profond que l'API accepte (46-1). */
export const MAX_DETAIL_LEVEL = 20;

/** Longueur maximale d'une liste `open` ou `closed` (46-1 : `400` au-delà). */
export const MAX_MANUAL_GROUPS = 500;

/** Préfixe des clés de mémoire ; la clé complète porte la société et le rapport. */
export const GROUPING_STORAGE_PREFIX = 'kesh-report-grouping';

/**
 * La vue de la première visite (AC-1, Q10) : hiérarchie montrée, tous les
 * groupes ouverts, sous-totaux sur la ligne du titre.
 */
export function defaultGroupingView(): GroupingView {
	return { shown: true, detailLevel: 'all', subtotalPosition: 'header', open: [], closed: [] };
}

/** Clé de mémoire d'un rapport d'une société. */
export function groupingStorageKey(companyId: number, report: GroupedReport): string {
	return `${GROUPING_STORAGE_PREFIX}:${companyId}:${report}`;
}

/** Un niveau de détail valide : un entier de 1 à 20, ou `all`. */
export function isDetailLevel(v: unknown): v is DetailLevel {
	return (
		v === 'all' ||
		(typeof v === 'number' && Number.isInteger(v) && v >= 1 && v <= MAX_DETAIL_LEVEL)
	);
}

/** Une liste d'identifiants valide : des entiers positifs, au plus 500, sans doublon. */
function readIdList(v: unknown): number[] | null {
	if (!Array.isArray(v) || v.length > MAX_MANUAL_GROUPS) return null;
	if (!v.every((e) => typeof e === 'number' && Number.isInteger(e) && e > 0)) return null;
	return [...new Set(v as number[])];
}

/**
 * Relit une vue mémorisée, champ par champ : un champ invalide prend sa valeur
 * par défaut (AC-5). Deux listes qui partagent un identifiant seraient refusées
 * en `400` par l'API : elles sont ignorées toutes les deux.
 */
export function sanitizeGroupingView(raw: unknown): GroupingView {
	const view = defaultGroupingView();
	if (typeof raw !== 'object' || raw === null) return view;
	const r = raw as Record<string, unknown>;
	if (typeof r.shown === 'boolean') view.shown = r.shown;
	if (isDetailLevel(r.detailLevel)) view.detailLevel = r.detailLevel;
	if (r.subtotalPosition === 'header' || r.subtotalPosition === 'footer') {
		view.subtotalPosition = r.subtotalPosition;
	}
	const open = readIdList(r.open);
	const closed = readIdList(r.closed);
	if (open && closed && !open.some((id) => closed.includes(id))) {
		view.open = open;
		view.closed = closed;
	} else if (open && !closed) {
		view.open = open;
	} else if (closed && !open) {
		view.closed = closed;
	}
	return view;
}

/**
 * La vue mémorisée d'un rapport, ou la vue par défaut (société inconnue,
 * stockage vide, illisible ou refusé).
 */
export function readGroupingView(companyId: number | null, report: GroupedReport): GroupingView {
	if (companyId === null) return defaultGroupingView();
	try {
		if (typeof localStorage === 'undefined') return defaultGroupingView();
		const raw = localStorage.getItem(groupingStorageKey(companyId, report));
		if (raw === null) return defaultGroupingView();
		return sanitizeGroupingView(JSON.parse(raw));
	} catch {
		return defaultGroupingView();
	}
}

/** Mémorise la vue d'un rapport ; sans effet si le stockage est refusé. */
export function writeGroupingView(
	companyId: number | null,
	report: GroupedReport,
	view: GroupingView,
): void {
	if (companyId === null) return;
	try {
		if (typeof localStorage !== 'undefined') {
			localStorage.setItem(groupingStorageKey(companyId, report), JSON.stringify(view));
		}
	} catch {
		// Navigation privée ou quota : la vue n'est pas retenue, l'écran reste juste.
	}
}

/**
 * Les paramètres de requête de la vue : aucun quand la hiérarchie est masquée
 * (l'écran à plat d'aujourd'hui, C-46-2-2).
 */
export function groupingParamsOf(view: GroupingView): GroupingParams | undefined {
	if (!view.shown) return undefined;
	return {
		detailLevel: view.detailLevel,
		subtotalPosition: view.subtotalPosition,
		open: [...view.open],
		closed: [...view.closed],
	};
}

/** Deux listes d'identifiants qui désignent les mêmes groupes, dans n'importe quel ordre. */
function sameIds(a: number[], b: number[]): boolean {
	if (a.length !== b.length) return false;
	const set = new Set(a);
	return b.every((id) => set.has(id));
}

/**
 * Deux jeux de paramètres qui demandent le même rapport (revue P1 M2) : tous deux
 * absents (hiérarchie masquée), ou mêmes niveau, place des sous-totaux et listes.
 */
export function sameGroupingParams(
	a: GroupingParams | undefined,
	b: GroupingParams | undefined,
): boolean {
	if (a === undefined || b === undefined) return a === b;
	return (
		a.detailLevel === b.detailLevel &&
		a.subtotalPosition === b.subtotalPosition &&
		sameIds(a.open, b.open) &&
		sameIds(a.closed, b.closed)
	);
}

/**
 * L'état qu'un groupe de profondeur `depth` reçoit du seul niveau, sans clic :
 * la règle d'ouverture du contrat de la 46-1 (`est_ouvert`, listes vides) —
 * ouvert si le niveau est « Tous les comptes » ou si `depth < detailLevel`.
 * Elle ne sert qu'à garder les listes manuelles minimales ({@link toggleGroup}) :
 * l'état affiché reste celui que l'API rend (C-46-2-1).
 */
function openByLevel(detailLevel: DetailLevel, depth: number): boolean {
	return detailLevel === 'all' || depth < detailLevel;
}

/**
 * Un clic sur l'accordéon d'un groupe (AC-3) : il prime sur le niveau. Ouvrir un
 * groupe fermé par le niveau l'ajoute à `open` ; fermer un groupe ouvert par le
 * niveau l'ajoute à `closed` ; le groupe quitte toujours l'autre liste. Un clic
 * qui **rend au groupe l'état du niveau** (le rouvrir après l'avoir fermé à la
 * main, ou l'inverse) le retire des deux listes au lieu de l'y inscrire : sans
 * quoi l'écran, qui montre l'état du niveau, garderait une liste non vide, et le
 * PDF exporté dirait « détail ajusté à la main » (Story 46-7, relevé de la revue
 * d'intégration de l'Epic 46).
 *
 * @param isOpen — l'état du groupe tel que l'API l'a rendu (`open` de sa ligne).
 * @param depth — la profondeur de sa ligne (`depth`), qui fixe l'état du niveau.
 */
export function toggleGroup(
	view: GroupingView,
	groupId: number,
	isOpen: boolean,
	depth: number,
): GroupingView {
	const open = view.open.filter((id) => id !== groupId);
	const closed = view.closed.filter((id) => id !== groupId);
	const wanted = !isOpen;
	if (wanted !== openByLevel(view.detailLevel, depth)) {
		if (wanted) open.push(groupId);
		else closed.push(groupId);
	}
	return { ...view, open, closed };
}

/** Changer de niveau vide les deux listes manuelles (AC-3). */
export function withDetailLevel(view: GroupingView, detailLevel: DetailLevel): GroupingView {
	return { ...view, detailLevel, open: [], closed: [] };
}

/** « Tout ouvrir » : niveau « Tous les comptes », listes vidées (C-46-2-3). */
export function openAll(view: GroupingView): GroupingView {
	return withDetailLevel(view, 'all');
}

/** « Tout fermer » : niveau 1, listes vidées (C-46-2-3). */
export function closeAll(view: GroupingView): GroupingView {
	return withDetailLevel(view, 1);
}

/**
 * Reprend les listes de l'écho de l'API (AC-10) : les groupes disparus du plan,
 * que l'API a ignorés, quittent la mémoire au prochain enregistrement.
 */
export function withEcho(view: GroupingView, echo: GroupingEcho): GroupingView {
	return { ...view, open: [...echo.open], closed: [...echo.closed] };
}

/**
 * Les niveaux numérotés que propose le sélecteur (AC-2) : 1 à `maxDepth − 1`,
 * bornés à 20 — au niveau `maxDepth`, tous les groupes sont ouverts, c'est
 * « Tous les comptes ». Un niveau mémorisé plus profond que le plan reste
 * proposé, pour que le sélecteur montre ce qui a été demandé (l'API le traite
 * comme `all`).
 */
export function detailLevelOptions(maxDepth: number, current: DetailLevel): number[] {
	const top = Math.min(Math.max(maxDepth - 1, 0), MAX_DETAIL_LEVEL);
	const levels = Array.from({ length: top }, (_, i) => i + 1);
	if (typeof current === 'number' && !levels.includes(current)) levels.push(current);
	return levels;
}
