/**
 * Helpers de l'assistant de saisie d'achat avec TVA récupérable (Story 18-1c).
 *
 * L'assistant pré-remplit une écriture comptable d'achat en 3 lignes
 * (`D charge / D impôt préalable / C contrepartie`) à partir d'un montant HT et
 * d'un taux TVA, postée via l'endpoint existant `POST /journal-entries`. Aucun
 * appel réseau ici : pure logique de génération de lignes.
 *
 * **Parité backend (DC-c2)** : `lineVatAmount` réplique EXACTEMENT
 * `kesh_core::accounting::line_vat_amount(base_ht, rate_percent)` —
 * `round(base × taux ÷ 100, 2)` avec arrondi **half-up away-from-zero**
 * (`MidpointAwayFromZero` backend ≡ `Big.roundHalfUp` côté front car les
 * montants d'achat HT sont positifs). Tout écart d'un centime déséquilibrerait
 * l'écriture → rejet `ENTRY_UNBALANCED` (le backend compare en égalité décimale
 * exacte, sans epsilon).
 *
 * **Verrou d'équilibre (F-OPUS-C1)** : les 3 lignes émettent toutes des montants
 * arrondis à 2 décimales (`débit charge = round₂(HT)`, PAS le HT brut). Comme
 * `vat` a exactement 2 décimales, `round₂(HT) + vat == round₂(HT + vat)` —
 * l'équilibre tient par construction même pour un HT saisi à 3-4 décimales
 * (admis par la lecture du journal, `readHtAmount` ; depuis la Story 36-3, un HT à
 * trois décimales après une partie entière de un à trois chiffres, « 100.005 », est
 * refusé comme ambigu à l'écran — `buildPurchaseVatLines` le calcule toujours).
 */

import Big from 'big.js';
import { parseAmount, readJournalAmount, type JournalAmountKind } from './balance';
import type { LineDraft } from './form-helpers';
import { computeChf, readRate, type FxErrorCode } from './fx';
import { foreignChf, syncDerivedChf, type LineForeign } from './fx-lines';

/**
 * Montant de TVA pour une base HT et un taux en pourcent, arrondi au centime
 * (half-up away-from-zero), en parité exacte avec le backend `line_vat_amount`.
 *
 * @param ht base HT (string, virgule ou point acceptés)
 * @param ratePercent taux en pourcent (string, ex. `"8.10"`)
 * @returns montant TVA à 2 décimales (string, ex. `"81.00"`)
 */
export function lineVatAmount(ht: string, ratePercent: string): string {
	// `parseAmount` normalise virgule → point avant `new Big(...)` (DRY, H1).
	const base = parseAmount(ht);
	const rate = parseAmount(ratePercent);
	return base.times(rate).div(100).round(2, Big.roundHalfUp).toFixed(2);
}

/** Arrondit un montant string (virgule ou point) à 2 décimales half-up. */
function round2(amount: string): string {
	return parseAmount(amount).round(2, Big.roundHalfUp).toFixed(2);
}

/** Paramètres de génération d'une écriture d'achat avec TVA récupérable. */
export interface PurchaseVatParams {
	/** Compte de charge (Expense, débit HT). */
	chargeAccountId: number;
	/** Montant HT saisi (string, virgule ou point). */
	htAmount: string;
	/** Taux TVA en pourcent (string, ex. `"8.10"`). */
	ratePercent: string;
	/** Compte de contrepartie (crédit TTC : fournisseur, banque, caisse…). */
	counterpartyAccountId: number;
	/** Compte d'impôt préalable (`default_vat_recoverable_account_id`). */
	recoverableAccountId: number;
}

/**
 * Génère les lignes équilibrées d'une écriture d'achat avec TVA récupérable.
 *
 * - `vat > 0` → 3 lignes : `D charge round₂(HT)`, `D impôt préalable vat`,
 *   `C contrepartie round₂(HT) + vat`.
 * - `vat == 0` (taux exempt/0) → 2 lignes : `D charge round₂(HT)`,
 *   `C contrepartie round₂(HT)` (pas de ligne d'impôt préalable — une ligne à
 *   `debit = 0` serait rejetée par `chk_jel_debit_credit_exclusive`).
 *
 * Équilibre garanti par construction : `Σdebit == Σcredit` exactement (DC-c7).
 */
