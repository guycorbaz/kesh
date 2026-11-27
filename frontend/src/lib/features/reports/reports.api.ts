// Story 9-1 — Client API rapports comptables (4 endpoints GET /api/v1/reports/*).

import Big from 'big.js';
import { apiClient } from '$lib/shared/utils/api-client';
import { i18nMsg } from '$lib/shared/utils/i18n.svelte';
import { todayLocal } from '$lib/shared/utils/local-date';
import { formatSwissAmount } from '$lib/features/journal-entries/balance';
import { reportHref } from './reports-url';
import type {
	AgedReceivablesDto,
	BalanceSheetDto,
	IncomeStatementDto,
	JournalReportDto,
	JournalReportQuery,
	ProjectExpensesDto,
	ProjectReportQuery,
	ProjectReportType,
	ProjectReturnDto,
	ReportQuery,
	ReportType,
	GeneralLedgerDto,
	GroupingParams,
	LedgerQuery,
	TrialBalanceDto,
	VatReportDto,
	EntryDocument,
} from './reports.types';

function buildQuery(params: Record<string, string | number | undefined>): string {
	const sp = new URLSearchParams();
	for (const [key, val] of Object.entries(params)) {
		if (val !== undefined && val !== null && val !== '') {
			sp.set(key, String(val));
		}
	}
	return sp.toString();
}

/**
 * La valeur de `detailLevel` qui demande un export **à plat** (sans colonnes
 * `Niveau;TypeLigne`) : sans paramètre, l'export de l'API rend la présentation
 * par groupes de l'écran.
 */
export const FLAT_DETAIL_LEVEL = 'flat';

/**
 * Story 46-2 — les paramètres de groupement d'une demande (46-1 § *Le contrat*) :
 * `detailLevel`, `subtotalPosition`, et `open` / `closed` en identifiants séparés
 * par des virgules, **omis quand la liste est vide** (une liste vide serait
 * refusée en `400` : « » n'est pas un identifiant). Sans `grouping`, aucun
 * paramètre : la réponse JSON est celle d'avant l'Epic 46, à plat. L'export,
 * lui, est groupé sans paramètre : {@link getReportExportUrl} demande le rendu
 * à plat par `detailLevel=flat`.
 */
export function groupingQueryParams(
	grouping: GroupingParams | undefined,
): Record<string, string | undefined> {
	if (!grouping) return {};
	return {
		detailLevel: String(grouping.detailLevel),
		subtotalPosition: grouping.subtotalPosition,
		open: grouping.open.length ? grouping.open.join(',') : undefined,
		closed: grouping.closed.length ? grouping.closed.join(',') : undefined,
	};
}

/**
 * Bilan. Story 46-2 — avec `grouping`, la réponse porte aussi `grouping` et
 * `groupedRows` (46-1) ; sans, elle est celle d'aujourd'hui.
 */
export async function getBalanceSheet(
	query: ReportQuery,
	grouping?: GroupingParams,
): Promise<BalanceSheetDto> {
	const qs = buildQuery({
		fiscalYearId: query.fiscalYearId,
		periodStart: query.periodStart,
		periodEnd: query.periodEnd,
		...groupingQueryParams(grouping),
	});
	return apiClient.get<BalanceSheetDto>(`/api/v1/reports/balance-sheet?${qs}`);
}

/** Compte de résultat ; `grouping` comme pour {@link getBalanceSheet}. */
export async function getIncomeStatement(
	query: ReportQuery,
	grouping?: GroupingParams,
): Promise<IncomeStatementDto> {
	const qs = buildQuery({
		fiscalYearId: query.fiscalYearId,
		periodStart: query.periodStart,
		periodEnd: query.periodEnd,
		...groupingQueryParams(grouping),
	});
	return apiClient.get<IncomeStatementDto>(`/api/v1/reports/income-statement?${qs}`);
}

/**
 * Balance des comptes ; `grouping` comme pour {@link getBalanceSheet} (Story
 * 46-6b : la réponse porte alors `grouping` et `groupedRows`, un tableau).
 */
export async function getTrialBalance(
	query: ReportQuery,
	grouping?: GroupingParams,
): Promise<TrialBalanceDto> {
	const qs = buildQuery({
		fiscalYearId: query.fiscalYearId,
		periodStart: query.periodStart,
		periodEnd: query.periodEnd,
		...groupingQueryParams(grouping),
	});
	return apiClient.get<TrialBalanceDto>(`/api/v1/reports/trial-balance?${qs}`);
}

