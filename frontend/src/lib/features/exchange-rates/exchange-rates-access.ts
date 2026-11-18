/**
 * Story 37-2 (AC-6, U3, A23) — la condition unique qui gouverne le dialogue du ↑
 * d'une correction de l'AFC (« corriger pour cette société » ou « pour toutes les
 * sociétés sous votre gestion »).
 *
 * `false` aujourd'hui : un utilisateur n'appartient qu'à une société
 * (`users.company_id NOT NULL`, aucune table de rattachement). Le ↑ d'une
 * correction reprend alors la valeur **directement** pour la société. Quand Kesh
 * gérera plusieurs sociétés par utilisateur, cette condition deviendra vraie et
 * le dialogue apparaîtra sans réécrire l'écran.
 */
export const CAN_MANAGE_SEVERAL_COMPANIES = false;

/** Les rôles qui saisissent, reprennent et récupèrent les cours (Consultation : lecture seule). */
export function canEditRates(role: string | null | undefined): boolean {
	return role === 'Admin' || role === 'Comptable';
}
