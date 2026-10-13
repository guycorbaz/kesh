/**
 * Story 28-5b (#526) — « rejetée », côté écran.
 *
 * Rejeter une proposition de réconciliation (`POST /reconciliation/reject`)
 * pose `autoMatchRejectedAt` : la transaction quitte la liste de la page
 * *Réconciliation*, mais reste dans le détail de son import, où son rejet
 * s'annule (`POST /reconciliation/transactions/{id}/unreject`).
 */
import type { BankTransactionDto } from '$lib/features/bank-import/bank-import.types';
import { isReconciled } from '$lib/features/bank-import/bank-import-deletion';

/**
 * Une transaction est **rejetée** si son marqueur de rejet est posé **et**
 * qu'elle n'est pas rapprochée (au sens d'{@link isReconciled} — pas de
 * seconde définition). Le serveur refuse d'annuler le rejet d'une rapprochée
 * (`409 RECONCILIATION_TRANSACTION_NOT_REJECTED`) : l'écran ne propose pas le
 * geste.
 *
 * ⚠️ `!= null`, et non `!== null` : un champ **absent** (`undefined`, réponse
 * d'un serveur antérieur à la v0.13.3) n'est pas un rejet.
 */
export function isRejected(tx: BankTransactionDto): boolean {
	return tx.autoMatchRejectedAt != null && !isReconciled(tx);
}
