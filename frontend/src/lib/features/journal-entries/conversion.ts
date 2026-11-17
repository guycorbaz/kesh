/**
 * Story 37-3b (AC-2 bis, R7) — l'assistant *Conversion* : une conversion entre
 * deux devises passe par le compte de conversion de la société, en **deux
 * écritures**, chacune rapprochable d'un seul relevé.
 *
 * - colonne *Débit* (écriture 1, date du débit sur le compte EUR) : le compte
 *   EUR est **crédité** au grand livre (montant EUR × cours = CHF₁), les frais
 *   sont au débit, le compte de conversion au débit du solde ;
 * - colonne *Crédit* (écriture 2, date du crédit sur le compte USD) : le compte
 *   USD est **débité** (montant USD × cours = CHF₂), les frais au débit, le
 *   compte de conversion **crédité du même CHF** qu'à l'écriture 1 (il revient
 *   à 0), et la **différence de change** qui équilibre l'écriture 2 — au
 *   crédit du compte de gains si les débits l'emportent, au débit du compte de
 *   pertes sinon, sur les comptes de différence du compte USD (`fxAccountsFor`).
 *
 * Lignes de chaque écriture, dans l'ordre que lit la table des refus
 * (`details.line`) : compte en devise, frais, compte de conversion, ligne de
 * différence. Fonctions pures ; le composant porte l'état et les appels.
 */

import Big from 'big.js';
import { accountCurrency, type AccountResponse } from '$lib/features/accounts/accounts.types';
import { fxAccountsFor, type FxAccountsOrigin } from '$lib/features/accounts/fx-accounts';
import type { CompanyJson } from '$lib/features/settings/settings.types';
import { computeChf, readForeignAmount, readRate } from './fx';
import { foreignChf, rateSourceToSend, type LineForeign } from './fx-lines';
import { journalAmountReader } from './balance';
import type { CreateJournalEntryLineRequest, CreateJournalEntryRequest } from './journal-entries.types';

/** Montant en devise lu, `null` vide ou illisible. */
function amountOrNull(raw: string): Big | null {
	const a = readForeignAmount(raw);
	return a === 'empty' ? null : a;
}

/** Montant et cours normalisés d'une partie en devise lisible (`toFixed`). */
function normalized(f: LineForeign): { currency: string; amount: string; rate: string } {
	const a = amountOrNull(f.amount);
	const r = readRate(f.rate);
	return {
		currency: f.currency,
		amount: a ? a.toFixed() : f.amount,
		rate: r !== null && r !== 'empty' ? r.toFixed() : f.rate,
	};
}

/** Une ligne de frais d'une colonne (toujours au débit). */
export interface FeeLine {
	accountId: number | null;
	/** Montant saisi : en CHF pour un compte CHF, dans la devise de la colonne sinon. */
	amount: string;
}

/** Une colonne de l'assistant. */
export interface ConversionColumn {
	date: string;
	accountId: number | null;
	/** Partie en devise du compte de la colonne (montant, cours, provenance). */
	foreign: LineForeign | null;
	fees: FeeLine[];
}

/** Ce que l'assistant connaît de la société et des exercices. */
export interface ConversionContext {
	accounts: AccountResponse[];
	company: Pick<CompanyJson, 'fxGainAccountId' | 'fxLossAccountId' | 'fxTransitAccountId'> | null;
	/** Exercices ouverts (bornes), pour le motif « date hors exercice ouvert ». */
	openFiscalYears: { startDate: string; endDate: string }[];
}

/** Motif pour lequel « Passer les écritures » est désactivé. */
export type ConversionBlocker =
	| 'incomplete'
	| 'date-outside-open-fiscal-year'
	| 'same-currency'
	| 'transit-missing'
	| 'pair-invalid'
	| 'fee-second-foreign-account'
	| 'transit-not-positive'
	| 'fee-invalid';

