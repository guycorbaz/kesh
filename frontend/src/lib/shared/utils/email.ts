/**
 * Pré-validation email côté client — Story 17-4d (code review Pass 1 PD2).
 *
 * Miroir **approché** de `is_valid_email_simple` backend
 * (`crates/kesh-api/src/routes/contacts.rs`) : partie locale non vide sans
 * espaces, domaine non vide sans espaces contenant un point. Le backend reste
 * autoritatif (il rejette en plus `..`, point en tête/queue de domaine) — un
 * cas passant ici mais rejeté serveur affiche le 400 localisé.
 *
 * But : stopper côté client les fautes courantes (`@` seul, `a@`, `@b`,
 * domaine sans point) au lieu d'un aller-retour serveur.
 */
export function isPlausibleEmail(email: string): boolean {
	return /^[^\s@]+@[^\s@]+\.[^\s@]+$/.test(email);
}
