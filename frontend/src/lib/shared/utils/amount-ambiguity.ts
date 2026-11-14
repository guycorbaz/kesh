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
 * `true` si la saisie peut se lire de deux façons : un point ou une virgule suivis
 * d'**exactement trois chiffres**, après une partie entière **non groupée** de un à
 * trois chiffres hors zéros de tête (« 1.234 » : mille deux cent trente-quatre à
 * l'allemande, ou un franc vingt-trois ?). Une partie entière nulle (« 0.500 »), de
 * quatre chiffres ou plus (« 1234.567 ») ou groupée (« 1'234.567 ») n'est pas
 * ambiguë ; une fraction vide (pas de séparateur décimal) non plus.
 *
 * Expression d'origine (35-6, `format.ts`) :
 * `hasDecimal && typedFraction.length === 3 && !grouped && /^0*[1-9][0-9]{0,2}$/.test(integerDigits)`
 * — `hasDecimal` est implicite ici, la fraction étant vide sans séparateur.
 */
export function isAmbiguousThousands(parts: AmountParts): boolean {
	return (
		parts.fraction.length === 3 &&
		!parts.grouped &&
		/^0*[1-9][0-9]{0,2}$/.test(parts.integerDigits)
	);
}
