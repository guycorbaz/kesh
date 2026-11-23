/**
 * Story 38-11d (AC-1) — règles d'écran des comptes de **prestataire** (PayPal
 * en phase 1 ; Wise et Stripe, Epics 39 et 40, n'y ajoutent rien), tenues hors
 * des composants pour être testées seules, à côté de `credit-card.ts`. Le
 * serveur reste l'autorité : chaque règle ici a son refus côté serveur
 * (`routes/bank_accounts.rs`, 38-11b).
 */
import { accountCurrency, type AccountResponse } from '$lib/features/accounts/accounts.types';
import type { BankAccountProvider, PaypalSettings } from './bank-accounts.api';
import type { ProviderApiStatus } from '$lib/features/provider-download/provider-download.api';

/** Ce qu'il faut d'un compte bancaire pour savoir s'il est celui d'un prestataire. */
export interface ProviderSource {
	/** `null` ou absent : compte ordinaire. */
	provider?: BankAccountProvider | string | null;
}

/** Longueur maximale du libellé d'un compte PayPal (38-11b, `PROVIDER_ACCOUNT_MAX_CHARS`). */
export const PAYPAL_ACCOUNT_MAX_CHARS = 120;

/** Le compte bancaire est-il le solde d'un compte PayPal ? (page PayPal) */
export function isPaypal(a: ProviderSource): boolean {
	return a.provider === 'paypal';
}

/** Le compte bancaire est-il celui d'un prestataire, quel qu'il soit ? */
export function isProvider(a: ProviderSource): boolean {
	return a.provider !== null && a.provider !== undefined;
}

/**
 * Les comptes où l'on importe un relevé bancaire, règle une facture, paie un
 * fournisseur, crée un lot `pain.001` ou rapproche par l'écran générique :
 * **jamais** celui d'un prestataire (patron de `withoutCreditCards`). Son relevé
 * s'importe et se rapproche par la page du prestataire.
 */
export function withoutProviders<T extends ProviderSource>(accounts: readonly T[]): T[] {
	return accounts.filter((a) => !isProvider(a));
}

/**
 * Forme de comparaison d'un libellé de compte, comme la collation `utf8mb4_unicode_ci`
 * du serveur : sans casse et sans accents (« Zürich » = « Zurich »).
 */
export function accountLabelKey(label: string): string {
	return label.normalize('NFD').replace(/\p{M}/gu, '').toLowerCase();
}

/**
 * Les libellés de compte PayPal **distincts** portés par les comptes chargés,
 * triés — `<datalist>` du formulaire, sélecteur de la page PayPal.
 */
export function paypalAccountLabels(
	accounts: readonly { provider?: string | null; paypal?: PaypalSettings | null }[],
): string[] {
	// Sans casse ni accents, comme le serveur (collation `_ci`) : le premier libellé rencontré est gardé.
	const labels = new Map<string, string>();
	for (const a of accounts) {
		if (isPaypal(a) && a.paypal) {
			const key = accountLabelKey(a.paypal.account);
			if (!labels.has(key)) labels.set(key, a.paypal.account);
		}
	}
	return [...labels.values()].sort((x, y) => x.localeCompare(y));
}

/**
 * Les comptes offerts comme **compte de frais** d'un compte PayPal : actifs,
 * imputables, de type charge, en CHF ou dans la devise `ledgerCurrency` du
 * compte du plan choisi (`null` = CHF) — le contrôle de 38-11b
 * (`PROVIDER_FEE_ACCOUNT_INVALID`), triés par numéro.
 */
export function feeAccountCandidates(
	accounts: readonly AccountResponse[],
	ledgerCurrency: string | null,
): AccountResponse[] {
	return accounts
		.filter((a) => {
			if (!a.active || !a.postable || a.accountType !== 'Expense') return false;
			const currency = accountCurrency(a);
			return currency === null || currency === ledgerCurrency;
		})
		.sort((a, b) => a.number.localeCompare(b.number));
}

/**
 * La ligne qui suit le nom d'un compte PayPal dans la liste des comptes
 * bancaires : « PayPal · <compte> · <devise> », la devise lue au compte du plan
 * lié (CHF s'il n'en porte pas). `null` hors d'un compte PayPal.
 */
export function paypalSummaryLine(
	a: { provider?: string | null; paypal?: PaypalSettings | null; journalAccountId: number | null },
	accounts: readonly AccountResponse[],
): string | null {
	if (!isPaypal(a) || !a.paypal) return null;
	const ledger = accounts.find((x) => x.id === a.journalAccountId);
	const currency = (ledger && accountCurrency(ledger)) ?? 'CHF';
	return `PayPal · ${a.paypal.account} · ${currency}`;
}

/**
 * La charge `paypal` du formulaire : libellé rogné et compte de frais. `null`
 * si le libellé est vide (le formulaire refuse alors avant l'envoi).
 */
export function paypalPayload(account: string, feeAccountId: number | null): PaypalSettings | null {
	const label = account.trim();
	return label === '' ? null : { account: label, feeAccountId };
}

/**
 * Story 38-12b (AC-1) — l'état du téléchargement par l'API du compte PayPal
 * `label` (sans casse ni accents, comme le serveur), lu sur ses comptes
 * bancaires (`providerApi` de la liste) : configuré si l'un d'eux l'est, le mode
 * du premier configuré ; `null` si aucun compte ne porte ce libellé. Tous les
 * soldes d'un même libellé partagent la même série : ils disent la même chose.
 */
export function paypalApiStatus(
	accounts: readonly { provider?: string | null; paypal?: PaypalSettings | null; providerApi?: ProviderApiStatus | null }[],
	label: string,
): ProviderApiStatus | null {
	const key = accountLabelKey(label);
	const mine = accounts.filter((a) => isPaypal(a) && a.paypal && accountLabelKey(a.paypal.account) === key);
	if (mine.length === 0) return null;
	const configured = mine.find((a) => a.providerApi?.configured);
	return configured?.providerApi ?? { configured: false };
}
