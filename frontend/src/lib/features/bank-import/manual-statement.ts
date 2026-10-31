/**
 * Relevé bancaire saisi à la main — logique pure de l'écran (Story v016-2b, #816).
 *
 * Aucune I/O, aucun texte d'interface : le composant `ManualStatementForm.svelte`
 * traduit ce que ces fonctions rendent. Toute l'arithmétique passe par `big.js`
 * (jamais `parseFloat`), et le format des montants par `isValidAmount`
 * **généralisée** (`journal-entries/balance.ts`), non par une seconde expression.
 *
 * Ce que le module porte :
 * - les totaux et l'**écart** (`arrivée − (départ + crédits − débits)`), un solde
 *   vide n'étant pas zéro (AC-21) ;
 * - les fautes d'une ligne vues côté client et la **raison unique** d'un
 *   « Enregistrer » inactif, la première de la séquence de l'AC-21 ;
 * - le corps envoyé (lignes vides retirées, montants normalisés `"-120.50"`) et la
 *   **table des rangs** — rang envoyé → ligne affichée (AC-20) ;
 * - l'instantané d'un aperçu (`manualSnapshotKey`) et la lecture d'un refus du
 *   serveur en désignation d'une cellule, d'un champ ou de lignes (AC-22).
 *
 * Les bornes sont celles du serveur (`kesh_core::bank_imports::manual`) : 500 lignes,
 * libellé de 500 caractères, contrepartie, référence et numéro de 255, treize
 * chiffres entiers et deux décimales. Les longueurs se comptent en points de code
 * après `trim`, comme `chars().count()` côté serveur. *Asymétrie assumée* : le
 * client compte les chiffres **saisis**, le serveur la valeur normalisée.
 */

import Big from 'big.js';
import { isValidAmount, parseAmount, type AmountFormatOptions } from '$lib/features/journal-entries/balance';
import type { ManualStatementBody, ManualStatementLineBody } from './bank-import.types';

/** Nombre maximal de lignes d'un relevé saisi (C-v016-2-3). */
export const MANUAL_MAX_LINES = 500;
/** Longueur maximale du libellé, en caractères après `trim`. */
export const MANUAL_DETAILS_MAX_CHARS = 500;
/** Longueur maximale de la contrepartie, de la référence et du numéro du relevé. */
export const MANUAL_SHORT_TEXT_MAX_CHARS = 255;

/** Format d'un solde : signé, requis, treize chiffres, deux décimales. */
export const MANUAL_BALANCE_FORMAT: AmountFormatOptions = {
	maxDecimals: 2,
	maxIntegerDigits: 13,
	allowNegative: true,
	allowEmpty: false,
};

/** Format d'un montant de ligne : positif, deux décimales, treize chiffres ; vide admis. */
export const MANUAL_LINE_AMOUNT_FORMAT: AmountFormatOptions = {
	maxDecimals: 2,
	maxIntegerDigits: 13,
};

/** Une ligne telle que saisie : chaînes brutes, Débit et Crédit séparés (Q4). */
export interface ManualLineInput {
	bookingDate: string;
	valueDate: string;
	details: string;
	counterpartyName: string;
	reference: string;
	debit: string;
	credit: string;
}

/** Le formulaire tel que saisi. */
export interface ManualStatementInput {
	bankAccountId: number | null;
	periodFrom: string;
	periodTo: string;
	statementNumber: string;
	openingBalance: string;
	closingBalance: string;
	lines: ManualLineInput[];
}

/** Les trois cases de confirmation qu'exige un aperçu à avertissements. */
export interface ManualConfirmations {
	confirmDuplicateFile: boolean;
	confirmDuplicateLines: boolean;
	confirmOpeningBalanceGap: boolean;
}

/** Une cellule de ligne qui peut être fautive. */
export type ManualLineCell = 'bookingDate' | 'details' | 'counterpartyName' | 'reference' | 'debit' | 'credit';

