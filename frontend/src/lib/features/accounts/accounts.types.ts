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
	version: number;
	createdAt: string;
	updatedAt: string;
}

export interface CreateAccountRequest {
	number: string;
	name: string;
	accountType: AccountType;
	parentId?: number | null;
	role?: AccountRole | null;
	postable?: boolean;
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
}

export interface ArchiveAccountRequest {
	version: number;
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
