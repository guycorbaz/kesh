/**
 * Story 29-1a (#232) — libellés des contrôles préalables à la clôture,
 * partagés par le panneau « Préparer la clôture » (`FiscalYearChecks.svelte`)
 * et le rappel du dialogue de clôture (page des exercices).
 *
 * Chaque appel porte sa clé en **littéral** (garde B, `i18n-keys.test.ts`) ;
 * préfixe `fiscal-years-checks-`, celui du dossier (`lint-i18n-ownership`).
 */

import { i18nMsg } from '$lib/shared/utils/i18n.svelte';
import type {
	ClosingCheckCode,
	EntryReason,
	NotCheckableReason
} from './fiscal-years.types';

/** Le libellé d'un contrôle. */
export function checkTitle(code: ClosingCheckCode): string {
	switch (code) {
		case 'EARLIER_FISCAL_YEAR_OPEN':
			return i18nMsg('fiscal-years-checks-k1-title', 'Exercices antérieurs');
		case 'PENDING_BANK_TRANSACTIONS':
			return i18nMsg('fiscal-years-checks-k2-title', 'Transactions bancaires à rapprocher');
		case 'BANK_STATEMENT_AT_YEAR_END':
			return i18nMsg('fiscal-years-checks-k3-title', 'Relevés bancaires de fin d’exercice');
		case 'DRAFT_INVOICES':
			return i18nMsg('fiscal-years-checks-k4-title', 'Factures en brouillon');
		case 'IMPORTED_SUPPLIER_INVOICES_TO_COMPLETE':
			return i18nMsg('fiscal-years-checks-k5-title', 'Factures fournisseur à compléter');
		case 'INCONSISTENT_ENTRIES':
			return i18nMsg('fiscal-years-checks-k6-title', 'Écritures incohérentes');
		case 'NEXT_FISCAL_YEAR':
			return i18nMsg('fiscal-years-checks-k7-title', 'Exercice suivant');
	}
}

/** Le texte d'aide d'un contrôle : ce qu'il vérifie. */
export function checkHelp(code: ClosingCheckCode): string {
	switch (code) {
		case 'EARLIER_FISCAL_YEAR_OPEN':
			return i18nMsg(
				'fiscal-years-checks-k1-help',
				'Un exercice ne se clôture qu’après tous ceux qui le précèdent.'
			);
		case 'PENDING_BANK_TRANSACTIONS':
			return i18nMsg(
				'fiscal-years-checks-k2-help',
				'Les transactions importées de l’exercice qui n’ont pas encore d’écriture.'
			);
		case 'BANK_STATEMENT_AT_YEAR_END':
			return i18nMsg(
				'fiscal-years-checks-k3-help',
				'Pour chaque compte bancaire actif dans l’exercice, le relevé arrêté au dernier jour de l’exercice, comparé aux livres.'
			);
		case 'DRAFT_INVOICES':
			return i18nMsg(
				'fiscal-years-checks-k4-help',
				'Les factures datées dans l’exercice et jamais validées : elles ne sont pas en comptabilité.'
			);
		case 'IMPORTED_SUPPLIER_INVOICES_TO_COMPLETE':
			return i18nMsg(
				'fiscal-years-checks-k5-help',
				'Les pièces importées qui attendent d’être complétées, toutes dates confondues.'
			);
		case 'INCONSISTENT_ENTRIES':
			return i18nMsg(
				'fiscal-years-checks-k6-help',
				'Des écritures héritées ou restaurées que Kesh ne produit pas lui-même : déséquilibrées, incomplètes ou mal datées.'
			);
		case 'NEXT_FISCAL_YEAR':
			return i18nMsg(
				'fiscal-years-checks-k7-help',
				'L’exercice qui suit, où se passent les écritures de l’année suivante.'
			);
	}
}

/** Le motif d'un compte non vérifiable (K3). */
export function notCheckableText(reason: NotCheckableReason, name: string): string {
	switch (reason) {
		case 'NOT_COMPARABLE':
			return i18nMsg(
				'fiscal-years-checks-k3-not-comparable',
				'{ $name } : non vérifiable — sans compte de grand livre lié, ou compte lié partagé avec un autre compte bancaire.',
				{ name }
			);
		case 'NO_BALANCE_STATEMENT':
			return i18nMsg(
				'fiscal-years-checks-k3-no-balance-statement',
				'{ $name } : non vérifiable — aucun relevé importé ne porte de solde (import CSV seulement).',
				{ name }
			);
	}
}

/** Le libellé d'un motif d'écriture incohérente (K6). */
export function entryReasonText(reason: EntryReason): string {
	switch (reason) {
		case 'UNBALANCED':
			return i18nMsg('fiscal-years-checks-k6-unbalanced', 'déséquilibrée');
		case 'TOO_FEW_LINES':
			return i18nMsg('fiscal-years-checks-k6-too-few-lines', 'moins de deux lignes');
		case 'DATE_OUTSIDE_ITS_YEAR':
			return i18nMsg('fiscal-years-checks-k6-date-outside', 'datée hors de son exercice');
		case 'OTHER_YEAR_DATED_INSIDE':
			return i18nMsg(
				'fiscal-years-checks-k6-other-year-inside',
				'd’un autre exercice, datée dans celui-ci'
			);
	}
}