export function buildPurchaseVatLines(params: PurchaseVatParams): LineDraft[] {
	const {
		chargeAccountId,
		htAmount,
		ratePercent,
		counterpartyAccountId,
		recoverableAccountId
	} = params;

	const charge = round2(htAmount);
	const vat = lineVatAmount(htAmount, ratePercent);
	const ttc = parseAmount(charge).plus(vat).toFixed(2);

	const lines: LineDraft[] = [
		{ accountId: chargeAccountId, debit: charge, credit: '', projectId: null }
	];

	// Ligne d'impôt préalable émise seulement si la TVA est strictement > 0
	// (F-OPUS-1 / contrainte DB `chk_jel_debit_credit_exclusive` : `debit > 0 XOR credit > 0`).
	if (parseAmount(vat).gt(0)) {
		lines.push({ accountId: recoverableAccountId, debit: vat, credit: '', projectId: null });
	}

	lines.push({ accountId: counterpartyAccountId, debit: '', credit: ttc, projectId: null });

	return lines;
}

/** Story 36-3 (D5) — la lecture du montant HT de l'assistant. */
export interface HtAmountReading {
	kind: JournalAmountKind;
	/** Charge normalisée (point décimal, sans apostrophe) d'une saisie claire ; `null` sinon. */
	payload: string | null;
	/** Claire et `> 0` une fois arrondie au centime : l'assistant peut insérer. */
	usable: boolean;
}

/**
 * Story 36-3 (#1051, D5) — lit le montant HT saisi dans l'assistant par la lecture
 * du journal (`readJournalAmount`) : apostrophe des milliers admise, « 1.234 »
 * `ambiguous`. `usable` garde la règle d'avant : un HT qui s'arrondit à 0.00 au
 * centime (« 0.004 ») produirait une ligne de charge nulle, refusée par le serveur.
 */
export function readHtAmount(raw: string): HtAmountReading {
	const r = readJournalAmount(raw);
	if (r.kind !== 'clear') return { kind: r.kind, payload: null, usable: false };
	return {
		kind: 'clear',
		payload: r.payload,
		usable: r.value.round(2, Big.roundHalfUp).gt(0)
	};
}

/**
 * Story 36-3, revue P1 (B3, E3) — le message à afficher sous le HT : `ambiguous`,
 * `unreadable`, `below-centime` (clair, mais arrondi à 0.00 au centime), ou `null`
 * (vide ou utilisable).
 */
export function htAmountProblem(
	ht: HtAmountReading
): 'ambiguous' | 'unreadable' | 'below-centime' | null {
	if (ht.kind === 'ambiguous' || ht.kind === 'unreadable') return ht.kind;
	if (ht.kind === 'clear' && !ht.usable) return 'below-centime';
	return null;
}

/**
 * Story 36-3 (V1-7 de la validation) — l'écriture que l'assistant insère, tirée de la
 * saisie **brute** (`params.htAmount`) : `null` si le HT n'est pas `usable` ; sinon les
 * lignes de `buildPurchaseVatLines` sur la charge **normalisée** (sans apostrophe,
 * point décimal), et `isExempt` si la TVA vaut 0.00. Seul point d'entrée de
 * `VatPurchaseAssistant.insert()` : la normalisation se teste ici, sans composant.
 */
export function purchaseVatEntry(
	params: PurchaseVatParams
): { lines: LineDraft[]; isExempt: boolean } | null {
	const ht = readHtAmount(params.htAmount);
	if (!ht.usable || ht.payload === null) return null;
	return {
		lines: buildPurchaseVatLines({ ...params, htAmount: ht.payload }),
		isExempt: lineVatAmount(ht.payload, params.ratePercent) === '0.00'
	};
}

/** Story 37-3b (S5) — la partie « devise » de l'assistant TVA achat. */
export interface PurchaseVatForeign {
	/**
	 * Le gabarit des lignes en devise : devise du compte en devise, cours (et sa
	 * provenance : proposé, `afcRate`, `afcMonth`, cours de la société) ; le
	 * montant et le sens sont posés par {@link foreignPurchaseVatEntry}.
	 */
	template: LineForeign;
	/** La charge est sur un compte en devise (sinon une ligne CHF au même cours). */
	chargeIsForeign: boolean;
	/** La contrepartie est sur un compte en devise. */
	counterpartyIsForeign: boolean;
}

