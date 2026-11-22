/**
 * Story 38-11d (AC-1) — règles d'écran des comptes de **prestataire** (PayPal
 * en phase 1 ; Stripe depuis la 40-7d, qui y ajoute son compte de versement ;
 * Wise, Epic 39, n'y ajoute rien), tenues hors
 * des composants pour être testées seules, à côté de `credit-card.ts`. Le
 * serveur reste l'autorité : chaque règle ici a son refus côté serveur
 * (`routes/bank_accounts.rs`, 38-11b).
 */
import { accountCurrency, type AccountResponse } from '$lib/features/accounts/accounts.types';
import { bankAccountName } from './format';
import type {
	BankAccountProvider,
	BankAccountSummary,
	PaypalSettings,
	StripeSettings,
} from './bank-accounts.api';

/** Story 40-7d — les prestataires dont l'écran règle les paramètres (Wise n'en a pas en phase 1). */
export type ConfigurableProvider = 'paypal' | 'stripe';

/** Story 40-7d — le nom affiché d'un prestataire (nom propre, non traduit). */
export const PROVIDER_NAMES: Record<ConfigurableProvider, string> = {
	paypal: 'PayPal',
	stripe: 'Stripe',
};
import type { ProviderApiStatus } from '$lib/features/provider-download/provider-download.api';

/** Ce qu'il faut d'un compte bancaire pour savoir s'il est celui d'un prestataire. */
export interface ProviderSource {
	/** `null` ou absent : compte ordinaire. */
	provider?: BankAccountProvider | string | null;
}

/** Longueur maximale du libellé d'un compte de prestataire (38-11b, `PROVIDER_ACCOUNT_MAX_CHARS`). */
export const PROVIDER_ACCOUNT_MAX_CHARS = 120;

/** Longueur maximale du libellé d'un compte PayPal : celle de tout prestataire. */
export const PAYPAL_ACCOUNT_MAX_CHARS = PROVIDER_ACCOUNT_MAX_CHARS;

/** Le compte bancaire est-il le solde d'un compte PayPal ? (page PayPal) */
export function isPaypal(a: ProviderSource): boolean {
	return a.provider === 'paypal';
}