export async function getJournalReport(query: JournalReportQuery): Promise<JournalReportDto> {
	const qs = buildQuery({
		fiscalYearId: query.fiscalYearId,
		periodStart: query.periodStart,
		periodEnd: query.periodEnd,
		journal: query.journal,
	});
	return apiClient.get<JournalReportDto>(`/api/v1/reports/journals?${qs}`);
}

export async function getVatReport(query: ReportQuery): Promise<VatReportDto> {
	const qs = buildQuery({
		fiscalYearId: query.fiscalYearId,
		periodStart: query.periodStart,
		periodEnd: query.periodEnd,
	});
	return apiClient.get<VatReportDto>(`/api/v1/reports/vat?${qs}`);
}

// ============================================================================
// Story 19-6a — Rapport « Dépenses par projet »
// ============================================================================

/** Construit la query string commune aux rapports projet (JSON + export). */
function buildProjectQuery(
	query: ProjectReportQuery,
	extra?: Record<string, string | number | undefined>,
): string {
	return buildQuery({
		projectId: query.projectId,
		mode: query.mode,
		fiscalYearId: query.mode === 'fiscal_year' ? query.fiscalYearId : undefined,
		periodStart: query.periodStart,
		periodEnd: query.periodEnd,
		...extra,
	});
}

export async function getProjectExpenses(query: ProjectReportQuery): Promise<ProjectExpensesDto> {
	const qs = buildProjectQuery(query);
	return apiClient.get<ProjectExpensesDto>(`/api/v1/reports/project-expenses?${qs}`);
}

export async function getProjectReturn(query: ProjectReportQuery): Promise<ProjectReturnDto> {
	const qs = buildProjectQuery(query);
	return apiClient.get<ProjectReturnDto>(`/api/v1/reports/project-return?${qs}`);
}

/** URL d'export d'un rapport projet. */
export function getProjectReportExportUrl(
	type: ProjectReportType,
	query: ProjectReportQuery,
	format: 'pdf' | 'csv',
): string {
	const qs = buildProjectQuery(query, { format });
	return `/api/v1/reports/${type}/export?${qs}`;
}

/** Télécharge un export de rapport projet (même mécanique que downloadReport). */
export async function downloadProjectReport(
	type: ProjectReportType,
	query: ProjectReportQuery,
	format: 'pdf' | 'csv',
	filename: string,
): Promise<void> {
	const url = getProjectReportExportUrl(type, query, format);
	const response = await apiClient.getBlob(url);
	const blob = await response.blob();
	triggerDownload(blob, filename);
}

/**
 * `true` si la **ligne calculée** « Résultat reporté (calculé) » du bilan doit être
 * affichée : seulement quand `retainedEarnings` est **non nul** (correctif
 * fix-bilan-resultat-reporte). À zéro, elle n'apporte rien et se confond avec le
 * compte physique de report itemisé juste au-dessus ; non nulle, elle signale un
 * résultat d'exercices antérieurs qu'aucune écriture n'a encore reporté. Le total des
 * capitaux propres l'inclut toujours. Pendant backend :
 * `BalanceSheet::shows_retained_earnings_line` (PDF, CSV), tenu en parallèle comme
 * `isReportEmpty`. La valeur est jugée **arrondie au centime, comme elle s'affiche**
 * (`roundToCents`) : un résidu sous le demi-centime n'affiche ni « 0.00 » ni
 * « -0.00 » (revue P1, LOW-1). Une valeur illisible reste affichée, comme avant.
 */
export function showsRetainedEarningsLine(bs: BalanceSheetDto): boolean {
	try {
		return !roundToCents(bs.retainedEarnings).eq(0);
	} catch {
		return true;
	}
}

/**
 * Helper Pass 1 AA-11 / BH-11 : détecte si un rapport est "vide" pour afficher
 * le message UX `reports-error-no-entries-in-period` (UX-DR38).
 */