/** Une ligne vide, celle qu'ouvre le formulaire et qu'ajoute « Ajouter une ligne ». */
export function emptyManualLine(): ManualLineInput {
	return {
		bookingDate: '',
		valueDate: '',
		details: '',
		counterpartyName: '',
		reference: '',
		debit: '',
		credit: '',
	};
}

/** Longueur en points de code après `trim` (`chars().count()` côté serveur). */
function charCount(s: string): number {
	return [...s.trim()].length;
}

/** Vrai pour une date `AAAA-MM-JJ` qui existe au calendrier. */
export function isIsoDate(s: string): boolean {
	const m = /^(\d{4})-(\d{2})-(\d{2})$/.exec(s);
	if (!m) return false;
	const [y, mo, d] = [Number(m[1]), Number(m[2]), Number(m[3])];
	const date = new Date(Date.UTC(y, mo - 1, d));
	return date.getUTCFullYear() === y && date.getUTCMonth() === mo - 1 && date.getUTCDate() === d;
}

/** Une ligne dont chaque champ est vide après `trim` : retirée à l'envoi (AC-20). */
export function isBlankLine(l: ManualLineInput): boolean {
	return (Object.keys(l) as (keyof ManualLineInput)[]).every((k) => l[k].trim() === '');
}

/** Un montant au format, ou `null` (vide ou hors forme). */
function amountOf(raw: string, format: AmountFormatOptions): Big | null {
	const t = raw.trim();
	if (t === '' || !isValidAmount(t, format)) return null;
	return parseAmount(t);
}

/** La période, si ses deux bornes sont lisibles ; sinon `null` (« hors période » non évalué). */
function readablePeriod(input: Pick<ManualStatementInput, 'periodFrom' | 'periodTo'>) {
	return isIsoDate(input.periodFrom) && isIsoDate(input.periodTo)
		? { from: input.periodFrom, to: input.periodTo }
		: null;
}

/**
 * Les cellules fautives d'une ligne non vide, vues côté client (règles de l'AC-4 de
 * la 2a) : date de comptabilisation vide, illisible ou hors période — la date de
 * valeur est libre — ; libellé vide ou trop long ; contrepartie ou référence trop
 * longues ; zéro ou deux montants ; montant nul ou hors forme. Dans cet ordre.
 */
export function lineFaults(
	l: ManualLineInput,
	period: Pick<ManualStatementInput, 'periodFrom' | 'periodTo'>,
): ManualLineCell[] {
	const faults: ManualLineCell[] = [];
	const p = readablePeriod(period);
	const date = l.bookingDate.trim();
	if (!isIsoDate(date) || (p !== null && (date < p.from || date > p.to))) faults.push('bookingDate');
	const details = charCount(l.details);
	if (details === 0 || details > MANUAL_DETAILS_MAX_CHARS) faults.push('details');
	if (charCount(l.counterpartyName) > MANUAL_SHORT_TEXT_MAX_CHARS) faults.push('counterpartyName');
	if (charCount(l.reference) > MANUAL_SHORT_TEXT_MAX_CHARS) faults.push('reference');
	const debit = l.debit.trim();
	const credit = l.credit.trim();
	if ((debit === '') === (credit === '')) {
		faults.push('debit', 'credit');
	} else {
		const cell: ManualLineCell = debit !== '' ? 'debit' : 'credit';
		const value = amountOf(debit !== '' ? debit : credit, MANUAL_LINE_AMOUNT_FORMAT);
		if (value === null || value.lte(0)) faults.push(cell);
	}
	return faults;
}

/** Totaux et écart, mis à jour à chaque frappe (AC-21). */
export interface ManualTotals {
	totalDebits: Big;
	totalCredits: Big;
	/** `départ + crédits − débits` ; `null` tant que le solde de départ n'est pas valide. */
	computedClosing: Big | null;
	/** Le solde d'arrivée saisi ; `null` s'il est vide ou invalide. */
	enteredClosing: Big | null;
	/** `arrivée − calculé` ; `null` tant que les deux soldes ne sont pas saisis. */
	gap: Big | null;
}

