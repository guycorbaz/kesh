/**
 * Type d'état du formulaire d'écriture.
 *
 * ⛔ **Story 24-4b (#380)** — les trois fonctions de reconstitution
 * (`amountToFieldValue`, `lineResponseToDraft`, `fromJournalEntryResponse`)
 * ont été retirées avec le gel : elles n'existaient que pour pré-remplir le
 * formulaire depuis une écriture existante, et une écriture comptabilisée ne se
 * réécrit plus. Le formulaire est en création seule ; la correction passe par la
 * contre-passation. Leur corps se relit dans l'historique si un jour un statut
 * brouillon les rend utiles — mais l'édition d'un brouillon n'aura ni le même
 * contrat ni les mêmes gardes.
 */

export interface LineDraft {
	accountId: number | null;
	debit: string;
	credit: string;
	/** Projet analytique de la ligne (Epic 19). `null` = non taguée. */
	projectId: number | null;
}
