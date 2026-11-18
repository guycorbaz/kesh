/**
 * Story 43-3 (#823) — les grandeurs de l'écran du règlement groupé, en entiers.
 *
 * Une transaction règle plusieurs factures. L'écran montre le montant du relevé, le total
 * imputé et l'**écart** ; il dit, s'il y a lieu, l'**arrondi** sous le centime. Ce module
 * calcule ces grandeurs sans aucun `Number` sur un montant (fiche, D3) : chaque montant se lit
 * en **unités de 1/10 000**, dans un `bigint` (choix C-e433-5). `0.10 + 0.20` y vaut `0.30`.
 *
 * La règle est celle du serveur (`group_difference`, Story 43-1a ; décision de Guy et de
 * l'orchestrateur du 2026-11-17, `kesh-gate-logs/e43/decision-difference.md`, C-43-25) :
 *
 * - `d = |montant| − Σ restes dus bruts` (`amountDueRaw`), quatre décimales, signé ;
 * - **écart** `e` = `d` arrondi au centime, demi-centime arrondi en s'éloignant de zéro —
 *   en unités : `e = signe(d) × ⌊(|d| + 50) / 100⌋ × 100`, comme `amount_due_to_centime` ;
 * - **arrondi** `r = d − e` (`|r| ≤ 50` unités), passé au compte d'arrondi ;
 * - l'**arrondi automatique** est le cas `e = 0`, soit `|d| < 50` : aucun compte demandé.
 *
 * Seul `|e|` se compare au plafond (`maxDifference`, réglage de la société lu dans la réponse
 * de la recherche, jamais écrit ici). Le compte de l'écart est obligatoire dès que `e ≠ 0` ;
 * un compte d'arrondi désigné l'est dès que `r ≠ 0`.
 */
import type { AccountResponse } from '$lib/features/accounts/accounts.types';
import { counterpartyAccounts, keepOfferedCounterparty } from './reconciliation-counterparties';

/** Unités par franc : un montant se lit en 1/10 000. */
export const UNITS_PER_UNIT = 10_000n;

/** Unités d'un centime. */
const CENTIME = 100n;

/** Le demi-centime, borne de l'arrondi automatique (`|d| < 50`). */
const HALF_CENTIME = 50n;

/**
 * Un montant décimal (`"-1250.00"`, `"10.0050"`) en unités de 1/10 000, ou `null` s'il est
 * illisible : forme `-?\d+(\.\d{1,4})?`, plus de quatre décimales refusées.
 */
export function parseUnits(value: string | null | undefined): bigint | null {
	if (typeof value !== 'string') return null;
	const m = /^(-?)(\d+)(?:\.(\d{1,4}))?$/.exec(value.trim());
	if (!m) return null;
	const [, sign, whole, frac = ''] = m;
	const units = BigInt(whole) * UNITS_PER_UNIT + BigInt(frac.padEnd(4, '0'));
	return sign === '-' ? -units : units;
}

/** La valeur absolue d'un `bigint`. */
function abs(u: bigint): bigint {
	return u < 0n ? -u : u;
}

/**
 * Des unités en texte, à `decimals` décimales (2 ou 4), sans signe pour un montant positif ou
 * nul, `-` devant un montant négatif. À deux décimales, la fraction sous le centime est
 * **arrondie** (demi en s'éloignant de zéro) — l'appelant n'y passe que des multiples de 100.
 */
export function formatUnits(units: bigint, decimals: 2 | 4 = 2): string {
	const negative = units < 0n;
	let a = abs(units);
	if (decimals === 2) a = ((a + HALF_CENTIME) / CENTIME) * CENTIME;
	const whole = a / UNITS_PER_UNIT;
	const frac = (a % UNITS_PER_UNIT).toString().padStart(4, '0').slice(0, decimals);
	return `${negative && a !== 0n ? '-' : ''}${whole.toString()}.${frac}`;
}

/** Comme `formatUnits`, avec `+` devant un montant strictement positif (l'arrondi signé). */
export function formatSignedUnits(units: bigint, decimals: 2 | 4 = 4): string {
	return units > 0n ? `+${formatUnits(units, decimals)}` : formatUnits(units, decimals);
}

/**
 * Le reste dû d'une facture tel que l'écran l'affiche : **brut** (quatre décimales) s'il porte
 * une fraction de centime, au centime sinon — la somme affichée est alors celle qui est
 * comparée (fiche, D3).
 */
export function displayDue(raw: string): string {
	const u = parseUnits(raw);
	return u === null ? raw : formatTotal(u);
}

