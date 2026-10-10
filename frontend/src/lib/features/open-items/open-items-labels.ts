/**
 * Textes de l'écran des postes ouverts (Story 15-1c-i, #518).
 *
 * ⚠️ Chaque fonction passe par `i18nMsg` avec une clé **littérale** (garde
 * `i18n-keys.test.ts` : aucun site non résolu ajouté) ; le repli est la valeur
 * `fr-CH` du catalogue, au caractère près (G13). Des `switch` exhaustifs : un
 * code neuf du serveur fait rougir le type-check au lieu d'afficher du vide.
 */
import { i18nMsg } from '$lib/shared/utils/i18n.svelte';
import { formatSwissAmount } from '$lib/features/journal-entries/balance';
import Big from 'big.js';
import { sideOf, type LetterBlocker } from './open-items';
import type {
	DocumentRef,
	DocumentState,
	LetteringOrigin,
	ManualDissolutionBlocker,
	OpenItem,
	OpenItemReason,
} from './open-items.types';

/** Un montant décimal en notation suisse, deux décimales (quatre s'il porte une fraction de centime). */
export function formatAmount(v: Big): string {
	// Une fraction de centime (les montants sont à quatre décimales) s'affiche à
	// quatre : « 0.00 » dirait nul un écart qui ne l'est pas (revue P1, E-7).
	return formatSwissAmount(v, v.eq(v.round(2)) ? 2 : 4);
}

/** `Exercice 2026 n° 12` — le numéro repart à 1 à chaque exercice (C127). */
export function entryRefLabel(fiscalYearName: string, entryNumber: number): string {
	return i18nMsg('open-items-entry-ref', '{ $fiscalYear } n° { $number }', {
		fiscalYear: fiscalYearName,
		number: entryNumber,
	});
}

/** Motif à la date (AC3, première dimension). */
export function reasonLabel(reason: OpenItemReason): string {
	switch (reason) {
		case 'unlettered':
			return i18nMsg('open-items-reason-unlettered', 'non lettrée');
		case 'letteredAfterAsOf':
			return i18nMsg('open-items-reason-lettered-after', 'lettrée après cette date');
		default: {
			const _exhaustif: never = reason;
			void _exhaustif;
			return '';
		}
	}
}

/** « le 12.03.2026 », après le code d'une ligne lettrée après la date. */
export function letteredOnLabel(date: string): string {
	return i18nMsg('open-items-reason-lettered-on', 'le { $date }', { date });
}

/** L'état de la pièce **aujourd'hui** (AC3, seconde dimension). */
export function documentStateLabel(state: DocumentState, amountDue: string | null): string {
	const amount = amountDue === null ? '' : formatAmountString(amountDue);
	switch (state) {
		case 'unpaid':
			return i18nMsg('open-items-state-unpaid', 'facture non réglée — reste dû : { $amount }', {
				amount,
			});
		case 'partiallySettled':
			return i18nMsg(
				'open-items-state-partially-settled',
				'facture partiellement réglée — reste dû : { $amount }',
				{ amount },
			);
		case 'nothingDue':
			return nothingDueLabel();
		case 'paidWithoutSettlementEntry':
			return i18nMsg(
				'open-items-state-paid-without-settlement',
				"marquée payée avant la v0.12.0, sans écriture d'encaissement : le compte porte encore cette créance",
			);
		default: {
			const _exhaustif: never = state;
			void _exhaustif;
			return '';
		}
	}
}

/** Le texte de `nothingDue` — écrit une fois, lu par le motif et par l'infobulle. */
function nothingDueLabel(): string {
	return i18nMsg(
		'open-items-state-nothing-due',
		'pièce soldée, lettrage non posé : période close ou avoir hérité — rien à faire ici',
	);
}

/** Montant décimal reçu en chaîne, formaté ; une chaîne illisible est rendue telle quelle. */
export function formatAmountString(v: string): string {
	try {
		return formatAmount(new Big(v));
	} catch {
		return v;
	}
}

/**
 * Pourquoi une ligne n'a **pas** de case (AC4) — par cause, dans l'ordre de la
 * fiche ; `''` si aucune des trois causes ne s'applique.
 */
export function noCheckboxLabel(item: OpenItem): string {
	if (item.reason === 'letteredAfterAsOf') {
		return i18nMsg('open-items-no-checkbox-lettered', 'déjà lettrée (code { $code })', {
			code: item.letteringCode ?? '',
		});
	}
	if (item.documentState === 'nothingDue') return nothingDueLabel();
	if (item.document && item.document.type !== 'bankTransaction') {
		return i18nMsg(
			'open-items-no-checkbox-document',
			'son lettrage suit sa pièce et ses règlements',
		);
	}
	return '';
}

