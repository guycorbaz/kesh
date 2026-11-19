export type AccountType = 'Asset' | 'Liability' | 'Revenue' | 'Expense';

/**
 * Rôle métier explicite d'un compte (Story 14-3a).
 *
 * Le rôle dit à quoi sert un compte indépendamment de son numéro — le plan
 * comptable suisse est un usage, pas une obligation légale. Miroir de
 * `kesh_db::entities::account::AccountRole`.
 */
export type AccountRole =
	| 'Receivable'
	| 'DefaultRevenue'
	| 'Payable'
	| 'VatRecoverable'
	| 'VatPayable'
	| 'VatSettlement'
	| 'EquityCapital'
	| 'EquityOther'
	| 'RetainedEarnings'
	| 'CurrentYearResult';

/** Les 10 rôles, dans l'ordre d'affichage du sélecteur. */
export const ACCOUNT_ROLES: readonly AccountRole[] = [
	'Receivable',
	'DefaultRevenue',
	'Payable',
	'VatRecoverable',
	'VatPayable',
	'VatSettlement',
	'EquityCapital',
	'EquityOther',
	'RetainedEarnings',
	'CurrentYearResult',
] as const;

/**
 * Clé i18n du libellé d'un rôle : `Receivable` → `account-role-receivable`.
 * Dérivée plutôt que codée dans une table, pour qu'un rôle ajouté à
 * `ACCOUNT_ROLES` ne puisse pas être oublié ici.
 */
export function accountRoleKey(role: AccountRole): string {
	return `account-role-${role.replace(/([a-z0-9])([A-Z])/g, '$1-$2').toLowerCase()}`;
}

export interface AccountResponse {
	id: number;
	companyId: number;
	number: string;
	name: string;
	accountType: AccountType;
	parentId: number | null;
	active: boolean;
	/** Rôle métier explicite, `null` si aucun (Story 14-3a). */
	role: AccountRole | null;
	/** Postabilité — indicatif en 14-3a, appliqué à la saisie par 14-3b. */
	postable: boolean;
	/**
	 * Lettrable (Story 15-1b AC11) — calculé par le serveur
	 * (`letterings::is_letterable`, comptes bancaires exclus) : aucune règle de
	 * lettrabilité n'est recopiée côté client (Story 15-1c-i, AC1).
	 */
	letterable: boolean;
	/**
	 * Story 37-1 (AC-4) — devise du compte (code ISO 4217) ; **absente** pour un
	 * compte en CHF (le serveur omet la clé, C-37-9). Toujours lire
	 * `account.currency ?? null` ({@link accountCurrency}).
	 */
	currency?: string;
	/**
	 * Story 37-1 (AC-4) — le compte porte au moins une ligne d'écriture : sa
	 * devise ne peut plus changer (409 `ACCOUNT_CURRENCY_LOCKED`). Absente =
	 * `false`.
	 */
	currencyLocked?: boolean;
	/**
	 * Story 37-1 (AC-4 bis) — comptes de différence de change **propres** à un
	 * compte en devise ; absents = le compte suit ceux de la société (« par
	 * défaut »). Les deux vont ensemble. Résolus par `fxAccountsFor`.
	 */
	fxGainAccountId?: number;
	fxLossAccountId?: number;
	version: number;
	createdAt: string;
	updatedAt: string;
}

/** Story 37-1 (AC-4) — la devise d'un compte, `null` pour le CHF. */
export function accountCurrency(account: Pick<AccountResponse, 'currency'>): string | null {
	return account.currency ?? null;
}

/**
 * Story 37-3b (AC-4) — **le** prédicat d'un compte en devise : les sélecteurs
 * des pièces (factures, avoirs, règlements, factures fournisseur), lots,
 * réconciliation, comptes bancaires liés, comptes par défaut et produits ne les
 * offrent pas (le serveur les refuserait, 37-3a).
 */
export function isForeignCurrencyAccount(account: Pick<AccountResponse, 'currency'>): boolean {
	return accountCurrency(account) !== null;
}

/** Story 37-3b (AC-4) — la liste sans les comptes en devise (`<select>` natifs, props). */
export function withoutForeignCurrency<T extends Pick<AccountResponse, 'currency'>>(
	accounts: readonly T[],
): T[] {
	return accounts.filter((a) => !isForeignCurrencyAccount(a));
}

export interface CreateAccountRequest {
	number: string;
	name: string;
	accountType: AccountType;
	parentId?: number | null;
	role?: AccountRole | null;
	postable?: boolean;
	/**
	 * Story 37-1 (AC-4) — devise ISO 4217 ; absente ou `null` = CHF. Refusée en
	 * mono-devise (409 `MULTI_CURRENCY_DISABLED`).
	 */
	currency?: string | null;
	/**
	 * Story 37-1 (AC-4 bis) — comptes propres ; absents = par défaut. Envoyés
	 * **ensemble** et seulement s'ils sont posés (le `POST` qui les porte
	 * sérialise sur la société).
	 */
	fxGainAccountId?: number | null;
	fxLossAccountId?: number | null;
}

/**
 * Sémantique full-replace : `role` et `postable` sont **obligatoires**, comme
 * `name` et `accountType`. Les rendre optionnels aurait permis d'effacer le
 * rôle d'un compte en corrigeant simplement son libellé.
 */