/** Story 40-7d — le compte bancaire est-il le solde d'un compte Stripe ? (page Stripe) */
export function isStripe(a: ProviderSource): boolean {
	return a.provider === 'stripe';
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

/** Ce qu'il faut d'un compte bancaire pour lire ses paramètres de prestataire. */
export interface ProviderSettingsSource {
	provider?: string | null;
	paypal?: PaypalSettings | null;
	stripe?: StripeSettings | null;
}

/**
 * Story 40-7d — les paramètres du prestataire `provider` que porte le compte,
 * `null` s'il n'est pas un compte de ce prestataire.
 */
export function providerSettingsOf(
	a: ProviderSettingsSource,
	provider: ConfigurableProvider,
): PaypalSettings | null {
	if (a.provider !== provider) return null;
	return (provider === 'paypal' ? a.paypal : a.stripe) ?? null;
}

/**
 * Les libellés **distincts** des comptes du prestataire `provider` portés par
 * les comptes chargés, triés — `<datalist>` du formulaire, sélecteur de la page
 * du prestataire. Story 40-7d : généralisé de PayPal à Stripe.
 */
export function providerAccountLabels(
	accounts: readonly ProviderSettingsSource[],
	provider: ConfigurableProvider,
): string[] {
	// Sans casse ni accents, comme le serveur (collation `_ci`) : le premier libellé rencontré est gardé.
	const labels = new Map<string, string>();
	for (const a of accounts) {
		const settings = providerSettingsOf(a, provider);
		if (settings) {
			const key = accountLabelKey(settings.account);
			if (!labels.has(key)) labels.set(key, settings.account);
		}
	}
	return [...labels.values()].sort((x, y) => x.localeCompare(y));
}

/** Les libellés de compte PayPal distincts (voir {@link providerAccountLabels}). */
export function paypalAccountLabels(accounts: readonly ProviderSettingsSource[]): string[] {
	return providerAccountLabels(accounts, 'paypal');
}

/** Story 40-7d — les libellés de compte Stripe distincts (voir {@link providerAccountLabels}). */
export function stripeAccountLabels(accounts: readonly ProviderSettingsSource[]): string[] {
	return providerAccountLabels(accounts, 'stripe');
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

/** La devise d'un compte bancaire, lue à son compte du plan lié (`null` = CHF ou aucun). */
function ledgerCurrencyOf(
	journalAccountId: number | null,
	accounts: readonly AccountResponse[],
): string | null {
	const ledger = accounts.find((x) => x.id === journalAccountId);
	return ledger ? accountCurrency(ledger) : null;
}

/**
 * La ligne qui suit le nom d'un compte de prestataire dans la liste des comptes
 * bancaires : « PayPal · <compte> · <devise> » ou « Stripe · … », la devise lue
 * au compte du plan lié (CHF s'il n'en porte pas). `null` hors d'un compte
 * PayPal ou Stripe. Story 40-7d : généralisé de PayPal à Stripe.
 */
export function providerSummaryLine(
	a: ProviderSettingsSource & { journalAccountId: number | null },
	accounts: readonly AccountResponse[],
): string | null {
	for (const provider of ['paypal', 'stripe'] as const) {
		const settings = providerSettingsOf(a, provider);
		if (settings) {
			const currency = ledgerCurrencyOf(a.journalAccountId, accounts) ?? 'CHF';
			return `${PROVIDER_NAMES[provider]} · ${settings.account} · ${currency}`;
		}
	}
	return null;
}

/** La ligne d'un compte PayPal (voir {@link providerSummaryLine}) ; `null` hors d'un compte PayPal. */
export function paypalSummaryLine(
	a: ProviderSettingsSource & { journalAccountId: number | null },
	accounts: readonly AccountResponse[],
): string | null {
	return isPaypal(a) ? providerSummaryLine(a, accounts) : null;
}

/**
 * Story 40-7d (AC-1) — les comptes offerts comme **compte bancaire de
 * versement** d'un compte Stripe, miroir du contrôle de 40-7b
 * (`STRIPE_PAYOUT_ACCOUNT_INVALID`) : comptes bancaires (non une carte) non
 * archivés ni clos, liés à un compte du plan **de la devise du solde**
 * (`ledgerCurrency`, `null` = CHF), sans prestataire ou Wise — jamais un compte
 * Stripe ni PayPal, ni le compte lui-même (`selfId`). Triés par nom affiché (`bankAccountName`).
 * Revue P1 de 40-7d (M1) : la destination déjà choisie (`keepId`) reste offerte même archivée ou
 * close depuis — un renommage ne l'efface pas (le serveur la garde, `payout_is_unchanged`) ; elle
 * reste soumise aux autres critères (devise, type, prestataire).
 */
export function payoutAccountCandidates(
	bankAccounts: readonly BankAccountSummary[],
	accounts: readonly AccountResponse[],
	ledgerCurrency: string | null,
	selfId: number | null,
	keepId: number | null = null,
): BankAccountSummary[] {
	return bankAccounts
		.filter(
			(b) =>
				b.id !== selfId &&
				(b.id === keepId || (!b.archived && b.closedOn === null)) &&
				b.accountKind === 'bank' &&
				(b.provider === null || b.provider === 'wise') &&
				b.journalAccountId !== null &&
				ledgerCurrencyOf(b.journalAccountId, accounts) === ledgerCurrency,
		)
		.sort((x, y) => bankAccountName(x).localeCompare(bankAccountName(y)));
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
 * Story 40-7d (AC-1) — la charge du prestataire `provider` du formulaire, sous
 * sa clé (`{ paypal }` ou `{ stripe }`, avec le compte de versement pour
 * Stripe). `null` si le libellé est vide.
 */
export function providerPayload(
	provider: ConfigurableProvider,
	account: string,
	feeAccountId: number | null,
	payoutBankAccountId: number | null,
): { paypal: PaypalSettings } | { stripe: StripeSettings } | null {
	const base = paypalPayload(account, feeAccountId);
	if (base === null) return null;
	return provider === 'paypal' ? { paypal: base } : { stripe: { ...base, payoutBankAccountId } };
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
