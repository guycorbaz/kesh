/**
 * Story 33-4 (#606) — un filtre de date ne part qu'une fois la date complète.
 *
 * Un `<input type="date">` rend une valeur dès que ses trois segments sont remplis :
 * l'année tapée au clavier passe donc par `0002`, `0020`, `0202` avant `2026`, et Chromium
 * émet `input` et `change` à chaque chiffre. Le serveur refuse ces années (1000 à 9999, la
 * plage de la base) : c'est juste, et ce n'est pas à lui de s'y plier. **C'est l'écran qui
 * n'envoie pas une date incomplète** : tant qu'elle l'est, le filtre garde sa dernière valeur
 * valide, donc la dernière requête, sans erreur ni liste vidée.
 *
 * Effacer le champ retire le filtre ; un champ à demi effacé (segment vidé, `badInput`) rend
 * `''` lui aussi, mais n'est pas une demande de retrait.
 */

/**
 * Vrai si `s` est une date réelle `AAAA-MM-JJ` d'une année entre 1000 et 9999 — la plage
 * que le serveur accepte (`kesh_core::dates`).
 */
export function estDateStockable(s: string | null | undefined): s is string {
	if (!s || !/^\d{4}-\d{2}-\d{2}$/.test(s)) return false;
	const [y, m, d] = s.split('-').map(Number);
	if (y < 1000) return false;
	const dt = new Date(Date.UTC(y, m - 1, d));
	return dt.getUTCFullYear() === y && dt.getUTCMonth() === m - 1 && dt.getUTCDate() === d;
}

/** La valeur d'un champ de filtre : une date complète, `''` s'il est vide, `null` si incomplet. */
function valeurDuChamp(input: HTMLInputElement): string | null {
	if (input.value === '') return input.validity?.badInput ? null : '';
	return estDateStockable(input.value) ? input.value : null;
}

/** La saisie d'un filtre : `appliquer` n'est appelé que pour une valeur que le serveur accepte. */
export function saisieDeFiltre(e: Event, appliquer: (date: string) => void): void {
	const valeur = valeurDuChamp(e.target as HTMLInputElement);
	if (valeur !== null) appliquer(valeur);
}

/**
 * La sortie du champ : une date restée incomplète retrouve la valeur du filtre, pour que
 * l'écran ne montre pas une date qui ne filtre rien. Jamais pendant la frappe : réécrire le
 * champ l'interromprait.
 */
export function sortieDeFiltre(e: Event, courante: string): void {
	const input = e.target as HTMLInputElement;
	if (valeurDuChamp(input) === null) input.value = courante;
}

/** La valeur d'un filtre lue à l'adresse : ce que le serveur refuserait n'est pas un filtre. */
export function filtreDeLAdresse(brut: string | null | undefined): string {
	return estDateStockable(brut) ? brut : '';
}
