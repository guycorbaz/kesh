/**
 * Dette i18n connue — clés demandées par le frontend et absentes des QUATRE catalogues.
 *
 * ⚠️ **Cette liste ne peut que DÉCROÎTRE.** La garde `i18n-keys.test.ts` échoue dans les
 * deux sens : si une clé demandée manque et n’est pas ici, et si une entrée d’ici est
 * désormais présente au catalogue ou n’est plus demandée par personne.
 *
 * ⚠️ **Une entrée « plus demandée » a DEUX causes, et la seconde se vérifie d’abord** :
 * la feature a été retirée (retirer la ligne), ou **l’extracteur ne la voit plus**
 * (réparer l’extracteur). Sur cette story, l’extracteur a successivement raté 29 clés
 * portées par des relais, puis 6 portées par des tables de données. Supprimer la ligne
 * serait effacer la dette et sortir de couverture le code qui la demande.
 *
 * Résorption : story 23-1b (contacts, 20 clés), puis 23-2 à 23-6 — cf. le plan
 * `_bmad-output/planning-artifacts/epic-23-dette-i18n.md`.
 */
export const DETTE_CONNUE: readonly string[] = [
	// ── Littéraux directs — résorbés par 23-1b et 23-3, puis 23-4 à 23-6 (160) ──
	// Demandés par un littéral au site d’appel, ou via un relais local (D4-bis).
	'bank-accounts-error-qr-iban-not-qr',
	'bank-accounts-help-qr-iban',
	'common-back',
	'credit-notes-col-date',
	'credit-notes-col-description',
	'credit-notes-col-line-total',
	'credit-notes-col-number',
	'credit-notes-col-qty',
	'credit-notes-col-status',
	'credit-notes-col-total',
	'credit-notes-col-unit-price',
	'credit-notes-col-vat',
	'credit-notes-confirm-body',
	'credit-notes-create-button',
	'credit-notes-create-error',
	'credit-notes-created',
	'credit-notes-download-pdf',
	'credit-notes-empty',
	'credit-notes-title',
	'credit-notes-view-entry',
	'credit-notes-view-invoice',
	'credit-notes-view-list',
	'invoice-detail-project-label',
	'invoice-field-project',
	'invoice-project-archived',
	'invoice-project-current',
	'invoice-project-none',
	'journal-entry-form-col-project',
	'journal-entry-project-archived',
	'journal-entry-project-none',
	'reconciliation-manual-project-label',
	'reconciliation-manual-project-none',
	'reconciliation-rules-default-project-archived',
	'reconciliation-rules-default-project-none',
	'reconciliation-rules-labels-default-project',
	'reconciliation-split-add-line',
	'reconciliation-split-bank-account-not-configured',
	'reconciliation-split-error-account-required',
	'reconciliation-split-error-amount-positive',
	'reconciliation-split-error-description-too-long',
	'reconciliation-split-error-max-lines',
	'reconciliation-split-error-min-lines',
	'reconciliation-split-error-no-proposal',
	'reconciliation-split-project-none',
	'reconciliation-split-remove-line',
	'reconciliation-split-th-account',
	'reconciliation-split-th-amount',
	'reconciliation-split-th-description',
	'reconciliation-split-th-project',
	'reconciliation-split-value-date-label',
	'reports-fiscal-year-label',
	'reports-generate',
	'reports-project-expenses-col-account',
	'reports-project-expenses-col-amount',
	'reports-project-expenses-empty',
	'reports-project-expenses-subtotal',
	'reports-project-expenses-title',
	'reports-project-expenses-total',
	'reports-project-mode-cumulative',
	'reports-project-mode-fiscal-year',
	'reports-project-mode-label',
	'reports-project-return-col-cost',
	'reports-project-return-col-net',
	'reports-project-return-col-project',
	'reports-project-return-col-return',
	'reports-project-return-col-revenue',
	'reports-project-return-empty',
	'reports-project-return-title',
	'reports-project-return-total',
	'reports-project-selector-label',
	'reports-project-selector-placeholder',

	// ── Menu principal — table `i18nKey` de routes/(app)/+layout.svelte — résorbés par 23-4 (4) ──
	// ⚠️ Entrées de la BARRE DE NAVIGATION : françaises dans les quatre langues aujourd’hui.

	// ── Onglets de rapport — table `labelKey` de routes/(app)/reports/+page.svelte — résorbés par 23-5 (2) ──
	// Révélés par l’inventaire des sites non résolus (D4-ter).
	'reports-project-expenses',
	'reports-project-return',

];