/** Un montant en unités, au centime s'il n'a pas de fraction de centime, brut sinon. */
export function formatTotal(units: bigint): string {
	return units % CENTIME === 0n ? formatUnits(units, 2) : formatUnits(units, 4);
}

/** `d` décomposé : écart au centime `e` et arrondi `r = d − e`. */
export interface Decomposition {
	/** `|montant|`, en unités. */
	transaction: bigint;
	/** `Σ amountDueRaw`, en unités. */
	total: bigint;
	/** `d = transaction − total`. */
	difference: bigint;
	/** `e`, multiple de 100. */
	gap: bigint;
	/** `r = d − e`, `|r| ≤ 50`. */
	rounding: bigint;
}

/** `e` = `d` arrondi au centime, demi-centime en s'éloignant de zéro. */
export function gapOf(difference: bigint): bigint {
	const e = ((abs(difference) + HALF_CENTIME) / CENTIME) * CENTIME;
	return difference < 0n ? -e : e;
}

/**
 * Décompose l'écart d'un groupe. `null` si un montant est illisible (l'écran n'envoie rien
 * qu'il ne sait pas calculer).
 */
export function decompose(
	transactionAmount: string,
	rawDues: readonly string[],
): Decomposition | null {
	const signed = parseUnits(transactionAmount);
	if (signed === null) return null;
	let total = 0n;
	for (const raw of rawDues) {
		const u = parseUnits(raw);
		if (u === null) return null;
		total += u;
	}
	const transaction = abs(signed);
	const difference = transaction - total;
	const gap = gapOf(difference);
	return { transaction, total, difference, gap, rounding: difference - gap };
}

/** La famille du règlement : clients pour un crédit, fournisseurs pour un débit (C-43-15). */
export type GroupFamily = 'client' | 'supplier';

/** Le sens de l'écart, qui choisit le libellé et le préremplissage (C-43-26). */
export type GapSense = 'client-under' | 'client-over' | 'supplier-over' | 'supplier-under';

/**
 * Le sens d'un écart non nul : client, `e < 0` moins-payé, `e > 0` trop-perçu ; fournisseur,
 * `e > 0` débit supérieur aux factures, `e < 0` débit inférieur. `null` si `e = 0`.
 */
export function gapSense(family: GroupFamily, gap: bigint): GapSense | null {
	if (gap === 0n) return null;
	if (family === 'client') return gap < 0n ? 'client-under' : 'client-over';
	return gap > 0n ? 'supplier-over' : 'supplier-under';
}

/** Les comptes désignés qui préremplissent le compte de l'écart. */
export interface DesignatedAccounts {
	roundingAccountId: number | null;
	bankFeesAccountId: number | null;
}

/**
 * Le compte qui préremplit le champ (C-43-26) : frais bancaires pour un moins-payé (client) ou
 * un débit supérieur aux factures (fournisseur), arrondi pour l'autre sens — **seulement s'il
 * figure dans la liste du sélecteur** (actif, imputable, hors banque de la transaction) ;
 * sinon `null`, champ vide : jamais une valeur invisible envoyée (#1000).
 */
export function prefillAccount(
	sense: GapSense | null,
	designated: DesignatedAccounts | null,
	accounts: readonly AccountResponse[],
	bankLedgerAccountId: number | null,
): number | null {
	if (sense === null || designated === null) return null;
	const id =
		sense === 'client-under' || sense === 'supplier-over'
			? designated.bankFeesAccountId
			: designated.roundingAccountId;
	if (id === null) return null;
	return counterpartyAccounts(accounts, bankLedgerAccountId).some((a) => a.id === id) ? id : null;
}

/**
 * Le choix de l'utilisateur dans le sélecteur du compte de l'écart. `byUser` se pose à toute
 * sélection (même du compte prérempli) et au vidage du champ ; tant qu'il est posé, le choix
 * reste — un compte, ou le champ vide —, quel que soit le signe de l'écart (fiche, D2 bis ;
 * le vidage, revue P1, C-e433-13).
 */
export interface AccountChoice {
	byUser: boolean;
	accountId: number | null;
}

/** Aucun choix : le préremplissage s'applique. */
export const NO_CHOICE: AccountChoice = { byUser: false, accountId: null };

/**
 * Le choix après un geste dans le sélecteur ; `null` = champ vidé par l'utilisateur, qui reste
 * vide (le « × » a un effet réel, revue P1, C-e433-13) jusqu'à ce qu'il choisisse un compte.
 */
export function selectAccount(id: number | null): AccountChoice {
	return { byUser: true, accountId: id };
}