/** Totaux des lignes non vides, montants hors forme comptés pour zéro. */
export function computeManualTotals(input: ManualStatementInput): ManualTotals {
	let totalDebits = new Big(0);
	let totalCredits = new Big(0);
	for (const l of input.lines) {
		if (isBlankLine(l)) continue;
		totalDebits = totalDebits.plus(amountOf(l.debit, MANUAL_LINE_AMOUNT_FORMAT) ?? 0);
		totalCredits = totalCredits.plus(amountOf(l.credit, MANUAL_LINE_AMOUNT_FORMAT) ?? 0);
	}
	const opening = amountOf(input.openingBalance, MANUAL_BALANCE_FORMAT);
	const enteredClosing = amountOf(input.closingBalance, MANUAL_BALANCE_FORMAT);
	const computedClosing = opening === null ? null : opening.plus(totalCredits).minus(totalDebits);
	const gap = computedClosing === null || enteredClosing === null ? null : enteredClosing.minus(computedClosing);
	return { totalDebits, totalCredits, computedClosing, enteredClosing, gap };
}

/**
 * La raison d'un « Enregistrer » inactif — **une seule**, la première de la séquence
 * de l'AC-21 : compte ; période (vide ou illisible, puis inversée) ; numéro ;
 * soldes ; nombre de lignes ; première ligne fautive (rang **affiché**, 1-based) ;
 * écart ; aperçu ; cases.
 */
export type ManualBlockReason =
	| { kind: 'account' }
	| { kind: 'period' }
	| { kind: 'period-inverted' }
	| { kind: 'statement-number' }
	| { kind: 'balances' }
	| { kind: 'line-count' }
	| { kind: 'line'; line: number }
	| { kind: 'gap' }
	| { kind: 'preview' }
	| { kind: 'confirmations' };

/** L'état de l'aperçu, vu par la raison : réussi sur l'état courant, cases exigées manquantes. */
export interface ManualPreviewState {
	previewSucceeded: boolean;
	confirmationsMissing: boolean;
}

export function saveBlockReason(input: ManualStatementInput, preview: ManualPreviewState): ManualBlockReason | null {
	if (input.bankAccountId === null) return { kind: 'account' };
	if (!isIsoDate(input.periodFrom) || !isIsoDate(input.periodTo)) return { kind: 'period' };
	if (input.periodFrom > input.periodTo) return { kind: 'period-inverted' };
	if (charCount(input.statementNumber) > MANUAL_SHORT_TEXT_MAX_CHARS) return { kind: 'statement-number' };
	if (
		amountOf(input.openingBalance, MANUAL_BALANCE_FORMAT) === null ||
		amountOf(input.closingBalance, MANUAL_BALANCE_FORMAT) === null
	) {
		return { kind: 'balances' };
	}
	if (input.lines.filter((l) => !isBlankLine(l)).length > MANUAL_MAX_LINES) return { kind: 'line-count' };
	const faulty = input.lines.findIndex((l) => !isBlankLine(l) && lineFaults(l, input).length > 0);
	if (faulty >= 0) return { kind: 'line', line: faulty + 1 };
	const { gap } = computeManualTotals(input);
	if (gap === null || !gap.eq(0)) return { kind: 'gap' };
	if (!preview.previewSucceeded) return { kind: 'preview' };
	if (preview.confirmationsMissing) return { kind: 'confirmations' };
	return null;
}

/**
 * « Vérifier » est actif dès qu'un compte et une période lisible sont saisis, même si
 * l'écart n'est pas nul ou qu'une ligne est fautive : les refus du serveur sont ainsi
 * atteints par l'usage normal (AC-21).
 */
export function canVerify(input: ManualStatementInput): boolean {
	return input.bankAccountId !== null && isIsoDate(input.periodFrom) && isIsoDate(input.periodTo);
}

