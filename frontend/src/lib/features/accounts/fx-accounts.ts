/**
 * Story 37-1 (AC-4 bis, S2) — comptes de différence de change d'un compte en
 * devise, côté écran.
 *
 * Le réglage de la société (gains, pertes) est le **défaut** ; un compte en
 * devise peut porter ses **propres** comptes (les deux ensemble). La résolution
 * est faite par une seule fonction serveur (`accounts::fx_accounts_for`) et par
 * son miroir {@link fxAccountsFor}, seul lu par le formulaire de compte (et, plus
 * tard, par la ligne d'écart et les assistants de 37-3b). ⚠️ Le miroir ne
 * revalide rien : la revalidation à l'usage reste celle du serveur.
 */

import type { CompanyJson } from '$lib/features/settings/settings.types';
import {
	accountCurrency,
	type AccountResponse,
	type CreateAccountRequest,
	type UpdateAccountRequest,
} from './accounts.types';

/** D'où viennent les comptes résolus : propres au compte, ou réglage de la société. */
export type FxAccountsOrigin = 'own' | 'company';

/** Les comptes de différence de change qu'emploierait une écriture sur un compte en devise. */
export interface FxAccounts {
	gain: number;
	loss: number;
	origin: FxAccountsOrigin;
}

/**
 * Miroir de la fonction serveur : pour un compte **en devise**, ses comptes
 * propres s'il en a (`own`), sinon ceux de la société (`company`) ; `null` pour
 * un compte en CHF, ou si la société n'en désigne pas.
 *
 * Fonction pure : le « par défaut » suit le réglage de la société tel qu'on le
 * lui passe — il suit donc un changement du réglage sans rien stocker.
 */
export function fxAccountsFor(
	account: Pick<AccountResponse, 'currency' | 'fxGainAccountId' | 'fxLossAccountId'>,
	company: Pick<CompanyJson, 'fxGainAccountId' | 'fxLossAccountId'>,
): FxAccounts | null {
	if (accountCurrency(account) === null) return null;
	const ownGain = account.fxGainAccountId ?? null;
	const ownLoss = account.fxLossAccountId ?? null;
	if (ownGain !== null && ownLoss !== null) {
		return { gain: ownGain, loss: ownLoss, origin: 'own' };
	}
	const gain = company.fxGainAccountId ?? null;
	const loss = company.fxLossAccountId ?? null;
	if (gain !== null && loss !== null) {
		return { gain, loss, origin: 'company' };
	}
	return null;
}

/** Commun aux deux filtres : actif, imputable, en CHF. */
function isActivePostableChf(a: AccountResponse): boolean {
	return a.active && a.postable && accountCurrency(a) === null;
}

/**
 * Comptes proposés pour les gains et les pertes de change (AC-3, AC-4 bis) :
 * `Expense` ou `Revenue`, imputables, actifs, en CHF, hors rôle
 * `DefaultRevenue` (les gains de change ne vont pas au chiffre d'affaires).
 * Confort de saisie seulement : le refus qui fait autorité est celui du serveur
 * (`FX_ACCOUNT_INVALID`), qui juge aussi la désignation « compte de produit par
 * défaut » des réglages de facturation et des articles.
 */
export function fxDifferenceAccountOptions(accounts: AccountResponse[]): AccountResponse[] {
	return accounts.filter(
		(a) =>
			isActivePostableChf(a) &&
			(a.accountType === 'Expense' || a.accountType === 'Revenue') &&
			a.role !== 'DefaultRevenue',
	);
}

/**
 * Comptes proposés pour le compte de conversion (AC-3, R7) : compte de bilan
 * (`Asset` ou `Liability`), imputable, actif, en CHF.
 */
export function fxTransitAccountOptions(accounts: AccountResponse[]): AccountResponse[] {
	return accounts.filter(
		(a) => isActivePostableChf(a) && (a.accountType === 'Asset' || a.accountType === 'Liability'),
	);
}

/** Choix du formulaire pour un compte en devise. */
export type FxChoice = 'default' | 'own';

/**
 * L'état des champs « devise » du formulaire de compte. Les identifiants sont
 * des chaînes (valeur d'un `<select>` natif) ; `''` = vide.
 */
export interface AccountCurrencyForm {
	/** Code ISO 4217, `''` = CHF. */
	currency: string;
	fxChoice: FxChoice;
	fxGain: string;
	fxLoss: string;
}

