/**
 * Story 34-3 (#967) — les contreparties d'une écriture manuelle ou d'un éclatement.
 *
 * Décision de Guy (fichier d'epic 34) : tout compte imputable est proposé, **sauf celui de la
 * banque** ; le filtre des classes 5, 6 et 7, recopié jusque-là dans les deux dialogues, est
 * retiré. Aucun rôle ne se déduit du numéro d'un compte (Story 14-3a) : le compte de la banque
 * est écarté par son **identifiant**, que le parent résout (`bank_account.journal_account_id`).
 *
 * La règle s'écrit ici une seule fois, pour `ManualMatchModal` et `TransactionSplitModal`
 * (AC-3, DRY). Le serveur reste juge : il refuse le compte de la banque en 400
 * `counterparty_equals_bank_ledger`, un compte archivé ou d'une autre société en 404, un
 * compte non imputable en 400 `ACCOUNT_NOT_POSTABLE` (`routes/reconciliation.rs`).
 */
import type { AccountResponse, AccountRole } from '$lib/features/accounts/accounts.types';

/**
 * Les comptes proposés comme contrepartie : actifs, imputables, de toute classe, sauf celui
 * de la banque.
 *
 * `bankLedgerAccountId` à `null` (chargement en cours, échec, compte bancaire absent ou non
 * relié) : aucun filtrage par identifiant — la garde serveur tranche (Story 15-6d, repli
 * inchangé). `active` et `postable` sont aussi filtrés par `AccountAutocomplete` ; ils le sont
 * ici pour que la fonction dise, seule, ce que le dialogue propose.
 */
export function counterpartyAccounts(
	accounts: readonly AccountResponse[],
	bankLedgerAccountId: number | null,
): AccountResponse[] {
	return accounts.filter((a) => a.active && a.postable && a.id !== bankLedgerAccountId);
}

/** Les deux rôles qui appellent l'avis débiteurs / créanciers (AC-4). */
export type ClaimRole = Extract<AccountRole, 'Receivable' | 'Payable'>;

/**
 * Story 34-3 (AC-4, C-e342-1) — le rôle de créance du compte choisi, s'il en porte un.
 *
 * Une écriture manuelle sur le compte débiteurs (ou créanciers) ne règle aucune facture : la
 * facture resterait due, sans lettrage, et rien ne le signalerait — le faux rattachement muet
 * du `CLAUDE.md`. Le dialogue avertit donc, sans bloquer. Seul le **rôle explicite** compte :
 * un compte 1100 sans rôle posé, ou un compte de TVA, n'avertit pas (hors périmètre écrit).
 */
export function claimRoleOf(account: AccountResponse | null | undefined): ClaimRole | null {
	const role = account?.role;
	return role === 'Receivable' || role === 'Payable' ? role : null;
}

/**
 * Le rôle de créance du compte `accountId`, lu dans la liste proposée ; `null` si aucun compte
 * n'est choisi ou s'il n'est pas dans la liste. Seule règle d'affichage de l'avis (revue de
 * code P1, B1-1) : le composant d'avis et la rangée d'*Éclater* qui le porte la partagent.
 */
export function claimRoleOfId(
	accounts: readonly AccountResponse[],
	accountId: number | null,
): ClaimRole | null {
	return accountId === null ? null : claimRoleOf(accounts.find((a) => a.id === accountId));
}
