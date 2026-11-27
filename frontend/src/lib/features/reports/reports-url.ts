/**
 * Story 55-1a (D1, D5) — l'état des Rapports dans l'URL : deux fonctions pures, la lecture
 * (`parseReportUrl`) et l'écriture (`buildReportSearch`), et ce qui en dépend (la période d'un
 * onglet classique déduite de l'URL, l'exercice qui contient une date). Aucun état, aucun `goto` : la
 * page appelle ces fonctions, et seulement elle.
 *
 * **Ordre canonique** : `tab`, `fy`, `accountId`, `from`, `to`, `includeZero`, puis, sur les onglets
 * groupés (Story 55-1b), la présentation `view`, `level`, `subtotals`, `open`, `closed` ; forme de
 * `URLSearchParams.toString()` (la virgule des listes s'écrit `%2C`, la lecture accepte aussi la virgule brute). Pour le grand livre, il rend exactement la sortie historique de
 * `ledgerHref` (`tab`, `accountId`, `from`, `to`), et `build(parse(u)) === u` pour toute URL canonique.
 */

import { estDateStockable } from '$lib/shared/utils/date-filtre';
import type { FiscalYearResponse } from '$lib/features/fiscal-years/fiscal-years.types';
import { groupingFromParams } from './reports-grouping';
import type { GroupingParams } from './reports.types';

/** Les neuf onglets de la page des Rapports, dans l'ordre de l'écran. */
export const REPORT_TABS = [
	'balance-sheet',
	'income-statement',
	'trial-balance',
	'journals',
	'vat',
	'project-expenses',
	'project-return',
	'aged-receivables',
	'general-ledger',
] as const;

/** L'identifiant d'un onglet. */
export type TabId = (typeof REPORT_TABS)[number];

/** Les onglets « classiques » : une période dans un seul exercice (`fiscalYearId` exigé par l'API). */
export type ClassicTab = 'balance-sheet' | 'income-statement' | 'trial-balance' | 'journals' | 'vat';

const CLASSIC: readonly TabId[] = ['balance-sheet', 'income-statement', 'trial-balance', 'journals', 'vat'];

/** Les onglets qui ont une présentation groupée (Epic 46) : le bilan, le compte de résultat, la balance. */
const GROUPED: readonly TabId[] = ['balance-sheet', 'income-statement', 'trial-balance'];

/**
 * Story 55-1b (D4) — la présentation d'un rapport groupé dans l'URL : les paramètres de regroupement
 * envoyés à l'API (hiérarchie montrée), ou `'flat'` (hiérarchie masquée, aucun paramètre envoyé).
 */
export type Presentation = GroupingParams | 'flat';

/** Les paramètres de présentation, dans leur ordre canonique. */
export const PRESENTATION_PARAMS = ['view', 'level', 'subtotals', 'open', 'closed'] as const;

/** Vrai pour un onglet qui a une présentation groupée. */
export function isGroupedTab(tab: TabId): boolean {
	return GROUPED.includes(tab);
}

/** Vrai pour un identifiant d'onglet connu. */
export function isTabId(value: string | null | undefined): value is TabId {
	return value != null && (REPORT_TABS as readonly string[]).includes(value);
}

/** Vrai pour un onglet classique. */
export function isClassicTab(tab: TabId): tab is ClassicTab {
	return CLASSIC.includes(tab);
}

/** Ce que les règles lisent d'un exercice. */
export type ExerciceBornes = Pick<FiscalYearResponse, 'id' | 'name' | 'startDate' | 'endDate'>;

/** L'exercice qui contient une date (`startDate ≤ date ≤ endDate`), ou `null`. */
export function exerciceDe(
	date: string,
	fiscalYears: readonly ExerciceBornes[],
): ExerciceBornes | null {
	return fiscalYears.find((fy) => fy.startDate <= date && date <= fy.endDate) ?? null;
}

/** Les paramètres d'une URL des Rapports, chacun jugé seul : invalide ou étranger à l'onglet → absent. */
export interface ReportUrlParams {
	/** L'onglet, `null` s'il manque ou est inconnu. */
	tab: TabId | null;
	/** L'exercice (onglets classiques), un identifiant connu ; `null` sinon. */
	fy: number | null;
	/** Les dates (onglets classiques et grand livre), `estDateStockable` ; `null` sinon. */
	from: string | null;
	to: string | null;
	/** Le compte du grand livre ; `null` sinon. */
	accountId: number | null;
	/** « Comptes sans mouvement » du grand livre (`includeZero=1`). */
	includeZero: boolean;
	/**
	 * Story 55-1b — la présentation (onglets groupés) : `null` si l'adresse n'en porte aucun champ
	 * valide ({@link groupingFromParams}, `anyValid` faux) ; `'flat'` pour `view=flat`, les autres champs
	 * étant alors ignorés ; sinon la vue de l'adresse, les champs absents à leur valeur par défaut.
	 */
	presentation: Presentation | null;
}

const ENTIER = /^[1-9][0-9]*$/;

/** Un entier strictement positif sûr, ou `null`. */
function entier(raw: string | null): number | null {
	if (raw === null || !ENTIER.test(raw)) return null;
	const n = Number(raw);
	return Number.isSafeInteger(n) ? n : null;
}

