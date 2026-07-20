/**
 * Calcule le `sent_at` (NaiveDateTime « YYYY-MM-DDTHH:MM:SS », UTC) transmis au
 * backend pour un rappel manuel enregistré à la date `sentAt` (« YYYY-MM-DD »).
 *
 * #259 : le backend rejette un `sent_at` dans le futur (422
 * `REMINDER_DATE_IN_FUTURE`). L'ancien comportement suffixait midi
 * (`${sentAt}T12:00:00`), ce qui produisait un instant **futur** pour la date du
 * jour tant que l'heure UTC courante était < 12:00 → rappel manuel « aujourd'hui »
 * rejeté toute la matinée (UTC). On envoie donc :
 *   - date du jour (UTC) → l'instant `now` courant, tronqué à la seconde, qui est
 *     par construction ≤ now côté backend (jamais futur) ;
 *   - date passée → midi, qui évite tout décalage d'affichage (un fuseau derrière
 *     UTC ne bascule pas sur la veille) sans jamais tomber dans le futur.
 *
 * #249 : le suffixe heure est requis dans les deux cas — une date nue
 * « YYYY-MM-DD » est rejetée à la désérialisation `NaiveDateTime`.
 *
 * @param sentAt date choisie, format « YYYY-MM-DD » (interprétée en UTC, cohérent
 *   avec `todayIso()` qui dérive de `toISOString()`).
 * @param now instant de référence (défaut `new Date()`), injecté pour les tests.
 * @returns chaîne « YYYY-MM-DDTHH:MM:SS » (UTC, sans suffixe de fuseau).
 */
export function reminderSentAtDateTime(sentAt: string, now: Date = new Date()): string {
	const todayUtc = now.toISOString().slice(0, 10);
	if (sentAt === todayUtc) {
		// Instant courant UTC → toujours ≤ now côté backend.
		return now.toISOString().slice(0, 19);
	}
	// Date passée : midi (anti-décalage d'affichage, jamais futur).
	return `${sentAt}T12:00:00`;
}
