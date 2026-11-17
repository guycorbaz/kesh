/**
 * Story 37-5 — logique pure de l'écran de réévaluation des comptes en devise
 * (`RevaluationPanel.svelte`, l'action « Réévaluer les devises » et l'avis du
 * contrôle K9) : quels cours envoyer, quels boutons de récupération proposer,
 * libellés de source et de genre. Aucun appel réseau ici.
 *
 * Chaque appel de traduction porte sa clé en **littéral** (garde B,
 * `i18n-keys.test.ts`) ; préfixe `fiscal-years-`, celui du dossier
 * (`lint-i18n-ownership`).
 */

import Big from 'big.js';
import { i18nMsg } from '$lib/shared/utils/i18n.svelte';
import { formatSwissDate } from '$lib/features/reports/reports.api';
import { monthLongLabel, sameRate } from '$lib/features/exchange-rates/exchange-rates-grid';
import type {
	RevaluationLine,
	RevaluationPreview,
	RevaluationRate,
	RevaluationRequest
} from './fiscal-years.types';

/** Le 1er du mois d'une date ISO. */
function monthOf(iso: string): string {
	return `${iso.slice(0, 7)}-01`;
}

/** Le 1er du mois suivant le mois de `iso`. */
function nextMonthOf(iso: string): string {
	const [y, m] = iso.split('-').map(Number);
	const ny = m === 12 ? y + 1 : y;
	const nm = m === 12 ? 1 : m + 1;
	return `${ny}-${String(nm).padStart(2, '0')}-01`;
}

/**
 * Le signe naturel d'un solde en débit net : un actif se lit au débit, un
 * passif au crédit (l'écart, lui, reste toujours en débit net).
 */
export function naturalAmount(line: Pick<RevaluationLine, 'accountType'>, amount: string): string {
	const x = new Big(amount);
	const n = line.accountType === 'Liability' ? x.times(-1) : x;
	return n.eq(0) ? '0' : n.toString();
}

/**
 * Les cours **modifiés** par l'utilisateur, à envoyer à l'aperçu : ceux dont le
 * champ diffère numériquement du cours par défaut, limités aux devises de
 * `currencies` (la requête refuse toute autre clé). Un champ vide ou égal au
 * défaut ne part pas.
 */
export function modifiedRates(
	inputs: Record<string, string>,
	rates: readonly RevaluationRate[],
	currencies: ReadonlySet<string>
): Record<string, string> {
	const out: Record<string, string> = {};
	for (const r of rates) {
		const v = (inputs[r.currency] ?? '').trim().replace(',', '.');
		if (!v || !currencies.has(r.currency)) continue;
		if (r.defaultRate !== null && sameRate(v, r.defaultRate)) continue;
		out[r.currency] = v;
	}
	return out;
}

/**
 * Les cours **affichés** des devises d'au moins un compte coché, à envoyer à
 * la passation (le serveur en déduit genre et source ; un cours égal au défaut
 * garde le genre et la source du défaut).
 */
export function displayedRates(
	inputs: Record<string, string>,
	currencies: ReadonlySet<string>
): Record<string, string> {
	const out: Record<string, string> = {};
	for (const c of currencies) {
		const v = (inputs[c] ?? '').trim().replace(',', '.');
		if (v) out[c] = v;
	}
	return out;
}

/** Les devises des comptes cochés. */
export function selectedCurrencies(
	lines: readonly RevaluationLine[],
	selected: ReadonlySet<number>
): Set<string> {
	return new Set(lines.filter((l) => selected.has(l.accountId)).map((l) => l.currency));
}

/** Le corps d'une relecture de l'aperçu : la sélection et les cours modifiés de ses devises. */
export function previewRequest(
	preview: RevaluationPreview,
	inputs: Record<string, string>,
	selected: ReadonlySet<number>
): RevaluationRequest {
	const currencies = selectedCurrencies(preview.lines, selected);
	return {
		accountIds: [...selected].sort((a, b) => a - b),
		rates: modifiedRates(inputs, preview.rates, currencies)
	};
}

/** Le corps de la passation : la sélection et les cours affichés de ses devises. */
export function postRequest(
	preview: RevaluationPreview,
	inputs: Record<string, string>,
	selected: ReadonlySet<number>
): RevaluationRequest {
	const currencies = selectedCurrencies(preview.lines, selected);
	return {
		accountIds: [...selected].sort((a, b) => a - b),
		rates: displayedRates(inputs, currencies)
	};
}

/** Le repli mensuel au genre `day` : un défaut mensuel alors que le jour est attendu. */
export function isMonthlyFallback(preview: RevaluationPreview, rate: RevaluationRate): boolean {
	return preview.closingRateKind === 'day' && rate.defaultKind === 'month';
}

/**
 * « Récupérer le cours du JJ.MM.AAAA » : genre de clôture `day`, repli mensuel
 * pris ou aucun cours, et date de fin **strictement passée** (Europe/Zurich :
 * 37-2 ne récupère un cours journalier que pour un jour antérieur à
 * aujourd'hui).
 */
