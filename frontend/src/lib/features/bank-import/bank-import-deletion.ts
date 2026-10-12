/**
 * Story 27-1 (#632) — supprimer un import bancaire dont aucune transaction
 * n'est rapprochée : la définition de « rapprochée », côté écran.
 */
import type { BankTransactionDto } from './bank-import.types';

/**
 * Une transaction est **rapprochée** si et seulement si son statut n'est pas
 * `pending` **ou** si elle pointe une écriture (`matchedEntryId` non nul).
 * Une transaction rejetée (`pending`, sans lien) ne l'est pas.
 *
 * ⚠️ **Écrite deux fois, et seulement deux** : ici, et côté serveur dans
 * `crates/kesh-db/src/repositories/bank_imports.rs` (`is_reconciled`, appelée
 * par `delete_unreconciled_in_tx`). L'écran n'est qu'une commodité : le serveur
 * seul décide (`409 BANK_IMPORT_HAS_RECONCILED_TRANSACTIONS`).
 */
export function isReconciled(tx: BankTransactionDto): boolean {
	return tx.status !== 'pending' || tx.matchedEntryId !== null;
}

/** Nombre de transactions rapprochées (au sens d'{@link isReconciled}). */
export function reconciledCount(txs: BankTransactionDto[]): number {
	return txs.filter(isReconciled).length;
}