/** Texte facultatif : trimé, vide → `null`. */
function optional(s: string): string | null {
	const t = s.trim();
	return t === '' ? null : t;
}

/**
 * Un montant à envoyer : au format, normalisé en point à deux décimales (`toFixed(2)`
 * — un `Big` rend `"-120.5"` par `toString()`) ; hors forme, la saisie trimée telle
 * quelle, pour que le serveur la désigne ; vide → `''` (requis) ou `null`.
 */
function normalizedAmount(raw: string, format: AmountFormatOptions): string {
	const t = raw.trim();
	if (t !== '' && isValidAmount(t, format)) return parseAmount(t).toFixed(2);
	return t;
}

/** Le corps envoyé, et la table des rangs : `rowMap[i]` = ligne affichée (0-based) du rang envoyé `i`. */
export interface BuiltManualBody {
	body: ManualStatementBody;
	rowMap: number[];
}

/**
 * Construit le corps de l'aperçu ou de la création. Les lignes vides sont retirées et
 * la table des rangs garde la correspondance (AC-20). Une ligne à deux montants ou sans
 * montant part sans sens ou sans montant : le serveur la désigne.
 */
export function buildManualStatementBody(
	input: ManualStatementInput,
	confirmations: ManualConfirmations,
): BuiltManualBody {
	const rowMap: number[] = [];
	const lines: ManualStatementLineBody[] = [];
	input.lines.forEach((l, screenIndex) => {
		if (isBlankLine(l)) return;
		rowMap.push(screenIndex);
		const debit = l.debit.trim();
		const credit = l.credit.trim();
		const raw = debit !== '' ? debit : credit;
		const direction = debit !== '' && credit === '' ? 'debit' : credit !== '' && debit === '' ? 'credit' : null;
		lines.push({
			bookingDate: l.bookingDate.trim(),
			valueDate: optional(l.valueDate),
			details: l.details.trim(),
			counterpartyName: optional(l.counterpartyName),
			reference: optional(l.reference),
			amount: raw === '' ? null : normalizedAmount(raw, MANUAL_LINE_AMOUNT_FORMAT),
			direction,
		});
	});
	return {
		body: {
			bankAccountId: input.bankAccountId ?? 0,
			periodFrom: input.periodFrom.trim(),
			periodTo: input.periodTo.trim(),
			statementNumber: optional(input.statementNumber),
			openingBalance: normalizedAmount(input.openingBalance, MANUAL_BALANCE_FORMAT),
			closingBalance: normalizedAmount(input.closingBalance, MANUAL_BALANCE_FORMAT),
			lines,
			confirmDuplicateFile: confirmations.confirmDuplicateFile,
			confirmDuplicateLines: confirmations.confirmDuplicateLines,
			confirmOpeningBalanceGap: confirmations.confirmOpeningBalanceGap,
		},
		rowMap,
	};
}

/**
 * L'instantané d'un corps, **sans** les cases mais **avec la table des rangs** : un aperçu
 * vaut pour l'état du relevé qu'il a vu ; cocher ou décocher une case ne le périme pas
 * (AC-22). La table des rangs y entre (revue de code P1, B2 = E1) : retirer ou ajouter une
 * ligne **vide** ne change pas le corps, mais déplace les lignes de l'écran — l'aperçu et
 * le refus affichés, qui désignent des lignes, sont alors périmés.
 */
export function manualSnapshotKey(built: BuiltManualBody): string {
	const { confirmDuplicateFile: _f, confirmDuplicateLines: _l, confirmOpeningBalanceGap: _g, ...rest } = built.body;
	return JSON.stringify({ body: rest, rowMap: built.rowMap });
}

/**
 * Le rang 1-based du serveur (`line` d'une erreur ou d'un doublon) traduit en ligne de
 * l'écran (0-based) par la table de l'instantané envoyé ; `null` s'il ne désigne rien.
 */