/** L'état initial des champs pour un compte existant (ou `null` : création). */
export function initialCurrencyForm(account: AccountResponse | null): AccountCurrencyForm {
	const gain = account?.fxGainAccountId ?? null;
	const loss = account?.fxLossAccountId ?? null;
	const own = gain !== null && loss !== null;
	return {
		currency: account ? (accountCurrency(account) ?? '') : '',
		fxChoice: own ? 'own' : 'default',
		fxGain: own ? String(gain) : '',
		fxLoss: own ? String(loss) : '',
	};
}

function normalizedCurrency(form: AccountCurrencyForm): string | null {
	const c = form.currency.trim().toUpperCase();
	return c === '' || c === 'CHF' ? null : c;
}

function idOrNull(v: string): number | null {
	return v === '' ? null : Number(v);
}

/**
 * Les comptes propres voulus par le formulaire : `null` = par défaut. Un compte
 * qui reste ou devient CHF n'en a jamais (le serveur refuserait en
 * `reason: "unexpected"`).
 */
function desiredOwn(form: AccountCurrencyForm): { gain: number | null; loss: number | null } | null {
	if (normalizedCurrency(form) === null || form.fxChoice === 'default') return null;
	return { gain: idOrNull(form.fxGain), loss: idOrNull(form.fxLoss) };
}

/**
 * Vrai si le choix « propres » est retenu sans les deux comptes : l'écran le
 * signale au lieu d'envoyer un corps que le serveur refuserait (`missing`).
 */
export function ownAccountsIncomplete(form: AccountCurrencyForm): boolean {
	const own = desiredOwn(form);
	return own !== null && (own.gain === null || own.loss === null);
}

/**
 * Les champs de devise d'une **création** : rien en mono-devise ; la devise si
 * elle n'est pas le CHF ; les comptes propres seulement s'ils sont choisis.
 */
export function currencyFieldsForCreate(
	form: AccountCurrencyForm,
	multiCurrencyEnabled: boolean,
): Pick<CreateAccountRequest, 'currency' | 'fxGainAccountId' | 'fxLossAccountId'> {
	if (!multiCurrencyEnabled) return {};
	const currency = normalizedCurrency(form);
	if (currency === null) return {};
	const own = desiredOwn(form);
	return own === null
		? { currency }
		: { currency, fxGainAccountId: own.gain, fxLossAccountId: own.loss };
}

/**
 * Les champs de devise d'une **modification**, envoyés **seulement s'ils
 * changent** (absents = inchangés) : la devise si elle diffère ; les deux
 * comptes propres ensemble si le choix diffère — `null` pour revenir au défaut,
 * notamment quand la devise est remise à vide sur un compte qui en avait.
 * Rien en mono-devise.
 */
export function currencyFieldsForUpdate(
	account: AccountResponse,
	form: AccountCurrencyForm,
	multiCurrencyEnabled: boolean,
): Pick<UpdateAccountRequest, 'currency' | 'fxGainAccountId' | 'fxLossAccountId'> {
	if (!multiCurrencyEnabled) return {};
	const out: Pick<UpdateAccountRequest, 'currency' | 'fxGainAccountId' | 'fxLossAccountId'> = {};
	const currency = normalizedCurrency(form);
	if (currency !== accountCurrency(account)) out.currency = currency;

	const before = initialCurrencyForm(account);
	const ownBefore =
		before.fxChoice === 'own' ? { gain: Number(before.fxGain), loss: Number(before.fxLoss) } : null;
	const ownAfter = desiredOwn(form);
	const same =
		ownBefore === null
			? ownAfter === null
			: ownAfter !== null && ownAfter.gain === ownBefore.gain && ownAfter.loss === ownBefore.loss;
	if (!same) {
		out.fxGainAccountId = ownAfter?.gain ?? null;
		out.fxLossAccountId = ownAfter?.loss ?? null;
	}
	return out;
}

/** Les champs que peut désigner `details.field` d'un `FX_ACCOUNT_INVALID`. */
export type FxAccountField = 'fxGainAccountId' | 'fxLossAccountId' | 'fxTransitAccountId';

/**
 * Le champ visé par un refus `FX_ACCOUNT_INVALID` (`details.field`), ou `null`
 * si `details` est illisible — l'écran affiche alors le message au pied du
 * formulaire.
 */
export function readFxAccountField(details: Record<string, unknown> | undefined): FxAccountField | null {
	const field = details?.field;
	return field === 'fxGainAccountId' || field === 'fxLossAccountId' || field === 'fxTransitAccountId'
		? field
		: null;
}