export function isReportEmpty(
	type: ReportType,
	dto:
		| BalanceSheetDto
		| IncomeStatementDto
		| TrialBalanceDto
		| JournalReportDto
		| VatReportDto
		| null
		| undefined,
): boolean {
	if (!dto) return true;
	switch (type) {
		case 'balance-sheet': {
			const bs = dto as BalanceSheetDto;
			// Story 14-1 : « vide » ⇒ aucun compte de bilan ET report/résultat nuls.
			// Sinon (actif/passif retombés à 0 mais report & résultat non nuls et
			// opposés) les 2 lignes de fonds propres seraient masquées à tort.
			return (
				bs.assets.length === 0 &&
				bs.liabilities.length === 0 &&
				// Story 14-3c : + equity (3e site de la garde vide, P1-F1) — sinon un
				// reclassement pur entre comptes de fonds propres masquerait la section.
				bs.equity.length === 0 &&
				Number(bs.retainedEarnings) === 0 &&
				Number(bs.equityResult) === 0
			);
		}
		case 'income-statement': {
			const is_ = dto as IncomeStatementDto;
			return is_.revenues.length === 0 && is_.expenses.length === 0;
		}
		case 'trial-balance': {
			const tb = dto as TrialBalanceDto;
			return tb.rows.length === 0;
		}
		case 'journals': {
			const jr = dto as JournalReportDto;
			return jr.journals.every((s) => s.entries.length === 0);
		}
		case 'vat': {
			// Story 18-1d : un décompte sans vente (rows vide) mais avec de la TVA
			// récupérable (achats seuls) n'est PAS vide — il doit afficher le pied
			// de tableau (récupérable + solde). Vide = aucune vente ET récupérable 0.
			// Story 25-4-d2c : ni avec des soldes dans la période.
			const vr = dto as VatReportDto;
			return (
				vr.rows.length === 0 &&
				Number(vr.totalVatRecoverable) === 0 &&
				vr.writeOffRows.length === 0
			);
		}
	}
}

/** Une date pure `AAAA-MM-JJ`, rien d'autre. */
const PURE_DATE_RE = /^(\d{4})-(\d{2})-(\d{2})$/;
/**
 * Un horodatage : `AAAA-MM-JJ`, `T` ou une espace, `HH:mm` (secondes et fraction
 * facultatives), puis `Z`, un décalage `±HH:MM`, ou rien.
 */
const TIMESTAMP_RE = /^(\d{4}-\d{2}-\d{2})[T ](\d{2}:\d{2}(?::\d{2}(?:\.\d+)?)?)(Z|[+-]\d{2}:\d{2})?$/;

/** Le fuseau où Kesh montre ses instants : celui de la Suisse, quel que soit le poste (#1511). */
const ZONE_SUISSE = 'Europe/Zurich';

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
 * lecture « naïf = UTC » de ce module : `zurichParts` et `formatInstant` l'appellent.
 */