/** Une ligne de l'aperçu. */
export interface PreviewLine {
	accountId: number;
	debit: string;
	credit: string;
	/** « EUR 1’000.00 × 0.947 » pour une ligne en devise. */
	foreign: { currency: string; amount: string; rate: string } | null;
	role: 'currency' | 'fee' | 'transit' | 'difference';
}

/** Le plan de la conversion : corps à envoyer et aperçu, ou les motifs de blocage. */
export type ConversionPlan =
	| { kind: 'blocked'; blockers: ConversionBlocker[]; pairOrigin: FxAccountsOrigin | 'missing' | null; holderNumber: string | null }
	| {
			kind: 'ready';
			entries: [CreateJournalEntryRequest, CreateJournalEntryRequest];
			preview: [PreviewLine[], PreviewLine[]];
			difference: { accountId: number; side: 'debit' | 'credit'; amount: string } | null;
			pairOrigin: FxAccountsOrigin;
			holderNumber: string;
	  };

function accountOf(ctx: ConversionContext, id: number | null): AccountResponse | undefined {
	return id === null ? undefined : ctx.accounts.find((a) => a.id === id);
}

/** Un compte de différence de change utilisable (comme le bouton d'écart). */
function usableFxAccount(ctx: ConversionContext, id: number): boolean {
	const a = accountOf(ctx, id);
	return (
		a !== undefined &&
		a.active &&
		a.postable &&
		(a.accountType === 'Expense' || a.accountType === 'Revenue') &&
		accountCurrency(a) === null
	);
}

/** La date tombe dans un exercice ouvert. */
function inOpenFiscalYear(ctx: ConversionContext, date: string): boolean {
	return ctx.openFiscalYears.some((fy) => fy.startDate <= date && date <= fy.endDate);
}

/**
 * Le CHF d'une ligne de frais : un frais sur un compte CHF vaut son montant ; un
 * frais sur un compte de la devise de la colonne se convertit **au cours de la
 * colonne**. `null` si le montant n'est pas lisible.
 */
export function feeLine(
	ctx: ConversionContext,
	fee: FeeLine,
	column: ConversionColumn,
): { chf: Big; foreign: { currency: string; amount: Big; rate: Big } | null } | null {
	const account = accountOf(ctx, fee.accountId);
	if (!account) return null;
	const currency = accountCurrency(account);
	if (currency === null) {
		// Un frais en CHF : au centime, sans arrondi silencieux (« 12.345 » est
		// refusé, non envoyé 12.35), positif (revue de code P1, 37-3b-CR-P1-9).
		const v = journalAmountReader(fee.amount);
		return v === null || v.lte(0) || v.round(2, Big.roundDown).cmp(v) !== 0 ? null : { chf: v, foreign: null };
	}
	// Un frais en devise est dans la devise de la colonne, et nulle autre : un
	// compte resté d'une devise précédente de la colonne ne se convertit pas au
	// cours de la nouvelle (revue de code P1, 37-3b-CR-P1-4).
	const columnAccount = accountOf(ctx, column.accountId);
	if (!columnAccount || accountCurrency(columnAccount) !== currency) return null;
	const amount = amountOrNull(fee.amount);
	const rate = column.foreign ? readRate(column.foreign.rate) : null;
	if (amount === null || rate === null || rate === 'empty') return null;
	const chf = computeChf(amount, rate);
	return chf.ok ? { chf: chf.value, foreign: { currency, amount, rate } } : null;
}

/**
 * Le plan de la conversion (aperçu et corps), ou les motifs de blocage. Les
 * corps portent la ligne de différence que montre l'aperçu (D6) — l'assistant
 * n'envoie pas `completeDifference`.
 */
