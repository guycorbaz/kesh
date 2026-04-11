/**
 * Debounce helper — pattern standard sans dépendance externe.
 *
 * Story 3.4 : utilisé pour les inputs text de filtre (description,
 * amountMin, amountMax) — évite un burst de requêtes réseau pendant
 * la frappe utilisateur.
 *
 * La fonction retournée expose une méthode `.cancel()` pour annuler
 * le timeout en cours — à appeler au démontage du composant Svelte
 * pour éviter un `loadFiltered()` sur un composant démonté.
 */

export interface DebouncedFn<Args extends unknown[]> {
	(...args: Args): void;
	/** Annule le timeout en cours — à appeler au démontage du composant. */
	cancel(): void;
}

export function debounce<Args extends unknown[]>(
	fn: (...args: Args) => void,
	delay: number
): DebouncedFn<Args> {
	let timeoutId: ReturnType<typeof setTimeout> | null = null;

	const debounced = ((...args: Args) => {
		if (timeoutId) clearTimeout(timeoutId);
		timeoutId = setTimeout(() => {
			timeoutId = null;
			fn(...args);
		}, delay);
	}) as DebouncedFn<Args>;

	debounced.cancel = () => {
		if (timeoutId) {
			clearTimeout(timeoutId);
			timeoutId = null;
		}
	};

	return debounced;
}