/**
 * Lit une URL des Rapports (D5) : chaque paramètre est jugé seul, une valeur invalide ou qui ne sert
 * pas l'onglet est traitée comme absente. `fy` n'est valide que s'il désigne un exercice connu.
 */
export function parseReportUrl(
	search: URLSearchParams,
	fiscalYearIds: readonly number[],
): ReportUrlParams {
	const raw = search.get('tab');
	const tab = isTabId(raw) ? raw : null;
	const datee = tab !== null && (isClassicTab(tab) || tab === 'general-ledger');
	const ledger = tab === 'general-ledger';
	const fy = tab !== null && isClassicTab(tab) ? entier(search.get('fy')) : null;
	const from = search.get('from');
	const to = search.get('to');
	return {
		tab,
		fy: fy !== null && fiscalYearIds.includes(fy) ? fy : null,
		from: datee && estDateStockable(from) ? from : null,
		to: datee && estDateStockable(to) ? to : null,
		accountId: ledger ? entier(search.get('accountId')) : null,
		includeZero: ledger && search.get('includeZero') === '1',
		presentation: tab !== null && isGroupedTab(tab) ? presentationOf(search) : null,
	};
}

/** La présentation d'une adresse (règles de `reports-grouping.ts`, aucune copie ici). */
function presentationOf(search: URLSearchParams): Presentation | null {
	const { view, anyValid } = groupingFromParams(search);
	if (!anyValid) return null;
	if (!view.shown) return 'flat';
	return {
		detailLevel: view.detailLevel,
		subtotalPosition: view.subtotalPosition,
		open: view.open,
		closed: view.closed,
	};
}

/** Une liste d'identifiants écrite : croissante, sans doublon. */
function idList(ids: readonly number[]): string {
	return [...new Set(ids)].sort((a, b) => a - b).join(',');
}

/**
 * Écrit l'état d'un onglet en recherche canonique, sans le `?` : seuls les paramètres de l'onglet,
 * dans l'ordre canonique ; `includeZero` faux et les valeurs absentes ne s'écrivent pas.
 */
export function buildReportSearch(p: Partial<ReportUrlParams> & { tab: TabId }): string {
	const qs = new URLSearchParams({ tab: p.tab });
	const classic = isClassicTab(p.tab);
	const ledger = p.tab === 'general-ledger';
	if (classic && p.fy != null) qs.set('fy', String(p.fy));
	if (ledger && p.accountId != null) qs.set('accountId', String(p.accountId));
	if ((classic || ledger) && p.from) qs.set('from', p.from);
	if ((classic || ledger) && p.to) qs.set('to', p.to);
	if (ledger && p.includeZero) qs.set('includeZero', '1');
	if (isGroupedTab(p.tab) && p.presentation != null) {
		if (p.presentation === 'flat') {
			qs.set('view', 'flat');
		} else {
			qs.set('view', 'grouped');
			qs.set('level', String(p.presentation.detailLevel));
			qs.set('subtotals', p.presentation.subtotalPosition);
			if (p.presentation.open.length > 0) qs.set('open', idList(p.presentation.open));
			if (p.presentation.closed.length > 0) qs.set('closed', idList(p.presentation.closed));
		}
	}
	return qs.toString();
}

/**
 * Story 55-1b — la recherche sans sa présentation : deux clés qui désignent **le même rapport** (onglet,
 * exercice, période, compte) ont la même partie « rapport », quelle que soit leur présentation.
 */
export function reportPart(search: string): string {
	const qs = new URLSearchParams(search);
	for (const name of PRESENTATION_PARAMS) qs.delete(name);
	return qs.toString();
}

/** L'adresse d'une vue des Rapports : `/reports?<recherche canonique>`. */
export function reportHref(p: Partial<ReportUrlParams> & { tab: TabId }): string {
	return `/reports?${buildReportSearch(p)}`;
}

/** La période d'un onglet classique lue dans l'URL, et l'exercice qu'elle désigne. */
export interface ClassicPeriod {
	from: string;
	to: string;
	/** L'exercice qui contient les deux dates, `null` si elles tombent dans deux exercices ou hors exercice. */
	fy: number | null;
}

/**
 * La période d'un onglet classique d'après l'URL (AC-15) : les dates décident ; une date absente prend
 * la borne de `fy` ; sans date ni `fy` valide, `null` (aucun champ n'est modifié). Un `fy` qui contredit
 * les dates est remplacé par l'exercice des dates.
 */
export function resolveClassicPeriod(
	p: ReportUrlParams,
	fiscalYears: readonly ExerciceBornes[],
): ClassicPeriod | null {
	const fyBornes = p.fy === null ? null : (fiscalYears.find((f) => f.id === p.fy) ?? null);
	const from = p.from ?? fyBornes?.startDate ?? null;
	const to = p.to ?? fyBornes?.endDate ?? null;
	if (from === null || to === null) return null;
	const debut = exerciceDe(from, fiscalYears);
	const fin = exerciceDe(to, fiscalYears);
	const fy = debut && fin && debut.id === fin.id && from <= to ? debut.id : null;
	return { from, to, fy };
}
