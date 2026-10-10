/**
 * Logique pure de l'écran des postes ouverts (Story 15-1c-i, #518) — aucune
 * clé i18n ici (les textes sont dans `open-items-labels.ts`).
 *
 * ⛔ **Aucune règle du serveur recopiée.** La case suit `manuallyLetterable`,
 * la période suit `inOpenPeriod`, la prévision du délettrage suit
 * `manualDissolutionBlockedBy`, le sens d'un solde suit **son seul signe** :
 * le type du compte n'est jamais lu.
 */
import Big from 'big.js';
import { isApiError } from '$lib/shared/utils/api-client';
import type { DocumentRef, LetteringDetail, SelectedLine } from './open-items.types';

/**
 * Plafond de lignes d'un lettrage — miroir de `MAX_LINES_PER_GROUP`
 * (`crates/kesh-core/src/lettering.rs`). Le serveur reste l'autorité (400
 * `LETTERING_TOO_MANY_LINES`) ; le bouton ne fait que prévenir.
 */
export const MAX_LINES_PER_GROUP = 200;

// ---------------------------------------------------------------------------
// Dates et état d'URL (AC1)
// ---------------------------------------------------------------------------

/** La date **locale** du navigateur, `AAAA-MM-JJ` (C-15-1c-5 — jamais l'UTC). */
export function todayLocal(now: Date = new Date()): string {
	const y = String(now.getFullYear()).padStart(4, '0');
	const m = String(now.getMonth() + 1).padStart(2, '0');
	const d = String(now.getDate()).padStart(2, '0');
	return `${y}-${m}-${d}`;
}

/**
 * Vrai si `s` est une date réelle `AAAA-MM-JJ` d'une année entre 1000 et 9999
 * — la plage que le serveur accepte (`parse_as_of`) : l'écran ne provoque
 * jamais son 400.
 */
export function isIsoDate(s: string | null | undefined): s is string {
	if (!s || !/^\d{4}-\d{2}-\d{2}$/.test(s)) return false;
	const [y, m, d] = s.split('-').map(Number);
	if (y < 1000) return false;
	const dt = new Date(Date.UTC(y, m - 1, d));
	return dt.getUTCFullYear() === y && dt.getUTCMonth() === m - 1 && dt.getUTCDate() === d;
}

/** L'état de l'écran porté par l'URL. */
export interface ScreenState {
	accountId: number | null;
	/** `null` si absent ou mal formé — l'écran réécrit alors la date du jour. */
	asOf: string | null;
	group: string | null;
}

/** Lit `accountId`, `asOf` et `group` de l'URL ; toute valeur invalide → `null`. */
export function parseScreenState(params: URLSearchParams): ScreenState {
	const rawAccount = params.get('accountId');
	const accountId =
		rawAccount !== null && /^[1-9]\d{0,15}$/.test(rawAccount) ? Number(rawAccount) : null;
	const rawAsOf = params.get('asOf');
	const group = params.get('group')?.trim() || null;
	return { accountId, asOf: isIsoDate(rawAsOf) ? rawAsOf : null, group };
}

/** Écrit l'état dans une copie de `base` (chemin conservé, autres paramètres remplacés). */
export function screenUrl(base: URL, s: ScreenState): URL {
	const url = new URL(base);
	const q = new URLSearchParams();
	if (s.accountId !== null) q.set('accountId', String(s.accountId));
	if (s.asOf !== null) q.set('asOf', s.asOf);
	if (s.group !== null) q.set('group', s.group);
	url.search = q.toString();
	return url;
}

/**
 * Le lien d'un groupe : `/open-items?group=<code>`, avec le compte et la date
 * quand on les connaît (C-15-1c-3 — sans eux, le groupe s'ouvre seul).
 */
export function groupHref(code: string, accountId?: number | null, asOf?: string | null): string {
	return openItemsPath(accountId ?? null, asOf ?? null, code);
}

/**
 * Story 15-1c-ii (AC9) — les postes ouverts d'un compte à une date :
 * `/open-items?accountId=<id>&asOf=<date>`. Le Grand livre y renvoie, avec la
 * fin de sa période pour date : la liste dont le total égale la clôture de la
 * section, au signe près.
 */
export function openItemsHref(accountId: number, asOf: string): string {
	return openItemsPath(accountId, asOf, null);
}

/**
 * Story 15-1c-ii (AC9, C-15-1c-8) — les identifiants des comptes lettrables
 * d'une liste de comptes, archivés compris : le Grand livre n'offre « Postes
 * ouverts de ce compte » qu'à eux (un compte non lettrable rendrait 409).
 */
export function letterableAccountIds(
	accounts: ReadonlyArray<{ id: number; letterable: boolean }>
): Set<number> {
	return new Set(accounts.filter((a) => a.letterable).map((a) => a.id));
}