export function planConversion(
	ctx: ConversionContext,
	debitCol: ConversionColumn,
	creditCol: ConversionColumn,
	labels: { first: string; second: string },
): ConversionPlan {
	const blockers: ConversionBlocker[] = [];
	const from = accountOf(ctx, debitCol.accountId);
	const to = accountOf(ctx, creditCol.accountId);
	const fromCur = from ? accountCurrency(from) : null;
	const toCur = to ? accountCurrency(to) : null;
	const chf1 = debitCol.foreign ? foreignChf(debitCol.foreign) : null;
	const chf2 = creditCol.foreign ? foreignChf(creditCol.foreign) : null;
	if (!from || !to || fromCur === null || toCur === null || chf1 === null || chf2 === null || !chf1.ok || !chf2.ok) {
		blockers.push('incomplete');
	}
	if (fromCur !== null && toCur !== null && fromCur === toCur) blockers.push('same-currency');
	for (const col of [debitCol, creditCol]) {
		if (!/^\d{4}-\d{2}-\d{2}$/.test(col.date) || !inOpenFiscalYear(ctx, col.date)) {
			if (!blockers.includes('date-outside-open-fiscal-year')) blockers.push('date-outside-open-fiscal-year');
		}
	}
	const transit = ctx.company?.fxTransitAccountId ?? null;
	if (transit === null) blockers.push('transit-missing');

	// La paire de l'écriture 2 : celle du compte USD, contrôlée même à différence nulle.
	let pairOrigin: FxAccountsOrigin | 'missing' | null = null;
	const pair = to && toCur !== null ? fxAccountsFor(to, ctx.company ?? { fxGainAccountId: null, fxLossAccountId: null }) : null;
	if (to && toCur !== null) {
		if (pair === null) {
			pairOrigin = 'missing';
			blockers.push('pair-invalid');
		} else {
			pairOrigin = pair.origin;
			if (!usableFxAccount(ctx, pair.gain) || !usableFxAccount(ctx, pair.loss)) blockers.push('pair-invalid');
		}
	}

	const fees1 = debitCol.fees.filter((f) => f.accountId !== null || f.amount.trim() !== '').map((f) => feeLine(ctx, f, debitCol));
	const fees2 = creditCol.fees.filter((f) => f.accountId !== null || f.amount.trim() !== '').map((f) => feeLine(ctx, f, creditCol));
	if (fees1.includes(null) || fees2.includes(null)) blockers.push('fee-invalid');

	if (blockers.length > 0 || !from || !to || !chf1?.ok || !chf2?.ok || transit === null || pair === null) {
		return { kind: 'blocked', blockers, pairOrigin, holderNumber: to?.number ?? null };
	}
	const fees1Ok = fees1 as NonNullable<(typeof fees1)[number]>[];
	const fees2Ok = fees2 as NonNullable<(typeof fees2)[number]>[];
	const feesSum1 = fees1Ok.reduce((s, f) => s.plus(f.chf), new Big(0));
	const feesSum2 = fees2Ok.reduce((s, f) => s.plus(f.chf), new Big(0));
	const transitChf = chf1.value.minus(feesSum1);
	if (transitChf.lte(0)) {
		return { kind: 'blocked', blockers: ['transit-not-positive'], pairOrigin, holderNumber: to.number };
	}
	const gap = chf2.value.plus(feesSum2).minus(transitChf);
	const difference = gap.eq(0)
		? null
		: gap.gt(0)
			? { accountId: pair.gain, side: 'credit' as const, amount: gap.toFixed(2) }
			: { accountId: pair.loss, side: 'debit' as const, amount: gap.abs().toFixed(2) };

	// Un frais de l'écriture 2 sur un second compte en devise qui n'est pas un
	// compte de charge de la devise (A33) : paire indéterminée pour le serveur
	// dès qu'il y a une différence.
	const secondForeign = creditCol.fees.some((f) => {
		const a = accountOf(ctx, f.accountId);
		return (
			a !== undefined &&
			accountCurrency(a) !== null &&
			a.id !== to.id &&
			!(a.accountType === 'Expense' && accountCurrency(a) === toCur)
		);
	});
	if (secondForeign && difference !== null) {
		return { kind: 'blocked', blockers: ['fee-second-foreign-account'], pairOrigin, holderNumber: to.number };
	}

	const fmt = (b: Big) => b.toFixed(2);
	const fromF = debitCol.foreign!;
	const toF = creditCol.foreign!;
	const preview1: PreviewLine[] = [
		{
			accountId: from.id,
			debit: '',
			credit: fmt(chf1.value),
			foreign: normalized(fromF),
			role: 'currency',
		},
		...debitCol.fees
			.filter((f) => f.accountId !== null)
			.map((f, k): PreviewLine => ({
				accountId: f.accountId!,
				debit: fmt(fees1Ok[k].chf),
				credit: '',
				foreign: fees1Ok[k].foreign
					? { currency: fees1Ok[k].foreign!.currency, amount: fees1Ok[k].foreign!.amount.toFixed(), rate: fees1Ok[k].foreign!.rate.toFixed() }
					: null,
				role: 'fee',
			})),
		{ accountId: transit, debit: fmt(transitChf), credit: '', foreign: null, role: 'transit' },
	];
	const preview2: PreviewLine[] = [
		{
			accountId: to.id,
			debit: fmt(chf2.value),
			credit: '',
			foreign: normalized(toF),
			role: 'currency',
		},
		...creditCol.fees
			.filter((f) => f.accountId !== null)
			.map((f, k): PreviewLine => ({
				accountId: f.accountId!,
				debit: fmt(fees2Ok[k].chf),
				credit: '',
				foreign: fees2Ok[k].foreign
					? { currency: fees2Ok[k].foreign!.currency, amount: fees2Ok[k].foreign!.amount.toFixed(), rate: fees2Ok[k].foreign!.rate.toFixed() }
					: null,
				role: 'fee',
			})),
		{ accountId: transit, debit: '', credit: fmt(transitChf), foreign: null, role: 'transit' },
		...(difference
			? [
					{
						accountId: difference.accountId,
						debit: difference.side === 'debit' ? difference.amount : '',
						credit: difference.side === 'credit' ? difference.amount : '',
						foreign: null,
						role: 'difference' as const,
					},
				]
			: []),
	];

	/** Le corps d'une écriture depuis son aperçu (devise toujours envoyée). */
	const toLines = (preview: PreviewLine[], col: ConversionColumn): CreateJournalEntryLineRequest[] =>
		preview.map((l) => {
			const out: CreateJournalEntryLineRequest = {
				accountId: l.accountId,
				debit: l.debit === '' ? '0' : l.debit,
				credit: l.credit === '' ? '0' : l.credit,
			};
			if (l.foreign) {
				out.foreign = {
					currency: l.foreign.currency,
					amount: l.foreign.amount,
					rate: l.foreign.rate,
					rateSource: col.foreign ? rateSourceToSend(col.foreign, col.date, false) : 'manual',
				};
			}
			return out;
		});

	return {
		kind: 'ready',
		entries: [
			{ entryDate: debitCol.date, journal: 'Banque', description: labels.first, lines: toLines(preview1, debitCol) },
			{ entryDate: creditCol.date, journal: 'Banque', description: labels.second, lines: toLines(preview2, creditCol) },
		],
		preview: [preview1, preview2],
		difference,
		pairOrigin: pair.origin,
		holderNumber: to.number,
	};
}

/**
 * Le champ visé par `details.line` d'un refus d'une écriture (rang dans le
 * corps **tel que l'assistant l'a construit**) : le compte en devise, la k-ième
 * ligne de frais, le compte de conversion ou la ligne de différence.
 */
export function conversionField(
	line: number,
	feeCount: number,
): { kind: 'currency' } | { kind: 'fee'; index: number } | { kind: 'transit' } | { kind: 'difference' } {
	if (line <= 1) return { kind: 'currency' };
	if (line <= 1 + feeCount) return { kind: 'fee', index: line - 2 };
	if (line === 2 + feeCount) return { kind: 'transit' };
	return { kind: 'difference' };
}