/**
 * Story 37-3b (AC-2, S5) — l'écriture d'achat avec TVA **en devise** : `HT` =
 * montant HT arrondi au centime, `TVA` = {@link lineVatAmount}(`HT`, taux) **en
 * devise** (même fonction qu'en CHF), `TTC` = `HT` + `TVA` ; une ligne sur un
 * compte en devise porte `foreign` (montant en devise, cours) et son CHF
 * `computeChf(montant, cours)` ; une ligne sur un compte CHF — l'impôt préalable
 * toujours — reçoit le CHF de son montant en devise **au même cours**, marquée
 * `chfFromForeign`. Les trois arrondis peuvent laisser un écart d'un centime :
 * aucune ligne n'est ajoutée, le bouton d'écart du formulaire le solde.
 *
 * Rend `null` si le HT ou le cours ne sont pas utilisables, et
 * `{ error: 'FX_LINE_ROUNDS_TO_ZERO' }` si une ligne CHF dérivée s'arrondit à 0.00
 * (une ligne nulle serait refusée par `chk_jel_debit_credit_exclusive`).
 */
export function foreignPurchaseVatEntry(
	params: PurchaseVatParams,
	fx: PurchaseVatForeign,
): { lines: LineDraft[]; isExempt: boolean } | { error: FxErrorCode } | null {
	const ht = readHtAmount(params.htAmount);
	const rate = readRate(fx.template.rate);
	if (!ht.usable || ht.payload === null || rate === null || rate === 'empty') return null;
	const charge = round2(ht.payload);
	const vat = lineVatAmount(charge, params.ratePercent);
	const ttc = parseAmount(charge).plus(vat).toFixed(2);
	const rateStr = rate.toFixed();

	/** Une ligne au montant en devise `amount` : en devise, ou CHF au même cours. */
	const line = (
		accountId: number,
		amount: string,
		side: 'debit' | 'credit',
		foreign: boolean,
	): LineDraft | { error: FxErrorCode } => {
		if (foreign) {
			const draft: LineDraft = {
				accountId,
				debit: '',
				credit: '',
				projectId: null,
				foreign: { ...fx.template, amount, side, rate: rateStr },
			};
			const chf = draft.foreign ? foreignChf(draft.foreign) : null;
			if (chf === null || !chf.ok) return { error: chf?.ok === false ? chf.error : 'FX_VALUE_OUT_OF_RANGE' };
			return syncDerivedChf(draft);
		}
		const chf = computeChf(amount, rate);
		if (!chf.ok) return { error: chf.error };
		const value = chf.value.toFixed(2);
		return {
			accountId,
			debit: side === 'debit' ? value : '',
			credit: side === 'credit' ? value : '',
			projectId: null,
			chfFromForeign: { rate: rateStr },
		};
	};

	const out: LineDraft[] = [];
	const parts: Array<[number, string, 'debit' | 'credit', boolean] | null> = [
		[params.chargeAccountId, charge, 'debit', fx.chargeIsForeign],
		parseAmount(vat).gt(0) ? [params.recoverableAccountId, vat, 'debit', false] : null,
		[params.counterpartyAccountId, ttc, 'credit', fx.counterpartyIsForeign],
	];
	for (const p of parts) {
		if (p === null) continue;
		const l = line(...p);
		if ('error' in l) return { error: l.error };
		out.push(l);
	}
	return { lines: out, isExempt: vat === '0.00' };
}

/**
 * `true` si une ligne de brouillon n'est pas vierge (compte choisi OU montant
 * saisi). Une `LineDraft` initiale `{ accountId: null, debit: '', credit: '' }`
 * est vierge. Sert à décider la confirmation avant remplacement (AC6/DC-c9).
 * Story 37-3b : le montant en devise compte aussi (il survit à un cours vide).
 */
export function isDraftLineNonEmpty(line: LineDraft): boolean {
	return (
		line.accountId !== null ||
		line.debit.trim() !== '' ||
		line.credit.trim() !== '' ||
		(line.foreign !== undefined && line.foreign.amount.trim() !== '')
	);
}