/** Un seul constructeur des liens vers l'écran, dans l'ordre que lit `parseScreenState`. */
function openItemsPath(accountId: number | null, asOf: string | null, group: string | null): string {
	const q = new URLSearchParams();
	if (accountId !== null) q.set('accountId', String(accountId));
	if (asOf) q.set('asOf', asOf);
	if (group !== null) q.set('group', group);
	return `/open-items?${q}`;
}

// ---------------------------------------------------------------------------
// Pièces (AC2)
// ---------------------------------------------------------------------------

/**
 * La cible du lien d'une pièce (C-15-1c-6), ou `null` : un règlement sans
 * facture n'a pas de lien, une transaction bancaire non plus (aucune route
 * n'ouvre une transaction).
 */
export function documentHref(doc: DocumentRef): string | null {
	switch (doc.type) {
		case 'invoice':
			return `/invoices/${doc.id}`;
		case 'creditNote':
			return `/credit-notes/${doc.id}`;
		case 'supplierInvoice':
			return `/supplier-invoices/${doc.id}`;
		case 'settlement':
			return doc.invoiceId === null ? null : `/invoices/${doc.invoiceId}`;
		case 'bankTransaction':
			return null;
		default: {
			const _exhaustif: never = doc.type;
			void _exhaustif;
			return null;
		}
	}
}

/**
 * Le numéro de la pièce d'un groupe `document` (validation P3, F-4 ;
 * C-15-1c-25) : le `number` de la **première** ligne `invoice` ou
 * `supplierInvoice`, sinon l'`invoiceNumber` de la première ligne
 * `settlement`, sinon `null`. Un groupe facture + avoir rend le numéro de la
 * facture, jamais celui de l'avoir.
 */
export function groupDocument(
	group: Pick<LetteringDetail, 'lines'>,
): { number: string; href: string | null } | null {
	for (const l of group.lines) {
		const d = l.document;
		if (d && (d.type === 'invoice' || d.type === 'supplierInvoice') && d.number) {
			return { number: d.number, href: documentHref(d) };
		}
	}
	for (const l of group.lines) {
		const d = l.document;
		if (d && d.type === 'settlement' && d.invoiceNumber) {
			return { number: d.invoiceNumber, href: documentHref(d) };
		}
	}
	return null;
}

// ---------------------------------------------------------------------------
// Sélection et lettrage manuel (AC4)
// ---------------------------------------------------------------------------

/** `Σ(débit − crédit)` en décimal — jamais `parseFloat` (« somme nulle » serait faux). */
export function selectionSum(lines: Iterable<Pick<SelectedLine, 'debit' | 'credit'>>): Big {
	let sum = new Big(0);
	for (const l of lines) sum = sum.plus(new Big(l.debit)).minus(new Big(l.credit));
	return sum;
}

/** Ce qui empêche « Lettrer », dans l'ordre de la fiche ; `null` = actif. */
export type LetterBlocker =
	| { kind: 'tooFew' }
	| { kind: 'tooMany' }
	| { kind: 'unbalanced'; difference: Big }
	| { kind: 'allClosed' };

/**
 * L'état du bouton « Lettrer » (AC4) : ≥ 2 lignes, ≤ 200, somme exactement
 * nulle, au moins une ligne en période ouverte. Prévision **indicative** : le
 * refus du serveur reste l'autorité.
 */
export function letterBlocker(lines: SelectedLine[]): LetterBlocker | null {
	if (lines.length < 2) return { kind: 'tooFew' };
	if (lines.length > MAX_LINES_PER_GROUP) return { kind: 'tooMany' };
	const difference = selectionSum(lines);
	if (!difference.eq(0)) return { kind: 'unbalanced', difference };
	if (!lines.some((l) => l.inOpenPeriod)) return { kind: 'allClosed' };
	return null;
}

/**
 * ⛔ Tout refus 404 ou 409 dit que la liste est périmée (C-15-1c-17) : la liste
 * et les propositions se rechargent, la sélection est effacée — **sans liste de
 * codes**, `LETTERING_CONCURRENT_CHANGE` compris (C-15-1c-24). Seuls les 400 de
 * forme gardent la sélection.
 */
export function isStaleRefusal(err: unknown): boolean {
	return isApiError(err) && (err.status === 404 || err.status === 409);
}

// ---------------------------------------------------------------------------
// Le sens d'un solde (AC8)
// ---------------------------------------------------------------------------

/**
 * Le sens d'un montant en **sens débit** (`Σ(débit − crédit)`), lu sur son
 * **seul signe** (C-15-1c-16) : aucun type de compte n'est lu.
 */
export function sideOf(amount: string): { abs: Big; side: 'debit' | 'credit' | null } {
	const v = new Big(amount);
	if (v.gt(0)) return { abs: v, side: 'debit' };
	if (v.lt(0)) return { abs: v.abs(), side: 'credit' };
	return { abs: v.abs(), side: null };
}

/** Égalité **numérique** de deux montants décimaux (`"-500.0000"` = `"-500"`). */
export function sameAmount(a: string, b: string): boolean {
	return new Big(a).eq(new Big(b));
}