function instantDuServeur(iso: string): Date | null {
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

/** Les styles de `formatInstant` : la date seule, ou la date et l'heure (les styles d'avant la 0.20.0). */
const STYLES_D_INSTANT = {
	date: { dateStyle: 'medium' },
	dateHeure: { dateStyle: 'medium', timeStyle: 'short' },
} as const;

/** Les formateurs de `formatInstant`, construits au premier appel de chaque `langue|style`. */
const FORMATEURS_D_INSTANT = new Map<string, Intl.DateTimeFormat>();

/**
 * Montre un instant enregistré par Kesh **à l'heure de Zurich**, dans la langue de l'interface
 * (`locale`), quel que soit le fuseau du poste (Story 50-6, #1511) : `date` = `25 nov. 2026`,
 * `dateHeure` = `25 nov. 2026, 00:30` (`fr-CH`). Lit le `Z`, un décalage, ou rien (UTC), avec
 * jusqu'à neuf décimales. Rend `null` pour une valeur absente ou illisible : chaque appelant
 * garde son repli. Les dates **saisies** (expiration d'une clé) n'en relèvent pas.
 */
export function formatInstant(
	iso: string | null | undefined,
	style: 'date' | 'dateHeure',
	locale: string,
): string | null {
	if (!iso) return null;
	const instant = instantDuServeur(iso);
	if (!instant) return null;
	const cle = `${locale}|${style}`;
	let formateur = FORMATEURS_D_INSTANT.get(cle);
	if (!formateur) {
		formateur = new Intl.DateTimeFormat(locale, { ...STYLES_D_INSTANT[style], timeZone: ZONE_SUISSE });
		FORMATEURS_D_INSTANT.set(cle, formateur);
	}
	return formateur.format(instant);
}

/**
 * Formate un montant décimal suisse — wrapper sûr autour de `formatSwissAmount`.
 * Si la valeur ne peut être parsée (NaN, vide, malformée), retourne la chaîne brute.
 * Pass 1 code review patch P22 — DRY (anciennement dupliqué dans les 4 vues).
 */
export function formatReportAmount(v: string): string {
	try {
		return formatSwissAmount(new Big(v));
	} catch {
		return v;
	}
}

/**
 * Story 37-4 (AC-5) — un montant en devise : `USD 1'233.57`. La somme exacte
 * (quatre décimales) est arrondie à deux, l'équidistant loin de zéro (`toFixed`
 * de big.js), comme au PDF. Un montant absent se lit 0.
 */
export function formatCurrencyAmount(currency: string, v: string | undefined): string {
	return `${currency} ${formatCurrencyFigure(v)}`;
}

/**
 * Arrondi d'affichage d'un montant exact (quatre décimales stockées) : deux
 * décimales, l'équidistant loin de zéro — pendant de `round_for_display` côté
 * serveur. Revue de code P1 (Story 37-4) : un résidu négatif sous le demi-centime
 * (« -0.0040 ») s'arrondit à un zéro **sans signe**, qui ne s'affiche pas « -0.00 ».
 * Lève sur une valeur illisible (comme `new Big`).
 */
// Nommé sans suffixe `Display` : la garde des libellés en dur (`i18n-libelle-en-dur.test.ts`)
// prend toute fonction `*Display` pour un libellé ; celle-ci rend un nombre.
export function roundToCents(v: string): Big {
	const r = new Big(v).round(2, Big.roundHalfUp);
	return r.eq(0) ? new Big(0) : r;
}

/**
 * Le chiffre d'un montant en devise, sans la devise (balance, où elle est dite
 * une fois en tête de la seconde ligne) : arrondi à deux décimales, sans
 * « -0.00 » (revue de code P2 : la balance le contournait).
 */
export function formatCurrencyFigure(v: string | undefined): string {
	let amount = v ?? '0';
	try {
		amount = roundToCents(amount).toString();
	} catch {
		// valeur illisible : rendue telle quelle par formatReportAmount
	}
	return formatReportAmount(amount);
}

// ============================================================================
// Story 9-2a — Export PDF & CSV
// ============================================================================

/**
 * Construit l'URL d'un endpoint d'export `/api/v1/reports/{type}/export?format=...`.
 * Pour le rapport `journals`, le paramètre optionnel `journal` est inclus si fourni.
 *
 * Story 46-2 (AC-6) — `grouping` : les paramètres de groupement de la dernière
 * requête affichée du bilan, du compte de résultat ou de la balance (46-6b), que
 * l'export reprend tels quels pour rendre ce que montre l'écran (rendu groupé :
 * 46-3, 46-6b). Sans `grouping` (hiérarchie masquée à l'écran), `detailLevel=flat` :
 * l'export de l'API est groupé par défaut, l'écran à plat demande donc le rendu à
 * plat explicitement. Ignoré pour les autres rapports.
 */
export function getReportExportUrl(
	type: ReportType,
	query: ReportQuery | JournalReportQuery,
	format: 'pdf' | 'csv',
	grouping?: GroupingParams,
): string {
	const params: Record<string, string | number | undefined> = {
		fiscalYearId: query.fiscalYearId,
		periodStart: query.periodStart,
		periodEnd: query.periodEnd,
		format,
	};
	if (type === 'journals') {
		params.journal = (query as JournalReportQuery).journal;
	}
	if (type === 'balance-sheet' || type === 'income-statement' || type === 'trial-balance') {
		Object.assign(
			params,
			grouping ? groupingQueryParams(grouping) : { detailLevel: FLAT_DETAIL_LEVEL },
		);
	}
	const qs = buildQuery(params);
	return `/api/v1/reports/${type}/export?${qs}`;
}

/**
 * Construit le filename `kesh-{typeSlug}-{companySlug}-{periodStart}_{periodEnd}.{ext}`
 * (AC #22 + Pass 1 ECH-H3 + Pass 1 code-review H10).
 *
 * Pipeline slug ASCII : NFD-strip diacritics + lowercase + non-alphanum → `-` +
 * collapse `-+` → `-` + truncate 20 chars + strip trailing `-`. Fallback
 * `'company'` si vide post-slug (e.g. nom CJK).
 *
 * Pass 1 code-review H10 (AA2-H1 + AA4-F1) : `typeSlug` est résolu via i18n
 * (clé `reports-filename-{type}`) au lieu d'être hardcodé FR. Cohérent avec
 * la résolution backend (cf. `routes/reports.rs::resolve_type_slug`) — en
 * DE-CH on obtient `bilanz`, en IT-CH `bilancio`, etc.
 */
export function buildExportFilename(
	type: ReportType,
	companyName: string,
	period: { start: string; end: string },
	format: 'pdf' | 'csv',
): string {
	const typeSlug = i18nMsg(`reports-filename-${type}`, TYPE_SLUGS_FALLBACK[type]);
	const companySlug = slugify(companyName, 'company');
	return `kesh-${typeSlug}-${companySlug}-${period.start}_${period.end}.${format}`;
}

/**
 * Fallbacks FR-CH si la clé i18n est manquante (cohérent avec les valeurs
 * fr-CH de `reports-filename-*`). Pass 1 code-review M16 : `trial-balance`
 * mis à jour `balance` → `balance-comptes` (clarté vs `bilan`).
 */
const TYPE_SLUGS_FALLBACK: Record<ReportType, string> = {
	'balance-sheet': 'bilan',
	'income-statement': 'compte-resultat',
	'trial-balance': 'balance-comptes',
	journals: 'journaux',
	vat: 'decompte-tva',
};

/**
 * Slug ASCII strict. Pipeline (cf. AC #22) :
 * 1. NFD-decompose + strip diacritics (`/[̀-ͯ]/g`).
 * 2. lowercase.
 * 3. `[^a-z0-9-]` → `-`.
 * 4. Collapse `-+` → `-`.
 * 5. Truncate 20 chars.
 * 6. Strip trailing `-` (Pass 1 ECH-H3).
 * 7. Fallback si vide.
 */
export function slugify(input: string, fallback: string): string {
	const normalised = input
		.normalize('NFD')
		.replace(/[̀-ͯ]/g, '')
		.toLowerCase()
		.replace(/[^a-z0-9-]/g, '-')
		.replace(/-+/g, '-')
		.slice(0, 20)
		.replace(/-+$/, '')
		.replace(/^-+/, '');
	return normalised.length === 0 ? fallback : normalised;
}

/**
 * Télécharge un export PDF/CSV via `fetch` + Blob + lien `<a download>` éphémère.
 *
 * Le 401 est géré par `apiClient` (redirect login). Autres erreurs (400, 500)
 * sont relayées via une exception formatée que `+page.svelte` capture dans
 * `errorMsg` (AC #23 + Pass 1 ECH-H3).
 *
 * @param type — Type du rapport (utilisé dans le URL path).
 * @param query — Query params (`fiscalYearId`, période, journal optionnel).
 * @param format — `'pdf'` ou `'csv'`.
 * @param filename — Filename suggéré au browser (construit via `buildExportFilename`).
 * @param grouping — Story 46-2 : paramètres de groupement du bilan, du compte de
 *   résultat ou de la balance (46-6b) affiché (voir `getReportExportUrl`).
 */
// --- Story 21-7 : Balance âgée débiteurs ---

/**
 * Balance âgée des créances clients, arrêtée à aujourd'hui (v1, pas de paramètre).
 * Endpoint tous rôles (D-7b). Wrapper dédié (PAS `downloadReport`/`ReportType`
 * qui présupposent une `ReportQuery` fiscalYearId/période).
 */
export async function getAgedReceivables(): Promise<AgedReceivablesDto> {
	return apiClient.get<AgedReceivablesDto>('/api/v1/reports/aged-receivables');
}

/**
 * Télécharge le CSV de la balance âgée (réservé Comptable+ côté serveur, D24).
 * Filename `balance-agee-{today}.csv` (patron `onExportCsv` échéancier).
 */
export async function downloadAgedReceivables(): Promise<void> {
	const response = await apiClient.getBlob('/api/v1/reports/aged-receivables/export?format=csv');
	const blob = await response.blob();
	// #889 : la date du jour du poste, non celle d'UTC (la veille entre minuit et 1 h).
	triggerDownload(blob, `balance-agee-${todayLocal()}.csv`);
}

export async function downloadReport(
	type: ReportType,
	query: ReportQuery | JournalReportQuery,
	format: 'pdf' | 'csv',
	filename: string,
	grouping?: GroupingParams,
): Promise<void> {
	const url = getReportExportUrl(type, query, format, grouping);
	// On utilise `apiClient.getBlob` pour bénéficier de la gestion 401 + ApiError centralisée.
	const response = await apiClient.getBlob(url);
	const blob = await response.blob();
	triggerDownload(blob, filename);
}

/**
 * Helper interne : déclenche le download navigateur via un lien éphémère.
 *
 * Pass 1 code-review M11 (BH4-F-01) : `removeChild` + `revokeObjectURL` dans
 * un `finally` pour garantir le cleanup même si `a.click()` jette (e.g. CSP
 * bloque le download, ou popup blocker). Sans ce garde, un crash de click
 * laisserait l'objet URL et le `<a>` éphémère attachés au DOM — fuite mémoire.
 */
function triggerDownload(blob: Blob, filename: string): void {
	const objectUrl = URL.createObjectURL(blob);
	const a = document.createElement('a');
	a.href = objectUrl;
	a.download = filename;
	try {
		document.body.appendChild(a);
		a.click();
	} finally {
		// Cleanup robuste — pas de remove() si appendChild a échoué
		if (a.parentNode) a.parentNode.removeChild(a);
		URL.revokeObjectURL(objectUrl);
	}
}

/**
 * Grand livre — l'extrait d'un ou plusieurs comptes.
 *
 * ⚠️ Ne prend **pas** de `fiscalYearId` : ce rapport franchit la borne
 * d'exercice, contrairement à tous les autres.
 */
export async function getGeneralLedger(query: LedgerQuery): Promise<GeneralLedgerDto> {
	const qs = buildQuery({
		from: query.from,
		to: query.to,
		accountIds: query.accountIds?.length ? query.accountIds.join(',') : undefined,
		includeZero: query.includeZero ? 'true' : undefined,
		offset: query.offset,
		limit: query.limit,
	});
	return apiClient.get<GeneralLedgerDto>(`/api/v1/reports/general-ledger?${qs}`);
}

/**
 * Télécharge le grand livre (PDF ou CSV).
 *
 * ⚠️ L'export n'est **pas** paginé côté serveur : l'écran s'arrête à 500 lignes
 * par compte, le fichier produit contient le livre entier. C'est ce que promet
 * le bandeau de troncature de l'écran.
 */
export async function downloadGeneralLedger(
	query: LedgerQuery,
	format: 'pdf' | 'csv',
	filename: string,
): Promise<void> {
	const qs = buildQuery({
		format,
		from: query.from,
		to: query.to,
		accountIds: query.accountIds?.length ? query.accountIds.join(',') : undefined,
		includeZero: query.includeZero ? 'true' : undefined,
	});
	const response = await apiClient.getBlob(`/api/v1/reports/general-ledger/export?${qs}`);
	const blob = await response.blob();
	triggerDownload(blob, filename);
}

/**
 * Lien vers le grand livre d'un compte, sur la période affichée.
 *
 * C'est ce lien qui fait exister le rapport : sans lui, le grand livre est une
 * page que personne n'ouvre, alors que la question « de quoi ce solde est-il
 * fait ? » se pose toujours devant un bilan ou une balance.
 */
export function ledgerHref(accountId: number, from: string, to: string): string {
	// Story 55-1a (AC-6, D6) — écrite sur la fonction d'écriture de l'URL des Rapports : sortie
	// inchangée à l'octet près (`tab`, `accountId`, `from`, `to`), que l'accueil et la réévaluation
	// emploient et que `homepage-charts.spec.ts` vérifie dans cet ordre.
	return reportHref({ tab: 'general-ledger', accountId, from, to });
}

/**
 * Story 55-3b (AC-2) — les justificatifs d'une écriture, lus au clic sur son icône
 * (`GET /api/v1/journal-entries/{id}/documents`, route de la 55-3a, tout rôle authentifié).
 */
export async function fetchEntryDocuments(entryId: number): Promise<EntryDocument[]> {
	return apiClient.get<EntryDocument[]>(`/api/v1/journal-entries/${entryId}/documents`);
}

/**
 * Story 55-3b (AC-3, D12) — le PDF d'une facture, en `Blob`.
 *
 * ⚠️ `GET /invoices/{id}/pdf` **fige** une facture qui ne l'est pas encore : l'écran des Rapports ne l'appelle
 * que pour un justificatif dont `pdfStatus` vaut `'frozen'` (le PDF figé, servi sans rien écrire).
 */
export async function fetchInvoicePdf(invoiceId: number): Promise<Blob> {
	const response = await apiClient.getBlob(`/api/v1/invoices/${invoiceId}/pdf`);
	return response.blob();
}

/** Story 55-3b (AC-3) — le PDF d'un avoir, en `Blob` (`GET /credit-notes/{id}/pdf`). */
export async function fetchCreditNotePdf(creditNoteId: number): Promise<Blob> {
	const response = await apiClient.getBlob(`/api/v1/credit-notes/${creditNoteId}/pdf`);
	return response.blob();
}