export function screenRowOf(rowMap: readonly number[], serverLine: number | null | undefined): number | null {
	if (typeof serverLine !== 'number' || serverLine < 1 || serverLine > rowMap.length) return null;
	return rowMap[serverLine - 1];
}

/**
 * Des rangs du serveur (doublons de l'aperçu ou d'un refus) traduits en lignes de l'écran
 * (0-based) — la **seule** traduction d'une liste de rangs (revue de code P1, A1). Un rang
 * que la table ne connaît pas est un défaut (le serveur répond sur le corps envoyé) : il est
 * journalisé, et la fonction rend `null` pour que l'écran dise un texte générique — jamais
 * le rang envoyé tel quel.
 */
export function screenRowsOf(
	rowMap: readonly number[],
	serverLines: readonly (number | null | undefined)[],
): number[] | null {
	const rows: number[] = [];
	for (const line of serverLines) {
		const row = screenRowOf(rowMap, line);
		if (row === null) {
			console.error('relevé saisi : rang du serveur hors de la table des rangs', line, rowMap.length);
			return null;
		}
		rows.push(row);
	}
	return rows;
}

/** Lecture d'un refus du serveur, pour l'affichage (AC-22). */
export type ManualErrorView =
	/** `BANK_STATEMENT_INVALID` sur une ligne : la cellule (ligne affichée, 0-based). */
	| { kind: 'cell'; row: number; field: string; reason: string }
	/** `BANK_STATEMENT_INVALID` sur l'entête : le champ du formulaire. */
	| { kind: 'field'; field: string; reason: string }
	/** Doublons refusés : les lignes affichées (vide si un rang est inconnu), et le message traduit du serveur. */
	| { kind: 'rows'; rows: number[]; message: string }
	/** `400 VALIDATION_ERROR` sans `field` : corps rejeté (trop volumineux ou illisible). */
	| { kind: 'rejected' }
	/** Relevé identique : clé propre de l'écran, non le message « fichier » du serveur. */
	| { kind: 'duplicate-file' }
	/** Tout autre refus du serveur : son message, traduit par le serveur. */
	| { kind: 'message'; message: string }
	/** Une erreur qui n'est pas une réponse du serveur. */
	| { kind: 'unexpected' };

interface ApiErrorLike {
	status: number;
	code: string;
	message: string;
	details?: Record<string, unknown>;
}

function isApiErrorLike(e: unknown): e is ApiErrorLike {
	return (
		typeof e === 'object' &&
		e !== null &&
		typeof (e as ApiErrorLike).code === 'string' &&
		typeof (e as ApiErrorLike).status === 'number'
	);
}

/** Traduit un refus en désignation, par la table des rangs de l'instantané envoyé. */
export function describeManualError(err: unknown, rowMap: readonly number[]): ManualErrorView {
	if (!isApiErrorLike(err)) return { kind: 'unexpected' };
	const d = err.details ?? {};
	switch (err.code) {
		case 'BANK_STATEMENT_INVALID': {
			const field = String(d.field ?? '');
			const reason = String(d.reason ?? '');
			const row = screenRowOf(rowMap, d.line as number | null | undefined);
			return row === null ? { kind: 'field', field, reason } : { kind: 'cell', row, field, reason };
		}
		case 'BANK_STATEMENT_DUPLICATE_LINES': {
			const lines = Array.isArray(d.lines) ? (d.lines as { line?: number }[]) : [];
			const rows = screenRowsOf(
				rowMap,
				lines.map((l) => l.line),
			);
			return { kind: 'rows', rows: rows ?? [], message: err.message };
		}
		case 'BANK_IMPORT_DUPLICATE_FILE':
			return { kind: 'duplicate-file' };
		case 'VALIDATION_ERROR':
			return d.field === undefined ? { kind: 'rejected' } : { kind: 'message', message: err.message };
		default:
			return { kind: 'message', message: err.message };
	}
}