/** Ce qui manque au bouton « Lettrer », dans l'ordre de la fiche. */
export function letterBlockerLabel(b: LetterBlocker): string {
	switch (b.kind) {
		case 'tooFew':
			return i18nMsg('open-items-letter-too-few', 'Sélectionnez au moins deux lignes.');
		case 'tooMany':
			return i18nMsg('open-items-letter-too-many', '{ $max } lignes au plus.', { max: 200 });
		case 'unbalanced':
			return i18nMsg(
				'open-items-letter-unbalanced',
				"La sélection ne s'équilibre pas : écart { $difference }.",
				{ difference: formatAmount(b.difference) },
			);
		case 'allClosed':
			return i18nMsg(
				'open-items-letter-all-closed',
				"Toutes ces lignes sont dans une période close : le lettrage n'y change plus.",
			);
		default: {
			const _exhaustif: never = b;
			void _exhaustif;
			return '';
		}
	}
}

/** Le texte de la pièce d'une ligne (AC2) — le lien est décidé par `documentHref`. */
export function documentLabel(doc: DocumentRef): string {
	switch (doc.type) {
		case 'invoice':
		case 'creditNote':
			return doc.number ?? '—';
		case 'supplierInvoice':
			return (
				doc.number ??
				i18nMsg('open-items-document-supplier-invoice', 'facture fournisseur')
			);
		case 'settlement':
			return i18nMsg('open-items-document-settlement', 'règlement de { $number }', {
				number: doc.invoiceNumber ?? '—',
			});
		case 'bankTransaction':
			return i18nMsg('open-items-document-bank-transaction', 'transaction bancaire');
		default: {
			const _exhaustif: never = doc.type;
			void _exhaustif;
			return '';
		}
	}
}

/**
 * Le motif d'un délettrage refusé d'avance (AC6) — au **texte même** que le
 * serveur emploie pour ce refus : les clés `error-lettering-*`, aucun texte
 * parallèle. Le suffixe du numéro de pièce n'est pas recomposé.
 */
export function dissolutionBlockerLabel(code: ManualDissolutionBlocker): string {
	switch (code) {
		case 'LETTERING_IS_DOCUMENT':
			return i18nMsg(
				'error-lettering-is-document',
				"Ce lettrage est celui d'une pièce : il suit la pièce et ses règlements, et ne se défait pas à la main.",
			);
		case 'LETTERING_LINE_OWNED_BY_DOCUMENT':
			return i18nMsg(
				'error-lettering-line-owned-by-document',
				'Une de ces lignes appartient à une pièce : elle ne se lettre ni ne se délettre à la main.',
			);
		case 'LETTERING_ALL_LINES_IN_CLOSED_PERIODS':
			return i18nMsg(
				'error-lettering-all-lines-in-closed-periods',
				"Toutes ces lignes sont dans une période close — exercice clôturé, exercice suivi d'un exercice clôturé, ou période verrouillée : le lettrage n'y change plus.",
			);
		default: {
			const _exhaustif: never = code;
			void _exhaustif;
			return '';
		}
	}
}

/** « débiteur » / « créditeur » — le sens d'un solde, lu sur son seul signe (AC8). */
export function sideLabel(side: 'debit' | 'credit'): string {
	return side === 'debit'
		? i18nMsg('open-items-side-debit', 'débiteur')
		: i18nMsg('open-items-side-credit', 'créditeur');
}

/**
 * Un montant du pied, en **valeur absolue suivie de son sens** (C-15-1c-16) :
 * `-500.0000` → « 500.00 créditeur », `0` → « 0.00 ».
 */
export function amountWithSideLabel(amount: string): string {
	const { abs, side } = sideOf(amount);
	const formatted = formatAmount(abs);
	return side === null ? formatted : `${formatted} ${sideLabel(side)}`;
}

/**
 * L'origine d'un groupe, en clair (AC6) : `document` → « lettrage de la pièce »
 * suivi de son numéro (`groupDocument`, C-15-1c-25), sinon « lettrage d'une
 * pièce ». ⚠️ Jamais « règlement » : un groupe `document` naît aussi d'un avoir
 * (C-15-1c-18).
 */
export function originLabel(origin: LetteringOrigin, documentNumber: string | null): string {
	switch (origin) {
		case 'manual':
			return i18nMsg('open-items-origin-manual', 'lettrage manuel');
		case 'reversal':
			return i18nMsg('open-items-origin-reversal', 'contre-passation');
		case 'document':
			return documentNumber === null
				? i18nMsg('open-items-origin-document-unknown', "lettrage d'une pièce")
				: i18nMsg('open-items-origin-document', 'lettrage de la pièce { $number }', {
						number: documentNumber,
					});
		default: {
			const _exhaustif: never = origin;
			void _exhaustif;
			return '';
		}
	}
}
