/**
 * Story 36-3 (#1051, D1) — le prédicat d'ambiguïté des milliers, extrait de la
 * Story 35-6 (`openingAmountKind`, `features/bank-accounts/format.ts`) pour être
 * partagé par les deux grammaires qui l'appliquent :
 *
 * - le solde d'ouverture d'un compte bancaire (`openingAmountKind`, 35-6), dont la
 *   table de cas est `features/bank-accounts/opening-amount-cases.json`, relue par
 *   le test de parité Rust (`crates/kesh-api/src/routes/opening_balances.rs`) ;
 * - la lecture d'un montant du journal (`readJournalAmount`,
 *   `features/journal-entries/balance.ts`), écriture manuelle et assistant TVA achat.
 *
 * Chaque grammaire découpe la saisie à sa façon ; le prédicat ne juge que les
 * parties déjà découpées.
 */

/** Une saisie découpée : partie entière sans séparateur de groupe, fraction telle que saisie. */
export interface AmountParts {
	/** Chiffres de la partie entière, séparateurs de groupe retirés (zéros de tête gardés). */
	integerDigits: string;
	/** Fraction **telle que saisie** (vide sans séparateur décimal). */
	fraction: string;
	/** La partie entière portait un séparateur de groupe (« 1'234 »). */
	grouped: boolean;
}

/**
 * Story 36-3 (stub de la preuve du rouge) — la lecture d'aujourd'hui : rien n'est ambigu.
 */
export function isAmbiguousThousands(_parts: AmountParts): boolean {
	return false;
}