export interface UpdateAccountRequest {
	name: string;
	accountType: AccountType;
	role: AccountRole | null;
	postable: boolean;
	version: number;
	/**
	 * Story 25-2-a (#382, #274) — confirme le changement d'`accountType` d'un
	 * compte **qui porte des écritures**. Omis, l'API refuse en 409
	 * `ACCOUNT_HAS_ENTRIES` et renvoie l'ampleur dans `details`.
	 *
	 * ⚠️ Optionnel ici alors que `role` et `postable` sont obligatoires, et
	 * l'asymétrie est voulue : ces deux-là perdent une donnée en silence si on
	 * les omet, tandis que le défaut de celui-ci est le défaut **sûr**.
	 */
	confirmAccountRetype?: boolean;
	/**
	 * Story 37-1 (AC-4) — **absente = devise inchangée**, `null` = CHF, valeur =
	 * devise. Optionnelle, à la différence de `role` : son absence n'efface rien.
	 * Le formulaire ne l'envoie que si elle change.
	 */
	currency?: string | null;
	/**
	 * Story 37-1 (AC-4 bis) — absents = inchangés, `null` = par défaut, valeur =
	 * compte propre. Les deux vont **ensemble** (l'un sans l'autre → 400
	 * `FX_ACCOUNT_INVALID`, `reason: "missing"`), et ne s'envoient que s'ils
	 * changent : le `PUT` qui les porte prend la société `FOR UPDATE`.
	 */
	fxGainAccountId?: number | null;
	fxLossAccountId?: number | null;
	/**
	 * Correctif `fix-api-parent-compte` — **absent = parent inchangé**, `null` =
	 * racine, valeur = nouveau parent (même société, actif ; ni le compte ni
	 * l'un de ses sous-comptes → 409 `ACCOUNT_PARENT_CYCLE`). Le formulaire ne
	 * l'envoie que s'il change (`parentFieldForUpdate`).
	 */
	parentId?: number | null;
}

/**
 * Story 25-2-a — contenu de `details` du 409 `ACCOUNT_HAS_ENTRIES`.
 *
 * ⚠️ `ApiError.details` est un `Record<string, unknown>` : ce type décrit ce que
 * le serveur envoie, il ne le garantit pas. Toute lecture passe par
 * `readRetypeImpact`, qui valide au lieu de supposer.
 */
export interface RetypeImpact {
	entryCount: number;
	closedFiscalYears: string[];
	fromType: string;
	toType: string;
}

/**
 * Lit l'ampleur d'un refus de retypage sans jamais faire confiance à la forme
 * reçue — un `details` absent, partiel ou mal typé rend `null`, et l'écran
 * retombe alors sur le message générique plutôt que d'afficher « undefined
 * écriture(s) ».
 */
export function readRetypeImpact(details: Record<string, unknown> | undefined): RetypeImpact | null {
	if (!details) return null;

	// ⚠️ Les QUATRE champs invalident, pas seulement `entryCount`.
	//
	// La première rédaction ne contrôlait que `entryCount` et dégradait les
	// trois autres en silence — `closedFiscalYears` non-tableau devenait `[]`,
	// `fromType`/`toType` manquants devenaient `''`. Sans conséquence visible,
	// puisque la modale n'affiche aujourd'hui que les deux premiers ; mais le
	// commentaire promettait de « valider au lieu de supposer », et le code ne
	// le faisait qu'au quart. *Relevé en passe 1 de revue.* Un jour où la modale
	// affichera le type de départ, un `''` silencieux produirait une phrase à
	// trou que rien ne signalerait.
	const { entryCount, closedFiscalYears, fromType, toType } = details;
	if (typeof entryCount !== 'number') return null;
	if (!Array.isArray(closedFiscalYears)) return null;
	if (typeof fromType !== 'string' || typeof toType !== 'string') return null;

	return {
		entryCount,
		closedFiscalYears: closedFiscalYears.filter((y): y is string => typeof y === 'string'),
		fromType,
		toType,
	};
}

/**
 * Les codes de réglages d'un refus `ACCOUNT_DESIGNATED_IN_SETTINGS` (Story 33-8,
 * #486), ou `null` si `details` est illisible : l'écran rend alors le message du
 * serveur plutôt qu'une liste à trous.
 */
export function readDesignatedSettings(
	details: Record<string, unknown> | undefined,
): string[] | null {
	const settings = details?.settings;
	if (!Array.isArray(settings) || settings.length === 0) return null;
	if (!settings.every((s) => typeof s === 'string')) return null;
	return settings as string[];
}

export interface ArchiveAccountRequest {
	version: number;
	/** Archiver un compte désigné dans les paramètres de facturation (Story 33-8, #486). */
	confirmDesignated?: boolean;
}

/** Story 14-3a (#269) — réactivation d'un compte archivé. */
export interface ReactivateAccountRequest {
	version: number;
	/**
	 * Réactiver sans le rôle porté au moment de l'archivage (code review 14-3a,
	 * D4). Omis / `false` par défaut : la réactivation échoue si le rôle
	 * singleton a été repris entre-temps. `true` = « réactiver quand même »,
	 * proposé par l'UI après ce refus.
	 */
	clearRole?: boolean;
}