/** Le plafond du nombre de factures d'un règlement groupé (43-1a, `too_many`). */
export const MAX_GROUP_INVOICES = 50;

/**
 * Le compte effectivement montré et envoyé (choix C-e433-3) : le choix de l'utilisateur s'il
 * en a fait un — retiré s'il est sorti de la liste (`keepOfferedCounterparty`, #1000) —,
 * sinon le préremplissage du sens courant.
 */
export function effectiveDifferenceAccount(
	choice: AccountChoice,
	prefill: number | null,
	accounts: readonly AccountResponse[],
	bankLedgerAccountId: number | null,
): number | null {
	if (!choice.byUser) return prefill;
	return keepOfferedCounterparty(choice.accountId, accounts, bankLedgerAccountId);
}

/** Ce qui retient *Enregistrer*, dans l'ordre où l'écran le dit. */
export type SubmitBlock =
	| 'too-few'
	| 'unreadable'
	| 'rounding-not-configured'
	| 'gap-too-large'
	| 'account-required';

/** Les entrées de `submitBlock`. */
export interface SubmitInput {
	/** Le nombre de factures imputées. */
	count: number;
	decomposition: Decomposition | null;
	/** Le plafond lu de la réponse de la recherche ; `null` tant qu'aucune n'est arrivée. */
	maxDifference: string | null;
	/**
	 * Le compte d'arrondi désigné : `true` désigné, `false` non désigné, `null` inconnu
	 * (réglages illisibles — le serveur tranche, choix C-e433-2).
	 */
	roundingDesignated: boolean | null;
	/** Le compte de l'écart effectif. */
	differenceAccountId: number | null;
	/**
	 * Le serveur a refusé l'envoi faute de compte de l'écart (`difference_account_required`) :
	 * le compte est demandé même si l'écart calculé à l'écran est nul (montants changés entre la
	 * recherche et l'envoi, relecture impossible — revue P2, L-1).
	 */
	serverWantsAccount?: boolean;
}

/**
 * **Toutes** les causes qui empêchent l'envoi, dans l'ordre où l'écran les dit ; vide si
 * *Enregistrer* est actif (fiche, D3) : au moins deux factures ; si `r ≠ 0`, un compte
 * d'arrondi désigné ; si `e ≠ 0`, `|e| ≤ maxDifference` et un compte choisi. Avec `e = 0`, le
 * plafond ne joue pas, même réglé à 0.00, comme le serveur. Une cause n'en masque aucune autre
 * (revue P1, A M-1, E-M1 : l'écart au-delà du plafond s'annonçait « dans le plafond » sous
 * deux factures). Le compte n'est demandé que si l'écart est dans le plafond, et d'au moins
 * deux factures (sous deux, l'écran n'annonce aucun écart).
 */
export function submitBlocks(input: SubmitInput): SubmitBlock[] {
	const blocks: SubmitBlock[] = [];
	if (input.count < 2) blocks.push('too-few');
	const dec = input.decomposition;
	if (dec === null) return [...blocks, 'unreadable'];
	if (dec.rounding !== 0n && input.roundingDesignated === false) blocks.push('rounding-not-configured');
	if (dec.gap !== 0n) {
		const max = parseUnits(input.maxDifference);
		// Revue P2 (M-1) : un plafond encore non lu (`null` pendant le chargement) n'est pas un
		// montant illisible ; sous deux factures, l'écran n'annonce rien de l'écart.
		if (max === null) {
			if (input.count >= 2) blocks.push('unreadable');
		} else if (abs(dec.gap) > max) blocks.push('gap-too-large');
		else if (input.differenceAccountId === null && input.count >= 2) blocks.push('account-required');
	} else if (input.serverWantsAccount && input.differenceAccountId === null && input.count >= 2) {
		blocks.push('account-required');
	}
	return blocks;
}

/** La première cause qui empêche l'envoi, ou `null` si *Enregistrer* est actif. */
export function submitBlock(input: SubmitInput): SubmitBlock | null {
	return submitBlocks(input)[0] ?? null;
}

/** Le numéro affichable d'une candidate : numéro client, numéro interne, à défaut celui du fournisseur. */
export function candidateNumber(c: {
	kind: 'invoice' | 'supplier_invoice';
	id: number;
	number?: string | null;
	internalNumber?: string | null;
	supplierInvoiceNumber?: string | null;
}): string {
	const n =
		c.kind === 'invoice' ? c.number : (c.internalNumber ?? c.supplierInvoiceNumber ?? null);
	return n !== null && n !== undefined && n.trim() !== '' ? n : `#${c.id}`;
}