export function canFetchDay(
	preview: RevaluationPreview,
	rate: RevaluationRate,
	today: string
): boolean {
	if (preview.closingRateKind !== 'day') return false;
	const missing = rate.defaultRate === null || isMonthlyFallback(preview, rate);
	return missing && preview.endDate < today;
}

/**
 * « Récupérer le mois » : aucun cours par défaut, et le mois de fin au plus le
 * mois suivant le mois courant (37-2 AC-5).
 */
export function canFetchMonth(
	preview: RevaluationPreview,
	rate: RevaluationRate,
	today: string
): boolean {
	return rate.defaultRate === null && monthOf(preview.endDate) <= nextMonthOf(today);
}

/** Date de fin pas encore passée, cours du jour attendu mais absent : le dire. */
export function dayNotYetPublished(
	preview: RevaluationPreview,
	rate: RevaluationRate,
	today: string
): boolean {
	if (preview.closingRateKind !== 'day' || preview.endDate < today) return false;
	return rate.defaultRate === null || isMonthlyFallback(preview, rate);
}

/** Le corps de `POST /exchange-rates/fetch` pour le mois de fin. */
export function monthFetchRequest(endDate: string): { year: number; month: number } {
	const [y, m] = endDate.split('-').map(Number);
	return { year: y, month: m };
}

/**
 * La source affichée (U1, A30) : « manuel » pour un cours modifié ici
 * (`request`), « AFC », « cours saisi », ou « cours saisi (<source>) » pour
 * toute autre source brute — la même que l'audit.
 */
export function sourceLabel(rate: RevaluationRate): string {
	if (rate.rateKind === 'request') {
		return i18nMsg('fiscal-years-revaluation-source-request', 'manuel');
	}
	switch (rate.defaultSource) {
		case null:
			return '';
		case 'afc':
			return i18nMsg('fiscal-years-revaluation-source-afc', 'AFC');
		case 'manual':
			return i18nMsg('fiscal-years-revaluation-source-manual', 'cours saisi');
		default:
			return i18nMsg('fiscal-years-revaluation-source-other', 'cours saisi ({ $source })', {
				source: rate.defaultSource
			});
	}
}

/**
 * Le genre affiché : « cours du JJ.MM.AAAA » (avec la date de publication si
 * elle diffère), « cours mensuel de <mois> », signalé comme repli au genre
 * `day` seulement.
 */
export function kindLabel(preview: RevaluationPreview, rate: RevaluationRate, locale: string): string {
	if (rate.rateDate === null || rate.rateKind === 'request') return '';
	if (rate.defaultKind === 'day') {
		const jour = formatSwissDate(rate.rateDate);
		if (rate.sourceDate && rate.sourceDate !== rate.rateDate) {
			return i18nMsg(
				'fiscal-years-revaluation-kind-day-published',
				'cours du { $date } (publié le { $published })',
				{ date: jour, published: formatSwissDate(rate.sourceDate) }
			);
		}
		return i18nMsg('fiscal-years-revaluation-kind-day', 'cours du { $date }', { date: jour });
	}
	const mois = monthLongLabel(rate.rateDate, locale);
	if (isMonthlyFallback(preview, rate)) {
		return i18nMsg(
			'fiscal-years-revaluation-kind-fallback',
			'cours du jour de clôture indisponible : cours mensuel de { $month }',
			{ month: mois }
		);
	}
	return i18nMsg('fiscal-years-revaluation-kind-month', 'cours mensuel de { $month }', {
		month: mois
	});
}

/** Le rappel du genre de clôture de l'exercice (R6). */
export function closingKindReminder(preview: RevaluationPreview, locale: string): string {
	return preview.closingRateKind === 'day'
		? i18nMsg(
				'fiscal-years-revaluation-closing-day',
				'Cours de clôture : cours journalier du { $date }',
				{ date: formatSwissDate(preview.endDate) }
			)
		: i18nMsg('fiscal-years-revaluation-closing-month', 'Cours de clôture : cours mensuel de { $month }', {
				month: monthLongLabel(monthOf(preview.endDate), locale)
			});
}

/** L'origine de la contrepartie (S2). */
export function originLabel(origin: RevaluationLine['fxAccountsOrigin']): string {
	switch (origin) {
		case 'own':
			return i18nMsg('fiscal-years-revaluation-origin-own', 'comptes propres');
		case 'company':
			return i18nMsg('fiscal-years-revaluation-origin-company', 'réglage de la société');
		default:
			return '';
	}
}

/** Le champ diffère du cours retenu par la société (affiché sous le champ). */
export function differsFromDefault(input: string, rate: RevaluationRate): boolean {
	const v = input.trim().replace(',', '.');
	return rate.defaultRate !== null && v !== '' && !sameRate(v, rate.defaultRate);
}

/** Le bouton « Passer les écritures » / « Réévaluer » est-il actif ? */
export function canPost(preview: RevaluationPreview, selected: ReadonlySet<number>): boolean {
	if (preview.blockers.length > 0 || selected.size === 0) return false;
	return preview.lines.some((l) => selected.has(l.accountId) && l.selectable);
}
